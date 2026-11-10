//! Sun's zones out of the player's ROM: what each is called, and every area's events (spikes 113
//! and 115).
//!
//! **Zone table** — `a/0/7/7` file 0, LZ11, 0x54 bytes a zone, 295 in Sun, which is also the file
//! the game loads into RAM. +0x1c u16 is the zone's line of system text file 67, the place names:
//! the player's house, zones 0x1a to 0x1d, is line 6, "Route 1". +0x3a u16 is what kind of place
//! the zone is: 3 outdoors (43 zones: every route, town and sea), 0 indoors (127), 2 a cave, trail,
//! ruin or garden (123: Mahalo Trail, Ten Carat Hill), 1 two of Iki Town's. +0x0c u16 is the zone's
//! own file of story text (the bedroom's is 524, "It's a Wii U!").
//!
//! **Event data** — `a/0/8/2` holds eleven files an **area**, and an area's first is its events,
//! magic 'ED'. The game keeps the number of the area it has loaded in RAM (`field`), so the files
//! are found by it: area 21, the player's house, is files 231 to 241; area 0 is Route 1 (zones 0, 1
//! and 6), area 1 Iki Town (zone 0xa).
//!
//! ```text
//! ED       'ED', u16 16 sections, and a u32 offset a section and one for the end
//! section  a two-letter tag, u16 the area's zones, and a u32 offset a zone and one for the
//!          end, from the section's start; a zone's part is a u32 count and its records, and
//!          after them the shapes and data the records point at, from the part's start
//! ```
//!
//! Every record opens with a u32 saying what it is, then f32 x, y, z and a quaternion; +0x2c u32
//! numbers it within its area. Nothing in the zone table or the area's other files says which zone
//! a part is: a warp's +0x34 does, so a zone with no warp (33 of 295) is in no area here.
//!
//! ```text
//! 1  people   0x78 a record: +0x24 u32 the flag that hides it, which the new game's script
//!             sets (spike 291; spike 115 took it for a script), +0x34 u32 the model it is
//!             drawn as (0x77, 0x04, 0x21 in the bedroom, as RAM has them), +0x38 u32 what it
//!             runs when talked to, a case of the zone's script below 0x100
//!             (`script.TALK_CASES`), +0x70 and +0x74 offsets to more of its own
//! 2  warps    0x58: +0x30 u32 the number of the warp it comes out at, +0x34 u16 its zone,
//!             +0x36 u16 the zone it comes out in, +0x38 u32 its type (`WARP_*`), +0x3c f32 x,
//!             y, z where a player coming in through it is put, +0x54 u32 its shape
//! 3  things   0x3c, read by facing them: +0x24 u32 script, +0x28 u32 1 on 95 of them (as many
//!             as Sun has Zygarde Cells), +0x30 u32 the case of the zone's script it runs,
//!             +0x34 u32 its kind where the script is 0 (`SIGNS`), +0x38 u32 its shape
//! 7  trainers 0x54: +0x30 u32 the model it is drawn as, +0x34 u32 `TRAINER_FLAGS` plus its
//!             number in the trainer table (Route 1's Lass Madison is 1001), +0x3c to +0x48
//!             four offsets, two of them to boxes of 80 and 40 by 150 around it
//! 10 items    0x40: +0x24 u32 one number apart for each item of an area, +0x30 u32 the item
//!             (system text file 36), +0x34 u16 how many, +0x38 u32 how it lies (`ITEM_BALL`,
//!             `HIDDEN_ITEM`), +0x3c u32 a box of 60 by 150
//! shape       u32 1, u32 kind: 0 a box, its centre and f32 width (x) and depth (z); 1 a
//!             circle, its centre, a quaternion and f32 radius; 2 a line, two ends and f32 a
//!             height
//! ```
//!
//! The bedroom's door, 0x12d in zone 0x1a, comes out at 0x12f in zone 0x1b, the living room, and
//! 0x12f comes out at 0x12d in 0x1a; the house's front door comes out in zone 0, whose place is
//! Route 1. A warp's number is its area's own: 13 numbers serve the whole game's warps.
//!
//! Only the words and numbers the game itself holds are read; what they mean on the field is
//! `field`'s.

use indexmap::IndexMap;

