//! Place names: what the game calls where the player arrives and where an exit leads, a part of a
//! place in parts by pa3ds's own name for it, and a slot on a map the game's text names.
//!
//! Every index is the zone table's in RAM and every word the ROM's: without the ROM nothing is
//! named here, and AreaNav's words still say where the player is, a moment later.
//!
//! Bound by:
//! - ADR-0042: a part of a place in parts is said walking into it, every time, and an exit or a leg
//!   into it says it (`part_name`, `PlaceNames.arrived`, `PlaceNames.leads_to`,
//!   `PlaceNames.place`).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::words::{current, own};
use crate::games::common::field::things::{
    FURNITURE_SOURCE, Interactable, MODEL_SOURCE, WARP_SOURCE, source_kind, source_number,
};
use crate::games::gen6::field::reader::FieldReader;
use crate::games::gen6::rom::archives::GameText;
use crate::py::Result;

/// The game's names for places and the slots on its maps, as the navigator and the arrival in a
/// place ask them.
pub struct PlaceNames {
    text: Rc<GameText>,
    items_at: IndexMap<(i64, i64), i64>,
    field: Rc<dyn Fn() -> Rc<FieldReader>>,
    /// Whether the player is a girl, and their name, as the last whole sweep read the save
    /// (`mystatus.player`): which of Littleroot's houses is theirs.
    pub player: RefCell<Option<(bool, String)>>,
    /// The place-name index of the zone the player was last polled in.
    place: Cell<Option<i64>>,
    /// The part of a place in parts last said, or None since the player left one.
    part: RefCell<Option<String>>,
}

impl PlaceNames {
    /// `text` is the game's own words out of the player's ROM, `items_at` the item a thing is by
    /// its record (`Game.items_at`), and `field` the field reader, whose zone table and parts say
    /// where each zone is.
    pub fn new(
        text: Rc<GameText>,
        items_at: IndexMap<(i64, i64), i64>,
        field: Rc<dyn Fn() -> Rc<FieldReader>>,
    ) -> Self {
        PlaceNames {
            text,
            items_at,
            field,
            player: RefCell::new(None),
            place: Cell::new(None),
            part: RefCell::new(None),
        }
    }

    /// What the game calls a slot on a map: the building behind a door in from outdoors, where
    /// the ROM's own tables pair the two (spike 86). Only a warp's slot is a door's: a sign is
    /// numbered in a table of its own. A warp into one of X's gates is the gate and the place
    /// beyond it, "Lumiose Gate to Route 4": Lumiose's five gates are all "Lumiose Gate".
    /// A thing the player takes an item at is that item, by its record
    /// (`Game.items_at`): Route 111's Root Fossil and Claw Fossil.
    pub fn slot(&self, zone: i64, slot: i64) -> Option<String> {
        let text = &self.text;
        if source_kind(slot) == MODEL_SOURCE {
            let item = self.items_at.get(&(zone, source_number(slot)));
            let items = &text.items;
            return match item {
                Some(&item) if item < items.len() as i64 => {
                    Some(items[item as usize].clone()).filter(|name| !name.is_empty())
                }
                _ => None,
            };
        }
        if source_kind(slot) != WARP_SOURCE {
            return None;
        }
        if let Some((gate, beyond)) = text.gate(zone, source_number(slot)) {
            return Some(own("leads-to", &[("thing", &gate), ("place", &beyond)]));
        }
        if let Some((girl, name)) = self.player.borrow().as_ref()
            && let Some(home) = text.home(zone, source_number(slot), *girl, name)
        {
            return Some(home);
        }
        text.door(zone, source_number(slot))
    }

    /// Whether a slot on a map is a sign that says only the name of a door there
    /// (`plates.door_signs`).
    pub fn door_sign(&self, zone: i64, slot: i64) -> bool {
        source_kind(slot) == FURNITURE_SOURCE
            && self.text.door_signs.contains(&(zone, source_number(slot)))
    }

