//! Where the player can walk: the chunks' cells, the grid of zones, a rail's corridor, rooms of
//! thin ice and rotating doors, and the moves that clear the way.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::pathfind::{self, Box, Cell, EAST, NORTH, SOUTH, WEST};
use crate::games::common::field::chunk::{
    ANY_FLOOR, BLOCKED, BOARD, CHUNK, CLIMB, COUNTER, Chunk, SHORE,
};
use crate::games::common::field::things::{
    Field, Interactable, POKEMON, TRIGGER_SOURCE, WARP_SOURCE,
};
use crate::py::{self, Error, Result};

/// What the ground asks of a rails' corridor (`gen6.field.rail.Corridor`): its own walls are the only
/// ones there, and it is the whole of where a way may be found.
pub trait Corridor {
    fn open(&self, cell: Cell) -> Result<bool>;

    fn r#box(&self) -> Result<Box>;
}

pub const CUT: i64 = 15;

pub const SURF: i64 = 57;

pub const STRENGTH: i64 = 70;

pub const WATERFALL: i64 = 127;

/// The moves that open a way, by their numbers, the same in every game and each its line in the
/// move names: "Surf", "なみのり" in both ORAS and X (spike 136).
pub const ROCK_SMASH: i64 = 249;

/// What a step through a big boulder costs on top of its steps, where a bush or a rock costs
/// `BOARD`: five hundred steps' worth, so Shift and P goes round a boulder wherever a way round
/// lies, and through one only where none does (Orion, 2026-10-11, in Seafloor Cavern). Where a
/// push leaves the boulder is not modelled, and one pushed the wrong way can shut the way it was
/// pushed to open.
pub const SHOVE: i64 = 1000;

