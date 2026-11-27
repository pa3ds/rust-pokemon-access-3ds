//! Azahar's `qt-config.ini`: where it is, what it binds each control to, and the settings and
//! bindings pa3ds writes into it at startup, for the settings file's `[azahar]`
//! (`pa3ds.core.config`) and the keys file's (`pa3ds.core.keyfile`) alike.
//!
//! Three facts about `qt-config.ini` that everything here turns on:
//!
//! - **`code:` is a Qt key code, not a Windows virtual-key code.** Azahar records `event->key()`
//!   (`citra_qt/configuration/configure_input.cpp`). Letters and digits are the same number in
//!   both — which is why nothing has noticed — but the semicolon is 59 to Qt and 186 to Windows,
//!   and the arrow keys are not in the virtual-key range at all.
//! - **A value is ignored while its `\default` flag is `true`**, and Azahar re-saves the file at
//!   startup. Every binding written here sets `...\default=false` beside it. `spikes/rig.py`
//!   carries the scar of learning that with `frame_limit`.
//! - **A stick is a parameter package**: `key:value` pairs separated by commas, with `$0`, `$1`
//!   and `$2` standing for a colon, a comma and a dollar inside a part
//!   (`common/param_package.cpp`). Azahar holds one in a sorted map, so the pairs come back out in
//!   alphabetical order. pa3ds reads the whole package, changes the directions it was asked for
//!   and writes the rest back untouched — a stick rebuilt from scratch would drop the annotations
//!   Azahar keeps beside each direction.
//!
//! Where `qt-config.ini` is: `%APPDATA%\Azahar\config`, or `user\config` beside `azahar.exe` for a
//! portable Azahar, which Azahar is whenever it finds a `user` folder there. A player with a
//! portable one names its folder in `[pa3ds] azahar_folder` (`emulator_config`, `portable`).
//!
//! Azahar saves `qt-config.ini` over whatever is there when it closes
//! (`GMainWindow::closeEvent`), so a patch written while it runs is lost. pa3ds patches at every
//! start while Azahar is closed; with Azahar open it writes nothing and says to close it and start
//! pa3ds again (`save`).
//!
//! Bound by:
//! - ADR-0026: Azahar's settings are written only while it is closed, a portable Azahar's through
//!   `azahar_folder`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use indexmap::{IndexMap, IndexSet};
use regex::Regex;

use crate::core::config::Value;
use crate::core::keys::PUNCTUATION;
use crate::core::words::own;
use crate::core::{folders, toml, window};
use crate::py::{self, Error, Kind, Result};

/// The Qt key codes that are not just the character's own. `Qt::Key_Left` is `0x01000012`, which
/// is the `16777234` in this machine's circle pad binding. Shift, ctrl and alt are here because a
/// stick's modifier is usually one of them: held down, it moves the stick halfway.
pub const QT_NAMED: [(&str, i64); 21] = [
    ("escape", 0x01000000),
    ("tab", 0x01000001),
    ("backspace", 0x01000003),
    ("return", 0x01000004),
    ("enter", 0x01000004),
    ("keypad enter", 0x01000005),
    ("insert", 0x01000006),
    ("delete", 0x01000007),
    ("home", 0x01000010),
    ("end", 0x01000011),
    ("left", 0x01000012),
    ("up", 0x01000013),
    ("right", 0x01000014),
    ("down", 0x01000015),
    ("pageup", 0x01000016),
    ("pagedown", 0x01000017),
    ("shift", 0x01000020),
    ("ctrl", 0x01000021),
    ("meta", 0x01000022),
    ("alt", 0x01000023),
    ("space", 0x20),
];

/// The punctuation keys pa3ds can name, so a binding cannot ask for a character no key carries.
pub const CHARACTERS: [&str; PUNCTUATION.len()] = {
    let mut out = [""; PUNCTUATION.len()];
    let mut at = 0;
    while at < PUNCTUATION.len() {
        out[at] = PUNCTUATION[at].1;
        at += 1;
    }
    out
};

