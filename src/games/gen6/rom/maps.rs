//! The ROM's map chunks a file at a time, as P's ground reaches past the chunks the game has loaded
//! (spike 136), Lumiose's streets in them (spike 106), and the teleports a Gym's module places on
//! its streets (spike 264).

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{f32, u16, u32};
use crate::core::rom::{Rom, lz11};
use crate::games::common::field::chunk::Floor;
use crate::games::common::garc::{garc_files, garc_spans};
use crate::games::gen6::field::zonedata::CELL;
use crate::games::gen6::field::{mapblock, rail};
use crate::py::{self, Error, Kind, Result};

/// `items[index]`: a negative index counts from the end, and one out of range raises.
#[track_caller]
fn item<T>(items: &[T], index: i64) -> Result<&T> {
    let at = if index < 0 {
        index + items.len() as i64
    } else {
        index
    };
    if at < 0 || at >= items.len() as i64 {
        return Err(Error::index("list index out of range"));
    }
    Ok(&items[at as usize])
}

/// What the field reads the ROM's chunks through (`GameText.map_chunks`): `MapChunks`, or a
/// test's own, as Python's tests hand over any object with these four methods.
pub trait ChunkSource: std::fmt::Debug {
    fn cells(&self, file: i64) -> Result<Option<Vec<i64>>>;
    fn floors(&self, file: i64) -> Result<Vec<Floor>>;
    fn heights(&self, file: i64) -> Result<Option<Vec<Option<f64>>>>;
    fn matrix(&self, number: i64) -> Result<Option<(i64, Vec<i64>)>>;
}

/// What the field reads Lumiose's streets through (`GameText.streets`): `Streets`, or a test's
/// own.
pub trait StreetSource: std::fmt::Debug {
    fn network(&self, chunks: &[i64]) -> Result<rail::Network>;
}

impl ChunkSource for MapChunks {
    fn cells(&self, file: i64) -> Result<Option<Vec<i64>>> {
        MapChunks::cells(self, file)
    }

    fn floors(&self, file: i64) -> Result<Vec<Floor>> {
        MapChunks::floors(self, file)
    }

    fn heights(&self, file: i64) -> Result<Option<Vec<Option<f64>>>> {
        MapChunks::heights(self, file)
    }

    fn matrix(&self, number: i64) -> Result<Option<(i64, Vec<i64>)>> {
        MapChunks::matrix(self, number)
    }
}

impl StreetSource for Streets {
    fn network(&self, chunks: &[i64]) -> Result<rail::Network> {
        Streets::network(self, chunks)
    }
}

/// The lines of Lumiose's streets out of the ROM's map chunks, a chunk file at a time as a
/// map asks for them and kept once read: the game loads a line only with the chunks round the
/// player, so a door further along the street was on no line pa3ds knew (spike 106).
#[derive(Debug)]
pub struct Streets {
    path: PathBuf,
    archive: String,
    read: RefCell<IndexMap<i64, rail::Network>>,
}

impl Streets {
    /// Which of a chunk file's sections holds its streets.
    pub const SECTION: i64 = 4;

    pub fn new(path: &Path, archive: &str) -> Self {
        Streets {
            path: path.to_path_buf(),
            archive: archive.to_string(),
            read: RefCell::new(IndexMap::new()),
        }
    }

    /// Every line and track in these chunk files' streets, by id, and where their points
    /// and joints are. A file the ROM does not have, or that is not a chunk's, holds none; so
    /// does every chunk if the ROM cannot be read.
    pub fn network(&self, chunks: &[i64]) -> Result<rail::Network> {
        let mut wanted: Vec<i64> = Vec::new();
        for &chunk in chunks {
            if !self.read.borrow().contains_key(&chunk) && !wanted.contains(&chunk) {
                wanted.push(chunk);
            }
        }
        if !wanted.is_empty() {
            let files = match self.files() {
                Ok(files) => files,
                Err(error) if error.is(Kind::RomError) || error.is(Kind::OsError) => Vec::new(),
                Err(error) => return Err(error),
            };
            for chunk in wanted {
                let network = if chunk < files.len() as i64 {
                    chunk_streets(item(&files, chunk)?)?
                } else {
                    no_streets()
                };
                self.read.borrow_mut().insert(chunk, network);
            }
        }
        let mut lines: IndexMap<i64, rail::Line> = IndexMap::new();
        let mut places: IndexMap<i64, (f64, f64)> = IndexMap::new();
        let mut heights: IndexMap<i64, f64> = IndexMap::new();
        let mut sides: IndexMap<i64, (f64, f64)> = IndexMap::new();
        let mut cameras = 0;
        // `dict.fromkeys(chunks)`: each chunk once, in the order first named.
        let once: IndexSet<i64> = chunks.iter().copied().collect();
        for chunk in &once {
            let network = self
                .read
                .borrow()
                .get(chunk)
                .cloned()
                .unwrap_or_else(no_streets);
            for line in network.lines.values() {
                lines.entry(line.id).or_insert_with(|| line.clone());
            }
            for (&point, &place) in &network.places {
                places.entry(point).or_insert(place);
            }
            for (&point, &height) in &network.heights {
                heights.entry(point).or_insert(height);
            }
            for (&point, &side) in &network.sides {
                sides.entry(point).or_insert(side);
            }
            cameras += network.cameras;
        }
        Ok(rail::Network {
            lines,
            places,
            cameras,
            heights,
            sides,
        })
    }

