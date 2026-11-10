//! Reading memory by address, whether from the running emulator or from a test.
//!
//! Bound by:
//! - ADR-0004: a poll reads in a phase of whole pages, pipelined, starting from the pages it read
//!   last time.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::rpc::{MAX_READ, Rpc};
use crate::py::{Error, Kind, Result};

/// A cached page: one RPC read, which is why it matches the read cap.
pub const PAGE: i64 = MAX_READ;
pub const ADDRESS_SPACE: i64 = 1 << 32;
/// Where a process's memory begins, with its code: nothing is mapped below it, so a read there is
/// a null pointer followed. Azahar logged each as `unmapped ReadBlock @ 0x00000000`, every 0.2 s
/// through every session, as the dialogue reader's pages were fetched ahead (2026-09-29).
pub const LOWEST: i64 = 0x00100000;

/// Whether a read is of memory no process has, which reads as zeros without being sent.
pub fn nowhere(address: i64, size: i64) -> bool {
    address < LOWEST || address + size > ADDRESS_SPACE
}

/// Anything that reads bytes by address: `PageCache`, `RpcMemory`, a `Snapshot`, or a fake.
pub trait Memory {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>>;
}

/// A cache a poll reads through in phases: [`PageCache`] or [`ExactCache`].
///
/// Python's `with cache.phase(name):` is [`phase`]; `begin` and `end` are its two halves, kept
/// apart so the trait can be a `dyn` one.
pub trait Cache: Memory {
    /// Start the phase: what `with cache.phase(name)` does before its body.
    fn begin(&self, name: &str) -> Result<()>;
    /// End the phase begun last, body raising or not.
    fn end(&self, name: &str);
    /// Many reads at once, keyed by address; a read that failed is left out.
    fn read_each(&self, reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>>;
}

/// `with cache.phase(name): body`. A phase that fails to begin runs no body and is not ended,
/// as a `contextmanager` whose setup raises never reaches its `finally`.
pub fn phase<C: Cache + ?Sized, R>(
    cache: &C,
    name: &str,
    body: impl FnOnce() -> Result<R>,
) -> Result<R> {
    struct Ending<'a, C: Cache + ?Sized>(&'a C, &'a str);
    impl<C: Cache + ?Sized> Drop for Ending<'_, C> {
        fn drop(&mut self) {
            self.0.end(self.1);
        }
    }
    cache.begin(name)?;
    let _ending = Ending(cache, name);
    body()
}

/// Reads of any size from the running emulator, in as few packets as the read cap allows.
///
/// `Rpc.read` refuses anything over one packet; readers should not have to know that a message
/// longer than 512 characters needs two.
pub struct RpcMemory {
    rpc: Rc<dyn Rpc>,
}

impl RpcMemory {
    pub fn new(rpc: Rc<dyn Rpc>) -> Self {
        RpcMemory { rpc }
    }
}

impl Memory for RpcMemory {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        if nowhere(address, size) {
            return Ok(vec![0; size.max(0) as usize]);
        }
        let data = if size <= MAX_READ {
            self.rpc.read(address, size)?
        } else {
            let whole = size / MAX_READ * MAX_READ;
            let starts: Vec<i64> = (address..address + whole)
                .step_by(MAX_READ as usize)
                .collect();
            let replies = self.rpc.read_many(&starts, MAX_READ)?;
            let missing = starts
                .iter()
                .filter(|start| replies.get(*start).map_or(0, Vec::len) as i64 != MAX_READ)
                .count();
            if missing > 0 {
                return Err(Error::new(
                    Kind::RpcError,
                    format!("{missing} blocks unread from {address:#010x}"),
                ));
            }
            let mut data: Vec<u8> = starts
                .iter()
                .flat_map(|start| replies[start].clone())
                .collect();
            if size > whole {
                data.extend(self.rpc.read(address + whole, size - whole)?);
            }
            data
        };
        if data.len() as i64 != size {
            return Err(Error::new(
                Kind::RpcError,
                format!(
                    "asked for {size} bytes at {address:#010x}, got {}",
                    data.len()
                ),
            ));
        }
        Ok(data)
    }
}

/// Reads served from whole pages, fetched pipelined, and fresh for one phase of polling.
///
/// A read sent alone is answered before the next is sent, and the readers made a great many:
/// 12.7 a dialogue poll, 45 to 90 a screen text poll, 490 a layout track, most of them within a
/// few pages (spike 62). Pipelined, a read costs about a third as much (spike 2), and one page
/// answers every read inside it.
///
/// Pages are kept only inside `phase`, and a phase starts with none, so nothing is ever read
/// older than the phase reading it. A phase begins by fetching, in one pipelined batch, the pages
/// the same phase read last time: a poll reads mostly what it read the poll before. Reads outside
/// any phase are not cached at all.
///
/// Not for what the GPU draws: its pages do not start where it does, and the emulator flushes it
/// wrong for them. `RpcMemory` reads it from its own first byte (`amie.AmiePage`).
pub struct PageCache {
    rpc: Rc<dyn Rpc>,
    state: RefCell<PageState>,
}

