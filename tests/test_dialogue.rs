//! Dialogue reader tests, against a fake message window laid out as spikes 8 to 14 read it.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::heaps::IN_USE;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::dialogue::{AWAY_POLLS, DialogueReader, PageShown, Shown, TextWritten};
use crate::games::common::{lyt, msgwin};

const CLEAR: &str = "\u{10}\u{1}\u{be01}";
const SCROLL: &str = "\u{10}\u{1}\u{be00}";
const RED: &str = "\u{10}\u{2}\u{ff00}\u{2}";
const PLAIN: &str = "\u{10}\u{2}\u{ff00}\u{0}";

const WINDOW: i64 = 0x08001000;
const OTHER_WINDOW: i64 = 0x08009000;
const RECORDS: i64 = 0x08002000;
const BUFFER: i64 = 0x08003000;
const MESSAGE: i64 = 0x08004000;
const BOXES: [i64; 2] = [0x08010000, 0x08011000];
const TYPED: [i64; 2] = [0x08020000, 0x08021000];
/// The SDK's block header, in front of every window.
const BLOCK: i64 = 0x30;
/// A tag processor that belongs to no window, as the options screen's own does.
const SCREEN_TAG: i64 = 0x00630D80;

/// `text.encode("utf-16-le")`.
fn encode(text: &str) -> Vec<u8> {
    text.encode_utf16().flat_map(u16::to_le_bytes).collect()
}

/// `text.encode("utf-16-le") + b"\0\0"`.
fn terminated(text: &str) -> Vec<u8> {
    let mut raw = encode(text);
    raw.extend([0, 0]);
    raw
}

/// `shown.text`, which both kinds of announcement have.
fn text_of(shown: &Shown) -> &str {
    match shown {
        Shown::Page(page) => &page.text,
        Shown::Text(written) => &written.text,
    }
}

/// Every announcement of a poll, as the pages they are: the tests that read a page's `why` say a
/// page was announced.
fn pages_of(shown: &[Shown]) -> Vec<&PageShown> {
    shown
        .iter()
        .map(|shown| match shown {
            Shown::Page(page) => page,
            Shown::Text(written) => panic!("{written:?} is not a page"),
        })
        .collect()
}

fn page(window: i64, index: i64, text: &str) -> Shown {
    Shown::Page(PageShown::new(window, index, text))
}

/// One two-line message window, its message buffer and its two TextBoxes.
///
/// With `layouts`, the reader is told the TextBoxes sit in a drawn layout, so it knows the box is
/// on screen.
struct Scene {
    memory: Rc<FakeMemory>,
    textbox: i64,
    pane: i64,
    window_class: i64,
    fake: FakeLayouts,
    work: i64,
    reader: DialogueReader,
}

