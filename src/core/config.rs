//! `<pa3ds>/user/config.toml`: pa3ds's settings, and the emulator settings pa3ds wants (ADR-0026).
//!
//! The file is written with the defaults, each explained in a comment, and afterwards only
//! **filled in**: a setting pa3ds gains is added at its default, and nothing the player has is
//! changed. A line commented out counts as there (`toml.offered`), so a line of `[azahar]` the
//! player wants Azahar to keep its own value for is commented out, not deleted (Orion,
//! 2026-10-17). The keys file is filled in the same way (`pa3ds.core.keyfile`), and stays a file of
//! its own. Its tables:
//!
//! - **`[pa3ds]`**: whether pa3ds answers its keys, how often it reads the game, and the folder of a
//!   portable Azahar, whose `user\config` holds the `qt-config.ini` patched rather than AppData's.
//! - **`[pa3ds.ui]`**: how menus, lists and screens are said.
//! - **`[pa3ds.audio]`**: how loud pa3ds's own sounds are, its tones, footsteps and clock ticks, 1
//!   as loud as the game and Azahar play it; until 2026-10-18 the tones were on or off under
//!   `[pa3ds.ui]`, and a player's value there is carried over (`MOVED`).
//! - **`[pa3ds.navigation]`**: how the map's list and the way to a thing are said.
//! - **`[pa3ds.extras]`**: what the game shows in a way pa3ds cannot read, said from its numbers.
//! - **`[azahar]`**: Azahar's own settings, written into its `qt-config.ini` whenever pa3ds
//!   starts while Azahar is closed: the RPC server pa3ds reads the game through, which Azahar
//!   ships switched off, and what made the game run smoothly on this machine (2026-09-11 and
//!   2026-09-18). None of them changes emulated timing, so none of them breaks a movie
//!   (ADR-0028): `cpu_clock_percentage`, `is_new_3ds` and `lle_applets` are the player's own
//!   business and not here.
//!
//! Settings are a file and not flags (Orion, 2026-09-30): only what a developer passes for one
//! run is a flag. Each table is a dataclass here, so its defaults, the file's lines and what a
//! reader is handed are one thing.
//!
//! Nothing here raises at a player. A file that is missing reads as the defaults, and a setting of
//! the wrong kind or an unknown name is logged and left at its default. A file that will not parse
//! is left alone and read as the defaults too, and said, interrupting: a player who edited it would
//! otherwise wonder why nothing they did took (Orion, 2026-10-17).
//!
//! Bound by:
//! - ADR-0017: an Extra, anything the game does not show, is a toggle of its own under
//!   `[pa3ds.extras]` and ships off.
//! - ADR-0026: a setting works the same in every game or is not one; `config.toml` is filled in,
//!   never rewritten.
//! - ADR-0028: `[azahar]` never sets what changes emulated timing, or every movie desyncs.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::core::keyfile;
use crate::core::toml;
use crate::core::touch;
use crate::core::words::own;
use crate::py::{self, Error, Result};

pub const FILE_NAME: &str = "config.toml";
pub const EMULATOR_SECTION: &str = "azahar";

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

/// What a settings table is made of, as far as reading it by name goes: Python's `dataclasses`
/// and `getattr` and `replace`, which Rust has no way to do without being told the fields.
pub trait Table: Default {
    /// `dataclasses.fields(self)` with each field's value, in the order they are declared.
    fn fields(&self) -> Vec<(&'static str, Value)>;
    /// `dataclasses.replace(self, **{setting: value})`, for a value that `_same_kind` accepted.
    fn set(&mut self, setting: &str, value: Value);
}

/// `[pa3ds]`.
#[derive(Debug, Clone, PartialEq)]
pub struct General {
    pub keys: bool,
    pub poll_rate: f64,
    pub azahar_folder: String,
    pub hide_window: bool,
}

impl Default for General {
    fn default() -> Self {
        General {
            keys: true,
            poll_rate: 60.0,
            azahar_folder: String::new(),
            hide_window: true,
        }
    }
}

impl Table for General {
    fn fields(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("keys", Value::Bool(self.keys)),
            ("poll_rate", Value::Float(self.poll_rate)),
            ("azahar_folder", Value::Str(self.azahar_folder.clone())),
            ("hide_window", Value::Bool(self.hide_window)),
        ]
    }

    fn set(&mut self, setting: &str, value: Value) {
        match (setting, value) {
            ("keys", Value::Bool(value)) => self.keys = value,
            ("poll_rate", Value::Int(value)) => self.poll_rate = value as f64,
            ("poll_rate", Value::Float(value)) => self.poll_rate = value,
            ("azahar_folder", Value::Str(value)) => self.azahar_folder = value,
            ("hide_window", Value::Bool(value)) => self.hide_window = value,
            _ => {}
        }
    }
}

