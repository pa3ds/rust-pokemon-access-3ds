//! `field::MapBlock`: one loaded chunk of the matrix, and the walkability of its cells.
//!
//! The outdoors is drawn in **chunks** of 40 by 40 cells, four zone blocks each way, and the game
//! keeps four MapBlocks: the chunk the player is in and the three nearest it, reloaded as the
//! player crosses into another quarter of a chunk. A MapBlock holds (spikes 82 and 94):
//!
//! ```text
//! +0x0c f32  the chunk's centre in world x, +0x14 in world z (18 units to a cell)
//! +0x18 ptr  a `gfl::fs::ArcFile`, null for a slot that is empty
//! +0x24 ptr  the chunk's file, decompressed whole into the linear heap
//! +0x90 u32  a size that changes with the chunk (0x2baf8 for Route 101's, 0x242e8 for
//!            Littleroot's, and 0 where the matrix has no chunk, as west of Littleroot); it is
//!            smaller than the file's own section table says the file is, so it is likely the
//!            packed size, and it is read only to tell one chunk's load from the next; a
//!            block's own buffer loaded again reads 0x1000 until seconds after its table is in
//! +0xa8 u32  where the load has got to (spike 94): 2 while the file is being read in, 3 the
//!            moment the cell table is in place, and 4 a poll later, until the next load
//! ```
//!
//! The file starts `GR`, u16 7 sections, and eight u32 offsets, the first 0x80, like the zone's own
//! file. X's has six sections and the same cell table in the first of them (spike 100). Section 0 is
//! the **cell table**: u16 width and height, both 40, then one u32 a cell, row by row from the chunk's
//! north-west corner. **Bit 0 of the word is set where the player cannot walk**: Littleroot's houses,
//! its trees and its two signs read `0x01000021`, its paths `0x20`, Route 101's tall grass
//! `0x20004004` and a ledge `0x75000021`. **The top byte is the cell's kind**, which the game tests
//! with one small function a kind (`word >> 24 == 0xd5`, spike 83): a Pokémon Center's PC is `0xd5`,
//! its counter `0xd4`. A ledge is one of four kinds, one a way it drops (`LEDGES`), and a sandy
//! slope one of four more, one a way it goes down, open but walked down only on foot: Granite Cave's
//! `0x69000020` east, Jagged Pass's `0x6c000020` south (`SLOPES`, spike 158). What the bits
//! between mean is left open.
//!
//! Section 2 is the ground's **mesh**, `coll`: the triangles of the ground's surface, so the height
//! the ground stands at under each cell. A chunk with a deck over its ground, as Mt. Chimney's catwalk
//! over the crater is, has two more sections for each such floor, nine in all for one: its own cell
//! table from section 7, open along the deck where the ground's has the crater as a wall, then its
//! own mesh. 38 of ORAS's 857 chunks have one floor over the ground or two (spike 156).

use std::collections::HashSet;

use indexmap::IndexMap;

use crate::core::binary::{f32, u16, u32};
use crate::core::memory::Memory;
pub use crate::games::common::field::chunk::CLIMB;
use crate::games::common::field::chunk::{BLOCKED, CHUNK, Chunk, Floor};
use crate::py::{self, Result};

/// What a sweep looks for.
pub const CLASS: &str = "N5field8MapBlockE";

pub const CENTRE_X: i64 = 0x0C;
pub const CENTRE_Z: i64 = 0x14;
pub const ARC: i64 = 0x18;
pub const FILE: i64 = 0x24;
pub const SIZE: i64 = 0x90;
pub const STATE: i64 = 0xA8;
/// Enough of a block to hold everything above.
pub const HEAD: i64 = 0xAC;

