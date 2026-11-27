//! Sun's Pokédex list: an entry said only once it has settled, as its rows are redrawn late as
//! the list scrolls (spike 317).

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeMemory};
use super::fake_speech::Recorder;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Speech;
use crate::core::words::{self, ENGLISH, Words};
use crate::games::common::battle::REASK;
use crate::games::common::cursors::shared::CursorItem;
use crate::games::common::screen::ScreenText;
use crate::games::common::session::Profile;
use crate::games::sm::field;
use crate::games::sm::profile::{self, Held, SunProfile, settled};

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

/// Rowlet's page is built after A opens it, and only a sweep finds its layout.
///
/// Python makes a bare `SunProfile` with a battle reader that wants no sweep: here a whole one,
/// whose battle reader, having read no HP box, wants none either.
#[test]
fn test_a_blank_pokedex_asks_a_sweep_as_it_goes_blank_and_again_after_a_while() {
    let memory: Rc<dyn Memory> = FakeMemory::new();
    let reader = SunProfile::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory, CODE)),
        Rc::new(Speech::plain(Recorder::new())),
        Rc::new(|_line: &str| {}),
    );
    reader.blank.set(true);
    let asked: Vec<bool> = (0..REASK + 2)
        .map(|_| reader.sweep_wanted().unwrap())
        .collect();
    let mut expected = vec![true];
    expected.extend((0..REASK).map(|_| false));
    expected.push(true);
    assert_eq!(asked, expected);
    reader.blank.set(false);
    assert!(!reader.sweep_wanted().unwrap());
}
