//! The player's own ROM: finding it, and reading files out of it (ADR-0008).
//!
//! Some of what a player needs to hear is not in RAM when it is wanted — a place's name, the rival's
//! — but it is in the ROM they are playing, in their language. This reads a 3DS ROM image as it lies
//! on disk, with no keys and no other tool: an NCSD image, its first partition's NCCH, and the RomFS
//! inside it (findings, spike 74). Nothing here knows about Pokémon; what the files mean is the game
//! package's.
//!
//! **Only decrypted images.** An encrypted one would need keys pa3ds will never ship, so it is
//! turned away with a reason, and pa3ds carries on without what the ROM would have named.
//!
//! **Finding it** needs nothing from the player. Azahar keeps the ROMs it has opened in its own
//! config (`Paths\recentFiles`, newest first) and the folder they were opened from
//! (`Paths\romsPath`); the running game's title id comes from the RPC's process list, and every
//! NCCH carries its program id. The first decrypted image whose id matches is the one being played —
//! found and validated, the way an anchor is, rather than a path written down. `--rom` names one
//! outright.
//!
//! ```text
//! NCSD    +0x100 "NCSD"   +0x120 eight partitions of (u32 offset, u32 size), in 0x200 units
//! NCCH    +0x100 "NCCH"   +0x118 u64 program id   +0x18f flags, bit 2 of which is "no crypto"
//!         +0x1a0 the ExeFS's offset, in 0x200 units    +0x1b0 the RomFS's offset and size
//! ExeFS   +0x00 ten entries of (8-byte name, u32 offset, u32 size), the files from +0x200;
//!         `icon` is the SMDH: +0x00 "SMDH", +0x08 sixteen titles of 0x200 bytes, a language
//!         each (`SMDH_LANGUAGES`), each from its short name, UTF-16, 0x80 bytes
//! IVFC    +0x00 "IVFC"    +0x08 the master hash's size   +0x4c level 3's block size, as a shift
//! level 3 follows the master hash, aligned to that block: +0x00 its header's length (0x28), then
//!         the directory and file metadata and where the files' data starts
//! ```
//!
//! The level offsets in the IVFC header describe the hash tree, not where the files are; that cost
//! spike 74 an hour.
//!
//! Bound by:
//! - ADR-0008: the player's own ROM, found among Azahar's recent files by the running title; an
//!   encrypted one is refused.

use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::core::binary;
use crate::py::{self, Error, Kind, Result};

/// NCSD and NCCH count in media units of 0x200 bytes.
pub const UNIT: i64 = 0x200;
pub const NCSD_MAGIC: &[u8] = b"NCSD";
pub const NCCH_MAGIC: &[u8] = b"NCCH";
pub const IVFC_MAGIC: &[u8] = b"IVFC";
/// Bit of the NCCH's flags byte at +0x18f that says its contents are stored in the clear.
pub const NO_CRYPTO: i64 = 0x04;
pub const LEVEL3_HEADER: i64 = 0x28;
/// An empty link in the RomFS's metadata.
pub const NONE: i64 = 0xFFFFFFFF;
pub const LZ11: i64 = 0x11;
pub const SMDH_MAGIC: &[u8] = b"SMDH";
pub const SMDH_TITLES: i64 = 0x08;
pub const SMDH_TITLE: i64 = 0x200;
pub const SMDH_SHORT: i64 = 0x80;
/// The language of each of the SMDH's titles, in its order.
pub const SMDH_LANGUAGES: [&str; 12] = [
    "ja", "en", "fr", "de", "it", "es", "zh-CN", "ko", "nl", "pt", "ru", "zh-TW",
];
/// What an NCSD image is called. A CIA is an installer, not an image, and is not read.
pub const EXTENSIONS: [&str; 2] = [".3ds", ".cci"];

#[derive(Debug, Clone, Copy, PartialEq)]
struct Entry {
    at: i64,
    size: i64,
}

/// One ROM image, open for reading. Files are read on demand; nothing is held but the index.
///
/// Python's `with Rom(path) as rom:` is a `Rom` that is dropped, which closes the file.
#[derive(Debug)]
pub struct Rom {
    pub path: PathBuf,
    pub title_id: i64,
    file: File,
    entries: IndexMap<String, Entry>,
}

