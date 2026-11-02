//! A Pokémon's cry out of the player's ROM, to play where pa3ds would say its name (spike A).
//!
//! The cries are a GARC of their own (`RomLayout.cries`): every sequence, a CSEQ, then every wave
//! archive, a CWAR of one wave. The sound archive has only slots for them, filled as the game runs,
//! so which is whose is a table in the executable, the one read as pa3ds starts, one record a species
//! from 0 to 721 and the forms after them:
//!
//! ```text
//! +0x00 u16 the record of the species' first other form, 0 where it has none   +0x02 u16 0
//! +0x04 four (u16 sequence, u16 wave file), both files of the GARC
//! ```
//!
//! The first pair is the whole cry; the others are the ones the game plays elsewhere, the whole cry
//! again and two short ones of a few notes. The waves are the files after the sequences, so a wave
//! file's number starts at the count of sequences, N: the table is found as Bulbasaur's record after
//! species 0's 20 zero bytes, `0, 0, (0, N), (0, N)`, and read on while its records hold, 913 in
//! Omega Ruby and 886 in X. A cry's sequence plays its wave once at the bank's original key, at a
//! main volume of its own (`c2`), so the wave is the cry and that volume is how loud the game plays
//! it.
//!
//! Read only once a game wants a cry, and then only what it needs: the GARC's index, and a sequence
//! and a wave a Pokémon.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::core::bcsar::{self, Sound};
use crate::core::binary::{take, u16, u32};
use crate::core::rom::Rom;
use crate::games::common::garc::garc_spans;
use crate::games::common::pokemon::Pokemon;
use crate::py::{self, Error, Kind, Result};

pub const RECORD: i64 = 20;
/// Records 0, no one, to 721, Volcanion; the forms follow.
pub const SPECIES: i64 = 722;
pub const PAIRS: i64 = 4;
pub const SEQUENCE: &[u8] = b"CSEQ";
/// The sequence command setting its main volume, out of 127.
pub const MAIN_VOLUME: i64 = 0xC2;
/// A CSEQ's block of commands.
pub const DATA: i64 = 0x5000;
pub const LOUDEST: i64 = 127;

/// The table: each record's first other form, and its pairs as (sequence, wave, ...).
pub type Records = Vec<(i64, Vec<i64>)>;
/// Plays a Pokémon's cry toward a side, 0 the left and 1 the right, and says whether it did
/// (`session.cry_player`).
///
/// A `Result`, where Python's `Callable` returns a bool: what it raises but the ROM's failing,
/// which it logs, goes on to the reader that asked, as Python's exception does.
pub type Crier = Rc<dyn Fn(&Pokemon, f64) -> Result<bool>>;

/// The cry table's records, found in `code` by Bulbasaur's record, species 0's first; None
/// where it is not there or does not reach the last species.
pub fn table(code: &[u8], sequences: i64, files: i64) -> Option<Records> {
    // `bytes(RECORD) + struct.pack("<6H", 0, 0, 0, sequences, 0, sequences)`.
    let mut needle = vec![0u8; RECORD as usize];
    for value in [0, 0, 0, sequences, 0, sequences] {
        needle.extend((value as u16).to_le_bytes());
    }
    // `code.find(needle)`.
    let at = code
        .windows(needle.len())
        .position(|window| window == needle)? as i64;
    let mut records: Records = vec![(0, vec![0; (PAIRS * 2) as usize])];
    while at + (records.len() as i64 + 1) * RECORD <= code.len() as i64 {
        let start = (at + records.len() as i64 * RECORD) as usize;
        let fields: Vec<i64> = code[start..start + RECORD as usize]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| i64::from(u16::from_le_bytes(*pair)))
            .collect();
        let (form, pad, pairs) = (fields[0], fields[1], fields[2..].to_vec());
        // Two forms' records after the species, 722 and 723, have a pair with no wave, 0.
        if pad != 0
            || !pairs
                .iter()
                .skip(1)
                .step_by(2)
                .all(|&w| w == 0 || (sequences <= w && w < files))
            || pairs.iter().step_by(2).any(|&s| s >= sequences)
        {
            break;
        }
        records.push((form, pairs));
    }
    if records.len() as i64 >= SPECIES {
        Some(records)
    } else {
        None
    }
}

