//! Rails: Lumiose City's lines, found through the game's table of those loaded, and laid end to end
//! into the corridor the player walks (spikes 102 and 103).

use std::collections::HashSet;

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use crate::core::discovery::ALLOCATION;
use crate::games::gen6::field::rail::{self, Line, Network};
use crate::py::Result;

pub(super) const TABLE: i64 = 0x08247214;
const OWNER: i64 = 0x08247CC4;
pub(super) const FILE: i64 = 0x147FA080;
// Three lines of Lumiose's, as spike 103 read them: A ends where B starts, and C, laid the other
// way round, ends there too. A line's id is its start point's with bit 15 set.
pub(super) const A: i64 = 0x0305AC3B;
pub(super) const B: i64 = 0x0504BEF6;
pub(super) const C: i64 = 0x0504F514;
const P0: i64 = 0x03052C3B;
const P1: i64 = 0x05043EF6;
pub(super) const P2: i64 = 0x05045991;
pub(super) const P3: i64 = 0x05047514;
/// Seven cells across, three either side of the middle.
pub(super) const ACROSS: i64 = 7;

/// A line's wall table: the outer column each side a wall, and `blocked` (row, offset)
/// besides.
pub(super) fn walls(along: i64, blocked: &[(i64, i64)]) -> Vec<i64> {
    let mut out = Vec::new();
    for row in 0..=along {
        for column in 0..ACROSS {
            let offset = column - ACROSS / 2;
            let wall = offset.abs() == ACROSS / 2 || blocked.contains(&(row, offset));
            out.push(if wall { 0x01000021 } else { 0x20 });
        }
    }
    out
}

fn words(values: &[i64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&value| (value as u32).to_le_bytes())
        .collect()
}

/// A line's record at `at` in the network's file, and its walls after it.
fn line_record(
    memory: &FakeMemory,
    at: i64,
    line: i64,
    start: i64,
    end: i64,
    along: i64,
    blocked: &[(i64, i64)],
) {
    let table = at + 0x100 - FILE;
    let mut head = vec![0u8; rail::LINE_HEAD as usize];
    head[..20].copy_from_slice(&words(&[line, 0x504, start, end, rail::WAY_SOUTH]));
    let put = |head: &mut Vec<u8>, at: i64, bytes: &[u8]| {
        head[at as usize..at as usize + bytes.len()].copy_from_slice(bytes);
    };
    put(&mut head, rail::LINE_ALONG, &(along as u16).to_le_bytes());
    put(
        &mut head,
        rail::LINE_ALONG + 2,
        &(ACROSS as u16).to_le_bytes(),
    );
    put(&mut head, rail::LINE_FILE, &words(&[FILE]));
    put(&mut head, rail::LINE_WALLS, &words(&[table]));
    memory.write(at, &head);
    memory.write(FILE + table, &words(&walls(along, blocked)));
}

/// The table listing lines A, B and C loaded, and one entry for a line no longer loaded.
pub(super) fn network(memory: &FakeMemory) {
    let records = [(A, FILE + 0xE8), (B, FILE + 0x5580), (C, FILE + 0x84E8)];
    line_record(memory, records[0].1, A, P0, P1, 10, &[(4, 1)]);
    line_record(memory, records[1].1, B, P1, P2, 6, &[]);
    line_record(memory, records[2].1, C, P3, P2, 5, &[]);
    let mut entries: Vec<u8> = Vec::new();
    for (line, record) in records {
        entries.extend(words(&[OWNER, line, 0, record]));
    }
    entries.extend(words(&[0, 0x0205DED0, 5, 0]));
    let mut body = vec![0u8; 0x20];
    body.extend(entries);
    memory.u32(TABLE, &[ALLOCATION, body.len() as i64]);
    memory.write(TABLE + 8, &body);
}

/// What `FakeMemory.pages()` is in Python, a dict in the order the pages were first written: here
/// by address, which only matters where two pages hold the line.
fn pages(memory: &FakeMemory) -> IndexMap<i64, Vec<u8>> {
    memory.pages()
}

