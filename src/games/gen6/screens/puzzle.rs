//! Pokémon-Amie's Tile Puzzle: which piece of the picture each tile holds (spike A).
//!
//! The puzzle shows its picture whole, three of the player's Pokémon on the sand under the sky,
//! then cuts it into tiles and shuffles them at Begin; two taps swap two tiles, and the clock runs
//! until every piece is home (as_amie_tile_puzzle). Easy cuts it into 20, five across and four
//! down; Normal into 30, six across and five down (as_amie_tile_puzzle_normal); Hard into 48, eight
//! across and six down (as_amie_tile_puzzle_hard). Each tile is a group of a touch area, a picture
//! of `frame.bclim` and `cursol.bclim`'s, drawn over the tile picked, and touch mode said every one
//! "frame": the picture itself is drawn by the puzzle's 3D scene. Its `app::puzzle::Piece` holds
//! the board, a record a piece:
//!
//! ```text
//! Piece   +0x78  a record a piece, 0x74 bytes each, numbered from how many there are down to 1
//! record  +0x0C  2 while the piece is the tile picked, to swap with the next; else 1
//!         +0x10  the piece's number; it belongs in the slot that many less than the count
//!         +0x70  the touch area it lies under, by its name as a number in base 36: Easy's
//!                `0` to `j`, 0 to 19, Normal's `qj` to `rc`, 955 to 984, Hard's `ii` to `jt`,
//!                666 to 713, a row at a time from the top left; less the first, its slot
//! ```
//!
//! So a tile's own touch area names its slot, wherever it is drawn: placed by their middles,
//! Hard's tiles, 38 pixels apart and not the 40 its rows would have sharing the screen as Easy's
//! and Normal's do, were "frame" at either end of a row, and a picked one fell out of touch mode,
//! taking its cursor back to the first tile (Orion, 2026-10-27).
//!
//! Each tile is said by its row and column and where its piece belongs: "row 1, column 2, in
//! place", "row 1, column 2, goes to row 3, column 4"; touch mode's keys moving from tile to tile
//! say only the row or the column that changed, "column 3, in place". Saying where a piece belongs
//! is more than a sighted player is told, who sees the picture whole and matches each piece by
//! what is on it; it is on for now (Orion, 2026-10-26), until movies of more puzzles show how the
//! game chooses its pictures and cuts them, and the tiles can be said as they look. Then where a
//! piece belongs becomes an Extra, an easy mode a player turns on.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::games::common::touch::Target;
use crate::py::Result;

pub const CLASS: &str = "N3app6puzzle5PieceE";
pub const APP: &str = "N3app6puzzle7ProcessE";
/// The picture over each tile, in a group with its touch area.
pub const FRAME: &str = "frame.bclim";
pub const RECORDS: i64 = 0x78;
pub const SIZE: i64 = 0x74;
/// Tiles across, by how many pieces: Easy, Normal and Hard.
pub const ACROSS: [(i64, i64); 3] = [(20, 5), (30, 6), (48, 8)];
/// `min(ACROSS)`.
pub const FEWEST: i64 = 20;
pub const STATE: i64 = 0x0C;
pub const NUMBER: i64 = 0x10;
pub const SLOT: i64 = 0x70;
pub const PICKED: i64 = 2;

pub const GRID: &str = "slot-grid";
pub const ROW: &str = "puzzle-row";
pub const COLUMN: &str = "puzzle-column";
pub const HOME: &str = "puzzle-home";
pub const GOES: &str = "puzzle-goes";
pub const PICKED_WORD: &str = "puzzle-picked";

/// `ACROSS[pieces]`, None where `pieces not in ACROSS`.
fn across(pieces: i64) -> Option<i64> {
    ACROSS
        .iter()
        .find(|&&(count, _)| count == pieces)
        .map(|&(_, across)| across)
}

/// The puzzle as it lies: how many tiles across, the first slot's touch area, and each slot's
/// piece.
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    pub across: i64,
    /// The number the first slot's touch area is named by, in base 36.
    pub first: i64,
    /// Each slot -> the slot its piece belongs in, and whether it is the tile picked.
    pub tiles: IndexMap<i64, (i64, bool)>,
}

impl Board {
    /// The slot under the touch area numbered `area`, or None for one of no slot.
    pub fn slot(&self, area: Option<i64>) -> Option<i64> {
        let slot = area?.checked_sub(self.first)?;
        self.tiles.contains_key(&slot).then_some(slot)
    }

    /// A slot by its row and column, counted from 1.
    pub fn place(&self, slot: i64) -> String {
        own(
            GRID,
            &[
                ("row", &(slot / self.across + 1)),
                ("column", &(slot % self.across + 1)),
            ],
        )
    }

    /// A tile, said as `tile`, then where its piece belongs and whether it is picked.
    pub fn said(&self, slot: i64, tile: &str) -> String {
        let (home, picked) = self.tiles[&slot];
        let said = if home == slot {
            own(HOME, &[("tile", &tile)])
        } else {
            own(GOES, &[("tile", &tile), ("home", &self.place(home))])
        };
        if picked {
            own(PICKED_WORD, &[("tile", &said)])
        } else {
            said
        }
    }
}

/// Names Tile Puzzle's tiles by where their pieces belong.
pub struct TilePuzzle {
    memory: Rc<dyn Memory>,
    layouts: Rc<Layouts>,
    /// Piece object -> the vtable it was found with.
    pieces: IndexMap<i64, i64>,
}

impl TilePuzzle {
    pub fn new(memory: Rc<dyn Memory>, layouts: Rc<Layouts>) -> Self {
        TilePuzzle {
            memory,
            layouts,
            pieces: IndexMap::new(),
        }
    }

