//! Pokémon-Amie's Head It: which Pokémon each ball of yarn falls at, and where each is tapped
//! (spike 273).
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
//! about a fifth of a second (spike 273, live).
//!
//! Head It's own keys tap the Pokémon left to right, the game's own order of its records, and the
//! round's three are said in that order in its opening, as Berry Picker's are (`_lineup`): all
//! three are in the records from the opening, those not yet up below the screen (spike 273's tape).
//!
//! Bound by:
//! - ADR-0009: a Pokémon is tapped on the player's key, by touch mode, as the target it has just
//!   named.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::clips;
use crate::core::discovery::HEAP_SPAN;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::core::words::own;
use crate::games::common::field::keys::{HEAD_IT, HEADERS};
use crate::games::common::layout::Layouts;
use crate::games::common::pokemon;
use crate::games::common::touch::{HEIGHT, ImageOf, Target, WIDTH};
use crate::games::gen6::rom::cries::Crier;
use crate::games::gen6::subscreen::berry_picker::{Name, Process, Touching};
use crate::games::gen6::subscreen::super_training::pictures;
use crate::py::{self, Error, PySet, Result};

pub const APP: &str = "N3app7heading7ProcessE";
/// Where the app's object says what the round is doing (spike 273, live): 0 and 1 its opening, 2
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
pub const GIANT: &str = "giant";
/// The balls' pictures: `ball00_00.bclim` and on through the round, and after time up one more,
/// the giant ball, `giant01.bclim` or `giant02.bclim`, down the middle (spike 273, live).
pub const BALLS: [&str; 2] = ["ball", GIANT];
/// Where each column's Pokémon is tapped across the screen: the middles of the touch areas over
/// them, `1f` to `1h`.
pub const COLUMNS: [i64; 3] = [53, 160, 267];
/// How far down the screen a Pokémon is tapped, within the touch area over it and the lowest of
/// the three its column's ball passes through: taps there headed balls in all three columns (spike
/// 273, live).
pub const BODY: i64 = 200;
/// How tall the touch areas over the Pokémon are.
pub const TALL: f64 = 120.0;
/// How far above the screen's middle a ball is as a tap sent then heads it at the sweet spot, for
/// 3 points rather than 1: tapped by a timer as the ball reached it, 0.2 s after a cue that much
/// ahead, all 6 were, and 0.05 s either side none of 12, though all were headed (spike 273, live).
/// Every ball's but the giant's (`GIANT_SWEET`).
pub const SWEET: f64 = 35.0;
/// `SWEET` for the giant ball after time up, 50 points rather than 10, higher as the ball is
/// bigger: or_amie_head_it_hard cut before it and played on, it fell through the same heights each
/// run, a frame apart, and a tap sent as it was read at 125 or 104 was a sweet spot every time, five
/// runs; at 82 and 60, 10; at 145, and at 37 and below, a miss (spike 273, live). A tap counts for
/// the frame it comes in, so the sweet spot runs from the frame at 125 to the one at 82, and 104
/// starts its middle: a frame to spare either side. Aimed at 115, half a frame early, a tap 0.4 s
/// after the cue was read at 145 once in two. Cued for `SWEET`, its "!" came late, and a tap the
/// lead after it missed (Orion, or_amie_head_it_normal).
pub const GIANT_SWEET: f64 = 104.0;
/// Seconds before the ball reaches `SWEET` the cue sounds, as it ships (`[pa3ds.amie]
/// head_it_lead`): about how long a player takes from the cue to the tap, the sound's way out to
/// their ears, hearing it, and the key's way through pa3ds to the game. It was 0.2, the delay after
/// which the spike's own taps, sent by a timer, all hit the sweet spot; Orion's, by ear, hit nothing
/// at it (2026-11-01).
pub const LEAD: f64 = 0.4;
/// The game's frames a second as `gen6.frames` counts them, the screen's: 59.83. Most falls fit
/// best stepping every other one, and the round's own count of frames (`Process` +0x60, 30 a second)
/// stands still while the giant ball falls (spike 277, live).
pub const FRAME_RATE: f64 = 268111856.0 / 4481136.0;
/// Seconds of ticks the frame clock is set by: when each frame began, by the tick that read its
/// count soonest after, so a stall of the emulator is forgotten a second on.
pub const WINDOW: f64 = 1.0;
/// Seconds a ball is followed before it may be cued. Timed by fewer heights, read from one tick to
/// the next a frame apart or none, its speed jumped; and on a tape, whose heights change five times
/// a second, a ball 43 units lower a tick later was cued 374 above `SWEET` (as_amie_head_it).
pub const RUN_IN: f64 = 0.2;
pub const FASTEST: f64 = 1500.0;
/// A ball read lower than `MISREAD` below where it could have fallen at `FASTEST` units a second
/// since its last height is misread, and skipped: of 1,700 reads a round live, 1 to 6 were a single
/// tick 249 to 319 below the ball, the next back in line, and timed as a fall such a read sounded the
/// cue at once, 0.66 s early (spike 273, live). The fastest fall read was 40 units a frame, 1,200 a
/// second, near the bottom.
pub const MISREAD: f64 = 150.0;
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
/// `Callable[[], int | None]`: the game's count of frames (`gen6.frames`).
pub type Frames = Rc<dyn Fn() -> Result<Option<i64>>>;