fn table_lines(memory: &FakeMemory) -> Result<IndexMap<i64, Line>> {
    rail::read_table(memory, TABLE, &IndexMap::new())
}

#[test]
fn test_the_table_is_found_through_the_line_the_player_is_on() -> Result<()> {
    let memory = FakeMemory::new();
    network(&memory);
    assert_eq!(rail::find_table(&pages(&memory), &*memory, B)?, Some(TABLE));
    assert_eq!(
        rail::find_table(&pages(&memory), &*memory, 0x0999_0000)?,
        None
    );
    let lines = table_lines(&memory)?;
    let mut ids: Vec<i64> = lines.keys().copied().collect();
    ids.sort();
    let mut expected = vec![A, B, C];
    expected.sort();
    assert_eq!(ids, expected);
    assert_eq!(
        (
            lines[&A].start,
            lines[&A].end,
            lines[&A].along,
            lines[&A].across
        ),
        (P0, P1, 10, ACROSS)
    );
    Ok(())
}

/// B runs on from where A ends; C ends where B ends, so it is laid the other way round, its
/// along running back and its offsets mirrored. Along is north and a falling offset east, as
/// the stick moves the player there (spike 103).
#[test]
fn test_lines_joined_at_a_point_are_laid_end_to_end_whichever_way_each_runs() -> Result<()> {
    let memory = FakeMemory::new();
    network(&memory);
    let corridor = rail::lay_out(&table_lines(&memory)?, B, None)?;
    let corridor = corridor.expect("a corridor");
    assert_eq!(corridor.cell(B, 0.0, 0.0)?, Some((0, 0)));
    assert_eq!(corridor.cell(B, 2.0, -1.0)?, Some((1, -2)));
    assert_eq!(corridor.cell(A, 10.0, 0.0)?, Some((0, 0)));
    assert_eq!(corridor.cell(A, 3.0, 2.0)?, Some((-2, 7)));
    assert_eq!(corridor.cell(C, 5.0, 0.0)?, Some((0, -6)));
    assert_eq!(corridor.cell(C, 0.0, 1.0)?, Some((1, -11)));
    assert_eq!(corridor.cell(0x0999_0000, 0.0, 0.0)?, None);
    Ok(())
}

#[test]
fn test_the_corridor_is_open_where_its_lines_are_and_walled_everywhere_else() -> Result<()> {
    let memory = FakeMemory::new();
    network(&memory);
    let corridor = rail::lay_out(&table_lines(&memory)?, B, None)?;
    let corridor = corridor.expect("a corridor");
    assert!(corridor.open((0, -3)) && corridor.open((2, -3)));
    assert!(!corridor.open((3, -3)));
    assert!(!corridor.open(corridor.cell(A, 4.0, 1.0)?.unwrap_or((0, 0))));
    assert!(!corridor.open((0, -40)) && !corridor.open((0, 40)));
    Ok(())
}

/// A door four cells along at the street's edge, as Lumiose's warp into zone 0x56 is.
#[test]
fn test_a_record_on_a_rail_covers_its_cells_in_the_corridor() -> Result<()> {
    let memory = FakeMemory::new();
    network(&memory);
    let corridor = rail::lay_out(&table_lines(&memory)?, A, None)?;
    let corridor = corridor.expect("a corridor");
    let mut record = Vec::new();
    record.extend(0x56u16.to_le_bytes());
    record.extend(0u16.to_le_bytes());
    record.extend(0x0703u16.to_le_bytes());
    record.extend(1u16.to_le_bytes());
    record.extend(words(&[A]));
    for value in [5i16, -3] {
        record.extend(value.to_le_bytes());
    }
    for value in [4u16, 1] {
        record.extend(value.to_le_bytes());
    }
    let (line, a, x, width, height) = rail::placed(&record)?;
    let cells = rail::cells_of(&corridor, line, a, x, width, height)?;
    assert_eq!(rail::spread(cells)?, ((3, -8), (1, 4)));
    Ok(())
}

