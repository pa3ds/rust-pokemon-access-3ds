//! Whether a zone's conveyor belts are turned round, by the switch the game has for them
//! (`Game.conveyors`).
//!
//! In X's Poké Ball Factory the field's module keeps a `field::FieldUniqueSequenceBallFactory`
//! while the player is in it, whose +0x14 points at a byte that is 0, or 1 once the switch has turned
//! the belts round, and back to 0 at the next press: across three presses of the switch, it and one
//! other byte of memory flipped each time and nothing else did, and a save made after one press
//! loads with it 1 and the belts turned (spike A).

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::u32;
use crate::core::discovery::{Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::games::gen6::field::features::Tracked;
use crate::py::Result;

/// How wide the pointer to the byte is.
pub const POINTER: i64 = 4;

/// The objects saying whether the belts are turned, found by a sweep as their module loads.
pub struct Switches {
    memory: Rc<dyn Memory>,
    /// Each object's class -> where in it the pointer to the byte is.
    states: IndexMap<&'static str, i64>,
    pub classes: Vec<&'static str>,
    /// Each class -> the object found, and its vtable.
    found: RefCell<IndexMap<&'static str, (i64, i64)>>,
}

impl Switches {
    /// Python's `states` is a set of (class, offset) built from `Game.conveyors`, whose order
    /// its strings make Python's own; here the `Game`'s order, each once.
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

    /// Whether the switch the object of `state` keeps has turned its belts, or None before a
    /// sweep has found it or while it is not that object any more.
    pub fn turned(&self, state: (&str, i64)) -> Result<Option<bool>> {
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
        Ok(Some(self.memory.read(to, 1)?[0] != 0))
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
