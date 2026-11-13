//! Parfum Palace's garden, zone 0x12e, where the player and Shauna corner a Furfrou in the hedge
//! maze (spike 292): the four spots the two stand at, and what to do next, listed while the chase
//! is on.
//!
//! The zone's script keeps the chase in three of the save's works, and moves both by them alone:
//!
//! ```text
//! 0x407e  how far the story is: 2 while the chase is on, 3 once the player has walked out by
//!         the maze's entrance, 4 as the Furfrou is caught
//! 0x4138  where Shauna waits: 0 following the player, 1 to 4 a spot, and 5 the south spot once
//!         her second hint has put her there, where she is no longer talked to
//! 0x4139  where the Furfrou stands, 1 to 4 a spot
//! ```
//!
//! The spots are the middles of the four sides of the maze's ring, 1 north (39, 30), 2 south
//! (39, 51), 3 east (48, 42) and 4 west (30, 42). Shauna waits at the one whose crossing the player
//! talks to her in. The Furfrou runs to another as the player crosses a line three cells from it, by
//! a table of the script's: from east or west to the north one only while Shauna waits at the south
//! one, and from the north one, whose only other way is a dead end, into that dead end and caught
//! where Shauna waits at the spot it would run to. So she waits south first and east or west after.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::discovery::Found;
use crate::core::memory::Memory;
use crate::core::pathfind::Cell;
use crate::core::words::{IntoValue, own};
use crate::games::common::field::things::{
    Interactable, TILE_SOURCE, TRIGGER_SOURCE, place_source,
};
use crate::games::gen6::field::features::{Feature, Tracked, ZoneAt};
use crate::games::gen6::field::works::Works;
use crate::games::gen6::rom::archives::GameText;
use crate::py::{Error, Result};

pub const GARDEN: i64 = 0x12E;
pub const STORY: i64 = 0x407E;
pub const SHAUNA_AT: i64 = 0x4138;
pub const FURFROU_AT: i64 = 0x4139;
/// What the story's work holds while the chase is on, and once the player has left the maze.
pub const CHASING: i64 = 2;
pub const LEFT: i64 = 3;
/// What Shauna's work holds: the Furfrou's holds a spot of the four.
pub const FOLLOWING: i64 = 0;
pub const NORTH: i64 = 1;
pub const SOUTH: i64 = 2;
pub const EAST: i64 = 3;
pub const WEST: i64 = 4;
pub const LOCKED: i64 = 5;
/// Each spot's cell and what it is listed as, a word that says what it is by.
pub const SPOTS: [(i64, (Cell, &str)); 4] = [
    (NORTH, ((39, 30), "north waiting spot")),
    (SOUTH, ((39, 51), "south waiting spot")),
    (EAST, ((48, 42), "east waiting spot")),
    (WEST, ((30, 42), "west waiting spot")),
];
/// (the Furfrou's spot, Shauna's) -> a cell on the line that sends it on toward being caught:
/// from east or west, the side of the maze's middle, to the north spot; from the north spot, the
/// side away from Shauna, into the dead end.
pub const CHASE: [((i64, i64), Cell); 4] = [
    ((EAST, SOUTH), (45, 42)),
    ((WEST, SOUTH), (33, 42)),
    ((NORTH, EAST), (36, 30)),
    ((NORTH, WEST), (42, 30)),
];
/// (the Furfrou's spot, Shauna's) -> the line across the path, its first cell and size, that
/// sends it the wrong way then: north of the east or west spot, off to the other of the two, and
/// from the north spot the side Shauna waits on, past her. Listed as a trigger is, so a way to
/// `CHASE`'s cell goes round it: P's shortest from Shauna at the east spot ran up the east path and
/// onto this line (x_parfum_palace_garden with presses added).
pub const KEEP_OFF: [((i64, i64), (Cell, (i64, i64))); 4] = [
    ((EAST, SOUTH), ((47, 39), (3, 1))),
    ((WEST, SOUTH), ((29, 39), (3, 1))),
    ((NORTH, EAST), ((42, 29), (1, 3))),
    ((NORTH, WEST), ((36, 29), (1, 3))),
];
/// The line's number as a trigger's source, past any trigger of the zone's own table.
pub const KEEP_OFF_SOURCE: i64 = 0x100;
/// On the line north of the south spot, which sends the Furfrou off it to the east or west.
pub const FROM_MIDDLE: Cell = (39, 48);
/// The cells of the entrance's two triggers: walked onto going out, Shauna stays at the south
/// spot and the Furfrou goes back to the east one; walked onto coming back, she follows again.
pub const OUTSIDE: Cell = (39, 56);
pub const INSIDE: Cell = (39, 54);
/// Shauna's sprite and the Furfrou's species, which the game's text names.
pub const SHAUNA: i64 = 0x03;
pub const FURFROU: i64 = 676;

/// `SPOTS[number]`, or None for no spot (`number in SPOTS`).
pub fn spot(number: i64) -> Option<(Cell, &'static str)> {
    SPOTS
        .iter()
        .find(|(each, _)| *each == number)
        .map(|&(_, spot)| spot)
}

