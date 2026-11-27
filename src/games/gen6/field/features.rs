//! What one game brings to the gen 6 field of its own: ORAS's DexNav, Secret Base spots, berry
//! spots and ash piles (`games.oras`), each found by a sweep as its module loads. The field reader
//! reads them through these, and never imports a game. What it has of its own on the ground is its
//! landforms' (`landform`).

use std::any::Any;
use std::rc::Rc;

use crate::core::discovery::Found;
use crate::core::pathfind::Cell;
use crate::games::common::field::things::Interactable;
use crate::games::gen6::field::voice::Speaking;
use crate::py::Result;

/// `Callable[[Cell], int | None]`: the zone the field map puts a cell in.
pub type ZoneAt<'a> = &'a dyn Fn(Cell) -> Result<Option<i64>>;

/// One of a game's own readers on the field, found by a sweep as its module loads: a feature,
/// or a landform's watch (`landform.Landform.watch`).
///
/// `Any`, so a landform can tell its own watch (`landform.as_watch`), as Python's `isinstance`
/// does.
pub trait Tracked: Any {
    /// What a sweep looks for to find it by.
    fn classes(&self) -> Vec<&'static str>;

    fn track(&self, found: &Found) -> Result<()>;

    /// Itself as `Sweeping`, where it is one: Python's `isinstance(reader, Sweeping)`, a
    /// runtime-checkable protocol.
    fn sweeping(&self) -> Option<&dyn Sweeping> {
        None
    }

    /// Itself as `voice.Speaking`, where it is one: Python's `isinstance(watch, Speaking)`, a
    /// runtime-checkable protocol.
    fn speaking(self: Rc<Self>) -> Option<Rc<dyn Speaking>> {
        None
    }
}

/// One of a game's own readers that may want a sweep sooner than the schedule has it: ORAS's
/// Secret Bases' decorations, inside a base before they are found
/// (`oras.field.secret_base.Decorations`).
pub trait Sweeping: Tracked {
    /// Why to sweep now, or None: `zone` is the one the player's model was built in, None while
    /// no poll has read it, and `clock` the field reader's, asked only when it matters.
    fn sweep_wanted(
        &self,
        zone: Option<i64>,
        clock: &dyn Fn() -> f64,
    ) -> Result<Option<&'static str>>;
}

/// Something a game has on its field that the gen 6 field reader lists without knowing what
/// it is, and says in its own words: ORAS's berry spots, ash piles, Secret Base spots and the
/// DexNav's find (`games.oras`).
pub trait Feature: Tracked {
    /// What it has in `zone` now. `zone_at` is the zone the field map puts a cell in, for a
    /// thing that keeps no zone of its own: the DexNav's find.
    fn listed(&self, zone: i64, zone_at: ZoneAt) -> Result<Vec<Interactable>>;

    /// Itself as `Watched`, where it is one: Python's `isinstance(feature, Watched)`, a
    /// runtime-checkable protocol.
    fn watched(self: Rc<Self>) -> Option<Rc<dyn Watched>> {
        None
    }
}

/// What a watched feature shows now (`Watched.watch`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sighting {
    /// What is up in the zone asked about, as `listed` lists it: the field is listed again as it
    /// changes.
    pub up: Option<Interactable>,
    /// What the game holds for it before any check, for the log as it changes; None before a
    /// sweep has found where.
    pub record: Option<String>,
    /// `up`, where the player went looking for it, to be guided to as it first shows.
    pub find: Option<Interactable>,
    /// Whether the player's own search found `find`, which is guided to in any mode, where any
    /// other find is only in touch mode (`Navigator.found`).
    pub searched: bool,
}

/// A feature that shows and goes between sweeps, and so is read at every poll: ORAS's DexNav's
/// find (`games.oras.field.hidden`), which decides itself which of its finds the player went
/// looking for.
pub trait Watched: Feature {
    /// What it shows in `zone` now, `zone_at` as `listed` takes it.
    fn watch(&self, zone: i64, zone_at: ZoneAt) -> Result<Sighting>;
}
