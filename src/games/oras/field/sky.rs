//! Soaring: where the player is over Hoenn on Latios or Latias, the places the game shows them, and
//! the flocks of Pokémon flying about.
//!
//! ORAS runs the sky as an app of its own, `app::skyTrip::MainProc`, from the module `DllSkyTrip`;
//! the field's lists are stale the whole time it runs (spike 166). The process leads to the camera,
//! which the module builds again once the climb out of the town is done, to the places' record and
//! to the manager of the sky's models:
//!
//! ```text
//! process +0xb4  -> the camera
//!         +0x154 -> the places: +0x40 -> a byte a row of the table, 1 where the game shows it
//!                   today; +0x44 u16 today's mirage spot
//!         +0x160 -> the manager: +0xa0 -> the model built last
//! camera  +0x0c f32 the heading, in radians: the player flies along (sin, cos) of it in the
//!               world's (x, z), so pi is north; left on the Circle Pad turns it up, right down
//!         +0x4c f32 x, +0x50 f32 the height, +0x54 f32 z: the player, the camera's target
//! model   +0x00 its class's vtable: a mirage spot `BMHikyo`, a flock `BMEncount`, the red beacon
//!               over a mirage spot `BMLegendPoint`, the ship `BMShip`
//!         +0x04 -> the next, +0x08 -> the one before; +0x20 f32 x, height, z
//! ```
//!
//! The Circle Pad's up flies forward and down back, left and right turn on the spot; X and Y pitch
//! Latios up and down, which changes the height while it flies, and the D-pad and B did nothing seen
//! (omega_ruby_aeon-flute.ctm, and a session of 2026-10-05 14:18).
//!
//! The places are the ROM's (`a/2/5/2`): one file packed in a two-letter archive, "SLAN", a header
//! then 117 rows of 0x34 bytes, a rectangle each, a place several where it is not a rectangle:
//!
//! ```text
//! +0x00 u32 the zone, then three more of the same place or 0xffffffff
//! +0x10 u32 the event flag set once the player has been there, or 0xffffffff for none
//! +0x14 u32 a mirage spot's number, 0xffffffff for any other place
//! +0x18 u32 0 for a place always there; 1 for the mirage spots a flag opens, 2 to 5 for the
//!           mirage forests, caves, islands and mountains, one a day, and 6 Crescent Isle
//! +0x1c u32 the colour the place is drawn in on a debug map: white a town, orange a route
//!           over land, cyan one over the sea, tomato anything else
//! +0x20 u32 0 for a row the game skips
//! +0x24 f32 x, z, width and depth, as fractions of the world's `BOUNDS`
//! ```
//!
//! The game shows a row where its flag is set and, for a mirage spot of kinds 2 to 6, where its
//! number is today's or one the save was given (`DllSkyTrip`'s `0x007231dc`); the places' byte a
//! row is that answer. It says the place under the player as screen text, with "Land" beside it
//! where the player can land, and it names the place a little ahead of the player rather than under
//! them: the first row shown whose rectangle holds the point `LOOKUP` ahead.

