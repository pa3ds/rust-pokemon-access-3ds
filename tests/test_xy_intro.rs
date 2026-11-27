//! X's opening: the boy or the girl in their mirrors, then the three looks of the one chosen, said
//! in pa3ds's own words as the cursor comes onto each (spike 216).

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use super::fake_rom::TmpDir;
use super::fake_words::words_in;
use crate::core::words::{ENGLISH, Words};
use crate::games::common::announce::Focus;
use crate::games::common::pictures::MODEL;
use crate::games::xy;
use crate::games::xy::intro;
use crate::games::xy::intro::IntroReader;

const PROCESS: i64 = 0x08C61BE8;
const HEAP: i64 = 0x081FB124;
const WORK: i64 = 0x081FB234;

const WORDS: &str = "
model-boy = Boy, red cap
model-girl = Girl, pink hat
model-boy_left = Pale skin
model-boy_middle = Fair skin
model-boy_right = Brown skin
model-girl_left = Pale skin, long hair
model-girl_middle = Fair skin, long hair
model-girl_right = Brown skin, long hair
";

/// pytest's autouse `looks`: the looks' words in place, read out of a folder of their own.
fn looks() {
    let tmp_path = TmpDir::new();
    words_in(tmp_path.path(), WORDS, "xy", ENGLISH, "pictures.ftl");
}

/// `Words(language, game)`.
fn words(language: &str, game: &std::path::Path) -> Words {
    Words::new(
        language,
        Some(game),
        IndexMap::new(),
        Rc::new(|_line: &str| {}),
    )
    .unwrap()
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
        looks();
        let reader = IntroReader::new(memory.clone());
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
    looks();
    let mut looks: Vec<String> = intro::GENDERS
        .iter()
        .map(|gender| gender.to_string())
        .collect();
    for gender in intro::GENDERS {
        for place in intro::PLACES {
            looks.push(format!("{gender}_{place}"));
        }
    }
    let in_language = |language: &str| -> Vec<Option<String>> {
        let words = words(language, &(xy::DATA)());
        looks
            .iter()
            .map(|look| words.get(&format!("{MODEL}{look}"), &[]))
            .collect()
    };
    let english = in_language(ENGLISH);
    assert!(english.iter().all(Option::is_some));
    assert_eq!(in_language("nl"), english);
}

#[test]
fn test_a_missing_folder_is_no_words_at_all() {
    looks();
    let tmp_path = TmpDir::new();
    let nowhere = words(ENGLISH, &tmp_path.path().join("nowhere").join("xy"));
    assert_eq!(nowhere.get(&format!("{MODEL}boy"), &[]), None);
}
