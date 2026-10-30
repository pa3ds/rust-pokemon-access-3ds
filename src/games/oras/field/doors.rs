//! Fortree Gym's rotating doors: wooden arms about a pivot that turn a quarter as one is walked
//! into (spike 161).
//!
//! The Gym loads a module of its own, `DllUSGymFlight.cro`, whose
//! `field::FieldUniqueSequenceGymFlight` builds its seven doors as it starts, from constants in
//! its code, and keeps how each stands:
//!
//! ```text
//! +0x1c -> each door's object, seven of them: f32 x, y, z of its pivot at +0x130, in world
//!          units, a corner where four cells meet, 18 to a cell
//! +0x48 each door's record, 16 bytes: u32 its arms' pattern (`PATTERNS`), a word, i32 how many
//!          quarter turns anticlockwise it stands at, -1 while the player is at it, and i32 the
//!          turn it stood at then
//! ```
//!
//! A door is walked through by walking into an arm, which turns it a quarter the way the step goes
//! where every cell its arms sweep through is open. The game keeps nine invisible models drawn as
//! `STOPPER`, its invisible blocker, in the Gym's records, and sets a few of them in the cells
//! round the door the player is at to stand in for its arms; a door's arms are what they stand in
//! for, so they are not listed and nobody stands in them (`gen6.field`).

use std::cell::{Cell as Slot, RefCell};
use std::rc::Rc;
use std::sync::OnceLock;

use indexmap::IndexMap;

use crate::core::binary::{self, u32};
use crate::core::discovery::{Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::core::pathfind::{EAST, NORTH, SOUTH, WEST};
use crate::games::common::field::rotating::RotatingDoor;
use crate::games::gen6::field::features::{DoorReader, Tracked};
use crate::py::{self, Error, Result};

pub const CLASS: &str = "N5field28FieldUniqueSequenceGymFlightE";
/// Fortree Gym's zone.
pub const GYM: i64 = 0x1BA;
pub const DOORS: i64 = 0x1C;
pub const RECORDS: i64 = 0x48;
pub const COUNT: i64 = 7;

/// `struct.Struct("<IIii")`: a door's pattern, a word, its turn and the turn it stood at.
pub struct Record;

pub const RECORD: Record = Record;

impl Record {
    /// `RECORD.size`.
    pub fn size(&self) -> i64 {
        16
    }

    /// `RECORD.unpack_from(blob, offset)`.
    #[track_caller]
    pub fn unpack_from(&self, blob: &[u8], offset: i64) -> Result<[i64; 4]> {
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
            binary::u32(blob, offset)?,
            binary::u32(blob, offset + 4)?,
            binary::s32(blob, offset + 8)?,
            binary::s32(blob, offset + 12)?,
        ])
    }
}

pub const HEAD: i64 = RECORDS + COUNT * 16;
pub const PIVOT: i64 = 0x130;
pub const CELL: i64 = 18;

/// Each pattern's arms at no turn, a way out from the pivot and a length in cells, as the
/// module's table of which steps walk into an arm has them: the steps into each arm from both its
/// sides, from the cell beside the pivot for one a cell long and from both cells for one two long
/// (spike 161). Doors 0, 1 and 2 are the first, 3 and 4 the second, 5 the third and 6 the fourth.
pub fn patterns() -> &'static IndexMap<i64, IndexMap<&'static str, i64>> {
    static PATTERNS: OnceLock<IndexMap<i64, IndexMap<&'static str, i64>>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        IndexMap::from([
            (0, IndexMap::from([(WEST, 1), (EAST, 1), (SOUTH, 1)])),
            (1, IndexMap::from([(NORTH, 2), (EAST, 2)])),
            (2, IndexMap::from([(NORTH, 2), (EAST, 1), (SOUTH, 1)])),
            (3, IndexMap::from([(NORTH, 1), (EAST, 1), (SOUTH, 2)])),
        ])
    })
}

/// Fortree Gym's doors, found by a sweep as its module loads, and read whenever the field's
/// ground is, since they turn as the player walks.
pub struct RotatingDoors {
    pub classes: Vec<&'static str>,
    memory: Rc<dyn Memory>,
    sequence: Slot<Option<i64>>,
    vtable: Slot<i64>,
    doors: RefCell<Vec<i64>>,
    pivots: RefCell<IndexMap<i64, (i64, i64)>>,
}

