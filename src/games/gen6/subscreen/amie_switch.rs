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

use std::collections::{BTreeSet, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::ImageOf;
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::screens::partyicons::{COUNT, SLOTS};
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
/// How far up from the screen's middle the rows reach: the rows stand at 30, 5 and -20, the
/// party's buttons at 66, and the heart marking the Pokémon chosen among them at 76.
pub const ROWS_BELOW: f64 = 50.0;
/// What the party's buttons draw, one a party member, 50 apart from -125 across once they rest.
pub const BUTTON: &str = "button_pokemon";

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
        let labels: Vec<&ScreenText> = found
            .values()
            .filter(|piece| self.labels.contains(&piece.text))
            .collect();
        if labels.len() != ROWS.len() {
            return Ok(found);
        }
        let rows = self.rows()?;
        let heights: Vec<f64> = labels.iter().map(|piece| piece.row).collect();
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
        let (true, Some(party)) = (PAGE_APPS.contains(&running), (self.party)()) else {
            return Ok(None);
        };
        let Some(slot) = self.slot(picture)? else {
            return Ok(None);
        };
        let head = self.memory.read(party, COUNT + 1)?;
        if !(0 <= slot && slot < i64::from(head[COUNT as usize]).min(SLOTS)) {
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

    /// Which of the party's buttons a picture stands in, or None: its place among the
    /// buttons'. They slide in and out as one, a button apart, so by its place alone the first
    /// button was said as the fourth as it slid in and as the second as it slid out
    /// (or_amie_affection_max, at 3 and at -61 where it rests at -125).
    fn slot(&self, picture: i64) -> Result<Option<i64>> {
        let layouts = &self.layouts;
        let Some(placement) = layouts.placement(picture)? else {
            return Ok(None);
        };
        let root = placement.root;
        let (across, up) = self.place(picture, root)?;
        // Only the screen's own: the page lying under it keeps its pictures, and a visitor there
        // took the name of the button above it, and kept it (or_amie_affection_increase).
        if up < ROWS_BELOW || !self.draws_rows(root)? {
            return Ok(None);
        }
        // `sorted({...})`: a set of the buttons' places, in order.
        let mut buttons: BTreeSet<i64> = BTreeSet::new();
        for (&pane, kind) in &layouts.tree(root) {
            if kind == lyt::PICTURE_CLASS
                && (self.image_of)(pane)?
                    .unwrap_or_default()
                    .starts_with(BUTTON)
            {
                buttons.insert(py::round(self.place(pane, root)?.0)?);
            }
        }
        let across = py::round(across)?;
        Ok(buttons
            .iter()
            .position(|&button| button == across)
            .map(|at| at as i64))
    }

    /// Where a pane stands, across and up, from its layout's root.
    fn place(&self, pane: i64, root: i64) -> Result<(f64, f64)> {
        let (shape, layouts) = (self.layouts.shape, &self.layouts);
        let (here, head) = (layouts.pane(pane)?, layouts.pane(root)?);
        Ok((
            f32(&here, shape.x)? - f32(&head, shape.x)?,
            f32(&here, shape.y)? - f32(&head, shape.y)?,
        ))
    }

    /// Whether a layout shows pictures of all three rows now, as only the Switch screen does
    /// while it is up: the page under it keeps a plate of hearts alone, off the screen's edge,
    /// and the screen's own layout stays loaded under the PlayNav with its rows hidden, where
    /// taken for the screen it named a picture of the page's (or_areanav_distribution's "5w").
    /// Asked of the layout as touch mode names a picture, it needs no poll of the screen's words
    /// before it.
    fn draws_rows(&self, root: i64) -> Result<bool> {
        let layouts = &self.layouts;
        let mut kinds: HashSet<&str> = HashSet::new();
        for (&pane, kind) in &layouts.tree(root) {
            if kind != lyt::PICTURE_CLASS
                || !layouts.placement(pane)?.is_some_and(|shown| shown.shown)
            {
                continue;
            }
            if let Some(image) = (self.image_of)(pane)?
                && let Some(row) = ROWS.iter().find(|&&row| image.starts_with(row))
            {
                kinds.insert(row);
            }
        }
        Ok(ROWS.iter().all(|row| kinds.contains(row)))
    }
}