use std::cell::{Cell as Slot, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::discovery::{HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::core::pathfind::{AHEAD, BEHIND, LEFT, RIGHT, way};
use crate::core::rom::lz11;
use crate::core::rtti::ClassNames;
use crate::core::speech::{Category, Speech};
use crate::core::words::own;
use crate::games::common::field::ground::NO_ZONE;
use crate::games::common::field::things::{
    APP_SOURCE, Field, Interactable, POKEMON, Steer, place_source, source_number,
};
use crate::games::common::garc::garc_files;
use crate::games::gen6::apps::LOADED_CODE;
use crate::games::gen6::game::OwnApp;
use crate::games::gen6::rom::archives::GameText;
use crate::py::{self, Error, Kind, Log, Result};

/// The sky's map of places: the one file of its archive, "SLAN" in a two-letter archive of one
/// (spike 166). Its neighbours `a/2/5/1`, `a/2/5/3` and `a/2/5/4` are the sky's models and
/// textures, the files the sky's process loads.
pub const ARCHIVE: &str = "a/2/5/2";

pub const TOWNS: &str = "towns";
pub const ROUTES: &str = "routes";
pub const LANDMARKS: &str = "landmarks";
/// The places on the sky's map: its towns and cities, its routes over land and sea, everything
/// else always there, caves and islands, as the game's own table groups them, and the mirage spots
/// it shows today (spike 166). Nothing on the field is in them.
pub const MIRAGE: &str = "mirage spots";
pub const SKY: [&str; 4] = [TOWNS, ROUTES, LANDMARKS, MIRAGE];

pub const APP: &str = "N3app7skyTrip8MainProcE";
pub const CAMERA: i64 = 0xB4;
pub const HEADING: i64 = 0x0C;
pub const X: i64 = 0x4C;
pub const HEIGHT: i64 = 0x50;
pub const Z: i64 = 0x54;
pub const CAMERA_READ: i64 = 0x58;
pub const PLACES: i64 = 0x154;
pub const SHOWN: i64 = 0x40;
pub const MANAGER: i64 = 0x160;
pub const LAST_MODEL: i64 = 0xA0;
pub const NEXT: i64 = 0x04;
pub const PREVIOUS: i64 = 0x08;
pub const WHERE: i64 = 0x20;
pub const FLOCK_CLASS: &str = "N3app7skyTrip9BMEncountE";
/// More models than the sky builds: a mirage spot, two flocks, three beacons and the ship were
/// seen.
pub const MODELS: usize = 64;
/// A flock of Pokémon flying over Hoenn, which the player flies into to battle one: pa3ds's word,
/// since the game draws the birds and never names them.
pub const FLOCK: &str = "flock";

pub const MAGIC: &[u8] = b"SLAN";
pub const ROWS: i64 = 0x28;
pub const ROW: i64 = 0x34;
pub const ROW_KIND: i64 = 0x18;
pub const ROW_COLOUR: i64 = 0x1C;
pub const ROW_RECT: i64 = 0x24;
pub const ALWAYS: i64 = 0;
/// A row's debug colour -> the category its place is listed in: the game's own grouping of its
/// places, which the colours are the only record of. A mirage spot is listed as one whatever its
/// colour.
///
/// A dict in Python, read here by `find` on the key: it is never iterated.
pub const COLOURS: [(i64, &str); 4] = [
    (0xFFFFFFFF, TOWNS),
    (0xFFFFA500, ROUTES),
    (0xFF00FFFF, ROUTES),
    (0xFFFF6347, LANDMARKS),
];

/// The world's west and north edges, its width and its depth: the rows' fractions are of this.
/// A literal of `DllSkyTrip`'s; the bottom screen's map, 320 wide, draws the player's arrow at
/// `320 * (x + 14773) / 43698 - 160`, as measured to a tenth of a pixel (spike 166).
pub const BOUNDS: (f64, f64, f64, f64) = (-14773.0, -11408.0, 43698.0, 28482.0);
/// Further from the world's middle than any flight goes: the player can fly past `BOUNDS`.
pub const FAR: f64 = 1e6;
/// How far ahead of the player the game looks for the place it names. Measured only from when
/// the words changed, which fits anything from 1150 to 1350 flying north from Littleroot Town and
/// 1400 to 1600 hovering there while turning (spike 166).
pub const LOOKUP: f64 = 1300.0;
// ponytail: one distance for every way and speed; read the game's own point if a place is said
// "here" well before or after the game says "Land".
/// World units a step of a bearing: flying forward covers about 13 a second (spike 166).
pub const UNIT: f64 = 100.0;
/// How far direction mode looks in the sky, in `UNIT`s each way: a quarter of the map across.
pub const REACH: i64 = 100;
/// Degrees off the nose a thing may be and still be said straight ahead.
pub const STRAIGHT: f64 = 10.0;
pub const TURN_STEP: i64 = 5;
/// Radians the heading may move between two polls and still have stopped turning.
pub const STILL: f64 = 0.005;
pub const COMPASS: [&str; 8] = [
    "north",
    "north-east",
    "east",
    "south-east",
    "south",
    "south-west",
    "west",
    "north-west",
];

/// One place on the sky's map, as the game names it.
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub name: String,
    pub category: String,
    pub zone: i64,
    /// (west, north, east, south) in world units, one a row of the table.
    pub rects: Vec<(f64, f64, f64, f64)>,
    /// Each rectangle's row of the table.
    pub rows: Vec<i64>,
}

impl Place {
    /// `Place(name, category, zone, rects)`: `rows` left at its default, `()`.
    pub fn new(name: &str, category: &str, zone: i64, rects: Vec<(f64, f64, f64, f64)>) -> Self {
        Place {
            name: name.to_string(),
            category: category.to_string(),
            zone,
            rects,
            rows: Vec::new(),
        }
    }

    /// This place as far as the game shows it, or None where it shows none of it.
    pub fn only(&self, shown: &HashSet<i64>) -> Result<Option<Place>> {
        // `zip(strict=True)` raises when one of the two ends first.
        if self.rows.len() != self.rects.len() {
            return Err(Error::value(if self.rows.len() < self.rects.len() {
                "zip() argument 2 is longer than argument 1"
            } else {
                "zip() argument 2 is shorter than argument 1"
            }));
        }
        let kept: Vec<(i64, (f64, f64, f64, f64))> = self
            .rows
            .iter()
            .zip(self.rects.iter())
            .filter(|(row, _rect)| shown.contains(row))
            .map(|(row, rect)| (*row, *rect))
            .collect();
        if kept.is_empty() {
            return Ok(None);
        }
        Ok(Some(Place {
            name: self.name.clone(),
            category: self.category.clone(),
            zone: self.zone,
            rects: kept.iter().map(|(_row, rect)| *rect).collect(),
            rows: kept.iter().map(|(row, _rect)| *row).collect(),
        }))
    }
}

/// Where the player is over Hoenn, and which way they face.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flight {
    pub x: f64,
    pub z: f64,
    pub heading: f64,
    pub height: f64,
}

