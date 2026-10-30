//! Pathfinding: the way on foot from one cell to another, round what cannot be walked, said as
//! its legs.
//!
//! Nothing here knows a game (ADR-0006). A map is cells, x east and z south; whether a cell can be
//! walked is the game's reading to answer, and a path is found inside a box of cells, since beyond
//! what the game has loaded a cell is assumed open and an unbounded search for a way round would
//! never end.
//!
//! A path is said as its **legs**, each a straight run: "5 east, 2 north". A player who cannot see
//! the map counts steps and turns, so a turn costs as much as a step (`TURN`): a way two steps
//! longer that saves three turns is the one taken. A player already walking one way turns if the
//! path starts another way, so going on straight costs nothing. A goal can ask to be reached facing
//! one way — the way to a door ends facing the door — and that settles a tie between two ways and
//! no more (`FACE`). A ledge is jumped down, one way only, and never climbed; a shore is jumped
//! either way (`over`). A step can carry a toll on top of its price, as going onto water to surf
//! does, and a leg a word to say it by: "Surf 12 south".
//!
//! Bound by:
//! - ADR-0024: a turn costs as much as a step.

use std::cell::{Cell as Slot, RefCell};
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt;
use std::hash::Hash;

use indexmap::IndexMap;

use crate::core::words::{own, said};
use crate::games::common::field::guide::Held;
use crate::games::common::field::things::{TELEPORTER, kind_said};
use crate::py::{self, Error, Result};

pub type Cell = (i64, i64);

pub const NORTH: &str = "north";
pub const EAST: &str = "east";
pub const SOUTH: &str = "south";
pub const WEST: &str = "west";
/// A step each way, in the order the search tries them.
pub const STEPS: [(&str, Cell); 4] = [
    (NORTH, (0, -1)),
    (EAST, (1, 0)),
    (SOUTH, (0, 1)),
    (WEST, (-1, 0)),
];
/// A step to a cell touching at a corner, taken only where `find`'s `slants` allows it.
pub const DIAGONALS: [(&str, Cell); 4] = [
    ("north-east", (1, -1)),
    ("south-east", (1, 1)),
    ("south-west", (-1, 1)),
    ("north-west", (-1, -1)),
];
/// Clockwise, a quarter turn apart.
pub const COMPASS: [&str; 4] = [NORTH, EAST, SOUTH, WEST];
pub const AHEAD: &str = "ahead";
pub const BEHIND: &str = "behind";
pub const LEFT: &str = "left";
/// The ways of a player's own frame, as a player heading one way is told them (`facing`).
pub const RIGHT: &str = "right";

pub const STEP: i64 = 2;
/// What a turn costs: a step's worth.
pub const TURN: i64 = 2;
/// How many cells `cover` tries before giving up, about half a second: Sootopolis Gym's biggest
/// room of thin ice, 78 tiles round three blocks of floor, is crossed in a few hundred.
pub const COVER_LIMIT: i64 = 20_000;
/// What arriving facing another way than the goal asks costs: half a step, less than anything
/// else can differ by, so it only chooses between ways that are otherwise the same.
pub const FACE: i64 = 1;

/// `COMPASS.index(way)`, which raises where `way` is none of the four.
#[track_caller]
fn compass_index(way: &str) -> Result<i64> {
    match COMPASS.iter().position(|&known| known == way) {
        Some(at) => Ok(at as i64),
        None => Err(Error::value("tuple.index(x): x not in tuple")),
    }
}

/// `STEPS[way]`, which raises where `way` is none of the four.
#[track_caller]
pub fn step_of(way: &str) -> Result<Cell> {
    match STEPS.iter().find(|(known, _)| *known == way) {
        Some(&(_, step)) => Ok(step),
        None => Err(Error::key(py::repr(way))),
    }
}

/// The cells a search may walk: west to east and north to south, both ends in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Box {
    pub west: i64,
    pub north: i64,
    pub east: i64,
    pub south: i64,
}

impl Box {
    pub fn holds(&self, cell: Cell) -> bool {
        self.west <= cell.0 && cell.0 <= self.east && self.north <= cell.1 && cell.1 <= self.south
    }

    /// The smallest box holding every cell, `margin` cells wider each way.
    pub fn around(cells: impl IntoIterator<Item = Cell>, margin: i64) -> Result<Box> {
        let held: Vec<Cell> = cells.into_iter().collect();
        let xs: Vec<i64> = held.iter().map(|cell| cell.0).collect();
        let zs: Vec<i64> = held.iter().map(|cell| cell.1).collect();
        let (Some(west), Some(north), Some(east), Some(south)) = (
            xs.iter().min(),
            zs.iter().min(),
            xs.iter().max(),
            zs.iter().max(),
        ) else {
            return Err(Error::value("min() iterable argument is empty"));
        };
        Ok(Box {
            west: west - margin,
            north: north - margin,
            east: east + margin,
            south: south + margin,
        })
    }

    /// This box cut down to the part inside `other`.
    pub fn within(&self, other: Box) -> Box {
        Box {
            west: self.west.max(other.west),
            north: self.north.max(other.north),
            east: self.east.min(other.east),
            south: self.south.min(other.south),
        }
    }
}

/// One straight run of a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leg {
    pub direction: &'static str,
    pub steps: i64,
    /// Where along the path the leg ends: the index of its last cell.
    pub end: i64,
    /// What the leg is walked with, said before it: "Surf". "" on foot.
    pub how: String,
    /// The handholds climbed in place before and after it, where it is the guide's subclass
    /// `Held`: None for a plain `Leg`. A field here, as Python's `isinstance(leg, Held)` is
    /// `held.is_some()`, so a list of legs can hold both.
    pub held: Option<Held>,
    /// Whether it is the guide's subclass `Vined`, the step that grabs a vine and its ride:
    /// Python's `isinstance(leg, Vined)`.
    pub vined: bool,
    /// Whether it is the guide's subclass `Teleported`: Python's `isinstance(leg, Teleported)`.
    /// A teleport, the step from a teleporter to where it puts the player, as a leg of its own:
    /// "6 north, teleporter, 2 east". Nothing is said as the player stands on the teleporter: the
    /// game is moving them, and the leg after it is said where it puts them (spike 258).
    pub teleported: bool,
}

