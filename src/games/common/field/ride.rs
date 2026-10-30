//! Riding: crossing ground only a Pokémon the player rides crosses, as X's Rhyhorn crosses Route
//! 9's spiky ground, got on and off on the flat ground beside it (spike 238). A traversal
//! (`traversal`): its state is whether the player is riding, 1 or 0.
//!
//! Bound by:
//! - ADR-0006: nothing here names a game's kinds; `gen6.game.Rides` carries them.

use std::collections::HashSet;
use std::rc::Rc;

use crate::core::pathfind::{Cell, State};
use crate::games::common::field::ground::Ground;
use crate::games::common::field::things::Field;
use crate::games::common::field::traversal::Traversal;
use crate::py::Result;

/// Whether the player rides a Pokémon over `ground`, the kinds it crosses and the flat ground
/// beside them: the bike's bits set, which the ride sets too (`Field.on_bike`), on a cell `kind`
/// says is of one of them. On Route 9 the Rhyhorn set +0x06 to 0x21 and +0x07 to 0x03, as the
/// bike does (spike 238).
pub fn riding(
    r#where: &Field,
    kind: &dyn Fn(Cell) -> Result<Option<i64>>,
    ground: &HashSet<i64>,
) -> Result<bool> {
    Ok(r#where.on_bike && kind(r#where.cell)?.is_some_and(|kind| ground.contains(&kind)))
}

/// Whether the player is riding after a step from `cell` to `step` across `ground`, riding
/// before it or not, or None where the step is not taken: into a Pokémon waiting to be ridden
/// they get on (`Ground.mounts`); into a thing it smashes (`Ground.smashed`) and onto the ground
/// only riding crosses (`Ground.rides`) they step only riding, onto that ground from it or from
/// flat ground (`Ground.stops`), and off it only onto flat ground; from flat ground onto any
/// other they get off. On Route 9 the Rhyhorn was got on at (19, 61), ridden onto the flat
/// ground at (20, 61) and on onto the spikes at (21, 61) (spike 238).
pub fn after(ground: &Ground, now: bool, cell: Cell, step: Cell) -> Result<Option<bool>> {
    // shortcut: a way that gets off stays on foot, the Pokémon left where they got off not
    // being one to get on again; no way on Route 9 needs it, follow it if a route does.
    if ground.mounts.contains(&step) {
        return Ok(Some(true));
    }
    if ground.smashed.contains(&step) && !now {
        return Ok(None);
    }
    let held = |kind: Option<i64>, set: &HashSet<i64>| kind.is_some_and(|k| set.contains(&k));
    let onto = ground.kind(step)?;
    if held(onto, &ground.rides) {
        let from = ground.kind(cell)?;
        return Ok(
            if now && (held(from, &ground.rides) || held(from, &ground.stops)) {
                Some(true)
            } else {
                None
            },
        );
    }
    if held(onto, &ground.stops) {
        return Ok(Some(now));
    }
    Ok(if held(ground.kind(cell)?, &ground.rides) {
        None
    } else {
        Some(false)
    })
}

/// Whether the player is riding on each cell of `path` across `ground`, from
/// `Ground.riding` on its first, or None from the first step not taken (`after`).
pub fn rode(ground: &Ground, path: &[Cell]) -> Result<Vec<Option<bool>>> {
    let mut out: Vec<Option<bool>> = vec![Some(ground.riding)];
    for pair in path.windows(2) {
        let next = match out[out.len() - 1] {
            None => None,
            Some(last) => after(ground, last, pair[0], pair[1])?,
        };
        out.push(next);
    }
    Ok(out)
}

/// Riding on a ground that has ground only a ridden Pokémon crosses (`Ground.rides`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Riding;

impl Riding {
    pub fn on(ground: &Ground) -> Option<Rc<dyn Traversal>> {
        if ground.rides.is_empty() {
            None
        } else {
            Some(Rc::new(Riding))
        }
    }
}

impl Traversal for Riding {
    fn start(&self, ground: &Ground, _cell: Cell, _floor: i64) -> Result<State> {
        Ok(vec![ground.riding as i64])
    }

    fn step(
        &self,
        ground: &Ground,
        state: &[i64],
        cell: Cell,
        step: Cell,
        _floor: i64,
    ) -> Result<Option<State>> {
        let now = after(ground, state[0] != 0, cell, step)?;
        Ok(now.map(|now| vec![now as i64]))
    }
}
