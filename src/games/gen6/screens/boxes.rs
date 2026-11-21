//! Who is in each of the PC Box's slots, said while the player carries a Pokémon over them
//! (or_organize_boxes).
//!
//! While a Pokémon is carried the top screen's summary is the carried one's, so a slot holding
//! another said only where it was (`common.slots`). The box's icons say no species; the save does,
//! and the game writes a box's and the Battle Box's slots the moment a Pokémon is dropped or two
//! are swapped. The save's party, `savedata::PokePartySave`, is written only as the Box closes:
//! swapping Pikachu into the party for Rayquaza left it holding Rayquaza, so the party is the
//! game's own list of it (`partyicons`). Read in Omega Ruby (2026-10-24):
//!
//! ```text
//! BoxPokemon  +0x04  31 boxes of 30 slots, a record 0xE8 bytes each, in reading order
//! BOX         +0x04  the boxes' 31 names, 0x22 bytes of UTF-16 each, ended by a 0
//! BattleBox   +0x04  six records
//! ```
//!
//! A box is the one the Box's work block names, a byte beside its mode (`Game.box_shown`): from 0,
//! and 0xFF on the Battle Box, written as the grid starts to turn, about 0.2 s before the title
//! above it (spike B, or_pc_organize). Where that byte is not measured, it is the box whose name is
//! the title, which the player can rename.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::pictures::TEXTURE;
use crate::games::common::{pokemon, slots};
use crate::games::gen6::screens::partyicons;
use crate::py::{self, Result};

pub const BOXES: &str = "N8savedata10BoxPokemonE";
pub const NAMES: &str = "N8savedata3BOXE";
pub const COUNT: i64 = 31;
pub const SLOTS: i64 = 30;
pub const NAME: i64 = 0x22;
pub const APP: &str = "N3app3box4ProcE";
/// The Box's modes as its tabs draw them (`pictures.TEXTURE`): normal, swap and multi-select.
pub const MODE_WORDS: [&str; 3] = ["mode01", "mode02", "mode03"];

/// A byte of the Box's work block: `where` is the pointer to it in the process, and the byte.
pub fn work_byte(memory: &dyn Memory, r#where: (i64, i64), process: i64) -> Result<i64> {
    let (pointer, byte) = r#where;
    let at = u32(&memory.read(process + pointer, 4)?, 0)? + byte;
    Ok(*memory
        .read(at, 1)?
        .first()
        .ok_or_else(|| py::Error::index("index out of range"))? as i64)
}

/// `Callable[[], int | None]`: the box the grid shows. It reads the game, so it can raise.
pub type Shown = Rc<dyn Fn() -> Result<Option<i64>>>;

