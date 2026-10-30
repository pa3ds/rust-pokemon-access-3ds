//! Landforms: each kind of special ground in P's way, one thing on `Game` (GLOSSARY: Landform).
//!
//! A landform says what it works out once a map from plain data, the ROM's chunks and the zone it
//! is in (`found`), what of it is listed (`listed`), and what it gives the ground the search walks:
//! its traversal or its rules (`laid`). Gen 6's own, which every game's chunks can hold, are
//! `GEN6`; a game's `Game.landforms` starts from them and adds its own: X's ridden Pokémon. The
//! field reader works each out once a map and keeps it (`FieldReader._found`), and never names
//! one.
//!
//! Bound by:
//! - ADR-0006: a landform names no game's kinds; a game's own data is given to it on its `Game`.

use std::any::Any;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::u16;
use crate::core::pathfind::{self, Cell};
use crate::games::common::field::chunk::HOLE as BOULDER_HOLE;
use crate::games::common::field::chunk::{
    BIKE_RAILS, BUMPY_SLOPES, CHUNK, CURRENTS, Chunk, DIVE, FALLS, SLOPES,
};
use crate::games::common::field::climb::Walls;
use crate::games::common::field::grind::GrindRails;
use crate::games::common::field::ground::{Ground, STRENGTH};
use crate::games::common::field::things::{
    ARROW_SOURCE, ARROWS, BIKE, BIKE_SOURCE, CLIMB_SOURCE, CLIMBING, CURRENT_SOURCE,
    CURRENTS_CATEGORY, DIVE_SOURCE, Field, Interactable, OBSTACLES, RIDE_SOURCE, RIDING,
    TILE_SOURCE, place_source,
};
use crate::games::gen6::field::terrain::Terrain;
use crate::games::gen6::field::zonetable::{
    INFO_KIND, INFO_MATRIX, INFO_NAME, INFO_PARENT, INFO_SIZE, NAME_MASK, UNDERWATER, ZONE_LIMIT,
    ZoneTable,
};
use crate::py::{self, Error, Result};

pub const SLOPE: &str = "slope";
pub const BUMPY_SLOPE: &str = "bumpy slope";
/// What a sandy slope only the Mach Bike rides up, a rocky step only the Acro Bike hops up, and an
/// end of a rail only the Acro Bike crosses, are called: the game names none of them, and its words
/// for the bikes are their items' (spike 159).
pub const BIKE_RAIL: &str = "rail";

/// What a hole a big boulder drops into is called, listed under obstacles with the boulders until
/// one fills it (Orion, 2026-10-11; spike 202): its cells are of a kind of their own
/// (`chunk.HOLE`). A hole in a Secret Base's floor is one too: the game names it only in the Solid
/// Board's description, "You can place this over holes to cross over them", never where it is.
pub const HOLE: &str = "hole";

/// What a patch of the deep water Dive goes down through is called: the game draws it darker than
/// the sea and asks about diving there, but never names it (spike 164).
pub const DIVE_SPOT: &str = "dive spot";

/// What the sea's currents that let the player go round one place are called, listed at the cell
/// of them nearest the player and said with where it lets them go: the game never names one.
pub const CURRENT: &str = "current";

/// What a patch of the flat ground a ridden Pokémon is got on and off on is called, listed at its
/// cell nearest the player: the game never names it (spike 238).
pub const FLAT_GROUND: &str = "flat ground";

/// What an arrow tile is called, with the way it pushes the player, "arrow east": the game draws
/// it and never names it (spike 265).
pub const ARROW: &str = "arrow";

/// What a run of the blue spots a climbing wall is climbed from is called, and one on a ledge at
/// its side, where it is got off: the Gym's guide says "start from one of those blue spots", and the
/// game names neither (spike 245).
pub const BLUE_SPOT: &str = "blue spot";
pub const WALL_LEDGE: &str = "wall ledge";

/// Cells apart the places two currents let the player go may be and the two still be listed as
/// one current. Route 133's currents let the player go at 152 cells, 71 groups of them side by side,
/// 28 within 5 cells of each other; Routes 132 and 134 have 20 and 16, Seafloor Cavern's two rooms 4
/// each (spike 208).
pub const LANDING_REACH: i64 = 5;

/// What a landform worked out (`Landform.found`), given back to `listed` and `laid`: Python's
/// `Any`, each landform downcasting its own.
pub type Found = Rc<dyn Any>;

