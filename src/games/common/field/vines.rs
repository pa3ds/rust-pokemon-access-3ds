//! Vines: X's vines the player faces and grabs with A, which carry them up or down a level or swing
//! them across a gap to the next platform, as in Coumarine City Gym (spike 250). A traversal
//! (`traversal`): its state is the vine held, or `OFF`.
//!
//! Bound by:
//! - ADR-0006: nothing here names a game's natives; `gen6.game.Vining` carries them.
//! - ADR-0024: a ride on a vine is said as a leg, the step that grabs it with the ride.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::pathfind::{Cell, Leg, State, Step};
use crate::games::common::field::chunk::{ANY_FLOOR, CLIMB};
use crate::games::common::field::ground::Ground;
use crate::games::common::field::traversal::{Name, Traversal};
use crate::py::{self, Error, Result};

pub const UP: &str = "vine up";
pub const DOWN: &str = "vine down";
/// What a ride from one end of a vine does, climbing up a level, down one, or swinging across a
/// gap on one; and what the list calls that end.
pub const SWING: &str = "swinging vine";

/// What the step that grabs a vine and its ride are said with, as one leg with the way faced to
/// grab it: "3 north, face north, climb the vine up, 2 east" (`Vined`). In Coumarine City Gym the
/// vines carry the player up or down a level, or across a gap on one (spike 250). The game names
/// no such thing.
pub const VINE: [(&str, &str); 3] = [
    (UP, "climb the vine up"),
    (DOWN, "climb the vine down"),
    (SWING, "swing on the vine"),
];
/// What a vine's ride is walked with until it is one leg with the step that grabbed it: never
/// said (`Vines.legs`).
pub const RIDE: &str = "ride the vine";

/// `VINE[ride]`.
fn vine_word(ride: &str) -> Result<&'static str> {
    VINE.iter()
        .find(|(known, _)| *known == ride)
        .map(|&(_, word)| word)
        .ok_or_else(|| Error::key(py::repr(ride)))
}

/// The search's state holding no vine. Holding one it is the end grabbed, by its place in
/// `Vines.vines` from 1.
pub const OFF: i64 = 0;

/// One end of a vine as its zone places it, before the floors round it are looked at.
#[derive(Debug, Clone, PartialEq)]
pub struct End {
    /// The number of its record's script: a vine's two ends are an odd number and the next.
    pub number: i64,
    pub cell: Cell,
    pub height: f64,
    /// Whether the vine is swung on across a gap, rather than climbed between two levels.
    pub swing: bool,
    /// Where it is grabbed from, and where a ride from its other end lands (`rail.Rails.spot`).
    pub stand: Cell,
    /// The way of the compass the player faces it from `stand`.
    pub faces: &'static str,
}

