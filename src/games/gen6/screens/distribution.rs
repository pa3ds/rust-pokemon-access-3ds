//! The AreaNav's Pokémon Distribution: each Pokémon's picture, named by its species.
//!
//! The Pokémon tab's View Details draws the place as a scene, with each Pokémon seen there walking
//! in it as its icon, a 40 by 30 picture and a touch area each. The icons are stored with a palette
//! in the ROM and expanded as they load, so no file holds the image a picture draws and the
//! texture index cannot name it (spikes 176 and 177). The AreaNav's navigator,
//! `app::tool::ZukanNavi`, lists the place's species, and loads the n-th one's icon into the n-th
//! texture of its icon loader (spike 178, on Route 101, 102, 103, 104 and Petalburg Woods, each
//! icon checked by eye):
//!
//! ```text
//! ZukanNavi +0x44     -> common::resource::PokeIcon
//!           +0x48     a record of 0x14 bytes a species: the national number, a u16, first;
//!                     +0x0c caught 2 or seen 3 (`STATE`), +0x0d how it is met (`WAY`)
//!           +0x1D8    how many records there are
//! PokeIcon  +0x0C     -> app::tool::LytTexReplaceSystem
//! LytTexReplaceSystem +0x04 -> its textures, a pointer each; +0x08 how many
//! app::tool::LytTexData +0x0C   the image's physical address, as a picture's material keeps it
//! ```
//!
//! A picture drawing the image of the n-th texture shows the n-th record's species. The game draws
//! no picture for a species the player has not seen, so a species named is one the game shows; and
//! while a place's icons load, every picture draws the grey shape of the last texture, past the
//! records, which names none.
//!
//! On the field the icons are the DexNav's, each a touch target named by `said`. In the AreaNav's
//! own app they are its Pokémon Distribution's, which no tap reaches, and `DistributionReader` says
//! them, a line a way.
//!
//! Bound by:
//! - ADR-0037: only "caught" is said, never "not caught", and each icon says whether it is met on
//!   the water or fishing.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{u16, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::screen::ScreenText;
use crate::games::common::{lyt, pictures};
use crate::py::{self, PySet, Result};

pub const CLASS: &str = "N3app4tool9ZukanNaviE";
pub const ICONS: i64 = 0x44;
pub const ICON_CLASS: &str = "N6common8resource8PokeIconE";
pub const REPLACE: i64 = 0x0C;
pub const REPLACE_CLASS: &str = "N3app4tool19LytTexReplaceSystemE";
pub const TEXTURES: i64 = 0x04;
pub const TEXTURE_COUNT: i64 = 0x08;
pub const IMAGE: i64 = 0x0C;
pub const RECORDS: i64 = 0x48;
pub const RECORD: i64 = 0x14;
pub const STATE: i64 = 0x0C;
pub const CAUGHT: i64 = 2;
/// A record's byte `STATE`: 2 for a species caught, 3 seen, 6 not seen, which the DexNav draws no
/// icon for (spike 195, movie27: Zigzagoon, Poochyena, Surskit, Masquerain and Magikarp 2, as the
/// save's Pokédex has them caught), and 1 for one met only by the DexNav's search and not seen,
/// which the AreaNav draws none for (spike 232: Ponyta, Mankey and Tyrogue on Jagged Pass). The
/// byte after it is the way they are met: 0 walking, in the grass or on a cave's floor, 1 on the
/// water, 2 fishing, 3 the DexNav's alone.
pub const SEEN: i64 = 3;
pub const WAY: i64 = 0x0D;
/// A record's byte `WAY` -> pa3ds's word for it, the one the Pokédex's habitat page says its icons
/// by (`habitat.WAYS`): the DexNav draws a Pokémon met walking on the land of its picture of the
/// area, one met surfing in a row along the top of the water and one fished for in a row below it
/// (movie27 on Route 102: Marill, Azumarill, Surskit and Masquerain 1, Magikarp, Goldeen and
/// Corphish 2). Walking says none: "grass" was wrong on a cave's floor, Meteor
/// Falls' Zubat and Bagon 0 (spike 232), and a Pokémon met in seaweed under water, which the game
/// shows as met in the grass, says none either. 3, met only by the DexNav's search, has no word: no
/// such Pokémon seen has been drawn yet.
///
/// Python's `dict`, as (byte, message) pairs in its order: it is looked up, and its values walked.
pub const WAYS: [(i64, &str); 2] = [(1, "habitat-water"), (2, "habitat-fishing")];
/// The place's name, UTF-16, the words the AreaNav's top screen titles it with (spike 177).
pub const PLACE: i64 = 0x25C;
pub const PLACE_SIZE: i64 = 0x40;
pub const RECORD_COUNT: i64 = 0x1D8;
/// Records the navigator has room for, from +0x48 to its count.
pub const MAX_RECORDS: i64 = 20;
pub const MAX_TEXTURES: i64 = 32;

