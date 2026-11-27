//! ORAS's AreaNav's views say which of its layouts it draws (spikes 181 and 183), and its lists
//! over the map are read row by row.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_own::kit;
use crate::core::discovery::Found;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::own::Own;
use crate::games::gen6::screens::sources::OwnSources;
use crate::games::gen6::screens::townmap;
use crate::games::oras::own::Oras;
use crate::games::oras::subscreen::areanav::{self, VIEWS};

const NUTS: &str = "N3app7townmap8NutsViewE";
const VIEW: i64 = 0x08401BE4;
const UTIL: i64 = 0x08402324;
const TABLE: i64 = 0x084022E4;
const HANDLE: i64 = 0x082486EC;
const WORK: i64 = 0x0820F000;
const CURSOR: i64 = 0x08402CAC;
const CURSOR_ROOT: i64 = 0x08402DC4;
const CURSOR_WORK: i64 = 0x08248E08;

/// The berry list's view, drawn while its byte at +0xAC is 1, its one layout's work at the
/// handle its `G2DUtil` lists.
fn berry_list(shown: u8) -> (Rc<FakeMemory>, areanav::Views) {
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
    let mut views = areanav::Views::new(
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
    assert_eq!(
        berry_list(0).1.put_away(&IndexMap::new()).unwrap(),
        HashSet::from([WORK])
    );
}

/// Its cursor's layout is none of its G2DUtil's: left drawn, Berry Trees' cursor read
/// Trainer's Eye's rows a move late (or_areanav_trainers_bug).
#[test]
fn test_a_list_put_away_puts_its_cursor_away() {
    let (memory, views) = berry_list(0);
    memory.u32(VIEW + areanav::CURSOR, &[CURSOR]);
    memory.u32(CURSOR + areanav::CURSOR_ROOT, &[CURSOR_ROOT]);
    let works_of = IndexMap::from([(CURSOR_ROOT, CURSOR_WORK)]);
    assert_eq!(
        views.put_away(&works_of).unwrap(),
        HashSet::from([WORK, CURSOR_WORK])
    );
    let drawn_at = VIEWS.iter().find(|(class, _)| *class == NUTS).unwrap().1;
    memory.write(VIEW + drawn_at, &[1]);
    assert_eq!(views.put_away(&works_of).unwrap(), HashSet::new());
}

#[test]
fn test_a_view_drawn_puts_nothing_away() {
    assert_eq!(
        berry_list(1).1.put_away(&IndexMap::new()).unwrap(),
        HashSet::new()
    );
}

#[test]
fn test_a_view_whose_memory_is_taken_says_nothing() {
    let (memory, views) = berry_list(0);
    memory.u32(VIEW, &[0]);
    assert_eq!(views.put_away(&IndexMap::new()).unwrap(), HashSet::new());
    assert!(!views.drawing().unwrap());
}

/// The map's places are offered only while no view is drawn over it (or_area_nav).
#[test]
fn test_a_view_drawn_is_drawn_over_the_map() {
    assert!(berry_list(1).1.drawing().unwrap());
    assert!(!berry_list(0).1.drawing().unwrap());
}

const ROOT: i64 = 0x08359350;

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
    let items = areanav::unplaced(cursors, "Petalburg City").items;
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
        views.put_away(&IndexMap::new()).unwrap();
        assert_eq!(views.listing(), listing);
    }
}

fn poll_of(items: Vec<CursorItem>, under: Vec<Vec<ScreenText>>) -> CursorPoll {
    CursorPoll {
        items,
        under,
        ..CursorPoll::default()
    }
}

