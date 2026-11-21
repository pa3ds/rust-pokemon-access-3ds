//! A plain menu's row read every tick (`gen6.screens.rows`).

use std::cell::RefCell;
use std::rc::Rc;

use super::fake_memory::{Counting, FakeMemory};
use crate::core::memory::Memory;
use crate::games::gen6::apps::{BATTLE, FIELD};
use crate::games::gen6::screens::rows;

const PROCESS: i64 = 0x08100000;
const MENU: i64 = 0x08200000;

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