/// A species' pairs in a form: its form's record, `first + form - 1`, where the species has
/// other forms and the table that one; its own otherwise. None for no species.
pub fn pairs(records: &[(i64, Vec<i64>)], species: i64, form: i64) -> Option<Vec<i64>> {
    if !(0 < species && species < SPECIES.min(records.len() as i64)) {
        return None;
    }
    let (first, own) = &records[species as usize];
    if form != 0 && *first != 0 && first + form - 1 < records.len() as i64 {
        return Some(records[(first + form - 1) as usize].1.clone());
    }
    Some(own.clone())
}

/// The game's cries, by species and form, read as they are first asked for.
pub struct Cries {
    rom: PathBuf,
    archive: String,
    code: Vec<u8>,
    records: Option<Records>,
    start: i64,
    spans: Vec<(i64, i64)>,
}

impl Cries {
    pub fn new(rom: &Path, archive: &str, code: &[u8]) -> Self {
        Cries {
            rom: rom.to_path_buf(),
            archive: archive.to_string(),
            code: code.to_vec(),
            records: None,
            start: 0,
            spans: Vec::new(),
        }
    }

    /// The whole cry of a species in a form, as loud as the game plays it; the species' own
    /// where its form has none of its own, and None where the game has none. A RomError where
    /// the ROM cannot be read.
    pub fn cry(&mut self, species: i64, form: i64) -> Result<Option<Sound>> {
        // `with Rom(...) as image:`, closed as `image` goes.
        let (cseq, cwar) = {
            let image = Rom::new(&self.rom)?;
            if self.records.is_none() {
                (self.start, self.spans) = garc_spans(&image, &self.archive)?;
                let sequences = first_wave(&image, &self.archive, self.start, &self.spans)?;
                self.records =
                    Some(table(&self.code, sequences, self.spans.len() as i64).unwrap_or_default());
            }
            let found = pairs(self.records.as_deref().unwrap_or_default(), species, form);
            let Some(found) = found.filter(|found| found[1] != 0) else {
                return Ok(None);
            };
            (self.file(&image, found[0])?, self.file(&image, found[1])?)
        };
        let (rate, samples) = wave(&cwar)?;
        Ok(Some(Sound {
            rate,
            samples,
            volume: main_volume(&cseq)? as f64 / LOUDEST as f64,
            quietest: 1.0,
        }))
    }

    fn file(&self, image: &Rom, number: i64) -> Result<Vec<u8>> {
        let &(start, end) = usize::try_from(number)
            .ok()
            .and_then(|number| self.spans.get(number))
            .ok_or_else(|| Error::index("list index out of range"))?;
        image.read_part(&self.archive, self.start + start, end - start)
    }
}

/// How many sequences come before the waves: the first file not a CSEQ, found by halves.
fn first_wave(image: &Rom, archive: &str, start: i64, spans: &[(i64, i64)]) -> Result<i64> {
    let (mut low, mut high) = (0, spans.len() as i64);
    while low < high {
        let middle = py::floordiv(low + high, 2);
        if image.read_part(archive, start + spans[middle as usize].0, 4)? == SEQUENCE {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    if low == 0 {
        return Err(Error::new(
            Kind::RomError,
            format!("{archive} holds no cries"),
        ));
    }
    Ok(low)
}

/// A CWAR's one wave: its INFO block (`+0x18`) lists it, at its FILE block (`+0x24`).
fn wave(cwar: &[u8]) -> Result<(i64, Vec<i16>)> {
    let entry = u32(cwar, 0x18)? + 8 + 4;
    let start = u32(cwar, 0x24)? + 8 + u32(cwar, entry + 4)?;
    bcsar::decode(py::slice(cwar, start, start + u32(cwar, entry + 8)?))
}

/// The main volume a cry's sequence opens with, or the loudest where it sets none.
fn main_volume(cseq: &[u8]) -> Result<i64> {
    for k in 0..u16(cseq, 0x10)? {
        // `struct.unpack_from("<HHII", ...)`: one read of the twelve bytes.
        let entry = take::<12>(cseq, 0x14 + k * 12)?;
        let kind = i64::from(u16::from_le_bytes([entry[0], entry[1]]));
        let at = i64::from(u32::from_le_bytes([entry[4], entry[5], entry[6], entry[7]]));
        if kind == DATA && byte(cseq, at + 8)? == MAIN_VOLUME {
            return byte(cseq, at + 9);
        }
    }
    Ok(LOUDEST)
}

/// `cseq[at]`: an `IndexError` past its end.
fn byte(data: &[u8], at: i64) -> Result<i64> {
    usize::try_from(at)
        .ok()
        .and_then(|at| data.get(at))
        .map(|&b| b as i64)
        .ok_or_else(|| Error::index("index out of range"))
}
