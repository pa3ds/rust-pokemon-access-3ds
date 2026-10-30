//! One file saying what every key does — pa3ds's keys and the emulator's buttons together.
//!
//! pa3ds takes the keys it is bound to and keeps them from the game (`pa3ds.core.keys`). That
//! hides a problem: **a key pa3ds takes is a button the player has lost.** The user's Azahar
//! happens to have I, J, K and L on the C-stick and O on the debug button, which cost nothing —
//! until M was bound to the navigation mode and M turned out to be Start.
//!
//! So `<pa3ds>/user/keys.toml` holds both halves, written with defaults on the first run and
//! afterwards only **filled in** with the keys pa3ds gains (`fill`):
//!
//! - **`[pa3ds.navigation]`** — key to action. A key that means the same thing in both navigation
//!   modes lives here; a key that does not lives in `[pa3ds.navigation.list]` or
//!   `[pa3ds.navigation.direction]`, which win over it. A key may carry modifiers: `shift+k`.
//! - **`[azahar]`** — button to key, and `[azahar.circle_pad]` / `[azahar.c_stick]` for the two
//!   sticks, which hold four directions, a modifier and a scale rather than one key. pa3ds patches
//!   these into Azahar's `qt-config.ini` at startup. It ships **empty**, so nothing is rebound
//!   until the player asks: this machine's config is hand-tuned, and a file that silently rewrote
//!   it would be the opposite of useful. The bindings Azahar has are written in as a comment,
//!   which is the only way to read them without Azahar's own settings dialog.
//!
//! Whatever `[azahar]` says, pa3ds reads the emulator's real bindings and **says which buttons its
//! keys are sitting on**. It does not refuse to bind them: a player who wants K on the C-stick may
//! have it, and being told is what they were missing.
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
//! A key pa3ds gains is added to the player's file at the next start, unless its key is already
//! one of theirs: then it is added **commented out**, with a comment saying what has the key, and
//! said, interrupting, for the player to choose (Orion, 2026-10-17, reopening ADR-0022's "never
//! written into a file that exists"). A key commented out counts as there, which is how a player
//! unbinds one on purpose.
//!
//! Nothing here raises at a player. A keys file that is missing or malformed reads as the
//! defaults, and a config that cannot be patched is said and left alone. A keys file that will not
//! parse is left alone and said, interrupting.
//!
//! Bound by:
//! - ADR-0022: `keys.toml` is the player's: a key gained is added at its table's end, commented out
//!   where it conflicts.
//! - ADR-0026: Azahar's settings are written only while it is closed, a portable Azahar's through
//!   `azahar_folder`.
//! - ADR-0032: everything the player owns is under `<pa3ds>/user`, which `PA3DS_HOME` moves; never
//!   `%APPDATA%`.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use indexmap::{IndexMap, IndexSet};
use regex::Regex;

use crate::core::config::Value;
use crate::core::keys::{ORDER, PUNCTUATION};
use crate::core::words::own;
use crate::core::{toml, window};
use crate::py::{self, Error, Kind, Result};

/// Mode -> key -> action, as `common.field.navigate` defines the modes and the actions. Nothing here
/// knows what an action means; this module only decides which keys carry them.
pub type Bindings = IndexMap<String, IndexMap<String, String>>;

pub const FILE_NAME: &str = "keys.toml";
pub const SECTION: &str = "pa3ds";
/// The one group of pa3ds's own keys there is so far. Its sub-tables are the navigation modes,
/// and the keys directly under it are the ones that mean the same thing in both.
pub const GROUP: &str = "navigation";
pub const EMULATOR_SECTION: &str = "azahar";

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

/// Controls whose loss is logged but not said. ORAS uses none of them — the C-stick does nothing
/// in it, and the debug and GPIO 14 buttons are the emulator's own — so taking one costs the player
/// nothing, and saying so at every start would be noise. Everything else is a button somebody
/// presses, and M turning out to be Start is exactly what this is for.
pub const QUIET: [&str; 3] = ["c_stick", "button_debug", "button_gpio14"];

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
fn controls(name: &str) -> Option<&'static str> {
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