/// A ball on its way down, as the screen's poll read it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ball {
    pub column: i64,
    /// Above the screen's middle.
    pub height: f64,
    /// Its layout's root's height, which `tick` tells its own from.
    pub root: f64,
    /// Its sweet spot's height: `SWEET`, or the giant's `GIANT_SWEET`.
    pub sweet: f64,
}

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
    /// Where an app's process begins while that app runs, else None
    /// (`apps.RunningApp.process`): Head It's says what the round is doing.
    process: Process,
    /// The balls on their way down at the last poll, by pane.
    falling: IndexMap<i64, Ball>,
    /// Sounds the cue for a ball in a column, the game's "!" (`profile.CUE_SOUND`), or None
    /// where the cue is off (`[pa3ds.amie] head_it_cue`) or the game's sounds are.
    pub(crate) cue: Option<Cue>,
    lead: f64,
    clock: Rc<dyn Fn() -> f64>,
    /// The game's count of frames (`gen6.frames`), which times a ball's fall, or None where it
    /// cannot be read, and the clock does.
    frames: Frames,
    sleep: Rc<dyn Fn(f64)>,
    /// The last second's ticks: when each read, and the game's count of frames then.
    ticks: Vec<(f64, Option<i64>)>,
    /// Each ball falling -> for each of its heights, the game's count of frames as it was
    /// read, and as the tick before read it, which saw the height before (`_stepped`).
    reads: IndexMap<i64, Vec<(i64, Option<i64>)>>,
    /// Plays a Pokémon's cry toward its side as its ball comes, and says whether it did; None
    /// where Pokémon are said by name alone (`[pa3ds.amie] feedback_style`).
    cry: Option<Crier>,
    /// Whether the cry, where it plays, is said in the name's place.
    instead: bool,
    /// Each ball falling -> the heights it has fallen through, each with when it was first
    /// read there: its whole fall.
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
    /// Whether balls fell at the last poll, from START! to the giant ball (`holding`).
    round: bool,
    /// The balls kept falling though the last poll missed them (`poll`).
    missed: PySet,
}

impl HeadIt {
    /// What the heap is swept for: the Pokémon, who stands where. The app's own object, which
    /// says whether balls fall, is the process the game runs, known as the app starts
    /// (`process`).
    pub const CLASSES: [&'static str; 1] = [MODEL];

    /// `HeadIt(...)` with every argument; Python's `lead` defaults to `LEAD`, its `clock` to
    /// `time.monotonic`, its `cry` to None, its `instead` to False, its `frames` to none, its
    /// `sleep` to `time.sleep` and its `process` to none.
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
        cry: Option<Crier>,
        instead: bool,
        frames: Frames,
        sleep: Rc<dyn Fn(f64)>,
        process: Process,
    ) -> Self {
        HeadIt {
            memory,
            names,
            layouts,
            image_of,
            name,
            models: IndexMap::new(),
            process,
            falling: IndexMap::new(),
            cue,
            lead,
            clock,
            frames,
            sleep,
            ticks: Vec::new(),
            reads: IndexMap::new(),
            cry,
            instead,
            heights: IndexMap::new(),
            cued: PySet::new(),
            told: false,
            cues: IndexMap::new(),
            delays: Vec::new(),
            estimated: false,
            round: false,
            missed: PySet::new(),
        }
    }

