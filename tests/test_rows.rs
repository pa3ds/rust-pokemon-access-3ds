//! A plain menu's row read every tick (`gen6.screens.rows`).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::fake_memory::{Counting, FakeClasses, FakeMemory};
use crate::core::memory::Memory;
use crate::games::gen6::apps::{BATTLE, FIELD};
use crate::games::gen6::screens::{boxes, rows};

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
        false,
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
        false,
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

/// ADR-0013: the key under the name entry's cursor says when a move is made, once the keyboard is
/// built (spike B on or_opening and x_opening).
#[test]
fn test_the_name_entrys_key_is_read_through_its_main_system() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(PROCESS + rows::KEYBOARD_SYSTEM, &[WORK]);
    memory.u32(WORK, &[classes.add("N3app8strinput4ProcE")]);
    let mut keyboard = rows::Rows::new(
        memory.clone(),
        Rc::new(|| Some(rows::KEYBOARD.to_string())),
        Rc::new(|| Some(PROCESS)),
        Rc::new(|| Ok(Some(MENU))),
        Rc::new(|| true),
        Some(rows::MENU_ROW),
        Some(rows::OPTIONS_ROW),
        Some(rows::PARTY_SLOT),
        false,
    );
    assert!(!keyboard.moved().unwrap()); // not its main system yet
    memory.u32(WORK, &[classes.add(rows::MAIN_SYSTEM)]);
    assert!(!keyboard.moved().unwrap()); // its keys not built yet
    memory.u32(WORK + rows::KEYS, &[OTHER]);
    assert!(!keyboard.moved().unwrap()); // the keyboard just opened: no move yet
    memory.write(OTHER + rows::KEY, &[0x0B]);
    assert!(keyboard.moved().unwrap());
    assert!(!keyboard.moved().unwrap());
    memory.write(OTHER + rows::KEY, &[0x0C]);
    assert!(keyboard.moved().unwrap());
}

/// ADR-0013: the PC's row says when a move is made, in whichever of its two menus is up; a menu
/// opening in place of the other is no move (spike B on or_pc_organize and x_pc_mode_buttons).
#[test]
fn test_the_pcs_row_is_read_from_the_menu_up_now() {
    let memory = FakeMemory::new();
    let (first, second, holder) = (0x08600000, 0x08700000, 0x08800000);
    let [process, work, other] = rows::controls(rows::PC_MENU).unwrap();
    memory.u32(PROCESS + process, &[WORK]);
    memory.u32(WORK + work, &[OTHER]);
    memory.u32(OTHER + other, &[holder]);
    let mut pc = rows::Rows::new(
        memory.clone(),
        Rc::new(|| Some(rows::PC_MENU.to_string())),
        Rc::new(|| Some(PROCESS)),
        Rc::new(|| Ok(Some(MENU))),
        Rc::new(|| true),
        Some(rows::MENU_ROW),
        Some(rows::OPTIONS_ROW),
        Some(rows::PARTY_SLOT),
        false,
    );
    assert!(!pc.moved().unwrap()); // no menu up yet
    memory.u32(holder + rows::IN_USE, &[first]);
    assert!(!pc.moved().unwrap()); // "Which PC should be accessed?" just up
    memory.u32(holder + rows::IN_USE, &[second]);
    assert!(!pc.moved().unwrap()); // Someone's PC opened, on its first row
    memory.write(second + rows::SLOT, &[1]);
    assert!(pc.moved().unwrap());
    assert!(!pc.moved().unwrap());
    memory.write(first + rows::SLOT, &[2]);
    memory.u32(holder + rows::IN_USE, &[first]);
    assert!(!pc.moved().unwrap()); // back on the first menu, where its cursor was
}

