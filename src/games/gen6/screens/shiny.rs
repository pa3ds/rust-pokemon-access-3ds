//! A shiny Pokémon said so under a horde's target cursor (spike 307).
//!
//! A battle says a move's target by the name its button shows, a horde's five "Zigzagoon" alike
//! (or_horde_battle), the shiny one among them, which the game draws sparkling and never names. The
//! battle keeps the target under the cursor as a mask of battle positions, and each HP gauge its
//! position and its Pokémon, whose record says whether it is shiny:
//!
//! ```text
//! btl::BattleViewSystem  +0x190 -> a block of its UI (`battle_bag.ui_block`)
//! the block               +0x54  u8 a bit a battle position the cursor is on, 0 on "return"
//! the block               +0x84  -> the target screen, `btl::app::TargetSelect1vs5` a horde's
//! a gauge                 +0x14  u32 its position (`battle.GAUGE_SIDE`), the player's 0
//! ```
//!
//! The battle reader keeps the Pokémon each gauge last showed (`BattleReader.shiny`). Only where a
//! target cursor moves onto the name of a shiny one is the mask read, so that of five alike the one
//! under the cursor is said "Shiny Zigzagoon", and the others by their names. Only in a horde: the
//! word is for telling its one shiny from the rest, which a double or triple battle's few foes do
//! not need, and the player knows their own, which sparkles as it is sent out. The party's list in
//! battle, `poke_list.bclyt`, says a row by its member's name first, so its rows are never taken
//! for targets. "Shiny" is a word of pa3ds's own, for what the game draws and never names, a
//! message in `data/` (`shiny-pokemon`).
//!
//! Bound by:
//! - ADR-0013: a cursor source says which item is under the cursor, the words still the screen's;
//!   its objects reached from the battle's and checked by their classes, as ADR-0003 has every
//!   structure found, only the offsets fixed.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::words::own;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::FileMatching;
use crate::games::common::pokemon;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::anchors::Kept;
use crate::games::gen6::apps::BATTLE;
use crate::games::gen6::screens::battle_bag::ui_block;
use crate::py::{self, Error, Result};

/// The view's UI block -> a mask of the battle positions the target cursor is on (spike 307).
pub const TARGETS: i64 = 0x54;
/// The view's UI block -> the target screen shown (spike 307).
pub const SCREEN: i64 = 0x84;
/// A horde's target screen, the player's one Pokémon against five (spike 307).
pub const HORDE_CLASS: &str = "N3btl3app16TargetSelect1vs5E";
/// The party's list in battle, a row a member, its name the row's first words (spike 307).
pub const PARTY_FILE: &str = "poke_list.bclyt";

/// A horde's target cursor said "Shiny" where the Pokémon under it is.
pub struct Shiny {
    memory: Rc<dyn Memory>,
    running: Rc<dyn Fn() -> Option<String>>,
    process: Rc<dyn Fn() -> Option<i64>>,
    gauges: Rc<dyn Fn() -> Vec<i64>>,
    layouts: Rc<dyn TrackedLayouts>,
    files: Rc<dyn FileMatching>,
    /// The battle's classes, named afresh for each battle.
    kept: Kept,
    /// Layout root -> its file's name, for the roots on screen in a battle.
    file: IndexMap<i64, String>,
}

impl Shiny {
    /// `targets`, the battle's shiny Pokémon by position, is handed to `poll`, as the battle
    /// reader is the profile's; `gauges` is its HP gauges kept.
    pub fn new(
        memory: Rc<dyn Memory>,
        running: Rc<dyn Fn() -> Option<String>>,
        process: Rc<dyn Fn() -> Option<i64>>,
        gauges: Rc<dyn Fn() -> Vec<i64>>,
        layouts: Rc<dyn TrackedLayouts>,
        files: Rc<dyn FileMatching>,
    ) -> Self {
        Shiny {
            kept: Kept::of(memory.clone(), process.clone()),
            memory,
            running,
            process,
            gauges,
            layouts,
            files,
            file: IndexMap::new(),
        }
    }

