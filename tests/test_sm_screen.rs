//! Sun's menus: its TextBoxes read through its own pane shape, and its cursor pane, laid out as
//! spike 112 read the title's menu, the options and the save prompt's Yes and No.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::core::words::{self, ENGLISH, Words};
use crate::games::common::cursors::reader::CursorReader;
use crate::games::common::lyt;
use crate::games::common::screen::{ScreenText, ScreenTextReader, join};
use crate::games::sm::lyt::SHAPE;
use crate::games::sm::{field, msgwin};

const MENU: i64 = 0x08200000;
const CURSOR: i64 = 0x08300000;
/// A static tag processor, as the title menu's labels have.
const TAG: i64 = 0x00673A00;
/// Sun's own alignment codes.
const LEFT: i64 = 1;
const CENTRE: i64 = 0;
#[allow(dead_code)]
const RIGHT: i64 = 2;

/// What `Title::heard` gives: the words of each item, and whether it was opened.
type Heard = (Vec<String>, bool);

/// `struct.pack("<f", value)`.
fn pack(value: f64) -> [u8; 4] {
    (value as f32).to_le_bytes()
}

/// What `Title::heard` is compared with.
fn said(heard: &[(&[&str], bool)]) -> Vec<Heard> {
    heard
        .iter()
        .map(|&(words, opened)| (words.iter().map(|w| w.to_string()).collect(), opened))
        .collect()
}

/// The words of the Title's `Continue` button, which every poll of its menu says first.
fn continue_words() -> Vec<String> {
    vec!["Continue".to_string(), "rewq".to_string()]
}

/// The title's menu: each button a pane holding its words and an empty `Null_Cur` of the
/// cursor's size, and the cursor a plain pane alone under the root of a layout of its own,
/// holding the arrow's picture, moved and sized onto the chosen button's `Null_Cur`.
struct Title {
    memory: Rc<FakeMemory>,
    classes: FakeClasses,
    textbox: i64,
    fake: FakeLayouts,
    root: i64,
    texts: Cell<i64>,
    boxes: RefCell<Vec<i64>>,
    continue_: i64,
    #[allow(dead_code)]
    gift: i64,
    slot: i64,
    icon: i64,
    cursor: i64,
    screen: ScreenTextReader,
    reader: CursorReader,
}

impl Title {
    fn new() -> Title {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let textbox = classes.add(lyt::TEXTBOX_CLASS);
        let picture = classes.add(lyt::PICTURE_CLASS);
        let fake = FakeLayouts::with_shape(&memory, &classes, SHAPE);
        let (work, root) = fake.screen_with(MENU, true, false);
        fake.project(work, 400.0);
        // The readers go first, as the struct wants every field: they read nothing until polled.
        let names = Rc::new(ClassNames::new(memory.clone(), CODE));
        let screen = ScreenTextReader::with(
            memory.clone(),
            names.clone(),
            fake.layouts.clone(),
            msgwin::CLASSES.iter().map(|c| c.to_string()).collect(),
            None,
            None,
            IndexMap::new(),
            HashSet::new(),
        );
        let reader = CursorReader::with(fake.layouts.clone(), names, None, true, None);
        let mut title = Title {
            memory,
            classes,
            textbox,
            fake,
            root,
            texts: Cell::new(0x08900000),
            boxes: RefCell::new(Vec::new()),
            continue_: 0,
            gift: 0,
            slot: 0,
            icon: 0,
            cursor: 0,
            screen,
            reader,
        };
        title.continue_ = title.button(
            MENU + 0x1000,
            (5.0, 32.0),
            (312.0, 20.0),
            &[("Continue", 83.0), ("rewq", 55.0)],
        );
        title.gift = title.button(
            MENU + 0x2000,
            (1.0, -69.0),
            (304.0, 20.0),
            &[("Mystery Gift", -69.0)],
        );
        title.slot = title.button(
            MENU + 0x3000,
            (1.0, -99.0),
            (304.0, 20.0),
            &[("BAG", -99.0)],
        );
        title.fake.visible(title.slot + 0x400, false);
        title.icon = title.button(MENU + 0x4000, (1.0, -129.0), (304.0, 20.0), &[]);
        let (work, cursor_root) = title.fake.screen_with(CURSOR, true, false);
        title.fake.project(work, 400.0);
        title.cursor = CURSOR + 0x800;
        title.fake.pane(title.cursor, cursor_root);
        title
            .fake
            .pane_with_vtable(title.cursor + 0x200, title.cursor, Some(picture));
        title.fake.track().unwrap();
        title.screen.track(title.boxes.borrow().clone());
        title.move_((5.0, 32.0), (312.0, 20.0));
        title
    }

