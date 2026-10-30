//! Field tests, against a fake town laid out as spike 78 read Littleroot: a zone data file behind
//! its holder, a pool of move models, the field map's grid and the zone table, and the hidden
//! Pokémon's record as spike 80 read it on Route 101.

use std::cell::{Cell as StdCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_rail;
use crate::core::config::{HIDDEN_OFF, HIDDEN_UNDERWATER};
use crate::core::discovery::{ALLOCATION, Found};
use crate::core::memory::Memory;
use crate::core::pathfind;
use crate::core::rtti::ClassNames;
use crate::games::common::field::chunk::{
    ANY_FLOOR, BLOCKED, CHUNK, COUNTER, Chunk, Floor, KIND_SHIFT,
};
use crate::games::common::field::grind::GrindRail;
use crate::games::common::field::ground::{BLOCK, Ground, NO_ZONE, Of, pushes};
use crate::games::common::field::rotating::{Doors, RotatingDoor};
use crate::games::common::field::things::{
    ASH_SOURCE, BERRY_SOURCE, DECORATION_SOURCE, EDGE_SOURCE, FURNITURE_SOURCE, Field,
    Interactable, SECRET_BASE_SOURCE, TILE_SOURCE, WARP_SOURCE, describe, place_source,
    source_kind, source_number,
};
use crate::games::common::field::traversal::Traversal;
use crate::games::gen6::field::features::SecretBaseReader;
use crate::games::gen6::field::features::{Feature, HiddenReader, Tracked, ZoneAt};
use crate::games::gen6::field::fieldmap::{
    MAP_BLOCK_SIZE, MAP_CHUNK_FILES, MAP_CHUNKS_TALL, MAP_CHUNKS_WIDE, MAP_GRID, MAP_KIND, MAP_MARK,
};
use crate::games::gen6::field::hidden_items::HIDDEN_ITEM;
use crate::games::gen6::field::landform::{ArrowTiles, Landform, Rides, gen6};
use crate::games::gen6::field::mmodel::PROP_MODEL;
use crate::games::gen6::field::reader::PC_TILE as PC_KIND;
use crate::games::gen6::field::reader::{
    ASKS, FieldReader, MOVE_MODEL_CLASS, TABLE_ASKS, TABLE_EVERY,
};
use crate::games::gen6::field::terrain::NO_CHUNK_FILE;
use crate::games::gen6::field::terrain::{FarCells, FarFloors, FarHeights, FarMatrices, Terrain};
use crate::games::gen6::field::works::{EVENT_CLASS, FIRST_WORK, WORKS};
use crate::games::gen6::field::zonedata::{
    ENTITY_HEADER, FURNITURE, NPC, WARP, WARP_DOOR, npc_trainers, zone_header,
};
use crate::games::gen6::field::zonetable::{
    INFO_KIND, INFO_MATRIX, INFO_NAME, INFO_PARENT, INFO_SETTING, INFO_SIZE, UNDERWATER,
};
use crate::games::gen6::game::{FeatureFactory, FurnitureKind, Game, SecretBaseFactory};
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::rom::maps::{ChunkSource, StreetSource};
use crate::games::oras;
use crate::games::oras::field::berries::BerrySpots;
use crate::games::oras::field::hidden::{self, DexNav};
use crate::games::oras::field::{ash, doors, ice, secret_base};
use crate::games::xy;
use crate::games::{
    gen6::field::camera, gen6::field::mapblock, gen6::field::mmodel, gen6::field::rail,
};
use crate::py::{self, Error, Kind, Result};

pub(crate) const TOWN: i64 = 0x6;
pub(crate) const ROUTE: i64 = 0x17;
pub(crate) const HOUSE: i64 = 0xDF;
pub(crate) const BEDROOM: i64 = 0xE0;
pub(crate) const FILE: i64 = 0x08D06000;
pub(crate) const OTHER: i64 = 0x08D02000;
pub(crate) const HOLDER: i64 = 0x08D0B930;
pub(crate) const POOL: i64 = 0x08D1E000;
pub(crate) const MODEL_STRIDE: i64 = 0xAB0;
pub(crate) const MAP: i64 = 0x08CF7DAC;
pub(crate) const TABLE: i64 = 0x08D11E54;
pub(crate) const TABLE_POINTER: i64 = 0x0864F754;
/// The hidden Pokémon's record, where six captures had it.
pub(crate) const HIDDEN: i64 = 0x08D374E4;
pub(crate) const HIDDEN_HEAP: i64 = 0x08D375F0;
pub(crate) const CORE_PARAM: i64 = 0x08D376C4;
/// The one `field::PlayerWork`, where three captures of movie3 had it (spike 135).
pub(crate) const WORK: i64 = 0x08C6A890;
/// The save's `field::EventWork`, where the Sea Mauville replays had it.
pub(crate) const EVENT_WORK: i64 = 0x08C7DAE8;
pub(crate) const POOCHYENA: i64 = 0x105;
/// The map blocks holding Littleroot's chunk and Route 101's, where spike 82 found them.
pub(crate) const TOWN_BLOCK: i64 = 0x08252118;
pub(crate) const ROUTE_BLOCK: i64 = 0x0824C060;
/// Their files in the linear heap.
pub(crate) const TOWN_CHUNK: i64 = 0x14AF3780;
pub(crate) const ROUTE_CHUNK: i64 = 0x148F3580;
/// The way north out of Littleroot: cells 99 to 101 of the border.
pub(crate) const GAP: (i64, i64) = (99, 102);

/// The ROM's map chunks as a test gives them, a function each.
struct Chunks {
    cells: FarCells,
    heights: Option<FarHeights>,
    floors: Option<FarFloors>,
    matrix: Option<FarMatrices>,
}

impl std::fmt::Debug for Chunks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Chunks")
    }
}

impl ChunkSource for Chunks {
    fn cells(&self, file: i64) -> Result<Option<Vec<i64>>> {
        (self.cells)(file)
    }

    fn floors(&self, file: i64) -> Result<Vec<Floor>> {
        match &self.floors {
            None => Ok(Vec::new()),
            Some(floors) => floors(file),
        }
    }

    fn heights(&self, file: i64) -> Result<Option<Vec<Option<f64>>>> {
        match &self.heights {
            None => Ok(None),
            Some(heights) => heights(file),
        }
    }

    fn matrix(&self, number: i64) -> Result<Option<(i64, Vec<i64>)>> {
        match &self.matrix {
            None => Ok(None),
            Some(matrix) => matrix(number),
        }
    }
}

/// Lumiose's streets as a test gives them.
struct Streets(rail::Streets);

impl std::fmt::Debug for Streets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Streets")
    }
}

impl StreetSource for Streets {
    fn network(&self, chunks: &[i64]) -> Result<rail::Network> {
        (self.0)(chunks)
    }
}

thread_local! {
    static GIVEN_FEATURE: RefCell<Option<Rc<dyn Feature>>> = const { RefCell::new(None) };
    static GIVEN_BASES: RefCell<Option<Rc<dyn SecretBaseReader>>> = const { RefCell::new(None) };
}

/// `_given(reader)`: a game's reader factory that hands over the reader a test has made. A
/// factory is a plain `fn`, so the reader waits in a thread-local for the field reader to ask.
fn given_feature(reader: Rc<dyn Feature>) -> FeatureFactory {
    GIVEN_FEATURE.with(|given| *given.borrow_mut() = Some(reader));
    |_memory, _text| GIVEN_FEATURE.with(|given| given.borrow().clone().expect("a test's reader"))
}

/// `_given(bases)`, for `Game.secret_bases`.
fn given_bases(bases: Rc<dyn SecretBaseReader>) -> SecretBaseFactory {
    GIVEN_BASES.with(|given| *given.borrow_mut() = Some(bases));
    |_memory, _image| GIVEN_BASES.with(|given| given.borrow().clone().expect("a test's bases"))
}

/// `DexNav`, as the factory `Game.hidden` holds.
fn dex_nav(memory: Rc<dyn Memory>) -> Rc<dyn HiddenReader> {
    Rc::new(DexNav::new(memory))
}

/// A gen 6 game with nothing of its own: a test gives it what it needs.
pub(crate) fn game() -> Game {
    Game::new(
        "test",
        [],
        std::path::PathBuf::new,
        oras::rom().clone(),
        mmodel::DRAWN,
    )
}

/// `field_reader`'s keyword arguments that are the ROM's words, chunks, `shared` and `natives`
/// furniture and `teleports`, and the shipped table's `tiles` and `furniture_words`, each as the
/// Python defaults it: set what differs with `..Given::default()`. The rest are the game's
/// fields, given as a `Game { .., ..game() }`.
#[derive(Default)]
pub(crate) struct Given {
    pub words: IndexMap<i64, String>,
    pub tiles: Option<IndexMap<i64, String>>,
    pub furniture_words: Option<IndexMap<i64, String>>,
    pub shared: IndexMap<(i64, i64), i64>,
    pub natives: IndexMap<(i64, i64), (i64, i64)>,
    pub teleports: IndexMap<(i64, i64), (i64, i64, i64)>,
    pub streets: Option<rail::Streets>,
    pub outdoors: HashSet<i64>,
    pub street_records: rail::Records,
    pub berries: Vec<(i64, (i64, i64))>,
    pub decorations: Vec<String>,
    pub zone_table: Vec<u8>,
    pub far: Option<FarCells>,
    pub far_heights: Option<FarHeights>,
    pub far_floors: Option<FarFloors>,
    pub matrices: Option<FarMatrices>,
    /// None is `time.monotonic`.
    pub clock: Option<Rc<dyn Fn() -> f64>>,
    /// `[pa3ds.extras] hidden_items`; None is `HIDDEN_OFF`.
    pub extras_hidden_items: Option<&'static str>,
}

/// A field reader of `game`, the ROM's words, chunks, `shared` and `natives` furniture and
/// `teleports` given, the shipped table's `tiles` and `furniture_words`, and `[pa3ds.extras]
/// hidden_items`.
pub(crate) fn field_reader(memory: Rc<FakeMemory>, given: Given, game: Game) -> Rc<FieldReader> {
    let Given {
        words,
        tiles,
        furniture_words,
        shared,
        natives,
        teleports,
        streets,
        outdoors,
        street_records,
        berries,
        decorations,
        zone_table,
        far,
        far_heights,
        far_floors,
        matrices,
        clock,
        extras_hidden_items,
    } = given;
    let text = GameText {
        pc: words.get(&PC_KIND).cloned(),
        streets: streets.map(|streets| Rc::new(Streets(streets)) as Rc<dyn StreetSource>),
        outdoors,
        street_records,
        berries,
        decorations,
        zone_table,
        map_chunks: far.map(|cells| {
            Rc::new(Chunks {
                cells,
                heights: far_heights,
                floors: far_floors,
                matrix: matrices,
            }) as Rc<dyn ChunkSource>
        }),
        shared,
        natives,
        teleports,
        ..GameText::default()
    };
    FieldReader::new(
        memory,
        &game,
        &text,
        b"image",
        clock.unwrap_or_else(|| Rc::new(py::time::monotonic)),
        tiles.as_ref(),
        furniture_words.as_ref(),
        extras_hidden_items.unwrap_or(HIDDEN_OFF),
    )
}

/// `struct.pack` of halfwords, signed or not: the bytes are the same.
fn le16(values: &[i64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&value| (value as u16).to_le_bytes())
        .collect()
}

/// `struct.pack` of words, signed or not.
fn le32(values: &[i64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&value| (value as u32).to_le_bytes())
        .collect()
}

/// `struct.pack` of single floats.
fn f32s(values: &[f64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&value| (value as f32).to_le_bytes())
        .collect()
}

/// `struct.pack_into`: `data` over `buffer` at `at`.
fn put(buffer: &mut [u8], at: i64, data: &[u8]) {
    buffer[at as usize..at as usize + data.len()].copy_from_slice(data);
}

/// `bytes.ljust(width, b"\0")`.
fn ljust(mut data: Vec<u8>, width: i64) -> Vec<u8> {
    if (data.len() as i64) < width {
        data.resize(width as usize, 0);
    }
    data
}

/// `memory.pages()`, a dict in the order the pages were first written.
pub(crate) fn pages(memory: &FakeMemory) -> IndexMap<i64, Vec<u8>> {
    memory.pages()
}

/// `zone_file`'s keyword arguments, each as the Python defaults it: set what differs with
/// `..ZoneSpec::default()`.
pub(crate) struct ZoneSpec<'a> {
    /// (script, x, z).
    pub furniture: &'a [(i64, i64, i64)],
    /// (script, x, z, width, height), after `furniture`: Python's `furniture` takes either.
    pub sized: &'a [(i64, i64, i64, i64, i64)],
    /// (model code, x, z, script), and the flag at +0x0a after them when it is not the usual
    /// 0x7d0 (3000 and up is a trainer).
    pub npcs: &'a [(i64, i64, i64, i64, Option<i64>)],
    /// (to zone, kind, x, z) in cells, then the height it stands at in world units and the warp
    /// of the zone it goes to that it comes out at, where not 0.
    pub warps: &'a [(i64, i64, i64, i64, Option<i64>, Option<i64>)],
    pub info: &'a [u8],
    /// ORAS's five or X's four, the third the zone's `script`.
    pub sections: i64,
    pub script: &'a [u8],
    /// (script, x, z, width) one cell tall, and the height it stands at after them where it is
    /// not 0, then the value and the work it is armed by.
    pub triggers: &'a [(i64, i64, i64, i64, Option<i64>, Option<(i64, i64)>)],
}

impl Default for ZoneSpec<'_> {
    fn default() -> Self {
        ZoneSpec {
            furniture: &[],
            sized: &[],
            npcs: &[],
            warps: &[],
            info: &[],
            sections: 5,
            script: &[],
            triggers: &[],
        }
    }
}

/// A zone data file: `furniture` as (script, x, z) and its width and height where not 1,
/// `npcs` as (model code, x, z, script) and
/// the flag at +0x0a after them when it is not the usual 0x7d0 (3000 and up is a trainer), and
/// `warps` as (to zone, kind, x, z) in cells, then the height it stands at in world units and
/// the warp of the zone it goes to that it comes out at, where not 0; `triggers` as (script,
/// x, z, width) one cell tall, and the height it stands at after them where it is not 0, then
/// the value and the work it is armed by;
/// `sections` is ORAS's five or X's four, the third the zone's `script`.
pub(crate) fn zone_file(spec: &ZoneSpec) -> Vec<u8> {
    let mut records: Vec<u8> = Vec::new();
    for &(script, x, z) in spec.furniture {
        records.extend(le16(&[script, 3, 0, 0, x, z, 1, 1, 0, 0]));
    }
    for &(script, x, z, wide, deep) in spec.sized {
        records.extend(le16(&[script, 3, 0, 0, x, z, wide, deep, 0, 0]));
    }
    for (i, npc) in spec.npcs.iter().enumerate() {
        records.extend(le16(&[
            i as i64,
            npc.0,
            0,
            0,
            npc.3,
            npc.4.unwrap_or(0x7D0),
            1,
            0,
            0,
            0,
            0,
            0,
            1,
            1,
            0xFFFF,
            0xFFFF,
            0xFFFF,
            0,
            0,
            0,
            npc.1,
            npc.2,
        ]));
        records.extend(f32s(&[0.0]));
    }
    for &(to, kind, x, z, height, slot) in spec.warps {
        records.extend(le16(&[
            to,
            slot.unwrap_or(0),
            kind,
            0,
            x * 18 + 9,
            height.unwrap_or(0),
            z * 18 + 9,
            1,
            1,
            0,
            0,
            0,
        ]));
    }
    for trigger in spec.triggers {
        records.extend(le16(&[
            trigger.0,
            trigger.5.map_or(0, |armed| armed.0),
            trigger.5.map_or(0, |armed| armed.1),
            0,
            0,
            0,
            trigger.1,
            trigger.2,
            trigger.3,
            1,
            trigger.4.unwrap_or(0),
            0,
        ]));
    }
    let mut entities = le32(&[8 + records.len() as i64]);
    entities.extend([
        (spec.furniture.len() + spec.sized.len()) as u8,
        spec.npcs.len() as u8,
        spec.warps.len() as u8,
        spec.triggers.len() as u8,
        0,
        0,
        0,
        0,
    ]);
    entities.extend(records);
    let info = ljust(spec.info.to_vec(), INFO_SIZE);
    let header = zone_header(spec.sections);
    let mut offsets = vec![
        header,
        header + info.len() as i64,
        header + info.len() as i64 + entities.len() as i64,
    ];
    let last = offsets[offsets.len() - 1] + spec.script.len() as i64;
    offsets.extend(std::iter::repeat_n(last, (spec.sections - 2) as usize));
    let mut out = b"ZO".to_vec();
    out.extend(le16(&[spec.sections]));
    out.extend(le32(&offsets));
    out.extend(info);
    out.extend(entities);
    out.extend(spec.script);
    out
}

pub(crate) fn town_info() -> Vec<u8> {
    let mut row = vec![0u8; INFO_SIZE as usize];
    put(&mut row, INFO_PARENT, &le16(&[TOWN, 170]));
    row
}

/// `chunk_file(origin, blocked)`: the common call, as the Python's defaults make it.
pub(crate) fn chunk_file(origin: (i64, i64), blocked: &[(i64, i64)]) -> Vec<u8> {
    chunk_file_with(origin, blocked, &[], 7)
}

/// The file of the chunk whose north-west cell is `origin`, as spike 82 read it: the cell
/// table in section 0, with the words the game uses for a path and for a tree, `tiles` as
/// other words at their cells (a later pair over an earlier, as a dict is), and the other
/// sections empty: six in ORAS, five in X.
pub(crate) fn chunk_file_with(
    origin: (i64, i64),
    blocked: &[(i64, i64)],
    tiles: &[((i64, i64), i64)],
    sections: i64,
) -> Vec<u8> {
    let mut words: Vec<i64> = Vec::new();
    for z in 0..CHUNK {
        for x in 0..CHUNK {
            let cell = (origin.0 + x, origin.1 + z);
            words.push(match tiles.iter().rev().find(|(tile, _)| *tile == cell) {
                Some(&(_, word)) => word,
                None if blocked.contains(&cell) => 0x01000021,
                None => 0x20,
            });
        }
    }
    let mut table = le16(&[CHUNK, CHUNK]);
    table.extend(le32(&words));
    let table = ljust(table, 0x1980);
    let mut offsets = vec![mapblock::HEADER];
    offsets.extend(std::iter::repeat_n(
        mapblock::HEADER + table.len() as i64,
        sections as usize,
    ));
    let mut head = mapblock::MAGIC.to_vec();
    head.extend(le16(&[sections]));
    head.extend(le32(&offsets));
    let mut out = ljust(head, mapblock::HEADER);
    out.extend(table);
    out
}

/// A row of trees across a chunk at `z`, with the way north out of Littleroot for a gap.
pub(crate) fn border(z: i64) -> Vec<(i64, i64)> {
    border_with(z, &[GAP])
}

/// A row of trees across a chunk at `z`, with gaps at these x ranges.
pub(crate) fn border_with(z: i64, gaps: &[(i64, i64)]) -> Vec<(i64, i64)> {
    (80..120)
        .filter(|x| !gaps.iter().any(|&(lo, hi)| lo <= *x && *x < hi))
        .map(|x| (x, z))
        .collect()
}

/// Littleroot as spike 78 found it, with the player outside the door.
pub(crate) struct Town {
    pub memory: Rc<FakeMemory>,
    pub vtable: i64,
    pub block_vtable: i64,
    /// The map blocks in use, by address, each with its file.
    pub blocks: IndexMap<i64, i64>,
    pub chunk_sections: i64,
    pub width: i64,
    pub heap_vtable: i64,
    pub core_vtable: i64,
    pub reader: Rc<FieldReader>,
}

impl Town {
    pub(crate) fn new() -> Self {
        Town::with(true, 5, 7)
    }

    /// `Town(**game)`: the town read by a reader of `game`, with `DexNav` unless it hides its own.
    pub(crate) fn of(game: Game) -> Self {
        Town::with_game(true, 5, 7, game)
    }

    pub(crate) fn with(chunks: bool, sections: i64, chunk_sections: i64) -> Self {
        Town::with_game(chunks, sections, chunk_sections, game())
    }

    fn with_game(chunks: bool, sections: i64, chunk_sections: i64, chosen: Game) -> Self {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let vtable = classes.add(MOVE_MODEL_CLASS);
        let block_vtable = classes.add(mapblock::CLASS);
        let mut town = Town {
            memory: memory.clone(),
            vtable,
            block_vtable,
            blocks: IndexMap::new(),
            chunk_sections,
            width: 0,
            heap_vtable: 0,
            core_vtable: 0,
            reader: field_reader(memory.clone(), Given::default(), game()),
        };
        let data = zone_file(&ZoneSpec {
            furniture: &[(0x13, 97, 171)],
            npcs: &[
                (0x119, 91, 175, 0x227, None),
                (mmodel::ITEM_BALL, 100, 180, 0x51A, None),
                (PROP_MODEL + 0x4E, 110, 170, 0x205, None),
            ],
            warps: &[
                (HOUSE, WARP_DOOR, 94, 171, None, None),
                (0xE1, WARP_DOOR, 106, 171, None, None),
            ],
            info: &town_info(),
            sections,
            ..ZoneSpec::default()
        });
        memory.write(FILE, &data);
        memory.write(OTHER, &zone_file(&ZoneSpec::default()));
        memory.u32(HOLDER, &[FILE, OTHER, 0x11 | (data.len() as i64) << 8]);
        // The pool: the player, the townsperson walked from its record's cell, the ball, the prop.
        town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (94, 172));
        town.model(1, TOWN, 0, 0x119, (109, 176));
        town.model(2, TOWN, 1, mmodel::ITEM_BALL, (100, 180));
        town.model(3, TOWN, 2, PROP_MODEL + 0x4E, (110, 170));
        for slot in 4..32 {
            memory.u32(POOL + slot * MODEL_STRIDE, &[town.vtable, 0]);
        }
        // The matrix: Route 101 above the town, both four blocks wide.
        town.matrix(8, &[(ROUTE, 8..12, 12..16), (TOWN, 8..12, 16..20)]);
        // The zone table, with the town's row the same bytes as the file's info.
        memory.write(TABLE + TOWN * INFO_SIZE, &town_info());
        let mut route = vec![0u8; INFO_SIZE as usize];
        put(&mut route, INFO_PARENT, &le16(&[ROUTE, 204]));
        memory.write(TABLE + ROUTE * INFO_SIZE, &route);
        let mut house = vec![0u8; INFO_SIZE as usize];
        put(&mut house, INFO_PARENT, &le16(&[TOWN, 170]));
        memory.write(TABLE + HOUSE * INFO_SIZE, &house);
        memory.u32(TABLE_POINTER, &[TABLE + TOWN * INFO_SIZE, TABLE]);
        // The hidden Pokémon's record: empty until `hide` puts one in the grass.
        town.heap_vtable = classes.add(hidden::HEAP_CLASS);
        town.core_vtable = classes.add(hidden::CORE_PARAM_CLASS);
        memory.u32(HIDDEN_HEAP, &[town.heap_vtable]);
        memory.u32(CORE_PARAM, &[town.core_vtable]);
        memory.write(HIDDEN, &vec![0u8; hidden::SIZE as usize]);
        memory.u32(HIDDEN + hidden::HEAP, &[HIDDEN_HEAP]);
        memory.u32(HIDDEN + hidden::CORE, &[CORE_PARAM]);
        memory.u32(WORK, &[classes.add(hidden::PLAYER_WORK_CLASS)]);
        memory.u32(WORK + hidden::WORK_RECORD, &[HIDDEN]);
        memory.write(HIDDEN + hidden::SLOT, &[hidden::NONE as u8]);
        // The chunks around the player: the town's, with trees along its north side but for
        // the way out, and Route 101's, with trees along its south side but for the way in.
        if chunks {
            town.block(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &border(160));
            town.block(ROUTE_BLOCK, ROUTE_CHUNK, (80, 120), &border(159));
        }
        town.reader = field_reader(
            memory.clone(),
            Given::default(),
            Game {
                hidden: chosen.hidden.or(Some(dex_nav)),
                ..chosen
            },
        );
        let _ = ClassNames::new(memory.clone(), CODE);
        town
    }

    /// The field map, `chunks` chunks of four blocks each way, with each zone at its
    /// (columns, rows) of blocks.
    pub(crate) fn matrix(&mut self, chunks: i64, zones: &[(i64, Range<i64>, Range<i64>)]) {
        self.width = chunks * 4;
        self.memory.u32(MAP - 8, &[ALLOCATION, MAP_BLOCK_SIZE]);
        self.memory.u32(MAP + MAP_KIND, &[4, 32]);
        self.memory
            .write(MAP + MAP_CHUNKS_WIDE, &le16(&[chunks, chunks]));
        let mut grid = vec![NO_ZONE; (self.width * self.width) as usize];
        for (zone, columns, rows) in zones {
            for row in rows.clone() {
                for column in columns.clone() {
                    grid[(row * self.width + column) as usize] = *zone;
                }
            }
        }
        self.memory.write(MAP + MAP_GRID, &le16(&grid));
    }

    /// `model(slot, zone, index, code, cell)` of one cell by one.
    pub(crate) fn model(
        &self,
        slot: i64,
        zone: i64,
        index: i64,
        code: i64,
        cell: (i64, i64),
    ) -> i64 {
        self.model_sized(slot, zone, index, code, cell, (1, 1))
    }

    pub(crate) fn model_sized(
        &self,
        slot: i64,
        zone: i64,
        index: i64,
        code: i64,
        cell: (i64, i64),
        size: (i64, i64),
    ) -> i64 {
        let at = POOL + slot * MODEL_STRIDE;
        self.memory.u32(at, &[self.vtable, 1]);
        self.memory
            .write(at + mmodel::SIZE, &[size.0 as u8, size.1 as u8]);
        self.memory.u32(at + mmodel::ZONE, &[zone << 16 | index]);
        self.memory.u32(at + mmodel::CHARACTER, &[code]);
        self.memory
            .write(at + mmodel::CELL_X, &f32s(&[cell.0 as f64]));
        self.memory
            .write(at + mmodel::CELL_Z, &f32s(&[cell.1 as f64]));
        // On the grid +0xec is where it is in the world too; on a rail it is 0.
        for x in [mmodel::PLACE_X, mmodel::WORLD_X] {
            self.memory.write(
                at + x,
                &f32s(&[(cell.0 * 18 + 9) as f64, 0.0, (cell.1 * 18 + 9) as f64]),
            );
        }
        at
    }

    /// Put a Pokémon in the grass at `cell`, as the game writes its record.
    pub(crate) fn hide(&self, species: i64, level: i64, cell: (i64, i64)) {
        self.hide_with(species, level, cell, true);
    }

    /// Put a Pokémon in the grass at `cell`, as the game writes its record; `up=False`
    /// leaves it as a battle does, the Pokémon still named and the slot byte 0xff.
    pub(crate) fn hide_with(&self, species: i64, level: i64, cell: (i64, i64), up: bool) {
        self.hide_state(species, level, cell, if up { 4 } else { hidden::NONE });
    }

    /// `hide(..., state=state)`: the slot byte written as given, found by the detector.
    pub(crate) fn hide_state(&self, species: i64, level: i64, cell: (i64, i64), state: i64) {
        self.hide_found(species, level, cell, state, 1);
    }

    /// `hide(..., state=state, found_by=found_by)`: how it was found too, 3 by a search.
    pub(crate) fn hide_found(
        &self,
        species: i64,
        level: i64,
        cell: (i64, i64),
        state: i64,
        found_by: i64,
    ) {
        self.memory.u32(HIDDEN + hidden::SPECIES, &[species, level]);
        self.memory
            .write(HIDDEN + hidden::FOUND_BY, &[found_by as u8]);
        self.memory.u32(HIDDEN + hidden::SET, &[1]);
        self.memory.write(HIDDEN + hidden::SLOT, &[state as u8]);
        self.memory
            .write(HIDDEN + hidden::WORLD_X, &f32s(&[(cell.0 * 18 + 9) as f64]));
        self.memory
            .write(HIDDEN + hidden::WORLD_Z, &f32s(&[(cell.1 * 18 + 9) as f64]));
    }

    /// A map block with no use tiles besides the ones at `blocked`.
    pub(crate) fn block(
        &mut self,
        address: i64,
        file: i64,
        origin: (i64, i64),
        blocked: &[(i64, i64)],
    ) {
        self.block_tiles(address, file, origin, blocked, &[]);
    }

    /// A map block holding the chunk whose north-west cell is `origin`, its file at `file`
    /// in the linear heap, with trees at `blocked` and `tiles` as other words.
    pub(crate) fn block_tiles(
        &mut self,
        address: i64,
        file: i64,
        origin: (i64, i64),
        blocked: &[(i64, i64)],
        tiles: &[((i64, i64), i64)],
    ) {
        let data = chunk_file_with(origin, blocked, tiles, self.chunk_sections);
        self.memory.write(file, &data);
        self.memory
            .u32(address, &[self.block_vtable, 0x01010101, 0]);
        let centre = ((origin.0 + CHUNK / 2) * 18, (origin.1 + CHUNK / 2) * 18);
        self.memory
            .write(address + mapblock::CENTRE_X, &f32s(&[centre.0 as f64]));
        self.memory
            .write(address + mapblock::CENTRE_Z, &f32s(&[centre.1 as f64]));
        self.memory.u32(address + mapblock::ARC, &[address + 0x700]);
        self.memory.u32(address + mapblock::FILE, &[file]);
        self.memory
            .u32(address + mapblock::SIZE, &[data.len() as i64]);
        self.memory
            .u32(address + mapblock::STATE, &[mapblock::READY]);
        self.blocks.insert(address, file);
    }

    pub(crate) fn found(&self) -> Found {
        let mut heap: IndexMap<i64, String> = IndexMap::new();
        for slot in 0..32 {
            heap.insert(POOL + slot * MODEL_STRIDE, MOVE_MODEL_CLASS.to_string());
        }
        heap.insert(HIDDEN_HEAP, hidden::HEAP_CLASS.to_string());
        heap.insert(WORK, hidden::PLAYER_WORK_CLASS.to_string());
        for &block in self.blocks.keys() {
            heap.insert(block, mapblock::CLASS.to_string());
        }
        Found {
            pages: pages(&self.memory),
            heap,
            partial: false,
        }
    }

    pub(crate) fn r#where(&self) -> Result<Field> {
        self.reader.track(&self.found())?;
        let r#where = self.reader.poll()?;
        let r#where = r#where.expect("the player's field");
        self.reader.interactables(&r#where)
    }

    pub(crate) fn listing(&self) -> Result<Vec<String>> {
        describe(&self.r#where()?, None, None).collect()
    }
}

