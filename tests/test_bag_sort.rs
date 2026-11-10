//! An item carried in the Bag, told by a cursor staying on it while the rows beside it change.

use std::rc::Rc;

use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::screens::bag_sort::BagSort;

const ROOT: i64 = 0x8296418;
const ORDER: [&str; 4] = ["Amulet Coin", "Macho Brace", "Icy Rock", "Shell Bell"];

fn rows(names: &[&str]) -> Vec<ScreenText> {
    names
        .iter()
        .enumerate()
        .map(|(i, name)| ScreenText {
            root: ROOT,
            ..ScreenText::placed(
                0x100 + i as i64,
                *name,
                false,
                8.0,
                70.0,
                90.0 - 30.0 * i as f64,
                12.5,
            )
        })
        .collect()
}

fn poll_of(screen: &[ScreenText], word: &str, moved: bool) -> CursorPoll {
    let piece = screen.iter().find(|one| one.text == word).unwrap().clone();
    let items = if moved {
        vec![CursorItem::new(0x800, vec![piece.clone()], false)]
    } else {
        Vec::new()
    };
    CursorPoll {
        items,
        under: vec![vec![piece]],
        ..CursorPoll::default()
    }
}

fn carry(sort: &mut BagSort, names: &[&str], moved: bool) -> (Option<String>, usize) {
    let screen = rows(names);
    let (line, poll) = sort.poll(poll_of(&screen, "Pearl", moved), &screen);
    (line, poll.items.len())
}

/// The cursor comes to rest on the item, and the rows round it finish being drawn.
fn landed(sort: &mut BagSort, names: &[&str]) {
    for _ in 0..3 {
        carry(sort, names, false);
    }
}

fn said(line: &str) -> Option<String> {
    Some(line.to_string())
}

#[test]
fn test_a_step_says_between_what_once_it_has_held_a_poll() {
    let mut sort = BagSort::new(Rc::new(|| true));
    landed(
        &mut sort,
        &["Amulet Coin", "Pearl", "Macho Brace", "Icy Rock"],
    );
    assert_eq!(
        carry(
            &mut sort,
            &["Amulet Coin", "Pearl", "Macho Brace", "Icy Rock"],
            false
        ),
        (None, 0)
    );
    // The cursor's move comes first and is not said: the same item again.
    assert_eq!(
        carry(
            &mut sort,
            &["Amulet Coin", "Pearl", "Macho Brace", "Icy Rock"],
            true
        ),
        (None, 0)
    );
    assert_eq!(
        carry(
            &mut sort,
            &["Amulet Coin", "Macho Brace", "Pearl", "Icy Rock"],
            false
        ),
        (None, 0)
    );
    assert_eq!(
        carry(
            &mut sort,
            &["Amulet Coin", "Macho Brace", "Pearl", "Icy Rock"],
            false
        ),
        (said("moving Pearl, between Macho Brace and Icy Rock"), 0)
    );
    assert_eq!(
        carry(
            &mut sort,
            &["Macho Brace", "Icy Rock", "Pearl", "Shell Bell"],
            false
        ),
        (None, 0)
    );
    assert_eq!(
        carry(
            &mut sort,
            &["Macho Brace", "Icy Rock", "Pearl", "Shell Bell"],
            false
        ),
        (said("between Icy Rock and Shell Bell"), 0)
    );
}

#[test]
fn test_at_the_ends_of_the_list_it_says_the_one_beside_it() {
    let mut sort = BagSort::new(Rc::new(|| true));
    landed(
        &mut sort,
        &["Amulet Coin", "Pearl", "Macho Brace", "Icy Rock"],
    );
    carry(
        &mut sort,
        &["Pearl", "Amulet Coin", "Macho Brace", "Icy Rock"],
        false,
    );
    assert_eq!(
        carry(
            &mut sort,
            &["Pearl", "Amulet Coin", "Macho Brace", "Icy Rock"],
            false
        )
        .0,
        said("moving Pearl, above Amulet Coin")
    );
    carry(
        &mut sort,
        &["Amulet Coin", "Macho Brace", "Icy Rock", "Pearl"],
        false,
    );
    assert_eq!(
        carry(
            &mut sort,
            &["Amulet Coin", "Macho Brace", "Icy Rock", "Pearl"],
            false
        )
        .0,
        said("below Icy Rock")
    );
}

#[test]
fn test_walking_the_list_says_nothing_and_leaves_the_cursor_its_line() {
    let mut sort = BagSort::new(Rc::new(|| true));
    for word in ORDER {
        let screen = rows(&ORDER);
        let (line, poll) = sort.poll(poll_of(&screen, word, true), &screen);
        assert!(line.is_none() && poll.items.len() == 1, "{word}");
    }
}

/// or_bag_and_battle_bag: on turning to a pocket whose first row reads as the cursor's did.
#[test]
fn test_another_pockets_rows_round_the_same_name_are_no_step() {
    let mut sort = BagSort::new(Rc::new(|| true));
    landed(
        &mut sort,
        &["Amulet Coin", "Pearl", "Macho Brace", "Icy Rock"],
    );
    assert_eq!(
        carry(&mut sort, &["Pearl", "Max Repel", "Repel"], false),
        (None, 0)
    );
    assert_eq!(
        carry(&mut sort, &["Pearl", "Max Repel", "Repel"], false),
        (None, 0)
    );
}

/// or_bag_and_battle_bag: a pocket's second row arrived a poll after its cursor.
#[test]
fn test_rows_still_being_drawn_as_the_cursor_lands_are_no_step() {
    let mut sort = BagSort::new(Rc::new(|| true));
    assert_eq!(
        carry(&mut sort, &["Pearl", "X Defense", "Repel"], false),
        (None, 0)
    );
    assert_eq!(
        carry(&mut sort, &["Pearl", "Max Repel", "Repel"], false),
        (None, 0)
    );
    assert_eq!(
        carry(&mut sort, &["Pearl", "Max Repel", "Repel"], false),
        (None, 0)
    );
}

#[test]
fn test_outside_the_bag_nothing_is_taken_from_a_cursor() {
    let mut sort = BagSort::new(Rc::new(|| false));
    let screen = rows(&["Amulet Coin", "Pearl", "Macho Brace"]);
    assert_eq!(
        carry(&mut sort, &["Amulet Coin", "Pearl", "Macho Brace"], true),
        (None, 1)
    );
    assert_eq!(sort.poll(poll_of(&screen, "Pearl", false), &screen).0, None);
}
