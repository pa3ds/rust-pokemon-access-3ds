//! The types a summary's Pokémon has, which it draws as two icons, `Icon_SkillType` both, no word on
//! them, and gen 6 names by their textures, which Sun's layouts do not give pa3ds (spike 318).
//!
//! The summary's frame keeps a view whose +0x90 is the Pokémon shown (its decompile, Ultra Sun):
//!
//! ```text
//! App::Status::StatusFrame  +0x88 -> a view    view +0x90 -> its pml::pokepara::PokemonParam,
//!                           whose record is the party's, +0x04 pointing at it
//! ```
//!
//! and the species' types are the ROM's, its personal data (`a/0/1/7`), 0x54 bytes a species, the
//! two types at +0x06 and +0x07, a form past the first at the index +0x1c names, +0x20 the forms
//! counted: Rowlet's 11 and 2, Grass and Flying. Their names are system text file 107, Normal at 0.
//!
//! Bound by:
//! - ADR-0035: the frame and the Pokémon are small blocks a sweep found.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{u16, u32};
use crate::core::memory::Memory;
use crate::core::rom::Rom;
use crate::core::rtti::ClassNames;
use crate::games::common::garc::garc_files;
use crate::games::common::pokemon;
use crate::py::{Error, Result};

pub const FRAME: &str = "N3App6Status11StatusFrameE";
pub const VIEW: i64 = 0x88;
pub const PARAM: i64 = 0x90;
pub const PERSONAL: &str = "a/0/1/7";
pub const TYPE_1: i64 = 0x06;
pub const TYPE_2: i64 = 0x07;
pub const FORM_INDEX: i64 = 0x1C;
pub const FORMS: i64 = 0x20;
/// System text file 107, the types' names, Normal at 0 (Sun's numbering).
pub const TYPE_NAMES: i64 = 107;
pub const TYPE_COUNT: i64 = 18;
/// The Pokémon's type icons' name, both of them; the move panel's is IconType_00.
pub const ICON: &[u8] = b"Icon_SkillType";
pub const SPECIES_MAX: i64 = 809;

/// Each species' and form's two types, by the ROM's personal data.
#[derive(Debug, Clone, PartialEq)]
pub struct Types {
    pub entries: Vec<Vec<u8>>,
}

/// `entry[at]`, an IndexError past its end.
fn byte(entry: &[u8], at: i64) -> Result<i64> {
    match entry.get(at as usize) {
        Some(&value) => Ok(value as i64),
        None => Err(Error::index("index out of range")),
    }
}

impl Types {
    pub fn load(rom: &Rom) -> Result<Types> {
        Ok(Types {
            entries: garc_files(&rom.read(PERSONAL)?)?,
        })
    }

    /// The species' types, one where both are the same; none for a species not there. Python's
    /// `form` defaults to 0.
    pub fn of(&self, species: i64, form: i64) -> Result<Vec<i64>> {
        if !(0 < species && species < self.entries.len() as i64) {
            return Ok(Vec::new());
        }
        let mut entry = &self.entries[species as usize];
        if form != 0 && entry.len() as i64 > FORMS && form < entry[FORMS as usize] as i64 {
            let index = u16(entry, FORM_INDEX)? + form - 1;
            if 0 < index && index < self.entries.len() as i64 {
                entry = &self.entries[index as usize];
            }
        }
        let (first, second) = (byte(entry, TYPE_1)?, byte(entry, TYPE_2)?);
        Ok(if first == second {
            vec![first]
        } else {
            vec![first, second]
        })
    }
}

/// The words for a summary's type icons, by the Pokémon it shows.
///
/// Its frames sit in a cell: the profile tracks them, and the screen reader's naming of pictures
/// asks it for words, through one `Rc`.
pub struct SummaryTypes {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    types: Option<Types>,
    words: IndexMap<i64, String>,
    frames: RefCell<Vec<i64>>,
}

impl SummaryTypes {
    /// `words` are the types' names by number (`TYPE_NAMES`).
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        types: Option<Types>,
        words: IndexMap<i64, String>,
    ) -> Self {
        SummaryTypes {
            memory,
            names,
            types,
            words,
            frames: RefCell::new(Vec::new()),
        }
    }

    /// Python's `adding` defaults to False.
    pub fn track(&self, frames: impl IntoIterator<Item = i64>, adding: bool) {
        let mut kept = self.frames.borrow_mut();
        *kept = if adding {
            let fresh: Vec<i64> = frames.into_iter().filter(|f| !kept.contains(f)).collect();
            [kept.clone(), fresh].concat()
        } else {
            frames.into_iter().collect()
        };
    }

    /// What the type icon `index` from the left says: that type of the Pokémon shown.
    pub fn word(&self, index: usize) -> Result<Option<String>> {
        let shown = match &self.types {
            None => None,
            Some(_) => self.shown()?,
        };
        let (Some(shown), Some(types)) = (shown, &self.types) else {
            return Ok(None);
        };
        let types = types.of(shown.species, shown.form)?;
        Ok(match types.get(index) {
            None => None,
            Some(number) => self.words.get(number).cloned(),
        })
    }

    fn shown(&self) -> Result<Option<pokemon::Pokemon>> {
        let frames = self.frames.borrow().clone();
        let mut frame = None;
        for f in frames {
            if self.names.of_object(f)?.as_deref() == Some(FRAME) {
                frame = Some(f);
                break;
            }
        }
        let Some(frame) = frame else {
            return Ok(None);
        };
        let view = u32(&self.memory.read(frame + VIEW, 4)?, 0)?;
        let param = if view != 0 {
            u32(&self.memory.read(view + PARAM, 4)?, 0)?
        } else {
            0
        };
        if param == 0 || self.names.of_object(param)?.as_deref() != Some(pokemon::PARAM_CLASS) {
            return Ok(None);
        }
        // Its record is not its own but the party's, which its +0x04 points at.
        let record = u32(&self.memory.read(param + pokemon::PARAM_RECORD, 4)?, 0)?;
        let plain = if record != 0 {
            pokemon::decrypt(&self.memory.read(record, pokemon::RECORD_SIZE)?)
        } else {
            None
        };
        match plain {
            None => Ok(None),
            Some(plain) => pokemon::parse(&plain, SPECIES_MAX),
        }
    }
}
