//! Pokémon-Amie's Head It: which Pokémon each ball of yarn falls at, and where each is tapped
//! (spike A).
//!
//! Balls of yarn fall from the top screen at the player's Pokémon, who stand along the bottom of the
//! bottom screen, left, in the middle and right; a tap on the Pokémon a ball falls at heads it back
//! ("Tap Pokémon to knock back the balls of yarn!"), and a ball that reaches the bottom is a "MISS!".
//! On Easy the Pokémon Amie plays with stands alone in the middle at first, and two more of the party
//! come up beside it as the round goes on (as_amie_head_it: Ralts, then Poochyena on the left and
//! rowan on the right). The game names neither the Pokémon nor which one a ball falls at, and the
//! touch areas over the Pokémon hang from their layout's root with no button: touch mode offered
//! nothing (spike 254).
//!
//! Each ball is a picture of the bottom screen's balls layout, `ball00_00.bclim` and on, its
//! texture turning as it spins, falling from 480 above the screen's middle straight down its column,
//! x -100, 0 or 100, past the screen's bottom in about a second and a half; the spare balls are
//! parked far off to the right. The Pokémon are 3D models, held by the game's `app::heading::Model`,
//! three records, one a Pokémon:
//!
//! ```text
//! Model   +0x0C  a record a Pokémon, 0x34 bytes each
//! record  +0x04  -> its `PokemonParam`, a copy of the party's
//!         +0x14  x: -120 on the left, 0 in the middle, 120 on the right
//!         +0x18  y: 25 while up, -200 before it comes up, rising between
//! ```
//!
//! So a ball is said as it comes by the Pokémon standing in its column, and each Pokémon up is
//! offered in touch mode by its name, a tap on its body: the touch areas over the Pokémon, `1f` to
//! `1h`, 106 by 120, stand at x -107, 0 and 107 from the screen's middle and reach from its middle
//! down to its bottom. The Pokémon stand there through the opening, which demonstrates a ball headed
//! back, and behind the results, so balls are said and the Pokémon offered only while the app's
//! object, `app::heading::Process`, says balls fall (`PHASE`). A tap heads a ball back once it is a
//! little over halfway down, about 1.1 to 1.2 seconds after pa3ds first sees it at the top, for
//! about a fifth of a second (spike A, live).
//!
//! Head It's own keys tap the Pokémon left to right, the game's own order of its records, and the
//! round's three are said in that order in its opening, as Berry Picker's are (`_lineup`): all
//! three are in the records from the opening, those not yet up below the screen (spike A's tape).
//!
//! Bound by:
//! - ADR-0009: a Pokémon is tapped on the player's key, by touch mode, as the target it has just
//!   named.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::discovery::HEAP_SPAN;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::core::words::own;
use crate::games::common::field::keys::{HEAD_IT, HEADERS};
use crate::games::common::layout::Layouts;
use crate::games::common::pokemon;
use crate::games::common::touch::{HEIGHT, ImageOf, Target, WIDTH};
use crate::games::gen6::subscreen::berry_picker::{Name, Touching};
use crate::games::gen6::subscreen::super_training::pictures;
use crate::py::{self, Error, Kind, PySet, Result};

