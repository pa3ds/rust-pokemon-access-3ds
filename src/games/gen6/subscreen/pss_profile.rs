//! A PSS profile's values: another player's message and their two locations, and the player's
//! own, which the profile draws into a picture a row, `prof_text.bclim`, and writes in no TextBox:
//! its rows' boxes hold placeholders, and another player's "Nice to meet ya!", "United States"
//! and "District of Columbia" went unsaid beside "Message", "Location 1" and "Location 2"
//! (spike 319).
//!
//! The rows are the profile app's objects, one a row, each holding its place down the list, and a
//! row of words its words:
//!
//! ```text
//! PssProfile*Category        +0x06 s16 its place down the list, 50 a row: 45, -5, -55, -105
//! PssProfileMessageCategory  +0x20 -> StrBuf, its words (+0x24 a second of the same)
//! ```
//!
//! The list's rows are drawn by five frames, the layout's root's own children, each holding a
//! picture of the texture: in the layout's order they draw the rows down, Message, History,
//! Location 1, Location 2 and a total, so a picture is named by its frame's order, the n-th
//! frame's by the n-th row down, of any kind. Each frame sits at its row's place less the list's
//! scroll, which the rows' places never take up: scrolled, the frames moved 25, 46 and then 6
//! units up together and kept their order, while the places stayed (another player's profile in
//! Y, spike 319). Sorting the pictures by height instead put the frames the texture draws in
//! elsewhere, hidden at 0, among the list's, and named only Message. History's row is a
//! `PssProfileHistoryCategory` of icons, each a picture of its own texture that screen text names
//! as the game's text does (`HISTORY`); two totals and a time follow Location 2.

