//! The PC Box in Sun and the Ultra games: the slot under the cursor and who is in it.
//!
//! The Box draws its slots as icons with no words, and the generic reader said nothing as the
//! cursor crossed them. The Box's own objects know both (Sun's Route 1 Pokémon Center, a Slowpoke
//! put in Box 1's sixth slot, driven live):
//!
//! ```text
//! App::Box::BoxLowBgFrame          +0x108 u32 the slot under the cursor: 0 to 29 the box's, six
//!                                  to a row from the top left, 30 to 35 the party's in order
//! App::Box::BoxUppPokeStatusFrame  +0x30 -> the PokemonParam of the top screen's panel, the
//!                                  Pokémon under the cursor: Slowpoke on its slot, nothing on
//!                                  an empty one (`common.pokemon`)
//! ```
//!
//! The box itself is the screen's title, which the screen reader says as it turns. The save's
//! boxes (`Savedata::BoxPokemon`) are a block far larger than ADR-0035 lets pa3ds read, and are not
//! needed.
//!
//! Bound by:
//! - ADR-0035: the panel's PokemonParam is a small block a sweep found.
//! - ADR-0043: a screen's cursor is read from its own objects.

use std::rc::Rc;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::announce::Focus;
use crate::games::common::{pokemon, slots};
use crate::py::Result;

pub const FRAME: &str = "N3App3Box13BoxLowBgFrameE";
pub const STATUS: &str = "N3App3Box21BoxUppPokeStatusFrameE";
pub const CLASSES: [&str; 2] = [FRAME, STATUS];
pub const SLOT: i64 = 0x108;
pub const PANEL: i64 = 0x30;
pub const BOX_SLOTS: i64 = 30;
pub const COLUMNS: i64 = 6;
/// Polls running that must read the same slot and Pokémon before it is said.
pub const STEADY: i64 = 3;
/// The Ultra games' last species.
pub const SPECIES_MAX: i64 = 809;

/// (the frame, the slot, the Pokémon's nickname or None for an empty slot).
type Key = (i64, i64, Option<String>);

/// The slot under the PC Box's cursor, said with the Pokémon in it as either changes.
pub struct BoxReader {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    found: Vec<i64>,
    said: Option<Key>,
    /// What the last poll read, and how many polls running have read it.
    seen: Option<Key>,
    steady: i64,
}

impl BoxReader {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        BoxReader {
            memory,
            names,
            found: Vec::new(),
            said: None,
            seen: None,
            steady: 0,
        }
    }

    /// Python's `adding` defaults to False.
    pub fn track(&mut self, found: impl IntoIterator<Item = i64>, adding: bool) {
        self.found = if adding {
            let mut kept = self.found.clone();
            kept.extend(found.into_iter().filter(|f| !self.found.contains(f)));
            kept
        } else {
            found.into_iter().collect()
        };
    }

    /// What the cursor has come to, if it has moved or the Pokémon under it changed.
    pub fn poll(&mut self) -> Result<Option<Focus>> {
        let frame = self.live(FRAME)?;
        let status = self.live(STATUS)?;
        let (Some(frame), Some(status)) = (frame, status) else {
            self.said = None;
            return Ok(None);
        };
        let slot = u32(&self.memory.read(frame + SLOT, 4)?, 0)?;
        if slot >= BOX_SLOTS + slots::PARTY_SLOTS {
            return Ok(None);
        }
        let param = u32(&self.memory.read(status + PANEL, 4)?, 0)?;
        let held = pokemon::read(&*self.memory, &self.names, param, SPECIES_MAX)?;
        let name = held.map(|held| held.nickname);
        let key = (frame, slot, name.clone());
        // The panel follows the cursor a poll or two late: "row 1, column 6, empty" came 40 ms
        // before the Slowpoke there, and "Party 2, Rowlet" 30 ms before the Wingull, so a slot is
        // said once `STEADY` polls running agree on it.
        if self.said.as_ref() == Some(&key) {
            self.steady = 0;
            return Ok(None);
        }
        self.steady = if self.seen.as_ref() == Some(&key) {
            self.steady + 1
        } else {
            1
        };
        self.seen = Some(key.clone());
        if self.steady < STEADY {
            return Ok(None);
        }
        let opened = self.said.as_ref().is_none_or(|said| said.0 != frame);
        self.said = Some(key);
        let r#where = if slot < BOX_SLOTS {
            own(
                slots::GRID,
                &[
                    ("row", &(slot / COLUMNS + 1)),
                    ("column", &(slot % COLUMNS + 1)),
                ],
            )
        } else {
            own(slots::PARTY, &[("number", &(slot - BOX_SLOTS + 1))])
        };
        let label = match name {
            None => own(slots::EMPTY_SLOT, &[("slot", &r#where)]),
            Some(name) => format!("{}, {name}", r#where),
        };
        Ok(Some(Focus {
            source: frame,
            label,
            index: slot,
            count: BOX_SLOTS + slots::PARTY_SLOTS,
            opened,
        }))
    }

    fn live(&self, name: &str) -> Result<Option<i64>> {
        for &f in &self.found {
            if self.names.of_object(f)?.as_deref() == Some(name) {
                return Ok(Some(f));
            }
        }
        Ok(None)
    }
}