/// Along a line is north where it runs forward, and an offset falling is east; a line laid
/// the other way round turns both.
#[test]
fn test_a_direction_on_a_line_is_a_bearing_in_the_corridor() -> Result<()> {
    let memory = FakeMemory::new();
    network(&memory);
    let corridor = rail::lay_out(&table_lines(&memory)?, B, None)?;
    let corridor = corridor.expect("a corridor");
    assert_eq!(corridor.bearing(B, 0.0, 1.0), Some(0.0));
    assert_eq!(
        crate::py::round(corridor.bearing(B, -1.0, 1.0).unwrap_or(0.0))?,
        45
    );
    assert_eq!(corridor.bearing(C, 0.0, 1.0), Some(180.0));
    assert_eq!(corridor.bearing(C, -1.0, 0.0), Some(270.0));
    assert_eq!(corridor.bearing(0x0999_0000, 0.0, 1.0), None);
    Ok(())
}

/// The middle of chunk (5, 4) in the world, which a section's points are placed from.
const MIDDLE: (i64, i64) = (5 * 720 + 360, 4 * 720 + 360);

fn put(out: &mut [u8], at: i64, bytes: &[u8]) {
    out[at as usize..at as usize + bytes.len()].copy_from_slice(bytes);
}

fn record(out: &mut Vec<u8>, tables: &[i64], kind: usize, i: i64, body: &[u8]) {
    let at = out.len() as i64;
    put(out, tables[kind] + 4 * i, &words(&[at]));
    out.extend_from_slice(body);
}

/// Chunk (5, 4)'s street section as the ROM keeps it: the chunk, 18.0, how many of each
/// kind of record and where each kind's offsets are, then the records, a line's or a track's
/// walls after it and its file pointer 0, a point or joint placed from the chunk's middle.
pub(super) fn street_section(
    lines: &[Line],
    points: &[(i64, (f64, f64))],
    joints: &[(i64, (f64, f64))],
) -> Vec<u8> {
    let plain: Vec<&Line> = lines.iter().filter(|line| !line.track).collect();
    let tracks: Vec<&Line> = lines.iter().filter(|line| line.track).collect();
    let counts = [
        plain.len() as i64,
        points.len() as i64,
        tracks.len() as i64,
        joints.len() as i64,
    ];
    let mut tables: Vec<i64> = Vec::new();
    let mut at = 0x40;
    for count in counts {
        tables.push(at);
        at += 4 * count;
    }
    let mut out = vec![0u8; at as usize];
    put(&mut out, 0, &words(&[0x0504]));
    put(&mut out, 4, &18.0f32.to_le_bytes());
    for (n, count) in counts.iter().enumerate() {
        put(&mut out, 8 + 2 * n as i64, &(*count as u16).to_le_bytes());
    }
    let mut header = tables.clone();
    header.push(at);
    put(&mut out, rail::SECTION_TABLES, &words(&header));

    for (kind, group) in [(rail::LINES, &plain), (rail::TRACKS, &tracks)] {
        for (i, line) in group.iter().enumerate() {
            let size = if line.track {
                rail::TRACK_WALLS + 4
            } else {
                rail::LINE_HEAD
            };
            let mut head = vec![0u8; size as usize];
            put(
                &mut head,
                0,
                &words(&[line.id, 0x504, line.start, line.end, line.way]),
            );
            put(
                &mut head,
                rail::LINE_ALONG,
                &(line.along as u16).to_le_bytes(),
            );
            put(
                &mut head,
                rail::LINE_ALONG + 2,
                &(line.across as u16).to_le_bytes(),
            );
            let wall = out.len() as i64 + size;
            put(
                &mut head,
                if line.track {
                    rail::TRACK_WALLS
                } else {
                    rail::LINE_WALLS
                },
                &words(&[wall]),
            );
            head.extend(words(&line.walls));
            record(&mut out, &tables, kind as usize, i as i64, &head);
        }
    }
    for (kind, group, place) in [
        (rail::POINTS, points, rail::POINT_PLACE),
        (rail::JOINTS, joints, rail::JOINT_PLACE),
    ] {
        for (i, &(point, (x, z))) in group.iter().enumerate() {
            let mut body = vec![0u8; place as usize + 12];
            put(&mut body, 0, &words(&[point, 0x504]));
            put(
                &mut body,
                place,
                &((x - MIDDLE.0 as f64) as f32).to_le_bytes(),
            );
            put(&mut body, place + 4, &0.0f32.to_le_bytes());
            put(
                &mut body,
                place + 8,
                &((z - MIDDLE.1 as f64) as f32).to_le_bytes(),
            );
            record(&mut out, &tables, kind as usize, i as i64, &body);
        }
    }
    out
}

