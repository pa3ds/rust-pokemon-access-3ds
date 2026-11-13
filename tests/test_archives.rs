//! The game's own words out of the player's ROM: place names, the rival by their sprite, the
//! buildings behind doors, and the PC, in ORAS and under X's own numbers.

use std::path::Path;

use indexmap::IndexMap;

use super::fake_rom::{
    GarcFile, MessageLine, TmpDir, garc, garc_with, lz11_literals, message_file, rom_image,
};
use super::test_rail::{ACROSS, C, P2, P3, street_section, walls};
use crate::core::rom::Rom;
use crate::games::common::field::chunk::CHUNK;
use crate::games::common::field::ground::{CUT, SURF};
use crate::games::common::garc::{garc_files, message_lines};
use crate::games::gen6::field::zonedata::WARP_DOOR;
use crate::games::gen6::field::zonetable::{INFO_KIND, INFO_NAME, INFO_PARENT, INFO_SIZE};
use crate::games::gen6::rom::archives::{self, BERRY_SPOT, GameText, PC_LINES};
use crate::games::gen6::rom::layout::{RomLayout, TownFacilities};
use crate::games::gen6::rom::maps::{MapChunks, Streets};
use crate::games::gen6::rom::people::DOUBLE;
use crate::games::{gen6::field::mapblock, gen6::field::rail};
use crate::games::{oras, xy};
use crate::py::Kind;

const OMEGA_RUBY: i64 = 0x000400000011C400;
const TRAINERS: [&str; 8] = [
    "-", "Brendan", "Brendan", "Brendan", "May", "May", "May", "Calvin",
];
const FACILITIES: [&str; 5] = [
    "",
    "Pokémon Center",
    "Poké Mart",
    "Petalburg City Gym",
    "Pokémon Lab",
];
const CLASSES: [&str; 6] = [
    "Pokémon Trainer",
    "Youngster",
    "Team Magma",
    "Twins",
    "Lass",
    "Hiker",
];
const YOUNGSTER: i64 = 1;
const MAGMA: i64 = 2;
const TWINS: i64 = 3;
const LASS: i64 = 4;
const HIKER: i64 = 5;
/// Each trainer's class, by trainer id: 7 and 8 are Youngsters, 9 and 10 grunts, 11 and 12 twins,
/// 13 a Lass, 14 a Hiker and 15 a Youngster.
const TRAINER_CLASSES: [i64; 16] = [
    0, 0, 0, 0, 0, 0, 0, YOUNGSTER, YOUNGSTER, MAGMA, MAGMA, TWINS, TWINS, LASS, HIKER, YOUNGSTER,
];
const DOUBLES: [i64; 1] = [TWINS];
const TRAINER_NAMES: [&str; 16] = [
    "-",
    "Brendan",
    "Brendan",
    "Brendan",
    "May",
    "May",
    "May",
    "Calvin",
    "Billy",
    "Grunt",
    "Grunt",
    "Amy & Liv",
    "Gina & Mia",
    "Tiana",
    "Brice",
    "Tommy",
];
/// (sprite, trainer) for every trainer standing in the fake towns.
const TRAINERS_ON_THE_MAP: [(i64, i64); 10] = [
    (0xDA, 7),
    (0xDA, 8),  // two Youngsters: the sprite is a Youngster,
    (0xDA, 11), // whatever a pair beside them is
    (0xC7, 9),
    (0xC7, 10), // two grunts, one name: "Team Magma Grunt"
    (0x10A, 11),
    (0x10A, 12), // twins: a pair class, so the sprite is nobody
    (0xDB, 13),  // one Lass only: not enough to say what the sprite is
    (0xE9, 14),
    (0xE9, 15), // a Hiker and a Youngster: two classes, so nobody
];
/// A town of the AreaNav names what it has by these lines of its text.
const CENTER: i64 = 1;
const MART: i64 = 2;
const GYM: i64 = 3;
const LAB: i64 = 4;

/// Three towns: a Center and a Mart; a Center, a Mart and a Gym; a Center and a lab.
const OLDALE: i64 = 1;
const PETALBURG: i64 = 2;
const LITTLEROOT: i64 = 3;
const ZONE_COUNT: i64 = 120;

/// A town's kind in the zone table.
fn town_kind(town: i64) -> i64 {
    match town {
        OLDALE => 0x06,
        PETALBURG => 0x08,
        LITTLEROOT => 0x05,
        _ => panic!("no such town"),
    }
}

type Doors = Vec<(i64, i64)>;

/// Each town's doors in from outdoors, in slot order: the zone behind, and that zone's kind. The
/// last of each is a house, whose kind is its town's.
fn buildings() -> Vec<(i64, Doors)> {
    vec![
        (OLDALE, vec![(40, 0x10), (41, 0x45), (42, 0x06)]),
        (
            PETALBURG,
            vec![(43, 0x10), (44, 0x45), (45, 0x11), (46, 0x08)],
        ),
        (LITTLEROOT, vec![(47, 0x10), (48, 0x12), (49, 0x05)]),
    ]
}

/// `{**BUILDINGS, PETALBURG: [*BUILDINGS[PETALBURG], (50, 0x77)]}`.
fn buildings_with_a_door_left_over() -> Vec<(i64, Doors)> {
    let mut all = buildings();
    for (town, doors) in &mut all {
        if *town == PETALBURG {
            doors.push((50, 0x77));
        }
    }
    all
}

type Listed = Vec<(i64, Vec<i64>)>;

fn listed() -> Listed {
    vec![
        (OLDALE, vec![CENTER, MART]),
        (PETALBURG, vec![CENTER, MART, GYM]),
        (LITTLEROOT, vec![CENTER, LAB]),
    ]
}

/// `struct.pack` of (value, width in bytes) fields, little-endian.
fn pack(fields: &[(i64, usize)]) -> Vec<u8> {
    fields
        .iter()
        .flat_map(|&(value, width)| value.to_le_bytes()[..width].to_vec())
        .collect()
}

/// `struct.pack_into`: `bytes` over `buffer` from `at`.
fn put(buffer: &mut [u8], at: i64, bytes: &[u8]) {
    buffer[at as usize..at as usize + bytes.len()].copy_from_slice(bytes);
}

/// `bytes.ljust(width, b"\0")`.
fn ljust(mut bytes: Vec<u8>, width: usize) -> Vec<u8> {
    if bytes.len() < width {
        bytes.resize(width, 0);
    }
    bytes
}

fn strings(lines: &[&str]) -> Vec<String> {
    lines.iter().map(|line| line.to_string()).collect()
}

fn text_lines(lines: &[String]) -> Vec<MessageLine> {
    lines
        .iter()
        .map(|line| MessageLine::from(line.as_str()))
        .collect()
}

