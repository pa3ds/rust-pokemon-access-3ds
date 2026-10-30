//! Sun's walls: an area's map blocks read from its block table, their floor and walls turned into
//! cells, in small fakes laid out as spike 114 read the ROM.

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use indexmap::IndexMap;

use crate::core::pathfind::{self, Options};
use crate::games::common::field::chunk::{CHUNK, Chunk};
use crate::games::common::field::ground::Ground;
use crate::games::sm::walls::{self, AreaWalls, SunWalls};
use crate::games::sm::zones::{AREA_FILES, EVENTS};
use crate::py::{Error, Kind, Result};

const CELL: f64 = 50.0;
/// A block's side in the fakes: twenty cells.
const SIZE: i64 = 1000;
const UP: (f64, f64, f64) = (0.0, 1.0, 0.0);
const EAST: (f64, f64, f64) = (1.0, 0.0, 0.0);
const WEST: (f64, f64, f64) = (-1.0, 0.0, 0.0);
const SOUTH: (f64, f64, f64) = (0.0, 0.0, 1.0);
const NORTH: (f64, f64, f64) = (0.0, 0.0, -1.0);

type Corner = (f64, f64, f64);
type Tri = (Corner, Corner, Corner, (f64, f64, f64), i64);

fn mesh(triangles: &[Tri]) -> Vec<u8> {
    let mut out = walls::MESH.to_vec();
    out.extend((triangles.len() as u32).to_le_bytes());
    for &(a, b, c, normal, kind) in triangles {
        let values = [
            a.0, a.1, a.2, 1.0, b.0, b.1, b.2, 1.0, c.0, c.1, c.2, 1.0, normal.0, normal.1,
            normal.2, 0.0,
        ];
        for value in values {
            out.extend((value as f32).to_le_bytes());
        }
        out.extend((kind as u32).to_le_bytes());
        out.extend(0xFFFFFFu32.to_le_bytes());
    }
    out
}

fn pack(magic: &[u8], entries: &[Vec<u8>]) -> Vec<u8> {
    let head = 4 + 4 * (entries.len() as i64 + 1);
    let mut offsets = vec![head];
    for entry in entries {
        offsets.push(offsets[offsets.len() - 1] + entry.len() as i64);
    }
    let mut out = magic.to_vec();
    out.extend((entries.len() as u16).to_le_bytes());
    for offset in offsets {
        out.extend((offset as u32).to_le_bytes());
    }
    out.extend(entries.concat());
    out
}

fn floor(west: f64, north: f64, east: f64, south: f64) -> Vec<Tri> {
    floor_at(west, north, east, south, 1, 0.0)
}

/// `floor(west, north, east, south, kind, y=...)`.
fn floor_at(west: f64, north: f64, east: f64, south: f64, kind: i64, y: f64) -> Vec<Tri> {
    vec![
        (
            (west, y, north),
            (east, y, north),
            (west, y, south),
            UP,
            kind,
        ),
        (
            (east, y, north),
            (east, y, south),
            (west, y, south),
            UP,
            kind,
        ),
    ]
}

/// A floor from its four corners, (x, y, z), which may be at different heights.
fn slope(nw: Corner, ne: Corner, se: Corner, sw: Corner) -> Vec<Tri> {
    vec![(nw, ne, sw, UP, 1), (ne, se, sw, UP, 1)]
}

/// An upright wall from `a` to `b`, seen from above as (x, z), facing `facing`, from y -25 to
/// 425.
fn wall(a: (f64, f64), b: (f64, f64), facing: (f64, f64, f64)) -> Vec<Tri> {
    wall_over(a, b, facing, -25.0, 425.0)
}

/// An upright wall from `a` to `b`, seen from above as (x, z), facing `facing`, from y `low`
/// to `high`.
fn wall_over(
    a: (f64, f64),
    b: (f64, f64),
    facing: (f64, f64, f64),
    low: f64,
    high: f64,
) -> Vec<Tri> {
    let ((ax, az), (bx, bz)) = (a, b);
    vec![
        (
            (ax, low, az),
            (bx, low, bz),
            (ax, high, az),
            facing,
            0xFFFFFFFF,
        ),
        (
            (bx, low, bz),
            (bx, high, bz),
            (ax, high, az),
            facing,
            0xFFFFFFFF,
        ),
    ]
}