/// A bound name without its modifiers: `shift+k` is still the k key as far as Azahar is
/// concerned, because the emulator watches the key and not what is held with it.
pub fn bare(name: &str) -> String {
    let mut name = name;
    for modifier in ORDER {
        name = name.strip_prefix(&format!("{modifier}+")).unwrap_or(name);
    }
    name.to_string()
}

/// Somewhere else to keep the player's things, for a test and for an agent's session: a run from
/// a checkout would otherwise write into whatever the person at this machine is really playing
/// with.
pub const HOME_ENV: &str = "PA3DS_HOME";

/// The folder pa3ds itself is in: beside the executable once there is one, and the checkout
/// otherwise.
///
/// There is always an executable here, where Python's `sys.frozen` is only set in a build.
pub fn app_path() -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    Ok(executable
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default())
}

/// `<pa3ds>/user`: the folder pa3ds keeps the player's own things in — the log, the keys file,
/// and the names they have given (`pa3ds.core.names`).
///
/// **Beside pa3ds, never in `%APPDATA%` and never under Program Files** (ADR-0032, the user
/// 2026-09-22): pa3ds is meant to sit next to its emulator in a folder that can be copied to
/// another machine whole, names and keys and all. `PA3DS_HOME` moves it, which is what a test
/// wants and what keeps an agent's run off the player's real files.
pub fn home_path() -> Result<PathBuf> {
    if let Some(moved) = std::env::var_os(HOME_ENV).filter(|moved| !moved.is_empty()) {
        return Ok(PathBuf::from(moved));
    }
    Ok(app_path()?.join("user"))
}

/// `<pa3ds>/data`: the words pa3ds ships, beside the player's folder, a game a folder and a
/// language a folder inside it, and `common` for what every game says alike (ADR-0040). Shipped
/// whole with each release, and replaced whole by an update: nothing in it is the player's.
///
/// A build run where it was built, `cargo run`, `cargo test` or `tape.exe` under `target\`, has
/// no folder beside it and reads the checkout's, which `tools/sync.py mirror` keeps as upstream's:
/// Python's `app_path` is the checkout's when there is no executable.
pub fn data_path() -> Result<PathBuf> {
    let beside = app_path()?.join("data");
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    Ok(if !beside.is_dir() && checkout.is_dir() {
        checkout
    } else {
        beside
    })
}