/// Names a picture the Pokémon Distribution draws a Pokémon's icon with.
pub struct Distribution {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    species: Vec<String>,
    navis: PySet,
}

impl Distribution {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>, species: Vec<String>) -> Self {
        Distribution {
            memory,
            names,
            species,
            navis: PySet::new(),
        }
    }

    /// The navigators a sweep found: all of them, or with `adding` the new ones.
    pub fn track(&mut self, found: impl IntoIterator<Item = i64>, adding: bool) {
        self.navis = if adding {
            self.navis.union(&PySet::of(found))
        } else {
            PySet::of(found)
        };
    }

    /// The species a picture draws the icon of, in the game's words, or None.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        let record = if self.species.is_empty() {
            None
        } else {
            self.record(picture)?
        };
        match record {
            None => Ok(None),
            Some(record) => self.species_of(&record),
        }
    }

    /// What touch mode calls a picture of the DexNav's list (ADR-0037): the species, how it is met
    /// and "caught" where the DexNav draws it in colour, joined by `gap`, ", " under `[pa3ds.ui]
    /// pauses`. "Magikarp fishing caught", "Ralts": never "not caught", a silhouette being the
    /// game's way of saying so, and the way before "caught", which would join it as "caught
    /// fishing".
    ///
    /// Python's `gap` defaults to " ".
    pub fn said(&self, picture: i64, gap: &str) -> Result<Option<String>> {
        let record = if self.species.is_empty() {
            None
        } else {
            self.record(picture)?
        };
        let species = match &record {
            None => None,
            Some(record) => self.species_of(record)?,
        };
        let (Some(record), Some(species)) = (record, species) else {
            return Ok(None);
        };
        let met = record[WAY as usize] as i64;
        let way = WAYS
            .iter()
            .find(|(way, _)| *way == met)
            .map(|(_, word)| *word);
        let mut words = vec![species];
        if let Some(way) = way {
            words.push(own(way, &[]));
        }
        if record[STATE as usize] as i64 == CAUGHT {
            words.push(own(pictures::CAUGHT, &[]));
        }
        Ok(Some(words.join(gap)))
    }

    /// The place and what the AreaNav's Pokémon Distribution draws there, a line a way (ADR-0037),
    /// once it draws every Pokémon its navigator has seen or caught, else None: its icons load a
    /// few at a time, and as the place changes the last place's pictures draw the new place's
    /// records for a moment (spike 232). In the navigator's order, those met walking first, then
    /// each way's line led by its word, "caught" after a species caught: "Swablu, Seedot caught",
    /// "water: Marill", "fishing: Magikarp caught", with spaces for the commas and colons but under
    /// `[pa3ds.ui] pauses`. One line, the last of a way ran on into the next way's word, "Seedot
    /// caught, water".
    ///
    /// Python's `pauses` defaults to False.
    pub fn listed(
        &self,
        icons: impl IntoIterator<Item = i64>,
        pauses: bool,
    ) -> Result<Option<(String, Vec<String>)>> {
        let mut drawn: IndexMap<i64, BTreeSet<i64>> = IndexMap::new();
        if !self.species.is_empty() {
            for icon in icons {
                if let Some((navi, slot)) = self.found(icon)? {
                    drawn.entry(navi).or_default().insert(slot);
                }
            }
        }
        if drawn.len() != 1 {
            return Ok(None);
        }
        let (navi, slots) = drawn.into_iter().next().expect("one navigator");
        let records = self.records(navi)?;
        let shown: BTreeSet<i64> = records
            .iter()
            .enumerate()
            .filter(|(_, record)| [CAUGHT, SEEN].contains(&(record[STATE as usize] as i64)))
            .map(|(slot, _)| slot as i64)
            .collect();
        if slots != shown {
            return Ok(None);
        }
        let mut ways: IndexMap<Option<&str>, Vec<String>> = IndexMap::new();
        for record in slots.iter().map(|slot| &records[*slot as usize]) {
            let Some(species) = self.species_of(record)? else {
                return Ok(None);
            };
            let met = record[WAY as usize] as i64;
            let way = WAYS
                .iter()
                .find(|(way, _)| *way == met)
                .map(|(_, word)| *word);
            ways.entry(way)
                .or_default()
                .push(if record[STATE as usize] as i64 == CAUGHT {
                    format!("{species} {}", own(pictures::CAUGHT, &[]))
                } else {
                    species
                });
        }
        let (gap, colon) = if pauses { (", ", ": ") } else { (" ", " ") };
        let place = py::decode_utf16le(&self.memory.read(navi + PLACE, PLACE_SIZE)?);
        let place = place.split('\0').next().unwrap_or_default().to_string();
        let order = std::iter::once(None).chain(WAYS.iter().map(|(_, word)| Some(*word)));
        let lines = order
            .filter_map(|way| {
                let species = ways.get(&way)?;
                let led = way.map_or(String::new(), |way| format!("{}{colon}", own(way, &[])));
                Some(led + &species.join(gap))
            })
            .collect();
        Ok(Some((place, lines)))
    }

    fn species_of(&self, record: &[u8]) -> Result<Option<String>> {
        let number = u16(record, 0)?;
        Ok(if 0 < number && number < self.species.len() as i64 {
            Some(self.species[number as usize].clone()).filter(|name| !name.is_empty())
        } else {
            None
        })
    }

    /// The navigator's record of the species a picture draws the icon of, or None.
    fn record(&self, picture: i64) -> Result<Option<Vec<u8>>> {
        let Some((navi, slot)) = self.found(picture)? else {
            return Ok(None);
        };
        Ok(Some(
            self.memory.read(navi + RECORDS + slot * RECORD, RECORD)?,
        ))
    }

    fn records(&self, navi: i64) -> Result<Vec<Vec<u8>>> {
        let count = u32(&self.memory.read(navi + RECORD_COUNT, 4)?, 0)?.min(MAX_RECORDS);
        let data = self.memory.read(navi + RECORDS, count * RECORD)?;
        Ok((0..count)
            .map(|slot| py::slice(&data, slot * RECORD, (slot + 1) * RECORD).to_vec())
            .collect())
    }

    /// The navigator and the slot of its record of the species a picture draws the icon of, or
    /// None.
    fn found(&self, picture: i64) -> Result<Option<(i64, i64)>> {
        if self.navis.is_empty() {
            return Ok(None);
        }
        let memory = &self.memory;
        let material = u32(&memory.read(picture + lyt::PICTURE_MATERIAL, 4)?, 0)?;
        if material == 0 {
            return Ok(None);
        }
        let image = u32(&memory.read(material + lyt::MATERIAL_TEXTURE, 4)?, 0)?;
        for navi in self.navis.iter() {
            let textures = self.textures(navi)?;
            let mut slot = None;
            for (i, texture) in textures.iter().enumerate() {
                if u32(&memory.read(texture + IMAGE, 4)?, 0)? == image {
                    slot = Some(i as i64);
                    break;
                }
            }
            let Some(slot) = slot else {
                continue;
            };
            let count = u32(&memory.read(navi + RECORD_COUNT, 4)?, 0)?;
            return Ok(if slot >= count.min(MAX_RECORDS) {
                None
            } else {
                Some((navi, slot))
            });
        }
        Ok(None)
    }

    /// The navigator's icon textures in order, or none where a link of the chain is not the
    /// class it should be: a navigator gone, its memory taken by something else.
    fn textures(&self, navi: i64) -> Result<Vec<i64>> {
        let (memory, names) = (&self.memory, &self.names);
        if names.of_vtable(u32(&memory.read(navi, 4)?, 0)?)?.as_deref() != Some(CLASS) {
            return Ok(Vec::new());
        }
        let icons = u32(&memory.read(navi + ICONS, 4)?, 0)?;
        if icons == 0
            || names
                .of_vtable(u32(&memory.read(icons, 4)?, 0)?)?
                .as_deref()
                != Some(ICON_CLASS)
        {
            return Ok(Vec::new());
        }
        let replace = u32(&memory.read(icons + REPLACE, 4)?, 0)?;
        if replace == 0
            || names
                .of_vtable(u32(&memory.read(replace, 4)?, 0)?)?
                .as_deref()
                != Some(REPLACE_CLASS)
        {
            return Ok(Vec::new());
        }
        let head = memory.read(replace + TEXTURES, 8)?;
        let (listed, count) = (u32(&head, 0)?, u32(&head, 4)?);
        if listed == 0 || !(0 < count && count <= MAX_TEXTURES) {
            return Ok(Vec::new());
        }
        let table = memory.read(listed, 4 * count)?;
        (0..count).map(|i| u32(&table, 4 * i)).collect()
    }
}

