//! The Bag's cursor source: the item under its cursor, from the Bag's own objects (spike A).
//!
//! pa3ds finds the Bag's cursor by the window drawn round its row and says the row's words. The top
//! screen's pocket, count and description are that item's details, which used to wait two polls
//! after a move whatever the screen held, and could be another item's when the pockets changed
//! quickly. The Bag keeps the item it has chosen, so its details are said the poll the top screen
//! names that item, and never while it names another (`ScreenAnnouncer.say`'s `chosen`;
//! docs/plans/cursor-sources.md):
//!
//! ```text
//! app::bag::Proc         +0x24 -> app::bag::MainSystem
//! app::bag::MainSystem   +0x58 -> a block of no class of its own, whose +0x00 ->
//! app::bag::GraphicWork  +0x10  u16 the item under the cursor, 0 in an empty pocket
//! ```
//!
//! An item's name is the ROM's, which the top screen writes among its words. The list's row may say
//! another thing: a TM's row is its move, "Rock Tomb", where the top screen says "TM39 Rock Tomb"
//! (or_tm_steel_wing_wingull), so the row is never taken for the name, nor the name for the row.
//!
//! Bound by:
//! - ADR-0013: a cursor source's details are said once the screen names its item (amended
//!   2026-11-14); its objects are reached from the Bag's process and checked by their classes, as
//!   ADR-0003 has every structure found, only the offsets fixed.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u16;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::CursorPoll;
use crate::games::gen6::anchors::follow;
use crate::games::gen6::apps::LOADED_CODE;
use crate::py::Result;

pub const MAIN: &str = "N3app3bag10MainSystemE";
pub const DRAW: &str = "N3app3bag11GraphicWorkE";

/// Where a game's Bag keeps the item under its cursor, from its process (module docstring).
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// The process's pointer to `app::bag::MainSystem`.
    pub main: i64,
    /// The pointers from there to `app::bag::GraphicWork`.
    pub draw: &'static [i64],
    /// `GraphicWork`'s u16 item number.
    pub item: i64,
}

pub const ORAS: Shape = Shape {
    main: 0x24,
    draw: &[0x58, 0x00],
    item: 0x10,
};

/// The name of the item the Bag's own objects say its list's cursor is on, while it runs.
pub struct Chosen {
    memory: Rc<dyn Memory>,
    shape: Shape,
    process: Rc<dyn Fn() -> Option<i64>>,
    items: Vec<String>,
    names: Option<(i64, ClassNames)>,
    /// The list's cursor: the one whose item is a row of its menu (`CursorPoll.rows`), not the
    /// TM's bar over the party.
    cursor: Option<i64>,
}

impl Chosen {
    /// `process` is the Bag's while it runs, else None; `items` the ROM's item names, by
    /// number (`GameText.items`).
    pub fn new(
        memory: Rc<dyn Memory>,
        shape: Shape,
        process: Rc<dyn Fn() -> Option<i64>>,
        items: Vec<String>,
    ) -> Self {
        Chosen {
            memory,
            shape,
            process,
            items,
            names: None,
            cursor: None,
        }
    }

    /// The list's cursor -> the name of the item the Bag has chosen; nothing while the Bag is
    /// not running, a pocket is empty, or its objects are not yet built.
    pub fn poll(&mut self, cursors: &CursorPoll) -> Result<IndexMap<i64, String>> {
        let process = (self.process)();
        let Some(process) = process else {
            self.names = None;
            self.cursor = None;
            return Ok(IndexMap::new());
        };
        for item in &cursors.items {
            if !item.pieces.is_empty()
                && item
                    .pieces
                    .iter()
                    .all(|piece| cursors.rows.contains(&piece.r#box))
            {
                self.cursor = Some(item.cursor);
            }
        }
        let number = self.number(process)?;
        let Some(cursor) = self.cursor else {
            return Ok(IndexMap::new());
        };
        if !(0 < number && number < self.items.len() as i64) {
            return Ok(IndexMap::new());
        }
        Ok(IndexMap::from([(cursor, self.items[number as usize].clone())]))
    }

    fn number(&mut self, process: i64) -> Result<i64> {
        // Named afresh for each Bag: its classes are in a module, whose memory is the next
        // module's once it is unloaded (`apps.RunningApp.poll`).
        if self.names.as_ref().is_none_or(|names| names.0 != process) {
            self.names = Some((process, ClassNames::new(self.memory.clone(), LOADED_CODE)));
        }
        let names = &self.names.as_ref().expect("named above").1;
        let memory = &*self.memory;
        let main = follow(memory, process, &[self.shape.main])?;
        let Some(main) = main else {
            return Ok(0);
        };
        if names.of_object(main)?.as_deref() != Some(MAIN) {
            return Ok(0);
        }
        let draw = follow(memory, main, self.shape.draw)?;
        let Some(draw) = draw else {
            return Ok(0);
        };
        if names.of_object(draw)?.as_deref() != Some(DRAW) {
            return Ok(0);
        }
        u16(&memory.read(draw + self.shape.item, 2)?, 0)
    }
}
