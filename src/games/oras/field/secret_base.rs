//! ORAS's Secret Bases: every spot one can be made at, and which is the player's (spike 154), one
//! of the field's features (`SecretBases`); and inside a base, what stands where and the cursor the
//! player decorates with (`Decorations`), the watch of a landform, a base's floor
//! (`SecretBaseFloor`), and a field voice, the cursor's line. Gen 6 names none of it.
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
//! +0x1c is the floor, where the desk, the chair and the mats stood; +0x18 holds the big mats that
//! lie under it, the Spikes Mat's and the Thunder Mat's three cells by three, which the player
//! walked across (or_secret_base_objects); +0x20, empty in every read so far, is taken for what
//! stands on a desk. The cell table knows nothing of any of it: the desk's cell stayed open with
//! the desk in it, and P went through it.
//!
//! Whether a floor decoration's cell stops the player is its class's to say, in
//! DllUSSecretBase.cro: the generic `SecretBaseGoodsModel`'s hit test (vtable slot 5) is true on
//! every cell of its box, and slot 12 sets the blocked bit where it hits, in the word the game
//! works out for the cell as the player moves, over the cell table's own, which stays as the ROM
//! has it. The classes `WALKED_ON` lists return false on every cell, or set cells of their own
//! whose kinds the executable's table has open; the balloon class is hit only at the Breakable
//! Door's two ends (`DOOR`). Spike 281 reads it all off the ROM.

use std::cell::{Cell as Slot, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::OnceLock;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{self, s16, u16, u32};
use crate::core::discovery::{Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::core::pathfind::Cell;
use crate::core::speech::Category;
use crate::core::words::own;
use crate::games::common::field::ground::{Ground, walkable};
use crate::games::common::field::things::{
    DECORATION_SOURCE, Interactable, SECRET_BASE_SOURCE, TILE_SOURCE, bearing, kind_said,
    place_source,
};
use crate::games::gen6::field::features::{Feature, Sweeping, Tracked, ZoneAt};
use crate::games::gen6::field::landform::{
    Found as Worked, HOLE, Here, Landform, as_watch, cell_number, spread, touching,
};
use crate::games::gen6::field::voice::{Said, Speaking};
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
/// The decorations' classes stood on rather than walked round, by the game's own hit test (spike
/// 281): Your Location, the Welcome Mats, the cushions and plain mats whose class is not hit, the
/// note and glitter mats, the Stand, the Stairs, the Tall Grass, the Solid Board and the Pitfall
/// Mat. The player walked six of each Stand's eight cells, and fell into the Pitfall Mat once and
/// was left on it, then crossed it (or_secret_base_objects). Every other class is solid to P:
/// those the game hits, as the Small Desk was walked round where P went through it, and the panels
/// and mats that move the player, which the game does not hit but a way keeps off. The Slide, the
/// tents and the Paradoxical Popper hit some cells and not others, and are kept whole.
pub const WALKED_ON: [&str; 10] = [
    "N5field10secretBase33SecretBaseGoodsModelJibunNoIbasyoE",
    "N5field10secretBase30SecretBaseGoodsModelOmanekiMatE",
    "N5field10secretBase26SecretBaseGoodsModelNotHitE",
    "N5field10secretBase27SecretBaseGoodsModelOnpuMatE",
    "N5field10secretBase31SecretBaseGoodsModelKirakiraMatE",
    "N5field10secretBase29SecretBaseGoodsModelOokinaDaiE",
    "N5field10secretBase26SecretBaseGoodsModelKaidanE",
    "N5field10secretBase28SecretBaseGoodsModelKusamuraE",
    "N5field10secretBase30SecretBaseGoodsModelJoubunaItaE",
    "N5field10secretBase29SecretBaseGoodsModelAnaakiMatE",
];
pub const BALLOONS: &str = "N5field10secretBase26SecretBaseGoodsModelFuusenE";
/// The Breakable Door, a decoration of the balloons' class, which the game hits at its two end
/// cells alone: the player walked through its middle (or_secret_base_objects). The balloons
/// themselves are never hit.
pub const DOOR: i64 = 0x96;
/// The Invisible Doll's class: the game hides the doll until it is examined, so it is never
/// listed.
pub const UNSEEN: &str = "N5field10secretBase37SecretBaseGoodsModelSuperKakureonDollE";

/// The mapper's layers, said in this order: what stands on a desk, the floor, the mats under it.
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

/// What a Secret Base's flag beside its PC is called: the game never names it where it stands
/// (`oras.FURNITURE`).
pub const FLAG: &str = "flag";

/// The cell kind of a Secret Base's crumbled cliff, where the Stairs are placed to climb to the
/// upper floor: always blocked, one patch of 3 cells by 2 a base (Route 116's, zone 0x1a7, at x 17
/// to 19, z 17 to 18, its top row at the upper floor's height 36 and its bottom at 18, the lower
/// floor 0). The 857 ORAS chunk files have it only in base shapes 11, 12, 21, 22, 23, 40 and 41,
/// the 13 spots that need Stairs, and X's 1030 have none (spike 201).
pub const CLIFF_TILE: i64 = 0xFD;

/// What a Secret Base's crumbled cliff is called: the game names it only in the Stairs'
/// description, "You can place this by a crumbled cliff to climb to a higher floor", never where
/// it stands (omega_ruby_secret-base3.ctm).
pub const CLIFF: &str = "crumbled cliff";

/// The cell kind of a hole in a Secret Base's floor, crossed on a Solid Board: always blocked,
/// two cells in Granite Cave's base (zone 0x1f0, at x 20, z 12 to 13), the only way into its
/// upper-left room (omega_ruby_secret-base6.ctm). The ROM has it only in base shapes 42 to 50,
/// and X's none (spike 201). Called a hole, as one a boulder drops into is (`landform.HOLE`): the
/// game names it only in the Solid Board's description, "You can place this over holes to cross
/// over them", never where it is.
pub const HOLE_TILE: i64 = 0xFE;

/// The cells of a Secret Base a decoration goes over, by kind, each with its name.
pub const BASE_TILES: [(i64, &str); 2] = [(CLIFF_TILE, CLIFF), (HOLE_TILE, HOLE)];

/// `BASE_TILES.get(kind)`.
fn base_tile(kind: i64) -> Option<&'static str> {
    BASE_TILES
        .iter()
        .find(|(tile, _)| *tile == kind)
        .map(|(_, name)| *name)
}

/// What the decorating cursor is on where nothing stands and nothing is in the way: the game says
/// nothing as it moves from one such cell to the next.
pub const BLANK: &str = "blank";

/// What the decorating cursor is on where the cell table has the cell blocked and no decoration,
/// PC or flag stands: the base's walls and the hedge round them.
pub const WALL: &str = "wall";

/// Said first while an item is held: whether it can be set down where the cursor is. The game
/// draws its squares green or red and says "You cannot set it there!" only once A is pressed
/// (omega_ruby_secret-base5.ctm).
pub fn fit(fits: bool) -> &'static str {
    if fits {
        "decorating-placeable"
    } else {
        "decorating-not-placeable"
    }
}

