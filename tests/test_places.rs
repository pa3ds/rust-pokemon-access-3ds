//! The game's names for places: a place said on arriving, a part of a place in parts, where an
//! exit leads, and a slot on a map the ROM names (`gen6.field.places`).

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_rom::TmpDir;
use super::fake_words::words_in;
use super::test_field::{Given, HOUSE, ROUTE, TABLE, TOWN, Town, dexnav, field_reader, game};
use crate::games::common::field::things::{Interactable, MODEL_SOURCE, WARP_SOURCE, place_source};
use crate::games::gen6::field::places::PlaceNames;
use crate::games::gen6::field::reader::FieldReader;
use crate::games::gen6::field::zonetable::{INFO_NAME, INFO_SIZE};
use crate::games::gen6::game::Game;
use crate::games::gen6::rom::archives::{GameText, no_text};
use crate::games::oras;
use crate::py::Result;

/// `PLACES`: the place names as the ROM lists them, Littleroot Town at 170 and Route 101 at 204.
fn places_listed() -> Vec<String> {
    let mut places = vec!["——————".to_string()];
    places.extend(std::iter::repeat_n(String::new(), 169));
    places.push("Littleroot Town".to_string());
    places.extend(std::iter::repeat_n(String::new(), 33));
    places.push("Route 101".to_string());
    places
}

/// `TRAINERS`.
fn trainers() -> Vec<String> {
    ["-", "Brendan", "Brendan", "Brendan", "May", "May", "May"]
        .iter()
        .map(|name| name.to_string())
        .collect()
}

/// `GameText(PLACES, TRAINERS)`.
fn text() -> GameText {
    GameText::new(places_listed(), trainers())
}

/// The town's place names, its zone table found by a sweep: `places(town, text)`. Python's
/// `lambda: town.reader` reads whichever reader the town holds when asked; here the reader is
/// shared with the test, which sets it where the Python's sets `town.reader`.
fn places_of(town: &Town, text: GameText) -> (PlaceNames, Rc<RefCell<Rc<FieldReader>>>) {
    town.reader.track(&town.found()).unwrap();
    let reader = Rc::new(RefCell::new(town.reader.clone()));
    let asked = reader.clone();
    let named = PlaceNames::new(
        Rc::new(text),
        oras::game().items_at.clone(),
        Rc::new(move || asked.borrow().clone()),
    );
    (named, reader)
}

/// `places(town)`.
fn places(town: &Town) -> PlaceNames {
    places_of(town, text()).0
}

/// The town's two zones given parts of their own, in `zones.ftl`.
fn parted(tmp_path: &TmpDir) {
    words_in(
        tmp_path.path(),
        &format!(
            "part-{TOWN:#x} = Northern {{ $place }}\npart-{ROUTE:#x} = Southern {{ $place }}\n"
        ),
        "oras",
        "en",
        "zones.ftl",
    );
}

fn named(place: &str, shown: Option<&str>) -> Option<(String, Option<String>)> {
    Some((place.to_string(), shown.map(str::to_string)))
}

/// The game shows a place's name when the place changes, not the zone: walking out of a house
/// into its own town arrives nowhere (spike 87).
#[test]
fn test_the_place_is_said_on_arriving_and_not_on_walking_into_a_building() -> Result<()> {
    let places = places(&Town::new());
    assert_eq!(places.arrived(TOWN)?, named("Littleroot Town", None));
    assert_eq!(places.arrived(HOUSE)?, None);
    assert_eq!(places.arrived(TOWN)?, None);
    assert_eq!(places.arrived(ROUTE)?, named("Route 101", None));
    Ok(())
}

/// ADR-0042: a part of a place in parts is said as the player walks into it, from another place
/// or the next part, with the place the game's sign says, and not on walking out of a house back
/// into it; and an exit into a part says the part, the way out of a house of the place too ("way
/// out to South Side" said nothing of where). Here the town's two zones are given parts of their
/// own.
#[test]
fn test_a_part_of_a_place_is_said_walking_into_it_and_by_the_exits_into_it() -> Result<()> {
    let tmp_path = TmpDir::new();
    parted(&tmp_path);
    let places = places(&Town::new());
    assert_eq!(
        places.arrived(TOWN)?,
        named("Northern Littleroot Town", Some("Littleroot Town"))
    );
    assert_eq!(places.arrived(HOUSE)?, None);
    assert_eq!(
        places.leads_to(HOUSE, TOWN)?.as_deref(),
        Some("Northern Littleroot Town")
    );
    assert_eq!(places.arrived(TOWN)?, None);
    assert_eq!(
        places.leads_to(TOWN, ROUTE)?.as_deref(),
        Some("Southern Route 101")
    );
    assert_eq!(
        places.arrived(ROUTE)?,
        named("Southern Route 101", Some("Route 101"))
    );
    assert_eq!(places.place(ROUTE)?.as_deref(), Some("Southern Route 101"));
    assert_eq!(places.place(HOUSE)?.as_deref(), Some("Littleroot Town"));
    Ok(())
}

