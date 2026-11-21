//! Discovery: find the game's objects by class, in the heap.
//!
//! A polymorphic object names its class through its vtable, so an object is found by sweeping
//! memory for the vtables of the classes wanted (`rtti`). The heap is swept a chunk at a time with
//! a callback between chunks so polling carries on (spike 59), and where it is is measured again
//! now and then: it grows after boot and when a battle starts.
//!
//! A sweep reads the whole heap, and costs about a quarter of a second. What the game has allocated
//! since is found where it allocated it (`pa3ds.core.heaps`), and only that memory is swept
//! (`Discovery.sweep_spans`, ADR-0034). Until then a sweep between whole sweeps read the spans
//! around what earlier sweeps had found (spikes 69 and 70).
//!
//! The linear heap is not swept. It is eight times bigger, and what the runtime needs from it, the
//! battle's layouts, is reached from a message window's handle into their pool (spike 64). A
//! rolling sweep of it, a slice at a time, was a sixth of the runtime's time until 2026-09-17.
//!
//! Nothing here knows the game. What to look for, and what to do with what was found, is the
//! profile's.
//!
//! Bound by:
//! - ADR-0003: objects are found by their class and checked by a validator, never at an address
//!   written down.
//! - ADR-0034: a sweep may be partial: a reader adds what it finds and forgets nothing for its
//!   absence.

use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::memory::Memory;
use crate::core::modules::{Module, Modules};
use crate::core::rpc::{Rpc, joined, reads_of};
use crate::core::rtti::{built_on, find_derived, find_objects, find_vtables, typeinfo_of};
use crate::core::snapshot::{DEFAULT_GAP, PAGE, PROBE_STEP, merge, probe, range_of, sweep};
use crate::py::PySet;
use crate::py::{self, Result};

/// Where the 3DS maps every executable, with room to spare. No ASLR, so this is platform, not game.
/// `regions` measures where inside it the code actually ends.
pub const CODE_SPAN: (i64, i64) = (0x00100000, 0x01000000);

/// Where the application heap is, and so where the message windows live. Nothing has been found
/// past `0x09000000` (spike 10), and every measurement since has ended by `0x08df0000`. The span is
/// probed every couple of seconds, so its size is a cost: `0x10000000` was 8,192 probes a
/// measurement and visibly slowed polling; this is 1,024.
pub const HEAP_SPAN: (i64, i64) = (0x08000000, 0x09000000);

/// The heap allocator's tag at the start of every block, followed by the block's size (spike 14):
/// a block of known size is found, and checked still to be there, by the two words.
pub const ALLOCATION: i64 = 0x5544;

/// Where the linear heap is. The battle builds its layouts and every TextBox they hold here, and
/// nothing in the heap points at them but the handles into their pool (spikes 45 and 63).
pub const LINEAR_SPAN: (i64, i64) = (0x14000000, 0x1C000000);

pub fn within(span: (i64, i64), address: i64) -> bool {
    span.0 <= address && address < span.1
}

/// What one sweep of the heap found.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Found {
    /// The heap's pages, as swept.
    pub pages: IndexMap<i64, Vec<u8>>,
    /// Object address -> class name.
    pub heap: IndexMap<i64, String>,
    /// Whether only some of the heap was swept: the blocks the game has just allocated. What such
    /// a sweep finds is new, and what it does not find may be anywhere else (spike 110).
    pub partial: bool,
}

impl Found {
    /// The objects of these classes found.
    pub fn of(&self, classes: &[&str]) -> Vec<i64> {
        self.heap
            .iter()
            .filter(|(_, name)| classes.contains(&name.as_str()))
            .map(|(&address, _)| address)
            .collect()
    }

    /// `size` bytes at `address` out of the pages swept, or None where the sweep did not read them
    /// all: what an object found holds, at no read more.
    pub fn read(&self, address: i64, size: i64) -> Option<&[u8]> {
        let start = address - address.rem_euclid(PAGE);
        let page = self.pages.get(&start)?;
        if address + size > start + page.len() as i64 {
            return None;
        }
        Some(&page[(address - start) as usize..(address - start + size) as usize])
    }

    /// Every word-aligned address in the heap's pages that holds one of these addresses.
    pub fn holders(&self, targets: &[i64]) -> Result<Vec<i64>> {
        Ok(find_objects(&self.pages, targets)?.into_keys().collect())
    }
}

/// `walk`: reads the heap for a profile that says where it may be read, calling back as it goes.
pub type Walk = Rc<dyn Fn(&dyn Fn()) -> Result<IndexMap<i64, Vec<u8>>>>;

