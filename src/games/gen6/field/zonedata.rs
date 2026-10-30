//! A zone's data file, as the ROM stores it and the game decompresses it whole into the heap
//! (spike 78). Magic `ZO`, a u16 count of sections (5, and 4 in X and Y) and one u32 offset more
//! than that, the first just past them. Section 0 is the zone's info (0x38 bytes), section 1 its
//! entity table, the same in both games (spike 100):
//!
//! ```text
//! +0x00 u32 size = 8 + the records      +0x04 u8 furniture, u8 NPCs, u8 warps, u8 triggers
//! +0x08 u8 areas, then the records in that order:
//! furniture 0x14: u16 script, kind, ?, placement, x, z, width, height, ?, 0
//! NPC       0x30: u16 index, model code, flag, 0, script, work, ?, 0 x5, width, height,
//!                 range x4, 0 x3, x, z, f32 y
//! warp      0x18: u16 to zone, to warp, kind, placement, s16 world x, y, z, u16 width,
//!                 height, 0 x3
//! ```
//!
//! A record whose placement is 1 is on a rail, as on Lumiose's boulevards: where the cell or the
//! world position would be it holds a u32 line id, s16 cells along it and across it, and u16 cells
//! it spans along and across (spike 102). Such a record is listed in the corridor of the rails
//! loaded (`rail`), and not at all off them.
//!
//! ```text
//! trigger   0x18: u16 script, value, work, way, 0, 0, x, z, width, height, 0, 0; armed
//!           while the work holds the value, 0 for always (spike 163), and the way probably
//!           the way it is walked onto to fire, 5 north, 6 south, 7 west, 8 east, 0 any
//!           (spike 132)
//! area      0x18: the same shape as a trigger (six on Route 102, none in Littleroot)
//! ```
//!
//! Cells are (x, z), x east and z south; a warp's world units are 18 to a cell, the centre at
//! `cell * 18 + 9`. Two files are kept in a pair of buffers behind a two-word holder, the current
//! one first, with the current one's LZ11 original after them.

use indexmap::IndexMap;

use crate::core::binary::{s16, u16, u32};
use crate::core::memory::Memory;
use crate::core::pathfind::Cell;
use crate::games::common::field::things::{FURNITURE_SOURCE, Interactable, place_source};
use crate::py::{self, Result};

pub const ZONE_MAGIC: &[u8] = b"ZO";

/// Five in ORAS, four in X and Y (spike 100): the info and the entity table lead in both.
pub const ZONE_SECTIONS: [i64; 2] = [5, 4];

pub const ZONE_INFO: i64 = 0;

pub const ZONE_ENTITIES: i64 = 1;

/// The zone's script, compiled Pawn (`script`, spike 107).
pub const ZONE_SCRIPT: i64 = 2;

pub const MAX_ZONE_SIZE: i64 = 0x20000;

pub const ENTITY_HEADER: i64 = 12;

pub const PLACEMENT: i64 = 0x06;

/// A furniture or warp record's +0x06: 0 on the grid, 1 on a rail (spike 102).
pub const ON_RAIL: i64 = 1;

/// An NPC record's index, which a move model built from it holds (`mmodel.index_of`).
pub const NPC_INDEX: i64 = 0x00;

pub const NPC_SPRITE: i64 = 0x02;

pub const NPC_SCRIPT: i64 = 0x08;

pub const NPC_FLAG: i64 = 0x0A;

/// The cell an NPC record starts its move model in.
pub const NPC_X: i64 = 0x28;
pub const NPC_Z: i64 = 0x2A;

/// An NPC record's +0x0a is 3000 plus a trainer's id when the NPC is one (spike 90).
pub const TRAINER_FLAGS: i64 = 3000;

pub const FURNITURE: i64 = 0x14;

pub const NPC: i64 = 0x30;

pub const WARP: i64 = 0x18;

pub const TRIGGER: i64 = 0x18;

pub const AREA: i64 = 0x18;

pub const TRIGGER_SCRIPT: i64 = 0x00;

pub const TRIGGER_X: i64 = 0x0C;

pub const TRIGGER_Z: i64 = 0x0E;

pub const TRIGGER_WIDTH: i64 = 0x10;

pub const TRIGGER_HEIGHT: i64 = 0x12;

// The work a trigger is armed by and the value it is armed at.
pub const TRIGGER_VALUE: i64 = 0x02;
pub const TRIGGER_WORK: i64 = 0x04;

/// How high in the world a trigger stands, in world units, where `TRIGGER_HEIGHT` is how many
/// cells deep it is: 108 for the top of each of Fortree City's ladders and 0 for its foot (spike
/// 160), -9 to 108 for Lavaridge Gym's (spike 157).
pub const TRIGGER_Y: i64 = 0x14;

