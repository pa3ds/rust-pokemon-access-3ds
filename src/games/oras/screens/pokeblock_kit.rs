//! Which of the Pokéblock Kit's two screens is shown, the Case or the Berry Blender, so that the
//! other's layouts are put away (spike B on or_pokeblock_kit).
//!
//! L and R turn the Kit between the Case and the Blender, and the game keeps both screens' layouts
//! on screen, every flag of each saying drawn. Once the Blender had been opened both cursors were
//! read as shown: turned back to the Case from the Blender, the Case's cursor still on "Give a
//! Pokéblock" as when it was left, nothing was said, three times running; and as the Case's party
//! list closed, its cursor read the hidden Blender's berry counts under it, "7, 2, 8". The Case's
//! process keeps where it is:
//!
//! ```text
//! app::poroc_case::Proc   +0x28  u8 4 the Case, its party and their questions; 7 the Blender;
//!                                5 and 6 turning from the Case to the Blender, 8 and 3 back;
//!                                5, 9 and 0xA closing, 0xB closed; 1 and 2 opening (read
//!                                every tick, live)
//! app::poroc_case::Proc   +0x20  -> the Case's views, of no class
//! the views              +0x30C  u8 the Case's own screen: 1 its list, 4 the party, 7 the Yes or
//!                                No, 0xA the block given, 3 back to the list; 2 the Blender,
//!                                and 2 again as the Kit closes (not read)
//! ```
//!
//! The screen not shown is put away by its layouts' files, and its lower screen's whole pool with
//! them: each cursor is a layout of its own, `base.bclyt`, built into its screen's pool, and left
//! out, each read the other screen's tab under it, "Make" in the Case. Turning, the screen turned
//! from stays up until the other is; closing, both are put away: the Blender came back for its
//! last polls, "10 90", "7 2 8", the values passed through closing and turning not yet known.
//!
//! Nor does the Kit keep an index of the block or the berry under its cursor: the cursor's place
//! is the one word that changes as it moves (`lyt.X`, `lyt.Y`), and it was seen at a screen poll,
//! up to a fifth of a second after, the block's name a poll after that. The places of the cursors
//! on screen are read every tick, and a move has the screen polled at once
//! (`Gen6Profile.poll_menus`). At rest each holds still, and it jumps from cell to cell (spike B
//! on or_pokeblock_kit).

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::games::common::lyt;
use crate::games::common::layout::{Layouts, TrackedLayouts};
use crate::games::common::layout_file::{FileMatching, LayoutFiles};
use crate::games::gen6::apps::RunningApp;
use crate::games::gen6::game::ScreensReader;
use crate::games::oras::screens::conditions::CASE;
use crate::py::{self, Error, PySet, Result};

pub const SEQUENCE: i64 = 0x28;
/// The Case's list and the block's details, the party and the Pokémon's star, and the block
/// given. A `frozenset` only tested.
pub const CASE_FILES: [&str; 4] = [
    "poroc_case_lower.bclyt",
    "poroc_case_about_upper.bclyt",
    "poroc_case_status_upper.bclyt",
    "poroc_case_eat_lower.bclyt",
];
pub const BLENDER_FILES: [&str; 2] = ["brender_lower.bclyt", "brender_upper.bclyt"];
/// The lower screens, whose pools hold their cursors.
pub const LOWER: [&str; 2] = ["poroc_case_lower.bclyt", "brender_lower.bclyt"];

/// `HIDDEN[sequence]`: `SEQUENCE` -> the files of the screens not shown; none on a value not
/// seen. A dict in the Python.
pub fn hidden(sequence: i64) -> Vec<&'static str> {
    match sequence {
        3 | 4 | 5 | 6 | 8 => BLENDER_FILES.to_vec(),
        7 => CASE_FILES.to_vec(),
        9 | 0xA | 0xB => [CASE_FILES.as_slice(), BLENDER_FILES.as_slice()].concat(),
        _ => Vec::new(),
    }
}
/// A cursor window's X to its Y, read whole and compared by those two alone.
pub const PLACE: i64 = lyt::Y - lyt::X + 4;