/// Each end with the other of its vine and what a ride from it does: a vine climbed is
/// climbed up from its lower end and down from its upper. An end with no other is no vine.
pub fn paired(ends: &[End]) -> Vec<(End, End, &'static str)> {
    let by: HashMap<i64, &End> = ends.iter().map(|end| (end.number, end)).collect();
    let mut out: Vec<(End, End, &'static str)> = Vec::new();
    for end in ends {
        let other = by.get(&if end.number % 2 != 0 {
            end.number + 1
        } else {
            end.number - 1
        });
        if let Some(other) = other {
            let ride = if end.swing {
                SWING
            } else if end.height < other.height {
                UP
            } else {
                DOWN
            };
            out.push((end.clone(), (*other).clone(), ride));
        }
    }
    out
}

/// One end of a vine, as a ride from it goes: grabbed from its spot, facing it, and landing
/// on the other end's.
#[derive(Debug, Clone, PartialEq)]
pub struct Vine {
    /// The number of its record's script (`End.number`), by which the list tells it apart.
    pub number: i64,
    /// The cell faced and grabbed: solid at its height, and floor on any other level.
    pub cell: Cell,
    pub height: f64,
    /// `UP`, `DOWN` or `SWING`.
    pub ride: &'static str,
    /// Where it is grabbed from (`End.stand`).
    pub stand: Cell,
    /// The way the player faces it there, which they press before A (`End.faces`).
    pub faces: &'static str,
    /// Where a ride from it lands: the other end's `stand`. Over x_gym_4_climbed every one of 22
    /// rides landed so, climbing up from (59.6, 73.4) at 3 onto (59.6, 74.4) at 148, down again,
    /// and swinging from (64.8, 74.2) to (71.2, 67.8) (spike 250).
    pub lands: Cell,
    /// How high it lands: the other end's height.
    pub landing: f64,
}

/// The zone's vines, as the search and the guide ask them: none on most maps
/// (`Vines::default()`, Python's `NO_VINES`). A vine held carries the player where it lands
/// (`leap`), and a way never ends holding one.
///
/// Python's frozensets and mappings here are only tested and looked up: `HashSet` and `HashMap`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Vines {
    pub vines: Vec<Vine>,
    /// Every end's cell, open to the search though blocked: (70, 63) has no floor at all.
    pub cells: HashSet<Cell>,
    /// (an end's spot, its cell) -> each end that step grabs, by its place in `vines`: a cell
    /// can hold an end on several levels, as (55, 73) holds four. A swing's spot touches its cell
    /// at a corner, the step a diagonal one (`Ground.slants`).
    pub boards: HashMap<(Cell, Cell), Vec<i64>>,
    /// Each end's cell, with the height of every end in it.
    pub levels: HashMap<Cell, Vec<f64>>,
}

impl Vines {
    /// The vines of `ends` (`paired`). In Coumarine Gym the player walked north up to (59,
    /// 72.9) and was stopped by the vine at (59, 72), height 0, whose other end is (59, 73) at
    /// 145 (spike 250).
    pub fn found(ends: &[End]) -> Vines {
        let out: Vec<Vine> = paired(ends)
            .into_iter()
            .map(|(end, other, ride)| Vine {
                number: end.number,
                cell: end.cell,
                height: end.height,
                ride,
                stand: end.stand,
                faces: end.faces,
                lands: other.stand,
                landing: other.height,
            })
            .collect();
        let mut boards: HashMap<(Cell, Cell), Vec<i64>> = HashMap::new();
        let mut levels: HashMap<Cell, Vec<f64>> = HashMap::new();
        for (i, vine) in out.iter().enumerate() {
            boards
                .entry((vine.stand, vine.cell))
                .or_default()
                .push(i as i64);
            levels.entry(vine.cell).or_default().push(vine.height);
        }
        Vines {
            vines: out,
            cells: levels.keys().copied().collect(),
            boards,
            levels,
        }
    }

    pub fn on(ground: &Ground) -> Option<Rc<dyn Traversal>> {
        if ground.vines.vines.is_empty() {
            None
        } else {
            Some(ground.vines.clone() as Rc<dyn Traversal>)
        }
    }

    /// Whether an end stands in `cell` within `CLIMB` of `height`: where the vine is in the
    /// way. Never where `height` is not given.
    pub fn at(&self, cell: Cell, height: Option<f64>) -> bool {
        let Some(height) = height else {
            return false;
        };
        self.levels
            .get(&cell)
            .is_some_and(|levels| levels.iter().any(|&level| (height - level).abs() <= CLIMB))
    }

    /// The vine held after the step, `OFF` once its ride lands, or None where the step cannot
    /// be taken: the search's `through`, from a floor at `here`, None where that is not known.
    /// Holding one, the one step is its ride (`leap`). A step from an end's spot into its cell
    /// at the end's level grabs it, never from a floor whose height is not known, which grabbed
    /// the ground's vine from the top level; any other step into an end's cell at its level is
    /// refused, as the player was stopped at (59, 72); on another level the cell is floor, as
    /// the top level, 720, walks over (59, 72).
    pub fn hold(&self, held: i64, cell: Cell, step: Cell, here: Option<f64>) -> Option<i64> {
        if held != OFF {
            return (step == self.vines[(held - 1) as usize].lands).then_some(OFF);
        }
        for &end in self.boards.get(&(cell, step)).into_iter().flatten() {
            if let Some(here) = here
                && (here - self.vines[end as usize].height).abs() <= CLIMB
            {
                return Some(end + 1);
            }
        }
        if self.levels.contains_key(&step) && (here.is_none() || self.at(step, here)) {
            None
        } else {
            Some(OFF)
        }
    }
}