pub const APP: &str = "N3app7heading7ProcessE";
/// Where the app's object says what the round is doing (spike A, live): 0 and 1 its opening, 2
/// the balls falling (`FALLING`), 3 time up, its "!", 4 one ball more, at the middle Pokémon, as
/// "Big Yarn Bonus" comes (`LAST`), 5 the results and 6 Play Again and Quit (`LIST`). Its clock, at
/// +0x4C, counts 1800 down from START! to -10, and stands there from time up.
pub const PHASE: i64 = 0x30;
pub const FALLING: i64 = 2;
pub const LAST: i64 = 4;
pub const LIST: i64 = 6;
pub const MODEL: &str = "N3app7heading5ModelE";
pub const RECORDS: i64 = 0x0C;
pub const SIZE: i64 = 0x34;
pub const COUNT: i64 = 3;
pub const PARAM: i64 = 0x04;
/// x, then y.
pub const PLACE: i64 = 0x14;
/// How far either side of the middle a Pokémon or a ball is in the middle column: the Pokémon
/// stand at -120, 0 and 120, the balls fall at -100, 0 and 100.
pub const APART: i64 = 50;
/// The balls' pictures: `ball00_00.bclim` and on through the round, and after time up one more,
/// the giant ball, `giant01.bclim` or `giant02.bclim`, down the middle (spike A, live).
pub const BALLS: [&str; 2] = ["ball", "giant"];
/// Where each column's Pokémon is tapped across the screen: the middles of the touch areas over
/// them, `1f` to `1h`.
pub const COLUMNS: [i64; 3] = [53, 160, 267];
/// How far down the screen a Pokémon is tapped, within the touch area over it and the lowest of
/// the three its column's ball passes through: taps there headed balls in all three columns (spike
/// A, live).
pub const BODY: i64 = 200;
/// How tall the touch areas over the Pokémon are.
pub const TALL: f64 = 120.0;
/// How far above the screen's middle a ball is as a tap sent then heads it at the sweet spot, for
/// 3 points rather than 1: tapped by a timer as the ball reached it, 0.2 s after a cue that much
/// ahead, all 6 were, and 0.05 s either side none of 12, though all were headed (spike A, live).
pub const SWEET: f64 = 35.0;
/// Seconds before the ball reaches `SWEET` the cue sounds, as it ships (`[pa3ds.amie]
/// head_it_lead`): about how long a player takes from the cue to the tap, the sound's way out to
/// their ears, hearing it, and the key's way through pa3ds to the game. It was 0.2, the delay after
/// which the spike's own taps, sent by a timer, all hit the sweet spot; Orion's, by ear, hit nothing
/// at it (2026-11-01).
pub const LEAD: f64 = 0.4;
/// Seconds between the heights a ball's fall is measured by, three of them. Read from one tick to
/// the next, a frame apart or none, its speed jumped; and on a tape, whose heights change five times
/// a second, a ball 43 units lower a tick later was cued 374 above `SWEET` (as_amie_head_it).
pub const SPAN: f64 = 0.1;
pub const COMING: &str = "head-it-ball";
pub const NOBODY: &str = "head-it-ball-nobody";
pub const NOT_UP: &str = "head-it-no-pokemon";
pub const LINEUP: &str = "head-it-lineup";
pub const ESTIMATE: &str = "head-it-estimate";
/// Seconds after a cue within which the key of its ball's Pokémon answers it: the ball is past
/// the bottom about 0.9 s after it (Orion's logged round at a 0.4 s lead).
pub const ANSWER: f64 = 1.0;
/// Answers a round needs before pa3ds says its estimate of the lead.
pub const TAPS: usize = 3;
/// What the log calls what Head It says.
pub const LOG: &str = "head it";

/// `Callable[[int], None]`: what sounds the cue for a ball in a column.
pub type Cue = Rc<dyn Fn(i64)>;

/// Says which Pokémon each of Head It's balls of yarn falls at, and offers each Pokémon up to
/// be tapped.
pub struct HeadIt {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    layouts: Rc<Layouts>,
    image_of: ImageOf,
    /// What a Pokémon of the player's is called.
    name: Name,
    /// Model object -> the vtable it was found with.
    models: IndexMap<i64, i64>,
    /// The app's own object -> the vtable it was found with: what the round is doing.
    apps: IndexMap<i64, i64>,
    /// The balls on their way down at the last poll, by pane: each one's column, and its
    /// layout's root's height, which its own is told from.
    falling: IndexMap<i64, (i64, f64)>,
    /// Sounds the cue for a ball in a column, the game's "!" (`profile.CUE_SOUND`), or None
    /// where the cue is off (`[pa3ds.amie] head_it_cue`) or the game's sounds are.
    pub(crate) cue: Option<Cue>,
    lead: f64,
    clock: Rc<dyn Fn() -> f64>,
    /// Each ball falling -> the heights it has fallen through, each with when it was first
    /// read there, from the newest at least twice `SPAN` old.
    heights: IndexMap<i64, Vec<(f64, f64)>>,
    /// The balls falling whose cue has sounded.
    ///
    /// A `PySet`: the balls gone by with none sounded are what is left of a set without them,
    /// which Python iterates as it lays it out (`poll`).
    cued: PySet,
    /// Whether the round about to start has had its three Pokémon said (`_lineup`).
    told: bool,
    /// Each column -> when its last cue sounded, until its key answers it (`pressed`).
    cues: IndexMap<i64, f64>,
    /// How long after its cue each answer of the round came.
    delays: Vec<f64>,
    /// Whether the round's results have had pa3ds's estimate said (`_estimate`).
    estimated: bool,
}