pub const MAGIC: &[u8] = b"GR";
/// Six in X and Y (spike 100), seven in ORAS, and two more for each floor a chunk has over its
/// ground: nine for Mt. Chimney's catwalk, eleven for two floors (spike 156), and in X eight and ten,
/// 35 of its 1,030 chunk files (spike A). The cell table is section 0 in every one. Mt. Chimney's
/// nine were turned away until 2026-10-02, and P took the crater for open ground and said "20 north"
/// into its wall (omega_ruby_mt-chimney.ctm); X's eight on Route 8 until 2026-10-20, and P said "11
/// west, 23 north" into one (x_route-8.ctm).
pub const SECTIONS: [i64; 6] = [6, 7, 8, 9, 10, 11];
pub const HEADER: i64 = 0x80;
/// The cell table's section.
pub const CELLS: i64 = 0;
/// The ground's mesh.
pub const MESH: i64 = 2;
/// An ORAS chunk's sections with its ground alone: one with more floors has each one's cell
/// table from here, then each one's mesh in the same order.
pub const ONE_FLOOR: i64 = 7;
/// X's, laid out the same way: Route 8's file 87 has its floor's table in section 6 and its mesh
/// in 7, and file 374, of ten, its two floors' tables in 6 and 7 and their meshes in 8 and 9.
pub const X_ONE_FLOOR: i64 = 6;
/// How far apart two open cells side by side of one floor must be to be a drop, not walked
/// between: across ORAS's chunks of one floor, the ground's mesh steps no more than a cell's height
/// between open neighbours but in eight files, where it drops 40 to 200, as Fortree City's
/// platforms in the trees stand 103 to 108 over the ground beside them (spike 160).
pub const DROP: f64 = 2.0 * CLIMB;
pub const COLL: &[u8] = b"coll";
/// A mesh's header: the magic, its size after the first 16 bytes, two words unread, then how deep
/// its tree of boxes goes and how many ways each box splits, across, up and along.
pub const MESH_HEAD: i64 = 0x20;
/// One box of the tree: two corners, four floats each.
pub const BOX: i64 = 0x20;
/// x, y and z in world units from the chunk's centre, and a fourth float.
pub const VERTEX: i64 = 0x10;
/// How far outside a triangle a cell's centre may be and still be under it: a centre on the
/// line between two triangles is under both.
pub const EDGE: f64 = 1e-4;
/// World units per cell.
pub const CELL: i64 = 18;
pub const MAX_FILE: i64 = 0x200000;
pub const READY: i64 = 4;
/// The cell table's size: its width and height, then a word a cell.
pub const TABLE: i64 = 4 + CHUNK * CHUNK * 4;

/// Where a block's chunk sits in the linear heap: enough to know it is still the same
/// chunk without reading the table again.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Loaded {
    pub file: i64,
    pub size: i64,
    pub origin: (i64, i64),
}

/// `min(*values)`: the first value, unless a later one is less. Rust's `f64::min` skips a NaN
/// where Python's `min` keeps whichever comes first, so a float read out of memory needs this.
fn least(values: &[f64]) -> f64 {
    let mut found = values[0];
    for &value in &values[1..] {
        if value < found {
            found = value;
        }
    }
    found
}

/// `max(*values)`, as `least` is `min`.
fn greatest(values: &[f64]) -> f64 {
    let mut found = values[0];
    for &value in &values[1..] {
        if value > found {
            found = value;
        }
    }
    found
}

/// What a block holds, or None for an empty slot, a chunk still loading, or a block that
/// is not one: a chunk's centre is 20 cells in from a chunk boundary, and the block's file and
/// size are set.
///
/// A block takes its new chunk's centre and file the moment a load starts, while the file still
/// holds the chunk the block had before: read then, Rustboro's Pokémon Center stood in another
/// chunk's walls, and no way to it was found (map_chunk.ctm, spike 94).
pub fn loaded(memory: &dyn Memory, block: i64) -> Result<Option<Loaded>> {
    let head = memory.read(block, HEAD)?;
    let (file, size) = (u32(&head, FILE)?, u32(&head, SIZE)?);
    if u32(&head, ARC)? == 0 || file == 0 || size == 0 || u32(&head, STATE)? != READY {
        return Ok(None);
    }
    let (x, z) = (
        f32(&head, CENTRE_X)? / CELL as f64,
        f32(&head, CENTRE_Z)? / CELL as f64,
    );
    // `int(x)` is an int of any size in Python and an `i64` here, which holds a chunk's centre
    // with room to spare; only a centre beyond 9e18 cells, in a block that is not one, differs.
    if x != py::int(x)? as f64 || z != py::int(z)? as f64 {
        return Ok(None);
    }
    let origin = (py::int(x)? - CHUNK / 2, py::int(z)? - CHUNK / 2);
    if py::modulo(origin.0, CHUNK) != 0 || py::modulo(origin.1, CHUNK) != 0 {
        return Ok(None);
    }
    Ok(Some(Loaded { file, size, origin }))
}

