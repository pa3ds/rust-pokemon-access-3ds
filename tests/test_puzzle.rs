//! Tile Puzzle's tiles are said by their place and where the piece each shows belongs.

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{FakeClasses, FakeMemory};
use super::test_memories::{record, record_form};
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::screens::puzzle;

const SCREEN: i64 = 0x08300000;
const BOARD: i64 = 0x08400000;
const TILES: i64 = 0x08310000;
const DIGITS: &str = "0123456789abcdefghijklmnopqrstuvwxyz";

fn base36(number: i64) -> String {
    let digit = DIGITS.as_bytes()[(number % 36) as usize] as char;
    if number >= 36 {
        format!("{}{digit}", base36(number / 36))
    } else {
        digit.to_string()
    }
}

/// A board whose piece belonging in slot i, numbered from len(slots) down, lies in
/// `slots[i]`, under the touch area numbered `first` more; and each slot's group as the game
/// builds it, its frame picture beside its touch area. Returns the reader, the pictures and the
/// touch areas.
fn board(
    slots: &[i64],
    picked: Option<i64>,
    first: i64,
) -> (puzzle::TilePuzzle, Vec<i64>, Vec<i64>) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let screens = FakeLayouts::new(&memory, &classes);
    let bounding = classes.add(lyt::BOUNDING_CLASS);
    let (_, screen) = screens.screen(SCREEN);
    let mut frames: Vec<i64> = Vec::new();
    let mut areas: Vec<i64> = Vec::new();
    for slot in 0..slots.len() as i64 {
        let group = TILES + slot * 0x300;
        screens.pane(group, screen);
        screens.pane(group + 0x100, group);
        screens.pane_with_vtable(group + 0x200, group, Some(bounding));
        memory.write(
            group + 0x200 + lyt::ORAS.name,
            base36(first + slot).as_bytes(),
        );
        frames.push(group + 0x100);
        areas.push(group + 0x200);
    }
    screens.track().unwrap();
    memory.u32(BOARD, &[classes.add(puzzle::CLASS)]);
    let pieces = slots.len() as i64;
    for (i, &slot) in slots.iter().enumerate() {
        let at = BOARD + puzzle::RECORDS + i as i64 * puzzle::SIZE;
        memory.u32(
            at + puzzle::STATE,
            &[if Some(slot) == picked {
                puzzle::PICKED
            } else {
                1
            }],
        );
        memory.u32(at + puzzle::NUMBER, &[pieces - i as i64]);
        memory.u32(at + puzzle::SLOT, &[first + slot]);
    }
    let mut reader = puzzle::TilePuzzle::new(memory.clone(), screens.layouts.clone());
    reader
        .track(&IndexMap::from([(BOARD, puzzle::CLASS.to_string())]), false)
        .unwrap();
    (reader, frames, areas)
}

fn name(reader: &puzzle::TilePuzzle, picture: i64) -> Option<String> {
    reader.name(picture).unwrap()
}

#[test]
fn test_a_tile_says_its_place_and_where_its_piece_belongs() {
    // as_amie_tile_puzzle: after the shuffle the top-left tile held the piece of row 3, column 5,
    // and the picked tile said so (spike 254).
    let mut slots: Vec<i64> = (0..20).collect();
    (slots[14], slots[0]) = (0, 14);
    let (reader, frames, _) = board(&slots, Some(0), 0);
    assert_eq!(
        name(&reader, frames[0]).as_deref(),
        Some("A1, goes to E3, picked")
    );
    assert_eq!(name(&reader, frames[14]).as_deref(), Some("E3, goes to A1"));
    assert_eq!(name(&reader, frames[7]).as_deref(), Some("C2, in place"));
}

#[test]
fn test_normals_thirty_tiles_lie_under_touch_areas_numbered_from_955() {
    // as_amie_tile_puzzle_normal: six across and five down, under touch areas `qj` to `rc`; the
    // first tap, at the bottom right, picked the piece numbered 26, and the second put it in row
    // 2, column 6 (spike 254).
    let mut slots: Vec<i64> = (0..30).collect();
    (slots[4], slots[29]) = (29, 4);
    let (reader, frames, _) = board(&slots, Some(29), 955);
    assert_eq!(
        name(&reader, frames[29]).as_deref(),
        Some("F5, goes to E1, picked")
    );
    assert_eq!(name(&reader, frames[4]).as_deref(), Some("E1, goes to F5"));
    assert_eq!(name(&reader, frames[11]).as_deref(), Some("F2, in place"));
}

#[test]
fn test_hards_forty_eight_tiles_are_named_at_either_end_of_a_row() {
    // as_amie_tile_puzzle_hard: eight across and six down, under `ii` to `jt`; placed by their
    // middles, the first and last columns were "frame" (Orion, 2026-10-27).
    let mut slots: Vec<i64> = (0..48).collect();
    (slots[0], slots[47]) = (47, 0);
    let (reader, frames, _) = board(&slots, Some(47), 666);
    assert_eq!(name(&reader, frames[0]).as_deref(), Some("A1, goes to H6"));
    assert_eq!(
        name(&reader, frames[47]).as_deref(),
        Some("H6, goes to A1, picked")
    );
    assert_eq!(name(&reader, frames[15]).as_deref(), Some("H2, in place"));
}

