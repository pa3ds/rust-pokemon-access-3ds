//! Lists: announce a Yes/No prompt, or any list window, as it opens and as its cursor moves.
//!
//! A list window is three objects (spikes 14, 18 and 20). `app::tool::MenuWindowController` is the
//! one a heap sweep reliably finds. Its drawer, an `app::tool::MenuWindowDrawer` such as
//! `AppMenuDrawer`, is at +0x08, and its `app::tool::MenuWindow` is at +0x0c — read through that
//! pointer, because a sweep can miss the window itself (spike 18).
//!
//! ```text
//! controller  +0x08 -> drawer   +0x0c -> window
//! drawer      +0x08 -> window   +0x0c -> controller   +0x20 + i * 0x18  u8 state of item i
//! window      +0x14 u32 width   +0x18 u32 height   +0x20 -> item records   +0x2c u32 cursor
//! item record 0x1c bytes, +0x14 -> gfl::str::StrBuf label
//! ```
//!
//! A window is built long before its prompt appears (spike 17), so finding one says nothing about
//! seeing one. The drawer's item states do: every item 0 while hidden, 2 on the item under the cursor
//! while the prompt is up, 6 on the item chosen (spike 20). A list is shown while its cursor's item
//! reads 2. The party menu's drawer has one item more than its window lists, and from RESTORE on
//! lights the item after the cursor's (or_touch_battle): the cursor reader says those, and the list
//! is still up for touch mode while any item reads 2.
//!
//! Bound by:
//! - ADR-0012: a list window says the item under its cursor as it opens, then each moved to.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::announce::Focus;
use crate::games::common::strbuf;
use crate::py::Result;

pub const CONTROLLER_CLASS: &str = "N3app4tool20MenuWindowControllerE";
/// A list window. Not a message window (`msgwin`) and not a layout pane (`lyt.WINDOW_CLASS`).
pub const MENU_WINDOW_CLASS: &str = "N3app4tool10MenuWindowE";
pub const DRAWER_CLASSES: [&str; 3] = [
    "N3app4tool16MenuWindowDrawerE",
    "N3app4tool13AppMenuDrawerE",
    "N3app4tool19KawaigariMenuDrawerE",
];

pub const CONTROLLER_DRAWER: i64 = 0x08;
pub const CONTROLLER_WINDOW: i64 = 0x0C;

pub const DRAWER_WINDOW: i64 = 0x08;
pub const DRAWER_CONTROLLER: i64 = 0x0C;
pub const DRAWER_ITEM_STATE: i64 = 0x20;
pub const DRAWER_ITEM_STRIDE: i64 = 0x18;
/// The state of the item under the cursor while the list is on screen.
pub const ITEM_HIGHLIGHTED: i64 = 2;

pub const WINDOW_WIDTH: i64 = 0x14;
pub const WINDOW_HEIGHT: i64 = 0x18;
pub const WINDOW_ITEMS: i64 = 0x20;
pub const WINDOW_CURSOR: i64 = 0x2C;
pub const WINDOW_HEAD: i64 = WINDOW_CURSOR + 4;

pub const ITEM_RECORD: i64 = 0x1C;
pub const ITEM_LABEL: i64 = 0x14;
/// The drawer holds its MenuCursor at +0x1a0, which leaves room for 16 item states before it.
pub const MAX_ITEMS: i64 = 16;

/// Tracks list windows through their controllers and reports what the cursor is on.
pub struct ListReader {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    active: Vec<i64>,
    /// Controller -> the index last announced, for lists currently on screen.
    shown: IndexMap<i64, i64>,
    /// Controller -> its labels, for lists currently on screen.
    labels: IndexMap<i64, Vec<String>>,
    /// Controller -> its labels, for lists on screen lit on any item.
    lit: IndexMap<i64, Vec<String>>,
}