impl Traversal for Vines {
    fn carries(&self) -> bool {
        true
    }

    fn start(&self, _ground: &Ground, _cell: Cell, _floor: i64) -> Result<State> {
        Ok(vec![OFF])
    }

    /// `hold`, from the height of `floor` in `cell`; and never into a vine's cell with
    /// no floor but to grab it.
    fn step(
        &self,
        ground: &Ground,
        state: &[i64],
        cell: Cell,
        step: Cell,
        floor: i64,
    ) -> Result<Option<State>> {
        let held = self.hold(state[0], cell, step, ground.height(cell, floor)?);
        let Some(held) = held else {
            return Ok(None);
        };
        if OFF == state[0]
            && state[0] == held
            && self.cells.contains(&step)
            && ground.floors(step)?.is_empty()
        {
            return Ok(None);
        }
        Ok(Some(vec![held]))
    }

    fn lands(
        &self,
        ground: &Ground,
        before: &[i64],
        after: &[i64],
        _cell: Cell,
        step: Cell,
        _floor: i64,
    ) -> Result<Option<i64>> {
        // A vine held is on no floor, and its ride lands on the floor at its height.
        if after[0] != OFF {
            return Ok(Some(ANY_FLOOR));
        }
        if before[0] != OFF {
            return Ok(Some(ground.floor(
                step,
                Some(self.vines[(before[0] - 1) as usize].landing),
            )?));
        }
        Ok(None)
    }

    fn stops(
        &self,
        _ground: &Ground,
        state: &[i64],
        _cell: Cell,
        _floor: i64,
        _ends: &HashSet<Cell>,
    ) -> Result<bool> {
        Ok(state[0] == OFF)
    }

    /// Where the vine held carries the player, the one move from its cell; None holding
    /// none.
    fn leap(&self, _ground: &Ground, state: &[i64], _cell: Cell, _floor: i64) -> Option<Cell> {
        if state[0] == OFF {
            None
        } else {
            Some(self.vines[(state[0] - 1) as usize].lands)
        }
    }

    /// Each step that grabs a vine, with what its ride does, and the ride after it.
    fn said(
        &self,
        _ground: &Ground,
        _way: &[Step],
        mine: &[&[i64]],
        _name: Name<'_>,
    ) -> Result<IndexMap<i64, String>> {
        let grabs: Vec<(i64, &Vine)> = mine
            .iter()
            .enumerate()
            .filter(|&(i, held)| i != 0 && held[0] != OFF)
            .map(|(i, held)| (i as i64, &self.vines[(held[0] - 1) as usize]))
            .collect();
        let mut out: IndexMap<i64, String> = IndexMap::new();
        for &(i, _) in &grabs {
            out.insert(i + 1, RIDE.to_string());
        }
        for &(i, vine) in &grabs {
            out.insert(i, vine_word(vine.ride)?.to_string());
        }
        Ok(out)
    }

    /// `legs` with each step that grabs a vine and the ride after it one leg, said the way
    /// the vine is faced from its spot (`Vined`, a `Leg` whose `vined` is set).
    fn legs(
        &self,
        _ground: &Ground,
        _way: &[Step],
        mine: &[&[i64]],
        legs: Vec<Leg>,
    ) -> Result<Vec<Leg>> {
        let mut out: Vec<Leg> = Vec::new();
        for leg in legs {
            match out.last_mut() {
                Some(grab) if leg.how == RIDE => {
                    let faces = self.vines[(mine[grab.end as usize][0] - 1) as usize].faces;
                    *grab = Leg {
                        direction: faces,
                        steps: grab.steps + leg.steps,
                        end: leg.end,
                        how: grab.how.clone(),
                        held: None,
                        vined: true,
                        teleported: false,
                    };
                }
                _ => out.push(leg),
            }
        }
        Ok(out)
    }
}
