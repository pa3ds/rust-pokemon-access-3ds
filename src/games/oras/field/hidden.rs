//! The DexNav's hidden Pokémon: the one whose tail shows above the tall grass, rustling, to be
//! sneaked up on. Spike 80 took the game's "kakuremino" classes, a hiding cloak, for its mechanism;
//! they are the move models' event types for trainers (`field::mmodel::EvTypeTrKakuremino` beside
//! `EvTypeTrMirror` and `EvTypeTrDiving`), alike with a tail up or not, and none points at the
//! record (spike 135).
//!
//! It is not a move model: the pool holds nothing for it while it is up (spike 80). The game keeps
//! one record for it, without a class of its own, which reads zero while nothing is hiding:
//!
//! ```text
//! +0x00 ptr  the record's own `gfl::heap::CTRExpHeap`, whatever the record holds
//! +0x04 u16  species, the national number (0x105 Poochyena, 0x11b Surskit); 0 while empty
//! +0x06 u16  not understood: 0 in the first finds, 0x97 beside a Wurmple and 0x5e beside a
//!            Surskit later on Route 102, where reading the species as a word hid them both
//! +0x08 u8   level; +0x0b read 3 once, for the fourth Zigzagoon of a chain (movie27, 629 s),
//!            and 0 at every other find, so the four bytes are no one word
//! +0x34 u32  a counter stepping 0 to 19 as the player walks through grass, reset by a find
//! +0x38 u32  0 for a search's find, 22 to 50 for a detection's: a countdown, perhaps
//! +0x48 u32  1 once a record has been set; a battle does not clear it
//! +0x8c f32  world x, +0x90 y (2.0 on grass), +0x94 z, the cell's centre at `cell * 18 + 9`
//! +0x98 u32  1 once a record has been set, as +0x48
//! +0xa0 ptr  the party's records while the tail is up, cleared by its battle
//! +0xa4 ptr  a `pml::pokepara::CoreParam`, the record's Pokémon, held even while empty
//! +0xae u8   the DexNav's state: below 0xd while a Pokémon is up, 1, 4, 5 and 7 found as the
//!            player walks, 8 found by a search, 0xa near enough that its details show; 0xd
//!            the DexNav's page of a Pokémon with none hiding, as after one has fainted, 0xe a
//!            search under way, 0x12 spotted and running, 0xff while the record is empty,
//!            after running from a battle, and while a zone change has left it stale (spike 80,
//!            movies 26 and 27)
//! +0xc9 u8   how it was found: 3 by a search, 1 by the detector with the DexNav open, 2 with
//!            it closed (spike 194: three searches and seven detections over movies 4, 26, 27)
//! ```
//!
//! The record is found through the one `field::PlayerWork`, which holds its address at +0x6c, told
//! by the heap at its +0x00 (spike 135: all three captures of movie3, before the lesson, with its
//! Poochyena's tail up and after its battle). It was found as what holds a CoreParam at +0xa4 until
//! then, searching every page for every CoreParam at every whole sweep. Its address was the same in
//! seven captures over three movies, and it is the same record for the lesson's Poochyena on Route
//! 101 and a Surskit found on Route 102.
//!
//! A battle leaves the species, level and place in the record (the user, 2026-09-22: the entries
//! did not clean up after a battle), and so does walking to another zone; the byte at +0xae is what
//! says the tail is up, in the nine captures over four movies it was read in. A Pokémon that faints
//! leaves it 0xd, not 0xff, and was listed "here" until the next search (movie26).
//!
//! The species is said in the log and not aloud: the game shows a tail, and the DexNav names the
//! Pokémon only once the species is registered, so a name would be an Extra (ADR-0017), and the
//! names are in the ROM's text in any case (names later).
//!
//! Bound by:
//! - ADR-0037: a hidden Pokémon is up while its state is below 0xd, and +0xc9 says a search's find
//!   from the detector's.

use std::cell::{Cell as Slot, RefCell};
use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;

use crate::core::binary::{f32, u16, u32};
use crate::core::discovery::Found;
use crate::core::memory::Memory;
use crate::core::pathfind::Cell;
use crate::games::common::field::things::{Interactable, POKEMON};
use crate::games::gen6::field::features::{Feature, HiddenReader, Hiding, Tracked, ZoneAt};
use crate::games::gen6::gflheap;
use crate::py::{self, Result};

