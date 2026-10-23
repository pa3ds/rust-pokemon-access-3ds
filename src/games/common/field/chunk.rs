//! A chunk's cell table as every game's ground is laid out in, its floors, and what each kind
//! of cell is.

use crate::core::pathfind::{EAST, NORTH, SOUTH, WEST};
use crate::py::{self, Error, Result};

/// Cells along a chunk's side.
pub const CHUNK: i64 = 40;

/// The bit of a cell's word that is set where the player cannot walk.
pub const BLOCKED: i64 = 0x1;

/// The bit of a cell's word that is set on water, where the blocked bit is clear and only a player
/// surfing goes. Route 109's sea is `0x3d1a0006`, and the player could not walk onto it
/// (omega_ruby_surf.ctm, spike 136). Across every chunk in both ROMs the bit is set on every word
/// of nine kinds and no other: the sea `0x3d`, the deep water Dive goes down through (`DIVE`),
/// `0x42` between the sea and walls, the four `0x94` to `0x97` (`CURRENTS`), and `0x3f` and
/// `0x40`, which are blocked as well (`FALLS`, `SHORE`). X has `0x3d`, `0x3f` and `0x40`.
pub const WATER: i64 = 0x2;

/// The kind of the strip of shore between land and the sea, one cell wide: water and blocked.
/// Surf jumps the player over it, never onto it, getting on the water and off: on Route 122 from
/// (22, 8) to (22, 10) over (22, 9), and from (54, 58) to (54, 56) over (54, 57), and P walled the
/// water off at both (omega_ruby_surf3.ctm, spike 162). Of its 1,528 cells in ORAS's chunks,
/// nearly all have land on one side and the sea on the other.
pub const SHORE: i64 = 0x40;

/// The kind of a waterfall, the other blocked water: a row of it one cell deep with water on both
/// sides, the water above it higher. Waterfall carries the player up over it, and they go back
/// down over it with no question asked: at the foot of Ever Grande City's, from (51, 52) to (51,
/// 50) over (51, 51), the sea below at height 0 and the water above at 234
/// (omega_ruby_waterfall.ctm, spike 170). All 72 cells of it in ORAS's chunks, in 8 files, are
/// `0x3f180003`, and wherever water lies on both sides the higher is north.
pub const FALLS: i64 = 0x3F;

/// The kind of the deep water a player surfing dives through, drawn darker than the sea round it.
/// The executable tests the cell under the player for it alone while they are on the water
/// (`0x003d3454`). Route 128 has three patches of it in its own zone, 3-by-3 squares at (307, 175)
/// and (339, 175) and a band of 296 cells running diagonally through a 37-by-36 box from (296,
/// 160), and the player sat on its edge at (314, 166) (omega_ruby_route-128-dive.ctm, spike 164).
/// Across ORAS's chunks, 45 of its patches are 3-by-3 squares and the rest are bands and blobs.
pub const DIVE: i64 = 0x41;

/// The kind of a hole in the floor a big boulder Strength pushes drops into, blocked: a square
/// of two cells by two, a boulder's size. Every cell of it in ORAS's chunks is `0x30020021`, in
/// nine zones and in none without a big boulder: Fiery Path, Seafloor Cavern's rooms and three
/// others. A boulder in one stays a move model, on the hole's four cells, the cells still blocked
/// in the chunk as loaded, and the player walks over it: Seafloor Cavern's first room with
/// boulders, zone `0x64`, has both of its in their holes at (10, 19) and (21, 22) (movie36,
/// spike 202).
pub const HOLE: i64 = 0x30;

/// What going onto water costs a path on top of the step, five steps' worth: the player faces the
/// water, presses A, and answers the game's question before the step is taken. So a way surfing
/// is taken over one on foot only when it saves more than that. Cutting a bush down or smashing a
/// rock is the same face, A and answer, and costs the same.
pub const BOARD: i64 = 10;

/// Where a cell's kind sits in its word: the top byte.
pub const KIND_SHIFT: i64 = 24;

/// The footstep of ground the game plays no footstep on: streets, floors, cave floors (spike
/// 225).
pub const SILENT: i64 = 0;

/// Where ORAS's cell word keeps its footstep: bits 12 to 16. X's are bits 11 to 15 (spike 231).
pub const FOOTSTEP_SHIFT: i64 = 12;

/// `footstep(word)`, at ORAS's `FOOTSTEP_SHIFT`.
pub fn footstep(word: i64) -> i64 {
    footstep_at(word, FOOTSTEP_SHIFT)
}

/// Which footstep the game plays on a cell, as it reads the word (`0x00402fcc`): five bits
/// from `shift`, and `SILENT` for an all-ones word or one of kind `0xff` (spike 225).
pub fn footstep_at(word: i64, shift: i64) -> i64 {
    if word == 0xFFFFFFFF || word >> KIND_SHIFT == 0xFF {
        return SILENT;
    }
    (word >> shift) & 0x1F
}

