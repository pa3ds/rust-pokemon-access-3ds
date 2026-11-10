//! Super Training: the party's slots are named by species, a round's goals are tapped at the pixel
//! that shoots them, and the training bags in the page's list are offered by the game's names.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_battle::record_of;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::{ImageOf, Target};
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::screens::distribution as d;
use crate::games::gen6::subscreen::super_training;

const PAGE: i64 = 0x08B00000;
const ICON: i64 = 0x08B02000;
const REPLACE: i64 = 0x08B03000;
const TABLE: i64 = 0x08B04000;
const TEXTURES: i64 = 0x08B10000;
const PARAMS: i64 = 0x08B20000;
const FIRST: i64 = 0x08B06000;
const SECOND: i64 = 0x08B07000;
const OTHER: i64 = 0x08B08000;
const IMAGE: i64 = 0x18300000;

fn species() -> Vec<String> {
    let mut names = vec![String::new(); 254];
    names.extend(["Sceptile".to_string(), "Breloom".to_string()]);
    names
}

fn floats(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn page(icons: i64) -> super_training::TrainingPage {
    paged(icons).0
}

fn paged(icons: i64) -> (super_training::TrainingPage, Rc<FakeMemory>, FakeClasses) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(PAGE, &[classes.add(super_training::CLASS)]);
    memory.u32(PAGE + icons, &[ICON]);
    memory.u32(ICON, &[classes.add(d::ICON_CLASS)]);
    memory.u32(ICON + d::REPLACE, &[REPLACE]);
    memory.u32(REPLACE, &[classes.add(d::REPLACE_CLASS)]);
    memory.u32(REPLACE + d::TEXTURES, &[TABLE, 2]);
    let param_class = classes.add(pokemon::PARAM_CLASS);
    for (i, species) in [255, 254].into_iter().enumerate() {
        let i = i as i64;
        let param = PARAMS + 0x200 * i;
        memory.u32(param, &[param_class, param + pokemon::RECORD_IN_PARAM]);
        memory.write(param + pokemon::RECORD_IN_PARAM, &record_of("", 0, species));
        // The records are rotated to start at the selected member, Sceptile (1): each holds its
        // member's party index, and the textures stay in party order.
        let record_at =
            PAGE + icons + super_training::PARAMS + super_training::PARAM_STRIDE * (1 - i);
        memory.u32(record_at, &[param]);
        memory.u32(record_at + super_training::INDEX, &[i]);
        memory.u32(TABLE + 4 * i, &[TEXTURES + 0x100 * i]);
        memory.u32(TEXTURES + 0x100 * i + d::IMAGE, &[IMAGE + 0x1000 * i]);
    }
    for (picture, image) in [
        (FIRST, IMAGE),
        (SECOND, IMAGE + 0x1000),
        (OTHER, IMAGE + 0x2000),
    ] {
        let material = picture + 0x400;
        memory.u32(picture + lyt::PICTURE_MATERIAL, &[material]);
        memory.u32(material + lyt::MATERIAL_TEXTURE, &[image]);
    }
    let mut found = super_training::TrainingPage::with_icons(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        species(),
        icons,
    );
    found
        .track(
            &IndexMap::from([(PAGE, super_training::CLASS.to_string())]),
            false,
        )
        .unwrap();
    (found, memory, classes)
}

#[test]
fn test_a_party_slot_is_named_by_the_member_whose_icon_it_draws_however_the_records_rotate() {
    let found = page(super_training::ICONS);
    assert_eq!(
        (
            found.name(FIRST, Some(FIELD)).unwrap(),
            found.name(SECOND, Some(FIELD)).unwrap()
        ),
        (Some("Breloom".to_string()), Some("Sceptile".to_string()))
    );
    assert_eq!(
        found.name(FIRST, Some(super_training::SPARRING)).unwrap(),
        None
    );
}

#[test]
fn test_xs_page_is_read_where_x_keeps_its_icons_and_records() {
    let found = page(crate::games::xy::game().training);
    assert_eq!(
        (
            found.name(FIRST, Some(FIELD)).unwrap(),
            found.name(SECOND, Some(FIELD)).unwrap()
        ),
        (Some("Breloom".to_string()), Some("Sceptile".to_string()))
    );
}

#[test]
fn test_a_picture_drawing_another_image_is_not_named() {
    assert_eq!(
        page(super_training::ICONS)
            .name(OTHER, Some(FIELD))
            .unwrap(),
        None
    );
}

