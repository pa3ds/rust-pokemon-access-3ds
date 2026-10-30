//! Pokémon-Amie's Berry Picker: which berry each Pokémon wants, and where it is given (spike 260).
//!
//! Three berries hang in the tree along the top of the bottom screen, and the player's Pokémon pop up
//! along the bottom, each with a bubble showing the berry it wants; a berry dragged from the tree to
//! the Pokémon that wants it is eaten, and a Pokémon waits two or three seconds. A tap knocks a berry
//! down where it hangs, and gives it to nobody (as_amie_berry_picker). The game names neither the
//! berries nor who wants which.
//!
//! The berries are pictures of `kinomi00.bclim` and on, each the game's item by the table here, so
//! touch mode says the item's name from the game's text (`BERRIES`). Each bubble is a group of the
//! bubble layout's, `balloon00.bclim` with the berry wanted drawn inside it, the layout keeping its
//! spare bubbles drawn off the screen. The Pokémon are 3D models, held by the game's
//! `app::nuts2::Model`, three records, one a Pokémon:
//!
//! ```text
//! Model   +0x0C  a record a Pokémon, 0x2C bytes each
//! record  +0x04  -> its `PokemonParam`, a copy of the party's
//!         +0x14  x: -120 on the left, 0 in the middle, 120 on the right
//!         +0x18  y: 15 while up, -255 gone, rising and sinking between
//! ```
//!
//! So a bubble on the screen names its berry, and the Pokémon standing where it is wants it: it is
//! offered in touch mode as "Treecko wants Pinap Berry", a place a drag lets go on, at the Pokémon's
//! body under the bubble, and said as the bubble comes. Let go there, or on the bubble, the berry
//! was eaten and a note showed in the bubble; the misses were Pokémon gone before the berry came
//! (spike 260, live from as_amie_berry_picker cut at the round's start). The opening before READY
//! shows three Pokémon wanting berries too, a demonstration, and is said nothing of: its
//! `app::nuts2::Demo` says when it is over (`OVER`).
//!
//! The round's three Pokémon are drawn from the party, and each keeps its record all round wherever
//! it pops up (spike 262), so Berry Picker's own keys name them by it: a key a berry, picked up off
//! the tree, and a key a Pokémon, given it (`BerryKeys`).
//!
//! Bound by:
//! - ADR-0009: a berry is dragged to the Pokémon that wants it, on the player's keys, by touch
//!   mode's or by Berry Picker's own, a berry's and then a Pokémon's.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::discovery::HEAP_SPAN;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::core::words::own;
use crate::games::common::field::keys::BERRIES as BERRY_KEYS;
use crate::games::common::field::keys::BERRY_PICKER;
use crate::games::common::field::keys::POKEMON as POKEMON_KEYS;
use crate::games::common::layout::Layouts;
use crate::games::common::touch::{HEIGHT, ImageOf, Target, TouchTargets, WIDTH};
use crate::games::common::{pokemon, strbuf};
use crate::games::gen6::screens::training::pictures;
use crate::py::{self, Error, Result};

