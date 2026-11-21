//! An item carried in the Bag, told by the Bag's mode, its steps by the rows beside it.

use std::cell::Cell;
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

struct Bag {
    carrying: Rc<Cell<bool>>,
    sort: BagSort,
}

impl Bag {
    fn new() -> Self {
        let carrying = Rc::new(Cell::new(false));
        let sort = BagSort::new({
            let carrying = carrying.clone();
            Rc::new(move || Ok(carrying.get()))
        });
        Bag { carrying, sort }
    }

    /// `poll(*names, under="Pearl", moved=False)`.
    fn poll(&mut self, names: &[&str]) -> (Option<String>, usize) {
        self.poll_under(names, "Pearl", false)
    }

    fn poll_under(&mut self, names: &[&str], under: &str, moved: bool) -> (Option<String>, usize) {
        let screen = rows(names);
        let (line, poll) = self
            .sort
            .poll(poll_of(&screen, under, moved), &screen)
            .unwrap();
        (line, poll.items.len())
    }
}

fn said(line: &str) -> Option<String> {
    Some(line.to_string())
}

/// or_bag_manual_sorting: the mode turned at 30.5 s, the first step at 33.2 s.
#[test]
fn test_the_pick_up_is_said_as_it_is_made() {
    let mut bag = Bag::new();
    assert_eq!(
        bag.poll_under(&["Amulet Coin", "Pearl", "Macho Brace"], "Pearl", true),
        (None, 1)
    );
    bag.carrying.set(true);
    assert_eq!(
        bag.poll(&["Amulet Coin", "Pearl", "Macho Brace"]),
        (said("moving Pearl"), 0)
    );
    assert_eq!(
        bag.poll(&["Amulet Coin", "Pearl", "Macho Brace"]),
        (None, 0)
    );
}

#[test]
fn test_a_step_says_between_what_once_it_has_held_a_poll() {
    let mut bag = Bag::new();
    bag.carrying.set(true);
    bag.poll(&["Amulet Coin", "Pearl", "Macho Brace", "Icy Rock"]);
    // The cursor's move comes first and is not said: the same item again.
    assert_eq!(
        bag.poll_under(
            &["Amulet Coin", "Pearl", "Macho Brace", "Icy Rock"],
            "Pearl",
            true
        ),
        (None, 0)
    );
    assert_eq!(
        bag.poll(&["Amulet Coin", "Macho Brace", "Pearl", "Icy Rock"]),
        (None, 0)
    );
    assert_eq!(
        bag.poll(&["Amulet Coin", "Macho Brace", "Pearl", "Icy Rock"]),
        (said("between Macho Brace and Icy Rock"), 0)
    );
    assert_eq!(
        bag.poll(&["Macho Brace", "Icy Rock", "Pearl", "Shell Bell"]),
        (None, 0)
    );
    assert_eq!(
        bag.poll(&["Macho Brace", "Icy Rock", "Pearl", "Shell Bell"]),
        (said("between Icy Rock and Shell Bell"), 0)
    );
}

#[test]
fn test_at_the_ends_of_the_list_it_says_the_one_beside_it() {
    let mut bag = Bag::new();
    bag.carrying.set(true);
    bag.poll(&["Amulet Coin", "Pearl", "Macho Brace", "Icy Rock"]);
    bag.poll(&["Pearl", "Amulet Coin", "Macho Brace", "Icy Rock"]);
    assert_eq!(
        bag.poll(&["Pearl", "Amulet Coin", "Macho Brace", "Icy Rock"])
            .0,
        said("above Amulet Coin")
    );
    bag.poll(&["Amulet Coin", "Macho Brace", "Icy Rock", "Pearl"]);
    assert_eq!(
        bag.poll(&["Amulet Coin", "Macho Brace", "Icy Rock", "Pearl"])
            .0,
        said("below Icy Rock")
    );
}

#[test]
fn test_the_cursor_off_the_carried_item_is_no_step() {
    let mut bag = Bag::new();
    bag.carrying.set(true);
    bag.poll(&["Amulet Coin", "Pearl", "Macho Brace", "Icy Rock"]);
    assert_eq!(
        bag.poll_under(
            &["Amulet Coin", "Pearl", "Macho Brace", "Icy Rock"],
            "Macho Brace",
            true
        ),
        (None, 0)
    );
    assert_eq!(
        bag.poll_under(
            &["Amulet Coin", "Pearl", "Macho Brace", "Icy Rock"],
            "Pearl",
            true
        ),
        (None, 0)
    );
    bag.poll(&["Amulet Coin", "Macho Brace", "Pearl", "Icy Rock"]);
    assert_eq!(
        bag.poll(&["Amulet Coin", "Macho Brace", "Pearl", "Icy Rock"])
            .0,
        said("between Macho Brace and Icy Rock")
    );
}

#[test]
fn test_the_drop_says_nothing_and_the_list_is_the_cursors_again() {
    let mut bag = Bag::new();
    bag.carrying.set(true);
    bag.poll(&["Amulet Coin", "Pearl", "Macho Brace"]);
    bag.carrying.set(false);
    assert_eq!(
        bag.poll(&["Amulet Coin", "Pearl", "Macho Brace"]),
        (None, 0)
    );
    bag.carrying.set(true);
    assert_eq!(
        bag.poll(&["Amulet Coin", "Pearl", "Macho Brace"]).0,
        said("moving Pearl"),
        "or_bag_manual_sorting: dropped and picked straight back up, the mode 6 for a poll"
    );
    bag.carrying.set(false);
    assert_eq!(
        bag.poll_under(
            &["Amulet Coin", "Pearl", "Macho Brace"],
            "Macho Brace",
            true
        ),
        (None, 1)
    );
    bag.carrying.set(true);
    assert_eq!(
        bag.poll_under(
            &["Amulet Coin", "Pearl", "Macho Brace"],
            "Macho Brace",
            false
        )
        .0,
        said("moving Macho Brace")
    );
}

#[test]
fn test_walking_the_list_says_nothing_and_leaves_the_cursor_its_line() {
    let mut bag = Bag::new();
    for word in ORDER {
        assert_eq!(bag.poll_under(&ORDER, word, true), (None, 1), "{word}");
    }
}
