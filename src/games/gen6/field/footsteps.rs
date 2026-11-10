//! pa3ds's own footstep where the game plays none (ADR-0025): at the game's own moment, in sounds
//! a game's `footsteps.toml` chooses by the ground (spike 227, docs/findings/footsteps.md).
//!
//! The game plays a footstep at frames of the walk's animation, 6 and 13 of a walk's fourteen and
//! 7 and 15 of a run's: one a cell walked. Which sound is the ground's, bits 12 to 16 of the word
//! of the cell the player stands in, and the game plays none where they are 0 (spike 225). Each
//! tick pa3ds reads the same: the move model's animation object at +0x98c, with its animation id
//! at +0x4a and its frame at +0x14, and the model's ground word at +0x148; at each footfall it
//! plays the table's clip for the cell's kind if it has one, played over the game's own where the
//! game has one, and otherwise its clip for silent ground where the game plays none. The game
//! plays the first three steps of a walk at full volume, the next three at 0.7 and every one
//! after at 0.49 (`0x007e6430`); pa3ds plays every step at full volume, so its footsteps never
//! fade under the game's sound.
//!
//! On a Pokémon on the sea, or under it, the game plays nothing at all. pa3ds plays the table's
//! clip every so many cells the player is carried, its `every`, by default 4.5 surfing and 3
//! diving: on the usual ride, 7.5 cells a second on the water, a sound every 0.6 s, and on
//! Sharpedo, 15 a second, every 0.3 s. Ground with a clip unlike the last's
//! plays at once, so the seaweed is heard as the player swims into it. The player rides a Pokémon
//! while the model says so (`mmodel.surfing`), whichever Pokémon it is, and is under the sea while
//! the zone is (spike 227).
//!
//! Measured in Omega Ruby and Alpha Sapphire, and in X, whose model keeps the same things a little
//! otherwise, its run another animation and its silent ground other footsteps (`Steps`, spike 231).
//! Sun's is another engine's.
//!
//! The port builds each game's table in (pa3ds/rust-pokemon-access-3ds#16): a player has no
//! reason to edit it, and only the translations ship as files. `shipped` is the table, and
//! `load` reads a table's text where the Python reads its file.
//!
//! Bound by:
//! - ADR-0025: pa3ds's footstep only where the game plays none, at the game's own moment.

use std::collections::HashSet;
use std::path::Path;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::bcsar::{self, Sound};
use crate::core::binary::{f32, u16, u32};
use crate::core::clips;
use crate::core::config::repr_of;
use crate::core::memory::Memory;
use crate::core::rom::Rom;
use crate::core::toml;
use crate::games::common::field::chunk::{FOOTSTEP_SHIFT, KIND_SHIFT, SILENT, footstep_at};
use crate::games::gen6::field::mmodel;
use crate::py::{self, Error, Kind, Log, Result};

/// The move model's animation object.
pub const ANIMATION: i64 = 0x98C;
/// The word of the cell the move model stands in (`0x004dbe1c`), which the game's footstep is
/// chosen by.
pub const GROUND: i64 = 0x148;
/// The animation's frame, a float.
pub const FRAME: i64 = 0x14;
pub const ANIMATION_ID: i64 = 0x4A;
pub const WALK: &str = "walk";
pub const RUN: &str = "run";
/// The animations the game plays footsteps in (`0x007e43b6`), by what the player is doing.
pub const MOVEMENTS: [(i64, &str); 6] = [
    (0x04, WALK),
    (0x6A, WALK),
    (0x79, WALK),
    (0x06, RUN),
    (0x6B, RUN),
    (0x7A, RUN),
];
/// The frames a foot comes down on (`0x007e5eb8`).
pub const FALLS: [(&str, &[i64]); 2] = [(WALK, &[6, 13]), (RUN, &[7, 15])];

/// `FALLS[movement]`.
pub fn falls(movement: &str) -> &'static [i64] {
    if movement == RUN {
        FALLS[1].1
    } else {
        FALLS[0].1
    }
}
pub const SURF: &str = "surf";
pub const DIVE: &str = "dive";
/// Cells carried between one sound and the next, where the table says no `every`.
pub const EVERY: [(&str, f64); 2] = [(SURF, 4.5), (DIVE, 3.0)];
/// Cells moved in one tick that are no swim but a warp: diving or surfacing, into a new zone.
pub const JUMP: f64 = 3.0;
/// A game's table of what to play on which ground, in its data folder.
pub const FILE: &str = "footsteps.toml";
/// The table's key for every ground the game plays no footstep on.
pub const SILENT_GROUND: &str = "silent";
pub const EVERY_KEY: &str = "every";