/// `<pa3ds>/user/keys.toml`, or under `home` instead.
pub fn keys_path(home: Option<&Path>) -> Result<PathBuf> {
    let home = match home {
        None => home_path()?,
        Some(home) => home.to_path_buf(),
    };
    Ok(home.join(FILE_NAME))
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
    let folders = match LOOKED_IN.with(|looked_in| looked_in.borrow().clone()) {
        Some(folders) => folders,
        None => [window::emulator_folder(window::EMULATOR), Some(app_path()?)],
    };
    for folder in folders.into_iter().flatten() {
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

// ---------------------------------------------------------------- the keys file

/// The keys that mean the same thing in every mode: what the file writes once under
/// `[pa3ds.navigation]` instead of repeating under each mode.
pub fn shared(bindings: &Bindings) -> IndexMap<String, String> {
    let modes: Vec<&IndexMap<String, String>> = bindings.values().collect();
    let Some((first, others)) = modes.split_first() else {
        return IndexMap::new();
    };
    first
        .iter()
        .filter(|(key, action)| others.iter().all(|other| other.get(*key) == Some(*action)))
        .map(|(key, action)| (key.clone(), action.clone()))
        .collect()
}

/// One key's lines: what its action does where the name does not say it all, and the key.
fn line(key: &str, action: &str, explained: &IndexMap<String, String>) -> Vec<String> {
    let said = explained.get(action).filter(|said| !said.is_empty());
    let mut lines = match said {
        Some(said) => toml::comment(py::wrap(said, 94).expect("94 is a positive width")),
        None => Vec::new(),
    };
    lines.push(format!("{} = {}", toml::quote(key), toml::quote(action)));
    lines
}

/// The keys file as it is first written: the defaults, and the emulator's own bindings set
/// down as a comment so a player can read their layout without Azahar's settings dialog.
/// `explained` is what an action does where its name does not say it all, written above each
/// key that does it.
pub fn render(
    bindings: &Bindings,
    emulator: &IndexMap<String, Vec<String>>,
    sticks: Option<&IndexMap<String, IndexMap<String, String>>>,
    explained: Option<&IndexMap<String, String>>,
) -> Result<String> {
    let no_sticks = IndexMap::new();
    let sticks = sticks.unwrap_or(&no_sticks);
    let no_explained = IndexMap::new();
    let explained = explained.unwrap_or(&no_explained);

    let line = |key: &str, action: &str| -> Vec<String> { self::line(key, action, explained) };

    let both = shared(bindings);
    let mut lines: Vec<String> = vec![
        "# What every key does: the keys pa3ds answers and the buttons Azahar is bound to.".into(),
        "# This file is yours: pa3ds adds the keys it gains later and never changes a line you"
            .into(),
        "# have. To unbind a key, put a # in front of it rather than deleting it, or pa3ds adds"
            .into(),
        "# it back.".into(),
        "".into(),
        "# pa3ds takes these while Azahar's window is in front, and the game never sees them."
            .into(),
        r#"# A key may carry modifiers in front of it: "shift+k", "ctrl+alt+k"."#.into(),
        "#".into(),
        "# These mean the same thing in both navigation modes.".into(),
        "".into(),
        format!("[{SECTION}.{GROUP}]"),
    ];
    for (key, action) in &both {
        lines.extend(line(key, action));
    }
    for (mode, keys) in bindings {
        lines.push("".into());
        lines.push(format!(
            "# What a key means in {mode} mode. One here wins over one above."
        ));
        lines.push(format!("[{SECTION}.{GROUP}.{mode}]"));
        for (key, action) in keys {
            if !both.contains_key(key) {
                lines.extend(line(key, action));
            }
        }
    }
    lines.extend([
        "".to_string(),
        "# Buttons to bind in the emulator, written into qt-config.ini when pa3ds starts.".into(),
        "# A button left out keeps whatever Azahar already has, so an empty section changes".into(),
        "# nothing. Azahar saves its own over them when it closes, so close it before starting"
            .into(),
        "# pa3ds after a change here.".into(),
        "#".into(),
        "# What Azahar has now, in the form a line here takes — uncomment one to change it:".into(),
    ]);
    for button in BUTTONS {
        let names = emulator
            .get(&format!("button_{button}"))
            .filter(|names| !names.is_empty());
        lines.push(match names {
            Some(names) => format!("#   {button} = \"{}\"", names[0]),
            None => format!("#   {button} = ?"),
        });
    }
    lines.extend([
        "".to_string(),
        format!("[{EMULATOR_SECTION}]"),
        "".into(),
        "# A stick is four directions, a modifier that moves it halfway while it is held — which"
            .into(),
        "# is how ORAS is walked rather than run — and the scale that halving uses. Each stick"
            .into(),
        "# takes a table of its own, and a setting left out keeps what Azahar has.".into(),
    ]);
    for stick in STICKS {
        let no_settings = IndexMap::new();
        let on = sticks.get(stick).unwrap_or(&no_settings);
        lines.extend(["#".to_string(), format!("# [{EMULATOR_SECTION}.{stick}]")]);
        for name in STICK_KEYS {
            lines.push(match on.get(name) {
                Some(key) => format!("#   {name} = \"{key}\""),
                None => format!("#   {name} = ?"),
            });
        }
        let scale = match py::float_of(on.get(SCALE).map(String::as_str).unwrap_or("0.5")) {
            Ok(scale) => py::format_g(scale),
            // Azahar's file is hand-editable too
            Err(error) if error.is(Kind::ValueError) => "?".to_string(),
            Err(error) => return Err(error),
        };
        lines.push(format!("#   {SCALE} = {scale}"));
    }
    lines.push("".into());
    Ok(lines.join("\n"))
}

/// Whether a parser handed back a table rather than one value. Asked here rather than with a
/// bare `isinstance` so what a caller holds stays plainly unknown until `toml.table` opens it.
fn is_table(value: &::toml::Value) -> bool {
    matches!(value, ::toml::Value::Table(_))
}

/// A table's strings keyed in lower case: a key and a button both name themselves in lower
/// case, and a file written by hand will not always agree.
fn strings(section: Option<&::toml::Value>) -> IndexMap<String, String> {
    toml::strings(section)
        .into_iter()
        .map(|(key, value)| (key.to_lowercase(), value))
        .collect()
}

/// `[pa3ds.navigation]` and its modes, merged: a mode's own key wins over a shared one.
/// `unbound` are the actions pa3ds has that no default key does.
///
/// None is the defaults themselves, which `prepare` tells from a file that happens to say the same
/// (`bindings is defaults` in Python).
fn read_keys(
    path: &Path,
    loaded: &IndexMap<String, ::toml::Value>,
    defaults: &Bindings,
    notes: &mut Vec<String>,
    unbound: &[&str],
) -> Option<Bindings> {
    let group = toml::table(loaded.get(SECTION)).get(GROUP).cloned();
    let Some(group) = group else {
        notes.push(format!(
            "{} has no [{SECTION}.{GROUP}], using the default keys",
            py::path_str(path)
        ));
        return None;
    };
    // Every action pa3ds has, pooled across the modes. A mode names its own, but a player who
    // writes an action under the wrong heading has made a typo, not a new feature, and a typo
    // that reaches the navigator quietly does something else rather than nothing.
    let known: HashSet<&str> = defaults
        .values()
        .flat_map(|keys| keys.values().map(String::as_str))
        .chain(unbound.iter().copied())
        .collect();
    let both = actions(path, Some(&group), &known, notes);
    let mut bindings: Bindings = IndexMap::new();
    for (mode, keys) in toml::table(Some(&group)) {
        if !is_table(&keys) {
            continue;
        }
        if !defaults.contains_key(&mode) {
            notes.push(format!(
                "{}: there is no mode called {}",
                py::path_str(path),
                py::repr(&mode)
            ));
            continue;
        }
        let mut merged = both.clone();
        merged.extend(actions(path, Some(&keys), &known, notes));
        bindings.insert(mode, merged);
    }
    for mode in defaults.keys() {
        if !bindings.contains_key(mode) {
            bindings.insert(mode.clone(), both.clone());
        }
    }
    if !bindings.values().any(|keys| !keys.is_empty()) {
        notes.push(format!(
            "{} binds no keys, using the defaults",
            py::path_str(path)
        ));
        return None;
    }
    Some(bindings)
}

/// A table of key to action, with the actions pa3ds does not have said and dropped.
fn actions(
    path: &Path,
    table: Option<&::toml::Value>,
    known: &HashSet<&str>,
    notes: &mut Vec<String>,
) -> IndexMap<String, String> {
    let mut out = strings(table);
    let unknown: Vec<String> = out
        .iter()
        .filter(|(_, action)| !known.contains(action.as_str()))
        .map(|(key, _)| key.clone())
        .collect();
    for key in unknown {
        notes.push(format!(
            "{}: {key} is set to {}, which pa3ds does not do",
            py::path_str(path),
            py::repr(&out[&key])
        ));
        out.shift_remove(&key);
    }
    out
}

/// `[azahar]`: a button to the key it wants, a stick to the table inside it.
fn read_emulator(
    path: &Path,
    loaded: &IndexMap<String, ::toml::Value>,
    notes: &mut Vec<String>,
) -> Wanted {
    let mut wanted = Wanted::new();
    for (name, value) in toml::table(loaded.get(EMULATOR_SECTION)) {
        let control = name.to_lowercase();
        match &value {
            ::toml::Value::String(value) if BUTTONS.contains(&control.as_str()) => {
                wanted.insert(control, Asked::Key(value.to_lowercase()));
            }
            value if is_table(value) && STICKS.contains(&control.as_str()) => {
                wanted.insert(control, Asked::Stick(stick(value)));
            }
            value if is_table(value) => notes.push(format!(
                "{}: there is no stick called {}",
                py::path_str(path),
                py::repr(&control)
            )),
            _ => notes.push(format!(
                "{}: there is no button called {}",
                py::path_str(path),
                py::repr(&control)
            )),
        }
    }
    wanted
}

/// A stick's settings: its keys by name, and `modifier_scale` as the number Azahar writes,
/// which a player gives as 0.5 and the config keeps as `0.500000`.
fn stick(stick: &::toml::Value) -> IndexMap<String, String> {
    let mut out: IndexMap<String, String> = strings(Some(stick))
        .into_iter()
        .map(|(name, value)| (name.to_lowercase(), value.to_lowercase()))
        .collect();
    let scale = toml::table(Some(stick)).get(SCALE).cloned();
    let scale = match scale {
        Some(::toml::Value::Integer(scale)) => Some(scale as f64),
        Some(::toml::Value::Float(scale)) => Some(scale),
        _ => None,
    };
    if let Some(scale) = scale {
        out.insert(SCALE.to_string(), py::fixed(scale, 6));
    }
    out
}

/// What the keys file settled: the bindings the hook takes, and what to say about it.
#[derive(Debug, Clone, PartialEq)]
pub struct Keys {
    pub bindings: Bindings,
    /// Everything worth logging: the file written, the buttons patched, the buttons taken.
    pub notes: Vec<String>,
    /// The one sentence worth saying out loud, or "" when there is nothing the player must
    /// hear.
    pub said: String,
    /// Whether `said` has something the player must act on: a new key left commented out, or a
    /// file that will not parse.
    pub warning: bool,
}

/// A key `fill` added to the file, and the key of the player's that kept it commented out.
#[derive(Debug, Clone, PartialEq)]
pub struct Added {
    pub table: String,
    pub key: String,
    pub action: String,
    /// The action the player already has on `key`, or None when the key was free.
    pub taken: Option<String>,
}

/// The keys file with every key pa3ds has and the file lacks added at the end of its table.
///
/// An action is lacking from a mode when neither `[pa3ds.navigation]` nor the mode's own table
/// has a line for it, commented out or not: one commented out is the player unbinding it. A key
/// every mode lacks, and that is the same key in each, goes under `[pa3ds.navigation]` once; the
/// rest under their modes. A key the player already has in a mode it would land in is added
/// commented out, with a comment saying what has it, so nothing they bound changes meaning.
pub fn fill(
    text: &str,
    loaded: &IndexMap<String, ::toml::Value>,
    defaults: &Bindings,
    explained: Option<&IndexMap<String, String>>,
) -> Result<(String, Vec<Added>)> {
    let no_explained = IndexMap::new();
    let explained = explained.unwrap_or(&no_explained);
    let have = toml::offered(text);
    let group = ::toml::Value::Table(
        toml::table(toml::table(loaded.get(SECTION)).get(GROUP))
            .into_iter()
            .collect(),
    );
    let live: IndexMap<&String, IndexMap<String, String>> = defaults
        .keys()
        .map(|mode| {
            let mut keys = strings(Some(&group));
            keys.extend(strings(toml::table(Some(&group)).get(mode)));
            (mode, keys)
        })
        .collect();

    let offered = |mode: &str, action: &str| -> bool {
        [
            format!("{SECTION}.{GROUP}"),
            format!("{SECTION}.{GROUP}.{mode}"),
        ]
        .iter()
        .any(|name| {
            have.get(name).is_some_and(|pairs| {
                pairs.iter().any(
                    |(_, value)| matches!(value, ::toml::Value::String(value) if value == action),
                )
            })
        })
    };

    let mut lacking: IndexMap<&String, IndexMap<String, String>> = defaults
        .iter()
        .map(|(mode, keys)| {
            let keys = keys
                .iter()
                .filter(|(_, action)| !offered(mode, action))
                .map(|(key, action)| (key.clone(), action.clone()))
                .collect();
            (mode, keys)
        })
        .collect();
    // (table, key, action, the modes it lands in).
    let mut adding: Vec<(String, String, String, Vec<&String>)> = Vec::new();
    for (key, action) in shared(defaults) {
        if defaults
            .keys()
            .all(|mode| lacking[mode].get(&key) == Some(&action))
        {
            for mode in defaults.keys() {
                lacking[mode].shift_remove(&key);
            }
            adding.push((
                format!("{SECTION}.{GROUP}"),
                key,
                action,
                defaults.keys().collect(),
            ));
        }
    }
    for (mode, keys) in &lacking {
        for (key, action) in keys {
            adding.push((
                format!("{SECTION}.{GROUP}.{mode}"),
                key.clone(),
                action.clone(),
                vec![*mode],
            ));
        }
    }

    let mut text = text.to_string();
    let mut added: Vec<Added> = Vec::new();
    for (table, key, action, modes) in adding {
        let taken = modes.iter().find_map(|mode| live[mode].get(&key).cloned());
        let mut lines = line(&key, &action, explained);
        if let Some(taken) = &taken {
            let note = format!(
                "{key} is already {}, so this is commented out: give one of them another key, \
                 then take the # off the line below.",
                toml::quote(taken)
            );
            let last = lines.pop().unwrap_or_default();
            lines.extend(toml::comment(py::wrap(&note, 94)?));
            lines.push(format!("# {last}"));
        }
        text = toml::append(&text, &table, &lines);
        added.push(Added {
            table,
            key,
            action,
            taken,
        });
    }
    Ok((text, added))
}

/// Which of the emulator's controls pa3ds's keys are sitting on: what it is called, which key
/// took it, and whether its loss is one to keep quiet about (`QUIET`).
///
/// A key bound with a modifier takes its button only while the modifier is held, and is not said
/// to sit on it: shift and T, touch mode for a day, was chosen so T stays this machine's D-pad
/// up (Orion, 2026-10-01). Sorted by the control, so the same overlap is reported the same way
/// every run.
pub fn taken(
    bindings: &Bindings,
    emulator: &IndexMap<String, Vec<String>>,
) -> Vec<(String, String, bool)> {
    let mine: HashSet<String> = bindings
        .values()
        .flat_map(|keys| keys.keys().filter(|key| bare(key) == **key).cloned())
        .collect();
    let mut found: Vec<(String, String, bool)> = Vec::new();
    for (control, names) in emulator {
        for name in names {
            if mine.contains(name) {
                found.push((
                    controls(control).unwrap_or(control).to_string(),
                    name.clone(),
                    QUIET.contains(&control.as_str()),
                ));
            }
        }
    }
    found.sort();
    found
}

/// Read the keys file, writing it with the defaults the first time, and patch the emulator.
///
/// `write` off reads and reports without touching either file, which is what a test and a dry
/// run want. `running` says Azahar is open, when the emulator's config is left alone (`save`).
/// `explained` is written above the keys of the actions it explains (`render`).
pub fn prepare(
    defaults: &Bindings,
    home: Option<&Path>,
    config: Option<&Path>,
    write: bool,
    running: bool,
    explained: Option<&IndexMap<String, String>>,
) -> Result<Keys> {
    prepare_with(defaults, home, config, write, running, explained, &[])
}

/// `prepare` with every argument: `unbound` are actions a player may bind that no default key
/// does, never written into the file.
pub fn prepare_with(
    defaults: &Bindings,
    home: Option<&Path>,
    config: Option<&Path>,
    write: bool,
    running: bool,
    explained: Option<&IndexMap<String, String>>,
    unbound: &[&str],
) -> Result<Keys> {
    let path = keys_path(home)?;
    let config = match config {
        None => emulator_config("")?,
        Some(config) => config.to_path_buf(),
    };
    let mut notes: Vec<String> = Vec::new();
    let text = read_config(&config, &mut notes);
    let mut emulator = emulator_keys(&text)?;

    let mut read: Option<Bindings> = None;
    let mut wanted = Wanted::new();
    let mut added: Vec<Added> = Vec::new();
    let mut problem = String::new();
    if !path.exists() {
        if write {
            let parent = path.parent().unwrap_or(&path);
            match std::fs::create_dir_all(parent) {
                Err(error) => notes.push(format!(
                    "{} could not be written, using the default keys: {}",
                    py::path_str(&path),
                    os_error(&error, parent, false)
                )),
                Ok(()) => {
                    let text = render(
                        defaults,
                        &emulator,
                        Some(&emulator_sticks(&text)?),
                        explained,
                    )?;
                    // `Path.write_text` opens the file in text mode, which writes each "\n" as the
                    // platform's line ending: a keys file pa3ds writes on Windows has CRLF lines,
                    // as the Python's did.
                    let body = if cfg!(windows) {
                        text.replace('\n', "\r\n")
                    } else {
                        text
                    };
                    match std::fs::write(&path, body) {
                        Ok(()) => notes.push(format!(
                            "wrote {} with the default keys",
                            py::path_str(&path)
                        )),
                        Err(error) => notes.push(format!(
                            "{} could not be written, using the default keys: {}",
                            py::path_str(&path),
                            os_error(&error, &path, true)
                        )),
                    }
                }
            }
        }
    } else {
        let (mine, mut loaded, found) = toml::load(&path);
        problem = found;
        if !problem.is_empty() {
            notes.push(format!(
                "{} could not be read, using the default keys: {problem}",
                py::path_str(&path)
            ));
        } else {
            let filled;
            (filled, added) = fill(&mine, &loaded, defaults, explained)?;
            for each in &added {
                let held = match &each.taken {
                    None => String::new(),
                    Some(taken) => format!(
                        ", commented out: {} is already {}",
                        each.key,
                        py::repr(taken)
                    ),
                };
                notes.push(format!(
                    "added to {} under [{}]: {} = {}{held}",
                    py::path_str(&path),
                    each.table,
                    toml::quote(&each.key),
                    toml::quote(&each.action)
                ));
            }
            if !added.is_empty() {
                loaded = toml::parse(&filled).0;
                if write && !toml::write(&path, &filled)? {
                    notes.push(format!("{} could not be written", py::path_str(&path)));
                }
            }
            read = read_keys(&path, &loaded, defaults, &mut notes, unbound);
            wanted = read_emulator(&path, &loaded, &mut notes);
        }
    }
    let bindings: &Bindings = read.as_ref().unwrap_or(defaults);

    let mut saved = String::new();
    if !wanted.is_empty() && !text.is_empty() {
        let (patched, changed) = patch(&text, &wanted)?;
        notes.extend(changed);
        saved = save(&config, &text, &patched, write, running, &mut notes);
        if saved == "applied" {
            emulator = emulator_keys(&patched)?;
        }
    }

    let over = taken(bindings, &emulator);
    for (control, key, _) in &over {
        notes.push(format!("pa3ds takes {key}, which is {control}"));
    }
    let mut held: Vec<(&str, &str, &str)> = Vec::new();
    for each in &added {
        if let Some(taken) = &each.taken {
            let one = (each.key.as_str(), each.action.as_str(), taken.as_str());
            if !held.contains(&one) {
                held.push(one);
            }
        }
    }
    let name = py::path_str(Path::new(path.file_name().unwrap_or_default()));
    let mut parts = sentence(&over, &saved);
    for (key, action, taken) in &held {
        parts.push(own(
            "keys-commented-out",
            &[
                ("file", &name),
                ("key", key),
                ("action", action),
                ("taken", taken),
            ],
        ));
    }
    if !held.is_empty() {
        parts.push(own("keys-edit-to-choose", &[("file", &name)]));
    }
    if !problem.is_empty() {
        parts.push(unreadable(&path, &problem));
    }
    Ok(Keys {
        bindings: bindings.clone(),
        notes,
        said: parts.join(". "),
        warning: !held.is_empty() || !problem.is_empty(),
    })
}

/// What is said of a file of the player's that will not parse: which, where, and the two ways
/// out. The settings file says it the same way.
pub fn unreadable(path: &Path, problem: &str) -> String {
    let name = py::path_str(Path::new(path.file_name().unwrap_or_default()));
    own("file-unreadable", &[("file", &name), ("problem", &problem)])
}

/// What is worth saying at startup of the emulator's buttons: which pa3ds has taken, and
/// whether a rebinding is waiting for Azahar to be closed.
fn sentence(over: &[(String, String, bool)], saved: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut loud: Vec<&str> = over
        .iter()
        .filter(|(_, _, quiet)| !quiet)
        .map(|(control, _, _)| control.as_str())
        .collect();
    loud.sort();
    loud.dedup();
    if !loud.is_empty() {
        parts.push(own("keys-taken", &[("controls", &listed(loud))]));
    }
    if saved == "applied" {
        parts.push(own("keys-applied", &[]));
    } else if saved == "waiting" {
        parts.push(own("keys-waiting", &[]));
    }
    parts
}

/// Several things as `A, B and C`: how a list is heard rather than how one is read.
fn listed<'a>(things: impl IntoIterator<Item = &'a str>) -> String {
    let items: Vec<&str> = things.into_iter().collect();
    match items.split_last() {
        None => String::new(),
        Some((only, [])) => only.to_string(),
        Some((last, rest)) => own("both", &[("these", &rest.join(", ")), ("last", last)]),
    }
}
