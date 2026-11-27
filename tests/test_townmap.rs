//! The place under the town map's cursor, ORAS's AreaNav's and X's Town Map's (spike 309).

use std::cell::RefCell;
use std::rc::Rc;

use super::fake_memory::{FakeClasses, FakeMemory};
use crate::games::gen6::screens::townmap;

const PROCESS: i64 = 0x08100000;
const MAIN: i64 = 0x08200000;
const GRAPHIC: i64 = 0x08300000;
const MAP: i64 = 0x08400000;
const NAMES: [(i64, &str); 2] = [(0x40, "Route 111"), (0x1F, "Mauville City")];

/// The app running, which the test changes: Python's one-item list.
type Running = Rc<RefCell<Option<String>>>;

/// The AreaNav's process leading to its `MapView`, the place under its cursor at +0xEC.
fn fly_map(place: i64) -> (Rc<FakeMemory>, Running, townmap::Places) {
    let memory = FakeMemory::new();
    memory.u32(PROCESS + townmap::ORAS.view[0], &[MAIN]);
    memory.u32(MAIN + townmap::ORAS.view[1], &[GRAPHIC]);
    memory.u32(GRAPHIC + townmap::ORAS.view[2], &[MAP]);
    memory.u32(
        MAP,
        &[FakeClasses::new(memory.clone()).add(townmap::MAP_VIEW_CLASS)],
    );
    memory.u32(MAP + townmap::ORAS.place, &[place]);
    let running: Running = Rc::new(RefCell::new(Some(townmap::APP.to_string())));
    let places = townmap::Places::new(
        memory.clone(),
        {
            let running = running.clone();
            Rc::new(move || running.borrow().clone())
        },
        Rc::new(|| Some(PROCESS)),
        Rc::new(|index: i64| {
            NAMES
                .iter()
                .find(|(each, _)| *each == index)
                .map(|(_, name)| name.to_string())
        }),
        townmap::ORAS,
    );
    (memory, running, places)
}

/// Spike M: the Fly map's cursor gliding from Route 111 into Mauville City.
#[test]
fn test_the_place_the_cursor_comes_onto_is_said_by_its_name() {
    let (memory, _running, mut places) = fly_map(0x40);
    // The map opening on where the player is.
    assert_eq!(places.poll(false).unwrap().as_deref(), Some("Route 111"));
    memory.u32(MAP + townmap::ORAS.place, &[0x1F]);
    assert!(places.turned().unwrap());
    assert_eq!(
        places.poll(false).unwrap().as_deref(),
        Some("Mauville City")
    );
    assert!(!places.turned().unwrap());
    assert_eq!(places.poll(false).unwrap(), None);
}

/// The cursor moved off a place onto none and back says it again; a list over the map says
/// its area only as it changes.
#[test]
fn test_a_place_moved_off_and_back_onto_is_said_again_but_not_over_a_list() {
    let (memory, _running, mut places) = fly_map(0x40);
    places.poll(false).unwrap();
    memory.u32(MAP + townmap::ORAS.place, &[0x1F]);
    assert_eq!(
        places.poll(false).unwrap().as_deref(),
        Some("Mauville City")
    );
    memory.u32(MAP + townmap::ORAS.place, &[townmap::NO_PLACE]);
    assert!(!places.turned().unwrap());
    assert_eq!(places.poll(false).unwrap(), None);
    assert_eq!(places.current().as_deref(), Some("Mauville City"));
    memory.u32(MAP + townmap::ORAS.place, &[0x1F]);
    assert!(places.turned().unwrap());
    // Off it a poll alone: a flicker on its edge.
    assert_eq!(places.poll(false).unwrap(), None);
    memory.u32(MAP + townmap::ORAS.place, &[townmap::NO_PLACE]);
    places.poll(false).unwrap();
    places.poll(false).unwrap();
    memory.u32(MAP + townmap::ORAS.place, &[0x1F]);
    assert_eq!(
        places.poll(false).unwrap().as_deref(),
        Some("Mauville City")
    );
    memory.u32(MAP + townmap::ORAS.place, &[townmap::NO_PLACE]);
    places.poll(true).unwrap();
    memory.u32(MAP + townmap::ORAS.place, &[0x1F]);
    assert_eq!(places.poll(true).unwrap(), None);
    memory.u32(MAP + townmap::ORAS.place, &[0x40]);
    assert_eq!(places.poll(true).unwrap().as_deref(), Some("Route 111"));
}

#[test]
fn test_another_app_reads_nothing_and_the_next_map_says_its_place() {
    let (memory, running, mut places) = fly_map(0x40);
    places.poll(false).unwrap();
    *running.borrow_mut() = Some("N3app8pokelist4ProcE".to_string());
    memory.u32(MAP + townmap::ORAS.place, &[0x1F]);
    assert!(!places.turned().unwrap());
    assert_eq!(places.poll(false).unwrap(), None);
    *running.borrow_mut() = Some(townmap::APP.to_string());
    assert_eq!(
        places.poll(false).unwrap().as_deref(),
        Some("Mauville City")
    );
}
