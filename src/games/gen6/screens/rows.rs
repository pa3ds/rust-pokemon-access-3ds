//! The row under a plain menu's cursor, or the key under the name entry's, read every tick, so
//! that a move is heard the tick it is made (spikes B and J).
//!
//! The X menu, the options screen, the party and the PC are found from the layout, a window or a
//! picture drawn round the row and its words under it, and pa3ds said each move the screen poll
//! its row's byte changed: the layout is not behind the game (findings/cursors.md, spikes J and
//! B). The name entry's key cursor is a picture moved from key to key (`common.cursors.key`). What
//! was left is the screen poll, a fifth of a second. The byte is read every tick while the last screen poll
//! found a cursor on screen, and when it changes the screen is polled at once, and again each tick
//! until the cursor's word is said, a poll's worth of ticks at most (`Gen6Profile`'s
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
//! app::strinput::Proc           +0x24  -> app::strinput::MainSystem
//! app::strinput::MainSystem    +0x3E0  -> the keyboard's cursor, null until the keyboard is built
//! the keyboard's cursor         +0x14  u8 the key under it, 0x0B Q to 0x2E return on the letters
//!                                      (spike B on or_opening and x_opening); +0x15 the key before
//! app::pc_menu::Proc   +0x24 +0x28 +0x4  -> what holds the PC's menus, through objects of no class
//! what holds them               +0x1C  -> the menu up now, "Which PC should be accessed?"'s or
//!                                         Someone's PC's, each built as it first opens
//! the menu up                   +0x14  u8 the row under its cursor, 0 Deposit Pokémon to 4 Log
//!                                      Out (spike B on or_pc_organize and x_pc_mode_buttons);
//!                                      +0x15 the row before
//! app::wazaoboe::Process        +0x44  -> app::wazaoboe::Lower, Move to forget's moves
//! app::wazaoboe::Lower          +0x54  -> its cursor, of no class, a PC menu's shape: +0x10 how
//!                                         many moves, +0x14 u8 the move under it, 0 to 4, the
//!                                         one to learn last (spike B on or_move_reminder)
//! app::wazaomoidashi::SelectWazaProc  +0x40  -> app::wazaomoidashi::ParentView
//! app::wazaomoidashi::ParentView      +0x7C  -> app::wazaomoidashi::LowerView
//! app::wazaomoidashi::LowerView       +0x94  -> the Move Reminder's list, of no class
//! the list                      +0x10  how many moves; +0x18 u32 the row the cursor is on, 0 to
//!                                      3, and +0x1C f32 the rows scrolled away, their sum the
//!                                      move (spike B on or_move_reminder, 16 moves)
//! ```
//!
//! The byte says when a row is chosen, never what: the words are still the screen's. Measured in
//! Omega Ruby and X; a game whose offsets are not measured reads nothing here (`Game.menu_row`,
//! `Game.options_row`, `Game.party_slot`, `Game.move_rows`), but for the keyboard's and the PC's,
//! the same in both. The move screens are measured in Omega Ruby alone: no X movie opens them.
//!
//! Bound by:
//! - ADR-0013: a cursor source says which item is under the cursor and when it moved, the words
//!   still the screen's.

use std::rc::Rc;