/// The map a landform is worked out on: the zone the player is in, and every chunk of its
/// matrix, the ROM's, or the loaded ones alone without the ROM.
pub struct Matrix<'a> {
    pub zone: i64,
    /// Each chunk by its key, (column, row).
    pub chunks: IndexMap<(i64, i64), Rc<Chunk>>,
    /// The zone the field map puts a cell in, None where it is not read.
    pub zone_at: &'a dyn Fn(Cell) -> Result<Option<i64>>,
    pub terrain: &'a Terrain,
    pub zones: &'a ZoneTable,
}

impl Matrix<'_> {
    /// Each cell of the zone, or of no zone the field map says, whose kind `kinds` takes, with
    /// its kind.
    pub fn cells(&self, kinds: &dyn Fn(i64) -> bool) -> Result<IndexMap<Cell, i64>> {
        let mut out: IndexMap<Cell, i64> = IndexMap::new();
        for chunk in self.chunks.values() {
            for (cell, kind) in chunk.kinds() {
                if kinds(kind) && {
                    let at = (self.zone_at)(cell)?;
                    at == Some(self.zone) || at.is_none()
                } {
                    out.insert(cell, kind);
                }
            }
        }
        Ok(out)
    }

    /// Whether the cell is blocked, or no chunk holds it.
    pub fn shut(&self, cell: Cell) -> Result<bool> {
        match self
            .chunks
            .get(&(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK)))
        {
            None => Ok(true),
            Some(chunk) => Ok(!chunk.walkable(cell)?),
        }
    }
}

/// What a landform is listed and laid on: where the player stands now, and who stands where.
///
/// `terrain` and `zones` are the port's: Python's `DiveSpots` keeps a cache over the matrix it was
/// found on, and works out the sea above from it as the ground is first asked for; here it keeps
/// the cache alone, and is handed the matrix's terrain and zone table then.
pub struct Here<'a> {
    pub r#where: &'a Field,
    /// The zone's move models, as the field reader lists them, whether the listing has them or
    /// not: guiding asks for the ground with nothing listed. Read only where a landform asks, as a
    /// zone with holes does.
    pub people: &'a dyn Fn() -> Result<Vec<Interactable>>,
    /// Sprite -> the move that clears the thing drawn as it (`Game.clears`).
    pub clears: &'a IndexMap<i64, i64>,
    pub terrain: &'a Terrain,
    pub zones: &'a ZoneTable,
}

/// One kind of special ground. `found` is worked out once a map and must read no memory: a tape
/// replay asks what it asked before, at the same moments, and a landform reading the game's memory
/// does so in `listed` or `laid`, where the ground or the listing is asked for.
///
/// `Send` and `Sync` because a `Game` lives in a `static`.
pub trait Landform: Any + Debug + Send + Sync {
    /// What it works out of `matrix`, given back to `listed` and `laid`.
    fn found(&self, _matrix: &Matrix) -> Result<Found> {
        Ok(Rc::new(()))
    }

    /// What of it is listed, in the zone the player stands in.
    fn listed(&self, _found: &Found, _here: &Here) -> Result<Vec<Interactable>> {
        Ok(Vec::new())
    }

    /// `ground` with what it gives the search.
    fn laid(&self, _found: &Found, ground: Ground, _here: &Here) -> Result<Ground> {
        Ok(ground)
    }
}

/// `found`, as the landform that worked it out made it.
fn found_as<T: 'static>(found: &Found) -> &T {
    found
        .downcast_ref::<T>()
        .expect("a landform is handed back what it found")
}

/// The landform as `T`, where it is one: Python's `isinstance`.
pub fn as_form<T: 'static>(form: &Arc<dyn Landform>) -> Option<&T> {
    (form.as_ref() as &dyn Any).downcast_ref::<T>()
}

/// The cell nearest the player, the first of those as near.
pub fn nearest(cells: impl IntoIterator<Item = Cell>, player: Cell) -> Cell {
    cells
        .into_iter()
        .min_by_key(|cell| ((cell.0 - player.0).abs() + (cell.1 - player.1).abs(), *cell))
        .expect("a patch has a cell")
}

/// A cell as a place's number: `x << 12 | z`, as a use tile's is.
pub fn cell_number(cell: Cell) -> i64 {
    cell.0 << 12 | cell.1
}

