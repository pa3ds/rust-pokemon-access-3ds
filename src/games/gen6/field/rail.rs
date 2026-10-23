//! Rails: how Lumiose City's boulevards place things in X, and the walls along them (spikes 102,
//! 103, 106, 107).
//!
//! A **line** is one stretch of boulevard between two points. A place on it is the line's id and
//! two numbers, in cells; the game keeps the walls of each line in a table of its own, a row a cell
//! along and a column a cell across, the rows running from the line's start point to its end point.
//! The two numbers are the line's **way** of measuring, a way of the compass the line keeps at
//! +0x10: the cells along and the offset across on a line measured south, both turned round on one
//! measured north, and the two swapped on one measured east or west (`Line.upright`, spike 107).
//! Lines meet at points, two to a point. A **track** is a short line of its own table from a point
//! to a **joint**, where up to eight tracks meet as streets cross and branch.
//!
//! ```text
//! table entry  u32 owner, u32 line id, u32 0, ptr the line's record, in the linear heap
//! line record  +0x00 id, its start point's with bit 15 set, +0x08 start point, +0x0c end
//!              point, +0x10 u32 its way (0 north, 1 south, 2 west, 3 east), +0x1c name,
//!              +0x3c u16 cells along, +0x3e u16 cells across, +0x44 ptr the file it is in,
//!              +0x54 u32 where its walls are in that file: (along + 1) rows of `across` words,
//!              bit 0 set where the player cannot walk, as a chunk's cells are
//! track record the same to +0x3e, its id never its start point's, its end a joint, then +0x40
//!              ptr the file and +0x44 its walls
//! ```
//!
//! The file is section 4 of a map chunk's file, and the game loads a line with the chunks it runs
//! through, so the table lists only the lines near the player, and a door on a line further on is
//! nowhere until the player walks up to it. The ROM has every chunk's section, each line whole in
//! every chunk it crosses, so `section_streets` reads a map's streets from there (spikes 106, 107).
//! Ids are a map's own: the same point id is in two other maps.
//!
//! ```text
//! section      +0x00 u32 the chunk's column << 8 | its row, +0x04 f32 18.0, +0x08 five u16
//!              counts, of lines, points, tracks, joints and cameras, +0x14 five u32s where each
//!              kind's table of offsets from the section's start is
//! point        +0x00 id, +0x08 the line or track ending there, +0x0c the line starting there,
//!              +0x10 f32 x, y, z from the middle of its chunk, 720 units a side
//! joint        +0x00 id, +0x08 eight u32 tracks meeting there, 0xffffffff after the last,
//!              +0x28 f32 x, y, z
//! ```
//!
//! Every line joined to the player's is laid out flat as one corridor, a grid of its own with the
//! first line running north, where a way is found as on any other ground. The camera there is
//! behind the player, and the D-pad works from it: up walks the way it looks, down walks back from
//! it, and left and right turn the player a quarter, the camera following only while the key is
//! held (spike 105). So a way on a rail is said from the camera (`pathfind.turned`), whose heading
//! the corridor gives as a bearing of its own (`bearing`).

use std::cell::{Cell as Slot, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{f32, s16, u16, u32};
use crate::core::discovery::{ALLOCATION, Found, HEAP_SPAN, LINEAR_SPAN, within};
use crate::core::memory::Memory;
use crate::core::pathfind::{Box, Cell};
use crate::games::common::field::ground::Lanes;
use crate::games::common::field::pad::quarter;
use crate::games::gen6::field::{camera, mmodel, zonedata};
use crate::py::{self, Error, Result};

/// A table entry: owner, line id, 0, the line's record.
pub const ENTRY: i64 = 16;
pub const LINE_HEAD: i64 = 0x58;
pub const LINE_START: i64 = 0x08;
pub const LINE_END: i64 = 0x0C;
pub const LINE_WAY: i64 = 0x10;
pub const LINE_ALONG: i64 = 0x3C;
pub const LINE_ACROSS: i64 = 0x3E;
pub const LINE_FILE: i64 = 0x44;
pub const LINE_WALLS: i64 = 0x54;
/// Set in a line's id over its start point's, and never so in a track's (829 lines, spike 107).
pub const LINE_BIT: i64 = 0x8000;
pub const TRACK_FILE: i64 = 0x40;
pub const TRACK_WALLS: i64 = 0x44;
pub const WAY_NORTH: i64 = 0;
pub const WAY_SOUTH: i64 = 1;
pub const WAY_WEST: i64 = 2;
pub const WAY_EAST: i64 = 3;
pub const SECTION_COUNTS: i64 = 0x08;
pub const SECTION_TABLES: i64 = 0x14;
/// The kinds of record a section lists, in the order of its counts and tables.
pub const LINES: i64 = 0;
pub const POINTS: i64 = 1;
pub const TRACKS: i64 = 2;
pub const JOINTS: i64 = 3;
pub const CAMERAS: i64 = 4;
pub const POINT_PLACE: i64 = 0x10;
pub const JOINT_PLACE: i64 = 0x28;
/// World units a map chunk is across: 40 cells of 18.
pub const CHUNK_UNITS: i64 = 720;
pub const BLOCKED: i64 = 0x1;
pub const MAX_TABLE: i64 = 0x4000;
/// Cells a line's wall table may hold: rows along times columns across.
pub const MAX_CELLS: i64 = 0x10000;

/// One stretch of boulevard and its walls.
///
/// `walls` is a shared slice where Python's is a tuple: a line is cloned wherever Python shares
/// the one frozen object, and this keeps that cheap.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub id: i64,
    pub start: i64,
    pub end: i64,
    /// Cells along it: its wall table has one row more, both ends included.
    pub along: i64,
    pub across: i64,
    pub walls: Rc<[i64]>,
    /// The way of the compass the game measures places on it by.
    pub way: i64,
    /// Whether it is a track, which the game measures from the joint it ends at, back along its
    /// rows: a person on a track measured west at (2, 0) stands at its joint, 2 cells to one side
    /// (spike 107).
    pub track: bool,
}

