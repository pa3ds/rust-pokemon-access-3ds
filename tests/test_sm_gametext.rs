//! Sun's own words for its field: the line a zone's script shows for a thing, a sign or a person,
//! the people by their trainers, and the doors by the Town Map and the plates (spike A).

use std::collections::HashSet;

use indexmap::IndexMap;

use crate::games::common::field::things::{FURNITURE_SOURCE, WARP_SOURCE};
use crate::games::sm::field::place_of;
use crate::games::sm::gametext::{self, Sign};
use crate::games::sm::script;
use crate::games::sm::text::SunText;
use crate::games::sm::zones::{self, Area, Record, SunZones};

const PROC: i64 = 46;
const RETN: i64 = 48;
const CALL: i64 = 49;
const SWITCH: i64 = 129;
const CASETBL: i64 = 130;
const SYSREQ_N: i64 = 135;

/// A packed push of a constant: the opcode low, the value high.
fn push(value: i64) -> i64 {
    (value & 0xFFFF) << 16 | script::PUSH_P_C
}

/// One cell in Pawn's compact encoding, 7 bits a byte, the first byte's 0x40 its sign.
fn compact(cell: i64) -> Vec<u8> {
    let mut value = if cell & 0x80000000 != 0 {
        cell - (1 << 32)
    } else {
        cell
    };
    let mut groups: Vec<i64> = Vec::new();
    loop {
        groups.insert(0, value & 0x7F);
        value >>= 7;
        if (value == 0 && groups[0] & 0x40 == 0) || (value == -1 && groups[0] & 0x40 != 0) {
            break;
        }
    }
    let last = groups.len() - 1;
    groups
        .iter()
        .enumerate()
        .map(|(i, &group)| (if i < last { group | 0x80 } else { group }) as u8)
        .collect()
}

fn le32(values: &[i64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&value| (value as u32).to_le_bytes())
        .collect()
}

/// A zone's script as Sun's are laid out: case 3 calls a function showing line 5 through a
/// helper of three arguments, case 4 a sign's line 4 through one of one, and case 5 a function
/// whose call comes nowhere near the message window.
fn zone_script() -> Vec<u8> {
    let rel = |at: i64, target: i64| (target - at) & 0xFFFFFFFF;

    #[rustfmt::skip]
    let mut code: Vec<i64> = vec![
        PROC, SYSREQ_N, 0, 0, RETN,                                            // 0x00 shows text
        PROC, push(1), push(0), push(5), push(12), CALL, rel(0x28, 0), RETN,   // 0x14
        PROC, push(4), push(4), CALL, rel(0x40, 0), RETN,                      // 0x34
        PROC, RETN,                                                            // 0x4c shows none
        PROC, push(9), push(4), CALL, rel(0x60, 0x4C), RETN,                   // 0x54
        PROC, SWITCH, rel(0x70, 0xA0),                                         // 0x6c main
        push(0), CALL, rel(0x7C, 0x14),                                        // 0x78 case 3
        push(0), CALL, rel(0x88, 0x34),                                        // 0x84 case 4
        push(0), CALL, rel(0x94, 0x54),                                        // 0x90 case 5
        RETN,                                                                  // 0x9c
    ];
    let table = 0xA0;
    let pairs = [(3, 0x78), (4, 0x84), (5, 0x90)];
    code.extend([CASETBL, pairs.len() as i64, rel(table + 4, 0x9C)]);
    for (index, (value, target)) in pairs.iter().enumerate() {
        let at = table + 4 * (3 + 2 * index as i64);
        code.extend([*value, rel(at, *target)]);
    }
    let cod = 0x40;
    let body: Vec<u8> = code.iter().flat_map(|&cell| compact(cell)).collect();
    // struct.pack("<IHBBHHIIIIIIIIII", ...)
    let mut head: Vec<u8> = Vec::new();
    head.extend((cod + body.len() as u32).to_le_bytes());
    head.extend((script::MAGIC as u16).to_le_bytes());
    head.extend([10u8, 10u8]);
    head.extend(0x1Cu16.to_le_bytes());
    head.extend(8u16.to_le_bytes());
    head.extend(le32(&[
        cod as i64,
        cod as i64 + 4 * code.len() as i64,
        0,
        0,
        0x6C,
        0x38,
        0x38,
        0x40,
        0x40,
        0x40,
    ]));
    head.resize(0x38, 0);
    head.extend(le32(&[0, script::MESSAGE]));
    head.extend(body);
    head
}

#[test]
fn test_a_case_shows_the_first_line_passed_to_a_call_that_comes_to_the_message_window() {
    let read = script::ZoneScript::new(&zone_script()).unwrap();
    assert_eq!(read.cases(), HashSet::from([3, 4, 5]));
    assert_eq!(read.line(3, None).unwrap(), Some(5));
    assert_eq!(read.line(4, Some(1)).unwrap(), Some(4));
    assert_eq!(read.line(3, Some(1)).unwrap(), None);
    assert_eq!(read.line(5, None).unwrap(), None);
    assert_eq!(read.line(6, None).unwrap(), None);
}