/// The north-west corner of the box round the cells, and its size.
pub fn spread(cells: &[Cell]) -> (Cell, (i64, i64)) {
    let xs = cells.iter().map(|cell| cell.0);
    let zs = cells.iter().map(|cell| cell.1);
    let (west, east) = (xs.clone().min().unwrap_or(0), xs.max().unwrap_or(0));
    let (north, south) = (zs.clone().min().unwrap_or(0), zs.max().unwrap_or(0));
    ((west, north), (east - west + 1, south - north + 1))
}

/// The four cells a step from this one.
pub fn beside(cell: Cell) -> [Cell; 4] {
    pathfind::STEPS.map(|(_, (dx, dz))| (cell.0 + dx, cell.1 + dz))
}

/// Each patch of cells side by side of one kind, the rails' kinds counting as one, its first cell
/// first.
pub fn patches(found: &IndexMap<Cell, i64>) -> Vec<Vec<Cell>> {
    let mut out: Vec<Vec<Cell>> = Vec::new();
    let mut left: BTreeSet<Cell> = found.keys().copied().collect();
    while let Some(&first) = left.iter().next() {
        left.remove(&first);
        let railed = BIKE_RAILS.contains(&found[&first]);
        let mut patch = vec![first];
        let mut todo = vec![first];
        while let Some(cell) = todo.pop() {
            for near in beside(cell) {
                if left.contains(&near)
                    && if railed {
                        BIKE_RAILS.contains(&found[&near])
                    } else {
                        found[&near] == found[&first]
                    }
                {
                    left.remove(&near);
                    patch.push(near);
                    todo.push(near);
                }
            }
        }
        out.push(patch);
    }
    out
}

/// `in SLOPES`, the kinds of sandy slope.
fn in_slopes(kind: i64) -> bool {
    SLOPES.iter().any(|&(known, _)| known == kind)
}

/// The ground the bikes cross and walking does not: the Mach Bike's slopes, each patch of one kind
/// a thing; the Acro Bike's rails, each end where the player gets on one; and its bumpy slopes,
/// each hopped up from the cell before it to the cell after, as a ledge is jumped down
/// (`Ground.hops`, spike 159). Listed under bike.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Bike;