    fn files(&self) -> Result<Vec<Vec<u8>>> {
        let rom = Rom::new(&self.path)?;
        garc_files(&rom.read(&self.archive)?)
    }
}

/// What `MapChunks` keeps of a chunk file: its cell table, its floors and its ground heights.
type Loaded = (Option<Vec<i64>>, Vec<Floor>, Option<Vec<Option<f64>>>);

/// The cell tables of the ROM's map chunks, a file at a time as P's ground reaches into a
/// chunk the game has not loaded, and kept once read: a way found where the map is not loaded
/// took the sea for land, and was given up when the chunks with the sea in them loaded
/// (omega_ruby_surf_2.ctm, spike 136).
#[derive(Debug)]
pub struct MapChunks {
    path: PathBuf,
    archive: String,
    matrices: Option<String>,
    read: RefCell<HashMap<i64, Loaded>>,
    matrix: RefCell<HashMap<i64, Option<(i64, Vec<i64>)>>>,
}

impl MapChunks {
    /// `MapChunks(path, archive)`: the common call, with no matrices to read.
    pub fn new(path: &Path, archive: &str) -> Self {
        MapChunks::with_matrices(path, archive, None)
    }

    pub fn with_matrices(path: &Path, archive: &str, matrices: Option<&str>) -> Self {
        MapChunks {
            path: path.to_path_buf(),
            archive: archive.to_string(),
            matrices: matrices.map(String::from),
            read: RefCell::new(HashMap::new()),
            matrix: RefCell::new(HashMap::new()),
        }
    }