#[test]
fn test_the_player_plays_while_a_piece_is_away_from_home() {
    // The board lies whole until Begin shuffles it (or_amie_tile_puzzle_play_again).
    let mut slots: Vec<i64> = (0..20).collect();
    assert!(!board(&slots, None, 0).0.playing().unwrap());
    (slots[14], slots[0]) = (0, 14);
    assert!(board(&slots, None, 0).0.playing().unwrap());
}

#[test]
fn test_records_out_of_order_or_two_in_one_slot_are_no_board() {
    // The module's own data held words like a Piece's vtable (spike 254).
    let mut slots: Vec<i64> = (0..20).collect();
    slots[3] = slots[4];
    let (reader, frames, _) = board(&slots, None, 0);
    assert!(reader.board().unwrap().is_none());
    assert_eq!(name(&reader, frames[0]), None);
}

const PICTURE: i64 = 0x08500000;
const PARAMS: i64 = 0x08600000;
const BACKGROUND: i64 = 0x08700000;
const CODE: i64 = 0x0804F354;
/// Where or_amie_tile_puzzle_play_again's first background's image was.
const IMAGE: i64 = 0x14FAD080;
/// Where or_amie_tile_puzzle_easy's three stood: right and behind, in front, left and behind.
const STANDING: [(f32, f32); 3] = [(170.0, -335.0), (0.0, -200.0), (-170.0, -335.0)];

/// A picture of a Pokémon a record, each a PokemonParam pointing to its record at +0x40, in
/// the places or_amie_tile_puzzle_easy's stood, or all at 0 unless `placed`, before `background`,
/// its shallow water by default (spike 256), its image loaded, found after a word of the module's
/// code like the background's vtable; and an Easy board, every piece home, as under Begin.
fn picture(records: &[Vec<u8>], placed: bool) -> puzzle::TilePuzzle {
    picture_before(records, placed, 12)
}

/// `picture(records, placed, background)`.
fn picture_before(records: &[Vec<u8>], placed: bool, background: i64) -> puzzle::TilePuzzle {
    picture_in(records, placed, background, FakeMemory::new())
}

/// `picture(records, placed, background, memory)`.
fn picture_in(
    records: &[Vec<u8>],
    placed: bool,
    background: i64,
    memory: Rc<FakeMemory>,
) -> puzzle::TilePuzzle {
    let classes = FakeClasses::new(memory.clone());
    memory.u32(PICTURE, &[classes.add(puzzle::PICTURE)]);
    let vtable = classes.add(puzzle::BACKGROUND);
    memory.u32(BACKGROUND, &[vtable]);
    memory.u32(BACKGROUND + puzzle::DRAWN, &[background]);
    memory.write(BACKGROUND + puzzle::COPY, &[background as u8]);
    memory.u32(BACKGROUND + puzzle::IMAGE, &[IMAGE]);
    memory.u32(CODE, &[vtable]);
    memory.u32(CODE + puzzle::DRAWN, &[0xE59400C8]);
    memory.write(CODE + puzzle::COPY, &[0x21]);
    for (i, (raw, (x, z))) in records.iter().zip(STANDING).enumerate() {
        let param = PARAMS + i as i64 * 0x200;
        memory.u32(PICTURE + puzzle::MODELS[i], &[param]);
        memory.u32(
            param + pokemon::PARAM_RECORD,
            &[param + pokemon::RECORD_IN_PARAM],
        );
        memory.write(param + pokemon::RECORD_IN_PARAM, raw);
        if placed {
            let mut point = Vec::new();
            for value in [x, 0.0, z] {
                point.extend(value.to_le_bytes());
            }
            memory.write(
                PICTURE + puzzle::POINTS + i as i64 * 0x30 + puzzle::STANDS,
                &point,
            );
        }
    }
    memory.u32(BOARD, &[classes.add(puzzle::CLASS)]);
    for i in 0..20 {
        memory.u32(
            BOARD + puzzle::RECORDS + i * puzzle::SIZE + puzzle::NUMBER,
            &[20 - i],
        );
        memory.u32(
            BOARD + puzzle::RECORDS + i * puzzle::SIZE + puzzle::SLOT,
            &[i],
        );
    }
    let mut reader = puzzle::TilePuzzle::new(
        memory.clone(),
        FakeLayouts::new(&memory, &classes).layouts.clone(),
    );
    reader
        .track(
            &IndexMap::from([
                (PICTURE, puzzle::PICTURE.to_string()),
                (BOARD, puzzle::CLASS.to_string()),
                (CODE, puzzle::BACKGROUND.to_string()),
                (BACKGROUND, puzzle::BACKGROUND.to_string()),
            ]),
            false,
        )
        .unwrap();
    reader
}

fn named(found: &pokemon::Pokemon) -> Option<String> {
    Some(if found.nicknamed {
        found.nickname.clone()
    } else if found.species == 25 {
        "Pikachu Rock Star".to_string()
    } else {
        "?".to_string()
    })
}

