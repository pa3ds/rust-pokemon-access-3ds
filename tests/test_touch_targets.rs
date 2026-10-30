//! Touch mode's walk over the bottom screen's targets (`games.common.touch`).
//!
//! What a touch area is called is measured against the game (spike 171); what is tested here is
//! the walk: reading order row by row, the keys' moves, and that a tap only ever reaches the target
//! pa3ds has just named.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::{self, Options, Target, TouchTargets};

fn dexnav() -> Target {
    Target::new(1, "DexNav", 285, 53, 67.0, 30.0)
}

fn area() -> Target {
    Target::new(2, "Route 101", 125, 105, 15.0, 210.0)
}

fn bar() -> Vec<Target> {
    ["POKÉMON", "POKÉDEX", "BAG"]
        .iter()
        .enumerate()
        .map(|(i, name)| Target::new(10 + i as i64, name, 60 + 40 * i as i64, 223, -103.0, 30.0))
        .collect()
}

/// Touch targets with the screen's given, and every tap kept.
///
/// Python subclasses `TouchTargets` and overrides `targets`; here the targets are handed to it
/// through its test-only `given`, and the layouts it never reads are an empty memory's.
struct Screen {
    targets: TouchTargets,
    tapped: Rc<RefCell<Vec<(i64, i64)>>>,
    /// Every slide, as Python's test keeps them by setting `_slide`.
    slid: Rc<RefCell<Vec<(Vec<(i64, i64)>, f64)>>>,
    found: Rc<RefCell<Vec<Target>>>,
}

impl Screen {
    fn new(found: Vec<Target>) -> Self {
        Screen::with_moved(found, None)
    }

    /// `Screen(found, moved)`.
    fn with_moved(found: Vec<Target>, moved: Option<touch::Moved>) -> Self {
        let tapped: Rc<RefCell<Vec<(i64, i64)>>> = Rc::new(RefCell::new(Vec::new()));
        let slid: Rc<RefCell<Vec<(Vec<(i64, i64)>, f64)>>> = Rc::new(RefCell::new(Vec::new()));
        let kept = slid.clone();
        let found = Rc::new(RefCell::new(found));
        let memory = FakeMemory::new();
        let layouts = Rc::new(Layouts::new(
            memory.clone(),
            Rc::new(ClassNames::new(memory.clone(), CODE)),
            lyt::ORAS,
        ));
        let keep = tapped.clone();
        let mut targets = TouchTargets::new(
            layouts,
            Rc::new(Vec::new),
            Rc::new(|_box| Ok(String::new())),
            Rc::new(|_pane| Ok(None)),
            Rc::new(|_root| None),
            Rc::new(|_line: &str| {}),
            Options {
                tap: Some(Rc::new(move |x, y| {
                    keep.borrow_mut().push((x, y));
                    None
                })),
                slide: Some(Rc::new(move |path: &[(i64, i64)], seconds| {
                    kept.borrow_mut().push((path.to_vec(), seconds));
                    None
                })),
                moved,
                ..Options::default()
            },
        );
        let given = found.clone();
        targets.given = Some(Rc::new(move || {
            let found = given.borrow();
            touch::rows(&found)
                .into_iter()
                .flatten()
                .map(|i| found[i].clone())
                .collect()
        }));
        Screen {
            targets,
            tapped,
            slid,
            found,
        }
    }

    fn start(&mut self) -> String {
        self.targets.start().unwrap()
    }

    fn press(&mut self, action: &str) -> String {
        self.targets.press(action).unwrap()
    }

    fn tapped(&self) -> Vec<(i64, i64)> {
        self.tapped.borrow().clone()
    }
}

#[test]
fn test_targets_are_walked_row_by_row_from_the_top() {
    let mut screen = Screen::new([bar(), vec![area(), dexnav()]].concat());
    assert_eq!(screen.start(), "DexNav");
    assert_eq!(
        (0..5)
            .map(|_| screen.press(touch::ITEM_NEXT))
            .collect::<Vec<_>>(),
        ["Route 101", "POKÉMON", "POKÉDEX", "BAG", "DexNav"]
    );
    assert_eq!(screen.press(touch::ITEM_PREVIOUS), "BAG");
    assert_eq!(screen.press(touch::ROW_NEXT), "DexNav");
    assert_eq!(screen.press(touch::ROW_PREVIOUS), "BAG");
    assert_eq!(screen.press(touch::ITEM_READ), "BAG");
}