/// The box the grid shows, from 0: None in the Battle Box, out of the Box, or unmeasured.
pub fn shown(
    memory: Rc<dyn Memory>,
    r#where: Option<(i64, i64)>,
    process: Rc<dyn Fn() -> Option<i64>>,
) -> Shown {
    Rc::new(move || {
        let running = process();
        let (Some(r#where), Some(running)) = (r#where, running) else {
            return Ok(None);
        };
        let number = work_byte(&*memory, r#where, running)?;
        Ok((number < COUNT).then_some(number))
    })
}

/// Says the PC Box's mode as Start and Select change it, and on opening where it is not
/// normal: its three tabs only pulse as one is chosen and draw nothing to tell them apart
/// (or_pc_mode_buttons, spike 288).
pub struct Mode {
    memory: Rc<dyn Memory>,
    r#where: (i64, i64),
    process: Rc<dyn Fn() -> Option<i64>>,
    last: Option<i64>,
    held: Option<i64>,
}

impl Mode {
    pub fn new(
        memory: Rc<dyn Memory>,
        r#where: (i64, i64),
        process: Rc<dyn Fn() -> Option<i64>>,
    ) -> Self {
        Mode {
            memory,
            r#where,
            process,
            last: None,
            held: None,
        }
    }

    /// The mode in words, once a change has held for a poll: the Box zeroes it as it closes,
    /// a poll before its process goes, and "normal mode" was said on leaving it in swap mode
    /// (or_touch_pc_party_summary).
    pub fn poll(&mut self) -> Result<Option<String>> {
        let process = (self.process)();
        let Some(process) = process else {
            self.last = None;
            self.held = None;
            return Ok(None);
        };
        let mode = work_byte(&*self.memory, self.r#where, process)?;
        if Some(mode) == self.last || mode >= MODE_WORDS.len() as i64 {
            self.held = None;
            return Ok(None);
        }
        if Some(mode) != self.held {
            self.held = Some(mode);
            return Ok(None);
        }
        let first = self.last.is_none();
        self.last = Some(mode);
        self.held = None;
        Ok(if first && mode == 0 {
            None
        } else {
            Some(own(&format!("{TEXTURE}{}", MODE_WORDS[mode as usize]), &[]))
        })
    }
}

/// Names the Pokémon in a slot of a box, the Battle Box or the party (`slots.Occupant`).
pub struct Occupants {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    party: Rc<dyn Fn() -> Option<i64>>,
    battle_box: Rc<dyn Fn() -> Option<i64>>,
    shown: Shown,
    /// Class -> the save's object of it, as last found.
    saves: IndexMap<String, i64>,
}

impl Occupants {
    /// `Occupants(memory, names, party, battle_box)`: `shown` left to its default, no box.
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        party: Rc<dyn Fn() -> Option<i64>>,
        battle_box: Rc<dyn Fn() -> Option<i64>>,
    ) -> Self {
        Occupants::with(memory, names, party, battle_box, Rc::new(|| Ok(None)))
    }

    pub fn with(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        party: Rc<dyn Fn() -> Option<i64>>,
        battle_box: Rc<dyn Fn() -> Option<i64>>,
        shown: Shown,
    ) -> Self {
        Occupants {
            memory,
            names,
            party,
            battle_box,
            shown,
            saves: IndexMap::new(),
        }
    }

    /// The save's boxes and their names, among what a sweep found by class.
    pub fn track(&mut self, found: &IndexMap<i64, String>) {
        for (&at, name) in found {
            if name == BOXES || name == NAMES {
                self.saves.insert(name.clone(), at);
            }
        }
    }

    /// Python's `__call__`: the name of the Pokémon in a slot (`slots.Held`), or None.
    pub fn call(&self, kind: &str, title: Option<&str>, index: i64) -> Result<Option<String>> {
        let memory = &self.memory;
        let mut record: Option<i64> = None;
        if kind == slots::PARTY {
            let party = (self.party)();
            let Some(party) = party else {
                return Ok(None);
            };
            let count = *memory
                .read(party, partyicons::COUNT + 1)?
                .last()
                .ok_or_else(|| py::Error::index("index out of range"))?
                as i64;
            if !(0 <= index && index < count) {
                return Ok(None);
            }
            let found = pokemon::read(
                &**memory,
                &self.names,
                u32(&memory.read(party + 4 * index, 4)?, 0)?,
                pokemon::SPECIES_MAX,
            )?;
            return Ok(found.map(|found| found.nickname));
        }
        if kind == slots::TITLED_LIST && index < partyicons::SLOTS {
            record = self.valid((self.battle_box)(), partyicons::BATTLE_BOX)?;
        } else if kind == slots::TITLED_GRID
            && index < SLOTS
            && let Some(r#box) = self.r#box(title)?
        {
            record = self.valid(self.saves.get(BOXES).copied(), BOXES)?;
            record = record.map(|record| record + pokemon::RECORD_SIZE * SLOTS * r#box);
        }
        let Some(record) = record else {
            return Ok(None);
        };
        let found = pokemon::stored(
            &**memory,
            record + partyicons::RECORDS + pokemon::RECORD_SIZE * index,
            pokemon::SPECIES_MAX,
        )?;
        Ok(found.map(|found| found.nickname))
    }

    /// The box the Box's byte says is shown; unmeasured, the one whose name is this title, where
    /// only one is.
    fn r#box(&self, title: Option<&str>) -> Result<Option<i64>> {
        if let Some(r#box) = (self.shown)()? {
            return Ok(Some(r#box));
        }
        let names = self.valid(self.saves.get(NAMES).copied(), NAMES)?;
        let (Some(names), Some(title)) = (names, title.filter(|title| !title.is_empty())) else {
            return Ok(None);
        };
        let raw = self
            .memory
            .read(names + partyicons::RECORDS, NAME * COUNT)?;
        let mut boxes: Vec<i64> = Vec::new();
        for i in 0..COUNT {
            let name = py::decode_utf16le(py::slice(&raw, NAME * i, NAME * (i + 1)));
            if name.split('\0').next() == Some(title) {
                boxes.push(i);
            }
        }
        Ok(if boxes.len() == 1 {
            Some(boxes[0])
        } else {
            None
        })
    }

    /// The save's object, while it is still one of its class.
    fn valid(&self, save: Option<i64>, kind: &str) -> Result<Option<i64>> {
        let Some(save) = save else {
            return Ok(None);
        };
        Ok((self.names.of_object(save)?.as_deref() == Some(kind)).then_some(save))
    }
}
