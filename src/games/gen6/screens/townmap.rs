//! The town map, ORAS's AreaNav and X's Town Map: the place under its map's cursor (spike 309),
//! and the layouts a `G2DUtil` draws, by which ORAS's AreaNav's views and the Move Reminder's pages
//! are put away (spikes 181 and 183).
//!
//! The place under the map's cursor is the town map's cursor source (`Places`, spike 309): the Fly
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
//!
//! ORAS's AreaNav draws views over its map, which X's Town Map has none of: Berry Trees', Trainer's
//! Eye's and Secret Bases' lists and the Pokémon Distribution. They are ORAS's own
//! (`oras.subscreen.areanav`), and what its lists make of the cursors comes in through its `Own`
//! (`AreaNav`, `OwnLists`, `gen6.own`). A view draws the layouts of Game Freak's layout helper,
//! `app::util::G2DUtil`, as the Move Reminder's pages do (`reminder`):
//!
//! ```text
//! view     +0x5C  -> app::util::G2DUtil
//! G2DUtil  +0x1C  -> its layouts, 8 bytes each: a handle, then flags; +0x20 how many
//! handle   +0x04  the layout's work, as a layout pool's table keeps it
//! ```
//!
//! Bound by:
//! - ADR-0043: the place under the map's cursor is said as it changes, the area over a list
//!   only as it changes.

use std::collections::HashSet;
use std::rc::Rc;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::games::common::cursors::shared::CursorPoll;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::anchors::Kept;
use crate::py::Result;

/// The AreaNav as an app of its own, opened from the PokéNav's window or the Bag's Town Map.
pub const APP: &str = "N3app7townmap4ProcE";
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
/// 309 on x_townmap).
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

/// The place under the AreaNav's map cursor, read every tick from its `MapView` (module
/// docstring), said by the ROM's name for it as it changes: the screen's same words are then
/// a title already said, and its facilities and description that place's (`ScreenAnnouncer`'s
/// `title`).
///
/// A place is said each time the cursor comes onto it, back from no place too, with its words:
/// a player moving off a place and back onto it hears it again, unless it was off for a poll
/// alone, a flicker on its edge. A list drawn over the map,
/// Trainer's Eye, Berry Trees or Secret Bases, moves the map to its row's place: that place is
/// said first, as the area switched to, and only as it changes, and its rows say it no more
/// (`current`): Berry Trees said "Roseli Berry: Route 123", then "Lum Berry: Route 123".
pub struct Places {
    memory: Rc<dyn Memory>,
    shape: Shape,
    running: Rc<dyn Fn() -> Option<String>>,
    name: Rc<dyn Fn(i64) -> Option<String>>,
    /// Its `MapView`, once found and checked by its class.
    kept: Kept,
    /// The touch area at the last screen poll, None before the map's first.
    last: Option<i64>,
    /// The last place the cursor was on, said or the one the map opened on: not
    /// `NO_PLACE`.
    place: Option<i64>,
    polls: i64,
    /// The screen poll the cursor went onto no place.
    off: i64,
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
            kept: Kept::new(memory.clone(), process, running.clone()),
            memory,
            shape,
            running,
            name,
            last: None,
            place: None,
            polls: 0,
            off: -2,
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
        self.polls += 1;
        let Some(place) = self.read()? else {
            (self.last, self.place) = (None, None);
            return Ok(None);
        };
        if Some(place) == self.last {
            return Ok(None);
        }
        self.last = Some(place);
        if place == NO_PLACE {
            self.off = self.polls;
            return Ok(None);
        }
        if Some(place) == self.place && (listing || self.polls - self.off <= 1) {
            // Off it for a poll and back, a flicker on its edge: the cursor said Route 116 three
            // times in a second, its description once (or_bag_and_battle_bag).
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
        if (self.running)().as_deref() != Some(APP) {
            self.kept.forget();
            return Ok(None);
        }
        let view = self
            .kept
            .find(&self.shape.view, Some(MAP_VIEW_CLASS), &[])?;
        match view {
            None => Ok(None),
            Some(view) => Ok(Some(u32(
                &self.memory.read(view + self.shape.place, 4)?,
                0,
            )?)),
        }
    }
}

/// What the AreaNav asks of the lists a game draws over its map (`gen6.own.Own`): ORAS's
/// Berry Trees, Trainer's Eye and Secret Bases; none in X's Town Map.
pub trait OwnLists {
    /// Whether it drew a list over the town map at the last `put_away`.
    fn listing(&self) -> bool {
        false
    }

    /// The cursors as the lists it draws over the town map have them, the map on `area`.
    fn lists(
        &self,
        cursors: CursorPoll,
        _area: Option<&str>,
        _on_screen: &[ScreenText],
    ) -> CursorPoll {
        cursors
    }
}

/// What the AreaNav makes of a screen poll: the place under its map's cursor said as a title
/// (`Places`), and what a game's own lists drawn over the map make of the cursors, the area the
/// map is on given (`OwnLists`). `places` is None in a game whose `MapView` is not measured: the
/// place is then the screen's alone.
pub struct AreaNav {
    lists: Rc<dyn OwnLists>,
    places: Option<Places>,
}

impl AreaNav {
    pub fn new(lists: Rc<dyn OwnLists>, places: Option<Places>) -> Self {
        AreaNav { lists, places }
    }

    /// Every tick: has the map's cursor come onto another place (`Places.turned`)?
    pub fn turned(&mut self) -> Result<bool> {
        match &mut self.places {
            None => Ok(false),
            Some(places) => places.turned(),
        }
    }

    /// Every screen poll, before the cursors: the place the cursor has just come onto.
    pub fn title(&mut self) -> Result<Option<String>> {
        match &mut self.places {
            None => Ok(None),
            Some(places) => places.poll(self.lists.listing()),
        }
    }

    /// The cursors, a list over the map read row by row, each row without its area.
    pub fn cursors(&mut self, cursors: CursorPoll, on_screen: &[ScreenText]) -> CursorPoll {
        self.lists.lists(cursors, self.area().as_deref(), on_screen)
    }

    /// The place the map's cursor is on or last was, as of the last screen poll.
    pub fn area(&self) -> Option<String> {
        self.places.as_ref().and_then(|places| places.current())
    }
}
