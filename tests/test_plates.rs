//! Doors named by the plates beside them, and Lumiose's warps by the street they come out on
//! (spike 107).

use std::collections::HashSet;

use indexmap::IndexMap;

use super::fake_rom::{MessageLine, message_file};
use super::test_field::{ZoneSpec, zone_file};
use super::test_script::{case_script, push, sign_script};
use crate::games::common::garc::message_lines;
use crate::games::{gen6::rom::plates, gen6::rom::script};
use crate::py::Result;

const TOWN: i64 = 1;
const CAFE: i64 = 2;
const LAB: i64 = 3;
const ELSEWHERE: i64 = 4;
const UPPER: i64 = 5;
const GYM_ROOM: i64 = 6;
const PLACE: i64 = 22;
const OTHER_PLACE: i64 = 23;
const STORY: i64 = 7;

/// The town's story file: the plates of a café, of a lab in two lines, of the player's house,
/// which says whose it is, of the gate out, and of a shack in three lines.
fn lines() -> Vec<MessageLine> {
    vec![
        MessageLine::from("Café Soleil"),
        MessageLine::from("Sycamore\nPokémon Lab"),
        MessageLine::from(vec![
            0x0010,
            0x0002,
            0x0100,
            0x0000,
            i64::from(b'H'),
            i64::from(b'o'),
            i64::from(b'm'),
            i64::from(b'e'),
        ]),
        MessageLine::from("Route 4 Gate"),
        MessageLine::from("Welcome to the\nshack!\nStay awhile."),
    ]
}

fn le16(values: &[i64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&value| (value as u16).to_le_bytes())
        .collect()
}

fn le32(values: &[i64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&value| (value as u32).to_le_bytes())
        .collect()
}

fn put(buffer: &mut [u8], at: i64, data: &[u8]) {
    buffer[at as usize..at as usize + data.len()].copy_from_slice(data);
}

fn table() -> Vec<u8> {
    let mut rows = vec![0u8; (8 * crate::games::gen6::field::zonetable::INFO_SIZE) as usize];
    for (zone, parent, place) in [
        (TOWN, TOWN, PLACE),
        (CAFE, TOWN, PLACE),
        (LAB, TOWN, PLACE),
        (ELSEWHERE, ELSEWHERE, OTHER_PLACE),
        (UPPER, TOWN, PLACE),
        (GYM_ROOM, TOWN, PLACE),
    ] {
        let at = zone * crate::games::gen6::field::zonetable::INFO_SIZE;
        put(&mut rows, at + plates::INFO_STORY, &le16(&[STORY]));
        put(
            &mut rows,
            at + crate::games::gen6::field::zonetable::INFO_PARENT,
            &le16(&[parent]),
        );
        put(
            &mut rows,
            at + crate::games::gen6::field::zonetable::INFO_NAME,
            &le16(&[place]),
        );
    }
    put(
        &mut rows,
        GYM_ROOM * crate::games::gen6::field::zonetable::INFO_SIZE
            + crate::games::gen6::field::zonetable::INFO_KIND,
        &le16(&[0x11]),
    );
    rows
}

fn story(file: i64) -> Result<Vec<Vec<u8>>> {
    if file == STORY {
        message_lines(&message_file(&lines()))
    } else {
        Ok(Vec::new())
    }
}

/// The town and the zones its doors lead to, as `plates` is given them: the town's signs
/// 1 to 5 each show one line of the story file as a plate, sign 1 line 0 and so on.
fn town(
    furniture: &[(i64, i64, i64)],
    warps: &[(i64, i64, i64, i64)],
) -> Result<IndexMap<i64, plates::Zone>> {
    let warps: Vec<_> = warps
        .iter()
        .map(|&(to, kind, x, z)| (to, kind, x, z, None, None))
        .collect();
    let code = sign_script(
        &(1..6)
            .map(|number| (number, vec![(number - 1, script::PLATE)]))
            .collect(),
        false,
    );
    let data = zone_file(&ZoneSpec {
        furniture,
        warps: &warps,
        sections: 4,
        script: &code,
        ..ZoneSpec::default()
    });
    let zone = crate::games::gen6::field::zonedata::zone_file(&data)?;
    let zone = zone.expect("a zone data file");
    let mut zones: IndexMap<i64, plates::Zone> = IndexMap::new();
    zones.insert(
        TOWN,
        (
            data.clone(),
            crate::games::gen6::field::zonedata::records(
                &data,
                &zone,
                crate::games::gen6::field::zonedata::FURNITURE,
            )?,
            crate::games::gen6::field::zonedata::records(
                &data,
                &zone,
                crate::games::gen6::field::zonedata::WARP,
            )?,
        ),
    );
    for inside in [CAFE, LAB, ELSEWHERE] {
        zones.insert(inside, (Vec::new(), Vec::new(), Vec::new()));
    }
    Ok(zones)
}

