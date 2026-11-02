//! Sun's field: where the player is, what is around them, and the keys that walk it, in a fake laid
//! out as spike 113 read the player's bedroom.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use indexmap::IndexMap;

use super::fake_emulator::FakeEmulator;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::fake_speech::Recorder;
use crate::core::config::Ui;
use crate::core::discovery::Found;
use crate::core::memory::{self, Cache, ExactCache, Memory, RpcMemory};
use crate::core::rpc::{MAX_READ, Process, Rpc, UdpRpc};
use crate::core::rtti::ClassNames;
use crate::core::speech::Speech;
use crate::games::common::field::things::Field;
use crate::games::common::scenes::{Beat, Scene};
use crate::games::common::session::Profile;
use crate::games::sm::camera;
use crate::games::sm::field::{self, SunFieldReader, Walls};
use crate::games::sm::heaps::{self, Blocks, Heaps};
use crate::games::sm::profile::{Options, SunProfile};
use crate::games::sm::text::SunText;
use crate::games::sm::walls::AreaWalls;
use crate::games::sm::zones::{self, Area, Record, Shape, SunZones};
use crate::py::{PySet, Result};

const C: f64 = field::CELL;
const BEDROOM: i64 = 0x1A;
const LIVING_ROOM: i64 = 0x1B;
const OUTSIDE: i64 = 0x00;
const AREA: i64 = 21;
const MANAGER: i64 = 0x34197000;
const PLAYER: i64 = 0x34198180;
const MEOWTH: i64 = 0x34198DF4;
const FREE: i64 = 0x341992B8;
const STATE: i64 = 0x30000E18;
const FIELDMAP: i64 = 0x30382440;
const LOADED: i64 = 0x341C86DC;
const THING: i64 = 0x30372C30;
const WARP: i64 = 0x30372894;
const AGAIN: i64 = 0x30372D00;
const BALL: i64 = 0x3037AEF4;
const POST: i64 = 0x30372E00;
const PROP: i64 = 0x30373000;
const BODY: i64 = 0x30373200;
const BODY_SHAPE: i64 = 0x30373300;
const TALK: i64 = 0x30373400;
const TALK_SHAPE: i64 = 0x30373500;
const HERE: i64 = 0x30373600;
const HERE_SHAPE: i64 = 0x30373700;
const BODIES: i64 = 0x3036F064;
const RANGES: i64 = 0x3036F4D8;
const TRANSLATOR: i64 = 0x3000A834;
const BOX: i64 = 0x30373100;
/// Each object's block, as the sweep reads it: nothing is read outside one.
const SIZES: &[(i64, i64)] = &[
    (MANAGER, 0x24C),
    (PLAYER, 0x780),
    (MEOWTH, 0x494),
    (FREE, 0x494),
    (STATE, 0x40),
    (FIELDMAP, 0x180),
    (LOADED, 0x9C),
    (THING, 0x24),
    (WARP, 0x24),
    (AGAIN, 0x24),
    (BALL, 0x24),
    (POST, 0x24),
    (0x30372F00, 0x24),
    (0x3037B068, 0x24),
    (PROP, 0x74),
    (BOX, 0x34),
    (TRANSLATOR, 0x24),
    (BODY, 0x74),
    (BODY_SHAPE, 0x18),
    (TALK, 0x74),
    (TALK_SHAPE, 0x18),
    (HERE, 0x74),
    (HERE_SHAPE, 0x18),
];

type Span = (i64, i64);

fn size_of(at: i64) -> i64 {
    SIZES.iter().find(|entry| entry.0 == at).unwrap().1
}

fn served(memory: &FakeMemory) -> HashMap<i64, Vec<u8>> {
    let mut pages = HashMap::new();
    for (base, page) in memory.pages() {
        for at in (0..page.len()).step_by(MAX_READ as usize) {
            let end = (at + MAX_READ as usize).min(page.len());
            pages.insert(base + at as i64, page[at..end].to_vec());
        }
    }
    pages
}

/// `emu.memory.update(pages)`.
fn update(emu: &FakeEmulator, memory: &FakeMemory) {
    for (at, data) in served(memory) {
        emu.set_page(at, data);
    }
}

fn connect(emu: &FakeEmulator) -> Rc<dyn Rpc> {
    Rc::new(UdpRpc::connect(emu.port()).unwrap())
}

fn floats(memory: &FakeMemory, address: i64, values: &[f64]) {
    let data: Vec<u8> = values
        .iter()
        .flat_map(|&value| (value as f32).to_le_bytes())
        .collect();
    memory.write(address, &data);
}

fn le16(value: i64) -> Vec<u8> {
    (value as u16).to_le_bytes().to_vec()
}

fn le32(value: i64) -> Vec<u8> {
    (value as u32).to_le_bytes().to_vec()
}

/// `struct.pack_into`: `data` at `at` in `buffer`.
fn put(buffer: &mut [u8], at: i64, data: &[u8]) {
    buffer[at as usize..at as usize + data.len()].copy_from_slice(data);
}

fn silent() -> Rc<dyn Fn(&str)> {
    Rc::new(|_line: &str| {})
}

fn names(memory: &Rc<dyn Memory>) -> Rc<ClassNames> {
    Rc::new(ClassNames::new(memory.clone(), CODE))
}

fn place_names() -> SunText {
    let mut lines = vec![String::new(); 25];
    lines[6] = "Route 1".to_string();
    lines[24] = "Iki Town".to_string();
    SunText {
        files: IndexMap::from([(zones::PLACE_NAMES, lines)]),
    }
}

/// The bedroom's area as the ROM has it: its door, three cells wide, and the Ditto pillow;
/// zone 0 is Iki Town here, so the door says where it goes.
fn rom() -> SunZones {
    let mut rows = vec![0u8; (zones::ZONE_ROW * 0x20) as usize];
    for (zone, place) in [(BEDROOM, 6), (LIVING_ROOM, 6), (OUTSIDE, 24)] {
        put(
            &mut rows,
            zone * zones::ZONE_ROW + zones::ROW_PLACE,
            &le16(place),
        );
    }
    // The bedroom and living room stand for Route 1's two zones where a test walks between
    // them: one ground, so outdoors.
    for zone in [BEDROOM, LIVING_ROOM] {
        put(
            &mut rows,
            zone * zones::ZONE_ROW + zones::ROW_KIND,
            &le16(zones::OUTDOORS),
        );
    }
    let door = Record {
        zone: Some(BEDROOM),
        leads_to: Some(OUTSIDE),
        kind: zones::WARP_DOOR,
        ..record(
            53.5 * C,
            60.5 * C,
            Some(Shape {
                west: 52.5 * C,
                north: 60.5 * C,
                east: 54.5 * C,
                south: 60.5 * C,
            }),
        )
    };
    let pillow = Record {
        kind: 0x33,
        ..record(
            50.5 * C,
            57.5 * C,
            Some(Shape {
                west: 50.1 * C,
                north: 57.1 * C,
                east: 50.9 * C,
                south: 57.9 * C,
            }),
        )
    };
    let post = Record {
        kind: 0x12,
        ..record(47.5 * C, 57.5 * C, None)
    };
    let ball = Record {
        kind: zones::ITEM_BALL,
        code: 4,
        ..record(48.5 * C, 62.5 * C, None)
    };
    let hidden = Record {
        kind: zones::HIDDEN_ITEM,
        code: 307,
        ..record(44.5 * C, 62.5 * C, None)
    };
    let other = Record {
        zone: Some(BEDROOM),
        kind: zones::ITEM_BALL,
        code: 5,
        ..record(46.5 * C, 63.5 * C, None)
    };
    SunZones {
        rows,
        places: place_names(),
        areas: IndexMap::from([(
            AREA,
            Area {
                warps: vec![vec![door]],
                things: vec![vec![pillow, post]],
                items: vec![vec![ball, hidden, other]],
                ..Area::default()
            },
        )]),
    }
}