/// `[pa3ds.ui]`: how menus, lists and screens are said.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ui {
    /// Off unless asked for, since a screen reader reads "Torchic, ♂, L, 5" more clumsily than
    /// "Torchic ♂ L 5" (the user, 2026-09-14).
    pub pauses: bool,
    pub interrupt: bool,
    pub unfocused: bool,
    pub descriptions: bool,
    pub language: String,
}

impl Table for Ui {
    fn fields(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("pauses", Value::Bool(self.pauses)),
            ("interrupt", Value::Bool(self.interrupt)),
            ("unfocused", Value::Bool(self.unfocused)),
            ("descriptions", Value::Bool(self.descriptions)),
            ("language", Value::Str(self.language.clone())),
        ]
    }

    fn set(&mut self, setting: &str, value: Value) {
        match (setting, value) {
            ("pauses", Value::Bool(value)) => self.pauses = value,
            ("interrupt", Value::Bool(value)) => self.interrupt = value,
            ("unfocused", Value::Bool(value)) => self.unfocused = value,
            ("descriptions", Value::Bool(value)) => self.descriptions = value,
            ("language", Value::Str(value)) => self.language = value,
            _ => {}
        }
    }
}

/// `[pa3ds.audio]`: how loud pa3ds's own sounds are beside the game's, 1 as loud as the game
/// (Orion, 2026-10-18).
#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    pub tones: f64,
    pub footsteps: f64,
    /// A tone at each tick of a game's clock, which is said only at its marks
    /// (`screen_announce`, `tones.ticking`); off until a player asks (Orion, 2026-10-26).
    pub ticks: f64,
    /// The game's own sounds for what a key has done: its touch sounds as a drag is picked up and
    /// let go of, in place of "holding" and "let go on", which slowed a timed game (Orion,
    /// 2026-10-28), and its error sound for a key with nothing to do, in place of saying why; 0
    /// says them all. `drag_sounds` until 2026-10-29, when the error sound joined them (Orion).
    pub feedback_sounds: f64,
}

impl Default for Audio {
    fn default() -> Self {
        Audio {
            tones: 1.0,
            footsteps: 1.0,
            ticks: 0.0,
            feedback_sounds: 1.0,
        }
    }
}

impl Table for Audio {
    fn fields(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("tones", Value::Float(self.tones)),
            ("footsteps", Value::Float(self.footsteps)),
            ("ticks", Value::Float(self.ticks)),
            ("feedback_sounds", Value::Float(self.feedback_sounds)),
        ]
    }

    fn set(&mut self, setting: &str, value: Value) {
        let value = match value {
            Value::Int(value) => value as f64,
            Value::Float(value) => value,
            _ => return,
        };
        match setting {
            "tones" => self.tones = value,
            "footsteps" => self.footsteps = value,
            "ticks" => self.ticks = value,
            "feedback_sounds" => self.feedback_sounds = value,
            _ => {}
        }
    }
}

/// `[pa3ds.navigation]`: how the map's list and the way to a thing are said.
#[derive(Debug, Clone, PartialEq)]
pub struct Navigation {
    pub positions: bool,
    pub position_first: bool,
    pub turn_by_turn: bool,
    pub guide_to_hidden: bool,
    pub track_boulders: bool,
    pub hide_door_signs: bool,
    pub show_y: bool,
    pub sort_by_distance: bool,
}

impl Default for Navigation {
    fn default() -> Self {
        Navigation {
            positions: true,
            position_first: true,
            turn_by_turn: true,
            guide_to_hidden: true,
            track_boulders: false,
            hide_door_signs: true,
            show_y: false,
            sort_by_distance: true,
        }
    }
}

impl Table for Navigation {
    fn fields(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("positions", Value::Bool(self.positions)),
            ("position_first", Value::Bool(self.position_first)),
            ("turn_by_turn", Value::Bool(self.turn_by_turn)),
            ("guide_to_hidden", Value::Bool(self.guide_to_hidden)),
            ("track_boulders", Value::Bool(self.track_boulders)),
            ("hide_door_signs", Value::Bool(self.hide_door_signs)),
            ("show_y", Value::Bool(self.show_y)),
            ("sort_by_distance", Value::Bool(self.sort_by_distance)),
        ]
    }

    fn set(&mut self, setting: &str, value: Value) {
        match (setting, value) {
            ("positions", Value::Bool(value)) => self.positions = value,
            ("position_first", Value::Bool(value)) => self.position_first = value,
            ("turn_by_turn", Value::Bool(value)) => self.turn_by_turn = value,
            ("guide_to_hidden", Value::Bool(value)) => self.guide_to_hidden = value,
            ("track_boulders", Value::Bool(value)) => self.track_boulders = value,
            ("hide_door_signs", Value::Bool(value)) => self.hide_door_signs = value,
            ("show_y", Value::Bool(value)) => self.show_y = value,
            ("sort_by_distance", Value::Bool(value)) => self.sort_by_distance = value,
            _ => {}
        }
    }
}

