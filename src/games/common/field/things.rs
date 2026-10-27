//! What is on a map worth walking to, where the player is among it, and how each thing is told
//! apart from one listing to the next.

use std::fmt;
use std::rc::Rc;

use crate::core::pathfind::{self, Cell, EAST, NORTH, SOUTH, WEST, way};
use crate::core::words::{own, said};
use crate::py::{Error, Result};

/// `{n:#x}`, which writes a negative number as Python does, with its sign before the prefix.
fn hex(n: i64) -> String {
    let sign = if n < 0 { "-" } else { "" };
    format!("{sign}{:#x}", n.unsigned_abs())
}

/// The bits of a place's `source` that say which table or encoding the rest of it is in. Places
/// are keyed by (map, source), and each kind numbers its own from zero — a sign by its index in the
/// furniture table, a warp by its index in the warp table, a use tile by its first cell, an edge by
/// the zone it leads to — so without a tag, two places of different kinds on one map share a name
/// whenever their numbers happen to coincide: sign 1 and door 1 did until 2026-09-23 (renaming the
/// sign renamed the door, and the ROM's name for the door was said for the sign), and an edge's
/// `zone << 16` reached the bit a use tile was marked with, `1 << 24`, for any zone from `0x100` up
/// (the same day). The tag sits above 32 bits because every number under it is built from u16s the
/// game holds — a zone shifted by 16 at most — so nothing read from the game can reach it. Five bits
/// since the seventeenth kind, `CLIMB_SOURCE` (2026-10-24).
pub const SOURCE_KIND: i64 = 0x1F << 32;

/// A furniture record's source is its bare index: the tag the first names were keyed under.
pub const FURNITURE_SOURCE: i64 = 0 << 32;

/// A warp's source: `WARP_SOURCE | index` in the warp table.
pub const WARP_SOURCE: i64 = 1 << 32;

/// A use tile's source, or a hole's a big boulder drops into: `TILE_SOURCE | x << 12 | z`, its
/// first cell.
pub const TILE_SOURCE: i64 = 2 << 32;

/// An edge's source: `EDGE_SOURCE | zone << 16 | gap`, the zone it leads to and the gap's place
/// along the border.
pub const EDGE_SOURCE: i64 = 3 << 32;

/// A berry spot's source: `BERRY_SOURCE | index` in the game's table of them (spike 141).
pub const BERRY_SOURCE: i64 = 4 << 32;

/// A trigger's source, one a game knows as something to walk to: `TRIGGER_SOURCE | index` in the
/// zone's table of triggers (spike 146).
pub const TRIGGER_SOURCE: i64 = 5 << 32;

/// A Secret Base spot's source: `SECRET_BASE_SOURCE | number`, the spot's number in the game's
/// table of them (spike 154).
pub const SECRET_BASE_SOURCE: i64 = 6 << 32;

/// An ash pile's source: `ASH_SOURCE | index` in the zone's list of them (spike 155).
pub const ASH_SOURCE: i64 = 7 << 32;

/// A slope's or a rail end's source: `BIKE_SOURCE | x << 12 | z`, its first cell.
pub const BIKE_SOURCE: i64 = 8 << 32;

/// A dive spot's source: `DIVE_SOURCE | x << 12 | z`, its patch's first cell.
pub const DIVE_SOURCE: i64 = 9 << 32;

/// A stair's source between two floors of a chunk: `STAIRS_SOURCE | x << 12 | z`, its first
/// cell.
pub const STAIRS_SOURCE: i64 = 10 << 32;

/// A thing an app of one game's own lists, by its number there: ORAS's soaring's places and flocks
/// (`oras.field.sky`, spike 166). The game names them, so they take no name of ours.
pub const APP_SOURCE: i64 = 11 << 32;

/// A decoration set down in a Secret Base: `DECORATION_SOURCE | x << 12 | z`, its first cell, a
/// kind of its own so a name given the crumbled cliff is not the Stairs' over it.
pub const DECORATION_SOURCE: i64 = 12 << 32;

/// One move model's own name, apart from its sprite's: `MODEL_SOURCE | index`, the record it was
/// built from, under that record's zone (`Interactable.record`). A big boulder keeps its record
/// however far it is pushed, and across a save (spike 202).
pub const MODEL_SOURCE: i64 = 13 << 32;

/// A current's source: `CURRENT_SOURCE | x << 12 | z`, the first cell of the currents that let the
/// player go round one place (spike 208).
pub const CURRENT_SOURCE: i64 = 14 << 32;

