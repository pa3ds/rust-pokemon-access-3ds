//! Snapshots: captured memory, replayable through readers in tests.
//!
//! A snapshot is sparse: only the pages that hold data are kept, which doubles as a readable map at
//! page resolution. Do not sweep the whole address space to find out where that data is. It is
//! 786,000 packets and a running emulator stops answering partway through, for good — `regions()`
//! probes for the map in a couple of seconds, and `sweep` then runs over what it found.
//!
//! **Capture with emulation paused.** A sweep takes seconds, during which a running game advances
//! about sixty frames a second, so an unpaused capture is a smear across hundreds of moments rather
//! than a picture of one. Diffing smeared captures buries every small signal: two captures of one
//! unchanging state came back 114,000 bytes apart. Paused, repeat captures are byte-identical, and
//! `churn` is how you check before trusting one.
//!
//! Raw snapshots hold the player's save data. They belong under `capture_dir()`, outside the
//! checkout, and never in the repo; only scrubbed snapshots are committed.
//!
//! Bound by:
//! - ADR-0029: raw captures hold the player's save and go outside the checkout.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use indexmap::{IndexMap, IndexSet};
#[cfg(windows)]
use windows_sys::Win32::Foundation::SYSTEMTIME;
#[cfg(windows)]
use windows_sys::Win32::System::SystemInformation::GetLocalTime;

use crate::core::memory::Memory;
use crate::core::rpc::{MAX_READ, Rpc};
use crate::py::{self, Error, Result, time};

/// Snapshot page size. One page is one RPC read, which is why it matches the read cap.
pub const PAGE: i64 = MAX_READ;

/// Every address the 3DS process could map, from zero past the top of VRAM.
pub const FULL_SPAN: (i64, i64) = (0x00000000, 0x30000000);

/// How far apart `regions` samples. Sixteen kilobytes is 49,000 probes over the whole
/// address space, a couple of seconds, and fine enough to separate the blocks that exist.
pub const PROBE_STEP: i64 = 0x4000;

/// Pages per batch of requests. Measured the hard way: a sweep asked for all at once gets part of
/// the way through and then the emulator stops answering at all.
pub const CHUNK_PAGES: i64 = 2048;

/// Seconds between batches, to let the server drain.
pub const CHUNK_PAUSE: f64 = 0.01;

/// Ignore holes smaller than a megabyte when merging. The heap is pitted with unused pages, and
/// reporting each run of them as its own region would be a map nobody can narrow a sweep with.
pub const DEFAULT_GAP: i64 = 0x100000;

/// Where raw snapshots live: `pa3ds/captures` under `%LOCALAPPDATA%`, outside the repo, because
/// they hold save data.
pub fn capture_dir() -> Result<PathBuf> {
    let Some(base) = std::env::var_os("LOCALAPPDATA") else {
        return Err(Error::key(py::repr("LOCALAPPDATA")));
    };
    Ok(PathBuf::from(base).join("pa3ds").join("captures"))
}

/// `range(low, high, step)`, which raises for a step of zero and counts down for a negative one.
#[track_caller]
pub(crate) fn range_of(low: i64, high: i64, step: i64) -> Result<Vec<i64>> {
    if step == 0 {
        return Err(Error::value("range() arg 3 must not be zero"));
    }
    let mut out = Vec::new();
    let mut at = low;
    while if step > 0 { at < high } else { at > high } {
        out.push(at);
        at += step;
    }
    Ok(out)
}

/// `time.strftime("%Y%m%d-%H%M%S")`: the local time, which Rust's standard library cannot give.
#[cfg(not(windows))]
fn stamp() -> String {
    std::process::Command::new("date")
        .arg("+%Y%m%d-%H%M%S")
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_default()
}

#[cfg(windows)]
fn stamp() -> String {
    // SAFETY: GetLocalTime fills the structure it is given and fails never.
    let now: SYSTEMTIME = unsafe {
        let mut now: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut now);
        now
    };
    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        now.wYear, now.wMonth, now.wDay, now.wHour, now.wMinute, now.wSecond
    )
}

