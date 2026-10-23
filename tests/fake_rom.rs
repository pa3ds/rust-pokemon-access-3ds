//! Small ROM images for tests: an NCSD holding an NCCH holding a RomFS, with GARC archives and
//! Gen 6 text files inside, built the way `pa3ds.core.rom` and `games.gen6.archives` read them.
//!
//! Nothing here comes from a real ROM, so the tests say what the readers expect of the format and
//! nothing about any game's contents beyond the file numbers the readers name.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use indexmap::IndexMap;

use crate::py;

pub const UNIT: i64 = 0x200;
pub const NONE: i64 = 0xFFFFFFFF;
pub const BLOCK_SHIFT: i64 = 12;
pub const MASTER_HASH: i64 = 0x20;

/// `struct.pack("<...")` of (value, width in bytes) fields.
fn pack(fields: &[(i64, usize)]) -> Vec<u8> {
    fields
        .iter()
        .flat_map(|&(value, width)| value.to_le_bytes()[..width].to_vec())
        .collect()
}

/// `struct.pack_into`: `bytes` over `buffer` from `at`.
fn put(buffer: &mut [u8], at: usize, bytes: &[u8]) {
    buffer[at..at + bytes.len()].copy_from_slice(bytes);
}

/// `bytes(-len(data) % to)`: the zeros that pad `data` to a multiple of `to`.
fn padding(data: &[u8], to: i64) -> Vec<u8> {
    vec![0; py::modulo(-(data.len() as i64), to) as usize]
}

/// An NCSD image whose first partition is an NCCH with this program id and these files.
pub fn rom_image(files: &[(&str, &[u8])], title: i64) -> Vec<u8> {
    rom_image_with(files, title, false)
}

/// `rom_image(files, title, encrypted)`.
pub fn rom_image_with(files: &[(&str, &[u8])], title: i64, encrypted: bool) -> Vec<u8> {
    rom_image_full(files, title, encrypted, &[])
}

/// `rom_image(files, title, names=names)`.
pub fn rom_image_named(files: &[(&str, &[u8])], title: i64, names: &[&str]) -> Vec<u8> {
    rom_image_full(files, title, false, names)
}

/// An NCSD image whose first partition is an NCCH with this program id and these files, and
/// with `names`, an ExeFS whose SMDH calls the game so in each of its languages in order.
pub fn rom_image_full(
    files: &[(&str, &[u8])],
    title: i64,
    encrypted: bool,
    names: &[&str],
) -> Vec<u8> {
    let mut romfs = romfs(files);
    romfs.extend(padding(&romfs, UNIT));
    let exefs = if !names.is_empty() {
        let mut smdh = b"SMDH".to_vec();
        smdh.extend(vec![0; 4 + 16 * 0x200]);
        for (index, name) in names.iter().enumerate() {
            let encoded: Vec<u8> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
            put(&mut smdh, 8 + index * 0x200, &encoded);
        }
        let mut exefs = vec![0; UNIT as usize];
        put(&mut exefs, 0, b"icon");
        put(&mut exefs, 8, &pack(&[(0, 4), (smdh.len() as i64, 4)]));
        exefs.extend(smdh);
        exefs.extend(padding(&exefs, UNIT));
        exefs
    } else {
        Vec::new()
    };
    let mut ncsd = vec![0; UNIT as usize];
    put(&mut ncsd, 0x100, b"NCSD");
    let ncch_units = 1;
    let ncch_size = 2 + (romfs.len() as i64 + UNIT - 1) / UNIT;
    put(&mut ncsd, 0x120, &pack(&[(ncch_units, 4), (ncch_size, 4)]));
    let mut ncch = vec![0; UNIT as usize];
    put(&mut ncch, 0x100, b"NCCH");
    put(&mut ncch, 0x118, &pack(&[(title, 8)]));
    ncch[0x18F] = if encrypted { 0 } else { 0x04 };
    put(
        &mut ncch,
        0x1B0,
        &pack(&[(1, 4), (romfs.len() as i64 / UNIT, 4)]),
    );
    if !exefs.is_empty() {
        put(
            &mut ncch,
            0x1A0,
            &pack(&[
                (1 + romfs.len() as i64 / UNIT, 4),
                (exefs.len() as i64 / UNIT, 4),
            ]),
        );
    }
    [ncsd, ncch, romfs, exefs].concat()
}

struct Dir {
    name: String,
    dirs: IndexMap<String, usize>,
    files: IndexMap<String, Vec<u8>>,
    at: i64,
}

impl Dir {
    fn new(name: &str) -> Dir {
        Dir {
            name: name.to_string(),
            dirs: IndexMap::new(),
            files: IndexMap::new(),
            at: 0,
        }
    }
}

