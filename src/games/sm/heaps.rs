//! gfl2's heaps, and the only memory of Sun's that pa3ds may sweep: their small allocations.
//!
//! ```text
//! table   the start of the application heap (`HEAP_SPAN`'s): 8 bytes an id, and at +0x00 of
//!         each a pointer to the heap with that id, or zero
//! heap    `gfl2::heap::CTRExpHeap`: +0x04 u32 its id, and at +0x38 the head of the `nn::fnd`
//!         heap it embeds
//! head    +0x00 its kind, 'EXPH' for an expanded heap    +0x08 -> the next sibling's head
//!         +0x0c -> its first child's head    +0x18 -> the first byte it hands out
//!         +0x1c -> its end    +0x2c -> its first used block (an expanded heap's), +0x30 its last
//! block   +0x00 u16 'UD'    +0x04 u32 the size of what follows the block's 16 bytes
//!         +0x08 -> the previous used block    +0x0c -> the next used block
//! ```
//!
//! Sun keeps nearly everything in linear memory from 0x30000000, and the GPU's own buffers sit there
//! too, in the gaps between heaps and inside some of them. Azahar writes back a buffer the GPU has
//! cleared whenever anything else reads it, and that asserted and took the emulator down, on Vulkan
//! and on OpenGL alike: sweeping the linear memory whole, sweeping every heap the table lists, and
//! sweeping even the two heaps every dialogue object was in, where a 529 KB block of the field's
//! turned out to be one (spike 111). The Azahar it was found on runs Vulkan.
//!
//! So nothing is swept whole. The heaps under the dialogue's are walked by their heads, each one's
//! used blocks by their 16-byte headers, and only blocks of `SMALL` or less are read, to the byte. A
//! message window is 0x114 bytes, a TextBox 0x120 and a talk window 0xe4, and a buffer the GPU
//! renders into is none of those sizes. The table and every heap are checked before they are walked:
//! an entry's object is a `CTRExpHeap` by RTTI with its own index as its id, and a block's header says
//! 'UD'; a walk that meets anything else stops there, and the next measurement walks it again.
//!
//! **Walked whole only now and then; watched in between** (ADR-0034, as ORAS's are). gfl2's heads are
//! ORAS's, and a heap object's +0x08 counts its blocks in use (spike 123), so between walks the heaps
//! under the walk's roots are watched for the blocks they have just allocated (`pa3ds.core.heaps`): a
//! block larger than `SMALL` known by its header alone and never read, heap 0x21's own blocks left
//! unwalked, and only a new small block swept (`to_sweep`). What may be read is every small block
//! the last walk found and every one allocated since, less any a newer block has been taken over
//! (`SmallBlocks`): a small block freed and its memory taken again, as part of a large block that may
//! be the GPU's, is out of bounds the look it is taken.
//!
//! Bound by:
//! - ADR-0035: in Sun only small allocations are read, a block of 4 KB or less, and to the byte.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{u16, u32};
use crate::core::discovery::HEAP_SPAN;
use crate::core::heaps::Block;
use crate::core::memory::Memory;
use crate::core::rpc::{Rpc, joined, reads_of};
use crate::core::rtti::ClassNames;
use crate::py::{self, Error, Log, PySet, Result};

