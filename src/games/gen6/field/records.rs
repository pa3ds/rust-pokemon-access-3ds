//! A zone's records as the list has them: its warps, its furniture and its triggers, each made an
//! interactable, out of the zone data the game has loaded or, for another part of a place in parts
//! listed whole, that part's zone data in the ROM (ADR-0042).
//!
//! What only the zone loaded has, the field reader asks of its own and hands in: where the rails
//! put a record on the grid, the save's works that shut a warp and arm a trigger, and the chunks
//! that wall a warp in. Another part's records are read with none of them.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{s16, u16};
use crate::core::memory::Memory;
use crate::core::pathfind::{self, Cell};
use crate::games::common::field::things::{
    Interactable, TELEPORTER, TRIGGER_SOURCE, WARP_SOURCE, place_source,
};
use crate::games::gen6::field::hidden_items::HIDDEN_ITEM;
use crate::games::gen6::field::landform::{beside, spread};
use crate::games::gen6::field::rail;
use crate::games::gen6::field::terrain::Terrain;
use crate::games::gen6::field::works::Works;
use crate::games::gen6::field::zonedata::{
    TRIGGER_HEIGHT, TRIGGER_SCRIPT, TRIGGER_VALUE, TRIGGER_WIDTH, TRIGGER_WORK, TRIGGER_X,
    TRIGGER_Y, TRIGGER_Z, WARP_DOOR, WARP_KIND, WARP_OUT, WARP_STAIRS, WARP_STREET, WARP_TO,
    WARP_Y, ZoneData, arrives, on_rail, read_furniture, trigger_placed, warp_box,
};
use crate::games::gen6::field::zonetable::ZoneTable;
use crate::games::gen6::game::{FurnitureKind, Game};
use crate::games::gen6::rom::archives::GameText;
use crate::py::Result;

/// How far over or under the ground it is walked from a warp walked through may stand and still be
/// an opening and not stairs. Over every room-to-room warp of this kind in ORAS the rise is 0 for
/// 104 of them and 1 either way for one (Seafloor Cavern's, Victory Road's entrance room), 4 either
/// way for six (Rustboro's two-storey houses, Lilycove Museum), 8 to 10 either way for 68 (Devon
/// Corporation, Littleroot's houses, the hideouts), -18 for two and 135 for three (old Mauville's
/// zone 0xf); none is 2 or 3 (2026-10-06).
pub const WARP_RISE: f64 = 2.0;
/// A warp walked through's low byte -> the way it is walked into (`WARP_STAIRS`). Every one of
/// ORAS's 362 off the rails is a wall; 361 have open floor beside them on the side this says they
/// are walked in from, and on no other side but Ever Grande City's way into Victory Road's lower
/// floor, whose ground behind stands 45 over it (spike 213).
pub const WALKED_INTO: [&str; 4] = [
    pathfind::SOUTH,
    pathfind::NORTH,
    pathfind::EAST,
    pathfind::WEST,
];

/// What a thing the player reads by facing it is, where nothing says more.
pub const PROP: &str = "object";
/// The key the records of every elevator's door share (`Records.key`): never said.
pub const ELEVATOR: &str = "elevator";
/// What a stair between two floors of one chunk is called, a warp between floors being stairs
/// too: the game names neither (spike 167).
pub const STAIRS: &str = "stairs";

/// Where a trigger placed on a street is on the grid, as its first corner and size, and how high
/// (`Records.triggers`).
pub type Placed = ((Cell, (i64, i64)), Option<f64>);

/// `Records.warps`' `on_grid` by default: no rails to lay a record.
pub fn off_the_rails(_record: &[u8]) -> Result<Option<Vec<Cell>>> {
    Ok(None)
}

/// `Records.warps`' `walled` by default; its `shut` by default is `|_| Ok(false)`.
pub fn never<T: ?Sized>(_what: &T) -> Result<bool> {
    Ok(false)
}

/// What a zone's warps, furniture and triggers are to the list.
pub struct Records {
    game: Rc<Game>,
    text: Rc<GameText>,
    zones: Rc<ZoneTable>,
    terrain: Rc<Terrain>,
    words: IndexMap<i64, String>,
    furniture: IndexMap<i64, String>,
}

