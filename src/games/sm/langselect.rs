//! The language choice Sun opens a new game with, and the confirmation after it.
//!
//! ```text
//! App::LangSelect::LangSelectUpperDraw
//!     +0xb4  u8 the screen: 0 the list of languages, 1 "Selected language" and its two choices
//!     +0x12c u8 the language under the cursor, in the list's order, 0 to 8 (+0x130 in the Ultra
//!            games, `sm.VersionLayout`)
//!     +0x12d u8 the choice under the cursor: 0 "Begin game", 1 "Select language", the byte
//!            after the language
//! ```
//!
//! Read in spike 111 over sun_2.ctm, a picture taken each time the word changed: every press moved
//! the language a step, English 0 to 繁體中文 8, and each matched the language by the arrow.
//!
//! The list's buttons are pictures, each language's name in its own script, and no word of them is in
//! memory. The game's own words for them are the English names in its system text, file 51, in the
//! list's order — the words "Selected language" shows the choice by — and the rest of what the two
//! screens say is there too: the title over the list, the confirmation's two choices, and the
//! instructions on the bottom screen.

use std::rc::Rc;

use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::announce::Focus;
use crate::games::sm::text::SunText;
use crate::games::sm::{SUN_LAYOUT, VersionLayout};
use crate::py::Result;

pub const CLASS: &str = "N3App10LangSelect19LangSelectUpperDrawE";
pub const SCREEN: i64 = 0xB4;
pub const LANGUAGE: i64 = SUN_LAYOUT.chosen_language;
/// The choice's byte, after the language's.
pub const CHOICE: i64 = 1;
pub const LIST: i64 = 0;
pub const CONFIRM: i64 = 1;

pub const TEXT_FILE: i64 = 51;
/// The bottom screen's instructions, a line a screen; line 2 is empty.
///
/// A dict keyed by `LIST` and `CONFIRM`, which are 0 and 1: here an array indexed by the screen.
pub const HELP: [i64; 2] = [1, 3];
/// "Play the game in" over the list, "Selected language" over the confirmation.
///
/// Indexed by the screen, as `HELP` is.
pub const TITLE: [i64; 2] = [4, 5];
/// The first line of each screen's items, and how many there are: the languages, then "Begin
/// game" and "Select language".
///
/// Indexed by the screen, as `HELP` is.
pub const ITEMS: [(i64, i64); 2] = [(8, 9), (17, 2)];

/// What one poll found to say: a screen that has come up, and the item under its cursor.
#[derive(Debug, Clone, PartialEq)]
pub struct Shown {
    /// The screen's title, when the screen has just come up.
    pub title: Option<String>,
    pub focus: Focus,
    /// The bottom screen's instructions, when the screen has just come up.
    pub help: Option<String>,
}

/// Says the language choice and its confirmation, a screen at a time and an item a move.
pub struct LangSelectReader {
    memory: Rc<dyn Memory>,
    language: i64,
    names: Rc<ClassNames>,
    text: Option<Rc<SunText>>,
    draws: Vec<i64>,
    /// The draw and the screen said last, so a screen is announced once as it comes up.
    shown: Option<(i64, i64)>,
    index: Option<i64>,
}

impl LangSelectReader {
    /// `memory` should read exactly what it is asked for (`heaps`); `text` is file 51 of the
    /// system text, and without it nothing is said, since the words are all the ROM's.
    ///
    /// Python's `layout` defaults to `SUN_LAYOUT`.
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        text: Option<Rc<SunText>>,
        layout: VersionLayout,
    ) -> Self {
        LangSelectReader {
            memory,
            language: layout.chosen_language,
            names,
            text,
            draws: Vec::new(),
            shown: None,
            index: None,
        }
    }

    /// Read these upper screens' draws from now on: in place of those read so far, or,
    /// `adding` those a sweep of the blocks just allocated found, beside them.
    pub fn track(&mut self, draws: impl IntoIterator<Item = i64>, adding: bool) {
        self.draws = if adding {
            let mut kept = self.draws.clone();
            kept.extend(draws.into_iter().filter(|d| !self.draws.contains(d)));
            kept
        } else {
            draws.into_iter().collect()
        };
    }

    /// Whether the language choice is up and read here, so its screen is this reader's.
    pub fn showing(&self) -> bool {
        self.shown.is_some()
    }

    /// The screen that has come up or the item the cursor has moved to, if either has.
    pub fn poll(&mut self) -> Result<Option<Shown>> {
        let mut draw = None;
        for &d in &self.draws {
            if self.names.of_object(d)?.as_deref() == Some(CLASS) {
                draw = Some(d);
                break;
            }
        }
        let (Some(draw), Some(text)) = (draw, self.text.clone()) else {
            self.shown = None;
            self.index = None;
            return Ok(None);
        };
        let screen = i64::from(self.memory.read(draw + SCREEN, 1)?[0]);
        if screen != LIST && screen != CONFIRM {
            return Ok(None);
        }
        let (first, count) = ITEMS[screen as usize];
        let index = i64::from(
            self.memory.read(
                draw + self.language + if screen == LIST { 0 } else { CHOICE },
                1,
            )?[0],
        );
        if index >= count {
            return Ok(None);
        }
        let Some(label) = text.line(TEXT_FILE, first + index) else {
            return Ok(None);
        };
        let opened = self.shown != Some((draw, screen));
        if !opened && Some(index) == self.index {
            return Ok(None);
        }
        self.shown = Some((draw, screen));
        self.index = Some(index);
        let focus = Focus {
            source: draw,
            label,
            index,
            count,
            opened,
        };
        if !opened {
            return Ok(Some(Shown {
                title: None,
                focus,
                help: None,
            }));
        }
        let mut title = text.line(TEXT_FILE, TITLE[screen as usize]);
        if screen == CONFIRM
            && let Some(said) = title.clone()
        {
            let chosen = i64::from(self.memory.read(draw + self.language, 1)?[0]);
            let language = text.line(TEXT_FILE, ITEMS[LIST as usize].0 + chosen);
            if let Some(language) = language {
                title = Some(format!("{said}: {language}"));
            }
        }
        Ok(Some(Shown {
            title,
            focus,
            help: text.line(TEXT_FILE, HELP[screen as usize]),
        }))
    }
}