/// The ways a big boulder standing in `cells` moves when pushed, each with whether that push
/// drops it into a hole, in `pathfind.STEPS`' order: where every cell it moves into is `floor` or
/// a hole, and a cell on the side it is pushed from is `floor` to push it from. It drops once all
/// four of its cells are a hole's. So 32 of the 37 pushes into a boulder in movie37 went, and the
/// other five were taps too short to push (spike 202). `floor` is where the player could stand:
/// open, and nobody in it.
///
/// ponytail: water, ledges and a floor's height are not looked at; Seafloor Cavern's rooms have
/// none of them round a boulder.
pub fn pushes(
    cells: impl IntoIterator<Item = Cell>,
    floor: &dyn Fn(Cell) -> Result<bool>,
    holes: &HashSet<Cell>,
) -> Result<Vec<(&'static str, bool)>> {
    let own: HashSet<Cell> = cells.into_iter().collect();
    let mut out: Vec<(&'static str, bool)> = Vec::new();
    for (way, (dx, dz)) in pathfind::STEPS {
        let moved: HashSet<Cell> = own.iter().map(|&(x, z)| (x + dx, z + dz)).collect();
        let behind: HashSet<Cell> = own
            .iter()
            .map(|&(x, z)| (x - dx, z - dz))
            .filter(|cell| !own.contains(cell))
            .collect();
        let mut goes = true;
        for &cell in moved.difference(&own) {
            if !(holes.contains(&cell) || floor(cell)?) {
                goes = false;
                break;
            }
        }
        if goes {
            let mut from = false;
            for &cell in &behind {
                if floor(cell)? {
                    from = true;
                    break;
                }
            }
            goes = from;
        }
        if goes {
            out.push((way, moved.is_subset(holes)));
        }
    }
    Ok(out)
}

/// Cells to a block of the matrix.
pub const BLOCK: i64 = 10;

pub const NO_ZONE: i64 = 0xFFFF;

/// Cells past the loaded chunks, the player and the goal that a path may go round through,
/// none of them known.
pub const SEARCH_MARGIN: i64 = 10;

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

/// What a cell of a room of thin ice is: a tile not yet walked onto, a tile that gives way under
/// the next step onto it, or floor that is no ice at all.
pub const WHOLE: i64 = 0;

pub const CRACKED: i64 = 1;

pub const SOLID: i64 = 2;

/// One of Sootopolis Gym's rooms of thin ice, as the Gym's module keeps it (spike 167). A tile
/// walked onto cracks, and stepping onto a cracked one drops the player to the floor below and
/// makes the room whole again. The room is cleared once every tile is cracked with the player on
/// one of its exits, and its ice is safe from then on.
#[derive(Debug, Clone, PartialEq)]
pub struct IceRoom {
    /// The north-west cell.
    pub corner: Cell,
    pub size: (i64, i64),
    /// How high its floor is, which says the floor of the Gym's chunks it is on.
    pub height: f64,
    /// `WHOLE`, `CRACKED` or `SOLID` a cell, row by row from the corner.
    pub states: Vec<i64>,
    /// The cells it is left by, onto the corridor up from it: the last tile cracked is one.
    pub exits: Vec<Cell>,
    pub cleared: bool,
}

impl IceRoom {
    /// A room not yet cleared; `cleared` is set with `..IceRoom::new(...)`.
    pub fn new(
        corner: Cell,
        size: (i64, i64),
        height: f64,
        states: Vec<i64>,
        exits: Vec<Cell>,
    ) -> Self {
        IceRoom {
            corner,
            size,
            height,
            states,
            exits,
            cleared: false,
        }
    }

    /// Each cell of the room with what it is.
    pub fn cells(&self) -> IndexMap<Cell, i64> {
        let wide = self.size.0;
        self.states
            .iter()
            .enumerate()
            .map(|(i, &state)| {
                let i = i as i64;
                (
                    (
                        self.corner.0 + py::modulo(i, wide),
                        self.corner.1 + py::floordiv(i, wide),
                    ),
                    state,
                )
            })
            .collect()
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

type Crossings = HashMap<(Cell, Cell), i64>;

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

/// The field map's zone ids, `width` blocks to a row.
#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub width: i64,
    pub height: i64,
    pub zones: Vec<i64>,
}

impl Grid {
    /// The zone id of a block, `NO_ZONE` off the grid.
    pub fn at(&self, column: i64, row: i64) -> Result<i64> {
        if !(0 <= column && column < self.width) || !(0 <= row && row < self.height) {
            return Ok(NO_ZONE);
        }
        match usize::try_from(row * self.width + column)
            .ok()
            .and_then(|at| self.zones.get(at))
        {
            Some(&zone) => Ok(zone),
            None => Err(Error::index("tuple index out of range")),
        }
    }
}

/// The chunk at a (column, row) of the matrix as the ROM has it, or None: read where no
/// loaded chunk holds a cell, so a way to something far off is not a guess. P's way to a
/// Fisherman across the sea on Route 109 walked where the sea was not loaded, and was given up
/// once it was (omega_ruby_surf_2.ctm, spike 136). None without the ROM's chunks to read.
pub type Far = Rc<dyn Fn(i64, i64) -> Result<Option<Rc<Chunk>>>>;

/// Where the player can walk, as far as the map loaded now says: what pathfinding walks on.
///
/// A cell cannot be walked where its chunk's blocked bit is set — a tree, a wall, a door, a sign,
/// a ledge — where the matrix has no zone, or where a move model stands. A ledge is jumped the
/// way it drops, from the cell before it to the cell after, and a slope, which is open, walked
/// only the way it goes down, as a conveyor belt is the way it runs (`ledge`); a current in the
/// sea carries a player on it its own way (`carry`). Only the four chunks around the player are loaded; the rest of the matrix is the ROM's (`far`). A cell
/// neither holds is taken to be open, and a path that turns out to cross a wall is found again
/// once the chunk with the wall in it loads.
/// Water is a wall too, unless the ground is one to `surf`, and a bush or a rock a move clears
/// unless it is one to clear them (`hm`), and a waterfall unless it is one to climb it (`falls`).
#[derive(Clone)]
pub struct Ground {
    pub chunks: Vec<Rc<Chunk>>,
    pub grid: Option<Grid>,
    /// Cells somebody or something stands in: a person, an item ball, a prop.
    pub standing: IndexSet<Cell>,
    /// Cells that do something when walked onto: a warp takes the player off the map, and a
    /// hidden Pokémon jumps out. Walked round, unless one is where the player is going.
    pub stepped: IndexSet<Cell>,
    /// The rails' corridor the player is in, whose own walls are the only ones there: a cell
    /// no line of it holds is not somewhere on these streets.
    pub corridor: Option<Rc<dyn Corridor>>,
    /// How many cells out from a solid thing a path may end, looking out from each side until
    /// one is open. One in ORAS, where a thing is its tiles. Sun's props are wider than the records
    /// their things are read by: the cells beside a flower pot's record were inside the pot, and P
    /// never said the player had arrived (sun_5.ctm, 2026-09-26).
    pub reach: i64,
    /// Cells of a conveyor belt, each with the way it runs: open, and walked only that way, as a
    /// slope is. The cell table has plain floor there; the belt is a trigger (spike 132).
    pub belts: IndexMap<Cell, &'static str>,
    /// Whether water is crossed, as Surf crosses it: Shift and P's ground, and P's for a player
    /// already on the water. P walked into the sea on Route 109 while the blocked bit was all it
    /// read (omega_ruby_surf.ctm, spike 136).
    pub surf: bool,
    /// The chunk at a (column, row) of the matrix as the ROM has it, or None: read where no
    /// loaded chunk holds a cell, so a way to something far off is not a guess. P's way to a
    /// Fisherman across the sea on Route 109 walked where the sea was not loaded, and was given up
    /// once it was (omega_ruby_surf_2.ctm, spike 136). None without the ROM's chunks to read.
    pub far: Option<Far>,
    /// Cells a thing a move clears stands in, each with the move: a bush Cut clears, a rock Rock
    /// Smash does, a boulder Strength pushes. Stood in like any other thing, and walked through
    /// where the ground is one to `hm`.
    pub cleared: IndexMap<Cell, i64>,
    /// The cells of each big boulder whose pushes are known, with the ways a push moves it: a
    /// step into one goes only those ways (`joined`), and into a boulder no push moves not at
    /// all. In Victory Road the player pushed one into the foot of a corridor, where it stuck,
    /// and Shift and P went on saying "Strength 2 south" through it (movie46, spike 213).
    ///
    /// A `frozenset` is a `HashSet`: it is only tested.
    pub shoves: IndexMap<Cell, HashSet<&'static str>>,
    /// Whether the way goes through what a move clears, and up a slope: Shift and P's ground.
    pub hm: bool,
    /// The cells of each hole a big boulder has dropped into: floor, though the chunk still has
    /// them blocked (`HOLE`, spike 202).
    pub filled: HashSet<Cell>,
    /// Whether the player is riding the Mach Bike, which rides up a slope as well as down: P's
    /// ground goes up one only then (spike 158).
    pub bike: bool,
    /// Cells side by side, each way, that can both be stood in and are not walked between: a
    /// platform in Fortree City's trees and the ground beside it, 108 below, but for its ladder
    /// (`joined`, spike 160).
    pub cliffs: IndexSet<(Cell, Cell)>,
    /// Each step a ladder or a stair makes, each way, with how high its two ends stand: from the
    /// floor at the first, it lands on the floor at the second, whatever floors they are
    /// (`climb`). Outside Sky Pillar the terraces stand 108 apart on three floors, and the one
    /// stair up to Zinnia's goes from floor 0 at 108 to floor 1 at 216; the sand runs on under the
    /// stair from 324 down to her terrace, and is walked on as sand (spike 193).
    pub ladders: IndexMap<(Cell, Cell), (f64, f64)>,
    /// The bumpy slopes, each with the way it is hopped up on the Acro Bike, from the cell before
    /// it to the cell after, as a ledge is jumped down: walls on foot (spike 159).
    pub hops: IndexMap<Cell, &'static str>,
    /// The waterfalls, each cell with the way it is climbed: jumped over either way as the shore
    /// is, where the ground is one with HMs, and a wall to any other (`over`, spike 170).
    pub falls: IndexMap<Cell, &'static str>,
    /// Fortree Gym's rotating doors, as they stand now: walked through by turning them, a step
    /// into an arm turning its door (`through`, spike 161).
    pub doors: Vec<RotatingDoor>,
    /// Sootopolis Gym's rooms of thin ice, as they are now, their cracked tiles in `standing`
    /// on their floor (spike 167).
    pub ice: Vec<IceRoom>,
    /// Under the water, each cell of deep water the player rises to the surface through, with
    /// every one the sea above joins it to, itself among them: the sea has a dive spot over each,
    /// the two matrices the same cells a fixed way apart, so a dive spot is the same bearing from
    /// the player above the water as below. Under Route 126 rock walls the water into pockets, and
    /// the way into Sootopolis City is reached only by diving at a spot over its own pocket; the
    /// player's own light rises into a pond walled off from the sea, and the light that reaches
    /// the sea is the other in their pocket, 47 east 4 north (spike 165). None above the water.
    ///
    /// A `frozenset` is a `HashSet` in an `Rc`, which the cells of one pocket share: it is only
    /// intersected and tested, so its order is never seen.
    pub surfaces: IndexMap<Cell, Rc<HashSet<Cell>>>,
    /// Whether a chunk with a deck is walked a floor at a time, the floor a step is on part of
    /// where a search stands (`climb`): its `chunks` keep their floors, each cell open where any
    /// floor is. Sootopolis Gym's lower floor lies under its rooms, joined to the entrance's room
    /// by a stair at each side; walked on one floor a cell, the Lass pacing below stood in the
    /// corridor above her, and P led the player from the lower floor to the cell under the way
    /// out (spike 167).
    pub stacked: bool,
    /// The player's height on a stacked ground, which says the floor they start on.
    pub level: Option<f64>,
    /// How high each thing in a cell of `standing` stands, None where not known: on a stacked
    /// ground it stands on its cell's floor nearest that, and in nobody's way on the others. In
    /// Sootopolis Gym a Beauty stands in the lower floor's corridor under an invisible blocker in
    /// the corridor above (spike 167).
    pub heights: IndexMap<Cell, Vec<Option<f64>>>,
    /// The heights of the map's floors other than the player's, nearest theirs first, at which
    /// the ground can be walked instead (`FieldSource.ground`). A chunk with a deck is walked on
    /// the floor nearest one height, and from the foot of Route 119 that kept the way under its
    /// bridge and cut the bridge, the only way north (spike 159).
    pub decks: Vec<f64>,
}

impl Default for Ground {
    fn default() -> Self {
        Ground {
            chunks: Vec::new(),
            grid: None,
            standing: IndexSet::new(),
            stepped: IndexSet::new(),
            corridor: None,
            reach: 1,
            belts: IndexMap::new(),
            surf: false,
            far: None,
            cleared: IndexMap::new(),
            shoves: IndexMap::new(),
            hm: false,
            filled: HashSet::new(),
            bike: false,
            cliffs: IndexSet::new(),
            ladders: IndexMap::new(),
            hops: IndexMap::new(),
            falls: IndexMap::new(),
            doors: Vec::new(),
            ice: Vec::new(),
            surfaces: IndexMap::new(),
            stacked: false,
            level: None,
            heights: IndexMap::new(),
            decks: Vec::new(),
        }
    }
}

impl fmt::Debug for Ground {
    /// The ground without its two callables, which have nothing to show.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ground")
            .field("chunks", &self.chunks)
            .field("grid", &self.grid)
            .field("standing", &self.standing)
            .field("stepped", &self.stepped)
            .field("corridor", &self.corridor.is_some())
            .field("reach", &self.reach)
            .field("belts", &self.belts)
            .field("surf", &self.surf)
            .field("far", &self.far.is_some())
            .field("cleared", &self.cleared)
            .field("shoves", &self.shoves)
            .field("hm", &self.hm)
            .field("bike", &self.bike)
            .field("cliffs", &self.cliffs)
            .field("ladders", &self.ladders)
            .field("hops", &self.hops)
            .field("falls", &self.falls)
            .field("doors", &self.doors)
            .field("ice", &self.ice)
            .field("surfaces", &self.surfaces)
            .field("stacked", &self.stacked)
            .field("level", &self.level)
            .field("heights", &self.heights)
            .field("decks", &self.decks)
            .finish()
    }
}

/// `Ground.of`'s arguments after `where`, each as the Python defaults it: set what differs with
/// `..Of::default()`.
#[derive(Default)]
pub struct Of {
    pub chunks: Vec<Rc<Chunk>>,
    pub grid: Option<Grid>,
    pub corridor: Option<Rc<dyn Corridor>>,
    pub belts: IndexMap<Cell, &'static str>,
    pub far: Option<Far>,
    pub clears: IndexMap<i64, i64>,
    pub solid: IndexMap<Cell, Option<f64>>,
}

impl Ground {
    /// The ground under `where`'s listing: its move models stand in their cells, all of a
    /// barrier's beam, and its warps, triggers and hidden Pokémon are stepped round: a way to
    /// Wattson over one of his Gym's switches would swing a barrier shut behind the player
    /// (spike 146). A move model drawn as a sprite `clears` has is cleared by that move, a script
    /// of its own or not: three of Rusturf Tunnel's ten rocks
    /// have a sign under them and a script, and are the rocks Rock Smash breaks to join the
    /// tunnel's halves, the only way through it to Verdanturf Town (spike 137). `solid` is
    /// cells something stands in that is not listed, with how high it stands where known: a
    /// Secret Base's desks and chairs, which the cell table has as open floor (spike 154), and
    /// the game's invisible blockers (spike 167).
    pub fn of(r#where: &Field, of: Of) -> Ground {
        let Of {
            chunks,
            grid,
            corridor,
            belts,
            far,
            clears,
            solid,
        } = of;
        let models: Vec<&Interactable> = r#where
            .interactables
            .iter()
            .filter(|item| item.move_model)
            .collect();
        let mut standing: IndexSet<Cell> = solid.keys().copied().collect();
        for item in &models {
            standing.extend(item.cells());
        }
        let mut heights: IndexMap<Cell, Vec<Option<f64>>> = IndexMap::new();
        let from_models = models.iter().map(|item| (item.cells(), item.height));
        let from_solid = solid.iter().map(|(&cell, &height)| (vec![cell], height));
        for (cells, height) in from_models.chain(from_solid) {
            for cell in cells {
                heights.entry(cell).or_default().push(height);
            }
        }
        let mut stepped: IndexSet<Cell> = IndexSet::new();
        for item in &r#where.interactables {
            let place = item.place_kind();
            if item.category == POKEMON
                || place == Some(WARP_SOURCE)
                || place == Some(TRIGGER_SOURCE)
            {
                stepped.extend(item.cells());
            }
        }
        let mut cleared: IndexMap<Cell, i64> = IndexMap::new();
        for item in &r#where.interactables {
            if item.move_model
                && let Some(&clear) = clears.get(&item.code)
            {
                for cell in item.cells() {
                    cleared.insert(cell, clear);
                }
            }
        }
        let mut shoves: IndexMap<Cell, HashSet<&'static str>> = IndexMap::new();
        for item in &r#where.interactables {
            if let Some(pushes) = &item.pushes {
                for cell in item.cells() {
                    shoves.insert(cell, pushes.iter().map(|(way, _)| *way).collect());
                }
            }
        }
        Ground {
            chunks,
            grid,
            standing,
            stepped,
            corridor,
            belts,
            far,
            cleared,
            shoves,
            heights,
            ..Ground::default()
        }
    }

    /// Whether the player can stand in the cell: on a stacked ground, on any floor of it
    /// (`open_at` with no height).
    pub fn open(&self, cell: Cell) -> Result<bool> {
        self.open_at(cell, None)
    }

    /// Whether the player can stand in the cell: on a stacked ground, on any floor of it, or on
    /// its floor at `height` where one is given (`floor_at`).
    pub fn open_at(&self, cell: Cell, height: Option<f64>) -> Result<bool> {
        if let Some(corridor) = &self.corridor {
            return Ok(!self.standing.contains(&cell) && corridor.open(cell)?);
        }
        if let Some(grid) = &self.grid
            && grid.at(py::floordiv(cell.0, BLOCK), py::floordiv(cell.1, BLOCK))? == NO_ZONE
        {
            return Ok(false);
        }
        let floors = self.floors(cell)?;
        if !floors.is_empty() {
            let Some(aim) = self.floor_at(cell, height)? else {
                return Ok(false);
            };
            let mut all = true;
            for &(k, _) in &floors {
                if (aim == ANY_FLOOR || aim == k) && !self.stands(cell, k)? {
                    all = false;
                    break;
                }
            }
            if all {
                return Ok(false);
            }
        } else if self.stands(cell, ANY_FLOOR)? {
            return Ok(false);
        }
        let Some(chunk) = self.chunk(cell)? else {
            return Ok(true);
        };
        Ok((chunk.walkable(cell)? || self.filled.contains(&cell))
            && (self.surf || !chunk.water(cell)?))
    }

    /// On a stacked ground, each floor open in the cell, by its place among its chunk's floors,
    /// with its height there; none where the ground is not stacked or the chunk has one floor.
    /// A floor with no height, its mesh over none of the cell, is not one where another has a
    /// height: there is nothing there to stand on, and Fortree City's ground at the end of a bridge
    /// was one, which P took from the bridge at 103 down onto the ground at 0
    /// (omega_ruby_tree-city-regression.ctm, 2026-10-06).
    pub fn floors(&self, cell: Cell) -> Result<Vec<(i64, Option<f64>)>> {
        let chunk = if self.stacked {
            self.chunk(cell)?
        } else {
            None
        };
        let Some(chunk) = chunk else {
            return Ok(Vec::new());
        };
        if chunk.floors.is_empty() {
            return Ok(Vec::new());
        }
        let i = (cell.1 - chunk.origin.1) * CHUNK + cell.0 - chunk.origin.0;
        let mut out = Vec::new();
        for (k, floor) in chunk.floors.iter().enumerate() {
            let word = usize::try_from(i)
                .ok()
                .and_then(|i| floor.words.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?;
            if word & BLOCKED == 0 {
                let height = usize::try_from(i)
                    .ok()
                    .and_then(|i| floor.heights.get(i))
                    .ok_or_else(|| Error::index("tuple index out of range"))?;
                out.push((k as i64, *height));
            }
        }
        let held: Vec<(i64, Option<f64>)> = out
            .iter()
            .copied()
            .filter(|floor| floor.1.is_some())
            .collect();
        Ok(if held.is_empty() { out } else { held })
    }

    /// The cell's open floor nearest `height`, or `ANY_FLOOR` where there is none to choose: no
    /// height, or no floors (`floors`).
    pub fn floor(&self, cell: Cell, height: Option<f64>) -> Result<i64> {
        let floors = self.floors(cell)?;
        let Some(height) = height.filter(|_| !floors.is_empty()) else {
            return Ok(ANY_FLOOR);
        };
        let mut best: Option<(f64, i64)> = None;
        for &(k, there) in &floors {
            let away = match there {
                None => f64::INFINITY,
                Some(there) => (there - height).abs(),
            };
            if best.is_none_or(|(least, _)| away < least) {
                best = Some((away, k));
            }
        }
        Ok(best.expect("a cell with floors has one nearest").1)
    }

    /// The cell's floor a thing standing at `height` is reached on there (`floor`), or None
    /// where it is further from `height` than a stair's step down from beside the thing,
    /// `2 * CLIMB`, or has no height there. Sootopolis Gym's guide stands by a statue on the
    /// floor over its lower one, and beside him, where the statue is, the lower floor alone is
    /// open, 200 below him (spike 167).
    pub fn floor_at(&self, cell: Cell, height: Option<f64>) -> Result<Option<i64>> {
        let k = self.floor(cell, height)?;
        let Some(height) = height.filter(|_| k != ANY_FLOOR) else {
            return Ok(Some(k));
        };
        let there = self
            .floors(cell)?
            .into_iter()
            .find(|&(floor, _)| floor == k)
            .ok_or_else(|| Error::key(k.to_string()))?
            .1;
        Ok(there
            .is_some_and(|there| (there - height).abs() <= 2.0 * CLIMB)
            .then_some(k))
    }

    /// Whether something stands in the cell on `floor`: on every floor of it, where how high it
    /// stands is not known, and on any where `floor` is `ANY_FLOOR`.
    pub fn stands(&self, cell: Cell, floor: i64) -> Result<bool> {
        if !self.standing.contains(&cell) || (self.hm && self.cleared.contains_key(&cell)) {
            return Ok(false);
        }
        if floor == ANY_FLOOR {
            return Ok(true);
        }
        let none = [None];
        let heights = match self.heights.get(&cell) {
            Some(heights) => heights.as_slice(),
            None => &none,
        };
        for &height in heights {
            if height.is_none() {
                return Ok(true);
            }
            let on = self.floor(cell, height)?;
            if on == ANY_FLOOR || on == floor {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The floor a step from `cell`, walked on `floor`, lands on at `step`: of its open floors
    /// nobody stands on, the nearest the height left, the same floor first; None where there is
    /// none and the step is not taken, and `ANY_FLOOR` where `step` has no floors to choose
    /// between. Another floor is stepped onto only within `CLIMB`; along one floor a step is
    /// taken as on any ground, a drop being a cliff: each of Sootopolis Gym's two stairs climbs
    /// its lower floor 18 or 19 a cell to 189, and steps across onto the entrance's room at 198
    /// (spike 167), and on Route 119 the way north goes on along the bridge at 99 where the path
    /// under it has dropped to 50 (spike 159). Where either cell has no height, no step goes onto
    /// another floor: the lower floor's table has the Gym's pool's edges open with no floor under
    /// them, and P stepped off one onto the room above.
    /// A step up or down a ladder, from the floor at its foot or its top, lands on the floor at
    /// its other end (`ladders`).
    pub fn climb(&self, floor: i64, cell: Cell, step: Cell) -> Result<Option<i64>> {
        let there = self.floors(step)?;
        if there.is_empty() {
            return Ok(Some(ANY_FLOOR));
        }
        let here = self
            .floors(cell)?
            .into_iter()
            .find(|&(k, _)| k == floor)
            .and_then(|(_, height)| height);
        let ladder = self.ladders.get(&(cell, step)).copied();
        let end = match (ladder, here) {
            (Some((from, to)), Some(here)) if (here - from).abs() <= CLIMB => Some(to),
            _ => None,
        };
        let mut best: Option<i64> = None;
        let mut off = (f64::INFINITY, true);
        for (k, height) in there {
            let away;
            if let (Some(end), Some(height)) = (end, height) {
                away = (height - end).abs();
            } else if let (Some(here), Some(height)) = (here, height) {
                away = (height - here).abs();
                if away > CLIMB && k != floor {
                    continue;
                }
            } else if k == floor || floor == ANY_FLOOR {
                away = 0.0;
            } else {
                continue;
            }
            if (away, k != floor) < off && !self.stands(step, k)? {
                best = Some(k);
                off = (away, k != floor);
            }
        }
        Ok(best)
    }

    /// The floor each cell of `path` is walked on, from the player's (`climb`): `ANY_FLOOR` all
    /// along off a stacked ground.
    pub fn walked(&self, path: &[Cell]) -> Result<Vec<i64>> {
        let first = path
            .first()
            .ok_or_else(|| Error::index("tuple index out of range"))?;
        let mut floors = vec![self.floor(*first, self.level)?];
        for pair in path.windows(2) {
            let on = self.climb(floors[floors.len() - 1], pair[0], pair[1])?;
            floors.push(on.unwrap_or(ANY_FLOOR));
        }
        Ok(floors)
    }

    /// Whether the cell is water, as far as a loaded chunk or the ROM's says.
    pub fn water(&self, cell: Cell) -> Result<bool> {
        if self.corridor.is_some() {
            return Ok(false);
        }
        match self.chunk(cell)? {
            Some(chunk) => chunk.water(cell),
            None => Ok(false),
        }
    }

    /// Whether the cell is shore the ground jumps over, getting on the water or off it: only
    /// a ground to `surf` does (`SHORE`).
    pub fn shore(&self, cell: Cell) -> Result<bool> {
        let chunk = if self.corridor.is_some() || !self.surf {
            None
        } else {
            self.chunk(cell)?
        };
        match chunk {
            Some(chunk) => Ok(chunk.kind(cell)? == SHORE),
            None => Ok(false),
        }
    }

    /// Whether the cell is blocked but jumped over either way: the shore, and a waterfall on
    /// a ground with HMs, climbed with Waterfall and gone down with nothing.
    pub fn over(&self, cell: Cell) -> Result<bool> {
        Ok(self.shore(cell)? || (self.hm && self.falls.contains_key(&cell)))
    }

    /// Whether the move from `cell` to `step` is onto, off or over a waterfall going up: a
    /// search's move over one lands two cells on.
    pub fn climbing(&self, cell: Cell, step: Cell) -> bool {
        let way = pathfind::towards(cell, step);
        let over = (
            py::floordiv(cell.0 + step.0, 2),
            py::floordiv(cell.1 + step.1, 2),
        );
        [cell, step, over]
            .iter()
            .any(|one| self.falls.get(one) == Some(&way))
    }

    /// What a move from `cell` to `step` costs on top of its steps: going onto water,
    /// through what a move clears, up a waterfall, or up a slope from the foot of it on foot,
    /// where the player gets on the bike first; through a big boulder, `SHOVE`.
    pub fn toll(&self, cell: Cell, step: Cell) -> Result<i64> {
        if self.cleared.get(&step) == Some(&STRENGTH) {
            return Ok(SHOVE);
        }
        if self.cleared.contains_key(&step) || (self.water(step)? && !self.water(cell)?) {
            return Ok(BOARD);
        }
        if (!self.bike && self.hopping(cell, step)) || self.climbing(cell, step) {
            return Ok(BOARD);
        }
        Ok(
            if !self.bike && self.climbs(cell, step)? && self.slope(cell)?.is_none() {
                BOARD
            } else {
                0
            },
        )
    }

    /// Which way the cell goes down, a ledge jumped, a slope walked or a belt ridden, or
    /// None where it is none of them or no loaded chunk holds it. A slope is None too where
    /// the ground rides up it (`bike`, `hm`).
    pub fn ledge(&self, cell: Cell) -> Result<Option<&'static str>> {
        if self.corridor.is_some() {
            return Ok(None);
        }
        if let Some(&way) = self.belts.get(&cell) {
            return Ok(Some(way));
        }
        let Some(chunk) = self.chunk(cell)? else {
            return Ok(None);
        };
        if (self.bike || self.hm) && chunk.slope(cell)?.is_some() {
            return Ok(None);
        }
        if let Some(&way) = self.hops.get(&cell) {
            // Hopped over on the Acro Bike as a ledge is jumped, but up: from (49, 31) through
            // (49, 30) to (49, 29) in Jagged Pass (omega_ruby_bike-jagged2.ctm, spike 159).
            return Ok(if self.bike || self.hm {
                Some(way)
            } else {
                None
            });
        }
        chunk.ledge(cell)
    }

    /// Which way the cell goes down where it is a slope, or None.
    pub fn slope(&self, cell: Cell) -> Result<Option<&'static str>> {
        let chunk = if self.corridor.is_some() {
            None
        } else {
            self.chunk(cell)?
        };
        match chunk {
            Some(chunk) => chunk.slope(cell),
            None => Ok(None),
        }
    }

    /// Which way the cell carries a player surfing on it where it is a current, or None:
    /// the search's `carry` (`CURRENTS`).
    pub fn carry(&self, cell: Cell) -> Result<Option<&'static str>> {
        let chunk = if self.corridor.is_some() {
            None
        } else {
            self.chunk(cell)?
        };
        match chunk {
            Some(chunk) => chunk.current(cell),
            None => Ok(None),
        }
    }

    /// Where a current carries a player on `cell`, a cell at a time, to the first cell it
    /// does not carry them on from, `cell` first: `cell` alone off a current, and where the
    /// way it carries is shut. Stops if the currents go round in a ring.
    pub fn drift(&self, cell: Cell) -> Result<Vec<Cell>> {
        let mut out = vec![cell];
        loop {
            let last = *out.last().expect("cell is first");
            let Some(way) = self.carry(last)? else {
                break;
            };
            let step = pathfind::STEPS
                .iter()
                .find(|(known, _)| *known == way)
                .map(|&(_, step)| step)
                .ok_or_else(|| py::Error::key(py::repr(way)))?;
            let ahead = (last.0 + step.0, last.1 + step.1);
            if !self.passable(ahead)? || out.contains(&ahead) {
                break;
            }
            out.push(ahead);
        }
        Ok(out)
    }

    /// Whether the move from `cell` to `step` is onto, off or over a bumpy slope, hopped up
    /// on the Acro Bike: a search's move over one lands two cells on.
    pub fn hopping(&self, cell: Cell, step: Cell) -> bool {
        let over = (
            py::floordiv(cell.0 + step.0, 2),
            py::floordiv(cell.1 + step.1, 2),
        );
        [cell, step, over]
            .iter()
            .any(|one| self.hops.contains_key(one))
    }

    /// Whether the move from `cell` to `step` goes up a slope, which only the Mach Bike
    /// does: onto it or along it any way but the way it goes down.
    pub fn climbs(&self, cell: Cell, step: Cell) -> Result<bool> {
        let down = self.slope(step)?;
        Ok(down.is_some_and(|down| down != pathfind::towards(cell, step)))
    }

    /// The loaded chunk holding the cell, or else the ROM's.
    fn chunk(&self, cell: Cell) -> Result<Option<Rc<Chunk>>> {
        let loaded = self.chunks.iter().find(|chunk| chunk.holds(cell)).cloned();
        let Some(far) = &self.far else {
            return Ok(loaded);
        };
        if loaded.is_some() {
            return Ok(loaded);
        }
        far(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK))
    }

    /// Whether the cell is a counter, which the player talks across.
    pub fn counter(&self, cell: Cell) -> Result<bool> {
        let chunk = if self.corridor.is_some() {
            None
        } else {
            self.chunk(cell)?
        };
        match chunk {
            Some(chunk) => Ok(chunk.kind(cell)? == COUNTER),
            None => Ok(false),
        }
    }

    /// Whether a step from `cell` to the cell beside it can be taken: not off a drop, and into a
    /// big boulder only a way it moves (`shoves`).
    pub fn joined(&self, cell: Cell, step: Cell) -> bool {
        let ways = self.shoves.get(&step);
        !self.cliffs.contains(&(cell, step))
            && ways.is_none_or(|ways| ways.contains(pathfind::towards(cell, step)))
    }

    /// How far each door stands turned now, in `doors`' order: where a search starts.
    pub fn turns(&self) -> Vec<i64> {
        self.doors.iter().map(|door| door.turn).collect()
    }

    /// Every step a door's arm can stand across, with the door's place in `doors`: for
    /// `through`, which a search asks of every step it takes.
    pub fn crossings(&self) -> Rc<HashMap<(Cell, Cell), i64>> {
        let pivots: Vec<Cell> = self.doors.iter().map(|door| door.pivot).collect();
        crossings_of(&pivots)
    }

    /// The doors' turns after a step from `cell` to `step` with them at `turns`: as they were
    /// where it walks into no arm, the door turned a quarter where it walks into one and every
    /// cell its arms sweep is open, and None where they are not, and the step is not taken.
    /// `crossings` is `crossings()`, worked out once a search.
    pub fn through(
        &self,
        turns: &[i64],
        cell: Cell,
        step: Cell,
        crossings: Option<&HashMap<(Cell, Cell), i64>>,
    ) -> Result<Option<Vec<i64>>> {
        let worked;
        let crossings = match crossings {
            Some(crossings) => crossings,
            None => {
                worked = self.crossings();
                &*worked
            }
        };
        let Some(&i) = crossings.get(&(cell, step)) else {
            return Ok(Some(turns.to_vec()));
        };
        let door = usize::try_from(i)
            .ok()
            .and_then(|i| self.doors.get(i))
            .ok_or_else(|| Error::index("tuple index out of range"))?;
        let at = i as usize;
        let turned = turns
            .get(at)
            .copied()
            .ok_or_else(|| Error::index("tuple index out of range"))?;
        let Some(pushed) = door.pushed(turned, cell, step)? else {
            return Ok(Some(turns.to_vec()));
        };
        for swept in door.swept(turned, pushed.1)? {
            if !(self.open(swept)? || swept == cell) {
                return Ok(None);
            }
        }
        let mut after = turns[..at].to_vec();
        after.push(py::modulo(turned + pushed.1, 4));
        after.extend_from_slice(py::slice_from(turns, i + 1));
        Ok(Some(after))
    }

    /// Where along `path` a step walks into a door's arm and turns it, by the index of the
    /// cell it steps to.
    pub fn pushes(&self, path: &[Cell]) -> Result<IndexSet<i64>> {
        let mut out: IndexSet<i64> = IndexSet::new();
        let mut turns = self.turns();
        for i in 1..path.len() {
            let after = self.through(&turns, path[i - 1], path[i], None)?;
            if let Some(after) = after
                && after != turns
            {
                out.insert(i as i64);
                turns = after;
            }
        }
        Ok(out)
    }

    /// Whether a path may go through the cell on its way somewhere else.
    pub fn passable(&self, cell: Cell) -> Result<bool> {
        self.passable_at(cell, None)
    }

    /// Whether a path may go through the cell on its way somewhere else (`open_at`).
    pub fn passable_at(&self, cell: Cell, height: Option<f64>) -> Result<bool> {
        Ok(!self.stepped.contains(&cell) && self.open_at(cell, height)?)
    }

    /// Where a path to `item` ends, each with the way to face there, or None for any way.
    ///
    /// A thing with any cell the player can stand in is walked onto: an edge's gap, the mat out
    /// of a building. A thing that is solid all over — a door, a sign, a person, a PC — is
    /// walked up to, to an open cell beside it, facing it, since that is where the player
    /// presses A or walks in from (the user, 2026-09-24); or across a counter from it, as the
    /// Nurse and a Poké Mart's clerk are talked to (`COUNTER`). A warp walked into one way is
    /// walked up to on that side alone (`Interactable.way_in`). On a stacked ground each is on
    /// the thing's own floor, which a search must reach it on (`floor`).
    pub fn goals(&self, item: &Interactable) -> Result<IndexMap<Cell, Option<&'static str>>> {
        let cells = item.cells();
        // A bush or a rock is walked up to and cleared from there, with HMs or not.
        let mut onto: IndexMap<Cell, Option<&'static str>> = IndexMap::new();
        for &cell in &cells {
            if self.open_at(cell, item.height)? && !self.cleared.contains_key(&cell) {
                onto.insert(cell, None);
            }
        }
        if !onto.is_empty() {
            return Ok(onto);
        }
        let mut beside: IndexMap<Cell, Option<&'static str>> = IndexMap::new();
        for out in 1..=self.reach {
            for &cell in &cells {
                for (direction, (dx, dz)) in pathfind::STEPS {
                    if item.way_in.is_some_and(|way| way != direction) {
                        continue;
                    }
                    let near = (cell.0 - dx * out, cell.1 - dz * out);
                    if !cells.contains(&near) && self.passable_at(near, item.height)? {
                        beside.insert(near, Some(direction));
                    } else if out == 1 && self.counter(near)? {
                        let across = (near.0 - dx, near.1 - dz);
                        if !cells.contains(&across) && self.passable(across)? {
                            beside.insert(across, Some(direction));
                        }
                    }
                }
            }
            if !beside.is_empty() {
                break;
            }
        }
        Ok(beside)
    }

    /// The cells a search from `start` may walk: the whole matrix where the ROM's chunks are
    /// read; otherwise round the loaded chunks, the start and the goals, `SEARCH_MARGIN`
    /// further, and never off the matrix; in a corridor, the corridor.
    ///
    /// A cell no chunk holds is taken to be open, so without the ROM's chunks a search kept to
    /// what is loaded, and P crossed Route 109's sea as land (spike 136). With them every cell
    /// is known, and keeping to the loaded chunks lost the way to Mauville City halfway up
    /// Route 110: it turns 38 east at x 56, and once the chunks east of x 40 unloaded no way was
    /// found inside x 49 (omega_ruby_movil.ctm, spike 142).
    pub fn r#box(&self, start: Cell, goals: impl IntoIterator<Item = Cell>) -> Result<Box> {
        if let Some(corridor) = &self.corridor {
            return corridor.r#box();
        }
        let mut cells: Vec<Cell> = vec![start];
        cells.extend(goals);
        for chunk in &self.chunks {
            cells.push(chunk.origin);
            cells.push((chunk.origin.0 + CHUNK - 1, chunk.origin.1 + CHUNK - 1));
        }
        let r#box = Box::around(cells, SEARCH_MARGIN)?;
        let Some(grid) = &self.grid else {
            return Ok(r#box);
        };
        let matrix = Box {
            west: 0,
            north: 0,
            east: grid.width * BLOCK - 1,
            south: grid.height * BLOCK - 1,
        };
        Ok(if self.far.is_some() {
            matrix
        } else {
            r#box.within(matrix)
        })
    }

    /// The way across this ground from `start` to any of `goals` by every move it has
    /// (`pathfind.find`), to `item` where one is walked to, whose floor a stacked ground ends
    /// on (`_walk`). `walkable` is where it may step, `passable` where it is not given.
    ///
    /// Python's `heading`, `item`, `closest` and `walkable` default to "", None, False and None.
    pub fn find(
        &self,
        start: Cell,
        goals: &IndexMap<Cell, Option<&'static str>>,
        heading: &str,
        item: Option<&Interactable>,
        closest: bool,
        walkable: Option<&dyn Fn(Cell) -> Result<bool>>,
    ) -> Result<Option<Vec<Cell>>> {
        let passable = |cell: Cell| self.passable(cell);
        let walkable: &dyn Fn(Cell) -> Result<bool> = walkable.unwrap_or(&passable);
        let r#box = self.r#box(start, goals.keys().copied())?;
        let ledge = |cell: Cell| self.ledge(cell);
        let toll = |cell: Cell, step: Cell| self.toll(cell, step);
        let joined = |cell: Cell, step: Cell| Ok(self.joined(cell, step));
        let over = |cell: Cell| self.over(cell);
        let walk = self.walk(start, item)?;
        let through = |turns: &[i64], cell: Cell, step: Cell| walk.through(turns, cell, step);
        let arrives = |cell: Cell, turns: &[i64]| walk.arrives(cell, turns);
        let carry = |cell: Cell| self.carry(cell);
        pathfind::find(
            start,
            goals,
            walkable,
            r#box,
            &pathfind::Options {
                heading,
                ledge: Some(&ledge),
                toll: Some(&toll),
                closest,
                joined: Some(&joined),
                turns: walk.turns.clone(),
                through: if walk.active { Some(&through) } else { None },
                over: Some(&over),
                arrives: if self.stacked { Some(&arrives) } else { None },
                carry: Some(&carry),
            },
        )
    }

    /// Every cell a way from `start` across this ground can end on, in the box round it and
    /// `aims`, with the floors it is reached on: on a stacked ground `(floor,)` each, the
    /// player's first, as a search walks them (`climb`), and `()` on any other
    /// (`pathfind.reached`). The rotating doors' arms are not followed.
    pub fn reached(
        &self,
        start: Cell,
        aims: &[Cell],
    ) -> Result<IndexMap<Cell, HashSet<pathfind::Turns>>> {
        let through = |turns: &[i64], cell: Cell, step: Cell| {
            let floor = self.climb(turns[0], cell, step)?;
            Ok(floor.map(|floor| vec![floor]))
        };
        let turns = if self.stacked {
            vec![self.floor(start, self.level)?]
        } else {
            Vec::new()
        };
        let r#box = if aims.is_empty() {
            self.r#box(start, [start])?
        } else {
            self.r#box(start, aims.iter().copied())?
        };
        let ledge = |cell: Cell| self.ledge(cell);
        let joined = |cell: Cell, step: Cell| Ok(self.joined(cell, step));
        let over = |cell: Cell| self.over(cell);
        let open = |cell: Cell| self.open(cell);
        let carry = |cell: Cell| self.carry(cell);
        pathfind::reached(
            start,
            &|cell: Cell| self.passable(cell),
            r#box,
            &pathfind::Reach {
                ledge: Some(&ledge),
                joined: Some(&joined),
                over: Some(&over),
                ends: Some(&open),
                carry: Some(&carry),
                turns,
                through: if self.stacked { Some(&through) } else { None },
            },
        )
    }

    /// What a search from `start` is given to walk the sea's currents, which carry the player
    /// their own way; through the ground's rotating doors, where they stand now and what each
    /// step does to them; and on a stacked ground the floor each step is on, the player's first
    /// and last, and the floor `item` is reached on.
    fn walk(&self, start: Cell, item: Option<&Interactable>) -> Result<Walk<'_>> {
        if self.doors.is_empty() && !self.stacked {
            return Ok(Walk {
                ground: self,
                active: false,
                turns: Vec::new(),
                crossings: None,
                height: None,
                own: ANY_FLOOR,
            });
        }
        // ponytail: every way the doors can stand is searched with the cells, 0.4 to 0.6 s across
        // Fortree Gym's seven; a search over the doors' turns alone, then the cells, if it grows.
        let crossings = if self.doors.is_empty() {
            None
        } else {
            Some(self.crossings())
        };
        if !self.stacked {
            return Ok(Walk {
                ground: self,
                active: true,
                turns: self.turns(),
                crossings,
                height: None,
                own: ANY_FLOOR,
            });
        }
        let height = item.and_then(|item| item.height);
        // Where a cell has no floor at the thing's height, the floor the thing stands on is where
        // a way that falls short ends: P to Wallace, the corridors up to him shut, went down to
        // the lower floor's room under him, nearer as the crow flies than the way up to him
        // (spike 167).
        let own = match item {
            None => ANY_FLOOR,
            Some(item) => self.floor(item.cell, height)?,
        };
        let mut turns = self.turns();
        turns.push(self.floor(start, self.level)?);
        Ok(Walk {
            ground: self,
            active: true,
            turns,
            crossings,
            height,
            own,
        })
    }
}

