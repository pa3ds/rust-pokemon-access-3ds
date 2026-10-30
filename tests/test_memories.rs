//! The credits' Pokémon in Omega Ruby and Alpha Sapphire's save, by the nicknames the player gave
//! them (spike 222).

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use crate::core::discovery::Found;
use crate::games::common::pokemon;
use crate::games::gen6::game::MemoriesReader;
use crate::games::oras::memories;
use crate::py::Result;

const SAVE: i64 = 0x08100000;
const BOXES: i64 = 0x08C9A140;
const PARTY: i64 = 0x08200000;

/// A stored Pokémon, with a key whose bits 13 to 17 leave its blocks in order.
pub(crate) fn record(key: i64, species: i64, nickname: &str, nicknamed: bool) -> Vec<u8> {
    record_form(key, species, nickname, nicknamed, 0)
}

/// `record(key, species, nickname, nicknamed, form)`.
pub(crate) fn record_form(
    key: i64,
    species: i64,
    nickname: &str,
    nicknamed: bool,
    form: i64,
) -> Vec<u8> {
    let mut plain = vec![0u8; pokemon::RECORD_SIZE as usize];
    plain[0..4].copy_from_slice(&(key as u32).to_le_bytes());
    plain[8..10].copy_from_slice(&(species as u16).to_le_bytes());
    plain[pokemon::GENDER as usize] = (form << 3) as u8;
    let name: Vec<u8> = nickname
        .encode_utf16()
        .flat_map(|unit| unit.to_le_bytes())
        .collect();
    let at = pokemon::NICKNAME as usize;
    plain[at..at + name.len()].copy_from_slice(&name);
    let flags = if nicknamed { pokemon::NICKNAMED } else { 0 };
    let at = pokemon::FLAGS as usize;
    plain[at..at + 4].copy_from_slice(&(flags as u32).to_le_bytes());
    let checksum = plain[pokemon::PLAIN as usize..]
        .chunks(2)
        .map(|word| i64::from(u16::from_le_bytes([word[0], word[1]])))
        .sum::<i64>()
        & 0xFFFF;
    let at = pokemon::CHECKSUM as usize;
    plain[at..at + 2].copy_from_slice(&(checksum as u16).to_le_bytes());
    let mut out = plain[..pokemon::PLAIN as usize].to_vec();
    out.extend(pokemon::crypt(&plain[pokemon::PLAIN as usize..], key));
    out
}

/// `struct.pack("<IH", key, packed)`.
fn entry(key: i64, packed: i64) -> Vec<u8> {
    let mut out = (key as u32).to_le_bytes().to_vec();
    out.extend((packed as u16).to_le_bytes());
    out
}

/// Each moment keeps its Pokémon's key, which finds it in the party or the boxes as it is now:
/// the Torchic of Route 103 is Blaze. The rival's Mudkip, one never nicknamed and one traded away
/// have no nickname to say.
#[test]
fn test_a_moments_pokemon_is_called_by_the_nickname_the_player_gave_it() -> Result<()> {
    let memory = FakeMemory::new();
    for (moment, key, packed) in [
        (1, 0x101, 255),
        (2, 0x202, 258),
        (10, 0x303, 284),
        (16, 0x404, 130),
        (25, 0x505, 383 | 1 << memories::FORM_SHIFT),
    ] {
        memory.write(
            SAVE + memories::RECORD + moment * memories::ENTRY,
            &entry(key, packed),
        );
    }
    memory.write(
        PARTY + pokemon::RECORD_IN_PARAM,
        &record(0x101, 257, "Blaze", true),
    );
    memory.write(
        PARTY + 0x200 + pokemon::RECORD_IN_PARAM,
        &record(0x404, 130, "Gyarados", false),
    );
    memory.write(
        BOXES + memories::BOXES_AT + 3 * pokemon::RECORD_SIZE,
        &record(0x303, 284, "bubbles", true),
    );
    let reader = memories::Memories::new(memory.clone());
    reader.track(&Found {
        pages: IndexMap::new(),
        heap: IndexMap::from([
            (SAVE, memories::CLASS.to_string()),
            (BOXES, memories::BOXES.to_string()),
            (PARTY, pokemon::PARAM_CLASS.to_string()),
            (PARTY + 0x200, pokemon::PARAM_CLASS.to_string()),
        ]),
        partial: false,
    });
    let mut said: Vec<Option<String>> = Vec::new();
    for moment in [1, 2, 10, 16, 25, 4] {
        said.push(reader.nickname(moment)?);
    }
    assert_eq!(
        said,
        [
            Some("Blaze".to_string()),
            None,
            Some("bubbles".to_string()),
            None,
            None,
            None
        ]
    );
    assert_eq!(reader.pokemon(25)?, Some((383, 1)));
    Ok(())
}
