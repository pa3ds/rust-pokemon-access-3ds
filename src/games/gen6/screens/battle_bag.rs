//! The battle's Bag: a pocket's page said as it turns, before its item (spike 305).
//!
//! The Bag's list shows six items a page under a title, the pocket and the page, "HP/PP RESTORE
//! 2/2". Read as screen text, the title was said as details after every move, a page turned to
//! came after its first item, and a page turned back to read its other items as lines of their
//! own, "Hyper Potion 3", "Elixir 1", "Rage Candy Bar 2" (or_bag_and_battle_bag). The list keeps
//! its page, reached from the battle's view:
//!
//! ```text
//! a gauge                  +0x04  -> btl::BattleViewSystem (spike 298)
//! btl::BattleViewSystem   +0x190  -> a block of its UI, whose
//!                         +0x1D8  is btl::BtlvUiBag, inside it (spike 301); +0x1D4 in
//!                                    X and Y (spike 305)
//! btl::BtlvUiBag           +0x18  -> btl::app::ItemList while a pocket is open, its
//!                                    pockets' menu (btl::app::ItemCategory) before
//! btl::app::ItemList       +0xD4  u8 the pocket: 0 HP/PP RESTORE, 1 STATUS RESTORE,
//!                                    2 POKÉ BALLS, 3 BATTLE ITEMS
//!                         +0x30C  u8 the page shown, from 0
//! ```
//!
//! The page is said the poll its byte changes, first, and the item after it (the announcer's
//! `title`), as the player chose: "page first, then the item, but only if the page actually
//! changes". The title's words came with the byte, and once a poll after it, so the page's number
//! is the byte's and the rest the title's. As a pocket opens its title is said once after its
//! first item, as it was. The list's words are its cursor's alone. Turning to a page with fewer
//! items put the cursor for a poll over the pockets' menu behind the list, all its buttons one
//! line, "HP/PP RESTORE POKÉ BALLS STATUS RESTORE BATTLE ITEMS LAST USED ITEM": while a pocket is
//! open, the menu is no cursor's. An item's menu open over the list (`btl::app::ItemInfo` at the
//! Bag's +0x18) leaves the list as it was: its words the cursor's still, and backing out of it no
//! pocket opened.
//!
//! Bound by:
//! - ADR-0013: a cursor source says which item and page are shown, the words still the screen's;
//!   its objects reached from the battle's and checked by their classes, as ADR-0003 has every
//!   structure found, only the offsets fixed.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::{FileMatching, FilesOnScreen};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::anchors::Kept;
use crate::py::{self, Error, Result};

pub const VIEW: i64 = 0x04;
pub const VIEW_CLASS: &str = "N3btl16BattleViewSystemE";
pub const UI: i64 = 0x190;
/// Where in the UI block the Bag is: Omega Ruby and Alpha Sapphire's, then X and Y's.
pub const BAGS: [i64; 2] = [0x1D8, 0x1D4];
pub const BAG_CLASS: &str = "N3btl9BtlvUiBagE";
pub const LIST: i64 = 0x18;
pub const LIST_CLASS: &str = "N3btl3app8ItemListE";
/// What the Bag's +0x18 is while an item's menu, USE, is open over the list (spike 305).
pub const INFO_CLASS: &str = "N3btl3app8ItemInfoE";
pub const POCKET: i64 = 0xD4;
pub const PAGE: i64 = 0x30C;
/// Where the battle's objects are: its UI's lie past the application heap (spike 305).
pub const OBJECTS: (i64, i64) = (0x08000000, 0x20000000);
pub const LIST_FILE: &str = "item_list.bclyt";
pub const POCKETS_FILE: &str = "item_category.bclyt";
/// The list's title's TextBoxes by name, the pocket, the slash and how many pages; the page is
/// `1o`, and the byte's here.
pub const TITLE: [&str; 3] = ["1k", "1q", "1s"];

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Heard {
    /// The page turned to, said first, its item after it.
    pub turned: Option<String>,
    /// The page of a pocket just opened, said after its first item.
    pub opened: Option<String>,
    /// The list's TextBoxes, its cursor's to say.
    pub unsaid: HashSet<i64>,
    /// The roots of the pockets' menu while a pocket is open, under no cursor.
    pub pockets: HashSet<i64>,
}

/// `NOTHING`: nothing turned or opened, and nothing of the screen the Bag's.
pub fn nothing() -> Heard {
    Heard::default()
}

/// The battle Bag's page, and what of its list the screen's reading leaves unsaid.
pub struct BattleBag {
    memory: Rc<dyn Memory>,
    process: Rc<dyn Fn() -> Option<i64>>,
    layouts: Rc<dyn TrackedLayouts>,
    gauges: Rc<dyn Fn() -> Vec<i64>>,
    file: FilesOnScreen,
    /// The battle's classes, named afresh for each battle.
    kept: Kept,
    /// The list, its pocket and its page at the last poll its words were on screen.
    shown: Option<(i64, i64, i64)>,
}

impl BattleBag {
    /// `process` is the app's process, `gauges` the battle's HP gauges kept.
    pub fn new(
        memory: Rc<dyn Memory>,
        process: Rc<dyn Fn() -> Option<i64>>,
        layouts: Rc<dyn TrackedLayouts>,
        files: Rc<dyn FileMatching>,
        gauges: Rc<dyn Fn() -> Vec<i64>>,
    ) -> Self {
        BattleBag {
            kept: Kept::of(memory.clone(), process.clone()),
            file: FilesOnScreen::new(layouts.clone(), files.clone()),
            memory,
            process,
            layouts,
            gauges,
            shown: None,
        }
    }

