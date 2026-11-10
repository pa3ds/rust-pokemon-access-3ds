//! Layouts: whether a pane is being drawn, on which screen, and what panes a layout holds.
//!
//! Everything ORAS draws in 2D is a NintendoWare layout: a tree of `nw::lyt::Pane`s under a root,
//! one `nw::lyt::Layout` per tree. Screens build their layouts long before showing them and keep
//! panes that are not on screen, so being findable says nothing about being seen. Two things do
//! (spikes 29 and 30), from the structures in `lyt`:
//!
//! - Every pane from this one up to the root has the visible bit, and this one's global alpha (its
//!   ancestors' folded in) is not 0.
//! - The root's layout is being drawn: Game Freak's work for it reads alive and drawn.
//!
//! A layout's panes are found by walking its tree from the root (spike 49), which reaches TextBoxes
//! a sweep would take seconds to find: the field's AreaNav keeps its work in the heap and its panes
//! in the linear heap.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{f32, u16, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::lyt;
use crate::py::PySet;
use crate::py::{self, Error, Kind, Result};

/// The deepest chain seen is five panes; anything past this is a loop or garbage.
pub const MAX_DEPTH: i64 = 32;
/// The most panes walked in one layout. The summary's biggest holds 410.
pub const MAX_PANES: i64 = 4096;
/// Tracks between walks of a layout already walked, for a tree rebuilt between two sweeps.
pub const REWALK_TRACKS: i64 = 5;
/// Works that stopped being works, kept to be looked at again. The field's are built again where
/// they were each time a screen opened from the X menu closes (spikes 41 and 49).
pub const DEAD_KEPT: i64 = 64;
/// Looks, a screen poll each, that a work found before its layout is built is looked at again: a
/// minute, and the net sweeps the whole heap every 30 s, so a work still waiting is found again
/// before it is let go. The battle makes the level up's work as it starts and builds its layout
/// only as the Pokémon grows, 24 s on in movie2, and looked at for five seconds it was never read
/// (2026-09-26).
pub const PENDING_LOOKS: i64 = 300;
/// Screen polls a layout just taken in has its tree walked again: taken in the moment its work was
/// allocated, Sun's save prompt's cursor had no panes yet, its tree was walked empty and never
/// again, and the prompt's Yes and No were read out whole (sun_3.ctm, spike 123).
pub const SETTLE_WALKS: i64 = 5;
/// Screen polls a layout just taken in is not taken for listed (`lyt.WORK_LISTED`): the AreaNav
/// draws its layouts itself, and one of its works read listed once, 0.17 s after it started, and
/// never again (or_area_nav). Taken for listed, a layout is taken off the list for good once it
/// reads 0, and the AreaNav's bar, its tabs and return, was left out of touch mode until pa3ds
/// closed.
pub const BUILT_POLLS: i64 = 5;
/// Works waiting for their layout, at most; the oldest is let go.
pub const PENDING_KEPT: i64 = 128;
/// Between the bottom screen's 320 and the top screen's 400, for a work's projection width.
pub const SCREEN_SPLIT: f64 = 360.0;
/// A root this far up or down from the middle is drawn by another view than its projection's. The
/// battle's works all read the bottom screen's projection and a view offset of 1,000; its
/// bottom-screen roots come out at (0, 0) and its top-screen roots at (0, 1000), and taken for the
/// bottom screen, the level up's values at its right edge read as past the side (spike 48).
pub const OTHER_VIEW: f64 = 500.0;

/// Where a pane is: whether it is on screen now, on which screen, and in which layout.
///
/// Python's `Placement(shown, top)` leaves the rest at their defaults: [`Placement::new`] does,
/// and the other fields are `pub` for a struct update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub shown: bool,
    pub top: bool,
    /// The root pane of its layout, which names the layout.
    pub root: i64,
    /// The pane directly under the root that this one hangs from. The game groups a screen's parts
    /// with it: the battle's HP box keeps the player's level and HP in one and the wild Pokemon's
    /// level in another, the party screen keeps "Choose a Pokemon." apart from its slots, and the
    /// battle bag gives each pocket name its own (spike 53). Free: the walk to the root passes it.
    pub panel: i64,
    /// Every pane to the root visible, and a global alpha above 0.
    pub visible: bool,
    /// The layout's work reads alive and drawn.
    pub drawn: bool,
    /// The layout's work reads alive, drawn or not.
    pub alive: bool,
}