#[test]
fn test_a_pack_reads_each_zone_s_script_and_none_for_one_it_cannot() {
    let (first, broken) = (zone_script(), b"not a script".to_vec());
    let offsets = [
        16,
        16 + first.len() as i64,
        16 + first.len() as i64 + broken.len() as i64,
    ];
    let mut pack: Vec<u8> = b"ZS".to_vec();
    pack.extend(2u16.to_le_bytes());
    pack.extend(le32(&offsets));
    pack.extend(&first);
    pack.extend(&broken);
    let read = script::scripts(&pack).unwrap();
    assert!(read.len() == 2 && read[1].is_none());
    assert_eq!(read[0].as_ref().unwrap().line(3, None).unwrap(), Some(5));
}

const TOWN: i64 = 0x0;
const CITY: i64 = 0x1;
const CENTER: i64 = 0x10;
const HOUSE: i64 = 0x11;
const LIBRARY: i64 = 0x12;
const OTHER_CENTER: i64 = 0x13;
const PLACE_LINES: [&str; 2] = ["Iki Town", "Malie City"];

fn pack_u16(rows: &mut [u8], at: i64, value: i64) {
    rows[at as usize..at as usize + 2].copy_from_slice(&(value as u16).to_le_bytes());
}

/// A town with a Pokémon Center, a house and a library, and a city with a Pokémon Center of
/// the same music, each building's door a record of the town's area.
fn doors_rom() -> SunZones {
    let mut rows = vec![0u8; (zones::ZONE_ROW * 0x20) as usize];
    for (zone, place, kind, music) in [
        (TOWN, 0, zones::OUTDOORS, 1),
        (CITY, 1, zones::OUTDOORS, 2),
        (CENTER, 0, zones::INDOORS, 14),
        (HOUSE, 0, zones::INDOORS, 28),
        (LIBRARY, 0, zones::INDOORS, 51),
        (OTHER_CENTER, 1, zones::INDOORS, 14),
    ] {
        let at = zone * zones::ZONE_ROW;
        pack_u16(&mut rows, at + gametext::ROW_MUSIC, music);
        pack_u16(&mut rows, at + zones::ROW_PLACE, place);
        pack_u16(&mut rows, at + zones::ROW_KIND, kind);
    }

    let door = |x: f64, zone: i64, to: i64| Record {
        x,
        z: 1000.0,
        shape: None,
        zone: Some(zone),
        leads_to: Some(to),
        kind: zones::WARP_DOOR,
        ..Default::default()
    };
    let town = vec![
        door(1000.0, TOWN, CENTER),
        door(3000.0, TOWN, HOUSE),
        door(5000.0, TOWN, LIBRARY),
    ];
    let city = vec![door(2000.0, CITY, OTHER_CENTER)];
    SunZones {
        rows,
        places: SunText {
            files: IndexMap::from([(
                zones::PLACE_NAMES,
                PLACE_LINES.iter().map(|line| line.to_string()).collect(),
            )]),
        },
        areas: IndexMap::from([
            (
                0,
                Area {
                    warps: vec![town],
                    things: vec![],
                    ..Default::default()
                },
            ),
            (
                1,
                Area {
                    warps: vec![city],
                    things: vec![],
                    ..Default::default()
                },
            ),
        ]),
    }
}

/// Town Map records of (zone, how it is marked, x, z, facility line).
fn town_map(records: &[(i64, i64, i64, i64, i64)]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for &(zone, marked, x, z, line) in records {
        let mut record = vec![0u8; gametext::TOWN_MAP_RECORD as usize];
        record[..20].copy_from_slice(&le32(&[0, zone, marked, x, z]));
        pack_u16(&mut record, gametext::TOWN_MAP_LINE, line);
        out.extend(record);
    }
    out
}

fn plate_at(x: f64, z: f64) -> Record {
    Record {
        x,
        z,
        shape: None,
        zone: Some(TOWN),
        kind: gametext::PLATE,
        ..Default::default()
    }
}

