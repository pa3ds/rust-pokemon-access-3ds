//! Pathfinding tests, over maps drawn a row a line: `#` cannot be walked, `v` is a ledge
//! jumped down southwards, `>` a slope walked down eastwards, and anything else can be walked.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::pathfind::{
    Box, Cell, EAST, Leg, NORTH, Options, Reach, SOUTH, WEST, cover, facing, find, legs, nearest,
    off, reached, spoken, towards, turned,
};
use crate::py::Result;

type Goals = IndexMap<Cell, Option<&'static str>>;

fn goals(cells: &[(Cell, Option<&'static str>)]) -> Goals {
    cells.iter().copied().collect()
}

fn grid(rows: &[&str]) -> Rc<Vec<Vec<u8>>> {
    Rc::new(rows.iter().map(|row| row.bytes().collect()).collect())
}

/// A map from (0, 0) at the top left, and the box it fills.
fn drawn(rows: &[&str]) -> (Rc<dyn Fn(Cell) -> Result<bool>>, Box) {
    let cells = grid(rows);
    let walkable = move |cell: Cell| Ok(!b"#v".contains(&cells[cell.1 as usize][cell.0 as usize]));
    (
        Rc::new(walkable),
        Box {
            west: 0,
            north: 0,
            east: rows[0].len() as i64 - 1,
            south: rows.len() as i64 - 1,
        },
    )
}

fn ledges(rows: &[&str]) -> Rc<dyn Fn(Cell) -> Result<Option<&'static str>>> {
    let cells = grid(rows);
    Rc::new(
        move |cell: Cell| Ok((cells[cell.1 as usize][cell.0 as usize] == b'v').then_some(SOUTH)),
    )
}

fn said(path: Option<Vec<Cell>>) -> Option<String> {
    path.map(|path| spoken(&legs(&path, None).unwrap()))
}

fn plain() -> Options<'static> {
    Options::default()
}

#[test]
fn test_open_ground_is_crossed_in_two_legs() {
    let (walkable, box_) = drawn(&["......"; 3]);
    let path = find(
        (0, 2),
        &goals(&[((5, 0), None)]),
        &*walkable,
        box_,
        &plain(),
    )
    .unwrap();
    assert_eq!(said(path).as_deref(), Some("5 east, 2 north"));
}

/// Every cell a way can end on, by `find`'s moves, an end stepped into and not on from.
#[test]
fn test_reached_is_where_find_goes_a_ledge_down_and_never_up() {
    let rows = ["...#.", "vv###", "...E."];
    let (walkable, box_) = drawn(&rows);
    let ledge = ledges(&rows);
    let ends: HashSet<Cell> = HashSet::from([(3, 2)]);
    let is_end = |cell: Cell| Ok(ends.contains(&cell));
    let below = reached(
        (0, 2),
        &|cell: Cell| Ok(walkable(cell)? && !ends.contains(&cell)),
        box_,
        &Reach {
            ledge: Some(&*ledge),
            ends: Some(&is_end),
            ..Reach::default()
        },
    )
    .unwrap();
    assert_eq!(
        below.keys().copied().collect::<HashSet<Cell>>(),
        HashSet::from([(0, 2), (1, 2), (2, 2), (3, 2)])
    );
    let above = reached(
        (0, 0),
        &*walkable,
        box_,
        &Reach {
            ledge: Some(&*ledge),
            ..Reach::default()
        },
    )
    .unwrap();
    assert_eq!(
        above.keys().copied().collect::<HashSet<Cell>>(),
        HashSet::from([
            (0, 0),
            (1, 0),
            (2, 0),
            (0, 2),
            (1, 2),
            (2, 2),
            (3, 2),
            (4, 2)
        ])
    );
}

/// The way to a door ends facing the door, so the last step is the one that goes in.
#[test]
fn test_a_goal_asking_to_be_faced_a_way_is_reached_going_that_way() {
    let (walkable, box_) = drawn(&["......"; 3]);
    let path = find(
        (0, 2),
        &goals(&[((5, 0), Some(EAST))]),
        &*walkable,
        box_,
        &plain(),
    )
    .unwrap();
    assert_eq!(said(path).as_deref(), Some("2 north, 5 east"));
    let path = find(
        (0, 2),
        &goals(&[((5, 0), Some(NORTH))]),
        &*walkable,
        box_,
        &plain(),
    )
    .unwrap();
    assert_eq!(said(path).as_deref(), Some("5 east, 2 north"));
}

