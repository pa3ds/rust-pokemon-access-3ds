//! The AreaNav's views say which of its layouts it draws (spikes 181 and 183).

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::screens::townmap::{self, VIEWS};

const NUTS: &str = "N3app7townmap8NutsViewE";
const VIEW: i64 = 0x08401BE4;
const UTIL: i64 = 0x08402324;
const TABLE: i64 = 0x084022E4;
const HANDLE: i64 = 0x082486EC;
const WORK: i64 = 0x0820F000;

/// The berry list's view, drawn while its byte at +0xAC is 1, its one layout's work at the
/// handle its `G2DUtil` lists.
fn berry_list(shown: u8) -> (Rc<FakeMemory>, townmap::Views) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(VIEW, &[classes.add(NUTS)]);
    let drawn_at = VIEWS.iter().find(|(class, _)| *class == NUTS).unwrap().1;
    memory.write(VIEW + drawn_at, &[shown]);
    memory.u32(VIEW + townmap::UTIL, &[UTIL]);
    memory.u32(UTIL, &[classes.add(townmap::UTIL_CLASS)]);
    memory.u32(UTIL + townmap::LAYOUTS, &[TABLE, 1]);
    memory.u32(TABLE, &[HANDLE, 0x101]);
    memory.u32(HANDLE + townmap::WORK, &[WORK]);
    let mut views = townmap::Views::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
    );
    views
        .track(&IndexMap::from([(VIEW, NUTS.to_string())]), false)
        .unwrap();
    (memory, views)
}

#[test]
fn test_a_view_put_away_puts_its_layouts_away() {
    assert_eq!(berry_list(0).1.put_away().unwrap(), HashSet::from([WORK]));
}

#[test]
fn test_a_view_drawn_puts_nothing_away() {
    assert_eq!(berry_list(1).1.put_away().unwrap(), HashSet::new());
}

#[test]
fn test_a_view_whose_memory_is_taken_says_nothing() {
    let (memory, views) = berry_list(0);
    memory.u32(VIEW, &[0]);
    assert_eq!(views.put_away().unwrap(), HashSet::new());
    assert!(!views.drawing().unwrap());
}

/// The map's places are offered only while no view is drawn over it (or_area_nav).
#[test]
fn test_a_view_drawn_is_drawn_over_the_map() {
    assert!(berry_list(1).1.drawing().unwrap());
    assert!(!berry_list(0).1.drawing().unwrap());
}

const PROCESS: i64 = 0x08100000;
const MAIN: i64 = 0x08200000;
const GRAPHIC: i64 = 0x08300000;
const MAP: i64 = 0x08400000;
const NAMES: [(i64, &str); 2] = [(0x40, "Route 111"), (0x1F, "Mauville City")];
const ROOT: i64 = 0x08359350;

/// The app running, which the test changes: Python's one-item list.
type Running = Rc<RefCell<Option<String>>>;

/// The AreaNav's process leading to its `MapView`, the place under its cursor at +0xEC.
fn fly_map(place: i64) -> (Rc<FakeMemory>, Running, townmap::Places) {
    let memory = FakeMemory::new();
    memory.u32(PROCESS + townmap::MAP_VIEW[0], &[MAIN]);
    memory.u32(MAIN + townmap::MAP_VIEW[1], &[GRAPHIC]);
    memory.u32(GRAPHIC + townmap::MAP_VIEW[2], &[MAP]);
    memory.u32(
        MAP,
        &[FakeClasses::new(memory.clone()).add(townmap::MAP_VIEW_CLASS)],
    );
    memory.u32(MAP + townmap::PLACE, &[place]);
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
        &townmap::MAP_VIEW,
    );
    (memory, running, places)
}

/// Spike M: the Fly map's cursor gliding from Route 111 into Mauville City.
#[test]
fn test_the_place_the_cursor_comes_onto_is_said_by_its_name() {
    let (memory, _running, mut places) = fly_map(0x40);
    // The map opening: the screen says where the player is.
    assert_eq!(places.poll(false).unwrap(), None);
    memory.u32(MAP + townmap::PLACE, &[0x1F]);
    assert!(places.turned().unwrap());
    assert_eq!(places.poll(false).unwrap().as_deref(), Some("Mauville City"));
    assert!(!places.turned().unwrap());
    assert_eq!(places.poll(false).unwrap(), None);
}

/// The cursor moved off a place onto none and back says it again; a list over the map says
/// its area only as it changes.
#[test]
fn test_a_place_moved_off_and_back_onto_is_said_again_but_not_over_a_list() {
    let (memory, _running, mut places) = fly_map(0x40);
    places.poll(false).unwrap();
    memory.u32(MAP + townmap::PLACE, &[0x1F]);
    assert_eq!(places.poll(false).unwrap().as_deref(), Some("Mauville City"));
    memory.u32(MAP + townmap::PLACE, &[townmap::NO_PLACE]);
    assert!(!places.turned().unwrap());
    assert_eq!(places.poll(false).unwrap(), None);
    assert_eq!(places.current().as_deref(), Some("Mauville City"));
    memory.u32(MAP + townmap::PLACE, &[0x1F]);
    assert!(places.turned().unwrap());
    assert_eq!(places.poll(false).unwrap().as_deref(), Some("Mauville City"));
    memory.u32(MAP + townmap::PLACE, &[townmap::NO_PLACE]);
    places.poll(true).unwrap();
    memory.u32(MAP + townmap::PLACE, &[0x1F]);
    assert_eq!(places.poll(true).unwrap(), None);
    memory.u32(MAP + townmap::PLACE, &[0x40]);
    assert_eq!(places.poll(true).unwrap().as_deref(), Some("Route 111"));
}

#[test]
fn test_another_app_reads_nothing_and_the_next_map_opens_unsaid() {
    let (memory, running, mut places) = fly_map(0x40);
    places.poll(false).unwrap();
    *running.borrow_mut() = Some("N3app8pokelist4ProcE".to_string());
    memory.u32(MAP + townmap::PLACE, &[0x1F]);
    assert!(!places.turned().unwrap());
    assert_eq!(places.poll(false).unwrap(), None);
    *running.borrow_mut() = Some(townmap::APP.to_string());
    assert_eq!(places.poll(false).unwrap(), None);
}

/// Trainer's Eye moves the map to each trainer's place: said first, as the area switched to,
/// its rows say it no more, "Norman" where it was "Norman Petalburg City".
#[test]
fn test_a_list_s_rows_leave_out_the_area_said() {
    let piece = |r#box: i64, text: &str| ScreenText {
        root: ROOT,
        ..ScreenText::new(r#box, text, true)
    };
    let (name, place, alone) = (
        piece(1, "Norman"),
        piece(2, "Petalburg City"),
        piece(3, "Petalburg City"),
    );
    let cursors = CursorPoll {
        items: vec![
            CursorItem::new(9, vec![name.clone(), place], true),
            CursorItem::new(8, vec![alone.clone()], false),
        ],
        ..CursorPoll::default()
    };
    let items = townmap::unplaced(cursors, "Petalburg City").items;
    let [rows, kept] = &items[..] else {
        panic!("two items: {items:?}");
    };
    assert_eq!(rows.pieces, vec![name]);
    assert_eq!(kept.pieces, vec![alone]);
}

/// As the last `put_away` read it, which the screen poll makes first.
#[test]
fn test_the_berry_list_drawn_is_a_list_and_put_away_is_not() {
    for (shown, listing) in [(1, true), (0, false)] {
        let views = berry_list(shown).1;
        views.put_away().unwrap();
        assert_eq!(views.listing(), listing);
    }
}