/// Café Soleil's plate two cells from its door names it; the Lab's, a sign between two
/// doors nearly as near, names neither; a plate by the gate to another place names nothing
/// more, since the gate says its place already; one that says who the player is names nothing;
/// and a plate of three lines 9 cells from its door is too far, where one line would not be.
#[test]
fn test_a_plate_names_the_door_it_stands_by() -> Result<()> {
    let zones = town(
        &[(1, 10, 8), (2, 20, 10), (3, 30, 8), (4, 40, 8), (5, 50, 1)],
        &[
            (CAFE, crate::games::gen6::field::zonedata::WARP_DOOR, 10, 10),
            (LAB, crate::games::gen6::field::zonedata::WARP_DOOR, 18, 10),
            (LAB, crate::games::gen6::field::zonedata::WARP_DOOR, 22, 10),
            (CAFE, crate::games::gen6::field::zonedata::WARP_DOOR, 30, 10),
            (
                ELSEWHERE,
                crate::games::gen6::field::zonedata::WARP_DOOR,
                40,
                10,
            ),
            (LAB, crate::games::gen6::field::zonedata::WARP_DOOR, 50, 10),
        ],
    )?;
    let named = plates::plate_names(&zones, &table(), &story, &HashSet::from([TOWN]), None, &[])?;
    assert_eq!(
        named,
        IndexMap::from([((TOWN, 0), "Café Soleil".to_string())])
    );
    Ok(())
}

/// Vaniville's houses have their plates at the garden gate, 12 cells from the door: a plate
/// of one line names the nearest door that far, and one of more lines only within 6.
#[test]
fn test_a_plate_of_one_line_names_a_door_further_off() -> Result<()> {
    let zones = town(
        &[(1, 10, 0)],
        &[(CAFE, crate::games::gen6::field::zonedata::WARP_DOOR, 10, 11)],
    )?;
    assert_eq!(
        plates::plate_names(&zones, &table(), &story, &HashSet::from([TOWN]), None, &[])?,
        IndexMap::from([((TOWN, 0), "Café Soleil".to_string())])
    );
    let zones = town(
        &[(5, 10, 3)],
        &[(CAFE, crate::games::gen6::field::zonedata::WARP_DOOR, 10, 11)],
    )?;
    assert_eq!(
        plates::plate_names(&zones, &table(), &story, &HashSet::from([TOWN]), None, &[])?,
        IndexMap::<(i64, i64), String>::new()
    );
    Ok(())
}

/// ORAS's plates put a slogan on a line of its own, "“Ride in Gravel and Shake Up Your
/// Soul!” Rydel's Cycles": a plate of more than one line holding a facility its town lists
/// is named by that alone, and a plate of one line keeps its own words, which may say more.
/// A kind that is the way into one building and no other, as ORAS's Devon Corporation is.
const ENTRANCE: i64 = 0x0F01;

