//! Where the player can walk: the chunks' cells, the grid of zones, a rail's corridor, rooms of
//! thin ice and rotating doors, and the moves that clear the way.
//!
//! Bound by:
//! - ADR-0024: a teleporter within its zone is walked onto and carries the player, a move of P's way
//!   and of ctrl+P's walk out alike, and the partner it puts them on does nothing until stepped off.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::pathfind::{self, Box, Cell, EAST, NORTH, SOUTH, WEST};
use crate::games::common::field::chunk::{
    ANY_FLOOR, BLOCKED, BOARD, CHUNK, CLIMB, COUNTER, Chunk, LAUNCH, RISE, SHORE, STONE,
};
use crate::games::common::field::climb::{FOOT, Walls};
use crate::games::common::field::grind::{GrindRails, OFF};
use crate::games::common::field::things::{
    ARROWS, Field, Interactable, POKEMON, TRIGGER_SOURCE, WARP_SOURCE,
};
use crate::games::common::field::vines::OFF as LET_GO;
use crate::games::common::field::vines::{Vine, Vines};
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

/// Whether the player walked onto the cell they stand in, where a teleporter fires, or a
/// teleport put them there, where it does nothing until they step off (`Ground.teleports`).
pub const STEPPED: i64 = 0;

pub const LANDED: i64 = 1;

/// The way an arrow's push carries the player on from the cell they stand in, by its place here,
/// as a search's turns keep it (`Ground.walk`): 0 for none.
const PUSHED: [Option<&str>; 5] = [
    None,
    Some(pathfind::COMPASS[0]),
    Some(pathfind::COMPASS[1]),
    Some(pathfind::COMPASS[2]),
    Some(pathfind::COMPASS[3]),
];

/// `PUSHED.index(on)`.
fn pushed_index(on: Option<&str>) -> i64 {
    PUSHED
        .iter()
        .position(|way| *way == on)
        .expect("a push is one of the compass's ways") as i64
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

/// `Callable[[Cell], float | None]`: a cell's height as the ROM's chunks say it
/// (`Ground.ground_height`).
pub type GroundHeight = Rc<dyn Fn(Cell) -> Result<Option<f64>>>;

/// Where the player can walk, as far as the map loaded now says: what pathfinding walks on.
///
/// A cell cannot be walked where its chunk's blocked bit is set — a tree, a wall, a door, a sign,
/// a ledge — where the matrix has no zone, or where a move model stands. A ledge is jumped the
/// way it drops, from the cell before it to the cell after, and a slope, which is open, walked
/// only the way it goes down, as Dewford Gym's conveyor belts are the way they run (`ledge`); a
/// current in the sea carries a player on it its own way, as the Poké Ball Factory's belts carry
/// one on foot (`carry`). Only the four chunks around the player are loaded; the rest of the matrix is the ROM's (`far`). A cell
/// neither holds is taken to be open, and a path that turns out to cross a wall is found again
/// once the chunk with the wall in it loads.
/// Water is a wall too, unless the ground is one to `surf`, and a bush or a rock a move clears
/// unless it is one to clear them (`hm`), and a waterfall unless it is one to climb it (`falls`).
/// So is ground only a ridden Pokémon crosses, but to a player riding one or getting on it on
/// the way (`ride`), and a thing it smashes, but to a player riding it (`smashed`).
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
    /// Cells of a conveyor belt that carries the player, each with the way it carries them on to
    /// the next: open, and left only that way until the belt ends, as a current is (`carry`). X's
    /// Poké Ball Factory's run along its walkways, turning corners, and set the player down on the
    /// cell past the last (spike 259).
    pub conveyors: IndexMap<Cell, &'static str>,
    /// The belts as they run once `switch` is pressed, none where no switch turns them: the
    /// factory's switch turns one of its belts round, and back at the next press (`turned`).
    pub switched: IndexMap<Cell, &'static str>,
    /// The cell of the switch that turns the belts round, pressed with A from beside it.
    pub switch: Option<Cell>,
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
    /// The height of a cell's floor off a stacked ground, where the ROM's chunks are read: a
    /// thing is walked up to only from a cell within `2 * CLIMB` of its height, as on a stacked
    /// ground (`open`).
    pub ground_height: Option<GroundHeight>,
    /// How high each thing in a cell of `standing` stands, None where not known: on a stacked
    /// ground it stands on its cell's floor nearest that, and in nobody's way on the others. In
    /// Sootopolis Gym a Beauty stands in the lower floor's corridor under an invisible blocker in
    /// the corridor above (spike 167).
    pub heights: IndexMap<Cell, Vec<Option<f64>>>,
    /// The bit of a cell's word that marks a strand the player walks diagonally between
    /// (`slants`); 0 in a game without one. X's Santalune Gym has a floor of them (spike 223).
    pub strands: i64,
    /// The cells of streets laid on the grid, each with the steps its street's ways allow, as
    /// (dx, dz), a diagonal among them, and the key that takes each (`rail.Rails.lanes`).
    pub lanes: Rc<Lanes>,
    /// Whether the streets are floors of their own (`Terrain.raised`): their lanes' keys are the
    /// compass's ways nearest each street's own (`rail.Rails.lanes`), which the camera's turn makes
    /// the pad's, since the keys follow cameras looking every way, and a run up or down a ramp is
    /// one leg (`guide.RAMP_UP`, spike 248). Along Coumarine Gym's curving platforms one key held
    /// walks the curve, said as one leg, "7 east" (spike 250).
    pub raised: bool,
    /// Cells people stand in on a ground that leaves them out of `standing`, a step into one
    /// costing as much as a boulder's (`SHOVE`): the way as if nobody stood anywhere still goes
    /// round whoever it can, so who it meets first is somebody in the way (`Guide._up_to`).
    ///
    /// A `frozenset` is a `HashSet`: it is only tested.
    pub crowd: HashSet<Cell>,
    /// The cell kinds only a player riding a Pokémon crosses, blocked bit and all, as X's Rhyhorn
    /// crosses Route 9's spiky ground (`ride`, spike 238); none in a game without one.
    pub rides: HashSet<i64>,
    /// The kinds of the flat ground beside them, walked on foot and riding: the only ground a way
    /// gets onto them from, and off them onto.
    pub stops: HashSet<i64>,
    /// Whether the player is riding the Pokémon now (`mounted`).
    pub riding: bool,
    /// The cells of a Pokémon waiting on flat ground to be ridden, where a player on foot steps
    /// in and is riding; none while they ride. Nobody stands in them.
    pub mounts: HashSet<Cell>,
    /// The cells of the things the ridden Pokémon smashes with A, stepped into only riding, faced
    /// and smashed first (`ride`, `toll`), and open from then on: Route 9's rocks, smashed on the
    /// Rhyhorn, needing no HM (spike 238). Nobody stands in them; on foot they are in the way.
    pub smashed: HashSet<Cell>,
    /// The climbing walls: a face's line walked on foot or climbed, onto it only at a blue spot
    /// and off it sideways onto a ledge only climbing; and their slides, open though blocked, which
    /// carry the player down them as a current does (`carry`; `climb.Walls`, spike 245).
    ///
    /// An `Rc`: the terrain finds them once a matrix and every ground built on it shares them, as
    /// the Python's grounds share the one object.
    pub walls: Rc<Walls>,
    /// The grind rails: open though blocked, got onto only from an end cell in line, and ridden
    /// end to end with no other step, diagonally where a rail bends (`slants`; `grind.GrindRails`,
    /// spike 247).
    ///
    /// An `Rc`, as `walls` is.
    pub grind_rails: Rc<GrindRails>,
    /// The vines: an end's cell solid at its level and floor on any other, grabbed from a cell
    /// beside it, facing it, and ridden to where it lands, a level up or down or across a gap
    /// (`vines.Vines`, spike 250).
    ///
    /// An `Rc`, as `walls` is.
    pub vines: Rc<Vines>,
    /// Each listed teleporter's cell and the floor it is on, `floor`'s and `ANY_FLOOR` off a
    /// stacked ground, with where it puts the player and on what floor: Laverre Gym's panels. Not
    /// in `stepped`: a step onto one is the way on, the teleport the one move from it, and one the
    /// player is put onto does nothing until they step off it and back on (`_walk`, spike 258).
    pub teleports: IndexMap<(Cell, i64), (Cell, i64)>,
    /// The cell kinds of the arrow tiles that push the player, each with the way it pushes: open,
    /// stepped onto any way, and the player pushed on that way a cell at a time over any floor
    /// until the cell on cannot be stood in, where they stop and walk any way; an arrow crossed
    /// turns the push its own way (`arrow`, `walk`). None on a ground without one: Lysandre Labs'
    /// floor has them, and the way to a Team Flare Grunt was said again at each cell an arrow
    /// pushed the player (x_lysandre_labs, spike 265).
    pub arrows: IndexMap<i64, &'static str>,
}