/// The chunk's cell table and its floors, or None where the file is not one: the magic, the
/// game's count of sections, offsets that climb from the header's end, and a table 40 by 40.
pub fn read(memory: &dyn Memory, held: &Loaded) -> Result<Option<Chunk>> {
    let Some(offsets) = offsets(&memory.read(held.file, HEADER)?)? else {
        return Ok(None);
    };
    let Some(words) = words(&memory.read(held.file + offsets[CELLS as usize], TABLE)?)? else {
        return Ok(None);
    };
    let floors = floors_in(&offsets, &|i| {
        memory.read(
            held.file + offsets[i as usize],
            offsets[i as usize + 1] - offsets[i as usize],
        )
    })?;
    Ok(Some(Chunk {
        floors,
        ..Chunk::new(held.origin, words)
    }))
}

/// The cell table of a chunk's whole file, as the ROM's map chunks hold it decompressed, or
/// None where it is not one, by the same checks as `read`.
pub fn cells(file: &[u8]) -> Result<Option<Vec<i64>>> {
    let Some(offsets) = offsets(py::slice(file, 0, HEADER))? else {
        return Ok(None);
    };
    if (file.len() as i64) < offsets[CELLS as usize] + TABLE {
        return Ok(None);
    }
    words(py::slice(
        file,
        offsets[CELLS as usize],
        offsets[CELLS as usize] + TABLE,
    ))
}

/// The height of a chunk's ground under each cell's centre, out of its mesh (`heights`), or
/// None where the file is not a chunk's or its mesh is not one.
pub fn ground_heights(file: &[u8]) -> Result<Option<Vec<Option<f64>>>> {
    let Some(offsets) = offsets(py::slice(file, 0, HEADER))? else {
        return Ok(None);
    };
    if (file.len() as i64) < offsets[MESH as usize + 1] {
        return Ok(None);
    }
    heights(py::slice(
        file,
        offsets[MESH as usize],
        offsets[MESH as usize + 1],
    ))
}

/// Every floor of a chunk's whole file, the ground first, or none for a chunk of one floor
/// or a file that is not a chunk's.
pub fn floors(file: &[u8]) -> Result<Vec<Floor>> {
    let Some(offsets) = offsets(py::slice(file, 0, HEADER))? else {
        return Ok(Vec::new());
    };
    if (file.len() as i64) < offsets[offsets.len() - 1] {
        return Ok(Vec::new());
    }
    floors_in(&offsets, &|i| {
        Ok(py::slice(file, offsets[i as usize], offsets[i as usize + 1]).to_vec())
    })
}

