//! Super Training: the party's slots are named by species, a round's goals are tapped at the pixel
//! that shoots them, and the training bags in the page's list are offered by the game's names.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_battle::record_of;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::games::common::field::keys::{
    BASE_STATS, DEFAULT_CONTEXTS, ROUND_PLACE, TRAINING_ROUND,
};
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
const STATS: [&str; 6] = ["HP", "Attack", "Defense", "Sp. Atk", "Sp. Def", "Speed"];

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
            Target {
                held: true,
                ..Target::new(goal, "goal 1", 126, 98, -3.0, 32.0)
            },
            Target {
                held: true,
                ..Target::new(red, "red goal 2", 126, 98, -3.0, 32.0)
            }
        ]
    );
    assert!(
        goals
            .targets(Some("N5field12FieldmapProcE"))
            .unwrap()
            .is_empty()
    );
}

/// A player did not know there could be more than one (2026-11-10).
#[test]
fn test_goals_are_numbered_from_the_left_only_when_several_are_up() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let picture = classes.add(lyt::PICTURE_CLASS);
    let fake = FakeLayouts::new(&memory, &classes);
    let (_work, group) = fake.screen_with(0x14B00000, true, true);
    memory.u32(group + lyt::CHILD_COUNT, &[2]);
    let (right, left) = (group + 0x100, group + 0x200);
    for (pane, x) in [(right, 60.0), (left, -60.0)] {
        fake.pane_with_vtable(pane, group, Some(picture));
        memory.write(pane + lyt::X, &floats(&[x]));
    }
    fake.track().unwrap();
    let drawn_now: Rc<RefCell<IndexMap<i64, &'static str>>> =
        Rc::new(RefCell::new(IndexMap::from([
            (right, "target01_8.bclim"),
            (left, "target01_8.bclim"),
        ])));
    let images = drawn_now.clone();
    let goals = super_training::Goals::new(
        fake.layouts.clone(),
        Rc::new(move |pane| Ok(images.borrow().get(&pane).map(|image| image.to_string()))),
    );
    let named: IndexMap<i64, String> = goals
        .targets(Some(super_training::SPARRING))
        .unwrap()
        .into_iter()
        .map(|target| (target.pane, target.name))
        .collect();
    assert_eq!(
        named,
        IndexMap::from([(left, "goal 1".to_string()), (right, "goal 2".to_string())])
    );
    drawn_now.borrow_mut().shift_remove(&left);
    fake.visible(left, false);
    fake.layouts.refresh();
    assert_eq!(
        goals
            .targets(Some(super_training::SPARRING))
            .unwrap()
            .into_iter()
            .map(|t| t.name)
            .collect::<Vec<_>>(),
        ["goal"]
    );
    // A Bot's barrier is shot as a goal is, and never numbered among them.
    drawn_now.borrow_mut().insert(left, "barrier_target.bclim");
    fake.visible(left, true);
    fake.layouts.refresh();
    assert_eq!(
        goals
            .targets(Some(super_training::SPARRING))
            .unwrap()
            .into_iter()
            .map(|t| (t.name, t.held))
            .collect::<Vec<_>>(),
        [("goal".to_string(), true), ("barrier".to_string(), true)]
    );
}

