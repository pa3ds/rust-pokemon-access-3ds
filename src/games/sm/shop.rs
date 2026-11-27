//! A Poké Mart's list in Sun and the Ultra games: the item under the cursor, by its own number.
//!
//! The list is one layout whose rows the generic cursor reader reads whole, "Poké Ball Potion
//! Antidote ... $200 $200 ... In Bag: 10", moving through it unsaid. The shop's own object knows
//! the item chosen (spike P, a Pokémon Center's Mart on Route 1, moved down and up a row at a
//! time):
//!
//! ```text
//! App::Shop::ShopDraw  +0xc4 u16 the item under the cursor, its number in the item names (system
//!                      text file 36, Sun's numbering): 4 Poké Ball, 17 Potion, 18 Antidote,
//!                      22 Paralyze Heal
//! App::Shop::ShopFrame +0x7c u16 the row, from 0
//! ```
//!
//! Sun's keeps the item at the same +0xc4 (Sun's Route 1 Mart, driven live to it through Iki
//! Town's festival: 4, 17, 18, 22 down the list and 18 back up). The version layout says where
//! (`sm.VersionLayout`), and a version whose field is not measured is not read.
//!
//! Bound by:
//! - ADR-0043: a screen's cursor is read from its own objects.

use std::rc::Rc;

use crate::core::binary::u16;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::announce::Focus;
use crate::games::sm::text::SunText;
use crate::py::Result;

pub const CLASS: &str = "N3App4Shop8ShopDrawE";
/// System text file 36, the items' names by number, in Sun's numbering.
pub const ITEM_NAMES: i64 = 36;
/// The start of the names of the list's rows' TextBoxes, `Text_ItemName_00` on: while they are on
/// screen the list is up, and `ShopDraw`, which only a sweep finds, is wanted.
pub const LIST_PANE: &[u8] = b"Text_ItemName";

/// The item under a Poké Mart list's cursor, said as it changes.
pub struct ShopReader {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    text: Option<Rc<SunText>>,
    item: Option<i64>,
    draws: Vec<i64>,
    /// The draw and the item said last.
    shown: Option<(i64, i64)>,
}

impl ShopReader {
    /// `item` is where `ShopDraw` keeps the item under the cursor, None where it is not
    /// known; `text` holds the item names, without which nothing is said.
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        text: Option<Rc<SunText>>,
        item: Option<i64>,
    ) -> Self {
        ShopReader {
            memory,
            names,
            text,
            item,
            draws: Vec::new(),
            shown: None,
        }
    }

    /// Python's `adding` defaults to False.
    pub fn track(&mut self, draws: impl IntoIterator<Item = i64>, adding: bool) {
        self.draws = if adding {
            let mut kept = self.draws.clone();
            kept.extend(draws.into_iter().filter(|d| !self.draws.contains(d)));
            kept
        } else {
            draws.into_iter().collect()
        };
    }

    /// Whether this version's shop is read here at all.
    pub fn reads(&self) -> bool {
        self.item.is_some() && self.text.is_some()
    }

    /// The item said last while the list is up, None while it is not.
    pub fn showing(&self) -> Option<String> {
        let (Some(shown), Some(text)) = (self.shown, &self.text) else {
            return None;
        };
        text.line(ITEM_NAMES, shown.1)
    }

    /// The item the cursor has come to, if it has moved or the list just opened.
    pub fn poll(&mut self) -> Result<Option<Focus>> {
        let (Some(item), Some(text)) = (self.item, self.text.clone()) else {
            return Ok(None);
        };
        let mut draw = None;
        for &d in &self.draws {
            if self.names.of_object(d)?.as_deref() == Some(CLASS) {
                draw = Some(d);
                break;
            }
        }
        let Some(draw) = draw else {
            self.shown = None;
            return Ok(None);
        };
        let number = u16(&self.memory.read(draw + item, 2)?, 0)?;
        let Some(name) = text.line(ITEM_NAMES, number) else {
            return Ok(None);
        };
        let opened = self.shown.is_none_or(|(shown, _)| shown != draw);
        if !opened && self.shown == Some((draw, number)) {
            return Ok(None);
        }
        self.shown = Some((draw, number));
        // Its place in the list is only logged (`Announcements.focus`), so it is not read.
        Ok(Some(Focus {
            source: draw,
            label: name,
            index: 0,
            count: 0,
            opened,
        }))
    }
}