/// ADR-0042: a thing standing in another part of the player's place says the part; one in the
/// player's own part, an exit into that part, and a place of no parts say nothing more.
#[test]
fn test_a_thing_of_another_part_is_said_with_its_part() -> Result<()> {
    let tmp_path = TmpDir::new();
    parted(&tmp_path);
    let mut town = Town::new();
    let both = vec![TOWN, ROUTE];
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            parts: IndexMap::from([(TOWN, both.clone()), (ROUTE, both)]),
            ..Given::default()
        },
        Game {
            features: vec![dexnav],
            ..game()
        },
    );
    town.memory.u16(TABLE + ROUTE * INFO_SIZE + INFO_NAME, 170);
    let (places, reader) = places_of(&town, text());
    town.reader.poll()?;
    let ball = Interactable::new("items", "item ball", (95, 140));
    assert_eq!(
        places.part(&ball, TOWN)?.as_deref(),
        Some("Southern Littleroot Town")
    );
    let here = Interactable {
        cell: (95, 170),
        ..ball.clone()
    };
    assert_eq!(places.part(&here, TOWN)?, None);
    let edge = Interactable {
        zone: Some(ROUTE),
        ..Interactable::new("exits", "edge", (99, 159))
    };
    assert_eq!(places.part(&edge, TOWN)?, None);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            features: vec![dexnav],
            ..game()
        },
    );
    *reader.borrow_mut() = town.reader.clone();
    town.reader.track(&town.found())?;
    assert_eq!(places.part(&ball, TOWN)?, None);
    Ok(())
}

/// AreaNav's words still say where the player is, a moment later, as they always have.
#[test]
fn test_without_the_rom_no_place_is_said_and_an_exit_is_only_what_it_is() -> Result<()> {
    let places = places_of(&Town::new(), no_text()).0;
    assert_eq!(places.arrived(TOWN)?, None);
    let mut led = Vec::new();
    for there in [HOUSE, 0xE1, ROUTE] {
        led.push(places.leads_to(TOWN, there)?);
    }
    assert_eq!(led, [None, None, None]);
    Ok(())
}

/// Nothing said the gap north of Littleroot leads to Route 101. The place is the zone table's
/// index for each side and the ROM's word for it; the house behind the first door shares the
/// town's place, and the second door's zone has no place at all.
#[test]
fn test_an_exit_into_another_place_is_said_with_it_and_a_door_of_the_town_is_not() -> Result<()> {
    let places = places(&Town::new());
    let mut led = Vec::new();
    for there in [HOUSE, 0xE1, ROUTE] {
        led.push(places.leads_to(TOWN, there)?);
    }
    assert_eq!(led, [None, None, Some("Route 101".to_string())]);
    Ok(())
}

/// The ROM names the building behind warp 0; sign 0 on the same map is not that building (the
/// wally.ctm replay of 2026-09-23 read Petalburg's signs as its Gym, Mart and Center).
#[test]
#[allow(clippy::identity_op)]
fn test_the_roms_door_name_is_a_doors_and_never_a_signs() {
    let text = GameText {
        doors: IndexMap::from([((TOWN, 0), "Pokémon Center".to_string())]),
        ..text()
    };
    let places = places_of(&Town::new(), text).0;
    assert_eq!(
        places.slot(TOWN, WARP_SOURCE | 0).as_deref(),
        Some("Pokémon Center")
    );
    assert_eq!(places.slot(TOWN, 0), None);
}

/// Lumiose's five gates were each "warp to Lumiose Gate".
#[test]
#[allow(clippy::identity_op)]
fn test_a_warp_into_a_gate_is_called_the_gate_and_the_place_beyond_it() {
    let text = GameText {
        gates: IndexMap::from([((TOWN, 0), (170, 204))]),
        ..text()
    };
    let places = places_of(&Town::new(), text).0;
    assert_eq!(
        places.slot(TOWN, WARP_SOURCE | 0).as_deref(),
        Some("Littleroot Town to Route 101")
    );
}

/// One model draws both of Route 111's fossils, and the game names them only as the item taken:
/// west the Root Fossil, east the Claw Fossil, listed under items (a player's report, spike
/// 233). Their names are the ROM's, in the player's language.
#[test]
fn test_route_111s_fossils_are_called_the_items_they_are_from_the_rom() -> Result<()> {
    let mut items = vec![String::new(); 101];
    items[99] = "Wurzelfossil".to_string();
    items[100] = "Klauenfossil".to_string();
    let places = places_of(&Town::new(), GameText { items, ..text() }).0;
    let mut called = Vec::new();
    for record in [3, 4, 5] {
        called.push(places.slot(0x23, place_source(MODEL_SOURCE, record)?));
    }
    assert_eq!(
        called,
        [
            Some("Wurzelfossil".to_string()),
            Some("Klauenfossil".to_string()),
            None
        ]
    );
    assert!(oras::game().pickups.contains(&oras::FOSSIL));
    Ok(())
}
