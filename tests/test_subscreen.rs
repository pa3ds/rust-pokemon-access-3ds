//! The subscreens answer for their readers as one, each reader in its own apps alone.

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses};
use super::test_amie::{PAGE, WALKER, page, species};
use crate::core::config::Amie;
use crate::core::rtti::ClassNames;
use crate::games::common::field::keys::{BERRIES, BERRY_PICKER};
use crate::games::common::touch::Target;
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::profile;
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::subscreen::berry_picker::{self, Touching};
use crate::games::gen6::subscreen::{self, Subscreens};
use crate::games::gen6::subscreen::{amie, super_training};
use crate::games::oras;
use crate::py::Result;

/// The subscreens over the Amie page of `test_amie`, its Pokémon Sceptile.
fn subscreens() -> Subscreens {
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
        Rc::new(|_| Ok(None)),
        IndexMap::new(),
        Rc::new(GameText {
            species: species(),
            ..GameText::new(Vec::new(), Vec::new())
        }),
        oras::game(),
        &Amie::default(),
        Rc::new(|said: &str| said.to_string()),
        Rc::new(|said: &str| said.to_string()),
        false,
    );
    found
        .track(&IndexMap::from([(PAGE, amie::CLASS.to_string())]), false)
        .unwrap();
    found
}

#[test]
fn test_the_amie_pages_pokemon_is_named_on_the_field_and_offered_to_stroke_in_amie() {
    // The Amie page names its Pokémon on the field, and Amie itself offers it to stroke: each
    // reader says what it has for the app running and nothing in another.
    let found = subscreens();
    assert_eq!(
        found.name(WALKER, Some(FIELD)).unwrap().as_deref(),
        Some("Sceptile")
    );
    assert_eq!(found.name(WALKER, Some(amie::AMIE)).unwrap(), None);
    assert!(found.targets(Some(FIELD)).unwrap().is_empty());
    let names: Vec<String> = found
        .targets(Some(amie::AMIE))
        .unwrap()
        .into_iter()
        .map(|target| target.name)
        .collect();
    assert_eq!(names, ["stroke Sceptile"]);
    assert!(found.poll(Some(FIELD)).unwrap().is_empty());
    // Keys of its own only while the screen that has them runs.
    assert_eq!(found.context(Some(berry_picker::APP)), Some(BERRY_PICKER));
    assert_eq!(found.context(Some(FIELD)), None);
    assert_eq!(
        found
            .press(BERRIES[0], Some(FIELD), &mut Untouched)
            .unwrap(),
        ""
    );
}

#[test]
fn test_touch_mode_in_the_subscreens_apps_is_as_they_say() {
    // ADR-0009: touch mode offers targets in Amie's and Super Training's apps, Shift alone drags
    // only in Amie's, and Berry Picker's drag is made on letting go, at full speed until its clock
    // is seen to run.
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
    assert_eq!(subscreen::DRAGS, amie::SCREENS);
    assert!(!subscreen::DRAGS.contains(&super_training::SPARRING));
    let found = subscreens();
    assert!(found.delayed(Some(berry_picker::APP)) && !found.delayed(Some(amie::AMIE)));
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
}
