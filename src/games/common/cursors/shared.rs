//! What every kind of cursor shares: where a pane is drawn, what a picture's texture says, and
//! an item said once, as it comes on screen and whenever it changes.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::words::own;
use crate::games::common::layout::{Layouts, MAX_DEPTH};
use crate::games::common::layout_file::{LayoutFile, LayoutFiles};
use crate::games::common::lyt;
use crate::games::common::pictures::{EMPTY, words_in};
use crate::games::common::screen::{Pictures, ScreenText, distinct, in_rows};
use crate::games::common::slots::Occupant;
use crate::py::{self, Result};

/// The biggest side of a picture taken for a cursor. The battle's is 20 by 20; a lone picture the
/// size of a screen is a dimmer.
pub const MARKER_SIZE: f64 = 64.0;
/// Units within which a cursor picture sits on a pane. The battle's lands exactly.
pub const SAME_SPOT: f64 = 1.0;
/// The words a texture's name carries when the designer drew a cursor with it, or its whole name:
/// the Poké Mart's `shop_cursor.bclim` (spike 95), the PC Box's `cursol01.bclim`, as they spelt it
/// there (2026-09-27), and the Bag's `select_bar.bclim`, the plate the chosen Pokémon is drawn on as
/// a TM is taught (spike 139). "select" alone is no cursor: the PC Box draws its grid with
/// `select_box01.bclim`. The ROM's other spellings wait for a movie that reaches them: `cursol` on the
/// field's bottom-screen buttons, the Town Map and a minigame, and `cursor1` in the move lists.
pub const CURSOR_TEXTURES: [&str; 3] = ["cursor", "cursol01", "select_bar"];

/// The pieces of screen text a cursor is over, when it came on screen or moved.
#[derive(Debug, Clone, PartialEq)]
pub struct CursorItem {
    pub cursor: i64,
    pub pieces: Vec<ScreenText>,
    /// True for the first item said since the cursor came on screen; false when it moved.
    pub opened: bool,
    /// No details follow it: an empty slot, over which the top screen's summary keeps showing
    /// the Pokémon carried or the one last passed (spike 129).
    pub quiet: bool,
    /// A pane of the list of rows it is one of, where its cursor knows it (`MenuCursors`), or 0:
    /// the details kept from moves along one list are not another's.
    pub rows: i64,
}

impl CursorItem {
    /// `CursorItem(cursor, pieces, opened)`: `quiet` and `rows` left to their defaults, False
    /// and 0.
    pub fn new(cursor: i64, pieces: Vec<ScreenText>, opened: bool) -> Self {
        CursorItem {
            cursor,
            pieces,
            opened,
            quiet: false,
            rows: 0,
        }
    }
}

/// What one cursor poll found (`CursorReader.poll`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CursorPoll {
    /// What each cursor on screen is over, whenever that changed.
    pub items: Vec<CursorItem>,
    /// The pieces under each cursor on screen.
    pub under: Vec<Vec<ScreenText>>,
    /// Layout root -> the TextBoxes of the list a cursor walks, for the cursors that know theirs
    /// (`ListCursors.lists`).
    pub lists: IndexMap<i64, HashSet<i64>>,
    /// The cursors on their way to an item they have not said yet (`ListCursors.moving`).
    pub moving: HashSet<i64>,
    /// The TextBoxes of the rows drawn like those an app menu's cursor is over
    /// (`MenuCursors.rows`): its list's, which are its to say.
    pub rows: HashSet<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub top: bool,
    pub left: f64,
    pub right: f64,
    pub low: f64,
    pub high: f64,
}

/// Where a cursor picture was, and the pane it sat on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    pub x: f64,
    pub y: f64,
    pub anchor: i64,
    pub root: i64,
}

/// What the Python keeps as `where: object`: where a cursor was when it last said an item, a place
/// to the unit or the pane it sat on, which is only ever compared for equality. `Nothing` is the
/// `None` a said item is reset to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Where {
    Nothing,
    Pane(i64),
    Place(i64, i64),
}

/// `tuple[tuple[str, ...], object]`: the words last reported and where they were.
pub type Words = (Vec<String>, Where);