#[test]
fn test_a_row_key_moves_straight_up_or_down() {
    // Orion, 2026-10-26: Tile Puzzle's grid walked by column, to the nearest across.
    let grid: Vec<Target> = (0..6)
        .map(|i: i64| {
            let name = format!("{}{}", ['a', 'b'][(i / 3) as usize], i % 3);
            Target::new(
                20 + i,
                &name,
                40 + 60 * (i % 3),
                30 + 60 * (i / 3),
                90.0 - 60.0 * (i / 3) as f64,
                58.0,
            )
        })
        .collect();
    let mut screen = Screen::new(grid);
    screen.start();
    screen.press(touch::ITEM_NEXT);
    assert_eq!(
        (0..2)
            .map(|_| screen.press(touch::ROW_NEXT))
            .collect::<Vec<_>>(),
        ["b1", "a1"]
    );
    assert_eq!(screen.press(touch::ROW_PREVIOUS), "b1");
}

#[test]
fn test_a_move_can_be_said_otherwise_than_by_the_name() {
    // Tile Puzzle says only the row or the column that changed; staying put says the name.
    let moved: touch::Moved = Rc::new(|before: &Target, now: &Target| {
        Ok(if before == now {
            None
        } else {
            Some(format!("from {}", before.name))
        })
    });
    let mut screen = Screen::with_moved(bar(), Some(moved));
    screen.start();
    assert_eq!(screen.press(touch::ITEM_NEXT), "from POKÉMON");
    assert_eq!(screen.press(touch::ITEM_READ), "POKÉDEX");
}

/// ADR-0009: with nothing chosen, and once the chosen target has left the screen, the tap key
/// only names the first: the player has not heard what it would touch.
#[test]
fn test_a_tap_reaches_only_the_target_just_named() {
    let mut screen = Screen::new([vec![dexnav()], bar()].concat());
    assert!(screen.press(touch::TAP) == "DexNav" && screen.tapped().is_empty());
    assert!(screen.press(touch::TAP).is_empty() && screen.tapped() == [(285, 53)]);
    *screen.found.borrow_mut() = bar();
    assert!(screen.press(touch::TAP) == "POKÉMON" && screen.tapped() == [(285, 53)]);
}

#[test]
fn test_nothing_on_the_bottom_screen_is_said_so() {
    let mut screen = Screen::new(Vec::new());
    let started = screen.start();
    assert!(started == "nothing to touch" && "nothing to touch" == screen.press(touch::ITEM_NEXT));
}

/// The X menu (layer 10) over the PokéNav (layer 1): its POKÉDEX button covers the DexNav, which a
/// tap would never reach; the shortcut bar, in no menu button, stays (movie21).
#[test]
fn test_a_target_under_a_layout_drawn_over_it_is_left_out() {
    let pokedex = Target {
        layer: 10,
        ..Target::new(30, "POKÉDEX", 240, 60, 60.0, 40.0)
    };
    let dexnav = Target {
        layer: 1,
        ..Target::new(1, "DexNav", 285, 53, 67.0, 30.0)
    };
    let areas = [(10, 240, 60, 160.0, 40.0), (1, 285, 53, 30.0, 30.0)];
    assert_eq!(
        touch::uncovered(
            &[vec![pokedex.clone(), dexnav.clone()], bar()].concat(),
            &areas
        ),
        [vec![pokedex], bar()].concat()
    );
    assert_eq!(
        touch::uncovered(std::slice::from_ref(&dexnav), &[(1, 240, 60, 160.0, 40.0)]),
        [dexnav]
    );
}

/// `{a, b}` of words.
fn words(items: &[&str]) -> HashSet<String> {
    items.iter().map(|item| item.to_string()).collect()
}

