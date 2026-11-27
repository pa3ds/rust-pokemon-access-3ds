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
//! The four rows shown each draw their picture in a pane of their own, Message's, History's,
//! Location 1's and Location 2's, from the top: their places in the layout are not the list's (76,
//! -5, -124 and -224 against 45, -5, -55 and -105), so a picture is named by its order, the n-th
//! row's picture down by the n-th row down, of any kind. History's row is a
//! `PssProfileHistoryCategory` of icons, unread; two totals and a time follow Location 2. Checked
//! with the list unscrolled.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, s16, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::touch::FileOf;
use crate::games::common::{lyt, strbuf};
use crate::py::{Error, PySet, Result, sorted_by_lt};

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
        let mut drawn = PySet::new();
        for p in layouts.tree(root).into_keys() {
            if self.drawn(&file, p)? {
                drawn.add(u32(&layouts.pane(p)?, lyt::PARENT)?);
            }
        }
        // `sorted(..., key=...)`: each key taken once, in the set's order.
        let mut keyed: Vec<(f64, i64)> = Vec::new();
        for frame in drawn.iter() {
            keyed.push((-self.height(frame, root)?, frame));
        }
        let frames: Vec<i64> = sorted_by_lt(keyed, |a, b| a.0 < b.0)
            .into_iter()
            .map(|(_key, frame)| frame)
            .collect();
        let frame = u32(&layouts.pane(picture)?, lyt::PARENT)?;
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

    /// A pane's y in its layout, its parents' added up to the root.
    fn height(&self, mut pane: i64, root: i64) -> Result<f64> {
        let mut y = 0.0;
        while pane != 0 && pane != root {
            let head = self.layouts.pane(pane)?;
            y += f32(&head, self.layouts.shape.y)?;
            pane = u32(&head, lyt::PARENT)?;
        }
        Ok(y)
    }
}
