//! The Pokédex lists: Omega Ruby's entry said the tick it is chosen, X's by its number and name.
//!
//! Omega Ruby's list keeps its rows in `app::sango_zukan::ZukanLaunch`, the process's +0xC8: nine
//! records of 0x44 bytes from +0x50, each an index and then pointers into a row's panes (spike B
//! on or_menus). The fifth's index is the entry chosen; the fourth's panes show its number and
//! name (spike N on or_pokedex: their words change the tick the index does, or a tick before). The
//! plate over the list is a picture fixed in place while the rows slide a row's height and back
//! under it, and its item was said once it had read the same for two polls, 0.3 to 0.4 s after
//! the index changed. The index is read every tick, and a move is said the poll it changes, in
//! the words of the fourth record's panes (`OrasList`): no slide can put another row's words
//! there. A page jumps seven, passing the entries between over 0.1 s, by one or two a tick: every
//! move is said once the index has held four ticks, as the Mart's jumps are once they hold
//! (`mart.Jumps`).
//!
//! X's list is described below.
//!
//! X's list is a row of numbered icons, and its cursor's words are the number alone, "004"; the
//! entry's name is the top screen's, on the line of its "No." and the same number, with the seen
//! and obtained counts above them and the rest of the entry below (x_pokedex). Said as they were, a
//! move was "004", then "SEEN 2 OBTAINED 1 No. Fennekin Fox Pokémon …": the counts at every move,
//! and "No." with its number left out as the cursor's words. A move is said as the panel names
//! it, "004 Fennekin", once the panel's number agrees with the cursor's, a few polls at most, what
//! the screen changes meanwhile held for its details (`CursorPoll.chased`); the details are the
//! rest of the panel. The line is left to the cursor (`CursorPoll.rows`), not taken off the
//! screen. The counts are still said with each entry: the details' table is told by how far right
//! the words before it reach (`screen_announce._columns`), and left out, the counts' "1" no longer
//! held HEIGHT's and WEIGHT's values in their labels' column, "Fire Fox Pokémon TYPE HEIGHT WEIGHT
//! 1'04" 20.7 lbs.".
//!
//! The entry under the cursor is `app::zukan::DataManager` +0x130 (spike B on x_pokedex), the
//! process's +0x3C; X said each move the poll it changed, so it is not read.

use std::collections::HashSet;
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::IndexMap;
use regex::Regex;

use crate::core::binary::{s32, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll, Rect};
use crate::games::common::screen::{ScreenText, join};
use crate::games::gen6::anchors::follow;
use crate::games::gen6::apps::LOADED_CODE;
use crate::py::{self, Result};

/// X's and Y's Pokédex.
pub const APP: &str = "N3app5zukan8MainProcE";
/// Omega Ruby's and Alpha Sapphire's Pokédex.
pub const ORAS_APP: &str = "N3app11sango_zukan9ZukanProcE";
/// The Pokédex's process -> `app::sango_zukan::ZukanLaunch`, its list.
pub const LAUNCH: i64 = 0xC8;
pub const LAUNCH_CLASS: &str = "N3app11sango_zukan11ZukanLaunchE";
pub const ROWS: i64 = 0x50;
pub const RECORD: i64 = 0x44;
/// The record whose index, a s32 at its start, is the entry chosen.
pub const CHOSEN: i64 = ROWS + 4 * RECORD;
/// The record whose panes show the entry chosen.
pub const SHOWN: i64 = ROWS + 3 * RECORD;
/// Ticks the index holds before its entry is said: a jump of seven steps it on by one or two
/// every tick or two (spike N), and a step holds from the first.
pub const STILL_TICKS: i64 = 4;
/// A list row's words: the entry's number in the Pokédex shown, three digits. Anchored at both
/// ends, as Python's `fullmatch` is.
pub static NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{3}$").unwrap());
/// The most polls a move waits for the panel to name its entry, said by its number alone
/// after.
pub const WAIT_POLLS: i64 = 3;
/// Units within which two pieces beside each other are on one line.
pub const ONE_LINE: f64 = 1.0;

/// Names X's Pokédex list's entries by the top screen's panel.
#[derive(Default)]
pub struct XyList {
    /// A move waiting for the panel to name its entry, and the polls it has waited.
    held: Option<(CursorItem, i64)>,
    /// The number last said with its name, the list's root and the panel's.
    named: Option<(String, i64, i64)>,
}

