//! Descriptions of the scenes the game shows without words, in pa3ds's own words (ADR-0038).
//!
//! A scene has no name to look it up by, so it is found by the line of the game's text it follows,
//! in the zone it is said in: a line is a file and a number in the ROM, the same in every language,
//! and the zone tells one scene from another where both say the same words, as "Gotcha! Groudon was
//! caught!" says what every capture does (spike 203). What follows the line is said a **beat** at a
//! time, each when what it waits for comes: at once, a scene of the game's own starting
//! (`app::field_demo::Proc`, which keeps nothing saying which), or the player arriving in a zone;
//! then `at` seconds on. A scene waiting for a demo or a zone is let go when the player arrives
//! anywhere else, as after losing a battle in it. A scene with no line is said each time the player
//! arrives in its zone: a room whose way on nothing shows but the room itself. A line with no zone
//! is said before the player has arrived anywhere: the opening's.
//!
//! Omega Ruby and Alpha Sapphire play their own scenes in rooms of their own, twin zones of one
//! layout (Seafloor Cavern's 0x6f and 0x6e), but some lines are said in a zone both share, as
//! Steven's on Route 128: a scene said there names the version it was described in by its title id.
//!
//! Sun's demos keep their number (`App::AppDemo::AppDemoProc` +0x24, Khai, 2026-09-30), so a scene
//! of Sun's is found by it, `demo = 6` for the chase after Kukui's call, in no zone and after no
//! line.
//!
//! A name the game shows is never written into a description: `{pokemon:383}` is the game's own
//! word for Groudon, out of the player's ROM, and so are `{trainer:N}`, `{item:N}` and `{place:N}`.
//!
//! The Python's `Scenes` keeps the callable it says each beat with, the announcements'
//! `narrate`; here the profile owns the announcements through `&mut`, so it hands the callable to
//! each method that may say a beat instead (`Say`).

use std::rc::Rc;
use std::sync::OnceLock;

use ::toml::Value;
use indexmap::IndexMap;
use regex::{Captures, Regex};

use crate::core::toml;
use crate::games::common::text::{PAGE_BREAKS, Token, tokens};
use crate::py::{self, Kind, Log, Result};

/// A game's descriptions in one language, beside its names (`data/<language>/`).
pub const FILE: &str = "scenes.toml";
/// The one control sequence inside a page that inserts no words (`games.common.text`).
pub const COLOUR: i64 = 0xFF00;

fn word_pattern() -> &'static Regex {
    static WORD: OnceLock<Regex> = OnceLock::new();
    WORD.get_or_init(|| Regex::new(r"\{(\w+):(\d+)\}").expect("a fixed pattern"))
}

/// Where a beat is said: the announcements' `narrate`, handed in by the profile.
pub type Say<'a> = &'a mut dyn FnMut(&str);

/// One piece of a description, and what it waits for after the beat before it.
#[derive(Debug, Clone, PartialEq)]
pub struct Beat {
    pub text: String,
    /// Wait for a scene of the game's own to start.
    pub demo: bool,
    /// Wait for the player to arrive in this zone.
    pub zone: Option<i64>,
    /// Then this many seconds.
    pub at: f64,
}

/// A description and the line it follows: the last page of the line's message, by its index and
/// a pattern of its words, or None for a scene said on arriving or as a demo starts; the zone it
/// is said in; or the number of the demo it is said over.
#[derive(Debug, Clone)]
pub struct Scene {
    pub page: Option<i64>,
    /// Python's pattern, `fullmatch`ed: here anchored at both ends, `\A(?:...)\z`.
    pub words: Option<Regex>,
    pub zone: Option<i64>,
    pub beats: Vec<Beat>,
    pub demo: Option<i64>,
}

/// The index of a message's last page, and a pattern its text matches as the dialogue reader
/// hears it: the words between inserts in order, anything where an insert was, as the player's
/// name is in "Raiden changed into the Magma Suit!". None for a line that is not text.
pub fn last_page(line: &[u8]) -> Result<Option<(i64, Regex)>> {
    let parts = match tokens(line) {
        Ok(parts) => parts,
        Err(error) if error.is(Kind::ValueError) => return Ok(None),
        Err(error) => return Err(error),
    };
    let breaks: Vec<usize> = parts
        .iter()
        .enumerate()
        .filter(|(_, part)| {
            matches!(part, Token::Control(control) if PAGE_BREAKS.contains(&control.code))
        })
        .map(|(i, _)| i)
        .collect();
    let page = breaks.len() as i64;
    let mut pieces: Vec<String> = vec![String::new()];
    let from = breaks.last().map_or(0, |last| last + 1);
    for part in &parts[from..] {
        match part {
            Token::Text(text) => pieces.last_mut().expect("never empty").push_str(text),
            Token::Control(control) if control.code != COLOUR => pieces.push(String::new()),
            Token::Control(_) => {}
        }
    }
    let words: Vec<String> = pieces
        .iter()
        .map(|piece| py::split(piece).join(" "))
        .collect();
    if words.iter().all(|word| word.is_empty()) {
        return Ok(None);
    }
    let joined = words
        .iter()
        .map(|word| regex::escape(word))
        .collect::<Vec<_>>()
        .join(".*?");
    let pattern = Regex::new(&format!(r"(?s)\A(?:{joined})\z")).expect("escaped words");
    Ok(Some((page, pattern)))
}