impl Scene {
    fn new(layouts: bool) -> Self {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let textbox = classes.add("N2nw3lyt7TextBoxE");
        let pane = classes.add("N2nw3lyt4PaneE");
        let window_class = classes.add("N5print6MsgWinE");
        memory.u32(OTHER_WINDOW, &[window_class]);
        memory.u32(WINDOW, &[window_class]);
        for window in [WINDOW, OTHER_WINDOW] {
            memory.u16(window - BLOCK, IN_USE);
        }
        memory.u32(WINDOW + msgwin::LINES, &[RECORDS]);
        memory.u32(WINDOW + msgwin::MESSAGE, &[BUFFER]);
        // The count is one byte, and a live window sets the byte after it.
        memory.write(WINDOW + msgwin::LINE_COUNT, &[2, 1]);
        let mut scene = Scene {
            memory: memory.clone(),
            textbox,
            pane,
            window_class,
            fake: FakeLayouts::new(&memory, &classes),
            work: 0,
            reader: bare_reader(&memory, None),
        };
        for (i, r#box) in BOXES.into_iter().enumerate() {
            memory.u32(RECORDS + i as i64 * msgwin::LINE_RECORD, &[r#box]);
            memory.u32(r#box, &[textbox]);
            memory.u32(r#box + lyt::TEXT, &[TYPED[i]]);
            scene.tag(i, WINDOW + msgwin::TAG_PROCESSOR);
            // A real capacity, so reading it as the length would type 134 halfwords and fail.
            memory.u16(r#box + lyt::CAPACITY, 0x86);
        }
        let (work, group) = scene.fake.screen(0x08100000);
        scene.work = work;
        for r#box in BOXES {
            scene.fake.pane_with_vtable(r#box, group, Some(textbox));
        }
        scene.fake.track().unwrap();
        let layouts = layouts.then(|| scene.fake.layouts.clone());
        scene.reader = bare_reader(&memory, layouts);
        scene.reader.track(&[WINDOW], false);
        scene
    }

    fn message(&self, text: &str) {
        let raw = encode(text);
        self.memory.write(MESSAGE, &terminated(text));
        self.memory.u32(BUFFER + 4, &[MESSAGE]);
        self.memory.u16(BUFFER + 0x0A, raw.len() as i64 / 2);
        for i in 0..BOXES.len() {
            self.r#type(i, "");
        }
    }

    fn r#type(&self, line: usize, text: &str) {
        let raw = encode(text);
        self.memory.write(TYPED[line], &terminated(text));
        self.memory
            .u16(BOXES[line] + lyt::LENGTH, raw.len() as i64 / 2);
    }

    fn tag(&self, line: usize, tag_processor: i64) {
        self.memory
            .u32(BOXES[line] + lyt::TAG_PROCESSOR, &[tag_processor]);
    }

    fn close(&self) {
        for i in 0..BOXES.len() {
            self.r#type(i, "");
            self.tag(i, 0);
        }
    }

    fn reopen(&self) {
        for i in 0..BOXES.len() {
            self.tag(i, WINDOW + msgwin::TAG_PROCESSOR);
        }
    }

    /// The box leaving the screen or coming back: both its lines, as the PC's prompt hides them
    /// (spike 192, movie9).
    fn box_shown(&self, shown: bool) {
        for &r#box in &BOXES {
            self.fake.visible(r#box, shown);
        }
    }

    fn said(&mut self) -> Vec<String> {
        self.reader
            .poll()
            .unwrap()
            .iter()
            .map(|shown| text_of(shown).to_string())
            .collect()
    }

    /// Write text into the first line the way the options screen writes a description.
    fn write_over(&self, text: &str) {
        self.tag(0, SCREEN_TAG);
        self.r#type(0, text);
    }
}

/// `DialogueReader(memory, ClassNames(memory, CODE), layouts)`.
fn bare_reader(
    memory: &Rc<FakeMemory>,
    layouts: Option<Rc<crate::games::common::layout::Layouts>>,
) -> DialogueReader {
    DialogueReader::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        layouts,
    )
}

/// The list `said` gives for these.
fn texts(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|text| text.to_string()).collect()
}

#[test]
fn test_a_page_is_said_when_its_first_characters_are_typed() {
    let mut scene = Scene::new(false);
    scene.message(&format!("Welcome to the\ntown!{CLEAR}\nIt is quiet."));
    assert_eq!(scene.said(), texts(&[]));
    scene.r#type(0, "Wel");
    assert_eq!(
        scene.reader.poll().unwrap(),
        [page(WINDOW, 0, "Welcome to the town!")]
    );
}

#[test]
fn test_a_page_is_said_once_however_often_it_is_polled() {
    let mut scene = Scene::new(false);
    scene.message("Hello.");
    scene.r#type(0, "He");
    scene.said();
    scene.r#type(0, "Hello.");
    assert_eq!(scene.said(), texts(&[]));
}

#[test]
fn test_the_next_page_is_said_when_it_starts_typing() {
    let mut scene = Scene::new(false);
    scene.message(&format!("First page.{CLEAR}\nSecond page."));
    scene.r#type(0, "First page.");
    scene.said();
    scene.r#type(0, "Sec");
    assert_eq!(scene.said(), texts(&["Second page."]));
}

#[test]
fn test_a_scrolled_page_wins_over_the_line_left_from_the_page_before() {
    let mut scene = Scene::new(false);
    scene.message(&format!("One line\nand another.{SCROLL}\nThen a third."));
    scene.r#type(0, "One line");
    scene.r#type(1, "and another.");
    scene.said();
    scene.r#type(0, "Then");
    assert_eq!(scene.said(), texts(&["Then a third."]));
}

#[test]
fn test_a_later_page_starting_like_an_earlier_one_is_not_mistaken_for_it() {
    let mut scene = Scene::new(false);
    scene.message(&format!(
        "You are here.{CLEAR}\nIt is late.{CLEAR}\nYou should rest."
    ));
    scene.r#type(0, "You are here.");
    scene.said();
    scene.r#type(0, "It is late.");
    scene.said();
    scene.r#type(0, "You");
    assert_eq!(scene.said(), texts(&["You should rest."]));
}

#[test]
fn test_pages_skipped_between_polls_are_all_said_in_order() {
    let mut scene = Scene::new(false);
    scene.message(&format!("One.{CLEAR}\nTwo.{CLEAR}\nThree."));
    scene.r#type(0, "One.");
    scene.said();
    scene.r#type(0, "Thr");
    assert_eq!(scene.said(), texts(&["Two.", "Three."]));
}

#[test]
fn test_joining_mid_message_says_only_the_page_on_screen() {
    let mut scene = Scene::new(false);
    scene.message(&format!("One.{CLEAR}\nTwo.{CLEAR}\nThree."));
    scene.r#type(0, "Three.");
    assert_eq!(scene.said(), texts(&["Three."]));
}

#[test]
fn test_a_later_message_first_seen_on_its_second_page_says_both() {
    let mut scene = Scene::new(false);
    scene.message("Before.");
    scene.r#type(0, "Before.");
    scene.said();
    scene.message(&format!("One.{SCROLL}\nTwo."));
    scene.r#type(0, "Two.");
    assert_eq!(scene.said(), texts(&["One.", "Two."]));
}

#[test]
fn test_a_window_found_after_startup_is_read_from_its_first_page() {
    let mut scene = Scene::new(false);
    scene.reader = bare_reader(&scene.memory, None);
    scene.reader.track(&[], false);
    scene.message(&format!("One.{CLEAR}\nTwo."));
    scene.r#type(0, "Two.");
    scene.reader.track(&[WINDOW], false);
    assert_eq!(scene.said(), texts(&["One.", "Two."]));
}

#[test]
fn test_a_sweep_that_misses_a_window_does_not_lose_its_place() {
    let mut scene = Scene::new(false);
    scene.message(&format!("One.{CLEAR}\nTwo."));
    scene.r#type(0, "One.");
    scene.said();
    scene.reader.track(&[], false);
    scene.reader.track(&[WINDOW], false);
    assert_eq!(scene.said(), texts(&[]));
    scene.r#type(0, "Two.");
    assert_eq!(scene.said(), texts(&["Two."]));
}

#[test]
fn test_colour_codes_are_matched_as_typed_and_not_spoken() {
    let mut scene = Scene::new(false);
    scene.message(&format!("Go see {RED}the lab{PLAIN} now."));
    scene.r#type(0, &format!("Go see {RED}the"));
    assert_eq!(scene.said(), texts(&["Go see the lab now."]));
}

#[test]
fn test_a_new_message_starts_again_from_its_first_page() {
    let mut scene = Scene::new(false);
    scene.message(&format!("One.{CLEAR}\nTwo."));
    scene.r#type(0, "Two.");
    scene.said();
    scene.message("Another message.");
    scene.r#type(0, "An");
    assert_eq!(scene.said(), texts(&["Another message."]));
}

#[test]
fn test_the_same_message_opened_again_is_said_again() {
    let mut scene = Scene::new(false);
    scene.message("Hello.");
    scene.r#type(0, "Hello.");
    scene.said();
    scene.close();
    assert_eq!(scene.said(), texts(&[]));
    scene.reopen();
    scene.r#type(0, "He");
    assert_eq!(scene.said(), texts(&["Hello."]));
}

#[test]
fn test_a_window_whose_lines_were_destroyed_says_nothing() {
    let mut scene = Scene::new(false);
    scene.message("Left over from an earlier screen.");
    scene.memory.u32(BOXES[1], &[scene.pane]);
    scene.r#type(0, "Left");
    assert_eq!(scene.said(), texts(&[]));
}

/// The window stays open and the buffer gets the same words; only the typing starts over.
#[test]
fn test_talking_to_the_same_person_again_is_heard_again() {
    let mut scene = Scene::new(false);
    scene.message("It's a doll!");
    scene.r#type(0, "It's a doll!");
    assert_eq!(scene.said(), texts(&["It's a doll!"]));
    scene.r#type(0, "");
    assert_eq!(scene.said(), texts(&[]));
    scene.r#type(0, "It'");
    assert_eq!(scene.said(), texts(&["It's a doll!"]));
}

/// Typing that starts over is the game saying the message again, box left the screen or not
/// (reversing their own rule of 2026-09-14).
///
/// That rule was added for the options screen, which retypes the text speed description as a
/// demonstration in a box that never leaves. It cost more than it saved: reading a signpost twice
/// was sometimes silent, because a box that closes and reopens inside a poll is never caught off
/// screen, and with nothing said there is no telling pa3ds having hung from the NPC having
/// finished. The options screen repeating itself is tolerable because nothing interrupts.
#[test]
fn test_a_message_typed_again_while_its_box_stays_on_screen_is_heard_again() {
    let mut scene = Scene::new(true);
    scene.message("Choose a text speed.");
    scene.r#type(0, "Choose a text speed.");
    assert_eq!(scene.said(), texts(&["Choose a text speed."]));
    scene.r#type(0, "Cho");
    assert_eq!(scene.said(), texts(&["Choose a text speed."]));
    scene.r#type(0, "Choose a text speed.");
    assert_eq!(scene.said(), texts(&[]));
}

#[test]
fn test_a_message_typed_again_after_its_box_left_the_screen_is_heard_again() {
    let mut scene = Scene::new(true);
    scene.message("It's a doll!");
    scene.r#type(0, "It's a doll!");
    scene.said();
    scene.fake.draw_with(scene.work, false, true);
    assert_eq!(scene.said(), texts(&[]));
    scene.fake.draw(scene.work);
    scene.r#type(0, "It'");
    assert_eq!(scene.said(), texts(&["It's a doll!"]));
}

/// A signpost's words appear whole, never typed, so only the box leaving the screen and coming
/// back tells a second read from the first (signs.ctm, 2026-09-16).
#[test]
fn test_a_sign_read_twice_is_heard_twice() {
    let mut scene = Scene::new(true);
    scene.message("wee's House");
    scene.r#type(0, "wee's House");
    assert_eq!(scene.said(), texts(&["wee's House"]));
    scene.box_shown(false);
    for _ in 0..AWAY_POLLS {
        assert_eq!(scene.said(), texts(&[]));
    }
    scene.box_shown(true);
    assert_eq!(scene.said(), texts(&["wee's House"]));
    assert_eq!(scene.said(), texts(&[]));
}

/// The BuzzNav scrolls a show's words through two lines, each fading out for a few frames as it
/// leaves the top; the box never leaves the screen (spike 192, movie25).
#[test]
fn test_a_box_scrolling_its_lines_is_not_heard_again() {
    let mut scene = Scene::new(true);
    scene.message("This is an HNN special report.");
    scene.r#type(0, "This is an HNN special report.");
    assert_eq!(scene.said(), texts(&["This is an HNN special report."]));
    for line in [0, 1, 0] {
        scene.fake.fade(BOXES[line], 0);
        for _ in 0..AWAY_POLLS + 1 {
            assert_eq!(scene.said(), texts(&[]));
        }
        scene.fake.fade(BOXES[line], 255);
        assert_eq!(scene.said(), texts(&[]));
    }
}

#[test]
fn test_a_box_that_flickers_off_is_not_heard_again() {
    let mut scene = Scene::new(true);
    scene.message("wee's House");
    scene.r#type(0, "wee's House");
    scene.said();
    scene.box_shown(false);
    scene.said();
    scene.box_shown(true);
    assert_eq!(scene.said(), texts(&[]));
}

/// Birch's box flickered off between pages in the opening, the next page was typed while it
/// was off, and its return said "one another out." twice (movie3 replay, 2026-09-17).
#[test]
fn test_a_page_typed_while_the_box_is_off_is_not_heard_again_as_it_comes_back() {
    let mut scene = Scene::new(true);
    scene.message(&format!("at times fighting{SCROLL}\none another out."));
    scene.r#type(0, "at times fighting");
    scene.said();
    scene.box_shown(false);
    for _ in 0..AWAY_POLLS {
        scene.said();
    }
    scene.r#type(0, "one");
    assert_eq!(scene.said(), texts(&["one another out."]));
    assert_eq!(scene.said(), texts(&[]));
    scene.box_shown(true);
    assert_eq!(scene.said(), texts(&[]));
    assert_eq!(scene.said(), texts(&[]));
}

#[test]
fn test_a_box_that_never_leaves_the_screen_is_not_heard_again() {
    let mut scene = Scene::new(true);
    scene.message("wee's House");
    scene.r#type(0, "wee's House");
    scene.said();
    for _ in 0..3 {
        assert_eq!(scene.said(), texts(&[]));
    }
}

#[test]
fn test_a_box_back_on_screen_mid_message_says_only_the_page_it_shows() {
    let mut scene = Scene::new(true);
    scene.message(&format!("One.{CLEAR}\nTwo.{CLEAR}\nThree."));
    scene.r#type(0, "One.");
    scene.said();
    scene.r#type(0, "Two.");
    assert_eq!(scene.said(), texts(&["Two."]));
    scene.fake.draw_with(scene.work, false, true);
    for _ in 0..AWAY_POLLS {
        assert_eq!(scene.said(), texts(&[]));
    }
    scene.fake.draw(scene.work);
    assert_eq!(scene.said(), texts(&["Two."]));
    scene.r#type(0, "Three.");
    assert_eq!(scene.said(), texts(&["Three."]));
}

#[test]
fn test_a_new_message_after_the_box_left_the_screen_is_read_from_its_first_page() {
    let mut scene = Scene::new(true);
    scene.message("wee's House");
    scene.r#type(0, "wee's House");
    scene.said();
    scene.box_shown(false);
    scene.said();
    scene.message(&format!("One.{SCROLL}\nTwo."));
    scene.r#type(0, "Two.");
    scene.box_shown(true);
    let shown = scene.reader.poll().unwrap();
    assert!(
        shown.iter().all(|page| matches!(page, Shown::Page(_))),
        "every announcement is a PageShown"
    );
    let whys: Vec<(&str, &str)> = pages_of(&shown)
        .iter()
        .map(|page| (page.text.as_str(), page.why.as_str()))
        .collect();
    assert_eq!(whys, [("One.", "new message"), ("Two.", "next")]);
}

/// Both signs of a second conversation at once: the box came back, and the typing started
/// over. Mom's "Now take care out there, honey!" in signs.ctm.
#[test]
fn test_talking_again_after_the_box_left_the_screen_is_heard_once() {
    let mut scene = Scene::new(true);
    scene.message("Now take care out there, honey!");
    scene.r#type(0, "Now take care out there, honey!");
    scene.said();
    scene.box_shown(false);
    for _ in 0..AWAY_POLLS {
        assert_eq!(scene.said(), texts(&[]));
    }
    scene.box_shown(true);
    scene.r#type(0, "No");
    assert_eq!(scene.said(), texts(&["Now take care out there, honey!"]));
    scene.r#type(0, "Now take care out there, honey!");
    assert_eq!(scene.said(), texts(&[]));
}

#[test]
fn test_a_window_rebuilt_with_the_same_message_is_heard_again() {
    let mut scene = Scene::new(false);
    scene.message("Hello.");
    scene.r#type(0, "Hello.");
    scene.said();
    for r#box in BOXES {
        scene.memory.u32(r#box, &[scene.pane]);
    }
    assert_eq!(scene.said(), texts(&[]));
    for r#box in BOXES {
        scene.memory.u32(r#box, &[scene.textbox]);
    }
    scene.r#type(0, "He");
    assert_eq!(scene.said(), texts(&["Hello."]));
}

#[test]
fn test_the_same_text_in_a_new_buffer_is_heard_again() {
    let mut scene = Scene::new(false);
    scene.message("Hello.");
    scene.r#type(0, "Hello.");
    scene.said();
    let other = BUFFER + 0x100;
    scene.memory.u32(other + 4, &[MESSAGE]);
    scene
        .memory
        .u16(other + 0x0A, "Hello.".chars().count() as i64);
    scene.memory.u32(WINDOW + msgwin::MESSAGE, &[other]);
    scene.r#type(0, "He");
    assert_eq!(scene.said(), texts(&["Hello."]));
}

#[test]
fn test_a_scroll_is_not_starting_over() {
    let mut scene = Scene::new(false);
    scene.message(&format!("One line\nand another.{SCROLL}\nThen a third."));
    scene.r#type(0, "One line");
    scene.r#type(1, "and another.");
    assert_eq!(scene.said(), texts(&["One line and another."]));
    scene.r#type(0, "");
    assert_eq!(scene.said(), texts(&[]));
    scene.r#type(0, "Th");
    assert_eq!(scene.said(), texts(&["Then a third."]));
}

#[test]
fn test_a_cleared_page_that_begins_like_the_first_is_not_starting_over() {
    let mut scene = Scene::new(false);
    scene.message(&format!("You are here.{CLEAR}\nYou should rest."));
    scene.r#type(0, "You are here.");
    assert_eq!(scene.said(), texts(&["You are here."]));
    scene.r#type(0, "");
    scene.said();
    scene.r#type(0, "You");
    assert_eq!(scene.said(), texts(&[]));
    scene.r#type(0, "You sh");
    assert_eq!(scene.said(), texts(&["You should rest."]));
}

/// A closed window's records can still point at TextBoxes a new window is typing into.
#[test]
fn test_line_objects_taken_over_by_another_window_say_nothing_here() {
    let mut scene = Scene::new(false);
    scene.message("The old message.");
    for i in 0..BOXES.len() {
        scene.tag(i, OTHER_WINDOW + msgwin::TAG_PROCESSOR);
    }
    scene.r#type(0, "The old");
    assert_eq!(scene.said(), texts(&[]));
}

#[test]
fn test_a_typed_length_beyond_the_capacity_is_not_believed() {
    let mut scene = Scene::new(false);
    scene.message("Hello.");
    scene.r#type(0, "He");
    scene.memory.u16(BOXES[0] + lyt::CAPACITY, 1);
    assert_eq!(scene.said(), texts(&[]));
}

#[test]
fn test_typed_text_matching_no_line_says_nothing() {
    let mut scene = Scene::new(false);
    scene.message("Hello.");
    scene.r#type(0, "Goodbye");
    assert_eq!(scene.said(), texts(&[]));
}

#[test]
fn test_a_page_with_nothing_to_say_is_skipped() {
    let mut scene = Scene::new(false);
    scene.message(&format!("{RED}{PLAIN}{CLEAR}\nReal words."));
    scene.r#type(0, "Real");
    assert_eq!(scene.said(), texts(&["Real words."]));
}

#[test]
fn test_a_malformed_message_says_nothing_rather_than_guessing() {
    let mut scene = Scene::new(false);
    scene.message("Cut off \u{10}\u{5}\u{ff00}");
    scene.r#type(0, "Cut");
    assert_eq!(scene.said(), texts(&[]));
}

#[test]
fn test_a_window_no_longer_tracked_is_not_read() {
    let mut scene = Scene::new(false);
    scene.message("Hello.");
    scene.reader.track(&[], false);
    scene.r#type(0, "Hello.");
    assert_eq!(scene.said(), texts(&[]));
}

/// A second window, laid out over the same two line objects, with its own message.
fn other_window(scene: &Scene, address: i64, text: &str) {
    let (buffer, message) = (address + 0x200, address + 0x400);
    scene.memory.u32(address, &[scene.window_class]);
    scene.memory.u16(address - BLOCK, IN_USE);
    scene.memory.u32(address + msgwin::LINES, &[RECORDS]);
    scene.memory.u32(address + msgwin::MESSAGE, &[buffer]);
    scene.memory.write(address + msgwin::LINE_COUNT, &[2, 1]);
    let raw = encode(text);
    scene.memory.write(message, &terminated(text));
    scene.memory.u32(buffer + 4, &[message]);
    scene.memory.u16(buffer + 0x0A, raw.len() as i64 / 2);
}

const NEW_WINDOW: i64 = 0x08030000;

/// Mom's window and the sign's typed into the same two TextBoxes, and the sign's window was
/// only found by the next sweep, after its box had closed (signs.ctm, spike 60).
#[test]
fn test_a_new_window_typing_into_a_known_message_box_is_read_without_a_sweep() {
    let mut scene = Scene::new(false);
    scene.message("Now take care out there, honey!");
    scene.r#type(0, "Now take care out there, honey!");
    assert_eq!(scene.said(), texts(&["Now take care out there, honey!"]));
    other_window(&scene, NEW_WINDOW, "wee's House");
    for i in 0..BOXES.len() {
        scene.tag(i, NEW_WINDOW + msgwin::TAG_PROCESSOR);
    }
    scene.r#type(0, "wee's House");
    scene.r#type(1, "");
    let shown = scene.reader.poll().unwrap();
    let found: Vec<(i64, &str)> = shown
        .iter()
        .map(|page| match page {
            Shown::Page(page) => (page.window, page.text.as_str()),
            Shown::Text(written) => (written.window, written.text.as_str()),
        })
        .collect();
    assert_eq!(found, [(NEW_WINDOW, "wee's House")]);
}

#[test]
fn test_a_window_the_screen_saw_typing_is_read_from_its_first_page() {
    let mut scene = Scene::new(false);
    scene.reader = bare_reader(&scene.memory, None);
    scene.reader.track(&[], false);
    scene.message(&format!("One.{CLEAR}\nTwo."));
    scene.r#type(0, "Two.");
    assert_eq!(scene.said(), texts(&[]));
    scene.reader.watch(&[BOXES[0]]);
    assert_eq!(scene.said(), texts(&["One.", "Two."]));
}

#[test]
fn test_a_window_found_typing_is_not_lost_to_a_sweep_that_missed_it() {
    let mut scene = Scene::new(false);
    scene.reader.track(&[], false);
    scene.message(&format!("One.{CLEAR}\nTwo."));
    scene.r#type(0, "One.");
    scene.reader.watch(&[BOXES[0]]);
    assert_eq!(scene.said(), texts(&["One."]));
    scene.reader.track(&[], false);
    scene.r#type(0, "Two.");
    assert_eq!(scene.said(), texts(&["Two."]));
}

#[test]
fn test_a_watched_textbox_that_is_gone_is_no_longer_watched() {
    let mut scene = Scene::new(false);
    scene.reader.track(&[], false);
    scene.message("Hello.");
    scene.reader.watch(&[BOXES[0]]);
    scene.memory.u32(BOXES[0], &[scene.pane]);
    assert_eq!(scene.said(), texts(&[]));
    scene.memory.u32(BOXES[0], &[scene.textbox]);
    scene.r#type(0, "He");
    assert_eq!(scene.said(), texts(&[]));
}

#[test]
fn test_text_a_screen_writes_into_a_watched_textbox_finds_no_window() {
    let mut scene = Scene::new(false);
    scene.reader.track(&[], false);
    scene.message("Hello.");
    scene.reader.watch(&BOXES);
    scene.write_over("You can choose from three different text-scrolling speeds.");
    scene.tag(1, 0x10);
    assert_eq!(scene.said(), texts(&[]));
}

/// The screen text reader leaves these to the dialogue reader.
#[test]
fn test_the_line_objects_of_tracked_windows_are_known() {
    let mut scene = Scene::new(false);
    scene.message("Hello.");
    scene.said();
    assert_eq!(
        scene.reader.line_objects().iter().collect::<HashSet<i64>>(),
        HashSet::from(BOXES)
    );
    scene.reader.track(&[], false);
    assert_eq!(
        scene.reader.line_objects().iter().collect::<HashSet<i64>>(),
        HashSet::new()
    );
}

#[test]
fn test_a_window_whose_lines_were_destroyed_has_no_line_objects() {
    let mut scene = Scene::new(false);
    scene.message("Hello.");
    scene.said();
    scene.memory.u32(BOXES[1], &[scene.pane]);
    scene.said();
    assert_eq!(
        scene.reader.line_objects().iter().collect::<HashSet<i64>>(),
        HashSet::new()
    );
}

/// A window whose own message has been read, as the options screen's is when it opens.
fn described() -> Scene {
    let mut scene = Scene::new(false);
    scene.message("Choose a text speed.");
    scene.r#type(0, "Choose a text speed.");
    assert_eq!(scene.said(), texts(&["Choose a text speed."]));
    scene
}

#[test]
fn test_text_written_over_a_window_is_said_once_it_holds() {
    let mut scene = described();
    scene.write_over("Battle effects\ncan be turned off.");
    assert_eq!(scene.said(), texts(&[]));
    assert_eq!(
        scene.reader.poll().unwrap(),
        [Shown::Text(TextWritten {
            window: WINDOW,
            text: "Battle effects can be turned off.".to_string()
        })]
    );
    assert_eq!(scene.said(), texts(&[]));
}

#[test]
fn test_each_new_text_written_is_said_and_one_written_back_is_said_again() {
    let mut scene = described();
    let (effects, style) = ("Battle effects\ncan be off.", "Battle style\ncan be set.");
    for text in [effects, style, effects] {
        scene.write_over(text);
        scene.said();
        assert_eq!(
            scene.said(),
            [text.split_whitespace().collect::<Vec<_>>().join(" ")]
        );
    }
}

#[test]
fn test_the_windows_own_message_typed_again_after_written_text_is_heard_again() {
    let mut scene = described();
    scene.write_over("Battle effects\ncan be off.");
    scene.said();
    scene.said();
    scene.r#type(0, "Choose a");
    assert_eq!(scene.said(), texts(&["Choose a text speed."]));
}

/// The text speed demonstration retypes the description whenever the speed changes.
#[test]
fn test_typing_restarted_under_another_tag_processor_is_not_heard_again() {
    let mut scene = Scene::new(false);
    scene.message("Choose a text speed.");
    scene.tag(0, SCREEN_TAG);
    scene.r#type(0, "Choose a text speed.");
    assert_eq!(scene.said(), texts(&["Choose a text speed."]));
    scene.r#type(0, "Ch");
    assert_eq!(scene.said(), texts(&[]));
    scene.r#type(0, "Choose a text speed.");
    assert_eq!(scene.said(), texts(&[]));
}

#[test]
fn test_text_written_where_none_of_the_windows_own_lines_are_left_is_not_said() {
    let mut scene = described();
    scene.tag(1, SCREEN_TAG);
    scene.write_over("Something else\nentirely.");
    scene.said();
    assert_eq!(scene.said(), texts(&[]));
}

/// Such a sweep says what is new and nothing of what is gone (spike 110).
#[test]
fn test_a_window_a_sweep_of_new_blocks_finds_is_read_beside_those_known() {
    let mut scene = Scene::new(false);
    scene.reader.track(&[OTHER_WINDOW], true);
    assert_eq!(scene.reader.windows(), [WINDOW, OTHER_WINDOW]);
    scene.reader.track(&[WINDOW], true);
    assert_eq!(scene.reader.windows(), [WINDOW, OTHER_WINDOW]);
}

/// Freed since a sweep found it: its pointers lead anywhere, and a read that fails loses the
/// whole poll.
#[test]
fn test_a_window_whose_memory_now_holds_something_else_is_not_read() {
    let mut scene = Scene::new(false);
    scene.message("Hello.");
    scene.memory.u32(WINDOW, &[scene.textbox]);
    scene.r#type(0, "Hello.");
    assert_eq!(scene.said(), texts(&[]));
}

/// `scene.memory.read` patched to note each address read.
struct Recording {
    memory: Rc<FakeMemory>,
    read: RefCell<Vec<i64>>,
}

impl Memory for Recording {
    fn read(&self, address: i64, size: i64) -> crate::py::Result<Vec<u8>> {
        self.read.borrow_mut().push(address);
        self.memory.read(address, size)
    }
}

/// A freed window keeps a window's vtable, and its lines may be memory the GPU has taken:
/// reading them crashed Azahar after Groudon's capture (spikes 168 and 169). The field's box keeps
/// its lines for the next window, so they stay watched until the app ends.
#[test]
fn test_a_freed_window_is_not_read_and_its_lines_only_until_its_app_ends() {
    let mut scene = Scene::new(false);
    // Rust cannot patch a method in place: the reader reads through a recording memory from the
    // start instead, and the test looks only at what it read after the patch would have been made.
    let memory = Rc::new(Recording {
        memory: scene.memory.clone(),
        read: RefCell::new(Vec::new()),
    });
    scene.reader = DialogueReader::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        None,
    );
    scene.reader.track(&[WINDOW], false);
    scene.message("Hello.");
    scene.r#type(0, "Hello.");
    assert_eq!(scene.said(), texts(&["Hello."]));
    scene.memory.u16(WINDOW - BLOCK, 0x4652); // 'FR'
    scene.message("Goodbye.");
    scene.r#type(0, "Goodbye.");
    memory.read.borrow_mut().clear();
    assert_eq!(scene.said(), texts(&[]));
    let read = memory.read.borrow().clone();
    assert!(
        !read
            .iter()
            .any(|&a| (RECORDS..RECORDS + 0x1000).contains(&a))
    );
    assert!(
        read.iter()
            .any(|&a| BOXES.iter().any(|&b| (b..b + 0x1000).contains(&a)))
    );
    scene.reader.forget_hosts();
    memory.read.borrow_mut().clear();
    assert_eq!(scene.said(), texts(&[]));
    let read = memory.read.borrow().clone();
    assert!(
        !read
            .iter()
            .any(|&a| BOXES.iter().any(|&b| (b..b + 0x1000).contains(&a)))
    );
}
