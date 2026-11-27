//! The game's button managers, which say whose buttons take taps now (spike 189).
//!
//! An app's buttons are answered through `app::tool::ButtonManager`s, each holding buttons in a
//! layout or two. A manager's update returns at once while its byte +0x24 is set, and the game sets
//! it to lock a screen's buttons under a menu: the party's list under "Do what with Blaze?"
//! (movie22). A layout whose every manager is locked takes no tap, however bright it is drawn.
//!
//! An app's bottom bar is a **touch bar** instead, a view of a class of the app's module's own
//! (`app::pokelist::TouchBar`, `app::bag::TouchBar`) built on the executable's
//! `app::ui::UITouchBarView`: it updates the manager at its +0x10 only while its byte +0x2d is set,
//! which the party clears under its menu too, the manager's own lock left open. The field's PokéNav
//! has no manager at all, and the PC Box keeps its grid's open under "Continue operations?": a list
//! window is answered first (`touch_targets.answered_first`).
//!
//! ```text
//! manager    +0x10  its buttons, a pointer each; +0x14 how many
//!            +0x24  locked
//! button     +0x08  a handle, whose +0x04 is its layout's work
//!            +0x10  the number of its touch area in that layout, or of its picture where it has
//!                   none, which the pane is named by in base 36: 0x111 is "7l"
//!            +0x18  its layout's animations, a word each to +0x28, 0xFFFFFFFF for none
//!            +0x28  the animation that greys it (`PASSIVE`)
//!            +0x35  it takes taps
//! touch bar  +0x10  its manager
//!            +0x14  its input listener, inside the touch bar itself
//!            +0x2d  it takes input
//! field menu +0x7c  its manager (X's +0x70, `Game.field_menu`)
//!            +0x9c  an app of the bottom screen has the input (X's +0x90)
//! ```
//!
//! Super Training's page locks no manager (spike 228, or_super_train): it switches its buttons off
//! one by one. Under its Use panel the party, the training-bags button and Super Training read 0 at
//! +0x35 and the bags 1, and a tap on a bag chose it while a tap on a party member did nothing;
//! under its Yes and No only those two and the bag chosen read 1, and taps on the bag chosen, a
//! party member and POKÉMON did nothing. A button switched off is left out, as a layout no button
//! of which takes taps is.
//!
//! The frame round the PlayNav's pages, its arrows and the X menu's icons along the bottom, is the
//! field's `field::subscreen::FieldMenu`'s, whose manager's buttons read 1 throughout: the menu sets
//! its +0x9c while the page has the input, from the bags list on, and draws the frame's icons
//! masked, and a tap on "previous" under the Use panel did nothing. Its manager counts as locked
//! meanwhile.
//!
//! A button switched off is drawn greyed where it has an animation to grey it: another player's
//! Battle, Trade and O-Power in the PSS, offline, their icons faded to alpha 100, the PlayNav
//! tutorial's arrows, Amie's difficulty buttons as their menu opens (`greyed`). Super Training's
//! buttons have no animations at all, and are drawn as bright as ever switched off; the DexNav's
//! panel none at +0x28 (or_pss_singleplayer_2, the round after it).

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::discovery::{HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::py::{self, PySet, Result};

pub const MANAGER_CLASS: &str = "N3app4tool13ButtonManagerE";
/// The class every app's touch bar is built on, found with every class built on it.
pub const BAR_CLASS: &str = "N3app2ui14UITouchBarViewE";
pub const BUTTONS: i64 = 0x10;
pub const COUNT: i64 = 0x14;
pub const LOCKED: i64 = 0x24;
pub const HANDLE: i64 = 0x08;
pub const WORK: i64 = 0x04;
pub const BAR_MANAGER: i64 = 0x10;
pub const LISTENER: i64 = 0x14;
pub const INPUT: i64 = 0x2D;
/// How far into a touch bar its listener is: a word of a module's file kept in the heap, its
/// vtable's address among its own, has no listener inside it.
pub const BAR_SIZE: i64 = 0x100;
pub const MAX_BUTTONS: i64 = 64;
pub const PANE: i64 = 0x10;
pub const PASSIVE: i64 = 0x28;
pub const ON: i64 = 0x35;
pub const MENU_CLASS: &str = "N5field9subscreen9FieldMenuE";
/// Where ORAS's field menu keeps its manager and the byte set while an app has the input.
pub const MENU: (i64, i64) = (0x7C, 0x9C);
pub const DIGITS: &str = "0123456789abcdefghijklmnopqrstuvwxyz";

/// The name of a layout's pane by its number, which Game Freak's layouts name it by in base
/// 36: 0x111 is "7l".
pub fn pane_name(number: i64) -> String {
    let digits: Vec<char> = DIGITS.chars().collect();
    let mut number = number;
    let mut name = String::new();
    loop {
        let digit = py::modulo(number, 36);
        number = py::floordiv(number, 36);
        name.insert(0, digits[digit as usize]);
        if number == 0 {
            return name;
        }
    }
}

/// The button managers and touch bars a sweep found, and the layouts they lock.
pub struct Buttons {
    memory: Rc<dyn Memory>,
    /// Where a field menu keeps its manager and the byte set while an app has the input.
    menu: (i64, i64),
    /// Manager -> the vtable it was found with.
    managers: IndexMap<i64, i64>,
    /// Touch bar -> the vtable it was found with.
    bars: IndexMap<i64, i64>,
    /// Field menu -> the vtable it was found with.
    menus: IndexMap<i64, i64>,
    /// Manager -> its list of buttons as last read, and the works of the layouts they are in.
    works: IndexMap<i64, (Vec<u8>, PySet)>,
}

impl Buttons {
    pub fn new(memory: Rc<dyn Memory>) -> Self {
        Buttons::with_menu(memory, MENU)
    }

    pub fn with_menu(memory: Rc<dyn Memory>, menu: (i64, i64)) -> Self {
        Buttons {
            memory,
            menu,
            managers: IndexMap::new(),
            bars: IndexMap::new(),
            menus: IndexMap::new(),
            works: IndexMap::new(),
        }
    }

    /// The managers, touch bars and field menus a sweep found: all of them, or with `adding`
    /// the new ones.
    pub fn track(
        &mut self,
        managers: impl IntoIterator<Item = i64>,
        bars: impl IntoIterator<Item = i64>,
        menus: impl IntoIterator<Item = i64>,
        adding: bool,
    ) -> Result<()> {
        let read = |at: i64| -> Result<i64> { u32(&self.memory.read(at, 4)?, 0) };
        let mut found: IndexMap<i64, i64> = IndexMap::new();
        for at in managers {
            found.insert(at, read(at)?);
        }
        let mut merged = if adding {
            self.managers.clone()
        } else {
            IndexMap::new()
        };
        merged.extend(found);
        self.managers = merged;
        let read = |at: i64| -> Result<i64> { u32(&self.memory.read(at, 4)?, 0) };
        let mut found: IndexMap<i64, i64> = IndexMap::new();
        for at in bars {
            found.insert(at, read(at)?);
        }
        let mut merged = if adding {
            self.bars.clone()
        } else {
            IndexMap::new()
        };
        merged.extend(found);
        self.bars = merged;
        let read = |at: i64| -> Result<i64> { u32(&self.memory.read(at, 4)?, 0) };
        let mut found: IndexMap<i64, i64> = IndexMap::new();
        for at in menus {
            found.insert(at, read(at)?);
        }
        let mut merged = if adding {
            self.menus.clone()
        } else {
            IndexMap::new()
        };
        merged.extend(found);
        self.menus = merged;
        let managers = &self.managers;
        self.works.retain(|at, _| managers.contains_key(at));
        Ok(())
    }

    /// The works of the layouts whose buttons take no tap now: every manager with buttons in
    /// them locked, or held by a touch bar taking no input. A layout no manager has buttons in is
    /// not among them.
    pub fn idle(&mut self) -> Result<HashSet<i64>> {
        let memory = self.memory.clone();
        let mut unfed: HashSet<i64> = HashSet::new();
        for (&bar, &vtable) in &self.bars {
            let head = memory.read(bar, INPUT + 1)?;
            if u32(&head, 0)? == vtable
                && 0 < u32(&head, LISTENER)? - bar
                && u32(&head, LISTENER)? - bar < BAR_SIZE
                && head[INPUT as usize] == 0
            {
                unfed.insert(u32(&head, BAR_MANAGER)?);
            }
        }
        let (held, busy) = self.menu;
        for (&menu, &vtable) in &self.menus {
            let head = memory.read(menu, busy + 1)?;
            // The sweep finds the menu by each of its vtables, and only the whole object holds a
            // manager the sweep found where the menu's is.
            if u32(&head, 0)? == vtable
                && self.managers.contains_key(&u32(&head, held)?)
                && head[busy as usize] != 0
            {
                unfed.insert(u32(&head, held)?);
            }
        }
        let mut taking: IndexMap<i64, bool> = IndexMap::new();
        let managers: Vec<(i64, i64)> = self.managers.iter().map(|(&m, &v)| (m, v)).collect();
        for (manager, vtable) in managers {
            let head = memory.read(manager, LOCKED + 1)?;
            if u32(&head, 0)? != vtable {
                continue;
            }
            let open = head[LOCKED as usize] == 0 && !unfed.contains(&manager);
            for work in self.works_of(manager, &head)?.iter() {
                let before = taking.get(&work).copied().unwrap_or(false);
                taking.insert(work, before || open);
            }
        }
        Ok(taking
            .into_iter()
            .filter(|&(_, open)| !open)
            .map(|(work, _)| work)
            .collect())
    }

    /// The field menu itself, of those the sweep found: the whole object, the one holding a
    /// manager the sweep found where the menu's is (`idle`).
    pub fn menu(&self) -> Result<Option<i64>> {
        let (held, _busy) = self.menu;
        for (&menu, &vtable) in &self.menus {
            let head = self.memory.read(menu, held + 4)?;
            if u32(&head, 0)? == vtable && self.managers.contains_key(&u32(&head, held)?) {
                return Ok(Some(menu));
            }
        }
        Ok(None)
    }

    /// (work, pane name) of every touch area whose buttons are all switched off now: its
    /// layout's work and its name, or its picture's where it has none.
    pub fn off(&self) -> Result<HashSet<(i64, String)>> {
        let (on, found, _greyed) = self.switched()?;
        // X's regimen menu keeps two buttons on its regimen's touch area, one of them off.
        Ok(found.difference(&on).cloned().collect())
    }

    /// (work, pane name) of every button switched on now. The PSS Settings' Yes and No are
    /// windows holding their words, with no touch area: the manager takes a tap on the window
    /// (spike 312).
    pub fn on(&self) -> Result<HashSet<(i64, String)>> {
        Ok(self.switched()?.0)
    }

    /// (work, pane name) of the buttons switched off now that have an animation to grey them
    /// (`PASSIVE`).
    pub fn greyed(&self) -> Result<HashSet<(i64, String)>> {
        let (on, _found, greyed) = self.switched()?;
        Ok(greyed.difference(&on).cloned().collect())
    }

    /// (work, pane name) of the buttons switched on now, of those switched off, and of those
    /// switched off that the game greys.
    #[allow(clippy::type_complexity)]
    fn switched(
        &self,
    ) -> Result<(
        HashSet<(i64, String)>,
        HashSet<(i64, String)>,
        HashSet<(i64, String)>,
    )> {
        let read = |at: i64, size: i64| self.memory.read(at, size);
        let mut found: HashSet<(i64, String)> = HashSet::new();
        let mut on: HashSet<(i64, String)> = HashSet::new();
        let mut greyed: HashSet<(i64, String)> = HashSet::new();
        for (&manager, &vtable) in &self.managers {
            let head = read(manager, COUNT + 4)?;
            let (table, count) = (u32(&head, BUTTONS)?, u32(&head, COUNT)?);
            if u32(&head, 0)? != vtable || table == 0 || !(0 < count && count <= MAX_BUTTONS) {
                continue;
            }
            let entries = read(table, 4 * count)?;
            let mut states: Vec<Vec<u8>> = Vec::new();
            for i in 0..count {
                let button = u32(&entries, 4 * i)?;
                if within(HEAP_SPAN, button) {
                    states.push(read(button, ON + 1)?);
                }
            }
            // A manager with no button on says nothing: an app leaves its so as it closes. Once in
            // six replays of bottom_screen2 the Amie tutorial's every touch area was left out with
            // nothing locked, as a freed manager's buttons in the layout slot it took would do.
            if !states.iter().any(|state| state[ON as usize] != 0) {
                continue;
            }
            for state in &states {
                let handle = u32(state, HANDLE)?;
                if within(HEAP_SPAN, handle) {
                    let key = (
                        u32(&read(handle + WORK, 4)?, 0)?,
                        pane_name(u32(state, PANE)?),
                    );
                    if state[ON as usize] != 0 {
                        on.insert(key.clone());
                    } else {
                        found.insert(key.clone());
                    }
                    if state[ON as usize] == 0 && u32(state, PASSIVE)? != 0xFFFFFFFF {
                        greyed.insert(key);
                    }
                }
            }
        }
        Ok((on, found, greyed))
    }

    /// The works of the layouts a manager's buttons are in, read again when its list of
    /// buttons is another than last read.
    fn works_of(&mut self, manager: i64, head: &[u8]) -> Result<PySet> {
        let listed = py::slice(head, BUTTONS, COUNT + 4).to_vec();
        if let Some(known) = self.works.get(&manager)
            && known.0 == listed
        {
            return Ok(known.1.clone());
        }
        let (table, count) = (u32(&listed, 0)?, u32(&listed, 4)?);
        let mut works = PySet::new();
        if table != 0 && 0 < count && count <= MAX_BUTTONS {
            let read = |at: i64, size: i64| self.memory.read(at, size);
            let entries = read(table, 4 * count)?;
            // Only where the heap is: a battle keeps much of itself in memory the GPU may hold,
            // where a read crashes players' Azahar (azahar-emu/azahar#2619).
            for i in 0..count {
                let button = u32(&entries, 4 * i)?;
                let handle = if within(HEAP_SPAN, button) {
                    u32(&read(button + HANDLE, 4)?, 0)?
                } else {
                    0
                };
                if within(HEAP_SPAN, handle) {
                    works.add(u32(&read(handle + WORK, 4)?, 0)?);
                }
            }
        }
        self.works.insert(manager, (listed, works));
        Ok(self.works[&manager].1.clone())
    }
}