/// Sweeps the heap for objects of the classes asked for.
pub struct Discovery {
    rpc: Rc<dyn Rpc>,
    between: Rc<dyn Fn()>,
    chunk: i64,
    walk: Option<Walk>,
    modules: Option<Modules>,
    /// Module name -> where in it each class wanted has its vtable, as an offset from its
    /// base: read from a module the first time it is loaded, and the same wherever it loads.
    offsets: IndexMap<String, IndexMap<i64, String>>,
    /// The modules the last look listed. Python's is a `frozenset`, only ever asked if it holds a
    /// module, which a `Vec` answers as well (a `Module` has no hash here).
    listed: Vec<Module>,
    /// Whether the last look left a module unread because it was new: look again soon.
    pub unsettled: bool,
    classes: Vec<String>,
    /// Each of `bases` -> its typeinfo and those of the executable's classes built on it, any of
    /// which a module's class may name as its own base.
    bases: IndexMap<String, PySet>,
    names: IndexMap<i64, String>,
    /// Where the heap was when last measured.
    pub heap: Vec<(i64, i64)>,
    /// The heap's blocks that held anything when last measured, `PROBE_STEP` apart.
    blocks: IndexSet<i64>,
}

impl Discovery {
    /// A discovery with Python's defaults: a `chunk` of 512 and no `walk`.
    pub fn new(
        rpc: Rc<dyn Rpc>,
        image: &[u8],
        base: i64,
        classes: &[&str],
        between: Rc<dyn Fn()>,
    ) -> Result<Self> {
        Discovery::with(rpc, image, base, classes, between, 512, None, &[])
    }

    /// `image` is the executable from `base`, to find vtables in. `between` is called between a
    /// sweep's chunks. `walk` reads the heap for a profile that says where it may be read: every
    /// span it may read, by its start, read to the byte and never a page around it, calling
    /// back as it goes. Without it, the heap is wherever `HEAP_SPAN` holds anything, read a page
    /// at a time. `bases` are classes of `classes` whose objects are found with those of every
    /// class built on them, under their name: an app's touch bar is of a class of its module's
    /// own, which pa3ds has no name for (spike 189).
    pub fn with(
        rpc: Rc<dyn Rpc>,
        image: &[u8],
        base: i64,
        classes: &[&str],
        between: Rc<dyn Fn()>,
        chunk: i64,
        walk: Option<Walk>,
        bases: &[&str],
    ) -> Result<Self> {
        let mut roots: IndexMap<String, PySet> = IndexMap::new();
        for &name in bases {
            let mut typeinfos = PySet::new();
            for vtable in find_vtables(image, base, name, 0, None)? {
                typeinfos.add(typeinfo_of(image, base, vtable)?);
            }
            roots.insert(name.to_string(), typeinfos);
        }
        // Each of `bases` -> its typeinfo and those of the executable's classes built on it, any
        // of which a module's class may name as its own base.
        let mut built: IndexMap<String, PySet> = IndexMap::new();
        for (name, typeinfos) in &roots {
            built.insert(name.clone(), built_on(image, base, typeinfos)?);
        }
        let mut names: IndexMap<i64, String> = IndexMap::new();
        for &name in classes {
            for vtable in find_vtables(image, base, name, 0, None)? {
                names.insert(vtable, name.to_string());
            }
        }
        for (name, typeinfos) in &roots {
            for vtable in find_derived(image, base, typeinfos, None)? {
                names.insert(vtable, name.clone());
            }
        }
        Ok(Discovery {
            rpc,
            between,
            chunk,
            walk,
            modules: None,
            offsets: IndexMap::new(),
            listed: Vec::new(),
            unsettled: false,
            classes: classes.iter().map(|name| name.to_string()).collect(),
            bases: built,
            names,
            heap: Vec::new(),
            blocks: IndexSet::new(),
        })
    }

    /// Those of these classes that have no vtable in the executable.
    pub fn missing(&self, classes: &[&str]) -> Vec<String> {
        let known: std::collections::HashSet<&str> =
            self.names.values().map(String::as_str).collect();
        classes
            .iter()
            .filter(|name| !known.contains(*name))
            .map(|name| name.to_string())
            .collect()
    }

