//! Where the player can walk: the chunks' cells, the grid of zones, a rail's corridor, rooms of
//! thin ice, the moves that clear the way, and the traversals a search crosses it by, each a module
//! of its own (`traversal`, `TRAVERSALS`).
//!
//! Bound by:
//! - ADR-0024: a teleporter within its zone is walked onto and carries the player, a move of P's way
//!   and of ctrl+P's walk out alike, and the partner it puts them on does nothing until stepped off.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::pathfind::{self, Box, Cell, Leg, Step};
use crate::games::common::field::arrow::Arrows;
use crate::games::common::field::chunk::{
    ANY_FLOOR, BLOCKED, BOARD, CHUNK, CLIMB, COUNTER, Chunk, LAUNCH, RISE, SHORE, STONE,
};
use crate::games::common::field::climb::Walls;
use crate::games::common::field::grind::GrindRails;
use crate::games::common::field::ride::{Riding, riding};
use crate::games::common::field::rotating::{Doors, RotatingDoor};
use crate::games::common::field::teleport::Teleporters;
use crate::games::common::field::things::{
    ARROWS, Field, Interactable, POKEMON, TRIGGER_SOURCE, WARP_SOURCE,
};
use crate::games::common::field::traversal::{Name, Traversal};
use crate::games::common::field::vines::Vines;
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

/// The game's own words a step can be said with, which the guide has: its moves by number, the
/// Mach Bike and the Acro Bike.
#[derive(Debug, Clone, PartialEq)]
pub struct StepWords {
    pub moves: IndexMap<i64, String>,
    pub bike: String,
    pub acro: String,
}

/// What a landform calls a step from a cell to the next on a floor, of its own ground, or None
/// (`Ground.words`).
pub type Word = Rc<dyn Fn(&Ground, Cell, Cell, i64, &StepWords) -> Result<Option<String>>>;

/// What `chunk_at`, `floors_at` and `cached_for` hold: each cell's chunk and floors as worked out,
/// and the chunks, the ROM's chunks and the stacking they were worked out for, the chunks and the
/// ROM's by identity, as Python's `is` compares them.
#[derive(Default)]
pub struct Cached {
    cached_for: Option<(Vec<*const Chunk>, Option<*const ()>, bool)>,
    chunk_at: HashMap<Cell, Option<Rc<Chunk>>>,
    floors_at: HashMap<Cell, Vec<(i64, Option<f64>)>>,
}

/// Cells to a block of the matrix.
pub const BLOCK: i64 = 10;

pub const NO_ZONE: i64 = 0xFFFF;

/// Cells past the loaded chunks, the player and the goal that a path may go round through,
/// none of them known.
pub const SEARCH_MARGIN: i64 = 10;

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

