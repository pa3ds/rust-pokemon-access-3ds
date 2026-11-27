//! Sootopolis Gym's rooms of thin ice: a tile cracks as the player steps onto it, and gives way
//! under the next step onto it (spike 167).
//!
//! The Gym loads a module of its own, `DllUSGymWater.cro`, whose
//! `field::FieldUniqueSequenceGymWater` checks the player's cell against three rooms at every
//! step. The module's image is mapped whole, the sequence's vtable at `VTABLE` in it, and two
//! words of its code hold where the loader put what the rooms are read from: `SOLID_AT` the cells
//! of the rooms that are no ice, `NO_ICE` pairs of u32 x and z, and `DATA_AT` the module's data,
//! whose first three words point to the rooms, the top room first:
//!
//! ```text
//! +0x00 u8 width, u8 height, u8 x and u8 z of the north-west cell
//! +0x04 i32 the floor's height in world units
//! +0x08 u16 the work value that is 1 once the room is cleared
//! +0x0c -> a byte a cell, row by row: low two bits 0 whole, 1 cracked (and the floor that is
//!          no ice, which is never stepped on as ice)
//! +0x10 -> the room's three exits, pairs of u32 x and z, in the module's own image after the
//!          cells that are no ice
//! +0x18 u8  1 once the room is cleared
//! ```
//!
//! A step onto a whole tile sets it to 1; a step onto a cracked one drops the player to the floor
//! below, and the room is whole again. Once every tile is 1, a step onto an exit clears the room,
//! and the module looks at it no more. The game names none of it.