impl Leg {
    /// A leg on foot; `how` is set with `..Leg::new(...)`.
    pub fn new(direction: &'static str, steps: i64, end: i64) -> Self {
        Leg {
            direction,
            steps,
            end,
            how: String::new(),
            held: None,
            vined: false,
            teleported: false,
        }
    }
}

impl fmt::Display for Leg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(held) = &self.held {
            return write!(f, "{}", held.said(self));
        }
        if self.vined {
            // `Vined.__str__`: the way faced to grab the vine, and what its ride does.
            return write!(
                f,
                "{}",
                own(
                    "p-vine",
                    &[
                        ("way", &way(self.direction)),
                        ("how", &said("how", &self.how)),
                    ]
                )
            );
        }
        if self.teleported {
            // `Teleported.__str__`.
            return write!(f, "{}", kind_said(TELEPORTER));
        }
        if !self.how.is_empty() {
            return write!(
                f,
                "{}",
                own(
                    "leg-with",
                    &[
                        ("how", &said("how", &self.how)),
                        ("steps", &self.steps),
                        ("way", &way(self.direction)),
                    ],
                )
            );
        }
        write!(
            f,
            "{}",
            own(
                "steps",
                &[("count", &self.steps), ("way", &way(self.direction))]
            )
        )
    }
}

/// `_SLANTS.get(step)`: a one-cell diagonal step -> its name, which `legs` says it by: `towards`
/// has no word for it.
fn slant(step: Cell) -> Option<&'static str> {
    DIAGONALS
        .iter()
        .find(|(_, known)| *known == step)
        .map(|&(name, _)| name)
}

/// What a step can change on the way besides where the player is: how far each of Fortree Gym's
/// rotating doors has turned (`find`'s `through`).
pub type Turns = Vec<i64>;

/// A cell, the way the player came into it ("" at the start), and the turns behind them.
type State<'a> = (Cell, &'a str, Turns);
/// Where every goal leads once facing its way is paid for.
const ARRIVED: State<'static> = ((0, 0), "arrived", Vec::new());

/// `find`'s arguments after `box`, each as the Python defaults it: pass what differs with
/// `..Options::default()`. A callable that is None is the Python's default lambda.
pub struct Options<'a> {
    pub heading: &'a str,
    pub ledge: Option<&'a dyn Fn(Cell) -> Result<Option<&'static str>>>,
    pub toll: Option<&'a dyn Fn(Cell, Cell) -> Result<i64>>,
    pub closest: bool,
    pub joined: Option<&'a dyn Fn(Cell, Cell) -> Result<bool>>,
    pub turns: Turns,
    pub through: Option<&'a dyn Fn(&[i64], Cell, Cell) -> Result<Option<Turns>>>,
    pub over: Option<&'a dyn Fn(Cell) -> Result<bool>>,
    pub arrives: Option<&'a dyn Fn(Cell, &[i64]) -> Result<bool>>,
    pub carry: Option<&'a dyn Fn(Cell) -> Result<Option<&'static str>>>,
    pub slants: Option<&'a dyn Fn(Cell, Cell) -> Result<bool>>,
    pub slide: Option<&'a dyn Fn(Cell) -> Result<bool>>,
    pub stops: Option<&'a dyn Fn(Cell, &[i64]) -> bool>,
    pub leap: Option<&'a dyn Fn(&[i64], Cell) -> Option<Cell>>,
    pub straight: bool,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Options {
            heading: "",
            ledge: None,
            toll: None,
            closest: false,
            joined: None,
            turns: Vec::new(),
            through: None,
            over: None,
            arrives: None,
            carry: None,
            slants: None,
            slide: None,
            stops: None,
            leap: None,
            straight: true,
        }
    }
}

/// The way of the compass `going` is, as `STEPS` holds it, or "" where it is none: a diagonal,
/// or no way at all.
fn compass_way(going: &str) -> &'static str {
    STEPS
        .iter()
        .find(|(way, _)| *way == going)
        .map(|&(way, _)| way)
        .unwrap_or("")
}

/// What `functools.cache` makes of a callable: asked once for each argument. A call that raises
/// is not remembered.
struct Cached<K, V>(RefCell<HashMap<K, V>>);

impl<K: Hash + Eq, V: Copy> Cached<K, V> {
    fn new() -> Self {
        Cached(RefCell::new(HashMap::new()))
    }

    fn get(&self, key: K, compute: impl FnOnce() -> Result<V>) -> Result<V> {
        let known = self.0.borrow().get(&key).copied();
        if let Some(value) = known {
            return Ok(value);
        }
        let value = compute()?;
        self.0.borrow_mut().insert(key, value);
        Ok(value)
    }
}

/// A place in the frontier. `heapq` orders its entries as tuples, and the third is a count no two
/// share, so the order is total and any min-heap pops the entries in the order `heapq` does.
struct Entry<'a> {
    key: (i64, i64, i64),
    state: State<'a>,
}

impl PartialEq for Entry<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for Entry<'_> {}

impl PartialOrd for Entry<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        Reverse(self.key).cmp(&Reverse(other.key))
    }
}