impl Records {
    /// `game` says what one gen 6 game has of its own, and `text` is what was read from the
    /// player's ROM. `zones` is the zone table, which tells a path from stairs, and `terrain`
    /// the matrix out of the ROM, which tells stairs up from stairs down. `words` is what each
    /// kind of use tile is, the PC's among them, and `furniture` what the shipped table says
    /// each piece of furniture is by the shared script it runs (`Names.words`).
    pub fn new(
        game: Rc<Game>,
        text: Rc<GameText>,
        zones: Rc<ZoneTable>,
        terrain: Rc<Terrain>,
        words: IndexMap<i64, String>,
        furniture: IndexMap<i64, String>,
    ) -> Self {
        Records {
            game,
            text,
            zones,
            terrain,
            words,
            furniture,
        }
    }

    /// The warps of zone `home`, its table's `records`, as the player walks into them (`warp`):
    /// each landing the player where it comes out where it leads back into `home`, as high as
    /// that warp for a pad into its own zone, and those of one kind into one place one exit
    /// (`exits`), the place told by `place` and the story the save plays by `played`. None of
    /// the kinds only a teleporter pad puts the player at (`Game.landings`).
    ///
    /// The zone loaded and another part out of the ROM are listed as they always have been,
    /// apart: the zone loaded by the game's own zone table (`ZoneTable.place_index`) and the
    /// story played, another part by the ROM's table (`ZoneTable.place_of`), which reads nothing
    /// of the game's, and no story. And the zone loaded alone asks what is live: where the rails
    /// put a record placed on one on the grid (`on_grid`: without them, one on a rail is not
    /// listed), whether the save's works `shut` a warp, by its slot, and whether the chunks wall
    /// one off the rails in (`walled`).
    ///
    /// Python's keyword arguments default to `_off_the_rails` and `_never`: `off_the_rails` and
    /// `never` here.
    #[allow(clippy::too_many_arguments)]
    pub fn warps(
        &self,
        records: &[Vec<u8>],
        home: Option<i64>,
        place: &dyn Fn(i64) -> Result<Option<i64>>,
        played: Option<&dyn Fn(i64, i64) -> Result<Option<i64>>>,
        on_grid: &dyn Fn(&[u8]) -> Result<Option<Vec<Cell>>>,
        shut: &dyn Fn(i64) -> Result<bool>,
        walled: &dyn Fn(&Interactable) -> Result<bool>,
    ) -> Result<Vec<Interactable>> {
        let mut out: Vec<Interactable> = Vec::new();
        for (i, record) in records.iter().enumerate() {
            let shape = on_grid(record)?;
            if (shape.is_none() && on_rail(record)?)
                || self.game.landings.contains(&u16(record, WARP_KIND)?)
            {
                continue;
            }
            if shut(i as i64)? {
                continue;
            }
            let at = match &shape {
                None => None,
                Some(shape) => Some(rail::spread(shape.iter().copied())?),
            };
            let (warp, back) = (
                self.warp(i as i64, record, at, home)?,
                arrives(record, home, records)?,
            );
            if shape.is_none() && walled(&warp)? {
                continue;
            }
            // A pad into its own zone is a teleporter P walks through, landing as high as the
            // warp it comes out at (`Ground.teleports`, spike 265).
            let landing = match back {
                Some(back) if warp.kind == TELEPORTER => Some(s16(back, WARP_Y)? as f64),
                _ => None,
            };
            out.push(Interactable {
                home,
                arrival: match back {
                    None => None,
                    Some(back) => Some(warp_box(back)?.0),
                },
                landing,
                shape: shape.unwrap_or_default(),
                ..warp
            });
        }
        exits(out, place, played)
    }

