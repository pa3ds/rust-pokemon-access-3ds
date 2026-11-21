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
//!
//! Bound by:
//! - ADR-0043: a page is said as it is turned to, from its list's flag; a switch says the page
//!   shown.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{u16, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::FileMatching;
use crate::games::common::pokemon::{self, Pokemon};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::anchors::Kept;
use crate::games::gen6::apps::RunningApp;
use crate::py::{Error, Result};

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
/// The conditions star's outline (spike 196). A layout drawing it draws the star's labels too: the
/// conditions page's; the Move Reminder's contest effects' upper screen, its moves' category icons
/// beside the star, where the labels were read with what stood near them, "Black Glasses
/// Coolness", "Beauty", "Cuteness Cleverness", "Toughness" (or_move_reminder_bugs); and the
/// Pokéblock Case's upper screen as it asks whom to give a block to, "Coolness", "Toughness,
/// Beauty" (or_pokeblock_kit).
pub const STAR: &str = "star_base.bclim";
/// The pages' moves, which Check Moves' cursors go over. A `frozenset` only tested.
pub const MOVES: [&str; 2] = ["waza.bclyt", "waza_contest.bclyt"];
/// The summary's process -> the place of the Pokémon shown in its party or box, from 0: it
/// changes at every switch, where `SHOWN` points at one copy of each in turn (spike 303).
pub const PLACE: i64 = 0x7C;
/// One of `LISTS`: where the summary's process keeps a page's list, the list's class, the layout
/// files of its page, and whether it is a list of moves.
pub type PageList = (i64, &'static str, &'static [&'static str], bool);

/// The summary's process -> each page's list, of the Pokémon's moves, their contest kinds or its
/// ribbons, by its class; the layout files of that page, the stats page, the conditions page and
/// the memo; and whether it is a list of moves, which Check Moves chooses from (spike 304).
pub const LISTS: [PageList; 3] = [
    (
        0x5C,
        "N3app6status8WazaListE",
        &["params.bclyt", "waza.bclyt"],
        true,
    ),
    (
        0x64,
        "N3app6status15ContestWazaListE",
        &["params_contest.bclyt", "waza_contest.bclyt"],
        true,
    ),
    (
        0x60,
        "N3app6status10RibbonListE",
        &["memo.bclyt", "ribbon.bclyt"],
        false,
    ),
];
/// A page's list -> 1 while its page is shown, else 0 (spike 304).
pub const SHOWING: i64 = 0x4;
/// A list of moves -> the move under the Check Moves cursor, by its number, or 0. Check Moves ended
/// clears it in the shown page's list alone: ended on the conditions page, the stats page's list
/// keeps its last move. Turning the page in Check Moves gives the move to the other page's list, so
/// both hold it (spike 304).
pub const CHOSEN: i64 = 0x22;

/// Where a game's summary keeps the Pokémon it shows and its pages' lists.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shape {
    /// As `SHOWN`.
    pub shown: i64,
    /// As `PLACE`.
    pub place: i64,
    /// As `LISTS`.
    pub lists: &'static [PageList],
}

pub const ORAS: Shape = Shape {
    shown: SHOWN,
    place: PLACE,
    lists: &LISTS,
};
/// X's: two pages, the stats with the moves and the memo with the ribbons, and no contest
/// page; each 0x8 to 0x10 sooner than in Omega Ruby (spikes 298 and 303 on x_summary_screen).
pub const XY: Shape = Shape {
    shown: 0x68,
    place: 0x6C,
    lists: &[
        (
            0x54,
            "N3app6status8WazaListE",
            &["params.bclyt", "waza.bclyt"],
            true,
        ),
        (
            0x58,
            "N3app6status10RibbonListE",
            &["memo.bclyt", "ribbon.bclyt"],
            false,
        ),
    ],
};

/// The `PokemonParam` the summary shows, or None while no summary runs.
pub fn shown(memory: &dyn Memory, apps: &RunningApp, shape: &Shape) -> Result<Option<i64>> {
    if apps.running.as_deref() != Some(SUMMARY) {
        return Ok(None);
    }
    let Some(process) = apps.process else {
        return Ok(None);
    };
    Ok(Some(u32(&memory.read(process + shape.shown, 4)?, 0)?))
}

/// The pieces with the rest of the pane their words hang from, in its order: a cursor's item
/// whose other words lay under another layout's row. The contest moves' kind and name hang from
/// one pane, a little apart in height.
fn rowed(pieces: Vec<ScreenText>, on_screen: &[ScreenText]) -> Vec<ScreenText> {
    if pieces.is_empty() {
        return pieces;
    }
    let groups: HashSet<(i64, i64)> = pieces.iter().map(|p| (p.root, p.parent)).collect();
    let boxes: HashSet<i64> = pieces.iter().map(|p| p.r#box).collect();
    let mates: Vec<ScreenText> = on_screen
        .iter()
        .filter(|p| groups.contains(&(p.root, p.parent)) && !boxes.contains(&p.r#box))
        .cloned()
        .collect();
    if mates.is_empty() {
        return pieces;
    }
    let mut all: Vec<ScreenText> = pieces.into_iter().chain(mates).collect();
    all.sort_by_key(|p| p.r#box);
    all
}

/// Which of the summary's pages is shown, and the words of the others, which it does not draw.
///
/// The summary keeps every page it has drawn on screen, every flag and place of it as when it
/// was drawn, and draws one: turned back to, a page said only its button, "Check Moves", its words
/// said once already, and switching Pokémon read every page's words, on the memo page too
/// (or_summary_screen_weirdness). Each page has a list, of the moves, their contest kinds or the
/// ribbons, flagged while its page is shown (`LISTS`, spike 304): the other pages' layouts are left
/// unheard (`hidden`), and a turn says the page turned to as a screen opening is read (`turned`).
/// Their layouts stay read as ever: put away, they left and came back, and the stats came back
/// into Check Moves' details with each move's. Check Moves flips the page under the player
/// between the moves and their contest kinds (spike 298): each move is heard in the shown page's
/// words, and no flip is a turn, told by the move both pages' lists hold (`CHOSEN`). A cursor
/// over the moves told it before, and stayed over them once Check Moves had ended on the
/// conditions page: the turns after it went unsaid (spike 304). Nor is a page changed with the
/// Pokémon: an Egg turns the summary to its memo, and the next Pokémon back to its stats, and the
/// Pokémon is said as a switch is. A page is drawn afresh as it is turned to, a poll on: read at
/// once, Pikachu's turned to after a switch said "Type: Steel", the Pokémon's before, then "Type:
/// Electric"; so a turn hears no page for a poll, and the page turned to whole after it.
pub struct Pages {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    measured: bool,
    shape: Shape,
    apps: Rc<RefCell<RunningApp>>,
    layouts: Rc<dyn TrackedLayouts>,
    files: Rc<dyn FileMatching>,
    kept: Kept,
    /// The process, and its pages' lists in `LISTS`'s order once each is of its class.
    lists: Option<(i64, Vec<i64>)>,
    /// The page shown, its place in `LISTS`.
    page: Option<usize>,
    /// The place of the Pokémon shown at the last poll (`PLACE`).
    shown: Option<i64>,
    /// Polls since the page last turned.
    since: i64,
    /// Polls since the page last changed with the Pokémon.
    paged: i64,
}

impl Pages {
    /// `names` names the Pokémon shown's record (`pokemon.read`); `shape` is where the game
    /// keeps it and its pages' lists, None where not measured: its pages are then its
    /// layouts' alone. Python's default is `Some(ORAS)`.
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        apps: Rc<RefCell<RunningApp>>,
        layouts: Rc<dyn TrackedLayouts>,
        files: Rc<dyn FileMatching>,
        shape: Option<Shape>,
    ) -> Self {
        let kept = Kept::new(
            memory.clone(),
            {
                let apps = apps.clone();
                Rc::new(move || apps.borrow().process)
            },
            {
                let apps = apps.clone();
                Rc::new(move || apps.borrow().running.clone())
            },
        );
        Pages {
            memory,
            names,
            measured: shape.is_some(),
            shape: shape.unwrap_or(ORAS),
            apps,
            layouts,
            files,
            kept,
            lists: None,
            page: None,
            shown: None,
            since: 2,
            paged: 2,
        }
    }

    /// Read the page shown.
    pub fn poll(&mut self) -> Result<()> {
        let process = {
            let apps = self.apps.borrow();
            if apps.running.as_deref() == Some(SUMMARY) && self.measured {
                apps.process
            } else {
                None
            }
        };
        self.since += 1;
        self.paged += 1;
        let Some(process) = process else {
            self.lists = None;
            self.page = None;
            self.shown = None;
            self.kept.forget();
            return Ok(());
        };
        // As the summary opens no list is flagged for a poll, and the page before holds.
        let page = self.flagged(process)?.or(self.page);
        let mon = u32(&self.memory.read(process + self.shape.place, 4)?, 0)?;
        if let (Some(page), Some(before)) = (page, self.page)
            && page != before
        {
            if Some(mon) != self.shown {
                self.paged = 0;
            } else if !self.flipped(before, page)? {
                self.since = 0;
            }
        } else if self.since == 1 && Some(mon) != self.shown {
            // Leaving an Egg, the stats page's list was flagged a poll before the place changed
            // (spike 304): a switch, not a turn, and no button said after it.
            self.since = 3;
            self.paged = 0;
        }
        self.page = page;
        self.shown = Some(mon);
        Ok(())
    }

    /// The works of the pages of a Pokémon's while the summary shows an Egg, else none.
    pub fn put_away(&self) -> Result<HashSet<i64>> {
        let mon = self.shown_pokemon()?;
        if !mon.is_some_and(|mon| mon.egg) {
            return Ok(HashSet::new());
        }
        Ok(self
            .layouts
            .works()
            .into_iter()
            .filter(|&(_work, root)| {
                self.files
                    .matching(&self.layouts.pane_names(root))
                    .is_some_and(|file| POKEMON_PAGES.contains(&file.name.as_str()))
            })
            .map(|(work, _root)| work)
            .collect())
    }

    /// The name of the Pokémon the summary shows, its nickname or its species', or None: a
    /// rewrite's first row (`ScreenAnnouncer.say`'s `subject`).
    pub fn subject(&self) -> Result<Option<String>> {
        let mon = self.shown_pokemon()?;
        Ok(mon.map(|mon| mon.nickname))
    }

    /// The poll's cursors and words as the summary's pages are heard, and the roots of the
    /// page turned to, said as it opened, the first of its words cutting off the page before
    /// (`ScreenAnnouncer.say`'s `lead`). `heard` is every word on screen to be heard.
    pub fn said(
        &self,
        cursors: CursorPoll,
        texts: Vec<ScreenText>,
        heard: &[ScreenText],
    ) -> (CursorPoll, Vec<ScreenText>, HashSet<i64>) {
        let unseen = self.hidden();
        let cursors = if unseen.is_empty() {
            cursors
        } else {
            // Check Moves' cursors are over both pages' moves: each move in the shown one's
            // words, with the rest of its row there. On the conditions page the cursor rests on
            // the battle moves' row, and the contest list's word under it was the kind alone.
            let items = cursors
                .items
                .iter()
                .map(|item| {
                    let shown: Vec<ScreenText> = item
                        .pieces
                        .iter()
                        .filter(|p| !unseen.contains(&p.root))
                        .cloned()
                        .collect();
                    CursorItem {
                        pieces: rowed(shown, heard),
                        ..item.clone()
                    }
                })
                .collect();
            CursorPoll { items, ..cursors }
        };
        // A page turned to is said as it opened, its words as they stand; the button under it a
        // poll later, queued.
        let turned = self.turned();
        let mut out = texts;
        let mut back = turned.clone();
        back.extend(self.button());
        if !back.is_empty() {
            let written: HashSet<i64> = out.iter().map(|text| text.r#box).collect();
            out = out
                .into_iter()
                .map(|text| {
                    if back.contains(&text.root) {
                        ScreenText {
                            changed: false,
                            returned: true,
                            ..text
                        }
                    } else {
                        text
                    }
                })
                .collect();
            out.extend(
                heard
                    .iter()
                    .filter(|piece| back.contains(&piece.root) && !written.contains(&piece.r#box))
                    .map(|piece| ScreenText {
                        changed: false,
                        returned: true,
                        ..piece.clone()
                    }),
            );
        }
        // A page changed with the Pokémon, an Egg's or the one after it, is said as a switch is:
        // the name, then the rest, its layouts coming back to the screen part of it.
        let switched = self.switched();
        if !switched.is_empty() {
            out = out
                .into_iter()
                .map(|text| {
                    if switched.contains(&text.root) {
                        ScreenText {
                            changed: true,
                            ..text
                        }
                    } else {
                        text
                    }
                })
                .collect();
        }
        (cursors, out, turned)
    }

    fn shown_pokemon(&self) -> Result<Option<Pokemon>> {
        let param = if self.measured {
            shown(&*self.memory, &self.apps.borrow(), &self.shape)?
        } else {
            None
        };
        match param {
            Some(param) if param != 0 => {
                pokemon::read(&*self.memory, &self.names, param, pokemon::SPECIES_MAX)
            }
            _ => Ok(None),
        }
    }

    /// The page whose list is flagged shown, or None while not one alone is.
    fn flagged(&mut self, process: i64) -> Result<Option<usize>> {
        if self.lists.as_ref().is_none_or(|lists| lists.0 != process) {
            let mut lists: Vec<i64> = Vec::new();
            for &(at, name, _, _) in self.shape.lists {
                let Some(found) = self.kept.find(&[at], Some(name), &[])? else {
                    return Ok(None);
                };
                lists.push(found);
            }
            self.lists = Some((process, lists));
        }
        let lists = &self.lists.as_ref().expect("found above").1;
        let mut flags: Vec<bool> = Vec::new();
        for &each in lists {
            let flag = *self
                .memory
                .read(each + SHOWING, 1)?
                .first()
                .ok_or_else(|| Error::index("index out of range"))?;
            flags.push(flag != 0);
        }
        Ok(if flags.iter().filter(|&&flag| flag).count() == 1 {
            flags.iter().position(|&flag| flag)
        } else {
            None
        })
    }

    /// Whether Check Moves turned the page: both pages' lists of moves hold the move chosen.
    fn flipped(&self, before: usize, after: usize) -> Result<bool> {
        let shape_lists = self.shape.lists;
        let Some((_, lists)) = &self.lists else {
            return Ok(false);
        };
        if !(shape_lists[before].3 && shape_lists[after].3) {
            return Ok(false);
        }
        let mut chosen: HashSet<i64> = HashSet::new();
        for page in [before, after] {
            chosen.insert(u16(&self.memory.read(lists[page] + CHOSEN, 2)?, 0)?);
        }
        Ok(chosen.len() == 1 && !chosen.contains(&0))
    }

    /// The cursors, those over the moves left out while Check Moves shows no move.
    ///
    /// Check Moves ended, the game leaves a moves cursor set until a switch (spike 304): ended
    /// on the conditions page, it owned that page's moves, which turning back to it left
    /// unsaid; and switching Pokémon said a move first, "Scratch, Toughness", then the
    /// Pokémon as its details, its whole stats page on the stats page (or_summary_spike). The
    /// shown page's list holds the move chosen while Check Moves is on, from the poll its
    /// cursor comes on screen (`CHOSEN`, spike 303), and is read only while a cursor is over the
    /// moves.
    pub fn cursors(&self, cursors: CursorPoll, on_screen: &[ScreenText]) -> Result<CursorPoll> {
        let moves: HashSet<i64> = if self.page.is_some() {
            self.roots(&MOVES)
        } else {
            HashSet::new()
        };
        if moves.is_empty()
            || !cursors
                .under
                .iter()
                .any(|under| under.iter().any(|p| moves.contains(&p.root)))
        {
            return Ok(cursors);
        }
        if self.chosen()? != 0 {
            return Ok(cursors);
        }
        let boxes: HashSet<i64> = on_screen
            .iter()
            .filter(|piece| moves.contains(&piece.root))
            .map(|piece| piece.r#box)
            .collect();
        Ok(CursorPoll {
            items: cursors
                .items
                .iter()
                .filter(|item| item.pieces.iter().all(|p| !moves.contains(&p.root)))
                .cloned()
                .collect(),
            under: cursors
                .under
                .iter()
                .filter(|under| under.iter().all(|p| !moves.contains(&p.root)))
                .cloned()
                .collect(),
            lists: cursors
                .lists
                .iter()
                .filter(|(root, _)| !moves.contains(root))
                .map(|(&root, listed)| (root, listed.clone()))
                .collect(),
            rows: cursors.rows.difference(&boxes).copied().collect(),
            ..cursors
        })
    }

    /// The move the shown page's list has chosen in Check Moves, or 0.
    fn chosen(&self) -> Result<i64> {
        let (Some(page), Some((_, lists))) = (self.page, &self.lists) else {
            return Ok(0);
        };
        if !self.shape.lists[page].3 {
            return Ok(0);
        }
        u16(&self.memory.read(lists[page] + CHOSEN, 2)?, 0)
    }

    /// The roots of the pages not shown, whose words are not heard; none before a page is
    /// known, every page's the poll it turns, as it is drawn afresh, and the button's until it
    /// is said (`button`).
    pub fn hidden(&self) -> HashSet<i64> {
        let Some(page) = self.page else {
            return HashSet::new();
        };
        let others: Vec<&str> = PAGES
            .iter()
            .copied()
            .filter(|name| !self.shape.lists[page].2.contains(name))
            .collect();
        if self.since == 0 {
            return self.roots(&[&PAGES[..], &[BUTTON]].concat());
        }
        if self.since == 1 {
            return self.roots(&[&others[..], &[BUTTON]].concat());
        }
        self.roots(&others)
    }

    /// The roots of the page turned to a poll ago, drawn afresh since, or none: its words are
    /// said as a screen opening's are, the first cutting off the page before
    /// (`ScreenAnnouncer.say`'s `lead`).
    pub fn turned(&self) -> HashSet<i64> {
        match self.page {
            Some(page) if self.since == 1 => self.roots(self.shape.lists[page].2),
            _ => HashSet::new(),
        }
    }

    /// The roots of the button under the page turned to two polls ago, or none: said after
    /// the page, queued behind it, if it is still there. A Pokémon with no ribbons has none on
    /// its memo, and the game hides the button left from the page before a poll after the turn:
    /// said with the page, the memo was followed by "Check Moves" (spike 304).
    pub fn button(&self) -> HashSet<i64> {
        match self.page {
            Some(_) if self.since == 2 => self.roots(&[BUTTON]),
            _ => HashSet::new(),
        }
    }

    /// The roots of the page shown and the Pokémon's name where the page changed with the
    /// Pokémon at the last poll or the one before, or none: an Egg's memo, or the stats of the
    /// Pokémon after an Egg, whose layouts come back to the screen a poll later, said as a switch
    /// is, the name first. Leaving an Egg said the stats page, then "Mario, Lv.66, None", the name
    /// twice.
    pub fn switched(&self) -> HashSet<i64> {
        match self.page {
            Some(page) if self.paged <= 1 => {
                self.roots(&[self.shape.lists[page].2, &[INFO, BUTTON]].concat())
            }
            _ => HashSet::new(),
        }
    }

    /// The roots of the layouts of these files.
    fn roots(&self, names: &[&str]) -> HashSet<i64> {
        self.layouts
            .works()
            .values()
            .copied()
            .filter(|&root| {
                self.files
                    .matching(&self.layouts.pane_names(root))
                    .is_some_and(|found| names.contains(&found.name.as_str()))
            })
            .collect()
    }

    /// The TextBoxes of the conditions star's labels, "Coolness" to "Toughness": a label
    /// with no value, said as a list on the conditions page, "Coolness", then "Beauty,
    /// Cleverness, Cuteness, Toughness". Never said; the conditions Extra says them with their
    /// values (`oras.screens.conditions`). The Move Reminder draws them as pictures hanging from
    /// its layout's root, and each move's category icon beside the star, which hangs from its
    /// row, is no label.
    pub fn star_labels(&self, on_screen: &[ScreenText], words: &HashSet<String>) -> HashSet<i64> {
        let mut stars: IndexMap<i64, bool> = IndexMap::new();
        let mut boxes: HashSet<i64> = HashSet::new();
        for piece in on_screen {
            if !words.contains(&piece.text) {
                continue;
            }
            if !stars.contains_key(&piece.root) {
                let found = self.files.matching(&self.layouts.pane_names(piece.root));
                stars.insert(
                    piece.root,
                    found.is_some_and(|found| {
                        found
                            .textures
                            .values()
                            .any(|drawn| drawn.iter().any(|texture| texture == STAR))
                    }),
                );
            }
            if stars[&piece.root] && !(piece.picture && piece.parent != piece.root) {
                boxes.insert(piece.r#box);
            }
        }
        boxes
    }
}
