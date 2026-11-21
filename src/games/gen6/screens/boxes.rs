//! Who is in each of the PC Box's slots, said while the player carries a Pokémon over them
//! (or_organize_boxes), and the box it turns to, said from its own byte (`Titles`).
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
//! A box is the one the Box's work block names, a byte beside its mode (`Shape.shown`): from 0,
//! and 0xFF on the Battle Box, written as the grid starts to turn, about 0.2 s before the title
//! above it (spike 298, or_pc_organize). Where that byte is not measured, it is the box whose name
//! is the title, which the player can rename.
//!
//! Bound by:
//! - ADR-0043: the slot is said as the pointer sets off, the box by its byte as it turns.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::pictures::TEXTURE;
use crate::games::common::screen_announce::REPEAT_POLLS;
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

/// Where the Box's process keeps its mode and the box its grid shows, each the pointer to
/// its work block and the byte in it (`work_byte`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shape {
    /// The pointer, and the mode: 0 normal, 1 swap, 2 multi-select (`Mode`, spike 288).
    pub mode: (i64, i64),
    /// The pointer, and the box the grid shows (`shown`, spike 298).
    pub shown: (i64, i64),
}

/// Omega Ruby's and X's, the same.
pub const GEN6: Shape = Shape {
    mode: (0x24, 0x21),
    shown: (0x24, 0x24),
};

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

/// The box the grid shows, from 0, and 0xFF the Battle Box; None out of the Box or where the
/// byte is not measured.
pub fn shown(
    memory: Rc<dyn Memory>,
    r#where: Option<(i64, i64)>,
    process: Rc<dyn Fn() -> Option<i64>>,
) -> Shown {
    Rc::new(move || {
        let running = process();
        match (r#where, running) {
            (Some(r#where), Some(running)) => Ok(Some(work_byte(&*memory, r#where, running)?)),
            _ => Ok(None),
        }
    })
}

/// `Callable[[], list[str] | None]`: the boxes' names in the save (`Occupants.names`). It reads
/// the game, so it can raise.
pub type BoxNames = Rc<dyn Fn() -> Result<Option<Vec<String>>>>;

/// Says the box the PC Box turns to as its byte changes (`shown`), by the name the player gave
/// it in the save: the title above the grid is written a poll or more after the byte, and
/// holding left through the boxes left BOX 6 and BOX 5 unsaid and said BOX 4 again after BOX 3
/// (spike 298, or_pc_organize). The screen's words for a box are then no news (`unsaid`).
pub struct Titles {
    shown: Shown,
    names: BoxNames,
    /// The boxes' names, read again as the Box opens and at each turn: the player renames
    /// one from the Box itself.
    known: Option<Vec<String>>,
    last: Option<i64>,
    held: Option<i64>,
    polls: i64,
    /// The name last said, and the poll it was said at.
    said: Option<(String, i64)>,
}

impl Titles {
    pub fn new(shown: Shown, names: BoxNames) -> Self {
        Titles {
            shown,
            names,
            known: None,
            last: None,
            held: None,
            polls: 0,
            said: None,
        }
    }

    /// The name of the box just turned to, or None. Nothing on opening, where the cursor's
    /// first slot names the box; nothing for the Battle Box, whose title is the screen's to say;
    /// and a turn to BOX 1 only once it has held a poll, as the Box zeroes its work block as it
    /// closes (`Mode`).
    pub fn poll(&mut self) -> Result<Option<String>> {
        self.polls += 1;
        let byte = (self.shown)()?;
        let Some(byte) = byte else {
            self.last = None;
            self.held = None;
            self.known = None;
            return Ok(None);
        };
        if Some(byte) == self.last {
            self.held = None;
            return Ok(None);
        }
        if byte == 0 && self.last.is_some() && self.held != Some(0) {
            self.held = Some(0);
            return Ok(None);
        }
        let opening = self.last.is_none();
        self.last = Some(byte);
        self.held = None;
        self.known = (self.names)()?;
        if opening || byte >= COUNT {
            return Ok(None);
        }
        let Some(known) = self.known.as_ref().filter(|known| !known.is_empty()) else {
            return Ok(None);
        };
        let name = known
            .get(byte as usize)
            .ok_or_else(|| py::Error::index("list index out of range"))?
            .clone();
        if name.is_empty() {
            return Ok(None);
        }
        self.said = Some((name.clone(), self.polls));
        Ok(Some(name))
    }

    /// Has the byte turned to another box since the last poll? Asked every tick, one read or
    /// two, so that a screen poll comes at each turn: polled only every 0.2 s, holding left or
    /// right turned faster than that, and said BOX 2, BOX 6, BOX 7 (or_pc_box_bugs). Not a turn
    /// to BOX 1, which `poll` says only once it has held a poll.
    pub fn turned(&self) -> Result<bool> {
        let byte = (self.shown)()?;
        Ok(byte.is_some_and(|byte| Some(byte) != self.last && byte != 0))
    }

    /// The words for a box the screen should leave unsaid this poll: while the byte names
    /// a box, every other box's name, a title written late, and its own name for `REPEAT_POLLS`
    /// after it was said from the byte.
    pub fn unsaid(&self) -> Result<HashSet<String>> {
        let Some(last) = self.last.filter(|&last| last < COUNT) else {
            return Ok(HashSet::new());
        };
        let Some(known) = self.known.as_ref().filter(|known| !known.is_empty()) else {
            return Ok(HashSet::new());
        };
        let name = known
            .get(last as usize)
            .ok_or_else(|| py::Error::index("list index out of range"))?;
        let mut quiet: HashSet<String> = known
            .iter()
            .filter(|each| !each.is_empty() && *each != name)
            .cloned()
            .collect();
        if let Some((said, at)) = &self.said
            && said == name
            && self.polls - at <= REPEAT_POLLS
        {
            quiet.insert(name.clone());
        }
        Ok(quiet)
    }
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
        if let Some(r#box) = (self.shown)()?
            && r#box < COUNT
        {
            return Ok(Some(r#box));
        }
        let title = title.filter(|title| !title.is_empty());
        let names = match title {
            Some(_) => self.names()?,
            None => None,
        };
        let boxes: Vec<i64> = names
            .unwrap_or_default()
            .iter()
            .enumerate()
            .filter(|(_, name)| Some(name.as_str()) == title)
            .map(|(i, _)| i as i64)
            .collect();
        Ok(if boxes.len() == 1 {
            Some(boxes[0])
        } else {
            None
        })
    }

    /// The boxes' names in the save, as the player gave them; None before a sweep finds them.
    pub fn names(&self) -> Result<Option<Vec<String>>> {
        let names = self.valid(self.saves.get(NAMES).copied(), NAMES)?;
        let Some(names) = names else {
            return Ok(None);
        };
        let raw = self
            .memory
            .read(names + partyicons::RECORDS, NAME * COUNT)?;
        Ok(Some(
            (0..COUNT)
                .map(|i| {
                    let name = py::decode_utf16le(py::slice(&raw, NAME * i, NAME * (i + 1)));
                    name.split('\0').next().unwrap_or("").to_string()
                })
                .collect(),
        ))
    }

    /// The save's object, while it is still one of its class.
    fn valid(&self, save: Option<i64>, kind: &str) -> Result<Option<i64>> {
        let Some(save) = save else {
            return Ok(None);
        };
        Ok((self.names.of_object(save)?.as_deref() == Some(kind)).then_some(save))
    }
}
