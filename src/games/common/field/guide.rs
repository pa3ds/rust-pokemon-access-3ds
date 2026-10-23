//! P: the way on foot to whatever the navigation keys are on, said as its legs, and guiding,
//! the player followed along it (GLOSSARY: Pathfinding, Guiding).
//!
//! Bound by:
//! - ADR-0024: P's way is said as legs and guided turn by turn; a player who leaves it is given a
//!   new one quietly.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::config::Navigation;
use crate::core::pathfind::{self, Cell, Leg};
use crate::core::words::own;
use crate::games::common::field::ground::{
    CUT, Ground, IceRoom, Of, ROCK_SMASH, SOLID, STRENGTH, SURF, WATERFALL, WHOLE,
};
use crate::games::common::field::things::{self as field, Field, Interactable};
use crate::py::{self, Error, Log, Result};

/// How far from a room of thin ice's exits a way may stop short and be taken over the ice
/// instead: Sootopolis Gym's shut corridors stop it two cells up from the first room's exit, one up
/// from the middle's, and on the top's (spike 167).
pub const ICE_REACH: i64 = 3;

/// What a step into one of Fortree Gym's rotating doors is said with, which turns it a quarter as
/// the player walks on: "push 1 west" (spike 161). The game names no such thing.
pub const PUSH: &str = "push";

/// What a leg a current in the sea carries the player along is said with, a run of them as one
/// by where it leaves the player: "Surf 3 south, ride the current 28 west 4 north" (spike 208); Orion
/// asked for "ride the current" over "current", which sounded wrong (2026-10-13). The game names no
/// such thing.
pub const CURRENT: &str = "ride the current";

/// What the step off a ridden Pokémon is said with, from flat ground onto plain ground, and the
/// rest of the leg it starts: "get off 3 north" (spike 238). The game names no such thing.
pub const GET_OFF: &str = "get off";

/// What the step into a thing the ridden Pokémon smashes is said with, "smash 1 east": on Route 9
/// the Rhyhorn smashes rocks with A (spike 238). The game names no such thing.
pub const SMASH: &str = "smash";

/// What a leg is said with where the ROM's move names cannot be read: pa3ds's own word for where
/// the leg goes.
pub const OWN_MOVE_WORDS: [(i64, &str); 5] = [
    (SURF, "water"),
    (CUT, "bush"),
    (STRENGTH, "boulder"),
    (ROCK_SMASH, "rock"),
    (WATERFALL, "waterfall"),
];

/// What a leg up a sandy slope is said with where the ROM's name for the Mach Bike cannot be
/// read.
pub const OWN_SLOPE_WORD: &str = "slope";

/// What a hop up a bumpy slope is said with where the ROM's name for the Acro Bike cannot be
/// read.
pub const OWN_HOP_WORD: &str = "bumpy slope";

/// Degrees the camera may be turned off a way of Lumiose's streets and still be said to look
/// along it. A tap of left or right leaves it 6 to 70 degrees round, and up then walks the player
/// across the street at that angle; a hold settles it within a degree of the way the player was
/// turned to. The corridor's frame is the line's where the player stands, which on a curving line
/// is up to 15 degrees from the ways the game turns the player to, so less would call a settled
/// camera angled (spike 105).
pub const ANGLED: f64 = 20.0;

/// Degrees the camera may drift and still be the same: walking a street eases it by a few.
pub const STILL: f64 = 0.5;

/// Seconds the camera must stay still to have stopped turning. A hold turns it about 150 degrees
/// a second and eases it to a stop over a fifth of a second (spike 105).
pub const SETTLE: f64 = 0.3;

/// Degrees the camera must have turned since the way was last said for it to be said again.
pub const MOVED: f64 = 3.0;

/// How far off a cell's centre the player may stand and be lined up with a leg, in cells: half
/// the seventh of a cell the circle pad moves them a frame. Off by 0.18 a step past a wall's corner
/// was refused, and off by 0.01 it went (movie38, spike 202); between the two is not known.
pub const LINED_UP: f64 = 0.07;

/// Seconds the player must stand still off a leg's line before being told which way to line up:
/// a tap of the circle pad moves them for about a fifth of a second (movie38, spike 202).
pub const STOPPED: f64 = 0.25;

/// What an item is kept by: `Interactable::key`.
pub type Key = (String, String, i64, Option<i64>);

/// A path being walked, and how far along it the player has got.
///
/// Shared between the navigator and the call that is holding it (`Guide::to`'s `held`), as
/// one object in Python, so it is an `Rc<RefCell<Guidance>>` where it is passed about.
#[derive(Debug, Clone, PartialEq)]
pub struct Guidance {
    /// What the path leads to, by which it is found again when the path has to be.
    pub key: Key,
    pub name: String,
    /// The field's `visit` when the path was found: a warp changes it.
    pub visit: Option<(i64, i64)>,
    pub path: Vec<Cell>,
    pub legs: Vec<Leg>,
    /// Each cell of the path and how far along it is, the first time where it is crossed twice
    /// (`at`).
    pub along: HashMap<Cell, i64>,
    /// Where the player was last seen.
    pub cell: Cell,
    /// The leg last said: the whole path is said first, so the first leg is said already.
    /// Python's default is 0.
    pub leg: i64,
    /// On a rail, where the camera looked when the way was last said (`Field.bearing`). Python's
    /// default is None.
    pub camera: Option<f64>,
    /// Where it looked when it last moved, and `since` when that was: it has stopped turning
    /// once it has stayed there `SETTLE` seconds. Python's default is None.
    pub seen: Option<f64>,
    /// Python's default is 0.0.
    pub since: f64,
    /// Whether the way was found with HMs, as it is found again whenever it has to be. Python's
    /// default is False.
    pub hm: bool,
    /// The height its decks were walked at (`Ground.decks`), or None at the player's own. Python's
    /// default is None.
    pub height: Option<f64>,
    /// Whether it ends on the last tile of a room of thin ice, which clears the room: said as
    /// "and the ice is cleared", and "ice cleared" on arriving (spike 167). Python's default is
    /// False.
    pub ice: bool,
    /// Whether it ends short of the thing, as near as the player can get, where they hear that
    /// they can't continue. Python's default is False.
    pub short: bool,
    /// What stops it where it ends short, said after "can't continue": who stands in the way,
    /// ", Team Magma Grunt in the way", or under the water where to dive again, ", surface and go
    /// to the dive spot 29 east 16 south"; "" where it is neither. Python's default is "".
    pub blocker: String,
    /// Whether a key asked for it. One found for the player unasked (`Navigator.found`) is found
    /// again quietly when they leave it: walking away from a hidden Pokémon the DexNav found, a new
    /// way was said at every step (movie26's replay, 2026-10-06). Python's default is True.
    pub asked: bool,
    /// For a big boulder, the way it is to be pushed, whose side the path ends on and is found
    /// again to.
    pub push: Option<&'static str>,
    /// Where in the world the player was last seen (`Field.place`), and `still` since when.
    pub place: Option<(f64, f64)>,
    pub still: f64,
    /// The place last told whether it is lined up with the next leg, so a player standing still
    /// hears it once.
    pub judged: Option<(f64, f64)>,
    /// The way the player was last told to step to line up, `west` for "a little west", until
    /// they are.
    pub nudge: Option<&'static str>,
}

