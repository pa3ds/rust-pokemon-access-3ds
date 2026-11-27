//! Who stands on the map: the move models in the pool, each what it is to the field (`Role`),
//! listed as what it is drawn as, and another part's NPC records as the models the game would build
//! from them, for a place in parts listed whole (ADR-0042).
//!
//! The pool's slots are found by a sweep and kept by the field reader, which hands them here
//! (`Models.pool`); the model's own shape is `mmodel`'s.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u16, u32};
use crate::core::memory::Memory;
use crate::core::pathfind::Cell;
use crate::games::common::field::chunk::{COUNTER, Chunk};
use crate::games::common::field::things::{Interactable, OBSTACLES, POKEMON};
use crate::games::gen6::field::landform::Rides;
use crate::games::gen6::field::mmodel;
use crate::games::gen6::field::records::{PROP, joined, warp_cells};
use crate::games::gen6::field::works::Works;
use crate::games::gen6::field::zonedata::{
    NPC, NPC_INDEX, NPC_SCRIPT, NPC_SPRITE, NPC_X, NPC_Y, NPC_Z, WARP, ZoneData, read_furniture,
    read_records,
};
use crate::games::gen6::game::Game;
use crate::py::Result;

/// What a beam between two posts is called, which a switch swings a quarter turn: the game never
/// names one, and its move model's sprite is the invisible blocker it stands elsewhere (spike
/// 146).
pub const BARRIER: &str = "barrier";

/// What a move model is to the field, which says whether it is listed and whether it stands in
/// the way (`Models.role`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The player drawn as another model, as the Magma Suit is put on in the Cave of Origin,
    /// listed there as "person, here" (spike 203): neither listed nor in the way.
    Look,
    /// A beam a switch swings, in its zone (`Game.barriers`, spike 146): listed under obstacles,
    /// and in the way as anything listed is.
    Barrier,
    /// The game's invisible blocker (`Game.blockers`): never listed, and in the way. Sootopolis
    /// Gym's nine, rows of three across the corridors up from its rooms, were listed as nameless
    /// people (spike 167). Round a zone's rotating doors they stand in for the doors' arms, which
    /// the doors are walked through by, and are not in the way: five were listed as "person 1
    /// west" round Fortree Gym's third door, and P said "person in the way" (spike 161).
    Blocker,
    /// A work object no script has filled (`mmodel.drawn_as` gives its own id): drawn as
    /// nothing, neither listed nor in the way. The Contest Spectacular Halls' lobbies stand ten,
    /// `0x7b` to `0x84` in two rows beside the carpet, listed as "person" until 2026-10-23; none
    /// was drawn, and the player walked onto one (or_contest_hall_phantom_people; findings, the
    /// map, 2026-10-23). One whose looks are still loading reads the same, and is left out until
    /// they have.
    Empty,
    /// A model drawn as nothing (`mmodel.NONDRAW`): in the way, as on each of Sootopolis City's
    /// locked doors, and in its own zone listed as what it stands in for (`Models.people`, spike
    /// 203).
    Nothing,
    /// Anything else: listed as what it is drawn as (`Models.item`).
    Drawn,
}

/// `_stand_ins`: what each model drawn as nothing stands in for by its record's index, the
/// cells of the zone's warps, and the records' indices of its gates.
type StandIns = (IndexMap<i64, i64>, HashSet<Cell>, HashSet<i64>);

/// The zone data in use (`FieldReader._current`), which the field reader hands in.
pub type Current = Rc<dyn Fn() -> Result<Option<ZoneData>>>;

/// The move models on the map, and another part's NPC records, as the list has them.
pub struct Models {
    memory: Rc<dyn Memory>,
    game: Rc<Game>,
    works: Rc<Works>,
    rides: Option<Rides>,
    current: Current,
    /// The move models' slots, as the last sweep found them.
    pub pool: RefCell<Vec<i64>>,
}