/// The cheapest path from `start` to any of `goals`, both ends included, or None when none
/// can be walked inside `box`; with `closest`, the cheapest path to the cell nearest the goals
/// of all those it can reach instead, which is `start` alone where none is nearer, and whether
/// it got there is whether it ends in `goals`. `goals` maps each cell to the way the player
/// should face on it, or None for any way. `start` is never asked about: the player is already
/// standing on it. Nor is a goal: a door is walked round on the way to anything else, and into
/// when it is the goal. `heading` is the way the player is walking, "" when that is not known.
///
/// `ledge` says which way a cell goes down, or None. A cell that cannot be walked is a ledge,
/// crossed that way only, from the cell before it to the cell after, which must be walkable,
/// and never stood on; it is in the path all the same, so each cell is a step from the last. A
/// cell that can is a slope, stepped onto, along and off only the way it goes down. `over` says
/// which cells that cannot be walked are crossed as a ledge is, but either way: the strip of
/// shore Surf jumps the player over, getting on the water and off it. `carry` says which way a
/// cell carries the player, a current: entered any way, and left only that way, a cell at a time,
/// unless the cell that way cannot be walked. The player does not turn there, so the turns it
/// makes cost nothing. `slide` says which cells are ice: entered any way, and left only the way
/// the player came in, unless the cell that way cannot be walked, where they stop and step off
/// any way; a goal is reached only where a slide stops. `slants` says whether a step between
/// two cells that touch only at a corner can be taken, where both can be walked: Santalune
/// Gym's floor is a web whose strands run so, the cells beside them walls (`DIAGONALS`). A
/// diagonal step costs a step, is a turn like any change of way, and is never taken off a
/// current.
///
/// `toll` is what a move from one cell to the next costs on top of its steps and turns, and
/// `joined` whether a step between two cells side by side can be taken at all: one off a
/// platform onto the ground far below cannot, where both cells can be stood in.
///
/// `through` is what a step does to `turns`, the state of what the player's steps turn: the
/// turns after it, or None where it cannot be taken. Fortree Gym's doors turn as an arm is
/// walked into, and whether the next can be depends on how each has turned, so the search is
/// over every way they can be turned on the way as well. Which floor the player is on is such
/// a state too, where one floor lies over another, and `arrives` says whether a cell stood on
/// with `turns` is on the goals' floor: a goal is reached only there, not on the floor above
/// it, and with `closest` the nearest cell on it comes before any nearer on another. `stops`
/// says whether the player can stop on a cell with `turns` at all, every one where it is None:
/// one a grind rail is carrying them along cannot, nor a vine's they hold, and no way ends
/// there, at a goal or short of one. With Shalour Gym's centre shut, P to the gym leader's spot
/// said "grind the rail 5 west, then can't continue", the cell 5 along a rail of 14 being the
/// nearest the centre (spike 247).
///
/// `leap` says where the player in a cell with `turns` is carried in one move, or None: there,
/// and nowhere else, the one move from it, a step's worth and a turn, with no way of its own and
/// through `through` for its turns. A vine in Coumarine Gym, grabbed from beside it, carries the
/// player up or down a level or across a gap, landing beside its other end (spike 250).
///
/// A* over (cell, heading), since a turn has a price and the same cell entered two ways is two
/// different places to go on from. The estimate is the distance to the goals' box, which never
/// overstates, so the first goal reached is the cheapest. `straight` says the way is no
/// shorter than that distance; where `leap` can carry the player further than its price, as a
/// teleporter across Laverre Gym does for four, it is False, and the search goes by cost alone.
pub fn find(
    start: Cell,
    goals: &IndexMap<Cell, Option<&'static str>>,
    walkable: &dyn Fn(Cell) -> Result<bool>,
    r#box: Box,
    options: &Options,
) -> Result<Option<Vec<Cell>>> {
    let Options {
        heading,
        ledge,
        toll,
        closest,
        joined,
        turns,
        through,
        over,
        arrives,
        carry,
        slants,
        slide,
        stops,
        leap,
        straight,
    } = options;
    if goals.is_empty() {
        return Ok(None);
    }
    let reach = Box::around(goals.keys().copied(), 0)?;
    let open_cells: Cached<Cell, bool> = Cached::new();
    // Every step asks whether its cells are slopes, what it costs and whether it can be taken,
    // and a search through Fortree Gym's doors takes each step once for every way they stand, so
    // each is worked out once.
    let down_cache: Cached<Cell, Option<&'static str>> = Cached::new();
    let over_cache: Cached<Cell, bool> = Cached::new();
    let toll_cache: Cached<(Cell, Cell), i64> = Cached::new();
    let joined_cache: Cached<(Cell, Cell), bool> = Cached::new();
    let carry_cache: Cached<Cell, Option<&'static str>> = Cached::new();
    let slants_cache: Cached<(Cell, Cell), bool> = Cached::new();
    let slide_cache: Cached<Cell, bool> = Cached::new();
    let down = |cell: Cell| {
        down_cache.get(cell, || match ledge {
            Some(ledge) => ledge(cell),
            None => Ok(None),
        })
    };
    let over = |cell: Cell| {
        over_cache.get(cell, || match over {
            Some(over) => over(cell),
            None => Ok(false),
        })
    };
    let toll = |cell: Cell, step: Cell| {
        toll_cache.get((cell, step), || match toll {
            Some(toll) => toll(cell, step),
            None => Ok(0),
        })
    };
    let joined = |cell: Cell, step: Cell| {
        joined_cache.get((cell, step), || match joined {
            Some(joined) => joined(cell, step),
            None => Ok(true),
        })
    };
    let carry = |cell: Cell| {
        carry_cache.get(cell, || match carry {
            Some(carry) => carry(cell),
            None => Ok(None),
        })
    };
    let slants = |cell: Cell, step: Cell| {
        slants_cache.get((cell, step), || match slants {
            Some(slants) => slants(cell, step),
            None => Ok(false),
        })
    };
    let slide = |cell: Cell| {
        slide_cache.get(cell, || match slide {
            Some(slide) => slide(cell),
            None => Ok(false),
        })
    };

    let arrives = |cell: Cell, turns: &[i64]| -> Result<bool> {
        match arrives {
            Some(arrives) => arrives(cell, turns),
            None => Ok(true),
        }
    };

    let estimate = |cell: Cell| -> i64 {
        STEP * ((reach.west - cell.0).max(0).max(cell.0 - reach.east)
            + (reach.north - cell.1).max(0).max(cell.1 - reach.south))
    };

    let can_walk = |cell: Cell| {
        open_cells.get(cell, || {
            if !r#box.holds(cell) {
                return Ok(false);
            }
            Ok(goals.contains_key(&cell) || walkable(cell)?)
        })
    };

    let first: State = (start, heading, turns.clone());
    let mut spent: HashMap<State, i64> = HashMap::from([(first.clone(), 0)]);
    let mut came: HashMap<State, State> = HashMap::new();
    let mut done: HashSet<State> = HashSet::new();
    let mut order: i64 = 0;
    let mut frontier: BinaryHeap<Entry> = BinaryHeap::new();
    frontier.push(Entry {
        key: (estimate(start), 0, order),
        state: first.clone(),
    });
    order += 1;
    // On the goals' floor first, then nearest the goals' box, then cheapest: the box is the
    // goals for an edge's gap or a cell beside a thing, which is what a way that falls short is
    // walked to, and the floor under a thing is not beside it.
    let mut nearest = (!arrives(start, turns)?, estimate(start), 0, first);
    while let Some(Entry { state, .. }) = frontier.pop() {
        if state == ARRIVED {
            return Ok(Some(back(&came, &came[&ARRIVED])));
        }
        if done.contains(&state) {
            continue;
        }
        done.insert(state.clone());
        let (cell, going, turned) = (state.0, state.1, &state.2);
        let cost = spent[&state];
        let way = compass_way(going);
        let sliding = if !way.is_empty() && slide(cell)? {
            way
        } else {
            ""
        };
        // A player sliding on ice is past a cell before they can stop on it, and one riding a
        // rail past each of its cells (`stops`).
        let past = !sliding.is_empty() && {
            let (dx, dz) = step_of(sliding)?;
            can_walk((cell.0 + dx, cell.1 + dz))?
        };
        if !past && stops.is_none_or(|stops| stops(cell, turned)) {
            // `min` keeps the first of equals, so a later state replaces it only by being less,
            // and between equal estimates and costs the states themselves are compared, as tuples.
            let candidate = (!arrives(cell, turned)?, estimate(cell), cost, state.clone());
            if candidate < nearest {
                nearest = candidate;
            }
            if let Some(&face) = goals.get(&cell)
                && arrives(cell, turned)?
            {
                let arrived = cost
                    + if face.is_some_and(|way| way != going) {
                        FACE
                    } else {
                        0
                    };
                if arrived < spent.get(&ARRIVED).copied().unwrap_or(arrived + 1) {
                    spent.insert(ARRIVED, arrived);
                    came.insert(ARRIVED, state.clone());
                    frontier.push(Entry {
                        key: (arrived, 0, order),
                        state: ARRIVED,
                    });
                    order += 1;
                }
            }
        }
        let carried = carry(cell)?;
        onward(
            cell,
            turned,
            &can_walk,
            &r#box,
            &down,
            &over,
            &joined,
            &carry,
            *through,
            &slants,
            sliding,
            *leap,
            &mut |direction, step, steps, after| {
                let onward: State = (step, direction, after);
                // Python's `going != direction != carried`: both inequalities, chained.
                let turn = if !going.is_empty() && going != direction && carried != Some(direction)
                {
                    TURN
                } else {
                    0
                };
                let price = cost + STEP * steps + turn + toll(cell, step)?;
                if price < spent.get(&onward).copied().unwrap_or(price + 1) {
                    spent.insert(onward.clone(), price);
                    came.insert(onward.clone(), state.clone());
                    let left = if *straight { estimate(step) } else { 0 };
                    frontier.push(Entry {
                        key: (price + left, left, order),
                        state: onward,
                    });
                    order += 1;
                }
                Ok(())
            },
        )?;
    }
    Ok(if *closest {
        Some(back(&came, &nearest.3))
    } else {
        None
    })
}

