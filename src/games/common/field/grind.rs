//! Grind rails: X's rails the player skates onto on roller skates and is carried along, end to
//! end, as in Shalour City Gym (spike 247). A traversal (`traversal`): its state is the ride the
//! player is on, or `OFF`.
//!
//! Bound by:
//! - ADR-0006: nothing here names a game's kinds; `gen6.game.Grinding` carries them.
//! - ADR-0024: a ride along a rail is said as legs, each covering cells.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::pathfind::{self, Cell, State};
use crate::games::common::field::chunk::{ANY_FLOOR, BLOCKED, KIND_SHIFT};
use crate::games::common::field::ground::Ground;
use crate::games::common::field::traversal::Traversal;
use crate::py::Result;

/// The search's state off every rail. On one it is a ride's number, from 1: `2 * i + 1` along
/// rail `i` from its first cell, `2 * i + 2` back from its last.
pub const OFF: i64 = 0;

/// One rail, as a ride along it goes: its end cell got on at, every rail cell in order, and
/// the cell it lands on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrindRail {
    pub cells: Vec<Cell>,
    /// Whether it is ridden either way, an end cell at each end: got on at its last cell too, and
    /// landing on its first.
    pub both: bool,
}

/// The matrix's grind rails, as the search and the guide ask them: none on most maps
/// (`GrindRails::default()`, Python's `NO_GRIND_RAILS`). A way never ends on a ride, which
/// carries the player on to the rail's end: P to Shalour Gym's centre, its gate shut, said
/// "grind the rail 5 west, then can't continue" (spike 247).
///
/// Python's frozensets and mappings here are only tested and looked up: `HashSet` and `HashMap`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GrindRails {
    pub rails: Vec<GrindRail>,
    /// Every rail cell, open to the search though blocked: Shalour Gym's are `0x8c000021`.
    pub cells: HashSet<Cell>,
    /// (a ride, a cell on it) -> the next cell of the ride, the end cell got on at among them.
    pub ahead: HashMap<(i64, Cell), Cell>,
    /// (an end cell, the rail cell in line past it) -> the ride that step gets on.
    pub boards: HashMap<(Cell, Cell), i64>,
    /// Every step of every ride, from the end cell got on at to the cell it lands on: said as one
    /// leg (`guide.GRIND`), and taken diagonally where a rail bends (`Ground.slants`).
    pub steps: HashSet<(Cell, Cell)>,
}

impl GrindRails {
    /// The rails, with what the search asks worked out once.
    pub fn of(rails: Vec<GrindRail>) -> GrindRails {
        let mut ahead: HashMap<(i64, Cell), Cell> = HashMap::new();
        let mut boards: HashMap<(Cell, Cell), i64> = HashMap::new();
        for (i, rail) in rails.iter().enumerate() {
            let i = i as i64;
            let mut rides: Vec<(i64, Vec<Cell>)> = vec![(2 * i + 1, rail.cells.clone())];
            if rail.both {
                rides.push((2 * i + 2, rail.cells.iter().rev().copied().collect()));
            }
            for (ride, cells) in rides {
                boards.insert((cells[0], cells[1]), ride);
                for pair in cells.windows(2) {
                    ahead.insert((ride, pair[0]), pair[1]);
                }
            }
        }
        let cells = rails
            .iter()
            .flat_map(|rail| rail.cells[1..rail.cells.len() - 1].iter().copied())
            .collect();
        let steps = ahead
            .iter()
            .map(|(&(_, cell), &step)| (cell, step))
            .collect();
        GrindRails {
            rails,
            cells,
            ahead,
            boards,
            steps,
        }
    }

    /// The rails in `floors`, each floor's cells with their words: a rail is a chain of
    /// `rail` cells of one floor, each beside the next or touching it at a corner. Past each end
    /// of the chain, in the line of its last step, is the cell a ride gets on at, where any floor
    /// has the `end` kind there, or lands on, where it is not that but open on some floor. A
    /// chain with an end at both is ridden either way; with one, only from it; with none, or
    /// with an end against a blocked cell, not at all. In Shalour City Gym the player skated
    /// north from `0x8b` at (67, 62) along `0x8c` (67, 61) to (67, 53) onto `0x8b` at (67, 52);
    /// the Gym's way out, on its floor 1, has an end only at its top, on floor 0, and lands on
    /// the floor by the entrance; and on Route 5 a rail over the water is broken by `0x90` at
    /// (701, 455), which neither half is ridden past (spike 247).
    ///
    /// Python takes a floor's rail cells as a `set` and pops a chain's first cell off it, in
    /// CPython's order for tuples; here it is the floor's own order, last first. The order
    /// numbers the rides alone: what a way is and how it is said do not depend on it.
    pub fn found(floors: &[IndexMap<Cell, i64>], rail: i64, end: i64) -> GrindRails {
        let words = |cell: Cell| -> Vec<i64> {
            floors
                .iter()
                .filter_map(|floor| floor.get(&cell).copied())
                .collect()
        };

        let kinds = |cell: Cell| -> HashSet<i64> {
            words(cell)
                .into_iter()
                .map(|word| word >> KIND_SHIFT)
                .collect()
        };

        let open = |cell: Cell| -> bool { words(cell).iter().any(|&word| word & BLOCKED == 0) };

        let past =
            |last: Cell, before: Cell| -> Cell { (2 * last.0 - before.0, 2 * last.1 - before.1) };

        let mut out: Vec<GrindRail> = Vec::new();
        for floor in floors {
            let mut left: IndexSet<Cell> = floor
                .iter()
                .filter(|&(_, &word)| word >> KIND_SHIFT == rail)
                .map(|(&cell, _)| cell)
                .collect();
            while !left.is_empty() {
                let Some(mut chain) = chain(&mut left) else {
                    continue;
                };
                if !kinds(past(chain[0], chain[1])).contains(&end) {
                    chain.reverse();
                }
                let n = chain.len();
                let (first, last) = (past(chain[0], chain[1]), past(chain[n - 1], chain[n - 2]));
                let (at_first, at_last) = (kinds(first).contains(&end), kinds(last).contains(&end));
                if at_first && (at_last || open(last)) {
                    let mut cells = vec![first];
                    cells.extend(chain);
                    cells.push(last);
                    out.push(GrindRail {
                        cells,
                        both: at_last,
                    });
                }
            }
        }
        GrindRails::of(out)
    }