#[test]
fn test_a_player_already_walking_one_way_goes_on_that_way_first() {
    let (walkable, box_) = drawn(&["......"; 3]);
    let north = Options {
        heading: NORTH,
        ..plain()
    };
    let path = find((0, 2), &goals(&[((5, 0), None)]), &*walkable, box_, &north).unwrap();
    assert_eq!(said(path).as_deref(), Some("2 north, 5 east"));
    let path = find(
        (0, 2),
        &goals(&[((5, 0), Some(NORTH))]),
        &*walkable,
        box_,
        &north,
    )
    .unwrap();
    assert_eq!(said(path).as_deref(), Some("2 north, 5 east"));
}

#[test]
fn test_a_wall_is_gone_round() {
    let (walkable, box_) = drawn(&["......", ".####.", "......"]);
    let path = find(
        (2, 2),
        &goals(&[((2, 0), None)]),
        &*walkable,
        box_,
        &plain(),
    )
    .unwrap();
    assert_eq!(said(path).as_deref(), Some("2 west, 2 north, 2 east"));
}

/// The shortest way weaves between the top two rows round their walls, 11 steps and 5
/// turns; the open row at the bottom is 13 steps and 2 turns.
#[test]
fn test_a_turn_costs_a_step_so_a_little_further_with_fewer_turns_wins() {
    let rows = ["..#...#..", "....#....", ".#######.", "........."];
    let (walkable, box_) = drawn(&rows);
    let path = find(
        (0, 0),
        &goals(&[((8, 1), None)]),
        &*walkable,
        box_,
        &plain(),
    )
    .unwrap();
    assert_eq!(said(path).as_deref(), Some("3 south, 8 east, 2 north"));
    let (walled, walled_box) = drawn(&[rows[0], rows[1], "#########"]);
    let path = find(
        (0, 0),
        &goals(&[((8, 1), None)]),
        &*walled,
        walled_box,
        &plain(),
    )
    .unwrap();
    assert_eq!(
        said(path).as_deref(),
        Some("1 south, 3 east, 1 north, 2 east, 1 south, 3 east")
    );
}

/// A door is walked round on the way elsewhere and into as the goal: a door no loaded chunk
/// held had "no path" from the rig (poke_center_pc.ctm, 2026-09-24).
#[test]
fn test_a_goal_is_walked_into_even_where_nothing_else_may_go() {
    let (walkable, box_) = drawn(&["..#.."]);
    let path = find(
        (0, 0),
        &goals(&[((2, 0), None)]),
        &*walkable,
        box_,
        &plain(),
    )
    .unwrap();
    assert_eq!(said(path).as_deref(), Some("2 east"));
}

/// Spike 160: Fortree City's platforms and the ground beside them are both open, 108 apart,
/// and P walked the player off one, "9 south" into nothing.
#[test]
fn test_a_step_off_a_drop_is_not_taken() {
    let (walkable, box_) = drawn(&["...", "...", "..."]);
    let drop: HashSet<(Cell, Cell)> = HashSet::from([((1, 0), (1, 1)), ((1, 1), (1, 0))]);
    let joined = |cell: Cell, step: Cell| Ok(!drop.contains(&(cell, step)));
    let way = find(
        (1, 0),
        &goals(&[((1, 2), None)]),
        &*walkable,
        box_,
        &Options {
            joined: Some(&joined),
            ..plain()
        },
    )
    .unwrap();
    assert_eq!(said(way).as_deref(), Some("1 east, 2 south, 1 west"));
}

#[test]
fn test_nothing_is_found_where_no_way_leads() {
    let (walkable, box_) = drawn(&["...#.", "...#.", "...#."]);
    assert_eq!(
        find(
            (0, 0),
            &goals(&[((4, 1), None)]),
            &*walkable,
            box_,
            &plain()
        )
        .unwrap(),
        None
    );
    assert_eq!(
        find((0, 0), &goals(&[]), &*walkable, box_, &plain()).unwrap(),
        None
    );
}

