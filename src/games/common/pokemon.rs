//! A Pokémon as the game keeps it: its record, and the object that holds it (spike 65).
//!
//! The record is the save file's own format, as PKHeX documents it: 0xE8 bytes, of which the first
//! 8 are plain and the rest four 56-byte blocks, shuffled by the encryption constant and XORed
//! with a keystream seeded by it. A 16-bit sum of the plain blocks checks it. The battle stats,
//! level and HP, are another 0x1C bytes XORed with the same keystream started over. In memory the
//! game keeps each Pokémon, a wild one included, as a `pml::pokepara::PokemonParam`:
//!
//! ```text
//! PokemonParam   +0x04 -> its record, which sits at +0x40    +0x08 -> its battle stats
//! record         +0x00 u32 encryption constant    +0x06 u16 checksum of 0x08-0xE8, decrypted
//!   decrypted    +0x08 u16 species    +0x1d u8 gender in bits 1-2: 0 male, 1 female, 2 neither,
//!                                             and the form from bit 3 (spike D)
//!                +0x24 u8 each the contest conditions, coolness to toughness, then sheen
//!                +0x40 the nickname, UTF-16LE: a Pokémon never nicknamed carries its species name
//!                +0x74 u32 the IVs in bits 0-29, then whether it is an Egg (30) and whether it
//!                      was nicknamed (31): set for every one Orion named, clear for Metagross,
//!                      Gyarados and the five of his first box he never did (spike 222)
//! battle stats   +0x04 u8 level    +0x08 u16 HP    +0x0a u16 maximum HP
//! ```
//!
//! The format was read, not PKHeX's code (clean-room, ADR-0030).

