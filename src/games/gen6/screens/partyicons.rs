//! The Bag's party slots, named by the species whose icon each draws (spike 184).
//!
//! The Bag lists the party beside its items, each slot an icon, a level and an HP bar, and no name:
//! touch mode said "L 14" (Orion, movie21). The icons come from a `common::resource::PokeIcon`, as
//! the AreaNav's Pokémon Distribution's do (`gen6.screens.distribution`): its `LytTexReplaceSystem` holds a
//! texture a party member, in the party's order, and each slot's picture draws its member's. The
//! party is a list of six `pml::pokepara::PokemonParam` pointers and then how many are in it, with
//! no class of its own:
//!
//! ```text
//! party     +0x00  six pointers to PokemonParams    +0x18 u8 how many are in the party
//! PokeIcon  +0x0C  -> LytTexReplaceSystem: +0x04 its textures, +0x08 how many
//! texture   +0x0C  the image's physical address, as a picture's material keeps it
//! ```
//!
//! The count is a byte: the Battle Maison sets the one after it to 2, and the party read as a u32
//! of 518 was no party. Its screen choosing the Battle Box or the party draws both on the top
//! screen from one PokeIcon of twelve textures, the Battle Box's six and then the party's, each
//! slot a level and an icon: "L 73 L 71" (or_multi_battle_partners).
//!
//! There the Maison keeps five more lists of six, one the Battle Box's, counting its two, which
//! was taken for the party: the party's first two were said as the Battle Box's (or_battle_box).
//! The party's list is the one whose members are the save's party, `savedata::PokePartySave`, by
//! each record's encryption constant; the Battle Box's icons are named from `savedata::BattleBox`:
//!
//! ```text
//! PokePartySave  +0x04  six records 0x104 apart, then +0x61C the count, a byte
//! BattleBox      +0x04  six records 0xE8 apart
//! ```
//!
//! Only while the Bag or that screen runs: another screen's icons are not the party's in its
//! order.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::screens::distribution as d;
use crate::py::{PySet, Result};

pub const BAG: &str = "N3app3bag4ProcE";
pub const GROUPS: &str = "N3app19battle_party_select4ProcE";
pub const ICON_CLASS: &str = d::ICON_CLASS;
pub const SLOTS: i64 = 6;

/// App -> the texture holding the party's first member's icon.
pub const FIRST: [(&str, i64); 2] = [(BAG, 0), (GROUPS, SLOTS)];

/// `FIRST.get(app)`.
fn first(app: &str) -> Option<i64> {
    FIRST
        .iter()
        .find(|(running, _)| *running == app)
        .map(|&(_, first)| first)
}
pub const COUNT: i64 = 0x18;
pub const PARTY_SAVE: &str = "N8savedata13PokePartySaveE";
pub const BATTLE_BOX: &str = "N8savedata9BattleBoxE";
/// Where each save object's records start.
pub const RECORDS: i64 = 0x04;
/// How far apart the save's party's records are: a record and the stats a party member has.
pub const SAVED: i64 = 0x104;

/// The party's list among the addresses holding a PokemonParam: six pointers to them, the
/// first `COUNT` of them filled, then the count. Of several, the one whose members' encryption
/// constants are `saved`, the save's party's, in order.
pub fn party_of(
    holders: impl IntoIterator<Item = i64>,
    memory: &dyn Memory,
    names: &ClassNames,
    saved: &[i64],
) -> Result<Option<i64>> {
    let mut found: Vec<i64> = Vec::new();
    for at in holders {
        let head = memory.read(at, COUNT + 4)?;
        let count = head[COUNT as usize] as i64;
        if !(0 < count && count <= SLOTS) {
            continue;
        }
        let mut all = true;
        for i in 0..SLOTS {
            if names.of_object(u32(&head, 4 * i)?)?.as_deref() != Some(pokemon::PARAM_CLASS) {
                all = false;
                break;
            }
        }
        if all {
            if !saved.is_empty() {
                let mut constants: Vec<i64> = Vec::new();
                for i in 0..count {
                    constants.push(u32(
                        &memory.read(u32(&head, 4 * i)? + pokemon::RECORD_IN_PARAM, 4)?,
                        0,
                    )?);
                }
                if constants == saved {
                    return Ok(Some(at));
                }
            }
            found.push(at);
        }
    }
    Ok(found.first().copied())
}

/// The encryption constants of the save's party's members, in order.
pub fn saved_party(memory: &dyn Memory, party: i64) -> Result<Vec<i64>> {
    let count = memory.read(party + RECORDS + SAVED * SLOTS, 1)?[0] as i64;
    let mut out = Vec::new();
    for i in 0..std::cmp::min(count, SLOTS) {
        out.push(u32(&memory.read(party + RECORDS + SAVED * i, 4)?, 0)?);
    }
    Ok(out)
}

/// Names a picture of the Bag's party by the species of the member whose icon it draws.
pub struct PartyIcons {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    species: Vec<String>,
    running: Rc<dyn Fn() -> Option<String>>,
    /// The party's list, as last found: the Switch screen names its buttons by it.
    pub party: Option<i64>,
    battle_box: Option<i64>,
    icons: PySet,
    /// Image -> the species it draws, until the next sweep: screen text asks of every
    /// picture at every poll, and naming one reads the party member's record.
    named: RefCell<IndexMap<i64, Option<String>>>,
}

