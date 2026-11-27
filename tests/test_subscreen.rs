//! The subscreens answer for their readers as one, each reader in its own apps alone.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::fake_speech::Recorder;
use super::test_amie::{PAGE, WALKER, page, species};
use crate::core::config::Tables;
use crate::core::rtti::ClassNames;
use crate::core::speech::Speech;
use crate::games::common::field::keys::{BERRIES, BERRY_PICKER};
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::{ImageOf, Target};
use crate::games::gen6::apps::{FIELD, RunningApp, UNNAMED};
use crate::games::gen6::feedback::FeedbackSounds;
use crate::games::gen6::profile;
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::screens::boxes;
use crate::games::gen6::screens::rows::PARTY;
use crate::games::gen6::subscreen::berry_picker::{self, Touching};
use crate::games::gen6::subscreen::o_power::{self, OPower};
use crate::games::gen6::subscreen::{self, Subscreens};
use crate::games::gen6::subscreen::{amie, head_it, pss, puzzle, super_training};
use crate::games::oras;
use crate::py::Result;

/// The PSS's profile, `app::pss::pss_profile::Proc`, one of its own apps.
fn pss_profile() -> String {
    format!("{}11pss_profile4ProcE", pss::PSS)
}

/// `SimpleNamespace(running=app, process=None)`: the app running, which a test changes.
fn running(app: &str) -> Rc<RefCell<RunningApp>> {
    let (_, memory) = page(amie::NUMBER);
    let apps = RunningApp::new(memory.clone(), Rc::new(ClassNames::new(memory, CODE)));
    let apps = Rc::new(RefCell::new(apps));
    apps.borrow_mut().running = Some(app.to_string());
    apps
}

/// The subscreens over the Amie page of `test_amie`, its Pokémon Sceptile, in whichever app
/// `apps.running` says runs.
fn subscreens(apps: Rc<RefCell<RunningApp>>) -> Subscreens {
    subscreens_drawing(apps, Rc::new(|_| Ok(None)))
}

/// `subscreens(apps, image_of)`: with what each picture draws.
fn subscreens_drawing(apps: Rc<RefCell<RunningApp>>, image_of: ImageOf) -> Subscreens {
    let (_, memory) = page(amie::NUMBER);
    let layouts = FakeLayouts::new(
        &memory,
        &FakeClasses::starting(memory.clone(), CODE.0 + 0x8000),
    )
    .layouts
    .clone();
    let found = Subscreens::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        layouts,
        image_of,
        IndexMap::new(),
        Rc::new(GameText {
            species: species(),
            ..GameText::new(Vec::new(), Vec::new())
        }),
        oras::game(),
        apps,
        &Tables::default(),
        Rc::new(FeedbackSounds::new(
            Rc::new(|_line: &str| {}),
            Rc::new(Speech::plain(Recorder::new())),
            None,
            None,
        )),
        Rc::new(|| Ok(None)),
        None,
        Rc::new(|_root: i64| None),
        Rc::new(Vec::new),
        Rc::new(|| None),
        None,
        Rc::new(String::new),
    );
    found
        .track(&IndexMap::from([(PAGE, amie::CLASS.to_string())]), false)
        .unwrap();
    found
}

#[test]
fn test_the_amie_pages_pokemon_is_named_on_the_field_and_offered_to_beckon_in_amie() {
    // The Amie page names its Pokémon on the field, and Amie itself offers it to beckon: each
    // reader says what it has for the app running and nothing in another.
    let apps = running(FIELD);
    let found = subscreens(apps.clone());
    assert_eq!(found.touched(WALKER).unwrap().as_deref(), Some("Sceptile"));
    assert!(found.targets().unwrap().is_empty());
    assert!(found.poll(&mut Untouched).unwrap().is_empty());
    apps.borrow_mut().running = Some(amie::AMIE.to_string());
    assert_eq!(found.touched(WALKER).unwrap(), None);
    let names: Vec<String> = found
        .targets()
        .unwrap()
        .into_iter()
        .map(|target| target.name)
        .collect();
    assert_eq!(names, ["beckon Sceptile", "stylus"]);
    // Keys of its own only while the screen that has them runs.
    apps.borrow_mut().running = Some(berry_picker::APP.to_string());
    assert_eq!(found.context().unwrap(), Some(BERRY_PICKER));
    apps.borrow_mut().running = Some(FIELD.to_string());
    assert_eq!(found.context().unwrap(), None);
    assert_eq!(found.press(BERRIES[0], &mut Untouched).unwrap(), "");
}