/// A placeholder, a blank, two names, and a line that is nothing but a control sequence, as
/// lines 81 and 82 of the real list are.
fn places() -> Vec<MessageLine> {
    vec![
        "——————".into(),
        "".into(),
        "Mystery Zone".into(),
        "Littleroot Town".into(),
        vec![0x0010, 0x0002, 0xBDFF, 0x0050].into(),
    ]
}

fn garc_of(files: Vec<Vec<u8>>) -> Vec<u8> {
    garc(&files.into_iter().map(GarcFile::from).collect::<Vec<_>>())
}

/// `text_archive`'s keyword arguments, each None where the Python leaves it at its default.
#[derive(Default)]
struct Text {
    trainers: Option<Vec<String>>,
    places: Option<Vec<MessageLine>>,
    facilities: Option<Vec<String>>,
    pc: Option<Vec<String>>,
    layout: Option<&'static RomLayout>,
    moves: Option<Vec<String>>,
    items: Option<Vec<String>>,
    species: Option<Vec<String>>,
}

/// A language's text archive; with `pc`, the PC's own file holding those lines where the game
/// has them (`archives.PC_LINES`) and others of its own between; with `moves`, the move names;
/// with `items`, the item names; with `species`, the Pokémon's.
fn text_archive(want: Text) -> Vec<u8> {
    let layout = want.layout.unwrap_or_else(oras::rom);
    let trainers = want.trainers.unwrap_or_else(|| strings(&TRAINER_NAMES));
    let places = want.places.unwrap_or_else(places);
    let facilities = want.facilities.unwrap_or_else(|| strings(&FACILITIES));
    let pc_file = if want.pc.is_some() { layout.pc } else { 0 };
    let mut files: Vec<Vec<u8>> = vec![Vec::new(); (layout.places.max(pc_file) + 1) as usize];
    if let (Some(species), Some(file)) = (&want.species, layout.species) {
        while files.len() <= file as usize {
            files.push(Vec::new());
        }
        files[file as usize] = message_file(&text_lines(species));
    }
    if let (Some(items), Some((bike, _))) = (&want.items, layout.bike) {
        while files.len() <= bike as usize {
            files.push(Vec::new());
        }
        files[bike as usize] = message_file(&text_lines(items));
    }
    if let (Some(moves), Some(file)) = (&want.moves, layout.moves) {
        files[file as usize] = message_file(&text_lines(moves));
    }
    if let Some(pc) = &want.pc {
        let mut lines: Vec<String> = (0..PC_LINES[PC_LINES.len() - 1] * 2)
            .map(|at| if at % 2 == 0 { "Records" } else { "Power Off" }.to_string())
            .collect();
        assert_eq!(PC_LINES.len(), pc.len());
        for (&line, words) in PC_LINES.iter().zip(pc) {
            lines[line as usize] = words.clone();
        }
        files[layout.pc as usize] = message_file(&text_lines(&lines));
    }
    files[layout.trainer_names as usize] = message_file(&text_lines(&trainers));
    files[layout.places as usize] = message_file(&places);
    if let Some(towns) = &layout.towns {
        files[towns.text as usize] = message_file(&text_lines(&facilities));
    }
    files[layout.class_names as usize] = message_file(&text_lines(&strings(&CLASSES)));
    garc_of(files)
}

/// A small file a trainer, as `a/0/3/6` holds them: the class at +0x02, the battle at +0x06;
/// in X, the class at +0x01 and the battle at +0x02.
fn trainers_archive(layout: &RomLayout) -> Vec<u8> {
    let mut records: Vec<Vec<u8>> = Vec::new();
    for kind in TRAINER_CLASSES {
        let mut record = vec![0u8; 24];
        let (at, form) = &layout.trainer_class;
        let width = match form.as_str() {
            "<H" => 2,
            "<B" => 1,
            other => panic!("a format the fake ROM does not pack: {other}"),
        };
        put(&mut record, *at, &pack(&[(kind, width)]));
        record[layout.trainer_battle as usize] = if DOUBLES.contains(&kind) {
            DOUBLE as u8
        } else {
            0
        };
        records.push(record);
    }
    garc_of(records)
}

/// Every zone's data file and the zone table after them, as `a/0/1/3` holds them: a town's
/// file lists its doors, and the table says each zone's kind, parent and place.
fn zones_archive(buildings: &[(i64, Doors)], layout: &RomLayout, sections: i64) -> Vec<u8> {
    let mut table = vec![0u8; (ZONE_COUNT * INFO_SIZE) as usize];
    for zone in 0..ZONE_COUNT {
        let parent = if zone < 30 { zone } else { 0 };
        put(
            &mut table,
            zone * INFO_SIZE + INFO_PARENT,
            &pack(&[(parent, 2)]),
        );
    }
    let mut files: Vec<Vec<u8>> = vec![Vec::new(); layout.zone_table as usize + 1];
    for (town, doors) in buildings {
        let row = town * INFO_SIZE;
        put(&mut table, row + INFO_KIND, &pack(&[(town_kind(*town), 2)]));
        put(
            &mut table,
            row + INFO_NAME,
            &pack(&[(0x400 | (100 + town), 2)]),
        );
        for &(to, kind) in doors {
            put(
                &mut table,
                to * INFO_SIZE + INFO_KIND,
                &pack(&[(kind, 2), (0, 2), (0, 2)]),
            );
            put(
                &mut table,
                to * INFO_SIZE + INFO_PARENT,
                &pack(&[(*town, 2), (100 + town, 2)]),
            );
        }
        let warps: Vec<(i64, i64, i64, i64)> = doors
            .iter()
            .enumerate()
            .map(|(slot, &(to, _kind))| (to, WARP_DOOR, 10 + slot as i64, 10))
            .collect();
        let npcs: Vec<(i64, i64, i64, i64, i64)> = if *town == OLDALE {
            TRAINERS_ON_THE_MAP
                .iter()
                .enumerate()
                .map(|(at, &(sprite, trainer))| (sprite, 20 + at as i64, 20, 0, 3000 + trainer))
                .collect()
        } else {
            Vec::new()
        };
        files[*town as usize] = lz11_literals(&zone_file(&npcs, &warps, sections));
    }
    files[layout.zone_table as usize] = table;
    garc_of(files)
}

/// What each town has, as the AreaNav lists it, or X's Town Map with `towns`.
fn areanav_archive(listed: &Listed, towns: &TownFacilities) -> Vec<u8> {
    let mut rows: Vec<u8> = Vec::new();
    for (town, lines) in listed {
        let mut row = vec![0u8; towns.row as usize];
        put(&mut row, towns.zone, &pack(&[(*town, 2)]));
        for (at, line) in lines.iter().enumerate() {
            put(&mut row, towns.first + 4 * at as i64, &pack(&[(*line, 4)]));
        }
        rows.extend(row);
    }
    garc_of(vec![rows])
}