/// Each move from `cell` as `find` takes them: its way, where it lands, and how many cells
/// on, two for a jump over a ledge or the shore. On a current, the one cell it carries the
/// player to, where that can be walked, and on ice the player is `sliding` on, the one cell on
/// that way, where it can be walked. After the four straight moves, each diagonal one `slants`
/// allows. Handed to `each` one at a time, as Python's generator yields them, so a move is taken
/// before the next is tried.
#[allow(clippy::too_many_arguments)]
fn moves(
    cell: Cell,
    can_walk: &dyn Fn(Cell) -> Result<bool>,
    r#box: &Box,
    down: &dyn Fn(Cell) -> Result<Option<&'static str>>,
    over: &dyn Fn(Cell) -> Result<bool>,
    joined: &dyn Fn(Cell, Cell) -> Result<bool>,
    carry: &dyn Fn(Cell) -> Result<Option<&'static str>>,
    slants: &dyn Fn(Cell, Cell) -> Result<bool>,
    sliding: &'static str,
    each: &mut dyn FnMut(&'static str, Cell, i64) -> Result<()>,
) -> Result<()> {
    // Python's `carry(cell) or sliding or None`.
    let carried = match carry(cell)? {
        Some(way) => Some(way),
        None if !sliding.is_empty() => Some(sliding),
        None => None,
    };
    if let Some(carried) = carried {
        let (dx, dz) = step_of(carried)?;
        let ahead = (cell.0 + dx, cell.1 + dz);
        if can_walk(ahead)? {
            return each(carried, ahead, 1);
        }
    }
    for (direction, (dx, dz)) in STEPS {
        let mut step = (cell.0 + dx, cell.1 + dz);
        let mut steps = 1;
        let unfit = |way: Option<&str>| way.is_some_and(|way| way != direction);
        if !can_walk(step)? {
            if !r#box.holds(step) || (down(step)? != Some(direction) && !over(step)?) {
                continue;
            }
            step = (step.0 + dx, step.1 + dz);
            steps = 2;
            if !can_walk(step)? {
                continue;
            }
        } else if unfit(down(step)?) || unfit(down(cell)?) || !joined(cell, step)? {
            continue;
        }
        each(direction, step, steps)?;
    }
    for (direction, (dx, dz)) in DIAGONALS {
        let step = (cell.0 + dx, cell.1 + dz);
        if can_walk(step)? && slants(cell, step)? {
            each(direction, step, 1)?;
        }
    }
    Ok(())
}