    /// The cursors, an item over a shiny Pokémon's name said with "Shiny".
    pub fn poll(
        &mut self,
        cursors: CursorPoll,
        on_screen: &[ScreenText],
        targets: &dyn Fn() -> IndexMap<i64, pokemon::Pokemon>,
    ) -> Result<CursorPoll> {
        if (self.running)().as_deref() != Some(BATTLE) {
            self.file = IndexMap::new();
            return Ok(cursors);
        }
        let targets = targets();
        if cursors.items.is_empty() || targets.is_empty() {
            return Ok(cursors);
        }
        let names: HashSet<String> = targets.values().map(|mon| mon.nickname.clone()).collect();
        let lists = self.party_lists(on_screen);
        let mut any = false;
        for item in &cursors.items {
            if names.contains(&name(item)) && !lists.contains(&first(item)?.root) {
                any = true;
                break;
            }
        }
        if !any {
            return Ok(cursors);
        }
        let position = self.position()?;
        let Some(mon) = position.and_then(|position| targets.get(&position)) else {
            return Ok(cursors);
        };
        let mut items: Vec<CursorItem> = Vec::new();
        for item in &cursors.items {
            items.push(
                if name(item) == mon.nickname && !lists.contains(&first(item)?.root) {
                    shiny(item)
                } else {
                    item.clone()
                },
            );
        }
        Ok(CursorPoll { items, ..cursors })
    }

    /// The roots of the party's list on screen.
    fn party_lists(&mut self, on_screen: &[ScreenText]) -> HashSet<i64> {
        // A set Python iterates below; what is done for each root reads nothing.
        let roots: IndexSet<i64> = on_screen.iter().map(|piece| piece.root).collect();
        // A layout's file is known only once its archive has been read: a miss is asked again.
        self.file.retain(|root, _| roots.contains(root));
        for &root in &roots {
            if self.file.contains_key(&root) {
                continue;
            }
            if let Some(found) = self.files.matching(&self.layouts.pane_names(root)) {
                self.file.insert(root, found.name.clone());
            }
        }
        self.file
            .iter()
            .filter(|(_, name)| *name == PARTY_FILE)
            .map(|(&root, _)| root)
            .collect()
    }

    /// The one battle position a horde's target cursor is on, or None.
    fn position(&mut self) -> Result<Option<i64>> {
        if (self.process)().is_none() {
            return Ok(None);
        }
        let names = self.kept.names();
        let Some(ui) = ui_block(&*self.memory, &names, &(self.gauges)())? else {
            return Ok(None);
        };
        let screen = u32(&self.memory.read(ui + SCREEN, 4)?, 0)?;
        if names.of_object(screen)?.as_deref() != Some(HORDE_CLASS) {
            return Ok(None);
        }
        let mask = *self
            .memory
            .read(ui + TARGETS, 1)?
            .first()
            .ok_or_else(|| Error::index("index out of range"))? as i64;
        Ok(if mask != 0 && mask & (mask - 1) == 0 {
            Some(63 - i64::from(mask.leading_zeros()))
        } else {
            None
        })
    }
}

/// `item.pieces[0]`, raising where it has none, as Python's indexing does.
fn first(item: &CursorItem) -> Result<&ScreenText> {
    item.pieces
        .first()
        .ok_or_else(|| Error::index("tuple index out of range"))
}

fn name(item: &CursorItem) -> String {
    item.pieces
        .first()
        .map(|first| py::strip(&first.text).to_string())
        .unwrap_or_default()
}

fn shiny(item: &CursorItem) -> CursorItem {
    let mut pieces = item.pieces.clone();
    let named = name(item);
    pieces[0].text = own("shiny-pokemon", &[("name", &named)]);
    CursorItem {
        pieces,
        ..item.clone()
    }
}
