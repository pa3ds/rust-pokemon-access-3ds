//! The AreaNav's views, which say which of its layouts it draws (spikes 181 and 183).
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
//! reads it. `MapView` and `GraphicWork` draw whenever the AreaNav does, and are not listed.
//!
//! ```text
//! view     +0x5C  -> app::util::G2DUtil
//! G2DUtil  +0x1C  -> its layouts, 8 bytes each: a handle, then flags; +0x20 how many
//! handle   +0x04  the layout's work, as a layout pool's table keeps it
//! ```
//!
//! The place under the map's cursor is the AreaNav's cursor source (`Places`, spike M): the Fly
//! map's, the Town Map's, and the place a View Details pages to. Its `MapView` keeps the touch
//! area the top screen tells of, and the module's code sets it before writing the place's words:
//!
//! ```text
//! app::townmap::Proc         +0x24  -> app::townmap::MainSystem
//! app::townmap::MainSystem   +0x18  -> app::townmap::GraphicWork
//! app::townmap::GraphicWork  +0xA4  -> app::townmap::MapView
//!                            +0xCC  u8 the view drawn over the map: 0 none, 2 the Pokémon
//!                                   Distribution, 3 Trainer's Eye, 4 Secret Bases, 5 Berry Trees
//! app::townmap::MapView      +0xEC  u32 the touch area, its index among the map's panes'
//!                                   names (`GameText.map_pane_names`), 0xFFFFFFFF over none
//! ```
//!
//! It changes a screen poll before the top screen's words are read, and a place crossed in less
//! than a poll's rewrite, as the Fly map's cursor glides, was heard only inside the next one's
//! details, "Mauville City", then "Route 110, Cycling Road Trick House, …" (or_bag_and_battle_bag).

use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::anchors::follow;
use crate::games::gen6::apps::LOADED_CODE;
use crate::py::Result;

/// The AreaNav as an app of its own, opened from the PokéNav's window or the Bag's Town Map.
pub const APP: &str = "N3app7townmap4ProcE";
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
pub const UTIL: i64 = 0x5C;
pub const UTIL_CLASS: &str = "N3app4util7G2DUtilE";
pub const LAYOUTS: i64 = 0x1C;
pub const LAYOUT_COUNT: i64 = 0x20;
pub const ENTRY: i64 = 8;
pub const WORK: i64 = 0x04;
pub const MAX_LAYOUTS: i64 = 16;
pub const MAP_VIEW_CLASS: &str = "N3app7townmap7MapViewE";

/// Where a game's town map keeps the place under its cursor (module docstring).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shape {
    /// The process -> `MainSystem` -> `GraphicWork` -> `MapView`.
    pub view: [i64; 3],
    /// `MapView` -> the touch area under the map's cursor, by its index among the map's panes.
    pub place: i64,
}

pub const ORAS: Shape = Shape {
    view: [0x24, 0x18, 0xA4],
    place: 0xEC,
};
/// X's: `GraphicWork` at `MainSystem` +0x14, `MapView` at its +0x90, the place 4 bytes on (spike
/// M on x_townmap).
pub const XY: Shape = Shape {
    view: [0x24, 0x14, 0x90],
    place: 0xF0,
};
pub const NO_PLACE: i64 = 0xFFFFFFFF;

/// The works of the layouts a `G2DUtil` draws.
pub fn util_works(memory: &dyn Memory, util: i64) -> Result<HashSet<i64>> {
    let listed = memory.read(util + LAYOUTS, 8)?;
    let (table, count) = (u32(&listed, 0)?, u32(&listed, 4)?);
    if table == 0 || !(0 < count && count <= MAX_LAYOUTS) {
        return Ok(HashSet::new());
    }
    let entries = memory.read(table, ENTRY * count)?;
    let mut works: HashSet<i64> = HashSet::new();
    for i in 0..count {
        let handle = u32(&entries, ENTRY * i)?;
        if handle != 0 {
            let work = u32(&memory.read(handle + WORK, 4)?, 0)?;
            if work != 0 {
                works.insert(work);
            }
        }
    }
    Ok(works)
}

/// The AreaNav's views a sweep found, and the layouts of those it has put away.
pub struct Views {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    /// View -> the vtable it was found with, where its byte saying it is drawn is, and its
    /// class.
    views: IndexMap<i64, (i64, i64, &'static str)>,
    /// A `Cell`, as `put_away` sets it through `&self`.
    listing: Cell<bool>,
}

impl Views {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        Views {
            memory,
            names,
            views: IndexMap::new(),
            listing: Cell::new(false),
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
                views.insert(view, (u32(&self.memory.read(view, 4)?, 0)?, drawn_at, class));
            }
        }
        self.views = views;
        Ok(())
    }

    /// Whether the AreaNav draws one of these views over its map now: the Pokémon
    /// Distribution, Trainer's Eye's rows, the berry list. Then a tap on the map reaches the view's
    /// own buttons, not a place (or_area_nav).
    pub fn drawing(&self) -> Result<bool> {
        Ok(self.read()?.iter().any(|&(drawn, _util, _name)| drawn))
    }

    /// Whether the AreaNav drew one of its lists over its map at the last `put_away`
    /// (`LISTS`): read again, its reads were new to the screen poll's tapes.
    pub fn listing(&self) -> bool {
        self.listing.get()
    }

    /// The works of the layouts of every view the AreaNav is not drawing now.
    pub fn put_away(&self) -> Result<HashSet<i64>> {
        let mut works: HashSet<i64> = HashSet::new();
        let read = self.read()?;
        self.listing.set(
            read.iter()
                .any(|&(drawn, _util, name)| drawn && LISTS.contains(&name)),
        );
        for (drawn, util, _name) in read {
            if !drawn {
                works.extend(util_works(&*self.memory, util)?);
            }
        }
        Ok(works)
    }

    /// Each view still one, whether it is drawn, its `G2DUtil` and its class.
    fn read(&self) -> Result<Vec<(bool, i64, &'static str)>> {
        let memory = &self.memory;
        let mut read: Vec<(bool, i64, &'static str)> = Vec::new();
        for (&view, &(vtable, drawn_at, name)) in &self.views {
            let head = memory.read(view, drawn_at.max(UTIL) + 4)?;
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
                read.push((head[drawn_at as usize] != 0, util, name));
            }
        }
        Ok(read)
    }
}