use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{f32, s16, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::FileOf;
use crate::games::common::{lyt, strbuf};
use crate::py::{self, Result, sorted_by_lt};

pub const MESSAGE: &str = "N3app3pss11pss_profile25PssProfileMessageCategoryE";
pub const HISTORY_ROW: &str = "N3app3pss11pss_profile25PssProfileHistoryCategoryE";
/// The profile's rows, of words, of History's icons, of a total and of a survey's answer.
pub const ROWS: [&str; 4] = [
    MESSAGE,
    HISTORY_ROW,
    "N3app3pss11pss_profile28PssProfileTotalCountCategoryE",
    "N3app3pss11pss_profile25PssProfileEnqueteCategoryE",
];
pub const PLACE: i64 = 0x06;
pub const WORDS: i64 = 0x20;
pub const FILE: &str = "profile_lower.bclyt";
pub const TEXTURE: &str = "prof_text.bclim";
/// History's icons, by the texture each draws, and the line of the PSS's text naming the app
/// among its apps' names: Battle, Trade, Battle Spot, GTS, O-Power and Wonder Trade. The row shows
/// up to six, each a picture of its kind's own (`PssProfileHistoryCategory`, spike 318's
/// decompile): six kinds of six pictures, all hidden and then the record's shown in its order, a
/// slot 28 across. The same lines in Omega Ruby's file 70 and X's 57.
pub const HISTORY: [(&str, i64); 6] = [
    ("prof_battle", 283),
    ("prof_trade", 284),
    ("prof_btlmatch", 285),
    ("prof_gts", 289),
    ("prof_asist", 290),
    ("prof_random", 293),
];

/// The profile's rows of words, named in its pictures.
pub struct ProfileValues {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    layouts: Rc<Layouts>,
    file_of: FileOf,
    /// A texture, less `.bclim` -> the game's word for it: History's (`HISTORY`).
    words: IndexMap<String, String>,
    gap: String,
    rows: Vec<i64>,
}

impl ProfileValues {
    /// What the heap is swept for: the rows.
    pub const CLASSES: [&'static str; 4] = ROWS;

    /// `ProfileValues(memory, names, layouts, file_of)`: no words, and a gap of " ".
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        file_of: FileOf,
    ) -> Self {
        ProfileValues::with_words(
            memory,
            names,
            layouts,
            file_of,
            IndexMap::new(),
            " ".to_string(),
        )
    }

    pub fn with_words(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        file_of: FileOf,
        words: IndexMap<String, String>,
        gap: String,
    ) -> Self {
        ProfileValues {
            memory,
            names,
            layouts,
            file_of,
            words,
            gap,
            rows: Vec::new(),
        }
    }

    /// Keep the rows a sweep found, in place of those kept, or with `adding` after them: a
    /// sweep of the blocks just allocated can find three of a profile's thirteen, and a profile
    /// whose kept rows were those three said Location 2's words as Location 1's and History's
    /// icons after Message. A closed profile's rows may still read as rows where they lay, and
    /// a later one's at the same places take theirs (`_placed`).
    ///
    /// Python's `adding` defaults to False.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let rows: Vec<i64> = found
            .iter()
            .filter(|(_, name)| ROWS.contains(&name.as_str()))
            .map(|(&at, _)| at)
            .collect();
        if !rows.is_empty() {
            self.rows = if adding {
                // `list(dict.fromkeys(...))`: each once, where it first came.
                let kept: IndexSet<i64> = self.rows.iter().chain(&rows).copied().collect();
                kept.into_iter().collect()
            } else {
                rows
            };
        }
        Ok(())
    }

    /// The words a profile's row draws in this picture, or None for any other picture: History
    /// whole for its first icon, and "" for its others.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        let layouts = &self.layouts;
        let placement = layouts.placement(picture)?;
        let Some(placement) = placement.filter(|placement| placement.shown) else {
            return Ok(None);
        };
        let root = placement.root;
        let file = (self.file_of)(root);
        let Some(file) = file.filter(|file| file.name == FILE) else {
            return Ok(None);
        };
        if self.history(&file, picture)?.is_some() {
            return Ok(Some(self.history_said(&file, root, picture)?));
        }
        if !self.drawn(&file, picture)? {
            return Ok(None);
        }
        let frames = self.frames(&file, root)?;
        let frame = u32(&layouts.pane(picture)?, lyt::PARENT)?;
        let rows = self.placed()?;
        let Some(index) = frames.iter().position(|&each| each == frame) else {
            return Ok(None);
        };
        if index >= rows.len() {
            return Ok(None);
        }
        let row = rows[index];
        if self.names.of_object(row)?.as_deref() != Some(MESSAGE) {
            return Ok(None);
        }
        strbuf::label(
            self.memory.as_ref(),
            u32(&self.memory.read(row + WORDS, 4)?, 0)?,
        )
    }

    /// The words on the screen with History's icons after its label: they hang from the
    /// layout's root, its rows' words from a frame a row, and were said after both Locations
    /// (or_pss_singleplayer).
    pub fn reworded(&self, found: IndexMap<i64, ScreenText>) -> Result<IndexMap<i64, ScreenText>> {
        let mut icons: Option<(i64, ScreenText)> = None;
        for (&r#box, piece) in &found {
            if !(piece.picture && !piece.text.is_empty()) {
                continue;
            }
            let Some(file) = (self.file_of)(piece.root).filter(|file| file.name == FILE) else {
                continue;
            };
            if self.history(&file, r#box)?.is_some() {
                icons = Some((r#box, piece.clone()));
                break;
            }
        }
        let Some((r#box, said)) = icons else {
            return Ok(found);
        };
        let file = (self.file_of)(said.root);
        let mut kinds: Vec<Option<String>> = Vec::new();
        for row in self.placed()? {
            kinds.push(self.names.of_object(row)?);
        }
        let frames = match &file {
            Some(file) => self.frames(file, said.root)?,
            None => Vec::new(),
        };
        let Some(index) = kinds
            .iter()
            .position(|kind| kind.as_deref() == Some(HISTORY_ROW))
        else {
            return Ok(found);
        };
        if index >= frames.len() {
            return Ok(found);
        }
        let label = self
            .layouts
            .children(said.root, frames[index])
            .into_iter()
            .find(|pane| found.get(pane).is_some_and(|piece| !piece.picture));
        let Some(label) = label else {
            return Ok(found);
        };
        let text = format!("{} {}", found[&label].text, said.text);
        let mut worded: IndexMap<i64, ScreenText> =
            found.into_iter().filter(|&(at, _)| at != r#box).collect();
        let piece = worded[&label].clone();
        worded.insert(label, ScreenText { text, ..piece });
        Ok(worded)
    }

    /// History's icons shown, counted, "Battle 3 times, Trade once", each app in the order
    /// it first stands, left to right, which is their record's order, for the first; "" for the
    /// others. Named one at a time, two alike side by side were taken for one thing drawn twice
    /// and the rest joined as a type's icons are, "Battle/O-Power/Trade O-Power/Trade"
    /// (or_pss_singleplayer); said whole in a row, "Battle, Trade, Battle, Trade, Battle,
    /// Battle" sounded like pa3ds repeating button names, where a sighted player sees a row of
    /// icons (Alpha Sapphire, live).
    fn history_said(&self, file: &LayoutFile, root: i64, picture: i64) -> Result<String> {
        let layouts = &self.layouts;
        let mut icons: Vec<(f64, i64, String)> = Vec::new();
        for (pane, made) in layouts.tree(root) {
            if made != lyt::PICTURE_CLASS {
                continue;
            }
            let Some(kind) = self.history(file, pane)? else {
                continue;
            };
            if !layouts.placement(pane)?.is_some_and(|shown| shown.shown) {
                continue;
            }
            icons.push((f32(&layouts.pane(pane)?, lyt::X)?, pane, kind));
        }
        // A tuple's order: the first parts that differ.
        let icons = sorted_by_lt(icons, |a, b| {
            if a.0 != b.0 {
                return a.0 < b.0;
            }
            if a.1 != b.1 {
                return a.1 < b.1;
            }
            a.2 < b.2
        });
        if icons.first().is_none_or(|first| first.1 != picture) {
            return Ok(String::new());
        }
        // `Counter(...)`: each word counted, in the order it first came.
        let mut counted: IndexMap<&str, i64> = IndexMap::new();
        for (_x, _pane, kind) in &icons {
            if let Some(word) = self.words.get(kind).filter(|word| !word.is_empty()) {
                *counted.entry(word.as_str()).or_insert(0) += 1;
            }
        }
        Ok(counted
            .iter()
            .map(|(word, count)| own("history-times", &[("what", word), ("count", count)]))
            .collect::<Vec<_>>()
            .join(&self.gap))
    }

    /// The list's frames, a row each in the rows' order: the root's children holding a
    /// picture of a row's words.
    fn frames(&self, file: &LayoutFile, root: i64) -> Result<Vec<i64>> {
        let layouts = &self.layouts;
        let mut frames: Vec<i64> = Vec::new();
        for frame in layouts.children(root, root) {
            for pane in layouts.children(root, frame) {
                if self.drawn(file, pane)? {
                    frames.push(frame);
                    break;
                }
            }
        }
        Ok(frames)
    }

    /// The rows kept, top first by their places down the list, the one kept last at each.
    fn placed(&self) -> Result<Vec<i64>> {
        let mut placed: IndexMap<i64, i64> = IndexMap::new();
        for &row in &self.rows {
            if self
                .names
                .of_object(row)?
                .is_some_and(|name| ROWS.contains(&name.as_str()))
            {
                placed.insert(s16(&self.memory.read(row, PLACE + 2)?, PLACE)?, row);
            }
        }
        // `sorted(placed, reverse=True)`: reversed, sorted, and reversed again.
        let mut places: Vec<i64> = placed.keys().rev().copied().collect();
        py::sort_by_lt(&mut places, |a, b| a < b);
        places.reverse();
        Ok(places.iter().map(|place| placed[place]).collect())
    }

    /// The texture of History's icon a pane draws, less `.bclim`, or None for another pane.
    fn history(&self, file: &LayoutFile, pane: i64) -> Result<Option<String>> {
        let name = self.layouts.pane_name(pane)?;
        Ok(file.textures.get(&name).and_then(|textures| {
            textures.iter().find_map(|texture| {
                let name = texture.strip_suffix(".bclim").unwrap_or(texture);
                HISTORY
                    .iter()
                    .any(|&(kind, _line)| kind == name)
                    .then(|| name.to_string())
            })
        }))
    }

    /// Whether a pane is a picture of a row's words, by the texture its file gives it.
    fn drawn(&self, file: &LayoutFile, pane: i64) -> Result<bool> {
        Ok(file
            .textures
            .get(&self.layouts.pane_name(pane)?)
            .is_some_and(|textures| textures.iter().any(|texture| texture == TEXTURE)))
    }
}