/// The floor a cell of a chunk with one floor is on, and one with floors where which is not
/// asked: whichever is open (`Ground.floor`).
pub const ANY_FLOOR: i64 = -1;

/// How far apart two floors may be where they meet and still be walked between, a cell's worth
/// of world units. Where a deck's cell touches the ground's in ORAS's chunks the two are 0 to 14
/// apart where they join, and 54 or more where the deck stands over lower ground (spike 156).
pub const CLIMB: f64 = 18.0;

/// A ledge's kind -> the way it is jumped down (spike 126). The game's own function at
/// `0x00402e74` turns these kinds into 0, 1, 2 and 3, the engine's north, south, west and east as a
/// rail's record numbers them (spike 107). omega_ruby_ledges.ctm jumped 0x75 south three times,
/// omega_ruby_ledges_2.ctm 0x73 west and omega_ruby_ledges_3.ctm 0x72 east; ORAS has no ledge that
/// drops north, so 0x74 rests on the numbering.
/// A ledge is blocked, and the player's cell goes from before it to after it in one move.
pub const LEDGES: [(i64, &str); 4] = [(0x74, NORTH), (0x75, SOUTH), (0x73, WEST), (0x72, EAST)];

/// A jumping stone's kind: a pillar in one of X's mud pits, blocked, that the player hops onto
/// from another stone or a `LAUNCH` two cells away in a line, over the mud between, and off it the
/// same way, each hop one move. On Route 8 they hopped from (544, 745) to (546, 745), on to (552,
/// 745) and back, and north from (548, 745) to (548, 741) (x_route-8-3.ctm, spike 235). X has 70
/// stones in 9 chunk files, ORAS none. The stones touching only at a corner, as Route 8's two sets
/// do, are not hopped between here: no recording has the player do it.
pub const STONE: i64 = 0x92;

/// The kind of the open cells at a mud pit's edge that hops onto its stones start and end on,
/// walked onto and off as any floor: 27 of them in X's chunks.
pub const LAUNCH: i64 = 0x93;

/// A counter's kind: the Pokémon Center's, the Poké Mart's. The game talks across one, to the
/// thing in the cell beyond it: the field DLL's function at `0x007175a8` reads the kind of the cell
/// the player faces, and where the executable's test at `0x00403004` says it is this one, looks two
/// cells ahead instead of one. No other kind is tested there. In Petalburg's Pokémon Center the
/// Nurse stands at (19, 15) with the counter at (19, 16) and every cell beside her a counter or a
/// wall, and P said "no path to Nurse" (poke_center_pc.ctm, 2026-10-01).
pub const COUNTER: i64 = 0xD4;

/// A sandy slope's kind -> the way it goes down, in the order the ledges' kinds go. Its cells are
/// open, but on foot the game never lets the player onto one going up: in Granite Cave 1F
/// (omega_ruby_cave.ctm) the player at (10, 29) faced west onto `0x69000020` at (9, 29) and (8, 29)
/// and did not move. Only the Mach Bike rides up one, as the player did up Jagged Pass's `0x6c`
/// slopes, north from (28, 80), sliding back down the first time (omega_ruby_bike.ctm, spike 158).
/// Across ORAS's map chunks the ground's mesh falls the same way across every cell of a kind: all
/// 25 of 0x69 east, all 44 of 0x6a west, 166 of 0x6c's 188 south and the rest level along its foot
/// or top. No chunk has 0x6b, so north rests on the numbering. On foot a slope is walked down only,
/// and on the bike either way (`Ground.bike`).
pub const SLOPES: [(i64, &str); 4] = [(0x69, EAST), (0x6A, WEST), (0x6B, NORTH), (0x6C, SOUTH)];

/// A current's kind -> the way it carries a player surfing on it, in the order the ledges' kinds
/// go. On Route 132 the player surfed west onto `0x95` at (415, 105) and was carried a cell at a
/// time to (387, 105), where the calm sea `0x3d` begins (movie43, spike 208). The matrix Routes 132
/// to 134 are on has 10,184 cells of `0x95`, 541 of `0x96` in diagonal bands, 363 of `0x97`, and
/// no `0x94`, so east rests on the numbering.
pub const CURRENTS: [(i64, &str); 4] = [(0x94, EAST), (0x95, WEST), (0x96, NORTH), (0x97, SOUTH)];

/// A bumpy slope's kind, the rocky step only the Acro Bike's bunny hop gets up: walled to the
/// player, and in Jagged Pass each one stands in a row of ledges dropping south, where a ledge
/// would be, five of them up a column at x 50 from z 86 to 73, or alone across a narrow way, as at
/// x 25. Only in 13 of the ROM's chunk files, the Safari Zone's among them, which has bumpy slopes
/// too (spike 159, 2026-10-03). Read off the map, not seen hopped.
pub const BUMPY_SLOPES: [i64; 1] = [0x6F];