fn romfs(files: &[(&str, &[u8])]) -> Vec<u8> {
    // Python holds `_Dir` objects and tells them apart by identity; here they are indices into
    // one list, the root first.
    let mut tree = vec![Dir::new("")];
    for &(path, data) in files {
        let mut folders: Vec<&str> = path.split('/').collect();
        let name = folders.pop().expect("a path has a name");
        let mut here = 0;
        for folder in folders {
            here = match tree[here].dirs.get(folder) {
                Some(&child) => child,
                None => {
                    tree.push(Dir::new(folder));
                    let child = tree.len() - 1;
                    tree[here].dirs.insert(folder.to_string(), child);
                    child
                }
            };
        }
        tree[here].files.insert(name.to_string(), data.to_vec());
    }

    let mut order: Vec<usize> = Vec::new();
    let mut pending: VecDeque<usize> = VecDeque::from([0]);
    while let Some(here) = pending.pop_front() {
        order.push(here);
        pending.extend(tree[here].dirs.values().copied());
    }
    let mut at = 0;
    for &here in &order {
        tree[here].at = at;
        at += 0x18 + padded(&tree[here].name);
    }
    let mut dirs: Vec<u8> = Vec::new();
    let mut files_meta: Vec<u8> = Vec::new();
    let mut data: Vec<u8> = Vec::new();
    let mut file_offset = 0;
    let parents: HashMap<usize, usize> = order
        .iter()
        .flat_map(|&here| tree[here].dirs.values().map(move |&child| (child, here)))
        .collect();
    for &here in &order {
        let siblings: Vec<usize> = match parents.get(&here) {
            Some(&parent) => tree[parent].dirs.values().copied().collect(),
            None => vec![here],
        };
        let index = siblings.iter().position(|&s| s == here).expect("a sibling");
        let after = siblings.get(index + 1).copied();
        let children: Vec<usize> = tree[here].dirs.values().copied().collect();
        let names: Vec<String> = tree[here].files.keys().cloned().collect();
        let first_file = if !names.is_empty() { file_offset } else { NONE };
        dirs.extend(pack(&[
            (parents.get(&here).map_or(0, |&parent| tree[parent].at), 4),
            (after.map_or(NONE, |after| tree[after].at), 4),
            (children.first().map_or(NONE, |&child| tree[child].at), 4),
            (first_file, 4),
            (NONE, 4),
            (tree[here].name.chars().count() as i64 * 2, 4),
        ]));
        dirs.extend(name(&tree[here].name));
        for (index, file_name) in names.iter().enumerate() {
            let blob = &tree[here].files[file_name];
            let size = 0x20 + padded(file_name);
            let sibling = if index + 1 < names.len() {
                file_offset + size
            } else {
                NONE
            };
            files_meta.extend(pack(&[
                (tree[here].at, 4),
                (sibling, 4),
                (data.len() as i64, 8),
                (blob.len() as i64, 8),
                (NONE, 4),
                (file_name.chars().count() as i64 * 2, 4),
            ]));
            files_meta.extend(name(file_name));
            file_offset += size;
            data.extend(blob);
            data.extend(padding(&data, 0x10));
        }
    }

    let header_size = 0x28;
    let dir_meta = header_size;
    let file_meta = dir_meta + dirs.len() as i64;
    let data_offset = (file_meta + files_meta.len() as i64 + 0xF) / 0x10 * 0x10;
    let mut level3 = pack(&[
        (header_size, 4),
        (header_size, 4),
        (0, 4),
        (dir_meta, 4),
        (dirs.len() as i64, 4),
        (file_meta, 4),
        (0, 4),
        (file_meta, 4),
        (files_meta.len() as i64, 4),
        (data_offset, 4),
    ]);
    level3.extend(dirs);
    level3.extend(files_meta);
    level3.extend(vec![0; data_offset as usize - level3.len()]);
    level3.extend(data);

    let mut ivfc = vec![0; 0x60];
    put(&mut ivfc, 0, b"IVFC");
    put(&mut ivfc, 0x08, &pack(&[(MASTER_HASH, 4)]));
    put(&mut ivfc, 0x4C, &pack(&[(BLOCK_SHIFT, 4)]));
    let block = 1 << BLOCK_SHIFT;
    let start = (0x60 + MASTER_HASH + block - 1) / block * block;
    let gap = vec![0; start as usize - ivfc.len()];
    [ivfc, gap, level3].concat()
}

fn name(text: &str) -> Vec<u8> {
    let mut raw: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
    raw.extend(padding(&raw, 4));
    raw
}

fn padded(text: &str) -> i64 {
    name(text).len() as i64
}

/// One entry of a GARC: a file, or its copies by bit.
pub enum GarcFile {
    One(Vec<u8>),
    Copies(IndexMap<i64, Vec<u8>>),
}

impl From<Vec<u8>> for GarcFile {
    fn from(file: Vec<u8>) -> Self {
        GarcFile::One(file)
    }
}

impl From<&[u8]> for GarcFile {
    fn from(file: &[u8]) -> Self {
        GarcFile::One(file.to_vec())
    }
}

impl From<IndexMap<i64, Vec<u8>>> for GarcFile {
    fn from(copies: IndexMap<i64, Vec<u8>>) -> Self {
        GarcFile::Copies(copies)
    }
}

/// A GARC holding these files, each one bit set in its FATB entry, or copies by bit: version
/// 0x0400, ORAS's and X's, or 0x0600, Sun's, whose header is longer by two words.
pub fn garc(files: &[GarcFile]) -> Vec<u8> {
    garc_with(files, 0x0400)
}