/// The party list's plates each hold a corner where a held item is drawn, its words all the
/// plate's, and a tap there did nothing (movie22); a button inside another with words of its own
/// stays.
#[test]
fn test_a_touch_area_inside_another_is_part_of_its_button() {
    let (plate, corner, other) = (1, 2, 3);
    let placed = IndexMap::from([
        (plate, (80, 35, 160.0, 60.0)),
        (corner, (125, 52, 30.0, 20.0)),
        (other, (240, 45, 160.0, 60.0)),
    ]);
    let mut own = IndexMap::from([
        (plate, words(&["Blaze", "L", "14"])),
        (corner, words(&["L", "14"])),
        (other, words(&["Beldum"])),
    ]);
    let none = HashSet::new();
    assert!(touch::within_another(corner, &placed, &own, &none));
    assert!(!touch::within_another(plate, &placed, &own, &none));
    own.insert(corner, words(&["ITEM"]));
    assert!(!touch::within_another(corner, &placed, &own, &none));
}

/// The BuzzNav's NEXT lies within a touch area over its whole panel, hanging from the root, whose
/// words are all of the panel's, and only the panel was offered (movie24).
#[test]
fn test_a_button_inside_the_screens_own_touch_area_is_its_own() {
    let (panel, next) = (1, 2);
    let placed = IndexMap::from([
        (panel, (125, 105, 250.0, 210.0)),
        (next, (125, 140, 150.0, 30.0)),
    ]);
    let own = IndexMap::from([
        (panel, words(&["NEXT", "Welcome to BuzzNav!"])),
        (next, words(&["NEXT"])),
    ]);
    assert!(touch::within_another(next, &placed, &own, &HashSet::new()));
    assert!(!touch::within_another(
        next,
        &placed,
        &own,
        &HashSet::from([panel])
    ));
}

