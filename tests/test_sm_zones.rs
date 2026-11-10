//! Sun's zones out of the ROM: every kind of record of an area's event data, and the zone table's
//! kind of place, laid out as spike 115 read Route 1 and Iki Town.

use indexmap::IndexMap;

use super::fake_rom::{
    GarcFile, MessageLine, TmpDir, garc_with, lz11_literals, message_file, rom_image,
};
use crate::core::rom::Rom;
use crate::games::sm::text::{SunText, TEXT_ARCHIVES};
use crate::games::sm::zones::{self, Area, Shape, SunZones};

const ROUTE: i64 = 0x0;
const NORTH: i64 = 0x1;
const TOWN: i64 = 0xA;
const AREA: i64 = 0;

fn u32le(value: i64) -> Vec<u8> {
    (value as u32).to_le_bytes().to_vec()
}

fn f32s(values: &[f64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&value| (value as f32).to_le_bytes())
        .collect()
}

/// An 'ED' of 16 sections, each with the parts given, a zone each, or empty ones.
fn event_data(parts: &IndexMap<i64, Vec<Vec<u8>>>) -> Vec<u8> {
    let zone_count = parts.values().map(Vec::len).max().unwrap();
    let mut sections: Vec<Vec<u8>> = Vec::new();
    for number in 0..16 {
        let given = parts
            .get(&number)
            .cloned()
            .unwrap_or_else(|| vec![u32le(0); zone_count]);
        let head = 4 + 4 * (zone_count as i64 + 1);
        let mut offsets = vec![head];
        for part in &given {
            offsets.push(offsets[offsets.len() - 1] + part.len() as i64);
        }
        let mut section = b"EM".to_vec();
        section.extend((zone_count as u16).to_le_bytes());
        for offset in offsets {
            section.extend(u32le(offset));
        }
        section.extend(given.concat());
        sections.push(section);
    }
    let mut at = 4 + 4 * (sections.len() as i64 + 1);
    let mut starts: Vec<i64> = Vec::new();
    for section in &sections {
        starts.push(at);
        at += section.len() as i64;
    }
    let mut head = b"ED".to_vec();
    head.extend((sections.len() as u16).to_le_bytes());
    for start in starts {
        head.extend(u32le(start));
    }
    head.extend(u32le(at));
    head.extend(sections.concat());
    head
}

/// A part: its count, its records, and each record's shape after them, the record's field at
/// the given offset pointing at it.
fn part(mut records: Vec<(Vec<u8>, Option<Vec<u8>>, Option<i64>)>) -> Vec<u8> {
    let body_length: usize = records.iter().map(|(record, _, _)| record.len()).sum();
    let mut tail: Vec<u8> = Vec::new();
    for (record, shape, at) in records.iter_mut() {
        if let (Some(shape), Some(at)) = (shape.as_ref(), *at) {
            let pointer = 4 + body_length + tail.len();
            record[at as usize..at as usize + 4].copy_from_slice(&u32le(pointer as i64));
            tail.extend(shape);
        }
    }
    let mut part = u32le(records.len() as i64);
    for (record, _shape, _at) in &records {
        part.extend(record);
    }
    part.extend(tail);
    part
}

fn record(
    size: i64,
    what: i64,
    x: f64,
    z: f64,
    number: i64,
    fields: &[(i64, &str, i64)],
) -> Vec<u8> {
    let mut data = vec![0u8; size as usize];
    data[0..4].copy_from_slice(&u32le(what));
    data[4..16].copy_from_slice(&f32s(&[x, 0.0, z]));
    let at = zones::NUMBER as usize;
    data[at..at + 4].copy_from_slice(&u32le(number));
    for &(offset, form, value) in fields {
        let bytes = match form {
            "<H" => (value as u16).to_le_bytes().to_vec(),
            "<I" => u32le(value),
            _ => panic!("no form {form}"),
        };
        data[offset as usize..offset as usize + bytes.len()].copy_from_slice(&bytes);
    }
    data
}

