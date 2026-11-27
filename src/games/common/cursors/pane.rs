//! Sun draws a ninth (spike 112), for every menu read so far — the title's, the X menu, the options
//! and the save prompt's Yes and No: a **cursor pane**, a plain pane alone under the root of a layout
//! of its own holding only the arrow's pictures, which the game moves and sizes onto the chosen item.
//! Each of the menu's buttons keeps an empty pane of exactly that place and size, and the item is
//! that button: all its words, as the title's Continue says the name, place and play time beside it.
//! Where the buttons' own panes are hidden and never placed, as the Yes and No's are, the item is
//! the words inside the cursor pane, as an app menu's cursor window's is. The talk window's answers
//! are pointed at as the Poké Mart's boxes' are, by a picture beside the words on its row, known by
//! where it stands, alone in a plain pane in the answers' window, since Sun's layout files are not
//! read to name its texture.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::words::own;
use crate::games::common::cursors::picture::PictureCursors;
use crate::games::common::cursors::shared::{
    CursorItem, Panes, SAME_SPOT, Said, Spot, Where, class_of, framed, near, place_of,
};
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::games::common::pictures::{EMPTY, pane_words};
use crate::games::common::screen::ScreenText;
use crate::py::Result;

/// Units a Sun cursor pane may be larger than the pane it is fitted to, each way. The battle's
/// command menu draws its corners in a pane 64 wide over a `Null_Cur` of 56 (spike 124); every
/// other menu's fits exactly, and among several panes in one place the nearest in size is taken.
pub const CURSOR_SLACK: f64 = 8.0;

/// Sun's cursor panes, each fitted to the button it is moved and sized onto.
///
/// Python's keeps the picture cursors (`pictures`), whose item a fitted pane's is; here they are
/// handed to `poll`, the reader holding both.
pub struct CursorPanes {
    panes: Rc<Panes>,
    layouts: Rc<Layouts>,
    said: Rc<Said>,
    /// Cursor pane on screen -> where it was at the last poll and the pane it was fitted to,
    /// an anchor of 0 where none was and the words inside it were its item.
    fits: IndexMap<i64, Spot>,
}

impl CursorPanes {
    pub fn new(panes: Rc<Panes>, said: Rc<Said>) -> Self {
        CursorPanes {
            layouts: panes.layouts.clone(),
            panes,
            said,
            fits: IndexMap::new(),
        }
    }

    /// Report the item under each of Sun's cursor panes on screen.
    pub fn poll(
        &mut self,
        on_screen: &[ScreenText],
        items: &mut Vec<CursorItem>,
        pictures: &PictureCursors,
    ) -> Result<()> {
        if !self.panes.cursor_panes {
            return Ok(());
        }
        let panes = self.panes_as_cursors()?;
        let gone: Vec<i64> = self
            .fits
            .keys()
            .copied()
            .filter(|pane| !panes.contains_key(pane))
            .collect();
        for pane in gone {
            self.fits.shift_remove(&pane);
            self.said.forget(pane);
        }
        for (&pane, &top) in &panes {
            let spot = self.fitted(pane, top)?;
            self.fits.insert(pane, spot);
            if spot.anchor != 0 {
                let mut item = pictures.item(&spot, top, on_screen)?;
                let group = self.panes.parent(spot.anchor)?;
                if item.is_empty() && self.words_hidden(spot.root, group)? {
                    // The X menu keeps a slot for every entry the story has yet to give, its words
                    // hidden, drawn as an empty plate the cursor still stops on (spike 112). A
                    // button of pictures alone, as a video call's, has no words to hide.
                    item = vec![ScreenText {
                        top,
                        root: spot.root,
                        parent: group,
                        ..ScreenText::new(group, own(EMPTY, &[]), false)
                    }];
                } else if item.is_empty()
                    && let Some(named) = self.button_name(spot.root, group)?
                {
                    // A button drawn as pictures alone is said by its pane's name, as ORAS says one
                    // by its texture's: the battle's `Btn_Fight`, `Btn_Bag`, `Btn_Pokemon_00` and
                    // `Btn_Run`, whose layout file Sun's readers do not have (spike 124).
                    item = vec![ScreenText {
                        top,
                        root: spot.root,
                        parent: group,
                        ..ScreenText::new(group, named, false)
                    }];
                }
                self.said
                    .report(pane, item, items, Where::Pane(spot.anchor), false);
                continue;
            }
            let Some(rect) = self.panes.rect(pane)? else {
                continue;
            };
            let inside = framed(on_screen, &rect);
            let at = place_of(&rect)?;
            self.said.report(pane, inside, items, at, false);
        }
        Ok(())
    }

