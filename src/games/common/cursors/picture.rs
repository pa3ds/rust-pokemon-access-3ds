//! The battle draws its cursor otherwise (spike 50): a small `nw::lyt::Picture` alone in a layout of
//! its own, placed exactly on an empty pane in the chosen button's group. The item is that group: the
//! screen text under it, as a move's "Scratch PP 35/35", or, where the button is all pictures, as the
//! command menu's FIGHT, the words of the name of its first picture's texture, "fight", from the
//! layout file the button's layout was built from.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::words::own;
use crate::games::common::cursors::shared::{
    CursorItem, MARKER_SIZE, Panes, Said, Spot, Where, near,
};
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::games::common::pictures::{EMPTY, MOVE};
use crate::games::common::screen::{ScreenText, distinct, in_rows};
use crate::py::{self, Error, Result};

/// The battle's cursors: a small picture alone in a layout, placed on a pane of its button.
pub struct PictureCursors {
    panes: Rc<Panes>,
    layouts: Rc<Layouts>,
    said: Rc<Said>,
    /// Cursor picture -> where it sat at the last poll it was found on a pane.
    spots: IndexMap<i64, Spot>,
    /// Cursor picture -> polls in a row it has not been found on a pane.
    missed: IndexMap<i64, i64>,
}

impl PictureCursors {
    pub fn new(panes: Rc<Panes>, said: Rc<Said>) -> Self {
        PictureCursors {
            layouts: panes.layouts.clone(),
            panes,
            said,
            spots: IndexMap::new(),
            missed: IndexMap::new(),
        }
    }

    /// Report the item under each cursor picture on screen, and return them, each with whether
    /// it is on the top screen.
    pub fn poll(
        &mut self,
        on_screen: &[ScreenText],
        items: &mut Vec<CursorItem>,
    ) -> Result<IndexMap<i64, bool>> {
        let markers = self.markers()?;
        let gone: Vec<i64> = self
            .spots
            .keys()
            .copied()
            .filter(|marker| !markers.contains_key(marker))
            .collect();
        for marker in gone {
            self.spots.shift_remove(&marker);
            self.missed.shift_remove(&marker);
            self.said.forget(marker);
        }
        for (&marker, &top) in &markers {
            let Some(spot) = self.spot(marker, top)? else {
                // A pane read mid-update can sit 1,000 units off for one poll (spike 50): a
                // cursor lost for one poll is not taken for one gone, and said again.
                let missed = self.missed.get(&marker).copied().unwrap_or(0) + 1;
                self.missed.insert(marker, missed);
                if missed > 1 {
                    self.said.forget(marker);
                }
                continue;
            };
            self.missed.insert(marker, 0);
            let item = self.item(&spot, top, on_screen)?;
            self.said
                .report(marker, item, items, Where::Pane(spot.anchor), false);
        }
        Ok(markers)
    }

    /// Cursor picture -> on the top screen, for every small picture alone in a layout shown.
    fn markers(&self) -> Result<IndexMap<i64, bool>> {
        let mut markers: IndexMap<i64, bool> = IndexMap::new();
        for root in self.layouts.roots() {
            let tree = self.layouts.tree(root);
            if tree.len() != 2 {
                continue;
            }
            let others: Vec<(i64, &String)> = tree
                .iter()
                .filter(|&(&pane, _name)| pane != root)
                .map(|(&pane, name)| (pane, name))
                .collect();
            let &[(marker, name)] = others.as_slice() else {
                return Err(Error::value(if others.is_empty() {
                    "not enough values to unpack (expected 1, got 0)".to_string()
                } else {
                    format!(
                        "too many values to unpack (expected 1, got {})",
                        others.len()
                    )
                }));
            };
            if name != lyt::PICTURE_CLASS {
                continue;
            }
            let head = self.layouts.pane(marker)?;
            let width = f32(&head, self.layouts.shape.width)?;
            if !(0.0 < width && width <= MARKER_SIZE) {
                continue;
            }
            let height = f32(&head, self.layouts.shape.height)?;
            if !(0.0 < height && height <= MARKER_SIZE) {
                continue;
            }
            let placement = self.layouts.placement(marker)?;
            if let Some(placement) = placement.filter(|placement| placement.shown) {
                markers.insert(marker, placement.top);
            }
        }
        Ok(markers)
    }