/// A map block's file: its model, five empty entries, the floor and a pack of four wall
/// meshes.
fn block(floor_: &[Tri], walls_: &[Tri]) -> Vec<u8> {
    let wall_meshes = pack(
        walls::PACK,
        &[mesh(walls_), mesh(&[]), mesh(&[]), mesh(&[])],
    );
    pack(
        walls::PACK,
        &[
            b"gfmodel".to_vec(),
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            mesh(floor_),
            wall_meshes,
        ],
    )
}

/// An area's block table, a 'TR' pack of one.
fn table(width: i64, height: i64, files: &IndexMap<(i64, i64), i64>) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();
    for value in [width, height, SIZE, 0, 0] {
        body.extend((value as u32).to_le_bytes());
    }
    for z in 0..height {
        for x in 0..width {
            let file = files.get(&(x, z)).copied().unwrap_or(walls::NO_FILE);
            body.extend((file as u16).to_le_bytes());
            body.extend(0xFFFFu16.to_le_bytes());
            body.extend(0xFFFFu16.to_le_bytes());
            body.extend(0u16.to_le_bytes());
            body.extend(0u32.to_le_bytes());
        }
    }
    pack(walls::TABLE, &[body])
}

/// A block of floor with a room walled in from x and z 110 to 890, facing in; a door in its
/// south wall from 475 to 575, which leaves one cell open between its sides; and a bed from
/// (610, 160) to (810, 360), walled all round and facing out, with floor under it.
fn room() -> Vec<u8> {
    let size = SIZE as f64;
    block(
        &floor(0.0, 0.0, size, size),
        &[
            wall((110.0, 110.0), (890.0, 110.0), SOUTH),
            wall((110.0, 110.0), (110.0, 890.0), EAST),
            wall((890.0, 110.0), (890.0, 890.0), WEST),
            wall((110.0, 890.0), (475.0, 890.0), NORTH),
            wall((575.0, 890.0), (890.0, 890.0), NORTH),
            wall((475.0, 890.0), (475.0, 960.0), EAST),
            wall((575.0, 890.0), (575.0, 960.0), WEST),
            wall((610.0, 160.0), (810.0, 160.0), NORTH),
            wall((610.0, 160.0), (610.0, 360.0), WEST),
            wall((810.0, 160.0), (810.0, 360.0), EAST),
            wall((610.0, 360.0), (810.0, 360.0), SOUTH),
        ]
        .concat(),
    )
}

fn area(files: &IndexMap<(i64, i64), Vec<u8>>) -> AreaWalls {
    area_with(files, true)
}

/// `area(files, heights=...)`.
fn area_with(files: &IndexMap<(i64, i64), Vec<u8>>, heights: bool) -> AreaWalls {
    let numbered: IndexMap<(i64, i64), i64> = files
        .keys()
        .enumerate()
        .map(|(index, &place)| (place, index as i64))
        .collect();
    let grid = walls::block_table(&table(3, 3, &numbered)).unwrap();
    let blocks: IndexMap<(i64, i64), Vec<u8>> = grid
        .files
        .keys()
        .map(|&place| (place, files[&place].clone()))
        .collect();
    walls::rasterize(&grid, &blocks, CELL, heights).unwrap()
}

fn ground_of(chunks: Vec<Chunk>) -> Ground {
    Ground {
        chunks: chunks.into_iter().map(Rc::new).collect(),
        ..Ground::default()
    }
}

fn places(files: Vec<((i64, i64), Vec<u8>)>) -> IndexMap<(i64, i64), Vec<u8>> {
    files.into_iter().collect()
}