impl Flight {
    pub fn ahead(&self) -> (f64, f64) {
        (self.heading.sin(), self.heading.cos())
    }
}

/// A flock of Pokémon in the sky, where it flies now.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flock {
    pub model: i64,
    pub x: f64,
    pub height: f64,
    pub z: f64,
}

/// Every place on the sky's map, merged by what the game calls it, in the table's order; none from
/// a file that is not the table.
pub fn places(packed: &[u8], place: &dyn Fn(i64) -> Option<String>) -> Result<Vec<Place>> {
    let data = if py::slice(packed, 0, 1) == [0x11] {
        lz11(packed)?
    } else {
        packed.to_vec()
    };
    if py::slice(&data, 0, 2) != b"TM" || data.len() < 12 {
        return Ok(Vec::new());
    }
    let (start, end) = (u32(&data, 4)?, u32(&data, 8)?);
    let data = py::slice(&data, start, end);
    if py::slice(data, 0, 4) != MAGIC || (data.len() as i64) < ROWS {
        return Ok(Vec::new());
    }
    let count = u32(data, 8)?;
    if ROWS + count * ROW > data.len() as i64 {
        return Ok(Vec::new());
    }
    let (west, north, wide, deep) = BOUNDS;
    let mut merged: IndexMap<String, Place> = IndexMap::new();
    for row in 0..count {
        let at = ROWS + row * ROW;
        let zone = u32(data, at)?;
        let mirage = u32(data, at + ROW_KIND)? != ALWAYS;
        let category = if mirage {
            Some(MIRAGE)
        } else {
            let colour = u32(data, at + ROW_COLOUR)?;
            COLOURS
                .iter()
                .find(|(known, _category)| *known == colour)
                .map(|(_colour, category)| *category)
        };
        let name = place(zone);
        let (Some(category), Some(name)) = (category, name) else {
            continue;
        };
        let (u, v, w, h) = (
            f32(data, at + ROW_RECT)?,
            f32(data, at + ROW_RECT + 4)?,
            f32(data, at + ROW_RECT + 8)?,
            f32(data, at + ROW_RECT + 12)?,
        );
        let rect = (
            west + u * wide,
            north + v * deep,
            west + (u + w) * wide,
            north + (v + h) * deep,
        );
        let made = match merged.get(&name) {
            None => Place {
                name: name.clone(),
                category: category.to_string(),
                zone,
                rects: vec![rect],
                rows: vec![row],
            },
            Some(was) => Place {
                name: name.clone(),
                category: was.category.clone(),
                zone: was.zone,
                rects: was.rects.iter().copied().chain([rect]).collect(),
                rows: was.rows.iter().copied().chain([row]).collect(),
            },
        };
        merged.insert(name, made);
    }
    Ok(merged.into_values().collect())
}

