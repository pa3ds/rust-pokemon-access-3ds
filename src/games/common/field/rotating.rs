//! Rotating doors: Fortree Gym's wooden arms about a pivot, turned a quarter as one is walked into
//! (spike 161). A traversal (`traversal`): its state is how far each door of the ground stands
//! turned, so the search is over every way they can be turned on the way as well, and whether the
//! next can be walked into depends on how each has turned.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::pathfind::{Cell, EAST, NORTH, SOUTH, State, WEST};
use crate::games::common::field::ground::Ground;
use crate::games::common::field::traversal::Traversal;
use crate::py::{self, Error, Result};

/// The ways round, a quarter turn anticlockwise from each to the next.
pub const ANTICLOCKWISE: [&str; 4] = [NORTH, WEST, SOUTH, EAST];

/// `ANTICLOCKWISE.index(way)`, which raises where `way` is none of the four.
#[track_caller]
fn anticlockwise_index(way: &str) -> Result<i64> {
    match ANTICLOCKWISE.iter().position(|&known| known == way) {
        Some(at) => Ok(at as i64),
        None => Err(Error::value("tuple.index(x): x not in tuple")),
    }
}

/// One of Fortree Gym's rotating doors: wooden arms standing on the lines between cells, out
/// from a pivot where four cells meet, turned a quarter about it as one is walked into (spike
/// 161). An arm stops a step across the line it stands on; a step into one turns the door the way
/// the step goes and is taken, but only where every cell the arms sweep through is open, an arm
/// two cells long sweeping the whole quarter between where it is and where it goes, one cell long
/// the cell beside the pivot.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RotatingDoor {
    /// The corner the arms turn about: the cell south-east of it is this cell.
    pub pivot: Cell,
    /// Each arm's way out from the pivot at no turn, and how many cells long it is.
    pub arms: IndexMap<&'static str, i64>,
    /// How many quarter turns anticlockwise it stands at now.
    pub turn: i64,
}

impl RotatingDoor {
    /// A door at no turn; `turn` is set with `..RotatingDoor::new(...)`.
    pub fn new(pivot: Cell, arms: IndexMap<&'static str, i64>) -> Self {
        RotatingDoor {
            pivot,
            arms,
            turn: 0,
        }
    }

    /// The arms as they stand `turn` quarters anticlockwise round.
    pub fn at(&self, turn: i64) -> Result<IndexMap<&'static str, i64>> {
        let mut out = IndexMap::new();
        for (&way, &length) in &self.arms {
            let round = py::modulo(anticlockwise_index(way)? + turn, 4);
            out.insert(ANTICLOCKWISE[round as usize], length);
        }
        Ok(out)
    }

    /// Which arm a step from `cell` to `step` walks into with the door at `turn`, as its way
    /// and the quarter turns the door takes, 1 anticlockwise or -1, or None for no arm.
    pub fn pushed(&self, turn: i64, cell: Cell, step: Cell) -> Result<Option<(&'static str, i64)>> {
        let ((px, pz), arms) = (self.pivot, self.at(turn)?);
        let (dx, dz) = (step.0 - cell.0, step.1 - cell.1);
        let (way, out, push);
        if dx != 0 && cell.0.max(step.0) == px {
            (way, out) = if cell.1 < pz {
                (NORTH, pz - cell.1)
            } else {
                (SOUTH, cell.1 - pz + 1)
            };
            push = if dx > 0 { EAST } else { WEST };
        } else if dz != 0 && cell.1.max(step.1) == pz {
            (way, out) = if cell.0 < px {
                (WEST, px - cell.0)
            } else {
                (EAST, cell.0 - px + 1)
            };
            push = if dz > 0 { SOUTH } else { NORTH };
        } else {
            return Ok(None);
        }
        if arms.get(way).copied().unwrap_or(0) < out {
            return Ok(None);
        }
        let ahead = ANTICLOCKWISE[py::modulo(anticlockwise_index(way)? + 1, 4) as usize];
        Ok(Some((way, if push == ahead { 1 } else { -1 })))
    }

    /// The cells the arms sweep through turning a quarter `by` (1 anticlockwise, -1 not).
    pub fn swept(&self, turn: i64, by: i64) -> Result<Vec<Cell>> {
        let (px, pz) = self.pivot;
        let mut out: Vec<Cell> = Vec::new();
        for (way, length) in self.at(turn)? {
            let goes = ANTICLOCKWISE[py::modulo(anticlockwise_index(way)? + by, 4) as usize];
            let east = way == EAST || goes == EAST;
            let south = way == SOUTH || goes == SOUTH;
            let xs = if east { [px, px + 1] } else { [px - 1, px - 2] };
            let zs = if south {
                [pz, pz + 1]
            } else {
                [pz - 1, pz - 2]
            };
            for &x in py::slice(&xs, 0, length) {
                for &z in py::slice(&zs, 0, length) {
                    out.push((x, z));
                }
            }
        }
        Ok(out)
    }
}

