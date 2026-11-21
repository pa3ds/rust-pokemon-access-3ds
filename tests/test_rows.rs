//! A plain menu's row read every tick (`gen6.screens.rows`).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::fake_memory::{Counting, FakeClasses, FakeMemory};
use crate::core::memory::Memory;
use crate::games::gen6::apps::{BATTLE, FIELD};
use crate::games::gen6::screens::rows;

const PROCESS: i64 = 0x08100000;
const MENU: i64 = 0x08200000;
const WORK: i64 = 0x08300000;
const OTHER: i64 = 0x08400000;
const OTHER_WORK: i64 = 0x08500000;

/// `_rows(memory, running, measured, shown)`.
fn rows_of(
    memory: Rc<dyn Memory>,
    running: &Rc<RefCell<String>>,
    measured: bool,
    shown: bool,
) -> rows::Rows {
    let running = running.clone();
    rows::Rows::new(
        memory,
        Rc::new(move || Some(running.borrow().clone())),
        Rc::new(|| Some(PROCESS)),
        Rc::new(|| Ok(Some(MENU))),
        Rc::new(move || shown),
        measured.then_some(rows::MENU_ROW),
        measured.then_some(rows::OPTIONS_ROW),
        None,
    )
}

fn running(app: &str) -> Rc<RefCell<String>> {
    Rc::new(RefCell::new(app.to_string()))
}

/// ADR-0013: the X menu's and the options' rows say when a move is made.
#[test]
fn test_a_row_moved_is_told_the_tick_its_byte_changes() {
    let memory = FakeMemory::new();
    let running = running(rows::OPTIONS);
    let mut options = rows_of(memory.clone(), &running, true, true);
    assert!(!options.moved().unwrap()); // the screen just opened: no move yet
    memory.write(PROCESS + rows::OPTIONS_ROW, &[1]);
    assert!(options.moved().unwrap());
    assert!(!options.moved().unwrap());
    // Another menu's row is not a move: the X menu, the options screen closed, holds its own.
    memory.write(MENU + rows::MENU_ROW, &[5]);
    *running.borrow_mut() = FIELD.to_string();
    assert!(!options.moved().unwrap());
    memory.write(MENU + rows::MENU_ROW, &[4]);
    assert!(options.moved().unwrap());
    *running.borrow_mut() = BATTLE.to_string();
    assert!(!options.moved().unwrap());
}

/// ADR-0013: the party's slot says when a move is made, its object checked by class once a party
/// screen, and found again for the next screen's (spike B).
#[test]
fn test_the_partys_slot_is_read_from_its_static_work() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(PROCESS + rows::PARTY_WORK, &[WORK]);
    memory.u32(WORK, &[classes.add(rows::STATIC_WORK)]);
    let process = Rc::new(Cell::new(PROCESS));
    let mut party = rows::Rows::new(
        memory.clone(),
        Rc::new(|| Some(rows::PARTY.to_string())),
        {
            let process = process.clone();
            Rc::new(move || Some(process.get()))
        },
        Rc::new(|| Ok(Some(MENU))),
        Rc::new(|| true),
        Some(rows::MENU_ROW),
        Some(rows::OPTIONS_ROW),
        Some(rows::PARTY_SLOT),
    );
    assert!(!party.moved().unwrap());
    memory.write(WORK + rows::PARTY_SLOT, &[2]);
    assert!(party.moved().unwrap());
    assert!(!party.moved().unwrap());
    // Another screen's party: its object is another class until it is built.
    memory.u32(OTHER + rows::PARTY_WORK, &[OTHER_WORK]);
    memory.u32(OTHER_WORK, &[classes.add("N3app8pokelist7ManagerE")]);
    process.set(OTHER);
    assert!(!party.moved().unwrap());
    memory.write(OTHER_WORK + rows::PARTY_SLOT, &[3]);
    assert!(!party.moved().unwrap());
    memory.u32(OTHER_WORK, &[classes.add(rows::STATIC_WORK)]);
    assert!(!party.moved().unwrap());
    memory.write(OTHER_WORK + rows::PARTY_SLOT, &[1]);
    assert!(party.moved().unwrap());
}

#[test]
fn test_a_game_not_measured_reads_nothing_nor_a_screen_with_no_cursor() {
    let memory = Counting::new(FakeMemory::new());
    let mut options = rows_of(memory.clone(), &running(rows::OPTIONS), false, true);
    assert!(!options.moved().unwrap());
    assert!(!options.moved().unwrap());
    let mut field = rows_of(memory.clone(), &running(FIELD), true, false);
    assert!(!field.moved().unwrap());
    assert_eq!(memory.reads.get(), 0);
}