use crate::core::binary::{self, u16, u32};
use crate::core::rom::{Rom, lz11};
use crate::games::common::garc::{garc_files, garc_spans};
use crate::games::sm::text::SunText;
use crate::py::{self, Error, Kind, Result};

pub const ZONE_TABLE: &str = "a/0/7/7";
pub const ZONE_ROW: i64 = 0x54;
pub const ROW_PLACE: i64 = 0x1C;
pub const ROW_KIND: i64 = 0x3A;
/// System text file 67: "Route 1", "Iki Town", "Hau'oli City".
pub const PLACE_NAMES: i64 = 67;
/// A zone's kind (`ROW_KIND`): a room, a cave or trail or ruin, and a route or town or sea.
pub const INDOORS: i64 = 0;
/// A zone's kind (`ROW_KIND`): a room, a cave or trail or ruin, and a route or town or sea.
pub const ENCLOSED: i64 = 2;
/// A zone's kind (`ROW_KIND`): a room, a cave or trail or ruin, and a route or town or sea.
pub const OUTDOORS: i64 = 3;

pub const EVENTS: &str = "a/0/8/2";
/// Files an area has in `EVENTS`, its events first.
pub const AREA_FILES: i64 = 11;
pub const ED: &[u8] = b"ED";
pub const PEOPLE: i64 = 1;
pub const WARPS: i64 = 2;
pub const THINGS: i64 = 3;
pub const TRAINERS: i64 = 7;
pub const ITEMS: i64 = 10;
pub const PERSON_RECORD: i64 = 0x78;
pub const WARP_RECORD: i64 = 0x58;
pub const THING_RECORD: i64 = 0x3C;
pub const TRAINER_RECORD: i64 = 0x54;
pub const ITEM_RECORD: i64 = 0x40;
pub const NUMBER: i64 = 0x2C;
pub const PERSON_FLAG: i64 = 0x24;
pub const PERSON_MODEL: i64 = 0x34;
pub const PERSON_TALK: i64 = 0x38;
pub const WARP_ZONE: i64 = 0x34;
pub const WARP_TO: i64 = 0x36;
pub const WARP_TYPE: i64 = 0x38;
pub const WARP_SHAPE: i64 = 0x54;
pub const THING_SCRIPT: i64 = 0x24;
pub const THING_CODE: i64 = 0x30;
pub const THING_KIND: i64 = 0x34;
pub const THING_SHAPE: i64 = 0x38;
pub const TRAINER_MODEL: i64 = 0x30;
pub const TRAINER_FLAG: i64 = 0x34;
/// A trainer's flag less its number in the trainer table (`a/1/0/5`): every one of the game's 180
/// trainer records is a trainer, and the 47 models they stand as are each one class (spike 291).
pub const TRAINER_FLAGS: i64 = 1000;
pub const ITEM_ITEM: i64 = 0x30;
pub const ITEM_LIES: i64 = 0x38;
pub const ITEM_SHAPE: i64 = 0x3C;
pub const BOX: i64 = 0;
pub const CIRCLE: i64 = 1;
pub const LINE: i64 = 2;

/// A line walked across into the next area, paired with one of the same type there: Route 1 to
/// Iki Town, Route 1 to Hau'oli City (84 of 586).
pub const WARP_EDGE: i64 = 0;
/// A box: a door in, paired with a `WARP_OUT` on the other side (100) or with another door
/// between two rooms (40).
pub const WARP_DOOR: i64 = 1;
/// A box leading to another place in the same zone, 248 units higher or lower: two pairs, in
/// zone 0x25 on Route 1 and in Vast Poni Canyon.
pub const WARP_UP: i64 = 3;
/// A box leading to another place in the same zone, 248 units higher or lower: two pairs, in
/// zone 0x25 on Route 1 and in Vast Poni Canyon.
pub const WARP_DOWN: i64 = 4;
/// A line with no door, paired with another: a cave's mouth, the way onto Mahalo Trail, from one
/// floor of a cave to the next (194 pairs).
pub const WARP_OPENING: i64 = 8;
/// A line: the mat out of a building, back through the door that led in.
pub const WARP_OUT: i64 = 9;

