//! `<pa3ds>/user/config.toml`: pa3ds's settings, and the emulator settings pa3ds wants (ADR-0026).
//!
//! The file is written once with the defaults, each explained in a comment, and never
//! overwritten — as the keys file is (`pa3ds.core.keyfile`), which stays a file of its own. Its
//! tables:
//!
//! - **`[pa3ds]`**: whether pa3ds answers its keys, and how often it reads the game.
//! - **`[pa3ds.ui]`**: how menus, lists and screens are said.
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
//! Nothing here raises at a player. A file that is missing or malformed reads as the defaults,
//! and a setting of the wrong kind or an unknown name is logged and left at its default.

use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::core::keyfile;
use crate::core::toml;
use crate::core::touch;
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
}

impl Default for General {
    fn default() -> Self {
        General {
            keys: true,
            poll_rate: 60.0,
        }
    }
}

impl Table for General {
    fn fields(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("keys", Value::Bool(self.keys)),
            ("poll_rate", Value::Float(self.poll_rate)),
        ]
    }

    fn set(&mut self, setting: &str, value: Value) {
        match (setting, value) {
            ("keys", Value::Bool(value)) => self.keys = value,
            ("poll_rate", Value::Int(value)) => self.poll_rate = value as f64,
            ("poll_rate", Value::Float(value)) => self.poll_rate = value,
            _ => {}
        }
    }
}

/// `[pa3ds.ui]`: how menus, lists and screens are said.
#[derive(Debug, Clone, PartialEq)]
pub struct Ui {
    pub pauses: bool,
    pub interrupt: bool,
    pub unfocused: bool,
    pub tones: bool,
    pub descriptions: bool,
}

impl Default for Ui {
    fn default() -> Self {
        Ui {
            pauses: false,
            interrupt: false,
            unfocused: false,
            tones: true,
            descriptions: false,
        }
    }
}

impl Table for Ui {
    fn fields(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("pauses", Value::Bool(self.pauses)),
            ("interrupt", Value::Bool(self.interrupt)),
            ("unfocused", Value::Bool(self.unfocused)),
            ("tones", Value::Bool(self.tones)),
            ("descriptions", Value::Bool(self.descriptions)),
        ]
    }

    fn set(&mut self, setting: &str, value: Value) {
        match (setting, value) {
            ("pauses", Value::Bool(value)) => self.pauses = value,
            ("interrupt", Value::Bool(value)) => self.interrupt = value,
            ("unfocused", Value::Bool(value)) => self.unfocused = value,
            ("tones", Value::Bool(value)) => self.tones = value,
            ("descriptions", Value::Bool(value)) => self.descriptions = value,
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
            _ => {}
        }
    }
}

/// `[pa3ds.extras]`: what the game shows in a way pa3ds cannot read, said from the game's own
/// numbers, each off unless asked for (ADR-0017).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Extras {
    pub conditions: bool,
}

impl Table for Extras {
    fn fields(&self) -> Vec<(&'static str, Value)> {
        vec![("conditions", Value::Bool(self.conditions))]
    }

    fn set(&mut self, setting: &str, value: Value) {
        if let ("conditions", Value::Bool(value)) = (setting, value) {
            self.conditions = value;
        }
    }
}

/// The tables of the file, in the order it writes them: each is a settings dataclass, and
/// `defaults` is what `kind()` holds.
pub const TABLES: [&str; 4] = ["pa3ds", "pa3ds.ui", "pa3ds.navigation", "pa3ds.extras"];

/// `kind().fields()` for the dataclass `TABLES` names.
fn defaults(name: &str) -> Vec<(&'static str, Value)> {
    match name {
        "pa3ds" => General::default().fields(),
        "pa3ds.ui" => Ui::default().fields(),
        "pa3ds.navigation" => Navigation::default().fields(),
        "pa3ds.extras" => Extras::default().fields(),
        _ => Vec::new(),
    }
}

