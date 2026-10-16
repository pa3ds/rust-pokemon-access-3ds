//! Menu cursor tests, against a fake cursor layout shaped as spikes 39 to 41 read the X menu's.

// Python builds each fake game in one `__init__`, calling its own methods on the half-built object
// and setting the reader last. Here a struct is built as soon as its fields exist and holds a
// reader made the plain way until `__init__`'s own line replaces it.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_layout_file as clyt;
use crate::core::binary;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::reader::CursorReader;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::lyt;
use crate::games::common::screen::{Pictures, ScreenText, join};
use crate::games::sm::lyt::SHAPE as SUN_SHAPE;

const BASE: i64 = 0x08200000;
/// The root of the menu layout the labels are in.
const MENU: i64 = 0x08300400;

/// `struct.pack("<f", ...)`, one `f` for each value.
fn pack(values: &[f64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&value| (value as f32).to_le_bytes())
        .collect()
}

/// `memory.write(address, struct.pack("<f", ...))`, one `f` for each value.
fn put(memory: &FakeMemory, address: i64, values: &[f64]) {
    memory.write(address, &pack(values));
}

/// `memory.write(pane + lyt.NAME, name.encode("ascii").ljust(lyt.NAME_SIZE, b"\0"))`.
fn write_name(memory: &FakeMemory, pane: i64, name: &str) {
    write_name_in(memory, pane, name, &lyt::ORAS);
}

/// `write_name` where `shape` keeps a pane's name.
fn write_name_in(memory: &FakeMemory, pane: i64, name: &str, shape: &lyt::PaneShape) {
    let mut bytes = name.as_bytes().to_vec();
    if (bytes.len() as i64) < lyt::NAME_SIZE {
        bytes.resize(lyt::NAME_SIZE as usize, 0);
    }
    memory.write(pane + shape.name, &bytes);
}

/// The cursor reader, keeping what its last poll found for a test to look at.
struct Reader {
    reader: CursorReader,
    last: CursorPoll,
}

impl From<CursorReader> for Reader {
    fn from(reader: CursorReader) -> Self {
        Reader {
            reader,
            last: CursorPoll::default(),
        }
    }
}

impl Reader {
    fn poll(&mut self, on_screen: &[ScreenText]) -> crate::py::Result<CursorPoll> {
        self.last = self.reader.poll(on_screen)?;
        Ok(self.last.clone())
    }
}

impl std::ops::Deref for Reader {
    type Target = CursorReader;

    fn deref(&self) -> &CursorReader {
        &self.reader
    }
}

impl std::ops::DerefMut for Reader {
    fn deref_mut(&mut self) -> &mut CursorReader {
        &mut self.reader
    }
}

/// `ClassNames(memory, CODE)`.
fn names_of(memory: &Rc<FakeMemory>) -> Rc<ClassNames> {
    Rc::new(ClassNames::new(memory.clone(), CODE))
}

/// A reader over the game's layouts, made the way every test but one makes it.
fn reader_of(fake: &FakeLayouts, memory: &Rc<FakeMemory>) -> Reader {
    CursorReader::new(fake.layouts.clone(), names_of(memory)).into()
}

/// `CursorReader(fake.layouts, ClassNames(memory, CODE), files)`.
fn reader_with_files(
    fake: &FakeLayouts,
    memory: &Rc<FakeMemory>,
    files: &Rc<LayoutFiles>,
) -> Reader {
    CursorReader::with_files(fake.layouts.clone(), names_of(memory), files.clone()).into()
}

/// `clyt.archive([clyt.layout_file(sections)])`.
fn layout_archive(sections: Vec<Vec<u8>>) -> Vec<u8> {
    clyt::archive(&[clyt::layout_file(&sections)])
}

/// `[(tuple(p.text for p in item.pieces), item.opened) for item in items]`.
fn heard_of(items: Vec<CursorItem>) -> Vec<(Vec<String>, bool)> {
    items
        .into_iter()
        .map(|item| {
            (
                item.pieces.iter().map(|p| p.text.clone()).collect(),
                item.opened,
            )
        })
        .collect()
}

/// What `heard` is compared with.
fn said(heard: &[(&[&str], bool)]) -> Vec<(Vec<String>, bool)> {
    heard
        .iter()
        .map(|&(texts, opened)| (texts.iter().map(|t| t.to_string()).collect(), opened))
        .collect()
}

/// What a `heard` of joined pieces is compared with.
fn lines(heard: &[(&str, bool)]) -> Vec<(String, bool)> {
    heard
        .iter()
        .map(|&(text, opened)| (text.to_string(), opened))
        .collect()
}

/// `{root: frozenset(boxes)}`.
fn lists_of(root: i64, boxes: impl IntoIterator<Item = i64>) -> IndexMap<i64, HashSet<i64>> {
    IndexMap::from([(root, boxes.into_iter().collect())])
}

