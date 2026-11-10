//! The Poké Mart draws a seventh (spike 95): two pictures textured `shop_cursor.bclim` in a pane of
//! their own, moved onto the chosen row. A picture whose texture the designer named a cursor is one
//! from the poll it is seen on, and the item is the words inside the pane holding it. Its boxes of
//! choices point one, `cursor.bclim`, at their words from beside them (spike 98): Buy, Sell and No
//! thanks, and a purchase's Yes and No, in a window alone under their root. Over no words, such a
//! picture says the words on its row to its right; and a window holding panes of its own is a box,
//! not the corners of an app menu's cursor. X's Mart draws `shop_cursor.bclim` as a window beside
//! its rows' words, and it walks them as the pictures do.
//!
//! The PC Box draws one with `cursol01.bclim`, as its designers spelt it there (spike 127): a picture
//! in a plain pane the game moves onto the chosen row of its menu for a chosen Pokémon, where each
//! row is a picture of its own. A named picture's list is the row it is over and the rows drawn like
//! it; over rows drawn apart like these it glides through each on its way, so its item is held until
//! it has read the same on two polls, as a plate's is. Once it has been over words it no longer
//! points at the words beside it: the Box's leaves for the grid of Pokémon a poll before its menu's
//! rows go. Over no words at all it may rest on a slot, a touch area of the Box's grid, its party or
//! a Pokémon's marks, said by where it is (`slots`, spike 129); each group of slots is reported as a
//! cursor of its own, so that the summary its moves change is not taken for the menu's details.
//!
//! The main menu's frame is a named cursor too (spike 96): a window drawn with `cursor_lt.bclim`, so
//! known from the poll it is seen on, where a frame directly under a root is otherwise known only
//! once it has moved.
//!
//! The plate and a named picture know the list they walk: the plate's rows, and the words beside
//! those the named picture is over. What else their layout shows — the Mart's money and the chosen
//! item's description — is the screen's own, not the menu's (`lists`). A plate says an item only
//! once it has read the same on two polls, because a list scrolling under it slides its rows a row's
//! height and back while their words are rewritten, and for a poll the row under the plate holds a
//! hidden row's stale words.

use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::memory::Memory;
use crate::games::common::cursors::menu::MenuCursors;
use crate::games::common::cursors::plate::Plate;
use crate::games::common::cursors::shared::{
    CursorItem, Panes, Rect, Said, Where, beside, centre, class_of, framed, near, place_of,
};
use crate::games::common::layout::Layouts;
use crate::games::common::screen::{
    SCREEN_HALF_HEIGHT, SCREEN_HALF_WIDTH, ScreenText, distinct, in_rows,
};
use crate::games::common::{lyt, slots};
use crate::py::{PySet, Result};

/// Ticks a named cursor must have stood where a screen poll finds it to be at rest there: a glide
/// moves it every frame, so it never stands on a slot or a row it passes for two (spike 129).
pub const REST_TICKS: i64 = 3;

/// The cursors that know the list they walk: plates, and pictures and windows drawn with a
/// texture their designer named a cursor.
///
/// Python's keeps the menu cursors (`menus`), which are cursors of their own kind; here they are
/// handed to `poll`, the reader holding both.
pub struct ListCursors {
    panes: Rc<Panes>,
    layouts: Rc<Layouts>,
    said: Rc<Said>,
    /// Plate or named cursor on screen -> (its layout's root, the TextBoxes of the list it
    /// walks).
    listing: IndexMap<i64, (i64, HashSet<i64>)>,
    /// Plate or named cursor -> polls in a row it has not been found.
    listing_missed: IndexMap<i64, i64>,
    /// Named cursors that have been over words, and so never point at words beside them.
    over: HashSet<i64>,
    /// Layouts a named cursor has been found in, looked in for it while their words are all
    /// off screen: the Box's box list hides the Box's every word, its pointer walks the list and
    /// the bottom bar, and on "change box" it said nothing (movie12, spike 143).
    named_roots: PySet,
    /// Cursors on their way, at this poll, to an item not said yet (`moving`).
    moving: HashSet<i64>,
    /// Named cursors on screen that walk rows or slots drawn apart, whose places each tick
    /// samples (`sample`): the Poké Mart's and the main menu's never glide, and are not.
    sampled: PySet,
    /// Named cursor -> where it stood at the last few ticks, oldest first.
    samples: IndexMap<i64, VecDeque<(f64, f64)>>,
    /// Named cursor -> the groups of slots it has been on, each reported as a cursor of its
    /// own, so that what the grid's moves change is not taken for the details of the menu's.
    slot_groups: IndexMap<i64, HashSet<i64>>,
}