/// Its target circles 41 pixels out every 11.9 s: aimed where it was, 50 shots missed it, and
/// led by its speed and turn the first broke it (spike 200, live on Graveler's round).
#[test]
fn test_a_barrier_is_shot_where_it_will_be_as_the_shot_lands() {
    let turn = 2.0 * std::f64::consts::PI / 11.9;
    let at = |when: f64| (41.0 * (turn * when).cos(), 41.0 * (turn * when).sin());
    let dist = |a: (f64, f64), b: (f64, f64)| crate::py::hypot(a.0 - b.0, a.1 - b.1);
    let seen: Vec<(f64, f64, f64)> = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
        .into_iter()
        .map(|when| (when, at(when).0, at(when).1))
        .collect();
    let led = super_training::ahead(&seen, 1.1, 1.27).unwrap_or((f64::INFINITY, f64::INFINITY));
    assert!(dist(led, at(2.37)) < 0.1);
    assert!(dist(at(1.0), at(2.37)) > 25.0);
    assert_eq!(
        super_training::ahead(&seen, 1.0 + super_training::LEAD_STALE + 0.1, 1.0),
        None
    );
    assert_eq!(
        super_training::ahead(&seen[seen.len() - 2..], 1.0, 1.0),
        None
    );
    // Going straight, it is led along its line.
    let line: Vec<(f64, f64, f64)> = [0.0, 0.4, 0.8]
        .into_iter()
        .map(|when| (when, 10.0 * when, 5.0))
        .collect();
    assert_eq!(super_training::ahead(&line, 0.8, 1.0), Some((18.0, 5.0)));
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

/// The game's text has no name for it, and tapping it can find a bag (spike 290).
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

/// What `core`'s page plays, in order: Python's `PLAYED`.
type Played = Rc<RefCell<Vec<&'static str>>>;

/// A page in a state, its layout drawn or not, whose Pokémon is called "Ralts", touch mode
/// having last tapped never.
fn core(state: i64, shown: bool) -> (super_training::CoreTraining, Rc<FakeMemory>) {
    let (reader, memory, _played) = played_core(state, shown, f64::NEG_INFINITY);
    (reader, memory)
}

/// `core`, touch mode having last tapped `tapped`, with what the page plays.
fn played_core(
    state: i64,
    shown: bool,
    tapped: f64,
) -> (super_training::CoreTraining, Rc<FakeMemory>, Played) {
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
    let played: Played = Rc::new(RefCell::new(Vec::new()));
    let (hit, finished) = (played.clone(), played.clone());
    (
        super_training::CoreTraining::with(
            Rc::new(RefCell::new(found)),
            fake.layouts.clone(),
            Rc::new(|_who| Some("Ralts".to_string())),
            Rc::new(move || hit.borrow_mut().push("hit")),
            Rc::new(move || finished.borrow_mut().push("finished")),
            Rc::new(move || tapped),
        ),
        memory,
        played,
    )
}

#[test]
fn test_each_hit_of_a_tap_landing_on_the_bag_is_heard_and_logged_once() {
    let (mut reader, memory, played) = played_core(1, true, crate::py::time::monotonic());
    let hitting = PAGE + super_training::ICONS + super_training::HITTING;
    let mut said = Vec::new();
    for now in [0, 1, 1, 0, 1] {
        memory.u32(hitting, &[now]);
        said.extend(reader.poll(Some(FIELD)).unwrap());
    }
    assert_eq!(*played.borrow(), ["hit", "hit"]);
    assert_eq!(
        said,
        vec![("training".to_string(), "hit".to_string(), None); 2]
    );
}

/// Once a minute, with no tap, even on an empty bag while the player walks.
#[test]
fn test_the_pokemons_own_hit_is_logged_and_not_heard() {
    let (mut reader, memory, played) = played_core(
        1,
        true,
        crate::py::time::monotonic() - 2.0 * super_training::TAPPED_HIT,
    );
    memory.u32(PAGE + super_training::ICONS + super_training::HITTING, &[1]);
    assert_eq!(
        reader.poll(Some(FIELD)).unwrap(),
        [("training".to_string(), "hit".to_string(), None)]
    );
    assert!(played.borrow().is_empty());
}

/// Taps 0.42 s apart kept the flag up from one swing to the next
/// (as_super_training_large_training_bag).
#[test]
fn test_a_swing_is_heard_as_the_pokemon_starts_it_with_the_hit_flag_still_up() {
    let (mut reader, memory, played) = played_core(1, true, crate::py::time::monotonic());
    memory.u32(PAGE + super_training::ICONS + super_training::HITTING, &[1]);
    for state in [6, 1, 6, 6, 1, 6] {
        memory.u32(
            PAGE + super_training::ICONS + super_training::STATE,
            &[state],
        );
        reader.poll(Some(FIELD)).unwrap();
    }
    assert_eq!(*played.borrow(), ["hit"; 3]);
}

#[test]
fn test_a_finished_bag_is_heard_with_its_words() {
    let (mut reader, _memory, played) =
        played_core(super_training::FINISHED, true, f64::NEG_INFINITY);
    for _ in 0..3 {
        reader.poll(Some(FIELD)).unwrap();
    }
    assert_eq!(*played.borrow(), ["finished"]);
}

/// A tap that finishes the bag goes on to what it gave: finished for one poll, with no "!" to
/// tap (as_st_tutorial2).
#[test]
fn test_a_bag_finished_by_a_tap_that_goes_straight_on_is_not_said() {
    let (mut reader, memory) = core(super_training::FINISHED, true);
    assert!(reader.poll(Some(FIELD)).unwrap().is_empty());
    memory.u32(PAGE + super_training::ICONS + super_training::STATE, &[8]);
    assert!(reader.poll(Some(FIELD)).unwrap().is_empty());
}

/// Writes a record's effort values, in the record's order, re-encrypting it as the game keeps it.
fn write_efforts(memory: &FakeMemory, param: i64, points: [i64; 6]) {
    let raw = memory
        .read(param + pokemon::RECORD_IN_PARAM, pokemon::RECORD_SIZE)
        .unwrap();
    let mut plain = pokemon::decrypt(&raw).unwrap();
    for (i, value) in points.into_iter().enumerate() {
        plain[pokemon::EFFORT as usize + i] = value as u8;
    }
    let checksum = plain[pokemon::PLAIN as usize..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|word| u16::from_le_bytes(*word) as i64)
        .sum::<i64>()
        & 0xFFFF;
    let at = pokemon::CHECKSUM as usize;
    plain[at..at + 2].copy_from_slice(&(checksum as u16).to_le_bytes());
    let mut out = plain[..pokemon::PLAIN as usize].to_vec();
    out.extend(pokemon::crypt(
        &plain[pokemon::PLAIN as usize..],
        0x00000101,
    ));
    memory.write(param + pokemon::RECORD_IN_PARAM, &out);
}

/// What `keys` says was wrong, in order.
type Wrong = Rc<RefCell<Vec<String>>>;

/// R's reader over a page whose Pokémon has these effort values in the game's order, drawing
/// this kind of shot in its own pane, and `stray` in another; and the lines heard for nothing to
/// say.
///
/// Python's patches the page's `efforts`; here they are written into the selected Pokémon's
/// record, and None is a page the sweep has not found.
fn keys(
    exact: bool,
    efforts: Option<[i64; 6]>,
    shot: &'static str,
    stray: Option<&'static str>,
) -> (super_training::TrainingKeys, Wrong) {
    let (mut found, page_memory, _classes) = paged(super_training::ICONS);
    match efforts {
        Some(efforts) => {
            let param = page_memory
                .read(PAGE + super_training::ICONS + super_training::PARAMS, 4)
                .unwrap();
            let param = i64::from(u32::from_le_bytes(param.try_into().unwrap()));
            let mut points = [0; 6];
            for (label, &record) in super_training::STAT_ORDER.iter().enumerate() {
                points[record] = efforts[label];
            }
            write_efforts(&page_memory, param, points);
        }
        None => found.track(&IndexMap::new(), false).unwrap(),
    }
    let found = Rc::new(RefCell::new(found));
    let images: Vec<&'static str> = [stray, Some(shot)].into_iter().flatten().collect();
    let (fake, drawn, panes) = drawing(&images);
    fake.memory.write(
        panes[panes.len() - 1] + fake.layouts.shape.name,
        super_training::SHOT_PANE,
    );
    let memory = FakeMemory::new();
    let sparring = Rc::new(RefCell::new(super_training::Round::new(
        fake.layouts.clone(),
        Rc::new(|_pane| Ok(None)),
        memory,
        Some(Rc::new(|| 0.0)),
    )));
    let wrong: Wrong = Rc::new(RefCell::new(Vec::new()));
    let said_wrong = {
        let wrong = wrong.clone();
        Rc::new(move |why: &str| {
            wrong.borrow_mut().push(why.to_string());
            String::new()
        })
    };
    let core_reader = Rc::new(RefCell::new(super_training::CoreTraining::new(
        found.clone(),
        fake.layouts.clone(),
        Rc::new(|_who| Some("Ralts".to_string())),
    )));
    (
        super_training::TrainingKeys::new(
            core_reader,
            found,
            sparring,
            STATS.iter().map(|stat| stat.to_string()).collect(),
            exact,
            fake.layouts.clone(),
            drawn,
            said_wrong,
        ),
        wrong,
    )
}

/// ADR-0017: the points are an Extra, off as it ships; off, the chart is said in fifths.
#[test]
fn test_r_says_each_stats_rough_share_of_the_chart_the_total_and_the_shot() {
    let (reader, _wrong) = keys(
        false,
        Some([0, 40, 100, 200, 0, 0]),
        "shot_icon02.bclim",
        None,
    );
    assert_eq!(
        reader.press(BASE_STATS, Some(FIELD)).unwrap(),
        "HP 0 of 5, Attack 1 of 5, Defense 2 of 5, Sp. Atk 4 of 5, Sp. Def 0 of 5, Speed 0 of 5, \
         total 4 of 5, blue shot"
    );
}

/// Another picture named a shot icon, drawn where one of the PlayNav's archive had been, was
/// said beside it: "orange shot, green shot".
#[test]
fn test_r_says_the_shot_of_the_pages_own_pane_alone() {
    let (reader, _wrong) = keys(
        false,
        Some([0, 0, 0, 0, 0, 0]),
        "shot_icon02.bclim",
        Some("shot_icon01.bclim"),
    );
    assert!(
        reader
            .press(BASE_STATS, Some(FIELD))
            .unwrap()
            .ends_with("total 0 of 5, blue shot")
    );
}

#[test]
fn test_a_stat_is_0_of_5_only_with_no_points_and_5_of_5_only_when_full() {
    let (reader, _wrong) = keys(
        false,
        Some([252, 251, 1, 0, 6, 0]),
        "shot_icon02.bclim",
        None,
    );
    assert!(
        reader.press(BASE_STATS, Some(FIELD)).unwrap().starts_with(
            "HP 5 of 5, Attack 4 of 5, Defense 1 of 5, Sp. Atk 0 of 5, Sp. Def 1 of 5"
        )
    );
}

/// ADR-0017: the Extra says the numbers the chart is drawn from, in the share's place.
#[test]
fn test_r_says_the_points_instead_with_the_extra_on() {
    let (reader, _wrong) = keys(true, Some([2, 1, 4, 0, 0, 2]), "shot_icon02.bclim", None);
    assert_eq!(
        reader.press(BASE_STATS, Some(FIELD)).unwrap(),
        "HP 2, Attack 1, Defense 4, Sp. Atk 0, Sp. Def 0, Speed 2, total 9 of 510, blue shot"
    );
}

#[test]
fn test_r_with_nothing_to_read_goes_to_wrong() {
    let (reader, wrong) = keys(false, None, "shot_icon02.bclim", None);
    assert_eq!(reader.press(BASE_STATS, Some(FIELD)).unwrap(), "");
    assert_eq!(*wrong.borrow(), ["no numbers yet"]);
}

#[test]
fn test_rs_table_follows_the_page_and_a_round() {
    let reader = keys(false, Some([2, 1, 4, 0, 0, 2]), "shot_icon02.bclim", None).0;
    assert_eq!(
        reader.context(Some(super_training::SPARRING)).unwrap(),
        Some(TRAINING_ROUND)
    );
    // this fake's layout is not the page's
    assert_eq!(reader.context(Some(FIELD)).unwrap(), None);
}

#[test]
fn test_r_says_a_rounds_score_and_clock_as_last_read() {
    let mut sparring = rounds(Rc::new(Cell::new(0.0))).0;
    assert_eq!(sparring.numbers().unwrap(), "");
    for _ in 0..2 {
        sparring
            .hear(&[score("34/50"), clock(88)].concat())
            .unwrap();
    }
    assert_eq!(sparring.numbers().unwrap(), "34 of 50, 1:28 left");
}

/// The first tutorial's round draws none (as_st_tutorial2).
#[test]
fn test_r_says_the_energy_meter_by_its_scale_where_the_round_draws_one() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let picture = classes.add(lyt::PICTURE_CLASS);
    let (_work, group) = fake.screen_with(0x14B00000, true, true);
    memory.u32(group + lyt::CHILD_COUNT, &[1]);
    fake.pane_with_vtable(group + 0x100, group, Some(picture));
    memory.write(group + 0x100 + lyt::SCALE_X, &floats(&[0.72]));
    fake.track().unwrap();
    let mut sparring = super_training::Round::new(
        fake.layouts.clone(),
        drawn(IndexMap::from([(
            group + 0x100,
            super_training::ENERGY_IMAGE,
        )])),
        memory.clone(),
        Some(Rc::new(|| 0.0)),
    );
    sparring.poll(Some(super_training::SPARRING)).unwrap();
    assert_eq!(sparring.numbers().unwrap(), "energy 4 of 5");
}

