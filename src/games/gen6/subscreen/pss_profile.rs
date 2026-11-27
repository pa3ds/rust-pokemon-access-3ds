//! A PSS profile's values: another player's message and their two locations, and the player's
//! own, which the profile draws into a picture a row, `prof_text.bclim`, and writes in no TextBox:
//! its rows' boxes hold placeholders, and another player's "Nice to meet ya!", "United States"
//! and "District of Columbia" went unsaid beside "Message", "Location 1" and "Location 2"
//! (spike A).
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
//! Y, spike A). Sorting the pictures by height instead put the frames the texture draws in
//! elsewhere, hidden at 0, among the list's, and named only Message. History's row is a
//! `PssProfileHistoryCategory` of icons, unread; two totals and a time follow Location 2.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{s16, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::touch::FileOf;
use crate::games::common::{lyt, strbuf};
use crate::py::{Error, Result, sorted_by_lt};

pub const MESSAGE: &str = "N3app3pss11pss_profile25PssProfileMessageCategoryE";
/// The profile's rows, of words, of History's icons, of a total and of a survey's answer.
pub const ROWS: [&str; 4] = [
    MESSAGE,
    "N3app3pss11pss_profile25PssProfileHistoryCategoryE",
    "N3app3pss11pss_profile28PssProfileTotalCountCategoryE",
    "N3app3pss11pss_profile25PssProfileEnqueteCategoryE",
];
pub const PLACE: i64 = 0x06;
pub const WORDS: i64 = 0x20;
pub const FILE: &str = "profile_lower.bclyt";
pub const TEXTURE: &str = "prof_text.bclim";

/// The profile's rows of words, named in its pictures.
pub struct ProfileValues {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    layouts: Rc<Layouts>,
    file_of: FileOf,
    rows: Vec<i64>,
}

impl ProfileValues {
    /// What the heap is swept for: the rows.
    pub const CLASSES: [&'static str; 4] = ROWS;

    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        file_of: FileOf,
    ) -> Self {
        ProfileValues {
            memory,
            names,
            layouts,
            file_of,
            rows: Vec::new(),
        }
    }

    /// Keep the rows a sweep found, in place of those kept: a profile opening allocates its
    /// rows at once, and a closed one's may still read as rows where it lay.
    ///
    /// Python's `adding` defaults to False.
    pub fn track(&mut self, found: &IndexMap<i64, String>, _adding: bool) -> Result<()> {
        let rows: Vec<i64> = found
            .iter()
            .filter(|(_, name)| ROWS.contains(&name.as_str()))
            .map(|(&at, _)| at)
            .collect();
        if !rows.is_empty() {
            self.rows = rows;
        }
        Ok(())
    }

    /// The words a profile's row draws in this picture, or None for any other picture.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        let layouts = &self.layouts;
        let placement = layouts.placement(picture)?;
        let Some(placement) = placement.filter(|placement| placement.shown) else {
            return Ok(None);
        };
        let root = placement.root;
        let file = (self.file_of)(root);
        let Some(file) = file else {
            return Ok(None);
        };
        if file.name != FILE || !self.drawn(&file, picture)? {
            return Ok(None);
        }
        let mut frames: Vec<i64> = Vec::new();
        for frame in layouts.children(root, root) {
            for pane in layouts.children(root, frame) {
                if self.drawn(&file, pane)? {
                    frames.push(frame);
                    break;
                }
            }
        }
        let frame = u32(&layouts.pane(picture)?, lyt::PARENT)?;
        if !frames.contains(&frame) {
            return Ok(None);
        }
        let mut placed: Vec<(i64, i64)> = Vec::new();
        for &row in &self.rows {
            if self
                .names
                .of_object(row)?
                .is_some_and(|name| ROWS.contains(&name.as_str()))
            {
                placed.push((s16(&self.memory.read(row, PLACE + 2)?, PLACE)?, row));
            }
        }
        let rows = sorted_by_lt(placed, |a, b| -a.0 < -b.0);
        let index = frames
            .iter()
            .position(|&each| each == frame)
            .ok_or_else(|| Error::value(format!("{frame} is not in list")))?;
        if index >= rows.len() {
            return Ok(None);
        }
        let row = rows[index].1;
        if self.names.of_object(row)?.as_deref() != Some(MESSAGE) {
            return Ok(None);
        }
        strbuf::label(
            self.memory.as_ref(),
            u32(&self.memory.read(row + WORDS, 4)?, 0)?,
        )
    }

    /// Whether a pane is a picture of a row's words, by the texture its file gives it.
    fn drawn(&self, file: &LayoutFile, pane: i64) -> Result<bool> {
        Ok(file
            .textures
            .get(&self.layouts.pane_name(pane)?)
            .is_some_and(|textures| textures.iter().any(|texture| texture == TEXTURE)))
    }
}