/// The kinds of the things that are signs (script 0). Route 1 has three of 0x12, three of 0x15
/// and one of 0x0e, and story text file 21, the game's signs, has seven for Route 1: "Route 1",
/// "Pokémon Research Lab" and "Ten Carat Hill", three "Trainer Tips", and "↓ Iki Town". Which
/// line a sign shows is chosen by its zone's script, so the words are not read.
pub const SIGNS: [i64; 3] = [0x0E, 0x12, 0x15];
/// How an item lies (`ITEM_LIES`) when it is a ball on the ground: 232 of them, the Potions and
/// Poké Balls.
pub const ITEM_BALL: i64 = 2;
/// An item that shows nothing: 132, the Pearls, the Plates and a Rare Candy.
pub const HIDDEN_ITEM: i64 = 0;

/// World units a position in RAM may differ from the ROM's and be the same record: the actors
/// hold the record's own floats, equal to the bit in every one of the bedroom's ten.
pub const SAME: f64 = 0.01;

/// Where a record reaches on the ground, as the world's x and z: west, north, east, south.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub west: f64,
    pub north: f64,
    pub east: f64,
    pub south: f64,
}

/// One record of an area's events, where the game places it.
///
/// `Record(x, z, shape, zone=...)` is `Record { x, z, shape, zone, ..Default::default() }`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Record {
    pub x: f64,
    pub z: f64,
    pub shape: Option<Shape>,
    /// The zone whose part holds it: for a warp its own +0x34, for the rest the zone of the
    /// part's warps, or None in a part without any.
    pub zone: Option<i64>,
    /// For a warp, the zone it comes out in.
    pub leads_to: Option<i64>,
    /// For a thing the script it runs, and for a person the case of the zone's script it runs
    /// when talked to (`PERSON_TALK`).
    pub script: i64,
    /// For a warp its type (`WARP_*`), for a thing its kind (`SIGNS`), for an item how it lies
    /// (`ITEM_BALL`), for a trainer their number in the trainer table.
    pub kind: i64,
    /// For a person or a trainer the model it is drawn as, for an item the item, for a thing its
    /// +0x30.
    pub code: i64,
    /// Its number within its area.
    pub number: i64,
}

/// One area's records, a list a zone of the area, in the area's order.
///
/// `Area(warps, things)` is `Area { warps, things, ..Default::default() }`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Area {
    pub warps: Vec<Vec<Record>>,
    pub things: Vec<Vec<Record>>,
    pub people: Vec<Vec<Record>>,
    pub items: Vec<Vec<Record>>,
    pub trainers: Vec<Vec<Record>>,
}

/// Sun's zones as its ROM has them: their place names, and every area's records.
#[derive(Debug, Clone, PartialEq)]
pub struct SunZones {
    pub rows: Vec<u8>,
    pub places: SunText,
    pub areas: IndexMap<i64, Area>,
}

impl SunZones {
    /// The zone table, the place names in `language`, and every area's events. A RomError
    /// where any is not what Sun has.
    pub fn load(rom: &Rom, language: &str) -> Result<SunZones> {
        let files = garc_files(&rom.read(ZONE_TABLE)?)?;
        let Some(table) = files.into_iter().next() else {
            return Err(Error::index("list index out of range"));
        };
        let rows = lz11(&table)?;
        if rows.is_empty() || rows.len() as i64 % ZONE_ROW != 0 {
            return Err(Error::new(
                Kind::RomError,
                format!("{ZONE_TABLE} is not a zone table"),
            ));
        }
        let places = SunText::load(rom, language, &[PLACE_NAMES])?;
        // Each area's first file alone, a few KB: the whole archive is 332 MB, and reading it
        // all held 660 MB at every start.
        let (start, spans) = garc_spans(rom, EVENTS)?;
        let mut areas: IndexMap<i64, Area> = IndexMap::new();
        for number in 0..spans.len() as i64 / AREA_FILES {
            let (begin, end) = spans[(number * AREA_FILES) as usize];
            let data = unpacked(&rom.read_part(EVENTS, start + begin, end - begin)?)?;
            if py::slice(&data, 0, 2) != ED {
                break;
            }
            match area(&data) {
                Ok(found) => {
                    areas.insert(number, found);
                }
                Err(error)
                    if error.is(Kind::RomError)
                        || error.is(Kind::StructError)
                        || error.is(Kind::IndexError) =>
                {
                    return Err(Error::new(
                        Kind::RomError,
                        format!("{EVENTS} area {number}: {error}"),
                    ));
                }
                Err(error) => return Err(error),
            }
        }
        if areas.is_empty() {
            return Err(Error::new(
                Kind::RomError,
                format!("{EVENTS} holds no event data"),
            ));
        }
        Ok(SunZones {
            rows,
            places,
            areas,
        })
    }