impl Guidance {
    /// How far along the path `cell` is: where the path crosses it more than once, as a way over
    /// Sootopolis Gym's thin ice crosses its floor, the first time from the leg being walked on.
    pub fn at(&self, cell: Cell) -> Result<Option<i64>> {
        let start = if self.leg != 0 {
            let at = self.leg - 1;
            usize::try_from(at)
                .ok()
                .and_then(|at| self.legs.get(at))
                .ok_or_else(|| Error::index("list index out of range"))?
                .end
        } else {
            0
        };
        let later = (start..self.path.len() as i64).find(|&i| self.path[i as usize] == cell);
        Ok(match later {
            None => self.along.get(&cell).copied(),
            Some(later) => Some(later),
        })
    }
}

/// What the navigator reads the map through: ORAS's `field.FieldReader`, or Sun's reader.
pub trait FieldSource {
    /// `visit`, a property in Python.
    fn visit(&self) -> Option<(i64, i64)>;

    fn poll(&self) -> Result<Option<Field>>;

    fn interactables(&self, r#where: &Field) -> Result<Field>;

    /// The ground under `where`, its decks walked at `height`, or at the player's own.
    fn ground(&self, r#where: &Field, height: Option<f64>) -> Result<Ground>;
}

/// `Callable[[], float]`: `time.monotonic` unless a test or a tape says otherwise.
pub type Clock = Rc<dyn Fn() -> f64>;

/// Where a way falls short by a room of thin ice not yet cleared, the way onto it and over every
/// whole tile once, ending on one of its exits, which clears it; None where there is no such
/// room, or no such way. Each of Sootopolis Gym's corridors up to Wallace is shut until the room
/// below it is cleared, and P to him stopped there, "26 north, then can't continue" (spike 167).
/// From inside the room the way goes on from where the player stands; from outside it, onto
/// whichever edge tile it can be crossed from, nearest first.
fn over_ice(
    r#where: &Field,
    ground: &Ground,
    path: &Option<Vec<Cell>>,
    heading: &str,
) -> Result<Option<Vec<Cell>>> {
    let end = match path {
        None => r#where.cell,
        Some(path) => *path
            .last()
            .ok_or_else(|| Error::index("tuple index out of range"))?,
    };
    let rooms: Vec<&IceRoom> = ground.ice.iter().filter(|room| !room.cleared).collect();
    let inside = rooms
        .iter()
        .find(|room| room.cells().contains_key(&r#where.cell))
        .copied();

    let off = |room: &IceRoom| -> Result<i64> {
        room.exits
            .iter()
            .map(|&(x, z)| (end.0 - x).abs() + (end.1 - z).abs())
            .min()
            .ok_or_else(|| Error::value("min() iterable argument is empty"))
    };

    let room = match inside {
        Some(room) => Some(room),
        None => {
            let mut best: Option<(&IceRoom, i64)> = None;
            for &room in &rooms {
                let away = off(room)?;
                if best.is_none_or(|(_, least)| away < least) {
                    best = Some((room, away));
                }
            }
            best.map(|(room, _)| room)
        }
    };
    let Some(room) = room else {
        return Ok(None);
    };
    if inside.is_none() && off(room)? > ICE_REACH {
        return Ok(None);
    }
    let cells = room.cells();
    let mut once: HashSet<Cell> = HashSet::new();
    for (&cell, &state) in &cells {
        if state == WHOLE && ground.open_at(cell, Some(room.height))? {
            once.insert(cell);
        }
    }
    let mut free: HashSet<Cell> = HashSet::new();
    for (&cell, &state) in &cells {
        if state == SOLID && ground.open_at(cell, Some(room.height))? {
            free.insert(cell);
        }
    }
    if inside.is_some() {
        return Ok(pathfind::cover(
            r#where.cell,
            once.iter().copied(),
            free.iter().copied(),
            room.exits.iter().copied(),
        ));
    }
    let mut edge: Vec<Cell> = Vec::new();
    for &cell in &once {
        let mut any = false;
        for (_, (dx, dz)) in pathfind::STEPS {
            let near = (cell.0 + dx, cell.1 + dz);
            if !cells.contains_key(&near) && ground.passable_at(near, Some(room.height))? {
                any = true;
                break;
            }
        }
        if any {
            edge.push(cell);
        }
    }
    edge.sort_by_key(|cell| {
        (
            (cell.0 - r#where.cell.0).abs() + (cell.1 - r#where.cell.1).abs(),
            *cell,
        )
    });
    for entry in edge {
        let aim = Interactable {
            height: Some(room.height),
            ..Interactable::new("exits", "ice", entry)
        };
        let goals: IndexMap<Cell, Option<&'static str>> = IndexMap::from([(entry, None)]);
        let onto = ground.find(
            r#where.cell,
            &goals,
            heading,
            Some(&aim),
            false,
            Some(&|cell: Cell| Ok(!cells.contains_key(&cell) && ground.passable(cell)?)),
        )?;
        if !reached(ground, &aim, &goals, &onto)? {
            continue;
        }
        let mut onto = onto.expect("a way that reached the entry was found");
        let across = pathfind::cover(
            entry,
            once.iter().copied(),
            free.iter().copied(),
            room.exits.iter().copied(),
        );
        if let Some(across) = across {
            onto.extend_from_slice(py::slice_from(&across, 1));
            return Ok(Some(onto));
        }
    }
    Ok(None)
}

/// Under the water, where the thing is in a pocket walled off from the player's: the way to the
/// nearest deep water the player can reach whose sea above reaches a dive spot over the thing's
/// pocket, to rise to the surface through, and what to do there, said after "can't continue": go
/// to that spot, the nearest, by its bearing from there, which is where the list puts it once they
/// have risen. None above the water, or where no light joins the two. Under Route 126 the player
/// dived into a pocket with no way to Sootopolis City's (omega_ruby_route-126-dive.ctm, spike
/// 165).
fn dive_again(
    start: Cell,
    ground: &Ground,
    goals: &IndexMap<Cell, Option<&'static str>>,
    heading: &str,
) -> Result<Option<(Vec<Cell>, String)>> {
    let r#box = ground.r#box(start, goals.keys().copied())?;
    let theirs: HashSet<Cell> = pocket(goals.keys().copied(), ground, &r#box)?
        .into_iter()
        .filter(|cell| ground.surfaces.contains_key(cell))
        .collect();
    if theirs.is_empty() {
        return Ok(None);
    }
    let aim: IndexMap<Cell, Option<&'static str>> = ground
        .surfaces
        .iter()
        .filter(|(_, joined)| joined.iter().any(|cell| theirs.contains(cell)))
        .map(|(&cell, _)| (cell, None))
        .collect();
    let walkable = |cell: Cell| ground.passable(cell);
    let ledge = |cell: Cell| ground.ledge(cell);
    let toll = |cell: Cell, step: Cell| ground.toll(cell, step);
    let up = pathfind::find(
        start,
        &aim,
        &walkable,
        r#box,
        &pathfind::Options {
            heading,
            ledge: Some(&ledge),
            toll: Some(&toll),
            ..pathfind::Options::default()
        },
    )?;
    let Some(up) = up else {
        return Ok(None);
    };
    let rise = *up
        .last()
        .ok_or_else(|| Error::index("tuple index out of range"))?;
    // ponytail: the spot nearest as the crow flies, the sea's own way there not found; a way with
    // Surf from the light would say it.
    let joined = ground
        .surfaces
        .get(&rise)
        .ok_or_else(|| Error::key(format!("({}, {})", rise.0, rise.1)))?;
    let down = joined
        .iter()
        .filter(|cell| theirs.contains(cell))
        .min_by_key(|cell| ((cell.0 - rise.0).abs() + (cell.1 - rise.1).abs(), **cell))
        .copied()
        .ok_or_else(|| Error::value("min() iterable argument is empty"))?;
    Ok(Some((
        up,
        format!(
            ", {}",
            own("p-surface", &[("bearing", &field::bearing(rise, down))])
        ),
    )))
}

/// Every open cell inside `box` joined side to side to any of `cells`, and the cells.
fn pocket(
    cells: impl IntoIterator<Item = Cell>,
    ground: &Ground,
    r#box: &pathfind::Box,
) -> Result<HashSet<Cell>> {
    let mut seen: HashSet<Cell> = cells.into_iter().collect();
    let mut todo: Vec<Cell> = seen.iter().copied().collect();
    while let Some((x, z)) = todo.pop() {
        for (_, (dx, dz)) in pathfind::STEPS {
            let near = (x + dx, z + dz);
            if !seen.contains(&near) && r#box.holds(near) && ground.open(near)? {
                seen.insert(near);
                todo.push(near);
            }
        }
    }
    Ok(seen)
}

/// `up_to`'s answer: the ground, goals, path, the height its decks are walked at, and who stands
/// in the way.
type Past = (
    Ground,
    IndexMap<Cell, Option<&'static str>>,
    Vec<Cell>,
    Option<f64>,
    Option<Interactable>,
);

/// The way to step to line up with the leg from `path[at]`, where the player stands off its
/// line and would catch a wall's corner on it: `west`, said "a little west" (`nudged`). None
/// where they are lined up,
/// or nothing beside the leg is in the way: off the line in open ground, the step is taken.
///
/// In Seafloor Cavern the way north was a gap a cell wide between two pillars, and the player,
/// told "2 north" there, stood 0.18 to 0.46 of a cell east or west of its line and was refused it
/// for a hundred seconds (movie38, spike 202).
fn nudge(
    ground: &Ground,
    place: Option<(f64, f64)>,
    path: &[Cell],
    at: i64,
) -> Result<Option<&'static str>> {
    let Some(place) = place else {
        return Ok(None);
    };
    if at + 1 >= path.len() as i64 {
        return Ok(None);
    }
    let cell = path[at as usize];
    let next = path[at as usize + 1];
    let (dx, dz) = (next.0 - cell.0, next.1 - cell.1);
    if dx.abs() + dz.abs() != 1 {
        return Ok(None);
    }
    let off = if dx == 0 {
        place.0 - cell.0 as f64
    } else {
        place.1 - cell.1 as f64
    };
    if off.abs() <= LINED_UP {
        return Ok(None);
    }
    let side = if off > 0.0 { 1 } else { -1 };
    let mut ahead: Vec<Cell> = Vec::new();
    for &step in &path[at as usize + 1..] {
        let last = ahead.last().copied().unwrap_or(cell);
        if (step.0 - last.0, step.1 - last.1) != (dx, dz) {
            break;
        }
        ahead.push(step);
    }
    let mut all = true;
    for &(x, z) in &ahead {
        let beside = if dx == 0 {
            (x + side, z)
        } else {
            (x, z + side)
        };
        if !ground.open(beside)? {
            all = false;
            break;
        }
    }
    if all {
        return Ok(None);
    }
    Ok(Some(if dx == 0 {
        if side > 0 {
            pathfind::WEST
        } else {
            pathfind::EAST
        }
    } else if side > 0 {
        pathfind::NORTH
    } else {
        pathfind::SOUTH
    }))
}

/// The way to step, as it is said: "a little west".
fn nudged(nudge: &str) -> String {
    own("p-nudge", &[("way", &pathfind::way(nudge))])
}

/// Whether a player told `nudge` now stands within `LINED_UP` of the line it was for.
fn lined(nudge: &str, place: (f64, f64), cell: Cell) -> bool {
    if nudge == pathfind::WEST || nudge == pathfind::EAST {
        (place.0 - cell.0 as f64).abs() <= LINED_UP
    } else {
        (place.1 - cell.1 as f64).abs() <= LINED_UP
    }
}

/// The ways a big boulder is to be pushed, whose sides a way to it ends on: `push` where it
/// can be pushed so, otherwise every way it can be (`Interactable.pushes`); none for anything
/// else, and for a boulder no push moves.
fn sides(item: &Interactable, push: Option<&str>) -> HashSet<&'static str> {
    let ways: HashSet<&'static str> = item.pushes.iter().flatten().map(|&(way, _)| way).collect();
    match push.and_then(|push| ways.get(push).copied()) {
        Some(push) => HashSet::from([push]),
        None => ways,
    }
}

/// `goals` kept to those faced the ways in `sides`: beside a boulder, facing it is the way it
/// is pushed from there (Orion, 2026-10-11: P left him south of a boulder that moves only
/// east). All of them where none of those is open.
fn aimed(
    goals: IndexMap<Cell, Option<&'static str>>,
    sides: &HashSet<&'static str>,
) -> IndexMap<Cell, Option<&'static str>> {
    let kept: IndexMap<Cell, Option<&'static str>> = goals
        .iter()
        .filter(|(_, way)| way.is_some_and(|way| sides.contains(way)))
        .map(|(&cell, &way)| (cell, way))
        .collect();
    if kept.is_empty() { goals } else { kept }
}

/// Where a way to `item` across `ground` ends: its cells, or beside it, and beside a big
/// boulder a side it is pushed from, `push` where it can be pushed so (`_sides`, `_aimed`).
fn aims(
    ground: &Ground,
    item: &Interactable,
    push: Option<&str>,
) -> Result<IndexMap<Cell, Option<&'static str>>> {
    Ok(aimed(ground.goals(item)?, &sides(item, push)))
}

/// Whether a way found ends at the thing: on one of `goals`, on the floor it was walked to
/// there (`Ground.walked`, `Ground.ends_at`). `path[-1]` raises on an empty path as Python's
/// index does.
fn reached(
    ground: &Ground,
    item: &Interactable,
    goals: &IndexMap<Cell, Option<&'static str>>,
    path: &Option<Vec<Cell>>,
) -> Result<bool> {
    let Some(path) = path else {
        return Ok(false);
    };
    let end = *path
        .last()
        .ok_or_else(|| Error::index("tuple index out of range"))?;
    if !goals.contains_key(&end) {
        return Ok(false);
    }
    let walked = ground.walked(path)?;
    ground.ends_at(item, end, walked[walked.len() - 1])
}

/// The way P finds to a thing, before it is said (`Guide._way`).
struct Way {
    /// The ground it was found on.
    ground: Ground,
    goals: IndexMap<Cell, Option<&'static str>>,
    path: Option<Vec<Cell>>,
    /// The height that ground's decks are walked at.
    height: Option<f64>,
    /// Whether it ends short of the thing.
    short: bool,
    /// Whether it ends clearing a room of thin ice, the way on.
    ice: bool,
    /// The heights a way past somebody standing in it is looked for at (`Guide._up_to`).
    levels: Vec<Option<f64>>,
}

/// How far along `path` the cell nearest `cell` is, when one is within `slack` cells of it
/// either way, the furthest along of the nearest; or None.
fn near(path: &[Cell], cell: Cell, slack: i64) -> Option<i64> {
    let best = path
        .iter()
        .enumerate()
        .map(|(at, &(x, z))| ((x - cell.0).abs().max((z - cell.1).abs()), -(at as i64)))
        .min();
    match best {
        Some(best) if best.0 <= slack => Some(-best.1),
        _ => None,
    }
}

/// The steps of `path` said with getting on a Pokémon to ride, by its name, "Rhyhorn 1
/// east", with smashing a thing in the way riding, "smash 1 east", and with getting off it, the
/// rest of that leg walked on as it is, "get off 3 north", from the player's first step, riding
/// or not (`Ground.rode`). Ridden on, the way carries no word, as on water for a player already
/// surfing (spike 238).
fn rides(ground: &Ground, path: &[Cell], mount: &str) -> Result<HashMap<(Cell, Cell), String>> {
    let mut out: HashMap<(Cell, Cell), String> = HashMap::new();
    let mut last: Option<(Cell, Cell)> = None;
    let rode = ground.rode(path)?;
    for (i, pair) in path.windows(2).enumerate() {
        let (cell, step) = (pair[0], pair[1]);
        let (riding, on) = (rode[i] == Some(true), rode[i + 1] == Some(true));
        if on && !riding {
            out.insert((cell, step), mount.to_string());
        } else if ground.smashed.contains(&step) {
            out.insert((cell, step), SMASH.to_string());
        } else if (riding && !on)
            || last.is_some_and(|last| {
                out.get(&last).map(String::as_str) == Some(GET_OFF)
                    && pathfind::towards(last.0, last.1) == pathfind::towards(cell, step)
            })
        {
            out.insert((cell, step), GET_OFF.to_string());
        }
        last = Some((cell, step));
    }
    Ok(out)
}

/// What is left of the way from its `at`th cell, said from where the player stands.
fn left(guidance: &Guidance, at: i64, r#where: &Field) -> Result<String> {
    let leg = guidance
        .legs
        .iter()
        .position(|leg| leg.end > at)
        .expect("a leg ends past the cell");
    let now = &guidance.legs[leg];
    let mut legs = vec![Leg {
        steps: now.end - at,
        ..now.clone()
    }];
    legs.extend_from_slice(&guidance.legs[leg + 1..]);
    spoken(&legs, r#where)
}

/// Legs as the player's keys walk them: by the compass on the grid, and on a rail, where the
/// keys work from the camera behind the player, from the way it looks (spike 105). A run of legs
/// a current carries the player along is said as one, by where it leaves them (`CURRENT`).
fn spoken(legs: &[Leg], r#where: &Field) -> Result<String> {
    if let Some(heading) = r#where.heading {
        let angled = match r#where.bearing {
            Some(bearing) => pathfind::off(bearing, heading)?.abs() > ANGLED,
            None => false,
        };
        return pathfind::turned(legs, heading, angled);
    }
    let mut parts: Vec<String> = Vec::new();
    let mut drift = (0, 0);
    for leg in legs.iter().map(Some).chain([None]) {
        if let Some(leg) = leg
            && leg.how == CURRENT
        {
            let (dx, dz) = pathfind::STEPS
                .iter()
                .find(|(known, _)| *known == leg.direction)
                .map(|&(_, step)| step)
                .ok_or_else(|| Error::key(py::repr(leg.direction)))?;
            drift = (drift.0 + dx * leg.steps, drift.1 + dz * leg.steps);
            continue;
        }
        if drift != (0, 0) {
            parts.push(own(
                "p-current",
                &[("bearing", &field::bearing((0, 0), drift))],
            ));
            drift = (0, 0);
        }
        if let Some(leg) = leg {
            parts.push(leg.to_string());
        }
    }
    Ok(parts.join(", "))
}

