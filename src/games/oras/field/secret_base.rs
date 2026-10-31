//! ORAS's Secret Bases: every spot one can be made at, and which is the player's (spike 154).
//!
//! No zone's data has a spot: Route 111's tree at (381, 116), the player's base, is a blocked cell
//! like any tree's, with no record and no move model, and the vine on it is a model the field draws
//! from a list of its own. The executable lists all 85 spots in one table, 20 bytes a spot:
//!
//! ```text
//! +0x00 u16 the spot's number, from 0 in order, which the save keeps for the player's base
//! +0x02 u8  its kind: 0 for 60 spots, 1 and 2 for 12 each, 3 for one; Route 111's tree is 2
//! +0x03 u8  the base's shape inside, 0 to 50: a zone of its own each, all "Secret Base"
//! +0x04 s32 x and +0x08 s32 z, the cell of the entrance; +0x0c s32 its height in world units
//! +0x10 u16 the zone, +0x12 u16 0x218 in 83 of the 85 rows
//! ```
//!
//! It is found by its shape, since where it is is the build's: the longest run of rows numbered
//! from 0 whose zone and cell are in range.
//!
//! The save keeps the player's own base as `savedata::SecretBase`, one all session, with the spot's
//! number at +0x32e, 0 where the player has none (PKHeX's format, from the 0x324 bytes before it).
//! After the player's base, 0x314 bytes from +0x32c, come 30 of other players', loaded by QR Code
//! and otherwise, 0x3e0 bytes each (or_secret_base_qr, read in the save itself):
//!
//! ```text
//! +0x002 s16 the spot's number, -1 for an empty slot
//! +0x21a     the owner's name, UTF-16 to a zero, 0x1a bytes
//! +0x234     the team's name the same way, 0x22 bytes; empty where the owner named none
//! ```
//!
//! Inside a base, the base's module (`DllUSSecretBase`) keeps what stands where and the cursor the
//! player decorates with (omega_ruby_secret-base2.ctm):
//!
//! ```text
//! field::secretBase::SecretBaseMapper   +0x04 u32 width and +0x08 u32 height, 40 by 40
//!                                       +0x18, +0x1c, +0x20 -> a layer: (cells, width, height)
//! a layer's cell                        u32 the decoration's model, u32 its row in the save;
//!                                       zero where nothing stands. Cell (x, z) is the
//!                                       z * 39 + x'th, measured at (14, 12), (18, 12), (14, 13)
//! a decoration's model                  +0x00 its class's vtable, +0x04 u32 the decoration
//! field::secretBase::SecretBaseCursorModel
//!                                       +0x04 u32 `MODES`: 0 walking, 1 an item held where it
//!                                       fits, 2 where it does not, 3 before the player first
//!                                       decorates
//!                                       +0x08 u32 1 while it is shown
//!                                       +0x30 u16 x and +0x32 u16 z, the cell it is on
//! ```
//!
//! +0x1c is the floor, where the desk, the chair and the mats stood; +0x18 and +0x20, empty in
//! every read so far, are taken for the wall's posters and what stands on a desk. The cell table
//! knows nothing of any of it: the desk's cell stayed open with the desk in it, and P went through
//! it.