pub const CLASS: &str = "N4gfl24heap10CTRExpHeapE";
pub const ENTRY: i64 = 8;
/// The table's first page: Sun's highest id in the movie was 0x32.
pub const ENTRIES: i64 = 0x80;
pub const ID: i64 = 0x04;
/// The `nn::fnd` heap's head inside a `CTRExpHeap`.
pub const HEAD: i64 = 0x38;
/// 'EXPH' as a head stores it, a little-endian word.
pub const EXPH: &[u8] = b"HPXE";
pub const NEXT: i64 = 0x08;
pub const CHILD: i64 = 0x0C;
pub const USED: i64 = 0x2C;
pub const HEAD_SIZE: i64 = USED + 4;
/// 'UD', the mark at the start of a used block's header.
pub const USED_BLOCK: i64 = 0x5544;
pub const BLOCK_SIZE: i64 = 0x04;
pub const BLOCK_NEXT: i64 = 0x0C;
pub const BLOCK_HEADER: i64 = 0x10;
/// The largest block read. The objects the dialogue reader needs are 0x120 bytes or less, and a
/// message's text is read through its buffer, not found by a sweep.
pub const SMALL: i64 = 0x1000;
/// Where the heap objects under the walk's roots are, which the watch keeps to: 0x3000010c to
/// 0x34296650 on Route 1 (spike 123).
pub const SPAN: (i64, i64) = (0x30000000, 0x34400000);
/// A heap object's count of its blocks in use, as ORAS's is: it matched a walk of the list in all
/// 169 heaps under the walk's roots on Route 1 (spike 123).
pub const COUNT: i64 = 0x08;
/// Reads a walk makes before calling back, as a sweep's chunk: about 16 ms.
pub const CHUNK: i64 = 512;
pub const MAX_HEAPS: i64 = 1024;
/// Heads and blocks walked before giving up, in case a torn read loops.
pub const MAX_BLOCKS: i64 = 0x10000;
/// The heaps every message window, talk window and TextBox of the movie was under, at each of six
/// moments from Kukui's call to the player's bedroom: 5, the application's, and 0x0b, which the
/// field builds. A window built under any other is not found, so a screen whose dialogue goes unsaid
/// may be using a heap of its own: measure it, and add it here.
pub const DIALOGUE_HEAPS: [i64; 2] = [0x05, 0x0B];
/// Heaps walked for the heaps inside them, but whose own blocks are not: 0x21 is the field's, and
/// holds its actors' heap (0x45) and its area's files' (0x44). Its own 4,219 blocks on Route 1 were
/// the rendering engine's animations, textures and command buffers, followed a header at a time,
/// and four of every five round trips of a walk (sun_4.ctm, 2026-09-26); over 58 captures of spike
/// 112's menus, language choice, keyboard and bedroom, no class pa3ds reads was in one.
pub const UNREAD: [i64; 1] = [0x21];

/// `blocks: Callable[[], Collection[tuple[int, int]]]`: the small blocks that may be read now,
/// (start, end) each.
///
/// The collection is shared, so that a reader asking whether it is the one it took last time
/// (`blocks() is not source`) is `!Rc::ptr_eq`: `SmallBlocks::spans` hands out the same one until a
/// walk or an allocation changes it.
pub type Blocks = Rc<dyn Fn() -> Rc<Vec<(i64, i64)>>>;

/// One walk of the heaps: where each small used block is, and what it held as it was walked.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Walk {
    /// Each small block's body, (start, end).
    pub spans: Vec<(i64, i64)>,
    /// Each body's bytes by its start, but for any whose read failed.
    pub bodies: IndexMap<i64, Vec<u8>>,
    /// The wanted heaps' objects, which the watch adopts (`pa3ds.core.heaps`).
    pub roots: Vec<i64>,
}

impl Walk {
    /// `Walk(spans, bodies)`: `roots` left to its default, none.
    pub fn new(spans: Vec<(i64, i64)>, bodies: IndexMap<i64, Vec<u8>>) -> Self {
        Walk {
            spans,
            bodies,
            roots: Vec::new(),
        }
    }
}

/// What a step of a heap's walk has yet to read: the body of the block before it, (start, end),
/// and the header at the block, which is zero for none.
type Step = (i64, Option<(i64, i64)>);

/// Where Sun's small allocations are now, under the heaps its dialogue lives in.
///
/// Shared, as the session hands it to the closure that walks: the one thing that changes is in a
/// `Cell`, and every method takes `&self`.
pub struct Heaps {
    rpc: Rc<dyn Rpc>,
    names: Rc<ClassNames>,
    log: Log,
    /// The ids in the order Python iterates the `frozenset` they are, which is the order the
    /// walk reads them in (`PySet`).
    wanted: Vec<i64>,
    unread: Vec<i64>,
    said: Cell<Option<i64>>,
}

impl Heaps {
    /// `Heaps(rpc, names)`: `log` left to its default, which says nothing, and `wanted` and
    /// `unread` to `DIALOGUE_HEAPS` and `UNREAD`.
    pub fn new(rpc: Rc<dyn Rpc>, names: Rc<ClassNames>) -> Self {
        let dialogue: Vec<i64> = PySet::of(DIALOGUE_HEAPS).iter().collect();
        Heaps::with(rpc, names, Rc::new(|_line| {}), &dialogue, &UNREAD)
    }