    /// The pane the cursor picture sits on, or None. Searched again only when it moved.
    fn spot(&mut self, marker: i64, top: bool) -> Result<Option<Spot>> {
        let shape = self.layouts.shape;
        let head = self.layouts.pane(marker)?;
        let (x, y) = (f32(&head, shape.x)?, f32(&head, shape.y)?);
        if let Some(last) = self.spots.get(&marker).copied()
            && near(last.x, x)
            && near(last.y, y)
        {
            let anchor = self.layouts.pane(last.anchor)?;
            let placement = self.layouts.placement(last.anchor)?;
            if near(f32(&anchor, shape.x)?, x)
                && near(f32(&anchor, shape.y)?, y)
                && placement.is_some_and(|placement| placement.shown)
            {
                return Ok(Some(last));
            }
        }
        let marker_root = u32(&head, lyt::PARENT)?;
        for root in self.layouts.roots() {
            if root == marker_root {
                continue;
            }
            let placement = self.layouts.placement(root)?;
            if !placement.is_some_and(|placement| placement.shown && placement.top == top) {
                continue;
            }
            for (&pane, name) in &self.layouts.tree(root) {
                if name != lyt::PANE_CLASS || pane == root {
                    continue;
                }
                let pane_head = self.layouts.pane(pane)?;
                if !(near(f32(&pane_head, shape.x)?, x) && near(f32(&pane_head, shape.y)?, y)) {
                    continue;
                }
                let pane_placement = self.layouts.placement(pane)?;
                if pane_placement.is_some_and(|placement| placement.shown) {
                    let spot = Spot {
                        x,
                        y,
                        anchor: pane,
                        root,
                    };
                    self.spots.insert(marker, spot);
                    return Ok(Some(spot));
                }
            }
        }
        self.spots.shift_remove(&marker);
        Ok(None)
    }

    /// The item a cursor picture's pane belongs to: the words under its group, or the words
    /// of the group's first picture's texture.
    pub fn item(
        &self,
        spot: &Spot,
        top: bool,
        on_screen: &[ScreenText],
    ) -> Result<Vec<ScreenText>> {
        let group = self.panes.parent(spot.anchor)?;
        let mut under: Vec<ScreenText> = Vec::new();
        for piece in on_screen {
            if self.panes.below(piece.r#box, group)? {
                under.push(piece.clone());
            }
        }
        let inside = distinct(in_rows(under));
        if !inside.is_empty() && !inside.iter().any(|p| p.text.chars().any(py::is_alnum)) {
            // Words with no letter or digit are no item: the battle's Bag draws an empty slot
            // dimmed with its times sign still in it, and was heard as "Times" (movie29, spike
            // 196).
            return Ok(vec![ScreenText {
                top,
                root: spot.root,
                parent: group,
                ..ScreenText::new(group, own(EMPTY, &[]), false)
            }]);
        }
        if !inside.is_empty() {
            return Ok(inside);
        }
        let Some(mut said) = self.panes.pictured(spot.root, group, on_screen, None)? else {
            return Ok(Vec::new());
        };
        if said.is_empty() {
            // A button drawn with no words: the move info's four, which only their places tell
            // apart, in the move list's order (or_move-learning).
            said = own(MOVE, &[("number", &self.number(spot.root, group)?)]);
        }
        Ok(vec![ScreenText {
            top,
            root: spot.root,
            parent: group,
            ..ScreenText::new(group, said, false)
        }])
    }

    /// A button's place among the buttons drawn like it, in reading order, from 1.
    fn number(&self, root: i64, group: i64) -> Result<i64> {
        let tree = self.layouts.tree(root);
        let place = |button: i64| -> Result<(i64, i64)> {
            let (mut x, mut y) = self.panes.place(button)?;
            let picture = self
                .layouts
                .children(root, button)
                .into_iter()
                .find(|pane| tree.get(pane).map(String::as_str) == Some(lyt::PICTURE_CLASS));
            if let Some(picture) = picture {
                (x, y) = (
                    x + self.panes.place(picture)?.0,
                    y + self.panes.place(picture)?.1,
                );
            }
            Ok((-py::round(y)?, py::round(x)?))
        };
        let mut keyed: Vec<((i64, i64), i64)> = Vec::new();
        for button in self.panes.alike(root, group, false)?.iter() {
            keyed.push((place(button)?, button));
        }
        let sorted = py::sorted_by_lt(keyed, |a, b| a.0 < b.0);
        let at = sorted
            .iter()
            .position(|&(_key, button)| button == group)
            .ok_or_else(|| Error::new(py::Kind::ValueError, format!("{group} is not in list")))?;
        Ok(at as i64 + 1)
    }
}
