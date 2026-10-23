//! The field: where the player is, and what is on the map worth walking to.
//!
//! What it lists is every game's (`common.field`: `Interactable`, `Field`, `Ground`); this reads
//! gen 6's memory for them.
//!
//! Four structures, none with a class of its own, read in spike 78:
//!
//! **Move model** — `field::mmodel::MoveModel`, a pool of 32, one per person or thing on the map:
//! +0x04 zero when free, +0x0c `zone << 16 | NPC index` (0xff for the player), +0x10 `flag << 16 |
//! model code`, +0x14 `script << 16`, the cell it is in as floats at +0x94 and +0x9c. The zone is
//! the one the model was built in: walking into the next outdoor zone keeps the model.
//!
//! **Hidden Pokémon** — the DexNav's find, whose tail shows above the tall grass: not a model in
//! the pool but a record of its own (spike 80), listed under a category of its own. ORAS alone has
//! one, and its reader is ORAS's (`oras.field.hidden`), handed to this one by the game
//! (`HiddenReader`).
//!
//! **Map block** — `field::MapBlock`, one of the four 40 by 40 chunks of the matrix the game keeps
//! loaded around the player, read in `mapblock` (spike 82). Its cell table says which cells can be
//! walked, which is how a zone's edge is narrowed to the gap in the trees, and what kind each cell
//! is, which is how a **use tile** is found: a PC, and whatever else the game runs a script for
//! when the player faces the cell and presses A, with no entity record behind it (spike 83).

use std::cell::{Cell as Slot, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{s16, u16, u32};
use crate::core::discovery::{Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::core::pathfind::{self, Cell};
use crate::core::words::own;
use crate::games::common::field::chunk::HOLE as BOULDER_HOLE;
use crate::games::common::field::chunk::{
    BIKE_RAILS, BLOCKED, BUMPY_SLOPES, CHUNK, CURRENTS, Chunk, DIVE, FALLS, SLOPES,
};
use crate::games::common::field::ground::{
    self as common, BLOCK, CRACKED, Far, Grid, Ground, NO_ZONE, Of, STRENGTH, pushes, riding,
    walkable,
};
use crate::games::common::field::things::{
    BIKE, BIKE_SOURCE, CURRENT_SOURCE, CURRENTS_CATEGORY, DECORATION_SOURCE, DIVE_SOURCE,
    EDGE_SOURCE, FURNITURE_SOURCE, Field, Interactable, OBSTACLES, POKEMON, RIDE_SOURCE, RIDING,
    STAIRS_SOURCE, TILE_SOURCE, TRIGGER_SOURCE, WARP_SOURCE, kind_said, place_source,
};
use crate::games::gen6::field::features::{
    DoorReader, Feature, HiddenReader, Hiding, IceReader, SecretBaseReader, Tracked,
};
use crate::games::gen6::field::rail::{self, Rails, keep};
use crate::games::gen6::field::terrain::{FarCells, FarFloors, FarHeights, FarMatrices, Terrain};
use crate::games::gen6::field::zonedata::{
    FURNITURE, FURNITURE_SCRIPT, LZ11, NPC, NPC_INDEX, NPC_SCRIPT, NPC_SPRITE, NPC_X, NPC_Z,
    TRIGGER_HEIGHT, TRIGGER_SCRIPT, TRIGGER_WIDTH, TRIGGER_X, TRIGGER_Y, TRIGGER_Z, WARP,
    WARP_DOOR, WARP_KIND, WARP_OUT, WARP_STAIRS, WARP_STREET, WARP_TO, WARP_Y, ZoneData, arrival,
    on_rail, read_furniture, read_records, read_triggers, read_zone, warp_box, zone_files,
};
use crate::games::gen6::field::zonetable::{
    INFO_KIND, INFO_MATRIX, INFO_NAME, INFO_PARENT, INFO_SIZE, NAME_MASK, UNDERWATER, ZONE_LIMIT,
    ZoneTable,
};
use crate::games::gen6::field::{fieldmap, mapblock, mmodel};
use crate::games::gen6::game::{FurnitureKind, Game, Rides};
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::rom::maps::{ChunkSource, StreetSource};
use crate::py::{self, Error, PySet, Result};

/// What a sweep looks for. The slot's own shape is `mmodel`'s, as every structure's is.
pub const MOVE_MODEL_CLASS: &str = mmodel::CLASS;

/// How far over or under the ground it is walked from a warp walked through may stand and still be
/// an opening and not stairs. Over every room-to-room warp of this kind in ORAS the rise is 0 for
/// 104 of them and 1 either way for one (Seafloor Cavern's, Victory Road's entrance room), 4 either
/// way for six (Rustboro's two-storey houses, Lilycove Museum), 8 to 10 either way for 68 (Devon
/// Corporation, Littleroot's houses, the hideouts), -18 for two and 135 for three (old Mauville's
/// zone 0xf); none is 2 or 3 (2026-10-06).
pub const WARP_RISE: f64 = 2.0;
/// A warp walked through's low byte -> the way it is walked into (`WARP_STAIRS`). Every one of
/// ORAS's 362 off the rails is a wall; 361 have open floor beside them on the side this says they
/// are walked in from, and on no other side but Ever Grande City's way into Victory Road's lower
/// floor, whose ground behind stands 45 over it (spike 213).
pub const WALKED_INTO: [&str; 4] = [
    pathfind::SOUTH,
    pathfind::NORTH,
    pathfind::EAST,
    pathfind::WEST,
];

/// The PC's cell kind, seen booted in poke_center_pc.ctm (spike 83).
pub const PC_TILE: i64 = 0xD5;

/// The cell kinds the game runs a script for when the player faces one and presses A: every kind
/// in the field's table of (kind test, script) pairs (spike 83). The PC is called what the game
/// calls it where the ROM can be read (`FieldReader`'s `words`). The rest, and the PC without the
/// ROM, are called what the game's shipped table says each kind is, `0xdd` a trash can (spike 210),
/// and what a prop is where it says nothing: the game's text has only what each of them says,
/// never a word for one.
///
/// Python's `USE_TILES` frozenset, `{PC_TILE, *range(0xD7, 0xE0)}`, as the test `kind in
/// USE_TILES`.
pub fn use_tiles(kind: i64) -> bool {
    kind == PC_TILE || (0xD7..0xE0).contains(&kind)
}

/// What a thing the player reads by facing it is, where nothing says more.
pub const PROP: &str = "object";

/// Sweeps a zone listed with no chunk asks for before leaving the rest to the schedule.
pub const ASKS: i64 = 4;
pub const TABLE_ASKS: i64 = 10;
/// Sweeps asked for while the zone data is in and the zone table is not, and the seconds between
/// them, before leaving the rest to the schedule. The table's row was in place 1.9 s after the
/// field started, but nothing pointed at its start yet, and it was found only by the whole sweep 30
/// s later: every exit said without its place, and the way to Oldale Town said as stairs
/// (omega_ruby_ledges_3.ctm, 2026-09-28).
pub const TABLE_EVERY: f64 = 2.0;

/// What a beam between two posts is called, which a switch swings a quarter turn: the game never
/// names one, and its move model's sprite is the invisible blocker it stands elsewhere (spike
/// 146).
pub const BARRIER: &str = "barrier";
/// The key the records of every elevator's door share (`FieldReader._furniture_key`): never said.
pub const ELEVATOR: &str = "elevator";

pub const SLOPE: &str = "slope";
pub const BUMPY_SLOPE: &str = "bumpy slope";
/// What a sandy slope only the Mach Bike rides up, a rocky step only the Acro Bike hops up, and an
/// end of a rail only the Acro Bike crosses, are called: the game names none of them, and its words
/// for the bikes are their items' (spike 159).
pub const BIKE_RAIL: &str = "rail";

/// What a patch of the deep water Dive goes down through is called: the game draws it darker than
/// the sea and asks about diving there, but never names it (spike 164).
pub const DIVE_SPOT: &str = "dive spot";

/// What the sea's currents that let the player go round one place are called, listed at the cell
/// of them nearest the player and said with where it lets them go: the game never names one.
pub const CURRENT: &str = "current";

/// What a patch of the flat ground a ridden Pokémon is got on and off on is called, listed at its
/// cell nearest the player: the game never names it (spike 238).
pub const FLAT_GROUND: &str = "flat ground";

/// Cells apart the places two currents let the player go may be and the two still be listed as
/// one current. Route 133's currents let the player go at 152 cells, 71 groups of them side by side,
/// 28 within 5 cells of each other; Routes 132 and 134 have 20 and 16, Seafloor Cavern's two rooms 4
/// each (spike 208).
pub const LANDING_REACH: i64 = 5;

/// What a stair between two floors of one chunk is called, a warp between floors being stairs
/// too: the game names neither (spike 167).
pub const STAIRS: &str = "stairs";

/// What a Secret Base's flag beside its PC is called: the game never names it where it stands.
pub const FLAG: &str = "flag";

/// The cell kind of a Secret Base's crumbled cliff, where the Stairs are placed to climb to the
/// upper floor: always blocked, one patch of 3 cells by 2 a base (Route 116's, zone 0x1a7, at x 17
/// to 19, z 17 to 18, its top row at the upper floor's height 36 and its bottom at 18, the lower
/// floor 0). The 857 ORAS chunk files have it only in base shapes 11, 12, 21, 22, 23, 40 and 41,
/// the 13 spots that need Stairs, and X's 1030 have none (spike 201).
pub const CLIFF_TILE: i64 = 0xFD;

/// What a Secret Base's crumbled cliff is called: the game names it only in the Stairs'
/// description, "You can place this by a crumbled cliff to climb to a higher floor", never where
/// it stands (omega_ruby_secret-base3.ctm).
pub const CLIFF: &str = "crumbled cliff";

/// The cell kind of a hole in a Secret Base's floor, crossed on a Solid Board: always blocked,
/// two cells in Granite Cave's base (zone 0x1f0, at x 20, z 12 to 13), the only way into its
/// upper-left room (omega_ruby_secret-base6.ctm). The ROM has it only in base shapes 42 to 50
/// (spike 201).
pub const HOLE_TILE: i64 = 0xFE;

/// What a hole in a Secret Base's floor is called: the game names it only in the Solid Board's
/// description, "You can place this over holes to cross over them", never where it is. A hole a big
/// boulder drops into is one too, listed under obstacles with the boulders until one fills it
/// (Orion, 2026-10-11; spike 202): its cells are of a kind of their own (`chunk.HOLE`).
pub const HOLE: &str = "hole";

/// The cells of a Secret Base a decoration goes over, by kind, each with its name.
pub const BASE_TILES: [(i64, &str); 2] = [(CLIFF_TILE, CLIFF), (HOLE_TILE, HOLE)];

/// `BASE_TILES.get(kind)`.
fn base_tile(kind: i64) -> Option<&'static str> {
    BASE_TILES
        .iter()
        .find(|(tile, _)| *tile == kind)
        .map(|(_, name)| *name)
}

/// What the decorating cursor is on where nothing stands and nothing is in the way: the game says
/// nothing as it moves from one such cell to the next (the user's word, 2026-10-02).
pub const BLANK: &str = "blank";

/// What the decorating cursor is on where the cell table has the cell blocked and no decoration,
/// PC or flag stands: the base's walls and the hedge round them.
pub const WALL: &str = "wall";

/// Said first while an item is held: whether it can be set down where the cursor is. The game
/// draws its squares green or red and says "You cannot set it there!" only once A is pressed
/// (omega_ruby_secret-base5.ctm).
pub fn fit(fits: bool) -> &'static str {
    if fits {
        "decorating-placeable"
    } else {
        "decorating-not-placeable"
    }
}

pub const BASE_ASKS: i64 = 5;
/// Sweeps asked for inside a Secret Base while what stands where is not found, and the seconds
/// between them. The base's module is loaded as the player goes in, and its classes are learned a
/// look after a look that lists it, too late for the sweep that look comes with.
pub const BASE_EVERY: f64 = 2.0;

/// What a move model is to the field, which says whether it is listed and whether it stands in
/// the way (`FieldReader._role`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The player drawn as another model, as the Magma Suit is put on in the Cave of Origin,
    /// listed there as "person, here" (spike 203): neither listed nor in the way.
    Look,
    /// A beam a switch swings, in its zone (`Game.barriers`, spike 146): listed under obstacles,
    /// and in the way as anything listed is.
    Barrier,
    /// The game's invisible blocker (`Game.blockers`): never listed, and in the way. Sootopolis
    /// Gym's nine, rows of three across the corridors up from its rooms, were listed as nameless
    /// people (spike 167). Round a zone's rotating doors they stand in for the doors' arms, which
    /// the doors are walked through by, and are not in the way: five were listed as "person 1
    /// west" round Fortree Gym's third door, and P said "person in the way" (spike 161).
    Blocker,
    /// A model drawn as nothing (`mmodel.NONDRAW`): in the way, as on each of Sootopolis City's
    /// locked doors, and in its own zone listed as what it stands in for (`_stand_ins`, spike
    /// 203).
    Nothing,
    /// Anything else: listed as what it is drawn as (`_model_item`).
    Drawn,
}

/// The two-word holder of the zone data buffers, current first.
struct Holder {
    address: i64,
    files: IndexMap<i64, ZoneData>,
}

/// What `_ways` was found for: the zone, the matrix's chunk files, and the chunks loaded.
type BikeKey = (i64, Rc<Vec<i64>>, Vec<(i64, i64)>);
/// `_ways_of`: what they were found for, the bike's things, the dive spots' patches, the
/// currents and the patches of flat ground.
type Ways = (
    Option<BikeKey>,
    Vec<Interactable>,
    Vec<Vec<Cell>>,
    Vec<IndexMap<Cell, Cell>>,
    Vec<Vec<Cell>>,
);
/// What `_find_ways` finds: the bike's things, the dive spots' patches, the currents and the
/// patches of flat ground.
type FoundWays = (
    Vec<Interactable>,
    Vec<Vec<Cell>>,
    Vec<IndexMap<Cell, Cell>>,
    Vec<Vec<Cell>>,
);
/// `_stand_ins`: what each model drawn as nothing stands in for by its record's index, the
/// cells of the zone's warps, and the records' indices of its gates.
type StandIns = (IndexMap<i64, i64>, HashSet<Cell>, HashSet<i64>);

