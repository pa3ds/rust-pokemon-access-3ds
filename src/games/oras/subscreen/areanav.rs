//! The AreaNav's views, which say which of its layouts it draws (spikes 181 and 183), and the
//! lists drawn over its map.
//!
//! The AreaNav keeps the layouts of a screen it has put away exactly as they were: the berry list
//! after its View Details, the Pokémon Distribution, the trainers' details. Its layout work, its
//! panes' flags and alpha, Game Freak's layout helper (`app::util::G2DUtil`) and its button manager
//! all read as drawn (spikes 174, 179 and 180), and a tap on a berry row moved the map's cursor
//! (movie20).
//! The game stops walking the layout altogether (spike 181, a read watchpoint on its root): each
//! screen is a view of the AreaNav's module, `app::townmap::NutsView` for the berry list, whose
//! update and draw return at once while a byte of its own is 0, and which otherwise draws the
//! layouts of the `G2DUtil` at its +0x5C (spike 183, the byte 1 while the list is up and 0 once it
//! is gone). Which byte is each class's own: the offsets in `VIEWS` are where each class's draw
//! reads it. `MapView` and `GraphicWork` draw whenever the AreaNav does, and are not listed. The
//! `G2DUtil`'s layouts are read as the Move Reminder's are (`townmap.util_works`).
//!
//! X's Town Map is the same app, `app::townmap`, and has none of these views, none of their
//! classes' names in the memory x_townmap's tape read: the place under the map's cursor is gen
//! 6's (`townmap.Places`), and what a list drawn over ORAS's map makes of the cursors is said here
//! (`subscreen.PokeNav`).
//!
//! Bound by:
//! - ADR-0043: a list over the map reads its rows without the area said, a berry with its time; a
//!   view not drawn is put away, its cursor's layout too.

use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::screens::scrolled::{Row, Scrolled, row};
use crate::games::gen6::screens::townmap::{UTIL, UTIL_CLASS, util_works};
use crate::py::Result;

/// Each view class whose draw is skipped while a byte of it is 0, by where that byte is.
pub const VIEWS: [(&str, i64); 5] = [
    ("N3app7townmap8NutsViewE", 0xAC),
    ("N3app7townmap14TrainerEyeViewE", 0xAC),
    ("N3app7townmap14SecretBaseViewE", 0xB4),
    ("N3app7townmap9ZukanViewE", 0x7C),
    ("N3app7townmap11MessageViewE", 0x7C),
];
/// The views drawn as lists whose rows name their places, the map following the row under their
/// cursor: Berry Trees, Trainer's Eye and Secret Bases (or_areanav_by_touch). A `frozenset` only
/// tested.
pub const LISTS: [&str; 3] = [
    "N3app7townmap8NutsViewE",
    "N3app7townmap14TrainerEyeViewE",
    "N3app7townmap14SecretBaseViewE",
];
pub const TRAINERS: &str = "N3app7townmap14TrainerEyeViewE";
pub const BERRIES: &str = "N3app7townmap8NutsViewE";
/// A list view -> its list's cursor, whose +0x58 is the root of the cursor's layout: Berry
/// Trees', Trainer's Eye's and Secret Bases' each (a probe on or_areanav_trainers_bug). The view's
/// `G2DUtil` does not hold it, and put away with the rest of Berry Trees, its cursor stayed drawn
/// where it last stood, over Trainer's Eye opened after it, reading each trainer a move late: the
/// area's switch said the last area's trainer, "Edwin" as Mauville City opened (a player's report,
/// or_areanav_trainers_bug).
pub const CURSOR: i64 = 0x94;
pub const CURSOR_ROOT: i64 = 0x58;

/// The AreaNav's views a sweep found, and the layouts of those it has put away.
pub struct Views {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    /// View -> the vtable it was found with, where its byte saying it is drawn is, and its
    /// class.
    views: IndexMap<i64, (i64, i64, &'static str)>,
    /// A `Cell`, as `put_away` sets it through `&self`.
    list: Cell<Option<&'static str>>,
}

impl Views {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        Views {
            memory,
            names,
            views: IndexMap::new(),
            list: Cell::new(None),
        }
    }