/// What `Ground.walk` gives a search.
///
/// Python returns the keyword arguments for `find` as a dict, `{"carry": ground.carry}` alone
/// where there are no doors and no floors (`active` is False and `turns` empty): `find` reads
/// them from this, and hands `pathfind.find` the ground's `carry` every time.
struct Walk<'a> {
    ground: &'a Ground,
    active: bool,
    turns: pathfind::Turns,
    crossings: Option<Rc<HashMap<(Cell, Cell), i64>>>,
    height: Option<f64>,
    own: i64,
}

impl Walk<'_> {
    fn through(&self, turns: &[i64], cell: Cell, step: Cell) -> Result<Option<pathfind::Turns>> {
        let ground = self.ground;
        let doors = if ground.stacked {
            &turns[..turns.len() - 1]
        } else {
            turns
        };
        let after = if ground.doors.is_empty() {
            Some(Vec::new())
        } else {
            ground.through(doors, cell, step, self.crossings.as_deref())?
        };
        let Some(mut after) = after else {
            return Ok(None);
        };
        if !ground.stacked {
            return Ok(Some(after));
        }
        let Some(floor) = ground.climb(turns[turns.len() - 1], cell, step)? else {
            return Ok(None);
        };
        after.push(floor);
        Ok(Some(after))
    }

    fn arrives(&self, cell: Cell, turns: &[i64]) -> Result<bool> {
        let at = self.ground.floor_at(cell, self.height)?;
        let floor = at.unwrap_or(self.own);
        Ok(floor == ANY_FLOOR || floor == turns[turns.len() - 1])
    }
}

/// Whether the cell can be walked, or None when no loaded chunk holds it.
pub fn walkable(chunks: &[Rc<Chunk>], cell: (i64, i64)) -> Result<Option<bool>> {
    for chunk in chunks {
        if chunk.holds(cell) {
            return Ok(Some(chunk.walkable(cell)?));
        }
    }
    Ok(None)
}