/// `[pa3ds.touch]`: how touch mode walks the bottom screen's buttons and drags, out of
/// `[pa3ds.navigation]` since touch mode gained settings of its own (Orion, 2026-10-29).
#[derive(Debug, Clone, PartialEq)]
pub struct Touch {
    pub wrap: bool,
    pub next_row: bool,
    pub drag: String,
}

impl Default for Touch {
    fn default() -> Self {
        Touch {
            wrap: true,
            next_row: true,
            drag: DRAG_HOLD.to_string(),
        }
    }
}

impl Table for Touch {
    fn fields(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("wrap", Value::Bool(self.wrap)),
            ("next_row", Value::Bool(self.next_row)),
            ("drag", Value::Str(self.drag.clone())),
        ]
    }

    fn set(&mut self, setting: &str, value: Value) {
        match (setting, value) {
            ("wrap", Value::Bool(value)) => self.wrap = value,
            ("next_row", Value::Bool(value)) => self.next_row = value,
            ("drag", Value::Str(value)) => self.drag = value,
            _ => {}
        }
    }
}

/// `[pa3ds.amie]`: how Pokémon-Amie's games are said (Orion, 2026-10-29).
#[derive(Debug, Clone, PartialEq)]
pub struct Amie {
    /// Berry Picker says the berry before the Pokémon, "Pinap Berry for Pikachu", in the order a
    /// berry is given, its key and then the Pokémon's; or the Pokémon first, "Pikachu wants Pinap
    /// Berry".
    pub berry_first: bool,
    /// Berry Picker says one Pokémon wanting a berry at a time, the next once a berry has been
    /// given or the one said has gone: three said together took 1.5 s (Orion).
    pub one_at_a_time: bool,
}

impl Default for Amie {
    fn default() -> Self {
        Amie {
            berry_first: true,
            one_at_a_time: true,
        }
    }
}

impl Table for Amie {
    fn fields(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("berry_first", Value::Bool(self.berry_first)),
            ("one_at_a_time", Value::Bool(self.one_at_a_time)),
        ]
    }

    fn set(&mut self, setting: &str, value: Value) {
        match (setting, value) {
            ("berry_first", Value::Bool(value)) => self.berry_first = value,
            ("one_at_a_time", Value::Bool(value)) => self.one_at_a_time = value,
            _ => {}
        }
    }
}

/// `[pa3ds.extras]`: what the game shows in a way pa3ds cannot read, said from the game's own
/// numbers, each off unless asked for (ADR-0017).
#[derive(Debug, Clone, PartialEq)]
pub struct Extras {
    pub conditions: bool,
    pub hidden_items: String,
}

impl Default for Extras {
    fn default() -> Self {
        Extras {
            conditions: false,
            hidden_items: HIDDEN_OFF.to_string(),
        }
    }
}

impl Table for Extras {
    fn fields(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("conditions", Value::Bool(self.conditions)),
            ("hidden_items", Value::Str(self.hidden_items.clone())),
        ]
    }

    fn set(&mut self, setting: &str, value: Value) {
        match (setting, value) {
            ("conditions", Value::Bool(value)) => self.conditions = value,
            ("hidden_items", Value::Str(value)) => self.hidden_items = value,
            _ => {}
        }
    }
}

/// The settings file's tables as read, carried whole from the file to the readers, each
/// taking its own: a new table is a class here and its reader, with nothing between to copy it
/// through (the deepening round, 2026-10-30).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tables {
    pub general: General,
    pub ui: Ui,
    pub audio: Audio,
    pub navigation: Navigation,
    pub touch: Touch,
    pub amie: Amie,
    pub extras: Extras,
}

pub const HIDDEN_OFF: &str = "off";
pub const HIDDEN_UNDERWATER: &str = "underwater";
pub const HIDDEN_ALL: &str = "all";
pub const DRAG_HOLD: &str = "hold";
pub const DRAG_TOGGLE: &str = "toggle";
/// Each setting that is one of a few words, by its name in the file: its words, the default
/// first. `[pa3ds.extras] hidden_items` lists none, those under the sea, where the Dowsing Machine
/// cannot be used and a sighted player is taken to see them, or all (Orion, 2026-10-26); a word it
/// does not have lists none. `[pa3ds.touch] drag` holds a touch mode drag while its key is held,
/// or picks up with one press and lets go with the next, for a player who cannot hold a key
/// (Orion, 2026-10-28).
pub const CHOICES: [(&str, &[&str]); 2] = [
    (
        "pa3ds.extras.hidden_items",
        &[HIDDEN_OFF, HIDDEN_UNDERWATER, HIDDEN_ALL],
    ),
    ("pa3ds.touch.drag", &[DRAG_HOLD, DRAG_TOGGLE]),
];