/// The way P finds from a player standing at `height`, on the chunks the area gives it.
fn way(
    ground: &AreaWalls,
    start: (i64, i64),
    goal: (i64, i64),
    height: f64,
) -> Option<Vec<(i64, i64)>> {
    let land = ground_of(
        ground
            .chunks(None, 0, Some(start), 0, Some(height))
            .unwrap(),
    );
    let goals = IndexMap::from([(goal, None)]);
    let found = pathfind::find(
        start,
        &goals,
        &|cell| land.passable(cell),
        land.r#box(start, [goal]).unwrap(),
        &Options::default(),
    )
    .unwrap();
    found.map(|found| pathfind::cells(&found))
}

#[test]
fn test_a_mesh_is_its_triangles_corners_normal_and_kind() {
    let triangles = walls::mesh(&mesh(&[(
        (1.0, 2.0, 3.0),
        (4.0, 5.0, 6.0),
        (7.0, 8.0, 9.0),
        UP,
        31,
    )]))
    .unwrap();
    assert_eq!(triangles.len(), 1);
    let triangle = &triangles[0];

    assert_eq!(
        triangle.corners,
        [(1.0, 2.0, 3.0), (4.0, 5.0, 6.0), (7.0, 8.0, 9.0)]
    );
    assert_eq!(triangle.normal, UP);
    assert_eq!(triangle.kind, 31);
    assert!(walls::mesh(b"BG\x00\x00").unwrap_err().is(Kind::RomError));
}

#[test]
fn test_a_block_table_places_each_block_by_its_record_row_by_row() {
    let files = IndexMap::from([((0, 0), 220), ((2, 0), 231), ((1, 1), 228)]);
    let read = walls::block_table(&table(3, 2, &files)).unwrap();

    assert_eq!((read.width, read.height, read.size), (3, 2, SIZE));
    assert_eq!(read.files, files);
}

#[test]
fn test_a_cell_is_open_on_floor_and_more_than_half_a_cell_from_a_wall() {
    let ground = area(&places(vec![((0, 0), room())]));

    assert_eq!(ground.open((6, 12)), Some(true));
    assert_eq!(
        ground.open((2, 12)),
        Some(false),
        "15 units from the west wall"
    );
    assert_eq!(ground.open((3, 12)), Some(true), "65 units from it");
    assert_eq!(
        ground.open((10, 18)),
        Some(true),
        "in the door, 50 units from either side"
    );
    assert_eq!(ground.open((9, 18)), Some(false), "on the door's side");
    assert_eq!(ground.open((25, 5)), None, "no block of the area holds it");
}

#[test]
fn test_a_cell_with_no_floor_under_it_is_closed() {
    let ground = area(&places(vec![(
        (0, 0),
        block(&floor(0.0, 0.0, 250.0, SIZE as f64), &[]),
    )]));

    assert_eq!(ground.open((4, 4)), Some(true));
    assert_eq!(ground.open((6, 4)), Some(false));
}

#[test]
fn test_a_wall_on_the_line_between_two_cells_closes_the_one_behind_it() {
    let size = SIZE as f64;
    let ground = area(&places(vec![(
        (0, 0),
        block(
            &floor(0.0, 0.0, size, size),
            &wall((0.0, 200.0), (size, 200.0), SOUTH),
        ),
    )]));

    assert_eq!(
        ground.open((4, 4)),
        Some(true),
        "in front of the wall, which faces south"
    );
    assert_eq!(ground.open((4, 3)), Some(false), "behind it");
}

#[test]
fn test_a_way_is_found_round_the_walls_and_out_through_the_door() {
    let ground = area(&places(vec![((0, 0), room())]));
    let land = ground_of(ground.chunks(None, 0, None, 0, None).unwrap());
    let goal = IndexMap::from([((10, 19), None)]);

    let found = pathfind::find(
        (6, 12),
        &goal,
        &|cell| land.passable(cell),
        land.r#box((6, 12), [(10, 19)]).unwrap(),
        &Options::default(),
    )
    .unwrap();

    let path = pathfind::cells(&found.expect("a way"));
    assert!(path.contains(&(10, 18)), "through the door");
    assert!(path.iter().all(|&cell| land.open(cell).unwrap()));
}