/// What `Bike` finds: its things, and the bumpy slopes' ways up.
type BikeFound = (Vec<Interactable>, IndexMap<Cell, &'static str>);

impl Landform for Bike {
    fn found(&self, matrix: &Matrix) -> Result<Found> {
        let kinds = matrix.cells(&|kind| {
            in_slopes(kind) || BIKE_RAILS.contains(&kind) || BUMPY_SLOPES.contains(&kind)
        })?;
        let mut out: Vec<Interactable> = Vec::new();
        for patch in patches(&kinds) {
            if BUMPY_SLOPES.contains(&kinds[&patch[0]]) {
                for &cell in &patch {
                    out.push(Interactable {
                        code: kinds[&cell],
                        source: place_source(BIKE_SOURCE, cell_number(cell))?,
                        home: Some(matrix.zone),
                        ..Interactable::new(BIKE, BUMPY_SLOPE, cell)
                    });
                }
            } else if in_slopes(kinds[&patch[0]]) {
                let (first, size) = spread(&patch);
                // The box's corner is not one of an L-shaped patch's cells: its kind is the patch's.
                out.push(Interactable {
                    size,
                    code: kinds[&patch[0]],
                    source: place_source(BIKE_SOURCE, cell_number(first))?,
                    home: Some(matrix.zone),
                    ..Interactable::new(BIKE, SLOPE, first)
                });
            } else {
                let held: HashSet<Cell> = patch.iter().copied().collect();
                for &cell in &patch {
                    if beside(cell)
                        .iter()
                        .filter(|near| held.contains(*near))
                        .count()
                        <= 1
                    {
                        out.push(Interactable {
                            code: kinds[&cell],
                            source: place_source(BIKE_SOURCE, cell_number(cell))?,
                            home: Some(matrix.zone),
                            ..Interactable::new(BIKE, BIKE_RAIL, cell)
                        });
                    }
                }
            }
        }
        let found: BikeFound = (out, matrix.terrain.rises(&BUMPY_SLOPES)?);
        Ok(Rc::new(found))
    }

    fn listed(&self, found: &Found, _here: &Here) -> Result<Vec<Interactable>> {
        Ok(found_as::<BikeFound>(found).0.clone())
    }

    fn laid(&self, found: &Found, ground: Ground, _here: &Here) -> Result<Ground> {
        Ok(Ground {
            hops: found_as::<BikeFound>(found).1.clone(),
            ..ground
        })
    }
}

/// The holes a big boulder Strength pushes drops into, each listed under obstacles; one lying
/// wholly in a hole is floor the player walks over (`Ground.filled`, spike 202).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Holes;

impl Landform for Holes {
    fn found(&self, matrix: &Matrix) -> Result<Found> {
        let mut out: Vec<Interactable> = Vec::new();
        for patch in patches(&matrix.cells(&|kind| kind == BOULDER_HOLE)?) {
            let (first, size) = spread(&patch);
            out.push(Interactable {
                size,
                code: BOULDER_HOLE,
                source: place_source(TILE_SOURCE, cell_number(first))?,
                home: Some(matrix.zone),
                ..Interactable::new(OBSTACLES, HOLE, first)
            });
        }
        Ok(Rc::new(out))
    }

    fn listed(&self, found: &Found, _here: &Here) -> Result<Vec<Interactable>> {
        Ok(found_as::<Vec<Interactable>>(found).clone())
    }

    fn laid(&self, found: &Found, ground: Ground, here: &Here) -> Result<Ground> {
        let cells = holes(found_as::<Vec<Interactable>>(found));
        if cells.is_empty() {
            return Ok(ground);
        }
        Ok(Ground {
            filled: dropped(&cells, &(here.people)()?, here.clears),
            ..ground
        })
    }
}

/// The cells of the holes `Holes` found.
pub fn holes(found: &[Interactable]) -> HashSet<Cell> {
    found.iter().flat_map(|item| item.cells()).collect()
}

/// The cells of the holes a reader's `Holes` found, `found` being what it handed back.
pub fn holes_found(found: &Found) -> HashSet<Cell> {
    holes(found_as::<Vec<Interactable>>(found))
}

/// The cells of every big boulder among `items` lying wholly in `holes`: pushed into one, it is
/// floor the player walks over, its model still there (spike 202).
pub fn dropped(
    holes: &HashSet<Cell>,
    items: &[Interactable],
    clears: &IndexMap<i64, i64>,
) -> HashSet<Cell> {
    items
        .iter()
        .filter(|item| {
            item.move_model
                && clears.get(&item.code) == Some(&STRENGTH)
                && item.cells().iter().all(|cell| holes.contains(cell))
        })
        .flat_map(|item| item.cells())
        .collect()
}

/// Each patch of the deep water Dive goes down through, listed under exits at its cell nearest
/// the player, so the way there ends on deep water: Route 128's biggest is a band running
/// diagonally through a box of 37 by 36, mostly plain sea (spike 164). Under the sea, each cell of
/// deep water with every one the sea above joins it to (`Ground.surfaces`, spike 165).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DiveSpots;

/// What `DiveSpots` finds: each patch, and the zone it was found in with the sea above, worked
/// out once as the ground is first asked for.
struct DiveFound {
    dives: Vec<Vec<Cell>>,
    zone: i64,
    surfaces: RefCell<Option<IndexMap<Cell, Rc<HashSet<Cell>>>>>,
}

impl Landform for DiveSpots {
    fn found(&self, matrix: &Matrix) -> Result<Found> {
        let dives = patches(&matrix.cells(&|kind| kind == DIVE)?);
        // The zone table is the game's memory, read once a zone and only as the ground is asked
        // for, as before landforms were.
        Ok(Rc::new(DiveFound {
            dives,
            zone: matrix.zone,
            surfaces: RefCell::new(None),
        }))
    }

    fn listed(&self, found: &Found, here: &Here) -> Result<Vec<Interactable>> {
        let mut out: Vec<Interactable> = Vec::new();
        for patch in &found_as::<DiveFound>(found).dives {
            out.push(Interactable {
                code: DIVE,
                source: place_source(DIVE_SOURCE, cell_number(patch[0]))?,
                home: Some(here.r#where.zone),
                ..Interactable::new(
                    "exits",
                    DIVE_SPOT,
                    nearest(patch.iter().copied(), here.r#where.cell),
                )
            });
        }
        Ok(out)
    }

    fn laid(&self, found: &Found, ground: Ground, here: &Here) -> Result<Ground> {
        let found = found_as::<DiveFound>(found);
        // `functools.cache`: an answer kept once made, a raise not.
        if found.surfaces.borrow().is_none() {
            let made = surfaces(here.terrain, here.zones, found.zone, &found.dives)?;
            *found.surfaces.borrow_mut() = Some(made);
        }
        let surfaces = found.surfaces.borrow().clone().unwrap_or_default();
        Ok(Ground { surfaces, ..ground })
    }
}

/// Under the water, each cell of deep water in the zone with every one the sea above joins it to;
/// none above the water, or where the sea above is not found. The sea above is the zone of the
/// same place that is its own parent and is not under the water (`Terrain.surfaces`).
fn surfaces(
    terrain: &Terrain,
    zones: &ZoneTable,
    zone: i64,
    dives: &[Vec<Cell>],
) -> Result<IndexMap<Cell, Rc<HashSet<Cell>>>> {
    let (row, rows) = (zones.row(zone)?, zones.rows()?);
    let (Some(row), Some(rows)) = (row, rows) else {
        return Ok(IndexMap::new());
    };
    if u16(&row, INFO_KIND)? != UNDERWATER || !terrain.known() {
        return Ok(IndexMap::new());
    }
    let mut below: IndexSet<Cell> = IndexSet::new();
    for patch in dives {
        below.extend(patch.iter().copied());
    }
    let mut seas: Vec<i64> = Vec::new();
    for other in 0..ZONE_LIMIT {
        let info = py::slice(&rows, other * INFO_SIZE, (other + 1) * INFO_SIZE);
        if !info.is_empty()
            && other != zone
            && u16(info, INFO_PARENT)? == other
            && u16(info, INFO_KIND)? != UNDERWATER
            && u16(info, INFO_NAME)? & NAME_MASK == u16(&row, INFO_NAME)? & NAME_MASK
        {
            seas.push(u16(info, INFO_MATRIX)?);
        }
    }
    terrain.surfaces(&below, &seas)
}

/// The sea's currents, those that let the player go round one place a thing, listed under
/// currents at its cell nearest the player and said with where it lets them go there (spike
/// 208). The search reads a current off the chunks itself (`Ground.carry`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Currents;

impl Landform for Currents {
    fn found(&self, matrix: &Matrix) -> Result<Found> {
        let mut flows: IndexMap<Cell, &'static str> = IndexMap::new();
        for chunk in matrix.chunks.values() {
            for (cell, kind) in chunk.kinds() {
                if let Some(&(_, way)) = CURRENTS.iter().find(|(known, _)| *known == kind) {
                    flows.insert(cell, way);
                }
            }
        }
        let landings = landings(&flows, &|cell| matrix.shut(cell))?;
        let mut mine: Vec<Cell> = Vec::new();
        for &cell in flows.keys() {
            let at = (matrix.zone_at)(cell)?;
            if at == Some(matrix.zone) || at.is_none() {
                mine.push(cell);
            }
        }
        let gathered = gathered(
            &mine.iter().map(|cell| landings[cell]).collect(),
            LANDING_REACH,
        );
        let mut which: std::collections::HashMap<Cell, usize> = std::collections::HashMap::new();
        for (i, group) in gathered.iter().enumerate() {
            for &landing in group {
                which.insert(landing, i);
            }
        }
        let mut currents: Vec<IndexMap<Cell, Cell>> = vec![IndexMap::new(); gathered.len()];
        for &cell in &mine {
            currents[which[&landings[&cell]]].insert(cell, landings[&cell]);
        }
        Ok(Rc::new(currents))
    }

    fn listed(&self, found: &Found, here: &Here) -> Result<Vec<Interactable>> {
        let mut out: Vec<Interactable> = Vec::new();
        for current in found_as::<Vec<IndexMap<Cell, Cell>>>(found) {
            let at = nearest(current.keys().copied(), here.r#where.cell);
            let first = *current.keys().min().expect("a current has a cell");
            out.push(Interactable {
                source: place_source(CURRENT_SOURCE, cell_number(first))?,
                home: Some(here.r#where.zone),
                arrival: Some(current[&at]),
                ..Interactable::new(CURRENTS_CATEGORY, CURRENT, at)
            });
        }
        Ok(out)
    }
}

/// Where a current lets the player go from each of its cells, as `Ground.drift` carries them
/// with nobody in the way: the first cell it carries them to that is no current, or the last
/// before the way is shut. Each cell's is the next's, so each is followed once.
fn landings(
    flows: &IndexMap<Cell, &'static str>,
    shut: &dyn Fn(Cell) -> Result<bool>,
) -> Result<IndexMap<Cell, Cell>> {
    let mut out: IndexMap<Cell, Cell> = IndexMap::new();
    for &start in flows.keys() {
        let mut cell = start;
        let mut carried: Vec<Cell> = Vec::new();
        while let Some(&way) = flows.get(&cell)
            && !out.contains_key(&cell)
        {
            carried.push(cell);
            let (dx, dz) = pathfind::STEPS
                .iter()
                .find(|(known, _)| *known == way)
                .map(|&(_, step)| step)
                .ok_or_else(|| Error::key(py::repr(way)))?;
            let ahead = (cell.0 + dx, cell.1 + dz);
            if shut(ahead)? || carried.contains(&ahead) {
                break;
            }
            cell = ahead;
        }
        let landing = out.get(&cell).copied().unwrap_or(cell);
        for one in carried {
            out.insert(one, landing);
        }
    }
    Ok(out)
}

/// `cells` in groups, each cell within `reach` cells either way of another of its group.
fn gathered(cells: &HashSet<Cell>, reach: i64) -> Vec<Vec<Cell>> {
    let mut left: BTreeSet<Cell> = cells.iter().copied().collect();
    let mut out: Vec<Vec<Cell>> = Vec::new();
    while let Some(&first) = left.iter().next() {
        left.remove(&first);
        let mut group = vec![first];
        let mut at = 0;
        while at < group.len() {
            let (x, z) = group[at];
            at += 1;
            let near: Vec<Cell> = left
                .iter()
                .copied()
                .filter(|cell| (cell.0 - x).abs() <= reach && (cell.1 - z).abs() <= reach)
                .collect();
            for cell in near {
                left.remove(&cell);
                group.push(cell);
            }
        }
        out.push(group);
    }
    out
}

/// The waterfalls, each cell with the way it is climbed, between the water below and above
/// (`Ground.falls`, spike 170). Not listed.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Waterfalls;

impl Landform for Waterfalls {
    fn found(&self, matrix: &Matrix) -> Result<Found> {
        Ok(Rc::new(matrix.terrain.rises(&[FALLS])?))
    }

    fn laid(&self, found: &Found, ground: Ground, _here: &Here) -> Result<Ground> {
        Ok(Ground {
            falls: found_as::<IndexMap<Cell, &'static str>>(found).clone(),
            ..ground
        })
    }
}

/// The ground only a Pokémon the player rides crosses, as X's Rhyhorn crosses Route 9's spiky
/// ground and its Mamoswine Route 17's deep snow: a wall on foot, got onto from the flat ground
/// beside it, where the Pokémon waits to be ridden and is got off (`Ground.mounted`, spikes 238
/// and 263). Each patch of the flat ground is listed under riding, at its cell nearest the
/// player, in a zone with ground to ride over. Each field holds every route's, as no zone has
/// two.
///
/// Its `frozenset`s are `HashSet`s: they are only tested.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rides {
    /// The cell kinds only a player riding crosses.
    pub ground: HashSet<i64>,
    /// The kinds of the flat ground it is got on and off on, walked on foot and riding.
    pub stops: HashSet<i64>,
    /// The characters of the Pokémon ridden.
    pub mounts: HashSet<i64>,
    /// The sprites of the things the ridden Pokémon smashes with A, in the way on foot.
    pub smashes: HashSet<i64>,
}

impl Landform for Rides {
    fn found(&self, matrix: &Matrix) -> Result<Found> {
        let kinds =
            matrix.cells(&|kind| self.ground.contains(&kind) || self.stops.contains(&kind))?;
        let found = patches(&kinds);
        // Vaniville Town's pen round Mom's Rhyhorn is flat ground too, with nothing to ride over.
        if !found
            .iter()
            .any(|patch| self.ground.contains(&kinds[&patch[0]]))
        {
            return Ok(Rc::new(Vec::<Vec<Cell>>::new()));
        }
        let flats: Vec<Vec<Cell>> = found
            .into_iter()
            .filter(|patch| self.stops.contains(&kinds[&patch[0]]))
            .collect();
        Ok(Rc::new(flats))
    }

    fn listed(&self, found: &Found, here: &Here) -> Result<Vec<Interactable>> {
        let mut out: Vec<Interactable> = Vec::new();
        for patch in found_as::<Vec<Vec<Cell>>>(found) {
            out.push(Interactable {
                source: place_source(RIDE_SOURCE, cell_number(patch[0]))?,
                home: Some(here.r#where.zone),
                ..Interactable::new(
                    RIDING,
                    FLAT_GROUND,
                    nearest(patch.iter().copied(), here.r#where.cell),
                )
            });
        }
        Ok(out)
    }

    fn laid(&self, found: &Found, ground: Ground, here: &Here) -> Result<Ground> {
        // Where no flat ground lies, nothing is got on or off, and no search carries a ride.
        if found_as::<Vec<Vec<Cell>>>(found).is_empty() {
            return Ok(ground);
        }
        ground.mounted(
            here.r#where,
            &self.ground,
            &self.stops,
            &self.mounts,
            &self.smashes,
        )
    }
}

/// The arrow tiles that push the player, by cell kind, as X's Lysandre Labs' (`arrow.Arrows`,
/// spike 265): each listed under arrows with where its push sets the player down, and the ground
/// pushing only in a zone with one.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArrowTiles {
    /// The cell kinds of the arrow tiles, each with the way it pushes.
    pub ways: IndexMap<i64, &'static str>,
}

impl Landform for ArrowTiles {
    fn found(&self, matrix: &Matrix) -> Result<Found> {
        let mut arrows: IndexMap<Cell, i64> = IndexMap::new();
        for chunk in matrix.chunks.values() {
            for (cell, kind) in chunk.kinds() {
                if self.ways.contains_key(&kind) && chunk.walkable(cell)? {
                    arrows.insert(cell, kind);
                }
            }
        }
        let ways: IndexMap<Cell, &'static str> = arrows
            .iter()
            .map(|(&cell, kind)| (cell, self.ways[kind]))
            .collect();
        let landed = pushed(&ways, &|cell| matrix.shut(cell))?;
        let mut out: Vec<Interactable> = Vec::new();
        for (&cell, &kind) in &arrows {
            let at = (matrix.zone_at)(cell)?;
            if at == Some(matrix.zone) || at.is_none() {
                out.push(Interactable {
                    code: kind,
                    source: place_source(ARROW_SOURCE, cell_number(cell))?,
                    home: Some(matrix.zone),
                    arrival: Some(landed[&cell]),
                    ..Interactable::new(ARROWS, &format!("{ARROW} {}", ways[&cell]), cell)
                });
            }
        }
        Ok(Rc::new(out))
    }

    fn listed(&self, found: &Found, _here: &Here) -> Result<Vec<Interactable>> {
        Ok(found_as::<Vec<Interactable>>(found).clone())
    }

    fn laid(&self, found: &Found, ground: Ground, _here: &Here) -> Result<Ground> {
        // Only where the zone has arrows does a search carry a push.
        if found_as::<Vec<Interactable>>(found).is_empty() {
            return Ok(ground);
        }
        Ok(Ground {
            arrows: self.ways.clone(),
            ..ground
        })
    }
}

/// Where a push from each arrow sets the player down, as the search walks it with nobody in the
/// way (`Ground._walk`): its way a cell at a time over any floor, turned by each arrow it crosses,
/// to the last cell before the way is shut. Stops on an arrow crossed twice, a ring.
fn pushed(
    arrows: &IndexMap<Cell, &'static str>,
    shut: &dyn Fn(Cell) -> Result<bool>,
) -> Result<IndexMap<Cell, Cell>> {
    let mut out: IndexMap<Cell, Cell> = IndexMap::new();
    for (&start, &way) in arrows {
        let (mut cell, mut way) = (start, way);
        // A set only tested.
        let mut crossed: HashSet<Cell> = HashSet::from([start]);
        loop {
            let (dx, dz) = pathfind::step_of(way)?;
            let ahead = (cell.0 + dx, cell.1 + dz);
            if shut(ahead)? {
                break;
            }
            cell = ahead;
            if let Some(&turned) = arrows.get(&cell) {
                if crossed.contains(&cell) {
                    break;
                }
                crossed.insert(cell);
                way = turned;
            }
        }
        out.insert(start, cell);
    }
    Ok(out)
}

/// The walls a player climbs, by the cell kinds of the ROM's chunks, as X's Cyllage Gym's
/// (`climb.Walls`, spike 245): each run of blue spots side by side on one face at one height
/// listed under climbing, a blue spot where a wall is got onto from the floor and a wall ledge
/// where it is got off onto a ledge at its side. Its sideways kinds shut the step onto a ramp
/// sideways where they stand at a ramp's sides, as in the Poké Ball Factory (`Terrain.sides`).
#[derive(Debug, Clone, PartialEq)]
pub struct Climbing {
    /// Where a face meets the floor, blocked.
    pub face: i64,
    /// The kinds of the step off a face sideways onto a ledge, each with its way.
    pub sideways: IndexMap<i64, &'static str>,
    /// The top of a slide down a wall, open: a slide only where the cells past it fall.
    pub slide: i64,
}

/// What `Climbing` finds: the walls, their sideways steps, and the spots listed.
type ClimbingFound = (Rc<Walls>, Rc<HashSet<(Cell, Cell)>>, Vec<Interactable>);

impl Landform for Climbing {
    fn found(&self, matrix: &Matrix) -> Result<Found> {
        let walls = matrix.terrain.walls(self)?;
        let mut spots: Vec<Interactable> = Vec::new();
        for (cells, ledge) in walls.spots() {
            for (first, size) in runs(cells) {
                let at = (matrix.zone_at)(first)?;
                if at.is_some_and(|at| at != matrix.zone) {
                    continue;
                }
                spots.push(Interactable {
                    size,
                    source: place_source(CLIMB_SOURCE, cell_number(first))?,
                    home: Some(matrix.zone),
                    ..Interactable::new(CLIMBING, if ledge { WALL_LEDGE } else { BLUE_SPOT }, first)
                });
            }
        }
        let found: ClimbingFound = (walls, matrix.terrain.sides(self)?, spots);
        Ok(Rc::new(found))
    }

    fn listed(&self, found: &Found, _here: &Here) -> Result<Vec<Interactable>> {
        Ok(found_as::<ClimbingFound>(found).2.clone())
    }

    fn laid(&self, found: &Found, ground: Ground, _here: &Here) -> Result<Ground> {
        let (walls, sides, _) = found_as::<ClimbingFound>(found);
        let mut cliffs = ground.cliffs.clone();
        cliffs.extend(sides.iter().copied());
        Ok(Ground {
            walls: walls.clone(),
            cliffs,
            ..ground
        })
    }
}

/// The rails a player on roller skates grinds along, by the cell kinds of the ROM's chunks, as
/// X's Shalour City Gym's (`grind.GrindRails`, spike 247). Not listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grinding {
    /// A cell of a rail, blocked.
    pub rail: i64,
    /// The cell past a rail's end that it is skated onto from, open.
    pub end: i64,
}

impl Landform for Grinding {
    fn found(&self, matrix: &Matrix) -> Result<Found> {
        Ok(Rc::new(matrix.terrain.grind_rails(self)?))
    }

    fn laid(&self, found: &Found, ground: Ground, _here: &Here) -> Result<Ground> {
        Ok(Ground {
            grind_rails: found_as::<Rc<GrindRails>>(found).clone(),
            ..ground
        })
    }
}

/// Each run of cells adjacent along a row or a column, as its first cell and its size.
pub fn runs(cells: impl IntoIterator<Item = Cell>) -> Vec<(Cell, (i64, i64))> {
    let mut left: Vec<Cell> = cells.into_iter().collect();
    left.sort();
    left.dedup();
    let mut out: Vec<(Cell, (i64, i64))> = Vec::new();
    while !left.is_empty() {
        let first = left.remove(0);
        let mut last = first;
        for step in [(1, 0), (0, 1)] {
            while let Some(at) = left
                .iter()
                .position(|&cell| cell == (last.0 + step.0, last.1 + step.1))
            {
                last = left.remove(at);
            }
            if last != first {
                break;
            }
        }
        out.push((first, (last.0 - first.0 + 1, last.1 - first.1 + 1)));
    }
    out
}

/// Gen 6's own landforms, which every game's chunks can hold, in the order they are listed and
/// laid: a game's own come after them (`Game.landforms`). A function, as `Arc`s are no const.
pub fn gen6() -> Vec<Arc<dyn Landform>> {
    vec![
        Arc::new(Bike),
        Arc::new(Holes),
        Arc::new(DiveSpots),
        Arc::new(Currents),
        Arc::new(Waterfalls),
    ]
}