/// What the record holds at +0xa4, which a sweep no longer looks for.
pub const CORE_PARAM_CLASS: &str = "N3pml8pokepara9CoreParamE";
pub const HEAP_CLASS: &str = gflheap::CLASS;
pub const PLAYER_WORK_CLASS: &str = "N5field10PlayerWorkE";
/// Where the player's work holds the record's address.
pub const WORK_RECORD: i64 = 0x6C;
/// What a sweep looks for, to find the record by.
pub const CLASSES: [&str; 2] = [PLAYER_WORK_CLASS, HEAP_CLASS];

pub const HEAP: i64 = 0x00;
pub const SPECIES: i64 = 0x04;
pub const LEVEL: i64 = 0x08;
pub const SET: i64 = 0x48;
pub const WORLD_X: i64 = 0x8C;
pub const WORLD_Z: i64 = 0x94;
pub const CORE: i64 = 0xA4;
/// The DexNav's state, named for a guess at its meaning before movie26 showed it to be one.
pub const SLOT: i64 = 0xAE;
pub const NONE: i64 = 0xFF;
/// The states from here up hold no Pokémon: 0xd its page with none hiding, 0xe a search, 0x12
/// spotted, `NONE`. Every state seen with one up was below it.
pub const GONE: i64 = 0x0D;
pub const FOUND_BY: i64 = 0xC9;
/// `FOUND_BY` for a Pokémon the DexNav's search found: the one the player asked for, where its
/// detector finds any (movie27: a Surskit came up while he searched for Zigzagoon).
pub const SEARCH: i64 = 3;
/// `FOUND_BY` for one its detector found with the DexNav open, as against 2 with the AreaNav
/// open, the player walking about with the map up rather than looking for Pokémon.
pub const ON_DEXNAV: i64 = 1;
/// Enough of the record to hold everything above.
pub const SIZE: i64 = 0xCC;

/// World units per cell, the centre at `cell * 18 + 9`.
pub const CELL: i64 = 18;
/// The national Pokédex's last number in this generation.
pub const MAX_SPECIES: i64 = 721;
pub const MAX_LEVEL: i64 = 100;

/// One hiding in the grass now.
#[derive(Debug, Clone, PartialEq)]
pub struct HiddenPokemon {
    pub species: i64,
    pub level: i64,
    pub cell: (i64, i64),
    pub searched: bool,
    /// Found by the detector with the DexNav open.
    pub on_dexnav: bool,
}

/// The record's address, or None when a sweep found nothing to reach it by: what the player's
/// work points at, holding one of the heaps the sweep found first.
pub fn find(found: &Found, memory: &dyn Memory) -> Result<Option<i64>> {
    let heaps: HashSet<i64> = found.of(&[HEAP_CLASS]).into_iter().collect();
    for work in found.of(&[PLAYER_WORK_CLASS]) {
        let record = u32(&memory.read(work + WORK_RECORD, 4)?, 0)?;
        if record != 0 && heaps.contains(&u32(&memory.read(record + HEAP, 4)?, 0)?) {
            return Ok(Some(record));
        }
    }
    Ok(None)
}

/// The record's words as they are, before any check, for the log: what the game held when a
/// rustle went unlisted.
#[derive(Debug, Clone, PartialEq)]
pub struct Words {
    pub species: i64,
    pub level: i64,
    pub slot: i64,
    pub cell: (i64, i64),
    pub found_by: i64,
}

impl fmt::Display for Words {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "species {} level {} slot {:#x} by {} at ({}, {})",
            self.species, self.level, self.slot, self.found_by, self.cell.0, self.cell.1
        )
    }
}

pub fn words(memory: &dyn Memory, record: i64) -> Result<Words> {
    let head = memory.read(record, SIZE)?;
    Ok(Words {
        species: u16(&head, SPECIES)?,
        level: head[LEVEL as usize] as i64,
        slot: head[SLOT as usize] as i64,
        cell: (cell(f32(&head, WORLD_X)?)?, cell(f32(&head, WORLD_Z)?)?),
        found_by: head[FOUND_BY as usize] as i64,
    })
}

