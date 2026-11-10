//! What a zone's switches have turned: its conveyor belts round (`landform.Conveyors`), or its
//! floor strips about their middles (`landform.TurningFloor`).
//!
//! In X's Poké Ball Factory the field's module keeps a `field::FieldUniqueSequenceBallFactory`
//! while the player is in it, whose +0x14 points at a byte that is 0, or 1 once the switch has turned
//! the belts round, and back to 0 at the next press: across three presses of the switch, it and one
//! other byte of memory flipped each time and nothing else did, and a save made after one press
//! loads with it 1 and the belts turned (spike 259).
//!
//! In Snowbelle City Gym a `field::FieldUniqueSequenceGymIce`'s +0x14 points at a u32 a floor
//! strip, how many times its switch has turned it, from 0 to 3 and round to 0: stepping onto the
//! pink switch five times made the first 1, 2, 3, 0 and 1, each as its turn ended, and the other
//! three stayed 0 (x_eighth_gym_switches, spike 286).

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::u32;
use crate::core::discovery::{Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::core::pathfind::Cell;
use crate::games::gen6::field::features::Tracked;
use crate::py::{self, Result};

/// How wide the pointer to what the switches have turned is.
pub const POINTER: i64 = 4;

/// One of Snowbelle City Gym's floor strips, turned about its middle by its switch, with gaps
/// in it the player cannot step into: which cells are gaps depends on how many times it has been
/// turned (`maps.gym_floor`).
#[derive(Debug, Clone, PartialEq)]
pub struct TurningStrip {
    /// The gaps after each number of turns, from none.
    pub gaps: Vec<HashSet<Cell>>,
}

impl TurningStrip {
    /// The gaps after `turns` turns.
    pub fn at(&self, turns: i64) -> &HashSet<Cell> {
        &self.gaps[py::modulo(turns, self.gaps.len() as i64) as usize]
    }
}

/// What a Gym's module turns and shuts (`maps.gym_floor`): Snowbelle City Gym's.
#[derive(Debug, Clone, PartialEq)]
pub struct GymFloor {
    /// The floor strips, in the order of their switches' counts.
    pub strips: Vec<TurningStrip>,
    /// The cells shut on every floor until the Gym's leader is beaten: the two ends of the ice
    /// up Snowbelle City Gym's middle, which Khai says is slid only then.
    pub shut: HashSet<Cell>,
}

/// The objects saying what the switches have turned, found by a sweep as their module
/// loads.
pub struct Switches {
    memory: Rc<dyn Memory>,
    /// Each object's class -> where in it the pointer to what is turned is.
    states: IndexMap<&'static str, i64>,
    pub classes: Vec<&'static str>,
    /// Each class -> the object found, and its vtable.
    found: RefCell<IndexMap<&'static str, (i64, i64)>>,
}

impl Switches {
    /// Python's `states` is a set of (class, offset) built from the game's `landform.Conveyors`
    /// and `landform.TurningFloor`, whose order its strings make Python's own; here the `Game`'s
    /// order, each once.
    pub fn new(memory: Rc<dyn Memory>, states: IndexSet<(&'static str, i64)>) -> Self {
        let states: IndexMap<&'static str, i64> = states.into_iter().collect();
        let classes = states.keys().copied().collect();
        Switches {
            memory,
            states,
            classes,
            found: RefCell::new(IndexMap::new()),
        }
    }

    pub fn track(&self, found: &Found) -> Result<()> {
        for (&name, &at) in &self.states {
            // The module's image is copied into the heap as it loads, vtable and all, so a sweep
            // finds the class's vtable in its code too: a find is taken only where it points into
            // the heap.
            let mut objects: Vec<i64> = Vec::new();
            for one in found.of(&[name]) {
                if self.pointed(one, at)?.is_some() {
                    objects.push(one);
                }
            }
            if let Some(&last) = objects.last() {
                let vtable = u32(&self.memory.read(last, POINTER)?, 0)?;
                self.found.borrow_mut().insert(name, (last, vtable));
            }
        }
        Ok(())
    }

    /// The `size` bytes the object of `state` points at, saying what its switches have
    /// turned, or None before a sweep has found it or while it is not that object any more.
    pub fn turned(&self, state: (&str, i64), size: i64) -> Result<Option<Vec<u8>>> {
        let (name, at) = state;
        let Some(&(one, vtable)) = self.found.borrow().get(name) else {
            return Ok(None);
        };
        if u32(&self.memory.read(one, POINTER)?, 0)? != vtable {
            return Ok(None);
        }
        let Some(to) = self.pointed(one, at)? else {
            return Ok(None);
        };
        Ok(Some(self.memory.read(to, size)?))
    }

    fn pointed(&self, one: i64, at: i64) -> Result<Option<i64>> {
        let to = u32(&self.memory.read(one + at, POINTER)?, 0)?;
        Ok(within(HEAP_SPAN, to).then_some(to))
    }
}

impl Tracked for Switches {
    fn classes(&self) -> Vec<&'static str> {
        self.classes.clone()
    }

    fn track(&self, found: &Found) -> Result<()> {
        Switches::track(self, found)
    }
}