pub const APP: &str = "N3app5nuts27ProcessE";
/// Where the app's object counts the round's clock down in frames, 30 a second at full speed from
/// 1800, through the round and on to 0 (spike 260, spike 262).
pub const CLOCK: i64 = 0x38;
pub const CLOCK_RATE: i64 = 30;
/// What the clock stands at before a round's START!, and again from each Play Again (spike 260).
pub const FULL: i64 = 60 * CLOCK_RATE;
/// More frames than any round's clock counts: a word past it is not one.
pub const LONGEST: i64 = 20 * 60 * CLOCK_RATE;
/// How long the clock is watched to say how fast the game runs: a player lowers Azahar's speed
/// limit for a higher score, and a drag is timed by the game's frames (`speed`).
pub const PACE_SECONDS: f64 = 0.5;
pub const MODEL: &str = "N3app5nuts25ModelE";
/// The game's opening, "Pick Berries for your Pokémon!", READY and START!, which shows three
/// Pokémon wanting berries before the round: a demonstration, said nothing of (Orion, 2026-10-28).
pub const DEMO: &str = "N3app5nuts24DemoE";
/// Where `DEMO` says whether it is over: 0 while it plays, 1 from just after START!, through the
/// round and its results (spike 260).
pub const OVER: i64 = 0x1C;
/// Where `DEMO` points at a `gfl::str::StrBuf` of its words, which tells it from the words like
/// its vtable the sweep finds in the module's code.
pub const TEXT: i64 = 0x04;
/// Each berry's picture and its item number, the line of the items' names the game calls it by:
/// Easy's Razz, Bluk and Pinap, read off the screen (as_amie_berry_picker), and Normal's Wepear and
/// Magost and Hard's Durin and Nomel, by their textures in the ROM, as Bulbapedia's Berry Picker page
/// adds them, and played by those names (or_amie_berry_picker_normal and _hard). In this order they
/// hang left to right on every difficulty's tree (spike 262). `kinomi07` to `kinomi09` were seen in
/// no game.
pub const BERRIES: [(&str, i64); 7] = [
    ("kinomi00.bclim", 164),
    ("kinomi01.bclim", 165),
    ("kinomi02.bclim", 168),
    ("kinomi03.bclim", 167),
    ("kinomi04.bclim", 176),
    ("kinomi05.bclim", 182),
    ("kinomi06.bclim", 178),
];
pub const BUBBLE: &str = "balloon00.bclim";
pub const BERRY: &str = "kinomi";
pub const IN_BUBBLE: f64 = 50.0;
/// A bubble's berry is drawn 50 above the bubble's place and level with it, within `NEAR`. The
/// game's opening, "Pick Berries for your Pokémon!", draws two empty bubbles in the layout of a tree
/// of its own, and each berry of that tree was said wanted, twice (spike 260).
pub const NEAR: f64 = 10.0;
pub const RECORDS: i64 = 0x0C;
pub const SIZE: i64 = 0x2C;
pub const COUNT: i64 = 3;
pub const PARAM: i64 = 0x04;
/// x, then y.
pub const PLACE: i64 = 0x14;
/// How far either side of the middle a Pokémon or a bubble is in the middle place: the bubbles
/// stand at -104, 0 and 105, the Pokémon at -120, 0 and 120.
pub const APART: i64 = 50;
/// How far down the screen a berry is let go on the Pokémon, near its feet: it fed Treecko, Ralts
/// and Poochyena on the left, in the middle and on the right.
pub const BODY: i64 = 215;
pub const WANTS: &str = "berry-picker-wants";
pub const FOR: &str = "berry-picker-for";
pub const WANTED: &str = "berry-picker-wanted";
pub const LINEUP: &str = "berry-picker-lineup";
pub const NOT_UP: &str = "berry-picker-not-up";
pub const NO_BERRY: &str = "berry-picker-no-berry";
/// What Berry Picker's keys say, in place of the game's own error sound where it is off
/// (`BerryKeys`).
pub const NO_POKEMON: &str = "berry-picker-no-pokemon";
/// What the log calls what Berry Picker says.
pub const LOG: &str = "berry picker";

/// What a Pokémon of the player's is called (`Gen6Profile._yours`).
pub type Name = Rc<dyn Fn(&pokemon::Pokemon) -> Option<String>>;

/// A bubble on the screen: its berry's picture, where the Pokémon under it is, and what it
/// wants, said.
#[derive(Debug, Clone, PartialEq)]
pub struct Want {
    pub pane: i64,
    pub x: i64,
    pub said: String,
}

/// Says which berry each of Berry Picker's Pokémon wants, and offers each to be given one.
pub struct BerryPicker {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    layouts: Rc<Layouts>,
    image_of: ImageOf,
    /// An image -> the game's word for it (`touch_words`): a berry's item name.
    words: IndexMap<String, String>,
    /// What a Pokémon of the player's is called.
    name: Name,
    clock: Rc<dyn Fn() -> f64>,
    /// Model object -> the vtable it was found with.
    models: IndexMap<i64, i64>,
    /// Demo object -> the vtable it was found with.
    demos: IndexMap<i64, i64>,
    /// The app's own object -> the vtable it was found with: the round's clock.
    apps: IndexMap<i64, i64>,
    /// The bubbles shown at the last poll, by their berry's picture and what it said.
    ///
    /// A set only tested.
    said: HashSet<(i64, String)>,
    /// When the round's clock was last looked at, and what it said.
    paced: Option<(f64, i64)>,
    speed: f64,
    /// Whether the round about to start has had its three Pokémon said (`lineup`).
    told: bool,
    /// Whether a want is said berry first, "Pinap Berry for Pikachu" (`[pa3ds.amie]`).
    berry_first: bool,
    /// Whether one want is said at a time, the next once a berry is given or the one said
    /// has gone (`[pa3ds.amie]`).
    one_at_a_time: bool,
    /// The bubbles come and not yet said, by pane and words, in the order they came.
    waiting: Vec<(i64, String)>,
    /// The bubble said last, waiting to be given a berry or to go.
    current: Option<(i64, String)>,
}