/// `garc(files, version)`.
pub fn garc_with(files: &[GarcFile], version: i64) -> Vec<u8> {
    let header = if version == 0x0400 { 0x1C } else { 0x24 };
    let entries: Vec<IndexMap<i64, Vec<u8>>> = files
        .iter()
        .map(|file| match file {
            GarcFile::Copies(copies) => copies.clone(),
            GarcFile::One(file) => IndexMap::from([(0, file.clone())]),
        })
        .collect();
    let mut fato = b"OTAF".to_vec();
    fato.extend(pack(&[
        (0x0C + 4 * entries.len() as i64, 4),
        (entries.len() as i64, 2),
        (0xFFFF, 2),
    ]));
    let mut table: Vec<u8> = Vec::new();
    let mut data: Vec<u8> = Vec::new();
    let mut blobs: Vec<&Vec<u8>> = Vec::new();
    for copies in &entries {
        fato.extend(pack(&[(table.len() as i64, 4)]));
        table.extend(pack(&[(copies.keys().map(|bit| 1 << bit).sum(), 4)]));
        let mut sorted: Vec<(&i64, &Vec<u8>)> = copies.iter().collect();
        sorted.sort();
        for (_bit, blob) in sorted {
            table.extend(pack(&[
                (data.len() as i64, 4),
                ((data.len() + blob.len()) as i64, 4),
                (blob.len() as i64, 4),
            ]));
            data.extend(blob);
            data.extend(padding(&data, 4));
            blobs.push(blob);
        }
    }
    let mut fatb = b"BTAF".to_vec();
    fatb.extend(pack(&[
        (0x0C + table.len() as i64, 4),
        (entries.len() as i64, 4),
    ]));
    fatb.extend(&table);
    let mut fimb = b"BMIF".to_vec();
    fimb.extend(pack(&[(0x0C, 4), (data.len() as i64, 4)]));
    let offset = header + (fato.len() + fatb.len() + fimb.len()) as i64;
    let largest = blobs.iter().map(|b| b.len()).max().unwrap_or(0) as i64;
    let mut head = b"CRAG".to_vec();
    head.extend(pack(&[
        (header, 4),
        (0xFEFF, 2),
        (version, 2),
        (4, 4),
        (offset, 4),
        (offset + data.len() as i64, 4),
        (largest, 4),
    ]));
    if version != 0x0400 {
        head.extend(pack(&[(largest, 4), (4, 4)]));
    }
    [head, fato, fatb, fimb, data].concat()
}

/// One line of a Gen 6 text file: a string, or its halfwords when it holds control sequences.
pub enum MessageLine {
    Text(String),
    Halfwords(Vec<i64>),
}

impl From<&str> for MessageLine {
    fn from(line: &str) -> Self {
        MessageLine::Text(line.to_string())
    }
}

impl From<Vec<i64>> for MessageLine {
    fn from(line: Vec<i64>) -> Self {
        MessageLine::Halfwords(line)
    }
}

/// A Gen 6 text file: each line a string, or its halfwords when it holds control
/// sequences, XORed with the game's key stream.
pub fn message_file(lines: &[MessageLine]) -> Vec<u8> {
    let first = 0x10;
    let mut table: Vec<u8> = Vec::new();
    let mut body: Vec<u8> = Vec::new();
    let mut key: i64 = 0x7C89;
    let heads = 4 + 8 * lines.len() as i64;
    for line in lines {
        let mut units: Vec<i64> = match line {
            MessageLine::Text(line) => line.encode_utf16().map(i64::from).collect(),
            MessageLine::Halfwords(line) => line.clone(),
        };
        units.push(0);
        table.extend(pack(&[
            (heads + body.len() as i64, 4),
            (units.len() as i64, 2),
            (0, 2),
        ]));
        let mut rolling = key;
        for unit in units {
            body.extend(pack(&[(unit ^ rolling, 2)]));
            rolling = (rolling << 3 | rolling >> 13) & 0xFFFF;
        }
        key = (key + 0x2983) & 0xFFFF;
    }
    let mut section = pack(&[(heads + body.len() as i64, 4)]);
    section.extend(table);
    section.extend(body);
    let mut file = pack(&[
        (1, 2),
        (lines.len() as i64, 2),
        (section.len() as i64, 4),
        (0, 4),
        (first, 4),
    ]);
    file.extend(section);
    file
}

/// LZ11 with every byte a literal: valid, and enough to test a reader.
pub fn lz11_literals(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x11];
    out.extend(&(data.len() as u32).to_le_bytes()[..3]);
    for chunk in data.chunks(8) {
        out.push(0);
        out.extend(chunk);
    }
    out
}

/// pytest's `tmp_path`: a folder of its own that is removed when the test is done with it.
pub struct TmpDir {
    path: PathBuf,
}

impl TmpDir {
    pub fn new() -> TmpDir {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "pa3ds-test-{}-{}",
            std::process::id(),
            COUNT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("a temporary folder");
        TmpDir { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TmpDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