/// What each of the emulator's controls is called out loud.
pub const CONTROLS: [(&str, &str); 20] = [
    ("button_a", "A"),
    ("button_b", "B"),
    ("button_x", "X"),
    ("button_y", "Y"),
    ("button_up", "up"),
    ("button_down", "down"),
    ("button_left", "left"),
    ("button_right", "right"),
    ("button_l", "L"),
    ("button_r", "R"),
    ("button_zl", "ZL"),
    ("button_zr", "ZR"),
    ("button_start", "Start"),
    ("button_select", "Select"),
    ("button_home", "Home"),
    ("button_power", "Power"),
    ("button_debug", "the debug button"),
    ("button_gpio14", "the GPIO 14 button"),
    ("c_stick", "the C-stick"),
    ("circle_pad", "the circle pad"),
];

// Written out where Python derives it from `CONTROLS`: the names of its `button_` entries, in
// order.
pub const BUTTONS: [&str; 18] = [
    "a", "b", "x", "y", "up", "down", "left", "right", "l", "r", "zl", "zr", "start", "select",
    "home", "power", "debug", "gpio14",
];
pub const STICKS: [&str; 2] = ["circle_pad", "c_stick"];
pub const DIRECTIONS: [&str; 4] = ["up", "down", "left", "right"];
pub const MODIFIER: &str = "modifier";
pub const SCALE: &str = "modifier_scale";
/// What a player may bind a key to on a stick. `modifier_scale` is the fifth setting and is a
/// number, not a key.
pub const STICK_KEYS: [&str; 5] = ["up", "down", "left", "right", MODIFIER];

pub const KEYBOARD: &str = "keyboard";

/// `CONTROLS.get(name)`.
pub(crate) fn controls(name: &str) -> Option<&'static str> {
    CONTROLS
        .iter()
        .find(|(control, _)| *control == name)
        .map(|(_, said)| *said)
}

/// `int(text)` of the digits `\d+` matched, for the callers that cannot raise. A number past
/// `i64` stops at its largest, and so does one `int()` refuses for its length.
fn digits_value(digits: &str) -> i64 {
    py::int_base10(digits).unwrap_or(i64::MAX)
}

/// A key's name as the number Azahar stores, or None for a name no key has.
///
/// Qt numbers a printable key by its unshifted character — `A` is `0x41`, `;` is `0x3b` — and
/// everything else from `0x01000000` up.
pub fn qt_code(name: &str) -> Result<Option<i64>> {
    let key = name.to_lowercase();
    if let Some((_, code)) = QT_NAMED.iter().find(|(named, _)| *named == key) {
        return Ok(Some(*code));
    }
    let mut characters = key.chars();
    let (Some(only), None) = (characters.next(), characters.next()) else {
        return Ok(None);
    };
    if py::is_alpha(only) {
        // Some letters have no single capital ("ß" is "SS"); Qt numbers those by the letter itself.
        let upper: Vec<char> = only.to_uppercase().collect();
        return Ok(Some(i64::from(
            if upper.len() == 1 { upper[0] } else { only } as u32,
        )));
    }
    if py::is_digit(only) || CHARACTERS.contains(&key.as_str()) {
        return Ok(Some(i64::from(only as u32)));
    }
    Ok(None)
}

/// What Azahar's `code:` number is called here, or the number itself when nothing knows it.
pub fn qt_name(code: i64) -> String {
    // `_BY_CODE` is a dict comprehension over `QT_NAMED`, so of two names for one code the
    // later one is kept: 0x01000004 reads back as "enter", not "return".
    if let Some((name, _)) = QT_NAMED.iter().rev().find(|(_, named)| *named == code) {
        return name.to_string();
    }
    if (0x41..=0x5A).contains(&code) {
        return (code as u8 as char).to_ascii_lowercase().to_string();
    }
    if (0x20..0x7F).contains(&code) {
        let character = code as u8 as char;
        if character.is_ascii_digit() || CHARACTERS.contains(&character.to_string().as_str()) {
            return character.to_string();
        }
    }
    format!("code {code}")
}