impl BerryPicker {
    /// What the heap is swept for: the app's own object, whose clock says how fast the game
    /// runs, the Pokémon, who want each berry, and the opening, which says whether the round
    /// has begun.
    pub const CLASSES: [&'static str; 3] = [APP, MODEL, DEMO];

    /// `BerryPicker(...)` with Python's default clock, `time.monotonic`.
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        words: IndexMap<String, String>,
        name: Name,
    ) -> Self {
        BerryPicker::with_clock(
            memory,
            names,
            layouts,
            image_of,
            words,
            name,
            Rc::new(py::time::monotonic),
        )
    }

    pub fn with_clock(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        words: IndexMap<String, String>,
        name: Name,
        clock: Rc<dyn Fn() -> f64>,
    ) -> Self {
        BerryPicker::with_settings(
            memory, names, layouts, image_of, words, name, clock, false, false,
        )
    }

    /// `BerryPicker(...)` with every argument, `berry_first` and `one_at_a_time` among them.
    #[allow(clippy::too_many_arguments)]
    pub fn with_settings(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        words: IndexMap<String, String>,
        name: Name,
        clock: Rc<dyn Fn() -> f64>,
        berry_first: bool,
        one_at_a_time: bool,
    ) -> Self {
        BerryPicker {
            memory,
            names,
            layouts,
            image_of,
            words,
            name,
            clock,
            models: IndexMap::new(),
            demos: IndexMap::new(),
            apps: IndexMap::new(),
            said: HashSet::new(),
            paced: None,
            speed: 1.0,
            told: false,
            berry_first,
            one_at_a_time,
            waiting: Vec::new(),
            current: None,
        }
    }