    /// Know the vtables of the classes wanted in every module the game has loaded, and return
    /// the classes found at a vtable not known before: learned for the first time, or a module
    /// loaded somewhere new.
    ///
    /// A DLL's classes are not in the executable: `field::MapBlock`'s vtable is at `0x007fd7c8`,
    /// past the `0x00698000` the code ends at on the title screen, so the runtime's sweeps found
    /// every other object on the field and never a block, while a spike reading the code once
    /// the field was up found all of them (2026-09-23). A module is loaded after whatever is
    /// loaded beneath it, so where it lands depends on that: the field's and a battle's were at
    /// the bottom of the modules' memory every time, the name entry's keyboard at three
    /// addresses (spike 140). So the modules are listed at every look (`modules`, a few reads),
    /// a module is read the first time it is seen and where each class has its vtable in it is
    /// kept, and a module seen again somewhere new has its vtables there with no read at all.
    ///
    /// Until spike 140, the code past where the executable's image ended was read again whenever a
    /// class was missing, and then at every app start (spike 138), about 30 ms each; and a module
    /// loaded below that end was never read: started on the field, the image took the field's
    /// module in, and a battle's, loaded where the field's had been, never had its gauges found.
    /// A vtable once learned is kept: a reader checks what it reads through one.
    ///
    /// **A module is read only once a look has listed it just as the look before did.** The game
    /// maps it and Azahar fixes it in one call, freeing the part past what it keeps, and reading
    /// it as its header first appeared read that part as it was freed: Azahar crashed twice in
    /// four replays of movie5 as Wally's battle loaded its module (2026-09-30). The look after,
    /// a poll later, finds it settled, its magic 'FIXD' where the game had it fixed.
    pub fn learn(&mut self, span: (i64, i64)) -> Result<Vec<String>> {
        if self.modules.is_none() {
            self.modules = Some(Modules::new(Rc::new(self.rpc.clone()), span));
        }
        let mut learned: Vec<String> = Vec::new();
        let listed = self.modules.as_mut().expect("set above").loaded()?;
        self.unsettled = false;
        for module in &listed {
            let offsets = self.offsets.get(&module.name).cloned();
            if offsets.is_none() && !self.listed.contains(module) {
                self.unsettled = true;
                continue;
            }
            let offsets = match offsets {
                Some(offsets) => offsets,
                None => {
                    let image = sweep(
                        &*self.rpc,
                        &[(module.base, module.base + module.size)],
                        true,
                        self.chunk,
                        0.0,
                        Some(&*self.between),
                    )?
                    .read(module.base, module.size)?;
                    let mut offsets: IndexMap<i64, String> = IndexMap::new();
                    for name in &self.classes {
                        for vtable in find_vtables(&image, module.base, name, 0, Some(span))? {
                            offsets.insert(vtable - module.base, name.clone());
                        }
                    }
                    for (vtable, name) in self.built_on(&image, module.base, Some(span))? {
                        offsets.insert(vtable - module.base, name);
                    }
                    self.offsets.insert(module.name.clone(), offsets.clone());
                    offsets
                }
            };
            for (offset, name) in &offsets {
                let vtable = module.base + offset;
                if !self.names.contains_key(&vtable) {
                    self.names.insert(vtable, name.clone());
                    if !learned.contains(name) {
                        learned.push(name.clone());
                    }
                }
            }
        }
        self.listed = listed;
        Ok(learned)
    }

    /// The vtables in a module's image of the classes built on one of `bases`, by its name.
    fn built_on(
        &self,
        image: &[u8],
        base: i64,
        code: Option<(i64, i64)>,
    ) -> Result<IndexMap<i64, String>> {
        let mut out: IndexMap<i64, String> = IndexMap::new();
        for (name, typeinfos) in &self.bases {
            for vtable in find_derived(image, base, typeinfos, code)? {
                out.insert(vtable, name.clone());
            }
        }
        Ok(out)
    }

    /// Measure where the heap is, before a whole sweep. Returns whether memory has turned up
    /// since the last measurement, told block by block rather than by the heap's merged spans:
    /// the first battle's turned up between blocks less than a megabyte apart (spike 70).
    ///
    /// The spans themselves only ever grow, and each block reaches one block further either side.
    /// A block's first page reading blank does not mean the game has given the memory back: the
    /// main menu's layout works sat at `0x081ff594`, in the block the title screen had kept its
    /// resource in, and the moment the title freed it the block dropped out of the measurement and
    /// the menu was never swept (main_menu.ctm, spike 72).
    pub fn measure_heap(&mut self) -> Result<bool> {
        if self.walk.is_some() {
            // Every sweep walks the heap afresh, which is its measurement.
            return Ok(false);
        }
        let blocks = probe(
            &*self.rpc,
            HEAP_SPAN,
            PROBE_STEP,
            Some(self.chunk),
            Some(&*self.between),
        )?;
        // `blocks` is a set Python iterates in hash order; `merge` sorts what it is given, so
        // the order cannot reach anything.
        let mut spans: Vec<(i64, i64)> = self.heap.clone();
        spans.extend(blocks.iter().map(|&block| widened(block)));
        self.heap = merge(&spans, DEFAULT_GAP);
        let new = blocks.iter().any(|block| !self.blocks.contains(block));
        self.blocks = blocks;
        Ok(new)
    }

