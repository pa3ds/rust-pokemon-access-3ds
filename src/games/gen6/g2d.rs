//! Game Freak's layout helper, `app::util::G2DUtil`, and the layouts it draws (spikes 181 and
//! 183): a view of ORAS's AreaNav (`oras.subscreen.areanav`) and of the Move Reminder's pages
//! (`gen6.screens.reminder`) draws the layouts of the one at its +0x5C, and Pokémon-Amie's decorate
//! screen's item pages point at one (`gen6.subscreen.amie`).
//!
//! ```text
//! view     +0x5C  -> app::util::G2DUtil
//! G2DUtil  +0x1C  -> its layouts, 8 bytes each: a handle, then flags; +0x20 how many
//! handle   +0x04  the layout's work, as a layout pool's table keeps it
//! ```

use std::collections::HashSet;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::py::Result;

/// A view -> its `G2DUtil`.
pub const UTIL: i64 = 0x5C;
pub const UTIL_CLASS: &str = "N3app4util7G2DUtilE";
pub const LAYOUTS: i64 = 0x1C;
pub const LAYOUT_COUNT: i64 = 0x20;
pub const ENTRY: i64 = 8;
pub const WORK: i64 = 0x04;
pub const MAX_LAYOUTS: i64 = 16;

/// The works of the layouts a `G2DUtil` draws.
pub fn util_works(memory: &dyn Memory, util: i64) -> Result<HashSet<i64>> {
    let listed = memory.read(util + LAYOUTS, 8)?;
    let (table, count) = (u32(&listed, 0)?, u32(&listed, 4)?);
    if table == 0 || !(0 < count && count <= MAX_LAYOUTS) {
        return Ok(HashSet::new());
    }
    let entries = memory.read(table, ENTRY * count)?;
    let mut works: HashSet<i64> = HashSet::new();
    for i in 0..count {
        let handle = u32(&entries, ENTRY * i)?;
        if handle != 0 {
            let work = u32(&memory.read(handle + WORK, 4)?, 0)?;
            if work != 0 {
                works.insert(work);
            }
        }
    }
    Ok(works)
}