/// World units per cell.
pub const CELL: i64 = 18;

pub const FURNITURE_SCRIPT: i64 = 0x00;

/// A furniture record's first cell and how many cells it covers, off the rails.
pub const FURNITURE_X: i64 = 0x08;
pub const FURNITURE_Z: i64 = 0x0A;
pub const FURNITURE_WIDTH: i64 = 0x0C;
pub const FURNITURE_HEIGHT: i64 = 0x0E;

/// A warp record's zone it goes to, and its kind (`WARP_DOOR` and the rest).
pub const WARP_TO: i64 = 0x00;
pub const WARP_KIND: i64 = 0x04;

/// A warp record's place in world units, s16 (`from_world`), and how many cells it covers, off
/// the rails.
pub const WARP_X: i64 = 0x08;
pub const WARP_Z: i64 = 0x0C;
pub const WARP_WIDTH: i64 = 0x0E;
pub const WARP_HEIGHT: i64 = 0x10;

/// How high in the world a warp off the rails stands: 198 for Sootopolis Gym's way out, the
/// height of the floor of its entrance's corridor over the lower floor's -18 (spike 167).
pub const WARP_Y: i64 = 0x0A;

/// A warp record's +0x02: which of the warps of the zone it goes to it comes out at.
pub const WARP_TO_SLOT: i64 = 0x02;

pub const LZ11: i64 = 0x11;

/// A door in from outdoors.
pub const WARP_DOOR: i64 = 0x0301;

/// The mat out of a building.
pub const WARP_OUT: i64 = 0x0200;

/// The high byte of a warp walked into off one of Lumiose's streets, in X: into one of its
/// buildings (spike 101), or onto Centrico Plaza, another zone of its streets (spike 107).
pub const WARP_STREET: i64 = 0x07;

/// The high byte of a warp walked through: stairs between floors, an opening between two rooms
/// of one floor, and the open path between two outdoor matrices. The low byte is the way it is
/// walked into, 0 south, 1 north, 2 east, 3 west (Oldale to Route 102 is 3, back again 2), and says
/// nothing of up or down: that is the warp's height over the ground it is walked from
/// (`FieldReader._stairs_word`, 2026-10-06).
pub const WARP_STAIRS: i64 = 0x05;

/// A zone data file found in the heap, validated.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneData {
    pub address: i64,
    pub offsets: Vec<i64>,
}

impl ZoneData {
    pub fn size(&self) -> i64 {
        self.offsets[self.offsets.len() - 1]
    }

    pub fn info(&self) -> i64 {
        self.address + self.offsets[ZONE_INFO as usize]
    }

    pub fn entities(&self) -> i64 {
        self.address + self.offsets[ZONE_ENTITIES as usize]
    }
}

/// Every zone data file in these pages, validated.
pub fn zone_files(pages: &IndexMap<i64, Vec<u8>>, memory: &dyn Memory) -> Result<Vec<ZoneData>> {
    let mut out: Vec<ZoneData> = Vec::new();
    for (&base, page) in pages {
        for at in find_all(page, ZONE_MAGIC) {
            if at % 4 == 0
                && let Some(data) = read_zone(memory, base + at as i64)?
            {
                out.push(data);
            }
        }
    }
    Ok(out)
}

/// (slot, zone behind it) for every door in from the outdoors in a zone data file, given whole as
/// the ROM keeps it once decompressed (spike 86). The slot is the warp's place among the zone's
/// warps, which is what a door the navigation keys are on is renamed by.
pub fn door_warps(data: &[u8]) -> Result<Vec<(i64, i64)>> {
    Ok(warps(data)?
        .into_iter()
        .filter(|&(_, _, kind)| kind == WARP_DOOR)
        .map(|(slot, to, _)| (slot, to))
        .collect())
}

/// (slot, zone it goes to, kind) for every warp in a zone data file held whole.
pub fn warps(data: &[u8]) -> Result<Vec<(i64, i64, i64)>> {
    let memory = Bytes(data);
    let Some(zone) = read_zone(&memory, 0)? else {
        return Ok(Vec::new());
    };
    let mut out: Vec<(i64, i64, i64)> = Vec::new();
    for (slot, record) in read_records(&memory, &zone, WARP)?.iter().enumerate() {
        out.push((slot as i64, u16(record, WARP_TO)?, u16(record, WARP_KIND)?));
    }
    Ok(out)
}