/// `Record(x, z, shape)`: every other field at its default.
fn record(x: f64, z: f64, shape: Option<Shape>) -> Record {
    Record {
        x,
        z,
        shape,
        ..Record::default()
    }
}

/// The player in the bedroom at cell (50, 60), the door two cells east, the pillow three north
/// and Meowth four west.
struct Bedroom {
    memory: Rc<FakeMemory>,
    vtables: HashMap<&'static str, i64>,
    found: Found,
    recorder: Rc<Recorder>,
    reads: Rc<RefCell<Vec<Span>>>,
}

fn pages_of(blocks: &[(i64, i64)]) -> IndexMap<i64, Vec<u8>> {
    blocks
        .iter()
        .map(|&(at, size)| (at, vec![0; size as usize]))
        .collect()
}

fn heap_of(found: &[(i64, &str)]) -> IndexMap<i64, String> {
    found
        .iter()
        .map(|&(at, name)| (at, name.to_string()))
        .collect()
}

impl Bedroom {
    fn new() -> Bedroom {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let vtables: HashMap<&'static str, i64> = [
            field::MANAGER,
            field::MODEL,
            field::PLAYER,
            field::STATE,
            field::THING,
            field::WARP,
            field::ITEM,
            field::PROP,
            field::BOX,
            field::CYLINDER,
            camera::CLASS,
        ]
        .into_iter()
        .map(|name| (name, classes.add(name)))
        .collect();
        memory.u32(MANAGER, &[vtables[field::MANAGER]]);
        memory.u32(MANAGER + field::MANAGER_PLAYER, &[PLAYER, MEOWTH, FREE]);
        for (model, name) in [
            (PLAYER, field::PLAYER),
            (MEOWTH, field::MODEL),
            (FREE, field::MODEL),
        ] {
            memory.u32(model, &[vtables[name]]);
        }
        memory.u32(PLAYER + field::MODEL_NODE, &[0x3090D96C]);
        floats(
            &memory,
            PLAYER + field::MODEL_PLACE,
            &[50.5 * C, 5.0, 60.5 * C],
        );
        memory.u32(MEOWTH + field::MODEL_NODE, &[0x3096D9CC]);
        floats(
            &memory,
            MEOWTH + field::MODEL_PLACE,
            &[46.5 * C, 1.0, 60.5 * C],
        );
        memory.u32(MEOWTH + field::MODEL_CODE, &[0x77]);
        memory.u32(MEOWTH + field::MODEL_SHOWN, &[1]);
        memory.u32(STATE, &[vtables[field::STATE]]);
        memory.u32(STATE + field::STATE_FIELD, &[FIELDMAP]);
        memory.u32(FIELDMAP + field::FIELD_AREA, &[LOADED, BEDROOM]);
        memory.u32(LOADED, &[AREA]);
        for (actor, name, (x, z)) in [
            (THING, field::THING, (50.5, 57.5)),
            (WARP, field::WARP, (53.5, 60.5)),
        ] {
            memory.u32(actor, &[vtables[name]]);
            floats(&memory, actor + field::ACTOR_X, &[x * C, 1.0, z * C]);
        }
        let found = Found {
            pages: pages_of(SIZES),
            heap: heap_of(&[
                (MANAGER, field::MANAGER),
                (STATE, field::STATE),
                (THING, field::THING),
                (WARP, field::WARP),
            ]),
            partial: false,
        };
        Bedroom {
            memory,
            vtables,
            found,
            recorder: Recorder::new(),
            reads: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// What the reader lists after a sweep that found `found`, this bedroom's by default;
    /// `walked` gives it every block the heaps' walk found, as the runtime does, but those
    /// `unwalked`.
    fn listed(
        &self,
        rom: Option<SunZones>,
        found: Option<&Found>,
        walked: bool,
        unwalked: &[i64],
    ) -> Option<Field> {
        let blocks: Vec<Span> = SIZES
            .iter()
            .filter(|(at, _size)| !unwalked.contains(at))
            .map(|&(at, size)| (at, at + size))
            .collect();
        let emu = FakeEmulator::new(served(&self.memory));
        let memory: Rc<dyn Memory> = Rc::new(Watched {
            memory: RpcMemory::new(connect(&emu)),
            reads: self.reads.clone(),
        });
        let blocks: Option<Blocks> = walked.then(|| {
            let blocks = Rc::new(blocks);
            Rc::new(move || blocks.clone()) as Blocks
        });
        let reader = SunFieldReader::with(
            memory.clone(),
            names(&memory),
            rom.map(Rc::new),
            blocks,
            None,
            None,
        );
        reader.track(found.unwrap_or(&self.found));
        let at = reader.poll().unwrap();
        at.map(|at| reader.interactables(&at).unwrap())
    }

    fn keys(
        &self,
        keys: &[&str],
        walls: Option<Rc<dyn Walls>>,
        found: Option<&Found>,
    ) -> Vec<String> {
        let emu = FakeEmulator::new(served(&self.memory));
        let memory: Rc<dyn Memory> = Rc::new(RpcMemory::new(connect(&emu)));
        let profile = SunProfile::with(
            memory.clone(),
            names(&memory),
            Rc::new(Speech::plain(self.recorder.clone())),
            silent(),
            Options {
                zones: Some(Rc::new(rom())),
                walls,
                ..Options::default()
            },
        );
        profile.track(found.unwrap_or(&self.found)).unwrap();
        profile.poll_field().unwrap();
        for key in keys {
            profile.press(key).unwrap();
        }
        self.recorder.spoken()
    }
}

/// Memory that notes every read outside the code.
struct Watched {
    memory: RpcMemory,
    reads: Rc<RefCell<Vec<Span>>>,
}

impl Memory for Watched {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        if !(CODE.0 <= address && address < CODE.1) {
            self.reads.borrow_mut().push((address, size));
        }
        self.memory.read(address, size)
    }
}

