//! The Secret Base PC's greyed categories are said empty (`oras.screens.secret_base`, spike 308).

use std::rc::Rc;

use indexmap::IndexMap;

use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::{FileMatching, LayoutFile};
use crate::games::common::screen::ScreenText;
use crate::games::oras::screens::secret_base;
use crate::py::Result;

const LIST: i64 = 0x08600000;
const OTHER: i64 = 0x08700000;

/// `FILES[root]`.
fn files(root: i64) -> &'static str {
    match root {
        LIST => secret_base::LIST,
        OTHER => "bag_lower.bclyt",
        _ => panic!("KeyError: {root}"),
    }
}

struct FakeLayouts;

impl TrackedLayouts for FakeLayouts {
    fn works(&self) -> IndexMap<i64, i64> {
        panic!("SimpleNamespace has no attribute 'works'")
    }

    fn pane_names(&self, root: i64) -> Vec<String> {
        vec![files(root).to_string()]
    }

    fn pane_name(&self, _pane: i64) -> Result<String> {
        panic!("SimpleNamespace has no attribute 'pane_name'")
    }
}

struct FakeFiles;

impl FileMatching for FakeFiles {
    fn matching(&self, panes: &[String]) -> Option<Rc<LayoutFile>> {
        Some(Rc::new(LayoutFile {
            name: panes[0].clone(),
            ..LayoutFile::new(Vec::new(), IndexMap::new(), IndexMap::new())
        }))
    }
}

#[test]
fn test_a_greyed_category_is_said_empty_and_nothing_else() {
    let found = IndexMap::from([
        (
            1,
            ScreenText {
                root: LIST,
                faded: true,
                ..ScreenText::new(1, "Desks", false)
            },
        ),
        (
            2,
            ScreenText {
                root: LIST,
                ..ScreenText::new(2, "Plants", false)
            },
        ),
        (
            3,
            ScreenText {
                root: OTHER,
                faded: true,
                ..ScreenText::new(3, "Potion", false)
            },
        ),
    ]);
    let said = secret_base::emptied(found, &FakeFiles, &FakeLayouts);
    let texts: Vec<String> = said.into_values().map(|piece| piece.text).collect();
    assert_eq!(texts, ["Desks, empty", "Plants", "Potion"]);
}