/// A round drawing one picture of the aura, shown or not by the second value of `fake.visible`
/// through the returned panes; and its layouts, which Python's keeps in `FAKES`.
fn charging(
    holding: Rc<Cell<bool>>,
) -> (
    super_training::Round,
    Vec<i64>,
    Rc<FakeMemory>,
    i64,
    FakeLayouts,
) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let picture = classes.add(lyt::PICTURE_CLASS);
    let (_work, group) = fake.screen_with(0x14B00000, true, true);
    memory.u32(group + lyt::CHILD_COUNT, &[1]);
    let aura = group + 0x100;
    fake.pane_with_vtable(aura, group, Some(picture));
    fake.track().unwrap();
    let sparring = super_training::Round::with(
        fake.layouts.clone(),
        drawn(IndexMap::from([(aura, super_training::CHARGE_IMAGE)])),
        memory.clone(),
        super_training::RoundOptions {
            now: Some(Rc::new(|| 0.0)),
            holding: Some(Rc::new(move || holding.get())),
            ..super_training::RoundOptions::default()
        },
    );
    (sparring, vec![aura], memory, group, fake)
}

fn warning(line: &str) -> (String, String, Option<Category>) {
    (
        "round".to_string(),
        line.to_string(),
        Some(Category::Warning),
    )
}