fn r#box(x: f64, z: f64, width: f64, depth: f64) -> Vec<u8> {
    [
        u32le(1),
        u32le(zones::BOX),
        f32s(&[x, 0.0, z, width, depth]),
    ]
    .concat()
}

fn line(ax: f64, az: f64, bx: f64, bz: f64) -> Vec<u8> {
    [
        u32le(1),
        u32le(zones::LINE),
        f32s(&[ax, 0.0, az, bx, 0.0, bz, 300.0]),
    ]
    .concat()
}

fn warp(x: f64, z: f64, zone: i64, to: i64, kind: i64) -> Vec<u8> {
    record(
        zones::WARP_RECORD,
        3,
        x,
        z,
        0x12E,
        &[
            (zones::WARP_ZONE, "<H", zone),
            (zones::WARP_TO, "<H", to),
            (zones::WARP_TYPE, "<I", kind),
        ],
    )
}

/// Route 1's area as spike 115 read it, cut down: a zone with the lab's door and a Potion and
/// a hidden Pearl, and the zone to the north with the edge to Iki Town, a person and a
/// trainer.
fn route_1() -> SunZones {
    let door = warp(29442.5, 21798.7, ROUTE, 0x25, zones::WARP_DOOR);
    let edge = warp(18497.5, 14483.25, NORTH, TOWN, zones::WARP_EDGE);
    let potion = record(
        zones::ITEM_RECORD,
        0xA,
        22357.5,
        20872.5,
        0xCE,
        &[
            (zones::ITEM_ITEM, "<I", 17),
            (zones::ITEM_LIES, "<I", zones::ITEM_BALL),
        ],
    );
    let pearl = record(
        zones::ITEM_RECORD,
        0xA,
        29241.3,
        23196.0,
        0xCA,
        &[
            (zones::ITEM_ITEM, "<I", 88),
            (zones::ITEM_LIES, "<I", zones::HIDDEN_ITEM),
        ],
    );
    let person = record(
        zones::PERSON_RECORD,
        1,
        22197.0,
        15793.0,
        0x18,
        &[
            (zones::PERSON_TALK, "<I", 0x2AA),
            (zones::PERSON_MODEL, "<I", 0x43),
        ],
    );
    let sign = record(
        zones::THING_RECORD,
        2,
        24588.3,
        20168.2,
        0x385,
        &[
            (zones::THING_CODE, "<I", 0xB),
            (zones::THING_KIND, "<I", 0x12),
        ],
    );
    let to_town = line(18651.75, 14483.25, 18342.25, 14483.25);
    let data = event_data(&IndexMap::from([
        (
            zones::WARPS,
            vec![
                part(vec![(
                    door,
                    Some(r#box(29442.5, 21798.7, 70.0, 200.0)),
                    Some(zones::WARP_SHAPE),
                )]),
                part(vec![(edge, Some(to_town), Some(zones::WARP_SHAPE))]),
            ],
        ),
        (
            zones::ITEMS,
            vec![
                part(vec![
                    (
                        potion,
                        Some(r#box(22357.5, 20872.5, 60.0, 150.0)),
                        Some(zones::ITEM_SHAPE),
                    ),
                    (
                        pearl,
                        Some(r#box(29241.3, 23196.0, 60.0, 150.0)),
                        Some(zones::ITEM_SHAPE),
                    ),
                ]),
                part(vec![]),
            ],
        ),
        (
            zones::PEOPLE,
            vec![part(vec![]), part(vec![(person, None, None)])],
        ),
        (
            zones::THINGS,
            vec![
                part(vec![(
                    sign,
                    Some(
                        [
                            u32le(1),
                            u32le(zones::CIRCLE),
                            f32s(&[24636.7, 0.0, 20255.7, 0.0, 0.0, 0.0, 1.0, 80.0]),
                        ]
                        .concat(),
                    ),
                    Some(zones::THING_SHAPE),
                )]),
                part(vec![]),
            ],
        ),
    ]));
    let mut rows = vec![0u8; (zones::ZONE_ROW * 0x30) as usize];
    for (zone, kind) in [
        (ROUTE, zones::OUTDOORS),
        (NORTH, zones::OUTDOORS),
        (TOWN, zones::OUTDOORS),
        (0x25, zones::INDOORS),
    ] {
        let at = (zone * zones::ZONE_ROW + zones::ROW_KIND) as usize;
        rows[at..at + 2].copy_from_slice(&(kind as u16).to_le_bytes());
    }
    let area = zones::area(&data).unwrap();
    SunZones {
        rows,
        places: SunText {
            files: IndexMap::new(),
        },
        areas: IndexMap::from([(AREA, area)]),
    }
}

#[test]
fn test_a_warp_keeps_its_type_and_says_where_it_leads() {
    let rom = route_1();
    let door = rom
        .record(AREA, zones::WARPS, 29442.5, 21798.7)
        .unwrap()
        .unwrap();
    let edge = rom
        .record(AREA, zones::WARPS, 18497.5, 14483.25)
        .unwrap()
        .unwrap();
    assert_eq!(
        (door.kind, door.leads_to, door.number),
        (zones::WARP_DOOR, Some(0x25), 0x12E)
    );
    assert_eq!(
        (edge.kind, edge.zone, edge.leads_to),
        (zones::WARP_EDGE, Some(NORTH), Some(TOWN))
    );
    assert_eq!(
        edge.shape,
        Some(Shape {
            west: 18342.25,
            north: 14483.25,
            east: 18651.75,
            south: 14483.25,
        })
    );
}

#[test]
fn test_the_zone_table_says_which_zones_are_outdoors() {
    let rom = route_1();
    assert!(rom.outdoors(ROUTE).unwrap() && rom.outdoors(TOWN).unwrap());
    assert!(!rom.outdoors(0x25).unwrap());
    assert_eq!(rom.zone_kind(0x25).unwrap(), Some(zones::INDOORS));
    assert_eq!(rom.zone_kind(0x400).unwrap(), None);
}

#[test]
fn test_items_are_read_with_the_item_and_how_they_lie() {
    let rom = route_1();
    let potion = rom
        .record(AREA, zones::ITEMS, 22357.5, 20872.5)
        .unwrap()
        .unwrap();
    let pearl = rom
        .record(AREA, zones::ITEMS, 29241.3, 23196.0)
        .unwrap()
        .unwrap();
    assert_eq!(
        (potion.code, potion.kind, potion.zone),
        (17, zones::ITEM_BALL, Some(ROUTE))
    );
    assert_eq!(
        potion.shape,
        Some(Shape {
            west: 22327.5,
            north: 20797.5,
            east: 22387.5,
            south: 20947.5,
        })
    );
    assert_eq!((pearl.code, pearl.kind), (88, zones::HIDDEN_ITEM));
}

#[test]
fn test_people_are_read_with_their_models_in_their_part_s_zone() {
    let rom = route_1();
    let person = rom
        .record(AREA, zones::PEOPLE, 22197.0, 15793.0)
        .unwrap()
        .unwrap();
    assert_eq!(
        (person.code, person.script, person.zone),
        (0x43, 0x2AA, Some(NORTH))
    );
    assert_eq!(
        rom.record(AREA, zones::PEOPLE, 25608.0, 13975.0).unwrap(),
        None
    );
}

#[test]
fn test_a_thing_keeps_the_kind_that_says_it_is_a_sign() {
    let sign = route_1()
        .record(AREA, zones::THINGS, 24588.3, 20168.2)
        .unwrap()
        .unwrap();
    assert!(zones::SIGNS.contains(&sign.kind));
    assert_eq!((sign.code, sign.script, sign.zone), (0xB, 0, Some(ROUTE)));
}

#[test]
fn test_an_area_built_with_its_warps_and_things_alone_has_no_people() {
    let rom = SunZones {
        rows: vec![0; zones::ZONE_ROW as usize],
        places: SunText {
            files: IndexMap::new(),
        },
        areas: IndexMap::from([(
            AREA,
            Area {
                warps: vec![],
                things: vec![],
                ..Default::default()
            },
        )]),
    };
    assert_eq!(
        rom.record(AREA, zones::PEOPLE, 22197.0, 15793.0).unwrap(),
        None
    );
    assert_eq!(rom.record(1, zones::ITEMS, 0.0, 0.0).unwrap(), None);
}

/// Area 0's events as they lie, area 1's packed in LZ11, and area 2's first file not events,
/// where the reading stops. The event archive is never read whole: Sun's is 332 MB.
///
/// The Python also had `Rom.read` assert it was never asked for the event archive. A Rust `Rom`
/// cannot be patched, so that guard is not here: `load` reaching the events through `read_part`
/// alone is only read off its source.
#[test]
fn test_the_rom_s_areas_are_read_file_by_file_from_their_events() {
    let events = |zone: i64| -> Vec<u8> {
        let door = warp(100.0, 200.0, zone, 0x25, zones::WARP_DOOR);
        event_data(&IndexMap::from([(
            zones::WARPS,
            vec![part(vec![(
                door,
                Some(r#box(100.0, 200.0, 70.0, 200.0)),
                Some(zones::WARP_SHAPE),
            )])],
        )]))
    };

    let others = || -> Vec<GarcFile> {
        (0..zones::AREA_FILES - 1)
            .map(|_| GarcFile::from(b"other".as_slice()))
            .collect()
    };
    let mut files: Vec<GarcFile> = vec![events(ROUTE).into()];
    files.extend(others());
    files.push(lz11_literals(&events(TOWN)).into());
    files.extend(others());
    files.push(b"not events".as_slice().into());
    files.extend(others());
    let archive = garc_with(&files, 0x0600);

    let mut rows = vec![0u8; (zones::ZONE_ROW * 0x10) as usize];
    let at = (TOWN * zones::ZONE_ROW + zones::ROW_PLACE) as usize;
    rows[at..at + 2].copy_from_slice(&1u16.to_le_bytes());
    let mut names: Vec<GarcFile> = (0..zones::PLACE_NAMES)
        .map(|_| GarcFile::from(Vec::new()))
        .collect();
    names.push(message_file(&[MessageLine::from("Route 1"), MessageLine::from("Iki Town")]).into());
    let text = garc_with(&names, 0x0600);
    let english = TEXT_ARCHIVES
        .iter()
        .find(|(language, _)| *language == "en")
        .map(|&(_, archive)| archive)
        .unwrap();
    let tmp_path = TmpDir::new();
    let path = tmp_path.path().join("sun.3ds");
    let zone_table = garc_with(&[lz11_literals(&rows).into()], 0x0600);
    std::fs::write(
        &path,
        rom_image(
            &[
                (zones::ZONE_TABLE, &zone_table),
                (english, &text),
                (zones::EVENTS, &archive),
            ],
            0x1,
        ),
    )
    .unwrap();

    let rom = Rom::new(&path).unwrap();
    let loaded = SunZones::load(&rom, "en").unwrap();
    let mut numbers: Vec<i64> = loaded.areas.keys().copied().collect();
    numbers.sort();
    assert_eq!(numbers, vec![0, 1]);
    let door = loaded
        .record(1, zones::WARPS, 100.0, 200.0)
        .unwrap()
        .unwrap();
    assert_eq!((door.zone, door.leads_to), (Some(TOWN), Some(0x25)));
    assert_eq!(loaded.place(TOWN).unwrap(), Some("Iki Town".to_string()));
}