/// The town's Center by its mark on the door, the city's by the music both share and both
/// towns list, the library by its plate six cells off, and the house by a plate that says only
/// its name, a door sign. A description on the map names no door.
#[test]
fn test_doors_are_named_by_the_town_map_s_mark_its_kind_and_the_plate_beside_them() {
    let mut facilities: Vec<String> = vec![String::new(); 30];
    facilities.push("A route of\nmany trainers.".to_string());
    facilities[20] = "Pokémon Center".to_string();
    let rom = doors_rom();
    let marks = town_map(&[
        (TOWN, gametext::MARKED_CENTER, 1000, 1000, 20),
        (CITY, gametext::MARKED_FACILITY, 0, 0, 20),
        (TOWN, gametext::MARKED_FACILITY, 3000, 1000, 30),
    ]);
    let plate = plate_at(5000.0, 1300.0);
    let small = plate_at(3000.0, 1100.0);
    let route = plate_at(1000.0, 1100.0);
    let signs = [
        Sign {
            area: 0,
            record: plate,
            said: "Malie Library\n“Read it all!”".to_string(),
        },
        Sign {
            area: 0,
            record: small,
            said: "Old House".to_string(),
        },
        Sign {
            area: 0,
            record: route,
            said: "Iki Town".to_string(),
        },
    ];
    let (named, door_signs) = gametext::doors(&rom, &marks, &facilities, &signs).unwrap();

    let door = |x: f64, zone: i64| -> Option<&str> {
        named
            .get(&(zone, place_of(WARP_SOURCE, x, 1000.0).unwrap()))
            .map(String::as_str)
    };
    assert_eq!(door(1000.0, TOWN), Some("Pokémon Center"));
    assert_eq!(door(2000.0, CITY), Some("Pokémon Center"));
    assert_eq!(door(5000.0, TOWN), Some("Malie Library"));
    assert_eq!(door(3000.0, TOWN), Some("Old House"));
    assert_eq!(
        door_signs,
        HashSet::from([(TOWN, place_of(FURNITURE_SOURCE, 3000.0, 1100.0).unwrap())])
    );
}

#[test]
fn test_a_mark_far_from_every_door_names_none() {
    let rom = doors_rom();
    let marks = town_map(&[(TOWN, gametext::MARKED_FACILITY, 2000, 1000, 20)]);
    let mut facilities: Vec<String> = vec![String::new(); 20];
    facilities.push("Pokémon Center".to_string());
    let (named, _signs) = gametext::doors(&rom, &marks, &facilities, &[]).unwrap();
    assert!(!named.contains_key(&(TOWN, place_of(WARP_SOURCE, 1000.0, 1000.0).unwrap())));
    assert!(!named.contains_key(&(TOWN, place_of(WARP_SOURCE, 3000.0, 1000.0).unwrap())));
}

#[test]
fn test_a_model_trainers_of_one_class_stand_as_is_that_class_and_each_trainer_their_own() {
    let trainer = |number: i64, model: i64, record: i64| Record {
        x: 0.0,
        z: 0.0,
        shape: None,
        zone: Some(TOWN),
        kind: number,
        code: model,
        number: record,
        ..Default::default()
    };

    let rom = SunZones {
        rows: vec![0u8; zones::ZONE_ROW as usize],
        places: SunText {
            files: IndexMap::new(),
        },
        areas: IndexMap::from([(
            0,
            Area {
                warps: vec![],
                things: vec![],
                trainers: vec![vec![
                    trainer(25, 0x43, 0x66),
                    trainer(26, 0x43, 0x67),
                    trainer(1, 0x44, 0x65),
                ]],
                ..Default::default()
            },
        )]),
    };
    let classes: Vec<String> = ["", "", "Youngster", "Lass"]
        .iter()
        .map(|class| class.to_string())
        .collect();
    let mut names: Vec<String> = vec![String::new(); 30];
    names[1] = "Madison".to_string();
    names[25] = "Jimmy".to_string();
    names[26] = "Kevin".to_string();
    let mut trainers: Vec<i64> = vec![0; 30];
    trainers[1] = 3;
    trainers[25] = 2;
    trainers[26] = 2;
    let (sprites, own) = gametext::people(&rom, &trainers, &classes, &names);
    assert_eq!(sprites, IndexMap::from([(0x43, "Youngster".to_string())]));
    assert_eq!(
        own,
        IndexMap::from([
            ((TOWN, 0x66), "Youngster Jimmy".to_string()),
            ((TOWN, 0x67), "Youngster Kevin".to_string()),
            ((TOWN, 0x65), "Lass Madison".to_string()),
        ])
    );
}

#[test]
fn test_a_thing_is_named_by_its_first_sentence_or_a_board_by_its_first_line() {
    let sentence = |said: &str| gametext::sentence(Some(said));
    assert_eq!(
        sentence(
            "It's a cute stuffed doll that looks like Pikachu,\nthe Mouse Pokémon!\n\
             It looks soft."
        )
        .as_deref(),
        Some("It's a cute stuffed doll that looks like Pikachu, the Mouse Pokémon!")
    );
    assert_eq!(
        sentence("Lush Jungle\n\u{201c}A treasure trove!\u{201d}").as_deref(),
        Some("Lush Jungle")
    );
    let stop = "\u{ff01}";
    assert_eq!(
        sentence(&format!(
            "ねずみポケモン ピカチュウの ぬいぐるみだ{stop}\nかわいい"
        )),
        Some(format!("ねずみポケモン ピカチュウの ぬいぐるみだ{stop}"))
    );
    assert_eq!(
        sentence("It's Meowth's bed.").as_deref(),
        Some("It's Meowth's bed")
    );
    assert_eq!(
        sentence("It's Cosmog's bed...").as_deref(),
        Some("It's Cosmog's bed...")
    );
    assert_eq!(sentence("..."), None);
    assert_eq!(sentence("\u{2019}s House"), None);
    assert_eq!(sentence(&("A".repeat(120) + ".")), None);
}