    pub fn zones(&self) -> i64 {
        self.rows.len() as i64 / ZONE_ROW
    }

    /// The zone's line of the place names, or None for a zone the table does not have.
    pub fn place_index(&self, zone: i64) -> Result<Option<i64>> {
        self.row(zone, ROW_PLACE)
    }

    /// What the game calls where the zone is: "Route 1" for the player's house.
    pub fn place(&self, zone: i64) -> Result<Option<String>> {
        Ok(match self.place_index(zone)? {
            None => None,
            Some(index) => self.places.line(PLACE_NAMES, index),
        })
    }

    /// `INDOORS`, `ENCLOSED`, `OUTDOORS` or another the table holds, or None for a zone it
    /// does not have.
    pub fn zone_kind(&self, zone: i64) -> Result<Option<i64>> {
        self.row(zone, ROW_KIND)
    }

    pub fn outdoors(&self, zone: i64) -> Result<bool> {
        Ok(self.zone_kind(zone)? == Some(OUTDOORS))
    }

    /// The record of `section` (`WARPS`, `THINGS`, `PEOPLE`, `ITEMS` or `TRAINERS`) this area
    /// places at (x, z), or None: a person who has walked is not.
    pub fn record(&self, area: i64, section: i64, x: f64, z: f64) -> Result<Option<Record>> {
        let lists: &[Vec<Record>] = match self.areas.get(&area) {
            None => &[],
            Some(found) => match section {
                WARPS => &found.warps,
                THINGS => &found.things,
                PEOPLE => &found.people,
                ITEMS => &found.items,
                TRAINERS => &found.trainers,
                _ => return Err(Error::key(section.to_string())),
            },
        };
        for records in lists {
            for record in records {
                if (record.x - x).abs() < SAME && (record.z - z).abs() < SAME {
                    return Ok(Some(record.clone()));
                }
            }
        }
        Ok(None)
    }

    fn row(&self, zone: i64, offset: i64) -> Result<Option<i64>> {
        if !(0 <= zone && zone < self.zones()) {
            return Ok(None);
        }
        Ok(Some(u16(&self.rows, zone * ZONE_ROW + offset)?))
    }
}

/// What a record reader is handed: the part, the record's offset and the part's zone.
type Reader = fn(&[u8], i64, Option<i64>) -> Result<Record>;

pub(crate) fn area(data: &[u8]) -> Result<Area> {
    let mut warps: Vec<Vec<Record>> = Vec::new();
    for part in parts(data, WARPS)? {
        warps.push(records(&part, WARP_RECORD, warp, None)?);
    }
    let homes: Vec<Option<i64>> = warps
        .iter()
        .map(|part| part.first().and_then(|record| record.zone))
        .collect();

    let section = |number: i64, size: i64, read: Reader| -> Result<Vec<Vec<Record>>> {
        let parts = parts(data, number)?;
        let mut out: Vec<Vec<Record>> = Vec::new();
        for (index, part) in parts.iter().enumerate() {
            out.push(records(
                part,
                size,
                read,
                homes.get(index).copied().flatten(),
            )?);
        }
        Ok(out)
    };

    Ok(Area {
        warps,
        things: section(THINGS, THING_RECORD, thing)?,
        people: section(PEOPLE, PERSON_RECORD, person)?,
        items: section(ITEMS, ITEM_RECORD, item)?,
        trainers: section(TRAINERS, TRAINER_RECORD, trainer)?,
    })
}

/// A section's parts, one a zone of the area, or none for a section the file does not have.
fn parts(data: &[u8], section: i64) -> Result<Vec<Vec<u8>>> {
    let sections = pack(data, ED)?;
    let body: &[u8] = if section < sections.len() as i64 {
        &sections[section as usize]
    } else {
        b""
    };
    if body.is_empty() {
        Ok(Vec::new())
    } else {
        pack(body, py::slice(body, 0, 2))
    }
}

