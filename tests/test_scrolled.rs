//! A list scrolling under its cursor standing still (`gen6.screens.scrolled`).

use std::collections::HashSet;

use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::screens::scrolled::Scrolled;

const LIST: i64 = 0x08200000;

fn poll(items: Vec<CursorItem>, under: Vec<Vec<ScreenText>>) -> CursorPoll {
    CursorPoll {
        items,
        under,
        ..CursorPoll::default()
    }
}

/// `ScreenText(box, text, changed=False, root=LIST)`.
fn piece(r#box: i64, text: &str) -> ScreenText {
    ScreenText {
        root: LIST,
        ..ScreenText::new(r#box, text, false)
    }
}

/// Search and sort scrolled APPEARANCE's dashes away for ENCOUNTERS' (or_pokedex).
#[test]
fn test_a_row_reading_as_the_last_is_a_move_once_still() {
    let appearance = vec![piece(1, "---")];
    let encounters = vec![piece(2, "---")];
    let list = HashSet::from([LIST]);
    let mut rows = Scrolled::new();
    let items = rows
        .poll(
            poll(
                vec![CursorItem::new(7, appearance.clone(), false)],
                vec![appearance.clone()],
            ),
            &list,
        )
        .items;
    let [said] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(said.pieces, appearance);
    assert!(
        rows.poll(poll(vec![], vec![encounters.clone()]), &list)
            .items
            .is_empty()
    );
    let items = rows
        .poll(poll(vec![], vec![encounters.clone()]), &list)
        .items;
    let [moved] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(
        (moved.cursor, moved.pieces.clone(), moved.opened),
        (7, encounters.clone(), false)
    );
    assert!(
        rows.poll(poll(vec![], vec![encounters.clone()]), &list)
            .items
            .is_empty()
    );
    assert!(
        rows.poll(poll(vec![], vec![appearance]), &HashSet::new())
            .items
            .is_empty()
    );
}

/// Search and sort's picker opening was framed with APPEARANCE's dashes, held back by the
/// cursor reader as an item still being laid out (or_pokedex).
#[test]
fn test_words_the_cursor_reader_held_back_are_no_move() {
    let unset = vec![piece(1, "---")];
    let laid_out = [unset.clone(), vec![piece(3, "Shiny")]].concat();
    let list = HashSet::from([LIST]);
    let mut rows = Scrolled::new();
    rows.poll(
        poll(
            vec![CursorItem::new(7, unset.clone(), false)],
            vec![unset],
        ),
        &list,
    );
    for _ in 0..2 {
        assert!(
            rows.poll(poll(vec![], vec![laid_out.clone()]), &list)
                .items
                .is_empty()
        );
    }
}