/// The tables of the file, in the order it writes them: each is a settings dataclass, and
/// `defaults` is what `kind()` holds.
pub const TABLES: [&str; 7] = [
    "pa3ds",
    "pa3ds.ui",
    "pa3ds.audio",
    "pa3ds.navigation",
    "pa3ds.touch",
    "pa3ds.amie",
    "pa3ds.extras",
];
/// Settings that moved, (old table, setting) -> (new table, setting): to `[pa3ds.audio]`
/// (2026-10-18), `drag_sounds` named `feedback_sounds` and touch mode's own to `[pa3ds.touch]`
/// (2026-10-29). The player's value is carried into the new line as it is filled in, tones turned
/// off as 0 (`fill`).
pub const MOVED: [((&str, &str), (&str, &str)); 6] = [
    (("pa3ds.ui", "tones"), ("pa3ds.audio", "tones")),
    (("pa3ds.ui", "footsteps"), ("pa3ds.audio", "footsteps")),
    (
        ("pa3ds.audio", "drag_sounds"),
        ("pa3ds.audio", "feedback_sounds"),
    ),
    (("pa3ds.navigation", "touch_wrap"), ("pa3ds.touch", "wrap")),
    (
        ("pa3ds.navigation", "touch_next_row"),
        ("pa3ds.touch", "next_row"),
    ),
    (("pa3ds.navigation", "drag"), ("pa3ds.touch", "drag")),
];

/// `kind().fields()` for the dataclass `TABLES` names.
pub(crate) fn defaults(name: &str) -> Vec<(&'static str, Value)> {
    match name {
        "pa3ds" => General::default().fields(),
        "pa3ds.ui" => Ui::default().fields(),
        "pa3ds.audio" => Audio::default().fields(),
        "pa3ds.navigation" => Navigation::default().fields(),
        "pa3ds.touch" => Touch::default().fields(),
        "pa3ds.amie" => Amie::default().fields(),
        "pa3ds.extras" => Extras::default().fields(),
        _ => Vec::new(),
    }
}

/// The message naming a setting, `pa3ds.ui.pauses` as `setting-pa3ds-ui-pauses`, in pa3ds's
/// own words (ADR-0040): its short name in the settings dialog. `-about` after it is what the
/// setting does, the comment above it in the file and its description in the dialog.
pub fn message(name: &str) -> String {
    format!("setting-{}", name.replace('.', "-"))
}

/// What a setting does, `pa3ds.ui.pauses`'s, in the words in place.
pub fn explained(name: &str) -> String {
    own(&format!("{}-about", message(name)), &[])
}

/// Azahar's settings as pa3ds ships them, each by its name in `qt-config.ini`.
pub fn azahar() -> IndexMap<&'static str, (Value, &'static str)> {
    IndexMap::from([
        (
            "enable_rpc_server",
            (
                Value::Bool(true),
                "the RPC server is how pa3ds reads the game: without it pa3ds hears nothing",
            ),
        ),
        (
            "log_filter",
            (
                Value::Str("*:Info RPC_Server:Error".to_string()),
                "keep the RPC server out of Azahar's log, which it would fill at a megabyte a \
                 second",
            ),
        ),
        (
            "graphics_api",
            (
                Value::Int(2),
                "Vulkan (2) rather than OpenGL (1): on OpenGL Azahar's window is slow to answer \
                 the screen reader",
            ),
        ),
        (
            "async_shader_compilation",
            (
                Value::Bool(true),
                "build shaders beside the game instead of pausing it for each new one",
            ),
        ),
        (
            "shaders_accurate_mul",
            (
                Value::Bool(false),
                "faster shaders, at the cost of a rare wrong pixel",
            ),
        ),
        (
            "use_vsync",
            (
                Value::Bool(false),
                "do not wait for the monitor between frames",
            ),
        ),
        (
            "use_touchpad",
            (
                Value::Bool(true),
                "take touches from a second device beside the mouse: pa3ds's taps in touch mode",
            ),
        ),
        (
            "controller_touch_device",
            (
                Value::Str(touch::DEVICE.to_string()),
                "that second device: pa3ds itself, in the touch screen's own pixels",
            ),
        ),
    ])
}

/// What the settings file settled, and what to log and say about it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Config {
    pub tables: Tables,
    pub notes: Vec<String>,
    pub said: String,
    /// Whether `said` is something the player must do: before Azahar has pa3ds's settings, or
    /// before their settings file is read at all.
    pub warning: bool,
    /// The `qt-config.ini` this run patched, or would have: the portable Azahar's when
    /// `azahar_folder` names one.
    pub emulator: PathBuf,
}

