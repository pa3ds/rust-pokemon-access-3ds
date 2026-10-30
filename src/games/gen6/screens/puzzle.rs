//! Pokémon-Amie's Tile Puzzle: which piece of the picture each tile holds (spike 254), and who is
//! in the picture (spike 256).
//!
//! The puzzle shows its picture whole, three of the player's Pokémon before a background, then
//! cuts it into tiles and shuffles them at Begin; two taps swap two tiles, and the clock runs
//! until every piece is home (or_amie_tile_puzzle_easy). Easy cuts it into 20, five across and four
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
//! Each tile is said as a spreadsheet names a cell, its column a letter and its row a number, and
//! where its piece belongs: "B1, in place", "B1, goes to D3" (Orion, 2026-10-28, shorter than "row
//! 1, column 2, goes to row 3, column 4", and than the row or column alone that a move said
//! before). Saying where a piece belongs is more than a sighted player is told, who sees the picture whole and matches each piece by
//! what is on it; it is on for now (Orion, 2026-10-26), until movies of more puzzles show how the
//! game chooses its pictures and cuts them, and the tiles can be said as they look. Then where a
//! piece belongs becomes an Extra, an easy mode a player turns on.
//!
//! The picture is a 3D scene, and its `app::puzzle::PokeModel` holds who is in it: three pointers
//! to a `PokemonParam` each, a copy of the party's whose record checks, and four points a Pokémon,
//! where it stands among them. The one Amie plays with stands in front in the middle, x 0, and two
//! more behind it, x below 0 on the left and above on the right (or_amie_tile_puzzle_easy's
//! Metagross, Pikachu and Blaze; as_amie_tile_puzzle_normal's Poochyena, Ralts and rowan). The two
//! behind are drawn afresh each time it opens, and the background too, whatever the place:
//! `app::puzzle::Bg` holds its number, which pa3ds's own words describe, as the game never names
//! it. With descriptions on, the picture is said once the puzzle holds them, left to right, then
//! its background, and again for each picture Play Again draws afresh.
//!
//! Bound by:
//! - ADR-0038: the picture is Narration's, said each time one is drawn, off unless asked for.

use std::cell::Cell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::memory::Memory;
use crate::core::speech::Category;
use crate::core::words::{current, own};
use crate::games::common::layout::Layouts;
use crate::games::common::touch::ImageOf;
use crate::games::common::{lyt, pokemon};
use crate::py::{self, Result};

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

/// The picture's Pokémon (spike 256).
pub const PICTURE: &str = "N3app6puzzle9PokeModelE";
/// Where `PICTURE` points to each Pokémon's `PokemonParam`, a copy of the party's, in the order of
/// `POINTS`; each followed by a counter its motion runs.
pub const MODELS: [i64; 3] = [0x0C, 0x1C, 0x2C];
/// Four points a Pokémon, x, y and z, 0x30 bytes apart: its box's least and greatest corners, the
/// spot it stands on (`STANDS`), and a point within the box, at Blaziken's head.
pub const POINTS: i64 = 0x38;
pub const STANDS: i64 = 0x18;
pub const PICTURE_SIZE: i64 = POINTS + MODELS.len() as i64 * 0x30;
/// The picture's background (spike 256).
pub const BACKGROUND: &str = "N3app6puzzle2BgE";
/// Where `BACKGROUND` holds the number of the background drawn, at random whatever the place, from
/// the eleven at `DllPuzzle.cro` +0xF21C: 13 the open sea, 44 rocky ground and so on.
pub const DRAWN: i64 = 0x1DC;
/// Where `BACKGROUND` holds the number again, a byte, and again 2 and 3 bytes on.
pub const COPY: i64 = 0x20;
/// Where `BACKGROUND` holds the physical address of the background's image: 0 while it is drawn,
/// as the puzzle opens and as Play Again draws a picture afresh, and another address after
/// (or_amie_tile_puzzle_play_again's backgrounds 1, 4 and 9, each with two Pokémon of its own).
pub const IMAGE: i64 = 0x88;

pub const CELL: &str = "puzzle-cell";
pub const HOME: &str = "puzzle-home";
pub const GOES: &str = "puzzle-goes";
pub const PICKED_WORD: &str = "puzzle-picked";
pub const LEFT: &str = "puzzle-picture-left";
pub const FRONT: &str = "puzzle-picture-front";
pub const RIGHT: &str = "puzzle-picture-right";
/// A background's words, by its number: `puzzle-background-13`, a game's own.
pub const SETTING: &str = "puzzle-background-";
pub const BACKGROUND_LOG: &str = "puzzle background";
/// What the log calls a picture's background's number, and the picture where it is not said.
pub const PICTURE_LOG: &str = "puzzle picture";