/// The table built in for the game whose data folder is `data`, by the folder's name, or "" for
/// a game with none.
pub fn shipped(data: &Path) -> &'static str {
    match data.file_name().and_then(|name| name.to_str()) {
        Some("oras") => include_str!("../../../../data/oras/footsteps.toml"),
        Some("xy") => include_str!("../../../../data/xy/footsteps.toml"),
        _ => "",
    }
}

/// Where a game keeps a step's moment and which ground it is silent on: Omega Ruby's by
/// default (`Game.steps`).
///
/// Python's `movements` and `falls` are mappings and `silent` a frozenset, each only looked up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Steps {
    pub animation: i64,
    pub animation_id: i64,
    pub frame: i64,
    pub movements: &'static [(i64, &'static str)],
    pub falls: &'static [(&'static str, &'static [i64])],
    /// Where the cell's word keeps its footstep, five bits (`chunk.footstep`).
    pub shift: i64,
    /// The footsteps whose walk and run play nothing.
    pub silent: &'static [i64],
}

impl Steps {
    /// Whether the game plays no footstep on the cell whose word this is.
    pub fn is_silent(&self, word: i64) -> bool {
        self.silent.contains(&footstep_at(word, self.shift))
    }

    /// `falls[movement]`, which raises where the movement has none.
    #[track_caller]
    fn falls_of(&self, movement: &str) -> Result<&'static [i64]> {
        match self.falls.iter().find(|&&(known, _)| known == movement) {
            Some(&(_, falls)) => Ok(falls),
            None => Err(Error::key(py::repr(movement))),
        }
    }
}

pub const ORAS: Steps = Steps {
    animation: ANIMATION,
    animation_id: ANIMATION_ID,
    frame: FRAME,
    movements: &MOVEMENTS,
    falls: &FALLS,
    shift: FOOTSTEP_SHIFT,
    silent: &[SILENT],
};

#[derive(Debug, Clone)]
struct Ride {
    movement: &'static str,
    x: f64,
    z: f64,
    clip: Option<String>,
    /// Cells carried since the last sound.
    travelled: f64,
}

/// `Callable[[str], None]`: plays a clip by its name, `Clips.play`.
pub type Play = Rc<dyn Fn(&str) -> Result<()>>;
/// `Callable[[int], bool]`: whether a zone is under the sea, `ZoneTable.underwater`.
pub type Underwater = Rc<dyn Fn(i64) -> Result<bool>>;

/// The player's footfalls, followed a tick at a time, and their swimming.
pub struct Footsteps {
    steps: Steps,
    form: Option<i64>,
    memory: Rc<dyn Memory>,
    play: Play,
    grounds: HashSet<String>,
    every: IndexMap<String, f64>,
    underwater: Underwater,
    /// The animation and frame the last tick read, None while no footsteps are played.
    seen: Option<(i64, f64)>,
    /// Where the player was carried to, and how far, None while they ride nothing.
    ride: Option<Ride>,
}

impl Footsteps {
    /// `Footsteps(memory, play, grounds)`: `every` at `EVERY`, and no zone under the sea.
    pub fn new(memory: Rc<dyn Memory>, play: Play, grounds: HashSet<String>) -> Self {
        Footsteps::with(
            memory,
            play,
            grounds,
            &IndexMap::new(),
            Rc::new(|_zone: i64| Ok(false)),
        )
    }

    /// `Footsteps(memory, play, grounds, every, underwater)`: ORAS's `steps`, and the form read
    /// where ORAS keeps it.
    pub fn with(
        memory: Rc<dyn Memory>,
        play: Play,
        grounds: HashSet<String>,
        every: &IndexMap<String, f64>,
        underwater: Underwater,
    ) -> Self {
        Footsteps::with_steps(
            memory,
            play,
            grounds,
            every,
            underwater,
            ORAS,
            Some(mmodel::FORM),
        )
    }