impl RotatingDoors {
    pub fn new(memory: Rc<dyn Memory>) -> Self {
        RotatingDoors {
            classes: vec![CLASS],
            memory,
            sequence: Slot::new(None),
            vtable: Slot::new(0),
            doors: RefCell::new(Vec::new()),
            pivots: RefCell::new(IndexMap::new()),
        }
    }

    pub fn track(&self, found: &Found) -> Result<()> {
        // The module's image is copied into the heap as it loads, vtable and all, and a sweep
        // finds the class's vtable in its code too: a find is taken only in the sequence's shape.
        let mut sequences: Vec<i64> = Vec::new();
        for at in found.of(&[CLASS]) {
            if self.head(at)?.is_some() {
                sequences.push(at);
            }
        }
        if let Some(&last) = sequences.last() {
            self.sequence.set(Some(last));
            let head = self.memory.read(last, HEAD)?;
            self.vtable.set(u32(&head, 0)?);
            let mut doors = Vec::new();
            for i in 0..COUNT {
                doors.push(u32(&head, DOORS + 4 * i)?);
            }
            *self.doors.borrow_mut() = doors;
            self.pivots.borrow_mut().clear();
        }
        Ok(())
    }

    /// The doors as they stand now, in the Gym, or none elsewhere and before a sweep has
    /// found them.
    pub fn doors(&self, zone: i64) -> Result<Vec<RotatingDoor>> {
        let Some(sequence) = self.sequence.get().filter(|_| zone == GYM) else {
            return Ok(Vec::new());
        };
        let Some(head) = self.head(sequence)? else {
            return Ok(Vec::new());
        };
        if u32(&head, 0)? != self.vtable.get() {
            return Ok(Vec::new());
        }
        let mut out: Vec<RotatingDoor> = Vec::new();
        let doors = self.doors.borrow().clone();
        for (i, door) in doors.into_iter().enumerate() {
            let [pattern, _, turn, before] =
                RECORD.unpack_from(&head, RECORDS + i as i64 * RECORD.size())?;
            let arms = patterns().get(&pattern);
            let pivot = self.pivot(door)?;
            let (Some(arms), Some(pivot)) = (arms, pivot) else {
                return Ok(Vec::new());
            };
            out.push(RotatingDoor {
                pivot,
                arms: arms.clone(),
                turn: py::modulo(if turn < 0 { before } else { turn }, 4),
            });
        }
        Ok(out)
    }

    /// A door's pivot as a cell, the one south-east of the corner: read once, since a door
    /// never moves.
    fn pivot(&self, door: i64) -> Result<Option<(i64, i64)>> {
        if !self.pivots.borrow().contains_key(&door) {
            let raw = self.memory.read(door + PIVOT, 12)?;
            let (x, z) = (binary::f32(&raw, 0)?, binary::f32(&raw, 8)?);
            if !((0.0..65536.0).contains(&x) && (0.0..65536.0).contains(&z)) {
                return Ok(None);
            }
            let cell = (
                py::floordiv(py::round(x)?, CELL),
                py::floordiv(py::round(z)?, CELL),
            );
            self.pivots.borrow_mut().insert(door, cell);
        }
        Ok(Some(self.pivots.borrow()[&door]))
    }

    fn head(&self, at: i64) -> Result<Option<Vec<u8>>> {
        let head = self.memory.read(at, HEAD)?;
        for i in 0..COUNT {
            if !within(HEAP_SPAN, u32(&head, DOORS + 4 * i)?) {
                return Ok(None);
            }
        }
        for i in 0..COUNT {
            let pattern = RECORD.unpack_from(&head, RECORDS + i * RECORD.size())?[0];
            if !(0 <= pattern && pattern < patterns().len() as i64) {
                return Ok(None);
            }
        }
        Ok(Some(head))
    }
}

impl Tracked for RotatingDoors {
    fn classes(&self) -> Vec<&'static str> {
        self.classes.clone()
    }

    fn track(&self, found: &Found) -> Result<()> {
        RotatingDoors::track(self, found)
    }
}

impl DoorReader for RotatingDoors {
    fn doors(&self, zone: i64) -> Result<Vec<RotatingDoor>> {
        RotatingDoors::doors(self, zone)
    }
}