    /// The models a sweep found: all of them, or with `adding` the new ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut kept = if adding {
            self.models.clone()
        } else {
            IndexMap::new()
        };
        for (&at, name) in found {
            if name == MODEL {
                kept.insert(at, u32(&self.memory.read(at, 4)?, 0)?);
            }
        }
        self.models = kept;
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
        self.round = phase.is_some_and(|phase| (FALLING..=LAST).contains(&phase));
        if !self.round {
            (self.falling, self.missed) = (IndexMap::new(), PySet::new());
            (self.heights, self.reads, self.cued) =
                (IndexMap::new(), IndexMap::new(), PySet::new());
            return Ok(said);
        }
        let mut falling = self.balls()?;
        // A ball missing from one poll and back at the next no higher is the same ball, not said
        // again nor timed afresh: one in two or three rounds went so mid-fall, at 271, and came
        // back at 170, said again and cued too late (spike 273, live). Gone, it is missing from
        // two; back higher, its pane holds the next ball.
        let back = std::mem::take(&mut self.missed);
        let kept: IndexMap<i64, Ball> = self
            .falling
            .iter()
            .filter(|&(pane, _)| !falling.contains_key(pane) && !back.contains(*pane))
            .map(|(&pane, &ball)| (pane, ball))
            .collect();
        self.missed = PySet::of_dict(&kept.keys().copied().collect::<Vec<i64>>());
        // `back & falling.keys()`, a set and a dict's keys: the set's `intersection` of the keys,
        // added in the dict's order, where the dict has no more of them; else a set of the
        // set's keys the dict has, added in the set's order.
        let shared = if falling.len() <= back.len() {
            PySet::of(falling.keys().copied().filter(|&pane| back.contains(pane)))
        } else {
            PySet::of(back.iter().filter(|pane| falling.contains_key(pane)))
        };
        let again = PySet::of(
            shared
                .iter()
                .filter(|pane| falling[pane].height > self.falling[pane].height),
        );
        falling.extend(kept);
        let falling_keys: Vec<i64> = falling.keys().copied().collect();
        // `self._falling.keys() - falling.keys() | again`: a set of the dict's keys, the others
        // discarded from it, and `again`'s added to a copy of it.
        let was: Vec<i64> = self.falling.keys().copied().collect();
        let mut gone = PySet::of_dict(&was);
        gone.difference_update(&PySet::of_dict(&falling_keys));
        let gone = gone.union(&again);
        let new: Vec<i64> = falling
            .iter()
            .filter(|&(pane, _)| !self.falling.contains_key(pane) || again.contains(*pane))
            .map(|(_, ball)| ball.column)
            .collect();
        if self.cue.is_some() {
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
        self.heights
            .retain(|pane, _| falling.contains_key(pane) && !gone.contains(*pane));
        let heights = &self.heights;
        self.reads.retain(|pane, _| heights.contains_key(pane));
        // `falling.keys() - gone`: a set of the dict's keys, `gone`'s discarded from it.
        let mut left = PySet::of_dict(&falling_keys);
        left.difference_update(&gone);
        self.cued.intersection_update(&left);
        if new.is_empty() {
            return Ok(said);
        }
        let standing: IndexMap<i64, (i64, String, bool, pokemon::Pokemon)> = self
            .records()?
            .into_iter()
            .filter(|(_, record)| record.2)
            .collect();
        for column in new {
            let Some((_, called, _, found)) = standing.get(&column) else {
                said.push((LOG.to_string(), own(NOBODY, &[]), Some(Category::Screen)));
                continue;
            };
            // Its cry toward its side, in its name's place where the player asks.
            if let Some(cry) = &self.cry
                && cry(found, column as f64 / 2.0)?
                && self.instead
            {
                continue;
            }
            said.push((
                LOG.to_string(),
                own(COMING, &[("pokemon", called)]),
                Some(Category::Screen),
            ));
        }
        Ok(said)
    }

    /// The round's three Pokémon left to right, its keys' order, said once in its opening and
    /// again in each Play Again's, as Berry Picker's are (Orion, 2026-11-01: "the same precedent
    /// as berry picker"): on Easy two come up only as the round goes on, and in Orion's first
    /// logged round he pressed the keys in the order the Pokémon came up, all but Windsong's
    /// wrong. Left to right is the game's own order of its records too (spike 273's tape).
    fn lineup(&mut self, phase: Option<i64>) -> Result<Vec<(String, String, Category)>> {
        if phase.is_none_or(|phase| phase >= FALLING) {
            self.told = false;
            return Ok(Vec::new());
        }
        let mut records: Vec<(i64, (i64, String, bool, pokemon::Pokemon))> =
            self.records()?.into_iter().collect();
        records.sort_by_key(|(column, _)| *column);
        let names: Vec<String> = records
            .into_iter()
            .map(|(_, (_, called, _, _))| called)
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
    /// of as Berry Picker's opening is (spike 273, live). Outside Head It none do: while the app's
    /// object was swept for, Quit closed the round with the object gone and the Pokémon not, and
    /// they were offered for 2 s (as_amie_head_it's tape).
    fn playing(&self) -> Result<bool> {
        let phase = self.phase()?;
        Ok(phase.is_some_and(|phase| (FALLING..=LAST).contains(&phase)))
    }

    /// What the round is doing (`PHASE`), read from Head It's process, or None outside it.
    fn phase(&self) -> Result<Option<i64>> {
        let Some(process) = (self.process)(APP) else {
            return Ok(None);
        };
        Ok(Some(u32(&self.memory.read(process + PHASE, 4)?, 0)?))
    }

    /// Whether the heap's whole sweep waits: while balls fall, as the last poll read, cue or
    /// no cue. Every 30 s it read nothing else for a second, 13,000 pages (or_amie_head_it_hard's
    /// tape): no ball was followed for its cue, and the keys waited for its end, as between its
    /// chunks only dialogue is polled. The round's objects are found as its opening builds
    /// them.
    pub fn holding(&self, running: Option<&str>) -> bool {
        running == Some(APP) && self.round
    }

    /// Whether `tick` has balls to follow: while the cue is on and balls fall.
    pub fn ticking(&self, running: Option<&str>) -> bool {
        self.cue.is_some() && running == Some(APP) && !self.falling.is_empty()
    }

    /// Follow each ball falling, every tick, and sound the cue as it is due: the lead before
    /// the ball reaches its sweet spot, `SWEET` or the giant's `GIANT_SWEET`, by its whole fall
    /// so far (`due`). Balls fall at different speeds (Bulbapedia: slow, fast and quickening
    /// with a combo), and each faster and faster: on as_amie_head_it's tape one went from 120
    /// units a second to 520 over 2 s, another reached 680 at `SWEET`. The screen's poll, a
    /// fifth of a second, finds the balls; a ball's height alone is read every tick.
    ///
    /// A tick reads 10 to 40 ms apart, up to a frame or two after the ball stepped, so each
    /// height is timed by the frame of the game's count it stepped on (`stepped`), a tick a
    /// frame turned in left out; a frame's moment is when the tick that read its count soonest
    /// after it did, over the last second (`WINDOW`). And a cue due before the next tick waits
    /// for its moment here, where it sounded at the read nearest it, up to half a tick off. A
    /// perfect player's taps, the lead after each cue on Hard, found the ball 0 to 50 above the
    /// middle at the next read 32 times in 33, against 23 to 28 before (spike 277, live). Without
    /// the count, each height is timed as it was read.
    pub fn tick(&mut self) -> Result<()> {
        let Some(cue) = self.cue.clone() else {
            return Ok(());
        };
        let counted = (self.frames)()?;
        let y = self.layouts.shape.y;
        // Not through `Layouts`, which keeps what it read until the screen's next poll.
        let mut heights: IndexMap<i64, f64> = IndexMap::new();
        for (&pane, ball) in &self.falling {
            heights.insert(pane, f32(&self.memory.read(pane + y, 4)?, 0)? - ball.root);
        }
        let now = (self.clock)();
        if counted.is_some() && (self.frames)()? != counted {
            return Ok(());
        }
        let before = self.ticks.last().and_then(|&(_, count)| count);
        self.ticks.retain(|&(when, _)| now - when < WINDOW);
        self.ticks.push((now, counted));
        let (origin, at) = match counted {
            None => (0.0, now),
            Some(counted) => {
                // Python's `min(...)`: the first of equal least.
                let mut origin: Option<f64> = None;
                for &(when, count) in &self.ticks {
                    if let Some(count) = count {
                        let each = when - count as f64 / FRAME_RATE;
                        if origin.is_none_or(|least| each < least) {
                            origin = Some(each);
                        }
                    }
                }
                (
                    origin.expect("this tick's count"),
                    counted as f64 / FRAME_RATE,
                )
            }
        };
        // The next tick, as far off as the last.
        let ahead = if self.ticks.len() > 1 {
            now - self.ticks[self.ticks.len() - 2].0
        } else {
            0.0
        };
        for (&pane, ball) in &self.falling {
            let (column, sweet, height) = (ball.column, ball.sweet, heights[&pane]);
            let seen = self.heights.entry(pane).or_default();
            let reads = self.reads.entry(pane).or_default();
            // Not moved since, the same frame read again; or headed back up, a fall no longer.
            if !seen.is_empty() && height == seen[seen.len() - 1].1 {
                continue;
            }
            if !seen.is_empty()
                && height
                    < seen[seen.len() - 1].1 - FASTEST * (at - seen[seen.len() - 1].0) - MISREAD
            {
                continue;
            }
            if !seen.is_empty() && height > seen[seen.len() - 1].1 {
                seen.clear();
                reads.clear();
            }
            seen.push((at, height));
            if let Some(counted) = counted {
                reads.push((counted, before));
            }
            if self.cued.contains(pane) || height < sweet || at - seen[0].0 < RUN_IN {
                continue;
            }
            let timed = if reads.len() == seen.len() {
                stepped(seen, reads)
            } else {
                seen.clone()
            };
            let moment = origin + timed[timed.len() - 1].0 + due(&timed, sweet) - self.lead;
            if moment - now <= ahead {
                // Python's `max(0.0, x)`: the first unless the second is strictly greater.
                let wait = moment - (self.clock)();
                (self.sleep)(if wait > 0.0 { wait } else { 0.0 });
                self.cued.add(pane);
                self.cues.insert(column, (self.clock)());
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

    /// The balls on the screen or above it on their way down, by pane.
    fn balls(&self) -> Result<IndexMap<i64, Ball>> {
        let shown = pictures(&self.layouts, &self.image_of, false, &|image: &str| {
            BALLS.iter().any(|ball| image.starts_with(ball))
        })?;
        let mut found = IndexMap::new();
        for (pane, _, image, across, up) in shown {
            if across.abs() < WIDTH as f64 / 2.0 && up > -(HEIGHT as f64) / 2.0 {
                let root = f32(&self.layouts.pane(pane)?, self.layouts.shape.y)? - up;
                let sweet = if image.starts_with(GIANT) {
                    GIANT_SWEET
                } else {
                    SWEET
                };
                found.insert(
                    pane,
                    Ball {
                        column: column(across),
                        height: up,
                        root,
                        sweet,
                    },
                );
            }
        }
        Ok(found)
    }

    /// Each column -> its Pokémon up, by its `PokemonParam` and what it is called.
    fn up(&self) -> Result<IndexMap<i64, (i64, String)>> {
        Ok(self
            .records()?
            .into_iter()
            .filter(|(_, (_, _, up, _))| *up)
            .map(|(column, (param, called, _, _))| (column, (param, called)))
            .collect())
    }

    /// Each column -> its Pokémon, by its `PokemonParam`, what it is called, whether it is
    /// up and its record: all three are there from the round's opening, those not yet up below
    /// the screen.
    #[allow(clippy::type_complexity)]
    fn records(&self) -> Result<IndexMap<i64, (i64, String, bool, pokemon::Pokemon)>> {
        let mut found: IndexMap<i64, (i64, String, bool, pokemon::Pokemon)> = IndexMap::new();
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
                if let Some(read) = read
                    && let Some(called) = (self.name)(&read)
                    && !called.is_empty()
                {
                    found.insert(column(x), (param, called, y > 0.0, read));
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

/// Seconds from the newest read until a ball reaches the height `at`, below 0 once past it,
/// from the (when, height) of its whole fall so far, oldest first: a parabola fitted to them by
/// least squares, all but the first, which can be the spare balls' 480 where the ball was 264 a
/// tick later (spike 273, live). A ball speeds up steadily as it falls, so the more of the fall
/// the better: over 528 falls of the live runs, each height timed as it was read, its moment
/// came within a frame of the cue's for 75% of them, where the speeds over the last 0.2 s alone
/// managed 44%, and within 50 ms for 92%, against 58% (spike 273). Timed by the frames the ball
/// stepped on (`stepped`), a fall lies on its parabola exactly (spike 277). Never, for a ball no
/// longer falling.
pub fn due(seen: &[(f64, f64)], at: f64) -> f64 {
    let Some((a, b, c)) = parabola(py::slice_from(seen, 1), seen[seen.len() - 1].0) else {
        return f64::INFINITY;
    };
    // The root where it falls, c t^2 + b t + (a - at) = 0 written so as not to lose b to
    // cancelling where c is small.
    let left = a - at;
    let reach = b * b - 4.0 * c * left;
    if b >= 0.0 || reach < 0.0 {
        return f64::INFINITY;
    }
    2.0 * left / (-b + reach.sqrt())
}

/// A ball's (when, height) timed by the frames it stepped on, from the game's count of frames
/// as each height was read and as the tick before read the one before it: a ball steps every
/// frame, or every other one, odd or even, and its heights lie on a parabola of its steps, so of
/// the three the one it fits best, leaving out any with a step no later than the tick that still
/// saw the height before. Its first height, which can be stale, is kept as it was read.
pub fn stepped(seen: &[(f64, f64)], reads: &[(i64, Option<i64>)]) -> Vec<(f64, f64)> {
    let (mut best, mut least) = (seen.to_vec(), f64::INFINITY);
    'parities: for parity in [None, Some(0), Some(1)] {
        let mut points: Vec<(f64, f64)> = py::slice(seen, 0, 1).to_vec();
        for (&(_, height), &(counted, before)) in seen.iter().skip(1).zip(reads.iter().skip(1)) {
            let frame = match parity {
                None => counted,
                Some(parity) => counted - py::modulo(counted - parity, 2),
            };
            if before.is_some_and(|before| frame <= before) {
                continue 'parities;
            }
            points.push((frame as f64 / FRAME_RATE, height));
        }
        let spread = spread(py::slice_from(&points, 1));
        if spread < least {
            (best, least) = (points, spread);
        }
    }
    best
}

/// How far the points lie off the parabola fitted to them, squared and summed.
fn spread(points: &[(f64, f64)]) -> f64 {
    if points.is_empty() {
        return f64::INFINITY;
    }
    let Some((a, b, c)) = parabola(points, points[points.len() - 1].0) else {
        return f64::INFINITY;
    };
    let origin = points[points.len() - 1].0;
    clips::sum(points.iter().map(|&(t, height)| {
        power_of(
            a + b * (t - origin) + c * power_of(t - origin, 2) - height,
            2,
        )
    }))
}

/// The least-squares height = a + b t + c t^2 through the points, t from `origin`; None for
/// fewer than four, or points that fix no parabola.
fn parabola(points: &[(f64, f64)], origin: f64) -> Option<(f64, f64, f64)> {
    if points.len() < 4 {
        return None;
    }
    let mut sums = [0.0; 5];
    let mut by = [0.0; 3];
    for &(when, height) in points {
        let t = when - origin;
        for (power, sum) in sums.iter_mut().enumerate() {
            *sum += power_of(t, power as i32);
        }
        for (power, sum) in by.iter_mut().enumerate() {
            *sum += power_of(t, power as i32) * height;
        }
    }
    let rows = [
        [sums[0], sums[1], sums[2]],
        [sums[1], sums[2], sums[3]],
        [sums[2], sums[3], sums[4]],
    ];
    let whole = determinant(&rows);
    if whole.abs() < 1e-12 {
        return None;
    }
    let [a, b, c] = [0, 1, 2].map(|k| {
        let mut m = rows;
        for (i, row) in m.iter_mut().enumerate() {
            row[k] = by[i];
        }
        determinant(&m) / whole
    });
    Some((a, b, c))
}

fn determinant(m: &[[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// `t ** power` as Python's float does it: 1 for a power of 0, else the C library's `pow` of the
/// size, as rail's are, never `powi`, its sign put back after for a negative `t`.
fn power_of(t: f64, power: i32) -> f64 {
    if power == 0 {
        return 1.0;
    }
    let size = t.abs().powf(f64::from(power));
    if t < 0.0 && power % 2 == 1 {
        -size
    } else {
        size
    }
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