/// Captured pages, keyed by page-aligned address.
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub pages: IndexMap<i64, Vec<u8>>,
}

impl Snapshot {
    pub fn nbytes(&self) -> i64 {
        self.pages.len() as i64 * PAGE
    }

    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// `bool(snapshot)`, which Python answers from `__len__`.
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// Contiguous captured spans as half-open (start, end) pairs. The readable map.
    pub fn ranges(&self) -> Vec<(i64, i64)> {
        let mut out: Vec<(i64, i64)> = Vec::new();
        let bases: BTreeSet<i64> = self.pages.keys().copied().collect();
        for base in bases {
            match out.last_mut() {
                Some(last) if base == last.1 => *last = (last.0, base + PAGE),
                _ => out.push((base, base + PAGE)),
            }
        }
        out
    }

    /// Yield (address, mine, theirs) for every byte that differs across both snapshots.
    ///
    /// Each item is a `Result` because Python's `zip(strict=True)` raises partway through the
    /// generator, after the differences before it were yielded, when two pages differ in length.
    pub fn diff<'a>(
        &'a self,
        other: &'a Snapshot,
    ) -> impl Iterator<Item = Result<(i64, i64, i64)>> + 'a {
        let bases: BTreeSet<i64> = self
            .pages
            .keys()
            .chain(other.pages.keys())
            .copied()
            .collect();
        let zeros = vec![0u8; PAGE as usize];
        bases
            .into_iter()
            .flat_map(move |base| {
                let mine = self.pages.get(&base).map_or(&zeros[..], Vec::as_slice);
                let theirs = other.pages.get(&base).map_or(&zeros[..], Vec::as_slice);
                let mut items: Vec<Result<(i64, i64, i64)>> = Vec::new();
                for (offset, (&a, &b)) in mine.iter().zip(theirs).enumerate() {
                    if a != b {
                        items.push(Ok((base + offset as i64, a as i64, b as i64)));
                    }
                }
                if mine.len() != theirs.len() {
                    items.push(Err(Error::value(format!(
                        "zip() argument 2 is {} than argument 1",
                        if theirs.len() < mine.len() {
                            "shorter"
                        } else {
                            "longer"
                        }
                    ))));
                }
                items
            })
            .scan(false, |failed, item| {
                if *failed {
                    return None;
                }
                *failed = item.is_err();
                Some(item)
            })
    }

    /// Write `<stamp>_<label>.idx` (page addresses) and `.dat`. Returns the .idx path.
    pub fn save(&self, label: &str, directory: Option<&Path>) -> Result<PathBuf> {
        let target = match directory {
            Some(directory) => directory.to_path_buf(),
            None => capture_dir()?,
        };
        fs::create_dir_all(&target)?;
        let stamp = stamp();
        let idx_path = target.join(format!("{stamp}_{label}.idx"));
        let addresses: Vec<i64> = {
            let sorted: BTreeSet<i64> = self.pages.keys().copied().collect();
            sorted.into_iter().collect()
        };
        let mut packed = Vec::with_capacity(addresses.len() * 4);
        for &address in &addresses {
            let word = u32::try_from(address)
                .map_err(|_| Error::unpack("'I' format requires 0 <= number <= 4294967295"))?;
            packed.extend_from_slice(&word.to_le_bytes());
        }
        fs::write(&idx_path, packed)?;
        let mut handle = fs::File::create(target.join(format!("{stamp}_{label}.dat")))?;
        for base in &addresses {
            handle.write_all(&self.pages[base])?;
        }
        Ok(idx_path)
    }

    pub fn load(idx_path: &Path) -> Result<Snapshot> {
        let raw = fs::read(idx_path)?;
        let count = raw.len() / 4;
        if raw.len() != count * 4 {
            return Err(Error::unpack(format!(
                "unpack requires a buffer of {} bytes",
                count * 4
            )));
        }
        let addresses: Vec<i64> = raw
            .as_chunks::<4>()
            .0
            .iter()
            .map(|&word| u32::from_le_bytes(word) as i64)
            .collect();
        let data = fs::read(idx_path.with_extension("dat"))?;
        let expected = addresses.len() as i64 * PAGE;
        if data.len() as i64 != expected {
            return Err(Error::value(format!(
                "{}: expected {expected} data bytes, got {}",
                idx_path
                    .file_name()
                    .map_or(String::new(), |name| name.to_string_lossy().into_owned()),
                data.len()
            )));
        }
        Ok(Snapshot {
            pages: addresses
                .iter()
                .enumerate()
                .map(|(i, &base)| {
                    (
                        base,
                        py::slice(&data, i as i64 * PAGE, (i as i64 + 1) * PAGE).to_vec(),
                    )
                })
                .collect(),
        })
    }
}

