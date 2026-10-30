//! Screen text tests, against fake layouts laid out as spikes 28 to 30 read them.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::screen::{
    self, Pictures, ScreenPoll, ScreenText, ScreenTextReader, after_the_name, in_rows, join,
};
use crate::games::common::{lyt, msgwin, pictures};

const LEFT: i64 = 0;
const CENTRE: i64 = 1;

/// The options screen's static tag processor (spike 28).
const TAG: i64 = 0x00630D80;
const BOTTOM: i64 = 0x08200000;
const TOP: i64 = 0x08300000;

/// Layouts, panes and TextBoxes, and a reader over them.
struct Game {
    memory: Rc<FakeMemory>,
    textbox: i64,
    other: i64,
    window: i64,
    fake: FakeLayouts,
    boxes: Vec<i64>,
    texts: i64,
    reader: ScreenTextReader,
    /// What the last poll found.
    polled: ScreenPoll,
}

/// `set()`: the TextBoxes to leave out, none.
fn none() -> HashSet<i64> {
    HashSet::new()
}

/// What `Game::heard` is compared with.
fn words(heard: &[(&str, bool)]) -> Vec<(String, bool)> {
    heard
        .iter()
        .map(|&(text, changed)| (text.to_string(), changed))
        .collect()
}

/// A reader over the game's layouts that is told what pictures are called.
fn reader_with(game: &Game, pictures: Pictures) -> ScreenTextReader {
    ScreenTextReader::with(
        game.memory.clone(),
        Rc::new(ClassNames::new(game.memory.clone(), CODE)),
        game.fake.layouts.clone(),
        msgwin::CLASSES.iter().map(|c| c.to_string()).collect(),
        None,
        Some(pictures),
        IndexMap::new(),
        HashSet::new(),
    )
}

/// `struct.pack("<f", value)` written at each offset of the pane.
fn write_floats(memory: &FakeMemory, pane: i64, values: &[(i64, f64)]) {
    for &(offset, value) in values {
        memory.write(pane + offset, &(value as f32).to_le_bytes());
    }
}

impl Game {
    fn new() -> Game {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let textbox = classes.add(lyt::TEXTBOX_CLASS);
        let other = classes.add("N3gfl4heap11NwAllocatorE");
        let window = classes.add("N5print6MsgWinE");
        let fake = FakeLayouts::new(&memory, &classes);
        let names = Rc::new(ClassNames::new(memory.clone(), CODE));
        let reader = ScreenTextReader::new(memory.clone(), names, fake.layouts.clone());
        Game {
            memory,
            textbox,
            other,
            window,
            fake,
            boxes: Vec::new(),
            texts: 0x08900000,
            reader,
            polled: ScreenPoll::default(),
        }
    }

    /// A drawn layout on the bottom screen: (work, group).
    fn screen(&self, base: i64) -> (i64, i64) {
        self.fake.screen_with(base, false, true)
    }

    /// A drawn layout on the top screen: (work, group).
    fn top_screen(&self, base: i64) -> (i64, i64) {
        self.fake.screen_with(base, true, true)
    }

    fn text_box(&mut self, at: i64, parent: i64, text: &str) -> i64 {
        self.text_box_tagged(at, parent, text, TAG)
    }

    fn text_box_tagged(&mut self, at: i64, parent: i64, text: &str, tag: i64) -> i64 {
        self.fake.pane_with_vtable(at, parent, Some(self.textbox));
        self.memory.u32(at + lyt::TAG_PROCESSOR, &[tag]);
        self.write(at, text);
        self.boxes.push(at);
        at
    }