impl ListReader {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        ListReader {
            memory,
            names,
            active: Vec::new(),
            shown: IndexMap::new(),
            labels: IndexMap::new(),
            lit: IndexMap::new(),
        }
    }

    /// Every label of every list up at the last poll, lit on its cursor's item or another:
    /// the lists touch mode answers first, and whose buttons it leaves out once they go.
    pub fn lit(&self) -> HashSet<String> {
        self.lit.values().flatten().cloned().collect()
    }

    /// Every label of every list on screen at the last poll.
    ///
    /// A list's labels are TextBoxes in a layout of its own, so the screen text reader sees them
    /// too. Leaving these out there is what keeps a prompt's options from being heard all at once
    /// as well as one at a time under the cursor.
    pub fn labels(&self) -> HashSet<String> {
        self.labels.values().flatten().cloned().collect()
    }

    /// Read these controllers from now on, as well as every one found before.
    ///
    /// A prompt gets a new controller the moment it appears, and a heap sweep every couple of
    /// seconds finds it late or not at all: across movie2 at 10x only the starter choice, which
    /// the player took time over, was announced. The game builds its prompts at addresses it has
    /// used before, so an address once seen is read on every poll after, and a prompt rebuilt
    /// there is heard at once. Whatever else lives there fails the class and link checks.
    pub fn track(&mut self, controllers: impl IntoIterator<Item = i64>) {
        for controller in controllers {
            if !self.active.contains(&controller) {
                self.active.push(controller);
            }
        }
    }

    pub fn poll(&mut self) -> Result<Vec<Focus>> {
        let mut focus: Vec<Focus> = Vec::new();
        for &controller in &self.active {
            let seen = self.read(controller)?;
            let (labels, index) = seen.unwrap_or((Vec::new(), None));
            if !labels.is_empty() {
                self.lit.insert(controller, labels.clone());
            } else {
                self.lit.shift_remove(&controller);
            }
            let Some(index) = index else {
                self.shown.shift_remove(&controller);
                self.labels.shift_remove(&controller);
                continue;
            };
            let before = self.shown.get(&controller).copied();
            if before != Some(index) {
                focus.push(Focus {
                    source: controller,
                    label: labels[index as usize].clone(),
                    index,
                    count: labels.len() as i64,
                    opened: before.is_none(),
                });
            }
            self.shown.insert(controller, index);
            self.labels.insert(controller, labels);
        }
        Ok(focus)
    }

    /// (labels, cursor) for a list on screen, or None for anything else. Lit on an item other
    /// than its cursor's, a list lit before keeps its labels, unread, and has no cursor.
    fn read(&self, controller: i64) -> Result<Option<(Vec<String>, Option<i64>)>> {
        let head = self.memory.read(controller, CONTROLLER_WINDOW + 4)?;
        let (drawer, window) = (
            u32(&head, CONTROLLER_DRAWER)?,
            u32(&head, CONTROLLER_WINDOW)?,
        );
        if self.names.of_object(window)?.as_deref() != Some(MENU_WINDOW_CLASS) {
            return Ok(None);
        }
        let drawer_class = self.names.of_object(drawer)?;
        if !drawer_class
            .as_deref()
            .is_some_and(|class| DRAWER_CLASSES.contains(&class))
        {
            return Ok(None);
        }
        let states = self
            .memory
            .read(drawer, DRAWER_ITEM_STATE + DRAWER_ITEM_STRIDE * MAX_ITEMS)?;
        if u32(&states, DRAWER_WINDOW)? != window || u32(&states, DRAWER_CONTROLLER)? != controller
        {
            return Ok(None);
        }

        let body = self.memory.read(window, WINDOW_HEAD)?;
        // Python's int does not overflow: two u32 words from freed memory can multiply past an i64,
        // and any such product is far over `MAX_ITEMS` just as it is there.
        let count = u32(&body, WINDOW_WIDTH)?.saturating_mul(u32(&body, WINDOW_HEIGHT)?);
        let cursor = u32(&body, WINDOW_CURSOR)?;
        if !(0 < count && count <= MAX_ITEMS) || !(0 <= cursor && cursor < count) {
            return Ok(None);
        }
        if states[(DRAWER_ITEM_STATE + DRAWER_ITEM_STRIDE * cursor) as usize] as i64
            != ITEM_HIGHLIGHTED
        {
            let known = self.lit.get(&controller);
            let lit = (0..MAX_ITEMS).any(|i| {
                states[(DRAWER_ITEM_STATE + DRAWER_ITEM_STRIDE * i) as usize] as i64
                    == ITEM_HIGHLIGHTED
            });
            return Ok(match known {
                Some(known) if lit && known.len() as i64 == count => Some((known.clone(), None)),
                _ => None,
            });
        }

        let records = self
            .memory
            .read(u32(&body, WINDOW_ITEMS)?, ITEM_RECORD * count)?;
        let mut labels: Vec<String> = Vec::new();
        for i in 0..count {
            let label = strbuf::label(&*self.memory, u32(&records, i * ITEM_RECORD + ITEM_LABEL)?)?;
            let Some(label) = label else {
                return Ok(None);
            };
            labels.push(label);
        }
        Ok(Some((labels, Some(cursor))))
    }
}