/// What the record holds, or None while no tail is up, or the bytes are not a record's: a
/// species out of the Pokédex or a level over 100.
pub fn read(memory: &dyn Memory, record: i64) -> Result<Option<HiddenPokemon>> {
    let held = words(memory, record)?;
    if !(1 <= held.species && held.species <= MAX_SPECIES)
        || !(1 <= held.level && held.level <= MAX_LEVEL)
    {
        return Ok(None);
    }
    if held.slot >= GONE {
        return Ok(None);
    }
    Ok(Some(HiddenPokemon {
        species: held.species,
        level: held.level,
        cell: held.cell,
        searched: held.found_by == SEARCH,
        on_dexnav: held.found_by == ON_DEXNAV,
    }))
}

/// What a DexNav find is called: the game shows a tail and never names it (the user,
/// 2026-09-22, asking for the shaking-tail Pokémon under a category of its own).
pub const HIDDEN_POKEMON: &str = "hidden Pokémon";

/// The record, for the gen 6 field to list (`gen6.field.HiddenReader`): found by a sweep and
/// kept, since it lives all session.
pub struct DexNav {
    memory: Rc<dyn Memory>,
    record: Slot<Option<i64>>,
    /// The hidden Pokémon `listed` last listed, which says how the DexNav found it.
    last: RefCell<Option<Hiding>>,
}

impl DexNav {
    pub fn new(memory: Rc<dyn Memory>) -> Self {
        DexNav {
            memory,
            record: Slot::new(None),
            last: RefCell::new(None),
        }
    }

    pub fn classes(&self) -> Vec<&'static str> {
        CLASSES.to_vec()
    }

    pub fn track(&self, found: &Found) -> Result<()> {
        self.record
            .set(find(found, &*self.memory)?.or(self.record.get()));
        Ok(())
    }

    pub fn read(&self) -> Result<Option<Hiding>> {
        let Some(record) = self.record.get() else {
            return Ok(None);
        };
        let held = read(&*self.memory, record)?;
        Ok(held.map(|held| Hiding {
            record,
            species: held.species,
            cell: held.cell,
            searched: held.searched,
            on_dexnav: held.on_dexnav,
        }))
    }

    /// The hidden Pokémon, while one is up in `zone`: one read of its record, cheap enough for
    /// every poll, since it comes and goes between sweeps.
    ///
    /// The record outlives its zone: walking onto Route 102 listed Route 101's Poochyena at 105
    /// west until the next find replaced it (spike 80's replay). So one is listed only where the
    /// field map puts its cell in the player's own zone.
    pub fn listed(&self, zone: i64, zone_at: ZoneAt) -> Result<Vec<Interactable>> {
        let found = self.read()?;
        let last = match found {
            Some(found) if zone_at(found.cell)? == Some(zone) => Some(found),
            _ => None,
        };
        *self.last.borrow_mut() = last.clone();
        let Some(last) = last else {
            return Ok(Vec::new());
        };
        Ok(vec![Interactable {
            code: last.species,
            source: last.record,
            ..Interactable::new(POKEMON, HIDDEN_POKEMON, last.cell)
        }])
    }

    pub fn words(&self) -> Result<Option<(String, Cell)>> {
        let Some(record) = self.record.get() else {
            return Ok(None);
        };
        let held = words(&*self.memory, record)?;
        Ok(Some((held.to_string(), held.cell)))
    }
}

impl Tracked for DexNav {
    fn classes(&self) -> Vec<&'static str> {
        DexNav::classes(self)
    }

    fn track(&self, found: &Found) -> Result<()> {
        DexNav::track(self, found)
    }
}

impl Feature for DexNav {
    fn listed(&self, zone: i64, zone_at: ZoneAt) -> Result<Vec<Interactable>> {
        DexNav::listed(self, zone, zone_at)
    }
}

impl HiddenReader for DexNav {
    fn last(&self) -> Option<Hiding> {
        self.last.borrow().clone()
    }

    fn words(&self) -> Result<Option<(String, Cell)>> {
        DexNav::words(self)
    }
}

fn cell(units: f64) -> Result<i64> {
    py::round((units - CELL as f64 / 2.0) / CELL as f64)
}
