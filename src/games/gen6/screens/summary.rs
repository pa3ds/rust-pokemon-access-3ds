//! A Pokémon's summary: which Pokémon it shows, by its process (spike 196), and which of its pages
//! (`Pages`).
//!
//! The summary's stats are on the top screen and built first, the Pokémon's name and level on the
//! bottom; a screen rewritten is led by the layout naming its Pokémon, said first where the game's
//! order put it in the middle (`ScreenAnnouncer.say`'s `subject`).
//!
//! An Egg's summary is its memo alone. Turned to from a Pokémon, the game writes the Egg's moves
//! into the move panel's boxes and stops drawing the panel, every flag of it still saying drawn:
//! its moves were read out (or_egg_summary). While an Egg is shown, its pages of a Pokémon's are
//! put away (`Layouts.put_away`), as the AreaNav's views are.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::layout::{Layouts, TrackedLayouts};
use crate::games::common::layout_file::{FileMatching, LayoutFiles};
use crate::games::common::pokemon;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::{LOADED_CODE, RunningApp};
use crate::py::{self, Result};

pub const SUMMARY: &str = "N3app6status7ProcessE";
/// The summary's process -> the `PokemonParam` it shows, a copy of the party member's record
/// (spike 196).
pub const SHOWN: i64 = 0x78;
/// The layout files of the summary's pages an Egg has none of: its stats and moves, their
/// contest kinds, and its ribbons.
pub const POKEMON_PAGES: [&str; 5] = [
    "params.bclyt",
    "waza.bclyt",
    "params_contest.bclyt",
    "waza_contest.bclyt",
    "ribbon.bclyt",
];
/// The layout files of the summary's pages, one page drawn at a time; not the Pokémon's name,
/// level and item, nor the button under them, which every page shows. A `frozenset` only tested.
pub const PAGES: [&str; 6] = [
    "params.bclyt",
    "waza.bclyt",
    "params_contest.bclyt",
    "waza_contest.bclyt",
    "ribbon.bclyt",
    "memo.bclyt",
];
/// The summary's process -> the code drawing the page shown: one value a page, changed at every
/// turn (spike B, or_summary_screen_weirdness).
pub const PAGE: i64 = 0x80;

/// The `PokemonParam` the summary shows, or None while no summary runs.
pub fn shown(memory: &dyn Memory, apps: &RunningApp) -> Result<Option<i64>> {
    if apps.running.as_deref() != Some(SUMMARY) {
        return Ok(None);
    }
    let Some(process) = apps.process else {
        return Ok(None);
    };
    Ok(Some(u32(&memory.read(process + SHOWN, 4)?, 0)?))
}

/// The name of the Pokémon the summary shows, its nickname or its species', or None.
pub fn name(memory: &dyn Memory, names: &ClassNames, apps: &RunningApp) -> Result<Option<String>> {
    let found = match shown(memory, apps)? {
        Some(param) if param != 0 => pokemon::read(memory, names, param, pokemon::SPECIES_MAX)?,
        _ => None,
    };
    Ok(found.map(|found| found.nickname))
}

/// The works of the pages of a Pokémon's while the summary shows an Egg, else none.
pub fn put_away(
    memory: &dyn Memory,
    names: &ClassNames,
    apps: &RunningApp,
    layouts: &Layouts,
    files: &LayoutFiles,
) -> Result<HashSet<i64>> {
    let mon = match shown(memory, apps)? {
        Some(param) if param != 0 => pokemon::read(memory, names, param, pokemon::SPECIES_MAX)?,
        _ => None,
    };
    if !mon.is_some_and(|mon| mon.egg) {
        return Ok(HashSet::new());
    }
    Ok(layouts
        .works()
        .into_iter()
        .filter(|&(_work, root)| {
            files
                .matching(&layouts.pane_names(root))
                .is_some_and(|file| POKEMON_PAGES.contains(&file.name.as_str()))
        })
        .map(|(work, _root)| work)
        .collect())
}

/// Which of the summary's pages is shown, and the words of the others, which it does not draw.
///
/// The summary keeps every page it has drawn on screen, every flag and place of it as when it
/// was drawn, and draws one: turned back to, a page said only its button, "Check Moves", its words
/// said once already, and switching Pokémon read every page's words, on the memo page too
/// (or_summary_screen_weirdness). Its process's `PAGE` changes at every turn, an address in the
/// code, never compared with one: each page's layouts are those first on screen while it is
/// shown. The other pages' words are then left unheard (`hidden`), and a turn says the page
/// turned to (`turned`). Their layouts stay read as ever: put away, they left and came back, and
/// the stats came back into Check Moves' details with each move's. While Check Moves is on, a
/// cursor on a page's moves, the page flips under the player between the moves and their contest
/// kinds (spike B), and every page is heard, as before.
pub struct Pages {
    memory: Rc<dyn Memory>,
    apps: Rc<RefCell<RunningApp>>,
    layouts: Rc<dyn TrackedLayouts>,
    files: Rc<dyn FileMatching>,
    /// Page value -> the works of its layouts.
    pages: IndexMap<i64, HashSet<i64>>,
    page: Option<i64>,
    /// Whether Check Moves is on.
    moves: bool,
    /// Whether the page turned at the last poll.
    turned: bool,
}