impl Line {
    /// A line measured south and not a track; `way` and `track` are set with
    /// `Line { way, track, ..Line::new(...) }`.
    pub fn new(
        id: i64,
        start: i64,
        end: i64,
        along: i64,
        across: i64,
        walls: impl Into<Rc<[i64]>>,
    ) -> Self {
        Line {
            id,
            start,
            end,
            along,
            across,
            walls: walls.into(),
            way: WAY_SOUTH,
            track: false,
        }
    }

    pub fn open(&self, row: i64, column: i64) -> bool {
        if !(0..=self.along).contains(&row) || !(0..self.across).contains(&column) {
            return false;
        }
        self.walls[(row * self.across + column) as usize] & BLOCKED == 0
    }

    /// A place as the game gives it on this line, as the cells along it from its start and
    /// the offset across it, rising to its left. The game measures by the line's way, as if
    /// the line ran that way of the world's (x, z): along it and across it on a line measured
    /// south, both turned round on one measured north, and on one measured east or west the
    /// two swapped. Through a warp into zone 0x44 the player stood at (-5, 8) on a line
    /// measured north, 5 cells from its start and 8 to its right (spike 107); a record fits
    /// its line so in 328 of Lumiose's 329, and every warp is on a wall beside an open cell,
    /// and 20 places of people and the player fit the world within a few units.
    pub fn upright(&self, a: f64, x: f64) -> (f64, f64) {
        let (along, across) = self.direction(a, x);
        if self.track {
            (self.along as f64 + along, across)
        } else {
            (along, across)
        }
    }

    /// A direction as the game gives it on this line, as its parts along the line and
    /// across it: measured by the way as a place is, and on a track turned round.
    pub fn direction(&self, a: f64, x: f64) -> (f64, f64) {
        let (along, across) = if self.way == WAY_NORTH {
            (-a, -x)
        } else if self.way == WAY_WEST {
            (-x, a)
        } else if self.way == WAY_EAST {
            (x, -a)
        } else {
            (a, x)
        };
        if self.track {
            (-along, -across)
        } else {
            (along, across)
        }
    }
}

/// A map's streets as the ROM keeps them: its lines and tracks by id, and where the world
/// has each point and joint they meet at, as (x, z).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Network {
    pub lines: IndexMap<i64, Line>,
    pub places: IndexMap<i64, (f64, f64)>,
    /// How many cameras its sections place: none in Lumiose, whose camera follows the player
    /// along a street, and some on Route 7 (2) and Parfum Palace's grounds (4), whose camera stays
    /// looking north while the keys walk the player the street's way nearest the one pressed (the
    /// tape of x_route-6-2.ctm, 2026-10-20).
    pub cameras: i64,
}

/// One cell's step in the corridor, a way of the compass: north is (0, -1).
pub type Step = (i64, i64);
pub const NORTH: Step = (0, -1);

/// The step a quarter turn left of `step`: west of north.
fn left(step: Step) -> Step {
    (step.1, -step.0)
}

pub const NORTH_KEYS: [(&str, Step); 2] = [("north", (0, -1)), ("south", (0, 1))];
/// The pad's two pairs of keys, each with the way on the screen it points.
pub const EAST_KEYS: [(&str, Step); 2] = [("east", (1, 0)), ("west", (-1, 0))];