/// The player's flight, read through the sky's process, or None before its camera is built or
/// where what is read is no place over the map.
pub fn read(memory: &dyn Memory, process: i64) -> Result<Option<Flight>> {
    let camera = u32(&memory.read(process + CAMERA, 4)?, 0)?;
    if !within(HEAP_SPAN, camera) {
        return Ok(None);
    }
    let head = memory.read(camera, CAMERA_READ)?;
    let flight = Flight {
        x: f32(&head, X)?,
        z: f32(&head, Z)?,
        heading: f32(&head, HEADING)?,
        height: f32(&head, HEIGHT)?,
    };
    // The camera reads all zeros while Latios climbs out of the town, before it is placed.
    if (flight.x == 0.0 && flight.z == 0.0)
        || ![flight.x, flight.z, flight.heading]
            .iter()
            .all(|value| value.is_finite() && value.abs() < FAR)
    {
        return Ok(None);
    }
    Ok(Some(flight))
}

/// The rows of the table the game shows today, or None before it has said.
pub fn shown(memory: &dyn Memory, process: i64, rows: i64) -> Result<Option<HashSet<i64>>> {
    let record = u32(&memory.read(process + PLACES, 4)?, 0)?;
    if !within(HEAP_SPAN, record) {
        return Ok(None);
    }
    let flags = u32(&memory.read(record + SHOWN, 4)?, 0)?;
    if !within(HEAP_SPAN, flags) {
        return Ok(None);
    }
    Ok(Some(
        memory
            .read(flags, rows)?
            .iter()
            .enumerate()
            .filter(|(_row, byte)| **byte != 0)
            .map(|(row, _byte)| row as i64)
            .collect(),
    ))
}

/// The flocks flying now, from the manager's models, walked both ways from the last built.
pub fn flocks(memory: &dyn Memory, process: i64, names: &ClassNames) -> Result<Vec<Flock>> {
    let manager = u32(&memory.read(process + MANAGER, 4)?, 0)?;
    if !within(HEAP_SPAN, manager) {
        return Ok(Vec::new());
    }
    let last = u32(&memory.read(manager + LAST_MODEL, 4)?, 0)?;
    if !within(HEAP_SPAN, last) {
        return Ok(Vec::new());
    }
    let mut seen: Vec<i64> = vec![last];
    for link in [PREVIOUS, NEXT] {
        let mut model = u32(&memory.read(last + link, 4)?, 0)?;
        while within(HEAP_SPAN, model) && !seen.contains(&model) && seen.len() < MODELS {
            seen.push(model);
            model = u32(&memory.read(model + link, 4)?, 0)?;
        }
    }
    let mut out: Vec<Flock> = Vec::new();
    for model in seen {
        if names.of_object(model)?.as_deref() == Some(FLOCK_CLASS) {
            let at = memory.read(model + WHERE, 12)?;
            let (x, height, z) = (f32(&at, 0)?, f32(&at, 4)?, f32(&at, 8)?);
            out.push(Flock {
                model,
                x,
                height,
                z,
            });
        }
    }
    Ok(out)
}

