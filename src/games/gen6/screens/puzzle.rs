//! Pokémon-Amie's Tile Puzzle: which piece of the picture each tile holds (spike A).
//!
//! The puzzle shows its picture whole, the player's party on the sand under the sky, then cuts it
//! into twenty tiles, five across and four down, and shuffles them; two taps swap two tiles, and
//! the clock runs until every piece is home (as_amie_tile_puzzle). Each tile is a touch area under
//! a picture of `frame.bclim` alone, and touch mode said every one "frame": the picture itself is
//! drawn by the puzzle's 3D scene. Its `app::puzzle::Piece` holds the board, a record a piece:
//!
//! ```text
//! Piece   +0x78  twenty records, 0x74 bytes each, the pieces numbered 20 down to 1
//! record  +0x0C  2 while the piece is the tile picked, to swap with the next; else 1
//!         +0x10  the piece's number; it belongs in slot 20 less it
//!         +0x70  the slot it is in, 0 to 19 a row at a time from the top left
//! ```
//!
//! So each tile is said by its row and column and where its piece belongs: "row 1, column 2, in
//! place", "row 1, column 2, goes to row 3, column 4". Saying where a piece belongs is more than a
//! sighted player is told, who sees the picture whole and matches each piece by what is on it; it
//! is on for now (Orion, 2026-10-26), until movies of more puzzles show how the game chooses its
//! pictures and cuts them, and the tiles can be said as they look. Then where a piece belongs
//! becomes an Extra, an easy mode a player turns on.
//!
//! Normal, Hard and Unlimited are not measured: a board of other than twenty pieces is not read.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::memory::Memory;
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::touch::{HEIGHT, WIDTH};
use crate::py::{self, Result};

pub const CLASS: &str = "N3app6puzzle5PieceE";
pub const APP: &str = "N3app6puzzle7ProcessE";
/// The picture over each tile, the one thing its touch area's group draws.
pub const FRAME: &str = "frame.bclim";
pub const RECORDS: i64 = 0x78;
pub const SIZE: i64 = 0x74;
pub const PIECES: i64 = 20;
pub const ACROSS: i64 = 5;
pub const STATE: i64 = 0x0C;
pub const NUMBER: i64 = 0x10;
pub const SLOT: i64 = 0x70;
pub const PICKED: i64 = 2;
/// The first tile's middle in pixels, and from one tile's to the next.
pub const LEFT: i64 = 40;
pub const TOP: i64 = 30;
pub const STEP: i64 = 60;
/// Pixels a picture's middle may lie from a tile's.
pub const SLACK: i64 = 6;

pub const GRID: &str = "slot-grid";
pub const HOME: &str = "puzzle-home";
pub const GOES: &str = "puzzle-goes";
pub const PICKED_WORD: &str = "puzzle-picked";

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

    /// Each slot -> the slot its piece belongs in, and whether it is picked; None with no
    /// board. A record out of its order, or two pieces in one slot, is no board: the module's
    /// own data holds words like a Piece's vtable.
    pub fn board(&self) -> Result<Option<IndexMap<i64, (i64, bool)>>> {
        'pieces: for (&piece, &vtable) in &self.pieces {
            let raw = self.memory.read(piece, RECORDS + PIECES * SIZE)?;
            if u32(&raw, 0)? != vtable {
                continue;
            }
            let mut board: IndexMap<i64, (i64, bool)> = IndexMap::new();
            for i in 0..PIECES {
                let at = RECORDS + i * SIZE;
                let (number, slot) = (u32(&raw, at + NUMBER)?, u32(&raw, at + SLOT)?);
                if number != PIECES - i || slot >= PIECES || board.contains_key(&slot) {
                    continue 'pieces;
                }
                board.insert(slot, (PIECES - number, u32(&raw, at + STATE)? == PICKED));
            }
            return Ok(Some(board));
        }
        Ok(None)
    }

    /// A tile's frame picture, said by where its piece belongs; None for any other.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        let slot = self.slot(picture)?;
        let board = match slot {
            Some(_) => self.board()?,
            None => None,
        };
        let (Some(slot), Some(board)) = (slot, board) else {
            return Ok(None);
        };
        let (home, picked) = board[&slot];
        let tile = place(slot);
        let said = if home == slot {
            own(HOME, &[("tile", &tile)])
        } else {
            own(GOES, &[("tile", &tile), ("home", &place(home))])
        };
        Ok(Some(if picked {
            own(PICKED_WORD, &[("tile", &said)])
        } else {
            said
        }))
    }

    /// The slot a picture lies over, by its middle: None off the grid.
    fn slot(&self, picture: i64) -> Result<Option<i64>> {
        let layouts = &self.layouts;
        let placement = layouts.placement(picture)?;
        let Some(placement) = placement.filter(|placement| !placement.top) else {
            return Ok(None);
        };
        let shape = layouts.shape;
        let (head, root) = (layouts.pane(picture)?, layouts.pane(placement.root)?);
        let x = WIDTH as f64 / 2.0 + f32(&head, shape.x)? - f32(&root, shape.x)?;
        let y = HEIGHT as f64 / 2.0 - (f32(&head, shape.y)? - f32(&root, shape.y)?);
        let (column, row) = (
            py::round((x - LEFT as f64) / STEP as f64)?,
            py::round((y - TOP as f64) / STEP as f64)?,
        );
        if !((0..ACROSS).contains(&column)
            && (0..PIECES / ACROSS).contains(&row)
            && (x - LEFT as f64 - (column * STEP) as f64).abs() <= SLACK as f64
            && (y - TOP as f64 - (row * STEP) as f64).abs() <= SLACK as f64)
        {
            return Ok(None);
        }
        Ok(Some(row * ACROSS + column))
    }
}

/// A slot by its row and column, counted from 1.
fn place(slot: i64) -> String {
    own(
        GRID,
        &[
            ("row", &(slot / ACROSS + 1)),
            ("column", &(slot % ACROSS + 1)),
        ],
    )
}