/// A piece of screen text whose words span 30 units from `x`.
fn label(r#box: i64, text: &str, x: f64, y: f64) -> ScreenText {
    label_with(r#box, text, x, y, false)
}

fn label_with(r#box: i64, text: &str, x: f64, y: f64, top: bool) -> ScreenText {
    ScreenText {
        top,
        root: MENU,
        ..ScreenText::placed(r#box, text, false, x, x + 30.0, y, 12.5)
    }
}

fn x_menu() -> Vec<ScreenText> {
    vec![
        label(1, "POKÉMON", -70.0, 60.0),
        label(2, "BAG", 90.0, 60.0),
        label(3, "wee", -70.0, 0.0),
        label(4, "SAVE", 90.0, 0.0),
    ]
}

/// A cursor layout: a root whose only child is a 150 by 40 window, drawn below.
struct Menu {
    memory: Rc<FakeMemory>,
    fake: FakeLayouts,
    window: i64,
    reader: Reader,
}

impl Menu {
    fn new() -> Menu {
        Menu::with(1)
    }

    fn with(children: i64) -> Menu {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let window_class = classes.add(lyt::WINDOW_CLASS);
        let fake = FakeLayouts::new(&memory, &classes);
        let (work, _group) = fake.screen(BASE);
        fake.project(work, 320.0);
        let root = BASE + 0x400;
        memory.u32(root + lyt::CHILD_COUNT, &[children]);
        let window = BASE + 0x800;
        fake.pane_with_vtable(window, root, Some(window_class));
        for (offset, value) in [
            (lyt::WIDTH, 150.0),
            (lyt::HEIGHT, 40.0),
            (lyt::SCALE_X, 1.0),
            (lyt::SCALE_Y, 1.0),
        ] {
            put(&memory, window + offset, &[value]);
        }
        let reader = reader_of(&fake, &memory);
        let mut menu = Menu {
            memory,
            fake,
            window,
            reader,
        };
        menu.r#move(-80.0, 60.0);
        menu.fake.track().unwrap();
        menu.reader = reader_of(&menu.fake, &menu.memory);
        menu.reader.track([menu.window]).unwrap();
        menu
    }

    fn r#move(&self, x: f64, y: f64) {
        put(&self.memory, self.window + lyt::X, &[x]);
        put(&self.memory, self.window + lyt::Y, &[y]);
    }

    fn heard(&mut self) -> Vec<(Vec<String>, bool)> {
        self.heard_with(&x_menu())
    }

    fn heard_with(&mut self, pieces: &[ScreenText]) -> Vec<(Vec<String>, bool)> {
        self.fake.layouts.refresh();
        let items = self.reader.poll(pieces).unwrap().items;
        heard_of(items)
    }
}

#[test]
fn test_a_cursor_coming_on_screen_says_the_item_under_it() {
    assert_eq!(Menu::new().heard(), said(&[(&["POKÉMON"], true)]));
}

#[test]
fn test_a_cursor_polled_again_says_nothing() {
    let mut menu = Menu::new();
    menu.heard();
    assert_eq!(menu.heard(), said(&[]));
}

#[test]
fn test_moving_the_cursor_says_the_new_item() {
    let mut menu = Menu::new();
    menu.heard();
    menu.r#move(80.0, 60.0);
    assert_eq!(menu.heard(), said(&[(&["BAG"], false)]));
    menu.r#move(80.0, 0.0);
    assert_eq!(menu.heard(), said(&[(&["SAVE"], false)]));
}

/// The party screen's slot holds a name, a gender sign, a level and HP.
#[test]
fn test_every_piece_inside_the_cursor_is_the_item() {
    let mut menu = Menu::new();
    let slot = [
        label(1, "Torchic", -140.0, 66.0),
        label(2, "L5", -100.0, 52.0),
    ];
    assert_eq!(menu.heard_with(&slot), said(&[(&["Torchic", "L5"], true)]));
}

/// The user heard "Growl PP / 40": the "/" is built before the numbers, and one "40" was taken
/// for a doubled label (2026-09-14).
#[test]
fn test_a_move_row_is_read_in_place_and_keeps_both_pp_numbers() {
    let mut menu = Menu::new();
    let row = [
        label(1, "Growl", -120.0, 70.0),
        label(2, "PP", -130.0, 50.0),
        label(3, "/", -87.0, 50.0),
        label(4, "40", -102.0, 50.0),
        label(5, "40", -72.0, 50.0),
    ];
    assert_eq!(
        menu.heard_with(&row),
        said(&[(&["Growl", "PP", "40", "/", "40"], true)])
    );
}

#[test]
fn test_a_label_drawn_twice_in_one_place_is_one_piece() {
    let mut menu = Menu::new();
    let doubled = [
        label(1, "POKÉMON", -70.0, 60.0),
        label(2, "POKÉMON", -69.0, 61.0),
    ];
    assert_eq!(menu.heard_with(&doubled), said(&[(&["POKÉMON"], true)]));
}

/// battles2.ctm: with the Pokédex in the X menu, OPTIONS sits over AreaNav's "Littleroot Town",
/// whose middle is under the cursor and whose start is well outside it, and the item was said as
/// "Littleroot Town OPTIONS" (the user, 2026-09-17). The cursor was 1 to 159 across, OPTIONS 46 to
/// 90 and the town -24 to 74 (spike 68).
#[test]
fn test_words_of_another_layout_reaching_out_from_under_the_menu_are_not_its_item() {
    let mut menu = Menu::new();
    let town = ScreenText {
        root: 0x15A5CCDC,
        ..ScreenText::placed(9, "Littleroot Town", false, -190.0, -92.0, 58.0, 13.0)
    };
    assert_eq!(
        menu.heard_with(&[label(1, "OPTIONS", -120.0, 60.0), town]),
        said(&[(&["OPTIONS"], true)])
    );
}

/// Where a piece's end is only an estimate, a long label can seem wider than its button.
#[test]
fn test_words_reaching_out_from_under_the_cursor_are_the_item_when_nothing_else_is() {
    let mut menu = Menu::new();
    let long = label(1, "A long item name", -200.0, 60.0);
    assert_eq!(
        menu.heard_with(&[ScreenText {
            right: -60.0,
            ..long
        }]),
        said(&[(&["A long item name"], true)])
    );
}

/// The Bag's list is a pane a row, each holding its item's words, under a layout of its own;
/// it writes an item into a row as it scrolls it into view, and "Soft Sand" was said as new
/// beside the cursor's "Quick Claw" (movie29, spike 196). The pocket's name, hanging from the
/// root, is no row.
#[test]
fn test_a_menu_cursors_rows_are_the_row_it_is_over_and_the_rows_drawn_like_it() {
    let mut menu = Menu::new();
    let (_work, root) = menu.fake.screen_with(0x08400000, false, false);
    let mut pieces: Vec<ScreenText> = Vec::new();
    for (i, (text, y)) in [
        ("Quick Claw", 60.0),
        ("Escape Rope", 30.0),
        ("Soft Sand", 0.0),
    ]
    .into_iter()
    .enumerate()
    {
        let row = 0x08401000 + i as i64 * 0x100;
        menu.fake.pane(row, root);
        for (offset, value) in [
            (lyt::X, -80.0),
            (lyt::Y, y),
            (lyt::WIDTH, 150.0),
            (lyt::HEIGHT, 30.0),
        ] {
            put(&menu.memory, row + offset, &[value]);
        }
        pieces.push(ScreenText {
            root,
            parent: row,
            ..ScreenText::placed(
                0x08402000 + i as i64 * 0x100,
                text,
                false,
                -120.0,
                -40.0,
                y,
                12.5,
            )
        });
    }
    let pocket = ScreenText {
        root,
        parent: root,
        ..ScreenText::placed(0x08403000, "ITEMS", false, -120.0, -60.0, 110.0, 12.5)
    };
    menu.fake.track().unwrap();
    let mut all = pieces.clone();
    all.push(pocket);
    assert_eq!(menu.heard_with(&all), said(&[(&["Quick Claw"], true)]));
    assert_eq!(
        menu.reader.last.rows.clone(),
        pieces
            .iter()
            .map(|piece| piece.r#box)
            .collect::<HashSet<i64>>()
    );
}

/// The Pokéblock Case's cursor sits on a block's cell, its count just below its frame, over
/// the Berry Blender's grid, whose cell sits off its middle with its count inside the frame:
/// framed, the case said the blender's "16" for its own "5" (movie30, spike 196).
#[test]
fn test_a_menu_cursor_sitting_on_a_pane_says_the_words_under_that_pane() {
    let mut menu = Menu::new();
    let mut pieces: Vec<ScreenText> = Vec::new();
    for (base, (x, y), text, row) in [
        (0x08400000, (-80.0, 60.0), "5", 36.0),
        (0x08500000, (-75.0, 70.0), "16", 58.0),
    ] {
        let (_work, root) = menu.fake.screen_with(base, false, false);
        let cell = base + 0x1000;
        menu.fake.pane(cell, root);
        for (offset, value) in [
            (lyt::X, x),
            (lyt::Y, y),
            (lyt::WIDTH, 150.0),
            (lyt::HEIGHT, 40.0),
        ] {
            put(&menu.memory, cell + offset, &[value]);
        }
        pieces.push(ScreenText {
            root,
            parent: cell,
            ..ScreenText::placed(base + 0x2000, text, false, -90.0, -80.0, row, 12.5)
        });
    }
    menu.fake.track().unwrap();
    assert_eq!(menu.heard_with(&pieces), said(&[(&["5"], true)]));
}

/// A summary keeps its battle page's moves and its contest page's on screen at one place,
/// and its cursor said the battle page's on the contest page (movie32).
#[test]
fn test_one_item_drawn_by_two_layouts_at_one_place_is_said_whole() {
    let mut menu = Menu::new();
    let mut pieces: Vec<ScreenText> = Vec::new();
    for (base, words) in [
        (0x08400000, ["Meteor Mash", "Steel"]),
        (0x08500000, ["Meteor Mash", "Coolness"]),
    ] {
        let (_work, root) = menu.fake.screen_with(base, false, false);
        let row = base + 0x1000;
        menu.fake.pane(row, root);
        for (offset, value) in [
            (lyt::X, -80.0),
            (lyt::Y, 60.0),
            (lyt::WIDTH, 150.0),
            (lyt::HEIGHT, 40.0),
        ] {
            put(&menu.memory, row + offset, &[value]);
        }
        for (i, text) in words.into_iter().enumerate() {
            let i = i as i64;
            pieces.push(ScreenText {
                root,
                parent: row,
                ..ScreenText::placed(
                    base + 0x2000 + 0x100 * i,
                    text,
                    false,
                    -140.0 + 50.0 * i as f64,
                    -100.0 + 50.0 * i as f64,
                    62.0 - 12.0 * i as f64,
                    12.5,
                )
            });
        }
    }
    menu.fake.track().unwrap();
    assert_eq!(
        menu.heard_with(&pieces),
        said(&[(&["Meteor Mash", "Steel", "Coolness"], true)])
    );
}

/// Confirming a language said "Spanish French Begin game": the list panel was still on screen
/// behind the confirmation panel, and the cursor covered rows of both (opening.ctm, spike 68). The
/// cursor ran from -76 to -34, "Begin game" sat at -55, and the list's rows at -40 and -70.
#[test]
fn test_the_panel_a_cursor_is_over_is_the_one_nearest_its_middle() {
    let mut menu = Menu::new();
    menu.r#move(-80.0, 60.0);
    let leaving = [
        ScreenText {
            panel: 0x11,
            ..label(1, "Spanish", -120.0, 75.0)
        },
        ScreenText {
            panel: 0x11,
            ..label(2, "French", -120.0, 45.0)
        },
    ];
    let confirm = ScreenText {
        panel: 0x22,
        ..label(3, "Begin game", -120.0, 60.0)
    };
    let [spanish, french] = leaving;
    assert_eq!(
        menu.heard_with(&[spanish, french, confirm]),
        said(&[(&["Begin game"], true)])
    );
}

/// The Pokédex's More list put its CANCEL row over the page's Habitat button, both on the
/// cursor's row: the list's corners sat centred on CANCEL, Habitat reached in at their left edge,
/// and the cursor said "Habitat" (or/movie7_menus, 2026-09-27).
#[test]
fn test_of_two_panels_on_the_cursors_row_the_one_nearer_its_middle_across_is_its_item() {
    let mut menu = Menu::new();
    menu.r#move(-80.0, 60.0);
    let habitat = ScreenText {
        panel: 0x11,
        ..label(1, "Habitat", -150.0, 60.0)
    };
    let cancel = ScreenText {
        panel: 0x22,
        ..label(2, "CANCEL", -95.0, 60.0)
    };
    assert_eq!(
        menu.heard_with(&[habitat, cancel]),
        said(&[(&["CANCEL"], true)])
    );
}

/// A party slot's name and level are rows of one panel.
#[test]
fn test_an_item_of_one_panel_keeps_its_rows() {
    let mut menu = Menu::new();
    let slot = [
        ScreenText {
            panel: 0x11,
            ..label(1, "Torchic", -140.0, 66.0)
        },
        ScreenText {
            panel: 0x11,
            ..label(2, "L5", -100.0, 52.0)
        },
    ];
    assert_eq!(menu.heard_with(&slot), said(&[(&["Torchic", "L5"], true)]));
}

#[test]
fn test_text_on_the_other_screen_is_not_under_the_cursor() {
    let mut menu = Menu::new();
    assert_eq!(
        menu.heard_with(&[label_with(9, "MEDICINE", -70.0, 60.0, true)]),
        said(&[])
    );
}

#[test]
fn test_a_hidden_cursor_says_nothing_and_says_its_item_again_when_back() {
    let mut menu = Menu::new();
    menu.heard();
    menu.fake.visible(menu.window, false);
    assert_eq!(menu.heard(), said(&[]));
    menu.fake.visible(menu.window, true);
    assert_eq!(menu.heard(), said(&[(&["POKÉMON"], true)]));
}

#[test]
fn test_a_cursor_over_nothing_waits_for_an_item() {
    let mut menu = Menu::new();
    menu.r#move(0.0, -100.0);
    assert_eq!(menu.heard(), said(&[]));
    menu.r#move(80.0, 0.0);
    assert_eq!(menu.heard(), said(&[(&["SAVE"], true)]));
}

#[test]
fn test_a_window_that_is_not_alone_in_its_layout_is_not_a_cursor() {
    assert_eq!(Menu::with(2).heard(), said(&[]));
}

#[test]
fn test_a_piece_whose_place_is_unknown_is_not_under_the_cursor() {
    let mut menu = Menu::new();
    assert_eq!(
        menu.heard_with(&[ScreenText::new(1, "POKÉMON", false)]),
        said(&[])
    );
}

#[test]
fn test_what_is_under_the_cursor_is_known_after_a_poll() {
    let mut menu = Menu::new();
    menu.heard();
    let under: Vec<Vec<String>> = menu
        .reader
        .last
        .under
        .iter()
        .map(|pieces| pieces.iter().map(|p| p.text.clone()).collect())
        .collect();
    assert_eq!(under, vec![vec!["POKÉMON".to_string()]]);
}

/// The battle's command menu layout, its cursor's layout, and the command menu's layout file.
const COMMAND: i64 = 0x14600000;
const MARKER: i64 = 0x14B00000;
const FILE: i64 = 0x14500000;
/// A pane under the move button's group, standing in for its "Scratch" TextBox.
const SCRATCH: i64 = COMMAND + 0x3600;

/// The battle's command menu and its cursor, laid out as spike 50 read them: each button a
/// group holding an empty 10 by 10 pane and a picture, and the cursor a 20 by 20 picture alone in
/// a layout of its own, placed exactly on the chosen button's empty pane.
struct Battle {
    memory: Rc<FakeMemory>,
    picture_class: i64,
    fake: FakeLayouts,
    root: i64,
    marker: i64,
    reader: Reader,
}

impl Battle {
    fn new() -> Battle {
        Battle::with(true)
    }

    fn with(with_file: bool) -> Battle {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let picture_class = classes.add(lyt::PICTURE_CLASS);
        let fake = FakeLayouts::new(&memory, &classes);
        let (work, root) = fake.screen_with(COMMAND, false, false);
        fake.project(work, 320.0);
        let reader = reader_of(&fake, &memory);
        let mut battle = Battle {
            memory,
            picture_class,
            fake,
            root,
            marker: 0,
            reader,
        };
        battle.name(root, "RootPane");
        battle.button(COMMAND + 0x1000, "23", "2j", (39.0, 12.0), "16");
        battle.button(COMMAND + 0x2000, "20", "2k", (-128.0, -70.0), "v");
        battle.plate(COMMAND + 0x3000, "7", "30", (-95.0, 67.0));
        battle.fake.pane(SCRATCH, COMMAND + 0x3000);
        battle.name(SCRATCH, "9");
        battle.plate(COMMAND + 0x4000, "1g", "37", (95.0, 7.0));
        let (work, marker_root) = battle.fake.screen_with(MARKER, false, false);
        battle.fake.project(work, 320.0);
        battle.marker = MARKER + 0x800;
        battle
            .fake
            .pane_with_vtable(battle.marker, marker_root, Some(battle.picture_class));
        battle.size(20.0, 20.0);
        battle.r#move(39.0, 12.0);
        battle.fake.track().unwrap();
        let files = Rc::new(LayoutFiles::new(battle.memory.clone()));
        if with_file {
            battle.memory.write(
                FILE,
                &layout_archive(vec![
                    clyt::section(
                        b"txl1",
                        &clyt::names_table(&[
                            "fight_button.bclim",
                            "bag_button.bclim",
                            "waza_plate_00.bclim",
                        ]),
                    ),
                    clyt::section(
                        b"mat1",
                        &clyt::materials(&[
                            clyt::material("16", &[0]),
                            clyt::material("v", &[1]),
                            clyt::material("7", &[2]),
                            clyt::material("1g", &[2]),
                        ]),
                    ),
                    clyt::pane(b"pan1", "RootPane"),
                    clyt::pane(b"pan1", "23"),
                    clyt::pane(b"pan1", "2j"),
                    clyt::pane_with(b"pic1", "16", 0),
                    clyt::pane(b"pan1", "20"),
                    clyt::pane(b"pan1", "2k"),
                    clyt::pane_with(b"pic1", "v", 1),
                    clyt::pane_with(b"pic1", "7", 2),
                    clyt::pane(b"pan1", "30"),
                    clyt::pane(b"pan1", "9"),
                    clyt::pane_with(b"pic1", "1g", 3),
                    clyt::pane(b"pan1", "37"),
                ]),
            );
            files.add_archive(FILE).unwrap();
        }
        battle.reader = reader_with_files(&battle.fake, &battle.memory, &files);
        battle
    }

    fn button(&self, group: i64, name: &str, anchor: &str, at: (f64, f64), picture: &str) -> i64 {
        self.fake.pane(group, self.root);
        self.name(group, name);
        self.fake.pane(group + 0x200, group);
        self.name(group + 0x200, anchor);
        put(&self.memory, group + 0x200 + lyt::X, &[at.0]);
        put(&self.memory, group + 0x200 + lyt::Y, &[at.1]);
        self.fake
            .pane_with_vtable(group + 0x400, group, Some(self.picture_class));
        self.name(group + 0x400, picture);
        group
    }

    /// A move button as the move list builds it: a picture holding its empty pane.
    fn plate(&self, group: i64, name: &str, anchor: &str, at: (f64, f64)) {
        self.fake
            .pane_with_vtable(group, self.root, Some(self.picture_class));
        self.name(group, name);
        self.fake.pane(group + 0x200, group);
        self.name(group + 0x200, anchor);
        put(&self.memory, group + 0x200 + lyt::X, &[at.0]);
        put(&self.memory, group + 0x200 + lyt::Y, &[at.1]);
    }

    fn name(&self, pane: i64, name: &str) {
        write_name(&self.memory, pane, name);
    }

    fn size(&self, width: f64, height: f64) {
        put(&self.memory, self.marker + lyt::WIDTH, &[width, height]);
    }

    fn r#move(&self, x: f64, y: f64) {
        put(&self.memory, self.marker + lyt::X, &[x]);
        put(&self.memory, self.marker + lyt::Y, &[y]);
    }

    fn heard(&mut self) -> Vec<(Vec<String>, bool)> {
        self.heard_with(&[])
    }

    fn heard_with(&mut self, pieces: &[ScreenText]) -> Vec<(Vec<String>, bool)> {
        self.fake.layouts.refresh();
        let items = self.reader.poll(pieces).unwrap().items;
        heard_of(items)
    }
}

/// The command menu draws FIGHT as a picture textured `fight_button.bclim` (spike 50).
#[test]
fn test_a_picture_cursor_on_a_buttons_pane_says_the_words_of_its_picture() {
    assert_eq!(Battle::new().heard(), said(&[(&["fight"], true)]));
}

#[test]
fn test_moving_a_picture_cursor_says_the_new_button() {
    let mut battle = Battle::new();
    battle.heard();
    battle.r#move(-128.0, -70.0);
    assert_eq!(battle.heard(), said(&[(&["bag"], false)]));
}

#[test]
fn test_a_button_with_words_says_its_words_rather_than_its_picture() {
    let mut battle = Battle::new();
    battle.r#move(-95.0, 67.0);
    let scratch = ScreenText {
        root: COMMAND + 0x400,
        ..ScreenText::placed(SCRATCH, "Scratch", false, -110.0, -70.0, 61.0, 12.5)
    };
    assert_eq!(battle.heard_with(&[scratch]), said(&[(&["Scratch"], true)]));
}

/// The battle's Bag draws an empty slot dimmed with its times sign still in it, and the cursor
/// on it said "Times" (movie29, Orion, 2026-10-08).
#[test]
fn test_a_button_whose_words_have_no_letter_or_digit_is_empty() {
    let mut battle = Battle::new();
    battle.r#move(-95.0, 67.0);
    let sign = ScreenText {
        root: COMMAND + 0x400,
        ..ScreenText::placed(SCRATCH, "\u{d7}", false, -100.0, -92.0, 61.0, 16.0)
    };
    assert_eq!(battle.heard_with(&[sign]), said(&[(&["empty"], true)]));
}

/// The move list's empty fourth slot was read "waza plate 00" (battles.ctm replay).
#[test]
fn test_an_empty_plate_like_those_words_are_written_on_is_said_as_empty() {
    let mut battle = Battle::new();
    battle.r#move(95.0, 7.0);
    let scratch = ScreenText {
        root: COMMAND + 0x400,
        ..ScreenText::placed(SCRATCH, "Scratch", false, -110.0, -70.0, 61.0, 12.5)
    };
    assert_eq!(battle.heard_with(&[scratch]), said(&[(&["empty"], true)]));
}

/// A party screen with five empty slots names every one of them the same, and comparing words
/// alone said it once and then nothing however far the player arrowed, until they landed on a
/// Pokemon and came back (the user, 2026-09-16). Both plates here carry `waza_plate_00`.
#[test]
fn test_moving_between_two_items_that_read_alike_is_still_said() {
    let mut battle = Battle::new();
    battle.heard();
    battle.r#move(-95.0, 67.0);
    assert_eq!(battle.heard(), said(&[(&["plate 00"], false)]));
    battle.r#move(95.0, 7.0);
    assert_eq!(battle.heard(), said(&[(&["plate 00"], false)]));
}

/// The place is compared as well as the words, so a cursor that has not moved stays quiet.
#[test]
fn test_a_cursor_standing_still_is_not_said_twice() {
    let mut battle = Battle::new();
    battle.heard();
    battle.r#move(-95.0, 67.0);
    assert_eq!(battle.heard(), said(&[(&["plate 00"], false)]));
    assert_eq!(battle.heard(), said(&[]));
}

/// Panes read mid-update sat 1,000 units off for a poll (spike 50).
#[test]
fn test_a_picture_cursor_lost_for_one_poll_is_not_said_again() {
    let mut battle = Battle::new();
    battle.heard();
    battle.r#move(39.0, -988.0);
    assert_eq!(battle.heard(), said(&[]));
    battle.r#move(39.0, 12.0);
    assert_eq!(battle.heard(), said(&[]));
}

#[test]
fn test_a_picture_cursor_gone_a_while_is_said_again_when_back() {
    let mut battle = Battle::new();
    battle.heard();
    battle.r#move(39.0, -988.0);
    battle.heard();
    battle.heard();
    battle.r#move(39.0, 12.0);
    assert_eq!(battle.heard(), said(&[(&["fight"], true)]));
}

#[test]
fn test_a_button_all_pictures_whose_layout_file_is_not_found_says_nothing() {
    assert_eq!(Battle::with(false).heard(), said(&[]));
}

#[test]
fn test_a_big_picture_alone_in_a_layout_is_not_a_cursor() {
    let mut battle = Battle::new();
    battle.size(320.0, 240.0);
    assert_eq!(battle.heard(), said(&[]));
}

const KEYBOARD: i64 = 0x08250000;
const KEYS: [(&str, f64, f64); 3] = [
    ("!", -130.0, 45.0),
    ("Q", -120.0, 10.0),
    ("W", -100.0, 10.0),
];
/// The keyboard's layout file, which names its keys' textures.
const KEYBOARD_FILE: i64 = 0x08280000;

/// The name entry's keyboard as spike 68 read it: each key a 20 by 30 picture holding its letter
/// and an `nw::lyt::Bounding` touch area, and the cursor a picture the game moves into the chosen
/// key's touch area, all in one layout.
struct Keyboard {
    memory: Rc<FakeMemory>,
    picture_class: i64,
    fake: FakeLayouts,
    root: i64,
    plates: IndexMap<String, i64>,
    touch: IndexMap<String, i64>,
    cursor: i64,
    second: i64,
    reader: Reader,
}

impl Keyboard {
    fn new() -> Keyboard {
        Keyboard::with(lyt::PICTURE_CLASS, false, false, false)
    }

    fn with(plate_class: &str, icon: bool, confirm: bool, second: bool) -> Keyboard {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let picture_class = classes.add(lyt::PICTURE_CLASS);
        let bounding_class = classes.add(lyt::BOUNDING_CLASS);
        let plate_vtable = if plate_class == lyt::PICTURE_CLASS {
            Some(picture_class)
        } else {
            None
        };
        let fake = FakeLayouts::new(&memory, &classes);
        let (work, root) = fake.screen_with(KEYBOARD, false, false);
        fake.project(work, 320.0);
        let mut plates: IndexMap<String, i64> = IndexMap::new();
        let mut touch: IndexMap<String, i64> = IndexMap::new();
        for (index, (letter, _x, _y)) in KEYS.iter().enumerate() {
            let plate = KEYBOARD + 0x1000 + index as i64 * 0x800;
            fake.pane_with_vtable(plate, root, plate_vtable);
            fake.pane_with_vtable(plate + 0x400, plate, Some(bounding_class));
            plates.insert(letter.to_string(), plate);
            touch.insert(letter.to_string(), plate + 0x400);
        }
        if confirm {
            // A key drawn without words, as the one that confirms a name is.
            let plate = KEYBOARD + 0x4000;
            fake.pane_with_vtable(plate, root, Some(picture_class));
            fake.pane_with_vtable(plate + 0x400, plate, Some(bounding_class));
            plates.insert("ok".to_string(), plate);
            touch.insert("ok".to_string(), plate + 0x400);
        }
        let reader = reader_of(&fake, &memory);
        let mut keyboard = Keyboard {
            memory,
            picture_class,
            fake,
            root,
            plates,
            touch,
            cursor: 0,
            second: 0,
            reader,
        };
        keyboard.cursor = keyboard.picture(KEYBOARD + 0x8000, "!");
        if icon {
            keyboard.picture(KEYBOARD + 0x8400, "W");
        }
        if second {
            // A second picture textured as a cursor, under the root and over Q, as the name
            // entry's layout keeps one (spike 96).
            keyboard.second = KEYBOARD + 0x8800;
            keyboard.fake.pane_with_vtable(
                keyboard.second,
                keyboard.root,
                Some(keyboard.picture_class),
            );
            put(
                &keyboard.memory,
                keyboard.second + lyt::WIDTH,
                &[20.0, 20.0],
            );
            put(
                &keyboard.memory,
                keyboard.second + lyt::SCALE_X,
                &[1.0, 0.0],
            );
            put(&keyboard.memory, keyboard.second + lyt::SCALE_Y, &[1.0]);
            put(&keyboard.memory, keyboard.second + lyt::X, &[-120.0, 0.0]);
            put(&keyboard.memory, keyboard.second + lyt::Y, &[10.0]);
        }
        keyboard.name(keyboard.root, "RootPane");
        for (index, (plate, touch)) in keyboard
            .plates
            .values()
            .zip(keyboard.touch.values())
            .enumerate()
        {
            keyboard.name(*plate, &format!("k{index}"));
            keyboard.name(*touch, &format!("b{index}"));
        }
        keyboard.name(keyboard.cursor, "cur");
        if second {
            keyboard.name(keyboard.second, "cur2");
        }
        keyboard.fake.track().unwrap();
        let files = Rc::new(LayoutFiles::new(keyboard.memory.clone()));
        if confirm {
            let mut sections = vec![
                clyt::section(
                    b"txl1",
                    &clyt::names_table(&["key_plate.bclim", "ok_button.bclim", "cursor.bclim"]),
                ),
                clyt::section(
                    b"mat1",
                    &clyt::materials(&[
                        clyt::material("k0", &[0]),
                        clyt::material("k1", &[0]),
                        clyt::material("k2", &[0]),
                        clyt::material("k3", &[1]),
                        clyt::material("cur", &[2]),
                    ]),
                ),
                clyt::pane(b"pan1", "RootPane"),
                clyt::pane_with(b"pic1", "k0", 0),
                clyt::pane(b"bnd1", "b0"),
                clyt::pane_with(b"pic1", "cur", 4),
                clyt::pane_with(b"pic1", "k1", 1),
                clyt::pane(b"bnd1", "b1"),
                clyt::pane_with(b"pic1", "k2", 2),
                clyt::pane(b"bnd1", "b2"),
                clyt::pane_with(b"pic1", "k3", 3),
                clyt::pane(b"bnd1", "b3"),
            ];
            if second {
                sections.push(clyt::pane_with(b"pic1", "cur2", 4));
            }
            keyboard
                .memory
                .write(KEYBOARD_FILE, &layout_archive(sections));
            files.add_archive(KEYBOARD_FILE).unwrap();
        }
        keyboard.reader = reader_with_files(&keyboard.fake, &keyboard.memory, &files);
        keyboard
    }

    fn name(&self, pane: i64, name: &str) {
        write_name(&self.memory, pane, name);
    }

    fn picture(&self, at: i64, letter: &str) -> i64 {
        self.fake
            .pane_with_vtable(at, self.touch[letter], Some(self.picture_class));
        at
    }

    fn r#move(&self, cursor: i64, letter: &str) {
        self.memory.u32(cursor + lyt::PARENT, &[self.touch[letter]]);
    }

    fn heard(&mut self) -> Vec<(Vec<String>, bool)> {
        self.fake.layouts.refresh();
        let pieces: Vec<ScreenText> = KEYS
            .iter()
            .enumerate()
            .map(|(index, &(letter, x, y))| ScreenText {
                root: self.root,
                parent: self.plates[letter],
                panel: self.plates[letter],
                ..ScreenText::placed(
                    0x9000 + index as i64,
                    letter,
                    false,
                    x - 3.0,
                    x + 3.0,
                    y,
                    12.5,
                )
            })
            .collect();
        heard_of(self.reader.poll(&pieces).unwrap().items)
    }
}