/// What `plates.entrance_names` gives a town whose signs 1, 2 and so on each show a line of
/// `plate_lines` as a plate, far from its doors, whose list has `listed`, and whose warp 1 is an
/// entrance into a house that `doors` has not named. Warp 0 is the door of the Café.
fn entrance_names(
    plate_lines: &[&str],
    listed: &[&str],
    doors: &[((i64, i64), &str)],
) -> Result<IndexMap<(i64, i64), String>> {
    let count = plate_lines.len() as i64;
    let code = sign_script(
        &(1..count + 1)
            .map(|n| (n, vec![(n - 1, script::PLATE)]))
            .collect(),
        false,
    );
    let furniture: Vec<(i64, i64, i64)> = (1..count + 1).map(|n| (n, 10 * n, 1)).collect();
    let data = zone_file(&ZoneSpec {
        furniture: &furniture,
        warps: &[
            (
                CAFE,
                crate::games::gen6::field::zonedata::WARP_DOOR,
                10,
                10,
                None,
                None,
            ),
            (LAB, ENTRANCE, 80, 10, None, None),
        ],
        sections: 4,
        script: &code,
        ..ZoneSpec::default()
    });
    let zone = crate::games::gen6::field::zonedata::zone_file(&data)?.expect("a zone data file");
    let mut zones: IndexMap<i64, plates::Zone> = IndexMap::new();
    zones.insert(
        TOWN,
        (
            data.clone(),
            crate::games::gen6::field::zonedata::records(
                &data,
                &zone,
                crate::games::gen6::field::zonedata::FURNITURE,
            )?,
            crate::games::gen6::field::zonedata::records(
                &data,
                &zone,
                crate::games::gen6::field::zonedata::WARP,
            )?,
        ),
    );
    zones.insert(CAFE, (Vec::new(), Vec::new(), Vec::new()));
    zones.insert(LAB, (Vec::new(), Vec::new(), Vec::new()));
    let message: Vec<MessageLine> = plate_lines
        .iter()
        .map(|&line| MessageLine::from(line))
        .collect();
    let lines = message_lines(&message_file(&message))?;
    let story = move |file: i64| -> Result<Vec<Vec<u8>>> {
        Ok(if file == STORY {
            lines.clone()
        } else {
            Vec::new()
        })
    };
    plates::entrance_names(
        &zones,
        &table(),
        &story,
        &HashSet::from([TOWN]),
        &IndexMap::from([(PLACE, listed.iter().map(|name| name.to_string()).collect())]),
        &doors
            .iter()
            .map(|&(door, name)| (door, name.to_string()))
            .collect(),
        &HashSet::from([ENTRANCE]),
    )
}

/// Rustboro's Devon Corporation plate, "The Devon Corporation / For all your living needs...",
/// stands 18 cells from its doors, past the Gym's, so no door is paired with it by distance. The
/// Café's plate holds a facility a door name claims already, and the town's list has one more
/// that no plate holds, which only the second rule would name (2026-10-06).
#[test]
fn test_an_entrance_is_the_facility_the_one_plate_holding_an_unclaimed_one_says() -> Result<()> {
    let named = entrance_names(
        &[
            "Café Soleil",
            "The Devon\nCorporation\nFor all your living needs",
        ],
        &["Café", "Devon Corporation", "Harbor"],
        &[((TOWN, 0), "Café Soleil")],
    )?;
    assert_eq!(
        named,
        IndexMap::from([((TOWN, 1), "Devon Corporation".to_string())])
    );
    Ok(())
}

/// Route 112's Cable Car station: its list has "Cable Car Base Station" besides the Café, and
/// the only plate is the Café's.
#[test]
fn test_an_entrance_no_plate_holds_is_the_one_facility_nothing_else_has() -> Result<()> {
    let named = entrance_names(
        &["Café Soleil"],
        &["Café", "Cable Car Base Station"],
        &[((TOWN, 0), "Café Soleil")],
    )?;
    assert_eq!(
        named,
        IndexMap::from([((TOWN, 1), "Cable Car Base Station".to_string())])
    );
    Ok(())
}

/// Lilycove's Museum: the only plate is the town's sign, which holds both "Harbor" and
/// "Lilycove Museum", so neither is its own and neither is left over. Two plates each holding
/// another one are two names, not one.
#[test]
fn test_an_entrance_is_left_unnamed_where_the_facilities_left_are_not_one() -> Result<()> {
    assert!(entrance_names(&["Lilycove Harbor Museum"], &["Harbor", "Museum"], &[])?.is_empty());
    assert!(entrance_names(&["Harbor", "Museum"], &["Harbor", "Museum"], &[])?.is_empty());
    Ok(())
}

#[test]
fn test_a_plate_of_more_than_one_line_is_named_by_what_its_towns_list_calls_it() -> Result<()> {
    let zones = town(
        &[(5, 10, 8), (1, 30, 8)],
        &[
            (CAFE, crate::games::gen6::field::zonedata::WARP_DOOR, 10, 10),
            (CAFE, crate::games::gen6::field::zonedata::WARP_DOOR, 30, 10),
        ],
    )?;
    let facilities = IndexMap::from([(PLACE, vec!["shack".to_string(), "Café".to_string()])]);
    let named = plates::plate_names(
        &zones,
        &table(),
        &story,
        &HashSet::from([TOWN]),
        Some(&facilities),
        &[],
    )?;
    assert_eq!(
        named,
        IndexMap::from([
            ((TOWN, 0), "shack".to_string()),
            ((TOWN, 1), "Café Soleil".to_string()),
        ])
    );
    Ok(())
}