/// Spikes 106 and 107: section 4 of a map chunk's file is its streets, each line's record as
/// the linear heap has it but for the file pointer, its walls counted from the section's start;
/// tracks, points and joints each have a table of their own, the points and joints placed from
/// the chunk's middle.
#[test]
fn test_a_chunks_street_section_holds_its_lines_tracks_and_joints() -> Result<()> {
    let c = Line::new(C, P3, P2, 5, ACROSS, walls(5, &[]));
    let a = Line {
        way: rail::WAY_NORTH,
        ..Line::new(A, P0, P1, 10, ACROSS, walls(10, &[(4, 1)]))
    };
    let track = Line {
        way: rail::WAY_WEST,
        track: true,
        ..Line::new(0x03047A5B, P1, 0x030439B4, 4, ACROSS, walls(4, &[]))
    };
    let network = rail::section_streets(&street_section(
        &[c.clone(), a.clone(), track.clone()],
        &[(P1, (3950.0, 3300.5))],
        &[(0x030439B4, (3950.0, 3228.5))],
    ))?;
    let lines: IndexMap<i64, Line> = [(C, c), (A, a), (track.id, track)].into_iter().collect();
    assert_eq!(network.lines, lines);
    let places: IndexMap<i64, (f64, f64)> =
        [(P1, (3950.0, 3300.5)), (0x030439B4, (3950.0, 3228.5))]
            .into_iter()
            .collect();
    assert_eq!(network.places, places);
    assert_eq!(
        rail::section_streets(&street_section(&[], &[], &[]))?,
        Network::default()
    );
    assert_eq!(rail::section_streets(b"GR\x06\x00")?, Network::default());
    Ok(())
}

/// Spike 107: the game measures a place on a line by the line's way, as if the line ran
/// that way of the world. Through a warp into zone 0x44 the player stood at (-5, 8) on a line
/// measured north: 5 cells from its start, 8 to its right. On a line measured east or west the
/// two numbers swap, and a track is measured from the joint at its end.
#[test]
fn test_a_place_is_measured_by_its_lines_way() {
    let line = |way: i64, track: bool| Line {
        way,
        track,
        ..Line::new(A, P0, P1, 20, ACROSS, walls(20, &[]))
    };
    assert_eq!(line(rail::WAY_SOUTH, false).upright(5.0, 2.0), (5.0, 2.0));
    assert_eq!(line(rail::WAY_NORTH, false).upright(-5.0, 8.0), (5.0, -8.0));
    assert_eq!(line(rail::WAY_EAST, false).upright(-9.0, 7.0), (7.0, 9.0));
    assert_eq!(line(rail::WAY_WEST, false).upright(-9.0, 7.0), (-7.0, -9.0));
    assert_eq!(line(rail::WAY_WEST, true).upright(2.0, 0.0), (20.0, -2.0));
    assert_eq!(
        line(rail::WAY_NORTH, false).direction(-1.0, 0.0),
        (1.0, 0.0)
    );
    assert_eq!(
        line(rail::WAY_SOUTH, true).direction(1.0, 0.0),
        (-1.0, -0.0)
    );
}