/// Screen polls a place's lines wait for the top screen to title it, about a second at the default
/// poll rate, then are said all the same: a place named otherwise there is not left unsaid.
pub const TITLE_POLLS: i64 = 60;

/// The AreaNav's Pokémon Distribution, a line a way, once each place's icons are all drawn
/// (`Distribution.listed`) and the top screen has titled the place: the icons were drawn up to a
/// quarter of a second before the title, and were heard before the place they are of (spike 232).
/// Its icons are this reader's (`owns`), as the habitat page's words are its reader's: left to
/// screen text, icons standing side by side ran together as one word, in the order the picture
/// stands them, "NuzleafZangooseSurskit".
pub struct DistributionReader {
    distribution: Rc<RefCell<Distribution>>,
    pauses: bool,
    owned: PySet,
    /// The top screen's words a poll ago: words there two polls running are reported.
    top: HashSet<String>,
    /// The lines last said, until the icons leave or change: a place said again once opened again.
    said: Vec<String>,
    waited: i64,
}

impl DistributionReader {
    /// Python's `pauses` defaults to False.
    pub fn new(distribution: Rc<RefCell<Distribution>>, pauses: bool) -> Self {
        DistributionReader {
            distribution,
            pauses,
            owned: PySet::new(),
            top: HashSet::new(),
            said: Vec::new(),
            waited: 0,
        }
    }