/// The name entry keeps a second picture textured as a cursor, and it said the key it lay on,
/// "“", beside the key cursor's "!" as the keyboard opened (opening.ctm replay, spike 96).
#[test]
fn test_a_keyboards_other_cursor_picture_is_left_to_its_key_cursor() {
    let mut keyboard = Keyboard::with(lyt::PICTURE_CLASS, false, true, true);
    assert_eq!(keyboard.heard(), said(&[(&["!"], true)]));
    assert_eq!(keyboard.reader.last.lists.clone(), IndexMap::new());
}

/// The name entry's keys were read a row at a time as the keyboard opened, and the key under
/// the cursor never (opening.ctm, the user, 2026-09-17).
#[test]
fn test_a_keyboard_says_the_key_its_cursor_is_in() {
    assert_eq!(Keyboard::new().heard(), said(&[(&["!"], true)]));
}

/// The name entry's cursor is drawn with a texture named a cursor, and read as a named cursor
/// as well it was said again every poll and the keyboard read whole (opening.ctm, spike 95).
#[test]
fn test_a_key_cursor_textured_as_a_cursor_is_read_once_as_a_key_cursor() {
    let mut keyboard = Keyboard::with(lyt::PICTURE_CLASS, false, true, false);
    put(
        &keyboard.memory,
        keyboard.cursor + lyt::WIDTH,
        &[20.0, 20.0],
    );
    put(&keyboard.memory, keyboard.cursor + lyt::SCALE_X, &[1.0]);
    put(&keyboard.memory, keyboard.cursor + lyt::SCALE_Y, &[1.0]);
    put(&keyboard.memory, keyboard.cursor + lyt::X, &[-130.0, 0.0]);
    put(&keyboard.memory, keyboard.cursor + lyt::Y, &[45.0]);
    assert_eq!(keyboard.heard(), said(&[(&["!"], true)]));
    assert_eq!(keyboard.heard(), said(&[]));
    assert_eq!(keyboard.reader.last.lists.clone(), IndexMap::new());
}

#[test]
fn test_a_keyboard_cursor_moving_into_another_key_says_that_key() {
    let mut keyboard = Keyboard::new();
    keyboard.heard();
    keyboard.r#move(keyboard.cursor, "Q");
    assert_eq!(keyboard.heard(), said(&[(&["Q"], false)]));
    keyboard.r#move(keyboard.cursor, "W");
    assert_eq!(keyboard.heard(), said(&[(&["W"], false)]));
}

/// The name box beside the keyboard keeps pictures in a touch area under a plain pane.
#[test]
fn test_a_picture_in_a_touch_area_of_a_plain_pane_is_not_a_keys_cursor() {
    assert_eq!(
        Keyboard::with(lyt::PANE_CLASS, false, false, false).heard(),
        said(&[])
    );
}

/// The key that confirms a name is a picture with no words in it, and said nothing (the user,
/// 2026-09-18). Its texture names it, as a battle button's does.
#[test]
fn test_a_key_drawn_without_words_is_said_by_its_picture() {
    let mut keyboard = Keyboard::with(lyt::PICTURE_CLASS, false, true, false);
    keyboard.heard();
    keyboard.r#move(keyboard.cursor, "ok");
    assert_eq!(keyboard.heard(), said(&[(&["ok"], false)]));
}

/// An icon standing in a button's touch area is not a cursor; the cursor is the one that
/// moves.
#[test]
fn test_of_several_pictures_in_keys_only_one_that_moves_is_a_cursor() {
    let mut keyboard = Keyboard::with(lyt::PICTURE_CLASS, true, false, false);
    assert_eq!(keyboard.heard(), said(&[]));
    keyboard.r#move(keyboard.cursor, "Q");
    assert_eq!(keyboard.heard(), said(&[(&["Q"], true)]));
}

/// The main menu's layout, drawn on the top screen.
const MAIN_MENU: i64 = 0x082A0000;
const MAIN_MENU_ROOT: i64 = MAIN_MENU + 0x400;
const MAIN_MENU_FILE: i64 = 0x082B0000;

/// A piece of the main menu, on the top screen under its root, hanging from `panel`.
fn main_menu_piece(
    r#box: i64,
    text: &str,
    (left, right, row): (f64, f64, f64),
    panel: i64,
) -> ScreenText {
    ScreenText {
        top: true,
        root: MAIN_MENU_ROOT,
        panel,
        parent: panel,
        ..ScreenText::placed(r#box, text, false, left, right, row, 16.0)
    }
}

fn r#continue() -> Vec<ScreenText> {
    let panel = MAIN_MENU + 0x600;
    vec![
        main_menu_piece(0x1, "CONTINUE", (-160.0, -100.0, 89.0), panel),
        main_menu_piece(0x2, "Leos", (-20.0, 10.0, 66.0), panel),
        main_menu_piece(0x3, "Time:", (-90.0, -50.0, 0.0), panel),
        main_menu_piece(0x4, "0:35", (-20.0, 10.0, 0.0), panel),
    ]
}

fn rows() -> Vec<ScreenText> {
    vec![
        main_menu_piece(0x5, "MYSTERY GIFT", (-90.0, 0.0, -32.0), MAIN_MENU + 0x1000),
        main_menu_piece(
            0x6,
            "LIVE COMPETITION",
            (-90.0, 20.0, -62.0),
            MAIN_MENU + 0x1400,
        ),
    ]
}

/// The main menu as spike 72 read it: CONTINUE's panel and the rows below it under one root,
/// and the cursor a window directly under that root, moved and resized onto the chosen item. A
/// second window directly under the root, hidden, is the frame of a dialog the menu can put up.
struct MainMenu {
    memory: Rc<FakeMemory>,
    fake: FakeLayouts,
    cursor: i64,
    other: i64,
    reader: Reader,
}

impl MainMenu {
    fn new() -> MainMenu {
        MainMenu::with(false, None)
    }

    fn with(second_shown: bool, frame: Option<&str>) -> MainMenu {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let window_class = classes.add(lyt::WINDOW_CLASS);
        let fake = FakeLayouts::new(&memory, &classes);
        let (work, panel) = fake.screen_with(MAIN_MENU, true, true);
        fake.project(work, 400.0);
        fake.pane(MAIN_MENU + 0x1000, MAIN_MENU_ROOT);
        fake.pane(MAIN_MENU + 0x1400, MAIN_MENU_ROOT);
        let cursor = MAIN_MENU + 0x2000;
        fake.pane_with_vtable(cursor, MAIN_MENU_ROOT, Some(window_class));
        let other = MAIN_MENU + 0x2400;
        fake.pane_with_vtable(other, MAIN_MENU_ROOT, Some(window_class));
        fake.visible(other, second_shown);
        let reader = reader_of(&fake, &memory);
        let mut menu = MainMenu {
            memory,
            fake,
            cursor,
            other,
            reader,
        };
        menu.place(menu.other, 0.0, 0.0, 310.0, 220.0);
        menu.memory.u32(MAIN_MENU_ROOT + lyt::CHILD_COUNT, &[5]);
        for window in [menu.cursor, menu.other] {
            for offset in [lyt::SCALE_X, lyt::SCALE_Y] {
                put(&menu.memory, window + offset, &[1.0]);
            }
        }
        menu.place(menu.cursor, 0.0, 48.0, 336.0, 140.0);
        let names = [
            ("RootPane", MAIN_MENU_ROOT),
            ("p", panel),
            ("r1", MAIN_MENU + 0x1000),
            ("r2", MAIN_MENU + 0x1400),
            ("1u", menu.cursor),
            ("2y", menu.other),
        ];
        for (name, pane) in names {
            write_name(&menu.memory, pane, name);
        }
        menu.fake.track().unwrap();
        let files = Rc::new(LayoutFiles::new(menu.memory.clone()));
        if let Some(frame) = frame {
            // The layout's file, naming what its two windows are drawn with (spike 96).
            menu.memory.write(
                MAIN_MENU_FILE,
                &layout_archive(vec![
                    clyt::section(b"txl1", &clyt::names_table(&[frame, "msgwin_c.bclim"])),
                    clyt::section(
                        b"mat1",
                        &clyt::materials(&[
                            clyt::material("1uLT", &[0]),
                            clyt::material("2yC", &[1]),
                        ]),
                    ),
                    clyt::pane(b"pan1", "RootPane"),
                    clyt::pane(b"pan1", "p"),
                    clyt::pane(b"pan1", "r1"),
                    clyt::pane(b"pan1", "r2"),
                    clyt::window("1u", 0, &[0]),
                    clyt::window("2y", 1, &[]),
                ]),
            );
            files.add_archive(MAIN_MENU_FILE).unwrap();
        }
        menu.reader = reader_with_files(&menu.fake, &menu.memory, &files);
        menu.reader.track([menu.cursor, menu.other]).unwrap();
        menu
    }

    fn place(&self, window: i64, x: f64, y: f64, width: f64, height: f64) {
        for (offset, value) in [
            (lyt::X, x),
            (lyt::Y, y),
            (lyt::WIDTH, width),
            (lyt::HEIGHT, height),
        ] {
            put(&self.memory, window + offset, &[value]);
        }
    }

    fn heard(&mut self) -> Vec<(String, bool)> {
        self.fake.layouts.refresh();
        let pieces: Vec<ScreenText> = r#continue().into_iter().chain(rows()).collect();
        self.reader
            .poll(&pieces)
            .unwrap()
            .items
            .iter()
            .map(|item| (join(&item.pieces, false), item.opened))
            .collect()
    }
}

/// The main menu's cursor is a window among the menu's own panes, not alone in a layout of its
/// own, and the menu was read as any screen with nothing said as the cursor moved (main_menu.ctm,
/// spike 72). A frame that never moves is a panel's border, so the item is heard from the first
/// move on: the poll after the cursor has come to rest on it, and at once after that.
#[test]
fn test_a_frame_in_the_menus_own_layout_is_a_cursor_once_it_has_moved() {
    let mut menu = MainMenu::new();
    assert_eq!(menu.heard(), lines(&[]));
    assert_eq!(menu.heard(), lines(&[]));
    menu.place(menu.cursor, 0.0, -33.0, 336.0, 43.0);
    assert_eq!(menu.heard(), lines(&[]));
    assert_eq!(menu.heard(), lines(&[("MYSTERY GIFT", true)]));
    menu.place(menu.cursor, 0.0, -63.0, 336.0, 43.0);
    assert_eq!(menu.heard(), lines(&[("LIVE COMPETITION", false)]));
}

/// The main menu was read whole as it opened, CONTINUE's panel and every row below it, and
/// only its moves one at a time (pokedex2.ctm, the user, 2026-09-24). Its frame is drawn with
/// `cursor_lt.bclim`, so it is a cursor from the first poll (spike 96).
#[test]
fn test_a_frame_drawn_as_a_cursor_says_its_item_as_the_menu_opens() {
    let mut menu = MainMenu::with(false, Some("cursor_lt.bclim"));
    assert_eq!(menu.heard(), lines(&[("CONTINUE Leos Time: 0:35", true)]));
    assert_eq!(menu.reader.last.lists.clone(), IndexMap::new());
    menu.place(menu.cursor, 0.0, -33.0, 336.0, 43.0);
    assert_eq!(menu.heard(), lines(&[("MYSTERY GIFT", false)]));
}

#[test]
fn test_a_frame_drawn_otherwise_waits_to_move_as_before() {
    let mut menu = MainMenu::with(false, Some("title_bar1_lt.bclim"));
    assert_eq!(menu.heard(), lines(&[]));
}

#[test]
fn test_a_frame_resized_back_onto_a_panel_says_the_whole_panel() {
    let mut menu = MainMenu::new();
    menu.heard();
    menu.heard();
    menu.place(menu.cursor, 0.0, -33.0, 336.0, 43.0);
    menu.heard();
    menu.heard();
    menu.place(menu.cursor, 0.0, 48.0, 336.0, 140.0);
    assert_eq!(menu.heard(), lines(&[("CONTINUE Leos Time: 0:35", false)]));
}

/// The language screen scrolls by moving its rows (spike 70); rows moving as one are not a
/// cursor moving among them.
#[test]
fn test_frames_that_move_together_are_a_list_scrolling_and_no_cursor() {
    let mut menu = MainMenu::with(true, None);
    assert_eq!(menu.heard(), lines(&[]));
    assert_eq!(menu.heard(), lines(&[]));
    menu.place(menu.cursor, 0.0, 18.0, 336.0, 140.0);
    menu.place(menu.other, 0.0, -30.0, 310.0, 220.0);
    assert_eq!(menu.heard(), lines(&[]));
    assert_eq!(menu.heard(), lines(&[]));
}