/// `Ground.lanes`: each cell of a street laid on the grid, with each step its ways allow and the
/// key that takes it.
pub type Lanes = IndexMap<Cell, IndexMap<(i64, i64), &'static str>>;

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
            conveyors: IndexMap::new(),
            switched: IndexMap::new(),
            switch: None,
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
            ground_height: None,
            heights: IndexMap::new(),
            strands: 0,
            lanes: Rc::new(IndexMap::new()),
            raised: false,
            crowd: HashSet::new(),
            rides: HashSet::new(),
            stops: HashSet::new(),
            riding: false,
            mounts: HashSet::new(),
            smashed: HashSet::new(),
            walls: Rc::new(Walls::default()),
            grind_rails: Rc::new(GrindRails::default()),
            vines: Rc::new(Vines::default()),
            teleports: IndexMap::new(),
            arrows: IndexMap::new(),
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
            .field("conveyors", &self.conveyors)
            .field("switched", &self.switched)
            .field("switch", &self.switch)
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
            .field("ground_height", &self.ground_height.is_some())
            .field("heights", &self.heights)
            .field("strands", &self.strands)
            .field("lanes", &self.lanes)
            .field("crowd", &self.crowd)
            .field("rides", &self.rides)
            .field("stops", &self.stops)
            .field("riding", &self.riding)
            .field("mounts", &self.mounts)
            .field("smashed", &self.smashed)
            .field("walls", &self.walls)
            .field("grind_rails", &self.grind_rails)
            .field("vines", &self.vines)
            .field("teleports", &self.teleports)
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
    /// A `frozenset`, only tested.
    pub crossed: HashSet<Cell>,
    pub far: Option<Far>,
    pub clears: IndexMap<i64, i64>,
    /// A `frozenset`, only tested.
    pub through: HashSet<i64>,
    pub solid: IndexMap<Cell, Option<f64>>,
    pub stacked: bool,
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
    /// the game's invisible blockers (spike 167). A move model drawn as a sprite in `through`
    /// neither stands in its cells nor is stepped round: Glittering Cave's lurking Pokémon,
    /// walked through once its battle is over (spike 239). A listed trigger in `crossed`, or on
    /// a belt, is walked over: Dewford Gym's switches and belts (`Game.crossed`). So is a
    /// teleporter that moves the player within the zone, and it carries them (`teleports`);
    /// `stacked` is whether the ground is, which says the floor each is on.
    pub fn of(r#where: &Field, of: Of) -> Result<Ground> {
        let Of {
            chunks,
            grid,
            corridor,
            belts,
            crossed,
            far,
            clears,
            through,
            solid,
            stacked,
        } = of;
        let listed: Vec<&Interactable> = r#where
            .interactables
            .iter()
            .filter(|item| !(item.move_model && through.contains(&item.code)))
            .collect();
        let mut panels: Vec<(Cell, Option<f64>, Cell, f64)> = Vec::new();
        for item in &listed {
            if let (Some(arrival), Some(landing)) = (item.arrival, item.landing) {
                for cell in item.cells() {
                    panels.push((cell, item.height, arrival, landing));
                }
            }
        }
        let models: Vec<&Interactable> = listed
            .iter()
            .copied()
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
        for item in &listed {
            let place = item.place_kind();
            if item.category == POKEMON
                || place == Some(WARP_SOURCE)
                || place == Some(TRIGGER_SOURCE)
            {
                stepped.extend(item.cells());
            }
        }
        stepped.retain(|cell| {
            !crossed.contains(cell)
                && !belts.contains_key(cell)
                && !panels.iter().any(|(panel, ..)| panel == cell)
        });
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
        let mut ground = Ground {
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
            stacked,
            ..Ground::default()
        };
        if panels.is_empty() {
            return Ok(ground);
        }
        let mut teleports = IndexMap::new();
        for (cell, height, lands, landing) in panels {
            teleports.insert(
                (cell, ground.floor(cell, height)?),
                (lands, ground.floor(lands, Some(landing))?),
            );
        }
        ground.teleports = teleports;
        Ok(ground)
    }

    /// This ground under `where`, with the kinds only a player riding crosses, `rides`, and
    /// the flat ground beside them, `stops`; riding as `riding` judges it; the Pokémon whose
    /// character is one of `mounts` in nobody's way; and the things drawn as one of `smashes`
    /// smashed riding (`smashed`). One waiting on flat ground is where a player on foot gets on
    /// (`mounts`). One ridden is not listed: its model stays where it was got on, and is put at
    /// the player's cell whenever the map's models are built again (spike 238).
    pub fn mounted(
        &self,
        r#where: &Field,
        rides: &HashSet<i64>,
        stops: &HashSet<i64>,
        mounts: &HashSet<i64>,
        smashes: &HashSet<i64>,
    ) -> Result<Ground> {
        let mut ground = Ground {
            rides: rides.clone(),
            stops: stops.clone(),
            ..self.clone()
        };
        let both: HashSet<i64> = rides.union(stops).copied().collect();
        let now = riding(r#where, &|cell| ground.kind(cell), &both)?;
        let models = || r#where.interactables.iter().filter(|item| item.move_model);
        let ridden: HashSet<Cell> = models()
            .filter(|item| mounts.contains(&item.code))
            .flat_map(|item| item.cells())
            .collect();
        let rocks: HashSet<Cell> = models()
            .filter(|item| smashes.contains(&item.code))
            .flat_map(|item| item.cells())
            .collect();
        let mut waiting: HashSet<Cell> = HashSet::new();
        if !now {
            for &cell in &ridden {
                if ground.kind(cell)?.is_some_and(|kind| stops.contains(&kind)) {
                    waiting.insert(cell);
                }
            }
        }
        // `standing - ridden - rocks`, in `standing`'s order.
        ground.standing = self
            .standing
            .iter()
            .filter(|cell| !ridden.contains(cell) && !rocks.contains(cell))
            .copied()
            .collect();
        ground.riding = now;
        ground.mounts = waiting;
        ground.smashed = rocks;
        Ok(ground)
    }

    /// Whether the player can stand in the cell: on a stacked ground, on any floor of it
    /// (`open_at` with no height).
    pub fn open(&self, cell: Cell) -> Result<bool> {
        self.open_at(cell, None)
    }

    /// Whether the player can stand in the cell: on a stacked ground, on any floor of it
    /// nobody stands on, and that is no water unless the ground is one to `surf`, or on its
    /// floor at `height` where one is given (`floor_at`); off one, within `2 * CLIMB` of
    /// `height` where one is given and the cell's is known (`ground_height`), so a thing at the
    /// foot of a cliff is not walked up to from its top. Never in a vine's cell at its height,
    /// and in any where no height is given, which the search grabs it in (`vines`).
    pub fn open_at(&self, cell: Cell, height: Option<f64>) -> Result<bool> {
        if let Some(corridor) = &self.corridor {
            return Ok(!self.standing.contains(&cell) && corridor.open(cell)?);
        }
        if let Some(grid) = &self.grid
            && grid.at(py::floordiv(cell.0, BLOCK), py::floordiv(cell.1, BLOCK))? == NO_ZONE
        {
            return Ok(false);
        }
        if self.vines.at(cell, height) {
            return Ok(false);
        }
        let floors = self.floors(cell)?;
        if !floors.is_empty() {
            let Some(aim) = self.floor_at(cell, height)? else {
                return Ok(false);
            };
            for &(k, _) in &floors {
                if (aim == ANY_FLOOR || aim == k)
                    && !self.stands(cell, k)?
                    && (self.surf || !self.water(cell, Some(k))?)
                {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        if self.stands(cell, ANY_FLOOR)? {
            return Ok(false);
        } else if let (Some(height), Some(ground_height)) = (height, &self.ground_height)
            && let Some(here) = ground_height(cell)?
            && (here - height).abs() > 2.0 * CLIMB
        {
            return Ok(false);
        }
        let Some(chunk) = self.chunk(cell)? else {
            return Ok(true);
        };
        Ok((chunk.walkable(cell)?
            || self.filled.contains(&cell)
            || chunk.kind(cell)? == STONE
            || self.rides.contains(&chunk.kind(cell)?)
            || self.walls.slides.contains_key(&cell)
            || self.grind_rails.cells.contains(&cell)
            || (height.is_none() && self.vines.cells.contains(&cell)))
            && (self.surf || !chunk.water(cell, ANY_FLOOR)?))
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

    /// Whether the cell, walked on `floor`, is on a ramp's floor (`Floor.ramp`).
    pub fn on_ramp(&self, cell: Cell, floor: i64) -> Result<bool> {
        let chunk = if self.stacked {
            self.chunk(cell)?
        } else {
            None
        };
        Ok(chunk.is_some_and(|chunk| {
            0 <= floor && floor < chunk.floors.len() as i64 && chunk.floors[floor as usize].ramp
        }))
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
    /// taken as on any ground, rising no more than `RISE`, a drop being a cliff: the Poké Ball
    /// Factory's floor at (1, 20) is 36 under the switch's at (1, 21), whose stair steps from it
    /// onto another floor at 45 (spike 259). Each of Sootopolis Gym's two stairs climbs its lower
    /// floor 18 or 19 a cell to 189, and steps across onto the entrance's room at 198
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
            if !self.surf && self.water(step, Some(k))? {
                continue;
            }
            let away;
            if let (Some(end), Some(height)) = (end, height) {
                away = (height - end).abs();
            } else if let (Some(here), Some(height)) = (here, height) {
                away = (height - here).abs();
                if away > CLIMB && k != floor {
                    continue;
                }
                // Along one floor too a step rises no more than `RISE`: a drop past `2 * CLIMB`
                // is a cliff (`cliffs`), and a climber steps off a face onto its ledge.
                if k == floor
                    && RISE < away
                    && away <= 2.0 * CLIMB
                    && !self.walls.sideways(cell, step)
                {
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
    /// along off a stacked ground, and on a vine's cell, held; and where its ride lands, or
    /// a teleporter puts the player, the floor it puts them on.
    pub fn walked(&self, path: &[Cell]) -> Result<Vec<i64>> {
        Ok(self.followed(path)?.0)
    }

    /// Each step of `path` that grabs a vine, by the index of its last cell, with the vine's
    /// end it grabs: the step after it is the ride. By place, not by its cells, which the way
    /// can walk again on another level (`pathfind.legs`'s `placed`).
    pub fn vined(&self, path: &[Cell]) -> Result<IndexMap<i64, Vine>> {
        Ok(self.followed(path)?.1)
    }

    /// Each step of `path` that is a teleport, by the index of the cell it puts the player
    /// on. By place, not by its cells: a way through Laverre Gym walks (1, 1) on two floors.
    ///
    /// A `frozenset` of ints only tested is a `HashSet`.
    pub fn teleported(&self, path: &[Cell]) -> Result<HashSet<i64>> {
        Ok(self.followed(path)?.2)
    }

    /// `walked`, `vined` and `teleported`, the vine held and whether the teleporter under the
    /// player fires followed along `path` as the search follows them (`Vines.hold`, `_walk`).
    #[allow(clippy::type_complexity)]
    fn followed(&self, path: &[Cell]) -> Result<(Vec<i64>, IndexMap<i64, Vine>, HashSet<i64>)> {
        let first = path
            .first()
            .ok_or_else(|| Error::index("tuple index out of range"))?;
        let mut floors = vec![self.floor(*first, self.level)?];
        let mut grabs: IndexMap<i64, Vine> = IndexMap::new();
        let mut jumps: HashSet<i64> = HashSet::new();
        let mut held = LET_GO;
        let mut landed = self.teleports.contains_key(&(*first, floors[0]));
        for (i, pair) in path.windows(2).enumerate() {
            let (i, cell, step) = (i as i64 + 1, pair[0], pair[1]);
            let sent = if landed {
                None
            } else {
                self.teleports
                    .get(&(cell, floors[floors.len() - 1]))
                    .copied()
            };
            if let Some(sent) = sent
                && sent.0 == step
            {
                jumps.insert(i);
                floors.push(sent.1);
                landed = self.teleports.contains_key(&(step, sent.1));
                continue;
            }
            landed = false;
            let was = held;
            if !self.vines.vines.is_empty() {
                let here = self.at(cell, floors[floors.len() - 1])?;
                held = self.vines.hold(held, cell, step, here).unwrap_or(LET_GO);
            }
            if held != LET_GO {
                grabs.insert(i, self.vines.vines[(held - 1) as usize].clone());
                floors.push(ANY_FLOOR);
            } else if was != LET_GO {
                floors.push(self.floor(step, Some(self.vines.vines[(was - 1) as usize].landing))?);
            } else {
                let on = self.climb(floors[floors.len() - 1], cell, step)?;
                floors.push(on.unwrap_or(ANY_FLOOR));
            }
        }
        Ok((floors, grabs, jumps))
    }

    /// How high the player stands in `cell` on `floor`: its height there on a stacked ground,
    /// the ground's off one, None where it is not known.
    fn at(&self, cell: Cell, floor: i64) -> Result<Option<f64>> {
        if self.stacked {
            // Python's `dict(self.floors(cell)).get(floor)`: the last of a floor's entries wins.
            return Ok(self
                .floors(cell)?
                .into_iter()
                .rev()
                .find(|&(k, _)| k == floor)
                .and_then(|(_, height)| height));
        }
        match &self.ground_height {
            None => Ok(None),
            Some(ground_height) => ground_height(cell),
        }
    }

    /// Whether the cell is water, as far as a loaded chunk or the ROM's says: on a stacked
    /// ground, on `floor`, or where none is given on its floor nearest the player's height
    /// (`level`). Route 120's bridges are decks at 0 over a pond at -63, and read off the
    /// ground every deck was the pond's: P on foot stopped at each, and Shift and P crossed
    /// them with Surf (or_route-120).
    pub fn water(&self, cell: Cell, floor: Option<i64>) -> Result<bool> {
        if self.corridor.is_some() {
            return Ok(false);
        }
        let Some(chunk) = self.chunk(cell)? else {
            return Ok(false);
        };
        let floor = match floor {
            Some(floor) => floor,
            None if !chunk.floors.is_empty() => self.floor(cell, self.level)?,
            None => ANY_FLOOR,
        };
        chunk.water(cell, floor)
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

    /// Whether the cell is blocked but jumped over either way: the shore, a waterfall on a
    /// ground with HMs, climbed with Waterfall and gone down with nothing, and a mud pit's mud
    /// between two of its stones in a line, hopped over (`STONE`).
    pub fn over(&self, cell: Cell) -> Result<bool> {
        Ok(self.shore(cell)? || (self.hm && self.falls.contains_key(&cell)) || self.mud(cell)?)
    }

    fn mud(&self, cell: Cell) -> Result<bool> {
        let (x, z) = cell;
        for (a, b) in [((x - 1, z), (x + 1, z)), ((x, z - 1), (x, z + 1))] {
            if self.hopped(a)? && self.hopped(b)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Whether a hop over a mud pit starts or ends on the cell: a stone or a launch.
    fn hopped(&self, cell: Cell) -> Result<bool> {
        let chunk = if self.corridor.is_some() {
            None
        } else {
            self.chunk(cell)?
        };
        match chunk {
            Some(chunk) => {
                let kind = chunk.kind(cell)?;
                Ok(kind == STONE || kind == LAUNCH)
            }
            None => Ok(false),
        }
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
    /// where the player gets on the bike first, or into a Pokémon waiting to be ridden, faced and
    /// got on with A (`mounts`), or into a thing it smashes, faced and smashed with A
    /// (`smashed`); through a big boulder or somebody of `crowd`, `SHOVE`; along a street a
    /// way it does not run, which the keys never walk and a way takes only where the cells laid
    /// on the grid leave it no other (`lanes`); and off a climbing wall's face onto its ledge or
    /// back, a step's worth a handhold climbed to it (`Walls.toll`), on top of the rest.
    pub fn toll(&self, cell: Cell, step: Cell) -> Result<i64> {
        let held = if self.walls.ledges.is_empty() {
            0
        } else {
            self.walls.toll(cell, step)?
        };
        Ok(held + self._toll(cell, step)?)
    }

    // Python's `_toll` sits beside the public `toll`, so it keeps its underscore.
    fn _toll(&self, cell: Cell, step: Cell) -> Result<i64> {
        if self.cleared.get(&step) == Some(&STRENGTH) || self.crowd.contains(&step) {
            return Ok(SHOVE);
        }
        if self.off_lane(cell, step) {
            return Ok(2 * pathfind::TURN);
        }
        if self.cleared.contains_key(&step)
            || (self.water(step, None)? && !self.water(cell, None)?)
            || (self.mounts.contains(&step) && !self.mounts.contains(&cell))
            || self.smashed.contains(&step)
        {
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

    /// Which way the cell carries a player surfing on it where it is a current, or a player
    /// on foot down a climbing wall's slide (`Walls.slides`) or along a conveyor belt on
    /// `floor` (`conveyed`), or None: the search's `carry` (`CURRENTS`).
    ///
    /// Python's `floor` defaults to `ANY_FLOOR`.
    pub fn carry(&self, cell: Cell, floor: i64) -> Result<Option<&'static str>> {
        if let Some(way) = self
            .walls
            .slides
            .get(&cell)
            .copied()
            .or_else(|| self.conveyed(cell, floor))
        {
            return Ok(Some(way));
        }
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

    /// Whether the cell is ice, which a player on foot slides over: the search's `slide`
    /// (`ICE`).
    pub fn slide(&self, cell: Cell) -> Result<bool> {
        let chunk = if self.corridor.is_some() {
            None
        } else {
            self.chunk(cell)?
        };
        match chunk {
            Some(chunk) => chunk.ice(cell),
            None => Ok(false),
        }
    }

    /// This ground with its switch pressed: the belts the other way (`switched`).
    pub fn turned(&self) -> Ground {
        Ground {
            conveyors: self.switched.clone(),
            switched: self.conveyors.clone(),
            ..self.clone()
        }
    }

    /// The way the cell's arrow tile pushes the player, or None where it is none (`arrows`).
    pub fn arrow(&self, cell: Cell) -> Result<Option<&'static str>> {
        let kind = if self.arrows.is_empty() {
            None
        } else {
            self.kind(cell)?
        };
        Ok(kind.and_then(|kind| self.arrows.get(&kind).copied()))
    }

    /// Each step of `path` an arrow's push carries the player, by the index of the cell it
    /// carries them to, replayed as the search walks it (`walk`): from an arrow its way, and on
    /// that way from each cell after, an arrow crossed turning it, until a step goes another way,
    /// the push having set them down.
    ///
    /// A `frozenset` of ints only tested is a `HashSet`.
    pub fn arrowed(&self, path: &[Cell]) -> Result<HashSet<i64>> {
        let mut out: HashSet<i64> = HashSet::new();
        if self.arrows.is_empty() {
            return Ok(out);
        }
        let mut way: Option<&str> = None;
        for (i, pair) in path.windows(2).enumerate() {
            let (cell, step) = (pair[0], pair[1]);
            way = self.arrow(cell)?.or(way);
            if way.is_some_and(|way| pathfind::towards(cell, step) == way) {
                out.insert(i as i64 + 1);
            } else {
                way = None;
            }
        }
        Ok(out)
    }

    /// Whether an arrow's push carries a player who came into `cell` going `heading` on from
    /// it: they stand on an arrow, or came off one pointing `heading` over the open cells behind
    /// them, and the cell on that way can be stood in.
    pub fn pushing(&self, cell: Cell, heading: &str) -> Result<bool> {
        let mut way = self.arrow(cell)?;
        if way.is_none()
            && pathfind::STEPS.iter().any(|(known, _)| *known == heading)
            && !self.arrows.is_empty()
        {
            let (dx, dz) = pathfind::step_of(heading)?;
            let mut back = (cell.0 - dx, cell.1 - dz);
            // A cell no chunk holds is taken to be open: the walk back stops there.
            while self.kind(back)?.is_some() && self.arrow(back)?.is_none() && self.open(back)? {
                back = (back.0 - dx, back.1 - dz);
            }
            way = if self.arrow(back)? == Some(heading) {
                pathfind::STEPS
                    .iter()
                    .find(|(known, _)| *known == heading)
                    .map(|(known, _)| *known)
            } else {
                None
            };
        }
        let Some(way) = way else {
            return Ok(false);
        };
        let (dx, dz) = pathfind::step_of(way)?;
        self.passable((cell.0 + dx, cell.1 + dz))
    }

    /// Which way a conveyor belt carries a player on `floor` of the cell, or None: a belt is
    /// the ground's, the first of a stacked ground's floors, and the Poké Ball Factory's walkway
    /// at 90 crosses over two of them, (15, 18) and (27, 18), which the belts under it carried
    /// the player off until a belt was a floor's (spike 259).
    ///
    /// Python's `floor` defaults to `ANY_FLOOR`.
    pub fn conveyed(&self, cell: Cell, floor: i64) -> Option<&'static str> {
        if floor == ANY_FLOOR || floor == 0 {
            self.conveyors.get(&cell).copied()
        } else {
            None
        }
    }

    /// Where a current carries a player on `cell`, a cell at a time, to the first cell it
    /// does not carry them on from, `cell` first: `cell` alone off a current, and where the
    /// way it carries is shut. Stops if the currents go round in a ring. On a stacked ground,
    /// on the floor the player's height says (`level`).
    pub fn drift(&self, cell: Cell) -> Result<Vec<Cell>> {
        let mut out = vec![cell];
        let floor = self.floor(cell, self.level)?;
        loop {
            let last = *out.last().expect("cell is first");
            let Some(way) = self.carry(last, floor)? else {
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

    /// Whether the player can step from `cell` to `step`, which touches it at a corner: on a
    /// strand, where both cells carry the `strands` bit and both cells between them are walls in
    /// the chunks' own tables, not merely somebody's. The Gym's floor is a spider web of open
    /// cells whose diagonal strands the player walked in one move each, (17, 65) to (18, 66)
    /// and (17, 63) to (18, 62) to (17, 61); across X's ROM 111 corner joins carry the bit on
    /// both cells, and the 177 without it are the corners of ordinary walls (spike 223). And
    /// along or across a street that runs so, where both cells are its (`lanes`), and along a
    /// grind rail where it bends, as Shalour Gym's do a cell at a time (`grind_rails`), and from
    /// a swing's spot onto its vine, which touch at a corner (`vines`).
    pub fn slants(&self, cell: Cell, step: Cell) -> Result<bool> {
        if !self.grind_rails.steps.is_empty() && self.grind_rails.steps.contains(&(cell, step)) {
            return Ok(true);
        }
        if self.vines.boards.contains_key(&(cell, step)) {
            return Ok(true);
        }
        if self.lanes.contains_key(&cell) && self.lanes.contains_key(&step) {
            return Ok(!self.off_lane(cell, step));
        }
        if self.strands == 0 || self.corridor.is_some() {
            return Ok(false);
        }
        let corners = [(step.0, cell.1), (cell.0, step.1)];
        for one in [cell, step] {
            match self.chunk(one)? {
                Some(chunk) if chunk.word(one)? & self.strands != 0 => {}
                _ => return Ok(false),
            }
        }
        for wall in corners {
            match self.chunk(wall)? {
                Some(chunk) if !chunk.walkable(wall)? => {}
                _ => return Ok(false),
            }
        }
        Ok(true)
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

    /// Whether a step from `cell` to the cell beside it can be taken: not off a drop but
    /// off a climbing wall's face onto its ledge, which only a climber takes (`_walk`), and
    /// into a big boulder only a way it moves (`shoves`).
    pub fn joined(&self, cell: Cell, step: Cell) -> bool {
        let ways = self.shoves.get(&step);
        (!self.cliffs.contains(&(cell, step)) || self.walls.sideways(cell, step))
            && ways.is_none_or(|ways| ways.contains(pathfind::towards(cell, step)))
    }

    /// Whether the player is riding after a step from `cell` to `step`, riding before it or
    /// not, or None where the step is not taken: into a Pokémon waiting to be ridden they get on
    /// (`mounts`); into a thing it smashes (`smashed`) and onto the ground only riding crosses
    /// (`rides`) they step only riding, onto that ground from it or from flat ground (`stops`),
    /// and off it only onto flat ground; from flat ground onto any other they get off. On Route
    /// 9 the Rhyhorn was got on at (19, 61), ridden onto the flat ground at (20, 61) and on onto
    /// the spikes at (21, 61) (spike 238).
    pub fn ride(&self, riding: bool, cell: Cell, step: Cell) -> Result<Option<bool>> {
        // shortcut: a way that gets off stays on foot, the Pokémon left where they got off not
        // being one to get on again; no way on Route 9 needs it, follow it if a route does.
        if self.mounts.contains(&step) {
            return Ok(Some(true));
        }
        if self.smashed.contains(&step) && !riding {
            return Ok(None);
        }
        let held = |kind: Option<i64>, set: &HashSet<i64>| kind.is_some_and(|k| set.contains(&k));
        let onto = self.kind(step)?;
        if held(onto, &self.rides) {
            let from = self.kind(cell)?;
            return Ok(
                if riding && (held(from, &self.rides) || held(from, &self.stops)) {
                    Some(true)
                } else {
                    None
                },
            );
        }
        if held(onto, &self.stops) {
            return Ok(Some(riding));
        }
        Ok(if held(self.kind(cell)?, &self.rides) {
            None
        } else {
            Some(false)
        })
    }

    /// Whether the player is riding on each cell of `path`, from `riding` on its first, or
    /// None from the first step not taken (`ride`).
    pub fn rode(&self, path: &[Cell]) -> Result<Vec<Option<bool>>> {
        let mut out: Vec<Option<bool>> = vec![Some(self.riding)];
        for pair in path.windows(2) {
            let next = match out[out.len() - 1] {
                None => None,
                Some(last) => self.ride(last, pair[0], pair[1])?,
            };
            out.push(next);
        }
        Ok(out)
    }

    /// The cell's kind, as far as a loaded chunk or the ROM's says.
    fn kind(&self, cell: Cell) -> Result<Option<i64>> {
        let chunk = if self.corridor.is_some() {
            None
        } else {
            self.chunk(cell)?
        };
        match chunk {
            Some(chunk) => Ok(Some(chunk.kind(cell)?)),
            None => Ok(None),
        }
    }

    /// The key that walks a step along or across a street, where it is one: up walks Route
    /// 7's north-west (`lanes`). None off the streets, where a way is walked by the compass.
    pub fn key(&self, cell: Cell, step: Cell) -> Option<&'static str> {
        let lane = self.lanes.get(&cell);
        lane?.get(&(step.0 - cell.0, step.1 - cell.1)).copied()
    }

    /// Whether a step between two cells of a street goes none of its ways.
    fn off_lane(&self, cell: Cell, step: Cell) -> bool {
        self.lanes.get(&cell).is_some_and(|lane| {
            self.lanes.contains_key(&step)
                && !lane.contains_key(&(step.0 - cell.0, step.1 - cell.1))
        })
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
    /// of a building; but one `faced`, a hidden item, is walked up to. A thing that is solid all
    /// over — a door, a sign, a person, a PC — is
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
            if !item.faced
                && self.open_at(cell, item.height)?
                && !self.cleared.contains_key(&cell)
                && !self.smashed.contains(&cell)
            {
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
                // On a diagonal street the thing is walked into across a corner, its way.
                for (direction, (dx, dz)) in pathfind::DIAGONALS {
                    let near = (cell.0 - dx * out, cell.1 - dz * out);
                    if self
                        .lanes
                        .get(&near)
                        .is_some_and(|lane| lane.contains_key(&(dx, dz)))
                        && !cells.contains(&near)
                        && self.passable_at(near, item.height)?
                    {
                        beside.insert(near, Some(direction));
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
    /// on (`_walk`, `_arrives`). `walkable` is where it may step, `passable` where it is not
    /// given. Across a teleporter the distance to the goals is no measure of the way left, a
    /// teleport crossing Laverre Gym in one move, so the search goes by cost alone. A way to a
    /// teleporter or an arrow ends on it, though it carries the player on.
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
        let ends: HashSet<Cell> =
            if item.is_some_and(|item| item.landing.is_some() || item.category == ARROWS) {
                goals.keys().copied().collect()
            } else {
                HashSet::new()
            };
        let walk = self.walk(start, ends)?;
        let through = |turns: &[i64], cell: Cell, step: Cell| walk.through(turns, cell, step);
        let at = self.arrives(item)?;
        let arrives = |cell: Cell, turns: &[i64]| match &at {
            Some(at) => at.arrives(cell, turns),
            None => Ok(true),
        };
        let carry = |cell: Cell, turns: &[i64]| walk.carry(cell, turns);
        let slants = |cell: Cell, step: Cell| self.slants(cell, step);
        let slide = |cell: Cell| self.slide(cell);
        let stops = |cell: Cell, turns: &[i64]| walk.stops(cell, turns);
        let leap = |turns: &[i64], cell: Cell| walk.leap(turns, cell);
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
                arrives: if at.is_some() { Some(&arrives) } else { None },
                carry: Some(&carry),
                slants: Some(&slants),
                slide: Some(&slide),
                stops: if walk.grinding || walk.vining || walk.teleporting || walk.pushing {
                    Some(&stops)
                } else {
                    None
                },
                leap: if walk.vining || walk.teleporting {
                    Some(&leap)
                } else {
                    None
                },
                straight: self.teleports.is_empty(),
            },
        )
    }

    /// Every cell a way from `start` across this ground can end on, in the box round it and
    /// `aims`, with each floor it is reached on, `ANY_FLOOR` off a stacked ground: by `find`'s
    /// moves, walked as `find` walks them (`_walk`, `pathfind.reached`), a teleporter's cell and
    /// an arrow's among `aims` reached as a way to it ends there. Whether one ends at a thing is
    /// `ends_at`.
    pub fn reached(&self, start: Cell, aims: &[Cell]) -> Result<IndexMap<Cell, HashSet<i64>>> {
        // ponytail: the rotating doors' turns are left out, every way they stand times every cell
        // being more than a walk can take (`find` through Fortree Gym's seven takes 0.4 to 0.6 s),
        // so what lies past them is taken to be reached; follow them if that keeps too much.
        let bare;
        let ground = if self.doors.is_empty() {
            self
        } else {
            bare = Ground {
                doors: Vec::new(),
                ..self.clone()
            };
            &bare
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
        // shortcut: an arrow among `aims` is an end for whatever it is an aim of, a thing beside
        // it too, where only a way to the arrow ends there; pass the arrows' own cells if that
        // keeps a thing listed that no way reaches.
        let mut ends: HashSet<Cell> = self.teleports.keys().map(|(cell, _)| *cell).collect();
        for &cell in aims {
            if self.arrow(cell)?.is_some() {
                ends.insert(cell);
            }
        }
        let walk = ground.walk(start, ends)?;
        let through = |turns: &[i64], cell: Cell, step: Cell| walk.through(turns, cell, step);
        let carry = |cell: Cell, turns: &[i64]| walk.carry(cell, turns);
        let slants = |cell: Cell, step: Cell| self.slants(cell, step);
        let slide = |cell: Cell| ground.slide(cell);
        let stops = |cell: Cell, turns: &[i64]| walk.stops(cell, turns);
        let leap = |turns: &[i64], cell: Cell| walk.leap(turns, cell);
        let states = pathfind::reached(
            start,
            &|cell: Cell| self.passable(cell),
            r#box,
            &pathfind::Reach {
                ledge: Some(&ledge),
                joined: Some(&joined),
                over: Some(&over),
                ends: Some(&open),
                carry: Some(&carry),
                turns: walk.turns.clone(),
                through: if walk.active { Some(&through) } else { None },
                slants: Some(&slants),
                slide: Some(&slide),
                stops: if walk.grinding || walk.vining || walk.teleporting || walk.pushing {
                    Some(&stops)
                } else {
                    None
                },
                leap: if walk.vining || walk.teleporting {
                    Some(&leap)
                } else {
                    None
                },
            },
        )?;
        Ok(states
            .into_iter()
            .map(|(cell, all)| {
                let floors = all
                    .iter()
                    .map(|turns| {
                        if self.stacked {
                            turns[turns.len() - 1]
                        } else {
                            ANY_FLOOR
                        }
                    })
                    .collect();
                (cell, floors)
            })
            .collect())
    }

    /// Whether a way ending on `cell`, on `floor`, ends at `item`: on a stacked ground on the
    /// thing's own floor there, not the floor over it; anywhere on any other ground.
    pub fn ends_at(&self, item: &Interactable, cell: Cell, floor: i64) -> Result<bool> {
        if !self.stacked {
            return Ok(true);
        }
        let at = self.floor_at(cell, item.height)?;
        Ok(at.is_some_and(|at| at == ANY_FLOOR || at == floor))
    }

    /// What a search from `start` is given to walk the sea's currents, which carry the player
    /// their own way, and Shoal Cave's ice, which they slide over their own (`slide`); on a
    /// ground a ridden Pokémon crosses, whether the player is riding, the first of the turns, 1
    /// riding (`ride`); on a ground with climbing walls, what the player may be doing on a face's
    /// line, on foot, on the wall or either, after it (`Walls.hold`); on a ground with grind
    /// rails, the ride the player is on, or none, after that (`GrindRails.hold`), and a way never
    /// ends on a ride, which carries the player on to the rail's end (`stops`): P to Shalour
    /// Gym's centre, its gate shut, said "grind the rail 5 west, then can't continue" (spike 247);
    /// on a ground with vines, the vine held, or none, after that (`Vines.hold`), carried where
    /// it lands (`leap`), never into a vine's cell with no floor but to grab it, and a way never
    /// ends holding one (`stops`); on a ground with teleporters, whether the player stepped onto
    /// the cell they stand in or was put there, after that (`STEPPED`, `LANDED`): a teleporter
    /// stepped onto carries them where it puts them, the one move from it (`leap`), and no way
    /// ends on one but at `ends` (`stops`); on a ground with arrows, the way a push carries the
    /// player on from the cell stepped to, or none, after that (`PUSHED`): an arrow's own on
    /// it, and on from the cells after it while each step goes that way (`carry`), the push
    /// ending at a step any other way, taken only where the cell on cannot be walked, and no way
    /// ends where a push carries the player on but at `ends` (`stops`); through the ground's
    /// rotating doors, where they stand now and what each step does to them; and on a stacked
    /// ground the floor each step is on, the player's first, the last of the turns.
    ///
    /// `ends` is a `frozenset`, only tested.
    fn walk(&self, start: Cell, ends: HashSet<Cell>) -> Result<Walk<'_>> {
        let (ride, walled) = (!self.rides.is_empty(), !self.walls.lines.is_empty());
        let (grinding, vining) = (
            !self.grind_rails.cells.is_empty(),
            !self.vines.vines.is_empty(),
        );
        let (teleporting, pushing) = (!self.teleports.is_empty(), !self.arrows.is_empty());
        if self.doors.is_empty()
            && !self.stacked
            && !ride
            && !walled
            && !grinding
            && !vining
            && !teleporting
            && !pushing
        {
            return Ok(Walk {
                ground: self,
                active: false,
                ride,
                walled,
                grinding,
                vining,
                teleporting,
                pushing,
                ends,
                turns: Vec::new(),
                crossings: None,
                arrows: RefCell::new(HashMap::new()),
                ahead_open: RefCell::new(HashMap::new()),
            });
        }
        // ponytail: every way the doors can stand is searched with the cells, 0.4 to 0.6 s across
        // Fortree Gym's seven; a search over the doors' turns alone, then the cells, if it grows.
        let crossings = if self.doors.is_empty() {
            None
        } else {
            Some(self.crossings())
        };
        let first = self.floor(start, self.level)?;
        let mut turns: Vec<i64> = Vec::new();
        if ride {
            turns.push(self.riding as i64);
        }
        if walled {
            turns.push(self.walls.start(start, self.level));
        }
        if grinding {
            turns.push(OFF);
        }
        if vining {
            turns.push(LET_GO);
        }
        if teleporting {
            turns.push(if self.teleports.contains_key(&(start, first)) {
                LANDED
            } else {
                STEPPED
            });
        }
        if pushing {
            turns.push(0);
        }
        turns.extend(self.turns());
        if self.stacked {
            turns.push(first);
        }
        Ok(Walk {
            ground: self,
            active: true,
            ride,
            walled,
            grinding,
            vining,
            teleporting,
            pushing,
            ends,
            turns,
            crossings,
            arrows: RefCell::new(HashMap::new()),
            ahead_open: RefCell::new(HashMap::new()),
        })
    }

    /// The search's `carry`: on a stacked ground on the floor the turns end with. Python's
    /// `_carry`.
    fn carried(&self, cell: Cell, turns: &[i64]) -> Result<Option<&'static str>> {
        let floor = if self.stacked {
            turns[turns.len() - 1]
        } else {
            ANY_FLOOR
        };
        self.carry(cell, floor)
    }

    /// On a stacked ground, what a search to `item` is given to say a cell with the turns it
    /// stands in is on the thing's floor; nothing on any other.
    fn arrives(&self, item: Option<&Interactable>) -> Result<Option<Arrives<'_>>> {
        if !self.stacked {
            return Ok(None);
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
        Ok(Some(Arrives {
            ground: self,
            height,
            own,
        }))
    }
}

/// What `Ground.walk` gives a search.
///
/// Python returns the keyword arguments for `find` as a dict, `{"carry": ground.carry}` alone
/// where there are no doors and no floors (`active` is False and `turns` empty): `find` and
/// `reached` read them from this, and hand `pathfind` the ground's `carry` every time.
struct Walk<'a> {
    ground: &'a Ground,
    active: bool,
    /// Whether the first of the turns is whether the player is riding.
    ride: bool,
    /// Whether the next is what the player may be doing on a climbing wall's line.
    walled: bool,
    /// Whether the next is the grind rail's ride the player is on, or none.
    grinding: bool,
    /// Whether the next is the vine the player holds, or none.
    vining: bool,
    /// Whether the next is whether the player stepped onto their cell or was put there.
    teleporting: bool,
    /// Whether the next is the way an arrow's push carries the player on (`PUSHED`).
    pushing: bool,
    /// The cells a way may end on a teleporter at.
    ends: HashSet<Cell>,
    turns: pathfind::Turns,
    crossings: Option<Rc<HashMap<(Cell, Cell), i64>>>,
    /// Each state asks its cell's arrow once a step it tries and again as it may stop there:
    /// Python's `cache(self.arrow)`, and `cache(self.passable)` of the cell a push goes on to.
    arrows: RefCell<HashMap<Cell, Option<&'static str>>>,
    ahead_open: RefCell<HashMap<Cell, bool>>,
}

/// What `Ground.arrives` gives a search to a thing on a stacked ground: Python's `arrives`
/// closure, over the thing's height and the floor it stands on.
struct Arrives<'a> {
    ground: &'a Ground,
    height: Option<f64>,
    own: i64,
}

impl Walk<'_> {
    /// Where the vine held is in the turns.
    fn vine(&self) -> usize {
        self.ride as usize + self.walled as usize + self.grinding as usize
    }

    /// Where whether a teleporter fires is in the turns, one further than the vine held.
    fn panel(&self) -> usize {
        self.vine() + self.vining as usize
    }

    /// Where the push is in the turns, one further again.
    fn push(&self) -> usize {
        self.panel() + self.teleporting as usize
    }

    /// `cache(self.arrow)`.
    fn arrow(&self, cell: Cell) -> Result<Option<&'static str>> {
        if let Some(&way) = self.arrows.borrow().get(&cell) {
            return Ok(way);
        }
        let way = self.ground.arrow(cell)?;
        self.arrows.borrow_mut().insert(cell, way);
        Ok(way)
    }

    /// Python's `shoved` closure: the way an arrow's push carries the player on from `cell`.
    fn shoved(&self, cell: Cell, turns: &[i64]) -> Result<Option<&'static str>> {
        if !self.pushing {
            return Ok(None);
        }
        Ok(self.arrow(cell)?.or(PUSHED[turns[self.push()] as usize]))
    }

    /// Python's `carried_on` closure: whether a push carries the player on from `cell`.
    fn carried_on(&self, cell: Cell, turns: &[i64]) -> Result<bool> {
        let Some(way) = self.shoved(cell, turns)? else {
            return Ok(false);
        };
        let (dx, dz) = pathfind::step_of(way)?;
        let ahead = (cell.0 + dx, cell.1 + dz);
        if let Some(&open) = self.ahead_open.borrow().get(&ahead) {
            return Ok(open);
        }
        let open = self.ground.passable(ahead)?;
        self.ahead_open.borrow_mut().insert(ahead, open);
        Ok(open)
    }

    /// Python's `carry` closure: the ground's carry, or an arrow's push.
    fn carry(&self, cell: Cell, turns: &[i64]) -> Result<Option<&'static str>> {
        Ok(match self.ground.carried(cell, turns)? {
            Some(way) => Some(way),
            None => self.shoved(cell, turns)?,
        })
    }

    /// Python's `fires` closure: where the teleporter in the cell puts the player and on what
    /// floor, where they stepped onto it; None where they were put there, or there is none.
    fn fires(&self, turns: &[i64], cell: Cell) -> Option<(Cell, i64)> {
        if !self.teleporting || turns[self.panel()] != STEPPED {
            return None;
        }
        let floor = if self.ground.stacked {
            turns[turns.len() - 1]
        } else {
            ANY_FLOOR
        };
        self.ground.teleports.get(&(cell, floor)).copied()
    }

    /// Python's `stops` closure: whether the player can stop on the cell with `turns`.
    fn stops(&self, cell: Cell, turns: &[i64]) -> Result<bool> {
        // A rail's own cells are only ever stood on mid-ride: its end cells are not among them.
        Ok(!self.ground.grind_rails.cells.contains(&cell)
            && (!self.vining || turns[self.vine()] == LET_GO)
            && (self.ends.contains(&cell)
                || (self.fires(turns, cell).is_none() && !self.carried_on(cell, turns)?)))
    }

    /// Python's `leap` closure: where the vine held carries the player, or the teleporter
    /// stepped onto puts them.
    fn leap(&self, turns: &[i64], cell: Cell) -> Option<Cell> {
        if self.vining
            && let Some(lands) = self.ground.vines.leap(turns[self.vine()])
        {
            return Some(lands);
        }
        self.fires(turns, cell).map(|sent| sent.0)
    }

    fn through(&self, turns: &[i64], cell: Cell, step: Cell) -> Result<Option<pathfind::Turns>> {
        let ground = self.ground;
        let sent = self.fires(turns, cell);
        if let Some(sent) = sent
            && sent.0 != step
        {
            return Ok(None);
        }
        let rode = if self.ride {
            ground.ride(turns[0] != 0, cell, step)?
        } else {
            Some(false)
        };
        let Some(rode) = rode else {
            return Ok(None);
        };
        let rest = if self.ride { &turns[1..] } else { turns };
        let held = if self.walled {
            ground.walls.hold(rest[0], cell, step)
        } else {
            Some(FOOT)
        };
        let Some(held) = held else {
            return Ok(None);
        };
        let rest = if self.walled { &rest[1..] } else { rest };
        let grind = if self.grinding {
            ground.grind_rails.hold(rest[0], cell, step)
        } else {
            Some(OFF)
        };
        let Some(grind) = grind else {
            return Ok(None);
        };
        let rest = if self.grinding { &rest[1..] } else { rest };
        let holding = if self.vining { rest[0] } else { LET_GO };
        let hold = if self.vining {
            let floor = if ground.stacked {
                rest[rest.len() - 1]
            } else {
                ANY_FLOOR
            };
            ground
                .vines
                .hold(holding, cell, step, ground.at(cell, floor)?)
        } else {
            Some(LET_GO)
        };
        let Some(hold) = hold else {
            return Ok(None);
        };
        if LET_GO == holding
            && holding == hold
            && ground.vines.cells.contains(&step)
            && ground.floors(step)?.is_empty()
        {
            return Ok(None);
        }
        let rest = if self.vining { &rest[1..] } else { rest };
        // A teleport puts the player on its partner's panel, which does nothing until they
        // step off it and back on.
        let put = match sent {
            Some(sent) if ground.teleports.contains_key(&(step, sent.1)) => LANDED,
            _ => STEPPED,
        };
        let rest = if self.teleporting { &rest[1..] } else { rest };
        let way = self.shoved(cell, turns)?;
        let on = way.filter(|&way| pathfind::towards(cell, step) == way);
        let rest = if self.pushing { &rest[1..] } else { rest };
        let doors = if ground.stacked {
            &rest[..rest.len() - 1]
        } else {
            rest
        };
        let after = if ground.doors.is_empty() {
            Some(Vec::new())
        } else {
            ground.through(doors, cell, step, self.crossings.as_deref())?
        };
        let Some(mut after) = after else {
            return Ok(None);
        };
        if ground.stacked {
            // A rail is on no floor the search walks: its cells are blocked on all of them,
            // and the cell it lands on is stepped onto from any (`climb`). Nor is a vine held,
            // and its ride lands on the floor at its height, as a teleport does on its own.
            let floor = if ground.grind_rails.cells.contains(&step) || hold != LET_GO {
                Some(ANY_FLOOR)
            } else if holding != LET_GO {
                Some(ground.floor(
                    step,
                    Some(ground.vines.vines[(holding - 1) as usize].landing),
                )?)
            } else if let Some(sent) = sent {
                Some(sent.1)
            } else {
                ground.climb(rest[rest.len() - 1], cell, step)?
            };
            let Some(floor) = floor else {
                return Ok(None);
            };
            after.push(floor);
        }
        if self.pushing {
            after.insert(0, pushed_index(on));
        }
        if self.teleporting {
            after.insert(0, put);
        }
        if self.vining {
            after.insert(0, hold);
        }
        if self.grinding {
            after.insert(0, grind);
        }
        if self.walled {
            after.insert(0, held);
        }
        if self.ride {
            after.insert(0, rode as i64);
        }
        Ok(Some(after))
    }
}

impl Arrives<'_> {
    fn arrives(&self, cell: Cell, turns: &[i64]) -> Result<bool> {
        let at = self.ground.floor_at(cell, self.height)?;
        let floor = at.unwrap_or(self.own);
        Ok(floor == ANY_FLOOR || floor == turns[turns.len() - 1])
    }
}

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

/// Whether the cell can be walked, or None when no loaded chunk holds it.
pub fn walkable(chunks: &[Rc<Chunk>], cell: (i64, i64)) -> Result<Option<bool>> {
    for chunk in chunks {
        if chunk.holds(cell) {
            return Ok(Some(chunk.walkable(cell)?));
        }
    }
    Ok(None)
}
