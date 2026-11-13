//! ORAS's berry spots: the patches of soft soil a Berry is planted in (spike 141).
//!
//! No zone's data has one: the field's DLL lists all 90, which the ROM is read for
//! (`archives.berry_spots`).
//!
//! What grows in each is the save's, `savedata::KinomiData`, one all session, which keeps the save
//! file's block at 0x1c400 whole from +4: a record a spot, 16 bytes, in the order of the field's
//! table (spike 293):
//!
//! ```text
//! +0x00 u8   the stage, as the game says it facing the spot: 1 "This is where you planted the
//!            Pecha Berry!", 2 sprouting, 3 its trunk getting bigger, 4 in bloom, 5 "The Oran
//!            Berry tree has 2 Berries!"
//! +0x02 u16  minutes since it was planted, 0 for a tree the game planted itself
//! +0x04 u16  the soil's moisture: 0 as a Berry is planted, 240 as it is watered, a minute at a
//!            time down to 0, where the game says "The soil is dry..." (or_berry_varieties)
//! +0x06 u16  the Berry, an item: 155 Oran, 151 Pecha
//! +0x08 f32  how many Berries it gives
//! +0x0c u32  1 for a tree the game planted itself
//! ```
//!
//! Each spot is listed by what grows in it, as a sighted player sees it on the field and the game
//! says it facing it: "soft soil" with nothing planted, stage 0 as the game leaves a spot once its
//! crop is picked ("It's soft soil."); "planted berry", "berry sprout", "growing berry tree" and
//! "berry tree in bloom", each "dry" while its soil is ("The soil is dry..."); and a tree with
//! Berries to pick by its Berry, "harvestable Nanab Berry tree", the Berries hanging on it each its
//! own shape and colour, Pinap's orange, Nanab's pink, Pomeg's red. Before then the plant shows
//! nothing of its Berry: a Berry just planted is a mound of soil, darker once watered
//! (or_berry_varieties).

use std::cell::Cell as Slot;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u16;
use crate::core::discovery::Found;
use crate::core::memory::Memory;
use crate::core::pathfind::Cell;
use crate::core::words::own;
use crate::games::common::field::things::{BERRIES, BERRY_SOURCE, Interactable, place_source};
use crate::games::gen6::field::features::{Feature, Tracked, ZoneAt};
use crate::games::gen6::rom::archives::GameText;
use crate::py::{Error, Result};

/// What a spot berries are planted in is called: the game says "It's soft soil." of an empty one
/// and names the Berry growing in one, but never names the spot.
pub const BERRY_SPOT: &str = "berry spot";
/// A tree with Berries to pick whose Berry has no name in the game's text.
pub const HARVESTABLE: &str = "harvestable berry tree";
pub const SOFT_SOIL: &str = "soft soil";
/// Stages 1 to 4, as the game says each facing it: "This is where you planted the Pecha Berry!",
/// "The Pecha Berry is sprouting!", "The Pecha Berry's trunk is getting bigger!", "The Pecha Berry
/// tree is in bloom!".
pub const GROWING: [&str; 4] = [
    "planted berry",
    "berry sprout",
    "growing berry tree",
    "berry tree in bloom",
];
/// The same, while the soil is dry.
pub const DRY: [&str; 4] = [
    "dry planted berry",
    "dry berry sprout",
    "dry growing berry tree",
    "dry berry tree in bloom",
];
pub const SAVE_CLASS: &str = "N8savedata10KinomiDataE";
/// Where the save's object keeps its spots' records.
pub const RECORDS: i64 = 0x04;
pub const RECORD: i64 = 0x10;
pub const STAGE: i64 = 0x00;
pub const MOISTURE: i64 = 0x04;
pub const ITEM: i64 = 0x06;
/// The stage of a tree with Berries to pick.
pub const RIPE: i64 = 5;

/// Every berry spot, (zone, cell) by its number, for the field to list, each said by what
/// grows in it where the save is found; none without the ROM.
pub struct BerrySpots {
    memory: Rc<dyn Memory>,
    spots: Vec<(i64, Cell)>,
    items: Vec<String>,
    save: Slot<Option<i64>>,
}

impl BerrySpots {
    pub fn new(memory: Rc<dyn Memory>, spots: Vec<(i64, Cell)>, items: Vec<String>) -> Self {
        BerrySpots {
            memory,
            spots,
            items,
            save: Slot::new(None),
        }
    }

    /// The factory `Game.features` holds.
    pub fn of(memory: Rc<dyn Memory>, text: &GameText) -> Rc<dyn Feature> {
        Rc::new(BerrySpots::new(
            memory,
            text.berries.clone(),
            text.items.clone(),
        ))
    }

    /// What each of these spots is said as, by the save's records, read in one go.
    fn kinds(&self, indices: &[i64]) -> Result<IndexMap<i64, String>> {
        let Some(save) = self.save.get() else {
            return Ok(IndexMap::new());
        };
        let (Some(&first), Some(&last)) = (indices.iter().min(), indices.iter().max()) else {
            return Ok(IndexMap::new());
        };
        let records = self
            .memory
            .read(save + RECORDS + first * RECORD, (last - first + 1) * RECORD)?;
        let mut kinds: IndexMap<i64, String> = IndexMap::new();
        for &index in indices {
            let at = (index - first) * RECORD;
            let stage = *records
                .get((at + STAGE) as usize)
                .ok_or_else(|| Error::index("index out of range"))? as i64;
            if stage == 0 {
                kinds.insert(index, SOFT_SOIL.to_string());
            } else if stage == RIPE {
                let item = u16(&records, at + ITEM)?;
                let berry = if item < self.items.len() as i64 {
                    self.items[item as usize].as_str()
                } else {
                    ""
                };
                kinds.insert(
                    index,
                    if berry.is_empty() {
                        HARVESTABLE.to_string()
                    } else {
                        own("berry-tree-harvestable", &[("berry", &berry)])
                    },
                );
            } else if stage < RIPE {
                let dry = u16(&records, at + MOISTURE)? == 0;
                let kinds_of = if dry { DRY } else { GROWING };
                kinds.insert(index, kinds_of[(stage - 1) as usize].to_string());
            }
        }
        Ok(kinds)
    }
}

impl Tracked for BerrySpots {
    fn classes(&self) -> Vec<&'static str> {
        vec![SAVE_CLASS]
    }

    fn track(&self, found: &Found) -> Result<()> {
        if let Some(&save) = found.of(&[SAVE_CLASS]).first() {
            self.save.set(Some(save));
        }
        Ok(())
    }
}

impl Feature for BerrySpots {
    fn listed(&self, zone: i64, _zone_at: ZoneAt) -> Result<Vec<Interactable>> {
        let here: Vec<(i64, Cell)> = self
            .spots
            .iter()
            .enumerate()
            .filter(|&(_, &(home, _))| home == zone)
            .map(|(index, &(_, cell))| (index as i64, cell))
            .collect();
        let indices: Vec<i64> = here.iter().map(|&(index, _)| index).collect();
        let kinds = self.kinds(&indices)?;
        let mut out: Vec<Interactable> = Vec::new();
        for (index, cell) in here {
            out.push(Interactable {
                source: place_source(BERRY_SOURCE, index)?,
                home: Some(zone),
                ..Interactable::new(
                    BERRIES,
                    kinds.get(&index).map_or(BERRY_SPOT, String::as_str),
                    cell,
                )
            });
        }
        Ok(out)
    }
}