/// The field map's zone ids, `width` blocks to a row.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
/// the way (`ride.after`), and a thing it smashes, but to a player riding it (`smashed`).
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
    /// into an arm turning its door (`rotating.Doors`, spike 161).
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
    /// crosses Route 9's spiky ground (`ride.after`, spike 238); none in a game without one.
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
    /// and smashed first (`ride.after`, `toll`), and open from then on: Route 9's rocks, smashed on
    /// the Rhyhorn, needing no HM (spike 238). Nobody stands in them; on foot they are in the way.
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
    /// What each landform calls a step of its own ground, "slide", "ride the current", the Acro
    /// Bike: a leg is said with the first that names its step, after a ramp's and before a cleared
    /// thing's move and Surf, which are no landform's (`Guide.to`).
    pub words: Vec<Word>,
    /// Each cell's chunk and floors as worked out (`_chunk`, `floors`): shared by the copies
    /// `replace` makes of this ground, as the guide makes one for each search (`Guide.ground`),
    /// and made afresh for a copy with other chunks. A search round Fortree City asked for them
    /// 13.5 and 8.3 million times over 193 searches, each worked out again.
    ///
    /// Python works them out afresh as a copy is made (`__post_init__`); here as a copy is first
    /// asked, which no copy can tell apart (`fresh`).
    pub cached: Rc<RefCell<Cached>>,
    /// Each listed teleporter's cell and the floor it is on, `floor`'s and `ANY_FLOOR` off a
    /// stacked ground, with where it puts the player and on what floor: Laverre Gym's panels. Not
    /// in `stepped`: a step onto one is the way on, the teleport the one move from it, and one the
    /// player is put onto does nothing until they step off it and back on (`teleport`, spike
    /// 258).
    pub teleports: IndexMap<(Cell, i64), (Cell, i64)>,
    /// The cell kinds of the arrow tiles that push the player, each with the way it pushes: open,
    /// stepped onto any way, and the player pushed on that way a cell at a time over any floor
    /// until the cell on cannot be stood in, where they stop and walk any way; an arrow crossed
    /// turns the push its own way (`arrow.Arrows`). None on a ground without one: Lysandre Labs'
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
            words: Vec::new(),
            cached: Rc::new(RefCell::new(Cached::default())),
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
            .field("words", &self.words.len())
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
    /// walked through once its battle is over (spike 239). A listed trigger in `crossed`
    /// is walked over: Dewford Gym's switches, and so is a belt (`landform.Belts`). So is a
    /// teleporter that moves the player within the zone, and it carries them (`teleports`);
    /// `stacked` is whether the ground is, which says the floor each is on.
    pub fn of(r#where: &Field, of: Of) -> Result<Ground> {
        let Of {
            chunks,
            grid,
            corridor,
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
            !crossed.contains(cell) && !panels.iter().any(|(panel, ..)| panel == cell)
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
        self.fresh();
        if let Some(known) = self.cached.borrow().floors_at.get(&cell) {
            return Ok(known.clone());
        }
        let out = self.floors_of(cell)?;
        self.cached.borrow_mut().floors_at.insert(cell, out.clone());
        Ok(out)
    }

    /// `floors` worked out.
    fn floors_of(&self, cell: Cell) -> Result<Vec<(i64, Option<f64>)>> {
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

    /// The floor each cell of `way` is walked on, as its search found it: `ANY_FLOOR` all
    /// along off a stacked ground, and on a vine's cell, held; and where its ride lands, or a
    /// teleporter puts the player, the floor it puts them on.
    pub fn walked(&self, way: &[Step]) -> Result<Vec<i64>> {
        way.iter()
            .map(|step| {
                if self.stacked {
                    step.state
                        .last()
                        .copied()
                        .ok_or_else(|| Error::index("tuple index out of range"))
                } else {
                    Ok(ANY_FLOOR)
                }
            })
            .collect()
    }

    /// What each traversal says the steps of `way` it takes are walked with, by the index of
    /// each step's last cell, the first traversal's where two say a step (`Traversal.said`).
    pub fn named(&self, way: &[Step], name: Name<'_>) -> Result<IndexMap<i64, String>> {
        let mut out: IndexMap<i64, String> = IndexMap::new();
        let mut from = 0;
        for traversal in self.traversals() {
            let to = from + traversal.width();
            let mine = mine(way, from, to)?;
            for (at, how) in traversal.said(self, way, &mine, name)? {
                out.entry(at).or_insert(how);
            }
            from = to;
        }
        Ok(out)
    }

    /// `legs`, `way`'s, with each traversal's own said as it says them (`Traversal.legs`).
    pub fn legs(&self, way: &[Step], legs: Vec<Leg>) -> Result<Vec<Leg>> {
        let mut legs = legs;
        let mut from = 0;
        for traversal in self.traversals() {
            let to = from + traversal.width();
            let mine = mine(way, from, to)?;
            legs = traversal.legs(self, way, &mine, legs)?;
            from = to;
        }
        Ok(legs)
    }

    /// Each cell of `path`, a way no search of this ground found, with the state one would
    /// give it, from `state`, or the player's where it is None: a way over thin ice, which
    /// steps on every tile once, and the way up to the surface from under the sea
    /// (`guide._over_ice`, `guide._dive_again`).
    ///
    /// ponytail: a step a search would refuse keeps the state, on any floor; neither way crosses
    /// a traversal. Find such a way by the search's own moves if one ever does.
    pub fn follow(&self, path: &[Cell], state: Option<pathfind::State>) -> Result<Vec<Step>> {
        let first = *path
            .first()
            .ok_or_else(|| Error::index("tuple index out of range"))?;
        let walk = self.walk(first, HashSet::new())?;
        let mut now: pathfind::State = match state {
            None => walk.state.clone(),
            Some(state) => state,
        };
        let mut out = vec![Step {
            cell: first,
            state: now.clone(),
        }];
        for pair in path.windows(2) {
            let (cell, step) = (pair[0], pair[1]);
            let after = if walk.active {
                walk.through(&now, cell, step)?
            } else {
                None
            };
            if let Some(after) = after {
                now = after;
            } else if self.stacked {
                now.pop();
                now.push(ANY_FLOOR);
            }
            out.push(Step {
                cell: step,
                state: now.clone(),
            });
        }
        Ok(out)
    }

    /// How high the player stands in `cell` on `floor`: its height there on a stacked ground,
    /// the ground's off one, None where it is not known.
    pub fn height(&self, cell: Cell, floor: i64) -> Result<Option<f64>> {
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
        self.current(cell)
    }

    /// Which way the cell carries a player surfing on it where it is a current in the sea, or
    /// None (`CURRENTS`).
    pub fn current(&self, cell: Cell) -> Result<Option<&'static str>> {
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
        self.fresh();
        if let Some(chunk) = self.cached.borrow().chunk_at.get(&cell) {
            return Ok(chunk.clone());
        }
        let mut chunk = self.chunks.iter().find(|chunk| chunk.holds(cell)).cloned();
        if chunk.is_none()
            && let Some(far) = &self.far
        {
            chunk = far(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK))?;
        }
        self.cached
            .borrow_mut()
            .chunk_at
            .insert(cell, chunk.clone());
        Ok(chunk)
    }

    /// `chunk_at` and `floors_at` emptied where they were worked out for other chunks, other
    /// ROM's chunks or another stacking than this ground's: Python's `__post_init__`.
    fn fresh(&self) {
        let now = (
            self.chunks.iter().map(Rc::as_ptr).collect::<Vec<_>>(),
            self.far.as_ref().map(|far| Rc::as_ptr(far) as *const ()),
            self.stacked,
        );
        let mut cached = self.cached.borrow_mut();
        if cached.cached_for.as_ref() != Some(&now) {
            cached.chunk_at.clear();
            cached.floors_at.clear();
            cached.cached_for = Some(now);
        }
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
    /// off a climbing wall's face onto its ledge, which only a climber takes (`Walls.hold`), and
    /// into a big boulder only a way it moves (`shoves`).
    pub fn joined(&self, cell: Cell, step: Cell) -> bool {
        let ways = self.shoves.get(&step);
        (!self.cliffs.contains(&(cell, step)) || self.walls.sideways(cell, step))
            && ways.is_none_or(|ways| ways.contains(pathfind::towards(cell, step)))
    }

    /// The cell's kind, as far as a loaded chunk or the ROM's says.
    pub fn kind(&self, cell: Cell) -> Result<Option<i64>> {
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

    /// The way across this ground from `start` to any of `goals` by every move it has, each
    /// cell with the player's state there (`pathfind.find`), to `item` where one is walked to,
    /// whose floor a stacked ground ends on (`_walk`, `_arrives`). `walkable` is where it may
    /// step, `passable` where it is not given. Across a teleporter the distance to the goals is
    /// no measure of the way left, a teleport crossing Laverre Gym in one move, so the search
    /// goes by cost alone. A way to a teleporter or an arrow ends on it, though it carries the
    /// player on.
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
    ) -> Result<Option<Vec<Step>>> {
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
        let through = |state: &[i64], cell: Cell, step: Cell| walk.through(state, cell, step);
        let at = self.arrives(item)?;
        let arrives = |cell: Cell, turns: &[i64]| match &at {
            Some(at) => at.arrives(cell, turns),
            None => Ok(true),
        };
        let carry = |cell: Cell, state: &[i64]| walk.carry(cell, state);
        let slants = |cell: Cell, step: Cell| self.slants(cell, step);
        let slide = |cell: Cell| self.slide(cell);
        let stops = |cell: Cell, state: &[i64]| walk.stops(cell, state);
        let leap = |state: &[i64], cell: Cell| walk.leap(state, cell);
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
                state: walk.state.clone(),
                through: if walk.active { Some(&through) } else { None },
                over: Some(&over),
                arrives: if at.is_some() { Some(&arrives) } else { None },
                carry: Some(&carry),
                slants: Some(&slants),
                slide: Some(&slide),
                stops: if walk.carries { Some(&stops) } else { None },
                leap: if walk.carries { Some(&leap) } else { None },
                straight: !self.traversals().iter().any(|way| way.far()),
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
        let through = |state: &[i64], cell: Cell, step: Cell| walk.through(state, cell, step);
        let carry = |cell: Cell, state: &[i64]| walk.carry(cell, state);
        let slants = |cell: Cell, step: Cell| self.slants(cell, step);
        let slide = |cell: Cell| ground.slide(cell);
        let stops = |cell: Cell, state: &[i64]| walk.stops(cell, state);
        let leap = |state: &[i64], cell: Cell| walk.leap(state, cell);
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
                state: walk.state.clone(),
                through: if walk.active { Some(&through) } else { None },
                slants: Some(&slants),
                slide: Some(&slide),
                stops: if walk.carries { Some(&stops) } else { None },
                leap: if walk.carries { Some(&leap) } else { None },
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

    /// The traversals this ground has, in `TRAVERSALS`' order, the order a search keeps their
    /// states in.
    pub fn traversals(&self) -> Vec<Rc<dyn Traversal>> {
        TRAVERSALS.iter().filter_map(|on| on(self)).collect()
    }

    /// What a search from `start` is given to walk the sea's currents, which carry the player
    /// their own way, and Shoal Cave's ice, which they slide over their own (`slide`); and on a
    /// ground with traversals or a stacked one, the player's state, each traversal's and the
    /// floor after them, the player's first: what each step does to it, where the floor is the
    /// one a traversal puts the player on, the first that does, or else the ground's climb
    /// (`Traversal.lands`, `climb`); and where a traversal carries the player on, the cells no
    /// way ends on, but at `ends`, and where it carries them in one move (`Traversal.stops`,
    /// `Traversal.leap`): no way ends on a grind rail's ride, P to Shalour Gym's centre, its
    /// gate shut, saying "grind the rail 5 west, then can't continue" (spike 247).
    ///
    /// `ends` is a `frozenset`, only tested.
    fn walk(&self, start: Cell, ends: HashSet<Cell>) -> Result<Walk<'_>> {
        let ways = self.traversals();
        let stacked = self.stacked;
        if ways.is_empty() && !stacked {
            return Ok(Walk {
                ground: self,
                active: false,
                ways,
                spans: Vec::new(),
                stacked,
                ends,
                state: Vec::new(),
                pushers: Vec::new(),
                carries: false,
            });
        }
        // The traversals that carry the player a cell at a time, as a current does: an arrow's
        // push.
        let pushers: Vec<usize> = (0..ways.len()).filter(|&i| ways[i].carries_on()).collect();
        let first = self.floor(start, self.level)?;
        let mut state: pathfind::State = Vec::new();
        let mut spans: Vec<(usize, usize)> = Vec::new();
        for way in &ways {
            let own = way.start(self, start, first)?;
            spans.push((state.len(), state.len() + own.len()));
            state.extend(own);
        }
        if stacked {
            state.push(first);
        }
        let carries = ways.iter().any(|way| way.carries());
        Ok(Walk {
            ground: self,
            active: true,
            ways,
            spans,
            stacked,
            ends,
            state,
            pushers,
            carries,
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
/// Python returns the keyword arguments for `find` as a dict, `{"carry": ground._carry, "slide":
/// ground.slide}` alone where there are no traversals and no floors (`active` is False and
/// `state` empty): `find` and `reached` read them from this, and hand `pathfind` the ground's
/// `carry` and `slide` every time. The search's state is flat: each traversal's part of it is its
/// `spans` entry, and the floor is last on a stacked ground.
struct Walk<'a> {
    ground: &'a Ground,
    active: bool,
    ways: Vec<Rc<dyn Traversal>>,
    /// Where each traversal's state is in the search's, in `ways`' order.
    spans: Vec<(usize, usize)>,
    stacked: bool,
    /// The cells a way may end on though a traversal carries the player on.
    ends: HashSet<Cell>,
    state: pathfind::State,
    /// The traversals that carry the player a cell at a time, by their place in `ways`.
    pushers: Vec<usize>,
    /// Whether any traversal carries the player on: then `stops` and `leap` are given.
    carries: bool,
}

/// What `Ground.arrives` gives a search to a thing on a stacked ground: Python's `arrives`
/// closure, over the thing's height and the floor it stands on.
struct Arrives<'a> {
    ground: &'a Ground,
    height: Option<f64>,
    own: i64,
}

impl Walk<'_> {
    /// The floor the player is on in `state`: its last on a stacked ground, `ANY_FLOOR` off one.
    fn floor(&self, state: &[i64]) -> i64 {
        if self.stacked {
            state[state.len() - 1]
        } else {
            ANY_FLOOR
        }
    }

    /// Python's `through` closure.
    fn through(&self, state: &[i64], cell: Cell, step: Cell) -> Result<Option<pathfind::State>> {
        let ground = self.ground;
        let floor = self.floor(state);
        let mut after: pathfind::State = Vec::new();
        for (way, &(from, to)) in self.ways.iter().zip(&self.spans) {
            let Some(then) = way.step(ground, &state[from..to], cell, step, floor)? else {
                return Ok(None);
            };
            after.extend(then);
        }
        if !self.stacked {
            return Ok(Some(after));
        }
        // Loops, not generators: a search round Fortree City takes this step a million times.
        let mut on = None;
        for (way, &(from, to)) in self.ways.iter().zip(&self.spans) {
            on = way.lands(
                ground,
                &state[from..to],
                &after[from..to],
                cell,
                step,
                floor,
            )?;
            if on.is_some() {
                break;
            }
        }
        if on.is_none() {
            on = ground.climb(floor, cell, step)?;
        }
        Ok(on.map(|on| {
            after.push(on);
            after
        }))
    }

    /// Python's `stops` closure.
    fn stops(&self, cell: Cell, state: &[i64]) -> Result<bool> {
        let floor = self.floor(state);
        for (way, &(from, to)) in self.ways.iter().zip(&self.spans) {
            if !way.stops(self.ground, &state[from..to], cell, floor, &self.ends)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Python's `leap` closure.
    fn leap(&self, state: &[i64], cell: Cell) -> Option<Cell> {
        let floor = self.floor(state);
        self.ways
            .iter()
            .zip(&self.spans)
            .find_map(|(way, &(from, to))| way.leap(self.ground, &state[from..to], cell, floor))
    }

    /// Python's `carry` closure, or the ground's own `_carry` where no traversal carries the
    /// player a cell at a time.
    fn carry(&self, cell: Cell, state: &[i64]) -> Result<Option<&'static str>> {
        if let Some(way) = self.ground.carried(cell, state)? {
            return Ok(Some(way));
        }
        let floor = self.floor(state);
        for &i in &self.pushers {
            let (from, to) = self.spans[i];
            if let Some(to) = self.ways[i].carry(self.ground, &state[from..to], cell, floor)? {
                return Ok(Some(to));
            }
        }
        Ok(None)
    }
}

impl Arrives<'_> {
    fn arrives(&self, cell: Cell, turns: &[i64]) -> Result<bool> {
        let at = self.ground.floor_at(cell, self.height)?;
        let floor = at.unwrap_or(self.own);
        Ok(floor == ANY_FLOOR || floor == turns[turns.len() - 1])
    }
}

/// Each kind of traversal, as what a ground has of it, or None: a new kind is a module and a line
/// here. The order is the search's: its states are kept in it, and where two put the player on a
/// floor, or carry them in one move, the first does (`Ground._walk`).
pub const TRAVERSALS: [fn(&Ground) -> Option<Rc<dyn Traversal>>; 7] = [
    Riding::on,
    Walls::on,
    GrindRails::on,
    Vines::on,
    Teleporters::on,
    Arrows::on,
    Doors::on,
];

/// `[step.state[i] for step in way]`: one traversal's own state at each cell of `way`, its run
/// `from..to` of the search's flat state (`traversal`).
fn mine(way: &[Step], from: usize, to: usize) -> Result<Vec<&[i64]>> {
    way.iter()
        .map(|step| {
            step.state
                .get(from..to)
                .ok_or_else(|| Error::index("tuple index out of range"))
        })
        .collect()
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