    /// `grounds` names the clips there are, `walk silent` or `dive 0x42` (`clip_name`);
    /// `every` the cells between sounds surfing and diving; `underwater` whether a zone is
    /// under the sea; `steps` where the game keeps a step; `form` where the model says what
    /// the player rides, None where it is not read (`Game.form`).
    pub fn with_steps(
        memory: Rc<dyn Memory>,
        play: Play,
        grounds: HashSet<String>,
        every: &IndexMap<String, f64>,
        underwater: Underwater,
        steps: Steps,
        form: Option<i64>,
    ) -> Self {
        let mut merged: IndexMap<String, f64> = EVERY
            .iter()
            .map(|&(movement, cells)| (movement.to_string(), cells))
            .collect();
        merged.extend(
            every
                .iter()
                .map(|(movement, &cells)| (movement.clone(), cells)),
        );
        Footsteps {
            steps,
            form,
            memory,
            play,
            grounds,
            every: merged,
            underwater,
            seen: None,
            ride: None,
        }
    }

    /// Read the player's move model, and play a footstep if a foot came down since the last
    /// tick on ground the table has a clip for, or a swim's sound if they have been carried
    /// far enough.
    pub fn poll(&mut self, model: i64) -> Result<()> {
        let head = self.memory.read(model, GROUND + 4)?;
        if !mmodel::in_use(&head)? || mmodel::index_of(&head)? != mmodel::PLAYER_INDEX {
            (self.seen, self.ride) = (None, None);
            return Ok(());
        }
        let form = match self.form {
            None => None,
            Some(at) => Some(byte(&self.memory.read(model + at, 1)?, 0)?),
        };
        if byte(&head, mmodel::RIDING)? & mmodel::BIKE != 0 && mmodel::surfing(&head, form)? {
            self.seen = None;
            return self.swim(&head);
        }
        self.ride = None;
        let steps = self.steps;
        let animation = u32(&self.memory.read(model + steps.animation, 4)?, 0)?;
        if animation == 0 {
            self.seen = None;
            return Ok(());
        }
        let body = self.memory.read(animation, steps.animation_id + 2)?;
        let which = u16(&body, steps.animation_id)?;
        let frame = f32(&body, steps.frame)?;
        let Some(&(_, movement)) = steps.movements.iter().find(|&&(id, _)| id == which) else {
            self.seen = None;
            return Ok(());
        };
        let seen = self.seen.replace((which, frame));
        // The game looks only at the frames an animation passes as it plays, never at where a new
        // one starts: a walk begun or turned mid-stride passes no footfall on its first tick.
        let Some((before, then)) = seen else {
            return Ok(());
        };
        if before != which || !came_down(steps.falls_of(movement)?, then, frame) {
            return Ok(());
        }
        if let Some(clip) = self.clip(movement, u32(&head, GROUND)?) {
            (self.play)(&clip)?;
        }
        Ok(())
    }

    fn swim(&mut self, head: &[u8]) -> Result<()> {
        let movement = if (self.underwater)(mmodel::zone_of(head)?)? {
            DIVE
        } else {
            SURF
        };
        let (x, z) = (f32(head, mmodel::WORLD_X)?, f32(head, mmodel::WORLD_Z)?);
        let (clip, every) = (
            self.clip(movement, u32(head, GROUND)?),
            self.every[movement],
        );
        let ride = match &mut self.ride {
            Some(ride) if ride.movement == movement => ride,
            _ => {
                // Getting on, diving or surfacing: the first cell carried plays.
                self.ride = Some(Ride {
                    movement,
                    x,
                    z,
                    clip,
                    travelled: every,
                });
                return Ok(());
            }
        };
        let moved = ((x - ride.x).abs() + (z - ride.z).abs()) / mmodel::CELL as f64;
        (ride.x, ride.z) = (x, z);
        if moved > JUMP {
            return Ok(());
        }
        ride.travelled += moved;
        if clip != ride.clip {
            // `max(ride.travelled, every)`: the first unless `every` is greater.
            let most = if every > ride.travelled {
                every
            } else {
                ride.travelled
            };
            (ride.clip, ride.travelled) = (clip.clone(), most);
        }
        if moved != 0.0
            && let Some(clip) = clip
            && ride.travelled >= every
        {
            ride.travelled = 0.0;
            (self.play)(&clip)?;
        }
        Ok(())
    }

