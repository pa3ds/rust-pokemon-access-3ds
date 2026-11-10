//! The Pokémon Omega Ruby and Alpha Sapphire's save keeps for its credits.
//!
//! The credits draw the journey a picture at a time, and the Pokémon in the pictures are the
//! player's own: the starter Birch was saved with, the one that beat each Gym, the one that beat
//! Wally on Victory Road and Steven. The save records each as the story reaches it, an entry a moment
//! in `savedata::SubEventSaveData` (the save file's block at 0x1e800, which the object keeps whole
//! from +4), from +0x2fc: 8 bytes, a u32 the Pokémon's own id, a u16 its species in bits 0 to 9 with
//! its form in 10 to 14 (1 in one save's Mega Latios and Primal Groudon) and a female bit 15, a u8
//! of flags; all zero for a moment the story has not reached, or one the player's game never
//! records. That save, Birch's rescue to the Champion, filled 20 of the first 26 (spike 222).
//!
//! The id is the Pokémon's encryption constant, the key every stored Pokémon opens with
//! (`games.common.pokemon`): each of that save's moments named one of its party by it, or the
//! Masquerain in its first box, and May's Mudkip none of them. So the nickname the player gave a
//! moment's Pokémon is found by its key among the party's `PokemonParam`s and the PC's boxes,
//! `savedata::BoxPokemon`, which keeps the save file's block at 0x33000 from +4: 31 boxes of 30
//! records. One traded away or released is in neither.
//!
//! Python's `Memories` keeps what a sweep found in plain attributes; here they are in `Cell` and
//! `RefCell`, so the profile can hold it in an `Rc` beside the scenes that ask it.
//!
//! Bound by:
//! - ADR-0038: the credits' Pokémon are the save's, read as each beat is said.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::core::binary::{u16, u32};
use crate::core::discovery::Found;
use crate::core::memory::Memory;
use crate::games::common::pokemon;
use crate::games::gen6::game::MemoriesReader;
use crate::py::{self, PySet, Result};

pub const CLASS: &str = "N8savedata16SubEventSaveDataE";
pub const BOXES: &str = "N8savedata10BoxPokemonE";
pub const BOX_RECORDS: i64 = 31 * 30;
pub const BOXES_AT: i64 = 4;
/// Where the record begins in the object, the size of an entry, and how many entries are read.
pub const RECORD: i64 = 0x2FC;
pub const ENTRY: i64 = 8;
pub const MOMENTS: i64 = 32;
pub const SPECIES: i64 = 0x3FF;
pub const FORM_SHIFT: i64 = 10;
pub const FORM: i64 = 0x1F;

/// The save's Pokémon for each of the credits' moments, read as each is asked for.
pub struct Memories {
    memory: Rc<dyn Memory>,
    save: Cell<Option<i64>>,
    boxes: Cell<Option<i64>>,
    /// Every PokemonParam swept, the party's among them.
    params: RefCell<PySet>,
}

impl Memories {
    pub fn new(memory: Rc<dyn Memory>) -> Self {
        Memories {
            memory,
            save: Cell::new(None),
            boxes: Cell::new(None),
            params: RefCell::new(PySet::new()),
        }
    }

    /// `Memories` as `Game.memories` builds it.
    pub fn of(memory: Rc<dyn Memory>) -> Rc<dyn MemoriesReader> {
        Rc::new(Memories::new(memory))
    }

    /// A moment's key and its species, form and sex, or None where it keeps none.
    fn entry(&self, moment: i64) -> Result<Option<(i64, i64)>> {
        let Some(save) = self.save.get() else {
            return Ok(None);
        };
        if !(0..MOMENTS).contains(&moment) {
            return Ok(None);
        }
        let read = self.memory.read(save + RECORD + moment * ENTRY, 6)?;
        let (key, packed) = (u32(&read, 0)?, u16(&read, 4)?);
        Ok(if packed & SPECIES != 0 {
            Some((key, packed))
        } else {
            None
        })
    }

    /// Each Pokémon the player keeps, as its record: the party's, then the boxes'.
    ///
    /// Python yields them one at a time and stops reading at the one asked for; this reads each
    /// only as `nickname` comes to it, through `each`, which says whether to go on.
    fn records(&self, each: &mut dyn FnMut(&[u8]) -> Result<bool>) -> Result<()> {
        let params: Vec<i64> = self.params.borrow().iter().collect();
        for param in params {
            let record = self
                .memory
                .read(param + pokemon::RECORD_IN_PARAM, pokemon::RECORD_SIZE)?;
            if !each(&record)? {
                return Ok(());
            }
        }
        if let Some(boxes) = self.boxes.get() {
            let boxes = self
                .memory
                .read(boxes + BOXES_AT, BOX_RECORDS * pokemon::RECORD_SIZE)?;
            let mut at = 0;
            while at < boxes.len() as i64 {
                if !each(py::slice(&boxes, at, at + pokemon::RECORD_SIZE))? {
                    return Ok(());
                }
                at += pokemon::RECORD_SIZE;
            }
        }
        Ok(())
    }
}

impl MemoriesReader for Memories {
    fn classes(&self) -> Vec<String> {
        vec![
            CLASS.to_string(),
            BOXES.to_string(),
            pokemon::PARAM_CLASS.to_string(),
        ]
    }

    /// Keep the save's objects a sweep found, which live all session, so a sweep that found
    /// none changes nothing; and the PokemonParams, which a partial sweep adds to.
    fn track(&self, found: &Found) {
        if let Some(&save) = found.of(&[CLASS]).first() {
            self.save.set(Some(save));
        }
        if let Some(&boxes) = found.of(&[BOXES]).first() {
            self.boxes.set(Some(boxes));
        }
        let params = PySet::of(found.of(&[pokemon::PARAM_CLASS]));
        let kept = if found.partial {
            self.params.borrow().union(&params)
        } else {
            params
        };
        *self.params.borrow_mut() = kept;
    }

    /// The national number and the form of the Pokémon the save keeps for a moment, or None
    /// where it keeps none or the save's object has not been found.
    fn pokemon(&self, moment: i64) -> Result<Option<(i64, i64)>> {
        Ok(self
            .entry(moment)?
            .map(|(_, packed)| (packed & SPECIES, packed >> FORM_SHIFT & FORM)))
    }

    /// The nickname the player gave the Pokémon of a moment, or None where they gave it
    /// none, no longer have it, or the moment keeps none.
    fn nickname(&self, moment: i64) -> Result<Option<String>> {
        let Some((key, _)) = self.entry(moment)? else {
            return Ok(None);
        };
        let mut said: Option<String> = None;
        self.records(&mut |record| {
            if u32(record, 0)? == key
                && let Some(plain) = pokemon::decrypt(record)
            {
                said = match pokemon::parse(&plain, pokemon::SPECIES_MAX)? {
                    Some(found) if found.nicknamed && !found.nickname.is_empty() => {
                        Some(found.nickname)
                    }
                    _ => None,
                };
                return Ok(false);
            }
            Ok(true)
        })?;
        Ok(said)
    }
}