/// The picture's Pokémon, each with where it stands: x, right the greater, and z, nearer the
/// greater.
pub type Picture = Vec<(pokemon::Pokemon, f64, f64)>;

/// What a Pokémon of the player's is called.
pub type Name = Rc<dyn Fn(&pokemon::Pokemon) -> Option<String>>;

/// The picture in words, left to right, each Pokémon called by `name`, then the background
/// where the data has words for its number: "Metagross on the left, Pikachu Rock Star in front,
/// Blaze on the right, in shallow water before a wooded bank"; None where a Pokémon has no name.
/// The one the player plays with stands in front in the middle, x 0, and two more behind it to
/// either side (or_amie_tile_puzzle_easy, as_amie_tile_puzzle_normal).
///
/// Python's `background` defaults to None.
pub fn described(
    picture: &Picture,
    name: &dyn Fn(&pokemon::Pokemon) -> Option<String>,
    background: Option<i64>,
) -> Option<String> {
    let mut said: Vec<String> = Vec::new();
    for (found, x, _) in py::sorted_by_lt(picture.clone(), |a, b| a.1 < b.1) {
        let called = name(&found)?;
        let word = if x < 0.0 {
            LEFT
        } else if x > 0.0 {
            RIGHT
        } else {
            FRONT
        };
        said.push(own(word, &[("pokemon", &called)]));
    }
    if let Some(background) = background
        && let Some(r#where) = current().get(&format!("{SETTING}{background}"), &[])
        && !r#where.is_empty()
    {
        said.push(r#where);
    }
    Some(said.join(", "))
}

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

    /// A slot as a spreadsheet names a cell, its column a letter from A and its row a number
    /// from 1: "B3" the second column of the third row (Orion, 2026-10-28).
    pub fn place(&self, slot: i64) -> String {
        let column = char::from_u32('A' as u32 + (slot % self.across) as u32)
            .expect("a column is a letter")
            .to_string();
        own(
            CELL,
            &[("column", &column), ("row", &(slot / self.across + 1))],
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
    /// The image a picture draws, by its file's name: a tile's is `FRAME`.
    image_of: ImageOf,
    /// What a Pokémon of the player's is called, or None where the game's text is not read,
    /// and the picture is not said (`poll`).
    name: Option<Name>,
    /// Whether the picture is said, by the narrator, or only logged (`[pa3ds.ui]
    /// descriptions`).
    describing: bool,
    /// Piece object -> the vtable it was found with.
    pieces: IndexMap<i64, i64>,
    /// PokeModel object -> the vtable it was found with.
    pictures: IndexMap<i64, i64>,
    /// Bg object -> the vtable it was found with.
    backgrounds: IndexMap<i64, i64>,
    /// The image of the background last said with its picture, or 0.
    shown: Cell<i64>,
}

impl TilePuzzle {
    /// What the heap is swept for: the board, which says where each tile's piece belongs,
    /// and the picture and its background, which say who and what the picture shows.
    pub const CLASSES: [&'static str; 3] = [CLASS, PICTURE, BACKGROUND];

    pub fn new(memory: Rc<dyn Memory>, layouts: Rc<Layouts>, image_of: ImageOf) -> Self {
        TilePuzzle::with(memory, layouts, image_of, None, false)
    }

    pub fn with(
        memory: Rc<dyn Memory>,
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        name: Option<Name>,
        describing: bool,
    ) -> Self {
        TilePuzzle {
            memory,
            layouts,
            image_of,
            name,
            describing,
            pieces: IndexMap::new(),
            pictures: IndexMap::new(),
            backgrounds: IndexMap::new(),
            shown: Cell::new(0),
        }
    }

    /// The boards, pictures and backgrounds a sweep found, by class: all of them, or with
    /// `adding` the new ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let kept = |objects: &IndexMap<i64, i64>| {
            if adding {
                objects.clone()
            } else {
                IndexMap::new()
            }
        };
        let (mut pieces, mut pictures, mut backgrounds) = (
            kept(&self.pieces),
            kept(&self.pictures),
            kept(&self.backgrounds),
        );
        for (&at, name) in found {
            let tracked = match name.as_str() {
                CLASS => &mut pieces,
                PICTURE => &mut pictures,
                BACKGROUND => &mut backgrounds,
                _ => continue,
            };
            tracked.insert(at, u32(&self.memory.read(at, 4)?, 0)?);
        }
        (self.pieces, self.pictures) = (pieces, pictures);
        self.backgrounds = backgrounds;
        Ok(())
    }

    /// Another app has started: the next puzzle's picture is new, even one of the same
    /// Pokémon.
    pub fn started(&self) {
        self.shown.set(0);
    }

    /// The picture's Pokémon the first time the puzzle holds them and its background is drawn,
    /// then None until another is drawn, by Play Again (`IMAGE`), or another app starts: each
    /// look is five reads a poll. Its `Bg` held 0 still as its Pokémon were placed in two of six
    /// replays (spike 256).
    /// The picture, each time one is drawn, as Narration's line, or only logged where
    /// descriptions are off; and its background's number, logged, as a background with no words
    /// yet is left out of the line. Drawn afresh each time the puzzle opens, so a replay's log
    /// says which three it drew, which proves it followed its movie.
    pub fn describe(
        &self,
        running: Option<&str>,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        if running != Some(APP) {
            return Ok(Vec::new());
        }
        let Some(name) = &self.name else {
            return Ok(Vec::new());
        };
        let Some(shown) = self.new_picture()? else {
            return Ok(Vec::new());
        };
        let background = self.background()?;
        let Some(said) = described(&shown, &**name, background) else {
            return Ok(Vec::new());
        };
        // `str(background)`: "None" where there is none.
        let number = background.map_or("None".to_string(), |number| number.to_string());
        Ok(vec![
            (BACKGROUND_LOG.to_string(), number, None),
            (
                PICTURE_LOG.to_string(),
                said,
                self.describing.then_some(Category::Scene),
            ),
        ])
    }

    pub fn new_picture(&self) -> Result<Option<Picture>> {
        let Some((number, image)) = self.drawn()? else {
            return Ok(None);
        };
        if number == 0 || image == 0 || image == self.shown.get() || self.board()?.is_none() {
            return Ok(None);
        }
        let found = self.picture()?;
        if found.is_some() {
            self.shown.set(image);
        }
        Ok(found)
    }

    /// The picture's Pokémon, each with where it stands, or None while none holds a record
    /// that checks, the module's own data holding words like a PokeModel's vtable, or one stands
    /// at z 0: placed, they stood 200 to 335 deep, and found as the puzzle opened all three stood
    /// at 0 (or_amie_tile_puzzle_easy_described).
    pub fn picture(&self) -> Result<Option<Picture>> {
        'pictures: for (&at, &vtable) in &self.pictures {
            let raw = self.memory.read(at, PICTURE_SIZE)?;
            if u32(&raw, 0)? != vtable {
                continue;
            }
            let mut shown: Picture = Vec::new();
            for (i, &model) in MODELS.iter().enumerate() {
                let param = u32(&raw, model)?;
                if param == 0 {
                    continue;
                }
                let record = u32(&self.memory.read(param + pokemon::PARAM_RECORD, 4)?, 0)?;
                let plain = pokemon::decrypt(&self.memory.read(record, pokemon::RECORD_SIZE)?);
                let found = match plain {
                    Some(plain) => pokemon::parse(&plain, pokemon::SPECIES_MAX)?,
                    None => None,
                };
                let Some(found) = found else {
                    continue 'pictures;
                };
                let stands = POINTS + i as i64 * 0x30 + STANDS;
                let (x, z) = (f32(&raw, stands)?, f32(&raw, stands + 8)?);
                if z == 0.0 {
                    continue 'pictures;
                }
                shown.push((found, x, z));
            }
            if !shown.is_empty() {
                return Ok(Some(shown));
            }
        }
        Ok(None)
    }

    /// The number of the background drawn, 0 until it is, or None where no background is
    /// found. The module's own code holds words like a Bg's vtable too, three of them, their
    /// number an instruction (or_amie_tile_puzzle_easy): a number is a byte held at `COPY` as
    /// well, and one there not yet is still being drawn.
    pub fn background(&self) -> Result<Option<i64>> {
        Ok(self.drawn()?.map(|(number, _)| number))
    }

    /// The number of the background drawn, 0 until it is, and its image (`IMAGE`); None
    /// where no background is found (`background`).
    fn drawn(&self) -> Result<Option<(i64, i64)>> {
        let mut drawing = None;
        for (&at, &vtable) in &self.backgrounds {
            let raw = self.memory.read(at, DRAWN + 4)?;
            if u32(&raw, 0)? != vtable {
                continue;
            }
            let number = u32(&raw, DRAWN)?;
            if number > 0xFF {
                continue;
            }
            if number != 0 && i64::from(raw[COPY as usize]) == number {
                return Ok(Some((number, u32(&raw, IMAGE)?)));
            }
            drawing = Some((0, 0));
        }
        Ok(drawing)
    }

    /// Is the player in the middle of play, a piece away from home? The board lies whole until
    /// Begin shuffles it, and once solved.
    pub fn playing(&self) -> Result<bool> {
        let board = self.board()?;
        Ok(board.is_some_and(|board| board.tiles.iter().any(|(slot, (home, _))| home != slot)))
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
    pub fn name(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        if running != Some(APP) || (self.image_of)(picture)?.as_deref() != Some(FRAME) {
            return Ok(None);
        }
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
