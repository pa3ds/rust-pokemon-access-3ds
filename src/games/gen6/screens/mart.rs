//! A Poké Mart's list: a jump of a page said where it lands, not where it passes (spike F,
//! or_pokemart_columns).
//!
//! Left and right move the list's cursor a page, and the game scrolls the list there over a few
//! frames: the row under the cursor read in between named an item passed on the way, "Ultra Ball",
//! said before the one landed on, "Revive", a poll later. The shop's own object says which item is
//! chosen, its bar's row on the screen plus the rows scrolled away, and they pass through the rows
//! in between as well. So a move of one row is said at once, as ever, and a move of more is said
//! once the row and the scroll have held still for a poll, in the screen's words for the item.
//!
//! ```text
//! EventShopBuyCall  +0x18 the bar's row on the screen   +0x1C the rows scrolled away
//!                   +0x80 the Buy, Sell and Quit box, a choice box (`choices`)
//! ```
//!
//! Measured in Omega Ruby; X's shop is another (spike 196).
//!
//! Bound by:
//! - ADR-0043: a page jump says only the item it stops on.

use std::collections::HashSet;
use std::rc::Rc;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::FileMatching;
use crate::py::{self, Result};

pub const BUY: &str = "N5field16EventShopBuyCallE";
/// Where in the shop's object its Buy, Sell and Quit box lies.
pub const BOX: i64 = 0x80;
pub const ROW: i64 = 0x18;
pub const SCROLL: i64 = 0x1C;
pub const LIST: &str = "shop_lower.bclyt";

/// Holds the Mart's list's cursor back while a jump of a page scrolls the list.
pub struct Jumps {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    boxes: Rc<dyn Fn() -> Vec<i64>>,
    layouts: Rc<dyn TrackedLayouts>,
    files: Rc<dyn FileMatching>,
    index: Option<i64>,
    /// The list's cursor, while a jump is held back.
    held: Option<i64>,
}

impl Jumps {
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        boxes: Rc<dyn Fn() -> Vec<i64>>,
        layouts: Rc<dyn TrackedLayouts>,
        files: Rc<dyn FileMatching>,
    ) -> Self {
        Jumps {
            memory,
            names,
            boxes,
            layouts,
            files,
            index: None,
            held: None,
        }
    }

    /// The cursors' poll, less the list's cursor's move while a jump scrolls the list, and
    /// with it the poll the list holds still.
    pub fn poll(&mut self, cursors: CursorPoll) -> Result<CursorPoll> {
        let roots = self.list_roots(&cursors);
        let call = if roots.is_empty() { None } else { self.call()? };
        let Some(call) = call else {
            self.index = None;
            self.held = None;
            return Ok(cursors);
        };
        let row = *self
            .memory
            .read(call + ROW, 1)?
            .first()
            .ok_or_else(|| py::Error::index("index out of range"))? as i64;
        let index = row + u32(&self.memory.read(call + SCROLL, 4)?, 0)?;
        let last = self.index.replace(index);
        let moved: Vec<CursorItem> = cursors
            .items
            .iter()
            .filter(|item| {
                item.pieces
                    .first()
                    .is_some_and(|piece| roots.contains(&piece.root))
            })
            .cloned()
            .collect();
        if let Some(last) = last
            && (index - last).abs() > 1
        {
            // A page on, or still on the way: what is under the cursor is passing.
            self.held = moved.first().map(|item| item.cursor).or(self.held);
            let items = cursors
                .items
                .iter()
                .filter(|item| !moved.contains(item))
                .cloned()
                .collect();
            return Ok(CursorPoll { items, ..cursors });
        }
        let Some(held) = self.held.filter(|_| Some(index) == last) else {
            return Ok(cursors);
        };
        // Still for a poll: the item landed on, in the screen's words, unless the cursor says it.
        self.held = None;
        if !moved.is_empty() {
            return Ok(cursors);
        }
        let under = cursors.under.iter().find(|pieces| {
            pieces
                .first()
                .is_some_and(|piece| roots.contains(&piece.root))
        });
        let Some(under) = under.cloned() else {
            return Ok(cursors);
        };
        let mut items = cursors.items.clone();
        items.push(CursorItem::new(held, under, false));
        Ok(CursorPoll { items, ..cursors })
    }

    /// The roots of the Mart's list's layout a cursor is over.
    fn list_roots(&self, cursors: &CursorPoll) -> HashSet<i64> {
        let roots: HashSet<i64> = cursors
            .under
            .iter()
            .filter_map(|pieces| pieces.first().map(|piece| piece.root))
            .collect();
        roots
            .into_iter()
            .filter(|&root| {
                self.files
                    .matching(&self.layouts.pane_names(root))
                    .is_some_and(|found| found.name == LIST)
            })
            .collect()
    }

    /// The shop's object, the one whose Buy, Sell and Quit box the choice reader found.
    fn call(&self) -> Result<Option<i64>> {
        for r#box in (self.boxes)() {
            if self.names.of_object(r#box - BOX)?.as_deref() == Some(BUY) {
                return Ok(Some(r#box - BOX));
            }
        }
        Ok(None)
    }
}