/// Spike 107: zone 0x44's warp into zone 0x3f at -4, 1 by -2 from 2, on a line measured
/// north, is on row 4 and across -2 and -1: the wall row before the street's first open one.
#[test]
fn test_a_record_on_a_line_measured_another_way_covers_its_cells() -> Result<()> {
    let north = Line {
        way: rail::WAY_NORTH,
        ..Line::new(A, P0, P1, 10, ACROSS, walls(10, &[]))
    };
    let lines: IndexMap<i64, Line> = [(A, north)].into_iter().collect();
    let corridor = rail::lay_out(&lines, A, None)?;
    let corridor = corridor.expect("a corridor");
    let mut record = Vec::new();
    record.extend(0x3Fu16.to_le_bytes());
    record.extend(0u16.to_le_bytes());
    record.extend(0x0300u16.to_le_bytes());
    record.extend(1u16.to_le_bytes());
    record.extend(words(&[A]));
    for value in [-4i16, 2, -1, -2] {
        record.extend(value.to_le_bytes());
    }
    assert_eq!(rail::placed(&record)?, (A, -4, 2, -1, -2));
    let (line, a, x, width, height) = rail::placed(&record)?;
    let cells: HashSet<(i64, i64)> = rail::cells_of(&corridor, line, a, x, width, height)?
        .into_iter()
        .collect();
    let expected: HashSet<(i64, i64)> = [
        corridor.cell(A, 4.0, -2.0)?.expect("a cell"),
        corridor.cell(A, 4.0, -1.0)?.expect("a cell"),
    ]
    .into_iter()
    .collect();
    assert_eq!(cells, expected);
    Ok(())
}

/// Spike 107: north of the blackout a street ends in a joint of three or more, and each goes
/// on from it the way the world has it, a quarter turn taken to the nearest. A turn in the
/// world towards a line's rising offset is to the left, so the street towards the world's -x
/// from a stem walked towards -z is laid west, and the other east.
#[test]
fn test_at_a_joint_each_street_is_laid_the_quarter_it_leaves_the_joint_by() -> Result<()> {
    let (stem, left, right) = (0x01, 0x02, 0x03);
    let (joint, south, west, east) = (0x10, 0x11, 0x12, 0x13);
    let lines: IndexMap<i64, Line> = [
        Line::new(stem, south, joint, 6, ACROSS, walls(6, &[])),
        Line::new(left, joint, west, 4, ACROSS, walls(4, &[])),
        Line::new(right, east, joint, 5, ACROSS, walls(5, &[])),
    ]
    .into_iter()
    .map(|line| (line.id, line))
    .collect();
    let places: IndexMap<i64, (f64, f64)> = [
        (south, (0.0, 108.0)),
        (joint, (0.0, 0.0)),
        (west, (-72.0, 3.0)),
        (east, (90.0, -2.0)),
    ]
    .into_iter()
    .collect();
    let corridor = rail::lay_out(&lines, stem, Some(&places))?;
    let corridor = corridor.expect("a corridor");
    assert_eq!(corridor.cell(stem, 6.0, 0.0)?, Some((0, -6)));
    assert_eq!(corridor.cell(left, 4.0, 0.0)?, Some((-4, -6)));
    assert_eq!(corridor.cell(right, 0.0, 0.0)?, Some((5, -6)));
    assert_eq!(corridor.bearing(left, 0.0, 1.0), Some(270.0));
    assert!(corridor.open((-3, -6)) && corridor.open((4, -6)));
    let unplaced = rail::lay_out(&lines, stem, None)?;
    let unplaced = unplaced.expect("a corridor");
    assert!(!unplaced.holds(left) && !unplaced.holds(right));
    Ok(())
}