    /// A warp as what the player walks through: a door, the mat out, stairs or an opening
    /// between rooms, or the path between two outdoor matrices, told by the kind and whether the
    /// zones it joins are outdoors; and what the game's table of kinds says of the rest, as a
    /// ladder or a teleporter pad (`Game.warps`). The place it arrives in is said by the
    /// navigator, which has the game's words (`Navigator.destination`).
    pub fn warp(
        &self,
        index: i64,
        record: &[u8],
        at: Option<(Cell, (i64, i64))>,
        home: Option<i64>,
    ) -> Result<Interactable> {
        let (to, kind) = (u16(record, WARP_TO)?, u16(record, WARP_KIND)?);
        let (cell, size) = match at {
            None => warp_box(record)?,
            Some(at) => at,
        };
        let what: String = if kind >> 8 == WARP_DOOR >> 8
            && match home {
                None => false,
                Some(home) => self.zones.open_air(home)?,
            }
            && self.zones.open_air(to)?
        {
            // A door's kind between two of the open air is the way through: one of Lumiose's
            // streets onto the next (spike 220), Route 7 onto Route 6 (2026-10-19).
            "path".to_string()
        } else if kind == WARP_DOOR
            || (kind >> 8 == WARP_STREET && !self.text.outdoors.contains(&to))
        {
            "door".to_string()
        } else if kind == WARP_OUT {
            "way out".to_string()
        } else if kind >> 8 == WARP_STAIRS {
            if self.zones.outdoors(to)? == Some(true) {
                "path".to_string()
            } else if at.is_none() {
                self.stairs_word(record, cell, size)?.to_string()
            } else {
                STAIRS.to_string()
            }
        } else if let Some(word) = self.game.warps.get(&kind) {
            word.clone()
        } else {
            // The high byte is what it is, the low the way it is walked into (spike 220).
            match kind >> 8 {
                high if high == WARP_DOOR >> 8 => "door",
                high if high == WARP_OUT >> 8 => "way out",
                _ => "warp",
            }
            .to_string()
        };
        let way_in = if at.is_none() && kind >> 8 == WARP_STAIRS {
            WALKED_INTO.get((kind & 0xFF) as usize).copied()
        } else {
            None
        };
        Ok(Interactable {
            size,
            zone: Some(to),
            code: kind,
            source: place_source(WARP_SOURCE, index)?,
            height: match at {
                None => Some(s16(record, WARP_Y)? as f64),
                Some(_) => None,
            },
            way_in,
            ..Interactable::new("exits", &what, cell)
        })
    }

