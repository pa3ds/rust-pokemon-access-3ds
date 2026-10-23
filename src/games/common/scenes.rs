//! Descriptions of the scenes the game shows without words, in pa3ds's own words (ADR-0038).
//!
//! A scene has no name to look it up by, so it is found by the line of the game's text it follows,
//! in the zone it is said in: a line is a file and a number in the ROM, the same in every language,
//! and the zone tells one scene from another where both say the same words, as "Gotcha! Groudon was
//! caught!" says what every capture does (spike 203); a scene may follow any one of several, as
//! Kyogre caught or knocked out plays the same one (Orion, 2026-10-20). What follows the line is
//! said a **beat** at a time, each when what it waits for comes: at once, a scene of the game's own
//! starting (`app::field_demo::Proc`, which keeps nothing saying which), or the player arriving in
//! a zone; then `at` seconds on. A scene waiting for a demo or a zone is let go when the player
//! arrives anywhere else, as after losing a battle in it. A scene with no line is said each time
//! the player arrives in its zone: a room whose way on nothing shows but the room itself. A line
//! with no zone is said before the player has arrived anywhere: the opening's. A scene of one of a
//! zone's triggers is said as the player first steps onto the trigger's cells after arriving: each
//! of the Pokémon League's corridors plays its scene as the player walks across the trigger halfway
//! along it, with no line before or after (Orion, 2026-10-16). A scene of an app is said as the
//! app starts, Omega Ruby's intro before the title screen as `app::field_demo::Proc` and then
//! `movie::MovieProc`, and let go as any other app starts: A skips the intro straight to the title
//! screen, and what it skipped is left unsaid (spike 236).
//!
//! A beat's seconds are the game's, counted in its frames (`GameClock`), where a game's profile can
//! read its count: Omega Ruby's input device manager counts 60 a second through every app, so a
//! beat waits while the emulator is paused and keeps up with fast-forward (spike 236).
//!
//! Omega Ruby and Alpha Sapphire play their own scenes in rooms of their own, twin zones of one
//! layout (Seafloor Cavern's 0x6f and 0x6e), but some lines are said in a zone both share, as
//! Steven's on Route 128: a scene said there names the version it was described in by its title id.
//!
//! Sun's demos keep their number (`App::AppDemo::AppDemoProc` +0x24, Khai, 2026-09-30), so a scene
//! of Sun's is found by it, `demo = 6` for the chase after Kukui's call, in no zone and after no
//! line.
//!
//! When each beat is said is a game's `scenes.toml`, every language's; what it says is the message
//! of its `id` in the language's `scenes.ftl` (ADR-0040), English's where the language has none.
//! A name the game shows is never written into a description: `{ POKEMON(383) }` is the game's own
//! word for Groudon, out of the player's ROM, and so are `TRAINER`, `ITEM` and `PLACE`, a whole
//! line of the game's text, `{ TEXT(123, 91) }`, and the game's own name, `{ GAME() }`. Words the
//! game draws as a picture that no line of its text holds are the picture's words in
//! `pictures.ftl`, `{ PICTURE("push_start") }`, as on any screen. The credits draw the player's own
//! Pokémon, which the save records as the story goes: `{ MEMORY(24) }` is the one it keeps for the
//! 24th moment, in the form it keeps, read from the save as the beat is said (spike 222), and
//! `{ YOURS(24) }` the same Pokémon as the player's: the nickname they gave it, where they still
//! have it, or else `scene-yours` with its name in, "your Mega Latios" (Orion, 2026-10-16).
//! Someone the game never names is `{ SPRITE(179) }`, called what a speaker drawn as sprite 179 is
//! (`pa3ds.core.names`): the player's own name for them first, so a rename is heard in the next
//! beat (`Described`).
//!
//! The Python's `Scenes` keeps the callable it says each beat with, the announcements'
//! `narrate`; here the profile owns the announcements through `&mut`, so it hands the callable to
//! each method that may say a beat instead (`Say`).
//!
//! The port builds each game's `scenes.toml` in (pa3ds/rust-pokemon-access-3ds#16): it is no
//! translation, and only the translations ship as files, the `scenes.ftl` among them. `shipped`
//! is the table, and `load` reads a table's text where the Python reads its file.
//!
//! Bound by:
//! - ADR-0038: a scene's beats are Narration's, after the line, demo, arrival or trigger they
//!   follow; off unless asked for.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::path::Path;
use std::rc::{Rc, Weak};