/// A patch of flat ground's source, where a ridden Pokémon is got on and off: `RIDE_SOURCE | x <<
/// 12 | z`, its first cell (spike 238).
pub const RIDE_SOURCE: i64 = 15 << 32;

/// A run of a climbing wall's blue spots: `CLIMB_SOURCE | x << 12 | z`, its first cell (spike 245).
pub const CLIMB_SOURCE: i64 = 16 << 32;

/// A hidden item's source: `HIDDEN_SOURCE | number`, its number in the executable's table (spike
/// 249).
pub const HIDDEN_SOURCE: i64 = 17 << 32;

/// A vine's end: `VINE_SOURCE | number`, its record's script in the zone's. A cell holds an end
/// on each of several levels, so its cell would not tell them apart (spike 250).
pub const VINE_SOURCE: i64 = 18 << 32;

/// Where a street ends on open ground: `OPENING_SOURCE | point`, the id of the point or joint it
/// ends at, a map's own (spike A).
pub const OPENING_SOURCE: i64 = 19 << 32;

/// A place's `source` from its kind's tag and its own number. The number must leave the tag's
/// bits clear, and no u16 the game holds can reach them, so one that does is a new encoding
/// written wider than the tag allows for: a programmer's error, refused here rather than keyed as
/// another kind's place.
pub fn place_source(kind: i64, payload: i64) -> Result<i64> {
    if payload & SOURCE_KIND != 0 {
        return Err(Error::value(format!(
            "place payload {} reaches the kind bits",
            hex(payload)
        )));
    }
    Ok(kind | payload)
}

/// The name of the one move model built from record `index` of `zone`'s table, as
/// `Interactable.naming` says one.
pub fn own_name(zone: i64, index: i64) -> Result<Naming> {
    Ok((PLACE, zone, place_source(MODEL_SOURCE, index)?))
}

/// Which kind of place a source is: one of the `*_SOURCE` tags.
pub fn source_kind(source: i64) -> i64 {
    source & SOURCE_KIND
}

/// A place's own number under its tag: a warp's index, an edge's `zone << 16 | gap`.
pub fn source_number(source: i64) -> i64 {
    source & !SOURCE_KIND
}

/// The category that is every category: what `Field.of` gives the whole list for.
pub const ALL: &str = "all";

pub const SPRITE: &str = "sprite";

/// The two name spaces of `pa3ds.core.names`, as `Interactable.naming` chooses between them.
pub const PLACE: &str = "place";