    /// The models, demonstrations and apps a sweep found, by class: all of them, or with `adding`
    /// the new ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut kept: IndexMap<&str, IndexMap<i64, i64>> = [
            (MODEL, &self.models),
            (DEMO, &self.demos),
            (APP, &self.apps),
        ]
        .into_iter()
        .map(|(kind, have)| {
            (
                kind,
                if adding {
                    have.clone()
                } else {
                    IndexMap::new()
                },
            )
        })
        .collect();
        for (&at, name) in found {
            if let Some(objects) = kept.get_mut(name.as_str()) {
                objects.insert(at, u32(&self.memory.read(at, 4)?, 0)?);
            }
        }
        self.models = kept.shift_remove(MODEL).unwrap_or_default();
        self.demos = kept.shift_remove(DEMO).unwrap_or_default();
        self.apps = kept.shift_remove(APP).unwrap_or_default();
        Ok(())
    }

    /// How fast the game runs beside full speed, by the round's clock: 1 until it has been seen
    /// to run.
    pub fn speed(&self) -> f64 {
        self.speed
    }

    /// The round's three Pokémon as it is about to start, and what a bubble that has just
    /// come says while it runs, as the screen's words, each with what the log calls it.
    pub fn poll(&mut self, running: Option<&str>) -> Result<Vec<(String, String, Category)>> {
        Ok(self
            .heard(running)?
            .into_iter()
            .map(|line| (LOG.to_string(), line, Category::Screen))
            .collect())
    }

    fn heard(&mut self, running: Option<&str>) -> Result<Vec<String>> {
        let mut lineup = self.lineup(running)?;
        if !self.playing(running)? {
            self.said = HashSet::new();
            self.paced = None;
            (self.waiting, self.current) = (Vec::new(), None);
            return Ok(lineup);
        }
        self.pace()?;
        let shown: Vec<(i64, String)> = self
            .wants()?
            .into_iter()
            .map(|want| (want.pane, want.said))
            .collect();
        let new: Vec<(i64, String)> = shown
            .iter()
            .filter(|each| !self.said.contains(*each))
            .cloned()
            .collect();
        self.said = shown.into_iter().collect();
        if !self.one_at_a_time {
            lineup.extend(new.into_iter().map(|(_, said)| said));
            return Ok(lineup);
        }
        // One at a time (Orion, 2026-10-29): three said together took 1.5 s, the first of them
        // all a player can act on. The next waits until a berry is given or the one said goes.
        let mut waiting: Vec<(i64, String)> = self
            .waiting
            .iter()
            .filter(|each| self.said.contains(*each))
            .cloned()
            .collect();
        waiting.extend(new);
        self.waiting = waiting;
        if self
            .current
            .as_ref()
            .is_none_or(|current| !self.said.contains(current))
        {
            self.current = None;
        }
        if self.current.is_none() && !self.waiting.is_empty() {
            let current = self.waiting.remove(0);
            lineup.push(current.1.clone());
            self.current = Some(current);
        }
        Ok(lineup)
    }

    /// A berry has been given, rightly or not: the next Pokémon waiting is said
    /// (`one_at_a_time`).
    pub fn given(&mut self) {
        self.current = None;
    }

    /// The round's three Pokémon in the game's own order, Berry Picker's own keys' order, said
    /// once while its clock stands full before START! and again before each Play Again's: the
    /// keys are learnt too late by pressing them as the round begins (Orion, 2026-10-29).
    fn lineup(&mut self, running: Option<&str>) -> Result<Vec<String>> {
        let frames = if running == Some(APP) {
            self.frames()?
        } else {
            None
        };
        if frames.is_none_or(|frames| frames < FULL) {
            self.told = false;
            return Ok(Vec::new());
        }
        let names: Vec<Option<String>> = self
            .records()?
            .into_iter()
            .map(|(called, _, _)| called)
            .collect();
        if self.told || names.len() as i64 != COUNT || names.iter().any(Option::is_none) {
            return Ok(Vec::new());
        }
        self.told = true;
        let names: Vec<String> = names.into_iter().flatten().collect();
        Ok(vec![own(
            LINEUP,
            &[
                ("first", &names[0]),
                ("second", &names[1]),
                ("third", &names[2]),
            ],
        )])
    }

    /// Each Pokémon wanting a berry, to let a dragged berry go on, while the round runs.
    pub fn targets(&self, running: Option<&str>) -> Result<Vec<Target>> {
        if !self.playing(running)? {
            return Ok(Vec::new());
        }
        Ok(self.wants()?.iter().map(target).collect())
    }

    /// The round's `index`-th Pokémon in the game's own order, which keeps each its place in it
    /// all round wherever it pops up (spike 262), and where to give it a berry while it is up
    /// wanting one, else None; None with no such Pokémon.
    pub fn pokemon(
        &self,
        index: usize,
        running: Option<&str>,
    ) -> Result<Option<(String, Option<Target>)>> {
        let records = self.records()?;
        let Some((Some(called), place, y)) = records.get(index).cloned() else {
            return Ok(None);
        };
        if !self.playing(running)? || records.iter().any(|other| other.1 == place && other.2 > y) {
            return Ok(Some((called, None)));
        }
        let want = self
            .wants()?
            .into_iter()
            .find(|want| self::place(want.x as f64 - WIDTH as f64 / 2.0) == place);
        Ok(Some((called, want.as_ref().map(target))))
    }

    /// The bubbles on the screen showing a berry, left to right.
    pub fn wants(&self) -> Result<Vec<Want>> {
        let shown = pictures(&self.layouts, &self.image_of, false, &|image: &str| {
            image == BUBBLE || image.starts_with(BERRY)
        })?;
        let bubbles: Vec<(i64, f64, f64)> = shown
            .iter()
            .filter(|(_, _, image, _, _)| image == BUBBLE)
            .map(|&(_, root, _, across, up)| (root, across, up))
            .collect();
        let standing = self.standing()?;
        let mut found: Vec<Want> = Vec::new();
        for (pane, root, image, across, up) in py::sorted_by_lt(shown, |a, b| a.3 < b.3) {
            if !image.starts_with(BERRY)
                || across.abs() >= WIDTH as f64 / 2.0
                || up.abs() >= HEIGHT as f64 / 2.0
                || !bubbles
                    .iter()
                    .any(|&(at, x, y)| at == root && inside(across - x, up - y))
            {
                continue;
            }
            let berry = match self.words.get(&image) {
                Some(word) if !word.is_empty() => word.clone(),
                _ => image.strip_suffix(".bclim").unwrap_or(&image).to_string(),
            };
            let who = standing.get(&place(across));
            let said = match who {
                None => own(WANTED, &[("berry", &berry)]),
                Some(who) if self.berry_first => own(FOR, &[("berry", &berry), ("pokemon", who)]),
                Some(who) => own(WANTS, &[("pokemon", who), ("berry", &berry)]),
            };
            found.push(Want {
                pane,
                x: py::round(WIDTH as f64 / 2.0 + across)?,
                said,
            });
        }
        Ok(found)
    }

    /// Watch the round's clock run down for how fast the game runs, at most every
    /// `PACE_SECONDS`; a clock that stands, as on the results, says nothing of it.
    fn pace(&mut self) -> Result<()> {
        let frames = self.frames()?;
        let now = (self.clock)();
        let Some(frames) = frames else {
            return Ok(());
        };
        let Some((then, before)) = self.paced.filter(|&(_, before)| frames <= before) else {
            self.paced = Some((now, frames));
            return Ok(());
        };
        if now - then >= PACE_SECONDS {
            if before > frames {
                // Python's `min(max(x, 0.05), 4.0)`: the first unless the second is strictly
                // greater, or less.
                let raw = (before - frames) as f64 / (now - then) / CLOCK_RATE as f64;
                let floor = if 0.05 > raw { 0.05 } else { raw };
                self.speed = if 4.0 < floor { 4.0 } else { floor };
            }
            self.paced = Some((now, frames));
        }
        Ok(())
    }

    /// The round's clock, in frames left, or None with no app object found.
    fn frames(&self) -> Result<Option<i64>> {
        // A copy in the module's code, as `Model`'s are, holds instructions where the clock is:
        // every one a word far beyond any round's frames.
        for (&at, &vtable) in &self.apps {
            let head = self.memory.read(at, CLOCK + 4)?;
            if u32(&head, 0)? == vtable {
                let value = u32(&head, CLOCK)?;
                if value <= LONGEST {
                    return Ok(Some(value));
                }
            }
        }
        Ok(None)
    }

    /// Whether the game runs and no demonstration is showing; with none found, it runs.
    fn playing(&self, running: Option<&str>) -> Result<bool> {
        if running != Some(APP) {
            return Ok(false);
        }
        for (&at, &vtable) in &self.demos {
            let head = self.memory.read(at, OVER + 4)?;
            let text = u32(&head, TEXT)?;
            if u32(&head, 0)? == vtable
                && HEAP_SPAN.0 <= text
                && text < HEAP_SPAN.1
                && self.names.of_object(text)?.as_deref() == Some(strbuf::CLASS)
            {
                return Ok(u32(&head, OVER)? != 0);
            }
        }
        Ok(true)
    }

    /// Each place -> what the Pokémon highest in it is called.
    fn standing(&self) -> Result<IndexMap<i64, String>> {
        let mut heights: IndexMap<i64, (f64, String)> = IndexMap::new();
        for (called, place, y) in self.records()? {
            if let Some(called) = called
                && heights.get(&place).is_none_or(|(high, _)| *high < y)
            {
                heights.insert(place, (y, called));
            }
        }
        Ok(heights
            .into_iter()
            .map(|(place, (_, called))| (place, called))
            .collect())
    }

    /// The round's Pokémon in the game's own order: what each is called, or None where it cannot
    /// be read, its place and how high it stands.
    fn records(&self) -> Result<Vec<(Option<String>, i64, f64)>> {
        let mut found: Vec<(Option<String>, i64, f64)> = Vec::new();
        for (&at, &vtable) in &self.models {
            let raw = self.memory.read(at, RECORDS + COUNT * SIZE)?;
            if u32(&raw, 0)? != vtable {
                continue;
            }
            for i in 0..COUNT {
                let record = RECORDS + i * SIZE;
                let param = u32(&raw, record + PARAM)?;
                let (x, y) = (f32(&raw, record + PLACE)?, f32(&raw, record + PLACE + 4)?);
                // The sweep finds two more in the module's code, which hold instructions where
                // a record's pointers are: read through, 0xebffe000 failed every poll of the round
                // and nothing more was said.
                if !(HEAP_SPAN.0 <= param && param < HEAP_SPAN.1) {
                    continue;
                }
                let read = pokemon::read(&*self.memory, &self.names, param, pokemon::SPECIES_MAX)?;
                found.push((
                    read.as_ref().and_then(|read| (self.name)(read)),
                    place(x),
                    y,
                ));
            }
        }
        Ok(found)
    }
}