/// `Path.home()`.
fn home() -> Result<PathBuf> {
    #[cfg(windows)]
    let found = std::env::var_os("USERPROFILE").or_else(|| {
        let path = std::env::var_os("HOMEPATH")?;
        let mut home = std::env::var_os("HOMEDRIVE").unwrap_or_default();
        home.push(path);
        Some(home)
    });
    #[cfg(not(windows))]
    let found = std::env::var_os("HOME");
    match found {
        Some(found) => Ok(PathBuf::from(found)),
        None => Err(Error::new(
            Kind::RuntimeError,
            "Could not determine home directory.",
        )),
    }
}

/// Azahar's `qt-config.ini`: `<folder>\user\config` for a portable Azahar the player named
/// (`[pa3ds] azahar_folder`), which Azahar uses when a `user` folder is beside its executable,
/// and otherwise `%APPDATA%\Azahar\config`, where the player's own Azahar keeps its settings.
/// The rig's fork keeps its own beside the executable and is never named here unless a player
/// chooses it.
pub fn emulator_config(folder: &str) -> Result<PathBuf> {
    if !folder.is_empty() {
        return Ok(py::to_path(folder)
            .join("user")
            .join("config")
            .join("qt-config.ini"));
    }
    let root = match std::env::var_os("APPDATA").filter(|root| !root.is_empty()) {
        Some(root) => PathBuf::from(root),
        None => home()?,
    };
    Ok(root.join("Azahar").join("config").join("qt-config.ini"))
}

thread_local! {
    /// The folders `found_portable` looks in where a test gives its own, in place of the running
    /// Azahar's and pa3ds's: Python's test monkeypatches `window.emulator_folder` and `app_path`.
    pub(crate) static LOOKED_IN: RefCell<Option<[Option<PathBuf>; 2]>> =
        const { RefCell::new(None) };
}