/// The layouts read through, and what is known of their panes' textures.
///
/// Shared by every kind of cursor, as one object in Python, so its caches are in `RefCell`s and its
/// methods take `&self`.
pub struct Panes {
    pub layouts: Rc<Layouts>,
    pub files: Option<Rc<LayoutFiles>>,
    /// What the game calls what a picture draws now, by its image (`gen6.rom.textures`), or
    /// None.
    pub pictures: Option<Pictures>,
    /// Who is in a PC Box's slot, said while a Pokémon is carried (`slots`), or None.
    pub occupant: Option<Occupant>,
    /// Whether Sun's cursor pane and its boxes' pointers are read: ORAS and X have none.
    pub cursor_panes: bool,
    /// (root, group) -> the words of the group's first picture's texture, once known.
    picture_words: RefCell<IndexMap<(i64, i64), Option<String>>>,
    /// (root, picture) -> whether its texture is named a cursor, once known.
    cursor_textures: RefCell<IndexMap<(i64, i64), bool>>,
}

impl Panes {
    pub fn new(
        layouts: Rc<Layouts>,
        files: Option<Rc<LayoutFiles>>,
        pictures: Option<Pictures>,
        cursor_panes: bool,
        occupant: Option<Occupant>,
    ) -> Self {
        Panes {
            layouts,
            files,
            pictures,
            occupant,
            cursor_panes,
            picture_words: RefCell::new(IndexMap::new()),
            cursor_textures: RefCell::new(IndexMap::new()),
        }
    }

    /// The file a layout was built from, where layout files are read and it is found.
    pub fn file(&self, root: i64) -> Option<Rc<LayoutFile>> {
        self.files
            .as_ref()
            .and_then(|files| files.matching(&self.layouts.pane_names(root)))
    }

    /// Where a pane is placed in its parent.
    pub fn place(&self, pane: i64) -> Result<(f64, f64)> {
        let head = self.layouts.pane(pane)?;
        Ok((
            f32(&head, self.layouts.shape.x)?,
            f32(&head, self.layouts.shape.y)?,
        ))
    }

    pub fn size(&self, pane: i64) -> Result<(f64, f64)> {
        let head = self.layouts.pane(pane)?;
        Ok((
            f32(&head, self.layouts.shape.width)?,
            f32(&head, self.layouts.shape.height)?,
        ))
    }

    /// Where a window is drawn, or None when it is not on screen or has no size yet.
    pub fn rect(&self, window: i64) -> Result<Option<Rect>> {
        let placement = self.layouts.placement(window)?;
        let Some(placement) = placement.filter(|placement| placement.shown) else {
            return Ok(None);
        };
        let head = self.layouts.pane(window)?;
        let shape = self.layouts.shape;
        let half_width = f32(&head, shape.width)? * f32(&head, shape.scale_x)? / 2.0;
        let half_height = f32(&head, shape.height)? * f32(&head, shape.scale_y)? / 2.0;
        if half_width <= 0.0 || half_height <= 0.0 {
            return Ok(None);
        }
        let (x, y) = (
            f32(&head, self.layouts.shape.x)?,
            f32(&head, self.layouts.shape.y)?,
        );
        Ok(Some(Rect {
            top: placement.top,
            left: x - half_width,
            right: x + half_width,
            low: y - half_height,
            high: y + half_height,
        }))
    }

    /// Is `pane` the group or somewhere under it?
    pub fn below(&self, mut pane: i64, group: i64) -> Result<bool> {
        for _ in 0..MAX_DEPTH {
            if pane == group {
                return Ok(true);
            }
            if pane == 0 {
                return Ok(false);
            }
            pane = self.parent(pane)?;
        }
        Ok(false)
    }

    /// The first texture a picture pane's layout file gives it, or None.
    pub fn texture(&self, pane: i64, file: &LayoutFile) -> Result<Option<String>> {
        let name = self.pane_name(pane)?;
        Ok(file
            .textures
            .get(&name)
            .and_then(|textures| textures.first().cloned()))
    }