    /// The table's clip for the cell's kind, or else for silent ground where the cell is,
    /// or None where it has neither.
    fn clip(&self, movement: &str, word: i64) -> Option<String> {
        let mut name = clip_name(movement, Some(word >> KIND_SHIFT));
        if !self.grounds.contains(&name) && self.steps.is_silent(word) {
            name = clip_name(movement, None);
        }
        self.grounds.contains(&name).then_some(name)
    }
}

/// `data[at]`: the byte, or the `IndexError` Python raises past the end of `bytes`.
#[track_caller]
fn byte(data: &[u8], at: i64) -> Result<i64> {
    match data.get(at as usize) {
        Some(&byte) => Ok(byte as i64),
        None => Err(Error::index("index out of range")),
    }
}

/// What the clip for a movement on a ground is called: on a kind of cell, `walk 0x24`, or on
/// silent ground, `walk silent`.
///
/// Python's `kind` defaults to None.
pub fn clip_name(movement: &str, kind: Option<i64>) -> String {
    match kind {
        None => format!("{movement} {SILENT_GROUND}"),
        // `f"{kind:#04x}"`, whose sign Python writes before the 0x: "-0x1" is already 4 wide.
        Some(kind) if kind < 0 => format!("{movement} -{:#x}", -kind),
        Some(kind) => format!("{movement} {kind:#04x}"),
    }
}

/// Whether the animation passed a footfall going from frame `before` to `now`, round the
/// end of its cycle where `now` is the lower. At most one a tick, however many it passed.
pub fn came_down(falls: &[i64], before: f64, now: f64) -> bool {
    if now >= before {
        return falls
            .iter()
            .any(|&fall| before < fall as f64 && fall as f64 <= now);
    }
    falls
        .iter()
        .any(|&fall| fall as f64 > before || fall as f64 <= now)
}

/// `int(text, 16)`, which takes a `0x` after the sign, and an underscore after that, as
/// `py::int_base16` does not.
fn hex(text: &str) -> Result<i64> {
    let invalid = || {
        Error::value(format!(
            "invalid literal for int() with base 16: {}",
            py::repr(text)
        ))
    };
    let stripped = py::strip(text);
    let (sign, rest) = match stripped.strip_prefix(['-', '+']) {
        Some(rest) => (&stripped[..1], rest),
        None => ("", stripped),
    };
    match rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X")) {
        Some(digits) if !digits.is_empty() => {
            let digits = digits.strip_prefix('_').unwrap_or(digits);
            py::int_base16(&format!("{sign}{digits}")).map_err(|_| invalid())
        }
        Some(_) => Err(invalid()),
        None => py::int_base16(text),
    }
}