#[test]
fn test_the_aura_of_a_press_held_is_said_as_it_comes_and_if_it_burns_out_held() {
    let holding = Rc::new(Cell::new(true));
    let (mut sparring, panes, _memory, _group, fake) = charging(holding);
    assert_eq!(
        sparring.poll(Some(super_training::SPARRING)).unwrap(),
        [warning("charged")]
    );
    assert!(
        sparring
            .poll(Some(super_training::SPARRING))
            .unwrap()
            .is_empty()
    );
    fake.visible(panes[0], false);
    fake.layouts.refresh();
    assert_eq!(
        (0..super_training::AURA_GONE)
            .map(|_| sparring.poll(Some(super_training::SPARRING)).unwrap())
            .collect::<Vec<_>>(),
        [vec![], vec![], vec![warning("fizzled")]]
    );
}

/// Its panes go undrawn for a poll now and then (as_st_l1_regimen).
#[test]
fn test_the_aura_flickering_is_said_once_and_never_fizzles() {
    let (mut sparring, panes, _memory, _group, fake) = charging(Rc::new(Cell::new(true)));
    let mut said = sparring.poll(Some(super_training::SPARRING)).unwrap();
    for shown in [false, true, false, false, true] {
        fake.visible(panes[0], shown);
        fake.layouts.refresh();
        said.extend(sparring.poll(Some(super_training::SPARRING)).unwrap());
    }
    assert_eq!(said, [warning("charged")]);
}

