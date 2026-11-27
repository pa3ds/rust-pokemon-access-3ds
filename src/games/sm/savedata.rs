//! The save's trainer record in Sun, `Savedata::MyStatus`: which language the game is in, reached
//! as the game reaches it, from its GameManager.
//!
//! Gen 6's record is found by a sweep (`gen6.mystatus`). Sun's is inside the save, one large block
//! no sweep reads (ADR-0035), so it is followed down the game's own pointers from the
//! GameManager's singleton, whose address the executable exports by name in the ROM's `static.crs`
//! (`pa3ds.core.cro`), and from nothing else:
//!
//! ```text
//! gfl2::base::Singleton<GameSys::GameManager>::m_instance   -> the GameManager
//! GameManager  +0x24 -> the game's data      +0x04 -> the save      +0xea0 its MyStatus
//!                                                                (+0xee8 in Ultra Sun)
//! MyStatus     +0x00 its vtable              +0x3d u8 the language (gen 6's numbers)
//! ```
//!
//! The chain is the one 68 of Sun's functions take in its decompile (`GetNexUniqueID`, `GetCountry`
//! and others are called on `*(*(GameManager + 0x24) + 4) + 0xea0`). The language's offset is read
//! from the game's own `MyStatus::GetPokeLanguageId()`, which the CRS also names: its first
//! instruction is `ldrb r0, [r0, #0x3d]`. Before anything else of it is read the record's vtable is
//! checked to be `Savedata::MyStatus`'s, so a chain that leads elsewhere reads nothing more.
//!
//! Bound by:
//! - ADR-0035: as amended for the save, reads to the byte, along the game's own pointers, of an
//!   object its RTTI names.
//! - ADR-0040: the language is the save's.

use std::cell::Cell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::gen6::mystatus::LANGUAGES;
use crate::games::sm::VersionLayout;
use crate::py::{Kind, Result};

pub const INSTANCE: &str = "_ZN4gfl24base9SingletonIN7GameSys11GameManagerEE10m_instanceE";
pub const LANGUAGE_GETTER: &str = "_ZNK8Savedata8MyStatus17GetPokeLanguageIdEv";
pub const CLASS: &str = "N8Savedata8MyStatusE";
pub const GAME_DATA: i64 = 0x24;
pub const SAVE: i64 = 0x04;
/// `ldrb r0, [r0, #imm12]`, the getter's first instruction, with its offset in the low 12 bits.
pub const LDRB_R0_R0: i64 = 0xE5D00000;

/// The language the save says, or None where the chain or the record is not there yet.
pub struct SaveLanguage {
    memory: Rc<dyn Memory>,
    my_status: i64,
    names: Rc<ClassNames>,
    instance: Option<i64>,
    getter: Option<i64>,
    offset: Cell<Option<i64>>,
}

impl SaveLanguage {
    /// `memory` reads exactly what it is asked for; `exports` is the CRS's (`cro.exports`);
    /// `layout` says where the version's save keeps its record (`SUN_LAYOUT` in Python's
    /// default).
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        exports: &IndexMap<String, i64>,
        layout: VersionLayout,
    ) -> SaveLanguage {
        SaveLanguage {
            memory,
            my_status: layout.my_status,
            names,
            instance: exports.get(INSTANCE).copied(),
            getter: exports.get(LANGUAGE_GETTER).copied(),
            offset: Cell::new(None),
        }
    }

    pub fn language(&self) -> Result<Option<String>> {
        let (Some(instance), Some(getter)) = (self.instance, self.getter) else {
            return Ok(None);
        };
        match self.read(instance, getter) {
            Err(error) if error.is(Kind::RpcError) => Ok(None),
            other => other,
        }
    }

    fn read(&self, instance: i64, getter: i64) -> Result<Option<String>> {
        let offset = match self.offset.get() {
            Some(offset) => offset,
            None => {
                let code = self.word(getter)?;
                let Some(code) = code.filter(|code| code & 0xFFFFF000 == LDRB_R0_R0) else {
                    return Ok(None);
                };
                self.offset.set(Some(code & 0xFFF));
                code & 0xFFF
            }
        };
        let mut at = self.word(instance)?;
        for step in [GAME_DATA, SAVE] {
            at = match at {
                None | Some(0) => None,
                Some(at) => self.word(at + step)?,
            };
        }
        let Some(at) = at.filter(|&at| at != 0) else {
            return Ok(None);
        };
        let record = at + self.my_status;
        if self.names.of_object(record)?.as_deref() != Some(CLASS) {
            return Ok(None);
        }
        let byte = self.memory.read(record + offset, 1)?;
        Ok(match byte.first() {
            None => None,
            Some(&byte) => LANGUAGES
                .iter()
                .find(|(number, _)| *number == byte)
                .map(|(_, code)| code.to_string()),
        })
    }

    fn word(&self, address: i64) -> Result<Option<i64>> {
        let data = self.memory.read(address, 4)?;
        Ok(if data.len() == 4 {
            Some(u32(&data, 0)?)
        } else {
            None
        })
    }
}