#[test]
fn test_the_pixel_that_shoots_a_goal_is_four_fifths_of_its_place_on_the_top_screen() {
    assert_eq!(super_training::aim(198.0, 153.0).unwrap(), Some((158, 122)));
    assert_eq!(super_training::aim(375.0, 25.0).unwrap(), Some((300, 20)));
    assert_eq!(super_training::aim(400.0, 25.0).unwrap(), None);
    assert_eq!(super_training::aim(200.0, 240.0).unwrap(), None);
}

fn drawn(images: IndexMap<i64, &'static str>) -> ImageOf {
    Rc::new(move |pane| Ok(images.get(&pane).map(|image| image.to_string())))
}

#[test]
fn test_a_goal_shown_while_a_round_runs_is_offered_at_the_pixel_that_shoots_it() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let picture = classes.add(lyt::PICTURE_CLASS);
    let fake = FakeLayouts::new(&memory, &classes);
    let (_work, group) = fake.screen_with(0x14B00000, true, true);
    let root = group - 0x200;
    memory.u32(root + lyt::CHILD_COUNT, &[1]);
    memory.u32(group + lyt::CHILD_COUNT, &[4]);
    // A pane's place is its own less the root's; its parent's offset is already in it.
    memory.write(root + lyt::X, &floats(&[50.0]));
    memory.write(group + lyt::X, &floats(&[-10.0]));
    memory.write(group + lyt::Y, &floats(&[-30.0]));
    let (goal, red, burst, hidden) = (group + 0x100, group + 0x200, group + 0x300, group + 0x400);
    for pane in [goal, red, burst, hidden] {
        fake.pane_with_vtable(pane, group, Some(picture));
        memory.write(pane + lyt::X, &floats(&[8.0]));
        memory.write(pane + lyt::Y, &floats(&[-3.0]));
        memory.write(pane + lyt::HEIGHT, &floats(&[32.0]));
    }
    fake.visible(hidden, false);
    fake.track().unwrap();
    let goals = super_training::Goals::new(
        fake.layouts.clone(),
        drawn(IndexMap::from([
            (goal, "target01_8.bclim"),
            (red, "big_target_3.bclim"),
            (burst, "target01_break.bclim"),
            (hidden, "target01_8.bclim"),
        ])),
    );
    assert_eq!(
        goals.targets(Some(super_training::SPARRING)).unwrap(),
        [
            Target::new(goal, "goal", 126, 98, -3.0, 32.0),
            Target::new(red, "red goal", 126, 98, -3.0, 32.0)
        ]
    );
    assert!(
        goals
            .targets(Some("N5field12FieldmapProcE"))
            .unwrap()
            .is_empty()
    );
}