/// Degrees between two bearings, the short way round.
fn apart(one: f64, other: f64) -> f64 {
    (py::modulo_f(one - other + 180.0, 360.0) - 180.0).abs()
}

/// `Callable[[Interactable, int], str]`: what a thing is said as (`Navigator.label`).
pub type Label = Rc<dyn Fn(&Interactable, i64) -> Result<String>>;
/// `Callable[[Interactable, int], str]`: what a thing is called (`Navigator.name_of`).
pub type NameOf = Rc<dyn Fn(&Interactable, i64) -> Result<String>>;
/// `Callable[[str], None]`: say something about the way unasked (`Navigator._tell`).
pub type Tell = Rc<dyn Fn(&str)>;

/// P and guiding: the way on foot to a thing, said as its legs, and the player followed along
/// it, the next leg said at each turn and the thing on arriving.
pub struct Guide {
    field: Rc<dyn FieldSource>,
    label: Label,
    name_of: NameOf,
    /// Says what is said unasked as the player walks (`Navigator._tell`).
    pub(crate) tell: Tell,
    log: Log,
    /// Whether a way is followed turn by turn (`[pa3ds.navigation]`).
    wording: Navigation,
    clock: Clock,
    /// What a leg on water, through a bush or through a rock is said with, by the move's
    /// number: the game's name for the move, out of the ROM's move names, or `OWN_MOVE_WORDS`
    /// when there is no ROM to read it from.
    moves: IndexMap<i64, String>,
    /// What a leg up a sandy slope is said with: the game's name for the Mach Bike.
    bike: String,
    /// What a hop up a bumpy slope is said with: the game's name for the Acro Bike, said on
    /// the bike too, since nothing read says which of the two the player is riding.
    acro: String,
    /// Whether the field's walls are read, so P can find a way. Sun keeps its walls as
    /// meshes pa3ds does not read, and a way found without them would lead through them.
    routes: bool,
    /// Cells either way a player may be off the path and still be walking it. None in ORAS,
    /// where every step lands in a cell of the grid. Sun's player walks anywhere, a body a cell
    /// and a half across, and drifts a cell off a straight leg as a matter of course: counted
    /// as leaving the path, every such step found a new one and said it, several times a second
    /// (sun_6.ctm, 2026-09-26).
    slack: i64,
    /// The way being walked, or None. Shared with the call that is holding it (`to`'s `held`),
    /// as one object in Python, so it is an `Rc<RefCell<Guidance>>`.
    pub guidance: Option<Rc<RefCell<Guidance>>>,
    /// Where the player was last looked at on the water and every cell a current carries them
    /// through from there, so it is said once (`swept`).
    drift: Vec<Cell>,
    /// What `reachable` was last asked and found.
    reached: RefCell<Option<(Asked, HashSet<Key>)>>,
}

