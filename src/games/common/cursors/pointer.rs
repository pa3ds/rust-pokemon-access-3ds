//! Wally's catching tutorial draws an eighth (spike 96): a **pointer**, a small picture alone in a
//! layout but for the one plain pane that places it, which the game puts on the button about to be
//! tapped. The item is the button whose touch area holds that place, in whatever layout came on that
//! screen last; where another cursor claims that layout, the pointer says nothing.

use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::games::common::cursors::shared::{CursorItem, MARKER_SIZE, Panes, Said, Where, on};
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::games::common::screen::{ScreenText, distinct, in_rows};
use crate::py::{PySet, Result};

/// The hand that shows a tap in the battle the player watches.
pub struct Pointers {
    panes: Rc<Panes>,
    layouts: Rc<Layouts>,
    said: Rc<Said>,
    /// Pointers on screen at the last poll.
    pointed: IndexSet<i64>,
    polls: i64,
    /// Layout root -> the poll it came on screen at, for layouts shown now.
    shown_since: IndexMap<i64, i64>,
}

impl Pointers {
    pub fn new(panes: Rc<Panes>, said: Rc<Said>) -> Self {
        Pointers {
            layouts: panes.layouts.clone(),
            panes,
            said,
            pointed: IndexSet::new(),
            polls: 0,
            shown_since: IndexMap::new(),
        }
    }

    /// Say the button a pointer is about to tap, where no other cursor claims its layout.
    ///
    /// Wally's catching tutorial is the one battle the player watches rather than plays, and has
    /// no battle cursor: the game shows his taps with a picture in a layout of its own, under a
    /// pane it places on the button he is about to tap, slides the picture onto it and grows it
    /// for the tap (spike 96). Its item is the button whose touch area holds that place, said
    /// once it has read the same for two polls, and meanwhile the button's layout is the
    /// pointer's, so the four moves are not read out whole.
    pub fn poll(&mut self, on_screen: &[ScreenText], items: &mut Vec<CursorItem>) -> Result<()> {
        self.note_shown()?;
        let pointers = self.pointers()?;
        let gone: Vec<i64> = self
            .pointed
            .iter()
            .copied()
            .filter(|pointer| !pointers.contains_key(pointer))
            .collect();
        for pointer in gone {
            self.pointed.shift_remove(&pointer);
            self.said.unsteady(pointer);
            self.said.forget(pointer);
        }
        if pointers.is_empty() {
            return Ok(());
        }
        let claimed = self
            .said
            .claimed_roots(&|cursor| pointers.contains_key(&cursor));
        for (&pointer, &(x, y, top)) in &pointers {
            self.pointed.insert(pointer);
            let found = self.touched(x, y, top)?;
            let Some((root, button)) = found.filter(|found| !claimed.contains(&found.0)) else {
                self.said.over_nothing(pointer);
                continue;
            };
            let mut under: Vec<ScreenText> = Vec::new();
            for piece in on_screen {
                if self.panes.below(piece.r#box, button)? {
                    under.push(piece.clone());
                }
            }
            let mut inside = distinct(in_rows(under));
            if inside.is_empty() {
                let Some(said) = self.panes.pictured(root, button, on_screen, None)? else {
                    continue;
                };
                inside = vec![ScreenText {
                    top,
                    root,
                    parent: button,
                    ..ScreenText::new(button, said, false)
                }];
            }
            self.said
                .report_steady(pointer, inside, items, Where::Pane(button), false);
        }
        Ok(())
    }

    /// Picture -> (x, y, on the top screen) of the place it points at, for every small picture
    /// shown alone in a layout but for the one plain pane that places it: Wally's pointer, whose
    /// pane stands where he is about to tap while the picture slides onto it.
    fn pointers(&self) -> Result<IndexMap<i64, (f64, f64, bool)>> {
        let mut found: IndexMap<i64, (f64, f64, bool)> = IndexMap::new();
        for root in self.layouts.roots() {
            let tree = self.layouts.tree(root);
            if tree.len() != 3 {
                continue;
            }
            let holders: Vec<i64> = tree
                .iter()
                .filter(|&(&pane, name)| name == lyt::PANE_CLASS && pane != root)
                .map(|(&pane, _name)| pane)
                .collect();
            let pictures: Vec<i64> = tree
                .iter()
                .filter(|&(_pane, name)| name == lyt::PICTURE_CLASS)
                .map(|(&pane, _name)| pane)
                .collect();
            if holders.len() != 1 || pictures.len() != 1 {
                continue;
            }
            let (holder, picture) = (holders[0], pictures[0]);
            if self.panes.parent(picture)? != holder {
                continue;
            }
            let (width, height) = self.panes.size(picture)?;
            if !(0.0 < width && width <= MARKER_SIZE && 0.0 < height && height <= MARKER_SIZE) {
                continue;
            }
            let placement = self.layouts.placement(picture)?;
            let Some(placement) = placement.filter(|placement| placement.shown) else {
                continue;
            };
            let (x, y) = self.panes.place(holder)?;
            found.insert(picture, (x, y, placement.top));
        }
        Ok(found)
    }

    /// (root, button) for the touch area shown at this place on this screen, where a button
    /// is what holds the touch area, as a move's plate holds its own.
    ///
    /// Any layout shown, words or none: the battle's command menu is all pictures. Of two, the
    /// one that came on screen last, as the one before can still be up behind it: the bag's
    /// LAST USED ITEM stood under Wally's tap on USE (wally_again.ctm replay); then the smaller.
    fn touched(&self, x: f64, y: f64, top: bool) -> Result<Option<(i64, i64)>> {
        let mut found: Vec<(i64, f64, i64, i64)> = Vec::new();
        for (&root, &since) in &self.shown_since {
            for (&pane, name) in &self.layouts.tree(root) {
                if name != lyt::BOUNDING_CLASS {
                    continue;
                }
                let rect = self.panes.rect(pane)?;
                let Some(rect) = rect.filter(|rect| rect.top == top && on((x, y), rect)) else {
                    continue;
                };
                let area = (rect.right - rect.left) * (rect.high - rect.low);
                found.push((-since, area, root, self.panes.parent(pane)?));
            }
        }
        let Some(&first) = found.first() else {
            return Ok(None);
        };
        // `min` keeps the first of the least, and a tuple with a NaN in it is less than none.
        let mut best = first;
        for &item in &found[1..] {
            if found_lt(&item, &best) {
                best = item;
            }
        }
        Ok(Some((best.2, best.3)))
    }

    /// Remember the poll each layout came on screen at, for layouts shown now.
    fn note_shown(&mut self) -> Result<()> {
        self.polls += 1;
        let mut shown = PySet::new();
        for root in self.layouts.roots() {
            let placement = self.layouts.placement(root)?;
            if placement.is_some_and(|placement| placement.shown) {
                shown.add(root);
            }
        }
        self.shown_since = shown
            .iter()
            .map(|root| {
                (
                    root,
                    self.shown_since.get(&root).copied().unwrap_or(self.polls),
                )
            })
            .collect();
        Ok(())
    }
}

/// `a < b` for the tuples `(-since, area, root, parent)`: the first place they differ decides,
/// and a NaN differs from everything, itself included, and is less than nothing.
fn found_lt(a: &(i64, f64, i64, i64), b: &(i64, f64, i64, i64)) -> bool {
    if a.0 != b.0 {
        return a.0 < b.0;
    }
    if a.1 != b.1 {
        return a.1 < b.1;
    }
    if a.2 != b.2 {
        return a.2 < b.2;
    }
    a.3 < b.3
}