    /// The words of a group's pane name, for a group holding no TextBox at all: `Btn_Fight` is
    /// "fight", `Btn_Pokemon_00` "pokémon", or pa3ds's own words for it where it has some. None
    /// for a group with words, or a name without any.
    fn button_name(&self, root: i64, group: i64) -> Result<Option<String>> {
        for (&pane, name) in &self.layouts.tree(root) {
            if name == lyt::TEXTBOX_CLASS && self.panes.below(pane, group)? {
                return Ok(None);
            }
        }
        let name = self.layouts.pane_name(group)?;
        Ok(pane_words(&name))
    }

    /// Does the group hold TextBoxes, none of them shown? Words shown but not yet read the same
    /// twice are words to wait for: said as empty, the X menu's SAVE was "empty" as it opened.
    fn words_hidden(&self, root: i64, group: i64) -> Result<bool> {
        let mut boxes: Vec<i64> = Vec::new();
        for (&pane, name) in &self.layouts.tree(root) {
            if name == lyt::TEXTBOX_CLASS && self.panes.below(pane, group)? {
                boxes.push(pane);
            }
        }
        if boxes.is_empty() {
            return Ok(false);
        }
        for &r#box in &boxes {
            let placement = self.layouts.placement(r#box)?;
            if placement.is_some_and(|placement| placement.shown) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Cursor pane -> on the top screen, for every plain pane shown alone under its layout's
    /// root with nothing but pictures under it.
    fn panes_as_cursors(&self) -> Result<IndexMap<i64, bool>> {
        let mut found: IndexMap<i64, bool> = IndexMap::new();
        for root in self.layouts.roots() {
            let tree = self.layouts.tree(root);
            let under = self.layouts.children(root, root);
            if under.len() != 1 || class_of(&tree, under[0]) != Some(lyt::PANE_CLASS) {
                continue;
            }
            let pictures: Vec<i64> = tree
                .keys()
                .copied()
                .filter(|&pane| pane != root && pane != under[0])
                .collect();
            if pictures.is_empty() || pictures.iter().any(|pane| tree[pane] != lyt::PICTURE_CLASS) {
                continue;
            }
            let placement = self.layouts.placement(under[0])?;
            if let Some(placement) = placement.filter(|placement| placement.shown) {
                found.insert(under[0], placement.top);
            }
        }
        Ok(found)
    }

    /// Where the cursor pane is, and the plain pane shown in another layout on the same screen
    /// with its place and size, or an anchor of 0 for none. Searched again only when the cursor
    /// has moved: a screen's every pane is read in the search.
    fn fitted(&self, cursor: i64, top: bool) -> Result<Spot> {
        let (x, y) = self.panes.place(cursor)?;
        if let Some(last) = self.fits.get(&cursor)
            && near(last.x, x)
            && near(last.y, y)
        {
            return Ok(*last);
        }
        let (width, height) = self.panes.size(cursor)?;
        let own = self.panes.parent(cursor)?;
        let mut fits: Vec<(f64, Spot)> = Vec::new();
        for root in self.layouts.roots() {
            let placement = self.layouts.placement(root)?;
            if root == own
                || !placement.is_some_and(|placement| placement.shown && placement.top == top)
            {
                continue;
            }
            for (&pane, name) in &self.layouts.tree(root) {
                if name != lyt::PANE_CLASS || pane == root {
                    continue;
                }
                let (at, size) = (self.panes.place(pane)?, self.panes.size(pane)?);
                let slack = max_of(width - size.0, height - size.1);
                if !(near(at.0, x)
                    && near(at.1, y)
                    && -SAME_SPOT <= width - size.0
                    && width - size.0 <= CURSOR_SLACK
                    && -SAME_SPOT <= height - size.1
                    && height - size.1 <= CURSOR_SLACK)
                {
                    continue;
                }
                let shown = self.layouts.placement(pane)?;
                if shown.is_some_and(|shown| shown.shown) {
                    fits.push((
                        slack,
                        Spot {
                            x,
                            y,
                            anchor: pane,
                            root,
                            worded: false,
                        },
                    ));
                }
            }
        }
        let Some(&first) = fits.first() else {
            return Ok(Spot {
                x,
                y,
                anchor: 0,
                root: 0,
                worded: false,
            });
        };
        // `min(fits, key=...)`: the first of the least.
        let mut best = first;
        for &fit in &fits[1..] {
            if fit.0 < best.0 {
                best = fit;
            }
        }
        Ok(best.1)
    }
}

/// Python's `max(a, b)` on floats: `a`, unless `b > a`, so a NaN stays where it is.
fn max_of(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}