/// What to do next, a message, and where; None while there is no chase.
pub fn step(story: i64, shauna: i64, furfrou: i64) -> Result<Option<(&'static str, Cell)>> {
    if story == LEFT {
        return Ok(Some(("furfrou-enter", INSIDE)));
    }
    if story != CHASING || spot(furfrou).is_none() {
        return Ok(None);
    }
    if shauna == LOCKED {
        return Ok(Some(("furfrou-leave", OUTSIDE)));
    }
    if furfrou == SOUTH {
        return Ok(Some(("furfrou-chase", FROM_MIDDLE)));
    }
    let wanted: &[i64] = if furfrou == NORTH {
        &[EAST, WEST]
    } else {
        &[SOUTH]
    };
    if wanted.contains(&shauna) {
        let Some(&(_, cell)) = CHASE.iter().find(|(key, _)| *key == (furfrou, shauna)) else {
            return Err(Error::key(format!("({furfrou}, {shauna})")));
        };
        return Ok(Some(("furfrou-chase", cell)));
    }
    if let Some((cell, _kind)) = spot(shauna) {
        return Ok(Some(("furfrou-fetch", cell)));
    }
    Ok(spot(wanted[0]).map(|(cell, _kind)| ("furfrou-wait", cell)))
}

/// The garden's spots and the next step of the chase, for the field to list
/// (`gen6.field.Feature`).
pub struct FurfrouChase {
    works: Works,
    pub classes: Vec<&'static str>,
    names: IndexMap<&'static str, Option<String>>,
}

impl FurfrouChase {
    pub fn new(memory: Rc<dyn Memory>, text: &GameText) -> Self {
        let works = Works::new(memory);
        let classes = works.classes.clone();
        FurfrouChase {
            works,
            classes,
            names: IndexMap::from([
                ("shauna", text.character(SHAUNA)),
                ("furfrou", text.pokemon(FURFROU, 0)),
            ]),
        }
    }

    /// The factory `Game.features` holds.
    pub fn of(memory: Rc<dyn Memory>, text: &GameText) -> Rc<dyn Feature> {
        Rc::new(FurfrouChase::new(memory, text))
    }

    pub fn track(&self, found: &Found) -> Result<()> {
        self.works.track(found)
    }

    pub fn listed(&self, zone: i64, _zone_at: ZoneAt) -> Result<Vec<Interactable>> {
        // The zone first: nothing is read of the works anywhere else.
        if zone != GARDEN {
            return Ok(Vec::new());
        }
        let Some(story) = self.works.value(STORY)? else {
            return Ok(Vec::new());
        };
        let shauna = self.works.value(SHAUNA_AT)?.unwrap_or(0);
        let furfrou = self.works.value(FURFROU_AT)?.unwrap_or(0);
        let found = step(story, shauna, furfrou)?;
        let spots: &[(i64, (Cell, &str))] = if story == CHASING { &SPOTS } else { &[] };
        let mut places: Vec<(String, Cell)> = spots
            .iter()
            .map(|&(_, (cell, kind))| (kind.to_string(), cell))
            .collect();
        let mut lines: Vec<Interactable> = Vec::new();
        // Without the game's names for the two, the step has nobody to name.
        let named = self
            .names
            .values()
            .all(|name| name.as_deref().is_some_and(|name| !name.is_empty()));
        if let Some((message, cell)) = found
            && named
        {
            let names: Vec<(&str, &dyn IntoValue)> = self
                .names
                .iter()
                .filter_map(|(key, name)| Some((*key, name.as_ref()? as &dyn IntoValue)))
                .collect();
            places.push((own(message, &names), cell));
            if message == "furfrou-chase"
                && let Some(&(_, (cell, size))) =
                    KEEP_OFF.iter().find(|(key, _)| *key == (furfrou, shauna))
            {
                lines.push(Interactable {
                    size,
                    home: Some(zone),
                    source: place_source(TRIGGER_SOURCE, KEEP_OFF_SOURCE)?,
                    ..Interactable::new("signs", &own("furfrou-keep-off", &names), cell)
                });
            }
        }
        let mut out: Vec<Interactable> = Vec::new();
        for (kind, cell) in places {
            out.push(Interactable {
                home: Some(zone),
                source: place_source(TILE_SOURCE, cell.0 << 12 | cell.1)?,
                ..Interactable::new("signs", &kind, cell)
            });
        }
        out.extend(lines);
        Ok(out)
    }
}

impl Tracked for FurfrouChase {
    fn classes(&self) -> Vec<&'static str> {
        self.classes.clone()
    }

    fn track(&self, found: &Found) -> Result<()> {
        FurfrouChase::track(self, found)
    }
}

impl Feature for FurfrouChase {
    fn listed(&self, zone: i64, zone_at: ZoneAt) -> Result<Vec<Interactable>> {
        FurfrouChase::listed(self, zone, zone_at)
    }
}