    /// The words of the texture of the first picture in a group, from its layout's file.
    ///
    /// A texture that also lies under words is a plate the words are written on, not a label:
    /// the move list's empty fourth slot is the plate every move is written on, and read as
    /// "waza plate 00" (battles.ctm replay). Such a group is said as empty. `skip` is a picture to
    /// pass over: a cursor standing in the group is not what the group is.
    pub fn pictured(
        &self,
        root: i64,
        group: i64,
        on_screen: &[ScreenText],
        skip: Option<i64>,
    ) -> Result<Option<String>> {
        if let Some(known) = self.picture_words.borrow().get(&(root, group)) {
            return Ok(known.clone());
        }
        let Some(file) = self.file(root) else {
            return Ok(None);
        };
        let tree = self.layouts.tree(root);
        let mut texture: Option<String> = None;
        for (&pane, name) in &tree {
            if name == lyt::PICTURE_CLASS && Some(pane) != skip && self.below(pane, group)? {
                texture = self.texture(pane, &file)?;
                if texture.is_some() {
                    break;
                }
            }
        }
        let mut said: Option<String> = texture
            .as_deref()
            .map(|texture| words_in(texture, &file.name));
        if let Some(texture) = &texture {
            for piece in on_screen {
                if piece.root != root {
                    continue;
                }
                let mut pane = self.parent(piece.r#box)?;
                for _ in 0..MAX_DEPTH {
                    if pane == 0 || pane == root {
                        break;
                    }
                    if class_of(&tree, pane) == Some(lyt::PICTURE_CLASS)
                        && self.texture(pane, &file)?.as_ref() == Some(texture)
                    {
                        said = Some(own(EMPTY, &[]));
                        break;
                    }
                    pane = self.parent(pane)?;
                }
                if said == Some(own(EMPTY, &[])) {
                    break;
                }
            }
        }
        self.picture_words
            .borrow_mut()
            .insert((root, group), said.clone());
        Ok(said)
    }

    /// A row of a list and the rows drawn like it: its parent's children of its class, at its
    /// x and of its size. The Poké Mart's rows share one window; each row of the PC Box's menu
    /// for a chosen Pokémon is a picture of its own (or/movie6_pc, 2026-09-27). Not in one
    /// `column`, the cells of a grid: the Pokéblock Case's (spike 196).
    pub fn alike(&self, root: i64, row: i64, column: bool) -> Result<py::PySet> {
        let tree = self.layouts.tree(root);
        let parent = self.parent(row)?;
        let ((x, _y), (width, height)) = (self.place(row)?, self.size(row)?);
        let mut alike: Vec<i64> = Vec::new();
        for pane in self.layouts.children(root, parent) {
            if class_of(&tree, pane) == class_of(&tree, row)
                && (!column || near(self.place(pane)?.0, x))
                && near(self.size(pane)?.0, width)
                && near(self.size(pane)?.1, height)
            {
                alike.push(pane);
            }
        }
        // `{row} | {...}`, a set iterated in its order: a move info button's number is its place
        // among them (`PictureCursors.number`).
        Ok(py::PySet::of([row]).union(&py::PySet::of(alike)))
    }

    /// A row and the rows drawn like it anywhere in its panel, in one column: its class, at
    /// its x and of its size, each under a pane of its own (`MenuCursors._rows`).
    pub fn alike_in(&self, root: i64, row: i64, panel: i64) -> Result<py::PySet> {
        let tree = self.layouts.tree(root);
        let ((x, _y), (width, height)) = (self.place(row)?, self.size(row)?);
        let mut alike: Vec<i64> = Vec::new();
        for &pane in tree.keys() {
            if class_of(&tree, pane) == class_of(&tree, row)
                && self.below(pane, panel)?
                && near(self.place(pane)?.0, x)
                && near(self.size(pane)?.0, width)
                && near(self.size(pane)?.1, height)
            {
                alike.push(pane);
            }
        }
        Ok(py::PySet::of([row]).union(&py::PySet::of(alike)))
    }

    /// Is this picture or window drawn with a texture its designer named a cursor? A pane
    /// that is neither has no texture in its layout's file, and is not. Without the file, as in
    /// Sun, a picture is a cursor by its place: a box's pointer.
    pub fn drawn_as_cursor(&self, root: i64, pane: i64, file: Option<&LayoutFile>) -> Result<bool> {
        if !self.cursor_textures.borrow().contains_key(&(root, pane)) {
            let Some(file) = file else {
                let drawn = self.points_in_box(root, pane)?;
                self.cursor_textures
                    .borrow_mut()
                    .insert((root, pane), drawn);
                return Ok(drawn);
            };
            let name = self.pane_name(pane)?;
            let textures = match file.textures.get(&name) {
                Some(textures) if !textures.is_empty() => Some(textures),
                _ => file
                    .frames
                    .get(&name)
                    .filter(|textures| !textures.is_empty()),
            };
            let drawn = textures.is_some_and(|textures| {
                textures.iter().any(|texture| {
                    let stem = match texture.rsplit_once('.') {
                        Some((stem, _)) => stem,
                        None => texture,
                    };
                    CURSOR_TEXTURES.contains(&stem)
                        || stem.split('_').any(|part| CURSOR_TEXTURES.contains(&part))
                })
            });
            self.cursor_textures
                .borrow_mut()
                .insert((root, pane), drawn);
        }
        Ok(self.cursor_textures.borrow()[&(root, pane)])
    }

    /// Is this picture alone in a plain pane that stands in a window among its words?
    ///
    /// Sun's talk window points at the player's answers so: `arrow_00` in `Null_arrow`, beside
    /// "Yeah!" and "...I guess" in `win_00`, moved from one's row to the other's, and read as
    /// screen text the box was said whole and never the move (sun_8.ctm). No other picture in
    /// Sun's menus so far stands so; its layout files, which would name its texture, are not
    /// read.
    pub fn points_in_box(&self, root: i64, pane: i64) -> Result<bool> {
        let tree = self.layouts.tree(root);
        let holder = self.parent(pane)?;
        let r#box = self.parent(holder)?;
        Ok(class_of(&tree, pane) == Some(lyt::PICTURE_CLASS)
            && class_of(&tree, holder) == Some(lyt::PANE_CLASS)
            && self.layouts.children(root, holder) == [pane]
            && class_of(&tree, r#box) == Some(lyt::WINDOW_CLASS)
            && self
                .layouts
                .children(root, r#box)
                .iter()
                .any(|&child| class_of(&tree, child) == Some(lyt::TEXTBOX_CLASS)))
    }

    /// `u32(self.layouts.pane(pane), lyt.PARENT)`.
    pub fn parent(&self, pane: i64) -> Result<i64> {
        u32(&self.layouts.pane(pane)?, lyt::PARENT)
    }

    /// A pane's name as its layout file gives it: the bytes up to the first NUL, decoded as ASCII.
    pub fn pane_name(&self, pane: i64) -> Result<String> {
        let raw = self.layouts.name(pane)?;
        let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
        Ok(py::decode_ascii(&raw[..end]))
    }
}

/// What each cursor was over and has said, by the cursor.
///
/// Shared by every kind of cursor, as one object in Python, so its tables are in `RefCell`s.
#[derive(Default)]
pub struct Said {
    /// Cursor -> the pieces under it at the last poll, for cursors on screen.
    pub under: RefCell<IndexMap<i64, Vec<ScreenText>>>,
    /// Cursor -> the words last reported and where they were, for cursors on screen.
    pub said: RefCell<IndexMap<i64, Words>>,
    /// Cursor -> the words under it and where, at the last poll, for those said once they
    /// read the same twice.
    pub steady: RefCell<IndexMap<i64, Words>>,
}

impl Said {
    /// Report what the cursor is over, whenever the words or the place they are in changed.
    ///
    /// The place matters as much as the words. A party screen with five empty slots gives every
    /// one of them the same name, so comparing words alone said it once and then nothing however
    /// far the player arrowed, until they landed on a Pokemon and came back.
    /// A cursor that has moved to another item has news even when the item reads the same.
    pub fn report(
        &self,
        cursor: i64,
        inside: Vec<ScreenText>,
        items: &mut Vec<CursorItem>,
        r#where: Where,
        quiet: bool,
    ) {
        self.report_in(cursor, inside, items, r#where, quiet, 0);
    }

    /// `report(..., rows=rows)`: the item one of the list of rows `rows` is a pane of.
    pub fn report_in(
        &self,
        cursor: i64,
        inside: Vec<ScreenText>,
        items: &mut Vec<CursorItem>,
        r#where: Where,
        quiet: bool,
        rows: i64,
    ) {
        self.under.borrow_mut().insert(cursor, inside.clone());
        if inside.is_empty() {
            return;
        }
        let said: Words = (
            inside.iter().map(|piece| piece.text.clone()).collect(),
            r#where,
        );
        let mut held = self.said.borrow_mut();
        if held.get(&cursor) != Some(&said) {
            items.push(CursorItem {
                cursor,
                pieces: inside,
                opened: !held.contains_key(&cursor),
                quiet,
                rows,
            });
            held.insert(cursor, said);
        }
    }

    /// Report what a cursor is over once it has read the same on two polls in a row.
    ///
    /// The Pokédex slides its rows a row's height and back as it scrolls, rewriting their words
    /// in between, and for that poll the row under its plate is one the list had hidden, with
    /// whatever words it was left with. What is under meanwhile is still the cursor's.
    pub fn report_steady(
        &self,
        cursor: i64,
        inside: Vec<ScreenText>,
        items: &mut Vec<CursorItem>,
        r#where: Where,
        quiet: bool,
    ) {
        self.under.borrow_mut().insert(cursor, inside.clone());
        let seen: Words = (
            inside.iter().map(|piece| piece.text.clone()).collect(),
            r#where,
        );
        if self.steady.borrow().get(&cursor) != Some(&seen) {
            self.steady.borrow_mut().insert(cursor, seen);
            return;
        }
        self.report(cursor, inside, items, r#where, quiet);
    }

    pub fn forget(&self, cursor: i64) {
        self.under.borrow_mut().shift_remove(&cursor);
        self.said.borrow_mut().shift_remove(&cursor);
    }
}

/// The words inside a cursor's rectangle, as a frame round them has them: the layouts' with
/// words wholly inside it, of the one panel nearest its middle, in rows.
pub fn framed(pieces: &[ScreenText], rect: &Rect) -> Vec<ScreenText> {
    let under = in_front(
        pieces
            .iter()
            .filter(|piece| within(piece, rect))
            .cloned()
            .collect(),
        rect,
    );
    distinct(in_rows(nearest_panel(under, rect)))
}

pub fn centre(rect: &Rect) -> (f64, f64) {
    ((rect.left + rect.right) / 2.0, (rect.low + rect.high) / 2.0)
}

pub fn within(piece: &ScreenText, rect: &Rect) -> bool {
    if piece.glyph <= 0.0 || piece.top != rect.top {
        return false;
    }
    let across = (piece.left + piece.right) / 2.0;
    rect.left <= across && across <= rect.right && rect.low <= piece.row && piece.row <= rect.high
}

/// The words a cursor picture points at from beside them: its panel's, on its row, to its
/// right. The Poké Mart's boxes draw `cursor.bclim` left of their words rather than over them
/// (Buy, Sell, No thanks; a purchase's Yes and No), and move it from one row's height to the
/// next (spike 98). Words of another panel are not pointed at: the Mart's count of what to buy
/// draws `shop_mark_cursor.bclim` arrows above and below it, bobbing over the list behind, and
/// taken for pointers they said that list's prices, "$ 3,000" and "$ 100", every poll as they
/// bobbed (omega_ruby_shop.ctm, 2026-10-04).
pub fn beside(pieces: &[ScreenText], rect: &Rect, root: i64, panel: i64) -> Vec<ScreenText> {
    pieces
        .iter()
        .filter(|piece| {
            piece.root == root
                && piece.panel == panel
                && piece.glyph > 0.0
                && piece.top == rect.top
                && rect.low <= piece.row
                && piece.row <= rect.high
                && piece.left >= rect.right - piece.glyph
        })
        .cloned()
        .collect()
}

/// The pieces of the one panel under the cursor, when several are: the panel holding the piece
/// nearest the middle of the cursor.
///
/// A screen can keep the panel it is leaving on screen for a poll: confirming a language, the list
/// was still there behind the confirmation, and the cursor over "Begin game" said "Spanish French
/// Begin game" (opening.ctm). An item can still span rows, as a party slot's
/// name and level do, because a panel is kept whole. Of pieces on one row, the nearest across wins:
/// the Pokédex's More list put CANCEL over the page's Habitat button, the cursor centred on CANCEL
/// and Habitat at its edge, and the cursor said "Habitat" (or/movie7_menus, 2026-09-27).
pub fn nearest_panel(pieces: Vec<ScreenText>, rect: &Rect) -> Vec<ScreenText> {
    let panels: HashSet<(i64, i64)> = pieces
        .iter()
        .map(|piece| (piece.root, piece.panel))
        .collect();
    if panels.len() < 2 {
        return pieces;
    }
    let middle = (rect.low + rect.high) / 2.0;
    let across = (rect.left + rect.right) / 2.0;
    let key = |piece: &ScreenText| {
        (
            (piece.row - middle).abs(),
            ((piece.left + piece.right) / 2.0 - across).abs(),
        )
    };
    // `min(pieces, key=...)`: the first of the least, by a tuple of floats, which a NaN never
    // makes less.
    let mut nearest = &pieces[0];
    let mut best = key(nearest);
    for piece in &pieces[1..] {
        let candidate = key(piece);
        let less = if candidate.0 != best.0 {
            candidate.0 < best.0
        } else {
            candidate.1 < best.1
        };
        if less {
            nearest = piece;
            best = candidate;
        }
    }
    let wanted = (nearest.root, nearest.panel);
    pieces
        .iter()
        .filter(|piece| (piece.root, piece.panel) == wanted)
        .cloned()
        .collect()
}

/// The pieces of the layouts with words wholly under the cursor, when any layout has some.
///
/// A layout drawn behind the menu can have words whose middle is under the cursor and whose ends
/// reach out past it: with the Pokédex in the X menu, OPTIONS sits over AreaNav's "Littleroot
/// Town", which starts well to the left of the button, and the item was said as "Littleroot Town
/// OPTIONS" (battles2.ctm, spike 68). A piece's ends are estimated, so half a glyph either way
/// still counts as under, and when no layout's words are wholly under the cursor, all are kept.
pub fn in_front(pieces: Vec<ScreenText>, rect: &Rect) -> Vec<ScreenText> {
    let whole: HashSet<i64> = pieces
        .iter()
        .filter(|piece| {
            rect.left - piece.glyph / 2.0 <= piece.left
                && piece.right <= rect.right + piece.glyph / 2.0
        })
        .map(|piece| piece.root)
        .collect();
    if whole.is_empty() {
        pieces
    } else {
        pieces
            .into_iter()
            .filter(|piece| whole.contains(&piece.root))
            .collect()
    }
}

pub fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= SAME_SPOT
}

/// Is this place inside the rectangle?
pub fn on(place: (f64, f64), rect: &Rect) -> bool {
    let (x, y) = place;
    rect.left <= x && x <= rect.right && rect.low <= y && y <= rect.high
}

/// A window's place and size to the unit, to say whether it has moved or been resized: the
/// main menu's cursor is 336 by 140 on CONTINUE's panel and 336 by 43 on the rows below it.
pub fn spot_of(rect: &Rect) -> Result<(i64, i64, i64, i64)> {
    Ok((
        py::round(rect.left)?,
        py::round(rect.low)?,
        py::round(rect.right - rect.left)?,
        py::round(rect.high - rect.low)?,
    ))
}

/// `(round(rect.left), round(rect.low))`: where a window is, to the unit. Written out at each of
/// its places in the Python, so a NaN is raised from the line that asked, as it is there.
#[track_caller]
pub fn place_of(rect: &Rect) -> Result<Where> {
    Ok(Where::Place(py::round(rect.left)?, py::round(rect.low)?))
}

/// The pane's class, or None for a pane not in the tree: `tree.get(pane)`.
pub fn class_of(tree: &IndexMap<i64, String>, pane: i64) -> Option<&str> {
    tree.get(&pane).map(String::as_str)
}
