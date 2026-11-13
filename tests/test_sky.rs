//! Soaring on Latios: the sky's places out of the ROM and the flocks flying about, said from where
//! the player flies (spike 166).

use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::rc::Rc;

use super::fake_memory::{FakeClasses, FakeMemory};
use super::fake_speech::Recorder;
use crate::core::rtti::ClassNames;
use crate::core::speech::Speech;
use crate::games::common::field::ground::Ground;
use crate::games::common::field::guide::FieldSource;
use crate::games::common::field::keys::{
    CATEGORY_NEXT, EAST as EAST_KEY, ITEM_READ, MODE_TOGGLE, PATHFIND, SOUTH as SOUTH_KEY,
};
use crate::games::common::field::navigate::{Navigator, Options};
use crate::games::common::field::things::{Field, POKEMON};
use crate::games::gen6::apps::LOADED_CODE;
use crate::games::oras::field::sky;
use crate::games::oras::field::sky::{LANDMARKS, MIRAGE, ROUTES, TOWNS};
use crate::py::Result;

const WHITE: i64 = 0xFFFFFFFF;
const ORANGE: i64 = 0xFFFFA500;
const TOMATO: i64 = 0xFFFF6347;

fn names(zone: i64) -> Option<String> {
    match zone {
        0x6 => Some("Littleroot Town"),
        0x7 => Some("Oldale Town"),
        0x17 => Some("Route 101"),
        0x4E => Some("Granite Cave"),
        0xC8 => Some("Mirage Island"),
        _ => None,
    }
    .map(str::to_string)
}

/// Headings: the player flies along (sin, cos) of theirs in the world's (x, z), z south.
const NORTH: f64 = PI;
const EAST: f64 = PI / 2.0;

/// One row of the table, its rectangle (west, north, east, south) in world units.
fn row(zone: i64, colour: i64, rect: (f64, f64, f64, f64), kind: i64) -> Vec<u8> {
    let (west, north, wide, deep) = sky::BOUNDS;
    let (left, top, right, bottom) = rect;
    let mut out: Vec<u8> = Vec::new();
    for word in [
        zone, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xB11, 0xFFFFFFFF, kind, colour, 1,
    ] {
        out.extend((word as u32).to_le_bytes());
    }
    for fraction in [
        (left - west) / wide,
        (top - north) / deep,
        (right - left) / wide,
        (bottom - top) / deep,
    ] {
        out.extend((fraction as f32).to_le_bytes());
    }
    out
}

/// The table as the ROM packs it: one file in a two-letter archive.
fn table(rows: &[Vec<u8>]) -> Vec<u8> {
    let mut slan: Vec<u8> = b"SLAN".to_vec();
    slan.extend([0; 4]);
    slan.extend((rows.len() as u32).to_le_bytes());
    slan.extend([0; 0x1C]);
    for row in rows {
        slan.extend(row);
    }
    let mut out: Vec<u8> = b"TM".to_vec();
    out.extend(1u16.to_le_bytes());
    out.extend(12u32.to_le_bytes());
    out.extend((12 + slan.len() as u32).to_le_bytes());
    out.extend(slan);
    out
}

fn places() -> Vec<sky::Place> {
    sky::places(
        &table(&[
            row(0x6, WHITE, (-4000.0, 8000.0, -2000.0, 10000.0), 0),
            row(0x17, ORANGE, (-4000.0, 6000.0, -2000.0, 8000.0), 0),
            row(0x17, ORANGE, (-2000.0, 6000.0, 0.0, 7000.0), 0),
            row(0xC8, TOMATO, (-9000.0, 0.0, -8000.0, 1000.0), 4),
            row(0x4E, TOMATO, (-9000.0, 9000.0, -8000.0, 10000.0), 0),
            row(0x7, 0x12345678, (0.0, 0.0, 1000.0, 1000.0), 0),
        ]),
        &names,
    )
    .unwrap()
}