/// Issue 3: a ledge was walked round like a wall, and "no path" where a jump was the only
/// way down. The ledge's cell is in the path, so each cell is a step from the last.
#[test]
fn test_a_ledge_is_jumped_down_and_never_climbed() {
    let rows = ["......", "vvvvv.", "......"];
    let (walkable, box_) = drawn(&rows);
    let ledge = ledges(&rows);
    let ledged = Options {
        ledge: Some(&*ledge),
        ..plain()
    };
    let down = find((0, 0), &goals(&[((0, 2), None)]), &*walkable, box_, &ledged).unwrap();
    assert_eq!(down, Some(vec![(0, 0), (0, 1), (0, 2)]));
    let up = find((0, 2), &goals(&[((0, 0), None)]), &*walkable, box_, &ledged).unwrap();
    assert_eq!(said(up).as_deref(), Some("5 east, 2 north, 5 west"));
    let walled = ["......", "vvvvvv", "......"];
    let (walkable, box_) = drawn(&walled);
    let ledge = ledges(&walled);
    let ledged = Options {
        ledge: Some(&*ledge),
        ..plain()
    };
    let path = find((2, 0), &goals(&[((0, 2), None)]), &*walkable, box_, &ledged).unwrap();
    assert_eq!(said(path).as_deref(), Some("2 west, 2 south"));
    assert_eq!(
        find((0, 2), &goals(&[((0, 0), None)]), &*walkable, box_, &ledged).unwrap(),
        None
    );
}

#[test]
fn test_a_ledge_is_not_jumped_onto_a_wall() {
    let rows = ["...", "vvv", "###", "..."];
    let (walkable, box_) = drawn(&rows);
    let ledge = ledges(&rows);
    let ledged = Options {
        ledge: Some(&*ledge),
        ..plain()
    };
    assert_eq!(
        find((0, 0), &goals(&[((0, 3), None)]), &*walkable, box_, &ledged).unwrap(),
        None
    );
}

/// Granite Cave 1F: P said "3 west" up a sandy slope the game never lets the player onto on
/// foot (omega_ruby_cave.ctm). A slope is open, and is walked only the way it goes down.
#[test]
fn test_a_slope_is_walked_down_and_never_up() {
    let rows = ["..>>..", "######"];
    let (walkable, box_) = drawn(&rows);
    let slope_rows = grid(&rows);
    let slopes = move |cell: Cell| -> Result<Option<&'static str>> {
        Ok((slope_rows[cell.1 as usize][cell.0 as usize] == b'>').then_some(EAST))
    };
    let sloped = Options {
        ledge: Some(&slopes),
        ..plain()
    };
    let path = find((0, 0), &goals(&[((5, 0), None)]), &*walkable, box_, &sloped).unwrap();
    assert_eq!(said(path).as_deref(), Some("5 east"));
    assert_eq!(
        find((5, 0), &goals(&[((0, 0), None)]), &*walkable, box_, &sloped).unwrap(),
        None
    );
    let rows = ["..>>..", "......"];
    let (walkable, box_) = drawn(&rows);
    let path = find((5, 0), &goals(&[((0, 0), None)]), &*walkable, box_, &sloped).unwrap();
    assert_eq!(said(path).as_deref(), Some("1 south, 5 west, 1 north"));
}

/// Route 132: the player surfed south onto a current running west and was carried west, a
/// cell at a time, wherever they meant to go (movie43, spike 208). A current is entered any way
/// and left only its own, and lets go where its way is shut.
#[test]
fn test_a_current_carries_the_player_its_way_until_that_way_is_shut() {
    let rows = ["#.....", "<<<<<<", "......"];
    let (walkable, box_) = drawn(&rows);
    let cells = grid(&rows);
    let carry = move |cell: Cell| -> Result<Option<&'static str>> {
        Ok((cells[cell.1 as usize][cell.0 as usize] == b'<').then_some(WEST))
    };
    let runs = |path: Option<Vec<Cell>>| -> Option<String> {
        let how = |cell: Cell, step: Cell| -> Result<String> {
            Ok(if carry(cell)? == Some(towards(cell, step)) {
                "current".to_string()
            } else {
                String::new()
            })
        };
        path.map(|path| spoken(&legs(&path, Some(&how)).unwrap()))
    };
    let carried = Options {
        carry: Some(&carry),
        ..plain()
    };
    assert_eq!(
        runs(
            find(
                (4, 0),
                &goals(&[((4, 2), None)]),
                &*walkable,
                box_,
                &carried
            )
            .unwrap()
        )
        .as_deref(),
        Some("1 south, current 4 west, 1 south, 4 east")
    );
    assert_eq!(
        find(
            (4, 2),
            &goals(&[((4, 0), None)]),
            &*walkable,
            box_,
            &carried
        )
        .unwrap(),
        None
    );
    let expected: HashSet<Cell> = (0..6).flat_map(|x| [(x, 1), (x, 2)]).collect();
    assert_eq!(
        reached(
            (4, 2),
            &*walkable,
            box_,
            &Reach {
                carry: Some(&carry),
                ..Reach::default()
            }
        )
        .unwrap()
        .keys()
        .copied()
        .collect::<HashSet<Cell>>(),
        expected
    );
}