use std::cell::Cell as Slot;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{s32, u32};
use crate::core::discovery::{Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::games::common::field::chunk::CHUNK;
use crate::games::common::field::ground::{CRACKED, Ground, IceRoom, SOLID, WHOLE};
use crate::games::gen6::field::features::Tracked;
use crate::games::gen6::field::landform::{Found as Worked, Here, Landform, as_watch};
use crate::games::gen6::rom::archives::GameText;
use crate::py::Result;

pub const CLASS: &str = "N5field27FieldUniqueSequenceGymWaterE";
/// Sootopolis Gym's zone.
pub const GYM: i64 = 0x1BC;
pub const VTABLE: i64 = 0x3310;
pub const SOLID_AT: i64 = 0x230C;
pub const DATA_AT: i64 = 0x2310;
/// How many cells are no ice: three blocks of nine in the top room, one in the middle.
pub const NO_ICE: i64 = 0x24;
pub const ROOMS: i64 = 3;
pub const RECORD: i64 = 0x1C;
pub const HEIGHT: i64 = 0x04;
pub const STATES: i64 = 0x0C;
pub const EXITS: i64 = 0x10;
pub const CLEARED: i64 = 0x18;
pub const EXIT_COUNT: i64 = 3;
pub const CRACK_BITS: i64 = 3;

/// Sootopolis Gym's rooms of thin ice on the ground as they are now (`Ground.ice`), read
/// through the Gym's module (`IceFloors`) whenever the ground is, since they crack as the player
/// walks: a cracked tile drops the player through, so it stands in the way on its own room's
/// floor, but under the player, who has just cracked it. Not listed.
#[derive(Debug)]
pub struct ThinIce;

impl Landform for ThinIce {
    fn watch(
        &self,
        memory: Rc<dyn Memory>,
        _image: &[u8],
        _text: &GameText,
    ) -> Option<Rc<dyn Tracked>> {
        Some(Rc::new(IceFloors::new(memory)))
    }

    fn laid(&self, _found: &Worked, ground: Ground, here: &Here) -> Result<Ground> {
        let floors = (here.watch)(self);
        let rooms = match floors.as_deref().and_then(as_watch::<IceFloors>) {
            Some(floors) => floors.rooms(here.r#where.zone)?,
            None => Vec::new(),
        };
        Ok(crack(ground, rooms, here.r#where.cell))
    }
}

/// `ThinIce.laid` once its watch has said the rooms: `ground` with them, where any are, their
/// cracked tiles standing but under the `player`. A function of its own, where Python's tests
/// subclass the landform over a fake watch: theirs lays what it reads through it.
pub fn crack(ground: Ground, rooms: Vec<IceRoom>, player: (i64, i64)) -> Ground {
    if rooms.is_empty() {
        return ground;
    }
    let mut cracked: IndexMap<(i64, i64), Option<f64>> = IndexMap::new();
    for room in &rooms {
        if !room.cleared {
            for (cell, state) in room.cells() {
                if state == CRACKED {
                    cracked.insert(cell, Some(room.height));
                }
            }
        }
    }
    Ground {
        ice: rooms,
        ..ground.stood(&cracked, player)
    }
}

/// Sootopolis Gym's rooms of thin ice, found by a sweep as its module loads, and read whenever
/// the field's ground is, since they crack as the player walks: `ThinIce`'s watch.
pub struct IceFloors {
    pub classes: Vec<&'static str>,
    memory: Rc<dyn Memory>,
    sequence: Slot<Option<i64>>,
    vtable: Slot<i64>,
}

impl IceFloors {
    pub fn new(memory: Rc<dyn Memory>) -> Self {
        IceFloors {
            classes: vec![CLASS],
            memory,
            sequence: Slot::new(None),
            vtable: Slot::new(0),
        }
    }

    pub fn track(&self, found: &Found) -> Result<()> {
        // The module's image is copied into the heap as it loads, vtable and all, so a sweep finds
        // the class's vtable there too: a find is taken only where its rooms can be read.
        let mut sequences: Vec<i64> = Vec::new();
        for at in found.of(&[CLASS]) {
            if !self.rooms_at(at)?.is_empty() {
                sequences.push(at);
            }
        }
        if let Some(&last) = sequences.last() {
            self.sequence.set(Some(last));
            self.vtable.set(u32(&self.memory.read(last, 4)?, 0)?);
        }
        Ok(())
    }

    /// The rooms as they are now, in the Gym, or none elsewhere, before a sweep has found them,
    /// and while a battle has the module's data torn down.
    pub fn rooms(&self, zone: i64) -> Result<Vec<IceRoom>> {
        let Some(sequence) = self.sequence.get().filter(|_| zone == GYM) else {
            return Ok(Vec::new());
        };
        if u32(&self.memory.read(sequence, 4)?, 0)? != self.vtable.get() {
            return Ok(Vec::new());
        }
        self.rooms_at(sequence)
    }

    /// `_rooms`.
    fn rooms_at(&self, sequence: i64) -> Result<Vec<IceRoom>> {
        let base = u32(&self.memory.read(sequence, 4)?, 0)? - VTABLE;
        let words = self.memory.read(base + SOLID_AT, 8)?;
        let (solid_at, data) = (u32(&words, 0)?, u32(&words, 4)?);
        if !within(HEAP_SPAN, data) {
            return Ok(Vec::new());
        }
        let solid = self.memory.read(solid_at, 8 * NO_ICE)?;
        let mut no_ice: HashSet<(i64, i64)> = HashSet::new();
        for i in 0..NO_ICE {
            no_ice.insert((u32(&solid, 8 * i)?, u32(&solid, 8 * i + 4)?));
        }
        let mut out: Vec<IceRoom> = Vec::new();
        let records = self.memory.read(data, 4 * ROOMS)?;
        for i in 0..ROOMS {
            let record = u32(&records, 4 * i)?;
            if !within(HEAP_SPAN, record) {
                return Ok(Vec::new());
            }
            let head = self.memory.read(record, RECORD)?;
            let (wide, tall, x, z) = (
                i64::from(head[0]),
                i64::from(head[1]),
                i64::from(head[2]),
                i64::from(head[3]),
            );
            let (states, exits) = (u32(&head, STATES)?, u32(&head, EXITS)?);
            if !(0 < wide && wide <= CHUNK && 0 < tall && tall <= CHUNK) || exits == 0 {
                return Ok(Vec::new());
            }
            let ends = self.memory.read(exits, 8 * EXIT_COUNT)?;
            // A room the save has cleared is given no tiles to keep (the module's `0x1ce0`).
            let cracked = if within(HEAP_SPAN, states) {
                self.memory.read(states, wide * tall)?
            } else {
                vec![0; (wide * tall) as usize]
            };
            let cells: Vec<(i64, i64)> = (0..wide * tall)
                .map(|i| (x + i % wide, z + i / wide))
                .collect();
            let mut exit_cells: Vec<(i64, i64)> = Vec::new();
            for k in 0..EXIT_COUNT {
                exit_cells.push((u32(&ends, 8 * k)?, u32(&ends, 8 * k + 4)?));
            }
            out.push(IceRoom {
                cleared: head[CLEARED as usize] != 0 || !within(HEAP_SPAN, states),
                ..IceRoom::new(
                    (x, z),
                    (wide, tall),
                    s32(&head, HEIGHT)? as f64,
                    cells
                        .iter()
                        .zip(&cracked)
                        .map(|(cell, &byte)| {
                            if no_ice.contains(cell) {
                                SOLID
                            } else if i64::from(byte) & CRACK_BITS == 1 {
                                CRACKED
                            } else {
                                WHOLE
                            }
                        })
                        .collect(),
                    exit_cells,
                )
            });
        }
        Ok(out)
    }
}

impl Tracked for IceFloors {
    fn classes(&self) -> Vec<&'static str> {
        self.classes.clone()
    }

    fn track(&self, found: &Found) -> Result<()> {
        IceFloors::track(self, found)
    }
}
