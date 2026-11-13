//! Layout files: what the CLYT a layout was built from says about its panes.
//!
//! Pane names are minified (spike 3), and a picture's words are in its texture. The layout files
//! stay in memory, and name the textures their pictures use, often as words: the battle's command
//! menu textures its buttons `fight_button.bclim`, `bag_button.bclim`, `pokemon_button.bclim` and
//! `run_button.bclim` (spike 50), which are said without the trailing "button".
//!
//! ```text
//! CLYT     +0x00 "CLYT"   +0x04 u16 0xFEFF   +0x06 u16 header size, 0x14
//!          +0x0c u32 file size   +0x10 u16 section count
//! section  +0x00 magic    +0x04 u32 size of the whole section
//! txl1     +0x08 u32 count, then count u32 offsets from +0x0c to NUL-terminated names
//! mat1     +0x08 u32 count, then count u32 offsets from the section's start to materials:
//!          +0x00 char[20] name   +0x30 u32 flags, the low two bits the texture map count
//!          +0x34 texture maps, 4 bytes each: u16 texture index, u8, u8
//! panes    pan1, pic1, txt1, wnd1, bnd1, prt1: +0x0c char[16] name
//! pic1     +0x5c u16 material index
//! wnd1     +0x5c u8 frame count   +0x60 u32 offset of its content   +0x64 u32 offset of its
//!          frame table, both from the section's start: the content's material index at its
//!          +0x10, and each frame table entry an offset to a frame whose material index is first
//! ```

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{u16, u32};
use crate::core::memory::Memory;
use crate::py::{self, Error, Kind, Result};

pub const MAGIC: &[u8] = b"CLYT";
pub const BOM: i64 = 0xFEFF;
pub const HEADER_SIZE: i64 = 0x14;
pub const FILE_SIZE: i64 = 0x0C;
pub const SECTION_COUNT: i64 = 0x10;
/// The biggest file believed. The biggest seen, the field's PlayNav map, is 71,868 bytes.
pub const MAX_FILE: i64 = 0x400000;

pub const ARCHIVE_MAGIC: &[u8] = b"darc";
pub const ARCHIVE_HEADER: i64 = 0x1C;
pub const ARCHIVE_SIZE: i64 = 0x0C;
/// The biggest archive believed; this only rejects a read of something that is not one.
pub const MAX_ARCHIVE: i64 = 0x400000;
pub const TEXTURE: &str = ".bclim";
pub const DARC_TABLE: i64 = 0x10;
/// A `darc`'s file table: entries of name offset, data offset and length, the first the root
/// directory, whose length is the number of entries; the names follow, in UTF-16.
pub const DARC_ENTRY: i64 = 12;
/// The bit of an entry's name offset that makes it a directory.
pub const DIRECTORY: i64 = 0x01000000;

pub const PANE_SECTIONS: [&[u8]; 6] = [b"pan1", b"pic1", b"txt1", b"wnd1", b"bnd1", b"prt1"];
pub const PANE_NAME: i64 = 0x0C;
pub const PANE_NAME_SIZE: i64 = 16;
pub const PICTURE_MATERIAL: i64 = 0x5C;
pub const WINDOW_FRAME_COUNT: i64 = 0x5C;
pub const WINDOW_CONTENT: i64 = 0x60;
pub const WINDOW_FRAMES: i64 = 0x64;
pub const CONTENT_MATERIAL: i64 = 0x10;
pub const MATERIAL_FLAGS: i64 = 0x30;
pub const MATERIAL_TEXTURE_MAPS: i64 = 0x34;

/// The pane names of a layout file in the order it lists them, and each picture's textures.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutFile {
    pub panes: Vec<String>,
    /// Picture pane name -> the names of the textures its material uses, in order.
    pub textures: IndexMap<String, Vec<String>>,
    /// Window pane name -> the names of the textures its content and frames are drawn with, in
    /// order: the main menu's cursor is a window drawn with `cursor_lt.bclim` (spike 96).
    pub frames: IndexMap<String, Vec<String>>,
    /// What its archive calls it, `waza.bclyt`; "" where it was not read out of an archive.
    pub name: String,
}

impl LayoutFile {
    /// `LayoutFile(panes, textures, frames)`: `name` left to its default, "".
    pub fn new(
        panes: Vec<String>,
        textures: IndexMap<String, Vec<String>>,
        frames: IndexMap<String, Vec<String>>,
    ) -> Self {
        LayoutFile {
            panes,
            textures,
            frames,
            name: String::new(),
        }
    }