/// The folder of a portable Azahar pa3ds finds with no `azahar_folder` set: the running
/// Azahar's, then the one pa3ds is in; None where neither is one. Settings left in AppData by an
/// Azahar used before were read over a portable one beside pa3ds, and no ROM was found (#14).
pub fn found_portable() -> Result<Option<PathBuf>> {
    let looked_in = match LOOKED_IN.with(|looked_in| looked_in.borrow().clone()) {
        Some(looked_in) => looked_in,
        None => [
            window::emulator_folder(window::EMULATOR),
            Some(folders::app_path()?),
        ],
    };
    for folder in looked_in.into_iter().flatten() {
        if let Ok(found) = portable(&folder) {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

/// The folder of a portable Azahar for a folder the player chose, or why it is not one. The
/// folder holding `azahar.exe` will do, and so will its `user` folder.
pub fn portable(chosen: &Path) -> std::result::Result<PathBuf, String> {
    let is_user = chosen
        .file_name()
        .is_some_and(|name| name.to_string_lossy().to_lowercase() == "user");
    let folder = match chosen.parent() {
        Some(parent) if is_user => parent,
        _ => chosen,
    };
    if !folder.join("azahar.exe").is_file() {
        return Err(own(
            "azahar-folder-no-exe",
            &[("folder", &py::path_str(folder))],
        ));
    }
    if !folder.join("user").is_dir() {
        return Err(own(
            "azahar-folder-not-portable",
            &[("folder", &py::path_str(folder))],
        ));
    }
    Ok(folder.to_path_buf())
}

// ---------------------------------------------------------------- parameter packages

/// One of Azahar's parameter strings as the pairs it holds, unescaped. A pair that is not one
/// pair is dropped, which is what the emulator does with it too.
pub fn unpack(serialized: &str) -> IndexMap<String, String> {
    let mut out = IndexMap::new();
    for pair in serialized.split(',') {
        if let Some((key, value)) = pair.split_once(':') {
            out.insert(unescape(key), unescape(value));
        }
    }
    out
}

/// Pairs back into one of Azahar's parameter strings, in the alphabetical order its own
/// sorted map hands them out in, so a value pa3ds did not change comes back byte for byte.
pub fn pack(params: &IndexMap<String, String>) -> String {
    let mut items: Vec<(&String, &String)> = params.iter().collect();
    items.sort();
    items
        .into_iter()
        .map(|(key, value)| format!("{}:{}", escape(key), escape(value)))
        .collect::<Vec<_>>()
        .join(",")
}

fn unescape(part: &str) -> String {
    part.replace("$0", ":")
        .replace("$1", ",")
        .replace("$2", "$")
}

fn escape(part: &str) -> String {
    part.replace('$', "$2")
        .replace(',', "$1")
        .replace(':', "$0")
}

/// One key as a binding of its own: what a button holds, and what a stick holds for each of
/// its directions.
pub fn keyboard_param(code: i64) -> String {
    pack(&IndexMap::from([
        ("code".to_string(), code.to_string()),
        ("engine".to_string(), KEYBOARD.to_string()),
    ]))
}

/// The key a binding is on, or None when it is not on the keyboard at all.
pub fn keyboard_code(params: &IndexMap<String, String>) -> Result<Option<i64>> {
    let code = params.get("code").map(String::as_str).unwrap_or("");
    // isdecimal, not isdigit: "²" is a digit that `int` refuses.
    let decimal = !code.is_empty() && code.chars().all(py::is_decimal);
    if params.get("engine").map(String::as_str) == Some(KEYBOARD) && decimal {
        Ok(Some(py::int_base10(code)?))
    } else {
        Ok(None)
    }
}

// ---------------------------------------------------------------- the emulator's config

// `\s` as Python reads it in a `str` pattern, which is `str.isspace`: wider than the regex crate's.
const SPACE: &str = concat!(
    r"[\t\n\x0b\x0c\r\x1c-\x1f \x85\xa0\x{1680}\x{2000}-\x{200a}",
    r"\x{2028}\x{2029}\x{202f}\x{205f}\x{3000}]"
);

static PROFILE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"(?m)^profile=(\d+){SPACE}*$")).expect("a fixed pattern")
});
static CONTROL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^profiles\\(\d+)\\(button_[a-z0-9_]+|c_stick|circle_pad)=(.*)$")
        .expect("a fixed pattern")
});

/// A setting's value with the quotes Azahar writes around a parameter string taken off.
fn plain(value: &str) -> &str {
    py::strip(value).trim_matches('"')
}

/// Which `profiles\<n>\` section is in use. `profile=0` chooses `profiles\1\`: the
/// setting counts from zero and the sections count from one.
fn profile(text: &str) -> i64 {
    let chosen = PROFILE.captures(text);
    chosen
        .map(|chosen| digits_value(&chosen[1]))
        .unwrap_or(0)
        .saturating_add(1)
}

/// Every control of the chosen profile with the key names bound to it, sticks included.
///
/// A stick holds four directions and a modifier in one value, so a control maps to a list, in
/// `STICK_KEYS` order rather than the alphabetical one the value is stored in. Anything not on
/// the keyboard — a gamepad, the touch screen — is left out, since a binding elsewhere costs the
/// player no key.
pub fn emulator_keys(text: &str) -> Result<IndexMap<String, Vec<String>>> {
    let wanted = profile(text);
    let mut found: IndexMap<String, Vec<String>> = IndexMap::new();
    for captured in CONTROL.captures_iter(text) {
        let (profile, control, value) = (&captured[1], &captured[2], &captured[3]);
        if digits_value(profile) != wanted {
            continue;
        }
        let params = unpack(plain(value));
        // A button is one binding of its own; a stick holds one per direction, nested inside.
        let mut inside = vec![params.clone()];
        for name in STICK_KEYS {
            inside.push(unpack(params.get(name).map(String::as_str).unwrap_or("")));
        }
        let mut codes = Vec::new();
        for part in &inside {
            if let Some(code) = keyboard_code(part)? {
                codes.push(code);
            }
        }
        let names: IndexSet<String> = codes.into_iter().map(qt_name).collect();
        if !names.is_empty() {
            found.insert(control.to_string(), names.into_iter().collect());
        }
    }
    Ok(found)
}

