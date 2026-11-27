//! Dowsing Machine tests, against the field's work and record as omega_ruby_item-finder.ctm read
//! them (2026-09-29).

use std::cell::Cell;
use std::rc::Rc;

use super::fake_memory::FakeMemory;
use crate::core::speech::Category;
use crate::games::gen6::field::dowsing::{LEVEL, Lights, PARENT, WORK};
use crate::games::gen6::field::voice::Said;

const PROCESS: i64 = 0x08C65224;
const FIELD_WORK: i64 = 0x08248DD4;
const RECORD: i64 = 0x0828B748;
const OFFSET: i64 = 0x1D0;

fn lights(level: i64) -> (Rc<FakeMemory>, Lights) {
    lights_under(level, FIELD_WORK)
}

fn lights_under(level: i64, parent: i64) -> (Rc<FakeMemory>, Lights) {
    let memory = FakeMemory::new();
    memory.u32(PROCESS + WORK, &[FIELD_WORK]);
    memory.u32(FIELD_WORK + OFFSET, &[RECORD]);
    memory.u32(RECORD + PARENT, &[parent]);
    memory.u32(RECORD + LEVEL, &[level]);
    let lights = Lights::new(memory.clone(), OFFSET);
    (memory, lights)
}

#[test]
fn test_a_colour_is_said_once_two_polls_have_read_it() {
    let (_, mut reader) = lights(2);
    assert_eq!(reader.poll(PROCESS).unwrap(), None);
    assert_eq!(
        reader.poll(PROCESS).unwrap().as_deref(),
        Some("dowsing green")
    );
    assert_eq!(reader.poll(PROCESS).unwrap(), None);
}

/// Turning, the level read 2 and then 3 within a frame or two before going off.
#[test]
fn test_a_level_read_for_one_poll_is_not_said() {
    let (memory, mut reader) = lights(0);
    reader.poll(PROCESS).unwrap();
    memory.u32(RECORD + LEVEL, &[3]);
    assert_eq!(reader.poll(PROCESS).unwrap(), None);
    memory.u32(RECORD + LEVEL, &[0]);
    assert_eq!(reader.poll(PROCESS).unwrap(), None);
    assert_eq!(reader.poll(PROCESS).unwrap(), None);
}

#[test]
fn test_lights_going_off_are_said_and_off_at_first_is_not() {
    let (memory, mut reader) = lights(0);
    assert_eq!(
        [reader.poll(PROCESS).unwrap(), reader.poll(PROCESS).unwrap()],
        [None, None]
    );
    memory.u32(RECORD + LEVEL, &[4]);
    assert_eq!(
        [reader.poll(PROCESS).unwrap(), reader.poll(PROCESS).unwrap()],
        [None, Some("dowsing red".to_string())]
    );
    memory.u32(RECORD + LEVEL, &[0]);
    assert_eq!(
        [reader.poll(PROCESS).unwrap(), reader.poll(PROCESS).unwrap()],
        [None, Some("dowsing nothing".to_string())]
    );
}

#[test]
fn test_a_record_not_pointing_back_at_the_work_is_not_read() {
    let (_, mut reader) = lights_under(1, 0x08000000);
    assert_eq!(
        [reader.poll(PROCESS).unwrap(), reader.poll(PROCESS).unwrap()],
        [None, None]
    );
}

/// Heard as a field voice, the lights are read only while the field's process is given, and
/// their line is said as screen text, logged as it is said.
#[test]
fn test_the_lights_are_a_field_voice_said_as_screen_text_while_the_field_runs() {
    let memory = lights(2).0;
    let running: Rc<Cell<Option<i64>>> = Rc::new(Cell::new(None));
    let given = running.clone();
    let mut reader = Lights::with_process(memory, OFFSET, Rc::new(move || given.get()));
    assert_eq!(
        [reader.heard(None).unwrap(), reader.heard(None).unwrap()],
        [None, None]
    );
    running.set(Some(PROCESS));
    assert_eq!(reader.heard(None).unwrap(), None);
    assert_eq!(
        reader.heard(None).unwrap(),
        Some(Said {
            words: "dowsing green".to_string(),
            category: Category::Screen,
            logged: "dowsing green".to_string(),
        })
    );
}