    /// `names` should read exactly what it is asked for, as `RpcMemory` does: every read here
    /// is to the byte, since the byte after a block may be the GPU's.
    ///
    /// `wanted` and `unread` are `frozenset`s in Python, only ever read in their numbers' order
    /// (`listed`), so the order they are passed in does not matter.
    pub fn with(
        rpc: Rc<dyn Rpc>,
        names: Rc<ClassNames>,
        log: Log,
        wanted: &[i64],
        unread: &[i64],
    ) -> Self {
        Heaps {
            rpc,
            names,
            log,
            wanted: wanted.to_vec(),
            unread: unread.to_vec(),
            said: Cell::new(None),
        }
    }

    /// What each small used block holds, as (start, end), under the wanted heaps.
    pub fn spans(&self) -> Result<Vec<(i64, i64)>> {
        Ok(self.walk(None, CHUNK)?.spans)
    }

    /// Every small used block under the wanted heaps, and what each holds, read as the walk
    /// goes. `between` is called every `chunk` reads or so, so a caller can keep polling.
    pub fn walk(&self, between: Option<&dyn Fn()>, chunk: i64) -> Result<Walk> {
        let table = self.rpc.read(HEAP_SPAN.0, ENTRY * ENTRIES)?;
        let heads = self.heads(Some(&table))?;
        let unread: HashSet<i64> = self.listed(&table, &self.unread)?.into_iter().collect();
        let expanded: Vec<i64> = heads
            .iter()
            .filter(|(head, kind)| kind.as_slice() == EXPH && !unread.contains(*head))
            .map(|(&head, _kind)| head)
            .collect();
        let mut sorted: Vec<i64> = heads.keys().copied().collect();
        sorted.sort();
        let walked = self.small_blocks(&expanded, &sorted, between, chunk)?;
        let walked = Walk {
            spans: walked.spans,
            bodies: walked.bodies,
            roots: self
                .listed(&table, &self.wanted)?
                .into_iter()
                .map(|head| head - HEAD)
                .collect(),
        };
        if Some(expanded.len() as i64) != self.said.get() {
            self.said.set(Some(expanded.len() as i64));
            let kib: i64 = walked.spans.iter().map(|&(low, high)| high - low).sum();
            (self.log)(&format!(
                "heaps: {} to sweep, {} small blocks, {} KiB",
                expanded.len(),
                walked.spans.len(),
                kib >> 10
            ));
        }
        Ok(walked)
    }

    /// The heads of the heaps with these ids, each checked to be a heap with its own id.
    fn listed(&self, table: &[u8], ids: &[i64]) -> Result<Vec<i64>> {
        let mut found: Vec<i64> = Vec::new();
        let mut ids = ids.to_vec();
        ids.sort();
        for index in ids {
            let heap = u32(table, index * ENTRY)?;
            if heap != 0
                && heap & 3 == 0
                && self.names.of_object(heap)?.as_deref() == Some(CLASS)
                && u32(&self.rpc.read(heap + ID, 4)?, 0)? == index
            {
                found.push(heap + HEAD);
            }
        }
        Ok(found)
    }

    /// Every heap head under the wanted heaps, and its kind, a level of the tree a read.
    pub(crate) fn heads(&self, table: Option<&[u8]>) -> Result<IndexMap<i64, Vec<u8>>> {
        let fetched;
        let table = match table {
            Some(table) => table,
            None => {
                fetched = self.rpc.read(HEAP_SPAN.0, ENTRY * ENTRIES)?;
                &fetched
            }
        };
        let roots = self.listed(table, &self.wanted)?;
        let mut heads: IndexMap<i64, Vec<u8>> = IndexMap::new();
        // Heads whose next sibling has been followed. A wanted heap can be under another, as 0x0b
        // is under 3: met first as a root, its siblings were never followed, and heap 3's walk lost
        // the 34 heaps after it (spike 113).
        let mut siblings: HashSet<i64> = HashSet::new();
        // A root's siblings are other heaps, not under it: from a root only its children follow.
        let mut level: Vec<(i64, bool)> = roots.iter().map(|&root| (root, true)).collect();
        while !level.is_empty() && (heads.len() as i64) < MAX_HEAPS {
            let reads: Vec<(i64, i64)> = level
                .iter()
                .map(|&(head, _root)| (head, HEAD_SIZE))
                .collect();
            let replies = self.rpc.read_each(&reads)?;
            let mut below: Vec<(i64, bool)> = Vec::new();
            for &(head, root) in &level {
                let data: &[u8] = replies.get(&head).map_or(&[], Vec::as_slice);
                if data.len() as i64 != HEAD_SIZE {
                    continue;
                }
                let mut links: Vec<i64> = Vec::new();
                if !heads.contains_key(&head) {
                    heads.insert(head, data[..4].to_vec());
                    links.push(u32(data, CHILD)?);
                }
                if !root && !siblings.contains(&head) {
                    siblings.insert(head);
                    links.push(u32(data, NEXT)?);
                }
                below.extend(
                    links
                        .into_iter()
                        .filter(|&link| link != 0 && link & 3 == 0)
                        .map(|link| (link, false)),
                );
            }
            level = below;
        }
        Ok(heads)
    }

