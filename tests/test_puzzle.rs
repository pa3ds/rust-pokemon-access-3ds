//! Tile Puzzle's tiles are said by their place and where the piece each shows belongs.

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{FakeClasses, FakeMemory};
use crate::games::common::lyt;
use crate::games::common::touch::Target;
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

fn tile(area: i64) -> Target {
    Target::new(area, "", 0, 0, 0.0, 40.0)
}

fn name(reader: &puzzle::TilePuzzle, picture: i64) -> Option<String> {
    reader.name(picture).unwrap()
}

fn moved(reader: &puzzle::TilePuzzle, before: i64, now: i64) -> Option<String> {
    reader.moved(&tile(before), &tile(now)).unwrap()
}

#[test]
fn test_a_tile_says_its_place_and_where_its_piece_belongs() {
    // as_amie_tile_puzzle: after the shuffle the top-left tile held the piece of row 3, column 5,
    // and the picked tile said so (spike A).
    let mut slots: Vec<i64> = (0..20).collect();
    (slots[14], slots[0]) = (0, 14);
    let (reader, frames, _) = board(&slots, Some(0), 0);
    assert_eq!(
        name(&reader, frames[0]).as_deref(),
        Some("row 1, column 1, goes to row 3, column 5, picked")
    );
    assert_eq!(
        name(&reader, frames[14]).as_deref(),
        Some("row 3, column 5, goes to row 1, column 1")
    );
    assert_eq!(
        name(&reader, frames[7]).as_deref(),
        Some("row 2, column 3, in place")
    );
}

#[test]
fn test_normals_thirty_tiles_lie_under_touch_areas_numbered_from_955() {
    // as_amie_tile_puzzle_normal: six across and five down, under touch areas `qj` to `rc`; the
    // first tap, at the bottom right, picked the piece numbered 26, and the second put it in row
    // 2, column 6 (spike A).
    let mut slots: Vec<i64> = (0..30).collect();
    (slots[4], slots[29]) = (29, 4);
    let (reader, frames, areas) = board(&slots, Some(29), 955);
    assert_eq!(
        name(&reader, frames[29]).as_deref(),
        Some("row 5, column 6, goes to row 1, column 5, picked")
    );
    assert_eq!(
        name(&reader, frames[4]).as_deref(),
        Some("row 1, column 5, goes to row 5, column 6")
    );
    assert_eq!(
        name(&reader, frames[11]).as_deref(),
        Some("row 2, column 6, in place")
    );
    assert_eq!(
        moved(&reader, areas[23], areas[29]).as_deref(),
        Some("row 5, goes to row 1, column 5, picked")
    );
}

#[test]
fn test_hards_forty_eight_tiles_are_named_at_either_end_of_a_row() {
    // as_amie_tile_puzzle_hard: eight across and six down, under `ii` to `jt`; placed by their
    // middles, the first and last columns were "frame" (Orion, 2026-10-27).
    let mut slots: Vec<i64> = (0..48).collect();
    (slots[0], slots[47]) = (47, 0);
    let (reader, frames, _) = board(&slots, Some(47), 666);
    assert_eq!(
        name(&reader, frames[0]).as_deref(),
        Some("row 1, column 1, goes to row 6, column 8")
    );
    assert_eq!(
        name(&reader, frames[47]).as_deref(),
        Some("row 6, column 8, goes to row 1, column 1, picked")
    );
    assert_eq!(
        name(&reader, frames[15]).as_deref(),
        Some("row 2, column 8, in place")
    );
}

#[test]
fn test_a_move_says_only_the_row_or_column_that_changed() {
    // Orion, 2026-10-26: O, O, O and L from row 1, column 2 say "row 2", "row 3", "row 4",
    // "column 3", each with where the tile's piece belongs.
    let mut slots: Vec<i64> = (0..20).collect();
    (slots[14], slots[0]) = (0, 14);
    let (reader, _, areas) = board(&slots, Some(0), 0);
    assert_eq!(
        moved(&reader, areas[1], areas[6]).as_deref(),
        Some("row 2, in place")
    );
    assert_eq!(
        moved(&reader, areas[13], areas[14]).as_deref(),
        Some("column 5, goes to row 1, column 1")
    );
    assert_eq!(
        moved(&reader, areas[19], areas[0]).as_deref(),
        Some("row 1, column 1, goes to row 3, column 5, picked")
    );
    assert_eq!(moved(&reader, areas[7], areas[7]), None);
    assert_eq!(moved(&reader, areas[7], SCREEN + 0x600), None);
}

#[test]
fn test_records_out_of_order_or_two_in_one_slot_are_no_board() {
    // The module's own data held words like a Piece's vtable (spike A).
    let mut slots: Vec<i64> = (0..20).collect();
    slots[3] = slots[4];
    let (reader, frames, _) = board(&slots, None, 0);
    assert!(reader.board().unwrap().is_none());
    assert_eq!(name(&reader, frames[0]), None);
}