use std::cell::{Cell as Slot, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::OnceLock;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{self, s16, u16, u32};
use crate::core::discovery::{Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::core::pathfind::Cell;
use crate::games::common::field::things::{Interactable, SECRET_BASE_SOURCE, place_source};
use crate::games::gen6::field::features::{Feature, SecretBaseReader, Tracked, ZoneAt};
use crate::games::gen6::rom::archives::GameText;
use crate::py::{self, Error, Result};

/// What a spot a Secret Base can be made at is called: the game draws a mark on the wall, the tree
/// or the bush, and says what it is on facing it, but never names the spot (spike 154). One another
/// player's base is loaded at is called as the game calls the base (`SecretBases.shared`).
pub const BASE_SPOT: &str = "secret base spot";

/// What the player's own Secret Base's spot is called, said with the place it leads into, as an
/// exit is: "entrance to Secret Base".
pub const ENTRANCE: &str = "entrance";

/// `struct.Struct("<HBBiiiHH")`: a spot's number, kind, shape, x, z, height, zone and the last
/// word.
pub struct Spot;

pub const SPOT: Spot = Spot;

impl Spot {
    /// `SPOT.size`.
    pub fn size(&self) -> i64 {
        20
    }

    /// `SPOT.unpack_from(blob, offset)`.
    #[track_caller]
    pub fn unpack_from(&self, blob: &[u8], offset: i64) -> Result<[i64; 8]> {
        if offset < 0 || offset + self.size() > blob.len() as i64 {
            return Err(Error::unpack(format!(
                "unpack_from requires a buffer of at least {} bytes for unpacking {} bytes at \
                 offset {offset} (actual buffer size is {})",
                offset + self.size(),
                self.size(),
                blob.len()
            )));
        }
        Ok([
            binary::u16(blob, offset)?,
            i64::from(blob[offset as usize + 2]),
            i64::from(blob[offset as usize + 3]),
            binary::s32(blob, offset + 4)?,
            binary::s32(blob, offset + 8)?,
            binary::s32(blob, offset + 12)?,
            binary::u16(blob, offset + 16)?,
            binary::u16(blob, offset + 18)?,
        ])
    }

    /// `SPOT.pack(*values)`: a value out of its field's range is `struct.error`.
    #[track_caller]
    pub fn pack(&self, values: [i64; 8]) -> Result<Vec<u8>> {
        const FIELDS: [(char, usize, i64, i64); 8] = [
            ('H', 2, 0, 0xFFFF),
            ('B', 1, 0, 0xFF),
            ('B', 1, 0, 0xFF),
            ('i', 4, -0x8000_0000, 0x7FFF_FFFF),
            ('i', 4, -0x8000_0000, 0x7FFF_FFFF),
            ('i', 4, -0x8000_0000, 0x7FFF_FFFF),
            ('H', 2, 0, 0xFFFF),
            ('H', 2, 0, 0xFFFF),
        ];
        let mut out = Vec::new();
        for (value, (code, width, least, most)) in values.into_iter().zip(FIELDS) {
            if !(least..=most).contains(&value) {
                return Err(Error::unpack(format!(
                    "'{code}' format requires {least} <= number <= {most}"
                )));
            }
            out.extend_from_slice(&value.to_le_bytes()[..width]);
        }
        Ok(out)
    }
}

/// Fewer rows than this in a run is a coincidence of the shape, not the table.
pub const SPOTS_LEAST: i64 = 32;
pub const ZONE_LIMIT: i64 = 0x400;
pub const MATRIX_CELLS: i64 = 0x400;
pub const KINDS: i64 = 4;

/// The zone inside Route 111's base, whose place is "Secret Base", as is each of the 52 zones a
/// base's shape is built in (0x190 to 0x1ab and 0x1df to 0x1f6): one stands for all of them where
/// only the place is said.
pub const INSIDE: i64 = 0x198;

/// The 51 zones a Secret Base is built in, one for each shape inside, all of whose place is "Secret
/// Base" (spike 154). 0x1f6 is a 52nd of that place, with no records at all.
///
/// Python's is a `frozenset` that `oras`'s tables are built by iterating. Its order there is the
/// slots' order, which for these ints in a table of 128 or more is ascending, as listed here.
pub fn zones() -> &'static IndexSet<i64> {
    static ZONES: OnceLock<IndexSet<i64>> = OnceLock::new();
    ZONES.get_or_init(|| (0x190..0x1AC).chain(0x1DF..0x1F6).collect())
}

pub const MAPPER_CLASS: &str = "N5field10secretBase16SecretBaseMapperE";
pub const CURSOR_CLASS: &str = "N5field10secretBase21SecretBaseCursorModelE";
/// The decorations' classes stood on rather than walked round: Your Location, which the player
/// walked onto, the Welcome Mats beside it, the one whose class says it is not hit, and the note
/// and glitter mats. Every other decoration is taken to be solid, as the Small Desk was: the player
/// walked round it where P went through it. The panels and mats that move the player are among
/// them, so a way keeps off them.
pub const WALKED_ON: [&str; 5] = [
    "N5field10secretBase33SecretBaseGoodsModelJibunNoIbasyoE",
    "N5field10secretBase30SecretBaseGoodsModelOmanekiMatE",
    "N5field10secretBase26SecretBaseGoodsModelNotHitE",
    "N5field10secretBase27SecretBaseGoodsModelOnpuMatE",
    "N5field10secretBase31SecretBaseGoodsModelKirakiraMatE",
];
// ponytail: solid by class, measured for the desk and Your Location alone; a mat found solid, or
// a desk found walked through, moves its class in or out of WALKED_ON.
/// The Invisible Doll's class: the game hides the doll until it is examined, so it is never
/// listed (the user, 2026-10-10).
pub const UNSEEN: &str = "N5field10secretBase37SecretBaseGoodsModelSuperKakureonDollE";

/// The mapper's layers, said in this order: what stands on a desk, the floor, the wall.
pub const LAYERS: [i64; 3] = [0x20, 0x1C, 0x18];
pub const FLOOR: i64 = 0x1C;
pub const GRID: i64 = 40;
/// How far apart a layer's rows are, in cells: one less than its width (see the docstring).
pub const ROW: i64 = 39;
pub const CELL: i64 = 8;
pub const CURSOR_SHOWN: i64 = 0x08;
pub const CURSOR_CELL: i64 = 0x30;
/// 0 walking the floor, 1 an item held where it can be set down, the game drawing its squares
/// green, 2 one held where it cannot, drawn red, and 3 as the base is built, before the player
/// first decorates. 1 and 2 turn with the cursor's cell, in the same frame
/// (omega_ruby_secret-base5.ctm).
pub const MODES: i64 = 4;
pub const FITS: i64 = 1;
pub const REFUSED: i64 = 2;

pub const SAVE_CLASS: &str = "N8savedata10SecretBaseE";
/// Where `savedata::SecretBase` keeps the number of the player's own spot: 8, Route 111's tree,
/// in both heaps read of omega_ruby_secret-base.ctm, on the route and in the base.
pub const OWN_SPOT: i64 = 0x32E;
/// Where `savedata::SecretBase` keeps the bases loaded from other players (see the module doc).
pub const SHARED: i64 = 0x640;
pub const SHARED_SIZE: i64 = 0x3E0;
pub const SHARED_COUNT: i64 = 30;
pub const SHARED_SPOT: i64 = 0x02;
pub const OWNER: i64 = 0x21A;
pub const TEAM: i64 = 0x234;
pub const OWNER_SIZE: i64 = 0x1A;
pub const TEAM_SIZE: i64 = 0x22;

/// Where `_FIRST_ROWS` matches: `(?=\x00\x00.{18}\x01\x00.{18}\x02\x00)` is a look-ahead, which
/// the `regex` crate has none of, so the three row numbers are tested at each offset by hand.
/// Python's `finditer` yields a match at each offset the look-ahead holds at.
fn first_rows(image: &[u8]) -> Vec<usize> {
    let row = SPOT.size() as usize;
    let mut out = Vec::new();
    if image.len() < 2 * row + 2 {
        return out;
    }
    for at in 0..=image.len() - (2 * row + 2) {
        if image[at..at + 2] == [0, 0]
            && image[at + row..at + row + 2] == [1, 0]
            && image[at + 2 * row..at + 2 * row + 2] == [2, 0]
        {
            out.push(at);
        }
    }
    out
}

/// (zone, cell) of every spot the executable lists, by its number, or none where it lists
/// none.
pub fn spots(image: &[u8]) -> Vec<(i64, Cell)> {
    let mut best: Vec<(i64, Cell)> = Vec::new();
    for start in first_rows(image) {
        let mut run: Vec<(i64, Cell)> = Vec::new();
        let mut at = start as i64;
        while at <= image.len() as i64 - SPOT.size() {
            let [number, kind, _shape, x, z, _height, zone, _] = SPOT
                .unpack_from(image, at)
                .expect("the loop's bound leaves a whole row");
            if number != run.len() as i64
                || kind >= KINDS
                || !(0 < zone && zone < ZONE_LIMIT)
                || !(0..MATRIX_CELLS).contains(&x)
                || !(0..MATRIX_CELLS).contains(&z)
            {
                break;
            }
            run.push((zone, (x, z)));
            at += SPOT.size();
        }
        if run.len() > best.len() {
            best = run;
        }
    }
    if best.len() as i64 >= SPOTS_LEAST {
        best
    } else {
        Vec::new()
    }
}

/// Every spot, out of the executable, and the player's own, out of the save; inside a base,
/// what stands in each cell and where the cursor the player decorates with is.
pub struct SecretBases {
    pub classes: Vec<&'static str>,
    pub inside: i64,
    pub zones: &'static IndexSet<i64>,
    memory: Rc<dyn Memory>,
    names: Vec<(String, String)>,
    pub spots: Vec<(i64, Cell)>,
    save: Slot<Option<i64>>,
    mapper: Slot<Option<i64>>,
    cursor: Slot<Option<i64>>,
    /// The vtables of `WALKED_ON`'s classes, as the sweeps found them.
    walked_on: RefCell<HashSet<i64>>,
    /// `UNSEEN`'s vtable, as the sweeps found it.
    unseen: RefCell<HashSet<i64>>,
}

impl SecretBases {
    pub fn new(memory: Rc<dyn Memory>, image: &[u8], text: &GameText) -> Self {
        let mut classes = vec![SAVE_CLASS, MAPPER_CLASS, CURSOR_CLASS];
        classes.extend(WALKED_ON);
        classes.push(UNSEEN);
        SecretBases {
            classes,
            inside: INSIDE,
            zones: zones(),
            memory,
            names: text.base_names.clone(),
            spots: spots(image),
            save: Slot::new(None),
            mapper: Slot::new(None),
            cursor: Slot::new(None),
            walked_on: RefCell::new(HashSet::new()),
            unseen: RefCell::new(HashSet::new()),
        }
    }

    pub fn track(&self, found: &Found) -> Result<()> {
        let save = found.of(&[SAVE_CLASS]);
        if let Some(&first) = save.first() {
            self.save.set(Some(first));
        }
        // The module's image is copied into the heap as it loads, vtables and all, and a sweep
        // finds every class of it there: a find is taken only in the shape of the object.
        let mut mappers: Vec<i64> = Vec::new();
        for at in found.of(&[MAPPER_CLASS]) {
            if self.layers(at)?.is_some() {
                mappers.push(at);
            }
        }
        if let Some(&last) = mappers.last() {
            self.mapper.set(Some(last));
        }
        let mut cursors: Vec<i64> = Vec::new();
        for at in found.of(&[CURSOR_CLASS]) {
            if self.cursor_head(at)?.is_some() {
                cursors.push(at);
            }
        }
        if let Some(&last) = cursors.last() {
            self.cursor.set(Some(last));
        }
        for at in found.of(&WALKED_ON) {
            let vtable = u32(&self.memory.read(at, 4)?, 0)?;
            self.walked_on.borrow_mut().insert(vtable);
        }
        for at in found.of(&[UNSEEN]) {
            let vtable = u32(&self.memory.read(at, 4)?, 0)?;
            self.unseen.borrow_mut().insert(vtable);
        }
        Ok(())
    }

    /// Whether a sweep has found what stands where in a base.
    pub fn furnished(&self) -> Result<bool> {
        match self.mapper.get() {
            None => Ok(false),
            Some(mapper) => Ok(self.layers(mapper)?.is_some()),
        }
    }

    /// The cell the decorating cursor is on and whether the item held can be set down there,
    /// None for nothing held; or None while it is not shown. One read, so the two agree.
    pub fn cursor(&self) -> Result<Option<(Cell, Option<bool>)>> {
        let head = match self.cursor.get() {
            None => None,
            Some(cursor) => self.cursor_head(cursor)?,
        };
        Ok(match head {
            Some((true, cell, mode)) => Some((
                cell,
                match mode {
                    FITS => Some(true),
                    REFUSED => Some(false),
                    _ => None,
                },
            )),
            _ => None,
        })
    }

    /// The decorations standing in a cell, by number, on a desk first, the wall last.
    pub fn standing(&self, cell: Cell) -> Result<Vec<i64>> {
        let layers = match self.mapper.get() {
            None => None,
            Some(mapper) => self.layers(mapper)?,
        };
        let index = index(cell);
        let (Some(layers), Some(index)) = (layers, index) else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        for offset in LAYERS {
            let model = u32(&self.memory.read(layers[&offset] + index * CELL, 4)?, 0)?;
            if within(HEAP_SPAN, model) {
                out.push(u32(&self.memory.read(model + 4, 4)?, 0)?);
            }
        }
        Ok(out)
    }

    /// The floor's cells a decoration stands in that is walked round, not over.
    pub fn solid(&self) -> Result<IndexSet<Cell>> {
        let layers = match self.mapper.get() {
            None => None,
            Some(mapper) => self.layers(mapper)?,
        };
        let Some(layers) = layers else {
            return Ok(IndexSet::new());
        };
        let floor = self.memory.read(layers[&FLOOR], GRID * GRID * CELL)?;
        let mut out = IndexSet::new();
        for z in 0..GRID {
            for x in 0..GRID {
                let model = u32(&floor, (z * ROW + x) * CELL)?;
                if within(HEAP_SPAN, model) {
                    let vtable = u32(&self.memory.read(model, 4)?, 0)?;
                    if !self.walked_on.borrow().contains(&vtable) {
                        out.insert((x, z));
                    }
                }
            }
        }
        Ok(out)
    }

    /// Each decoration standing in the base, by number, with its cells: one a model, however
    /// many cells and layers it covers. The Invisible Doll is left out (`UNSEEN`).
    pub fn placed(&self) -> Result<Vec<(i64, Vec<Cell>)>> {
        let layers = match self.mapper.get() {
            None => None,
            Some(mapper) => self.layers(mapper)?,
        };
        let Some(layers) = layers.filter(|layers| !layers.is_empty()) else {
            return Ok(Vec::new());
        };
        let mut models: IndexMap<i64, Vec<Cell>> = IndexMap::new();
        for offset in LAYERS {
            let grid = self.memory.read(layers[&offset], GRID * GRID * CELL)?;
            for z in 0..GRID {
                for x in 0..GRID {
                    let model = u32(&grid, (z * ROW + x) * CELL)?;
                    if within(HEAP_SPAN, model) {
                        models.entry(model).or_default().push((x, z));
                    }
                }
            }
        }
        let mut heads: IndexMap<i64, Vec<u8>> = IndexMap::new();
        for &model in models.keys() {
            heads.insert(model, self.memory.read(model, 8)?);
        }
        let mut out = Vec::new();
        for (model, cells) in models {
            let head = &heads[&model];
            if !self.unseen.borrow().contains(&u32(head, 0)?) {
                out.push((u32(head, 4)?, cells));
            }
        }
        Ok(out)
    }

    /// Where each layer's cells start, or None where `mapper` is not a mapper's shape.
    fn layers(&self, mapper: i64) -> Result<Option<IndexMap<i64, i64>>> {
        let head = self
            .memory
            .read(mapper, LAYERS.into_iter().max().unwrap_or(0) + 4)?;
        if u32(&head, 4)? != GRID || u32(&head, 8)? != GRID {
            return Ok(None);
        }
        let mut out: IndexMap<i64, i64> = IndexMap::new();
        for offset in LAYERS {
            let layer = u32(&head, offset)?;
            if !within(HEAP_SPAN, layer) {
                return Ok(None);
            }
            let shape = self.memory.read(layer, 12)?;
            if !within(HEAP_SPAN, u32(&shape, 0)?)
                || (u32(&shape, 4)?, u32(&shape, 8)?) != (GRID, GRID)
            {
                return Ok(None);
            }
            out.insert(offset, u32(&shape, 0)?);
        }
        Ok(Some(out))
    }

    /// Whether the cursor is shown, its cell and its mode, or None where `cursor` is not its
    /// shape.
    fn cursor_head(&self, cursor: i64) -> Result<Option<(bool, Cell, i64)>> {
        let head = self.memory.read(cursor, CURSOR_CELL + 4)?;
        let (x, z) = (u16(&head, CURSOR_CELL)?, u16(&head, CURSOR_CELL + 2)?);
        if u32(&head, 4)? >= MODES || u32(&head, CURSOR_SHOWN)? > 1 || x >= GRID || z >= GRID {
            return Ok(None);
        }
        Ok(Some((
            u32(&head, CURSOR_SHOWN)? != 0,
            (x, z),
            u32(&head, 4)?,
        )))
    }

    /// The number of the player's own spot, or None while the player has none or the save is
    /// not found yet.
    /// The zone's spots: the player's own base as its entrance, leading inside, another player's
    /// by the game's name for it, and every other spot as a spot (spike 154). A spot is a tree, a
    /// wall or a bush, walked up to and faced as a sign is.
    pub fn listed(&self, zone: i64, _zone_at: ZoneAt) -> Result<Vec<Interactable>> {
        let own = self.own()?;
        let shared = if (0..self.spots.len() as i64)
            .any(|number| self.spots[number as usize].0 == zone && Some(number) != own)
        {
            self.shared()?
        } else {
            HashMap::new()
        };
        spots_in(&self.spots, own, &shared, self.inside, zone)
    }

    /// What the game calls each base loaded from another player, by its spot's number: by its
    /// team's name, or by its owner's where the team has none ("Team Tarin's Base"). None before
    /// the save is found, or without the game's words for them.
    pub fn shared(&self) -> Result<HashMap<i64, String>> {
        let Some(save) = self.save.get() else {
            return Ok(HashMap::new());
        };
        if self.names.len() < 2 {
            return Ok(HashMap::new());
        }
        let data = self
            .memory
            .read(save + SHARED, SHARED_SIZE * SHARED_COUNT)?;
        let mut out: HashMap<i64, String> = HashMap::new();
        for at in (0..data.len() as i64).step_by(SHARED_SIZE as usize) {
            let number = s16(&data, at + SHARED_SPOT)?;
            let team = name(py::slice(&data, at + TEAM, at + TEAM + TEAM_SIZE));
            let owner = name(py::slice(&data, at + OWNER, at + OWNER + OWNER_SIZE));
            let ((before, after), name) = if !team.is_empty() {
                (&self.names[1], team)
            } else {
                (&self.names[0], owner)
            };
            if 0 <= number && number < self.spots.len() as i64 && !name.is_empty() {
                out.insert(number, format!("{before}{name}{after}"));
            }
        }
        Ok(out)
    }

    pub fn own(&self) -> Result<Option<i64>> {
        let Some(save) = self.save.get() else {
            return Ok(None);
        };
        let number = s16(&self.memory.read(save + OWN_SPOT, 2)?, 0)?;
        Ok(if 0 < number && number < self.spots.len() as i64 {
            Some(number)
        } else {
            None
        })
    }
}

/// A name the save keeps as UTF-16 to a zero.
fn name(raw: &[u8]) -> String {
    let text = py::decode_utf16le(raw);
    py::strip(text.split('\0').next().unwrap_or("")).to_string()
}

/// Which of a layer's cells `cell` is, or None off the base's grid.
fn index(cell: Cell) -> Option<i64> {
    let (x, z) = cell;
    if (0..GRID).contains(&x) && (0..GRID).contains(&z) {
        Some(z * ROW + x)
    } else {
        None
    }
}

impl Tracked for SecretBases {
    fn classes(&self) -> Vec<&'static str> {
        self.classes.clone()
    }

    fn track(&self, found: &Found) -> Result<()> {
        SecretBases::track(self, found)
    }
}