/// (sprite, trainer id) for every NPC record in a zone data file that is a trainer: +0x0a holds
/// 3000 plus the trainer's id, the flag the game sets once they are beaten (spike 90). `trainers`
/// is how many there are, so a flag past the last is not taken for one.
pub fn npc_trainers(data: &[u8], trainers: i64) -> Result<Vec<(i64, i64)>> {
    let memory = Bytes(data);
    let Some(zone) = read_zone(&memory, 0)? else {
        return Ok(Vec::new());
    };
    let mut found: Vec<(i64, i64)> = Vec::new();
    for record in read_records(&memory, &zone, NPC)? {
        let trainer = u16(&record, NPC_FLAG)? - TRAINER_FLAGS;
        if 0 < trainer && trainer < trainers {
            found.push((u16(&record, NPC_SPRITE)?, trainer));
        }
    }
    Ok(found)
}

/// A file held whole, read as memory from address 0.
pub struct Bytes<'a>(pub &'a [u8]);

impl Memory for Bytes<'_> {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        let mut out = py::slice(self.0, address, address + size).to_vec();
        while (out.len() as i64) < size {
            out.push(0);
        }
        Ok(out)
    }
}

/// The size of a zone data file's header: the magic, the count, and an offset a section with one
/// more for the end.
pub fn zone_header(sections: i64) -> i64 {
    4 + 4 * (sections + 1)
}

/// A zone data file as the ROM keeps it once decompressed, validated, or None.
pub fn zone_file(data: &[u8]) -> Result<Option<ZoneData>> {
    read_zone(&Bytes(data), 0)
}

/// The records of one kind in a zone data file held whole (`_records`).
pub fn records(data: &[u8], zone: &ZoneData, kind: i64) -> Result<Vec<Vec<u8>>> {
    read_records(&Bytes(data), zone, kind)
}

/// The zone data file at `address`, if what is there is one: the magic, the game's count of
/// sections, offsets that climb from the header's end, and an entity table sized to its counts.
///
/// Python's `_zone_file`: renamed, as the public `zone_file` takes the name it would have here.
pub fn read_zone(memory: &dyn Memory, address: i64) -> Result<Option<ZoneData>> {
    let head = memory.read(address, zone_header(ZONE_SECTIONS[0]))?;
    let sections = u16(&head, 2)?;
    if py::slice(&head, 0, 2) != ZONE_MAGIC || !ZONE_SECTIONS.contains(&sections) {
        return Ok(None);
    }
    let mut offsets: Vec<i64> = Vec::new();
    for i in 0..sections + 1 {
        offsets.push(u32(&head, 4 + 4 * i)?);
    }
    if offsets[0] != zone_header(sections) || offsets[offsets.len() - 1] > MAX_ZONE_SIZE {
        return Ok(None);
    }
    if offsets.windows(2).any(|pair| pair[0] > pair[1]) {
        return Ok(None);
    }
    let data = ZoneData { address, offsets };
    let table = memory.read(data.entities(), ENTITY_HEADER)?;
    let counts: Vec<i64> = py::slice(&table, 4, 9).iter().map(|&n| n as i64).collect();
    let expected = 8
        + counts[0] * FURNITURE
        + counts[1] * NPC
        + counts[2] * WARP
        + counts[3] * TRIGGER
        + counts[4] * AREA;
    if u32(&table, 0)? != expected {
        return Ok(None);
    }
    Ok(Some(data))
}

/// The entity table's records of one kind: `FURNITURE`, `NPC` or `WARP`.
///
/// Python's `_records`: renamed, as the public `records` takes the name it would have here.
pub fn read_records(memory: &dyn Memory, data: &ZoneData, kind: i64) -> Result<Vec<Vec<u8>>> {
    let head = memory.read(data.entities(), ENTITY_HEADER)?;
    let (furniture, npcs, warps) = (head[4] as i64, head[5] as i64, head[6] as i64);
    let mut at = data.entities() + ENTITY_HEADER;
    let count = if kind == FURNITURE {
        furniture
    } else if kind == NPC {
        at += furniture * FURNITURE;
        npcs
    } else {
        at += furniture * FURNITURE + npcs * NPC;
        warps
    };
    let raw = memory.read(at, count * kind)?;
    Ok((0..count)
        .map(|i| py::slice(&raw, i * kind, (i + 1) * kind).to_vec())
        .collect())
}

/// The entity table's triggers, after its warps.
pub fn read_triggers(memory: &dyn Memory, data: &ZoneData) -> Result<Vec<Vec<u8>>> {
    let head = memory.read(data.entities(), ENTITY_HEADER)?;
    let (furniture, npcs, warps, triggers) = (
        head[4] as i64,
        head[5] as i64,
        head[6] as i64,
        head[7] as i64,
    );
    let at = data.entities() + ENTITY_HEADER + furniture * FURNITURE + npcs * NPC + warps * WARP;
    let raw = memory.read(at, triggers * TRIGGER)?;
    Ok((0..triggers)
        .map(|i| py::slice(&raw, i * TRIGGER, (i + 1) * TRIGGER).to_vec())
        .collect())
}

