//! The name entry's keyboard draws a third kind (spike 68): each key is a picture holding its letter
//! and an `nw::lyt::Bounding` touch area, and the cursor is a picture the game moves into the chosen
//! key's touch area. The item is the key's words, or, for a key drawn without any — the one that
//! confirms the name — the words of its picture's texture, as a battle button's are.
//!
//! Bound by:
//! - ADR-0014: the key under the cursor, a capital as "Cap O", and the letters left after one is
//!   deleted.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::games::common::cursors::shared::{CursorItem, Panes, Said, Where, class_of};
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::games::common::screen::{ScreenText, distinct, in_rows};
use crate::py::{PySet, Result};

/// The name entry's cursor: a picture moved into the chosen key's touch area.
pub struct KeyCursors {
    panes: Rc<Panes>,
    layouts: Rc<Layouts>,
    said: Rc<Said>,
    /// Picture in a key's touch area -> the key it was in at the last poll it was seen.
    key_plates: IndexMap<i64, i64>,
    /// Pictures in a key's touch area that have moved from one key to another.
    moved_keys: HashSet<i64>,
}

impl KeyCursors {
    pub fn new(panes: Rc<Panes>, said: Rc<Said>) -> Self {
        KeyCursors {
            layouts: panes.layouts.clone(),
            panes,
            said,
            key_plates: IndexMap::new(),
            moved_keys: HashSet::new(),
        }
    }

    /// Report the key each key cursor is in, and return every picture in a key's touch area,
    /// with its layout's root and the key.
    pub fn poll(
        &mut self,
        on_screen: &[ScreenText],
        items: &mut Vec<CursorItem>,
    ) -> Result<IndexMap<i64, (i64, i64)>> {
        let keys = self.keys(on_screen)?;
        let gone: Vec<i64> = self
            .key_plates
            .keys()
            .copied()
            .filter(|picture| !keys.contains_key(picture))
            .collect();
        for picture in gone {
            self.key_plates.shift_remove(&picture);
            self.moved_keys.remove(&picture);
            self.said.forget(picture);
        }
        let mut alone: HashMap<i64, i64> = HashMap::new();
        for &(root, _plate) in keys.values() {
            *alone.entry(root).or_insert(0) += 1;
        }
        for (&picture, &(root, plate)) in &keys {
            if alone[&root] == 1 || self.moved_keys.contains(&picture) {
                let item = self.key_item(root, plate, picture, on_screen)?;
                self.said
                    .report_in(picture, item, items, Where::Pane(plate), false, 0, true);
            }
        }
        Ok(keys)
    }

    /// Picture -> (root, key) for every picture shown in a key's touch area, where a key is a
    /// picture or a window holding a touch area, as the name entry's keyboard draws its keys.
    ///
    /// One of these is taken for a cursor while it is alone in its layout, or once it has moved
    /// from key to key: an icon standing in a button's touch area stays where it is. Where the
    /// layout's file is known, only a picture drawn as a cursor is one, as the name entry's is:
    /// the PC Box keeps each party Pokémon's icon in its slot's touch area, which made it a
    /// keyboard where no named cursor is looked for, and a lone Pokémon's icon a key cursor
    /// said as "poke icon" (or/movie6_pc, 2026-09-27).
    fn keys(&mut self, on_screen: &[ScreenText]) -> Result<IndexMap<i64, (i64, i64)>> {
        let mut found: IndexMap<i64, (i64, i64)> = IndexMap::new();
        let mut roots = PySet::new();
        for piece in on_screen {
            roots.add(piece.root);
        }
        for root in roots.iter().collect::<Vec<_>>() {
            let tree = self.layouts.tree(root);
            let file = self.panes.file(root);
            for (&pane, name) in &tree {
                if name != lyt::PICTURE_CLASS {
                    continue;
                }
                if file.is_some() && !self.panes.drawn_as_cursor(root, pane, file.as_deref())? {
                    continue;
                }
                let touch = self.panes.parent(pane)?;
                if class_of(&tree, touch) != Some(lyt::BOUNDING_CLASS) {
                    continue;
                }
                let plate = self.panes.parent(touch)?;
                if !matches!(
                    class_of(&tree, plate),
                    Some(lyt::PICTURE_CLASS | lyt::WINDOW_CLASS)
                ) {
                    continue;
                }
                let placement = self.layouts.placement(pane)?;
                if !placement.is_some_and(|placement| placement.shown) {
                    continue;
                }
                if self.key_plates.get(&pane).copied().unwrap_or(plate) != plate {
                    self.moved_keys.insert(pane);
                }
                self.key_plates.insert(pane, plate);
                found.insert(pane, (root, plate));
            }
        }
        Ok(found)
    }

    /// The words of the key a cursor is in: its own, or those of its picture's texture.
    fn key_item(
        &self,
        root: i64,
        plate: i64,
        cursor: i64,
        on_screen: &[ScreenText],
    ) -> Result<Vec<ScreenText>> {
        let inside = distinct(in_rows(
            on_screen
                .iter()
                .filter(|piece| piece.root == root && piece.parent == plate)
                .cloned(),
        ));
        if !inside.is_empty() {
            return Ok(inside);
        }
        let Some(said) = self.panes.pictured(root, plate, on_screen, Some(cursor))? else {
            return Ok(Vec::new());
        };
        Ok(vec![ScreenText {
            root,
            parent: plate,
            ..ScreenText::new(plate, said, false)
        }])
    }
}