/// `ground.goals(...)` as a map written the way the tests write one.
fn goals_of(
    cells: &[((i64, i64), Option<&'static str>)],
) -> IndexMap<(i64, i64), Option<&'static str>> {
    cells.iter().copied().collect()
}

#[test]
fn test_the_player_is_the_model_with_index_ff() -> Result<()> {
    let town = Town::new();
    town.reader.track(&town.found())?;
    let r#where = town.reader.poll()?.expect("a field");
    assert_eq!((r#where.zone, r#where.cell), (TOWN, (94, 172)));
    Ok(())
}

/// Jagged Pass: +0x06 read 0x01 on foot and 0x21 on the Mach Bike (omega_ruby_bike.ctm,
/// spike 158). Surfing sets it too, with +0x07 0x13 where the bike has 0x03, and is not the bike
/// (omega_ruby_surf3.ctm, spike 162).
#[test]
fn test_the_player_is_on_the_bike_while_the_models_riding_bit_is_set() -> Result<()> {
    let town = Town::new();
    town.reader.track(&town.found())?;
    town.memory.write(POOL + 6, &[0x01, 0x00]);
    assert!(town.reader.poll()?.is_some_and(|r#where| !r#where.on_bike));
    town.memory.write(POOL + 6, &[0x21, 0x03]);
    assert!(town.reader.poll()?.is_some_and(|r#where| r#where.on_bike));
    town.memory.write(POOL + 6, &[0x21, 0x13]);
    assert!(town.reader.poll()?.is_some_and(|r#where| !r#where.on_bike));
    Ok(())
}

/// kyogre.ctm, sharpedo.ctm and movie43 (spike 227): +0x06 and +0x07 read 0x21 and 0x03 on the
/// Mach Bike, on Sharpedo and on Kyogre alike, and +0xa2c 2 on the bike, 0 on a Pokémon. Just
/// after a save is loaded on the water it reads 1, as on foot.
#[test]
fn test_the_bike_is_told_from_a_pokemon_by_what_the_player_moves_as() -> Result<()> {
    let riding = |six: u8, seven: u8| {
        let mut head = vec![0u8; mmodel::RIDING as usize];
        head.extend([six, seven]);
        head.extend(vec![0u8; (mmodel::HEAD - mmodel::RIDING - 2) as usize]);
        head
    };
    let head = riding(0x21, 0x03);
    assert!(mmodel::on_bike(&head, Some(2))? && !mmodel::surfing(&head, Some(2))?);
    assert!(!mmodel::on_bike(&head, Some(0))? && mmodel::surfing(&head, Some(0))?);
    assert!(mmodel::surfing(&head, Some(1))?);
    // X's, by +0x07 alone
    assert!(mmodel::on_bike(&head, None)?);
    let head = riding(0x01, 0x00);
    assert!(!mmodel::on_bike(&head, Some(1))? && !mmodel::surfing(&head, Some(1))?);
    Ok(())
}

#[test]
fn test_nothing_is_said_before_a_sweep() -> Result<()> {
    assert!(Town::new().reader.poll()?.is_none());
    Ok(())
}

#[test]
fn test_the_listing_names_each_category_nearest_first() -> Result<()> {
    assert_eq!(
        Town::new().listing()?,
        [
            "exits: door to zone 0xdf 1 north; door to zone 0xe1 12 east 1 north; \
             edge to zone 0x17 5 east 13 north",
            "signs: sign 3 east 1 north; object 16 east 2 north",
            "people: person 15 east 4 south",
            "items: item ball 6 east 8 south",
        ]
    );
    Ok(())
}

#[test]
fn test_a_person_is_where_its_model_is_not_where_its_record_was() -> Result<()> {
    let cells: Vec<(i64, i64)> = Town::new()
        .r#where()?
        .of("people")
        .iter()
        .map(|item| item.cell)
        .collect();
    assert_eq!(cells, [(109, 176)]);
    Ok(())
}

/// Walking into the next zone empties the models of the zone left and fills their slots with
/// the new zone's people: the way to a Youngster on Route 102 became a way to Norman in
/// Petalburg City, in the same slot, and nothing said the Youngster had gone (spike 131, movie4).
/// A battle builds everybody again where they were, and they stay who they were.
#[test]
fn test_a_slot_filled_with_another_zones_person_is_not_the_same_person() -> Result<()> {
    let town = Town::new();
    let youngster = town.r#where()?.of("people")[0].key();
    town.model(1, ROUTE, 0, 0x119, (109, 176));
    assert_ne!(town.r#where()?.of("people")[0].key(), youngster);
    town.model(1, TOWN, 0, 0x119, (109, 176));
    assert_eq!(town.r#where()?.of("people")[0].key(), youngster);
    Ok(())
}

#[test]
fn test_the_zone_comes_from_the_matrix_once_the_player_has_walked_on() -> Result<()> {
    let town = Town::new();
    town.reader.track(&town.found())?;
    town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (99, 150));
    let r#where = town.reader.poll()?.expect("a field");
    assert_eq!(r#where.zone, ROUTE);
    Ok(())
}

/// Spike 159: the foot of Route 119 has the top of the route for its parent and is on the
/// same matrix, and the way from Route 118 onto it was said as stairs. A room's matrix is its
/// own.
#[test]
fn test_a_warp_between_two_outdoor_zones_is_a_path() -> Result<()> {
    let town = Town::new();
    let mut route = vec![0u8; INFO_SIZE as usize];
    let mut foot = vec![0u8; INFO_SIZE as usize];
    let mut bedroom = vec![0u8; INFO_SIZE as usize];
    for row in [&mut route, &mut foot] {
        put(row, INFO_PARENT, &le16(&[ROUTE, 204]));
        put(row, INFO_MATRIX, &le16(&[24]));
    }
    put(&mut bedroom, INFO_PARENT, &le16(&[TOWN, 170]));
    put(&mut bedroom, INFO_MATRIX, &le16(&[0xA5]));
    town.memory.write(TABLE + ROUTE * INFO_SIZE, &route);
    town.memory.write(TABLE + 0x2F * INFO_SIZE, &foot);
    town.memory.write(TABLE + BEDROOM * INFO_SIZE, &bedroom);
    let data = zone_file(&ZoneSpec {
        warps: &[
            (ROUTE, 0x0503, 80, 100, None, None),
            (BEDROOM, 0x0501, 16, 9, None, None),
            (0x2F, 0x0501, 30, 40, None, None),
        ],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    let exits = town.listing()?[0].clone();
    assert!(exits.contains("path to zone 0x17") && exits.contains("stairs to zone 0xe0"));
    assert!(exits.contains("path to zone 0x2f"));
    Ok(())
}

/// Devon Corporation's 0x0501 at (31, 14) stands at 9 over ground 0 at (31, 15), and its
/// zone 0x120's 0x0503 at (10, 10) at -9 over ground 0 at (11, 10): stairs up and stairs down,
/// where Victory Road's and Seafloor Cavern's stand at the ground's own height, a passage
/// (2026-10-06). The kind's low byte is the way it is walked into, never up or down.
#[test]
fn test_a_warp_walked_through_is_stairs_by_its_height_over_the_ground_it_is_walked_from()
-> Result<()> {
    let mut town = Town::with(false, 5, 7);
    let mut files = vec![NO_CHUNK_FILE; 64];
    files[4 * 8 + 2] = 7;
    town.memory.write(MAP + MAP_CHUNK_FILES, &le16(&files));
    let mut bedroom = vec![0u8; INFO_SIZE as usize];
    put(&mut bedroom, INFO_PARENT, &le16(&[TOWN, 170]));
    put(&mut bedroom, INFO_MATRIX, &le16(&[0xA5]));
    town.memory.write(TABLE + BEDROOM * INFO_SIZE, &bedroom);
    let heights: Vec<Option<f64>> = (0..CHUNK * CHUNK)
        .map(|i| {
            Some(if (i % CHUNK, i / CHUNK) == (30, 12) {
                -54.0
            } else {
                0.0
            })
        })
        .collect();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            far: Some(Rc::new(|_file| Ok(None))),
            far_heights: Some(Rc::new(move |file| {
                Ok(if file == 7 {
                    Some(heights.clone())
                } else {
                    None
                })
            })),
            ..Given::default()
        },
        game(),
    );
    let data = zone_file(&ZoneSpec {
        warps: &[
            (BEDROOM, 0x0501, 94, 171, Some(9), None),
            (BEDROOM, 0x0503, 100, 171, Some(-9), None),
            (BEDROOM, 0x0501, 110, 171, Some(-54), None),
        ],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    let exits: Vec<Interactable> = town
        .r#where()?
        .of("exits")
        .into_iter()
        .filter(|item| item.zone == Some(BEDROOM))
        .collect();
    let kinds: HashMap<(i64, i64), String> = exits
        .iter()
        .map(|item| (item.cell, item.kind.clone()))
        .collect();
    assert_eq!(
        kinds,
        HashMap::from([
            ((94, 171), "stairs up".to_string()),
            ((100, 171), "stairs down".to_string()),
            ((110, 171), "passage".to_string()),
        ])
    );
    let ways: HashMap<(i64, i64), Option<&str>> =
        exits.iter().map(|item| (item.cell, item.way_in)).collect();
    assert_eq!(
        ways,
        HashMap::from([
            ((94, 171), Some("north")),
            ((100, 171), Some("west")),
            ((110, 171), Some("north")),
        ])
    );
    Ok(())
}

/// Victory Road's ladders were said "warp" (2026-10-06): 0x0b00 leads down and 0x0a01 up. A
/// kind the table does not have is still a warp, and without a table all of them are.
#[test]
fn test_a_ladder_a_teleporter_and_an_entrance_are_what_the_games_table_of_kinds_says() -> Result<()>
{
    let mut town = Town::new();
    let data = zone_file(&ZoneSpec {
        warps: &[
            (BEDROOM, 0x0B00, 94, 171, None, None),
            (BEDROOM, 0x0A01, 100, 171, None, None),
            (BEDROOM, 0x0F01, 106, 171, None, None),
            (BEDROOM, 0x0999, 110, 171, None, None),
        ],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    let kinds: HashSet<String> = town
        .r#where()?
        .of("exits")
        .into_iter()
        .filter(|item| item.zone == Some(BEDROOM))
        .map(|item| item.kind)
        .collect();
    assert_eq!(kinds, HashSet::from(["warp".to_string()]));
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            warps: oras::warps().clone(),
            ..game()
        },
    );
    let kinds: HashMap<(i64, i64), String> = town
        .r#where()?
        .of("exits")
        .into_iter()
        .filter(|item| item.zone == Some(BEDROOM))
        .map(|item| (item.cell, item.kind))
        .collect();
    assert_eq!(
        kinds,
        HashMap::from([
            ((94, 171), "ladder down".to_string()),
            ((100, 171), "ladder up".to_string()),
            ((106, 171), "entrance".to_string()),
            ((110, 171), "warp".to_string()),
        ])
    );
    Ok(())
}

/// Spike 251: the Tower of Mastery's stairs up are three warps side by side into zone 0xb7, and
/// three more on the same cells into 0xb9, both the Tower, and were listed six times. Lumiose's
/// mats out of its Pokémon Centers are three too. One kind into one zone is one exit over the
/// cells it covers, and one on the same cells into the same place is said and walked to alike.
/// Lilycove's way into the hideout is Team Aqua's and Team Magma's on the same cells, and Omega
/// Ruby takes the second: into another place, both stay.
#[test]
fn test_warps_side_by_side_into_one_zone_are_one_exit_and_one_place_on_a_cell_once() -> Result<()> {
    let town = Town::new();
    let data = zone_file(&ZoneSpec {
        warps: &[
            (BEDROOM, 0x0500, 94, 171, None, None),
            (BEDROOM, 0x0500, 95, 171, None, None),
            (BEDROOM, 0x0500, 96, 171, None, None),
            (HOUSE, 0x0500, 94, 171, None, None),
            (HOUSE, 0x0500, 95, 171, None, None),
            (HOUSE, 0x0500, 96, 171, None, None),
            (BEDROOM, 0x0500, 98, 171, None, None),
        ],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    for zone in [BEDROOM, HOUSE] {
        town.memory.u16(TABLE + zone * INFO_SIZE + INFO_NAME, 170);
    }
    let exits = |town: &Town| -> Result<Vec<(Option<i64>, (i64, i64), (i64, i64))>> {
        let mut out: Vec<(Option<i64>, (i64, i64), (i64, i64))> = town
            .r#where()?
            .of("exits")
            .into_iter()
            .filter(|item| item.zone == Some(BEDROOM) || item.zone == Some(HOUSE))
            .map(|item| (item.zone, item.cell, item.size))
            .collect();
        out.sort();
        Ok(out)
    };
    assert_eq!(
        exits(&town)?,
        [
            (Some(BEDROOM), (94, 171), (3, 1)),
            (Some(BEDROOM), (98, 171), (1, 1)),
        ]
    );
    town.memory.u16(TABLE + HOUSE * INFO_SIZE + INFO_NAME, 171);
    assert_eq!(
        exits(&town)?,
        [
            (Some(HOUSE), (94, 171), (3, 1)),
            (Some(BEDROOM), (94, 171), (3, 1)),
            (Some(BEDROOM), (98, 171), (1, 1)),
        ]
    );
    Ok(())
}

/// A warp's high byte is what it is and its low byte the way it is walked into, as a stair's
/// is: Aquacorde's houses' mats out face east and west (0x0202, 0x0203) and Lumiose's doors
/// north, east, south and west (0x0301, 0x0302, 0x0300, 0x0303), and every one but the first
/// two kinds was said "warp" (spike 220). The game's table still wins: X's 0x0201 is a door.
#[test]
fn test_a_door_or_a_mat_out_walked_into_any_way_is_a_door_or_a_way_out() -> Result<()> {
    let mut town = Town::new();
    let data = zone_file(&ZoneSpec {
        warps: &[
            (BEDROOM, 0x0300, 94, 171, None, None),
            (BEDROOM, 0x0302, 98, 171, None, None),
            (BEDROOM, 0x0303, 102, 171, None, None),
            (BEDROOM, 0x0202, 106, 171, None, None),
            (BEDROOM, 0x0203, 110, 171, None, None),
            (BEDROOM, 0x0201, 114, 171, None, None),
        ],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            warps: IndexMap::from([(0x0201, "door".to_string())]),
            ..game()
        },
    );
    let kinds: HashMap<i64, String> = town
        .r#where()?
        .of("exits")
        .into_iter()
        .filter(|item| item.zone == Some(BEDROOM))
        .map(|item| (item.cell.0, item.kind))
        .collect();
    assert_eq!(
        kinds,
        HashMap::from([
            (94, "door".to_string()),
            (98, "door".to_string()),
            (102, "door".to_string()),
            (106, "way out".to_string()),
            (110, "way out".to_string()),
            (114, "door".to_string()),
        ])
    );
    Ok(())
}

/// A warp of a door's kind from one zone of the open air into another is the way through,
/// whichever way it faces: Lumiose's streets onto each other (spike 220), and Route 7 onto Route
/// 6 and into its berry field, which were said "door" (x_route-7and6.ctm, 2026-10-19). The zone
/// table's setting says which zones are the open air, Route 20's woods among them; a house
/// entered by a door of the same kind is not, nor are the S.S. Tidal's cabins.
#[test]
fn test_a_door_between_two_zones_of_the_open_air_is_a_path() -> Result<()> {
    let mut town = Town::new();
    let mut table = town.memory.read(TABLE, (BEDROOM + 1) * INFO_SIZE)?;
    for (zone, setting) in [(TOWN, 1), (ROUTE, 6), (BEDROOM, 2)] {
        table[(zone * INFO_SIZE + INFO_SETTING) as usize] = setting;
    }
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            zone_table: table,
            ..Given::default()
        },
        game(),
    );
    let data = zone_file(&ZoneSpec {
        warps: &[
            (ROUTE, 0x0302, 94, 171, None, None),
            (ROUTE, 0x0301, 98, 171, None, None),
            (BEDROOM, 0x0302, 102, 171, None, None),
        ],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    let kinds: HashMap<i64, String> = town
        .r#where()?
        .of("exits")
        .into_iter()
        .filter(|item| item.cell.1 == 171)
        .map(|item| (item.cell.0, item.kind))
        .collect();
    assert_eq!(
        kinds,
        HashMap::from([
            (94, "path".to_string()),
            (98, "path".to_string()),
            (102, "door".to_string()),
        ])
    );
    Ok(())
}

/// Team Aqua's Hideout (2026-10-06): zone 0x5d's warp 1, a 0x0601 pad, comes out at its zone's
/// warp 2, the warp its +0x02 gives; a warp into another zone says nothing of the sort.
#[test]
fn test_a_teleporter_into_its_own_zone_says_the_cell_it_comes_out_at() -> Result<()> {
    let mut town = Town::new();
    let data = zone_file(&ZoneSpec {
        warps: &[
            (HOUSE, WARP_DOOR, 94, 171, None, None),
            (TOWN, 0x0601, 95, 171, Some(0), Some(2)),
            (TOWN, 0x0601, 100, 176, Some(0), Some(1)),
        ],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            warps: oras::warps().clone(),
            ..game()
        },
    );
    let arrivals: HashMap<(i64, i64), Option<(i64, i64)>> = town
        .r#where()?
        .of("exits")
        .into_iter()
        .filter(|item| item.zone == Some(TOWN) || item.kind == "door")
        .map(|item| (item.cell, item.arrival))
        .collect();
    assert_eq!(
        arrivals,
        HashMap::from([
            ((94, 171), None),
            ((95, 171), Some((100, 176))),
            ((100, 176), Some((95, 171))),
        ])
    );
    assert!(
        town.listing()?[0].contains("teleporter to zone 0x6 1 east 1 north out at 6 east 4 south")
    );
    Ok(())
}

/// Lysandre Labs (spike 265): the pad at (11, 2) puts the player on (33, 27), a 0x0001 warp that
/// does nothing stepped onto. It is not listed, and the pad is a teleporter P walks through,
/// landing the player as high as the warp it comes out at.
#[test]
fn test_a_pads_landing_is_not_listed_and_the_pad_lands_the_player_as_high_as_it() -> Result<()> {
    let mut town = Town::new();
    let data = zone_file(&ZoneSpec {
        warps: &[
            (HOUSE, WARP_DOOR, 94, 171, None, None),
            (TOWN, 0x0601, 95, 171, Some(0), Some(2)),
            (TOWN, 0x0001, 100, 176, Some(18), Some(1)),
        ],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            warps: xy::warps().clone(),
            landings: HashSet::from(xy::LANDINGS),
            ..game()
        },
    );
    let pads: Vec<(String, (i64, i64), Option<(i64, i64)>, Option<f64>)> = town
        .r#where()?
        .of("exits")
        .into_iter()
        .filter(|item| item.zone == Some(TOWN))
        .map(|item| (item.kind.clone(), item.cell, item.arrival, item.landing))
        .collect();
    assert_eq!(
        pads,
        [(
            "teleporter".to_string(),
            (95, 171),
            Some((100, 176)),
            Some(18.0)
        )]
    );
    Ok(())
}

/// Spike 107: Lumiose's walk-in warps lead into its buildings and onto Centrico Plaza, a zone
/// of its streets that is not its own parent, which only the ROM says is walked as outdoors.
#[test]
fn test_a_warp_off_a_street_onto_another_street_is_not_a_door() -> Result<()> {
    let plaza = 0x45;
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            outdoors: HashSet::from([plaza]),
            ..Given::default()
        },
        game(),
    );
    let data = zone_file(&ZoneSpec {
        warps: &[
            (plaza, 0x0701, 80, 100, None, None),
            (BEDROOM, 0x0702, 16, 9, None, None),
        ],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    let exits = town.listing()?[0].clone();
    assert!(exits.contains("warp to zone 0x45") && exits.contains("door to zone 0xe0"));
    Ok(())
}

/// Spike 107: the matrix can put the player in the next zone a poll before its data is in,
/// and the town's doors were named for the next zone's doors of their numbers. A zone data file
/// holds its own row of the zone table, every row different, so its records are the town's.
#[test]
fn test_a_record_is_named_by_the_zone_whose_table_it_is_in() -> Result<()> {
    let town = Town::new();
    town.reader.track(&town.found())?;
    town.model(0, ROUTE, mmodel::PLAYER_INDEX, 0xAC, (99, 150));
    let r#where = town.reader.poll()?;
    assert!(
        r#where
            .as_ref()
            .is_some_and(|r#where| r#where.zone == ROUTE)
    );
    let r#where = r#where.unwrap();
    let doors: Vec<Interactable> = town
        .reader
        .interactables(&r#where)?
        .of("exits")
        .into_iter()
        .filter(|item| item.kind == "door")
        .collect();
    assert!(!doors.is_empty());
    let named: HashSet<_> = doors.iter().map(|item| item.naming(r#where.zone)).collect();
    let expected: HashSet<_> = doors
        .iter()
        .map(|item| Some(("place", TOWN, item.source)))
        .collect();
    assert_eq!(named, expected);
    Ok(())
}

#[test]
fn test_a_stale_file_is_not_the_current_one() -> Result<()> {
    let town = Town::new();
    let stale = zone_file(&ZoneSpec {
        furniture: &[(0x8, 11, 6)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(OTHER, &stale);
    assert!(town.listing()?[1].contains("sign 3 east 1 north"));
    assert_eq!(town.listing()?[1].split(";").count(), 2);
    Ok(())
}

#[test]
fn test_a_file_whose_size_does_not_match_its_counts_is_not_one() -> Result<()> {
    let town = Town::new();
    town.memory.u32(FILE + zone_header(5) + INFO_SIZE, &[999]);
    town.reader.track(&town.found())?;
    assert!(town.reader.poll()?.is_none());
    Ok(())
}

#[test]
fn test_the_zone_table_says_which_zones_are_outdoors() -> Result<()> {
    let town = Town::new();
    town.reader.track(&town.found())?;
    assert_eq!(town.reader.zones.outdoors(TOWN)?, Some(true));
    assert_eq!(town.reader.zones.outdoors(ROUTE)?, Some(true));
    assert_eq!(town.reader.zones.outdoors(HOUSE)?, Some(false));
    Ok(())
}

/// The lesson's Poochyena, 5 west 1 north of the sneaking spot (spike 80).
#[test]
fn test_a_hidden_pokemon_is_listed_under_its_own_category_by_species() -> Result<()> {
    let town = Town::new();
    town.hide(POOCHYENA, 5, (94, 129));
    town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (99, 130));
    let listing = town.listing()?;
    assert_eq!(
        listing.last().map(String::as_str),
        Some("pokémon: hidden Pokémon species 261 5 west 1 north")
    );
    let hiding = town.r#where()?.of("pokémon");
    assert_eq!(
        hiding
            .iter()
            .map(|item| (item.kind.as_str(), item.cell, item.code))
            .collect::<Vec<_>>(),
        [("hidden Pokémon", (94, 129), POOCHYENA)]
    );
    Ok(())
}

/// A Wurmple on Route 102 held 0x97 in the halfword after its species, and read as a word
/// it was off the Pokédex and went unlisted (hidden_tail_3.ctm, 2026-09-23).
#[test]
fn test_the_species_is_a_halfword_whatever_follows_it() -> Result<()> {
    let town = Town::new();
    town.hide(POOCHYENA, 5, (94, 129));
    town.memory
        .write(HIDDEN + hidden::SPECIES + 2, &0x97u16.to_le_bytes());
    town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (99, 130));
    assert_eq!(
        town.listing()?.last().map(String::as_str),
        Some("pokémon: hidden Pokémon species 261 5 west 1 north")
    );
    Ok(())
}

/// Route 101's Poochyena stayed in the record onto Route 102 (spike 80).
#[test]
fn test_a_record_from_another_zone_is_stale_and_not_listed() -> Result<()> {
    let town = Town::new();
    town.hide(POOCHYENA, 5, (94, 129));
    assert!(town.reader.hidden(TOWN)?.is_none());
    assert!(town.reader.hidden(ROUTE)?.is_none());
    town.reader.track(&town.found())?;
    assert!(town.reader.hidden(TOWN)?.is_none());
    assert!(town.reader.hidden(ROUTE)?.is_some());
    assert!(!town.listing()?.iter().any(|line| line.starts_with("pok")));
    Ok(())
}

/// Route 104's matrix is 10 chunks wide: the Taillow in its grass went unlisted while the
/// grid was read 32 blocks to a row (hidden_tail.ctm, 2026-09-23).
#[test]
fn test_a_matrix_wider_than_32_blocks_is_read_at_its_own_width() -> Result<()> {
    let mut town = Town::new();
    let route_104 = 0x1B;
    town.matrix(10, &[(route_104, 8..16, 8..16), (0x1C, 8..16, 16..24)]);
    town.model(0, route_104, mmodel::PLAYER_INDEX, 0xAC, (137, 126));
    town.hide(278, 3, (140, 119));
    town.reader.track(&town.found())?;
    let r#where = town.reader.poll()?;
    assert!(r#where.is_some_and(|r#where| r#where.zone == route_104));
    assert_eq!(
        town.listing()?.last().map(String::as_str),
        Some("pokémon: hidden Pokémon species 278 3 east 7 north")
    );
    Ok(())
}

#[test]
fn test_an_empty_record_lists_no_pokemon() -> Result<()> {
    assert!(
        !Town::new()
            .listing()?
            .iter()
            .any(|line| line.starts_with("pokémon"))
    );
    Ok(())
}

/// After the Surskit's battle the record still named it at its cell (spike 80, the second
/// movie); only the slot byte said it was gone.
#[test]
fn test_a_record_whose_pokemon_has_gone_lists_none() -> Result<()> {
    let town = Town::new();
    town.hide_with(POOCHYENA, 5, (94, 129), false);
    town.reader.track(&town.found())?;
    assert!(town.reader.hidden(ROUTE)?.is_none());
    Ok(())
}

/// The byte is the DexNav's state: 0xd its page with none hiding, as after a faint, and 0xe a
/// search; 8 a search's find and 0xa near enough for its details (Orion, movie26).
#[test]
fn test_a_pokemon_that_fainted_or_a_search_under_way_lists_none() -> Result<()> {
    let town = Town::new();
    for (state, listed) in [(0x0D, false), (0x0E, false), (0x08, true), (0x0A, true)] {
        town.hide_state(POOCHYENA, 5, (94, 129), state);
        town.reader.track(&town.found())?;
        assert_eq!(town.reader.hidden(ROUTE)?.is_some(), listed, "{state:#x}");
    }
    Ok(())
}

/// A Zigzagoon read at level 50331650, 0x03000002, went unlisted (Orion, movie27); and +0xc9
/// is 3 for a search's find, 1 for the detector's with the DexNav open, 2 with the AreaNav
/// (spike 194).
#[test]
fn test_the_level_is_one_byte_and_how_it_was_found_is_another() -> Result<()> {
    let town = Town::new();
    town.reader.track(&town.found())?;
    for (found_by, searched, on_dexnav) in [(3, true, false), (1, false, true), (2, false, false)] {
        town.hide_found(POOCHYENA, 0x03000002, (94, 129), 0x08, found_by);
        town.reader.track(&town.found())?;
        assert!(town.reader.hidden(ROUTE)?.is_some());
        let hiding = town.reader.hiding();
        assert!(
            hiding
                .is_some_and(|hiding| (hiding.searched, hiding.on_dexnav) == (searched, on_dexnav))
        );
    }
    Ok(())
}

/// What the player's work points at is not the record while it holds no heap first.
#[test]
fn test_the_record_is_told_by_the_heap_it_holds_first() -> Result<()> {
    let town = Town::new();
    town.hide(POOCHYENA, 5, (94, 129));
    town.memory.u32(HIDDEN + hidden::HEAP, &[0]);
    town.reader.track(&town.found())?;
    assert!(town.reader.hidden(ROUTE)?.is_none());
    Ok(())
}

/// The one `field::PlayerWork` holds the record's address at +0x6c in all three captures of
/// movie3 (spike 135), so no page is searched for what holds a CoreParam, and CoreParams, which
/// the net found everywhere after every battle, are not swept for at all.
#[test]
fn test_the_record_is_found_through_the_players_work_and_no_coreparam_is_swept() -> Result<()> {
    assert!(!hidden::CLASSES.contains(&hidden::CORE_PARAM_CLASS));
    let town = Town::new();
    town.hide(POOCHYENA, 5, (94, 129));
    let hidden = DexNav::new(town.memory.clone());
    hidden.track(&town.found())?;
    assert!(hidden.read()?.is_some());
    // The record lives all session: a sweep that finds no way to it keeps it.
    town.memory.u32(WORK + hidden::WORK_RECORD, &[0]);
    hidden.track(&town.found())?;
    assert!(hidden.read()?.is_some());
    Ok(())
}

#[test]
fn test_a_species_or_level_off_the_scale_is_not_a_pokemon() -> Result<()> {
    let town = Town::new();
    town.hide(0x2000, 5, (94, 129));
    town.reader.track(&town.found())?;
    assert!(town.reader.hidden(ROUTE)?.is_none());
    town.hide(POOCHYENA, 200, (94, 129));
    assert!(town.reader.hidden(ROUTE)?.is_none());
    Ok(())
}

/// The edges listed under `exits`, as (cell, size, source).
fn edges(town: &Town) -> Result<Vec<((i64, i64), (i64, i64), i64)>> {
    Ok(town
        .r#where()?
        .of("exits")
        .iter()
        .filter(|item| item.kind == "edge")
        .map(|item| (item.cell, item.size, item.source))
        .collect())
}

/// Littleroot's north side is 40 cells of trees but for three (the user, 2026-09-22).
#[test]
fn test_an_edge_is_the_gap_in_the_border() -> Result<()> {
    assert_eq!(
        edges(&Town::new())?,
        [((99, 159), (3, 1), EDGE_SOURCE | ROUTE << 16)]
    );
    Ok(())
}

#[test]
fn test_a_border_with_no_chunk_loaded_is_listed_whole() -> Result<()> {
    assert_eq!(
        edges(&Town::with(false, 5, 7))?,
        [((80, 159), (40, 1), EDGE_SOURCE | ROUTE << 16)]
    );
    assert!(Town::with(false, 5, 7).listing()?[0].contains("edge to zone 0x17 13 north"));
    Ok(())
}

/// Only the four chunks around the player are loaded, so the far side is often unknown.
#[test]
fn test_a_border_known_on_one_side_is_narrowed_by_that_side() -> Result<()> {
    let mut town = Town::with(false, 5, 7);
    town.block(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &border(160));
    assert_eq!(
        edges(&town)?,
        [((99, 159), (3, 1), EDGE_SOURCE | ROUTE << 16)]
    );
    let mut route = Town::with(false, 5, 7);
    route.block(ROUTE_BLOCK, ROUTE_CHUNK, (80, 120), &border(159));
    assert_eq!(
        edges(&route)?,
        [((99, 159), (3, 1), EDGE_SOURCE | ROUTE << 16)]
    );
    Ok(())
}

/// x_route-8.ctm (spike 234): Route 8's border to Cyllage City was one edge with the chunk north
/// of it loaded and two once that unloaded, and the way P was guiding to the first, through the
/// gap, became a way to the new first, where there is none: "no path to edge". The ROM's chunk
/// narrows the border as the loaded one does, so the edge is the same whatever is loaded.
#[test]
fn test_a_side_no_loaded_chunk_holds_narrows_an_edge_by_the_roms_chunk() -> Result<()> {
    let mut loaded = Town::with(false, 5, 7);
    loaded.block(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &[]);
    loaded.block(ROUTE_BLOCK, ROUTE_CHUNK, (80, 120), &border(159));
    let mut town = Town::with(false, 5, 7);
    town.block(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &[]);
    let mut files = vec![NO_CHUNK_FILE; 64];
    files[3 * 8 + 2] = 8;
    town.memory.write(MAP + MAP_CHUNK_FILES, &le16(&files));
    let route = mapblock::cells(&chunk_file((80, 120), &border(159)))?.unwrap();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            far: Some(Rc::new(move |file| {
                Ok(if file == 8 { Some(route.clone()) } else { None })
            })),
            ..Given::default()
        },
        game(),
    );
    let expected = [((99, 159), (3, 1), EDGE_SOURCE | ROUTE << 16)];
    assert_eq!(edges(&town)?, expected);
    assert_eq!(edges(&loaded)?, expected);
    Ok(())
}

#[test]
fn test_each_gap_in_a_border_is_an_exit_of_its_own() -> Result<()> {
    let mut town = Town::with(false, 5, 7);
    town.block(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border_with(160, &[(85, 87), GAP]),
    );
    town.block(
        ROUTE_BLOCK,
        ROUTE_CHUNK,
        (80, 120),
        &border_with(159, &[(85, 87), GAP]),
    );
    assert_eq!(
        edges(&town)?,
        [
            ((99, 159), (3, 1), EDGE_SOURCE | ROUTE << 16 | 1),
            ((85, 159), (2, 1), EDGE_SOURCE | ROUTE << 16),
        ]
    );
    let keys: HashSet<_> = town
        .r#where()?
        .of("exits")
        .iter()
        .map(|item| item.key())
        .collect();
    assert_eq!(keys.len(), 4);
    Ok(())
}

#[test]
fn test_a_gap_open_on_one_side_only_is_not_a_way_across() -> Result<()> {
    let mut town = Town::with(false, 5, 7);
    town.block(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border_with(160, &[(85, 87), GAP]),
    );
    town.block(ROUTE_BLOCK, ROUTE_CHUNK, (80, 120), &border(159));
    assert_eq!(
        edges(&town)?,
        [((99, 159), (3, 1), EDGE_SOURCE | ROUTE << 16)]
    );
    Ok(())
}

/// The blocks are built after the rest of the field, so the first listing after a state
/// load had none and said the whole border (the replays of 2026-09-22).
#[test]
fn test_a_zone_listed_without_any_chunk_asks_for_one_sweep() -> Result<()> {
    let mut town = Town::with(false, 5, 7);
    assert!(town.reader.sweep_wanted(true)?.is_none());
    town.r#where()?;
    assert!(town.reader.sweep_wanted(true)?.is_some());
    assert!(town.reader.sweep_wanted(true)?.is_none());
    // The sweep asked for found none either: the next listing asks again.
    town.r#where()?;
    assert!(town.reader.sweep_wanted(true)?.is_some());
    assert!(town.reader.sweep_wanted(true)?.is_none());
    // Once a sweep finds the blocks, the asking stops.
    town.block(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &border(160));
    town.r#where()?;
    assert!(town.reader.sweep_wanted(true)?.is_none());
    assert!(Town::new().reader.sweep_wanted(true)?.is_none());
    let with_chunks = Town::new();
    with_chunks.r#where()?;
    assert!(with_chunks.reader.sweep_wanted(true)?.is_none());
    Ok(())
}

/// A matrix whose blocks no sweep can find must not keep the runtime sweeping every tick:
/// the replays of 2026-09-23 swept three times a second until the field was left.
#[test]
fn test_a_zone_stops_asking_for_sweeps_after_a_few_and_asks_afresh_in_another() -> Result<()> {
    let town = Town::with(false, 5, 7);
    for _ in 0..ASKS {
        town.r#where()?;
        assert!(town.reader.sweep_wanted(true)?.is_some());
    }
    town.r#where()?;
    assert!(town.reader.sweep_wanted(true)?.is_none());
    // Another zone's listing starts its own count: the player's block is Route 101's now.
    town.memory
        .u16(MAP + MAP_GRID + (17 * town.width + 9) * 2, ROUTE);
    town.r#where()?;
    assert!(town.reader.sweep_wanted(true)?.is_some());
    Ok(())
}

/// A battle tears the map blocks down and the field goes on listing: every one of the 72
/// asks over the recordings of ORAS and X came while another app ran (spike 130). The want is
/// spent, so the field coming back does not sweep for it either.
#[test]
fn test_a_zone_listed_while_another_app_runs_asks_for_no_sweep() -> Result<()> {
    let town = Town::with(false, 5, 7);
    town.r#where()?;
    assert!(town.reader.sweep_wanted(false)?.is_none());
    town.r#where()?;
    assert!(town.reader.sweep_wanted(true)?.is_none());
    // Another zone listed without a chunk on the field asks afresh.
    town.memory
        .u16(MAP + MAP_GRID + (17 * town.width + 9) * 2, ROUTE);
    town.r#where()?;
    assert!(town.reader.sweep_wanted(true)?.is_some());
    Ok(())
}

/// The front of the player's house, its door at (94, 171) blocked as spike 82 found it.
fn house_front() -> Vec<(i64, i64)> {
    (90..99)
        .flat_map(|x| (167..172).map(move |z| (x, z)))
        .collect()
}

/// The user, 2026-09-24: a path to something solid ends on an open cell beside it.
#[test]
fn test_a_solid_thing_is_walked_up_to_facing_it_and_an_open_one_onto() -> Result<()> {
    let mut town = Town::new();
    town.block(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &[border(160), house_front()].concat(),
    );
    let r#where = town.r#where()?;
    let ground = town.reader.ground(&r#where)?;
    let door = r#where
        .of("exits")
        .into_iter()
        .find(|item| item.zone == Some(HOUSE))
        .unwrap();
    let edge = r#where
        .of("exits")
        .into_iter()
        .find(|item| item.kind == "edge")
        .unwrap();
    assert_eq!(
        ground.goals(&door)?,
        goals_of(&[((94, 172), Some("north"))])
    );
    assert_eq!(
        ground.goals(&edge)?,
        goals_of(&[((99, 159), None), ((100, 159), None), ((101, 159), None)])
    );
    let person = r#where.of("people").into_iter().next().unwrap();
    assert_eq!(
        ground.goals(&person)?,
        goals_of(&[
            ((109, 175), Some("south")),
            ((110, 176), Some("west")),
            ((109, 177), Some("north")),
            ((108, 176), Some("east")),
        ])
    );
    // A hidden item lies on open ground and is found facing it, as a person is talked to.
    let hidden = Interactable {
        category: "items".to_string(),
        kind: "hidden item".to_string(),
        cell: (105, 165),
        size: (1, 1),
        faced: true,
        ..edge.clone()
    };
    assert_eq!(
        ground.goals(&hidden)?,
        goals_of(&[
            ((105, 164), Some("south")),
            ((106, 165), Some("west")),
            ((105, 166), Some("north")),
            ((104, 165), Some("east")),
        ])
    );
    Ok(())
}

#[test]
fn test_a_warp_is_walked_round_unless_it_is_where_the_path_goes() -> Result<()> {
    let town = Town::new();
    let r#where = town.r#where()?;
    let ground = town.reader.ground(&r#where)?;
    let other_door = r#where
        .of("exits")
        .into_iter()
        .find(|item| item.zone == Some(0xE1))
        .unwrap();
    assert!(ground.open((106, 171))? && !ground.passable((106, 171))?);
    let goals = ground.goals(&other_door)?;
    assert_eq!(goals, goals_of(&[((106, 171), None)]));
    let path = pathfind::find(
        r#where.cell,
        &goals,
        &|cell| ground.passable(cell),
        ground.r#box(r#where.cell, goals.keys().copied())?,
        &pathfind::Options::default(),
    )?;
    assert!(path.is_some_and(|path| path[path.len() - 1].cell == (106, 171)));
    Ok(())
}

/// Dewford Gym (spike 132): a belt is a trigger over plain floor, and walking north onto one
/// carries the player back. A trigger the game knows as a belt is walked only the way it runs;
/// any other trigger is floor, as it was.
#[test]
fn test_a_conveyor_belt_is_ridden_the_way_it_runs_and_walked_round_against_it() -> Result<()> {
    let mut town = Town::new();
    town.memory.write(
        FILE,
        &zone_file(&ZoneSpec {
            info: &town_info(),
            triggers: &[(0x9, 90, 175, 3, None, None), (0xA, 90, 178, 3, None, None)],
            ..ZoneSpec::default()
        }),
    );
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            belts: IndexMap::from([((TOWN, 0x9), pathfind::SOUTH)]),
            ..game()
        },
    );
    let ground = town.reader.ground(&town.r#where()?)?;
    assert!(ground.open((91, 175))? && ground.ledge((91, 175))? == Some(pathfind::SOUTH));
    assert_eq!(ground.ledge((91, 178))?, None);
    let r#box = ground.r#box((91, 174), [(91, 176)])?;
    let ledge = |cell| ground.ledge(cell);
    let walk = pathfind::Options {
        ledge: Some(&ledge),
        ..pathfind::Options::default()
    };
    let down = pathfind::find(
        (91, 174),
        &goals_of(&[((91, 176), None)]),
        &|cell| ground.passable(cell),
        r#box,
        &walk,
    )?
    .expect("a way down");
    assert_eq!(
        pathfind::spoken(&pathfind::legs(&pathfind::cells(&down), None)?),
        "2 south"
    );
    let up = pathfind::find(
        (91, 176),
        &goals_of(&[((91, 174), None)]),
        &|cell| ground.passable(cell),
        r#box,
        &walk,
    )?
    .expect("a way up");
    for cell in pathfind::cells(&up) {
        assert!(ground.ledge(cell)?.is_none());
    }
    Ok(())
}

/// Dewford Gym's switches and belts were in no list (a user, 2026-10-26). Listed, P's way down
/// its corridor still walks over both, as before, where another listed trigger is stepped
/// round: its switches only light the side passages (spike 132).
#[test]
fn test_dewford_gyms_switches_and_belts_are_listed_and_walked_over() -> Result<()> {
    let mut town = Town::new();
    town.memory.write(
        FILE,
        &zone_file(&ZoneSpec {
            info: &town_info(),
            triggers: &[(0x9, 90, 175, 3, None, None), (0x5, 91, 177, 1, None, None)],
            ..ZoneSpec::default()
        }),
    );
    let owned = |(category, kind): (&str, &str)| (category.to_string(), kind.to_string(), None);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            belts: IndexMap::from([((TOWN, 0x9), pathfind::SOUTH)]),
            triggers: IndexMap::from([
                ((TOWN, 0x9), owned(oras::BELT)),
                ((TOWN, 0x5), owned(oras::SWITCH)),
            ]),
            crossed: HashSet::from([(TOWN, 0x5)]),
            ..game()
        },
    );
    let r#where = town.r#where()?;
    let mut kinds: Vec<String> = r#where
        .of("signs")
        .iter()
        .filter(|item| item.code == 0x5 || item.code == 0x9)
        .map(|item| item.kind.clone())
        .collect();
    kinds.sort();
    assert_eq!(kinds, ["conveyor belt", "switch"]);
    let ground = town.reader.ground(&r#where)?;
    let r#box = ground.r#box((91, 174), [(91, 179)])?;
    let ledge = |cell| ground.ledge(cell);
    let down = pathfind::find(
        (91, 174),
        &goals_of(&[((91, 179), None)]),
        &|cell| ground.passable(cell),
        r#box,
        &pathfind::Options {
            ledge: Some(&ledge),
            ..pathfind::Options::default()
        },
    )?;
    let down = down.expect("a way down");
    assert_eq!(
        pathfind::spoken(&pathfind::legs(&pathfind::cells(&down), None)?),
        "5 south"
    );
    Ok(())
}

/// Mauville Gym (spike 146): a beam between two posts is a move model at its middle, which the
/// list called a person. Facing south the beam runs east and west; a switch turns it to face east
/// and run north and south, and it stands in all three of its cells either way.
#[test]
fn test_a_barrier_is_listed_as_one_and_stands_across_the_way_its_model_faces() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            barriers: HashSet::from([(TOWN, 0x138)]),
            ..game()
        },
    );
    let across = town.model_sized(4, TOWN, 3, 0x138, (100, 175), (3, 1));
    let down = town.model_sized(5, TOWN, 4, 0x138, (104, 175), (3, 1));
    town.memory
        .write(across + mmodel::FACING_SIN + 8, &f32s(&[1.0]));
    town.memory.write(down + mmodel::FACING_SIN, &f32s(&[1.0]));
    let r#where = town.r#where()?;
    let mut barriers: Vec<((i64, i64), (i64, i64))> = r#where
        .of("obstacles")
        .iter()
        .filter(|item| item.kind == "barrier")
        .map(|item| (item.cell, item.size))
        .collect();
    barriers.sort();
    assert_eq!(barriers, [((99, 175), (3, 1)), ((104, 174), (1, 3))]);
    assert!(
        !r#where
            .of("people")
            .iter()
            .any(|item| item.kind == "barrier")
    );
    let ground = town.reader.ground(&r#where)?;
    for cell in [(99, 175), (101, 175), (104, 174), (104, 176)] {
        assert!(!ground.open(cell)?);
    }
    Ok(())
}

/// Fiery Path's boulder (omega_ruby_strength-boulders.ctm): its model is in one cell, but the
/// player faced it from two, and its model says it is two by two, running on east and south.
#[test]
fn test_a_big_boulder_stands_in_four_cells_and_strength_clears_them_all() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            clears: oras::game().clears.clone(),
            ..game()
        },
    );
    town.model_sized(4, TOWN, 3, oras::BIG_BOULDER, (100, 175), (2, 2));
    let r#where = town.r#where()?;
    let boulder = r#where
        .of("obstacles")
        .into_iter()
        .find(|item| item.code == oras::BIG_BOULDER)
        .unwrap();
    assert_eq!((boulder.cell, boulder.size), ((100, 175), (2, 2)));
    let ground = town.reader.ground(&r#where)?;
    let cells = [(100, 175), (101, 175), (100, 176), (101, 176)];
    for cell in cells {
        assert!(!ground.open(cell)?);
    }
    let strong = Ground {
        hm: true,
        ..ground.clone()
    };
    for cell in cells {
        assert!(strong.open(cell)?);
    }
    Ok(())
}

/// movie37 (spike 202): every boulder of room 0x69 kept its record however far it was pushed,
/// and the record kept the room's own cell, so the list can tell one boulder from the next and
/// say how far each has been pushed.
#[test]
fn test_a_big_boulder_keeps_its_record_and_where_the_record_starts_it() -> Result<()> {
    let mut town = Town::new();
    let data = zone_file(&ZoneSpec {
        info: &town_info(),
        npcs: &[
            (0x119, 91, 175, 0x227, None),
            (mmodel::ITEM_BALL, 100, 180, 0x51A, None),
            (PROP_MODEL + 0x4E, 110, 170, 0x205, None),
            (oras::BIG_BOULDER, 98, 175, 0, None),
        ],
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            clears: oras::game().clears.clone(),
            ..game()
        },
    );
    town.model_sized(4, TOWN, 3, oras::BIG_BOULDER, (100, 175), (2, 2));
    let r#where = town.r#where()?;
    let boulder = r#where
        .of("obstacles")
        .into_iter()
        .find(|item| item.code == oras::BIG_BOULDER)
        .unwrap();
    let person = r#where.of("people").into_iter().next().unwrap();
    assert_eq!(
        (boulder.record, boulder.start),
        (Some((TOWN, 3)), Some((98, 175)))
    );
    assert_eq!(
        (person.record, person.start, person.pushes),
        (Some((TOWN, 0)), None, None)
    );
    assert_eq!(
        boulder.pushes,
        Some(vec![
            ("north", false),
            ("east", false),
            ("south", false),
            ("west", false)
        ])
    );
    Ok(())
}

/// Seafloor Cavern's eight-boulder room from x 8 and z 17, as the ROM's chunks have it (spike 202):
/// # a wall, H a hole.
const ROOM_0X69: [&str; 14] = [
    "###########...##########",
    "########HH.....HH#######",
    "########HH.....HH#######",
    "########..#####..#######",
    "########.........#######",
    "########.#.....#.#######",
    "########...#.#...#######",
    "#########.......########",
    "########.........#######",
    "#######...........######",
    "#######..#.#.#.#..######",
    "########HH.....HH#######",
    "########HH.....HH#######",
    "########..#...#..#######",
];

/// Pushes movie37 made, the stick lined up with the replay (spike 202): record 0 went half onto
/// the north-west hole at 246 s and dropped in at the next push, and against the wall west of it
/// it stayed; record 6 stayed against record 7, south of it, three times.
#[test]
fn test_a_big_boulder_moves_where_every_cell_ahead_is_floor_or_a_hole() {
    let mut cells: HashMap<(i64, i64), u8> = HashMap::new();
    for (z, row) in ROOM_0X69.iter().enumerate() {
        for (x, kind) in row.bytes().enumerate() {
            cells.insert((x as i64 + 8, z as i64 + 17), kind);
        }
    }
    let holes: HashSet<(i64, i64)> = cells
        .iter()
        .filter(|&(_, &kind)| kind == b'H')
        .map(|(&cell, _)| cell)
        .collect();

    fn square(x: i64, z: i64) -> [(i64, i64); 4] {
        [(x, z), (x + 1, z), (x, z + 1), (x + 1, z + 1)]
    }

    let ways = |boulder: (i64, i64), others: &[(i64, i64)]| -> Vec<(&'static str, bool)> {
        let standing: HashSet<(i64, i64)> =
            others.iter().flat_map(|&(x, z)| square(x, z)).collect();
        pushes(
            square(boulder.0, boulder.1),
            &|cell| Ok(cells.get(&cell) == Some(&b'.') && !standing.contains(&cell)),
            &holes,
        )
        .unwrap()
    };

    assert_eq!(ways((16, 20), &[]), [("north", false)]);
    assert_eq!(ways((16, 19), &[]), [("north", true)]);
    assert_eq!(ways((22, 23), &[(23, 25)]), []);
    assert_eq!(ways((22, 23), &[]), [("south", false)]);
}

/// Seafloor Cavern (movie36, spike 202): a hole is four cells of kind 0x30, blocked; a boulder
/// pushed into one stays a model on its four cells, which stay blocked in the chunk, and the
/// player walked over it while the list said "big boulder, here". Guiding asks for the ground
/// with nothing listed, and is told the same. A hole in a corner, walls north and west of it as
/// room 0x69's north-west one has, is no place the game parks a model (movie37).
#[test]
fn test_a_hole_is_listed_until_a_big_boulder_fills_it_and_then_both_are_floor() -> Result<()> {
    let hole = 0x30020021;
    let empty = [(100, 175), (101, 175), (100, 176), (101, 176)];
    let full = [(104, 175), (105, 175), (104, 176), (105, 176)];
    let corner = [(103, 174), (104, 174), (105, 174), (103, 175), (103, 176)];
    let mut town = Town::new();
    let mut blocked = border(160);
    blocked.extend(corner);
    let tiles: Vec<((i64, i64), i64)> = empty
        .iter()
        .chain(&full)
        .map(|&cell| (cell, hole))
        .collect();
    town.block_tiles(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &blocked, &tiles);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            clears: oras::game().clears.clone(),
            ..game()
        },
    );
    town.model_sized(4, TOWN, 3, oras::BIG_BOULDER, (104, 175), (2, 2));
    town.model_sized(5, TOWN, 4, oras::BIG_BOULDER, (100, 179), (2, 2));
    let r#where = town.r#where()?;
    let listed: Vec<(String, (i64, i64), (i64, i64))> = r#where
        .of("obstacles")
        .into_iter()
        .filter(|item| item.kind == "hole" || item.code == oras::BIG_BOULDER)
        .map(|item| (item.kind.clone(), item.cell, item.size))
        .collect();
    assert_eq!(
        listed,
        [
            ("hole".to_string(), (100, 175), (2, 2)),
            ("object".to_string(), (100, 179), (2, 2))
        ]
    );
    let bare = Field {
        interactables: Vec::new(),
        ..r#where.clone()
    };
    for ground in [town.reader.ground(&r#where)?, town.reader.ground(&bare)?] {
        for cell in full {
            assert!(ground.open(cell)?);
        }
        for cell in empty {
            assert!(!ground.open(cell)?);
        }
    }
    Ok(())
}

/// Mauville Gym (spike 146): its switches are triggers the list never had, and stepping on one
/// turns barriers, so a way elsewhere keeps off them. A passage joins two rooms with no floor
/// between them; a trigger the game has not said is either is left out, as before.
#[test]
fn test_a_gyms_switches_and_passages_are_listed_and_walked_round_unless_the_goal() -> Result<()> {
    let mut town = Town::new();
    town.memory.write(
        FILE,
        &zone_file(&ZoneSpec {
            info: &town_info(),
            triggers: &[
                (0x4, 91, 175, 1, None, None),
                (0x9, 95, 178, 3, None, None),
                (0xB, 90, 180, 1, None, None),
            ],
            ..ZoneSpec::default()
        }),
    );
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            triggers: IndexMap::from([
                (
                    (TOWN, 0x4),
                    ("signs".to_string(), "switch".to_string(), None),
                ),
                (
                    (TOWN, 0x9),
                    ("exits".to_string(), "passage".to_string(), None),
                ),
            ]),
            ..game()
        },
    );
    let r#where = town.r#where()?;
    let switch = r#where
        .of("signs")
        .into_iter()
        .find(|item| item.kind == "switch")
        .unwrap();
    let passage = r#where
        .of("exits")
        .into_iter()
        .find(|item| item.kind == "passage")
        .unwrap();
    assert_eq!(
        (switch.cell, switch.size, passage.cell, passage.size),
        ((91, 175), (1, 1), (95, 178), (3, 1))
    );
    let codes: Vec<i64> = r#where
        .interactables
        .iter()
        .filter(|item| item.kind == "switch" || item.kind == "passage")
        .map(|item| item.code)
        .collect();
    assert_eq!(codes, [0x4, 0x9]);
    let ground = town.reader.ground(&r#where)?;
    assert!(ground.open((91, 175))? && !ground.passable((91, 175))?);
    let goals = ground.goals(&switch)?;
    let path = pathfind::find(
        (91, 172),
        &goals,
        &|cell| ground.passable(cell),
        ground.r#box((91, 172), goals.keys().copied())?,
        &pathfind::Options::default(),
    )?;
    assert!(path.is_some_and(|path| path[path.len() - 1].cell == (91, 175)));
    Ok(())
}

/// Spike 252: each of Prism Tower's quiz rooms keeps a record of three by three in the walls of
/// the map's corner at (0, 0), listed as a sign 24 north; a sign in a wall beside the floor
/// stays.
#[test]
fn test_furniture_with_nothing_beside_it_to_stand_on_is_not_listed() -> Result<()> {
    let mut town = Town::new();
    let walls: Vec<(i64, i64)> = (0..5).flat_map(|x| (0..5).map(move |z| (x, z))).collect();
    town.block(ROUTE_BLOCK, ROUTE_CHUNK, (0, 0), &walls);
    let data = zone_file(&ZoneSpec {
        sized: &[(0x1, 0, 0, 3, 3), (0x2, 4, 4, 1, 1)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    let codes: Vec<i64> = town
        .r#where()?
        .of("signs")
        .iter()
        .map(|item| item.code)
        .filter(|&code| code == 0x1 || code == 0x2)
        .collect();
    assert_eq!(codes, [0x2]);
    Ok(())
}

/// Spike 164: the way into Seafloor Cavern is a trigger under Route 128, seven cells wide, with no
/// warp record behind it: listed as an exit leading to the zone its script warps to.
#[test]
fn test_a_trigger_into_another_place_is_an_exit_leading_there() -> Result<()> {
    let mut town = Town::new();
    town.memory.write(
        FILE,
        &zone_file(&ZoneSpec {
            info: &town_info(),
            triggers: &[(0x5, 90, 175, 7, None, None)],
            ..ZoneSpec::default()
        }),
    );
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            triggers: IndexMap::from([(
                (TOWN, 0x5),
                ("exits".to_string(), "entrance".to_string(), Some(0x62)),
            )]),
            ..game()
        },
    );
    let r#where = town.r#where()?;
    let entrance = r#where
        .of("exits")
        .into_iter()
        .find(|item| item.kind == "entrance")
        .unwrap();
    assert_eq!(
        (entrance.cell, entrance.size, entrance.zone),
        ((90, 175), (7, 1), Some(0x62))
    );
    Ok(())
}

/// Sea Mauville's sunken control room: the Scanner is a trigger armed while work 0x400c holds
/// 1, which taking it sets to 0. Listed as a hidden item where the Extra lists them, under the
/// sea though the zone table does not say so, until it is taken; with the Extra off, never.
#[test]
fn test_an_item_a_trigger_gives_is_a_hidden_item_while_the_trigger_is_armed() -> Result<()> {
    let mut town = Town::new();
    town.memory.write(
        FILE,
        &zone_file(&ZoneSpec {
            info: &town_info(),
            triggers: &[(0x2, 90, 175, 1, Some(-4), Some((1, 0x400C)))],
            ..ZoneSpec::default()
        }),
    );
    let work = EVENT_WORK + WORKS + (0x400C - FIRST_WORK) * 2;
    town.memory.write(work, &le16(&[1]));

    let listed = |town: &mut Town, setting: &'static str| -> Result<Vec<(i64, i64)>> {
        town.reader = field_reader(
            town.memory.clone(),
            Given {
                extras_hidden_items: Some(setting),
                ..Given::default()
            },
            Game {
                hidden_triggers: HashSet::from([(TOWN, 0x2)]),
                ..game()
            },
        );
        town.reader.track(&Found {
            pages: IndexMap::new(),
            heap: IndexMap::from([(EVENT_WORK, EVENT_CLASS.to_string())]),
            partial: false,
        })?;
        Ok(town
            .r#where()?
            .of("items")
            .into_iter()
            .filter(|item| item.kind == HIDDEN_ITEM)
            .map(|item| item.cell)
            .collect())
    };

    assert_eq!(listed(&mut town, HIDDEN_OFF)?, Vec::new());
    assert_eq!(listed(&mut town, HIDDEN_UNDERWATER)?, vec![(90, 175)]);
    town.memory.write(work, &le16(&[0]));
    assert_eq!(listed(&mut town, HIDDEN_UNDERWATER)?, Vec::new());
    Ok(())
}

/// Laverre Gym's panels (spike 258): each trigger the ROM says puts the player somewhere in the
/// zone is listed under signs as a teleporter, with where and how high; the one by the entrance,
/// armed by work 0x40f3 at 1, only while the work holds it, the Extra off as it ships.
#[test]
fn test_a_teleporter_is_a_sign_saying_where_it_puts_the_player_while_it_is_armed() -> Result<()> {
    let mut town = Town::new();
    town.memory.write(
        FILE,
        &zone_file(&ZoneSpec {
            info: &town_info(),
            triggers: &[
                (0x3, 95, 175, 1, Some(90), None),
                (0x4, 96, 175, 1, Some(180), Some((1, 0x40F3))),
            ],
            ..ZoneSpec::default()
        }),
    );
    let work = EVENT_WORK + WORKS + (0x40F3 - FIRST_WORK) * 2;
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            teleports: IndexMap::from([((TOWN, 0x3), (100, 0, 176)), ((TOWN, 0x4), (90, 0, 170))]),
            ..Given::default()
        },
        game(),
    );
    town.reader.track(&Found {
        pages: IndexMap::new(),
        heap: IndexMap::from([(EVENT_WORK, EVENT_CLASS.to_string())]),
        partial: false,
    })?;

    #[allow(clippy::type_complexity)]
    let teleporters = |town: &mut Town| -> Result<
        Vec<((i64, i64), Option<f64>, Option<(i64, i64)>, Option<f64>)>,
    > {
        Ok(town
            .r#where()?
            .of("signs")
            .into_iter()
            .filter(|item| item.kind == "teleporter")
            .map(|item| (item.cell, item.height, item.arrival, item.landing))
            .collect())
    };

    assert_eq!(
        teleporters(&mut town)?,
        vec![((95, 175), Some(90.0), Some((100, 176)), Some(0.0))]
    );
    town.memory.write(work, &le16(&[1]));
    let cells: Vec<(i64, i64)> = teleporters(&mut town)?
        .into_iter()
        .map(|(cell, ..)| cell)
        .collect();
    assert_eq!(cells, vec![(95, 175), (96, 175)]);
    Ok(())
}

/// Laverre Gym's panel at (29, 1) on its middle floor lies under the cells of a bureau's
/// record, which has no floor, and the listing left it out as a sign the record covers.
#[test]
fn test_a_teleporter_under_a_furniture_records_cells_is_still_listed() -> Result<()> {
    let mut town = Town::new();
    town.memory.write(
        FILE,
        &zone_file(&ZoneSpec {
            info: &town_info(),
            furniture: &[(1, 95, 175)],
            triggers: &[(0x3, 95, 175, 1, Some(90), None)],
            ..ZoneSpec::default()
        }),
    );
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            teleports: IndexMap::from([((TOWN, 0x3), (100, 0, 176))]),
            ..Given::default()
        },
        game(),
    );
    let mut kinds: Vec<String> = town
        .r#where()?
        .of("signs")
        .into_iter()
        .filter(|item| item.cell == (95, 175))
        .map(|item| item.kind.clone())
        .collect();
    kinds.sort();
    assert_eq!(kinds, vec!["sign", "teleporter"]);
    Ok(())
}

/// Spike 224: the pole in Santalune Gym's lobby is a furniture record whose script warps the
/// player to the web room below: listed as an exit leading there, while the statue stays a
/// sign.
#[test]
fn test_furniture_that_leads_to_another_zone_is_an_exit_while_the_rest_stays_a_sign() -> Result<()>
{
    let mut town = Town::new();
    let data = zone_file(&ZoneSpec {
        furniture: &[(0x1, 90, 160), (0x5, 92, 165)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER, &[FILE, OTHER, 0x11 | (data.len() as i64) << 8]);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            furniture: IndexMap::from([(
                (TOWN, 0x1),
                FurnitureKind::Exit("exits".to_string(), "warp".to_string(), 0x38),
            )]),
            ..game()
        },
    );
    let r#where = town.r#where()?;
    let exits: Vec<(String, (i64, i64), Option<i64>)> = r#where
        .of("exits")
        .into_iter()
        .filter(|item| item.place_kind() == Some(FURNITURE_SOURCE))
        .map(|item| (item.kind.clone(), item.cell, item.zone))
        .collect();
    assert_eq!(exits, [("warp".to_string(), (90, 160), Some(0x38))]);
    let signs: Vec<(String, (i64, i64))> = r#where
        .of("signs")
        .into_iter()
        .filter(|item| item.place_kind() == Some(FURNITURE_SOURCE))
        .map(|item| (item.kind.clone(), item.cell))
        .collect();
    assert_eq!(signs, [("sign".to_string(), (92, 165))]);
    Ok(())
}

/// X's house (x_house-map.ctm): the bedroom and the ground floor are one zone side by side
/// on one chunk, no floor between them, and the stairs are triggers the game moves the player
/// through. In the bedroom the list held Mom and the way out below, and P found no way to
/// them. Only the player's room is listed, the stairs down among it.
#[test]
fn test_a_zone_of_rooms_lists_only_the_room_the_player_stands_in() -> Result<()> {
    let mut town = Town::new();
    let mut blocked = border(160);
    blocked.extend((80..120).map(|x| (x, 174)));
    town.block(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &blocked);
    town.memory.write(
        FILE,
        &zone_file(&ZoneSpec {
            info: &town_info(),
            triggers: &[(0x6, 96, 173, 1, None, None), (0x5, 96, 175, 1, None, None)],
            ..ZoneSpec::default()
        }),
    );
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            rooms: HashSet::from([TOWN]),
            triggers: IndexMap::from([
                (
                    (TOWN, 0x6),
                    ("exits".to_string(), "stairs down".to_string(), None),
                ),
                (
                    (TOWN, 0x5),
                    ("exits".to_string(), "stairs up".to_string(), None),
                ),
            ]),
            ..game()
        },
    );
    let mut listed: Vec<(String, (i64, i64))> = town
        .r#where()?
        .interactables
        .iter()
        .filter(|item| item.kind != "edge")
        .map(|item| (item.kind.clone(), item.cell))
        .collect();
    listed.sort();
    assert_eq!(
        listed,
        [
            ("object".to_string(), (110, 170)),
            ("stairs down".to_string(), (96, 173))
        ]
    );
    Ok(())
}

#[test]
fn test_the_ground_ends_with_the_matrix_and_is_open_where_no_chunk_is_loaded() -> Result<()> {
    let town = Town::new();
    let ground = town.reader.ground(&town.r#where()?)?;
    assert!(!ground.open((79, 172))?);
    assert!(!ground.open((80, 160))?);
    let unloaded = Town::with(false, 5, 7);
    assert!(
        unloaded
            .reader
            .ground(&unloaded.r#where()?)?
            .open((80, 160))?
    );
    Ok(())
}

/// Petalburg's Pokémon Center (poke_center_pc.ctm): the Nurse at (19, 15) has the counter in
/// front of her and walls about her, and P said "no path to Nurse". The game talks across one
/// counter cell, so the way ends the far side of it, facing her; a wall is not talked across.
#[test]
fn test_a_person_behind_a_counter_is_talked_to_across_it() -> Result<()> {
    let counter = COUNTER << KIND_SHIFT | BLOCKED;
    let walls: HashMap<(i64, i64), i64> = HashMap::from([
        ((18, 15), BLOCKED),
        ((20, 15), BLOCKED),
        ((19, 14), BLOCKED),
        ((19, 16), counter),
        ((18, 16), counter),
        ((20, 16), counter),
    ]);
    let words: Vec<i64> = (0..CHUNK * CHUNK)
        .map(|i| walls.get(&(i % CHUNK, i / CHUNK)).copied().unwrap_or(0))
        .collect();
    let nurse = Interactable {
        move_model: true,
        ..Interactable::new("people", "person", (19, 15))
    };
    let ground = Ground::of(
        &Field {
            interactables: vec![nurse.clone()],
            ..Field::new(0x103, (19, 19))
        },
        Of {
            chunks: vec![Rc::new(Chunk::new((0, 0), words))],
            ..Of::default()
        },
    )?;
    assert_eq!(
        ground.goals(&nurse)?,
        goals_of(&[((19, 17), Some(pathfind::NORTH))])
    );
    Ok(())
}

/// Spike 223: Santalune Gym's floor is a web whose open cells touch at corners, both
/// neighbours walls, and the player walked them; they carry bit 3 of the cell word. A corner
/// without the bit is a wall's corner.
#[test]
fn test_two_open_cells_touching_at_a_corner_are_joined_only_by_a_strand() -> Result<()> {
    let (strand, wall) = (0x6828, 0x01006821);
    let ground = |bit: i64, strands: i64| -> Ground {
        let open: HashMap<(i64, i64), i64> = HashMap::from([((5, 5), bit), ((6, 6), bit)]);
        let words: Vec<i64> = (0..CHUNK * CHUNK)
            .map(|i| open.get(&(i % CHUNK, i / CHUNK)).copied().unwrap_or(wall))
            .collect();
        Ground {
            chunks: vec![Rc::new(Chunk::new((0, 0), words))],
            strands,
            ..Ground::default()
        }
    };
    let goals = goals_of(&[((6, 6), None)]);
    let way = ground(strand, 0x8).find((5, 5), &goals, "", None, false, None)?;
    assert_eq!(
        way.map(|way| pathfind::cells(&way)),
        Some(vec![(5, 5), (6, 6)])
    );
    assert_eq!(
        ground(strand & !0x8, 0x8).find((5, 5), &goals, "", None, false, None)?,
        None
    );
    assert_eq!(
        ground(strand, 0).find((5, 5), &goals, "", None, false, None)?,
        None
    );
    Ok(())
}

/// Spike 136: P's way to a Fisherman across the sea walked where the sea was not loaded. The
/// field map lists each chunk's file, and the ROM's file there is the chunk the game loads.
#[test]
fn test_the_ground_where_no_chunk_is_loaded_is_the_roms_chunk_the_field_map_lists() -> Result<()> {
    let mut town = Town::with(false, 5, 7);
    let mut files = vec![NO_CHUNK_FILE; 64];
    files[4 * 8 + 2] = 7;
    town.memory.write(MAP + MAP_CHUNK_FILES, &le16(&files));
    let town_chunk = mapblock::cells(&chunk_file((80, 160), &border(160)))?.unwrap();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            far: Some(Rc::new(move |file| {
                Ok(if file == 7 {
                    Some(town_chunk.clone())
                } else {
                    None
                })
            })),
            ..Given::default()
        },
        game(),
    );
    let ground = town.reader.ground(&town.r#where()?)?;
    assert!(!ground.open((80, 160))? && ground.open((99, 160))?);
    assert!(
        ground.open((80, 159))?,
        "Route 101's chunk is listed as none, and is not known"
    );
    Ok(())
}

/// Spike 160: Fortree City's platforms stand 108 over the open ground beside them in one cell
/// table, and P walked the player off one, "9 south" into nothing. A ladder is two one-cell
/// triggers, its top at the platform's height and its foot at the ground's.
#[test]
fn test_a_drop_is_not_walked_off_but_by_its_ladder() -> Result<()> {
    let mut town = Town::with(false, 5, 7);
    let mut files = vec![NO_CHUNK_FILE; 64];
    files[4 * 8 + 2] = 7;
    town.memory.write(MAP + MAP_CHUNK_FILES, &le16(&files));
    let words = mapblock::cells(&chunk_file((80, 160), &border(160)))?.unwrap();
    let heights: Vec<Option<f64>> = (0..CHUNK * CHUNK)
        .map(|i| Some(if i / CHUNK == 10 { 108.0 } else { 0.0 }))
        .collect();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            far: Some(Rc::new(move |file| {
                Ok(if file == 7 { Some(words.clone()) } else { None })
            })),
            far_heights: Some(Rc::new(move |file| {
                Ok(if file == 7 {
                    Some(heights.clone())
                } else {
                    None
                })
            })),
            ..Given::default()
        },
        game(),
    );
    let data = zone_file(&ZoneSpec {
        triggers: &[
            (0xC, 95, 170, 1, Some(108), None),
            (0xB, 95, 171, 1, Some(0), None),
        ],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    let ground = town.reader.ground(&town.r#where()?)?;
    assert!(!ground.joined((94, 170), (94, 171)) && !ground.joined((94, 171), (94, 170)));
    assert!(ground.joined((95, 170), (95, 171)) && ground.joined((95, 171), (95, 170)));
    assert!(ground.joined((94, 170), (95, 170)));
    Ok(())
}

/// omega_ruby_movil.ctm (spike 142): the way to Mauville City turns 38 east at x 56, and once
/// the chunks east of x 40 unloaded a search kept round the loaded ones found none. With the
/// ROM's chunks every cell of the matrix is known; without them a cell no chunk holds is a
/// guess, and the search stays near what is loaded (spike 136).
#[test]
fn test_the_search_is_the_whole_matrix_where_the_roms_chunks_are_read() -> Result<()> {
    let mut town = Town::new();
    let near = town
        .reader
        .ground(&town.r#where()?)?
        .r#box((94, 172), [(94, 171)])?;
    assert_eq!(
        near,
        pathfind::Box {
            west: 70,
            north: 110,
            east: 129,
            south: 209
        }
    );
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            far: Some(Rc::new(|_file| Ok(None))),
            ..Given::default()
        },
        game(),
    );
    let whole = town
        .reader
        .ground(&town.r#where()?)?
        .r#box((94, 172), [(94, 171)])?;
    assert_eq!(
        whole,
        pathfind::Box {
            west: 0,
            north: 0,
            east: 319,
            south: 319
        }
    );
    Ok(())
}

#[test]
fn test_a_block_whose_file_is_not_a_chunk_is_left_out() -> Result<()> {
    let town = Town::new();
    town.memory.write(TOWN_CHUNK, b"ZO");
    town.memory.u32(ROUTE_BLOCK + mapblock::SIZE, &[0]);
    assert_eq!(
        edges(&town)?,
        [((80, 159), (40, 1), EDGE_SOURCE | ROUTE << 16)]
    );
    Ok(())
}

/// A `coll` mesh of one level box of cells, ends out, counted from the chunk's north-west
/// corner: a tree one deep, its one leaf two triangles, as Mt. Chimney's catwalk's (spike 156).
fn level_mesh(west_north: (i64, i64), east_south: (i64, i64), height: f64) -> Vec<u8> {
    let (w, n) = (west_north.0 * 18 - 360, west_north.1 * 18 - 360);
    let (e, s) = (east_south.0 * 18 - 360, east_south.1 * 18 - 360);
    let corners = [(w, n), (e, n), (w, s), (w, s), (e, n), (e, s)];
    let mut body = le32(&[1, 1, 1, 1, 0, corners.len() as i64]);
    for (x, z) in corners {
        body.extend(f32s(&[x as f64, height, z as f64, 1.0]));
    }
    let mut out = b"coll".to_vec();
    out.extend(le32(&[0x10 + body.len() as i64, 0, 1]));
    out.extend(body);
    out
}

/// A chunk in ORAS's nine sections with a deck over its ground, X's eight `in_x`: the ground's
/// table blocked at `crater`, its mesh level over the chunk at `ground`; the deck's table open
/// at `deck` alone, its mesh level over `deck_box` at `over`. Cells are the map's.
fn deck_file(
    origin: (i64, i64),
    crater: &HashSet<(i64, i64)>,
    deck: &HashSet<(i64, i64)>,
    ground: f64,
    over: f64,
    deck_box: ((i64, i64), (i64, i64)),
    in_x: bool,
) -> Vec<u8> {
    let table = |open_at: &dyn Fn((i64, i64)) -> bool| -> Vec<u8> {
        let mut words: Vec<i64> = Vec::new();
        for z in 0..CHUNK {
            for x in 0..CHUNK {
                words.push(if open_at((origin.0 + x, origin.1 + z)) {
                    0x20
                } else {
                    0x01000021
                });
            }
        }
        let mut out = le16(&[CHUNK, CHUNK]);
        out.extend(le32(&words));
        ljust(out, 0x1980)
    };
    let corner = [
        (deck_box.0.0 - origin.0, deck_box.0.1 - origin.1),
        (deck_box.1.0 - origin.0, deck_box.1.1 - origin.1),
    ];
    let mut sections = vec![
        table(&|cell| !crater.contains(&cell)),
        Vec::new(),
        level_mesh((0, 0), (40, 40), ground),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];
    if !in_x {
        sections.push(Vec::new());
    }
    sections.push(table(&|cell| deck.contains(&cell)));
    sections.push(level_mesh(corner[0], corner[1], over));
    let mut offsets = vec![mapblock::HEADER];
    for body in &sections {
        offsets.push(offsets[offsets.len() - 1] + body.len() as i64);
    }
    let mut head = mapblock::MAGIC.to_vec();
    head.extend(le16(&[sections.len() as i64]));
    head.extend(le32(&offsets));
    let mut out = ljust(head, mapblock::HEADER);
    for body in sections {
        out.extend(body);
    }
    out
}

/// omega_ruby_mt-chimney.ctm (spike 156): Mt. Chimney's summit chunks have nine sections, the
/// catwalk over the crater a floor of its own, and were turned away whole, so P took the crater
/// for open ground and said "20 north" into its wall. The ground's table has the crater as a
/// wall; the catwalk's is open across it, at the height of the ground it starts from. Maxie on
/// the catwalk is listed, as a person walled in by the ground's table alone was not. Its floors
/// both at the player's height, it is walked a floor at a time all the same: walked one floor a
/// cell, an item ball 104 below on the path by the crater was taken for one beside the player
/// (or_3d_pathfinding).
#[test]
fn test_a_chunk_with_a_deck_at_the_players_height_is_walked_a_floor_at_a_time() -> Result<()> {
    let town = Town::new();
    let crater: HashSet<(i64, i64)> = (96..111)
        .flat_map(|x| (168..177).map(move |z| (x, z)))
        .collect();
    let catwalk: HashSet<(i64, i64)> = (96..111).map(|x| (x, 172)).collect();
    town.memory.write(
        TOWN_CHUNK,
        &deck_file(
            (80, 160),
            &crater,
            &catwalk,
            108.0,
            108.0,
            ((96, 172), (111, 173)),
            false,
        ),
    );
    town.memory.write(POOL + mmodel::WORLD_Y, &f32s(&[110.0]));
    let at = town.model(1, TOWN, 0, 0x119, (105, 172));
    town.memory.write(at + mmodel::WORLD_Y, &f32s(&[108.0]));
    let r#where = town.r#where()?;
    let maxie = r#where
        .interactables
        .iter()
        .find(|item| item.cell == (105, 172))
        .unwrap()
        .clone();
    let ground = town.reader.ground(&r#where)?;
    assert!(ground.stacked);
    assert!(ground.open((100, 172))? && !ground.open((100, 171))?);
    let path = ground
        .find(
            r#where.cell,
            &ground.goals(&maxie)?,
            "",
            Some(&maxie),
            false,
            None,
        )?
        .expect("a way to Maxie");
    assert_eq!(
        pathfind::spoken(&pathfind::legs(&pathfind::cells(&path), None)?),
        "10 east"
    );
    Ok(())
}

/// x_route-8.ctm (spike 234): X's chunks with a floor over the ground have eight sections, six
/// and the floor's table and mesh, and were turned away whole, so P took the player's chunk on
/// Route 8 for open ground and said "11 west, 23 north" into its wall.
#[test]
fn test_a_chunk_of_x_s_eight_sections_has_its_deck_from_section_six() -> Result<()> {
    let town = Town::with(true, 5, 6);
    let crater: HashSet<(i64, i64)> = (96..111)
        .flat_map(|x| (168..177).map(move |z| (x, z)))
        .collect();
    let catwalk: HashSet<(i64, i64)> = (96..111).map(|x| (x, 172)).collect();
    town.memory.write(
        TOWN_CHUNK,
        &deck_file(
            (80, 160),
            &crater,
            &catwalk,
            108.0,
            108.0,
            ((96, 172), (111, 173)),
            true,
        ),
    );
    town.memory.write(POOL + mmodel::WORLD_Y, &f32s(&[110.0]));
    let r#where = town.r#where()?;
    let ground = town.reader.ground(&r#where)?;
    assert!(ground.open((100, 172))? && !ground.open((100, 171))?);
    Ok(())
}

/// Spikes 159 and 167: Route 119's bridge stands over the path it crosses, and from the foot of
/// the route, the player's height walked the path and cut the bridge, the only way north. The
/// ground is walked a floor at a time instead: the bridge is open over the river, a step along it
/// stays on it, and none takes the player onto it from the path far below.
#[test]
fn test_a_matrix_with_a_deck_is_walked_a_floor_at_a_time() -> Result<()> {
    let town = Town::new();
    let river: HashSet<(i64, i64)> = (100..103)
        .flat_map(|x| (160..200).map(move |z| (x, z)))
        .collect();
    let bridge: HashSet<(i64, i64)> = (96..107).map(|x| (x, 172)).collect();
    town.memory.write(
        TOWN_CHUNK,
        &deck_file(
            (80, 160),
            &river,
            &bridge,
            0.0,
            99.0,
            ((96, 172), (107, 173)),
            false,
        ),
    );
    let ground = town.reader.ground(&town.r#where()?)?;
    assert!(ground.stacked);
    assert!(ground.open((100, 172))?);
    assert_eq!(ground.floors((99, 172))?, [(0, Some(0.0)), (1, Some(99.0))]);
    assert_eq!(ground.climb(1, (99, 172), (100, 172))?, Some(1));
    assert_eq!(ground.climb(0, (99, 172), (100, 172))?, None);
    Ok(())
}

/// Spike 167: each of Sootopolis Gym's two stairs climbs from its lower floor to 189, beside the
/// entrance's room at 198 on the floor over it, and is listed at its top. Where the floor over it
/// lies above the lower floor's own, far below, nothing is walked between them.
#[test]
fn test_a_stair_is_where_the_ground_meets_a_floor_over_it() {
    let mut ground_heights: HashMap<(i64, i64), f64> = (6..10)
        .map(|z| ((3, z), 189.0 - 18.0 * (z - 6) as f64))
        .collect();
    ground_heights.insert((10, 5), -18.0);
    let deck_cells: HashMap<(i64, i64), f64> = (3..11).map(|x| ((x, 5), 198.0)).collect();

    fn floor(heights: &HashMap<(i64, i64), f64>) -> Floor {
        let cells: Vec<(i64, i64)> = (0..CHUNK * CHUNK).map(|i| (i % CHUNK, i / CHUNK)).collect();
        Floor {
            words: cells
                .iter()
                .map(|cell| {
                    if heights.contains_key(cell) {
                        0x20
                    } else {
                        0x01000021
                    }
                })
                .collect(),
            heights: cells
                .iter()
                .map(|cell| heights.get(cell).copied())
                .collect(),
            ramp: false,
        }
    }

    let (lower, upper) = (floor(&ground_heights), floor(&deck_cells));
    let chunk = Chunk {
        floors: vec![lower.clone(), upper],
        ..Chunk::new((40, 0), lower.words)
    };
    assert_eq!(mapblock::joins(&chunk), IndexMap::from([((43, 6), 189.0)]));
}

/// Khai, playing x_tower_of_mastery's save (spike 248): every street of the Tower of Mastery
/// is measured east, and its lane's key said "east" for a step north up the ramp, "8 west, 16
/// east" for a way west and then north. Where the streets are floors of their own a street's
/// keys are the ways nearest its own.
#[test]
fn test_a_step_up_a_street_is_said_by_the_way_it_runs_not_the_way_it_is_measured() -> Result<()> {
    let ground = raised_lanes(&[(1, (9.0, 81.0)), (2, (9.0, 9.0))], &[(1, 2, 4)])?;
    let path = ground.find((0, 4), &goals_of(&[((0, 0), None)]), "", None, false, None)?;
    let path = pathfind::cells(&path.expect("a way"));
    let key = |cell, step| ground.key(cell, step);
    assert_eq!(
        pathfind::spoken(&pathfind::legs_with(&path, None, Some(&key), None)?),
        "4 north"
    );
    Ok(())
}

/// A ground whose streets are floors of their own, each line (start, end, cells along) one
/// cell wide and measured east, as all the Tower of Mastery's are, laid by `Rails.lanes`.
fn raised_lanes(points: &[(i64, (f64, f64))], lines: &[(i64, i64, i64)]) -> Result<Ground> {
    let network = rail::Network {
        lines: lines
            .iter()
            .enumerate()
            .map(|(i, &(start, end, along))| {
                let id = 0x8000 | i as i64;
                (
                    id,
                    rail::Line {
                        way: rail::WAY_EAST,
                        ..rail::Line::new(id, start, end, along, 1, vec![0x20; along as usize + 1])
                    },
                )
            })
            .collect(),
        places: points.iter().copied().collect(),
        cameras: 2,
        ..Default::default()
    };
    let rails = rail::Rails::new(
        FakeMemory::new(),
        Some(Rc::new(move |_files: &[i64]| Ok(network.clone()))),
        rail::Records::default(),
        camera::ORAS,
        Vec::new(),
    );
    Ok(Ground {
        lanes: rails.lanes(&[1], true, 0)?,
        raised: true,
        ..Ground::default()
    })
}

/// x_gym_4 (spike 250): Coumarine Gym's lowest platform curves round the tree, and holding right
/// alone walked the player south-east, east and north-east along it from (56.2, 74.2) to (64.3,
/// 73.7), where P said each bend as a leg of its own, "2 south-east, 2 north-east".
#[test]
fn test_a_curving_street_one_key_walks_is_one_leg() -> Result<()> {
    let ground = raised_lanes(
        &[
            (1, (9.0, 45.0)),
            (2, (81.0, 81.0)),
            (3, (189.0, 81.0)),
            (4, (261.0, 45.0)),
        ],
        &[(1, 2, 4), (2, 3, 6), (3, 4, 4)],
    )?;
    let path: Vec<(i64, i64)> = [(0, 2), (1, 3), (2, 3), (3, 4)]
        .into_iter()
        .chain((4..11).map(|x| (x, 4)))
        .chain([(11, 3), (12, 3), (13, 2), (14, 2)])
        .collect();
    let key = |cell, step| ground.key(cell, step);
    assert_eq!(
        pathfind::spoken(&pathfind::legs_with(&path, None, Some(&key), None)?),
        "14 east"
    );
    Ok(())
}

/// The ROM's map chunks for one chunk, file 0: the ground open along z 8 at 0, from x 0 to
/// 4 and again from 10 to 13, walled between as the Tower of Mastery's is under its ramp past
/// the ramp's foot, and a landing at 90 at its end, x 14 to 16, a floor of its own.
///
/// Python's `RampChunks` class; here its ground and landing, which `ramp_terrain` hands the
/// terrain as its methods would be.
fn ramp_chunks() -> (Floor, Floor) {
    let cells: Vec<(i64, i64)> = (0..CHUNK * CHUNK).map(|i| (i % CHUNK, i / CHUNK)).collect();
    let ground: HashSet<(i64, i64)> = (0..5).chain(10..14).map(|x| (x, 8)).collect();
    let landing: HashSet<(i64, i64)> = (14..17).map(|x| (x, 8)).collect();
    let words = |open: &HashSet<(i64, i64)>| -> Vec<i64> {
        cells
            .iter()
            .map(|cell| {
                if open.contains(cell) {
                    0x20
                } else {
                    0x01000021
                }
            })
            .collect()
    };
    (
        Floor {
            words: words(&ground),
            heights: vec![Some(0.0); (CHUNK * CHUNK) as usize],
            ramp: false,
        },
        Floor {
            words: words(&landing),
            heights: cells
                .iter()
                .map(|cell| landing.contains(cell).then_some(90.0))
                .collect(),
            ramp: false,
        },
    )
}

/// A terrain reading `ramp_chunks` as the ROM's, with `streets` its streets.
fn ramp_terrain(streets: Option<rail::Streets>) -> Terrain {
    let (ground, landing) = ramp_chunks();
    let (cells, heights) = (ground.words.clone(), ground.heights.clone());
    let floors = vec![ground, landing];
    Terrain::new(
        Some(Rc::new(move |_file| Ok(Some(cells.clone()))) as FarCells),
        Some(Rc::new(move |_file| Ok(floors.clone())) as FarFloors),
        Some(Rc::new(move |_file| Ok(Some(heights.clone()))) as FarHeights),
        Some(Rc::new(|_number| Ok(None)) as FarMatrices),
        streets,
    )
}

/// x_tower_of_mastery (spike 248): the Tower of Mastery's ramp winds up round the ring of
/// floor under it, and is in its chunk's streets alone, each point with its height. Its chunk's
/// floors are the ground and the landings, so P walked the ground under the ramp and said "then
/// can't continue" to every stair. Its street is a floor of its own now, climbed a cell at a
/// time from the ground to the landing at its top, and the ground under its top is not.
#[test]
fn test_a_ramp_in_the_streets_is_climbed_from_the_ground_to_the_landing_it_reaches() -> Result<()> {
    let (foot, top) = (0x11, 0x12);
    let street = rail::Network {
        lines: IndexMap::from([(
            0x8011,
            rail::Line {
                way: rail::WAY_EAST,
                ..rail::Line::new(0x8011, foot, top, 9, 1, vec![0x20; 10])
            },
        )]),
        places: IndexMap::from([
            (foot, (4.5 * 18.0, 8.5 * 18.0)),
            (top, (13.5 * 18.0, 8.5 * 18.0)),
        ]),
        cameras: 0,
        heights: IndexMap::from([(foot, 0.0), (top, 90.0)]),
        sides: IndexMap::from([(foot, (0.0, 9.0)), (top, (0.0, 9.0))]),
    };
    let streets: rail::Streets = Rc::new(move |_files: &[i64]| Ok(street.clone()));

    let goal = goals_of(&[((16, 8), None)]);
    for (read, found) in [(Some(streets), true), (None, false)] {
        let terrain = ramp_terrain(read);
        *terrain.files.borrow_mut() = (1, Rc::new(vec![0]));
        let far = terrain.far(0, 0)?.expect("the chunk");
        let chunk = terrain.with_streets(&far)?;
        let laid = Chunk {
            floors: chunk.floors.clone(),
            ..(*terrain.flattened(&chunk, None, false)?).clone()
        };
        let ground = Ground {
            chunks: vec![Rc::new(laid)],
            stacked: terrain.stacked(&|| Ok(Vec::new()))?,
            level: Some(0.0),
            ..Ground::default()
        };
        let way = ground.find((1, 8), &goal, "", None, false, None)?;
        assert_eq!(way.is_some(), found, "found {found}");
        if let Some(way) = way {
            let path = pathfind::cells(&way);
            let mut heights = Vec::new();
            for (&cell, floor) in path.iter().zip(ground.walked(&way)?) {
                let floors = ground.floors(cell)?;
                let (_, height) = floors
                    .iter()
                    .find(|(k, _)| *k == floor)
                    .expect("the floor walked");
                heights.push(height.unwrap_or(0.0));
            }
            assert_eq!(heights[..4], [0.0, 0.0, 0.0, 0.0]);
            assert_eq!(heights.last(), Some(&90.0));
            let mut sorted = heights.clone();
            py::sort_by_lt(&mut sorted, |a, b| a < b);
            assert_eq!(heights, sorted);
        }
    }
    Ok(())
}

/// Spike 167: Sootopolis Gym's invisible blockers, sprite 0x8a, rows of three across the
/// corridors up from its rooms, were listed as nameless people. Nobody is listed there, and the
/// cell is still in the way. So is `STOPPER3`, ten of them round Seafloor Cavern's submarine and
/// one on the Sealed Chamber's wall (movie44), but in Mauville Gym, where it is a barrier.
#[test]
fn test_the_invisible_blocker_is_nobody_and_stands_in_the_way() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            blockers: HashSet::from(oras::BLOCKERS),
            barriers: HashSet::from([(ROUTE, oras::STOPPER3)]),
            ..game()
        },
    );
    town.model(1, TOWN, 0, oras::STOPPER, (96, 172));
    town.model(2, TOWN, 1, oras::STOPPER3, (97, 172));
    town.model(3, ROUTE, 0, oras::STOPPER3, (98, 172));
    let r#where = town.r#where()?;
    let seen: Vec<(i64, i64)> = r#where
        .interactables
        .iter()
        .filter(|item| item.move_model)
        .map(|item| item.cell)
        .collect();
    assert_eq!(seen, [(98, 172)]);
    assert!(!town.reader.ground(&r#where)?.open((96, 172))?);
    assert!(!town.reader.ground(&r#where)?.open((97, 172))?);
    Ok(())
}

/// Route 6 (x_route-6.ctm, 2026-10-19): `MONSBALL_RARE` and the grass a wild Pokémon leaps
/// out of were each listed under interactables as an object, which nothing answered. The
/// grass is called by its sprite, as any thing is.
#[test]
fn test_a_rare_ball_is_an_item_ball_and_grass_a_pokemon_leaps_out_of_is_pokemon() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            ambushes: HashSet::from([xy::TOBI_KUSA]),
            ..game()
        },
    );
    town.model(2, TOWN, 1, 0x2001, (100, 180));
    town.model(3, TOWN, 2, xy::TOBI_KUSA, (110, 170));
    let r#where = town.r#where()?;
    let items: Vec<(String, (i64, i64))> = r#where
        .of("items")
        .into_iter()
        .map(|item| (item.kind, item.cell))
        .collect();
    assert_eq!(items, [("item ball".to_string(), (100, 180))]);
    let grass: Vec<_> = r#where
        .of("pokémon")
        .iter()
        .map(|item| (item.cell, item.naming(TOWN)))
        .collect();
    assert_eq!(grass, [((110, 170), Some(("sprite", xy::TOBI_KUSA, 0)))]);
    assert!(!r#where.of("signs").iter().any(|item| item.move_model));
    Ok(())
}

/// Petalburg Woods (or_mossy_rock, 2026-10-25): the sparkle a Mega Stone lies at was listed
/// under interactables, an object called "Mega Stone" by its sprite. It is an item.
#[test]
fn test_a_mega_stone_is_an_item() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            pickups: HashSet::from([oras::KIRAKIRA]),
            ..game()
        },
    );
    town.model(2, TOWN, 1, oras::KIRAKIRA, (100, 180));
    let r#where = town.r#where()?;
    let items: Vec<_> = r#where
        .of("items")
        .iter()
        .map(|item| (item.cell, item.naming(TOWN)))
        .collect();
    assert_eq!(items, [((100, 180), Some(("sprite", oras::KIRAKIRA, 0)))]);
    assert!(
        r#where
            .of("signs")
            .iter()
            .all(|item| item.code != oras::KIRAKIRA)
    );
    Ok(())
}

/// Glittering Cave (spike 239): nine models drawn as `ENEMY3D` were people standing in the way,
/// "22 south, then can't continue". A wild battle begins as the player comes up to one, and after
/// it they walk straight through its cell.
#[test]
fn test_a_spot_a_pokemon_lurks_at_unseen_is_pokemon_and_walked_through() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            lurkers: HashSet::from([xy::ENEMY3D]),
            ..game()
        },
    );
    town.model(3, TOWN, 3, xy::ENEMY3D, (100, 170));
    let r#where = town.r#where()?;
    let cells: Vec<(i64, i64)> = r#where.of("pokémon").iter().map(|item| item.cell).collect();
    assert_eq!(cells, [(100, 170)]);
    let ground = town.reader.ground(&r#where)?;
    assert!(ground.open((100, 170))? && !ground.stepped.contains(&(100, 170)));
    Ok(())
}

/// Spike 203: deep in the Cave of Origin three models drawn as nothing stand in a row in front
/// of Groudon, which stands in the magma, with its script; in Sootopolis City one stands on each
/// locked door. Each was listed as a nameless person, and so was the player in the Magma Suit,
/// a work object at their own cell drawn as `MAGMA_HERO_RS`. The three are one Groudon; the
/// door's is nobody, and in the way; the suit is the player.
#[test]
fn test_a_model_drawn_as_nothing_is_what_it_stands_in_for() -> Result<()> {
    let (groudon, script) = (0x101B, 0x2E8);
    let mut town = Town::new();
    let data = zone_file(&ZoneSpec {
        npcs: &[
            (groudon, 100, 165, script, None),
            (mmodel::NONDRAW, 99, 170, script, None),
            (mmodel::NONDRAW, 100, 170, script, None),
            (mmodel::NONDRAW, 106, 171, 0x2F0, None),
        ],
        warps: &[(0xE1, WARP_DOOR, 106, 171, None, None)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER, &[FILE, OTHER, 0x11 | (data.len() as i64) << 8]);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            player_looks: oras::PLAYER_LOOKS.collect(),
            ..game()
        },
    );
    town.model(1, TOWN, 1, mmodel::NONDRAW, (99, 170));
    town.model(2, TOWN, 2, mmodel::NONDRAW, (100, 170));
    town.model(3, TOWN, 3, mmodel::NONDRAW, (106, 171));
    town.model(4, TOWN, 4, 0xB1, (94, 172));
    let people = |r#where: &Field| -> IndexMap<(i64, i64), i64> {
        r#where
            .interactables
            .iter()
            .filter(|item| item.category == "people")
            .map(|item| (item.cell, item.code))
            .collect()
    };
    let r#where = town.r#where()?;
    let sized: IndexMap<(i64, i64), (i64, (i64, i64))> = r#where
        .interactables
        .iter()
        .filter(|item| item.category == "people")
        .map(|item| (item.cell, (item.code, item.size)))
        .collect();
    assert_eq!(sized, IndexMap::from([((99, 170), (groudon, (2, 1)))]));
    assert!(!town.reader.ground(&r#where)?.open((106, 171))?);
    // Once Groudon itself stands where it can be listed, it is listed and not its stand-ins.
    town.model(5, TOWN, 0, groudon, (100, 165));
    let r#where = town.r#where()?;
    assert_eq!(people(&r#where), IndexMap::from([((100, 165), groudon)]));
    Ok(())
}

/// Sea Mauville (or_sea_mauville_locked_doors): each locked door is furniture on its door's
/// own cell, saying "The door is locked." until its key opens it, and it was listed as a sign
/// beside the door, before and after. Only the door is listed; furniture beside it still is.
#[test]
fn test_furniture_standing_on_a_warp_is_its_lock_and_the_door_is_listed() -> Result<()> {
    let town = Town::new();
    let data = zone_file(&ZoneSpec {
        furniture: &[(0x7, 106, 171), (0x8, 104, 171)],
        warps: &[(0xE1, WARP_DOOR, 106, 171, None, None)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER, &[FILE, OTHER, 0x11 | (data.len() as i64) << 8]);
    let r#where = town.r#where()?;
    let signs: HashSet<(i64, i64)> = r#where.of("signs").iter().map(|item| item.cell).collect();
    assert!(signs.contains(&(104, 171)) && !signs.contains(&(106, 171)));
    let doors: Vec<(i64, i64)> = r#where
        .of("exits")
        .iter()
        .filter(|item| item.kind == "door")
        .map(|item| item.cell)
        .collect();
    assert_eq!(doors, vec![(106, 171)]);
    Ok(())
}

/// Spike 223: X's Santalune Gym has five models drawn as nothing in its corridor behind Viola,
/// all running script 0x269, which no drawn record has; they were five nameless people. In a
/// game with `gates` they are listed as nobody and stand in the way, while a model drawn as
/// nothing with a script of its own is still listed.
#[test]
fn test_a_row_of_models_drawn_as_nothing_with_a_script_no_drawn_record_has_is_a_gate() -> Result<()>
{
    let (nothing, script) = (mmodel::NONDRAW, 0x269);
    let gate: Vec<(i64, i64)> = (0..5).map(|i| (98 + i, 170)).collect();
    let mut npcs = vec![(0xB1, 94, 174, 0x100, None)];
    npcs.extend(gate.iter().map(|&(x, z)| (nothing, x, z, script, None)));
    npcs.push((nothing, 104, 170, 0x270, None));
    let data = zone_file(&ZoneSpec {
        npcs: &npcs,
        info: &town_info(),
        ..ZoneSpec::default()
    });
    let people = |game: Game| -> Result<HashSet<(i64, i64)>> {
        let mut town = Town::new();
        town.memory.write(FILE, &data);
        town.memory
            .u32(HOLDER, &[FILE, OTHER, 0x11 | (data.len() as i64) << 8]);
        town.reader = field_reader(town.memory.clone(), Given::default(), game);
        town.model(1, TOWN, 0, 0xB1, (94, 174));
        for (slot, &cell) in (2..).zip(&gate) {
            town.model(slot, TOWN, slot - 1, nothing, cell);
        }
        town.model(7, TOWN, 6, nothing, (104, 170));
        let r#where = town.r#where()?;
        let ground = town.reader.ground(&r#where)?;
        for &cell in &gate {
            assert!(!ground.open(cell)?);
        }
        Ok(r#where
            .interactables
            .iter()
            .filter(|item| item.category == "people")
            .map(|item| item.cell)
            .collect())
    };
    let mut everyone: HashSet<(i64, i64)> = HashSet::from([(94, 174), (104, 170)]);
    everyone.extend(gate.iter().copied());
    assert_eq!(people(game())?, everyone);
    assert_eq!(
        people(Game {
            gates: true,
            ..game()
        })?,
        HashSet::from([(94, 174), (104, 170)])
    );
    Ok(())
}

/// Each of X's boutiques stands a model drawn as nothing on its fitting room, which is
/// listed, and it was listed beside it as "person 1 north" (x_cyllage_boutique_fitting_room).
#[test]
fn test_a_model_drawn_as_nothing_on_a_piece_of_furniture_is_nobody() -> Result<()> {
    let mut town = Town::new();
    let data = zone_file(&ZoneSpec {
        sized: &[(0x29D3, 106, 171, 1, 1)],
        npcs: &[(mmodel::NONDRAW, 106, 171, 0, None)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER, &[FILE, OTHER, 0x11 | (data.len() as i64) << 8]);
    town.reader = field_reader(town.memory.clone(), Given::default(), game());
    town.model(1, TOWN, 0, mmodel::NONDRAW, (106, 171));
    let r#where = town.r#where()?;
    assert!(r#where.of("people").is_empty());
    assert!(
        r#where
            .of("signs")
            .iter()
            .map(|item| item.cell)
            .any(|cell| cell == (106, 171))
    );
    Ok(())
}

/// Seafloor Cavern's submarine is four furniture records with one script, two 4 by 4 and an
/// end of 1 by 2 at each side, and was listed as four signs; an elevator's door is two records
/// side by side with a shared script each, and was two (Orion, movie44). Each is one thing over
/// the box round it, by its first record; one script's records apart stay apart.
#[test]
fn test_furniture_of_one_script_that_touches_is_one_thing() -> Result<()> {
    let mut town = Town::new();
    let data = zone_file(&ZoneSpec {
        sized: &[
            (1, 107, 166, 4, 4),
            (1, 111, 166, 4, 4),
            (1, 106, 167, 1, 2),
            (1, 115, 167, 1, 2),
            (0x27E3, 98, 159, 1, 1),
            (0x27E2, 99, 159, 1, 1),
            (5, 90, 160, 1, 1),
            (5, 92, 160, 1, 1),
        ],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER, &[FILE, OTHER, 0x11 | (data.len() as i64) << 8]);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            elevators: oras::ELEVATORS.collect(),
            ..game()
        },
    );
    let mut signs: Vec<((i64, i64), (i64, i64), i64)> = town
        .r#where()?
        .interactables
        .iter()
        .filter(|item| item.place_kind() == Some(FURNITURE_SOURCE))
        .map(|item| (item.cell, item.size, source_number(item.source)))
        .collect();
    signs.sort();
    assert_eq!(
        signs,
        [
            ((90, 160), (1, 1), 6),
            ((92, 160), (1, 1), 7),
            ((98, 159), (2, 1), 4),
            ((106, 166), (10, 4), 0)
        ]
    );
    Ok(())
}

/// A vending machine's record runs a script of its zone's, whose case runs one of five shared
/// ones; an elevator's door is a shared script's own number (spike 210). The shipped table says
/// what each is; a record of the zone's own script stays a sign.
#[test]
fn test_furniture_is_what_the_shipped_table_says_the_shared_script_it_runs_is() -> Result<()> {
    let mut town = Town::new();
    let data = zone_file(&ZoneSpec {
        furniture: &[(9, 90, 160), (0x27E3, 98, 159), (5, 92, 165)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER, &[FILE, OTHER, 0x11 | (data.len() as i64) << 8]);
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            shared: IndexMap::from([((TOWN, 9), 0x28F4)]),
            furniture_words: Some(IndexMap::from([
                (0x28F4, "vending machine".to_string()),
                (0x27E3, "elevator".to_string()),
            ])),
            ..Given::default()
        },
        game(),
    );
    let mut seen: Vec<((i64, i64), String)> = town
        .r#where()?
        .interactables
        .iter()
        .filter(|item| item.place_kind() == Some(FURNITURE_SOURCE))
        .map(|item| (item.cell, item.kind.clone()))
        .collect();
    seen.sort();
    assert_eq!(
        seen,
        [
            ((90, 160), "vending machine".to_string()),
            ((92, 165), "sign".to_string()),
            ((98, 159), "elevator".to_string())
        ]
    );
    Ok(())
}

/// Spike 161: Fortree Gym's third door, arms north and west two cells long about the corner at
/// (23, 23), stood with the ground walled at (21, 23) and (21, 24). Walked into westward from
/// (23, 22), it would swing its west arm through that quarter, and the game did not turn it;
/// walked into northward from (22, 23), it turns the other way, through open cells.
#[test]
fn test_a_rotating_door_turns_only_where_its_arms_sweep_open_cells() -> Result<()> {
    let door = RotatingDoor {
        turn: 1,
        ..RotatingDoor::new((23, 23), doors::patterns()[&1].clone())
    };
    assert_eq!(
        door.at(1)?,
        IndexMap::from([(pathfind::NORTH, 2), (pathfind::WEST, 2)])
    );
    let blocked: HashSet<(i64, i64)> = HashSet::from([(21, 23), (21, 24)]);
    let words: Vec<i64> = (0..CHUNK * CHUNK)
        .map(|i| {
            if blocked.contains(&(i % CHUNK, i / CHUNK)) {
                BLOCKED
            } else {
                0x20
            }
        })
        .collect();
    let ground = Ground {
        chunks: vec![Rc::new(Chunk::new((0, 0), words))],
        doors: vec![door],
        ..Ground::default()
    };
    let doors = Doors::of(&ground).expect("the ground's doors");
    assert_eq!(
        doors.step(&ground, &[1], (23, 22), (22, 22), ANY_FLOOR)?,
        None
    );
    assert_eq!(
        doors.step(&ground, &[1], (22, 23), (22, 22), ANY_FLOOR)?,
        Some(vec![0])
    );
    assert_eq!(
        doors.step(&ground, &[1], (24, 22), (25, 22), ANY_FLOOR)?,
        Some(vec![1])
    );
    assert_eq!(
        ground.named(
            &ground.follow(&[(22, 24), (22, 23), (22, 22)], None)?,
            &|_| Ok(String::new())
        )?,
        IndexMap::from([(2, "push".to_string())])
    );
    Ok(())
}

/// Spike 156: across ORAS's decks, one meets the ground within 14 units where it is walked
/// onto, and stands 54 or more over the ground beside it. On the deck the ground below its sides
/// is closed; under it the deck is closed where it leaves the ground, and the ground under it
/// is walked.
#[test]
fn test_a_deck_over_lower_ground_is_kept_to_and_walked_under() -> Result<()> {
    let ground = Floor {
        words: vec![0x20; (CHUNK * CHUNK) as usize],
        heights: vec![Some(-63.0); (CHUNK * CHUNK) as usize],
        ramp: false,
    };
    let on_deck = |i: i64| i / CHUNK == 5 && (3..=20).contains(&(i % CHUNK));
    let deck = Floor {
        words: (0..CHUNK * CHUNK)
            .map(|i| if on_deck(i) { 0x20 } else { 0x01000021 })
            .collect(),
        heights: (0..CHUNK * CHUNK)
            .map(|i| if on_deck(i) { Some(0.0) } else { None })
            .collect(),
        ramp: false,
    };
    // The deck runs on past the ground's edge at x 20, over nothing the ground has.
    let ground = Floor {
        words: ground
            .words
            .iter()
            .enumerate()
            .map(|(i, &word)| {
                if i as i64 % CHUNK == 20 {
                    0x01000021
                } else {
                    word
                }
            })
            .collect(),
        ..ground
    };
    let chunk = Chunk {
        floors: vec![ground.clone(), deck],
        ..Chunk::new((0, 0), ground.words.clone())
    };
    let on = mapblock::flat(&chunk, Some(2.0));
    assert!(on.walkable((3, 5))? && on.walkable((20, 5))?);
    assert!(!on.walkable((10, 4))? && !on.walkable((10, 6))? && !on.walkable((2, 5))?);
    let under = mapblock::flat(&chunk, Some(-61.0));
    assert!(under.walkable((10, 4))? && under.walkable((10, 5))? && under.walkable((10, 6))?);
    assert!(!under.walkable((20, 5))?);
    Ok(())
}

#[test]
fn test_a_chunk_is_read_again_only_when_its_block_reloads() -> Result<()> {
    let mut town = Town::new();
    assert_eq!(
        edges(&town)?,
        [((99, 159), (3, 1), EDGE_SOURCE | ROUTE << 16)]
    );
    // A ready block's file rewritten in place, which the game does only through a load that
    // takes the block out of ready first: the table stays as read.
    town.memory.write(
        TOWN_CHUNK,
        &chunk_file((80, 160), &border_with(160, &[(85, 87)])),
    );
    assert_eq!(
        edges(&town)?,
        [((99, 159), (3, 1), EDGE_SOURCE | ROUTE << 16)]
    );
    // Another chunk loaded into the block, its file elsewhere, is read fresh.
    town.block(
        TOWN_BLOCK,
        TOWN_CHUNK + 0x100100,
        (80, 160),
        &border_with(160, &[(85, 87)]),
    );
    town.block(
        ROUTE_BLOCK,
        ROUTE_CHUNK + 0x100100,
        (80, 120),
        &border_with(159, &[(85, 87)]),
    );
    assert_eq!(
        edges(&town)?,
        [((85, 159), (2, 1), EDGE_SOURCE | ROUTE << 16)]
    );
    Ok(())
}

#[test]
fn test_a_chunk_still_loading_is_not_read_until_its_block_is_ready() -> Result<()> {
    let mut town = Town::new();
    // The block has the new chunk's centre and file, and the file still the old chunk's table.
    town.block(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border_with(160, &[(85, 87)]),
    );
    town.memory
        .write(TOWN_CHUNK, &chunk_file((80, 160), &border(160)));
    town.memory.u32(TOWN_BLOCK + mapblock::STATE, &[2]);
    assert!(town.reader.ground(&town.r#where()?)?.open((85, 160))?);
    town.memory.write(
        TOWN_CHUNK,
        &chunk_file((80, 160), &border_with(160, &[(85, 87)])),
    );
    town.memory
        .u32(TOWN_BLOCK + mapblock::STATE, &[mapblock::READY]);
    assert!(town.reader.ground(&town.r#where()?)?.open((85, 160))?);
    assert!(!town.reader.ground(&town.r#where()?)?.open((88, 160))?);
    Ok(())
}

/// A Pokémon Center's PC as spike 83 read it, and a kind the game uses that is two cells wide
/// where it is found.
pub(crate) const PC_TILE: i64 = 0xD50E0001;
pub(crate) const SHELF_TILE: i64 = 0xD90E0001;

#[test]
fn test_a_use_tile_is_listed_by_its_kind_and_a_run_of_one_kind_is_one_thing() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            tiles: Some(IndexMap::from([(PC_KIND, "PC".to_string())])),
            ..Given::default()
        },
        Game {
            hidden: Some(dex_nav),
            ..game()
        },
    );
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border(160),
        &[
            ((95, 170), PC_TILE),
            ((100, 165), SHELF_TILE),
            ((101, 165), SHELF_TILE),
        ],
    );
    let signs = town.r#where()?.of("signs");
    assert_eq!(
        signs
            .iter()
            .map(|item| (item.kind.as_str(), item.cell, item.size))
            .collect::<Vec<_>>(),
        [
            ("PC", (95, 170), (1, 1)),
            ("sign", (97, 171), (1, 1)),
            ("object", (100, 165), (2, 1)),
            ("object", (110, 170), (1, 1)),
        ]
    );
    assert!(
        town.listing()?.contains(
            &"signs: PC 1 east 2 north; sign 3 east 1 north; object 6 east 7 north; \
          object 16 east 2 north"
                .to_string()
        )
    );
    Ok(())
}

pub(crate) const CABINET_TILE: i64 = 0xDA0E0001;
/// The Azurill Doll's sprite, a thing drawn on the map and listed among the signs (spike 211).
pub(crate) const DOLL: i64 = 0x204A;

/// The Azurill Doll is a doll drawn, a furniture record in front of it saying "It's an
/// Azurill Doll!", and the cabinet under it: three signs in one cell (Orion, 2026-10-14). A
/// record over a cell is the record; a shelf longer than what stands on it stays.
#[test]
fn test_one_spot_is_one_sign_the_thing_drawn_first_then_its_record_then_the_cell() -> Result<()> {
    let mut town = Town::new();
    let data = zone_file(&ZoneSpec {
        furniture: &[(1, 100, 165), (2, 104, 165)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER, &[FILE, OTHER, 0x11 | (data.len() as i64) << 8]);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            hidden: Some(dex_nav),
            ..game()
        },
    );
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border(160),
        &[
            ((100, 165), CABINET_TILE),
            ((104, 165), SHELF_TILE),
            ((105, 165), SHELF_TILE),
            ((108, 165), CABINET_TILE),
        ],
    );
    town.model(1, TOWN, 0, DOLL, (100, 165));
    let signs = town.r#where()?.of("signs");
    let mut seen: Vec<((i64, i64), (i64, i64), i64)> = signs
        .iter()
        .filter(|item| item.cell.1 == 165)
        .map(|item| {
            let kind = if item.move_model {
                -1
            } else {
                source_kind(item.source)
            };
            (item.cell, item.size, kind)
        })
        .collect();
    seen.sort();
    assert_eq!(
        seen,
        [
            ((100, 165), (1, 1), -1),
            ((104, 165), (1, 1), FURNITURE_SOURCE),
            ((104, 165), (2, 1), TILE_SOURCE),
            ((108, 165), (1, 1), TILE_SOURCE)
        ]
    );
    Ok(())
}

/// Every cell of one kind runs one script, so the kind says what it is: `0xd9` a bookshelf
/// wherever it stands (spike 210). The PC's word is the ROM's, ahead of the shipped table's.
#[test]
fn test_a_use_tile_is_what_the_shipped_table_says_its_kind_is_and_the_pc_what_the_rom_says()
-> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            words: IndexMap::from([(PC_KIND, "パソコン".to_string())]),
            tiles: Some(IndexMap::from([
                (PC_KIND, "PC".to_string()),
                (SHELF_TILE >> 24, "bookshelf".to_string()),
            ])),
            ..Given::default()
        },
        Game {
            hidden: Some(dex_nav),
            ..game()
        },
    );
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border(160),
        &[
            ((95, 170), PC_TILE),
            ((100, 165), SHELF_TILE),
            ((101, 165), SHELF_TILE),
        ],
    );
    let signs = town.r#where()?.of("signs");
    let seen: Vec<(String, (i64, i64))> = signs
        .iter()
        .filter(|item| [(95, 170), (100, 165)].contains(&item.cell))
        .map(|item| (item.kind.clone(), item.size))
        .collect();
    assert_eq!(
        seen,
        [
            ("パソコン".to_string(), (1, 1)),
            ("bookshelf".to_string(), (2, 1))
        ]
    );
    Ok(())
}

/// Spike 159: the player could not find the Mach Bike's slopes or the Acro Bike's rails to
/// record them, since nothing listed either. A patch of one slope kind is one slope; a rail, its
/// cells walled, is got on at either end, as Route 119's runs west, north and west again.
#[test]
fn test_a_slope_is_one_thing_and_a_rail_is_listed_at_its_ends() -> Result<()> {
    let slope: Vec<((i64, i64), i64)> = (84..87)
        .flat_map(|x| (165..170).map(move |z| ((x, z), 0x6C000020)))
        .collect();
    let rail: Vec<((i64, i64), i64)> = (100..105)
        .map(|x| (x, 175))
        .chain([(104, 176), (104, 177)])
        .chain((104..109).map(|x| (x, 178)))
        .map(|cell| (cell, 0x6E000021))
        .collect();
    // Jagged Pass's bumpy slopes stand one at a time in rows of ledges, each a thing of its own.
    let bumpy = [((90, 170), 0x6F000021), ((90, 173), 0x6F000021)];
    let mut town = Town::new();
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border(160),
        &[slope, rail, bumpy.to_vec()].concat(),
    );
    let bike = town.r#where()?.of("bike");
    let mut seen: Vec<(&str, (i64, i64), (i64, i64))> = bike
        .iter()
        .map(|item| (item.kind.as_str(), item.cell, item.size))
        .collect();
    seen.sort();
    assert_eq!(
        seen,
        [
            ("bumpy slope", (90, 170), (1, 1)),
            ("bumpy slope", (90, 173), (1, 1)),
            ("rail", (100, 175), (1, 1)),
            ("rail", (108, 178), (1, 1)),
            ("slope", (84, 165), (3, 5)),
        ]
    );
    let sources: HashSet<i64> = bike.iter().map(|item| item.source).collect();
    assert_eq!(sources.len(), 5);
    Ok(())
}

/// Spike 164: Route 128's dive spots were not listed. A patch of deep water is one exit, at its own
/// cell nearest the player: its biggest is a diagonal band, whose box is mostly plain sea.
#[test]
fn test_a_dive_spot_is_an_exit_at_its_patchs_cell_nearest_the_player() -> Result<()> {
    let mut tiles: Vec<((i64, i64), i64)> = (84..87)
        .flat_map(|x| (165..168).map(move |z| ((x, z), 0x411A0006)))
        .collect();
    let mut band: IndexMap<(i64, i64), i64> = IndexMap::new();
    for i in 0..6 {
        band.insert((100 + i, 165 + i), 0x411A0006);
    }
    for i in 0..5 {
        band.insert((101 + i, 165 + i), 0x411A0006);
    }
    tiles.extend(band);
    let mut town = Town::new();
    town.block_tiles(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &border(160), &tiles);
    let r#where = town.r#where()?;
    let dives: Vec<Interactable> = r#where
        .of("exits")
        .into_iter()
        .filter(|item| item.kind == "dive spot")
        .collect();
    assert_eq!(r#where.cell, (94, 172));
    let mut seen: Vec<((i64, i64), (i64, i64))> =
        dives.iter().map(|item| (item.cell, item.size)).collect();
    seen.sort();
    assert_eq!(seen, [((86, 167), (1, 1)), ((100, 165), (1, 1))]);
    let sources: HashSet<i64> = dives.iter().map(|item| item.source).collect();
    assert_eq!(sources.len(), 2);
    Ok(())
}

/// Route 9's ride as `Rides`, smashing `smashes`.
fn route_9(smashes: &[i64]) -> Rides {
    Rides {
        ground: HashSet::from([0x32]),
        stops: HashSet::from([0x31]),
        mounts: HashSet::from([0x100F]),
        smashes: smashes.iter().copied().collect(),
    }
}

/// Spike 238: on X's Route 9 a Rhyhorn is got on and off on flat ground, five patches between the
/// spikes and plain ground, and Khai was "not sure when you can get off rhyhorn so that prob need
/// its own category" (2026-10-22). A patch is one thing, at its cell nearest the player. Vaniville
/// Town's pen round Mom's Rhyhorn is flat ground with no spikes, and nothing to ride there.
#[test]
fn test_flat_ground_is_listed_under_riding_a_patch_at_a_time() -> Result<()> {
    let square: Vec<((i64, i64), i64)> = (84..87)
        .flat_map(|x| (165..168).map(move |z| ((x, z), 0x31000020)))
        .collect();
    let row: Vec<((i64, i64), i64)> = (100..104).map(|x| ((x, 175), 0x31000020)).collect();
    let spikes: Vec<((i64, i64), i64)> = (100..104).map(|x| ((x, 176), 0x32000025)).collect();
    let mut listed: Vec<Vec<Interactable>> = Vec::new();
    for tiles in [
        [square.clone(), row.clone()].concat(),
        [square.clone(), row.clone(), spikes.clone()].concat(),
    ] {
        let mut town = Town::new();
        town.reader = field_reader(
            town.memory.clone(),
            Given::default(),
            Game {
                hidden: Some(dex_nav),
                landforms: [gen6(), vec![Arc::new(route_9(&[])) as Arc<dyn Landform>]].concat(),
                ..game()
            },
        );
        town.block_tiles(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &border(160), &tiles);
        listed.push(town.r#where()?.of("riding"));
    }
    let (pen, flat) = (&listed[0], &listed[1]);
    assert!(pen.is_empty());
    let mut seen: Vec<(String, (i64, i64))> = flat
        .iter()
        .map(|item| (item.kind.clone(), item.cell))
        .collect();
    seen.sort();
    assert_eq!(
        seen,
        [
            ("flat ground".to_string(), (86, 167)),
            ("flat ground".to_string(), (100, 175))
        ]
    );
    let sources: HashSet<i64> = flat.iter().map(|item| item.source).collect();
    assert_eq!(sources.len(), 2);
    Ok(())
}

/// Spike 245: Cyllage Gym's walls are climbed from the blue spots at their foot, two side by side
/// at (25, 32) and (26, 32), and got off onto a ledge at a face's side, (31, 32) at 54; Khai
/// asked for their "entry point and exit point maybe? in a category" (2026-10-24). The plateau
/// behind a face's top, open beside it at another height, is neither.
#[test]
fn test_climbing_walls_are_listed_a_run_of_blue_spots_or_a_ledge_at_a_time() -> Result<()> {
    let mut town = Town::with(false, 5, 7);
    let mut files = vec![NO_CHUNK_FILE; 64];
    files[4 * 8 + 2] = 7;
    town.memory.write(MAP + MAP_CHUNK_FILES, &le16(&files));
    let mut tiles: Vec<((i64, i64), i64)> = [105, 106, 111]
        .iter()
        .map(|&x| ((x, 171), 0x59000001))
        .collect();
    tiles.push(((110, 172), 0x51000020));
    let mut blocked: Vec<(i64, i64)> = border(160);
    blocked.extend((107..111).map(|x| (x, 171)));
    blocked.extend([(112, 172), (111, 173)]);
    let words = mapblock::cells(&chunk_file_with((80, 160), &blocked, &tiles, 7))?.unwrap();
    let heights: Vec<Option<f64>> = (0..CHUNK * CHUNK)
        .map(|i| {
            let cell = (80 + i % CHUNK, 160 + i / CHUNK);
            Some(if cell == (111, 171) || cell == (111, 172) {
                54.0
            } else if cell == (105, 170) {
                90.0
            } else {
                0.0
            })
        })
        .collect();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            far: Some(Rc::new(move |file| {
                Ok(if file == 7 { Some(words.clone()) } else { None })
            })),
            far_heights: Some(Rc::new(move |file| {
                Ok(if file == 7 {
                    Some(heights.clone())
                } else {
                    None
                })
            })),
            ..Given::default()
        },
        Game {
            landforms: [gen6(), vec![Arc::new(xy::climbing()) as Arc<dyn Landform>]].concat(),
            ..game()
        },
    );
    let climbing = town.r#where()?.of("climbing");
    let mut seen: Vec<(String, (i64, i64), (i64, i64))> = climbing
        .iter()
        .map(|item| (item.kind.clone(), item.cell, item.size))
        .collect();
    seen.sort();
    assert_eq!(
        seen,
        [
            ("blue spot".to_string(), (105, 172), (2, 1)),
            ("wall ledge".to_string(), (111, 172), (1, 1))
        ]
    );
    let sources: HashSet<i64> = climbing.iter().map(|item| item.source).collect();
    assert_eq!(sources.len(), 2);
    Ok(())
}

/// Spike 259: in X's Poké Ball Factory the cell beside a ramp is `0x51` to `0x54`, a climbing
/// wall's sideways kinds, and the player could not step east from (14, 22), `0x51`, onto the ramp
/// at (15, 22), nor west back off it, and P's way to a belt went that way. The step its kind
/// names is closed both ways, and only that one.
#[test]
fn test_a_ramps_side_is_never_stepped_across_on_foot() -> Result<()> {
    let mut town = Town::with(false, 5, 7);
    let mut files = vec![NO_CHUNK_FILE; 64];
    files[4 * 8 + 2] = 7;
    town.memory.write(MAP + MAP_CHUNK_FILES, &le16(&files));
    let words = mapblock::cells(&chunk_file_with(
        (80, 160),
        &[],
        &[((94, 175), 0x51000020)],
        7,
    ))?
    .unwrap();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            far: Some(Rc::new(move |file| {
                Ok(if file == 7 { Some(words.clone()) } else { None })
            })),
            ..Given::default()
        },
        Game {
            landforms: [gen6(), vec![Arc::new(xy::climbing()) as Arc<dyn Landform>]].concat(),
            ..game()
        },
    );
    let ground = town.reader.ground(&town.r#where()?)?;
    assert!(!ground.joined((94, 175), (95, 175)) && !ground.joined((95, 175), (94, 175)));
    assert!(ground.joined((94, 175), (93, 175)) && ground.joined((94, 175), (94, 176)));
    Ok(())
}

/// Spike 245: in Cyllage Gym the player stepped south onto `0x8a` at (16, 20), at 189, and was
/// carried down the blocked cells south of it, 171 to 117, onto the floor at 108 (x_gym_2_slide).
/// X has `0x8a` in other chunks too; where the cells past it do not fall, it is no slide.
#[test]
fn test_a_slide_is_found_where_the_cells_past_its_top_fall_to_a_lower_landing() -> Result<()> {
    let mut town = Town::with(false, 5, 7);
    let mut files = vec![NO_CHUNK_FILE; 64];
    files[4 * 8 + 2] = 7;
    town.memory.write(MAP + MAP_CHUNK_FILES, &le16(&files));
    let falls: IndexMap<(i64, i64), f64> = (171..175)
        .map(|z| ((95, z), 189.0 - 18.0 * (z - 170) as f64))
        .collect();
    let level = [(100, 171), (100, 172)];
    let tops = [((95, 170), 0x8A000020), ((100, 170), 0x8A000020)];
    let mut blocked: Vec<(i64, i64)> = falls.keys().copied().collect();
    blocked.extend(level);
    let words = mapblock::cells(&chunk_file_with((80, 160), &blocked, &tops, 7))?.unwrap();
    let heights: Vec<Option<f64>> = (0..CHUNK * CHUNK)
        .map(|i| {
            let cell = (80 + i % CHUNK, 160 + i / CHUNK);
            Some(falls.get(&cell).copied().unwrap_or(if cell == (95, 175) {
                108.0
            } else if cell.1 >= 170 {
                189.0
            } else {
                198.0
            }))
        })
        .collect();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            far: Some(Rc::new(move |file| {
                Ok(if file == 7 { Some(words.clone()) } else { None })
            })),
            far_heights: Some(Rc::new(move |file| {
                Ok(if file == 7 {
                    Some(heights.clone())
                } else {
                    None
                })
            })),
            ..Given::default()
        },
        Game {
            landforms: [gen6(), vec![Arc::new(xy::climbing()) as Arc<dyn Landform>]].concat(),
            ..game()
        },
    );
    let slides = town.reader.ground(&town.r#where()?)?.walls.slides.clone();
    let want: IndexMap<(i64, i64), &str> = (170..175).map(|z| ((95, z), pathfind::SOUTH)).collect();
    assert_eq!(slides, want);
    Ok(())
}

/// Spike 247: Shalour Gym's way out is a rail on its floor 1, skated onto from an end cell on
/// floor 0 by the centre, landing on the floor by the entrance, which is no end cell: ridden that
/// one way. On Route 5 a rail over the water is broken in the middle by `0x90`, blocked, and
/// neither half of it lands anywhere.
#[test]
fn test_grind_rails_are_found_on_every_floor_and_never_past_a_blocked_cell() -> Result<()> {
    let mut town = Town::with(false, 5, 7);
    let mut files = vec![NO_CHUNK_FILE; 64];
    files[4 * 8 + 2] = 7;
    town.memory.write(MAP + MAP_CHUNK_FILES, &le16(&files));
    let mut ground: IndexMap<(i64, i64), i64> = IndexMap::from([((85, 170), 0x8B000020)]);
    ground.extend([86, 87, 88].map(|x| ((x, 170), 0x8C000021)));
    ground.insert((89, 170), 0x90000021);
    ground.extend([90, 91, 92].map(|x| ((x, 170), 0x8C000021)));
    ground.insert((93, 170), 0x8B000020);
    ground.insert((100, 170), 0x8B000020);
    let upper: IndexMap<(i64, i64), i64> = (171..176).map(|z| ((100, z), 0x8C000021)).collect();
    let cells: Vec<(i64, i64)> = (0..CHUNK * CHUNK)
        .map(|i| (80 + i % CHUNK, 160 + i / CHUNK))
        .collect();
    let words: Vec<i64> = cells
        .iter()
        .map(|cell| ground.get(cell).copied().unwrap_or(0))
        .collect();
    let floors = vec![
        Floor {
            words: words.clone(),
            heights: vec![Some(0.0); cells.len()],
            ramp: false,
        },
        Floor {
            words: cells
                .iter()
                .map(|cell| upper.get(cell).copied().unwrap_or(BLOCKED))
                .collect(),
            heights: vec![None; cells.len()],
            ramp: false,
        },
    ];
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            far: Some(Rc::new(move |file| {
                Ok(if file == 7 { Some(words.clone()) } else { None })
            })),
            far_floors: Some(Rc::new(move |file| {
                Ok(if file == 7 {
                    floors.clone()
                } else {
                    Vec::new()
                })
            })),
            ..Given::default()
        },
        Game {
            landforms: [gen6(), vec![Arc::new(xy::GRINDING) as Arc<dyn Landform>]].concat(),
            ..game()
        },
    );
    let rails = town
        .reader
        .ground(&town.r#where()?)?
        .grind_rails
        .rails
        .clone();
    let mut want = vec![(100, 170)];
    want.extend(upper.keys().copied());
    want.push((100, 176));
    assert_eq!(
        rails,
        [GrindRail {
            cells: want,
            both: false
        }]
    );
    Ok(())
}

/// Spike 250: Coumarine Gym's vines are furniture records on its streets whose zone script case
/// hands a native one constant, 57 for a vine climbed, each as high as its line's start: the vine
/// at (59, 72), at 0, is climbed up, and its other end, (59, 73) at 145, down. A record whose case
/// runs a vending machine's shared script is no vine.
#[test]
fn test_vines_are_listed_under_climbing_each_end_by_what_its_ride_does() -> Result<()> {
    let low = rail::Line::new(0x8001, 1, 2, 4, 3, vec![0; 15]);
    let high = rail::Line::new(0x8002, 3, 4, 4, 3, vec![0; 15]);
    let start = ((100 * 18 + 9) as f64, (160 * 18 + 9) as f64);
    let end = (start.0, start.1 + (4 * 18) as f64);
    let network = rail::Network {
        lines: IndexMap::from([(low.id, low.clone()), (high.id, high.clone())]),
        places: IndexMap::from([(1, start), (2, end), (3, start), (4, end)]),
        cameras: 0,
        heights: IndexMap::from([(1, 0.0), (2, 0.0), (3, 145.0), (4, 145.0)]),
        ..Default::default()
    };
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            streets: Some(Rc::new(move |_chunks: &[i64]| Ok(network.clone()))),
            far: Some(Rc::new(|_file| Ok(None))),
            natives: IndexMap::from([
                ((TOWN, 0x1B), (57, 0)),
                ((TOWN, 0x1C), (57, 1)),
                ((TOWN, 9), (90, 0x28F4)),
            ]),
            ..Given::default()
        },
        Game {
            vining: Some(xy::VINING),
            ..game()
        },
    );
    town.memory
        .write(MAP + MAP_CHUNK_FILES, &le16(&[0, 700, 701]));
    let mut data = zone_file(&ZoneSpec {
        furniture: &[(0x1C, 0, 0), (0x1B, 0, 0), (9, 0, 0)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    let first = zone_header(5) + INFO_SIZE + ENTITY_HEADER;
    for (i, (line, along)) in [(&low, 2), (&high, 3), (&low, 1)].into_iter().enumerate() {
        put(
            &mut data,
            first + i as i64 * FURNITURE + 6,
            &[le16(&[1]), le32(&[line.id]), le16(&[along, 0, 1, 1])].concat(),
        );
    }
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    let mut listed: Vec<(String, (i64, i64), Option<f64>)> = town
        .r#where()?
        .of("climbing")
        .iter()
        .map(|item| (item.kind.clone(), item.cell, item.height))
        .collect();
    listed.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    assert_eq!(
        listed,
        [
            ("vine down".to_string(), (100, 163), Some(145.0)),
            ("vine up".to_string(), (100, 162), Some(0.0)),
        ]
    );
    Ok(())
}

/// Spike 238: on Route 9 the Rhyhorn smashes rocks with A, and while the player rode it the list
/// said "Rhyhorn, here", its model left where it was got on.
#[test]
fn test_a_rock_the_rhyhorn_smashes_is_an_obstacle_and_the_ridden_rhyhorn_is_not_listed()
-> Result<()> {
    let mut tiles: Vec<((i64, i64), i64)> = (90..100).map(|x| ((x, 172), 0x31000020)).collect();
    tiles.extend((90..100).map(|x| ((x, 173), 0x32000025)));
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            hidden: Some(dex_nav),
            landforms: [
                gen6(),
                vec![Arc::new(route_9(&[0x200D])) as Arc<dyn Landform>],
            ]
            .concat(),
            ..game()
        },
    );
    town.block_tiles(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &border(160), &tiles);
    town.model(4, TOWN, 5, 0x100F, (95, 172));
    town.model(5, TOWN, 6, 0x200D, (97, 173));
    let on_foot = town.r#where()?;
    let obstacles: Vec<i64> = on_foot
        .of("obstacles")
        .iter()
        .map(|item| item.code)
        .collect();
    assert_eq!(obstacles, [0x200D]);
    assert!(on_foot.of("people").iter().any(|item| item.code == 0x100F));
    town.memory.write(POOL + 6, &[0x21, 0x03]);
    assert!(
        !town
            .r#where()?
            .of("people")
            .iter()
            .any(|item| item.code == 0x100F)
    );
    Ok(())
}

/// Spike 208: the sea's currents carry a player surfing to where they let go, and Orion asked
/// for them in the list, as a teleporter is with where it comes out (2026-10-13). Those that let
/// the player go within `LANDING_REACH` of each other are one current, at its cell nearest the
/// player, with where that cell lets them go.
#[test]
fn test_currents_that_let_the_player_go_round_one_place_are_one_listed_where_they_start()
-> Result<()> {
    let mut tiles: Vec<((i64, i64), i64)> = (100..106).map(|x| ((x, 165), 0x951A0002)).collect();
    tiles.extend((100..104).map(|x| ((x, 167), 0x951A0002)));
    tiles.extend((84..90).map(|x| ((x, 175), 0x941A0002)));
    let mut town = Town::new();
    town.block_tiles(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &border(160), &tiles);
    let currents = town.r#where()?.of("currents");
    let seen: Vec<(String, (i64, i64), Option<(i64, i64)>)> = currents
        .iter()
        .map(|item| (item.kind.clone(), item.cell, item.arrival))
        .collect();
    assert_eq!(
        seen,
        [
            ("current".to_string(), (89, 175), Some((90, 175))),
            ("current".to_string(), (100, 167), Some((99, 167))),
        ]
    );
    let sources: HashSet<i64> = currents.iter().map(|item| item.source).collect();
    assert_eq!(sources.len(), 2);
    assert!(town.listing()?.contains(
        &"currents: current 5 west 3 south out at 4 west 3 south; current 6 east 5 north out at \
          5 east 5 north"
            .to_string()
    ));
    Ok(())
}

/// Spike 265: Lysandre Labs' arrow tiles push the player their way until the way is shut, an arrow
/// crossed turning the push its own way. Each is listed under arrows by its way, with where its
/// push stops, and the ground there pushes.
#[test]
fn test_each_arrow_is_listed_by_its_way_with_where_its_push_sets_the_player_down() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            landforms: [
                gen6(),
                vec![Arc::new(ArrowTiles {
                    ways: IndexMap::from(xy::ARROWS),
                }) as Arc<dyn Landform>],
            ]
            .concat(),
            ..game()
        },
    );
    let mut blocked = border(160);
    blocked.extend([(104, 165), (107, 175)]);
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &blocked,
        &[
            ((100, 170), 0x76000020),
            ((104, 170), 0x78000020),
            ((110, 175), 0x77000020),
        ],
    );
    let r#where = town.r#where()?;
    let mut seen: Vec<(String, (i64, i64), Option<(i64, i64)>)> = r#where
        .of("arrows")
        .iter()
        .map(|item| (item.kind.clone(), item.cell, item.arrival))
        .collect();
    seen.sort();
    assert_eq!(
        seen,
        [
            ("arrow east".to_string(), (100, 170), Some((104, 166))),
            ("arrow north".to_string(), (104, 170), Some((104, 166))),
            ("arrow west".to_string(), (110, 175), Some((108, 175))),
        ]
    );
    assert_eq!(
        town.reader.ground(&r#where)?.arrow((110, 175))?,
        Some(pathfind::WEST)
    );
    Ok(())
}

/// Spike 165: under Route 126 the player's light rose into a pond walled off from the sea, and
/// only their other light reached a dive spot into the way to Sootopolis City. The sea above is
/// the zone of the same place that is not under the water, its matrix out of the ROM, laid over
/// this one where every cell of deep water lands on deep water.
#[test]
fn test_under_the_water_each_light_is_joined_to_those_the_sea_above_reaches() -> Result<()> {
    fn square(x: i64, z: i64) -> Vec<(i64, i64)> {
        (0..3)
            .flat_map(|dx| (0..3).map(move |dz| (x + dx, z + dz)))
            .collect()
    }

    let lights = [square(84, 165), square(100, 180), square(110, 165)];
    let town = Town::with(false, 5, 7);
    let mut files = vec![NO_CHUNK_FILE; 64];
    files[4 * 8 + 2] = 7;
    town.memory.write(MAP + MAP_CHUNK_FILES, &le16(&files));
    let mut under = town_info();
    put(&mut under, INFO_KIND, &le16(&[UNDERWATER]));
    let data = zone_file(&ZoneSpec {
        info: &under,
        ..ZoneSpec::default()
    });
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    town.memory.write(TABLE + TOWN * INFO_SIZE, &under);
    let mut sea = vec![0u8; INFO_SIZE as usize];
    put(&mut sea, INFO_PARENT, &le16(&[0x37, 170]));
    put(&mut sea, INFO_MATRIX, &le16(&[28]));
    town.memory.write(TABLE + 0x37 * INFO_SIZE, &sea);
    let deep = 0x411A0006;
    let tiles: Vec<((i64, i64), i64)> = lights.iter().flatten().map(|&cell| (cell, deep)).collect();
    let below = mapblock::cells(&chunk_file_with((80, 160), &[], &tiles, 7))?
        .expect("a chunk's cell table");
    // The sea above is one chunk, 80 west and 160 north, with rock between the first light
    // and the other two.
    let rock: Vec<(i64, i64)> = (0..CHUNK).map(|z| (15, z)).collect();
    let moved: Vec<((i64, i64), i64)> = lights
        .iter()
        .flatten()
        .map(|&(x, z)| ((x - 80, z - 160), deep))
        .collect();
    let above =
        mapblock::cells(&chunk_file_with((0, 0), &rock, &moved, 7))?.expect("a chunk's cell table");
    let tables: HashMap<i64, Vec<i64>> = HashMap::from([(7, below), (8, above)]);
    let reader = field_reader(
        town.memory.clone(),
        Given {
            far: Some(Rc::new(move |file| Ok(tables.get(&file).cloned()))),
            matrices: Some(Rc::new(|number| {
                Ok(if number == 28 {
                    Some((1, vec![8]))
                } else {
                    None
                })
            })),
            ..Given::default()
        },
        game(),
    );
    reader.track(&town.found())?;
    let r#where = reader.poll()?.expect("the player's field");
    let r#where = reader.interactables(&r#where)?;
    let surfaces = reader.ground(&r#where)?.surfaces;
    let set = |cells: &[(i64, i64)]| -> HashSet<(i64, i64)> { cells.iter().copied().collect() };
    assert_eq!(*surfaces[&(85, 166)], set(&lights[0]));
    let both = set(&[lights[1].clone(), lights[2].clone()].concat());
    assert_eq!(*surfaces[&(101, 181)], both);
    assert_eq!(*surfaces[&(110, 165)], both);
    Ok(())
}

#[test]
fn test_a_slope_patch_with_no_cell_at_its_corner_is_one_slope() -> Result<()> {
    let slope: Vec<((i64, i64), i64)> = [(85, 165), (85, 166), (84, 166)]
        .into_iter()
        .map(|cell| (cell, 0x6C000020))
        .collect();
    let mut town = Town::new();
    town.block_tiles(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &border(160), &slope);
    let seen: Vec<(String, (i64, i64), (i64, i64))> = town
        .r#where()?
        .of("bike")
        .into_iter()
        .map(|item| (item.kind, item.cell, item.size))
        .collect();
    assert_eq!(seen, [("slope".to_string(), (84, 165), (2, 2))]);
    Ok(())
}

#[test]
fn test_a_use_tile_in_another_zone_of_the_loaded_chunks_is_not_listed() -> Result<()> {
    let mut town = Town::new();
    town.block_tiles(
        ROUTE_BLOCK,
        ROUTE_CHUNK,
        (80, 120),
        &border(159),
        &[((96, 150), PC_TILE)],
    );
    let kinds: Vec<String> = town
        .r#where()?
        .of("signs")
        .iter()
        .map(|item| item.kind.clone())
        .collect();
    assert_eq!(kinds, ["sign", "object"]);
    Ok(())
}

/// No zone's data has the berry spots, only the ROM's table of them (spike 141): the player's
/// zone's are listed, each a place of its own by its place in the table.
#[test]
fn test_a_berry_spot_is_listed_under_its_own_category_in_its_own_zone() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            berries: vec![(ROUTE, (90, 150)), (TOWN, (97, 175)), (TOWN, (96, 175))],
            ..Given::default()
        },
        Game {
            features: vec![BerrySpots::of],
            ..game()
        },
    );
    let r#where = town.r#where()?;
    let berries = r#where.of("berries");
    assert_eq!(
        berries
            .iter()
            .map(|item| (item.kind.as_str(), item.cell))
            .collect::<Vec<_>>(),
        [("berry spot", (96, 175)), ("berry spot", (97, 175))]
    );
    assert_eq!(
        berries.iter().map(|item| item.source).collect::<Vec<_>>(),
        [
            place_source(BERRY_SOURCE, 2)?,
            place_source(BERRY_SOURCE, 1)?
        ]
    );
    let lines: Vec<String> = describe(&r#where, None, None).collect::<Result<_>>()?;
    assert!(
        lines
            .contains(&"berries: berry spot 2 east 3 south; berry spot 3 east 3 south".to_string())
    );
    Ok(())
}

/// Route 113's piles are a list its own module writes, a state a pile in the save: 2 set out,
/// 3 walked into, 1 collected (spike 155). The module's image in the heap carries the sequence's
/// vtable too, and is no sequence; nor is one started in another zone.
#[test]
fn test_ash_piles_are_listed_under_their_own_category_until_walked_into() -> Result<()> {
    let mut town = Town::new();
    let (sequence, image, states, piles) = (0x082D5B30, 0x0819338C, 0x08C6BC9C, 0x08196804);
    town.memory.u32(sequence, &[0x00843060]);
    town.memory.u32(sequence + ash::STATES, &[states]);
    town.memory.u32(sequence + ash::COUNT, &[3, piles]);
    town.memory.u16(sequence + ash::ZONE, TOWN);
    town.memory.write(states, &[2, 0, 1]);
    for (n, (x, z)) in [(97, 175), (100, 165), (90, 170)].into_iter().enumerate() {
        town.memory.write(
            piles + n as i64 * 12,
            &f32s(&[(x * 18 + 9) as f64, 38.0, (z * 18 + 9) as f64]),
        );
    }
    town.memory
        .u32(image, &[0x00843060, 0xE59F1014, 0xE92D4010]);
    let reader = ash::AshPiles::new(town.memory.clone());
    reader.track(&Found {
        pages: pages(&town.memory),
        heap: IndexMap::from([
            (sequence, ash::CLASS.to_string()),
            (image, ash::CLASS.to_string()),
        ]),
        partial: true,
    })?;
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            features: vec![given_feature(Rc::new(reader))],
            ..game()
        },
    );
    assert!(
        town.listing()?
            .contains(&"ash: ash pile 3 east 3 south; ash pile 6 east 7 north".to_string())
    );
    assert_eq!(
        town.r#where()?
            .of("ash")
            .iter()
            .map(|item| item.source)
            .collect::<Vec<_>>(),
        [place_source(ASH_SOURCE, 0)?, place_source(ASH_SOURCE, 1)?]
    );
    town.memory.write(states, &[3]);
    assert_eq!(
        town.r#where()?
            .of("ash")
            .iter()
            .map(|item| item.cell)
            .collect::<Vec<_>>(),
        [(100, 165)]
    );
    town.memory.u16(sequence + ash::ZONE, ROUTE);
    assert!(town.r#where()?.of("ash").is_empty());
    Ok(())
}

#[test]
fn test_record_sizes_are_what_the_game_stores() {
    assert_eq!(
        (FURNITURE, NPC, WARP, ENTITY_HEADER),
        (0x14, 0x30, 0x18, 12)
    );
    assert_eq!((BLOCK, MAP_MARK, mmodel::IN_USE), (10, 0xC, 4));
    assert_eq!(
        (MAP_CHUNKS_WIDE, MAP_CHUNKS_TALL, MAP_GRID),
        (0x36, 0x38, 0x3C)
    );
    assert_eq!(
        (
            hidden::SPECIES,
            hidden::LEVEL,
            hidden::WORLD_X,
            hidden::CORE,
            hidden::SLOT
        ),
        (4, 8, 0x8C, 0xA4, 0xAE)
    );
    assert_eq!(
        (
            mapblock::CENTRE_X,
            mapblock::CENTRE_Z,
            mapblock::ARC,
            mapblock::FILE,
            mapblock::SIZE,
            mapblock::STATE,
            mapblock::READY
        ),
        (0x0C, 0x14, 0x18, 0x24, 0x90, 0xA8, 4)
    );
    assert_eq!((CHUNK, BLOCKED), (40, 1));
}

/// The low ten bits of the zone row's +0x1c are the place-name index, the same for Littleroot
/// and the house in it; the bits above are flags, set on the zone that carries the name (spike
/// 85).
#[test]
fn test_a_zone_and_every_building_in_it_share_a_place() -> Result<()> {
    let town = Town::new();
    town.reader.track(&town.found())?;
    town.memory
        .u16(TABLE + TOWN * INFO_SIZE + INFO_NAME, 0x400 | 170);
    assert_eq!(town.reader.zones.place_index(TOWN)?, Some(170));
    assert_eq!(town.reader.zones.place_index(HOUSE)?, Some(170));
    assert_eq!(town.reader.zones.place_index(ROUTE)?, Some(204));
    Ok(())
}

#[test]
fn test_no_place_without_the_zone_table() -> Result<()> {
    assert_eq!(Town::new().reader.zones.place_index(TOWN)?, None);
    Ok(())
}

/// The table's row was in place as the field started and nothing pointed at its start yet:
/// every exit was said without its place, and a path as stairs, until the scheduled sweep 30 s
/// later (omega_ruby_ledges_3.ctm, 2026-09-28).
#[test]
fn test_a_field_without_its_zone_table_asks_for_sweeps_until_one_finds_it() -> Result<()> {
    let now = Rc::new(StdCell::new(0.0));
    let mut town = Town::new();
    let clock = now.clone();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            clock: Some(Rc::new(move || clock.get())),
            ..Given::default()
        },
        game(),
    );
    town.memory.u32(TABLE_POINTER, &[0, 0]);
    assert!(town.reader.sweep_wanted(true)?.is_none());
    town.r#where()?;
    assert_eq!(town.reader.zones.place_index(TOWN)?, None);
    assert!(town.reader.sweep_wanted(true)?.is_some());
    assert!(town.reader.sweep_wanted(true)?.is_none());
    now.set(now.get() + TABLE_EVERY);
    assert!(town.reader.sweep_wanted(true)?.is_some());
    town.memory
        .u32(TABLE_POINTER, &[TABLE + TOWN * INFO_SIZE, TABLE]);
    town.r#where()?;
    now.set(now.get() + TABLE_EVERY);
    assert!(town.reader.sweep_wanted(true)?.is_none());
    assert!(town.reader.zones.place_index(TOWN)?.is_some());
    Ok(())
}