    /// `LayoutFile(panes, textures)`: `frames` left to its default, an empty dict.
    pub fn without_frames(panes: Vec<String>, textures: IndexMap<String, Vec<String>>) -> Self {
        LayoutFile::new(panes, textures, IndexMap::new())
    }
}

/// The size of the layout file whose first `HEADER_SIZE` bytes these are, or None.
pub fn size_of(head: &[u8]) -> Option<i64> {
    if (head.len() as i64) < HEADER_SIZE || &head[..4] != MAGIC {
        return None;
    }
    // A head of `HEADER_SIZE` bytes holds all three fields, so none of these can raise.
    let held = "a head of HEADER_SIZE bytes holds this";
    if u16(head, 4).expect(held) != BOM || u16(head, 6).expect(held) != HEADER_SIZE {
        return None;
    }
    let size = u32(head, FILE_SIZE).expect(held);
    if HEADER_SIZE < size && size <= MAX_FILE {
        Some(size)
    } else {
        None
    }
}

/// The layout file in `blob`, or None when it is not one or is cut short.
pub fn parse(blob: &[u8]) -> Option<LayoutFile> {
    size_of(py::slice(blob, 0, HEADER_SIZE))?;
    let mut names: Vec<String> = Vec::new();
    let mut textures: Vec<String> = Vec::new();
    let mut materials: Vec<Vec<i64>> = Vec::new();
    let mut pictures: IndexMap<String, i64> = IndexMap::new();
    let mut windows: IndexMap<String, Vec<i64>> = IndexMap::new();
    let mut at = HEADER_SIZE;
    // The Python's `try: ... except struct.error: return None`: only `u16` and `u32` raise in it,
    // and both raise `struct.error`.
    let walked: Result<bool> = (|| {
        for _ in 0..u16(blob, SECTION_COUNT)? {
            let magic = py::slice(blob, at, at + 4);
            let size = u32(blob, at + 4)?;
            if size < 8 || at + size > blob.len() as i64 {
                return Ok(false);
            }
            if magic == b"txl1" {
                let base = at + 0x0C;
                textures = (0..u32(blob, at + 8)?)
                    .map(|i| {
                        Ok(name(
                            blob,
                            base + u32(blob, base + 4 * i)?,
                            blob.len() as i64,
                        ))
                    })
                    .collect::<Result<Vec<String>>>()?;
            } else if magic == b"mat1" {
                for i in 0..u32(blob, at + 8)? {
                    let material = at + u32(blob, at + 0x0C + 4 * i)?;
                    let count = u32(blob, material + MATERIAL_FLAGS)? & 3;
                    materials.push(
                        (0..count)
                            .map(|k| u16(blob, material + MATERIAL_TEXTURE_MAPS + 4 * k))
                            .collect::<Result<Vec<i64>>>()?,
                    );
                }
            } else if PANE_SECTIONS.contains(&magic) {
                let pane = name(blob, at + PANE_NAME, at + PANE_NAME + PANE_NAME_SIZE);
                names.push(pane.clone());
                if magic == b"pic1" {
                    pictures.insert(pane, u16(blob, at + PICTURE_MATERIAL)?);
                } else if magic == b"wnd1" {
                    windows.insert(pane, window_materials(py::slice(blob, at, at + size)));
                }
            }
            at += size;
        }
        Ok(true)
    })();
    match walked {
        Ok(true) => {}
        Ok(false) | Err(_) => return None,
    }
    let drawn_with = |indices: &[i64]| -> Vec<String> {
        let mut said: Vec<String> = Vec::new();
        for &index in indices {
            if index < materials.len() as i64 {
                for &i in &materials[index as usize] {
                    if i < textures.len() as i64 && !said.contains(&textures[i as usize]) {
                        said.push(textures[i as usize].clone());
                    }
                }
            }
        }
        said
    };

    Some(LayoutFile::new(
        names,
        pictures
            .iter()
            .filter(|&(_, &index)| index < materials.len() as i64)
            .map(|(name, &index)| {
                (
                    name.clone(),
                    materials[index as usize]
                        .iter()
                        .filter(|&&i| i < textures.len() as i64)
                        .map(|&i| textures[i as usize].clone())
                        .collect(),
                )
            })
            .collect(),
        windows
            .iter()
            .map(|(name, indices)| (name.clone(), drawn_with(indices)))
            .collect(),
    ))
}