/// A frame somewhere new every poll until it stops is a screen opening, not a cursor moving:
/// taken for one, it would make a menu of the screen it frames and leave that screen's words
/// unsaid.
#[test]
fn test_a_frame_sliding_into_place_is_not_a_cursor() {
    let mut menu = MainMenu::new();
    for y in [200.0, 150.0, 100.0] {
        menu.place(menu.cursor, 0.0, y, 336.0, 140.0);
        assert_eq!(menu.heard(), lines(&[]));
    }
    menu.place(menu.cursor, 0.0, 48.0, 336.0, 140.0);
    assert_eq!(menu.heard(), lines(&[]));
    assert_eq!(menu.heard(), lines(&[]));
    assert_eq!(menu.heard(), lines(&[]));
}

#[test]
fn test_a_frame_leaving_its_rest_without_resting_again_is_not_a_cursor() {
    let mut menu = MainMenu::new();
    menu.heard();
    menu.heard();
    for y in [100.0, 150.0] {
        menu.place(menu.cursor, 0.0, y, 336.0, 140.0);
        assert_eq!(menu.heard(), lines(&[]));
    }
    menu.fake.visible(menu.cursor, false);
    assert_eq!(menu.heard(), lines(&[]));
}

#[test]
fn test_a_frame_hidden_for_one_poll_is_still_the_cursor() {
    let mut menu = MainMenu::new();
    menu.heard();
    menu.heard();
    menu.place(menu.cursor, 0.0, -33.0, 336.0, 43.0);
    menu.heard();
    menu.heard();
    menu.fake.visible(menu.cursor, false);
    assert_eq!(menu.heard(), lines(&[]));
    menu.fake.visible(menu.cursor, true);
    menu.place(menu.cursor, 0.0, -63.0, 336.0, 43.0);
    assert_eq!(menu.heard(), lines(&[("LIVE COMPETITION", false)]));
}

const CHOOSER: i64 = 0x08265000;
const CHOOSER_FILE: i64 = 0x08290000;
const BOY: (f64, f64) = (-80.0, 0.0);
const GIRL: (f64, f64) = (80.0, 0.0);

/// The opening's choice of boy or girl as spike 71 read it: two groups side by side under one
/// pane, each a white backing, a portrait and a frame of its own, and the cursor a window beside
/// them that the game slides from one group's place onto the other's, all in one layout.
struct Chooser {
    memory: Rc<FakeMemory>,
    fake: FakeLayouts,
    root: i64,
    groups: IndexMap<String, i64>,
    words_box: Option<i64>,
    cursor: i64,
    reader: Reader,
}

impl Chooser {
    fn new() -> Chooser {
        Chooser::with(false, IndexMap::new())
    }

    fn with(words_in_girl: bool, textures: IndexMap<String, String>) -> Chooser {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let picture = classes.add(lyt::PICTURE_CLASS);
        let window = classes.add(lyt::WINDOW_CLASS);
        let fake = FakeLayouts::new(&memory, &classes);
        let (work, root) = fake.screen_with(CHOOSER, true, false);
        fake.project(work, 400.0);
        let holder = CHOOSER + 0x1000;
        fake.pane(holder, root);
        let reader = reader_of(&fake, &memory);
        let mut chooser = Chooser {
            memory,
            fake,
            root,
            groups: IndexMap::new(),
            words_box: None,
            cursor: 0,
            reader,
        };
        let mut panes: Vec<(String, i64)> =
            vec![("RootPane".to_string(), root), ("k".to_string(), holder)];
        for (index, (side, at)) in [("boy", BOY), ("girl", GIRL)].into_iter().enumerate() {
            let index = index as i64;
            let group = CHOOSER + 0x2000 + index * 0x1000;
            chooser.fake.pane(group, holder);
            chooser.place(group, at);
            chooser
                .fake
                .pane_with_vtable(group + 0x200, group, Some(picture));
            chooser
                .fake
                .pane_with_vtable(group + 0x400, group, Some(picture));
            chooser
                .fake
                .pane_with_vtable(group + 0x600, group, Some(window));
            chooser.place(group + 0x600, at);
            chooser.groups.insert(side.to_string(), group);
            panes.extend([
                (format!("g{index}"), group),
                (format!("w{index}"), group + 0x200),
                (format!("p{index}"), group + 0x400),
                (format!("f{index}"), group + 0x600),
            ]);
        }
        chooser.words_box = if words_in_girl {
            Some(chooser.groups["girl"] + 0x800)
        } else {
            None
        };
        if let Some(words_box) = chooser.words_box {
            chooser.fake.pane(words_box, chooser.groups["girl"]);
            panes.push(("t1".to_string(), words_box));
        }
        chooser.cursor = CHOOSER + 0x4000;
        chooser
            .fake
            .pane_with_vtable(chooser.cursor, holder, Some(window));
        chooser.place(chooser.cursor, BOY);
        panes.push(("cur".to_string(), chooser.cursor));
        for (name, pane) in &panes {
            write_name(&chooser.memory, *pane, name);
        }
        chooser.fake.track().unwrap();
        let mut sections = vec![
            clyt::section(
                b"txl1",
                &clyt::names_table(&["1pix_wht.bclim", "intro_male.bclim", "intro_famale.bclim"]),
            ),
            clyt::section(
                b"mat1",
                &clyt::materials(&[
                    clyt::material("w0", &[0]),
                    clyt::material("p0", &[1]),
                    clyt::material("w1", &[0]),
                    clyt::material("p1", &[2]),
                ]),
            ),
        ];
        for (name, _pane) in &panes {
            let first = name.chars().next().unwrap();
            let kind: &[u8] = if "wp".contains(first) {
                b"pic1"
            } else if "fc".contains(first) {
                b"wnd1"
            } else {
                b"pan1"
            };
            let material = match name.as_str() {
                "w0" => 0,
                "p0" => 1,
                "w1" => 2,
                "p1" => 3,
                _ => 0,
            };
            sections.push(clyt::pane_with(kind, name, material));
        }
        chooser
            .memory
            .write(CHOOSER_FILE, &layout_archive(sections));
        let files = Rc::new(LayoutFiles::new(chooser.memory.clone()));
        files.add_archive(CHOOSER_FILE).unwrap();
        chooser.reader = CursorReader::with_textures(
            chooser.fake.layouts.clone(),
            names_of(&chooser.memory),
            Some(files),
            false,
            None,
            IndexMap::new(),
            textures,
        )
        .into();
        chooser
    }

    fn place(&self, pane: i64, at: (f64, f64)) {
        put(&self.memory, pane + lyt::X, &[at.0]);
        put(&self.memory, pane + lyt::Y, &[at.1]);
    }

    fn heard(&mut self) -> Vec<(Vec<String>, bool)> {
        self.fake.layouts.refresh();
        let mut pieces: Vec<ScreenText> = Vec::new();
        if let Some(words_box) = self.words_box {
            pieces = vec![ScreenText {
                top: true,
                root: self.root,
                parent: self.groups["girl"],
                ..ScreenText::placed(words_box, "May", false, 70.0, 90.0, 0.0, 12.5)
            }];
        }
        heard_of(self.reader.poll(&pieces).unwrap().items)
    }
}

/// The choice of boy or girl said nothing (opening.ctm, the user, 2026-09-19). Each choice is
/// a portrait on a white backing; the backing is the same on both, the portrait is not.
#[test]
fn test_a_frame_on_a_choice_says_the_texture_only_that_choice_has() {
    assert_eq!(Chooser::new().heard(), said(&[(&["male"], true)]));
}

/// `intro_famale.bclim` is the game's own misspelling, and the spelling table says "female".
#[test]
fn test_a_frame_sliding_onto_the_other_choice_says_it_once_it_is_there() {
    let mut chooser = Chooser::new();
    chooser.heard();
    chooser.place(chooser.cursor, (-16.0, 0.0));
    assert_eq!(chooser.heard(), said(&[]));
    chooser.place(chooser.cursor, GIRL);
    assert_eq!(chooser.heard(), said(&[(&["female"], false)]));
    chooser.place(chooser.cursor, BOY);
    assert_eq!(chooser.heard(), said(&[(&["male"], false)]));
}

/// The user, 2026-10-15: the boy and the girl said as "Boy, ..." and "Girl, ...", as X's
/// are, not as "male" and "female".
#[test]
fn test_a_choice_whose_portrait_pa3ds_describes_says_the_description_whole() {
    let mut chooser = Chooser::with(
        false,
        IndexMap::from([
            (
                "intro_male".to_string(),
                "Boy, brown hair, black hat".to_string(),
            ),
            (
                "intro_famale".to_string(),
                "Girl, brown hair, red bandana".to_string(),
            ),
        ]),
    );
    assert_eq!(
        chooser.heard(),
        said(&[(&["Boy, brown hair, black hat"], true)])
    );
    chooser.place(chooser.cursor, GIRL);
    assert_eq!(
        chooser.heard(),
        said(&[(&["Girl, brown hair, red bandana"], false)])
    );
}

/// Each portrait has a frame of its own, at its group's place, that never moves.
#[test]
fn test_a_frame_round_its_own_choice_is_not_a_cursor() {
    let mut chooser = Chooser::new();
    chooser.fake.visible(chooser.cursor, false);
    assert_eq!(chooser.heard(), said(&[]));
}

#[test]
fn test_a_choice_with_words_says_its_words() {
    let mut chooser = Chooser::with(true, IndexMap::new());
    chooser.heard();
    chooser.place(chooser.cursor, GIRL);
    assert_eq!(chooser.heard(), said(&[(&["May"], false)]));
}

const DEX: i64 = 0x08270000;
const ROW_YS: [f64; 5] = [125.0, 95.0, 65.0, 35.0, 5.0];

/// The Pokédex list as spike 95 read it: rows of 160 by 30 directly under the root, one above
/// the other at x 60, each holding a number and a name, and a plate — a window with no panes of
/// its own, two panes down from the root — that stays on the row at y 5 while the rows scroll.
struct Dex {
    memory: Rc<FakeMemory>,
    fake: FakeLayouts,
    root: i64,
    rows: Vec<i64>,
    boxes: Vec<(i64, i64)>,
    plate: i64,
    words: Vec<String>,
    reader: Reader,
}

impl Dex {
    fn new() -> Dex {
        Dex::with(ROW_YS.len(), 40.0)
    }

    fn with(rows: usize, plate_height: f64) -> Dex {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let window = classes.add(lyt::WINDOW_CLASS);
        let textbox = classes.add(lyt::TEXTBOX_CLASS);
        let fake = FakeLayouts::new(&memory, &classes);
        let (work, root) = fake.screen_with(DEX, false, false);
        fake.project(work, 320.0);
        let reader = reader_of(&fake, &memory);
        let mut dex = Dex {
            memory,
            fake,
            root,
            rows: Vec::new(),
            boxes: Vec::new(),
            plate: 0,
            words: Vec::new(),
            reader,
        };
        for (index, y) in ROW_YS[..rows].iter().enumerate() {
            let row = DEX + 0x1000 + index as i64 * 0x1000;
            dex.fake.pane(row, dex.root);
            dex.shape(row, (60.0, *y), (160.0, 30.0));
            dex.fake.pane_with_vtable(row + 0x200, row, Some(textbox));
            dex.fake.pane_with_vtable(row + 0x400, row, Some(textbox));
            dex.rows.push(row);
            dex.boxes.push((row + 0x200, row + 0x400));
        }
        let holder = DEX + 0x8000;
        dex.fake.pane(holder, dex.root);
        dex.fake.pane(holder + 0x200, holder);
        dex.plate = holder + 0x400;
        dex.fake
            .pane_with_vtable(dex.plate, holder + 0x200, Some(window));
        dex.shape(dex.plate, (85.0, 5.0), (240.0, plate_height));
        dex.fake.track().unwrap();
        dex.words = [
            "080",
            "?????",
            "081",
            "?????",
            "082",
            "?????",
            "083",
            "?????",
            "084",
            "Magnemite",
        ][10 - 2 * rows..]
            .iter()
            .map(|word| word.to_string())
            .collect();
        dex.reader = reader_of(&dex.fake, &dex.memory);
        dex
    }

    fn shape(&self, pane: i64, at: (f64, f64), size: (f64, f64)) {
        put(&self.memory, pane + lyt::X, &[at.0]);
        put(&self.memory, pane + lyt::Y, &[at.1]);
        put(&self.memory, pane + lyt::WIDTH, &[size.0, size.1]);
        put(&self.memory, pane + lyt::SCALE_X, &[1.0]);
        put(&self.memory, pane + lyt::SCALE_Y, &[1.0]);
    }

    fn slide(&self, dy: f64) {
        for (row, y) in self.rows.iter().zip(ROW_YS) {
            put(&self.memory, row + lyt::Y, &[y + dy]);
        }
    }

    fn pieces(&self) -> Vec<ScreenText> {
        let mut out: Vec<ScreenText> = Vec::new();
        for (index, (number, name)) in self.boxes.iter().enumerate() {
            let y =
                binary::f32(&self.memory.read(self.rows[index] + lyt::Y, 4).unwrap(), 0).unwrap();
            out.push(ScreenText {
                root: self.root,
                parent: self.rows[index],
                ..ScreenText::placed(
                    *number,
                    self.words[2 * index].as_str(),
                    false,
                    10.0,
                    30.0,
                    y,
                    12.5,
                )
            });
            out.push(ScreenText {
                root: self.root,
                parent: self.rows[index],
                ..ScreenText::placed(
                    *name,
                    self.words[2 * index + 1].as_str(),
                    false,
                    40.0,
                    110.0,
                    y,
                    12.5,
                )
            });
        }
        out
    }

    fn heard(&mut self) -> Vec<(Vec<String>, bool)> {
        self.fake.layouts.refresh();
        let pieces = self.pieces();
        heard_of(self.reader.poll(&pieces).unwrap().items)
    }
}

/// The Pokédex read every row as the list scrolled and never the chosen one (pokedex.ctm, the
/// user, 2026-09-24): its plate never moves, and the rows move under it.
#[test]
fn test_a_plate_the_rows_scroll_under_says_the_row_it_sits_on() {
    let mut dex = Dex::new();
    assert_eq!(dex.heard(), said(&[]));
    assert_eq!(dex.heard(), said(&[(&["084", "Magnemite"], true)]));
    assert_eq!(dex.heard(), said(&[]));
}

/// Scrolling slides the rows a row down and back while their words are rewritten, and for
/// that poll the row under the plate holds words it is about to lose (spike 95).
#[test]
fn test_rows_scrolling_under_a_plate_say_the_new_row_once_it_holds_still() {
    let mut dex = Dex::new();
    dex.heard();
    dex.heard();
    dex.slide(27.0);
    assert_eq!(dex.heard(), said(&[]));
    dex.slide(0.0);
    dex.words = [
        "079", "?????", "080", "?????", "081", "?????", "082", "?????", "083", "?????",
    ]
    .iter()
    .map(|word| word.to_string())
    .collect();
    assert_eq!(dex.heard(), said(&[]));
    assert_eq!(dex.heard(), said(&[(&["083", "?????"], false)]));
}

#[test]
fn test_a_plates_list_is_every_row_it_walks() {
    let mut dex = Dex::new();
    dex.heard();
    assert_eq!(
        dex.reader.last.lists.clone(),
        lists_of(
            dex.root,
            dex.boxes.iter().flat_map(|&(number, name)| [number, name])
        )
    );
}

#[test]
fn test_a_window_as_tall_as_several_rows_is_no_plate() {
    let mut dex = Dex::with(ROW_YS.len(), 100.0);
    dex.heard();
    assert_eq!(dex.heard(), said(&[]));
    assert_eq!(dex.reader.last.lists.clone(), IndexMap::new());
}

#[test]
fn test_two_rows_are_not_a_list_for_a_plate() {
    let mut dex = Dex::with(2, 40.0);
    dex.heard();
    assert_eq!(dex.heard(), said(&[]));
}

#[test]
fn test_a_plate_that_leaves_the_screen_is_forgotten_and_said_again_when_back() {
    let mut dex = Dex::new();
    dex.heard();
    dex.heard();
    dex.fake.visible(dex.plate, false);
    assert_eq!(dex.heard(), said(&[]));
    assert_ne!(dex.reader.last.lists.clone(), IndexMap::new());
    assert_eq!(dex.heard(), said(&[]));
    assert_eq!(dex.reader.last.lists.clone(), IndexMap::new());
    dex.fake.visible(dex.plate, true);
    dex.heard();
    assert_eq!(dex.heard(), said(&[(&["084", "Magnemite"], true)]));
}