/// Each stick with the key on each of its settings: `{"c_stick": {"up": "i", ...}}`.
///
/// `emulator_keys` answers "which keys does this control cost me"; this answers "what is it
/// bound to", which is what the keys file has to show for a stick to be editable by hand.
pub fn emulator_sticks(text: &str) -> Result<IndexMap<String, IndexMap<String, String>>> {
    let wanted = profile(text);
    let mut found: IndexMap<String, IndexMap<String, String>> = IndexMap::new();
    for captured in CONTROL.captures_iter(text) {
        let (profile, control, value) = (&captured[1], &captured[2], &captured[3]);
        if digits_value(profile) != wanted || !STICKS.contains(&control) {
            continue;
        }
        let params = unpack(plain(value));
        let mut on: IndexMap<String, String> = IndexMap::new();
        for name in STICK_KEYS {
            let inner = unpack(params.get(name).map(String::as_str).unwrap_or(""));
            if let Some(code) = keyboard_code(&inner)? {
                on.insert(name.to_string(), qt_name(code));
            }
        }
        if let Some(scale) = params.get(SCALE).filter(|scale| !scale.is_empty()) {
            on.insert(SCALE.to_string(), scale.clone());
        }
        if !on.is_empty() {
            found.insert(control.to_string(), on);
        }
    }
    Ok(found)
}

/// The line ending a line was read with, so patching one line does not change the file's.
fn ending(line: &str) -> &'static str {
    for end in ["\r\n", "\n", "\r"] {
        if line.ends_with(end) {
            return end;
        }
    }
    ""
}

/// The emulator's config as lines, so patching changes the lines it means to and leaves the
/// file's own line endings alone.
struct Config {
    profile: i64,
    lines: Vec<String>,
    r#where: HashMap<String, usize>,
}

impl Config {
    fn new(text: &str) -> Self {
        let lines: Vec<String> = py::splitlines_keepends(text)
            .into_iter()
            .map(str::to_string)
            .collect();
        let r#where = lines
            .iter()
            .enumerate()
            .map(|(index, line)| {
                let name = line.split_once('=').map_or(line.as_str(), |(name, _)| name);
                (py::strip(name).to_string(), index)
            })
            .collect();
        Config {
            profile: profile(text),
            lines,
            r#where,
        }
    }

    fn text(&self) -> String {
        self.lines.concat()
    }

    fn setting(&self, control: &str) -> String {
        format!("profiles\\{}\\{control}", self.profile)
    }

    fn value(&self, setting: &str) -> Option<String> {
        let index = *self.r#where.get(setting)?;
        Some(
            plain(
                self.lines[index]
                    .split_once('=')
                    .map_or("", |(_, value)| value),
            )
            .to_string(),
        )
    }

    /// Write a value and force its `\default` flag off; True when something changed.
    ///
    /// The flag is not optional: Azahar ignores a stored value while its flag is true and
    /// re-saves the file at startup, which is how a `frame_limit` of 1000 was silently ignored
    /// for a whole afternoon.
    fn set(&mut self, setting: &str, value: &str, quoted: bool) -> bool {
        let (at, flag_at) = (
            self.r#where.get(setting).copied(),
            self.r#where.get(&format!("{setting}\\default")).copied(),
        );
        let (Some(at), Some(flag_at)) = (at, flag_at) else {
            return false;
        };
        let was = plain(self.lines[at].split_once('=').map_or("", |(_, was)| was));
        let flagged = py::strip(
            self.lines[flag_at]
                .split_once('=')
                .map_or("", |(_, flagged)| flagged),
        );
        if was == value && flagged == "false" {
            return false; // as it was, quotes and all: a start that changes nothing writes nothing
        }
        let written = if quoted {
            format!("\"{value}\"")
        } else {
            value.to_string()
        };
        self.lines[at] = format!("{setting}={written}{}", ending(&self.lines[at]));
        self.lines[flag_at] = format!("{setting}\\default=false{}", ending(&self.lines[flag_at]));
        true
    }
}