    /// The views a sweep found, by class: all of them, or with `adding` the new ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut views = if !adding {
            IndexMap::new()
        } else {
            self.views.clone()
        };
        for (&view, name) in found {
            if let Some(&(class, drawn_at)) = VIEWS.iter().find(|(class, _)| class == name) {
                views.insert(
                    view,
                    (u32(&self.memory.read(view, 4)?, 0)?, drawn_at, class),
                );
            }
        }
        self.views = views;
        Ok(())
    }

    /// Whether the AreaNav draws one of these views over its map now: the Pokémon
    /// Distribution, Trainer's Eye's rows, the berry list. Then a tap on the map reaches the view's
    /// own buttons, not a place (or_area_nav).
    pub fn drawing(&self) -> Result<bool> {
        Ok(self
            .read()?
            .iter()
            .any(|(drawn, _util, _name, _head)| *drawn))
    }

    /// Whether the AreaNav drew one of its lists over its map at the last `put_away`
    /// (`LISTS`): read again, its reads were new to the screen poll's tapes.
    pub fn listing(&self) -> bool {
        self.list.get().is_some()
    }

    /// The class of the list the AreaNav drew over its map at the last `put_away`, or None.
    pub fn list_drawn(&self) -> Option<&'static str> {
        self.list.get()
    }

    /// The works of the layouts of every view the AreaNav is not drawing now, its list's
    /// cursor's among them (`CURSOR`), by `works_of` a layout's root -> its work.
    pub fn put_away(&self, works_of: &IndexMap<i64, i64>) -> Result<HashSet<i64>> {
        let mut works: HashSet<i64> = HashSet::new();
        let read = self.read()?;
        self.list.set(
            read.iter()
                .find(|(drawn, _util, name, _head)| *drawn && LISTS.contains(name))
                .map(|&(_, _, name, _)| name),
        );
        for (drawn, util, name, head) in read {
            if drawn {
                continue;
            }
            works.extend(util_works(&*self.memory, util)?);
            if LISTS.contains(&name) {
                let cursor = u32(&head, CURSOR)?;
                if cursor != 0 {
                    let root = u32(&self.memory.read(cursor + CURSOR_ROOT, 4)?, 0)?;
                    if let Some(&work) = works_of.get(&root) {
                        works.insert(work);
                    }
                }
            }
        }
        Ok(works)
    }

    /// Each view still one, whether it is drawn, its `G2DUtil`, its class and the head of it
    /// read.
    #[allow(clippy::type_complexity)]
    fn read(&self) -> Result<Vec<(bool, i64, &'static str, Vec<u8>)>> {
        let memory = &self.memory;
        let mut read: Vec<(bool, i64, &'static str, Vec<u8>)> = Vec::new();
        for (&view, &(vtable, drawn_at, name)) in &self.views {
            let head = memory.read(view, drawn_at.max(UTIL).max(CURSOR) + 4)?;
            // A view freed and its memory taken by something else says nothing.
            if u32(&head, 0)? != vtable {
                continue;
            }
            let util = u32(&head, UTIL)?;
            if util != 0
                && self
                    .names
                    .of_vtable(u32(&memory.read(util, 4)?, 0)?)?
                    .as_deref()
                    == Some(UTIL_CLASS)
            {
                read.push((head[drawn_at as usize] != 0, util, name, head));
            }
        }
        Ok(read)
    }
}

/// A list over the map: its moves along rows reading alike (`scrolled`), its rows one list,
/// and Berry Trees' time since planting said with each berry.
///
/// Each row was its own list of rows, the panels its details are said from kept apart
/// (`CursorItem.rows`); a list is one. Berry Trees writes a berry's time on the top screen, the
/// poll its row is under the cursor or the one after, and the same words for berries planted
/// the same minute, which no change of the screen's said: a berry is said a poll late, with
/// the time standing then, "Lum Berry, It has been 25 days…", and the time on its own never,
/// as the minutes tick over too. The first of an area is queued behind its name: a poll late,
/// it came as a move 0.3 s after "Route 104", cutting it off (or_areanav_berries).
#[derive(Default)]
pub struct ListRows {
    /// The layouts of the list's rows, those naming the area.
    lists: HashSet<i64>,
    scrolled: Scrolled,
    /// A berry list -> the row under its cursor at the last screen poll.
    seen: IndexMap<i64, Row>,
    /// A berry list -> the row last said.
    said: IndexMap<i64, Row>,
    /// A berry list -> its cursor.
    cursors: IndexMap<i64, i64>,
    /// A berry list -> the area its last row said was in.
    areas: IndexMap<i64, String>,
}