use crate::core::binary;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::gen6::anchors::follow;
use crate::games::gen6::apps::{FIELD, LOADED_CODE};
use crate::games::gen6::screens::boxes;
use crate::py::{self, Error, Result};

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
pub const KEYBOARD: &str = "N3app8strinput4ProcE";
/// The name entry's process -> its `app::strinput::MainSystem`.
pub const KEYBOARD_SYSTEM: i64 = 0x24;
pub const MAIN_SYSTEM: &str = "N3app8strinput10MainSystemE";
/// `MainSystem` -> the keyboard's cursor, null until the keyboard is built.
pub const KEYS: i64 = 0x3E0;
/// The keyboard's cursor -> the key under it, in Omega Ruby and X (spike B).
pub const KEY: i64 = 0x14;
pub const PC_MENU: &str = "N3app7pc_menu4ProcE";
/// The PC's and the Box's processes -> their control blocks, through objects of no class (spike
/// B). A dict in the Python: `controls` looks an app up in it.
pub const CONTROLS: [(&str, [i64; 3]); 2] = [
    (PC_MENU, [0x24, 0x28, 0x4]),
    (boxes::APP, [0x24, 0x50, 0x4]),
];
/// A control block -> the cursor that has the input: the PC's menu up now, "Which PC should be
/// accessed?"'s or Someone's PC's, each built as it first opens; the Box's grid, with its party and
/// its name bar, the Battle Box's, or a menu's.
pub const IN_USE: i64 = 0x1C;
/// The cursor in use -> the slot or row under it, in Omega Ruby and X (spike B).
pub const SLOT: i64 = 0x14;
/// The cursor in use -> how many slots it walks (spike B).
pub const SLOTS: i64 = 0x10;
/// The slots the Box's grid cursor walks: the box's 30, the party's 6, its name bar's and the
/// bottom bar's (spike B on or_organize_boxes).
pub const GRID_SLOTS: i64 = 48;
pub const FORGET: &str = "N3app8wazaoboe7ProcessE";
/// Move to forget's process -> `app::wazaoboe::Lower` -> its cursor, whose row is at `SLOT`.
pub const FORGET_ROWS: [i64; 2] = [0x44, 0x54];
pub const LOWER: &str = "N3app8wazaoboe5LowerE";
pub const REMEMBER: &str = "N3app13wazaomoidashi14SelectWazaProcE";
/// The Move Reminder's process -> `ParentView` -> `LowerView` -> its list.
pub const REMEMBER_LIST: [i64; 3] = [0x40, 0x7C, 0x94];
pub const PARENT_VIEW: &str = "N3app13wazaomoidashi10ParentViewE";
/// The Move Reminder's list -> the row the cursor is on, a u32, and after it the rows scrolled
/// away, an f32: both read, as a move down a long list changes the scroll alone.
pub const LIST_ROW: i64 = 0x18;
/// Ticks a row's move is chased, the screen polled at each until the cursor's word is said: a
/// screen poll's worth.
pub const CHASE: i64 = 12;

/// The Move Reminder's move: the row the cursor is on plus the rows scrolled away.
fn r#move(row_and_scroll: &[u8]) -> Result<f64> {
    let (row, scroll) = (
        binary::u32(row_and_scroll, 0)?,
        binary::f32(row_and_scroll, 4)?,
    );
    Ok(row as f64 + scroll)
}

/// `CONTROLS[app]`: the hops to an app's control block, where it has one.
pub fn controls(app: &str) -> Option<[i64; 3]> {
    CONTROLS
        .iter()
        .find(|(each, _)| *each == app)
        .map(|&(_, hops)| hops)
}

/// The row under the X menu's, the options screen's, the party's, the PC's or the move screens'
/// cursor, or the key under the name entry's, read every tick.
pub struct Rows {
    memory: Rc<dyn Memory>,
    running: Rc<dyn Fn() -> Option<String>>,
    process: Rc<dyn Fn() -> Option<i64>>,
    menu: Rc<dyn Fn() -> Result<Option<i64>>>,
    shown: Rc<dyn Fn() -> bool>,
    menu_row: Option<i64>,
    options_row: Option<i64>,
    party_slot: Option<i64>,
    move_rows: bool,
    /// Where the row was read last tick, and what it read.
    last: Option<(i64, Vec<u8>)>,
    /// Where the row was read and what it read as it last moved, while it is still that menu's:
    /// none once another menu has taken the input.
    moved_to: Option<(i64, Vec<u8>)>,
    /// The app running when the row was last looked for.
    app: Option<String>,
    /// The party's, the name entry's, the PC's or a move screen's process and the object holding
    /// its byte or its menus, once found: its class, where it has one, is checked once a screen,
    /// and only what it holds read each tick after. Dropped once another app runs, lest the next
    /// of the same app be built at the same address round other objects.
    work: Option<(i64, i64)>,
    names: Option<(i64, ClassNames)>,
}

