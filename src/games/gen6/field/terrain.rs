//! The matrix the player is on, as the player's ROM has it: every chunk's cell table, floors and
//! ground heights by the file the field map lists for it, and what is read off them: where the
//! ground drops, which way a bumpy slope or a waterfall rises, whether it has a deck, and which
//! deep water the sea above joins (spikes 136, 156, 159, 160, 165, 170). Nothing here reads RAM:
//! the field reader says which files the matrix lists (`files`), and asks.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::pathfind::{self, Cell};
use crate::games::common::field::chunk::{BLOCKED, CHUNK, Chunk, DIVE, Floor, KIND_SHIFT, SHORE};
use crate::games::common::field::climb::Walls;
use crate::games::common::field::grind::GrindRails;
use crate::games::gen6::field::landform::{Climbing, Grinding};
use crate::games::gen6::field::{mapblock, rail};
use crate::py::{self, Error, Result};

/// What the list has where the matrix has no chunk: most of Route 109's 12 by 10 (spike 136).
pub const NO_CHUNK_FILE: i64 = 0xFFFF;

/// How many chunks flattened at a height are kept (`FieldReader._flattened`).
pub const FLATTENED: i64 = 64;

/// A chunk file's cell table out of the ROM, by its number (`archives.MapChunks`); None where
/// there is no ROM to read, and a chunk the game has not loaded is not known.
pub type FarCells = Rc<dyn Fn(i64) -> Result<Option<Vec<i64>>>>;

/// A chunk file's floors out of the ROM, by its number, where it has a deck over its ground
/// (`archives.MapChunks.floors`).
pub type FarFloors = Rc<dyn Fn(i64) -> Result<Vec<Floor>>>;

/// A chunk file's ground heights out of the ROM, by its number, which say where it drops
/// (`archives.MapChunks.heights`).
pub type FarHeights = Rc<dyn Fn(i64) -> Result<Option<Vec<Option<f64>>>>>;

/// A matrix's width in chunks and each chunk's file, out of the ROM, by its number
/// (`archives.MapChunks.matrix`): the sea's above the water, whose pockets say which deep water
/// joins which (`_surfaces`). None without a ROM.
pub type FarMatrices = Rc<dyn Fn(i64) -> Result<Option<(i64, Vec<i64>)>>>;

/// A cell's word on a matrix out of the ROM, None off it or where it has no chunk (`_sea_words`).
type Word = Rc<dyn Fn(Cell) -> Result<Option<i64>>>;

/// Every way `below` can be moved to land each of its cells on a cell of `deep`: each that lands
/// its first cell on one, given up at the first cell that lands elsewhere.
fn shifts(below: &IndexSet<Cell>, deep: &IndexSet<Cell>) -> Vec<Cell> {
    let Some(&first) = below.iter().min() else {
        return Vec::new();
    };
    deep.iter()
        .map(|&(x, z)| (x - first.0, z - first.1))
        .filter(|&(dx, dz)| below.iter().all(|&(x, z)| deep.contains(&(x + dx, z + dz))))
        .collect()
}

/// Every cell the player can reach from `start` above the water, surfing and on foot: open cells
/// side by side, and the shore, which Surf jumps.
fn sea(start: Cell, word: &Word) -> Result<HashSet<Cell>> {
    // ponytail: a ledge is a wall either way here; none is known to part the sea above the water a
    // player dives through.
    let mut seen: HashSet<Cell> = HashSet::from([start]);
    let mut todo = vec![start];
    while let Some(here) = todo.pop() {
        for near in beside(here) {
            let one = word(near)?;
            if let Some(one) = one
                && !seen.contains(&near)
                && (one & BLOCKED == 0 || one >> KIND_SHIFT == SHORE)
            {
                seen.insert(near);
                todo.push(near);
            }
        }
    }
    Ok(seen)
}

/// `pathfind.STEPS[way]`.
fn step_of(way: &str) -> Cell {
    pathfind::STEPS
        .iter()
        .find(|(known, _)| *known == way)
        .map(|&(_, step)| step)
        .expect("a way of the compass")
}

/// The four cells a step from this one.
fn beside(cell: Cell) -> [Cell; 4] {
    pathfind::STEPS.map(|(_, (dx, dz))| (cell.0 + dx, cell.1 + dz))
}