/// Every place the game shows, each at its nearest to where the game looks, and every flock, from
/// the player, since flying into one is what battles it: in the player's own frame, north ahead,
/// so a bearing is said "32 ahead 8 left" (`Field.heading`), and P says the turn to face it
/// (`turn`, `chase`). Without the game's word on what it shows, every place always there.
pub fn listed(
    flight: &Flight,
    known: &[Place],
    today: Option<&HashSet<i64>>,
    flying: &[Flock],
) -> Result<Field> {
    let (look_x, look_z) = look(flight, 0.0);
    let mut items: Vec<Interactable> = Vec::new();
    let mut places: IndexMap<i64, Place> = IndexMap::new();
    for (number, place) in known.iter().enumerate() {
        let number = number as i64;
        let kept = match today {
            Some(today) => place.only(today)?,
            None if place.category != MIRAGE => Some(place.clone()),
            None => None,
        };
        let Some(kept) = kept else {
            continue;
        };
        let (near_x, near_z) = nearest(&kept, look_x, look_z)?;
        let at = cell(flight, near_x - look_x, near_z - look_z)?;
        items.push(Interactable {
            code: kept.zone,
            source: place_source(APP_SOURCE, number)?,
            ..Interactable::new(&kept.category, &kept.name, at)
        });
        places.insert(number, kept);
    }
    let birds: IndexMap<i64, Flock> = flying.iter().map(|flock| (flock.model, *flock)).collect();
    for flock in flying {
        let at = cell(flight, flock.x - flight.x, flock.z - flight.z)?;
        items.push(Interactable {
            source: place_source(APP_SOURCE, flock.model)?,
            ..Interactable::new(POKEMON, FLOCK, at)
        });
    }
    let flight = *flight;
    Ok(Field {
        heading: Some("north"),
        reach: Some(REACH),
        steer: Some(Steer(Rc::new(move |item: &Interactable| {
            steer(&flight, &places, &birds, source_number(item.source))
        }))),
        interactables: items,
        ..Field::new(NO_ZONE, (0, 0))
    })
}

/// Which way to turn to face a place and how far to fly then: "turn left 40 degrees, 33 ahead",
/// "33 ahead" where it is nearly straight on, "here" where the game looks into it, and the turn
/// alone where turning on the spot brings it there, since that carries the point the game looks at
/// round with the nose.
pub fn turn(flight: &Flight, place: &Place) -> Result<String> {
    for step in (0..181).step_by(TURN_STEP as usize) {
        let angles: Vec<i64> = if step != 0 {
            vec![step, -step]
        } else {
            vec![0]
        };
        for angle in angles {
            let (look_x, look_z) = look(flight, angle as f64);
            if inside(place, look_x, look_z)? {
                return if angle != 0 {
                    turned(angle as f64)
                } else {
                    Ok(own("here", &[]))
                };
            }
        }
    }
    let (near_x, near_z) = nearest(place, flight.x, flight.z)?;
    let (ahead, right) = frame(flight, near_x - flight.x, near_z - flight.z);
    let angle = degrees(right.atan2(ahead));
    let away = py::round((py::hypot(ahead, right) - LOOKUP) / UNIT)?;
    let mut said: Vec<String> = if away != 0 {
        vec![own(
            "steps",
            &[
                ("count", &away.abs()),
                ("way", &way(if away > 0 { AHEAD } else { BEHIND })),
            ],
        )]
    } else {
        Vec::new()
    };
    if angle.abs() > STRAIGHT {
        said.insert(0, turned(angle)?);
    }
    let said = said.join(", ");
    Ok(if said.is_empty() {
        own("here", &[])
    } else {
        said
    })
}

/// Which way to turn to face a flock, how far it is then, and how far above or below the player it
/// flies: "turn right 30 degrees, 12 ahead, 2 below".
pub fn chase(flight: &Flight, flock: &Flock) -> Result<String> {
    let (ahead, right) = frame(flight, flock.x - flight.x, flock.z - flight.z);
    let angle = degrees(right.atan2(ahead));
    let mut said = vec![own(
        "steps",
        &[
            ("count", &py::round(py::hypot(ahead, right) / UNIT)?),
            ("way", &way(AHEAD)),
        ],
    )];
    if angle.abs() > STRAIGHT {
        said.insert(0, turned(angle)?);
    }
    let rise = py::round((flock.height - flight.height) / UNIT)?;
    if rise != 0 {
        said.push(own(
            "steps",
            &[
                ("count", &rise.abs()),
                ("way", &way(if rise > 0 { "above" } else { "below" })),
            ],
        ));
    }
    Ok(said.join(", "))
}