    fn write(&mut self, r#box: i64, text: &str) {
        let raw: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
        self.memory.write(self.texts, &raw);
        self.memory.u32(r#box + lyt::TEXT, &[self.texts]);
        self.memory.u16(r#box + lyt::CAPACITY, 0x40);
        self.memory.u16(r#box + lyt::LENGTH, (raw.len() / 2) as i64);
        self.texts += 0x100;
    }

    /// Where a drawn pane is, laid out as spike 33 read the party screen's, centred and anchored
    /// in the centre.
    fn place(&self, r#box: i64, at: (f64, f64), width: f64, glyph: f64) {
        self.place_aligned(r#box, at, width, glyph, CENTRE);
    }

    fn place_aligned(&self, r#box: i64, at: (f64, f64), width: f64, glyph: f64, align: i64) {
        let anchor = CENTRE;
        write_floats(
            &self.memory,
            r#box,
            &[
                (lyt::SCALE_X, 1.0),
                (lyt::X, at.0),
                (lyt::Y, at.1),
                (lyt::WIDTH, width),
            ],
        );
        self.memory
            .write(r#box + lyt::ANCHOR, &[(0x10 | anchor << 2) as u8]);
        write_floats(&self.memory, r#box, &[(lyt::GLYPH_WIDTH, glyph)]);
        self.memory
            .write(r#box + lyt::TEXT_POSITION, &[(3 + align) as u8]);
    }

    fn sweep(&mut self) {
        self.fake.track().unwrap();
        self.reader.track(self.boxes.clone());
    }

    /// `heard()`: what comes on screen, holding words until they read the same twice.
    fn heard(&mut self) -> Vec<(String, bool)> {
        self.heard_with(&none(), true)
    }

    /// `heard(hold=False)`: what comes on screen at once.
    fn heard_now(&mut self) -> Vec<(String, bool)> {
        self.heard_with(&none(), false)
    }

    fn heard_with(&mut self, exclude: &HashSet<i64>, hold: bool) -> Vec<(String, bool)> {
        self.polled = self.reader.poll(exclude, hold).unwrap();
        self.polled
            .texts
            .clone()
            .into_iter()
            .map(|t| (t.text, t.changed))
            .collect()
    }
}

/// An options row on the bottom screen: (game, work, group, the value's TextBox).
fn options() -> (Game, i64, i64, i64) {
    let mut game = Game::new();
    let (work, group) = game.screen(BOTTOM);
    game.text_box(BOTTOM + 0x1000, group, "TEXT SPEED");
    let value = game.text_box(BOTTOM + 0x1200, group, "SLOW");
    game.sweep();
    (game, work, group, value)
}

/// The options row, already heard.
fn settled() -> (Game, i64, i64, i64) {
    let (mut game, work, group, value) = options();
    game.heard();
    game.heard();
    (game, work, group, value)
}

#[test]
fn test_words_on_screen_are_heard_once_they_hold_for_a_poll() {
    let (mut game, ..) = options();
    assert_eq!(game.heard(), words(&[]));
    assert_eq!(
        game.heard(),
        words(&[("TEXT SPEED", false), ("SLOW", false)])
    );
    assert_eq!(game.heard(), words(&[]));
}

#[test]
fn test_without_holding_words_on_screen_are_heard_at_once() {
    let (mut game, ..) = options();
    assert_eq!(
        game.heard_now(),
        words(&[("TEXT SPEED", false), ("SLOW", false)])
    );
}

/// The list window's layout had every pane visible for the 23 s its prompt was not up.
#[test]
fn test_a_layout_not_being_drawn_is_silent_until_it_is() {
    let (mut game, work, _group, _value) = options();
    game.fake.draw_with(work, false, true);
    game.heard();
    assert_eq!(game.heard(), words(&[]));
    game.fake.draw(work);
    game.heard();
    assert_eq!(
        game.heard(),
        words(&[("TEXT SPEED", false), ("SLOW", false)])
    );
}

#[test]
fn test_a_layout_no_longer_alive_is_silent() {
    let (mut game, work, _group, _value) = options();
    game.fake.draw_with(work, true, false);
    assert_eq!(game.heard_now(), words(&[]));
}

/// CHARACTERS had its own flag set; its row's was clear, and it was not on screen.
#[test]
fn test_a_hidden_ancestor_hides_everything_under_it() {
    let (mut game, _work, group, _value) = options();
    game.fake.visible(group, false);
    assert_eq!(game.heard_now(), words(&[]));
}

#[test]
fn test_a_hidden_textbox_is_silent_while_its_siblings_are_heard() {
    let (mut game, _work, _group, value) = options();
    game.fake.visible(value, false);
    assert_eq!(game.heard_now(), words(&[("TEXT SPEED", false)]));
}

/// The summary screen leaves its move detail labels visible at a global alpha of 0.
#[test]
fn test_a_textbox_faded_to_nothing_is_silent() {
    let (mut game, _work, _group, value) = options();
    game.fake.fade(value, 0);
    assert_eq!(game.heard_now(), words(&[("TEXT SPEED", false)]));
}

#[test]
fn test_a_placeholder_no_code_wrote_is_silent() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    game.text_box_tagged(BOTTOM + 0x1000, group, "-------------------", 0);
    game.sweep();
    assert_eq!(game.heard_now(), words(&[]));
}

#[test]
fn test_a_value_written_over_is_heard_as_a_change() {
    let (mut game, _work, _group, value) = settled();
    game.write(value, "FAST");
    assert_eq!(game.heard(), words(&[]));
    assert_eq!(game.heard(), words(&[("FAST", true)]));
}

#[test]
fn test_words_still_being_written_are_heard_only_once_they_hold() {
    let (mut game, _work, _group, value) = settled();
    game.write(value, "NOR");
    assert_eq!(game.heard(), words(&[]));
    game.write(value, "NORMAL");
    assert_eq!(game.heard(), words(&[]));
    assert_eq!(game.heard(), words(&[("NORMAL", true)]));
}

#[test]
fn test_a_screen_opening_marks_its_pieces_as_coming_with_it() {
    let (mut game, ..) = options();
    game.heard();
    let arrived: Vec<(String, bool)> = game
        .reader
        .poll(&none(), true)
        .unwrap()
        .texts
        .into_iter()
        .map(|p| (p.text, p.with_layout))
        .collect();
    assert_eq!(arrived, words(&[("TEXT SPEED", true), ("SLOW", true)]));
}

/// The user chose to hear a screen coming back as a screen opening is heard (2026-09-14).
#[test]
fn test_a_screen_coming_back_is_heard_as_it_came() {
    let (mut game, work, _group, _value) = settled();
    game.fake.draw_with(work, false, true);
    assert_eq!(game.heard(), words(&[]));
    assert_eq!(game.heard(), words(&[]));
    game.fake.draw(work);
    game.heard();
    let arrived: Vec<(String, bool, bool)> = game
        .reader
        .poll(&none(), true)
        .unwrap()
        .texts
        .into_iter()
        .map(|p| (p.text, p.changed, p.with_layout))
        .collect();
    assert_eq!(
        arrived,
        vec![
            ("TEXT SPEED".to_string(), false, true),
            ("SLOW".to_string(), false, true)
        ]
    );
}

/// The options screen's question appears while its rows are up, and is said though the rows
/// are a menu's.
#[test]
fn test_words_that_come_later_in_a_screen_already_up_do_not_come_with_it() {
    let (mut game, _work, group, _value) = settled();
    game.heard();
    game.text_box(
        BOTTOM + 0x1400,
        group,
        "Would you like to make these changes?",
    );
    game.sweep();
    game.heard();
    let arrived: Vec<(String, bool)> = game
        .reader
        .poll(&none(), true)
        .unwrap()
        .texts
        .into_iter()
        .map(|p| (p.text, p.with_layout))
        .collect();
    assert_eq!(
        arrived,
        words(&[("Would you like to make these changes?", false)])
    );
}

#[test]
fn test_words_hidden_and_shown_again_are_heard_again() {
    let (mut game, _work, group, _value) = settled();
    game.fake.visible(group, false);
    assert_eq!(game.heard(), words(&[]));
    assert_eq!(game.heard(), words(&[]));
    game.fake.visible(group, true);
    game.heard();
    assert_eq!(
        game.heard(),
        words(&[("TEXT SPEED", false), ("SLOW", false)])
    );
}

/// Panes read mid-update sat 1,000 units off for a poll, and the battle's move list read "3535"
/// as its pieces came back (battles.ctm replay).
#[test]
fn test_words_missing_for_a_single_poll_are_taken_as_still_there() {
    let (mut game, _work, _group, value) = settled();
    game.fake.visible(value, false);
    assert_eq!(game.heard(), words(&[]));
    let on_screen: Vec<String> = game
        .reader
        .on_screen()
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert_eq!(on_screen, vec!["TEXT SPEED", "SLOW"]);
    game.fake.visible(value, true);
    assert_eq!(game.heard(), words(&[]));
    assert_eq!(game.heard(), words(&[]));
}

/// The Mart's list, wrapping to its end, wrote "Quit" into a row and hid the row's price in
/// one frame, and its cursor read "Quit $ 2,500", the price the row had (movie35).
#[test]
fn test_words_hidden_as_their_row_is_rewritten_are_not_taken_as_still_there() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    let name = game.text_box(BOTTOM + 0x1000, group, "Max Potion");
    let price = game.text_box(BOTTOM + 0x1200, group, "$ 2,500");
    game.place(name, (-55.0, -30.0), 160.0, 12.5);
    game.place(price, (137.0, -30.0), 104.0, 14.0);
    game.sweep();
    game.heard();
    game.heard();
    game.write(name, "Quit");
    game.fake.visible(price, false);
    game.heard();
    let texts: Vec<String> = game
        .reader
        .on_screen()
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert_eq!(texts, ["Quit"]);
}

#[test]
fn test_a_destroyed_textbox_is_silent_and_one_rebuilt_there_is_heard() {
    let (mut game, _work, _group, value) = settled();
    // a destructor leaves the base class's vtable
    game.memory.u32(value, &[game.fake.pane_class]);
    assert_eq!(game.heard(), words(&[]));
    assert_eq!(game.heard(), words(&[]));
    game.memory.u32(value, &[game.textbox]);
    game.heard();
    assert_eq!(game.heard(), words(&[("SLOW", false)]));
}

/// The rolling sweep found Birch's box's lines before the dialogue reader found the box.
#[test]
fn test_a_message_windows_own_lines_are_left_out_before_it_is_found() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    let window = 0x08500000;
    game.memory.u32(window, &[game.window]);
    game.text_box_tagged(
        BOTTOM + 0x1000,
        group,
        "Whew...",
        window + msgwin::TAG_PROCESSOR,
    );
    game.sweep();
    assert_eq!(game.heard_now(), words(&[]));
}

/// They lead the dialogue reader to a window no sweep has found yet (spike 60).
#[test]
fn test_a_message_windows_lines_are_handed_on_to_find_the_window() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    let window = 0x08500000;
    game.memory.u32(window, &[game.window]);
    let line = game.text_box_tagged(
        BOTTOM + 0x1000,
        group,
        "Whew...",
        window + msgwin::TAG_PROCESSOR,
    );
    game.sweep();
    game.heard_now();
    assert_eq!(game.polled.message_lines.clone(), IndexSet::from([line]));
}

/// A torn read left a tag below 0x9c, and taking it for a window's read a negative address.
#[test]
fn test_a_tag_processor_too_small_to_be_a_windows_is_not_followed() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    game.text_box_tagged(BOTTOM + 0x1000, group, "HP", 0x10);
    game.sweep();
    assert_eq!(game.heard_now(), words(&[("HP", false)]));
    assert_eq!(game.polled.message_lines.clone(), IndexSet::new());
}

#[test]
fn test_line_objects_passed_in_are_left_out() {
    let (mut game, _work, _group, value) = options();
    assert_eq!(
        game.heard_with(&HashSet::from([value]), false),
        words(&[("TEXT SPEED", false)])
    );
}

#[test]
fn test_the_top_screen_is_heard_before_the_bottom() {
    let (mut game, ..) = options();
    let (_work, group) = game.top_screen(TOP);
    game.text_box(TOP + 0x1000, group, "TMs & HMs");
    game.sweep();
    assert_eq!(
        game.heard_now(),
        words(&[("TMs & HMs", false), ("TEXT SPEED", false), ("SLOW", false),])
    );
}

/// The menu cursor's layouts are 400 wide and drawn on the bottom screen (spike 40).
#[test]
fn test_the_screen_a_layout_was_drawn_to_beats_its_width() {
    let (mut game, ..) = options();
    let (work, group) = game.top_screen(TOP);
    game.text_box(TOP + 0x1000, group, "BAG");
    game.fake.project(work, 320.0);
    game.sweep();
    assert_eq!(
        game.heard_now(),
        words(&[("TEXT SPEED", false), ("SLOW", false), ("BAG", false)])
    );
}

#[test]
fn test_control_codes_are_left_out_of_the_words() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    game.text_box(
        BOTTOM + 0x1000,
        group,
        "\u{10}\u{2}＀\u{2}Torchic\u{10}\u{2}＀\u{0}",
    );
    game.sweep();
    assert_eq!(game.heard_now(), words(&[("Torchic", false)]));
}

#[test]
fn test_a_chain_that_leaves_the_pane_classes_is_not_believed() {
    let (mut game, _work, group, _value) = options();
    game.memory.u32(group, &[game.other]);
    assert_eq!(game.heard_now(), words(&[]));
}

#[test]
fn test_a_root_no_layout_owns_is_silent() {
    let mut game = Game::new();
    let root = 0x08400000;
    game.fake.pane(root, 0);
    game.text_box(root + 0x1000, root, "Orphan");
    game.sweep();
    assert_eq!(game.heard_now(), words(&[]));
}

#[test]
fn test_a_work_whose_drawinfo_is_something_else_is_not_believed() {
    let (mut game, work, _group, _value) = options();
    game.memory.u32(work + lyt::WORK_DRAWINFO, &[game.other]);
    game.sweep();
    assert_eq!(game.heard_now(), words(&[]));
}

/// The party screen's first slot as spike 33 measured it: seven TextBoxes.
fn party_slot(game: &mut Game, group: i64) {
    let pieces = [
        ("Torchic", (-70.0, 88.0), 100.0, 12.5, LEFT),
        ("\u{e08e}", (-13.9, 88.0), 14.0, 14.0, LEFT),
        ("L", (-106.0, 102.0), 20.0, 16.0, CENTRE),
        ("5", (-81.0, 102.0), 30.0, 16.0, LEFT),
        ("19", (-115.1, 63.0), 30.0, 16.0, CENTRE),
        ("/", (-95.0, 63.0), 10.0, 16.0, CENTRE),
        ("19", (-75.0, 63.0), 30.0, 16.0, CENTRE),
    ];
    for (i, (text, at, width, glyph, align)) in pieces.into_iter().enumerate() {
        let r#box = game.text_box(BOTTOM + 0x1000 + i as i64 * 0x200, group, text);
        game.place_aligned(r#box, at, width, glyph, align);
    }
    game.sweep();
}

/// The user: "Torchic male L5 19/19 is fine as is" (2026-09-14).
#[test]
fn test_pieces_drawn_side_by_side_are_joined_as_drawn() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    party_slot(&mut game, group);
    assert_eq!(
        join(&game.reader.poll(&none(), false).unwrap().texts, false),
        "Torchic \u{e08e} L5 19/19"
    );
}

#[test]
fn test_pauses_go_only_between_pieces_that_are_apart() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    party_slot(&mut game, group);
    assert_eq!(
        join(&game.reader.poll(&none(), false).unwrap().texts, true),
        "Torchic, \u{e08e}, L5, 19/19"
    );
}