/// Which of a pair of keys points nearer the way (dx, dz).
fn key(dx: f64, dz: f64, keys: &[(&'static str, Step); 2]) -> &'static str {
    // `max` keeps the first of two that tie.
    let toward = |step: Step| dx * step.0 as f64 + dz * step.1 as f64;
    if toward(keys[1].1) > toward(keys[0].1) {
        keys[1].0
    } else {
        keys[0].0
    }
}

/// The step to one of the eight cells round a cell nearest the way (dx, dz) points.
fn nearest_step(dx: f64, dz: f64) -> Result<Step> {
    let eighth = py::modulo(py::round(dz.atan2(dx) / (std::f64::consts::PI / 4.0))?, 8);
    Ok([
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
    ][eighth as usize])
}

/// `step` turned clockwise by `quarters` quarter turns.
fn turned(step: Step, quarters: i64) -> Step {
    let mut step = step;
    for _ in 0..py::modulo(quarters, 4) {
        step = (-step.1, step.0);
    }
    step
}

/// A line as the corridor has it: where the middle of its first row is, and the step one
/// row on takes, its offsets rising to the left of that.
#[derive(Debug, Clone, PartialEq)]
pub struct Laid {
    pub line: Line,
    pub origin: Cell,
    pub ahead: Step,
}

impl Laid {
    pub fn cell(&self, along: f64, across: f64) -> Result<Cell> {
        let ((ox, oz), (ax, az), (lx, lz)) = (self.origin, self.ahead, left(self.ahead));
        Ok((
            py::round(ox as f64 + along * ax as f64 + across * lx as f64)?,
            py::round(oz as f64 + along * az as f64 + across * lz as f64)?,
        ))
    }

    /// The row and the offset a cell is at, on the line or off it.
    pub fn place(&self, cell: Cell) -> (i64, i64) {
        let (dx, dz) = (cell.0 - self.origin.0, cell.1 - self.origin.1);
        let ((ax, az), (lx, lz)) = (self.ahead, left(self.ahead));
        (dx * ax + dz * az, dx * lx + dz * lz)
    }
}

/// A line as the world has it: where its start is in world units, and the unit step along
/// it, its offsets rising to its left as in the corridor (`_left`).
#[derive(Debug, Clone, Copy, PartialEq)]
struct World {
    start: (f64, f64),
    ahead: (f64, f64),
}

impl World {
    /// The grid's cell a row and an offset of the line are in.
    fn cell(&self, row: f64, offset: f64) -> Result<Cell> {
        let ((ax, az), (sx, sz)) = (self.ahead, self.start);
        let cell = zonedata::CELL as f64;
        let (x, z) = (
            sx + (row * ax + offset * az) * cell,
            sz + (row * az - offset * ax) * cell,
        );
        Ok((
            py::round((x - cell / 2.0) / cell)?,
            py::round((z - cell / 2.0) / cell)?,
        ))
    }

    /// The row and the offset of the line a grid cell's middle is nearest.
    fn place(&self, at: Cell) -> Result<(i64, i64)> {
        let ((ax, az), (sx, sz)) = (self.ahead, self.start);
        let cell = zonedata::CELL as f64;
        let (dx, dz) = (
            at.0 as f64 * cell + cell / 2.0 - sx,
            at.1 as f64 * cell + cell / 2.0 - sz,
        );
        Ok((
            py::round((dx * ax + dz * az) / cell)?,
            py::round((dx * az - dz * ax) / cell)?,
        ))
    }
}

/// The streets joined to the line the player is on, laid out flat: a cell is (x, z) with z
/// falling northward and x rising eastward, the way right moves, and the first line runs north
/// from (0, 0). A line goes on straight from the one it meets at a point, so a curving avenue
/// is laid as it is walked, and turns at a joint by the quarter nearest the way it leaves it in
/// the world (`lay_out`, spike 107).
#[derive(Debug, Clone, PartialEq)]
pub struct Corridor {
    pub laid: Vec<Laid>,
}

impl Corridor {
    fn of(&self, line: i64) -> Option<&Laid> {
        self.laid.iter().find(|laid| laid.line.id == line)
    }

    pub fn holds(&self, line: i64) -> bool {
        self.of(line).is_some()
    }

    pub fn line(&self, line: i64) -> Option<&Line> {
        self.of(line).map(|laid| &laid.line)
    }

    /// Where a place on one of the corridor's lines is in it, or None off them.
    pub fn cell(&self, line: i64, along: f64, across: f64) -> Result<Option<Cell>> {
        match self.of(line) {
            None => Ok(None),
            Some(laid) => Ok(Some(laid.cell(along, across)?)),
        }
    }

    /// A place or a direction as the game gives it on one of the corridor's lines, as its
    /// parts along the line and across it (`Line.upright`), or None off them.
    pub fn upright(&self, line: i64, a: f64, x: f64) -> Option<(f64, f64)> {
        self.of(line).map(|laid| laid.line.upright(a, x))
    }

    /// Where a place the game gives on one of the corridor's lines is in it.
    pub fn at(&self, line: i64, a: f64, x: f64) -> Result<Option<Cell>> {
        match self.upright(line, a, x) {
            None => Ok(None),
            Some(place) => self.cell(line, place.0, place.1),
        }
    }

    /// Where a direction on a line points in the corridor, in degrees clockwise from its
    /// north, the direction given as its parts across the line and along it; None for a line
    /// the corridor does not hold.
    pub fn bearing(&self, line: i64, across: f64, along: f64) -> Option<f64> {
        let laid = self.of(line)?;
        let ((ax, az), (lx, lz)) = (laid.ahead, left(laid.ahead));
        let (east, north) = (
            along * ax as f64 + across * lx as f64,
            -(along * az as f64 + across * lz as f64),
        );
        Some(py::modulo_f(east.atan2(north).to_degrees(), 360.0))
    }

    /// Whether the player can stand in a cell of the corridor: a cell no line holds is not
    /// somewhere on these streets.
    pub fn open(&self, cell: Cell) -> bool {
        for laid in &self.laid {
            let (row, offset) = laid.place(cell);
            if laid
                .line
                .open(row, offset + py::floordiv(laid.line.across, 2))
            {
                return true;
            }
        }
        false
    }

    pub fn r#box(&self) -> Result<Box> {
        let mut corners: Vec<Cell> = Vec::new();
        for laid in &self.laid {
            for along in [0, laid.line.along] {
                for across in [
                    -py::floordiv(laid.line.across, 2),
                    py::floordiv(laid.line.across, 2),
                ] {
                    corners.push(laid.cell(along as f64, across as f64)?);
                }
            }
        }
        let xs: Vec<i64> = corners.iter().map(|corner| corner.0).collect();
        let zs: Vec<i64> = corners.iter().map(|corner| corner.1).collect();
        let (Some(&west), Some(&north), Some(&east), Some(&south)) = (
            xs.iter().min(),
            zs.iter().min(),
            xs.iter().max(),
            zs.iter().max(),
        ) else {
            return Err(Error::value("min() iterable argument is empty"));
        };
        Ok(Box {
            west,
            north,
            east,
            south,
        })
    }
}

/// The streets joined to the line `through`, laid out with it running north from (0, 0).
/// Where two lines meet at a point the next goes on straight; where three or more meet, at a
/// joint, each is turned the quarter nearest the way it leaves the joint in the world, from
/// `places`, the world's (x, z) of each point, and a line whose way there is not known is left
/// out rather than laid across another.
pub fn lay_out(
    lines: &IndexMap<i64, Line>,
    through: i64,
    places: Option<&IndexMap<i64, (f64, f64)>>,
) -> Result<Option<Corridor>> {
    let Some(first) = lines.get(&through) else {
        return Ok(None);
    };
    let mut meeting: IndexMap<i64, Vec<&Line>> = IndexMap::new();
    for line in lines.values() {
        // Python's `{line.start, line.end}` is a set, visited in hash order; each point's list is
        // filled in `lines`' order whichever point comes first, so only the pair matters.
        let points = if line.start == line.end {
            vec![line.start]
        } else {
            vec![line.start, line.end]
        };
        for point in points {
            meeting.entry(point).or_default().push(line);
        }
    }
    let nowhere = IndexMap::new();
    let places = places.unwrap_or(&nowhere);
    let mut laid: IndexMap<i64, Laid> = IndexMap::new();
    laid.insert(
        first.id,
        Laid {
            line: first.clone(),
            origin: (0, 0),
            ahead: NORTH,
        },
    );
    let mut queue: VecDeque<i64> = VecDeque::from([first.id]);
    while let Some(next) = queue.pop_front() {
        let here = laid[&next].clone();
        for (point, row, outward) in [
            (here.line.end, here.line.along, here.ahead),
            (here.line.start, 0, turned(here.ahead, 2)),
        ] {
            let met: &[&Line] = meeting.get(&point).map_or(&[], Vec::as_slice);
            let mut sorted = met.to_vec();
            sorted.sort_by_key(|line| line.id);
            for other in sorted {
                if laid.contains_key(&other.id) {
                    continue;
                }
                let mut way = outward;
                if met.len() > 2 {
                    let Some(quarters) = quarters(&here.line, other, point, places)? else {
                        continue;
                    };
                    way = turned(outward, quarters);
                }
                let at = here.cell(row as f64, 0.0)?;
                laid.insert(other.id, leaving(other, point, at, way));
                queue.push_back(other.id);
            }
        }
    }
    Ok(Some(Corridor {
        laid: laid.into_values().collect(),
    }))
}

/// `line` laid so its end at `point` is the cell `at` and it runs on from there `way`.
fn leaving(line: &Line, point: i64, at: Cell, way: Step) -> Laid {
    if line.start == point {
        return Laid {
            line: line.clone(),
            origin: at,
            ahead: way,
        };
    }
    Laid {
        line: line.clone(),
        origin: (at.0 + line.along * way.0, at.1 + line.along * way.1),
        ahead: turned(way, 2),
    }
}

/// How many quarters clockwise `other` turns from the way `here` goes into the joint at
/// `point`, from where the world has the joint and each line's far end; None where it has not
/// got them. The world measures a heading from +z towards +x, as a line's way at a model's
/// +0xa4 and +0xac is given, and a line's offset rises a quarter on from its way there, which
/// is its left: so a world turn is the corridor's turned the other way round (spike 107).
fn quarters(
    here: &Line,
    other: &Line,
    point: i64,
    places: &IndexMap<i64, (f64, f64)>,
) -> Result<Option<i64>> {
    let centre = places.get(&point);
    let came = places.get(&if here.end == point {
        here.start
    } else {
        here.end
    });
    let goes = places.get(&if other.start == point {
        other.end
    } else {
        other.start
    });
    let (Some(centre), Some(came), Some(goes)) = (centre, came, goes) else {
        return Ok(None);
    };
    let into = (centre.0 - came.0).atan2(centre.1 - came.1);
    let out = (goes.0 - centre.0).atan2(goes.1 - centre.1);
    Ok(Some(py::modulo(
        py::round(-(out - into).to_degrees() / 90.0)?,
        4,
    )))
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

/// The table the rails loaded are listed in, found through the entry of a line the player
/// is on: the block the entry sits in, from its allocator's tag.
pub fn find_table(
    pages: &IndexMap<i64, Vec<u8>>,
    memory: &dyn Memory,
    line: i64,
) -> Result<Option<i64>> {
    let mut needle = Vec::new();
    needle.extend_from_slice(&u32_bytes(line)?);
    needle.extend_from_slice(&[0; 4]);
    for (&base, page) in pages {
        let mut at = find(page, &needle, 0);
        while let Some(index) = at {
            let entry = base + index as i64 - 4;
            if index % 4 == 0 && self::entry(memory, entry)?.is_some() {
                for back in (0..MAX_TABLE).step_by(4) {
                    let head = memory.read(entry - back, 8)?;
                    if u32(&head, 0)? == ALLOCATION
                        && back < u32(&head, 4)?
                        && u32(&head, 4)? <= MAX_TABLE
                    {
                        return Ok(Some(entry - back));
                    }
                }
                return Ok(None);
            }
            at = find(page, &needle, index + 1);
        }
    }
    Ok(None)
}

/// `struct.pack("<I", value)`.
#[track_caller]
fn u32_bytes(value: i64) -> Result<[u8; 4]> {
    match std::primitive::u32::try_from(value) {
        Ok(value) => Ok(value.to_le_bytes()),
        Err(_) => Err(Error::unpack(
            "'I' format requires 0 <= number <= 4294967295",
        )),
    }
}

/// Every line the table lists as loaded, read afresh only where its record is new.
pub fn read_table(
    memory: &dyn Memory,
    table: i64,
    known: &IndexMap<i64, Line>,
) -> Result<IndexMap<i64, Line>> {
    let size = u32(&memory.read(table + 4, 4)?, 0)?;
    if !(0 < size && size <= MAX_TABLE) {
        return Ok(IndexMap::new());
    }
    let raw = memory.read(table + 8, size)?;
    let mut out: IndexMap<i64, Line> = IndexMap::new();
    for at in (0..size - ENTRY + 1).step_by(4) {
        let (owner, line, zero, record) = (
            u32(&raw, at)?,
            u32(&raw, at + 4)?,
            u32(&raw, at + 8)?,
            u32(&raw, at + 12)?,
        );
        if zero != 0 || line == 0 || !within(HEAP_SPAN, owner) || !within(LINEAR_SPAN, record) {
            continue;
        }
        if let Some(cached) = known.get(&line) {
            out.insert(line, cached.clone());
            continue;
        }
        let read = read_line(memory, record, line)?;
        if let Some(read) = read {
            out.insert(line, read);
        }
    }
    Ok(out)
}

/// The line or track whose record is at `record`, or None where the record is not its.
pub fn read_line(memory: &dyn Memory, record: i64, line: i64) -> Result<Option<Line>> {
    let head = memory.read(record, LINE_HEAD)?;
    if u32(&head, 0)? != line || !sized(&head)? {
        return Ok(None);
    }
    let track = !is_line(&head)?;
    let file = u32(&head, if track { TRACK_FILE } else { LINE_FILE })?;
    if !within(LINEAR_SPAN, file) {
        return Ok(None);
    }
    let walls = file + u32(&head, if track { TRACK_WALLS } else { LINE_WALLS })?;
    Ok(Some(self::line(
        &head,
        &memory.read(walls, cells(&head)? * 4)?,
        track,
    )?))
}

/// Every line and track a chunk's street section holds, as the ROM keeps them, and where
/// the world has its points and joints; nothing from a section that is not one, and a chunk
/// with no streets has a section of none.
pub fn section_streets(section: &[u8]) -> Result<Network> {
    if (section.len() as i64) < SECTION_TABLES + 4 * (JOINTS + 1) {
        return Ok(Network::default());
    }
    let mut counts = Vec::new();
    let mut tables = Vec::new();
    for kind in 0..=CAMERAS {
        counts.push(u16(section, SECTION_COUNTS + 2 * kind)?);
    }
    for kind in 0..=JOINTS {
        tables.push(u32(section, SECTION_TABLES + 4 * kind)?);
    }
    let chunk = u32(section, 0)?;
    let middle = (
        ((chunk >> 8) & 0xFF) as f64 * CHUNK_UNITS as f64 + CHUNK_UNITS as f64 / 2.0,
        (chunk & 0xFF) as f64 * CHUNK_UNITS as f64 + CHUNK_UNITS as f64 / 2.0,
    );
    let mut lines: IndexMap<i64, Line> = IndexMap::new();
    let mut places: IndexMap<i64, (f64, f64)> = IndexMap::new();
    for kind in 0..=JOINTS {
        let (count, table) = (counts[kind as usize], tables[kind as usize]);
        if table + 4 * count > section.len() as i64 {
            continue;
        }
        for i in 0..count {
            let record = u32(section, table + 4 * i)?;
            if kind == LINES || kind == TRACKS {
                let head = py::slice(section, record, record + LINE_HEAD);
                if (head.len() as i64) < LINE_HEAD || !sized(head)? {
                    continue;
                }
                let walls = u32(
                    head,
                    if kind == TRACKS {
                        TRACK_WALLS
                    } else {
                        LINE_WALLS
                    },
                )?;
                let raw = py::slice(section, walls, walls + cells(head)? * 4);
                if raw.len() as i64 == cells(head)? * 4 {
                    lines.insert(u32(head, 0)?, self::line(head, raw, kind == TRACKS)?);
                }
                continue;
            }
            let at = record
                + if kind == POINTS {
                    POINT_PLACE
                } else {
                    JOINT_PLACE
                };
            if at + 12 <= section.len() as i64 {
                places.insert(
                    u32(section, record)?,
                    (
                        middle.0 + f32(section, at)?,
                        middle.1 + f32(section, at + 8)?,
                    ),
                );
            }
        }
    }
    Ok(Network {
        lines,
        places,
        cameras: counts[CAMERAS as usize],
    })
}

fn sized(head: &[u8]) -> Result<bool> {
    let (along, across) = (u16(head, LINE_ALONG)?, u16(head, LINE_ACROSS)?);
    Ok(along != 0 && across != 0 && cells(head)? <= MAX_CELLS)
}

fn cells(head: &[u8]) -> Result<i64> {
    Ok((u16(head, LINE_ALONG)? + 1) * u16(head, LINE_ACROSS)?)
}

fn is_line(head: &[u8]) -> Result<bool> {
    Ok(u32(head, 0)? == (u32(head, LINE_START)? | LINE_BIT))
}

fn line(head: &[u8], walls: &[u8], track: bool) -> Result<Line> {
    let count = walls.len() / 4;
    if !walls.len().is_multiple_of(4) {
        return Err(Error::unpack(format!(
            "unpack requires a buffer of {} bytes",
            count * 4
        )));
    }
    let mut words = Vec::with_capacity(count);
    for i in 0..count {
        words.push(u32(walls, 4 * i as i64)?);
    }
    Ok(Line {
        id: u32(head, 0)?,
        start: u32(head, LINE_START)?,
        end: u32(head, LINE_END)?,
        along: u16(head, LINE_ALONG)?,
        across: u16(head, LINE_ACROSS)?,
        walls: words.into(),
        way: u32(head, LINE_WAY)?,
        track,
    })
}

/// A furniture or warp record placed on a rail: (line, a, x, cells a, cells x), the same
/// five fields in both kinds of record (spike 102), all five signed and measured by the line's
/// way (`Line.upright`, spike 107).
pub fn placed(record: &[u8]) -> Result<(i64, i64, i64, i64, i64)> {
    Ok((
        u32(record, 0x08)?,
        s16(record, 0x0C)?,
        s16(record, 0x0E)?,
        s16(record, 0x10)?,
        s16(record, 0x12)?,
    ))
}

/// The corridor's cells a record covers: `width` on from `a` and `height` on from `x`, a
/// negative size running back from where the record starts, each measured by the line's way:
/// the warps a player comes into zone 0x44 by sit at -4 on a line measured north, across the
/// street on the wall row before its first open one (spike 107).
pub fn cells_of(
    corridor: &Corridor,
    line: i64,
    a: i64,
    x: i64,
    width: i64,
    height: i64,
) -> Result<Vec<Cell>> {
    let mut out = Vec::new();
    for along in span(a, width) {
        for across in span(x, height) {
            if let Some(cell) = corridor.at(line, along as f64, across as f64)? {
                out.push(cell);
            }
        }
    }
    Ok(out)
}

/// The cells a record spans from `start`: `size` onwards, or back from it when `size` is
/// negative, and the one cell for a size of 0.
pub fn span(start: i64, size: i64) -> std::ops::Range<i64> {
    if size < 0 {
        return start + size + 1..start + 1;
    }
    start..start + size.max(1)
}

fn entry(memory: &dyn Memory, entry: i64) -> Result<Option<i64>> {
    let raw = memory.read(entry, ENTRY)?;
    let (owner, line, zero, record) = (u32(&raw, 0)?, u32(&raw, 4)?, u32(&raw, 8)?, u32(&raw, 12)?);
    if zero != 0 || !within(HEAP_SPAN, owner) || !within(LINEAR_SPAN, record) {
        return Ok(None);
    }
    Ok(if u32(&memory.read(record, 4)?, 0)? == line {
        Some(record)
    } else {
        None
    })
}

/// The first corner and size of the box holding these cells.
pub fn spread(cells: impl IntoIterator<Item = Cell>) -> Result<(Cell, (i64, i64))> {
    let cells: Vec<Cell> = cells.into_iter().collect();
    let xs: Vec<i64> = cells.iter().map(|c| c.0).collect();
    let zs: Vec<i64> = cells.iter().map(|c| c.1).collect();
    let (Some(&west), Some(&north), Some(&east), Some(&south)) = (
        xs.iter().min(),
        zs.iter().min(),
        xs.iter().max(),
        zs.iter().max(),
    ) else {
        return Err(Error::value("min() iterable argument is empty"));
    };
    Ok(((west, north), (east - west + 1, south - north + 1)))
}

/// Every line and track in a map's chunks and where its joints are, out of the ROM
/// (`archives.Streets`).
pub type Streets = Rc<dyn Fn(&[i64]) -> Result<Network>>;

/// A zone's furniture and warps out of the ROM, by zone (`Rails.records`).
pub type Records = IndexMap<i64, (Vec<Vec<u8>>, Vec<Vec<u8>>)>;

/// `a or b` for an address that is None or a number: 0 is as false as None.
pub fn keep(found: Option<i64>, held: Option<i64>) -> Option<i64> {
    if found.is_some_and(|address| address != 0) {
        found
    } else {
        held
    }
}

/// Lumiose's streets as the field walks them: the lines the game has loaded, every line of the
/// map's streets out of the ROM, and the corridor the player is in.
pub struct Rails {
    memory: Rc<dyn Memory>,
    /// Where there is no ROM to read, None, and only the lines the game has loaded are known,
    /// with no joint to turn at.
    streets: Option<Streets>,
    /// The furniture and warps of every zone with records on streets, out of the ROM: on a
    /// street the records of the other zones of the map are listed too, since South Boulevard
    /// is two zones and the one the player is not in had Café Soleil (spike 107).
    pub records: Records,
    /// The chunk files the lines of the map were last read for.
    chunk_files: RefCell<Vec<i64>>,
    /// Where the world has each point and joint of the map's streets (`Network`).
    places: RefCell<IndexMap<i64, (f64, f64)>>,
    /// The table of the rails loaded, found through the line the player was on.
    table: Slot<Option<i64>>,
    /// Every line read from it, by id, kept once read: a line's walls and the points it joins
    /// the others at never change, and the table misses a line it holds for a poll now and then
    /// as the game moves its entries about, which laid the corridor out afresh under a way being
    /// walked and lost the thing it led to (spike 105).
    lines: RefCell<IndexMap<i64, Line>>,
    /// The corridor the last poll found the player in, or None off the rails.
    pub corridor: RefCell<Option<Rc<Corridor>>>,
    /// The block holding the camera's heading, once a sweep has found it (`camera`).
    camera: Slot<Option<i64>>,
    /// The line the corridor is laid out from. Kept while it is loaded and joined to the
    /// player's, so the cells a way was found in stay where they were as the player walks from
    /// one line onto the next; laid afresh from the player's own line otherwise. Which way it
    /// runs does not matter: a way there is said from the way the player faces.
    anchor: Slot<Option<i64>>,
    /// The chunk files the streets' open cells were laid on the grid for, and the cells
    /// with their steps and keys (`lanes`).
    lanes: RefCell<(Vec<i64>, Rc<Lanes>)>,
    /// How many cameras the map's streets place (`Network.cameras`).
    cameras: Slot<i64>,
}

impl Rails {
    pub fn new(memory: Rc<dyn Memory>, streets: Option<Streets>, records: Records) -> Self {
        Rails {
            memory,
            streets,
            records,
            chunk_files: RefCell::new(Vec::new()),
            places: RefCell::new(IndexMap::new()),
            table: Slot::new(None),
            lines: RefCell::new(IndexMap::new()),
            corridor: RefCell::new(None),
            camera: Slot::new(None),
            anchor: Slot::new(None),
            lanes: RefCell::new((Vec::new(), Rc::new(IndexMap::new()))),
            cameras: Slot::new(0),
        }
    }

    /// Take in what a sweep found: the camera, and the table of the rails loaded, through the
    /// line the player's model `head` is on.
    pub fn track(&self, found: &Found, head: Option<&[u8]>) -> Result<()> {
        self.camera
            .set(keep(camera::find(found, &*self.memory)?, self.camera.get()));
        if let Some(head) = head
            && mmodel::on_rail(head)?
        {
            let line = mmodel::rail(head)?.0;
            self.table.set(keep(
                find_table(&found.pages, &*self.memory, line)?,
                self.table.get(),
            ));
        }
        Ok(())
    }

    /// The quarters clockwise from north the pad's up walks the player off the rails: where
    /// the camera looks (`camera.looks`), `current` while no camera has been found.
    pub fn turn(&self, current: i64) -> Result<i64> {
        let looks = match self.camera.get() {
            None => None,
            Some(block) => camera::looks(&*self.memory, block)?,
        };
        match looks {
            None => Ok(current),
            Some(looks) => quarter(looks, Some(current)),
        }
    }

    /// The heading of a camera behind a player off the rails, as in Glittering Cave, in
    /// degrees clockwise from north; None with no camera found or none behind (spike 239). Up
    /// walks the world's (sin a, cos a) of its angle, as `quarter` has it.
    pub fn behind(&self) -> Result<Option<f64>> {
        let yaw = match self.camera.get() {
            None => None,
            Some(block) => camera::behind(&*self.memory, block)?,
        };
        Ok(yaw.map(|yaw| py::modulo_f(180.0 - yaw, 360.0)))
    }

    /// Where the player's model `head` is in the corridor of the streets joined to its line, and
    /// the way up walks them, or None while the line is not known. `files` are the map's chunk
    /// files, as the field map lists them.
    pub fn poll(&self, head: &[u8], files: &[i64]) -> Result<Option<(Cell, Option<f64>)>> {
        *self.corridor.borrow_mut() = None;
        let (line, a, x) = mmodel::rail(head)?;
        let corridor = self.read_corridor(line, files)?;
        if let Some(corridor) = &corridor {
            self.anchor.set(Some(corridor.laid[0].line.id));
        }
        let at = match &corridor {
            None => None,
            Some(corridor) => corridor.at(line, a, x)?,
        };
        let (Some(corridor), Some(at)) = (corridor, at) else {
            return Ok(None);
        };
        *self.corridor.borrow_mut() = Some(corridor.clone());
        Ok(Some((at, self.bearing(&corridor, line, head)?)))
    }

    /// Where up walks the player in the corridor: the camera's heading, turned from the world
    /// into the line's frame by the way the line runs where they stand; the way they face while
    /// no camera has been found, which is the same until left or right is tapped or down pressed.
    fn bearing(&self, corridor: &Corridor, line: i64, head: &[u8]) -> Result<Option<f64>> {
        let Some(on) = corridor.line(line) else {
            return Ok(None);
        };
        let world = match self.camera.get() {
            None => None,
            Some(block) => camera::heading(&*self.memory, block)?,
        };
        let (across, along) = match world {
            None => {
                let (x, a) = mmodel::rail_facing(head)?;
                let (along, across) = on.direction(a, x);
                (across, along)
            }
            Some(world) => {
                let toward = (world - mmodel::rail_way(head)?).to_radians();
                let (across, along) = (toward.sin(), toward.cos());
                if on.track {
                    (-across, -along)
                } else {
                    (across, along)
                }
            }
        };
        Ok(corridor.bearing(line, across, along))
    }

    /// The streets joined to `line`, laid out flat, or None while the line is not known: every
    /// line and track in the map's chunks where the ROM can be read, and the lines the game has
    /// loaded.
    fn read_corridor(&self, line: i64, files: &[i64]) -> Result<Option<Rc<Corridor>>> {
        self.read_streets(files)?;
        if let Some(table) = self.table.get() {
            let read = read_table(&*self.memory, table, &self.lines.borrow())?;
            self.lines.borrow_mut().extend(read);
        }
        if !self.lines.borrow().contains_key(&line) {
            return Ok(None);
        }
        if let Some(anchor) = self.anchor.get() {
            let kept = lay_out(&self.lines.borrow(), anchor, Some(&self.places.borrow()))?;
            if let Some(kept) = kept
                && kept.holds(line)
            {
                return Ok(Some(Rc::new(kept)));
            }
        }
        Ok(lay_out(&self.lines.borrow(), line, Some(&self.places.borrow()))?.map(Rc::new))
    }

    /// Where a record placed on a rail is on the grid, as the cells it covers, for a
    /// player walking the cells beside the streets: Route 7's way onto Route 6 is on a short
    /// street of its own, which the player walks onto from the grid, and no exit led to Route 6
    /// until they had (x_route-7and6.ctm, 2026-10-19). It is placed from where the world has its
    /// line's two ends, its offsets rising to the line's left as in the corridor; None for a
    /// record on the grid, or on a line whose ends the ROM does not place. `files` are the map's
    /// chunk files, as the field map lists them.
    pub fn on_grid(&self, record: &[u8], files: &[i64]) -> Result<Option<Vec<Cell>>> {
        if u16(record, zonedata::PLACEMENT)? != zonedata::ON_RAIL {
            return Ok(None);
        }
        self.read_streets(files)?;
        let (number, a, x, width, height) = placed(record)?;
        let line = self.lines.borrow().get(&number).cloned();
        let world = line.as_ref().and_then(|line| self.in_world(line));
        let (Some(line), Some(world)) = (line, world) else {
            return Ok(None);
        };
        // Python's is a set of tuples, visited in its own order where none of the cells round
        // them is the warp's; in the order the record spans them here.
        let mut wanted: IndexSet<(i64, i64)> = IndexSet::new();
        for along in span(a, width) {
            for across in span(x, height) {
                let (row, offset) = line.upright(along as f64, across as f64);
                wanted.insert((py::round(row)?, py::round(offset)?));
            }
        }
        // Each grid cell is the row and offset its middle is nearest, as `lanes` lays them, so
        // none of the warp's is a cell of the street the player stands in.
        let mut around: Vec<Cell> = Vec::new();
        for &(row, offset) in &wanted {
            around.push(world.cell(row as f64, offset as f64)?);
        }
        let r#box = Box::around(around.iter().copied(), 1)?;
        let mut out: Vec<Cell> = Vec::new();
        for cz in r#box.north..=r#box.south {
            for cx in r#box.west..=r#box.east {
                if wanted.contains(&world.place((cx, cz))?) {
                    out.push((cx, cz));
                }
            }
        }
        Ok(Some(if out.is_empty() { around } else { out }))
    }

    /// Whether a player on `line` walks it under a camera of the map's own that stays put, as
    /// on Route 7's and Parfum Palace's streets, rather than one that follows them as Lumiose's
    /// does (`Network.cameras`): the keys walk the player the street's way nearest the one
    /// pressed on the screen, so the street is ground of the grid's, walked by the compass. Not
    /// a line whose two ends the world has in one place, which the grid cannot lay out. `files`
    /// are the map's chunk files, as the field map lists them.
    pub fn fixed(&self, files: &[i64], line: i64) -> Result<bool> {
        self.read_streets(files)?;
        let on = self.lines.borrow().get(&line).cloned();
        Ok(self.cameras.get() > 0 && on.is_some_and(|on| self.in_world(&on).is_some()))
    }

    /// The cells of the map's streets the player can stand in, laid on the grid where the
    /// world has them, each with the steps the street's ways let the player take from it, the
    /// nearest of the eight to along the street and across it, and the key that takes each:
    /// up and down along a line measured north or south and left and right across it, each the
    /// way nearer its own, and the other way round on a line measured east or west. The chunks
    /// under a street mark every cell a wall, so from the grid beside Route 7's street onto
    /// Route 6 P found nothing at all to walk to, "no path" (2026-10-19); and up walks Route 7's
    /// street north-west and Parfum Palace's north-east, left across Route 7's south-west, never
    /// straight north or west, and P said "7 north-west" for up (the tapes of x_route-6-2.ctm
    /// and x_route-6-3.ctm, 2026-10-20). A cell is open where the cell of a line it is in, row
    /// and offset, is.
    pub fn lanes(&self, files: &[i64]) -> Result<Rc<Lanes>> {
        self.read_streets(files)?;
        if self.lanes.borrow().0 != files {
            let mut lanes: Lanes = IndexMap::new();
            for line in self.lines.borrow().values() {
                let Some(world) = self.in_world(line) else {
                    continue;
                };
                let (ax, az) = world.ahead;
                let (along, across) = if line.way == WAY_NORTH || line.way == WAY_SOUTH {
                    (&NORTH_KEYS, &EAST_KEYS)
                } else {
                    (&EAST_KEYS, &NORTH_KEYS)
                };
                let mut ways: IndexMap<Step, &'static str> = IndexMap::new();
                for ((dx, dz), keys) in [
                    ((ax, az), along),
                    ((-ax, -az), along),
                    ((az, -ax), across),
                    ((-az, ax), across),
                ] {
                    ways.insert(nearest_step(dx, dz)?, key(dx, dz, keys));
                }
                let mut corners: Vec<Cell> = Vec::new();
                for row in [0, line.along] {
                    for offset in [-line.across, line.across] {
                        corners.push(world.cell(row as f64, offset as f64)?);
                    }
                }
                let r#box = Box::around(corners, 0)?;
                for cx in r#box.west..=r#box.east {
                    for cz in r#box.north..=r#box.south {
                        let (row, offset) = world.place((cx, cz))?;
                        if line.open(row, offset + py::floordiv(line.across, 2)) {
                            lanes.entry((cx, cz)).or_default().extend(ways.clone());
                        }
                    }
                }
            }
            *self.lanes.borrow_mut() = (files.to_vec(), Rc::new(lanes));
        }
        Ok(self.lanes.borrow().1.clone())
    }

    /// Where the world has `line`, or None where the ROM does not place both its ends.
    fn in_world(&self, line: &Line) -> Option<World> {
        let places = self.places.borrow();
        let (Some(&start), Some(&end)) = (places.get(&line.start), places.get(&line.end)) else {
            return None;
        };
        if start == end {
            return None;
        }
        let length = py::hypot(start.0 - end.0, start.1 - end.1);
        Some(World {
            start,
            ahead: ((end.0 - start.0) / length, (end.1 - start.1) / length),
        })
    }

    /// Take in every line of the map's streets from the ROM, once a map: the game loads a line
    /// only with the chunks round the player, and a door on a line further on was nowhere, the
    /// Pokémon Lab's among them (spike 106). A map numbers its lines and points afresh, the same
    /// id in two maps two different things (spike 107), so another map's are let go.
    fn read_streets(&self, files: &[i64]) -> Result<()> {
        if files.is_empty() || files == self.chunk_files.borrow().as_slice() {
            return Ok(());
        }
        *self.chunk_files.borrow_mut() = files.to_vec();
        *self.lines.borrow_mut() = IndexMap::new();
        *self.places.borrow_mut() = IndexMap::new();
        self.anchor.set(None);
        self.cameras.set(0);
        if let Some(streets) = &self.streets {
            let wanted: Vec<i64> = files.iter().copied().filter(|&file| file != 0).collect();
            let network = streets(&wanted)?;
            self.lines.borrow_mut().extend(network.lines);
            self.places.borrow_mut().extend(network.places);
            self.cameras.set(network.cameras);
        }
        Ok(())
    }
}

/// Where a record placed on a rail is in the corridor, as its first corner and size, or None for
/// one on the grid or on a line the corridor does not hold.
pub fn spot(corridor: &Corridor, record: &[u8]) -> Result<Option<(Cell, (i64, i64))>> {
    if !zonedata::on_rail(record)? {
        return Ok(None);
    }
    let (line, a, x, width, height) = placed(record)?;
    let cells = cells_of(corridor, line, a, x, width, height)?;
    if cells.is_empty() {
        Ok(None)
    } else {
        Ok(Some(spread(cells)?))
    }
}