/// The compass point the player faces, of eight.
pub fn facing(heading: f64) -> Result<&'static str> {
    let clockwise = py::modulo_f(degrees(std::f64::consts::PI - heading), 360.0);
    Ok(COMPASS[py::modulo(py::round(clockwise / 45.0)?, COMPASS.len() as i64) as usize])
}

/// Soaring on Latios, an app of ORAS's own for the gen 6 profile to run (`gen6.game.OwnApp`): the
/// sky's places and flocks for the navigation keys, and the way the player faces, said once a turn
/// stops on another, as the bottom screen's arrow shows it.
pub struct Soaring {
    memory: Rc<dyn Memory>,
    known: Vec<Place>,
    rows: i64,
    speech: Rc<Speech>,
    log: Log,
    /// The heading the player soared with at the last poll, and the compass point last said for
    /// it: said again once a turn stops on another.
    heading: Slot<Option<f64>>,
    facing: RefCell<Option<&'static str>>,
}

impl Soaring {
    pub fn new(memory: Rc<dyn Memory>, known: Vec<Place>, speech: Rc<Speech>, log: Log) -> Self {
        let rows = 1 + known
            .iter()
            .flat_map(|place| place.rows.iter().copied())
            .max()
            .unwrap_or(-1);
        Soaring {
            memory,
            known,
            rows,
            speech,
            log,
            heading: Slot::new(None),
            facing: RefCell::new(None),
        }
    }

    /// Soaring where the ROM's map of places is read, or None: without it, the field's lists are
    /// walked as they were. The factory `Game.apps` holds.
    pub fn of(
        memory: Rc<dyn Memory>,
        text: &GameText,
        speech: Rc<Speech>,
        log: Log,
    ) -> Result<Option<Rc<dyn OwnApp>>> {
        let Some(packed) = text.files.get(ARCHIVE) else {
            return Ok(None);
        };
        let known = (|| -> Result<Vec<Place>> {
            let files = garc_files(packed)?;
            let Some(first) = files.first() else {
                return Err(Error::index("list index out of range"));
            };
            places(first, &|zone| text.zone_place(zone))
        })();
        let known = match known {
            Ok(known) => known,
            Err(error)
                if error.is(Kind::RomError)
                    || error.is(Kind::IndexError)
                    || error.is(Kind::StructError) =>
            {
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        if known.is_empty() {
            return Ok(None);
        }
        Ok(Some(Rc::new(Soaring::new(memory, known, speech, log))))
    }
}

impl OwnApp for Soaring {
    fn app(&self) -> &'static str {
        APP
    }

    fn name(&self) -> &'static str {
        "sky"
    }

    fn categories(&self) -> Vec<&'static str> {
        SKY.to_vec()
    }

    fn started(&self) {
        // Each flight says the way the player faces as it settles, the first included.
        self.heading.set(None);
        *self.facing.borrow_mut() = None;
    }

    fn listed(&self, process: i64) -> Result<Option<Field>> {
        let Some(flight) = read(&*self.memory, process)? else {
            return Ok(None);
        };
        // A module's classes are named afresh: the memory is the next module's once it unloads.
        let names = ClassNames::new(self.memory.clone(), LOADED_CODE);
        let today = shown(&*self.memory, process, self.rows)?;
        let flying = flocks(&*self.memory, process, &names)?;
        Ok(Some(listed(&flight, &self.known, today.as_ref(), &flying)?))
    }

    /// One small read a field poll while soaring.
    fn poll(&self, process: i64) -> Result<()> {
        let Some(flight) = read(&*self.memory, process)? else {
            return Ok(());
        };
        let was = self.heading.replace(Some(flight.heading));
        let Some(was) = was else {
            return Ok(());
        };
        if (flight.heading - was).abs() > STILL {
            return Ok(());
        }
        let word = facing(flight.heading)?;
        if *self.facing.borrow() != Some(word) {
            *self.facing.borrow_mut() = Some(word);
            (self.log)(&format!(
                "sky: facing {word} at ({}, {}) height {}",
                py::fixed(flight.x, 0),
                py::fixed(flight.z, 0),
                py::fixed(flight.height, 0)
            ));
            self.speech.say(
                &own("sky-facing", &[("way", &way(word))]),
                Category::Guidance,
            );
        }
        Ok(())
    }
}

