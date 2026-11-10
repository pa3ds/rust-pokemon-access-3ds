//! Who the people on a map are, out of the ROM: the kinds of trainer each sprite stands as
//! (spike 90), the story's people by sprite, and the Pokémon a model draws (spike 193).

use std::collections::HashSet;
use std::ops::Range;
use std::sync::LazyLock;

use indexmap::IndexMap;

use crate::core::binary::{self, u16, u32};
use crate::core::rom::{Rom, lz11, lz11_limit};
use crate::games::common::garc::{garc_files, garc_spans};
use crate::games::gen6::field::{mmodel, zonedata};
use crate::games::gen6::rom::layout::RomLayout;
use crate::py::{self, Error, Kind, Result};

/// A trainer record's battle: every trainer of a pair class — "Twins", "Young Couple" — fights
/// doubles, and no class of one person does throughout (spike 90).
pub const DOUBLE: i64 = 1;

/// `struct.calcsize(form)` and `struct.unpack_from(form, record, at)[0]` for the one-field
/// little-endian formats a trainer's class is read with.
fn class_form(form: &str) -> Result<(i64, bool)> {
    match form {
        "<B" | "<b" => Ok((1, form == "<b")),
        "<H" | "<h" => Ok((2, form == "<h")),
        "<I" | "<i" => Ok((4, form == "<i")),
        _ => Err(Error::unpack("bad char in struct format")),
    }
}

fn unpack_class(form: &str, record: &[u8], at: i64) -> Result<i64> {
    let (size, signed) = class_form(form)?;
    Ok(match (size, signed) {
        (1, false) => binary::take::<1>(record, at)?[0] as i64,
        (1, true) => binary::take::<1>(record, at)?[0] as i8 as i64,
        (2, false) => u16(record, at)?,
        (2, true) => binary::s16(record, at)?,
        (4, false) => u32(record, at)?,
        _ => binary::s32(record, at)?,
    })
}

/// Each trainer's class by trainer id, and the classes that are a pair of people: those whose
/// every trainer fights doubles. ORAS's `a/0/3/6` holds a small file a trainer with the class a
/// u16 at +0x02, X's `a/0/3/8` with the class a byte at +0x01 (spike 100).
pub fn trainer_classes(rom: &Rom, layout: &RomLayout) -> Result<(Vec<i64>, HashSet<i64>)> {
    let records = garc_files(&rom.read(&layout.trainers)?)?;
    let (at, form) = (layout.trainer_class.0, layout.trainer_class.1.as_str());
    let end = at + class_form(form)?.0;
    let mut classes: Vec<i64> = Vec::new();
    for record in &records {
        classes.push(if record.len() as i64 >= end {
            unpack_class(form, record, at)?
        } else {
            0
        });
    }
    let mut battles: IndexMap<i64, HashSet<i64>> = IndexMap::new();
    for (&kind, record) in classes.iter().zip(&records) {
        if record.len() as i64 > layout.trainer_battle {
            battles
                .entry(kind)
                .or_default()
                .insert(record[layout.trainer_battle as usize] as i64);
        }
    }
    Ok((
        classes,
        battles
            .into_iter()
            .filter(|(_kind, seen)| seen.len() == 1 && seen.contains(&DOUBLE))
            .map(|(kind, _seen)| kind)
            .collect(),
    ))
}

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

/// Character -> national number, for each Pokémon the field can draw: the character
/// `field.POKEMON_MODEL` + n is the archive's model `first` + n, whose file names the Pokémon
/// it draws (spike 193). Machoke's `0x103f` is model 509, `pm0067`; Whismur's beside Zinnia
/// `0x1039` is 503, `pm0293`; Rayquaza's in Sky Pillar `0x101f` is 477, `pm0384`. A model
/// naming no Pokémon or more than one is left out, and so is one LZ11 cannot unpack: most are
/// packed, Lisia's Altaria's among them. Where `species` is given, a model's tag is the
/// Pokémon it says (`tagged_species`), and one it does not have is left out.
pub fn pokemon_models(
    rom: &Rom,
    archive: &str,
    first: i64,
    species: Option<&IndexMap<i64, i64>>,
) -> Result<IndexMap<i64, i64>> {
    let (start, spans) = garc_spans(rom, archive)?;
    let mut out: IndexMap<i64, i64> = IndexMap::new();
    for (n, &span) in py::slice_from(&spans, first).iter().enumerate() {
        let named = model_tags(rom, archive, start, span)?;
        if named.len() == 1 {
            let tag = *named.iter().next().expect("one species");
            let number = match species {
                None => Some(tag),
                Some(species) => species.get(&tag).copied(),
            };
            if let Some(number) = number {
                out.insert(mmodel::POKEMON_MODEL + n as i64, number);
            }
        }
    }
    Ok(out)
}

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
    // Python's `numbers or range(...)`: an empty range is as good as none.
    let read = numbers
        .filter(|numbers| !numbers.is_empty())
        .unwrap_or(1..(head.len() as i64 / 4 + 1));
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

/// Sprite -> the kind of trainer it is, for every sprite the maps show two trainers or more
/// standing as, all of one class (spike 90): "Youngster", "Hiker". When they also share one name,
/// as grunts do, the name follows the class: "Team Magma Grunt", as the game says it in battle.
///
/// A trainer of a pair class says nothing of the sprite, since "Teammates" is two people and the
/// old man drawn as one of them in town is neither: the sprite is named by its other trainers,
/// as the Fairy Tale Girls' four are beside two pairs of Mysterious Sisters (spike 193). Left
/// out: a sprite whose other trainers are of more than one class, which the sprite is cannot be
/// told; and a sprite fewer than two of them stand as, which may be a look everybody shares that
/// happens to battle once.
pub fn trainer_sprites(
    zones: &[Vec<u8>],
    class_of: &[i64],
    pairs: &HashSet<i64>,
    classes: &[String],
    names: &[String],
    layout: &RomLayout,
) -> Result<IndexMap<i64, String>> {
    let mut standing: IndexMap<i64, Vec<i64>> = IndexMap::new();
    for data in py::slice(zones, 0, layout.zone_table) {
        if !data.is_empty() {
            for (sprite, trainer) in zonedata::npc_trainers(&lz11(data)?, class_of.len() as i64)? {
                if !pairs.contains(&class_of[trainer as usize]) {
                    standing.entry(sprite).or_default().push(trainer);
                }
            }
        }
    }
    let mut words: IndexMap<i64, String> = IndexMap::new();
    for (&sprite, trainers) in &standing {
        let kinds: HashSet<i64> = trainers
            .iter()
            .map(|&trainer| class_of[trainer as usize])
            .collect();
        if trainers.len() < 2 || kinds.len() != 1 {
            continue;
        }
        let kind = *kinds.iter().next().expect("one class");
        if !(0 <= kind && kind < classes.len() as i64) {
            continue;
        }
        let kind = &classes[kind as usize];
        let who: HashSet<&String> = trainers
            .iter()
            .filter(|&&trainer| trainer < names.len() as i64)
            .map(|&trainer| &names[trainer as usize])
            .collect();
        if kind.is_empty() {
            continue;
        }
        words.insert(
            sprite,
            if who.len() == 1 && who.iter().all(|name| !name.is_empty()) {
                format!("{kind} {}", who.iter().next().expect("one name"))
            } else {
                kind.clone()
            },
        );
    }
    Ok(words)
}
