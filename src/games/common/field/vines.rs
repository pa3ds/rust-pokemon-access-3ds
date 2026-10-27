//! Vines: X's vines the player faces and grabs with A, which carry them up or down a level or swing
//! them across a gap to the next platform, as in Coumarine City Gym (spike 250).
//!
//! Bound by:
//! - ADR-0006: nothing here names a game's natives; `gen6.game.Vining` carries them.
//! - ADR-0024: a ride on a vine is said as a leg, the step that grabs it with the ride.

use std::collections::{HashMap, HashSet};

use crate::core::pathfind::Cell;
use crate::games::common::field::chunk::CLIMB;

pub const UP: &str = "vine up";
pub const DOWN: &str = "vine down";
/// What a ride from one end of a vine does, climbing up a level, down one, or swinging across a
/// gap on one; and what the list calls that end.
pub const SWING: &str = "swinging vine";

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
/// (`Vines::default()`, Python's `NO_VINES`).
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

    /// Where the vine held carries the player, the one move from its cell: the search's
    /// `leap`. None holding none.
    pub fn leap(&self, held: i64) -> Option<Cell> {
        if held == OFF {
            None
        } else {
            Some(self.vines[(held - 1) as usize].lands)
        }
    }
}
