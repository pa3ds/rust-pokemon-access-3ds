//! Who is in a PC Box's slot, from the save: its boxes by name, the Battle Box, the party's list.

use std::cell::Cell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_pokemon::second_wurmple;
use crate::core::rtti::ClassNames;
use crate::games::common::{pokemon, slots};
use crate::games::gen6::screens::{boxes, partyicons};

const BOXES: i64 = 0x08C9A140;
const NAMES: i64 = 0x08C6DEF8;
const BATTLE: i64 = 0x08C6E33C;
const PARTY: i64 = 0x08CF71F0;
const PARAM: i64 = 0x08CF7300;

#[test]
fn test_the_box_says_its_mode_as_it_changes_and_on_opening_only_where_not_normal() {
    let memory = FakeMemory::new();
    let (process, kept) = (0x08C652BC, 0x0821C560);
    let running: Rc<Cell<Option<i64>>> = Rc::new(Cell::new(Some(process)));
    memory.u32(process + 0x24, &[kept]);
    let mut mode = boxes::Mode::new(memory.clone(), (0x24, 0x21), {
        let running = running.clone();
        Rc::new(move || running.get())
    });

    let poll_at = |mode: &mut boxes::Mode, values: &[u8]| -> Vec<Option<String>> {
        let mut out: Vec<Option<String>> = Vec::new();
        for value in values {
            memory.write(kept + 0x21, &[*value]);
            out.push(mode.poll().unwrap());
        }
        out
    };
    let said = |words: &[Option<&str>]| -> Vec<Option<String>> {
        words.iter().map(|word| word.map(str::to_string)).collect()
    };

    assert_eq!(
        poll_at(&mut mode, &[0, 0, 1, 1, 2, 2, 2, 0, 0]),
        said(&[
            None,
            None,
            None,
            Some("swap mode"),
            None,
            Some("multi-select mode"),
            None,
            None,
            Some("normal mode"),
        ])
    );
    // The Box zeroes it as it closes, one poll before its process goes: nothing is said.
    assert_eq!(
        poll_at(&mut mode, &[1, 1, 0]),
        said(&[None, Some("swap mode"), None])
    );
    running.set(None);
    assert_eq!(mode.poll().unwrap(), None);
    running.set(Some(process));
    assert_eq!(
        poll_at(&mut mode, &[1, 1]),
        said(&[None, Some("swap mode")])
    );
}

#[test]
fn test_a_slot_is_named_from_the_save_of_its_group() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(BOXES, &[classes.add(boxes::BOXES)]);
    memory.u32(NAMES, &[classes.add(boxes::NAMES)]);
    memory.u32(BATTLE, &[classes.add(partyicons::BATTLE_BOX)]);
    for (i, name) in ["BOX 1", "Shiny", "BOX 1"].iter().enumerate() {
        let raw: Vec<u8> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
        memory.write(NAMES + partyicons::RECORDS + boxes::NAME * i as i64, &raw);
    }
    // Box 2's fifth slot, the Battle Box's second.
    memory.write(
        BOXES + partyicons::RECORDS + pokemon::RECORD_SIZE * (boxes::SLOTS + 4),
        &second_wurmple(),
    );
    memory.write(
        BATTLE + partyicons::RECORDS + pokemon::RECORD_SIZE,
        &second_wurmple(),
    );
    memory.u32(
        PARAM,
        &[
            classes.add(pokemon::PARAM_CLASS),
            PARAM + pokemon::RECORD_IN_PARAM,
        ],
    );
    memory.write(PARAM + pokemon::RECORD_IN_PARAM, &second_wurmple());
    memory.u32(PARTY, &[PARAM]);
    memory.u32(PARTY + partyicons::COUNT, &[1]);
    let mut occupants = boxes::Occupants::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        Rc::new(|| Some(PARTY)),
        Rc::new(|| Some(BATTLE)),
    );
    occupants.track(&IndexMap::from([
        (BOXES, boxes::BOXES.to_string()),
        (NAMES, boxes::NAMES.to_string()),
    ]));
    let named =
        |kind: &str, title: Option<&str>, index: i64| occupants.call(kind, title, index).unwrap();
    assert_eq!(
        named(slots::TITLED_GRID, Some("Shiny"), 4).as_deref(),
        Some("Wurmple")
    );
    assert_eq!(named(slots::TITLED_GRID, Some("Shiny"), 3), None);
    // Two boxes of one name are neither.
    assert_eq!(named(slots::TITLED_GRID, Some("BOX 1"), 4), None);
    assert_eq!(
        named(slots::TITLED_LIST, Some("BATTLE BOX"), 1).as_deref(),
        Some("Wurmple")
    );
    assert_eq!(named(slots::PARTY, None, 0).as_deref(), Some("Wurmple"));
    assert_eq!(named(slots::PARTY, None, 1), None);
    assert_eq!(named(slots::GRID, None, 4), None);
}