impl Placement {
    /// `Placement(shown, top)`: `root` and `panel` 0, and `visible`, `drawn` and `alive` True.
    pub fn new(shown: bool, top: bool) -> Self {
        Placement {
            shown,
            top,
            root: 0,
            panel: 0,
            visible: true,
            drawn: true,
            alive: true,
        }
    }
}

/// `_climb`'s answer: (visible, root, panel, the root's head).
type Climb = (bool, i64, i64, Vec<u8>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Layout {
    work: i64,
    top: bool,
    /// The layout object the work held when taken in: another there is another layout.
    layout: i64,
}

/// `head[at]`, raising `IndexError` where `head` stops short of it.
#[track_caller]
fn byte(head: &[u8], at: i64) -> Result<i64> {
    match usize::try_from(at).ok().and_then(|at| head.get(at)) {
        Some(&found) => Ok(i64::from(found)),
        None => Err(Error::index("index out of range")),
    }
}

/// Tracks layout works and answers whether panes under them are being drawn.
///
/// Reads are cached until `refresh`, so a reader that asks about many panes sharing ancestors
/// reads each ancestor once per poll.
///
/// Every reader holds the same one, so it is shared (`Rc<Layouts>`) and its methods take `&self`.
pub struct Layouts {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    /// Where this game's panes keep their fields, for every reader that reads them here.
    pub shape: lyt::PaneShape,
    roots: RefCell<IndexMap<i64, Layout>>,
    panes: RefCell<IndexMap<(i64, i64), Vec<u8>>>,
    states: RefCell<IndexMap<i64, Vec<u8>>>,
    /// Pane -> whether it and the panes above it are visible, its root, the pane under the root it
    /// hangs from and the root's head, or None off any tree: `placement`'s walk up, read once until
    /// `refresh`, since a touch mode walk asked 22,000 times (movie22).
    climbs: RefCell<IndexMap<i64, Option<Climb>>>,
    /// The works of layouts their app has put away and does not draw, though every flag of theirs
    /// says drawn (`put_away`).
    put_away: RefCell<HashSet<i64>>,
    /// (work, root) of every layout seen on the system's list of layouts to draw: one taken off it
    /// is no longer drawn (`lyt.WORK_LISTED`).
    listed: RefCell<HashSet<(i64, i64)>>,
    /// Root -> screen polls left before its layout, just taken in, is taken for listed
    /// (`BUILT_POLLS`).
    built: RefCell<IndexMap<i64, i64>>,
    /// Root -> every pane under it, with its class name, as last walked.
    contents: RefCell<IndexMap<i64, IndexMap<i64, String>>>,
    /// Root -> the names of the panes under it, in the order walked.
    pane_names: RefCell<IndexMap<i64, Vec<String>>>,
    /// Root -> each pane under it -> its children in order, as last walked.
    children: RefCell<IndexMap<i64, IndexMap<i64, Vec<i64>>>>,
    /// Root -> the track its tree was last walked at.
    walked: RefCell<IndexMap<i64, i64>>,
    /// Root -> screen polls left to walk its tree again, for layouts just taken in.
    settling: RefCell<IndexMap<i64, i64>>,
    tracks: Cell<i64>,
    /// Layout pool -> (its table, its capacity), for pools a handle or a work has led to (spikes
    /// 64 and 99).
    pools: RefCell<IndexMap<i64, (i64, i64)>>,
    /// Watched pool -> how many of its slots were in use when its table was last read.
    in_use: RefCell<IndexMap<i64, i64>>,
    /// A pool's own work -> the pool, or None where no pool was found for it.
    owns: RefCell<IndexMap<i64, Option<i64>>>,
    /// Work -> the root it had, for works that stopped being works, oldest first.
    dead: RefCell<IndexMap<i64, i64>>,
    /// Work -> looks left, for works found before their layout was built.
    pending: RefCell<IndexMap<i64, i64>>,
}

impl Layouts {
    /// `Layouts(memory, names, shape)`; Python's default `shape` is `lyt::ORAS`.
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>, shape: lyt::PaneShape) -> Self {
        Layouts {
            memory,
            names,
            shape,
            roots: RefCell::new(IndexMap::new()),
            panes: RefCell::new(IndexMap::new()),
            states: RefCell::new(IndexMap::new()),
            climbs: RefCell::new(IndexMap::new()),
            put_away: RefCell::new(HashSet::new()),
            listed: RefCell::new(HashSet::new()),
            built: RefCell::new(IndexMap::new()),
            contents: RefCell::new(IndexMap::new()),
            pane_names: RefCell::new(IndexMap::new()),
            children: RefCell::new(IndexMap::new()),
            walked: RefCell::new(IndexMap::new()),
            settling: RefCell::new(IndexMap::new()),
            tracks: Cell::new(0),
            pools: RefCell::new(IndexMap::new()),
            in_use: RefCell::new(IndexMap::new()),
            owns: RefCell::new(IndexMap::new()),
            dead: RefCell::new(IndexMap::new()),
            pending: RefCell::new(IndexMap::new()),
        }
    }

    /// Know these works from now on, as well as every one known before that is still a work.
    ///
    /// A sweep taken while a scene loads can miss a work that is back a moment later, and losing
    /// it would make everything it draws read as gone. Works are checked here, not at every poll:
    /// a work's layout and root do not change for as long as it lives. A layout new since the
    /// last track has its tree walked, and every layout is walked again now and then.
    pub fn track(&self, works: impl IntoIterator<Item = i64>) -> Result<()> {
        self.tracks.set(self.tracks.get() + 1);
        let known: IndexMap<i64, i64> = self
            .roots
            .borrow()
            .iter()
            .map(|(&root, layout)| (layout.work, root))
            .collect();
        let mut roots: IndexMap<i64, Layout> = IndexMap::new();
        // `{*works, *known}`, a list then a dict, iterated in Python's hash order (see `PySet`).
        let mut every = PySet::of(works);
        every.update_dict(&known.keys().copied().collect::<Vec<_>>());
        for work in every.iter() {
            let found = self.layout(work)?;
            let Some((root, top, layout)) = found else {
                if let Some(&root) = known.get(&work) {
                    self.bury(work, root);
                } else {
                    let mut pending = self.pending.borrow_mut();
                    pending.shift_remove(&work);
                    pending.insert(work, PENDING_LOOKS);
                    while pending.len() as i64 > PENDING_KEPT {
                        pending.shift_remove_index(0);
                    }
                }
                continue;
            };
            self.pending.borrow_mut().shift_remove(&work);
            roots.insert(root, Layout { work, top, layout });
            self.dead.borrow_mut().shift_remove(&work);
        }
        for layout in roots.values() {
            self.watch_pool_of(layout.work)?;
        }
        for (&root, layout) in &roots {
            let walked = self.walked.borrow().get(&root).copied();
            let stale = self.tracks.get() - walked.unwrap_or(-REWALK_TRACKS) >= REWALK_TRACKS;
            let changed = self.roots.borrow().get(&root) != Some(layout);
            if changed || stale {
                if changed {
                    self.settling.borrow_mut().insert(root, SETTLE_WALKS);
                    self.built.borrow_mut().insert(root, BUILT_POLLS);
                }
                self.walk(root)?;
            }
        }
        *self.roots.borrow_mut() = roots;
        let roots = self.roots.borrow();
        self.contents
            .borrow_mut()
            .retain(|root, _| roots.contains_key(root));
        self.pane_names
            .borrow_mut()
            .retain(|root, _| roots.contains_key(root));
        self.children
            .borrow_mut()
            .retain(|root, _| roots.contains_key(root));
        self.walked
            .borrow_mut()
            .retain(|root, _| roots.contains_key(root));
        Ok(())
    }

    /// Take back layouts built again where they were. Returns whether any were.
    ///
    /// A screen opened from the X menu frees the field's layouts, and closing it builds them again
    /// at the same works and roots: without this, AreaNav's "Littleroot Town" waited for the next
    /// sweep, up to 4 s after the field was back. A different layout built
    /// at a work freed before waits for the sweep, which finds it with whatever else came with
    /// it: the save prompt's list layout, taken back at an old work, had its "YES NO" read before
    /// the sweep found the list that owns them. Cheap enough for every poll: a read or two for
    /// each work kept.
    ///
    /// A work found before its layout was built is taken in here too, once it is, and so is a
    /// work known already that holds another layout now. A pool's slot is built into again and
    /// again, and the level up's panel, built into the battle's pool as the Pokémon grew, was never
    /// read: the work was known, and only a sweep had it looked at again (movie2, 2026-09-26). So
    /// each known work's layout pointer is read here, a word each, from works that sit together
    /// in their pools.
    pub fn revive(&self) -> Result<bool> {
        let mut back = false;
        let built: IndexMap<i64, i64> = self
            .built
            .borrow()
            .iter()
            .filter(|&(_, &left)| left > 1)
            .map(|(&root, &left)| (root, left - 1))
            .collect();
        *self.built.borrow_mut() = built;
        let settling: Vec<(i64, i64)> = self
            .settling
            .borrow()
            .iter()
            .map(|(&root, &left)| (root, left))
            .collect();
        for (root, left) in settling {
            let known = self.roots.borrow().contains_key(&root);
            if !known || left <= 1 {
                self.settling.borrow_mut().shift_remove(&root);
            } else {
                self.settling.borrow_mut().insert(root, left - 1);
            }
            if known {
                let before = self.contents.borrow().get(&root).cloned();
                self.walk(root)?;
                back = back || self.contents.borrow().get(&root) != before.as_ref();
            }
        }
        let known: Vec<(i64, Layout)> = self
            .roots
            .borrow()
            .iter()
            .map(|(&root, &known)| (root, known))
            .collect();
        for (root, known) in known {
            let layout = match self.memory.read(known.work + lyt::WORK_LAYOUT, 4) {
                Ok(word) => u32(&word, 0)?,
                Err(error) if error.is(Kind::RpcError) => continue,
                Err(error) => return Err(error),
            };
            if layout == known.layout {
                continue;
            }
            self.forget(root);
            self.pending.borrow_mut().insert(known.work, PENDING_LOOKS);
        }
        let pending: Vec<(i64, i64)> = self
            .pending
            .borrow()
            .iter()
            .map(|(&work, &left)| (work, left))
            .collect();
        for (work, left) in pending {
            let Some((root, top, layout)) = self.layout(work)? else {
                if left > 1 {
                    self.pending.borrow_mut().insert(work, left - 1);
                } else {
                    self.pending.borrow_mut().shift_remove(&work);
                }
                continue;
            };
            self.pending.borrow_mut().shift_remove(&work);
            self.roots
                .borrow_mut()
                .insert(root, Layout { work, top, layout });
            self.walk(root)?;
            self.settling.borrow_mut().insert(root, SETTLE_WALKS);
            self.built.borrow_mut().insert(root, BUILT_POLLS);
            self.watch_pool_of(work)?;
            back = true;
        }
        let dead: Vec<(i64, i64)> = self
            .dead
            .borrow()
            .iter()
            .map(|(&work, &was)| (work, was))
            .collect();
        for (work, was) in dead {
            let found = self.layout(work)?;
            let Some((root, top, layout)) = found.filter(|found| found.0 == was) else {
                continue;
            };
            self.dead.borrow_mut().shift_remove(&work);
            self.roots
                .borrow_mut()
                .insert(root, Layout { work, top, layout });
            self.walk(root)?;
            self.settling.borrow_mut().insert(root, SETTLE_WALKS);
            self.built.borrow_mut().insert(root, BUILT_POLLS);
            back = true;
        }
        Ok(back)
    }

    /// Stop reading a layout: its work holds another now.
    fn forget(&self, root: i64) {
        self.roots.borrow_mut().shift_remove(&root);
        self.contents.borrow_mut().shift_remove(&root);
        self.pane_names.borrow_mut().shift_remove(&root);
        self.children.borrow_mut().shift_remove(&root);
        self.walked.borrow_mut().shift_remove(&root);
    }

    /// Read the layout pools these table entries are in from now on, as `poll_pools` does.
    ///
    /// An entry is a handle: a message window holds its layout's at +0x14 (spike 64). The pool is
    /// checked against its own shape, so a handle into memory since freed leads nowhere.
    pub fn watch_pools(&self, entries: impl IntoIterator<Item = i64>) -> Result<()> {
        for entry in entries {
            let pool = u32(&self.memory.read(entry, lyt::ENTRY_SIZE)?, lyt::ENTRY_POOL)?;
            if self.pools.borrow().contains_key(&pool) {
                continue;
            }
            let Some(shape) = self.pool(pool)? else {
                continue;
            };
            let (table, capacity) = shape;
            if table <= entry
                && entry < table + capacity * lyt::ENTRY_SIZE
                && py::modulo(entry - table, 8) == 0
            {
                self.pools.borrow_mut().insert(pool, shape);
            }
        }
        Ok(())
    }

    /// Take in every layout built into a watched pool since the last look. Returns whether any
    /// was.
    ///
    /// The battle builds its layouts into a pool of its own in the linear heap, and the rolling
    /// sweep found them a slice at a time, up to 3.5 s late; its message window leads to that pool,
    /// and the pool's table lists every work in it (spike 64).
    pub fn poll_pools(&self) -> Result<bool> {
        let mut known: HashSet<i64> = self.roots.borrow().values().map(|l| l.work).collect();
        let mut taken = false;
        let pools: Vec<i64> = self.pools.borrow().keys().copied().collect();
        for pool in pools {
            let Some(shape) = self.pool(pool)? else {
                self.pools.borrow_mut().shift_remove(&pool);
                self.in_use.borrow_mut().shift_remove(&pool);
                continue;
            };
            // A layout built into a pool takes a slot: while the count holds, the table is not
            // read.
            let in_use = u16(&self.memory.read(pool, lyt::POOL_HEAD)?, lyt::POOL_IN_USE)?;
            if self.in_use.borrow().get(&pool) == Some(&in_use) {
                continue;
            }
            self.in_use.borrow_mut().insert(pool, in_use);
            let (table, capacity) = shape;
            let entries = self.memory.read(table, capacity * lyt::ENTRY_SIZE)?;
            for at in (0..capacity * lyt::ENTRY_SIZE).step_by(lyt::ENTRY_SIZE as usize) {
                let work = u32(&entries, at + lyt::ENTRY_WORK)?;
                if u32(&entries, at + lyt::ENTRY_POOL)? != pool || known.contains(&work) {
                    continue;
                }
                let Some((root, top, layout)) = self.layout(work)? else {
                    continue;
                };
                self.dead.borrow_mut().shift_remove(&work);
                self.roots
                    .borrow_mut()
                    .insert(root, Layout { work, top, layout });
                self.walk(root)?;
                self.built.borrow_mut().insert(root, BUILT_POLLS);
                known.insert(work);
                taken = true;
            }
        }
        Ok(taken)
    }

    /// Watch the pool a work was built in, found from the work alone (spike 99).
    ///
    /// A work's first word points 0x104 into its pool's own work, which comes after the pool's last
    /// slot, and a pool of capacity c starts 0x78 + c * 0x10c before its own work: its header, a
    /// table of c entries from +0x48, 0x30 bytes, then c slots of 0x104. The capacity is tried from
    /// the least that leaves room for this work, and the pool is the one whose header points at
    /// itself and says that capacity. Each pool is looked for once, and read whole at the next
    /// poll, for any layout in it the sweep did not find.
    fn watch_pool_of(&self, work: i64) -> Result<()> {
        let own = u32(&self.memory.read(work, 4)?, 0)? - lyt::WORK_SIZE;
        if self.owns.borrow().contains_key(&own)
            || own <= work
            || py::modulo(own - work, lyt::WORK_SIZE) != 0
        {
            return Ok(());
        }
        self.owns.borrow_mut().insert(own, None);
        for capacity in py::floordiv(own - work, lyt::WORK_SIZE)..=lyt::MAX_POOL {
            let pool = own - capacity * (lyt::WORK_SIZE + lyt::ENTRY_SIZE) - lyt::POOL_SLOTS_OFFSET;
            let head = match self.memory.read(pool, lyt::POOL_HEAD) {
                Ok(head) => head,
                // below the heap: no pool this big
                Err(error) if error.is(Kind::RpcError) => return Ok(()),
                Err(error) => return Err(error),
            };
            if u32(&head, lyt::POOL_SELF)? != pool || u16(&head, lyt::POOL_CAPACITY)? != capacity {
                continue;
            }
            if let Some(shape) = self.pool(pool)? {
                self.owns.borrow_mut().insert(own, Some(pool));
                self.pools.borrow_mut().entry(pool).or_insert(shape);
            }
            return Ok(());
        }
        Ok(())
    }

    /// Watch every layout pool whose header is in these pages, and return those new.
    ///
    /// A screen makes its pool before it builds a layout into it, and the layouts are built into
    /// the pool's slots, not allocated: a sweep of the blocks just allocated sees the pool, and
    /// the works built after only if something reads the pool (spike 110). A header is known by
    /// its table 0x48 into it and itself at +0x08.
    pub fn watch_pools_in(&self, pages: &IndexMap<i64, Vec<u8>>) -> Result<Vec<i64>> {
        let mut new: Vec<i64> = Vec::new();
        for (&base, page) in pages {
            let words: Vec<i64> = page
                .as_chunks::<4>()
                .0
                .iter()
                .map(|word| i64::from(u32::from_le_bytes(*word)))
                .collect();
            let headers: Vec<usize> = words
                .iter()
                .enumerate()
                .filter(|&(i, &word)| word == base + 4 * i as i64 + lyt::POOL_TABLE_OFFSET)
                .map(|(i, _)| i)
                .collect();
            for index in headers {
                let pool = base + 4 * index as i64;
                if self.pools.borrow().contains_key(&pool) {
                    continue;
                }
                if index + 2 < words.len() && words[index + 2] != pool {
                    continue;
                }
                let shape = self.pool(pool)?;
                if let Some(shape) = shape
                    && u32(&self.memory.read(pool, lyt::POOL_HEAD)?, lyt::POOL_SELF)? == pool
                {
                    self.pools.borrow_mut().insert(pool, shape);
                    new.push(pool);
                }
            }
        }
        Ok(new)
    }

    /// Every layout pool watched, with its capacity.
    pub fn pools(&self) -> IndexMap<i64, i64> {
        self.pools
            .borrow()
            .iter()
            .map(|(&pool, &(_table, capacity))| (pool, capacity))
            .collect()
    }

    /// Every tracked layout's work, with its root.
    pub fn works(&self) -> IndexMap<i64, i64> {
        self.roots
            .borrow()
            .iter()
            .map(|(&root, layout)| (layout.work, root))
            .collect()
    }

    /// Forget what was read, so the next questions see the game as it is now.
    pub fn refresh(&self) {
        self.panes.borrow_mut().clear();
        self.states.borrow_mut().clear();
        self.climbs.borrow_mut().clear();
    }

    /// Where a layout is drawn among its screen's others: the higher over the lower. The field's
    /// bottom screen draws its background at 0, the PokéNav's layouts at 1 to 3 and the X menu at
    /// 10 (movie21); the AreaNav draws all of its at 1, its views in their own order.
    pub fn layer(&self, root: i64) -> Result<i64> {
        let Some(layout) = self.roots.borrow().get(&root).copied() else {
            return Ok(0);
        };
        let state = self.state(layout.work)?;
        u16(&state, lyt::WORK_LAYER)
    }

    /// Say which layouts, by their works, their app is not drawing now: the AreaNav's screens it
    /// has left keep their layouts as they were (`gen6.screens.townmap`).
    pub fn put_away(&self, works: impl IntoIterator<Item = i64>) {
        *self.put_away.borrow_mut() = works.into_iter().collect();
    }

    /// The shape's head of a pane, read once until `refresh`.
    pub fn pane(&self, address: i64) -> Result<Vec<u8>> {
        self.head(address, self.shape.head)
    }

    /// The shape's head of a TextBox, through its text fields, read once until `refresh`.
    pub fn textbox(&self, address: i64) -> Result<Vec<u8>> {
        self.head(address, self.shape.textbox_head)
    }

    /// A pane's name as its layout file gives it, NUL-padded, read once until `refresh`.
    pub fn name(&self, address: i64) -> Result<Vec<u8>> {
        Ok(py::slice(
            &self.pane(address)?,
            self.shape.name,
            self.shape.name + lyt::NAME_SIZE,
        )
        .to_vec())
    }

    fn head(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        // A read no longer than the object: in Sun the byte after one may be the GPU's (ADR-0035).
        if !self.panes.borrow().contains_key(&(address, size)) {
            let read = self.memory.read(address, size)?;
            self.panes.borrow_mut().insert((address, size), read);
        }
        Ok(self.panes.borrow()[&(address, size)].clone())
    }

    /// Where `pane` is, or None when it is not under the root of any layout being tracked.
    pub fn placement(&self, pane: i64) -> Result<Option<Placement>> {
        if !self.climbs.borrow().contains_key(&pane) {
            let climbed = self.climb(pane)?;
            self.climbs.borrow_mut().insert(pane, climbed);
        }
        let walked = self.climbs.borrow()[&pane].clone();
        let Some((visible, root, panel, head)) = walked else {
            return Ok(None);
        };
        let Some(layout) = self.roots.borrow().get(&root).copied() else {
            return Ok(None);
        };
        let state = self.state(layout.work)?;
        let alive = byte(&state, lyt::WORK_ALIVE)? == 1;
        if byte(&state, lyt::WORK_LISTED)? == 1 && !self.built.borrow().contains_key(&root) {
            self.listed.borrow_mut().insert((layout.work, root));
        }
        let drawn = alive
            && byte(&state, lyt::WORK_DRAWN)? == 1
            && !self.put_away.borrow().contains(&layout.work);
        let projection = f32(&state, lyt::WORK_PROJECTION)?;
        let mut top = if projection < 0.0 {
            -2.0 / projection >= SCREEN_SPLIT
        } else {
            layout.top
        };
        let root_y = f32(&head, self.shape.y)?;
        if root_y.abs() >= OTHER_VIEW {
            top = true;
        }
        Ok(Some(Placement {
            shown: visible && drawn,
            top,
            root,
            panel,
            visible,
            drawn,
            alive,
        }))
    }

    /// A work's head through its projection, read once until `refresh`.
    fn state(&self, work: i64) -> Result<Vec<u8>> {
        if !self.states.borrow().contains_key(&work) {
            let read = self.memory.read(work, lyt::WORK_PROJECTION + 4)?;
            self.states.borrow_mut().insert(work, read);
        }
        Ok(self.states.borrow()[&work].clone())
    }

    fn climb(&self, pane: i64) -> Result<Option<Climb>> {
        let mut root = pane;
        let mut head = self.pane(pane)?;
        let mut panel = pane;
        let mut visible = byte(&head, self.shape.alpha)? > 0;
        for _ in 0..MAX_DEPTH {
            visible = visible && byte(&head, self.shape.flags)? & lyt::VISIBLE != 0;
            let parent = u32(&head, lyt::PARENT)?;
            if parent == 0 {
                return Ok(Some((visible, root, panel, head)));
            }
            head = self.pane(parent)?;
            let name = self.names.of_vtable(u32(&head, 0)?)?;
            if !name.is_some_and(|name| name.starts_with(lyt::CLASS_PREFIX)) {
                return Ok(None);
            }
            (panel, root) = (root, parent);
        }
        Ok(None)
    }

    /// Has this layout been taken off the system's list of layouts to draw, which it was seen on?
    /// Then it is not drawn, though every flag of its own says it is: the battle's command buttons
    /// under its move list (spike 188). Asked by touch mode alone: as drawn, the move list came
    /// back to the screen each turn, and its words were read out whole before its cursor said the
    /// move under it (night regression, movie5).
    pub fn taken_off(&self, root: i64) -> Result<bool> {
        let Some(layout) = self.roots.borrow().get(&root).copied() else {
            return Ok(false);
        };
        if !self.listed.borrow().contains(&(layout.work, root)) {
            return Ok(false);
        }
        let state = self.state(layout.work)?;
        Ok(byte(&state, lyt::WORK_LISTED)? == 0)
    }

    /// The roots of every layout tracked, whose works were still works at the last sweep.
    ///
    /// Python returns a `set`; the `IndexSet` here is in the order Python iterates it (`PySet`).
    pub fn roots(&self) -> IndexSet<i64> {
        self.roots_set().iter().collect()
    }

    /// `roots()` as the `PySet` Python's set is, for the callers that go on to `-` or `&` it.
    pub fn roots_set(&self) -> PySet {
        let roots: Vec<i64> = self.roots.borrow().keys().copied().collect();
        PySet::of_dict(&roots)
    }

    /// The roots of layouts whose works stopped being works, which `revive` takes back if they
    /// are built again where they were.
    ///
    /// Python returns a `set`; the `IndexSet` here is in the order Python iterates it (`PySet`).
    pub fn buried(&self) -> IndexSet<i64> {
        let dead: Vec<i64> = self.dead.borrow().values().copied().collect();
        PySet::of(dead).iter().collect()
    }

    /// The names of the panes under a tracked root, depth first in child order, the order its
    /// layout file lists them in.
    pub fn pane_names(&self, root: i64) -> Vec<String> {
        self.pane_names
            .borrow()
            .get(&root)
            .cloned()
            .unwrap_or_default()
    }

    /// Every pane under a tracked root, with its class name, as its tree was last walked.
    pub fn tree(&self, root: i64) -> IndexMap<i64, String> {
        self.contents
            .borrow()
            .get(&root)
            .cloned()
            .unwrap_or_default()
    }

    /// The children of a pane under a tracked root, in order, as its tree was last walked.
    pub fn children(&self, root: i64, pane: i64) -> Vec<i64> {
        self.children
            .borrow()
            .get(&root)
            .and_then(|family| family.get(&pane))
            .cloned()
            .unwrap_or_default()
    }

    /// Every pane of the named class in the layouts tracked, as their trees were last walked.
    ///
    /// Python returns a `set`; the `IndexSet` here is in the order Python iterates it (`PySet`).
    pub fn panes(&self, name: &str) -> IndexSet<i64> {
        let panes: Vec<i64> = self
            .contents
            .borrow()
            .values()
            .flat_map(|contents| contents.iter())
            .filter(|(_, pane_name)| pane_name.as_str() == name)
            .map(|(&pane, _)| pane)
            .collect();
        PySet::of(panes).iter().collect()
    }

    fn bury(&self, work: i64, root: i64) {
        let mut dead = self.dead.borrow_mut();
        dead.shift_remove(&work);
        dead.insert(work, root);
        while dead.len() as i64 > DEAD_KEPT {
            dead.shift_remove_index(0);
        }
    }

    /// Read every pane under `root` into `contents`, depth first in child order.
    fn walk(&self, root: i64) -> Result<()> {
        let mut contents: IndexMap<i64, String> = IndexMap::new();
        let mut stack: Vec<i64> = vec![root];
        let mut names: Vec<String> = Vec::new();
        let mut family: IndexMap<i64, Vec<i64>> = IndexMap::new();
        while !stack.is_empty() && (contents.len() as i64) < MAX_PANES {
            let pane = stack.pop().expect("the stack is not empty");
            if contents.contains_key(&pane) {
                continue;
            }
            let head = self.memory.read(pane, self.shape.name + lyt::NAME_SIZE)?;
            let Some(name) = self.names.of_vtable(u32(&head, 0)?)? else {
                continue;
            };
            if !name.starts_with(lyt::CLASS_PREFIX) {
                continue;
            }
            contents.insert(pane, name);
            let label = py::slice_from(&head, self.shape.name);
            let label = label.split(|&b| b == 0).next().unwrap_or_default();
            names.push(py::decode_ascii(label));
            let mut children: Vec<i64> = Vec::new();
            let mut node = u32(&head, lyt::CHILD_LIST)?;
            while node != 0 && node != pane + lyt::CHILD_LIST && (children.len() as i64) < MAX_PANES
            {
                children.push(node - lyt::NODE);
                node = u32(&self.memory.read(node, 4)?, 0)?;
                if children.contains(&(node - lyt::NODE)) {
                    break;
                }
            }
            family.insert(pane, children.clone());
            stack.extend(children.iter().rev());
        }
        let children: IndexMap<i64, Vec<i64>> = family
            .into_iter()
            .map(|(pane, children)| {
                let kept = children
                    .into_iter()
                    .filter(|child| contents.contains_key(child))
                    .collect();
                (pane, kept)
            })
            .collect();
        self.contents.borrow_mut().insert(root, contents);
        self.pane_names.borrow_mut().insert(root, names);
        self.children.borrow_mut().insert(root, children);
        self.walked.borrow_mut().insert(root, self.tracks.get());
        Ok(())
    }

    /// (table, capacity) for memory laid out as a layout pool, or None.
    fn pool(&self, pool: i64) -> Result<Option<(i64, i64)>> {
        let head = self.memory.read(pool, lyt::POOL_HEAD)?;
        let (table, end) = (
            u32(&head, lyt::POOL_TABLE)?,
            u32(&head, lyt::POOL_TABLE_END)?,
        );
        let capacity = u16(&head, lyt::POOL_CAPACITY)?;
        if table != pool + lyt::POOL_TABLE_OFFSET || !(0 < capacity && capacity <= lyt::MAX_POOL) {
            return Ok(None);
        }
        Ok(if end - table == capacity * lyt::ENTRY_SIZE {
            Some((table, capacity))
        } else {
            None
        })
    }

    /// (root pane, on the top screen, layout object) for a work laid out as one, or None.
    fn layout(&self, work: i64) -> Result<Option<(i64, bool, i64)>> {
        let head = self.memory.read(work, lyt::WORK_HEAD)?;
        let drawinfo = self.names.of_vtable(u32(&head, lyt::WORK_DRAWINFO)?)?;
        if drawinfo.as_deref() != Some(lyt::DRAWINFO_CLASS) {
            return Ok(None);
        }
        let layout = u32(&head, lyt::WORK_LAYOUT)?;
        if self.names.of_object(layout)?.as_deref() != Some(lyt::LAYOUT_CLASS) {
            return Ok(None);
        }
        let body = self.memory.read(layout, lyt::LAYOUT_WIDTH + 4)?;
        // A layout object is made before its root pane: taken in then, Sun's save prompt's cursor
        // had a root of zero, and was never taken in again (sun_3.ctm, spike 123).
        let root = u32(&body, lyt::LAYOUT_ROOT)?;
        if root == 0 {
            return Ok(None);
        }
        let width = f32(&body, lyt::LAYOUT_WIDTH)?;
        Ok(Some((root, width >= lyt::TOP_SCREEN_WIDTH, layout)))
    }
}