impl Feature for SecretBases {
    fn listed(&self, zone: i64, zone_at: ZoneAt) -> Result<Vec<Interactable>> {
        SecretBases::listed(self, zone, zone_at)
    }
}

impl SecretBaseReader for SecretBases {
    fn zones(&self) -> IndexSet<i64> {
        self.zones.clone()
    }

    fn furnished(&self) -> Result<bool> {
        SecretBases::furnished(self)
    }

    fn cursor(&self) -> Result<Option<(Cell, Option<bool>)>> {
        SecretBases::cursor(self)
    }

    fn standing(&self, cell: Cell) -> Result<Vec<i64>> {
        SecretBases::standing(self, cell)
    }

    fn solid(&self) -> Result<IndexSet<Cell>> {
        SecretBases::solid(self)
    }

    fn placed(&self) -> Result<Vec<(i64, Vec<Cell>)>> {
        SecretBases::placed(self)
    }
}

/// `SecretBases.listed` of every spot, the player's own, other players' names for theirs and the
/// zone inside: a function of its own, since the tests' Secret Bases, which subclass `SecretBases`
/// in Python, list theirs so too.
pub fn spots_in(
    spots: &[(i64, Cell)],
    own: Option<i64>,
    shared: &HashMap<i64, String>,
    inside: i64,
    zone: i64,
) -> Result<Vec<Interactable>> {
    let mut out: Vec<Interactable> = Vec::new();
    for (number, &(home, cell)) in spots.iter().enumerate() {
        let number = number as i64;
        if home != zone {
            continue;
        }
        out.push(if Some(number) == own {
            Interactable {
                zone: Some(inside),
                source: place_source(SECRET_BASE_SOURCE, number)?,
                home: Some(home),
                ..Interactable::new("exits", ENTRANCE, cell)
            }
        } else {
            Interactable {
                source: place_source(SECRET_BASE_SOURCE, number)?,
                home: Some(home),
                ..Interactable::new(
                    "exits",
                    shared.get(&number).map_or(BASE_SPOT, String::as_str),
                    cell,
                )
            }
        });
    }
    Ok(out)
}