impl HeadIt {
    /// What the heap is swept for: the Pokémon, who stands where, and the app's own object, which
    /// says whether balls fall.
    pub const CLASSES: [&'static str; 2] = [MODEL, APP];

    /// `HeadIt(...)` with every argument; Python's `lead` defaults to `LEAD` and its `clock` to
    /// `time.monotonic`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        name: Name,
        cue: Option<Cue>,
        lead: f64,
        clock: Rc<dyn Fn() -> f64>,
    ) -> Self {
        HeadIt {
            memory,
            names,
            layouts,
            image_of,
            name,
            models: IndexMap::new(),
            apps: IndexMap::new(),
            falling: IndexMap::new(),
            cue,
            lead,
            clock,
            heights: IndexMap::new(),
            cued: PySet::new(),
            told: false,
            cues: IndexMap::new(),
            delays: Vec::new(),
            estimated: false,
        }
    }

    /// The models and apps a sweep found, by class: all of them, or with `adding` the new
    /// ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut kept: IndexMap<&str, IndexMap<i64, i64>> =
            [(MODEL, &self.models), (APP, &self.apps)]
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
        self.apps = kept.shift_remove(APP).unwrap_or_default();
        Ok(())
    }

    /// The round's three Pokémon as its opening starts (`_lineup`), and while balls fall each
    /// ball that has just come, by the Pokémon it falls at, as the screen's words; and, only
    /// logged (None), each ball gone by with the cue on and none sounded for it, with its last
    /// heights: one of Orion's logged round's twenty went so (77.49 s), and nothing said why.
    pub fn poll(
        &mut self,
        running: Option<&str>,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        let phase = if running == Some(APP) {
            self.phase()?
        } else {
            None
        };
        let mut said: Vec<(String, String, Option<Category>)> = Vec::new();
        for (logged, line, category) in self.lineup(phase)?.into_iter().chain(self.estimate(phase))
        {
            said.push((logged, line, Some(category)));
        }
        if phase.is_none_or(|phase| !(FALLING..=LAST).contains(&phase)) {
            self.falling = IndexMap::new();
            (self.heights, self.cued) = (IndexMap::new(), PySet::new());
            return Ok(said);
        }
        let falling = self.balls()?;
        let new: Vec<i64> = falling
            .iter()
            .filter(|(pane, _)| !self.falling.contains_key(*pane))
            .map(|(_, &(column, _))| column)
            .collect();
        let falling_keys: Vec<i64> = falling.keys().copied().collect();
        if self.cue.is_some() {
            // `self._falling.keys() - falling.keys() - self._cued`: a set of the dict's keys, the
            // others discarded from it, then a set of those `_cued` lacks, iterated as laid out.
            let was: Vec<i64> = self.falling.keys().copied().collect();
            let mut gone = PySet::of_dict(&was);
            gone.difference_update(&PySet::of_dict(&falling_keys));
            for pane in gone.difference(&self.cued).iter() {
                let seen = self.heights.get(&pane).map(Vec::as_slice).unwrap_or(&[]);
                let last = &seen[seen.len().saturating_sub(4)..];
                let heights: Vec<String> = last
                    .iter()
                    .map(|&(when, height)| {
                        format!("{} at {}", py::fixed(height, 0), py::fixed(when, 2))
                    })
                    .collect();
                said.push((
                    LOG.to_string(),
                    format!(
                        "no cue for a ball gone by, its last heights {}",
                        heights.join(", ")
                    ),
                    None,
                ));
            }
        }
        self.falling = falling;
        // Here, not in `tick`, where it was: with no ball falling nothing ticks.
        let falling = &self.falling;
        self.heights.retain(|pane, _| falling.contains_key(pane));
        self.cued
            .intersection_update(&PySet::of_dict(&falling_keys));
        if new.is_empty() {
            return Ok(said);
        }
        let standing = self.standing()?;
        said.extend(new.into_iter().map(|column| {
            let line = match standing.get(&column) {
                Some(called) => own(COMING, &[("pokemon", called)]),
                None => own(NOBODY, &[]),
            };
            (LOG.to_string(), line, Some(Category::Screen))
        }));
        Ok(said)
    }

    /// The round's three Pokémon left to right, its keys' order, said once in its opening and
    /// again in each Play Again's, as Berry Picker's are (Orion, 2026-11-01: "the same precedent
    /// as berry picker"): on Easy two come up only as the round goes on, and in Orion's first
    /// logged round he pressed the keys in the order the Pokémon came up, all but Windsong's
    /// wrong. Left to right is the game's own order of its records too (spike A's tape).
    fn lineup(&mut self, phase: Option<i64>) -> Result<Vec<(String, String, Category)>> {
        if phase.is_none_or(|phase| phase >= FALLING) {
            self.told = false;
            return Ok(Vec::new());
        }
        let mut records: Vec<(i64, (i64, String, bool))> = self.records()?.into_iter().collect();
        records.sort_by_key(|&(column, _)| column);
        let names: Vec<String> = records
            .into_iter()
            .map(|(_, (_, called, _))| called)
            .collect();
        if self.told || names.len() as i64 != COUNT {
            return Ok(Vec::new());
        }
        self.told = true;
        Ok(vec![(
            LOG.to_string(),
            own(
                LINEUP,
                &[
                    ("first", &names[0]),
                    ("second", &names[1]),
                    ("third", &names[2]),
                ],
            ),
            Category::Screen,
        )])
    }

    /// pa3ds's estimate of the player's lead, said once at the round's results: the middle
    /// of how long after each cue the key of its ball's Pokémon came, for the player to put in
    /// `[pa3ds.amie] head_it_lead` or nudge from. Never used in the round itself (Orion,
    /// 2026-11-01: a lead changed mid-round throws off a player's own sense of it), and said as
    /// pa3ds's, not the game's.
    fn estimate(&mut self, phase: Option<i64>) -> Vec<(String, String, Category)> {
        let Some(phase) = phase.filter(|&phase| phase >= FALLING) else {
            (self.cues, self.delays, self.estimated) = (IndexMap::new(), Vec::new(), false);
            return Vec::new();
        };
        if phase <= LAST || self.estimated || self.delays.len() < TAPS {
            return Vec::new();
        }
        self.estimated = true;
        vec![(
            LOG.to_string(),
            own(
                ESTIMATE,
                &[
                    ("seconds", &py::fixed(median(&self.delays), 2)),
                    ("taps", &self.delays.len()),
                ],
            ),
            Category::Screen,
        )]
    }

    /// A Pokémon's own key has tapped it: an answer to its last cue, timed, where one
    /// sounded within `ANSWER`.
    pub fn pressed(&mut self, column: i64) {
        let when = self.cues.shift_remove(&column);
        if let Some(when) = when {
            let delay = (self.clock)() - when;
            if delay <= ANSWER {
                self.delays.push(delay);
            }
        }
    }

    /// Each Pokémon up, by its name, at its body, while balls fall: they stand there through
    /// the opening and behind the results and the list after, Play Again and Quit.
    pub fn targets(&self, running: Option<&str>) -> Result<Vec<Target>> {
        if running != Some(APP) || !self.playing()? {
            return Ok(Vec::new());
        }
        let mut up: Vec<(i64, (i64, String))> = self.up()?.into_iter().collect();
        up.sort();
        Ok(up
            .into_iter()
            .map(|(column, (param, called))| {
                Target::new(
                    param,
                    &called,
                    COLUMNS[column as usize],
                    BODY,
                    HEIGHT as f64 / 2.0 - BODY as f64,
                    TALL,
                )
            })
            .collect())
    }

    /// Whether balls fall, from START! through the one after time up. The opening
    /// demonstrates a ball falling at the Pokémon and headed back, the game's own, said nothing
    /// of as Berry Picker's opening is (spike A, live). With no app's object found, none do: the
    /// sweep finds it with the Pokémon, and as Quit closed the round it was gone while they were
    /// not, and they were offered for 2 s (as_amie_head_it's tape).
    fn playing(&self) -> Result<bool> {
        let phase = self.phase()?;
        Ok(phase.is_some_and(|phase| (FALLING..=LAST).contains(&phase)))
    }

    /// What the round is doing (`PHASE`), or None with no app's object found.
    fn phase(&self) -> Result<Option<i64>> {
        for (&at, &vtable) in &self.apps {
            let head = self.memory.read(at, PHASE + 4)?;
            // A copy in the module's code, as `Model`'s are, holds an instruction there: a word
            // far past the last phase.
            if u32(&head, 0)? == vtable {
                let phase = u32(&head, PHASE)?;
                if phase <= LIST {
                    return Ok(Some(phase));
                }
            }
        }
        Ok(None)
    }

    /// Whether `tick` has balls to follow: while the cue is on and balls fall.
    pub fn ticking(&self, running: Option<&str>) -> bool {
        self.cue.is_some() && running == Some(APP) && !self.falling.is_empty()
    }

    /// Follow each ball falling, every tick, and sound the cue as it is due: the lead before
    /// the ball reaches `SWEET`, by how fast it falls and how much faster it has grown over the
    /// last two `SPAN`s. Balls fall at different speeds (Bulbapedia: slow, fast and quickening
    /// with a combo), and each faster and faster: on as_amie_head_it's tape one went from 120
    /// units a second to 520 over 2 s, another reached 680 at `SWEET`, so a ball taken to fall
    /// on as fast as it falls now was cued late, the more so the longer the lead. The screen's
    /// poll, a fifth of a second, finds the balls; a ball's height alone is read every tick.
    pub fn tick(&mut self) -> Result<()> {
        let Some(cue) = self.cue.clone() else {
            return Ok(());
        };
        let now = (self.clock)();
        let y = self.layouts.shape.y;
        for (&pane, &(column, root)) in &self.falling {
            // Not through `Layouts`, which keeps what it read until the screen's next poll.
            let height = f32(&self.memory.read(pane + y, 4)?, 0)? - root;
            let seen = self.heights.entry(pane).or_default();
            // Not moved since, the same frame read again; or headed back up, a fall no longer.
            if !seen.is_empty() && height == seen[seen.len() - 1].1 {
                continue;
            }
            if !seen.is_empty() && height > seen[seen.len() - 1].1 {
                seen.clear();
            }
            seen.push((now, height));
            while seen.len() > 1 && now - seen[1].0 >= 2.0 * SPAN {
                seen.remove(0);
            }
            if self.cued.contains(pane) || height < SWEET || now - seen[0].0 < 2.0 * SPAN {
                continue;
            }
            if due(seen, SWEET)? <= self.lead {
                self.cued.add(pane);
                self.cues.insert(column, now);
                cue(column);
            }
        }
        Ok(())
    }

    /// The Pokémon up in a column while balls fall, as touch mode offers it, or None.
    pub fn pokemon(&self, column: usize, running: Option<&str>) -> Result<Option<Target>> {
        Ok(self
            .targets(running)?
            .into_iter()
            .find(|target| target.x == COLUMNS[column]))
    }

    /// The balls on the screen or above it on their way down, by pane, each with its column
    /// and its layout's root's height.
    fn balls(&self) -> Result<IndexMap<i64, (i64, f64)>> {
        let shown = pictures(&self.layouts, &self.image_of, false, &|image: &str| {
            BALLS.iter().any(|ball| image.starts_with(ball))
        })?;
        let mut found = IndexMap::new();
        for (pane, _, _, across, up) in shown {
            if across.abs() < WIDTH as f64 / 2.0 && up > -(HEIGHT as f64) / 2.0 {
                let root = f32(&self.layouts.pane(pane)?, self.layouts.shape.y)? - up;
                found.insert(pane, (column(across), root));
            }
        }
        Ok(found)
    }

    /// Each column -> what the Pokémon up in it is called.
    fn standing(&self) -> Result<IndexMap<i64, String>> {
        Ok(self
            .up()?
            .into_iter()
            .map(|(column, (_, called))| (column, called))
            .collect())
    }

    /// Each column -> its Pokémon up, by its `PokemonParam` and what it is called.
    fn up(&self) -> Result<IndexMap<i64, (i64, String)>> {
        Ok(self
            .records()?
            .into_iter()
            .filter(|(_, (_, _, up))| *up)
            .map(|(column, (param, called, _))| (column, (param, called)))
            .collect())
    }

    /// Each column -> its Pokémon, by its `PokemonParam`, what it is called and whether it is
    /// up: all three are there from the round's opening, those not yet up below the screen.
    fn records(&self) -> Result<IndexMap<i64, (i64, String, bool)>> {
        let mut found: IndexMap<i64, (i64, String, bool)> = IndexMap::new();
        for (&at, &vtable) in &self.models {
            let raw = self.memory.read(at, RECORDS + COUNT * SIZE)?;
            if u32(&raw, 0)? != vtable {
                continue;
            }
            for i in 0..COUNT {
                let record = RECORDS + i * SIZE;
                let param = u32(&raw, record + PARAM)?;
                let (x, y) = (f32(&raw, record + PLACE)?, f32(&raw, record + PLACE + 4)?);
                // The sweep finds three more in the module's code, which holds instructions where
                // a record's pointers are, as Berry Picker's `Model`s do (spike 260).
                if !(HEAP_SPAN.0 <= param && param < HEAP_SPAN.1) {
                    continue;
                }
                let read = pokemon::read(&*self.memory, &self.names, param, pokemon::SPECIES_MAX)?;
                if let Some(called) = read.as_ref().and_then(|read| (self.name)(read))
                    && !called.is_empty()
                {
                    found.insert(column(x), (param, called, y > 0.0));
                }
            }
        }
        Ok(found)
    }
}