pub const ASKS: i64 = 5;
/// Sweeps asked for inside a Secret Base while what stands where is not found, and the seconds
/// between them. The base's module is loaded as the player goes in, and its classes are learned a
/// look after a look that lists it, too late for the sweep that look comes with.
pub const EVERY: f64 = 2.0;

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

/// Every spot, out of the executable, and the player's own and other players', out of the
/// save: the spots listed among the field's features (`gen6.field.features.Feature`).
pub struct SecretBases {
    pub classes: Vec<&'static str>,
    pub inside: i64,
    memory: Rc<dyn Memory>,
    names: Vec<(String, String)>,
    pub spots: Vec<(i64, Cell)>,
    save: Slot<Option<i64>>,
}

impl SecretBases {
    pub fn new(memory: Rc<dyn Memory>, image: &[u8], text: &GameText) -> Self {
        SecretBases {
            classes: vec![SAVE_CLASS],
            inside: INSIDE,
            memory,
            names: text.base_names.clone(),
            spots: spots(image),
            save: Slot::new(None),
        }
    }

    pub fn track(&self, found: &Found) -> Result<()> {
        let save = found.of(&[SAVE_CLASS]);
        if let Some(&first) = save.first() {
            self.save.set(Some(first));
        }
        Ok(())
    }

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

