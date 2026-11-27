//! The names the executable exports, out of the ROM's `static.crs`: an anchor read from the
//! player's own ROM, so a variable the game reaches by name is found as the game finds it.
//!
//! A 3DS game loads code modules (CROs, `pa3ds.core.modules`), and the executable describes itself
//! to them in the same format, `static.crs` in the RomFS, whose named exports give the addresses of
//! functions and variables the modules use. Sun's and Ultra Sun's name thousands, among them the
//! singletons' instance pointers, `gfl2::base::Singleton<GameSys::GameManager>::m_instance`. The
//! executable sits at its own fixed addresses, and the CRS's segment table holds them as they are:
//!
//! ```text
//! +0x80  'CRO0'
//! +0xc8  u32 the segment table's offset    +0xcc  u32 its count, 12 bytes each:
//!        u32 the segment's address, u32 its size, u32 its kind
//! +0xd0  u32 the named exports' offset     +0xd4  u32 their count, 8 bytes each:
//!        u32 the name's offset in the file, u32 where it is: the segment's index in the low
//!        4 bits, the offset into the segment above them
//! ```
//!
//! The format is read from Azahar's loader (`ldr_ro/cro_helper.h`), not its code.

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::py::{self, Kind, Result};

pub const MAGIC: &[u8] = b"CRO0";
pub const HEADER: i64 = 0x80;
pub const SEGMENTS: i64 = 0xC8;
pub const NAMED: i64 = 0xD0;

/// Every named export of a `static.crs`, by its mangled name, at its address; empty for a file
/// that is not one.
pub fn exports(crs: &[u8]) -> Result<IndexMap<String, i64>> {
    if py::slice(crs, HEADER, HEADER + 4) != MAGIC {
        return Ok(IndexMap::new());
    }
    match named(crs) {
        Err(error) if error.is(Kind::StructError) => Ok(IndexMap::new()),
        other => other,
    }
}

/// `exports`' `try` block, whose `struct.error` it catches.
fn named(crs: &[u8]) -> Result<IndexMap<String, i64>> {
    let (segments_at, segments) = (u32(crs, SEGMENTS)?, u32(crs, SEGMENTS + 4)?);
    let (named_at, named) = (u32(crs, NAMED)?, u32(crs, NAMED + 4)?);
    let mut table: Vec<(i64, i64)> = Vec::new();
    for i in 0..segments {
        let at = segments_at + 12 * i;
        table.push((u32(crs, at)?, u32(crs, at + 4)?));
    }
    let mut out: IndexMap<String, i64> = IndexMap::new();
    for i in 0..named {
        let at = named_at + 8 * i;
        let (name_at, tag) = (u32(crs, at)?, u32(crs, at + 4)?);
        let (index, offset) = (tag & 0xF, tag >> 4);
        // `crs.find(b"\0", name_at)`: -1 where none follows.
        let end = py::slice_from(crs, name_at)
            .iter()
            .position(|&byte| byte == 0)
            .map(|found| name_at + found as i64);
        let Some(end) = end else {
            continue;
        };
        if index >= table.len() as i64 || offset >= table[index as usize].1 {
            continue;
        }
        out.insert(
            py::decode_ascii(py::slice(crs, name_at, end)),
            table[index as usize].0 + offset,
        );
    }
    Ok(out)
}
