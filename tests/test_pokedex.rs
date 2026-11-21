//! X's Pokédex list names each entry by the top screen's panel (x_pokedex).

use std::collections::HashSet;

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
fn row() -> ScreenText {
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
    [vec![row()], panel(number, name)].concat()
}

fn moved() -> CursorPoll {
    CursorPoll {
        items: vec![CursorItem::new(0x083615B4, vec![row()], false)],
        under: vec![vec![row()]],
        ..CursorPoll::default()
    }
}

/// `CursorPoll([], [(ROW,)], {}, frozenset(), frozenset())`.
fn nothing() -> CursorPoll {
    CursorPoll {
        under: vec![vec![row()]],
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
    entries.poll(Some(pokedex::APP), moved(), &[row()]);
    for _ in 0..pokedex::WAIT_POLLS - 1 {
        assert!(
            entries
                .poll(Some(pokedex::APP), nothing(), &[row()])
                .items
                .is_empty()
        );
    }
    let items = entries.poll(Some(pokedex::APP), nothing(), &[row()]).items;
    let [item] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(join(&item.pieces, false), "004");
}

#[test]
fn test_another_app_is_left_as_it_is() {
    let cursors = moved();
    assert_eq!(
        pokedex::XyList::new().poll(Some("N3app3bag4ProcE"), cursors.clone(), &[row()]),
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