    /// The small used blocks of these expanded heaps and their bodies, walked side by side, a
    /// block of every heap a round. A block holding another heap's head is that heap's, walked
    /// as its own.
    ///
    /// A block's body is read in the round after its header, with the next block's header when
    /// that follows it, as most do: one read a block rather than a header and then a body. Every
    /// read is still exactly inside a small block or a header.
    ///
    /// A read costs the emulator about 28 us and a round about 140 us more (sun_4.ctm,
    /// 2026-09-26). Walking each list from both ends as well halved the rounds and took as long,
    /// since a body read going backwards cannot share a read with the header before it.
    fn small_blocks(
        &self,
        expanded: &[i64],
        heads: &[i64],
        between: Option<&dyn Fn()>,
        chunk: i64,
    ) -> Result<Walk> {
        let used: Vec<(i64, i64)> = expanded.iter().map(|&head| (head + USED, 4)).collect();
        let replies = self.rpc.read_each(&used)?;
        // Each heap's next header to read, and the body before it still to be read.
        let mut at: IndexMap<i64, Step> = IndexMap::new();
        for &head in expanded {
            if replies.get(&(head + USED)).map_or(0, Vec::len) == 4 {
                at.insert(head, (u32(&replies[&(head + USED)], 0)?, None));
            }
        }
        let mut spans: Vec<(i64, i64)> = Vec::new();
        let mut bodies: IndexMap<i64, Vec<u8>> = IndexMap::new();
        let (mut walked, mut sent) = (0, 0);
        while !at.is_empty() && walked < MAX_BLOCKS {
            let reads: Vec<(i64, i64)> = at
                .values()
                .flat_map(|&(block, body)| reads(block, body))
                .collect();
            if let Some(between) = between
                && sent >= chunk
            {
                between();
                sent = 0;
            }
            sent += reads.len() as i64;
            let replies = self.rpc.read_each(&reads)?;
            let mut following: IndexMap<i64, Step> = IndexMap::new();
            for (&head, &(block, body)) in &at {
                let (held, data) = read_back(&replies, block, body);
                if let (Some(body), Some(held)) = (body, held) {
                    bodies.insert(body.0, held);
                }
                let Some(data) = data else {
                    continue;
                };
                if u16(&data, 0)? != USED_BLOCK {
                    continue;
                }
                walked += 1;
                let span = (
                    block + BLOCK_HEADER,
                    block + BLOCK_HEADER + u32(&data, BLOCK_SIZE)?,
                );
                let small = if span.1 - span.0 <= SMALL && !holds(heads, span) {
                    spans.push(span);
                    Some(span)
                } else {
                    None
                };
                following.insert(head, (u32(&data, BLOCK_NEXT)?, small));
            }
            at = following
                .into_iter()
                .filter(|(_head, step)| step.0 != 0 || step.1.is_some())
                .collect();
        }
        Ok(Walk::new(spans, bodies))
    }
}

