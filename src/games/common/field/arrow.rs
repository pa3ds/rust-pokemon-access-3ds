//! Arrow tiles: X's Lysandre Labs' floor, cells that push a player stepping onto them on their own
//! way, a cell at a time over any floor, until the cell on cannot be stood in, where they stop and
//! walk any way; an arrow crossed turns the push its own way (spike 265). A traversal
//! (`traversal`): its state is the way a push carries the player on from the cell they stand in, by
//! its place in `PUSHED`, 0 for none.
//!
//! Bound by:
//! - ADR-0024: an arrow's push is a move of P's way, said as one leg by where it sets the player
//!   down, and no way ends where a push carries the player on.

use std::collections::HashSet;
use std::rc::Rc;

use crate::core::pathfind::{self, Cell, State};
use crate::games::common::field::ground::Ground;
use crate::games::common::field::traversal::Traversal;
use crate::py::Result;

/// The way an arrow's push carries the player on from the cell they stand in, by its place here,
/// as a search's state keeps it: 0 for none.
pub const PUSHED: [Option<&str>; 5] = [
    None,
    Some(pathfind::COMPASS[0]),
    Some(pathfind::COMPASS[1]),
    Some(pathfind::COMPASS[2]),
    Some(pathfind::COMPASS[3]),
];

/// `PUSHED.index(way)`.
fn pushed_index(way: Option<&str>) -> i64 {
    PUSHED
        .iter()
        .position(|known| *known == way)
        .expect("a push is one of the compass's ways") as i64
}

/// A ground's arrow tiles (`Ground.arrows`): a push carries the player on, as a current does
/// (`carry`), and no way ends where one carries them on but at a goal, a way to the arrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arrows;

impl Arrows {
    pub fn on(ground: &Ground) -> Option<Rc<dyn Traversal>> {
        if ground.arrows.is_empty() {
            None
        } else {
            Some(Rc::new(Arrows))
        }
    }
}

impl Traversal for Arrows {
    fn carries(&self) -> bool {
        true
    }

    fn carries_on(&self) -> bool {
        true
    }

    fn start(&self, _ground: &Ground, _cell: Cell, _floor: i64) -> Result<State> {
        Ok(vec![0])
    }

    /// The push on from `step`: an arrow's own from its cell, and on from the cells after it
    /// while each step goes that way; none after a step any other way, which `pathfind.find`
    /// takes only where the cell on cannot be walked.
    fn step(
        &self,
        ground: &Ground,
        state: &[i64],
        cell: Cell,
        step: Cell,
        floor: i64,
    ) -> Result<Option<State>> {
        let way = self.carry(ground, state, cell, floor)?;
        Ok(Some(vec![match way {
            Some(way) if pathfind::towards(cell, step) == way => pushed_index(Some(way)),
            _ => 0,
        }]))
    }

    fn carry(
        &self,
        ground: &Ground,
        state: &[i64],
        cell: Cell,
        _floor: i64,
    ) -> Result<Option<&'static str>> {
        Ok(ground.arrow(cell)?.or(PUSHED[state[0] as usize]))
    }

    fn stops(
        &self,
        ground: &Ground,
        state: &[i64],
        cell: Cell,
        floor: i64,
        ends: &HashSet<Cell>,
    ) -> Result<bool> {
        if ends.contains(&cell) {
            return Ok(true);
        }
        let Some(way) = self.carry(ground, state, cell, floor)? else {
            return Ok(true);
        };
        let (dx, dz) = pathfind::step_of(way)?;
        Ok(!ground.passable((cell.0 + dx, cell.1 + dz))?)
    }
}