pub(crate) type Files = IndexMap<String, Vec<u8>>;

/// `{**files, name: data}`: a name already there keeps its place.
fn with(mut files: Files, name: &str, data: Vec<u8>) -> Files {
    files.insert(name.to_string(), data);
    files
}

/// The archives `GameText.load` reads, for a ROM of three towns.
pub(crate) fn oras_files(text: Option<Vec<u8>>) -> Files {
    let mut files = Files::new();
    files.insert(
        "a/0/7/3".to_string(),
        text.filter(|text| !text.is_empty())
            .unwrap_or_else(|| text_archive(Text::default())),
    );
    files.insert(
        oras::ZONES.to_string(),
        zones_archive(&buildings(), oras::rom(), 5),
    );
    files.insert(
        oras::AREANAV.0.to_string(),
        areanav_archive(&listed(), oras::towns()),
    );
    files.insert(oras::TRAINERS.to_string(), trainers_archive(oras::rom()));
    files
}

/// The PC's own lines as both games' ROMs hold them (`archives.PC_LINES`, 2026-09-24).
fn pc_lines_seen() -> Vec<(&'static str, Vec<String>)> {
    vec![
        (
            "en",
            strings(&[
                "booted up the PC!",
                "Someone\u{2019}s PC",
                "Lanette\u{2019}s PC",
                "Professor\u{2019}s PC",
            ]),
        ),
        (
            "ja",
            strings(&[
                "は\nパソコンの\u{3000}スイッチを\u{3000}いれた\u{ff01}",
                "だれかのパソコン",
                "マユミのパソコン",
                "はかせのパソコン",
            ]),
        ),
        (
            "fr",
            strings(&[
                "allume le PC.",
                "PC de ???",
                "PC d\u{2019}Annette",
                "PC du Professeur",
            ]),
        ),
        (
            "it",
            strings(&[
                "Hai acceso il PC.",
                "PC di ???",
                "PC di Lanette",
                "PC del Prof. Birch",
            ]),
        ),
        (
            "de",
            strings(&[
                "schaltet den PC ein.",
                "Jemandes PC",
                "Lanettes PC",
                "PC des Professors",
            ]),
        ),
        (
            "ko",
            strings(&["PC 전원을 켰다!", "누군가의 PC", "보은의 PC", "박사의 PC"]),
        ),
    ]
}

fn pc_lines(language: &str) -> Vec<String> {
    pc_lines_seen()
        .into_iter()
        .find(|(known, _)| *known == language)
        .unwrap()
        .1
}

/// The archives `GameText.load` reads for X, under X's numbers: the same three towns, whose
/// places are lines 101 to 103, and the Town Map listing what each has.
fn xy_files(places: Option<Vec<MessageLine>>, trainers: Option<Vec<String>>) -> Files {
    let places = places.unwrap_or_else(|| {
        let mut all = self::places();
        all.extend((0..101 - all.len()).map(|_| MessageLine::from("")));
        all.extend([
            MessageLine::from("Oldale Town"),
            MessageLine::from("Petalburg City"),
            MessageLine::from("Littleroot Town"),
        ]);
        all
    });
    let text = text_archive(Text {
        trainers,
        places: Some(places),
        pc: Some(pc_lines("en")),
        layout: Some(xy::rom()),
        ..Text::default()
    });
    let layout = xy::rom();
    let mut files = Files::new();
    files.insert(xy::text_archives()["en"].to_string(), text);
    files.insert(layout.zones.clone(), zones_archive(&buildings(), layout, 4));
    files.insert(layout.trainers.clone(), trainers_archive(layout));
    let towns = layout.towns.as_ref().expect("X has a Town Map");
    files.insert(towns.archive.clone(), areanav_archive(&listed(), towns));
    files
}

fn open_rom(tmp_path: &Path, archive_files: &Files, title: i64) -> Rom {
    let path = tmp_path.join("or.3ds");
    let files: Vec<(&str, &[u8])> = archive_files
        .iter()
        .map(|(name, data)| (name.as_str(), data.as_slice()))
        .collect();
    std::fs::write(&path, rom_image(&files, title)).unwrap();
    Rom::new(&path).unwrap()
}

/// A zone data file: `npcs` as (model code, x, z, script, the flag at +0x0a: 3000 and up is a
/// trainer) and `warps` as (to zone, kind, x, z) in cells; `sections` is ORAS's five or X's four.
/// Mirrors `test_field.zone_file`, which does the same for the field tests.
fn zone_file(
    npcs: &[(i64, i64, i64, i64, i64)],
    warps: &[(i64, i64, i64, i64)],
    sections: i64,
) -> Vec<u8> {
    let mut records: Vec<u8> = Vec::new();
    for (i, npc) in npcs.iter().enumerate() {
        records.extend(pack(&[
            (i as i64, 2),
            (npc.0, 2),
            (0, 2),
            (0, 2),
            (npc.3, 2),
            (npc.4, 2),
            (1, 2),
            (0, 2),
            (0, 2),
            (0, 2),
            (0, 2),
            (0, 2),
            (1, 2),
            (1, 2),
            (0xFFFF, 2),
            (0xFFFF, 2),
            (0xFFFF, 2),
            (0, 2),
            (0, 2),
            (0, 2),
            (npc.1, 2),
            (npc.2, 2),
            (0, 4),
        ]));
    }
    for &(to, kind, x, z) in warps {
        records.extend(pack(&[
            (to, 2),
            (0, 2),
            (kind, 2),
            (0, 2),
            (x * 18 + 9, 2),
            (0, 2),
            (z * 18 + 9, 2),
            (1, 2),
            (1, 2),
            (0, 2),
            (0, 2),
            (0, 2),
        ]));
    }
    let mut entities = pack(&[
        (8 + records.len() as i64, 4),
        (0, 1),
        (npcs.len() as i64, 1),
        (warps.len() as i64, 1),
        (0, 1),
        (0, 1),
        (0, 1),
        (0, 1),
        (0, 1),
    ]);
    entities.extend(records);
    let info = vec![0u8; INFO_SIZE as usize];
    let header = crate::games::gen6::field::zonedata::zone_header(sections);
    let mut offsets = vec![
        header,
        header + info.len() as i64,
        header + info.len() as i64 + entities.len() as i64,
    ];
    let end = *offsets.last().unwrap();
    offsets.extend(std::iter::repeat_n(end, (sections - 2) as usize));
    let mut out = b"ZO".to_vec();
    out.extend(pack(&[(sections, 2)]));
    out.extend(pack(&offsets.iter().map(|&at| (at, 4)).collect::<Vec<_>>()));
    out.extend(info);
    out.extend(entities);
    out
}

