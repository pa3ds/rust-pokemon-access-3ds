//! A shiny Pokémon said so under a battle's cursors: a move's target, and the party (spike K).
//!
//! A battle says a move's target by the name its button shows, a horde's five "Zigzagoon" alike
//! (or_horde_battle), and the party by each member's row, "Groudon L45 156/156", the shiny ones
//! among them too, which the game draws sparkling and never names. The battle keeps the target
//! under the cursor as a mask of battle positions, and each HP gauge its position and its Pokémon,
//! whose record says whether it is shiny:
//!
//! ```text
//! btl::BattleViewSystem  +0x190 -> a block of its UI (`battle_bag.ui_block`)
//! the block               +0x54  u8 a bit a battle position the cursor is on, 0 on "return"
//! a gauge                 +0x14  u32 its position (`battle.GAUGE_SIDE`), the player's 0
//! ```
//!
//! The battle reader keeps the Pokémon each gauge last showed (`BattleReader.shiny`). Only where a
//! target cursor moves onto the name of a shiny one is the mask read, so that of five alike the one
//! under the cursor is said "Shiny Zigzagoon", and the others by their names. The party's list in
//! battle, `poke_list.bclyt`, says a row by its member's name first: while it shows, the party's
//! records are read once, through the party's own list (`partyicons`), and a shiny member's row is
//! said "Shiny Groudon L45 156/156". "Shiny" is a word of pa3ds's own, for what the game draws and
//! never names, a message in `data/` (`shiny-pokemon`).
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
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::FileMatching;
use crate::games::common::pokemon;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::{BATTLE, LOADED_CODE};
use crate::games::gen6::screens::battle_bag::ui_block;
use crate::games::gen6::screens::partyicons;
use crate::py::{self, Error, Result};

/// The view's UI block -> a mask of the battle positions the target cursor is on (spike K).
pub const TARGETS: i64 = 0x54;
/// The party's list in battle, a row a member, its name the row's first words (spike K).
pub const PARTY_FILE: &str = "poke_list.bclyt";

/// A battle's cursor items said "Shiny" where the Pokémon under them is.
pub struct Shiny {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    running: Rc<dyn Fn() -> Option<String>>,
    process: Rc<dyn Fn() -> Option<i64>>,
    gauges: Rc<dyn Fn() -> Vec<i64>>,
    party: Rc<dyn Fn() -> Option<i64>>,
    layouts: Rc<dyn TrackedLayouts>,
    files: Rc<dyn FileMatching>,
    battle_names: Option<(i64, ClassNames)>,
    /// Layout root -> its file's name, for the roots on screen in a battle.
    file: IndexMap<i64, String>,
    /// The party's shiny members' names, read as the party's list showed, until it goes.
    shiny_party: Option<HashSet<String>>,
}

impl Shiny {
    /// `targets`, the battle's shiny Pokémon by position, is handed to `poll`, as the battle
    /// reader is the profile's; `gauges` is its HP gauges kept, `party` the party's own list,
    /// `names` the executable's classes, the records' too.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        running: Rc<dyn Fn() -> Option<String>>,
        process: Rc<dyn Fn() -> Option<i64>>,
        gauges: Rc<dyn Fn() -> Vec<i64>>,
        party: Rc<dyn Fn() -> Option<i64>>,
        layouts: Rc<dyn TrackedLayouts>,
        files: Rc<dyn FileMatching>,
    ) -> Self {
        Shiny {
            memory,
            names,
            running,
            process,
            gauges,
            party,
            layouts,
            files,
            battle_names: None,
            file: IndexMap::new(),
            shiny_party: None,
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
            self.shiny_party = None;
            return Ok(cursors);
        }
        let lists = self.party_lists(on_screen);
        if lists.is_empty() {
            self.shiny_party = None;
        } else if self.shiny_party.is_none() {
            self.shiny_party = Some(self.read_party()?);
        }
        if cursors.items.is_empty() {
            return Ok(cursors);
        }
        let party = self.shiny_party.clone().unwrap_or_default();
        let targets = targets();
        let mut items: Vec<CursorItem> = cursors
            .items
            .iter()
            .map(|item| {
                if item
                    .pieces
                    .first()
                    .is_some_and(|first| lists.contains(&first.root))
                    && party.contains(&name(item))
                {
                    shiny(item)
                } else {
                    item.clone()
                }
            })
            .collect();
        let names: HashSet<String> = targets.values().map(|mon| mon.nickname.clone()).collect();
        let mut any = false;
        for item in &items {
            if names.contains(&name(item)) && !lists.contains(&first(item)?.root) {
                any = true;
                break;
            }
        }
        if any {
            let position = self.position()?;
            let mon = position.and_then(|position| targets.get(&position));
            if let Some(mon) = mon {
                let mut shown: Vec<CursorItem> = Vec::new();
                for item in &items {
                    shown.push(
                        if name(item) == mon.nickname && !lists.contains(&first(item)?.root) {
                            shiny(item)
                        } else {
                            item.clone()
                        },
                    );
                }
                items = shown;
            }
        }
        Ok(if items == cursors.items {
            cursors
        } else {
            CursorPoll { items, ..cursors }
        })
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

    /// The names of the party's shiny members.
    fn read_party(&self) -> Result<HashSet<String>> {
        let Some(party) = (self.party)() else {
            return Ok(HashSet::new());
        };
        let count = *self
            .memory
            .read(party, partyicons::COUNT + 1)?
            .last()
            .ok_or_else(|| Error::index("index out of range"))? as i64;
        let mut shiny: HashSet<String> = HashSet::new();
        for i in 0..count.min(partyicons::SLOTS) {
            let param = u32(&self.memory.read(party + 4 * i, 4)?, 0)?;
            let mon = pokemon::read(&*self.memory, &self.names, param, pokemon::SPECIES_MAX)?;
            if let Some(mon) = mon
                && mon.shiny
            {
                shiny.insert(mon.nickname);
            }
        }
        Ok(shiny)
    }

    /// The one battle position the target cursor is on, or None.
    fn position(&mut self) -> Result<Option<i64>> {
        let Some(process) = (self.process)() else {
            return Ok(None);
        };
        // Named afresh for each battle: its classes are in a module, whose memory is the next
        // module's once it is unloaded.
        if self
            .battle_names
            .as_ref()
            .is_none_or(|names| names.0 != process)
        {
            self.battle_names = Some((process, ClassNames::new(self.memory.clone(), LOADED_CODE)));
        }
        let names = &self.battle_names.as_ref().expect("named above").1;
        let Some(ui) = ui_block(&*self.memory, names, &(self.gauges)())? else {
            return Ok(None);
        };
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
