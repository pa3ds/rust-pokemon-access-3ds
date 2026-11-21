//! Choice boxes: announce the field's Yes/No box as it opens and as its cursor moves.
//!
//! When a question in the field has been read and the player presses A, the game puts a small box of
//! options beside the speaker — "Yes" and "No" for Birch's nickname question and his Route 103
//! question. It is not a list window, and it has no vtable (spikes 24 and 26), so no class name finds
//! it. Something it holds has one: the input device manager, `gfl::ui::CTR_DeviceManager`, sits at
//! +0x0c. Every heap word pointing at the manager is a candidate, and the layout is the validator.
//!
//! ```text
//! box   +0x0c -> gfl::ui::CTR_DeviceManager
//!       +0x7c -> entry table: count entries of (u32 value, gfl::str::StrBuf label)
//!       +0x80 u32 cursor row           +0x84 u32 first row shown
//!       +0x88 u32 count                +0x90 s16 state: 0 or 1 opening, 3 up, 4 answered
//! ```
//!
//! At most four rows show (the draw loop at 0x00357d20), and the answer is the entry at the first row
//! shown plus the cursor row (0x003583a8). The box is allocated when A is pressed on the question,
//! reaches state 3 within half a second as it appears, and is freed half a second after the answer.
//! movie3 built it at the same address for three prompts in a row (spike 26).
//!
//! Bound by:
//! - ADR-0012: a choice box says the option under its cursor as it opens, then each moved to.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{s16, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::announce::Focus;
use crate::games::common::strbuf;
use crate::py::Result;

pub const DEVICE_MANAGER_CLASS: &str = "N3gfl2ui17CTR_DeviceManagerE";

pub const BOX_DEVICE_MANAGER: i64 = 0x0C;
pub const BOX_ENTRIES: i64 = 0x7C;
pub const BOX_CURSOR: i64 = 0x80;
pub const BOX_FIRST_ROW: i64 = 0x84;
pub const BOX_COUNT: i64 = 0x88;
pub const BOX_STATE: i64 = 0x90;
pub const BOX_HEAD: i64 = BOX_STATE + 2;

pub const ENTRY: i64 = 8;
pub const ENTRY_LABEL: i64 = 4;
pub const ROWS: i64 = 4;
pub const MAX_ENTRIES: i64 = 16;
/// Up and taking input. 0 and 1 were seen as it opened, 4 once answered.
pub const STATE_UP: i64 = 3;
/// The box's update switches on five states, 0 to 4.
pub const MAX_STATE: i64 = 4;

/// Tracks choice boxes and reports the option under the cursor.
pub struct ChoiceReader {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    active: Vec<i64>,
    /// Box -> the index last announced, for boxes currently up.
    shown: IndexMap<i64, i64>,
    /// Box -> its labels, for boxes currently up.
    labels: IndexMap<i64, Vec<String>>,
}

impl ChoiceReader {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        ChoiceReader {
            memory,
            names,
            active: Vec::new(),
            shown: IndexMap::new(),
            labels: IndexMap::new(),
        }
    }

    /// Every box found so far, up or not: the Poké Mart's Buy, Sell and Quit is one, inside
    /// the shop's own object (`mart`).
    pub fn boxes(&self) -> Vec<i64> {
        self.active.clone()
    }

    /// Every label of every box up at the last poll, for the screen text reader to leave out.
    pub fn labels(&self) -> HashSet<String> {
        self.labels.values().flatten().cloned().collect()
    }

    /// Read the boxes among these addresses from now on, as well as every box found before.
    ///
    /// Candidates are all the objects that hold the device manager at +0x0c, and most of them are
    /// something else, so only those laid out as a choice box when found are kept. A box once
    /// found is read on every poll after: the game builds the next box where the last one was,
    /// and a sweep can take seconds to find a new one.
    pub fn track(&mut self, candidates: impl IntoIterator<Item = i64>) -> Result<()> {
        for address in candidates {
            if !self.active.contains(&address) && self.read(address)?.is_some() {
                self.active.push(address);
            }
        }
        Ok(())
    }

    pub fn poll(&mut self) -> Result<Vec<Focus>> {
        let mut focus: Vec<Focus> = Vec::new();
        for &address in &self.active {
            let seen = self.read(address)?;
            let Some((labels, index, _state)) = seen.filter(|seen| seen.2 == STATE_UP) else {
                self.shown.shift_remove(&address);
                self.labels.shift_remove(&address);
                continue;
            };
            let before = self.shown.get(&address).copied();
            if before != Some(index) {
                focus.push(Focus {
                    source: address,
                    label: labels[index as usize].clone(),
                    index,
                    count: labels.len() as i64,
                    opened: before.is_none(),
                });
            }
            self.shown.insert(address, index);
            self.labels.insert(address, labels);
        }
        Ok(focus)
    }

    /// (labels, index under the cursor, state) for anything laid out as a choice box.
    fn read(&self, r#box: i64) -> Result<Option<(Vec<String>, i64, i64)>> {
        let head = self.memory.read(r#box, BOX_HEAD)?;
        if self
            .names
            .of_object(u32(&head, BOX_DEVICE_MANAGER)?)?
            .as_deref()
            != Some(DEVICE_MANAGER_CLASS)
        {
            return Ok(None);
        }
        let (count, cursor) = (u32(&head, BOX_COUNT)?, u32(&head, BOX_CURSOR)?);
        let first = u32(&head, BOX_FIRST_ROW)?;
        let state = s16(&head, BOX_STATE)?;
        if !(0 < count && count <= MAX_ENTRIES) || !(0..=MAX_STATE).contains(&state) {
            return Ok(None);
        }
        if !(cursor < count.min(ROWS)) || !(first + cursor < count) {
            return Ok(None);
        }
        let table = self.memory.read(u32(&head, BOX_ENTRIES)?, ENTRY * count)?;
        let mut labels: Vec<String> = Vec::new();
        for i in 0..count {
            let buffer = u32(&table, i * ENTRY + ENTRY_LABEL)?;
            if self.names.of_object(buffer)?.as_deref() != Some(strbuf::CLASS) {
                return Ok(None);
            }
            let label = strbuf::label(&*self.memory, buffer)?;
            let Some(label) = label else {
                return Ok(None);
            };
            labels.push(label);
        }
        Ok(Some((labels, first + cursor, state)))
    }
}
