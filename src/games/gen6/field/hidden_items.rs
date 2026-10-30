//! Hidden items, what the Dowsing Machine finds: listed under `[pa3ds.extras] hidden_items`
//! (spike 249).
//!
//! No zone's data has one, nor any script or file of the ROM's: the executable lists every hidden
//! item in one table, 14 bytes an item, and the Dowsing Machine's search walks it for the zone the
//! player is in (Omega Ruby's `0x4e49e4`, which a write watchpoint on the machine's record led to):
//!
//! ```text
//! +0x00 u16 the item       +0x02 u16 its number, from 1 in order
//! +0x04 u16 x, +0x06 s16 its height in world units, +0x08 u16 z: the cell it lies in
//! +0x0a u16 the zone       +0x0c u8 0x80, with a chance in the low bits for some (50 for
//!                               Shards and Big Pearls)
//! ```
//!
//! Omega Ruby and Alpha Sapphire list the same 171, 43 of them under the sea, and X 151. The Poké
//! Ball the Dowsing Machine found on Route 110 at (10, 59) is the 27th (or_route_110_dowsing). The
//! table is found by its shape, as the Secret Bases' spots are: the longest run of rows numbered
//! from 1 whose zone and cell are in range. An item is gone once the event flag `Game.hidden_items`
//! plus its number is set: the save's flags (`works.Works`).
//!
//! A second table of 33 rows lies past it, items found only while something else holds (their flags
//! carry 0x8000), each with several to choose from: not read.
//!
//! An item a trigger gives is listed with them (`Game.hidden_triggers`): the Scanner in Sea
//! Mauville's sunken control room, whose trigger is armed while work `0x400c` holds 1 and whose
//! script sets it to 0 once the Scanner is taken.
//!
//! Bound by:
//! - ADR-0017: an Extra, shipping off. Under the sea it says what a sighted player is taken to see
//!   there, where the Dowsing Machine cannot be used (Orion, 2026-10-26); everywhere, what the game
//!   keeps from everyone.

use std::rc::Rc;

use crate::core::binary;
use crate::core::memory::Memory;
use crate::core::pathfind::Cell;
use crate::games::common::field::things::{HIDDEN_SOURCE, Interactable, place_source};
use crate::games::gen6::field::features::ZoneAt;
use crate::games::gen6::field::works::{FLAGS, Works};
use crate::py::{Error, Result};

/// What one is listed as: the game shows a glint at most, and never names it.
pub const HIDDEN_ITEM: &str = "hidden item";

/// `struct.Struct("<HHHhHHBx")`: a row's item, number, x, height, z, zone and bits. Python's
/// `Row` is the tuple a row is read into, so the struct is `RowFormat` here.
pub struct RowFormat;

pub const ROW: RowFormat = RowFormat;

impl RowFormat {
    /// `ROW.size`.
    pub fn size(&self) -> i64 {
        14
    }

    /// `ROW.unpack_from(blob, offset)`.
    #[track_caller]
    pub fn unpack_from(&self, blob: &[u8], offset: i64) -> Result<[i64; 7]> {
        if offset < 0 || offset + self.size() > blob.len() as i64 {
            return Err(Error::unpack(format!(
                "unpack_from requires a buffer of at least {} bytes for unpacking {} bytes at \
                 offset {offset} (actual buffer size is {})",
                offset + self.size(),
                self.size(),
                blob.len()
            )));
        }
        Ok([
            binary::u16(blob, offset)?,
            binary::u16(blob, offset + 2)?,
            binary::u16(blob, offset + 4)?,
            binary::s16(blob, offset + 6)?,
            binary::u16(blob, offset + 8)?,
            binary::u16(blob, offset + 10)?,
            i64::from(blob[offset as usize + 12]),
        ])
    }

    /// `ROW.pack(*values)`: a value out of its field's range is `struct.error`.
    #[track_caller]
    pub fn pack(&self, values: [i64; 7]) -> Result<Vec<u8>> {
        const FIELDS: [(char, usize, i64, i64); 7] = [
            ('H', 2, 0, 0xFFFF),
            ('H', 2, 0, 0xFFFF),
            ('H', 2, 0, 0xFFFF),
            ('h', 2, -0x8000, 0x7FFF),
            ('H', 2, 0, 0xFFFF),
            ('H', 2, 0, 0xFFFF),
            ('B', 1, 0, 0xFF),
        ];
        let mut out = Vec::new();
        for (value, (code, width, least, most)) in values.into_iter().zip(FIELDS) {
            if !(least..=most).contains(&value) {
                return Err(Error::unpack(format!(
                    "'{code}' format requires {least} <= number <= {most}"
                )));
            }
            out.extend_from_slice(&value.to_le_bytes()[..width]);
        }
        // The pad byte, `x`.
        out.push(0);
        Ok(out)
    }
}

