//! TOML, read and written: the one format pa3ds keeps settings and names in.
//!
//! The player's files are TOML and nothing else (ADR-0031). `tomllib` reads it and the standard
//! library has no writer, so the writing is here — it is a few lines for the shape these files have,
//! which is tables of strings, and hand-writing it is what lets a file carry **comments**. A names
//! file that cannot say which game it belongs to is a file nobody can pick up a year later.
//!
//! Reading never raises at a player. A file that is missing, unreadable or malformed reads as
//! nothing at all: being told the wrong name is a nuisance, and a reader that stops mid-game is not.
//! A single line that is the wrong shape loses itself rather than the file, because these are
//! written by hand as often as by pa3ds.
//!
//! The keys file and the settings file are **filled in** as pa3ds gains keys and settings: a line
//! is added at the end of its table, and nothing else in the file changes (`offered`, `append`). A
//! line the player has commented out counts as there, which is how they turn one off.

use std::ffi::OsString;
use std::path::{Component, Path};
use std::sync::LazyLock;

use indexmap::IndexMap;
use regex::Regex;
use toml::Value;

use crate::py::{self, Error, Kind, Result};

/// What a basic string may not hold as itself. Every other control character goes as `\uXXXX`.
pub const ESCAPES: [(char, &str); 7] = [
    ('\\', "\\\\"),
    ('"', "\\\""),
    ('\u{8}', "\\b"),
    ('\u{c}', "\\f"),
    ('\n', "\\n"),
    ('\r', "\\r"),
    ('\t', "\\t"),
];

/// A TOML file as its top-level tables, or empty when there is nothing readable there.
pub fn read(path: &Path) -> IndexMap<String, Value> {
    load(path).1
}

/// A file's text, its top-level tables, and what stops it being read: "" when nothing does,
/// or where the TOML breaks, "Unterminated string (at line 2, column 21)", for a player to be
/// told. A file that cannot be read is empty tables, as `read` has it.
pub fn load(path: &Path) -> (String, IndexMap<String, Value>, String) {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => return (String::new(), IndexMap::new(), strerror(&error)),
    };
    let Ok(text) = String::from_utf8(bytes) else {
        return (
            String::new(),
            IndexMap::new(),
            "it is not UTF-8 text".to_string(),
        );
    };
    // `Path.read_text` reads with universal newlines: "\r\n" and a lone "\r" are each "\n".
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let (loaded, problem) = parse(&text);
    (text, loaded, problem)
}

/// `OSError.strerror` of what the file system answered.
fn strerror(error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::NotFound => "No such file or directory".to_string(),
        std::io::ErrorKind::PermissionDenied => "Permission denied".to_string(),
        _ => error.to_string(),
    }
}

/// TOML text as its top-level tables, and where it breaks when it does.
///
/// The `toml` crate's words for what is wrong are its own, not `tomllib`'s: the line and the
/// column are the same, and the message before them is the crate's first line.
pub fn parse(text: &str) -> (IndexMap<String, Value>, String) {
    // `tomllib` takes U+FEFF for an invalid statement where the `toml` crate skips it.
    if text.starts_with('\u{feff}') {
        return (
            IndexMap::new(),
            "Invalid statement (at line 1, column 1)".to_string(),
        );
    }
    match text.parse::<toml::Table>() {
        Ok(loaded) => (table(Some(&Value::Table(loaded))), String::new()),
        Err(error) => {
            let mut words = error.message().lines().next().unwrap_or("").chars();
            let said: String = words
                .next()
                .map(|first| first.to_uppercase().chain(words).collect())
                .unwrap_or_default();
            let at = error.span().map_or(0, |span| span.start);
            let before = text.get(..at).unwrap_or(text);
            let line = before.matches('\n').count() + 1;
            let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
            (
                IndexMap::new(),
                format!("{said} (at line {line}, column {column})"),
            )
        }
    }
}

static HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*\[([^\[\]]+)\]\s*(?:#.*)?$").expect("a valid pattern"));

/// The table a header line opens, `pa3ds.ui` for `[pa3ds.ui]`, or None for any other line.
fn header(line: &str) -> Option<String> {
    let found = HEADER.captures(line)?;
    Some(
        found[1]
            .split('.')
            .map(|part| py::strip(part).trim_matches(|c| c == '"' || c == '\''))
            .collect::<Vec<_>>()
            .join("."),
    )
}

/// The one `key = value` a line holds, or None for anything else: a comment, a header, a
/// blank. Each line is parsed whole, so a comment in prose is never mistaken for a setting.
fn entry(line: &str) -> Option<(String, Value)> {
    let parsed = line.parse::<toml::Table>().ok()?;
    if parsed.len() != 1 {
        return None;
    }
    let (key, value) = parsed.into_iter().next()?;
    if matches!(value, Value::Table(_)) {
        return None;
    }
    Some((key, value))
}

/// Every `key = value` line of a file by the table it is under, the ones commented out
/// (`# key = value`) as well as the rest: everything the file has been given, counting what the
/// player put a # in front of to turn it off. Lines above the first header are under "". Pairs,
/// not a table: a key commented out beside the same key in use is two lines, both offered.
pub fn offered(text: &str) -> IndexMap<String, Vec<(String, Value)>> {
    let mut found: IndexMap<String, Vec<(String, Value)>> = IndexMap::new();
    let mut current = String::new();
    for line in py::splitlines(text) {
        if let Some(name) = header(line) {
            current = name;
        } else if let Some(pair) =
            entry(py::strip(line).strip_prefix('#').unwrap_or(py::strip(line)))
        {
            found.entry(current.clone()).or_default().push(pair);
        }
    }
    found
}

/// `text` with `lines` added to the table `name`: after its last line that is not a comment,
/// so a comment above a line stays above it and one introducing the next table stays with that,
/// or straight after its header when it has none. A table the file does not have is added at the
/// end, header and all.
pub fn append<S: AsRef<str>>(text: &str, name: &str, lines: &[S]) -> String {
    let mut rows: Vec<String> = py::splitlines(text).into_iter().map(String::from).collect();
    let mut at: Option<usize> = None;
    let mut current = String::new();
    for (index, row) in rows.iter().enumerate() {
        if let Some(found) = header(row) {
            if found == name {
                at = Some(index);
            }
            current = found;
        } else if current == name && entry(row).is_some() {
            at = Some(index);
        }
    }
    let lines = lines.iter().map(|line| line.as_ref().to_string());
    match at {
        None => {
            rows.extend(["".to_string(), format!("[{name}]")]);
            rows.extend(lines);
        }
        Some(at) => {
            rows.splice(at + 1..at + 1, lines);
        }
    }
    rows.join("\n") + "\n"
}

/// `text` with `line` in place of the one setting `key` in the table `name`: the one in use,
/// or else the first commented out. None when the table has neither, for `append`. Nothing else
/// in the text changes, line endings included.
pub fn replace(text: &str, name: &str, key: &str, line: &str) -> Option<String> {
    let mut rows: Vec<String> = py::splitlines_keepends(text)
        .into_iter()
        .map(String::from)
        .collect();
    let sets = |row: &str| entry(row).is_some_and(|(found, _)| found == key);
    let mut current = String::new();
    let mut chosen: Option<usize> = None;
    for (index, row) in rows.iter().enumerate() {
        // Python's `$` matches before a line's ending and the `regex` crate's does not.
        if let Some(found) = header(py::rstrip(row)) {
            current = found;
        } else if current == name {
            let bare = py::strip(row);
            if sets(bare) {
                chosen = Some(index);
                break;
            }
            if chosen.is_none() && sets(bare.strip_prefix('#').unwrap_or(bare)) {
                chosen = Some(index);
            }
        }
    }
    let chosen = chosen?;
    let ending = &rows[chosen][rows[chosen].trim_end_matches(['\r', '\n']).len()..];
    rows[chosen] = format!("{line}{ending}");
    Some(rows.concat())
}

/// `read` of a file the binary carries, by its text: None where the game has no such file, as a
/// missing file reads as nothing.
pub fn read_text(text: Option<&str>) -> IndexMap<String, Value> {
    let Some(text) = text else {
        return IndexMap::new();
    };
    parse(text).0
}

/// Write a file whole and move it into place; False when it could not be written.
///
/// Whole and then moved because a rename lands mid-game: a half-written file would be read back
/// as no names at all the next time pa3ds started.
pub fn write(path: &Path, text: &str) -> Result<bool> {
    let made = std::fs::create_dir_all(path.parent().unwrap_or(path));
    if made.is_err() {
        return Ok(false);
    }
    let name = name_of(path);
    if name.is_empty() {
        return Err(Error::value(format!("{} has an empty name", repr_of(path))));
    }
    let mut beside_name = name;
    beside_name.push(".new");
    let beside = path.with_file_name(beside_name);
    // `Path.write_text` opens the file in text mode, which writes each "\n" as the platform's line
    // ending: a file pa3ds writes on Windows has CRLF lines, as the Python's did.
    let body = if cfg!(windows) {
        text.replace('\n', "\r\n")
    } else {
        text.to_string()
    };
    if std::fs::write(&beside, body).is_err() || std::fs::rename(&beside, path).is_err() {
        return Ok(false);
    }
    Ok(true)
}

/// Whatever a parser handed back, as a table of named things and nothing else.
pub fn table(value: Option<&Value>) -> IndexMap<String, Value> {
    match value {
        Some(Value::Table(held)) => held
            .iter()
            .map(|(key, held)| (key.clone(), held.clone()))
            .collect(),
        _ => IndexMap::new(),
    }
}

/// The named strings in a table, and nothing else: a number or a table where a name should be
/// is a typo, not a reason to stop reading the rest of the file.
pub fn strings(value: Option<&Value>) -> IndexMap<String, String> {
    table(value)
        .into_iter()
        .filter_map(|(key, held)| match held {
            Value::String(held) if !held.is_empty() => Some((key, held)),
            _ => None,
        })
        .collect()
}

/// One TOML basic string, escaped. A player may call something anything at all, quotation
/// marks and backslashes included, and a name that broke the file would take the rest with it.
pub fn quote(text: &str) -> String {
    let out: String = text
        .chars()
        .map(|character| {
            ESCAPES
                .iter()
                .find(|(escaped, _)| *escaped == character)
                .map(|(_, escape)| escape.to_string())
                .unwrap_or_else(|| escaped(character))
        })
        .collect();
    format!("\"{out}\"")
}

fn escaped(character: char) -> String {
    if (character as u32) < 0x20 || character as u32 == 0x7F {
        format!("\\u{:04x}", character as u32)
    } else {
        character.to_string()
    }
}

/// A table's lines, sorted, so a file written twice from the same names is the same file.
pub fn pairs(named: &IndexMap<String, String>) -> Vec<String> {
    let mut items: Vec<(&String, &String)> = named.iter().collect();
    items.sort();
    items
        .into_iter()
        .map(|(key, value)| format!("{} = {}", quote(key), quote(value)))
        .collect()
}

/// Lines as a comment block. A line of its own for each, since a file is read in a terminal
/// as often as in an editor and neither wraps kindly.
pub fn comment<S: AsRef<str>>(lines: impl IntoIterator<Item = S>) -> Vec<String> {
    lines
        .into_iter()
        .map(|line| py::rstrip(&format!("# {}", line.as_ref())).to_string())
        .collect()
}

/// `path.name`: Rust's `file_name` leaves out `..` and Python's does not.
fn name_of(path: &Path) -> OsString {
    match path.components().next_back() {
        Some(Component::Normal(name)) => name.to_os_string(),
        Some(Component::ParentDir) => OsString::from(".."),
        _ => OsString::new(),
    }
}

/// `repr(path)`, as `with_suffix` words an empty name.
fn repr_of(path: &Path) -> String {
    let class = if cfg!(windows) {
        "WindowsPath"
    } else {
        "PosixPath"
    };
    let shown = path.to_string_lossy().replace('\\', "/");
    let shown = if shown.is_empty() { "." } else { &shown };
    format!("{class}({})", py::repr(shown))
}

/// The `UnicodeDecodeError` that `bytes.decode()` raises for these bytes.
pub(crate) fn decode_error(bytes: &[u8], error: std::str::Utf8Error) -> Error {
    let start = error.valid_up_to();
    let length = error.error_len().unwrap_or(bytes.len() - start);
    let end = start + length;
    let reason = match error.error_len() {
        None => "unexpected end of data",
        Some(_) if length == 1 && !(0xC2..=0xF4).contains(&bytes[start]) => "invalid start byte",
        Some(_) => "invalid continuation byte",
    };
    let message = if length > 1 {
        format!(
            "'utf-8' codec can't decode bytes in position {start}-{}: {reason}",
            end - 1
        )
    } else {
        format!(
            "'utf-8' codec can't decode byte {:#04x} in position {start}: {reason}",
            bytes[start]
        )
    };
    Error::new(Kind::UnicodeDecodeError, message)
}