/// The material indices a window section draws its content and each frame with, or none
/// when the section is too short to say: a window is only ever asked what it is drawn with, and
/// a file whose windows cannot say still names its panes and its pictures.
fn window_materials(section: &[u8]) -> Vec<i64> {
    let used = || -> Result<Vec<i64>> {
        let content = u32(section, WINDOW_CONTENT)?;
        let table = u32(section, WINDOW_FRAMES)?;
        let mut used = vec![u16(section, content + CONTENT_MATERIAL)?];
        let frames = match section.get(WINDOW_FRAME_COUNT as usize) {
            Some(&count) => count as i64,
            None => return Err(Error::index("index out of range")),
        };
        for k in 0..frames {
            used.push(u16(section, u32(section, table + 4 * k)?)?);
        }
        Ok(used)
    };
    match used() {
        Ok(used) => used,
        Err(error) => {
            debug_assert!(error.is(Kind::StructError) || error.is(Kind::IndexError));
            Vec::new()
        }
    }
}

/// The address of every layout file starting in these pages, by its header.
///
/// A layout's work leads to the archive holding its files (spike 64), and the files are found
/// in the archive by their headers, then matched to layouts by their panes' names.
pub fn find(pages: &IndexMap<i64, Vec<u8>>) -> Vec<i64> {
    let mut found: Vec<i64> = Vec::new();
    let mut sorted: Vec<(&i64, &Vec<u8>)> = pages.iter().collect();
    sorted.sort_by_key(|&(page, _)| *page);
    for (&page, data) in sorted {
        find_in(page, data, &mut found);
    }
    found
}

/// `find`'s walk of one page, which `LayoutFiles.add_archive` makes of its archive alone without
/// building the one-entry dict (and copying the archive into it) the Python does.
fn find_in(page: i64, data: &[u8], found: &mut Vec<i64>) {
    let mut start = 0;
    while let Some(hit) = find_bytes(data, MAGIC, start) {
        start = hit + MAGIC.len();
        if size_of(py::slice(data, hit as i64, hit as i64 + HEADER_SIZE)).is_some() {
            found.push(page + hit as i64);
        }
    }
}

/// `data.find(needle, start)`, as an index.
fn find_bytes(data: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    data.get(start..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|at| start + at)
}

/// (name, start, length) of each file of a `darc` archive whose header is at the start of
/// `data`, its directories left out.
pub fn darc_entries(data: &[u8]) -> Result<Vec<(String, i64, i64)>> {
    let table = u32(data, DARC_TABLE)?;
    let count = u32(data, table + 8)?;
    let names = table + count * DARC_ENTRY;
    let mut out: Vec<(String, i64, i64)> = Vec::new();
    for index in 0..count {
        let at = table + index * DARC_ENTRY;
        let (name_at, start, length) = (u32(data, at)?, u32(data, at + 4)?, u32(data, at + 8)?);
        if name_at & DIRECTORY != 0 {
            continue;
        }
        let first = names + name_at;
        let mut end = first;
        while end + 1 < data.len() as i64 && py::slice(data, end, end + 2) != b"\0\0" {
            end += 2;
        }
        out.push((
            py::decode_utf16le(py::slice(data, first, end)),
            start,
            length,
        ));
    }
    Ok(out)
}

/// Layout files found in memory, matched to layouts by the names of their panes.
///
/// A layout builds its panes in the order its file lists them (spike 50), so the names of its
/// panes, walked depth first, are the names its file lists. The files are resident and pooled
/// (spike 3), so each is read once.
pub struct LayoutFiles {
    memory: Rc<dyn Memory>,
    archives: RefCell<HashSet<i64>>,
    by_panes: RefCell<IndexMap<Vec<String>, Rc<LayoutFile>>>,
    /// The same files by their pane names sorted, for a layout whose panes have been moved.
    by_names: RefCell<IndexMap<Vec<String>, Vec<Rc<LayoutFile>>>>,
    /// Where each texture file of the archives read starts -> its name (`image_at`).
    images: RefCell<IndexMap<i64, String>>,
}