/// Amie's games draw a result's count a digit a picture (or_amie_stars): screen text names
/// each by its file in the games alone, and touch mode, named apart, has no name for one.
#[test]
fn test_screen_text_names_a_digit_of_amies_games_and_touch_mode_names_apart() {
    let apps = running(head_it::APP);
    let found = subscreens_drawing(apps.clone(), Rc::new(|_| Ok(Some("7.bclim".to_string()))));
    assert_eq!(found.name(1).unwrap().as_deref(), Some("7"));
    assert_eq!(found.touched(1).unwrap(), None);
    apps.borrow_mut().running = Some(FIELD.to_string());
    assert_eq!(found.name(1).unwrap(), None);
}

#[test]
fn test_touch_mode_in_the_subscreens_apps_is_as_they_say() {
    // ADR-0009: touch mode offers targets in Amie's and Super Training's apps and the PSS's own,
    // beside the profile's, Shift alone drags in Amie's and holds a charge in Super Training's
    // rounds, and Berry Picker's drag is made on letting go, at full speed until its clock is seen
    // to run.
    let mut drags: Vec<&str> = amie::SCREENS.to_vec();
    drags.push(super_training::SPARRING);
    assert_eq!(subscreen::DRAGS.to_vec(), drags);
    let apps = running(berry_picker::APP);
    let found = subscreens(apps.clone());
    assert!(found.delayed());
    let profile_app = pss_profile();
    let mut offered: Vec<&str> = vec![amie::AMIE];
    offered.extend(amie::GAMES);
    offered.extend([
        super_training::SPARRING,
        pss::FRIEND_ENTRY,
        pss::COLOSSEUM,
        pss::DIALOG,
        &profile_app,
    ]);
    for app in offered {
        apps.borrow_mut().running = Some(app.to_string());
        assert!(found.offered(), "{app}");
    }
    assert!(found.in_pss());
    let mut others: Vec<Option<&str>> = profile::TOUCHED.iter().map(|app| Some(*app)).collect();
    others.extend([None, Some(boxes::APP)]);
    for app in others {
        apps.borrow_mut().running = app.map(String::from);
        assert!(!found.offered() && !found.in_pss(), "{app:?}");
    }
    apps.borrow_mut().running = Some(amie::AMIE.to_string());
    assert!(!found.delayed() && !found.in_pss());
    assert_eq!(found.pace(), 1.0);
}

/// ADR-0013: a clock is said at its marks while a screen that times the player runs. The
/// party opened from the Colosseum before a battle with another player counts down from 1:39,
/// and was said once and never ticked (spike 319); the party opened from the field is not.
#[test]
fn test_the_team_choice_of_a_battle_in_the_pss_is_timed() {
    let apps = running(pss::COLOSSEUM);
    let found = subscreens(apps.clone());
    for app in [pss::COLOSSEUM, PARTY] {
        apps.borrow_mut().running = Some(app.to_string());
        found.started();
    }
    assert!(found.timed());
    for app in [FIELD, PARTY] {
        apps.borrow_mut().running = Some(app.to_string());
        found.started();
    }
    assert!(!found.timed());
    // Alpha Sapphire ran an app no class name is found for between the two, for 0.2 s.
    for app in [
        pss::COLOSSEUM.to_string(),
        format!("{UNNAMED} 0x006f9a28"),
        PARTY.to_string(),
    ] {
        apps.borrow_mut().running = Some(app);
        found.started();
    }
    assert!(found.timed());
}

