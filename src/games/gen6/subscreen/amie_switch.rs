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
//! Bound by:
//! - ADR-0037: pictures counted and said in words of pa3ds's own, by the game's own label.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::ImageOf;
use crate::games::gen6::subscreen::amie::METER;
use crate::games::gen6::subscreen::super_training::pictures;
use crate::py::Result;

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

/// Gives the Switch screen's row labels their rows.
pub struct SwitchScreen {
    layouts: Rc<Layouts>,
    image_of: ImageOf,
    /// What the game calls the rows, none where its text is not read.
    labels: Vec<String>,
    /// `pictures`, as a test replaces it.
    pub(crate) pictures: PicturesOf,
}

impl SwitchScreen {
    pub fn new(layouts: Rc<Layouts>, image_of: ImageOf, labels: Vec<String>) -> Self {
        SwitchScreen {
            layouts,
            image_of,
            labels,
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
    pub fn reworded(&self, found: IndexMap<i64, ScreenText>) -> Result<IndexMap<i64, ScreenText>> {
        if found
            .values()
            .filter(|piece| self.labels.contains(&piece.text))
            .count()
            != ROWS.len()
        {
            return Ok(found);
        }
        let rows = self.rows()?;
        Ok(found
            .into_iter()
            .map(|(r#box, piece)| {
                let Some(row) = self.labels.iter().position(|label| *label == piece.text) else {
                    return (r#box, piece);
                };
                let text = own(METER, &[("label", &piece.text), ("count", &rows[row])]);
                (r#box, ScreenText { text, ..piece })
            })
            .collect())
    }
}