#[test]
fn test_chunks_close_the_cells_the_player_cannot_walk_to() {
    let ground = area(&places(vec![((0, 0), room())]));

    let anywhere = ground_of(ground.chunks(None, 0, None, 0, None).unwrap());
    let in_the_room = ground_of(ground.chunks(None, 0, Some((6, 12)), 0, None).unwrap());

    assert!(
        anywhere.open((14, 5)).unwrap(),
        "floor in the middle of the bed"
    );
    assert!(!in_the_room.open((14, 5)).unwrap());
    assert!(
        in_the_room.open((10, 19)).unwrap(),
        "outside, through the door"
    );
    assert!(
        in_the_room.open((25, 5)).unwrap(),
        "no block holds it: not known, so open"
    );
    let inside = ground.reachable((6, 12), None);
    let near_the_wall = ground.reachable((2, 12), None);
    assert!(
        inside.is_subset(&near_the_wall) && inside != near_the_wall,
        "too near a wall to be open, the player may be either side of it"
    );
}

#[test]
fn test_chunks_near_cells_are_only_those_round_them() {
    let ground = area(&places(vec![((0, 0), room()), ((2, 2), room())]));

    let near = ground.chunks(Some(&[(6, 12)]), 2, None, 0, None).unwrap();

    assert_eq!(
        near.iter().map(|chunk| chunk.origin).collect::<Vec<_>>(),
        [(0, 0)]
    );
    assert_eq!(
        ground
            .chunks(None, 0, None, 0, None)
            .unwrap()
            .iter()
            .map(|chunk| chunk.origin)
            .collect::<Vec<_>>(),
        [(0, 0), (40, 40)]
    );
    assert!(
        near.iter()
            .all(|chunk| chunk.words.len() as i64 == CHUNK * CHUNK)
    );
}

/// Ground at y 0, and a bridge 200 above it from z 400 to 600, all the way across, its
/// railings from 200 to 300.
fn bridge() -> Vec<u8> {
    let size = SIZE as f64;
    block(
        &[
            floor(0.0, 0.0, size, size),
            floor_at(0.0, 400.0, size, 600.0, 1, 200.0),
        ]
        .concat(),
        &[
            wall_over((0.0, 400.0), (size, 400.0), SOUTH, 200.0, 300.0),
            wall_over((0.0, 600.0), (size, 600.0), NORTH, 200.0, 300.0),
        ]
        .concat(),
    )
}

#[test]
fn test_a_path_under_a_bridge_is_not_closed_by_the_bridges_railings() {
    let ground = area(&places(vec![((0, 0), bridge())]));
    let flat = area_with(&places(vec![((0, 0), bridge())]), false);

    assert_eq!(ground.floors((5, 9)), [(0, 0), (200, 200)]);
    let under = ground.pocket((5, 9), Some(0.0));
    assert!(
        ground.pocket((5, 2), Some(0.0)) == under && under == ground.pocket((5, 17), Some(0.0))
    );
    assert_ne!(ground.pocket((5, 9), Some(200.0)), under);
    assert!(way(&ground, (5, 2), (5, 17), 0.0).is_some());
    assert_ne!(
        flat.pocket((5, 2), None),
        flat.pocket((5, 17), None),
        "every wall closes every floor"
    );
}

#[test]
fn test_a_player_on_the_bridge_is_not_given_the_path_under_it() {
    let ground = area(&places(vec![((0, 0), bridge())]));

    let on = ground_of(
        ground
            .chunks(None, 0, Some((5, 9)), 0, Some(200.0))
            .unwrap(),
    );
    let under = ground_of(ground.chunks(None, 0, Some((5, 9)), 0, Some(0.0)).unwrap());

    assert!(on.open((15, 10)).unwrap() && !on.open((5, 2)).unwrap());
    assert!(under.open((5, 2)).unwrap());
}