/// Slateport's list is full at six and leaves out the Oceanic Museum, whose plate says its
/// slogan first and its name on the next page: the list's text still names it (Orion,
/// 2026-10-13). Only a whole line, or two run together as the Japanese break a name: a plate
/// of one line keeps its own words where a facility is within them, "Poké Ball Boutique".
#[test]
fn test_a_plate_is_named_by_a_line_that_is_a_facility_of_the_lists_text() -> Result<()> {
    let mut museum: Vec<i64> = "The endless sea.".chars().map(|c| c as i64).collect();
    museum.extend([
        crate::games::common::text::MARKER,
        1,
        crate::games::common::text::PAGE_CLEAR,
    ]);
    museum.extend("Oceanic Museum".chars().map(|c| c as i64));
    let said = vec![
        MessageLine::from("Poké Ball Boutique"),
        MessageLine::from(museum),
        MessageLine::from("Welcome!\nSlateport\nMarket"),
    ];
    let story = move |file: i64| -> Result<Vec<Vec<u8>>> {
        if file == STORY {
            message_lines(&message_file(&said))
        } else {
            Ok(Vec::new())
        }
    };
    let zones = town(
        &[(1, 10, 8), (2, 30, 8), (3, 50, 8)],
        &[
            (CAFE, crate::games::gen6::field::zonedata::WARP_DOOR, 10, 10),
            (CAFE, crate::games::gen6::field::zonedata::WARP_DOOR, 30, 10),
            (CAFE, crate::games::gen6::field::zonedata::WARP_DOOR, 50, 10),
        ],
    )?;
    let every = ["Boutique", "Oceanic Museum", "Slateport Market"].map(String::from);
    let named = plates::plate_names(
        &zones,
        &table(),
        &story,
        &HashSet::from([TOWN]),
        None,
        &every,
    )?;
    assert_eq!(
        named,
        IndexMap::from([
            ((TOWN, 0), "Poké Ball Boutique".to_string()),
            ((TOWN, 1), "Oceanic Museum".to_string()),
            ((TOWN, 2), "Slateport Market".to_string()),
        ])
    );
    Ok(())
}

/// The Japanese museum's plate puts its name in 「」 after its motto, and the door was named
/// by the motto (a user, 2026-10-26).
#[test]
fn test_a_name_in_quotation_marks_on_a_line_of_its_own_names_the_plate() -> Result<()> {
    let said = vec![MessageLine::from(
        "はてしない うみ……\n「うみの かがく はくぶつかん」",
    )];
    let story = move |file: i64| -> Result<Vec<Vec<u8>>> {
        if file == STORY {
            message_lines(&message_file(&said))
        } else {
            Ok(Vec::new())
        }
    };
    let zones = town(
        &[(1, 10, 8)],
        &[(CAFE, crate::games::gen6::field::zonedata::WARP_DOOR, 10, 10)],
    )?;
    let named = plates::plate_names(
        &zones,
        &table(),
        &story,
        &HashSet::from([TOWN]),
        None,
        &["うみのかがくはくぶつかん".to_string()],
    )?;
    assert_eq!(
        named,
        IndexMap::from([((TOWN, 0), "うみのかがくはくぶつかん".to_string())])
    );
    Ok(())
}