/// Set in every row's last byte.
pub const MARKED: i64 = 0x80;
pub const ZONE_LIMIT: i64 = 0x400;
pub const MATRIX_CELLS: i64 = 0x400;
/// Fewer rows than this in a run is not the table: Omega Ruby's has 171.
pub const ITEMS_LEAST: i64 = 100;

/// A hidden item as the table has it: its zone, its cell, its height and its item.
pub type Row = (i64, Cell, i64, i64);

/// Where `_FIRST_ROWS` matches: `(?=..\x01\x00.{12}\x02\x00)` is a look-ahead, which the `regex`
/// crate has none of, so the first two rows' numbers are tested at each offset by hand. Python's
/// `finditer` yields a match at each offset the look-ahead holds at.
fn first_rows(image: &[u8]) -> Vec<usize> {
    let row = ROW.size() as usize;
    let mut out = Vec::new();
    if image.len() < row + 4 {
        return out;
    }
    for at in 0..=image.len() - (row + 4) {
        if image[at + 2..at + 4] == [1, 0] && image[at + row + 2..at + row + 4] == [2, 0] {
            out.push(at);
        }
    }
    out
}

/// Every hidden item the executable lists, by its number less one, or none where it lists
/// none.
pub fn rows(image: &[u8]) -> Vec<Row> {
    let mut best: Vec<Row> = Vec::new();
    for start in first_rows(image) {
        let mut run: Vec<Row> = Vec::new();
        let mut at = start as i64;
        while at <= image.len() as i64 - ROW.size() {
            let [item, number, x, y, z, zone, bits] = ROW
                .unpack_from(image, at)
                .expect("the loop's bound leaves a whole row");
            if number != run.len() as i64 + 1
                || bits & MARKED == 0
                || !(0 < zone && zone < ZONE_LIMIT)
                || !(0..MATRIX_CELLS).contains(&x)
                || !(0..MATRIX_CELLS).contains(&z)
            {
                break;
            }
            run.push((zone, (x, z), y, item));
            at += ROW.size();
        }
        // `max(best, run, key=len)` keeps the first of two as long.
        if run.len() > best.len() {
            best = run;
        }
    }
    if best.len() as i64 >= ITEMS_LEAST {
        best
    } else {
        Vec::new()
    }
}

/// `Callable[[int], bool]`: whether to list them in a zone.
pub type Only = Rc<dyn Fn(i64) -> Result<bool>>;

/// The hidden items of the zone the player is in that are still to be found, out of the
/// executable's table and the save's flags (`gen6.field.Feature`).
pub struct HiddenItems {
    memory: Rc<dyn Memory>,
    works: Rc<Works>,
    rows: Vec<Row>,
    flags: i64,
    only: Option<Only>,
}

impl HiddenItems {
    /// `works` holds the save's flags; `flags` is the flag before the first item's; `only`
    /// says which zones to list them in, under the sea, or None for every zone.
    pub fn new(
        memory: Rc<dyn Memory>,
        works: Rc<Works>,
        image: &[u8],
        flags: i64,
        only: Option<Only>,
    ) -> Self {
        HiddenItems {
            memory,
            works,
            rows: rows(image),
            flags,
            only,
        }
    }

    /// Each hidden item in `zone` not yet found; none before a sweep has found the flags.
    pub fn listed(&self, zone: i64, _zone_at: ZoneAt) -> Result<Vec<Interactable>> {
        let Some(event) = self.works.event.get() else {
            return Ok(Vec::new());
        };
        if let Some(only) = &self.only
            && !only(zone)?
        {
            return Ok(Vec::new());
        }
        let here: Vec<(i64, Row)> = (1..)
            .zip(self.rows.iter().copied())
            .filter(|(_, row)| row.0 == zone)
            .collect();
        if here.is_empty() {
            return Ok(Vec::new());
        }
        let first = (self.flags + 1) >> 3;
        let bits = self.memory.read(
            event + FLAGS + first,
            ((self.flags + self.rows.len() as i64) >> 3) - first + 1,
        )?;
        let mut out = Vec::new();
        for (number, (_zone, cell, y, item)) in here {
            let flag = self.flags + number;
            let byte = *bits
                .get(((flag >> 3) - first) as usize)
                .ok_or_else(|| Error::index("index out of range"))?;
            if i64::from(byte) >> (flag & 7) & 1 != 0 {
                continue;
            }
            out.push(Interactable {
                code: item,
                home: Some(zone),
                height: Some(y as f64),
                source: place_source(HIDDEN_SOURCE, number)?,
                faced: true,
                ..Interactable::new("items", HIDDEN_ITEM, cell)
            });
        }
        Ok(out)
    }
}