/// Ground at y 0; a porch 80 above it from x and z 200 to 600, with no rail; and stairs
/// down its south side from x 350 to 450, a slope to the ground at z 760.
fn porch() -> Vec<u8> {
    let size = SIZE as f64;
    block(
        &[
            floor(0.0, 0.0, size, size),
            floor_at(200.0, 200.0, 600.0, 600.0, 1, 80.0),
            slope(
                (350.0, 80.0, 600.0),
                (450.0, 80.0, 600.0),
                (450.0, 0.0, 760.0),
                (350.0, 0.0, 760.0),
            ),
        ]
        .concat(),
        &[],
    )
}

/// The ground runs on under Route 1's porch 41 to 76 units below its deck (spike 120).
#[test]
fn test_the_ground_under_a_porch_is_not_stood_on() {
    let ground = area(&places(vec![((0, 0), porch())]));

    assert_eq!(ground.floors((5, 8)), [(80, 80)]);
    assert_eq!(
        ground.floors((7, 12)),
        [(61, 74)],
        "the top of the stairs, over the ground"
    );
}

#[test]
fn test_a_way_off_a_porch_goes_down_its_stairs() {
    let ground = area(&places(vec![((0, 0), porch())]));
    let flat = area_with(&places(vec![((0, 0), porch())]), false);

    let path = way(&ground, (5, 8), (5, 17), 80.0).expect("a way");
    let straight = way(&flat, (5, 8), (5, 17), 80.0);

    assert!(
        path.contains(&(7, 12)) || path.contains(&(8, 12)),
        "down the stairs"
    );
    assert!(!path.contains(&(5, 12)));
    assert!(
        straight.is_some_and(|straight| straight.contains(&(5, 12))),
        "heights not read: off the edge"
    );
}

/// A plateau at y 200 west of x 500 and ground at 0 east of it, with no wall between them
/// as along Iki Town's edge over Route 1, joined south of z 800 by a slope from x 300 to 700;
/// walled on the other three sides, so no way goes round through cells no block holds.
fn cliff() -> Vec<u8> {
    let size = SIZE as f64;
    block(
        &[
            floor_at(0.0, 0.0, 500.0, 800.0, 1, 200.0),
            floor(500.0, 0.0, size, 800.0),
            floor_at(0.0, 800.0, 300.0, size, 1, 200.0),
            floor(700.0, 800.0, size, size),
            slope(
                (300.0, 200.0, 800.0),
                (700.0, 0.0, 800.0),
                (700.0, 0.0, size),
                (300.0, 200.0, size),
            ),
        ]
        .concat(),
        &[
            wall((0.0, 25.0), (size, 25.0), SOUTH),
            wall((25.0, 0.0), (25.0, size), EAST),
            wall((975.0, 0.0), (975.0, size), WEST),
        ]
        .concat(),
    )
}

#[test]
fn test_a_cliff_with_no_wall_is_walked_round_from_either_side() {
    let ground = area(&places(vec![((0, 0), cliff())]));

    let above = ground_of(
        ground
            .chunks(None, 0, Some((8, 5)), 0, Some(200.0))
            .unwrap(),
    );
    let below = ground_of(ground.chunks(None, 0, Some((12, 5)), 0, Some(0.0)).unwrap());
    let path = way(&ground, (8, 5), (12, 5), 200.0);

    assert_eq!(
        ground.pocket((5, 5), Some(200.0)),
        ground.pocket((15, 5), Some(0.0)),
        "joined by the slope"
    );
    assert!(
        above.open((9, 5)).unwrap() && !above.open((10, 5)).unwrap(),
        "the cell further from the player"
    );
    assert!(below.open((10, 5)).unwrap() && !below.open((9, 5)).unwrap());
    assert!(
        path.is_some_and(|path| path.iter().map(|&(_, z)| z).max().unwrap() >= 16),
        "round by the slope"
    );
}

