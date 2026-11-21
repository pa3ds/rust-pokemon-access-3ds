//! The row under a plain menu's cursor, read every tick, so that a move is heard the tick it is
//! made (spikes B and J).
//!
//! The X menu, the options screen and the party are found from the layout, a window or a picture
//! drawn round the row and its words under it, and pa3ds said each move the screen poll its row's
//! byte changed: the layout is not behind the game (findings/cursors.md, spikes J and B). What was
//! left is the screen poll, a fifth of a second. The byte is read every tick while the last screen
//! poll found a cursor on screen, and when it changes the screen is polled at once, and again each
//! tick until the cursor's word is said, a poll's worth of ticks at most (`Gen6Profile`'s
//! `poll_menus`). Over or_menus's 36 moves the word came a median 0.017 s after its byte changed,
//! where it had come 0.121 s after:
//!
//! ```text
//! field::subscreen::FieldMenu  +0x82  u8 the X menu's button, 0 POKÉMON to 5 OPTIONS (spike J)
//!                              +0x76  the same in X, 0xC earlier as its button manager is
//! app::config::Process          +0x4C  u8 the options' row, 0 TEXT SPEED to 7 QUIT (spike B),
//!                                      in X too, which has a row more and counts on past it
//! app::pokelist::Proc           +0x90  -> app::pokelist::StaticWork
//! app::pokelist::StaticWork    +0x1A8  u8 the party's slot, 0 to 5 across its two columns, the
//!                                      slot chosen to switch with too (spike B on or_horde_battle
//!                                      and or_riding_sharpedo); +0x1B0 in X and Y (x_hoard_battle,
//!                                      y_legend_wakening_boss). +0x42 is the slot too, but holds
//!                                      the Pokémon being switched until the switch is made.
//! ```
//!
//! The byte says when a row is chosen, never what: the words are still the screen's. Measured in
//! Omega Ruby and X; a game whose offsets are not measured reads nothing here (`Game.menu_row`,
//! `Game.options_row`, `Game.party_slot`).
//!
//! Bound by:
//! - ADR-0013: a cursor source says which item is under the cursor and when it moved, the words
//!   still the screen's.

use std::rc::Rc;

use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::gen6::anchors::follow;
use crate::games::gen6::apps::{FIELD, LOADED_CODE};
use crate::py::{Error, Result};

pub const OPTIONS: &str = "N3app6config7ProcessE";
pub const PARTY: &str = "N3app8pokelist4ProcE";
/// The party's process -> its `app::pokelist::StaticWork`.
pub const PARTY_WORK: i64 = 0x90;
pub const STATIC_WORK: &str = "N3app8pokelist10StaticWorkE";
/// `StaticWork` -> the party's slot under the cursor, in Omega Ruby (spike B).
pub const PARTY_SLOT: i64 = 0x1A8;
/// The same in X and Y (spike B on x_hoard_battle and y_legend_wakening_boss).
pub const XY_PARTY_SLOT: i64 = 0x1B0;
/// The field menu -> the X menu's button under the cursor, in Omega Ruby (spike J).
pub const MENU_ROW: i64 = 0x82;
/// The same in X (spike J on x_opening).
pub const XY_MENU_ROW: i64 = 0x76;
/// The options screen's process -> the row under its cursor, in Omega Ruby and X (spike B).
pub const OPTIONS_ROW: i64 = 0x4C;
/// Ticks a row's move is chased, the screen polled at each until the cursor's word is said: a
/// screen poll's worth.
pub const CHASE: i64 = 12;

/// The row under the X menu's, the options screen's or the party's cursor, read every tick.
pub struct Rows {
    memory: Rc<dyn Memory>,
    running: Rc<dyn Fn() -> Option<String>>,
    process: Rc<dyn Fn() -> Option<i64>>,
    menu: Rc<dyn Fn() -> Result<Option<i64>>>,
    shown: Rc<dyn Fn() -> bool>,
    menu_row: Option<i64>,
    options_row: Option<i64>,
    party_slot: Option<i64>,
    /// Where the row was read last tick, and what it read.
    last: Option<(i64, i64)>,
    /// The party's process and its `StaticWork`, once found: its class is checked once a party
    /// screen, and only its byte read each tick after.
    work: Option<(i64, i64)>,
    names: Option<(i64, ClassNames)>,
}

impl Rows {
    /// `menu` is the field menu, `shown` whether a cursor is on screen, which the field's menu row
    /// is read only while it is; `menu_row`, `options_row` and `party_slot` where the game keeps
    /// their rows, None where not measured. Python's default for `party_slot` is None.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        memory: Rc<dyn Memory>,
        running: Rc<dyn Fn() -> Option<String>>,
        process: Rc<dyn Fn() -> Option<i64>>,
        menu: Rc<dyn Fn() -> Result<Option<i64>>>,
        shown: Rc<dyn Fn() -> bool>,
        menu_row: Option<i64>,
        options_row: Option<i64>,
        party_slot: Option<i64>,
    ) -> Self {
        Rows {
            memory,
            running,
            process,
            menu,
            shown,
            menu_row,
            options_row,
            party_slot,
            last: None,
            work: None,
            names: None,
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
    fn at(&mut self) -> Result<Option<i64>> {
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
        if running.as_deref() == Some(PARTY)
            && let Some(party_slot) = self.party_slot
        {
            return Ok(self.party_work()?.map(|work| work + party_slot));
        }
        Ok(None)
    }

    fn party_work(&mut self) -> Result<Option<i64>> {
        let Some(process) = (self.process)() else {
            return Ok(None);
        };
        if self.work.is_none_or(|work| work.0 != process) {
            // Named afresh for each party screen: its classes are in a module, whose memory is
            // the next module's once it is unloaded (`apps.RunningApp.poll`).
            if self.names.as_ref().is_none_or(|names| names.0 != process) {
                self.names = Some((process, ClassNames::new(self.memory.clone(), LOADED_CODE)));
            }
            let work = follow(&*self.memory, process, &[PARTY_WORK])?;
            let names = &self.names.as_ref().expect("named above").1;
            let Some(work) = work else {
                return Ok(None);
            };
            if names.of_object(work)?.as_deref() != Some(STATIC_WORK) {
                return Ok(None);
            }
            self.work = Some((process, work));
        }
        Ok(self.work.map(|work| work.1))
    }
}