/// `<pa3ds>/user/config.toml`, or under `home` instead.
pub fn config_path(home: Option<&Path>) -> Result<PathBuf> {
    let home = match home {
        None => keyfile::home_path()?,
        Some(home) => home.to_path_buf(),
    };
    Ok(home.join(FILE_NAME))
}

/// The file as it is first written: every setting at its default, each explained.
pub fn render() -> Result<String> {
    let mut lines: Vec<String> = vec![
        "# pa3ds's settings, read as pa3ds starts. This file is yours: pa3ds adds the settings it"
            .to_string(),
        "# gains later and never changes a line you have. To turn a line off, put a # in front of"
            .to_string(),
        "# it rather than deleting it, or pa3ds adds it back.".to_string(),
    ];
    for name in TABLES {
        lines.extend(["".to_string(), format!("[{name}]")]);
        for (setting, value, said) in settings(name)? {
            lines.extend(setting_lines(setting, &value, &said));
        }
    }
    lines.extend([
        "".to_string(),
        format!("[{EMULATOR_SECTION}]"),
        "".to_string(),
        "# Azahar's own settings, written into its qt-config.ini whenever pa3ds starts while"
            .to_string(),
        "# Azahar is closed. Azahar saves its own over them when it closes, so close it before"
            .to_string(),
        "# starting pa3ds after a change here. A line commented out leaves Azahar's own value"
            .to_string(),
        "# alone.".to_string(),
    ]);
    for (setting, value, said) in settings(EMULATOR_SECTION)? {
        lines.extend(setting_lines(setting, &value, &said));
    }
    lines.push("".to_string());
    Ok(lines.join("\n"))
}

/// A table's settings as the file has them: name, default and what it does.
fn settings(table: &str) -> Result<Vec<(&'static str, Value, String)>> {
    if table == EMULATOR_SECTION {
        return Ok(azahar()
            .into_iter()
            .map(|(name, (default, said))| (name, default, said.to_string()))
            .collect());
    }
    Ok(defaults(table)
        .into_iter()
        .map(|(setting, default)| (setting, default, explained(&format!("{table}.{setting}"))))
        .collect())
}

/// One setting's lines: a blank, what it does, and the setting at its default.
fn setting_lines(name: &str, value_: &Value, said: &str) -> Vec<String> {
    let mut lines = vec!["".to_string()];
    lines.extend(_explained(said));
    lines.push(format!("{name} = {}", value(value_)));
    lines
}

/// The file with every setting it lacks added at its default, at the end of its table, and
/// the name of each added. One commented out is not lacking: the player turned it off.
pub fn fill(text: &str) -> Result<(String, Vec<String>)> {
    let have = toml::offered(text);
    let parsed = toml::parse(text).0;
    let mut text = text.to_string();
    let mut added: Vec<String> = Vec::new();
    for table in TABLES.into_iter().chain([EMULATOR_SECTION]) {
        let named: Vec<&str> = have
            .get(table)
            .map(|pairs| pairs.iter().map(|(name, _)| name.as_str()).collect())
            .unwrap_or_default();
        let lacking: Vec<(&str, Value, String)> = settings(table)?
            .into_iter()
            .filter(|(name, _, _)| !named.contains(name))
            .map(|(name, value, said)| (name, carried(&parsed, table, name, value), said))
            .collect();
        if !lacking.is_empty() {
            let lines: Vec<String> = lacking
                .iter()
                .flat_map(|(name, value, said)| setting_lines(name, value, said))
                .collect();
            text = toml::append(&text, table, &lines);
            added.extend(
                lacking
                    .iter()
                    .map(|(name, _, _)| format!("[{table}] {name}")),
            );
        }
    }
    Ok((text, added))
}

/// A setting's value as the file had it before it moved here (`MOVED`), or its default: a
/// number as it was, a number's on or off as 1 or 0, and anything else as it was where it is
/// the setting's kind.
fn carried(
    parsed: &IndexMap<String, ::toml::Value>,
    table: &str,
    setting: &str,
    default: Value,
) -> Value {
    let old = MOVED
        .iter()
        .find(|&&(_, after)| after == (table, setting))
        .map(|&(before, _)| before);
    let Some((old_table, old_setting)) = old else {
        return default;
    };
    let mut found = parsed.clone();
    for key in old_table.split('.') {
        found = toml::table(found.get(key));
    }
    let value = found.get(old_setting);
    if let Value::Float(_) = default {
        match value {
            Some(::toml::Value::Boolean(value)) => {
                return Value::Float(if *value { 1.0 } else { 0.0 });
            }
            Some(::toml::Value::Integer(value)) => return Value::Float(*value as f64),
            Some(::toml::Value::Float(value)) => return Value::Float(*value),
            _ => {}
        }
    }
    match value {
        Some(value) if same_kind(value, &default) => value_of(value).unwrap_or(default),
        _ => default,
    }
}