use ::toml::Value;
use indexmap::IndexMap;
use regex::Regex;

use crate::core::toml;
use crate::core::words::{Function, Words, current};
use crate::games::common::pictures;
use crate::games::common::text::{PAGE_BREAKS, Token, tokens};
use crate::py::{self, Kind, Log, Result};

/// A game's scenes and when each beat of them is said, every language's (`data/<game>/`).
pub const FILE: &str = "scenes.toml";

/// The `scenes.toml` built in for the game whose data folder is `data`, by the folder's name, or
/// "" for a game with none.
pub fn shipped(data: &Path) -> &'static str {
    match data.file_name().and_then(|name| name.to_str()) {
        Some("oras") => include_str!("../../../data/oras/scenes.toml"),
        Some("sm") => include_str!("../../../data/sm/scenes.toml"),
        _ => "",
    }
}
/// The one control sequence inside a page that inserts no words (`games.common.text`).
pub const COLOUR: i64 = 0xFF00;
/// The controls choosing between two forms of the words after them, by the player's gender ("boy"
/// or "girl", "May" or "Brendan") or by a number (`garc.NUMBER_FORM`): the words after one are both
/// forms back to back, the first's letters counted in its second argument's low byte and the
/// other's in its high byte.
pub const CHOICES: [i64; 2] = [0x1100, 0x1101];
/// Where a choice stands in a page's words until they are made a pattern.
pub const CHOICE: &str = "\u{E000}";

/// The functions a description names what the ROM lists with, by what each is the game's word
/// for: `{ POKEMON(383) }`.
pub const NAMED: [(&str, &str); 4] = [
    ("POKEMON", "pokemon"),
    ("TRAINER", "trainer"),
    ("ITEM", "item"),
    ("PLACE", "place"),
];
/// The functions that put in a whole line of the game's text, by its archive: words the game
/// draws as a picture, `{ TEXT(123, 91) }`, "THE END" over the credits' last shooting stars.
pub const LINES: [(&str, &str); 2] = [("TEXT", "text"), ("STORY", "story")];
/// The functions that take a name from the player's own save or names as a beat is said, not
/// from the ROM: `{ MEMORY(24) }` is the Pokémon the save keeps for the credits' 24th moment, the
/// one that won Wally's battle on Victory Road (spike 222), `{ YOURS(24) }` the same Pokémon as the
/// player's own, by the nickname they gave it ("Windsong") or as `scene-yours` says it ("your
/// Mega Latios"), and `{ SPRITE(179) }` Professor Birch.
pub const RECALLED: [(&str, &str); 3] = [
    ("MEMORY", "memory"),
    ("YOURS", "yours"),
    ("SPRITE", "sprite"),
];
pub const FRAMES_A_SECOND: i64 = 60;
/// The most frames the game's count may move on between two looks and be believed, 100 s of
/// game: more is a count read from another game, or from memory that is no longer the count.
pub const LEAP: i64 = 6_000;
/// The function that puts in what the game calls itself, out of the ROM: `{ GAME() }`, "Pokémon
/// Omega Ruby", the words of its title screen's logo.
pub const GAME: &str = "GAME";
/// The function that puts in a picture's words, `{ PICTURE("push_start") }`, by its texture's
/// name: what `pictures.ftl` calls it, or words made from the name (`common.pictures`).
pub const PICTURE: &str = "PICTURE";
/// How the player's own Pokémon they never named is said, its name `$pokemon`: "your Mega
/// Latios".
pub const YOURS: &str = "scene-yours";

/// `recall(kind, number)`: the player's own word for a `RECALLED` name, read as a beat is said.
pub type Recall = Rc<dyn Fn(&str, i64) -> Result<Option<String>>>;

/// Where a beat is said: the announcements' `narrate`, handed in by the profile.
pub type Say<'a> = &'a mut dyn FnMut(&str);

/// One piece of a description, and what it waits for after the beat before it.
#[derive(Debug, Clone, PartialEq)]
pub struct Beat {
    /// Its words' id in each language's `scenes.ftl`.
    pub message: String,
    /// Wait for a scene of the game's own to start.
    pub demo: bool,
    /// Wait for the player to arrive in this zone.
    pub zone: Option<i64>,
    /// Then this many seconds.
    pub at: f64,
}