    /// What a warp walked through is, from how its height stands to the ground it is walked
    /// from: "stairs up" over `WARP_RISE`, "stairs down" under it, and a "passage" between,
    /// an opening with no floor between its rooms. The kind's low byte says the way it is walked
    /// into, so the cell the player stands in is the one on the side opposite. "stairs" where the
    /// ROM's heights are not read, and for a way of walking the kind does not number.
    fn stairs_word(&self, record: &[u8], cell: Cell, size: (i64, i64)) -> Result<&'static str> {
        let way = u16(record, WARP_KIND)? & 0xFF;
        let (x, z) = cell;
        let (wide, deep) = size;
        let behind = [(x, z - 1), (x, z + deep), (x - 1, z), (x + wide, z)];
        let ground = if way < behind.len() as i64 {
            self.terrain.height(behind[way as usize])?
        } else {
            None
        };
        let Some(ground) = ground else {
            return Ok(STAIRS);
        };
        let rise = s16(record, WARP_Y)? as f64 - ground;
        if rise.abs() <= WARP_RISE {
            return Ok("passage");
        }
        Ok(if rise > 0.0 {
            "stairs up"
        } else {
            "stairs down"
        })
    }

    /// The furniture of zone `home`, its zone data `data`, as the list has it: each its kind
    /// (`kind`), and where the game's table sends the player through it, the category it is
    /// listed under and the zone it leads to (`Game.furniture`). Those on a rail are the rails'
    /// to place.
    pub fn furniture(
        &self,
        memory: &dyn Memory,
        data: &ZoneData,
        home: Option<i64>,
    ) -> Result<Vec<Interactable>> {
        Ok(read_furniture(memory, data)?
            .into_iter()
            .map(|item| self.listed(home, item))
            .collect())
    }

    fn listed(&self, home: Option<i64>, item: Interactable) -> Interactable {
        let listed = Interactable {
            kind: self.kind(home, &item),
            home,
            ..item
        };
        let known = home.and_then(|home| self.game.furniture.get(&(home, listed.code)));
        if let Some(FurnitureKind::Exit(category, _, zone)) = known {
            return Interactable {
                category: category.clone(),
                zone: Some(*zone),
                ..listed
            };
        }
        listed
    }

    /// What a furniture record is: a sign, unless the game's table of them says otherwise, or
    /// the shipped table says what the shared script it runs is: the one its zone's case runs,
    /// or its own number, which is a shared script's from 2000 up (spike 210). Every vending
    /// machine is one of five, 0x28f0 to 0x28f4. The PC in a Secret Base is called what the
    /// game calls a PC, as the Pokémon Center's is: the table gives it the PC's kind of use
    /// tile. A record the player leaves by is a category, a kind and a zone, and its kind is the
    /// second.
    pub fn kind(&self, home: Option<i64>, item: &Interactable) -> String {
        let kind = match home {
            Some(home) => self.game.furniture.get(&(home, item.code)),
            None => None,
        };
        let Some(kind) = kind else {
            let shared = match home {
                None => item.code,
                Some(home) => self
                    .text
                    .shared
                    .get(&(home, item.code))
                    .copied()
                    .unwrap_or(item.code),
            };
            return self
                .furniture
                .get(&shared)
                .cloned()
                .unwrap_or_else(|| item.kind.clone());
        };
        match kind {
            FurnitureKind::Tile(tile) => self
                .words
                .get(tile)
                .cloned()
                .unwrap_or_else(|| PROP.to_string()),
            FurnitureKind::Word(word) => word.clone(),
            FurnitureKind::Exit(_, kind, _) => kind.clone(),
        }
    }

    /// What makes furniture records that touch one thing (`joined`): their script, which the
    /// game runs whichever of them the player faces, as round Seafloor Cavern's submarine, four
    /// records saying "“Submarine Explorer 1” is painted on the hull" and listed as four signs.
    /// The ROM has 33 such pairs in 12 zones. An elevator's door is two
    /// records with a shared script each (`Game.elevators`), both of which run the elevator.
    ///
    /// Python's key is the script or `ELEVATOR`, an int or a str; here both are text, which no
    /// script's number spells as `ELEVATOR` does.
    pub fn key(&self, item: &Interactable) -> String {
        if self.game.elevators.contains(&item.code) {
            ELEVATOR.to_string()
        } else {
            item.code.to_string()
        }
    }

    /// The triggers of zone `home`, its table's `records`, that `Game.triggers` knows as
    /// something to walk to: Mauville Gym's switches, and the way between its two rooms, which
    /// share no floor (spike 146); Lavaridge Gym's trapdoors and geysers (spike 157); Mossdeep
    /// Gym's lifts and teleporter (spike 163); the way into Seafloor Cavern under Route 128
    /// (spike 164). And where the Extra lists hidden items, `hidden`, those a trigger gives,
    /// while it is armed by the save's `works`: once taken it is not. One placed on a street is
    /// where the rails lay it on the grid, as high as its line's start (`laid`): Coumarine Gym's
    /// way down, on its top level (spike 250). Each teleporter the ROM says puts the player
    /// somewhere in the zone, while it is armed, a sign with where it puts them (`arrival`,
    /// `landing`): Laverre Gym's panels (spike 258).
    pub fn triggers(
        &self,
        records: &[Vec<u8>],
        home: i64,
        works: &Works,
        hidden: bool,
        laid: &dyn Fn((i64, i64, i64, i64, i64)) -> Result<Option<Placed>>,
    ) -> Result<Vec<Interactable>> {
        let mut out: Vec<Interactable> = Vec::new();
        for (i, record) in records.iter().enumerate() {
            let script = u16(record, TRIGGER_SCRIPT)?;
            let armed = (u16(record, TRIGGER_WORK)?, u16(record, TRIGGER_VALUE)?);
            if let Some(&lands) = self.text.teleports.get(&(home, script)) {
                if works.armed(armed.0, armed.1)? {
                    out.push(Interactable {
                        size: (u16(record, TRIGGER_WIDTH)?, u16(record, TRIGGER_HEIGHT)?),
                        code: script,
                        source: place_source(TRIGGER_SOURCE, i as i64)?,
                        home: Some(home),
                        height: Some(s16(record, TRIGGER_Y)? as f64),
                        arrival: Some((lands.0, lands.2)),
                        landing: Some(lands.1 as f64),
                        ..Interactable::new(
                            "signs",
                            TELEPORTER,
                            (u16(record, TRIGGER_X)?, u16(record, TRIGGER_Z)?),
                        )
                    });
                }
                continue;
            }
            if self.game.hidden_triggers.contains(&(home, script)) {
                if hidden && works.holds(armed.0, armed.1)? {
                    out.push(Interactable {
                        size: (u16(record, TRIGGER_WIDTH)?, u16(record, TRIGGER_HEIGHT)?),
                        code: script,
                        source: place_source(TRIGGER_SOURCE, i as i64)?,
                        home: Some(home),
                        height: Some(s16(record, TRIGGER_Y)? as f64),
                        ..Interactable::new(
                            "items",
                            HIDDEN_ITEM,
                            (u16(record, TRIGGER_X)?, u16(record, TRIGGER_Z)?),
                        )
                    });
                }
                continue;
            }
            let Some((category, kind, to)) = self.game.triggers.get(&(home, script)) else {
                continue;
            };
            let place = trigger_placed(record)?;
            let (r#box, height) = match place {
                None => (
                    (
                        (u16(record, TRIGGER_X)?, u16(record, TRIGGER_Z)?),
                        (u16(record, TRIGGER_WIDTH)?, u16(record, TRIGGER_HEIGHT)?),
                    ),
                    Some(s16(record, TRIGGER_Y)? as f64),
                ),
                Some(place) => {
                    let Some(on) = laid(place)? else {
                        continue;
                    };
                    on
                }
            };
            out.push(Interactable {
                size: r#box.1,
                zone: *to,
                code: script,
                source: place_source(TRIGGER_SOURCE, i as i64)?,
                home: Some(home),
                height,
                ..Interactable::new(category, kind, r#box.0)
            });
        }
        // Exits of one kind into one zone that touch are one, as warps are (`exits`): the
        // Radiant Chamber's lift is four triggers round its platform. In the table's order still.
        // Python's key is the tuple (kind, zone, height, home); its parts written out here.
        let mut listed = joined(out, &|item| {
            (item.category == "exits").then(|| {
                format!(
                    "{}|{:?}|{:?}|{:?}",
                    item.kind, item.zone, item.height, item.home
                )
            })
        });
        listed.sort_by_key(|item| item.source);
        Ok(listed)
    }
}