/// The chunk as a player standing at `height` walks it, one floor a cell: the open floor
/// nearest their height. Where two cells side by side are on floors that do not meet, as a deck
/// and the ground below its side do, the one further from the player's height is closed, so a
/// way on the deck keeps to it and a way under it stays under (spike 156). A chunk of one floor
/// is the ground's table as it is.
///
/// With no height, each cell is on the first floor open there and no seam is closed: where
/// anybody could stand, as the listing asks. Mt. Chimney's ground alone had Maxie and Tabitha
/// on the catwalk walled in, and the listing left them out (omega_ruby_mt-chimney.ctm).
pub fn flat(chunk: &Chunk, height: Option<f64>) -> Chunk {
    if chunk.floors.is_empty() {
        return chunk.clone();
    }
    let levels = &chunk.floors;
    let mut on: Vec<Option<usize>> = Vec::new();
    for i in 0..CHUNK * CHUNK {
        let (mut best, mut off): (Option<usize>, f64) = (None, f64::INFINITY);
        for (k, floor) in levels.iter().enumerate() {
            if floor.words[i as usize] & BLOCKED != 0 {
                continue;
            }
            let level = floor.heights[i as usize];
            let away = match (level, height) {
                (Some(level), Some(height)) => (level - height).abs(),
                _ => f64::INFINITY,
            };
            if best.is_none() || away < off {
                (best, off) = (Some(k), away);
            }
        }
        on.push(best);
    }
    let mut words: Vec<i64> = on
        .iter()
        .enumerate()
        .map(|(i, &k)| match k {
            None => chunk.words[i],
            Some(k) => levels[k].words[i],
        })
        .collect();
    let Some(height) = height else {
        return Chunk::new(chunk.origin, words);
    };
    // ponytail: seams are looked for inside the chunk only; a deck that ends on a chunk's edge
    // would need the next chunk's floors, and none seen yet does.
    for (i, &k) in on.iter().enumerate() {
        let Some(k) = k else {
            continue;
        };
        let i = i as i64;
        let (x, z) = (i % CHUNK, i / CHUNK);
        let next = [
            if x + 1 < CHUNK { Some(i + 1) } else { None },
            if z + 1 < CHUNK { Some(i + CHUNK) } else { None },
        ];
        for j in next {
            let other = j.and_then(|j| on[j as usize]);
            let (Some(j), Some(other)) = (j, other) else {
                continue;
            };
            if other == k {
                continue;
            }
            let (a, b) = (
                levels[k].heights[i as usize],
                levels[other].heights[j as usize],
            );
            let (Some(a), Some(b)) = (a, b) else {
                continue;
            };
            if (a - b).abs() <= CLIMB {
                continue;
            }
            let blocked = if (a - height).abs() > (b - height).abs() {
                i
            } else {
                j
            };
            words[blocked as usize] |= BLOCKED;
        }
    }
    Chunk {
        heights: on
            .iter()
            .enumerate()
            .map(|(i, &k)| k.and_then(|k| levels[k].heights[i]))
            .collect(),
        ..Chunk::new(chunk.origin, words)
    }
}

/// The ground's cells a floor over it is walked onto from, with the ground's height there: open
/// on the ground alone, beside a cell open on another floor alone within `CLIMB` of it. Where
/// Sootopolis Gym's two stairs climb from its lower floor to 189, beside the entrance's room at
/// 198 (spike 167).
pub fn joins(chunk: &Chunk) -> IndexMap<(i64, i64), f64> {
    let mut out: IndexMap<(i64, i64), f64> = IndexMap::new();
    let Some((ground, decks)) = chunk.floors.split_first() else {
        return out;
    };
    for i in 0..CHUNK * CHUNK {
        let Some(here) = ground.heights[i as usize] else {
            continue;
        };
        if ground.words[i as usize] & BLOCKED != 0
            || decks
                .iter()
                .any(|deck| deck.words[i as usize] & BLOCKED == 0)
        {
            continue;
        }
        let (x, z) = (i % CHUNK, i / CHUNK);
        let beside: Vec<i64> = [
            (i - 1, x > 0),
            (i + 1, x + 1 < CHUNK),
            (i - CHUNK, z > 0),
            (i + CHUNK, z + 1 < CHUNK),
        ]
        .into_iter()
        .filter(|&(_, inside)| inside)
        .map(|(j, _)| j)
        .collect();
        let near = beside.iter().any(|&j| {
            decks.iter().any(|deck| {
                ground.words[j as usize] & BLOCKED != 0
                    && deck.words[j as usize] & BLOCKED == 0
                    && deck.heights[j as usize].is_some_and(|there| (there - here).abs() <= CLIMB)
            })
        });
        if near {
            out.insert((chunk.origin.0 + x, chunk.origin.1 + z), here);
        }
    }
    out
}

