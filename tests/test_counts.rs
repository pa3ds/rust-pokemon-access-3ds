//! A Poké Mart's count, said whole at each change (`gen6.screens.counts`).

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::{FileMatching, LayoutFile};
use crate::games::common::screen::ScreenText;
use crate::games::common::screen_order::join;
use crate::games::gen6::screens::counts::Counts;
use crate::py::{Error, Result};

const ROOT: i64 = 0x82F5A5C;
const GROUP: i64 = 0x8322B7C;
const IN_BAG: i64 = 0x831EBAC;
const SIGN: char = '\u{D7}';

struct FakeLayouts;

impl TrackedLayouts for FakeLayouts {
    fn works(&self) -> IndexMap<i64, i64> {
        panic!("_Layouts has no attribute 'works'")
    }

    fn pane_names(&self, root: i64) -> Vec<String> {
        if root == ROOT {
            vec!["shop".to_string()]
        } else {
            Vec::new()
        }
    }

    fn pane_name(&self, pane: i64) -> Result<String> {
        match pane {
            GROUP => Ok("2t".to_string()),
            IN_BAG => Ok("2r".to_string()),
            _ => Err(Error::key(pane.to_string())),
        }
    }
}

struct FakeFiles;

impl FileMatching for FakeFiles {
    fn matching(&self, panes: &[String]) -> Option<Rc<LayoutFile>> {
        (!panes.is_empty()).then(|| {
            Rc::new(LayoutFile {
                name: "shop_lower.bclyt".to_string(),
                ..LayoutFile::new(Vec::new(), IndexMap::new(), IndexMap::new())
            })
        })
    }
}

fn piece(r#box: i64, text: &str, parent: i64, left: f64) -> ScreenText {
    ScreenText {
        left,
        right: left + 10.0,
        root: ROOT,
        parent,
        panel: parent,
        ..ScreenText::new(r#box, text, false)
    }
}

fn screen(number: &str, price: &str) -> Vec<ScreenText> {
    vec![
        piece(1, "In Bag:", IN_BAG, 0.0),
        piece(2, "8", IN_BAG, 40.0),
        piece(3, &SIGN.to_string(), GROUP, 0.0),
        piece(4, number, GROUP, 10.0),
        piece(5, price, GROUP, 60.0),
    ]
}

#[test]
fn test_the_count_is_said_whole_at_every_change_and_its_words_are_its_own() {
    let mut counts = Counts::new(Rc::new(FakeLayouts), Rc::new(FakeFiles), false);
    let first = screen("01", "$ 1,200");
    let (line, owned) = counts.poll(&first).unwrap();
    assert_eq!(line, Some(join(&first[2..], false)));
    assert_eq!(owned, HashSet::from([3, 4, 5]));
    // Held, it moves every poll: each count said, where the screen waited for it to stop.
    assert_eq!(counts.poll(&first).unwrap().0, None);
    for (number, price) in [
        ("99", "$ 118,800"),
        ("98", "$ 117,600"),
        ("97", "$ 116,400"),
    ] {
        assert_eq!(
            counts.poll(&screen(number, price)).unwrap().0,
            Some(join(&screen(number, price)[2..], false))
        );
    }
    // Back to a count said before, it is said again.
    assert_eq!(
        counts.poll(&first).unwrap().0,
        Some(join(&first[2..], false))
    );
    // Gone from the screen, nothing is the count's.
    assert_eq!(counts.poll(&[]).unwrap(), (None, HashSet::new()));
}

#[test]
fn test_xs_count_is_its_group_named_30() {
    struct XLayouts;

    impl TrackedLayouts for XLayouts {
        fn works(&self) -> IndexMap<i64, i64> {
            FakeLayouts.works()
        }

        fn pane_names(&self, root: i64) -> Vec<String> {
            FakeLayouts.pane_names(root)
        }

        fn pane_name(&self, pane: i64) -> Result<String> {
            match pane {
                GROUP => Ok("30".to_string()),
                IN_BAG => Ok("2w".to_string()),
                _ => Err(Error::key(pane.to_string())),
            }
        }
    }

    let mut counts = Counts::new(Rc::new(XLayouts), Rc::new(FakeFiles), false);
    assert_eq!(
        counts.poll(&screen("04", "$ 800")).unwrap().1,
        HashSet::from([3, 4, 5])
    );
}
