//! GARC archives, and the text files in them: how every one of the games keeps its words and its
//! data in the ROM (spike 74; Sun's version, spike 111).
//!
//! ```text
//! GARC  +0x00 "CRAG"  +0x04 header size  +0x0a version  then the data offset and the file size,
//!       at +0x10 and +0x14 in version 4 and at +0x18 and +0x1c in version 6
//! FATO  +0x08 count            FATB  +0x0c on: per file a bitmask, and (start, end, length)
//!                              for each bit set; the first is the file
//! text  +0x02 lines  +0x0c the section; a section is a u32 length, then per line (u32 offset
//!       from the section, u16 length, u16 flags). A line is UTF-16 XORed with a key stream: the
//!       first line's key is 0x7c89, each line after adds 0x2983, and inside a line the key
//!       rotates left three bits a character.
//! ```
//!
//! A text file's lines keep their control sequences (`text`); `plain` takes them out.

use indexmap::IndexMap;

use crate::core::binary::{self, u16, u32};
use crate::core::rom::Rom;
use crate::games::common::text::{MARKER, Token, tokens};
use crate::py::{self, Error, Kind, Result};

pub const GARC_MAGIC: &[u8] = b"CRAG";
pub const FATO_MAGIC: &[u8] = b"OTAF";
pub const FATB_MAGIC: &[u8] = b"BTAF";
pub const FIRST_KEY: i64 = 0x7C89;
pub const NEXT_KEY: i64 = 0x2983;
/// The control choosing between a word's singular and plural endings (`singular`).
pub const NUMBER_FORM: i64 = 0x1101;

/// The files inside a GARC archive, in order, each entry's first copy. A RomError if it is not
/// one.
pub fn garc_files(blob: &[u8]) -> Result<Vec<Vec<u8>>> {
    Ok(garc_variants(blob)?
        .into_iter()
        .map(|copies| copies.into_values().next().unwrap_or_default())
        .collect())
}

/// Every entry of a GARC as its copies by variant, in order. A RomError if it is not one.
///
/// An entry holds a copy a language, its FATB bits saying which, by the game's language number
/// (1 Japanese, 2 English, 3 French, 4 Italian, 5 German, 7 Spanish, 8 Korean), with bit 0 the
/// copy any language without its own draws. The English game's Pokédex tabs FRE, GER and KOR are
/// in bit 2, where bit 0 has FRA, DEU and a Korean tab's (spike 147).
pub fn garc_variants(blob: &[u8]) -> Result<Vec<IndexMap<i64, Vec<u8>>>> {
    if py::slice(blob, 0, 4) != GARC_MAGIC {
        return Err(Error::new(Kind::RomError, "not a GARC archive"));
    }
    // The data offset is at +0x10 in version 0x0400, ORAS's and X's, and in 0x0600, Sun's, whose
    // header is longer by the largest file's unpadded size and the padding (spike 111).
    let header = u32(blob, 0x04)?;
    let data = u32(blob, 0x10)?;
    if py::slice(blob, header, header + 4) != FATO_MAGIC {
        return Err(Error::new(
            Kind::RomError,
            "a GARC without its FATO where its header says",
        ));
    }
    let count = u16(blob, header + 0x08)?;
    let fatb = header + u32(blob, header + 0x04)?;
    if py::slice(blob, fatb, fatb + 4) != FATB_MAGIC {
        return Err(Error::new(
            Kind::RomError,
            "a GARC without its FATB after its FATO",
        ));
    }
    let mut entries: Vec<IndexMap<i64, Vec<u8>>> = Vec::new();
    let mut at = fatb + 0x0C;
    for _ in 0..count {
        let bits = u32(blob, at)?;
        at += 4;
        let mut copies: IndexMap<i64, Vec<u8>> = IndexMap::new();
        for bit in 0..32 {
            if bits >> bit & 1 != 0 {
                let raw = binary::take::<12>(blob, at)?;
                let (start, end) = (word(&raw, 0), word(&raw, 4));
                at += 12;
                copies.insert(bit, py::slice(blob, data + start, data + end).to_vec());
            }
        }
        entries.push(copies);
    }
    Ok(entries)
}

/// Where a GARC's files start, and each file's (start, end) from there, read without the
/// files themselves: Sun's event archive is 332 MB, and its readers want a few of its files.
/// The first of a file's versions is the one read, as `garc_files` does. A RomError where the
/// archive is not a GARC.
pub fn garc_spans(image: &Rom, archive: &str) -> Result<(i64, Vec<(i64, i64)>)> {
    let head = image.read_part(archive, 0, 0x14)?;
    if py::slice(&head, 0, 4) != GARC_MAGIC {
        return Err(Error::new(
            Kind::RomError,
            format!("{archive} is not a GARC archive"),
        ));
    }
    let (header, data) = (u32(&head, 4)?, u32(&head, 0x10)?);
    let table = image.read_part(archive, 0, data)?;
    if py::slice(&table, header, header + 4) != FATO_MAGIC {
        return Err(Error::new(
            Kind::RomError,
            format!("{archive}: no FATO where its header says"),
        ));
    }
    let count = u16(&table, header + 8)?;
    let fatb = header + u32(&table, header + 4)?;
    if py::slice(&table, fatb, fatb + 4) != FATB_MAGIC {
        return Err(Error::new(
            Kind::RomError,
            format!("{archive}: no FATB after its FATO"),
        ));
    }
    let mut spans: Vec<(i64, i64)> = Vec::new();
    let mut at = fatb + 0x0C;
    for _ in 0..count {
        let bits = u32(&table, at)?;
        at += 4;
        let mut first: Option<(i64, i64)> = None;
        for bit in 0..32 {
            if bits >> bit & 1 != 0 {
                if first.is_none() {
                    first = Some((u32(&table, at)?, u32(&table, at + 4)?));
                }
                at += 12;
            }
        }
        spans.push(first.unwrap_or((0, 0)));
    }
    Ok((data, spans))
}

