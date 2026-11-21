//! The Bag's cursor source: the item under its cursor, from the Bag's own objects (spike A).
//!
//! pa3ds finds the Bag's cursor by the window drawn round its row and says the row's words. The top
//! screen's pocket, count and description are that item's details, which used to wait two polls
//! after a move whatever the screen held, and could be another item's when the pockets changed
//! quickly. The Bag keeps the item it has chosen, so its details are said the poll the top screen
//! names that item, and never while it names another (`ScreenAnnouncer.say`'s `chosen`;
//! ADR-0043):
//!
//! ```text
//! app::bag::Proc         +0x24 -> app::bag::MainSystem
//! app::bag::MainSystem   +0x3C  u8 the pocket: 0 Items, 1 Medicine, 2 TMs & HMs, 3 Berries,
//!                               4 Key Items, the order of the ROM's names for them
//! app::bag::MainSystem   +0x40  u8 the Bag's mode: 6 the list, 7 an item carried with Y,
//!                               12 an item's action list, 13 and 17 the party for a TM and
//!                               for Give (spike B; `carrying`, for `bag_sort`)
//! app::bag::MainSystem   +0x58 -> a block of no class of its own, whose +0x00 ->
//! app::bag::GraphicWork  +0x10  u16 the item under the cursor, 0 in an empty pocket
//! ```
//!
//! X keeps the pocket at +0x30, the mode at +0x34 and the block at +0x4C, 12 bytes sooner,
//! numbered and laid out the same (spike A on x_field_bag, spike B on x_bag_manual_sorting).
//!
//! The pocket's name is said the poll its number changes, before the item. The top screen's title,
//! written with the item's details a poll or two later, said them as "Potion", then "MEDICINE" with
//! the count and description, and an empty pocket's 0.27 to 0.41 s after it changed (spike A,
//! or_menus).
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
use crate::py::{self, Error, Result};

pub const MAIN: &str = "N3app3bag10MainSystemE";
pub const DRAW: &str = "N3app3bag11GraphicWorkE";
/// ITEMS, MEDICINE, TMs & HMs, BERRIES, KEY ITEMS: the first lines of a text file that goes on
/// with the Bag's other words, so a pocket byte past them names no pocket.
pub const POCKETS: i64 = 5;

/// Where a game's Bag keeps the item under its cursor, from its process (module docstring).
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// The process's pointer to `app::bag::MainSystem`.
    pub main: i64,
    /// `MainSystem`'s byte numbering the pocket open.
    pub pocket: i64,
    /// `MainSystem`'s byte numbering what the Bag is doing, `CARRYING` an item carried.
    pub mode: i64,
    /// The pointers from there to `app::bag::GraphicWork`.
    pub draw: &'static [i64],
    /// `GraphicWork`'s u16 item number.
    pub item: i64,
}

pub const ORAS: Shape = Shape {
    main: 0x24,
    pocket: 0x3C,
    mode: 0x40,
    draw: &[0x58, 0x00],
    item: 0x10,
};
pub const XY: Shape = Shape {
    main: 0x24,
    pocket: 0x30,
    mode: 0x34,
    draw: &[0x4C, 0x00],
    item: 0x10,
};
/// The Bag's mode while an item is carried with Y, from its pick-up to its drop (spike B on
/// or_bag_manual_sorting and x_bag_manual_sorting): no other mode of the list, its menus or its
/// party read 7.
pub const CARRYING: i64 = 7;

/// The name of the item the Bag's own objects say its list's cursor is on, and of its pocket
/// as it changes, while it runs.
pub struct Chosen {
    memory: Rc<dyn Memory>,
    shape: Shape,
    process: Rc<dyn Fn() -> Option<i64>>,
    items: Vec<String>,
    pockets: Vec<String>,
    names: Option<(i64, ClassNames)>,
    /// The list's cursor: the one whose item is a row of its menu (`CursorPoll.rows`), not the
    /// TM's bar over the party.
    cursor: Option<i64>,
    /// The pocket last named.
    pocket: Option<i64>,
}

