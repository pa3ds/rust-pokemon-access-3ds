//! Tile Puzzle's tiles are said by their place and where the piece each shows belongs.

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{FakeClasses, FakeMemory};
use crate::games::common::lyt;
use crate::games::gen6::screens::puzzle;

const SCREEN: i64 = 0x08300000;
const BOARD: i64 = 0x08400000;
const FRAMES: i64 = 0x08310000;

/// A board whose piece belonging in slot i, numbered 20 less i, lies in `slots[i]`, and a
/// frame picture over each slot where the game draws its tiles, five across and four down.
fn board(slots: &[i64], picked: Option<i64>) -> (puzzle::TilePuzzle, Vec<i64>) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let screens = FakeLayouts::new(&memory, &classes);
    let (_, group) = screens.screen(SCREEN);
    let mut frames: Vec<i64> = Vec::new();
    for slot in 0..puzzle::PIECES {
        let at = FRAMES + slot * 0x100;
        screens.pane(at, group);
        let (row, column) = (slot / puzzle::ACROSS, slot % puzzle::ACROSS);
        memory.write(
            at + lyt::ORAS.x,
            &((column * 60 + 40 - 160) as f32).to_le_bytes(),
        );
        memory.write(
            at + lyt::ORAS.y,
            &((120 - (row * 60 + 30)) as f32).to_le_bytes(),
        );
        frames.push(at);
    }
    screens.track().unwrap();
    memory.u32(BOARD, &[classes.add(puzzle::CLASS)]);
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
        memory.u32(at + puzzle::NUMBER, &[puzzle::PIECES - i as i64]);
        memory.u32(at + puzzle::SLOT, &[slot]);
    }
    let mut reader = puzzle::TilePuzzle::new(memory.clone(), screens.layouts.clone());
    reader
        .track(&IndexMap::from([(BOARD, puzzle::CLASS.to_string())]), false)
        .unwrap();
    (reader, frames)
}

#[test]
fn test_a_tile_says_its_place_and_where_its_piece_belongs() {
    // as_amie_tile_puzzle: after the shuffle the top-left tile held the piece of row 3, column 5,
    // and the picked tile said so (spike A).
    let mut slots: Vec<i64> = (0..puzzle::PIECES).collect();
    (slots[14], slots[0]) = (0, 14);
    let (reader, frames) = board(&slots, Some(0));
    assert_eq!(
        reader.name(frames[0]).unwrap().as_deref(),
        Some("row 1, column 1, goes to row 3, column 5, picked")
    );
    assert_eq!(
        reader.name(frames[14]).unwrap().as_deref(),
        Some("row 3, column 5, goes to row 1, column 1")
    );
    assert_eq!(
        reader.name(frames[7]).unwrap().as_deref(),
        Some("row 2, column 3, in place")
    );
}

#[test]
fn test_records_out_of_order_or_two_in_one_slot_are_no_board() {
    // The module's own data held words like a Piece's vtable (spike A).
    let mut slots: Vec<i64> = (0..puzzle::PIECES).collect();
    slots[3] = slots[4];
    let (reader, frames) = board(&slots, None);
    assert!(reader.board().unwrap().is_none());
    assert_eq!(reader.name(frames[0]).unwrap(), None);
}
