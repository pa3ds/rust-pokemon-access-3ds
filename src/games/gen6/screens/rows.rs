//! The row under a plain menu's cursor, read every tick, so that a move is heard the tick it is
//! made (spikes B and J).
//!
//! The X menu and the options screen are found from the layout, a window drawn round the row and
//! its words under it, and pa3ds said each move the screen poll its row's byte changed: the layout
//! is not behind the game (findings/cursors.md, spikes J and B). What was left is the screen poll,
//! a fifth of a second. The byte is read every tick while the last screen poll found a cursor on
//! screen, and when it changes the screen is polled at once, and again each tick until the
//! cursor's word is said, a poll's worth of ticks at most (`Gen6Profile`'s `poll_menus`). Over
//! or_menus's 36 moves the word came a median 0.017 s after its byte changed, where it had come
//! 0.121 s after:
//!
//! ```text
//! field::subscreen::FieldMenu  +0x82  u8 the X menu's button, 0 POKÉMON to 5 OPTIONS (spike J)
//! app::config::Process          +0x4C  u8 the options' row, 0 TEXT SPEED to 7 QUIT (spike B)
//! ```
//!
//! The byte says when a row is chosen, never what: the words are still the screen's. Measured in
//! Omega Ruby; a game whose offsets are not measured reads nothing here (`Game.menu_row`,
//! `Game.options_row`).
//!
//! Bound by:
//! - ADR-0013: a cursor source says which item is under the cursor and when it moved, the words
//!   still the screen's.

use std::rc::Rc;

use crate::core::memory::Memory;
use crate::games::gen6::apps::FIELD;
use crate::py::{Error, Result};

pub const OPTIONS: &str = "N3app6config7ProcessE";
/// The field menu -> the X menu's button under the cursor, in Omega Ruby (spike J).
pub const MENU_ROW: i64 = 0x82;
/// The options screen's process -> the row under its cursor, in Omega Ruby (spike B).
pub const OPTIONS_ROW: i64 = 0x4C;
/// Ticks a row's move is chased, the screen polled at each until the cursor's word is said: a
/// screen poll's worth.
pub const CHASE: i64 = 12;

/// The row under the X menu's or the options screen's cursor, read every tick.
pub struct Rows {
    memory: Rc<dyn Memory>,
    running: Rc<dyn Fn() -> Option<String>>,
    process: Rc<dyn Fn() -> Option<i64>>,
    menu: Rc<dyn Fn() -> Result<Option<i64>>>,
    shown: Rc<dyn Fn() -> bool>,
    menu_row: Option<i64>,
    options_row: Option<i64>,
    /// Where the row was read last tick, and what it read.
    last: Option<(i64, i64)>,
}

impl Rows {
    /// `menu` is the field menu, `shown` whether a cursor is on screen, which the field's menu row
    /// is read only while it is; `menu_row` and `options_row` where the game keeps their rows, None
    /// where not measured.
    pub fn new(
        memory: Rc<dyn Memory>,
        running: Rc<dyn Fn() -> Option<String>>,
        process: Rc<dyn Fn() -> Option<i64>>,
        menu: Rc<dyn Fn() -> Result<Option<i64>>>,
        shown: Rc<dyn Fn() -> bool>,
        menu_row: Option<i64>,
        options_row: Option<i64>,
    ) -> Self {
        Rows {
            memory,
            running,
            process,
            menu,
            shown,
            menu_row,
            options_row,
            last: None,
        }
    }

    /// Whether the row changed since the last tick, in the same menu.
    pub fn moved(&mut self) -> Result<bool> {
        let at = self.at()?;
        let now = match at {
            None => None,
            Some(at) => Some((
                at,
                *self
                    .memory
                    .read(at, 1)?
                    .first()
                    .ok_or_else(|| Error::index("index out of range"))? as i64,
            )),
        };
        let last = std::mem::replace(&mut self.last, now);
        Ok(match (now, last) {
            (Some(now), Some(last)) => now.0 == last.0 && now.1 != last.1,
            _ => false,
        })
    }

    /// Where the row of the menu up now is kept, or None.
    fn at(&self) -> Result<Option<i64>> {
        let running = (self.running)();
        if !(self.shown)() {
            return Ok(None);
        }
        if running.as_deref() == Some(OPTIONS)
            && let Some(options_row) = self.options_row
        {
            return Ok((self.process)().map(|process| process + options_row));
        }
        if running.as_deref() == Some(FIELD)
            && let Some(menu_row) = self.menu_row
        {
            return Ok((self.menu)()?.map(|menu| menu + menu_row));
        }
        Ok(None)
    }
}