    /// The number of the player's own spot, or None while the player has none or the save is
    /// not found yet.
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

/// Inside a base, what stands in each cell and where the cursor the player decorates with is,
/// read through the base's module: the watch of a base's floor (`SecretBaseFloor`), which asks
/// for a sweep until it finds them (`sweep_wanted`) and says what the cursor is on, a field voice
/// (`said`).
///
/// What it reads through the module is a `Module` of its own, `ModuleObjects` in the game: where
/// Python's tests subclass `Decorations` to give their own reads, keeping the rest, a test gives
/// its own `Module`.
pub struct Decorations {
    pub classes: Vec<&'static str>,
    /// The zones a base is built in, inside which what stands where is read.
    pub zones: IndexSet<i64>,
    names: Vec<String>,
    module: Rc<dyn Module>,
    /// Sweeps asked for in the base the player is in so far (`ASKS`).
    asks: Slot<i64>,
    /// When the next may be asked for.
    next: Slot<f64>,
    /// The cell the cursor was last said on, and what was there (`said`).
    said: RefCell<Option<(Cell, String)>>,
}

impl Decorations {
    pub fn new(memory: Rc<dyn Memory>, text: &GameText) -> Self {
        Decorations::reading(
            Rc::new(ModuleObjects::new(memory)),
            zones().clone(),
            text.decorations.clone(),
        )
    }

    /// What stands in `zones`, read through `module`, by the game's `names` for each
    /// decoration.
    pub fn reading(module: Rc<dyn Module>, zones: IndexSet<i64>, names: Vec<String>) -> Self {
        let mut classes = vec![MAPPER_CLASS, CURSOR_CLASS];
        classes.extend(WALKED_ON);
        classes.push(BALLOONS);
        classes.push(UNSEEN);
        Decorations {
            classes,
            zones,
            names,
            module,
            asks: Slot::new(0),
            next: Slot::new(0.0),
            said: RefCell::new(None),
        }
    }

    pub fn track(&self, found: &Found) -> Result<()> {
        self.module.track(found)
    }

    /// Whether a sweep has found what stands where in a base.
    pub fn furnished(&self) -> Result<bool> {
        self.module.furnished()
    }

    /// The cell the decorating cursor is on and whether the item held can be set down there,
    /// None for nothing held; or None while it is not shown.
    pub fn cursor(&self) -> Result<Option<(Cell, Option<bool>)>> {
        self.module.cursor()
    }

    /// The decorations standing in a cell, by number, on a desk first, the wall last.
    pub fn standing(&self, cell: Cell) -> Result<Vec<i64>> {
        self.module.standing(cell)
    }

    /// The floor's cells a decoration stands in that is walked round, not over.
    pub fn solid(&self) -> Result<IndexSet<Cell>> {
        self.module.solid()
    }

    /// Each decoration standing in the base, by number, with its cells; none the game hides.
    pub fn placed(&self) -> Result<Vec<(i64, Vec<Cell>)>> {
        self.module.placed()
    }

    /// What the game calls a decoration, by its number, or None where its text has no word for
    /// it.
    pub fn named(&self, number: i64) -> Option<String> {
        if number < self.names.len() as i64 && !self.names[number as usize].is_empty() {
            Some(self.names[number as usize].clone())
        } else {
            None
        }
    }

    /// Why to sweep now rather than on schedule (`features.Sweeping`): in a base, `zone`, with
    /// what stands where not found, at most `ASKS` sweeps `EVERY` seconds apart by `clock`. Out
    /// of one, none, and the asks begin again at the next.
    pub fn sweep_wanted(
        &self,
        zone: Option<i64>,
        clock: &dyn Fn() -> f64,
    ) -> Result<Option<&'static str>> {
        if !zone.is_some_and(|zone| self.zones.contains(&zone)) {
            self.asks.set(0);
            return Ok(None);
        }
        if !self.furnished()? && self.asks.get() < ASKS {
            let now = clock();
            if now >= self.next.get() {
                self.next.set(now + EVERY);
                self.asks.set(self.asks.get() + 1);
                return Ok(Some(
                    "in a Secret Base with nothing found standing in it, sweeping for it",
                ));
            }
        }
        Ok(None)
    }

