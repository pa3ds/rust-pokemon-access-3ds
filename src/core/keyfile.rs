//! One file saying what every key does — pa3ds's keys and the emulator's buttons together.
//!
//! pa3ds takes the keys it is bound to and keeps them from the game (`pa3ds.core.keys`). That
//! hides a problem: **a key pa3ds takes is a button the player has lost.** One player's Azahar
//! happens to have I, J, K and L on the C-stick and O on the debug button, which cost nothing —
//! until M was bound to the navigation mode and M turned out to be Start.
//!
//! So `<pa3ds>/user/keys.toml` holds both halves, written with defaults on the first run and
//! afterwards **tidied**, a key whose action pa3ds no longer has taken out and the tables put in
//! order (`tidy`), and **filled in** with the keys pa3ds gains (`fill`):
//!
//! - **`[pa3ds.navigation]`** — key to action. A key that means the same thing in both navigation
//!   modes lives here; a key that does not lives in `[pa3ds.navigation.list]` or
//!   `[pa3ds.navigation.direction]`, which win over it. A key may carry modifiers: `shift+k`.
//! - **`[pa3ds.<context>]`** — the keys of a screen that has its own, `[pa3ds.amie.berry_picker]`:
//!   they win over the modes' while it runs, and pa3ds takes them from the emulator only then.
//! - **`[azahar]`** — button to key, and `[azahar.circle_pad]` / `[azahar.c_stick]` for the two
//!   sticks, which hold four directions, a modifier and a scale rather than one key. pa3ds patches
//!   these into Azahar's `qt-config.ini` at startup. It ships **empty**, so nothing is rebound
//!   until the player asks: this machine's config is hand-tuned, and a file that silently rewrote
//!   it would be the opposite of useful. The bindings Azahar has are written in as a comment,
//!   which is the only way to read them without Azahar's own settings dialog.
//!
//! Whatever `[azahar]` says, pa3ds reads the emulator's real bindings and **says which buttons its
//! keys are sitting on**. It does not refuse to bind them: a player who wants K on the C-stick may
//! have it, and being told is what they were missing. Azahar's file itself, what it binds and how
//! a binding is written into it, is `pa3ds.core.azahar`'s.
//!
//! A key pa3ds gains is added to the player's file at the next start, unless its key is already
//! one of theirs: then it is added **commented out**, with a comment saying what has the key, and
//! said, interrupting, for the player to choose (reopening ADR-0022's "never written into a file
//! that exists"). A key commented out counts as there, which is how a player
//! unbinds one on purpose.
//!
//! Nothing here raises at a player. A keys file that is missing or malformed reads as the
//! defaults, and a config that cannot be patched is said and left alone. A keys file that will not
//! parse is left alone and said, interrupting.
//!
//! Bound by:
//! - ADR-0022: `keys.toml` is the player's: a key gained is added at its table's end, commented out
//!   where it conflicts; a key in use whose action pa3ds does not have is taken out, and the tables
//!   are kept in order (as amended 2026-11-04).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::core::azahar::{
    Asked, BUTTONS, SCALE, STICK_KEYS, STICKS, Wanted, controls, emulator_config, emulator_keys,
    emulator_sticks, os_error, patch, read_config, save,
};
use crate::core::folders::home_path;
use crate::core::keys::{ORDER, SHIFT};
use crate::core::toml;
use crate::core::words::{listed, own};
use crate::py::{self, Kind, Result};

/// Mode -> key -> action, as `common.field.navigate` defines the modes and the actions. Nothing here
/// knows what an action means; this module only decides which keys carry them.
pub type Bindings = IndexMap<String, IndexMap<String, String>>;

pub const FILE_NAME: &str = "keys.toml";
pub const SECTION: &str = "pa3ds";
/// The one group of pa3ds's own keys there is so far. Its sub-tables are the navigation modes,
/// and the keys directly under it are the ones that mean the same thing in both.
pub const GROUP: &str = "navigation";
pub const EMULATOR_SECTION: &str = "azahar";

