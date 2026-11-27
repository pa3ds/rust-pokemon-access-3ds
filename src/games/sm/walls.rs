//! Sun's walls out of the player's ROM: which cells of an area can be walked on foot (spike 114).
//!
//! Sun has no cell table. What the player walks on and walks into are triangle meshes, one set to a
//! **map block** of the world, 4,200 world units square, and an area lists the blocks the game
//! keeps loaded while the player is in it: its own and those around it, so the tables of two areas
//! side by side share blocks.
//!
//! **Block table** — an area's third file of `a/0/8/2` ('TR'), a pack of one:
//!
//! ```text
//! +0x00 u32 width and u32 height in blocks, u32 a block's side in world units (4,200), u32 0,
//!       u32 how many alternates follow the table
//! +0x14 width x height records of 12 bytes, a row at a time from the north-west:
//!       +0x00 u16 the block's file in `a/0/8/6`, 0xffff for none; the rest is not read
//! then the alternates, 12 bytes each: u32 a record, u16 its number, u16 another file
//! ```
//!
//! Block (x, z) of the table starts at world x times 4,200 and z times 4,200. The player's bedroom
//! is block (0, 0) of area 21's table, Route 1 beside the house block (6, 4) of area 0's, and Iki
//! Town block (4, 3) of area 1's. Which alternate the game loads is not known, so the first file is
//! read.
//!
//! **Map block** — a file of `a/0/8/6`, a 'BG' pack of eight: 0 the block's model (`gfmodel`), 1 to
//! 5 empty, 6 the floor mesh, 7 a 'BG' pack of four wall meshes. Wall mesh 0 is every wall, fence
//! and cliff; 1 runs along the edge of the sea; 2 is short boxes with lids, the story's barriers,
//! one of them round a warp a person stands in the way of; 3 was empty in every block read.
//!
//! A **barrier** (mesh 2) is not a wall: it holds while the story keeps someone standing in it,
//! and then the player walks through it as through open ground. Route 1's beside the house, from
//! (584, 336) to (588, 339) with nobody in it, was walked through from north to south in Sun after
//! Iki Town's festival, and taken as a wall P said there was no way out of the cell before it
//! (spike P). So an area keeps each barrier's cells apart (`AreaWalls.barriers`), and the field
//! closes those of a barrier a person stands in (`field`).
//!
//! **Mesh** — `00 05 12 14`, u32 count, then that many triangles of 0x48 bytes: three corners (f32
//! x, y, z, 1), the normal (f32 x, y, z, 0), u32 the ground's kind (tall grass 31 and the open sea
//! 8 outdoors, 0xffffffff on a wall), and a u32 of flags that is not read.
//!
//! A cell is `field.CELL` world units, so a block is 84 cells a side. A cell is **open** where some
//! floor lies under it, at any of nine points spread over it, and no wall comes within half a cell
//! of its centre. Half a cell is the least that keeps a way from stepping through a wall: a wall
//! between two cells' centres is less than half a cell from one of them, unless it crosses exactly
//! halfway. A cell of the area's blocks with no floor under it is closed; a cell in none of them is
//! not known.
//!
//! **Heights** (spike 120). A cell can hold more than one **floor**, each a range of heights: the
//! ground and a porch's deck over it, a town's edge and the route below it. Floors under one point
//! less than `STEP` apart are one floor; a floor with another less than `HEADROOM` above it is
//! covered, and not stood on, as the ground under a deck is. A wall closes only the floors it rises
//! above and starts below `HEADROOM` over: outdoors every wall runs from y -1,600 to 1,700 and
//! closes them all, indoors 0 to 400 or 600. Two neighbouring cells' floors are walked between
//! only where they come within `STEP` of each other, so a drop with no wall along it — Iki Town's
//! edge above Route 1, 189 units — joins nothing, and a **pocket** is the floors so joined. Where
//! two cells side by side hold floors of one pocket that do not join — the way down to the route
//! runs round by the stairs — one of the two is closed in the chunks P walks: the one whose floor
//! is further from the player's.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};

use indexmap::IndexMap;

use crate::core::binary::{self, u16, u32};
use crate::core::pathfind::Box;
use crate::core::rom::Rom;
use crate::games::common::field::chunk::{BLOCKED, CHUNK, Chunk};
use crate::games::common::garc::garc_spans;
use crate::games::sm::camera::{turned, turned_box, unturned};
use crate::games::sm::zones::{AREA_FILES, EVENTS, pack, pack_with, unpacked};
use crate::py::hypot;
use crate::py::{self, Error, Kind, Result};

pub type Cell = (i64, i64);
pub type Block = (i64, i64);
/// One floor of a cell: the lowest and the highest of it, in world units.
pub type Level = (i64, i64);

pub const MAPS: &str = "a/0/8/6";
/// The area's file in `EVENTS` that is its block table, counted from its events.
pub const BLOCK_TABLE: i64 = 2;
pub const TABLE: &[u8] = b"TR";
pub const TABLE_HEAD: i64 = 0x14;
pub const TABLE_RECORD: i64 = 12;
pub const NO_FILE: i64 = 0xFFFF;
pub const PACK: &[u8] = b"BG";
pub const FLOOR: i64 = 6;
pub const WALLS: i64 = 7;
/// The wall mesh of the story's barriers, kept apart from the walls.
pub const BARRIERS: usize = 2;
/// Cells within which two of a barrier's triangles touch and are one box.
pub const TOUCH: f64 = 0.01;
pub const MESH: &[u8] = b"\x00\x05\x12\x14";
pub const TRIANGLE: i64 = 0x48;
pub const KIND: i64 = 0x40;
/// How near a wall, in cells, a cell's centre may not come and be open (the docstring says why).
pub const REACH: f64 = 0.5;
/// Cells within which a wall counts as lying exactly on the line between two cells.
pub const EXACT: f64 = 1e-6;
/// Where in a cell, each way, floor is looked for: nine points, any of which will do.
pub const SAMPLES: [f64; 3] = [0.25, 0.5, 0.75];
/// `len(SAMPLES) ** 2`.
pub const POINTS: i64 = (SAMPLES.len() * SAMPLES.len()) as i64;
/// How far apart, in world units, two floors may be and still be walked between, and two
/// floors under one point be one. Stairs are ramps of 34 to 36 degrees, whose cells' floors come
/// within 20 of their neighbours'; a drop with no wall along it is 59 (the edge of Iki Town's
/// wooden platforms) to 250 (spike 120). Whether the game lets a player up 59 is not known.
pub const STEP: i64 = 50;
/// How much room, in world units, a floor needs above it to be stood on. The ground runs on under
/// a porch's deck 41 to 76 units below it (spike 120).
pub const HEADROOM: i64 = 100;
/// World units within which a floor found again under a point is the one already found there:
/// two triangles meet along their shared side.
pub const SAME: f64 = 1.0;
/// Areas kept once loaded: the one the player is in and the last few they came from, since a
/// door is often walked back through.
pub const KEEP: i64 = 4;
/// Sets of an area's tiles kept, each for the pockets a player stood in (`AreaWalls.chunks`).
pub const TILE_SETS: i64 = 4;

