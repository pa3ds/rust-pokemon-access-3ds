//! Sun's own words out of the player's ROM: its system text, a file a screen.
//!
//! The text files are ORAS's format in version 0x0600 archives, one archive a language: `a/0/3/0` is
//! the Japanese in kana and `a/0/3/1` in kanji, then English, French, Italian, German, Spanish,
//! Korean, and Chinese in simplified and traditional characters, each told by the word the Bag's
//! empty pocket says (spike 111). Only the files a reader needs are kept, each by Sun's number: the
//! Ultra games keep most of them a few files on (`sm.VersionLayout`).

use indexmap::IndexMap;

use crate::core::rom::Rom;
use crate::games::common::garc::{garc_files, message_lines, plain};
use crate::games::sm::layout_of;
use crate::py::{self, Error, Kind, Result};

/// A dict in the order Python wrote it, looked up by its first of each pair.
pub const TEXT_ARCHIVES: [(&str, &str); 9] = [
    ("ja", "a/0/3/0"),
    ("en", "a/0/3/2"),
    ("fr", "a/0/3/3"),
    ("it", "a/0/3/4"),
    ("de", "a/0/3/5"),
    ("es", "a/0/3/6"),
    ("ko", "a/0/3/7"),
    ("zh-CN", "a/0/3/8"),
    ("zh-TW", "a/0/3/9"),
];

/// Some of Sun's system text files, each a list of lines without their control sequences.
#[derive(Debug, Clone, PartialEq)]
pub struct SunText {
    pub files: IndexMap<i64, Vec<String>>,
}

impl SunText {
    /// These system text files in `language`, numbered as Sun numbers them whichever version
    /// `rom` is. A RomError for a language Sun has no text in, or a file that is not one.
    pub fn load(rom: &Rom, language: &str, files: &[i64]) -> Result<SunText> {
        let archive = TEXT_ARCHIVES
            .iter()
            .find(|(known, _)| *known == language)
            .map(|&(_, archive)| archive);
        let Some(archive) = archive else {
            return Err(Error::new(
                Kind::RomError,
                format!("Sun has no text in {}", py::repr(language)),
            ));
        };
        let texts = garc_files(&rom.read(archive)?)?;
        let layout = layout_of(rom.title_id);
        let mut loaded: IndexMap<i64, Vec<String>> = IndexMap::new();
        for &number in files {
            // `texts[layout.system(number)]`, which counts a negative number from the end.
            let at = layout.system(number);
            let at = if at < 0 { at + texts.len() as i64 } else { at };
            let Some(text) = usize::try_from(at).ok().and_then(|at| texts.get(at)) else {
                let last = files.iter().max().expect("a file was asked for");
                return Err(Error::new(
                    Kind::RomError,
                    format!("{archive} has no file {last}"),
                ));
            };
            let lines = message_lines(text)?
                .iter()
                .map(|line| plain(line))
                .collect();
            loaded.insert(number, lines);
        }
        Ok(SunText { files: loaded })
    }

    /// One line of a file loaded, or None for one that is not there or says nothing.
    pub fn line(&self, file: i64, index: i64) -> Option<String> {
        let lines: &[String] = self.files.get(&file).map_or(&[], Vec::as_slice);
        if 0 <= index && index < lines.len() as i64 {
            let line = &lines[index as usize];
            if line.is_empty() {
                None
            } else {
                Some(line.clone())
            }
        } else {
            None
        }
    }
}