    pub fn poll(&mut self, on_screen: &[ScreenText]) -> Result<Heard> {
        let files = self
            .file
            .poll(on_screen.iter().map(|piece| piece.root))
            .clone();
        let lists: HashSet<i64> = files
            .iter()
            .filter(|(_, name)| *name == LIST_FILE)
            .map(|(&root, _)| root)
            .collect();
        let shown = if lists.is_empty() { None } else { self.list()? };
        let Some(shown) = shown else {
            self.shown = None;
            return Ok(nothing());
        };
        let words: Vec<ScreenText> = on_screen
            .iter()
            .filter(|piece| lists.contains(&piece.root))
            .cloned()
            .collect();
        let unsaid: HashSet<i64> = words.iter().map(|piece| piece.r#box).collect();
        let pockets: HashSet<i64> = files
            .iter()
            .filter(|(_, name)| *name == POCKETS_FILE)
            .map(|(&root, _)| root)
            .collect();
        let shown = match shown {
            Listed::List(shown) => shown,
            Listed::Info => {
                // An item's menu open over the list: its words are still the cursor's, and the
                // menu no turn. An item chosen the poll its list showed read the whole list as
                // the menu opened, "Super Potion", a row of empty slots, and its page
                // (x_route_9_rhyhorn).
                return Ok(Heard {
                    turned: None,
                    opened: None,
                    unsaid,
                    pockets,
                });
            }
        };
        let last = self.shown.replace(shown);
        let title = self.title(&words, shown.2)?;
        let (mut turned, mut opened) = (None, None);
        if let Some(last) = last
            && (last.0, last.1) == (shown.0, shown.1)
        {
            turned = if last.2 != shown.2 { title } else { None };
        } else if title.is_some() {
            opened = title;
        } else {
            // Its words not written yet: opened at the next poll.
            self.shown = None;
        }
        Ok(Heard {
            turned,
            opened,
            unsaid,
            pockets,
        })
    }

    /// The open pocket's list, its pocket and its page; the item's menu open over it, its
    /// address alone; or None while neither is.
    fn list(&mut self) -> Result<Option<Listed>> {
        let process = (self.process)();
        if process.is_none() {
            return Ok(None);
        }
        let names = self.kept.names();
        let memory = &*self.memory;
        let Some(ui) = ui_block(memory, &names, &(self.gauges)())? else {
            return Ok(None);
        };
        let mut bag = None;
        for at in BAGS {
            if names.of_object(ui + at)?.as_deref() == Some(BAG_CLASS) {
                bag = Some(ui + at);
                break;
            }
        }
        let Some(bag) = bag else {
            return Ok(None);
        };
        let items = u32(&memory.read(bag + LIST, 4)?, 0)?;
        let kind = if within(items) {
            names.of_object(items)?
        } else {
            None
        };
        if kind.as_deref() == Some(INFO_CLASS) {
            return Ok(Some(Listed::Info));
        }
        if kind.as_deref() != Some(LIST_CLASS) {
            return Ok(None);
        }
        let byte = |at: i64| -> Result<i64> {
            Ok(*memory
                .read(at, 1)?
                .first()
                .ok_or_else(|| Error::index("index out of range"))? as i64)
        };
        let pocket = byte(items + POCKET)?;
        let page = byte(items + PAGE)?;
        Ok(Some(Listed::List((items, pocket, page))))
    }

    /// The pocket and the page, "HP/PP RESTORE 2/2", or None while its words are not all on
    /// screen.
    fn title(&self, words: &[ScreenText], page: i64) -> Result<Option<String>> {
        let mut named: IndexMap<String, String> = IndexMap::new();
        for piece in words {
            named.insert(name(&self.layouts.name(piece.r#box)?), piece.text.clone());
        }
        if !TITLE
            .iter()
            .all(|r#box| named.get(*r#box).is_some_and(|text| !text.is_empty()))
        {
            return Ok(None);
        }
        let [pocket, slash, pages] = TITLE.map(|r#box| named[r#box].clone());
        Ok(Some(format!("{pocket} {}{slash}{pages}", page + 1)))
    }
}

/// The battle view's block of its UI, through the view a gauge points at, or None.
pub fn ui_block(memory: &dyn Memory, names: &ClassNames, gauges: &[i64]) -> Result<Option<i64>> {
    // A sweep can take a word equal to a gauge's vtable for a gauge: the first kept read
    // 0xe58d8004 at +0x04 (spike 305). The view is the one a gauge points at.
    let mut view = None;
    for &gauge in gauges {
        let at = u32(&memory.read(gauge + VIEW, 4)?, 0)?;
        if within(at) && names.of_object(at)?.as_deref() == Some(VIEW_CLASS) {
            view = Some(at);
            break;
        }
    }
    let Some(view) = view else {
        return Ok(None);
    };
    let ui = u32(&memory.read(view + UI, 4)?, 0)?;
    Ok(if within(ui) { Some(ui) } else { None })
}

/// What `list` found: Python's `tuple[int, int, int] | int`.
enum Listed {
    /// The open pocket's list, its pocket and its page.
    List((i64, i64, i64)),
    /// The item's menu open over the list (the Python returns its address, which nothing reads).
    Info,
}

fn within(address: i64) -> bool {
    OBJECTS.0 <= address && address < OBJECTS.1
}

/// A pane's name: its bytes up to the first zero.
fn name(raw: &[u8]) -> String {
    let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
    py::decode_ascii(&raw[..end])
}