/// What a setting does, as a sentence in comment lines short enough for a terminal.
// Python's `_explained` sits beside a public `explained`, so it keeps its underscore.
fn _explained(said: &str) -> Vec<String> {
    let mut rest = said.chars();
    let first: String = rest
        .next()
        .map(|first| first.to_uppercase().collect())
        .unwrap_or_default();
    let sentence = format!("{first}{}", rest.as_str());
    let sentence = if sentence.ends_with('.') {
        sentence
    } else {
        format!("{sentence}.")
    };
    toml::comment(py::wrap(&sentence, 94).expect("94 is a positive width"))
}

fn value(value: &Value) -> String {
    match value {
        Value::Bool(value) => value.to_string(),
        Value::Str(value) => toml::quote(value),
        Value::Int(value) => py::format_g(*value as f64),
        Value::Float(value) => py::format_g(*value),
    }
}

/// Whether a value from the file can stand for a default: 30 for 60.0, but never true for a
/// number, which Python would otherwise count as one.
fn same_kind(value: &::toml::Value, default: &Value) -> bool {
    let is_bool = matches!(value, ::toml::Value::Boolean(_));
    if matches!(default, Value::Bool(_)) || is_bool {
        return is_bool && matches!(default, Value::Bool(_));
    }
    match default {
        Value::Int(_) | Value::Float(_) => {
            matches!(value, ::toml::Value::Integer(_) | ::toml::Value::Float(_))
        }
        Value::Str(_) => matches!(value, ::toml::Value::String(_)),
        Value::Bool(_) => false,
    }
}

/// A value from the file as the kind of setting it is, which only `bool`, `int`, `float` and
/// `str` are.
fn value_of(value: &::toml::Value) -> Option<Value> {
    match value {
        ::toml::Value::Boolean(value) => Some(Value::Bool(*value)),
        ::toml::Value::Integer(value) => Some(Value::Int(*value)),
        ::toml::Value::Float(value) => Some(Value::Float(*value)),
        ::toml::Value::String(value) => Some(Value::Str(value.clone())),
        _ => None,
    }
}