impl Rom {
    pub fn new(path: &Path) -> Result<Rom> {
        let mut file = File::open(path)?;
        match index(&mut file) {
            Ok((title_id, entries)) => Ok(Rom {
                path: path.to_path_buf(),
                title_id,
                file,
                entries,
            }),
            Err(error) => {
                drop(file);
                if error.is(Kind::RomError) {
                    return Err(error);
                }
                if error.is(Kind::StructError)
                    || error.is(Kind::OsError)
                    || error.is(Kind::UnicodeDecodeError)
                {
                    return Err(Error::new(
                        Kind::RomError,
                        format!("{}: {error}", name(path)),
                    ));
                }
                Err(error)
            }
        }
    }

    pub fn close(self) {
        drop(self.file);
    }

    /// Every file in the RomFS, by its path: `a/0/7/3`.
    pub fn paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = self.entries.keys().cloned().collect();
        paths.sort();
        paths
    }

    /// One file's bytes. A path the ROM does not have is a RomError, not an empty file.
    pub fn read(&self, path: &str) -> Result<Vec<u8>> {
        let Some(entry) = self.entries.get(path) else {
            return Err(Error::new(
                Kind::RomError,
                format!("{} has no {path}", name(&self.path)),
            ));
        };
        self.read_part(path, 0, entry.size)
    }

    /// What the game calls itself in a language, its SMDH's short name, "Pokémon Omega
    /// Ruby"; English's where the language has none, and None where the ROM has no SMDH.
    pub fn title(&self, language: &str) -> Result<Option<String>> {
        let smdh = match self.smdh() {
            Ok(Some(smdh)) => smdh,
            Ok(None) => return Ok(None),
            Err(error) if error.is(Kind::OsError) || error.is(Kind::StructError) => {
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        if py::slice(&smdh, 0, 4) != SMDH_MAGIC {
            return Ok(None);
        }
        // `dict.fromkeys((language, "en"))`.
        let wanted: &[&str] = if language == "en" {
            &["en"]
        } else {
            &[language, "en"]
        };
        for wanted in wanted {
            if let Some(index) = SMDH_LANGUAGES.iter().position(|one| one == wanted) {
                let at = SMDH_TITLES + index as i64 * SMDH_TITLE;
                let decoded = decode_utf16le_ignore(py::slice(&smdh, at, at + SMDH_SHORT));
                let said = decoded.split('\0').next().unwrap_or_default();
                if !py::strip(said).is_empty() {
                    return Ok(Some(py::split(said).join(" ")));
                }
            }
        }
        Ok(None)
    }

    /// The body of `title`'s `try`: the SMDH's bytes, or None where the ExeFS has no `icon`.
    fn smdh(&self) -> Result<Option<Vec<u8>>> {
        let mut file = &self.file;
        seek(&mut file, 0)?;
        let partition = binary::u32(&read(&mut file, UNIT)?, 0x120)? * UNIT;
        seek(&mut file, partition)?;
        let exefs = partition + binary::u32(&read(&mut file, UNIT)?, 0x1A0)? * UNIT;
        seek(&mut file, exefs)?;
        let head = read(&mut file, UNIT)?;
        let mut icon = None;
        for at in (0..0xA0).step_by(0x10) {
            let name = py::slice(&head, at, at + 8);
            let end = name
                .iter()
                .rposition(|&byte| byte != 0)
                .map_or(0, |last| last + 1);
            if &name[..end] == b"icon" {
                icon = Some(binary::u32(&head, at + 8)?);
                break;
            }
        }
        let Some(icon) = icon else {
            return Ok(None);
        };
        seek(&mut file, exefs + UNIT + icon)?;
        Ok(Some(read(
            &mut file,
            SMDH_TITLES + SMDH_LANGUAGES.len() as i64 * SMDH_TITLE,
        )?))
    }

    /// `size` bytes of one file from `start`. Sun's map archive is 62 MB and a map block
    /// is a fraction of one, so a block is read alone. A part past the file's end is a
    /// RomError.
    pub fn read_part(&self, path: &str, start: i64, size: i64) -> Result<Vec<u8>> {
        let Some(entry) = self.entries.get(path) else {
            return Err(Error::new(
                Kind::RomError,
                format!("{} has no {path}", name(&self.path)),
            ));
        };
        if start < 0 || size < 0 || start + size > entry.size {
            return Err(Error::new(
                Kind::RomError,
                format!(
                    "{}: {path} has no bytes {} to {}",
                    name(&self.path),
                    hex(start),
                    hex(start + size)
                ),
            ));
        }
        let mut file = &self.file;
        seek(&mut file, entry.at + start)?;
        let data = read(&mut file, size)?;
        if data.len() as i64 != size {
            return Err(Error::new(
                Kind::RomError,
                format!("{}: {path} is cut short", name(&self.path)),
            ));
        }
        Ok(data)
    }
}

/// A ROM image's program id, from its first 0x200-byte header and its NCCH's, or None when
/// the file is not a readable, decrypted image. Two small reads: cheap enough to ask of every
/// ROM Azahar remembers.
///
/// An IndexError from a header cut short inside the NCCH's flags is not caught in Python, so it
/// is returned here.
pub fn title_id(path: &Path) -> Result<Option<i64>> {
    let Ok(mut rom) = File::open(path) else {
        return Ok(None);
    };
    match header(&mut rom) {
        Ok((program, _)) => Ok(Some(program)),
        Err(error)
            if error.is(Kind::RomError)
                || error.is(Kind::OsError)
                || error.is(Kind::StructError) =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

/// The first candidate that is a decrypted image of this title, or None. Why each one before
/// it was passed over goes to `notes`: a player told only that none was found re-decrypted ROMs
/// that were fine (#14).
pub fn find(
    title: i64,
    candidates: impl IntoIterator<Item = PathBuf>,
    mut notes: Option<&mut Vec<String>>,
) -> Result<Option<PathBuf>> {
    for path in candidates {
        let why = if !EXTENSIONS.contains(&suffix(&path).to_lowercase().as_str()) {
            "is not a ROM image".to_string()
        } else if !path.is_file() {
            "is not there".to_string()
        } else {
            match title_id(&path)? {
                None => "cannot be read as a decrypted ROM image".to_string(),
                Some(found) if found != title => format!("is title {found:016x}"),
                Some(_) => return Ok(Some(path)),
            }
        };
        if let Some(notes) = notes.as_deref_mut() {
            notes.push(format!("{} {why}", py::path_str(&path)));
        }
    }
    Ok(None)
}

/// The ROM images Azahar's `qt-config.ini` points at: its recent files, newest first, then
/// whatever sits in the folder it opens ROMs from. Duplicates are left out.
pub fn azahar_roms(config: &str) -> Vec<PathBuf> {
    let mut recent: Vec<String> = Vec::new();
    let mut folder = "";
    for line in py::splitlines(config) {
        let (key, value) = line.split_once('=').unwrap_or((line, ""));
        if py::strip(key) == r"Paths\recentFiles" {
            recent = string_list(value);
        } else if py::strip(key) == r"Paths\romsPath" {
            folder = py::strip(value);
        }
    }
    let mut found: Vec<PathBuf> = recent
        .iter()
        .filter(|item| !item.is_empty())
        .map(|item| py::to_path(item))
        .collect();
    if !folder.is_empty() {
        // `contextlib.suppress(OSError)`: a folder that cannot be listed, or an entry that fails
        // part way, adds nothing, as the failed `sorted(...)` added nothing.
        let listing: std::io::Result<Vec<PathBuf>> = std::fs::read_dir(py::to_path(folder))
            .and_then(|entries| {
                entries
                    .map(|entry| entry.map(|entry| entry.path()))
                    .collect()
            });
        if let Ok(mut listing) = listing {
            listing.retain(|path| EXTENSIONS.contains(&suffix(path).to_lowercase().as_str()));
            // Windows compares paths without regard to case, and sorts them so.
            listing.sort_by_cached_key(|path| py::normcase(&path.to_string_lossy()));
            found.extend(listing);
        }
    }
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<PathBuf> = Vec::new();
    for path in found {
        let key = py::normcase(&py::normpath(&path.to_string_lossy()));
        if seen.insert(key) {
            out.push(path);
        }
    }
    out
}

/// `lz11(blob)`: the whole of it.
pub fn lz11(blob: &[u8]) -> Result<Vec<u8>> {
    lz11_limit(blob, None)
}

/// Decompress Nintendo's LZ11, written from the format as it is documented. A zone's data
/// file is kept in the ROM this way (spike 86). With `limit`, no more than its first `limit`
/// bytes: a field model names its Pokémon in its first 12 KB of up to 218 (spike 193).
///
/// 0x11, then the size in 24 bits, or in the 32 after them when those are zero. Then a flag byte
/// before every eight blocks, high bit first: a clear bit is one literal byte, a set bit a
/// reference back into what is already out, whose first nibble says how wide its length is.
pub fn lz11_limit(blob: &[u8], limit: Option<i64>) -> Result<Vec<u8>> {
    if blob.is_empty() || blob[0] as i64 != LZ11 {
        return Err(Error::new(Kind::RomError, "not LZ11"));
    }
    let mut size = byte(blob, 1)? | byte(blob, 2)? << 8 | byte(blob, 3)? << 16;
    let mut at = 4;
    if size == 0 {
        size = binary::u32(blob, 4)?;
        at = 8;
    }
    if let Some(limit) = limit {
        size = size.min(limit);
    }
    let mut out = lz11_blocks(blob, size, &mut at).map_err(|error| {
        if error.is(Kind::IndexError) {
            Error::new(Kind::RomError, "LZ11 ends early")
        } else {
            error
        }
    })?;
    out.truncate(size as usize);
    Ok(out)
}

/// The body of `lz11`'s `try`: an IndexError is a blob that ends before its size is reached.
fn lz11_blocks(blob: &[u8], size: i64, at: &mut i64) -> Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();
    while (out.len() as i64) < size {
        let flags = byte(blob, *at)?;
        *at += 1;
        if flags == 0 && size - out.len() as i64 >= 8 {
            // Eight literals: common in images, and one slice instead of eight appends.
            if *at + 8 > blob.len() as i64 {
                return Err(Error::index(""));
            }
            out.extend_from_slice(py::slice(blob, *at, *at + 8));
            *at += 8;
            continue;
        }
        for bit in (0..=7).rev() {
            if out.len() as i64 >= size {
                break;
            }
            if flags >> bit & 1 == 0 {
                out.push(byte(blob, *at)? as u8);
                *at += 1;
                continue;
            }
            let first = byte(blob, *at)?;
            let (length, back);
            if first >> 4 == 0 {
                length = ((first & 0xF) << 4 | byte(blob, *at + 1)? >> 4) + 0x11;
                back = ((byte(blob, *at + 1)? & 0xF) << 8 | byte(blob, *at + 2)?) + 1;
                *at += 3;
            } else if first >> 4 == 1 {
                length =
                    ((first & 0xF) << 12 | byte(blob, *at + 1)? << 4 | byte(blob, *at + 2)? >> 4)
                        + 0x111;
                back = ((byte(blob, *at + 2)? & 0xF) << 8 | byte(blob, *at + 3)?) + 1;
                *at += 4;
            } else {
                length = (first >> 4) + 1;
                back = ((first & 0xF) << 8 | byte(blob, *at + 1)?) + 1;
                *at += 2;
            }
            if back > out.len() as i64 {
                return Err(Error::new(
                    Kind::RomError,
                    "LZ11 reaches back before its start",
                ));
            }
            // A reference that overlaps what it writes repeats its last `back` bytes; copied a
            // slice at a time, not a byte, the census of the ROM's textures took 58 s, not
            // (spike 147).
            let start = out.len() - back as usize;
            if back >= length {
                out.extend_from_within(start..start + length as usize);
            } else {
                let tail = out[start..].to_vec();
                for i in 0..length as usize {
                    out.push(tail[i % back as usize]);
                }
            }
        }
    }
    Ok(out)
}

/// `blob[at]`: an IndexError past the end.
#[track_caller]
fn byte(blob: &[u8], at: i64) -> Result<i64> {
    match usize::try_from(at).ok().and_then(|at| blob.get(at)) {
        Some(&b) => Ok(b as i64),
        None => Err(Error::index("index out of range")),
    }
}

/// `Rom.__init__`'s reads of the image: (program id, the entries by path).
fn index(file: &mut File) -> Result<(i64, IndexMap<String, Entry>)> {
    let (title_id, romfs) = header(file)?;
    let level3 = level3(file, romfs)?;
    let entries = walk(file, level3)?;
    Ok((title_id, entries.into_iter().collect()))
}

/// (program id, where the RomFS starts) for a decrypted NCSD image.
fn header<R: Read + Seek>(rom: &mut R) -> Result<(i64, i64)> {
    seek(rom, 0)?;
    let head = read(rom, UNIT)?;
    if py::slice(&head, 0x100, 0x104) != NCSD_MAGIC {
        return Err(Error::new(Kind::RomError, "not an NCSD image"));
    }
    let partition = binary::u32(&head, 0x120)? * UNIT;
    seek(rom, partition)?;
    let ncch = read(rom, UNIT)?;
    if py::slice(&ncch, 0x100, 0x104) != NCCH_MAGIC {
        return Err(Error::new(
            Kind::RomError,
            "its first partition is not an NCCH",
        ));
    }
    if byte(&ncch, 0x18F)? & NO_CRYPTO == 0 {
        return Err(Error::new(
            Kind::RomError,
            "encrypted; pa3ds reads decrypted images only",
        ));
    }
    let program = u64::from_le_bytes(binary::take(&ncch, 0x118)?) as i64;
    let offset = binary::u32(&ncch, 0x1B0)?;
    if offset == 0 {
        return Err(Error::new(Kind::RomError, "it has no RomFS"));
    }
    Ok((program, partition + offset * UNIT))
}

/// Where the RomFS's level 3 — the file system itself — begins.
fn level3<R: Read + Seek>(rom: &mut R, ivfc_at: i64) -> Result<i64> {
    seek(rom, ivfc_at)?;
    let ivfc = read(rom, 0x60)?;
    if py::slice(&ivfc, 0, 4) != IVFC_MAGIC {
        return Err(Error::new(
            Kind::RomError,
            "no IVFC header where the NCCH says the RomFS is",
        ));
    }
    let master = binary::u32(&ivfc, 0x08)?;
    let shift = binary::u32(&ivfc, 0x4C)?;
    // Python's int has no width. A shift this large is no IVFC header, and Python would have
    // run out of memory on it, which a panic stands in for.
    assert!(
        shift < 62,
        "IVFC block size shift {shift} is not a block size"
    );
    let block = 1_i64 << shift;
    let start = ivfc_at + py::floordiv(0x60 + master + block - 1, block) * block;
    seek(rom, start)?;
    let word = read(rom, 4)?;
    // `struct.unpack` and not `unpack_from`: its message differs.
    let &[a, b, c, d] = word.as_slice() else {
        return Err(Error::unpack("unpack requires a buffer of 4 bytes"));
    };
    if u32::from_le_bytes([a, b, c, d]) as i64 != LEVEL3_HEADER {
        return Err(Error::new(
            Kind::RomError,
            format!("no RomFS level 3 at {start:#x}"),
        ));
    }
    Ok(start)
}

/// Every file in the RomFS: a directory lists its first child directory and first file, and
/// each entry links to its next sibling.
///
/// Python yields these one by one and `Rom` consumes them all at once, so this returns them all.
fn walk<R: Read + Seek>(rom: &mut R, start: i64) -> Result<Vec<(String, Entry)>> {
    seek(rom, start)?;
    let header = read(rom, LEVEL3_HEADER)?;
    let raw = binary::take::<8>(&header, 0x0C)?;
    let (dir_meta, dir_size) = (word(&raw, 0), word(&raw, 4));
    let raw = binary::take::<8>(&header, 0x1C)?;
    let (file_meta, file_size) = (word(&raw, 0), word(&raw, 4));
    let data = start + binary::u32(&header, 0x24)?;
    seek(rom, start + dir_meta)?;
    let dirs = read(rom, dir_size)?;
    seek(rom, start + file_meta)?;
    let files = read(rom, file_size)?;
    let mut out: Vec<(String, Entry)> = Vec::new();
    let mut pending: Vec<(i64, String)> = vec![(0, String::new())];
    let mut visited: HashSet<i64> = HashSet::new();
    while let Some((offset, path)) = pending.pop() {
        if visited.contains(&offset) || offset + 0x18 > dirs.len() as i64 {
            return Err(Error::new(
                Kind::RomError,
                "the RomFS's directories do not hold together",
            ));
        }
        visited.insert(offset);
        let raw = binary::take::<24>(&dirs, offset)?;
        let (sibling, child, first, length) =
            (word(&raw, 4), word(&raw, 8), word(&raw, 12), word(&raw, 20));
        let name =
            py::decode_utf16le_strict(py::slice(&dirs, offset + 0x18, offset + 0x18 + length))?;
        let here = if !name.is_empty() {
            format!("{path}/{name}")
        } else {
            path.clone()
        };
        let mut at = first;
        while at != NONE {
            let raw = binary::take::<32>(&files, at)?;
            let (next_file, r#where, size, length) =
                (word(&raw, 4), wide(&raw, 8), wide(&raw, 16), word(&raw, 28));
            let name = py::decode_utf16le_strict(py::slice(&files, at + 0x20, at + 0x20 + length))?;
            out.push((
                format!("{here}/{name}").trim_start_matches('/').to_string(),
                Entry {
                    at: data + r#where,
                    size,
                },
            ));
            at = next_file;
        }
        if sibling != NONE && offset != 0 {
            pending.push((sibling, path.clone()));
        }
        if child != NONE {
            pending.push((child, here));
        }
    }
    Ok(out)
}

/// A QSettings string list as Azahar writes one: comma-separated, an item holding a comma
/// quoted, and a backslash doubled.
fn string_list(value: &str) -> Vec<String> {
    let value: Vec<char> = value.chars().collect();
    let mut items: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut at = 0;
    while at < value.len() {
        let char = value[at];
        if char == '\\' && at + 1 < value.len() {
            current.push(value[at + 1]);
            at += 2;
            continue;
        }
        if char == '"' {
            quoted = !quoted;
        } else if char == ',' && !quoted {
            items.push(py::strip(&current).to_string());
            current = String::new();
        } else {
            current.push(char);
        }
        at += 1;
    }
    items.push(py::strip(&current).to_string());
    items.into_iter().filter(|item| !item.is_empty()).collect()
}

/// `path.name`.
fn name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// `path.suffix`.
fn suffix(path: &Path) -> String {
    let name = name(path);
    match name.rfind('.') {
        Some(i) if 0 < i && i < name.len() - 1 => name[i..].to_string(),
        _ => String::new(),
    }
}

/// `f"{x:#x}"` for an int, which writes a negative one as `-0x10`.
fn hex(x: i64) -> String {
    if x < 0 {
        format!("-{:#x}", x.unsigned_abs())
    } else {
        format!("{x:#x}")
    }
}

/// `file.seek(to)`.
fn seek<R: Seek>(rom: &mut R, to: i64) -> Result<()> {
    rom.seek(SeekFrom::Start(to as u64))?;
    Ok(())
}

/// `file.read(size)`: up to `size` bytes, fewer only at the end of the file.
fn read<R: Read>(rom: &mut R, size: i64) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    rom.take(size as u64).read_to_end(&mut data)?;
    Ok(data)
}

/// `bytes.decode("utf-16-le", "ignore")`: a lone surrogate, or an odd byte at the end, is left
/// out. `py` has only "replace" and "strict".
fn decode_utf16le_ignore(data: &[u8]) -> String {
    let units = data
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair));
    char::decode_utf16(units)
        .filter_map(|unit| unit.ok())
        .collect()
}

/// A little-endian u32 out of bytes a `struct.unpack_from` already took, as an int.
fn word(raw: &[u8], at: usize) -> i64 {
    u32::from_le_bytes(raw[at..at + 4].try_into().expect("4 bytes")) as i64
}

/// A little-endian u64 likewise.
fn wide(raw: &[u8], at: usize) -> i64 {
    u64::from_le_bytes(raw[at..at + 8].try_into().expect("8 bytes")) as i64
}
