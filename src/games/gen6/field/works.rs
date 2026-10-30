//! The save's event works and flags, `field::EventWork`, found by a sweep: what arms a trigger
//! and what says a hidden item is found (`hidden_items`).
//!
//! The works are u16s from +0x04, work 0x4000 first: entering Glacia's corridor, whose trigger is
//! armed while work `0x414d` holds 0, took it from 1 to 0, and its scene put it back to 1
//! (or_elite_four_glacia). The flags are a bit each from +0x2f4, the low bit first.

use std::cell::Cell;
use std::rc::Rc;

use crate::core::discovery::Found;
use crate::core::memory::Memory;
use crate::games::gen6::field::features::Tracked;
use crate::py::Result;

pub const EVENT_CLASS: &str = "N5field9EventWorkE";
/// Where `field::EventWork` keeps its flags (Omega Ruby's `0x12fd48`).
pub const FLAGS: i64 = 0x2F4;
pub const WORKS: i64 = 0x04;
/// Where `field::EventWork` keeps its works, a u16 each, and the first one's number.
pub const FIRST_WORK: i64 = 0x4000;

/// The save's `field::EventWork`, tracked by every sweep (`gen6.field.Tracked`).
pub struct Works {
    pub classes: Vec<&'static str>,
    memory: Rc<dyn Memory>,
    /// Where the save keeps its works, once a sweep has found them.
    pub event: Cell<Option<i64>>,
}

impl Works {
    pub fn new(memory: Rc<dyn Memory>) -> Self {
        Works {
            classes: vec![EVENT_CLASS],
            memory,
            event: Cell::new(None),
        }
    }

    pub fn track(&self, found: &Found) -> Result<()> {
        if let Some(&last) = found.of(&[EVENT_CLASS]).last() {
            self.event.set(Some(last));
        }
        Ok(())
    }

    /// Whether the save's `work` holds `value`; False before a sweep has found the works.
    pub fn holds(&self, work: i64, value: i64) -> Result<bool> {
        let Some(event) = self.event.get() else {
            return Ok(false);
        };
        let at = event + WORKS + (work - FIRST_WORK) * 2;
        // `int.from_bytes(..., "little")`, which takes however many bytes there are.
        let held = self
            .memory
            .read(at, 2)?
            .iter()
            .rev()
            .fold(0i64, |held, &byte| held << 8 | i64::from(byte));
        Ok(held == value)
    }

    /// Whether a trigger armed by `work` at `value` is armed now: always for work 0, which
    /// arms nothing, as every Laverre Gym panel's but one (spike 258).
    pub fn armed(&self, work: i64, value: i64) -> Result<bool> {
        Ok(work == 0 || self.holds(work, value)?)
    }
}

impl Tracked for Works {
    fn classes(&self) -> Vec<&'static str> {
        self.classes.clone()
    }

    fn track(&self, found: &Found) -> Result<()> {
        Works::track(self, found)
    }
}