impl Models {
    /// `game` says what one gen 6 game has of its own; `works` are the save's works and flags,
    /// which say whether an item ball is picked up; `rides` is what the game's ridden Pokémon
    /// cross, or None in a game without one; and `current` gives the zone data in use, whose
    /// records say what a model drawn as nothing stands in for.
    pub fn new(
        memory: Rc<dyn Memory>,
        game: Rc<Game>,
        works: Rc<Works>,
        rides: Option<Rides>,
        current: Current,
    ) -> Self {
        Models {
            memory,
            game,
            works,
            rides,
            current,
            pool: RefCell::new(Vec::new()),
        }
    }

    /// Every model in use but the player's, with its head, in the pool's order; none the
    /// game has hidden (`mmodel.hidden`), which is neither listed nor in the way. The Secret
    /// Base Guild hides four round Aarune filled with the pals of the player's own Secret Base,
    /// and they were listed until 2026-11-12, one on the player's own cell
    /// (or_secret_base_guild_bug).
    ///
    /// Python's generator reads each head as its loop comes to it; so does this iterator, so the
    /// reads a loop makes of each model come between the heads, as there.
    pub fn others(&self) -> impl Iterator<Item = Result<(i64, Vec<u8>)>> + '_ {
        let models = self.pool.borrow().clone();
        models.into_iter().filter_map(move |model| {
            let other = || -> Result<Option<(i64, Vec<u8>)>> {
                let head = self.memory.read(model, mmodel::HEAD)?;
                Ok((mmodel::in_use(&head)?
                    && mmodel::index_of(&head)? != mmodel::PLAYER_INDEX
                    && !mmodel::hidden(&head)?)
                .then_some((model, head)))
            };
            other().transpose()
        })
    }

    /// What a model drawn as `code` is to the field (`Role`), by the game's tables of looks,
    /// barriers and blockers: a barrier only in its own `zone`, given for a model built there.
    pub fn role(&self, code: i64, zone: Option<i64>) -> Role {
        if self.game.player_looks.contains(&code) {
            Role::Look
        } else if zone.is_some_and(|zone| self.game.barriers.contains(&(zone, code))) {
            Role::Barrier
        } else if self.game.blockers.contains(&code) {
            Role::Blocker
        } else if mmodel::WORK_OBJECTS.contains(&code) {
            Role::Empty
        } else if code == mmodel::NONDRAW {
            Role::Nothing
        } else {
            Role::Drawn
        }
    }

    /// Every model listed (`role`): an item ball or a thing by its model code, a person
    /// otherwise. The moving truck has a person's kind of code and no script, and trainers have
    /// no script either, so it passes for a person until model codes have names. A big boulder
    /// in a hole is no parked model, however walled in: Seafloor Cavern's room 0x69 has walls
    /// north and west of its north-west hole, and the boulder pushed into it was left out, its
    /// hole still listed and a wall (movie37). A model on a street that is a floor of its own is
    /// on floor, though its chunk walls the cell: in Coumarine Gym the entrance listed one of the
    /// four Pokémon Rangers, the one over the corridor its chunk leaves open (x_gym_4_climbed).
    /// `chunks` are the chunks loaded with those streets laid in, and `holes` the zone's holes a
    /// big boulder drops into.
    ///
    /// A model drawn as nothing is called what it stands in for (`_stand_ins`), those side by
    /// side as one (`joined`): the three in front of Groudon are one "Groudon", the first by its
    /// record, and none once Groudon itself is listed. It is left out on a door, which is listed
    /// already (spike 203), and where it is one of a gate's (`Game.gates`), which is nobody
    /// (spike 223).
    pub fn people(
        &self,
        zone: i64,
        chunks: &[Rc<Chunk>],
        holes: &HashSet<Cell>,
    ) -> Result<Vec<Interactable>> {
        let mut out: Vec<Interactable> = Vec::new();
        // The stand-ins called what another record draws.
        let mut standing_in: Vec<Interactable> = Vec::new();
        let mut stand_ins: Option<StandIns> = None;
        let none = HashSet::new();
        let rides = match &self.rides {
            None => &none,
            Some(rides) => &rides.ground,
        };
        for other in self.others() {
            let (model, head) = other?;
            let Some(cell) = mmodel::cell(&head)? else {
                continue;
            };
            if !holes.contains(&cell) && walled_in(chunks, cell, rides)? {
                continue;
            }
            let code = self.drawn_as(model, &head)?;
            let role = self.role(code, Some(mmodel::zone_of(&head)?));
            if role == Role::Look || role == Role::Blocker || role == Role::Empty {
                continue;
            }
            let mut stood_in = None;
            if role == Role::Nothing && mmodel::zone_of(&head)? == zone {
                if stand_ins.is_none() {
                    stand_ins = Some(self.stand_ins()?);
                }
                let (by_index, doors, gates) = stand_ins.as_ref().expect("just read");
                stood_in = by_index.get(&mmodel::index_of(&head)?).copied();
                if stood_in.is_none()
                    && (doors.contains(&cell) || gates.contains(&mmodel::index_of(&head)?))
                {
                    continue;
                }
            }
            let (cell, size) = mmodel::footprint(&head, cell)?;
            // `stood_in or code`: a stand-in's sprite is never NONDRAW, 0, so never falsy.
            let called = match stood_in {
                Some(sprite) if sprite != 0 => sprite,
                _ => code,
            };
            let item = self.item(model, &head, cell, size, Some(called))?;
            match stood_in {
                None => out.push(item),
                Some(_) => standing_in.push(item),
            }
        }
        let listed: HashSet<i64> = out.iter().map(|item| item.code).collect();
        standing_in.sort_by_key(|item| item.record.unwrap_or((0, 0)));
        for item in joined(standing_in, &|item| Some(item.code.to_string())) {
            if !listed.contains(&item.code) {
                out.push(item);
            }
        }
        Ok(out)
    }

    /// What the zone's models drawn as nothing stand in for, by their records' indices: the
    /// sprite of another record with the same script, as the three in a row in front of Groudon
    /// deep in the Cave of Origin have its script, where Groudon itself stands in the magma; the
    /// cells of the zone's warps, where Sootopolis City's ten stand, one on each locked door
    /// (spike 203); and, in a game with `gates`, the records' indices of the gates, models drawn
    /// as nothing that share a script with one another and with no drawn record: the five behind
    /// Viola in Santalune Gym, shut until the game opens the tent after her battle (spike 223).
    /// The cells of its furniture are among the warps': each of X's boutiques stands one on its
    /// fitting room, listed already, and it was listed beside it as "person 1 north"
    /// (x_cyllage_boutique_fitting_room).
    fn stand_ins(&self) -> Result<StandIns> {
        let Some(data) = (self.current)()? else {
            return Ok((IndexMap::new(), HashSet::new(), HashSet::new()));
        };
        let records = read_records(&*self.memory, &data, NPC)?;
        let mut unseen: HashSet<i64> = self.game.blockers.clone();
        unseen.insert(mmodel::NONDRAW);
        let mut drawn: IndexMap<i64, i64> = IndexMap::new();
        for record in &records {
            if !unseen.contains(&u16(record, NPC_SPRITE)?) && u16(record, NPC_SCRIPT)? != 0 {
                drawn.insert(u16(record, NPC_SCRIPT)?, u16(record, NPC_SPRITE)?);
            }
        }
        let mut stand_ins: IndexMap<i64, i64> = IndexMap::new();
        for record in &records {
            if u16(record, NPC_SPRITE)? == mmodel::NONDRAW
                && let Some(sprite) = drawn.get(&u16(record, NPC_SCRIPT)?)
            {
                stand_ins.insert(u16(record, NPC_INDEX)?, *sprite);
            }
        }
        let mut shared: HashMap<i64, i64> = HashMap::new();
        for record in &records {
            if u16(record, NPC_SPRITE)? == mmodel::NONDRAW {
                *shared.entry(u16(record, NPC_SCRIPT)?).or_default() += 1;
            }
        }
        let mut gates: HashSet<i64> = HashSet::new();
        for record in &records {
            if !self.game.gates || u16(record, NPC_SPRITE)? != mmodel::NONDRAW {
                continue;
            }
            let script = u16(record, NPC_SCRIPT)?;
            if script != 0 && !drawn.contains_key(&script) && shared[&script] > 1 {
                gates.insert(u16(record, NPC_INDEX)?);
            }
        }
        let mut doors = warp_cells(&read_records(&*self.memory, &data, WARP)?)?;
        for item in read_furniture(&*self.memory, &data)? {
            doors.extend(item.cells());
        }
        Ok((stand_ins, doors, gates))
    }

    /// The cells of the models never listed as themselves, with how high: those drawn as
    /// nothing, in the way, and the invisible blockers, in the way but round rotating doors
    /// (`role`, `FieldReader.ground`).
    #[allow(clippy::type_complexity)]
    pub fn unseen(&self) -> Result<(IndexMap<Cell, Option<f64>>, IndexMap<Cell, Option<f64>>)> {
        let mut nothing: IndexMap<Cell, Option<f64>> = IndexMap::new();
        let mut blockers: IndexMap<Cell, Option<f64>> = IndexMap::new();
        for other in self.others() {
            let (model, head) = other?;
            let code = self.drawn_as(model, &head)?;
            let cells = match self.role(code, Some(mmodel::zone_of(&head)?)) {
                Role::Nothing => &mut nothing,
                Role::Blocker => &mut blockers,
                _ => continue,
            };
            if let Some(cell) = mmodel::cell(&head)? {
                cells.insert(cell, mmodel::height(&head)?);
            }
        }
        Ok((nothing, blockers))
    }

    /// A model as what it is, standing in `size` cells from `cell`: an item ball or a thing by its
    /// model code, a person otherwise; `code` is the sprite it is called by, where that is not the
    /// one it is drawn with.
    ///
    /// Told apart by the record it was built from (`Interactable.key`): walking into the next
    /// zone fills the slots of the zone left with the new zone's people, and the way to a
    /// Youngster on Route 102 became a way to Norman in Petalburg City, in the same slot (spike
    /// 131).
    pub fn item(
        &self,
        model: i64,
        head: &[u8],
        cell: Cell,
        size: (i64, i64),
        code: Option<i64>,
    ) -> Result<Interactable> {
        let code = match code {
            Some(code) => code,
            None => self.drawn_as(model, head)?,
        };
        let source = u32(head, mmodel::ZONE)? << 32 | model;
        let (category, kind) = self.kind_of(code, mmodel::zone_of(head)?);
        Ok(Interactable {
            size,
            code,
            source,
            move_model: true,
            height: mmodel::height(head)?,
            record: Some((mmodel::zone_of(head)?, mmodel::index_of(head)?)),
            ..Interactable::new(category, kind, cell)
        })
    }

    /// An NPC record of another part as the list has the model built from it, which the game
    /// has not built: by its sprite (`_kind_of`), where its record starts it, told by its record
    /// as the model is (`Interactable.key`). None for what is never listed (`role`'s looks,
    /// blockers, work objects and models drawn as nothing), and for an item ball or a thing to
    /// pick up whose flag says it is gone, or while the save's flags are not found: its record's
    /// script number names the flag the game sets as it is picked up (spike 269). A barrier's
    /// beam is drawn as the blocker it stands elsewhere, and its role is asked with no zone: in
    /// another part it is that blocker, never listed.
    pub fn record(&self, zone: i64, record: &[u8]) -> Result<Option<Interactable>> {
        let code = u16(record, NPC_SPRITE)?;
        if self.role(code, None) != Role::Drawn {
            return Ok(None);
        }
        let (category, kind) = self.kind_of(code, zone);
        if category == "items" && self.works.flag(u16(record, NPC_SCRIPT)?)? != Some(false) {
            return Ok(None);
        }
        let index = u16(record, NPC_INDEX)?;
        let height = f32(record, NPC_Y)?;
        Ok(Some(Interactable {
            code,
            source: (zone << 16 | index) << 32,
            move_model: true,
            height: if height.is_finite() {
                Some(height)
            } else {
                None
            },
            record: Some((zone, index)),
            ..Interactable::new(category, kind, (u16(record, NPC_X)?, u16(record, NPC_Z)?))
        }))
    }

    /// Who or what a model is drawn as, a work object as whoever it was filled with
    /// (`mmodel.drawn_as`), where the game keeps it (`Game.drawn`, `Game.models`).
    fn drawn_as(&self, model: i64, head: &[u8]) -> Result<i64> {
        mmodel::drawn_as(
            &*self.memory,
            model,
            head,
            self.game.drawn,
            self.game.models,
        )
    }

    /// The category and kind of a model drawn as `code`, built from a record of `zone`: an
    /// item ball or a thing by its code, a person otherwise.
    fn kind_of(&self, code: i64, zone: i64) -> (&'static str, &'static str) {
        let game = &self.game;
        if mmodel::ITEM_BALLS.contains(&code) {
            return ("items", "item ball");
        }
        if game.pickups.contains(&code) {
            return ("items", PROP);
        }
        if game.ambushes.contains(&code) || game.lurkers.contains(&code) {
            return (POKEMON, PROP);
        }
        if game.clears.contains_key(&code)
            || self
                .rides
                .as_ref()
                .is_some_and(|rides| rides.smashes.contains(&code))
        {
            return (OBSTACLES, PROP);
        }
        if code >= mmodel::PROP_MODEL {
            return ("signs", PROP);
        }
        if game.barriers.contains(&(zone, code)) {
            return (OBSTACLES, BARRIER);
        }
        ("people", "person")
    }
}