/// What one control is asked to be: a button's key, or the settings inside a stick.
#[derive(Debug, Clone, PartialEq)]
pub enum Asked {
    Key(String),
    Stick(IndexMap<String, String>),
}

/// Button to key, and a stick to the settings inside it, as `[azahar]` holds them.
pub type Wanted = IndexMap<String, Asked>;

/// The config with `wanted` written into it, and a line for each control actually changed.
///
/// A control already bound the way it asks for is not rewritten, so a start that changes nothing
/// says nothing and does not tell the player to restart the game for no reason.
pub fn patch(text: &str, wanted: &Wanted) -> Result<(String, Vec<String>)> {
    let mut config = Config::new(text);
    let mut changed = Vec::new();
    for (control, asked) in wanted {
        match asked {
            Asked::Key(asked) => patch_button(&mut config, control, asked, &mut changed)?,
            Asked::Stick(asked) => patch_stick(&mut config, control, asked, &mut changed)?,
        }
    }
    Ok((config.text(), changed))
}

/// The settings of Azahar's among `names` its config has no line for, as `patch_settings`
/// finds them: by name, or under the input profile in use.
pub fn lacks<'a>(text: &str, names: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let config = Config::new(text);
    names
        .into_iter()
        .filter(|name| {
            config.value(name).is_none() && config.value(&config.setting(name)).is_none()
        })
        .map(str::to_string)
        .collect()
}

/// The config with Azahar's own settings written into it (`pa3ds.core.config`), and a line
/// for each changed. A setting is found by its name alone: these are unique across the file's
/// sections, and one the file does not have is said rather than invented. A setting of the
/// input profile in use, as `controller_touch_device` is, is found under its profile.
pub fn patch_settings(text: &str, wanted: &IndexMap<String, Value>) -> (String, Vec<String>) {
    let mut config = Config::new(text);
    let mut changed = Vec::new();
    for (name, asked) in wanted {
        let value = match asked {
            Value::Bool(asked) => asked.to_string(),
            Value::Int(asked) => asked.to_string(),
            Value::Float(asked) => py::float_repr(*asked),
            Value::Str(asked) => asked.clone(),
        };
        let setting = if config.value(name).is_some() {
            name.clone()
        } else {
            config.setting(name)
        };
        if config.value(&setting).is_none() {
            changed.push(format!("Azahar has no setting called {}", py::repr(name)));
        } else if config.set(&setting, &value, matches!(asked, Value::Str(_))) {
            changed.push(format!("Azahar's {name} is now {value}"));
        }
    }
    (config.text(), changed)
}

/// `str(OSError)` as Python words it: errno and its text for what `open` raised, the Windows
/// error for what a direct Windows call (`mkdir`) raised, and the path as `repr` shows it.
pub(crate) fn os_error(error: &std::io::Error, path: &Path, opened: bool) -> String {
    let shown = py::repr(&py::path_str(path));
    let Some(code) = error.raw_os_error() else {
        return error.to_string();
    };
    let words = error.to_string();
    let text = words
        .strip_suffix(&format!(" (os error {code})"))
        .unwrap_or(&words);
    if cfg!(windows) {
        let errno = match code {
            2 | 3 => Some((2, "No such file or directory")),
            5 | 32 | 33 => Some((13, "Permission denied")),
            80 => Some((17, "File exists")),
            112 => Some((28, "No space left on device")),
            _ => None,
        };
        return match errno {
            Some((errno, text)) if opened => format!("[Errno {errno}] {text}: {shown}"),
            _ => format!(
                "[WinError {code}] {}: {shown}",
                text.strip_suffix('.').unwrap_or(text)
            ),
        };
    }
    format!("[Errno {code}] {text}: {shown}")
}