/// `Found(pages, heap)` with the heap given as a map.
fn found_of(pages: &IndexMap<i64, Vec<u8>>, heap: IndexMap<i64, String>) -> Found {
    Found {
        pages: pages.clone(),
        heap,
        partial: false,
    }
}

/// `{**heap, **added}`.
fn with_heap(bedroom: &Bedroom, added: &[(i64, &str)]) -> Found {
    let mut heap = bedroom.found.heap.clone();
    heap.extend(heap_of(added));
    found_of(&bedroom.found.pages, heap)
}

#[test]
fn test_the_player_is_in_the_zone_the_field_says_at_the_cell_their_model_stands_in() {
    let at = Bedroom::new().listed(None, None, false, &[]).unwrap();
    assert_eq!((at.zone, at.cell), (BEDROOM, (50, 60)));
}

#[test]
fn test_people_things_and_exits_are_listed_where_the_field_placed_them() {
    let at = Bedroom::new()
        .listed(Some(rom()), None, false, &[])
        .unwrap();
    let listed: HashMap<(String, String), ((i64, i64), (i64, i64), Option<i64>)> = at
        .interactables
        .iter()
        .map(|item| {
            (
                (item.category.clone(), item.kind.clone()),
                (item.cell, item.size, item.zone),
            )
        })
        .collect();
    let key = |category: &str, kind: &str| (category.to_string(), kind.to_string());
    assert_eq!(
        listed,
        HashMap::from([
            (key("people", "person"), ((46, 60), (1, 1), None)),
            (key("signs", "object"), ((50, 57), (1, 1), None)),
            (key("exits", "door"), ((52, 60), (3, 1), Some(OUTSIDE))),
        ])
    );
    let meowth = at
        .interactables
        .iter()
        .find(|item| item.category == "people")
        .unwrap();
    assert_eq!(
        (meowth.code, meowth.source, meowth.move_model),
        (0x77, MEOWTH, true)
    );
}

#[test]
fn test_without_the_rom_each_thing_is_its_own_cell_and_an_exit_leads_nowhere_named() {
    let at = Bedroom::new().listed(None, None, false, &[]).unwrap();
    let exits: Vec<_> = at
        .of("exits")
        .iter()
        .map(|item| (item.cell, item.size, item.zone))
        .collect();
    assert_eq!(exits, [((53, 60), (1, 1), None)]);
}

/// The byte after an object may be the GPU's, and reading that takes the emulator down
/// (ADR-0035): a manager pointing past its blocks is not followed.
#[test]
fn test_nothing_is_read_outside_a_block_the_sweep_read() {
    let bedroom = Bedroom::new();
    bedroom
        .memory
        .u32(MANAGER + field::MANAGER_PLAYER + 8, &[0x3F000000]);
    assert!(bedroom.listed(Some(rom()), None, false, &[]).is_some());
    assert!(!bedroom.reads.borrow().is_empty());
    for &(address, size) in bedroom.reads.borrow().iter() {
        assert!(
            SIZES
                .iter()
                .any(|&(start, length)| start <= address && address + size <= start + length),
            "{address:#x}+{size:#x}"
        );
    }
}

/// An `Rpc` that notes each read sent alone, as against in a burst.
struct Singles {
    rpc: Rc<dyn Rpc>,
    alone: RefCell<Vec<i64>>,
}

impl Rpc for Singles {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        self.alone.borrow_mut().push(address);
        self.rpc.read(address, size)
    }

    fn read_each(&self, reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        self.rpc.read_each(reads)
    }

    fn reply_seconds(&self) -> f64 {
        self.rpc.reply_seconds()
    }

    fn unsure(&self) -> bool {
        self.rpc.unsure()
    }

    fn confirm(&self) -> Result<bool> {
        self.rpc.confirm()
    }

    fn attach(&self) -> Result<Process> {
        self.rpc.attach()
    }
}

/// An area loading answers each read in about 5 ms, and the listing on arriving on Route 1,
/// a read or two an actor and a model, held polling up 1.35 s (sun_10.ctm, spike 60).
#[test]
fn test_the_listing_reads_the_actors_and_the_models_in_one_burst() {
    let bedroom = Bedroom::new();
    let blocks: Vec<Span> = SIZES.iter().map(|&(at, size)| (at, at + size)).collect();
    let emu = FakeEmulator::new(served(&bedroom.memory));
    let singles = Rc::new(Singles {
        rpc: connect(&emu),
        alone: RefCell::new(Vec::new()),
    });
    let cache = Rc::new(ExactCache::new(singles.clone(), None));
    let memory: Rc<dyn Memory> = cache.clone();
    let blocks = Rc::new(blocks);
    let reader = SunFieldReader::with(
        memory.clone(),
        names(&memory),
        Some(Rc::new(rom())),
        Some(Rc::new(move || blocks.clone())),
        None,
        Some({
            let cache = cache.clone();
            Rc::new(move |reads: &[(i64, i64)]| cache.read_each(reads))
        }),
    );
    reader.track(&bedroom.found);
    let listed = memory::phase(&*cache, "field", || {
        let at = reader.poll()?;
        assert!(at.is_some());
        singles.alone.borrow_mut().clear();
        reader.interactables(&at.unwrap())
    })
    .unwrap();
    assert_eq!(listed.interactables.len(), 3);
    let alone = singles.alone.borrow();
    assert!(![THING, WARP, MEOWTH].iter().any(|at| alone.contains(at)));
}

/// Such a sweep reads only the blocks near what was found before, and the field and the area
/// loaded were not among them: the keys said "no map" in the bedroom (sun_3.ctm).
#[test]
fn test_a_sweep_between_whole_sweeps_leaves_the_field_readable_through_the_heaps_walk() {
    let bedroom = Bedroom::new();
    let near = found_of(
        &pages_of(&[MANAGER, PLAYER, STATE, THING, WARP].map(|at| (at, size_of(at)))),
        bedroom.found.heap.clone(),
    );
    assert_eq!(bedroom.listed(Some(rom()), Some(&near), false, &[]), None);
    let at = bedroom.listed(Some(rom()), Some(&near), true, &[]).unwrap();
    assert_eq!((at.zone, at.interactables.len()), (BEDROOM, 3));
}

#[test]
fn test_the_field_is_not_up_until_the_player_has_a_model_on_it() {
    let bedroom = Bedroom::new();
    bedroom.memory.u32(PLAYER + field::MODEL_NODE, &[0]);
    assert_eq!(bedroom.listed(Some(rom()), None, false, &[]), None);
}