/// `ScreenText(box, text, changed, root=ROOT)`.
fn at_root(r#box: i64, text: &str, changed: bool) -> ScreenText {
    ScreenText {
        root: ROOT,
        ..ScreenText::new(r#box, text, changed)
    }
}

/// Route 104's three Lum Berry rows scroll under a cursor standing still: the boxes under it
/// change, the words do not; the time since planting is said with each (or_areanav_berries).
#[test]
fn test_a_berry_is_said_with_its_time_once_its_row_holds_still() {
    let first = vec![
        at_root(1, "Lum Berry", false),
        at_root(2, "Route 104", false),
    ];
    let second = vec![
        at_root(5, "Lum Berry", false),
        at_root(6, "Route 104", false),
    ];
    let time = ScreenText {
        root: 0x900,
        top: true,
        ..ScreenText::new(9, "It has been 25 days", false)
    };
    let with_time = |row: &[ScreenText]| [row.to_vec(), vec![time.clone()]].concat();
    let mut rows = areanav::ListRows::new();
    let berries = Some(areanav::BERRIES);
    let held = rows.poll(
        poll_of(
            vec![CursorItem::new(7, first.clone(), true)],
            vec![first.clone()],
        ),
        berries,
        Some("Route 104"),
        &with_time(&first),
    );
    assert!(held.items.is_empty() && held.chased);
    let items = rows
        .poll(
            poll_of(vec![], vec![first.clone()]),
            berries,
            Some("Route 104"),
            &with_time(&first),
        )
        .items;
    let [opened] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(
        (
            opened.cursor,
            opened.pieces.clone(),
            opened.opened,
            opened.rows
        ),
        (7, with_time(&first), true, ROOT)
    );
    assert!(
        rows.poll(
            poll_of(vec![], vec![first.clone()]),
            berries,
            Some("Route 104"),
            &with_time(&first)
        )
        .items
        .is_empty()
    );
    assert!(
        rows.poll(
            poll_of(vec![], vec![second.clone()]),
            berries,
            Some("Route 104"),
            &with_time(&second)
        )
        .chased
    );
    let items = rows
        .poll(
            poll_of(vec![], vec![second.clone()]),
            berries,
            Some("Route 104"),
            &with_time(&second),
        )
        .items;
    let [moved] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(
        (moved.pieces.clone(), moved.opened),
        (with_time(&second), false)
    );
    // The next area's rows are written into the same boxes.
    let other = vec![
        at_root(5, "Razz Berry", true),
        at_root(6, "Route 110", true),
    ];
    rows.poll(
        poll_of(
            vec![CursorItem::new(7, other.clone(), false)],
            vec![other.clone()],
        ),
        berries,
        Some("Route 110"),
        &with_time(&other),
    );
    let items = rows
        .poll(
            poll_of(vec![], vec![other.clone()]),
            berries,
            Some("Route 110"),
            &with_time(&other),
        )
        .items;
    let [turned] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(
        (turned.pieces.clone(), turned.opened),
        (with_time(&other), true)
    );
    assert_eq!(
        areanav::berry_times(&with_time(&second), berries),
        HashSet::from([9])
    );
    assert_eq!(
        areanav::berry_times(&with_time(&second), Some(areanav::TRAINERS)),
        HashSet::new()
    );
}

#[test]
fn test_trainer_s_eye_s_moves_are_said_at_once_and_a_row_reading_alike_once_still() {
    let first = vec![
        at_root(1, "Madeline", false),
        at_root(2, "Route 113", false),
    ];
    let second = vec![
        at_root(5, "Madeline", false),
        at_root(6, "Route 113", false),
    ];
    let mut rows = areanav::ListRows::new();
    let trainers = Some(areanav::TRAINERS);
    let items = rows
        .poll(
            poll_of(
                vec![CursorItem {
                    rows: 3,
                    ..CursorItem::new(7, first.clone(), false)
                }],
                vec![first.clone()],
            ),
            trainers,
            Some("Route 113"),
            &first,
        )
        .items;
    let [said] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!((said.pieces.clone(), said.rows), (first.clone(), ROOT));
    assert!(
        rows.poll(
            poll_of(vec![], vec![second.clone()]),
            trainers,
            Some("Route 113"),
            &second
        )
        .items
        .is_empty()
    );
    let items = rows
        .poll(
            poll_of(vec![], vec![second.clone()]),
            trainers,
            Some("Route 113"),
            &second,
        )
        .items;
    let [moved] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(moved.pieces, second);
    assert!(
        rows.poll(
            poll_of(vec![], vec![second.clone()]),
            None,
            Some("Route 113"),
            &second
        )
        .items
        .is_empty()
    );
}

#[test]
fn test_trainer_s_eye_s_place_block_is_the_layout_naming_the_area() {
    let top = |r#box: i64, text: &str, root: i64| ScreenText {
        root,
        top: true,
        ..ScreenText::new(r#box, text, false)
    };
    let name = top(1, "Route 104", ROOT);
    let told = top(2, "This path, rich with water", ROOT);
    let card = top(3, "Haley", ROOT + 0x100);
    let row = ScreenText {
        root: ROOT + 0x200,
        ..ScreenText::new(4, "Route 104", false)
    };
    assert_eq!(
        areanav::place_block(&[name, told, card, row], "Route 104"),
        HashSet::from([1, 2])
    );
}

/// A game whose map's place is not measured says no place, and Berry Trees' times are still
/// said with their berries alone.
#[test]
fn test_the_areanav_unmeasured_still_leaves_berry_times_to_their_berries() {
    let (memory, _) = berry_list(1);
    let own = Rc::new(Oras::new(&kit(memory, None, None)));
    own.track(&Found {
        heap: IndexMap::from([(VIEW, NUTS.to_string())]),
        ..Found::default()
    })
    .unwrap();
    own.put_away(&IndexMap::new()).unwrap();
    let mut nav = townmap::AreaNav::new(own.clone(), None);
    let time = ScreenText {
        top: true,
        root: 0x1,
        ..ScreenText::new(9, "12 hours", true)
    };
    assert!(!nav.turned().unwrap());
    assert_eq!(nav.title().unwrap(), None);
    assert_eq!(
        own.unsaid(nav.area().as_deref(), &[time]),
        HashSet::from([9])
    );
}
