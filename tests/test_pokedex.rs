//! X's Pokédex list names each entry by the top screen's panel (x_pokedex).

use std::collections::HashSet;
use std::rc::Rc;

use super::fake_memory::{FakeClasses, FakeMemory};
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::screen::{ScreenText, join};
use crate::games::gen6::screens::pokedex;

const LIST: i64 = 0x0831EEBC;
const PANEL: i64 = 0x082D14F8;

/// A piece of the screen: `ScreenText(box, text, changed, row, left, glyph=12.0, root)`.
fn piece(r#box: i64, text: &str, changed: bool, row: f64, left: f64, root: i64) -> ScreenText {
    ScreenText {
        row,
        left,
        glyph: 12.0,
        root,
        ..ScreenText::new(r#box, text, changed)
    }
}

/// Python's `ROW`.
fn list_row() -> ScreenText {
    piece(0x08322FD4, "004", true, 50.0, 49.5, LIST)
}

/// The top screen's line naming an entry: "No.", its number and its name.
fn panel(number: &str, name: &str) -> Vec<ScreenText> {
    vec![
        piece(0x082DAE70, "No.", false, 64.0, -3.8, PANEL),
        piece(0x082DBA08, number, true, 64.0, 15.0, PANEL),
        piece(0x082DC158, name, true, 64.0, 80.0, PANEL),
    ]
}

/// `ROW` followed by the panel's line.
fn with_panel(number: &str, name: &str) -> Vec<ScreenText> {
    [vec![list_row()], panel(number, name)].concat()
}

fn moved() -> CursorPoll {
    CursorPoll {
        items: vec![CursorItem::new(0x083615B4, vec![list_row()], false)],
        under: vec![vec![list_row()]],
        ..CursorPoll::default()
    }
}

/// `CursorPoll([], [(ROW,)], {}, frozenset(), frozenset())`.
fn nothing() -> CursorPoll {
    CursorPoll {
        under: vec![vec![list_row()]],
        ..CursorPoll::default()
    }
}

#[test]
fn test_a_move_is_said_with_the_name_the_panel_gives_its_number() {
    let cursors =
        pokedex::XyList::new().poll(Some(pokedex::APP), moved(), &with_panel("004", "Fennekin"));
    let [item] = &cursors.items[..] else {
        panic!("one item: {:?}", cursors.items);
    };
    assert_eq!(join(&item.pieces, false), "004 Fennekin");
    assert_eq!(
        cursors.rows,
        HashSet::from([0x082DAE70, 0x082DBA08, 0x082DC158])
    );
}

#[test]
fn test_a_move_waits_while_the_panel_names_the_entry_before() {
    let mut entries = pokedex::XyList::new();
    let cursors = entries.poll(Some(pokedex::APP), moved(), &with_panel("003", "?????"));
    assert!(cursors.items.is_empty());
    assert!(cursors.chased);
    let items = entries
        .poll(Some(pokedex::APP), nothing(), &with_panel("004", "Fennekin"))
        .items;
    let [item] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(join(&item.pieces, false), "004 Fennekin");
}

#[test]
fn test_a_move_the_panel_never_names_is_said_by_its_number() {
    let mut entries = pokedex::XyList::new();
    entries.poll(Some(pokedex::APP), moved(), &[list_row()]);
    for _ in 0..pokedex::WAIT_POLLS - 1 {
        assert!(
            entries
                .poll(Some(pokedex::APP), nothing(), &[list_row()])
                .items
                .is_empty()
        );
    }
    let items = entries.poll(Some(pokedex::APP), nothing(), &[list_row()]).items;
    let [item] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(join(&item.pieces, false), "004");
}

#[test]
fn test_another_app_is_left_as_it_is() {
    let cursors = moved();
    assert_eq!(
        pokedex::XyList::new().poll(Some("N3app3bag4ProcE"), cursors.clone(), &[list_row()]),
        cursors
    );
}

/// Search and sort frames a field's value alone: one not set said only its dashes
/// (or_pokedex); one set says what it is.
#[test]
fn test_a_search_field_not_set_is_said_by_its_label() {
    let label = piece(1, "COLOR", false, -22.0, -150.0, PANEL);
    let unset = piece(2, "---", false, -22.0, 35.6, PANEL);
    let red = piece(3, "Red", false, -22.0, 35.6, PANEL);
    for (value, said) in [(unset, "COLOR ---"), (red, "Red")] {
        let cursors = CursorPoll {
            items: vec![CursorItem::new(7, vec![value.clone()], false)],
            ..CursorPoll::default()
        };
        let items = pokedex::labelled(
            Some(pokedex::ORAS_APP),
            cursors,
            &[label.clone(), value.clone()],
        )
        .items;
        let [item] = &items[..] else {
            panic!("one item: {items:?}");
        };
        assert_eq!(join(&item.pieces, false), said);
    }
}

const PROCESS: i64 = 0x08100000;
const LAUNCH: i64 = 0x08200000;
const NUMBER_BOX: i64 = 0x08300010;
const NAME_BOX: i64 = 0x08300020;
const PLATE_ROOT: i64 = 0x082EC748;

/// Omega Ruby's list: `ZukanLaunch` with the chosen entry in its fifth record and the panes
/// showing it in its fourth (spike N).
fn oras_list(index: i64) -> (Rc<FakeMemory>, pokedex::OrasList) {
    let memory = FakeMemory::new();
    memory.u32(PROCESS + pokedex::LAUNCH, &[LAUNCH]);
    memory.u32(
        LAUNCH,
        &[FakeClasses::new(memory.clone()).add(pokedex::LAUNCH_CLASS)],
    );
    memory.u32(LAUNCH + pokedex::CHOSEN, &[index]);
    memory.u32(LAUNCH + pokedex::SHOWN + 4, &[NUMBER_BOX, NAME_BOX]);
    let entries = pokedex::OrasList::new(
        memory.clone(),
        Rc::new(|| Some(pokedex::ORAS_APP.to_string())),
        Rc::new(|| Some(PROCESS)),
    );
    (memory, entries)
}

fn row(number: &str, name: &str) -> Vec<ScreenText> {
    let at = |r#box: i64, text: &str, left: f64| ScreenText {
        left,
        root: PLATE_ROOT,
        ..ScreenText::new(r#box, text, true)
    };
    vec![at(NUMBER_BOX, number, 2.0), at(NAME_BOX, name, 36.0)]
}

/// The plate's own poll: its item, where it says one, from the rows sliding under it.
fn plate(pieces: &[ScreenText], moved: bool) -> CursorPoll {
    let items = if moved {
        vec![CursorItem::new(5, vec![pieces[0].clone()], false)]
    } else {
        Vec::new()
    };
    CursorPoll {
        items,
        under: vec![pieces.to_vec()],
        ..CursorPoll::default()
    }
}

/// The plate said the row sliding under it, a poll or two late; the row the game shows the
/// entry in is said once the index has held still, a jump's passing entries never.
#[test]
fn test_an_entry_is_said_in_its_row_s_words_once_the_index_holds() {
    let (memory, mut entries) = oras_list(3);
    for _ in 0..pokedex::STILL_TICKS + 1 {
        entries.moved().unwrap();
    }
    let items = entries
        .cursors(plate(&row("004", "Torchic"), false), &row("004", "Torchic"))
        .unwrap()
        .items;
    let [opened] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert!(opened.opened && join(&opened.pieces, false) == "004 Torchic");
    memory.u32(LAUNCH + pokedex::CHOSEN, &[5]);
    assert!(entries.moved().unwrap());
    let held = entries
        .cursors(plate(&row("005", "Combusken"), true), &row("006", "Blaziken"))
        .unwrap();
    assert!(held.items.is_empty() && held.chased);
    memory.u32(LAUNCH + pokedex::CHOSEN, &[10]);
    entries.moved().unwrap();
    for _ in 0..pokedex::STILL_TICKS {
        assert!(!entries.moved().unwrap());
    }
    let items = entries
        .cursors(plate(&row("010", "Poochyena"), true), &row("011", "Mightyena"))
        .unwrap()
        .items;
    let [item] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert!(join(&item.pieces, false) == "011 Mightyena" && !item.opened);
}

#[test]
fn test_an_index_moved_with_its_row_s_words_not_yet_waits() {
    let (memory, mut entries) = oras_list(3);
    for _ in 0..pokedex::STILL_TICKS + 1 {
        entries.moved().unwrap();
    }
    entries
        .cursors(plate(&row("004", "Torchic"), false), &row("004", "Torchic"))
        .unwrap();
    memory.u32(LAUNCH + pokedex::CHOSEN, &[4]);
    entries.moved().unwrap();
    for _ in 0..pokedex::STILL_TICKS {
        entries.moved().unwrap();
    }
    assert!(
        entries
            .cursors(plate(&row("004", "Torchic"), true), &row("004", "Torchic"))
            .unwrap()
            .items
            .is_empty()
    );
    let items = entries
        .cursors(plate(&row("005", "Combusken"), true), &row("005", "Combusken"))
        .unwrap()
        .items;
    let [item] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(join(&item.pieces, false), "005 Combusken");
}
