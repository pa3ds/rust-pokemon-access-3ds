//! Parfum Palace's garden: the Furfrou cornered in its maze (spike 292).

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use crate::core::discovery::Found;
use crate::core::pathfind::Cell;
use crate::games::gen6::field::works::{EVENT_CLASS, FIRST_WORK, WORKS};
use crate::games::gen6::rom::archives::GameText;
use crate::games::xy::garden::{
    CHASING, EAST, FOLLOWING, FURFROU, FURFROU_AT, FurfrouChase, GARDEN, INSIDE, LEFT, LOCKED,
    NORTH, OUTSIDE, SHAUNA, SHAUNA_AT, SOUTH, SPOTS, STORY, WEST, spot, step,
};

const EVENT: i64 = 0x08100000;

/// Where the zone's script sends the Furfrou as the player comes at it from `side`, or None
/// as it is caught: its functions 2086 to 2933, a table a spot (spike 292).
fn runs(furfrou: i64, side: &str, shauna: i64) -> Option<i64> {
    let south = shauna == SOUTH || shauna == LOCKED;
    if furfrou == NORTH {
        let (blocked, free) = if side == "west" {
            (EAST, EAST)
        } else {
            (WEST, WEST)
        };
        return if shauna == blocked { None } else { Some(free) };
    }
    if furfrou == SOUTH {
        let away = if side == "east" { WEST } else { EAST };
        return Some(if shauna == away {
            if away == WEST { EAST } else { WEST }
        } else {
            away
        });
    }
    if furfrou == WEST {
        return Some(if south {
            if side == "north" { EAST } else { NORTH }
        } else {
            SOUTH
        });
    }
    if side == "north" {
        return Some(if south { WEST } else { SOUTH });
    }
    if side == "south" {
        return Some(if south {
            NORTH
        } else if shauna == WEST {
            SOUTH
        } else {
            WEST
        });
    }
    Some(if south { NORTH } else { SOUTH })
}

fn side_of(cell: Cell, furfrou: i64) -> &'static str {
    let ((x, z), _kind) = spot(furfrou).unwrap();
    match (cell.0 - x, cell.1 - z) {
        (-3, 0) => "west",
        (3, 0) => "east",
        (0, -3) => "north",
        (0, 3) => "south",
        other => panic!("KeyError: {other:?}"),
    }
}

#[test]
fn test_the_steps_corner_the_furfrou_from_wherever_the_two_stand() {
    for &(furfrou, _) in &SPOTS {
        for shauna in [FOLLOWING, NORTH, SOUTH, EAST, WEST] {
            if shauna == furfrou {
                continue;
            }
            let (mut at, mut waits) = (furfrou, shauna);
            let mut caught = false;
            for _ in 0..8 {
                let found = step(CHASING, waits, at).unwrap();
                assert!(found.is_some());
                let (message, cell) = found.unwrap();
                if message == "furfrou-fetch" {
                    assert_eq!(cell, spot(waits).unwrap().0);
                    waits = FOLLOWING;
                } else if message == "furfrou-wait" {
                    waits = SPOTS
                        .iter()
                        .find(|(_, (where_, _kind))| *where_ == cell)
                        .unwrap()
                        .0;
                    assert_ne!(waits, at);
                } else {
                    assert_eq!(message, "furfrou-chase");
                    let Some(ran) = runs(at, side_of(cell, at), waits) else {
                        caught = true;
                        break;
                    };
                    at = ran;
                }
            }
            assert!(caught, "never caught from {furfrou}, {shauna}");
        }
    }
}

#[test]
fn test_shauna_locked_at_the_south_spot_is_left_and_come_back_to() {
    assert_eq!(
        step(CHASING, LOCKED, EAST).unwrap(),
        Some(("furfrou-leave", OUTSIDE))
    );
    assert_eq!(
        step(LEFT, SOUTH, 0).unwrap(),
        Some(("furfrou-enter", INSIDE))
    );
    assert_eq!(step(1, SOUTH, 0).unwrap(), None);
}

/// The Python's `Text`, which names every sprite Shauna and every species Furfrou.
fn text() -> GameText {
    let mut species = vec![String::new(); FURFROU as usize + 1];
    species[FURFROU as usize] = "Furfrou".to_string();
    GameText {
        sprites: IndexMap::from([(SHAUNA, "Shauna".to_string())]),
        species,
        ..GameText::default()
    }
}

#[test]
fn test_the_spots_and_the_step_are_listed_while_the_chase_is_on() {
    let memory = FakeMemory::new();
    let chase = FurfrouChase::new(memory.clone(), &text());

    let works = |story: i64, shauna: i64, furfrou: i64| -> Vec<(String, Cell)> {
        for (work, value) in [(STORY, story), (SHAUNA_AT, shauna), (FURFROU_AT, furfrou)] {
            memory.write(
                EVENT + WORKS + (work - FIRST_WORK) * 2,
                &(value as u16).to_le_bytes(),
            );
        }
        chase
            .listed(GARDEN, &|_cell| Ok(None))
            .unwrap()
            .into_iter()
            .map(|item| (item.kind, item.cell))
            .collect()
    };
    let pairs = |items: &[(&str, Cell)]| -> Vec<(String, Cell)> {
        items
            .iter()
            .map(|&(kind, cell)| (kind.to_string(), cell))
            .collect()
    };

    assert_eq!(works(CHASING, FOLLOWING, EAST), pairs(&[]));
    chase
        .track(&Found {
            pages: IndexMap::new(),
            heap: IndexMap::from([(EVENT, EVENT_CLASS.to_string())]),
            partial: false,
        })
        .unwrap();
    assert_eq!(
        works(CHASING, FOLLOWING, EAST),
        pairs(&[
            ("north waiting spot", (39, 30)),
            ("south waiting spot", (39, 51)),
            ("east waiting spot", (48, 42)),
            ("west waiting spot", (30, 42)),
            ("next step, Shauna waits here", (39, 51)),
        ])
    );
    // The line north of the Furfrou sends it west, not to the dead end: a way goes round it.
    let listed = works(CHASING, SOUTH, EAST);
    assert_eq!(
        listed[listed.len() - 2..],
        pairs(&[
            ("next step, walk here and Furfrou runs", (45, 42)),
            ("keep off, Furfrou gets away from here", (47, 39)),
        ])
    );
    assert_eq!(works(1, SOUTH, 0), pairs(&[]));
    assert_eq!(
        chase.listed(GARDEN + 1, &|_cell| Ok(None)).unwrap(),
        Vec::new()
    );
}