    /// The boards a sweep found, by class: all of them, or with `adding` the new ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut pieces = if adding {
            self.pieces.clone()
        } else {
            IndexMap::new()
        };
        for (&piece, name) in found {
            if name == CLASS {
                pieces.insert(piece, u32(&self.memory.read(piece, 4)?, 0)?);
            }
        }
        self.pieces = pieces;
        Ok(())
    }

    /// The board, or None. A count of pieces of no difficulty's, a record out of its order or
    /// two pieces in one slot is no board: the module's own data holds words like a Piece's
    /// vtable. Read as Easy's first, and again whole for more pieces.
    pub fn board(&self) -> Result<Option<Board>> {
        for (&piece, &vtable) in &self.pieces {
            let mut raw = self.memory.read(piece, RECORDS + FEWEST * SIZE)?;
            let pieces = u32(&raw, RECORDS + NUMBER)?;
            let Some(wide) = across(pieces) else {
                continue;
            };
            if u32(&raw, 0)? != vtable {
                continue;
            }
            if pieces > FEWEST {
                raw = self.memory.read(piece, RECORDS + pieces * SIZE)?;
            }
            let mut records: Vec<(i64, i64, i64)> = Vec::new();
            for at in (RECORDS..RECORDS + pieces * SIZE).step_by(SIZE as usize) {
                records.push((
                    u32(&raw, at + NUMBER)?,
                    u32(&raw, at + SLOT)?,
                    u32(&raw, at + STATE)?,
                ));
            }
            let first = records
                .iter()
                .map(|&(_, slot, _)| slot)
                .min()
                .expect("a board has pieces");
            let numbers: Vec<i64> = records.iter().map(|&(number, _, _)| number).collect();
            let mut slots: Vec<i64> = records.iter().map(|&(_, slot, _)| slot).collect();
            slots.sort();
            if numbers != (1..=pieces).rev().collect::<Vec<i64>>()
                || slots != (first..first + pieces).collect::<Vec<i64>>()
            {
                continue;
            }
            return Ok(Some(Board {
                across: wide,
                first,
                tiles: records
                    .iter()
                    .map(|&(number, slot, state)| {
                        (slot - first, (pieces - number, state == PICKED))
                    })
                    .collect(),
            }));
        }
        Ok(None)
    }

    /// A tile's frame picture, said by where its piece belongs; None for any other.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        let area = self.area(picture)?;
        let board = match area {
            Some(_) => self.board()?,
            None => None,
        };
        let slot = board.as_ref().and_then(|board| board.slot(area));
        let (Some(board), Some(slot)) = (board, slot) else {
            return Ok(None);
        };
        Ok(Some(board.said(slot, &board.place(slot))))
    }

    /// Touch mode's key moving from one tile to another: only the row or the column that
    /// changed, then where its piece belongs, "column 3, in place" (Orion, 2026-10-26); None
    /// for its whole name, staying on one or moving off the board.
    pub fn moved(&self, before: &Target, now: &Target) -> Result<Option<String>> {
        let Some(board) = self.board()? else {
            return Ok(None);
        };
        let (old, new) = (
            board.slot(self.number(before.pane)?),
            board.slot(self.number(now.pane)?),
        );
        let (Some(old), Some(new)) = (old, new) else {
            return Ok(None);
        };
        if old == new {
            return Ok(None);
        }
        let ((row, column), (was_row, was_column)) = (
            (new / board.across, new % board.across),
            (old / board.across, old % board.across),
        );
        let tile = if row != was_row && column != was_column {
            board.place(new)
        } else if row != was_row {
            own(ROW, &[("row", &(row + 1))])
        } else {
            own(COLUMN, &[("column", &(column + 1))])
        };
        Ok(Some(board.said(new, &tile)))
    }

    /// The number of the touch area in a picture's group; None on the top screen or with
    /// none beside it.
    fn area(&self, picture: i64) -> Result<Option<i64>> {
        let layouts = &self.layouts;
        let placement = layouts.placement(picture)?;
        let Some(placement) = placement.filter(|placement| !placement.top) else {
            return Ok(None);
        };
        let tree = layouts.tree(placement.root);
        let group = u32(&layouts.pane(picture)?, lyt::PARENT)?;
        for p in layouts.children(placement.root, group) {
            if tree.get(&p).map(String::as_str) == Some(lyt::BOUNDING_CLASS) {
                return self.number(p);
            }
        }
        Ok(None)
    }

    /// A touch area's name as a number in base 36, or None for a name that is not one.
    fn number(&self, area: i64) -> Result<Option<i64>> {
        let raw = self.layouts.name(area)?;
        let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
        Ok(int_base36(&raw[..end]))
    }
}

/// `int(name.decode("ascii"), 36)`, None where either raises: ASCII only, outer whitespace, a
/// sign, and single underscores between digits. `py` has no base 36. A number past an i64 is
/// None too, where Python's is a number no touch area's slot is: either way the area has no slot.
fn int_base36(name: &[u8]) -> Option<i64> {
    if !name.is_ascii() {
        return None;
    }
    let text = std::str::from_utf8(name).ok()?;
    let text = text.trim_matches(|c| matches!(c, ' ' | '\t' | '\n' | '\x0b' | '\x0c' | '\r'));
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    if digits.is_empty() || digits.starts_with('_') || digits.ends_with('_') {
        return None;
    }
    let mut value: i64 = 0;
    let mut previous = None;
    for c in digits.chars() {
        if c == '_' {
            if previous == Some('_') {
                return None;
            }
        } else {
            value = value
                .checked_mul(36)?
                .checked_add(i64::from(c.to_digit(36)?))?;
        }
        previous = Some(c);
    }
    Some(if negative { -value } else { value })
}