const MART: i64 = 0x08280000;
const MART_FILE: i64 = 0x082A0000;
const MART_ROWS: [(&str, &str, f64); 3] = [
    ("Poké Ball", "$ 200", 90.0),
    ("Potion", "$ 300", 70.0),
    ("Quit", "", 30.0),
];

/// The Poké Mart's list as spike 95 read it, on the top screen: its words in a window under
/// the root, and the cursor a pane beside them holding the two halves of `shop_cursor.bclim`,
/// moved onto the chosen row. `party` puts the PC Box's party panel beside it, a picture holding
/// a touch area a slot and each the icon of the Pokémon in it.
struct Mart {
    memory: Rc<FakeMemory>,
    fake: FakeLayouts,
    root: i64,
    pieces: Vec<ScreenText>,
    cursor: i64,
    reader: Reader,
}

impl Mart {
    fn new() -> Mart {
        Mart::with("shop_cursor.bclim", 0)
    }

    fn with(texture: &str, party: i64) -> Mart {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let window = classes.add(lyt::WINDOW_CLASS);
        let picture = classes.add(lyt::PICTURE_CLASS);
        let bounding = classes.add(lyt::BOUNDING_CLASS);
        let fake = FakeLayouts::new(&memory, &classes);
        let (work, root) = fake.screen_with(MART, true, false);
        fake.project(work, 400.0);
        let list = MART + 0x1000;
        fake.pane_with_vtable(list, root, Some(window));
        let mut panes: Vec<(String, i64)> =
            vec![("RootPane".to_string(), root), ("2c".to_string(), list)];
        let mut pieces: Vec<ScreenText> = Vec::new();
        for (index, (item, price, y)) in MART_ROWS.iter().enumerate() {
            let index = index as i64;
            for (column, (words, x)) in [(item, -55.0), (price, 137.0)].into_iter().enumerate() {
                let column = column as i64;
                let r#box = list + 0x200 * (2 * index + column + 1);
                fake.pane(r#box, list);
                panes.push((format!("t{}", 2 * index + column), r#box));
                if !words.is_empty() {
                    pieces.push(ScreenText {
                        top: true,
                        root,
                        parent: list,
                        ..ScreenText::placed(r#box, *words, false, x, x + 50.0, *y, 13.0)
                    });
                }
            }
        }
        let cursor = MART + 0x3000;
        fake.pane(cursor, list);
        put(&memory, cursor + lyt::WIDTH, &[280.0, 20.0]);
        put(&memory, cursor + lyt::SCALE_X, &[1.0]);
        put(&memory, cursor + lyt::SCALE_Y, &[1.0]);
        let reader = reader_of(&fake, &memory);
        let mart = Mart {
            memory,
            fake,
            root,
            pieces,
            cursor,
            reader,
        };
        mart.r#move(90.0);
        mart.fake
            .pane_with_vtable(mart.cursor + 0x200, mart.cursor, Some(picture));
        mart.fake
            .pane_with_vtable(mart.cursor + 0x400, mart.cursor, Some(picture));
        panes.extend([
            ("3z".to_string(), mart.cursor),
            ("3v".to_string(), mart.cursor + 0x200),
            ("3x".to_string(), mart.cursor + 0x400),
        ]);
        if party != 0 {
            let panel = MART + 0x5000;
            mart.fake.pane_with_vtable(panel, mart.root, Some(picture));
            panes.push(("pp".to_string(), panel));
            for slot in 0..party {
                let touch = panel + 0x400 * (slot + 1);
                mart.fake.pane_with_vtable(touch, panel, Some(bounding));
                mart.fake
                    .pane_with_vtable(touch + 0x200, touch, Some(picture));
                panes.extend([
                    (format!("pb{slot}"), touch),
                    (format!("pi{slot}"), touch + 0x200),
                ]);
            }
        }
        for (name, pane) in &panes {
            write_name(&mart.memory, *pane, name);
        }
        mart.fake.track().unwrap();
        let mut sections = vec![
            clyt::section(b"txl1", &clyt::names_table(&[texture, "poke_icon.bclim"])),
            clyt::section(
                b"mat1",
                &clyt::materials(&[
                    clyt::material("3v", &[0]),
                    clyt::material("3x", &[0]),
                    clyt::material("pi", &[1]),
                ]),
            ),
        ];
        for (name, _pane) in &panes {
            let kind: &[u8] =
                if ["3v", "3x", "pp"].contains(&name.as_str()) || name.starts_with("pi") {
                    b"pic1"
                } else if name.starts_with("pb") {
                    b"bnd1"
                } else if name == "2c" {
                    b"wnd1"
                } else {
                    b"pan1"
                };
            let material = if name == "3x" {
                1
            } else if name.starts_with('p') {
                2
            } else {
                0
            };
            sections.push(clyt::pane_with(kind, name, material));
        }
        mart.memory.write(MART_FILE, &layout_archive(sections));
        let files = Rc::new(LayoutFiles::new(mart.memory.clone()));
        files.add_archive(MART_FILE).unwrap();
        let reader = reader_with_files(&mart.fake, &mart.memory, &files);
        Mart { reader, ..mart }
    }

    fn r#move(&self, y: f64) {
        put(&self.memory, self.cursor + lyt::X, &[74.0]);
        put(&self.memory, self.cursor + lyt::Y, &[y]);
    }

    fn heard(&mut self) -> Vec<(Vec<String>, bool)> {
        self.fake.layouts.refresh();
        heard_of(self.reader.poll(&self.pieces).unwrap().items)
    }
}

const CONTEST: i64 = 0x08600000;
const CONTEST_FILE: i64 = 0x08620000;
const CONTEST_MOVES: [(&str, f64, f64); 4] = [
    ("Hold Back", -80.0, 10.0),
    ("Meteor Mash", 80.0, 10.0),
    ("Zen Headbutt", -80.0, -36.0),
    ("Iron Defense", 80.0, -36.0),
];

/// A contest's moves as spike 196 read them: a button a move, 148 by 43, holding an empty 20
/// by 20 pane near its top and its name below it, and `cursor.bclim` drawn on the chosen
/// button's empty pane.
struct Contest {
    memory: Rc<FakeMemory>,
    fake: FakeLayouts,
    root: i64,
    pieces: Vec<ScreenText>,
    cursor: i64,
    reader: Reader,
}

impl Contest {
    fn new() -> Contest {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let picture = classes.add(lyt::PICTURE_CLASS);
        let fake = FakeLayouts::new(&memory, &classes);
        let (work, root) = fake.screen_with(CONTEST, false, false);
        fake.project(work, 320.0);
        let place = |pane: i64, x: f64, y: f64, width: f64, height: f64| {
            Contest::place_in(&memory, pane, x, y, width, height)
        };
        let mut panes: Vec<(String, i64)> = vec![("RootPane".to_string(), root)];
        let mut pieces: Vec<ScreenText> = Vec::new();
        for (index, (r#move, x, y)) in CONTEST_MOVES.into_iter().enumerate() {
            let index = index as i64;
            let button = CONTEST + 0x1000 * (index + 1);
            fake.pane(button, root);
            place(button, x, y, 148.0, 43.0);
            fake.pane(button + 0x200, button);
            place(button + 0x200, x, y + 15.0, 20.0, 20.0);
            fake.pane(button + 0x400, button);
            panes.extend([
                (format!("b{index}"), button),
                (format!("s{index}"), button + 0x200),
                (format!("t{index}"), button + 0x400),
            ]);
            // Where spike 196 estimated the words: Hold Back, at -80, from -38.8 to 16.5.
            pieces.push(ScreenText {
                root,
                parent: button,
                ..ScreenText::placed(
                    button + 0x400,
                    r#move,
                    false,
                    x + 41.0,
                    x + 97.0,
                    y + 10.0,
                    12.5,
                )
            });
        }
        let cursor = CONTEST + 0x8000;
        fake.pane_with_vtable(cursor, root, Some(picture));
        place(cursor, -80.0, 25.0, 16.0, 16.0);
        panes.push(("7c".to_string(), cursor));
        for (name, pane) in &panes {
            write_name(&memory, *pane, name);
        }
        fake.track().unwrap();
        let mut sections = vec![
            clyt::section(b"txl1", &clyt::names_table(&["cursor.bclim"])),
            clyt::section(b"mat1", &clyt::materials(&[clyt::material("7c", &[0])])),
        ];
        for (name, _pane) in &panes {
            let kind: &[u8] = if name == "7c" { b"pic1" } else { b"pan1" };
            sections.push(clyt::pane_with(kind, name, 0));
        }
        memory.write(CONTEST_FILE, &layout_archive(sections));
        let files = Rc::new(LayoutFiles::new(memory.clone()));
        files.add_archive(CONTEST_FILE).unwrap();
        let reader = reader_with_files(&fake, &memory, &files);
        Contest {
            memory,
            fake,
            root,
            pieces,
            cursor,
            reader,
        }
    }

    fn place_in(memory: &FakeMemory, pane: i64, x: f64, y: f64, width: f64, height: f64) {
        put(memory, pane + lyt::X, &[x]);
        put(memory, pane + lyt::Y, &[y]);
        put(memory, pane + lyt::WIDTH, &[width, height]);
        put(memory, pane + lyt::SCALE_X, &[1.0]);
        put(memory, pane + lyt::SCALE_Y, &[1.0]);
    }

    fn place(&self, pane: i64, x: f64, y: f64, width: f64, height: f64) {
        Contest::place_in(&self.memory, pane, x, y, width, height);
    }

    fn heard(&mut self) -> Vec<(Vec<String>, bool)> {
        self.fake.layouts.refresh();
        heard_of(self.reader.poll(&self.pieces).unwrap().items)
    }
}

/// A contest's moves read as "Hold Back: Meteor Mash, Zen Headbutt: Iron Defense", and the
/// cursor between them said nothing (movie30, Orion, 2026-10-08): it sits on an empty pane at
/// the top of the chosen move's button, its name below it. Its list is all four buttons.
#[test]
fn test_a_cursor_picture_on_an_empty_pane_says_its_buttons_words() {
    let mut contest = Contest::new();
    assert_eq!(contest.heard(), said(&[(&["Hold Back"], true)]));
    contest.place(contest.cursor, 80.0, -21.0, 16.0, 16.0);
    assert_eq!(contest.heard(), said(&[(&["Iron Defense"], false)]));
    assert_eq!(
        contest.reader.last.lists.clone(),
        lists_of(contest.root, contest.pieces.iter().map(|p| p.r#box))
    );
}

/// The Mart read its whole list, and at each move only the count in the bag and the
/// description (shop.ctm, the user, 2026-09-24). Its cursor is `shop_cursor.bclim`.
#[test]
fn test_a_picture_textured_as_a_cursor_says_the_row_it_is_on() {
    assert_eq!(
        Mart::new().heard(),
        said(&[(&["Poké Ball", "$ 200"], true)])
    );
}

/// Up from the top of the list wraps round to Quit, which has no price.
#[test]
fn test_a_named_cursor_moving_says_the_row_it_lands_on() {
    let mut mart = Mart::new();
    mart.heard();
    mart.r#move(30.0);
    assert_eq!(mart.heard(), said(&[(&["Quit"], false)]));
}

#[test]
fn test_a_named_cursors_list_is_the_words_beside_those_it_is_over() {
    let mut mart = Mart::new();
    mart.heard();
    assert_eq!(
        mart.reader.last.lists.clone(),
        lists_of(mart.root, mart.pieces.iter().map(|p| p.r#box))
    );
}

/// The PC's Box draws its cursor with `cursol01.bclim`, and its menu for a chosen Pokémon
/// was said whole and its cursor never followed (or/movie6_pc, the user, 2026-09-27).
#[test]
fn test_a_picture_textured_as_the_designers_spelt_cursor_is_one_too() {
    assert_eq!(
        Mart::with("cursol01.bclim", 0).heard(),
        said(&[(&["Poké Ball", "$ 200"], true)])
    );
}

/// The PC Box keeps each party Pokémon's icon in its slot's touch area, as the name entry
/// keeps its key cursor in a key's, and the Box was taken for a keyboard, where no named cursor
/// is looked for (or/movie6_pc, 2026-09-27). The name entry's key cursor is drawn as a cursor;
/// an icon is not, alone in the party or beside another.
#[test]
fn test_icons_in_touch_areas_do_not_make_a_named_cursors_layout_a_keyboard() {
    for party in [1, 2] {
        assert_eq!(
            Mart::with("cursol01.bclim", party).heard(),
            said(&[(&["Poké Ball", "$ 200"], true)]),
            "party {party}"
        );
    }
}

const BOX: i64 = 0x083A0000;
const BOX_FILE: i64 = 0x083C0000;
const BOX_ROWS: [&str; 5] = ["DEPOSIT", "SUMMARY", "MARKING", "RELEASE", "CANCEL"];

/// The PC Box's menu for a chosen Pokémon as or/movie6_pc traced it: each row a picture of its
/// own, `bar1.bclim`, 110 by 30 at x 105 and 30 apart, holding its word, and the cursor a plain
/// pane holding `cursol01.bclim`, which the game moves onto the chosen row.
struct BoxMenu {
    memory: Rc<FakeMemory>,
    fake: FakeLayouts,
    root: i64,
    pieces: Vec<ScreenText>,
    cursor: i64,
    reader: Reader,
}

impl BoxMenu {
    fn new() -> BoxMenu {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let picture = classes.add(lyt::PICTURE_CLASS);
        let fake = FakeLayouts::new(&memory, &classes);
        let (work, root) = fake.screen_with(BOX, false, false);
        fake.project(work, 320.0);
        let reader = reader_of(&fake, &memory);
        let mut menu = BoxMenu {
            memory,
            fake,
            root,
            pieces: Vec::new(),
            cursor: 0,
            reader,
        };
        let mut panes: Vec<(String, i64)> = vec![("RootPane".to_string(), root)];
        for (index, word) in BOX_ROWS.iter().enumerate() {
            let index = index as i64;
            let (bar, y) = (BOX + 0x1000 + 0x400 * index, 55.0 - 30.0 * index as f64);
            menu.fake.pane_with_vtable(bar, menu.root, Some(picture));
            menu.place(bar, 105.0, y, 110.0, 30.0);
            menu.fake.pane(bar + 0x200, bar);
            panes.extend([
                (format!("b{index}"), bar),
                (format!("t{index}"), bar + 0x200),
            ]);
            menu.pieces.push(ScreenText {
                root: menu.root,
                parent: bar,
                ..ScreenText::placed(bar + 0x200, *word, false, 75.0, 135.0, y, 13.0)
            });
        }
        menu.cursor = BOX + 0x3000;
        menu.fake.pane(menu.cursor, menu.root);
        menu.place(menu.cursor, 105.0, 55.0, 20.0, 30.0);
        menu.fake
            .pane_with_vtable(menu.cursor + 0x200, menu.cursor, Some(picture));
        panes.extend([
            ("9t".to_string(), menu.cursor),
            ("2i".to_string(), menu.cursor + 0x200),
        ]);
        for (name, pane) in &panes {
            write_name(&menu.memory, *pane, name);
        }
        menu.fake.track().unwrap();
        let mut sections = vec![
            clyt::section(
                b"txl1",
                &clyt::names_table(&["cursol01.bclim", "bar1.bclim"]),
            ),
            clyt::section(
                b"mat1",
                &clyt::materials(&[clyt::material("2i", &[0]), clyt::material("b", &[1])]),
            ),
        ];
        for (name, _pane) in &panes {
            let kind: &[u8] = if name == "2i" || name.starts_with('b') {
                b"pic1"
            } else {
                b"pan1"
            };
            sections.push(clyt::pane_with(
                kind,
                name,
                if name == "2i" { 0 } else { 1 },
            ));
        }
        menu.memory.write(BOX_FILE, &layout_archive(sections));
        let files = Rc::new(LayoutFiles::new(menu.memory.clone()));
        files.add_archive(BOX_FILE).unwrap();
        menu.reader = reader_with_files(&menu.fake, &menu.memory, &files);
        menu
    }

    fn place(&self, pane: i64, x: f64, y: f64, width: f64, height: f64) {
        put(&self.memory, pane + lyt::X, &[x]);
        put(&self.memory, pane + lyt::Y, &[y]);
        put(&self.memory, pane + lyt::WIDTH, &[width, height]);
        put(&self.memory, pane + lyt::SCALE_X, &[1.0]);
        put(&self.memory, pane + lyt::SCALE_Y, &[1.0]);
    }

    fn r#move(&self, x: f64, y: f64) {
        put(&self.memory, self.cursor + lyt::X, &[x]);
        put(&self.memory, self.cursor + lyt::Y, &[y]);
    }

    /// One of the loop's ticks between screen polls, with the pointer here.
    fn tick(&mut self, x: f64, y: f64) {
        self.r#move(x, y);
        self.reader.sample(&*self.memory).unwrap();
    }

    fn heard(&mut self) -> Vec<(Vec<String>, bool)> {
        self.fake.layouts.refresh();
        heard_of(self.reader.poll(&self.pieces).unwrap().items)
    }
}

#[test]
fn test_the_box_menu_says_the_row_its_cursor_rests_on_at_once() {
    let mut r#box = BoxMenu::new();
    assert_eq!(r#box.heard(), said(&[(&["DEPOSIT"], true)]));
    for _ in 0..3 {
        r#box.tick(105.0, 25.0);
    }
    assert_eq!(r#box.heard(), said(&[(&["SUMMARY"], false)]));
}

/// Each row of the Box's menu is a picture of its own, where the Mart's share one window, and
/// the four rows not under the cursor were said with it as the menu opened (or/movie6_pc).
#[test]
fn test_a_named_cursors_list_is_its_row_and_the_rows_drawn_like_it() {
    let mut r#box = BoxMenu::new();
    r#box.heard();
    assert_eq!(
        r#box.reader.last.lists.clone(),
        lists_of(r#box.root, r#box.pieces.iter().map(|p| p.r#box))
    );
}

/// Each tick reads where the sampled cursors stand: the Mart's cursor never glides.
#[test]
fn test_only_a_cursor_walking_rows_or_slots_drawn_apart_is_sampled() {
    let mut mart = Mart::new();
    mart.heard();
    assert!(!mart.reader.sampling());
    let mut r#box = BoxMenu::new();
    r#box.heard();
    assert!(r#box.reader.sampling());
}

/// The Box's cursor glides from row to row over several frames, through each row's own
/// place on the way, and a poll on the way said SUMMARY or RELEASE as it passed on the way from
/// CANCEL round to DEPOSIT (or/movie6_pc). A poll finding it on SUMMARY's place, a tick after it
/// stood elsewhere, is on its way.
#[test]
fn test_a_cursor_gliding_over_rows_drawn_alike_says_only_the_row_it_rests_on() {
    let mut r#box = BoxMenu::new();
    r#box.heard();
    for y in [-35.0, -5.0, 25.0] {
        r#box.tick(105.0, y);
    }
    assert_eq!(r#box.heard(), said(&[]));
    for _ in 0..3 {
        r#box.tick(105.0, -65.0);
    }
    assert_eq!(r#box.heard(), said(&[(&["CANCEL"], false)]));
}

/// Choosing DEPOSIT sends the Box's cursor to the grid of Pokémon a poll before the menu's
/// rows go, and it said "SUMMARY", the word on its row to its right (or/movie6_pc).
#[test]
fn test_a_named_cursor_once_over_words_does_not_point_beside_it() {
    let mut r#box = BoxMenu::new();
    r#box.heard();
    r#box.heard();
    r#box.r#move(-35.0, 25.0);
    assert_eq!(r#box.heard(), said(&[]));
    assert_eq!(r#box.heard(), said(&[]));
}

const GRID: i64 = 0x08400000;
const GRID_FILE: i64 = 0x08440000;
const BAR: i64 = 0x08480000;
const BAR_FILE: i64 = 0x08460000;

/// One pane of the grid's layout: its name, its address, the kind of pane its layout file says
/// it is, and the material that file draws it with.
type Entry = (String, i64, &'static [u8], i64);

/// The PC Box as spike 129 traced it, small: a grid of touch areas under the root titled by the
/// box's name above it, a party panel of touch areas in two columns the second 20 units lower, the
/// Pokémon icons exactly on the slots they are in and the one carried 10 units above its slot, a
/// mark's touch area beside its picture, a lone button's, and the pointer, `cursol01.bclim` in a
/// plain pane of its own, resting exactly on a touch area.
struct BoxGrid {
    shape: lyt::PaneShape,
    memory: Rc<FakeMemory>,
    fake: FakeLayouts,
    root: i64,
    names: Vec<Entry>,
    at: i64,
    mark: i64,
    cursor: i64,
    reader: Reader,
    title: ScreenText,
}

impl BoxGrid {
    fn new() -> BoxGrid {
        BoxGrid::with(false, lyt::ORAS)
    }

    /// `bar` adds the bottom bar many screens share, `touch_bar_lower`, holding a button
    /// `change_box.bclim` where the Box's own layout leaves an empty pane (spike 143, movie12).
    /// `shape` lays the panes out as that game's NintendoWare does.
    fn with(bar: bool, shape: lyt::PaneShape) -> BoxGrid {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let picture = classes.add(lyt::PICTURE_CLASS);
        let bounding = classes.add(lyt::BOUNDING_CLASS);
        let fake = FakeLayouts::with_shape(&memory, &classes, shape);
        let (work, root) = fake.screen_with(GRID, false, false);
        fake.project(work, 320.0);
        let reader = reader_of(&fake, &memory);
        let mut grid = BoxGrid {
            shape,
            memory,
            fake,
            root,
            names: vec![("RootPane".to_string(), root, &b"pan1"[..], 0)],
            at: GRID + 0x1000,
            mark: 0,
            cursor: 0,
            reader,
            title: ScreenText::new(0, "", false),
        };
        for row in 0..2 {
            for column in 0..3 {
                grid.add(
                    &format!("s{row}{column}"),
                    root,
                    Some(bounding),
                    b"bnd1",
                    0,
                    -130.0 + 30.0 * column as f64,
                    55.0 - 30.0 * row as f64,
                    40.0,
                    30.0,
                );
            }
        }
        let panel = grid.add(
            "pp",
            root,
            Some(picture),
            b"pic1",
            3,
            105.0,
            0.0,
            110.0,
            160.0,
        );
        for (slot, (x, y)) in [(80.0, 55.0), (130.0, 35.0), (80.0, 15.0), (130.0, -5.0)]
            .into_iter()
            .enumerate()
        {
            grid.add(
                &format!("p{slot}"),
                panel,
                Some(bounding),
                b"bnd1",
                0,
                x,
                y,
                50.0,
                40.0,
            );
        }
        for (name, x, y) in [
            ("i0", 80.0, 55.0),
            ("i1", -100.0, 25.0),
            ("ih", -130.0, 65.0),
        ] {
            grid.add(name, root, Some(picture), b"pic1", 1, x, y, 40.0, 30.0);
        }
        let marks = grid.add(
            "mk",
            root,
            Some(picture),
            b"pic1",
            3,
            105.0,
            -50.0,
            110.0,
            30.0,
        );
        grid.add(
            "mt",
            marks,
            Some(bounding),
            b"bnd1",
            0,
            80.0,
            -50.0,
            40.0,
            25.0,
        );
        grid.mark = grid.add(
            "mp",
            marks,
            Some(picture),
            b"pic1",
            2,
            78.0,
            -50.0,
            10.0,
            10.0,
        );
        grid.fake.fade(grid.mark, 127);
        // The summary's: a touch area hanging from its mark's picture (movie22).
        let hung = grid.add(
            "sm",
            root,
            Some(picture),
            b"pic1",
            2,
            -85.0,
            -95.0,
            10.0,
            10.0,
        );
        grid.add(
            "st",
            hung,
            Some(bounding),
            b"bnd1",
            0,
            -85.0,
            -95.0,
            10.0,
            10.0,
        );
        let button = grid.add(
            "bt",
            root,
            Some(picture),
            b"pic1",
            3,
            0.0,
            120.0,
            60.0,
            20.0,
        );
        grid.add(
            "bb",
            button,
            Some(bounding),
            b"bnd1",
            0,
            0.0,
            120.0,
            60.0,
            20.0,
        );
        grid.cursor = grid.add("9t", root, None, b"pan1", 0, 80.0, 55.0, 20.0, 30.0);
        grid.add(
            "2i",
            grid.cursor,
            Some(picture),
            b"pic1",
            0,
            82.0,
            70.0,
            20.0,
            30.0,
        );
        for (name, pane, _kind, _material) in &grid.names {
            write_name_in(&grid.memory, *pane, name, &grid.shape);
        }
        let mut bar_walked: Vec<Entry> = Vec::new();
        if bar {
            let (bar_work, bar_root) = grid.fake.screen_with(BAR, false, false);
            grid.fake.project(bar_work, 320.0);
            let grid_names = std::mem::replace(
                &mut grid.names,
                vec![("RootPane".to_string(), bar_root, &b"pan1"[..], 0)],
            );
            let group = grid.add("q", bar_root, None, b"pan1", 0, -110.0, -105.0, 80.0, 30.0);
            grid.add(
                "o",
                group,
                Some(picture),
                b"pic1",
                0,
                -110.0,
                -105.0,
                80.0,
                30.0,
            );
            grid.add(
                "r",
                group,
                Some(bounding),
                b"bnd1",
                0,
                -110.0,
                -105.0,
                80.0,
                30.0,
            );
            for (name, pane, _kind, _material) in &grid.names {
                write_name_in(&grid.memory, *pane, name, &grid.shape);
            }
            bar_walked = grid.walked();
            grid.names = grid_names;
        }
        grid.fake.track().unwrap();
        let mut sections = vec![
            clyt::section(
                b"txl1",
                &clyt::names_table(&[
                    "cursol01.bclim",
                    "poke_icon.bclim",
                    "box_mark_04.bclim",
                    "box1.bclim",
                ]),
            ),
            clyt::section(
                b"mat1",
                &clyt::materials(
                    &(0..4)
                        .map(|i| clyt::material(&format!("m{i}"), &[i]))
                        .collect::<Vec<_>>(),
                ),
            ),
        ];
        for (name, _pane, kind, material) in grid.walked() {
            sections.push(clyt::pane_with(kind, &name, material));
        }
        grid.memory.write(GRID_FILE, &layout_archive(sections));
        let files = Rc::new(LayoutFiles::new(grid.memory.clone()));
        files.add_archive(GRID_FILE).unwrap();
        if bar {
            let mut sections = vec![
                clyt::section(b"txl1", &clyt::names_table(&["change_box.bclim"])),
                clyt::section(b"mat1", &clyt::materials(&[clyt::material("m0", &[0])])),
            ];
            for (name, _pane, kind, material) in &bar_walked {
                sections.push(clyt::pane_with(kind, name, *material));
            }
            grid.memory.write(BAR_FILE, &layout_archive(sections));
            files.add_archive(BAR_FILE).unwrap();
        }
        grid.reader = reader_with_files(&grid.fake, &grid.memory, &files);
        grid.title = ScreenText {
            root: grid.root,
            parent: grid.root,
            ..ScreenText::placed(GRID + 0xF000, "BOX 1", false, -80.0, -30.0, 90.0, 13.0)
        };
        grid
    }

    #[allow(clippy::too_many_arguments)]
    fn add(
        &mut self,
        name: &str,
        parent: i64,
        vtable: Option<i64>,
        kind: &'static [u8],
        material: i64,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> i64 {
        let pane = self.at;
        self.at += 0x200;
        self.fake.pane_with_vtable(pane, parent, vtable);
        put(&self.memory, pane + self.shape.x, &[x]);
        put(&self.memory, pane + self.shape.y, &[y]);
        put(&self.memory, pane + self.shape.width, &[width, height]);
        put(&self.memory, pane + self.shape.scale_x, &[1.0]);
        put(&self.memory, pane + self.shape.scale_y, &[1.0]);
        self.names.push((name.to_string(), pane, kind, material));
        pane
    }

    /// The panes in the order their layout's file lists them: depth first, as built.
    fn walked(&self) -> Vec<Entry> {
        let mut children: IndexMap<i64, Vec<Entry>> = IndexMap::new();
        for entry in &self.names[1..] {
            let parent =
                binary::u32(&self.memory.read(entry.1 + lyt::PARENT, 4).unwrap(), 0).unwrap();
            children.entry(parent).or_default().push(entry.clone());
        }
        let mut out: Vec<Entry> = Vec::new();

        fn walk(entry: &Entry, children: &IndexMap<i64, Vec<Entry>>, out: &mut Vec<Entry>) {
            out.push(entry.clone());
            if let Some(below) = children.get(&entry.1) {
                for child in below {
                    walk(child, children, out);
                }
            }
        }

        walk(&self.names[0], &children, &mut out);
        out
    }

    /// Move the pointer and poll twice: a slot is said once the pointer has stayed.
    fn rest(&mut self, x: f64, y: f64) -> Vec<(Vec<String>, bool)> {
        self.rest_with(x, y, true)
    }

    /// Without `words`, the Box's words are all off screen, as under its box list.
    fn rest_with(&mut self, x: f64, y: f64, words: bool) -> Vec<(Vec<String>, bool)> {
        put(&self.memory, self.cursor + self.shape.x, &[x]);
        put(&self.memory, self.cursor + self.shape.y, &[y]);
        let mut heard: Vec<(Vec<String>, bool)> = Vec::new();
        for _ in 0..2 {
            self.fake.layouts.refresh();
            let pieces = if words {
                vec![self.title.clone()]
            } else {
                Vec::new()
            };
            heard.extend(heard_of(self.reader.poll(&pieces).unwrap().items));
        }
        heard
    }
}

#[test]
fn test_a_party_slot_is_said_by_its_number() {
    assert_eq!(
        BoxGrid::new().rest(80.0, 55.0),
        said(&[(&["Party 1"], true)])
    );
}

/// Sun keeps a pane's place, size and name elsewhere than ORAS (spike 112).
#[test]
fn test_a_slot_is_read_through_its_game_s_pane_shape() {
    assert_eq!(
        BoxGrid::with(false, SUN_SHAPE).rest(80.0, 55.0),
        said(&[(&["Party 1"], true)])
    );
}

#[test]
fn test_an_empty_slot_is_said_empty() {
    let mut r#box = BoxGrid::new();
    r#box.rest(80.0, 55.0);
    assert_eq!(
        r#box.rest(130.0, 35.0),
        said(&[(&["Party 2, empty"], false)])
    );
}

#[test]
fn test_a_box_slot_is_said_by_the_box_s_name_its_row_and_its_column() {
    let mut r#box = BoxGrid::new();
    r#box.rest(80.0, 55.0);
    assert_eq!(
        r#box.rest(-100.0, 25.0),
        said(&[(&["BOX 1, row 2, column 2"], false)])
    );
}

/// The carried Pokémon's icon rides 10 units above the slot the pointer rests on (spike 129).
#[test]
fn test_the_pokemon_carried_over_a_slot_does_not_fill_it() {
    let mut r#box = BoxGrid::new();
    r#box.rest(80.0, 55.0);
    assert_eq!(
        r#box.rest(-130.0, 55.0),
        said(&[(&["BOX 1, row 1, column 1, empty"], false)])
    );
}

#[test]
fn test_an_empty_slot_brings_no_details_and_a_full_one_does() {
    let mut r#box = BoxGrid::new();
    r#box.rest(80.0, 55.0);
    put(&r#box.memory, r#box.cursor + lyt::X, &[130.0]);
    put(&r#box.memory, r#box.cursor + lyt::Y, &[35.0]);
    let mut quiet: Vec<bool> = Vec::new();
    for _ in 0..2 {
        r#box.fake.layouts.refresh();
        quiet.extend(
            r#box
                .reader
                .poll(&[r#box.title.clone()])
                .unwrap()
                .items
                .iter()
                .map(|item| item.quiet),
        );
    }
    assert_eq!(quiet, vec![true]);
}

/// What the grid's moves change is the grid's details, not the menu's: the summary of the
/// Pokémon under the pointer was read again after every row of the Box's menu (spike 129).
#[test]
fn test_each_group_of_slots_is_a_cursor_of_its_own() {
    let mut r#box = BoxGrid::new();
    let mut groups: Vec<i64> = Vec::new();
    for (x, y) in [(80.0, 55.0), (-100.0, 25.0)] {
        put(&r#box.memory, r#box.cursor + lyt::X, &[x]);
        put(&r#box.memory, r#box.cursor + lyt::Y, &[y]);
        for _ in 0..2 {
            r#box.fake.layouts.refresh();
            groups.extend(
                r#box
                    .reader
                    .poll(&[r#box.title.clone()])
                    .unwrap()
                    .items
                    .iter()
                    .map(|item| item.cursor),
            );
        }
    }
    assert_eq!(groups, vec![r#box.names[7].1, r#box.root]);
}

/// From Party 1 into the box and back, Party 1 went unsaid: the party had said it last.
#[test]
fn test_coming_back_to_the_slot_a_group_last_said_says_it_again() {
    let mut r#box = BoxGrid::new();
    r#box.rest(80.0, 55.0);
    r#box.rest(-100.0, 25.0);
    assert_eq!(r#box.rest(80.0, 55.0), said(&[(&["Party 1"], false)]));
}

/// The top screen shows the carried Pokémon's summary over every slot, and carrying Umbra over
/// Blaze's slot said "Party 1" and Umbra's summary (or/movie9_pc_organize_screens).
#[test]
fn test_a_slot_under_a_pokemon_carried_is_said_without_the_summary() {
    let mut r#box = BoxGrid::new();
    let carried = r#box
        .names
        .iter()
        .find(|(name, _pane, _kind, _material)| name == "ih")
        .unwrap()
        .1;
    put(&r#box.memory, carried + lyt::X, &[80.0]);
    put(&r#box.memory, carried + lyt::Y, &[65.0]);
    let mut heard: Vec<(String, bool)> = Vec::new();
    for _ in 0..2 {
        r#box.fake.layouts.refresh();
        heard.extend(
            r#box
                .reader
                .poll(&[r#box.title.clone()])
                .unwrap()
                .items
                .iter()
                .map(|item| (item.pieces[0].text.clone(), item.quiet)),
        );
    }
    assert_eq!(heard, lines(&[("Party 1", true)]));
}

#[test]
fn test_a_pointer_on_its_way_to_a_slot_is_moving_with_its_groups() {
    let mut r#box = BoxGrid::new();
    r#box.rest(80.0, 55.0);
    for (x, y) in [(-40.0, 40.0), (-100.0, 25.0)] {
        put(&r#box.memory, r#box.cursor + lyt::X, &[x]);
        put(&r#box.memory, r#box.cursor + lyt::Y, &[y]);
        r#box.reader.sample(&*r#box.memory).unwrap();
    }
    r#box.fake.layouts.refresh();
    assert_eq!(
        r#box.reader.poll(&[r#box.title.clone()]).unwrap().items,
        Vec::<CursorItem>::new()
    );
    let party = r#box.names[7].1;
    assert_eq!(
        r#box.reader.last.moving.clone(),
        HashSet::from([r#box.cursor, party, r#box.root])
    );
    for _ in 0..3 {
        r#box.reader.sample(&*r#box.memory).unwrap();
    }
    r#box.fake.layouts.refresh();
    let texts: Vec<String> = r#box
        .reader
        .poll(&[r#box.title.clone()])
        .unwrap()
        .items
        .iter()
        .map(|item| item.pieces[0].text.clone())
        .collect();
    assert_eq!(texts, vec!["BOX 1, row 2, column 2".to_string()]);
    assert_eq!(r#box.reader.last.moving.clone(), HashSet::new());
}

/// A mark not set is drawn half-transparent, and setting ▲ drew it solid
/// (or/movie11_pokedex_and_box).
#[test]
fn test_a_mark_is_said_by_its_sign_and_whether_it_is_set() {
    let mut r#box = BoxGrid::new();
    r#box.rest(80.0, 55.0);
    assert_eq!(r#box.rest(80.0, -50.0), said(&[(&["♥, off"], false)]));
    r#box.fake.fade(r#box.mark, 255);
    assert_eq!(r#box.rest(80.0, -50.0), said(&[(&["♥, on"], false)]));
    // The summary hangs each mark's touch area from its picture (movie22).
    assert_eq!(r#box.rest(-85.0, -95.0), said(&[(&["♥, on"], false)]));
}

/// Down from the grid the Box's pointer rests on the bottom bar's `change_box.bclim`, in a
/// layout many screens share, where the Box's own leaves an empty pane (spike 143, movie12).
#[test]
fn test_the_bottom_bars_button_under_the_pointer_is_said_by_its_picture() {
    let mut r#box = BoxGrid::with(true, lyt::ORAS);
    r#box.rest(80.0, 55.0);
    assert_eq!(
        r#box.rest(-110.0, -105.0),
        said(&[(&["change box"], false)])
    );
}

/// The box list hides every word of the Box's layout while its pointer walks the list and the
/// bottom bar: on "change box" it said nothing; and read as the Box's slots, the list's folders
/// said "row 2, column 1, empty" where the screen above says the box's name (movie12, spike
/// 143).
#[test]
fn test_with_the_boxs_words_hidden_its_pointer_says_only_another_layouts_button() {
    let mut r#box = BoxGrid::with(true, lyt::ORAS);
    r#box.rest(80.0, 55.0);
    assert_eq!(r#box.rest_with(-100.0, 55.0, false), said(&[]));
    assert_eq!(
        r#box.rest_with(-110.0, -105.0, false),
        said(&[(&["change box"], false)])
    );
}

#[test]
fn test_a_lone_buttons_touch_area_is_no_slot() {
    let mut r#box = BoxGrid::new();
    r#box.rest(80.0, 55.0);
    assert_eq!(r#box.rest(0.0, 120.0), said(&[]));
}

/// `cursor` is a word of the texture's name, not any run of its letters.
#[test]
fn test_a_picture_with_any_other_texture_is_no_cursor() {
    assert_eq!(Mart::with("shop_cursors_bg.bclim", 0).heard(), said(&[]));
}

const CONFIRM: i64 = 0x08340000;
const CONFIRM_FILE: i64 = 0x08360000;

/// The Poké Mart's "Is that OK?" as spike 98 read it: a window alone under its root holding
/// "Yes" and "No" one above the other, and a picture textured `cursor.bclim` left of them, which
/// the game moves from one row's height to the other's.
struct Confirm {
    memory: Rc<FakeMemory>,
    fake: FakeLayouts,
    root: i64,
    r#box: i64,
    pieces: Vec<ScreenText>,
    cursor: i64,
    reader: Reader,
}

impl Confirm {
    fn new() -> Confirm {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let window = classes.add(lyt::WINDOW_CLASS);
        let picture = classes.add(lyt::PICTURE_CLASS);
        let fake = FakeLayouts::new(&memory, &classes);
        let (work, root) = fake.screen_with(CONFIRM, false, false);
        fake.project(work, 320.0);
        memory.u32(root + lyt::CHILD_COUNT, &[1]);
        let r#box = CONFIRM + 0x1000;
        fake.pane_with_vtable(r#box, root, Some(window));
        memory.u32(r#box + lyt::CHILD_COUNT, &[3]);
        put(&memory, r#box + lyt::WIDTH, &[100.0, 60.0]);
        put(&memory, r#box + lyt::SCALE_X, &[1.0]);
        put(&memory, r#box + lyt::SCALE_Y, &[1.0]);
        put(&memory, r#box + lyt::X, &[139.0]);
        let mut pieces: Vec<ScreenText> = Vec::new();
        let mut panes: Vec<(String, i64)> =
            vec![("RootPane".to_string(), root), ("0".to_string(), r#box)];
        for (index, (words, y)) in [("Yes", 10.0), ("No", -10.0)].into_iter().enumerate() {
            let text = r#box + 0x200 * (index as i64 + 1);
            fake.pane(text, r#box);
            panes.push((format!("t{index}"), text));
            pieces.push(ScreenText {
                root,
                parent: r#box,
                panel: r#box,
                ..ScreenText::placed(text, words, false, 127.0, 151.0, y, 12.5)
            });
        }
        let cursor = CONFIRM + 0x2000;
        fake.pane_with_vtable(cursor, r#box, Some(picture));
        put(&memory, cursor + lyt::WIDTH, &[16.0, 16.0]);
        put(&memory, cursor + lyt::SCALE_X, &[1.0]);
        put(&memory, cursor + lyt::SCALE_Y, &[1.0]);
        let reader = reader_of(&fake, &memory);
        let mut confirm = Confirm {
            memory,
            fake,
            root,
            r#box,
            pieces,
            cursor,
            reader,
        };
        confirm.point(10.0);
        panes.push(("5".to_string(), confirm.cursor));
        for (name, pane) in &panes {
            write_name(&confirm.memory, *pane, name);
        }
        confirm.fake.track().unwrap();
        let mut sections = vec![
            clyt::section(
                b"txl1",
                &clyt::names_table(&["cursor.bclim", "menu_win_lu.bclim"]),
            ),
            clyt::section(b"mat1", &clyt::materials(&[clyt::material("5", &[0])])),
        ];
        for (name, _pane) in &panes {
            let kind: &[u8] = if name == "5" {
                b"pic1"
            } else if name == "0" {
                b"wnd1"
            } else {
                b"pan1"
            };
            sections.push(clyt::pane_with(kind, name, 0));
        }
        confirm
            .memory
            .write(CONFIRM_FILE, &layout_archive(sections));
        let files = Rc::new(LayoutFiles::new(confirm.memory.clone()));
        files.add_archive(CONFIRM_FILE).unwrap();
        confirm.reader = reader_with_files(&confirm.fake, &confirm.memory, &files);
        confirm.reader.track([confirm.r#box]).unwrap();
        confirm
    }

    fn point(&self, y: f64) {
        put(&self.memory, self.cursor + lyt::X, &[113.0]);
        put(&self.memory, self.cursor + lyt::Y, &[y]);
    }

    fn heard(&mut self) -> Vec<(Vec<String>, bool)> {
        self.fake.layouts.refresh();
        heard_of(self.reader.poll(&self.pieces).unwrap().items)
    }
}

/// Read as an app menu's cursor, the Mart's box of Yes and No said "Yes No" as it opened and
/// nothing as the player moved (movie5, the user, 2026-09-25). A cursor's corners hold nothing.
#[test]
fn test_a_window_holding_panes_of_its_own_is_a_box_and_no_cursor() {
    let mut menu = Menu::new();
    menu.memory.u32(menu.window + lyt::CHILD_COUNT, &[3]);
    assert_eq!(menu.heard(), said(&[]));
}

#[test]
fn test_a_cursor_picture_beside_the_words_says_the_row_it_points_at() {
    assert_eq!(Confirm::new().heard(), said(&[(&["Yes"], true)]));
}

#[test]
fn test_a_cursor_picture_beside_the_words_moving_says_the_row_it_lands_on() {
    let mut confirm = Confirm::new();
    confirm.heard();
    confirm.point(-10.0);
    assert_eq!(confirm.heard(), said(&[(&["No"], false)]));
}

/// The Mart's count draws arrows above and below it over the list behind, and taken for
/// pointers they said the list's prices every poll as they bobbed (omega_ruby_shop.ctm).
#[test]
fn test_a_cursor_picture_points_at_no_words_of_another_panel() {
    let mut confirm = Confirm::new();
    confirm.pieces = confirm
        .pieces
        .iter()
        .map(|piece| ScreenText {
            panel: confirm.root,
            ..piece.clone()
        })
        .collect();
    assert_eq!(confirm.heard(), said(&[]));
}

#[test]
fn test_a_cursor_picture_beside_the_words_walks_them_as_its_list() {
    let mut confirm = Confirm::new();
    confirm.heard();
    assert_eq!(
        confirm.reader.last.lists.clone(),
        lists_of(confirm.root, confirm.pieces.iter().map(|p| p.r#box))
    );
}

const TUTORIAL: i64 = 0x08230000;
const MOVES: i64 = 0x14730000;
const BEHIND: i64 = 0x14610000;
const MOVE_BUTTONS: [(&str, f64, f64); 3] = [
    ("Tackle", -80.0, 50.0),
    ("Growl", 80.0, 50.0),
    ("Tail Whip", -80.0, -10.0),
];

/// Wally's catching tutorial as spike 96 read it: the move list's buttons, each holding its
/// words and a touch area, and his pointer a small picture in a layout of its own, under a pane
/// placed on the button he is about to tap.
struct Tutorial {
    memory: Rc<FakeMemory>,
    bounding: i64,
    fake: FakeLayouts,
    pieces: Vec<ScreenText>,
    holder: i64,
    pointer: i64,
    reader: Reader,
}

impl Tutorial {
    fn new() -> Tutorial {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let picture = classes.add(lyt::PICTURE_CLASS);
        let bounding = classes.add(lyt::BOUNDING_CLASS);
        let fake = FakeLayouts::new(&memory, &classes);
        let reader = reader_of(&fake, &memory);
        let mut tutorial = Tutorial {
            memory,
            bounding,
            fake,
            pieces: Vec::new(),
            holder: 0,
            pointer: 0,
            reader,
        };
        let _moves = tutorial.buttons(MOVES, &MOVE_BUTTONS);
        let (work, pointer_root) = tutorial.fake.screen_with(TUTORIAL, false, false);
        tutorial.fake.project(work, 320.0);
        tutorial.holder = TUTORIAL + 0x1000;
        tutorial.fake.pane(tutorial.holder, pointer_root);
        tutorial.pointer = TUTORIAL + 0x1400;
        tutorial
            .fake
            .pane_with_vtable(tutorial.pointer, tutorial.holder, Some(picture));
        put(
            &tutorial.memory,
            tutorial.pointer + lyt::WIDTH,
            &[60.0, 60.0],
        );
        tutorial.fake.track().unwrap();
        tutorial.reader = reader_of(&tutorial.fake, &tutorial.memory);
        tutorial
    }

    /// A layout of buttons at `base`, each words and a 150 by 60 touch area. Returns its
    /// root.
    fn buttons(&mut self, base: i64, buttons: &[(&str, f64, f64)]) -> i64 {
        let (work, root) = self.fake.screen_with(base, false, false);
        self.fake.project(work, 320.0);
        for (index, (words, x, y)) in buttons.iter().enumerate() {
            let button = base + 0x1000 + index as i64 * 0x1000;
            self.fake.pane(button, root);
            let touch = button + 0x200;
            self.fake
                .pane_with_vtable(touch, button, Some(self.bounding));
            put(&self.memory, touch + lyt::WIDTH, &[150.0, 60.0]);
            put(&self.memory, touch + lyt::SCALE_X, &[1.0]);
            put(&self.memory, touch + lyt::SCALE_Y, &[1.0]);
            put(&self.memory, touch + lyt::X, &[*x]);
            put(&self.memory, touch + lyt::Y, &[*y]);
            let r#box = button + 0x400;
            self.fake.pane(r#box, button);
            self.pieces.push(ScreenText {
                root,
                parent: button,
                panel: button,
                ..ScreenText::placed(r#box, *words, false, x - 30.0, x + 30.0, *y, 12.5)
            });
        }
        root
    }

    fn point(&self, x: f64, y: f64) {
        put(&self.memory, self.holder + lyt::X, &[x]);
        put(&self.memory, self.holder + lyt::Y, &[y]);
    }

    fn heard(&mut self) -> Vec<(Vec<String>, bool)> {
        self.fake.layouts.refresh();
        heard_of(self.reader.poll(&self.pieces).unwrap().items)
    }
}

/// Wally's catching tutorial read his four moves out whole, and never the one he chose
/// (wally_again.ctm, the user, 2026-09-24): the tutorial has no battle cursor, only his pointer.
#[test]
fn test_a_pointer_says_the_button_it_is_about_to_tap() {
    let mut tutorial = Tutorial::new();
    tutorial.point(-50.0, 65.0);
    assert_eq!(tutorial.heard(), said(&[]));
    let under: Vec<String> = tutorial
        .reader
        .last
        .under
        .iter()
        .flat_map(|pieces| pieces.iter().map(|p| p.text.clone()))
        .collect();
    assert_eq!(under, vec!["Tackle".to_string()]);
    assert_eq!(tutorial.heard(), said(&[(&["Tackle"], true)]));
    assert_eq!(tutorial.heard(), said(&[]));
}

#[test]
fn test_a_pointer_that_leaves_and_comes_back_is_said_again() {
    let mut tutorial = Tutorial::new();
    tutorial.point(-50.0, 65.0);
    tutorial.heard();
    tutorial.heard();
    tutorial.fake.visible(tutorial.pointer, false);
    assert_eq!(tutorial.heard(), said(&[]));
    tutorial.fake.visible(tutorial.pointer, true);
    tutorial.point(80.0, 40.0);
    tutorial.heard();
    assert_eq!(tutorial.heard(), said(&[(&["Growl"], true)]));
}

#[test]
fn test_a_pointer_over_no_touch_area_says_nothing() {
    let mut tutorial = Tutorial::new();
    tutorial.point(0.0, 110.0);
    tutorial.heard();
    assert_eq!(tutorial.heard(), said(&[]));
    assert_eq!(
        tutorial.reader.last.under.clone(),
        Vec::<Vec<ScreenText>>::new()
    );
}

/// The bag's LAST USED ITEM stood under Wally's tap on USE, its screen still up behind the
/// one USE is on (wally_again.ctm replay).
#[test]
fn test_a_pointer_takes_the_screen_that_came_last_over_the_one_behind_it() {
    let mut tutorial = Tutorial::new();
    tutorial.point(-50.0, 65.0);
    tutorial.heard();
    tutorial.heard();
    tutorial.buttons(BEHIND, &[("USE", -60.0, 60.0)]);
    tutorial.fake.track().unwrap();
    tutorial.heard();
    assert_eq!(tutorial.heard(), said(&[(&["USE"], false)]));
}

#[test]
fn test_a_big_picture_placed_by_a_pane_is_no_pointer() {
    let mut tutorial = Tutorial::new();
    put(
        &tutorial.memory,
        tutorial.pointer + lyt::WIDTH,
        &[320.0, 240.0],
    );
    tutorial.point(-50.0, 65.0);
    tutorial.heard();
    assert_eq!(tutorial.heard(), said(&[]));
}

const BAG: i64 = 0x08500000;
const BAG_FILE: i64 = 0x08520000;

/// The Bag's party as a TM is taught, as spike 139 traced it: each slot a plain pane holding
/// whether its Pokémon can learn the move, and `select_bar.bclim`, a picture under the root
/// holding the chosen Pokémon's name and its own ABLE!, moved onto the chosen slot.
struct BagParty {
    memory: Rc<FakeMemory>,
    fake: FakeLayouts,
    root: i64,
    bar: i64,
    reader: Reader,
    pieces: Vec<ScreenText>,
}

impl BagParty {
    fn new() -> BagParty {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let picture = classes.add(lyt::PICTURE_CLASS);
        let fake = FakeLayouts::new(&memory, &classes);
        let (work, root) = fake.screen_with(BAG, false, false);
        fake.project(work, 320.0);
        let (slot, bar) = (BAG + 0x1000, BAG + 0x2000);
        fake.pane(slot, root);
        fake.pane(slot + 0x200, slot);
        fake.pane_with_vtable(bar, root, Some(picture));
        fake.pane(bar + 0x200, bar);
        fake.pane(bar + 0x400, bar);
        put(&memory, bar + lyt::WIDTH, &[160.0, 40.0]);
        put(&memory, bar + lyt::SCALE_X, &[1.0]);
        put(&memory, bar + lyt::SCALE_Y, &[1.0]);
        let reader = reader_of(&fake, &memory);
        let mut bag = BagParty {
            memory,
            fake,
            root,
            bar,
            reader,
            pieces: Vec::new(),
        };
        bag.r#move(-81.0);
        let panes = [
            ("RootPane", root),
            ("5f", slot),
            ("66", slot + 0x200),
            ("4w", bar),
            ("4y", bar + 0x200),
            ("7t", bar + 0x400),
        ];
        for (name, pane) in panes {
            write_name(&bag.memory, pane, name);
        }
        bag.fake.track().unwrap();
        let mut sections = vec![
            clyt::section(b"txl1", &clyt::names_table(&["select_bar.bclim"])),
            clyt::section(b"mat1", &clyt::materials(&[clyt::material("4w", &[0])])),
        ];
        for (name, _pane) in panes {
            sections.push(clyt::pane_with(
                if name == "4w" { b"pic1" } else { b"pan1" },
                name,
                0,
            ));
        }
        bag.memory.write(BAG_FILE, &layout_archive(sections));
        let files = Rc::new(LayoutFiles::new(bag.memory.clone()));
        files.add_archive(BAG_FILE).unwrap();
        bag.reader = reader_with_files(&bag.fake, &bag.memory, &files);
        bag.pieces = bag.chosen("Wingull");
        bag.pieces.push(ScreenText {
            root: bag.root,
            parent: slot,
            ..ScreenText::placed(slot + 0x200, "ABLE!", false, -71.0, -31.0, 28.0, 13.0)
        });
        bag
    }

    fn chosen(&self, name: &str) -> Vec<ScreenText> {
        vec![
            ScreenText {
                root: self.root,
                parent: self.bar,
                ..ScreenText::placed(self.bar + 0x200, name, false, -121.0, -81.0, 41.0, 13.0)
            },
            ScreenText {
                root: self.root,
                parent: self.bar,
                ..ScreenText::placed(self.bar + 0x400, "ABLE!", false, -71.0, -31.0, 23.0, 13.0)
            },
        ]
    }

    fn r#move(&self, x: f64) {
        put(&self.memory, self.bar + lyt::X, &[x, 30.0]);
    }

    fn heard(&mut self) -> Vec<(Vec<String>, bool)> {
        self.fake.layouts.refresh();
        heard_of(self.reader.poll(&self.pieces).unwrap().items)
    }
}

/// The select bar's name was never said, and the slot's own ABLE! under it lies nearer its
/// middle; its level is written a poll after its name (omega_ruby_hm-tm.ctm, spike 139).
#[test]
fn test_a_named_cursor_holding_words_says_its_own_once_they_hold_still() {
    let mut bag = BagParty::new();
    assert_eq!(bag.heard(), said(&[]));
    assert_eq!(bag.heard(), said(&[(&["Wingull", "ABLE!"], true)]));
    let ralts = bag.chosen("Ralts");
    bag.pieces.splice(..2, ralts);
    assert_eq!(bag.heard(), said(&[]));
    assert_eq!(bag.heard(), said(&[(&["Ralts", "ABLE!"], false)]));
}

/// The Bag keeps its select bar at x -320 while the player walks its items (spike 139).
#[test]
fn test_a_named_cursor_parked_off_its_screen_is_no_cursor() {
    let mut bag = BagParty::new();
    bag.r#move(-320.0);
    assert_eq!(bag.heard(), said(&[]));
    assert_eq!(bag.heard(), said(&[]));
}

/// The Pokédex's OTHER LANGUAGES tabs: two pictures a tab, whose textures the game swaps as it
/// runs, named by the image each draws (spike 147).
#[test]
fn test_a_cursor_over_pictures_alone_says_what_the_game_calls_them() {
    let mut menu = Menu::new();
    let picture =
        FakeClasses::starting(menu.memory.clone(), CODE.0 + 0x8000).add(lyt::PICTURE_CLASS);
    let tabs = 0x08400000;
    let work = menu.fake.screen(tabs).0;
    menu.fake.project(work, 320.0);
    let group = tabs + 0x600;
    let named: IndexMap<i64, Option<String>> = IndexMap::from([
        (tabs + 0x800, Some("French".to_string())),
        (tabs + 0x900, Some("French".to_string())),
        (tabs + 0xA00, None),
    ]);
    menu.memory.u32(tabs + 0x400 + lyt::CHILD_COUNT, &[1]);
    menu.memory
        .u32(group + lyt::CHILD_COUNT, &[named.len() as i64]);
    for (pane, x) in named.keys().zip([-90.0, -70.0, 60.0]) {
        menu.fake.pane_with_vtable(*pane, group, Some(picture));
        put(&menu.memory, pane + lyt::X, &[x]);
        put(&menu.memory, pane + lyt::Y, &[60.0]);
    }
    menu.fake.track().unwrap();
    // `pictures=named.get`, a dict's `get`: None where the pane is not in it.
    let pictures: Pictures = Rc::new(move |pane: i64| Ok(named.get(&pane).cloned().flatten()));
    menu.reader = CursorReader::with(
        menu.fake.layouts.clone(),
        names_of(&menu.memory),
        None,
        false,
        Some(pictures),
        IndexMap::new(),
    )
    .into();
    menu.reader.track([menu.window]).unwrap();
    assert_eq!(menu.heard_with(&[]), said(&[(&["French"], true)]));
    menu.r#move(60.0, 60.0);
    assert_eq!(menu.heard_with(&[]), said(&[]));
}

const SECRET_BASE: i64 = 0x084C0000;
const SECRET_BASE_FILE: i64 = 0x084E0000;

/// A Secret Base's list as spike 154 traced it: rows 210 by 20 under the list's pane, and the
/// cursor, `s_b_cursor.bclim` twice in a pane 30 by 40 at the row's middle, alone in a pane the
/// game moves onto the chosen row at the row's own size. `yes_no` is its YES and NO instead: the
/// two words beside the cursor's pane, 110 by 40, whose halves are 50 by 20.
struct BaseMenu {
    memory: Rc<FakeMemory>,
    fake: FakeLayouts,
    root: i64,
    yes_no: bool,
    x: f64,
    rows: Vec<(i64, &'static str, f64)>,
    pieces: Vec<ScreenText>,
    moved: Vec<i64>,
    reader: Reader,
}

impl BaseMenu {
    fn new() -> BaseMenu {
        BaseMenu::with(false)
    }

    fn with(yes_no: bool) -> BaseMenu {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let picture = classes.add(lyt::PICTURE_CLASS);
        let fake = FakeLayouts::new(&memory, &classes);
        let (work, root) = fake.screen_with(SECRET_BASE, true, false);
        fake.project(work, 400.0);
        let r#box = SECRET_BASE + 0x1000;
        fake.pane(r#box, root);
        let mut panes: Vec<(String, i64)> =
            vec![("RootPane".to_string(), root), ("1w".to_string(), r#box)];
        let (rows, x, wide): (Vec<(&'static str, f64)>, f64, f64) = if yes_no {
            (vec![("YES", -30.0), ("NO", -50.0)], 140.0, 70.0)
        } else {
            (
                vec![("Desks", 80.0), ("Chairs", 60.0), ("Mats", 40.0)],
                -85.0,
                210.0,
            )
        };
        let reader = reader_of(&fake, &memory);
        let mut menu = BaseMenu {
            memory,
            fake,
            root,
            yes_no,
            x,
            rows: Vec::new(),
            pieces: Vec::new(),
            moved: Vec::new(),
            reader,
        };
        for (index, (word, y)) in rows.iter().enumerate() {
            let row = SECRET_BASE + 0x2000 + 0x400 * index as i64;
            menu.fake.pane(row, r#box);
            menu.place(row, menu.x, *y, wide, 20.0);
            panes.push((format!("r{index}"), row));
            menu.rows.push((row, *word, *y));
        }
        menu.pieces = menu.words(&[]);
        let first = rows[0].1;
        let half: (f64, f64);
        if yes_no {
            let holder = SECRET_BASE + 0x3000;
            menu.fake.pane(holder, r#box);
            menu.place(holder, menu.x, first, 110.0, 40.0);
            menu.moved = vec![holder];
            half = (50.0, 20.0);
            panes.push(("4q".to_string(), holder));
        } else {
            let (row_pane, holder) = (SECRET_BASE + 0x3000, SECRET_BASE + 0x3200);
            menu.fake.pane(row_pane, r#box);
            menu.place(row_pane, menu.x, first, 210.0, 20.0);
            menu.fake.pane(holder, row_pane);
            menu.place(holder, menu.x, first, 30.0, 40.0);
            menu.moved = vec![row_pane, holder];
            half = (120.0, 20.0);
            panes.extend([("3i".to_string(), row_pane), ("4r".to_string(), holder)]);
        }
        let holder = *menu.moved.last().unwrap();
        for (picture_pane, name) in [(holder + 0x100, "c0"), (holder + 0x180, "c1")] {
            menu.fake
                .pane_with_vtable(picture_pane, holder, Some(picture));
            menu.place(picture_pane, menu.x, first, half.0, half.1);
            menu.moved.push(picture_pane);
            panes.push((name.to_string(), picture_pane));
        }
        for (name, pane) in &panes {
            write_name(&menu.memory, *pane, name);
        }
        menu.fake.track().unwrap();
        let mut sections = vec![
            clyt::section(b"txl1", &clyt::names_table(&["s_b_cursor.bclim"])),
            clyt::section(
                b"mat1",
                &clyt::materials(&[clyt::material("c0", &[0]), clyt::material("c1", &[0])]),
            ),
        ];
        for (name, _pane) in &panes {
            let cursor_picture = name == "c0" || name == "c1";
            sections.push(clyt::pane_with(
                if cursor_picture { b"pic1" } else { b"pan1" },
                name,
                if cursor_picture { 0 } else { 1 },
            ));
        }
        menu.memory
            .write(SECRET_BASE_FILE, &layout_archive(sections));
        let files = Rc::new(LayoutFiles::new(menu.memory.clone()));
        files.add_archive(SECRET_BASE_FILE).unwrap();
        menu.reader = reader_with_files(&menu.fake, &menu.memory, &files);
        menu
    }

    /// Each row's words, from the left of its box, or those `rewritten` gives it instead.
    fn words(&self, rewritten: &[(&str, &[&str])]) -> Vec<ScreenText> {
        let mut pieces: Vec<ScreenText> = Vec::new();
        for (row, word, y) in &self.rows {
            let left = self.x - if self.yes_no { 35.0 } else { 105.0 };
            let default: &[&str] = &[*word];
            let texts = rewritten
                .iter()
                .find(|(key, _)| key == word)
                .map_or(default, |(_, texts)| *texts);
            for text in texts {
                pieces.push(ScreenText {
                    top: true,
                    root: self.root,
                    parent: *row,
                    ..ScreenText::placed(
                        *row,
                        *text,
                        false,
                        left,
                        left + 13.0 * text.chars().count() as f64,
                        *y,
                        13.0,
                    )
                });
            }
        }
        pieces
    }

    fn place(&self, pane: i64, x: f64, y: f64, width: f64, height: f64) {
        put(&self.memory, pane + lyt::X, &[x]);
        put(&self.memory, pane + lyt::Y, &[y]);
        put(&self.memory, pane + lyt::WIDTH, &[width, height]);
        put(&self.memory, pane + lyt::SCALE_X, &[1.0]);
        put(&self.memory, pane + lyt::SCALE_Y, &[1.0]);
    }

    /// Put the cursor on the row at `y`, standing there the ticks a poll needs.
    fn rest(&mut self, y: f64) {
        for pane in &self.moved {
            put(&self.memory, pane + lyt::Y, &[y]);
        }
        for _ in 0..3 {
            self.reader.sample(&*self.memory).unwrap();
        }
    }

    fn heard(&mut self) -> Vec<(Vec<String>, bool)> {
        self.fake.layouts.refresh();
        heard_of(self.reader.poll(&self.pieces).unwrap().items)
    }
}

/// A Secret Base's PC menu and its decorations' categories and items were read as a whole
/// screen, and their cursor said nothing, or an item's count alone: the pane holding its
/// pictures is 30 wide at the row's middle, and the row's words begin left of it (spike 154).
#[test]
fn test_a_cursor_alone_in_a_pane_moved_onto_the_row_says_that_row() {
    let mut menu = BaseMenu::new();
    menu.rest(80.0);
    assert_eq!(menu.heard(), said(&[(&["Desks"], true)]));
    menu.rest(60.0);
    assert_eq!(menu.heard(), said(&[(&["Chairs"], false)]));
}

/// The YES and NO keep the cursor's two halves, 20 tall, in a pane of 40, and it said "YES
/// NO" (spike 154).
#[test]
fn test_a_cursor_two_rows_tall_says_the_row_its_pictures_are_on() {
    let mut r#box = BaseMenu::with(true);
    r#box.rest(-30.0);
    assert_eq!(r#box.heard(), said(&[(&["YES"], true)]));
    r#box.rest(-50.0);
    assert_eq!(r#box.heard(), said(&[(&["NO"], false)]));
}

/// Opening a category writes its items into the rows a poll before the categories' words go,
/// and the row under the cursor was said with both (spike 154).
#[test]
fn test_words_rewritten_under_a_cursor_at_rest_are_said_once_they_hold() {
    let mut menu = BaseMenu::new();
    menu.rest(80.0);
    menu.heard();
    menu.pieces = menu.words(&[("Desks", &["Desks", "Red Flower"])]);
    menu.rest(80.0);
    assert_eq!(menu.heard(), said(&[]));
    menu.pieces = menu.words(&[("Desks", &["Red Flower"])]);
    menu.rest(80.0);
    assert_eq!(menu.heard(), said(&[]));
    menu.rest(80.0);
    assert_eq!(menu.heard(), said(&[(&["Red Flower"], false)]));
}
