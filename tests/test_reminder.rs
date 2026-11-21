//! The Move Reminder draws one page of its moves' effects, and its other page's layouts are put
//! away (`gen6.screens.reminder`, spike B on or_move_reminder_bugs).

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use super::fake_memory::{FakeClasses, FakeMemory};
use crate::games::gen6::screens::{reminder, rows, townmap};

const PROCESS: i64 = 0x08100000;
const PARENT: i64 = 0x08200000;
const BATTLE_VIEW: i64 = 0x08300000;
const CONTEST_VIEW: i64 = 0x08310000;
const BATTLE_UTIL: i64 = 0x08320000;
const CONTEST_UTIL: i64 = 0x08330000;

/// `TABLES[util]`.
fn tables(util: i64) -> i64 {
    if util == BATTLE_UTIL {
        0x08340000
    } else {
        0x08350000
    }
}

/// `WORKS[util]`.
fn works(util: i64) -> i64 {
    if util == BATTLE_UTIL {
        0x08360000
    } else {
        0x08370000
    }
}

/// `CURSOR_WORKS[util]`: the works of each list's cursor's layout, which is no `G2DUtil`'s.
fn cursor_works(util: i64) -> i64 {
    if util == BATTLE_UTIL {
        0x08380000
    } else {
        0x08390000
    }
}

/// Its lower views, the battle effects' and the contest effects', one drawn, each with one
/// layout and its list's cursor.
fn move_reminder(contest_shown: bool) -> (Rc<FakeMemory>, FakeClasses, reminder::Pages) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(PROCESS + rows::REMEMBER_LIST[0], &[PARENT]);
    memory.u32(PARENT, &[classes.add(rows::PARENT_VIEW)]);
    let util_class = classes.add(townmap::UTIL_CLASS);
    let cursor_class = classes.add(reminder::CURSOR_CLASS);
    for (at, view, util, drawn, name) in [
        (
            0x7C,
            BATTLE_VIEW,
            BATTLE_UTIL,
            !contest_shown,
            "N3app13wazaomoidashi9LowerViewE",
        ),
        (
            0x84,
            CONTEST_VIEW,
            CONTEST_UTIL,
            contest_shown,
            rows::LOWER_CONTEST_VIEW,
        ),
    ] {
        memory.u32(PARENT + at, &[view]);
        memory.u32(view, &[classes.add(name)]);
        memory.write(view + reminder::DRAWN, &[drawn as u8]);
        memory.u32(view + townmap::UTIL, &[util]);
        memory.u32(util, &[util_class]);
        memory.u32(util + townmap::LAYOUTS, &[tables(util), 1]);
        memory.u32(tables(util), &[tables(util) + 0x100, 0x101]);
        memory.u32(tables(util) + 0x100 + townmap::WORK, &[works(util)]);
        let (cursor, handle) = (tables(util) + 0x200, tables(util) + 0x300);
        memory.u32(view + reminder::cursors(at).unwrap(), &[cursor]);
        memory.u32(cursor, &[cursor_class]);
        memory.u32(cursor + reminder::CURSOR_LAYOUT, &[handle]);
        memory.u32(handle + townmap::WORK, &[cursor_works(util)]);
    }
    memory.write(PARENT + rows::PAGE, &[contest_shown as u8]);
    let running = Rc::new(RefCell::new(rows::REMEMBER.to_string()));
    let pages = reminder::Pages::new(
        memory.clone(),
        Rc::new(move || Some(running.borrow().clone())),
        Rc::new(|| Some(PROCESS)),
    );
    (memory, classes, pages)
}

/// `reminder.Pages(memory, lambda: app, lambda: PROCESS)`.
fn pages_of(memory: &Rc<FakeMemory>, app: &'static str) -> reminder::Pages {
    reminder::Pages::new(
        memory.clone(),
        Rc::new(move || Some(app.to_string())),
        Rc::new(|| Some(PROCESS)),
    )
}

#[test]
fn test_the_page_not_shown_is_put_away_with_its_lists_cursor() {
    assert_eq!(
        move_reminder(false).2.put_away().unwrap(),
        HashSet::from([works(CONTEST_UTIL), cursor_works(CONTEST_UTIL)])
    );
    assert_eq!(
        move_reminder(true).2.put_away().unwrap(),
        HashSet::from([works(BATTLE_UTIL), cursor_works(BATTLE_UTIL)])
    );
}

/// The page turns some 0.4 s before the views' bytes do (spike B on or_move_reminder_bugs).
#[test]
fn test_a_page_turning_puts_both_pages_away_until_its_views_are_drawn() {
    let (memory, _classes, mut pages) = move_reminder(false);
    memory.write(PARENT + rows::PAGE, &[1]);
    assert_eq!(
        pages.put_away().unwrap(),
        HashSet::from([
            works(BATTLE_UTIL),
            works(CONTEST_UTIL),
            cursor_works(BATTLE_UTIL),
            cursor_works(CONTEST_UTIL),
        ])
    );
}

#[test]
fn test_nothing_is_put_away_but_by_the_move_reminders_own_views() {
    let (memory, classes, _pages) = move_reminder(false);
    memory.u32(PARENT, &[classes.add("N3app13wazaomoidashi9LowerViewE")]);
    assert_eq!(
        pages_of(&memory, rows::REMEMBER).put_away().unwrap(),
        HashSet::new()
    );
    assert_eq!(
        pages_of(&memory, rows::FORGET).put_away().unwrap(),
        HashSet::new()
    );
}

/// X's `ParentView` holds other things where Omega Ruby's holds the contest effects' views
/// (spike B on x_move_reminder).
#[test]
fn test_a_move_reminder_of_one_page_puts_nothing_away() {
    let (memory, classes, _pages) = move_reminder(false);
    memory.u32(CONTEST_VIEW, &[classes.add("N3app13wazaomoidashi9UpperViewE")]);
    assert_eq!(
        pages_of(&memory, rows::REMEMBER).put_away().unwrap(),
        HashSet::new()
    );
}