impl LayoutFiles {
    pub fn new(memory: Rc<dyn Memory>) -> Self {
        LayoutFiles {
            memory,
            archives: RefCell::new(HashSet::new()),
            by_panes: RefCell::new(IndexMap::new()),
            by_names: RefCell::new(IndexMap::new()),
            images: RefCell::new(IndexMap::new()),
        }
    }

    /// The name of the texture file of an archive read that starts at this address, or None.
    ///
    /// A picture's material points at the image it draws, which is a file of its layout's own
    /// archive where the game has not copied it elsewhere: the Pokédex's habitat swaps its
    /// icons between `map_ico_00.bclim` and the grey `map_ico_00_1.bclim` (spike 153).
    pub fn image_at(&self, address: i64) -> Option<String> {
        self.images.borrow().get(&address).cloned()
    }

    /// Read every layout file in the `darc` archive at this address, the first time it is given.
    ///
    /// Each layout's work leads to the archive its files came from (spike 64), which is how its
    /// files are found without scanning every swept page for them.
    ///
    /// ```text
    /// darc  +0x00 "darc"   +0x04 u16 0xFEFF   +0x06 u16 header size, 0x1c   +0x0c u32 size
    /// ```
    pub fn add_archive(&self, address: i64) -> Result<()> {
        if !self.archives.borrow_mut().insert(address) {
            return Ok(());
        }
        let head = self.memory.read(address, ARCHIVE_HEADER)?;
        if py::slice(&head, 0, 4) != ARCHIVE_MAGIC
            || u16(&head, 4)? != BOM
            || u16(&head, 6)? != ARCHIVE_HEADER
        {
            return Ok(());
        }
        let size = u32(&head, ARCHIVE_SIZE)?;
        if !(ARCHIVE_HEADER < size && size <= MAX_ARCHIVE) {
            return Ok(());
        }
        let data = self.memory.read(address, size)?;
        let mut named: IndexMap<i64, String> = IndexMap::new();
        match darc_entries(&data) {
            Ok(entries) => {
                for (name, start, _length) in entries {
                    named.insert(start, name.clone());
                    if name.ends_with(TEXTURE) {
                        self.images.borrow_mut().insert(address + start, name);
                    }
                }
            }
            Err(error) if error.is(Kind::StructError) || error.is(Kind::IndexError) => {}
            Err(error) => return Err(error),
        }
        let mut offsets: Vec<i64> = Vec::new();
        find_in(0, &data, &mut offsets);
        for offset in offsets {
            let length = size_of(py::slice(&data, offset, offset + HEADER_SIZE));
            let mut parsed = match length {
                Some(length) => parse(py::slice(&data, offset, offset + length)),
                None => None,
            };
            if let (Some(parsed), Some(name)) = (parsed.as_mut(), named.get(&offset)) {
                parsed.name = name.clone();
            }
            let new = match &parsed {
                Some(parsed) => !self.by_panes.borrow().contains_key(parsed.panes.as_slice()),
                None => false,
            };
            if let (true, Some(parsed)) = (new, parsed) {
                let parsed = Rc::new(parsed);
                self.by_panes
                    .borrow_mut()
                    .insert(parsed.panes.clone(), parsed.clone());
                let mut sorted = parsed.panes.clone();
                sorted.sort();
                self.by_names
                    .borrow_mut()
                    .entry(sorted)
                    .or_default()
                    .push(parsed);
            }
        }
        Ok(())
    }

    /// The file whose panes have exactly these names in this order, or None.
    ///
    /// A layout can move a pane: the name entry's keyboard carries its cursor from key to key, so
    /// the order its panes are walked in changes as the player types and no longer matches the
    /// order its file lists (opening.ctm, 2026-09-18). Failing an exact match, a file whose pane
    /// names are the same set is taken, and only when it is the only such file.
    pub fn matching(&self, panes: &[String]) -> Option<Rc<LayoutFile>> {
        if let Some(exact) = self.by_panes.borrow().get(panes) {
            return Some(exact.clone());
        }
        let mut sorted = panes.to_vec();
        sorted.sort();
        let by_names = self.by_names.borrow();
        match by_names.get(&sorted) {
            Some(same) if same.len() == 1 => Some(same[0].clone()),
            _ => None,
        }
    }
}

fn name(blob: &[u8], start: i64, limit: i64) -> String {
    let window = py::slice(blob, start, limit);
    let end = window.iter().position(|&byte| byte == 0);
    py::decode_ascii(&window[..end.unwrap_or(window.len())])
}
