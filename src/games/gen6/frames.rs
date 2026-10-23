//! The game's own count of frames, which times a scene's beats (`common.scenes.GameClock`).
//!
//! The input device manager, `gfl::ui::CTR_DeviceManager`, is built as the game boots and kept
//! through every app, and what it points at counts the frames the game has read its buttons in, 60
//! a second from boot (spike A):
//!
//! ```text
//! manager  +0x0c -> the input's work
//! work     +0x18 u32 frames since the game booted
//! ```
//!
//! The manager is the one the sweep finds for the field's Yes/No box (`screens.choices`). A count
//! that cannot be read is None, and `GameClock` keeps time by the wall clock until one can.
//!
//! Bound by:
//! - ADR-0038: a scene's beats are timed by the game's own frames, where they can be read.

use crate::core::binary::u32;
use crate::core::discovery::{HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::py::{Kind, Result};

pub const WORK: i64 = 0x0C;
pub const COUNT: i64 = 0x18;

/// The game's count of frames, through the first device manager leading into the heap; None
/// where none does, or the emulator does not answer.
pub fn frames(memory: &dyn Memory, managers: &[i64]) -> Result<Option<i64>> {
    let found = (|| -> Result<Option<i64>> {
        for manager in managers {
            let work = u32(&memory.read(manager + WORK, 4)?, 0)?;
            if within(HEAP_SPAN, work) {
                return Ok(Some(u32(&memory.read(work + COUNT, 4)?, 0)?));
            }
        }
        Ok(None)
    })();
    match found {
        Err(error) if error.is(Kind::RpcError) => Ok(None),
        found => found,
    }
}