/// The file of the chunk whose north-west cell is `origin`, with the cells in `blocked` as the
/// words the game uses for a tree and the others as a path, and the other sections empty.
/// Mirrors `test_field.chunk_file`, which does the same for the field tests.
fn chunk_file(origin: (i64, i64), blocked: &[(i64, i64)], sections: i64) -> Vec<u8> {
    let mut table = pack(&[(CHUNK, 2), (CHUNK, 2)]);
    for z in 0..CHUNK {
        for x in 0..CHUNK {
            let word = if blocked.contains(&(origin.0 + x, origin.1 + z)) {
                0x01000021
            } else {
                0x20
            };
            table.extend(pack(&[(word, 4)]));
        }
    }
    let table = ljust(table, 0x1980);
    let mut offsets = vec![mapblock::HEADER];
    offsets.extend(std::iter::repeat_n(
        mapblock::HEADER + table.len() as i64,
        sections as usize,
    ));
    let mut head = mapblock::MAGIC.to_vec();
    head.extend(pack(&[(sections, 2)]));
    head.extend(pack(&offsets.iter().map(|&at| (at, 4)).collect::<Vec<_>>()));
    let mut out = ljust(head, mapblock::HEADER as usize);
    out.extend(table);
    out
}

#[test]
fn test_places_are_named_in_the_language_asked_for() {
    let tmp_path = TmpDir::new();
    let french = text_archive(Text {
        trainers: Some(strings(&[
            "-", "Brice", "Brice", "Brice", "Flora", "Flora", "Flora",
        ])),
        places: Some(vec![
            "——————".into(),
            "".into(),
            "Zone Mystère".into(),
            "Bourg-en-Vol".into(),
        ]),
        ..Text::default()
    });
    let rom = open_rom(
        tmp_path.path(),
        &with(oras_files(None), "a/0/7/4", french),
        OMEGA_RUBY,
    );
    let english = GameText::load(&rom, "en", oras::rom()).unwrap();
    let francais = GameText::load(&rom, "fr", oras::rom()).unwrap();
    assert_eq!(english.place(3), Some("Littleroot Town".to_string()));
    assert_eq!(francais.place(3), Some("Bourg-en-Vol".to_string()));
}

#[test]
fn test_no_name_is_made_of_a_placeholder_a_blank_or_a_control() {
    let text = GameText::new(
        strings(&["——————", "", "Mystery Zone", ""]),
        strings(&TRAINERS),
    );
    assert_eq!(text.place(0), None);
    assert_eq!(text.place(1), None);
    assert_eq!(text.place(2), Some("Mystery Zone".to_string()));
    assert_eq!(text.place(3), None);
    assert_eq!(text.place(99), None);
}

#[test]
fn test_a_line_of_controls_alone_says_nothing() {
    let tmp_path = TmpDir::new();
    let rom = open_rom(tmp_path.path(), &oras_files(None), OMEGA_RUBY);
    assert_eq!(
        GameText::load(&rom, "en", oras::rom()).unwrap().place(4),
        None
    );
}

/// Spike 89: the rival's work object is filled with SUPPORT_F_RS as May and SUPPORT_M_RS as
/// Brendan, and their bikes after them; the slot itself is nobody.
#[test]
fn test_the_rival_is_named_by_the_sprite_they_are_drawn_with() {
    let tmp_path = TmpDir::new();
    let rom = open_rom(tmp_path.path(), &oras_files(None), OMEGA_RUBY);
    let text = GameText::load(&rom, "en", oras::rom()).unwrap();
    let may = Some("May".to_string());
    let brendan = Some("Brendan".to_string());
    assert_eq!(
        [0x13A, 0x13C].map(|sprite| text.character(sprite)),
        [may.clone(), may]
    );
    assert_eq!(
        [0x13B, 0x13D].map(|sprite| text.character(sprite)),
        [brendan.clone(), brendan]
    );
    assert_eq!(text.character(0x7A), None);
    assert_eq!(text.character(0xAC), None);
}

/// Norman's sprite is trainer 570 in the ROM's own list, in every language; a trainer the list
/// does not reach is left to the shipped table rather than read past its end.
#[test]
fn test_a_person_is_named_by_the_trainer_their_sprite_is() {
    let tmp_path = TmpDir::new();
    let mut names = strings(&TRAINER_NAMES);
    names.extend((0..570 - TRAINER_NAMES.len()).map(|_| String::new()));
    names.push("Norman".to_string());
    let text = text_archive(Text {
        trainers: Some(names),
        ..Text::default()
    });
    let rom = open_rom(tmp_path.path(), &oras_files(Some(text)), OMEGA_RUBY);
    let text = GameText::load(&rom, "en", oras::rom()).unwrap();
    assert_eq!(text.character(0x13F), Some("Norman".to_string()));
    assert_eq!(text.character(0xBF), Some("Norman".to_string()));
    assert_eq!(text.character(0xBB), None);
}

/// Spike 90: two Youngsters make the sprite a Youngster, and grunts who share a name are said
/// as the game says them in battle. Twins are a pair, two people, and say nothing of a sprite,
/// as two Mysterious Sisters said nothing of the Fairy Tale Girls' (spike 193); one Lass is too few
/// to go on, and a sprite standing for a Hiker and a Youngster could be either.
#[test]
fn test_a_sprite_trainers_stand_as_is_named_for_their_class() {
    let tmp_path = TmpDir::new();
    let rom = open_rom(tmp_path.path(), &oras_files(None), OMEGA_RUBY);
    let text = GameText::load(&rom, "en", oras::rom()).unwrap();
    assert_eq!(text.character(0xDA), Some("Youngster".to_string()));
    assert_eq!(text.character(0xC7), Some("Team Magma Grunt".to_string()));
    assert_eq!(text.character(0x10A), None);
    assert_eq!(text.character(0xDB), None);
    assert_eq!(text.character(0xE9), None);
}

