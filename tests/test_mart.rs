//! A Poké Mart's list jumped a page, said where it lands (`gen6.screens.mart`).

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::{FileMatching, LayoutFile};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::screens::mart;
use crate::py::Result;

const CALL: i64 = 0x08803C04;
const LIST: i64 = 0x082F5A5C;
const CURSOR: i64 = 0x082F710C;

struct FakeLayouts;

impl TrackedLayouts for FakeLayouts {
    fn works(&self) -> IndexMap<i64, i64> {
        panic!("_Layouts has no attribute 'works'")
    }

    fn pane_names(&self, root: i64) -> Vec<String> {
        if root == LIST {
            vec!["shop".to_string()]
        } else {
            Vec::new()
        }
    }

    fn name(&self, _pane: i64) -> Result<Vec<u8>> {
        panic!("_Layouts has no attribute 'name'")
    }
}

struct FakeFiles;

impl FileMatching for FakeFiles {
    fn matching(&self, panes: &[String]) -> Option<Rc<LayoutFile>> {
        (!panes.is_empty()).then(|| {
            Rc::new(LayoutFile {
                name: mart::LIST.to_string(),
                ..LayoutFile::new(Vec::new(), IndexMap::new(), IndexMap::new())
            })
        })
    }
}

/// A row of the list, its TextBox a number made of its words, as Python's `hash(name) & 0xFFFF`.
fn row(name: &str) -> Vec<ScreenText> {
    let r#box = name
        .bytes()
        .fold(0i64, |hash, byte| (hash * 31 + byte as i64) & 0xFFFF);
    vec![ScreenText {
        root: LIST,
        ..ScreenText::new(r#box, name, false)
    }]
}

#[test]
fn test_a_jump_is_said_where_it_lands_and_a_step_at_once() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(CALL, &[classes.add(mart::BUY)]);
    let mut jumps = mart::Jumps::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        Rc::new(|| vec![CALL + mart::BOX]),
        Rc::new(FakeLayouts),
        Rc::new(FakeFiles),
    );

    let mut poll = |row_at: u8, scroll: i64, under: &str, moved: bool| -> Vec<String> {
        memory.write(CALL + mart::ROW, &[row_at]);
        memory.u32(CALL + mart::SCROLL, &[scroll]);
        let items = if moved {
            vec![CursorItem::new(CURSOR, row(under), false)]
        } else {
            Vec::new()
        };
        let cursors = CursorPoll {
            items,
            under: vec![row(under)],
            lists: IndexMap::new(),
            moving: HashSet::new(),
            rows: HashSet::new(),
        };
        jumps
            .poll(cursors)
            .unwrap()
            .items
            .iter()
            .map(|item| item.pieces[0].text.clone())
            .collect()
    };

    assert_eq!(poll(1, 0, "Great Ball", true), ["Great Ball"]);
    // A step said at once.
    assert_eq!(poll(2, 0, "Ultra Ball", true), ["Ultra Ball"]);
    // A page on, the list scrolls through the rows between: nothing until it holds still.
    assert_eq!(poll(4, 2, "Ultra Ball", true), Vec::<String>::new());
    assert_eq!(poll(4, 4, "Revive", true), Vec::<String>::new());
    assert_eq!(poll(4, 4, "Revive", false), ["Revive"]);
    assert_eq!(poll(4, 4, "Revive", false), Vec::<String>::new());
}
