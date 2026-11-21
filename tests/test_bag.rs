//! The Bag's cursor source: the item its own objects say its list's cursor is on (spike A).

use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{FakeClasses, FakeMemory};
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::screens::bag;

const ITEMS: [&str; 3] = ["???", "Master Ball", "Ultra Ball"];
const PROCESS: i64 = 0x08C652BC;
const MAIN: i64 = 0x0821C560;
const BLOCK: i64 = 0x0821D000;
const DRAW: i64 = 0x0821E000;

fn items() -> Vec<String> {
    ITEMS.iter().map(|name| name.to_string()).collect()
}

fn bag_objects(memory: &Rc<FakeMemory>, item: i64) {
    let classes = FakeClasses::new(memory.clone());
    memory.u32(PROCESS + 0x24, &[MAIN]);
    memory.u32(MAIN, &[classes.add(bag::MAIN)]);
    memory.u32(MAIN + 0x58, &[BLOCK]);
    memory.u32(BLOCK, &[DRAW]);
    memory.u32(DRAW, &[classes.add(bag::DRAW)]);
    memory.u16(DRAW + 0x10, item);
}

fn moved(cursor: i64, r#box: i64, rows: &[i64]) -> CursorPoll {
    let row = ScreenText::new(r#box, "Ultra Ball", false);
    CursorPoll {
        items: vec![CursorItem::new(cursor, vec![row.clone()], false)],
        under: vec![vec![row]],
        lists: IndexMap::new(),
        moving: HashSet::new(),
        rows: rows.iter().copied().collect(),
    }
}

fn nothing() -> CursorPoll {
    CursorPoll {
        items: Vec::new(),
        under: Vec::new(),
        lists: IndexMap::new(),
        moving: HashSet::new(),
        rows: HashSet::new(),
    }
}

fn ultra_ball() -> IndexMap<i64, String> {
    IndexMap::from([(0x0C, "Ultra Ball".to_string())])
}

#[test]
fn test_the_lists_cursor_is_on_the_item_the_bags_objects_say() {
    let memory = FakeMemory::new();
    bag_objects(&memory, 2);
    let running: Rc<Cell<Option<i64>>> = Rc::new(Cell::new(Some(PROCESS)));
    let mut chosen = bag::Chosen::new(
        memory.clone(),
        bag::ORAS,
        {
            let running = running.clone();
            Rc::new(move || running.get())
        },
        items(),
    );
    // The TM's bar over the party is a cursor of the Bag's too, but not its list's.
    assert_eq!(chosen.poll(&moved(0x0D, 5, &[])).unwrap(), IndexMap::<i64, String>::new());
    assert_eq!(chosen.poll(&moved(0x0C, 5, &[5])).unwrap(), ultra_ball());
    assert_eq!(chosen.poll(&nothing()).unwrap(), ultra_ball());
    memory.u16(DRAW + 0x10, 0);
    assert_eq!(
        chosen.poll(&nothing()).unwrap(),
        IndexMap::<i64, String>::new(),
        "an empty pocket"
    );
    running.set(None);
    assert_eq!(chosen.poll(&moved(0x0C, 5, &[5])).unwrap(), IndexMap::<i64, String>::new());
}

#[test]
fn test_objects_of_other_classes_say_nothing() {
    let memory = FakeMemory::new();
    bag_objects(&memory, 2);
    memory.u32(
        DRAW,
        &[FakeClasses::starting(memory.clone(), 0x00180000).add("N3app3bag8BagFrameE")],
    );
    let mut chosen = bag::Chosen::new(memory.clone(), bag::ORAS, Rc::new(|| Some(PROCESS)), items());
    assert_eq!(chosen.poll(&moved(0x0C, 5, &[5])).unwrap(), IndexMap::<i64, String>::new());
}
