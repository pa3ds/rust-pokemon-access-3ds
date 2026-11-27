//! Sun's Pokédex list: an entry said only once it has settled, as its rows are redrawn late as
//! the list scrolls (spike P).

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::words::{self, ENGLISH, Words};
use crate::games::common::cursors::shared::CursorItem;
use crate::games::common::screen::ScreenText;
use crate::games::sm::field;
use crate::games::sm::profile::{self, Held, settled};

const CURSOR: i64 = 0x30024024;

fn item(words: &[&str]) -> CursorItem {
    CursorItem::new(
        CURSOR,
        words
            .iter()
            .enumerate()
            .map(|(n, word)| ScreenText::new(n as i64, *word, false))
            .collect(),
        false,
    )
}

/// Python makes a `SunProfile` with nothing but its `_held` and asks its `_settled`: here the
/// function it calls, given a `_held` of its own.
fn settle(polls: Vec<Vec<CursorItem>>) -> Vec<String> {
    let mut held = Held::new();
    let mut said: Vec<String> = Vec::new();
    for items in polls {
        said.extend(settled(&mut held, items).unwrap().iter().map(|found| {
            found
                .pieces
                .iter()
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        }));
    }
    said
}

#[test]
fn test_an_entry_redrawn_a_poll_later_is_said_once_as_it_settles() {
    let mut polls = vec![vec![item(&["012"])], vec![item(&["?????", "012"])]];
    polls.extend((0..profile::SETTLE).map(|_| Vec::new()));
    assert_eq!(settle(polls), ["????? 012"]);
}

#[test]
fn test_the_list_read_at_once_is_not_said() {
    let whole = item(&["Rowlet", "001", "?????", "002", "?????", "003"]);
    let mut polls = vec![vec![whole]];
    polls.extend((0..profile::SETTLE + 1).map(|_| Vec::new()));
    assert!(settle(polls).is_empty());
}

#[test]
fn test_a_pokedex_s_counts_are_said_as_caught_and_seen() {
    words::r#use(Some(Rc::new(
        Words::new(
            ENGLISH,
            Some(&field::names()),
            IndexMap::new(),
            Rc::new(|_line: &str| {}),
        )
        .unwrap(),
    )));
    let mut polls = vec![vec![item(&["Alola Pokédex", "3", "9"])]];
    polls.extend((0..profile::SETTLE).map(|_| Vec::new()));
    assert_eq!(settle(polls), ["Alola Pokédex 3 caught 9 seen"]);
}