/// Stern's Shipyard's plate shows one of three notices as the story goes, each under its
/// name, one in French capitalised otherwise; the door was "door" (a user, 2026-10-26). A plate
/// whose notices share no one facility names nothing.
#[test]
fn test_a_plate_showing_one_of_several_notices_is_named_by_the_facility_all_of_them_hold()
-> Result<()> {
    let said = vec![
        MessageLine::from("Stern's Shipyard\nWanted: a sailor."),
        MessageLine::from("STERN'S Shipyard\nFerry at the Harbor."),
        MessageLine::from("Harbor\nNo ferry today."),
        MessageLine::from("Stern's Shipyard\nThe ferry is done."),
    ];
    let code = sign_script(
        &IndexMap::from([
            (
                1,
                vec![(0, script::PLATE), (1, script::PLATE), (3, script::PLATE)],
            ),
            (2, vec![(0, script::PLATE), (2, script::PLATE)]),
        ]),
        false,
    );
    let data = zone_file(&ZoneSpec {
        furniture: &[(1, 10, 8), (2, 30, 8)],
        warps: &[
            (
                CAFE,
                crate::games::gen6::field::zonedata::WARP_DOOR,
                10,
                10,
                None,
                None,
            ),
            (
                CAFE,
                crate::games::gen6::field::zonedata::WARP_DOOR,
                30,
                10,
                None,
                None,
            ),
        ],
        sections: 4,
        script: &code,
        ..ZoneSpec::default()
    });
    let zone = crate::games::gen6::field::zonedata::zone_file(&data)?.expect("a zone data file");
    let zones: IndexMap<i64, plates::Zone> = IndexMap::from([
        (
            TOWN,
            (
                data.clone(),
                crate::games::gen6::field::zonedata::records(
                    &data,
                    &zone,
                    crate::games::gen6::field::zonedata::FURNITURE,
                )?,
                crate::games::gen6::field::zonedata::records(
                    &data,
                    &zone,
                    crate::games::gen6::field::zonedata::WARP,
                )?,
            ),
        ),
        (CAFE, (Vec::new(), Vec::new(), Vec::new())),
    ]);
    let shown = message_lines(&message_file(&said))?;
    let story = move |file: i64| -> Result<Vec<Vec<u8>>> {
        Ok(if file == STORY {
            shown.clone()
        } else {
            Vec::new()
        })
    };
    let facilities = IndexMap::from([(
        PLACE,
        vec!["Stern's Shipyard".to_string(), "Harbor".to_string()],
    )]);
    let named = plates::plate_names(
        &zones,
        &table(),
        &story,
        &HashSet::from([TOWN]),
        Some(&facilities),
        &[],
    )?;
    assert_eq!(
        named,
        IndexMap::from([((TOWN, 0), "Stern's Shipyard".to_string())])
    );
    Ok(())
}

/// Mauville Hills' "Apt. 13 / Wattson's Home" says nothing its door is not called (Orion,
/// 2026-10-13); a plate that says more than the name is not one.
#[test]
fn test_a_plate_saying_only_its_doors_name_is_a_door_sign() -> Result<()> {
    let zones = town(
        &[(1, 10, 8), (5, 30, 8)],
        &[
            (CAFE, crate::games::gen6::field::zonedata::WARP_DOOR, 10, 10),
            (CAFE, crate::games::gen6::field::zonedata::WARP_DOOR, 30, 10),
        ],
    )?;
    let doors = IndexMap::from([
        ((TOWN, 0), "Café Soleil".to_string()),
        ((TOWN, 1), "shack".to_string()),
    ]);
    assert_eq!(
        plates::door_signs(&zones, &table(), &story, &HashSet::from([TOWN]), &doors)?,
        HashSet::from([(TOWN, 0)])
    );
    Ok(())
}

/// A warp placed on a street: where it goes, which warp there it comes out at, and where
/// on its line it is, as the game measures it.
fn warp_on(line: i64, to: i64, back: i64, a: i64, x: i64, size: (i64, i64)) -> Vec<u8> {
    [
        le16(&[
            to,
            back,
            0x0300,
            crate::games::gen6::field::zonedata::ON_RAIL,
        ]),
        le32(&[line]),
        le16(&[a, x, size.0, size.1]),
        vec![0; 4],
    ]
    .concat()
}

fn mark(line: i64, street: i64, a: i64, x: i64, size: (i64, i64)) -> Vec<u8> {
    [
        le16(&[
            2000,
            street,
            0,
            15,
            crate::games::gen6::field::zonedata::ON_RAIL,
            0,
        ]),
        le32(&[line]),
        le16(&[a, x, size.0, size.1]),
    ]
    .concat()
}