/// Continued from a save in Fortree City, no word a sweep read pointed at the table's start
/// in ten sweeps, and every path out of town was said as a passage with no place
/// (omega_ruby_tree-city-regression.ctm, 2026-10-06). The game's table is the ROM's byte for
/// byte.
#[test]
fn test_a_zone_table_nothing_points_at_is_found_by_the_roms_own_bytes() -> Result<()> {
    let mut town = Town::new();
    town.memory.u32(TABLE_POINTER, &[0, 0]);
    let rom = town.memory.read(TABLE, (BEDROOM + 1) * INFO_SIZE)?;
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            zone_table: rom,
            ..Given::default()
        },
        Game {
            hidden: Some(dex_nav),
            ..game()
        },
    );
    town.r#where()?;
    assert_eq!(town.reader.zones.place_index(ROUTE)?, Some(204));
    Ok(())
}

#[test]
fn test_a_zone_table_no_sweep_finds_is_left_to_the_schedule() -> Result<()> {
    let now = Rc::new(StdCell::new(0.0));
    let mut town = Town::new();
    let clock = now.clone();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            clock: Some(Rc::new(move || clock.get())),
            ..Given::default()
        },
        game(),
    );
    town.memory.u32(TABLE_POINTER, &[0, 0]);
    town.r#where()?;
    for _ in 0..TABLE_ASKS {
        assert!(town.reader.sweep_wanted(true)?.is_some());
        now.set(now.get() + TABLE_EVERY);
    }
    assert!(town.reader.sweep_wanted(true)?.is_none());
    Ok(())
}

