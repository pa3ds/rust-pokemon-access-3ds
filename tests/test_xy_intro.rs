//! X's opening: the boy or the girl in their mirrors, then the three looks of the one chosen, said
//! in pa3ds's own words as the cursor comes onto each (spike 216).

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use crate::games::common::announce::Focus;
use crate::games::gen6::game::OpeningReader;
use crate::games::xy;
use crate::games::xy::intro;
use crate::games::xy::intro::{IntroReader, looks};

const PROCESS: i64 = 0x08C61BE8;
const HEAP: i64 = 0x081FB124;
const WORK: i64 = 0x081FB234;

fn table(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn words() -> IndexMap<String, IndexMap<String, String>> {
    IndexMap::from([
        (
            "gender".to_string(),
            table(&[("boy", "Boy, red cap"), ("girl", "Girl, pink hat")]),
        ),
        (
            "boy".to_string(),
            table(&[
                ("left", "Pale skin"),
                ("middle", "Fair skin"),
                ("right", "Brown skin"),
            ]),
        ),
        (
            "girl".to_string(),
            table(&[
                ("left", "Pale skin, long hair"),
                ("middle", "Fair skin, long hair"),
                ("right", "Brown skin, long hair"),
            ]),
        ),
    ])
}

/// The intro's process and its work, as X lays them out.
struct Opening {
    memory: Rc<FakeMemory>,
    reader: IntroReader,
}

impl Opening {
    fn new() -> Self {
        let memory = FakeMemory::new();
        memory.u32(PROCESS + intro::HEAP, &[HEAP]);
        memory.u32(PROCESS + intro::WORK, &[WORK]);
        memory.u32(WORK + intro::BACK, &[PROCESS + intro::BACK_TO, HEAP]);
        let reader = IntroReader::new(memory.clone(), words());
        Opening { memory, reader }
    }

    fn show(&mut self, step: i64, gender: i64, place: i64) -> Option<Focus> {
        self.memory.u32(WORK + intro::STEP, &[step]);
        self.memory.write(WORK + intro::GENDER, &[gender as u8]);
        self.memory.write(WORK + intro::LOOK, &[place as u8]);
        self.reader.poll(PROCESS).unwrap()
    }
}

fn focus(label: &str, index: i64, count: i64, opened: bool) -> Focus {
    Focus {
        source: WORK,
        label: label.to_string(),
        index,
        count,
        opened,
    }
}

#[test]
fn test_the_mirrors_say_the_boy_as_they_open_and_each_one_moved_to() {
    let mut opening = Opening::new();
    assert_eq!(opening.show(0x0B, 0, 0), None);
    assert_eq!(
        opening.show(intro::CHOOSING_GENDER, 0, 0),
        Some(focus("Boy, red cap", 0, 2, true))
    );
    assert_eq!(opening.show(intro::CHOOSING_GENDER, 0, 0), None);
    assert_eq!(
        opening.show(intro::CHOOSING_GENDER, 1, 0),
        Some(focus("Girl, pink hat", 1, 2, false))
    );
}

/// The mirror's byte stays on the girl chosen, and the portraits are hers.
#[test]
fn test_the_portraits_are_the_chosen_ones_looks() {
    let mut opening = Opening::new();
    opening.show(intro::CHOOSING_GENDER, 1, 0);
    assert_eq!(opening.show(0x0E, 1, 0), None);
    assert_eq!(
        opening.show(intro::CHOOSING_LOOK, 1, 0),
        Some(focus("Pale skin, long hair", 0, 3, true))
    );
    assert_eq!(
        opening.show(intro::CHOOSING_LOOK, 1, 2),
        Some(focus("Brown skin, long hair", 2, 3, false))
    );
}

#[test]
fn test_a_work_that_does_not_point_back_is_not_read() {
    let mut opening = Opening::new();
    opening.memory.u32(WORK + intro::WORK_HEAP, &[0x08000000]);
    assert_eq!(opening.show(intro::CHOOSING_GENDER, 0, 0), None);
}

#[test]
fn test_every_choice_has_words_and_a_language_without_a_file_has_the_english() {
    let english = looks(xy::LOOKS, "en");
    assert_eq!(english["gender"].len(), intro::GENDERS.len());
    for gender in intro::GENDERS {
        assert!(english["gender"].contains_key(gender));
    }
    for gender in intro::GENDERS {
        assert_eq!(english[gender].len(), intro::PLACES.len());
        for place in intro::PLACES {
            assert!(english[gender].contains_key(place));
        }
    }
    assert_eq!(looks(xy::LOOKS, "fr"), english);
}

#[test]
fn test_a_missing_folder_is_no_words_at_all() {
    let nothing = IndexMap::<String, String>::new;
    assert_eq!(
        looks(&[], "en"),
        IndexMap::from([
            ("gender".to_string(), nothing()),
            ("boy".to_string(), nothing()),
            ("girl".to_string(), nothing()),
        ])
    );
}