/// `line(archive, file, number)`: a line of the "story" text or the "text" as the ROM has it.
pub type Line<'a> = &'a dyn Fn(&str, i64, i64) -> Result<Option<Vec<u8>>>;
/// `word(kind, number)`: the game's own word for what a description names, or None.
pub type Word<'a> = &'a dyn Fn(&str, i64) -> Option<String>;

/// The scenes of a file, each with the line it follows out of the ROM and its names in the
/// game's own words. A line is `story:402:30`, the story text's file 402 and its line 30, or
/// `text:147:67` out of the rest of the game's text, in its zone or, with none, before the player
/// is anywhere; a scene with no line is said on arriving in its zone, or as its demo starts,
/// `demo = 6`, where the game numbers them. A scene whose line or names the ROM does not have is
/// left out, and logged. A scene with a `title` is the running `title`'s alone: one version's,
/// said at a line in a zone the other version shares.
///
/// Python reads the file at a path; the binary carries it, so this takes its text, None where
/// the game has no file in the language, as `toml.read` of a missing file is nothing.
pub fn load(
    text: Option<&str>,
    line: Line,
    word: Word,
    log: &Log,
    title: Option<i64>,
) -> Result<Vec<Scene>> {
    let mut scenes: Vec<Scene> = Vec::new();
    for table in tables(toml::read_text(text).get("scene")) {
        // `isinstance(only, int)`, a bool among them, as `int_of` reads it.
        if title.is_some_and(|title| int_of(table.get("title")).is_some_and(|only| only != title)) {
            continue;
        }
        let after = str_of(table.get("after"));
        let zone = int_of(table.get("zone"));
        // `isinstance(demo, int) and not isinstance(demo, bool)`.
        let demo = match table.get("demo") {
            Some(Value::Integer(number)) => Some(*number),
            _ => None,
        };
        let (archive, at) = partition(&after);
        let (file, number) = partition(at);
        let raw = if py::isdigit(file) && py::isdigit(number) {
            line(archive, py::int_base10(file)?, py::int_base10(number)?)?
        } else {
            None
        };
        let page = match raw {
            None => None,
            Some(raw) => last_page(&raw)?,
        };
        let beats: Vec<Option<Beat>> = tables(table.get("beat"))
            .iter()
            .map(|held| beat(held, word))
            .collect();
        let kept: Vec<Beat> = beats.iter().flatten().cloned().collect();
        if (!after.is_empty() && page.is_none())
            || (demo.is_none() && after.is_empty() && zone.is_none())
            || kept.is_empty()
            || kept.len() < beats.len()
        {
            log_left_out(log, &after, demo);
            continue;
        }
        let (page, words) = match page {
            Some((page, words)) => (Some(page), Some(words)),
            None => (None, None),
        };
        scenes.push(Scene {
            page,
            words,
            zone,
            beats: kept,
            demo,
        });
    }
    Ok(scenes)
}

fn log_left_out(log: &Log, after: &str, demo: Option<i64>) {
    let what = match demo {
        _ if !after.is_empty() => after.to_string(),
        None => "arriving".to_string(),
        Some(demo) => format!("demo {demo}"),
    };
    log(&format!(
        "scene after {what}: left out, the ROM does not have its line or its names"
    ));
}

/// `text.partition(":")`'s head and tail.
fn partition(text: &str) -> (&str, &str) {
    match text.split_once(':') {
        Some((head, tail)) => (head, tail),
        None => (text, ""),
    }
}

/// `str(table.get(key, ""))`, for what a scenes file holds: a string as it is, a number or a
/// truth value as Python writes it. Anything else, an array or a table, is written as TOML
/// writes it where Python would write its repr; no file of ours holds one there.
fn str_of(value: Option<&Value>) -> String {
    match value {
        None => String::new(),
        Some(Value::String(text)) => text.clone(),
        Some(Value::Integer(number)) => number.to_string(),
        Some(Value::Float(number)) => py::float_repr(*number),
        Some(Value::Boolean(truth)) => (if *truth { "True" } else { "False" }).to_string(),
        Some(other) => other.to_string(),
    }
}

/// The value as an int, where Python's `isinstance(value, int)` holds: a bool is one.
fn int_of(value: Option<&Value>) -> Option<i64> {
    match value {
        Some(Value::Integer(number)) => Some(*number),
        Some(Value::Boolean(truth)) => Some(*truth as i64),
        _ => None,
    }
}

/// `bool(value)`.
fn truth(value: Option<&Value>) -> bool {
    match value {
        None => false,
        Some(Value::Boolean(truth)) => *truth,
        Some(Value::Integer(number)) => *number != 0,
        Some(Value::Float(number)) => *number != 0.0,
        Some(Value::String(text)) => !text.is_empty(),
        Some(Value::Array(items)) => !items.is_empty(),
        Some(Value::Table(items)) => !items.is_empty(),
        Some(Value::Datetime(_)) => true,
    }
}