/// Left 0, the middle 1, right 2.
fn column(x: f64) -> i64 {
    i64::from(x > -APART as f64) + i64::from(x > APART as f64)
}

/// Seconds until a ball reaches the height `at`, from the (when, height) it fell through,
/// oldest first: falling on as it has, and growing faster as steadily as it has. Its speed over
/// the older half and over the newer is its speed in the middle of each, exactly so for a ball
/// sped up steadily, and how much it grew between the two is how fast it speeds up. Never, for
/// a ball no longer falling.
pub fn due(seen: &[(f64, f64)], at: f64) -> Result<f64> {
    if seen.len() < 3 {
        return Ok(f64::INFINITY);
    }
    let ((first, top), (last, now)) = (seen[0], seen[seen.len() - 1]);
    // Python's `min(..., key=...)`: the first of equal nearest.
    let mut nearest = seen[1];
    for &each in &seen[2..seen.len() - 1] {
        if (each.0 - (first + last) / 2.0).abs() < (nearest.0 - (first + last) / 2.0).abs() {
            nearest = each;
        }
    }
    let (middle, height) = nearest;
    let older = divide(top - height, middle - first)?;
    let newer = divide(height - now, last - middle)?;
    // Python's `max(x, 0.0)`: the first unless the second is strictly greater.
    let grown = divide(newer - older, (last - first) / 2.0)?;
    let faster = if 0.0 > grown { 0.0 } else { grown };
    let speed = newer + faster * (last - middle) / 2.0;
    let left = now - at;
    if speed <= 0.0 {
        return Ok(f64::INFINITY);
    }
    if faster == 0.0 {
        return Ok(left / speed);
    }
    Ok(((speed * speed + 2.0 * faster * left).sqrt() - speed) / faster)
}

