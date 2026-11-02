//! Riding: crossing ground only a Pokémon the player rides crosses, as X's Rhyhorn crosses Route
//! 9's spiky ground and its Mamoswine Route 17's deep snow, got on and off on the flat ground beside
//! them (spikes 238 and 263). A traversal (`traversal`): its state is whether the player is riding,
//! on foot, or on foot having got off on the way (`RIDING`, `ON_FOOT`, `GOT_OFF`).
//!
//! Bound by:
//! - ADR-0006: nothing here names a game's kinds; `gen6.field.landform.Rides` carries them.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::pathfind::{self, Cell, State, Step};
use crate::games::common::field::ground::Ground;
use crate::games::common::field::things::Field;
use crate::games::common::field::traversal::{Name, Traversal};
use crate::py::Result;

/// What the step off a ridden Pokémon is said with, from flat ground onto plain ground, and the
/// rest of the leg it starts: "get off 3 north" (spike 238). The game names no such thing.
pub const GET_OFF: &str = "get off";

/// What the step into a thing the ridden Pokémon smashes is said with, "smash 1 east": on Route 9
/// the Rhyhorn smashes rocks with A (spike 238). The game names no such thing.
pub const SMASH: &str = "smash";

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

/// The riding traversal's states: on foot, riding, and on foot having got off on the way.
pub const ON_FOOT: i64 = 0;
pub const RIDING: i64 = 1;
pub const GOT_OFF: i64 = 2;

/// Whether the player is riding after a step from `cell` to `step` across `ground`, riding
/// before it or not, or None where the step is not taken: into a Pokémon waiting to be ridden
/// they get on (`Ground.mounts`); into a thing it smashes (`Ground.smashed`) and onto the ground
/// only riding crosses (`Ground.rides`) they step only riding, onto that ground from it or from
/// flat ground (`Ground.stops`), and off it only onto flat ground; from flat ground onto any
/// other they get off. Riding, a step is taken only where the Pokémon's body fits (`fits`), and
/// where it no longer fits on flat ground they get off there and walk on (spike 272), `off`, over
/// the rocks the ride smashed: on Route 17 one stands at the foot of the flat ground at (82, 92),
/// across the only way on. On Route 9 the Rhyhorn was got on at (19, 61), ridden onto the flat
/// ground at (20, 61) and on onto the spikes at (21, 61) (spike 238).
pub fn after(
    ground: &Ground,
    now: bool,
    cell: Cell,
    step: Cell,
    off: bool,
) -> Result<Option<bool>> {
    // shortcut: a way that gets off stays on foot, the Pokémon left where they got off not
    // being one to get on again; no way on Route 9 needs it, follow it if a route does.
    if ground.mounts.contains(&step) {
        return Ok(Some(true));
    }
    // shortcut: having got off, every rock is taken for smashed, not only those the ride met;
    // no way on Routes 9 and 17 walks past one it did not (spike 272), track them if one does.
    if ground.smashed.contains(&step) && !now && !off {
        return Ok(None);
    }
    let held = |kind: Option<i64>, set: &HashSet<i64>| kind.is_some_and(|k| set.contains(&k));
    let onto = ground.kind(step)?;
    if held(onto, &ground.rides) {
        let taken =
            now && {
                let from = ground.kind(cell)?;
                held(from, &ground.rides) || held(from, &ground.stops)
            } && fits(ground, step)?;
        return Ok(if taken { Some(true) } else { None });
    }
    if held(onto, &ground.stops) {
        if !now || fits(ground, step)? {
            return Ok(Some(now));
        }
        return Ok(if held(ground.kind(cell)?, &ground.stops) {
            Some(false)
        } else {
            None
        });
    }
    Ok(if held(ground.kind(cell)?, &ground.rides) {
        None
    } else {
        Some(false)
    })
}

fn square(cell: Cell, body: i64) -> Vec<Cell> {
    (-body..body + 1)
        .flat_map(|dx| (-body..body + 1).map(move |dz| (cell.0 + dx, cell.1 + dz)))
        .collect()
}

/// Whether the ridden Pokémon's body fits round `cell`, every cell it reaches open
/// (`Ground.body`): riding, the player's model stands in 3 by 3 cells, and held east along
/// Route 9's spikes beside the wall at row 62 it stopped at (35, 61) (spike 272).
pub fn fits(ground: &Ground, cell: Cell) -> Result<bool> {
    for near in square(cell, ground.body) {
        if !ground.open(near)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Whether a move from `cell` to `step` brings the ridden Pokémon's body up against a thing
/// it smashes (`Ground.smashed`), which is faced and smashed with A there, a cell or more short
/// of the thing's own (spike 272).
pub fn bumps(ground: &Ground, cell: Cell, step: Cell) -> bool {
    // shortcut: asked by `Ground.toll`, which knows no riding, a step on foot up beside a rock
    // costs a smash too; rocks stand on flat ground in three of Route 9's patches.
    if ground.smashed.is_empty() {
        return false;
    }
    let touches = |at: Cell| {
        square(at, ground.body)
            .iter()
            .any(|near| ground.smashed.contains(near))
    };
    touches(step) && !touches(cell)
}

/// Whether the player is riding on each cell of `path` across `ground`, from
/// `Ground.riding` on its first, or None from the first step not taken (`after`): where a way
/// found on other ground, as if nobody stood anywhere, is refused riding on this one
/// (`guide.Guide._up_to`).
pub fn rode(ground: &Ground, path: &[Cell]) -> Result<Vec<Option<bool>>> {
    let mut states: Vec<Option<i64>> = vec![Some(ground.riding as i64)];
    for pair in path.windows(2) {
        let next = match states[states.len() - 1] {
            None => None,
            Some(now) => Riding
                .step(ground, &[now], pair[0], pair[1], 0)?
                .map(|state| state[0]),
        };
        states.push(next);
    }
    Ok(states
        .into_iter()
        .map(|state| state.map(|state| state == RIDING))
        .collect())
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
        let now = after(ground, state[0] == RIDING, cell, step, state[0] == GOT_OFF)?;
        Ok(now.map(|now| {
            vec![if now {
                RIDING
            } else if state[0] == ON_FOOT {
                ON_FOOT
            } else {
                GOT_OFF
            }]
        }))
    }

    /// The steps said with getting on a Pokémon to ride, by its name, "Rhyhorn 1 east", with
    /// smashing a thing in the way riding, "smash 1 east", and with getting off it, the rest of
    /// that leg walked on as it is, "get off 3 north", from the player's first step, riding or
    /// not. Ridden on, the way carries no word, as on water for a player already surfing
    /// (spike 238).
    fn said(
        &self,
        ground: &Ground,
        way: &[Step],
        mine: &[&[i64]],
        name: Name<'_>,
    ) -> Result<IndexMap<i64, String>> {
        let mut out: IndexMap<i64, String> = IndexMap::new();
        for i in 1..way.len() {
            let (cell, step) = (way[i - 1].cell, way[i].cell);
            let (riding, on) = (mine[i - 1][0] == RIDING, mine[i][0] == RIDING);
            if on && !riding {
                out.insert(i as i64, name(&ground.mounts)?);
            } else if on && bumps(ground, cell, step) {
                out.insert(i as i64, SMASH.to_string());
            } else if (riding && !on)
                || (out.get(&(i as i64 - 1)).map(String::as_str) == Some(GET_OFF)
                    && pathfind::towards(way[i - 2].cell, cell) == pathfind::towards(cell, step))
            {
                out.insert(i as i64, GET_OFF.to_string());
            }
        }
        Ok(out)
    }
}