/// The chunk files the ways up a kind's cells were found for, and the ways (`rises`).
type Rises = (Rc<Vec<i64>>, IndexMap<Cell, &'static str>);

/// Each ROM chunk of the matrix with its ground heights, by its key (`Terrain::tables`).
type Tables = IndexMap<(i64, i64), (Vec<i64>, Vec<Option<f64>>)>;

/// Each chunk's floors of the matrix's streets, by its key, and every cell of one
/// (`Terrain::found_streets`).
type StreetFloors = (Rc<IndexMap<(i64, i64), Vec<Floor>>>, Rc<HashSet<Cell>>);

/// The ROM's chunks of the matrix the field map lists, read a file at a time and kept.
pub struct Terrain {
    streets: Option<rail::Streets>,
    /// The chunk files the streets' floors were found for, each chunk's floors of them by its
    /// key, and every cell of one (`with_streets`).
    streets_of: RefCell<(Option<Rc<Vec<i64>>>, StreetFloors)>,
    cells: Option<FarCells>,
    floors: Option<FarFloors>,
    heights: Option<FarHeights>,
    matrices: Option<FarMatrices>,
    /// The matrix's width in chunks, and each chunk's file, row by row, as the field map lists
    /// them now: the field reader sets them as it reads the map.
    pub files: RefCell<(i64, Rc<Vec<i64>>)>,
    /// (column, row, file) -> the ROM's chunk there, once read.
    chunks: RefCell<HashMap<(i64, i64, i64), Option<Rc<Chunk>>>>,
    /// (id of a chunk with floors, the player's height rounded, or None for any floor) -> the
    /// chunk and the chunk flattened so: the ground is built again at every step guided.
    ///
    /// The id is the `Rc`'s address, which the chunk kept here holds fixed.
    flat: RefCell<HashMap<(usize, Option<i64>), (Rc<Chunk>, Rc<Chunk>)>>,
    /// (the chunk files, the height rounded) -> the drops found for them (`cliffs`).
    drops: RefCell<HashMap<(Vec<i64>, Option<i64>), IndexSet<(Cell, Cell)>>>,
    /// Kinds -> the chunk files the ways up their cells were found for, and the ways (`rises`).
    ///
    /// Python keys it by a `frozenset`; a `BTreeSet` is equal where the frozenset is.
    rises_of: RefCell<IndexMap<BTreeSet<i64>, Rises>>,
    /// The chunk files the climbing walls were found for, and the walls (`walls`).
    walls_of: RefCell<(Option<Rc<Vec<i64>>>, Rc<Walls>)>,
    /// The chunk files the climbing walls' sideways steps were found for, and the steps
    /// (`sides`). A `frozenset`, only tested.
    sides_of: RefCell<(Option<Rc<Vec<i64>>>, Rc<HashSet<(Cell, Cell)>>)>,
    /// The chunk files the grind rails were found for, and the rails (`grind_rails`).
    grinds_of: RefCell<(Option<Rc<Vec<i64>>>, Rc<GrindRails>)>,
    /// The chunk files the floors were read for, and whether one has a deck (`stacked`).
    stacked: RefCell<(Option<Rc<Vec<i64>>>, bool)>,
}

impl Terrain {
    /// Python takes the ROM's `MapChunks`, or None without a ROM, and keeps four of its methods;
    /// here they are handed over as they are kept. `streets` the streets in chunk files
    /// (`maps.Streets.network`), or None in a game without them.
    pub fn new(
        cells: Option<FarCells>,
        floors: Option<FarFloors>,
        heights: Option<FarHeights>,
        matrices: Option<FarMatrices>,
        streets: Option<rail::Streets>,
    ) -> Self {
        Terrain {
            streets,
            streets_of: RefCell::new((None, (Rc::new(IndexMap::new()), Rc::new(HashSet::new())))),
            cells,
            floors,
            heights,
            matrices,
            files: RefCell::new((0, Rc::new(Vec::new()))),
            chunks: RefCell::new(HashMap::new()),
            flat: RefCell::new(HashMap::new()),
            drops: RefCell::new(HashMap::new()),
            rises_of: RefCell::new(IndexMap::new()),
            walls_of: RefCell::new((None, Rc::new(Walls::default()))),
            sides_of: RefCell::new((None, Rc::new(HashSet::new()))),
            grinds_of: RefCell::new((None, Rc::new(GrindRails::default()))),
            stacked: RefCell::new((None, false)),
        }
    }

    /// Whether the ROM's chunks are read: without, a chunk the game has not loaded is not known.
    pub fn known(&self) -> bool {
        self.cells.is_some()
    }

    /// Each of `below`'s cells of deep water with every one the sea above joins it to: the sea
    /// is the matrix of one of `seas` laid over this one where every cell of deep water lands on
    /// deep water, which in ORAS it does exactly and in one place: under Route 126 the cells are
    /// the sea's less (80, 80), under Route 128 the same cells (spike 165). Where more than one
    /// fits, or none, nothing is joined: a way said wrong is worse than none.
    pub fn surfaces(
        &self,
        below: &IndexSet<Cell>,
        seas: &[i64],
    ) -> Result<IndexMap<Cell, Rc<HashSet<Cell>>>> {
        let mut found: Vec<(Word, Cell)> = Vec::new();
        for &number in seas {
            let matrix = match &self.matrices {
                None => None,
                Some(matrices) => matrices(number)?,
            };
            let Some(matrix) = matrix else {
                continue;
            };
            let (word, deep) = self.sea_words(matrix.0, &matrix.1)?;
            for shift in shifts(below, &deep) {
                found.push((word.clone(), shift));
            }
        }
        if found.len() != 1 || below.is_empty() {
            return Ok(IndexMap::new());
        }
        let (word, (dx, dz)) = &found[0];
        let mut out: IndexMap<Cell, Rc<HashSet<Cell>>> = IndexMap::new();
        for &cell in below {
            if out.contains_key(&cell) {
                continue;
            }
            let pocket = sea((cell.0 + dx, cell.1 + dz), word)?;
            let joined: Rc<HashSet<Cell>> = Rc::new(
                pocket
                    .iter()
                    .map(|&(x, z)| (x - dx, z - dz))
                    .filter(|here| below.contains(here))
                    .collect(),
            );
            for &here in joined.iter() {
                out.insert(here, joined.clone());
            }
        }
        Ok(out)
    }

    /// The chunk at (column, row) of the matrix as the ROM has it, by the file the field map
    /// lists there, or None off the matrix, where it lists none, or without a ROM.
    pub fn far(&self, column: i64, row: i64) -> Result<Option<Rc<Chunk>>> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        let Some(cells) = &self.cells else {
            return Ok(None);
        };
        if !(0 <= column && column < wide) || row < 0 {
            return Ok(None);
        }
        let at = row * wide + column;
        if at >= files.len() as i64 || files[at as usize] == NO_CHUNK_FILE {
            return Ok(None);
        }
        let file = files[at as usize];
        let key = (column, row, file);
        if !self.chunks.borrow().contains_key(&key) {
            let words = cells(file)?;
            let levels = match &self.floors {
                None => Vec::new(),
                Some(floors) => floors(file)?,
            };
            let chunk = words.map(|words| {
                Rc::new(Chunk {
                    floors: levels,
                    ..Chunk::new((column * CHUNK, row * CHUNK), words)
                })
            });
            self.chunks.borrow_mut().insert(key, chunk);
        }
        Ok(self.chunks.borrow()[&key].clone())
    }

    /// The chunk on the floor nearest `height` cell by cell, or on any floor open where
    /// `at_height` is False (`mapblock.flat`), made once a height.
    pub fn flattened(
        &self,
        chunk: &Rc<Chunk>,
        height: Option<f64>,
        at_height: bool,
    ) -> Result<Rc<Chunk>> {
        if chunk.floors.is_empty() || (at_height && height.is_none()) {
            return Ok(chunk.clone());
        }
        let rounded = match height {
            Some(height) if at_height => Some(py::round(height)?),
            _ => None,
        };
        let key = (Rc::as_ptr(chunk) as usize, rounded);
        if !self.flat.borrow().contains_key(&key) {
            // ponytail: emptied whole when full; a few chunks at a few heights is all a map holds.
            if self.flat.borrow().len() as i64 > FLATTENED {
                self.flat.borrow_mut().clear();
            }
            let flat = mapblock::flat(chunk, rounded.map(|height| height as f64));
            self.flat
                .borrow_mut()
                .insert(key, (chunk.clone(), Rc::new(flat)));
        }
        Ok(self.flat.borrow()[&key].1.clone())
    }

    /// Open cells side by side, each way, a drop apart (`mapblock.DROP`), but for the zone's
    /// `ladders`: Fortree City's platforms stand 108 over the ground beside them in one cell
    /// table, and P walked the player off one, "9 south" into nothing, where the game lets them
    /// down only by a ladder (spike 160). Read off the ROM's chunks of one floor across the whole
    /// matrix, a chunk with a deck on the floor each cell is walked on at `level`, the player's
    /// height; none without the ROM.
    pub fn cliffs(
        &self,
        level: Option<f64>,
        ladders: &IndexMap<(Cell, Cell), (f64, f64)>,
    ) -> Result<IndexSet<(Cell, Cell)>> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        let key = (
            (*files).clone(),
            match level {
                None => None,
                Some(level) => Some(py::round(level)?),
            },
        );
        let held = self.drops.borrow().get(&key).cloned();
        let drops = match held {
            Some(drops) => drops,
            None => {
                // ponytail: emptied whole when full; a map is walked at a few heights.
                if self.drops.borrow().len() as i64 > FLATTENED {
                    self.drops.borrow_mut().clear();
                }
                let found = self.find_drops(wide, &files, level)?;
                self.drops.borrow_mut().insert(key, found.clone());
                found
            }
        };
        if drops.is_empty() {
            return Ok(drops);
        }
        Ok(drops
            .into_iter()
            .filter(|pair| !ladders.contains_key(pair))
            .collect())
    }

    /// Every drop on the matrix with its decks walked at `level` (`cliffs`).
    fn find_drops(
        &self,
        wide: i64,
        files: &[i64],
        level: Option<f64>,
    ) -> Result<IndexSet<(Cell, Cell)>> {
        if self.heights.is_none() || wide == 0 {
            return Ok(IndexSet::new());
        }
        let mut tables: IndexMap<(i64, i64), (Vec<i64>, Vec<Option<f64>>)> = IndexMap::new();
        for (at, &file) in files.iter().enumerate() {
            let at = at as i64;
            let Some(chunk) = self.far(py::modulo(at, wide), py::floordiv(at, wide))? else {
                continue;
            };
            if !chunk.floors.is_empty() {
                let flat = self.flattened(&chunk, level, true)?;
                let heights = if flat.heights.is_empty() {
                    chunk.floors[0].heights.clone()
                } else {
                    flat.heights.clone()
                };
                tables.insert(chunk.key(), (flat.words.clone(), heights));
            } else if let Some(heights) = self.heights.as_ref().map_or(Ok(None), |f| f(file))? {
                tables.insert(chunk.key(), (chunk.words.clone(), heights));
            }
        }

        let height = |cell: Cell| -> Result<Option<f64>> {
            let Some(table) =
                tables.get(&(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK)))
            else {
                return Ok(None);
            };
            let i = py::modulo(cell.1, CHUNK) * CHUNK + py::modulo(cell.0, CHUNK);
            let word = *usize::try_from(i)
                .ok()
                .and_then(|i| table.0.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?;
            if word & BLOCKED != 0 {
                return Ok(None);
            }
            Ok(*usize::try_from(i)
                .ok()
                .and_then(|i| table.1.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?)
        };

        // A street's floor is not among the floors flattened here, and a ramp's height changes
        // from one cell to the next: a drop beside one is the ground's under it, which
        // `Ground.climb` keeps it from as it does any floor a stair's step away.
        let streets = self.found_streets()?.1;
        let mut out: IndexSet<(Cell, Cell)> = IndexSet::new();
        for &(column, row) in tables.keys() {
            for z in row * CHUNK..(row + 1) * CHUNK {
                for x in column * CHUNK..(column + 1) * CHUNK {
                    let Some(here) = height((x, z))? else {
                        continue;
                    };
                    for near in [(x + 1, z), (x, z + 1)] {
                        let there = height(near)?;
                        if let Some(there) = there
                            && (here - there).abs() > mapblock::DROP
                            && !streets.contains(&(x, z))
                            && !streets.contains(&near)
                        {
                            out.insert(((x, z), near));
                            out.insert((near, (x, z)));
                        }
                    }
                }
            }
        }
        Ok(out)
    }

    /// Each cell of `kinds` on the matrix with the way it goes up: between the two open cells on
    /// its opposite sides, towards the higher, as the ROM's heights have them. Every one of
    /// Jagged Pass's 24 bumpy slopes has one such pair, a cell's height apart, north for most and
    /// west for three (spike 159), and Ever Grande City's waterfall has the sea at 0 below and
    /// the water at 234 above (spike 170). None without the ROM's heights.
    pub fn rises(&self, kinds: &[i64]) -> Result<IndexMap<Cell, &'static str>> {
        let files = self.files.borrow().1.clone();
        let key: BTreeSet<i64> = kinds.iter().copied().collect();
        if let Some(cached) = self.rises_of.borrow().get(&key)
            && cached.0 == files
        {
            return Ok(cached.1.clone());
        }
        let mut out: IndexMap<Cell, &'static str> = IndexMap::new();
        let tables = self.tables()?;

        let read = |cell: Cell| -> Result<Option<(i64, Option<f64>)>> {
            let table = tables.get(&(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK)));
            let i = py::modulo(cell.1, CHUNK) * CHUNK + py::modulo(cell.0, CHUNK);
            let Some(table) = table else {
                return Ok(None);
            };
            let word = *usize::try_from(i)
                .ok()
                .and_then(|i| table.0.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?;
            let height = *usize::try_from(i)
                .ok()
                .and_then(|i| table.1.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?;
            Ok(Some((word, height)))
        };

        for (&(column, row), (words, _)) in &tables {
            for (i, &word) in words.iter().enumerate() {
                let i = i as i64;
                if !kinds.contains(&(word >> KIND_SHIFT)) {
                    continue;
                }
                let cell = (column * CHUNK + i % CHUNK, row * CHUNK + i / CHUNK);
                for (one, other) in [
                    (pathfind::NORTH, pathfind::SOUTH),
                    (pathfind::WEST, pathfind::EAST),
                ] {
                    let (first, second) = {
                        let first = step_of(one);
                        let first = read((cell.0 + first.0, cell.1 + first.1))?;
                        let second = step_of(other);
                        let second = read((cell.0 + second.0, cell.1 + second.1))?;
                        (first, second)
                    };
                    let (Some(first), Some(second)) = (first, second) else {
                        continue;
                    };
                    if (first.0 | second.0) & BLOCKED != 0 {
                        continue;
                    }
                    if let (Some(a), Some(b)) = (first.1, second.1)
                        && a != b
                    {
                        out.insert(cell, if a > b { one } else { other });
                    }
                }
            }
        }
        self.rises_of.borrow_mut().insert(key.clone(), (files, out));
        Ok(self.rises_of.borrow()[&key].1.clone())
    }

    /// Each step, both ways, from a cell of one of `climbing`'s sideways kinds the way its
    /// kind names, off the ROM's chunks, found once a matrix: on foot never taken. In the Poké
    /// Ball Factory those kinds stand at a ramp's sides, and the player could not step east from
    /// (14, 22), `0x51`, onto the ramp at (15, 22), nor west back off it, nor north from (15, 4),
    /// `0x53`, onto the ramp at (15, 3) (spike 259); in Cyllage Gym the step is off a face onto a
    /// ledge, a climber's (`Ground.joined`).
    pub fn sides(&self, climbing: &Climbing) -> Result<Rc<HashSet<(Cell, Cell)>>> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        if self.sides_of.borrow().0.as_ref() != Some(&files) {
            let mut out: HashSet<(Cell, Cell)> = HashSet::new();
            for at in 0..files.len() as i64 {
                let Some(chunk) = self.far(py::modulo(at, wide), py::floordiv(at, wide))? else {
                    continue;
                };
                for (cell, kind) in chunk.kinds() {
                    if let Some(&way) = climbing.sideways.get(&kind) {
                        let (dx, dz) = pathfind::STEPS
                            .iter()
                            .find(|(known, _)| *known == way)
                            .map(|&(_, step)| step)
                            .ok_or_else(|| Error::key(py::repr(way)))?;
                        let step = (cell.0 + dx, cell.1 + dz);
                        out.insert((cell, step));
                        out.insert((step, cell));
                    }
                }
            }
            *self.sides_of.borrow_mut() = (Some(files), Rc::new(out));
        }
        Ok(self.sides_of.borrow().1.clone())
    }

    /// Every climbing wall on the matrix (`Walls.found`), off the ROM's chunks with their
    /// ground heights, found once a matrix; none without the ROM's heights. Only chunks with a
    /// `climbing.face` or `climbing.slide` cell are read whole: Cyllage Gym's one chunk holds
    /// all 28 of X's faces and both its slides (spike 245).
    pub fn walls(&self, climbing: &Climbing) -> Result<Rc<Walls>> {
        let files = self.files.borrow().1.clone();
        if self.walls_of.borrow().0.as_ref() != Some(&files) {
            let kinds = [climbing.face, climbing.slide];
            let mut cells: IndexMap<Cell, (i64, Option<f64>)> = IndexMap::new();
            for ((column, row), (words, heights)) in self.tables()? {
                if words
                    .iter()
                    .any(|&word| kinds.contains(&(word >> KIND_SHIFT)))
                {
                    for (i, &word) in words.iter().enumerate() {
                        let height = *heights
                            .get(i)
                            .ok_or_else(|| Error::index("tuple index out of range"))?;
                        let i = i as i64;
                        cells.insert(
                            (column * CHUNK + i % CHUNK, row * CHUNK + i / CHUNK),
                            (word, height),
                        );
                    }
                }
            }
            let walls = Rc::new(Walls::found(
                &cells,
                climbing.face,
                &climbing.sideways,
                climbing.slide,
            ));
            *self.walls_of.borrow_mut() = (Some(files), walls);
        }
        Ok(self.walls_of.borrow().1.clone())
    }

    /// Every grind rail on the matrix (`GrindRails.found`), off the ROM's chunks, every floor
    /// of each, found once a matrix; none without the ROM. Only chunks with a `grinding.rail`
    /// cell on a floor are read, and the chunks round them for the cells past a rail's ends:
    /// Shalour Gym's way out is on its floor 1, and on Route 5 a rail runs on into file 57,
    /// which holds 21 of its cells and no end (spike 247).
    pub fn grind_rails(&self, grinding: &Grinding) -> Result<Rc<GrindRails>> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        if self.grinds_of.borrow().0.as_ref() != Some(&files) {
            let mut chunks: IndexMap<(i64, i64), Rc<Chunk>> = IndexMap::new();
            for at in 0..files.len() as i64 {
                if let Some(chunk) = self.far(py::modulo(at, wide), py::floordiv(at, wide))? {
                    chunks.insert(chunk.key(), chunk);
                }
            }

            // A chunk with a deck keeps its ground as its first floor.
            fn tables(chunk: &Chunk) -> Vec<&Vec<i64>> {
                if chunk.floors.is_empty() {
                    vec![&chunk.words]
                } else {
                    chunk.floors.iter().map(|floor| &floor.words).collect()
                }
            }

            let held: HashSet<(i64, i64)> = chunks
                .iter()
                .filter(|(_, chunk)| {
                    tables(chunk).iter().any(|table| {
                        table
                            .iter()
                            .any(|&word| word >> KIND_SHIFT == grinding.rail)
                    })
                })
                .map(|(&key, _)| key)
                .collect();
            let near: HashSet<(i64, i64)> = held
                .iter()
                .flat_map(|&(column, row)| {
                    [-1, 0, 1].into_iter().flat_map(move |dx| {
                        [-1, 0, 1]
                            .into_iter()
                            .map(move |dz| (column + dx, row + dz))
                    })
                })
                .collect();
            let mut keys: Vec<(i64, i64)> = near
                .into_iter()
                .filter(|key| chunks.contains_key(key))
                .collect();
            keys.sort();
            let mut floors: Vec<IndexMap<Cell, i64>> = Vec::new();
            for key in keys {
                for (k, table) in tables(&chunks[&key]).into_iter().enumerate() {
                    if k == floors.len() {
                        floors.push(IndexMap::new());
                    }
                    for (i, &word) in table.iter().enumerate() {
                        let i = i as i64;
                        floors[k]
                            .insert((key.0 * CHUNK + i % CHUNK, key.1 * CHUNK + i / CHUNK), word);
                    }
                }
            }
            let rails = Rc::new(GrindRails::found(&floors, grinding.rail, grinding.end));
            *self.grinds_of.borrow_mut() = (Some(files), rails);
        }
        Ok(self.grinds_of.borrow().1.clone())
    }

    /// Each ROM chunk of the matrix with its ground heights, by its key: its cell table and
    /// a height a cell (`rises`, `walls`).
    fn tables(&self) -> Result<Tables> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        let mut tables: Tables = IndexMap::new();
        for (at, &file) in files.iter().enumerate() {
            let at = at as i64;
            let chunk = self.far(py::modulo(at, wide), py::floordiv(at, wide))?;
            let heights = match &self.heights {
                None => None,
                Some(heights) => heights(file)?,
            };
            if let (Some(chunk), Some(heights)) = (chunk, heights) {
                tables.insert(chunk.key(), (chunk.words.clone(), heights));
            }
        }
        Ok(tables)
    }

    /// Whether a chunk of the matrix has a deck over its ground, a floor with a cell open at
    /// a height: the loaded chunks, and with the ROM every chunk of the matrix, once a matrix.
    /// Not whether a deck stands apart from the player: Mt. Chimney's ground and catwalk both
    /// sit at 108 over most of their cells, the ground falling to 4 only down the path by the
    /// crater, and a player on the summit was walked one floor a cell, an item ball on that path
    /// 104 below taken for one beside them (or_3d_pathfinding, 2026-10-24).
    pub fn stacked(&self, loaded: &dyn Fn() -> Result<Vec<Rc<Chunk>>>) -> Result<bool> {
        let files = self.files.borrow().1.clone();
        if self.stacked.borrow().0.as_ref() != Some(&files) {
            let mut floors: Vec<Floor> = Vec::new();
            for chunk in loaded()? {
                floors.extend(chunk.floors.iter().cloned());
            }
            if let Some(far_floors) = &self.floors {
                for file in py::PySet::of(files.iter().copied())
                    .difference(&py::PySet::of([NO_CHUNK_FILE]))
                    .iter()
                {
                    floors.extend(far_floors(file)?);
                }
            }
            // `raised or any(...)` over `zip(..., strict=True)`: it stops at the first open
            // cell, and a floor whose two lists differ in length raises only once the shorter
            // runs out.
            let mut any = self.raised()?;
            'floors: for floor in &floors {
                if any {
                    break;
                }
                for (&word, &height) in floor.words.iter().zip(&floor.heights) {
                    if height.is_some() && word & BLOCKED == 0 {
                        any = true;
                        break 'floors;
                    }
                }
                if floor.words.len() != floor.heights.len() {
                    return Err(Error::value(format!(
                        "zip() argument 2 is {} than argument 1",
                        if floor.heights.len() < floor.words.len() {
                            "shorter"
                        } else {
                            "longer"
                        }
                    )));
                }
            }
            *self.stacked.borrow_mut() = (Some(files), any);
        }
        Ok(self.stacked.borrow().1)
    }

    /// Whether the matrix's streets are floors of their own, a ramp among them
    /// (`rail.street_floors`): walked a floor at a time, and opened by no lane.
    pub fn raised(&self) -> Result<bool> {
        Ok(!self.found_streets()?.1.is_empty())
    }

    /// The chunk with a floor for each of the streets' floors over it after its own, its
    /// ground first where it has no other (`rail.street_floors`): walked a floor at a time, a
    /// step up a ramp is taken as one up a stair is (`Ground.climb`).
    pub fn with_streets(&self, chunk: &Rc<Chunk>) -> Result<Rc<Chunk>> {
        let found = self.found_streets()?.0;
        let streets = match found.get(&chunk.key()) {
            Some(streets) if !streets.is_empty() => streets,
            _ => return Ok(chunk.clone()),
        };
        let mut floors = if chunk.floors.is_empty() {
            let mut heights = Vec::new();
            for i in 0..CHUNK * CHUNK {
                heights
                    .push(self.height((chunk.origin.0 + i % CHUNK, chunk.origin.1 + i / CHUNK))?);
            }
            vec![Floor {
                words: chunk.words.clone(),
                heights,
                ramp: false,
            }]
        } else {
            chunk.floors.clone()
        };
        floors.extend(streets.iter().cloned());
        Ok(Rc::new(Chunk {
            floors,
            ..(**chunk).clone()
        }))
    }

    /// Each chunk's floors of the matrix's streets, by its key, and every cell of one, out of
    /// the streets in the ROM's chunks, once a matrix; none without the ROM or the streets, or
    /// where no ramp is among them. A street's cell keeps the word of the ground under it where
    /// that is open, plain floor where it is not, and every other cell is blocked. The Tower of
    /// Mastery's ramp, the only way up from its ground to its landings, is in its chunk's
    /// streets and in none of its floors' meshes (spike 248).
    fn found_streets(&self) -> Result<StreetFloors> {
        let files = self.files.borrow().1.clone();
        if self.streets_of.borrow().0.as_ref() != Some(&files) {
            let mut found: IndexMap<(i64, i64), Vec<Floor>> = IndexMap::new();
            let mut floors: Vec<(IndexMap<Cell, f64>, bool)> = Vec::new();
            if let Some(streets) = &self.streets
                && self.known()
            {
                let listed: Vec<i64> = files
                    .iter()
                    .copied()
                    .filter(|&f| f != NO_CHUNK_FILE)
                    .collect();
                floors = rail::street_floors(&streets(&listed)?, &|cell| self.ground(cell))?;
            }
            for (floor, ramp) in &floors {
                let keys: BTreeSet<(i64, i64)> = floor
                    .keys()
                    .map(|&(x, z)| (py::floordiv(x, CHUNK), py::floordiv(z, CHUNK)))
                    .collect();
                for key in keys {
                    let Some(chunk) = self.far(key.0, key.1)? else {
                        continue;
                    };
                    let under = if chunk.floors.is_empty() {
                        &chunk.words
                    } else {
                        &chunk.floors[0].words
                    };
                    let mut words = Vec::new();
                    let mut heights = Vec::new();
                    for i in 0..CHUNK * CHUNK {
                        let cell = (chunk.origin.0 + i % CHUNK, chunk.origin.1 + i / CHUNK);
                        words.push(if !floor.contains_key(&cell) {
                            BLOCKED
                        } else {
                            let word = *under
                                .get(i as usize)
                                .ok_or_else(|| Error::index("tuple index out of range"))?;
                            if word & BLOCKED != 0 { 0 } else { word }
                        });
                        heights.push(floor.get(&cell).copied());
                    }
                    found.entry(key).or_default().push(Floor {
                        words,
                        heights,
                        ramp: *ramp,
                    });
                }
            }
            let cells: HashSet<Cell> = floors
                .iter()
                .flat_map(|(floor, _)| floor.keys().copied())
                .collect();
            *self.streets_of.borrow_mut() = (Some(files), (Rc::new(found), Rc::new(cells)));
        }
        Ok(self.streets_of.borrow().1.clone())
    }

    /// The heights of the floors at a cell of the ROM's chunks, walls among them, none where
    /// the chunk is not read or no floor has a height there (`rail.street_floors`): Route 7's
    /// street runs over cells its chunk walls, at the ground's height.
    fn ground(&self, cell: Cell) -> Result<Vec<f64>> {
        let Some(chunk) = self.far(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK))?
        else {
            return Ok(Vec::new());
        };
        if !chunk.floors.is_empty() {
            let i = py::modulo(cell.1, CHUNK) * CHUNK + py::modulo(cell.0, CHUNK);
            let mut out = Vec::new();
            for floor in &chunk.floors {
                let height = *floor
                    .heights
                    .get(i as usize)
                    .ok_or_else(|| Error::index("tuple index out of range"))?;
                out.extend(height);
            }
            return Ok(out);
        }
        Ok(self.height(cell)?.into_iter().collect())
    }

    /// The ROM's ground height at a cell of the current matrix, or None where the chunk is not
    /// listed or its heights are not read.
    pub fn height(&self, cell: Cell) -> Result<Option<f64>> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        let (column, row) = (py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK));
        let at = row * wide + column;
        let Some(heights) = &self.heights else {
            return Ok(None);
        };
        if !(0 <= column && column < wide) || row < 0 || at >= files.len() as i64 {
            return Ok(None);
        }
        let file = files[at as usize];
        if file == NO_CHUNK_FILE {
            return Ok(None);
        }
        let Some(heights) = heights(file)? else {
            return Ok(None);
        };
        let index = py::modulo(cell.1, CHUNK) * CHUNK + py::modulo(cell.0, CHUNK);
        heights
            .get(index as usize)
            .copied()
            .ok_or_else(|| Error::index("list index out of range"))
    }

    /// A cell's word on a matrix out of the ROM, None off it or where it has no chunk, and the
    /// matrix's deep water.
    fn sea_words(&self, wide: i64, files: &[i64]) -> Result<(Word, IndexSet<Cell>)> {
        let mut tables: Vec<Option<Vec<i64>>> = Vec::new();
        for &file in files {
            tables.push(match &self.cells {
                Some(cells) if file != NO_CHUNK_FILE => cells(file)?,
                _ => None,
            });
        }
        let mut deep: IndexSet<Cell> = IndexSet::new();
        for (at, table) in tables.iter().enumerate() {
            let at = at as i64;
            for (i, &one) in table.iter().flatten().enumerate() {
                let i = i as i64;
                if one >> KIND_SHIFT == DIVE {
                    deep.insert((
                        py::modulo(at, wide) * CHUNK + i % CHUNK,
                        py::floordiv(at, wide) * CHUNK + i / CHUNK,
                    ));
                }
            }
        }

        let word = move |cell: Cell| -> Result<Option<i64>> {
            let (column, row) = (py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK));
            let at = row * wide + column;
            let table = if 0 <= column && column < wide && 0 <= at && at < tables.len() as i64 {
                tables[at as usize].as_ref()
            } else {
                None
            };
            match table {
                None => Ok(None),
                Some(table) => {
                    let index = py::modulo(cell.1, CHUNK) * CHUNK + py::modulo(cell.0, CHUNK);
                    table
                        .get(index as usize)
                        .copied()
                        .map(Some)
                        .ok_or_else(|| Error::index("tuple index out of range"))
                }
            }
        };
        Ok((Rc::new(word), deep))
    }
}