/// A TOML array of tables as its tables; none where it is not one.
fn tables(value: Option<&Value>) -> Vec<IndexMap<String, Value>> {
    match value {
        Some(Value::Array(items)) => items.iter().map(|item| toml::table(Some(item))).collect(),
        _ => Vec::new(),
    }
}

/// A beat of the file, its names put in, or None where one is missing or the beat is not.
fn beat(table: &IndexMap<String, Value>, word: Word) -> Option<Beat> {
    let Some(Value::String(text)) = table.get("text") else {
        return None;
    };
    let mut missing = false;
    let text = word_pattern()
        .replace_all(text, |found: &Captures| {
            let said = py::int_base10(&found[2])
                .ok()
                .and_then(|number| word(&found[1], number));
            missing |= said.is_none();
            said.unwrap_or_default()
        })
        .into_owned();
    let at = match table.get("at") {
        None => 0.0,
        Some(Value::Integer(number)) => *number as f64,
        Some(Value::Float(number)) => *number,
        Some(Value::Boolean(truth)) => *truth as i64 as f64,
        Some(_) => return None,
    };
    if missing {
        return None;
    }
    Some(Beat {
        text,
        demo: truth(table.get("demo")),
        zone: int_of(table.get("zone")),
        at,
    })
}

/// Follows the scene the player is in, and says each beat of its description as it comes.
pub struct Scenes {
    scenes: Vec<Scene>,
    clock: Rc<dyn Fn() -> f64>,
    /// The scene followed now, by its place in `scenes`.
    scene: Option<usize>,
    /// The beat said next.
    next: usize,
    /// When the last thing a beat waits for came: the line, a demo, an arrival.
    since: f64,
    /// Whether what the next beat waits for has come.
    met: bool,
}

impl Scenes {
    pub fn new(scenes: Vec<Scene>, clock: Rc<dyn Fn() -> f64>) -> Self {
        Scenes {
            scenes,
            clock,
            scene: None,
            next: 0,
            since: 0.0,
            met: false,
        }
    }

    /// `Scenes(scenes, say)`: on the monotonic clock.
    pub fn with_monotonic(scenes: Vec<Scene>) -> Self {
        Scenes::new(scenes, Rc::new(py::time::monotonic))
    }

    /// A page of dialogue just said, in the zone the player is in.
    pub fn heard(&mut self, page: i64, text: &str, zone: Option<i64>, say: Say) {
        let found = self.scenes.iter().position(|scene| {
            scene.zone == zone
                && scene.page == Some(page)
                && scene
                    .words
                    .as_ref()
                    .is_some_and(|words| words.is_match(text))
        });
        if let Some(found) = found {
            self.start(found, say);
        }
    }

    /// A scene of the game's own has started, by its number where the game keeps one: a beat
    /// waiting for it comes, or else the scene said over it starts.
    pub fn demo(&mut self, number: Option<i64>, say: Say) {
        if self.waiting().is_some_and(|beat| beat.demo) {
            self.came(say);
        } else if let Some(number) = number
            && let Some(found) = self
                .scenes
                .iter()
                .position(|scene| scene.demo == Some(number))
        {
            self.start(found, say);
        }
    }

    /// The player is in another zone: a beat waiting for it comes, a scene waiting for another is
    /// let go, and a scene said on arriving there starts.
    pub fn arrived(&mut self, zone: i64, say: Say) {
        if let Some(beat) = self.waiting()
            && (beat.zone.is_some() || beat.demo)
        {
            if beat.zone == Some(zone) {
                self.came(say);
                return;
            }
            self.scene = None;
        }
        let found = self.scenes.iter().position(|scene| {
            scene.words.is_none() && scene.demo.is_none() && scene.zone == Some(zone)
        });
        if let Some(found) = found {
            self.start(found, say);
        }
    }

    fn start(&mut self, scene: usize, say: Say) {
        self.scene = Some(scene);
        self.next = 0;
        self.met = false;
        self.since = (self.clock)();
        self.poll(say);
    }

    /// Say every beat whose time has come.
    pub fn poll(&mut self, say: Say) {
        while let Some(scene) = self.scene
            && self.next < self.scenes[scene].beats.len()
        {
            let beat = &self.scenes[scene].beats[self.next];
            if ((beat.demo || beat.zone.is_some()) && !self.met)
                || (self.clock)() < self.since + beat.at
            {
                return;
            }
            say(&beat.text);
            self.next += 1;
            self.met = false;
        }
        self.scene = None;
    }

    /// The beat next to be said, while it waits for something still to come.
    fn waiting(&self) -> Option<&Beat> {
        let scene = self.scene?;
        if self.met || self.next >= self.scenes[scene].beats.len() {
            return None;
        }
        Some(&self.scenes[scene].beats[self.next])
    }

    fn came(&mut self, say: Say) {
        self.met = true;
        self.since = (self.clock)();
        self.poll(say);
    }
}