/// The PC Box under "Continue operations?" drew its grid as bright as ever, and touch mode
/// offered it with YES and NO (movie22); a layout at a higher layer stays.
#[test]
fn test_while_a_list_window_is_up_only_its_buttons_are_offered() {
    let (r#box, bar, question, above) = (0x10, 0x20, 0x30, 0x40);
    let placed = |pane, name: &str, x, y, row, layer, root| Target {
        layer,
        root,
        ..Target::new(pane, name, x, y, row, 30.0)
    };
    let slot = placed(1, "BOX 1, row 1, column 1, empty", 30, 65, 55.0, 1, r#box);
    let exit = placed(2, "exit", 255, 225, -105.0, 1, bar);
    let yes = placed(3, "YES", 275, 155, -35.0, 1, question);
    let no = placed(4, "NO", 275, 185, -65.0, 1, question);
    let over = placed(5, "zoom in", 20, 20, 100.0, 2, above);
    let found = vec![slot, exit, yes.clone(), no.clone(), over.clone()];
    assert_eq!(
        touch::answered_first(&found, &words(&["YES", "NO"])),
        [yes, no, over]
    );
    assert_eq!(touch::answered_first(&found, &HashSet::new()), found);
    assert_eq!(
        touch::answered_first(&found, &words(&["Buy", "Sell"])),
        found
    );
}

/// The PlayNav's pages and the sub-screen tutorial's arrows both draw `button.bclim`, a ◀
/// mirrored by its material on the right; the same texture lower down is a plate (spike 198).
#[test]
fn test_a_bare_button_in_the_top_bars_corners_is_the_page_arrows() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (bounding, picture) = (
        classes.add(lyt::BOUNDING_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let (_work, group) = fake.screen(0x08200000);
    let mut drawn: HashSet<i64> = HashSet::new();
    for (i, (x, y)) in [(-140.0f32, 105.0f32), (140.0, 105.0), (0.0, 0.0)]
        .into_iter()
        .enumerate()
    {
        let button = group + 0x1000 * (i as i64 + 1);
        fake.pane(button, group);
        fake.pane_with_vtable(button + 0x200, button, Some(bounding));
        fake.pane_with_vtable(button + 0x400, button, Some(picture));
        memory.write(button + 0x200 + lyt::X, &x.to_le_bytes());
        memory.write(button + 0x200 + lyt::Y, &y.to_le_bytes());
        memory.write(
            button + 0x200 + lyt::WIDTH,
            &[30.0f32.to_le_bytes(), 30.0f32.to_le_bytes()].concat(),
        );
        drawn.insert(button + 0x400);
    }
    fake.track().unwrap();
    let mut targets = TouchTargets::new(
        fake.layouts.clone(),
        Rc::new(Vec::new),
        Rc::new(|_box| Ok(String::new())),
        Rc::new(move |pane| {
            Ok(if drawn.contains(&pane) {
                Some("button.bclim".to_string())
            } else {
                None
            })
        }),
        Rc::new(|_root| None),
        Rc::new(|_line: &str| {}),
        Options::default(),
    );
    assert_eq!(
        targets
            .targets()
            .unwrap()
            .iter()
            .map(|t| (t.name.as_str(), t.x, t.y))
            .collect::<Vec<_>>(),
        [
            ("previous", 20, 15),
            ("next", 300, 15),
            ("button", 160, 120)
        ]
    );
}

/// Super Training's party under its Use panel: drawn as bright as ever, its buttons read 0,
/// and a tap on Breloom did nothing (spike 228).
#[test]
fn test_a_touch_area_whose_button_is_switched_off_is_left_out() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (bounding, picture) = (
        classes.add(lyt::BOUNDING_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let (work, group) = fake.screen(0x08200000);
    for (i, (x, name)) in [(-140.0f32, b"2q"), (140.0, b"2r")].into_iter().enumerate() {
        let button = group + 0x1000 * (i as i64 + 1);
        fake.pane(button, group);
        fake.pane_with_vtable(button + 0x200, button, Some(bounding));
        fake.pane_with_vtable(button + 0x400, button, Some(picture));
        memory.write(button + 0x200 + lyt::NAME, name);
        memory.write(button + 0x200 + lyt::X, &x.to_le_bytes());
        memory.write(
            button + 0x200 + lyt::WIDTH,
            &[40.0f32.to_le_bytes(), 30.0f32.to_le_bytes()].concat(),
        );
    }
    fake.track().unwrap();
    let mut targets = TouchTargets::new(
        fake.layouts.clone(),
        Rc::new(Vec::new),
        Rc::new(|_box| Ok(String::new())),
        Rc::new(|_pane| Ok(Some("slot.bclim".to_string()))),
        Rc::new(|_root| None),
        Rc::new(|_line: &str| {}),
        Options {
            images: Some(IndexMap::from([(
                "slot.bclim".to_string(),
                "slot".to_string(),
            )])),
            off: Some(Rc::new(move || {
                Ok(HashSet::from([(work, "2q".to_string())]))
            })),
            ..Options::default()
        },
    );
    assert_eq!(
        targets
            .targets()
            .unwrap()
            .iter()
            .map(|t| (t.name.as_str(), t.x))
            .collect::<Vec<_>>(),
        [("slot", 300)]
    );
}

/// The DexNav switches off the icon of a species not caught, and it is said by its species
/// (ADR-0037): Ralts was left out with its button (or_movie27_dex_nav_regression).
#[test]
fn test_a_button_switched_off_whose_picture_is_said_all_the_same_is_offered() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (bounding, picture) = (
        classes.add(lyt::BOUNDING_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let (work, group) = fake.screen(0x08200000);
    let mut icons: IndexMap<i64, String> = IndexMap::new();
    for (i, (x, name, species)) in [
        (-140.0f32, b"2q", "Ralts"),
        (140.0, b"2r", "Zigzagoon caught"),
    ]
    .into_iter()
    .enumerate()
    {
        let button = group + 0x1000 * (i as i64 + 1);
        fake.pane(button, group);
        fake.pane_with_vtable(button + 0x200, button, Some(bounding));
        fake.pane_with_vtable(button + 0x400, button, Some(picture));
        memory.write(button + 0x200 + lyt::NAME, name);
        memory.write(button + 0x200 + lyt::X, &x.to_le_bytes());
        memory.write(
            button + 0x200 + lyt::WIDTH,
            &[40.0f32.to_le_bytes(), 30.0f32.to_le_bytes()].concat(),
        );
        icons.insert(button + 0x400, species.to_string());
    }
    fake.track().unwrap();
    let icons = Rc::new(icons);
    let mut targets = TouchTargets::new(
        fake.layouts.clone(),
        Rc::new(Vec::new),
        Rc::new(|_box| Ok(String::new())),
        Rc::new(|_pane| Ok(None)),
        Rc::new(|_root| None),
        Rc::new(|_line: &str| {}),
        Options {
            picture_name: Some({
                let icons = icons.clone();
                Rc::new(move |pane| Ok(icons.get(&pane).cloned()))
            }),
            off: Some(Rc::new(move || {
                Ok(HashSet::from([(work, "2q".to_string())]))
            })),
            said_off: Some({
                let icons = icons.clone();
                Rc::new(move |pane| Ok(icons.contains_key(&pane)))
            }),
            ..Options::default()
        },
    );
    assert_eq!(
        targets
            .targets()
            .unwrap()
            .iter()
            .map(|t| (t.name.as_str(), t.x))
            .collect::<Vec<_>>(),
        [("Ralts", 20), ("Zigzagoon caught", 300)]
    );
}

/// Screen text names pictures as touch mode does: the Bag's party slots were offered as
/// "Torchic Torchic L14" and the Battle Maison's partners as "May May"
/// (or_multi_battle_partners).
#[test]
fn test_a_picture_screen_text_names_is_said_once() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (bounding, picture) = (
        classes.add(lyt::BOUNDING_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let (_work, group) = fake.screen(0x08200000);
    let button = group + 0x1000;
    fake.pane(button, group);
    fake.pane_with_vtable(button + 0x200, button, Some(bounding));
    fake.pane_with_vtable(button + 0x400, button, Some(picture));
    memory.write(
        button + 0x200 + lyt::WIDTH,
        &[40.0f32.to_le_bytes(), 40.0f32.to_le_bytes()].concat(),
    );
    fake.track().unwrap();
    let words = vec![ScreenText::new(button + 0x400, "May", false)];
    let mut targets = TouchTargets::new(
        fake.layouts.clone(),
        Rc::new(move || words.clone()),
        Rc::new(|_box| Ok(String::new())),
        Rc::new(|_pane| Ok(None)),
        Rc::new(|_root| None),
        Rc::new(|_line: &str| {}),
        Options {
            picture_name: Some(Rc::new(move |pane| {
                Ok((pane == button + 0x400).then(|| "May".to_string()))
            })),
            ..Options::default()
        },
    );
    assert_eq!(
        targets
            .targets()
            .unwrap()
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>(),
        ["May"]
    );
}

/// The Pokémon-Amie page's PLAY is a picture of its word inside the touch area behind its
/// menu, which closes it: the backdrop took PLAY's word, and PLAY was left out (spike 198).
#[test]
fn test_a_button_the_game_names_inside_the_menus_backdrop_is_its_own() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (bounding, picture) = (
        classes.add(lyt::BOUNDING_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let (_work, group) = fake.screen(0x08200000);
    let menu = group + 0x1000;
    let (backdrop, behind, play, area) = (menu + 0x200, menu + 0x400, menu + 0x600, menu + 0x800);
    fake.pane(menu, group);
    fake.pane_with_vtable(backdrop, menu, Some(bounding));
    fake.pane_with_vtable(behind, menu, Some(picture));
    fake.pane_with_vtable(play, menu, Some(picture));
    fake.pane_with_vtable(area, play, Some(bounding));
    memory.write(
        backdrop + lyt::WIDTH,
        &[320.0f32.to_le_bytes(), 180.0f32.to_le_bytes()].concat(),
    );
    for pane in [play, area] {
        memory.write(pane + lyt::Y, &(-61.0f32).to_le_bytes());
    }
    memory.write(
        area + lyt::WIDTH,
        &[90.0f32.to_le_bytes(), 54.0f32.to_le_bytes()].concat(),
    );
    fake.track().unwrap();
    let drawn: IndexMap<i64, String> = IndexMap::from([
        (behind, "msg_win_c.bclim".to_string()),
        (play, "kwg_button01.bclim".to_string()),
    ]);
    let mut targets = TouchTargets::new(
        fake.layouts.clone(),
        Rc::new(Vec::new),
        Rc::new(|_box| Ok(String::new())),
        Rc::new(move |pane| Ok(drawn.get(&pane).cloned())),
        Rc::new(|_root| None),
        Rc::new(|_line: &str| {}),
        Options {
            images: Some(IndexMap::from([(
                "kwg_button01.bclim".to_string(),
                "PLAY".to_string(),
            )])),
            ..Options::default()
        },
    );
    assert_eq!(
        targets
            .targets()
            .unwrap()
            .iter()
            .map(|t| (t.name.as_str(), t.x, t.y))
            .collect::<Vec<_>>(),
        [("close", 160, 120), ("PLAY", 160, 181)]
    );
}

fn floats(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

/// Super Training's carousel moves its party's pictures while their touch areas and parents
/// stay put, and a tap selects what is drawn there: the middle was said "Breloom" with Sceptile
/// drawn in it (2026-10-10, live from omega_ruby_bottom-screen5's end).
#[test]
fn test_a_touch_area_is_named_by_the_picture_now_lying_in_it() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (bounding, picture) = (
        classes.add(lyt::BOUNDING_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let (_work, group) = fake.screen(0x08200000);
    let mut pictures: IndexMap<i64, String> = IndexMap::new();
    let mut places: Vec<i64> = Vec::new();
    for (i, (x, name)) in [(-80.0f32, "Sceptile"), (0.0, "Latios"), (80.0, "Breloom")]
        .into_iter()
        .enumerate()
    {
        let parent = group + 0x1000 * (i as i64 + 1);
        fake.pane(parent, group);
        fake.pane_with_vtable(parent + 0x200, parent, Some(bounding));
        fake.pane_with_vtable(parent + 0x400, parent, Some(picture));
        for pane in [parent + 0x200, parent + 0x400] {
            memory.write(pane + lyt::SCALE_X, &floats(&[1.0]));
            memory.write(pane + lyt::SCALE_Y, &floats(&[1.0]));
            memory.write(pane + lyt::WIDTH, &floats(&[40.0, 30.0]));
        }
        for pane in [parent + 0x200, parent + 0x400] {
            memory.write(pane + lyt::X, &floats(&[x]));
        }
        pictures.insert(parent + 0x400, name.to_string());
        places.push(parent + 0x400);
    }
    fake.track().unwrap();
    let mut targets = TouchTargets::new(
        fake.layouts.clone(),
        Rc::new(Vec::new),
        Rc::new(|_box| Ok(String::new())),
        Rc::new(|_pane| Ok(None)),
        Rc::new(|_root| None),
        Rc::new(|_line: &str| {}),
        Options {
            picture_name: Some(Rc::new(move |pane| Ok(pictures.get(&pane).cloned()))),
            ..Options::default()
        },
    );
    let named = |targets: &mut TouchTargets| -> IndexMap<i64, String> {
        targets
            .targets()
            .unwrap()
            .into_iter()
            .map(|t| (t.x, t.name))
            .collect()
    };
    assert_eq!(
        named(&mut targets),
        IndexMap::from([
            (80, "Sceptile".to_string()),
            (160, "Latios".to_string()),
            (240, "Breloom".to_string())
        ])
    );
    for (pane, x) in [(places[0], 80.0f32), (places[2], -80.0)] {
        memory.write(pane + lyt::X, &floats(&[x]));
    }
    fake.layouts.refresh();
    let after = named(&mut targets);
    assert_eq!(
        (
            after[&80].as_str(),
            after[&160].as_str(),
            after[&240].as_str()
        ),
        ("Breloom", "Latios", "Sceptile")
    );
}

/// Amie's Interior Items move a page's pictures out from under touch areas that stay put, and
/// a slot empty on the next page was still said by the item it had held (or_pokemon_me2).
#[test]
fn test_a_touch_area_whose_picture_has_moved_away_is_not_said_by_it() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (bounding, picture) = (
        classes.add(lyt::BOUNDING_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let (_work, group) = fake.screen(0x08200000);
    let parent = group + 0x1000;
    let (area, item) = (parent + 0x200, parent + 0x400);
    fake.pane(parent, group);
    fake.pane_with_vtable(area, parent, Some(bounding));
    fake.pane_with_vtable(item, parent, Some(picture));
    for pane in [area, item] {
        memory.write(pane + lyt::SCALE_X, &floats(&[1.0, 1.0]));
        memory.write(pane + lyt::WIDTH, &floats(&[40.0, 40.0]));
    }
    fake.track().unwrap();
    let mut targets = TouchTargets::new(
        fake.layouts.clone(),
        Rc::new(Vec::new),
        Rc::new(|_box| Ok(String::new())),
        Rc::new(|_pane| Ok(None)),
        Rc::new(|_root| None),
        Rc::new(|_line: &str| {}),
        Options {
            picture_name: Some(Rc::new(move |pane| {
                Ok((pane == item).then(|| "Leaf Cushion".to_string()))
            })),
            ..Options::default()
        },
    );
    let names = |targets: &mut TouchTargets| -> Vec<String> {
        targets
            .targets()
            .unwrap()
            .into_iter()
            .map(|t| t.name)
            .collect()
    };
    assert_eq!(names(&mut targets), ["Leaf Cushion"]);
    memory.write(item + lyt::X, &floats(&[280.0]));
    fake.layouts.refresh();
    assert!(names(&mut targets).is_empty());
}

/// Amie's decorate screen: a bar hanging from the root under its two tabs and its exit named
/// the exit by the tabs' words, and was offered at its middle, on a tab (x_pokemon-me).
#[test]
fn test_the_screens_bar_neither_names_a_button_nor_is_offered_over_one() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (bounding, picture) = (
        classes.add(lyt::BOUNDING_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let textbox = classes.add(lyt::TEXTBOX_CLASS);
    let (_work, root) = fake.screen_with(0x08200000, false, false);
    let (bar, tab, back) = (root + 0x1000, root + 0x2000, root + 0x3000);
    let (tab_picture, tab_words, tab_touch, back_touch) =
        (tab + 0x200, tab + 0x400, tab + 0x600, back + 0x200);
    for (pane, parent, kind, x, size) in [
        (bar, root, Some(bounding), 0.0f32, [320.0f32, 30.0]),
        (tab, root, None, 0.0, [100.0, 25.0]),
        (tab_picture, tab, Some(picture), 0.0, [100.0, 25.0]),
        (tab_words, tab, Some(textbox), 0.0, [90.0, 20.0]),
        (tab_touch, tab, Some(bounding), 0.0, [100.0, 25.0]),
        (back, root, Some(picture), 145.0, [30.0, 30.0]),
        (back_touch, back, Some(bounding), 145.0, [30.0, 30.0]),
    ] {
        match kind {
            Some(kind) => fake.pane_with_vtable(pane, parent, Some(kind)),
            None => fake.pane(pane, parent),
        }
        memory.write(pane + lyt::X, &floats(&[x]));
        memory.write(pane + lyt::Y, &floats(&[-105.0]));
        memory.write(pane + lyt::WIDTH, &floats(&size));
    }
    fake.track().unwrap();
    let words = vec![ScreenText::new(tab_words, "Poké Puffs", false)];
    let drawn: IndexMap<i64, String> = IndexMap::from([
        (tab_picture, "button_system.bclim".to_string()),
        (back, "button_back.bclim".to_string()),
    ]);
    let mut targets = TouchTargets::new(
        fake.layouts.clone(),
        Rc::new(move || words.clone()),
        Rc::new(move |pane| {
            Ok(if pane == tab_words {
                "Poké Puffs".to_string()
            } else {
                String::new()
            })
        }),
        Rc::new(move |pane| Ok(drawn.get(&pane).cloned())),
        Rc::new(|_root| None),
        Rc::new(|_line: &str| {}),
        Options::default(),
    );
    assert_eq!(
        targets
            .targets()
            .unwrap()
            .iter()
            .map(|t| (t.name.as_str(), t.x))
            .collect::<Vec<_>>(),
        [("Poké Puffs", 160), ("return", 305)]
    );
}

/// In the AreaNav a tap on a place on its map chooses it; its bar, drawn at the map's layer,
/// takes a tap on a place under it, and a place off the screen is none (or_area_nav). On the
/// field, where the map is a window, no place is offered.
#[test]
fn test_the_areanavs_places_are_offered_by_name_where_a_tap_reaches_them() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (bounding, picture) = (
        classes.add(lyt::BOUNDING_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let (_work, root) = fake.screen_with(0x08200000, false, false);
    let (_bar_work, bar) = fake.screen_with(0x08300000, false, false);
    let land = root + 0x1000;
    fake.pane_with_vtable(land, root, Some(picture));
    for (pane, parent, kind, name, x, y, size) in [
        (
            land + 0x200,
            land,
            bounding,
            &b"m"[..],
            -100.0f32,
            -25.0f32,
            [11.0f32, 11.0],
        ),
        (
            land + 0x400,
            land,
            bounding,
            b"1p",
            -20.0,
            -105.0,
            [22.0, 11.0],
        ),
        (
            land + 0x600,
            land,
            bounding,
            b"zz",
            200.0,
            0.0,
            [11.0, 11.0],
        ),
        (
            bar + 0x1000,
            bar,
            bounding,
            b"5x",
            0.0,
            -105.0,
            [320.0, 30.0],
        ),
    ] {
        fake.pane_with_vtable(pane, parent, Some(kind));
        memory.write(pane + lyt::NAME, name);
        memory.write(pane + lyt::X, &floats(&[x]));
        memory.write(pane + lyt::Y, &floats(&[y]));
        memory.write(pane + lyt::WIDTH, &floats(&size));
    }
    fake.track().unwrap();
    let places: IndexMap<String, String> =
        [("m", "Route 101"), ("1p", "Route 102"), ("zz", "Route 103")]
            .into_iter()
            .map(|(pane, place)| (pane.to_string(), place.to_string()))
            .collect();
    let shown: Rc<RefCell<IndexMap<String, String>>> = Rc::new(RefCell::new(IndexMap::new()));
    let mut targets = TouchTargets::new(
        fake.layouts.clone(),
        Rc::new(Vec::new),
        Rc::new(|_box| Ok(String::new())),
        Rc::new(move |pane| Ok((pane == land).then(|| touch::MAP.to_string()))),
        Rc::new(|_root| None),
        Rc::new(|_line: &str| {}),
        Options {
            places: Some({
                let shown = shown.clone();
                Rc::new(move || Ok(Rc::new(shown.borrow().clone())))
            }),
            ..Options::default()
        },
    );
    assert_eq!(targets.targets().unwrap(), []);
    shown.borrow_mut().extend(places);
    assert_eq!(
        targets
            .targets()
            .unwrap()
            .iter()
            .map(|t| (t.name.as_str(), t.x, t.y))
            .collect::<Vec<_>>(),
        [("Route 101", 60, 145)]
    );
}

/// The Pokémon-Amie menu's backdrop, a touch area over the whole page, hangs from a pane
/// holding no picture of a Pokémon, and was said "Sceptile" by the one lying within it from
/// another pane, rather than "close" (omega_ruby_bottom-screen2: `68` in `f`, the Pokémon's
/// picture in `8`).
#[test]
fn test_a_touch_area_holding_no_such_picture_is_not_named_by_one_lying_in_it() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (bounding, picture) = (
        classes.add(lyt::BOUNDING_CLASS),
        classes.add(lyt::PICTURE_CLASS),
    );
    let (_work, group) = fake.screen(0x08200000);
    let (backdrop, pokemon) = (group + 0x1200, group + 0x2200);
    fake.pane(group + 0x1000, group);
    fake.pane(group + 0x2000, group);
    fake.pane_with_vtable(backdrop, group + 0x1000, Some(bounding));
    fake.pane_with_vtable(pokemon, group + 0x2000, Some(picture));
    for (pane, (width, height)) in [(backdrop, (320.0f32, 180.0f32)), (pokemon, (40.0, 30.0))] {
        memory.write(pane + lyt::SCALE_X, &floats(&[1.0]));
        memory.write(pane + lyt::SCALE_Y, &floats(&[1.0]));
        memory.write(pane + lyt::WIDTH, &floats(&[width, height]));
    }
    fake.track().unwrap();
    let mut targets = TouchTargets::new(
        fake.layouts.clone(),
        Rc::new(Vec::new),
        Rc::new(|_box| Ok(String::new())),
        Rc::new(|_pane| Ok(None)),
        Rc::new(|_root| None),
        Rc::new(|_line: &str| {}),
        Options {
            picture_name: Some(Rc::new(move |pane| {
                Ok((pane == pokemon).then(|| "Sceptile".to_string()))
            })),
            ..Options::default()
        },
    );
    assert!(targets.targets().unwrap().is_empty());
}

/// Pokémon-Amie's Pokémon is stroked, not tapped: its target's key slides along the path
/// (spike 200).
#[test]
fn test_a_stroke_slides_along_its_path_where_a_tap_would_press() {
    let stroke = Target {
        path: vec![(110, 110), (210, 110)],
        seconds: 3.0,
        ..Target::new(1, "stroke Sceptile", 160, 120, 0.0, 240.0)
    };
    let mut screen = Screen::new(vec![stroke]);
    assert_eq!(screen.press(touch::TAP), "stroke Sceptile");
    assert_eq!(screen.press(touch::TAP), "");
    assert!(
        *screen.slid.borrow() == [(vec![(110, 110), (210, 110)], 3.0)]
            && screen.tapped().is_empty()
    );
}