/// Every cell the warps off the rails cover.
pub fn warp_cells(records: &[Vec<u8>]) -> Result<HashSet<Cell>> {
    let mut out: HashSet<Cell> = HashSet::new();
    for record in records {
        if on_rail(record)? {
            continue;
        }
        let ((x0, z0), (wide, deep)) = warp_box(record)?;
        for x in x0..x0 + 1.max(wide) {
            for z in z0..z0 + 1.max(deep) {
                out.insert((x, z));
            }
        }
    }
    Ok(out)
}

/// `items` with those of one `key` that touch side to side as one thing over the box round
/// them, as the first of them listed (`spread`); a key of None is kept apart. One thing the
/// game builds of several records: Seafloor Cavern's submarine is four furniture records with
/// one script, an elevator's door two side by side, and three stand-ins in a row in front of
/// Groudon (spike 209). An item is never split: a warp across a diagonal street covers cells
/// touching only at their corners, and grouped by its cells it was listed once a cell, under one
/// key, and round Prism Tower `L` went back and forth between two exits.
pub fn joined(
    items: Vec<Interactable>,
    key: &dyn Fn(&Interactable) -> Option<String>,
) -> Vec<Interactable> {
    let mut out: Vec<Interactable> = Vec::new();
    let mut keyed: IndexMap<String, Vec<Interactable>> = IndexMap::new();
    for item in items {
        match key(&item) {
            None => out.push(item),
            Some(which) => keyed.entry(which).or_default().push(item),
        }
    }
    for alike in keyed.values() {
        // Each group's cells and its members, by their places in `alike`.
        let mut groups: Vec<(HashSet<Cell>, Vec<usize>)> = Vec::new();
        for (index, item) in alike.iter().enumerate() {
            let cells: HashSet<Cell> = item.cells().into_iter().collect();
            let mut near = cells.clone();
            for &cell in &cells {
                near.extend(beside(cell));
            }
            let (touched, rest): (Vec<_>, Vec<_>) = groups
                .into_iter()
                .partition(|group| !near.is_disjoint(&group.0));
            groups = rest;
            let (mut all, mut members) = (cells, Vec::new());
            for (cells, those) in touched {
                all.extend(cells);
                members.extend(those);
            }
            members.push(index);
            groups.push((all, members));
        }
        for (cells, members) in groups {
            // `min(members, key=alike.index)`: the first item equal to each member.
            let first = members
                .iter()
                .map(|&i| alike.iter().position(|item| *item == alike[i]).unwrap_or(i))
                .min()
                .expect("a group has a member");
            let first = &alike[first];
            let group: Vec<Cell> = cells.iter().copied().collect();
            let (cell, size) = spread(&group);
            // Warps across a diagonal street cover a row of cells, not their box.
            let shape = if members.iter().any(|&i| !alike[i].shape.is_empty()) {
                let mut sorted = group.clone();
                sorted.sort();
                sorted
            } else {
                Vec::new()
            };
            out.push(Interactable {
                cell,
                size,
                shape,
                ..first.clone()
            });
        }
    }
    out
}

