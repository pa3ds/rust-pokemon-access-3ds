//! The row under a plain menu's cursor, or the key under the name entry's, read every tick, so
//! that a move is heard the tick it is made (spikes 298 and 306).
//!
//! The X menu, the options screen, the party and the PC are found from the layout, a window or a
//! picture drawn round the row and its words under it, and pa3ds said each move the screen poll
//! its row's byte changed: the layout is not behind the game (findings/cursors.md, spikes 306 and
//! 298). The name entry's key cursor is a picture moved from key to key (`common.cursors.key`).
//! What was left is the screen poll, a fifth of a second. The byte is read every tick while the
//! last screen poll found a cursor on screen, and when it changes the screen is polled at once,
//! and again each tick until the cursor's word is said, a poll's worth of ticks at most
//! (`Gen6Profile`'s `poll_menus`). Over or_menus's 36 moves the word came a median 0.017 s after
//! its byte changed, where it had come 0.121 s after:
//!
//! ```text
//! field::subscreen::FieldMenu  +0x82  u8 the X menu's button, 0 POKÉMON to 5 OPTIONS (spike 306)
//!                              +0x76  the same in X, 0xC earlier as its button manager is
//! app::config::Process          +0x4C  u8 the options' row, 0 TEXT SPEED to 7 QUIT (spike 298),
//!                                      in X too, which has a row more and counts on past it;
//!                                      from +0x4D a byte a row, its value: TEXT SPEED 1 NORMAL,
//!                                      2 FAST (spike 298 on or_opening and x_opening), read with
//!                                      the row, so a value turned is a move too
//! app::pokelist::Proc           +0x90  -> app::pokelist::StaticWork
//! app::pokelist::StaticWork    +0x1A8  u8 the party's slot, 0 to 5 across its two columns, the
//!                                      slot chosen to switch with too (spike 298 on
//!                                      or_horde_battle and or_riding_sharpedo); +0x1B0 in X and Y
//!                                      (x_hoard_battle, y_legend_wakening_boss). +0x42 is the
//!                                      slot too, but holds the Pokémon being switched until the
//!                                      switch is made.
//! app::strinput::Proc           +0x24  -> app::strinput::MainSystem
//! app::strinput::MainSystem    +0x3E0  -> the keyboard's cursor, null until the keyboard is built
//! the keyboard's cursor         +0x14  u8 the key under it, 0x0B Q to 0x2E return on the letters
//!                                      (spike 298 on or_opening and x_opening); +0x15 the key
//!                                      before
//! app::pc_menu::Proc   +0x24 +0x28 +0x4  -> what holds the PC's menus, through objects of no class
//! what holds them               +0x1C  -> the menu up now, "Which PC should be accessed?"'s or
//!                                         Someone's PC's, each built as it first opens
//! the menu up                   +0x14  u8 the row under its cursor, 0 Deposit Pokémon to 4 Log
//!                                      Out (spike 298 on or_pc_organize and x_pc_mode_buttons);
//!                                      +0x15 the row before
//! app::wazaoboe::Process        +0x44  -> app::wazaoboe::Lower, Move to forget's moves
//! app::wazaoboe::Lower          +0x54  -> its cursor, of no class, a PC menu's shape: +0x10 how
//!                                         many moves, +0x14 u8 the move under it, 0 to 4, the
//!                                         one to learn last (spike 298 on or_move_reminder)
//! the Move Reminder's list      its row and the rows scrolled away (`reminder.Row`)
//! app::title_menu::Proc         +0x24  -> app::title_menu::MainSystem
//! app::title_menu::MainSystem  +0x1114 u8 the title menu's row, 0 CONTINUE, 1 MYSTERY GIFT, 2
//!                                      LIVE COMPETITION (spike 298 on or_pokemon_center_pc)
//! app::poke3select::Proc        +0x3C  -> app::poke3select::View
//! app::poke3select::View        +0x80  -> app::poke3select::lower::View
//! app::poke3select::lower::View +0xB4  u8 the starter's ball under the cursor, 0 to 2 left to
//!                                      right, 3 before the first move, read with no cursor on
//!                                      screen: the ball is told by its description (spike 298 on
//!                                      or_opening and x_opening; upper::View +0xEC the same)
//! contest::MainProc             +0x30  -> contest::Main, a contest's appeals
//! contest::Main                 +0x30  -> contest::AppealUi
//! contest::AppealUi            +0x24C  u8 the entrant the cursor has picked, 4 to 7 its portrait
//!                                      left to right, kept as the cursor goes down into the
//!                                      moves it shows: a move between portraits, not one onto
//!                                      them (spike 298 on or_toughness_contest, Omega Ruby alone)
//! app::kisekae::FittingProc  +0x24 +0x4 +0x20 +0x28  -> the fitting room's cursor, a PC menu's
//!                                      shape: +0x10 how many slots, 32; +0x14 u8 the slot
//!                                      under it, 0 hat to 7 hat accessory down the categories,
//!                                      0xD or 0xE the list of things to wear, 0x16 Change, 0x1E
//!                                      Back; +0x15 the slot before. The list scrolls under it,
//!                                      a move along the list left to the screen poll (spike 298
//!                                      on x_cyllage_boutique_fitting_room, X alone)
//! ```
//!
//! The byte says when a row is chosen, never what: the words are still the screen's. Measured in
//! Omega Ruby and X; a game whose offsets are not measured reads nothing here (`Shape`,
//! `CursorShapes.rows`), but for the keyboard's, the PC's, the move screens',
//! the title menu's and the starter's, the same in both (spike 298 on x_move_reminder and
//! x_title_screen for the move screens and the title menu), and the contest's, which X and Y have
//! none of, and the fitting room's, which Omega Ruby has none of.
//!
//! Bound by:
//! - ADR-0013: a cursor source says which item is under the cursor and when it moved, the words
//!   still the screen's.
//! - ADR-0043: a row, key or ball is read every tick, a change polling the screen at once.

