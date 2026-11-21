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
use crate::py::Result;

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
/// The button under every page, Check Moves or Check Ribbons, said after the page turned to.
pub const BUTTON: &str = "touch_bar_lower.bclyt";
/// The Pokémon's name, level and item, on every page.
pub const INFO: &str = "info.bclyt";
/// The conditions page's layout, which draws the conditions' star and its labels.
pub const STAR_PAGE: &str = "params_contest.bclyt";
/// The summary's process -> the place of the Pokémon shown in its party or box, from 0: it
/// changes at every switch, where `SHOWN` points at one copy of each in turn (spike G).
pub const PLACE: i64 = 0x7C;
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
/// code at another place each time the summary opens, never compared with one: each page's
/// layouts are those first on screen while it is shown (spike G). The other pages' words are
/// then left unheard (`hidden`), and a turn says the page turned to as a screen opening is read
/// (`turned`). Their layouts stay read as ever: put away, they left and came back, and the stats
/// came back into Check Moves' details with each move's. Check Moves, a cursor on a page's moves,
/// flips the page under the player between the moves and their contest kinds (spike B): each
/// move is heard with the page shown's words, and no flip is a turn. Nor is a page changed with
/// the Pokémon: an Egg turns the summary to its memo, and the next Pokémon back to its stats,
/// and the Pokémon is said as a switch is. A page is drawn afresh as it is turned to, a poll on:
/// read at once, Pikachu's turned to after a switch said "Type: Steel", the Pokémon's before,
/// then "Type: Electric"; so a turn hears no page for a poll, and the page turned to whole after
/// it.
pub struct Pages {
    memory: Rc<dyn Memory>,
    apps: Rc<RefCell<RunningApp>>,
    layouts: Rc<dyn TrackedLayouts>,
    files: Rc<dyn FileMatching>,
    /// Page value -> the works of its layouts.
    pages: IndexMap<i64, HashSet<i64>>,
    page: Option<i64>,
    /// The place of the Pokémon shown at the last poll (`PLACE`).
    shown: Option<i64>,
    /// Polls since the page last turned.
    since: i64,
    /// Polls since the page last changed with the Pokémon.
    paged: i64,
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
            shown: None,
            since: 2,
            paged: 2,
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
        self.since += 1;
        self.paged += 1;
        let Some(process) = process else {
            self.pages = IndexMap::new();
            self.page = None;
            self.shown = None;
            return Ok(());
        };
        let word = u32(&self.memory.read(process + PAGE, 4)?, 0)?;
        if !(LOADED_CODE.0 <= word && word < LOADED_CODE.1) {
            return Ok(());
        }
        let mon = u32(&self.memory.read(process + PLACE, 4)?, 0)?;
        let works = self.layouts.works();
        let moves = self
            .pages
            .values()
            .flatten()
            .any(|work| works.get(work).is_some_and(|root| cursored.contains(root)));
        if Some(word) != self.page
            && !moves
            && Some(mon) == self.shown
            && self.page.is_some_and(|page| self.pages.contains_key(&page))
            && self.pages.contains_key(&word)
        {
            self.since = 0;
        }
        if Some(word) != self.page && Some(mon) != self.shown && self.shown.is_some() {
            self.paged = 0;
        }
        self.page = Some(word);
        self.shown = Some(mon);
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

    /// The roots of the pages not shown, whose words are not heard; none while the page shown
    /// has no layouts of its own, and every page's the poll it turns, as it is drawn afresh.
    pub fn hidden(&self) -> HashSet<i64> {
        let mine = self.page.and_then(|page| self.pages.get(&page));
        let Some(mine) = mine.filter(|mine| !mine.is_empty()) else {
            return HashSet::new();
        };
        let works = self.layouts.works();
        if self.since == 0 {
            // The button too, said after the page once it is drawn.
            let mut roots: HashSet<i64> = self
                .pages
                .values()
                .flatten()
                .filter_map(|work| works.get(work).copied())
                .collect();
            roots.extend(self.of(BUTTON));
            return roots;
        }
        self.pages
            .values()
            .flatten()
            .filter(|work| !mine.contains(work))
            .filter_map(|work| works.get(work).copied())
            .collect()
    }

    /// The roots of the page turned to a poll ago, drawn afresh since, or none: its words are
    /// said as a screen opening's are, the first cutting off the page before
    /// (`ScreenAnnouncer.say`'s `lead`).
    pub fn turned(&self) -> HashSet<i64> {
        let Some(page) = self.page.filter(|_| self.since == 1) else {
            return HashSet::new();
        };
        let works = self.layouts.works();
        let mut roots: HashSet<i64> = self
            .pages
            .get(&page)
            .into_iter()
            .flatten()
            .filter_map(|work| works.get(work).copied())
            .collect();
        roots.extend(self.of(BUTTON));
        roots
    }

    /// The roots of the page shown and the Pokémon's name where the page changed with the
    /// Pokémon at the last poll or the one before, or none: an Egg's memo, or the stats of the
    /// Pokémon after an Egg, whose layouts come back to the screen a poll later, said as a switch
    /// is, the name first. Leaving an Egg said the stats page, then "Mario, Lv.66, None", the name
    /// twice.
    pub fn switched(&self) -> HashSet<i64> {
        let Some(page) = self.page.filter(|_| self.paged <= 1) else {
            return HashSet::new();
        };
        let works = self.layouts.works();
        let mut roots: HashSet<i64> = self
            .pages
            .get(&page)
            .into_iter()
            .flatten()
            .filter_map(|work| works.get(work).copied())
            .collect();
        roots.extend(self.of(INFO));
        roots.extend(self.of(BUTTON));
        roots
    }

    /// The roots of the layouts of this file.
    fn of(&self, name: &str) -> HashSet<i64> {
        self.layouts
            .works()
            .values()
            .copied()
            .filter(|&root| {
                self.files
                    .matching(&self.layouts.pane_names(root))
                    .is_some_and(|found| found.name == name)
            })
            .collect()
    }

    /// The TextBoxes of the conditions star's labels, "Coolness" to "Toughness": a label
    /// with no value, said as a list on the conditions page, "Coolness", then "Beauty,
    /// Cleverness, Cuteness, Toughness". Never said; the conditions Extra says them with their
    /// values (`oras.screens.conditions`).
    pub fn star_labels(&self, on_screen: &[ScreenText], words: &HashSet<String>) -> HashSet<i64> {
        let mut files: IndexMap<i64, String> = IndexMap::new();
        let mut boxes: HashSet<i64> = HashSet::new();
        for piece in on_screen {
            if !words.contains(&piece.text) {
                continue;
            }
            if !files.contains_key(&piece.root) {
                let found = self.files.matching(&self.layouts.pane_names(piece.root));
                files.insert(
                    piece.root,
                    found.map(|found| found.name.clone()).unwrap_or_default(),
                );
            }
            if files[&piece.root] == STAR_PAGE {
                boxes.insert(piece.r#box);
            }
        }
        boxes
    }
}
