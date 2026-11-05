//! Pictures named by the image they draw, where no layout file names it (spike 147).
//!
//! Some screens swap a picture's texture as they run, so its layout file names the wrong one or none:
//! the Pokédex's OTHER LANGUAGES tabs, and the entry's type. What a picture draws now is at its
//! material (`lyt.PICTURE_MATERIAL`): the image's physical address, the size it is stored at and the
//! GPU's format, which together are the whole image. **The image in memory is the ROM file's, byte
//! for byte**, so an index of the ROM's textures by a hash of their image names it.
//!
//! The index is built from the player's own ROM, never shipped: a copy of the game whose pictures
//! differ indexes its own. It is built once, in a process of its own so reading the game goes on at
//! full speed meanwhile, and kept as TOML (ADR-0031) in `<pa3ds>/user/cache/textures/<rom>.toml`,
//! beside the player's names (ADR-0032); deleting it only has it built again. It holds every
//! language's copy (`garc.garc_variants`), and names are file names, the same in every language, so
//! one file serves whichever language the ROM is played in.
//!
//! Only textures inside a `darc` are indexed: a texture with no file name gives nothing to say. And
//! only the archives a game lists (`games.oras.TEXTURE_ARCHIVES`): the whole of Omega Ruby took 554 s,
//! its 131 archives with named textures 12.7 s, 8.5 of them the title screen's; X's 112, 3.9 s
//! (`games.xy.TEXTURE_ARCHIVES`, spike A). A game that lists none has no index.
//!
//! The Rust runtime builds the index on a thread, not in a process: Python used a process only to
//! get out from under its interpreter lock. What is logged, when the index is ready and the file
//! written are the same.
//!
//! Bound by:
//! - ADR-0036: a picture is named by its image, indexed from the player's own ROM, and no image
//!   over 4 KB is read.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;
use std::sync::LazyLock;
use std::thread::JoinHandle;

use indexmap::IndexMap;
use regex::Regex;
use sha1::{Digest, Sha1};

use crate::core::binary::{u16, u32};
use crate::core::memory::Memory;
use crate::core::rom::{Rom, lz11};
use crate::core::toml;
use crate::core::words::own;
use crate::games::common::garc::{GARC_MAGIC, garc_variants};
use crate::games::common::layout_file::{TEXTURE, darc_entries};
use crate::games::common::{lyt, pictures};
use crate::games::gen6::rom::archives::GameText;
use crate::py::{self, Kind, Result};

/// Bits a pixel by the GPU's texture format, the number a material keeps (`lyt.MATERIAL_FORMAT`):
/// RGBA8 0, RGB8, RGBA5551, RGB565, RGBA4, IA8, HILO8, I8, A8, IA4, I4, A4, ETC1, ETC1A4 0xD.
///
/// Python's `dict`, as (format, bits) pairs: it is only ever looked up.
pub const BITS: [(i64, i64); 14] = [
    (0, 32),
    (1, 24),
    (2, 16),
    (3, 16),
    (4, 16),
    (5, 16),
    (6, 16),
    (7, 8),
    (8, 8),
    (9, 8),
    (0xA, 4),
    (0xB, 4),
    (0xC, 4),
    (0xD, 8),
];
/// The largest image read: every texture a rule in `said` names is 2 or 4 KB in Omega Ruby — the
/// type and category icons, the status icons, the language tabs — so a larger image can be named by
/// nothing, and is not read. Reading one is not only wasted: the image is in the GPU's memory, and
/// Azahar flushes its GPU's copy of what the RPC reads from the RPC's own thread, racing the thread
/// that draws. A battle's start crashed it 5 times in 12 while pa3ds read a 128 KB image in video
/// memory the GPU was drawing into; the ROM has 66 of that size, none of them named (spike 149).
pub const MAX_IMAGE: i64 = 0x1000;
/// (physical low, high, where the game's process sees it): FCRAM as the linear heap, and video
/// memory, which the RPC reads as it reads anything else (spike 147).
pub const PHYSICAL: [(i64, i64, i64); 2] = [
    (0x20000000, 0x28000000, 0x14000000),
    (0x18000000, 0x18600000, 0x1F000000),
];

pub const DARC: &[u8] = b"darc";
/// A `darc`'s byte order mark and header size, which tell a real header from the word in data.
pub const DARC_HEAD: &[u8] = b"\xff\xfe\x1c\x00";
pub const DARC_SIZE: i64 = 0x0C;
/// A `.bclim` ends in its header, "CLIM", and an `imag` block whose last word is the image's
/// length: the image itself is the file's start.
pub const CLIM_FOOTER: i64 = 0x28;
pub const LZ11: i64 = 0x11;
pub const MAX_DEPTH: i64 = 4;