#[test]
fn test_the_keys_walk_the_list_nearest_first_and_p_says_there_is_no_route() {
    assert_eq!(
        Bedroom::new().keys(&["k", "l", "l", "p"], None, None),
        [
            "Route 1",
            "door to Iki Town, 1 of 3, 2 east",
            "object, 2 of 3, 3 north",
            "person, 3 of 3, 4 west",
            "person, 4 west, no route in this game yet",
        ]
    );
}

/// The layout of 'ED' as spike 113 read area 21 out of `a/0/8/2`: a section a kind, a part a
/// zone, each part's records and then the shapes they point at.
#[test]
fn test_an_area_s_warps_and_things_are_read_from_its_event_data() {
    let mut warp = vec![0u8; zones::WARP_RECORD as usize];
    put(&mut warp, 4, &f32s(&[2075.0, 1.0, 2565.0]));
    put(
        &mut warp,
        zones::WARP_ZONE,
        &[le16(BEDROOM), le16(LIVING_ROOM)].concat(),
    );
    let line = [
        le32(1),
        le32(zones::LINE),
        f32s(&[2140.0, 1.0, 2565.0, 2010.0, 1.0, 2565.0, 400.0]),
    ]
    .concat();
    let after = 4 + warp.len() as i64;
    put(&mut warp, zones::WARP_SHAPE, &le32(after));
    let mut thing = vec![0u8; zones::THING_RECORD as usize];
    put(&mut thing, 4, &f32s(&[2454.0, -5.0, 1977.0]));
    put(&mut thing, zones::THING_SCRIPT, &le32(0x378));
    let r#box = [
        le32(1),
        le32(zones::BOX),
        f32s(&[2454.0, -5.0, 1977.0, 100.0, 150.0]),
    ]
    .concat();
    let after = 4 + thing.len() as i64;
    put(&mut thing, zones::THING_SHAPE, &le32(after));
    let parts: HashMap<i64, Vec<u8>> = HashMap::from([
        (zones::WARPS, [le32(1), warp, line].concat()),
        (zones::THINGS, [le32(1), thing, r#box].concat()),
    ]);
    let mut sections: Vec<Vec<u8>> = Vec::new();
    for number in 0..4 {
        let part = parts.get(&number).cloned().unwrap_or_else(|| le32(0));
        sections.push(
            [
                b"EM".to_vec(),
                le16(1),
                le32(12),
                le32(12 + part.len() as i64),
                part,
            ]
            .concat(),
        );
    }
    let mut offsets: Vec<i64> = Vec::new();
    let mut at = 4 + 4 * (sections.len() as i64 + 1);
    for section in &sections {
        offsets.push(at);
        at += section.len() as i64;
    }
    let mut data = [b"ED".to_vec(), le16(sections.len() as i64)].concat();
    for offset in offsets {
        data.extend(le32(offset));
    }
    data.extend(le32(at));
    data.extend(sections.concat());
    let area = zones::area(&data).unwrap();
    let rom = SunZones {
        rows: vec![0; (zones::ZONE_ROW * 0x20) as usize],
        places: place_names(),
        areas: IndexMap::from([(AREA, area)]),
    };
    let door = rom
        .record(AREA, zones::WARPS, 2075.0, 2565.0)
        .unwrap()
        .unwrap();
    assert_eq!(
        (door.zone, door.leads_to, door.shape),
        (
            Some(BEDROOM),
            Some(LIVING_ROOM),
            Some(Shape {
                west: 2010.0,
                north: 2565.0,
                east: 2140.0,
                south: 2565.0
            })
        )
    );
    let shelf = rom
        .record(AREA, zones::THINGS, 2454.0, 1977.0)
        .unwrap()
        .unwrap();
    assert_eq!(
        (shelf.script, shelf.shape),
        (
            0x378,
            Some(Shape {
                west: 2404.0,
                north: 1902.0,
                east: 2504.0,
                south: 2052.0
            })
        )
    );
    assert_eq!(
        rom.record(AREA, zones::THINGS, 2454.0, 1978.0).unwrap(),
        None
    );
}

fn f32s(values: &[f64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&value| (value as f32).to_le_bytes())
        .collect()
}

/// Heap 0x0b is under heap 3: met first as a root, its siblings were not followed, and heap
/// 3's walk lost the 34 heaps after it (spike 113).
#[test]
fn test_a_wanted_heap_under_another_still_has_the_heaps_after_it_walked() {
    let memory = FakeMemory::new();
    let ctr = FakeClasses::new(memory.clone()).add(heaps::CLASS);
    let table = 0x08000000;
    let heap = |at: i64, ident: i64, used: i64, child: i64, sibling: i64| -> i64 {
        memory.u32(at, &[ctr]);
        memory.u32(at + heaps::ID, &[ident]);
        memory.write(at + heaps::HEAD, heaps::EXPH);
        memory.u32(at + heaps::HEAD + heaps::NEXT, &[sibling, child]);
        memory.u32(at + heaps::HEAD + heaps::USED, &[used]);
        memory.u32(table + ident * heaps::ENTRY, &[at]);
        at + heaps::HEAD
    };
    let block = |at: i64, size: i64| -> Span {
        memory.u16(at, heaps::USED_BLOCK);
        memory.u32(at + heaps::BLOCK_SIZE, &[size]);
        (at + heaps::BLOCK_HEADER, at + heaps::BLOCK_HEADER + size)
    };
    let models = block(0x34197000, 0x4B4);
    let dialogue = block(0x32A70000, 0x114);
    let after = heap(0x33000000, 0x24, models.0 - heaps::BLOCK_HEADER, 0, 0);
    let under = heap(
        0x32A6913C - heaps::HEAD,
        0x0B,
        dialogue.0 - heaps::BLOCK_HEADER,
        0,
        after,
    );
    heap(0x32869038 - heaps::HEAD, 3, 0, under, 0);
    let emu = FakeEmulator::new(served(&memory));
    let classes: Rc<dyn Memory> = memory.clone();
    let wanted: Vec<i64> = PySet::of([3, 0x0B]).iter().collect();
    let mut spans = Heaps::with(
        connect(&emu),
        names(&classes),
        Rc::new(|_line: &str| {}),
        &wanted,
        &heaps::UNREAD,
    )
    .spans()
    .unwrap();
    spans.sort();
    assert_eq!(spans, [dialogue, models]);
}

/// On Route 1's second zone the word read 0x10001, and every key said "no map" from there to
/// Iki Town (sun_4.ctm, 2026-09-26).
#[test]
fn test_a_zone_is_the_field_s_low_half_word() {
    let bedroom = Bedroom::new();
    bedroom.memory.u16(FIELDMAP + field::FIELD_ZONE + 2, 1);
    let at = bedroom.listed(Some(rom()), None, false, &[]).unwrap();
    assert_eq!(at.zone, BEDROOM);
}

/// An item actor is a ball still on the ground; a thing of a signpost's kind is a sign; Iki
/// Town built five of its things twice, once for each of two scripts (spike 115).
#[test]
fn test_a_ball_on_the_ground_a_signpost_and_a_thing_built_twice_are_each_said_once() {
    let bedroom = Bedroom::new();
    let added = [
        (BALL, field::ITEM, (48.5, 62.5)),
        (AGAIN, field::THING, (50.5, 57.5)),
        (POST, field::THING, (47.5, 57.5)),
    ];
    for (actor, name, (x, z)) in added {
        bedroom.memory.u32(actor, &[bedroom.vtables[name]]);
        floats(
            &bedroom.memory,
            actor + field::ACTOR_X,
            &[x * C, 1.0, z * C],
        );
    }
    let found = with_heap(&bedroom, &added.map(|(actor, name, _at)| (actor, name)));
    let at = bedroom
        .listed(Some(rom()), Some(&found), false, &[])
        .unwrap();
    let mut listed: Vec<(String, String, (i64, i64), i64)> = at
        .interactables
        .iter()
        .filter(|item| item.category != "people")
        .map(|item| {
            (
                item.category.clone(),
                item.kind.clone(),
                item.cell,
                item.code,
            )
        })
        .collect();
    listed.sort();
    let row = |category: &str, kind: &str, cell, code| {
        (category.to_string(), kind.to_string(), cell, code)
    };
    assert_eq!(
        listed,
        [
            row("exits", "door", (52, 60), zones::WARP_DOOR),
            row("items", "item ball", (48, 62), 4),
            row("signs", "object", (50, 57), 0),
            row("signs", "sign", (47, 57), 0),
        ]
    );
}

/// The field is in heap 0x21, whose own blocks the walk passes over: its block's header, 0x30
/// before it, says the block is there (sun_4.ctm, 2026-09-26).
#[test]
fn test_the_field_is_read_through_its_own_block_s_header_where_the_walk_passes_it_over() {
    let bedroom = Bedroom::new();
    let header = FIELDMAP - field::OBJECT - heaps::BLOCK_HEADER;
    assert_eq!(bedroom.listed(Some(rom()), None, true, &[FIELDMAP]), None);
    bedroom.memory.u16(header, heaps::USED_BLOCK);
    bedroom.memory.u32(header + heaps::BLOCK_SIZE, &[0x1A0]);
    let at = bedroom
        .listed(Some(rom()), None, true, &[FIELDMAP])
        .unwrap();
    assert_eq!(at.zone, BEDROOM);
}

/// The bedroom's area with a wall two cells west of the player, from z 58 to 62.
struct TestWalls {
    walls: Arc<AreaWalls>,
}

impl TestWalls {
    fn walls() -> Rc<dyn Walls> {
        let side = 84;
        let mut cells = vec![1u8; side * side];
        for z in 58..63 {
            cells[z * side + 48] = 0;
        }
        Rc::new(TestWalls {
            walls: Arc::new(AreaWalls::new(
                side as i64,
                IndexMap::from([((0, 0), cells)]),
                None,
            )),
        })
    }
}

impl Walls for TestWalls {
    fn prefetch(&self, area: i64) {
        let _ = self.area(area);
    }

    fn area(&self, area: i64) -> Result<Option<Arc<AreaWalls>>> {
        Ok((area == AREA).then(|| self.walls.clone()))
    }
}

/// Sun's walls are the ROM's meshes, made into cells (spike 114): P walks round the wall
/// between the player and Meowth rather than saying there is no route.
#[test]
fn test_p_finds_the_way_round_the_walls_out_of_the_rom() {
    let spoken = Bedroom::new().keys(&["l", "l", "p"], Some(TestWalls::walls()), None);
    assert_eq!(spoken[2], "person, 3 of 3, 4 west");
    assert_eq!(spoken[3..], ["3 north, 4 west, 2 south"]);
}

/// Walking out of the house, every person stood at the world's corner for a listing, as
/// "39 west 219 north" (sun_4.ctm, 2026-09-26).
#[test]
fn test_a_model_the_area_has_not_placed_yet_is_not_listed() {
    let bedroom = Bedroom::new();
    floats(
        &bedroom.memory,
        MEOWTH + field::MODEL_PLACE,
        &[0.0, 0.0, 0.0],
    );
    let at = bedroom.listed(Some(rom()), None, false, &[]).unwrap();
    assert!(at.of("people").is_empty());
}

/// Walking back into the house, Route 1's doors were still actors until the next sweep, and
/// were listed in the living room as six warps (sun_1.ctm, 2026-09-26).
#[test]
fn test_an_actor_the_loaded_area_has_no_record_of_is_the_last_area_s_and_is_not_listed() {
    let bedroom = Bedroom::new();
    let stale = 0x30372F00;
    bedroom.memory.u32(stale, &[bedroom.vtables[field::WARP]]);
    floats(
        &bedroom.memory,
        stale + field::ACTOR_X,
        &[20.5 * C, 1.0, 30.5 * C],
    );
    let found = with_heap(&bedroom, &[(stale, field::WARP)]);
    let at = bedroom
        .listed(Some(rom()), Some(&found), true, &[])
        .unwrap();
    let cells: Vec<(i64, i64)> = at.of("exits").iter().map(|item| item.cell).collect();
    assert_eq!(cells, [(52, 60)]);
}

/// The living room held five models nobody sees, one in the player's own cell, and P led the
/// user to it: their +0x308 is 0, as every model not drawn had (spike 117).
#[test]
fn test_a_model_on_the_field_that_is_not_drawn_is_not_listed() {
    let bedroom = Bedroom::new();
    bedroom.memory.u32(MEOWTH + field::MODEL_SHOWN, &[0]);
    let at = bedroom.listed(Some(rom()), None, false, &[]).unwrap();
    assert!(at.of("people").is_empty());
}

/// Three of the seven item actors beside the house were hidden items, and P led the user onto
/// one where nothing was (sun_4.ctm, 2026-09-26).
#[test]
fn test_a_hidden_item_is_not_listed_though_the_field_builds_it_an_actor() {
    let bedroom = Bedroom::new();
    let hidden = 0x3037B068;
    for (actor, (x, z)) in [(BALL, (48.5, 62.5)), (hidden, (44.5, 62.5))] {
        bedroom.memory.u32(actor, &[bedroom.vtables[field::ITEM]]);
        floats(
            &bedroom.memory,
            actor + field::ACTOR_X,
            &[x * C, 1.0, z * C],
        );
    }
    let found = with_heap(&bedroom, &[(BALL, field::ITEM), (hidden, field::ITEM)]);
    let at = bedroom
        .listed(Some(rom()), Some(&found), true, &[])
        .unwrap();
    let cells: Vec<(i64, i64)> = at.of("items").iter().map(|item| item.cell).collect();
    assert_eq!(cells, [(48, 62)]);
}

/// A prop in a collision group at a cell's centre: a box if `values` are a box's twelve
/// floats, a cylinder of the radius given otherwise.
fn prop(bedroom: &Bedroom, actor: i64, shape: i64, group: i64, at: (f64, f64), values: &[f64]) {
    let kind = if values.len() == 12 {
        field::BOX
    } else {
        field::CYLINDER
    };
    bedroom.memory.u32(actor, &[bedroom.vtables[field::PROP]]);
    bedroom.memory.u32(actor + field::PROP_SCENE, &[group]);
    bedroom.memory.u32(actor + field::PROP_SHAPE, &[shape]);
    floats(
        &bedroom.memory,
        actor + field::PROP_X,
        &[at.0 * C, 0.0, at.1 * C],
    );
    bedroom.memory.u32(shape, &[bedroom.vtables[kind]]);
    let laid: Vec<f64> = if kind == field::CYLINDER {
        [&[0.0, 0.0, 0.0], values].concat()
    } else {
        values.to_vec()
    };
    floats(&bedroom.memory, shape + field::SHAPE_X, &laid);
}

/// A flower pot on the porch is a box 104 units a side round a record of a unit: P led the
/// user to the cell beside the record, inside the pot, and never said they had arrived
/// (sun_5.ctm, 2026-09-26). The pot is in the bodies' group, which Meowth's body shows, so its
/// cells are closed and the way ends a cell further out; the range Meowth is talked to from is
/// another group's, and closes nothing.
#[test]
fn test_a_path_to_a_thing_ends_where_the_player_can_stand_outside_its_prop() {
    let bedroom = Bedroom::new();
    prop(
        &bedroom,
        PROP,
        BOX,
        BODIES,
        (50.5, 57.5),
        &[
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 52.0, 52.0, 150.0,
        ],
    );
    prop(
        &bedroom,
        BODY,
        BODY_SHAPE,
        BODIES,
        (46.5, 60.5),
        &[40.0, 150.0],
    );
    prop(
        &bedroom,
        TALK,
        TALK_SHAPE,
        RANGES,
        (46.5, 60.5),
        &[160.0, 150.0],
    );
    let found = with_heap(
        &bedroom,
        &[
            (PROP, field::PROP),
            (BODY, field::PROP),
            (TALK, field::PROP),
        ],
    );
    let spoken = bedroom.keys(&["l", "p"], Some(TestWalls::walls()), Some(&found));
    assert_eq!(spoken[1], "object, 2 of 3, 3 north");
    assert_eq!(spoken.last().unwrap(), "1 north");
}

/// Taken as solid, every group walled the user in at their own door and sent them round Route
/// 1 to get home (sun_6.ctm, 2026-09-26): without a body to go by, nothing is closed.
#[test]
fn test_no_prop_is_solid_until_a_drawn_person_shows_the_bodies_group() {
    let bedroom = Bedroom::new();
    prop(
        &bedroom,
        PROP,
        BOX,
        BODIES,
        (50.5, 57.5),
        &[
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 52.0, 52.0, 150.0,
        ],
    );
    let found = with_heap(&bedroom, &[(PROP, field::PROP)]);
    let spoken = bedroom.keys(&["l", "p"], Some(TestWalls::walls()), Some(&found));
    assert_eq!(spoken.last().unwrap(), "3 north");
}

/// Outside the user's front door a box covered the cell they stood in, and P said "no path"
/// to everything on Route 1 (sun_6.ctm, 2026-09-26).
#[test]
fn test_a_prop_round_the_player_s_own_cell_does_not_wall_them_in() {
    let bedroom = Bedroom::new();
    prop(
        &bedroom,
        BODY,
        BODY_SHAPE,
        BODIES,
        (46.5, 60.5),
        &[40.0, 150.0],
    );
    prop(
        &bedroom,
        HERE,
        HERE_SHAPE,
        BODIES,
        (50.5, 60.5),
        &[120.0, 150.0],
    );
    let found = with_heap(&bedroom, &[(BODY, field::PROP), (HERE, field::PROP)]);
    let spoken = bedroom.keys(&["l", "p"], Some(TestWalls::walls()), Some(&found));
    assert_eq!(spoken.last().unwrap(), "3 north");
}

/// On the porch the camera looks west, and up walked the user to the door that pa3ds called
/// "2 west" (sun_5.ctm, spike 119). Turned, the door two cells east of the player is behind them,
/// and the way round the wall to Meowth, four cells west, is ahead.
#[test]
fn test_facing_west_north_is_the_way_up_walks_and_the_way_is_said_so() {
    let bedroom = Bedroom::new();
    bedroom
        .memory
        .u32(TRANSLATOR, &[bedroom.vtables[camera::CLASS]]);
    floats(
        &bedroom.memory,
        TRANSLATOR + camera::ANGLE,
        &[-std::f64::consts::PI / 2.0],
    );
    bedroom.memory.u32(TRANSLATOR + camera::SET, &[1]);
    let found = with_heap(&bedroom, &[(TRANSLATOR, camera::CLASS)]);
    let spoken = bedroom.keys(
        &["k", "l", "l", "p"],
        Some(TestWalls::walls()),
        Some(&found),
    );
    assert_eq!(
        spoken,
        [
            "Route 1",
            "door to Iki Town, 1 of 3, 2 south",
            "object, 2 of 3, 3 east",
            "person, 3 of 3, 4 north",
            "3 east, 4 north, 2 west"
        ]
    );
}

/// Back from Kukui's catching tutorial the field builds its camera again, and until it has an
/// angle it reads 0.0, south: Route 1 looks north, and the field was listed turned half round
/// (sun_10.ctm, 2026-09-27).
#[test]
fn test_a_camera_given_no_angle_yet_keeps_the_compass_as_it_was() {
    let bedroom = Bedroom::new();
    bedroom
        .memory
        .u32(TRANSLATOR, &[bedroom.vtables[camera::CLASS]]);
    floats(
        &bedroom.memory,
        TRANSLATOR + camera::ANGLE,
        &[std::f64::consts::PI],
    );
    bedroom.memory.u32(TRANSLATOR + camera::SET, &[1]);
    let found = with_heap(&bedroom, &[(TRANSLATOR, camera::CLASS)]);
    let emu = FakeEmulator::new(served(&bedroom.memory));
    let memory: Rc<dyn Memory> = Rc::new(RpcMemory::new(connect(&emu)));
    let reader = SunFieldReader::with(
        memory.clone(),
        names(&memory),
        Some(Rc::new(rom())),
        None,
        None,
        None,
    );
    reader.track(&found);
    let before = reader.poll().unwrap();
    floats(&bedroom.memory, TRANSLATOR + camera::ANGLE, &[0.0]);
    bedroom.memory.u32(TRANSLATOR + camera::SET, &[0]);
    update(&emu, &bedroom.memory);
    let building = reader.poll().unwrap();
    bedroom.memory.u32(TRANSLATOR + camera::SET, &[1]);
    update(&emu, &bedroom.memory);
    let south = reader.poll().unwrap();
    let (before, building, south) = (before.unwrap(), building.unwrap(), south.unwrap());
    assert_eq!(before.cell, building.cell);
    assert_eq!(building.cell, (50, 60));
    assert_eq!(south.cell, camera::turned((50, 60), 2));
}

/// Route 1's two zones are one area: walking from one to the other dropped the way home as
/// though through a door (sun_6.ctm, 2026-09-26).
#[test]
fn test_walking_into_another_zone_of_the_area_is_not_a_warp() {
    let bedroom = Bedroom::new();
    let emu = FakeEmulator::new(served(&bedroom.memory));
    let memory: Rc<dyn Memory> = Rc::new(RpcMemory::new(connect(&emu)));
    let reader = SunFieldReader::with(
        memory.clone(),
        names(&memory),
        Some(Rc::new(rom())),
        None,
        None,
        None,
    );
    reader.track(&bedroom.found);
    assert!(reader.poll().unwrap().is_some());
    let before = reader.visit();
    bedroom
        .memory
        .u16(FIELDMAP + field::FIELD_ZONE, LIVING_ROOM);
    update(&emu, &bedroom.memory);
    let at = reader.poll().unwrap();
    assert!(at.is_some() && at.unwrap().zone == LIVING_ROOM);
    assert_eq!(reader.visit(), before);
}

/// The chase after Kukui's call ends on the field, Route 1's zone 0x0, and the beats over its
/// arrival waited out a timer from the demo's start (movie1_opening.ctm, 2026-10-13).
#[test]
fn test_a_beat_waiting_for_a_zone_comes_as_the_field_arrives_there() {
    let bedroom = Bedroom::new();
    let scene = Scene {
        lines: Vec::new(),
        zone: None,
        beats: vec![Beat {
            message: "field-1".to_string(),
            demo: false,
            zones: HashSet::from([BEDROOM]),
            at: 0.0,
        }],
        demo: Some(6),
        trigger: None,
        app: None,
    };
    let emu = FakeEmulator::new(served(&bedroom.memory));
    let memory: Rc<dyn Memory> = Rc::new(RpcMemory::new(connect(&emu)));
    let profile = SunProfile::with(
        memory.clone(),
        names(&memory),
        Rc::new(Speech::plain(bedroom.recorder.clone())),
        silent(),
        Options {
            ui: Some(Ui {
                descriptions: true,
                ..Ui::default()
            }),
            zones: Some(Rc::new(rom())),
            scenes: vec![scene],
            said: Some(Rc::new(|message: &str| {
                (message == "field-1").then(|| "A field.".to_string())
            })),
            ..Options::default()
        },
    );
    profile.track(&bedroom.found).unwrap();
    let scenes = profile.scenes.as_ref().unwrap();
    scenes
        .borrow_mut()
        .demo(Some(6), &mut |_: &str| {})
        .unwrap();
    assert!(bedroom.recorder.spoken().is_empty());
    profile.poll_field().unwrap();
    assert!(
        bedroom
            .recorder
            .spoken()
            .iter()
            .any(|line| line.contains("A field."))
    );
}

/// Sun's player walks anywhere, a body a cell and a half across: counted as leaving the way,
/// every step a cell off it found a new one and said it (sun_6.ctm, 2026-09-26).
#[test]
fn test_drifting_a_cell_off_the_way_is_still_walking_it() {
    let bedroom = Bedroom::new();
    let emu = FakeEmulator::new(served(&bedroom.memory));
    let memory: Rc<dyn Memory> = Rc::new(RpcMemory::new(connect(&emu)));
    let profile = SunProfile::with(
        memory.clone(),
        names(&memory),
        Rc::new(Speech::plain(bedroom.recorder.clone())),
        silent(),
        Options {
            zones: Some(Rc::new(rom())),
            walls: Some(TestWalls::walls()),
            ..Options::default()
        },
    );
    profile.track(&bedroom.found).unwrap();
    profile.poll_field().unwrap();
    for key in ["l", "l", "p"] {
        profile.press(key).unwrap();
    }
    let said = bedroom.recorder.spoken().len();
    // A cell east of the start and one north: off the way, which goes north first.
    floats(
        &bedroom.memory,
        PLAYER + field::MODEL_PLACE,
        &[51.5 * C, 5.0, 59.5 * C],
    );
    update(&emu, &bedroom.memory);
    profile.poll_guidance().unwrap();
    assert_eq!(bedroom.recorder.spoken()[said..], Vec::<String>::new());
    floats(
        &bedroom.memory,
        PLAYER + field::MODEL_PLACE,
        &[55.5 * C, 5.0, 59.5 * C],
    );
    update(&emu, &bedroom.memory);
    profile.poll_guidance().unwrap();
    assert_eq!(bedroom.recorder.spoken().len(), said + 1);
}

/// Through a door the area loaded changes before the player's model is placed in it, and for
/// two seconds everything was said from where it had stood in the house (sun_4.ctm).
#[test]
fn test_the_field_is_not_up_until_the_player_is_placed_in_the_area_loaded() {
    let bedroom = Bedroom::new();
    let emu = FakeEmulator::new(served(&bedroom.memory));
    let memory: Rc<dyn Memory> = Rc::new(RpcMemory::new(connect(&emu)));
    let reader = SunFieldReader::with(
        memory.clone(),
        names(&memory),
        Some(Rc::new(rom())),
        None,
        None,
        None,
    );
    reader.track(&bedroom.found);
    assert!(reader.poll().unwrap().is_some());
    bedroom.memory.u32(LOADED, &[0]);
    update(&emu, &bedroom.memory);
    assert!(reader.poll().unwrap().is_none());
    floats(
        &bedroom.memory,
        PLAYER + field::MODEL_PLACE,
        &[561.5 * C, 5.0, 362.5 * C],
    );
    update(&emu, &bedroom.memory);
    let at = reader.poll().unwrap();
    assert!(at.is_some() && at.unwrap().cell == (561, 362));
}

/// Walking from Route 1's first zone into its second made the thing P led to "gone"
/// (sun_6.ctm, 2026-09-26): a thing's zone is its record's.
#[test]
fn test_a_thing_is_the_same_thing_from_either_zone_of_its_area() {
    let bedroom = Bedroom::new();
    let before = bedroom.listed(Some(rom()), None, false, &[]);
    bedroom
        .memory
        .u16(FIELDMAP + field::FIELD_ZONE, LIVING_ROOM);
    let after = bedroom.listed(Some(rom()), None, false, &[]);
    let (before, after) = (before.unwrap(), after.unwrap());
    let keys = |at: &Field| -> Vec<_> { at.of("exits").iter().map(|item| item.key()).collect() };
    assert_eq!(keys(&before), keys(&after));
}

/// The field keeps only the zone's actors the player is in: stepping over the line between
/// Route 1's two zones, the thing P was leading to was "gone" (sun_6.ctm, 2026-09-26). An exit
/// stays; a ball is gone once its zone's balls are there without it.
#[test]
fn test_things_of_the_area_s_other_zones_are_kept_until_their_own_zone_has_them_gone() {
    let bedroom = Bedroom::new();
    let second = 0x3037B200;
    for (actor, (x, z)) in [(BALL, (48.5, 62.5)), (second, (46.5, 63.5))] {
        bedroom.memory.u32(actor, &[bedroom.vtables[field::ITEM]]);
        floats(
            &bedroom.memory,
            actor + field::ACTOR_X,
            &[x * C, 1.0, z * C],
        );
    }
    let mut heap = bedroom.found.heap.clone();
    heap.extend(heap_of(&[(BALL, field::ITEM), (second, field::ITEM)]));
    let mut blocks = bedroom.found.pages.clone();
    blocks.insert(second, vec![0; 0x24]);
    let emu = FakeEmulator::new(served(&bedroom.memory));
    let memory: Rc<dyn Memory> = Rc::new(RpcMemory::new(connect(&emu)));
    let reader = SunFieldReader::with(
        memory.clone(),
        names(&memory),
        Some(Rc::new(rom())),
        None,
        None,
        None,
    );
    let listed = |category: &str| -> Vec<(i64, i64)> {
        let at = reader.poll().unwrap().unwrap();
        let mut cells: Vec<(i64, i64)> = reader
            .interactables(&at)
            .unwrap()
            .of(category)
            .iter()
            .map(|item| item.cell)
            .collect();
        cells.sort();
        cells
    };

    reader.track(&found_of(&blocks, heap.clone()));
    assert_eq!(listed("items"), [(46, 63), (48, 62)]);
    bedroom
        .memory
        .u16(FIELDMAP + field::FIELD_ZONE, LIVING_ROOM);
    update(&emu, &bedroom.memory);
    reader.track(&found_of(
        &blocks,
        heap.iter()
            .filter(|(_at, name)| !field::ACTORS.contains(&name.as_str()))
            .map(|(&at, name)| (at, name.clone()))
            .collect(),
    ));
    assert_eq!(listed("items"), [(46, 63), (48, 62)]);
    assert_eq!(listed("exits"), [(52, 60)]);
    bedroom.memory.u16(FIELDMAP + field::FIELD_ZONE, BEDROOM);
    update(&emu, &bedroom.memory);
    reader.track(&found_of(
        &blocks,
        heap.iter()
            .filter(|&(&at, _)| at != BALL)
            .map(|(&at, name)| (at, name.clone()))
            .collect(),
    ));
    assert_eq!(listed("items"), [(46, 63)]);
    assert_eq!(listed("exits"), [(52, 60)]);
}

/// Route 1's two zones are one ground with a line between them the player cannot see, and
/// the field builds only the zone they are in (sun_7.ctm, 2026-09-26).
#[test]
fn test_an_exit_of_a_zone_the_player_has_not_been_in_is_listed_from_the_rom() {
    let bedroom = Bedroom::new();
    bedroom
        .memory
        .u16(FIELDMAP + field::FIELD_ZONE, LIVING_ROOM);
    let found = found_of(
        &bedroom.found.pages,
        bedroom
            .found
            .heap
            .iter()
            .filter(|(_at, name)| !field::ACTORS.contains(&name.as_str()))
            .map(|(&at, name)| (at, name.clone()))
            .collect(),
    );
    let at = bedroom
        .listed(Some(rom()), Some(&found), false, &[])
        .unwrap();
    let exits: Vec<_> = at
        .of("exits")
        .iter()
        .map(|item| (item.kind.clone(), item.cell, item.zone))
        .collect();
    assert_eq!(exits, [("door".to_string(), (52, 60), Some(OUTSIDE))]);
}

/// The player's house is one area whose rooms are zones laid out far apart, and from the
/// porch the other rooms' mats were listed hundreds of cells away (2026-09-27).
#[test]
fn test_indoors_only_the_player_s_own_zone_is_listed() {
    let bedroom = Bedroom::new();
    bedroom
        .memory
        .u16(FIELDMAP + field::FIELD_ZONE, LIVING_ROOM);
    let mut rows = rom().rows;
    for zone in [BEDROOM, LIVING_ROOM] {
        put(
            &mut rows,
            zone * zones::ZONE_ROW + zones::ROW_KIND,
            &le16(zones::INDOORS),
        );
    }
    let indoors = SunZones {
        rows,
        places: place_names(),
        areas: rom().areas,
    };
    let found = found_of(
        &bedroom.found.pages,
        bedroom
            .found
            .heap
            .iter()
            .filter(|(_at, name)| !field::ACTORS.contains(&name.as_str()))
            .map(|(&at, name)| (at, name.clone()))
            .collect(),
    );
    let at = bedroom
        .listed(Some(indoors), Some(&found), false, &[])
        .unwrap();
    assert!(at.of("exits").is_empty());
}

/// Route 1's zones have place lines of their own, both "Route 1", and it was said at every
/// step over the line between them (sun_7.ctm, 2026-09-26).
#[test]
fn test_walking_between_two_zones_of_one_name_says_it_once() {
    let bedroom = Bedroom::new();
    let mut rows = rom().rows;
    put(
        &mut rows,
        LIVING_ROOM * zones::ZONE_ROW + zones::ROW_PLACE,
        &le16(7),
    );
    let mut lines = vec![String::new(); 25];
    lines[6] = "Route 1".to_string();
    lines[7] = "Route 1".to_string();
    let named = SunZones {
        rows,
        places: SunText {
            files: IndexMap::from([(zones::PLACE_NAMES, lines)]),
        },
        areas: rom().areas,
    };
    let emu = FakeEmulator::new(served(&bedroom.memory));
    let memory: Rc<dyn Memory> = Rc::new(RpcMemory::new(connect(&emu)));
    let profile = SunProfile::with(
        memory.clone(),
        names(&memory),
        Rc::new(Speech::plain(bedroom.recorder.clone())),
        silent(),
        Options {
            zones: Some(Rc::new(named)),
            ..Options::default()
        },
    );
    profile.track(&bedroom.found).unwrap();
    for zone in [BEDROOM, LIVING_ROOM, BEDROOM] {
        bedroom.memory.u16(FIELDMAP + field::FIELD_ZONE, zone);
        update(&emu, &bedroom.memory);
        profile.poll_field().unwrap();
    }
    assert_eq!(bedroom.recorder.spoken(), ["Route 1"]);
}
