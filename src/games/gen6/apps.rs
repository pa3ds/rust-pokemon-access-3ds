//! The running app: which of the game's screens is running, and the moment another starts.
//!
//! Every screen the game runs is a process of its own — the field, the Bag, the Pokédex, a battle —
//! built one at a time and run by the game's process manager, `gamesystem::GameProcManager`, which
//! lives for the whole session (spike 99). The manager points into the process running now, and the
//! process begins with its class, so one read says when another screen has started, and the class
//! says which:
//!
//! ```text
//! manager  +0x10 -> a word inside the running process (+0x64 into the field's, +0x58 into the
//!                   Bag's, +0x150 into the Pokédex's, +0x90 into Wally's battle)
//! process  +0x00 vtable, whose class names the screen    +0x1c -> the manager
//! ```
//!
//! A new screen's objects are what the heap is swept for, and the moment one starts is when to sweep,
//! rather than whenever the schedule next comes round.

use std::collections::HashSet;
use std::rc::Rc;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::py::{self, Result};

pub const MANAGER_CLASS: &str = "N10gamesystem15GameProcManagerE";
/// The field's process: where a page is shown through a talk object, unlike a battle's.
pub const FIELD: &str = "N5field12FieldmapProcE";
/// A battle's process, which starts once a battle, however many switches and Bags it holds.
pub const BATTLE: &str = "N3btl14BattleMainProcE";
/// A scene of the game's own, played in its module: Groudon woken in Seafloor Cavern, the opening
/// before the title. Its process is the same in every one, and says nothing of which (spike 203).
pub const DEMO: &str = "N3app10field_demo4ProcE";
/// Every app that plays a scene of the game's own: the field's demos, the Hall of Fame, and Omega
/// Ruby and Alpha Sapphire's credits, which start three times over the ending, before May's battle,
/// after it and after the save (spike 222).
pub const DEMOS: [&str; 3] = [
    DEMO,
    "N3app11dendou_demo4ProcE",
    "N3app15sango_staffroll13StaffrollProcE",
];
/// The PlayNav's tutorial the first time one of its apps opens, its pages turned by the arrows of
/// the touch screen (or_playnav_amie_tutorial).
pub const TUTORIAL: &str = "N3app19sub_screen_tutorial4ProcE";
pub const MANAGER_RUNNING: i64 = 0x10;
pub const PROCESS_MANAGER: i64 = 0x1C;
/// How far into a process the manager's pointer may land. The Pokédex's lands deepest yet: 0x150.
pub const PROCESS_REACH: i64 = 0x200;
/// Sweeps of the whole heap asked for while no manager is known, and the seconds between them,
/// before leaving the rest to the schedule. The manager lies in memory no heap the watch covers, so
/// only a whole sweep finds it: attached early in the game's boot, the first sweep came before the
/// game had built it, no app start was seen, and every key said "no map" until the scheduled sweep
/// 30 s later (the user's session of 2026-09-27 23:48; omega_ruby_ledges_3.ctm, one replay in
/// three).
pub const ASKS: i64 = 15;
pub const EVERY: f64 = 2.0;
/// Where the executable and the code modules the game loads for each app are mapped. An app's class
/// is in its module, past where the executable ends (`field::FieldmapProc`'s vtable is at
/// 0x007fa2c4), so the class names the runtime reads from the executable alone do not know it.
pub const LOADED_CODE: (i64, i64) = (0x00100000, 0x01000000);

/// Tracks the process manager and says when the process it runs changes.
pub struct RunningApp {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    clock: Rc<dyn Fn() -> f64>,
    asks: i64,
    /// When the next sweep for the manager may be asked for (`EVERY`).
    next: f64,
    manager: Option<i64>,
    /// What the manager pointed at when a process was last found there.
    pointer: Option<i64>,
    /// The class of the process running, as last found.
    pub running: Option<String>,
    /// Where the process running begins, as last found.
    pub process: Option<i64>,
    /// The vtable the process last found begins with.
    vtable: i64,
    /// Whether the process running is of a class that has run before this session.
    pub back: bool,
    ran: HashSet<String>,
}