#[derive(Default)]
struct PageState {
    pages: HashMap<i64, Vec<u8>>,
    /// The pages each phase in progress has read, outermost first.
    phases: Vec<BTreeSet<i64>>,
    /// Phase name -> the pages it read the last time it ran.
    used: IndexMap<String, Vec<i64>>,
}

impl PageCache {
    pub fn new(rpc: Rc<dyn Rpc>) -> Self {
        PageCache {
            rpc,
            state: RefCell::new(PageState::default()),
        }
    }

    /// Every page the phases read the last time each ran: what the readers were looking at.
    ///
    /// A snapshot of these is the cheapest honest picture of a moment the readers failed on
    /// (ADR-0019), and it is already scrubbed — it holds the pages a reader touched and nothing
    /// else.
    pub fn recent(&self) -> Vec<i64> {
        let state = self.state.borrow();
        let all: BTreeSet<i64> = state.used.values().flatten().copied().collect();
        all.into_iter().collect()
    }

    /// Fetch these pages in one pipelined batch. `strict` raises for any left unanswered.
    fn fetch(&self, bases: &[i64], strict: bool) -> Result<()> {
        if bases.is_empty() {
            return Ok(());
        }
        let replies = self.rpc.read_many(bases, PAGE)?;
        let mut state = self.state.borrow_mut();
        for &base in bases {
            match replies.get(&base) {
                Some(data) if data.len() as i64 == PAGE => {
                    state.pages.insert(base, data.clone());
                }
                _ if strict => {
                    return Err(Error::new(
                        Kind::RpcError,
                        format!("page {base:#010x} unread"),
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn clear_outside_phases(&self) {
        let mut state = self.state.borrow_mut();
        if state.phases.is_empty() {
            state.pages.clear();
        }
    }

    fn touch(&self, bases: impl Iterator<Item = i64> + Clone) {
        for touched in self.state.borrow_mut().phases.iter_mut() {
            touched.extend(bases.clone());
        }
    }
}

fn bases_of(address: i64, size: i64) -> impl Iterator<Item = i64> + Clone {
    let first = address & !(PAGE - 1);
    (first..address + size).step_by(PAGE as usize)
}

impl Memory for PageCache {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        if size <= 0 {
            return Ok(Vec::new());
        }
        if nowhere(address, size) {
            // A pointer out of freed or torn memory, or a null one. It reads as zeros, as
            // unmapped memory does, so every check a reader makes turns it away. Raising
            // instead aborted a whole screen text poll over one dead pane's parent, a replay of
            // movie3's summary on 2026-09-17, and would again every poll it stayed in a tree.
            return Ok(vec![0; size as usize]);
        }
        let first = address & !(PAGE - 1);
        self.clear_outside_phases();
        let missing: Vec<i64> = {
            let state = self.state.borrow();
            bases_of(address, size)
                .filter(|base| !state.pages.contains_key(base))
                .collect()
        };
        if !missing.is_empty() {
            self.fetch(&missing, true)?;
        }
        self.touch(bases_of(address, size));
        let data: Vec<u8> = {
            let state = self.state.borrow();
            bases_of(address, size)
                .flat_map(|base| state.pages[&base].iter().copied())
                .collect()
        };
        self.clear_outside_phases();
        let start = (address - first) as usize;
        Ok(data[start..start + size as usize].to_vec())
    }
}

impl Cache for PageCache {
    fn begin(&self, name: &str) -> Result<()> {
        let used = {
            let mut state = self.state.borrow_mut();
            state.pages.clear();
            state.phases.push(BTreeSet::new());
            state.used.get(name).cloned().unwrap_or_default()
        };
        // The Python fetches inside the phase's `try`, so a fetch that raises still ends it.
        self.fetch(&used, false).inspect_err(|_| self.end(name))
    }

    fn end(&self, name: &str) {
        let mut state = self.state.borrow_mut();
        let touched = state.phases.pop().unwrap_or_default();
        state
            .used
            .insert(name.to_string(), touched.into_iter().collect());
        if state.phases.is_empty() {
            state.pages.clear();
        }
    }

    /// Many reads at once, keyed by address, their pages fetched in one batch; a read whose
    /// page went unanswered, or that no request can carry, is left out.
    fn read_each(&self, reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        let wanted: Vec<(i64, i64)> = reads
            .iter()
            .copied()
            .filter(|&(at, size)| size > 0 && !nowhere(at, size))
            .collect();
        self.clear_outside_phases();
        let missing: Vec<i64> = {
            let state = self.state.borrow();
            let pages: BTreeSet<i64> = wanted
                .iter()
                .flat_map(|&(at, size)| bases_of(at, size))
                .collect();
            pages
                .into_iter()
                .filter(|base| !state.pages.contains_key(base))
                .collect()
        };
        self.fetch(&missing, false)?;
        let mut out = IndexMap::new();
        for &(at, size) in &wanted {
            let first = at & !(PAGE - 1);
            self.touch(bases_of(at, size));
            let state = self.state.borrow();
            if bases_of(at, size).all(|base| state.pages.contains_key(&base)) {
                let data: Vec<u8> = bases_of(at, size)
                    .flat_map(|base| state.pages[&base].iter().copied())
                    .collect();
                let start = (at - first) as usize;
                out.insert(at, data[start..start + size as usize].to_vec());
            }
        }
        self.clear_outside_phases();
        Ok(out)
    }
}

/// Which reads [`ExactCache`] may fetch ahead.
pub type Within = Rc<dyn Fn(i64, i64) -> bool>;

/// `PageCache` for a game where no byte may be read past what is asked: Sun, whose large blocks
/// may be the GPU's (ADR-0035).
///
/// A phase begins by fetching, in one pipelined batch, exactly the reads the same phase made last
/// time, and answers a read at the same address from them; anything else is read alone, to the
/// byte. Looking at Sun's 170 heaps, a read each, took 12.5 ms sent one after another (spike 123).
/// Reads outside any phase are never cached.
///
/// With `within`, only a read it allows is fetched ahead: a screen poll's reads are inside the
/// small blocks that may be read now, and one of last poll's may since have been taken over by a
/// block that may not be, the GPU's (ADR-0035). Such a read is read alone, if a reader still
/// asks for it, through whatever bounds that reader keeps.
pub struct ExactCache {
    rpc: Rc<dyn Rpc>,
    within: Option<Within>,
    exact: RpcMemory,
    state: RefCell<ExactState>,
}

#[derive(Default)]
struct ExactState {
    held: IndexMap<i64, Vec<u8>>,
    /// The reads the phase in progress has made, or None outside one.
    reads: Option<IndexSet<(i64, i64)>>,
    /// Phase name -> the reads it made the last time it ran.
    used: IndexMap<String, IndexSet<(i64, i64)>>,
}

impl ExactCache {
    pub fn new(rpc: Rc<dyn Rpc>, within: Option<Within>) -> Self {
        ExactCache {
            exact: RpcMemory::new(rpc.clone()),
            rpc,
            within,
            state: RefCell::new(ExactState::default()),
        }
    }
}

impl Memory for ExactCache {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        let in_phase = self.state.borrow().reads.is_some();
        if !in_phase || nowhere(address, size) {
            return self.exact.read(address, size);
        }
        let held = self.state.borrow().held.get(&address).cloned();
        let data = match held {
            Some(held) if held.len() as i64 >= size => held[..size as usize].to_vec(),
            _ => self.exact.read(address, size)?,
        };
        if 0 < size
            && size <= PAGE
            && let Some(reads) = self.state.borrow_mut().reads.as_mut()
        {
            reads.insert((address, size));
        }
        Ok(data)
    }
}

impl Cache for ExactCache {
    fn begin(&self, name: &str) -> Result<()> {
        let mut reads = self
            .state
            .borrow()
            .used
            .get(name)
            .cloned()
            .unwrap_or_default();
        if let Some(within) = &self.within {
            reads.retain(|&(at, size)| within(at, size));
        }
        let wanted: Vec<(i64, i64)> = reads.iter().copied().collect();
        let replies = if wanted.is_empty() {
            IndexMap::new()
        } else {
            self.rpc.read_each(&wanted)?
        };
        let mut state = self.state.borrow_mut();
        state.held = wanted
            .iter()
            .filter_map(|&(at, size)| match replies.get(&at) {
                Some(data) if data.len() as i64 == size => Some((at, data.clone())),
                _ => None,
            })
            .collect();
        state.reads = Some(IndexSet::new());
        Ok(())
    }

    fn end(&self, name: &str) {
        let mut state = self.state.borrow_mut();
        let reads = state.reads.take().unwrap_or_default();
        state.used.insert(name.to_string(), reads);
        state.held.clear();
    }

    /// Many reads at once, keyed by address: those fetched as the phase started answered from
    /// that, the rest sent in one pipelined burst; a read that failed is left out. Inside a
    /// phase, what the burst fetched answers the phase's reads after it.
    fn read_each(&self, reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        let mut out = IndexMap::new();
        let mut missing = Vec::new();
        {
            let state = self.state.borrow();
            for &(at, size) in reads {
                if nowhere(at, size) {
                    continue;
                }
                let held = if state.reads.is_some() {
                    state.held.get(&at)
                } else {
                    None
                };
                match held {
                    Some(held) if held.len() as i64 >= size => {
                        out.insert(at, held[..size as usize].to_vec());
                    }
                    _ => missing.push((at, size)),
                }
            }
        }
        if !missing.is_empty() {
            let fetched = self.rpc.read_each(&missing)?;
            let mut state = self.state.borrow_mut();
            let in_phase = state.reads.is_some();
            for (at, data) in fetched {
                // Not kept for the next phase: a burst is the marks of the blocks just allocated,
                // which the next look does not read again, and thousands of them fetched ahead
                // cost a look more than they answered (sun_4.ctm, 2026-09-27).
                if in_phase {
                    state.held.insert(at, data.clone());
                }
                out.insert(at, data);
            }
        }
        Ok(out)
    }
}