impl Chosen {
    /// `process` is the Bag's while it runs, else None; `items` the ROM's item names, by
    /// number (`GameText.items`), and `pockets` its pockets' (`GameText.pockets`).
    pub fn new(
        memory: Rc<dyn Memory>,
        shape: Shape,
        process: Rc<dyn Fn() -> Option<i64>>,
        items: Vec<String>,
        pockets: Vec<String>,
    ) -> Self {
        Chosen {
            memory,
            shape,
            process,
            items,
            pockets: py::slice(&pockets, 0, POCKETS).to_vec(),
            names: None,
            cursor: None,
            pocket: None,
        }
    }

    /// The list's cursor -> the name of the item the Bag has chosen, nothing while the Bag is
    /// not running, a pocket is empty, or its objects are not yet built; and the pocket's name
    /// where it has changed, from the poll the list's cursor first shows, the Bag built.
    pub fn poll(
        &mut self,
        cursors: &CursorPoll,
    ) -> Result<(IndexMap<i64, String>, Option<String>)> {
        let process = (self.process)();
        let Some(process) = process else {
            self.names = None;
            self.cursor = None;
            self.pocket = None;
            return Ok((IndexMap::new(), None));
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
        let main = self.main(process)?;
        let number = self.number(main)?;
        let mut pocket = None;
        if let Some(main) = main
            && self.cursor.is_some()
        {
            let at = *self
                .memory
                .read(main + self.shape.pocket, 1)?
                .first()
                .ok_or_else(|| Error::index("index out of range"))? as i64;
            if Some(at) != self.pocket && at < self.pockets.len() as i64 {
                self.pocket = Some(at);
                pocket = Some(self.pockets[at as usize].clone());
            }
        }
        let Some(cursor) = self.cursor else {
            return Ok((IndexMap::new(), pocket));
        };
        if !(0 < number && number < self.items.len() as i64) {
            return Ok((IndexMap::new(), pocket));
        }
        Ok((
            IndexMap::from([(cursor, self.items[number as usize].clone())]),
            pocket,
        ))
    }

    /// Whether an item is picked up with Y, from the Bag's mode.
    pub fn carrying(&mut self) -> Result<bool> {
        let process = (self.process)();
        let main = match process {
            None => None,
            Some(process) => self.main(process)?,
        };
        let Some(main) = main else {
            return Ok(false);
        };
        let mode = *self
            .memory
            .read(main + self.shape.mode, 1)?
            .first()
            .ok_or_else(|| Error::index("index out of range"))? as i64;
        Ok(mode == CARRYING)
    }

    fn main(&mut self, process: i64) -> Result<Option<i64>> {
        // Named afresh for each Bag: its classes are in a module, whose memory is the next
        // module's once it is unloaded (`apps.RunningApp.poll`).
        if self.names.as_ref().is_none_or(|names| names.0 != process) {
            self.names = Some((process, ClassNames::new(self.memory.clone(), LOADED_CODE)));
        }
        let main = follow(&*self.memory, process, &[self.shape.main])?;
        let Some(main) = main else {
            return Ok(None);
        };
        let names = &self.names.as_ref().expect("named above").1;
        Ok((names.of_object(main)?.as_deref() == Some(MAIN)).then_some(main))
    }

    fn number(&self, main: Option<i64>) -> Result<i64> {
        let draw = match main {
            None => None,
            Some(main) => follow(&*self.memory, main, self.shape.draw)?,
        };
        let Some(draw) = draw else {
            return Ok(0);
        };
        let Some((_, names)) = &self.names else {
            return Ok(0);
        };
        if names.of_object(draw)?.as_deref() != Some(DRAW) {
            return Ok(0);
        }
        u16(&self.memory.read(draw + self.shape.item, 2)?, 0)
    }
}