use crate::core::binary::{u16, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::py::{self, Error, Result};

pub const PARAM_CLASS: &str = "N3pml8pokepara12PokemonParamE";
pub const PARAM_RECORD: i64 = 0x04;
/// Where a PokemonParam keeps its record: +0x04 pointing here is a validator.
pub const RECORD_IN_PARAM: i64 = 0x40;

pub const RECORD_SIZE: i64 = 0xE8;
/// The encryption constant, 2 unused bytes and the checksum are not encrypted.
pub const PLAIN: i64 = 0x08;
pub const BLOCK: i64 = 56;
pub const CHECKSUM: i64 = 0x06;
pub const SPECIES: i64 = 0x08;
pub const GENDER: i64 = 0x1D;
pub const NICKNAME: i64 = 0x40;
pub const NICKNAME_SIZE: i64 = 0x1A;
pub const FLAGS: i64 = 0x74;
pub const NICKNAMED: i64 = 1 << 31;
/// ORAS's last species. Sun keeps its Pokémon in the same record (spike 124), up to 802.
pub const SPECIES_MAX: i64 = 721;

/// The contest conditions, a byte each: coolness, beauty, cuteness, cleverness, toughness, then
/// sheen, in the decrypted record (Pikachu's 104, 70, 70, 70, 70, 0, movie33).
pub const CONTEST: i64 = 0x24;
pub const CONTEST_CONDITIONS: i64 = 5;

pub const MALE: i64 = 0;
pub const FEMALE: i64 = 1;

pub const LEVEL: i64 = 0x04;
pub const HP: i64 = 0x08;

/// Where blocks A, B, C and D are stored, by the constant's bits 13-17 mod 24: for 22, "2310"
/// stores A third, B fourth, C second and D first. A wrong entry still passes the checksum, a sum
/// that does not care about order, and garbles only the records with that value (spike 65).
pub const POSITIONS: [&str; 24] = [
    "0123", "0132", "0213", "0312", "0231", "0321", "1023", "1032", "2013", "3012", "2031", "3021",
    "1203", "1302", "2103", "3102", "2301", "3201", "1230", "1320", "2130", "3120", "2310", "3210",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pokemon {
    pub species: i64,
    pub nickname: String,
    pub gender: i64,
    /// Whether the nickname is the player's, not the species' name it carries otherwise.
    pub nicknamed: bool,
    pub form: i64,
}

/// XOR each halfword with the next keystream word from `seed`. Its own inverse.
pub fn crypt(data: &[u8], seed: i64) -> Vec<u8> {
    let mut seed = seed;
    let mut out: Vec<u8> = Vec::with_capacity(data.len() / 2 * 2);
    for word in data.as_chunks::<2>().0 {
        // The mask makes wrapping arithmetic the same as Python's unbounded ints.
        seed = seed.wrapping_mul(0x41C64E6D).wrapping_add(0x6073) & 0xFFFFFFFF;
        let word = i64::from(u16::from_le_bytes(*word));
        out.extend(((word ^ (seed >> 16)) as u16).to_le_bytes());
    }
    out
}

/// The record decrypted, its blocks back in order, or None when its checksum does not hold.
pub fn decrypt(raw: &[u8]) -> Option<Vec<u8>> {
    if (raw.len() as i64) < RECORD_SIZE {
        return None;
    }
    // A record of RECORD_SIZE bytes holds every field read here, so none of these can raise.
    let held = "a record of RECORD_SIZE bytes holds this";
    let constant = u32(raw, 0).expect(held);
    let body = crypt(py::slice(raw, PLAIN, RECORD_SIZE), constant);
    let order = POSITIONS[py::modulo((constant >> 13) & 31, 24) as usize].as_bytes();
    let mut plain = py::slice(raw, 0, PLAIN).to_vec();
    for block in 0..4 {
        let at = i64::from(order[block] - b'0');
        plain.extend(py::slice(&body, at * BLOCK, (at + 1) * BLOCK));
    }
    let total = plain[PLAIN as usize..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|word| i64::from(u16::from_le_bytes(*word)))
        .sum::<i64>()
        & 0xFFFF;
    if total == u16(raw, CHECKSUM).expect(held) {
        Some(plain)
    } else {
        None
    }
}

/// The Pokémon a decrypted record describes, or None when it is not one. `species_max` is the
/// last species the game has: ORAS's 721, Sun's 802 (`games.sm.battle`). Python's default for it
/// is [`SPECIES_MAX`].
///
/// Raises where `plain` is too short for the fields read, as Python's unpacking and indexing do.
pub fn parse(plain: &[u8], species_max: i64) -> Result<Option<Pokemon>> {
    let species = u16(plain, SPECIES)?;
    if !(0 < species && species <= species_max) {
        return Ok(None);
    }
    let nickname = py::decode_utf16le(py::slice(plain, NICKNAME, NICKNAME + NICKNAME_SIZE));
    let nickname = nickname.split('\0').next().unwrap_or_default();
    let gender = match usize::try_from(GENDER).ok().and_then(|at| plain.get(at)) {
        Some(&byte) => i64::from(byte),
        None => return Err(Error::index("index out of range")),
    };
    let flags = u32(plain, FLAGS)?;
    Ok(Some(Pokemon {
        species,
        nickname: nickname.to_string(),
        gender: (gender >> 1) & 3,
        nicknamed: flags & NICKNAMED != 0,
        form: gender >> 3,
    }))
}

/// The contest conditions of the Pokémon a `PokemonParam` holds, coolness to toughness, or
/// None when it is not one.
pub fn contest_conditions(
    memory: &dyn Memory,
    names: &ClassNames,
    param: i64,
) -> Result<Option<Vec<i64>>> {
    if names.of_object(param)?.as_deref() != Some(PARAM_CLASS) {
        return Ok(None);
    }
    let plain = decrypt(&memory.read(param + RECORD_IN_PARAM, RECORD_SIZE)?);
    Ok(plain.map(|plain| {
        py::slice(&plain, CONTEST, CONTEST + CONTEST_CONDITIONS)
            .iter()
            .map(|&byte| i64::from(byte))
            .collect()
    }))
}

/// The Pokémon of an encrypted record where a save keeps it, a box's slot, or None. Python's
/// default for `species_max` is [`SPECIES_MAX`].
pub fn stored(memory: &dyn Memory, record: i64, species_max: i64) -> Result<Option<Pokemon>> {
    match decrypt(&memory.read(record, RECORD_SIZE)?) {
        None => Ok(None),
        Some(plain) => parse(&plain, species_max),
    }
}

/// The Pokémon a `pml::pokepara::PokemonParam` holds, or None when it is not one. Python's default
/// for `species_max` is [`SPECIES_MAX`].
pub fn read(
    memory: &dyn Memory,
    names: &ClassNames,
    param: i64,
    species_max: i64,
) -> Result<Option<Pokemon>> {
    if names.of_object(param)?.as_deref() != Some(PARAM_CLASS) {
        return Ok(None);
    }
    if u32(&memory.read(param + PARAM_RECORD, 4)?, 0)? != param + RECORD_IN_PARAM {
        return Ok(None);
    }
    match decrypt(&memory.read(param + RECORD_IN_PARAM, RECORD_SIZE)?) {
        None => Ok(None),
        Some(plain) => parse(&plain, species_max),
    }
}