/// The mark nearest where the warp comes out, on the line it comes out on, names the street:
/// the Vernal Avenue mark at the alley's mouth, not the one saying "no street" further on. A
/// warp onto North Boulevard comes out on a line with no mark, in a zone all of whose marks
/// say North Boulevard.
#[test]
fn test_a_warp_between_two_of_lumioses_zones_is_named_for_the_street_it_comes_out_on() -> Result<()>
{
    let (avenue, boulevard, north, landing) = (0x0305C50A, 0x0504BEF6, 0x0003ABFA, 0x0004F00D);
    let mut zones: IndexMap<i64, plates::Zone> = IndexMap::new();
    zones.insert(
        TOWN,
        (
            Vec::new(),
            Vec::new(),
            vec![
                warp_on(boulevard, UPPER, 1, 8, 11, (1, 1)),
                warp_on(boulevard, LAB, 0, 1, 0, (1, 1)),
            ],
        ),
    );
    zones.insert(
        UPPER,
        (
            Vec::new(),
            Vec::new(),
            vec![
                warp_on(avenue, CAFE, 0, 2, 0, (1, 1)),
                warp_on(avenue, TOWN, 0, -4, 8, (1, 1)),
            ],
        ),
    );
    zones.insert(
        LAB,
        (
            Vec::new(),
            Vec::new(),
            vec![
                warp_on(landing, TOWN, 1, -46, 0, (1, 1)),
                warp_on(north, CAFE, 0, 3, 0, (1, 1)),
            ],
        ),
    );
    let marks = [
        le32(&[3]),
        mark(avenue, 3, -6, -9, (-1, 19)),
        mark(avenue, 13, -20, -9, (-1, 19)),
        mark(0x0999, 1, 0, 0, (1, 1)),
    ]
    .concat();
    let boulevard_marks = [le32(&[1]), mark(north, 2, 0, -33, (30, 67))].concat();
    let names: Vec<String> = [
        "South",
        "Boulevard",
        "North",
        "Boulevard",
        "Vernal",
        "Avenue",
    ]
    .iter()
    .map(|name| name.to_string())
    .collect();
    assert_eq!(
        plates::street_names(&zones, &table(), &[boulevard_marks, marks], &names)?,
        IndexMap::from([
            ((TOWN, 0), "Vernal Avenue".to_string()),
            ((TOWN, 1), "North Boulevard".to_string()),
        ])
    );
    Ok(())
}

/// Lumiose's other zones and Coumarine's lower town are not their own parents, but the
/// player walks about in them: a building's way out comes out there, or their records are on
/// streets. Coumarine's Gym has its puzzle on rails, and a kind of its own.
#[test]
fn test_where_a_buildings_way_out_comes_out_in_its_towns_place_is_outdoors() -> Result<()> {
    let mut zones: IndexMap<i64, plates::Zone> = IndexMap::new();
    zones.insert(TOWN, (Vec::new(), Vec::new(), Vec::new()));
    zones.insert(
        CAFE,
        (
            Vec::new(),
            Vec::new(),
            vec![
                [
                    le16(&[
                        UPPER,
                        0,
                        crate::games::gen6::field::zonedata::WARP_OUT,
                        0,
                        9,
                        0,
                        9,
                        1,
                        1,
                    ]),
                    vec![0; 6],
                ]
                .concat(),
            ],
        ),
    );
    zones.insert(UPPER, (Vec::new(), Vec::new(), Vec::new()));
    zones.insert(
        LAB,
        (
            Vec::new(),
            Vec::new(),
            vec![warp_on(0x0305C50A, TOWN, 0, 1, 1, (1, 1))],
        ),
    );
    zones.insert(
        GYM_ROOM,
        (
            Vec::new(),
            Vec::new(),
            vec![warp_on(0x0103B0E9, TOWN, 0, 1, 1, (1, 1))],
        ),
    );
    assert_eq!(
        plates::outdoors(&zones, &table())?,
        HashSet::from([TOWN, UPPER, LAB])
    );
    Ok(())
}

/// Every vending machine is a record of its zone's own script whose case runs one of five
/// shared ones (spike 210); a record of a shared script's own number is left to its number.
#[test]
fn test_furniture_whose_case_runs_a_shared_script_is_that_script() -> Result<()> {
    let code = case_script(&IndexMap::from([
        (
            9,
            vec![
                script::PROC,
                push(0x28F4),
                script::NATIVE,
                90,
                4,
                script::RETN,
            ],
        ),
        (3, vec![script::PROC, script::RETN]),
    ]));
    let data = zone_file(&ZoneSpec {
        furniture: &[(9, 90, 160), (3, 92, 160), (0x27E3, 98, 159)],
        sections: 4,
        script: &code,
        ..ZoneSpec::default()
    });
    let zone = crate::games::gen6::field::zonedata::zone_file(&data)?.expect("a zone data file");
    let furniture = crate::games::gen6::field::zonedata::records(
        &data,
        &zone,
        crate::games::gen6::field::zonedata::FURNITURE,
    )?;
    let zones: IndexMap<i64, plates::Zone> =
        IndexMap::from([(TOWN, (data, furniture, Vec::new()))]);
    assert_eq!(
        plates::native_furniture(&zones)?,
        IndexMap::from([((TOWN, 9), (90, 0x28F4))])
    );
    assert_eq!(
        plates::shared_furniture(&plates::native_furniture(&zones)?),
        IndexMap::from([((TOWN, 9), 0x28F4)])
    );
    Ok(())
}
