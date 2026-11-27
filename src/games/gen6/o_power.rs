//! O-Power: the energy the PSS's O-Power list spends, and the powers running, said by R (spike Q).
//!
//! ```text
//!     savedata::AssistPowerData   the save's O-Power record, in the heap from before the title screen
//!         +0x68  u32 the energy, 0 to 10
//! ```
//!
//! Using Hatching, Bargain, Prize Money and Encounter Power took it 10, 8, 6, 2, 0, each as the game
//! charged the power's cost (or_pss_o_power); Y's record keeps it at the same place. The O-Power
//! list draws it as ten orbs along its bottom, pictures whose inner part stays shown, as it empties,
//! for a while after the orb is spent, and named nothing: read from the record, it is the number the
//! orbs settle on.
//!
//! The powers running are words of the top screen's Active panel, `assist_power_upper.bclyt`, up
//! while the PSS is: "Active", the kind of power, and a row for each, "Hatching Power", "Lv. 1",
//! "02:58", its time left ticking each second, left unsaid as a clock's tick
//! (`common.screen_announce`). R says the energy and then the panel while O-Power's list or the panel
//! is up, the energy first.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::discovery::Found;
use crate::core::memory::Memory;
use crate::core::words::own;
use crate::games::common::screen::{ScreenText, distinct, in_rows, join};
use crate::games::common::touch::{FileOf, OnScreen};
use crate::py::Result;

pub const CLASS: &str = "N8savedata15AssistPowerDataE";
pub const ENERGY: i64 = 0x68;
/// The orbs the list draws, the most energy there is.
pub const GAUGE: i64 = 10;
pub const LIST: &str = "assist_power_lower.bclyt";
pub const ACTIVE: &str = "assist_power_upper.bclyt";
pub const ENERGY_WORDS: &str = "o-power-energy";

/// `Callable[[], Sequence[int]]`: the roots of the layouts drawn now. It reads memory.
pub type Shown = Rc<dyn Fn() -> Result<Vec<i64>>>;

/// The save's O-Power record and the screens that show it.
pub struct OPower {
    memory: Rc<dyn Memory>,
    on_screen: OnScreen,
    file_of: FileOf,
    /// The roots of the layouts drawn now, on either screen.
    shown: Shown,
    pauses: bool,
    records: Vec<i64>,
}

impl OPower {
    /// `OPower(memory, on_screen, file_of, shown, pauses)`; Python's `pauses` defaults to False.
    pub fn new(
        memory: Rc<dyn Memory>,
        on_screen: OnScreen,
        file_of: FileOf,
        shown: Shown,
        pauses: bool,
    ) -> Self {
        OPower {
            memory,
            on_screen,
            file_of,
            shown,
            pauses,
            records: Vec::new(),
        }
    }

    /// Keep the record a sweep found, while a sweep finds one.
    pub fn track(&mut self, found: &Found) {
        let records = found.of(&[CLASS]);
        if !records.is_empty() {
            self.records = records;
        }
    }

    /// The energy, then the Active panel's words, while O-Power's list or the panel is up;
    /// None elsewhere.
    pub fn said(&self) -> Result<Option<String>> {
        let mut files: IndexMap<i64, String> = IndexMap::new();
        for root in (self.shown)()? {
            if let Some(file) = (self.file_of)(root) {
                files.insert(root, file.name.clone());
            }
        }
        if !files.values().any(|name| name == LIST || name == ACTIVE) {
            return Ok(None);
        }
        let mut parts: Vec<String> = Vec::new();
        if let Some(energy) = self.energy()? {
            parts.push(own(ENERGY_WORDS, &[("count", &energy)]));
        }
        let panel: Vec<ScreenText> = (self.on_screen)()
            .into_iter()
            .filter(|piece| files.get(&piece.root).map(String::as_str) == Some(ACTIVE))
            .collect();
        if !panel.is_empty() {
            parts.push(join(&distinct(in_rows(panel)), self.pauses));
        }
        let said = parts.join(if self.pauses { ", " } else { " " });
        Ok(if said.is_empty() { None } else { Some(said) })
    }

    /// The energy the record holds, or None where no record is known or it holds more than
    /// the gauge.
    pub fn energy(&self) -> Result<Option<i64>> {
        for &record in &self.records {
            let energy = u32(&self.memory.read(record + ENERGY, 4)?, 0)?;
            if energy <= GAUGE {
                return Ok(Some(energy));
            }
        }
        Ok(None)
    }
}