/// A step across a door's line -> the door's place in `doors`.
pub type Crossings = HashMap<(Cell, Cell), i64>;

thread_local! {
    static CROSSINGS: RefCell<HashMap<Vec<Cell>, Rc<Crossings>>> = RefCell::new(HashMap::new());
}

/// Every step, each way, across the lines out from each rotating door's pivot that an arm
/// two cells long can stand on, the only steps a door can stop or be turned by, with the door's
/// place in `pivots`. Asked of every step a search takes, so worked out once a set of doors.
fn crossings_of(pivots: &[Cell]) -> Rc<Crossings> {
    if let Some(known) = CROSSINGS.with(|cache| cache.borrow().get(pivots).cloned()) {
        return known;
    }
    let mut out: Crossings = HashMap::new();
    for (i, &(px, pz)) in pivots.iter().enumerate() {
        for k in [1, 2] {
            for (a, b) in [
                ((px - 1, pz - k), (px, pz - k)),
                ((px - 1, pz + k - 1), (px, pz + k - 1)),
                ((px - k, pz - 1), (px - k, pz)),
                ((px + k - 1, pz - 1), (px + k - 1, pz)),
            ] {
                out.insert((a, b), i as i64);
                out.insert((b, a), i as i64);
            }
        }
    }
    let out = Rc::new(out);
    CROSSINGS.with(|cache| cache.borrow_mut().insert(pivots.to_vec(), out.clone()));
    out
}

/// A ground's rotating doors, walked through by turning them: a step into an arm turns its
/// door (`step`).
#[derive(Debug, Clone, PartialEq)]
pub struct Doors {
    pub doors: Vec<RotatingDoor>,
    /// Every step a door's arm can stand across, with the door's place in `doors` (`_crossings`):
    /// asked of every step a search takes.
    pub crossings: Rc<Crossings>,
}

impl Doors {
    /// The ground's doors, or None where it has none. Python's `on`, typed as the doors here so
    /// the guide can ask `pushes`; `TRAVERSALS` takes it as a traversal (`traversal`).
    pub fn of(ground: &Ground) -> Option<Doors> {
        if ground.doors.is_empty() {
            return None;
        }
        let pivots: Vec<Cell> = ground.doors.iter().map(|door| door.pivot).collect();
        Some(Doors {
            doors: ground.doors.clone(),
            crossings: crossings_of(&pivots),
        })
    }

    pub fn on(ground: &Ground) -> Option<Rc<dyn Traversal>> {
        Doors::of(ground).map(|doors| Rc::new(doors) as Rc<dyn Traversal>)
    }

    /// Where along `path` a step walks into a door's arm and turns it, by the index of the
    /// cell it steps to.
    pub fn pushes(&self, ground: &Ground, path: &[Cell]) -> Result<IndexSet<i64>> {
        let mut out: IndexSet<i64> = IndexSet::new();
        let mut turns = self.start(ground, path[0], 0)?;
        for i in 1..path.len() {
            let after = self.step(ground, &turns, path[i - 1], path[i], 0)?;
            if let Some(after) = after
                && after != turns
            {
                out.insert(i as i64);
                turns = after;
            }
        }
        Ok(out)
    }
}

impl Traversal for Doors {
    /// How far each door stands turned now, in `doors`' order.
    fn start(&self, _ground: &Ground, _cell: Cell, _floor: i64) -> Result<State> {
        Ok(self.doors.iter().map(|door| door.turn).collect())
    }

    /// The doors' turns after the step: as they were where it walks into no arm, the door
    /// turned a quarter where it walks into one and every cell its arms sweep is open on
    /// `ground`, and None where they are not, and the step is not taken.
    fn step(
        &self,
        ground: &Ground,
        state: &[i64],
        cell: Cell,
        step: Cell,
        _floor: i64,
    ) -> Result<Option<State>> {
        let Some(&i) = self.crossings.get(&(cell, step)) else {
            return Ok(Some(state.to_vec()));
        };
        let door = usize::try_from(i)
            .ok()
            .and_then(|i| self.doors.get(i))
            .ok_or_else(|| Error::index("tuple index out of range"))?;
        let at = i as usize;
        let turned = state
            .get(at)
            .copied()
            .ok_or_else(|| Error::index("tuple index out of range"))?;
        let Some(pushed) = door.pushed(turned, cell, step)? else {
            return Ok(Some(state.to_vec()));
        };
        for swept in door.swept(turned, pushed.1)? {
            if !(ground.open(swept)? || swept == cell) {
                return Ok(None);
            }
        }
        let mut after = state[..at].to_vec();
        after.push(py::modulo(turned + pushed.1, 4));
        after.extend_from_slice(py::slice_from(state, i + 1));
        Ok(Some(after))
    }
}
