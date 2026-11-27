//! A Pokémon's contest conditions as they are said, an Extra (movie33's Pikachu).

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::{FileMatching, LayoutFile};
use crate::games::common::screen::ScreenText;
use crate::games::oras::screens::conditions::{STAR, said, star_labels};
use crate::py::Result;

fn words() -> Vec<String> {
    ["Coolness", "Beauty", "Cuteness", "Cleverness", "Toughness"]
        .iter()
        .map(|word| word.to_string())
        .collect()
}

const PIKACHU: [i64; 5] = [116, 70, 70, 70, 70];
const STARRED: i64 = 0x0822C000;
const PARAMS: i64 = 0x0822A000;
/// The summary's conditions page, which draws the star's outline, and its stats page.
const FILES: [(i64, &str); 2] = [(STARRED, "params_contest.bclyt"), (PARAMS, "params.bclyt")];

struct FakeLayouts;

impl TrackedLayouts for FakeLayouts {
    fn works(&self) -> IndexMap<i64, i64> {
        panic!("_Layouts has no attribute 'works'")
    }

    fn pane_names(&self, root: i64) -> Vec<String> {
        let (_, file) = FILES
            .iter()
            .find(|&&(each, _)| each == root)
            .expect("a root of FILES");
        vec![file.to_string()]
    }

    fn name(&self, _pane: i64) -> Result<Vec<u8>> {
        panic!("_Layouts has no attribute 'name'")
    }
}

struct FakeFiles;

impl FileMatching for FakeFiles {
    fn matching(&self, panes: &[String]) -> Option<Rc<LayoutFile>> {
        let name = &panes[0];
        let star: IndexMap<String, Vec<String>> = if name == FILES[0].1 {
            IndexMap::from([("p_star".to_string(), vec![STAR.to_string()])])
        } else {
            IndexMap::new()
        };
        Some(Rc::new(LayoutFile {
            name: name.clone(),
            ..LayoutFile::new(Vec::new(), star, IndexMap::new())
        }))
    }
}

/// `_piece(root, text, box)`.
fn piece(root: i64, text: &str, r#box: i64) -> ScreenText {
    ScreenText {
        top: true,
        root,
        ..ScreenText::new(root + r#box, text, false)
    }
}

/// In a layout drawing the star alone: a label drawn as a picture hangs from the root, as the
/// Move Reminder's do; a move's category icon beside the star hangs from its row.
#[test]
fn test_the_stars_labels_are_left_to_the_extra_and_a_category_icon_is_no_label() {
    let star = [
        piece(STARRED, "Coolness", 4),
        piece(PARAMS, "HP 199/227", 1),
        ScreenText {
            picture: true,
            parent: STARRED,
            ..piece(STARRED, "Beauty", 6)
        },
        ScreenText {
            picture: true,
            parent: STARRED + 0x100,
            ..piece(STARRED, "Toughness", 5)
        },
    ];
    let words: HashSet<String> = ["Coolness", "Beauty", "Toughness", "HP 199/227"]
        .iter()
        .map(|word| word.to_string())
        .collect();
    assert_eq!(
        star_labels(&FakeLayouts, &FakeFiles, &star, &words),
        HashSet::from([STARRED + 4, STARRED + 6])
    );
}

#[test]
fn test_each_condition_is_said_after_the_games_word_for_it() {
    assert_eq!(
        said(Some("Conditions"), &words(), &PIKACHU, false),
        "Conditions Coolness 116 Beauty 70 Cuteness 70 Cleverness 70 Toughness 70"
    );
}

#[test]
fn test_pauses_put_a_colon_after_the_heading_and_commas_between() {
    assert_eq!(
        said(Some("Conditions"), &words(), &PIKACHU, true),
        "Conditions: Coolness 116, Beauty 70, Cuteness 70, Cleverness 70, Toughness 70"
    );
}

#[test]
fn test_without_the_games_heading_the_conditions_are_said_alone() {
    assert_eq!(
        said(None, &words(), &[0, 0, 0, 0, 0], false),
        "Coolness 0 Beauty 0 Cuteness 0 Cleverness 0 Toughness 0"
    );
}