/// Which name a thing has, as `Interactable.naming` says it: (`SPRITE`, character, 0) or
/// (`PLACE`, map, slot).
pub type Naming = (&'static str, i64, i64);

pub const POKEMON: &str = "pokémon";

/// The soil berries are planted in, a category of its own (the user, 2026-09-29).
pub const BERRIES: &str = "berries";

/// Route 113's ash piles, a category of its own (the user, 2026-10-02).
pub const ASH: &str = "ash";

/// The ground only a bike gets up or across, the Mach Bike's slopes and the Acro Bike's rails and
/// bumpy slopes, a category of its own: the game names none of them, and the player could not find
/// them to record them (the user, 2026-10-03).
pub const BIKE: &str = "bike";

/// What stands in the way until a move clears it, a category of its own: big boulders, bushes,
/// cracked boulders, the holes Strength's boulders drop into, and Mauville Gym's barriers. Under
/// signs they outnumbered the signs in Seafloor Cavern's rooms (Orion, 2026-10-13).
pub const OBSTACLES: &str = "obstacles";

/// The sea's currents, a category of their own, each by where it lets the player go, as a
/// teleporter is by where it comes out (Orion, 2026-10-13).
pub const CURRENTS_CATEGORY: &str = "currents";

/// The flat ground a ridden Pokémon is got on and off on, a category of its own: "not sure when you
/// can get off rhyhorn so that prob need its own category" (Khai, 2026-10-22).
pub const RIDING: &str = "riding";

/// The blue spots climbing walls are got on at and the ledges they are got off onto, and the ends
/// of vines, a category of its own: "mark its entry point and exit point maybe? in a category"
/// (Khai, 2026-10-24).
pub const CLIMBING: &str = "climbing";

/// In the order they are walked and logged.
pub const CATEGORIES: [&str; 12] = [
    "exits",
    "signs",
    OBSTACLES,
    "people",
    "items",
    BERRIES,
    ASH,
    BIKE,
    RIDING,
    CLIMBING,
    CURRENTS_CATEGORY,
    POKEMON,
];

/// One thing worth walking to, in the player's cells.
#[derive(Debug, Clone, PartialEq)]
pub struct Interactable {
    /// One of `CATEGORIES`: `exits`, `signs`, `obstacles`, `people`, `items`, `berries`, `ash`,
    /// `bike`, `riding`, `climbing`, `currents` or `pokémon`, or in the sky `towns`, `routes`,
    /// `landmarks` or `mirage spots`.
    pub category: String,
    /// What it is, in pa3ds's own words: `door`, `sign`, `PC`, `person`, `item ball`, `edge`,
    /// `berry spot`, `ash pile`, `hidden Pokémon`, `barrier`, `switch`, `passage`, `trapdoor`,
    /// `geyser`, `lift`, `teleporter`, `dive spot`, `stairs`, `stairs up`, `stairs down`, `ladder
    /// up`, `ladder down`, `entrance`, `slope`, `bumpy slope`, `rail`, `current`, `blue spot`,
    /// `wall ledge`, `vine up`, `vine down`, `swinging vine`, `open ground`, a `flock` in the sky;
    /// or in the sky, the game's own name for a place.
    pub kind: String,
    pub cell: (i64, i64),
    pub size: (i64, i64),
    /// For an exit, the zone it leads to.
    pub zone: Option<i64>,
    /// The record's model code or script, a use tile's kind, or a hidden Pokémon's species, for
    /// the log and for names later.
    pub code: i64,
    /// What this was read from: a model's address for a move model, with the zone and index of the
    /// record it was built from above it in gen 6, since a slot is filled with the next zone's
    /// people as the player walks into it (spike 131); and for a place its kind's tag
    /// (`SOURCE_KIND`) over its own number — a furniture record's index, a warp's index, an edge's
    /// zone with the gap's place along the border after it (`zone << 16 | gap`), a use tile's first
    /// cell (`x << 12 | z`). With the kind, it tells the same thing apart from one press to the
    /// next while the list is re-sorted around a moving player.
    pub source: i64,
    /// Whether this is a move model — someone or something standing on the map now — rather than
    /// a record in the zone's own table. It decides which name a rename gives it (`naming`), and it
    /// is why `source` is an address here and an index there.
    pub move_model: bool,
    /// For a record of a zone's table, that zone, where it is known: the one the player is in,
    /// or on Lumiose's streets another zone of the same street (spike 107). A place is named by
    /// it, and told apart by it from the same slot of the other zone.
    pub home: Option<i64>,
    /// How high in the world it stands, where the game says: a move model's, a warp's, a
    /// trigger's, a stair's top. It says which floor of a chunk with a deck the thing is on:
    /// Sootopolis Gym's lower floor lies under its rooms, and P led the player to a Beauty down
    /// there by the room over her head (spike 167).
    pub height: Option<f64>,
    /// For an exit that comes out in the zone it is in, the cell it comes out at: a hideout's
    /// teleporter pads, most of which lead to another pad of their own room (2026-10-06). For a
    /// current, where it lets the player go from `cell`.
    pub arrival: Option<(i64, i64)>,
    /// For a move model built from a record of a zone's table, that zone and the record's index:
    /// which one it is, wherever it walks or is pushed (spike 202). None where it is not read: Sun's
    /// models.
    pub record: Option<(i64, i64)>,
    /// For a big boulder, where its record starts it, as `cell` says where it is: where the room
    /// had it before anybody pushed it (spike 202). None for anything else.
    pub start: Option<(i64, i64)>,
    /// For a big boulder, the ways it moves when pushed, each with whether that push drops it
    /// into a hole (`ground.pushes`); none for one that cannot be moved. None for anything else.
    pub pushes: Option<Vec<(&'static str, bool)>>,
    /// For a warp walked into one way, that way: it is reached from its cells on the side it is
    /// walked in from, and from no other. Ever Grande City's way into Victory Road's lower floor
    /// is a cave mouth over the water, walked into north, and P took the player to the ground over
    /// it, 45 higher, "passage, 1 south" (movie45, spike 213). None for anything else.
    pub way_in: Option<&'static str>,
    /// The cells it covers where they are not the whole of its box: a warp across a diagonal
    /// street, a row of cells the box round it holds a square of, and a way to it ended a step to
    /// one side (x_route-6-2.ctm, 2026-10-20). Empty for the box.
    pub shape: Vec<Cell>,
    /// Whether it is taken from beside it, facing it, though its cell can be stood in: a hidden
    /// item, which A finds from the cell before it (spike 145).
    pub faced: bool,
}

impl Interactable {
    /// An interactable of one cell with the Python's defaults for the rest, which are set with
    /// `..Interactable::new(...)`.
    pub fn new(category: &str, kind: &str, cell: (i64, i64)) -> Self {
        Interactable {
            category: category.to_string(),
            kind: kind.to_string(),
            cell,
            size: (1, 1),
            zone: None,
            code: 0,
            source: 0,
            move_model: false,
            home: None,
            height: None,
            arrival: None,
            record: None,
            start: None,
            pushes: None,
            way_in: None,
            shape: Vec::new(),
            faced: false,
        }
    }

    pub fn key(&self) -> (String, String, i64, Option<i64>) {
        (
            self.category.clone(),
            self.kind.clone(),
            self.source,
            self.home,
        )
    }

    /// Which kind of place this is, one of the `*_SOURCE` tags; None for a move model, whose
    /// source has its record's index where a place's has its kind.
    pub fn place_kind(&self) -> Option<i64> {
        if self.move_model {
            None
        } else {
            Some(source_kind(self.source))
        }
    }

    /// Which name belongs to this, as (what, map, slot), or None for a thing with no name of
    /// its own.
    ///
    /// A **move model is a character**, so naming the person standing in the Gym names Norman
    /// everywhere, and names him as a speaker too: it is the same number either way
    /// (`mmodel.character`). Everything else on the map is a **place**, a slot on one map, so a
    /// sign named in Petalburg says nothing about a sign in Rustboro.
    ///
    /// The cost of that is a sprite everybody nameless shares: naming "the man in the Mart"
    /// names every townsperson drawn as him. That is already how the shipped table works —
    /// `0x128` is "Woman", Mrs. Birch and 27 zones of others — so it is the behaviour being
    /// kept, not a new one.
    ///
    /// **A hidden Pokémon has no name.** It is a different Pokémon each time and gone in a
    /// minute, its `source` is the record's address rather than a slot in any table, and the
    /// game names the species itself once the DexNav has registered it — so a name here would be
    /// a key that never matches again and would replace something the game says.
    pub fn naming(&self, zone: i64) -> Option<Naming> {
        if self.category == POKEMON && !self.move_model {
            return None;
        }
        // A move model's source has its record's index where a place's has its kind, so it is
        // asked first: one whose index ended in 0xb read as the sky's and went nameless, a
        // Street Thug in Mauville among them (movie42).
        if self.move_model {
            return Some((SPRITE, self.code, 0));
        }
        // What a game's own app lists, the sky's places, the game names itself, and a name of
        // ours never replaces the game's (ADR-0023).
        if self.place_kind() == Some(APP_SOURCE) {
            return None;
        }
        Some((PLACE, self.home.unwrap_or(zone), self.source))
    }

    /// The name of this one alone, ahead of its sprite's, as `naming` says one: a slot on its
    /// record's map. None for anything but a move model with a record. Youngster Calvin is one of
    /// eleven drawn as a Youngster, and one big boulder of eight in a room (Orion, 2026-10-11).
    pub fn own(&self) -> Result<Option<Naming>> {
        self.record
            .map(|(zone, index)| own_name(zone, index))
            .transpose()
    }

    /// Direction and distance from `origin`, as "3 east 2 north", or "here"; for a player
    /// heading one way on a rail, from where they face: "9 ahead 8 left".
    pub fn bearing(&self, origin: (i64, i64), heading: Option<&str>) -> Result<String> {
        if let Some(heading) = heading {
            return pathfind::facing(origin, self.nearest(origin), heading);
        }
        Ok(bearing(origin, self.nearest(origin)))
    }

    /// The cell of this that is closest to `origin`: an exit can be several cells wide.
    pub fn nearest(&self, origin: (i64, i64)) -> (i64, i64) {
        if let Some(&first) = self.shape.iter().min_by_key(|cell| {
            (
                (cell.0 - origin.0).abs() + (cell.1 - origin.1).abs(),
                **cell,
            )
        }) {
            return first;
        }
        let x = origin.0.max(self.cell.0).min(self.cell.0 + self.size.0 - 1);
        let z = origin.1.max(self.cell.1).min(self.cell.1 + self.size.1 - 1);
        (x, z)
    }

    /// Every cell this covers.
    pub fn cells(&self) -> Vec<Cell> {
        if !self.shape.is_empty() {
            return self.shape.clone();
        }
        let mut out = Vec::new();
        for dz in 0..self.size.1 {
            for dx in 0..self.size.0 {
                out.push((self.cell.0 + dx, self.cell.1 + dz));
            }
        }
        out
    }
}

/// Where the player is, and what is around them.
///
/// `Default` is written out, since `walkable` defaults to true.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub zone: i64,
    pub cell: (i64, i64),
    pub interactables: Vec<Interactable>,
    /// Whether the player is on a rail, as on Lumiose's boulevards, where the game keeps the
    /// streets' walls in a map of the rail's own and the chunks mark every cell a wall, the
    /// player's own included (spike 102). With the rails loaded, `cell` and every cell listed are
    /// the corridor's (`Corridor`).
    pub on_rail: bool,
    /// On a rail or under a camera behind the player, the way up walks them, nearest `bearing`,
    /// which is the way their keys work from: up walks the way the camera looks. On a rail down
    /// walks back from it, and left and right turn the player a quarter, the camera following only
    /// while the key is held (spike 105); in Glittering Cave see `turning`.
    pub heading: Option<&'static str>,
    /// On a rail or under a camera behind the player, the way up walks them, in degrees
    /// clockwise from north, the corridor's on a rail: the camera's heading, which on a rail a tap
    /// of left or right leaves anywhere (spikes 105 and 239).
    pub bearing: Option<f64>,
    /// Whether the camera behind the player turns with them, as in Glittering Cave, rather than
    /// being held round, as on Lumiose's rails: left and right turn the player a quarter and down
    /// turns them round, the camera swinging after them, and only up walks (spike 239,
    /// x_glittering_cave). Meaningful only with `heading` set.
    pub turning: bool,
    /// Whether the player is riding the Mach Bike, which rides up a sandy slope (spike 158).
    pub on_bike: bool,
    /// Whether the player is surfing, where a current can carry them (spike 208).
    pub surfing: bool,
    /// How far direction mode looks, in this map's steps each way, where not the field's ten
    /// cells: the sky's places are far apart (`oras.sky.REACH`).
    pub reach: Option<i64>,
    /// While the player is soaring, the places listed in their own frame (`oras.sky`): the turn
    /// to face a thing and how far it is then, which P says, since there is no ground to find a way
    /// over.
    pub steer: Option<Steer>,
    /// Where the player stands in the world, in cells, a cell's centre whole: (20.31, 24.0) in
    /// `cell` (20, 24). The circle pad stops them anywhere, and a step past a wall's corner is
    /// refused to a player off the centre on the corner's side (movie38, spike 202). None where it
    /// is not read: on a rail, and in Sun.
    pub place: Option<(f64, f64)>,
    /// Whether the player can walk now: not while another screen runs over a map still loaded,
    /// where the list is still read but P finds no way (Orion, 2026-10-12).
    pub walkable: bool,
    /// How high the player stands in the world, as `Interactable.height` says a thing does: 18 to
    /// a cell's width in gen 6. None where it is not read: in Sun.
    pub height: Option<f64>,
}

impl Default for Field {
    fn default() -> Self {
        Field {
            zone: 0,
            cell: (0, 0),
            interactables: Vec::new(),
            on_rail: false,
            heading: None,
            bearing: None,
            turning: false,
            on_bike: false,
            surfing: false,
            reach: None,
            steer: None,
            place: None,
            walkable: true,
            height: None,
        }
    }
}

/// `Callable[[Interactable], str]`, a `Field.steer`. It can raise, as the Python's reads a table by
/// a number and rounds floats. Two fields are equal when they hold the same function, as
/// Python compares callables.
#[derive(Clone)]
pub struct Steer(pub Rc<dyn Fn(&Interactable) -> Result<String>>);

impl fmt::Debug for Steer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Steer(..)")
    }
}