impl Rows {
    /// `menu` is the field menu, `shown` whether a cursor is on screen, which the field's menu row
    /// is read only while it is; `menu_row`, `options_row` and `party_slot` where the game keeps
    /// their rows, None where not measured, and `move_rows` whether the move screens' are.
    /// Python's defaults are None for `party_slot` and False for `move_rows`.
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
        move_rows: bool,
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
            move_rows,
            last: None,
            moved_to: None,
            app: None,
            work: None,
            names: None,
        }
    }

    /// Whether the row changed since the last tick, in the same menu.
    pub fn moved(&mut self) -> Result<bool> {
        let at = self.at()?;
        let now = match at {
            None => None,
            Some((at, size)) => Some((at, self.memory.read(at, size)?)),
        };
        let last = std::mem::replace(&mut self.last, now.clone());
        let (Some(now), Some(last)) = (now, last) else {
            self.moved_to = None;
            return Ok(false);
        };
        if now.0 != last.0 {
            self.moved_to = None;
            return Ok(false);
        }
        if now.1 == last.1 {
            return Ok(false);
        }
        self.moved_to = Some(now.clone());
        // The Move Reminder's list wrapping round rewrites every row over a few frames after
        // its row changes, and a poll at once read one half written: "Dragon Breath Normal",
        // then "Sky Attack" (or_move_reminder, live). A jump is left to the screen poll.
        Ok(now.1.len() == 1 || (r#move(&now.1)? - r#move(&last.1)?).abs() <= 1.0)
    }

    /// The slot the Box's pointer has been moved to, on it or gliding there, while its grid cursor
    /// has the input, as the game counts them (`slots.nth`). None until it moves: back from a
    /// menu, a Pokémon just picked up was said in its slot as its icon was yet to rise, then said
    /// empty (or_organize_boxes). None anywhere else, the Battle Box and the Box's menus too.
    pub fn box_slot(&self) -> Result<Option<i64>> {
        if (self.running)().as_deref() != Some(boxes::APP) {
            return Ok(None);
        }
        let Some((at, slot)) = &self.moved_to else {
            return Ok(None);
        };
        let slots = *self
            .memory
            .read(at - SLOT + SLOTS, 1)?
            .first()
            .ok_or_else(|| Error::index("index out of range"))? as i64;
        Ok(if slots == GRID_SLOTS {
            Some(*slot
                .first()
                .ok_or_else(|| Error::index("index out of range"))? as i64)
        } else {
            None
        })
    }

    /// Where the row of the menu up now is kept and how many bytes it takes, or None.
    fn at(&mut self) -> Result<Option<(i64, i64)>> {
        let running = (self.running)();
        if running != self.app {
            (self.app, self.work) = (running.clone(), None);
        }
        if !(self.shown)() {
            return Ok(None);
        }
        if running.as_deref() == Some(OPTIONS)
            && let Some(options_row) = self.options_row
        {
            return Ok((self.process)().map(|process| (process + options_row, 1)));
        }
        if running.as_deref() == Some(FIELD)
            && let Some(menu_row) = self.menu_row
        {
            return Ok((self.menu)()?.map(|menu| (menu + menu_row, 1)));
        }
        if running.as_deref() == Some(PARTY)
            && let Some(party_slot) = self.party_slot
        {
            return Ok(self
                .found(&[PARTY_WORK], Some(STATIC_WORK))?
                .map(|work| (work + party_slot, 1)));
        }
        if running.as_deref() == Some(KEYBOARD) {
            return Ok(self
                .found(&[KEYBOARD_SYSTEM, KEYS], Some(MAIN_SYSTEM))?
                .map(|keys| (keys + KEY, 1)));
        }
        if let Some(hops) = running.as_deref().and_then(controls) {
            let control = self.found(&hops, None)?;
            // The cursor in use changes as another takes the input, so this hop is followed
            // every tick.
            let cursor = match control {
                None => None,
                Some(control) => follow(&*self.memory, control, &[IN_USE])?,
            };
            return Ok(cursor.map(|cursor| (cursor + SLOT, 1)));
        }
        if running.as_deref() == Some(FORGET) && self.move_rows {
            let cursor = self.found(&FORGET_ROWS, Some(LOWER))?;
            return Ok(cursor.map(|cursor| (cursor + SLOT, 1)));
        }
        if running.as_deref() == Some(REMEMBER) && self.move_rows {
            let moves = self.found(&REMEMBER_LIST, Some(PARENT_VIEW))?;
            return Ok(moves.map(|moves| (moves + LIST_ROW, 8)));
        }
        Ok(None)
    }

    /// Where `hops` lead from the process, once the object its first hop leads to is of class
    /// `name`, where it has one.
    fn found(&mut self, hops: &[i64], name: Option<&str>) -> Result<Option<i64>> {
        let Some(process) = (self.process)() else {
            return Ok(None);
        };
        if self.work.is_none_or(|work| work.0 != process) {
            let Some(first) = follow(&*self.memory, process, py::slice(hops, 0, 1))? else {
                return Ok(None);
            };
            if let Some(name) = name {
                // Named afresh for each screen: its classes are in a module, whose memory is the
                // next module's once it is unloaded (`apps.RunningApp.poll`).
                if self.names.as_ref().is_none_or(|names| names.0 != process) {
                    self.names = Some((process, ClassNames::new(self.memory.clone(), LOADED_CODE)));
                }
                let names = &self.names.as_ref().expect("named above").1;
                if names.of_object(first)?.as_deref() != Some(name) {
                    return Ok(None);
                }
            }
            let Some(work) = follow(&*self.memory, first, py::slice_from(hops, 1))? else {
                return Ok(None);
            };
            self.work = Some((process, work));
        }
        Ok(self.work.map(|work| work.1))
    }
}