/// Touch mode as Berry Picker's keys ask it: the targets it offers, and a drag given
/// (`TouchTargets`, or a test's stand-in, as Python duck-types it).
pub trait Touching {
    fn targets(&mut self) -> Result<Vec<Target>>;
    fn give(&mut self, picked: &Target, over: &Target) -> String;
}

impl Touching for TouchTargets {
    fn targets(&mut self) -> Result<Vec<Target>> {
        TouchTargets::targets(self)
    }

    fn give(&mut self, picked: &Target, over: &Target) -> String {
        TouchTargets::give(self, picked, over)
    }
}

/// Berry Picker's own keys (Orion, 2026-10-29): a berry's picks it up off the tree, and a
/// Pokémon's gives it the berry picked up, the drag made then as touch mode's is on letting go; a
/// Pokémon's with none picked up says what it wants. Another berry's key picks that one up instead.
/// Touch mode's own keys still drag as ever, for a player who wants the challenge.
///
/// `picked` and `wrong` are what is heard as a berry is picked up, and for a key with nothing to
/// do, a berry the round has not or a Pokémon not up: a sound, and "", or else the words.
///
/// Python's holds touch mode, which is built after the PlayNav's screens, as it offers their
/// targets; here touch mode, which the navigator owns, is handed to `press`.
pub struct BerryKeys {
    picker: Rc<RefCell<BerryPicker>>,
    /// An image -> the game's word for it: a berry's item name, the tree's targets' names.
    words: IndexMap<String, String>,
    picked: Rc<dyn Fn(&str) -> String>,
    wrong: Rc<dyn Fn(&str) -> String>,
    /// The berry picked up, by name, while one is.
    berry: Option<String>,
}

