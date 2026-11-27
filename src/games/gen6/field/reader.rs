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
//! one, and its reader is ORAS's (`oras.field.hidden`), handed to this one among the game's
//! features, one read at every poll (`features.Watched`).
//!
//! **Map block** — `field::MapBlock`, one of the four 40 by 40 chunks of the matrix the game keeps
//! loaded around the player, read in `mapblock` (spike 82). Its cell table says which cells can be
//! walked, which is how a zone's edge is narrowed to the gap in the trees, and what kind each cell
//! is, which is how a **use tile** is found: a PC, and whatever else the game runs a script for
//! when the player faces the cell and presses A, with no entity record behind it (spike 83).
//!
//! Bound by:
//! - ADR-0042: the gaps into the next part of a place in parts are one exit (`edges`), and where
//!   the game lists the place whole (`Game.whole`), the list holds it, the other parts' things out
//!   of the ROM (`with_other_parts`), of a part the player has stood in only what the game built
//!   there (`built`).

use std::cell::{Cell as Slot, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{s16, u16, u32};
use crate::core::config::{HIDDEN_ALL, HIDDEN_UNDERWATER};
use crate::core::discovery::{Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::core::pathfind::{self, Cell};
use crate::games::common::field::chunk::{CHUNK, Chunk};
use crate::games::common::field::ground::{
    self as common, BLOCK, Far, Grid, Ground, GroundHeight, NO_ZONE, Of, STRENGTH, StepWords, Word,
    pushes, walkable,
};
use crate::games::common::field::ride::riding;
use crate::games::common::field::things::{
    EDGE_SOURCE, FURNITURE_SOURCE, Field, Interactable, OPENING_SOURCE, STAIRS_SOURCE,
    TELEPORT_SOURCE, TELEPORTER, TILE_SOURCE, TRIGGER_SOURCE, WARP_SOURCE, place_source,
};
use crate::games::gen6::anchors::follow;
use crate::games::gen6::field::conveyors::Switches;
use crate::games::gen6::field::features::{Feature, Sighting, Tracked, Watched};
use crate::games::gen6::field::hidden_items::{HiddenItems, Only};
use crate::games::gen6::field::landform::{
    Conveyors, Found as Worked, Here, Holes, Matrix, Rides, TurningFloor, as_form, beside, dropped,
    holes_found, identity, runs, touching,
};
use crate::games::gen6::field::models::Models;
use crate::games::gen6::field::rail::{self, Rails, keep};
use crate::games::gen6::field::records::{
    PROP, Placed, Records, STAIRS, exits, joined, never, off_the_rails, warp_cells,
};
use crate::games::gen6::field::terrain::{FarCells, FarFloors, FarHeights, FarMatrices, Terrain};
use crate::games::gen6::field::voice::{Said, Speaking, Voice};
use crate::games::gen6::field::works::Works;
use crate::games::gen6::field::zonedata::{
    Bytes, FURNITURE, FURNITURE_SCRIPT, LZ11, NPC, NPC_INDEX, NPC_SCRIPT, NPC_SPRITE, NPC_X, NPC_Z,
    TRIGGER_HEIGHT, TRIGGER_SCRIPT, TRIGGER_WIDTH, TRIGGER_X, TRIGGER_Y, TRIGGER_Z, WARP, WARP_OUT,
    ZoneData, read_furniture, read_records, read_triggers, read_zone, zone_files,
};
use crate::games::gen6::field::zonetable::{INFO_SIZE, ZoneTable};
use crate::games::gen6::field::{fieldmap, mapblock, mmodel};
use crate::games::gen6::game::Game;
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::rom::maps::{ChunkSource, StreetSource};
use crate::py::{self, PySet, Result};

/// What a sweep looks for. The slot's own shape is `mmodel`'s, as every structure's is.
///
/// The Python's docstring for this follows `MAP_IN`'s since `WORK` and the rest came between
/// them, where it documents nothing; it is kept here, on the constant it names.
pub const MOVE_MODEL_CLASS: &str = mmodel::CLASS;
/// Where the field's process holds the field's work (`FieldReader.anchor`, spike 282).
pub const WORK: i64 = 0x24;
pub const RECORD_DATA: i64 = 0x08;
pub const TABLE_HOPS: [i64; 2] = [0x04, 0x84];
pub const TABLE_IN: i64 = 0x30;
pub const MAP_HOPS: [i64; 3] = [0x1C, 0x60, 0x88];
/// From the zone's record to its data and to the zone table, and from the field's process to the
/// field map, the same in ORAS and X (spike 282).
pub const MAP_IN: i64 = 0x18;

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

/// Sweeps a zone listed with no chunk asks for before leaving the rest to the schedule.
pub const ASKS: i64 = 4;
pub const TABLE_ASKS: i64 = 10;
/// Sweeps asked for while the zone data is in and the zone table is not, and the seconds between
/// them, before leaving the rest to the schedule. The table's row was in place 1.9 s after the
/// field started, but nothing pointed at its start yet, and it was found only by the whole sweep 30
/// s later: every exit said without its place, and the way to Oldale Town said as stairs
/// (omega_ruby_ledges_3.ctm, 2026-09-28).
pub const TABLE_EVERY: f64 = 2.0;

/// What a street's end on ground walked on the grid is called, as Magenta Plaza is at the end of
/// its alleys: the game's street marks name few of them (spike 251).
pub const OPEN_GROUND: &str = "open ground";
/// Cells round a street's end looked in for open ground: of zone 0x43's 53 ends, the 29 by a
/// plaza or a square have between 5 and all 25 of the cells this far round open, and the rest none
/// (spike 251).
pub const OPENING_REACH: i64 = 2;

/// The two-word holder of the zone data buffers, current first.
struct Holder {
    address: i64,
    files: IndexMap<i64, ZoneData>,
}

/// What the landforms were worked out on (`_found`): the zone, the matrix's chunk files, the
/// chunks loaded where the ROM's are not read, and the field map's grid.
type FoundKey = (i64, Rc<Vec<i64>>, Vec<(i64, i64)>, Option<Grid>);

/// `_twins`: each NPC record of a place in parts an earlier part keeps too -> that part's.
type Twins = HashMap<(i64, i64), (i64, i64)>;

/// Tracks the player's model and the current zone's data, and lists what is on the map.
///
/// Every holder shares the one reader, so it is made in an `Rc` (`FieldReader::new`) and its
/// methods take `&self`; the ground it hands out keeps the reader's terrain alive through `me`.
pub struct FieldReader {
    me: Weak<FieldReader>,
    memory: Rc<dyn Memory>,
    /// What one gen 6 game has of its own, read where it is asked.
    game: Rc<Game>,
    /// The game's words and tables out of the player's ROM, or `NO_TEXT` without one.
    text: Rc<GameText>,
    /// The zone table, which names every place and tells a path from stairs.
    pub zones: Rc<ZoneTable>,
    /// What the game has of its own to list: ORAS's berry spots, ash piles and Secret Base
    /// spots.
    features: Vec<Rc<dyn Feature>>,
    /// Those read at every poll, since they show and go between sweeps (`watch`): ORAS's
    /// DexNav's find. Listed after the rest, and in the player's own part of a place in parts
    /// alone.
    watched: Vec<Rc<dyn Watched>>,
    /// What each landform worked out, by the map it was worked out on (`_found`).
    found_of: RefCell<HashMap<FoundKey, Rc<Vec<Worked>>>>,
    /// Each landform's watch, by the landform (`Landform.watch`): by its identity, as one
    /// holding a mapping does not hash.
    watches: IndexMap<usize, Rc<dyn Tracked>>,
    /// The game's own field voices, its landforms' watches that speak (`voice.Speaking`), in
    /// its landforms' order: ORAS's decorating cursor.
    pub voices: Vec<Voice>,
    /// What the game's ridden Pokémon cross, or None in a game without one: who is in the way
    /// and who is listed asks it too.
    rides: Option<Rides>,
    /// The save's works and flags, which arm triggers and say which hidden items are found.
    works: Rc<Works>,
    /// What each zone's switches have turned (`landform.Conveyors`, `landform.TurningFloor`).
    switches: Option<Switches>,
    /// The hidden items still to be found, where the Extra lists them (spike 249).
    hidden_items: Option<HiddenItems>,
    /// (zone, the zone it leads to) of each trigger listed as an exit into another place.
    trigger_exits: HashSet<(i64, i64)>,
    /// The matrix out of the ROM, which never touches RAM.
    pub terrain: Rc<Terrain>,
    /// Zone -> its triggers' boxes, (x, z, width, height) in the table's order, once read
    /// (`trigger_at`).
    trigger_boxes: RefCell<HashMap<i64, Vec<(i64, i64, i64, i64)>>>,
    /// The player's height in the world at the last poll, which says which floor of a chunk with a
    /// deck they are on (`mapblock.flat`).
    height: Slot<Option<f64>>,
    clock: Rc<dyn Fn() -> f64>,
    /// Lumiose's streets, which X places things on rather than on cells.
    pub rails: Rails,
    twins_of: RefCell<HashMap<Vec<i64>, Rc<Twins>>>,
    /// Each part of a place in parts the player has stood in -> the indices of the NPC records
    /// the game built models for there, when last seen: another part's records are listed only
    /// where it built them (`part_records`).
    built: RefCell<HashMap<i64, HashSet<i64>>>,
    /// What each kind of use tile is: the game's own word where it has one, the PC's out of
    /// the ROM (`archives.PC_LINES`), ahead of the shipped table's.
    words: IndexMap<i64, String>,
    /// What the zone's warps, furniture and triggers are to the list.
    records: Records,
    /// Who stands on the map, the pool of move models a sweep found among them.
    models: Models,
    player: Slot<Option<i64>>,
    /// Whether the player's model was on a rail at the last poll.
    on_rail: Slot<bool>,
    /// Whether that rail was a street under a camera of the map's own (`Rails.fixed`).
    fixed: Slot<bool>,
    /// The quarters clockwise from north up walks the player, as `turn` last found them.
    turn: Slot<i64>,
    /// Whether the last poll found the camera behind a player off the rails (`Rails.behind`).
    behind: Slot<bool>,
    holder: RefCell<Option<Holder>>,
    map: Slot<Option<i64>>,
    /// The field map's grid as last read: a sweep ago, or when the player's model changed.
    grid: RefCell<Option<Grid>>,
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
    /// Bases' spots and its hidden items. `tiles` and `furniture` are what the shipped table says
    /// each kind of use tile is, and each piece of furniture by the shared script it runs
    /// (`Names.words`). `hidden_items` is `[pa3ds.extras] hidden_items`, Python's default
    /// `HIDDEN_OFF`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        memory: Rc<dyn Memory>,
        game: &Game,
        text: &GameText,
        image: &[u8],
        clock: Rc<dyn Fn() -> f64>,
        tiles: Option<&IndexMap<i64, String>>,
        furniture: Option<&IndexMap<i64, String>>,
        hidden_items: &str,
    ) -> Rc<FieldReader> {
        let chunks: Option<Rc<dyn ChunkSource>> = text.map_chunks.clone();
        let streets: Option<rail::Streets> =
            text.streets.clone().map(|streets: Rc<dyn StreetSource>| {
                Rc::new(move |chunks: &[i64]| streets.network(chunks)) as rail::Streets
            });
        let zones = Rc::new(ZoneTable::new(memory.clone(), text.zone_table.clone()));
        let terrain = Rc::new(Terrain::new(
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
            streets.clone(),
        ));
        let mut words: IndexMap<i64, String> = tiles.cloned().unwrap_or_default();
        if let Some(pc) = text.pc.clone()
            && !pc.is_empty()
        {
            words.insert(PC_TILE, pc);
        }
        // One `Game` and one text for the reader, its records and its models, which Python's
        // share by reference.
        let shared = Rc::new(game.clone());
        let words_of = Rc::new(text.clone());
        let records = Records::new(
            shared.clone(),
            words_of.clone(),
            zones.clone(),
            terrain.clone(),
            words.clone(),
            furniture.cloned().unwrap_or_default(),
        );
        let made: Vec<Rc<dyn Feature>> = game
            .features
            .iter()
            .map(|make| make(memory.clone(), image, text))
            .collect();
        let watched: Vec<Rc<dyn Watched>> = made
            .iter()
            .filter_map(|feature| feature.clone().watched())
            .collect();
        let features: Vec<Rc<dyn Feature>> = made
            .into_iter()
            .filter(|feature| feature.clone().watched().is_none())
            .collect();
        let watches: IndexMap<usize, Rc<dyn Tracked>> = game
            .landforms
            .iter()
            .filter_map(|form| {
                form.watch(memory.clone(), image, text)
                    .map(|watch| (identity(&**form), watch))
            })
            .collect();
        let speaking: Vec<Rc<dyn Speaking>> = watches
            .values()
            .filter_map(|watch| watch.clone().speaking())
            .collect();
        let listing = hidden_items == HIDDEN_UNDERWATER || hidden_items == HIDDEN_ALL;
        let underwater = hidden_items == HIDDEN_UNDERWATER;
        let hidden_memory = memory.clone();
        let works = Rc::new(Works::new(memory.clone()));
        let hidden_works = works.clone();
        let hidden_items = move |me: &Weak<FieldReader>| {
            if !listing || image.is_empty() {
                return None;
            }
            // `self.zones.underwater`, through the reader being made.
            let only: Option<Only> = underwater.then(|| {
                let me = me.clone();
                Rc::new(move |zone: i64| {
                    me.upgrade()
                        .expect("a reader is made in an Rc, which its holders keep")
                        .zones
                        .underwater(zone)
                }) as Only
            });
            Some(HiddenItems::new(hidden_memory, hidden_works, image, only))
        };
        let rides = game.landforms.iter().find_map(as_form::<Rides>).cloned();
        Rc::new_cyclic(|me| FieldReader {
            me: me.clone(),
            hidden_items: hidden_items(me),
            // `self._current`, through the reader being made.
            models: Models::new(
                memory.clone(),
                shared.clone(),
                works.clone(),
                rides.clone(),
                {
                    let me = me.clone();
                    Rc::new(move || {
                        me.upgrade()
                            .expect("a reader is made in an Rc, which its holders keep")
                            .current()
                    })
                },
            ),
            game: shared,
            text: words_of,
            works,
            switches: {
                let states: IndexSet<(&'static str, i64)> = game
                    .landforms
                    .iter()
                    .filter_map(|form| {
                        as_form::<Conveyors>(form)
                            .map(|form| form.state)
                            .or_else(|| as_form::<TurningFloor>(form).map(|form| form.state))
                    })
                    .collect();
                if states.is_empty() {
                    None
                } else {
                    Some(Switches::new(memory.clone(), states))
                }
            },
            zones,
            features,
            watched,
            found_of: RefCell::new(HashMap::new()),
            watches,
            // `partial(self._said, watch)`, through the reader being made.
            voices: speaking
                .into_iter()
                .map(|watch| {
                    let me = me.clone();
                    Rc::new(move |r#where: Option<&Field>| {
                        me.upgrade()
                            .expect("a reader is made in an Rc, which its holders keep")
                            .said(&*watch, r#where)
                    }) as Voice
                })
                .collect(),
            rides,
            trigger_exits: game
                .triggers
                .iter()
                .filter_map(|(&(zone, _), known)| known.2.map(|to| (zone, to)))
                .collect(),
            terrain,
            trigger_boxes: RefCell::new(HashMap::new()),
            height: Slot::new(None),
            clock,
            rails: Rails::new(
                memory.clone(),
                streets,
                text.street_records.clone(),
                game.camera,
                text.rail_teleports.clone(),
            ),
            twins_of: RefCell::new(HashMap::new()),
            built: RefCell::new(HashMap::new()),
            words,
            records,
            player: Slot::new(None),
            on_rail: Slot::new(false),
            fixed: Slot::new(false),
            turn: Slot::new(0),
            behind: Slot::new(false),
            holder: RefCell::new(None),
            map: Slot::new(None),
            grid: RefCell::new(None),
            blocks: RefCell::new(Vec::new()),
            chunks: RefCell::new(HashMap::new()),
            wanted: Slot::new(None),
            asked: Slot::new(None),
            asks: Slot::new(0),
            table_asks: Slot::new(0),
            table_next: Slot::new(0.0),
            visit: Slot::new(None),
            memory,
        })
    }

    /// What a sweep looks for for the game's own readers and its landforms' watches: ORAS's
    /// berry spots, ash piles, Secret Base spots and DexNav, then the save's works, then its
    /// rotating doors, thin ice and Secret Bases' decorations, in the order the log names
    /// them.
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
        for feature in &self.watched {
            out.push(&**feature);
        }
        out.push(&*self.works);
        for watch in self.watches.values() {
            out.push(&**watch);
        }
        if let Some(switches) = &self.switches {
            out.push(switches);
        }
        out
    }

    /// Take in what a sweep found: the model pool, the zone data and its holder, the field map and
    /// the zone table.
    pub fn track(&self, found: &Found) -> Result<()> {
        let mut models = found.of(&[MOVE_MODEL_CLASS]);
        if !models.is_empty() {
            models.sort();
            *self.models.pool.borrow_mut() = models;
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
        let offset = self.game.form;
        let form = match offset {
            None => None,
            Some(at) => Some(
                *self
                    .memory
                    .read(player + at, 1)?
                    .first()
                    .ok_or_else(|| py::Error::index("index out of range"))? as i64,
            ),
        };
        let (bike, acro) = (mmodel::on_bike(&head, form)?, mmodel::acro(&head, form)?);
        let settling = mmodel::just_built(&head)?;
        self.on_rail.set(mmodel::on_rail(&head)?);
        // One map's streets for the whole poll, the sweep's as the grid is (#18). Without the ROM
        // the sweep keeps no files, and only the field map's own let the last map's lines go.
        let files: Vec<i64> = if self.terrain.known() || !self.on_rail.get() {
            self.terrain.files.borrow().1.to_vec()
        } else {
            self.read_files()?.1
        };
        // Under a street's own camera the keys walk the world's ways, so its streets are the
        // grid's ground (`Rails.fixed`).
        self.fixed
            .set(self.on_rail.get() && self.rails.fixed(&files, mmodel::rail(&head)?.0)?);
        if !self.on_rail.get() || self.fixed.get() {
            *self.rails.corridor.borrow_mut() = None;
            let behind = self.rails.behind()?;
            self.behind.set(behind.is_some());
            return Ok(Some(Field {
                on_bike: bike,
                acro,
                place: Some(mmodel::place(&head)?),
                surfing: mmodel::surfing(&head, form)?,
                height: self.height.get(),
                settling,
                heading: match behind {
                    None => None,
                    Some(behind) => Some(pathfind::nearest(behind)?),
                },
                bearing: behind,
                turning: self.behind.get(),
                ..Field::new(zone, cell)
            }));
        }
        let on = self.rails.poll(&head, &files)?;
        let Some((at, bearing)) = on else {
            return Ok(Some(Field {
                on_rail: true,
                on_bike: bike,
                height: self.height.get(),
                settling,
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
            settling,
            ..Field::new(zone, at)
        }))
    }

    /// The quarters clockwise from north the pad's up walks the player, as the last poll
    /// found them: where the camera looks (`Rails.turn`), and north on a rail, where the keys
    /// walk the corridor or a street's lanes and the camera's own heading reads 0 (spike 230),
    /// and under a camera behind the player, where the ways are said from `Field.heading`
    /// (spike 239). Not on a rail where the streets are floors of their own: the Tower of
    /// Mastery's fourteen cameras each look their own way over the ramp, up walking the player
    /// east down its first lap's east side and south along its landings, and north was said
    /// for all of them, "8 west, 16 east" for a way west and then north (spike 248). Nor on a
    /// street under a camera of the map's own, where the keys walk the street's way nearest the
    /// one pressed on the screen: round Prism Tower the camera looks east, and right walked the
    /// player south, left north, down west and up east, where "18 north" was said for the way
    /// west and "19 west" for the exit there (spike 251).
    pub fn turn(&self, current: i64) -> Result<i64> {
        if self.behind.get()
            || (self.on_rail.get() && !self.fixed.get() && !self.terrain.raised()?)
        {
            self.turn.set(0);
        } else {
            self.turn.set(self.rails.turn(current)?);
        }
        Ok(self.turn.get())
    }

    /// The quarter of the world, clockwise from north, up walks the player, as the last poll
    /// and `turn` found it: on Lumiose's rails and under a camera behind the player, where the
    /// compass is not turned, the camera's own heading (`Rails.facing`); elsewhere the compass's
    /// turn. `current` is the quarter last taken, kept until the camera is past its edge.
    pub fn facing(&self, current: Option<i64>) -> Result<Option<i64>> {
        if self.behind.get()
            || (self.on_rail.get() && !self.fixed.get() && !self.terrain.raised()?)
        {
            return self.rails.facing(current);
        }
        Ok(Some(self.turn.get()))
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
    /// floor at a time, each step onto the floor beside it it joins (`Ground.climb`). Where the
    /// player's height is not read, a chunk with a deck is walked on the first floor open in
    /// each cell (`mapblock.flat`, spike 156).
    ///
    /// The invisible blockers stand in the way once the landforms are laid, but round rotating
    /// doors, whose arms they stand in for (`models.Role.BLOCKER`).
    pub fn ground(&self, r#where: &Field) -> Result<Ground> {
        if r#where.on_rail {
            let corridor = self
                .rails
                .corridor
                .borrow()
                .clone()
                .map(|corridor| corridor as Rc<dyn common::Corridor>);
            return Ground::of(
                r#where,
                Of {
                    corridor,
                    ..Of::default()
                },
            );
        }
        let files = self.terrain.files.borrow().1.clone();
        let lanes = self
            .rails
            .lanes(&files, self.terrain.raised()?, self.turn.get())?;
        // Where the streets are floors of their own, a cell they open is open on its own floor.
        let covered: HashSet<Cell> = if self.terrain.raised()? {
            HashSet::new()
        } else {
            lanes.keys().copied().collect()
        };
        let (nothing, blockers) = self.models.unseen()?;
        let loaded = || self.chunks_loaded();
        let level = self.height.get();
        let stacked = level.is_some() && self.terrain.stacked(&loaded)?;

        let mut chunks: Vec<Rc<Chunk>> = Vec::new();
        for chunk in self.chunks_loaded()? {
            chunks.push(laid(&self.terrain, &chunk, stacked, level)?.opened(&covered));
        }
        let ladders = self.ladders_here()?;
        let cliffs = self.terrain.cliffs(&ladders)?;
        let trusted = self.zone_at(r#where.cell)?.is_some();
        let me = self
            .me
            .upgrade()
            .expect("a reader is made in an Rc, which its holders keep");
        let far_covered = covered.clone();
        // A walk asks for the chunk at every cell it steps on, and opening one scans every cell
        // of X's streets: most of the half a second Ctrl and P's walk took on Route 7 (spike 233).
        // `functools.cache`: an answer kept once made, a raise not.
        let made: RefCell<HashMap<(i64, i64), Option<Rc<Chunk>>>> = RefCell::new(HashMap::new());
        let far: Far = Rc::new(move |column, row| {
            if let Some(kept) = made.borrow().get(&(column, row)) {
                return Ok(kept.clone());
            }
            let chunk = match me.terrain.far(column, row)? {
                None => None,
                Some(chunk) => {
                    Some(laid(&me.terrain, &chunk, stacked, level)?.opened(&far_covered))
                }
            };
            made.borrow_mut().insert((column, row), chunk.clone());
            Ok(chunk)
        });
        let grid = if trusted {
            self.grid.borrow().clone()
        } else {
            None
        };
        let crossed: HashSet<Cell> = self
            .trigger_cells(r#where.zone, self.game.crossed.is_empty(), &|key| {
                self.game.crossed.contains(&key)
            })?
            .into_keys()
            .collect();
        let ground_height: Option<GroundHeight> = if stacked {
            None
        } else {
            let me = self
                .me
                .upgrade()
                .expect("a reader is made in an Rc, which its holders keep");
            Some(Rc::new(move |cell| me.terrain.height(cell)))
        };
        let mut ground = Ground {
            cliffs,
            ladders,
            mats: r#where
                .interactables
                .iter()
                .filter(|item| item.place_kind() == Some(WARP_SOURCE) && item.code == WARP_OUT)
                .flat_map(|item| item.cells())
                .collect(),
            level,
            ground_height,
            lanes,
            raised: self.terrain.raised()?,
            words: self
                .game
                .landforms
                .iter()
                .map(|form| {
                    let form = form.clone();
                    Rc::new(
                        move |ground: &Ground, cell, step, floor, words: &StepWords| {
                            form.word(ground, cell, step, floor, words)
                        },
                    ) as Word
                })
                .collect(),
            ..Ground::of(
                r#where,
                Of {
                    chunks,
                    grid,
                    crossed,
                    far: if trusted && self.terrain.known() {
                        Some(far)
                    } else {
                        None
                    },
                    clears: self.game.clears.clone(),
                    through: self.game.lurkers.clone(),
                    solid: nothing,
                    stacked,
                    ..Of::default()
                },
            )?
        };
        let here = self.here(r#where, None);
        for (form, found) in self
            .game
            .landforms
            .iter()
            .zip(self.found(r#where.zone)?.iter())
        {
            ground = form.laid(found, ground, &here)?;
        }
        Ok(if ground.doors.is_empty() {
            ground.stood(&blockers, r#where.cell)
        } else {
            ground
        })
    }

    /// The zone's ladders (`ladders`), none without its zone data.
    fn ladders_here(&self) -> Result<IndexMap<(Cell, Cell), (f64, f64)>> {
        match self.current()? {
            None => Ok(IndexMap::new()),
            Some(data) => ladders(&read_triggers(&*self.memory, &data)?),
        }
    }

    /// What a landform's watch that speaks says now, given what the landforms are laid on where
    /// the player is: nothing while nothing says where.
    fn said(&self, watch: &dyn Speaking, r#where: Option<&Field>) -> Result<Option<Said>> {
        match r#where {
            None => Ok(None),
            Some(r#where) => watch.said(&self.here(r#where, None)),
        }
    }

    /// The cells of the zone's triggers whose (zone, script) `known` holds, each with it.
    /// `none` is whether `known` is empty.
    fn trigger_cells(
        &self,
        zone: i64,
        none: bool,
        known: &dyn Fn((i64, i64)) -> bool,
    ) -> Result<IndexMap<Cell, (i64, i64)>> {
        let data = self.current()?;
        let Some(data) = data.filter(|_| !none) else {
            return Ok(IndexMap::new());
        };
        let Some(home) = self.home(&data, &[zone])? else {
            return Ok(IndexMap::new());
        };
        let mut out: IndexMap<Cell, (i64, i64)> = IndexMap::new();
        for record in read_triggers(&*self.memory, &data)? {
            let key = (home, u16(&record, TRIGGER_SCRIPT)?);
            if !known(key) {
                continue;
            }
            let (x, z) = (u16(&record, TRIGGER_X)?, u16(&record, TRIGGER_Z)?);
            for dx in 0..u16(&record, TRIGGER_WIDTH)? {
                for dz in 0..u16(&record, TRIGGER_HEIGHT)? {
                    out.insert((x + dx, z + dz), key);
                }
            }
        }
        Ok(out)
    }

    /// The zone's triggers listed (`Records.triggers`), read only in a game that lists any.
    fn listed_triggers(&self, data: &ZoneData, zone: i64) -> Result<Vec<Interactable>> {
        let game = &self.game;
        let home = if game.triggers.is_empty()
            && game.hidden_triggers.is_empty()
            && self.text.teleports.is_empty()
        {
            None
        } else {
            self.home(data, &[zone])?
        };
        let Some(home) = home else {
            return Ok(Vec::new());
        };
        self.records.triggers(
            &read_triggers(&*self.memory, data)?,
            home,
            &self.works,
            self.hidden_items.is_some(),
            &|place| self.laid(place),
        )
    }

    /// Where a trigger placed on a street is on the grid, where the street has it, and as
    /// high as its line's start; None where the rails lay it nowhere.
    fn laid(&self, place: (i64, i64, i64, i64, i64)) -> Result<Option<Placed>> {
        let files = self.terrain.files.borrow().1.clone();
        let shape = self.rails.laid(place, &files)?;
        let Some(shape) = shape.filter(|shape| !shape.is_empty()) else {
            return Ok(None);
        };
        Ok(Some((
            rail::spread(shape)?,
            self.rails.line_height(place.0, &files)?,
        )))
    }

    /// Whether the save's works shut the zone's warp in `slot` (`Game.armed_warps`). Both of a
    /// lift's warps stay listed until a sweep has found the works.
    fn shut(&self, home: Option<i64>, slot: i64) -> Result<bool> {
        let armed = home.and_then(|home| self.game.armed_warps.get(&(home, slot)));
        Ok(match armed {
            Some(&(work, value)) => {
                self.works.event.get().is_some() && !self.works.holds(work, value)?
            }
            None => false,
        })
    }

    /// `where` with everything worth walking to in its zone listed.
    pub fn interactables(&self, r#where: &Field) -> Result<Field> {
        let corridor = self.rails.corridor.borrow().clone();
        if let Some(corridor) = corridor.filter(|_| r#where.on_rail) {
            let mut listed = self.on_the_rails(&corridor, r#where.zone)?;
            if self.rails.folded.get() {
                listed = self.in_parts(&corridor, r#where.cell, listed)?;
            }
            return Ok(Field {
                interactables: listed,
                ..r#where.clone()
            });
        }
        let mut items: Vec<Interactable> = Vec::new();
        let data = self.current()?;
        let mut people = self.people(r#where.zone)?;
        let loaded = self.chunks_loaded()?;
        let chunk_at = |cell: Cell| -> Result<Option<Rc<Chunk>>> {
            // The ROM's chunk where none is loaded, as the ground has it: a battle unloads them all.
            match loaded.iter().find(|chunk| chunk.holds(cell)) {
                Some(chunk) => Ok(Some(chunk.clone())),
                None => self
                    .terrain
                    .far(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK)),
            }
        };
        let kind = |cell: Cell| -> Result<Option<i64>> {
            match chunk_at(cell)? {
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
            let warps = read_records(&*self.memory, data, WARP)?;
            // Furniture standing on a warp is its lock, and the door is listed in its place:
            // Sea Mauville's five, which say "The door is locked." until their key opens them and
            // were listed beside their doors after (or_sea_mauville_locked_doors), and in X the
            // hidden door "Open sesame" opens and the Kalos Power Plant's: none other in either.
            let locks = warp_cells(&warps)?;
            let mut furniture: Vec<Interactable> = Vec::new();
            // Each of Prism Tower's quiz rooms keeps a record in the walls of its far corner, with
            // nothing beside it to stand on, and listed it as a sign (spike 252).
            for item in self.records.furniture(&*self.memory, data, home)? {
                if item.cells().iter().all(|cell| locks.contains(cell))
                    || buried(&chunk_at, &item.cells())?
                {
                    continue;
                }
                furniture.push(item);
            }
            items.extend(joined(furniture, &|item| Some(self.records.key(item))));
            let files = self.terrain.files.borrow().1.clone();
            // A warp walled in is where only a script puts the player: the Radiant Chamber's
            // door, whose lift is listed in its place (`Game.triggers`). Not every walled-in warp:
            // arriving in Sootopolis City, the last place's chunks walled in every door for a
            // poll (as_cave_of_origin).
            items.extend(self.records.warps(
                &warps,
                home,
                &|zone| self.zones.place_index(zone),
                Some(&|one, other| self.played(one, other)),
                &|record| self.rails.on_grid(record, &files),
                &|slot| self.shut(home, slot),
                &|warp| {
                    Ok(home
                        .zip(warp.zone)
                        .is_some_and(|exit| self.trigger_exits.contains(&exit))
                        && buried(&chunk_at, &warp.cells())?)
                },
            )?);
        }
        items.extend(self.boulders(people.clone(), r#where.zone, data.as_ref(), home)?);
        items.extend(self.edges(r#where.zone, !r#where.on_rail, true)?);
        items.extend(self.use_tiles(r#where.zone)?);
        let here = self.here(r#where, Some(people.clone()));
        for (form, found) in self
            .game
            .landforms
            .iter()
            .zip(self.found(r#where.zone)?.iter())
        {
            items.extend(form.listed(found, &here)?);
        }
        items.extend(self.stairs_in(r#where.zone)?);
        if let Some(data) = &data {
            items.extend(self.listed_triggers(data, r#where.zone)?);
        }
        let zone_at = |cell: Cell| self.zone_at(cell);
        for feature in &self.features {
            items.extend(feature.listed(r#where.zone, &zone_at)?);
        }
        for feature in &self.watched {
            items.extend(feature.listed(r#where.zone, &zone_at)?);
        }
        if let Some(hidden_items) = &self.hidden_items {
            items.extend(hidden_items.listed(r#where.zone, &zone_at)?);
        }
        if self.game.whole.contains(&r#where.zone)
            && let Some(parts) = self
                .text
                .parts
                .get(&r#where.zone)
                .filter(|parts| !parts.is_empty())
        {
            items = self.with_other_parts(r#where, parts, items, &people)?;
        }
        // A boulder in a hole is floor now, and neither it nor the hole is anything to find.
        let holes = self.holes(r#where.zone)?;
        if !holes.is_empty() {
            let gone = dropped(&holes, &items, &self.game.clears);
            if !gone.is_empty() {
                items.retain(|item| !item.cells().iter().all(|cell| gone.contains(cell)));
            }
        }
        if self.game.rooms.contains(&r#where.zone) {
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
            .any(|item| self.game.clears.get(&item.code) == Some(&STRENGTH))
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
        let filled = dropped(&holes, &people, &self.game.clears);
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
            if self.game.clears.get(&item.code) == Some(&STRENGTH)
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

    /// The cells of the zone's holes a big boulder drops into (`landform.Holes`).
    fn holes(&self, zone: i64) -> Result<HashSet<Cell>> {
        let mut out: HashSet<Cell> = HashSet::new();
        for (form, found) in self.game.landforms.iter().zip(self.found(zone)?.iter()) {
            if as_form::<Holes>(form).is_some() {
                out.extend(holes_found(found));
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
        if !self.game.stairs.contains(&zone) {
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

    /// What the furniture record in a cell of `zone` is called (`Records.kind`), None where none
    /// stands.
    fn furniture_at(&self, zone: i64, cell: Cell) -> Result<Option<String>> {
        let data = self.current()?;
        let home = match &data {
            None => None,
            Some(data) => self.home(data, &[zone])?,
        };
        let items = match &data {
            None => Vec::new(),
            Some(data) => read_furniture(&*self.memory, data)?,
        };
        for item in items {
            if item.cells().contains(&cell) {
                return Ok(Some(self.records.kind(home, &item)));
            }
        }
        Ok(None)
    }

    /// Take what the field's process leads to and no class names, each checked by its shape
    /// (spike 282), where none is known that holds: the current zone's data, held where the
    /// zone's record keeps it; the zone table, inside an object the record leads to; and the
    /// field map, inside one the game's data leads to. Returns those taken, for the log: the
    /// moment the field can be listed, a second before the sweep 1.5 s after the field starts
    /// would find them.
    ///
    /// ```text
    /// process  +0x24 -> the field's work  +0xF4, X's +0xE4 -> the zone's record
    /// record   +0x08 -> the zone's data     +0x04 +0x84 -> the zone table, 0x30 in
    /// process  +0x1C -> the manager  +0x60 -> the game's data  +0x88 -> the field map,
    ///          0x18 in
    /// ```
    pub fn anchor(&self, process: i64) -> Result<Vec<String>> {
        let mut taken: Vec<String> = Vec::new();
        let record = follow(&*self.memory, process, &[WORK, self.game.zone_record])?;
        if let Some(record) = record {
            let holder = record + RECORD_DATA;
            if self.current()?.is_none()
                && read_zone(&*self.memory, u32(&self.memory.read(holder, 4)?, 0)?)?.is_some()
            {
                *self.holder.borrow_mut() = Some(Holder {
                    address: holder,
                    files: IndexMap::new(),
                });
                taken.push("zone data".to_string());
            }
            let table = follow(&*self.memory, record, &TABLE_HOPS)?;
            if let Some(table) = table
                && self.zones.take(table + TABLE_IN)?
            {
                taken.push("zone table".to_string());
            }
        }
        let game = follow(&*self.memory, process, &MAP_HOPS)?;
        let stale = match self.map.get() {
            None => true,
            Some(map) => !fieldmap::valid(&*self.memory, map)?,
        };
        if stale
            && let Some(game) = game
            && fieldmap::valid(&*self.memory, game + MAP_IN)?
        {
            self.map.set(Some(game + MAP_IN));
            self.read_map()?;
            taken.push("field map".to_string());
        }
        Ok(taken)
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
        let zone = self.visit.get().map(|visit| visit.1);
        for reader in self.tracked() {
            if let Some(reader) = reader.sweeping()
                && let Some(why) = reader.sweep_wanted(zone, &*self.clock)?
            {
                return Ok(Some(why));
            }
        }
        Ok(None)
    }

    /// What each watched feature shows in `zone` now (`features.Watched`): read at every
    /// poll, since it shows and goes between sweeps.
    pub fn watch(&self, zone: i64) -> Result<Vec<Sighting>> {
        let mut seen = Vec::new();
        for feature in &self.watched {
            seen.push(feature.watch(zone, &|cell| self.zone_at(cell))?);
        }
        Ok(seen)
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
        let mut warped: Vec<Interactable> = Vec::new();
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
                    let warp = self.records.warp(i as i64, record, Some(at), owner)?;
                    warped.push(Interactable {
                        home: owner,
                        ..warp
                    });
                }
            }
        }
        out.extend(exits(warped, &|zone| self.zones.place_index(zone), None)?);
        // Off North Boulevard into Magenta Plaza's alley only the way back was listed: the alley
        // ends on the plaza, which is walked on the grid and is on no line (spike 251).
        for (point, cell, ground) in self.rails.ends(corridor)? {
            if self.by_open_ground(ground)? {
                out.push(Interactable {
                    home,
                    source: place_source(OPENING_SOURCE, point)?,
                    ..Interactable::new("exits", OPEN_GROUND, cell)
                });
            }
        }
        for other in self.models.others() {
            let (model, head) = other?;
            if !mmodel::on_rail(&head)? {
                continue;
            }
            let (line, a, x) = mmodel::rail(&head)?;
            if let Some(cell) = corridor.at(line, a, x)? {
                out.push(self.models.item(model, &head, cell, (1, 1), None)?);
            }
        }
        Ok(out)
    }

    /// A folded map's listing as the player at `at` meets it (`rail.lay_keyed`, spike 264): with
    /// the teleporters of the Gym's module between its parts, each where it sends the player from
    /// with where it puts them (`rail.Teleport`); its furniture one thing a script where records
    /// touch, and none in another part where its script has a record in the player's, as each of
    /// Anistar City Gym's statues is three records, one on the lobby's street and two on the
    /// landing's tracks, listed as three signs; and what stands in another part `apart`.
    fn in_parts(
        &self,
        corridor: &rail::Corridor,
        at: Cell,
        listed: Vec<Interactable>,
    ) -> Result<Vec<Interactable>> {
        let here = corridor.part(at);
        let mut listed = listed;
        listed.extend(rail_teleporters(
            &self.rails.teleports,
            &|place| self.rails.place_height(place),
            corridor,
        )?);
        let furnished =
            |item: &Interactable| !item.move_model && item.place_kind() == Some(FURNITURE_SOURCE);
        let furniture: Vec<Interactable> = listed
            .iter()
            .filter(|item| furnished(item))
            .cloned()
            .collect();
        // A set only tested.
        let near: HashSet<i64> = furniture
            .iter()
            .filter(|item| corridor.part(item.cell) == here)
            .map(|item| item.code)
            .collect();
        let kept: Vec<Interactable> = furniture
            .into_iter()
            .filter(|item| corridor.part(item.cell) == here || !near.contains(&item.code))
            .collect();
        // `item not in furniture` by equality: what is not furniture by the same test.
        let mut listed: Vec<Interactable> =
            listed.into_iter().filter(|item| !furnished(item)).collect();
        listed.extend(joined(kept, &|item| Some(self.records.key(item))));
        Ok(listed
            .into_iter()
            .map(|item| Interactable {
                apart: corridor.part(item.cell) != here,
                lands_apart: item
                    .arrival
                    .is_some_and(|arrival| corridor.part(arrival) != corridor.part(item.cell)),
                ..item
            })
            .collect())
    }

    /// Whether a cell the grid can be walked in lies within `OPENING_REACH` of `cell`, in the
    /// chunks loaded or, past them, the ROM's: the chunks under a street mark every cell of it a
    /// wall (spike 102), so open ground beside a street's end is ground off it.
    fn by_open_ground(&self, cell: Cell) -> Result<bool> {
        let loaded = self.chunks_loaded()?;
        for x in cell.0 - OPENING_REACH..cell.0 + OPENING_REACH + 1 {
            for z in cell.1 - OPENING_REACH..cell.1 + OPENING_REACH + 1 {
                let chunk = match loaded.iter().find(|chunk| chunk.holds((x, z))) {
                    Some(chunk) => Some(chunk.clone()),
                    None => self
                        .terrain
                        .far(py::floordiv(x, CHUNK), py::floordiv(z, CHUNK))?,
                };
                if let Some(chunk) = chunk
                    && chunk.holds((x, z))
                    && chunk.walkable((x, z))?
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn find_player(&self) -> Result<Option<i64>> {
        if let Some(player) = self.player.get()
            && self.is_player(player)?
        {
            return Ok(Some(player));
        }
        let mut found = None;
        let models = self.models.pool.borrow().clone();
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

    /// Every model listed in `zone` (`Models.people`), walled in by the chunks loaded with
    /// the streets that are floors of their own: Coumarine Gym's chunks wall every cell but its
    /// entrance's, and three of its four trainers stand on platforms over them.
    fn people(&self, zone: i64) -> Result<Vec<Interactable>> {
        let mut chunks: Vec<Rc<Chunk>> = Vec::new();
        for chunk in self.chunks_loaded()? {
            let chunk = self.terrain.with_streets(&chunk)?;
            chunks.push(self.terrain.flattened(&chunk, None, false)?);
        }
        self.models.people(zone, &chunks, &self.holes(zone)?)
    }

    /// Which of two twin zones the save's story plays (`Game.version_flags`): the higher
    /// where the first flag is set, the lower where the second is; None in a game without
    /// twins, in a save with neither, and before a sweep has found the works. Read only where
    /// two warps on one cell lead to two places, so that no other listing reads the flags.
    fn played(&self, one: i64, other: i64) -> Result<Option<i64>> {
        let Some((higher, lower)) = self.game.version_flags else {
            return Ok(None);
        };
        if self.works.flag(higher)? == Some(true) {
            return Ok(Some(one.max(other)));
        }
        Ok(if self.works.flag(lower)? == Some(true) {
            Some(one.min(other))
        } else {
            None
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

    /// The zone the field map puts a cell in, or None where it puts none or is not read.
    ///
    /// Python's `_zone_at` and the `zone_at` that returns it, one here, their names being one.
    pub fn zone_at(&self, cell: (i64, i64)) -> Result<Option<i64>> {
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
    /// wide and the way onto Route 101 is 3 of them (the walkable cells were asked for rather than
    /// the whole border). A cell is a way across when neither side of it is
    /// blocked in the chunks loaded now, or the ROM's where none is, and each run of such cells is
    /// an exit of its own, the same runs whatever is loaded: Route 8's border to Cyllage City split
    /// in two as the chunk north of it unloaded, and P's way to the gap became a way to the new
    /// first run, "no path to edge" (x_route-8.ctm, spike 234). Without the ROM a side no loaded
    /// chunk holds is not known; a border with neither side known is listed whole, as it was before
    /// the chunks were read. Asked of another part of the player's place, `track` off, it asks for
    /// no sweep (`sweep_wanted`).
    fn edges(&self, zone: i64, narrow: bool, track: bool) -> Result<Vec<Interactable>> {
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
        if track && wanted != self.wanted.get() {
            self.wanted.set(wanted);
            self.asks.set(0);
        }
        let mut out: Vec<Interactable> = Vec::new();
        let place = self.zones.place_of(zone)?;
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
            if !crossings.is_empty() && place.is_some() && self.zones.place_of(*other)? == place {
                // The next part of a place in parts is one way in, at whichever gap is nearest:
                // each quarter of the Safari Zone listed six or seven gaps into the next
                // (ADR-0042).
                let mut shape: Vec<Cell> = crossings
                    .iter()
                    .copied()
                    .collect::<IndexSet<Cell>>()
                    .into_iter()
                    .collect();
                shape.sort_by_key(|cell| (cell.1, cell.0));
                out.push(Interactable {
                    zone: Some(*other),
                    shape: shape.clone(),
                    source: place_source(EDGE_SOURCE, other << 16)?,
                    ..Interactable::new("exits", "edge", shape[0])
                });
                continue;
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

    /// Every zone of the place in parts `zone` is a part of, itself among them, or none for a
    /// zone that is no part (`GameText.parts`, ADR-0042).
    pub fn parts(&self, zone: i64) -> Vec<i64> {
        self.text.parts.get(&zone).cloned().unwrap_or_default()
    }

    /// `items`, the player's own part's, with the rest of their place in parts (ADR-0042).
    ///
    /// The game builds only the zone the player is in, and the list held only that: two
    /// quarters of the Safari Zone away the Mega Stone was nowhere, and the guide said it gone
    /// once the player left the way in the next quarter (or_safari_zone_multizone_pathfinding).
    /// Each other part's furniture, warps and NPC records out of the ROM (`part_records`), its
    /// landforms, its features but those watched, and its edges into other places; and each part
    /// one way in, with every gap into it from any part of the place, at the nearest of them. A
    /// record two parts keep is one thing, told by the first part's (`twins`).
    ///
    /// Python's `people` is no longer read here.
    fn with_other_parts(
        &self,
        r#where: &Field,
        parts: &[i64],
        items: Vec<Interactable>,
        _people: &[Interactable],
    ) -> Result<Vec<Interactable>> {
        let built: HashSet<i64> = items
            .iter()
            .filter(|item| item.move_model)
            .filter_map(|item| item.record)
            .filter(|record| record.0 == r#where.zone)
            .map(|record| record.1)
            .collect();
        self.built.borrow_mut().insert(r#where.zone, built);
        let twins = self.twins(parts)?;
        let items: Vec<Interactable> = items
            .into_iter()
            .map(
                |item| match item.record.and_then(|record| twins.get(&record)) {
                    Some(&one) if item.move_model => Interactable {
                        record: Some(one),
                        ..item
                    },
                    _ => item,
                },
            )
            .collect();
        let mut live: HashSet<(i64, i64)> = items
            .iter()
            .filter(|item| item.move_model)
            .filter_map(|item| item.record)
            .collect();
        let mut gaps: IndexMap<i64, HashSet<Cell>> = IndexMap::new();
        let mut out: Vec<Interactable> = items
            .into_iter()
            .filter(|item| {
                !(item.kind == "edge" && item.zone.is_some_and(|zone| parts.contains(&zone)))
            })
            .collect();
        for &part in parts {
            for edge in self.edges(part, !r#where.on_rail, false)? {
                if let Some(zone) = edge.zone.filter(|zone| parts.contains(zone)) {
                    if zone != r#where.zone {
                        gaps.entry(zone).or_default().extend(edge.cells());
                    }
                } else if part != r#where.zone {
                    out.push(edge);
                }
            }
            if part == r#where.zone {
                continue;
            }
            out.extend(self.part_records(part, &mut live, &twins)?);
            let there = Field {
                zone: part,
                ..r#where.clone()
            };
            let here = self.here(&there, Some(Vec::new()));
            for (form, found) in self.game.landforms.iter().zip(self.found(part)?.iter()) {
                out.extend(form.listed(found, &here)?);
            }
            let zone_at = |cell: Cell| self.zone_at(cell);
            for feature in &self.features {
                out.extend(feature.listed(part, &zone_at)?);
            }
            if let Some(hidden_items) = &self.hidden_items {
                out.extend(hidden_items.listed(part, &zone_at)?);
            }
        }
        let mut gaps: Vec<(i64, HashSet<Cell>)> = gaps.into_iter().collect();
        gaps.sort_by_key(|(part, _)| *part);
        for (part, cells) in gaps {
            let mut shape: Vec<Cell> = cells.into_iter().collect();
            shape.sort_by_key(|cell| (cell.1, cell.0));
            out.push(Interactable {
                zone: Some(part),
                shape: shape.clone(),
                source: place_source(EDGE_SOURCE, part << 16)?,
                ..Interactable::new("exits", "edge", shape[0])
            });
        }
        Ok(out)
    }

    /// Another part's furniture, warps and NPC records out of its zone data in the ROM, each
    /// as its zone's own are listed: an NPC record whose model is built already, or listed
    /// already, `live`, which it adds to, is listed as that, and one another part keeps too as
    /// that part's (`twins`). Of a part the player has stood in, only the records the game built
    /// there when they were last in it (`built`): its zone's script keeps some unbuilt, by
    /// works no record says. In Southern Route 119 the list held a Kecleon out of the northern
    /// strip's records, whom the game had not built there, and P said "Kecleon in the way"; its
    /// record's +0x0a and six others' unbuilt, 2 to 6 and 2000, are no flags the save sets, and
    /// the same numbers in the southern strip's are built (or_no_path).
    fn part_records(
        &self,
        zone: i64,
        live: &mut HashSet<(i64, i64)>,
        twins: &Twins,
    ) -> Result<Vec<Interactable>> {
        let built = self.built.borrow().get(&zone).cloned();
        let Some(data) = self.text.part_data.get(&zone) else {
            return Ok(Vec::new());
        };
        let memory = Bytes(data);
        let Some(found) = read_zone(&memory, 0)? else {
            return Ok(Vec::new());
        };
        let mut out = self.records.furniture(&memory, &found, Some(zone))?;
        out.extend(self.records.warps(
            &read_records(&memory, &found, WARP)?,
            Some(zone),
            &|zone| self.zones.place_of(zone),
            None,
            &off_the_rails,
            &|_| Ok(false),
            &never,
        )?);
        for record in read_records(&memory, &found, NPC)? {
            let here = (zone, u16(&record, NPC_INDEX)?);
            if built.as_ref().is_some_and(|built| !built.contains(&here.1)) {
                continue;
            }
            let one = twins.get(&here).copied().unwrap_or(here);
            if !live.contains(&one)
                && let Some(item) = self.models.record(zone, &record)?
            {
                live.insert(one);
                out.push(Interactable {
                    record: Some(one),
                    ..item
                });
            }
        }
        Ok(out)
    }

    /// Each NPC record of a place in parts that an earlier part keeps too, the same sprite and
    /// script on the same cell, -> that part's: one thing. The Safari Zone's two southern
    /// quarters both keep the item ball at (244, 217), its record 3 in each and the one flag,
    /// so that whichever is loaded shows it, and it was listed twice.
    fn twins(&self, parts: &[i64]) -> Result<Rc<Twins>> {
        if let Some(known) = self.twins_of.borrow().get(parts) {
            return Ok(known.clone());
        }
        let mut first: HashMap<(i64, i64, i64, i64), (i64, i64)> = HashMap::new();
        let mut out: Twins = HashMap::new();
        for &zone in parts {
            let Some(data) = self.text.part_data.get(&zone) else {
                continue;
            };
            let memory = Bytes(data);
            let Some(found) = read_zone(&memory, 0)? else {
                continue;
            };
            for record in read_records(&memory, &found, NPC)? {
                let here = (zone, u16(&record, NPC_INDEX)?);
                let same = (
                    u16(&record, NPC_X)?,
                    u16(&record, NPC_Z)?,
                    u16(&record, NPC_SPRITE)?,
                    u16(&record, NPC_SCRIPT)?,
                );
                let one = *first.entry(same).or_insert(here);
                if one != here {
                    out.insert(here, one);
                }
            }
        }
        let out = Rc::new(out);
        self.twins_of
            .borrow_mut()
            .insert(parts.to_vec(), out.clone());
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

    /// What each landform worked out of the zone's matrix (`Landform.found`), in
    /// `Game.landforms`' order: once a map, off every chunk of the matrix where the ROM's are
    /// read, and off the chunks loaded without them. The game loads the quarter round the
    /// player, and Route 119's rails are 67 cells north of where it starts.
    fn found(&self, zone: i64) -> Result<Rc<Vec<Worked>>> {
        let mut loaded: IndexMap<(i64, i64), Rc<Chunk>> = IndexMap::new();
        for chunk in self.chunks_loaded()? {
            loaded.insert(chunk.key(), chunk);
        }
        let (wide, files) = {
            let held = self.terrain.files.borrow();
            (held.0, held.1.clone())
        };
        let key: FoundKey = (
            zone,
            files.clone(),
            if self.terrain.known() {
                Vec::new()
            } else {
                let mut keys: Vec<(i64, i64)> = loaded.keys().copied().collect();
                keys.sort();
                keys
            },
            self.grid.borrow().clone(),
        );
        if let Some(found) = self.found_of.borrow().get(&key) {
            return Ok(found.clone());
        }
        let mut chunks = loaded;
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
        let zone_at = |cell: Cell| self.zone_at(cell);
        let matrix = Matrix {
            zone,
            chunks,
            zone_at: &zone_at,
            terrain: &self.terrain,
            zones: &self.zones,
        };
        let mut found: Vec<Worked> = Vec::new();
        for form in &self.game.landforms {
            found.push(form.found(&matrix)?);
        }
        let found = Rc::new(found);
        self.found_of.borrow_mut().insert(key, found.clone());
        Ok(found)
    }

    /// What the landforms are listed and laid on, where `where` is: its `people` where the
    /// listing has read them, and else read only where a landform asks.
    fn here<'a>(&'a self, r#where: &'a Field, people: Option<Vec<Interactable>>) -> Here<'a> {
        let zone = r#where.zone;
        Here {
            r#where,
            people: match people {
                Some(people) => Box::new(move || Ok(people.clone())),
                None => Box::new(move || self.people(zone)),
            },
            clears: &self.game.clears,
            memory: &*self.memory,
            data: Box::new(|| self.current()),
            home: Box::new(move |data| self.home(data, &[zone])),
            trigger_cells: Box::new(move |none, known| self.trigger_cells(zone, none, known)),
            rails: &self.rails,
            files: self.terrain.files.borrow().1.clone(),
            natives: &self.text.natives,
            turned: Box::new(|state, size| match &self.switches {
                None => Ok(None),
                Some(switches) => switches.turned(state, size),
            }),
            gym_floor: self.text.gym_floor.as_ref(),
            watch: Box::new(|form| self.watches.get(&identity(form)).cloned()),
            terrain: &self.terrain,
            zones: &self.zones,
            chunks: Box::new(|| self.chunks_loaded()),
            zone_at: Box::new(|cell| self.zone_at(cell)),
            furniture: Box::new(move |cell| self.furniture_at(zone, cell)),
        }
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
    let chunk = &terrain.with_streets(chunk)?;
    if stacked && !chunk.floors.is_empty() {
        let flat = terrain.flattened(chunk, None, false)?;
        return Ok(Rc::new(Chunk {
            floors: chunk.floors.clone(),
            ..(*flat).clone()
        }));
    }
    terrain.flattened(chunk, level, true)
}

/// Whether every cell round `cells`, and they, are walls in their chunks (`chunk_at`) or off
/// the map's north or west edge, where no chunk is: nothing stands beside them to face them
/// from. Prism Tower's quiz rooms keep a record in the corner at (0, 0) (spike 252). A chunk with
/// a deck is no wall: its cell table is the ground's alone.
fn buried(chunk_at: &dyn Fn(Cell) -> Result<Option<Rc<Chunk>>>, cells: &[Cell]) -> Result<bool> {
    let wall = |near: Cell| -> Result<bool> {
        if near.0 < 0 || near.1 < 0 {
            return Ok(true);
        }
        Ok(match chunk_at(near)? {
            None => false,
            Some(chunk) => chunk.floors.is_empty() && !chunk.walkable(near)?,
        })
    };
    for &(x, z) in cells {
        for dx in -1..=1 {
            for dz in -1..=1 {
                if !wall((x + dx, z + dz))? {
                    return Ok(false);
                }
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
/// and the cabinet under it, and was listed three times in one cell (spike 211). A sign that
/// reaches past what covers it, a shelf longer than what stands on it, stays.
/// A trigger is no layer of a spot: it is stepped on, not read, and on a floor of its own, as
/// Laverre Gym's teleporter at (29, 1) on its middle floor is under the cells of a bureau's
/// record, which has no floor (spike 258).
fn one_a_spot(items: Vec<Interactable>) -> Vec<Interactable> {
    let covered: Vec<(i64, HashSet<Cell>)> = items
        .iter()
        .filter(|item| layered(item))
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
        .filter(|item| !layered(item) || !under(item))
        .collect()
}

/// Whether `item` is one of the signs a spot stacks, read where it stands (`_one_a_spot`).
fn layered(item: &Interactable) -> bool {
    item.category == "signs" && item.place_kind() != Some(TRIGGER_SOURCE)
}

/// `FieldReader._rail_teleporters`, over the rails' `teleports` and `place_height`, which
/// Python's test hands a stand-in for: the teleports on the corridor's streets, each a teleporter
/// where it sends the player from, carrying them to where it puts them (`Ground.teleports`), the
/// first of several from one place. Anistar City Gym's lobby carpet puts the player on one of
/// three tracks of one joint, the cell it is in once the lobby's street is laid onto them: it is
/// listed, where the Gym appears, and carries the player nowhere (spike 264). Each carries them
/// back too: the module lists one way of most, and a player stepping back onto the first one's
/// landing was put on its start, three times (2026-10-30).
pub(crate) fn rail_teleporters(
    teleports: &[rail::Teleport],
    place_height: &dyn Fn(rail::Place) -> Option<f64>,
    corridor: &rail::Corridor,
) -> Result<Vec<Interactable>> {
    // A set only tested.
    let mut seen: HashSet<Cell> = HashSet::new();
    let back = teleports.iter().map(|t| rail::Teleport {
        start: t.lands,
        lands: t.start,
    });
    let mut out: Vec<Interactable> = Vec::new();
    for (i, teleport) in teleports.iter().copied().chain(back).enumerate() {
        let start = corridor.at(teleport.start.0, teleport.start.1, teleport.start.2)?;
        let lands = corridor.at(teleport.lands.0, teleport.lands.1, teleport.lands.2)?;
        let (Some(start), Some(lands)) = (start, lands) else {
            continue;
        };
        if seen.contains(&start) {
            continue;
        }
        seen.insert(start);
        let moves = lands != start;
        out.push(Interactable {
            code: i as i64,
            source: place_source(TELEPORT_SOURCE, i as i64)?,
            height: place_height(teleport.start),
            arrival: moves.then_some(lands),
            landing: if moves {
                place_height(teleport.lands)
            } else {
                None
            },
            ..Interactable::new("signs", TELEPORTER, start)
        });
    }
    Ok(out)
}
