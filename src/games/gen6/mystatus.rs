//! The save's trainer record, `savedata::MyStatus`: which language the game is in (spike 226).
//!
//! ```text
//! savedata::MyStatus   (in the heap from before the title screen)
//!     +0x08  the save's trainer record, whole (PKHeX's format):
//!            +0x04 u8 the version, 0x1b Omega Ruby, 0x18 X
//!            +0x05 u8 the player's gender, 0 a boy and 1 a girl, so the object's +0x0d
//!            +0x2d u8 the language, so the object's +0x35
//!            +0x48 the player's name, UTF-16, 13 units with its end, so the object's +0x50
//! ```
//!
//! The gender reads 0 in every Omega Ruby save of a boy on hand (2026-11-11), and 1 in a record
//! with no trainer yet, no name and no ID, so a gender is believed only beside a name. No girl's
//! save has been read: 1 for her is PKHeX's format.
//!
//! The language is the one the player chose as their game began, or on a new game the one it starts
//! with: 1 Japanese, 2 English, 3 French, 4 Italian, 5 German, 7 Spanish, 8 Korean, and in Sun, whose
//! record pa3ds cannot reach, 9 and 10 Chinese. A fan translation keeps another language's number:
//! the Chinese Omega Ruby says Japanese, whose text it replaced.
//!
//! Bound by:
//! - ADR-0040: the language is the save's.

use crate::core::discovery::Found;
use crate::py;

pub const CLASS: &str = "N8savedata8MyStatusE";
pub const GENDER: i64 = 0x0D;
pub const NAME: i64 = 0x50;
pub const NAME_SIZE: i64 = 26;
pub const LANGUAGE: i64 = 0x35;
/// The record's number for a language -> its code, the name of its folders under `data/` and its
/// key in a game's text archives.
pub const LANGUAGES: [(u8, &str); 9] = [
    (1, "ja"),
    (2, "en"),
    (3, "fr"),
    (4, "it"),
    (5, "de"),
    (7, "es"),
    (8, "ko"),
    (9, "zh-CN"),
    (10, "zh-TW"),
];

/// The language the save says, out of the pages a sweep has already read, or None where the sweep
/// found no record or it holds no language pa3ds knows.
pub fn language(found: &Found) -> Option<&'static str> {
    for address in found.of(&[CLASS]) {
        let Some(byte) = found.read(address + LANGUAGE, 1) else {
            continue;
        };
        if let Some((_, code)) = LANGUAGES.iter().find(|(number, _)| *number == byte[0]) {
            return Some(code);
        }
    }
    None
}

/// Whether the player is a girl, and their name, out of the pages a sweep has already read;
/// None where the sweep found no record, the player has no name yet or the gender is neither.
pub fn player(found: &Found) -> Option<(bool, String)> {
    for address in found.of(&[CLASS]) {
        let gender = found.read(address + GENDER, 1);
        let raw = found.read(address + NAME, NAME_SIZE);
        let name = match raw {
            Some(raw) if raw.len() as i64 == NAME_SIZE => py::decode_utf16le(raw)
                .split('\0')
                .next()
                .unwrap_or("")
                .to_string(),
            _ => String::new(),
        };
        if let Some(&[gender]) = gender
            && (gender == 0 || gender == 1)
            && !name.is_empty()
        {
            return Some((gender != 0, name));
        }
    }
    None
}