impl ListCursors {
    pub fn new(panes: Rc<Panes>, said: Rc<Said>) -> Self {
        ListCursors {
            layouts: panes.layouts.clone(),
            panes,
            said,
            listing: IndexMap::new(),
            listing_missed: IndexMap::new(),
            over: HashSet::new(),
            named_roots: PySet::new(),
            moving: HashSet::new(),
            sampled: PySet::new(),
            samples: IndexMap::new(),
            slot_groups: IndexMap::new(),
        }
    }

    /// Is there a named cursor on screen whose place each tick should sample?
    pub fn sampling(&self) -> bool {
        !self.sampled.is_empty()
    }

    /// Note where each named cursor stands now. Called every tick, so that a screen poll can
    /// tell a cursor at rest from one gliding past a slot or a row: the Box's pointer glides
    /// through each on its way, and holding what it was over for two screen polls instead left
    /// out a row or a slot moved past within half a second, and read a slot's summary before the
    /// slot (or/movie11_pokedex_and_box, spike 129).
    pub fn sample(&mut self, memory: &dyn Memory) -> Result<()> {
        for cursor in self.sampled.iter().collect::<Vec<_>>() {
            let head = memory.read(cursor + lyt::X, lyt::Y - lyt::X + 4)?;
            let place = (f32(&head, 0)?, f32(&head, lyt::Y - lyt::X)?);
            let samples = self.samples.entry(cursor).or_default();
            // `deque(maxlen=REST_TICKS)` drops its oldest when full.
            if samples.len() as i64 >= REST_TICKS {
                samples.pop_front();
            }
            samples.push_back(place);
        }
        Ok(())
    }

    /// Has this cursor stood where it is for the last ticks sampled? One not sampled yet, as
    /// the moment it comes on screen, has nothing to say it is moving.
    fn at_rest(&self, cursor: i64, place: (f64, f64)) -> bool {
        self.samples.get(&cursor).is_none_or(|samples| {
            samples
                .iter()
                .all(|&(x, y)| near(x, place.0) && near(y, place.1))
        })
    }

    /// The cursors on their way to an item they have not said yet, as of the last poll, with
    /// every group of slots theirs: the details gathered for where one was are no longer its
    /// (spike 129).
    pub fn moving(&self) -> HashSet<i64> {
        let mut out = self.moving.clone();
        for cursor in &self.moving {
            if let Some(groups) = self.slot_groups.get(cursor) {
                out.extend(groups);
            }
        }
        out
    }

    /// Layout root -> the TextBoxes of the list a cursor walks, as of the last poll, for the
    /// cursors that know theirs: a plate's rows, and the words beside those a named cursor is
    /// over. The rest of such a layout is not the menu's.
    pub fn lists(&self) -> IndexMap<i64, HashSet<i64>> {
        let mut out: IndexMap<i64, HashSet<i64>> = IndexMap::new();
        for (root, boxes) in self.listing.values() {
            // A cursor over no words, and pointing at none beside it, knows no list.
            if !boxes.is_empty() {
                let mut joined = out.get(root).cloned().unwrap_or_default();
                joined.extend(boxes);
                out.insert(*root, joined);
            }
        }
        out
    }

