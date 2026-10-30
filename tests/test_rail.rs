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
    let corridor = rail::lay_out(&table_lines(&memory)?, B, None, false)?;
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
    let corridor = rail::lay_out(&table_lines(&memory)?, B, None, false)?;
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
    let corridor = rail::lay_out(&table_lines(&memory)?, A, None, false)?;
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
    let corridor = rail::lay_out(&table_lines(&memory)?, B, None, false)?;
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
    let corridor = rail::lay_out(&lines, A, None, false)?;
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
    let corridor = rail::lay_out(&lines, stem, Some(&places), false)?;
    let corridor = corridor.expect("a corridor");
    assert_eq!(corridor.cell(stem, 6.0, 0.0)?, Some((0, -6)));
    assert_eq!(corridor.cell(left, 4.0, 0.0)?, Some((-4, -6)));
    assert_eq!(corridor.cell(right, 0.0, 0.0)?, Some((5, -6)));
    assert_eq!(corridor.bearing(left, 0.0, 1.0), Some(270.0));
    assert!(corridor.open((-3, -6)) && corridor.open((4, -6)));
    let unplaced = rail::lay_out(&lines, stem, None, false)?;
    let unplaced = unplaced.expect("a corridor");
    assert!(!unplaced.holds(left) && !unplaced.holds(right));
    Ok(())
}

/// Spike 251: the alley off North Boulevard ends in a joint of one track at each end, and one
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
        Vec::new(),
    );
    rails.lanes(&[1], false, 0)?;
    let corridor = rail::lay_out(&lines, stem, Some(&places), false)?.expect("a corridor");
    let mut ends = rails.ends(&corridor)?;
    ends.sort();
    assert_eq!(ends, [(south, (0, 0), (0, 6)), (plaza, (0, -7), (0, -1))]);
    Ok(())
}

const CLIMB: i64 = 0x01;
const TRACK: i64 = 0x02;
const ON: i64 = 0x03;
const BEYOND: i64 = 0x04;
const ALONE: i64 = 0x05;
const FOOT: i64 = 0x10;
const TOP: i64 = 0x11;
const JOINT: i64 = 0x12;
const MEETS: i64 = 0x13;
const FAR: i64 = 0x14;
const NEXT: i64 = 0x15;
const LAST: i64 = 0x16;
const AWAY: i64 = 0x17;
const END: i64 = 0x18;

/// Anistar City Gym's streets in small (x_gym_7, spike A): a climb measured north, its ends at one
/// spot 90 apart in height, as the lobby's is; a track measured west from its top to a joint; a
/// line measured south from a point of its own at the joint's spot and height, ending a cell from
/// where another goes on south; and a street no other meets. `folded()` with the top at (9, 9).
fn folded(top: (f64, f64)) -> Network {
    let lines = [
        Line {
            way: rail::WAY_NORTH,
            ..Line::new(CLIMB, FOOT, TOP, 5, ACROSS, walls(5, &[]))
        },
        Line {
            way: rail::WAY_WEST,
            track: true,
            ..Line::new(TRACK, TOP, JOINT, 3, ACROSS, walls(3, &[]))
        },
        Line::new(ON, MEETS, FAR, 4, ACROSS, walls(4, &[])),
        Line::new(BEYOND, NEXT, LAST, 2, ACROSS, walls(2, &[])),
        Line::new(ALONE, AWAY, END, 2, ACROSS, walls(2, &[])),
    ];
    let places = IndexMap::from([
        (FOOT, (9.0, 9.0)),
        (TOP, top),
        (JOINT, (63.0, 9.0)),
        (MEETS, (63.0, 9.0)),
        (FAR, (63.0, 81.0)),
        (NEXT, (63.0, 99.0)),
        (LAST, (63.0, 135.0)),
        (AWAY, (900.0, 9.0)),
        (END, (900.0, 45.0)),
    ]);
    let heights = IndexMap::from([
        (FOOT, 0.0),
        (TOP, 90.0),
        (JOINT, 90.0),
        (MEETS, 90.0),
        (FAR, 90.0),
        (NEXT, 90.0),
        (LAST, 90.0),
        (AWAY, 0.0),
        (END, 0.0),
    ]);
    Network {
        lines: lines.into_iter().map(|line| (line.id, line)).collect(),
        places,
        cameras: 1,
        heights,
        ..Network::default()
    }
}

