//! The Switch screen of the Pokémon-Amie page, where the game shows a Pokémon's three numbers.
//!
//! SWITCH on the page's menu opens the party as six round buttons, and for the Pokémon chosen a
//! picture of it, its name, and three rows of five pictures: Affection's hearts, Fullness's puffs and
//! Enjoyment's notes, each `heart_00.bclim`, `ic_hungry.bclim` or `ic_asobu.bclim` where filled and
//! `heart_01.bclim` or the `_black` one where empty (or_amie_interactions). Nowhere else does the game
//! show them. Screen text said the rows' labels and nothing of the pictures: "Switch Pokémon!
//! Metagross Affection Fullness Enjoyment". `SwitchScreen` gives each label its row, "Affection 1 of
//! 5", so that a row is heard after the Pokémon's name, as the screen opens and another is chosen.
//!
//! The party's six buttons draw each member's icon with nothing to say whose, and touch mode said
//! "button pokémon". They stand in the party's order across the screen, so each is named by the
//! party member its place holds.
//!
//! Bound by:
//! - ADR-0037: pictures counted and said in words of pa3ds's own, by the game's own label.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::pokemon;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::ImageOf;
use crate::games::gen6::screens::partyicons::COUNT;
use crate::games::gen6::subscreen::amie::{METER, PAGE_APPS};
use crate::games::gen6::subscreen::berry_picker::Name;
use crate::games::gen6::subscreen::super_training::pictures;
use crate::py::{self, Result};

/// `super_training.pictures`, the pictures of a layout's whose images a test wants: kept by the
/// screen, which a test replaces where Python's monkeypatches the module's.
pub type PicturesOf = fn(
    &Layouts,
    &ImageOf,
    bool,
    &dyn Fn(&str) -> bool,
) -> Result<Vec<(i64, i64, String, f64, f64)>>;

/// What a row's pictures start with, in the order of the labels.
pub const ROWS: [&str; 3] = ["heart_", "ic_hungry", "ic_asobu"];
/// A row's picture where it is not filled.
pub const EMPTY: [&str; 3] = [
    "heart_01.bclim",
    "ic_hungry_black.bclim",
    "ic_asobu_black.bclim",
];
/// How far up from the screen's middle the rows reach: the rows stand at 30, 5 and -20, and the
/// heart marking the Pokémon chosen among the party's buttons at 76.
pub const ROWS_BELOW: f64 = 50.0;
pub const FIRST_BUTTON: f64 = -125.0;
/// Where the party's first button stands across the screen, from its middle, and how far apart
/// they are.
pub const BUTTONS_APART: f64 = 50.0;

/// Gives the Switch screen's row labels their rows, and names its party's buttons.
pub struct SwitchScreen {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    layouts: Rc<Layouts>,
    image_of: ImageOf,
    /// What the game calls the rows, none where its text is not read.
    labels: Vec<String>,
    /// The party's list, as last found.
    party: Rc<dyn Fn() -> Option<i64>>,
    /// A party member as the player knows it.
    called: Name,
    /// Whether the screen's rows were on the screen at the last poll.
    up: bool,
    /// `pictures`, as a test replaces it.
    pub(crate) pictures: PicturesOf,
}

impl SwitchScreen {
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        labels: Vec<String>,
        party: Rc<dyn Fn() -> Option<i64>>,
        called: Name,
    ) -> Self {
        SwitchScreen {
            memory,
            names,
            layouts,
            image_of,
            labels,
            party,
            called,
            up: false,
            pictures,
        }
    }

    /// How many pictures of each row are filled.
    fn rows(&self) -> Result<Vec<i64>> {
        let mut filled = vec![0; ROWS.len()];
        let wanted = |image: &str| ROWS.iter().any(|row| image.starts_with(row));
        for (_pane, _root, image, _across, up) in
            (self.pictures)(&self.layouts, &self.image_of, false, &wanted)?
        {
            if up < ROWS_BELOW && !EMPTY.contains(&image.as_str()) {
                let row = ROWS
                    .iter()
                    .position(|row| image.starts_with(row))
                    .expect("a picture of a row");
                filled[row] += 1;
            }
        }
        Ok(filled)
    }

    /// The words on the screen, each row's label with how many of its pictures are filled.
    pub fn reworded(
        &mut self,
        found: IndexMap<i64, ScreenText>,
    ) -> Result<IndexMap<i64, ScreenText>> {
        self.up = found
            .values()
            .filter(|piece| self.labels.contains(&piece.text))
            .count()
            == ROWS.len();
        if !self.up {
            return Ok(found);
        }
        let rows = self.rows()?;
        let heights: Vec<f64> = found
            .values()
            .filter(|piece| self.labels.contains(&piece.text))
            .map(|piece| piece.row)
            .collect();
        // Python's `min` and `max` of floats: the first unless another is strictly beyond it.
        let low = heights[1..]
            .iter()
            .fold(heights[0], |low, &row| if row < low { row } else { low });
        let high = heights[1..]
            .iter()
            .fold(heights[0], |high, &row| if row > high { row } else { high });
        let mut worded = found.clone();
        for (r#box, piece) in found {
            if let Some(row) = self.labels.iter().position(|label| *label == piece.text) {
                let text = own(METER, &[("label", &piece.text), ("count", &rows[row])]);
                worded.insert(r#box, ScreenText { text, ..piece });
            } else if low <= piece.row && piece.row <= high {
                // The Pokémon's name, beside the rows and lower than two of them, read first:
                // "Affection 1 of 5 Fullness 0 of 5 Metagross Enjoyment 4 of 5".
                worded.insert(
                    r#box,
                    ScreenText {
                        row: high + 1.0,
                        ..piece
                    },
                );
            }
        }
        Ok(worded)
    }

    /// The party member a picture of the party's buttons stands for, or None.
    pub fn name(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        let (party, slot) = ((self.party)(), self.slot(picture)?);
        let (true, true, Some(party), Some(slot)) =
            (self.up, PAGE_APPS.contains(&running), party, slot)
        else {
            return Ok(None);
        };
        let head = self.memory.read(party, COUNT + 1)?;
        if !(0 <= slot && slot < i64::from(head[COUNT as usize])) {
            return Ok(None);
        }
        let found = pokemon::read(
            &*self.memory,
            &self.names,
            u32(&head, 4 * slot)?,
            pokemon::SPECIES_MAX,
        )?;
        Ok(found.and_then(|found| (self.called)(&found)))
    }

    /// Which of the party's buttons a picture stands in, by its place, or None.
    fn slot(&self, picture: i64) -> Result<Option<i64>> {
        let layouts = &self.layouts;
        let Some(placement) = layouts.placement(picture)? else {
            return Ok(None);
        };
        let (shape, root, here) = (
            layouts.shape,
            layouts.pane(placement.root)?,
            layouts.pane(picture)?,
        );
        if f32(&here, shape.y)? - f32(&root, shape.y)? < ROWS_BELOW {
            return Ok(None);
        }
        Ok(Some(py::round(
            (f32(&here, shape.x)? - f32(&root, shape.x)? - FIRST_BUTTON) / BUTTONS_APART,
        )?))
    }
}