    pub fn owns(&self, piece: &ScreenText) -> bool {
        self.owned.contains(piece.r#box)
    }

    /// The lines to say after the screen's words this poll, if any. `running`: the AreaNav's app
    /// is; elsewhere its navigators are gone (`Distribution`).
    pub fn poll(&mut self, on_screen: &[ScreenText], running: bool) -> Result<Vec<String>> {
        let distribution = self.distribution.borrow();
        let mut owned = Vec::new();
        for p in on_screen {
            if running && p.picture && distribution.name(p.r#box)?.is_some() {
                owned.push(p.r#box);
            }
        }
        self.owned = PySet::of(owned);
        let listed = distribution.listed(self.owned.iter(), self.pauses)?;
        let top: HashSet<String> = on_screen
            .iter()
            .filter(|p| p.top)
            .map(|p| p.text.clone())
            .collect();
        let titled: HashSet<String> = top.intersection(&self.top).cloned().collect();
        self.top = top;
        let Some((place, lines)) = listed else {
            (self.said, self.waited) = (Vec::new(), 0);
            return Ok(Vec::new());
        };
        if lines == self.said {
            return Ok(Vec::new());
        }
        self.waited += 1;
        if !titled.contains(&place) && self.waited < TITLE_POLLS {
            return Ok(Vec::new());
        }
        (self.said, self.waited) = (lines.clone(), 0);
        Ok(lines)
    }
}