/// Tracks the player's model and the current zone's data, and lists what is on the map.
///
/// Every holder shares the one reader, so it is made in an `Rc` (`FieldReader::new`) and its
/// methods take `&self`; the ground it hands out keeps the reader's terrain alive through `me`.
pub struct FieldReader {
    me: Weak<FieldReader>,
    memory: Rc<dyn Memory>,
    /// The zone table, which names every place and tells a path from stairs.
    pub zones: ZoneTable,
    /// Warp kind -> what it is, where the kind says more than a door or stairs (`Game.warps`).
    warp_words: IndexMap<i64, String>,
    /// What the game has of its own to list: ORAS's berry spots and ash piles.
    features: Vec<Rc<dyn Feature>>,
    /// The game's rotating doors, or None in a game without them.
    doors: Option<Rc<dyn DoorReader>>,
    /// The sprites of the game's invisible blockers (`Game.blockers`).
    blockers: HashSet<i64>,
    /// The scripts of the furniture records of elevators' doors (`Game.elevators`).
    elevators: HashSet<i64>,
    /// The sprites the player is drawn as (`Game.player_looks`).
    player_looks: HashSet<i64>,
    /// Zones whose floors' stairs are listed (`Game.stairs`).
    stairs: HashSet<i64>,
    /// Zones listed a room at a time (`Game.rooms`).
    rooms: HashSet<i64>,
    /// The game's rooms of thin ice, or None in a game without them.
    ice: Option<Rc<dyn IceReader>>,
    /// What the game calls each decoration, by its number (`GameText.decorations`).
    decorations: Vec<String>,
    /// (zone, a furniture record's script) -> what it is, where it is no sign (`Game.furniture`):
    /// a Secret Base's PC and flag.
    furniture_kinds: IndexMap<(i64, i64), FurnitureKind>,
    /// The bit of a cell's word that marks a strand walked diagonally (`Game.strands`).
    strands: i64,
    /// What the game's ridden Pokémon cross, or None in a game without one (`Game.rides`).
    rides: Option<Rides>,
    /// Whether a row of models drawn as nothing with a script of its own is a gate
    /// (`Game.gates`).
    gates: bool,
    /// The game's Secret Bases, or None in a game without them.
    secret_bases: Option<Rc<dyn SecretBaseReader>>,
    /// (zone, sprite) of the move models that are a barrier's beam (`Game.barriers`).
    barriers: HashSet<(i64, i64)>,
    /// (zone, a trigger's script) -> the category and kind it is listed as, and for an exit into
    /// another place, the zone it leads to (`Game.triggers`).
    triggers: IndexMap<(i64, i64), (String, String, Option<i64>)>,
    /// Sprite -> the move that clears the thing drawn as it (`Game.clears`).
    clears: IndexMap<i64, i64>,
    /// The sprites of the spots a wild Pokémon leaps out of (`Game.ambushes`).
    ambushes: HashSet<i64>,
    /// The matrix out of the ROM, which never touches RAM.
    pub terrain: Terrain,
    /// Underwater zone -> its `Ground.surfaces`, once found.
    surfaces_of: RefCell<HashMap<i64, IndexMap<Cell, Rc<HashSet<Cell>>>>>,
    /// Zone -> its triggers' boxes, (x, z, width, height) in the table's order, once read
    /// (`trigger_at`).
    trigger_boxes: RefCell<HashMap<i64, Vec<(i64, i64, i64, i64)>>>,
    /// The player's height in the world at the last poll, which says which floor of a chunk with a
    /// deck they are on (`mapblock.flat`).
    height: Slot<Option<f64>>,
    /// What the slopes, rail ends, dive spots, currents and flat ground were found for, and
    /// what they are: the bike's things, each dive spot's patch, each current's cells with where
    /// each lets the player go, and each patch of flat ground (`_ways`).
    ways_of: RefCell<Ways>,
    clock: Rc<dyn Fn() -> f64>,
    /// Lumiose's streets, which X places things on rather than on cells.
    pub rails: Rails,
    /// (zone, a trigger's script) -> the way the conveyor belt it is runs (`Game.belts`).
    belts: IndexMap<(i64, i64), &'static str>,
    /// Zones walked about in as outdoors that are not their own parent, out of the ROM
    /// (`plates.outdoors`): a warp off a street into one of them is not a door.
    outdoors: HashSet<i64>,
    /// Where the game keeps the sprite a move model is drawn with (`mmodel.drawn_as`).
    drawn: i64,
    /// Where the game's things' and Pokémon's models start (`Game.models`).
    model_starts: Option<(i64, i64)>,
    /// Where the player's model says what they move as (`Game.form`).
    form: Option<i64>,
    /// What each kind of use tile is: the game's own word where it has one, the PC's out of
    /// the ROM (`archives.PC_LINES`), ahead of the shipped table's.
    words: IndexMap<i64, String>,
    /// What furniture is by the game's shared script it runs: a vending machine.
    furniture_words: IndexMap<i64, String>,
    /// (zone, a furniture record's script) -> the shared script it runs
    /// (`plates.shared_furniture`).
    shared: IndexMap<(i64, i64), i64>,
    models: RefCell<Vec<i64>>,
    player: Slot<Option<i64>>,
    /// Whether the player's model was on a rail at the last poll.
    on_rail: Slot<bool>,
    holder: RefCell<Option<Holder>>,
    map: Slot<Option<i64>>,
    /// The field map's grid as last read: a sweep ago, or when the player's model changed.
    grid: RefCell<Option<Grid>>,
    /// The game's hidden Pokémon, or None in a game without one.
    hidden: Option<Rc<dyn HiddenReader>>,
    /// The map blocks, once a sweep has found them.
    blocks: RefCell<Vec<i64>>,
    /// Every chunk's cell table read so far, by where its file sat: a block's file is read once,
    /// and again only when the block has loaded another chunk into it.
    chunks: RefCell<HashMap<mapblock::Loaded, Rc<Chunk>>>,
    /// The zone whose edges were listed whole because no block held a chunk. The blocks' class is
    /// in the field's DLL, which no sweep can look for until the runtime has read it, and a sweep
    /// before the blocks are built finds none; either way the listing at the door of the house
    /// sent the player at the trees (the replays of 2026-09-22), so a sweep is asked for.
    wanted: Slot<Option<i64>>,
    /// The zone a sweep was asked for since the last sweep, so a listing asks once per sweep and
    /// not every poll. Only a matrix with a neighbouring zone asks, so a building, whose matrix has
    /// none, never does.
    asked: Slot<Option<i64>>,
    /// Sweeps asked for the zone in `_wanted` so far. A sweep that still finds no chunk is
    /// followed by another, up to `ASKS`: the blocks may be built after the field's other objects,
    /// but a matrix whose blocks a sweep can never find must not keep the runtime sweeping every
    /// tick, and the scheduled whole sweep catches up in a few seconds.
    asks: Slot<i64>,
    /// Sweeps asked for the zone table so far (`TABLE_ASKS`).
    table_asks: Slot<i64>,
    /// When the next may be asked for.
    table_next: Slot<f64>,
    /// Sweeps asked for in the Secret Base the player is in so far (`BASE_ASKS`).
    base_asks: Slot<i64>,
    base_next: Slot<f64>,
    /// The player's model and the zone it was built in, as the last poll read them.
    visit: Slot<Option<(i64, i64)>>,
}

impl common::Corridor for rail::Corridor {
    fn open(&self, cell: Cell) -> Result<bool> {
        Ok(rail::Corridor::open(self, cell))
    }

    fn r#box(&self) -> Result<pathfind::Box> {
        rail::Corridor::r#box(self)
    }
}