impl PartyIcons {
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        species: Vec<String>,
        running: Rc<dyn Fn() -> Option<String>>,
    ) -> Self {
        PartyIcons {
            memory,
            names,
            species,
            running,
            party: None,
            battle_box: None,
            icons: PySet::new(),
            named: RefCell::new(IndexMap::new()),
        }
    }

    /// What a sweep found: the PokemonParams, to find the party by, the PokeIcons, and the
    /// save's party and Battle Box.
    pub fn track(
        &mut self,
        params: &[i64],
        icons: impl IntoIterator<Item = i64>,
        holders: &dyn Fn(&[i64]) -> Result<Vec<i64>>,
        adding: bool,
        saves: impl IntoIterator<Item = i64>,
    ) -> Result<()> {
        self.icons = if adding {
            self.icons.union(&PySet::of(icons))
        } else {
            PySet::of(icons)
        };
        self.named.borrow_mut().clear();
        let mut saved: Vec<i64> = Vec::new();
        for save in saves {
            let kind = self.names.of_object(save)?;
            if kind.as_deref() == Some(PARTY_SAVE) {
                saved = saved_party(&*self.memory, save)?;
            } else if kind.as_deref() == Some(BATTLE_BOX) {
                self.battle_box = Some(save);
            }
        }
        if !params.is_empty() && (self.party.is_none() || !adding) {
            let found = party_of(holders(params)?, &*self.memory, &self.names, &saved)?;
            self.party = match found {
                Some(party) if party != 0 => Some(party),
                _ => self.party,
            };
        }
        Ok(())
    }

    /// The party's list, as last found.
    pub fn party(&self) -> Option<i64> {
        self.party
    }

    /// The save's Battle Box, as last found.
    pub fn battle_box(&self) -> Option<i64> {
        self.battle_box
    }

    /// The species of the party member a picture draws the icon of, or None.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        if (self.running)().as_deref().and_then(first).is_none() || self.species.is_empty() {
            return Ok(None);
        }
        if self.party.is_none() {
            return Ok(None);
        }
        let memory = &self.memory;
        let material = u32(&memory.read(picture + lyt::PICTURE_MATERIAL, 4)?, 0)?;
        if material == 0 {
            return Ok(None);
        }
        let image = u32(&memory.read(material + lyt::MATERIAL_TEXTURE, 4)?, 0)?;
        if !self.named.borrow().contains_key(&image) {
            let drawn = self.species_drawn(image)?;
            self.named.borrow_mut().insert(image, drawn);
        }
        Ok(self.named.borrow()[&image].clone())
    }

    /// The species of the party member whose icon is drawn from this image, or None.
    fn species_drawn(&self, image: i64) -> Result<Option<String>> {
        let memory = &self.memory;
        let Some(party_at) = self.party else {
            return Ok(None);
        };
        for icon in self.icons.iter() {
            if self.names.of_object(icon)?.as_deref() != Some(ICON_CLASS) {
                continue;
            }
            let replace = u32(&memory.read(icon + d::REPLACE, 4)?, 0)?;
            if replace == 0 || self.names.of_object(replace)?.as_deref() != Some(d::REPLACE_CLASS) {
                continue;
            }
            let head = memory.read(replace + d::TEXTURES, 8)?;
            let (listed, count) = (u32(&head, 0)?, u32(&head, 4)?);
            if listed == 0 || !(0 < count && count <= d::MAX_TEXTURES) {
                continue;
            }
            let table = memory.read(listed, 4 * count)?;
            let mut slot = None;
            for i in 0..count {
                if u32(&memory.read(u32(&table, 4 * i)? + d::IMAGE, 4)?, 0)? == image {
                    slot = Some(i);
                    break;
                }
            }
            let Some(mut slot) = slot else {
                continue;
            };
            let first = first((self.running)().as_deref().unwrap_or("")).unwrap_or(0);
            if slot < first {
                return self.boxed(slot);
            }
            slot -= first;
            let party = memory.read(party_at, COUNT + 4)?;
            // The list found in an earlier Bag may be another object's now, counting past six.
            if !(0 <= slot && slot < std::cmp::min(party[COUNT as usize] as i64, SLOTS)) {
                return Ok(None);
            }
            let found = pokemon::read(
                &**memory,
                &self.names,
                u32(&party, 4 * slot)?,
                pokemon::SPECIES_MAX,
            )?;
            let Some(found) = found else {
                return Ok(None);
            };
            if found.species >= self.species.len() as i64 {
                return Ok(None);
            }
            return Ok(
                Some(self.species[found.species as usize].clone()).filter(|name| !name.is_empty())
            );
        }
        Ok(None)
    }

    /// The species of the Battle Box's member in this slot, or None.
    fn boxed(&self, slot: i64) -> Result<Option<String>> {
        let Some(r#box) = self.battle_box else {
            return Ok(None);
        };
        if self.names.of_object(r#box)?.as_deref() != Some(BATTLE_BOX) {
            return Ok(None);
        }
        let found = pokemon::stored(
            &*self.memory,
            r#box + RECORDS + pokemon::RECORD_SIZE * slot,
            pokemon::SPECIES_MAX,
        )?;
        let Some(found) = found else {
            return Ok(None);
        };
        if found.species >= self.species.len() as i64 {
            return Ok(None);
        }
        Ok(Some(self.species[found.species as usize].clone()).filter(|name| !name.is_empty()))
    }
}