    /// Report what each plate (`plate.plates`) and each named cursor shown is over. Pictures
    /// `taken` by another kind of cursor are left to it, and the layouts of `keyboards` to their
    /// key cursors.
    #[allow(clippy::too_many_arguments)]
    pub fn poll(
        &mut self,
        on_screen: &[ScreenText],
        items: &mut Vec<CursorItem>,
        plates: IndexMap<i64, Plate>,
        taken: &HashSet<i64>,
        keyboards: &PySet,
        menus: &MenuCursors,
    ) -> Result<()> {
        self.moving = HashSet::new();
        // A picture another kind already reads is that kind's: the name entry's key cursor is
        // textured as a cursor too, and read twice it was said again every poll (opening.ctm).
        // So is a keyboard's whole layout: its second cursor picture said "“" as it opened.
        let named = self.named(on_screen, taken, keyboards, menus)?;
        let named_keys: Vec<i64> = named.keys().copied().collect();
        self.sampled
            .intersection_update(&PySet::of_dict(&named_keys));
        self.samples.retain(|cursor, _| named.contains_key(cursor));
        // Lost for one poll is not gone: rows sliding past can leave none under the plate.
        let lost: Vec<i64> = self
            .listing
            .keys()
            .copied()
            .filter(|cursor| !plates.contains_key(cursor) && !named.contains_key(cursor))
            .collect();
        for cursor in lost {
            let missed = self.listing_missed.get(&cursor).copied().unwrap_or(0) + 1;
            self.listing_missed.insert(cursor, missed);
            if missed > 1 {
                self.listing.shift_remove(&cursor);
                self.listing_missed.shift_remove(&cursor);
                self.said.steady.borrow_mut().shift_remove(&cursor);
                self.over.remove(&cursor);
                for group in self.slot_groups.shift_remove(&cursor).unwrap_or_default() {
                    self.said.steady.borrow_mut().shift_remove(&group);
                    self.said.forget(group);
                }
                self.said.forget(cursor);
            }
        }
        for cursor in plates.keys().chain(named.keys()) {
            self.listing_missed.shift_remove(cursor);
        }
        for (&window, (root, row, rows)) in &plates {
            let pieces: Vec<&ScreenText> = on_screen
                .iter()
                .filter(|piece| piece.root == *root)
                .collect();
            let mut boxes: HashSet<i64> = HashSet::new();
            for piece in &pieces {
                let mut any = false;
                for &r in rows {
                    if self.panes.below(piece.r#box, r)? {
                        any = true;
                        break;
                    }
                }
                if any {
                    boxes.insert(piece.r#box);
                }
            }
            self.listing.insert(window, (*root, boxes));
            let mut chosen: Vec<ScreenText> = Vec::new();
            for piece in &pieces {
                if self.panes.below(piece.r#box, *row)? {
                    chosen.push((*piece).clone());
                }
            }
            let item = distinct(in_rows(chosen));
            self.said
                .report_steady(window, item, items, Where::Pane(*row), false);
        }
        let shown_roots: HashSet<i64> = on_screen.iter().map(|piece| piece.root).collect();
        for (&holder, &(root, walks)) in &named {
            let Some(rect) = self.named_rect(holder, root)? else {
                continue;
            };
            let mut own: Vec<ScreenText> = Vec::new();
            for piece in on_screen {
                if piece.root == root && self.panes.below(piece.r#box, holder)? {
                    own.push(piece.clone());
                }
            }
            // The Bag's select bar carries the chosen Pokémon's name, level and ABLE! over the
            // slot's own ABLE!, which lies nearer its middle (spike 139).
            let mut inside: Vec<ScreenText> = if !own.is_empty() {
                distinct(in_rows(own.clone()))
            } else {
                framed(on_screen, &rect)
            };
            // Asked even with words framed: a contest's small cursor stands on its button's empty
            // pane with the words' middle under it, and taken as framed alone its two columns of
            // moves were one, the other column read as new (or_toughness_contest).
            let button = if walks && own.is_empty() {
                self.on_empty_pane(holder, root, on_screen)?
            } else {
                Vec::new()
            };
            if inside.is_empty() {
                inside = button.clone();
            }
            if !inside.is_empty() {
                self.over.insert(holder);
            } else if walks && !self.over.contains(&holder) {
                // The Mart's boxes point at their words from beside them. The PC Box's cursor is
                // over its menu's words, and leaves for the grid of Pokémon a poll before they go,
                // where the word on its row to its right was said (or/movie6_pc, 2026-09-27).
                let placement = self.layouts.placement(holder)?;
                let panel = placement.map_or(0, |placement| placement.panel);
                inside = distinct(in_rows(beside(on_screen, &rect, root, panel)));
            }
            let (mut at_slot, mut alone, mut reporter) = (false, false, holder);
            if inside.is_empty()
                && walks
                && let Some(files) = self.panes.files.clone()
            {
                // The PC Box's pointer over its grid of Pokémon and its party, where a slot holds
                // an icon and no words: said by where it is (spike 129).
                let file = files.matching(&self.layouts.pane_names(root));
                let (x, y) = ((rect.left + rect.right) / 2.0, (rect.low + rect.high) / 2.0);
                // With its layout's words all hidden, as under the Box's box list, a cursor looks
                // only for another layout's button: the list's folders read as the Box's slots,
                // "row 2, column 1, empty", and left the box's name above them unsaid (movie12).
                let mut found = if !shown_roots.contains(&root) {
                    None
                } else {
                    slots::slot(
                        &self.layouts,
                        file.as_deref(),
                        root,
                        x,
                        y,
                        on_screen,
                        self.panes.occupant.as_ref(),
                    )?
                };
                if found.as_ref().is_none_or(|found| found.1.is_empty()) {
                    found = slots::button(&self.layouts, &files, root, x, y, rect.top)?;
                }
                if let Some((touch, said, was_alone)) = found {
                    alone = was_alone;
                    inside = vec![ScreenText {
                        top: rect.top,
                        root,
                        parent: touch,
                        ..ScreenText::new(touch, said, false)
                    }];
                    at_slot = true;
                    reporter = u32(&self.layouts.pane(touch)?, lyt::PARENT)?;
                    self.slot_groups.entry(holder).or_default();
                    // The same pointer going on from the party to the box has moved, and brings
                    // the summary of what it moved to, where an item opening brings none.
                    let mut held = self.said.said.borrow_mut();
                    if !held.contains_key(&reporter)
                        && (held.contains_key(&holder)
                            || self.slot_groups[&holder]
                                .iter()
                                .any(|group| held.contains_key(group)))
                    {
                        held.insert(reporter, (Vec::new(), Where::Nothing));
                    }
                    self.slot_groups.entry(holder).or_default().insert(reporter);
                }
            }
            // Where the pointer went on to another group, what each other one last said is
            // forgotten, so that coming back to the slot it left says it again.
            let mut others: HashSet<i64> = HashSet::from([holder]);
            if let Some(groups) = self.slot_groups.get(&holder) {
                others.extend(groups);
            }
            others.remove(&reporter);
            for other in others {
                let mut held = self.said.said.borrow_mut();
                if held.contains_key(&other) {
                    held.insert(other, (Vec::new(), Where::Nothing));
                }
                self.said.steady.borrow_mut().shift_remove(&other);
            }
            // A button's cursor walks a grid of buttons alike, the contest's two columns of moves.
            // A cursor carrying its own words walks no list of its layout: taken for one, the
            // Bag's select bar left the item list to be read whole as the Bag came back.
            let mut rows: HashSet<(i64, i64)> = HashSet::new();
            if walks && !at_slot && own.is_empty() {
                for piece in &inside {
                    for row in self
                        .panes
                        .alike(piece.root, piece.parent, button.is_empty())?
                        .iter()
                    {
                        rows.insert((piece.root, row));
                    }
                }
            }
            self.listing.insert(
                holder,
                (
                    root,
                    on_screen
                        .iter()
                        .filter(|piece| rows.contains(&(piece.root, piece.parent)))
                        .map(|piece| piece.r#box)
                        .collect(),
                ),
            );
            let at = place_of(&rect)?;
            if rows.len() > 1 || at_slot {
                self.sampled.add(holder);
                // The PC Box's cursor glides from row to row and slot to slot over several frames,
                // through each one's own place, and a poll on the way said the rows it was leaving
                // and passing (or/movie6_pc, 2026-09-27): what it is over is said once it is at
                // rest, as the ticks between polls say (`sample`).
                if !self.at_rest(holder, centre(&rect)) {
                    self.moving.insert(holder);
                    continue;
                }
                let held = self.said.said.borrow().get(&reporter).map(|said| said.1);
                if held == Some(at) {
                    // Words rewritten under a cursor that has not moved: a Secret Base's list
                    // writes the next list's words into its rows a poll before it hides the last
                    // one's, and the category and the item were said together (spike 154).
                    self.said.report_steady(reporter, inside, items, at, alone);
                } else {
                    self.said.report(reporter, inside, items, at, alone);
                }
            } else if !own.is_empty() {
                // The select bar's name is written a poll before its level, and moving from Ralts
                // said "Wingull L14 ABLE!" with Ralts's level (spike 139).
                self.said.report_steady(holder, inside, items, at, false);
            } else {
                // Not held: the Mart's cursor jumps whole from row to row, and holding it said the
                // new item's count and description before the item.
                self.said.report(holder, inside, items, at, false);
            }
        }
        Ok(())
    }

    /// The words of the button whose empty pane a cursor picture sits on, as the battle's
    /// does (`PictureCursors.item`): a contest draws `cursor.bclim` over the chosen move, on a
    /// 20 by 20 pane at the top of the move's button, its words below it, and said nothing
    /// (movie30, spike 196).
    fn on_empty_pane(
        &self,
        holder: i64,
        root: i64,
        on_screen: &[ScreenText],
    ) -> Result<Vec<ScreenText>> {
        let (x, y) = self.panes.place(holder)?;
        let tree = self.layouts.tree(root);
        // A Secret Base's list moves a plain pane holding its cursor onto the row, where the
        // list's own empty panes stand too (spike 154): only a picture marks a button so.
        if class_of(&tree, holder) != Some(lyt::PICTURE_CLASS) {
            return Ok(Vec::new());
        }
        for (&pane, name) in &tree {
            if name != lyt::PANE_CLASS
                || pane == holder
                || pane == root
                || !self.layouts.children(root, pane).is_empty()
                || !(near(self.panes.place(pane)?.0, x) && near(self.panes.place(pane)?.1, y))
            {
                continue;
            }
            let group = u32(&self.layouts.pane(pane)?, lyt::PARENT)?;
            if group == root {
                continue;
            }
            let mut words: Vec<ScreenText> = Vec::new();
            for piece in on_screen {
                if piece.root == root && self.panes.below(piece.parent, group)? {
                    words.push(piece.clone());
                }
            }
            if !words.is_empty() {
                return Ok(distinct(in_rows(words)));
            }
            // A button of no words: a contest's portraits, whose entrant the screen names
            // elsewhere as the cursor lands, "Pokémon: Bolt, Trainer: Camden" (movie33). Its
            // item is nameless, and said by its details.
            let placement = self.layouts.placement(holder)?;
            let top = placement.is_some_and(|placement| placement.top);
            return Ok(vec![ScreenText {
                top,
                root,
                parent: group,
                ..ScreenText::new(group, "", false)
            }]);
        }
        Ok(Vec::new())
    }

    /// Pane -> (root, whether it walks a list), for every pane shown that is drawn as a
    /// cursor, by the name its designer gave the texture.
    ///
    /// A picture marks a row among the words beside it, which are its list: the picture, or the
    /// plain pane holding it when that holds nothing else, as the Poké Mart keeps the two halves
    /// of `shop_cursor.bclim` (spike 95). A window frames its item as an app menu's cursor does,
    /// and its layout is the menu's whole, as the main menu's `cursor_lt.bclim` frame's is: known
    /// from the poll it is seen on, where a frame is otherwise known only once it has moved
    /// (spike 96). Pictures `taken` by another kind of cursor are left to it, and the layouts of
    /// `keyboards` to their key cursors.
    fn named(
        &mut self,
        on_screen: &[ScreenText],
        taken: &HashSet<i64>,
        keyboards: &PySet,
        menus: &MenuCursors,
    ) -> Result<IndexMap<i64, (i64, bool)>> {
        let mut found: IndexMap<i64, (i64, bool)> = IndexMap::new();
        if self.panes.files.is_none() && !self.panes.cursor_panes {
            return Ok(found);
        }
        self.named_roots
            .intersection_update(&self.layouts.roots_set());
        let mut shown = PySet::new();
        for piece in on_screen {
            shown.add(piece.root);
        }
        let walked = shown.union(&self.named_roots).difference(keyboards);
        for root in walked.iter().collect::<Vec<_>>() {
            let file = self.panes.file(root);
            if file.is_none() && !self.panes.cursor_panes {
                continue;
            }
            let tree = self.layouts.tree(root);
            for (&pane, name) in &tree {
                if name != lyt::PICTURE_CLASS && name != lyt::WINDOW_CLASS {
                    continue;
                }
                if taken.contains(&pane) || menus.windows.contains(&pane) {
                    continue;
                }
                if !self.panes.drawn_as_cursor(root, pane, file.as_deref())? {
                    continue;
                }
                let mut holder = pane;
                let parent = self.panes.parent(pane)?;
                if name == lyt::PICTURE_CLASS
                    && parent != root
                    && class_of(&tree, parent) == Some(lyt::PANE_CLASS)
                {
                    let mut all = true;
                    for child in self.layouts.children(root, parent) {
                        if !self.panes.drawn_as_cursor(root, child, file.as_deref())? {
                            all = false;
                            break;
                        }
                    }
                    if all {
                        holder = parent;
                        let above = self.panes.parent(holder)?;
                        if above != root
                            && class_of(&tree, above) == Some(lyt::PANE_CLASS)
                            && self.layouts.children(root, above) == [holder]
                        {
                            // A Secret Base's lists keep their cursor's pane, 30 by 40, alone in a
                            // pane the game moves onto the chosen row at the row's own size, and the
                            // row's words reach out past the smaller one (spike 154).
                            holder = above;
                        }
                    }
                }
                let placement = self.layouts.placement(holder)?;
                if placement.is_some_and(|placement| placement.shown)
                    && !self.parked(holder, root)?
                {
                    let walks =
                        name == lyt::PICTURE_CLASS || self.among_rows(root, holder, &tree)?;
                    found.insert(holder, (root, walks));
                    self.named_roots.add(root);
                }
            }
        }
        Ok(found)
    }

    /// Is this window drawn among the words it frames, beside them in the pane holding them?
    /// X's Poké Mart draws `shop_cursor.bclim` as a window in its list's window, beside every
    /// row's words, where ORAS's is a pane holding two pictures, and taken for a frame it made a
    /// menu of the whole layout: the description and the count to buy went unsaid
    /// (x_poke-mart). It walks its list as a picture does. The main menu's
    /// frame hangs from its root, its rows in panes of their own.
    fn among_rows(&self, root: i64, window: i64, tree: &IndexMap<i64, String>) -> Result<bool> {
        let parent = u32(&self.layouts.pane(window)?, lyt::PARENT)?;
        Ok(parent != root
            && self
                .layouts
                .children(root, parent)
                .into_iter()
                .any(|child| class_of(tree, child) == Some(lyt::TEXTBOX_CLASS)))
    }

    /// Where a named cursor is drawn: its holder's place, a row of its pictures tall where a
    /// plain pane holds pictures shorter than itself. A Secret Base's YES and NO keep their
    /// cursor's two halves, 20 tall, in a pane of 40, two rows, which said "YES NO" (spike 154).
    fn named_rect(&self, holder: i64, root: i64) -> Result<Option<Rect>> {
        let rect = self.panes.rect(holder)?;
        let tree = self.layouts.tree(root);
        let Some(rect) = rect else {
            return Ok(None);
        };
        if class_of(&tree, holder) != Some(lyt::PANE_CLASS) {
            return Ok(Some(rect));
        }
        let children = self.layouts.children(root, holder);
        if children.is_empty()
            || children
                .iter()
                .any(|&child| class_of(&tree, child) != Some(lyt::PICTURE_CLASS))
        {
            return Ok(Some(rect));
        }
        let mut drawn: Vec<Rect> = Vec::new();
        for &child in &children {
            if let Some(found) = self.panes.rect(child)? {
                drawn.push(found);
            }
        }
        // `max(..., default=0.0)`: the first of the greatest, so a NaN first stays.
        let mut tall = 0.0;
        for (index, found) in drawn.iter().enumerate() {
            let height = found.high - found.low;
            if index == 0 || height > tall {
                tall = height;
            }
        }
        let middle = (rect.low + rect.high) / 2.0;
        if !(0.0 < tall && tall < rect.high - rect.low) {
            return Ok(Some(rect));
        }
        Ok(Some(Rect {
            top: rect.top,
            left: rect.left,
            right: rect.right,
            low: middle - tall / 2.0,
            high: middle + tall / 2.0,
        }))
    }

    /// Is the pane drawn wholly off its screen, judged from its layout's root? The Bag keeps
    /// its select bar at x -320 while the player walks its items, left of the bottom screen, and
    /// taken for a cursor there it said the words nearest it (spike 139).
    fn parked(&self, pane: i64, root: i64) -> Result<bool> {
        let Some(rect) = self.panes.rect(pane)? else {
            return Ok(false);
        };
        let (x, y) = self.panes.place(root)?;
        let half_width = SCREEN_HALF_WIDTH[usize::from(rect.top)];
        Ok(rect.right - x < -half_width
            || rect.left - x > half_width
            || rect.high - y < -SCREEN_HALF_HEIGHT
            || rect.low - y > SCREEN_HALF_HEIGHT)
    }
}
