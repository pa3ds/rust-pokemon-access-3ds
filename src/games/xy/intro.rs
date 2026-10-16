//! X's opening asks the player to be a boy or a girl, then which of three looks they have, and
//! draws both choices as models with no word on them: the boy and the girl each in a mirror, the
//! looks each a portrait in a frame. No layout holds them, only the message window under them, so
//! neither the cursors nor screen text have anything to read (spike 216).
//!
//! The choice is the intro's own, in the work its process points at:
//!
//! ```text
//! app::intro::IntroProc
//!     +0x24  -> the intro's heap
//!     +0x54  -> the work
//! work
//!     +0x04  -> the process's +0x20      +0x08 -> the intro's heap, as the process's +0x24
//!     +0x24  u32 the opening's step: 0x0c the mirrors take the player's choice, 0x10 the
//!            portraits do
//!     +0x3d  u8 the mirror under the cursor: 0 the boy, 1 the girl; it stays on the one chosen
//!     +0x40  u8 the portrait under the cursor: 0 the left, 1 the middle, 2 the right
//! ```
//!
//! Read over x_character-desc.ctm and x_character-desc2.ctm, a picture taken with every change: the
//! glow round a mirror or a portrait was on the one each byte named, a quarter second ahead of it,
//! and each byte stepped once a press of the circle pad, wrapping round from the last to the first.
//! The work's two pointers back are its check, since its class is in the intro's module.
//!
//! What each choice is called is pa3ds's own, in `looks.toml` beside X's names, a language a folder:
//! the game never names them.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::toml;
use crate::games::common::announce::Focus;
use crate::games::gen6::game::{Data, OpeningReader};
use crate::py::Result;

pub const APP: &str = "N3app5intro9IntroProcE";
pub const HEAP: i64 = 0x24;
pub const WORK: i64 = 0x54;
pub const BACK: i64 = 0x04;
pub const BACK_TO: i64 = 0x20;
pub const WORK_HEAP: i64 = 0x08;
pub const STEP: i64 = 0x24;
pub const GENDER: i64 = 0x3D;
pub const LOOK: i64 = 0x40;
pub const SIZE: i64 = LOOK + 1;

pub const CHOOSING_GENDER: i64 = 0x0C;
pub const CHOOSING_LOOK: i64 = 0x10;
pub const GENDERS: [&str; 2] = ["boy", "girl"];
pub const PLACES: [&str; 3] = ["left", "middle", "right"];

/// pa3ds's words for each choice: `[gender]` by `GENDERS`, and `[boy]` and `[girl]` by
/// `PLACES`.
pub const LOOKS_FILE: &str = "looks.toml";
/// Whose words are said where the player's language has none yet.
pub const FALLBACK: &str = "en";

/// `toml.read(data / language / LOOKS_FILE)`, `data` being the table of files the binary carries
/// where Python's is the folder: a language it has no file for reads as nothing.
fn read(data: Data, language: &str) -> IndexMap<String, ::toml::Value> {
    toml::read_text(
        data.iter()
            .find(|(held, _)| *held == language)
            .map(|(_, file)| *file),
    )
}

/// The words for each choice from `data/<language>/looks.toml`, or the English where the
/// language has no file. A table missing or malformed is no words at all.
pub fn looks(data: Data, language: &str) -> IndexMap<String, IndexMap<String, String>> {
    let mut found = read(data, language);
    if found.is_empty() {
        found = read(data, FALLBACK);
    }
    std::iter::once("gender")
        .chain(GENDERS)
        .map(|section| (section.to_string(), toml::strings(found.get(section))))
        .collect()
}

/// Says the mirror or portrait under the cursor, as the choice opens and at each move.
pub struct IntroReader {
    memory: Rc<dyn Memory>,
    words: IndexMap<String, IndexMap<String, String>>,
    /// The step, the gender and the place said last.
    shown: Option<(i64, i64, i64)>,
}

impl IntroReader {
    pub fn new(memory: Rc<dyn Memory>, words: IndexMap<String, IndexMap<String, String>>) -> Self {
        IntroReader {
            memory,
            words,
            shown: None,
        }
    }
}

impl OpeningReader for IntroReader {
    fn app(&self) -> &'static str {
        APP
    }

    /// The choice under the cursor, when it has just opened or moved; else None.
    fn poll(&mut self, process: i64) -> Result<Option<Focus>> {
        let head = self.memory.read(process, WORK + 4)?;
        let work = u32(&head, WORK)?;
        let found = if work != 0 {
            self.memory.read(work, SIZE)?
        } else {
            vec![0; SIZE as usize]
        };
        if u32(&found, BACK)? != process + BACK_TO || u32(&found, WORK_HEAP)? != u32(&head, HEAP)? {
            self.shown = None;
            return Ok(None);
        }
        let (step, gender, mut place) = (
            u32(&found, STEP)?,
            found[GENDER as usize] as i64,
            found[LOOK as usize] as i64,
        );
        if (step != CHOOSING_GENDER && step != CHOOSING_LOOK)
            || gender >= GENDERS.len() as i64
            || (step == CHOOSING_LOOK && place >= PLACES.len() as i64)
        {
            self.shown = None;
            return Ok(None);
        }
        if step == CHOOSING_GENDER {
            place = 0;
        }
        let shown = (step, gender, place);
        if Some(shown) == self.shown {
            return Ok(None);
        }
        let opened = self.shown.is_none_or(|(last, ..)| last != step);
        self.shown = Some(shown);
        let (label, index, count) = if step == CHOOSING_GENDER {
            let label = self
                .words
                .get("gender")
                .and_then(|words| words.get(GENDERS[gender as usize]));
            (label, gender, GENDERS.len() as i64)
        } else {
            let label = self
                .words
                .get(GENDERS[gender as usize])
                .and_then(|words| words.get(PLACES[place as usize]));
            (label, place, PLACES.len() as i64)
        };
        Ok(label.map(|label| Focus {
            source: work,
            label: label.clone(),
            index,
            count,
            opened,
        }))
    }
}
