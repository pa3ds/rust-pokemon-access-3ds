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
//! Bound by:
//! - ADR-0009: a Pokémon is tapped on the player's key, by touch mode, as the target it has just
//!   named.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::discovery::HEAP_SPAN;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::pokemon;
use crate::games::common::touch::{HEIGHT, ImageOf, Target, WIDTH};
use crate::games::gen6::subscreen::berry_picker::Name;
use crate::games::gen6::subscreen::super_training::pictures;
use crate::py::Result;

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
pub const COMING: &str = "head-it-ball";
pub const NOBODY: &str = "head-it-ball-nobody";
/// What the log calls what Head It says.
pub const LOG: &str = "head it";

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
    /// The balls on their way down at the last poll, by pane.
    ///
    /// A set only tested.
    falling: HashSet<i64>,
}

impl HeadIt {
    /// What the heap is swept for: the Pokémon, who stands where, and the app's own object, which
    /// says whether balls fall.
    pub const CLASSES: [&'static str; 2] = [MODEL, APP];

    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        name: Name,
    ) -> Self {
        HeadIt {
            memory,
            names,
            layouts,
            image_of,
            name,
            models: IndexMap::new(),
            apps: IndexMap::new(),
            falling: HashSet::new(),
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

    /// Each ball that has just come, by the Pokémon it falls at, as the screen's words, while
    /// balls fall.
    pub fn poll(&mut self, running: Option<&str>) -> Result<Vec<(String, String, Category)>> {
        if running != Some(APP) || !self.playing()? {
            self.falling = HashSet::new();
            return Ok(Vec::new());
        }
        let falling = self.balls()?;
        let new: Vec<i64> = falling
            .iter()
            .filter(|(pane, _)| !self.falling.contains(*pane))
            .map(|(_, &column)| column)
            .collect();
        self.falling = falling.keys().copied().collect();
        if new.is_empty() {
            return Ok(Vec::new());
        }
        let standing = self.standing()?;
        Ok(new
            .into_iter()
            .map(|column| {
                let line = match standing.get(&column) {
                    Some(called) => own(COMING, &[("pokemon", called)]),
                    None => own(NOBODY, &[]),
                };
                (LOG.to_string(), line, Category::Screen)
            })
            .collect())
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
        for (&at, &vtable) in &self.apps {
            let head = self.memory.read(at, PHASE + 4)?;
            // A copy in the module's code, as `Model`'s are, holds an instruction there: a word
            // far past the last phase.
            if u32(&head, 0)? == vtable {
                let phase = u32(&head, PHASE)?;
                if phase <= LIST {
                    return Ok((FALLING..=LAST).contains(&phase));
                }
            }
        }
        Ok(false)
    }

    /// The balls on the screen or above it on their way down, by pane, each with its column.
    fn balls(&self) -> Result<IndexMap<i64, i64>> {
        let shown = pictures(&self.layouts, &self.image_of, false, &|image: &str| {
            BALLS.iter().any(|ball| image.starts_with(ball))
        })?;
        Ok(shown
            .into_iter()
            .filter(|&(_, _, _, across, up)| {
                across.abs() < WIDTH as f64 / 2.0 && up > -(HEIGHT as f64) / 2.0
            })
            .map(|(pane, _, _, across, _)| (pane, column(across)))
            .collect())
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
        let mut found: IndexMap<i64, (i64, String)> = IndexMap::new();
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
                if y <= 0.0 || !(HEAP_SPAN.0 <= param && param < HEAP_SPAN.1) {
                    continue;
                }
                let read = pokemon::read(&*self.memory, &self.names, param, pokemon::SPECIES_MAX)?;
                if let Some(called) = read.as_ref().and_then(|read| (self.name)(read))
                    && !called.is_empty()
                {
                    found.insert(column(x), (param, called));
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