pub const SECTION: &str = "textures";

/// The Pokédex tabs' language codes in the order the game names the languages
/// (`GameText.languages`): `p_zkn_word_button_fra` is French, said as the game says it.
pub const LANGUAGES: [&str; 7] = ["eng", "spa", "fra", "ger", "ita", "jpn", "kor"];
/// A Pokédex language tab, drawn as two pictures, the second's texture ending in 2 (spike 147).
///
/// Written for `fullmatch`: anchored at both ends. Python's `re.match` of the other patterns is
/// anchored at the start only.
pub static TAB: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\Ap_zkn_word_button_([a-z]{3})2?\z").expect("a pattern"));
/// A type's icon, by the game's number for the type: `type_icon_09_honoo` is Fire (spike 147).
pub static TYPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A(?:s_)?type_icon_(\d\d)_").expect("a pattern"));
/// A move category's icon, 00 status, 01 physical, 02 special (`GameText.categories`).
pub static CATEGORY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\Adamage_type_icon_(\d\d)_").expect("a pattern"));
/// A move's contest category on a contest's move list, 00 coolness to 04 toughness
/// (`GameText.contest_categories`, spike 196).
///
/// Written for `fullmatch`, as `TAB` is.
pub static CONTEST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\Acontest_icon_(\d\d)\z").expect("a pattern"));
/// A status condition's icon, from 01 paralysis to 05 poisoned, and 06 badly poisoned, which the
/// game says as poisoned (`GameText.conditions`). Every screen that draws one names paralysis's in
/// its layout file and swaps it for the Pokémon's own: the battle's HP boxes, its party list, the
/// summary (spike 147).
pub static CONDITION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\Asick_icon_(\d\d)_").expect("a pattern"));
pub const BADLY_POISONED: i64 = 6;
pub const POISONED: i64 = 5;
pub const POKERUS: &str = "pokerus_icon";
/// Pokérus's line among the conditions' names.
pub const POKERUS_LINE: i64 = 6;

/// Where the game's process sees a physical address, or None for one it does not.
pub fn r#virtual(physical: i64) -> Option<i64> {
    for (low, high, seen) in PHYSICAL {
        if low <= physical && physical < high {
            return Some(physical - low + seen);
        }
    }
    None
}

/// Where the image a picture draws now is, whatever its size, or None: for naming it by the
/// file of its layout's archive it is (`LayoutFiles.image_at`) without reading it.
pub fn drawn_at(memory: &dyn Memory, picture: i64) -> Result<Option<i64>> {
    let material = u32(&memory.read(picture + lyt::PICTURE_MATERIAL, 4)?, 0)?;
    if material == 0 {
        return Ok(None);
    }
    Ok(r#virtual(u32(
        &memory.read(material + lyt::MATERIAL_TEXTURE, 4)?,
        0,
    )?))
}

/// Where the image a picture draws now is, and its length; None where its material does not
/// say one.
pub fn drawn(memory: &dyn Memory, picture: i64) -> Result<Option<(i64, i64)>> {
    let material = u32(&memory.read(picture + lyt::PICTURE_MATERIAL, 4)?, 0)?;
    if material == 0 {
        return Ok(None);
    }
    let record = memory.read(
        material + lyt::MATERIAL_TEXTURE,
        lyt::MATERIAL_FORMAT + 4 - lyt::MATERIAL_TEXTURE,
    )?;
    let at = r#virtual(u32(&record, 0)?);
    let stored = lyt::MATERIAL_STORED - lyt::MATERIAL_TEXTURE;
    let (width, height) = (u16(&record, stored)?, u16(&record, stored + 2)?);
    let format = u32(&record, lyt::MATERIAL_FORMAT - lyt::MATERIAL_TEXTURE)?;
    let bits = BITS
        .iter()
        .find(|&&(held, _)| held == format)
        .map(|&(_, bits)| bits);
    let (Some(at), Some(bits)) = (at, bits) else {
        return Ok(None);
    };
    if width == 0 || height == 0 {
        return Ok(None);
    }
    let size = py::floordiv(width * height * bits, 8);
    Ok(if size <= MAX_IMAGE {
        Some((at, size))
    } else {
        None
    })
}

/// The whole image a picture draws now, or None where its material does not say one.
pub fn image(memory: &dyn Memory, picture: i64) -> Result<Option<Vec<u8>>> {
    match drawn(memory, picture)? {
        Some((at, size)) => Ok(Some(memory.read(at, size)?)),
        None => Ok(None),
    }
}