/// A ROM's two archives by file number, counting what is read.
struct Archives {
    files: HashMap<String, HashMap<i64, Vec<u8>>>,
    reads: Mutex<Vec<(String, Vec<i64>)>>,
}

impl Archives {
    fn new(events: HashMap<i64, Vec<u8>>, maps: HashMap<i64, Vec<u8>>) -> Archives {
        Archives {
            files: HashMap::from([
                (EVENTS.to_string(), events),
                (walls::MAPS.to_string(), maps),
            ]),
            reads: Mutex::new(Vec::new()),
        }
    }

    fn call(&self, archive: &str, files: &[i64]) -> Result<Vec<Vec<u8>>> {
        self.reads
            .lock()
            .unwrap()
            .push((archive.to_string(), files.to_vec()));
        files
            .iter()
            .map(|file| match self.files[archive].get(file) {
                Some(data) => Ok(data.clone()),
                None => Err(Error::new(Kind::RomError, format!("no file {file}"))),
            })
            .collect()
    }

    fn reads(&self) -> Vec<(String, Vec<i64>)> {
        self.reads.lock().unwrap().clone()
    }
}

fn reader(archives: &Arc<Archives>) -> walls::Read {
    let archives = Arc::clone(archives);
    Arc::new(move |archive, files| archives.call(archive, files))
}

fn rom(areas: &[i64]) -> Arc<Archives> {
    let events = areas
        .iter()
        .map(|area| {
            (
                area * AREA_FILES + walls::BLOCK_TABLE,
                table(1, 1, &IndexMap::from([((0, 0), 7)])),
            )
        })
        .collect();
    Arc::new(Archives::new(events, HashMap::from([(7, room())])))
}

#[test]
fn test_an_area_is_read_the_first_time_it_is_asked_for_and_then_kept() {
    let archives = rom(&[21, 0]);
    let walls_ = SunWalls::new(reader(&archives), CELL);

    let first = walls_.area(21).unwrap();
    let again = walls_.area(21).unwrap();

    let (first, again) = (first.expect("walls"), again.expect("walls"));
    assert!(Arc::ptr_eq(&first, &again));
    assert_eq!(
        archives.reads(),
        [
            (
                EVENTS.to_string(),
                vec![21 * AREA_FILES + walls::BLOCK_TABLE]
            ),
            (walls::MAPS.to_string(), vec![7]),
        ]
    );
    assert_eq!(first.open((6, 12)), Some(true));
}

#[test]
fn test_only_the_areas_asked_for_last_are_kept() {
    let archives = rom(&[1, 2, 3]);
    let walls_ = SunWalls::with_keep(reader(&archives), CELL, 2);

    walls_.area(1).unwrap();
    walls_.area(2).unwrap();
    walls_.area(1).unwrap();
    walls_.area(3).unwrap();
    let before = archives.reads().len();
    walls_.area(1).unwrap();
    walls_.area(2).unwrap();

    assert_eq!(
        archives.reads().len(),
        before + 2,
        "2 was dropped for 3, 1 was not"
    );
}

#[test]
fn test_an_area_the_rom_cannot_give_has_no_walls_and_says_why() {
    let walls_ = SunWalls::new(reader(&rom(&[21])), CELL);

    assert!(walls_.area(22).unwrap().is_none());
    assert!(walls_.errors()[&22].contains("no file"));
}

/// Route 1's walls took 0.8 s to read, and its first P 2 s: they are read as the player
/// arrives, and a P pressed meanwhile waits for them (sun_4.ctm, 2026-09-26).
#[test]
fn test_an_area_read_ahead_is_read_once_when_p_asks_for_it_meanwhile() {
    let archives = rom(&[21]);
    let walls_ = SunWalls::new(reader(&archives), CELL);
    walls_.prefetch(21);
    assert!(walls_.area(21).unwrap().is_some());
    assert_eq!(
        archives.reads(),
        [
            (
                EVENTS.to_string(),
                vec![21 * AREA_FILES + walls::BLOCK_TABLE]
            ),
            (walls::MAPS.to_string(), vec![7]),
        ]
    );
}