/// The height of a `coll` mesh over each cell's centre, row by row as a cell table is, None
/// where the mesh is over none of it; or None where the bytes are not a mesh. Where it is over a
/// centre twice, the higher.
///
/// The mesh is a tree of boxes `depth` deep, each split into `across * up * along`, and each of
/// its leaves a run of vertices, a triangle every three, the same triangle in every leaf it
/// crosses (spike 156).
pub fn heights(mesh: &[u8]) -> Result<Option<Vec<Option<f64>>>> {
    if (mesh.len() as i64) < MESH_HEAD || py::slice(mesh, 0, 4) != COLL {
        return Ok(None);
    }
    let (depth, across, up, along) = (
        u32(mesh, 0x10)?,
        u32(mesh, 0x14)?,
        u32(mesh, 0x18)?,
        u32(mesh, 0x1C)?,
    );
    // Three u32s multiplied overflow an `i64`, where Python's int does not.
    let ways = across as i128 * up as i128 * along as i128;
    if !(1..=8).contains(&depth) || !(1..=64).contains(&ways) {
        return Ok(None);
    }
    let ways = ways as i64;
    let leaves = ways.pow((depth - 1) as u32);
    let table = MESH_HEAD + BOX * (1..depth).map(|level| ways.pow(level as u32)).sum::<i64>();
    let vertices = table + 8 * leaves;
    if vertices > mesh.len() as i64 {
        return Ok(None);
    }
    let mut under: Vec<Vec<f64>> = vec![Vec::new(); (CHUNK * CHUNK) as usize];
    let mut seen: HashSet<[u64; 9]> = HashSet::new();
    for leaf in 0..leaves {
        let (start, count) = (
            u32(mesh, table + 8 * leaf)?,
            u32(mesh, table + 8 * leaf + 4)?,
        );
        if count % 3 != 0 || vertices + VERTEX * (start + count) > mesh.len() as i64 {
            return Ok(None);
        }
        for first in (start..start + count).step_by(3) {
            let mut corners = [(0.0, 0.0, 0.0); 3];
            for (n, at) in (vertices + VERTEX * first..vertices + VERTEX * (first + 3))
                .step_by(VERTEX as usize)
                .enumerate()
            {
                corners[n] = (f32(mesh, at)?, f32(mesh, at + 4)?, f32(mesh, at + 8)?);
            }
            // A set of tuples of floats finds a triangle again by `==`, which a NaN never
            // equals and 0.0 and -0.0 do.
            let key = seen_key(&corners);
            if key.is_none_or(|key| !seen.contains(&key)) {
                if let Some(key) = key {
                    seen.insert(key);
                }
                lay(&corners, &mut under)?;
            }
        }
    }
    Ok(Some(
        under
            .iter()
            .map(|found| {
                if found.is_empty() {
                    None
                } else {
                    Some(greatest(found))
                }
            })
            .collect(),
    ))
}

/// A triangle as a set would hold it, or None where a NaN means it is never found again.
fn seen_key(corners: &[(f64, f64, f64); 3]) -> Option<[u64; 9]> {
    let mut key = [0u64; 9];
    for (n, &(x, y, z)) in corners.iter().enumerate() {
        for (m, value) in [x, y, z].into_iter().enumerate() {
            if value.is_nan() {
                return None;
            }
            key[n * 3 + m] = if value == 0.0 { 0 } else { value.to_bits() };
        }
    }
    Some(key)
}