/// The battle's Bag draws an item's sign and count in touching boxes, the words estimated
/// apart: "Poké Ball, times, 10" under pauses (movie3, spike 196).
#[test]
fn test_a_sign_is_joined_to_the_number_after_it() {
    let name = ScreenText::placed(1, "Poké Ball", false, -108.0, -52.0, 92.0, 12.5);
    let sign = ScreenText::placed(3, TIMES, false, -98.0, -90.0, 73.0, 16.0);
    let count = ScreenText::placed(2, "10", false, -74.0, -60.0, 73.0, 14.0);
    assert_eq!(
        join(&[name, sign, count], true),
        format!("Poké Ball, {TIMES}10")
    );
}

/// The summary's HP hangs every piece from its root, each a panel of its own, and its "/",
/// the layout file's and never written, was left out: "HP 30 30" (spike 148, Orion 2026-09-30).
#[test]
fn test_the_designer_s_slash_joins_numbers_hanging_straight_from_the_root() {
    let mut game = Game::new();
    let (_work, root) = game.fake.screen_with(BOTTOM, false, false);
    for (i, (text, x, glyph, tag)) in [
        ("HP", 36.0, 12.5, TAG),
        ("/", 130.0, 14.0, 0),
        ("30", 110.0, 14.0, TAG),
        ("30", 150.0, 14.0, TAG),
    ]
    .into_iter()
    .enumerate()
    {
        let r#box = game.text_box_tagged(BOTTOM + 0x1000 + i as i64 * 0x200, root, text, tag);
        game.place(
            r#box,
            (x, 85.0),
            if text != "HP" { 14.0 } else { 12.0 },
            glyph,
        );
    }
    game.sweep();
    assert_eq!(
        join(
            &in_rows(game.reader.poll(&none(), false).unwrap().texts),
            false
        ),
        "HP 30/30"
    );
}

