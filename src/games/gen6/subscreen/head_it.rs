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
//! The round's score, its combo and each "MISS!" come at every ball, faster than a screen reader
//! says them behind the balls (players, 2026-11-04), and are left unsaid as they come unless the
//! player asks (`[pa3ds.amie] head_it_score`); a key says them, with the clock or the chances left
//! (`score`, Orion, 2026-11-04). The game's words, as the screen draws them, each known by the
//! layout it is drawn in and that layout's pane (`SCORE_WINDOW`, `HIT`, `STATUS`).
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
use crate::games::common::field::keys::{HEAD_IT, HEADERS, SCORE_KEY};
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::pokemon;
use crate::games::common::screen::{ScreenText, in_rows, join};
use crate::games::common::touch::{FileOf, HEIGHT, ImageOf, OnScreen, Target, WIDTH};
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
pub const BALL: &str = "N3app7heading4BallE";
pub const BALL_RECORDS: i64 = 0x1C;
pub const BALL_SIZE: i64 = 0x3C;
pub const BALL_COUNT: i64 = 20;
pub const SPEED: i64 = 0x10;
/// The game's own record of each ball, held by its `app::heading::Ball`, a record a ball, its
/// index first (spike 273, live):
///
/// ```text
/// record  +0x10  its speed: how far it fell at its last step
///         +0x14  its rise: how much faster each step makes it, 0.2 to 1.2, the same from the
///                top to the bottom
///         +0x28  x, then y, as its picture has them
/// ```
///
/// Each step it falls its speed and its rise more, a step every other frame (`STEP`): over 35 falls
/// of Unlimited and 35 of a round of Hard live, the giant ball's among them, every height was the
/// one before less the speed after, to the last digit.
pub const ACROSS: i64 = 0x28;
/// How far from where a ball's picture was read its record may be, a step apart as the two were
/// read: the fastest steps read were 27.
pub const NEAR: f64 = 30.0;
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
/// 3 points rather than 1, and 6 rather than 2 from the 16th hit of a combo: the middle of 12 to -38,
/// where taps sent 0 to 0.2 s after balls of 23 to 33 a step reached 35 were, the same heights at
/// every speed; a ball is headed at all from about 50 to -100 (or_amie_head_it_hard, spike 273,
/// live). It was 35, from taps by a cue that took a ball to keep its speed, 40 to 70 ms late: timed
/// by its record, every tap at 35 was headed and none at the sweet spot. Every ball's but the
/// giant's (`GIANT_SWEET`).
pub const SWEET: f64 = -13.0;
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
/// The game's frames a second as `gen6.frames` counts them, the screen's: 59.83. The round's own
/// count of frames (`Process` +0x60, 30 a second) stands still while the giant ball falls (spike
/// 277, live).
pub const FRAME_RATE: f64 = 268111856.0 / 4481136.0;
/// Seconds a ball's step takes, by its record (`BALL`): two frames, every ball on the same ones. Of
/// the steps of three runs live, Omega Ruby's Unlimited and Hard and X's Hard, 2.0 frames apart on
/// average, nine in ten were first read on an even frame and the rest a frame later (spike 273).
pub const STEP: f64 = 2.0 / FRAME_RATE;
/// Seconds apart ticks read at most while balls fall (`tick`), 10 to 40 ms, and up to 100 behind a
/// slow poll of the screen (spike 273, live): a cue due before the next waits for its moment, the
/// next taken to be as far off as the furthest apart of the last second's, those further apart
/// being the time no ball fell. Taken from the tick before, a second back, a ball cued from its
/// first read after none fell waited 0.3 to 0.7 s in the tick, and the screen with it; with that
/// time in it at 0.1 s, a cue waited 93 ms (spike 273, live). Taken to be as far off as the last
/// alone, a tick 10 ms after the one before let a cue due 20 ms on wait for one 40 ms on.
pub const TICK: f64 = 0.1;
/// Seconds of ticks the frame clock is set by: when each frame began, by the tick that read its
/// count soonest after, so a stall of the emulator is forgotten a second on.
pub const WINDOW: f64 = 1.0;
/// A picture of the layout the round's score and combo are drawn in: the score in pane `2`,
/// "819", and from the third hit the combo in pane `7`, "98-HIT COMBO", for a second after each
/// hit; its other words are the results' "Poké Puffs received!" (or_amie_head_it_normal's and
/// _unlimited's tapes).
pub const SCORE_WINDOW: &str = "scorewindow.bclim";
pub const SCORE_PANE: &str = "2";
pub const COMBO_PANE: &str = "7";
/// A picture of the layout a ball headed or missed is drawn in, whose only words are "MISS!", a
/// pane a column (`MISSED`).
pub const HIT: &str = "hit01.bclim";
pub const MISSED: &str = "missed";
/// A picture of the layout the round's clock is drawn in, "Time Left: 23", in group `5m`, or
/// Unlimited's chances, "Chances:" and "2", in group `5j` (`GROUPS`, `LEFT`); and its high score,
/// and the results.
pub const STATUS: &str = "win_heading.bclim";
pub const GROUPS: [&str; 2] = ["5m", "5j"];
pub const LEFT: &str = "left";
pub const COMING: &str = "head-it-ball";
pub const NOBODY: &str = "head-it-ball-nobody";
pub const NOT_UP: &str = "head-it-no-pokemon";
pub const NO_SCORE: &str = "head-it-no-score";
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
    /// The layout file a layout's root was built from, which knows the score's layouts by
    /// their pictures (`scored`).
    file_of: FileOf,
    /// The screen's words as its last poll read them.
    on_screen: OnScreen,
    /// The round's combo as the screen last showed it, until a "MISS!" ends it (`score`).
    combo: String,
    names: Rc<ClassNames>,
    layouts: Rc<Layouts>,
    image_of: ImageOf,
    /// What a Pokémon of the player's is called.
    name: Name,
    /// Model object -> the vtable it was found with.
    models: IndexMap<i64, i64>,
    /// The `Ball` objects the sweep found: one is the round's, and the sweep finds others
    /// that never change (spike 273, live).
    ball_objects: Vec<i64>,
    /// Each ball falling -> where its record in the round's `Ball` begins, once found
    /// (`_in_flight`).
    flights: IndexMap<i64, i64>,
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
    /// How many of the round's ticks first read a ball's step on an even frame, and on an odd
    /// one: its balls step on the frames of the most (`STEP`).
    pub(crate) parity: [i64; 2],
    /// Plays a Pokémon's cry toward its side as its ball comes, and says whether it did; None
    /// where Pokémon are said by name alone (`[pa3ds.amie] head_it_style`).
    cry: Option<Crier>,
    /// Whether the cry, where it plays, is said in the name's place.
    instead: bool,
    /// How a ball is said as it comes: spoken, or only logged, for a player who asks for the
    /// cue alone (`head_it_style` "none").
    said: Option<Category>,
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
    /// What the heap is swept for: the Pokémon, who stands where, and the balls' own records,
    /// how fast each falls. The app's own object, which says whether balls fall, is the process
    /// the game runs, known as the app starts (`process`).
    pub const CLASSES: [&'static str; 2] = [MODEL, BALL];

    /// `HeadIt(...)` with every argument; Python's `lead` defaults to `LEAD`, its `clock` to
    /// `time.monotonic`, its `cry` to None, its `instead` to False, its `named` to True, its
    /// `frames` to none, its `sleep` to `time.sleep`, its `process` to none, its `file_of` to
    /// none and its `on_screen` to nothing.
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
        named: bool,
        frames: Frames,
        sleep: Rc<dyn Fn(f64)>,
        process: Process,
        file_of: FileOf,
        on_screen: OnScreen,
    ) -> Self {
        HeadIt {
            memory,
            file_of,
            on_screen,
            combo: String::new(),
            names,
            layouts,
            image_of,
            name,
            models: IndexMap::new(),
            ball_objects: Vec::new(),
            flights: IndexMap::new(),
            process,
            falling: IndexMap::new(),
            cue,
            lead,
            clock,
            frames,
            sleep,
            ticks: Vec::new(),
            parity: [0, 0],
            cry,
            instead,
            said: named.then_some(Category::Screen),
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

    /// The models and `Ball`s a sweep found: all of them, or with `adding` the new ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut kept = if adding {
            self.models.clone()
        } else {
            IndexMap::new()
        };
        let mut balls = if adding {
            self.ball_objects.clone()
        } else {
            Vec::new()
        };
        for (&at, name) in found {
            if name == MODEL {
                kept.insert(at, u32(&self.memory.read(at, 4)?, 0)?);
            } else if name == BALL && !balls.contains(&at) {
                balls.push(at);
            }
        }
        (self.models, self.ball_objects) = (kept, balls);
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
            (self.falling, self.missed, self.combo) =
                (IndexMap::new(), PySet::new(), String::new());
            (self.heights, self.cued, self.flights, self.parity) =
                (IndexMap::new(), PySet::new(), IndexMap::new(), [0, 0]);
            return Ok(said);
        }
        let shown = self.shown()?;
        if shown.contains_key(MISSED) {
            self.combo = String::new();
        } else if let Some(combo) = shown.get(COMBO_PANE) {
            self.combo = join(&in_rows(combo.clone()), false);
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
        self.flights.retain(|pane, _| heights.contains_key(pane));
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
                said.push((LOG.to_string(), own(NOBODY, &[]), self.said));
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
                self.said,
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

    /// Whether words on the screen are the round's score, its combo or a "MISS!": what comes
    /// at every ball (`screen_announce`).
    pub fn scored(&self, piece: &ScreenText, running: Option<&str>) -> Result<bool> {
        if running != Some(APP) {
            return Ok(false);
        }
        let part = self.part(piece, (self.file_of)(piece.root).as_deref())?;
        Ok(part.is_some_and(|part| [SCORE_PANE, COMBO_PANE, MISSED].contains(&part)))
    }

    /// The round's score, its combo and its clock or chances, "819, 98-HIT COMBO, Time
    /// Left: 23", or "" where none is on the screen. The combo is the one last shown: the game
    /// shows it for a second after each hit, and it stands until a "MISS!" (or_amie_head_it_
    /// normal's tape).
    pub fn score(&self, running: Option<&str>) -> Result<String> {
        if running != Some(APP) {
            return Ok(String::new());
        }
        let shown = self.shown()?;
        let mut said: Vec<String> = Vec::new();
        if let Some(score) = shown.get(SCORE_PANE) {
            said.push(join(&in_rows(score.clone()), false));
        }
        if !self.combo.is_empty() {
            said.push(self.combo.clone());
        }
        if let Some(left) = shown.get(LEFT) {
            said.push(join(&in_rows(left.clone()), false));
        }
        Ok(said.join(", "))
    }

    /// The round's words on the screen by what they are (`_part`).
    fn shown(&self) -> Result<IndexMap<&'static str, Vec<ScreenText>>> {
        let mut shown: IndexMap<&'static str, Vec<ScreenText>> = IndexMap::new();
        let mut files: IndexMap<i64, Option<Rc<LayoutFile>>> = IndexMap::new();
        for piece in (self.on_screen)() {
            if !files.contains_key(&piece.root) {
                files.insert(piece.root, (self.file_of)(piece.root));
            }
            if let Some(part) = self.part(&piece, files[&piece.root].as_deref())? {
                shown.entry(part).or_default().push(piece);
            }
        }
        Ok(shown)
    }

    /// What words of a layout built from `file` are of the round: its score `SCORE_PANE`, its
    /// combo `COMBO_PANE`, a "MISS!" `MISSED`, or its clock or chances `LEFT`; else None.
    fn part(&self, piece: &ScreenText, file: Option<&LayoutFile>) -> Result<Option<&'static str>> {
        let Some(file) = file else {
            return Ok(None);
        };
        if draws(file, HIT) {
            return Ok(Some(MISSED));
        }
        if draws(file, SCORE_WINDOW) {
            let name = name(&self.layouts, piece.r#box)?;
            return Ok([SCORE_PANE, COMBO_PANE]
                .into_iter()
                .find(|&pane| pane == name));
        }
        if draws(file, STATUS) && GROUPS.contains(&name(&self.layouts, piece.parent)?.as_str()) {
            return Ok(Some(LEFT));
        }
        Ok(None)
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
    /// the ball reaches its sweet spot, `SWEET` or the giant's `GIANT_SWEET`. The game's own
    /// record of the ball (`_in_flight`) says how far it falls at its next step and how much
    /// faster each step makes it, so when it gets there (`falls`). Balls fall at different
    /// speeds, each faster and faster, and past about 100 hits in under a second: fitted to the
    /// heights read of its first 0.2 s, as the cue was timed before, a third or more of such a
    /// ball's cues came early (spike 273, live). The screen's poll, a fifth of a second, finds
    /// the balls; a ball's record is read every tick.
    ///
    /// A tick reads 10 to 40 ms apart, up to a frame after the ball stepped, so a height is timed
    /// by the frame it stepped on: the last of the parity most of the round's steps were first
    /// read on (`STEP`). A frame's moment is when the tick that read its count soonest after it
    /// did, over the last second (`WINDOW`). And a cue due before the next tick waits for its
    /// moment here, where it sounded at the read nearest it, up to half a tick off: a perfect
    /// player's taps, the lead after each cue on Hard, found the ball 0 to 50 above the middle
    /// at the next read 32 times in 33, against 23 to 28 before (spike 277, live). Without the
    /// count, each height is timed as it was read.
    ///
    /// A ball is cued only falling, its last height below the one before: headed back up, its
    /// record's speed stands where it was (spike 273, live). A ball whose record is not found is
    /// not cued.
    pub fn tick(&mut self) -> Result<()> {
        let Some(cue) = self.cue.clone() else {
            return Ok(());
        };
        let counted = (self.frames)()?;
        let y = self.layouts.shape.y;
        // Not through `Layouts`, which keeps what it read until the screen's next poll.
        let mut read: IndexMap<i64, f64> = IndexMap::new();
        for (&pane, ball) in &self.falling {
            read.insert(pane, f32(&self.memory.read(pane + y, 4)?, 0)? - ball.root);
        }
        let flights = self.in_flight(&read)?;
        let now = (self.clock)();
        if counted.is_some() && (self.frames)()? != counted {
            return Ok(());
        }
        let heights: IndexMap<i64, f64> = read
            .iter()
            .map(|(&pane, &height)| {
                (
                    pane,
                    flights.get(&pane).map_or(height, |&(flown, _, _)| flown),
                )
            })
            .collect();
        let moved: Vec<i64> = heights
            .iter()
            .filter(|&(pane, &height)| match self.heights.get(pane) {
                Some(seen) if !seen.is_empty() => seen[seen.len() - 1].1 != height,
                _ => true,
            })
            .map(|(&pane, _)| pane)
            .collect();
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
                if moved.iter().any(|pane| flights.contains_key(pane)) {
                    self.parity[py::modulo(counted, 2) as usize] += 1;
                }
                let odd = i64::from(self.parity[1] > self.parity[0]);
                let frame = if self.parity.iter().any(|&each| each != 0) {
                    counted - py::modulo(counted - odd, 2)
                } else {
                    counted
                };
                (
                    origin.expect("this tick's count"),
                    frame as f64 / FRAME_RATE,
                )
            }
        };
        for &pane in &moved {
            let seen = self.heights.entry(pane).or_default();
            if !seen.is_empty() && heights[&pane] > seen[seen.len() - 1].1 {
                seen.clear();
            }
            seen.push((at, heights[&pane]));
        }
        // The next tick, as far off as the furthest apart of the last second's, but for those
        // further apart than `TICK`, between which no ball fell.
        // Python's `max(..., default=0.0)`: the first unless a later one is strictly greater.
        let mut ahead: Option<f64> = None;
        for pair in self.ticks.windows(2) {
            let gap = pair[1].0 - pair[0].0;
            if gap <= TICK && ahead.is_none_or(|most| gap > most) {
                ahead = Some(gap);
            }
        }
        let ahead = ahead.unwrap_or(0.0);
        let mut due: Vec<(f64, i64)> = Vec::new();
        for (&pane, &(height, speed, rise)) in &flights {
            let (ball, seen) = (self.falling[&pane], &self.heights[&pane]);
            if !self.cued.contains(pane) && height >= ball.sweet && seen.len() >= 2 {
                due.push((
                    origin + seen[seen.len() - 1].0 + falls(height, speed, rise, ball.sweet)
                        - self.lead,
                    pane,
                ));
            }
        }
        // Soonest first: two balls due before the next tick, waited for in the order found, cued
        // the sooner late.
        // Python's `sorted` of (moment, pane) tuples: the first parts that differ, by `<`.
        let due = py::sorted_by_lt(
            due,
            |a: &(f64, i64), b: &(f64, i64)| {
                if a.0 != b.0 { a.0 < b.0 } else { a.1 < b.1 }
            },
        );
        for (moment, pane) in due {
            if moment - now > ahead {
                break;
            }
            let column = self.falling[&pane].column;
            // Python's `max(0.0, x)`: the first unless the second is strictly greater.
            let wait = moment - (self.clock)();
            (self.sleep)(if wait > 0.0 { wait } else { 0.0 });
            self.cued.add(pane);
            self.cues.insert(column, (self.clock)());
            cue(column);
        }
        Ok(())
    }

    /// Each ball falling whose record in the round's `Ball` is found -> its height, speed and
    /// rise as the record has them: the record falling in its column, within `NEAR` of the
    /// height its picture was read at, kept for the ball once found. The giant ball's is one
    /// of the twenty, stepping as the others do (spike 273, live on Hard).
    fn in_flight(
        &mut self,
        heights: &IndexMap<i64, f64>,
    ) -> Result<IndexMap<i64, (f64, f64, f64)>> {
        let mut found: IndexMap<i64, (f64, f64, f64)> = IndexMap::new();
        let mut records: Option<Vec<(i64, Vec<u8>)>> = None;
        for (&pane, &height) in heights {
            let ball = self.falling[&pane];
            let kept_looked: Vec<(i64, Vec<u8>)>;
            let looked: &[(i64, Vec<u8>)] = match self.flights.get(&pane) {
                Some(&kept) => {
                    kept_looked = vec![(kept, self.memory.read(kept, BALL_SIZE)?)];
                    &kept_looked
                }
                None => {
                    if records.is_none() {
                        records = Some(self.ball_records()?);
                    }
                    records.as_deref().unwrap_or(&[])
                }
            };
            let mut best: Option<(f64, i64, (f64, f64, f64))> = None;
            for (at, raw) in looked {
                let (speed, rise) = (f32(raw, SPEED)?, f32(raw, SPEED + 4)?);
                let (x, y) = (f32(raw, ACROSS)?, f32(raw, ACROSS + 4)?);
                let off = (y - height).abs();
                if speed > 0.0
                    && column(x) == ball.column
                    && off <= NEAR
                    && best.is_none_or(|(least, _, _)| off < least)
                {
                    best = Some((off, *at, (y, speed, rise)));
                }
            }
            let Some((_, at, flight)) = best else {
                self.flights.shift_remove(&pane);
                continue;
            };
            self.flights.insert(pane, at);
            found.insert(pane, flight);
        }
        Ok(found)
    }

    /// Every record of the `Ball`s found, where each begins and what it holds: a record
    /// holds its index first, which the others found never do.
    fn ball_records(&self) -> Result<Vec<(i64, Vec<u8>)>> {
        let mut records: Vec<(i64, Vec<u8>)> = Vec::new();
        for &at in &self.ball_objects {
            let raw = self
                .memory
                .read(at, BALL_RECORDS + BALL_COUNT * BALL_SIZE)?;
            for i in 0..BALL_COUNT {
                let start = BALL_RECORDS + i * BALL_SIZE;
                if u32(&raw, start)? == i {
                    records.push((
                        at + start,
                        py::slice(&raw, start, start + BALL_SIZE).to_vec(),
                    ));
                }
            }
        }
        Ok(records)
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

/// Whether a layout file has a picture drawn with a texture, shown or not.
fn draws(file: &LayoutFile, texture: &str) -> bool {
    file.textures
        .values()
        .any(|textures| textures.iter().any(|each| each == texture))
}

fn name(layouts: &Layouts, pane: i64) -> Result<String> {
    let name = layouts.name(pane)?;
    let end = name
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(name.len());
    Ok(py::decode_ascii(&name[..end]))
}

/// Seconds from a ball's last step until it reaches the height `at`, by its record (`BALL`):
/// `k` steps on it has fallen `k` speeds and `k (k + 1) / 2` rises, a step every `STEP`. Between
/// steps, as the parabola through them; never, for a ball not falling.
pub fn falls(height: f64, speed: f64, rise: f64, at: f64) -> f64 {
    // rise / 2 k^2 + (speed + rise / 2) k = height - at, written so as not to lose it to
    // cancelling where the rise is small.
    let left = height - at;
    let b = speed + rise / 2.0;
    let reach = b * b + 2.0 * rise * left;
    if b <= 0.0 || reach < 0.0 {
        return f64::INFINITY;
    }
    2.0 * left / (b + reach.sqrt()) * STEP
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
/// And a key that says the round's score, its combo and its clock or chances (`HeadIt.score`),
/// which are left unsaid as they come unless the player asks (Orion, 2026-11-04: "the game itself
/// already plays a sound" for a miss).
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

    /// Tap the Pokémon of the key, or say the score, with the line to say.
    pub fn press(
        &self,
        action: &str,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<String> {
        if action == SCORE_KEY {
            let score = self.game.borrow().score(running)?;
            return Ok(if score.is_empty() {
                (self.wrong)(&own(NO_SCORE, &[]))
            } else {
                score
            });
        }
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