/// Spike 89: the rival's model reads 0x7a at +0x10 in both openings, and the sprite a script
/// filled it with at +0x898 — May's as the boy, Brendan's as the girl. A work object whose looks
/// are not in yet is its own id, and any other model is its +0x10 as it always was.
///
/// One nobody filled is drawn as nothing, and is neither listed nor in the way: the ten in the
/// Contest Spectacular Halls' lobbies were "person", and the player walked onto one
/// (or_contest_hall_phantom_people).
#[test]
fn test_a_work_object_is_whoever_it_was_filled_with() -> Result<()> {
    let town = Town::new();
    let rival = town.model(1, TOWN, 3, 0x7A, (104, 179));
    let codes = |town: &Town| -> Result<Vec<i64>> {
        Ok(town
            .r#where()?
            .of("people")
            .iter()
            .map(|item| item.code)
            .collect())
    };
    town.memory.u32(rival + mmodel::DRAWN, &[0x13A]);
    assert_eq!(codes(&town)?, [0x13A]);
    town.memory.u32(rival + mmodel::DRAWN, &[0x7A]);
    let r#where = town.r#where()?;
    assert!(r#where.of("people").is_empty());
    assert!(!town.reader.ground(&r#where)?.standing.contains(&(104, 179)));
    town.model(1, TOWN, 3, 0x119, (104, 179));
    town.memory.u32(rival + mmodel::DRAWN, &[0x13B]);
    assert_eq!(codes(&town)?, [0x119]);
    Ok(())
}

/// X keeps the sprite a model is drawn with at +0x890, and a pointer where ORAS keeps it
/// (spike 100).
#[test]
fn test_a_work_object_is_read_where_the_game_keeps_its_sprite() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            drawn: xy::DRAWN,
            ..game()
        },
    );
    let rival = town.model(1, TOWN, 3, 0x7A, (104, 179));
    town.memory.u32(rival + mmodel::DRAWN, &[0x08286CF4]);
    town.memory.u32(rival + xy::DRAWN, &[0x13A]);
    let codes: Vec<i64> = town
        .r#where()?
        .of("people")
        .iter()
        .map(|item| item.code)
        .collect();
    assert_eq!(codes, [0x13A]);
    Ok(())
}

