//! The Bag's cursor source: the item its own objects say its list's cursor is on, and its pocket
//! (spike A).

use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{FakeClasses, FakeMemory};
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::screens::bag;

const ITEMS: [&str; 3] = ["???", "Master Ball", "Ultra Ball"];
const POCKETS: [&str; 3] = ["ITEMS", "MEDICINE", "TMs & HMs"];
const PROCESS: i64 = 0x08C652BC;
const MAIN: i64 = 0x0821C560;
const BLOCK: i64 = 0x0821D000;
const DRAW: i64 = 0x0821E000;

fn strings(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

/// `NONE`: a poll with no cursor.
fn none() -> CursorPoll {
    CursorPoll::default()
}

fn nothing() -> IndexMap<i64, String> {
    IndexMap::new()
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

fn chosen_at(memory: &Rc<FakeMemory>, process: Rc<dyn Fn() -> Option<i64>>) -> bag::Chosen {
    bag::Chosen::new(
        memory.clone(),
        bag::ORAS,
        process,
        strings(&ITEMS),
        strings(&POCKETS),
    )
}

fn ultra_ball() -> IndexMap<i64, String> {
    IndexMap::from([(0x0C, "Ultra Ball".to_string())])
}

#[test]
fn test_the_lists_cursor_is_on_the_item_the_bags_objects_say() {
    let memory = FakeMemory::new();
    bag_objects(&memory, 2);
    let running: Rc<Cell<Option<i64>>> = Rc::new(Cell::new(Some(PROCESS)));
    let mut chosen = chosen_at(&memory, {
        let running = running.clone();
        Rc::new(move || running.get())
    });
    // The TM's bar over the party is a cursor of the Bag's too, but not its list's.
    assert_eq!(chosen.poll(&moved(0x0D, 5, &[])).unwrap().0, nothing());
    assert_eq!(chosen.poll(&moved(0x0C, 5, &[5])).unwrap().0, ultra_ball());
    assert_eq!(chosen.poll(&none()).unwrap().0, ultra_ball());
    memory.u16(DRAW + 0x10, 0);
    assert_eq!(chosen.poll(&none()).unwrap().0, nothing(), "an empty pocket");
    running.set(None);
    assert_eq!(
        chosen.poll(&moved(0x0C, 5, &[5])).unwrap(),
        (nothing(), None)
    );
}

/// Built, the Bag shows its list's cursor; before, its pocket may not yet be the one open.
#[test]
fn test_the_pocket_is_named_as_it_changes_once_the_list_has_shown() {
    let memory = FakeMemory::new();
    bag_objects(&memory, 2);
    let mut chosen = chosen_at(&memory, Rc::new(|| Some(PROCESS)));
    assert_eq!(chosen.poll(&none()).unwrap().1, None);
    assert_eq!(
        chosen.poll(&moved(0x0C, 5, &[5])).unwrap().1.as_deref(),
        Some("ITEMS")
    );
    assert_eq!(chosen.poll(&none()).unwrap().1, None);
    memory.write(MAIN + 0x3C, &[2]);
    memory.u16(DRAW + 0x10, 0);
    assert_eq!(
        chosen.poll(&none()).unwrap(),
        (nothing(), Some("TMs & HMs".to_string())),
        "an empty pocket, its list's cursor gone"
    );
    assert_eq!(chosen.poll(&none()).unwrap().1, None);
}

#[test]
fn test_objects_of_other_classes_say_nothing() {
    let memory = FakeMemory::new();
    bag_objects(&memory, 2);
    memory.u32(
        DRAW,
        &[FakeClasses::starting(memory.clone(), 0x00180000).add("N3app3bag8BagFrameE")],
    );
    let mut chosen = chosen_at(&memory, Rc::new(|| Some(PROCESS)));
    assert_eq!(chosen.poll(&moved(0x0C, 5, &[5])).unwrap().0, nothing());
}