#[test]
fn test_the_picture_is_three_of_the_party_said_left_to_right() {
    // ADR-0038: the picture is said left to right. or_amie_tile_puzzle_easy: Blaze behind on the
    // right, Pikachu in its first form, the one Amie played with, in front, and Metagross behind
    // on the left (spike 256).
    let reader = picture(
        &[
            record(0x1, 257, "Blaze", true),
            record_form(0x2, 25, "Pikachu", false, 1),
            record(0x3, 376, "Metagross", true),
        ],
        true,
    );
    let shown = reader.picture().unwrap().expect("a picture");
    assert_eq!(
        shown
            .iter()
            .map(|(found, _, _)| (found.species, found.form))
            .collect::<Vec<_>>(),
        [(257, 0), (25, 1), (376, 0)]
    );
    assert_eq!(
        puzzle::described(&shown, &named, None).as_deref(),
        Some("Metagross on the left, Pikachu Rock Star in front, Blaze on the right")
    );
}

#[test]
fn test_the_background_is_said_after_the_pokemon_by_its_number() {
    // ADR-0038: or_amie_tile_puzzle_easy drew background 12, the shallow water (spike 256); a
    // number with no words in the data is left out, its Pokémon said all the same.
    let party = [
        record(0x1, 257, "Blaze", true),
        record_form(0x2, 25, "Pikachu", false, 1),
    ];
    let reader = picture(&party, true);
    let shown = reader.picture().unwrap().expect("a picture");
    assert_eq!(reader.background().unwrap(), Some(12));
    assert_eq!(
        puzzle::described(&shown, &named, reader.background().unwrap()).as_deref(),
        Some(
            "Pikachu Rock Star in front, Blaze on the right, in shallow water before a wooded bank"
        )
    );
    let unknown = picture_before(&party, true, 99);
    assert_eq!(
        puzzle::described(&shown, &named, unknown.background().unwrap()).as_deref(),
        Some("Pikachu Rock Star in front, Blaze on the right")
    );
}

#[test]
fn test_the_picture_waits_for_its_background_to_be_drawn() {
    // Spike D: in two of six replays shifted inside Amie, the puzzle's Bg held 0 as its Pokémon
    // were placed, and the picture was said without its background.
    let party = [
        record(0x1, 257, "Blaze", true),
        record(0x2, 25, "Pikachu", false),
    ];
    assert!(
        picture_before(&party, true, 0)
            .new_picture()
            .unwrap()
            .is_none()
    );
}

#[test]
fn test_the_picture_is_given_once_until_another_app_starts() {
    // ADR-0038: said once an app.
    let reader = picture(
        &[
            record(0x1, 257, "Blaze", true),
            record(0x2, 25, "Pikachu", false),
        ],
        true,
    );
    assert!(reader.new_picture().unwrap().is_some());
    assert!(reader.new_picture().unwrap().is_none());
    reader.started();
    assert!(reader.new_picture().unwrap().is_some());
}

#[test]
fn test_play_again_draws_a_picture_afresh_said_once_its_image_is_loaded() {
    // or_amie_tile_puzzle_play_again: each Play Again drew two Pokémon and a background of its
    // own, 1, 4 and 9, its image 0 as it was drawn and at another address after; the picture was
    // said only as the puzzle opened.
    let memory = FakeMemory::new();
    let reader = picture_in(
        &[
            record(0x1, 257, "Blaze", true),
            record(0x2, 25, "Pikachu", false),
        ],
        true,
        12,
        memory.clone(),
    );
    assert!(reader.new_picture().unwrap().is_some());
    memory.u32(BACKGROUND + puzzle::IMAGE, &[0]);
    memory.u32(BACKGROUND + puzzle::DRAWN, &[4]);
    memory.write(BACKGROUND + puzzle::COPY, &[4]);
    assert!(reader.new_picture().unwrap().is_none());
    memory.u32(BACKGROUND + puzzle::IMAGE, &[0x14E7ED80]);
    assert!(reader.new_picture().unwrap().is_some() && reader.background().unwrap() == Some(4));
    assert!(reader.new_picture().unwrap().is_none());
}

#[test]
fn test_a_record_that_does_not_check_is_no_picture() {
    // The module's own code holds words like a PokeModel's vtable, its pointers instructions.
    let mut broken = record(0x1, 257, "Blaze", true);
    broken[pokemon::CHECKSUM as usize] ^= 1;
    let reader = picture(&[broken], true);
    assert!(reader.picture().unwrap().is_none());
    assert!(reader.new_picture().unwrap().is_none());
}

#[test]
fn test_the_picture_waits_for_its_pokemon_to_be_placed() {
    // or_amie_tile_puzzle_easy_described: found as the puzzle opened, all three stood at 0, and
    // were said all "in front".
    let reader = picture(
        &[
            record(0x1, 257, "Blaze", true),
            record(0x2, 25, "Pikachu", false),
        ],
        false,
    );
    assert!(reader.new_picture().unwrap().is_none());
}