/// Add a triangle's height at each cell centre it is over.
fn lay(corners: &[(f64, f64, f64); 3], under: &mut [Vec<f64>]) -> Result<()> {
    let [(x1, y1, z1), (x2, y2, z2), (x3, y3, z3)] = *corners;
    let det = (z2 - z3) * (x1 - x3) + (x3 - x2) * (z1 - z3);
    if det == 0.0 {
        return Ok(());
    }
    let half = (CHUNK * CELL) as f64 / 2.0;

    let span = |points: [f64; 3]| -> Result<(i64, i64)> {
        let low = py::ceil((least(&points) + half) / CELL as f64 - 0.5)?;
        let high = py::floor((greatest(&points) + half) / CELL as f64 - 0.5)?;
        Ok((low.max(0), high.min(CHUNK - 1) + 1))
    };

    let (first, last) = span([z1, z2, z3])?;
    for z in first..last {
        let cz = (z * CELL) as f64 + CELL as f64 / 2.0 - half;
        let (first, last) = span([x1, x2, x3])?;
        for x in first..last {
            let cx = (x * CELL) as f64 + CELL as f64 / 2.0 - half;
            let a = ((z2 - z3) * (cx - x3) + (x3 - x2) * (cz - z3)) / det;
            let b = ((z3 - z1) * (cx - x3) + (x1 - x3) * (cz - z3)) / det;
            if least(&[a, b, 1.0 - a - b]) >= -EDGE {
                under[(z * CHUNK + x) as usize].push(a * y1 + b * y2 + (1.0 - a - b) * y3);
            }
        }
    }
    Ok(())
}

/// Every floor of a chunk with more than its ground, the ground first, or none, as well
/// where a floor's table or mesh is not one; `section` reads one section by its number.
///
/// Python's `_floors`: renamed, as the public `floors` takes the name it would have here.
fn floors_in(offsets: &[i64], section: &dyn Fn(i64) -> Result<Vec<u8>>) -> Result<Vec<Floor>> {
    let sections = offsets.len() as i64 - 1;
    // X's counts are even and ORAS's odd, a floor adding two to either.
    let alone = if py::modulo(sections, 2) == 0 {
        X_ONE_FLOOR
    } else {
        ONE_FLOOR
    };
    let over = py::floordiv(sections - alone, 2);
    if over <= 0 {
        return Ok(Vec::new());
    }
    let mut out: Vec<Floor> = Vec::new();
    let mut pairs = vec![(CELLS, MESH)];
    for k in 0..over {
        pairs.push((alone + k, alone + over + k));
    }
    for (table, mesh) in pairs {
        let raw = section(table)?;
        let words = if raw.len() as i64 >= TABLE {
            words(&raw)?
        } else {
            None
        };
        let levels = heights(&section(mesh)?)?;
        let (Some(words), Some(levels)) = (words, levels) else {
            return Ok(Vec::new());
        };
        out.push(Floor {
            words,
            heights: levels,
        });
    }
    Ok(out)
}

/// A chunk file's section offsets, from its header: the magic, the game's count of sections,
/// offsets that climb from the header's end, and room for a cell table in section 0.
fn offsets(head: &[u8]) -> Result<Option<Vec<i64>>> {
    if (head.len() as i64) < HEADER {
        return Ok(None);
    }
    let sections = u16(head, 2)?;
    if py::slice(head, 0, 2) != MAGIC || !SECTIONS.contains(&sections) {
        return Ok(None);
    }
    let mut offsets: Vec<i64> = Vec::new();
    for i in 0..sections + 1 {
        offsets.push(u32(head, 4 + 4 * i)?);
    }
    if offsets[0] != HEADER || offsets[offsets.len() - 1] > MAX_FILE {
        return Ok(None);
    }
    if offsets.windows(2).any(|pair| pair[0] > pair[1]) {
        return Ok(None);
    }
    if offsets[CELLS as usize + 1] - offsets[CELLS as usize] < TABLE {
        return Ok(None);
    }
    Ok(Some(offsets))
}

/// A cell table's words, or None where it is not 40 by 40.
fn words(table: &[u8]) -> Result<Option<Vec<i64>>> {
    if u16(table, 0)? != CHUNK || u16(table, 2)? != CHUNK {
        return Ok(None);
    }
    let mut words: Vec<i64> = Vec::new();
    for i in 0..CHUNK * CHUNK {
        words.push(u32(table, 4 + 4 * i)?);
    }
    Ok(Some(words))
}
