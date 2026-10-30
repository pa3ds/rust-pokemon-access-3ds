//! The PlayNav's screens answer for their readers as one, each reader in its own apps alone.

use std::collections::HashSet;
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
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::screens::amie;
use crate::games::gen6::screens::berry_picker::{self, Touching};
use crate::games::gen6::screens::playnav::PlayNav;
use crate::games::oras;
use crate::py::Result;

#[test]
fn test_the_amie_pages_pokemon_is_named_on_the_field_and_offered_to_stroke_in_amie() {
    // The Amie page names its Pokémon on the field, and Amie itself offers it to stroke: each
    // reader says what it has for the app running and nothing in another.
    let (_, memory) = page(amie::NUMBER);
    let layouts = FakeLayouts::new(
        &memory,
        &FakeClasses::starting(memory.clone(), CODE.0 + 0x8000),
    )
    .layouts
    .clone();
    let playnav = PlayNav::new(
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
        Rc::new(|| Ok(HashSet::new())),
        Rc::new(|| Ok(HashSet::new())),
        Rc::new(|said: &str| said.to_string()),
        Rc::new(|said: &str| said.to_string()),
        false,
    );
    playnav
        .track(&IndexMap::from([(PAGE, amie::CLASS.to_string())]), false)
        .unwrap();
    assert_eq!(
        playnav.name(WALKER, Some(FIELD)).unwrap().as_deref(),
        Some("Sceptile")
    );
    assert_eq!(playnav.name(WALKER, Some(amie::AMIE)).unwrap(), None);
    assert!(playnav.targets(Some(FIELD)).unwrap().is_empty());
    let names: Vec<String> = playnav
        .targets(Some(amie::AMIE))
        .unwrap()
        .into_iter()
        .map(|target| target.name)
        .collect();
    assert_eq!(names, ["stroke Sceptile"]);
    assert!(playnav.poll(Some(FIELD)).unwrap().is_empty());
    // Keys of its own only while the screen that has them runs.
    assert_eq!(playnav.context(Some(berry_picker::APP)), Some(BERRY_PICKER));
    assert_eq!(playnav.context(Some(FIELD)), None);
    assert_eq!(
        playnav
            .press(BERRIES[0], Some(FIELD), &mut Untouched)
            .unwrap(),
        ""
    );
}

/// Touch mode that is never touched: Python's `cast(TouchTargets, None)`.
struct Untouched;

impl Touching for Untouched {
    fn targets(&mut self) -> Result<Vec<Target>> {
        Ok(Vec::new())
    }

    fn give(&mut self, _picked: &Target, _over: &Target) -> String {
        String::new()
    }
}