/// The rows of every place always there: all but the mirage spot.
fn always() -> HashSet<i64> {
    HashSet::from([0, 1, 2, 4])
}

const PROCESS: i64 = 0x08C65224;

fn place(name: &str) -> sky::Place {
    places().into_iter().find(|p| p.name == name).unwrap()
}

/// `pytest.approx` on a tuple: within a millionth of each expected number.
fn approx(got: &[f64], expected: &[f64]) -> bool {
    got.len() == expected.len()
        && got
            .iter()
            .zip(expected)
            .all(|(got, expected)| (got - expected).abs() <= (1e-6 * expected.abs()).max(1e-12))
}

#[test]
fn test_places_are_grouped_by_the_games_colours_and_merged_by_name() {
    let found: Vec<(String, String, Vec<i64>)> = places()
        .into_iter()
        .map(|p| (p.name, p.category, p.rows))
        .collect();
    let expected: Vec<(String, String, Vec<i64>)> = [
        ("Littleroot Town", TOWNS, vec![0]),
        ("Route 101", ROUTES, vec![1, 2]),
        ("Mirage Island", MIRAGE, vec![3]),
        ("Granite Cave", LANDMARKS, vec![4]),
    ]
    .into_iter()
    .map(|(name, category, rows)| (name.to_string(), category.to_string(), rows))
    .collect();
    assert_eq!(found, expected);
    let (west, north, east, south) = place("Littleroot Town").rects[0];
    assert!(approx(
        &[west, north, east, south],
        &[-4000.0, 8000.0, -2000.0, 10000.0]
    ));
}

#[test]
fn test_a_file_that_is_not_the_table_has_no_places() {
    let mut file: Vec<u8> = b"TM".to_vec();
    file.extend(1u16.to_le_bytes());
    file.extend(12u32.to_le_bytes());
    file.extend(16u32.to_le_bytes());
    file.extend(b"ZONE");
    assert!(sky::places(&file, &names).unwrap().is_empty());
}