impl Memory for Snapshot {
    /// Read across page boundaries. Pages absent from the snapshot read as zeros.
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        if size < 0 {
            return Err(Error::value(format!(
                "size must not be negative, got {size}"
            )));
        }
        let (mut address, mut size) = (address, size);
        let mut out: Vec<u8> = Vec::new();
        while size > 0 {
            let base = address & !(PAGE - 1);
            let offset = address - base;
            let take = (PAGE - offset).min(size);
            match self.pages.get(&base) {
                Some(page) => out.extend_from_slice(py::slice(page, offset, offset + take)),
                None => out.extend(std::iter::repeat_n(0, take as usize)),
            }
            address += take;
            size -= take;
        }
        Ok(out)
    }
}

/// Capture every page in `spans`, dropping all-zero pages unless `keep_empty`.
///
/// Unmapped memory reads back as zeros rather than failing, so a dropped page means "nothing
/// there" and not "refused".
///
/// Asked in chunks with a breath between them. The emulator's RPC server keeps up for a few
/// thousand packets and then stops answering for good, which no amount of retrying fixes and
/// pacing does; a chunk is also small enough that one stall costs a chunk, not the whole sweep.
/// `between` is called between chunks, so a caller can keep polling while it sweeps.
///
/// Python's defaults are `spans` of `[FULL_SPAN]`, `keep_empty` false, `chunk` of `CHUNK_PAGES`,
/// `pause` of `CHUNK_PAUSE` and no `between`.
pub fn sweep(
    rpc: &dyn Rpc,
    spans: &[(i64, i64)],
    keep_empty: bool,
    chunk: i64,
    pause: f64,
    between: Option<&dyn Fn()>,
) -> Result<Snapshot> {
    let addresses: Vec<i64> = spans
        .iter()
        .flat_map(|&(lo, hi)| (lo..hi).step_by(PAGE as usize))
        .collect();
    let mut pages: IndexMap<i64, Vec<u8>> = IndexMap::new();
    for start in range_of(0, addresses.len() as i64, chunk)? {
        if start != 0
            && let Some(between) = between
        {
            between();
        }
        for (base, data) in rpc.read_many(py::slice(&addresses, start, start + chunk), PAGE)? {
            if keep_empty || data.iter().any(|&byte| byte != 0) {
                let mut data = data;
                if (data.len() as i64) < PAGE {
                    data.resize(PAGE as usize, 0);
                }
                pages.insert(base, data);
            }
        }
        if pause != 0.0 {
            time::sleep(pause);
        }
    }
    Ok(Snapshot { pages })
}

/// Join spans separated by less than `gap`, so a heap full of holes reads as one region.
///
/// Python's default `gap` is `DEFAULT_GAP`.
pub fn merge(ranges: &[(i64, i64)], gap: i64) -> Vec<(i64, i64)> {
    let mut sorted: Vec<(i64, i64)> = ranges.to_vec();
    sorted.sort();
    let mut merged: Vec<(i64, i64)> = Vec::new();
    for (low, high) in sorted {
        match merged.last_mut() {
            Some(last) if low - last.1 <= gap => *last = (last.0, last.1.max(high)),
            _ => merged.push((low, high)),
        }
    }
    merged
}