/// Whether the cell and every cell round it are walls in the chunks loaded: where the game keeps a
/// move model it has built and not put on the map. A Secret Base parks the six people of the
/// player's secret team in the walls of its far corner until each is registered, and they were
/// listed as six people there with no way to them (spike 154). Ground only a ridden Pokémon
/// crosses, of a kind of `rides`, is no wall: once its chunk was loaded, Route 9's rocks on the
/// spikes and the Rhyhorn ridden over them were left out (spike 238). Nor is a counter, which a
/// clerk behind it is talked to across: Lavaridge's herbalist, the Lilycove Department Store's
/// receptionists, Devon's fossil scientist and the Secret Base Guild's two shops each stand in
/// walls but for the counter in front, and went unlisted (a player's report, spike 233). Off
/// the map's north or west edge is wall too, as `reader._buried` has it: Shabboneau Castle's
/// caretaker, whose record stands him at (0, 0) until his scene walks him in, was listed there
/// after it, with no way to him.
pub fn walled_in(chunks: &[Rc<Chunk>], cell: Cell, rides: &HashSet<i64>) -> Result<bool> {
    let wall = |near: Cell| -> Result<bool> {
        if near.0 < 0 || near.1 < 0 {
            return Ok(true);
        }
        match chunks.iter().find(|chunk| chunk.holds(near)) {
            None => Ok(false),
            Some(chunk) => Ok(!chunk.walkable(near)?
                && !rides.contains(&chunk.kind(near)?)
                && chunk.kind(near)? != COUNTER),
        }
    };
    let (x, z) = cell;
    for dx in [-1, 0, 1] {
        for dz in [-1, 0, 1] {
            if !wall((x + dx, z + dz))? {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