impl XyList {
    pub fn new() -> Self {
        XyList::default()
    }

    /// The cursors' poll with a move's entry named by the panel, or held for it, and the
    /// panel's line naming the entry the cursor's own, unsaid.
    pub fn poll(
        &mut self,
        running: Option<&str>,
        cursors: CursorPoll,
        on_screen: &[ScreenText],
    ) -> CursorPoll {
        if running != Some(APP) {
            (self.held, self.named) = (None, None);
            return cursors;
        }
        if let Some((_, _, panel)) = &self.named
            && !on_screen.iter().any(|p| p.root == *panel)
        {
            // The panel gone, for an entry's pages: back on the list it is said whole.
            self.named = None;
        }
        let moves: Vec<CursorItem> = cursors
            .items
            .iter()
            .filter(|item| !item.opened && !number(item).is_empty())
            .cloned()
            .collect();
        let mut items: Vec<CursorItem> = cursors
            .items
            .iter()
            .filter(|item| !moves.contains(item))
            .cloned()
            .collect();
        if let Some(last) = moves.last() {
            self.held = Some((last.clone(), 0));
        }
        let mut chased = cursors.chased;
        if let Some((item, waited)) = self.held.clone() {
            let (number, root) = (number(&item), item.pieces[0].root);
            if let Some((_, said, name)) = line(&number, root, on_screen) {
                (self.held, self.named) = (None, Some((number, root, said.root)));
                let mut pieces = item.pieces.clone();
                pieces.push(name);
                items.push(CursorItem { pieces, ..item });
            } else if waited >= WAIT_POLLS {
                self.held = None;
                items.push(item);
            } else {
                self.held = Some((item, waited + 1));
                chased = true;
            }
        }
        let mut quiet: HashSet<i64> = HashSet::new();
        if let Some((number, root, panel)) = &self.named {
            let shown: Vec<ScreenText> = on_screen
                .iter()
                .filter(|p| p.root == *panel)
                .cloned()
                .collect();
            if let Some((before, said, name)) = line(number, *root, &shown) {
                quiet = before
                    .iter()
                    .chain([&said, &name])
                    .map(|p| p.r#box)
                    .collect();
            }
        }
        let rows = cursors.rows.union(&quiet).copied().collect();
        CursorPoll {
            items,
            chased,
            rows,
            ..cursors
        }
    }
}

/// The entry number a list's row says, or "" for another cursor's item.
fn number(item: &CursorItem) -> String {
    let text = join(&item.pieces, false);
    if NUMBER.is_match(&text) {
        text
    } else {
        String::new()
    }
}

/// The panel's line naming this entry, from a layout other than the list's: the words before
/// its number, the number, and the name after it; None while the panel names another.
fn line(
    number: &str,
    root: i64,
    on_screen: &[ScreenText],
) -> Option<(Vec<ScreenText>, ScreenText, ScreenText)> {
    for said in on_screen {
        if said.root == root || said.text != number {
            continue;
        }
        let beside: Vec<&ScreenText> = on_screen
            .iter()
            .filter(|p| {
                p.root == said.root && p.r#box != said.r#box && (p.row - said.row).abs() < ONE_LINE
            })
            .collect();
        let after = py::sorted_by_lt(
            beside
                .iter()
                .filter(|p| p.left > said.left)
                .copied()
                .collect(),
            |a: &&ScreenText, b: &&ScreenText| a.left < b.left,
        );
        if let Some(first) = after.first() {
            return Some((
                beside
                    .iter()
                    .filter(|p| p.left < said.left)
                    .map(|p| (*p).clone())
                    .collect(),
                said.clone(),
                (*first).clone(),
            ));
        }
    }
    None
}

/// The cursors' poll with a search field named by its label, one of `labels`.
///
/// Search and sort in both games is a table, each field's name at its row's left and its value
/// beside it, and the cursor frames the value alone: a field not set was its dashes alone, and
/// which of NAME, COLOR, APPEARANCE or ENCOUNTERS it was went unsaid (or_pokedex, x_pokedex);
/// one set, its letter alone, "A". The labels are the game's text's (`GameText.search_labels`):
/// the type grid's "Fighting" has "Normal" beside it, no label, and a picker's choices are on
/// rows under its label's.
pub fn labelled(
    running: Option<&str>,
    cursors: CursorPoll,
    on_screen: &[ScreenText],
    labels: &HashSet<String>,
) -> CursorPoll {
    if running != Some(APP) && running != Some(ORAS_APP) {
        return cursors;
    }
    let mut items: Vec<CursorItem> = Vec::new();
    for item in &cursors.items {
        let first = item.pieces.first();
        let Some(first) = first.filter(|first| !labels.contains(&first.text)) else {
            items.push(item.clone());
            continue;
        };
        let boxes: HashSet<i64> = item.pieces.iter().map(|p| p.r#box).collect();
        // `max(before, key=left)`: the first of equal bests, as Python's.
        let mut label: Option<&ScreenText> = None;
        for p in on_screen {
            if p.root == first.root
                && !boxes.contains(&p.r#box)
                && (p.row - first.row).abs() < ONE_LINE
                && p.left < first.left
                && labels.contains(&p.text)
                && label.is_none_or(|best| p.left > best.left)
            {
                label = Some(p);
            }
        }
        items.push(match label {
            None => item.clone(),
            Some(label) => CursorItem {
                pieces: [vec![label.clone()], item.pieces.clone()].concat(),
                ..item.clone()
            },
        });
    }
    CursorPoll { items, ..cursors }
}

/// The label of the search field a menu cursor frames with nothing in it (`menu.Blank`):
/// TYPE's two and SHAPE's are blank until set, and the cursor on them said nothing
/// (or_pokedex). Its label is left of it on its row.
pub fn blank(
    running: Option<&str>,
    rect: &Rect,
    on_screen: &[ScreenText],
    labels: &HashSet<String>,
) -> Vec<ScreenText> {
    if running != Some(APP) && running != Some(ORAS_APP) {
        return Vec::new();
    }
    // `max(left, key=left)`: the first of equal bests, as Python's.
    let mut best: Option<&ScreenText> = None;
    for p in on_screen {
        if labels.contains(&p.text)
            && p.top == rect.top
            && rect.low <= p.row
            && p.row <= rect.high
            && p.right <= rect.left
            && best.is_none_or(|best| p.left > best.left)
        {
            best = Some(p);
        }
    }
    best.into_iter().cloned().collect()
}

/// The boxes of search and sort's table that come on screen with it or back to it, a menu
/// the cursor says a field of at a time: opened before its cursor came, the table was read
/// whole, "SORT By Number NAME", each field's dashes and on, then "Reset Confirm" (or_pokedex);
/// back from a picker, it was the cursor's next field's details. The table is the panel
/// holding two labels or more (`labelled`): an entry's page has TYPE alone, and a picker's
/// panel its own label, left to say which picker opened.
pub fn tabled(
    running: Option<&str>,
    texts: &[ScreenText],
    on_screen: &[ScreenText],
    cursors: &CursorPoll,
    labels: &HashSet<String>,
) -> HashSet<i64> {
    if running != Some(APP) && running != Some(ORAS_APP) {
        return HashSet::new();
    }
    let mut held: IndexMap<(i64, i64), HashSet<&str>> = IndexMap::new();
    for p in on_screen {
        if labels.contains(&p.text) {
            held.entry((p.root, p.panel))
                .or_default()
                .insert(p.text.as_str());
        }
    }
    let tables: HashSet<(i64, i64)> = held
        .iter()
        .filter(|(_, words)| words.len() > 1)
        .map(|(&panel, _)| panel)
        .collect();
    let under: HashSet<i64> = cursors.under.iter().flatten().map(|p| p.r#box).collect();
    texts
        .iter()
        .filter(|p| {
            tables.contains(&(p.root, p.panel))
                && !under.contains(&p.r#box)
                && (p.with_layout || p.returned)
        })
        .map(|p| p.r#box)
        .collect()
}

/// Omega Ruby's Pokédex list's entry, said the tick it is chosen in its row's own words.
pub struct OrasList {
    memory: Rc<dyn Memory>,
    running: Rc<dyn Fn() -> Option<String>>,
    process: Rc<dyn Fn() -> Option<i64>>,
    /// The process and its `ZukanLaunch`, once found and checked by its class.
    launch: Option<(i64, Option<i64>)>,
    /// The panes the record showing the entry chosen points at. A `frozenset` only tested.
    boxes: HashSet<i64>,
    index: Option<i64>,
    /// Ticks the index has held, after a step or a jump.
    still: i64,
    /// The entry last said, and its words, while the list is up.
    said: Option<(i64, String)>,
}

impl OrasList {
    pub fn new(
        memory: Rc<dyn Memory>,
        running: Rc<dyn Fn() -> Option<String>>,
        process: Rc<dyn Fn() -> Option<i64>>,
    ) -> Self {
        OrasList {
            memory,
            running,
            process,
            launch: None,
            boxes: HashSet::new(),
            index: None,
            still: 0,
            said: None,
        }
    }

    /// Has the entry chosen changed since the last tick? Asked every tick, one read.
    pub fn moved(&mut self) -> Result<bool> {
        let Some(index) = self.read()? else {
            self.index = None;
            return Ok(false);
        };
        if Some(index) == self.index {
            self.still += 1;
            return Ok(false);
        }
        (self.index, self.still) = (Some(index), 0);
        Ok(true)
    }

    /// The cursors' poll with the plate's items replaced by the entry chosen, said the poll
    /// it changes in its row's words, a jump once it holds still.
    pub fn cursors(
        &mut self,
        cursors: CursorPoll,
        on_screen: &[ScreenText],
    ) -> Result<CursorPoll> {
        let launch = match self.launch {
            Some((_, Some(launch))) if (self.running)().as_deref() == Some(ORAS_APP) => launch,
            _ => {
                self.said = None;
                return Ok(cursors);
            }
        };
        if !on_screen.iter().any(|p| self.boxes.contains(&p.r#box)) {
            // Its panes are set once the list is built, after the Pokédex starts.
            let record = self.memory.read(launch + SHOWN, RECORD)?;
            let mut boxes = HashSet::new();
            for at in (4..RECORD).step_by(4) {
                boxes.insert(u32(&record, at)?);
            }
            self.boxes = boxes;
        }
        let pieces: Vec<ScreenText> = py::sorted_by_lt(
            on_screen
                .iter()
                .filter(|p| self.boxes.contains(&p.r#box))
                .cloned()
                .collect(),
            |a: &ScreenText, b: &ScreenText| a.left < b.left,
        );
        let root = pieces.first().map(|p| p.root);
        let Some(root) = root.filter(|root| {
            cursors
                .under
                .iter()
                .any(|under| under.first().is_some_and(|first| first.root == *root))
        }) else {
            // No plate over the list: its entry is said afresh when it comes back.
            self.said = None;
            return Ok(cursors);
        };
        let plate: Vec<CursorItem> = cursors
            .items
            .iter()
            .filter(|item| item.pieces.first().is_some_and(|first| first.root == root))
            .cloned()
            .collect();
        let items: Vec<CursorItem> = cursors
            .items
            .iter()
            .filter(|item| !plate.contains(item))
            .cloned()
            .collect();
        let (index, words) = (self.index, join(&pieces, false));
        let Some(index) = index.filter(|index| {
            self.said
                .as_ref()
                .is_none_or(|(said, _)| said != index)
        }) else {
            return Ok(CursorPoll { items, ..cursors });
        };
        if self.still < STILL_TICKS {
            return Ok(CursorPoll {
                items,
                chased: true,
                ..cursors
            });
        }
        if self.said.as_ref().is_some_and(|(_, said)| *said == words) {
            // The index has moved and its row's words not yet.
            return Ok(CursorPoll {
                items,
                chased: true,
                ..cursors
            });
        }
        let cursor = plate.first().map_or(launch, |item| item.cursor);
        let opened = self.said.is_none();
        self.said = Some((index, words));
        let mut items = items;
        items.push(CursorItem::new(cursor, pieces, opened));
        Ok(CursorPoll { items, ..cursors })
    }

    /// The entry chosen, while the Pokédex runs and its list is built.
    fn read(&mut self) -> Result<Option<i64>> {
        let process = (self.process)();
        let Some(process) = process.filter(|_| (self.running)().as_deref() == Some(ORAS_APP))
        else {
            self.launch = None;
            return Ok(None);
        };
        if self
            .launch
            .is_none_or(|(found, launch)| found != process || launch.is_none())
        {
            let mut launch = follow(&*self.memory, process, &[LAUNCH])?;
            if let Some(found) = launch
                && ClassNames::new(self.memory.clone(), LOADED_CODE)
                    .of_object(found)?
                    .as_deref()
                    != Some(LAUNCH_CLASS)
            {
                launch = None;
            }
            self.launch = Some((process, launch));
        }
        let Some((_, Some(launch))) = self.launch else {
            return Ok(None);
        };
        s32(&self.memory.read(launch + CHOSEN, 4)?, 0)
            .map(Some)
    }
}