/// Where memory actually is, measured now rather than guessed once and copied around.
///
/// Three spikes in a row searched `0x08000000`-`0x08800000` because that is where the first one
/// happened to look, and the heap has a second populated block above it holding the objects that
/// own the text. A span written by hand is a claim about the game that nothing checks.
///
/// This reads the first page of every `step` bytes, not every page. Sweeping the whole address
/// space exactly is 786,000 packets, and the emulator stops answering long before the end of
/// that; at block resolution it is a few thousand and takes seconds. The cost is that a block
/// whose first page is blank reads as empty, so this is a map to narrow a sweep with, not a map
/// to trust byte for byte — `sweep(rpc, regions(rpc))` is the pair that gives both.
///
/// With `chunk`, probes are sent that many at a time with `between` called between them, as
/// `sweep` does: the linear heap's 8,192 probes held polling up for about 270 ms in one go.
///
/// Python's defaults are `span` of `FULL_SPAN`, `step` of `PROBE_STEP`, `gap` of `DEFAULT_GAP`,
/// and no `chunk` or `between`.
pub fn regions(
    rpc: &dyn Rpc,
    span: (i64, i64),
    step: i64,
    gap: i64,
    chunk: Option<i64>,
    between: Option<&dyn Fn()>,
) -> Result<Vec<(i64, i64)>> {
    let found = probe(rpc, span, step, chunk, between)?;
    let blocks: Vec<(i64, i64)> = found.iter().map(|&at| (at, at + step)).collect();
    Ok(merge(&blocks, gap))
}

/// The addresses `step` apart across `span` whose first page holds anything: `regions` before
/// its blocks are merged, for telling where memory has turned up since the last time.
///
/// Python's default `step` is `PROBE_STEP`, and it has no `chunk` or `between` by default.
pub fn probe(
    rpc: &dyn Rpc,
    span: (i64, i64),
    step: i64,
    chunk: Option<i64>,
    between: Option<&dyn Fn()>,
) -> Result<IndexSet<i64>> {
    let (low, high) = span;
    let probes = range_of(low, high, step)?;
    let size = match chunk {
        Some(chunk) if chunk != 0 => chunk,
        _ if probes.is_empty() => 1,
        _ => probes.len() as i64,
    };
    let mut replies: IndexMap<i64, Vec<u8>> = IndexMap::new();
    for start in range_of(0, probes.len() as i64, size)? {
        if start != 0
            && let Some(between) = between
        {
            between();
        }
        replies.extend(rpc.read_many(py::slice(&probes, start, start + size), PAGE)?);
    }
    Ok(probes
        .iter()
        .copied()
        .filter(|at| {
            replies
                .get(at)
                .is_some_and(|data| data.iter().any(|&b| b != 0))
        })
        .collect())
}

/// Bytes that change across `gap` seconds at `probes`. Zero means nothing is advancing.
///
/// Use it to check a capture is worth taking. Zero is necessary but not sufficient: a probe set
/// that happens to sit in quiet memory reports zero while the game runs, so the real proof a
/// snapshot is coherent is that two captures of one state come back identical.
///
/// Python's default `gap` is 0.15.
pub fn churn(rpc: &dyn Rpc, probes: &[i64], gap: f64) -> Result<i64> {
    let pages: Vec<i64> = probes.to_vec();
    let first = rpc.read_many(&pages, PAGE)?;
    time::sleep(gap);
    let second = rpc.read_many(&pages, PAGE)?;
    let mut changed = 0;
    for (base, before) in &first {
        if let Some(after) = second.get(base) {
            changed += before.iter().zip(after).filter(|(a, b)| a != b).count() as i64;
        }
    }
    Ok(changed)
}