/// Spike 221: Mom's Rhyhorn outside the house in Vaniville is work object 0x7b, whose +0x890
/// held 0x106, the model `RIDE_POKE01_EX`, and the list said "person". A person's model is its
/// sprite, a thing's is MONSBALL's 0xab on from 0x2000, a Pokémon's `EV_POKE01`'s 0xf2 on from
/// 0x1000 (spikes 100 and 218).
#[test]
fn test_an_x_work_object_holds_the_model_it_was_filled_with_not_its_sprite() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            drawn: xy::DRAWN,
            models: Some(xy::MODELS),
            ..game()
        },
    );
    let rhyhorn = town.model(1, TOWN, 3, 0x7B, (104, 179));
    let mut codes: Vec<i64> = Vec::new();
    for model in [0x106, 0x13, 0xAC] {
        town.memory.u32(rhyhorn + xy::DRAWN, &[model]);
        codes.extend(
            town.r#where()?
                .interactables
                .iter()
                .filter(|item| item.cell == (104, 179))
                .map(|item| item.code),
        );
    }
    assert_eq!(codes, [0x1014, 0x13, 0x2001]);
    Ok(())
}

/// Spike 100: X's zone file has one section fewer, and its info and entity table lead as
/// ORAS's do. Refused, it left X with no map at all.
#[test]
fn test_a_zone_file_of_x_s_four_sections_is_read_as_oras_s_five() -> Result<()> {
    assert_eq!(Town::with(true, 4, 7).listing()?, Town::new().listing()?);
    Ok(())
}