/// Spike A: Anistar City Gym's camera stays looking north, and up walked the player a cell north
/// along every street of x_gym_7, laid along its own way and a track against it; two points at one
/// spot and height are one, as the lobby's street and the landing's track are; and a street joined
/// to none is laid apart, with a part of its own, even a cell on from a dead end at its height: so
/// joined off the first teleporter's landing, P's way south met a wall (Khai, 2026-10-30). The
/// climb's two ends at one spot are no join: they are 90 apart in height.
#[test]
fn test_a_folded_maps_streets_are_laid_by_the_keys() -> Result<()> {
    let network = folded((9.0, 9.0));
    assert!(network.folded());
    let corridor = rail::lay_keyed(&network)?;
    let foot = corridor.cell(CLIMB, 0.0, 0.0)?.expect("the climb is laid");
    let at = |line: i64, along: i64| -> Result<Option<(i64, i64)>> {
        Ok(corridor
            .cell(line, along as f64, 0.0)?
            .map(|cell| (cell.0 - foot.0, cell.1 - foot.1)))
    };
    assert_eq!(at(CLIMB, 5)?, Some((0, -5)));
    assert!(at(TRACK, 0)? == Some((0, -5)) && at(TRACK, 3)? == Some((3, -5)));
    assert!(at(ON, 0)? == Some((3, -5)) && at(ON, 4)? == Some((3, -1)));
    for apart in [BEYOND, ALONE] {
        let cell = corridor.cell(apart, 0.0, 0.0)?;
        assert!(
            cell.is_some_and(|cell| cell.0 - foot.0 >= 3 + ACROSS / 2 + rail::APART),
            "{apart}"
        );
        assert_ne!(corridor.part(cell.unwrap()), corridor.part(foot), "{apart}");
    }
    Ok(())
}

/// Spike A: in Anistar City Gym pa3ds said "20 behind 1 left" and "12 back, hold right" from a
/// camera that never turns. A folded map is never the grid's ground, and its corridor gives no way
/// up walks, so a way is said by the compass; with the climb's ends apart in the world the map is
/// not folded, and the street is walked on the grid.
#[test]
fn test_a_folded_map_is_walked_as_a_corridor_said_by_the_compass() -> Result<()> {
    use crate::games::gen6::field::{camera, mmodel};
    let mut head = vec![0u8; mmodel::HEAD as usize];
    head[mmodel::RAIL_LINE as usize..mmodel::RAIL_LINE as usize + 4]
        .copy_from_slice(&(ON as u32).to_le_bytes());
    head[mmodel::RAIL_ALONG as usize..mmodel::RAIL_ALONG as usize + 4]
        .copy_from_slice(&2.0f32.to_le_bytes());
    let rails = rail::Rails::new(
        FakeMemory::new(),
        Some(std::rc::Rc::new(|_files: &[i64]| Ok(folded((9.0, 9.0))))),
        rail::Records::default(),
        camera::ORAS,
        Vec::new(),
    );
    assert!(!rails.fixed(&[1], ON)?);
    let on = rails.poll(&head, &[1])?;
    let corridor = rails.corridor.borrow().clone();
    assert!(on.is_some_and(|on| on.1.is_none()) && corridor.is_some());
    assert_eq!(Some(on.unwrap().0), corridor.unwrap().cell(ON, 2.0, 0.0)?);
    let apart = rail::Rails::new(
        FakeMemory::new(),
        Some(std::rc::Rc::new(|_files: &[i64]| Ok(folded((9.0, -81.0))))),
        rail::Records::default(),
        camera::ORAS,
        Vec::new(),
    );
    assert!(apart.fixed(&[1], ON)?);
    Ok(())
}