#[test]
fn test_the_aura_going_as_the_press_is_let_go_says_nothing() {
    let holding = Rc::new(Cell::new(true));
    let (mut sparring, panes, _memory, _group, fake) = charging(holding.clone());
    sparring.poll(Some(super_training::SPARRING)).unwrap();
    holding.set(false);
    fake.visible(panes[0], false);
    fake.layouts.refresh();
    assert_eq!(
        (0..super_training::AURA_GONE + 1)
            .map(|_| sparring.poll(Some(super_training::SPARRING)).unwrap())
            .collect::<Vec<_>>(),
        vec![Vec::new(); 4]
    );
}

/// The record's: HP, Attack, Defense, Speed, Sp. Atk, Sp. Def.
#[test]
fn test_the_game_labels_its_stats_in_another_order_than_the_record_keeps_them() {
    let (found, memory, _classes) = paged(super_training::ICONS);
    let param = PARAMS;
    write_efforts(&memory, param, [1, 2, 3, 4, 5, 6]);
    // The selected record is the second built, Sceptile's; the first is `PARAMS`.
    memory.u32(
        PAGE + super_training::ICONS + super_training::PARAMS,
        &[param],
    );
    assert_eq!(found.efforts().unwrap(), Some(vec![1, 2, 3, 5, 6, 4]));
}