/// A bottom-screen layout drawing a picture of each image, with what each draws, and the
/// pictures in order.
fn drawing(images: &[&'static str]) -> (FakeLayouts, ImageOf, Vec<i64>) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let picture = classes.add(lyt::PICTURE_CLASS);
    let fake = FakeLayouts::new(&memory, &classes);
    let (_work, group) = fake.screen(0x14B00000);
    let panes: Vec<i64> = (0..images.len() as i64)
        .map(|i| group + 0x100 * (i + 1))
        .collect();
    for &pane in &panes {
        fake.pane_with_vtable(pane, group, Some(picture));
    }
    fake.track().unwrap();
    let drawn = drawn(panes.iter().copied().zip(images.iter().copied()).collect());
    (fake, drawn, panes)
}

/// Touch mode places each and leaves it out as it does a touch area (`test_touch_targets`).
#[test]
fn test_training_bags_are_offered_by_the_games_names_on_the_field() {
    let (fake, drawn, panes) = drawing(&["sand_bag19.bclim", "f_msg_win_c01.bclim"]);
    let bags = super_training::Bags::new(
        fake.layouts.clone(),
        drawn,
        IndexMap::from([("sand_bag19.bclim".to_string(), "Strength Bag".to_string())]),
    );
    assert_eq!(
        bags.offer(Some(FIELD)).unwrap(),
        IndexMap::from([(panes[0], "Strength Bag".to_string())])
    );
    assert!(
        bags.offer(Some(super_training::SPARRING))
            .unwrap()
            .is_empty()
    );
}

/// The game's text has no name for it, and tapping it can find a bag (spike A).
#[test]
fn test_the_empty_bag_is_offered_but_not_the_one_waiting_behind_the_top_bar() {
    let (fake, drawn, panes) = drawing(&["sand_bag00.bclim", "sand_bag00.bclim"]);
    fake.memory.write(panes[0] + lyt::Y, &floats(&[64.0]));
    fake.memory.write(panes[1] + lyt::Y, &floats(&[120.0]));
    let bags = super_training::Bags::new(fake.layouts.clone(), drawn, IndexMap::new());
    assert_eq!(
        bags.offer(Some(FIELD)).unwrap(),
        IndexMap::from([(panes[0], "empty bag".to_string())])
    );
}

/// A page in a state, its layout drawn or not, whose Pokémon is called "Ralts".
fn core(state: i64, shown: bool) -> (super_training::CoreTraining, Rc<FakeMemory>) {
    let (found, memory, classes) = paged(super_training::ICONS);
    memory.u32(
        PAGE + super_training::ICONS + super_training::STATE,
        &[state],
    );
    let fake = FakeLayouts::new(&memory, &classes);
    let (_work, group) = fake.screen(0x14B00000);
    let picture = classes.add(lyt::PICTURE_CLASS);
    memory.u32(
        group + lyt::CHILD_COUNT,
        &[super_training::PAGE_PANES.len() as i64],
    );
    let mut names = super_training::PAGE_PANES.to_vec();
    names.sort();
    for (i, name) in names.into_iter().enumerate() {
        let pane = group + 0x100 * (i as i64 + 1);
        fake.pane_with_vtable(pane, group, Some(picture));
        memory.write(pane + lyt::NAME, name.as_bytes());
    }
    fake.visible(group - 0x200, shown);
    fake.track().unwrap();
    (
        super_training::CoreTraining::new(
            Rc::new(RefCell::new(found)),
            fake.layouts.clone(),
            Rc::new(|_who| Some("Ralts".to_string())),
        ),
        memory,
    )
}

fn finished() -> (String, String, Option<Category>) {
    (
        "training".to_string(),
        "Ralts finished Core Training".to_string(),
        Some(Category::Screen),
    )
}

#[test]
fn test_a_finished_bag_is_said_once_while_the_pokemon_waits_for_a_tap() {
    let (mut reader, memory) = core(super_training::FINISHED, true);
    assert_eq!(reader.poll(Some(FIELD)).unwrap(), [finished()]);
    assert!(reader.poll(Some(FIELD)).unwrap().is_empty());
    memory.u32(PAGE + super_training::ICONS + super_training::STATE, &[1]);
    assert!(reader.poll(Some(FIELD)).unwrap().is_empty());
    memory.u32(
        PAGE + super_training::ICONS + super_training::STATE,
        &[super_training::FINISHED],
    );
    assert_eq!(reader.poll(Some(FIELD)).unwrap(), [finished()]);
}

/// A sweep finds three objects of the class and one is the page; the others hold noise.
#[test]
fn test_the_state_is_read_from_the_object_that_holds_the_pages_icons() {
    let (mut found, memory, _classes) = paged(super_training::ICONS);
    let decoy = PAGE + 0x4000;
    memory.write(
        decoy,
        &crate::core::memory::Memory::read(memory.as_ref(), PAGE, 4).unwrap(),
    );
    memory.u32(decoy + super_training::ICONS, &[OTHER]);
    memory.u32(
        decoy + super_training::ICONS + super_training::PARAMS,
        &[PARAMS],
    );
    memory.u32(decoy + super_training::ICONS + super_training::STATE, &[9]);
    memory.u32(
        PAGE + super_training::ICONS + super_training::STATE,
        &[super_training::FINISHED],
    );
    found
        .track(
            &IndexMap::from([
                (decoy, super_training::CLASS.to_string()),
                (PAGE, super_training::CLASS.to_string()),
            ]),
            false,
        )
        .unwrap();
    let state = found.selected().unwrap();
    assert!(state.is_some_and(|state| state.0 == super_training::FINISHED));
}

#[test]
fn test_a_finished_bag_is_not_said_with_the_page_put_away_or_in_another_app() {
    assert!(
        core(super_training::FINISHED, false)
            .0
            .poll(Some(FIELD))
            .unwrap()
            .is_empty()
    );
    assert!(
        core(super_training::FINISHED, true)
            .0
            .poll(Some(super_training::MENU))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn test_a_state_not_measured_goes_to_the_log_alone() {
    let (mut reader, _memory) = core(5, true);
    assert_eq!(
        reader.poll(Some(FIELD)).unwrap(),
        [("training".to_string(), "state 5".to_string(), None)]
    );
    assert!(reader.poll(Some(FIELD)).unwrap().is_empty());
}

#[test]
fn test_the_regimen_menus_medal_count_is_said_as_one() {
    let menu = IndexMap::from([
        (1, ScreenText::new(1, "x 0", false)),
        (2, ScreenText::new(2, "1/1", false)),
    ]);
    let medals = super_training::Medals::new(Rc::new(|| Some(super_training::MENU.to_string())));
    assert_eq!(
        medals
            .reworded(menu.clone())
            .unwrap()
            .values()
            .map(|piece| piece.text.as_str())
            .collect::<Vec<_>>(),
        ["0 medals", "1/1"]
    );
    assert_eq!(
        super_training::Medals::new(Rc::new(|| Some(FIELD.to_string())))
            .reworded(menu.clone())
            .unwrap(),
        menu
    );
}

/// Super Training's welcome waits for a tap on no button (spike 253).
#[test]
fn test_a_messages_waiting_arrow_is_offered_as_continue_on_the_field() {
    let (fake, drawn, panes) = drawing(&["sand_bag19.bclim", super_training::ARROW]);
    let waiting = super_training::Waiting::new(fake.layouts.clone(), drawn);
    assert_eq!(
        waiting.offer(Some(FIELD)).unwrap(),
        IndexMap::from([(panes[1], "continue".to_string())])
    );
    assert!(
        waiting
            .offer(Some(super_training::SPARRING))
            .unwrap()
            .is_empty()
    );
}

/// Pictures of a text's characters 20 apart from `x`, "/" the score's slash.
fn digits(text: &str, y: f64, x: f64, image: &str) -> Vec<super_training::Seen> {
    text.chars()
        .enumerate()
        .map(|(i, c)| {
            let what = if c == '/' {
                "slash".to_string()
            } else {
                c.to_string()
            };
            (image.replace("{}", &what), x + 20.0 * i as f64, y)
        })
        .collect()
}

fn score(text: &str) -> Vec<super_training::Seen> {
    digits(text, 203.0, 257.0, "score_nom{}.bclim")
}

/// The clock's face and its digits, as measured: "2:28" at x 352, 373, 391.
fn clock(left: i64) -> Vec<super_training::Seen> {
    let (minutes, seconds) = (left / 60, left % 60);
    let mut out = vec![("timer_c.bclim".to_string(), 325.0, 30.0)];
    out.extend(digits(
        &format!("{minutes}{seconds:02}"),
        30.0,
        352.0,
        "num{}_c.bclim",
    ));
    out
}

fn seen(items: &[(&str, f64, f64)]) -> Vec<super_training::Seen> {
    items
        .iter()
        .map(|(image, x, y)| (image.to_string(), *x, *y))
        .collect()
}

#[test]
fn test_the_score_and_the_target_are_read_from_the_row_of_the_shown_slash() {
    let row = seen(&[
        ("score_nom1.bclim", 257.0, 203.0),
        ("score_nom5.bclim", 277.0, 203.0),
        ("score_nom7.bclim", 297.0, 203.0),
        ("score_nomslash.bclim", 312.0, 203.0),
        ("score_nom3.bclim", 347.0, 203.0),
        ("score_nom0.bclim", 367.0, 203.0),
        ("score_nom0.bclim", 387.0, 203.0),
        ("score_nom9.bclim", 300.0, 64.0),
        ("score_nom8.bclim", 300.0, 292.0),
    ]);
    assert_eq!(super_training::read_score(&row).unwrap(), Some((157, 300)));
}

/// A shot's "-8" pops up at y 200, and read "80 of 50" for 0 (as_super_training_tutorial).
#[test]
fn test_a_points_popup_level_with_the_score_is_not_read_as_part_of_it() {
    let row = seen(&[
        ("score_nom8.bclim", 159.0, 200.0),
        ("score_nom0.bclim", 297.0, 203.0),
        ("score_nomslash.bclim", 312.0, 203.0),
        ("score_nom5.bclim", 326.0, 203.0),
        ("score_nom0.bclim", 340.0, 203.0),
    ]);
    assert_eq!(super_training::read_score(&row).unwrap(), Some((0, 50)));
}

fn rounds(times: Rc<Cell<f64>>) -> (super_training::Round, Rc<FakeMemory>, FakeClasses) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let sparring = super_training::Round::new(
        fake.layouts.clone(),
        Rc::new(|_pane| Ok(None)),
        memory.clone(),
        Some(Rc::new(move || times.get())),
    );
    (sparring, memory, classes)
}

fn screen(line: &str) -> (String, Category) {
    (line.to_string(), Category::Screen)
}

#[test]
fn test_a_score_that_moves_faster_than_every_two_seconds_is_said_once_with_its_newest() {
    let times = Rc::new(Cell::new(0.0));
    let mut sparring = rounds(times.clone()).0;
    let mut said: Vec<(String, Category)> = Vec::new();
    for (at, now) in [
        (0.0, "0/300"),
        (0.0, "0/300"),
        (0.1, "10/300"),
        (0.2, "10/300"),
        (0.5, "20/300"),
        (0.6, "20/300"),
        (1.0, "30/300"),
        (2.2, "30/300"),
        (3.0, "30/300"),
    ] {
        times.set(at);
        said.extend(sparring.hear(&score(now)).unwrap());
    }
    assert_eq!(said, [screen("10 of 300"), screen("30 of 300")]);
}

/// A hit from 0 to 177 drew its last digit a frame before the rest (spike 200, live).
#[test]
fn test_a_score_read_mid_change_is_not_said() {
    let mut sparring = rounds(Rc::new(Cell::new(0.0))).0;
    let mut said = Vec::new();
    for now in ["0/300", "0/300", "7/300", "177/300", "177/300"] {
        said.extend(sparring.hear(&score(now)).unwrap());
    }
    assert_eq!(said, [screen("177 of 300")]);
}

#[test]
fn test_the_clock_is_said_once_as_it_passes_a_half_minute_and_not_at_2_28() {
    let mut sparring = rounds(Rc::new(Cell::new(0.0))).0;
    let mut said = Vec::new();
    for left in [151, 149, 148] {
        said.extend(sparring.hear(&clock(left)).unwrap());
    }
    assert_eq!(said, [screen("2:30 left")]);
}

#[test]
fn test_a_banner_is_said_once_while_shown_and_again_once_hidden_between() {
    let mut sparring = rounds(Rc::new(Cell::new(0.0))).0;
    let start = seen(&[("start.bclim", 200.0, 120.0)]);
    let mut said = Vec::new();
    for pictures in [start.clone(), start.clone(), Vec::new(), start] {
        said.extend(sparring.hear(&pictures).unwrap());
    }
    assert_eq!(said, [screen("start"), screen("start")]);
}

#[test]
fn test_the_banners_are_said_by_their_textures_words() {
    assert_eq!(
        ["start", "clear", "cleared", "gameover_c", "target01_8"]
            .iter()
            .map(|image| super_training::banner(&format!("{image}.bclim")))
            .collect::<Vec<_>>(),
        [
            Some("start".to_string()),
            Some("clear".to_string()),
            Some("cleared".to_string()),
            Some("game over".to_string()),
            None
        ]
    );
}

#[test]
fn test_a_ball_of_the_bots_going_into_flight_warns_once() {
    let (mut sparring, memory, classes) = rounds(Rc::new(Cell::new(0.0)));
    let vtable = classes.add(super_training::MISSILE);
    memory.u32(0x08C00000, &[vtable]);
    sparring
        .track(
            &IndexMap::from([(0x08C00000, super_training::MISSILE.to_string())]),
            false,
        )
        .unwrap();
    let mut said = Vec::new();
    for state in [0u32, 1, 1, 2] {
        memory.write(0x08C00004, &state.to_le_bytes());
        memory.u32(0x08C00000 + super_training::BOT_BALL, &[0x08D00000]);
        said.extend(sparring.poll(Some(super_training::SPARRING)).unwrap());
    }
    assert_eq!(
        said,
        [(
            super_training::ROUND_LOG.to_string(),
            "shot coming".to_string(),
            Category::Warning
        )]
    );
}

#[test]
fn test_a_ball_with_no_pointer_going_into_flight_does_not_warn() {
    let (mut sparring, memory, classes) = rounds(Rc::new(Cell::new(0.0)));
    memory.u32(0x08C00000, &[classes.add(super_training::MISSILE)]);
    sparring
        .track(
            &IndexMap::from([(0x08C00000, super_training::MISSILE.to_string())]),
            false,
        )
        .unwrap();
    let mut said = Vec::new();
    for state in [2, 1] {
        memory.u32(0x08C00004, &[state]);
        said.extend(sparring.poll(Some(super_training::SPARRING)).unwrap());
    }
    assert!(said.is_empty());
}
