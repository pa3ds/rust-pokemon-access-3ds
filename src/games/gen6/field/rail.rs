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
//! the corridor gives as a bearing of its own (`bearing`). On a **folded** map, Anistar City Gym's,
//! the camera stays looking north and the corridor is laid by the keys, its ways said by the
//! compass (`Network.folded`, spike A).

use std::cell::{Cell as Slot, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{f32, s16, u16, u32};
use crate::core::discovery::{ALLOCATION, Found, HEAP_SPAN, LINEAR_SPAN, within};
use crate::core::memory::Memory;
use crate::core::pathfind::{Box, Cell};
use crate::games::common::field::chunk::CLIMB;
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
/// A point's f32 x, y, z from its street's middle to its edge, square to the street there: 27
/// units on the Tower of Mastery's streets, three cells across, and along every street of X that is
/// as long as the line between its ends, square to that line (spike 248).
pub const POINT_SIDE: i64 = 0x64;
pub const JOINT_PLACE: i64 = 0x28;
/// How far apart a street's ends stand for it to be a ramp's: the Tower of Mastery's landings'
/// points stand level to the unit, at 180, 361, 540 and 720 (spike 248).
pub const SLOPE: f64 = 1.0;
/// Places a cell along a street's curve that it is laid from, each with its row's cells across.
pub const SAMPLES: i64 = 4;
/// World units a map chunk is across: 40 cells of 18.
pub const CHUNK_UNITS: i64 = 720;
/// Cells left between two parts of a folded map's streets in its corridor (`lay_keyed`).
pub const APART: i64 = 8;
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
    /// How high the world has each point and joint: the Tower of Mastery's ramp climbs from 2 to
    /// 720 by its points, and nothing else in the chunk says how high it is (spike 248).
    pub heights: IndexMap<i64, f64>,
    /// Each point's side as (x, z) (`POINT_SIDE`), which says the way its streets leave it.
    pub sides: IndexMap<i64, (f64, f64)>,
}

impl Network {
    /// Whether its streets place cameras and the world has both ends of one of them in one
    /// spot, a street that climbs straight up: Anistar City Gym's, matrix 68, alone of X's ten
    /// maps with streets, 17 of its 153 streets, as line 0x1019239 from (729, 1089) at 394 to (729,
    /// 1089) at 303, 5 cells along. Its camera stays looking north, and with each line laid
    /// along its own way and each track against it, every up press over x_gym_7 walked the
    /// player a cell north and every left a cell west, across 8 lines and tracks. So the whole
    /// map is walked as one corridor laid by the keys (`lay_keyed`), never on the grid, where a
    /// street climbs 4 rows over 1 world unit and P said "up the ramp 12 north" for a way 3
    /// west, 8 north (spike A).
    pub fn folded(&self) -> bool {
        // Python's `==` of one tuple with itself holds by identity, a NaN in it too: a line
        // starting and ending at one point is at one spot.
        self.cameras > 0
            && self.lines.values().any(|line| {
                self.places.get(&line.start).is_some_and(|start| {
                    line.start == line.end || self.places.get(&line.end) == Some(start)
                })
            })
    }
}

/// A place on a street as the game gives one: the line's id, and the cells along it and across
/// it it measures (`mmodel.rail`).
pub type Place = (i64, f64, f64);

/// One teleport of a Gym's module: the place on a street that sends the player, and the place
/// it puts them (`maps.rail_teleports`, spike A).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Teleport {
    pub start: Place,
    pub lands: Place,
}

/// One cell's step in the corridor, a way of the compass: north is (0, -1).
pub type Step = (i64, i64);
pub const NORTH: Step = (0, -1);
/// The step a line's rows rise by under a camera looking north, by the way it is measured.
pub const WAY_STEPS: [(i64, Step); 4] = [
    (WAY_NORTH, NORTH),
    (WAY_SOUTH, (0, 1)),
    (WAY_WEST, (-1, 0)),
    (WAY_EAST, (1, 0)),
];

/// The step `line`'s rows rise by on a folded map (`Network.folded`): along its way, and
/// against it on a track, which the game measures from its joint (spike A).
fn keyed(line: &Line) -> Step {
    let step = WAY_STEPS
        .iter()
        .find(|(way, _)| *way == line.way)
        .map_or((0, 1), |&(_, step)| step);
    if line.track { turned(step, 2) } else { step }
}

/// The step a quarter turn left of `step`: west of north.
fn left(step: Step) -> Step {
    (step.1, -step.0)
}

pub const NORTH_KEYS: [(&str, Step); 2] = [("north", (0, -1)), ("south", (0, 1))];
/// The pad's two pairs of keys, each with the way on the screen it points.
pub const EAST_KEYS: [(&str, Step); 2] = [("east", (1, 0)), ("west", (-1, 0))];