/// Spike 193: the character 0x1000 + n is drawn with the field's model 0x1BE + n, whose file
/// names the Pokémon it draws, packed or not: Machoke with the movers, Whismur beside Zinnia. A
/// model naming two Pokémon is left out.
#[test]
fn test_a_pokemon_on_the_map_is_named_by_the_pokemon_its_model_draws() {
    let tmp_path = TmpDir::new();
    let mut species = vec![String::new(); 300];
    species[67] = "Machoke".to_string();
    species[293] = "Whismur".to_string();
    let (archive, first) = oras::POKEMON_MODELS;
    let mut models: Vec<Vec<u8>> = vec![Vec::new(); first as usize];
    models.push(b"MM\x02\x00pm0067_00_fi".to_vec());
    models.push(lz11_literals(b"MMpm0293_00_fi_body"));
    models.push(b"MMpm0032_00_fipm0320_00_fi_body".to_vec());
    let mut files = oras_files(Some(text_archive(Text {
        species: Some(species),
        ..Text::default()
    })));
    files.insert(archive.to_string(), garc_of(models));
    let rom = open_rom(tmp_path.path(), &files, OMEGA_RUBY);
    let text = GameText::load(&rom, "en", oras::rom()).unwrap();
    assert_eq!(
        (0..3)
            .map(|n| text.character(0x1000 + n))
            .collect::<Vec<_>>(),
        [
            Some("Machoke".to_string()),
            Some("Whismur".to_string()),
            None
        ]
    );
}

/// Spike 218: X tags its Kalos Pokémon's models apart from their numbers, Fletchling (661)
/// `pm0753`; the battle models say which species each tag is, the last of a model's eight files
/// tagged. A tag no species has names nothing.
#[test]
fn test_a_tag_that_is_not_a_national_number_is_named_through_the_battle_models() {
    let tmp_path = TmpDir::new();
    let mut species = vec![String::new(); 700];
    species[67] = "Machoke".to_string();
    species[661] = "Fletchling".to_string();
    let mut head = vec![0u8; 4 * 661];
    head[4 * 66..4 * 66 + 3].copy_from_slice(&[0, 0, 1]);
    head[4 * 660..4 * 660 + 3].copy_from_slice(&[2, 0, 1]);
    let mut battle: Vec<Vec<u8>> = vec![head];
    battle.extend(vec![Vec::new(); 24]);
    battle[8] = b"pm0067_00".to_vec();
    battle[24] = b"pm0753_00".to_vec();
    let (archive, first) = oras::POKEMON_MODELS;
    let mut models: Vec<Vec<u8>> = vec![Vec::new(); first as usize];
    models.push(b"MMpm0753_00_fi".to_vec());
    models.push(b"MMpm0067_00_fi".to_vec());
    models.push(b"MMpm0661_00_fi".to_vec());
    let mut files = oras_files(Some(text_archive(Text {
        species: Some(species),
        ..Text::default()
    })));
    files.insert(archive.to_string(), garc_of(models));
    files.insert("a/0/0/7".to_string(), garc_of(battle));
    let rom = open_rom(tmp_path.path(), &files, OMEGA_RUBY);
    let layout = RomLayout {
        battle_models: Some("a/0/0/7".to_string()),
        ..oras::rom().clone()
    };
    let text = GameText::load(&rom, "en", &layout).unwrap();
    assert_eq!(
        (0..3)
            .map(|n| text.character(0x1000 + n))
            .collect::<Vec<_>>(),
        [
            Some("Fletchling".to_string()),
            Some("Machoke".to_string()),
            None
        ]
    );
}

/// A Chinese patch of Omega Ruby ends each of the six names with a zero and another halfword
/// after it, a different one each time (rust-pokemon-access-3ds#34).
#[test]
fn test_trainer_names_ended_early_by_a_zero_are_still_the_protagonists() {
    let tmp_path = TmpDir::new();
    let mut names = strings(&TRAINER_NAMES);
    for n in 1..7u32 {
        names[n as usize].push('\0');
        names[n as usize].push(char::from_u32(0xB000 + n).unwrap());
    }
    let text = text_archive(Text {
        trainers: Some(names),
        ..Text::default()
    });
    let rom = open_rom(tmp_path.path(), &oras_files(Some(text)), OMEGA_RUBY);
    let text = GameText::load(&rom, "en", oras::rom()).unwrap();
    assert_eq!(
        [0x13A, 0x13B].map(|sprite| text.character(sprite)),
        [Some("May".to_string()), Some("Brendan".to_string())]
    );
}

/// The file numbers are a fact about ORAS; a ROM they do not fit is refused, not misread.
#[test]
fn test_a_trainer_list_that_is_not_the_protagonists_is_turned_away() {
    let tmp_path = TmpDir::new();
    let shuffled = text_archive(Text {
        trainers: Some(strings(&[
            "-", "Brendan", "May", "Brendan", "May", "Brendan", "May",
        ])),
        ..Text::default()
    });
    let rom = open_rom(tmp_path.path(), &oras_files(Some(shuffled)), OMEGA_RUBY);
    let error = GameText::load(&rom, "en", oras::rom()).unwrap_err();
    assert!(error.is(Kind::RomError) && error.message.contains("trainer names"));
}

#[test]
fn test_a_language_the_game_does_not_carry_is_a_rom_error() {
    let tmp_path = TmpDir::new();
    let rom = open_rom(tmp_path.path(), &oras_files(None), OMEGA_RUBY);
    let error = GameText::load(&rom, "pt", oras::rom()).unwrap_err();
    assert!(error.is(Kind::RomError) && error.message.contains("pt"));
    let error = GameText::load(&rom, "fr", oras::rom()).unwrap_err();
    assert!(error.is(Kind::RomError));
}

#[test]
fn test_what_is_not_an_archive_is_a_rom_error() {
    assert!(garc_files(b"ZO\x05\x00").unwrap_err().is(Kind::RomError));
    assert!(
        message_lines(b"\x01\x00\x09\x00")
            .unwrap_err()
            .is(Kind::RomError)
    );
}

/// The name insert's argument 0 ends "Haus von <name>": only the padding after it goes.
#[test]
fn test_a_line_ending_on_a_control_keeps_its_zero_arguments() {
    let mut insert: Vec<i64> = "Haus von ".chars().map(|c| c as i64).collect();
    insert.extend([0x10, 2, 0x100, 0]);
    let padded: Vec<i64> = insert.iter().copied().chain([0, 0]).collect();
    let packed: Vec<u8> = insert
        .iter()
        .flat_map(|&unit| (unit as u16).to_le_bytes())
        .collect();
    let hi: Vec<u8> = "Hi".encode_utf16().flat_map(u16::to_le_bytes).collect();
    assert_eq!(
        message_lines(&message_file(&[
            MessageLine::from(insert),
            MessageLine::from(padded),
            MessageLine::from("Hi"),
        ]))
        .unwrap(),
        [packed.clone(), packed, hi]
    );
}