/// The lines of one text file, each as UTF-16LE with its control sequences left in and its
/// trailing zeros taken off, short of a control that ends it (`_kept`).
pub fn message_lines(blob: &[u8]) -> Result<Vec<Vec<u8>>> {
    if blob.len() < 0x10 {
        return Err(Error::new(Kind::RomError, "too short for a text file"));
    }
    let raw = binary::take::<16>(blob, 0)?;
    let sections = u16::from_le_bytes([raw[0], raw[1]]) as i64;
    let count = u16::from_le_bytes([raw[2], raw[3]]) as i64;
    let section = word(&raw, 12);
    if sections == 0 || section + 4 + count * 8 > blob.len() as i64 {
        return Err(Error::new(
            Kind::RomError,
            "a text file whose lines run past its end",
        ));
    }
    let mut lines: Vec<Vec<u8>> = Vec::new();
    let mut key = FIRST_KEY;
    for index in 0..count {
        let raw = binary::take::<8>(blob, section + 4 + index * 8)?;
        let offset = word(&raw, 0);
        let length = u16::from_le_bytes([raw[4], raw[5]]) as i64;
        let at = section + offset;
        if at + length * 2 > blob.len() as i64 {
            return Err(Error::new(
                Kind::RomError,
                format!("line {index} runs past the end of its file"),
            ));
        }
        let mut rolling = key;
        let mut clear: Vec<i64> = Vec::new();
        for unit in 0..length {
            let unit = u16(blob, at + unit * 2)?;
            clear.push(unit ^ rolling);
            rolling = (rolling << 3 | rolling >> 13) & 0xFFFF;
        }
        let kept = kept(&clear);
        lines.push(
            clear[..kept]
                .iter()
                .flat_map(|&unit| (unit as u16).to_le_bytes())
                .collect(),
        );
        key = (key + NEXT_KEY) & 0xFFFF;
    }
    Ok(lines)
}

/// How many of a line's halfwords are its own: all but its trailing zeros, short of the
/// arguments of a control that ends it, as a name insert's 0 that ends "Haus von <name>".
fn kept(units: &[i64]) -> usize {
    let mut end = units.len();
    while end > 0 && units[end - 1] == 0 {
        end -= 1;
    }
    let mut at = 0;
    while at + 1 < end {
        if units[at] == MARKER {
            at += 2 + units[at + 1] as usize;
            end = end.max(at.min(units.len()));
        } else {
            at += 1;
        }
    }
    end
}

/// A line's words without its control sequences, or "" when it is only controls or broken.
pub fn plain(line: &[u8]) -> String {
    match tokens(line) {
        Ok(tokens) => {
            let words: String = tokens
                .iter()
                .filter_map(|token| match token {
                    Token::Text(text) => Some(text.as_str()),
                    Token::Control(_) => None,
                })
                .collect();
            py::strip(&words).to_string()
        }
        Err(error) if error.is(Kind::ValueError) => String::new(),
        Err(error) => panic!("{error:?}"),
    }
}

/// A line's words as `plain` gives them, in the singular. X's English names an interior item
/// with its plural's ending after a number-form control, whose second argument counts the
/// singular's letters in its low byte and the plural's after them in its high byte: "Leaf
/// Cushion", the control, then "s" (spike 228, x_pokemon-me).
pub fn singular(line: &[u8]) -> String {
    let parts = match tokens(line) {
        Ok(parts) => parts,
        Err(error) if error.is(Kind::ValueError) => return String::new(),
        Err(error) => panic!("{error:?}"),
    };
    let mut words: Vec<String> = Vec::new();
    let mut form: Option<(i64, i64)> = None;
    for part in parts {
        match part {
            Token::Control(control) => {
                if control.code == NUMBER_FORM && control.args.len() >= 2 {
                    form = Some((control.args[1] & 0xFF, control.args[1] >> 8));
                }
            }
            Token::Text(text) => {
                let mut part = text;
                if let Some((kept, dropped)) = form.take() {
                    let chars: Vec<char> = part.chars().collect();
                    let head: String = chars.iter().take(kept as usize).collect();
                    let tail: String = chars.iter().skip((kept + dropped) as usize).collect();
                    part = head + &tail;
                }
                words.push(part);
            }
        }
    }
    py::strip(&words.concat()).to_string()
}

/// A little-endian u32 out of bytes a `struct.unpack_from` already took, as an int.
fn word(raw: &[u8], at: usize) -> i64 {
    u32::from_le_bytes(raw[at..at + 4].try_into().expect("4 bytes")) as i64
}