/// The summary's trainer panel where spike 148 measured it: labels from x -195, values drawn
/// to -30, Exp. Points' and To Next Lv.'s a row under them, the ability's description under it.
fn summary_panel() -> Vec<ScreenText> {
    [
        ("OT", -195.0, -182.0, 20.0, 12.5),
        ("Raiden", -68.0, -30.0, 20.0, 12.5),
        ("ID No.", -195.0, -158.0, 0.0, 12.5),
        ("16285", -65.0, -30.0, 0.0, 14.0),
        ("Exp. Points", -195.0, -126.0, -10.0, 12.5),
        ("597", -51.0, -30.0, -30.0, 14.0),
        ("To Next Lv.", -195.0, -126.0, -50.0, 12.5),
        ("145", -51.0, -30.0, -70.0, 14.0),
        ("Blaze", -20.0, 11.0, -50.0, 12.5),
        (
            "Powers up Fire-type moves when the Pokémon is in trouble.",
            -20.0,
            190.0,
            -70.0,
            12.5,
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(r#box, (text, left, right, row, glyph))| {
        ScreenText::placed(r#box as i64, text, false, left, right, row, glyph)
    })
    .collect()
}

/// Orion, 2026-09-30: "OT: Raiden, ID No.: 16285", to hear where one thing ends.
#[test]
fn test_with_pauses_a_label_is_followed_by_a_colon() {
    assert_eq!(
        join(&summary_panel(), true),
        "OT: Raiden, ID No.: 16285, Exp. Points: 597, To Next Lv.: 145, \
         Blaze, Powers up Fire-type moves when the Pokémon is in trouble."
    );
    assert_eq!(
        join(&summary_panel(), false),
        "OT Raiden ID No. 16285 Exp. Points 597 To Next Lv. 145 \
         Blaze Powers up Fire-type moves when the Pokémon is in trouble."
    );
}

/// Pieces where spike 148 measured them: words, left, right, row, glyph.
fn measured(pieces: &[(&str, f64, f64, f64, f64)]) -> Vec<ScreenText> {
    pieces
        .iter()
        .enumerate()
        .map(|(r#box, &(text, left, right, row, glyph))| {
            ScreenText::placed(r#box as i64, text, false, left, right, row, glyph)
        })
        .collect()
}

/// The DexNav's page, where pauses read "Found Pokémon: Search Level, 64, Name, Zigzagoon"
/// and "Pokédex: SEARCH": its labels sit over their values, and its title and buttons are where
/// a label could be (spike 195). Its words say which is which.
#[test]
fn test_the_games_own_labels_are_labels_wherever_they_are_drawn() {
    let measured_ = measured(&[
        ("Found Pokémon", -155.0, -95.0, 104.0, 12.5),
        ("Search Level", 40.0, 100.0, 104.0, 12.5),
        ("64", 120.0, 132.0, 104.0, 14.0),
        ("Name", -155.0, -130.0, 86.0, 12.5),
        ("Zigzagoon", -155.0, -100.0, 68.0, 12.5),
        ("Type", 60.0, 85.0, 86.0, 12.5),
        ("Normal", 60.0, 95.0, 68.0, 12.5),
        ("Pokédex", -150.0, -110.0, -70.0, 12.5),
        ("SEARCH", -20.0, 20.0, -70.0, 12.5),
    ]);
    let [
        title,
        level,
        number,
        name,
        species,
        kind,
        normal,
        dex,
        search,
    ]: [ScreenText; 9] = measured_.try_into().unwrap();
    let page = vec![
        ScreenText {
            plain: true,
            ..title
        },
        ScreenText {
            label: true,
            ..level
        },
        number.clone(),
        ScreenText {
            label: true,
            ..name
        },
        species,
        ScreenText {
            label: true,
            ..kind
        },
        normal,
        ScreenText { plain: true, ..dex },
        ScreenText {
            plain: true,
            ..search
        },
    ];
    assert_eq!(
        join(&page, true),
        "Found Pokémon, Search Level: 64, Name: Zigzagoon, Type: Normal, Pokédex, SEARCH"
    );
    assert_eq!(
        join(&page, false),
        "Found Pokémon Search Level 64 Name Zigzagoon Type Normal Pokédex SEARCH"
    );
    // A label touching its value is one still: "Search Level5" was said as one word.
    let touching = ScreenText {
        text: "5".to_string(),
        left: 101.0,
        right: 108.0,
        ..number
    };
    assert_eq!(
        join(&[page[1].clone(), touching.clone()], true),
        "Search Level: 5"
    );
    assert_eq!(join(&[page[1].clone(), touching], false), "Search Level 5");
}

/// "Potential: 2 stars", "First move: tackle, egg move" under pauses, and spaces otherwise
/// (Orion, 2026-10-06): the stars are three pictures, filled or not, and the red mark is drawn
/// beside First Move's label, built before it (spike 195).
#[test]
fn test_the_dexnav_s_stars_are_counted_and_its_mark_said_after_what_it_marks() {
    let pieces = measured(&[
        ("Level", -155.0, -125.0, 60.0, 12.5),
        ("Lv.", -145.0, -128.0, 40.0, 12.5),
        ("3", -126.0, -119.0, 40.0, 14.0),
        ("Potential", 60.0, 110.0, 40.0, 12.5),
        ("First Move", -140.0, -80.0, 10.0, 12.5),
        ("Helping Hand", -155.0, -85.0, -10.0, 12.5),
        ("Held Item", 50.0, 100.0, 10.0, 12.5),
        ("Revive", 30.0, 65.0, -10.0, 12.5),
    ]);
    let groups = [0x100, 0x100, 0x100, 0x200, 0x300, 0x300, 0x400, 0x400];
    let labels: IndexMap<String, String> = [
        ("Level", pictures::HIGHER_LEVEL),
        ("Potential", ""),
        ("First Move", pictures::EGG_MOVE),
        ("Held Item", ""),
    ]
    .into_iter()
    .map(|(label, mark)| (label.to_string(), mark.to_string()))
    .collect();
    let words: Vec<ScreenText> = pieces
        .into_iter()
        .enumerate()
        .map(|(i, piece)| ScreenText {
            r#box: 0x40 + 4 * i as i64,
            parent: groups[i],
            label: labels.contains_key(&piece.text),
            ..piece
        })
        .collect();
    let stars: IndexMap<i64, ScreenText> = [
        (0x90, pictures::STAR, 77.0),
        (0x94, pictures::STAR, 91.0),
        (0x98, pictures::NO_STAR, 105.0),
    ]
    .into_iter()
    .map(|(r#box, sign, left)| {
        (
            r#box,
            ScreenText {
                parent: 0x200,
                picture: true,
                ..ScreenText::placed(r#box, sign, false, left, left + 16.0, 25.0, 16.0)
            },
        )
    })
    .collect();
    let mark = ScreenText {
        parent: 0x300,
        picture: true,
        ..ScreenText::placed(0x30, pictures::SPECIAL, false, -160.0, -144.0, 10.0, 16.0)
    };
    let mut found: IndexMap<i64, ScreenText> = words
        .into_iter()
        .map(|piece| (piece.r#box, piece))
        .collect();
    found.extend(screen::counted_stars(&stars).unwrap());
    found.insert(mark.r#box, mark);
    let mut said: Vec<ScreenText> = screen::by_their_labels(&found, &labels)
        .into_values()
        .collect();
    said.sort_by_key(|piece| piece.r#box);
    assert_eq!(
        join(&said, true),
        "Level: Lv.3, Potential: 2 stars, First Move: Helping Hand, egg move, Held Item: Revive"
    );
    assert_eq!(
        join(&said, false),
        "Level Lv.3 Potential 2 stars First Move Helping Hand egg move Held Item Revive"
    );
}

/// A contest draws a move's appeal as hearts under "Appeal", four for Hold Back (spike 196).
#[test]
fn test_a_contest_moves_hearts_are_counted() {
    let hearts: IndexMap<i64, ScreenText> =
        [(0x90, -30.0), (0x94, -20.0), (0x98, -10.0), (0x9C, 0.0)]
            .into_iter()
            .map(|(r#box, left)| {
                (
                    r#box,
                    ScreenText {
                        parent: 0x200,
                        picture: true,
                        ..ScreenText::placed(
                            r#box,
                            pictures::HEART,
                            false,
                            left,
                            left + 10.0,
                            -65.0,
                            10.0,
                        )
                    },
                )
            })
            .collect();
    let counted: Vec<ScreenText> = screen::counted_stars(&hearts)
        .unwrap()
        .into_values()
        .collect();
    let [counted] = counted.try_into().unwrap();
    assert_eq!(
        (counted.text.as_str(), counted.left, counted.right),
        ("4 hearts", -30.0, 10.0)
    );
}

/// or_amie_stars: Puzzle Wizard's five stars landed in the five slots, Easy's half star under
/// the fifth; a rating of four and a half would show the half star alone there.
#[test]
fn test_a_ratings_half_star_counts_only_where_no_whole_one_covers_it() {
    let slots = [-66.0, -38.0, -10.0, 18.0, 46.0];
    let rating = |whole: usize, half_at: Option<usize>| -> String {
        let picture = |r#box: i64, text: &str, left: f64| ScreenText {
            parent: 0x200,
            picture: true,
            ..ScreenText::placed(r#box, text, false, left, left + 20.0, -98.0, 20.0)
        };
        let mut drawn: Vec<ScreenText> = slots[..whole]
            .iter()
            .enumerate()
            .map(|(i, &left)| picture(0x200 + 4 * i as i64, pictures::STAR, left))
            .collect();
        if let Some(at) = half_at {
            drawn.push(picture(0x300, pictures::HALF_STAR, slots[at]));
        }
        let counted: Vec<ScreenText> = screen::counted_stars(
            &drawn
                .into_iter()
                .map(|piece| (piece.r#box, piece))
                .collect(),
        )
        .unwrap()
        .into_values()
        .collect();
        let [counted] = counted.try_into().unwrap();
        counted.text
    };

    assert_eq!(rating(5, Some(4)), "5 stars");
    assert_eq!(rating(4, Some(4)), "4 and a half stars");
    assert_eq!(rating(0, Some(0)), "half a star");
}

/// or_amie_stars: an empty slot's corners were 00 00 00 3c, an earned star's ff ff ff ff; the
/// empty ones are left unsaid.
#[test]
fn test_a_star_drawn_as_a_faint_black_shadow_is_not_earned() {
    assert!(screen::shadow(&[0, 0, 0, 0x3C].repeat(4)));
    assert!(!screen::shadow(&[0xFF; 16]));
}

/// The Pokéblock Case's star draws "TOUGH" touching Electro Ball's icon on its row, and it
/// read "Coolness/Toughness" (movie33).
#[test]
fn test_pictures_of_two_groups_side_by_side_are_two() {
    let r#move = ScreenText {
        parent: 7,
        picture: true,
        ..ScreenText::placed(1, "Coolness", false, -60.0, -10.0, 50.0, 20.0)
    };
    let star = ScreenText {
        parent: 3,
        picture: true,
        ..ScreenText::placed(2, "Toughness", false, -5.0, 45.0, 50.0, 20.0)
    };
    let found = screen::side_by_side(&IndexMap::from([(1, r#move), (2, star)]));
    let mut texts: Vec<String> = found.into_values().map(|piece| piece.text).collect();
    texts.sort();
    assert_eq!(texts, ["Coolness", "Toughness"]);
}

/// A summary hangs Appeal's hearts and Jamming's in one window (movie33).
#[test]
fn test_hearts_on_two_rows_of_one_group_are_counted_apart() {
    let hearts: IndexMap<i64, ScreenText> = [
        (0x90, -100.0, -5.0),
        (0x94, -90.0, -5.0),
        (0x98, -100.0, -28.0),
    ]
    .into_iter()
    .map(|(r#box, left, row)| {
        (
            r#box,
            ScreenText {
                parent: 0x200,
                picture: true,
                ..ScreenText::placed(r#box, pictures::HEART, false, left, left + 10.0, row, 10.0)
            },
        )
    })
    .collect();
    let mut texts: Vec<String> = screen::counted_stars(&hearts)
        .unwrap()
        .into_values()
        .map(|piece| piece.text)
        .collect();
    texts.sort();
    assert_eq!(texts, ["1 heart", "2 hearts"]);
}

/// The results draw each entrant's place as a picture left of "Raiden & Metagross" (spike
/// 196): "Raiden & Metagross: third place" under pauses (Orion, 2026-10-09).
#[test]
fn test_a_contests_place_is_read_after_its_entrant() {
    let names = ScreenText {
        parent: 7,
        ..ScreenText::placed(0x20, "Raiden & Metagross", false, -75.0, 37.5, -45.0, 12.5)
    };
    let place = ScreenText {
        parent: 7,
        picture: true,
        ..ScreenText::placed(
            0x10,
            own(pictures::place("three").unwrap(), &[]),
            false,
            -154.0,
            -124.0,
            -55.0,
            30.0,
        )
    };
    let found: IndexMap<i64, ScreenText> = IndexMap::from([(0x10, place), (0x20, names)]);
    let mut placed: Vec<ScreenText> = screen::places_last(&found).into_values().collect();
    placed.sort_by_key(|piece| piece.r#box);
    let said = in_rows(placed);
    assert_eq!(join(&said, true), "Raiden & Metagross: third place");
    assert_eq!(join(&said, false), "Raiden & Metagross third place");
}

/// A status move's power is a dash, and under pauses it read "Power, —" (movie32).
#[test]
fn test_a_dash_is_a_labels_value() {
    let power = ScreenText::placed(1, "Power", false, -210.0, -180.0, -10.0, 12.5);
    let dash = ScreenText::placed(2, "—", false, -122.0, -118.0, -10.0, 12.5);
    assert_eq!(join(&[power, dash], true), "Power: —");
}

/// The main menu's CONTINUE panel (movie11, 2026-10-01): "Badges:, 0" before.
#[test]
fn test_a_label_with_a_colon_of_its_own_is_followed_by_a_space() {
    let panel = measured(&[
        ("CONTINUE", -136.0, -86.0, 89.0, 12.5),
        ("Raiden", -60.0, -23.0, 66.0, 12.5),
        ("Petalburg City", -60.0, 28.0, 46.0, 12.5),
        ("Badges:", -134.0, -90.0, 20.0, 12.5),
        ("0", -63.0, -56.0, 20.0, 14.0),
        ("Pokédex:", -14.0, 36.0, 20.0, 12.5),
        ("7", 68.0, 75.0, 20.0, 14.0),
    ]);
    assert_eq!(
        join(&panel, true),
        "CONTINUE: Raiden, Petalburg City, Badges: 0, Pokédex: 7"
    );
    // The trainer card: what follows a label's own colon is a value, and labels nothing.
    let card = measured(&[
        ("Name:", -190.0, -150.0, 60.0, 12.5),
        ("Raiden", -100.0, -60.0, 60.0, 12.5),
        ("ID No.:", 0.0, 40.0, 60.0, 12.5),
        ("16285", 90.0, 130.0, 60.0, 14.0),
    ]);
    assert_eq!(join(&card, true), "Name: Raiden, ID No.: 16285");
}

/// The Box's summary panel (movie11, 2026-10-01): "ITEM: Potion: Sand Attack" and "Ember:
/// Sand Attack" before; ABILITY's value is drawn a little above it.
#[test]
fn test_a_value_is_no_label_and_a_list_s_next_row_is_no_value() {
    let panel = measured(&[
        ("NATURE", -190.0, -153.0, -70.0, 12.5),
        ("Bold", -110.0, -85.0, -70.0, 12.5),
        ("ABILITY", -190.0, -146.0, -90.0, 12.5),
        ("Run Away", -110.0, -60.0, -80.0, 12.5),
        ("ITEM", -190.0, -165.0, -100.0, 12.5),
        ("Potion", -110.0, -72.0, -100.0, 12.5),
        ("Sand Attack", 50.0, 119.0, -100.0, 12.5),
    ]);
    assert_eq!(
        join(&panel, true),
        "NATURE: Bold, ABILITY: Run Away, ITEM: Potion, Sand Attack"
    );
    let moves = measured(&[
        ("Ember", 50.0, 81.0, -80.0, 12.5),
        ("Sand Attack", 50.0, 119.0, -100.0, 12.5),
    ]);
    assert_eq!(join(&moves, true), "Ember, Sand Attack");
}

/// Switching Pokémon changed the ability and a move together, "Blaze: Ember"; a type's icon,
/// "Fire", came before MOVES LEARNED (movie11, 2026-10-01; the icon's place is made up, as
/// spike 148 logs no picture's place).
#[test]
fn test_changed_words_and_a_picture_s_are_no_label() {
    let [blaze, ember] = measured(&[
        ("Blaze", -110.0, -79.0, -80.0, 12.5),
        ("Ember", 50.0, 81.0, -80.0, 12.5),
    ])
    .try_into()
    .unwrap();
    assert_eq!(
        join(
            &[
                ScreenText {
                    changed: true,
                    ..blaze
                },
                ember
            ],
            true
        ),
        "Blaze, Ember"
    );
    let [fire, heading] = measured(&[
        ("Fire", -20.0, 0.0, -10.0, 12.5),
        ("MOVES LEARNED", 40.0, 121.0, -20.0, 12.5),
    ])
    .try_into()
    .unwrap();
    assert_eq!(
        join(
            &[
                ScreenText {
                    picture: true,
                    ..fire
                },
                heading
            ],
            true
        ),
        "Fire, MOVES LEARNED"
    );
}

/// The Box's summary draws the gender sign, a glyph of the game's own, just under "Lv.5":
/// "Umbra, : Lv.5" once a value could sit above its label (movie11, 2026-10-01).
#[test]
fn test_a_sign_is_no_label() {
    let panel = measured(&[
        ("Umbra", -170.0, -139.0, 110.0, 12.5),
        ("\u{e08e}", -70.0, -63.0, 110.0, 14.0),
        ("Lv.", -34.0, -15.0, 120.0, 12.5),
        ("5", -15.0, -8.0, 120.0, 14.0),
    ]);
    assert_eq!(join(&panel, true), "Umbra, \u{e08e}, Lv.5");
}

/// A move's "PP" box touches its "35" box; the words inside them do not.
#[test]
fn test_a_label_beside_its_value_keeps_its_space() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    for (i, (text, x, width, glyph)) in [
        ("PP", 60.0, 35.0, 12.5),
        ("35", 88.0, 20.0, 14.0),
        ("/", 103.0, 10.0, 14.0),
        ("35", 118.0, 20.0, 14.0),
    ]
    .into_iter()
    .enumerate()
    {
        let r#box = game.text_box(BOTTOM + 0x1000 + i as i64 * 0x200, group, text);
        game.place(r#box, (x, 80.0), width, glyph);
    }
    game.sweep();
    assert_eq!(
        join(&game.reader.poll(&none(), false).unwrap().texts, false),
        "PP 35/35"
    );
}

#[test]
fn test_a_layout_is_settling_while_its_words_have_not_held_for_a_poll() {
    let (mut game, _work, _group, value) = options();
    game.heard();
    assert_eq!(
        game.polled.settling.clone(),
        IndexSet::from([BOTTOM + 0x400])
    );
    game.heard();
    assert_eq!(game.polled.settling.clone(), IndexSet::new());
    game.write(value, "FAST");
    game.heard();
    assert_eq!(
        game.polled.settling.clone(),
        IndexSet::from([BOTTOM + 0x400])
    );
}

/// AreaNav's TextBoxes are in the linear heap, swept a slice at a time; its layout work is in
/// the heap, and its tree leads to them (spike 49).
#[test]
fn test_words_in_a_layout_are_found_from_its_tree_without_a_sweep_finding_them() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    game.text_box(BOTTOM + 0x1000, group, "Littleroot Town");
    game.fake.track().unwrap();
    game.reader.track([]);
    assert_eq!(game.heard_now(), words(&[("Littleroot Town", false)]));
}

/// Closing a screen opened from the X menu builds the field's layouts again at their works.
#[test]
fn test_a_layout_built_again_where_it_was_is_read_as_soon_as_it_is_revived() {
    let mut game = Game::new();
    let (work, group) = game.screen(BOTTOM);
    game.text_box(BOTTOM + 0x1000, group, "Littleroot Town");
    game.fake.track().unwrap();
    game.reader.track([]);
    game.heard_now();
    game.memory.u32(work + lyt::WORK_DRAWINFO, &[0]);
    game.fake.track().unwrap();
    assert_eq!(game.heard_now(), words(&[]));
    assert_eq!(game.heard_now(), words(&[]));
    assert!(!game.fake.layouts.revive().unwrap());
    game.memory
        .u32(work + lyt::WORK_DRAWINFO, &[game.fake.drawinfo]);
    assert!(game.fake.layouts.revive().unwrap());
    assert_eq!(game.heard_now(), words(&[("Littleroot Town", false)]));
}

/// A layout built again where it was is the one that was there, and a menu stays a menu: the
/// battle's move list, forgotten while it was gone, was read out whole the moment it showed
/// again (movie4, 2026-09-26).
#[test]
fn test_a_layout_gone_is_buried_until_it_is_built_again_where_it_was() {
    let mut game = Game::new();
    let (work, group) = game.screen(BOTTOM);
    game.text_box(BOTTOM + 0x1000, group, "Tackle");
    game.fake.track().unwrap();
    let roots = game.fake.layouts.roots();
    assert_eq!(roots.len(), 1);
    let root = roots[0];
    assert_eq!(game.fake.layouts.buried(), IndexSet::new());
    game.memory.u32(work + lyt::WORK_DRAWINFO, &[0]);
    game.fake.track().unwrap();
    assert_eq!(game.fake.layouts.buried(), IndexSet::from([root]));
    game.memory
        .u32(work + lyt::WORK_DRAWINFO, &[game.fake.drawinfo]);
    assert!(game.fake.layouts.revive().unwrap());
    assert_eq!(game.fake.layouts.buried(), IndexSet::new());
}

/// The save prompt's list layout, built at an old work, had "YES NO" read before the sweep
/// found the list that owns them (menus.ctm replay, spike 49).
#[test]
fn test_a_different_layout_built_at_a_freed_work_waits_for_a_sweep() {
    let mut game = Game::new();
    let (work, group) = game.screen(BOTTOM);
    game.text_box(BOTTOM + 0x1000, group, "Littleroot Town");
    game.fake.track().unwrap();
    game.reader.track([]);
    game.heard_now();
    game.memory.u32(work + lyt::WORK_DRAWINFO, &[0]);
    game.fake.track().unwrap();
    game.heard_now();
    game.memory.u32(
        work + lyt::WORK_LAYOUT,
        &[BOTTOM + 0x3000, game.fake.drawinfo],
    );
    game.memory.u32(BOTTOM + 0x3000, &[game.fake.layout_class]);
    game.memory
        .u32(BOTTOM + 0x3000 + lyt::LAYOUT_ROOT, &[BOTTOM + 0x3400]);
    game.fake.pane(BOTTOM + 0x3400, 0);
    game.text_box(BOTTOM + 0x3800, BOTTOM + 0x3400, "YES");
    assert!(!game.fake.layouts.revive().unwrap());
    assert_eq!(game.heard_now(), words(&[]));
}

#[test]
fn test_words_on_screen_for_the_first_time_are_not_returning() {
    let (mut game, ..) = options();
    let returned: Vec<bool> = game
        .reader
        .poll(&none(), false)
        .unwrap()
        .texts
        .into_iter()
        .map(|p| p.returned)
        .collect();
    assert_eq!(returned, vec![false, false]);
}

#[test]
fn test_words_coming_back_as_they_left_are_marked_as_returning() {
    let (mut game, _work, _group, value) = settled();
    game.fake.visible(value, false);
    game.heard();
    game.heard();
    game.fake.visible(value, true);
    game.heard();
    let [back]: [ScreenText; 1] = game
        .reader
        .poll(&none(), true)
        .unwrap()
        .texts
        .try_into()
        .unwrap();
    assert_eq!((back.text.as_str(), back.returned), ("SLOW", true));
}

#[test]
fn test_words_changed_while_off_screen_are_not_returning() {
    let (mut game, _work, _group, value) = settled();
    game.fake.visible(value, false);
    game.heard();
    game.heard();
    game.write(value, "FAST");
    game.fake.visible(value, true);
    game.heard();
    let [back]: [ScreenText; 1] = game
        .reader
        .poll(&none(), true)
        .unwrap()
        .texts
        .try_into()
        .unwrap();
    assert_eq!((back.text.as_str(), back.returned), ("FAST", false));
}

/// The X menu, closed, keeps its labels visible and drawn below the bottom screen (spike 48).
#[test]
fn test_words_placed_outside_their_screen_are_silent_until_they_come_onto_it() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    let label = game.text_box(BOTTOM + 0x1000, group, "POKÉMON");
    game.place(label, (-64.0, -189.0), 100.0, 16.0);
    let banner = game.text_box(BOTTOM + 0x1200, group, "Littleroot Town");
    game.place(banner, (0.0, 100.0), 150.0, 16.0);
    game.sweep();
    assert_eq!(game.heard_now(), words(&[("Littleroot Town", false)]));
    game.place(label, (-64.0, 61.0), 100.0, 16.0);
    assert_eq!(game.heard_now(), words(&[("POKÉMON", false)]));
}

/// A letter deleted from a name only takes a word off the screen (the user, 2026-09-18).
#[test]
fn test_words_that_leave_the_screen_are_reported_as_they_last_read() {
    let (mut game, _work, _group, value) = settled();
    assert_eq!(game.polled.left.clone(), vec![]);
    game.fake.visible(value, false);
    game.heard();
    game.heard();
    let left: Vec<(String, i64)> = game
        .polled
        .left
        .clone()
        .into_iter()
        .map(|piece| (piece.text, piece.r#box))
        .collect();
    assert_eq!(left, vec![("SLOW".to_string(), value)]);
    game.heard();
    assert_eq!(game.polled.left.clone(), vec![]);
}

/// The language screen's list moves its rows rather than rewriting them: moving the cursor to
/// Spanish brought "Japanese" up onto the top screen, and it was said as newly shown beside the
/// cursor's "Spanish" (opening.ctm, the user, 2026-09-17; spike 68).
#[test]
fn test_words_moving_onto_their_screen_unchanged_have_scrolled_in() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    let row = game.text_box(BOTTOM + 0x1000, group, "Japanese");
    game.place(row, (-120.0, -160.0), 100.0, 12.5);
    game.sweep();
    assert_eq!(game.heard_now(), words(&[]));
    game.place(row, (-120.0, -100.0), 100.0, 12.5);
    let [piece]: [ScreenText; 1] = game
        .reader
        .poll(&none(), false)
        .unwrap()
        .texts
        .try_into()
        .unwrap();
    assert_eq!((piece.text.as_str(), piece.scrolled), ("Japanese", true));
}

#[test]
fn test_words_shown_where_they_are_have_not_scrolled_in() {
    let (mut game, _work, _group, value) = settled();
    game.fake.visible(value, false);
    game.heard();
    game.heard();
    game.fake.visible(value, true);
    let [back]: [ScreenText; 1] = game
        .reader
        .poll(&none(), false)
        .unwrap()
        .texts
        .try_into()
        .unwrap();
    assert_eq!((back.text.as_str(), back.scrolled), ("SLOW", false));
}

#[test]
fn test_words_changed_while_below_their_screen_have_not_scrolled_in() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    let row = game.text_box(BOTTOM + 0x1000, group, "Japanese");
    game.place(row, (-120.0, -160.0), 100.0, 12.5);
    game.sweep();
    game.heard_now();
    game.write(row, "Korean");
    game.place(row, (-120.0, -100.0), 100.0, 12.5);
    let [piece]: [ScreenText; 1] = game
        .reader
        .poll(&none(), false)
        .unwrap()
        .texts
        .try_into()
        .unwrap();
    assert_eq!((piece.text.as_str(), piece.scrolled), ("Korean", false));
}

/// The battle's top-screen roots come out at (0, 1000), with the view offset in them, and its
/// level up's "HP + 2" at y 1074: judged from (0, 0) every one was left out (spikes 47 and 48).
#[test]
fn test_words_are_placed_from_their_layouts_root() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    write_floats(&game.memory, BOTTOM + 0x400, &[(lyt::Y, 1000.0)]);
    let stat = game.text_box(BOTTOM + 0x1000, group, "HP + 2");
    game.place(stat, (59.0, 1074.0), 100.0, 16.0);
    let parked = game.text_box(BOTTOM + 0x1200, group, "POKÉMON");
    game.place(parked, (-64.0, 811.0), 100.0, 16.0);
    game.sweep();
    assert_eq!(game.heard_now(), words(&[("HP + 2", false)]));
}

/// The battle's top-screen works read the bottom screen's projection; their roots come out at
/// (0, 1000), and the level up's values at x 168 were taken as past the bottom screen's side.
#[test]
fn test_a_root_far_from_the_middle_is_on_the_top_screen_whatever_its_projection() {
    let mut game = Game::new();
    let (work, group) = game.screen(BOTTOM);
    game.fake.project(work, 320.0);
    write_floats(&game.memory, BOTTOM + 0x400, &[(lyt::Y, 1000.0)]);
    let value = game.text_box(BOTTOM + 0x1000, group, "22");
    game.place(value, (180.0, 1074.0), 30.0, 16.0);
    game.sweep();
    assert_eq!(game.heard_now(), words(&[("22", false)]));
    let top: Vec<bool> = game.reader.on_screen().into_iter().map(|p| p.top).collect();
    assert_eq!(top, vec![true]);
}

#[test]
fn test_words_past_the_side_of_their_screen_are_silent() {
    let mut game = Game::new();
    let (_work, group) = game.screen(BOTTOM);
    let beside = game.text_box(BOTTOM + 0x1000, group, "Next page");
    game.place(beside, (260.0, 0.0), 80.0, 16.0);
    game.sweep();
    assert_eq!(game.heard_now(), words(&[]));
}

#[test]
fn test_pieces_whose_place_is_unknown_are_kept_apart() {
    assert_eq!(
        join(
            &[
                ScreenText::new(1, "L", false),
                ScreenText::new(2, "5", false)
            ],
            false
        ),
        "L 5"
    );
}

#[test]
fn test_words_on_screen_stay_read_when_a_later_sweep_misses_them() {
    let (mut game, _work, _group, value) = settled();
    game.fake.layouts.track([]).unwrap();
    game.reader.track([]);
    assert_eq!(game.heard(), words(&[]));
    game.write(value, "FAST");
    game.heard();
    assert_eq!(game.heard(), words(&[("FAST", true)]));
}

/// Its block is swept as the game takes it, and the layout is built into it after: the level
/// up's panel was found that way and never read (movie2, 2026-09-26).
#[test]
fn test_a_work_found_before_its_layout_is_built_is_taken_in_once_it_is() {
    let mut game = Game::new();
    let (work, group) = game.screen(BOTTOM);
    game.text_box(BOTTOM + 0x1000, group, "HP + 3");
    let layout = game.memory.read(work + lyt::WORK_LAYOUT, 4).unwrap();
    game.memory.u32(work + lyt::WORK_LAYOUT, &[0]);
    game.fake.layouts.track([work]).unwrap();
    assert_eq!(game.fake.layouts.works(), IndexMap::new());
    game.memory.write(work + lyt::WORK_LAYOUT, &layout);
    assert!(game.fake.layouts.revive().unwrap());
    assert_eq!(
        game.fake
            .layouts
            .works()
            .keys()
            .copied()
            .collect::<Vec<i64>>(),
        vec![work]
    );
}

/// A pool's slot is built into again and again: the level up's panel, built into a known work
/// of the battle's pool, was never read (movie2, 2026-09-26).
#[test]
fn test_a_known_work_built_again_for_another_layout_is_read_as_that_layout() {
    let mut game = Game::new();
    let (work, group) = game.screen(BOTTOM);
    game.text_box(BOTTOM + 0x1000, group, "Tackle");
    game.fake.track().unwrap();
    assert!(!game.fake.layouts.revive().unwrap());
    let (layout, root) = (BOTTOM + 0x5000, BOTTOM + 0x5400);
    game.memory.u32(layout, &[game.fake.layout_class]);
    game.memory.u32(layout + lyt::LAYOUT_ROOT, &[root]);
    game.fake.pane(root, 0);
    game.memory.u32(work + lyt::WORK_LAYOUT, &[layout]);
    assert!(game.fake.layouts.revive().unwrap());
    assert_eq!(game.fake.layouts.roots(), IndexSet::from([root]));
}

/// The move-forget screen hangs every word from its root and draws a frame round the chosen
/// move's power; the Pokémon's stats are in none (omega_ruby_hm-tm.ctm, spike 139).
#[test]
fn test_words_hanging_from_the_root_are_framed_by_the_window_drawn_round_them() {
    let mut game = Game::new();
    let classes = FakeClasses::starting(game.memory.clone(), CODE.0 + 0x8000);
    let (frame_class, picture) = (
        classes.add(lyt::WINDOW_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let (_work, root) = game.fake.screen_with(TOP, true, false);
    let frame = TOP + 0x800;
    game.fake.pane_with_vtable(frame, root, Some(frame_class));
    write_floats(
        &game.memory,
        frame,
        &[
            (lyt::X, -116.0),
            (lyt::Y, -10.0),
            (lyt::WIDTH, 172.0),
            (lyt::HEIGHT, 85.0),
            (lyt::SCALE_X, 1.0),
            (lyt::SCALE_Y, 1.0),
        ],
    );
    let power = game.text_box(TOP + 0x1000, root, "POWER");
    game.place(power, (-150.0, -10.0), 90.0, 12.0);
    let attack = game.text_box(TOP + 0x1200, root, "ATTACK");
    game.place(attack, (55.0, 50.0), 70.0, 12.0);
    // The category icon hangs from the root too, and grouped with the stats it read them all as
    // each move's details (or_tm-hm, 2026-10-18).
    let icon = TOP + 0x1400;
    game.fake.pane_with_vtable(icon, root, Some(picture));
    write_floats(
        &game.memory,
        icon,
        &[
            (lyt::X, -80.0),
            (lyt::Y, 13.0),
            (lyt::WIDTH, 50.0),
            (lyt::HEIGHT, 20.0),
            (lyt::SCALE_X, 1.0),
            (lyt::SCALE_Y, 1.0),
        ],
    );
    game.reader = reader_with(
        &game,
        Rc::new(move |pane| Ok((pane == icon).then(|| "SPECIAL".to_string()))),
    );
    game.sweep();
    game.heard();
    let frames: HashMap<String, i64> = game
        .reader
        .poll(&none(), true)
        .unwrap()
        .texts
        .into_iter()
        .map(|piece| (piece.text, piece.frame))
        .collect();
    assert_eq!(
        frames,
        HashMap::from([
            ("POWER".to_string(), frame),
            ("ATTACK".to_string(), 0),
            ("SPECIAL".to_string(), frame)
        ])
    );
}

/// The Bag's sign before an item's count.
const TIMES: &str = "\u{00d7}";

/// The Bag hangs an item's count under its times sign, which hangs from the root with the
/// item's name and description; as a group of its own the count was read after the description
/// (movie29, spike 196).
#[test]
fn test_words_hung_under_other_words_are_of_their_group() {
    let mut game = Game::new();
    let (_work, root) = game.fake.screen_with(TOP, true, false);
    let times = game.text_box(TOP + 0x1000, root, TIMES);
    game.place(times, (30.0, 46.0), 20.0, 12.5);
    let count = game.text_box(TOP + 0x1200, times, "5");
    game.place(count, (55.0, 45.0), 30.0, 14.0);
    game.sweep();
    game.heard();
    let groups: HashMap<String, (i64, i64)> = game
        .reader
        .poll(&none(), true)
        .unwrap()
        .texts
        .into_iter()
        .map(|piece| (piece.text.clone(), piece.group()))
        .collect();
    assert_eq!(groups["5"], groups[TIMES]);
    assert_eq!(groups[TIMES], (root, root));
}

/// The Pokédex entry's type is an icon beside the word TYPE, heard as the game names the type
/// (spike 147): "TYPE Fire". A picture it has no word for says nothing, nor one parked off the
/// screen, as the summary parks a fifth move's type icon above the bottom screen (movie7).
#[test]
fn test_a_picture_the_game_names_by_its_image_is_heard_where_it_is() {
    let mut game = Game::new();
    let picture =
        FakeClasses::starting(game.memory.clone(), CODE.0 + 0x8000).add(lyt::PICTURE_CLASS);
    let (_work, group) = game.screen(BOTTOM);
    let label = game.text_box(BOTTOM + 0x1000, group, "TYPE");
    game.place(label, (-40.0, 0.0), 40.0, 12.5);
    let (icon, blank, parked) = (BOTTOM + 0x1200, BOTTOM + 0x1400, BOTTOM + 0x1600);
    for (pane, x, y) in [(icon, 20.0, 0.0), (blank, 80.0, 0.0), (parked, 20.0, 140.0)] {
        game.fake.pane_with_vtable(pane, group, Some(picture));
        write_floats(
            &game.memory,
            pane,
            &[
                (lyt::X, x),
                (lyt::Y, y),
                (lyt::WIDTH, 50.0),
                (lyt::HEIGHT, 20.0),
                (lyt::SCALE_X, 1.0),
                (lyt::SCALE_Y, 1.0),
            ],
        );
    }
    game.memory.u32(BOTTOM + 0x400 + lyt::CHILD_COUNT, &[1]);
    game.memory.u32(group + lyt::CHILD_COUNT, &[4]);
    let called: HashMap<i64, &str> = HashMap::from([(icon, "Fire"), (parked, "Normal")]);
    game.reader = reader_with(
        &game,
        Rc::new(move |pane| Ok(called.get(&pane).map(|words| words.to_string()))),
    );
    game.sweep();
    game.heard();
    assert_eq!(game.heard(), words(&[("TYPE", false), ("Fire", false)]));
}

/// Surskit's entry read "Type BugWater", its two type icons joined as drawn (Orion,
/// 2026-09-29): "Bug/Water".
#[test]
fn test_two_icons_side_by_side_are_said_with_a_slash_between() {
    let mut game = Game::new();
    let picture =
        FakeClasses::starting(game.memory.clone(), CODE.0 + 0x8000).add(lyt::PICTURE_CLASS);
    let (_work, group) = game.screen(BOTTOM);
    let label = game.text_box(BOTTOM + 0x1000, group, "TYPE");
    game.place(label, (-60.0, 0.0), 40.0, 12.5);
    let (bug, water) = (BOTTOM + 0x1200, BOTTOM + 0x1400);
    for (pane, x) in [(bug, 0.0), (water, 52.0)] {
        game.fake.pane_with_vtable(pane, group, Some(picture));
        write_floats(
            &game.memory,
            pane,
            &[
                (lyt::X, x),
                (lyt::Y, 0.0),
                (lyt::WIDTH, 50.0),
                (lyt::HEIGHT, 20.0),
                (lyt::SCALE_X, 1.0),
                (lyt::SCALE_Y, 1.0),
            ],
        );
    }
    game.memory.u32(BOTTOM + 0x400 + lyt::CHILD_COUNT, &[1]);
    game.memory.u32(group + lyt::CHILD_COUNT, &[3]);
    let called: HashMap<i64, &str> = HashMap::from([(bug, "Bug"), (water, "Water")]);
    game.reader = reader_with(
        &game,
        Rc::new(move |pane| Ok(called.get(&pane).map(|words| words.to_string()))),
    );
    game.sweep();
    game.heard();
    assert_eq!(
        game.heard(),
        words(&[("TYPE", false), ("Bug/Water", false)])
    );
}

/// The Pokédex entry's group of Name holds its value and the type icon, and moved after the
/// label, the icon read "Name Fire Torchic" (movie7, after "Name" became one of the labels the
/// game's text names).
#[test]
fn test_a_label_with_a_value_of_its_own_keeps_its_group_s_picture_where_it_is() {
    let pieces = measured(&[
        ("Name", -150.0, -125.0, 20.0, 12.5),
        ("Torchic", -110.0, -70.0, 20.0, 12.5),
    ]);
    let fire = ScreenText {
        parent: 0x100,
        picture: true,
        ..ScreenText::placed(0x90, "Fire", false, 20.0, 70.0, 20.0, 20.0)
    };
    let found: IndexMap<i64, ScreenText> = IndexMap::from([
        (
            0x40,
            ScreenText {
                r#box: 0x40,
                parent: 0x100,
                label: true,
                ..pieces[0].clone()
            },
        ),
        (
            0x44,
            ScreenText {
                r#box: 0x44,
                parent: 0x100,
                ..pieces[1].clone()
            },
        ),
        (0x90, fire),
    ]);
    let labels = IndexMap::from([("Name".to_string(), String::new())]);
    let moved = screen::by_their_labels(&found, &labels);
    assert_eq!(moved, found);
}

/// The red mark is drawn beside First Move's label, its move on the row below, and was heard
/// "First Move: egg move, Helping Hand" once the battle's rule for a group's one picture had
/// moved it (spike 195).
#[test]
fn test_the_dexnav_s_mark_is_read_after_the_move_it_marks() {
    let mut game = Game::new();
    let picture =
        FakeClasses::starting(game.memory.clone(), CODE.0 + 0x8000).add(lyt::PICTURE_CLASS);
    let (_work, group) = game.screen(BOTTOM);
    let mark = BOTTOM + 0x0E00;
    game.fake.pane_with_vtable(mark, group, Some(picture));
    write_floats(
        &game.memory,
        mark,
        &[
            (lyt::X, -100.0),
            (lyt::Y, 10.0),
            (lyt::WIDTH, 16.0),
            (lyt::HEIGHT, 16.0),
            (lyt::SCALE_X, 1.0),
            (lyt::SCALE_Y, 1.0),
        ],
    );
    let label = game.text_box(BOTTOM + 0x1000, group, "First Move");
    game.place(label, (-50.0, 10.0), 80.0, 12.5);
    let r#move = game.text_box(BOTTOM + 0x1200, group, "Helping Hand");
    game.place(r#move, (-50.0, -10.0), 100.0, 12.5);
    game.memory.u32(BOTTOM + 0x400 + lyt::CHILD_COUNT, &[1]);
    game.memory.u32(group + lyt::CHILD_COUNT, &[3]);
    game.reader = ScreenTextReader::with(
        game.memory.clone(),
        Rc::new(ClassNames::new(game.memory.clone(), CODE)),
        game.fake.layouts.clone(),
        msgwin::CLASSES.iter().map(|c| c.to_string()).collect(),
        None,
        Some(Rc::new(move |pane| {
            Ok((pane == mark).then(|| pictures::SPECIAL.to_string()))
        })),
        IndexMap::from([("First Move".to_string(), pictures::EGG_MOVE.to_string())]),
        HashSet::new(),
    );
    game.sweep();
    let said = in_rows(game.reader.poll(&none(), false).unwrap().texts);
    assert_eq!(join(&said, false), "First Move Helping Hand egg move");
    assert_eq!(join(&said, true), "First Move: Helping Hand, egg move");
}

/// The battle's move list draws the type icon left of the PP in its left column and right of
/// it in its right one (movie5); both read the type after the name (Orion, 2026-09-29). The
/// Pokédex's TYPE, one row of three in its group, stays where it is drawn.
#[test]
fn test_a_moves_type_is_read_after_its_name_on_either_side_of_the_move_list() {
    fn piece(
        r#box: i64,
        text: &str,
        left: f64,
        row: f64,
        parent: i64,
        picture: bool,
    ) -> ScreenText {
        let glyph = if picture { 20.0 } else { 12.5 };
        ScreenText {
            parent,
            picture,
            ..ScreenText::placed(
                r#box,
                text,
                false,
                left,
                left + 6.0 * text.chars().count() as f64,
                row,
                glyph,
            )
        }
    }

    fn read(pieces: Vec<ScreenText>) -> String {
        let by_box: IndexMap<i64, ScreenText> = pieces.into_iter().map(|p| (p.r#box, p)).collect();
        join(&in_rows(after_the_name(&by_box).values().cloned()), false)
    }

    assert_eq!(
        read(vec![
            piece(1, "Growl", 72.0, 61.0, 9, false),
            piece(2, "Normal", 110.0, 40.0, 9, true),
            piece(3, "PP", 23.0, 40.0, 9, false),
            piece(4, "40/40", 53.0, 40.0, 9, false),
        ]),
        "Growl Normal PP 40/40"
    );
    assert_eq!(
        read(vec![
            piece(1, "Tackle", -108.0, 61.0, 9, false),
            piece(2, "Normal", -159.0, 40.0, 9, true),
            piece(3, "PP", -93.0, 40.0, 9, false),
            piece(4, "35/35", -63.0, 40.0, 9, false),
        ]),
        "Tackle Normal PP 35/35"
    );
    assert_eq!(
        read(vec![
            piece(1, "TYPE", -30.0, 30.0, 9, false),
            piece(2, "Fire", 10.0, 30.0, 9, true),
            piece(3, "HEIGHT", -30.0, 10.0, 9, false),
            piece(4, "WEIGHT", -30.0, -10.0, 9, false),
        ]),
        "TYPE Fire HEIGHT WEIGHT"
    );
}