/// Which of the Kit's screens is up, and the other's layouts.
pub struct Kit {
    memory: Rc<dyn Memory>,
    apps: Rc<RefCell<RunningApp>>,
    layouts: Rc<dyn TrackedLayouts>,
    files: Rc<dyn FileMatching>,
    /// Work -> its first word, which points into its pool's own work: the same for every work of
    /// a pool (`Layouts`). Read once a work, while the Kit runs.
    pools: IndexMap<i64, i64>,
    /// Cursor window -> its X and Y as of the last tick.
    places: IndexMap<i64, Vec<u8>>,
}

impl Kit {
    pub fn new(
        memory: Rc<dyn Memory>,
        apps: Rc<RefCell<RunningApp>>,
        layouts: Rc<dyn TrackedLayouts>,
        files: Rc<dyn FileMatching>,
    ) -> Self {
        Kit {
            memory,
            apps,
            layouts,
            files,
            pools: IndexMap::new(),
            places: IndexMap::new(),
        }
    }

    /// `Kit(memory, apps, layouts, files)`, as the gen 6 profile builds it (`Game.screens`).
    pub fn of(
        memory: Rc<dyn Memory>,
        apps: Rc<RefCell<RunningApp>>,
        layouts: Rc<Layouts>,
        files: Rc<LayoutFiles>,
    ) -> Box<dyn ScreensReader> {
        Box::new(Kit::new(memory, apps, layouts, files))
    }

    fn pool(&mut self, work: i64) -> Result<i64> {
        if !self.pools.contains_key(&work) {
            let first = u32(&self.memory.read(work, 4)?, 0)?;
            self.pools.insert(work, first);
        }
        Ok(self.pools[&work])
    }
}

impl ScreensReader for Kit {
    /// Whether a cursor on screen has moved since the last tick, in the Kit: `windows`, the
    /// menus' cursors on screen at the last screen poll. One small read each, every tick.
    fn moved(&mut self, windows: &[i64]) -> Result<bool> {
        if self.apps.borrow().running.as_deref() != Some(CASE) {
            self.places = IndexMap::new();
            return Ok(false);
        }
        let last = std::mem::take(&mut self.places);
        for &window in windows {
            let place = self.memory.read(window + lyt::X, PLACE)?;
            let kept = [py::slice(&place, 0, 4), py::slice_from(&place, -4)].concat();
            self.places.insert(window, kept);
        }
        Ok(self
            .places
            .iter()
            .any(|(window, place)| last.get(window).unwrap_or(place) != place))
    }

    /// The works of the layouts of the screens not shown, or none outside the Kit.
    fn put_away(&mut self) -> Result<HashSet<i64>> {
        let (running, process) = {
            let apps = self.apps.borrow();
            (apps.running.clone(), apps.process)
        };
        let Some(process) = process.filter(|_| running.as_deref() == Some(CASE)) else {
            self.pools.clear();
            return Ok(HashSet::new());
        };
        let sequence = *self
            .memory
            .read(process + SEQUENCE, 1)?
            .first()
            .ok_or_else(|| Error::index("index out of range"))?;
        let hidden = hidden(sequence as i64);
        if hidden.is_empty() {
            return Ok(HashSet::new());
        }
        let works = self.layouts.works();
        let mut named: IndexMap<i64, String> = IndexMap::new();
        for (&work, &root) in &works {
            if let Some(file) = self.files.matching(&self.layouts.pane_names(root)) {
                named.insert(work, file.name.clone());
            }
        }
        let away = PySet::of(
            named
                .iter()
                .filter(|(_, name)| hidden.contains(&name.as_str()))
                .map(|(&work, _)| work),
        );
        // `away` is a set of ints, iterated in CPython's order: each pool is read in that order.
        let mut pools: HashSet<i64> = HashSet::new();
        for work in away.iter().collect::<Vec<i64>>() {
            if LOWER.contains(&named[&work].as_str()) {
                pools.insert(self.pool(work)?);
            }
        }
        let mut result: HashSet<i64> = away.iter().collect();
        for &work in works.keys() {
            if pools.contains(&self.pool(work)?) {
                result.insert(work);
            }
        }
        Ok(result)
    }
}
