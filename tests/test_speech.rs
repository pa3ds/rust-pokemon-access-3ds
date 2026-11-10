//! Speech policy tests, against a backend that records instead of talking.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use crate::core::speech::{Backend, Category, Speech, default_interrupts};

#[derive(Default)]
struct Recorder {
    spoken: RefCell<Vec<(String, bool)>>,
    brailled: RefCell<Vec<String>>,
}

impl Backend for Recorder {
    fn speak(&self, text: &str, interrupt: bool) {
        self.spoken.borrow_mut().push((text.to_string(), interrupt));
    }

    fn braille(&self, text: &str) {
        self.brailled.borrow_mut().push(text.to_string());
    }
}

fn spoken(backend: &Recorder) -> Vec<(String, bool)> {
    backend.spoken.borrow().clone()
}

fn texts(backend: &Recorder) -> Vec<String> {
    backend
        .spoken
        .borrow()
        .iter()
        .map(|(text, _)| text.clone())
        .collect()
}

/// The screen reader cuts speech on a key press already.
#[test]
fn test_nothing_interrupts_by_default() {
    let backend = Rc::new(Recorder::default());
    let speech = Speech::plain(backend.clone());
    for category in Category::ALL {
        if category != Category::Key && category != Category::Warning {
            speech.say("Bag", category);
        }
    }
    assert!(spoken(&backend).iter().all(|(_, interrupt)| !interrupt));
}

/// The player asked for it now, and the key never reached the game's window for the screen
/// reader to cut speech on.
#[test]
fn test_the_answer_to_a_key_interrupts() {
    let backend = Rc::new(Recorder::default());
    Speech::plain(backend.clone()).say("door, 1 north", Category::Key);
    assert_eq!(spoken(&backend), [("door, 1 north".to_string(), true)]);
}

/// "Close Azahar" must not wait behind the console being read out.
#[test]
fn test_a_warning_interrupts_from_any_window() {
    let backend = Rc::new(Recorder::default());
    Speech::new(
        backend.clone(),
        default_interrupts(),
        Some(Rc::new(|| false)),
    )
    .say("close Azahar", Category::Warning);
    assert_eq!(spoken(&backend), [("close Azahar".to_string(), true)]);
}

#[test]
fn test_menus_can_be_made_to_interrupt() {
    let backend = Rc::new(Recorder::default());
    Speech::new(backend.clone(), HashSet::from([Category::Menu]), None).say("Bag", Category::Menu);
    assert_eq!(spoken(&backend), [("Bag".to_string(), true)]);
}

#[test]
fn test_dialogue_queues() {
    let backend = Rc::new(Recorder::default());
    Speech::plain(backend.clone()).say("Welcome.", Category::Dialogue);
    assert_eq!(spoken(&backend), [("Welcome.".to_string(), false)]);
}

#[test]
fn test_the_interrupt_policy_is_configurable() {
    let backend = Rc::new(Recorder::default());
    Speech::new(backend.clone(), HashSet::from([Category::Dialogue]), None)
        .say("Welcome.", Category::Dialogue);
    assert_eq!(spoken(&backend), [("Welcome.".to_string(), true)]);
}

/// The game showing the same words again is heard again. Readers never report a still state.
#[test]
fn test_the_same_text_twice_is_said_twice() {
    let backend = Rc::new(Recorder::default());
    let speech = Speech::plain(backend.clone());
    assert!(speech.say("It's a doll!", Category::Dialogue));
    assert!(speech.say("It's a doll!", Category::Dialogue));
    assert_eq!(texts(&backend), ["It's a doll!", "It's a doll!"]);
}

#[test]
fn test_line_breaks_are_spoken_as_spaces() {
    let backend = Rc::new(Recorder::default());
    Speech::plain(backend.clone()).say("Wrapped at\nthe edge.", Category::Dialogue);
    assert_eq!(
        spoken(&backend),
        [("Wrapped at the edge.".to_string(), false)]
    );
}

#[test]
fn test_the_game_fonts_gender_glyphs_become_the_signs_a_screen_reader_says() {
    let backend = Rc::new(Recorder::default());
    Speech::plain(backend.clone()).say("Torchic\u{e08e} Torchic \u{e08f}", Category::Screen);
    assert_eq!(
        spoken(&backend),
        [("Torchic♂ Torchic ♀".to_string(), false)]
    );
}

#[test]
fn test_blank_text_says_nothing() {
    let backend = Rc::new(Recorder::default());
    assert!(!Speech::plain(backend.clone()).say(" \n ", Category::Dialogue));
    assert!(spoken(&backend).is_empty());
}

#[test]
fn test_braille_gets_what_speech_got() {
    let backend = Rc::new(Recorder::default());
    Speech::plain(backend.clone()).say("Bag", Category::Menu);
    assert_eq!(*backend.brailled.borrow(), ["Bag"]);
}

/// Reading another window is never talked over (ADR-0021).
#[test]
fn test_the_game_is_not_said_while_another_window_is_in_front() {
    let backend = Rc::new(Recorder::default());
    let speech = Speech::new(
        backend.clone(),
        HashSet::from([Category::Key]),
        Some(Rc::new(|| false)),
    );
    for category in [
        Category::Dialogue,
        Category::Menu,
        Category::Screen,
        Category::Battle,
        Category::Prompt,
        Category::Guidance,
    ] {
        assert!(!speech.say("Welcome to the world of Pokémon!", category));
    }
    assert!(spoken(&backend).is_empty());
    assert!(backend.brailled.borrow().is_empty());
}

/// "pa3ds ready" comes while the player may still be in the window they started it from, and
/// a key's answer comes only while the game is in front anyway.
#[test]
fn test_pa3ds_own_lines_and_key_answers_are_said_in_any_window() {
    let backend = Rc::new(Recorder::default());
    let speech = Speech::new(
        backend.clone(),
        HashSet::from([Category::Key]),
        Some(Rc::new(|| false)),
    );
    speech.say("pa3ds ready", Category::System);
    speech.say("door, 1 north", Category::Key);
    assert_eq!(texts(&backend), ["pa3ds ready", "door, 1 north"]);
}

#[test]
fn test_the_game_is_said_again_once_its_window_is_in_front() {
    let backend = Rc::new(Recorder::default());
    let front = Rc::new(Cell::new(false));
    let listening = front.clone();
    let speech = Speech::new(
        backend.clone(),
        HashSet::from([Category::Key]),
        Some(Rc::new(move || listening.get())),
    );
    speech.say("Hello there!", Category::Dialogue);
    front.set(true);
    speech.say("Glad to meet you!", Category::Dialogue);
    assert_eq!(texts(&backend), ["Glad to meet you!"]);
}
