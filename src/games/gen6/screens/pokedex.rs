//! X's Pokédex list: each entry said by its number and the name the top screen gives it.
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
use std::sync::LazyLock;

use regex::Regex;

use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::screen::{ScreenText, join};
use crate::py;

/// X's and Y's Pokédex.
pub const APP: &str = "N3app5zukan8MainProcE";
/// Omega Ruby's and Alpha Sapphire's Pokédex.
pub const ORAS_APP: &str = "N3app11sango_zukan9ZukanProcE";
/// A cursor's words with no letter or digit: a search field not set, full-width dashes or "---".
/// Anchored at both ends, as Python's `fullmatch` is.
pub static WORDLESS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[^\w]+$").unwrap());
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

/// The cursors' poll with a search field not set named by its label.
///
/// Search and sort in both games is a table, each field's name at its row's left and its value
/// beside it, and the cursor frames the value alone: a field not set was its dashes alone, and
/// which of NAME, COLOR, APPEARANCE or ENCOUNTERS it was went unsaid (or_pokedex, x_pokedex).
/// Only words with no letter or digit: a value set says what it is, "Red", and the type grid's
/// "Fighting" has "Normal" beside it, no label.
pub fn labelled(running: Option<&str>, cursors: CursorPoll, on_screen: &[ScreenText]) -> CursorPoll {
    if running != Some(APP) && running != Some(ORAS_APP) {
        return cursors;
    }
    let mut items: Vec<CursorItem> = Vec::new();
    for item in &cursors.items {
        let first = item.pieces.first();
        let Some(first) = first.filter(|_| WORDLESS.is_match(&join(&item.pieces, false))) else {
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
                && !WORDLESS.is_match(&p.text)
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