/// The Center's kind is in three towns and the Mart's in two, so both are shared; Petalburg's
/// Gym and Littleroot's lab are what is left in their towns; a house, whose kind is its town's,
/// is nobody's to name.
#[test]
fn test_doors_the_rom_names_and_the_ones_it_does_not() {
    let tmp_path = TmpDir::new();
    let rom = open_rom(tmp_path.path(), &oras_files(None), OMEGA_RUBY);
    let text = GameText::load(&rom, "en", oras::rom()).unwrap();
    let some = |name: &str| Some(name.to_string());
    assert_eq!(
        (0..3)
            .map(|slot| text.door(OLDALE, slot))
            .collect::<Vec<_>>(),
        [some("Pokémon Center"), some("Poké Mart"), None]
    );
    assert_eq!(
        (0..4)
            .map(|slot| text.door(PETALBURG, slot))
            .collect::<Vec<_>>(),
        [
            some("Pokémon Center"),
            some("Poké Mart"),
            some("Petalburg City Gym"),
            None
        ]
    );
    assert_eq!(
        (0..3)
            .map(|slot| text.door(LITTLEROOT, slot))
            .collect::<Vec<_>>(),
        [some("Pokémon Center"), some("Pokémon Lab"), None]
    );
    assert_eq!(text.door(OLDALE, 9), None);
}

/// Dewford's Gym and Dewford Hall, as the real ROM has it: two doors left for one facility.
/// The Gym's room is a Gym's by its kind, and the Gym the first facility only its town lists;
/// which the other door is, is not the ROM's to say, so it is not guessed.
#[test]
fn test_two_doors_left_for_one_facility_name_only_the_gyms() {
    let tmp_path = TmpDir::new();
    let files = with(
        oras_files(None),
        oras::ZONES,
        zones_archive(&buildings_with_a_door_left_over(), oras::rom(), 5),
    );
    let rom = open_rom(tmp_path.path(), &files, OMEGA_RUBY);
    let text = GameText::load(&rom, "en", oras::rom()).unwrap();
    assert_eq!(
        text.door(PETALBURG, 2),
        Some("Petalburg City Gym".to_string())
    );
    assert_eq!(text.door(PETALBURG, 4), None);
    assert_eq!(text.door(PETALBURG, 0), Some("Pokémon Center".to_string()));
}

#[test]
fn test_doors_are_named_in_the_language_asked_for() {
    let tmp_path = TmpDir::new();
    let japanese = text_archive(Text {
        facilities: Some(strings(&[
            "",
            "ポケモンセンター",
            "フレンドリィショップ",
            "トウカシティジム",
            "ポケモンけんきゅうじょ",
        ])),
        ..Text::default()
    });
    let rom = open_rom(
        tmp_path.path(),
        &with(oras_files(None), "a/0/7/1", japanese),
        OMEGA_RUBY,
    );
    let text = GameText::load(&rom, "ja", oras::rom()).unwrap();
    assert_eq!(
        text.door(PETALBURG, 2),
        Some("トウカシティジム".to_string())
    );
}

#[test]
fn test_a_zone_table_that_is_not_one_is_turned_away() {
    let tmp_path = TmpDir::new();
    let mut files: Vec<Vec<u8>> = vec![Vec::new(); oras::ZONE_TABLE as usize];
    files.push(vec![0u8; 56]);
    let files = with(oras_files(None), oras::ZONES, garc_of(files));
    let rom = open_rom(tmp_path.path(), &files, OMEGA_RUBY);
    let error = GameText::load(&rom, "en", oras::rom()).unwrap_err();
    assert!(error.is(Kind::RomError) && error.message.contains("zone table"));
}

/// Nothing labels the PC where it stands, but booting it and the three PCs it offers all name
/// it, spaced or not, before a particle or after one.
#[test]
fn test_the_pc_is_the_word_its_own_lines_all_say() {
    for (language, lines) in pc_lines_seen() {
        assert_eq!(
            archives::shared_word(&lines),
            Some(
                if language == "ja" {
                    "パソコン"
                } else {
                    "PC"
                }
                .to_string()
            ),
            "{language}"
        );
    }
}

#[test]
fn test_lines_sharing_no_word_share_nothing() {
    assert_eq!(
        archives::shared_word(&strings(&["a cat", "the dog", "an owl"])),
        None
    );
    assert_eq!(archives::shared_word(&[]), None);
}

#[test]
fn test_the_pc_is_called_what_the_roms_own_lines_call_it() {
    let tmp_path = TmpDir::new();
    let french = text_archive(Text {
        trainers: Some(strings(&[
            "-", "Brice", "Brice", "Brice", "Flora", "Flora", "Flora",
        ])),
        pc: Some(pc_lines("fr")),
        ..Text::default()
    });
    let rom = open_rom(
        tmp_path.path(),
        &with(oras_files(None), "a/0/7/4", french),
        OMEGA_RUBY,
    );
    assert_eq!(
        GameText::load(&rom, "fr", oras::rom()).unwrap().pc,
        Some("PC".to_string())
    );
    assert_eq!(GameText::load(&rom, "en", oras::rom()).unwrap().pc, None);
}

#[test]
fn test_the_moves_are_called_what_the_roms_move_names_call_them() {
    let tmp_path = TmpDir::new();
    let mut moves: Vec<String> = vec!["-".to_string(); SURF as usize];
    moves.extend(strings(&["Surf", "Force"]));
    moves[CUT as usize] = "Coupe".to_string();
    let french = text_archive(Text {
        trainers: Some(strings(&[
            "-", "Brice", "Brice", "Brice", "Flora", "Flora", "Flora",
        ])),
        moves: Some(moves),
        ..Text::default()
    });
    let rom = open_rom(
        tmp_path.path(),
        &with(oras_files(None), "a/0/7/4", french),
        OMEGA_RUBY,
    );
    let text = GameText::load(&rom, "fr", oras::rom()).unwrap();
    assert_eq!(
        text.moves,
        IndexMap::from([(CUT, "Coupe".to_string()), (SURF, "Surf".to_string())]),
        "Rock Smash's line is past the file"
    );
    let english = GameText::load(&rom, "en", oras::rom()).unwrap();
    assert_eq!(english.moves, IndexMap::<i64, String>::new());
}

#[test]
fn test_the_bike_is_called_what_the_roms_item_names_call_the_mach_bike() {
    let tmp_path = TmpDir::new();
    let mut items: Vec<String> = vec!["-".to_string(); oras::MACH_BIKE.1 as usize];
    items.extend(strings(&["Vélo Course", "Vélo Cross"]));
    let french = text_archive(Text {
        trainers: Some(strings(&[
            "-", "Brice", "Brice", "Brice", "Flora", "Flora", "Flora",
        ])),
        items: Some(items),
        ..Text::default()
    });
    let rom = open_rom(
        tmp_path.path(),
        &with(oras_files(None), "a/0/7/4", french),
        OMEGA_RUBY,
    );
    assert_eq!(
        GameText::load(&rom, "fr", oras::rom()).unwrap().bike,
        Some("Vélo Course".to_string())
    );
    assert_eq!(GameText::load(&rom, "en", oras::rom()).unwrap().bike, None);
}