/// The cell a warp of `zone`'s own table comes out at where it leads back into `zone`: the
/// warp of its slot (`WARP_TO_SLOT`), in the same table. None for a warp into another zone, whose
/// table is not at hand.
pub fn arrival(record: &[u8], zone: Option<i64>, warps: &[Vec<u8>]) -> Result<Option<Cell>> {
    let slot = u16(record, WARP_TO_SLOT)?;
    if Some(u16(record, WARP_TO)?) != zone || slot >= warps.len() as i64 {
        return Ok(None);
    }
    Ok(Some(warp_box(&warps[slot as usize])?.0))
}

/// A warp record's first cell and how many cells it covers, off the rails.
pub fn warp_box(record: &[u8]) -> Result<(Cell, (i64, i64))> {
    Ok((
        (
            from_world(s16(record, WARP_X)?),
            from_world(s16(record, WARP_Z)?),
        ),
        (u16(record, WARP_WIDTH)?, u16(record, WARP_HEIGHT)?),
    ))
}

/// A furniture record's first cell and how many cells it covers, off the rails.
pub fn furniture_box(record: &[u8]) -> Result<(Cell, (i64, i64))> {
    Ok((
        (u16(record, FURNITURE_X)?, u16(record, FURNITURE_Z)?),
        (
            u16(record, FURNITURE_WIDTH)?,
            u16(record, FURNITURE_HEIGHT)?,
        ),
    ))
}

/// What is read by facing it: signs outdoors, the notebook and the clock indoors. NPCs come from
/// the models in use, since a record is where one started and half of them are hidden by the
/// story; triggers and areas are scripts, not places.
pub fn read_furniture(memory: &dyn Memory, data: &ZoneData) -> Result<Vec<Interactable>> {
    let mut out: Vec<Interactable> = Vec::new();
    for (i, record) in read_records(memory, data, FURNITURE)?.iter().enumerate() {
        if !on_rail(record)? {
            let (cell, size) = furniture_box(record)?;
            out.push(Interactable {
                size,
                code: u16(record, FURNITURE_SCRIPT)?,
                source: place_source(FURNITURE_SOURCE, i as i64)?,
                ..Interactable::new("signs", "sign", cell)
            });
        }
    }
    Ok(out)
}

/// Whether a furniture or warp record is placed on a rail rather than a cell.
pub fn on_rail(record: &[u8]) -> Result<bool> {
    Ok(u16(record, PLACEMENT)? == ON_RAIL)
}

pub const TRIGGER_PLACEMENT: i64 = 0x08;

/// A trigger placed on a rail: (line, a, x, cells a, cells x), as a furniture record's
/// (`rail.placed`) but 4 bytes on, its placement at +0x08 and the five from +0x0c, where one on
/// the grid has its cell and size. Coumarine Gym's one trigger, script 0x2b, lies so on its top
/// level's east street, `0x0101fbb9` at (-1, 1), and drops the player walking onto it east to
/// the ground (spike 250). None for a trigger on the grid.
pub fn trigger_placed(record: &[u8]) -> Result<Option<(i64, i64, i64, i64, i64)>> {
    if u16(record, TRIGGER_PLACEMENT)? != ON_RAIL {
        return Ok(None);
    }
    Ok(Some((
        u32(record, TRIGGER_X)?,
        s16(record, TRIGGER_WIDTH)?,
        s16(record, TRIGGER_HEIGHT)?,
        s16(record, TRIGGER_Y)?,
        s16(record, TRIGGER_Y + 2)?,
    )))
}

/// The cell a place in world units is in, as a warp record gives its place.
pub fn from_world(units: i64) -> i64 {
    py::floordiv(units - CELL / 2, CELL)
}

/// Every place `needle` starts in `page`.
pub fn find_all(page: &[u8], needle: &[u8]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    let mut at = find(page, needle, 0);
    while let Some(index) = at {
        out.push(index);
        at = find(page, needle, index + 1);
    }
    out
}

/// `bytes.find(needle, start)`.
fn find(page: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    let tail = page.get(start..)?;
    let first = *needle.first()?;
    let mut from = 0;
    while let Some(skip) = tail[from..].iter().position(|&byte| byte == first) {
        let at = from + skip;
        if tail[at..].starts_with(needle) {
            return Some(start + at);
        }
        from = at + 1;
    }
    None
}
