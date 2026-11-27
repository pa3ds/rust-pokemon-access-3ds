//! What one game brings to the gen 6 field of its own: ORAS's DexNav, Secret Bases, berry spots,
//! ash piles, rotating doors and thin ice (`games.oras`), each found by a sweep as its module loads.
//! The field reader reads them through these, and never imports a game.

use std::any::Any;
use std::rc::Rc;

use indexmap::IndexSet;

use crate::core::discovery::Found;
use crate::core::pathfind::Cell;
use crate::games::common::field::ground::IceRoom;
use crate::games::common::field::rotating::RotatingDoor;
use crate::games::common::field::things::Interactable;
use crate::py::Result;

/// `Callable[[Cell], int | None]`: the zone the field map puts a cell in.
pub type ZoneAt<'a> = &'a dyn Fn(Cell) -> Result<Option<i64>>;

/// One of a game's own readers on the field, found by a sweep as its module loads.
///
/// `Any`, so a landform can tell its own watch (`landform.as_watch`), as Python's `isinstance`
/// does.
pub trait Tracked: Any {
    /// What a sweep looks for to find it by.
    fn classes(&self) -> Vec<&'static str>;

    fn track(&self, found: &Found) -> Result<()>;
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

/// A game's rotating doors, where it has them: Fortree Gym's (`games.oras.field.doors`). Read
/// whenever the ground is, since they turn as the player walks.
pub trait DoorReader: Tracked {
    /// The doors in `zone` as they stand now, or none.
    fn doors(&self, zone: i64) -> Result<Vec<RotatingDoor>>;
}

/// A game's rooms of thin ice, where it has them: Sootopolis Gym's (`oras.field.ice`). Read
/// whenever the ground is, since they crack as the player walks.
pub trait IceReader: Tracked {
    /// The rooms in `zone` as they are now, or none.
    fn rooms(&self, zone: i64) -> Result<Vec<IceRoom>>;
}

/// A game's Secret Bases, where it has them: ORAS's (`oras.field.secret_base`), listing every
/// spot; inside one, what stands where and the cursor the player decorates with.
pub trait SecretBaseReader: Feature {
    /// Every zone a base is built in.
    fn zones(&self) -> IndexSet<i64>;

    /// Whether a sweep has found what stands where in a base.
    fn furnished(&self) -> Result<bool>;

    /// The cell the decorating cursor is on and whether the item held can be set down there,
    /// None for nothing held; or None while it is not shown.
    fn cursor(&self) -> Result<Option<(Cell, Option<bool>)>>;

    /// The decorations standing in a cell, by number.
    fn standing(&self, cell: Cell) -> Result<Vec<i64>>;

    /// The cells a decoration stands in that is walked round, not over.
    fn solid(&self) -> Result<IndexSet<Cell>>;

    /// Each decoration standing in the base, by number, with its cells; none the game hides.
    fn placed(&self) -> Result<Vec<(i64, Vec<Cell>)>>;
}