/// The place under the AreaNav's map cursor, read every tick from its `MapView` (module
/// docstring), said by the ROM's name for it as it changes: the screen's same words are then
/// a title already said, and its facilities and description that place's (`ScreenAnnouncer`'s
/// `title`).
///
/// A place is said each time the cursor comes onto it, back from no place too, with its words:
/// a player moving off a place and back onto it hears it again. A list drawn over the map,
/// Trainer's Eye, Berry Trees or Secret Bases, moves the map to its row's place: that place is
/// said first, as the area switched to, and only as it changes, and its rows say it no more
/// (`current`): Berry Trees said "Roseli Berry: Route 123", then "Lum Berry: Route 123".
pub struct Places {
    memory: Rc<dyn Memory>,
    shape: Shape,
    running: Rc<dyn Fn() -> Option<String>>,
    process: Rc<dyn Fn() -> Option<i64>>,
    name: Rc<dyn Fn(i64) -> Option<String>>,
    /// The process and its `MapView`, once found and checked by its class, or None for one
    /// not built yet.
    view: Option<(i64, Option<i64>)>,
    /// The touch area at the last screen poll, None before the map's first.
    last: Option<i64>,
    /// The last place the cursor was on, said or the one the map opened on: not
    /// `NO_PLACE`.
    place: Option<i64>,
}

impl Places {
    /// `name` is what the game calls the place of a touch area, by its index, or None;
    /// `shape` where the game keeps it (Python's default `ORAS`).
    pub fn new(
        memory: Rc<dyn Memory>,
        running: Rc<dyn Fn() -> Option<String>>,
        process: Rc<dyn Fn() -> Option<i64>>,
        name: Rc<dyn Fn(i64) -> Option<String>>,
        shape: Shape,
    ) -> Self {
        Places {
            memory,
            shape,
            running,
            process,
            name,
            view: None,
            last: None,
            place: None,
        }
    }

    /// Has the cursor come onto another place since the last screen poll? Asked every tick,
    /// one read, so that a screen poll comes as it does.
    pub fn turned(&mut self) -> Result<bool> {
        let place = self.read()?;
        Ok(place.is_some_and(|place| Some(place) != self.last && place != NO_PLACE))
    }

    /// The name of the place the cursor has just come onto, the map's first as it opens, or
    /// None: nothing over no place, where the game writes nothing, nor, while a list is drawn
    /// over the map (`listing`), the area it was already in. X writes a place's description
    /// before its name, and said as they came the Town Map opened on "A rural town where
    /// Pokémon and windmills…", then "Dendemille Town" (x_townmap).
    pub fn poll(&mut self, listing: bool) -> Result<Option<String>> {
        let Some(place) = self.read()? else {
            (self.last, self.place) = (None, None);
            return Ok(None);
        };
        if Some(place) == self.last {
            return Ok(None);
        }
        self.last = Some(place);
        if place == NO_PLACE || (listing && Some(place) == self.place) {
            return Ok(None);
        }
        self.place = Some(place);
        Ok((self.name)(place))
    }

    /// The name of the place the cursor is on or last was, as of the last screen poll.
    pub fn current(&self) -> Option<String> {
        self.place.and_then(|place| (self.name)(place))
    }

    /// The touch area under the cursor, while the AreaNav runs and its `MapView` is built.
    fn read(&mut self) -> Result<Option<i64>> {
        let process = (self.process)();
        let Some(process) = process.filter(|_| (self.running)().as_deref() == Some(APP)) else {
            self.view = None;
            return Ok(None);
        };
        if self
            .view
            .is_none_or(|(found, view)| found != process || view.is_none())
        {
            let view = follow(&*self.memory, process, &self.shape.view)?;
            let names = ClassNames::new(self.memory.clone(), LOADED_CODE);
            // Its class is the module's, named while it is loaded.
            let checked = match view {
                Some(view) if names.of_object(view)?.as_deref() == Some(MAP_VIEW_CLASS) => {
                    Some(view)
                }
                _ => None,
            };
            self.view = Some((process, checked));
        }
        let Some((_, Some(view))) = self.view else {
            return Ok(None);
        };
        Ok(Some(u32(
            &self.memory.read(view + self.shape.place, 4)?,
            0,
        )?))
    }
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