/// What a step of a heap's walk reads: the body still to be read and the header at `block`,
/// zero for none, as one span when the header follows the body.
fn spans(block: i64, body: Option<(i64, i64)>) -> Vec<(i64, i64)> {
    if let Some(body) = body
        && body.1 == block
    {
        return vec![(body.0, block + BLOCK_HEADER)];
    }
    [
        body,
        if block != 0 {
            Some((block, block + BLOCK_HEADER))
        } else {
            None
        },
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// A step's spans as reads of `MAX_READ` or less.
fn reads(block: i64, body: Option<(i64, i64)>) -> Vec<(i64, i64)> {
    reads_of(&spans(block, body))
}

/// A step's body and header out of the replies to its reads, each None where a read of it
/// failed or it was not asked for.
fn read_back(
    replies: &IndexMap<i64, Vec<u8>>,
    block: i64,
    body: Option<(i64, i64)>,
) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    let parts: Vec<Option<Vec<u8>>> = spans(block, body)
        .into_iter()
        .map(|(low, high)| joined(replies, low, high))
        .collect();
    let Some(body) = body else {
        return (None, parts.first().cloned().flatten());
    };
    if body.1 == block {
        let Some(whole) = parts[0].as_ref() else {
            return (None, None);
        };
        return (
            Some(py::slice(whole, 0, -BLOCK_HEADER).to_vec()),
            Some(py::slice_from(whole, -BLOCK_HEADER).to_vec()),
        );
    }
    (
        parts[0].clone(),
        if parts.len() > 1 {
            parts[1].clone()
        } else {
            None
        },
    )
}

/// Whether any of these heads, sorted, is inside this block.
fn holds(heads: &[i64], body: (i64, i64)) -> bool {
    let at = heads.partition_point(|&head| head < body.0);
    at < heads.len() && heads[at] < body.1
}

/// Whether a heap object is one of these ids, whose own blocks the watch leaves unwalked, as
/// the walk does. Told by the id, since heap 0x21 is made after the watch starts.
///
/// A read that fails is a panic carrying the error: the callback answers a bool, where Python's
/// raised through the watch that called it.
pub fn unread(memory: Rc<dyn Memory>, ids: &[i64]) -> Rc<dyn Fn(i64) -> bool> {
    let ids: Vec<i64> = ids.to_vec();
    Rc::new(move |heap| {
        let id = memory
            .read(heap + ID, 4)
            .and_then(|data| u32(&data, 0))
            .unwrap_or_else(|error| std::panic::panic_any(error));
        ids.contains(&id)
    })
}

/// What of a new block may be swept: all of a small one, none of a larger one, which may be
/// the GPU's.
pub fn to_sweep(block: Block) -> Option<(i64, i64)> {
    if 0 < block.size && block.size <= SMALL {
        Some(block.memory())
    } else {
        None
    }
}

/// The small blocks that may be read: every one the last walk found, and every one allocated
/// since, less those a newer block has been taken over (spike 123).
///
/// Shared: the session's reads, its watch and the readers' bounds all ask it, so the mutable
/// parts are in `RefCell` and every method takes `&self`.
pub struct SmallBlocks {
    state: RefCell<SmallState>,
}

struct SmallState {
    /// Start -> end of each block's body.
    spans: IndexMap<i64, i64>,
    sorted: Option<Rc<Vec<(i64, i64)>>>,
}

impl Default for SmallBlocks {
    fn default() -> Self {
        SmallBlocks::new()
    }
}

impl SmallBlocks {
    pub fn new() -> Self {
        SmallBlocks {
            state: RefCell::new(SmallState {
                spans: IndexMap::new(),
                sorted: Some(Rc::new(Vec::new())),
            }),
        }
    }

    /// Take a walk's blocks as all there is. Returns the headers of the blocks it no longer
    /// found, which the heaps' watch should forget as `allocated`'s: the field's layouts, freed
    /// under the save prompt and taken again as they were after it, were never handed over, and
    /// the prompt's blocks left over them kept the X menu's cursor unread until the next walk
    /// (sun_3.ctm, 2026-09-27).
    ///
    /// Python's is the difference of two key views, a set, in its hash order; here it is in the
    /// order the blocks were taken in, which only the watch's `forget`, a set, reads.
    pub fn walked(&self, spans: &[(i64, i64)]) -> Vec<i64> {
        let mut state = self.state.borrow_mut();
        let walked: IndexMap<i64, i64> = spans.iter().copied().collect();
        let gone = state
            .spans
            .keys()
            .filter(|start| !walked.contains_key(*start))
            .map(|start| start - BLOCK_HEADER)
            .collect();
        state.spans = walked;
        state.sorted = None;
        gone
    }

    /// Take in the blocks the heaps have just allocated: a small one may be read, and whatever
    /// any of them lies over, header and all, may not. Returns the headers of the blocks taken
    /// out, which the heaps' watch should forget: one taken again later where it was, as it was,
    /// is new again, and none of it may be read until it is.
    pub fn allocated(&self, blocks: &[Block]) -> Vec<i64> {
        if blocks.is_empty() {
            return Vec::new();
        }
        let mut dropped: Vec<i64> = Vec::new();
        let spans = self.spans();
        let starts: Vec<i64> = spans.iter().map(|&(start, _end)| start).collect();
        let mut state = self.state.borrow_mut();
        for block in blocks {
            let (low, high) = (block.address, block.memory().1);
            let mut at = (starts.partition_point(|&start| start <= low) as i64 - 1).max(0) as usize;
            while at < spans.len() && spans[at].0 < high {
                let (start, end) = spans[at];
                if end > low && state.spans.shift_remove(&start).is_some() {
                    dropped.push(start - BLOCK_HEADER);
                }
                at += 1;
            }
        }
        for block in blocks {
            if let Some(small) = to_sweep(*block) {
                state.spans.insert(small.0, small.1);
            }
        }
        state.sorted = None;
        dropped
            .into_iter()
            .filter(|header| !state.spans.contains_key(&(header + BLOCK_HEADER)))
            .collect()
    }

    /// Whether this is still a block that may be read.
    pub fn holds(&self, span: (i64, i64)) -> bool {
        self.state.borrow().spans.get(&span.0) == Some(&span.1)
    }

    /// Whether `size` bytes at `address` lie inside one block that may be read.
    pub fn may_read(&self, address: i64, size: i64) -> bool {
        let spans = self.spans();
        let at = spans.partition_point(|&span| span <= (address, 0xFFFFFFFF)) as i64 - 1;
        at >= 0 && address + size <= spans[at as usize].1
    }

    /// Every block that may be read, (start, end), sorted.
    ///
    /// The same collection comes back until a walk or an allocation changes it (`Blocks`).
    pub fn spans(&self) -> Rc<Vec<(i64, i64)>> {
        let mut state = self.state.borrow_mut();
        if state.sorted.is_none() {
            let mut sorted: Vec<(i64, i64)> = state.spans.iter().map(|(&s, &e)| (s, e)).collect();
            sorted.sort();
            state.sorted = Some(Rc::new(sorted));
        }
        state.sorted.clone().expect("set above")
    }
}

/// Memory that reads only inside the small blocks the last walk found, and reads the rest as
/// zeros (ADR-0035).
///
/// Sun's layout readers follow pointers from a layout's work down its tree of panes to their
/// words, and a pointer out of memory since freed can lead anywhere, the GPU's buffers included.
/// Zeros are what the readers already pass over: a pane with no class is not a pane, and a
/// TextBox whose words have no length says nothing. Over sun_3.ctm the reads it refused were the
/// works of layouts since freed, which `Layouts` checks again at every sweep and poll (spike 112).
pub struct InBlocks {
    memory: Rc<dyn Memory>,
    blocks: Blocks,
    state: RefCell<InState>,
}

#[derive(Default)]
struct InState {
    source: Option<Rc<Vec<(i64, i64)>>>,
    spans: Vec<(i64, i64)>,
    starts: Vec<i64>,
}

impl InBlocks {
    pub fn new(memory: Rc<dyn Memory>, blocks: Blocks) -> Self {
        InBlocks {
            memory,
            blocks,
            state: RefCell::new(InState::default()),
        }
    }
}

impl Memory for InBlocks {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        // The heaps' watch takes a block out of bounds the look a larger one is taken over it,
        // with no sweep to follow (`SmallBlocks`): its spans are taken in whenever they change.
        let inside = {
            let mut state = self.state.borrow_mut();
            let current = (self.blocks)();
            let changed = match &state.source {
                Some(source) => !Rc::ptr_eq(&current, source),
                None => true,
            };
            if changed {
                let source = (self.blocks)();
                let mut spans: Vec<(i64, i64)> = source.to_vec();
                spans.sort();
                state.starts = spans.iter().map(|&(start, _end)| start).collect();
                state.spans = spans;
                state.source = Some(source);
            }
            let at = state.starts.partition_point(|&start| start <= address) as i64 - 1;
            at >= 0 && address + size <= state.spans[at as usize].1
        };
        if inside {
            return self.memory.read(address, size);
        }
        if size < 0 {
            return Err(Error::value("negative count"));
        }
        Ok(vec![0; size as usize])
    }
}