/// `repr(value)` of what a TOML parser handed back to Python: a `dict`, a `list`, a `datetime`.
pub(crate) fn repr_of(value: &::toml::Value) -> String {
    match value {
        ::toml::Value::String(value) => py::repr(value),
        ::toml::Value::Integer(value) => value.to_string(),
        ::toml::Value::Float(value) => py::float_repr(*value),
        ::toml::Value::Boolean(true) => "True".to_string(),
        ::toml::Value::Boolean(false) => "False".to_string(),
        ::toml::Value::Datetime(value) => datetime_repr(value),
        ::toml::Value::Array(items) => {
            let items: Vec<String> = items.iter().map(repr_of).collect();
            format!("[{}]", items.join(", "))
        }
        ::toml::Value::Table(items) => {
            let items: Vec<String> = items
                .iter()
                .map(|(key, held)| format!("{}: {}", py::repr(key), repr_of(held)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

/// `repr` of the `date`, `time` or `datetime` that `tomllib` makes of a TOML date and time.
fn datetime_repr(value: &::toml::value::Datetime) -> String {
    let time = value.time.map(|time| {
        let microsecond = time.nanosecond / 1000;
        let mut parts = vec![time.hour.to_string(), time.minute.to_string()];
        if microsecond != 0 {
            parts.push(time.second.to_string());
            parts.push(microsecond.to_string());
        } else if time.second != 0 {
            parts.push(time.second.to_string());
        }
        parts
    });
    let date = value.date.map(|date| {
        vec![
            date.year.to_string(),
            date.month.to_string(),
            date.day.to_string(),
        ]
    });
    match (date, time) {
        (Some(date), None) => format!("datetime.date({})", date.join(", ")),
        (None, Some(time)) => format!("datetime.time({})", time.join(", ")),
        (date, time) => {
            let parts = [date.unwrap_or_default(), time.unwrap_or_default()].concat();
            let zone = match value.offset {
                None => String::new(),
                Some(::toml::value::Offset::Z)
                | Some(::toml::value::Offset::Custom { minutes: 0 }) => {
                    ", tzinfo=datetime.timezone.utc".to_string()
                }
                Some(::toml::value::Offset::Custom { minutes }) => {
                    let seconds = i64::from(minutes) * 60;
                    let (days, seconds) =
                        (py::floordiv(seconds, 86400), py::modulo(seconds, 86400));
                    let mut delta = Vec::new();
                    if days != 0 {
                        delta.push(format!("days={days}"));
                    }
                    if seconds != 0 {
                        delta.push(format!("seconds={seconds}"));
                    }
                    format!(
                        ", tzinfo=datetime.timezone(datetime.timedelta({}))",
                        delta.join(", ")
                    )
                }
            };
            format!("datetime.datetime({}{zone})", parts.join(", "))
        }
    }
}

/// One table's settings: what the file says where it is the right kind, and the defaults for
/// the rest. A table under this one (`[pa3ds.ui]` under `[pa3ds]`) is its own business.
fn table<T: Table>(
    found: &IndexMap<String, ::toml::Value>,
    name: &str,
    path: &Path,
    notes: &mut Vec<String>,
) -> T {
    let mut defaults = T::default();
    // The fields, not `hasattr`: `__doc__ = "x"` is an attribute of every table and no setting.
    let fields = defaults.fields();
    let mut kept: Vec<(&str, Value)> = Vec::new();
    for (setting, value) in found {
        if TABLES.contains(&format!("{name}.{setting}").as_str()) {
            continue;
        }
        if let Some((_, (table, now))) = MOVED
            .iter()
            .find(|((old, moved), _)| *old == name && *moved == setting.as_str())
        {
            notes.push(format!(
                "{}: [{name}] {setting} is [{table}] {now} now",
                py::path_str(path)
            ));
            continue;
        }
        match fields.iter().find(|(field, _)| *field == setting.as_str()) {
            None => notes.push(format!(
                "{}: [{name}] has no setting called {}",
                py::path_str(path),
                py::repr(setting)
            )),
            Some((field, default)) => match value_of(value) {
                Some(kept_value) if same_kind(value, default) => kept.push((*field, kept_value)),
                _ => notes.push(format!(
                    "{}: [{name}] {setting} = {} is not a setting it can take",
                    py::path_str(path),
                    repr_of(value)
                )),
            },
        }
    }
    for (setting, value) in kept {
        defaults.set(setting, value);
    }
    defaults
}

/// `[pa3ds.ui] language` as the settings file says it, or "": read before anything is said,
/// `prepare`'s own sentence among it, so that it is said in that language.
pub fn ui_language(home: Option<&Path>) -> Result<String> {
    let ours = toml::table(toml::read(&config_path(home)?).get("pa3ds"));
    Ok(match toml::table(ours.get("ui")).get("language") {
        Some(::toml::Value::String(language)) => language.clone(),
        _ => String::new(),
    })
}

/// Read the settings file, writing it the first time, and patch Azahar's settings.
///
/// `write` off reads and reports without touching either file. `running` says Azahar is open,
/// when its config is left alone (`keyfile.save`).
pub fn prepare(
    home: Option<&Path>,
    emulator: Option<&Path>,
    write: bool,
    running: bool,
) -> Result<Config> {
    let path = config_path(home)?;
    let mut notes: Vec<String> = Vec::new();
    let mut loaded: IndexMap<String, ::toml::Value> = IndexMap::new();
    let mut problem = String::new();
    if !path.exists() {
        if write && toml::write(&path, &render()?)? {
            notes.push(format!(
                "wrote {} with the default settings",
                py::path_str(&path)
            ));
        }
        let mut shipped = ::toml::Table::new();
        for (name, (default, _)) in azahar() {
            shipped.insert(
                name.to_string(),
                match default {
                    Value::Bool(default) => ::toml::Value::Boolean(default),
                    Value::Int(default) => ::toml::Value::Integer(default),
                    Value::Float(default) => ::toml::Value::Float(default),
                    Value::Str(default) => ::toml::Value::String(default),
                },
            );
        }
        loaded.insert(EMULATOR_SECTION.to_string(), ::toml::Value::Table(shipped));
    } else {
        let text;
        (text, loaded, problem) = toml::load(&path);
        let (filled, added) = if problem.is_empty() {
            fill(&text)?
        } else {
            (text, Vec::new())
        };
        if !problem.is_empty() {
            notes.push(format!(
                "{} could not be read, using the default settings: {problem}",
                py::path_str(&path)
            ));
        } else if !added.is_empty() {
            notes.push(format!(
                "added to {}: {}",
                py::path_str(&path),
                added.join(", ")
            ));
            loaded = toml::parse(&filled).0;
            if write && !toml::write(&path, &filled)? {
                notes.push(format!("{} could not be written", py::path_str(&path)));
            }
        }
    }
    for name in loaded.keys() {
        if !TABLES.contains(&name.as_str()) && name != EMULATOR_SECTION {
            let only: Vec<String> = TABLES.iter().map(|table| format!("[{table}]")).collect();
            notes.push(format!(
                "{}: there is no [{name}] table, only {} and [{EMULATOR_SECTION}]",
                py::path_str(&path),
                only.join(", ")
            ));
        }
    }

    let ours = toml::table(loaded.get("pa3ds"));
    let mut general: General = table(&ours, "pa3ds", &path, &mut notes);
    // `not general.poll_rate > 0` is also true of a NaN, which no comparison orders.
    if general.poll_rate.is_nan() || general.poll_rate <= 0.0 {
        let rate = General::default().poll_rate;
        notes.push(format!(
            "{}: poll_rate must be above zero, reading at {}",
            py::path_str(&path),
            py::format_g(rate)
        ));
        general.poll_rate = rate;
    }
    let ui: Ui = table(&toml::table(ours.get("ui")), "pa3ds.ui", &path, &mut notes);
    let audio: Audio = table(
        &toml::table(ours.get("audio")),
        "pa3ds.audio",
        &path,
        &mut notes,
    );
    if audio.tones < 0.0 || audio.footsteps < 0.0 || audio.ticks < 0.0 {
        notes.push(format!(
            "{}: [pa3ds.audio] cannot be below 0, playing at its default",
            py::path_str(&path)
        ));
        // The Python zips the three with `dataclasses.astuple(Audio())`, strict, and since
        // `drag_sounds`, now `feedback_sounds`, that has four: it raises here (an upstream bug,
        // ported as it is).
        return Err(Error::value("zip() argument 2 is longer than argument 1"));
    }
    let navigation: Navigation = table(
        &toml::table(ours.get("navigation")),
        "pa3ds.navigation",
        &path,
        &mut notes,
    );
    let touch: Touch = table(
        &toml::table(ours.get("touch")),
        "pa3ds.touch",
        &path,
        &mut notes,
    );
    let amie: Amie = table(
        &toml::table(ours.get("amie")),
        "pa3ds.amie",
        &path,
        &mut notes,
    );
    let extras: Extras = table(
        &toml::table(ours.get("extras")),
        "pa3ds.extras",
        &path,
        &mut notes,
    );

    let mut wanted: IndexMap<String, Value> = IndexMap::new();
    for (name, value) in toml::table(loaded.get(EMULATOR_SECTION)) {
        match value_of(&value) {
            Some(value) => {
                wanted.insert(name, value);
            }
            None => notes.push(format!(
                "{}: Azahar's {name} is not a setting it can take",
                py::path_str(&path)
            )),
        }
    }

    let mut saved = String::new();
    let mut folder = general.azahar_folder.clone();
    if emulator.is_none()
        && folder.is_empty()
        && let Some(found) = keyfile::found_portable()?
    {
        folder = py::path_str(&found);
        notes.push(format!("a portable Azahar found at {folder}"));
    }
    let config = match emulator {
        None => keyfile::emulator_config(&folder)?,
        Some(emulator) => emulator.to_path_buf(),
    };
    notes.push(format!(
        "Azahar's settings are read from {}",
        py::path_str(&config)
    ));
    if !wanted.is_empty() {
        let text = keyfile::read_config(&config, &mut notes);
        if !text.is_empty() {
            let (patched, changed) = keyfile::patch_settings(&text, &wanted);
            notes.extend(changed);
            saved = keyfile::save(&config, &text, &patched, write, running, &mut notes);
        } else {
            // An Azahar never started has no settings yet: said, or its RPC server stays off
            // unheard.
            saved = "missing".to_string();
        }
    }
    let said = if saved.is_empty() {
        String::new()
    } else {
        own(&format!("azahar-settings-{saved}"), &[])
    };
    let said = if problem.is_empty() {
        said
    } else {
        keyfile::unreadable(&path, &problem)
    };
    Ok(Config {
        tables: Tables {
            general,
            ui,
            audio,
            navigation,
            touch,
            amie,
            extras,
        },
        notes,
        said,
        warning: !problem.is_empty() || matches!(saved.as_str(), "waiting" | "missing"),
        emulator: config,
    })
}

/// Set `azahar_folder` in the settings file to `folder`, as the player asked by choosing it
/// (`change`).
pub fn remember_azahar(folder: &Path, home: Option<&Path>) -> Result<bool> {
    change(
        &[(
            "pa3ds.azahar_folder".to_string(),
            Value::Str(py::path_str(folder)),
        )],
        home,
    )
}

/// Set each setting in the settings file, named by its table and itself
/// (`pa3ds.ui.pauses`), as the player asked: on its line, commented out or not, or else added to
/// its table. Nothing else in the file changes, its comments and order kept (the settings
/// dialog, docs/plans/pa3ds-window.md). False when the file cannot be read or written.
pub fn change(settings: &[(String, Value)], home: Option<&Path>) -> Result<bool> {
    let path = config_path(home)?;
    let (mut text, _, problem) = toml::load(&path);
    if !problem.is_empty() {
        return Ok(false);
    }
    for (name, setting_value) in settings {
        let (table, setting) = name.rsplit_once('.').unwrap_or(("", name));
        let line = format!("{setting} = {}", value(setting_value));
        text = toml::replace(&text, table, setting, &line)
            .unwrap_or_else(|| toml::append(&text, table, &[&line]));
    }
    toml::write(&path, &text)
}