/// Which of a pair of keys points nearer the way (dx, dz).
fn key(dx: f64, dz: f64, keys: &[(&'static str, Step)]) -> &'static str {
    // `max` keeps the first of two that tie.
    let toward = |step: Step| dx * step.0 as f64 + dz * step.1 as f64;
    let mut best = keys[0];
    for &one in &keys[1..] {
        if toward(one.1) > toward(best.1) {
            best = one;
        }
    }
    best.0
}

/// `NORTH_KEYS + EAST_KEYS`.
const COMPASS_KEYS: [(&str, Step); 4] = [NORTH_KEYS[0], NORTH_KEYS[1], EAST_KEYS[0], EAST_KEYS[1]];

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
    /// Which part of a folded map's streets it is in, the streets its points join it to, by the
    /// lowest line id among them (`lay_keyed`); 0 elsewhere.
    pub part: i64,
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
    /// Where the world has a row and an offset of the line, in world units.
    fn at(&self, row: f64, offset: f64) -> (f64, f64) {
        let ((ax, az), (sx, sz)) = (self.ahead, self.start);
        let cell = zonedata::CELL as f64;
        (
            sx + (row * ax + offset * az) * cell,
            sz + (row * az - offset * ax) * cell,
        )
    }

    /// The grid's cell a row and an offset of the line are in.
    fn cell(&self, row: f64, offset: f64) -> Result<Cell> {
        let cell = zonedata::CELL as f64;
        let (x, z) = self.at(row, offset);
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
/// the world (`lay_out`, spike 107). On a folded map each line runs the way the keys walk it,
/// and every part of its streets is laid, apart (`lay_keyed`).
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

    /// Which part of a folded map's streets a cell is in (`Laid.part`), or None where no line
    /// holds it.
    pub fn part(&self, cell: Cell) -> Option<i64> {
        for laid in &self.laid {
            let (row, offset) = laid.place(cell);
            if 0 <= row
                && row <= laid.line.along
                && offset.abs() <= py::floordiv(laid.line.across, 2)
            {
                return Some(laid.part);
            }
        }
        None
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
/// out rather than laid across another. `keyed`, on a folded map (`Network.folded`), lays each
/// by the keys instead, along its own way and a track against it (`keyed`).
pub fn lay_out(
    lines: &IndexMap<i64, Line>,
    through: i64,
    places: Option<&IndexMap<i64, (f64, f64)>>,
    keyed: bool,
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
            ahead: if keyed { self::keyed(first) } else { NORTH },
            part: 0,
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
                if keyed {
                    way = if other.start == point {
                        self::keyed(other)
                    } else {
                        turned(self::keyed(other), 2)
                    };
                } else if met.len() > 2 {
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

/// A folded map's streets as one corridor laid by the keys (`lay_out`'s `keyed`): each part
/// its points join, from its lowest line id, laid `APART` cells east of the parts before it, so
/// no two share a cell. Two points the world has at one spot and height are one: the lobby's
/// street and the landing's track meet so at (729, 1089) and 394 in Anistar City Gym. The Gym
/// is then six parts, joined by its teleporters alone (`Teleport`). Laid where the screen
/// shows each, 126 cells would be two parts' (spike A). A street's dead end a cell from
/// another's point at its height is no way onto it: joined so, P said "1 west, 14 south" off
/// the first teleporter's landing, and the keys met a wall (Khai, 2026-10-30).
///
/// A track's walls run from its joint, as the game measures places on it (`Line.direction`),
/// and are turned round to run from its start as a line's do. Read from their starts, each of
/// the Gym's 20 tracks of two cells or more that widens toward one end widened away from its
/// joint and closed at it, and 22 cells met others only at a corner; turned, none does, and a
/// walk from the landing reaches all seven people. Lumiose's tracks are read as they were:
/// nothing there has said which way theirs run.
pub fn lay_keyed(network: &Network) -> Result<Corridor> {
    // Python's dict keyed by (x, z, height): a float key matches by `==`, so a NaN never does.
    let mut first: Vec<((f64, f64, i64), i64)> = Vec::new();
    let mut same: HashMap<i64, i64> = HashMap::new();
    let mut points: Vec<(i64, (f64, f64))> = network
        .places
        .iter()
        .map(|(&point, &place)| (point, place))
        .collect();
    points.sort_by_key(|&(point, _)| point);
    for (point, place) in points {
        if let Some(&height) = network.heights.get(&point) {
            let key = (place.0, place.1, py::round(height)?);
            let one = match first.iter().find(|(held, _)| *held == key) {
                Some(&(_, one)) => one,
                None => {
                    first.push((key, point));
                    point
                }
            };
            same.insert(point, one);
        }
    }
    let mut left: IndexMap<i64, Line> = network
        .lines
        .iter()
        .map(|(&number, line)| {
            (
                number,
                Line {
                    start: same.get(&line.start).copied().unwrap_or(line.start),
                    end: same.get(&line.end).copied().unwrap_or(line.end),
                    walls: if line.track {
                        line.walls.iter().rev().copied().collect()
                    } else {
                        line.walls.clone()
                    },
                    ..line.clone()
                },
            )
        })
        .collect();
    let mut laid: Vec<Laid> = Vec::new();
    let mut east = 0;
    while let Some(&part) = left.keys().min() {
        let corridor =
            lay_out(&left, part, None, true)?.expect("a part is laid from a line it holds");
        let r#box = corridor.r#box()?;
        let shift = east - r#box.west;
        laid.extend(corridor.laid.iter().map(|one| Laid {
            origin: (one.origin.0 + shift, one.origin.1),
            part,
            ..one.clone()
        }));
        east = r#box.east + shift + APART;
        for one in &corridor.laid {
            left.shift_remove(&one.line.id);
        }
    }
    Ok(Corridor { laid })
}

/// `line` laid so its end at `point` is the cell `at` and it runs on from there `way`.
fn leaving(line: &Line, point: i64, at: Cell, way: Step) -> Laid {
    if line.start == point {
        return Laid {
            line: line.clone(),
            origin: at,
            ahead: way,
            part: 0,
        };
    }
    Laid {
        line: line.clone(),
        origin: (at.0 + line.along * way.0, at.1 + line.along * way.1),
        ahead: turned(way, 2),
        part: 0,
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
    let mut heights: IndexMap<i64, f64> = IndexMap::new();
    let mut sides: IndexMap<i64, (f64, f64)> = IndexMap::new();
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
                heights.insert(u32(section, record)?, f32(section, at + 4)?);
            }
            let side = record + POINT_SIDE;
            if kind == POINTS && side + 12 <= section.len() as i64 {
                sides.insert(
                    u32(section, record)?,
                    (f32(section, side)?, f32(section, side + 8)?),
                );
            }
        }
    }
    Ok(Network {
        lines,
        places,
        cameras: counts[CAMERAS as usize],
        heights,
        sides,
    })
}

/// The streets as floors of their own, where a **ramp** is among them, a street standing off
/// the ground: a point of it more than `CLIMB` from every height `ground` has at its cell. Each
/// floor is each cell of its streets' lanes with its height there (`_laid`). The Tower of
/// Mastery's ramp winds twice round the ring of floor under it, from 2 to 720, each lap over the
/// one before in the grid and 360 above it, and its chunk's floors are the ground at 0 and the
/// landings alone, so P found no way off the ground: "then can't continue" to every stair
/// (x_tower_of_mastery, spike 248). Coumarine's and Anistar's streets run over cells their chunks
/// wall, as Route 7's do, and up to 770 over them. Every street is a floor there, its cells open
/// on it alone: a cell a street opens on no floor is any floor, and a way through one stepped
/// from one lap onto the next. None where every street lies on the ground, as Route 7's, Parfum
/// Palace's and Lumiose's do, walked as the grid's ground (`Rails.lanes`). Streets are packed
/// into as few floors as keep apart every two cells, the same or side by side, more than `CLIMB`
/// from each other's height: a lap and the lap over it are two. A ramp's streets, whose ends
/// stand more than `SLOPE` apart, are packed apart from level ones, and each floor says which it
/// is: a run up a ramp is said as one leg (`guide.RAMP_UP`).
pub fn street_floors(
    network: &Network,
    ground: &dyn Fn(Cell) -> Result<Vec<f64>>,
) -> Result<Vec<(IndexMap<Cell, f64>, bool)>> {
    // The points alone: laying Lumiose's 294 streets, some 67 cells across, takes seconds.
    let off = |point: i64, height: f64| -> Result<bool> {
        let under = match network.places.get(&point) {
            None => Vec::new(),
            Some(&(x, z)) => {
                let cell = zonedata::CELL as f64;
                ground((py::floor(x / cell)?, py::floor(z / cell)?))?
            }
        };
        Ok(!under.is_empty() && under.iter().all(|&one| (height - one).abs() > CLIMB))
    };

    let mut any = false;
    for (&point, &height) in &network.heights {
        if off(point, height)? {
            any = true;
            break;
        }
    }
    if !any {
        return Ok(Vec::new());
    }
    let mut floors: Vec<(IndexMap<Cell, f64>, bool)> = Vec::new();
    for line in network.lines.values() {
        let lane = laid(line, network)?;
        if lane.is_empty() {
            continue;
        }
        // Both ends have heights, or `laid` laid nothing.
        let sloped = (network.heights[&line.end] - network.heights[&line.start]).abs() > SLOPE;

        let fits = |floor: &IndexMap<Cell, f64>| {
            lane.iter().all(|(&(x, z), &height)| {
                [(x, z), (x + 1, z), (x - 1, z), (x, z + 1), (x, z - 1)]
                    .iter()
                    .filter_map(|near| floor.get(near))
                    .all(|&there| (there - height).abs() <= CLIMB)
            })
        };
        let floor = match floors
            .iter()
            .position(|(cells, ramp)| *ramp == sloped && fits(cells))
        {
            Some(at) => &mut floors[at].0,
            None => {
                floors.push((IndexMap::new(), sloped));
                &mut floors.last_mut().expect("the floor just pushed").0
            }
        };
        for (&cell, &height) in &lane {
            floor.entry(cell).or_insert(height);
        }
    }
    Ok(floors)
}

/// Each grid cell of `line` the player can stand in, with its height: the line laid as the
/// cubic from its start point to its end point that leaves each square to its side there, as
/// fast as the line between them is long, its height rising evenly along it, and a cell open
/// where the row and column of the line's walls at its middle are. A straight street's sides
/// are square to the line between its ends, and the Tower of Mastery's streets, arcs round its
/// statue, bulge two cells out from that line, where the player walked: laid so, all 177 cells
/// x_tower_of_mastery stood on lie on a floor at their height (spike 248). None where the world
/// does not place both its ends or say how high they are.
// `t ** 3` and `t ** 2` are `powf`, the C library's `pow` as Python's are, never `powi`.
fn laid(line: &Line, network: &Network) -> Result<IndexMap<Cell, f64>> {
    let (start, end) = (
        network.places.get(&line.start),
        network.places.get(&line.end),
    );
    let (low, high) = (
        network.heights.get(&line.start),
        network.heights.get(&line.end),
    );
    let (Some(&start), Some(&end), Some(&low), Some(&high)) = (start, end, low, high) else {
        return Ok(IndexMap::new());
    };
    if start == end {
        return Ok(IndexMap::new());
    }
    let ((x0, z0), (x1, z1)) = (start, end);
    // `math.dist`, which is `hypot` of the differences.
    let chord = py::hypot(x0 - x1, z0 - z1);
    let ahead = ((x1 - x0) / chord, (z1 - z0) / chord);

    let leaving = |point: i64| -> (f64, f64) {
        let (sx, sz) = network.sides.get(&point).copied().unwrap_or((0.0, 0.0));
        if py::hypot(sx, sz) == 0.0 {
            return ahead;
        }
        let way = if sz * ahead.0 - sx * ahead.1 >= 0.0 {
            (sz, -sx)
        } else {
            (-sz, sx)
        };
        (way.0 / py::hypot(sx, sz), way.1 / py::hypot(sx, sz))
    };

    let ((a0, b0), (a1, b1)) = (leaving(line.start), leaving(line.end));
    let cell_units = zonedata::CELL as f64;
    let mut out: IndexMap<Cell, f64> = IndexMap::new();
    let mut nearest: HashMap<Cell, f64> = HashMap::new();
    let steps = SAMPLES * line.along;
    for k in 0..=steps {
        let t = k as f64 / steps as f64;
        let (h00, h10, h01, h11) = (
            2.0 * t.powf(3.0) - 3.0 * t.powf(2.0) + 1.0,
            t.powf(3.0) - 2.0 * t.powf(2.0) + t,
            -2.0 * t.powf(3.0) + 3.0 * t.powf(2.0),
            t.powf(3.0) - t.powf(2.0),
        );
        let (d00, d10, d01, d11) = (
            6.0 * t.powf(2.0) - 6.0 * t,
            3.0 * t.powf(2.0) - 4.0 * t + 1.0,
            -6.0 * t.powf(2.0) + 6.0 * t,
            3.0 * t.powf(2.0) - 2.0 * t,
        );
        let x = h00 * x0 + h10 * chord * a0 + h01 * x1 + h11 * chord * a1;
        let z = h00 * z0 + h10 * chord * b0 + h01 * z1 + h11 * chord * b1;
        let dx = d00 * x0 + d10 * chord * a0 + d01 * x1 + d11 * chord * a1;
        let dz = d00 * z0 + d10 * chord * b0 + d01 * z1 + d11 * chord * b1;
        let speed = py::hypot(dx, dz);
        let speed = if speed == 0.0 { 1.0 } else { speed };
        let row = py::round(t * line.along as f64)?;
        // Out to a quarter of a cell inside the street's edge, half its width from its middle.
        for fourth in -2 * line.across + 1..2 * line.across {
            let offset = fourth as f64 / 4.0;
            let column = 0.max((line.across - 1).min(py::round(offset)? + line.across / 2));
            if !line.open(row, column) {
                continue;
            }
            // Offsets rise to the line's left, as `_World.cell` lays them.
            let (px, pz) = (
                x + offset * dz / speed * cell_units,
                z - offset * dx / speed * cell_units,
            );
            let cell = (py::floor(px / cell_units)?, py::floor(pz / cell_units)?);
            // A cell is as high as the street where it passes nearest the cell's middle.
            let off = py::hypot(
                px - (cell.0 as f64 + 0.5) * cell_units,
                pz - (cell.1 as f64 + 0.5) * cell_units,
            );
            if off < nearest.get(&cell).copied().unwrap_or(f64::INFINITY) {
                nearest.insert(cell, off);
                out.insert(cell, low + (high - low) * t);
            }
        }
    }
    Ok(out)
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
    /// Where the game's camera block keeps where it looks (`Game.camera`).
    shape: camera::Shape,
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
    /// How high the world has each of them (`Network.heights`).
    heights: RefCell<IndexMap<i64, f64>>,
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
    /// The chunk files the streets' open cells were laid on the grid for, whether by the
    /// compass and whether under a camera turned an odd number of quarters, and the cells with
    /// their steps and keys (`lanes`).
    lanes: RefCell<((Vec<i64>, bool, i64), Rc<Lanes>)>,
    /// How many cameras the map's streets place (`Network.cameras`).
    cameras: Slot<i64>,
    /// Those of the game's Gym modules, out of the ROM (`maps.rail_teleports`): a map's are
    /// those on its streets.
    pub teleports: Vec<Teleport>,
    /// Whether the map's streets are folded (`Network.folded`), walked as one corridor laid by
    /// the keys (`lay_keyed`).
    pub folded: Slot<bool>,
    /// A folded map's corridor, laid once for its streets as last read (`lay_keyed`).
    keyed: RefCell<Option<Rc<Corridor>>>,
}

impl Rails {
    /// `streets` reads every line and track in a map's chunks and where its joints are, out of
    /// the ROM (`archives.Streets`); None where there is no ROM to read, and only the lines the
    /// game has loaded are known, with no joint to turn at. `shape` is where the game's camera
    /// block keeps where it looks (`Game.camera`). `teleports` are those of the game's Gym
    /// modules, out of the ROM (`maps.rail_teleports`): a map's are those on its streets.
    pub fn new(
        memory: Rc<dyn Memory>,
        streets: Option<Streets>,
        records: Records,
        shape: camera::Shape,
        teleports: Vec<Teleport>,
    ) -> Self {
        Rails {
            memory,
            shape,
            streets,
            records,
            chunk_files: RefCell::new(Vec::new()),
            places: RefCell::new(IndexMap::new()),
            heights: RefCell::new(IndexMap::new()),
            table: Slot::new(None),
            lines: RefCell::new(IndexMap::new()),
            corridor: RefCell::new(None),
            camera: Slot::new(None),
            anchor: Slot::new(None),
            lanes: RefCell::new(((Vec::new(), false, 0), Rc::new(IndexMap::new()))),
            cameras: Slot::new(0),
            teleports,
            folded: Slot::new(false),
            keyed: RefCell::new(None),
        }
    }

    /// Take in what a sweep found: the camera, and the table of the rails loaded, through the
    /// line the player's model `head` is on.
    pub fn track(&self, found: &Found, head: Option<&[u8]>) -> Result<()> {
        self.camera.set(keep(
            camera::find(found, &*self.memory, &self.shape)?,
            self.camera.get(),
        ));
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
            Some(block) => camera::looks(&*self.memory, block, &self.shape)?,
        };
        match looks {
            None => Ok(current),
            Some(looks) => quarter(looks, Some(current)),
        }
    }

    /// The quarter of the world, clockwise from north, up walks a player on a rail or under
    /// a camera behind them: the camera's heading (`camera.heading`), `current` while no camera
    /// has been found or it holds no angle.
    pub fn facing(&self, current: Option<i64>) -> Result<Option<i64>> {
        let yaw = match self.camera.get() {
            None => None,
            Some(block) => camera::heading(&*self.memory, block, &self.shape)?,
        };
        match yaw {
            None => Ok(current),
            Some(yaw) => Ok(Some(quarter(yaw.to_radians(), current)?)),
        }
    }

    /// The heading of a camera behind a player off the rails, as in Glittering Cave, in
    /// degrees clockwise from north; None with no camera found or none behind (spike 239). Up
    /// walks the world's (sin a, cos a) of its angle, as `quarter` has it.
    pub fn behind(&self) -> Result<Option<f64>> {
        let yaw = match self.camera.get() {
            None => None,
            Some(block) => camera::behind(&*self.memory, block, &self.shape)?,
        };
        Ok(yaw.map(|yaw| py::modulo_f(180.0 - yaw, 360.0)))
    }

    /// Where the player's model `head` is in the corridor of the streets joined to its line, and
    /// the way up walks them, or None while the line is not known; no way on a folded map, whose
    /// corridor is the compass's (`Network.folded`). `files` are the map's chunk files, as the
    /// field map lists them.
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
        let way = if self.folded.get() {
            None
        } else {
            self.bearing(&corridor, line, head)?
        };
        Ok(Some((at, way)))
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
            Some(block) => camera::heading(&*self.memory, block, &self.shape)?,
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
        if let Some(keyed) = self.keyed.borrow().as_ref() {
            return Ok(keyed.holds(line).then(|| keyed.clone()));
        }
        if let Some(anchor) = self.anchor.get() {
            let kept = lay_out(
                &self.lines.borrow(),
                anchor,
                Some(&self.places.borrow()),
                false,
            )?;
            if let Some(kept) = kept
                && kept.holds(line)
            {
                return Ok(Some(Rc::new(kept)));
            }
        }
        Ok(lay_out(
            &self.lines.borrow(),
            line,
            Some(&self.places.borrow()),
            false,
        )?
        .map(Rc::new))
    }

    /// Each dead end of the corridor's streets, a point or joint no other line or track
    /// meets, as its id, the street's open cell in the corridor nearest it (`_open_toward`) and
    /// the grid's cell the world has the end in; none the world does not place, nor one no open
    /// cell leads to. Zone 0x43's map has 53, all but one the joint of one track (spike 107),
    /// and 29 stand by open ground on the grid: the plazas, where the street hands the player to
    /// the grid (spike 251).
    pub fn ends(&self, corridor: &Corridor) -> Result<Vec<(i64, Cell, Cell)>> {
        let lines = self.lines.borrow();
        let places = self.places.borrow();
        let mut meeting: HashMap<i64, Vec<&Line>> = HashMap::new();
        for line in lines.values() {
            // Python's `{line.start, line.end}`: one point where a line starts and ends at it.
            let mut points = vec![line.start];
            if line.end != line.start {
                points.push(line.end);
            }
            for point in points {
                meeting.entry(point).or_default().push(line);
            }
        }
        let mut out: Vec<(i64, Cell, Cell)> = Vec::new();
        for laid in &corridor.laid {
            for point in [laid.line.start, laid.line.end] {
                let place = places.get(&point).copied();
                let met = meeting
                    .get(&point)
                    .ok_or_else(|| Error::key(point.to_string()))?
                    .len();
                let Some(place) = place.filter(|_| met == 1) else {
                    continue;
                };
                if let Some(cell) = open_toward(corridor, &laid.line, point, &meeting)? {
                    let size = zonedata::CELL as f64;
                    out.push((
                        point,
                        cell,
                        (py::floor(place.0 / size)?, py::floor(place.1 / size)?),
                    ));
                }
            }
        }
        Ok(out)
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
        self.laid(placed(record)?, files)
    }

    /// The cells on the grid of a place on a rail, (line, a, x, cells a, cells x), as a
    /// record gives it (`placed`, `zonedata.trigger_placed`): `on_grid`'s.
    pub fn laid(
        &self,
        place: (i64, i64, i64, i64, i64),
        files: &[i64],
    ) -> Result<Option<Vec<Cell>>> {
        self.read_streets(files)?;
        let (number, a, x, width, height) = place;
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

    /// How high a record placed on a rail stands: as high as the world has its line's start
    /// point, as each of Coumarine Gym's vines is at its level (spike 250); None for a line whose
    /// start the ROM does not place. `files` are the map's chunk files, as the field map lists
    /// them.
    pub fn height(&self, record: &[u8], files: &[i64]) -> Result<Option<f64>> {
        self.line_height(placed(record)?.0, files)
    }

    /// How high the world has the start of line `number`, None where the ROM does not say.
    pub fn line_height(&self, number: i64, files: &[i64]) -> Result<Option<f64>> {
        self.read_streets(files)?;
        let line = self.lines.borrow().get(&number).cloned();
        Ok(line.and_then(|line| self.heights.borrow().get(&line.start).copied()))
    }

    /// How high a place on a street is, rising evenly along its line from its start's height to
    /// its end's, as the ramp's do (`laid`); None where the ROM does not say.
    pub fn place_height(&self, place: Place) -> Option<f64> {
        let lines = self.lines.borrow();
        let line = lines.get(&place.0)?;
        if line.along == 0 {
            return None;
        }
        let heights = self.heights.borrow();
        let (low, high) = (*heights.get(&line.start)?, *heights.get(&line.end)?);
        Some(low + (high - low) * line.upright(place.1, place.2).0 / line.along as f64)
    }

    /// Where a player stands to use a record placed on a rail just off its street, and the
    /// way of the compass they face it: on a track, the track's start, the end of the street it
    /// runs off, faced the way the street runs into it; on a line, one cell nearer the street,
    /// facing the record. Over x_gym_4_climbed every vine of Coumarine Gym was grabbed from such
    /// a spot and every ride landed on the other end's, to a tenth of a cell but one by 0.4:
    /// the swings' from each platform's end, (64.8, 74.2) to (71.2, 67.8), the climbs' from
    /// one cell nearer the street than the vine, (59.6, 73.4) under (59.6, 72.4) (spike 250).
    /// None for a record on the grid or a line the world does not place. `files` are the map's
    /// chunk files, as the field map lists them.
    pub fn spot(&self, record: &[u8], files: &[i64]) -> Result<Option<(Cell, &'static str)>> {
        if u16(record, zonedata::PLACEMENT)? != zonedata::ON_RAIL {
            return Ok(None);
        }
        self.read_streets(files)?;
        let (number, a, x, _, _) = placed(record)?;
        let line = self.lines.borrow().get(&number).cloned();
        let world = line.as_ref().and_then(|line| self.in_world(line));
        let (Some(line), Some(world)) = (line, world) else {
            return Ok(None);
        };
        if line.track {
            let places = self.places.borrow();
            // Python's `next(...)` over the streets the track runs off: the first one's other
            // end, which may be a place the world does not have.
            let into = self
                .lines
                .borrow()
                .values()
                .find(|other| {
                    !other.track && (line.start == other.start || line.start == other.end)
                })
                .and_then(|other| {
                    places
                        .get(&if other.end == line.start {
                            other.start
                        } else {
                            other.end
                        })
                        .copied()
                });
            let start = *places
                .get(&line.start)
                .ok_or_else(|| Error::key(line.start.to_string()))?;
            let cell = zonedata::CELL as f64;
            let spot = (
                py::round((start.0 - cell / 2.0) / cell)?,
                py::round((start.1 - cell / 2.0) / cell)?,
            );
            let ahead = match into {
                None => world.ahead,
                Some(into) => (start.0 - into.0, start.1 - into.1),
            };
            return Ok(Some((spot, key(ahead.0, ahead.1, &COMPASS_KEYS))));
        }
        let (row, offset) = line.upright(a as f64, x as f64);
        let (vx, vz) = world.at(row, offset);
        let (row, offset) = line.upright((a + 1) as f64, x as f64);
        let (sx, sz) = world.at(row, offset);
        Ok(Some((
            world.cell(row, offset)?,
            key(vx - sx, vz - sz, &COMPASS_KEYS),
        )))
    }

    /// Whether a player on `line` walks it under a camera of the map's own that stays put, as
    /// on Route 7's and Parfum Palace's streets, rather than one that follows them as Lumiose's
    /// does (`Network.cameras`): the keys walk the player the street's way nearest the one
    /// pressed on the screen, so the street is ground of the grid's, walked by the compass. Not
    /// a line whose two ends the world has in one place, which the grid cannot lay out, nor any
    /// line of a folded map, walked as a corridor by the keys (`Network.folded`). `files` are
    /// the map's chunk files, as the field map lists them.
    pub fn fixed(&self, files: &[i64], line: i64) -> Result<bool> {
        self.read_streets(files)?;
        let on = self.lines.borrow().get(&line).cloned();
        Ok(self.cameras.get() > 0
            && !self.folded.get()
            && on.is_some_and(|on| self.in_world(&on).is_some()))
    }

    /// The cells of the map's streets the player can stand in, laid on the grid where the
    /// world has them, each with the steps the street's ways let the player take from it, the
    /// nearest of the eight to along the street and across it, and the key that takes each:
    /// up and down along a line measured north or south and left and right across it, each the
    /// way nearer its own, and the other way round on a line measured east or west; with
    /// `compass`, where the streets are floors of their own, the way of the four nearest the
    /// street's own way along it and across it. The Tower of Mastery's streets are all measured
    /// east, the north-bound ones too (spike 248), and in Coumarine Gym the player held right
    /// alone along the lowest platform's arc, going south-east, east and north-east from (56.2,
    /// 74.2) to (64.3, 73.7), and left alone from (63.4, 73.2) south-west and west (spike 250).
    /// The chunks under a street mark every cell a wall, so from the grid beside Route 7's street
    /// onto Route 6 P found nothing at all to walk to, "no path" (2026-10-19); and up walks Route
    /// 7's street north-west and Parfum Palace's north-east, left across Route 7's south-west,
    /// never straight north or west, and P said "7 north-west" for up (the tapes of
    /// x_route-6-2.ctm and x_route-6-3.ctm, 2026-10-20). A cell is open where the cell of a line
    /// it is in, row and offset, is.
    ///
    /// The pairs are the screen's: under a camera looking east, as round Prism Tower, `turn` 1,
    /// the quarters up walks from north, a line measured east runs up and down the screen, and
    /// left walked the player north along one and down west across it (spike 251). Each way is
    /// named the world's, which `pathfind.way` says as the key.
    pub fn lanes(&self, files: &[i64], compass: bool, turn: i64) -> Result<Rc<Lanes>> {
        self.read_streets(files)?;
        let odd = py::modulo(turn, 2);
        let cached = {
            let lanes = self.lanes.borrow();
            lanes.0.0 == files && lanes.0.1 == compass && lanes.0.2 == odd
        };
        if !cached {
            let mut lanes: Lanes = IndexMap::new();
            for line in self.lines.borrow().values() {
                let Some(world) = self.in_world(line) else {
                    continue;
                };
                let (ax, az) = world.ahead;
                let upright = (line.way == WAY_NORTH || line.way == WAY_SOUTH) != (odd != 0);
                let (along, across): (&[(&str, Step)], &[(&str, Step)]) = if compass {
                    (&COMPASS_KEYS, &COMPASS_KEYS)
                } else if upright {
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
            *self.lanes.borrow_mut() = ((files.to_vec(), compass, odd), Rc::new(lanes));
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
        *self.heights.borrow_mut() = IndexMap::new();
        self.folded.set(false);
        *self.keyed.borrow_mut() = None;
        if let Some(streets) = &self.streets {
            let wanted: Vec<i64> = files.iter().copied().filter(|&file| file != 0).collect();
            let network = streets(&wanted)?;
            let folded = network.folded();
            let keyed = if folded {
                Some(Rc::new(lay_keyed(&network)?))
            } else {
                None
            };
            self.lines.borrow_mut().extend(network.lines);
            self.places.borrow_mut().extend(network.places);
            self.heights.borrow_mut().extend(network.heights);
            self.cameras.set(network.cameras);
            self.folded.set(folded);
            *self.keyed.borrow_mut() = keyed;
        }
        Ok(())
    }
}

/// Lines and tracks walked back from a dead end for the street's last open cell: off North
/// Boulevard the alley's open cells stop four lines and tracks short of the joint onto Magenta
/// Plaza, seven cells back, the rest of the way walled (spike 251).
pub const BACK_LINES: i64 = 8;

/// The open cell of the street ending at `point` on `line` nearest that end, nearest the
/// street's middle, in the corridor: walked back along it line by line, up to `BACK_LINES`, and
/// None where it branches or ends first. A street's last cells before a plaza are walls in its
/// table, where the player walks on the grid (spike 251).
fn open_toward(
    corridor: &Corridor,
    line: &Line,
    point: i64,
    meeting: &HashMap<i64, Vec<&Line>>,
) -> Result<Option<Cell>> {
    let (mut line, mut point) = (line, point);
    for _ in 0..BACK_LINES {
        let rows: Vec<i64> = if line.end == point {
            (0..=line.along).rev().collect()
        } else {
            (0..=line.along).collect()
        };
        let middle = py::floordiv(line.across, 2);
        // `sorted` is stable: of two columns as near the middle, the western first.
        let mut columns: Vec<i64> = (0..line.across).collect();
        columns.sort_by_key(|column| (column - middle).abs());
        for &row in &rows {
            for &column in &columns {
                if line.open(row, column) {
                    return corridor.cell(line.id, row as f64, (column - middle) as f64);
                }
            }
        }
        point = if line.end == point {
            line.start
        } else {
            line.end
        };
        let on: Vec<&Line> = meeting
            .get(&point)
            .map(|lines| {
                lines
                    .iter()
                    .copied()
                    .filter(|other| other.id != line.id)
                    .collect()
            })
            .unwrap_or_default();
        if on.len() != 1 {
            return Ok(None);
        }
        line = on[0];
    }
    Ok(None)
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