/// Going onto water to surf costs more than a step: the way on foot is taken unless the
/// way over the water saves more than the toll.
#[test]
fn test_a_toll_is_paid_only_where_the_way_it_saves_is_longer() {
    let rows = ["......", "~~~~~.", "......"];
    let (walkable, box_) = drawn(&rows);
    let toll_rows = grid(&rows);
    let toll = move |cell: Cell, step: Cell| -> Result<i64> {
        Ok(
            if toll_rows[step.1 as usize][step.0 as usize] == b'~'
                && toll_rows[cell.1 as usize][cell.0 as usize] != b'~'
            {
                10
            } else {
                0
            },
        )
    };
    let tolled = Options {
        toll: Some(&toll),
        ..plain()
    };
    let path = find((0, 0), &goals(&[((0, 2), None)]), &*walkable, box_, &tolled).unwrap();
    assert_eq!(said(path).as_deref(), Some("2 south"));
    let path = find(
        (4, 0),
        &goals(&[((4, 2), None)]),
        &*walkable,
        box_,
        &plain(),
    )
    .unwrap();
    assert_eq!(said(path).as_deref(), Some("2 south"));
    let path = find((4, 0), &goals(&[((4, 2), None)]), &*walkable, box_, &tolled).unwrap();
    assert_eq!(said(path).as_deref(), Some("1 east, 2 south, 1 west"));
}

#[test]
fn test_a_leg_ends_where_what_it_is_walked_with_changes() {
    let path = [(0, 0), (0, 1), (0, 2), (0, 3), (1, 3)];
    let water: HashSet<Cell> = HashSet::from([(0, 2), (0, 3), (1, 3)]);
    let how =
        |_cell: Cell, step: Cell| Ok(if water.contains(&step) { "Surf" } else { "" }.to_string());
    let runs = legs(&path, Some(&how)).unwrap();
    assert_eq!(
        runs,
        vec![
            Leg::new(SOUTH, 1, 1),
            Leg {
                how: "Surf".to_string(),
                ..Leg::new(SOUTH, 2, 3)
            },
            Leg {
                how: "Surf".to_string(),
                ..Leg::new(EAST, 1, 4)
            },
        ]
    );
    assert_eq!(spoken(&runs), "1 south, Surf 2 south, Surf 1 east");
}

#[test]
fn test_a_search_stays_inside_its_box() {
    let (walkable, _) = drawn(&["......"; 3]);
    let inside = Box {
        west: 0,
        north: 0,
        east: 3,
        south: 2,
    };
    assert_eq!(
        find(
            (0, 0),
            &goals(&[((5, 0), None)]),
            &*walkable,
            inside,
            &plain()
        )
        .unwrap(),
        None
    );
}

#[test]
fn test_standing_on_a_goal_is_a_path_of_one_cell() {
    let (walkable, box_) = drawn(&["..."]);
    assert_eq!(
        find(
            (1, 0),
            &goals(&[((1, 0), Some(EAST))]),
            &*walkable,
            box_,
            &plain()
        )
        .unwrap(),
        Some(vec![(1, 0)])
    );
}

#[test]
fn test_legs_say_where_along_the_path_each_ends() {
    let path = [(0, 2), (1, 2), (2, 2), (2, 1), (2, 0)];
    assert_eq!(
        legs(&path, None).unwrap(),
        vec![Leg::new(EAST, 2, 2), Leg::new(NORTH, 2, 4)]
    );
    assert_eq!(spoken(&legs(&path, None).unwrap()), "2 east, 2 north");
    assert_eq!(legs(&path[..1], None).unwrap(), vec![]);
}

/// Lumiose's boulevards (spike 105): up walks the way the camera looks, a held left or right
/// turns it a quarter, and down walks back without turning it, so a leg to one side is a hold
/// and one behind is steps back, the camera still looking the way it did.
#[test]
fn test_a_way_is_said_from_the_camera_for_a_player_whose_keys_work_from_it() {
    let way = [
        Leg::new(NORTH, 3, 3),
        Leg::new(EAST, 5, 8),
        Leg::new(NORTH, 2, 10),
        Leg::new(SOUTH, 1, 11),
        Leg::new(WEST, 2, 13),
    ];
    assert_eq!(
        turned(&way, NORTH, false).unwrap(),
        "3 forward, hold right, 5 forward, hold left, 2 forward, 1 back, hold left, 2 forward"
    );
    assert_eq!(
        turned(&[Leg::new(WEST, 4, 4)], NORTH, false).unwrap(),
        "hold left, 4 forward"
    );
}

