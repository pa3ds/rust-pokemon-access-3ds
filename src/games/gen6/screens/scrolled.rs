//! A list scrolling under its cursor, the cursor standing still.
//!
//! The cursor reader tells a move by the words under a cursor or where the cursor is (`Said`), and
//! a list that scrolls under a cursor standing still, to a row reading as the row before, makes no
//! move to it: the AreaNav's Route 104 has three Lum Berry rows in a row (or_areanav_berries), and
//! the Pokédex's search and sort scrolls APPEARANCE's dashes away for ENCOUNTERS' (or_pokedex).
//! The rows are other boxes: each of search and sort's fields keeps its own as it scrolls.
//!
//! Bound by:
//! - ADR-0043: a row of the same words in other boxes, still a poll, is a move.

use std::collections::HashSet;

use indexmap::IndexMap;

use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::screen::ScreenText;

/// A list's row: its boxes and their words. A `frozenset`, only compared.
pub type Row = HashSet<(i64, String)>;

pub fn row(pieces: &[ScreenText]) -> Row {
    pieces.iter().map(|p| (p.r#box, p.text.clone())).collect()
}

/// The moves along rows reading alike of the lists a screen names: the words under the
/// cursor those last said, their boxes others, said once they have held still a poll. Only
/// words the same: words the cursor reader held back, as an item still being laid out, are
/// left to it, and search and sort's picker opening was read as APPEARANCE's dashes with "Mega
/// Evolution Shiny" (or_pokedex).
#[derive(Default)]
pub struct Scrolled {
    /// A list's layout -> the row under its cursor at the last screen poll.
    seen: IndexMap<i64, Row>,
    /// A list's layout -> the row last said.
    said: IndexMap<i64, Row>,
    /// A list's layout -> its cursor.
    cursors: IndexMap<i64, i64>,
}

impl Scrolled {
    pub fn new() -> Self {
        Scrolled::default()
    }

    /// The cursors' poll with the moves of the lists in the layouts `lists` added.
    pub fn poll(&mut self, cursors: CursorPoll, lists: &HashSet<i64>) -> CursorPoll {
        if lists.is_empty() {
            self.seen.clear();
            self.said.clear();
            self.cursors.clear();
            return cursors;
        }
        for item in &cursors.items {
            if let Some(first) = item.pieces.first()
                && lists.contains(&first.root)
            {
                self.cursors.insert(first.root, item.cursor);
                self.said.insert(first.root, row(&item.pieces));
            }
        }
        let mut items = cursors.items.clone();
        for pieces in &cursors.under {
            let Some(root) = pieces.first().map(|first| first.root) else {
                continue;
            };
            if !lists.contains(&root) || !self.cursors.contains_key(&root) {
                continue;
            }
            let now = row(pieces);
            let seen = self.seen.insert(root, now.clone());
            let texts = |row: &Row| {
                let mut texts: Vec<String> = row.iter().map(|(_box, text)| text.clone()).collect();
                texts.sort();
                texts
            };
            if let Some(said) = self.said.get(&root)
                && now != *said
                && Some(&now) == seen.as_ref()
                && texts(&now) == texts(said)
            {
                self.said.insert(root, now);
                items.push(CursorItem::new(self.cursors[&root], pieces.clone(), false));
            }
        }
        CursorPoll { items, ..cursors }
    }
}