/// `hashlib.sha1(data).hexdigest()`.
pub fn digest(data: &[u8]) -> String {
    Sha1::digest(data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The files of a `darc` archive whose header is at the start of `data`, with their names.
///
/// Borrowed where Python copies each file's bytes: nothing here changes them.
pub fn darc_files(data: &[u8]) -> Result<Vec<(String, &[u8])>> {
    Ok(darc_entries(data)?
        .into_iter()
        .map(|(name, start, length)| (name, py::slice(data, start, start + length)))
        .collect())
}

/// A `.bclim`'s image, or None when it is not one.
pub fn texture_image(data: &[u8]) -> Option<&[u8]> {
    let foot = py::slice_from(data, -CLIM_FOOTER);
    if (foot.len() as i64) < CLIM_FOOTER
        || py::slice(foot, 0, 4) != b"CLIM"
        || py::slice(foot, 0x14, 0x18) != b"imag"
    {
        return None;
    }
    // The foot is `CLIM_FOOTER` bytes, which hold this word.
    let size = u32(foot, CLIM_FOOTER - 4).expect("a foot of CLIM_FOOTER bytes holds this");
    if size <= data.len() as i64 - CLIM_FOOTER {
        Some(py::slice(data, 0, size))
    } else {
        None
    }
}

/// (name, image) of every texture in a `darc` somewhere inside `blob`, through GARCs and their
/// every copy, LZ11, and the packs whose darcs follow a table of names (34 KB in, `a/1/0/3`).
///
/// Python's generator, as a walk that calls `found(name, image)` for each one it would yield, in
/// the order it would yield them, and raises where the generator would have raised.
pub fn named_textures(blob: &[u8], depth: i64, found: &mut dyn FnMut(&str, &[u8])) -> Result<()> {
    if depth > MAX_DEPTH || blob.len() < 8 {
        return Ok(());
    }
    if blob[0] as i64 == LZ11 && py::slice(blob, 4, 8) != DARC {
        let unpacked = match lz11(blob) {
            Ok(unpacked) => unpacked,
            Err(error) if error.is(Kind::RomError) => return Ok(()),
            Err(error) => return Err(error),
        };
        return named_textures(&unpacked, depth + 1, found);
    }
    if py::slice(blob, 0, 4) == GARC_MAGIC {
        let entries = match garc_variants(blob) {
            Ok(entries) => entries,
            Err(error) if error.is(Kind::RomError) => return Ok(()),
            Err(error) => return Err(error),
        };
        for copies in entries {
            for file in copies.values() {
                named_textures(file, depth + 1, found)?;
            }
        }
        return Ok(());
    }
    let mut at = find(blob, DARC, 0);
    while let Some(here) = at {
        let start = here as i64;
        if py::slice(blob, start + 4, start + 8) == DARC_HEAD {
            let data = py::slice(blob, start, start + u32(blob, start + DARC_SIZE)?);
            let files = match darc_files(data) {
                Ok(files) => files,
                Err(error) if error.is(Kind::StructError) => Vec::new(),
                Err(error) => return Err(error),
            };
            for (name, file) in files {
                if name.ends_with(TEXTURE) {
                    if let Some(image) = texture_image(file) {
                        found(&name, image);
                    }
                } else {
                    named_textures(file, depth + 1, found)?;
                }
            }
        }
        at = find(blob, DARC, here + 4);
    }
    Ok(())
}

/// Hash of an image -> the names of every texture drawing it, space-separated.
pub fn census(rom: &Rom, archives: &[impl AsRef<str>]) -> Result<IndexMap<String, String>> {
    // A `set` of strings, sorted before it is joined: a `BTreeSet` is that, in code point order.
    let mut names: IndexMap<String, BTreeSet<String>> = IndexMap::new();
    for path in archives {
        let blob = match rom.read(path.as_ref()) {
            Ok(blob) => blob,
            Err(error) if error.is(Kind::RomError) => continue,
            Err(error) => return Err(error),
        };
        named_textures(&blob, 0, &mut |name, data| {
            names
                .entry(digest(data))
                .or_default()
                .insert(name.strip_suffix(TEXTURE).unwrap_or(name).to_string());
        })?;
    }
    Ok(names
        .into_iter()
        .map(|(key, found)| (key, found.into_iter().collect::<Vec<_>>().join(" ")))
        .collect())
}

pub fn render(index: &IndexMap<String, String>, title: &str, rom: &str) -> String {
    let named = if title.is_empty() { rom } else { title };
    let lines = toml::comment([
        format!("The textures of {named}, ROM {rom}, by a hash of the image each draws."),
        String::new(),
        "pa3ds builds this from your ROM the first time it reads it, to name pictures the game"
            .to_string(),
        "fills in as it runs. Nothing here is yours to edit: delete it and it is built again."
            .to_string(),
    ]);
    let mut out = lines;
    out.push(String::new());
    out.push(format!("[{SECTION}]"));
    out.extend(toml::pairs(index));
    out.join("\n") + "\n"
}

/// The index kept at `path`, or empty where there is none yet.
pub fn load(path: &Path) -> Result<IndexMap<String, String>> {
    Ok(toml::strings(toml::read(path).get(SECTION)))
}

/// Build the index and keep it at `path`: what the thread started for it runs.
pub fn build(
    rom_path: &Path,
    archives: &[String],
    path: &Path,
    title: &str,
    rom: &str,
) -> Result<()> {
    let index = {
        let opened = Rom::new(rom_path)?;
        census(&opened, archives)?
    };
    toml::write(path, &render(&index, title, rom))?;
    Ok(())
}

/// The names of the textures a picture draws now, by its image; none where it is not indexed.
pub fn names(
    index: &IndexMap<String, String>,
    memory: &dyn Memory,
    picture: i64,
) -> Result<Vec<String>> {
    Ok(match image(memory, picture)? {
        Some(found) => py::split(index.get(&digest(&found)).map_or("", String::as_str))
            .into_iter()
            .map(String::from)
            .collect(),
        None => Vec::new(),
    })
}

/// What the game calls what a picture draws, from the name of its texture: a language tab's
/// language, a type's, a move category's or a status condition's icon; or the sign of what the
/// DexNav draws in place of words (`pictures.DRAWN`). None for a texture no rule reads, or one
/// the ROM has no word for.
///
/// A `sick_icon_00_` texture indexes the conditions from -1: the last one, or an IndexError where
/// the ROM names none, as Python's indexing does.
pub fn said(textures: &[impl AsRef<str>], text: &GameText) -> Result<Option<String>> {
    for texture in textures {
        let texture = texture.as_ref();
        if let Some(sign) = pictures::drawn(texture) {
            return Ok(Some(sign.to_string()));
        }
        if let Some(place) = pictures::place(texture) {
            return Ok(Some(own(place, &[])));
        }
        let (index, words): (i64, &[String]);
        let tab = TAB
            .captures(texture)
            .map(|tab| tab[1].to_string())
            .filter(|language| LANGUAGES.contains(&language.as_str()));
        if let Some(language) = tab {
            index = LANGUAGES
                .iter()
                .position(|held| *held == language)
                .expect("a language of LANGUAGES") as i64;
            words = &text.languages;
        } else if let Some(kind) = TYPE.captures(texture) {
            index = int(&kind[1]);
            words = &text.types;
        } else if let Some(category) = CATEGORY.captures(texture) {
            index = int(&category[1]);
            words = &text.categories;
        } else if let Some(contest) = CONTEST.captures(texture) {
            index = int(&contest[1]);
            words = &text.contest_categories;
        } else if let Some(condition) = CONDITION.captures(texture) {
            let number = int(&condition[1]);
            index = (if number == BADLY_POISONED {
                POISONED
            } else {
                number
            }) - 1;
            words = &text.conditions;
        } else if texture == POKERUS {
            index = POKERUS_LINE;
            words = &text.conditions;
        } else {
            continue;
        }
        if let Some(word) = usize::try_from(index).ok().and_then(|at| words.get(at))
            && !word.is_empty()
        {
            return Ok(Some(word.clone()));
        }
    }
    Ok(None)
}

/// The index of one ROM, read once it is there: at the start, or once the thread building it
/// is done. Until then, and where it could not be built, no picture is named by its image.
///
/// Shared (`Rc<Index>`) as every reader's are, so its methods take `&self`.
pub struct Index {
    path: PathBuf,
    text: Rc<GameText>,
    building: Option<JoinHandle<()>>,
    index: RefCell<Option<IndexMap<String, String>>>,
    /// (address, length) of an image -> its names: hashed once, not every poll a cursor
    /// rests on it.
    seen: RefCell<HashMap<(i64, i64), Vec<String>>>,
}

impl Index {
    /// `Index(path, text, building)`.
    pub fn new(path: PathBuf, text: Rc<GameText>, building: Option<JoinHandle<()>>) -> Self {
        Index {
            path,
            text,
            building,
            index: RefCell::new(None),
            seen: RefCell::new(HashMap::new()),
        }
    }

    /// `Index(path, text)`: the index is already there.
    pub fn without_building(path: PathBuf, text: Rc<GameText>) -> Self {
        Index::new(path, text, None)
    }

    /// What the game calls what this picture draws now, or None.
    pub fn said(&self, memory: &dyn Memory, picture: i64) -> Result<Option<String>> {
        let found = self.names(memory, picture)?;
        if !found.is_empty() && self.text.loaded() {
            said(&found, &self.text)
        } else {
            Ok(None)
        }
    }

    pub fn names(&self, memory: &dyn Memory, picture: i64) -> Result<Vec<String>> {
        if self.index.borrow().is_none() {
            if self
                .building
                .as_ref()
                .is_some_and(|building| !building.is_finished())
            {
                return Ok(Vec::new());
            }
            let loaded = load(&self.path)?;
            *self.index.borrow_mut() = Some(loaded);
        }
        let held = self.index.borrow();
        let index = held.as_ref().expect("the index was just loaded");
        let at = if !index.is_empty() {
            drawn(memory, picture)?
        } else {
            None
        };
        let Some(at) = at else {
            return Ok(Vec::new());
        };
        if !self.seen.borrow().contains_key(&at) {
            let said = py::split(
                index
                    .get(&digest(&memory.read(at.0, at.1)?))
                    .map_or("", String::as_str),
            )
            .into_iter()
            .map(String::from)
            .collect();
            self.seen.borrow_mut().insert(at, said);
        }
        Ok(self.seen.borrow()[&at].clone())
    }
}

/// The index kept at `path`, building it first on a thread of its own where there is none;
/// None where it cannot be had, and the log says why.
pub fn start(
    path: &Path,
    rom_path: Option<&Path>,
    archives: &[String],
    title: &str,
    rom: &str,
    text: Rc<GameText>,
    log: &dyn Fn(&str),
) -> Result<Option<Index>> {
    if path.exists() {
        return Ok(Some(Index::without_building(path.to_path_buf(), text)));
    }
    let Some(rom_path) = rom_path else {
        log("no ROM to index the textures of; pictures the game fills in go unnamed");
        return Ok(None);
    };
    let (own_rom, own_archives, own_path) = (
        rom_path.to_path_buf(),
        archives.to_vec(),
        path.to_path_buf(),
    );
    let (own_title, own_name) = (title.to_string(), rom.to_string());
    let building = std::thread::Builder::new()
        .name("textures".to_string())
        .spawn(move || {
            // The process Python started printed its traceback to the console and died; the
            // index is then read as there being none.
            if let Err(error) = build(&own_rom, &own_archives, &own_path, &own_title, &own_name) {
                eprintln!(
                    "indexing the textures failed: {error:?} at {}",
                    error.fault()
                );
            }
        })?;
    let name = rom_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    log(&format!(
        "indexing the textures of {name} into {}, once",
        shown(path)
    ));
    Ok(Some(Index::new(path.to_path_buf(), text, Some(building))))
}

/// `data.find(needle, start)`, as an index.
fn find(data: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    let first = needle[0];
    let mut from = start;
    while let Some(hit) = data.get(from..)?.iter().position(|&byte| byte == first) {
        let at = from + hit;
        if data[at..].starts_with(needle) {
            return Some(at);
        }
        from = at + 1;
    }
    None
}

/// `int(text)` of the two decimal digits a pattern took, which Python's `\d` lets be any script's:
/// a digit's value is its place in the run of ten the script keeps together.
fn int(text: &str) -> i64 {
    text.chars()
        .fold(0, |number, digit| number * 10 + value(digit))
}

fn value(digit: char) -> i64 {
    use unicode_general_category::{GeneralCategory::DecimalNumber, get_general_category};
    let mut before = 0;
    let mut at = digit as u32;
    while at > 0 && char::from_u32(at - 1).is_some_and(|c| get_general_category(c) == DecimalNumber)
    {
        before += 1;
        at -= 1;
    }
    before % 10
}

/// `str(path)` as `pathlib` writes it on Windows: separators made backslashes, doubled ones and
/// `.` parts gone. Rust's `display` leaves a path as it was built.
fn shown(path: &Path) -> String {
    let path: PathBuf = path
        .components()
        .filter(|part| !matches!(part, Component::CurDir))
        .collect();
    let shown = path.to_string_lossy().into_owned();
    if shown.is_empty() {
        ".".to_string()
    } else {
        shown
    }
}