impl ListRows {
    pub fn new() -> Self {
        ListRows::default()
    }

    /// The cursors' poll with the list `drawn` over the map read so (`Views.list_drawn`).
    pub fn poll(
        &mut self,
        cursors: CursorPoll,
        drawn: Option<&str>,
        area: Option<&str>,
        on_screen: &[ScreenText],
    ) -> CursorPoll {
        let (Some(drawn), Some(area)) = (drawn, area) else {
            self.lists.clear();
            self.seen.clear();
            self.said.clear();
            self.cursors.clear();
            self.areas.clear();
            return self.scrolled.poll(cursors, &HashSet::new());
        };
        let mut under: IndexMap<i64, &Vec<ScreenText>> = IndexMap::new();
        for pieces in &cursors.under {
            if let Some(first) = pieces.first() {
                under.insert(first.root, pieces);
            }
        }
        for (&root, pieces) in &under {
            if pieces.iter().any(|p| p.text == area) {
                self.lists.insert(root);
            }
        }
        if drawn != BERRIES {
            let cursors = self.scrolled.poll(cursors, &self.lists);
            let items = cursors
                .items
                .iter()
                .map(|item| match item.pieces.first() {
                    Some(first) if self.lists.contains(&first.root) => CursorItem {
                        rows: first.root,
                        ..item.clone()
                    },
                    _ => item.clone(),
                })
                .collect();
            return CursorPoll { items, ..cursors };
        }
        let mut items: Vec<CursorItem> = Vec::new();
        for item in &cursors.items {
            match item.pieces.first() {
                Some(first) if self.lists.contains(&first.root) => {
                    self.cursors.insert(first.root, item.cursor);
                }
                _ => items.push(item.clone()),
            }
        }
        let mut chased = cursors.chased;
        for (&root, pieces) in &under {
            if !self.lists.contains(&root) || !self.cursors.contains_key(&root) {
                continue;
            }
            let now = row(pieces);
            let seen = self.seen.insert(root, now.clone());
            if Some(&now) == self.said.get(&root) {
                continue;
            }
            if Some(&now) != seen.as_ref() {
                chased = true;
                continue;
            }
            self.said.insert(root, now);
            let words: Vec<ScreenText> = on_screen.iter().filter(|p| p.top).cloned().collect();
            items.push(CursorItem {
                rows: root,
                ..CursorItem::new(
                    self.cursors[&root],
                    [pieces.to_vec(), words].concat(),
                    self.areas.get(&root).map(String::as_str) != Some(area),
                )
            });
            self.areas.insert(root, area.to_string());
        }
        CursorPoll {
            items,
            chased,
            ..cursors
        }
    }
}

/// The boxes of Berry Trees' time since planting, said with its berry alone (`ListRows`).
pub fn berry_times(on_screen: &[ScreenText], drawn: Option<&str>) -> HashSet<i64> {
    if drawn == Some(BERRIES) {
        on_screen
            .iter()
            .filter(|p| p.top)
            .map(|p| p.r#box)
            .collect()
    } else {
        HashSet::new()
    }
}

/// The boxes of the top screen's layout naming the area: its name, its facilities and its
/// description, which Trainer's Eye draws above each trainer's card. The area is said as it
/// changes (`Places`), and its words were said again with every trainer once the card was
/// hidden, "Route 104, This path, rich with water…" (or_areanav_trainers).
pub fn place_block(on_screen: &[ScreenText], area: &str) -> HashSet<i64> {
    let roots: HashSet<i64> = on_screen
        .iter()
        .filter(|p| p.top && p.text == area)
        .map(|p| p.root)
        .collect();
    on_screen
        .iter()
        .filter(|p| roots.contains(&p.root))
        .map(|p| p.r#box)
        .collect()
}

/// The cursors' poll with the area said already taken out of each item's words, where more
/// are left: a list over the map names each row's place.
pub fn unplaced(cursors: CursorPoll, area: &str) -> CursorPoll {
    let items = cursors
        .items
        .iter()
        .map(|item| {
            let kept: Vec<ScreenText> = item
                .pieces
                .iter()
                .filter(|p| p.text != area)
                .cloned()
                .collect();
            if !kept.is_empty() && kept.len() < item.pieces.len() {
                CursorItem {
                    pieces: kept,
                    ..item.clone()
                }
            } else {
                item.clone()
            }
        })
        .collect();
    CursorPoll { items, ..cursors }
}
