//! The Dowsing Machine's lights: what the machine the player holds out shows of a hidden item.
//!
//! The machine is drawn in the player's hands on the field, not on the bottom screen. Its two lights
//! come on only while the player faces a hidden item near enough, and their colour says how near:
//! blue, green, yellow, then red with the item in the cell in front (omega_ruby_item-finder.ctm,
//! 2026-09-29). X's machine shoots two beams the way the player faces instead, in the same four
//! colours at the same levels (x_route_8_dowsing, spike 243). The game keeps what they show in a small
//! record of no class of its own, reached from the field's process through the field's work, which
//! has no class either:
//!
//! ```text
//! process (`field::FieldmapProc`)  +0x24 -> the field's work
//! work                             +0x1d0 in ORAS, +0x1ac in X -> the record (`Game.dowsing`)
//! record  +0x00 -> the work    +0x08 u32 1 with the item in front    +0x0c u32 a timer
//!         +0x10 u32 the lights: 0 off, 1 blue, 2 green, 3 yellow, 4 red
//!         +0x14 s32 x and +0x18 s32 z, the cell of the item they show; -1 while they are off
//! ```
//!
//! Turning or stepping, the level can read another value for a frame or two before it settles, so it
//! is said once two polls have read it. Where the item is stays unsaid, since the game shows only
//! the colour and which way the player faces (ADR-0017).

use std::rc::Rc;

use crate::core::binary::u32;
use crate::core::discovery::{HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::core::speech::Category;
use crate::core::words::own;
use crate::games::common::field::things::Field;
use crate::games::gen6::field::voice::Said;
use crate::py::Result;

/// Where the field's process holds the field's work.
pub const WORK: i64 = 0x24;
pub const PARENT: i64 = 0x00;
pub const LEVEL: i64 = 0x10;
pub const HEAD: i64 = LEVEL + 4;
/// What each level of the lights shows, from 1.
pub const COLOURS: [&str; 4] = ["blue", "green", "yellow", "red"];
pub const DARK: &str = "nothing";

/// The lights' level, or None where the field's work holds no record that points back at it
/// or the level is out of range.
pub fn level(memory: &dyn Memory, process: i64, offset: i64) -> Result<Option<i64>> {
    let work = u32(&memory.read(process + WORK, 4)?, 0)?;
    if !within(HEAP_SPAN, work) {
        return Ok(None);
    }
    let record = u32(&memory.read(work + offset, 4)?, 0)?;
    if !within(HEAP_SPAN, record) {
        return Ok(None);
    }
    let head = memory.read(record, HEAD)?;
    if u32(&head, PARENT)? != work || u32(&head, LEVEL)? > COLOURS.len() as i64 {
        return Ok(None);
    }
    Ok(Some(u32(&head, LEVEL)?))
}

/// What the lights show, said as it changes: the colour, or "nothing" once they go off. A
/// field voice (`heard`).
pub struct Lights {
    memory: Rc<dyn Memory>,
    offset: i64,
    process: Rc<dyn Fn() -> Option<i64>>,
    /// The level the last poll read.
    read: Option<i64>,
    /// The level last said; off until the lights first come on.
    said: i64,
}

impl Lights {
    /// `Lights(memory, offset)`: `process` left to its default, none.
    pub fn new(memory: Rc<dyn Memory>, offset: i64) -> Self {
        Lights::with_process(memory, offset, Rc::new(|| None))
    }

    /// `process` is the field's process while the field runs, None otherwise.
    pub fn with_process(
        memory: Rc<dyn Memory>,
        offset: i64,
        process: Rc<dyn Fn() -> Option<i64>>,
    ) -> Self {
        Lights {
            memory,
            offset,
            process,
            read: None,
            said: 0,
        }
    }

    /// What to say, when the level has settled on another than the one last said.
    pub fn poll(&mut self, process: i64) -> Result<Option<String>> {
        let now = level(&*self.memory, process, self.offset)?;
        let read = std::mem::replace(&mut self.read, now);
        let Some(now) = self.read else {
            return Ok(None);
        };
        if Some(now) != read || now == self.said {
            return Ok(None);
        }
        self.said = now;
        let light = if now != 0 {
            COLOURS[(now - 1) as usize]
        } else {
            DARK
        };
        Ok(Some(own(
            "dowsing",
            &[("light", &own(&format!("dowsing-{light}"), &[]))],
        )))
    }

    /// The lights as a field voice: polled while the field runs, wherever the player is,
    /// and said as screen text. Three small reads a field poll.
    pub fn heard(&mut self, _where: Option<&Field>) -> Result<Option<Said>> {
        let said = match (self.process)() {
            None => None,
            Some(process) => self.poll(process)?,
        };
        Ok(said.map(|said| Said {
            words: said.clone(),
            category: Category::Screen,
            logged: said,
        }))
    }
}
