//! Teleporters within a zone: X's Laverre City Gym's panels, triggers that put the player on a
//! cell and floor of the same zone, most often the partner's panel, which does nothing until they
//! step off it and back on (spike 258). A traversal (`traversal`): its state is whether the player
//! stepped onto the cell they stand in, `STEPPED`, or a teleport put them there, `LANDED`.
//!
//! Bound by:
//! - ADR-0024: a teleporter within its zone is walked onto and carries the player, a move of P's way
//!   and of ctrl+P's walk out alike, and the partner it puts them on does nothing until stepped off.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::pathfind::{Cell, Leg, State, Step};
use crate::games::common::field::ground::Ground;
use crate::games::common::field::things::TELEPORTER;
use crate::games::common::field::traversal::{Name, Traversal};
use crate::py::Result;

/// Whether the player walked onto the cell they stand in, where a teleporter fires, or a
/// teleport put them there, where it does nothing until they step off it.
pub const STEPPED: i64 = 0;
pub const LANDED: i64 = 1;

/// Each teleporter's cell and the floor it is on, with where it puts the player and on what
/// floor (`Ground.teleports`).
pub type Panels = IndexMap<(Cell, i64), (Cell, i64)>;

/// A ground's teleporters: one stepped onto carries the player where it puts them, the one
/// move from it (`leap`), and no way ends on one but at a goal, a way to it.
#[derive(Debug, Clone, PartialEq)]
pub struct Teleporters {
    pub panels: Panels,
}

impl Teleporters {
    pub fn on(ground: &Ground) -> Option<Rc<dyn Traversal>> {
        if ground.teleports.is_empty() {
            None
        } else {
            Some(Rc::new(Teleporters {
                panels: ground.teleports.clone(),
            }))
        }
    }

    /// Where the teleporter in `cell` on `floor` puts a player in `state`, or None where
    /// there is none, or it was what put them there.
    pub fn fires(&self, state: i64, cell: Cell, floor: i64) -> Option<(Cell, i64)> {
        if state != STEPPED {
            return None;
        }
        self.panels.get(&(cell, floor)).copied()
    }
}

impl Traversal for Teleporters {
    fn carries(&self) -> bool {
        true
    }

    /// A teleport crosses Laverre Gym in one move, so the distance to the goals is no measure
    /// of the way left.
    fn far(&self) -> bool {
        true
    }

    fn start(&self, _ground: &Ground, cell: Cell, floor: i64) -> Result<State> {
        Ok(vec![if self.panels.contains_key(&(cell, floor)) {
            LANDED
        } else {
            STEPPED
        }])
    }

    fn step(
        &self,
        _ground: &Ground,
        state: &[i64],
        cell: Cell,
        step: Cell,
        floor: i64,
    ) -> Result<Option<State>> {
        let sent = self.fires(state[0], cell, floor);
        if let Some(sent) = sent
            && sent.0 != step
        {
            return Ok(None);
        }
        // A teleport puts the player on its partner's panel, which does nothing until they step
        // off it and back on.
        Ok(Some(vec![match sent {
            Some(sent) if self.panels.contains_key(&(step, sent.1)) => LANDED,
            _ => STEPPED,
        }]))
    }

    fn lands(
        &self,
        _ground: &Ground,
        before: &[i64],
        _after: &[i64],
        cell: Cell,
        _step: Cell,
        floor: i64,
    ) -> Result<Option<i64>> {
        Ok(self.fires(before[0], cell, floor).map(|sent| sent.1))
    }

    fn stops(
        &self,
        _ground: &Ground,
        state: &[i64],
        cell: Cell,
        floor: i64,
        ends: &HashSet<Cell>,
    ) -> Result<bool> {
        Ok(ends.contains(&cell) || self.fires(state[0], cell, floor).is_none())
    }

    fn leap(&self, _ground: &Ground, state: &[i64], cell: Cell, floor: i64) -> Option<Cell> {
        self.fires(state[0], cell, floor).map(|sent| sent.0)
    }

    /// Each step that is a teleport, by the index of the cell it puts the player on: by place,
    /// not by its cells, as a way through Laverre Gym walks (1, 1) on two floors.
    fn said(
        &self,
        ground: &Ground,
        way: &[Step],
        mine: &[&[i64]],
        _name: Name<'_>,
    ) -> Result<IndexMap<i64, String>> {
        let floors = ground.walked(way)?;
        Ok((1..way.len())
            .filter(|&i| {
                self.fires(mine[i - 1][0], way[i - 1].cell, floors[i - 1])
                    .is_some_and(|sent| sent.0 == way[i].cell)
            })
            .map(|i| (i as i64, TELEPORTER.to_string()))
            .collect())
    }

    /// `legs` with each teleport a leg of its own (`Teleported`, a `Leg` whose `teleported` is
    /// set): "6 north, teleporter, 2 east". Nothing is said as the player stands on the
    /// teleporter: the game is moving them, and the leg after it is said where it puts them
    /// (spike 258).
    fn legs(
        &self,
        _ground: &Ground,
        _way: &[Step],
        _mine: &[&[i64]],
        legs: Vec<Leg>,
    ) -> Result<Vec<Leg>> {
        Ok(legs
            .into_iter()
            .map(|leg| {
                if leg.how == TELEPORTER {
                    Leg {
                        direction: leg.direction,
                        steps: leg.steps,
                        end: leg.end,
                        how: leg.how,
                        held: None,
                        vined: false,
                        teleported: true,
                    }
                } else {
                    leg
                }
            })
            .collect())
    }
}
