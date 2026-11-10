//! Sun's running app: the moment another of the game's screens starts (spike 123).
//!
//! `GameSys::GameProcManager` lives all session, as ORAS's `gamesystem::GameProcManager` does
//! (`games.gen6.apps`), and its +0x10 points at a record of the process running now: zero between
//! two, and somewhere else once another has started. Over sun_4.ctm it moved at 1.9 s, 8.4 s and
//! 20.4 s, the last as the field came up. The record and the process are both in heap 6, two small
//! blocks on Route 1 and in Iki Town, which the walk reads for this (`HEAP`); each object sits 0x20
//! into its block, after gfl2's own header (spike 116, `processes`). Which process it is, is read only
//! through the small blocks the last walk found and the watch has seen allocated since
//! (`heaps.SmallBlocks`), and a pointer into no small block names nothing rather than read past a
//! block into the GPU's (ADR-0035).
//!
//! ```text
//! manager  +0x00 vtable    +0x10 -> the running process's record, or 0 between two
//! record   +0x00 u32 2 as seen    +0x0c -> the process, whose vtable names it: `Field::FieldProc`
//! process  +0x00 vtable    +0x28 -> the manager
//! ```

use std::rc::Rc;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::sm::heaps::Blocks;
use crate::py::Result;

pub const MANAGER_CLASS: &str = "N7GameSys15GameProcManagerE";
pub const RUNNING: i64 = 0x10;
/// The heap the processes and their records are made in (spike 116).
pub const HEAP: i64 = 0x06;
pub const RECORD_PROCESS: i64 = 0x0C;
/// Polls a process started unnamed is looked at again. Its blocks are allocated as it starts and
/// may be read only once the heaps' watch has handed them over, a look after: named as the pointer
/// moved, three of five went unnamed over sun_3.ctm.
pub const NAME_POLLS: i64 = 300;
/// Every cutscene of Sun's, Kukui's call among them (sun_1.ctm).
pub const DEMO: &str = "N3App7AppDemo11AppDemoProcE";
/// Which demo the process plays, a u32 the same all through it: 55 Kukui's call, 6 the chase
/// after it. The rest of the process read the same in both.
pub const DEMO_NUMBER: i64 = 0x24;

/// Tracks the process manager and says when another process has started.
pub struct RunningApp {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    blocks: Option<Blocks>,
    manager: Option<i64>,
    /// Where the manager pointed when a process was last seen starting.
    pointer: Option<i64>,
    /// Polls left to name the process running now, which started unnamed.
    unnamed: i64,
    /// The process running now, once it has been named.
    process: Option<i64>,
}

impl RunningApp {
    /// `RunningApp(memory, names)`: `blocks` left to its default, none.
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        RunningApp::with(memory, names, None)
    }

    /// `blocks` gives the small blocks that may be read now, (start, end) each, through which
    /// the process pointed at is named; without them it goes unnamed.
    pub fn with(memory: Rc<dyn Memory>, names: Rc<ClassNames>, blocks: Option<Blocks>) -> Self {
        RunningApp {
            memory,
            names,
            blocks,
            manager: None,
            pointer: None,
            unnamed: 0,
            process: None,
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

    /// Where the manager points and the class of the process there, when another process
    /// has started since the last poll; else None. The zero between two processes is waited out,
    /// and a process in no small block is unnamed.
    pub fn poll(&mut self) -> Result<Option<(i64, Option<String>)>> {
        let Some(manager) = self.manager else {
            return Ok(None);
        };
        let head = self.memory.read(manager, RUNNING + 4)?;
        if self.names.of_vtable(u32(&head, 0)?)?.as_deref() != Some(MANAGER_CLASS) {
            self.manager = None;
            return Ok(None);
        }
        let pointer = u32(&head, RUNNING)?;
        if pointer == 0 || Some(pointer) == self.pointer {
            return Ok(None);
        }
        self.pointer = Some(pointer);
        self.process = None;
        let name = self.name(pointer)?;
        self.unnamed = if name.as_deref().is_some_and(|name| !name.is_empty()) {
            0
        } else {
            NAME_POLLS
        };
        Ok(Some((pointer, name)))
    }

    /// The class of the process running now, the poll it can first be read, where it started
    /// unnamed; else None.
    pub fn late_name(&mut self) -> Result<Option<String>> {
        let (Some(manager), Some(pointer)) = (self.manager, self.pointer) else {
            return Ok(None);
        };
        if self.unnamed == 0 {
            return Ok(None);
        }
        self.unnamed -= 1;
        if u32(&self.memory.read(manager + RUNNING, 4)?, 0)? != pointer {
            self.unnamed = 0;
            return Ok(None);
        }
        let name = self.name(pointer)?;
        if name.is_some() {
            self.unnamed = 0;
        }
        Ok(name)
    }

    /// Which demo the process running now plays, its word at `DEMO_NUMBER`, once it has been
    /// named and while the word lies inside the small blocks; else None. Asked only of a process
    /// named `DEMO`.
    pub fn demo(&self) -> Result<Option<i64>> {
        let (Some(process), Some(blocks)) = (self.process, &self.blocks) else {
            return Ok(None);
        };
        let mut spans: Vec<(i64, i64)> = blocks().to_vec();
        spans.sort();
        if !inside(&spans, process + DEMO_NUMBER, 4) {
            return Ok(None);
        }
        Ok(Some(u32(&self.memory.read(process + DEMO_NUMBER, 4)?, 0)?))
    }

    fn name(&mut self, record: i64) -> Result<Option<String>> {
        let Some(blocks) = &self.blocks else {
            return Ok(None);
        };
        let mut spans: Vec<(i64, i64)> = blocks().to_vec();
        spans.sort();
        if !inside(&spans, record + RECORD_PROCESS, 4) {
            return Ok(None);
        }
        let process = u32(&self.memory.read(record + RECORD_PROCESS, 4)?, 0)?;
        if !inside(&spans, process, 4) {
            return Ok(None);
        }
        let name = self.names.of_object(process)?;
        if name.is_some() {
            self.process = Some(process);
        }
        Ok(name)
    }
}

/// Whether `size` bytes at `address` lie inside one of these sorted spans.
fn inside(spans: &[(i64, i64)], address: i64, size: i64) -> bool {
    let at = spans.partition_point(|&span| span <= (address, 0xFFFFFFFF)) as i64 - 1;
    at >= 0 && address + size <= spans[at as usize].1
}