    /// A matrix's width in chunks and each chunk's file, row by row, as its field map lists
    /// them, or None where the ROM has no such matrix or cannot be read: magic `MM`, a u16
    /// count of sections and a u32 offset a section, the first section a u16 0 or 1, a u16 0,
    /// u16 width and height in chunks, then the files. Under Route 126 it is 3 by 3, files
    /// 251 to 259, as the field map in RAM has them (spike 165).
    pub fn matrix(&self, number: i64) -> Result<Option<(i64, Vec<i64>)>> {
        if !self.matrix.borrow().contains_key(&number) {
            self.matrix.borrow_mut().insert(number, None);
            match self.read_matrix(number) {
                Ok(Some(found)) => {
                    self.matrix.borrow_mut().insert(number, Some(found));
                }
                Ok(None) => {}
                Err(error)
                    if error.is(Kind::RomError)
                        || error.is(Kind::OsError)
                        || error.is(Kind::ValueError)
                        || error.is(Kind::IndexError)
                        || error.is(Kind::StructError) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(self.matrix.borrow()[&number].clone())
    }

    fn read_matrix(&self, number: i64) -> Result<Option<(i64, Vec<i64>)>> {
        let matrices = self.matrices.as_deref().unwrap_or("");
        let data = {
            let rom = Rom::new(&self.path)?;
            let (start, spans) = garc_spans(&rom, matrices)?;
            let (at, end) = *item(&spans, number)?;
            rom.read_part(matrices, start + at, end - at)?
        };
        let first = u32(&data, 4)?;
        let (wide, tall) = (u16(&data, first + 4)?, u16(&data, first + 6)?);
        if py::slice(&data, 0, 2) == b"MM" && wide != 0 {
            let mut files = Vec::new();
            for i in 0..wide * tall {
                files.push(u16(&data, first + 8 + 2 * i)?);
            }
            return Ok(Some((wide, files)));
        }
        Ok(None)
    }

    /// One chunk file's cell table, or None where the ROM has no such file, it is not a
    /// chunk's, or the ROM cannot be read.
    pub fn cells(&self, file: i64) -> Result<Option<Vec<i64>>> {
        Ok(self.load(file)?.0)
    }

    /// One chunk file's floors, the ground first, or none for a chunk of one floor
    /// (`mapblock.floors`).
    pub fn floors(&self, file: i64) -> Result<Vec<Floor>> {
        Ok(self.load(file)?.1)
    }

    /// One chunk file's ground heights, a cell at a time, or None
    /// (`mapblock.ground_heights`).
    pub fn heights(&self, file: i64) -> Result<Option<Vec<Option<f64>>>> {
        Ok(self.load(file)?.2)
    }

    fn load(&self, file: i64) -> Result<Loaded> {
        if !self.read.borrow().contains_key(&file) {
            let loaded = match self.read_file(file) {
                Ok(loaded) => loaded,
                Err(error)
                    if error.is(Kind::RomError)
                        || error.is(Kind::OsError)
                        || error.is(Kind::ValueError)
                        || error.is(Kind::IndexError)
                        || error.is(Kind::StructError) =>
                {
                    (None, Vec::new(), None)
                }
                Err(error) => return Err(error),
            };
            self.read.borrow_mut().insert(file, loaded);
        }
        Ok(self.read.borrow()[&file].clone())
    }

    fn read_file(&self, file: i64) -> Result<Loaded> {
        let packed = {
            let rom = Rom::new(&self.path)?;
            let (start, spans) = garc_spans(&rom, &self.archive)?;
            let (at, end) = if file < spans.len() as i64 {
                *item(&spans, file)?
            } else {
                (0, 0)
            };
            rom.read_part(&self.archive, start + at, end - at)?
        };
        let data = if py::slice(&packed, 0, 1) == [0x11] {
            lz11(&packed)?
        } else {
            packed
        };
        Ok((
            mapblock::cells(&data)?,
            mapblock::floors(&data)?,
            mapblock::ground_heights(&data)?,
        ))
    }
}

/// `NO_STREETS`: a network with no lines and no places, which a `const` cannot hold.
pub fn no_streets() -> rail::Network {
    rail::Network::default()
}

/// One chunk file's streets, or none from a file that is not a chunk's.
fn chunk_streets(packed: &[u8]) -> Result<rail::Network> {
    let data = if py::slice(packed, 0, 1) == [0x11] {
        match lz11(packed) {
            Ok(data) => data,
            Err(error)
                if error.is(Kind::RomError)
                    || error.is(Kind::ValueError)
                    || error.is(Kind::IndexError) =>
            {
                return Ok(no_streets());
            }
            Err(error) => return Err(error),
        }
    } else {
        packed.to_vec()
    };
    if py::slice(&data, 0, 2) != b"GR" || data.len() < 8 {
        return Ok(no_streets());
    }
    let sections = u16(&data, 2)?;
    if sections <= Streets::SECTION || (data.len() as i64) < 8 + 4 * sections {
        return Ok(no_streets());
    }
    let (start, end) = (
        u32(&data, 4 + 4 * Streets::SECTION)?,
        u32(&data, 4 + 4 * Streets::SECTION + 4)?,
    );
    if start < end && end <= data.len() as i64 {
        rail::section_streets(py::slice(&data, start, end))
    } else {
        Ok(no_streets())
    }
}

/// One place of a Gym module's teleports: u32 a street's id, u32 a way, f32 along and f32 across
/// it in world units, each a whole number of cells.
pub const TELEPORT_ENTRY: i64 = 16;

/// The teleports a Gym's module holds: the longest run of its places (`TELEPORT_ENTRY`), the
/// first half where each teleport sends the player from and the second, in the same order, where
/// it puts them. DllUSGymEsper.cro, Anistar City Gym's, holds 38 from 0x2020, just before
/// `field::FieldUniqueSequenceGymEsper`'s name: the lobby's carpet at row 4 of its street is the
/// first start, landing on the landing's joint, and the joint at the Gym's middle at 770 sends
/// the player 3 across onto line 0x101df4b, as x_gym_7 and x_gym_7_teleporter did at 26.51 s
/// and 18.46 s (spike 264). The second word is not always the line's way, so only the run says
/// what is a place.
pub fn rail_teleports(module: &[u8]) -> Result<Vec<rail::Teleport>> {
    let place = |at: i64| -> Result<Option<rail::Place>> {
        let (line, way) = (u32(module, at)?, u32(module, at + 4)?);
        let (along, across) = (f32(module, at + 8)?, f32(module, at + 12)?);
        if line == 0
            || way > 3
            || ![along, across]
                .iter()
                .all(|&v| v.is_finite() && (v / CELL as f64).fract() == 0.0)
        {
            return Ok(None);
        }
        Ok(Some((line, along / CELL as f64, across / CELL as f64)))
    };
    let size = module.len() as i64;
    let mut best: Vec<rail::Place> = Vec::new();
    let mut start = 0;
    while start + TELEPORT_ENTRY <= size {
        let mut run: Vec<rail::Place> = Vec::new();
        let mut at = start;
        while at + TELEPORT_ENTRY <= size {
            let Some(one) = place(at)? else {
                break;
            };
            run.push(one);
            at += TELEPORT_ENTRY;
        }
        start = if run.is_empty() { start + 4 } else { at + 4 };
        if run.len() > best.len() {
            best = run;
        }
    }
    let half = best.len() / 2;
    Ok((0..half)
        .map(|i| rail::Teleport {
            start: best[i],
            lands: best[half + i],
        })
        .collect())
}