impl Pages {
    pub fn new(
        memory: Rc<dyn Memory>,
        apps: Rc<RefCell<RunningApp>>,
        layouts: Rc<dyn TrackedLayouts>,
        files: Rc<dyn FileMatching>,
    ) -> Self {
        Pages {
            memory,
            apps,
            layouts,
            files,
            pages: IndexMap::new(),
            page: None,
            moves: false,
            turned: false,
        }
    }

    /// Read the page shown, and take page layouts on screen for the first time as its.
    /// `cursored` is the roots of the layouts a cursor is over: a page's, Check Moves is on.
    pub fn poll(&mut self, on_screen: &[ScreenText], cursored: &HashSet<i64>) -> Result<()> {
        let process = {
            let apps = self.apps.borrow();
            if apps.running.as_deref() == Some(SUMMARY) {
                apps.process
            } else {
                None
            }
        };
        self.turned = false;
        let Some(process) = process else {
            self.pages = IndexMap::new();
            self.page = None;
            self.moves = false;
            return Ok(());
        };
        let word = u32(&self.memory.read(process + PAGE, 4)?, 0)?;
        if !(LOADED_CODE.0 <= word && word < LOADED_CODE.1) {
            return Ok(());
        }
        let works = self.layouts.works();
        self.moves = self
            .pages
            .values()
            .flatten()
            .any(|work| works.get(work).is_some_and(|root| cursored.contains(root)));
        self.turned = Some(word) != self.page
            && !self.moves
            && self.page.is_some_and(|page| self.pages.contains_key(&page))
            && self.pages.contains_key(&word);
        self.page = Some(word);
        let known: HashSet<i64> = self.pages.values().flatten().copied().collect();
        let roots: HashSet<i64> = on_screen.iter().map(|piece| piece.root).collect();
        for (&work, &root) in &works {
            if known.contains(&work) || !roots.contains(&root) {
                continue;
            }
            let file = self.files.matching(&self.layouts.pane_names(root));
            if file.is_some_and(|file| PAGES.contains(&file.name.as_str())) {
                self.pages.entry(word).or_default().insert(work);
            }
        }
        Ok(())
    }

    /// The roots of the pages not shown, whose words are not heard; none while Check Moves is
    /// on or the page shown has no layouts of its own.
    pub fn hidden(&self) -> HashSet<i64> {
        let mine = self.page.and_then(|page| self.pages.get(&page));
        let Some(mine) = mine.filter(|mine| !self.moves && !mine.is_empty()) else {
            return HashSet::new();
        };
        let works = self.layouts.works();
        self.pages
            .values()
            .flatten()
            .filter(|work| !mine.contains(work))
            .filter_map(|work| works.get(work).copied())
            .collect()
    }

    /// The roots of the page turned to at the last poll, or none: its words are news, said as
    /// a screen rewritten is, its first row at once and the rest after, a turn on cutting them
    /// off.
    pub fn turned(&self) -> HashSet<i64> {
        let Some(page) = self.page.filter(|_| self.turned) else {
            return HashSet::new();
        };
        let works = self.layouts.works();
        self.pages
            .get(&page)
            .into_iter()
            .flatten()
            .filter_map(|work| works.get(work).copied())
            .collect()
    }

    /// What leads what is said of the summary: on a page just turned to, its first line, the
    /// top screen's highest words; else the Pokémon's `name`, as switching Pokémon is. A rewrite
    /// is led by its first row that is no sentence, and the memo page, sentences, was led by its
    /// button, "Check Ribbons"; led by the name, both its first pages began "Name Metagross",
    /// and a Torchic named Blaze its stats page at its ability, "To Next Lv. Blaze"
    /// (or_menus).
    pub fn lead(&self, on_screen: &[ScreenText], name: Option<String>) -> Option<String> {
        let turned = self.turned();
        // `max(page, key=lambda p: (p.top, p.row))`: the first of equal bests, a NaN row never
        // greater.
        let mut best: Option<&ScreenText> = None;
        for piece in on_screen {
            if !turned.contains(&piece.root) || py::strip(&piece.text).is_empty() {
                continue;
            }
            let greater = match best {
                None => true,
                Some(best) if piece.top != best.top => piece.top > best.top,
                Some(best) => piece.row > best.row,
            };
            if greater {
                best = Some(piece);
            }
        }
        match best {
            Some(best) => Some(py::strip(&best.text).to_string()),
            None => name,
        }
    }
}