/// P in the sky, for the place or the flock listed under this number.
fn steer(
    flight: &Flight,
    places: &IndexMap<i64, Place>,
    birds: &IndexMap<i64, Flock>,
    number: i64,
) -> Result<String> {
    if let Some(flock) = birds.get(&number) {
        return chase(flight, flock);
    }
    match places.get(&number) {
        Some(place) => turn(flight, place),
        None => Err(Error::key(number.to_string())),
    }
}

/// A world offset as a cell of the player's frame: x right, z behind, in `UNIT`s.
fn cell(flight: &Flight, dx: f64, dz: f64) -> Result<(i64, i64)> {
    let (ahead, right) = frame(flight, dx, dz);
    Ok((py::round(right / UNIT)?, py::round(-ahead / UNIT)?))
}

/// A turn this many degrees clockwise, said.
fn turned(angle: f64) -> Result<String> {
    let degrees = py::round(angle.abs() / TURN_STEP as f64)? * TURN_STEP;
    Ok(own(
        "sky-turn",
        &[
            ("way", &way(if angle > 0.0 { RIGHT } else { LEFT })),
            ("degrees", &degrees),
        ],
    ))
}

/// Where the game looks for the place it names, with the player turned this many degrees
/// clockwise.
fn look(flight: &Flight, turned: f64) -> (f64, f64) {
    let heading = flight.heading - radians(turned);
    (
        flight.x + LOOKUP * heading.sin(),
        flight.z + LOOKUP * heading.cos(),
    )
}

fn inside(place: &Place, x: f64, z: f64) -> Result<bool> {
    Ok(nearest(place, x, z)? == (x, z))
}

/// The rectangle of a place nearest (x, z).
fn part(place: &Place, x: f64, z: f64) -> Result<(f64, f64, f64, f64)> {
    // `min(place.rects, key=...)`: the first of the least, a NaN key never winning.
    let key = |rect: &(f64, f64, f64, f64)| {
        py::hypot(
            min(max(x, rect.0), rect.2) - x,
            min(max(z, rect.1), rect.3) - z,
        )
    };
    let Some(first) = place.rects.first() else {
        return Err(Error::value("min() iterable argument is empty"));
    };
    let (mut best, mut least) = (*first, key(first));
    for rect in &place.rects[1..] {
        let value = key(rect);
        if value < least {
            (best, least) = (*rect, value);
        }
    }
    Ok(best)
}

/// The point of a place nearest (x, z): (x, z) itself inside it.
fn nearest(place: &Place, x: f64, z: f64) -> Result<(f64, f64)> {
    let (west, north, east, south) = part(place, x, z)?;
    Ok((min(max(x, west), east), min(max(z, north), south)))
}

/// A world offset as (ahead, right) of the player: right is ahead turned a quarter clockwise.
fn frame(flight: &Flight, dx: f64, dz: f64) -> (f64, f64) {
    let (ahead_x, ahead_z) = flight.ahead();
    (dx * ahead_x + dz * ahead_z, -dx * ahead_z + dz * ahead_x)
}

/// `math.degrees`, which CPython computes as `x * (180 / pi)`.
fn degrees(x: f64) -> f64 {
    x * (180.0 / std::f64::consts::PI)
}

/// `math.radians`, which CPython computes as `x * (pi / 180)`.
fn radians(x: f64) -> f64 {
    x * (std::f64::consts::PI / 180.0)
}

/// Python's `min(a, b)` of floats: `a` unless `b` is strictly less, so a NaN stays where Python
/// puts it.
fn min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Python's `max(a, b)` of floats: `a` unless `b` is strictly greater.
fn max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}
