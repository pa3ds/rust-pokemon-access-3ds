//! Who the people on a map are, out of the ROM: the kinds of trainer each sprite stands as
//! (spike 90), the story's people by sprite, and the Pokémon a model draws (spike 193), by the
//! species tag its files name (`common.species_tags`).

use std::collections::HashSet;

use indexmap::IndexMap;

use crate::core::binary::{self, u16, u32};
use crate::core::rom::{Rom, lz11};
use crate::games::common::garc::{garc_files, garc_spans};
use crate::games::common::species_tags::model_tags;
use crate::games::gen6::field::{mmodel, zonedata};
use crate::games::gen6::rom::layout::RomLayout;
use crate::py::{self, Error, Result};

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