impl PartialEq for Steer {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Field {
    /// A field with the Python's defaults for all but where the player is, the rest set with
    /// `..Field::new(...)`.
    pub fn new(zone: i64, cell: (i64, i64)) -> Self {
        Field {
            zone,
            cell,
            ..Field::default()
        }
    }

    /// `of(category)`: nearest first.
    pub fn of(&self, category: &str) -> Vec<Interactable> {
        self.of_with(category, true)
    }

    /// This category's interactables (every one for `ALL`), nearest first, or with `nearest`
    /// false in the order the zone's tables list them, which walking leaves alone.
    pub fn of_with(&self, category: &str, nearest: bool) -> Vec<Interactable> {
        let mut items: Vec<Interactable> = self
            .interactables
            .iter()
            .filter(|item| category == ALL || item.category == category)
            .cloned()
            .collect();
        if nearest {
            items.sort_by_key(|item| distance(self.cell, item.nearest(self.cell)));
        }
        items
    }
}

/// One line a category, for the log: what is there and where, nearest first, what each is
/// `called` when it has a name, and the place an exit `leads` to when that is somewhere else. A
/// replay presses no navigation keys, so this is where it shows which door the game's own words
/// named (spike 86) and where each exit goes.
///
/// The lines are made as they are asked for, as the Python's generator makes them, so a
/// `called` or `leads` that is never reached is never run; one that fails ends the lines.
pub fn describe<'a>(
    r#where: &'a Field,
    called: Option<&'a dyn Fn(&Interactable) -> Result<Option<String>>>,
    leads: Option<&'a dyn Fn(&Interactable) -> Result<Option<String>>>,
) -> impl Iterator<Item = Result<String>> + 'a {
    let mut categories = CATEGORIES.iter();
    let mut failed = false;
    std::iter::from_fn(move || {
        if failed {
            return None;
        }
        for category in categories.by_ref() {
            let items = r#where.of(category);
            if items.is_empty() {
                continue;
            }
            let line = (|| -> Result<String> {
                let mut said: Vec<String> = Vec::new();
                for item in &items {
                    let place = match leads {
                        Some(leads) => leads(item)?,
                        None => None,
                    };
                    let detail = detail(item, place.as_deref());
                    let name = called_name(item, called)?;
                    let bearing = item.bearing(r#where.cell, None)?;
                    let out = out(item, r#where.cell);
                    said.push(format!("{}{detail}{name} {bearing}{out}", item.kind));
                }
                Ok(format!("{category}: {}", said.join("; ")))
            })();
            failed = line.is_err();
            return Some(line);
        }
        None
    })
}

fn called_name(
    item: &Interactable,
    called: Option<&dyn Fn(&Interactable) -> Result<Option<String>>>,
) -> Result<String> {
    let name = match called {
        Some(called) => called(item)?,
        None => None,
    };
    Ok(match name {
        None => String::new(),
        Some(name) => format!(" \"{name}\""),
    })
}

fn out(item: &Interactable, origin: (i64, i64)) -> String {
    match item.arrival {
        None => String::new(),
        Some(arrival) => format!(" out at {}", bearing(origin, arrival)),
    }
}

fn detail(item: &Interactable, place: Option<&str>) -> String {
    if let Some(zone) = item.zone {
        return format!(" to zone {}", hex(zone))
            + &match place {
                None => String::new(),
                Some(place) => format!(" ({place})"),
            };
    }
    if item.category == POKEMON && !item.move_model {
        return format!(" species {}", item.code);
    }
    String::new()
}

/// Direction and distance from `origin` to `cell`, as "3 east 2 north", or "here".
pub fn bearing(origin: (i64, i64), cell: (i64, i64)) -> String {
    let (dx, dz) = (cell.0 - origin.0, cell.1 - origin.1);
    let mut parts: Vec<String> = Vec::new();
    if dx != 0 {
        parts.push(own(
            "steps",
            &[
                ("count", &dx.abs()),
                ("way", &way(if dx > 0 { EAST } else { WEST })),
            ],
        ));
    }
    if dz != 0 {
        parts.push(own(
            "steps",
            &[
                ("count", &dz.abs()),
                ("way", &way(if dz > 0 { SOUTH } else { NORTH })),
            ],
        ));
    }
    if parts.is_empty() {
        own("here", &[])
    } else {
        parts.join(" ")
    }
}

/// What a thing is, as it is said: pa3ds's own word in the player's language,
/// `kind-stairs-up`, or the word itself where it is the game's or the data's already, "PC",
/// "trash can".
pub fn kind_said(kind: &str) -> String {
    said("kind", kind)
}

/// A category of the list as it is said, `category-exits`.
pub fn category_said(category: &str) -> String {
    said("category", category)
}

fn distance(a: (i64, i64), b: (i64, i64)) -> i64 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}