/// A description and the lines it follows, any one of them: the last page of each line's message,
/// by its index and a pattern of its words, none for a scene said on arriving or as a demo starts;
/// the zone it is said in; the number of the demo it is said over; or the zone's trigger, by its
/// place in the zone's table of them, that starts it.
#[derive(Debug, Clone)]
pub struct Scene {
    /// Python's patterns, `fullmatch`ed: here anchored at both ends, `\A(?:...)\z`.
    pub lines: Vec<(i64, Regex)>,
    pub zone: Option<i64>,
    pub beats: Vec<Beat>,
    pub demo: Option<i64>,
    pub trigger: Option<i64>,
    /// The app that plays it, by its class, `movie::MovieProc`: said as the app starts, and let
    /// go as any other starts.
    pub app: Option<String>,
}

/// The index of a message's last page, and a pattern its text matches as the dialogue reader
/// hears it: the words between inserts in order, anything where an insert was, as the player's
/// name is in "Raiden changed into the Magma Suit!", and either form where the game chooses one,
/// "boy" or "girl" in Norman's "How about our little boy's favorite?". None for a line that is
/// not text.
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
    let mut forms: Vec<(String, String)> = Vec::new();
    let mut choice: Option<(i64, i64)> = None;
    let from = breaks.last().map_or(0, |last| last + 1);
    for part in &parts[from..] {
        match part {
            Token::Text(text) => {
                let mut text = text.clone();
                if let Some((first, other)) = choice.take() {
                    let chars: Vec<char> = text.chars().collect();
                    forms.push((
                        py::slice(&chars, 0, first).iter().collect(),
                        py::slice(&chars, first, first + other).iter().collect(),
                    ));
                    pieces.last_mut().expect("never empty").push_str(CHOICE);
                    text = py::slice_from(&chars, first + other).iter().collect();
                }
                pieces.last_mut().expect("never empty").push_str(&text);
            }
            Token::Control(control)
                if CHOICES.contains(&control.code) && control.args.len() >= 2 =>
            {
                choice = Some((control.args[1] & 0xFF, control.args[1] >> 8));
            }
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
    let mut joined = words
        .iter()
        .map(|word| regex::escape(word))
        .collect::<Vec<_>>()
        .join(".*?");
    for (first, other) in &forms {
        joined = joined.replacen(
            CHOICE,
            &format!("(?:{}|{})", regex::escape(first), regex::escape(other)),
            1,
        );
    }
    let pattern = Regex::new(&format!(r"(?s)\A(?:{joined})\z")).expect("escaped words");
    Ok(Some((page, pattern)))
}

/// `line(archive, file, number)`: a line of the "story" text or the "text" as the ROM has it.
pub type Line<'a> = &'a dyn Fn(&str, i64, i64) -> Result<Option<Vec<u8>>>;
/// `line`, as `Described` keeps it for its messages to call.
pub type Lines = Rc<dyn Fn(&str, i64, i64) -> Result<Option<Vec<u8>>>>;
/// `word(kind, number)`: the game's own word for what a description names, or None.
pub type Word = Rc<dyn Fn(&str, i64) -> Option<String>>;

/// The scenes of a game's `scenes.toml`, its `text`, each with the line it follows out of the ROM, and
/// each beat by its words' id, `known` where the words and every name they hold out of the ROM
/// are there (`Described.known`). A line is `story:402:30`, the story text's file 402 and its
/// line 30, or
/// `text:147:67` out of the rest of the game's text, in its zone or, with none, before the player
/// is anywhere; `after` may list several, `["text:147:67", "text:148:1"]`, and any one of them
/// starts the scene, as a legendary caught or knocked out plays the same one. A scene with no
/// line is said on arriving in its zone, as the player steps onto one of its triggers,
/// `trigger = 0`, or as its demo starts, `demo = 6`, where the game numbers them, or as the app
/// that plays it starts, `app = "movie::MovieProc"`. A scene whose lines or names the ROM does not
/// have is left out, and logged. A scene with a `title` is the running `title`'s alone: one
/// version's, said at a line in a zone the other version shares.
pub fn load(
    text: &str,
    line: Line,
    known: &dyn Fn(&str) -> bool,
    log: &Log,
    title: Option<i64>,
) -> Result<Vec<Scene>> {
    let mut scenes: Vec<Scene> = Vec::new();
    for table in tables(toml::parse(text).0.get("scene")) {
        // `isinstance(only, int)`, a bool among them, as `int_of` reads it.
        if title.is_some_and(|title| int_of(table.get("title")).is_some_and(|only| only != title)) {
            continue;
        }
        // `[after] if isinstance(after, str) else after if isinstance(after, list) else []`.
        let after: Vec<Value> = match table.get("after") {
            Some(Value::String(text)) => vec![Value::String(text.clone())],
            Some(Value::Array(items)) => items.clone(),
            _ => Vec::new(),
        };
        let zone = int_of(table.get("zone"));
        let afters: Vec<String> = after
            .iter()
            .filter(|one| truth(Some(one)))
            .map(|one| str_of(Some(one)))
            .collect();
        // `isinstance(demo, int) and not isinstance(demo, bool)`.
        let demo = match table.get("demo") {
            Some(Value::Integer(number)) => Some(*number),
            _ => None,
        };
        let trigger = match table.get("trigger") {
            Some(Value::Integer(number)) => Some(*number),
            _ => None,
        };
        let app = match table.get("app") {
            Some(Value::String(app)) => Some(app.clone()),
            _ => None,
        };
        let pages: Vec<Option<(i64, Regex)>> = afters
            .iter()
            .map(|one| followed(one, line))
            .collect::<Result<_>>()?;
        let beats: Vec<Option<Beat>> = tables(table.get("beat")).iter().map(beat).collect();
        let kept: Vec<Beat> = beats
            .iter()
            .flatten()
            .filter(|beat| known(&beat.message))
            .cloned()
            .collect();
        if pages.iter().any(Option::is_none)
            || (demo.is_none() && app.is_none() && afters.is_empty() && zone.is_none())
            || kept.is_empty()
            || kept.len() < beats.len()
        {
            log_left_out(log, &afters.join(" or "), demo, trigger, app.as_deref());
            continue;
        }
        scenes.push(Scene {
            lines: pages.into_iter().flatten().collect(),
            zone,
            beats: kept,
            demo,
            trigger,
            app,
        });
    }
    Ok(scenes)
}

fn log_left_out(
    log: &Log,
    after: &str,
    demo: Option<i64>,
    trigger: Option<i64>,
    app: Option<&str>,
) {
    let what = match demo {
        _ if !after.is_empty() => after.to_string(),
        _ if let Some(trigger) = trigger => format!("trigger {trigger}"),
        _ if let Some(app) = app => format!("app {app}"),
        None => "arriving".to_string(),
        Some(demo) => format!("demo {demo}"),
    };
    log(&format!(
        "scene after {what}: left out, the ROM does not have its line or its names"
    ));
}

/// The last page of a line a scene follows, `story:402:30`, or None where the ROM has none.
fn followed(after: &str, line: Line) -> Result<Option<(i64, Regex)>> {
    let (archive, at) = partition(after);
    let (file, number) = partition(at);
    let raw = if py::isdigit(file) && py::isdigit(number) {
        line(archive, py::int_base10(file)?, py::int_base10(number)?)?
    } else {
        None
    };
    match raw {
        None => Ok(None),
        Some(raw) => last_page(&raw),
    }
}

/// `text.partition(":")`'s head and tail.
fn partition(text: &str) -> (&str, &str) {
    match text.split_once(':') {
        Some((head, tail)) => (head, tail),
        None => (text, ""),
    }
}

/// `str(one)` of a line a scene's `after` lists: a string as it is, a number or a truth value as
/// Python writes it. Anything else, an array or a table, is written as TOML writes it where Python
/// would write its repr; no file of ours holds one there.
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

/// A beat of the file, or None where it is not one.
fn beat(table: &IndexMap<String, Value>) -> Option<Beat> {
    let Some(Value::String(message)) = table.get("id") else {
        return None;
    };
    let at = match table.get("at") {
        None => 0.0,
        Some(Value::Integer(number)) => *number as f64,
        Some(Value::Float(number)) => *number,
        Some(Value::Boolean(truth)) => *truth as i64 as f64,
        Some(_) => return None,
    };
    Some(Beat {
        message: message.clone(),
        demo: truth(table.get("demo")),
        zone: int_of(table.get("zone")),
        at,
    })
}

/// The words of the scenes' beats in the player's language, the names in them put in: the
/// game's own out of the ROM, and the player's out of their save and their names as each beat
/// is said.
///
/// A name that cannot be found, or whose reading raises, leaves its beat unsaid: Python's Fluent
/// keeps a function's exception with the words, and a message with an error is not said
/// (`Words.get`).
pub struct Described {
    words: Rc<Words>,
    /// Whether a beat is being looked over as its scene loads, before any save is read.
    checking: Rc<Cell<bool>>,
}

impl Described {
    /// `word` is the game's word for a Pokémon, a trainer, an item or a place by its
    /// number, `line` a line of its text, and `recall` the player's own for what `RECALLED`
    /// names: None where there is none.
    pub fn new(line: Lines, word: Word, recall: Option<Recall>) -> Result<Described> {
        let checking = Rc::new(Cell::new(false));
        // The words the functions are given to, for YOURS's own message: they hold the functions,
        // so the functions hold them weakly.
        let these: Rc<RefCell<Weak<Words>>> = Rc::new(RefCell::new(Weak::new()));
        let mut functions: IndexMap<String, Function> = IndexMap::new();
        for (name, kind) in NAMED {
            let word = word.clone();
            functions.insert(
                name.to_string(),
                Rc::new(move |values| {
                    let [number] = values else {
                        return None;
                    };
                    word(kind, whole(number)?)
                }),
            );
        }
        for (name, archive) in LINES {
            let line = line.clone();
            functions.insert(
                name.to_string(),
                Rc::new(move |values| {
                    let [file, number] = values else {
                        return None;
                    };
                    let raw = line(archive, whole(file)?, whole(number)?).ok()?;
                    words(raw.as_deref())
                }),
            );
        }
        for (name, kind) in RECALLED {
            let (recall, checking, these) = (recall.clone(), checking.clone(), these.clone());
            functions.insert(
                name.to_string(),
                Rc::new(move |values| {
                    let [number] = values else {
                        return None;
                    };
                    if checking.get() {
                        return Some(kind.to_string());
                    }
                    let moment = whole(number)?;
                    let recall = recall.as_ref()?;
                    let mut said = recall(kind, moment).ok()?;
                    if said.is_none()
                        && kind == "yours"
                        && let Some(species) = recall("memory", moment).ok()?
                    {
                        said = these
                            .borrow()
                            .upgrade()?
                            .get(YOURS, &[("pokemon", &species)]);
                    }
                    said
                }),
            );
        }
        functions.insert(
            GAME.to_string(),
            Rc::new(move |values| {
                if !values.is_empty() {
                    return None;
                }
                word("game", 0)
            }),
        );
        functions.insert(
            PICTURE.to_string(),
            Rc::new(|values| {
                let [texture] = values else {
                    return None;
                };
                Some(pictured(texture))
            }),
        );
        let words = Rc::new(current().with_functions(functions)?);
        *these.borrow_mut() = Rc::downgrade(&words);
        Ok(Described { words, checking })
    }

    /// Whether a beat has words, and the ROM every name of the game's in them.
    pub fn known(&self, message: &str) -> bool {
        self.checking.set(true);
        let found = self.words.get(message, &[]).is_some();
        self.checking.set(false);
        found
    }

    /// A beat's words as it is said, or None where the save has none of a name in them: a
    /// moment the player's game never recorded is not described.
    pub fn said(&self, message: &str) -> Option<String> {
        self.words.get(message, &[])
    }
}

/// A picture's words by its texture's name (`pictures.words`). `str(texture)` of a number, which
/// no file passes, is written as an int where it has no fraction.
fn pictured(texture: &crate::core::words::Value) -> String {
    let texture = match texture {
        crate::core::words::Value::Text(text) => text.clone(),
        crate::core::words::Value::Number(number) if number.fract() == 0.0 => {
            (*number as i64).to_string()
        }
        crate::core::words::Value::Number(number) => py::float_repr(*number),
    };
    pictures::words(&texture)
}

/// `int(str(number))` of a value Fluent hands a function: a number written without a fraction.
fn whole(value: &crate::core::words::Value) -> Option<i64> {
    match value {
        crate::core::words::Value::Number(number) if number.fract() == 0.0 => Some(*number as i64),
        crate::core::words::Value::Number(_) => None,
        crate::core::words::Value::Text(text) => py::int_base10(text).ok(),
    }
}

/// A line of the game's text as one run of words, its line breaks spaces; None for none.
fn words(raw: Option<&[u8]>) -> Option<String> {
    let parts = match raw {
        None => Vec::new(),
        Some(raw) => match tokens(raw) {
            Ok(parts) => parts,
            Err(_) => return None,
        },
    };
    let joined: String = parts
        .iter()
        .filter_map(|part| match part {
            Token::Text(text) => Some(text.as_str()),
            Token::Control(_) => None,
        })
        .collect();
    let said = py::split(&joined).join(" ");
    if said.is_empty() { None } else { Some(said) }
}

/// `clock()`: the seconds a scene's beats are timed by. Python's may raise, as `GameClock`'s
/// reading of the game's frames may.
pub type Clock = Rc<dyn Fn() -> Result<f64>>;

/// Seconds as the game counts its frames, so a beat waits while the emulator is paused and
/// keeps up with fast-forward; seconds of the wall clock while the count cannot be read, or goes
/// back or jumps further than the game could run (`LEAP`).
pub struct GameClock {
    frames: Rc<dyn Fn() -> Result<Option<i64>>>,
    wall: Rc<dyn Fn() -> f64>,
    now: Cell<f64>,
    /// The count and the wall clock as last looked at.
    last: Cell<Option<(Option<i64>, f64)>>,
}

impl GameClock {
    pub fn new(frames: Rc<dyn Fn() -> Result<Option<i64>>>, wall: Rc<dyn Fn() -> f64>) -> Self {
        GameClock {
            frames,
            wall,
            now: Cell::new(0.0),
            last: Cell::new(None),
        }
    }

    /// `GameClock(frames)`: on the monotonic clock.
    pub fn with_monotonic(frames: Rc<dyn Fn() -> Result<Option<i64>>>) -> Self {
        GameClock::new(frames, Rc::new(py::time::monotonic))
    }

    /// `clock()`.
    pub fn call(&self) -> Result<f64> {
        let (frames, wall) = ((self.frames)()?, (self.wall)());
        if let Some((counted, then)) = self.last.get() {
            match (frames, counted) {
                (Some(frames), Some(counted))
                    if 0 <= frames - counted && frames - counted <= LEAP =>
                {
                    self.now
                        .set(self.now.get() + (frames - counted) as f64 / FRAMES_A_SECOND as f64);
                }
                _ => self.now.set(self.now.get() + (wall - then)),
            }
        }
        self.last.set(Some((frames, wall)));
        Ok(self.now.get())
    }
}

/// Follows the scene the player is in, and says each beat of its description as it comes.
pub struct Scenes {
    scenes: Vec<Scene>,
    /// A beat's words as it is said, by their id, or None where they are not to be said
    /// (`Described.said`).
    said: Rc<dyn Fn(&str) -> Option<String>>,
    clock: Clock,
    /// The scene followed now, by its place in `scenes`.
    scene: Option<usize>,
    /// The beat said next.
    next: usize,
    /// When the last thing a beat waits for came: the line, a demo, an arrival.
    since: f64,
    /// Whether what the next beat waits for has come.
    met: bool,
    /// The triggers of the zone stepped onto since arriving in it, each said once: a
    /// corridor's trigger is armed only until its scene has played, and walking back over it
    /// plays nothing.
    stepped_on: HashSet<i64>,
}

impl Scenes {
    pub fn new(scenes: Vec<Scene>, said: Rc<dyn Fn(&str) -> Option<String>>, clock: Clock) -> Self {
        Scenes {
            scenes,
            said,
            clock,
            scene: None,
            next: 0,
            since: 0.0,
            met: false,
            stepped_on: HashSet::new(),
        }
    }

    /// `Scenes(scenes, say, said)`: on the monotonic clock.
    pub fn with_monotonic(scenes: Vec<Scene>, said: Rc<dyn Fn(&str) -> Option<String>>) -> Self {
        Scenes::new(scenes, said, Rc::new(|| Ok(py::time::monotonic())))
    }

    /// The triggers of a zone that start a scene, still to be stepped onto: where nothing is
    /// wanted, the field's triggers are not read.
    pub fn triggers(&self, zone: i64) -> HashSet<i64> {
        self.scenes
            .iter()
            .filter(|scene| scene.zone == Some(zone))
            .filter_map(|scene| scene.trigger)
            .filter(|trigger| !self.stepped_on.contains(trigger))
            .collect()
    }

    /// The player has stepped onto one of the zone's triggers: the scene it starts starts, the
    /// first time since arriving.
    pub fn stepped(&mut self, zone: i64, trigger: i64, say: Say) -> Result<()> {
        if !self.stepped_on.insert(trigger) {
            return Ok(());
        }
        let found = self
            .scenes
            .iter()
            .position(|scene| scene.zone == Some(zone) && scene.trigger == Some(trigger));
        if let Some(found) = found {
            self.start(found, say)?;
        }
        Ok(())
    }

    /// A page of dialogue just said, in the zone the player is in.
    pub fn heard(&mut self, page: i64, text: &str, zone: Option<i64>, say: Say) -> Result<()> {
        let found = self.scenes.iter().position(|scene| {
            scene.zone == zone
                && scene
                    .lines
                    .iter()
                    .any(|(at, words)| *at == page && words.is_match(text))
        });
        if let Some(found) = found {
            self.start(found, say)?;
        }
        Ok(())
    }

    /// A scene of the game's own has started, by its number where the game keeps one: a beat
    /// waiting for it comes, or else the scene said over it starts.
    pub fn demo(&mut self, number: Option<i64>, say: Say) -> Result<()> {
        if self.waiting().is_some_and(|beat| beat.demo) {
            self.came(say)?;
        } else if let Some(number) = number
            && let Some(found) = self
                .scenes
                .iter()
                .position(|scene| scene.demo == Some(number))
        {
            self.start(found, say)?;
        }
        Ok(())
    }

    /// An app has started, in the zone the player is in: a scene another app plays is let go, the
    /// player having skipped it or it having ended, and a scene of this one starts.
    pub fn started(&mut self, app: &str, zone: Option<i64>, say: Say) -> Result<()> {
        if let Some(scene) = self.scene
            && self.scenes[scene]
                .app
                .as_deref()
                .is_some_and(|playing| playing != app)
        {
            self.scene = None;
        }
        let found = self
            .scenes
            .iter()
            .position(|scene| scene.app.as_deref() == Some(app) && scene.zone == zone);
        if let Some(found) = found {
            self.start(found, say)?;
        }
        Ok(())
    }

    /// The player is in another zone: a beat waiting for it comes, a scene waiting for another is
    /// let go, and a scene said on arriving there starts.
    pub fn arrived(&mut self, zone: i64, say: Say) -> Result<()> {
        self.stepped_on.clear();
        if let Some(beat) = self.waiting()
            && (beat.zone.is_some() || beat.demo)
        {
            if beat.zone == Some(zone) {
                return self.came(say);
            }
            self.scene = None;
        }
        let found = self.scenes.iter().position(|scene| {
            scene.lines.is_empty()
                && scene.demo.is_none()
                && scene.app.is_none()
                && scene.trigger.is_none()
                && scene.zone == Some(zone)
        });
        if let Some(found) = found {
            self.start(found, say)?;
        }
        Ok(())
    }

    fn start(&mut self, scene: usize, say: Say) -> Result<()> {
        // Python's one assignment reads the clock before it sets anything.
        let since = (self.clock)()?;
        self.scene = Some(scene);
        self.next = 0;
        self.met = false;
        self.since = since;
        self.poll(say)
    }

    /// Say every beat whose time has come.
    pub fn poll(&mut self, say: Say) -> Result<()> {
        while let Some(scene) = self.scene
            && self.next < self.scenes[scene].beats.len()
        {
            let beat = &self.scenes[scene].beats[self.next];
            if ((beat.demo || beat.zone.is_some()) && !self.met)
                || (self.clock)()? < self.since + beat.at
            {
                return Ok(());
            }
            if let Some(text) = (self.said)(&beat.message) {
                say(&text);
            }
            self.next += 1;
            self.met = false;
        }
        self.scene = None;
        Ok(())
    }

    /// The beat next to be said, while it waits for something still to come.
    fn waiting(&self) -> Option<&Beat> {
        let scene = self.scene?;
        if self.met || self.next >= self.scenes[scene].beats.len() {
            return None;
        }
        Some(&self.scenes[scene].beats[self.next])
    }

    fn came(&mut self, say: Say) -> Result<()> {
        let since = (self.clock)()?;
        self.met = true;
        self.since = since;
        self.poll(say)
    }
}