/// Spike 100: X's chunk file has one section fewer, and the same cell table in the first.
#[test]
fn test_a_chunk_of_x_s_six_sections_narrows_an_edge_as_oras_s_seven() -> Result<()> {
    let x = Town::with(true, 5, 6);
    assert_eq!(x.listing()?, Town::new().listing()?);
    assert_eq!(x.reader.chunks_loaded()?.len(), 2);
    Ok(())
}

/// Spike 100: in X the current zone's row ran from one page into the next, and the word before
/// the table's pointer led to the row of the zone the player had come from. The town's row here
/// moves to a table whose town row crosses a page, and the pointer is left beside the house's
/// row, which is a building's of a kind of its own.
#[test]
fn test_the_zone_table_is_found_by_its_start_from_a_row_across_two_pages() -> Result<()> {
    let town = Town::new();
    let table = 0x08D40000 - 0x10 - TOWN * INFO_SIZE;
    for zone in [TOWN, ROUTE, HOUSE] {
        town.memory.write(
            table + zone * INFO_SIZE,
            &town.memory.read(TABLE + zone * INFO_SIZE, INFO_SIZE)?,
        );
        town.memory
            .write(TABLE + zone * INFO_SIZE, &vec![0u8; INFO_SIZE as usize]);
    }
    town.memory.u16(table + HOUSE * INFO_SIZE + INFO_KIND, 0x10);
    town.memory
        .u32(TABLE_POINTER, &[table + HOUSE * INFO_SIZE, table]);
    town.reader.track(&town.found())?;
    assert_eq!(town.reader.zones.outdoors(TOWN)?, Some(true));
    assert_eq!(town.reader.zones.outdoors(HOUSE)?, Some(false));
    Ok(())
}

/// A replay presses no keys, so the log is where a door's name from the ROM is seen.
#[test]
fn test_the_listing_says_what_a_named_thing_is_called() -> Result<()> {
    let town = Town::new();
    let r#where = town.r#where()?;
    let called = |item: &Interactable| -> Result<Option<String>> {
        Ok(if item.kind == "door" && item.zone == Some(HOUSE) {
            Some("Pokémon Center".to_string())
        } else {
            None
        })
    };
    let exits = describe(&r#where, Some(&called), None).next().unwrap()?;
    assert!(
        exits
            .starts_with("exits: door to zone 0xdf \"Pokémon Center\" 1 north; door to zone 0xe1 ")
    );
    Ok(())
}

/// Spike 90: an NPC record's +0x0a is 3000 plus the trainer's id, the flag the game sets once
/// they are beaten, and anything else there is not a trainer.
#[test]
fn test_a_trainer_is_read_off_the_flag_its_record_holds() -> Result<()> {
    let data = zone_file(&ZoneSpec {
        npcs: &[
            (0xDA, 1, 1, 0, Some(3007)),
            (0xDB, 2, 2, 0, Some(3010)),
            (0x119, 3, 3, 0x227, None),
            (0xDA, 4, 4, 0, Some(3000 + 999)),
        ],
        ..ZoneSpec::default()
    });
    assert_eq!(npc_trainers(&data, 950)?, [(0xDA, 7), (0xDB, 10)]);
    assert_eq!(npc_trainers(b"not a zone", 950)?, []);
    Ok(())
}

/// A door and a sign are numbered in tables of their own, so a warp's slot carries a tag of
/// its own, and a name given to one is never the other's (2026-09-23).
#[test]
fn test_a_warp_is_numbered_apart_from_the_furniture() -> Result<()> {
    let r#where = Town::new().r#where()?;
    let doors: Vec<i64> = r#where
        .of("exits")
        .iter()
        .filter(|item| item.kind == "door")
        .map(|item| item.source)
        .collect();
    let signs: Vec<i64> = r#where
        .of("signs")
        .iter()
        .filter(|item| item.kind == "sign")
        .map(|item| item.source)
        .collect();
    assert!(
        !doors.is_empty()
            && doors
                .iter()
                .all(|&source| source_kind(source) == WARP_SOURCE)
    );
    assert!(
        !signs.is_empty()
            && signs
                .iter()
                .all(|&source| source_kind(source) == FURNITURE_SOURCE)
    );
    Ok(())
}

/// Each kind of place numbers its own from zero, so the key tells them apart by a tag of the
/// kind's own, and no two places on one map share a source (2026-09-23).
#[test]
fn test_every_kind_of_place_on_one_map_is_numbered_apart() -> Result<()> {
    let mut town = Town::new();
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border(160),
        &[((95, 170), PC_TILE)],
    );
    let r#where = town.r#where()?;
    let places: Vec<Interactable> = ["exits", "signs"]
        .iter()
        .flat_map(|category| r#where.of(category))
        .filter(|item| !item.move_model)
        .collect();
    let kinds: HashMap<&str, Option<i64>> = places
        .iter()
        .map(|item| (item.kind.as_str(), item.place_kind()))
        .collect();
    assert_eq!(
        kinds,
        HashMap::from([
            ("door", Some(WARP_SOURCE)),
            ("edge", Some(EDGE_SOURCE)),
            ("object", Some(TILE_SOURCE)),
            ("sign", Some(FURNITURE_SOURCE)),
        ])
    );
    let sources: HashSet<i64> = places.iter().map(|item| item.source).collect();
    assert_eq!(sources.len(), places.len());
    let edge = places.iter().find(|item| item.kind == "edge").unwrap();
    assert_eq!(source_number(edge.source), ROUTE << 16);
    Ok(())
}

/// Until 2026-09-23 an edge was `zone << 16 | gap` and a use tile `1 << 24 | x << 12 | z`,
/// so an edge into the Safari Zone (0x18e) at gap 0 and a tile whose first cell is (2272, 0)
/// were the same number. Under tags of their own they are not.
#[test]
fn test_the_edge_and_the_tile_that_were_one_slot_are_two() -> Result<()> {
    // `zone << 16 | gap` at gap 0, and `x << 12 | z` at z 0.
    let (edge, tile) = (0x18E << 16, 2272 << 12);
    assert_eq!(1 << 24 | tile, edge);
    assert_ne!(
        place_source(EDGE_SOURCE, edge)?,
        place_source(TILE_SOURCE, tile)?
    );
    assert_eq!(source_number(place_source(EDGE_SOURCE, edge)?), edge);
    Ok(())
}

/// The tag sits above every number a u16 of the game's can make — a zone shifted by 16 at
/// most — so one that reaches it is an encoding written too wide, refused rather than keyed as
/// another kind's place.
#[test]
fn test_a_place_whose_number_reaches_the_tag_is_refused() -> Result<()> {
    assert_eq!(
        place_source(EDGE_SOURCE, 0xFFFE << 16 | 0xFFFF)?,
        EDGE_SOURCE | 0xFFFE << 16 | 0xFFFF
    );
    let error: Error = place_source(EDGE_SOURCE, 1 << 32).unwrap_err();
    assert!(error.is(Kind::ValueError));
    Ok(())
}

/// Out of the ROM's own lines for it (`archives.PC_LINES`), in the player's language; the
/// other use tiles have no word in the game's text, and stay objects.
#[test]
fn test_the_pc_is_called_what_the_game_calls_it() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            words: IndexMap::from([(PC_KIND, "パソコン".to_string())]),
            ..Given::default()
        },
        game(),
    );
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border(160),
        &[((95, 170), PC_TILE), ((100, 165), SHELF_TILE)],
    );
    let kinds: Vec<String> = town
        .r#where()?
        .of("signs")
        .iter()
        .map(|item| item.kind.clone())
        .collect();
    assert_eq!(kinds, ["パソコン", "sign", "object", "object"]);
    Ok(())
}