    /// Sweep the whole heap, as last measured.
    ///
    /// With `walk`, every sweep is the walk: it reads every block of the heap it may read, in
    /// about the reads it takes to find them, and says where the heap is now.
    pub fn sweep(&mut self) -> Result<Found> {
        let pages = if self.walk.is_some() {
            self.walked()?
        } else {
            self.read(&self.heap.clone())?
        };
        let heap = self.found(&pages)?;
        Ok(Found {
            pages,
            heap,
            partial: false,
        })
    }

    /// Sweep these spans alone: the memory the game has just allocated, which is where its new
    /// objects are. A partial find, which says nothing of what it did not read.
    ///
    /// With `walk`, each span is read to the byte, as the walk reads, and never a page around
    /// it: in Sun the byte after a block may be the GPU's (spike 123).
    pub fn sweep_spans(&mut self, spans: &[(i64, i64)]) -> Result<Found> {
        if self.walk.is_some() {
            let pages = self.exact(&merge(spans, 0))?;
            let heap = self.found(&pages)?;
            return Ok(Found {
                pages,
                heap,
                partial: true,
            });
        }
        // Only where spans share a page: joined by `merge`'s megabyte, movie4's new blocks held
        // 138 MB and their sweeps read 951 MB, 33 s of the replay (spike 133).
        let paged: Vec<(i64, i64)> = spans
            .iter()
            .map(|&(low, high)| (low & !(PAGE - 1), high))
            .collect();
        let merged = merge(&paged, 0);
        if merged.is_empty() {
            return Ok(Found {
                partial: true,
                ..Found::default()
            });
        }
        let pages = self.read(&merged)?;
        let heap = self.found(&pages)?;
        Ok(Found {
            pages,
            heap,
            partial: true,
        })
    }

    /// These spans' bytes by start, read to the byte, polling between chunks; a span whose
    /// read failed is left out.
    fn exact(&self, spans: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        let reads = reads_of(spans);
        let mut replies: IndexMap<i64, Vec<u8>> = IndexMap::new();
        for start in range_of(0, reads.len() as i64, self.chunk)? {
            if start != 0 {
                (self.between)();
            }
            replies.extend(
                self.rpc
                    .read_each(py::slice(&reads, start, start + self.chunk))?,
            );
        }
        let mut out: IndexMap<i64, Vec<u8>> = IndexMap::new();
        for &(low, high) in spans {
            if let Some(data) = joined(&replies, low, high) {
                out.insert(low, data);
            }
        }
        Ok(out)
    }

    fn found(&self, pages: &IndexMap<i64, Vec<u8>>) -> Result<IndexMap<i64, String>> {
        // `set(self._names)`: `find_objects` goes vtable by vtable, so the set's order is the
        // order of what is found, which a whole sweep's log line counts in (sun_10.ctm's tape).
        let names: Vec<i64> = self.names.keys().copied().collect();
        let vtables: Vec<i64> = PySet::of_dict(&names).iter().collect();
        let mut out: IndexMap<i64, String> = IndexMap::new();
        for (address, vtable) in find_objects(pages, &vtables)? {
            out.insert(address, self.names[&vtable].clone());
        }
        Ok(out)
    }

    /// The heap as `walk` reads it, which is where it is now.
    fn walked(&mut self) -> Result<IndexMap<i64, Vec<u8>>> {
        let walk = self.walk.clone().expect("assert self._walk is not None");
        let pages = walk(&*self.between)?;
        let spans: Vec<(i64, i64)> = pages
            .iter()
            .map(|(&at, data)| (at, at + data.len() as i64))
            .collect();
        self.heap = merge(&spans, 0);
        Ok(pages)
    }

    /// Sweep the heap whole for these classes alone, at once: spikes' (150, 293).
    pub fn sweep_for(&mut self, classes: &[&str]) -> Result<Vec<i64>> {
        // A set comprehension, whose order is the order of the objects returned.
        let vtables: Vec<i64> = PySet::of(
            self.names
                .iter()
                .filter(|(_, name)| classes.contains(&name.as_str()))
                .map(|(&vtable, _)| vtable),
        )
        .iter()
        .collect();
        let pages = if self.walk.is_some() {
            self.walked()?
        } else {
            self.read(&self.heap.clone())?
        };
        Ok(find_objects(&pages, &vtables)?.into_keys().collect())
    }

    /// These spans' bytes by address, whole pages of the heap.
    fn read(&self, spans: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        Ok(sweep(
            &*self.rpc,
            spans,
            false,
            self.chunk,
            0.0,
            Some(&*self.between),
        )?
        .pages)
    }
}

/// A block of the heap with a block either side. A block's first page reading blank does not
/// mean the game has given its memory back, so a measurement is a floor, not a map.
fn widened(block: i64) -> (i64, i64) {
    (
        (block - PROBE_STEP).max(HEAP_SPAN.0),
        (block + 2 * PROBE_STEP).min(HEAP_SPAN.1),
    )
}