#[test]
fn test_the_boxs_own_byte_says_which_of_two_boxes_of_one_name_is_shown() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(BOXES, &[classes.add(boxes::BOXES)]);
    memory.u32(NAMES, &[classes.add(boxes::NAMES)]);
    for i in 0..3 {
        let raw: Vec<u8> = "BOX".encode_utf16().flat_map(u16::to_le_bytes).collect();
        memory.write(NAMES + partyicons::RECORDS + boxes::NAME * i, &raw);
    }
    memory.write(
        BOXES + partyicons::RECORDS + pokemon::RECORD_SIZE * (2 * boxes::SLOTS + 4),
        &second_wurmple(),
    );
    let (process, kept) = (0x08C652BC, 0x0821C560);
    let running: Rc<Cell<Option<i64>>> = Rc::new(Cell::new(Some(process)));
    memory.u32(process + 0x24, &[kept]);
    let shown = boxes::shown(memory.clone(), Some((0x24, 0x24)), {
        let running = running.clone();
        Rc::new(move || running.get())
    });
    let mut occupants = boxes::Occupants::with(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        Rc::new(|| None),
        Rc::new(|| None),
        shown.clone(),
    );
    occupants.track(&IndexMap::from([
        (BOXES, boxes::BOXES.to_string()),
        (NAMES, boxes::NAMES.to_string()),
    ]));
    memory.write(kept + 0x24, &[2]);
    assert_eq!(
        occupants
            .call(slots::TITLED_GRID, Some("BOX"), 4)
            .unwrap()
            .as_deref(),
        Some("Wurmple")
    );
    memory.write(kept + 0x24, &[1]);
    assert_eq!(
        occupants.call(slots::TITLED_GRID, Some("BOX"), 4).unwrap(),
        None
    );
    // The Battle Box reads 0xFF; out of the Box, the title alone names none of three.
    memory.write(kept + 0x24, &[0xFF]);
    assert_eq!(shown().unwrap(), Some(0xFF));
    running.set(None);
    assert_eq!(shown().unwrap(), None);
    assert_eq!(
        occupants.call(slots::TITLED_GRID, Some("BOX"), 4).unwrap(),
        None
    );
}

/// `poll_at(*values)`: each value as the byte, and what the titles said of it.
fn poll_at(
    titles: &mut boxes::Titles,
    byte: &Cell<Option<i64>>,
    values: &[Option<i64>],
) -> Vec<Option<String>> {
    let mut out: Vec<Option<String>> = Vec::new();
    for value in values {
        byte.set(*value);
        out.push(titles.poll().unwrap());
    }
    out
}

/// `titles.unsaid()`, sorted: a set of words, which Python compares as a set.
fn unsaid(titles: &boxes::Titles) -> Vec<String> {
    let mut words: Vec<String> = titles.unsaid().unwrap().into_iter().collect();
    words.sort();
    words
}

fn said(words: &[Option<&str>]) -> Vec<Option<String>> {
    words.iter().map(|word| word.map(str::to_string)).collect()
}

#[test]
fn test_a_turn_says_the_box_from_its_byte_and_leaves_the_screens_box_names_unsaid() {
    let byte: Rc<Cell<Option<i64>>> = Rc::new(Cell::new(Some(0)));
    let mut names: Vec<String> = ["BOX 1", "BOX 2", "Shiny"]
        .iter()
        .map(|name| name.to_string())
        .collect();
    names.extend(vec![String::new(); (boxes::COUNT - 3) as usize]);
    let mut titles = boxes::Titles::new(
        {
            let byte = byte.clone();
            Rc::new(move || Ok(byte.get()))
        },
        Rc::new(move || Ok(Some(names.clone()))),
    );
    let titles = &mut titles;

    // Opening says nothing: the cursor's first slot names the box. Then each turn at once.
    assert_eq!(
        poll_at(titles, &byte, &[Some(0), Some(1), Some(2)]),
        said(&[None, Some("BOX 2"), Some("Shiny")])
    );
    // Every other box's name is a title written late, and the one just said already is.
    assert_eq!(unsaid(titles), ["BOX 1", "BOX 2", "Shiny"]);
    poll_at(titles, &byte, &[Some(2); 6]);
    assert_eq!(unsaid(titles), ["BOX 1", "BOX 2"]);
    // The Battle Box's title is the screen's: nothing said, nothing left unsaid.
    assert_eq!(poll_at(titles, &byte, &[Some(0xFF)]), said(&[None]));
    assert_eq!(unsaid(titles), Vec::<String>::new());
    assert_eq!(poll_at(titles, &byte, &[Some(1)]), said(&[Some("BOX 2")]));
    // BOX 1 once it holds a poll: the Box zeroes its work as it closes.
    assert_eq!(
        poll_at(titles, &byte, &[Some(0), Some(0)]),
        said(&[None, Some("BOX 1")])
    );
    assert_eq!(
        poll_at(titles, &byte, &[Some(2), Some(0), None]),
        said(&[Some("Shiny"), None, None])
    );
    // Opened again on BOX 1, nothing.
    assert_eq!(
        poll_at(titles, &byte, &[Some(0), Some(0)]),
        said(&[None, None])
    );
}

/// Holding right turned faster than the screen polls, and said BOX 2, BOX 6, BOX 7
/// (or_pc_box_bugs): each turn is told as it comes, but BOX 1, said once it holds.
#[test]
fn test_a_turn_is_told_the_tick_the_byte_turns_so_no_box_passed_goes_unsaid() {
    let byte: Rc<Cell<Option<i64>>> = Rc::new(Cell::new(None));
    let mut titles = boxes::Titles::new(
        {
            let byte = byte.clone();
            Rc::new(move || Ok(byte.get()))
        },
        Rc::new(|| Ok(Some((1..8).map(|n| format!("BOX {n}")).collect()))),
    );
    assert!(!titles.turned().unwrap()); // not in the Box
    byte.set(Some(2));
    assert!(titles.turned().unwrap());
    titles.poll().unwrap();
    assert!(!titles.turned().unwrap());
    byte.set(Some(3));
    assert!(titles.turned().unwrap());
    byte.set(Some(0));
    assert!(!titles.turned().unwrap());
}