/// Spike 102: on Lumiose's boulevards a model's +0x94 is a rail's id and +0x9c a distance
/// along it, which read as a cell put the player at x = 0; where it is in the world is at
/// +0x114 on a rail and on the grid alike.
#[test]
fn test_a_model_on_a_rail_is_where_the_world_has_it() -> Result<()> {
    let town = Town::new();
    let at = POOL + MODEL_STRIDE;
    town.memory.u32(at + mmodel::CELL_X, &[0x0504D991]);
    town.memory.write(at + mmodel::CELL_Z, &f32s(&[23.0]));
    town.memory
        .write(at + mmodel::PLACE_X, &f32s(&[0.0, 0.0, 23.0 * 18.0]));
    let cells: Vec<(i64, i64)> = town
        .r#where()?
        .of("people")
        .iter()
        .map(|item| item.cell)
        .collect();
    assert_eq!(cells, [(109, 176)]);
    Ok(())
}

/// A model built but not yet placed is at the world's origin, which is no cell's centre.
#[test]
fn test_a_model_the_game_has_not_placed_is_not_listed() -> Result<()> {
    let town = Town::new();
    let at = POOL + MODEL_STRIDE;
    for x in [mmodel::PLACE_X, mmodel::WORLD_X] {
        town.memory.write(at + x, &f32s(&[0.0, 0.0, 0.0]));
    }
    assert_eq!(town.r#where()?.of("people"), []);
    Ok(())
}

/// Spike 102: a record placed on a rail holds a rail's id where its cell would be, which
/// read as one put Lumiose's signs 44,000 cells away. It is left out until rails are read,
/// and the records after it keep their slots, which is what a door's name is keyed by.
#[test]
fn test_a_sign_or_a_warp_on_a_rail_is_not_listed_and_keeps_its_slot() -> Result<()> {
    let town = Town::new();
    let data = zone_file(&ZoneSpec {
        furniture: &[(0x13, 97, 171), (0x14, 0xD991, 0x0504)],
        warps: &[
            (HOUSE, WARP_DOOR, 94, 171, None, None),
            (0xE1, WARP_DOOR, 106, 171, None, None),
        ],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    let first = zone_header(5) + INFO_SIZE + ENTITY_HEADER;
    let mut rail = data.clone();
    put(&mut rail, first + FURNITURE + 6, &le16(&[1]));
    put(&mut rail, first + 2 * FURNITURE + 6, &le16(&[1]));
    town.memory.write(FILE, &rail);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (rail.len() as i64) << 8]);
    let r#where = town.r#where()?;
    let signs: Vec<i64> = r#where
        .of("signs")
        .iter()
        .filter(|item| item.kind == "sign")
        .map(|item| item.source)
        .collect();
    assert_eq!(signs, [place_source(FURNITURE_SOURCE, 0)?]);
    let doors: Vec<i64> = r#where
        .of("exits")
        .iter()
        .filter(|item| item.kind == "door")
        .map(|item| source_number(item.source))
        .collect();
    assert_eq!(doors, [1]);
    Ok(())
}

/// The town's reader given the ROM's streets as `line` alone, its ends where the world has
/// `start` and `end`, and a map of chunk files 700 and 701 with no chunks to read.
fn on_one_line(
    town: &mut Town,
    line: &rail::Line,
    start: (f64, f64),
    end: (f64, f64),
    cameras: i64,
) {
    on_one_line_under(town, line, start, end, cameras, game().camera);
}

/// `on_one_line`, the game's camera `shape`'s, as Python's tests give `camera=camera.X`.
fn on_one_line_under(
    town: &mut Town,
    line: &rail::Line,
    start: (f64, f64),
    end: (f64, f64),
    cameras: i64,
    shape: camera::Shape,
) {
    let network = rail::Network {
        lines: IndexMap::from([(line.id, line.clone())]),
        places: IndexMap::from([(1, start), (2, end)]),
        cameras,
        ..Default::default()
    };
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            streets: Some(Rc::new(move |_chunks: &[i64]| Ok(network.clone()))),
            far: Some(Rc::new(|_file| Ok(None))),
            ..Given::default()
        },
        Game {
            camera: shape,
            ..game()
        },
    );
    town.memory
        .write(MAP + MAP_CHUNK_FILES, &le16(&[0, 700, 701]));
}

/// Route 7's way onto Route 6 is placed on a short street of its own, which the player walks
/// onto from the grid, and no exit led to Route 6 until they had (x_route-7and6.ctm,
/// 2026-10-19). Off the rails it is placed from where the ROM has its line's two ends: on a line
/// running south, an offset across rises to the east.
#[test]
fn test_a_warp_on_a_rail_is_listed_off_the_rails_where_the_world_has_its_line() -> Result<()> {
    let line = rail::Line::new(0x8001, 1, 2, 4, 3, vec![0; 15]);
    let start = ((100 * 18 + 9) as f64, (160 * 18 + 9) as f64);
    let mut town = Town::new();
    on_one_line(
        &mut town,
        &line,
        start,
        (start.0, start.1 + (4 * 18) as f64),
        0,
    );
    let mut data = zone_file(&ZoneSpec {
        warps: &[(ROUTE, WARP_DOOR, 0, 0, None, None)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    let first = zone_header(5) + INFO_SIZE + ENTITY_HEADER;
    put(
        &mut data,
        first + 6,
        &[le16(&[1]), le32(&[line.id]), le16(&[2, -1, 1, 3])].concat(),
    );
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    let doors: Vec<((i64, i64), (i64, i64))> = town
        .r#where()?
        .of("exits")
        .iter()
        .filter(|item| item.kind == "door")
        .map(|item| (item.cell, item.size))
        .collect();
    assert_eq!(doors, [((99, 162), (3, 1))]);
    Ok(())
}

/// The chunks under Route 7's street onto Route 6 mark every cell a wall, and from the grid
/// P found nothing to walk to, "no path to path" (2026-10-19). Off the rails the street's open
/// cells are open ground, its walls still walls: here its row 1.
#[test]
fn test_a_street_beside_the_grid_is_walked_where_the_world_has_it() -> Result<()> {
    let walls: Vec<i64> = [vec![0; 3], vec![1; 3], vec![0; 9]].concat();
    let line = rail::Line::new(0x8001, 1, 2, 4, 3, walls);
    let start = ((100 * 18 + 9) as f64, (160 * 18 + 9) as f64);
    let mut town = Town::new();
    on_one_line(
        &mut town,
        &line,
        start,
        (start.0, start.1 + (4 * 18) as f64),
        0,
    );
    let walled: Vec<(i64, i64)> = (97..104)
        .flat_map(|x| (158..166).map(move |z| (x, z)))
        .collect();
    town.block(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &walled);
    let ground = town.reader.ground(&town.r#where()?)?;
    for x in [99, 100, 101] {
        for z in [160, 162, 163, 164] {
            assert!(ground.open((x, z))?, "({x}, {z})");
        }
        assert!(!ground.open((x, 161))?, "({x}, 161)");
    }
    assert!(!ground.open((98, 162))? && !ground.open((102, 162))?);
    Ok(())
}

/// Route 7's street onto Route 6 is shown by a camera of its own that stays put, and up
/// walks the street's way nearest north; laid out as one of Lumiose's, P said "camera angled,
/// hold left, hold right, 10 back" over and over, and only the street was listed
/// (x_route-6-2.ctm, 2026-10-20). Where the map's streets place cameras, a player on one is
/// where the world has them, on the grid, with the whole zone round them.
#[test]
fn test_under_a_streets_own_camera_the_player_is_on_the_grid() -> Result<()> {
    let line = rail::Line::new(0x8001, 1, 2, 4, 3, vec![0; 15]);
    let start = ((100 * 18 + 9) as f64, (160 * 18 + 9) as f64);
    let mut town = Town::new();
    on_one_line_under(
        &mut town,
        &line,
        start,
        (start.0, start.1 + (4 * 18) as f64),
        1,
        camera::X,
    );
    town.memory.u32(POOL + mmodel::CELL_X, &[line.id]);
    town.memory.write(POOL + mmodel::PLACE_X, &f32s(&[0.0]));
    let r#where = town.r#where()?;
    assert!(!r#where.on_rail && r#where.cell == (94, 172));
    let kinds: Vec<String> = r#where
        .of("exits")
        .into_iter()
        .map(|item| item.kind)
        .collect();
    assert_eq!(kinds, ["door", "door", "edge"]);
    // Route 7's camera looks north, and the compass is the world's.
    let block = camera_of(&town, &camera::X)?;
    town.memory
        .write(block + camera::X.looks, &f32s(&[std::f64::consts::PI]));
    assert_eq!(town.reader.turn(0)?, 0);
    Ok(())
}

/// Spike 251: round Prism Tower the camera looks east, and right walked the player south, left
/// north, down west and up east, on a street running north measured east; P said "18 north" for
/// the way west and the list "19 west" for the exit there. The compass turns with the camera,
/// and a street's pairs of keys are the screen's: left and right along a street running up the
/// world's north, which is across the screen.
#[test]
fn test_under_a_streets_own_camera_turned_east_the_keys_are_the_screens() -> Result<()> {
    let line = rail::Line {
        way: rail::WAY_EAST,
        ..rail::Line::new(0x8001, 1, 2, 4, 3, vec![0; 15])
    };
    let start = ((100 * 18 + 9) as f64, (176 * 18 + 9) as f64);
    let mut town = Town::new();
    on_one_line_under(
        &mut town,
        &line,
        start,
        (start.0, start.1 - (4 * 18) as f64),
        1,
        camera::X,
    );
    town.memory.u32(POOL + mmodel::CELL_X, &[line.id]);
    town.memory.write(POOL + mmodel::PLACE_X, &f32s(&[0.0]));
    let r#where = town.r#where()?;
    let block = camera_of(&town, &camera::X)?;
    town.memory.write(
        block + camera::X.looks,
        &f32s(&[std::f64::consts::PI / 2.0]),
    );
    let turn = town.reader.turn(0)?;
    assert_eq!(turn, 1);
    let ground = town.reader.ground(&r#where)?;
    pathfind::walked(turn);
    let said: IndexMap<String, (i64, i64)> = [(100, 173), (100, 175), (99, 174), (101, 174)]
        .into_iter()
        .map(|step| {
            (
                pathfind::way(ground.key((100, 174), step).unwrap_or("")),
                step,
            )
        })
        .collect();
    pathfind::walked(0);
    assert_eq!(
        said,
        IndexMap::from([
            ("west".to_string(), (100, 173)),
            ("east".to_string(), (100, 175)),
            ("south".to_string(), (99, 174)),
            ("north".to_string(), (101, 174)),
        ])
    );
    assert_eq!(town.reader.facing(None)?, Some(1));
    Ok(())
}

/// A poll asked the rails whether the player's street was under a camera of its own with the
/// chunk files the last sweep read, and where they were on it with the files the field map lists
/// now: from a map change until the next sweep each poll took both maps' streets in from the ROM,
/// and looked the line up in the map it is not of (#18).
#[test]
fn test_a_map_changed_since_the_sweep_has_one_maps_streets_read_until_the_next() -> Result<()> {
    let line = rail::Line::new(0x8001, 1, 2, 4, 3, vec![0; 15]);
    let network = rail::Network {
        lines: IndexMap::from([(line.id, line.clone())]),
        places: IndexMap::from([(1, (0.0, 0.0)), (2, (0.0, (4 * 18) as f64))]),
        cameras: 0,
        ..Default::default()
    };
    let asked: Rc<RefCell<Vec<Vec<i64>>>> = Rc::new(RefCell::new(Vec::new()));
    let mut town = Town::new();
    let heard = asked.clone();
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            streets: Some(Rc::new(move |chunks: &[i64]| {
                heard.borrow_mut().push(chunks.to_vec());
                Ok(network.clone())
            })),
            far: Some(Rc::new(|_file| Ok(None))),
            ..Given::default()
        },
        game(),
    );
    town.memory
        .write(MAP + MAP_CHUNK_FILES, &le16(&[0, 700, 701]));
    town.memory.u32(POOL + mmodel::CELL_X, &[line.id]);
    town.memory.write(POOL + mmodel::PLACE_X, &f32s(&[0.0]));
    town.reader.track(&town.found())?;
    town.memory
        .write(MAP + MAP_CHUNK_FILES, &le16(&[0, 800, 801]));
    town.reader.poll()?;
    town.reader.poll()?;
    assert_eq!(*asked.borrow(), [vec![700, 701]]);
    Ok(())
}

/// Up walks Route 7's street north-west, a cell of the street at a time, never straight
/// north or west, and P's "7 north-west" was walked by holding north (the tapes of
/// x_route-6-2.ctm and x_route-6-3.ctm, 2026-10-20). Laid on the grid, a street's cells let the
/// player step the street's ways, a diagonal among them, and a way along it is one leg, said by
/// its key: up and down along a line measured north or south.
#[test]
fn test_a_diagonal_street_is_one_leg_said_by_the_key_that_walks_it() -> Result<()> {
    let line = rail::Line::new(0x8001, 1, 2, 6, 3, vec![0; 21]);
    let start = ((100 * 18 + 9) as f64, (170 * 18 + 9) as f64);
    let end = (
        start.0 - (6 * 18) as f64 / 2f64.sqrt(),
        start.1 - (6 * 18) as f64 / 2f64.sqrt(),
    );
    let mut town = Town::new();
    on_one_line(&mut town, &line, start, end, 1);
    town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (100, 170));
    let r#where = town.r#where()?;
    let ground = town.reader.ground(&r#where)?;
    let path = ground.find(
        (100, 170),
        &IndexMap::from([((96, 166), None)]),
        "",
        None,
        false,
        None,
    )?;
    let path = pathfind::cells(&path.expect("a way along the street"));
    assert_eq!(
        pathfind::spoken(&pathfind::legs(&path, None)?),
        "4 north-west"
    );
    let key = |cell, step| ground.key(cell, step);
    assert_eq!(
        pathfind::spoken(&pathfind::legs_with(&path, None, Some(&key), None)?),
        "4 north"
    );
    assert_eq!(ground.key((100, 170), (101, 169)), Some("east"));
    Ok(())
}

/// Round Prism Tower `L` went back and forth between two exits (Khai, 2026-10-27): a warp
/// across a diagonal street covers cells touching only at their corners, and joining warps by
/// their cells listed it once a cell, each under its one key, which the list walks by.
#[test]
fn test_a_warp_across_a_diagonal_street_is_listed_once() -> Result<()> {
    let line = rail::Line::new(0x8001, 1, 2, 6, 3, vec![0; 21]);
    let start = ((100 * 18 + 9) as f64, (170 * 18 + 9) as f64);
    let end = (
        start.0 - (6 * 18) as f64 / 2f64.sqrt(),
        start.1 - (6 * 18) as f64 / 2f64.sqrt(),
    );
    let mut town = Town::new();
    on_one_line(&mut town, &line, start, end, 1);
    town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (100, 170));
    let mut data = zone_file(&ZoneSpec {
        warps: &[(0x56, 0x0703, 0, 0, None, None)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    let first = zone_header(5) + INFO_SIZE + ENTITY_HEADER;
    put(
        &mut data,
        first + 6,
        &[le16(&[1]), le32(&[line.id]), le16(&[3, -1, 1, 3])].concat(),
    );
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    let warps: Vec<Interactable> = town
        .r#where()?
        .of("exits")
        .into_iter()
        .filter(|item| item.zone == Some(0x56))
        .collect();
    let cells = if warps.len() == 1 {
        warps[0].cells()
    } else {
        Vec::new()
    };
    let xs: HashSet<i64> = cells.iter().map(|cell| cell.0).collect();
    let zs: HashSet<i64> = cells.iter().map(|cell| cell.1).collect();
    assert!(cells.len() > 1 && xs.len() == zs.len() && zs.len() == cells.len());
    Ok(())
}

/// Spike 102: on a rail +0xec is 0, where on the grid it is the world's x; the chunks there
/// mark every cell a wall, so the border is listed whole rather than narrowed away.
#[test]
fn test_a_player_on_a_rail_is_said_to_be_and_the_edges_are_not_narrowed() -> Result<()> {
    let mut town = Town::new();
    let walled: Vec<(i64, i64)> = (80..120)
        .flat_map(|x| (160..200).map(move |z| (x, z)))
        .collect();
    town.block(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &walled);
    let player = POOL;
    town.memory.write(player + mmodel::PLACE_X, &f32s(&[0.0]));
    let r#where = town.r#where()?;
    assert!(r#where.on_rail);
    let sizes: Vec<(i64, i64)> = r#where
        .of("exits")
        .iter()
        .filter(|item| item.kind == "edge")
        .map(|item| item.size)
        .collect();
    assert_eq!(sizes, [(40, 1)]);
    Ok(())
}

/// On the grid +0x94 is the cell a step heads for, which guiding says each turn by as the
/// step begins (spike 92); halfway there the world has it between the two.
#[test]
fn test_a_step_in_progress_is_in_the_cell_it_is_heading_for() -> Result<()> {
    let town = Town::new();
    let at = town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (95, 172));
    for x in [mmodel::PLACE_X, mmodel::WORLD_X] {
        town.memory.write(
            at + x,
            &f32s(&[94.5 * 18.0 + 9.0, 0.0, (172 * 18 + 9) as f64]),
        );
    }
    town.reader.track(&town.found())?;
    let r#where = town.reader.poll()?;
    assert!(r#where.is_some_and(|r#where| r#where.cell == (95, 172) && !r#where.on_rail));
    Ok(())
}

/// Spikes 102 and 103: on Lumiose's boulevards the player, the people and the doors are placed
/// on rails, and the walls are the rails' own. With the rails loaded, each is where the
/// corridor puts it, laid out the way the stick moves the player there.
#[test]
fn test_on_the_rails_everything_is_in_the_corridor_and_its_walls_are_the_ground() -> Result<()> {
    use test_rail::{A, B, network};

    let town = Town::of(Game {
        camera: camera::X,
        ..game()
    });
    network(&town.memory);
    let player = town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (94, 172));
    town.memory.u32(player + mmodel::CELL_X, &[B]);
    town.memory.write(player + mmodel::CELL_Z, &f32s(&[2.0]));
    town.memory
        .write(player + mmodel::PLACE_X, &f32s(&[0.0, 0.0, 36.0, 18.0]));
    let person = town.model(1, TOWN, 0, 0x119, (109, 176));
    town.memory.u32(person + mmodel::CELL_X, &[A]);
    town.memory.write(person + mmodel::CELL_Z, &f32s(&[7.0]));
    town.memory
        .write(person + mmodel::PLACE_X, &f32s(&[0.0, 0.0, 126.0, -36.0]));
    for slot in [2, 3] {
        town.memory
            .u32(POOL + slot * MODEL_STRIDE + mmodel::IN_USE, &[0]);
    }
    let door = zone_file(&ZoneSpec {
        warps: &[(0x56, 0x0703, 0, 0, None, None)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    let first = zone_header(5) + INFO_SIZE + ENTITY_HEADER;
    let mut placed = door;
    put(
        &mut placed,
        first + 6,
        &[le16(&[1]), le32(&[A]), le16(&[5, -3, 4, 1])].concat(),
    );
    town.memory.write(FILE, &placed);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (placed.len() as i64) << 8]);
    town.memory
        .write(player + mmodel::FACING_SIN, &f32s(&[-1.0, 0.0]));
    let r#where = town.r#where()?;
    assert!(r#where.on_rail && r#where.cell == (-1, -2) && r#where.heading == Some("east"));
    let exits = r#where.of("exits");
    assert_eq!(
        exits
            .iter()
            .map(|item| (item.kind.as_str(), item.zone, item.cell, item.size))
            .collect::<Vec<_>>(),
        [("door", Some(0x56), (3, 2), (1, 4))]
    );
    assert_eq!(
        r#where
            .of("people")
            .iter()
            .map(|item| item.cell)
            .collect::<Vec<_>>(),
        [(2, 3)]
    );
    let ground = town.reader.ground(&r#where)?;
    assert!(ground.open((0, 0))? && !ground.open((3, 0))? && !ground.open((2, 3))?);
    Ok(())
}

/// Spike 107: through a warp into zone 0x44 the player stood at -5 along and 8 across, the
/// line's frame turned round, and was taken to be 5 cells before the line's start, on no
/// street at all, so no way was found to anything.
#[test]
fn test_a_player_kept_in_the_line_turned_round_is_turned_back() -> Result<()> {
    use test_rail::{B, network};

    let town = Town::new();
    network(&town.memory);
    town.memory.u32(
        test_rail::FILE + 0x5580 + rail::LINE_WAY,
        &[rail::WAY_NORTH],
    );
    let player = town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (94, 172));
    town.memory.u32(player + mmodel::CELL_X, &[B]);
    town.memory.write(player + mmodel::CELL_Z, &f32s(&[-2.0]));
    town.memory
        .write(player + mmodel::PLACE_X, &f32s(&[0.0, 0.0, -36.0, 36.0]));
    town.memory
        .write(player + mmodel::FACING_SIN, &f32s(&[0.0, 0.0, -1.0]));
    let r#where = town.r#where()?;
    assert!(r#where.on_rail && r#where.cell == (2, -2) && r#where.heading == Some("north"));
    assert!(town.reader.ground(&r#where)?.open(r#where.cell)?);
    Ok(())
}

/// Spike 105: down walks the player back and turns them round, and leaves the camera, which
/// is what up walks by. The camera's heading is the world's, turned into the line's frame by
/// the way the line runs where the player stands.
#[test]
fn test_on_the_rails_up_walks_the_way_the_camera_looks_not_the_way_the_player_faces() -> Result<()>
{
    use test_rail::{B, network};

    let town = Town::of(Game {
        camera: camera::X,
        ..game()
    });
    network(&town.memory);
    let player = town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (94, 172));
    town.memory.u32(player + mmodel::CELL_X, &[B]);
    town.memory.write(player + mmodel::CELL_Z, &f32s(&[2.0]));
    town.memory
        .write(player + mmodel::PLACE_X, &f32s(&[0.0, 0.0, 36.0, 0.0]));
    town.memory
        .write(player + mmodel::FACING_SIN, &f32s(&[0.0, 0.0, -1.0]));
    let way = 124.0f64.to_radians();
    town.memory.write(
        player + mmodel::LINE_SIN,
        &f32s(&[way.sin(), 0.0, way.cos()]),
    );
    let block = camera_of(&town, &camera::X)?;
    for (yaw, heading, bearing) in [(127.0f64, "north", 357), (-146.0, "west", 270)] {
        town.memory.write(block + X_YAW, &f32s(&[yaw.to_radians()]));
        let r#where = town.reader.poll()?.expect("a field");
        assert_eq!(r#where.heading, Some(heading));
        assert_eq!(py::round(r#where.bearing.unwrap_or(0.0))?, bearing);
    }
    Ok(())
}

const X_YAW: i64 = match camera::X.yaw {
    Some(yaw) => yaw,
    None => 0,
};

/// A camera block of `shape`'s game, found by the town's reader through a camera object.
fn camera_of(town: &Town, shape: &camera::Shape) -> Result<i64> {
    let (block, holder) = (0x08277140, 0x0827855C);
    town.memory.u32(block, &[ALLOCATION, shape.size]);
    town.memory.u32(
        holder,
        &[FakeClasses::new(town.memory.clone()).add(camera::CLASS)],
    );
    town.memory.u32(holder + camera::BLOCK, &[block]);
    let found = town.found();
    let mut heap = found.heap.clone();
    heap.insert(holder, camera::CLASS.to_string());
    town.reader.track(&Found {
        pages: found.pages.clone(),
        heap,
        partial: false,
    })?;
    Ok(block)
}

/// Spike 230: Parfum Palace's garden, where the camera looks west, a little south, and up
/// walked Khai west; it swings round to north as he walks up the garden, past the edge of a
/// quarter only (`pad.HYSTERESIS`).
#[test]
fn test_off_the_rails_up_walks_the_way_the_camera_looks() -> Result<()> {
    let town = Town::of(Game {
        camera: camera::X,
        ..game()
    });
    town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (94, 172));
    let block = camera_of(&town, &camera::X)?;
    let mut turn = 0;
    for (looks, quarter) in [(-70.0f64, 3), (-130.0, 3), (-166.8, 0), (180.0, 0)] {
        town.memory
            .write(block + camera::X.looks, &f32s(&[looks.to_radians()]));
        turn = town.reader.turn(turn)?;
        assert_eq!(turn, quarter, "{looks}");
    }
    town.memory.u32(block, &[0, 0]);
    assert_eq!(town.reader.turn(3)?, 3);
    Ok(())
}

/// The Battle Resort's beach: Omega Ruby's camera looks east there, and its block is 16 bytes
/// smaller than X's, so it was never found; P said "5 north" and up walked the player east,
/// into a hut's counter (or_battle_resort_pathfinding_glitch). ORAS keeps no heading behind the
/// player, so nothing is said from one.
#[test]
fn test_oras_camera_turns_the_compass_too() -> Result<()> {
    let town = Town::new();
    town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (94, 172));
    let block = camera_of(&town, &camera::ORAS)?;
    town.memory.write(
        block + camera::ORAS.looks,
        &f32s(&[std::f64::consts::PI / 2.0]),
    );
    town.memory.write(block + X_YAW, &f32s(&[1.0]));
    assert_eq!(town.reader.turn(0)?, 1);
    let r#where = town.reader.poll()?.expect("a poll");
    assert!(r#where.heading.is_none() && !r#where.turning);
    town.memory
        .write(block + camera::ORAS.looks, &f32s(&[-std::f64::consts::PI]));
    assert_eq!(town.reader.turn(1)?, 0);
    town.memory.u32(block, &[ALLOCATION, camera::X.size]);
    assert_eq!(town.reader.turn(1)?, 1);
    Ok(())
}

/// Spike 239: in Glittering Cave `LOOKS` reads 0, which was taken for south, so P said "22
/// south" for 22 north, and `YAW`, the heading on a rail, holds where the camera looks: north,
/// 7.7 degrees west. The ways are said from that heading, and the compass is not turned.
#[test]
fn test_under_a_camera_behind_the_player_off_the_rails_the_ways_are_said_from_it() -> Result<()> {
    let town = Town::of(Game {
        camera: camera::X,
        ..game()
    });
    town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (94, 172));
    let block = camera_of(&town, &camera::X)?;
    town.memory
        .write(block + X_YAW, &f32s(&[(-172.3f64).to_radians()]));
    town.memory.write(block + camera::X.looks, &f32s(&[0.0]));
    let r#where = town.reader.poll()?;
    let r#where = r#where.expect("a poll");
    assert!(r#where.heading == Some("north") && r#where.turning);
    assert_eq!(py::round_to(r#where.bearing.unwrap_or(0.0), 1), 352.3);
    assert_eq!(town.reader.turn(2)?, 0);
    Ok(())
}

/// The game moves the table's entries about as lines load, and a read now and then misses
/// one: the corridor was laid out afresh from the player's own line, every cell moved under the
/// way being walked, and the door on the missed line went from the list (spike 105).
#[test]
fn test_a_line_the_table_misses_for_a_poll_stays_in_the_corridor() -> Result<()> {
    use test_rail::{A, B, TABLE, network};

    let town = Town::new();
    network(&town.memory);
    let player = town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (94, 172));
    town.memory.u32(player + mmodel::CELL_X, &[A]);
    town.memory.write(player + mmodel::CELL_Z, &f32s(&[9.0]));
    town.memory
        .write(player + mmodel::PLACE_X, &f32s(&[0.0, 0.0, 162.0, 0.0]));
    let door = zone_file(&ZoneSpec {
        warps: &[(0x56, 0x0703, 0, 0, None, None)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    let first = zone_header(5) + INFO_SIZE + ENTITY_HEADER;
    let mut placed = door;
    put(
        &mut placed,
        first + 6,
        &[le16(&[1]), le32(&[B]), le16(&[1, 0, 1, 1])].concat(),
    );
    town.memory.write(FILE, &placed);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (placed.len() as i64) << 8]);
    town.reader.track(&town.found())?;
    let before = town.reader.poll()?.expect("a field");
    town.memory.u32(TABLE + 8 + 0x20 + 16 + 4, &[0]);
    let after = town.reader.poll()?.expect("a field");
    assert_eq!(after.cell, before.cell);
    let cells: Vec<(i64, i64)> = town
        .reader
        .interactables(&after)?
        .of("exits")
        .iter()
        .map(|item| item.cell)
        .collect();
    assert_eq!(cells, [(0, -11)]);
    Ok(())
}

/// Spike 106: the game loads a line only with the chunks round the player, so the Pokémon
/// Lab's door, on a line two further down the boulevard, was on no line pa3ds knew. The field
/// map lists each chunk's file, and the ROM has every chunk's streets.
#[test]
fn test_a_door_on_a_line_the_game_has_not_loaded_is_listed_from_the_rom() -> Result<()> {
    use test_rail::{ACROSS, B, C, P2, P3, TABLE, network, walls};

    let mut town = Town::new();
    network(&town.memory);
    town.memory.u32(TABLE + 8 + 0x20 + 2 * 16 + 4, &[0]);
    let asked: Rc<RefCell<Vec<Vec<i64>>>> = Rc::new(RefCell::new(Vec::new()));
    let unloaded = rail::Line::new(C, P3, P2, 5, ACROSS, walls(5, &[]));
    let record = asked.clone();
    let streets = move |chunks: &[i64]| -> Result<rail::Network> {
        record.borrow_mut().push(chunks.to_vec());
        Ok(rail::Network {
            lines: if chunks.contains(&700) {
                IndexMap::from([(C, unloaded.clone())])
            } else {
                IndexMap::new()
            },
            places: IndexMap::new(),
            cameras: 0,
            ..Default::default()
        })
    };
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            streets: Some(Rc::new(streets)),
            ..Given::default()
        },
        game(),
    );
    town.memory
        .write(MAP + MAP_CHUNK_FILES, &le16(&[0, 700, 701]));
    let player = town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (94, 172));
    town.memory.u32(player + mmodel::CELL_X, &[B]);
    town.memory.write(player + mmodel::CELL_Z, &f32s(&[2.0]));
    town.memory
        .write(player + mmodel::PLACE_X, &f32s(&[0.0, 0.0, 36.0, 0.0]));
    let door = zone_file(&ZoneSpec {
        warps: &[(0x53, 0x0703, 0, 0, None, None)],
        info: &town_info(),
        ..ZoneSpec::default()
    });
    let first = zone_header(5) + INFO_SIZE + ENTITY_HEADER;
    let mut placed = door;
    put(
        &mut placed,
        first + 6,
        &[le16(&[1]), le32(&[C]), le16(&[1, 0, 1, 1])].concat(),
    );
    town.memory.write(FILE, &placed);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (placed.len() as i64) << 8]);
    let r#where = town.r#where()?;
    let exits: Vec<(Option<i64>, (i64, i64))> = r#where
        .of("exits")
        .iter()
        .map(|item| (item.zone, item.cell))
        .collect();
    assert_eq!(exits, [(Some(0x53), (0, -10))]);
    town.reader.poll()?;
    assert_eq!(*asked.borrow(), [vec![700, 701]]);
    Ok(())
}

/// Spike 107: South Boulevard is two zones, and from the Lab Café Soleil, in the other, was
/// not listed. The ROM's records of every zone of the map the player is on are listed with
/// the zone's own, each named by its zone; a zone of another map is not, its line ids its own.
#[test]
fn test_on_a_street_the_other_zones_of_its_map_are_listed_by_their_own_zone() -> Result<()> {
    use test_rail::{A, B, C, network};

    let (other, elsewhere) = (0x40, 0x44);
    let mut town_row = town_info();
    put(&mut town_row, INFO_MATRIX, &le16(&[128]));
    let mut town = Town::new();
    network(&town.memory);
    town.memory.write(TABLE + TOWN * INFO_SIZE, &town_row);
    for (zone, matrix) in [(other, 128), (elsewhere, 130)] {
        let mut row = vec![0u8; INFO_SIZE as usize];
        put(&mut row, INFO_MATRIX, &le16(&[matrix]));
        put(&mut row, INFO_PARENT, &le16(&[0x3F, 170]));
        town.memory.write(TABLE + zone * INFO_SIZE, &row);
    }

    let door_on = |line: i64, along: i64, to: i64| -> Vec<u8> {
        [
            le16(&[to, 0, 0x0703, 1]),
            le32(&[line]),
            le16(&[along, -3, 1, 1]),
            vec![0; 4],
        ]
        .concat()
    };

    town.reader = field_reader(
        town.memory.clone(),
        Given {
            street_records: IndexMap::from([
                (other, (Vec::new(), vec![door_on(A, 5, 0x7D)])),
                (elsewhere, (Vec::new(), vec![door_on(C, 1, 0x59)])),
            ]),
            ..Given::default()
        },
        game(),
    );
    let player = town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (94, 172));
    town.memory.u32(player + mmodel::CELL_X, &[B]);
    town.memory.write(player + mmodel::CELL_Z, &f32s(&[2.0]));
    town.memory
        .write(player + mmodel::PLACE_X, &f32s(&[0.0, 0.0, 36.0, 0.0]));
    let mut data = zone_file(&ZoneSpec {
        warps: &[(0x53, 0x0703, 0, 0, None, None)],
        info: &town_row,
        ..ZoneSpec::default()
    });
    let first = zone_header(5) + INFO_SIZE + ENTITY_HEADER;
    put(
        &mut data,
        first + 6,
        &[le16(&[1]), le32(&[B]), le16(&[1, -3, 1, 1])].concat(),
    );
    town.memory.write(FILE, &data);
    town.memory
        .u32(HOLDER + 8, &[0x11 | (data.len() as i64) << 8]);
    let exits = town.r#where()?.of("exits");
    assert_eq!(
        exits
            .iter()
            .map(|item| (item.zone, item.home, item.naming(TOWN)))
            .collect::<Vec<_>>(),
        [
            (
                Some(0x53),
                Some(TOWN),
                Some(("place", TOWN, exits[0].source))
            ),
            (
                Some(0x7D),
                Some(other),
                Some(("place", other, exits[1].source))
            ),
        ]
    );
    assert!(exits[0].key() != exits[1].key() && exits[0].source == exits[1].source);
    Ok(())
}

/// A way is found in the corridor's cells, so they must not move under the player walking
/// from one line onto the next: the corridor stays laid out from the line it first was.
#[test]
fn test_the_corridor_stays_laid_out_from_one_line_as_the_player_walks_onto_the_next() -> Result<()>
{
    use test_rail::{A, B, network};

    let town = Town::of(Game {
        camera: camera::X,
        ..game()
    });
    network(&town.memory);
    let player = town.model(0, TOWN, mmodel::PLAYER_INDEX, 0xAC, (94, 172));
    town.memory.u32(player + mmodel::CELL_X, &[B]);
    town.memory.write(player + mmodel::CELL_Z, &f32s(&[2.0]));
    town.memory
        .write(player + mmodel::PLACE_X, &f32s(&[0.0, 0.0, 36.0, 0.0]));
    town.reader.track(&town.found())?;
    let before = town.reader.poll()?;
    town.memory.u32(player + mmodel::CELL_X, &[A]);
    town.memory.write(player + mmodel::CELL_Z, &f32s(&[9.0]));
    let after = town.reader.poll()?;
    let (before, after) = (before.expect("a field"), after.expect("a field"));
    assert_eq!((before.cell, after.cell), ((0, -2), (0, 1)));
    Ok(())
}

/// Such a sweep finds the new block alone; the rest are kept while they still carry a map
/// block's vtable (spike 110).
#[test]
fn test_a_sweep_of_new_blocks_adds_map_blocks_and_keeps_those_still_map_blocks() -> Result<()> {
    let mut town = Town::new();
    town.reader.track(&town.found())?;
    assert_eq!(town.reader.chunks_loaded()?.len(), 2);
    let new = TOWN_BLOCK + 0x1000;
    town.block(new, TOWN_CHUNK + 0x100100, (120, 160), &border(160));
    town.memory.u32(ROUTE_BLOCK, &[0]);
    town.reader.track(&Found {
        pages: pages(&town.memory),
        heap: IndexMap::from([(new, mapblock::CLASS.to_string())]),
        partial: true,
    })?;
    assert_eq!(town.reader.chunks_loaded()?.len(), 2);
    Ok(())
}

/// A game's Secret Bases as `oras.field.secret_base` reads them: every spot, and the player's
/// own; inside the town, standing for a base, what stands where and the decorating cursor's
/// cell.
pub(crate) struct Bases {
    pub spots: Vec<(i64, (i64, i64))>,
    own: Option<i64>,
    standing: IndexMap<(i64, i64), Vec<i64>>,
    solid: IndexSet<(i64, i64)>,
    /// The cursor's cell, or None while it is not shown.
    pub at: StdCell<Option<(i64, i64)>>,
    /// Whether the item held can be set down at the cursor, None for nothing held.
    pub fits: StdCell<Option<bool>>,
    placed: Vec<(i64, Vec<(i64, i64)>)>,
}

impl Bases {
    fn new(
        spots: &[(i64, (i64, i64))],
        own: Option<i64>,
        standing: &[((i64, i64), Vec<i64>)],
        solid: &[(i64, i64)],
    ) -> Rc<Bases> {
        Bases::placing(spots, own, standing, solid, &[])
    }

    /// `Bases(spots, own, standing, solid, placed)`.
    fn placing(
        spots: &[(i64, (i64, i64))],
        own: Option<i64>,
        standing: &[((i64, i64), Vec<i64>)],
        solid: &[(i64, i64)],
        placed: &[(i64, Vec<(i64, i64)>)],
    ) -> Rc<Bases> {
        Rc::new(Bases {
            spots: spots.to_vec(),
            own,
            standing: standing.iter().cloned().collect(),
            solid: solid.iter().copied().collect(),
            at: StdCell::new(None),
            fits: StdCell::new(None),
            placed: placed.to_vec(),
        })
    }
}

/// Python's `Bases` subclasses `SecretBases`, and keeps its `classes`, `track` and `listed`;
/// here `listed` is the same function the real reader's is.
impl Tracked for Bases {
    fn classes(&self) -> Vec<&'static str> {
        vec![secret_base::SAVE_CLASS]
    }

    fn track(&self, _found: &Found) -> Result<()> {
        Ok(())
    }
}

impl Feature for Bases {
    fn listed(&self, zone: i64, _zone_at: ZoneAt) -> Result<Vec<Interactable>> {
        secret_base::spots_in(&self.spots, self.own, secret_base::INSIDE, zone)
    }
}

impl SecretBaseReader for Bases {
    fn zones(&self) -> IndexSet<i64> {
        IndexSet::from([TOWN])
    }

    fn furnished(&self) -> Result<bool> {
        Ok(true)
    }

    fn cursor(&self) -> Result<Option<((i64, i64), Option<bool>)>> {
        Ok(self.at.get().map(|at| (at, self.fits.get())))
    }

    fn standing(&self, cell: (i64, i64)) -> Result<Vec<i64>> {
        Ok(self.standing.get(&cell).cloned().unwrap_or_default())
    }

    fn solid(&self) -> Result<IndexSet<(i64, i64)>> {
        Ok(self.solid.clone())
    }

    fn placed(&self) -> Result<Vec<(i64, Vec<(i64, i64)>)>> {
        Ok(self.placed.clone())
    }
}

/// No zone's data has a spot a Secret Base can be made at, only the executable's table of them
/// (spike 154): Route 111's tree, the player's base, was not in the list. The player's own is its
/// entrance, into the place inside; the rest are spots.
#[test]
fn test_secret_base_spots_are_exits_and_the_players_own_leads_inside() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            secret_bases: Some(given_bases(Bases::new(
                &[(ROUTE, (90, 150)), (TOWN, (97, 175)), (TOWN, (100, 165))],
                Some(1),
                &[],
                &[],
            ))),
            ..game()
        },
    );
    let exits: Vec<Interactable> = town
        .r#where()?
        .of("exits")
        .into_iter()
        .filter(|item| item.place_kind() == Some(SECRET_BASE_SOURCE))
        .collect();
    assert_eq!(
        exits
            .iter()
            .map(|item| (item.kind.as_str(), item.cell, item.zone, item.source))
            .collect::<Vec<_>>(),
        [
            (
                "entrance",
                (97, 175),
                Some(secret_base::INSIDE),
                place_source(SECRET_BASE_SOURCE, 1)?
            ),
            (
                "secret base spot",
                (100, 165),
                None,
                place_source(SECRET_BASE_SOURCE, 2)?
            ),
        ]
    );
    Ok(())
}

/// Inside a Secret Base the PC and its flag were two signs, and the trigger the player arrives
/// on, the way out, was not listed (spike 154). The PC is called what the game calls one.
#[test]
fn test_a_secret_bases_pc_flag_and_way_out_are_listed_as_what_they_are() -> Result<()> {
    let mut town = Town::new();
    town.memory.write(
        FILE,
        &zone_file(&ZoneSpec {
            info: &town_info(),
            triggers: &[(oras::WAY_OUT, 94, 173, 1, None, None)],
            furniture: &[(oras::BASE_FLAG, 97, 171), (oras::BASE_PC, 96, 171)],
            ..ZoneSpec::default()
        }),
    );
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            words: IndexMap::from([(PC_KIND, "パソコン".to_string())]),
            ..Given::default()
        },
        Game {
            triggers: IndexMap::from([(
                (TOWN, oras::WAY_OUT),
                ("exits".to_string(), "way out".to_string(), None),
            )]),
            furniture: IndexMap::from([
                ((TOWN, oras::BASE_PC), FurnitureKind::Tile(PC_KIND)),
                (
                    (TOWN, oras::BASE_FLAG),
                    FurnitureKind::Word("flag".to_string()),
                ),
            ]),
            ..game()
        },
    );
    let r#where = town.r#where()?;
    let mut signs: Vec<(String, (i64, i64))> = r#where
        .of("signs")
        .iter()
        .filter(|item| item.place_kind() == Some(FURNITURE_SOURCE))
        .map(|item| (item.kind.clone(), item.cell))
        .collect();
    signs.sort();
    assert_eq!(
        signs,
        [
            ("flag".to_string(), (97, 171)),
            ("パソコン".to_string(), (96, 171))
        ]
    );
    let ways: Vec<(String, (i64, i64))> = r#where
        .of("exits")
        .iter()
        .filter(|item| item.kind == "way out")
        .map(|item| (item.kind.clone(), item.cell))
        .collect();
    assert_eq!(ways, [("way out".to_string(), (94, 173))]);
    Ok(())
}

/// A Secret Base keeps the six slots of the player's secret team in the walls of its far
/// corner while they are empty, and they were listed as six people with no way to them (spike
/// 154). One with an open cell beside it is still somebody. The slots are work objects nobody
/// filled, left out for that as well now; these are people, walled in or not.
#[test]
fn test_a_model_walled_in_on_every_side_is_not_on_the_map() -> Result<()> {
    let mut town = Town::new();
    let walled: Vec<(i64, i64)> = border(160)
        .into_iter()
        .chain((110..113).flat_map(|x| (176..179).map(move |z| (x, z))))
        .chain([(100, 178)])
        .collect();
    town.block(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &walled);
    town.model(4, TOWN, 3, 0x119, (111, 177));
    town.model(5, TOWN, 4, 0x11A, (100, 178));
    let people: Vec<(i64, i64)> = town
        .r#where()?
        .of("people")
        .iter()
        .map(|item| item.cell)
        .collect();
    assert!(!people.contains(&(111, 177)) && people.contains(&(100, 178)));
    Ok(())
}

/// Decorating, the cursor went over the floor, the PC and the hedge with the game saying
/// "Select location" over all three, and nothing from one empty cell to the next
/// (omega_ruby_secret-base2.ctm). A decoration is said by the game's name for it.
#[test]
fn test_the_decorating_cursor_says_what_stands_under_it() -> Result<()> {
    let mut town = Town::new();
    town.memory.write(
        FILE,
        &zone_file(&ZoneSpec {
            info: &town_info(),
            furniture: &[(oras::BASE_PC, 96, 171)],
            ..ZoneSpec::default()
        }),
    );
    town.block(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &[border(160), vec![(96, 171), (95, 170)]].concat(),
    );
    let bases = Bases::new(&[], None, &[((97, 172), vec![10])], &[]);
    let mut decorations: Vec<String> = vec!["Small Desk".to_string()];
    decorations.extend(std::iter::repeat_n(String::new(), 9));
    decorations.push("Small Chair".to_string());
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            decorations,
            tiles: Some(IndexMap::from([(PC_KIND, "PC".to_string())])),
            ..Given::default()
        },
        Game {
            secret_bases: Some(given_bases(bases.clone())),
            furniture: IndexMap::from([((TOWN, oras::BASE_PC), FurnitureKind::Tile(PC_KIND))]),
            ..game()
        },
    );
    let r#where = town.r#where()?;
    let mut said: Vec<((i64, i64), Option<((i64, i64), String)>)> = Vec::new();
    for cell in [(97, 172), (96, 171), (95, 170), (93, 172)] {
        bases.at.set(Some(cell));
        said.push((cell, town.reader.decorating(&r#where)?));
    }
    assert_eq!(
        said,
        [
            ((97, 172), Some(((97, 172), "Small Chair".to_string()))),
            ((96, 171), Some(((96, 171), "PC".to_string()))),
            ((95, 170), Some(((95, 170), "wall".to_string()))),
            ((93, 172), Some(((93, 172), "blank".to_string()))),
        ]
    );
    bases.at.set(None);
    assert_eq!(town.reader.decorating(&r#where)?, None);
    Ok(())
}

/// A crumbled cliff's cell as the ROM has it: kind 0xfd over the blocked bit.
const CLIFF_WORD: i64 = 0xFD0001A1;

fn cliff_cells() -> Vec<(i64, i64)> {
    (104..107)
        .flat_map(|x| (166..168).map(move |z| (x, z)))
        .collect()
}

fn tiles(cells: &[(i64, i64)], word: i64) -> Vec<((i64, i64), i64)> {
    cells.iter().map(|&cell| (cell, word)).collect()
}

fn bases_game(bases: Rc<Bases>) -> Game {
    Game {
        secret_bases: Some(given_bases(bases)),
        ..game()
    }
}

/// The base's listing said only "way out", "PC" and "flag", so nothing told the player where
/// the Stairs go (omega_ruby_secret-base3.ctm). The ROM's 3 by 2 patch of kind 0xfd is now one
/// exit over all six cells, and still no ground to walk on.
#[test]
fn test_a_bases_crumbled_cliff_is_listed_once_as_one_exit() -> Result<()> {
    let mut town = Town::new();
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border(160),
        &tiles(&cliff_cells(), CLIFF_WORD),
    );
    let r#where = town.r#where()?;
    let cliffs: Vec<_> = r#where
        .of("exits")
        .into_iter()
        .filter(|item| item.kind == "crumbled cliff")
        .map(|item| (item.kind, item.cell, item.size))
        .collect();
    assert_eq!(cliffs, [("crumbled cliff".to_string(), (104, 166), (3, 2))]);
    let ground = town.reader.ground(&r#where)?;
    assert!(!ground.open((105, 167))? && ground.open((103, 167))?);
    Ok(())
}

/// The cell table keeps kind 0xfd under the Stairs set down at (18, 17), and the cliff stayed
/// in the exits as "crumbled cliff, 1 of 2" (omega_ruby_secret-base4.ctm). It leaves the list
/// once a decoration stands on it.
#[test]
fn test_a_crumbled_cliff_the_stairs_stand_on_is_no_longer_listed() -> Result<()> {
    let mut town = Town::new();
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border(160),
        &tiles(&cliff_cells(), CLIFF_WORD),
    );
    let standing: Vec<_> = cliff_cells().into_iter().map(|c| (c, vec![10])).collect();
    let bases = Bases::new(&[], None, &standing, &[]);
    town.reader = field_reader(town.memory.clone(), Given::default(), bases_game(bases));
    assert!(
        town.r#where()?
            .of("exits")
            .iter()
            .all(|item| item.kind != "crumbled cliff")
    );
    Ok(())
}

/// The cursor crossing the cliff's three cells said "wall" each time, in the session before
/// omega_ruby_secret-base3.ctm. It says the cliff's name there, and still "wall" over a blocked
/// cell of any other kind.
#[test]
fn test_the_decorating_cursor_says_crumbled_cliff_over_the_cliff_and_wall_over_a_wall() -> Result<()>
{
    let mut town = Town::new();
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &[border(160), vec![(95, 170)]].concat(),
        &tiles(&cliff_cells(), CLIFF_WORD),
    );
    let bases = Bases::new(&[], None, &[], &[]);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        bases_game(bases.clone()),
    );
    let r#where = town.r#where()?;
    let mut said: Vec<((i64, i64), String)> = Vec::new();
    for cell in [(105, 167), (95, 170)] {
        bases.at.set(Some(cell));
        let found = town
            .reader
            .decorating(&r#where)?
            .expect("the cursor is shown");
        said.push((cell, found.1));
    }
    assert_eq!(
        said,
        [
            ((105, 167), "crumbled cliff".to_string()),
            ((95, 170), "wall".to_string())
        ]
    );
    Ok(())
}

const HOLE_CELLS: [(i64, i64); 2] = [(100, 172), (100, 173)];

/// Granite Cave's base has two blocked cells of kind 0xfe at (20, 12) and (20, 13), the only
/// way into its upper-left room (omega_ruby_secret-base6.ctm).
#[test]
fn test_a_bases_hole_is_an_exit_and_named_under_the_cursor_until_a_board_covers_it() -> Result<()> {
    let mut town = Town::new();
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border(160),
        &tiles(&HOLE_CELLS, 0xFE0001A1),
    );
    let bases = Bases::new(&[], None, &[], &[]);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        bases_game(bases.clone()),
    );
    let r#where = town.r#where()?;
    let holes: Vec<_> = r#where
        .of("exits")
        .into_iter()
        .filter(|item| item.kind == "hole")
        .map(|item| (item.cell, item.size))
        .collect();
    assert_eq!(holes, [((100, 172), (1, 2))]);
    bases.at.set(Some((100, 173)));
    assert_eq!(
        town.reader.decorating(&r#where)?,
        Some(((100, 173), "hole".to_string()))
    );
    let standing: Vec<_> = HOLE_CELLS.iter().map(|&c| (c, vec![10])).collect();
    let covered = Bases::new(&[], None, &standing, &[]);
    town.reader = field_reader(town.memory.clone(), Given::default(), bases_game(covered));
    assert!(
        town.r#where()?
            .of("exits")
            .iter()
            .all(|item| item.kind != "hole")
    );
    Ok(())
}

/// The player climbed the Stairs set down over the cliff, whose cells the cell table keeps
/// blocked and the mapper has as a decoration, and P said "can't continue"
/// (omega_ruby_secret-base4.ctm). A Solid Board over a hole is crossed the same way.
#[test]
fn test_p_walks_over_the_stairs_on_the_cliff_and_a_board_over_a_hole() -> Result<()> {
    let mut town = Town::new();
    let mut words = tiles(&cliff_cells(), CLIFF_WORD);
    words.extend(tiles(&HOLE_CELLS, 0xFE0001A1));
    town.block_tiles(TOWN_BLOCK, TOWN_CHUNK, (80, 160), &border(160), &words);
    let covered: Vec<(i64, i64)> = [cliff_cells(), HOLE_CELLS.to_vec()].concat();
    let standing: Vec<_> = covered.iter().map(|&c| (c, vec![10])).collect();
    let solid: Vec<(i64, i64)> = [covered.clone(), vec![(95, 172)]].concat();
    let bases = Bases::new(&[], None, &standing, &solid);
    town.reader = field_reader(town.memory.clone(), Given::default(), bases_game(bases));
    let ground = town.reader.ground(&town.r#where()?)?;
    for &cell in &covered {
        assert!(ground.open(cell)?, "{cell:?}");
    }
    assert!(!ground.open((95, 172))?);
    Ok(())
}

/// Holding the Stairs, the game drew their squares green over (18, 17), where they were set
/// down, and red elsewhere, and said "You cannot set it there!" only once A was pressed
/// (omega_ruby_secret-base5.ctm).
#[test]
fn test_a_held_item_says_first_whether_it_can_be_placed_under_the_cursor() -> Result<()> {
    let mut town = Town::new();
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border(160),
        &tiles(&cliff_cells(), CLIFF_WORD),
    );
    let bases = Bases::new(&[], None, &[], &[]);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        bases_game(bases.clone()),
    );
    let r#where = town.r#where()?;
    bases.at.set(Some((105, 167)));
    let mut said = Vec::new();
    for fits in [Some(true), Some(false), None] {
        bases.fits.set(fits);
        said.push(town.reader.decorating(&r#where)?);
    }
    assert_eq!(
        said,
        [
            Some(((105, 167), "placeable, crumbled cliff".to_string())),
            Some(((105, 167), "not placeable, crumbled cliff".to_string())),
            Some(((105, 167), "crumbled cliff".to_string())),
        ]
    );
    Ok(())
}

/// Set down, the Stairs and the desks were not in the list at all, and once listed, every
/// base's five Welcome Mats in a line were five things (the user, 2026-10-10). Those of a name
/// that touch are one; one apart is another.
#[test]
fn test_a_bases_decorations_are_signs_by_the_games_names_one_over_its_cells() -> Result<()> {
    let mut town = Town::new();
    let bases = Bases::placing(
        &[],
        None,
        &[],
        &[],
        &[
            (10, vec![(97, 172), (98, 172)]),
            (0, vec![(95, 174)]),
            (5, vec![(85, 165)]),
            (1, vec![(90, 170)]),
            (1, vec![(90, 171)]),
            (2, vec![(90, 172)]),
            (1, vec![(93, 170)]),
        ],
    );
    let mut decorations = vec!["Small Desk".to_string()];
    decorations.extend(std::iter::repeat_n("Welcome Mat".to_string(), 2));
    decorations.extend(std::iter::repeat_n(String::new(), 7));
    decorations.push("Small Chair".to_string());
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            decorations,
            ..Given::default()
        },
        bases_game(bases),
    );
    let mut found: Vec<_> = town
        .r#where()?
        .of("signs")
        .into_iter()
        .filter(|item| item.place_kind() == Some(DECORATION_SOURCE))
        .map(|item| (item.kind, item.cell, item.size))
        .collect();
    found.sort();
    assert_eq!(
        found,
        [
            ("Small Chair".to_string(), (97, 172), (2, 1)),
            ("Small Desk".to_string(), (95, 174), (1, 1)),
            ("Welcome Mat".to_string(), (90, 170), (1, 3)),
            ("Welcome Mat".to_string(), (93, 170), (1, 1)),
        ]
    );
    Ok(())
}

/// P's way to the PC went through the Small Desk, whose cell the cell table has as open floor,
/// and the player walked round it (omega_ruby_secret-base2.ctm).
#[test]
fn test_p_goes_round_a_decoration_the_cell_table_has_as_floor() -> Result<()> {
    let mut town = Town::new();
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            secret_bases: Some(given_bases(Bases::new(&[], None, &[], &[(95, 172)]))),
            ..game()
        },
    );
    let ground = town.reader.ground(&town.r#where()?)?;
    assert!(!ground.open((95, 172))? && ground.open((96, 172))?);
    Ok(())
}

