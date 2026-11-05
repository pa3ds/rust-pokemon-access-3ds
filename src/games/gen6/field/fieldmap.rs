//! The field map, the matrix of the outdoors the player is in: a heap block of 0x9180 bytes with a
//! grid of u16 zone ids at +0x3c, one per block of 10 by 10 cells, 0xffff where there is no zone,
//! row by row. How wide the grid is depends on the matrix: +0x36 and +0x38 are its width and height
//! in chunks of four blocks, 8 by 8 for Littleroot's and Petalburg's, 10 by 10 for Route 104's, 1 by
//! 1 for the bedroom. Outdoor zones on one matrix share their cells; a building's floor is a matrix
//! of its own.

use indexmap::IndexMap;

use crate::core::binary::{u16, u32};
use crate::core::discovery::ALLOCATION;
use crate::core::memory::Memory;
use crate::games::common::field::chunk::CHUNK;
use crate::games::common::field::ground::{BLOCK, Grid};
use crate::games::gen6::field::zonedata::find_all;
use crate::py::Result;

pub const MAP_BLOCK_SIZE: i64 = 0x9180;

pub const MAP_KIND: i64 = 0x08;

/// 32 in every matrix seen, whatever its width: it was taken for the width until Route 104's
/// matrix, 40 blocks wide, still read 32 here.
pub const MAP_MARK: i64 = 0x0C;

pub const MAP_MARKED: i64 = 32;

pub const MAP_CHUNKS_WIDE: i64 = 0x36;

/// Every matrix seen is square, so which of the two is the width is inferred from the order, not
/// measured.
pub const MAP_CHUNKS_TALL: i64 = 0x38;

pub const MAP_GRID: i64 = 0x3C;

/// Each chunk's file in the ROM's map chunks, a u16 a chunk row by row, after room for a grid of
/// 128 by 128 blocks: 619 to 660 for the part of Lumiose round South Boulevard, 7 chunks by 6, the
/// chunk each street section says it is (spike 106), and in ORAS 43 to 46 for Route 109's four
/// round the beach, each the ROM's file whose cell table the game loads (spike 136).
pub const MAP_CHUNK_FILES: i64 = 0x803C;

pub const BLOCKS_A_CHUNK: i64 = CHUNK / BLOCK;

/// The field map's grid in blocks, (width, height), from its header; None where the sizes are zero
/// or the grid would not fit in the block.
pub fn size(head: &[u8]) -> Result<Option<(i64, i64)>> {
    let width = u16(head, MAP_CHUNKS_WIDE)? * BLOCKS_A_CHUNK;
    let height = u16(head, MAP_CHUNKS_TALL)? * BLOCKS_A_CHUNK;
    if width == 0 || height == 0 || MAP_GRID + width * height * 2 > MAP_BLOCK_SIZE {
        return Ok(None);
    }
    Ok(Some((width, height)))
}

/// The field map in these pages, by its block's size and its marks.
pub fn find(memory: &dyn Memory, pages: &IndexMap<i64, Vec<u8>>) -> Result<Option<i64>> {
    let mut needle: Vec<u8> = Vec::new();
    needle.extend_from_slice(&(ALLOCATION as u32).to_le_bytes());
    needle.extend_from_slice(&(MAP_BLOCK_SIZE as u32).to_le_bytes());
    for (&base, page) in pages {
        for index in find_all(page, &needle) {
            let block = base + index as i64 + needle.len() as i64;
            if valid(memory, block)? {
                return Ok(Some(block));
            }
        }
    }
    Ok(None)
}

/// Whether the field map is at `block`, by its marks and a size it can be.
pub fn valid(memory: &dyn Memory, block: i64) -> Result<bool> {
    let head = memory.read(block, MAP_GRID)?;
    Ok(u32(&head, MAP_KIND)? == 4 && u32(&head, MAP_MARK)? == MAP_MARKED && size(&head)?.is_some())
}

/// The grid of zones as the field map at `address` has it now: the game rewrites the one
/// block in place for each matrix, so its size is read with it.
pub fn grid(memory: &dyn Memory, address: i64) -> Result<Option<Grid>> {
    let Some((width, height)) = size(&memory.read(address, MAP_GRID)?)? else {
        return Ok(None);
    };
    let raw = memory.read(address + MAP_GRID, width * height * 2)?;
    let mut zones: Vec<i64> = Vec::new();
    for i in 0..width * height {
        zones.push(u16(&raw, i * 2)?);
    }
    Ok(Some(Grid {
        width,
        height,
        zones,
    }))
}

/// The matrix's width in chunks and each chunk's file, row by row, as the field map lists
/// them now; (0, ()) where its size is not one.
pub fn files(memory: &dyn Memory, address: i64) -> Result<(i64, Vec<i64>)> {
    let head = memory.read(address, MAP_GRID)?;
    if size(&head)?.is_none() {
        return Ok((0, Vec::new()));
    }
    let wide = u16(&head, MAP_CHUNKS_WIDE)?;
    let count = wide * u16(&head, MAP_CHUNKS_TALL)?;
    let raw = memory.read(address + MAP_CHUNK_FILES, count * 2)?;
    let mut files: Vec<i64> = Vec::new();
    for i in 0..count {
        files.push(u16(&raw, 2 * i)?);
    }
    Ok((wide, files))
}