    fn button(&self, group: i64, at: (f64, f64), size: (f64, f64), words: &[(&str, f64)]) -> i64 {
        self.fake.pane(group, self.root);
        self.fake.pane(group + 0x200, group);
        self.set(group + 0x200, at, size);
        for (index, &(text, y)) in words.iter().enumerate() {
            let r#box = group + 0x400 + index as i64 * 0x200;
            self.text_box(r#box, group, text, (0.0, y));
        }
        group
    }

    /// `text_box(box, parent, text, at)`: centred, and anchored in the centre.
    fn text_box(&self, r#box: i64, parent: i64, text: &str, at: (f64, f64)) {
        self.text_box_aligned(r#box, parent, text, at, CENTRE, CENTRE);
    }

    fn text_box_aligned(
        &self,
        r#box: i64,
        parent: i64,
        text: &str,
        at: (f64, f64),
        align: i64,
        anchor: i64,
    ) {
        self.fake
            .pane_with_vtable(r#box, parent, Some(self.textbox));
        self.set(r#box, at, (260.0, 20.0));
        let raw: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
        self.memory.write(self.texts.get(), &raw);
        self.memory.u32(r#box + SHAPE.text, &[self.texts.get()]);
        self.memory.u32(r#box + SHAPE.tag_processor, &[TAG]);
        self.memory.u16(r#box + SHAPE.capacity, 0x40);
        self.memory
            .u16(r#box + SHAPE.length, (raw.len() / 2) as i64);
        self.memory.write(r#box + SHAPE.glyph_width, &pack(15.0));
        self.memory
            .write(r#box + SHAPE.anchor, &[(0x10 | anchor) as u8]);
        self.memory
            .write(r#box + SHAPE.text_position, &[(0x04 | align) as u8]);
        self.texts.set(self.texts.get() + 0x100);
        self.boxes.borrow_mut().push(r#box);
    }

    fn set(&self, pane: i64, at: (f64, f64), size: (f64, f64)) {
        self.memory.write(pane + SHAPE.scale_x, &pack(1.0));
        self.memory.write(pane + SHAPE.scale_y, &pack(1.0));
        self.memory.write(pane + SHAPE.x, &pack(at.0));
        self.memory.write(pane + SHAPE.y, &pack(at.1));
        let mut width = pack(size.0).to_vec();
        width.extend(pack(size.1));
        self.memory.write(pane + SHAPE.width, &width);
    }

    /// `move(at, size)`.
    fn move_(&self, at: (f64, f64), size: (f64, f64)) {
        self.set(self.cursor, at, size);
    }

    /// What the screen shows, once its words have held for two polls.
    fn seen(&mut self) -> Vec<ScreenText> {
        self.screen.poll(&HashSet::new(), false).unwrap();
        self.screen.on_screen()
    }

    fn heard(&mut self) -> Vec<Heard> {
        let on_screen = self.seen();
        let items = self.reader.poll(&on_screen).unwrap().items;
        items
            .iter()
            .map(|item| {
                (
                    item.pieces.iter().map(|p| p.text.clone()).collect(),
                    item.opened,
                )
            })
            .collect()
    }
}

#[test]
fn test_a_sun_textbox_is_read_through_suns_pane_shape() {
    let mut title = Title::new();
    let seen: HashSet<String> = title.seen().into_iter().map(|piece| piece.text).collect();
    let expected: HashSet<String> = ["Continue", "rewq", "Mystery Gift"]
        .iter()
        .map(|w| w.to_string())
        .collect();
    assert_eq!(seen, expected);
}

/// Sun keeps centre as 0 and left as 1, where ORAS keeps left as 0 (spike 112): "rewq" is
/// anchored and set at the left of its 260 units, at x -36.
#[test]
fn test_a_sun_textbox_is_placed_by_suns_alignment_codes() {
    let mut title = Title::new();
    let r#box = title.continue_ + 0x2000;
    title.text_box_aligned(r#box, title.continue_, "Route 1", (-36.0, 35.0), LEFT, LEFT);
    title.screen.track(title.boxes.borrow().clone());
    let pieces: Vec<ScreenText> = title
        .seen()
        .into_iter()
        .filter(|piece| piece.text == "Route 1")
        .collect();
    let [piece] = pieces.as_slice() else {
        panic!("one piece of \"Route 1\", got {pieces:?}");
    };
    assert_eq!(piece.left, -36.0);
}

/// The title's Continue: the cursor sits on the report's `Null_Cur`, not on its words.
#[test]
fn test_the_cursor_pane_says_every_word_of_the_button_it_is_fitted_to() {
    assert_eq!(Title::new().heard(), said(&[(&["Continue", "rewq"], true)]));
}

#[test]
fn test_moving_the_cursor_pane_says_the_next_button() {
    let mut title = Title::new();
    title.heard();
    title.move_((1.0, -69.0), (304.0, 20.0));
    assert_eq!(title.heard(), said(&[(&["Mystery Gift"], false)]));
}

/// The X menu's slots for entries the story has yet to give: BAG, hidden, on an empty plate.
#[test]
fn test_a_button_whose_words_are_hidden_is_said_as_empty() {
    let mut title = Title::new();
    title.heard();
    title.move_((1.0, -99.0), (304.0, 20.0));
    assert_eq!(title.heard(), said(&[(&["empty"], false)]));
}

/// The X menu's SAVE, the poll it opens on, before its words have read the same twice.
#[test]
fn test_a_button_whose_words_are_shown_but_not_yet_read_waits_for_them() {
    let mut title = Title::new();
    title.heard();
    title.move_((1.0, -69.0), (304.0, 20.0));
    let unread: Vec<ScreenText> = title
        .seen()
        .into_iter()
        .filter(|piece| piece.text != "Mystery Gift")
        .collect();
    assert!(title.reader.poll(&unread).unwrap().items.is_empty());
    assert_eq!(title.heard(), said(&[(&["Mystery Gift"], false)]));
}

/// A video call's buttons are icons, with no words to hide and no name to say.
#[test]
fn test_a_button_of_pictures_alone_says_nothing() {
    let mut title = Title::new();
    title.heard();
    title.move_((1.0, -129.0), (304.0, 20.0));
    assert_eq!(title.heard(), said(&[]));
}

/// The battle's command menu: `Btn_Fight`, `Btn_Bag`, `Btn_Pokemon_00` and `Btn_Run`, each a
/// Parts pane of pictures with no words, and no layout file to name the pictures from (spike
/// 124). Its cursor pane is 64 wide over a `Null_Cur` of 56, and fits it still.
#[test]
fn test_a_button_of_pictures_alone_is_said_by_its_panes_name() {
    let mut title = Title::new();
    title.heard();
    let mut name = b"Btn_Pokemon_00".to_vec();
    name.extend([0, 0]);
    title.memory.write(title.icon + SHAPE.name, &name);
    title.move_((1.0, -129.0), (312.0, 20.0));
    assert_eq!(title.heard(), said(&[(&["pokémon"], false)]));
}

/// The passport's photo choice: `Hero_00` to `Hero_07`, each a Parts pane of one picture, were
/// said "hero" (sun_1.ctm). Sun's English data file describes each photo.
#[test]
fn test_a_button_of_pictures_alone_is_said_by_pa3ds_own_words_where_it_has_some() {
    let mut title = Title::new();
    words::r#use(Some(Rc::new(
        Words::new(
            ENGLISH,
            Some(&field::names()),
            IndexMap::new(),
            Rc::new(|_line: &str| {}),
        )
        .unwrap(),
    )));
    title.reader = CursorReader::with(
        title.fake.layouts.clone(),
        Rc::new(ClassNames::new(title.memory.clone(), CODE)),
        None,
        true,
        None,
    );
    title.heard();
    let mut name = b"Hero_00".to_vec();
    name.extend([0; 9]);
    title.memory.write(title.icon + SHAPE.name, &name);
    title.move_((1.0, -129.0), (304.0, 20.0));
    assert_eq!(
        title.heard(),
        said(&[(
            &["Boy, fair skin, blond hair, blue eyes, black cap with a white logo, white T-shirt"],
            false
        )])
    );
}

/// The Yes and No's own `Null_Cur`s are hidden and never placed: its item is its words.
#[test]
fn test_a_cursor_pane_fitted_to_no_button_says_the_words_inside_it() {
    let mut title = Title::new();
    title.heard();
    title.move_((0.0, -69.0), (200.0, 20.0));
    assert_eq!(title.heard(), said(&[(&["Mystery Gift"], false)]));
}

const ANSWERS: i64 = 0x08600000;

/// The talk window's answers, as sun_8.ctm has Kukui's question: "Yeah!" and "...I guess" in
/// one window, and a picture alone in a plain pane pointing at one from its left.
#[test]
fn test_a_pointer_beside_the_answers_in_a_box_says_the_one_it_is_on() {
    let mut title = Title::new();
    let window = title.classes.add(lyt::WINDOW_CLASS);
    let picture = title.classes.add(lyt::PICTURE_CLASS);
    let (work, root) = title.fake.screen_with(ANSWERS, true, false);
    title.fake.project(work, 400.0);
    let r#box = ANSWERS + 0x1000;
    title.fake.pane_with_vtable(r#box, root, Some(window));
    title.set(r#box, (144.5, -27.0), (71.0, 60.0));
    title.text_box_aligned(r#box + 0x200, r#box, "Yeah!", (120.0, -17.0), LEFT, LEFT);
    title.text_box_aligned(
        r#box + 0x400,
        r#box,
        "...I guess",
        (120.0, -37.0),
        LEFT,
        LEFT,
    );
    let holder = r#box + 0x800;
    title.fake.pane(holder, r#box);
    title
        .fake
        .pane_with_vtable(holder + 0x200, holder, Some(picture));
    title.set(holder + 0x200, (107.0, -17.0), (24.0, 20.0));

    fn point(title: &Title, holder: i64, y: f64) {
        title.set(holder, (107.0, y), (24.0, 20.0));
    }

    fn heard(title: &mut Title) -> Vec<Heard> {
        title
            .heard()
            .into_iter()
            .filter(|said| said.0 != continue_words())
            .collect()
    }

    point(&title, holder, -17.0);
    title.fake.track().unwrap();
    title.screen.track(title.boxes.borrow().clone());
    assert_eq!(heard(&mut title), said(&[(&["Yeah!"], true)]));
    point(&title, holder, -37.0);
    assert_eq!(heard(&mut title), said(&[(&["...I guess"], false)]));
}

const KEYS: i64 = 0x08400000;

/// The name entry's keys, and its cursor: a window alone under a Parts pane inside the keys'
/// own layout, moved and sized onto the chosen key (spike 112).
struct Keyboard {
    title: Title,
    #[allow(dead_code)]
    frame: i64,
    window: i64,
}

impl Keyboard {
    fn new() -> Keyboard {
        let mut title = Title::new();
        let memory = title.memory.clone();
        let parts = title.classes.add(lyt::PARTS_CLASS);
        let window = title.classes.add(lyt::WINDOW_CLASS);
        let (work, root) = title.fake.screen_with(KEYS, false, false);
        title.fake.project(work, 320.0);
        title.text_box(KEYS + 0x1000, root, "!", (-130.0, 45.0));
        title.text_box(KEYS + 0x1200, root, "Delete", (105.0, 45.0));
        let frame = KEYS + 0x2000;
        title.fake.pane_with_vtable(frame, root, Some(parts));
        memory.u32(frame + lyt::CHILD_COUNT, &[1]);
        let window_pane = KEYS + 0x2200;
        title
            .fake
            .pane_with_vtable(window_pane, frame, Some(window));
        title.fake.track().unwrap();
        title.screen.track(title.boxes.borrow().clone());
        title.reader.track([window_pane]).unwrap();
        let keyboard = Keyboard {
            title,
            frame,
            window: window_pane,
        };
        keyboard.move_((-130.0, 45.0), (38.0, 48.0));
        keyboard
    }

    /// `move(at, size)`.
    fn move_(&self, at: (f64, f64), size: (f64, f64)) {
        self.title.set(self.window, at, size);
    }

    fn heard(&mut self) -> Vec<Heard> {
        self.title
            .heard()
            .into_iter()
            .filter(|heard| heard.0 != continue_words())
            .collect()
    }
}

#[test]
fn test_the_keyboards_cursor_says_the_key_it_is_on() {
    let mut keyboard = Keyboard::new();
    assert_eq!(keyboard.heard(), said(&[(&["!"], true)]));
    keyboard.move_((105.0, 45.0), (68.0, 50.0));
    assert_eq!(keyboard.heard(), said(&[(&["Delete"], false)]));
}

/// The keyboard's other page keeps its cursor parked where the page in front has "!".
#[test]
fn test_a_keyboard_cursor_parked_over_another_layouts_keys_says_nothing() {
    let mut keyboard = Keyboard::new();
    let memory = keyboard.title.memory.clone();
    let parts = keyboard.title.classes.add(lyt::PARTS_CLASS);
    let window = keyboard.title.classes.add(lyt::WINDOW_CLASS);
    let other = 0x08500000;
    let (work, root) = keyboard.title.fake.screen_with(other, false, false);
    keyboard.title.fake.project(work, 320.0);
    keyboard
        .title
        .fake
        .pane_with_vtable(other + 0x2000, root, Some(parts));
    memory.u32(other + 0x2000 + lyt::CHILD_COUNT, &[1]);
    keyboard
        .title
        .fake
        .pane_with_vtable(other + 0x2200, other + 0x2000, Some(window));
    keyboard
        .title
        .set(other + 0x2200, (-120.0, 45.0), (40.0, 48.0));
    keyboard.title.fake.track().unwrap();
    keyboard.title.reader.track([other + 0x2200]).unwrap();
    assert_eq!(keyboard.heard(), said(&[(&["!"], true)]));
}

/// Rowlet's slot keeps "Lv." as a lone "l" in the digits' font, just left of the level in its
/// own TextBox: "Rowlet 22 22 l5" (sun_9.ctm). A lone "l" anywhere else is left as it is.
#[test]
fn test_the_party_screens_level_glyph_is_said_as_the_games_words_for_a_level() {
    let title = Title::new();
    let slot = MENU + 0x5000;
    title.fake.pane(slot, title.root);
    title.text_box_aligned(slot + 0x200, slot, "l", (-101.0, 98.0), LEFT, LEFT);
    title.text_box_aligned(slot + 0x400, slot, "5", (-94.0, 98.0), LEFT, LEFT);
    title.text_box_aligned(slot + 0x600, slot, "l", (-101.0, 60.0), LEFT, LEFT);
    title.fake.track().unwrap();
    let mut screen = ScreenTextReader::with(
        title.memory.clone(),
        Rc::new(ClassNames::new(title.memory.clone(), CODE)),
        title.fake.layouts.clone(),
        msgwin::CLASSES.iter().map(|c| c.to_string()).collect(),
        Some("Lv.".to_string()),
        None,
        IndexMap::new(),
        HashSet::new(),
    );
    screen.track(title.boxes.borrow().clone());
    screen.poll(&HashSet::new(), false).unwrap();
    let texts: Vec<String> = screen
        .on_screen()
        .into_iter()
        .filter(|piece| piece.parent == slot)
        .map(|piece| piece.text)
        .collect();
    assert_eq!(texts, ["Lv.", "5", "l"]);
}

/// The party slot's "/" is the layout file's own, with no tag processor, between the current
/// HP and the maximum that code writes: read as written words only, it was "Rowlet 22" and the
/// maximum run together (sun_9.ctm). The designer's placeholders, and a "/" beside no written
/// words of its own panel, are left out.
#[test]
fn test_the_designers_slash_between_the_hp_and_its_maximum_is_read() {
    let mut title = Title::new();
    let slot = MENU + 0x5000;
    title.fake.pane(slot, title.root);
    title.text_box_aligned(slot + 0x200, slot, "22", (-93.0, 62.0), LEFT, LEFT);
    title.text_box_aligned(slot + 0x400, slot, "/", (-78.0, 62.0), LEFT, LEFT);
    title.text_box_aligned(
        slot + 0x600,
        slot,
        "\u{ff12}\u{ff12}",
        (-70.0, 62.0),
        LEFT,
        LEFT,
    );
    title.text_box_aligned(slot + 0x800, slot, "999", (-93.0, 20.0), LEFT, LEFT);
    title.text_box_aligned(slot + 0xA00, slot, "/", (-78.0, 20.0), LEFT, LEFT);
    // As the HP box comes on its panels sit at one place, a double battle's "/" on the player's.
    let other = MENU + 0x6000;
    title.fake.pane(other, title.root);
    title.text_box_aligned(other + 0x200, other, "/", (-78.0, 62.0), LEFT, LEFT);
    for designed in [slot + 0x400, slot + 0x800, slot + 0xA00, other + 0x200] {
        title.memory.u32(designed + SHAPE.tag_processor, &[0]);
    }
    title.fake.track().unwrap();
    title.screen.track(title.boxes.borrow().clone());
    let seen = title.seen();
    let in_slot: Vec<ScreenText> = seen
        .iter()
        .filter(|piece| piece.parent == slot)
        .cloned()
        .collect();
    assert_eq!(join(&in_slot, false), "22/\u{ff12}\u{ff12}");
    assert!(!seen.iter().any(|piece| piece.parent == other));
}