/// Spike A: read from their starts, each of Anistar City Gym's tracks that widens toward one end
/// widened away from its joint and closed at it; read from the joint, its walkways meet side to
/// side, and a walk over them reaches every person in the Gym.
#[test]
fn test_on_a_folded_map_a_tracks_walls_run_from_its_joint() -> Result<()> {
    let mut network = folded((9.0, 9.0));
    let mut joint_side = vec![0x20; ACROSS as usize];
    joint_side.extend(vec![0x21; (ACROSS * 3) as usize]);
    let track = network.lines[&TRACK].clone();
    network.lines.insert(
        TRACK,
        Line {
            walls: joint_side.into(),
            ..track
        },
    );
    let corridor = rail::lay_keyed(&network)?;
    let laid = corridor.line(TRACK).expect("the track is laid");
    assert_eq!(
        (0..4)
            .map(|row| laid.open(row, ACROSS / 2))
            .collect::<Vec<_>>(),
        [false, false, false, true]
    );
    Ok(())
}

/// Spike A: DllUSGymEsper.cro holds 19 places where a teleport starts and then the 19 it puts the
/// player, in the same order, each a street, a word, and along and across in world units: the
/// lobby's carpet at 72 along its street, 4 cells, and the joint at the Gym's middle sending the
/// player 54 across, 3 cells, onto line 0x101df4b.
#[test]
fn test_a_gym_modules_teleports_are_the_first_half_of_its_places_and_where_each_lands() -> Result<()>
{
    use crate::games::gen6::rom::maps;
    let entries: [(u32, u32, f32, f32); 4] = [
        (0x0101C50A, 1, 72.0, 0.0),
        (0x01010122, 1, 0.0, 0.0),
        (0x01013BF7, 1, 0.0, 0.0),
        (0x0101DF4B, 2, 0.0, -54.0),
    ];
    let mut module: Vec<u8> = b"\x01\x00\x01\x01".repeat(4);
    for (line, way, along, across) in entries {
        module.extend(line.to_le_bytes());
        module.extend(way.to_le_bytes());
        module.extend(along.to_le_bytes());
        module.extend(across.to_le_bytes());
    }
    module.extend([0u8; 16]);
    assert_eq!(
        maps::rail_teleports(&module)?,
        [
            rail::Teleport {
                start: (0x0101C50A, 4.0, 0.0),
                lands: (0x01013BF7, 0.0, 0.0)
            },
            rail::Teleport {
                start: (0x01010122, 0.0, 0.0),
                lands: (0x0101DF4B, 0.0, -3.0)
            },
        ]
    );
    Ok(())
}

/// Khai, 2026-10-30: stepping back onto the landing of Anistar City Gym's first teleporter put the
/// player on its start, three times, though the module lists it one way. Each teleport is listed
/// as a teleporter where it starts and another where it lands, sending back.
///
/// Python hands `FieldReader._rail_teleporters` a stand-in for the rails; here the function both
/// call is handed their teleports and heights.
#[test]
fn test_a_gym_modules_teleport_is_a_teleporter_at_each_end() -> Result<()> {
    use crate::games::gen6::field::reader;
    let corridor = rail::lay_keyed(&folded((9.0, 9.0)))?;
    let teleport = rail::Teleport {
        start: (ON, 1.0, 0.0),
        lands: (ALONE, 1.0, 0.0),
    };
    let listed = reader::rail_teleporters(&[teleport], &|_place| Some(90.0), &corridor)?;
    let start = corridor.at(teleport.start.0, teleport.start.1, teleport.start.2)?;
    let lands = corridor.at(teleport.lands.0, teleport.lands.1, teleport.lands.2)?;
    assert_eq!(
        listed
            .iter()
            .map(|item| (Some(item.cell), item.arrival))
            .collect::<Vec<_>>(),
        [(start, lands), (lands, start)]
    );
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