/// Amie's games time the player, their clocks said at their marks (ADR-0013); and Tile
/// Puzzle's picture is new with any app started, even one of the same Pokémon.
#[test]
fn test_amies_games_are_timed_and_another_app_starts_tile_puzzles_picture_afresh() {
    let apps = running(puzzle::APP);
    let found = subscreens(apps.clone());
    assert!(found.timed());
    apps.borrow_mut().running = Some(FIELD.to_string());
    assert!(!found.timed());
    // Python's spy on `started`: the background last said with a picture is forgotten.
    found.puzzle.borrow().shown.set(0x1234);
    found.started();
    assert_eq!(found.puzzle.borrow().shown.get(), 0);
}

/// ADR-0013: O-Power's list and Active panel, up on the field, turn rows by themselves, the
/// page showing kept for R, as the PSS's own apps turn theirs; R says O-Power's line, where a
/// battle's sides and a contest's have none, and Shift and R, the other side, nothing of it.
/// Its record is swept for first of the subscreens' classes, as the log has always named it.
#[test]
fn test_o_powers_screens_turn_rows_and_r_says_its_energy_there() {
    let apps = running(FIELD);
    let mut found = subscreens(apps.clone());
    assert_eq!(found.classes[0], o_power::CLASS);
    // Python's fake O-Power: its list up or not, and each `poll` kept, seen here as the words on
    // screen asked once by `up` and once more by `poll`.
    const LIST: i64 = 0x15DF062C;
    const RECORD: i64 = 0x08C7FD3C;
    let up = Rc::new(Cell::new(false));
    let asked = Rc::new(Cell::new(0));
    let memory = FakeMemory::new();
    memory.u32(RECORD + o_power::ENERGY, &[8]);
    let mut fake = OPower::new(
        memory,
        {
            let (up, asked) = (up.clone(), asked.clone());
            Rc::new(move || {
                asked.set(asked.get() + 1);
                let listed = ScreenText {
                    root: LIST,
                    ..ScreenText::new(1, "Hatching Power", false)
                };
                if up.get() { vec![listed] } else { Vec::new() }
            })
        },
        Rc::new(|root: i64| {
            (root == LIST).then(|| {
                Rc::new(LayoutFile {
                    name: o_power::LIST.to_string(),
                    ..LayoutFile::new(Vec::new(), IndexMap::new(), IndexMap::new())
                })
            })
        }),
        Rc::new(|| Ok(vec![LIST])),
        Rc::new(|_root: i64| Ok(Vec::new())),
        false,
    );
    fake.track(
        &IndexMap::from([(RECORD, o_power::CLASS.to_string())]),
        false,
    );
    found.o_power = Rc::new(RefCell::new(fake));
    assert!(!found.turns() && asked.get() == 1);
    up.set(true);
    assert!(found.turns() && asked.get() == 3);
    apps.borrow_mut().running = Some(amie::AMIE.to_string());
    assert!(!found.turns());
    apps.borrow_mut().running = Some(pss_profile());
    assert!(found.turns() && asked.get() == 3);
    assert_eq!(
        found.sides(true).unwrap().as_deref(),
        Some("energy 8 of 10")
    );
    assert_eq!(found.sides(false).unwrap(), None);
}

/// Touch mode that is never touched: Python's `cast(TouchMode, None)`.
struct Untouched;

impl Touching for Untouched {
    fn targets(&mut self) -> Result<Vec<Target>> {
        Ok(Vec::new())
    }

    fn give(&mut self, _picked: &Target, _over: &Target) -> String {
        String::new()
    }

    fn tap(&mut self, _target: &Target) -> String {
        String::new()
    }
}
