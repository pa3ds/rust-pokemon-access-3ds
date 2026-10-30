//! List window tests, against a fake Yes/No prompt laid out as spikes 14, 18 and 20 read it.

use std::collections::HashSet;
use std::rc::Rc;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::announce::Focus;
use crate::games::gen6::screens::lists::{
    CONTROLLER_DRAWER, DRAWER_CONTROLLER, DRAWER_ITEM_STATE, DRAWER_ITEM_STRIDE, DRAWER_WINDOW,
    ITEM_LABEL, ITEM_RECORD, ListReader, WINDOW_CURSOR, WINDOW_HEIGHT, WINDOW_ITEMS, WINDOW_WIDTH,
};

const CONTROLLER: i64 = 0x08001000;
const DRAWER: i64 = 0x08002000;
const WINDOW: i64 = 0x08003000;
const RECORDS: i64 = 0x08004000;
const STRBUFS: i64 = 0x08005000;
const TEXTS: i64 = 0x08006000;

const HIDDEN: i64 = 0;
const HIGHLIGHTED: i64 = 2;
const CHOSEN: i64 = 6;

struct Prompt {
    memory: Rc<FakeMemory>,
    labels: Vec<&'static str>,
    other: i64,
    reader: ListReader,
}

impl Prompt {
    fn new() -> Prompt {
        Prompt::of(&["YES", "NO"])
    }

    fn of(labels: &[&'static str]) -> Prompt {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        memory.u32(
            CONTROLLER,
            &[classes.add("N3app4tool20MenuWindowControllerE")],
        );
        memory.u32(CONTROLLER + CONTROLLER_DRAWER, &[DRAWER, WINDOW]);
        memory.u32(DRAWER, &[classes.add("N3app4tool13AppMenuDrawerE")]);
        memory.u32(DRAWER + DRAWER_WINDOW, &[WINDOW, CONTROLLER]);
        memory.u32(WINDOW, &[classes.add("N3app4tool10MenuWindowE")]);
        let other = classes.add("N3app4tool8TimeIconE");
        memory.u32(WINDOW + WINDOW_WIDTH, &[labels.len() as i64]);
        memory.u32(WINDOW + WINDOW_HEIGHT, &[1]);
        memory.u32(WINDOW + WINDOW_ITEMS, &[RECORDS]);
        for (i, label) in labels.iter().enumerate() {
            let i = i as i64;
            let (strbuf, text) = (STRBUFS + i * 0x10, TEXTS + i * 0x100);
            let raw: Vec<u8> = label.encode_utf16().flat_map(u16::to_le_bytes).collect();
            let mut written = raw.clone();
            written.extend([0, 0]);
            memory.write(text, &written);
            memory.u32(strbuf + 4, &[text]);
            memory.u16(strbuf + 0x0A, (raw.len() / 2) as i64);
            memory.u32(RECORDS + i * ITEM_RECORD + ITEM_LABEL, &[strbuf]);
        }
        let mut reader = ListReader::new(
            memory.clone(),
            Rc::new(ClassNames::new(memory.clone(), CODE)),
        );
        reader.track([CONTROLLER]);
        let prompt = Prompt {
            memory,
            labels: labels.to_vec(),
            other,
            reader,
        };
        prompt.show(None);
        prompt
    }

    /// Put the cursor on `cursor` with that item highlighted; None hides the list.
    fn show(&self, cursor: Option<i64>) {
        self.show_as(cursor, HIGHLIGHTED);
    }

    /// Put the cursor on `cursor` with that item in `state`; None hides the list.
    fn show_as(&self, cursor: Option<i64>, state: i64) {
        for i in 0..self.labels.len() as i64 {
            let on = cursor.is_some() && Some(i) == cursor;
            self.memory.write(
                DRAWER + DRAWER_ITEM_STATE + i * DRAWER_ITEM_STRIDE,
                &[(if on { state } else { HIDDEN }) as u8],
            );
        }
        self.memory
            .u32(WINDOW + WINDOW_CURSOR, &[cursor.unwrap_or(0)]);
    }

    fn heard(&mut self) -> Vec<(String, bool)> {
        self.reader
            .poll()
            .unwrap()
            .into_iter()
            .map(|focus| (focus.label, focus.opened))
            .collect()
    }
}

fn heard_of(items: &[(&str, bool)]) -> Vec<(String, bool)> {
    items
        .iter()
        .map(|&(label, opened)| (label.to_string(), opened))
        .collect()
}

fn focus(label: &str, index: i64, count: i64) -> Focus {
    Focus {
        source: CONTROLLER,
        label: label.to_string(),
        index,
        count,
        opened: true,
    }
}

#[test]
fn test_a_hidden_list_says_nothing() {
    assert_eq!(Prompt::new().heard(), heard_of(&[]));
}

#[test]
fn test_opening_announces_the_item_under_the_cursor() {
    let mut prompt = Prompt::new();
    prompt.show(Some(0));
    assert_eq!(prompt.reader.poll().unwrap(), vec![focus("YES", 0, 2)]);
}

#[test]
fn test_an_open_list_polled_again_says_nothing() {
    let mut prompt = Prompt::new();
    prompt.show(Some(0));
    prompt.heard();
    assert_eq!(prompt.heard(), heard_of(&[]));
}