impl BerryKeys {
    pub fn new(
        picker: Rc<RefCell<BerryPicker>>,
        words: IndexMap<String, String>,
        picked: Rc<dyn Fn(&str) -> String>,
        wrong: Rc<dyn Fn(&str) -> String>,
    ) -> Self {
        BerryKeys {
            picker,
            words,
            picked,
            wrong,
            berry: None,
        }
    }

    /// The keys' table in the keys file while Berry Picker runs, `[pa3ds.amie.berry_picker]`,
    /// else None.
    pub fn context(&self, running: Option<&str>) -> Option<&'static str> {
        (running == Some(APP)).then_some(BERRY_PICKER)
    }

    /// Answer one of the keys, with the line to say.
    pub fn press(
        &mut self,
        action: &str,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<String> {
        if let Some(number) = BERRY_KEYS.iter().position(|&berry| berry == action) {
            let name = BERRIES
                .get(number)
                .and_then(|(image, _)| self.words.get(*image))
                .cloned();
            let Some(name) = name else {
                return Ok((self.wrong)(&own(NO_BERRY, &[("number", &(number + 1))])));
            };
            if tree(touch, &name)?.is_none() {
                return Ok((self.wrong)(&own(NO_BERRY, &[("number", &(number + 1))])));
            }
            self.berry = Some(name.clone());
            return Ok((self.picked)(&name));
        }
        let index = POKEMON_KEYS
            .iter()
            .position(|&pokemon| pokemon == action)
            .ok_or_else(|| Error::value("tuple.index(x): x not in tuple"))?;
        let found = self.picker.borrow().pokemon(index, running)?;
        let Some((called, target)) = found else {
            return Ok((self.wrong)(&own(NO_POKEMON, &[("number", &(index + 1))])));
        };
        let Some(berry) = self.berry.clone() else {
            return Ok(match target {
                Some(target) => target.name,
                None => own(NOT_UP, &[("pokemon", &called)]),
            });
        };
        let berry = tree(touch, &berry)?;
        let (Some(target), Some(berry)) = (target, berry) else {
            return Ok((self.wrong)(&own(NOT_UP, &[("pokemon", &called)])));
        };
        self.berry = None;
        self.picker.borrow_mut().given();
        Ok(touch.give(&berry, &target))
    }
}

/// The berry by that name hanging in the tree, as touch mode offers it.
fn tree(touch: &mut dyn Touching, name: &str) -> Result<Option<Target>> {
    Ok(touch
        .targets()?
        .into_iter()
        .find(|target| target.name == name))
}

/// Where a Pokémon wanting a berry is given one, as touch mode offers it.
fn target(want: &Want) -> Target {
    Target::new(
        want.pane,
        &want.said,
        want.x,
        BODY,
        HEIGHT as f64 / 2.0 - BODY as f64,
        APART as f64,
    )
}

/// Whether a berry so far across and up from a bubble's place is drawn in it.
fn inside(across: f64, up: f64) -> bool {
    across.abs() <= NEAR && (up - IN_BUBBLE).abs() <= NEAR
}

/// Left 0, the middle 1, right 2.
fn place(x: f64) -> i64 {
    i64::from(x > -APART as f64) + i64::from(x > APART as f64)
}