/// ADR-0013: the Box's slot says when a move is made, the grid's and its party's alike; a menu
/// taking the input is no move (spike B on or_organize_boxes).
#[test]
fn test_the_boxs_slot_is_read_from_the_cursor_in_use() {
    let memory = FakeMemory::new();
    let (grid, menu, control) = (0x08600000, 0x08700000, 0x08800000);
    let [process, work, other] = rows::controls(boxes::APP).unwrap();
    memory.u32(PROCESS + process, &[WORK]);
    memory.u32(WORK + work, &[OTHER]);
    memory.u32(OTHER + other, &[control]);
    memory.u32(control + rows::IN_USE, &[grid]);
    memory.write(grid + rows::SLOTS, &[rows::GRID_SLOTS as u8]);
    memory.write(menu + rows::SLOTS, &[6]);
    let mut r#box = rows::Rows::new(
        memory.clone(),
        Rc::new(|| Some(boxes::APP.to_string())),
        Rc::new(|| Some(PROCESS)),
        Rc::new(|| Ok(Some(MENU))),
        Rc::new(|| true),
        Some(rows::MENU_ROW),
        Some(rows::OPTIONS_ROW),
        Some(rows::PARTY_SLOT),
        false,
    );
    assert!(!r#box.moved().unwrap());
    assert_eq!(r#box.box_slot().unwrap(), None); // not moved yet
    memory.write(grid + rows::SLOT, &[30]); // Party 1
    assert!(r#box.moved().unwrap());
    assert_eq!(r#box.box_slot().unwrap(), Some(30));
    memory.u32(control + rows::IN_USE, &[menu]);
    assert!(!r#box.moved().unwrap()); // its menu opened over the Pokémon chosen
    memory.write(menu + rows::SLOT, &[1]);
    assert!(r#box.moved().unwrap());
    assert_eq!(r#box.box_slot().unwrap(), None); // a menu's row is no slot
    memory.u32(control + rows::IN_USE, &[grid]);
    assert!(!r#box.moved().unwrap());
    assert_eq!(r#box.box_slot().unwrap(), None); // back from the menu, not moved since
}

/// ADR-0013: Move to forget's row and the Move Reminder's row and scroll say when a move is
/// made, a move down a long list scrolling it alone; a list built again for the same process,
/// after another app, is found again (spike B on or_move_reminder).
#[test]
fn test_the_move_screens_rows_are_read_through_their_views() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let (lower, cursor) = (0x08600000, 0x08700000);
    memory.u32(PROCESS + rows::FORGET_ROWS[0], &[lower]);
    memory.u32(lower, &[classes.add(rows::LOWER)]);
    memory.u32(lower + rows::FORGET_ROWS[1], &[cursor]);
    let running = running(rows::FORGET);
    let mut moves = rows::Rows::new(
        memory.clone(),
        {
            let running = running.clone();
            Rc::new(move || Some(running.borrow().clone()))
        },
        Rc::new(|| Some(PROCESS)),
        Rc::new(|| Ok(Some(MENU))),
        Rc::new(|| true),
        None,
        None,
        None,
        true,
    );
    assert!(!moves.moved().unwrap());
    memory.write(cursor + rows::SLOT, &[4]); // the move to learn
    assert!(moves.moved().unwrap());
    let (parent, view, first, second) = (0x08800000, 0x08900000, 0x08A00000, 0x08B00000);
    let [hop_parent, hop_view, hop_list] = rows::REMEMBER_LIST;
    memory.u32(PROCESS + hop_parent, &[parent]);
    memory.u32(parent, &[classes.add(rows::PARENT_VIEW)]);
    memory.u32(parent + hop_view, &[view]);
    memory.u32(view + hop_list, &[first]);
    // Row 2, 1.0 scrolled away: the fourth move.
    memory.write(first + rows::LIST_ROW, &[2, 0, 0, 0, 0x00, 0x00, 0x80, 0x3f]);
    *running.borrow_mut() = rows::REMEMBER.to_string();
    assert!(!moves.moved().unwrap());
    memory.write(first + rows::LIST_ROW + 4, &[0x00, 0x00, 0x00, 0x40]); // 2.0: scrolled alone
    assert!(moves.moved().unwrap());
    // Wrapped round to the first move: every row is rewritten after, so the screen poll says it.
    memory.write(first + rows::LIST_ROW, &[0, 0, 0, 0, 0, 0, 0, 0]);
    assert!(!moves.moved().unwrap());
    *running.borrow_mut() = "N3app13wazaomoidashi4ProcE".to_string(); // the Reminder's own, between its lists
    assert!(!moves.moved().unwrap());
    memory.u32(view + hop_list, &[second]);
    *running.borrow_mut() = rows::REMEMBER.to_string();
    assert!(!moves.moved().unwrap());
    memory.write(second + rows::LIST_ROW, &[1]);
    assert!(moves.moved().unwrap());
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