use std::rc::Rc;

use crate::core::memory::Memory;
use crate::games::gen6::anchors::{Kept, follow};
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::screens::{boxes, reminder};
use crate::py::{self, Error, Result};

pub const OPTIONS: &str = "N3app6config7ProcessE";
pub const PARTY: &str = "N3app8pokelist4ProcE";
/// The party's process -> its `app::pokelist::StaticWork`.
pub const PARTY_WORK: i64 = 0x90;
pub const STATIC_WORK: &str = "N3app8pokelist10StaticWorkE";
/// `StaticWork` -> the party's slot under the cursor, in Omega Ruby (spike 298).
pub const PARTY_SLOT: i64 = 0x1A8;
/// The same in X and Y (spike 298 on x_hoard_battle and y_legend_wakening_boss).
pub const XY_PARTY_SLOT: i64 = 0x1B0;
/// The field menu -> the X menu's button under the cursor, in Omega Ruby (spike 306).
pub const MENU_ROW: i64 = 0x82;
/// The same in X (spike 306 on x_opening).
pub const XY_MENU_ROW: i64 = 0x76;
/// The options screen's process -> the row under its cursor, in Omega Ruby and X (spike 298).
pub const OPTIONS_ROW: i64 = 0x4C;
/// The options' row and the values after it, a byte a row, X's eight rows and Omega Ruby's: a
/// value turned with left or right is said the tick it is drawn, as a row is.
pub const OPTIONS_READ: i64 = 10;
pub const KEYBOARD: &str = "N3app8strinput4ProcE";
/// The name entry's process -> its `app::strinput::MainSystem`.
pub const KEYBOARD_SYSTEM: i64 = 0x24;
pub const MAIN_SYSTEM: &str = "N3app8strinput10MainSystemE";
/// `MainSystem` -> the keyboard's cursor, null until the keyboard is built.
pub const KEYS: i64 = 0x3E0;
/// The keyboard's cursor -> the key under it, in Omega Ruby and X (spike 298).
pub const KEY: i64 = 0x14;
pub const PC_MENU: &str = "N3app7pc_menu4ProcE";
/// The PC's and the Box's processes -> their control blocks, through objects of no class (spike
/// 298). A dict in the Python: `controls` looks an app up in it.
pub const CONTROLS: [(&str, [i64; 3]); 2] = [
    (PC_MENU, [0x24, 0x28, 0x4]),
    (boxes::APP, [0x24, 0x50, 0x4]),
];
/// A control block -> the cursor that has the input: the PC's menu up now, "Which PC should be
/// accessed?"'s or Someone's PC's, each built as it first opens; the Box's grid, with its party and
/// its name bar, the Battle Box's, or a menu's.
pub const IN_USE: i64 = 0x1C;
/// The cursor in use -> the slot or row under it, in Omega Ruby and X (spike 298).
pub const SLOT: i64 = 0x14;
/// The cursor in use -> how many slots it walks (spike 298).
pub const SLOTS: i64 = 0x10;
/// The slots the Box's grid cursor walks: the box's 30, the party's 6, its name bar's and the
/// bottom bar's (spike 298 on or_organize_boxes).
pub const GRID_SLOTS: i64 = 48;
pub const FORGET: &str = "N3app8wazaoboe7ProcessE";
/// Move to forget's process -> `app::wazaoboe::Lower` -> its cursor, whose row is at `SLOT`.
pub const FORGET_ROWS: [i64; 2] = [0x44, 0x54];
pub const LOWER: &str = "N3app8wazaoboe5LowerE";
pub const TITLE: &str = "N3app10title_menu4ProcE";
/// The title menu's process -> its `app::title_menu::MainSystem`.
pub const TITLE_SYSTEM: i64 = 0x24;
pub const TITLE_MAIN: &str = "N3app10title_menu10MainSystemE";
/// `MainSystem` -> the title menu's row, in Omega Ruby (spike 298).
pub const TITLE_ROW: i64 = 0x1114;
pub const STARTER: &str = "N3app11poke3select4ProcE";
/// The starter's process -> `app::poke3select::View` -> its `lower::View`.
pub const STARTER_VIEWS: [i64; 2] = [0x3C, 0x80];
pub const STARTER_VIEW: &str = "N3app11poke3select4ViewE";
/// `lower::View` -> the ball under the cursor, in Omega Ruby and X (spike 298).
pub const BALL: i64 = 0xB4;
pub const CONTEST: &str = "N7contest8MainProcE";
/// A contest's appeals' process -> `contest::Main` -> its `contest::AppealUi`.
pub const CONTEST_UI: [i64; 2] = [0x30, 0x30];
pub const CONTEST_MAIN: &str = "N7contest4MainE";
/// `AppealUi` -> the entrant the cursor has picked, its portrait 4 to 7 (spike 298).
pub const PICKED: i64 = 0x24C;
pub const FITTING: &str = "N3app7kisekae11FittingProcE";
/// The fitting room's process -> its cursor, whose slot is at `SLOT` (spike 298).
pub const FITTING_CURSOR: [i64; 4] = [0x24, 0x4, 0x20, 0x28];
/// The fitting room's slots below this are its categories, hat to hat accessory.
pub const CATEGORIES: i64 = 8;