    pub fn on(ground: &Ground) -> Option<Rc<dyn Traversal>> {
        if ground.grind_rails.cells.is_empty() {
            None
        } else {
            Some(ground.grind_rails.clone() as Rc<dyn Traversal>)
        }
    }

    /// The ride after the step, `OFF` once it lands, or None where it cannot be taken: the
    /// search's `through`. On a rail the one step is the ride's next cell; onto one only from an
    /// end cell, in line; and never onto or off a rail's cells sideways: the Gym's entrance rail
    /// runs up a corridor of open floor either side, and the floor rail crosses its way out at
    /// (40, 55), where a ride never changes rails.
    pub fn hold(&self, ride: i64, cell: Cell, step: Cell) -> Option<i64> {
        if ride != OFF {
            if self.ahead.get(&(ride, cell)) != Some(&step) {
                return None;
            }
            return Some(if self.ahead.contains_key(&(ride, step)) {
                ride
            } else {
                OFF
            });
        }
        if let Some(&onto) = self.boards.get(&(cell, step)) {
            return Some(onto);
        }
        // A player standing on a rail is being carried along it, one way or the other.
        if self.cells.contains(&step) || self.cells.contains(&cell) {
            None
        } else {
            Some(OFF)
        }
    }
}

impl Traversal for GrindRails {
    fn carries(&self) -> bool {
        true
    }

    fn start(&self, _ground: &Ground, _cell: Cell, _floor: i64) -> Result<State> {
        Ok(vec![OFF])
    }

    fn step(
        &self,
        _ground: &Ground,
        state: &[i64],
        cell: Cell,
        step: Cell,
        _floor: i64,
    ) -> Result<Option<State>> {
        Ok(self.hold(state[0], cell, step).map(|ride| vec![ride]))
    }

    fn lands(
        &self,
        _ground: &Ground,
        _before: &[i64],
        _after: &[i64],
        _cell: Cell,
        step: Cell,
        _floor: i64,
    ) -> Result<Option<i64>> {
        // A rail is on no floor the search walks: its cells are blocked on all of them, and
        // the cell it lands on is stepped onto from any (`Ground.climb`).
        Ok(self.cells.contains(&step).then_some(ANY_FLOOR))
    }

    fn stops(
        &self,
        _ground: &Ground,
        _state: &[i64],
        cell: Cell,
        _floor: i64,
        _ends: &HashSet<Cell>,
    ) -> Result<bool> {
        // A rail's own cells are only ever stood on mid-ride: its end cells are not among them.
        Ok(!self.cells.contains(&cell))
    }
}

/// One chain of `left`'s cells, taken out of it, in order from one end: None where it is not
/// one line, two ends and nothing branching, or is a single cell, which has no line.
fn chain(left: &mut IndexSet<Cell>) -> Option<Vec<Cell>> {
    let mut todo: Vec<Cell> = vec![left.pop().expect("a cell is left")];
    let mut part: HashSet<Cell> = HashSet::new();
    while let Some(cell) = todo.pop() {
        part.insert(cell);
        let near = near(cell, |one| left.contains(&one));
        for one in &near {
            left.shift_remove(one);
        }
        todo.extend(near);
    }
    let beside: HashMap<Cell, Vec<Cell>> = part
        .iter()
        .map(|&cell| (cell, near(cell, |one| part.contains(&one))))
        .collect();
    let mut ends: Vec<Cell> = beside
        .iter()
        .filter(|(_, near)| near.len() == 1)
        .map(|(&cell, _)| cell)
        .collect();
    ends.sort();
    if ends.len() != 2 || beside.values().any(|near| near.len() > 2) {
        return None;
    }
    let mut out: Vec<Cell> = vec![ends[0]];
    while out.len() < part.len() {
        let recent = &out[out.len().saturating_sub(2)..];
        let next = *beside[&out[out.len() - 1]]
            .iter()
            .find(|cell| !recent.contains(cell))
            .expect("a chain goes on to its other end");
        out.push(next);
    }
    Some(out)
}

/// The cells `of` holds beside `cell` or touching it at a corner (Python's `_near(cell, cells)`).
fn near(cell: Cell, of: impl Fn(Cell) -> bool) -> Vec<Cell> {
    let (x, z) = cell;
    pathfind::STEPS
        .iter()
        .chain(pathfind::DIAGONALS.iter())
        .map(|&(_, (dx, dz))| (x + dx, z + dz))
        .filter(|&one| of(one))
        .collect()
}