    /// What the decorating cursor is on and where, a field voice (`voice.Speaking`): said each
    /// time it moves or what is there changes, so a desk set down under it is said as it lands.
    /// The decorations standing in its cell, by the game's names for them, or the PC or the
    /// flag, or a wall, or the crumbled cliff or a hole, or blank. The game names a decoration
    /// on the bottom screen only, says "Select location" alike over the floor, the PC and the
    /// walls, and nothing at all moving from one empty cell to the next
    /// (omega_ruby_secret-base2.ctm). The cursor crossing the cliff said "wall" three times, in
    /// the session before omega_ruby_secret-base3.ctm. With an item held, `FIT` comes first.
    pub fn said(&self, here: &Here) -> Result<Option<Said>> {
        let r#where = here.r#where;
        let cursor = if self.zones.contains(&r#where.zone) {
            self.cursor()?
        } else {
            None
        };
        let mut on: Option<(Cell, String)> = None;
        if let Some((cell, fits)) = cursor {
            let under = kind_said(&self.under(here, cell)?);
            on = Some((
                cell,
                match fits {
                    None => under,
                    Some(fits) => format!("{}, {under}", own(fit(fits), &[])),
                },
            ));
        }
        if on == *self.said.borrow() {
            return Ok(None);
        }
        *self.said.borrow_mut() = on.clone();
        let Some((cell, words)) = on else {
            return Ok(None);
        };
        let said = format!("{words}, {}", bearing(r#where.cell, cell));
        Ok(Some(Said {
            words: said.clone(),
            category: Category::Menu,
            logged: format!("cursor: {said}"),
        }))
    }

    /// What is in the cursor's `cell` (`said`): the decorations, else the furniture the field
    /// names, else blank or what blocks it.
    fn under(&self, here: &Here, cell: Cell) -> Result<String> {
        let mut names: Vec<String> = Vec::new();
        for number in self.standing(cell)? {
            if let Some(name) = self.named(number) {
                names.push(name);
            }
        }
        if !names.is_empty() {
            return Ok(names.join(", "));
        }
        if let Some(called) = (here.furniture)(cell)? {
            return Ok(called);
        }
        let chunks = (here.chunks)()?;
        if walkable(&chunks, cell)? != Some(false) {
            return Ok(BLANK.to_string());
        }
        let Some(held) = chunks.iter().find(|chunk| chunk.holds(cell)) else {
            return Ok(WALL.to_string());
        };
        Ok(base_tile(held.kind(cell)?).unwrap_or(WALL).to_string())
    }
}

/// What `Decorations` reads through the base's module.
pub trait Module {
    fn track(&self, found: &Found) -> Result<()>;

    fn furnished(&self) -> Result<bool>;

    fn cursor(&self) -> Result<Option<(Cell, Option<bool>)>>;

    fn standing(&self, cell: Cell) -> Result<Vec<i64>>;

    fn solid(&self) -> Result<IndexSet<Cell>>;

    fn placed(&self) -> Result<Vec<(i64, Vec<Cell>)>>;
}

/// The base's module's objects a sweep finds: its mapper, which keeps what stands where, the
/// decorating cursor, and the decorations' classes (see the module doc).
pub struct ModuleObjects {
    memory: Rc<dyn Memory>,
    mapper: Slot<Option<i64>>,
    cursor: Slot<Option<i64>>,
    /// The vtables of `WALKED_ON`'s classes, as the sweeps found them.
    walked_on: RefCell<HashSet<i64>>,
    /// `BALLOONS`'s vtable, as the sweeps found it.
    balloons: RefCell<HashSet<i64>>,
    /// `UNSEEN`'s vtable, as the sweeps found it.
    unseen: RefCell<HashSet<i64>>,
}

impl ModuleObjects {
    pub fn new(memory: Rc<dyn Memory>) -> Self {
        ModuleObjects {
            memory,
            mapper: Slot::new(None),
            cursor: Slot::new(None),
            walked_on: RefCell::new(HashSet::new()),
            balloons: RefCell::new(HashSet::new()),
            unseen: RefCell::new(HashSet::new()),
        }
    }

    pub fn track(&self, found: &Found) -> Result<()> {
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
        for at in found.of(&[BALLOONS]) {
            let vtable = u32(&self.memory.read(at, 4)?, 0)?;
            self.balloons.borrow_mut().insert(vtable);
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
        let mut models: IndexMap<i64, Vec<Cell>> = IndexMap::new();
        for z in 0..GRID {
            for x in 0..GRID {
                let model = u32(&floor, (z * ROW + x) * CELL)?;
                if within(HEAP_SPAN, model) {
                    models.entry(model).or_default().push((x, z));
                }
            }
        }
        let mut out = IndexSet::new();
        for (model, mut cells) in models {
            let head = self.memory.read(model, 8)?;
            if self.walked_on.borrow().contains(&u32(&head, 0)?) {
                continue;
            }
            if self.balloons.borrow().contains(&u32(&head, 0)?) {
                cells = if u32(&head, 4)? == DOOR {
                    vec![cells[0], cells[cells.len() - 1]]
                } else {
                    Vec::new()
                };
            }
            out.extend(cells);
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

impl Tracked for Decorations {
    fn classes(&self) -> Vec<&'static str> {
        self.classes.clone()
    }

    fn track(&self, found: &Found) -> Result<()> {
        Decorations::track(self, found)
    }

    fn sweeping(&self) -> Option<&dyn Sweeping> {
        Some(self)
    }

    fn speaking(self: Rc<Self>) -> Option<Rc<dyn Speaking>> {
        Some(self)
    }
}

impl Sweeping for Decorations {
    fn sweep_wanted(
        &self,
        zone: Option<i64>,
        clock: &dyn Fn() -> f64,
    ) -> Result<Option<&'static str>> {
        Decorations::sweep_wanted(self, zone, clock)
    }
}

impl Speaking for Decorations {
    fn said(&self, here: &Here) -> Result<Option<Said>> {
        Decorations::said(self, here)
    }
}

impl Module for ModuleObjects {
    fn track(&self, found: &Found) -> Result<()> {
        ModuleObjects::track(self, found)
    }

    fn furnished(&self) -> Result<bool> {
        ModuleObjects::furnished(self)
    }

    fn cursor(&self) -> Result<Option<(Cell, Option<bool>)>> {
        ModuleObjects::cursor(self)
    }

    fn standing(&self, cell: Cell) -> Result<Vec<i64>> {
        ModuleObjects::standing(self, cell)
    }

    fn solid(&self) -> Result<IndexSet<Cell>> {
        ModuleObjects::solid(self)
    }

    fn placed(&self) -> Result<Vec<(i64, Vec<Cell>)>> {
        ModuleObjects::placed(self)
    }
}

/// A Secret Base's floor as its decorations leave it, read through the base's module, its watch
/// (`Decorations`): the crumbled cliff and holes listed under exits, each decoration among the
/// signs by the game's name for it, and on the ground those walked round standing in the way and
/// the cliff and holes a decoration covers walked over. The cell table knows nothing of the
/// decorations: the desk's cell stayed open with the desk in it, and P went through it
/// (omega_ruby_secret-base2.ctm). Its tiles are read off the chunks loaded now, as the field reads
/// its use tiles, not worked out off the ROM's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecretBaseFloor;

impl Landform for SecretBaseFloor {
    fn watch(
        &self,
        memory: Rc<dyn Memory>,
        image: &[u8],
        text: &GameText,
    ) -> Option<Rc<dyn Tracked>> {
        // Read only with the executable, as the spots are.
        if image.is_empty() {
            None
        } else {
            Some(Rc::new(Decorations::new(memory, text)))
        }
    }

    fn listed(&self, _found: &Worked, here: &Here) -> Result<Vec<Interactable>> {
        floor_listed(self, here)
    }

    fn laid(&self, _found: &Worked, ground: Ground, here: &Here) -> Result<Ground> {
        floor_laid(self, ground, here)
    }
}

/// `SecretBaseFloor.listed`, of `form`'s watch: a function of its own, where Python's tests
/// subclass the landform over a watch of theirs, which lists as the game's does.
pub fn floor_listed(form: &dyn Landform, here: &Here) -> Result<Vec<Interactable>> {
    let watch = (here.watch)(form);
    let inside = inside(watch.as_deref(), here);
    let mut out = tiles(here, inside)?;
    if let Some(inside) = inside {
        out.extend(placed(inside)?);
    }
    Ok(out)
}

/// `SecretBaseFloor.laid`, of `form`'s watch, as `floor_listed` is.
pub fn floor_laid(form: &dyn Landform, ground: Ground, here: &Here) -> Result<Ground> {
    let watch = (here.watch)(form);
    let Some(inside) = inside(watch.as_deref(), here) else {
        return Ok(ground);
    };
    let covered = covered(here, inside)?;
    let mut ground = ground;
    if !covered.is_empty() {
        ground = Ground {
            chunks: ground
                .chunks
                .iter()
                .map(|chunk| chunk.clone().opened(&covered))
                .collect(),
            ..ground
        };
    }
    let mut solid: IndexMap<Cell, Option<f64>> = IndexMap::new();
    for cell in inside.solid()? {
        if !covered.contains(&cell) {
            solid.insert(cell, None);
        }
    }
    Ok(ground.stood(&solid, here.r#where.cell))
}

/// The base's watch, in a zone a base is built in, or None.
fn inside<'a>(watch: Option<&'a dyn Tracked>, here: &Here) -> Option<&'a Decorations> {
    watch
        .and_then(as_watch::<Decorations>)
        .filter(|inside| inside.zones.contains(&here.r#where.zone))
}

/// The base's crumbled cliff and holes, each patch of touching cells of a kind in `BASE_TILES`
/// the chunks loaded now hold as one thing over its bounding box: `landform.runs` would split the
/// cliff's 3 cells by 2 into two rows and list it twice. A cell of the map, so only the chunks
/// loaded now say where it is, in the player's zone or none. Nothing once a decoration stands on
/// any of its cells: the Stairs set down over the cliff left the cell table at kind `CLIFF_TILE`
/// and the base's mapper with a decoration in each of the six (omega_ruby_secret-base4.ctm).
fn tiles(here: &Here, inside: Option<&Decorations>) -> Result<Vec<Interactable>> {
    let zone = here.r#where.zone;
    let mut found: IndexMap<i64, HashSet<Cell>> = IndexMap::new();
    for chunk in (here.chunks)()? {
        for (cell, kind) in chunk.kinds() {
            if base_tile(kind).is_some() {
                let at = (here.zone_at)(cell)?;
                if at == Some(zone) || at.is_none() {
                    found.entry(kind).or_default().insert(cell);
                }
            }
        }
    }
    let covered = match inside {
        None => HashSet::new(),
        Some(inside) => covered(here, inside)?,
    };
    let mut out: Vec<Interactable> = Vec::new();
    for (kind, cells) in found {
        for group in touching(&cells) {
            if group.iter().any(|cell| covered.contains(cell)) {
                continue;
            }
            let (first, size) = spread(&group);
            out.push(Interactable {
                size,
                code: kind,
                source: place_source(TILE_SOURCE, cell_number(first))?,
                ..Interactable::new(
                    "exits",
                    base_tile(kind).expect("a base tile's kind"),
                    first,
                )
            });
        }
    }
    Ok(out)
}

/// The cells of the base's crumbled cliff and holes a decoration stands on, walked over: the
/// player climbed the Stairs set down over the cliff to the upper floor, the cell table keeping
/// all six cells blocked under them, and P said "can't continue" (omega_ruby_secret-base4.ctm).
/// The Solid Board over a hole is taken to be crossed the same way, by its description; none has
/// been set down yet.
fn covered(here: &Here, inside: &Decorations) -> Result<HashSet<Cell>> {
    let zone = here.r#where.zone;
    let mut out = HashSet::new();
    for chunk in (here.chunks)()? {
        for (cell, kind) in chunk.kinds() {
            if base_tile(kind).is_none() {
                continue;
            }
            let at = (here.zone_at)(cell)?;
            if (at == Some(zone) || at.is_none()) && !inside.standing(cell)?.is_empty() {
                out.insert(cell);
            }
        }
    }
    Ok(out)
}

/// The decorations set down in the base, among the signs by the game's names for them, each one
/// thing over its cells: set down, the Stairs and the desks were not listed at all. The Invisible
/// Doll is not, as the game hides it.
///
/// Those of one name that touch are one thing: every base's five Welcome Mats in a line were
/// listed five times. Apart, they stay apart, so the way to one does not end on the floor between
/// them.
fn placed(inside: &Decorations) -> Result<Vec<Interactable>> {
    let mut named: IndexMap<String, HashSet<Cell>> = IndexMap::new();
    for (number, cells) in inside.placed()? {
        if let Some(name) = inside.named(number) {
            named.entry(name).or_default().extend(cells);
        }
    }
    let mut out: Vec<Interactable> = Vec::new();
    for (name, cells) in named {
        for group in touching(&cells) {
            let (first, size) = spread(&group);
            out.push(Interactable {
                size,
                source: place_source(DECORATION_SOURCE, cell_number(first))?,
                ..Interactable::new("signs", &name, first)
            });
        }
    }
    Ok(out)
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