#[test]
fn test_a_chunk_files_cell_table_is_read_alone_from_the_map_chunks() {
    let tmp_path = TmpDir::new();
    let town = chunk_file((80, 160), &[(80, 160)], 7);
    let path = tmp_path.path().join("or.3ds");
    let archive = garc_of(vec![b"not a chunk".to_vec(), lz11_literals(&town)]);
    std::fs::write(
        &path,
        rom_image(&[(oras::MAP_CHUNKS, &archive)], oras::OMEGA_RUBY),
    )
    .unwrap();
    let chunks = MapChunks::new(&path, oras::MAP_CHUNKS);
    let cells = chunks.cells(1).unwrap();
    assert_eq!(cells, mapblock::cells(&town).unwrap());
    let cells = cells.unwrap();
    assert_eq!(cells[..2], [0x01000021, 0x20]);
    assert!(chunks.cells(0).unwrap().is_none() && chunks.cells(2).unwrap().is_none());
    assert!(
        MapChunks::new(&tmp_path.path().join("missing.3ds"), oras::MAP_CHUNKS)
            .cells(1)
            .unwrap()
            .is_none()
    );
}

/// Spike 141: the berry spots are a table of the field's DLL, found by its shape; a short run
/// of the same shape is not it, and a ROM without the DLL has none.
#[test]
fn test_the_berry_spots_are_the_longest_run_of_their_shape_in_the_field_dll() {
    let tmp_path = TmpDir::new();
    let spots: Vec<(i64, (i64, i64))> = (0..40).map(|i| (0x18 + i % 3, (100 + i, 50))).collect();
    let table: Vec<u8> = spots
        .iter()
        .flat_map(|&(zone, (x, z))| BERRY_SPOT.pack([x, z, -34, zone, 0x218]).unwrap())
        .collect();
    let short: Vec<u8> = (0..10)
        .flat_map(|_| BERRY_SPOT.pack([1, 2, 0, 5, 7]).unwrap())
        .collect();
    let mut dll = b"CRO0".to_vec();
    dll.extend(vec![0u8; 0x3E]);
    dll.extend(&short);
    dll.extend([0xFFu8; 6]);
    dll.extend(&table);
    dll.extend(vec![0u8; 20]);
    assert_eq!(archives::berry_spots(&dll), spots);
    let mut brief = b"CRO0".to_vec();
    brief.extend(&short);
    assert_eq!(archives::berry_spots(&brief), []);
    let rom = open_rom(
        tmp_path.path(),
        &with(oras_files(None), oras::BERRY_SPOTS, dll),
        OMEGA_RUBY,
    );
    assert_eq!(
        GameText::load(&rom, "en", oras::rom()).unwrap().berries,
        spots
    );
    drop(rom);
    let rom = open_rom(tmp_path.path(), &oras_files(None), OMEGA_RUBY);
    assert_eq!(GameText::load(&rom, "en", oras::rom()).unwrap().berries, []);
}

/// Spikes 100 and 101: X's places, trainers, PC and Town Map are the same kinds of file as
/// ORAS's under other numbers, and a trainer's class is a byte a place sooner; its Town Map names
/// the buildings behind doors as ORAS's AreaNav does.
#[test]
fn test_x_keeps_the_same_words_under_numbers_of_its_own() {
    let tmp_path = TmpDir::new();
    let rom = open_rom(tmp_path.path(), &xy_files(None, None), xy::X);
    let text = GameText::load(&rom, "en", xy::rom()).unwrap();
    assert_eq!(text.place(103), Some("Littleroot Town".to_string()));
    assert_eq!(text.character(0xDA), Some("Youngster".to_string()));
    assert_eq!(text.character(0x10A), None);
    assert_eq!(text.pc, Some("PC".to_string()));
    let some = |name: &str| Some(name.to_string());
    assert_eq!(
        (0..3)
            .map(|slot| text.door(OLDALE, slot))
            .collect::<Vec<_>>(),
        [some("Pokémon Center"), some("Poké Mart"), None]
    );
}

/// Spike 108: X's LEADER1 is Viola, trainer 6, and the rival drawn as RIVALW or RIVALW_EX is
/// Serena, trainer 575; ORAS's Norman sprite means nothing in X.
#[test]
fn test_x_names_its_story_people_by_its_own_sprites() {
    let tmp_path = TmpDir::new();
    let mut trainers = strings(&TRAINER_NAMES);
    trainers.extend((0..575 - TRAINER_NAMES.len()).map(|_| String::new()));
    trainers.push("Serena".to_string());
    trainers[6] = "Viola".to_string();
    let rom = open_rom(tmp_path.path(), &xy_files(None, Some(trainers)), xy::X);
    let text = GameText::load(&rom, "en", xy::rom()).unwrap();
    assert_eq!(text.character(0x0F), Some("Viola".to_string()));
    let serena = Some("Serena".to_string());
    assert_eq!(
        [0x94, 0xA0].map(|sprite| text.character(sprite)),
        [serena.clone(), serena]
    );
    assert_eq!(text.character(0xBF), None);
}

/// Spike 107: in X every Gym town's row lists its Gym just before the Pokémon Center, the
/// facility the most towns share, and a Gym's room has a kind of its own; so a warp into one is
/// named so even where its town has another facility left over, as Shalour has.
#[test]
fn test_a_gym_is_named_for_the_line_its_town_map_row_lists_before_the_center() {
    let tmp_path = TmpDir::new();
    let mut rows = listed();
    for (town, lines) in &mut rows {
        if *town == PETALBURG {
            *lines = vec![GYM, CENTER, MART, LAB];
        }
    }
    let layout = xy::rom();
    let towns = layout.towns.as_ref().expect("X has a Town Map");
    let files = with(
        with(
            xy_files(None, None),
            &layout.zones,
            zones_archive(&buildings_with_a_door_left_over(), layout, 4),
        ),
        &towns.archive,
        areanav_archive(&rows, towns),
    );
    let rom = open_rom(tmp_path.path(), &files, xy::X);
    let text = GameText::load(&rom, "en", xy::rom()).unwrap();
    assert_eq!(
        text.door(PETALBURG, 2),
        Some("Petalburg City Gym".to_string())
    );
    assert_eq!(text.door(PETALBURG, 4), None);
    assert_eq!(text.door(PETALBURG, 0), Some("Pokémon Center".to_string()));
}