/// A tap leaves the camera anywhere, and up walks across the street at its angle; a hold
/// always settles it a quarter from the way it is nearest, so one each way lines it up, and a
/// first leg to one side needs only its own hold (spike 105).
#[test]
fn test_a_camera_left_between_two_ways_is_straightened_before_walking_ahead_or_back() {
    assert_eq!(
        turned(&[Leg::new(NORTH, 5, 5)], NORTH, true).unwrap(),
        "camera angled, hold left, hold right, 5 forward"
    );
    assert_eq!(
        turned(&[Leg::new(SOUTH, 2, 2), Leg::new(EAST, 1, 3)], NORTH, true).unwrap(),
        "camera angled, hold left, hold right, 2 back, hold right, 1 forward"
    );
    assert_eq!(
        turned(&[Leg::new(EAST, 4, 4)], NORTH, true).unwrap(),
        "hold right, 4 forward"
    );
}

#[test]
fn test_a_bearing_is_the_way_it_is_nearest_and_how_far_off_it() {
    let ways: Vec<&str> = [10.0, 50.0, 181.0, 314.0, 316.0]
        .into_iter()
        .map(|bearing| nearest(bearing).unwrap())
        .collect();
    assert_eq!(ways, [NORTH, EAST, SOUTH, WEST, NORTH]);
    assert!(
        off(80.0, EAST).unwrap() == -10.0
            && off(10.0, NORTH).unwrap() == 10.0
            && off(350.0, NORTH).unwrap() == -10.0
    );
}

#[test]
fn test_a_place_is_said_from_the_way_the_player_faces() {
    assert_eq!(facing((0, 0), (8, -9), NORTH).unwrap(), "9 ahead 8 right");
    assert_eq!(facing((0, 0), (8, -9), EAST).unwrap(), "8 ahead 9 left");
    assert_eq!(facing((0, 0), (0, 3), NORTH).unwrap(), "3 behind");
    assert_eq!(facing((2, 2), (2, 2), WEST).unwrap(), "here");
}

/// Spike 167: each of Sootopolis Gym's rooms of thin ice is cleared by stepping on every tile
/// once and last on an exit, the top row's middle three, the blocks of floor among the ice
/// crossed as often as wanted. Its first room, 5 by 5, cannot be done from a corner's colour,
/// and its top room, 78 tiles round three blocks, was not done by Warnsdorff's rule alone.
#[test]
fn test_thin_ice_is_crossed_over_every_tile_once_and_left_by_an_exit() {
    type Room = (HashSet<Cell>, HashSet<Cell>, Vec<Cell>);

    fn room(corner: Cell, size: (i64, i64), blocks: &[Cell]) -> Room {
        let cells: HashSet<Cell> = (0..size.0)
            .flat_map(|x| (0..size.1).map(move |z| (corner.0 + x, corner.1 + z)))
            .collect();
        let free: HashSet<Cell> = blocks
            .iter()
            .flat_map(|&(x, z)| (0..3).flat_map(move |dx| (0..3).map(move |dz| (x + dx, z + dz))))
            .collect();
        (
            cells.difference(&free).copied().collect(),
            free,
            [17, 18, 19].iter().map(|&x| (x, corner.1)).collect(),
        )
    }

    let rooms: [(Room, Cell); 3] = [
        (room((16, 48), (5, 5), &[]), (18, 52)),
        (room((15, 36), (7, 7), &[(17, 38)]), (17, 42)),
        (
            room((11, 24), (15, 7), &[(13, 26), (17, 26), (21, 26)]),
            (19, 30),
        ),
    ];
    for ((once, free, exits), entry) in &rooms {
        let way = cover(
            *entry,
            once.iter().copied(),
            free.iter().copied(),
            exits.iter().copied(),
        )
        .expect("a way over the ice");
        assert!(exits.contains(&way[way.len() - 1]));
        assert!(
            once.iter()
                .all(|cell| way.iter().filter(|c| *c == cell).count() == 1)
        );
        assert!(
            way.iter()
                .all(|cell| once.contains(cell) || free.contains(cell))
        );
        assert!(
            way.windows(2)
                .all(|pair| (pair[0].0 - pair[1].0).abs() + (pair[0].1 - pair[1].1).abs() == 1)
        );
    }
    let (first, free, exits) = room((16, 48), (5, 5), &[]);
    assert_eq!(
        cover(
            (17, 52),
            first.iter().copied(),
            free.iter().copied(),
            exits.iter().copied()
        ),
        None
    );
}