#[test]
fn test_moving_the_cursor_announces_the_new_item() {
    let mut prompt = Prompt::new();
    prompt.show(Some(0));
    prompt.heard();
    prompt.show(Some(1));
    assert_eq!(prompt.heard(), heard_of(&[("NO", false)]));
    prompt.show(Some(0));
    assert_eq!(prompt.heard(), heard_of(&[("YES", false)]));
}

#[test]
fn test_choosing_an_item_says_nothing() {
    let mut prompt = Prompt::new();
    prompt.show(Some(1));
    prompt.heard();
    prompt.show_as(Some(1), CHOSEN);
    assert_eq!(prompt.heard(), heard_of(&[]));
}

#[test]
fn test_the_same_prompt_shown_again_is_announced_as_opening_again() {
    let mut prompt = Prompt::new();
    prompt.show(Some(0));
    prompt.heard();
    prompt.show(None);
    assert_eq!(prompt.heard(), heard_of(&[]));
    prompt.show(Some(0));
    assert_eq!(prompt.heard(), heard_of(&[("YES", true)]));
}

#[test]
fn test_a_list_that_opens_on_its_second_item_says_that_item() {
    let mut prompt = Prompt::of(&["Buy", "Sell", "Quit"]);
    prompt.show(Some(2));
    assert_eq!(prompt.reader.poll().unwrap(), vec![focus("Quit", 2, 3)]);
}

#[test]
fn test_a_drawer_that_points_at_another_controller_is_not_this_list() {
    let mut prompt = Prompt::new();
    prompt.memory.u32(DRAWER + DRAWER_CONTROLLER, &[0x08009000]);
    prompt.show(Some(0));
    assert_eq!(prompt.heard(), heard_of(&[]));
}

#[test]
fn test_a_window_pointer_to_something_else_is_not_a_list() {
    let mut prompt = Prompt::new();
    prompt.memory.u32(WINDOW, &[prompt.other]);
    prompt.show(Some(0));
    assert_eq!(prompt.heard(), heard_of(&[]));
}

#[test]
fn test_a_cursor_past_the_last_item_is_not_believed() {
    let mut prompt = Prompt::new();
    prompt.show(Some(0));
    prompt.memory.u32(WINDOW + WINDOW_CURSOR, &[7]);
    assert_eq!(prompt.heard(), heard_of(&[]));
}

#[test]
fn test_the_labels_of_a_list_are_known_while_it_is_on_screen() {
    let mut prompt = Prompt::new();
    prompt.heard();
    assert_eq!(prompt.reader.labels(), HashSet::new());
    prompt.show(Some(1));
    prompt.heard();
    assert_eq!(
        prompt.reader.labels(),
        HashSet::from(["YES".to_string(), "NO".to_string()])
    );
    prompt.show(None);
    prompt.heard();
    assert_eq!(prompt.reader.labels(), HashSet::new());
}

#[test]
fn test_a_list_lit_past_its_cursor_is_up_for_touch_mode_and_says_nothing() {
    // or_touch_battle: the party menu's drawer lit the item after the cursor's from RESTORE on.
    // The cursor reader said RESTORE, and touch mode dropped the menu's buttons while it was up.
    let labels = ["SUMMARY", "USE A MOVE", "RESTORE", "ITEM", "SWITCH", "BACK"];
    let all: HashSet<String> = labels.iter().map(|label| label.to_string()).collect();
    let mut prompt = Prompt::of(&labels);
    prompt.show(Some(1));
    prompt.heard();
    assert!(prompt.reader.lit() == prompt.reader.labels() && prompt.reader.lit() == all);
    prompt.show(Some(3));
    prompt.memory.u32(WINDOW + WINDOW_CURSOR, &[2]);
    assert_eq!(prompt.heard(), heard_of(&[]));
    assert_eq!(
        (prompt.reader.lit(), prompt.reader.labels()),
        (all.clone(), HashSet::new())
    );
    prompt.show(Some(1));
    assert_eq!(prompt.heard(), heard_of(&[("USE A MOVE", true)]));
    prompt.show(None);
    prompt.memory.write(
        DRAWER + DRAWER_ITEM_STATE + 6 * DRAWER_ITEM_STRIDE,
        &[HIGHLIGHTED as u8],
    );
    prompt.heard();
    assert_eq!(prompt.reader.lit(), all);
    prompt.memory.write(
        DRAWER + DRAWER_ITEM_STATE + 6 * DRAWER_ITEM_STRIDE,
        &[CHOSEN as u8],
    );
    prompt.heard();
    assert_eq!(prompt.reader.lit(), HashSet::new());
}

#[test]
fn test_a_list_first_seen_lit_past_its_cursor_is_not_read() {
    let mut prompt = Prompt::new();
    prompt.show(Some(1));
    prompt.memory.u32(WINDOW + WINDOW_CURSOR, &[0]);
    assert_eq!(prompt.heard(), heard_of(&[]));
    assert_eq!(prompt.reader.lit(), HashSet::new());
}

/// Prompts are rebuilt at addresses already used; a sweep need not find them again.
#[test]
fn test_a_controller_missing_from_a_later_sweep_is_still_read() {
    let mut prompt = Prompt::new();
    prompt.reader.track([]);
    prompt.show(Some(0));
    assert_eq!(prompt.heard(), heard_of(&[("YES", true)]));
    prompt.show(Some(1));
    assert_eq!(prompt.heard(), heard_of(&[("NO", false)]));
}