/// The base's mapper keeps three layers of 40 by 40 cells, each a decoration's model and its
/// row in the save, cell (x, z) the z * 39 + x'th; the cursor keeps its cell and whether it is
/// shown (omega_ruby_secret-base2.ctm). The module's image, copied into the heap with every
/// class's vtable in it, is taken for neither.
#[test]
fn test_a_bases_decorations_and_cursor_are_read_from_its_modules_objects() -> Result<()> {
    let memory = FakeMemory::new();
    let (mapper, cursor, image, desk, mat) =
        (0x08441CCC, 0x0842EEAC, 0x0818AF08, 0x083F2198, 0x083E3A28);
    memory.u32(mapper, &[0x0084A320, secret_base::GRID, secret_base::GRID]);
    let mut cells: HashMap<i64, i64> = HashMap::new();
    for (n, &offset) in secret_base::LAYERS.iter().enumerate() {
        let layer = 0x08450000 + n as i64 * 0x4000;
        memory.u32(mapper + offset, &[layer]);
        memory.u32(layer, &[layer + 0x10, secret_base::GRID, secret_base::GRID]);
        cells.insert(offset, layer + 0x10);
    }
    let floor = cells[&secret_base::FLOOR];
    memory.u32(floor + (13 * 39 + 14) * secret_base::CELL, &[desk, 6]);
    memory.u32(floor + (12 * 39 + 18) * secret_base::CELL, &[mat, 0]);
    memory.u32(desk, &[0x0084A344, 0]);
    memory.u32(mat, &[0x0084B064, 0xAD]);
    let doll = 0x083E4000;
    memory.u32(floor + (12 * 39 + 20) * secret_base::CELL, &[doll, 1]);
    memory.u32(doll, &[0x0084C000, 0x50]);
    memory.u32(cursor, &[0x0084A3A4, 0, 1]);
    memory.write(cursor + secret_base::CURSOR_CELL, &le16(&[14, 13]));
    memory.u32(image, &[0x0084A3A4, 0xE59F1018, 0xE92D4010]);
    let bases = secret_base::SecretBases::new(memory.clone(), b"");
    bases.track(&Found {
        pages: pages(&memory),
        heap: IndexMap::from([
            (mapper, secret_base::MAPPER_CLASS.to_string()),
            (cursor, secret_base::CURSOR_CLASS.to_string()),
            (image, secret_base::CURSOR_CLASS.to_string()),
            (mat, secret_base::WALKED_ON[0].to_string()),
            (doll, secret_base::UNSEEN.to_string()),
        ]),
        partial: false,
    })?;
    assert_eq!(bases.cursor()?, Some(((14, 13), None)));
    assert!(bases.standing((14, 13))? == [0] && bases.standing((18, 12))? == [0xAD]);
    assert_eq!(
        bases.solid()?.into_iter().collect::<HashSet<_>>(),
        HashSet::from([(14, 13), (20, 12)])
    );
    assert_eq!(
        bases.placed()?,
        [(0xAD, vec![(18, 12)]), (0, vec![(14, 13)])]
    );
    memory.u32(cursor + 4, &[secret_base::FITS]);
    assert_eq!(bases.cursor()?, Some(((14, 13), Some(true))));
    memory.u32(cursor + 4, &[secret_base::REFUSED]);
    assert_eq!(bases.cursor()?, Some(((14, 13), Some(false))));
    memory.u32(cursor + secret_base::CURSOR_SHOWN, &[0]);
    assert_eq!(bases.cursor()?, None);
    Ok(())
}

/// ORAS lists its 85 spots in the executable, 20 bytes a spot numbered from 0 (spike 154); a run
/// too short to be the table is none.
#[test]
fn test_the_executables_secret_base_table_is_found_by_its_shape() -> Result<()> {
    let mut rows: Vec<u8> = Vec::new();
    for n in 0..40 {
        rows.extend(secret_base::SPOT.pack([n, 2, 8, 300 + n, 116, 0, 0x24, 0x218])?);
    }
    let mut image = vec![0x07u8; 6];
    image.extend(&rows);
    image.extend(secret_base::SPOT.pack([0, 0, 0, 0, 0, 0, 0, 0])?);
    let spots = secret_base::spots(&image);
    assert_eq!((spots.len(), spots[8]), (40, (0x24, (308, 116))));
    assert_eq!(secret_base::spots(&rows[..20 * 10]), []);
    Ok(())
}

/// The save keeps the player's base by its spot's number, 8 for Route 111's tree (spike 154),
/// and 0 while the player has none.
#[test]
fn test_the_players_own_spot_is_the_saves() -> Result<()> {
    let memory = FakeMemory::new();
    let save = 0x08C8ADA8;
    let mut rows: Vec<u8> = Vec::new();
    for n in 0..40 {
        rows.extend(secret_base::SPOT.pack([n, 0, 0, n, 1, 0, 0x24, 0x218])?);
    }
    let bases = secret_base::SecretBases::new(memory.clone(), &rows);
    memory.write(save + secret_base::OWN_SPOT, &8i16.to_le_bytes());
    assert_eq!(bases.own()?, None);
    bases.track(&Found {
        pages: pages(&memory),
        heap: IndexMap::from([(save, secret_base::SAVE_CLASS.to_string())]),
        partial: false,
    })?;
    assert_eq!(bases.own()?, Some(8));
    memory.write(save + secret_base::OWN_SPOT, &0i16.to_le_bytes());
    assert_eq!(bases.own()?, None);
    Ok(())
}

/// Spike 167: Sootopolis Gym's module, mapped whole with its sequence's vtable at +0x3310, keeps
/// in two words of its code where the cells that are no ice are and where its data is, whose first
/// three words point to the rooms: here the first, 5 by 5 from (16, 48) at 254, with (18, 52)
/// cracked as the movie's first step onto the ice left it.
#[test]
fn test_the_ice_gyms_rooms_are_read_through_its_module() -> Result<()> {
    let memory = FakeMemory::new();
    let (base, sequence, data, room, states, exits, solid) = (
        0x00838000, 0x082D376C, 0x0818EEF0, 0x0818EF10, 0x08323B40, 0x0818F000, 0x0083B020,
    );
    memory.u32(sequence, &[base + ice::VTABLE]);
    memory.u32(base + ice::SOLID_AT, &[solid, data]);
    memory.u32(solid, &vec![0; 2 * ice::NO_ICE as usize]);
    memory.u32(data, &[room, room, room]);
    let mut record = vec![5, 5, 16, 48];
    record.extend(le32(&[254, 0x4046, states, exits]));
    memory.write(room, &record);
    memory.u32(exits, &[17, 48, 18, 48, 19, 48]);
    memory.write(states + 4 * 5 + 2, &[1]);
    let floors = ice::IceFloors::new(memory.clone());
    floors.track(&Found {
        pages: pages(&memory),
        heap: IndexMap::from([(sequence, ice::CLASS.to_string())]),
        partial: false,
    })?;
    let first = floors.rooms(ice::GYM)?.remove(0);
    assert_eq!(
        (
            first.corner,
            first.size,
            first.height,
            first.exits.clone(),
            first.cleared
        ),
        (
            (16, 48),
            (5, 5),
            254.0,
            vec![(17, 48), (18, 48), (19, 48)],
            false
        )
    );
    let cracked: Vec<(i64, i64)> = first
        .cells()
        .into_iter()
        .filter(|&(_, state)| state != 0)
        .map(|(cell, _)| cell)
        .collect();
    assert_eq!(cracked, [(18, 52)]);
    assert!(floors.rooms(0x1BB)?.is_empty());
    Ok(())
}
