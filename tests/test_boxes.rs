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