/// Write a patched config, and say what became of it: "applied", "waiting" while Azahar is
/// open, or "" when nothing changed or it could not be written.
///
/// Azahar saves its own settings over the file when it closes, so a patch written while it runs
/// is lost — which is why "restart the game" was the wrong thing to tell the player. Nothing is
/// written while it is open, and the player is told to close it and start pa3ds again.
pub fn save(
    config: &Path,
    text: &str,
    patched: &str,
    write: bool,
    running: bool,
    notes: &mut Vec<String>,
) -> String {
    if patched == text {
        return String::new();
    }
    if running {
        notes.push(format!(
            "{} left alone: Azahar is open, and saves its own settings over the \
             file when it closes",
            py::path_str(config)
        ));
        return "waiting".to_string();
    }
    if write && let Err(error) = std::fs::write(config, patched) {
        notes.push(format!(
            "{} could not be written: {}",
            py::path_str(config),
            os_error(&error, config, true)
        ));
        return String::new();
    }
    "applied".to_string()
}

/// The emulator's config as it is on disk, or "" when it cannot be read.
pub fn read_config(config: &Path, notes: &mut Vec<String>) -> String {
    // newline="" both ways, or Python quietly rewrites a CRLF config as LF on the way in and
    // hands the whole file back changed on the way out.
    let error = match std::fs::read(config) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => return text,
            Err(error) => toml::decode_error(error.as_bytes(), error.utf8_error()).message,
        },
        Err(error) => os_error(&error, config, true),
    };
    notes.push(format!(
        "{} could not be read: {error}",
        py::path_str(config)
    ));
    String::new()
}

fn patch_button(
    config: &mut Config,
    button: &str,
    key: &str,
    changed: &mut Vec<String>,
) -> Result<()> {
    let setting = config.setting(&format!("button_{button}"));
    let code = qt_code(key)?;
    match code {
        None => changed.push(format!("{button}: no key is called {}", py::repr(key))),
        Some(code) => {
            if config.value(&setting).is_none() {
                changed.push(format!("{button}: no {setting} in the config to patch"));
            } else if config.set(&setting, &keyboard_param(code), true) {
                let named = format!("button_{button}");
                let said = controls(&named).ok_or_else(|| Error::key(py::repr(&named)))?;
                changed.push(format!("{said} is now {key}"));
            }
        }
    }
    Ok(())
}

/// One stick, changed in place.
///
/// The whole package is read, the settings asked for are replaced, and everything else is put
/// back as it was. Azahar keeps a per-direction annotation **keyed by that direction's own
/// binding** (`configure_input.cpp`), so a direction that moves takes its annotation with it;
/// rebuilding the value instead would drop whatever a gamepad had left in there.
fn patch_stick(
    config: &mut Config,
    stick: &str,
    asked: &IndexMap<String, String>,
    changed: &mut Vec<String>,
) -> Result<()> {
    let setting = config.setting(stick);
    let Some(current) = config.value(&setting) else {
        changed.push(format!("{stick}: no {setting} in the config to patch"));
        return Ok(());
    };
    let mut params = unpack(&current);
    for (name, value) in asked {
        if name == SCALE {
            params.insert(SCALE.to_string(), value.clone());
            continue;
        }
        let code = qt_code(value)?;
        if !STICK_KEYS.contains(&name.as_str()) {
            changed.push(format!("{stick}: a stick has no {}", py::repr(name)));
        } else if let Some(code) = code {
            let now = keyboard_param(code);
            let was = params.get(name).cloned();
            if let Some(was) = was
                && params.contains_key(&was)
            {
                let held = params.shift_remove(&was).unwrap_or_default();
                params.insert(now.clone(), held); // the annotation follows its direction
            }
            params.insert(name.clone(), now);
        } else {
            changed.push(format!("{stick}: no key is called {}", py::repr(value)));
        }
    }
    if config.set(&setting, &pack(&params), true) {
        let said = asked
            .iter()
            .map(|(name, value)| format!("{name} {value}"))
            .collect::<Vec<_>>()
            .join(", ");
        changed.push(format!(
            "{} is now {said}",
            controls(stick).unwrap_or(stick)
        ));
    }
    Ok(())
}