/// Ticks a row's move is chased, the screen polled at each until the cursor's word is said: a
/// screen poll's worth.
pub const CHASE: i64 = 12;

/// Where a game keeps the rows that sit at other offsets in each game: the X menu's button,
/// the options' row and the party's slot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shape {
    /// As `MENU_ROW`.
    pub menu_row: i64,
    /// As `OPTIONS_ROW`.
    pub options_row: i64,
    /// As `PARTY_SLOT`.
    pub party_slot: i64,
}

pub const ORAS: Shape = Shape {
    menu_row: MENU_ROW,
    options_row: OPTIONS_ROW,
    party_slot: PARTY_SLOT,
};
pub const XY: Shape = Shape {
    menu_row: XY_MENU_ROW,
    options_row: OPTIONS_ROW,
    party_slot: XY_PARTY_SLOT,
};


/// `CONTROLS[app]`: the hops to an app's control block, where it has one.
pub fn controls(app: &str) -> Option<[i64; 3]> {
    CONTROLS
        .iter()
        .find(|(each, _)| *each == app)
        .map(|&(_, hops)| hops)
}

/// The row under the X menu's, the options screen's, the party's, the PC's or the move screens'
/// cursor, the key under the name entry's, the entrant a contest's has picked or the slot under
/// the fitting room's, read every tick.
pub struct Rows {
    memory: Rc<dyn Memory>,
    running: Rc<dyn Fn() -> Option<String>>,
    process: Rc<dyn Fn() -> Option<i64>>,
    menu: Rc<dyn Fn() -> Result<Option<i64>>>,
    shown: Rc<dyn Fn() -> bool>,
    shape: Option<Shape>,
    /// Where the row was read last tick, and what it read.
    last: Option<(i64, Vec<u8>)>,
    /// Where the row was read and what it read as it last moved, while it is still that menu's:
    /// none once another menu has taken the input.
    moved_to: Option<(i64, Vec<u8>)>,
    /// The party's, the name entry's, the PC's, a move screen's, the title menu's, the
    /// starter's, a contest's or the fitting room's object holding its byte or its menus, once
    /// found: its class, where it has one, is checked once a screen, and only what it holds read
    /// each tick after. Kept by the app as well as the process: X's fitting room is built where
    /// its title menu was, and kept by the process alone it read the title menu's object for its
    /// cursor and heard no move (x_cyllage_boutique_fitting_room).
    kept: Kept,
    reminder: reminder::Row,
}