/// X's trainer names do not open with the protagonists, so its text is checked by its place
/// names instead: a list too short for the towns' places is not X's.
#[test]
fn test_a_place_list_the_zone_table_does_not_fit_is_turned_away() {
    let tmp_path = TmpDir::new();
    let rom = open_rom(tmp_path.path(), &xy_files(Some(places()), None), xy::X);
    let error = GameText::load(&rom, "en", xy::rom()).unwrap_err();
    assert!(error.is(Kind::RomError) && error.message.contains("place names"));
}

/// Spike 106: the game loads a line of Lumiose's streets only with the chunks round the
/// player, and the ROM keeps every chunk's streets in section 4 of its file.
#[test]
fn test_the_streets_of_a_maps_chunks_are_read_out_of_the_rom() {
    let tmp_path = TmpDir::new();
    let line = rail::Line::new(C, P3, P2, 5, ACROSS, walls(5, &[]));
    let sections: Vec<Vec<u8>> = vec![
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        street_section(std::slice::from_ref(&line), &[], &[]),
        Vec::new(),
    ];
    let mut offsets: Vec<i64> = Vec::new();
    let mut at = 8 + 4 * (sections.len() as i64 + 1);
    for section in &sections {
        offsets.push(at);
        at += section.len() as i64;
    }
    offsets.push(at);
    let mut chunk = b"GR".to_vec();
    chunk.extend(pack(&[(sections.len() as i64, 2)]));
    chunk.extend(pack(&offsets.iter().map(|&at| (at, 4)).collect::<Vec<_>>()));
    let padding = 8 + 4 * (sections.len() + 1) - chunk.len();
    chunk.extend(vec![0u8; padding]);
    chunk.extend(sections.concat());
    let mut empty = b"GR".to_vec();
    empty.extend(pack(&[(6, 2)]));
    empty.extend(pack(&[(32, 4); 7]));
    empty.extend([0u8; 4]);
    let path = tmp_path.path().join("x.3ds");
    let archive = garc_of(vec![empty, lz11_literals(&chunk)]);
    std::fs::write(
        &path,
        rom_image(&[("a/0/4/1", &archive)], 0x0004000000055D00),
    )
    .unwrap();
    let streets = Streets::new(&path, "a/0/4/1");
    assert_eq!(
        streets.network(&[0, 1, 7]).unwrap(),
        rail::Network {
            lines: IndexMap::from([(C, line)]),
            places: IndexMap::new(),
            cameras: 0,
            ..Default::default()
        }
    );
    assert_eq!(
        Streets::new(&tmp_path.path().join("gone.3ds"), "a/0/4/1")
            .network(&[1])
            .unwrap(),
        rail::Network {
            lines: IndexMap::new(),
            places: IndexMap::new(),
            cameras: 0,
            ..Default::default()
        }
    );
}

/// Sun's archives are all version 0x0600, and read from its largest file's size, as the data
/// offset once was, every text file was garbage (spike 111).
#[test]
fn test_a_version_6_archive_s_files_are_read_from_where_its_header_says() {
    let files: Vec<Vec<u8>> = vec![
        message_file(&[MessageLine::from("Play the game in")]),
        message_file(&[MessageLine::from("English"), MessageLine::from("Spanish")]),
    ];
    let archive = garc_with(
        &files
            .iter()
            .cloned()
            .map(GarcFile::from)
            .collect::<Vec<_>>(),
        0x0600,
    );
    assert_eq!(garc_files(&archive).unwrap(), files);
}

/// The map's archive opens with its files' names and then its panes'; an AreaNav row keeps
/// its zone's touch area among those at +0x04, and the zone names it (spike 152: Route 101 is
/// `m`). Rows sharing a touch area, as a cave's floors do, name it by the first.
#[test]
fn test_the_maps_touch_areas_are_named_by_the_areanav_row_pointing_at_them() {
    let mut head = pack(&[(1, 4)]);
    head.extend(ljust(b"map_lower.bclyt".to_vec(), 64));
    head.extend(pack(&[(3, 4)]));
    for name in [b"l", b"m", b"n"] {
        head.extend(ljust(name.to_vec(), 32));
    }
    let towns = oras::towns();
    let mut nav: Vec<u8> = Vec::new();
    for (zone, pane) in [(6, 0), (23, 1), (24, 1), (99, 7)] {
        nav.extend(ljust(
            pack(&[(zone, 2), (zone, 2), (pane, 2)]),
            towns.row as usize,
        ));
    }
    let names: IndexMap<i64, &str> = IndexMap::from([
        (6, "Littleroot Town"),
        (23, "Route 101"),
        (24, "Route 101 again"),
    ]);
    let place = |zone: i64| names.get(&zone).map(|name| name.to_string());
    assert_eq!(
        archives::map_places(&head, &nav, towns, 0x04, &place).unwrap(),
        IndexMap::from([
            ("l".to_string(), "Littleroot Town".to_string()),
            ("m".to_string(), "Route 101".to_string())
        ])
    );
    assert!(
        archives::map_places(&head[..40], &nav, towns, 0x04, &place)
            .unwrap_err()
            .is(Kind::RomError)
    );
}

#[test]
fn test_each_species_other_forms_are_named_in_turn_after_every_first_form() {
    // Egg, Unown with three forms, Latios and Groudon with two; a line between, then the forms.
    let names = strings(&[
        "",
        "One form",
        "Latios",
        "Groudon",
        "",
        "One form",
        "One form",
        "Mega Latios",
        "Primal Reversion",
    ]);
    let counts = [1, 3, 2, 2];
    assert_eq!(
        archives::form_names(&names, &counts),
        IndexMap::from([
            ((2, 1), "Mega Latios".to_string()),
            ((3, 1), "Primal Reversion".to_string())
        ])
    );
    assert_eq!(
        archives::form_names(&names[..names.len() - 1], &counts),
        IndexMap::<(i64, i64), String>::new()
    );
}

#[test]
fn test_a_form_is_named_as_the_game_names_it_with_its_species_where_it_does_not() {
    let text = GameText {
        species: strings(&["Egg", "Unown", "Latios", "Groudon"]),
        forms: IndexMap::from([
            ((2, 1), "Mega Latios".to_string()),
            ((3, 1), "Primal Reversion".to_string()),
        ]),
        ..GameText::new(strings(&["Littleroot Town"]), strings(&TRAINERS))
    };
    assert_eq!(text.pokemon(2, 1).as_deref(), Some("Mega Latios"));
    assert_eq!(
        text.pokemon(3, 1).as_deref(),
        Some("Primal Reversion Groudon")
    );
    assert_eq!(text.pokemon(3, 0).as_deref(), Some("Groudon"));
    assert_eq!(text.pokemon(1, 2).as_deref(), Some("Unown"));
    assert_eq!(text.pokemon(0, 0), None);
}