/// A piece of words at a row, as the screen text reader makes one.
fn worded(r#box: i64, text: &str, row: f64) -> ScreenText {
    ScreenText {
        row,
        glyph: 12.0,
        ..ScreenText::new(r#box, text, false)
    }
}

/// A regimen's description: its header, the six stats a row each, and its target time.
fn gains_screen(gain_row: Option<f64>) -> IndexMap<i64, ScreenText> {
    let mut pieces = vec![worded(1, "Possible base stat increase:", 250.0)];
    for (i, stat) in STATS.iter().enumerate() {
        pieces.push(worded(2 + i as i64, stat, 220.0 - 30.0 * i as f64));
    }
    pieces.push(worded(20, "Target Time Remaining:", 119.0));
    pieces.push(worded(21, "2:30.0", 141.0));
    if let Some(gain_row) = gain_row {
        pieces.push(worded(30, "+4", gain_row));
    }
    pieces
        .into_iter()
        .map(|piece| (piece.r#box, piece))
        .collect()
}

const HEADERS: [&str; 2] = ["Possible base stat increase:", "Base stat increase:"];

fn gains(running: &'static str) -> super_training::Gains {
    super_training::Gains::new(
        Rc::new(move || Some(running.to_string())),
        &STATS.map(str::to_string),
        &HEADERS.map(str::to_string),
    )
}

fn texts(found: &IndexMap<i64, ScreenText>) -> Vec<&str> {
    found.values().map(|piece| piece.text.as_str()).collect()
}

/// The six stats, read in build order among its target time, ran together.
#[test]
fn test_a_regimens_gain_is_said_beside_its_stat_and_the_others_left_out() {
    let gains = gains(super_training::MENU);
    assert_eq!(
        texts(&gains.reworded(gains_screen(Some(70.0))).unwrap()),
        [
            "Possible base stat increase: Speed +4",
            "Target Time Remaining:",
            "2:30.0"
        ]
    );
}

/// Another layout's "HP" on the gain's row, with one of the regimen's labels not up yet,
/// took the "+4" a second time and threw.
#[test]
fn test_a_regimens_gain_is_said_beside_its_own_layouts_stat_alone() {
    let mut screen = gains_screen(Some(70.0));
    screen.shift_remove(&3);
    screen.insert(
        40,
        ScreenText {
            root: 1,
            ..worded(40, "HP", 70.0)
        },
    );
    let gains = gains(super_training::MENU);
    assert_eq!(gains.reworded(screen.clone()).unwrap(), screen);
    screen.insert(3, gains_screen(Some(70.0))[&3].clone());
    assert_eq!(
        texts(&gains.reworded(screen).unwrap()),
        [
            "Possible base stat increase: Speed +4",
            "Target Time Remaining:",
            "2:30.0",
            "HP"
        ]
    );
}

/// Until the game first draws a layout just built, its panes read at (0, 0): every stat was on
/// the gain's row, and a second stat took the "+4" again and threw.
#[test]
fn test_a_regimens_gain_is_taken_once_before_its_layout_is_first_drawn() {
    let unplaced: IndexMap<i64, ScreenText> = gains_screen(Some(70.0))
        .into_iter()
        .map(|(r#box, piece)| {
            (
                r#box,
                ScreenText {
                    glyph: 12.0,
                    ..ScreenText::new(r#box, piece.text, false)
                },
            )
        })
        .collect();
    let gains = gains(super_training::MENU);
    assert_eq!(
        texts(&gains.reworded(unplaced).unwrap()),
        [
            "Possible base stat increase: HP +4",
            "Target Time Remaining:",
            "2:30.0"
        ]
    );
}

#[test]
fn test_a_regimens_header_waits_for_its_gain_and_other_screens_are_left_alone() {
    let gains = gains(super_training::MENU);
    assert_eq!(
        texts(&gains.reworded(gains_screen(None)).unwrap()),
        [
            "Possible base stat increase:",
            "Target Time Remaining:",
            "2:30.0"
        ]
    );
    let elsewhere = self::gains(FIELD);
    assert_eq!(
        elsewhere.reworded(gains_screen(Some(70.0))).unwrap(),
        gains_screen(Some(70.0))
    );
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
    assert!(reader.poll(Some(FIELD)).unwrap().is_empty());
    assert_eq!(reader.poll(Some(FIELD)).unwrap(), [finished()]);
    assert!(reader.poll(Some(FIELD)).unwrap().is_empty());
    memory.u32(PAGE + super_training::ICONS + super_training::STATE, &[1]);
    assert!(reader.poll(Some(FIELD)).unwrap().is_empty());
    memory.u32(
        PAGE + super_training::ICONS + super_training::STATE,
        &[super_training::FINISHED],
    );
    assert!(reader.poll(Some(FIELD)).unwrap().is_empty());
    assert_eq!(reader.poll(Some(FIELD)).unwrap(), [finished()]);
}

/// Treecko, chosen, rested at once, and the game said so only once a regimen was tried; a tap
/// woke it, to state 2 (as_st_graveler_barrier).
#[test]
fn test_a_resting_pokemon_is_said_as_the_page_shows_it_and_again_on_coming_back() {
    let resting = || {
        vec![(
            "training".to_string(),
            "Ralts is resting".to_string(),
            Some(Category::Screen),
        )]
    };
    let (mut reader, memory) = core(super_training::RESTING, true);
    assert_eq!(reader.poll(Some(FIELD)).unwrap(), resting());
    assert!(reader.poll(Some(FIELD)).unwrap().is_empty());
    memory.u32(PAGE + super_training::ICONS + super_training::STATE, &[2]);
    assert!(reader.poll(Some(FIELD)).unwrap().is_empty());
    memory.u32(
        PAGE + super_training::ICONS + super_training::STATE,
        &[super_training::RESTING],
    );
    assert_eq!(reader.poll(Some(FIELD)).unwrap(), resting());
    assert!(reader.poll(Some(super_training::MENU)).unwrap().is_empty());
    assert_eq!(reader.poll(Some(FIELD)).unwrap(), resting());
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
    assert!(state.is_some_and(|state| state.state == super_training::FINISHED));
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

/// A shot's "-8" pops up at y 200, and read "80 of 50" for 0 (as_st_tutorial).
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

const BALL: i64 = 0x08C00000;
const PLAYER_AT: i64 = 0x08C20000;

/// What a round's cues told, in order.
type Heard = Rc<RefCell<Vec<String>>>;

/// A round with a ball of the Bot's, its cues written down as they come.
fn cueing(player_z: Option<f32>, lead: f64) -> (super_training::Round, Rc<FakeMemory>, Heard) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let heard: Heard = Rc::new(RefCell::new(Vec::new()));
    let (fired, now) = (heard.clone(), heard.clone());
    let mut sparring = super_training::Round::with(
        fake.layouts.clone(),
        Rc::new(|_pane| Ok(None)),
        memory.clone(),
        super_training::RoundOptions {
            now: Some(Rc::new(|| 0.0)),
            shot_fired: Some(Rc::new(move |said: &str| {
                fired.borrow_mut().push(said.to_string())
            })),
            block_now: Some(Rc::new(move |said: &str| {
                now.borrow_mut().push(said.to_string())
            })),
            lead,
            ..super_training::RoundOptions::default()
        },
    );
    memory.u32(BALL, &[classes.add(super_training::MISSILE)]);
    memory.u32(BALL + super_training::BOT_BALL, &[0x08D00000]);
    let mut found = IndexMap::from([(BALL, super_training::MISSILE.to_string())]);
    if let Some(player_z) = player_z {
        memory.u32(PLAYER_AT, &[classes.add(super_training::PLAYER)]);
        memory.write(PLAYER_AT + super_training::PLACE_Z, &floats(&[player_z]));
        found.insert(PLAYER_AT, super_training::PLAYER.to_string());
    }
    sparring.track(&found, false).unwrap();
    (sparring, memory, heard)
}

/// In steps of 10, as the map's list says a bearing, up nearer the Bot; its bobbing where it
/// started is no step (the places as_st_l1_regimen_again's census read).
#[test]
fn test_c_says_how_far_up_and_across_the_pokemon_stands_from_where_it_started() {
    let (sparring, memory, _heard) = cueing(Some(228.5), 0.6);
    let mut said: Vec<String> = Vec::new();
    for (x, z) in [(-2.4, 232.2), (-24.9, 159.6), (31.4, 227.2), (-2.4, 240.0)] {
        memory.write(PLAYER_AT + super_training::PLACE_X, &floats(&[x]));
        memory.write(PLAYER_AT + super_training::PLACE_Z, &floats(&[z]));
        said.push(sparring.place().unwrap());
    }
    assert_eq!(said, ["at the start", "7 up 2 left", "3 right", "1 down"]);
    assert_eq!(cueing(None, 0.6).0.place().unwrap(), "");
    assert_eq!(DEFAULT_CONTEXTS[TRAINING_ROUND]["c"], ROUND_PLACE);
}

fn fly(memory: &FakeMemory, z: f32, frames: i64, state: i64) {
    fly_ball(memory, z, frames, state, BALL);
}

/// `fly(memory, z, frames, state, ball)`.
fn fly_ball(memory: &FakeMemory, z: f32, frames: i64, state: i64, ball: i64) {
    memory.u32(ball + super_training::MISSILE_STATE, &[state]);
    memory.write(ball + super_training::PLACE_Z, &floats(&[z]));
    memory.u32(ball + super_training::BALL_FRAMES, &[frames]);
}

#[test]
fn test_a_ball_of_the_bots_going_into_flight_is_cued_once_as_it_leaves() {
    let (mut sparring, memory, heard) = cueing(None, 0.6);
    for state in [0, 1, 1, 2] {
        fly(&memory, 20.0, 3, state);
        assert!(
            sparring
                .poll(Some(super_training::SPARRING))
                .unwrap()
                .is_empty()
        );
    }
    assert_eq!(*heard.borrow(), ["shot coming"]);
}

/// A ball that misses stays in flight where it stopped, and the Bot fires it again with its
/// frames counted from the start: two hit uncued so (as_st_graveler_barrier's tape).
#[test]
fn test_a_ball_fired_again_while_still_in_flight_is_cued_again() {
    let (mut sparring, memory, heard) = cueing(None, 0.6);
    fly(&memory, 0.0, 0, 0);
    sparring.poll(Some(super_training::SPARRING)).unwrap();
    for (z, frames) in [
        (20.0, 3),
        (180.0, 40),
        (230.0, 90),
        (230.0, 140),
        (25.0, 4),
        (90.0, 20),
    ] {
        fly(&memory, z, frames, 1);
        sparring.poll(Some(super_training::SPARRING)).unwrap();
    }
    assert_eq!(*heard.borrow(), ["shot coming", "shot coming"]);
}

/// A ball here comes 4.5 a frame at 30 a second: 0.6 s out it is 81 from the Pokémon.
#[test]
fn test_the_moment_to_block_comes_the_lead_before_the_ball_arrives() {
    let (mut sparring, memory, heard) = cueing(Some(200.0), 0.6);
    fly(&memory, 0.0, 0, 0);
    sparring.poll(Some(super_training::SPARRING)).unwrap();
    fly(&memory, 45.0, 10, 1);
    sparring.poll(Some(super_training::SPARRING)).unwrap();
    assert!(sparring.ticking(Some(super_training::SPARRING)));
    heard.borrow_mut().clear();
    for frames in [20, 26] {
        fly(&memory, 45.0 + 4.5 * (frames - 10) as f32, frames, 1);
        sparring.tick().unwrap();
    }
    assert!(heard.borrow().is_empty());
    fly(&memory, 45.0 + 4.5 * (27 - 10) as f32, 27, 1);
    sparring.tick().unwrap();
    assert_eq!(*heard.borrow(), ["now"]);
    fly(&memory, 45.0 + 4.5 * (30 - 10) as f32, 30, 1);
    sparring.tick().unwrap();
    assert_eq!(*heard.borrow(), ["now"]);
}

/// Graveler's Bot fires four at once, which were four "now"s in 0.02 s; one fired 10 frames
/// behind them, 0.33 s, has its own (as_st_graveler_barrier).
#[test]
fn test_balls_arriving_together_are_cued_with_one_now() {
    let (mut sparring, memory, heard) = cueing(Some(200.0), 0.6);
    let (together, behind) = (BALL + 0x400, BALL + 0x800);
    for ball in [together, behind] {
        memory.write(ball, &memory.read(BALL, 4).unwrap());
        memory.u32(ball + super_training::BOT_BALL, &[0x08D00000]);
    }
    sparring
        .track(
            &IndexMap::from([
                (together, super_training::MISSILE.to_string()),
                (behind, super_training::MISSILE.to_string()),
            ]),
            true,
        )
        .unwrap();
    for ball in [BALL, together, behind] {
        fly_ball(&memory, 0.0, 0, 0, ball);
    }
    sparring.poll(Some(super_training::SPARRING)).unwrap();
    for (ball, z) in [(BALL, 45.0), (together, 45.0), (behind, 0.0)] {
        fly_ball(&memory, z, 10, 1, ball);
    }
    sparring.poll(Some(super_training::SPARRING)).unwrap();
    heard.borrow_mut().clear();
    for frames in [27, 37] {
        for (ball, z) in [(BALL, 45.0f32), (together, 45.0), (behind, 0.0)] {
            fly_ball(&memory, z + 4.5 * (frames - 10) as f32, frames, 1, ball);
        }
        sparring.tick().unwrap();
        assert_eq!(
            *heard.borrow(),
            vec!["now"; if frames == 27 { 1 } else { 2 }]
        );
    }
}

#[test]
fn test_nothing_is_followed_once_the_ball_has_landed() {
    let (mut sparring, memory, _heard) = cueing(None, 0.6);
    fly(&memory, 0.0, 0, 0);
    sparring.poll(Some(super_training::SPARRING)).unwrap();
    fly(&memory, 45.0, 10, 1);
    sparring.poll(Some(super_training::SPARRING)).unwrap();
    fly(&memory, 45.0, 10, 2);
    sparring.tick().unwrap();
    assert!(!sparring.ticking(Some(super_training::SPARRING)));
    assert!(!sparring.holding(Some(super_training::SPARRING)));
}

/// L held as it arrived, no points lost, and the ball's state 3 where a hit's is 2
/// (as_st_tutorial2 against as_st_tutorial).
#[test]
fn test_a_ball_stopped_by_the_barrier_is_said_blocked() {
    let (mut sparring, memory, _heard) = cueing(None, 0.6);
    let mut said = Vec::new();
    for state in [0, 1, 3, 3] {
        memory.u32(BALL + super_training::MISSILE_STATE, &[state]);
        said.extend(sparring.poll(Some(super_training::SPARRING)).unwrap());
    }
    assert_eq!(
        said,
        [(
            super_training::ROUND_LOG.to_string(),
            "blocked".to_string(),
            Some(Category::Screen)
        )]
    );
}

#[test]
fn test_a_ball_with_no_pointer_going_into_flight_is_not_cued() {
    let (mut sparring, memory, heard) = cueing(None, 0.6);
    memory.u32(BALL + super_training::BOT_BALL, &[0]);
    for state in [2, 1] {
        memory.u32(BALL + super_training::MISSILE_STATE, &[state]);
        sparring.poll(Some(super_training::SPARRING)).unwrap();
    }
    assert!(heard.borrow().is_empty());
}