/// Spike A: the alley off North Boulevard ends in a joint of one track at each end, and one
/// opens onto Magenta Plaza, walked on the grid. Each end no other line meets is its cell in
/// the corridor and the grid's cell the world has it in; a point two lines meet is none.
#[test]
fn test_a_streets_dead_ends_are_where_the_corridor_and_the_world_have_them() -> Result<()> {
    let (stem, track) = (0x01, 0x02);
    let (south, joint, plaza) = (0x10, 0x11, 0x12);
    let network = rail::Network {
        lines: IndexMap::from([
            (
                stem,
                Line::new(stem, south, joint, 6, ACROSS, walls(6, &[])),
            ),
            (
                track,
                Line {
                    track: true,
                    ..Line::new(track, plaza, joint, 1, ACROSS, walls(1, &[]))
                },
            ),
        ]),
        places: IndexMap::from([
            (south, (9.0, 117.0)),
            (joint, (9.0, 9.0)),
            (plaza, (9.0, -9.0)),
        ]),
        ..Default::default()
    };
    let lines = network.lines.clone();
    let places = network.places.clone();
    let rails = rail::Rails::new(
        FakeMemory::new(),
        Some(std::rc::Rc::new(move |_files: &[i64]| Ok(network.clone()))),
        rail::Records::default(),
        crate::games::gen6::field::camera::ORAS,
    );
    rails.lanes(&[1], false, 0)?;
    let corridor = rail::lay_out(&lines, stem, Some(&places))?.expect("a corridor");
    let mut ends = rails.ends(&corridor)?;
    ends.sort();
    assert_eq!(ends, [(south, (0, 0), (0, 6)), (plaza, (0, -7), (0, -1))]);
    Ok(())
}

/// x_tower_of_mastery (spike 248): the Tower of Mastery's ramp winds twice round its ring,
/// each lap over the one before in the grid and 360 above it. Where a street stands off the
/// ground every street is a floor, the laps two, rising a cell at a time along each; where
/// every street lies on the ground, as Route 7's does, none is.
#[test]
fn test_streets_are_floors_of_their_own_where_one_stands_off_the_ground() -> Result<()> {
    let (first, second, foot, top, high_foot, high_top) = (0x01, 0x02, 0x11, 0x12, 0x21, 0x22);
    let places: IndexMap<i64, (f64, f64)> = IndexMap::from([
        (foot, (9.0, 9.0)),
        (top, (9.0 + 18.0 * 8.0, 9.0)),
        (high_foot, (9.0, 9.0)),
        (high_top, (9.0 + 18.0 * 8.0, 9.0)),
    ]);
    let sides: IndexMap<i64, (f64, f64)> =
        places.keys().map(|&point| (point, (0.0, 18.0))).collect();
    let lines: IndexMap<i64, Line> = IndexMap::from([
        (
            first,
            Line {
                way: rail::WAY_EAST,
                ..Line::new(first, foot, top, 8, 1, vec![0x20; 9])
            },
        ),
        (
            second,
            Line {
                way: rail::WAY_EAST,
                ..Line::new(second, high_foot, high_top, 8, 1, vec![0x20; 9])
            },
        ),
    ]);
    let heights: IndexMap<i64, f64> = IndexMap::from([
        (foot, 0.0),
        (top, 80.0),
        (high_foot, 360.0),
        (high_top, 440.0),
    ]);

    let ground = |_cell: (i64, i64)| -> Result<Vec<f64>> { Ok(vec![0.0]) };

    let network = |heights: &IndexMap<i64, f64>| Network {
        lines: lines.clone(),
        places: places.clone(),
        cameras: 2,
        heights: heights.clone(),
        sides: sides.clone(),
    };
    let floors = rail::street_floors(&network(&heights), &ground)?;
    let [(lap, ramp), (over, _)] = &floors[..] else {
        panic!("two floors, not {}", floors.len());
    };
    assert!(ramp);
    assert_eq!(
        (0..9).map(|x| lap[&(x, 0)]).collect::<Vec<f64>>(),
        (0..9).map(|x| 10.0 * x as f64).collect::<Vec<f64>>()
    );
    let raised: IndexMap<(i64, i64), f64> = lap
        .iter()
        .map(|(&cell, &height)| (cell, height + 360.0))
        .collect();
    assert_eq!(over, &raised);
    let flat: IndexMap<i64, f64> = heights.keys().map(|&point| (point, 0.0)).collect();
    assert!(rail::street_floors(&network(&flat), &ground)?.is_empty());
    Ok(())
}