impl Rows {
    /// `menu` is the field menu, `shown` whether a cursor is on screen, which the field's menu row
    /// is read only while it is; `shape` where the game keeps the rows of its own, None where
    /// not measured.
    pub fn new(
        memory: Rc<dyn Memory>,
        running: Rc<dyn Fn() -> Option<String>>,
        process: Rc<dyn Fn() -> Option<i64>>,
        menu: Rc<dyn Fn() -> Result<Option<i64>>>,
        shown: Rc<dyn Fn() -> bool>,
        shape: Option<Shape>,
    ) -> Self {
        Rows {
            kept: Kept::new(memory.clone(), process.clone(), running.clone()),
            reminder: reminder::Row::new(memory.clone(), running.clone(), process.clone()),
            memory,
            running,
            process,
            menu,
            shown,
            shape,
            last: None,
            moved_to: None,
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
        let running = (self.running)();
        if running.as_deref() == Some(reminder::REMEMBER) {
            // The Move Reminder's list wrapping round rewrites every row over a few frames after
            // its row changes, and a poll at once read one half written: "Dragon Breath Normal",
            // then "Sky Attack" (or_move_reminder, live). A jump is left to the screen poll.
            return Ok(!reminder::jumped(&last.1, &now.1)?);
        }
        if running.as_deref() == Some(FITTING) {
            // The fitting room's cursor glides onto its list and its buttons, and a poll at once
            // read the card half written, "Two-Tone Bag", "red", or the tab it passed, "hat
            // accessory bag" (x_cyllage_boutique_fitting_room, live): those moves are left to
            // the screen poll, and only one between categories, which it jumps, is chased.
            let first = |read: &[u8]| -> Result<i64> {
                Ok(*read
                    .first()
                    .ok_or_else(|| Error::index("index out of range"))? as i64)
            };
            return Ok(first(&now.1)?.max(first(&last.1)?) < CATEGORIES);
        }
        Ok(true)
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
            Some(
                *slot
                    .first()
                    .ok_or_else(|| Error::index("index out of range"))? as i64,
            )
        } else {
            None
        })
    }

    /// Where the row of the menu up now is kept and how many bytes it takes, or None.
    fn at(&mut self) -> Result<Option<(i64, i64)>> {
        let running = (self.running)();
        if running.as_deref() == Some(STARTER) {
            // No cursor is found on the starter's screen, its ball told by its description.
            let view = self.found(&STARTER_VIEWS, Some(STARTER_VIEW))?;
            return Ok(view.map(|view| (view + BALL, 1)));
        }
        if !(self.shown)() {
            return Ok(None);
        }
        let shape = self.shape;
        if running.as_deref() == Some(OPTIONS)
            && let Some(shape) = shape
        {
            return Ok((self.process)().map(|process| (process + shape.options_row, OPTIONS_READ)));
        }
        if running.as_deref() == Some(FIELD)
            && let Some(shape) = shape
        {
            return Ok((self.menu)()?.map(|menu| (menu + shape.menu_row, 1)));
        }
        if running.as_deref() == Some(PARTY)
            && let Some(shape) = shape
        {
            return Ok(self
                .found(&[PARTY_WORK], Some(STATIC_WORK))?
                .map(|work| (work + shape.party_slot, 1)));
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
        if running.as_deref() == Some(FORGET) {
            let cursor = self.found(&FORGET_ROWS, Some(LOWER))?;
            return Ok(cursor.map(|cursor| (cursor + SLOT, 1)));
        }
        if running.as_deref() == Some(TITLE) {
            let main = self.found(&[TITLE_SYSTEM], Some(TITLE_MAIN))?;
            return Ok(main.map(|main| (main + TITLE_ROW, 1)));
        }
        if running.as_deref() == Some(reminder::REMEMBER) {
            return self.reminder.at();
        }
        if running.as_deref() == Some(CONTEST) {
            let ui = self.found(&CONTEST_UI, Some(CONTEST_MAIN))?;
            return Ok(ui.map(|ui| (ui + PICKED, 1)));
        }
        if running.as_deref() == Some(FITTING) {
            let cursor = self.found(&FITTING_CURSOR, None)?;
            return Ok(cursor.map(|cursor| (cursor + SLOT, 1)));
        }
        Ok(None)
    }


    /// Where `hops` lead from the process, once the object its first hop leads to is of class
    /// `name`, where it has one.
    fn found(&self, hops: &[i64], name: Option<&str>) -> Result<Option<i64>> {
        self.kept
            .find(py::slice(hops, 0, 1), name, py::slice_from(hops, 1))
    }
}