/// One triangle of a mesh, in world units.
#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    pub corners: [(f64, f64, f64); 3],
    pub normal: (f64, f64, f64),
    pub kind: i64,
}

/// Which map blocks an area loads, by where each lies.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockTable {
    pub width: i64,
    pub height: i64,
    /// A block's side in world units.
    pub size: i64,
    /// Each block (x, z) that has one, its file in `MAPS`.
    pub files: IndexMap<Block, i64>,
}

/// A map block's floor mesh and its wall meshes.
pub type Meshes = (Vec<Triangle>, Vec<Vec<Triangle>>);

/// `min(a, b)`: `a`, unless `b` is less, so a NaN keeps whichever came first as Python's does.
fn least(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// `max(a, b)`, as `least` is `min`.
fn greatest(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// `min(values)`, which keeps the first of equal or unordered values.
fn least_of(values: &[f64]) -> f64 {
    values[1..]
        .iter()
        .fold(values[0], |best, &x| least(best, x))
}

/// `max(values)`, as `least_of` is `min`.
fn greatest_of(values: &[f64]) -> f64 {
    values[1..]
        .iter()
        .fold(values[0], |best, &x| greatest(best, x))
}

/// `sorted(values)` for floats: stable, and -0.0 and 0.0 are equal.
fn sorted(values: &[f64]) -> Vec<f64> {
    let mut ordered = values.to_vec();
    ordered.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ordered
}

/// The 32-bit float at word `index` of a triangle's bytes, as the float Python reads it.
fn float(raw: &[u8; 64], index: usize) -> f64 {
    f32::from_le_bytes(raw[index * 4..index * 4 + 4].try_into().expect("4 bytes")) as f64
}

/// A mesh's triangles. A RomError if it is not a mesh.
pub fn mesh(data: &[u8]) -> Result<Vec<Triangle>> {
    if py::slice(data, 0, 4) != MESH {
        return Err(Error::new(Kind::RomError, "not a collision mesh"));
    }
    let count = u32(data, 4)?;
    if 8 + count * TRIANGLE > data.len() as i64 {
        return Err(Error::new(
            Kind::RomError,
            format!("a mesh of {count} triangles runs past its end"),
        ));
    }
    let mut triangles: Vec<Triangle> = Vec::new();
    for at in (8..8 + count * TRIANGLE).step_by(TRIANGLE as usize) {
        let values = binary::take::<64>(data, at)?;
        let corners = [
            (float(&values, 0), float(&values, 1), float(&values, 2)),
            (float(&values, 4), float(&values, 5), float(&values, 6)),
            (float(&values, 8), float(&values, 9), float(&values, 10)),
        ];
        triangles.push(Triangle {
            corners,
            normal: (float(&values, 12), float(&values, 13), float(&values, 14)),
            kind: u32(data, at + KIND)?,
        });
    }
    Ok(triangles)
}

/// An area's block table, from its 'TR' pack. A RomError if it is not one.
pub fn block_table(data: &[u8]) -> Result<BlockTable> {
    let mut entries = pack_with(data, TABLE, 1)?;
    if entries.len() != 1 {
        return Err(Error::value(format!(
            "too many values to unpack (expected 1, got {})",
            entries.len()
        )));
    }
    let table = entries.remove(0);
    let (width, height, size) = (u32(&table, 0)?, u32(&table, 4)?, u32(&table, 8)?);
    // A product of two u32 and a record's size can pass an i64 where Python's int does not.
    if width == 0
        || height == 0
        || size == 0
        || TABLE_HEAD.saturating_add(width.saturating_mul(height).saturating_mul(TABLE_RECORD))
            > table.len() as i64
    {
        return Err(Error::new(
            Kind::RomError,
            format!("a block table of {width} by {height} blocks of {size} does not fit"),
        ));
    }
    let mut files: IndexMap<Block, i64> = IndexMap::new();
    for index in 0..width * height {
        let file = u16(&table, TABLE_HEAD + index * TABLE_RECORD)?;
        if file != NO_FILE {
            files.insert((index % width, index / width), file);
        }
    }
    Ok(BlockTable {
        width,
        height,
        size,
        files,
    })
}

/// A map block's floor and its wall meshes, from its 'BG' pack. A RomError if it is not one.
pub fn block_meshes(data: &[u8]) -> Result<Meshes> {
    let entries = pack_with(data, PACK, WALLS + 1)?;
    let floor = if !entries[FLOOR as usize].is_empty() {
        mesh(&entries[FLOOR as usize])?
    } else {
        Vec::new()
    };
    let mut walls: Vec<Vec<Triangle>> = Vec::new();
    for part in pack(&entries[WALLS as usize], PACK)? {
        if !part.is_empty() {
            walls.push(mesh(&part)?);
        }
    }
    Ok((floor, walls))
}

/// The heights of an area's open cells' floors, in world units (`rasterize`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Floors {
    /// Each block's cells, a row at a time from the north-west: its lowest open floor's bottom.
    pub low: IndexMap<Block, Vec<i16>>,
    /// And that floor's top.
    pub high: IndexMap<Block, Vec<i16>>,
    /// The open floors above the lowest, lowest first, of each cell that has more than one.
    pub more: IndexMap<Cell, Vec<Level>>,
}

/// `functools.lru_cache(maxsize=size)`: the entries least recently used first, the one asked for
/// last moved to the end, and the first dropped when a new one would pass `size`.
struct Lru<K, V> {
    size: usize,
    entries: Vec<(K, V)>,
}

impl<K: PartialEq, V> Lru<K, V> {
    fn new(size: i64) -> Self {
        Lru {
            size: size.max(0) as usize,
            entries: Vec::new(),
        }
    }

    fn get(&mut self, key: &K) -> Option<&mut V> {
        let at = self.entries.iter().position(|(held, _)| held == key)?;
        let entry = self.entries.remove(at);
        self.entries.push(entry);
        self.entries.last_mut().map(|(_, value)| value)
    }

    fn insert(&mut self, key: K, value: V) {
        if self.size == 0 {
            return;
        }
        if self.entries.len() >= self.size {
            self.entries.remove(0);
        }
        self.entries.push((key, value));
    }
}

/// The tiles made, by the pockets they leave open (None for all) and the quarter they are turned
/// by, the last used kept.
type TileSets = Lru<(Option<Vec<i64>>, i64), IndexMap<Block, Option<Chunk>>>;

/// `mutex.lock()`, which a Python `with lock:` is: released when the block ends in an exception,
/// so a read that panicked does not leave the lock held for good.
fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// One area's ground: which cells of its blocks the player can stand in, on which floors,
/// and which floors are joined side to side into one **pocket**, all of it walked to from any
/// of it.
pub struct AreaWalls {
    floors: Floors,
    labels: IndexMap<Block, Vec<i32>>,
    more: IndexMap<Cell, Vec<i64>>,
    open: IndexMap<Block, Vec<u8>>,
    seams: Vec<(Cell, Cell)>,
    side: i64,
    /// The tiles made, by the pockets they leave open (None for all) and the quarter they are
    /// turned by, the last used kept.
    tile_sets: Mutex<TileSets>,
    /// The cells of each of the story's barriers. Python's are `frozenset`s, only tested and
    /// gathered into another set.
    pub barriers: Vec<HashSet<Cell>>,
}

impl AreaWalls {
    /// `side` cells to a block's side; `cells` each block's cells, a byte each, a row at a
    /// time from the north-west, 1 where open; `floors` the heights of the open cells' floors,
    /// or None for one floor a cell, every one at the same height; `barriers` the cells of each
    /// of the story's barriers, which `cells` leaves open.
    pub fn new(
        side: i64,
        cells: IndexMap<Block, Vec<u8>>,
        floors: Option<Floors>,
        barriers: Vec<HashSet<Cell>>,
    ) -> AreaWalls {
        let floors = floors.unwrap_or_default();
        let (labels, more, seams) = label(&cells, &floors, side);
        AreaWalls {
            barriers,
            floors,
            labels,
            more,
            open: cells,
            seams,
            side,
            tile_sets: Mutex::new(Lru::new(TILE_SETS)),
        }
    }

    pub fn blocks(&self) -> HashSet<Block> {
        self.open.keys().copied().collect()
    }

    /// Whether the player can stand in the cell, on any of its floors, or None where no block
    /// of the area holds it.
    pub fn open(&self, cell: Cell) -> Option<bool> {
        let block = (
            py::floordiv(cell.0, self.side),
            py::floordiv(cell.1, self.side),
        );
        let cells = self.open.get(&block)?;
        let (x, z) = (cell.0 - block.0 * self.side, cell.1 - block.1 * self.side);
        Some(cells[(z * self.side + x) as usize] != 0)
    }

    /// The floors of the cell the player can stand on, lowest first: none where it is not
    /// open.
    pub fn floors(&self, cell: Cell) -> Vec<Level> {
        self.levels(cell)
            .into_iter()
            .map(|(level, _pocket)| level)
            .collect()
    }

    /// The pocket an open cell's floor is in, numbered from 1, or None for any other cell: the
    /// floor nearest `height`, or the highest.
    pub fn pocket(&self, cell: Cell, height: Option<f64>) -> Option<i64> {
        let levels = self.levels(cell);
        if levels.is_empty() {
            None
        } else {
            Some(nearest(&levels, height).1)
        }
    }

    /// The pockets a player standing in the cell can walk to: its own, or where the cell is
    /// not open — the player in bed, or nearer a wall than half a cell — those of the open cells
    /// nearest round it, up to two cells away, which may be either side of the wall. Choosing
    /// one would close the player's own pocket whenever it chose wrong, and the cell does not
    /// say which side of the wall the player is on. None at all further than that.
    ///
    /// With `height`, the player's y, only the floors within `STEP` of it: the route below a
    /// town's edge is not where a player standing on the edge is. Every floor where none of
    /// those cells has one there, as when the player is in the air.
    pub fn reachable(&self, standing: Cell, height: Option<f64>) -> HashSet<i64> {
        let wanteds: Vec<Option<f64>> = match height {
            Some(height) => vec![Some(height), None],
            None => vec![None],
        };
        for wanted in wanteds {
            for ring in 0..3_i64 {
                let mut found: HashSet<i64> = HashSet::new();
                for dx in -ring..=ring {
                    for dz in -ring..=ring {
                        if dx.abs().max(dz.abs()) != ring {
                            continue;
                        }
                        for (level, pocket) in self.levels((standing.0 + dx, standing.1 + dz)) {
                            if wanted.is_none_or(|wanted| {
                                apart(whole_range(level), (wanted, wanted)) <= STEP as f64
                            }) {
                                found.insert(pocket);
                            }
                        }
                    }
                }
                if !found.is_empty() {
                    return found;
                }
            }
        }
        HashSet::new()
    }

    /// The area's cells as ORAS's chunks, which is what `field.Ground` walks: 40 cells a
    /// side, a closed cell's blocked bit set, and a cell no block holds left open, as a cell no
    /// loaded chunk holds is. Every chunk the area has, or only those `margin` cells or less
    /// from the box round `near`: the pathfinder searches the box round every chunk it is given,
    /// and a whole area's is up to 400 cells across.
    ///
    /// With `standing`, the player's cell, an open cell they cannot walk to from there is
    /// closed as well. Floor runs on under a bed and under a house seen from outdoors, walled
    /// all round, and a thing on the bed would otherwise be a goal to walk onto that no way
    /// reaches, where it should be walked up to. With `height` as well, the player's y (their
    /// model's), only the pockets of the floors at that height are theirs (`reachable`).
    ///
    /// A chunk has one floor a cell. Where two cells side by side hold floors that do not join
    /// — a drop with no wall along it — one of the two is closed, never `standing`: the one
    /// whose floor is further from `height`, or without it the lower. ORAS's pathfinding walks
    /// round a ledge in the same way, as it does a tree.
    ///
    /// With `turn`, the chunks are the area's cells turned that many quarters (`camera`), as the
    /// pad walks them: `near` is in those cells, and `standing` in the world's.
    pub fn chunks(
        &self,
        near: Option<&[Cell]>,
        margin: i64,
        standing: Option<Cell>,
        turn: i64,
        height: Option<f64>,
    ) -> Result<Vec<Chunk>> {
        let keep: Option<HashSet<i64>> = match standing {
            None => None,
            Some(standing) => {
                let found = self.reachable(standing, height);
                if found.is_empty() { None } else { Some(found) }
            }
        };
        // A player walking along a wall stands in its pocket and the one behind it by turns, and
        // the navigator asks for the ground every poll while it guides: making Route 1's 111
        // tiles again at each turn took 0.14 s.
        let key = (
            keep.as_ref().map(|keep| {
                let mut pockets: Vec<i64> = keep.iter().copied().collect();
                pockets.sort();
                pockets
            }),
            turn,
        );
        let mut tile_sets = locked(&self.tile_sets);
        if tile_sets.get(&key).is_none() {
            tile_sets.insert(key.clone(), IndexMap::new());
        }
        let made = tile_sets.get(&key).expect("the set just made");
        let side = CHUNK;
        let spans: Vec<(i64, i64, i64, i64)> = match near {
            None => {
                let mut spans = Vec::new();
                for &(bx, bz) in self.open.keys() {
                    let ((west, north), (width, depth)) = turned_box(
                        (bx * self.side, bz * self.side),
                        (self.side, self.side),
                        turn,
                    );
                    spans.push((west, north, west + width - 1, north + depth - 1));
                }
                spans
            }
            Some(cells) => {
                if cells.is_empty() {
                    return Ok(Vec::new());
                }
                let r#box = Box::around(cells.iter().copied(), margin)?;
                vec![(r#box.west, r#box.north, r#box.east, r#box.south)]
            }
        };
        let mut found: HashSet<(i64, i64)> = HashSet::new();
        for &(west, north, east, south) in &spans {
            for tz in py::floordiv(north, side)..py::floordiv(south, side) + 1 {
                for tx in py::floordiv(west, side)..py::floordiv(east, side) + 1 {
                    found.insert((tx, tz));
                }
            }
        }
        let mut tiles: Vec<(i64, i64)> = found.into_iter().collect();
        tiles.sort_by_key(|tile| (tile.1, tile.0));
        // Which cell of a seam is closed changes with the player's height, which changes on
        // every step up a stair, while a whole area holds a few hundred seams at most: they are
        // laid over the tiles kept rather than kept in them.
        let mut cut: IndexMap<Block, Vec<i64>> = IndexMap::new();
        for cell in self.cut(keep.as_ref(), standing, height) {
            let (x, z) = turned(cell, turn);
            cut.entry((py::floordiv(x, side), py::floordiv(z, side)))
                .or_default()
                .push(py::modulo(z, side) * side + py::modulo(x, side));
        }
        let mut chunks: Vec<Chunk> = Vec::new();
        for tile in tiles {
            let Some(mut chunk) = self.tile(tile, keep.as_ref(), made, turn) else {
                continue;
            };
            if let Some(ats) = cut.get(&tile) {
                let mut words = chunk.words.clone();
                for &at in ats {
                    words[at as usize] |= BLOCKED;
                }
                chunk = Chunk::new(chunk.origin, words);
            }
            chunks.push(chunk);
        }
        Ok(chunks)
    }

    /// The cells closed where two side by side hold floors of the pockets kept that do not
    /// join (`chunks` says which).
    fn cut(
        &self,
        keep: Option<&HashSet<i64>>,
        standing: Option<Cell>,
        height: Option<f64>,
    ) -> HashSet<Cell> {
        let mut closed: HashSet<Cell> = HashSet::new();
        for &(a, b) in &self.seams {
            let kept = |found: &(Level, i64)| keep.is_none_or(|keep| keep.contains(&found.1));
            let ours: Vec<(Level, i64)> = self.levels(a).into_iter().filter(kept).collect();
            let theirs: Vec<(Level, i64)> = self.levels(b).into_iter().filter(kept).collect();
            if ours.is_empty() || theirs.is_empty() {
                continue;
            }
            let (first, second) = (nearest(&ours, height).0, nearest(&theirs, height).0);
            if apart(whole_range(first), whole_range(second)) <= STEP as f64 {
                continue;
            }
            let mut shut = match height {
                None => {
                    if first < second {
                        a
                    } else {
                        b
                    }
                }
                Some(height) => {
                    if apart(whole_range(first), (height, height))
                        > apart(whole_range(second), (height, height))
                    {
                        a
                    } else {
                        b
                    }
                }
            };
            if Some(shut) == standing {
                shut = if shut == a { b } else { a };
            }
            closed.insert(shut);
        }
        closed
    }

    fn tile(
        &self,
        tile: Block,
        keep: Option<&HashSet<i64>>,
        made: &mut IndexMap<Block, Option<Chunk>>,
        turn: i64,
    ) -> Option<Chunk> {
        if !made.contains_key(&tile) {
            let origin = (tile.0 * CHUNK, tile.1 * CHUNK);
            let mut words: Vec<i64> = Vec::new();
            let mut known = false;
            for z in origin.1..origin.1 + CHUNK {
                for x in origin.0..origin.0 + CHUNK {
                    let world = unturned((x, z), turn);
                    let mut walkable = self.open(world);
                    known = known || walkable.is_some();
                    if walkable == Some(true)
                        && let Some(keep) = keep
                    {
                        walkable = Some(self.kept(world, keep));
                    }
                    words.push(if walkable == Some(false) { BLOCKED } else { 0 });
                }
            }
            made.insert(
                tile,
                if known {
                    Some(Chunk::new(origin, words))
                } else {
                    None
                },
            );
        }
        made[&tile].clone()
    }

    /// Whether an open cell has a floor in one of the pockets kept.
    fn kept(&self, cell: Cell, keep: &HashSet<i64>) -> bool {
        let side = self.side;
        let (bx, bz) = (py::floordiv(cell.0, side), py::floordiv(cell.1, side));
        if keep.contains(
            &(self.labels[&(bx, bz)][((cell.1 - bz * side) * side + cell.0 - bx * side) as usize]
                as i64),
        ) {
            return true;
        }
        self.more
            .get(&cell)
            .is_some_and(|pockets| pockets.iter().any(|pocket| keep.contains(pocket)))
    }

    /// The open floors of the cell, lowest first, each with its pocket.
    fn levels(&self, cell: Cell) -> Vec<(Level, i64)> {
        let side = self.side;
        let block = (py::floordiv(cell.0, side), py::floordiv(cell.1, side));
        let Some(opened) = self.open.get(&block) else {
            return Vec::new();
        };
        let at = ((cell.1 - block.1 * side) * side + cell.0 - block.0 * side) as usize;
        if opened[at] == 0 {
            return Vec::new();
        }
        let first = match self.floors.low.get(&block) {
            Some(low) => (low[at] as i64, self.floors.high[&block][at] as i64),
            None => (0, 0),
        };
        let mut found = vec![(first, self.labels[&block][at] as i64)];
        if let Some(more) = self.floors.more.get(&cell)
            && !more.is_empty()
        {
            let pockets = &self.more[&cell];
            assert_eq!(
                more.len(),
                pockets.len(),
                "zip() argument 2 is not as long as argument 1"
            );
            found.extend(more.iter().copied().zip(pockets.iter().copied()));
        }
        found
    }
}

/// An area's walls from its block table and each block's file (`MAPS`), a block whose file
/// is empty left unknown. A RomError where a block is not one. With `heights` False, a cell's
/// floors are one and every wall near it closes it, as spike 114 read them, which is what
/// spike 120 measures heights against.
pub fn rasterize(
    table: &BlockTable,
    blocks: &IndexMap<Block, Vec<u8>>,
    cell: f64,
    heights: bool,
) -> Result<AreaWalls> {
    if cell == 0.0 {
        return Err(Error::new(Kind::ZeroDivisionError, "division by zero"));
    }
    let side = table.size as f64 / cell;
    if side != py::int(side)? as f64 {
        return Err(Error::new(
            Kind::RomError,
            format!(
                "a block of {} is not a whole number of cells of {}",
                table.size,
                py::float_repr(cell)
            ),
        ));
    }
    let mut raster = Raster::new(py::int(side)?, heights);
    let mut meshes: IndexMap<Block, Meshes> = IndexMap::new();
    let mut barriers: Vec<HashSet<Cell>> = Vec::new();
    for (&block, data) in blocks {
        let data = unpacked(data)?;
        if !data.is_empty() {
            meshes.insert(block, block_meshes(&data)?);
            raster.add(block);
        }
    }
    for (floor, walls) in meshes.values() {
        for triangle in floor {
            let [a, b, c] = triangle.corners;
            raster.floor([
                (a.0 / cell, a.1, a.2 / cell),
                (b.0 / cell, b.1, b.2 / cell),
                (c.0 / cell, c.1, c.2 / cell),
            ])?;
        }
        for (index, wall) in walls.iter().enumerate() {
            if index == BARRIERS {
                barriers.extend(boxes(wall, cell)?);
                continue;
            }
            for triangle in wall {
                let ys = [
                    triangle.corners[0].1,
                    triangle.corners[1].1,
                    triangle.corners[2].1,
                ];
                raster.wall(
                    flat(triangle, cell),
                    triangle.normal,
                    (least_of(&ys), greatest_of(&ys)),
                )?;
            }
        }
    }
    let (cells, floors) = raster.cells()?;
    Ok(AreaWalls::new(raster.side, cells, Some(floors), barriers))
}

/// The cells of each box a mesh's triangles make: triangles whose bounds touch are one box,
/// and a cell is in it where its centre lies within the box's bounds.
fn boxes(triangles: &[Triangle], cell: f64) -> Result<Vec<HashSet<Cell>>> {
    let mut bounds: Vec<[f64; 4]> = Vec::new();
    for triangle in triangles {
        let xs = triangle.corners.map(|(x, _y, _z)| x / cell);
        let zs = triangle.corners.map(|(_x, _y, z)| z / cell);
        let mut new = [
            least_of(&xs),
            greatest_of(&xs),
            least_of(&zs),
            greatest_of(&zs),
        ];
        let touching: Vec<usize> = bounds
            .iter()
            .enumerate()
            .filter(|(_at, r#box)| {
                new[0] <= r#box[1] + TOUCH
                    && new[1] >= r#box[0] - TOUCH
                    && new[2] <= r#box[3] + TOUCH
                    && new[3] >= r#box[2] - TOUCH
            })
            .map(|(at, _box)| at)
            .collect();
        for &at in &touching {
            let r#box = bounds[at];
            new = [
                least(new[0], r#box[0]),
                greatest(new[1], r#box[1]),
                least(new[2], r#box[2]),
                greatest(new[3], r#box[3]),
            ];
        }
        // Removed from the last, so the places of those before it hold.
        for &at in touching.iter().rev() {
            bounds.remove(at);
        }
        bounds.push(new);
    }
    let mut out: Vec<HashSet<Cell>> = Vec::new();
    for [west, east, north, south] in bounds {
        let mut cells: HashSet<Cell> = HashSet::new();
        for x in py::floor(west)?..py::ceil(east)? {
            for z in py::floor(north)?..py::ceil(south)? {
                if west <= x as f64 + 0.5
                    && x as f64 + 0.5 <= east
                    && north <= z as f64 + 0.5
                    && z as f64 + 0.5 <= south
                {
                    cells.insert((x, z));
                }
            }
        }
        out.push(cells);
    }
    Ok(out)
}

/// Files of one archive of the ROM, by their numbers, in the order asked.
///
/// It is read on the thread of `SunWalls.prefetch` as well as the caller's, so it is shared as
/// the thread needs it to be.
pub type Read = Arc<dyn Fn(&str, &[i64]) -> Result<Vec<Vec<u8>>> + Send + Sync>;

/// Every area's walls, each read from the ROM the first time it is asked for.
pub struct SunWalls {
    shared: Arc<Shared>,
}

/// What `SunWalls` and the thread of its `prefetch` hold between them.
struct Shared {
    read: Read,
    cell: f64,
    /// The last `keep` areas asked for, read once each.
    ///
    /// Held while an area is looked up or read: `prefetch` reads on a thread of its own, and a
    /// P pressed meanwhile waits for it rather than reading the area a second time.
    area: Mutex<Lru<i64, Option<Arc<AreaWalls>>>>,
    /// Why an area has no walls, for each that failed to read.
    errors: Mutex<IndexMap<i64, String>>,
}

impl SunWalls {
    /// `SunWalls(read, cell)`: `keep` areas kept is `KEEP`; `with_keep` says another.
    pub fn new(read: Read, cell: f64) -> SunWalls {
        SunWalls::with_keep(read, cell, KEEP)
    }

    pub fn with_keep(read: Read, cell: f64, keep: i64) -> SunWalls {
        SunWalls {
            shared: Arc::new(Shared {
                read,
                cell,
                area: Mutex::new(Lru::new(keep)),
                errors: Mutex::new(IndexMap::new()),
            }),
        }
    }

    /// Why an area has no walls, for each that failed to read. A copy: the thread of `prefetch`
    /// may add to it meanwhile.
    pub fn errors(&self) -> IndexMap<i64, String> {
        locked(&self.shared.errors).clone()
    }

    /// Walls read from this ROM, opened again by its path each time an area is first asked
    /// for. The two archives' file tables are read now: a RomError where either is not a
    /// GARC.
    pub fn from_rom(image: &Rom, cell: f64) -> Result<SunWalls> {
        let path = image.path.clone();
        let mut tables: IndexMap<String, (i64, Vec<(i64, i64)>)> = IndexMap::new();
        for archive in [EVENTS, MAPS] {
            tables.insert(archive.to_string(), garc_spans(image, archive)?);
        }

        let read = move |archive: &str, files: &[i64]| -> Result<Vec<Vec<u8>>> {
            let Some((data, spans)) = tables.get(archive) else {
                return Err(Error::key(py::repr(archive)));
            };
            if files
                .iter()
                .any(|&file| !(0 <= file && file < spans.len() as i64))
            {
                return Err(Error::new(
                    Kind::RomError,
                    format!(
                        "{archive} has {} files, not {}",
                        spans.len(),
                        files.iter().max().expect("a file out of range") + 1
                    ),
                ));
            }
            let again = Rom::new(&path)?;
            files
                .iter()
                .map(|&file| {
                    let (start, end) = spans[file as usize];
                    again.read_part(archive, data + start, end - start)
                })
                .collect()
        };

        Ok(SunWalls::new(Arc::new(read), cell))
    }

    /// Read the area's walls on a thread of its own, so the first P in it does not wait for
    /// them: Route 1's took 0.8 s, and its first P 2 s (sun_4.ctm, 2026-09-26).
    pub fn prefetch(&self, area: i64) {
        let shared = Arc::clone(&self.shared);
        // A daemon thread: its handle is dropped, not joined. Python prints a traceback to
        // stderr where the thread's call raises; nothing reads this thread's result here.
        std::thread::spawn(move || {
            let _ = shared.area(area);
        });
    }

    /// The area's walls, or None where they cannot be read (`errors` says why).
    pub fn area(&self, area: i64) -> Result<Option<Arc<AreaWalls>>> {
        self.shared.area(area)
    }
}

impl Shared {
    fn area(&self, area: i64) -> Result<Option<Arc<AreaWalls>>> {
        let mut kept = locked(&self.area);
        if let Some(walls) = kept.get(&area) {
            return Ok(walls.clone());
        }
        let walls = self.load_or_none(area)?;
        kept.insert(area, walls.clone());
        Ok(walls)
    }

    fn load_or_none(&self, area: i64) -> Result<Option<Arc<AreaWalls>>> {
        match self.load(area) {
            Ok(walls) => Ok(Some(Arc::new(walls))),
            Err(error)
                if error.is(Kind::RomError)
                    || error.is(Kind::OsError)
                    || error.is(Kind::StructError)
                    || error.is(Kind::IndexError)
                    || error.is(Kind::ValueError) =>
            {
                locked(&self.errors).insert(area, error.to_string());
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    fn load(&self, area: i64) -> Result<AreaWalls> {
        let mut read = (self.read)(EVENTS, &[area * AREA_FILES + BLOCK_TABLE])?;
        if read.len() != 1 {
            return Err(Error::value(if read.is_empty() {
                "not enough values to unpack (expected 1, got 0)".to_string()
            } else {
                format!("too many values to unpack (expected 1, got {})", read.len())
            }));
        }
        let packed = read.remove(0);
        let table = block_table(&unpacked(&packed)?)?;
        let mut files: Vec<i64> = table.files.values().copied().collect();
        files.sort();
        files.dedup();
        let read = (self.read)(MAPS, &files)?;
        if read.len() != files.len() {
            return Err(Error::value(format!(
                "zip() argument 2 is {} than argument 1",
                if read.len() > files.len() {
                    "longer"
                } else {
                    "shorter"
                }
            )));
        }
        let data: IndexMap<i64, Vec<u8>> = files.into_iter().zip(read).collect();
        let blocks: IndexMap<Block, Vec<u8>> = table
            .files
            .iter()
            .map(|(&block, file)| (block, data[file].clone()))
            .collect();
        rasterize(&table, &blocks, self.cell, true)
    }
}

/// An area's cells as they are marked, a block at a time, in cells: the floors under each of
/// a cell's nine points, and how high the walls near its centre reach.
struct Raster {
    side: i64,
    heights: bool,
    /// Each block's cells' nine points, a cell after another: the first floor's height found
    /// under each, NaN where none is. An array of 32-bit floats in the Python, and a height is
    /// rounded to one as it is stored.
    points: IndexMap<Block, Vec<f32>>,
    /// The other floors found under a point, by its block and its place in `points`.
    more: IndexMap<(Block, i64), Vec<f64>>,
    /// Each block's cells a wall closes, by their place in the block: the lowest and highest
    /// of those walls, overlapping ones as one.
    walls: IndexMap<Block, IndexMap<i64, Vec<(f64, f64)>>>,
}

impl Raster {
    fn new(side: i64, heights: bool) -> Raster {
        Raster {
            side,
            heights,
            points: IndexMap::new(),
            more: IndexMap::new(),
            walls: IndexMap::new(),
        }
    }

    fn add(&mut self, block: Block) {
        self.points.insert(
            block,
            vec![f32::NAN; (self.side * self.side * POINTS) as usize],
        );
        self.walls.insert(block, IndexMap::new());
    }

    /// The triangle's height under each of the points it covers: x and z in cells, y in
    /// world units.
    fn floor(&mut self, corners: [(f64, f64, f64); 3]) -> Result<()> {
        let [(ax, ay, az), (bx, by, bz), (cx, cy, cz)] = corners;
        let area = (bx - ax) * (cz - az) - (bz - az) * (cx - ax);
        if area == 0.0 {
            return Ok(());
        }
        // Its plane: y = ay + east * (x - ax) + south * (z - az).
        let east = ((by - ay) * (cz - az) - (bz - az) * (cy - ay)) / area;
        let south = ((bx - ax) * (cy - ay) - (by - ay) * (cx - ax)) / area;
        let flat = [(ax, az), (bx, bz), (cx, cz)];
        let (low, high) = (least_of(&[az, bz, cz]), greatest_of(&[az, bz, cz]));
        let (side, count) = (self.side, SAMPLES.len() as i64);
        for (down, &dz) in SAMPLES.iter().enumerate() {
            let down = down as i64;
            let (first, stop) = (py::ceil(low - dz)?, py::floor(high - dz)? + 1);
            for row in first..stop {
                let Some(across) = across(&flat, row as f64 + dz) else {
                    continue;
                };
                let (block_z, z) = (py::floordiv(row, side), py::modulo(row, side));
                let level = ay + south * ((row as f64 + dz) - az);
                for (along, &dx) in SAMPLES.iter().enumerate() {
                    let along = along as i64;
                    let (mut column, last) = (py::ceil(across.0 - dx)?, py::floor(across.1 - dx)?);
                    // A block's stretch of the run at a time, the point stepping a cell east.
                    while column <= last {
                        let (block_x, x) = (py::floordiv(column, side), py::modulo(column, side));
                        let end = last.min((block_x + 1) * side - 1);
                        if let Some(points) = self.points.get_mut(&(block_x, block_z)) {
                            let mut at = (z * side + x) * POINTS + down * count + along;
                            let mut y = level + east * ((column as f64 + dx) - ax);
                            for _ in 0..end - column + 1 {
                                let found = points[at as usize] as f64;
                                if found.is_nan() {
                                    points[at as usize] = y as f32;
                                } else if (found - y).abs() > SAME {
                                    self.more
                                        .entry(((block_x, block_z), at))
                                        .or_default()
                                        .push(y);
                                }
                                at += POINTS;
                                y += east;
                            }
                        }
                        column = end + 1;
                    }
                }
            }
        }
        Ok(())
    }

    /// A wall from `span`'s low to its high on every cell whose centre is nearer the
    /// triangle, seen from above, than `REACH`.
    fn wall(
        &mut self,
        corners: [(f64, f64); 3],
        normal: (f64, f64, f64),
        span: (f64, f64),
    ) -> Result<()> {
        let xs = [corners[0].0, corners[1].0, corners[2].0];
        let zs = [corners[0].1, corners[1].1, corners[2].1];
        for row in py::floor(least_of(&zs) - REACH)?..py::floor(greatest_of(&zs) + REACH)? + 1 {
            for column in
                py::floor(least_of(&xs) - REACH)?..py::floor(greatest_of(&xs) + REACH)? + 1
            {
                let centre = (column as f64 + 0.5, row as f64 + 0.5);
                let distance = distance(centre, &corners);
                // A wall on the line between two cells is exactly `REACH` from both centres, as
                // the bed's side in the bedroom is, and must still close one of them or a way
                // steps through it: the one behind it, away from the way it faces.
                let behind = ((centre.0 - corners[0].0) * normal.0
                    + (centre.1 - corners[0].1) * normal.2)
                    <= 0.0;
                if distance < REACH || (behind && distance < REACH + EXACT) {
                    self.wall_(row, column, span);
                }
            }
        }
        Ok(())
    }

    /// Each block's cells, 1 where some floor of it is open, and the open floors.
    fn cells(&self) -> Result<(IndexMap<Block, Vec<u8>>, Floors)> {
        let side = self.side;
        let mut crowded: IndexMap<Block, HashSet<i64>> = IndexMap::new();
        for &(block, at) in self.more.keys() {
            crowded.entry(block).or_default().insert(at / POINTS);
        }
        let mut cells: IndexMap<Block, Vec<u8>> = IndexMap::new();
        let mut low: IndexMap<Block, Vec<i16>> = IndexMap::new();
        let mut high: IndexMap<Block, Vec<i16>> = IndexMap::new();
        let mut more: IndexMap<Cell, Vec<Level>> = IndexMap::new();
        let nothing: HashSet<i64> = HashSet::new();
        for (&block, points) in &self.points {
            let mut opened = vec![0u8; (side * side) as usize];
            let mut lows = vec![0i16; (side * side) as usize];
            let mut highs = vec![0i16; (side * side) as usize];
            let walls = &self.walls[&block];
            let full = crowded.get(&block).unwrap_or(&nothing);
            for index in 0..side * side {
                let start = index * POINTS;
                let ys: Vec<f64> = if full.contains(&index) {
                    self.uncovered(block, points, start)
                } else {
                    points[start as usize..(start + POINTS) as usize]
                        .iter()
                        .filter(|y| !y.is_nan())
                        .map(|&y| y as f64)
                        .collect()
                };
                if ys.is_empty() {
                    continue;
                }
                let (bottom, top) = (least_of(&ys), greatest_of(&ys));
                let mut levels: Vec<(f64, f64)> = if !self.heights || top - bottom <= STEP as f64 {
                    vec![(bottom, top)]
                } else {
                    bands(&ys)
                };
                if let Some(spans) = walls.get(&index)
                    && !spans.is_empty()
                {
                    levels.retain(|&level| {
                        self.heights && !spans.iter().any(|&span| closes(span, level))
                    });
                }
                if levels.is_empty() {
                    continue;
                }
                opened[index as usize] = 1;
                let (bottom, top) = whole(levels[0])?;
                lows[index as usize] = bottom as i16;
                highs[index as usize] = top as i16;
                if levels.len() > 1 {
                    let cell = (block.0 * side + index % side, block.1 * side + index / side);
                    let mut rest: Vec<Level> = Vec::new();
                    for &level in &levels[1..] {
                        rest.push(whole(level)?);
                    }
                    more.insert(cell, rest);
                }
            }
            cells.insert(block, opened);
            low.insert(block, lows);
            high.insert(block, highs);
        }
        Ok((cells, Floors { low, high, more }))
    }

    /// The heights of the floors under a cell's points that are not covered by another less
    /// than `HEADROOM` above: the ground under a porch's deck is not stood on.
    fn uncovered(&self, block: Block, points: &[f32], start: i64) -> Vec<f64> {
        let mut ys: Vec<f64> = Vec::new();
        for at in start..start + POINTS {
            if points[at as usize].is_nan() {
                continue;
            }
            let mut found = vec![points[at as usize] as f64];
            if let Some(more) = self.more.get(&(block, at)) {
                found.extend(more);
            }
            let found = sorted(&found);
            if !self.heights {
                ys.extend(found);
                continue;
            }
            let floors = bands(&found);
            for pair in floors.windows(2) {
                let (below, above) = (pair[0], pair[1]);
                if above.0 - below.1 >= HEADROOM as f64 {
                    ys.push(below.0);
                    ys.push(below.1);
                }
            }
            let last = floors[floors.len() - 1];
            ys.push(last.0);
            ys.push(last.1);
        }
        ys
    }

    /// `_wall`: a wall's span marked on one cell. Named for what it marks, since `wall` is the
    /// triangle's.
    fn wall_(&mut self, row: i64, column: i64, span: (f64, f64)) {
        let side = self.side;
        let (block_x, x) = (py::floordiv(column, side), py::modulo(column, side));
        let (block_z, z) = (py::floordiv(row, side), py::modulo(row, side));
        let Some(walls) = self.walls.get_mut(&(block_x, block_z)) else {
            return;
        };
        let spans = walls.entry(z * side + x).or_default();
        let (mut low, mut high) = span;
        // Two walls that overlap close a floor where either would, so they are kept as one:
        // outdoors every wall near a cell runs from y -1,600 to 1,700.
        let overlapping: Vec<(f64, f64)> = spans
            .iter()
            .copied()
            .filter(|other| other.0 <= high && low <= other.1)
            .collect();
        for other in overlapping {
            if let Some(at) = spans.iter().position(|held| *held == other) {
                spans.remove(at);
            }
            low = least(low, other.0);
            high = greatest(high, other.1);
        }
        spans.push((low, high));
    }
}

/// Heights as floors: those less than `STEP` apart one floor, lowest first.
fn bands(ys: &[f64]) -> Vec<(f64, f64)> {
    let ordered = sorted(ys);
    let mut bands = vec![(ordered[0], ordered[0])];
    for &y in &ordered[1..] {
        let last = bands.len() - 1;
        if y - bands[last].1 <= STEP as f64 {
            bands[last] = (bands[last].0, y);
        } else {
            bands.push((y, y));
        }
    }
    bands
}

/// Whether a wall from `span`'s low to its high stands in the way of a player on the floor:
/// it rises above the floor, and starts below `HEADROOM` over it.
fn closes(span: (f64, f64), level: (f64, f64)) -> bool {
    span.1 > level.0 && span.0 < level.1 + HEADROOM as f64
}

/// A floor in whole world units, within what an array of shorts holds.
fn whole(level: (f64, f64)) -> Result<Level> {
    let limit = 0x7FFF;
    Ok((
        (-limit).max(limit.min(py::floor(level.0)?)),
        (-limit).max(limit.min(py::ceil(level.1)?)),
    ))
}

/// A floor's two heights as the floats `apart` compares.
fn whole_range(level: Level) -> (f64, f64) {
    (level.0 as f64, level.1 as f64)
}

/// How far apart two ranges of heights are, 0 where they overlap.
fn apart(a: (f64, f64), b: (f64, f64)) -> f64 {
    greatest(greatest(0.0, a.0 - b.1), b.0 - a.1)
}

/// Of floors each with something, the one nearest `height`, or the highest.
fn nearest<T: Copy>(levels: &[(Level, T)], height: Option<f64>) -> (Level, T) {
    let Some(height) = height else {
        return levels[levels.len() - 1];
    };
    let key = |found: &(Level, T)| apart(whole_range(found.0), (height, height));
    let mut best = levels[0];
    for found in &levels[1..] {
        if key(found) < key(&best) {
            best = *found;
        }
    }
    best
}

/// Each open floor numbered by the pocket it is in, from 1: a pocket is every open floor
/// joined to another of a cell beside it, across blocks too, where the two come within `STEP`.
/// The first floor of each block's cells is numbered in an array a block, 0 for a cell that is
/// not open; the rest by cell. And the **seams**: every two cells side by side, west or north
/// first, some floor of which does not join some floor of the other.
fn label(
    cells: &IndexMap<Block, Vec<u8>>,
    floors: &Floors,
    side: i64,
) -> (
    IndexMap<Block, Vec<i32>>,
    IndexMap<Cell, Vec<i64>>,
    Vec<(Cell, Cell)>,
) {
    let mut labels: IndexMap<Block, Vec<i32>> = cells
        .keys()
        .map(|&block| (block, vec![0; (side * side) as usize]))
        .collect();
    let (extra, lows, highs) = (&floors.more, &floors.low, &floors.high);
    let mut more: IndexMap<Cell, Vec<i64>> = extra
        .iter()
        .map(|(&cell, levels)| (cell, vec![0; levels.len()]))
        .collect();
    let mut seams: HashSet<(Cell, Cell)> = HashSet::new();
    let mut blocks: Vec<(&Block, &Vec<u8>)> = cells.iter().collect();
    blocks.sort_by_key(|(block, _)| **block);
    let mut seeds: Vec<(i64, i64, i64)> = Vec::new();
    for (&(bx, bz), block) in blocks {
        for (index, &value) in block.iter().enumerate() {
            if value != 0 {
                let index = index as i64;
                seeds.push((bx * side + index % side, bz * side + index / side, 0));
            }
        }
    }
    let mut extras: Vec<(&Cell, &Vec<Level>)> = extra.iter().collect();
    extras.sort_by_key(|(cell, _)| **cell);
    for (&(x, z), found) in extras {
        for floor in 1..=found.len() as i64 {
            seeds.push((x, z, floor));
        }
    }
    let mut pocket: i64 = 0;
    // Route 1 has 115,000 open cells and 600 with a second floor, so a neighbour's first floor is
    // read straight from its block's arrays rather than through a call: with the raster's loops
    // done likewise, reading Route 1 went from 2.4 s to 1.3 s (spike 120).
    for (x, z, floor) in seeds {
        if floor != 0 {
            let numbers = more.get_mut(&(x, z)).expect("a cell with more floors");
            if numbers[(floor - 1) as usize] != 0 {
                continue;
            }
            numbers[(floor - 1) as usize] = pocket + 1;
        } else {
            let (bx, bz) = (py::floordiv(x, side), py::floordiv(z, side));
            let mine = labels.get_mut(&(bx, bz)).expect("a block of the area");
            let at = ((z - bz * side) * side + x - bx * side) as usize;
            if mine[at] != 0 {
                continue;
            }
            mine[at] = (pocket + 1) as i32;
        }
        pocket += 1;
        let mut queue: Vec<(i64, i64, i64)> = vec![(x, z, floor)];
        while let Some((cx, cz, cf)) = queue.pop() {
            let (bottom, top) = if cf != 0 {
                extra[&(cx, cz)][(cf - 1) as usize]
            } else {
                let (bx, bz) = (py::floordiv(cx, side), py::floordiv(cz, side));
                let at = ((cz - bz * side) * side + cx - bx * side) as usize;
                match lows.get(&(bx, bz)) {
                    Some(found) => (found[at] as i64, highs[&(bx, bz)][at] as i64),
                    None => (0, 0),
                }
            };
            for (sx, sz) in [(cx + 1, cz), (cx - 1, cz), (cx, cz + 1), (cx, cz - 1)] {
                let (bx, bz) = (py::floordiv(sx, side), py::floordiv(sz, side));
                let block = cells.get(&(bx, bz));
                let at = ((sz - bz * side) * side + sx - bx * side) as usize;
                let Some(block) = block else {
                    continue;
                };
                if block[at] == 0 {
                    continue;
                }
                let there = match lows.get(&(bx, bz)) {
                    Some(found) => (found[at] as i64, highs[&(bx, bz)][at] as i64),
                    None => (0, 0),
                };
                let others: &[Level] = extra.get(&(sx, sz)).map_or(&[], Vec::as_slice);
                for (index, &(low, high)) in std::iter::once(&there).chain(others).enumerate() {
                    if low - top > STEP || bottom - high > STEP {
                        seams.insert(if (cz, cx) < (sz, sx) {
                            ((cx, cz), (sx, sz))
                        } else {
                            ((sx, sz), (cx, cz))
                        });
                    } else if index != 0 {
                        let numbers = more.get_mut(&(sx, sz)).expect("a cell with more floors");
                        if numbers[index - 1] == 0 {
                            numbers[index - 1] = pocket;
                            queue.push((sx, sz, index as i64));
                        }
                    } else if labels[&(bx, bz)][at] == 0 {
                        labels.get_mut(&(bx, bz)).expect("a block of the area")[at] = pocket as i32;
                        queue.push((sx, sz, 0));
                    }
                }
            }
        }
    }
    let mut seams: Vec<(Cell, Cell)> = seams.into_iter().collect();
    seams.sort();
    (labels, more, seams)
}

/// The triangle seen from above, in cells.
fn flat(triangle: &Triangle, cell: f64) -> [(f64, f64); 3] {
    let [a, b, c] = triangle.corners;
    [
        (a.0 / cell, a.2 / cell),
        (b.0 / cell, b.2 / cell),
        (c.0 / cell, c.2 / cell),
    ]
}

/// Where the line at `z` crosses the triangle, west end and east end, or None.
fn across(corners: &[(f64, f64); 3], z: f64) -> Option<(f64, f64)> {
    let mut xs: Vec<f64> = Vec::new();
    for ((ax, az), (bx, bz)) in [
        (corners[0], corners[1]),
        (corners[1], corners[2]),
        (corners[2], corners[0]),
    ] {
        if az == bz {
            if az == z {
                xs.push(ax);
                xs.push(bx);
            }
        } else if least(az, bz) <= z && z <= greatest(az, bz) {
            xs.push(ax + (z - az) * (bx - ax) / (bz - az));
        }
    }
    if xs.is_empty() {
        None
    } else {
        Some((least_of(&xs), greatest_of(&xs)))
    }
}

/// How far the point is from the triangle, zero inside it.
fn distance(point: (f64, f64), corners: &[(f64, f64); 3]) -> f64 {
    let [a, b, c] = *corners;
    let area = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
    // An upright wall is a triangle with no area from above: "inside" it would then be anywhere
    // on the line it stands on, however far along, so only its edges are measured.
    if area.abs() > 1e-9 {
        let sides = [(a, b), (b, c), (c, a)]
            .map(|(p, q)| (q.0 - p.0) * (point.1 - p.1) - (q.1 - p.1) * (point.0 - p.0));
        if sides.iter().all(|&side| side >= 0.0) || sides.iter().all(|&side| side <= 0.0) {
            return 0.0;
        }
    }
    least(
        least(to_segment(point, a, b), to_segment(point, b, c)),
        to_segment(point, c, a),
    )
}

fn to_segment(point: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dz) = (b.0 - a.0, b.1 - a.1);
    let length = dx * dx + dz * dz;
    let t = if length == 0.0 {
        0.0
    } else {
        greatest(
            0.0,
            least(1.0, ((point.0 - a.0) * dx + (point.1 - a.1) * dz) / length),
        )
    };
    hypot(point.0 - a.0 - t * dx, point.1 - a.1 - t * dz)
}
