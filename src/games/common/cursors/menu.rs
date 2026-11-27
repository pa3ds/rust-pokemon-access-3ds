//! App menus draw their cursor — the red corners around the chosen item — as a layout of its own: a
//! root pane whose only child is an `nw::lyt::Window` (spike 40). The window sits on the chosen item
//! at the item's size (spike 39), so the item under the cursor is the screen text inside the window's
//! rectangle, on the screen the cursor is drawn to (spikes 40 and 41). The X menu, the bag and the save
//! prompt's list all draw theirs this way.
//!
//! ```text
//! Window  +0x0c -> the layout's root    +0x48 f32 width    +0x4c f32 height
//!         +0x80 f32 global scale x      +0x8c f32 x        +0x94 f32 global scale y   +0x9c f32 y
//! root    +0x0c 0, a root has no parent                    +0x10 u32 child count, 1 for a cursor
//! ```
//!
//! Sun's name entry keyboard builds its cursor the same way, as a window alone under a Parts pane, a
//! layout built into the keys' own (spike 112).

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::u32;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::{
    CursorItem, Panes, Rect, Said, framed, near, place_of,
};
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::games::common::screen::{ScreenText, distinct, in_rows};
use crate::py::{self, Result};

/// Units a cursor's width or height may differ from the pane it sits on: the Bag's is 157 by 42
/// on rows of 150 by 30, the Pokéblock Case's 52 by 52 on cells of 50 by 50 (spike 196).
pub const ON_PANE: f64 = 16.0;
/// What a menu cursor framing no words is on, from its rectangle, the words of its layout on
/// screen and the named pictures it frames, or none, said before those pictures: the screen's own
/// knowledge of a field left blank or shown as a picture.
pub type Blank = Rc<dyn Fn(&Rect, &[ScreenText], &[ScreenText]) -> Vec<ScreenText>>;

/// The windows that are an app menu's cursor, each read on every poll once found.
pub struct MenuCursors {
    panes: Rc<Panes>,
    blank: Option<Blank>,
    layouts: Rc<Layouts>,
    said: Rc<Said>,
    names: Rc<ClassNames>,
    /// The cursors found so far.
    pub windows: Vec<i64>,
    /// The cursors on screen, as of the last poll.
    pub shown: Vec<i64>,
    /// The TextBoxes of the rows drawn like the ones the cursors are over, as of the last
    /// poll: the Bag's list writes an item into a row as it scrolls it into view, and "Soft
    /// Sand" was said as new beside the cursor's "Quick Claw" (movie29, spike 196).
    pub rows: HashSet<i64>,
}

impl MenuCursors {
    pub fn new(
        panes: Rc<Panes>,
        said: Rc<Said>,
        names: Rc<ClassNames>,
        blank: Option<Blank>,
    ) -> Self {
        MenuCursors {
            blank,
            layouts: panes.layouts.clone(),
            panes,
            said,
            names,
            windows: Vec::new(),
            shown: Vec::new(),
            rows: HashSet::new(),
        }
    }

    /// Read the cursors among these windows from now on, as well as every one found before.
    ///
    /// Most windows are not cursors, so only those shaped as one when found are kept. A cursor
    /// once found is read on every poll after: menus are rebuilt where they were, and a sweep
    /// takes seconds to find a new one. Cursor pictures are found in the layouts themselves.
    pub fn track(&mut self, windows: impl IntoIterator<Item = i64>) -> Result<()> {
        for window in windows {
            if !self.windows.contains(&window) && self.shaped(window)? {
                self.windows.push(window);
            }
        }
        Ok(())
    }