/// What each setting does, as the file's comment above it says it.
pub const EXPLAINED: [(&str, &str); 15] = [
    (
        "pa3ds.keys",
        "answer pa3ds's keys and bind Azahar's buttons, as keys.toml says",
    ),
    (
        "pa3ds.poll_rate",
        "how many times a second to read the game: 60, the game's own frame rate; 30 for a slower \
         machine",
    ),
    (
        "pa3ds.ui.pauses",
        "put pauses between pieces of screen text said together: a colon after a label, as in \
         \"OT: Raiden\", and a comma between the rest",
    ),
    (
        "pa3ds.ui.interrupt",
        "cut off speech when a menu's cursor moves, instead of queueing; pa3ds's own keys always \
         do",
    ),
    (
        "pa3ds.ui.unfocused",
        "keep saying what the game shows while another window is in front; otherwise pa3ds is \
         quiet then, but for its own keys and messages",
    ),
    (
        "pa3ds.ui.tones",
        "a short tone as a map's cursor or touch mode moves, higher the further up and panned the \
         further right, to say where the cursor is",
    ),
    (
        "pa3ds.ui.descriptions",
        "describe the scenes the game shows without words, as Groudon wakes: pa3ds's own words, \
         said after the line each scene follows; for the scenes described so far, in English",
    ),
    (
        "pa3ds.navigation.positions",
        "say where a thing is in the list, as in \"door, 1 of 3\"",
    ),
    (
        "pa3ds.navigation.position_first",
        "say the place in the list before the way to it, \"door, 1 of 3, 1 north\"; false says \
         \"door, 1 north, 1 of 3\"",
    ),
    (
        "pa3ds.navigation.turn_by_turn",
        "once P has found the way, say each turn as it is reached; false says the way once, and P \
         again says it from where you stand",
    ),
    (
        "pa3ds.navigation.guide_to_hidden",
        "when a DexNav search finds the Pokémon searched for, or the DexNav finds one by itself \
         while it is open and you are in touch mode, find the way to it as P does and keep P on \
         it, whatever the list was on; one found while the AreaNav is open is left alone",
    ),
    (
        "pa3ds.navigation.track_boulders",
        "say how far a big boulder has moved from where its room starts it, as in \"big boulder, \
         2 east, moved 1 north\", so you know which you have pushed; the game shows where each \
         boulder is, never where it started",
    ),
    (
        "pa3ds.navigation.hide_door_signs",
        "leave a sign out of the list when all it says is the name of a door beside it, as \
         Mauville Hills' \"Apt. 13 Wattson's Home\"; a sign that says more, as a Gym's does, stays",
    ),
    (
        "pa3ds.navigation.show_y",
        "say how high as well, as y after x and z, when C says where you stand and shift and C \
         where the thing chosen is: \"x 71, z 35, y 20\"; in the game's own units, 18 to a \
         square's width, and left out where pa3ds does not know it, as in Sun and Moon",
    ),
    (
        "pa3ds.extras.conditions",
        "say a Pokémon's contest conditions as numbers where the game draws them as a star, on \
         the summary and in the Pokéblock Case: the star is shown to everyone but pa3ds cannot \
         read it, so its numbers are all it can give, and they are more exact than the star",
    ),
];

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
    pub general: General,
    pub ui: Ui,
    pub navigation: Navigation,
    pub extras: Extras,
    pub notes: Vec<String>,
    pub said: String,
    /// Whether `said` is something the player must do before Azahar has pa3ds's settings.
    pub warning: bool,
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
        "# pa3ds's settings. Written once with the defaults, and never overwritten — this file is"
            .to_string(),
        "# yours. pa3ds reads it as it starts.".to_string(),
    ];
    for name in TABLES {
        lines.extend(["".to_string(), format!("[{name}]")]);
        for (setting, default) in defaults(name) {
            let key = format!("{name}.{setting}");
            let said = EXPLAINED
                .iter()
                .find(|(explained, _)| *explained == key)
                .map(|(_, said)| *said)
                .ok_or_else(|| Error::key(py::repr(&key)))?;
            lines.push("".to_string());
            lines.extend(explained(said));
            lines.push(format!("{setting} = {}", value(&default)));
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
        "# starting pa3ds after a change here. A line taken out leaves Azahar's own value alone."
            .to_string(),
    ]);
    for (name, (default, said)) in azahar() {
        lines.push("".to_string());
        lines.extend(explained(said));
        lines.push(format!("{name} = {}", value(&default)));
    }
    lines.push("".to_string());
    Ok(lines.join("\n"))
}

/// What a setting does, as a sentence in comment lines short enough for a terminal.
fn explained(said: &str) -> Vec<String> {
    let mut rest = said.chars();
    let first: String = rest
        .next()
        .map(|first| first.to_uppercase().collect())
        .unwrap_or_default();
    toml::comment(
        py::wrap(&format!("{first}{}.", rest.as_str()), 94).expect("94 is a positive width"),
    )
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
fn repr_of(value: &::toml::Value) -> String {
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
        loaded = toml::read(&path);
        if loaded.is_empty() {
            notes.push(format!(
                "{} could not be read, using the default settings",
                py::path_str(&path)
            ));
        }
    }
    for name in loaded.keys() {
        if !TABLES.contains(&name.as_str()) && name != EMULATOR_SECTION {
            notes.push(format!(
                "{}: there is no [{name}] table, only [pa3ds], [pa3ds.ui], \
                 [pa3ds.navigation], [pa3ds.extras] and [azahar]",
                py::path_str(&path)
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
    let navigation: Navigation = table(
        &toml::table(ours.get("navigation")),
        "pa3ds.navigation",
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
    let config = match emulator {
        None => keyfile::emulator_config()?,
        Some(emulator) => emulator.to_path_buf(),
    };
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
    let said = match saved.as_str() {
        "applied" => "Azahar settings applied",
        "waiting" => "close Azahar and start pa3ds again to apply its settings",
        "missing" => {
            "Azahar's settings were not found: start Azahar once, close it, and start pa3ds again"
        }
        _ => "",
    };
    Ok(Config {
        general,
        ui,
        navigation,
        extras,
        notes,
        said: said.to_string(),
        warning: matches!(saved.as_str(), "waiting" | "missing"),
    })
}