/// Controls whose loss is logged but not said. ORAS uses none of them — the C-stick does nothing
/// in it, and the debug and GPIO 14 buttons are the emulator's own — so taking one costs the player
/// nothing, and saying so at every start would be noise. Everything else is a button somebody
/// presses, and M turning out to be Start is exactly what this is for.
pub const QUIET: [&str; 3] = ["c_stick", "button_debug", "button_gpio14"];

/// A bound name without its modifiers: `shift+k` is still the k key as far as Azahar is
/// concerned, because the emulator watches the key and not what is held with it.
pub fn bare(name: &str) -> String {
    let mut name = name;
    for modifier in ORDER {
        name = name.strip_prefix(&format!("{modifier}+")).unwrap_or(name);
    }
    name.to_string()
}

/// `<pa3ds>/user/keys.toml`, or under `home` instead.
pub fn keys_path(home: Option<&Path>) -> Result<PathBuf> {
    let home = match home {
        None => home_path()?,
        Some(home) => home.to_path_buf(),
    };
    Ok(home.join(FILE_NAME))
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
/// key that does it. `contexts` are the keys of each screen that has its own, a table each.
pub fn render(
    bindings: &Bindings,
    emulator: &IndexMap<String, Vec<String>>,
    sticks: Option<&IndexMap<String, IndexMap<String, String>>>,
    explained: Option<&IndexMap<String, String>>,
    contexts: Option<&Bindings>,
) -> Result<String> {
    let no_sticks = IndexMap::new();
    let sticks = sticks.unwrap_or(&no_sticks);
    let no_explained = IndexMap::new();
    let explained = explained.unwrap_or(&no_explained);
    let no_contexts = IndexMap::new();
    let contexts = contexts.unwrap_or(&no_contexts);

    let line = |key: &str, action: &str| -> Vec<String> { self::line(key, action, explained) };

    let both = shared(bindings);
    let mut lines: Vec<String> = vec![
        "# What every key does: the keys pa3ds answers and the buttons Azahar is bound to.".into(),
        "# This file is yours: pa3ds adds the keys it gains later, takes out a key whose action"
            .into(),
        "# it no longer has, and changes no other line you have. To unbind a key, put a # in"
            .into(),
        "# front of it rather than deleting it, or pa3ds adds it back.".into(),
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
    for (context, keys) in contexts {
        lines.push("".into());
        lines.push(format!(
            "# What a key means while {context} runs, in every mode, winning over the keys"
        ));
        lines.push(
            "# above. pa3ds takes these only then: a key that is one of Azahar's buttons stays"
                .into(),
        );
        lines.push("# that button everywhere else.".into());
        lines.push(format!("[{SECTION}.{context}]"));
        for (key, action) in keys {
            lines.extend(line(key, action));
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
///
/// None is the defaults themselves, which `prepare` tells from a file that happens to say the same
/// (`bindings is defaults` in Python).
fn read_keys(
    path: &Path,
    loaded: &IndexMap<String, ::toml::Value>,
    defaults: &Bindings,
    notes: &mut Vec<String>,
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

/// The table a dotted name leads to, `pa3ds.amie.berry_picker`, or None where there is none.
fn table_at(loaded: &IndexMap<String, ::toml::Value>, name: &str) -> Option<::toml::Value> {
    let mut parts = name.split('.');
    let mut at: Option<::toml::Value> = loaded.get(parts.next()?).cloned();
    for part in parts {
        // `_is_table(at)` before each step: the whole file is a table, and None is not.
        let value = at.filter(is_table)?;
        at = toml::table(Some(&value)).get(part).cloned();
    }
    at.filter(is_table)
}

/// Each context's own table, `[pa3ds.<context>]`, its defaults where the file has none.
fn read_contexts(
    path: &Path,
    loaded: &IndexMap<String, ::toml::Value>,
    defaults: &Bindings,
    notes: &mut Vec<String>,
) -> Bindings {
    let mut out: Bindings = IndexMap::new();
    for (context, keys) in defaults {
        let table = table_at(loaded, &format!("{SECTION}.{context}"));
        let read = match table {
            None => keys.clone(),
            Some(table) => {
                let known: HashSet<&str> = keys.values().map(String::as_str).collect();
                actions(path, Some(&table), &known, notes)
            }
        };
        out.insert(context.clone(), read);
    }
    out
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
    /// Each context's keys, which the hook takes only while its screen runs.
    pub contexts: Bindings,
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
/// commented out, with a comment saying what has it, so nothing they bound changes meaning. A
/// context's keys go in its own table, `[pa3ds.<context>]`, which wins over the modes' while its
/// screen runs, so only a key of its own table keeps one out.
pub fn fill(
    text: &str,
    loaded: &IndexMap<String, ::toml::Value>,
    defaults: &Bindings,
    explained: Option<&IndexMap<String, String>>,
    contexts: Option<&Bindings>,
) -> Result<(String, Vec<Added>)> {
    let no_explained = IndexMap::new();
    let explained = explained.unwrap_or(&no_explained);
    let no_contexts = IndexMap::new();
    let contexts = contexts.unwrap_or(&no_contexts);
    let have = toml::offered(text);
    let group = ::toml::Value::Table(
        toml::table(toml::table(loaded.get(SECTION)).get(GROUP))
            .into_iter()
            .collect(),
    );
    let mut live: IndexMap<&String, IndexMap<String, String>> = defaults
        .keys()
        .map(|mode| {
            let mut keys = strings(Some(&group));
            keys.extend(strings(toml::table(Some(&group)).get(mode)));
            (mode, keys)
        })
        .collect();
    for context in contexts.keys() {
        let table = table_at(loaded, &format!("{SECTION}.{context}"));
        live.insert(context, strings(table.as_ref()));
    }

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
    for (context, keys) in contexts {
        let name = format!("{SECTION}.{context}");
        let given: Vec<&::toml::Value> = have
            .get(&name)
            .map(|pairs| pairs.iter().map(|(_, value)| value).collect())
            .unwrap_or_default();
        for (key, action) in keys {
            if !given
                .iter()
                .any(|value| matches!(value, ::toml::Value::String(value) if value == action))
            {
                adding.push((name.clone(), key.clone(), action.clone(), vec![context]));
            }
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

/// The keys file with each key in use bound to an action its table does not have taken out,
/// with the comment above it, and each button and stick's setting Azahar has none of; a table
/// pa3ds does not have taken out whole; and the tables in order, the modes', the screens' own and
/// `[azahar]` and its sticks last, with the comments that introduce `[azahar]`. The text as it
/// was where nothing needs it, and what was taken out (ADR-0022 as amended 2026-11-04).
///
/// A key commented out stays, whatever it does: the file's list of Azahar's buttons above
/// `[azahar]`, and its sticks' settings under it, are lines commented out.
pub fn tidy(text: &str, defaults: &Bindings, contexts: Option<&Bindings>) -> (String, Vec<String>) {
    let no_contexts = IndexMap::new();
    let contexts = contexts.unwrap_or(&no_contexts);
    let known: HashSet<&str> = defaults
        .values()
        .flat_map(|keys| keys.values().map(String::as_str))
        .collect();
    let mut actions: IndexMap<String, HashSet<&str>> = IndexMap::new();
    actions.insert(format!("{SECTION}.{GROUP}"), known.clone());
    for mode in defaults.keys() {
        actions.insert(format!("{SECTION}.{GROUP}.{mode}"), known.clone());
    }
    for (context, keys) in contexts {
        actions.insert(
            format!("{SECTION}.{context}"),
            keys.values().map(String::as_str).collect(),
        );
    }
    let sticks: Vec<String> = STICKS
        .iter()
        .map(|stick| format!("{EMULATOR_SECTION}.{stick}"))
        .collect();
    let order: Vec<String> = actions
        .keys()
        .cloned()
        .chain([EMULATOR_SECTION.to_string()])
        .chain(sticks.iter().cloned())
        .collect();
    let mut kept: Vec<(String, Vec<String>)> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    for (name, rows) in toml::blocks(text, &[EMULATOR_SECTION]) {
        if !name.is_empty() && !order.contains(&name) {
            removed.push(format!("[{name}]"));
            continue;
        }

        let unknown = |key: &str, value: &::toml::Value| {
            if let Some(actions) = actions.get(&name) {
                return !matches!(value, ::toml::Value::String(value)
                    if actions.contains(value.as_str()));
            }
            if name == EMULATOR_SECTION {
                return !BUTTONS.contains(&key.to_lowercase().as_str());
            }
            !sticks.contains(&name)
                || !STICK_KEYS
                    .into_iter()
                    .chain([SCALE])
                    .any(|each| each == key.to_lowercase())
        };

        let (rows, gone) = toml::drop(&rows, unknown, false);
        removed.extend(gone.into_iter().map(|key| {
            if name.is_empty() {
                key
            } else {
                format!("[{name}] {key}")
            }
        }));
        kept.push((name, rows));
    }
    let at = |name: &str| {
        if name.is_empty() {
            0
        } else {
            order
                .iter()
                .position(|each| each == name)
                .expect("a table kept is in order")
                + 1
        }
    };
    let ordered = py::sorted_by_lt(kept.clone(), |a, b| at(&a.0) < at(&b.0));
    if removed.is_empty() && ordered == kept {
        return (text.to_string(), Vec::new());
    }
    (toml::joined(&ordered), removed)
}

/// Which of the emulator's controls pa3ds's keys are sitting on: what it is called, which key
/// took it, and whether its loss is one to keep quiet about (`QUIET`).
///
/// A key bound with a modifier takes its button only while the modifier is held, and is not said
/// to sit on it: shift and T, touch mode for a day, was chosen so T stays this machine's D-pad
/// up. Nor is Shift alone, touch mode's drag as it ships, which the hook hears
/// and passes on (`core.keys`): a stick's modifier on Shift still walks. Sorted by the control,
/// so the same overlap is reported the same way every run.
pub fn taken(
    bindings: &Bindings,
    emulator: &IndexMap<String, Vec<String>>,
) -> Vec<(String, String, bool)> {
    let mine: HashSet<String> = bindings
        .values()
        .flat_map(|keys| {
            keys.keys()
                .filter(|key| bare(key) == **key && *key != SHIFT)
                .cloned()
        })
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
    prepare_with(defaults, home, config, write, running, explained, None)
}

/// `prepare` with every argument: `contexts` are the keys of each screen that has its own, by
/// default.
pub fn prepare_with(
    defaults: &Bindings,
    home: Option<&Path>,
    config: Option<&Path>,
    write: bool,
    running: bool,
    explained: Option<&IndexMap<String, String>>,
    contexts: Option<&Bindings>,
) -> Result<Keys> {
    let no_contexts = IndexMap::new();
    let contexts = contexts.unwrap_or(&no_contexts);
    let path = keys_path(home)?;
    let config = match config {
        None => emulator_config("")?,
        Some(config) => config.to_path_buf(),
    };
    let mut notes: Vec<String> = Vec::new();
    let text = read_config(&config, &mut notes);
    let mut emulator = emulator_keys(&text)?;

    let mut read: Option<Bindings> = None;
    let mut keyed: Bindings = contexts.clone();
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
                        Some(contexts),
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
            // Tidied first, so a key a renamed action had is free for the new name's line; and
            // again after, for the tables filling in added at the end.
            let (tidied, removed) = tidy(&mine, defaults, Some(contexts));
            if !removed.is_empty() {
                notes.push(format!(
                    "took out of {}, which pa3ds does not do: {}",
                    py::path_str(&path),
                    removed.join(", ")
                ));
                loaded = toml::parse(&tidied).0;
            }
            let filled;
            (filled, added) = fill(&tidied, &loaded, defaults, explained, Some(contexts))?;
            let filled = tidy(&filled, defaults, Some(contexts)).0;
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
            if filled != mine {
                loaded = toml::parse(&filled).0;
                if write && !toml::write(&path, &filled)? {
                    notes.push(format!("{} could not be written", py::path_str(&path)));
                }
            }
            read = read_keys(&path, &loaded, defaults, &mut notes);
            keyed = read_contexts(&path, &loaded, contexts, &mut notes);
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
    for (context, keys) in &keyed {
        let alone: Bindings = IndexMap::from([(context.clone(), keys.clone())]);
        for (control, key, _) in taken(&alone, &emulator) {
            notes.push(format!(
                "pa3ds takes {key} while {context} runs, which is {control}"
            ));
        }
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
        contexts: keyed,
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