/// Each move a search takes from `cell` with `turned`, `find`'s and `reached`'s alike: its
/// way, where it lands, how many cells on, and the turns after it (`through`), as they were
/// over a jump; a move `through` refuses is not taken. Where `leap` carries the player, that is
/// the one move, with no way.
#[allow(clippy::too_many_arguments)]
fn onward(
    cell: Cell,
    turned: &[i64],
    can_walk: &dyn Fn(Cell) -> Result<bool>,
    r#box: &Box,
    down: &dyn Fn(Cell) -> Result<Option<&'static str>>,
    over: &dyn Fn(Cell) -> Result<bool>,
    joined: &dyn Fn(Cell, Cell) -> Result<bool>,
    carry: &dyn Fn(Cell) -> Result<Option<&'static str>>,
    through: Option<&dyn Fn(&[i64], Cell, Cell) -> Result<Option<Turns>>>,
    slants: &dyn Fn(Cell, Cell) -> Result<bool>,
    sliding: &'static str,
    leap: Option<&dyn Fn(&[i64], Cell) -> Option<Cell>>,
    each: &mut dyn FnMut(&'static str, Cell, i64, Turns) -> Result<()>,
) -> Result<()> {
    if let Some(leap) = leap
        && let Some(to) = leap(turned, cell)
    {
        let after = match through {
            None => Some(turned.to_vec()),
            Some(through) => through(turned, cell, to)?,
        };
        if let Some(after) = after
            && can_walk(to)?
        {
            each("", to, 1, after)?;
        }
        return Ok(());
    }
    moves(
        cell,
        can_walk,
        r#box,
        down,
        over,
        joined,
        carry,
        slants,
        sliding,
        &mut |direction, step, steps| {
            let after = match through {
                Some(through) if steps != 2 => through(turned, cell, step)?,
                _ => Some(turned.to_vec()),
            };
            if let Some(after) = after {
                each(direction, step, steps, after)?;
            }
            Ok(())
        },
    )
}

/// `reached`'s keyword arguments, each as the Python defaults it.
#[derive(Default)]
pub struct Reach<'a> {
    pub ledge: Option<&'a dyn Fn(Cell) -> Result<Option<&'static str>>>,
    pub joined: Option<&'a dyn Fn(Cell, Cell) -> Result<bool>>,
    pub over: Option<&'a dyn Fn(Cell) -> Result<bool>>,
    pub ends: Option<&'a dyn Fn(Cell) -> Result<bool>>,
    pub carry: Option<&'a dyn Fn(Cell) -> Result<Option<&'static str>>>,
    pub turns: Turns,
    pub through: Option<&'a dyn Fn(&[i64], Cell, Cell) -> Result<Option<Turns>>>,
    pub slants: Option<&'a dyn Fn(Cell, Cell) -> Result<bool>>,
    pub slide: Option<&'a dyn Fn(Cell) -> Result<bool>>,
    pub stops: Option<&'a dyn Fn(Cell, &[i64]) -> bool>,
    pub leap: Option<&'a dyn Fn(&[i64], Cell) -> Option<Cell>>,
}