    /// What the game calls the place an exit from `here` into `there` arrives in, or None when
    /// it is the place the player is in already.
    ///
    /// A place, not a zone, for the reason `arrived` gives: Littleroot and every house in it are
    /// one place, so the door of a house leads nowhere new and says nothing more, while the gap
    /// at the top of the town leads onto Route 101 and says so.
    ///
    /// The other zone of the player's own matrix is said all the same: Route 119 is two zones
    /// the game calls by one name, and the edge between them was the only exit said bare, "edge"
    /// (spike 159). And an exit into a part of a place in parts says the
    /// part, from anywhere: the way out of the Weather Institute, "way out to Northern Route 119"
    /// (ADR-0042).
    pub fn leads_to(&self, here: i64, there: i64) -> Result<Option<String>> {
        let text = &self.text;
        if !text.loaded() {
            return Ok(None);
        }
        let field = (self.field)();
        let zones = &field.zones;
        let (ours, theirs) = (zones.place_index(here)?, zones.place_index(there)?);
        let (Some(ours), Some(theirs)) = (ours, theirs) else {
            return Ok(None);
        };
        let place = text.place(theirs);
        if let Some(name) = &place
            && there != here
            && let Some(part) = part_name(there, name)
        {
            return Ok(Some(part));
        }
        if ours == theirs && !zones.one_matrix(here, there)? {
            return Ok(None);
        }
        Ok(place)
    }

    /// What the game calls the place `zone` is in, or the part of it where it is a place in
    /// parts, "Southern Route 119" (ADR-0042); None where not known.
    pub fn place(&self, zone: i64) -> Result<Option<String>> {
        let index = (self.field)().zones.place_index(zone)?;
        let place = match index {
            Some(index) if self.text.loaded() => self.text.place(index),
            _ => None,
        };
        Ok(place.map(|place| {
            part_name(zone, &place)
                .filter(|part| !part.is_empty())
                .unwrap_or(place)
        }))
    }

    /// The part of the player's place in parts `item` stands in, where it is another part than
    /// the player's `zone` and has a name: said after its bearing (ADR-0042). None for an exit
    /// into that part, which says it already, and for anything of the player's own part.
    pub fn part(&self, item: &Interactable, zone: i64) -> Result<Option<String>> {
        let field = (self.field)();
        let Some(there) = field.zone_at(item.cell)? else {
            return Ok(None);
        };
        if there == zone
            || item.zone == Some(there)
            || !field.parts(zone).contains(&there)
            || !self.text.loaded()
        {
            return Ok(None);
        }
        let index = field.zones.place_of(there)?;
        let place = index.and_then(|index| self.text.place(index));
        Ok(place.and_then(|place| part_name(there, &place)))
    }

    /// The street of Lumiose's a warp between two of its zones comes out on, in the game's
    /// own words (spike 107): its zones are all "Lumiose City", so the place says nothing.
    pub fn street(&self, zone: i64, slot: i64) -> Option<String> {
        if source_kind(slot) != WARP_SOURCE {
            return None;
        }
        self.text.street(zone, source_number(slot))
    }

    /// What to say of where the player is, polled in `zone`, and what the game's sign shows
    /// where pa3ds says more (`ScreenAnnouncer.place`); None where nothing is said.
    ///
    /// Said when the place changes, not the zone: a town and every building in it share a
    /// place, so walking out of a house arrives nowhere, as the game has it (spike 87).
    ///
    /// A part of a place in parts is said too as the player walks into it, every time, from the
    /// next part or from another place, "Safari Zone Northwest", with the place the sign says:
    /// the confirmation they have moved that the game, its zones all one place, never gives
    /// (ADR-0042). Into a house of the place and back out into the part it stands in, nothing is
    /// said.
    pub fn arrived(&self, zone: i64) -> Result<Option<(String, Option<String>)>> {
        if !self.text.loaded() {
            return Ok(None);
        }
        let Some(index) = (self.field)().zones.place_index(zone)? else {
            return Ok(None);
        };
        let name = self.text.place(index);
        let part = name.as_deref().and_then(|name| part_name(zone, name));
        if part.is_some() && part != *self.part.borrow() {
            self.place.set(Some(index));
            *self.part.borrow_mut() = part.clone();
            return Ok(Some((part.unwrap_or_default(), name)));
        }
        if Some(index) == self.place.get() {
            return Ok(None);
        }
        self.place.set(Some(index));
        *self.part.borrow_mut() = part;
        Ok(name.map(|name| (name, None)))
    }
}

/// What pa3ds calls `zone` as a part of a place in parts, the game's `place` in it, "Southern
/// Route 119"; None for a zone that is no part (ADR-0042, the game's `zones.ftl`).
pub fn part_name(zone: i64, place: &str) -> Option<String> {
    current().get(&format!("part-{zone:#x}"), &[("place", &place)])
}