#[test]
fn test_bearings_are_from_where_the_game_looks_in_the_players_own_frame() {
    // Flying north from the south edge of Littleroot Town: the game looks 1300 ahead, inside it.
    let known = places();
    let flight = sky::Flight {
        x: -3000.0,
        z: 9900.0,
        heading: NORTH,
        height: 1900.0,
    };
    let r#where = sky::listed(&flight, &known, Some(&always()), &[]).unwrap();
    let bearings = |r#where: &Field| -> HashMap<String, String> {
        [TOWNS, ROUTES, LANDMARKS]
            .iter()
            .flat_map(|category| r#where.of(category))
            .map(|item| {
                let bearing = item.bearing(r#where.cell, r#where.heading).unwrap();
                (item.kind, bearing)
            })
            .collect()
    };
    let expected: HashMap<String, String> = [
        ("Littleroot Town", "here"),
        ("Route 101", "6 ahead"),
        ("Granite Cave", "4 behind 50 left"),
    ]
    .into_iter()
    .map(|(kind, bearing)| (kind.to_string(), bearing.to_string()))
    .collect();
    assert_eq!(bearings(&r#where), expected);
    // Turned east, what was on the left is behind.
    let east = sky::Flight {
        heading: EAST,
        ..flight
    };
    let r#where = sky::listed(&east, &known, Some(&always()), &[]).unwrap();
    assert_eq!(
        r#where.of(LANDMARKS)[0]
            .bearing(r#where.cell, r#where.heading)
            .unwrap(),
        "63 behind"
    );
}

#[test]
fn test_p_says_the_smallest_turn_that_brings_a_place_where_the_game_looks() {
    let north = sky::Flight {
        x: -3000.0,
        z: 9900.0,
        heading: NORTH,
        height: 1900.0,
    };
    assert_eq!(
        sky::turn(&north, &place("Littleroot Town")).unwrap(),
        "here"
    );
    assert_eq!(sky::turn(&north, &place("Route 101")).unwrap(), "6 ahead");
    assert_eq!(
        sky::turn(&north, &place("Granite Cave")).unwrap(),
        "turn left 90 degrees, 37 ahead"
    );
    // Over Littleroot facing east, the point the game looks at is past the town's east edge.
    let over = sky::Flight {
        x: -2500.0,
        z: 9300.0,
        heading: EAST,
        height: 1900.0,
    };
    assert_eq!(
        sky::turn(&over, &place("Littleroot Town")).unwrap(),
        "turn left 70 degrees"
    );
}

#[test]
fn test_only_what_the_game_shows_today_is_listed_mirage_spots_too() {
    let north = sky::Flight {
        x: -3000.0,
        z: 9900.0,
        heading: NORTH,
        height: 1900.0,
    };
    let kinds = |r#where: Field| -> HashSet<String> {
        r#where
            .interactables
            .into_iter()
            .map(|item| item.kind)
            .collect()
    };
    let known = places();
    let expected =
        |names: &[&str]| -> HashSet<String> { names.iter().map(|name| name.to_string()).collect() };
    assert_eq!(
        kinds(sky::listed(&north, &known, Some(&HashSet::from([0, 3])), &[]).unwrap()),
        expected(&["Littleroot Town", "Mirage Island"])
    );
    // Before the game has said, every place always there, and no mirage spot.
    assert_eq!(
        kinds(sky::listed(&north, &known, None, &[]).unwrap()),
        expected(&["Littleroot Town", "Route 101", "Granite Cave"])
    );
}

#[test]
fn test_the_rows_shown_today_are_read_through_the_skys_process() {
    let memory = FakeMemory::new();
    memory.u32(PROCESS + sky::PLACES, &[0x0828D6E4]);
    memory.u32(0x0828D6E4 + sky::SHOWN, &[0x0828D75C]);
    memory.write(0x0828D75C, &[1, 0, 0, 1, 0]);
    assert_eq!(
        sky::shown(&*memory, PROCESS, 5).unwrap(),
        Some(HashSet::from([0, 3]))
    );
}

#[test]
fn test_flocks_are_found_among_the_skys_models_and_chased_from_the_player() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let [ship, flock, beacon] = [
        "N3app7skyTrip6BMShipE",
        sky::FLOCK_CLASS,
        "N3app7skyTrip13BMLegendPointE",
    ]
    .map(|name| classes.add(name));
    memory.u32(PROCESS + sky::MANAGER, &[0x08295D04]);
    memory.u32(0x08295D04 + sky::LAST_MODEL, &[0x082A1EA4]);
    // The models are linked both ways; the manager holds the last built, here in the middle.
    for (model, vtable, after, before) in [
        (0x082A3884, flock, 0x082A1EA4, 0),
        (0x082A1EA4, ship, 0x082A56B4, 0x082A3884),
        (0x082A56B4, beacon, 0, 0x082A1EA4),
    ] {
        memory.u32(model, &[vtable, after, before]);
    }
    memory.write(
        0x082A3884 + sky::WHERE,
        &[-2465.0f32, 1700.0, 4874.0]
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<u8>>(),
    );
    let found = sky::flocks(
        &*memory,
        PROCESS,
        &ClassNames::new(memory.clone(), LOADED_CODE),
    )
    .unwrap();
    assert_eq!(
        found,
        vec![sky::Flock {
            model: 0x082A3884,
            x: -2465.0,
            height: 1700.0,
            z: 4874.0,
            legend: "",
            reach: 0.0
        }]
    );
    let north = sky::Flight {
        x: -3000.0,
        z: 9900.0,
        heading: NORTH,
        height: 1900.0,
    };
    let r#where = sky::listed(&north, &places(), Some(&HashSet::new()), &found).unwrap();
    let birds = r#where.of(POKEMON);
    assert_eq!(birds.len(), 1);
    let bird = &birds[0];
    assert_eq!(
        (
            bird.kind.as_str(),
            bird.bearing(r#where.cell, r#where.heading).unwrap()
        ),
        ("flock", "50 ahead 5 right".to_string())
    );
    let steer = r#where.steer.as_ref().unwrap();
    assert_eq!((steer.0)(bird).unwrap(), "51 ahead, 2 below");
    let east = sky::Flight {
        heading: EAST,
        ..north
    };
    assert_eq!(
        sky::chase(&east, &found[0]).unwrap(),
        "turn left 85 degrees, 51 ahead, 2 below"
    );
}

/// or_soaring_rifts: north of Fortree City with a Castform in the party, the game asked to fly
/// into the storm clouds 2000 from a flock's model that never moved, which carries Tornadus's
/// flag; the list had said it as a flock (spike 295).
#[test]
fn test_a_legends_storm_clouds_are_a_mirage_spot_said_to_where_the_game_asks_to_fly_in() {
    let memory = FakeMemory::new();
    let clouds = FakeClasses::new(memory.clone()).add(sky::FLOCK_CLASS);
    memory.u32(PROCESS + sky::MANAGER, &[0x08295D04]);
    memory.u32(0x08295D04 + sky::LAST_MODEL, &[0x082A3884]);
    memory.u32(0x082A3884, &[clouds, 0, 0]);
    memory.write(
        0x082A3884 + sky::WHERE,
        &[8956.5f32, 2000.0, -8631.25]
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<u8>>(),
    );
    memory.u32(0x082A3884 + sky::LEGEND, &[0xB6C]);
    memory.write(0x082A3884 + sky::REACH_IN, &2000.0f32.to_le_bytes());
    let found = sky::flocks(
        &*memory,
        PROCESS,
        &ClassNames::new(memory.clone(), LOADED_CODE),
    )
    .unwrap();
    assert_eq!(
        found,
        vec![sky::Flock {
            model: 0x082A3884,
            x: 8956.5,
            height: 2000.0,
            z: -8631.25,
            legend: sky::STORM_CLOUDS,
            reach: 2000.0
        }]
    );
    let east = sky::Flight {
        x: 6388.0,
        z: -8324.0,
        heading: EAST,
        height: 1898.0,
    };
    let r#where = sky::listed(&east, &places(), Some(&HashSet::new()), &found).unwrap();
    let spots = r#where.of(MIRAGE);
    let [spot] = spots.as_slice() else {
        panic!("one mirage spot: {spots:?}");
    };
    assert!(r#where.of(POKEMON).is_empty());
    assert_eq!(
        (
            spot.kind.as_str(),
            spot.bearing(r#where.cell, r#where.heading).unwrap()
        ),
        ("storm clouds", "6 ahead 1 left".to_string())
    );
    let steer = r#where.steer.as_ref().unwrap();
    assert_eq!((steer.0)(spot).unwrap(), "6 ahead");
    let nearer = sky::Flight {
        x: 7087.0,
        z: -8290.0,
        ..east
    };
    assert_eq!(sky::chase(&nearer, &found[0]).unwrap(), "here");
}

#[test]
fn test_flocks_stop_where_the_models_links_loop_back() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let flock = classes.add(sky::FLOCK_CLASS);
    memory.u32(PROCESS + sky::MANAGER, &[0x08295D04]);
    memory.u32(0x08295D04 + sky::LAST_MODEL, &[0x082A1EA4]);
    // Freed models whose links were rewritten: each leads back to the other, both ways.
    memory.u32(0x082A1EA4, &[flock, 0x082A3884, 0x082A3884]);
    memory.u32(0x082A3884, &[flock, 0x082A1EA4, 0x082A1EA4]);
    let found = sky::flocks(
        &*memory,
        PROCESS,
        &ClassNames::new(memory.clone(), LOADED_CODE),
    )
    .unwrap();
    assert_eq!(
        found.iter().map(|each| each.model).collect::<Vec<_>>(),
        vec![0x082A1EA4, 0x082A3884]
    );
}

#[test]
fn test_the_heading_is_said_as_a_compass_point() {
    let said: Vec<&str> = [PI, PI / 2.0, 0.0, 3.0 * PI / 2.0, 3.9]
        .iter()
        .map(|heading| sky::facing(*heading).unwrap())
        .collect();
    assert_eq!(said, ["north", "east", "south", "west", "north-west"]);
}

#[test]
fn test_the_flight_is_read_through_the_skys_process_but_not_before_its_camera_is_placed() {
    let memory = FakeMemory::new();
    memory.u32(0x08C65224 + sky::CAMERA, &[0x0823C960]);
    assert_eq!(sky::read(&*memory, 0x08C65224).unwrap(), None);
    memory.write(0x0823C960 + sky::HEADING, &(NORTH as f32).to_le_bytes());
    memory.write(
        0x0823C960 + sky::X,
        &[-3896.3f32, 1900.0, 9522.2]
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<u8>>(),
    );
    let flight = sky::read(&*memory, 0x08C65224).unwrap().unwrap();
    assert!(approx(
        &[flight.x, flight.z, flight.height],
        &[-3896.3, 9522.2, 1900.0]
    ));
}

/// Stands in for the profile's source of what the keys walk, with the player soaring as
/// `flight` says.
struct Sky {
    flight: sky::Flight,
}

impl FieldSource for Sky {
    fn visit(&self) -> Option<(i64, i64)> {
        None
    }

    fn poll(&self) -> Result<Option<Field>> {
        Ok(Some(sky::listed(
            &self.flight,
            &places(),
            Some(&always()),
            &[],
        )?))
    }

    fn interactables(&self, r#where: &Field) -> Result<Field> {
        Ok(r#where.clone())
    }

    fn ground(&self, _where: &Field) -> Result<Ground> {
        panic!("no ground in the sky")
    }
}

#[test]
fn test_the_keys_walk_the_skys_places_and_p_says_the_turn() {
    let backend = Recorder::new();
    let mut nav = Navigator::new(
        Rc::new(Sky {
            flight: sky::Flight {
                x: -3000.0,
                z: 9900.0,
                heading: NORTH,
                height: 1900.0,
            },
        }),
        Rc::new(Speech::plain(backend.clone())),
        Rc::new(|_line: &str| {}),
        Options {
            categories: sky::SKY
                .iter()
                .map(|category| category.to_string())
                .collect(),
            ..Options::default()
        },
    );
    nav.press(CATEGORY_NEXT).unwrap();
    nav.press(CATEGORY_NEXT).unwrap();
    nav.press(ITEM_READ).unwrap();
    nav.press(CATEGORY_NEXT).unwrap();
    nav.press(PATHFIND).unwrap();
    assert_eq!(
        backend.spoken(),
        [
            "towns, 1",
            "routes, 1",
            "Route 101, 1 of 1, 6 ahead",
            "landmarks, 1",
            "Granite Cave, turn left 90 degrees, 37 ahead"
        ]
    );
}

#[test]
fn test_direction_mode_looks_round_the_player_and_says_so_in_their_frame() {
    let backend = Recorder::new();
    let mut nav = Navigator::new(
        Rc::new(Sky {
            flight: sky::Flight {
                x: -3000.0,
                z: 9900.0,
                heading: EAST,
                height: 1900.0,
            },
        }),
        Rc::new(Speech::plain(backend.clone())),
        Rc::new(|_line: &str| {}),
        Options {
            categories: sky::SKY
                .iter()
                .map(|category| category.to_string())
                .collect(),
            ..Options::default()
        },
    );
    nav.press(MODE_TOGGLE).unwrap();
    nav.press(EAST_KEY).unwrap();
    nav.press(SOUTH_KEY).unwrap();
    nav.press(SOUTH_KEY).unwrap();
    assert_eq!(
        backend.spoken()[1..],
        [
            "nothing right",
            "Littleroot Town, 1 of 3, 3 behind",
            "Route 101, 2 of 3, 3 behind 19 left"
        ]
    );
}