/// Every cell inside `box` a way from `start` can end on, by `find`'s moves, with each state
/// of `turns` it can be reached in (`through`, as `find` has them): a cell that can be walked, or
/// one of `ends`, which a way may step into and not on from, as `find` steps into a goal it would
/// walk round on the way to anything else. What `find` would reach from `start`, found once for
/// every goal at a time. On ice (`slide`), the way the player slides is part of where they are,
/// and a way ends only where a slide stops, as `find`'s does; nor where `stops` says the player
/// cannot, along a grind rail, though a thing beside one has a goal on its cell (spike 247).
/// Where `leap` carries the player, it does here too.
pub fn reached(
    start: Cell,
    walkable: &dyn Fn(Cell) -> Result<bool>,
    r#box: Box,
    options: &Reach,
) -> Result<IndexMap<Cell, HashSet<Turns>>> {
    let down_cache: Cached<Cell, Option<&'static str>> = Cached::new();
    let walk_cache: Cached<Cell, bool> = Cached::new();
    let walked_cache: Cached<Cell, bool> = Cached::new();
    let carry_cache: Cached<Cell, Option<&'static str>> = Cached::new();
    let slants_cache: Cached<(Cell, Cell), bool> = Cached::new();
    let slide_cache: Cached<Cell, bool> = Cached::new();
    let down = |cell: Cell| {
        down_cache.get(cell, || match options.ledge {
            Some(ledge) => ledge(cell),
            None => Ok(None),
        })
    };
    let carry = |cell: Cell| {
        carry_cache.get(cell, || match options.carry {
            Some(carry) => carry(cell),
            None => Ok(None),
        })
    };
    let slants = |cell: Cell, step: Cell| {
        slants_cache.get((cell, step), || match options.slants {
            Some(slants) => slants(cell, step),
            None => Ok(false),
        })
    };
    let slide = |cell: Cell| {
        slide_cache.get(cell, || match options.slide {
            Some(slide) => slide(cell),
            None => Ok(false),
        })
    };
    let walk = |cell: Cell| walk_cache.get(cell, || walkable(cell));
    let can_walk = |cell: Cell| {
        walked_cache.get(cell, || {
            Ok(r#box.holds(cell)
                && (walk(cell)?
                    || match options.ends {
                        Some(ends) => ends(cell)?,
                        None => false,
                    }))
        })
    };
    let over = |cell: Cell| match options.over {
        Some(over) => over(cell),
        None => Ok(false),
    };
    let joined = |cell: Cell, step: Cell| match options.joined {
        Some(joined) => joined(cell, step),
        None => Ok(true),
    };
    let mut out: IndexMap<Cell, HashSet<Turns>> =
        IndexMap::from([(start, HashSet::from([options.turns.clone()]))]);
    let mut seen: HashSet<(Cell, Turns, &'static str)> =
        HashSet::from([(start, options.turns.clone(), "")]);
    let mut todo: Vec<(Cell, Turns, &'static str)> = vec![(start, options.turns.clone(), "")];
    let mut at = 0;
    while at < todo.len() {
        let (cell, turned, sliding) = todo[at].clone();
        at += 1;
        if cell != start && !walk(cell)? {
            continue;
        }
        onward(
            cell,
            &turned,
            &can_walk,
            &r#box,
            &down,
            &over,
            &joined,
            &carry,
            options.through,
            &slants,
            sliding,
            options.leap,
            &mut |direction, step, _, after| {
                let way = compass_way(direction);
                let on = if !way.is_empty() && slide(step)? {
                    way
                } else {
                    ""
                };
                let state = (step, after.clone(), on);
                if seen.contains(&state) {
                    return Ok(());
                }
                seen.insert(state.clone());
                todo.push(state);
                // A cell slid over is no end: the player is past it before they can stop. Nor is a
                // rail's, ridden.
                let past = !on.is_empty() && {
                    let (dx, dz) = step_of(on)?;
                    can_walk((step.0 + dx, step.1 + dz))?
                };
                if !past && options.stops.is_none_or(|stops| stops(step, &after)) {
                    out.entry(step).or_default().insert(after);
                }
                Ok(())
            },
        )?;
    }
    Ok(out)
}

/// A way from `start` that steps on every cell of `once` exactly once and ends on one of
/// `ends`, crossing `free` cells as often as it likes, both ends included; None where there is
/// none, or where `COVER_LIMIT` cells have been tried without finding one. Thin ice, which cracks
/// under a step and gives way under the next, is crossed so, solid floor among it free.
///
/// A search in depth, the next cell taken first where it is furthest from `ends`, then where it
/// has the fewest ways on (Warnsdorff's rule), and turned back where what is left can no longer
/// be walked in one go: a cell of `once` with no way in, cut off from the player, or a second
/// with only one way, which only the last can have. By Warnsdorff's rule alone the Gym's top
/// room was not crossed in 400,000 cells tried; furthest first, every room is crossed from
/// every way onto it in a few hundred (spike 167).
pub fn cover(
    start: Cell,
    once: impl IntoIterator<Item = Cell>,
    free: impl IntoIterator<Item = Cell>,
    ends: impl IntoIterator<Item = Cell>,
) -> Option<Vec<Cell>> {
    cover_with(start, once, free, ends, COVER_LIMIT)
}

/// `cover(start, once, free, ends, limit)`.
pub fn cover_with(
    start: Cell,
    once: impl IntoIterator<Item = Cell>,
    free: impl IntoIterator<Item = Cell>,
    ends: impl IntoIterator<Item = Cell>,
    limit: i64,
) -> Option<Vec<Cell>> {
    let mut search = Cover {
        left: once.into_iter().collect(),
        free: free.into_iter().collect(),
        ends: ends.into_iter().collect(),
        tried: 0,
        limit,
    };
    search.left.remove(&start);
    let found = search.walk(start);
    found.map(|found| {
        let mut path = vec![start];
        path.extend(found);
        path
    })
}

/// `cover`'s sets and its count of the cells tried, which Python's nested functions share. The
/// sets are only tested and counted, never iterated into an answer, so a hash set's order is
/// never seen.
struct Cover {
    left: HashSet<Cell>,
    free: HashSet<Cell>,
    ends: HashSet<Cell>,
    tried: i64,
    limit: i64,
}

impl Cover {
    fn beside(cell: Cell) -> Vec<Cell> {
        STEPS
            .iter()
            .map(|&(_, (dx, dz))| (cell.0 + dx, cell.1 + dz))
            .collect()
    }

    fn away(&self, cell: Cell) -> i64 {
        self.ends
            .iter()
            .map(|end| (cell.0 - end.0).abs() + (cell.1 - end.1).abs())
            .min()
            .unwrap_or(0)
    }

    /// The free cells walked to from `cell` through free cells alone, each with the cell before
    /// it, in the order they were reached.
    fn reach(&self, cell: Cell) -> IndexMap<Cell, Option<Cell>> {
        let mut came: IndexMap<Cell, Option<Cell>> = IndexMap::from([(cell, None)]);
        let mut queue = vec![cell];
        let mut at = 0;
        while at < queue.len() {
            let here = queue[at];
            at += 1;
            for near in Cover::beside(here) {
                if self.free.contains(&near) && !came.contains_key(&near) {
                    came.insert(near, Some(here));
                    queue.push(near);
                }
            }
        }
        came
    }

    /// The ways into and out of a cell still to step on: one for each such cell beside it, two
    /// for free floor beside it, which is entered and left as often as needed, and one for the
    /// player beside it.
    fn links(&self, cell: Cell, near_player: &HashSet<Cell>) -> i64 {
        let around = Cover::beside(cell);
        around
            .iter()
            .filter(|near| self.left.contains(near))
            .count() as i64
            + 2 * around.iter().any(|near| self.free.contains(near)) as i64
            + around.iter().any(|near| near_player.contains(near)) as i64
    }

    fn stuck(&self, near_player: &HashSet<Cell>) -> bool {
        let ways: Vec<(Cell, i64)> = self
            .left
            .iter()
            .map(|&cell| (cell, self.links(cell, near_player)))
            .collect();
        if ways.iter().any(|&(_, count)| count == 0) {
            return true;
        }
        let ends_here: Vec<Cell> = ways
            .iter()
            .filter(|&&(cell, count)| {
                count == 1
                    && !Cover::beside(cell)
                        .iter()
                        .any(|near| near_player.contains(near))
            })
            .map(|&(cell, _)| cell)
            .collect();
        if ends_here.len() > 1 || (!ends_here.is_empty() && !self.ends.contains(&ends_here[0])) {
            return true;
        }
        let mut seen: HashSet<Cell> = near_player.clone();
        let mut queue: Vec<Cell> = seen.iter().copied().collect();
        let mut at = 0;
        while at < queue.len() {
            let here = queue[at];
            at += 1;
            for near in Cover::beside(here) {
                if (self.left.contains(&near) || self.free.contains(&near)) && !seen.contains(&near)
                {
                    seen.insert(near);
                    queue.push(near);
                }
            }
        }
        !self.left.is_subset(&seen)
    }

    fn walk(&mut self, cell: Cell) -> Option<Vec<Cell>> {
        if self.left.is_empty() {
            return if self.ends.contains(&cell) {
                Some(Vec::new())
            } else {
                None
            };
        }
        self.tried += 1;
        if self.tried > self.limit {
            return None;
        }
        let came = self.reach(cell);
        if self.stuck(&came.keys().copied().collect()) {
            return None;
        }
        let mut options: IndexMap<Cell, Vec<Cell>> = IndexMap::new();
        for &here in came.keys() {
            for near in Cover::beside(here) {
                if self.left.contains(&near) && !options.contains_key(&near) {
                    let mut route = vec![near];
                    let mut back = Some(here);
                    while let Some(at) = back
                        && at != cell
                    {
                        route.push(at);
                        back = came[&at];
                    }
                    route.reverse();
                    options.insert(near, route);
                }
            }
        }
        let none = HashSet::new();
        let mut order: Vec<(i64, i64, Cell)> = options
            .keys()
            .map(|&near| (-self.away(near), self.links(near, &none), near))
            .collect();
        order.sort();
        for (_, _, step) in order {
            self.left.remove(&step);
            let rest = self.walk(step);
            self.left.insert(step);
            if let Some(rest) = rest {
                let mut way = options[&step].clone();
                way.extend(rest);
                return Some(way);
            }
        }
        None
    }
}

fn back(came: &HashMap<State, State>, state: &State) -> Vec<Cell> {
    let mut state = state;
    let mut cells = vec![state.0];
    while let Some(was) = came.get(state) {
        // A jump comes two cells from the last, in a line: the ledge between goes in too. A
        // diagonal step is two cells off by the same count and jumps nothing, nor a leap, which
        // has no way.
        let ((x, z), (x0, z0)) = (state.0, was.0);
        if !state.1.is_empty() && (x - x0).abs() + (z - z0).abs() == 2 && (x == x0 || z == z0) {
            cells.push((py::floordiv(x + x0, 2), py::floordiv(z + z0, 2)));
        }
        state = was;
        cells.push(state.0);
    }
    cells.reverse();
    cells
}

/// `legs(path, how)`, with no step walked by a key other than its way's.
pub fn legs(path: &[Cell], how: Option<&dyn Fn(Cell, Cell) -> Result<String>>) -> Result<Vec<Leg>> {
    legs_with(path, how, None, None)
}

/// A path's straight runs, in order. `how` says what each step is walked with, and a run
/// ends where that changes as well as where the path turns: "3 south, Surf 12 south". `key` is
/// the key that walks a step where that is not its way, as up walks a street north-west.
/// `placed` says both for a step by its place in the path, the index of its last cell, where
/// its two cells do not: in Coumarine Gym the way climbs the vine from (59, 73) to (59, 72) at
/// 0 and later walks over the same two cells at 720 (spike 250).
pub fn legs_with(
    path: &[Cell],
    how: Option<&dyn Fn(Cell, Cell) -> Result<String>>,
    key: Option<&dyn Fn(Cell, Cell) -> Option<&'static str>>,
    placed: Option<&IndexMap<i64, (String, Option<&'static str>)>>,
) -> Result<Vec<Leg>> {
    let mut out: Vec<Leg> = Vec::new();
    for i in 1..path.len() {
        let (walked, keyed) = match placed.and_then(|placed| placed.get(&(i as i64))) {
            Some((walked, keyed)) => (walked.clone(), *keyed),
            None => (
                match how {
                    Some(how) => how(path[i - 1], path[i])?,
                    None => String::new(),
                },
                key.and_then(|key| key(path[i - 1], path[i])),
            ),
        };
        let mut direction = keyed.unwrap_or("");
        if direction.is_empty() {
            direction = towards(path[i - 1], path[i]);
        }
        if direction.is_empty() {
            direction = slant((path[i].0 - path[i - 1].0, path[i].1 - path[i - 1].1)).unwrap_or("");
        }
        match out.last_mut() {
            Some(last) if last.direction == direction && last.how == walked => {
                *last = Leg {
                    direction,
                    steps: last.steps + 1,
                    end: i as i64,
                    how: walked,
                    held: None,
                    vined: false,
                    teleported: false,
                };
            }
            _ => out.push(Leg {
                direction,
                steps: 1,
                end: i as i64,
                how: walked,
                held: None,
                vined: false,
                teleported: false,
            }),
        }
    }
    Ok(out)
}

/// The compass's eight ways, clockwise.
pub const EIGHTHS: [&str; 8] = [
    NORTH,
    "north-east",
    EAST,
    "south-east",
    SOUTH,
    "south-west",
    WEST,
    "north-west",
];

thread_local! {
    /// Quarters clockwise from the way the pad's up walks to north (`walked`).
    ///
    /// Python's `_turn`, renamed since `TURN` is taken, and the module's own where this is a
    /// thread's, as `core::words`' words are, so tests running side by side do not turn each
    /// other's compass. The runtime has one thread.
    static TURNED: Slot<i64> = const { Slot::new(0) };
}

/// Say every way of the compass from here on as the pad key that walks it, where up walks the
/// player `turn` quarters clockwise from north: in Parfum Palace's garden right walked Khai
/// north and up west, 3 (x_route-6-4.ctm, 2026-10-20). North is the way up walks, as in Sun.
pub fn walked(turn: i64) {
    TURNED.with(|held| held.set(py::modulo(turn, 4)));
}

/// A way of the compass, `north` or `north-east`, or of a player's own frame, `ahead`, as
/// it is said: a way of the compass as the key that walks it (`walked`).
pub fn way(direction: &str) -> String {
    let turn = TURNED.with(Slot::get);
    let mut direction = direction;
    if turn != 0
        && let Some(at) = EIGHTHS.iter().position(|&known| known == direction)
    {
        direction = EIGHTHS[py::modulo(at as i64 - 2 * turn, EIGHTHS.len() as i64) as usize];
    }
    own(&format!("way-{direction}"), &[])
}

/// Legs as they are said: "5 east, 2 north".
pub fn spoken(runs: &[Leg]) -> String {
    runs.iter()
        .map(|leg| leg.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Legs as they are said to a player whose keys work from a camera behind them, as on
/// Lumiose's boulevards: up walks the way the camera looks, down walks back from it without
/// turning it, and left and right turn the player a quarter, the camera following only while
/// the key is held (spike 105). So a leg to one side is a hold, then the steps forward, and a
/// leg behind is steps back: "hold right, 5 forward, 3 back", "hold left, 4 forward".
///
/// `angled` is a camera a tap left pointing between two ways, where neither up nor down walks
/// a leg straight. A hold always settles it on a way, the one a quarter from the way it is
/// nearest, so a first leg to one side is said as ever, and one ahead or behind after holding
/// one way and back: "camera angled, hold left, hold right, 5 forward".
///
/// `turning` is a camera that turns with the player, as in Glittering Cave: left and right turn
/// them a quarter and down turns them round, the camera swinging after them, and only up walks
/// (spike 239). So every leg not ahead is a turn, then the steps forward: "turn left, 5 forward",
/// "turn around, 1 forward". The camera settles on its own, so `angled` is ignored.
pub fn turned(runs: &[Leg], heading: &str, angled: bool, turning: bool) -> Result<String> {
    let mut heading = heading;
    let mut words: Vec<String> = Vec::new();
    for (i, leg) in runs.iter().enumerate() {
        let turn = py::modulo(
            compass_index(leg.direction)? - compass_index(heading)?,
            COMPASS.len() as i64,
        );
        if turning {
            if turn != 0 {
                words.push(own(
                    ["turn-right", "turn-around", "turn-left"][turn as usize - 1],
                    &[],
                ));
            }
            words.push(own("rail-forward", &[("steps", &leg.steps)]));
            heading = leg.direction;
            continue;
        }
        if i == 0 && angled && (turn == 0 || turn == 2) {
            words.push(own("rail-angled", &[]));
        }
        if turn == 2 {
            words.push(own("rail-back", &[("steps", &leg.steps)]));
            continue;
        }
        if turn != 0 {
            words.push(own(
                if turn == 1 {
                    "rail-hold-right"
                } else {
                    "rail-hold-left"
                },
                &[],
            ));
        }
        words.push(own("rail-forward", &[("steps", &leg.steps)]));
        heading = leg.direction;
    }
    Ok(words.join(", "))
}

/// The way of the compass nearest a bearing in degrees clockwise from north.
pub fn nearest(bearing: f64) -> Result<&'static str> {
    Ok(COMPASS[py::modulo(py::round(bearing / 90.0)?, COMPASS.len() as i64) as usize])
}

/// How far a bearing in degrees clockwise from north is turned from `heading`, clockwise
/// positive, between -180 and 180.
pub fn off(bearing: f64, heading: &str) -> Result<f64> {
    Ok(py::modulo_f(
        bearing - 90.0 * compass_index(heading)? as f64 + 180.0,
        360.0,
    ) - 180.0)
}

/// Where `cell` is from `origin` for a player heading one way: "9 ahead 8 left", "3 behind",
/// or "here".
pub fn facing(origin: Cell, cell: Cell, heading: &str) -> Result<String> {
    let (dx, dz) = (cell.0 - origin.0, cell.1 - origin.1);
    let (ahead_x, ahead_z) = step_of(heading)?;
    let (right_x, right_z) =
        step_of(COMPASS[py::modulo(compass_index(heading)? + 1, COMPASS.len() as i64) as usize])?;
    let (ahead, right) = (dx * ahead_x + dz * ahead_z, dx * right_x + dz * right_z);
    let mut parts: Vec<String> = Vec::new();
    if ahead != 0 {
        parts.push(own(
            "steps",
            &[
                ("count", &ahead.abs()),
                ("way", &way(if ahead > 0 { AHEAD } else { BEHIND })),
            ],
        ));
    }
    if right != 0 {
        parts.push(own(
            "steps",
            &[
                ("count", &right.abs()),
                ("way", &way(if right > 0 { RIGHT } else { LEFT })),
            ],
        ));
    }
    Ok(if parts.is_empty() {
        own("here", &[])
    } else {
        parts.join(" ")
    })
}

/// Which way `cell` lies from `origin` in a straight line, or "" when it is not in one.
pub fn towards(origin: Cell, cell: Cell) -> &'static str {
    let (dx, dz) = (cell.0 - origin.0, cell.1 - origin.1);
    if dx != 0 && dz == 0 {
        return if dx > 0 { EAST } else { WEST };
    }
    if dz != 0 && dx == 0 {
        return if dz > 0 { SOUTH } else { NORTH };
    }
    ""
}