/// Every clip of the table `table` holds as `text`, by its name (`clip_name`): the game's sounds
/// out of the player's ROM at `rom`, WAVs of pa3ds's own in the folder `sounds`, mixed; and the
/// cells between sounds surfing and diving that it sets (`every`). A clip whose sounds are not
/// all there is left out, and the log says so.
pub fn load(
    table: &str,
    text: &str,
    rom: Option<&Path>,
    archive: &str,
    sounds: &Path,
    log: &Log,
) -> Result<(IndexMap<String, Sound>, IndexMap<String, f64>)> {
    let (parsed, broken) = toml::parse(text);
    if !broken.is_empty() || text.is_empty() {
        log(&format!(
            "footsteps are off: {table} {}",
            if broken.is_empty() {
                "is empty"
            } else {
                broken.as_str()
            }
        ));
        return Ok((IndexMap::new(), IndexMap::new()));
    }
    let mut wanted: IndexMap<String, Vec<(String, f64)>> = IndexMap::new();
    let mut every: IndexMap<String, f64> = IndexMap::new();
    for movement in [WALK, RUN, SURF, DIVE] {
        for (ground, layers) in toml::table(parsed.get(movement)) {
            if ground == EVERY_KEY && EVERY.iter().any(|&(swim, _)| swim == movement) {
                // `isinstance(layers, int | float) and layers > 0`, a bool being an int.
                let cells = match layers {
                    ::toml::Value::Integer(cells) => Some(cells as f64),
                    ::toml::Value::Float(cells) => Some(cells),
                    ::toml::Value::Boolean(on) => Some(if on { 1.0 } else { 0.0 }),
                    _ => None,
                };
                match cells {
                    Some(cells) if cells > 0.0 => {
                        every.insert(movement.to_string(), cells);
                    }
                    _ => log(&format!(
                        "footsteps: {movement} every = {} is not a number of cells",
                        repr_of(&layers)
                    )),
                }
                continue;
            }
            let kind = if ground == SILENT_GROUND {
                None
            } else {
                match hex(&ground) {
                    Ok(kind) => Some(kind),
                    Err(error) if error.is(Kind::ValueError) => {
                        log(&format!(
                            "footsteps: {movement} {} is not a ground, left out",
                            py::repr(&ground)
                        ));
                        continue;
                    }
                    Err(error) => return Err(error),
                }
            };
            wanted.insert(clip_name(movement, kind), layers_of(&layers));
        }
    }
    // A set of strings, which Python iterates in an order of its own each run: in the order the
    // table names them here.
    let wanted_sounds: IndexSet<String> = wanted
        .values()
        .flatten()
        .map(|(sound, _)| sound.clone())
        .collect();
    let named: IndexSet<String> = wanted_sounds
        .iter()
        .filter(|sound| !sound.to_lowercase().ends_with(".wav"))
        .cloned()
        .collect();
    let mut found: IndexMap<String, Sound> = IndexMap::new();
    if let (false, Some(rom)) = (named.is_empty(), rom) {
        let read = Rom::new(rom)
            .and_then(|image| bcsar::sounds(&image, archive, named.iter().map(String::as_str)));
        match read {
            Ok(sounds) => found = sounds,
            Err(error)
                if [
                    Kind::RomError,
                    Kind::OsError,
                    Kind::StructError,
                    Kind::KeyError,
                    Kind::IndexError,
                ]
                .iter()
                .any(|&kind| error.is(kind)) =>
            {
                let name = rom
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                log(&format!(
                    "footsteps: the game's own sounds are off, {name}: {}",
                    error.message
                ));
            }
            Err(error) => return Err(error),
        }
    }
    for sound in wanted_sounds.difference(&named) {
        match clips::wave(&sounds.join(sound)) {
            Ok(wave) => {
                found.insert(sound.clone(), wave);
            }
            // OSError, EOFError and wave.Error, the last two a ValueError here (`clips.wave`).
            Err(error) if error.is(Kind::OsError) || error.is(Kind::ValueError) => {
                log(&format!(
                    "footsteps: {sound} cannot be played: {}",
                    error.message
                ));
            }
            Err(error) => return Err(error),
        }
    }
    let mut out: IndexMap<String, Sound> = IndexMap::new();
    for (name, layers) in &wanted {
        let missing: Vec<&str> = layers
            .iter()
            .filter(|(sound, _)| !found.contains_key(sound))
            .map(|(sound, _)| sound.as_str())
            .collect();
        if !missing.is_empty() {
            log(&format!(
                "footsteps: {name} left out, without {}",
                missing.join(", ")
            ));
        } else if !layers.is_empty() {
            let mixed: Vec<(Sound, f64)> = layers
                .iter()
                .map(|(sound, level)| (found[sound].clone(), *level))
                .collect();
            out.insert(name.clone(), clips::mix(&mixed)?);
        }
    }
    let mut names: Vec<&str> = out.keys().map(String::as_str).collect();
    names.sort();
    log(&format!(
        "footsteps: {}",
        if names.is_empty() {
            "none".to_string()
        } else {
            names.join(", ")
        }
    ));
    Ok((out, every))
}

/// A ground's sounds: each a name, or `{ sound = name, volume = 0.5 }`.
fn layers_of(value: &::toml::Value) -> Vec<(String, f64)> {
    let items: Vec<::toml::Value> = match value {
        ::toml::Value::Array(items) => items.clone(),
        other => vec![other.clone()],
    };
    let mut out: Vec<(String, f64)> = Vec::new();
    for item in items {
        if let ::toml::Value::String(sound) = &item {
            out.push((sound.clone(), 1.0));
        } else if let Some(::toml::Value::String(sound)) = toml::table(Some(&item)).get("sound") {
            // `float(level) if isinstance(level, int | float) else 1.0`, a bool being an int.
            let level = match toml::table(Some(&item)).get("volume") {
                Some(::toml::Value::Integer(level)) => *level as f64,
                Some(::toml::Value::Float(level)) => *level,
                Some(::toml::Value::Boolean(on)) => f64::from(u8::from(*on)),
                _ => 1.0,
            };
            out.push((sound.clone(), level));
        }
    }
    out
}