impl FieldReader {
    /// `game` says what one gen 6 game has of its own, `text` is what was read from the player's
    /// ROM, or `NO_TEXT` without one, and `image` the executable, where ORAS lists its Secret
    /// Bases' spots. `tiles` and `furniture` are what the shipped table says each kind of use tile
    /// is, and each piece of furniture by the shared script it runs (`Names.words`).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        memory: Rc<dyn Memory>,
        game: &Game,
        text: &GameText,
        image: &[u8],
        clock: Rc<dyn Fn() -> f64>,
        tiles: Option<&IndexMap<i64, String>>,
        furniture: Option<&IndexMap<i64, String>>,
    ) -> Rc<FieldReader> {
        let chunks: Option<Rc<dyn ChunkSource>> = text.map_chunks.clone();
        let terrain = Terrain::new(
            chunks
                .clone()
                .map(|chunks| Rc::new(move |file: i64| chunks.cells(file)) as FarCells),
            chunks
                .clone()
                .map(|chunks| Rc::new(move |file: i64| chunks.floors(file)) as FarFloors),
            chunks
                .clone()
                .map(|chunks| Rc::new(move |file: i64| chunks.heights(file)) as FarHeights),
            chunks.map(|chunks| Rc::new(move |number: i64| chunks.matrix(number)) as FarMatrices),
        );
        let streets: Option<rail::Streets> =
            text.streets.clone().map(|streets: Rc<dyn StreetSource>| {
                Rc::new(move |chunks: &[i64]| streets.network(chunks)) as rail::Streets
            });
        let mut words: IndexMap<i64, String> = tiles.cloned().unwrap_or_default();
        if let Some(pc) = text.pc.clone()
            && !pc.is_empty()
        {
            words.insert(PC_TILE, pc);
        }
        let features: Vec<Rc<dyn Feature>> = game
            .features
            .iter()
            .map(|make| make(memory.clone(), text))
            .collect();
        let secret_bases = match game.secret_bases {
            Some(make) if !image.is_empty() => Some(make(memory.clone(), image)),
            _ => None,
        };
        Rc::new_cyclic(|me| FieldReader {
            me: me.clone(),
            zones: ZoneTable::new(memory.clone(), text.zone_table.clone()),
            warp_words: game.warps.clone(),
            features,
            doors: game.doors.map(|make| make(memory.clone())),
            blockers: game.blockers.clone(),
            elevators: game.elevators.clone(),
            player_looks: game.player_looks.clone(),
            stairs: game.stairs.clone(),
            rooms: game.rooms.clone(),
            ice: game.ice.map(|make| make(memory.clone())),
            decorations: text.decorations.clone(),
            furniture_kinds: game.furniture.clone(),
            strands: game.strands,
            rides: game.rides.clone(),
            gates: game.gates,
            secret_bases,
            barriers: game.barriers.clone(),
            triggers: game.triggers.clone(),
            clears: game.clears.clone(),
            ambushes: game.ambushes.clone(),
            terrain,
            surfaces_of: RefCell::new(HashMap::new()),
            trigger_boxes: RefCell::new(HashMap::new()),
            height: Slot::new(None),
            ways_of: RefCell::new((None, Vec::new(), Vec::new(), Vec::new(), Vec::new())),
            clock,
            rails: Rails::new(memory.clone(), streets, text.street_records.clone()),
            belts: game.belts.clone(),
            outdoors: text.outdoors.clone(),
            drawn: game.drawn,
            model_starts: game.models,
            form: game.form,
            words,
            furniture_words: furniture.cloned().unwrap_or_default(),
            shared: text.shared.clone(),
            models: RefCell::new(Vec::new()),
            player: Slot::new(None),
            on_rail: Slot::new(false),
            holder: RefCell::new(None),
            map: Slot::new(None),
            grid: RefCell::new(None),
            hidden: game.hidden.map(|make| make(memory.clone())),
            blocks: RefCell::new(Vec::new()),
            chunks: RefCell::new(HashMap::new()),
            wanted: Slot::new(None),
            asked: Slot::new(None),
            asks: Slot::new(0),
            table_asks: Slot::new(0),
            table_next: Slot::new(0.0),
            base_asks: Slot::new(0),
            base_next: Slot::new(0.0),
            visit: Slot::new(None),
            memory,
        })
    }

    /// What a sweep looks for for the game's own readers: ORAS's DexNav, Secret Bases, ash piles,
    /// rotating doors and thin ice.
    pub fn classes(&self) -> Vec<&'static str> {
        self.tracked()
            .into_iter()
            .flat_map(|reader| reader.classes())
            .collect()
    }

    fn tracked(&self) -> Vec<&dyn Tracked> {
        let mut out: Vec<&dyn Tracked> = Vec::new();
        for feature in &self.features {
            out.push(&**feature);
        }
        if let Some(bases) = &self.secret_bases {
            out.push(&**bases);
        }
        if let Some(hidden) = &self.hidden {
            out.push(&**hidden);
        }
        if let Some(doors) = &self.doors {
            out.push(&**doors);
        }
        if let Some(ice) = &self.ice {
            out.push(&**ice);
        }
        out
    }

    /// The hidden Pokémon last listed, which says how the DexNav found it.
    pub fn hiding(&self) -> Option<Hiding> {
        self.hidden.as_ref().and_then(|hidden| hidden.last())
    }

    /// Take in what a sweep found: the model pool, the zone data and its holder, the field map and
    /// the zone table.
    pub fn track(&self, found: &Found) -> Result<()> {
        let mut models = found.of(&[MOVE_MODEL_CLASS]);
        if !models.is_empty() {
            models.sort();
            *self.models.borrow_mut() = models;
        }
        let mut files: IndexMap<i64, ZoneData> = IndexMap::new();
        for data in zone_files(&found.pages, &*self.memory)? {
            files.insert(data.address, data);
        }
        if !files.is_empty() {
            if let Some(holder) = self.find_holder(found, &files)? {
                *self.holder.borrow_mut() = Some(holder);
            }
            if let Some(holder) = self.holder.borrow_mut().as_mut() {
                holder.files.extend(files);
            }
        }
        self.map.set(keep(
            fieldmap::find(&*self.memory, &found.pages)?,
            self.map.get(),
        ));
        self.read_map()?;
        if self.zones.base.get().is_none() && self.holder.borrow().is_some() {
            self.find_table(found)?;
        }
        for reader in self.tracked() {
            reader.track(found)?;
        }
        let player = self.find_player()?;
        let head = match player {
            None => None,
            Some(player) => Some(self.memory.read(player, mmodel::HEAD)?),
        };
        self.rails.track(found, head.as_deref())?;
        let blocks = found.of(&[mapblock::CLASS]);
        if !blocks.is_empty() && found.partial {
            // A sweep of the blocks just allocated finds the new ones alone; the rest are kept
            // while they still carry a map block's vtable.
            let vtable = u32(&self.memory.read(blocks[0], 4)?, 0)?;
            let held = self.blocks.borrow().clone();
            let mut kept: Vec<i64> = Vec::new();
            for block in held {
                if u32(&self.memory.read(block, 4)?, 0)? == vtable {
                    kept.push(block);
                }
            }
            kept.extend(blocks);
            kept.sort();
            kept.dedup();
            *self.blocks.borrow_mut() = kept;
        } else if !blocks.is_empty() {
            let mut sorted = blocks;
            sorted.sort();
            *self.blocks.borrow_mut() = sorted;
        }
        if !found.partial {
            // A sweep has been; if it still found no chunk, the next listing may ask for another.
            self.asked.set(None);
        }
        Ok(())
    }

    /// The player's move model as the last poll found it, not looked at since.
    pub fn player(&self) -> Option<i64> {
        self.player.get()
    }

    /// The player's zone and cell, or None while nothing says where they are: before the first
    /// sweep, and on the title screen, where the pool holds the save's models with no zone data
    /// behind them.
    pub fn poll(&self) -> Result<Option<Field>> {
        let Some(player) = self.find_player()? else {
            return Ok(None);
        };
        if self.current()?.is_none() {
            return Ok(None);
        }
        let head = self.memory.read(player, mmodel::HEAD)?;
        let Some(cell) = mmodel::cell(&head)? else {
            return Ok(None);
        };
        self.visit.set(Some((player, mmodel::zone_of(&head)?)));
        self.height.set(mmodel::height(&head)?);
        let zone = match self.zone_at(cell)? {
            Some(zone) => zone,
            None => mmodel::zone_of(&head)?,
        };
        let form = match self.form {
            None => None,
            Some(at) => Some(
                *self
                    .memory
                    .read(player + at, 1)?
                    .first()
                    .ok_or_else(|| py::Error::index("index out of range"))? as i64,
            ),
        };
        let bike = mmodel::on_bike(&head, form)?;
        self.on_rail.set(mmodel::on_rail(&head)?);
        // Under a street's own camera the keys walk the world's ways, so its streets are the
        // grid's ground (`Rails.fixed`).
        if !mmodel::on_rail(&head)? || {
            let files = self.terrain.files.borrow().1.clone();
            self.rails.fixed(&files, mmodel::rail(&head)?.0)?
        } {
            *self.rails.corridor.borrow_mut() = None;
            return Ok(Some(Field {
                on_bike: bike,
                place: Some(mmodel::place(&head)?),
                surfing: mmodel::surfing(&head, form)?,
                height: self.height.get(),
                ..Field::new(zone, cell)
            }));
        }
        let on = self.rails.poll(&head, &self.read_files()?.1)?;
        let Some((at, bearing)) = on else {
            return Ok(Some(Field {
                on_rail: true,
                on_bike: bike,
                height: self.height.get(),
                ..Field::new(zone, cell)
            }));
        };
        let heading = match bearing {
            None => None,
            Some(bearing) => Some(pathfind::nearest(bearing)?),
        };
        Ok(Some(Field {
            on_rail: true,
            heading,
            bearing,
            on_bike: bike,
            height: self.height.get(),
            ..Field::new(zone, at)
        }))
    }

    /// The quarters clockwise from north the pad's up walks the player, as the last poll
    /// found them: where the camera looks (`Rails.turn`), and north on a rail, where the keys
    /// walk the corridor or a street's lanes and the camera's own heading reads 0 (spike 230).
    pub fn turn(&self, current: i64) -> Result<i64> {
        if self.on_rail.get() {
            Ok(0)
        } else {
            self.rails.turn(current)
        }
    }

    /// The player's model and the zone it was built in, as the last poll read them.
    ///
    /// Every warp builds the player a new model in the zone it arrives in, and walking across an
    /// edge keeps the old one (spike 78), so this changes at a warp and not at an edge. It may also
    /// change where the game rebuilds the field around a player standing still.
    pub fn visit(&self) -> Option<(i64, i64)> {
        self.visit.get()
    }

    /// Where the player can walk around `where`, with its listing's people standing in their
    /// cells when it has one.
    ///
    /// A matrix that puts no zone at the player's own cell is not trusted to say where the map
    /// ends: the grid was read 32 blocks wide on a matrix 40 wide until 2026-09-23, and put no
    /// zone under the player at all.
    ///
    /// Where the matrix has a chunk with a deck over its ground, the ground is stacked: walked a
    /// floor at a time, each step onto the floor beside it it joins (`Ground.climb`). Otherwise a
    /// chunk with a deck is walked on the floor nearest the player's height, cell by cell
    /// (`mapblock.flat`, spike 156), or nearest `height` where one is given.
    pub fn ground(&self, r#where: &Field, height: Option<f64>) -> Result<Ground> {
        if r#where.on_rail {
            let corridor = self
                .rails
                .corridor
                .borrow()
                .clone()
                .map(|corridor| corridor as Rc<dyn common::Corridor>);
            return Ok(Ground::of(
                r#where,
                Of {
                    corridor,
                    ..Of::default()
                },
            ));
        }
        let bases = &self.secret_bases;
        let ice = match &self.ice {
            None => Vec::new(),
            Some(ice) => ice.rooms(r#where.zone)?,
        };
        // A cracked tile drops the player through, on its own room's floor (spike 167).
        let mut cracked: IndexMap<Cell, Option<f64>> = IndexMap::new();
        for room in &ice {
            if !room.cleared {
                for (cell, state) in room.cells() {
                    if state == CRACKED {
                        cracked.insert(cell, Some(room.height));
                    }
                }
            }
        }
        let files = self.terrain.files.borrow().1.clone();
        let lanes = self.rails.lanes(&files)?;
        let mut covered = self.covered(r#where.zone)?;
        covered.extend(lanes.keys().copied());
        let mut solid: IndexMap<Cell, Option<f64>> = IndexMap::new();
        if let Some(bases) = bases
            && bases.zones().contains(&r#where.zone)
        {
            for cell in bases.solid()? {
                if !covered.contains(&cell) {
                    solid.insert(cell, None);
                }
            }
        }
        solid.extend(self.unseen(r#where.zone)?);
        solid.extend(cracked);
        let loaded = || self.chunks_loaded();
        let stacked = !self.terrain.decks(self.height.get(), &loaded)?.is_empty();
        let level = if height.is_none() || stacked {
            self.height.get()
        } else {
            height
        };

        let mut chunks: Vec<Rc<Chunk>> = Vec::new();
        for chunk in self.chunks_loaded()? {
            chunks.push(opened(
                laid(&self.terrain, &chunk, stacked, level)?,
                &covered,
            ));
        }
        let decks = if stacked {
            Vec::new()
        } else {
            self.terrain.decks(level, &loaded)?
        };
        let ladders = self.ladders_here()?;
        let cliffs = self.terrain.cliffs(level, &ladders)?;
        // From the models, not the listing: guiding asks with nothing listed.
        let holes = self.holes(r#where.zone)?;
        let filled = if holes.is_empty() {
            HashSet::new()
        } else {
            self.dropped(&holes, &self.people(r#where.zone)?)
        };
        let doors = match &self.doors {
            None => Vec::new(),
            Some(doors) => doors.doors(r#where.zone)?,
        };
        let (hops, falls) = (
            self.terrain.rises(&BUMPY_SLOPES)?,
            self.terrain.rises(&[FALLS])?,
        );
        let surfaces = self.surfaces(r#where.zone)?;
        let trusted = self.zone_at(r#where.cell)?.is_some();
        let me = self
            .me
            .upgrade()
            .expect("a reader is made in an Rc, which its holders keep");
        let far_covered = covered.clone();
        let far: Far = Rc::new(move |column, row| match me.terrain.far(column, row)? {
            None => Ok(None),
            Some(chunk) => Ok(Some(opened(
                laid(&me.terrain, &chunk, stacked, level)?,
                &far_covered,
            ))),
        });
        let grid = if trusted {
            self.grid.borrow().clone()
        } else {
            None
        };
        let belts = self.belts_here(r#where.zone)?;
        let ground = Ground {
            decks,
            cliffs,
            ladders,
            doors,
            hops,
            falls,
            surfaces,
            stacked,
            level,
            ice,
            filled,
            strands: self.strands,
            lanes,
            ..Ground::of(
                r#where,
                Of {
                    chunks,
                    grid,
                    belts,
                    far: if trusted && self.terrain.known() {
                        Some(far)
                    } else {
                        None
                    },
                    clears: self.clears.clone(),
                    solid,
                    ..Of::default()
                },
            )
        };
        // Where no flat ground lies, nothing is got on or off, and no search carries a ride.
        let flats = !self.found_ways(r#where.zone)?.3.is_empty();
        match &self.rides {
            Some(rides) if flats => ground.mounted(
                r#where,
                &rides.ground,
                &rides.stops,
                &rides.mounts,
                &rides.smashes,
            ),
            _ => Ok(ground),
        }
    }

    /// Under the water, each cell of deep water in the zone with every one the sea above joins it
    /// to (`Ground.surfaces`); none above the water, or where the sea above is not found.
    fn surfaces(&self, zone: i64) -> Result<IndexMap<Cell, Rc<HashSet<Cell>>>> {
        if !self.surfaces_of.borrow().contains_key(&zone) {
            let found = self.find_surfaces(zone)?;
            self.surfaces_of.borrow_mut().insert(zone, found);
        }
        Ok(self.surfaces_of.borrow()[&zone].clone())
    }

    /// The sea above is the zone of the same place that is its own parent and is not under the
    /// water (`Terrain.surfaces`).
    fn find_surfaces(&self, zone: i64) -> Result<IndexMap<Cell, Rc<HashSet<Cell>>>> {
        let (row, rows) = (self.zones.row(zone)?, self.zones.rows()?);
        let (Some(row), Some(rows)) = (row, rows) else {
            return Ok(IndexMap::new());
        };
        if u16(&row, INFO_KIND)? != UNDERWATER || !self.terrain.known() {
            return Ok(IndexMap::new());
        }
        let mut below: IndexSet<Cell> = IndexSet::new();
        for patch in self.found_ways(zone)?.1 {
            below.extend(patch);
        }
        let mut seas: Vec<i64> = Vec::new();
        for other in 0..ZONE_LIMIT {
            let info = py::slice(&rows, other * INFO_SIZE, (other + 1) * INFO_SIZE);
            if !info.is_empty()
                && other != zone
                && u16(info, INFO_PARENT)? == other
                && u16(info, INFO_KIND)? != UNDERWATER
                && u16(info, INFO_NAME)? & NAME_MASK == u16(&row, INFO_NAME)? & NAME_MASK
            {
                seas.push(u16(info, INFO_MATRIX)?);
            }
        }
        self.terrain.surfaces(&below, &seas)
    }

    /// The zone's ladders (`ladders`), none without its zone data.
    fn ladders_here(&self) -> Result<IndexMap<(Cell, Cell), (f64, f64)>> {
        match self.current()? {
            None => Ok(IndexMap::new()),
            Some(data) => ladders(&read_triggers(&*self.memory, &data)?),
        }
    }

    /// The cell the player's Secret Base's decorating cursor is on and what is there, or None while
    /// it is not shown: the decorations standing in it, by the game's names for them, or the PC or
    /// the flag, or a wall, or blank. The game names a decoration on the bottom screen only, says
    /// "Select location" alike over the floor, the PC and the walls, and nothing at all moving from
    /// one empty cell to the next (omega_ruby_secret-base2.ctm).
    pub fn decorating(&self, r#where: &Field) -> Result<Option<(Cell, String)>> {
        let cursor = match &self.secret_bases {
            Some(bases) if bases.zones().contains(&r#where.zone) => bases.cursor()?,
            _ => None,
        };
        let (Some(bases), Some((cell, fits))) = (&self.secret_bases, cursor) else {
            return Ok(None);
        };
        let under = kind_said(&self.under_cursor(r#where, bases.as_ref(), cell)?);
        Ok(Some((
            cell,
            match fits {
                None => under,
                Some(fits) => format!("{}, {under}", own(fit(fits), &[])),
            },
        )))
    }

    fn under_cursor(
        &self,
        r#where: &Field,
        bases: &dyn SecretBaseReader,
        cell: Cell,
    ) -> Result<String> {
        let mut names: Vec<&str> = Vec::new();
        let standing = bases.standing(cell)?;
        for number in standing {
            if number < self.decorations.len() as i64
                && !self.decorations[number as usize].is_empty()
            {
                names.push(&self.decorations[number as usize]);
            }
        }
        if !names.is_empty() {
            return Ok(names.join(", "));
        }
        let data = self.current()?;
        let home = match &data {
            None => None,
            Some(data) => self.home(data, &[r#where.zone])?,
        };
        let items = match &data {
            None => Vec::new(),
            Some(data) => read_furniture(&*self.memory, data)?,
        };
        for item in items {
            if item.cells().contains(&cell) {
                return Ok(self.furniture_kind(home, &item));
            }
        }
        let chunks = self.chunks_loaded()?;
        if walkable(&chunks, cell)? != Some(false) {
            return Ok(BLANK.to_string());
        }
        let Some(held) = chunks.iter().find(|chunk| chunk.holds(cell)) else {
            return Ok(WALL.to_string());
        };
        Ok(base_tile(held.kind(cell)?).unwrap_or(WALL).to_string())
    }

    /// The cells of the zone's conveyor belts, each with the way it runs: its triggers whose
    /// scripts `Game.belts` knows as belts.
    fn belts_here(&self, zone: i64) -> Result<IndexMap<Cell, &'static str>> {
        let data = self.current()?;
        let Some(data) = data.filter(|_| !self.belts.is_empty()) else {
            return Ok(IndexMap::new());
        };
        let Some(home) = self.home(&data, &[zone])? else {
            return Ok(IndexMap::new());
        };
        let mut out: IndexMap<Cell, &'static str> = IndexMap::new();
        for record in read_triggers(&*self.memory, &data)? {
            let Some(&way) = self.belts.get(&(home, u16(&record, TRIGGER_SCRIPT)?)) else {
                continue;
            };
            let (x, z) = (u16(&record, TRIGGER_X)?, u16(&record, TRIGGER_Z)?);
            for dx in 0..u16(&record, TRIGGER_WIDTH)? {
                for dz in 0..u16(&record, TRIGGER_HEIGHT)? {
                    out.insert((x + dx, z + dz), way);
                }
            }
        }
        Ok(out)
    }

    /// The zone's triggers `Game.triggers` knows as something to walk to: Mauville Gym's switches,
    /// and the way between its two rooms, which share no floor (spike 146); Lavaridge Gym's
    /// trapdoors and geysers (spike 157); Mossdeep Gym's lifts and teleporter (spike 163); the way
    /// into Seafloor Cavern under Route 128 (spike 164).
    fn listed_triggers(&self, data: &ZoneData, zone: i64) -> Result<Vec<Interactable>> {
        let home = if self.triggers.is_empty() {
            None
        } else {
            self.home(data, &[zone])?
        };
        let Some(home) = home else {
            return Ok(Vec::new());
        };
        let mut out: Vec<Interactable> = Vec::new();
        for (i, record) in read_triggers(&*self.memory, data)?.iter().enumerate() {
            let script = u16(record, TRIGGER_SCRIPT)?;
            if let Some((category, kind, to)) = self.triggers.get(&(home, script)) {
                out.push(Interactable {
                    size: (u16(record, TRIGGER_WIDTH)?, u16(record, TRIGGER_HEIGHT)?),
                    zone: *to,
                    code: script,
                    source: place_source(TRIGGER_SOURCE, i as i64)?,
                    home: Some(home),
                    height: Some(s16(record, TRIGGER_Y)? as f64),
                    ..Interactable::new(
                        category,
                        kind,
                        (u16(record, TRIGGER_X)?, u16(record, TRIGGER_Z)?),
                    )
                });
            }
        }
        Ok(out)
    }

    /// `where` with everything worth walking to in its zone listed.
    pub fn interactables(&self, r#where: &Field) -> Result<Field> {
        let corridor = self.rails.corridor.borrow().clone();
        if let Some(corridor) = corridor.filter(|_| r#where.on_rail) {
            return Ok(Field {
                interactables: self.on_the_rails(&corridor, r#where.zone)?,
                ..r#where.clone()
            });
        }
        let mut items: Vec<Interactable> = Vec::new();
        let data = self.current()?;
        let mut people = self.people(r#where.zone)?;
        let kind = |cell: Cell| -> Result<Option<i64>> {
            // The ROM's chunk where none is loaded, as the ground has it: a battle unloads them all.
            let loaded = self
                .chunks_loaded()?
                .into_iter()
                .find(|chunk| chunk.holds(cell));
            let chunk = match loaded {
                Some(chunk) => Some(chunk),
                None => self
                    .terrain
                    .far(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK))?,
            };
            match chunk {
                Some(chunk) => Ok(Some(chunk.kind(cell)?)),
                None => Ok(None),
            }
        };
        if let Some(rides) = &self.rides {
            let both: HashSet<i64> = rides.ground.union(&rides.stops).copied().collect();
            if riding(r#where, &kind, &both)? {
                // The ridden Pokémon's model stays where it was got on: nobody to walk to (spike
                // 238).
                people.retain(|item| !rides.mounts.contains(&item.code));
            }
        }
        let mut home = None;
        if let Some(data) = &data {
            home = self.home(data, &[r#where.zone])?;
            let mut furniture: Vec<Interactable> = Vec::new();
            for item in read_furniture(&*self.memory, data)? {
                furniture.push(self.furniture_listed(home, item));
            }
            items.extend(joined(furniture, &|item| Some(self.furniture_key(item))));
            let warps = read_records(&*self.memory, data, WARP)?;
            for (i, record) in warps.iter().enumerate() {
                let files = self.terrain.files.borrow().1.clone();
                let shape = self.rails.on_grid(record, &files)?;
                if shape.is_none() && on_rail(record)? {
                    continue;
                }
                let at = match &shape {
                    None => None,
                    Some(shape) => Some(rail::spread(shape.iter().copied())?),
                };
                let warp = self.warp(i as i64, record, at, home)?;
                items.push(Interactable {
                    home,
                    arrival: arrival(record, home, &warps)?,
                    shape: shape.unwrap_or_default(),
                    ..warp
                });
            }
        }
        items.extend(self.boulders(people, r#where.zone, data.as_ref(), home)?);
        items.extend(self.edges(r#where.zone, !r#where.on_rail)?);
        items.extend(self.use_tiles(r#where.zone)?);
        items.extend(self.ways(r#where.zone, r#where.cell)?);
        items.extend(self.stairs_in(r#where.zone)?);
        items.extend(self.base_tiles(r#where.zone)?);
        items.extend(self.placed(r#where.zone)?);
        if let Some(data) = &data {
            items.extend(self.listed_triggers(data, r#where.zone)?);
        }
        let zone_at = |cell: Cell| self.zone_at(cell);
        for feature in &self.features {
            items.extend(feature.listed(r#where.zone, &zone_at)?);
        }
        if let Some(bases) = &self.secret_bases {
            items.extend(bases.listed(r#where.zone, &zone_at)?);
        }
        if let Some(hidden) = &self.hidden {
            items.extend(hidden.listed(r#where.zone, &zone_at)?);
        }
        // A boulder in a hole is floor now, and neither it nor the hole is anything to find.
        let holes = self.holes(r#where.zone)?;
        if !holes.is_empty() {
            let gone = self.dropped(&holes, &items);
            if !gone.is_empty() {
                items.retain(|item| !item.cells().iter().all(|cell| gone.contains(cell)));
            }
        }
        if self.rooms.contains(&r#where.zone) {
            items = self.in_room(items, r#where.cell)?;
        }
        Ok(Field {
            interactables: one_a_spot(items),
            ..r#where.clone()
        })
    }

    /// `items` less the zone's other rooms' (`Game.rooms`): those on or beside the open floor
    /// joined side to side to the player's cell, as a sign on a wall is. All of them where the
    /// player stands on no open floor the chunks loaded hold.
    fn in_room(&self, items: Vec<Interactable>, player: Cell) -> Result<Vec<Interactable>> {
        let mut floor: HashSet<Cell> = HashSet::new();
        for chunk in self.chunks_loaded()? {
            for (cell, _) in chunk.kinds() {
                if chunk.walkable(cell)? {
                    floor.insert(cell);
                }
            }
        }
        let room: Option<HashSet<Cell>> = touching(&floor)
            .into_iter()
            .find(|group| group.contains(&player))
            .map(|group| group.into_iter().collect());
        let Some(room) = room else {
            return Ok(items);
        };
        Ok(items
            .into_iter()
            .filter(|item| {
                item.cells().iter().any(|&cell| {
                    std::iter::once(cell)
                        .chain(beside(cell))
                        .any(|near| room.contains(&near))
                })
            })
            .collect())
    }

    /// `people` with each big boulder told the ways it moves (`Interactable.pushes`), and
    /// where its record starts it where the record is `home`'s (`Interactable.start`): the
    /// record keeps the room's own cell as the boulder is pushed about (spike 202). A boulder in
    /// its hole is floor, and nobody in the way.
    fn boulders(
        &self,
        people: Vec<Interactable>,
        zone: i64,
        data: Option<&ZoneData>,
        home: Option<i64>,
    ) -> Result<Vec<Interactable>> {
        if !people
            .iter()
            .any(|item| self.clears.get(&item.code) == Some(&STRENGTH))
        {
            return Ok(people);
        }
        let mut starts: HashMap<i64, Cell> = HashMap::new();
        if let Some(data) = data {
            for record in read_records(&*self.memory, data, NPC)? {
                starts.insert(
                    u16(&record, NPC_INDEX)?,
                    (u16(&record, NPC_X)?, u16(&record, NPC_Z)?),
                );
            }
        }
        let holes = self.holes(zone)?;
        let filled = self.dropped(&holes, &people);
        let standing: HashSet<Cell> = people
            .iter()
            .flat_map(|item| item.cells())
            .filter(|cell| !filled.contains(cell))
            .collect();
        let mut chunks: Vec<Rc<Chunk>> = Vec::new();
        for chunk in self.chunks_loaded()? {
            chunks.push(self.terrain.flattened(&chunk, None, false)?);
        }
        let floor = |cell: Cell| -> Result<bool> {
            Ok(
                (filled.contains(&cell) || walkable(&chunks, cell)? == Some(true))
                    && !standing.contains(&cell),
            )
        };

        let mut out: Vec<Interactable> = Vec::new();
        for mut item in people {
            if self.clears.get(&item.code) == Some(&STRENGTH)
                && !item.cells().iter().all(|cell| holes.contains(cell))
            {
                // Going through a passage out of the room, a poll found its chunks gone and
                // every boulder immoveable (movie37, spike 202): with no walls, nothing is said.
                if !chunks.is_empty() {
                    item.pushes = Some(pushes(item.cells(), &floor, &holes)?);
                }
                if let Some((zone, index)) = item.record
                    && Some(zone) == home
                    && let Some(&cell) = starts.get(&index)
                {
                    let (wide, deep) = item.size;
                    item.start = Some((
                        cell.0 - py::floordiv(wide - 1, 2),
                        cell.1 - py::floordiv(deep - 1, 2),
                    ));
                }
            }
            out.push(item);
        }
        Ok(out)
    }

    /// The cells of the zone's holes a big boulder drops into (`BOULDER_HOLE`).
    fn holes(&self, zone: i64) -> Result<HashSet<Cell>> {
        Ok(self
            .found_ways(zone)?
            .0
            .iter()
            .filter(|item| item.code == BOULDER_HOLE)
            .flat_map(|item| item.cells())
            .collect())
    }

    /// The cells of every big boulder among `items` lying wholly in `holes`: pushed into one,
    /// it is floor the player walks over, its model still there (spike 202).
    fn dropped(&self, holes: &HashSet<Cell>, items: &[Interactable]) -> HashSet<Cell> {
        items
            .iter()
            .filter(|item| {
                item.move_model
                    && self.clears.get(&item.code) == Some(&STRENGTH)
                    && item.cells().iter().all(|cell| holes.contains(cell))
            })
            .flat_map(|item| item.cells())
            .collect()
    }

    /// The Secret Base's crumbled cliff and holes, each patch of touching cells of a kind in
    /// `BASE_TILES` the zone's chunks hold as one thing over its bounding box: `runs` would split
    /// the cliff's 3 cells by 2 into two rows and list it twice. A cell of the map, so only the
    /// chunks loaded now say where it is. Nothing once a decoration stands on any of its cells:
    /// the Stairs set down over the cliff left the cell table at kind `CLIFF_TILE` and the base's
    /// mapper with a decoration in each of the six (omega_ruby_secret-base4.ctm).
    fn base_tiles(&self, zone: i64) -> Result<Vec<Interactable>> {
        let mut found: IndexMap<i64, HashSet<Cell>> = IndexMap::new();
        for chunk in self.chunks_loaded()? {
            for (cell, kind) in chunk.kinds() {
                if base_tile(kind).is_some() {
                    let at = self.zone_at(cell)?;
                    if at == Some(zone) || at.is_none() {
                        found.entry(kind).or_default().insert(cell);
                    }
                }
            }
        }
        let covered = self.covered(zone)?;
        let mut out: Vec<Interactable> = Vec::new();
        for (kind, cells) in found {
            for group in touching(&cells) {
                if group.iter().any(|cell| covered.contains(cell)) {
                    continue;
                }
                let (first, size) = spread(&group);
                out.push(Interactable {
                    size,
                    code: kind,
                    source: place_source(TILE_SOURCE, first.0 << 12 | first.1)?,
                    ..Interactable::new(
                        "exits",
                        base_tile(kind).expect("a base tile's kind"),
                        first,
                    )
                });
            }
        }
        Ok(out)
    }

    /// The cells of the Secret Base's crumbled cliff and holes a decoration stands on, walked
    /// over: the player climbed the Stairs set down over the cliff to the upper floor, the cell
    /// table keeping all six cells blocked under them, and P said "can't continue"
    /// (omega_ruby_secret-base4.ctm). The Solid Board over a hole is taken to be crossed the same
    /// way, by its description; none has been set down yet.
    fn covered(&self, zone: i64) -> Result<HashSet<Cell>> {
        let Some(bases) = &self.secret_bases else {
            return Ok(HashSet::new());
        };
        if !bases.zones().contains(&zone) {
            return Ok(HashSet::new());
        }
        let mut out = HashSet::new();
        for chunk in self.chunks_loaded()? {
            for (cell, kind) in chunk.kinds() {
                if base_tile(kind).is_none() {
                    continue;
                }
                let at = self.zone_at(cell)?;
                if (at == Some(zone) || at.is_none()) && !bases.standing(cell)?.is_empty() {
                    out.insert(cell);
                }
            }
        }
        Ok(out)
    }

    /// The decorations set down in a Secret Base, among the signs by the game's names for them,
    /// each one thing over its cells: set down, the Stairs and the desks were not listed at all
    /// (the user, 2026-10-10). The Invisible Doll is not, as the game hides it.
    ///
    /// Those of one name that touch are one thing: every base's five Welcome Mats in a line were
    /// listed five times (the user, 2026-10-10). Apart, they stay apart, so the way to one does not
    /// end on the floor between them.
    fn placed(&self, zone: i64) -> Result<Vec<Interactable>> {
        let Some(bases) = &self.secret_bases else {
            return Ok(Vec::new());
        };
        if !bases.zones().contains(&zone) {
            return Ok(Vec::new());
        }
        let mut named: IndexMap<String, HashSet<Cell>> = IndexMap::new();
        for (number, cells) in bases.placed()? {
            if number < self.decorations.len() as i64
                && !self.decorations[number as usize].is_empty()
            {
                named
                    .entry(self.decorations[number as usize].clone())
                    .or_default()
                    .extend(cells);
            }
        }
        let mut out: Vec<Interactable> = Vec::new();
        for (name, cells) in named {
            for group in touching(&cells) {
                let (first, size) = spread(&group);
                out.push(Interactable {
                    size,
                    source: place_source(DECORATION_SOURCE, cell_number(first))?,
                    ..Interactable::new("signs", &name, first)
                });
            }
        }
        Ok(out)
    }

    /// The stairs between the floors of the zone's chunks, where it is one of `Game.stairs`: each
    /// run of the cells of the ground a floor over it is walked onto from, at the ground's height
    /// there (`mapblock.joins`). The game names none of them. Sootopolis Gym's two, each two cells
    /// wide at (11, 59) and (24, 59), climb from its lower floor to 189 beside the entrance's room
    /// at 198, the only way back up (spike 167).
    fn stairs_in(&self, zone: i64) -> Result<Vec<Interactable>> {
        if !self.stairs.contains(&zone) {
            return Ok(Vec::new());
        }
        let mut tops: IndexMap<Cell, f64> = IndexMap::new();
        for chunk in self.chunks_loaded()? {
            tops.extend(mapblock::joins(&chunk));
        }
        let mut out: Vec<Interactable> = Vec::new();
        for (first, size) in runs(tops.keys().copied()) {
            out.push(Interactable {
                size,
                source: place_source(STAIRS_SOURCE, first.0 << 12 | first.1)?,
                home: Some(zone),
                height: Some(tops[&first]),
                ..Interactable::new("exits", STAIRS, first)
            });
        }
        Ok(out)
    }

    /// What makes furniture records that touch one thing (`_joined`): their script, which the
    /// game runs whichever of them the player faces, as round Seafloor Cavern's submarine, four
    /// records saying "“Submarine Explorer 1” is painted on the hull" and listed as four signs
    /// (Orion, 2026-10-13). The ROM has 33 such pairs in 12 zones. An elevator's door is two
    /// records with a shared script each (`Game.elevators`), both of which run the elevator.
    ///
    /// Python's key is the script or `ELEVATOR`, an int or a str; here both are text, which no
    /// script's number spells as `ELEVATOR` does.
    fn furniture_key(&self, item: &Interactable) -> String {
        if self.elevators.contains(&item.code) {
            ELEVATOR.to_string()
        } else {
            item.code.to_string()
        }
    }

    /// What a furniture record is: a sign, unless the game's table of them says otherwise, or
    /// the shipped table says what the shared script it runs is: the one its zone's case runs,
    /// or its own number, which is a shared script's from 2000 up (spike 210). Every vending
    /// machine is one of five, 0x28f0 to 0x28f4. The PC in a Secret Base is called what the
    /// game calls a PC, as the Pokémon Center's is: the table gives it the PC's kind of use
    /// tile. A record the player leaves by is a category, a kind and a zone, and its kind is the
    /// second.
    fn furniture_kind(&self, home: Option<i64>, item: &Interactable) -> String {
        let kind = match home {
            Some(home) => self.furniture_kinds.get(&(home, item.code)),
            None => None,
        };
        let Some(kind) = kind else {
            let shared = match home {
                None => item.code,
                Some(home) => self
                    .shared
                    .get(&(home, item.code))
                    .copied()
                    .unwrap_or(item.code),
            };
            return self
                .furniture_words
                .get(&shared)
                .cloned()
                .unwrap_or_else(|| item.kind.clone());
        };
        match kind {
            FurnitureKind::Tile(tile) => self
                .words
                .get(tile)
                .cloned()
                .unwrap_or_else(|| PROP.to_string()),
            FurnitureKind::Word(word) => word.clone(),
            FurnitureKind::Exit(_, kind, _) => kind.clone(),
        }
    }

    /// A furniture record as the list has it: its kind (`_furniture_kind`), and where the
    /// table sends the player through it, the category it is listed under and the zone it leads
    /// to (`Game.furniture`).
    fn furniture_listed(&self, home: Option<i64>, item: Interactable) -> Interactable {
        let listed = Interactable {
            kind: self.furniture_kind(home, &item),
            home,
            ..item
        };
        let known = home.and_then(|home| self.furniture_kinds.get(&(home, listed.code)));
        if let Some(FurnitureKind::Exit(category, _, zone)) = known {
            return Interactable {
                category: category.clone(),
                zone: Some(*zone),
                ..listed
            };
        }
        listed
    }

    /// Why to sweep now rather than on schedule, or None.
    ///
    /// A zone's edges were listed whole because no block held a chunk, no sweep has been asked for
    /// since the last one, and fewer than `ASKS` have been asked for the zone. The player's own
    /// chunk is always loaded, so no chunk at all means the blocks are not known yet, or moved;
    /// only a sweep finds them.
    ///
    /// Or the zone data is in and the zone table is not, which names every place and tells a path
    /// from stairs, and only a sweep finds it (`TABLE_ASKS`).
    ///
    /// Nothing while the field is not the app `running`: a battle tears the map blocks down where
    /// they are and builds them again there, which the field reads as no chunk, and all 72 asks
    /// over the recordings of ORAS and X came in another app, 32 whole sweeps in movie4 (spike
    /// 130). The zone's asks are spent instead, so its want is not swept for the moment the field
    /// comes back.
    pub fn sweep_wanted(&self, running: bool) -> Result<Option<&'static str>> {
        if !running {
            self.asks.set(ASKS);
            return Ok(None);
        }
        if self.wanted.get().is_some()
            && self.wanted.get() != self.asked.get()
            && self.asks.get() < ASKS
        {
            self.asked.set(self.wanted.get());
            self.asks.set(self.asks.get() + 1);
            return Ok(Some("no chunk loaded, sweeping for the map blocks"));
        }
        if self.zones.base.get().is_none()
            && self.holder.borrow().is_some()
            && self.table_asks.get() < TABLE_ASKS
        {
            let now = (self.clock)();
            if now >= self.table_next.get() {
                self.table_next.set(now + TABLE_EVERY);
                self.table_asks.set(self.table_asks.get() + 1);
                return Ok(Some("no zone table, sweeping for it"));
            }
        }
        let in_a_base = match (&self.secret_bases, self.visit.get()) {
            (Some(bases), Some(visit)) => bases.zones().contains(&visit.1),
            _ => false,
        };
        if !in_a_base {
            self.base_asks.set(0);
        } else if let Some(bases) = &self.secret_bases
            && !bases.furnished()?
            && self.base_asks.get() < BASE_ASKS
        {
            let now = (self.clock)();
            if now >= self.base_next.get() {
                self.base_next.set(now + BASE_EVERY);
                self.base_asks.set(self.base_asks.get() + 1);
                return Ok(Some(
                    "in a Secret Base with nothing found standing in it, sweeping for it",
                ));
            }
        }
        Ok(None)
    }

    /// The record's words as they are, with the zone the matrix puts its cell in, for the log; None
    /// before a sweep has found the record, and in a game without one.
    pub fn hidden_words(&self) -> Result<Option<String>> {
        let held = match &self.hidden {
            None => None,
            Some(hidden) => hidden.words()?,
        };
        let Some((words, cell)) = held else {
            return Ok(None);
        };
        let zone = match self.zone_at(cell)? {
            None => "none".to_string(),
            Some(zone) => format!("{zone:#x}"),
        };
        Ok(Some(format!("{words} zone {zone}")))
    }

    /// The game's hidden Pokémon, while one is up in `zone`: read at every poll, since it comes and
    /// goes between sweeps.
    pub fn hidden(&self, zone: i64) -> Result<Option<Interactable>> {
        let listed = match &self.hidden {
            None => Vec::new(),
            Some(hidden) => hidden.listed(zone, &|cell| self.zone_at(cell))?,
        };
        Ok(listed.into_iter().next())
    }

    /// Which of the zone's triggers, by its place in the zone's table, the cell is inside, of
    /// those `among`; None for none (`scenes.Scenes.stepped`). The boxes are read once a zone,
    /// from its own zone data: not the zone before's, which can still be in use as the player
    /// arrives.
    pub fn trigger_at(&self, zone: i64, cell: Cell, among: &HashSet<i64>) -> Result<Option<i64>> {
        if !self.trigger_boxes.borrow().contains_key(&zone) {
            let Some(data) = self.current()? else {
                return Ok(None);
            };
            if self.home(&data, &[zone])? != Some(zone) {
                return Ok(None);
            }
            let mut boxes: Vec<(i64, i64, i64, i64)> = Vec::new();
            for record in read_triggers(&*self.memory, &data)? {
                boxes.push((
                    u16(&record, TRIGGER_X)?,
                    u16(&record, TRIGGER_Z)?,
                    u16(&record, TRIGGER_WIDTH)?,
                    u16(&record, TRIGGER_HEIGHT)?,
                ));
            }
            self.trigger_boxes.borrow_mut().insert(zone, boxes);
        }
        Ok(self.trigger_boxes.borrow()[&zone]
            .iter()
            .enumerate()
            .find(|(i, (x, z, width, height))| {
                among.contains(&(*i as i64))
                    && *x <= cell.0
                    && cell.0 < x + width
                    && *z <= cell.1
                    && cell.1 < z + height
            })
            .map(|(i, _)| i as i64))
    }

    /// Which zone a zone data file is (`ZoneTable.home`): the zones `likely` and the player's
    /// model's are looked at first.
    fn home(&self, data: &ZoneData, likely: &[i64]) -> Result<Option<i64>> {
        let mut zones: Vec<i64> = likely.to_vec();
        if let Some(visit) = self.visit.get() {
            zones.push(visit.1);
        }
        let info = self.memory.read(data.info(), INFO_SIZE)?;
        self.zones.home(&info, &zones)
    }

    /// What is on the corridor's lines: the signs and warps placed on them, the zone's own and
    /// those of the other zones of its map out of the ROM, and the people standing on them, in the
    /// corridor's cells. Edges and the cells used by facing them are the grid's, and the chunks
    /// under the rails mark every cell a wall, so neither is.
    fn on_the_rails(&self, corridor: &rail::Corridor, zone: i64) -> Result<Vec<Interactable>> {
        let mut out: Vec<Interactable> = Vec::new();
        let mut tables: Vec<(Option<i64>, Vec<Vec<u8>>, Vec<Vec<u8>>)> = Vec::new();
        let data = self.current()?;
        let home = match &data {
            None => None,
            Some(data) => self.home(data, &[zone])?,
        };
        if let Some(data) = &data {
            tables.push((
                home,
                read_records(&*self.memory, data, FURNITURE)?,
                read_records(&*self.memory, data, WARP)?,
            ));
        }
        let matrix = self.zones.matrix(home.unwrap_or(zone))?;
        let mut others: Vec<(&i64, &(Vec<Vec<u8>>, Vec<Vec<u8>>))> =
            self.rails.records.iter().collect();
        others.sort_by_key(|(other, _)| **other);
        for (&other, (furniture, warps)) in others {
            if Some(other) != home
                && let Some(matrix) = matrix
                && self.zones.matrix(other)? == Some(matrix)
            {
                tables.push((Some(other), furniture.clone(), warps.clone()));
            }
        }
        for (owner, furniture, warps) in tables {
            for (i, record) in furniture.iter().enumerate() {
                if let Some(at) = rail::spot(corridor, record)? {
                    out.push(Interactable {
                        size: at.1,
                        code: u16(record, FURNITURE_SCRIPT)?,
                        source: place_source(FURNITURE_SOURCE, i as i64)?,
                        home: owner,
                        ..Interactable::new("signs", "sign", at.0)
                    });
                }
            }
            for (i, record) in warps.iter().enumerate() {
                if let Some(at) = rail::spot(corridor, record)? {
                    let warp = self.warp(i as i64, record, Some(at), owner)?;
                    out.push(Interactable {
                        home: owner,
                        ..warp
                    });
                }
            }
        }
        for other in self.others() {
            let (model, head) = other?;
            if !mmodel::on_rail(&head)? {
                continue;
            }
            let (line, a, x) = mmodel::rail(&head)?;
            if let Some(cell) = corridor.at(line, a, x)? {
                out.push(self.model_item(model, &head, cell, (1, 1), None)?);
            }
        }
        Ok(out)
    }

    /// A warp as what the player walks through: a door, the mat out, stairs or an opening
    /// between rooms, or the path between two outdoor matrices, told by the kind and whether the
    /// zones it joins are outdoors; and what the game's table of kinds says of the rest, as a
    /// ladder or a teleporter pad (`Game.warps`). The place it arrives in is said by the
    /// navigator, which has the game's words (`Navigator.destination`).
    fn warp(
        &self,
        index: i64,
        record: &[u8],
        at: Option<(Cell, (i64, i64))>,
        home: Option<i64>,
    ) -> Result<Interactable> {
        let (to, kind) = (u16(record, WARP_TO)?, u16(record, WARP_KIND)?);
        let (cell, size) = match at {
            None => warp_box(record)?,
            Some(at) => at,
        };
        let what: String = if kind >> 8 == WARP_DOOR >> 8
            && match home {
                None => false,
                Some(home) => self.zones.open_air(home)?,
            }
            && self.zones.open_air(to)?
        {
            // A door's kind between two of the open air is the way through: one of Lumiose's
            // streets onto the next (spike 220), Route 7 onto Route 6 (2026-10-19).
            "path".to_string()
        } else if kind == WARP_DOOR || (kind >> 8 == WARP_STREET && !self.outdoors.contains(&to)) {
            "door".to_string()
        } else if kind == WARP_OUT {
            "way out".to_string()
        } else if kind >> 8 == WARP_STAIRS {
            if self.zones.outdoors(to)? == Some(true) {
                "path".to_string()
            } else if at.is_none() {
                self.stairs_word(record, cell, size)?.to_string()
            } else {
                STAIRS.to_string()
            }
        } else if let Some(word) = self.warp_words.get(&kind) {
            word.clone()
        } else {
            // The high byte is what it is, the low the way it is walked into (spike 220).
            match kind >> 8 {
                high if high == WARP_DOOR >> 8 => "door",
                high if high == WARP_OUT >> 8 => "way out",
                _ => "warp",
            }
            .to_string()
        };
        let way_in = if at.is_none() && kind >> 8 == WARP_STAIRS {
            WALKED_INTO.get((kind & 0xFF) as usize).copied()
        } else {
            None
        };
        Ok(Interactable {
            size,
            zone: Some(to),
            code: kind,
            source: place_source(WARP_SOURCE, index)?,
            height: match at {
                None => Some(s16(record, WARP_Y)? as f64),
                Some(_) => None,
            },
            way_in,
            ..Interactable::new("exits", &what, cell)
        })
    }

    /// What a warp walked through is, from how its height stands to the ground it is walked
    /// from: "stairs up" over `WARP_RISE`, "stairs down" under it, and a "passage" between,
    /// an opening with no floor between its rooms. The kind's low byte says the way it is walked
    /// into, so the cell the player stands in is the one on the side opposite. "stairs" where the
    /// ROM's heights are not read, and for a way of walking the kind does not number.
    fn stairs_word(&self, record: &[u8], cell: Cell, size: (i64, i64)) -> Result<&'static str> {
        let way = u16(record, WARP_KIND)? & 0xFF;
        let (x, z) = cell;
        let (wide, deep) = size;
        let behind = [(x, z - 1), (x, z + deep), (x - 1, z), (x + wide, z)];
        let ground = if way < behind.len() as i64 {
            self.terrain.height(behind[way as usize])?
        } else {
            None
        };
        let Some(ground) = ground else {
            return Ok(STAIRS);
        };
        let rise = s16(record, WARP_Y)? as f64 - ground;
        if rise.abs() <= WARP_RISE {
            return Ok("passage");
        }
        Ok(if rise > 0.0 {
            "stairs up"
        } else {
            "stairs down"
        })
    }

    fn find_player(&self) -> Result<Option<i64>> {
        if let Some(player) = self.player.get()
            && self.is_player(player)?
        {
            return Ok(Some(player));
        }
        let mut found = None;
        let models = self.models.borrow().clone();
        for model in models {
            if self.is_player(model)? {
                found = Some(model);
                break;
            }
        }
        self.player.set(found);
        // A new model for the player means a warp, and a warp can be onto another matrix.
        self.read_map()?;
        Ok(self.player.get())
    }

    fn is_player(&self, model: i64) -> Result<bool> {
        let head = self.memory.read(model, mmodel::ZONE + 4)?;
        Ok(mmodel::in_use(&head)? && mmodel::index_of(&head)? == mmodel::PLAYER_INDEX)
    }

    /// The zone data in use now: the holder's first buffer.
    fn current(&self) -> Result<Option<ZoneData>> {
        let address = match self.holder.borrow().as_ref() {
            None => return Ok(None),
            Some(holder) => holder.address,
        };
        let address = u32(&self.memory.read(address, 4)?, 0)?;
        let known = self
            .holder
            .borrow()
            .as_ref()
            .and_then(|holder| holder.files.get(&address).cloned());
        if known.is_some() {
            return Ok(known);
        }
        let data = read_zone(&*self.memory, address)?;
        if let Some(data) = &data
            && let Some(holder) = self.holder.borrow_mut().as_mut()
        {
            holder.files.insert(address, data.clone());
        }
        Ok(data)
    }

    /// Every model in use but the player's, with its head, in the pool's order.
    ///
    /// Python's generator reads each head as its loop comes to it; so does this iterator, so the
    /// reads a loop makes of each model come between the heads, as there.
    fn others(&self) -> impl Iterator<Item = Result<(i64, Vec<u8>)>> + '_ {
        let models = self.models.borrow().clone();
        models.into_iter().filter_map(move |model| {
            let other = || -> Result<Option<(i64, Vec<u8>)>> {
                let head = self.memory.read(model, mmodel::HEAD)?;
                Ok(
                    (mmodel::in_use(&head)? && mmodel::index_of(&head)? != mmodel::PLAYER_INDEX)
                        .then_some((model, head)),
                )
            };
            other().transpose()
        })
    }

    /// What a model drawn as `code` is to the field (`Role`), by the game's tables of looks,
    /// barriers and blockers.
    fn role(&self, head: &[u8], code: i64) -> Result<Role> {
        Ok(if self.player_looks.contains(&code) {
            Role::Look
        } else if self.barriers.contains(&(mmodel::zone_of(head)?, code)) {
            Role::Barrier
        } else if self.blockers.contains(&code) {
            Role::Blocker
        } else if code == mmodel::NONDRAW {
            Role::Nothing
        } else {
            Role::Drawn
        })
    }

    /// Every model listed (`_role`): an item ball or a thing by its model code, a person
    /// otherwise. The moving truck has a person's kind of code and no script, and trainers have
    /// no script either, so it passes for a person until model codes have names. A big boulder
    /// in a hole is no parked model, however walled in: Seafloor Cavern's room 0x69 has walls
    /// north and west of its north-west hole, and the boulder pushed into it was left out, its
    /// hole still listed and a wall (movie37).
    ///
    /// A model drawn as nothing is called what it stands in for (`stand_ins`), those side by
    /// side as one (`_joined`): the three in front of Groudon are one "Groudon", the first by its
    /// record, and none once Groudon itself is listed. It is left out on a door, which is listed
    /// already (spike 203), and where it is one of a gate's (`Game.gates`), which is nobody
    /// (spike 223).
    fn people(&self, zone: i64) -> Result<Vec<Interactable>> {
        let mut out: Vec<Interactable> = Vec::new();
        // The stand-ins called what another record draws.
        let mut standing_in: Vec<Interactable> = Vec::new();
        let mut chunks: Vec<Rc<Chunk>> = Vec::new();
        for chunk in self.chunks_loaded()? {
            chunks.push(self.terrain.flattened(&chunk, None, false)?);
        }
        let holes = self.holes(zone)?;
        let mut stand_ins: Option<StandIns> = None;
        let none = HashSet::new();
        let rides = match &self.rides {
            None => &none,
            Some(rides) => &rides.ground,
        };
        for other in self.others() {
            let (model, head) = other?;
            let Some(cell) = mmodel::cell(&head)? else {
                continue;
            };
            if !holes.contains(&cell) && walled_in(&chunks, cell, rides)? {
                continue;
            }
            let code =
                mmodel::drawn_as(&*self.memory, model, &head, self.drawn, self.model_starts)?;
            let role = self.role(&head, code)?;
            if role == Role::Look || role == Role::Blocker {
                continue;
            }
            let mut stood_in = None;
            if role == Role::Nothing && mmodel::zone_of(&head)? == zone {
                if stand_ins.is_none() {
                    stand_ins = Some(self.stand_ins()?);
                }
                let (by_index, doors, gates) = stand_ins.as_ref().expect("just read");
                stood_in = by_index.get(&mmodel::index_of(&head)?).copied();
                if stood_in.is_none()
                    && (doors.contains(&cell) || gates.contains(&mmodel::index_of(&head)?))
                {
                    continue;
                }
            }
            let (cell, size) = mmodel::footprint(&head, cell)?;
            // `stood_in or code`: a stand-in's sprite is never NONDRAW, 0, so never falsy.
            let called = match stood_in {
                Some(sprite) if sprite != 0 => sprite,
                _ => code,
            };
            let item = self.model_item(model, &head, cell, size, Some(called))?;
            match stood_in {
                None => out.push(item),
                Some(_) => standing_in.push(item),
            }
        }
        let listed: HashSet<i64> = out.iter().map(|item| item.code).collect();
        standing_in.sort_by_key(|item| item.record.unwrap_or((0, 0)));
        for item in joined(standing_in, &|item| Some(item.code.to_string())) {
            if !listed.contains(&item.code) {
                out.push(item);
            }
        }
        Ok(out)
    }

    /// What the zone's models drawn as nothing stand in for, by their records' indices: the
    /// sprite of another record with the same script, as the three in a row in front of Groudon
    /// deep in the Cave of Origin have its script, where Groudon itself stands in the magma; the
    /// cells of the zone's warps, where Sootopolis City's ten stand, one on each locked door
    /// (spike 203); and, in a game with `gates`, the records' indices of the gates, models drawn
    /// as nothing that share a script with one another and with no drawn record: the five behind
    /// Viola in Santalune Gym, shut until the game opens the tent after her battle (spike 223).
    fn stand_ins(&self) -> Result<StandIns> {
        let Some(data) = self.current()? else {
            return Ok((IndexMap::new(), HashSet::new(), HashSet::new()));
        };
        let records = read_records(&*self.memory, &data, NPC)?;
        let mut unseen: HashSet<i64> = self.blockers.clone();
        unseen.insert(mmodel::NONDRAW);
        let mut drawn: IndexMap<i64, i64> = IndexMap::new();
        for record in &records {
            if !unseen.contains(&u16(record, NPC_SPRITE)?) && u16(record, NPC_SCRIPT)? != 0 {
                drawn.insert(u16(record, NPC_SCRIPT)?, u16(record, NPC_SPRITE)?);
            }
        }
        let mut stand_ins: IndexMap<i64, i64> = IndexMap::new();
        for record in &records {
            if u16(record, NPC_SPRITE)? == mmodel::NONDRAW
                && let Some(sprite) = drawn.get(&u16(record, NPC_SCRIPT)?)
            {
                stand_ins.insert(u16(record, NPC_INDEX)?, *sprite);
            }
        }
        let mut shared: HashMap<i64, i64> = HashMap::new();
        for record in &records {
            if u16(record, NPC_SPRITE)? == mmodel::NONDRAW {
                *shared.entry(u16(record, NPC_SCRIPT)?).or_default() += 1;
            }
        }
        let mut gates: HashSet<i64> = HashSet::new();
        for record in &records {
            if !self.gates || u16(record, NPC_SPRITE)? != mmodel::NONDRAW {
                continue;
            }
            let script = u16(record, NPC_SCRIPT)?;
            if script != 0 && !drawn.contains_key(&script) && shared[&script] > 1 {
                gates.insert(u16(record, NPC_INDEX)?);
            }
        }
        let mut doors: HashSet<Cell> = HashSet::new();
        for record in read_records(&*self.memory, &data, WARP)? {
            if on_rail(&record)? {
                continue;
            }
            let ((x0, z0), (wide, deep)) = warp_box(&record)?;
            for x in x0..x0 + 1.max(wide) {
                for z in z0..z0 + 1.max(deep) {
                    doors.insert((x, z));
                }
            }
        }
        Ok((stand_ins, doors, gates))
    }

    /// The cells of the models in the way but never listed as themselves, with how high: the
    /// invisible blockers, but round a zone's rotating doors, and the models drawn as nothing
    /// (`_role`).
    fn unseen(&self, zone: i64) -> Result<IndexMap<Cell, Option<f64>>> {
        let mut unseen: Vec<Role> = vec![Role::Nothing];
        let rotating = match &self.doors {
            Some(doors) => !doors.doors(zone)?.is_empty(),
            None => false,
        };
        if !rotating {
            unseen.push(Role::Blocker);
        }
        let mut out: IndexMap<Cell, Option<f64>> = IndexMap::new();
        for other in self.others() {
            let (model, head) = other?;
            let code =
                mmodel::drawn_as(&*self.memory, model, &head, self.drawn, self.model_starts)?;
            if unseen.contains(&self.role(&head, code)?)
                && let Some(cell) = mmodel::cell(&head)?
            {
                out.insert(cell, mmodel::height(&head)?);
            }
        }
        Ok(out)
    }

    /// A model as what it is, standing in `size` cells from `cell`: an item ball or a thing by its
    /// model code, a person otherwise; `code` is the sprite it is called by, where that is not the
    /// one it is drawn with.
    ///
    /// Told apart by its slot and the record it was built from: walking into the next zone fills
    /// the slots of the zone left with the new zone's people, and the way to a Youngster on Route
    /// 102 became a way to Norman in Petalburg City, in the same slot (spike 131).
    fn model_item(
        &self,
        model: i64,
        head: &[u8],
        cell: Cell,
        size: (i64, i64),
        code: Option<i64>,
    ) -> Result<Interactable> {
        let code = match code {
            Some(code) => code,
            None => mmodel::drawn_as(&*self.memory, model, head, self.drawn, self.model_starts)?,
        };
        let source = u32(head, mmodel::ZONE)? << 32 | model;
        let (category, kind) = if mmodel::ITEM_BALLS.contains(&code) {
            ("items", "item ball")
        } else if self.ambushes.contains(&code) {
            (POKEMON, PROP)
        } else if self.clears.contains_key(&code)
            || self
                .rides
                .as_ref()
                .is_some_and(|rides| rides.smashes.contains(&code))
        {
            (OBSTACLES, PROP)
        } else if code >= mmodel::PROP_MODEL {
            ("signs", PROP)
        } else if self.barriers.contains(&(mmodel::zone_of(head)?, code)) {
            (OBSTACLES, BARRIER)
        } else {
            ("people", "person")
        };
        Ok(Interactable {
            size,
            code,
            source,
            move_model: true,
            height: mmodel::height(head)?,
            record: Some((mmodel::zone_of(head)?, mmodel::index_of(head)?)),
            ..Interactable::new(category, kind, cell)
        })
    }

    /// The grid, and each chunk's file where the ROM's chunks are read, as the matrix has them now:
    /// the game rewrites the one block in place for each matrix, so the sizes are read with them.
    fn read_map(&self) -> Result<()> {
        let grid = match self.map.get() {
            None => None,
            Some(map) => fieldmap::grid(&*self.memory, map)?,
        };
        *self.grid.borrow_mut() = grid;
        let (wide, files) = if self.terrain.known() {
            self.read_files()?
        } else {
            (0, Vec::new())
        };
        *self.terrain.files.borrow_mut() = (wide, Rc::new(files));
        Ok(())
    }

    /// The matrix's width in chunks and each chunk's file, row by row, as the field map lists them
    /// now; (0, ()) where there is no field map.
    fn read_files(&self) -> Result<(i64, Vec<i64>)> {
        match self.map.get() {
            None => Ok((0, Vec::new())),
            Some(map) => fieldmap::files(&*self.memory, map),
        }
    }

    fn zone_at(&self, cell: (i64, i64)) -> Result<Option<i64>> {
        let grid = self.grid.borrow();
        let Some(grid) = grid.as_ref() else {
            return Ok(None);
        };
        let zone = grid.at(py::floordiv(cell.0, BLOCK), py::floordiv(cell.1, BLOCK))?;
        Ok(if zone == NO_ZONE { None } else { Some(zone) })
    }

    /// Where the zone's blocks touch another zone's: the open way into it, as each run of cells
    /// along the shared border that can be walked across.
    ///
    /// The border is a run of blocks, and most of it is trees: Littleroot's north side is 40 cells
    /// wide and the way onto Route 101 is 3 of them (the user, 2026-09-22, asking for the walkable
    /// cells rather than the whole border). A cell is a way across when neither side of it is
    /// blocked in the chunks loaded now, or the ROM's where none is, and each run of such cells is
    /// an exit of its own, the same runs whatever is loaded: Route 8's border to Cyllage City split
    /// in two as the chunk north of it unloaded, and P's way to the gap became a way to the new
    /// first run, "no path to edge" (x_route-8.ctm, spike 234). Without the ROM a side no loaded
    /// chunk holds is not known; a border with neither side known is listed whole, as it was before
    /// the chunks were read.
    fn edges(&self, zone: i64, narrow: bool) -> Result<Vec<Interactable>> {
        let Some(grid) = self.grid.borrow().clone() else {
            return Ok(Vec::new());
        };
        let mut borders: IndexMap<i64, Vec<(Cell, Cell)>> = IndexMap::new();
        for row in 0..grid.height {
            for column in 0..grid.width {
                if grid.at(column, row)? != zone {
                    continue;
                }
                for (dc, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let other = grid.at(column + dc, row + dr)?;
                    if other == zone || other == NO_ZONE {
                        continue;
                    }
                    borders
                        .entry(other)
                        .or_default()
                        .extend(border(column, row, dc, dr));
                }
            }
        }
        let mut chunks: Vec<Rc<Chunk>> = Vec::new();
        for chunk in self.chunks_loaded()? {
            chunks.push(self.terrain.flattened(&chunk, None, false)?);
        }
        let wanted = if !borders.is_empty() && chunks.is_empty() {
            Some(zone)
        } else {
            None
        };
        if !narrow {
            // On a rail the chunks mark every cell a wall, and would narrow every edge away.
            chunks = Vec::new();
        } else {
            let held: HashSet<(i64, i64)> = chunks.iter().map(|chunk| chunk.key()).collect();
            // A set of tuples in Python, iterated in its hash order: the chunks it adds cover
            // cells no other chunk does, so the order they are added in changes nothing.
            let mut keys: IndexSet<(i64, i64)> = IndexSet::new();
            for pairs in borders.values() {
                for pair in pairs {
                    for cell in [pair.0, pair.1] {
                        keys.insert((py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK)));
                    }
                }
            }
            for key in keys {
                if held.contains(&key) {
                    continue;
                }
                if let Some(far) = self.terrain.far(key.0, key.1)? {
                    chunks.push(self.terrain.flattened(&far, None, false)?);
                }
            }
        }
        if wanted != self.wanted.get() {
            self.wanted.set(wanted);
            self.asks.set(0);
        }
        let mut out: Vec<Interactable> = Vec::new();
        for (other, pairs) in &borders {
            let mut known = false;
            'any: for pair in pairs {
                for cell in [pair.0, pair.1] {
                    if walkable(&chunks, cell)?.is_some() {
                        known = true;
                        break 'any;
                    }
                }
            }
            let mut crossings: Vec<Cell> = Vec::new();
            if known {
                for &(mine, theirs) in pairs {
                    if walkable(&chunks, mine)? != Some(false)
                        && walkable(&chunks, theirs)? != Some(false)
                    {
                        crossings.push(theirs);
                    }
                }
            } else {
                crossings.extend(pairs.iter().map(|&(_, theirs)| theirs));
            }
            for (gap, (first, size)) in runs(crossings).into_iter().enumerate() {
                out.push(Interactable {
                    size,
                    zone: Some(*other),
                    source: place_source(EDGE_SOURCE, other << 16 | gap as i64)?,
                    ..Interactable::new("exits", "edge", first)
                });
            }
        }
        Ok(out)
    }

    /// The cells of the zone the player uses by facing them, each run of one kind a thing: a PC is
    /// one cell of a Pokémon Center's counter, and a bookshelf two cells wide is one shelf, not
    /// two.
    ///
    /// They are cells of the map, not records: a Center's zone data has its bookshelf as furniture
    /// and no PC at all, and the player's bedroom has no record for its trash can (spikes 78 and
    /// 83). So only the chunks loaded now can say where they are, and outdoors a chunk holds
    /// several zones, of which the matrix says whose each cell is.
    fn use_tiles(&self, zone: i64) -> Result<Vec<Interactable>> {
        let mut found: IndexMap<i64, Vec<Cell>> = IndexMap::new();
        for chunk in self.chunks_loaded()? {
            for (cell, kind) in chunk.kinds() {
                if use_tiles(kind) {
                    let at = self.zone_at(cell)?;
                    if at == Some(zone) || at.is_none() {
                        found.entry(kind).or_default().push(cell);
                    }
                }
            }
        }
        let mut out: Vec<Interactable> = Vec::new();
        for (kind, cells) in found {
            for (first, size) in runs(cells) {
                let called = match self.words.get(&kind) {
                    Some(word) => word.as_str(),
                    None => PROP,
                };
                out.push(Interactable {
                    size,
                    code: kind,
                    source: place_source(TILE_SOURCE, first.0 << 12 | first.1)?,
                    ..Interactable::new("signs", called, first)
                });
            }
        }
        Ok(out)
    }

    /// The zone's slopes, each patch of one kind a thing, each end of its rails, where the
    /// player gets on one (spike 159), each patch of deep water a dive spot (spike 164), and
    /// its currents, those that let the player go round one place a thing (spike 208), and each
    /// patch of the flat ground a ridden Pokémon is got on and off on (spike 238). Cells of the map
    /// with no record behind them, as use tiles are, but read off the whole matrix where the
    /// ROM's chunks are: the game loads the quarter round the player, and Route 119's rails are
    /// 67 cells north of where it starts. Off the chunks loaded without the ROM.
    fn ways(&self, zone: i64, player: Cell) -> Result<Vec<Interactable>> {
        let (mut out, dives, currents, flats) = self.found_ways(zone)?;
        let nearest = |cells: &mut dyn Iterator<Item = Cell>| -> Cell {
            cells
                .min_by_key(|cell| ((cell.0 - player.0).abs() + (cell.1 - player.1).abs(), *cell))
                .expect("a patch has a cell")
        };
        // Route 128's biggest patch is a band running diagonally through a box of 37 by 36, mostly
        // plain sea: a spot is placed at its cell nearest the player, so the way there ends on
        // deep water. A current is too, said with where it lets the player go from there.
        for patch in &dives {
            out.push(Interactable {
                code: DIVE,
                source: place_source(DIVE_SOURCE, cell_number(patch[0]))?,
                home: Some(zone),
                ..Interactable::new("exits", DIVE_SPOT, nearest(&mut patch.iter().copied()))
            });
        }
        for current in &currents {
            let at = nearest(&mut current.keys().copied());
            let first = *current.keys().min().expect("a current has a cell");
            out.push(Interactable {
                source: place_source(CURRENT_SOURCE, cell_number(first))?,
                home: Some(zone),
                arrival: Some(current[&at]),
                ..Interactable::new(CURRENTS_CATEGORY, CURRENT, at)
            });
        }
        for patch in &flats {
            out.push(Interactable {
                source: place_source(RIDE_SOURCE, cell_number(patch[0]))?,
                home: Some(zone),
                ..Interactable::new(RIDING, FLAT_GROUND, nearest(&mut patch.iter().copied()))
            });
        }
        Ok(out)
    }

    /// The bike's things in the zone, its holes, each patch of deep water, its currents and
    /// each patch of flat ground, found once a map.
    fn found_ways(&self, zone: i64) -> Result<FoundWays> {
        let mut loaded: IndexMap<(i64, i64), Rc<Chunk>> = IndexMap::new();
        for chunk in self.chunks_loaded()? {
            loaded.insert(chunk.key(), chunk);
        }
        let files = self.terrain.files.borrow().1.clone();
        let key: BikeKey = (
            zone,
            files,
            if self.terrain.known() {
                Vec::new()
            } else {
                let mut keys: Vec<(i64, i64)> = loaded.keys().copied().collect();
                keys.sort();
                keys
            },
        );
        if self.ways_of.borrow().0.as_ref() != Some(&key) {
            let (out, dives, currents, flats) = self.find_ways(zone, &loaded)?;
            *self.ways_of.borrow_mut() = (Some(key), out, dives, currents, flats);
        }
        let held = self.ways_of.borrow();
        Ok((
            held.1.clone(),
            held.2.clone(),
            held.3.clone(),
            held.4.clone(),
        ))
    }

    /// The bike's things in the zone and its holes, each patch of deep water, each of its
    /// currents, its cells with where each lets the player go, and each patch of the flat
    /// ground a ridden Pokémon is got on and off on, where the zone has ground one crosses
    /// (`_ways`).
    fn find_ways(&self, zone: i64, loaded: &IndexMap<(i64, i64), Rc<Chunk>>) -> Result<FoundWays> {
        let (wide, files) = {
            let held = self.terrain.files.borrow();
            (held.0, held.1.clone())
        };
        let mut chunks = loaded.clone();
        if wide != 0 {
            for at in 0..files.len() as i64 {
                let spot = (py::modulo(at, wide), py::floordiv(at, wide));
                if !chunks.contains_key(&spot)
                    && let Some(far) = self.terrain.far(spot.0, spot.1)?
                {
                    chunks.insert(spot, far);
                }
            }
        }
        let mut flows: IndexMap<Cell, &'static str> = IndexMap::new();
        for chunk in chunks.values() {
            for (cell, kind) in chunk.kinds() {
                if let Some(&(_, way)) = CURRENTS.iter().find(|(known, _)| *known == kind) {
                    flows.insert(cell, way);
                }
            }
        }
        let shut = |cell: Cell| -> Result<bool> {
            match chunks.get(&(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK))) {
                None => Ok(true),
                Some(chunk) => Ok(!chunk.walkable(cell)?),
            }
        };
        let landings = landings(&flows, &shut)?;
        let mut mine: Vec<Cell> = Vec::new();
        for &cell in flows.keys() {
            let at = self.zone_at(cell)?;
            if at == Some(zone) || at.is_none() {
                mine.push(cell);
            }
        }
        let gathered = gathered(
            &mine.iter().map(|cell| landings[cell]).collect(),
            LANDING_REACH,
        );
        let mut which: HashMap<Cell, usize> = HashMap::new();
        for (i, group) in gathered.iter().enumerate() {
            for &landing in group {
                which.insert(landing, i);
            }
        }
        let mut currents: Vec<IndexMap<Cell, Cell>> = vec![IndexMap::new(); gathered.len()];
        for &cell in &mine {
            currents[which[&landings[&cell]]].insert(cell, landings[&cell]);
        }
        let none = HashSet::new();
        let ridden: (&HashSet<i64>, &HashSet<i64>) = match &self.rides {
            None => (&none, &none),
            Some(rides) => (&rides.ground, &rides.stops),
        };
        let mut found: IndexMap<Cell, i64> = IndexMap::new();
        for chunk in chunks.values() {
            for (cell, kind) in chunk.kinds() {
                if (in_slopes(kind)
                    || BIKE_RAILS.contains(&kind)
                    || BUMPY_SLOPES.contains(&kind)
                    || kind == DIVE
                    || kind == BOULDER_HOLE
                    || ridden.0.contains(&kind)
                    || ridden.1.contains(&kind))
                    && {
                        let at = self.zone_at(cell)?;
                        at == Some(zone) || at.is_none()
                    }
                {
                    found.insert(cell, kind);
                }
            }
        }
        let kind_at = |cell: Cell| -> Result<i64> {
            found
                .get(&cell)
                .copied()
                .ok_or_else(|| Error::key(format!("({}, {})", cell.0, cell.1)))
        };
        let mut out: Vec<Interactable> = Vec::new();
        let mut dives: Vec<Vec<Cell>> = Vec::new();
        let mut flats: Vec<Vec<Cell>> = Vec::new();
        let mut spikes = false;
        for patch in patches(&found) {
            if kind_at(patch[0])? == DIVE {
                dives.push(patch);
                continue;
            }
            if ridden.0.contains(&kind_at(patch[0])?) {
                spikes = true;
                continue;
            }
            if ridden.1.contains(&kind_at(patch[0])?) {
                flats.push(patch);
                continue;
            }
            if kind_at(patch[0])? == BOULDER_HOLE {
                let (first, size) = spread(&patch);
                out.push(Interactable {
                    size,
                    code: BOULDER_HOLE,
                    source: place_source(TILE_SOURCE, cell_number(first))?,
                    home: Some(zone),
                    ..Interactable::new(OBSTACLES, HOLE, first)
                });
                continue;
            }
            if BUMPY_SLOPES.contains(&kind_at(patch[0])?) {
                for &cell in &patch {
                    out.push(Interactable {
                        code: kind_at(cell)?,
                        source: place_source(BIKE_SOURCE, cell_number(cell))?,
                        home: Some(zone),
                        ..Interactable::new(BIKE, BUMPY_SLOPE, cell)
                    });
                }
                continue;
            }
            if in_slopes(kind_at(patch[0])?) {
                let (first, size) = spread(&patch);
                // The box's corner is not one of an L-shaped patch's cells: its kind is the patch's.
                out.push(Interactable {
                    size,
                    code: kind_at(patch[0])?,
                    source: place_source(BIKE_SOURCE, cell_number(first))?,
                    home: Some(zone),
                    ..Interactable::new(BIKE, SLOPE, first)
                });
                continue;
            }
            let held: HashSet<Cell> = patch.iter().copied().collect();
            for &cell in &patch {
                if beside(cell)
                    .iter()
                    .filter(|near| held.contains(*near))
                    .count()
                    <= 1
                {
                    out.push(Interactable {
                        code: kind_at(cell)?,
                        source: place_source(BIKE_SOURCE, cell_number(cell))?,
                        home: Some(zone),
                        ..Interactable::new(BIKE, BIKE_RAIL, cell)
                    });
                }
            }
        }
        // Vaniville Town's pen round Mom's Rhyhorn is flat ground too, with nothing to ride over.
        Ok((
            out,
            dives,
            currents,
            if spikes { flats } else { Vec::new() },
        ))
    }

    /// The cell tables of the chunks loaded now.
    pub fn chunks_loaded(&self) -> Result<Vec<Rc<Chunk>>> {
        let mut out: Vec<Rc<Chunk>> = Vec::new();
        let blocks = self.blocks.borrow().clone();
        for block in blocks {
            let Some(held) = mapblock::loaded(&*self.memory, block)? else {
                continue;
            };
            let known = self.chunks.borrow().get(&held).cloned();
            let chunk = match known {
                Some(chunk) => chunk,
                None => {
                    let Some(chunk) = mapblock::read(&*self.memory, &held)? else {
                        continue;
                    };
                    let chunk = Rc::new(chunk);
                    self.chunks.borrow_mut().insert(held, chunk.clone());
                    chunk
                }
            };
            out.push(chunk);
        }
        Ok(out)
    }

    /// The word pair pointing at the two zone data buffers, followed by an LZ11 header.
    ///
    /// The second buffer holds whatever zone was loaded before, and after a battle the same zone
    /// again; while the first is being replaced it can be anything, so only the first has to be a
    /// zone data file.
    fn find_holder(
        &self,
        found: &Found,
        files: &IndexMap<i64, ZoneData>,
    ) -> Result<Option<Holder>> {
        let targets: Vec<i64> = files.keys().copied().collect();
        for holder in found.holders(&targets)? {
            let words = self.memory.read(holder, 12)?;
            if files.contains_key(&u32(&words, 0)?)
                && within(HEAP_SPAN, u32(&words, 4)?)
                && u32(&words, 8)? & 0xFF == LZ11
            {
                return Ok(Some(Holder {
                    address: holder,
                    files: files.clone(),
                }));
            }
        }
        Ok(None)
    }

    /// Look for the zone table through the current zone's info (`ZoneTable.find`): the zone the
    /// info is for is the one the player's model was built in, or the one the matrix puts them in
    /// once they have walked on.
    fn find_table(&self, found: &Found) -> Result<()> {
        let data = self.current()?;
        let player = self.find_player()?;
        let (Some(data), Some(player)) = (data, player) else {
            return Ok(());
        };
        let head = self.memory.read(player, mmodel::HEAD)?;
        let mut zones = PySet::of([mmodel::zone_of(&head)?]);
        let cell = mmodel::cell(&head)?;
        if let Some(cell) = cell
            && let Some(walked) = self.zone_at(cell)?
        {
            zones.add(walked);
        }
        let info = self.memory.read(data.info(), INFO_SIZE)?;
        self.zones.find(found, &info, data.info(), &zones)
    }
}

/// A chunk as the ground is laid from it (`FieldReader.ground`'s `laid`): stacked, a chunk with
/// floors flattened to any floor open with its floors kept; otherwise flattened at `level`.
fn laid(
    terrain: &Terrain,
    chunk: &Rc<Chunk>,
    stacked: bool,
    level: Option<f64>,
) -> Result<Rc<Chunk>> {
    if stacked && !chunk.floors.is_empty() {
        let flat = terrain.flattened(chunk, None, false)?;
        return Ok(Rc::new(Chunk {
            floors: chunk.floors.clone(),
            ..(*flat).clone()
        }));
    }
    terrain.flattened(chunk, level, true)
}

/// Whether the cell and every cell round it are walls in the chunks loaded: where the game keeps a
/// move model it has built and not put on the map. A Secret Base parks the six people of the
/// player's secret team in the walls of its far corner until each is registered, and they were
/// listed as six people there with no way to them (spike 154). Ground only a ridden Pokémon
/// crosses, of a kind of `rides`, is no wall: once its chunk was loaded, Route 9's rocks on the
/// spikes and the Rhyhorn ridden over them were left out (spike 238).
fn walled_in(chunks: &[Rc<Chunk>], cell: Cell, rides: &HashSet<i64>) -> Result<bool> {
    let wall = |near: Cell| -> Result<bool> {
        match chunks.iter().find(|chunk| chunk.holds(near)) {
            None => Ok(false),
            Some(chunk) => Ok(!chunk.walkable(near)? && !rides.contains(&chunk.kind(near)?)),
        }
    };
    let (x, z) = cell;
    for dx in [-1, 0, 1] {
        for dz in [-1, 0, 1] {
            if !wall((x + dx, z + dz))? {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// The cells along the side of block (`column`, `row`) that faces the block (`dc`, `dr`) away, each
/// paired with the neighbouring block's cell against it.
fn border(column: i64, row: i64, dc: i64, dr: i64) -> Vec<(Cell, Cell)> {
    let (x, z) = (column * BLOCK, row * BLOCK);
    let mut out: Vec<(Cell, Cell)> = Vec::new();
    for i in 0..BLOCK {
        if dc != 0 {
            let mine = (x + if dc > 0 { BLOCK - 1 } else { 0 }, z + i);
            out.push((mine, (mine.0 + dc, mine.1)));
        } else {
            let mine = (x + i, z + if dr > 0 { BLOCK - 1 } else { 0 });
            out.push((mine, (mine.0, mine.1 + dr)));
        }
    }
    out
}

/// The steps a ladder makes, each way, with how high it leaves and lands: two one-cell triggers
/// side by side whose heights (+0x14) are a drop apart, its top on a platform and its foot on
/// the ground below, as Fortree City's five are, at (21, 13) over (21, 14) and so on (spike
/// 160), and the seven stairs between the terraces outside Sky Pillar (spike 193).
fn ladders(triggers: &[Vec<u8>]) -> Result<IndexMap<(Cell, Cell), (f64, f64)>> {
    let mut ends: IndexMap<Cell, i64> = IndexMap::new();
    for record in triggers {
        if u16(record, TRIGGER_WIDTH)? == 1 && u16(record, TRIGGER_HEIGHT)? == 1 {
            ends.insert(
                (u16(record, TRIGGER_X)?, u16(record, TRIGGER_Z)?),
                s16(record, TRIGGER_Y)?,
            );
        }
    }
    let mut out: IndexMap<(Cell, Cell), (f64, f64)> = IndexMap::new();
    for (&cell, &high) in &ends {
        for near in beside(cell) {
            if let Some(&low) = ends.get(&near)
                && ((high - low).abs() as f64) > mapblock::DROP
            {
                out.insert((cell, near), (high as f64, low as f64));
            }
        }
    }
    Ok(out)
}

/// Each patch of cells side by side of one kind, the rails' kinds counting as one, its first cell
/// first.
fn patches(found: &IndexMap<Cell, i64>) -> Vec<Vec<Cell>> {
    let mut out: Vec<Vec<Cell>> = Vec::new();
    let mut left: std::collections::BTreeSet<Cell> = found.keys().copied().collect();
    while let Some(&first) = left.iter().next() {
        left.remove(&first);
        let railed = BIKE_RAILS.contains(&found[&first]);
        let mut patch = vec![first];
        let mut todo = vec![first];
        while let Some(cell) = todo.pop() {
            for near in beside(cell) {
                if left.contains(&near)
                    && if railed {
                        BIKE_RAILS.contains(&found[&near])
                    } else {
                        found[&near] == found[&first]
                    }
                {
                    left.remove(&near);
                    patch.push(near);
                    todo.push(near);
                }
            }
        }
        out.push(patch);
    }
    out
}

/// The four cells a step from this one.
fn beside(cell: Cell) -> [Cell; 4] {
    pathfind::STEPS.map(|(_, (dx, dz))| (cell.0 + dx, cell.1 + dz))
}

/// The north-west corner of the box round the cells, and its size.
fn spread(cells: &[Cell]) -> (Cell, (i64, i64)) {
    let xs = cells.iter().map(|cell| cell.0);
    let zs = cells.iter().map(|cell| cell.1);
    let (west, east) = (xs.clone().min().unwrap_or(0), xs.max().unwrap_or(0));
    let (north, south) = (zs.clone().min().unwrap_or(0), zs.max().unwrap_or(0));
    ((west, north), (east - west + 1, south - north + 1))
}

/// The chunk with `cells` it holds open: the blocked bit cleared, the kind kept.
fn opened(chunk: Rc<Chunk>, cells: &HashSet<Cell>) -> Rc<Chunk> {
    let held: Vec<Cell> = cells
        .iter()
        .copied()
        .filter(|&cell| chunk.holds(cell))
        .collect();
    if held.is_empty() {
        return chunk;
    }
    let mut words = chunk.words.clone();
    for (x, z) in held {
        words[((z - chunk.origin.1) * CHUNK + x - chunk.origin.0) as usize] &= !BLOCKED;
    }
    Rc::new(Chunk {
        words,
        ..(*chunk).clone()
    })
}

/// Where a current lets the player go from each of its cells, as `Ground.drift` carries them
/// with nobody in the way: the first cell it carries them to that is no current, or the last
/// before the way is shut. Each cell's is the next's, so each is followed once.
fn landings(
    flows: &IndexMap<Cell, &'static str>,
    shut: &dyn Fn(Cell) -> Result<bool>,
) -> Result<IndexMap<Cell, Cell>> {
    let mut out: IndexMap<Cell, Cell> = IndexMap::new();
    for &start in flows.keys() {
        let mut cell = start;
        let mut carried: Vec<Cell> = Vec::new();
        while let Some(&way) = flows.get(&cell)
            && !out.contains_key(&cell)
        {
            carried.push(cell);
            let (dx, dz) = pathfind::STEPS
                .iter()
                .find(|(known, _)| *known == way)
                .map(|&(_, step)| step)
                .ok_or_else(|| Error::key(py::repr(way)))?;
            let ahead = (cell.0 + dx, cell.1 + dz);
            if shut(ahead)? || carried.contains(&ahead) {
                break;
            }
            cell = ahead;
        }
        let landing = out.get(&cell).copied().unwrap_or(cell);
        for one in carried {
            out.insert(one, landing);
        }
    }
    Ok(out)
}

/// `cells` in groups, each cell within `reach` cells either way of another of its group.
fn gathered(cells: &HashSet<Cell>, reach: i64) -> Vec<Vec<Cell>> {
    let mut left: std::collections::BTreeSet<Cell> = cells.iter().copied().collect();
    let mut out: Vec<Vec<Cell>> = Vec::new();
    while let Some(&first) = left.iter().next() {
        left.remove(&first);
        let mut group = vec![first];
        let mut at = 0;
        while at < group.len() {
            let (x, z) = group[at];
            at += 1;
            let near: Vec<Cell> = left
                .iter()
                .copied()
                .filter(|cell| (cell.0 - x).abs() <= reach && (cell.1 - z).abs() <= reach)
                .collect();
            for cell in near {
                left.remove(&cell);
                group.push(cell);
            }
        }
        out.push(group);
    }
    out
}

/// How high a sign stands among the signs of one spot (`_one_a_spot`): a thing drawn there,
/// then a furniture record, then the cell's own kind.
fn over(item: &Interactable) -> i64 {
    if item.move_model {
        0
    } else if item.place_kind() == Some(FURNITURE_SOURCE) {
        1
    } else {
        2
    }
}

/// `items` less every sign whose cells a sign above it covers (`_over`): the Azurill Doll in
/// Littleroot is a doll drawn, a furniture record in front of it saying "It's an Azurill Doll!",
/// and the cabinet under it, and was listed three times in one cell (Orion, 2026-10-14; spike
/// 211). A sign that reaches past what covers it, a shelf longer than what stands on it, stays.
fn one_a_spot(items: Vec<Interactable>) -> Vec<Interactable> {
    let covered: Vec<(i64, HashSet<Cell>)> = items
        .iter()
        .filter(|item| item.category == "signs")
        .map(|item| (over(item), item.cells().into_iter().collect()))
        .collect();
    let under = |item: &Interactable| -> bool {
        let level = over(item);
        let cells: HashSet<Cell> = item.cells().into_iter().collect();
        covered
            .iter()
            .any(|(above, over)| *above < level && cells.is_subset(over))
    };
    items
        .into_iter()
        .filter(|item| item.category != "signs" || !under(item))
        .collect()
}

/// `items` with those of one `key` that touch side to side as one thing over the box round
/// them, as the first of them listed (`_touching`, `_spread`); a key of None is kept apart. One
/// thing the game builds of several records: Seafloor Cavern's submarine is four furniture
/// records with one script, an elevator's door two side by side, and three stand-ins in a row
/// in front of Groudon (spike 209).
fn joined(
    items: Vec<Interactable>,
    key: &dyn Fn(&Interactable) -> Option<String>,
) -> Vec<Interactable> {
    let mut out: Vec<Interactable> = Vec::new();
    let mut keyed: IndexMap<String, Vec<Interactable>> = IndexMap::new();
    for item in items {
        match key(&item) {
            None => out.push(item),
            Some(which) => keyed.entry(which).or_default().push(item),
        }
    }
    for alike in keyed.values() {
        let cells: HashSet<Cell> = alike.iter().flat_map(|item| item.cells()).collect();
        for group in touching(&cells) {
            let group_cells: HashSet<Cell> = group.iter().copied().collect();
            let first = alike
                .iter()
                .find(|item| item.cells().iter().any(|cell| group_cells.contains(cell)))
                .expect("a group's cells are an item's");
            let (cell, size) = spread(&group);
            out.push(Interactable {
                cell,
                size,
                ..first.clone()
            });
        }
    }
    out
}

/// `cells` in groups of those that touch side to side, the group with the first cell first.
fn touching(cells: &HashSet<Cell>) -> Vec<Vec<Cell>> {
    let mut left: std::collections::BTreeSet<Cell> = cells.iter().copied().collect();
    let mut out: Vec<Vec<Cell>> = Vec::new();
    while let Some(&first) = left.iter().next() {
        let mut group: Vec<Cell> = Vec::new();
        left.remove(&first);
        let mut edge = vec![first];
        while let Some(cell) = edge.pop() {
            group.push(cell);
            let (x, z) = cell;
            for near in [(x - 1, z), (x + 1, z), (x, z - 1), (x, z + 1)] {
                if left.remove(&near) {
                    edge.push(near);
                }
            }
        }
        out.push(group);
    }
    out
}

/// A cell as a place's number: `x << 12 | z`, as a use tile's is.
fn cell_number(cell: Cell) -> i64 {
    cell.0 << 12 | cell.1
}

/// Each run of cells adjacent along a row or a column, as its first cell and its size.
fn runs(cells: impl IntoIterator<Item = Cell>) -> Vec<(Cell, (i64, i64))> {
    let mut left: Vec<Cell> = cells.into_iter().collect();
    left.sort();
    left.dedup();
    let mut out: Vec<(Cell, (i64, i64))> = Vec::new();
    while !left.is_empty() {
        let first = left.remove(0);
        let mut last = first;
        for step in [(1, 0), (0, 1)] {
            while let Some(at) = left
                .iter()
                .position(|&cell| cell == (last.0 + step.0, last.1 + step.1))
            {
                last = left.remove(at);
            }
            if last != first {
                break;
            }
        }
        out.push((first, (last.0 - first.0 + 1, last.1 - first.1 + 1)));
    }
    out
}

/// `in SLOPES`, the kinds of sandy slope.
fn in_slopes(kind: i64) -> bool {
    SLOPES.iter().any(|&(known, _)| known == kind)
}
