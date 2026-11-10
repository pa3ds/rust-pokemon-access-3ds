//! The key items registered to the Y button's menu, each with the way it is chosen (spike B).
//!
//! Y on the field opens four slots, each an item's icon, one used by each way of the circle pad or
//! D-pad, and in Omega Ruby writes one line, "SELECT/START Switch Buttons", where X writes no words
//! at all: the icons are textures filled in as the game runs, and the menu names none of them
//! (Orion, or_registered_items_menu, x_registered_items_menu). The save's Bag holds what they
//! draw, in both:
//!
//! ```text
//! savedata::BAG  +0x0E  four u16 item numbers: up, right, down, left
//! ```
//!
//! The layout's four icons stand in that order, `p`, `j`, `l` and `n`, and it is told from every
//! other by the first of them drawing `item_icon.bclim`. It is among the layouts tracked only while
//! the menu is open: in both games each press of Y brought it and took it away.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u16;
use crate::core::memory::Memory;
use crate::core::words::own;
use crate::games::common::layout_file::LayoutFile;
use crate::py::Result;

pub const BAG: &str = "N8savedata3BAGE";
/// The way each slot of the Bag's four is chosen by, in the order it keeps them.
pub const WAYS: [&str; 4] = ["up", "right", "down", "left"];
pub const ICON_PANE: &str = "p";
/// The layout's first slot and what it draws: what tells the menu from every other layout.
pub const ICON: [&str; 1] = ["item_icon.bclim"];

/// Says what each way of the Y button's menu uses, as the menu opens.
pub struct Registered {
    memory: Rc<dyn Memory>,
    slots: i64,
    items: Vec<String>,
    file_of: Rc<dyn Fn(i64) -> Option<Rc<LayoutFile>>>,
    bag: Option<i64>,
    open: bool,
}

impl Registered {
    pub fn new(
        memory: Rc<dyn Memory>,
        slots: i64,
        items: Vec<String>,
        file_of: Rc<dyn Fn(i64) -> Option<Rc<LayoutFile>>>,
    ) -> Self {
        Registered {
            memory,
            slots,
            items,
            file_of,
            bag: None,
            open: false,
        }
    }

    /// The save's Bag, among what a sweep found by class.
    pub fn track(&mut self, found: &IndexMap<i64, String>) {
        for (&at, name) in found {
            if name == BAG {
                self.bag = Some(at);
            }
        }
    }

    /// Each registered item with its way, when the menu has just opened; else None. `roots`
    /// are the layouts tracked now.
    pub fn poll(&mut self, roots: impl IntoIterator<Item = i64>) -> Result<Option<String>> {
        let was = self.open;
        self.open = roots.into_iter().any(|root| self.menu(root));
        let Some(bag) = self.bag else {
            return Ok(None);
        };
        if was || !self.open {
            return Ok(None);
        }
        let data = self.memory.read(bag + self.slots, 2 * WAYS.len() as i64)?;
        let mut said: Vec<String> = Vec::new();
        for (i, way) in WAYS.iter().enumerate() {
            let number = u16(&data, 2 * i as i64)?;
            if 0 < number && number < self.items.len() as i64 {
                let way = own(&format!("registered-{way}"), &[]);
                said.push(own(
                    "registered-slot",
                    &[("way", &way), ("item", &self.items[number as usize])],
                ));
            }
        }
        let said = said.join(", ");
        Ok(if said.is_empty() { None } else { Some(said) })
    }

    fn menu(&self, root: i64) -> bool {
        let file = (self.file_of)(root);
        file.is_some_and(|file| {
            file.textures
                .get(ICON_PANE)
                .is_some_and(|drawn| *drawn == ICON)
        })
    }
}