/// `statistics.median`: the middle of the values sorted, or the mean of the two middle ones.
fn median(values: &[f64]) -> f64 {
    let data = py::sorted_by_lt(values.to_vec(), |a: &f64, b: &f64| a < b);
    let n = data.len();
    if n % 2 == 1 {
        return data[n / 2];
    }
    (data[n / 2 - 1] + data[n / 2]) / 2.0
}

/// `a / b` on floats, raising `ZeroDivisionError` as Python does where `b` is 0: two heights read
/// at the same time.
#[track_caller]
fn divide(a: f64, b: f64) -> Result<f64> {
    if b == 0.0 {
        return Err(Error::new(
            Kind::ZeroDivisionError,
            "float division by zero",
        ));
    }
    Ok(a / b)
}

/// Head It's own keys (Orion, 2026-10-31): a key a Pokémon, by where it stands, left, in the
/// middle and right, which taps it, as touch mode's tap does the Pokémon it has just said. Touch
/// mode's own keys still tap there as ever.
///
/// `wrong` is what is heard for a key with nothing to do, a Pokémon not up or no ball falling: a
/// sound, and "", or else the words.
///
/// Python's holds touch mode, which is built after the subscreens, as it offers their targets;
/// here touch mode, which the navigator owns, is handed to `press`, as `BerryKeys`' is.
pub struct HeadItKeys {
    game: Rc<RefCell<HeadIt>>,
    wrong: Rc<dyn Fn(&str) -> String>,
}

impl HeadItKeys {
    pub fn new(game: Rc<RefCell<HeadIt>>, wrong: Rc<dyn Fn(&str) -> String>) -> Self {
        HeadItKeys { game, wrong }
    }

    /// The keys' table in the keys file while Head It runs, `[pa3ds.amie.head_it]`, else
    /// None.
    pub fn context(&self, running: Option<&str>) -> Option<&'static str> {
        (running == Some(APP)).then_some(HEAD_IT)
    }

    /// Tap the Pokémon of the key, with the line to say.
    pub fn press(
        &self,
        action: &str,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<String> {
        let column = HEADERS
            .iter()
            .position(|&header| header == action)
            .ok_or_else(|| Error::value("tuple.index(x): x not in tuple"))?;
        let target = self.game.borrow().pokemon(column, running)?;
        let Some(target) = target else {
            return Ok((self.wrong)(&own(NOT_UP, &[("number", &(column + 1))])));
        };
        self.game.borrow_mut().pressed(column as i64);
        Ok(touch.tap(&target))
    }
}