/// The Acro Bike's rails, by their kinds, not Lumiose's (`gen6.field.rail`): walled to the player, and
/// crossed only on the Acro Bike, hopping from rail to rail. Both are only in Route 119's map
/// files, the Safari Zone's, which has rails too, and one other, and on Route 119 each runs in a
/// line from open ground to open ground: 0x6e at (107, 219) to (114, 219), (114, 222) and (121,
/// 222), beside the path north up the route, and 0x6d up and down the lines of 0x6e in the route's
/// top (spike 159). Read off the map, not seen ridden.
pub const BIKE_RAILS: [i64; 2] = [0x6D, 0x6E];

/// `table.get(kind)`, for a table in the order its Python dict was written in.
fn way_of(table: &[(i64, &'static str)], kind: i64) -> Option<&'static str> {
    table
        .iter()
        .find(|(known, _)| *known == kind)
        .map(|&(_, way)| way)
}

/// One floor of a chunk that has more than one: its own cell table, and the height of the
/// floor under each cell, None where none is.
#[derive(Debug, Clone, PartialEq)]
pub struct Floor {
    pub words: Vec<i64>,
    pub heights: Vec<Option<f64>>,
}

/// One chunk's cell table, at the cells it covers.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Chunk {
    /// The north-west cell.
    pub origin: (i64, i64),
    /// One a cell, `CHUNK` to a row.
    pub words: Vec<i64>,
    /// Every floor of a chunk with a deck over its ground, the ground first, or none: Mt.
    /// Chimney's catwalk over the crater is a floor of its own, open where the ground's table has
    /// the crater as a wall (spike 156). `words` is always the ground's.
    pub floors: Vec<Floor>,
    /// For a chunk flattened onto one floor a cell at a height, the height of each cell's floor,
    /// None where none is open (`mapblock.flat`); none otherwise.
    pub heights: Vec<Option<f64>>,
}

impl Chunk {
    /// A chunk with no floors and no heights, which `Chunk(origin, words)` is; the rest is set
    /// with `..Chunk::new(...)`.
    pub fn new(origin: (i64, i64), words: Vec<i64>) -> Self {
        Chunk {
            origin,
            words,
            floors: Vec::new(),
            heights: Vec::new(),
        }
    }

    /// Which chunk of the matrix this is, as (column, row).
    pub fn key(&self) -> (i64, i64) {
        (
            py::floordiv(self.origin.0, CHUNK),
            py::floordiv(self.origin.1, CHUNK),
        )
    }

    pub fn holds(&self, cell: (i64, i64)) -> bool {
        0 <= cell.0 - self.origin.0
            && cell.0 - self.origin.0 < CHUNK
            && 0 <= cell.1 - self.origin.1
            && cell.1 - self.origin.1 < CHUNK
    }

    /// `self.words[z * CHUNK + x]` for the cell, which counts from the end for a negative index
    /// as Python's tuples do, and raises past either end. Raised for the caller, as the Python
    /// raised it in each of the readers below.
    #[track_caller]
    pub fn word(&self, cell: (i64, i64)) -> Result<i64> {
        let (x, z) = (cell.0 - self.origin.0, cell.1 - self.origin.1);
        let len = self.words.len() as i64;
        let at = z * CHUNK + x;
        let at = if at < 0 { at + len } else { at };
        match usize::try_from(at).ok().and_then(|at| self.words.get(at)) {
            Some(&word) => Ok(word),
            None => Err(Error::index("tuple index out of range")),
        }
    }

    pub fn walkable(&self, cell: (i64, i64)) -> Result<bool> {
        Ok(self.word(cell)? & BLOCKED == 0)
    }

    pub fn water(&self, cell: (i64, i64)) -> Result<bool> {
        Ok(self.word(cell)? & WATER != 0)
    }

    /// Which way the cell goes down, a ledge jumped or a slope walked, or None where it is
    /// neither.
    pub fn ledge(&self, cell: (i64, i64)) -> Result<Option<&'static str>> {
        match way_of(&LEDGES, self.kind(cell)?) {
            Some(way) => Ok(Some(way)),
            None => self.slope(cell),
        }
    }

    /// Which way the cell goes down where it is a slope, or None.
    pub fn slope(&self, cell: (i64, i64)) -> Result<Option<&'static str>> {
        Ok(way_of(&SLOPES, self.kind(cell)?))
    }

    /// Which way the cell carries a player surfing on it where it is a current, or None.
    pub fn current(&self, cell: (i64, i64)) -> Result<Option<&'static str>> {
        Ok(way_of(&CURRENTS, self.kind(cell)?))
    }

    pub fn kind(&self, cell: (i64, i64)) -> Result<i64> {
        Ok(self.word(cell)? >> KIND_SHIFT)
    }

    /// Every cell of the chunk with its kind.
    pub fn kinds(&self) -> impl Iterator<Item = ((i64, i64), i64)> + '_ {
        self.words.iter().enumerate().map(|(i, &word)| {
            let i = i as i64;
            (
                (self.origin.0 + i % CHUNK, self.origin.1 + i / CHUNK),
                word >> KIND_SHIFT,
            )
        })
    }
}