    /// Report the item under each menu cursor on screen.
    pub fn poll(&mut self, on_screen: &[ScreenText], items: &mut Vec<CursorItem>) -> Result<()> {
        self.rows = HashSet::new();
        self.shown = Vec::new();
        for &cursor in &self.windows {
            let Some(rect) = self.rect(cursor)? else {
                self.said.forget(cursor);
                continue;
            };
            self.shown.push(cursor);
            // A cursor built into a layout frames that layout's words: Sun's keyboard keeps a
            // second, parked over the keys of the page drawn in front of it (spike 112).
            let own = self.built_into(cursor)?;
            let theirs: Vec<ScreenText> = on_screen
                .iter()
                .filter(|piece| own.is_none() || own == Some(piece.root))
                .cloned()
                .collect();
            let mut inside = self.on_pane(&theirs, &rect)?;
            if inside.is_empty() {
                inside = framed(&theirs, &rect);
            }
            if inside.is_empty() {
                inside = self.pictures_inside(&rect)?;
            }
            let mut rows: HashSet<(i64, i64)> = HashSet::new();
            for piece in &inside {
                if piece.parent != piece.root {
                    for row in self.rows_of(piece)?.iter() {
                        rows.insert((piece.root, row));
                    }
                }
            }
            self.rows.extend(
                on_screen
                    .iter()
                    .filter(|piece| {
                        rows.contains(&(piece.root, piece.parent))
                            || rows.contains(&(piece.root, piece.panel))
                    })
                    .map(|piece| piece.r#box),
            );
            if let Some(blank) = &self.blank
                && inside.iter().all(|piece| piece.picture)
            {
                // A field with nothing in it, said by what its screen names it: the Pokédex's
                // search and sort frames TYPE's and SHAPE's values alone, blank until set, and
                // the cursor on them said nothing (or_pokedex). Words beside it, no row of its.
                // So with a picture alone: SHAPE set said its shape without its label, "Fins"
                // (heard in play).
                inside = [blank(&rect, &theirs, &inside), inside].concat();
            }
            let at = place_of(&rect)?;
            let said = self.said.last(cursor);
            let before: HashSet<i64> = self.said.under(cursor).iter().map(|p| p.r#box).collect();
            let now: HashSet<i64> = inside.iter().map(|p| p.r#box).collect();
            if let Some(said) = said
                && said.1 == at
                && now.difference(&before).next().is_some()
                && now.intersection(&before).next().is_some()
            {
                // Words coming beside those under a cursor that has not moved are an item still
                // being laid out: the fitting room writes the next colour into the card under its
                // cursor and shows its price or "Wearing" a poll before it hides the other, and
                // each move was said twice, "Two-Tone Bag black $2,900 Wearing" and then
                // "Two-Tone Bag black Wearing" (x_cyllage_boutique_fitting_room). Held a poll
                // otherwise, an item lost the details that changed with it: words rewritten in
                // place, the Bag's next item with its description (or_dexnav_chain_linoone), a
                // row slid under the cursor, and words only leaving, the AreaNav's leader card
                // with its town's facilities (or_areanav_by_touch).
                self.said.note(cursor, inside);
            } else {
                let rows = rows.iter().map(|&(_root, row)| row).min().unwrap_or(0);
                self.said
                    .report_in(cursor, inside, items, at, false, rows, false);
            }
        }
        Ok(())
    }

    /// The row a cursor is over and the rows drawn like it: its words' parent's, or, where
    /// that pane stands alone, its panel's. Each row of the PC's menus is a panel of its own, its
    /// words two panes below, and the rows the cursor was not on, "Log Out", "Organize Items",
    /// were said as new (movie6).
    ///
    /// Python's `_rows`: `rows` is the field it adds to.
    fn rows_of(&self, piece: &ScreenText) -> Result<py::PySet> {
        let alike = self.panes.alike(piece.root, piece.parent, false)?;
        if alike.len() == 1 && piece.panel != 0 && piece.panel != piece.parent {
            let panels = self.panes.alike(piece.root, piece.panel, false)?;
            if panels.len() > 1 {
                return Ok(panels);
            }
            // The fitting room's list is a column of cards, each in a pane of its own, the
            // cursor on the middle one and the next and the last peeking at the screen's edges:
            // their names were said as each move's details (x_cyllage_boutique_fitting_room).
            return self.panes.alike_in(piece.root, piece.parent, piece.panel);
        }
        Ok(alike)
    }

    /// The words under the one pane the cursor sits on, centred where it is and of about its
    /// size, or none when no layout, or more than one, has such a pane with words under it.
    ///
    /// The Pokéblock Kit draws its case's grid over the Berry Blender's, both shown, and the
    /// case's cursor sat on a block's cell, 50 by 50 where it is 52 by 52, its count just below
    /// its frame; framed, it said the count of the blender's cell behind, nearer its middle
    /// (movie30, spike 196).
    fn on_pane(&self, pieces: &[ScreenText], rect: &Rect) -> Result<Vec<ScreenText>> {
        let (x, y) = ((rect.left + rect.right) / 2.0, (rect.low + rect.high) / 2.0);
        let (width, height) = (rect.right - rect.left, rect.high - rect.low);
        let mut found: IndexMap<i64, Vec<ScreenText>> = IndexMap::new();
        let roots: IndexSet<i64> = pieces
            .iter()
            .filter(|piece| piece.top == rect.top)
            .map(|piece| piece.root)
            .collect();
        for root in roots {
            let mine: Vec<&ScreenText> = pieces.iter().filter(|piece| piece.root == root).collect();
            for &pane in self.layouts.tree(root).keys() {
                if pane == root
                    || !(near(self.panes.place(pane)?.0, x) && near(self.panes.place(pane)?.1, y))
                {
                    continue;
                }
                let (pane_width, pane_height) = self.panes.size(pane)?;
                if (pane_width - width).abs() > ON_PANE || (pane_height - height).abs() > ON_PANE {
                    continue;
                }
                let mut under: Vec<ScreenText> = Vec::new();
                for piece in &mine {
                    if self.panes.below(piece.parent, pane)? {
                        under.push((*piece).clone());
                    }
                }
                if !under.is_empty() {
                    found.insert(root, distinct(in_rows(under)));
                    break;
                }
            }
        }
        let items: Vec<Vec<ScreenText>> = found.into_values().collect();
        let firsts: HashSet<&str> = items.iter().map(|item| item[0].text.as_str()).collect();
        if items.len() > 1 && firsts.len() == 1 {
            // One item drawn by two layouts at one place, one of them not shown, which nothing
            // read so far tells: a summary's moves, its battle page's and its contest page's
            // (movie32, spike 196). Said whole once, "Meteor Mash Steel PP 10/10 Coolness".
            let texts: HashSet<&str> = items[0].iter().map(|piece| piece.text.as_str()).collect();
            let mut whole = items[0].clone();
            for item in &items[1..] {
                for piece in item {
                    if !texts.contains(piece.text.as_str()) {
                        whole.push(piece.clone());
                    }
                }
            }
            return Ok(whole);
        }
        Ok(if items.len() == 1 {
            items[0].clone()
        } else {
            Vec::new()
        })
    }

    /// Is this a window alone under the root of its layout, holding nothing of its own? In
    /// Sun, alone under a Parts pane, the root of a layout built into another, is alone too.
    ///
    /// A window holding panes is a box its words are drawn in: the Poké Mart's Yes and No are
    /// two TextBoxes in one, alone under its root, and read as a cursor it said "Yes No" and
    /// never moved (spike 98). The corners of an app menu's cursor hold nothing.
    fn shaped(&self, window: i64) -> Result<bool> {
        if self.names.of_object(window)?.as_deref() != Some(lyt::WINDOW_CLASS) {
            return Ok(false);
        }
        let head = self.layouts.pane(window)?;
        if u32(&head, lyt::CHILD_COUNT)? != 0 {
            return Ok(false);
        }
        let root = u32(&head, lyt::PARENT)?;
        if root == 0 {
            return Ok(false);
        }
        let head = self.layouts.pane(root)?;
        if u32(&head, lyt::CHILD_COUNT)? != 1 {
            return Ok(false);
        }
        if u32(&head, lyt::PARENT)? == 0 {
            return Ok(true);
        }
        // Sun's keyboard builds its cursor as a layout of its own inside the keys' layout: the
        // window alone under a Parts pane, moved and sized onto the chosen key (spike 112).
        Ok(self.panes.cursor_panes
            && self.names.of_vtable(u32(&head, 0)?)?.as_deref() == Some(lyt::PARTS_CLASS))
    }

    /// Where the cursor is, or None when it is not shaped as a cursor or not on screen.
    fn rect(&self, cursor: i64) -> Result<Option<Rect>> {
        if !self.shaped(cursor)? {
            return Ok(None);
        }
        self.panes.rect(cursor)
    }

    /// The root of the layout a cursor window is built into, as Sun's keyboard's is under a
    /// Parts pane, or None for a cursor that is a layout of its own.
    fn built_into(&self, window: i64) -> Result<Option<i64>> {
        let parent = self.panes.parent(window)?;
        let vtable = u32(&self.layouts.pane(parent)?, 0)?;
        if self.names.of_vtable(vtable)?.as_deref() != Some(lyt::PARTS_CLASS) {
            return Ok(None);
        }
        let placement = self.layouts.placement(window)?;
        Ok(placement.map(|placement| placement.root))
    }

    /// What the game calls the pictures shown inside a cursor with no words under it: the
    /// Pokédex's language tabs, whose textures the game swaps as it runs, each drawn as two
    /// pictures that name one language (spike 147).
    fn pictures_inside(&self, rect: &Rect) -> Result<Vec<ScreenText>> {
        let Some(pictures) = self.panes.pictures.clone() else {
            return Ok(Vec::new());
        };
        let mut said: IndexMap<String, ScreenText> = IndexMap::new();
        for root in self.layouts.roots() {
            let placement = self.layouts.placement(root)?;
            if !placement.is_some_and(|placement| placement.shown && placement.top == rect.top) {
                continue;
            }
            for (&pane, name) in &self.layouts.tree(root) {
                if name != lyt::PICTURE_CLASS {
                    continue;
                }
                let (x, y) = self.panes.place(pane)?;
                let shown = self.layouts.placement(pane)?;
                let inside = rect.left <= x && x <= rect.right && rect.low <= y && y <= rect.high;
                if !inside || !shown.is_some_and(|shown| shown.shown) {
                    continue;
                }
                let words = pictures(pane)?;
                if let Some(words) = words
                    && !words.is_empty()
                    && !said.contains_key(&words)
                {
                    let piece = ScreenText {
                        top: rect.top,
                        root,
                        parent: self.panes.parent(pane)?,
                        panel: shown.as_ref().map_or(0, |shown| shown.panel),
                        picture: true,
                        ..ScreenText::new(pane, words.clone(), false)
                    };
                    said.insert(words, piece);
                }
            }
        }
        Ok(said.into_values().collect())
    }
}
