//! The zone table: every zone's info, a row a zone.
//!
//! **Zone table** — every zone's info, 0x38 bytes a row indexed by zone id, the ROM's own file
//! loaded whole, so the current zone's info is also its row and the table starts that zone's rows
//! before it (spike 100). Row +0x1a is the parent outdoor
//! zone, +0x08 an area kind — a town and its plain houses share one, and the buildings with a
//! name of their own have theirs: 0x10 a Pokémon Center, 0x45 a Poké Mart, 0x11 a Gym room — and
//! +0x1c a u16 whose low ten bits are the zone's place-name index, for when names come from the
//! ROM (spikes 85 and 86).

use std::cell::{Cell as Slot, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u16;
use crate::core::discovery::{Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::games::gen6::field::zonedata::find_all;
use crate::py::{self, Result};

pub const INFO_SIZE: i64 = 0x38;

/// The map matrix a zone is on, a file of `a/0/4/2` in X: Lumiose's zones 0x3f and 0x40 are both
/// South Boulevard's, 128 (spike 107).
pub const INFO_MATRIX: i64 = 0x04;

/// What a zone with no matrix of its own has at `INFO_MATRIX`: ORAS's placeholder zones 0 to 5,
/// and 39 rooms, the Trick House's among them, none of them walked between across an edge.
pub const NO_MATRIX: i64 = 0;

/// More rows than any game's zone table has: ORAS has 536, X 360.
pub const ZONE_LIMIT: i64 = 0x400;

/// An area kind: a town and its plain houses share one, and a building with a name of its own has
/// its own, the same in every town (spike 86).
pub const INFO_KIND: i64 = 0x08;

/// The area kind of every zone under the sea, and of no other: ORAS's under Routes 107, 124, 126 to
/// 130 and 134, under Sootopolis City, and Seafloor Cavern's first room, which the player swims
/// into (spike 165).
pub const UNDERWATER: i64 = 0xC2;
/// The area kind of X's gates between a town and a route, every one of its 16 and nothing else;
/// no zone of ORAS has it (2026-10-28).
pub const GATE: i64 = 0x47;

/// What kind of place a zone is, a byte: 1 the open air, every town and route of both games and
/// Frost Cavern's and the Poké Ball Factory's grounds, and 6 X's two woods; 2 to 5 under a roof, as
/// houses, caves, the S.S. Tidal, Pokémon Centers and X's gates are (2026-10-19).
pub const INFO_SETTING: i64 = 0x20;

/// Python's is a frozenset, only tested.
pub const OPEN_AIR: [u8; 2] = [1, 6];

pub const INFO_PARENT: i64 = 0x1A;

pub const INFO_NAME: i64 = 0x1C;

/// The place-name index is the low ten bits of +0x1c; the bits above are flags (spike 85).
pub const NAME_MASK: i64 = 0x3FF;

/// Every address in these pages where `needle` starts, a copy that runs on into the page after
/// included.
pub fn spanning(pages: &IndexMap<i64, Vec<u8>>, needle: &[u8]) -> Vec<i64> {
    let mut out: Vec<i64> = Vec::new();
    for (&base, page) in pages {
        let following: &[u8] = pages
            .get(&(base + page.len() as i64))
            .map_or(&[], Vec::as_slice);
        let mut joined = page.clone();
        joined.extend_from_slice(py::slice(following, 0, needle.len() as i64 - 1));
        for at in find_all(&joined, needle) {
            if at < page.len() {
                out.push(base + at as i64);
            }
        }
    }
    out
}

/// The game's zone table, once a sweep has found it: a zone's row, its place and its matrix.
pub struct ZoneTable {
    memory: Rc<dyn Memory>,
    /// The ROM's zone table, which the game's is a copy of; empty without the ROM.
    rom: Vec<u8>,
    /// Where the table starts, or None until a sweep has found it.
    pub base: Slot<Option<i64>>,
    /// A zone data file's info -> the zone it is, once looked up (`home`).
    homes: RefCell<HashMap<Vec<u8>, i64>>,
}

impl ZoneTable {
    /// `rom` is the ROM's zone table, which the game's is a copy of; empty without the ROM.
    pub fn new(memory: Rc<dyn Memory>, rom: Vec<u8>) -> Self {
        ZoneTable {
            memory,
            rom,
            base: Slot::new(None),
            homes: RefCell::new(HashMap::new()),
        }
    }

    /// Look for the table through the current zone's info: the same bytes sit in the table's
    /// row for that zone, so the table starts that many rows before them, and something in
    /// the heap points at its start. `own` is where the info itself is, which is no row.
    ///
    /// The zone the info is for is one of `zones`: the one the player's model was built in, or
    /// the one the matrix puts them in once they have walked on. A row can run from one page
    /// into the next, as Route 4's did in X; searched a page at a time it was never found (spike
    /// 100). Which row the word before the table's pointer leads to is not the current zone's
    /// in X either (it was the zone the player had come from), so the table is found by its
    /// start.
    ///
    /// Its start is taken where a word points at it, or where it is the ROM's zone table byte
    /// for byte: continued from a save in Fortree City, no word the sweep reads pointed at it
    /// in ten sweeps, and every warp was named without it (omega_ruby_tree-city-regression.ctm,
    /// 2026-10-06).
    pub fn find(&self, found: &Found, info: &[u8], own: i64, zones: &py::PySet) -> Result<()> {
        if self.base.get().is_some() || !info.iter().any(|&byte| byte != 0) {
            return Ok(());
        }
        let parent = u16(info, INFO_PARENT)?;
        for row in spanning(&found.pages, info) {
            if row == own {
                continue;
            }
            for zone in zones.iter() {
                let base = row - zone * INFO_SIZE;
                if !within(HEAP_SPAN, base) {
                    continue;
                }
                let held = !found.holders(&[base])?.is_empty()
                    || (!self.rom.is_empty()
                        && self.memory.read(base, self.rom.len() as i64)? == self.rom);
                if !held {
                    continue;
                }
                // The info's parent is an outdoor zone, which is its own parent in the table.
                if u16(
                    &self.memory.read(base + parent * INFO_SIZE, INFO_SIZE)?,
                    INFO_PARENT,
                )? == parent
                {
                    self.base.set(Some(base));
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// A zone's row: its parent zone at +0x1a, and at +0x1c a u16 whose low ten bits are its
    /// place-name index, for naming exits once names come from the ROM (spike 85; the bits
    /// above are flags, and 0x400 marks the zone that carries the place's name). None without
    /// the table.
    pub fn row(&self, zone: i64) -> Result<Option<Vec<u8>>> {
        let Some(base) = self.base.get() else {
            return Ok(None);
        };
        Ok(Some(self.memory.read(base + zone * INFO_SIZE, INFO_SIZE)?))
    }

    /// Every row the table may hold, or None without it.
    pub fn rows(&self) -> Result<Option<Vec<u8>>> {
        let Some(base) = self.base.get() else {
            return Ok(None);
        };
        Ok(Some(self.memory.read(base, ZONE_LIMIT * INFO_SIZE)?))
    }

    /// The zone's place-name index, or None without the table: the same for a town and
    /// every building in it, so it changes only when the player arrives somewhere.
    pub fn place_index(&self, zone: i64) -> Result<Option<i64>> {
        match self.row(zone)? {
            None => Ok(None),
            Some(row) => Ok(Some(u16(&row, INFO_NAME)? & NAME_MASK)),
        }
    }

    /// Which zone a zone data file with this info is: the zone whose row is a copy of it,
    /// every row different from every other in both games (spike 107). The matrix can put the
    /// player in the next zone a poll before its data is in, and a door of the town was named
    /// for the door of the next zone's with its number. The zones `likely` are looked at first;
    /// None without the table.
    pub fn home(&self, info: &[u8], likely: &[i64]) -> Result<Option<i64>> {
        if self.base.get().is_none() {
            return Ok(None);
        }
        if let Some(&zone) = self.homes.borrow().get(info) {
            return Ok(Some(zone));
        }
        for &zone in likely {
            if self.row(zone)?.as_deref() == Some(info) {
                self.homes.borrow_mut().insert(info.to_vec(), zone);
                return Ok(Some(zone));
            }
        }
        let rows = self.rows()?.unwrap_or_default();
        for zone in 0..ZONE_LIMIT {
            if py::slice(&rows, zone * INFO_SIZE, (zone + 1) * INFO_SIZE) == info {
                self.homes.borrow_mut().insert(info.to_vec(), zone);
                return Ok(Some(zone));
            }
        }
        Ok(None)
    }

    /// The zone's matrix, or None without the table or for `NO_MATRIX`.
    pub fn matrix(&self, zone: i64) -> Result<Option<i64>> {
        let matrix = match self.row(zone)? {
            None => None,
            Some(row) => Some(u16(&row, INFO_MATRIX)?),
        };
        Ok(if matrix == Some(NO_MATRIX) {
            None
        } else {
            matrix
        })
    }

    /// Whether a zone is one of the outdoors, or None without the table: its own parent, or
    /// on its parent's matrix, as the foot of Route 119 is on its top's. Taking only the
    /// first, the way from Route 118 onto Route 119 was said as stairs (spike 159).
    pub fn outdoors(&self, zone: i64) -> Result<Option<bool>> {
        let Some(row) = self.row(zone)? else {
            return Ok(None);
        };
        let parent = u16(&row, INFO_PARENT)?;
        let matrix = self.matrix(zone)?;
        if parent == zone {
            return Ok(Some(true));
        }
        Ok(Some(match matrix {
            None => false,
            Some(matrix) => self.matrix(parent)? == Some(matrix),
        }))
    }

    /// Whether a zone is under the sea, its kind `UNDERWATER`: read out of the ROM's table,
    /// or the game's where there is no ROM; False with neither.
    pub fn underwater(&self, zone: i64) -> Result<bool> {
        let at = zone * INFO_SIZE + INFO_KIND;
        if self.rom.len() as i64 >= at + 2 {
            return Ok(u16(&self.rom, at)? == UNDERWATER);
        }
        Ok(match self.row(zone)? {
            None => false,
            Some(row) => u16(&row, INFO_KIND)? == UNDERWATER,
        })
    }

    /// Whether a zone is the open air (`INFO_SETTING`): read out of the ROM's table, or the
    /// game's where there is no ROM; False with neither.
    pub fn open_air(&self, zone: i64) -> Result<bool> {
        let at = zone * INFO_SIZE + INFO_SETTING;
        if self.rom.len() as i64 > at {
            return Ok(OPEN_AIR.contains(&self.rom[at as usize]));
        }
        Ok(match self.row(zone)? {
            None => false,
            Some(row) => OPEN_AIR.contains(&row[INFO_SETTING as usize]),
        })
    }

    /// Whether two zones are on one matrix, walked between with no warp: Route 119's foot and
    /// its top, which the game calls by one name (spike 159).
    pub fn one_matrix(&self, here: i64, there: i64) -> Result<bool> {
        let matrix = self.matrix(here)?;
        Ok(match matrix {
            None => false,
            Some(matrix) => Some(matrix) == self.matrix(there)?,
        })
    }
}