/// `warps` as the player walks into them: those of one kind into one zone that touch side to
/// side as one exit (`joined`), and none whose cells a warp before it of its kind into the same
/// `place` covers already, which is said alike and walked to alike. Lumiose's streets go on into
/// the next through three warps a cell apart, "warp to Centrico Plaza here", "1 east" and "1
/// west", and the Tower of Mastery's stairs up are three side by side twice over, to zones 0xb7
/// and 0xb9, both the Tower. A game takes one of two warps on one cell into two places by the
/// story its save plays: Lilycove's way into the hideout is Team Aqua's and Team Magma's on the
/// same cells, and Omega Ruby's is Magma's, the second (spike 251). `played` says which of the
/// two zones is the story's, and only its warp is listed; where it says neither, both stay
/// (`Game.version_flags`, spike 212).
pub fn exits(
    warps: Vec<Interactable>,
    place: &dyn Fn(i64) -> Result<Option<i64>>,
    played: Option<&dyn Fn(i64, i64) -> Result<Option<i64>>>,
) -> Result<Vec<Interactable>> {
    let mut kept: Vec<Interactable> = Vec::new();
    for item in warps {
        let cells: HashSet<Cell> = item.cells().into_iter().collect();
        // Indices into `kept` rather than the items, so that `kept.index(twin)` is the twin's.
        let under: Vec<usize> = (0..kept.len())
            .filter(|&i| {
                kept[i].kind == item.kind && cells.is_subset(&kept[i].cells().into_iter().collect())
            })
            .collect();
        let mut covered = false;
        for &i in &under {
            if place(kept[i].zone.unwrap_or(0))? == place(item.zone.unwrap_or(0))? {
                covered = true;
                break;
            }
        }
        if covered {
            continue;
        }
        let twin = under.iter().copied().find(|&i| kept[i].zone.is_some());
        if let (Some(played), Some(twin)) = (played, twin)
            && let (Some(twin_zone), Some(zone)) = (kept[twin].zone, item.zone)
            && zone != 0
            && let Some(story) = played(twin_zone, zone)?
        {
            if story == zone {
                kept[twin] = item;
            }
            continue;
        }
        kept.push(item);
    }
    // Python's key is the tuple (kind, zone, height, home); its parts written out here.
    Ok(joined(kept, &|item| {
        Some(format!(
            "{}|{:?}|{:?}|{:?}",
            item.kind, item.zone, item.height, item.home
        ))
    }))
}