/// `magic.decode()`, which Python does strictly: the text of a pack's magic, or a
/// UnicodeDecodeError for a magic that is not UTF-8, as a section's own first two bytes may not
/// be.
fn decode_magic(magic: &[u8]) -> Result<String> {
    match std::str::from_utf8(magic) {
        Ok(text) => Ok(text.to_string()),
        Err(error) => {
            let at = error.valid_up_to();
            let (end, reason) = match error.error_len() {
                None => (magic.len(), "unexpected end of data"),
                Some(len) => {
                    let lead = magic[at];
                    let reason = if (0xC2..=0xF4).contains(&lead) {
                        "invalid continuation byte"
                    } else {
                        "invalid start byte"
                    };
                    (at + len, reason)
                }
            };
            let bytes = if end - at == 1 {
                format!("byte {:#04x} in position {at}", magic[at])
            } else {
                format!("bytes in position {at}-{}", end - 1)
            };
            Err(Error::new(
                Kind::UnicodeDecodeError,
                format!("'utf-8' codec can't decode {bytes}: {reason}"),
            ))
        }
    }
}

/// The entries of a two-letter pack: the magic, u16 count, and u32 offsets, one an entry and
/// one for the end. A RomError if it is not one.
///
/// `pack(data, magic)` is `pack(data, magic, least=0)`: `pack_with` takes `least`.
pub fn pack(data: &[u8], magic: &[u8]) -> Result<Vec<Vec<u8>>> {
    pack_with(data, magic, 0)
}

/// The entries of a two-letter pack: the magic, u16 count, and u32 offsets, one an entry and
/// one for the end. A RomError if it is not one, or has fewer than `least` entries.
pub fn pack_with(data: &[u8], magic: &[u8], least: i64) -> Result<Vec<Vec<u8>>> {
    if py::slice(data, 0, 2) != magic || data.len() < 8 {
        return Err(Error::new(
            Kind::RomError,
            format!("not a {} pack", decode_magic(magic)?),
        ));
    }
    let count = u16(data, 2)?;
    if count < least || 4 + (count + 1) * 4 > data.len() as i64 {
        return Err(Error::new(
            Kind::RomError,
            format!(
                "a {} pack of {count} entries, not {least}",
                decode_magic(magic)?
            ),
        ));
    }
    let mut offsets: Vec<i64> = Vec::new();
    for index in 0..count + 1 {
        offsets.push(u32(data, 4 + index * 4)?);
    }
    let length = data.len() as i64;
    if offsets
        .windows(2)
        .any(|pair| !(pair[0] <= pair[1] && pair[1] <= length))
    {
        return Err(Error::new(
            Kind::RomError,
            format!("a {} pack whose entries run past it", decode_magic(magic)?),
        ));
    }
    Ok((0..count as usize)
        .map(|index| py::slice(data, offsets[index], offsets[index + 1]).to_vec())
        .collect())
}

/// The data, LZ11-decoded where it is packed.
pub fn unpacked(data: &[u8]) -> Result<Vec<u8>> {
    if py::slice(data, 0, 1) == b"\x11" {
        lz11(data)
    } else {
        Ok(data.to_vec())
    }
}

/// A part's records, each read by `read` from the part, the record's offset and the part's
/// zone.
fn records(part: &[u8], size: i64, read: Reader, zone: Option<i64>) -> Result<Vec<Record>> {
    let mut out: Vec<Record> = Vec::new();
    if part.len() < 4 {
        return Ok(out);
    }
    let end = 4 + u32(part, 0)? * size;
    let mut at = 4;
    while at < end {
        out.push(read(part, at, zone)?);
        at += size;
    }
    Ok(out)
}

/// A little-endian f32 of bytes `struct.unpack_from` already took, as the float it is.
fn float(raw: &[u8], index: usize) -> f64 {
    f32::from_le_bytes(raw[index * 4..index * 4 + 4].try_into().expect("4 bytes")) as f64
}