/// What `reachable` is asked: the zone, the player's cell, what is listed and the visit.
type Asked = (i64, Cell, Vec<Interactable>, Option<(i64, i64)>);

impl Guide {
    /// `label` and `name_of` say a thing as the navigator does; `tell` says what is said
    /// unasked, as the player walks. Python's `clock` defaults to `time.monotonic`, `routes` to
    /// True and `slack` to 0.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        field: Rc<dyn FieldSource>,
        label: Label,
        name_of: NameOf,
        tell: Tell,
        log: Log,
        wording: Navigation,
        clock: Clock,
        routes: bool,
        slack: i64,
        moves: Option<IndexMap<i64, String>>,
        bike: Option<String>,
        acro: Option<String>,
    ) -> Self {
        let mut own: IndexMap<i64, String> = OWN_MOVE_WORDS
            .iter()
            .map(|&(clear, word)| (clear, word.to_string()))
            .collect();
        own.extend(moves.unwrap_or_default());
        Guide {
            field,
            label,
            name_of,
            tell,
            log,
            wording,
            clock,
            moves: own,
            bike: match bike {
                Some(bike) if !bike.is_empty() => bike,
                _ => OWN_SLOPE_WORD.to_string(),
            },
            acro: match acro {
                Some(acro) if !acro.is_empty() => acro,
                _ => OWN_HOP_WORD.to_string(),
            },
            routes,
            slack,
            guidance: None,
            drift: Vec::new(),
            reached: RefCell::new(None),
        }
    }

    /// The keys of the things listed in `where` that Shift and P finds a way to, ending at
    /// them and not short of them: every one where no way is found on this ground (Sun's, the
    /// rails', the sky). Judged as Shift and P judges its own way, with the same pieces: the
    /// same grounds in turn (`_grounds`), the same cells at each thing, a side a big boulder is
    /// pushed from (`_aims`), the same moves from floor to floor (`Ground.reached`, one walk out
    /// to every thing at once where Shift and P searches for one), the same end at it
    /// (`Ground.ends_at`), and over thin ice Shift and P's own way (`_way`). Only the rotating
    /// doors are not followed, so what lies past them is kept (`Ground.reached`).
    ///
    /// Orion's, 2026-10-11: in Team Magma's Hideout the list held every teleporter of the place,
    /// most of them in rooms the player was not in. Walked a cell at a time, any floor of it,
    /// Victory Road's stacked 1F kept all 13 things listed, 8 of them short of Shift and P's
    /// reach (movie47, spike 215).
    pub fn reachable(&self, r#where: &Field) -> Result<HashSet<Key>> {
        let everything: HashSet<Key> = r#where
            .interactables
            .iter()
            .map(|item| item.key())
            .collect();
        if !self.routes || r#where.on_rail || r#where.steer.is_some() {
            return Ok(everything);
        }
        let asked: Asked = (
            r#where.zone,
            r#where.cell,
            r#where.interactables.clone(),
            self.field.visit(),
        );
        if let Some((last, keys)) = &*self.reached.borrow()
            && *last == asked
        {
            return Ok(keys.clone());
        }
        let mut out: HashSet<Key> = HashSet::new();
        let grounds = self
            .grounds(r#where, true, None)?
            .collect::<Result<Vec<(Ground, Option<f64>)>>>()?;
        // ponytail: a walk over the whole matrix each time the player has moved, 0.9 s on one of
        // 8 by 8 chunks and tens of milliseconds in a cave; keep it while they stay where it
        // reached, if a route's wait is heard.
        for (ground, _) in &grounds {
            let mut goals: Vec<(&Interactable, IndexMap<Cell, Option<&'static str>>)> = Vec::new();
            for item in &r#where.interactables {
                if !out.contains(&item.key()) {
                    goals.push((item, aims(ground, item, None)?));
                }
            }
            let cells: Vec<Cell> = goals
                .iter()
                .flat_map(|(_, cells)| cells.keys().copied())
                .collect();
            let reached = ground.reached(r#where.cell, &cells)?;
            for (item, cells) in &goals {
                let mut any = false;
                'cells: for &cell in cells.keys() {
                    for &floor in reached.get(&cell).into_iter().flatten() {
                        if ground.ends_at(item, cell, floor)? {
                            any = true;
                            break 'cells;
                        }
                    }
                }
                if any {
                    out.insert(item.key());
                }
            }
        }
        if !grounds[0].0.ice.is_empty() {
            // A way over thin ice steps on every tile once, which no walk out follows.
            let mut over: HashSet<Key> = HashSet::new();
            for item in &r#where.interactables {
                if !out.contains(&item.key())
                    && let Some(way) = self.way(r#where, item, true, "", None, None)?
                    && !way.short
                {
                    over.insert(item.key());
                }
            }
            out.extend(over);
        }
        *self.reached.borrow_mut() = Some((asked, out.clone()));
        Ok(out)
    }

    /// The way from where the player stands to `item`, as `to` says it and `reachable`
    /// judges it, or None off the rails the game has loaded. Found on each ground in turn until
    /// one ends at the thing (`_grounds`), to its cells, a side a big boulder is pushed from
    /// (`_aims`), and where it falls short, over a room of thin ice instead, which clears it
    /// (`_over_ice`).
    ///
    /// Python's `heading`, `height` and `push` default to "", None and None.
    fn way(
        &self,
        r#where: &Field,
        item: &Interactable,
        hm: bool,
        heading: &str,
        height: Option<f64>,
        push: Option<&str>,
    ) -> Result<Option<Way>> {
        let mut height = height;
        let mut grounds = self.grounds(r#where, hm, height)?;
        let (mut ground, _) = grounds.next().expect("the first ground is always there")?;
        if r#where.on_rail && ground.corridor.is_none() {
            return Ok(None);
        }
        let mut goals = aims(&ground, item, push)?;
        let mut path = ground.find(r#where.cell, &goals, heading, Some(item), true, None)?;
        let levels: Vec<Option<f64>> = std::iter::once(height)
            .chain(ground.decks.iter().map(|&deck| Some(deck)))
            .collect();
        if !reached(&ground, item, &goals, &path)? {
            for other in grounds {
                let (other, deck) = other?;
                let aim = aims(&other, item, push)?;
                let found = other.find(r#where.cell, &aim, heading, Some(item), false, None)?;
                if let Some(found) = found {
                    ground = other;
                    goals = aim;
                    path = Some(found);
                    height = deck;
                    break;
                }
            }
        }
        let mut short = !reached(&ground, item, &goals, &path)?;
        let over = if short {
            over_ice(r#where, &ground, &path, heading)?
        } else {
            None
        };
        let ice = over.is_some();
        if let Some(over) = over {
            path = Some(over);
            short = false;
        }
        Ok(Some(Way {
            ground,
            goals,
            path,
            height,
            short,
            ice,
            levels,
        }))
    }

    /// The grounds a way is looked for on, in turn, with the height each one's decks are
    /// walked at: `height` or the player's own, then each other deck's (spike 159).
    ///
    /// Python's generator makes its first ground at its first `next`, which every caller asks
    /// for at once; the others, as Python's, only as they are asked for.
    fn grounds<'a>(
        &'a self,
        r#where: &'a Field,
        hm: bool,
        height: Option<f64>,
    ) -> Result<impl Iterator<Item = Result<(Ground, Option<f64>)>> + 'a> {
        // ponytail: where the ground is not stacked, a whole map's decks walked at one height, so
        // a way that goes under a deck and later onto it is not found; a stacked one finds it.
        let ground = self.ground(r#where, hm, height)?;
        let decks = ground.decks.clone();
        Ok(std::iter::once(Ok((ground, height))).chain(
            decks
                .into_iter()
                .map(move |deck| Ok((self.ground(r#where, hm, Some(deck))?, Some(deck)))),
        ))
    }

    /// Find the way from where the player stands to `item`, say it, and follow the player
    /// along it; or say why there is nothing to follow. `hm` finds it over water too.
    ///
    /// Where the decks at the player's height lead nowhere, they are walked at each other
    /// floor's height in turn: from the foot of Route 119 the way north is over a bridge high
    /// above the path it crosses (spike 159). Where nothing leads there, the way goes as near as
    /// the player can get, said with ", then can't continue", and "no path" only where nowhere
    /// is nearer than where they stand (the user, 2026-10-03). `height` is where to walk them
    /// first, the height a way being found again was walked at: at the player's own, every
    /// step off a way over a bridge searched the whole map before finding it again.
    ///
    /// `heading` is the way a player who has left the path was walking. A new path that starts
    /// that way is taken quietly, and its next turn said when they reach it: a player who took
    /// a way of their own as short as ours heard the whole way again at every step, north
    /// first where ours went east first (poke_center_pc.ctm, 2026-09-24).
    ///
    /// `held` is the way being walked when P is pressed again. Where no new way is found and the
    /// player is on that one to the same thing, it is kept and what is left of it said: P again
    /// halfway up Route 110 said "no path to path" and left nothing to follow, while the way
    /// being walked still went on (omega_ruby_movil.ctm, spike 142).
    ///
    /// `named` says what the way leads to before it, for a way nobody asked for (`found`), and
    /// `asked` is whether a key asked for it (`_Guidance.asked`).
    ///
    /// A big boulder is gone to on a side it is pushed from, the way `push` says where it can be
    /// pushed so and the nearest otherwise, and the way is said with the push: "to push east:
    /// 3 west, 2 north" (Orion, 2026-10-11).
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn to(
        &mut self,
        r#where: &Field,
        item: &Interactable,
        say: &dyn Fn(&str),
        heading: &str,
        hm: bool,
        held: Option<Rc<RefCell<Guidance>>>,
        height: Option<f64>,
        named: bool,
        asked: bool,
        push: Option<&str>,
    ) -> Result<()> {
        self.guidance = None;
        let name = (self.name_of)(item, r#where.zone)?;
        if !self.routes {
            let text = format!(
                "{}, {}, {}",
                (self.label)(item, r#where.zone)?,
                item.bearing(r#where.cell, r#where.heading)?,
                own("p-no-route-in-game", &[])
            );
            say(&text);
            return Ok(());
        }
        let Some(way) = self.way(r#where, item, hm, heading, height, push)? else {
            // Off the rails the game has loaded, the walls are not known, and the chunks there
            // mark every cell a wall: a way found on them would be nonsense.
            let text = format!(
                "{name}, {}, {}",
                item.bearing(r#where.cell, r#where.heading)?,
                own("p-no-route-on-streets", &[])
            );
            say(&text);
            return Ok(());
        };
        let Way {
            mut ground,
            goals,
            mut path,
            mut height,
            short,
            ice,
            levels,
        } = way;
        let sides = sides(item, push);
        if short {
            let at = match &held {
                Some(held) if held.borrow().key == item.key() => held.borrow().at(r#where.cell)?,
                _ => None,
            };
            if let Some(held) = &held
                && let Some(at) = at
                && at < held.borrow().path.len() as i64 - 1
            {
                self.guidance = Some(held.clone());
                let mut text = left(&held.borrow(), at, r#where)?;
                if held.borrow().short {
                    text += &format!(", {}", own("p-then-cant-continue", &[]));
                }
                say(&text);
                return Ok(());
            }
        }
        let mut blocker = String::new();
        if short && let Some(past) = self.up_to(r#where, item, hm, heading, &levels, &sides)? {
            // Python also rebinds `goals` here, and reads it again only where not short.
            let (past_ground, _goals, way, past_height, stood) = past;
            ground = past_ground;
            path = Some(way);
            height = past_height;
            blocker = match stood {
                None => String::new(),
                Some(stood) => {
                    let thing = (self.label)(&stood, r#where.zone)?;
                    format!(", {}", own("p-in-the-way", &[("thing", &thing)]))
                }
            };
        } else if short && let Some(again) = dive_again(r#where.cell, &ground, &goals, heading)? {
            let (way, text) = again;
            path = Some(way);
            blocker = text;
        }
        let path = match path {
            Some(path) if !(short && path.len() == 1) => path,
            _ => {
                say(&(own("p-no-path-to", &[("thing", &name)]) + &blocker));
                return Ok(());
            }
        };
        let pushing = if short || sides.is_empty() {
            None
        } else {
            path.last()
                .and_then(|end| goals.get(end).copied().flatten())
        };
        let to_push = match pushing {
            None => String::new(),
            Some(pushing) => own("p-to-push", &[("way", &pathfind::way(pushing))]) + " ",
        };
        if path.len() == 1 {
            let text = format!(
                "{to_push}{name}, {}",
                item.bearing(r#where.cell, r#where.heading)?
            );
            say(&text);
            return Ok(());
        }
        // A player already surfing needs no telling that the water is crossed with Surf.
        let surfing = ground.water(r#where.cell)?;
        let mut pushed: HashSet<(Cell, Cell)> = HashSet::new();
        for i in ground.pushes(&path)? {
            pushed.insert((path[i as usize - 1], path[i as usize]));
        }

        let lands = |cell: Cell, step: Cell| -> Result<Cell> {
            // The jump over the shore is one move, said with the side it lands on: "Surf 30
            // east, 2 north", not "Surf 30 east, Surf 1 north, 1 north".
            if !ground.shore(step)? {
                return Ok(step);
            }
            Ok((2 * step.0 - cell.0, 2 * step.1 - cell.1))
        };
        let mut mount = String::new();
        for thing in &r#where.interactables {
            if thing.move_model
                && thing
                    .cells()
                    .iter()
                    .any(|cell| ground.mounts.contains(cell))
            {
                mount = (self.label)(thing, r#where.zone)?;
                break;
            }
        }
        let ridden = if ground.rides.is_empty() {
            HashMap::new()
        } else {
            rides(&ground, &path, &mount)?
        };
        let how = |cell: Cell, step: Cell| -> Result<String> {
            if let Some(word) = ridden.get(&(cell, step)) {
                return Ok(word.clone());
            }
            if pushed.contains(&(cell, step)) {
                return Ok(PUSH.to_string());
            }
            if ground.carry(cell)? == Some(pathfind::towards(cell, step)) {
                return Ok(CURRENT.to_string());
            }
            if ground.hopping(cell, step) {
                return Ok(self.acro.clone());
            }
            if ground.climbing(cell, step) {
                return self
                    .moves
                    .get(&WATERFALL)
                    .cloned()
                    .ok_or_else(|| Error::key(WATERFALL.to_string()));
            }
            if let Some(clear) = ground.cleared.get(&step) {
                return self
                    .moves
                    .get(clear)
                    .cloned()
                    .ok_or_else(|| Error::key(clear.to_string()));
            }
            if !surfing && ground.water(lands(cell, step)?)? {
                return self
                    .moves
                    .get(&SURF)
                    .cloned()
                    .ok_or_else(|| Error::key(SURF.to_string()));
            }
            if !ground.bike && ground.climbs(cell, step)? {
                return Ok(self.bike.clone());
            }
            Ok(String::new())
        };
        let key = |cell: Cell, step: Cell| ground.key(cell, step);
        let legs = pathfind::legs_with(&path, Some(&how), Some(&key))?;
        let visit = self.field.visit();
        let along: HashMap<Cell, i64> = path
            .iter()
            .enumerate()
            .rev()
            .map(|(i, &cell)| (cell, i as i64))
            .collect();
        let nudge = nudge(&ground, r#where.place, &path, 0)?;
        let since = (self.clock)();
        let first = legs.first().map(|leg| leg.direction);
        self.guidance = Some(Rc::new(RefCell::new(Guidance {
            key: item.key(),
            name,
            visit,
            path,
            legs: legs.clone(),
            along,
            cell: r#where.cell,
            leg: 0,
            camera: r#where.bearing,
            seen: r#where.bearing,
            since,
            hm,
            height,
            ice,
            short,
            blocker: blocker.clone(),
            asked,
            push: pushing,
            place: r#where.place,
            still: (self.clock)(),
            judged: r#where.place,
            nudge,
        })));
        let first = first.ok_or_else(|| Error::index("list index out of range"))?;
        if first != heading {
            // Short of the thing, the way goes as near as it can, which the player hears the
            // end of: they asked for that over "no path" (the user, 2026-10-03).
            let mut text = if named {
                format!("{}, ", (self.label)(item, r#where.zone)?)
            } else {
                String::new()
            };
            text += &to_push;
            if let Some(nudge) = nudge {
                text += &(own("p-nudge-then", &[("nudge", &nudged(nudge))]) + " ");
            }
            text += &spoken(&legs, r#where)?;
            if short {
                text += &format!(", {}{blocker}", own("p-then-cant-continue", &[]));
            }
            if ice {
                text += &format!(", {}", own("p-and-ice-cleared", &[]));
            }
            say(&text);
        }
        if !self.wording.turn_by_turn {
            // Said once and not followed: P again says it from wherever the player has got to.
            self.guidance = None;
        }
        Ok(())
    }

    /// Keep up with the player along the path being walked: the next leg as they reach each
    /// turn, the thing itself when they arrive, and a new path when they leave this one.
    pub fn follow(&mut self, r#where: &Field) -> Result<()> {
        let Some(guidance) = self.guidance.clone() else {
            return Ok(());
        };
        let visit = self.field.visit();
        let settled = self.settled(r#where, &mut guidance.borrow_mut());
        let seen = guidance.borrow().cell;
        if r#where.cell == seen {
            // The field rebuilt around a player standing still is not a warp: nothing moved.
            guidance.borrow_mut().visit = visit;
            if settled && let Some(bearing) = r#where.bearing {
                self.turned(r#where, &guidance, bearing)?;
            }
            return self.line_up(r#where, &guidance);
        }
        if visit != guidance.borrow().visit {
            self.guidance = None;
            (self.log)(&format!(
                "path: a warp, the way to {} is left",
                guidance.borrow().name
            ));
            return Ok(());
        }
        let was = std::mem::replace(&mut guidance.borrow_mut().cell, r#where.cell);
        {
            let mut guidance = guidance.borrow_mut();
            guidance.place = r#where.place;
            guidance.still = (self.clock)();
            guidance.nudge = None;
        }
        let heading = pathfind::towards(was, r#where.cell);
        let mut at = guidance.borrow().at(r#where.cell)?;
        // A step along a diagonal street lands up to a cell off the diagonal the way was laid on
        // the grid by (the tape of x_route-6-2.ctm, 2026-10-20).
        let slack = if self.slack != 0 {
            self.slack
        } else {
            i64::from(
                guidance
                    .borrow()
                    .path
                    .windows(2)
                    .any(|pair| pair[0].0 != pair[1].0 && pair[0].1 != pair[1].1),
            )
        };
        if at.is_none() && slack != 0 {
            at = near(&guidance.borrow().path, r#where.cell, slack);
        }
        let Some(at) = at else {
            return self.again(r#where, &guidance, heading);
        };
        let last = guidance.borrow().path.len() as i64 - 1;
        if at == last {
            self.guidance = None;
            let listed = self.field.interactables(r#where)?;
            let item = self.find(&listed, &guidance.borrow());
            if let Some(item) = item {
                let text = format!(
                    "{}{}{}, {}",
                    if guidance.borrow().short {
                        format!(
                            "{}{}, ",
                            own("p-cant-continue", &[]),
                            guidance.borrow().blocker
                        )
                    } else {
                        String::new()
                    },
                    if guidance.borrow().ice {
                        format!("{}, ", own("p-ice-cleared", &[]))
                    } else {
                        String::new()
                    },
                    guidance.borrow().name,
                    item.bearing(r#where.cell, r#where.heading)?
                );
                (self.tell)(&text);
            }
            return Ok(());
        }
        // A chunk loaded since the path was found can show a wall where nothing was known. A
        // ledge on the way is not one, nor the shore or a waterfall: they are jumped.
        let (hm, height) = {
            let guidance = guidance.borrow();
            (guidance.hm, guidance.height)
        };
        let ground = self.ground(r#where, hm, height)?;
        let mut all = true;
        for &cell in py::slice_from(&guidance.borrow().path, at + 1) {
            if !(ground.open(cell)? || ground.ledge(cell)?.is_some() || ground.over(cell)?) {
                all = false;
                break;
            }
        }
        if !all {
            return self.again(r#where, &guidance, heading);
        }
        let leg = guidance
            .borrow()
            .legs
            .iter()
            .position(|leg| leg.end > at)
            .expect("a leg ends past the cell") as i64;
        if leg != guidance.borrow().leg {
            // A current's run of legs was said whole as it began, where it leaves the player.
            let riding = {
                let guidance = guidance.borrow();
                py::slice(&guidance.legs, guidance.leg, leg + 1)
                    .iter()
                    .all(|one| one.how == CURRENT)
            };
            let (now, rest) = {
                let mut guidance = guidance.borrow_mut();
                guidance.leg = leg;
                guidance.camera = r#where.bearing;
                let now = guidance.legs[leg as usize].clone();
                let rest: Vec<Leg> = if now.how == CURRENT {
                    py::slice_from(&guidance.legs, leg + 1).to_vec()
                } else {
                    Vec::new()
                };
                (now, rest)
            };
            let mut run = vec![Leg {
                steps: now.end - at,
                ..now
            }];
            run.extend(rest.into_iter().take_while(|one| one.how == CURRENT));
            if !riding {
                let text = spoken(&run, r#where)?;
                (self.tell)(&text);
            }
        }
        Ok(())
    }

    /// Say where a current takes the player as it catches them, "ride the current 28 west",
    /// unless the way being guided says it: it says each current on it as it begins. On Route
    /// 132 the player cannot turn on one, and is carried a cell at a time to calm water or onto
    /// a sandbar, wherever they had meant to go (movie43, spike 208). Only a player surfing is
    /// looked at, once a cell.
    pub fn swept(&mut self, r#where: &Field) -> Result<()> {
        if !r#where.surfing {
            self.drift = Vec::new();
        }
        if !r#where.surfing || self.drift.contains(&r#where.cell) {
            return Ok(());
        }
        let drift = self.ground(r#where, false, None)?.drift(r#where.cell)?;
        self.drift = drift.clone();
        let guidance = self.guidance.clone();
        let off_the_way = match &guidance {
            None => true,
            Some(guidance) => guidance.borrow().at(r#where.cell)?.is_none(),
        };
        if drift.len() > 1 && off_the_way {
            let text = own(
                "p-current",
                &[(
                    "bearing",
                    &field::bearing(r#where.cell, drift[drift.len() - 1]),
                )],
            );
            (self.tell)(&text);
        }
        Ok(())
    }

    /// Within a cell of the way, tell a player who has stopped off the next leg's line, where
    /// a corner would catch them on it, which way to step, "a little west"; and once they come
    /// within `LINED_UP` of it, moving or not, "lined up" (Orion, 2026-10-11, who lined up by
    /// sneaking: a gentle tilt of the circle pad moves them a little at a time).
    fn line_up(&self, r#where: &Field, guidance: &Rc<RefCell<Guidance>>) -> Result<()> {
        let Some(place) = r#where.place else {
            return Ok(());
        };
        let now = (self.clock)();
        if Some(place) != guidance.borrow().place {
            let lined_up = {
                let mut guidance = guidance.borrow_mut();
                guidance.place = Some(place);
                guidance.still = now;
                match guidance.nudge {
                    Some(nudge) if lined(nudge, place, r#where.cell) => {
                        guidance.nudge = None;
                        true
                    }
                    _ => false,
                }
            };
            if lined_up {
                (self.tell)(&own("p-lined-up", &[]));
            }
            return Ok(());
        }
        {
            let mut guidance = guidance.borrow_mut();
            if now - guidance.still < STOPPED || Some(place) == guidance.judged {
                return Ok(());
            }
            guidance.judged = Some(place);
        }
        let Some(at) = guidance.borrow().at(r#where.cell)? else {
            return Ok(());
        };
        let (hm, height) = {
            let guidance = guidance.borrow();
            (guidance.hm, guidance.height)
        };
        let ground = self.ground(r#where, hm, height)?;
        let nudge = nudge(&ground, Some(place), &guidance.borrow().path, at)?;
        if let Some(nudge) = nudge {
            guidance.borrow_mut().nudge = Some(nudge);
            (self.tell)(&nudged(nudge));
        }
        Ok(())
    }

    /// Where somebody stands in the way: the way there as if nobody stood anywhere, as far
    /// as the first who does, with its ground, goals, path, the height its decks are walked at,
    /// and who it is; or None where nobody is what stops it. In Fortree City the invisible
    /// Kecleon stands in the one way to the Gym until the Devon Scope shows it, and the way as
    /// near as the player could get as the crow flies ended behind a wall from the Gym, on a
    /// walkway 7 cells south of its door (spike 160). What stands unlisted stays: Sootopolis
    /// Gym's invisible blockers shut the corridors up to Wallace, and cleared away, the Lass
    /// pacing the lower floor was said to be in the way up to him (spike 167). Whoever can be
    /// walked round is: on Route 7 the way west went through Shauna, standing in the open, on
    /// its way to the Snorlax filling the road, and she was said to be in the way (2026-10-19).
    fn up_to(
        &self,
        r#where: &Field,
        item: &Interactable,
        hm: bool,
        heading: &str,
        levels: &[Option<f64>],
        sides: &HashSet<&'static str>,
    ) -> Result<Option<Past>> {
        let own: HashSet<Cell> = item.cells().into_iter().collect();
        let listed: HashSet<Cell> = r#where
            .interactables
            .iter()
            .filter(|thing| thing.move_model)
            .flat_map(|thing| thing.cells())
            .collect();
        for &level in levels {
            let ground = self.ground(r#where, hm, level)?;
            // A boulder a push will not move that way is in the way too (movie46, spike 213), and
            // to a player on foot a rock only a ridden Pokémon smashes (spike 238).
            let no_smashes = HashSet::new();
            let smashes = if ground.riding {
                &no_smashes
            } else {
                &ground.smashed
            };
            let crowd: HashSet<Cell> = ground
                .standing
                .iter()
                .chain(smashes.iter())
                .filter(|cell| listed.contains(cell) && !own.contains(cell))
                .copied()
                .collect();
            let mut clear = ground.clone();
            // `standing - crowd`, in `standing`'s order.
            clear.standing = ground
                .standing
                .iter()
                .filter(|cell| !crowd.contains(cell))
                .copied()
                .collect();
            clear.shoves = IndexMap::new();
            clear.smashed = ground.smashed.difference(&crowd).copied().collect();
            clear.crowd = crowd;
            let goals = aimed(clear.goals(item)?, sides);
            let way = clear.find(r#where.cell, &goals, heading, Some(item), false, None)?;
            let Some(way) = way else {
                continue;
            };
            let floors = clear.walked(&way)?;
            let rode = ground.rode(&way)?;
            let mut found: Option<usize> = None;
            for (i, &cell) in way.iter().enumerate() {
                if ground.stands(cell, floors[i])?
                    || (i > 0 && (!ground.joined(way[i - 1], cell) || rode[i].is_none()))
                {
                    found = Some(i);
                    break;
                }
            }
            let Some(at) = found else {
                continue;
            };
            let stood = r#where
                .interactables
                .iter()
                .find(|thing| thing.move_model && thing.cells().contains(&way[at]))
                .cloned();
            return Ok(Some((ground, goals, way[..at].to_vec(), level, stood)));
        }
        Ok(None)
    }

    /// Whether the camera has stopped turning: it has stayed within `STILL` of one bearing
    /// for `SETTLE` seconds. False off the rails, where there is no camera to turn.
    fn settled(&self, r#where: &Field, guidance: &mut Guidance) -> bool {
        let Some(bearing) = r#where.bearing else {
            guidance.seen = None;
            return false;
        };
        let now = (self.clock)();
        if guidance
            .seen
            .is_none_or(|seen| apart(bearing, seen) > STILL)
        {
            guidance.seen = Some(bearing);
            guidance.since = now;
            return false;
        }
        now - guidance.since >= SETTLE
    }

    /// The way on from here again, when the camera has stopped somewhere new with the
    /// player standing still. On a rail the way is said from the camera, which left and right
    /// turn only while held: a tap turns the player and leaves the camera where it was, or
    /// between two ways, and the way said before is wrong for it, and a hold makes a turn said
    /// done, which is worth hearing (spike 105).
    fn turned(
        &self,
        r#where: &Field,
        guidance: &Rc<RefCell<Guidance>>,
        bearing: f64,
    ) -> Result<()> {
        if guidance
            .borrow()
            .camera
            .is_some_and(|camera| apart(bearing, camera) < MOVED)
        {
            return Ok(());
        }
        let at = guidance.borrow().at(r#where.cell)?;
        let last = guidance.borrow().path.len() as i64 - 1;
        let Some(at) = at else {
            return Ok(());
        };
        if at == last {
            return Ok(());
        }
        guidance.borrow_mut().camera = Some(bearing);
        let text = left(&guidance.borrow(), at, r#where)?;
        (self.tell)(&text);
        Ok(())
    }

    /// Find the way again from where the player has got to, walking `heading`.
    fn again(
        &mut self,
        r#where: &Field,
        guidance: &Rc<RefCell<Guidance>>,
        heading: &str,
    ) -> Result<()> {
        let listed = self.field.interactables(r#where)?;
        if Ground::of(&listed, Of::default())
            .stepped
            .contains(&r#where.cell)
        {
            // On a door that is not the way's end, or the hidden Pokémon: what the cell does is
            // about to happen, and a path said now would be for a map the player is leaving. The
            // game walks the player into a door's own cell before the warp (poke_center_pc.ctm).
            return Ok(());
        }
        let ground = {
            let guidance = guidance.borrow();
            self.ground(r#where, guidance.hm, guidance.height)?
        };
        if ground.falls.contains_key(&r#where.cell) {
            // Halfway up or down a waterfall, the player lands a cell on: going down Ever Grande
            // City's, the way found there was "Waterfall 1 north", back up (spike 170).
            return Ok(());
        }
        if ground.drift(r#where.cell)?.len() > 1 {
            // A current is carrying the player where they cannot turn, and the way is found
            // where it leaves them: on Route 133 a new way was said at every cell of 63 (movie43).
            return Ok(());
        }
        let Some(item) = self.find(&listed, &guidance.borrow()) else {
            self.guidance = None;
            return Ok(());
        };
        let (hm, height, asked, push) = {
            let guidance = guidance.borrow();
            (guidance.hm, guidance.height, guidance.asked, guidance.push)
        };
        let log = self.log.clone();
        let quietly = move |text: &str| log(&format!("path, quietly: {text}"));
        let tell = self.tell.clone();
        let say: &dyn Fn(&str) = if asked { &*tell } else { &quietly };
        self.to(
            &listed, &item, say, heading, hm, None, height, false, asked, push,
        )
    }

    /// The field's ground, its decks walked at `height` or the player's own, crossing water
    /// for a way with HMs and for a player already on it: P from the water keeps to it, or
    /// there is no first step to take. Only a way with HMs cuts through a bush. A slope is
    /// ridden up with HMs and by a player on the bike, not on a Pokémon they ride, which sets
    /// the bike's bits too (`Ground.riding`, spike 238).
    pub fn ground(&self, r#where: &Field, hm: bool, height: Option<f64>) -> Result<Ground> {
        let mut ground = self.field.ground(r#where, height)?;
        ground.surf = hm || ground.water(r#where.cell)?;
        ground.hm = hm;
        ground.bike = r#where.on_bike && !ground.riding;
        Ok(ground)
    }

    /// What the path leads to, where it is now, or None, said, when it has gone: a person
    /// walks, and a ball is picked up.
    fn find(&self, listed: &Field, guidance: &Guidance) -> Option<Interactable> {
        let item = listed
            .interactables
            .iter()
            .find(|item| item.key() == guidance.key)
            .cloned();
        if item.is_none() {
            (self.tell)(&own("p-gone", &[("thing", &guidance.name)]));
        }
        item
    }
}
