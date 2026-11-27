//! The subscreens answer for their readers as one, each reader in its own apps alone.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses};
use super::test_amie::{PAGE, WALKER, page, species};
use crate::core::config::Tables;
use crate::core::rtti::ClassNames;
use crate::games::common::field::keys::{BERRIES, BERRY_PICKER};
use crate::games::common::touch::{ImageOf, Target};
use crate::games::gen6::apps::{FIELD, RunningApp};
use crate::games::gen6::profile;
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::subscreen::berry_picker::{self, Touching};
use crate::games::gen6::subscreen::{self, Subscreens};
use crate::games::gen6::subscreen::{amie, head_it, super_training};
use crate::games::oras;
use crate::py::Result;

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
        Rc::new(|said: &str| said.to_string()),
        Rc::new(|said: &str| said.to_string()),
        None,
        None,
        Rc::new(|| Ok(None)),
        None,
        Rc::new(|_root: i64| None),
        Rc::new(Vec::new),
        Rc::new(|| None),
        Rc::new(|| {}),
        Rc::new(|| {}),
        Rc::new(|_said: &str| {}),
        Rc::new(|_said: &str| {}),
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
    // ADR-0009: touch mode offers targets in Amie's and Super Training's apps, Shift alone drags
    // in Amie's and holds a charge in Super Training's rounds, and Berry Picker's drag is made on
    // letting go, at full speed until its clock is seen to run.
    for app in [amie::AMIE, amie::GAMES[0], amie::GAMES[1], amie::GAMES[2]]
        .into_iter()
        .chain([super_training::SPARRING])
    {
        assert!(profile::TOUCHED.contains(&app), "{app}");
    }
    assert!(
        subscreen::TOUCHED
            .iter()
            .all(|app| profile::TOUCHED.contains(app))
    );
    let mut drags: Vec<&str> = amie::SCREENS.to_vec();
    drags.push(super_training::SPARRING);
    assert_eq!(subscreen::DRAGS.to_vec(), drags);
    let apps = running(berry_picker::APP);
    let found = subscreens(apps.clone());
    assert!(found.delayed());
    apps.borrow_mut().running = Some(amie::AMIE.to_string());
    assert!(!found.delayed());
    assert_eq!(found.pace(), 1.0);
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