fn place(part: &[u8], at: i64) -> Result<(f64, f64)> {
    let raw = binary::take::<12>(part, at + 4)?;
    Ok((float(&raw, 0), float(&raw, 2)))
}

fn warp(part: &[u8], at: i64, _zone: Option<i64>) -> Result<Record> {
    let (x, z) = place(part, at)?;
    Ok(Record {
        x,
        z,
        shape: shape(part, u32(part, at + WARP_SHAPE)?)?,
        zone: Some(u16(part, at + WARP_ZONE)?),
        leads_to: Some(u16(part, at + WARP_TO)?),
        kind: u32(part, at + WARP_TYPE)?,
        number: u32(part, at + NUMBER)?,
        ..Default::default()
    })
}

fn thing(part: &[u8], at: i64, zone: Option<i64>) -> Result<Record> {
    let (x, z) = place(part, at)?;
    Ok(Record {
        x,
        z,
        shape: shape(part, u32(part, at + THING_SHAPE)?)?,
        zone,
        script: u32(part, at + THING_SCRIPT)?,
        kind: u32(part, at + THING_KIND)?,
        code: u32(part, at + THING_CODE)?,
        number: u32(part, at + NUMBER)?,
        ..Default::default()
    })
}

fn person(part: &[u8], at: i64, zone: Option<i64>) -> Result<Record> {
    let (x, z) = place(part, at)?;
    Ok(Record {
        x,
        z,
        shape: None,
        zone,
        script: u32(part, at + PERSON_TALK)?,
        code: u32(part, at + PERSON_MODEL)?,
        number: u32(part, at + NUMBER)?,
        ..Default::default()
    })
}

fn trainer(part: &[u8], at: i64, zone: Option<i64>) -> Result<Record> {
    let (x, z) = place(part, at)?;
    Ok(Record {
        x,
        z,
        shape: None,
        zone,
        kind: u32(part, at + TRAINER_FLAG)? - TRAINER_FLAGS,
        code: u32(part, at + TRAINER_MODEL)?,
        number: u32(part, at + NUMBER)?,
        ..Default::default()
    })
}

fn item(part: &[u8], at: i64, zone: Option<i64>) -> Result<Record> {
    let (x, z) = place(part, at)?;
    Ok(Record {
        x,
        z,
        shape: shape(part, u32(part, at + ITEM_SHAPE)?)?,
        zone,
        kind: u32(part, at + ITEM_LIES)?,
        code: u32(part, at + ITEM_ITEM)?,
        number: u32(part, at + NUMBER)?,
        ..Default::default()
    })
}

/// `min(a, b)`: `a`, unless `b` is less, so a NaN keeps whichever came first as Python's does.
fn least(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// `max(a, b)`, as `least` is `min`.
fn greatest(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Where a shape reaches, or None for a kind not seen.
fn shape(part: &[u8], at: i64) -> Result<Option<Shape>> {
    if at + 8 > part.len() as i64 {
        return Ok(None);
    }
    let kind = u32(part, at + 4)?;
    if kind == BOX {
        let raw = binary::take::<20>(part, at + 8)?;
        let (x, z, width, depth) = (
            float(&raw, 0),
            float(&raw, 2),
            float(&raw, 3),
            float(&raw, 4),
        );
        return Ok(Some(Shape {
            west: x - width / 2.0,
            north: z - depth / 2.0,
            east: x + width / 2.0,
            south: z + depth / 2.0,
        }));
    }
    if kind == CIRCLE {
        let raw = binary::take::<12>(part, at + 8)?;
        let (x, z) = (float(&raw, 0), float(&raw, 2));
        let radius = float(&binary::take::<4>(part, at + 0x24)?, 0);
        return Ok(Some(Shape {
            west: x - radius,
            north: z - radius,
            east: x + radius,
            south: z + radius,
        }));
    }
    if kind == LINE {
        let raw = binary::take::<24>(part, at + 8)?;
        let (ax, az, bx, bz) = (
            float(&raw, 0),
            float(&raw, 2),
            float(&raw, 3),
            float(&raw, 5),
        );
        return Ok(Some(Shape {
            west: least(ax, bx),
            north: least(az, bz),
            east: greatest(ax, bx),
            south: greatest(az, bz),
        }));
    }
    Ok(None)
}