impl RunningApp {
    /// `RunningApp(memory, names)`: `clock` left to its default, `time.monotonic`.
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        RunningApp::with(memory, names, Rc::new(py::time::monotonic))
    }

    pub fn with(memory: Rc<dyn Memory>, names: Rc<ClassNames>, clock: Rc<dyn Fn() -> f64>) -> Self {
        RunningApp {
            memory,
            names,
            clock,
            asks: 0,
            next: 0.0,
            manager: None,
            pointer: None,
            running: None,
            process: None,
            vtable: 0,
            back: false,
            ran: HashSet::new(),
        }
    }

    /// Take the manager among these, once one is found: it lives as long as the game does.
    pub fn track(&mut self, managers: impl IntoIterator<Item = i64>) -> Result<()> {
        if self.manager.is_none() {
            for manager in managers {
                if self.names.of_object(manager)?.as_deref() == Some(MANAGER_CLASS) {
                    self.manager = Some(manager);
                    break;
                }
            }
        }
        Ok(())
    }

    /// Whether to sweep the whole heap now for the manager, which only such a sweep finds:
    /// none is known, and fewer than `ASKS` have been asked for, the last `EVERY` ago.
    pub fn sweep_wanted(&mut self) -> bool {
        let now = (self.clock)();
        if self.manager.is_some() || self.asks >= ASKS || now < self.next {
            return false;
        }
        self.next = now + EVERY;
        self.asks += 1;
        true
    }

    /// The class of the process running, when it is another than at the last poll; else None.
    ///
    /// A pointer into memory with no process there yet, as between two screens, is waited out: the
    /// running app is the last one found until a process is.
    ///
    /// The same pointer is news too when the process it lands in has begun with another class
    /// since: Super Training's regimen menu and the regimen it starts are built at one address,
    /// and the manager points at the same word in both (omega_ruby_bottom-screen5).
    pub fn poll(&mut self) -> Result<Option<String>> {
        let Some(manager) = self.manager else {
            return Ok(None);
        };
        let head = self.memory.read(manager, MANAGER_RUNNING + 4)?;
        if self.names.of_vtable(u32(&head, 0)?)?.as_deref() != Some(MANAGER_CLASS) {
            self.manager = None;
            return Ok(None);
        }
        let pointer = u32(&head, MANAGER_RUNNING)?;
        if Some(pointer) == self.pointer
            && let Some(process) = self.process
            && u32(&self.memory.read(process, 4)?, 0)? == self.vtable
        {
            return Ok(None);
        }
        let Some((process, vtable)) = self.process(manager, pointer)? else {
            return Ok(None);
        };
        self.process = Some(process);
        self.vtable = vtable;
        self.pointer = Some(pointer);
        // Read afresh every time: a module's memory is the next module's once it is unloaded, so a
        // name kept for a vtable's address could be another app's by now.
        let name = ClassNames::new(self.memory.clone(), LOADED_CODE).of_vtable(vtable)?;
        let running = match name {
            Some(name) if !name.is_empty() => name,
            _ => format!("a class with its vtable at {vtable:#010x}"),
        };
        self.back = self.ran.contains(&running);
        self.ran.insert(running.clone());
        self.running = Some(running.clone());
        Ok(Some(running))
    }

    /// Where the process `pointer` lands in begins, and its vtable: the nearest object at or
    /// before it that points back at the manager and begins with a pointer into the code, or
    /// None.
    fn process(&self, manager: i64, pointer: i64) -> Result<Option<(i64, i64)>> {
        if pointer < PROCESS_REACH {
            return Ok(None);
        }
        let block = self
            .memory
            .read(pointer - PROCESS_REACH, PROCESS_REACH + PROCESS_MANAGER + 4)?;
        let (low, high) = LOADED_CODE;
        for back in (0..=PROCESS_REACH).step_by(4) {
            let at = PROCESS_REACH - back;
            let vtable = u32(&block, at)?;
            if u32(&block, at + PROCESS_MANAGER)? == manager && low <= vtable && vtable < high {
                return Ok(Some((pointer - back, vtable)));
            }
        }
        Ok(None)
    }
}
