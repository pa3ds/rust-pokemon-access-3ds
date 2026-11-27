//! Which Pokémon a model draws, by the species tag its files name: gen 6's and Sun's models alike
//! (spikes 193, 218, 291). Gen 6's people out of the ROM (`gen6.rom.people`) and Sun's
//! (`sm.gametext`) both read them.

use std::collections::HashSet;
use std::ops::Range;
use std::sync::LazyLock;

use indexmap::IndexMap;

use crate::core::binary::u16;
use crate::core::rom::{Rom, lz11_limit};
use crate::games::common::garc::garc_spans;
use crate::py::{self, Error, Kind, Result};

/// How a Pokémon's model names what it draws, by its national number: `pm0067` Machoke.
///
/// `[0-9]` is Python's bytes `\d`: ASCII digits only, where the `regex` crate's `\d` takes any
/// script's.
pub static SPECIES_TAG: LazyLock<regex::bytes::Regex> =
    LazyLock::new(|| regex::bytes::Regex::new(r"pm([0-9]{4})").expect("a valid pattern"));

/// How much of a field model is unpacked to read its names: every one of ORAS's 98 Pokémon
/// models names them in its first 12 KB, and unpacking them whole took 0.33 s of the start, this
/// much 0.07 s (spike 193).
pub const MODEL_NAMES: i64 = 0x4000;
/// How many files a battle model is (`tagged_species`).
pub const MODEL_FILES: i64 = 8;

/// `tagged_species(rom, archive)`: `files` and `tagged` at their defaults, `MODEL_FILES`, and
/// every species read.
pub fn tagged_species(rom: &Rom, archive: &str) -> Result<IndexMap<i64, i64>> {
    tagged_species_with(rom, archive, MODEL_FILES, MODEL_FILES, None)
}

/// Tag -> national number, out of the battle models, for a game that tags a Pokémon's models
/// apart from its number, as X does its Kalos Pokémon's: Fletchling, 661, is `pm0753`, and Litleo
/// `pm0704` (spike 218). File 0 gives each species from Bulbasaur four bytes, the first of its
/// models (a u16) and how many (a byte); model m is `files` files from 1 + files·m, the file
/// `tagged` of them tagged: X's eight, the last; Sun's nine, the second, its Stufful `pm0852`
/// (spike 291). With `numbers`, those species alone are read. A tag of two species, or a file of
/// two tags, is left out.
pub fn tagged_species_with(
    rom: &Rom,
    archive: &str,
    files: i64,
    tagged: i64,
    numbers: Option<Range<i64>>,
) -> Result<IndexMap<i64, i64>> {
    let (start, spans) = garc_spans(rom, archive)?;
    let (at, end) = *spans
        .first()
        .ok_or_else(|| Error::index("list index out of range"))?;
    let head = rom.read_part(archive, start + at, end - at)?;
    let mut found: IndexMap<i64, HashSet<i64>> = IndexMap::new();
    let read = numbers.unwrap_or(1..(head.len() as i64 / 4 + 1));
    for number in read {
        if !(0 < number && number <= head.len() as i64 / 4) {
            continue;
        }
        let model = u16(&head, (number - 1) * 4)?;
        let count = head[((number - 1) * 4 + 2) as usize];
        let file = files * model + tagged;
        if count != 0 && file < spans.len() as i64 {
            let tags = model_tags(rom, archive, start, spans[file as usize])?;
            if tags.len() == 1 {
                let tag = *tags.iter().next().expect("one tag");
                found.entry(tag).or_default().insert(number);
            }
        }
    }
    Ok(found
        .into_iter()
        .filter(|(_, numbers)| numbers.len() == 1)
        .map(|(tag, numbers)| (tag, *numbers.iter().next().expect("one number")))
        .collect())
}

/// The species tags in the first `MODEL_NAMES` bytes of one of an archive's files, none where
/// LZ11 cannot unpack it.
pub fn model_tags(rom: &Rom, archive: &str, start: i64, span: (i64, i64)) -> Result<HashSet<i64>> {
    let (at, end) = span;
    let data = rom.read_part(archive, start + at, end - at)?;
    let data = if data.first() == Some(&0x11) {
        match lz11_limit(&data, Some(MODEL_NAMES)) {
            Ok(data) => data,
            Err(error)
                if error.is(Kind::RomError)
                    || error.is(Kind::ValueError)
                    || error.is(Kind::IndexError) =>
            {
                return Ok(HashSet::new());
            }
            Err(error) => return Err(error),
        }
    } else {
        py::slice(&data, 0, MODEL_NAMES).to_vec()
    };
    Ok(SPECIES_TAG
        .captures_iter(&data)
        .map(|found| {
            found[1]
                .iter()
                .fold(0, |number, digit| number * 10 + (digit - b'0') as i64)
        })
        .collect())
}
