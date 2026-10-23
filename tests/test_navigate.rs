//! Navigation tests: the keys walking a fake map's list, and what each press says.

use std::cell::{Cell as Slot, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::config::Navigation;
use crate::core::keyfile;
use crate::core::keys::key_name;
use crate::core::names::Names;
use crate::core::pathfind::{self, Cell, nearest};
use crate::core::speech::{Backend, Category, Speech, default_interrupts};
use crate::games::common::field::chunk::{BLOCKED, CHUNK, Chunk, Floor};
use crate::games::common::field::ground::{
    CUT, Ground, IceRoom, Of, ROCK_SMASH, RotatingDoor, STRENGTH, SURF, WATERFALL,
};
use crate::games::common::field::guide::{FieldSource, Guidance};
use crate::games::common::field::keys::{
    CATEGORY_NEXT, CATEGORY_PREVIOUS, COORDINATES, DEFAULT_BINDINGS, DIRECTION, EAST, ENEMY_SIDE,
    ITEM_COORDINATES, ITEM_NEXT, ITEM_PREVIOUS, ITEM_READ, LIST, MODE_TOGGLE, NORTH, OUR_SIDE,
    PATH_NEXT, PATH_PREVIOUS, PATHFIND, PATHFIND_HM, PATHFIND_OFF, RENAME, RENAME_SPEAKER, SOUTH,
    TOUCH, WEST, bound_keys,
};
use crate::games::common::field::navigate::{DoorSign, LeadsTo, Navigator, Options, Sides, Street};
use crate::games::common::field::things::{
    FURNITURE_SOURCE, Field, Interactable, WARP_SOURCE, place_source,
};
use crate::games::common::touch::{ROW_NEXT, ROW_PREVIOUS, TAP};
use crate::games::oras::{self, BIG_BOULDER, BUSH, CRACKED_BOULDER};
use crate::py::{self, Result};

/// Python's `Recorder`: what it was given to say, and whether each was said interrupting. The
/// shared `fake_speech::Recorder` keeps the text alone.
#[derive(Default)]
struct Recorder {
    spoken: RefCell<Vec<(String, bool)>>,
}

impl Backend for Recorder {
    fn speak(&self, text: &str, interrupt: bool) {
        self.spoken.borrow_mut().push((text.to_string(), interrupt));
    }

    fn braille(&self, _text: &str) {}
}

/// Petalburg Woods' ledges, jumped south in omega_ruby_ledges.ctm (spike 126).
const SOUTH_LEDGE: i64 = 0x75020021;
/// Jagged Pass's bumpy slopes, hopped up on the Acro Bike (omega_ruby_bike-jagged2.ctm,
/// spike 159).
const BUMPY: i64 = 0x6F000021;
/// Route 109's sea (omega_ruby_surf.ctm, spike 136).
const SEA: i64 = 0x3D1A0006;
/// Route 122's strip of shore, jumped over getting on the water and off (spike 162).
const SHORE: i64 = 0x40180023;
/// Ever Grande City's waterfall, climbed north (omega_ruby_waterfall.ctm, spike 170).
const FALLS: i64 = 0x3F180003;
/// Jagged Pass's sandy slopes, ridden up north on the Mach Bike (omega_ruby_bike.ctm, spike 158).
const SOUTH_SLOPE: i64 = 0x6C000020;
/// Route 132's current, which carried the player west (movie43, spike 208).
const WEST_CURRENT: i64 = 0x951A0002;
/// A jumping stone in X's Route 8 mud pit, hopped onto two cells at a time (x_route-8-3.ctm,
/// spike 235).
const STONE: i64 = 0x92000021;
/// Where the hops onto the stones start and end, at the pit's edge.
const LAUNCH: i64 = 0x93000020;
/// X's Route 9's spiky ground, blocked, crossed only riding a Rhyhorn (x_route_9_rhyhorn, spike
/// A).
const SPIKY: i64 = 0x32000025;
/// The flat ground between the spikes and plain ground, where the Rhyhorn is got on and off.
const FLAT: i64 = 0x31000020;
/// The Rhyhorn's character, waiting on flat ground to be ridden.
const RHYHORN: i64 = 0x100F;
/// Route 9's rocks, `IROCK02`, which the Rhyhorn smashes with A.
const ROCK: i64 = 0x200D;

/// What `FakeField.ground` is run through before it is handed over: Python's tests replace the
/// method itself, which Rust cannot do to a value behind a trait.
type Edit = Rc<dyn Fn(Ground) -> Ground>;

/// Stands in for the field reader: a fixed list, or none while off the map, over one loaded
/// chunk with `blocked` cells in it, `ledges` dropping south, `sea`, and `slopes` going down
/// south.
struct FakeField {
    r#where: RefCell<Option<Field>>,
    blocked: RefCell<HashSet<Cell>>,
    ledges: HashSet<Cell>,
    sea: HashSet<Cell>,
    slopes: HashSet<Cell>,
    /// Cells of the Acro Bike's bumpy slopes, each hopped up north.
    bumpy: RefCell<HashSet<Cell>>,
    shore: RefCell<HashSet<Cell>>,
    /// Cells of a waterfall, each climbed north.
    falls: RefCell<HashSet<Cell>>,
    /// Cells of a current carrying the player west.
    currents: RefCell<HashSet<Cell>>,
    /// A floor's height -> the blocked cells its deck opens, walked at that height alone. A
    /// list, since a float is no key in Rust.
    decks: RefCell<Vec<(f64, HashSet<Cell>)>>,
    surfaces: RefCell<IndexMap<Cell, Rc<HashSet<Cell>>>>,
    /// A stacked ground's cells, each with its lower floor's height and its upper's, None where
    /// that floor is closed; no cell, and the ground is not stacked.
    floors: RefCell<IndexMap<Cell, (Option<f64>, Option<f64>)>>,
    /// A cell and one of its floors, open there with no height: its mesh over none of it.
    bare: RefCell<HashSet<(Cell, usize)>>,
    /// The player's height on a stacked ground.
    level: Slot<Option<f64>>,
    /// A stacked ground's ladders, each step with the heights it leaves and lands at.
    ladders: RefCell<IndexMap<(Cell, Cell), (f64, f64)>>,
    ice: RefCell<Vec<IceRoom>>,
    stones: RefCell<HashSet<Cell>>,
    launches: RefCell<HashSet<Cell>>,
    spiky: RefCell<HashSet<Cell>>,
    flat: RefCell<HashSet<Cell>>,
    visit: Slot<Option<(i64, i64)>>,
    edit: RefCell<Option<Edit>>,
}

impl FakeField {
    fn new(
        r#where: Option<Field>,
        blocked: &[Cell],
        ledges: &[Cell],
        sea: &[Cell],
        slopes: &[Cell],
    ) -> Self {
        FakeField {
            r#where: RefCell::new(r#where),
            blocked: RefCell::new(blocked.iter().copied().collect()),
            ledges: ledges.iter().copied().collect(),
            sea: sea.iter().copied().collect(),
            slopes: slopes.iter().copied().collect(),
            bumpy: RefCell::new(HashSet::new()),
            shore: RefCell::new(HashSet::new()),
            falls: RefCell::new(HashSet::new()),
            currents: RefCell::new(HashSet::new()),
            decks: RefCell::new(Vec::new()),
            surfaces: RefCell::new(IndexMap::new()),
            floors: RefCell::new(IndexMap::new()),
            bare: RefCell::new(HashSet::new()),
            level: Slot::new(None),
            ladders: RefCell::new(IndexMap::new()),
            ice: RefCell::new(Vec::new()),
            stones: RefCell::new(HashSet::new()),
            launches: RefCell::new(HashSet::new()),
            spiky: RefCell::new(HashSet::new()),
            flat: RefCell::new(HashSet::new()),
            visit: Slot::new(Some((0x08D209B8, 0x6))),
            edit: RefCell::new(None),
        }
    }
}

impl FieldSource for FakeField {
    fn visit(&self) -> Option<(i64, i64)> {
        self.visit.get()
    }

    fn poll(&self) -> Result<Option<Field>> {
        let r#where = self.r#where.borrow();
        let Some(r#where) = r#where.as_ref() else {
            return Ok(None);
        };
        Ok(Some(Field {
            interactables: Vec::new(),
            ..r#where.clone()
        }))
    }

    fn interactables(&self, r#where: &Field) -> Result<Field> {
        let here = self.r#where.borrow();
        let here = here.as_ref().unwrap();
        Ok(Field {
            interactables: here.interactables.clone(),
            ..r#where.clone()
        })
    }

    fn ground(&self, r#where: &Field, height: Option<f64>) -> Result<Ground> {
        let origin = (80, 160);
        let cells: Vec<Cell> = (0..CHUNK * CHUNK)
            .map(|i| (origin.0 + i % CHUNK, origin.1 + i / CHUNK))
            .collect();
        let decks = self.decks.borrow();
        let opened: HashSet<Cell> = match height {
            Some(height) => decks
                .iter()
                .find(|(known, _)| *known == height)
                .map(|(_, opened)| opened.clone())
                .unwrap_or_default(),
            None => HashSet::new(),
        };
        let blocked = self.blocked.borrow();
        let bumpy = self.bumpy.borrow();
        let shore = self.shore.borrow();
        let falls = self.falls.borrow();
        let currents = self.currents.borrow();
        let stones = self.stones.borrow();
        let launches = self.launches.borrow();
        let spiky = self.spiky.borrow();
        let flat = self.flat.borrow();
        let words: Vec<i64> = cells
            .iter()
            .map(|cell| {
                if self.ledges.contains(cell) {
                    SOUTH_LEDGE
                } else if currents.contains(cell) {
                    WEST_CURRENT
                } else if self.sea.contains(cell) {
                    SEA
                } else if self.slopes.contains(cell) {
                    SOUTH_SLOPE
                } else if bumpy.contains(cell) {
                    BUMPY
                } else if shore.contains(cell) {
                    SHORE
                } else if falls.contains(cell) {
                    FALLS
                } else if stones.contains(cell) {
                    STONE
                } else if launches.contains(cell) {
                    LAUNCH
                } else if spiky.contains(cell) {
                    SPIKY
                } else if flat.contains(cell) {
                    FLAT
                } else if blocked.contains(cell) && !opened.contains(cell) {
                    BLOCKED
                } else {
                    0
                }
            })
            .collect();
        let floors = self.floors.borrow();
        if !floors.is_empty() {
            let one = |cell: &Cell, k: usize| {
                let (lower, upper) = floors.get(cell).copied().unwrap_or((None, None));
                if k == 0 { lower } else { upper }
            };
            let stack: Vec<Floor> = (0..2)
                .map(|k| Floor {
                    words: cells
                        .iter()
                        .map(|cell| {
                            if one(cell, k).is_some() || self.bare.borrow().contains(&(*cell, k)) {
                                0
                            } else {
                                BLOCKED
                            }
                        })
                        .collect(),
                    heights: cells.iter().map(|cell| one(cell, k)).collect(),
                })
                .collect();
            let words: Vec<i64> = cells
                .iter()
                .map(|cell| {
                    if floors.contains_key(cell) {
                        0
                    } else {
                        BLOCKED
                    }
                })
                .collect();
            let mut ground = Ground::of(
                r#where,
                Of {
                    chunks: vec![Rc::new(Chunk {
                        floors: stack,
                        ..Chunk::new(origin, words)
                    })],
                    ..Of::default()
                },
            );
            ground.stacked = true;
            ground.level = self.level.get();
            ground.ladders = self.ladders.borrow().clone();
            return Ok(ground);
        }
        let mut ground = Ground::of(
            r#where,
            Of {
                chunks: vec![Rc::new(Chunk::new(origin, words))],
                clears: oras::game().clears.clone(),
                ..Of::default()
            },
        );
        ground.ice = self.ice.borrow().clone();
        ground.decks = decks
            .iter()
            .map(|(known, _)| *known)
            .filter(|known| Some(*known) != height)
            .collect();
        ground.hops = bumpy
            .iter()
            .map(|&cell| (cell, pathfind::NORTH))
            .collect::<IndexMap<_, _>>();
        ground.falls = falls
            .iter()
            .map(|&cell| (cell, pathfind::NORTH))
            .collect::<IndexMap<_, _>>();
        ground.surfaces = self.surfaces.borrow().clone();
        if !spiky.is_empty() {
            ground = ground.mounted(
                r#where,
                &HashSet::from([SPIKY >> 24]),
                &HashSet::from([FLAT >> 24]),
                &HashSet::from([RHYHORN]),
                &HashSet::from([ROCK]),
            )?;
        }
        let edit = self.edit.borrow().clone();
        Ok(match edit {
            Some(edit) => edit(ground),
            None => ground,
        })
    }
}

/// The town: a door and an edge to the north, a sign, and two people.
fn town() -> Field {
    Field {
        interactables: vec![
            Interactable {
                zone: Some(0xDF),
                source: 0,
                ..Interactable::new("exits", "door", (94, 171))
            },
            Interactable {
                size: (40, 1),
                zone: Some(0x17),
                source: 0x17,
                ..Interactable::new("exits", "edge", (80, 159))
            },
            Interactable {
                source: 0,
                ..Interactable::new("signs", "sign", (97, 171))
            },
            Interactable {
                source: 0x08D1E9A8,
                ..Interactable::new("people", "person", (104, 179))
            },
            Interactable {
                source: 0x08D1F458,
                ..Interactable::new("people", "person", (90, 160))
            },
        ],
        ..Field::new(0x6, (94, 172))
    }
}

/// `navigator`'s arguments, each as the Python defaults it.
struct Want {
    r#where: Option<Field>,
    exits: bool,
    direction: bool,
    blocked: Vec<Cell>,
    ledges: Vec<Cell>,
    sea: Vec<Cell>,
    slopes: Vec<Cell>,
    names: Option<Rc<Names>>,
    leads_to: Option<LeadsTo>,
    street: Option<Street>,
    clock: Rc<dyn Fn() -> f64>,
    navigation: Option<Navigation>,
    door_sign: Option<DoorSign>,
}

impl Default for Want {
    fn default() -> Self {
        Want {
            r#where: Some(town()),
            exits: false,
            direction: false,
            blocked: Vec::new(),
            ledges: Vec::new(),
            sea: Vec::new(),
            slopes: Vec::new(),
            names: None,
            leads_to: None,
            street: None,
            clock: Rc::new(|| 0.0),
            navigation: None,
            door_sign: None,
        }
    }
}

/// A `Want` on the field the test names.
fn on(r#where: Field) -> Want {
    Want {
        r#where: Some(r#where),
        ..Want::default()
    }
}

/// What `navigator` hands back: the navigator, what it said, what it logged, and the field it
/// reads, which Python's tests reach through `nav._field`.
struct Rig {
    nav: Navigator,
    backend: Rc<Recorder>,
    log: Rc<RefCell<Vec<String>>>,
    field: Rc<FakeField>,
}

impl Rig {
    fn press(&mut self, action: &str) {
        self.nav.press(action).unwrap();
    }

    fn press_key(&mut self, key: &str) {
        self.nav.press_key(key).unwrap();
    }
}

/// A navigator on `where`, moved on to the exits category or into direction mode first if
/// asked, with what those presses said forgotten.
fn navigator(want: Want) -> Rig {
    let backend = Rc::new(Recorder::default());
    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let field = Rc::new(FakeField::new(
        want.r#where,
        &want.blocked,
        &want.ledges,
        &want.sea,
        &want.slopes,
    ));
    let lines = log.clone();
    let nav = Navigator::new(
        field.clone(),
        Rc::new(Speech::plain(backend.clone())),
        Rc::new(move |line: &str| lines.borrow_mut().push(line.to_string())),
        Options {
            names: want.names,
            leads_to: want.leads_to,
            street: want.street,
            clock: Some(want.clock),
            navigation: want.navigation,
            door_sign: want.door_sign,
            moves: Some(IndexMap::from([
                (SURF, "Surf".to_string()),
                (CUT, "Cut".to_string()),
                (STRENGTH, "Strength".to_string()),
                (ROCK_SMASH, "Rock Smash".to_string()),
                (WATERFALL, "Waterfall".to_string()),
            ])),
            bike: Some("Mach Bike".to_string()),
            ..Options::default()
        },
    );
    let mut rig = Rig {
        nav,
        backend,
        log,
        field,
    };
    if want.exits {
        rig.press(CATEGORY_NEXT);
    }
    if want.direction {
        rig.press(MODE_TOGGLE);
    }
    rig.backend.spoken.borrow_mut().clear();
    rig.log.borrow_mut().clear();
    rig
}

fn said(backend: &Recorder) -> Vec<String> {
    backend
        .spoken
        .borrow()
        .iter()
        .map(|(text, _)| text.clone())
        .collect()
}

/// `said(backend)[-1]`.
fn last(backend: &Recorder) -> String {
    said(backend).pop().unwrap()
}

/// `said(backend)[-count:]`.
fn tail(backend: &Recorder, count: usize) -> Vec<String> {
    let said = said(backend);
    said[said.len() - count..].to_vec()
}

/// `said(backend)[start:]`.
fn from(backend: &Recorder, start: usize) -> Vec<String> {
    said(backend)[start..].to_vec()
}

/// `backend.spoken`, to compare with what was said and how.
fn heard(backend: &Recorder) -> Vec<(String, bool)> {
    backend.spoken.borrow().clone()
}

fn walk_to(rig: &Rig, cell: Cell) {
    let mut slot = rig.field.r#where.borrow_mut();
    let was = slot.as_ref().unwrap();
    *slot = Some(Field {
        interactables: was.interactables.clone(),
        ..Field::new(was.zone, cell)
    });
}

/// Walk the player through `cells`, the runtime following each step as it polls.
fn step(rig: &mut Rig, cells: &[Cell]) {
    for &cell in cells {
        walk_to(rig, cell);
        let r#where = rig.field.poll().unwrap().unwrap();
        rig.nav.follow(&r#where).unwrap();
    }
}

fn guidance(rig: &Rig) -> Option<Rc<RefCell<Guidance>>> {
    rig.nav.guide.guidance.clone()
}

/// The cells of the way being walked.
fn path_of(way: &Option<Rc<RefCell<Guidance>>>) -> Vec<Cell> {
    way.as_ref().unwrap().borrow().path.clone()
}

fn cells(xs: std::ops::Range<i64>, z: i64) -> Vec<Cell> {
    xs.map(|x| (x, z)).collect()
}

#[test]
fn test_both_modes_start_on_everything_nearest_first() {
    let mut rig = navigator(Want::default());
    rig.press(ITEM_READ);
    rig.press(ITEM_NEXT);
    assert_eq!(
        said(&rig.backend),
        ["door, 1 of 5, 1 north", "sign, 2 of 5, 3 east 1 north"]
    );
}

/// Players asked that a thing keep its place, to tell which house they had been in by it.
#[test]
fn test_unsorted_the_list_keeps_the_tables_order_as_the_player_walks() {
    let unsorted = Navigation {
        sort_by_distance: false,
        ..Navigation::default()
    };
    let mut rig = navigator(Want {
        navigation: Some(unsorted.clone()),
        ..Want::default()
    });
    walk_to(&rig, (90, 161));
    rig.press(ITEM_READ);
    rig.press(ITEM_NEXT);
    let mut looking = navigator(Want {
        direction: true,
        navigation: Some(unsorted),
        ..Want::default()
    });
    walk_to(&looking, (90, 161));
    looking.press(NORTH);
    let mut heard = said(&rig.backend);
    heard.extend(said(&looking.backend));
    assert_eq!(
        heard,
        [
            "door, 1 of 5, 4 east 10 south",
            "edge, 2 of 5, 2 north",
            "person, 1 of 2, 1 north"
        ]
    );
}

/// Orion, 2026-09-30: "Nurse, 2 west 1 north, 1 of 9", and a setting for each.
#[test]
fn test_the_place_in_the_list_may_follow_the_way_or_be_left_out() {
    let mut after = navigator(Want {
        navigation: Some(Navigation {
            position_first: false,
            ..Navigation::default()
        }),
        ..Want::default()
    });
    after.press(ITEM_READ);
    let mut without = navigator(Want {
        navigation: Some(Navigation {
            positions: false,
            ..Navigation::default()
        }),
        ..Want::default()
    });
    without.press(ITEM_READ);
    let mut both = said(&after.backend);
    both.extend(said(&without.backend));
    assert_eq!(both, ["door, 1 north, 1 of 5", "door, 1 north"]);
}

/// One of room 0x69's boulders (`boulders`).
fn boulder(
    cell: Cell,
    source: i64,
    index: i64,
    start: Cell,
    pushes: &[(&'static str, bool)],
) -> Interactable {
    Interactable {
        size: (2, 2),
        code: 0x2009,
        source,
        move_model: true,
        record: Some((0x69, index)),
        start: Some(start),
        pushes: Some(pushes.to_vec()),
        ..Interactable::new("obstacles", "object", cell)
    }
}

/// Room 0x69 of Seafloor Cavern partway through movie37 (spike 202): record 5 pushed five east,
/// record 0 half onto the north-west hole, and record 1 against the east wall.
fn boulders() -> Field {
    Field {
        interactables: vec![
            boulder(
                (21, 25),
                0x08D21F18,
                5,
                (16, 25),
                &[("west", false), ("south", false)],
            ),
            boulder(
                (16, 19),
                0x08D1E9A8,
                0,
                (16, 20),
                &[("north", true), ("east", false), ("south", false)],
            ),
            boulder((23, 20), 0x08D1F458, 1, (23, 20), &[]),
        ],
        ..Field::new(0x69, (20, 30))
    }
}

/// Orion, 2026-10-11: what it can do, not what it is doing, and the push that drops it.
#[test]
fn test_a_big_boulder_says_the_ways_it_can_be_pushed_after_the_way_to_it() {
    let mut rig = navigator(on(boulders()));
    rig.press(CATEGORY_NEXT);
    for _ in boulders().interactables {
        rig.press(ITEM_NEXT);
    }
    assert_eq!(
        said(&rig.backend),
        [
            "obstacles, 3",
            "object, 2 of 3, 3 east 9 north, immoveable",
            "object, 3 of 3, 3 west 10 north, moveable north into a hole, east or south",
            "object, 1 of 3, 1 east 4 north, moveable west or south"
        ]
    );
}

/// Orion, 2026-10-11: which boulders have been pushed, with the way to it as a teleporter's
/// way out is, the place in the list before or after them both; off unless asked for, since the
/// game never shows where one started.
#[test]
fn test_a_big_boulder_pushed_says_how_far_from_its_start_where_asked() {
    let mut first = navigator(Want {
        navigation: Some(Navigation {
            track_boulders: true,
            ..Navigation::default()
        }),
        ..on(boulders())
    });
    first.press(CATEGORY_NEXT);
    first.press(ITEM_READ);
    first.press(ITEM_PREVIOUS);
    first.press(ITEM_PREVIOUS);
    let mut after = navigator(Want {
        navigation: Some(Navigation {
            track_boulders: true,
            position_first: false,
            ..Navigation::default()
        }),
        ..on(boulders())
    });
    after.press(CATEGORY_NEXT);
    after.press(ITEM_READ);
    let mut both = said(&first.backend);
    both.extend(said(&after.backend));
    assert_eq!(
        both,
        [
            "obstacles, 3",
            "object, 1 of 3, 1 east 4 north, moved 5 east, moveable west or south",
            "object, 3 of 3, 3 west 10 north, moved 1 north, moveable north into a hole, east or \
             south",
            "object, 2 of 3, 3 east 9 north, immoveable",
            "obstacles, 3",
            "object, 1 east 4 north, moved 5 east, moveable west or south, 1 of 3"
        ]
    );
}

/// Mauville Hills' second floor: Apt. 13's plate, sign 12, which says only the name its door is
/// called, and Apt. 4's, which says the flat is vacant; and a person whose source is the plate's.
fn hills() -> Field {
    Field {
        interactables: vec![
            Interactable {
                code: 13,
                source: place_source(FURNITURE_SOURCE, 12).unwrap(),
                ..Interactable::new("signs", "sign", (51, 35))
            },
            Interactable {
                code: 4,
                source: place_source(FURNITURE_SOURCE, 3).unwrap(),
                ..Interactable::new("signs", "sign", (60, 35))
            },
            Interactable {
                code: 0x10,
                source: 12,
                move_model: true,
                ..Interactable::new("people", "person", (52, 38))
            },
        ],
        ..Field::new(0x1BE, (50, 40))
    }
}

/// Orion, 2026-10-13: Mauville Hills' doors are named by their plates, and each plate
/// listed beside its door said the name a second time. A person is never taken for one.
#[test]
fn test_a_sign_saying_only_its_doors_name_is_left_out_unless_asked_for() {
    let door_sign: DoorSign = Rc::new(|zone, slot| (zone, slot) == (0x1BE, 12));
    let mut hiding = navigator(Want {
        door_sign: Some(door_sign.clone()),
        ..on(hills())
    });
    hiding.press(CATEGORY_NEXT);
    hiding.press(CATEGORY_NEXT);
    let mut shown = navigator(Want {
        door_sign: Some(door_sign),
        navigation: Some(Navigation {
            hide_door_signs: false,
            ..Navigation::default()
        }),
        ..on(hills())
    });
    shown.press(CATEGORY_NEXT);
    let mut both = said(&hiding.backend);
    both.extend(said(&shown.backend));
    assert_eq!(both, ["interactables, 1", "people, 1", "interactables, 2"]);
}

#[test]
fn test_a_category_is_said_with_how_many_things_are_in_it() {
    let mut rig = navigator(Want::default());
    rig.press(CATEGORY_NEXT);
    rig.press(CATEGORY_NEXT);
    assert_eq!(said(&rig.backend), ["exits, 2", "interactables, 1"]);
}

#[test]
fn test_empty_categories_are_skipped_either_way() {
    let mut rig = navigator(Want::default());
    rig.press(CATEGORY_PREVIOUS);
    assert_eq!(said(&rig.backend), ["people, 2"]);
    rig.press(CATEGORY_NEXT);
    rig.press(CATEGORY_NEXT);
    assert_eq!(tail(&rig.backend, 2), ["all, 5", "exits, 2"]);
}

#[test]
fn test_items_are_said_with_their_place_nearest_first_and_wrap() {
    let mut rig = navigator(Want {
        exits: true,
        ..Want::default()
    });
    rig.press(ITEM_NEXT);
    rig.press(ITEM_NEXT);
    rig.press(ITEM_PREVIOUS);
    assert_eq!(
        said(&rig.backend),
        [
            "edge, 2 of 2, 13 north",
            "door, 1 of 2, 1 north",
            "edge, 2 of 2, 13 north"
        ]
    );
}

#[test]
fn test_reading_the_item_again_uses_the_bearing_from_where_the_player_stands() {
    let mut rig = navigator(Want {
        exits: true,
        ..Want::default()
    });
    rig.press(ITEM_READ);
    walk_to(&rig, (94, 180));
    rig.press(ITEM_READ);
    assert_eq!(
        said(&rig.backend),
        ["door, 1 of 2, 1 north", "door, 1 of 2, 9 north"]
    );
}

#[test]
fn test_with_nothing_on_the_map_every_key_says_so() {
    let here = town();
    let nothing = Field::new(here.zone, here.cell);
    let mut rig = navigator(on(nothing));
    rig.press(CATEGORY_NEXT);
    rig.press(ITEM_NEXT);
    rig.press(NORTH);
    assert_eq!(
        said(&rig.backend),
        ["nothing here", "nothing here", "nothing north"]
    );
}

#[test]
fn test_off_the_map_there_is_no_map() {
    let mut rig = navigator(Want {
        r#where: None,
        ..Want::default()
    });
    rig.press(ITEM_READ);
    rig.press(NORTH);
    assert_eq!(said(&rig.backend), ["no map", "no map"]);
    assert_eq!(*rig.log.borrow(), ["key: no map", "key: no map"]);
}

#[test]
fn test_c_says_the_players_cell_and_off_the_map_no_map() {
    let mut rig = navigator(Want::default());
    rig.press(COORDINATES);
    walk_to(&rig, (94, 180));
    rig.press(COORDINATES);
    let mut off = navigator(Want {
        r#where: None,
        ..Want::default()
    });
    off.press(COORDINATES);
    assert_eq!(
        [said(&rig.backend), said(&off.backend)].concat(),
        ["x 94, z 172", "x 94, z 180", "no map"]
    );
}

/// Orion, 2026-10-15: the cells of the thing the keys are on, as C says the player's, for
/// picturing the map; an edge 40 cells wide by the cells it spans.
#[test]
fn test_shift_c_says_where_the_chosen_thing_is_by_the_cells_it_spans() {
    let mut rig = navigator(Want {
        exits: true,
        ..Want::default()
    });
    rig.press_key("shift+c");
    rig.press(ITEM_NEXT);
    rig.press_key("shift+c");
    assert_eq!(
        said(&rig.backend),
        [
            "x 94, z 171",
            "edge, 2 of 2, 13 north",
            "x 80 to 119, z 159"
        ]
    );
}

/// `[pa3ds.navigation] show_y`: y after x and z, in the game's own units, rounded, for the
/// player and for a thing whose height is read; none where it is not, as for a sign. Last,
/// as length, width and height go (Orion, 2026-10-15).
#[test]
fn test_show_y_says_how_high_the_player_and_the_thing_stand_where_known() {
    let door = Interactable {
        zone: Some(0xDF),
        source: 0,
        height: Some(36.4),
        ..Interactable::new("exits", "door", (94, 171))
    };
    let sign = Interactable {
        source: 0,
        ..Interactable::new("signs", "sign", (97, 171))
    };
    let r#where = Field {
        interactables: vec![door, sign],
        height: Some(20.0),
        ..Field::new(0x6, (94, 172))
    };
    let mut rig = navigator(Want {
        navigation: Some(Navigation {
            show_y: true,
            ..Navigation::default()
        }),
        ..on(r#where)
    });
    rig.press(COORDINATES);
    rig.press_key("shift+c");
    rig.press(ITEM_NEXT);
    rig.press_key("shift+c");
    assert_eq!(
        said(&rig.backend),
        [
            "x 94, z 172, y 20",
            "x 94, z 171, y 36",
            "sign, 2 of 2, 3 east 1 north",
            "x 97, z 171"
        ]
    );
}

#[test]
fn test_a_key_answer_interrupts() {
    let mut rig = navigator(Want {
        exits: true,
        ..Want::default()
    });
    rig.press(ITEM_READ);
    assert_eq!(
        heard(&rig.backend),
        [("door, 1 of 2, 1 north".to_string(), true)]
    );
    // `Speech(backend)._interrupts` is private to Speech here: a new Speech takes these.
    assert!(default_interrupts().contains(&Category::Key));
}

/// K tracked one person and then, a few steps on, another (the user, 2026-09-21).
#[test]
fn test_the_chosen_person_stays_chosen_as_walking_reorders_the_list() {
    let mut rig = navigator(Want::default());
    rig.press(CATEGORY_PREVIOUS);
    rig.press(ITEM_NEXT);
    assert_eq!(last(&rig.backend), "person, 2 of 2, 10 east 7 south");
    // Walking towards them makes them the nearest; the list re-sorts, the choice does not move.
    walk_to(&rig, (104, 176));
    rig.press(ITEM_READ);
    assert_eq!(last(&rig.backend), "person, 1 of 2, 3 south");
    rig.press(ITEM_NEXT);
    assert_eq!(last(&rig.backend), "person, 2 of 2, 14 west 16 north");
}

#[test]
fn test_the_selection_is_kept_within_a_list_that_shrank() {
    let mut rig = navigator(Want {
        exits: true,
        ..Want::default()
    });
    rig.press(ITEM_NEXT);
    let here = town();
    walk_to(&rig, here.cell);
    *rig.field.r#where.borrow_mut() = Some(Field {
        interactables: here.interactables[..1].to_vec(),
        ..Field::new(here.zone, here.cell)
    });
    rig.press(ITEM_READ);
    assert_eq!(last(&rig.backend), "door, 1 of 1, 1 north");
}

// Where an exit leads.

/// The town's place and Route 101's, as the profile has them from the zone table and the
/// ROM: the edge leads onto the route, the door into one of the town's own houses.
fn route_101() -> LeadsTo {
    Rc::new(|here: i64, there: i64| {
        assert_eq!(here, town().zone);
        Ok(if there == 0x17 {
            Some("Route 101".to_string())
        } else {
            None
        })
    })
}

/// The user, 2026-09-24: nothing said that the gap north of Littleroot leads to Route 101.
/// The door into a house of the town leads nowhere new, and says nothing more.
#[test]
fn test_an_exit_into_another_place_says_the_place() {
    let mut rig = navigator(Want {
        exits: true,
        leads_to: Some(route_101()),
        ..Want::default()
    });
    rig.press(ITEM_READ);
    rig.press(ITEM_NEXT);
    assert_eq!(
        said(&rig.backend),
        [
            "door, 1 of 2, 1 north",
            "edge to Route 101, 2 of 2, 13 north"
        ]
    );
}

/// The place is the game's own word, so a name of the player's comes before it rather than
/// instead of it; a name that already says it is left alone.
#[test]
fn test_a_named_exit_keeps_the_place_after_its_name() {
    let names = Rc::new(Names::default());
    names
        .name_place(town().zone, 0x17, "the gap in the trees")
        .unwrap();
    let mut rig = navigator(Want {
        exits: true,
        names: Some(names.clone()),
        leads_to: Some(route_101()),
        ..Want::default()
    });
    rig.press(ITEM_PREVIOUS);
    names
        .name_place(town().zone, 0x17, "the way to Route 101")
        .unwrap();
    rig.press(ITEM_READ);
    assert_eq!(
        said(&rig.backend),
        [
            "the gap in the trees to Route 101, 2 of 2, 13 north",
            "the way to Route 101, 2 of 2, 13 north"
        ]
    );
}

/// Spike 107: Lumiose's zones are all "Lumiose City", so the place says nothing of where a
/// warp between two of them comes out; the street it comes out on does.
#[test]
fn test_an_exit_within_one_place_says_the_street_it_comes_out_on() {
    let mut rig = navigator(Want {
        exits: true,
        street: Some(Rc::new(|_zone: i64, source: i64| {
            Ok(if source == 0 {
                Some("Vernal Avenue".to_string())
            } else {
                None
            })
        })),
        ..Want::default()
    });
    rig.press(ITEM_READ);
    rig.press(ITEM_NEXT);
    assert_eq!(
        said(&rig.backend),
        [
            "door to Vernal Avenue, 1 of 2, 1 north",
            "edge, 2 of 2, 13 north"
        ]
    );
}

/// Team Aqua's Hideout (2026-10-06): both bearings are from the player.
#[test]
fn test_a_teleporter_says_where_it_comes_out_after_the_way_to_it() {
    let pad = Interactable {
        zone: Some(town().zone),
        source: 0,
        arrival: Some((111, 180)),
        ..Interactable::new("exits", "teleporter", (97, 172))
    };
    let mut rig = navigator(Want {
        r#where: Some(Field {
            interactables: vec![pad],
            ..Field::new(town().zone, town().cell)
        }),
        exits: true,
        ..Want::default()
    });
    rig.press(ITEM_READ);
    assert_eq!(
        said(&rig.backend),
        ["teleporter, 1 of 1, 3 east, out at 17 east 8 south"]
    );
}

#[test]
fn test_without_the_games_words_an_exit_is_what_it_is() {
    let mut rig = navigator(Want {
        exits: true,
        ..Want::default()
    });
    rig.press(ITEM_PREVIOUS);
    assert_eq!(said(&rig.backend), ["edge, 2 of 2, 13 north"]);
}

// Direction mode.

/// The user's example (2026-09-21): M into direction mode, then O for north.
#[test]
fn test_standing_south_of_a_door_o_says_door_1_north() {
    let mut rig = navigator(Want::default());
    rig.press_key("'");
    rig.press_key("o");
    assert_eq!(
        said(&rig.backend),
        ["direction mode", "door, 1 of 2, 1 north"]
    );
    assert_eq!(rig.nav.mode(), DIRECTION);
}

#[test]
fn test_a_direction_cycles_its_side_nearest_first_and_wraps() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    for _ in 0..3 {
        rig.press(NORTH);
    }
    assert_eq!(
        said(&rig.backend),
        [
            "door, 1 of 2, 1 north",
            "sign, 2 of 2, 3 east 1 north",
            "door, 1 of 2, 1 north"
        ]
    );
}

/// The edge 13 north and the person 12 north are beyond it (the user, 2026-09-21: 10).
#[test]
fn test_a_direction_looks_ten_cells_each_way_and_no_further() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    walk_to(&rig, (94, 169));
    rig.press(NORTH);
    rig.press(NORTH);
    rig.press(NORTH);
    assert_eq!(
        said(&rig.backend),
        [
            "edge, 1 of 2, 10 north",
            "person, 2 of 2, 4 west 9 north",
            "edge, 1 of 2, 10 north"
        ]
    );
}

#[test]
fn test_a_thing_north_east_is_heard_under_north_and_under_east() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    rig.press(EAST);
    rig.press(EAST);
    rig.press(SOUTH);
    rig.press(WEST);
    assert_eq!(
        said(&rig.backend),
        [
            "sign, 1 of 2, 3 east 1 north",
            "person, 2 of 2, 10 east 7 south",
            "person, 1 of 1, 10 east 7 south",
            "nothing west"
        ]
    );
}

#[test]
fn test_another_direction_starts_again_from_its_nearest() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    rig.press(NORTH);
    rig.press(NORTH);
    rig.press(EAST);
    rig.press(NORTH);
    assert_eq!(
        tail(&rig.backend, 2),
        ["sign, 1 of 2, 3 east 1 north", "door, 1 of 2, 1 north"]
    );
}

#[test]
fn test_a_direction_with_nothing_that_way_says_so_by_category() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    rig.press(CATEGORY_NEXT);
    rig.press(CATEGORY_NEXT);
    rig.press(SOUTH);
    rig.press(EAST);
    let here = town();
    let only_door = Field {
        interactables: here.interactables[..1].to_vec(),
        ..Field::new(here.zone, here.cell)
    };
    let mut rig2 = navigator(Want {
        direction: true,
        ..on(only_door)
    });
    rig2.press(SOUTH);
    assert_eq!(
        said(&rig.backend),
        [
            "exits, 2",
            "interactables, 1",
            "no interactables south",
            "sign, 1 of 1, 3 east 1 north"
        ]
    );
    assert_eq!(said(&rig2.backend), ["nothing south"]);
}

#[test]
fn test_walking_keeps_the_chosen_and_steps_on_from_it() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    rig.press(NORTH);
    rig.press(NORTH);
    assert_eq!(last(&rig.backend), "sign, 2 of 2, 3 east 1 north");
    // Beside the sign, it is the nearest thing north; the next press steps on past it.
    walk_to(&rig, (97, 172));
    rig.press(NORTH);
    assert_eq!(last(&rig.backend), "door, 2 of 2, 3 west 1 north");
}

#[test]
fn test_the_category_keys_work_in_both_modes_and_forget_the_direction() {
    let mut rig = navigator(Want {
        direction: true,
        ..Want::default()
    });
    rig.press_key("]");
    rig.press(NORTH);
    rig.press_key("[");
    rig.press(NORTH);
    assert_eq!(
        said(&rig.backend),
        [
            "exits, 2",
            "door, 1 of 1, 1 north",
            "all, 5",
            "door, 1 of 2, 1 north"
        ]
    );
}

#[test]
fn test_the_toggle_switches_mode_and_a_key_means_what_its_mode_says() {
    let mut rig = navigator(Want::default());
    rig.press_key(";");
    rig.press_key("o");
    rig.press_key("'");
    rig.press_key("o");
    rig.press_key("'");
    rig.press_key("'");
    assert_eq!(
        said(&rig.backend),
        [
            "exits, 2",
            "direction mode",
            "door, 1 of 1, 1 north",
            "touch mode, nothing to touch",
            "list mode"
        ]
    );
    assert_eq!(rig.nav.mode(), LIST);
}

/// The user's ask (2026-09-22): the shaking-tail Pokémon under a category of its own.
#[test]
fn test_a_hidden_pokemon_is_its_own_category_after_items() {
    let grass = Field {
        interactables: vec![
            Interactable {
                code: 0x105,
                source: 0x08D374E4,
                ..Interactable::new("pokémon", "hidden Pokémon", (94, 129))
            },
            Interactable {
                source: 0x08D1F458,
                ..Interactable::new("people", "person", (99, 129))
            },
        ],
        ..Field::new(0x17, (99, 130))
    };
    let mut rig = navigator(on(grass));
    rig.press(CATEGORY_PREVIOUS);
    rig.press(ITEM_READ);
    rig.press(CATEGORY_PREVIOUS);
    assert_eq!(
        said(&rig.backend),
        [
            "pokémon, 1",
            "hidden Pokémon, 1 of 1, 5 west 1 north",
            "people, 1"
        ]
    );
    rig.press(MODE_TOGGLE);
    rig.press(CATEGORY_NEXT);
    rig.press(WEST);
    rig.press(SOUTH);
    assert_eq!(
        tail(&rig.backend, 3),
        [
            "pokémon, 1",
            "hidden Pokémon, 1 of 1, 5 west 1 north",
            "no pokémon south"
        ]
    );
}

// The way there.

/// A house's front wall, its door at (95, 172) blocked in it as spike 82 found doors.
fn house_front() -> Vec<Cell> {
    cells(88..101, 172)
}

fn street() -> Field {
    Field {
        interactables: vec![
            Interactable {
                zone: Some(0xDF),
                source: place_source(WARP_SOURCE, 0).unwrap(),
                ..Interactable::new("exits", "door", (95, 172))
            },
            Interactable {
                source: 0x08D1E9A8,
                move_model: true,
                ..Interactable::new("people", "person", (100, 180))
            },
        ],
        ..Field::new(0x6, (90, 175))
    }
}

/// The user's example (2026-09-24): "5 east, 2 north", and at the end of the 5 east, north.
/// The door is solid, so the way ends in front of it, and says it there.
#[test]
fn test_p_says_the_way_leg_by_leg_and_the_next_leg_at_each_turn() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press_key("p");
    assert_eq!(said(&rig.backend), ["5 east, 2 north"]);
    step(&mut rig, &[(91, 175), (92, 175), (93, 175), (94, 175)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north"]);
    step(&mut rig, &[(95, 175)]);
    assert_eq!(last(&rig.backend), "2 north");
    step(&mut rig, &[(95, 174), (95, 173)]);
    assert_eq!(from(&rig.backend, 1), ["2 north", "door, 1 north"]);
    assert!(!rig.nav.guiding());
}

#[test]
fn test_without_turn_by_turn_the_way_is_said_once_and_p_says_it_again() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        navigation: Some(Navigation {
            turn_by_turn: false,
            ..Navigation::default()
        }),
        ..on(street())
    });
    rig.press_key("p");
    assert!(!rig.nav.guiding());
    step(
        &mut rig,
        &[(91, 175), (92, 175), (93, 175), (94, 175), (95, 175)],
    );
    rig.press_key("p");
    assert_eq!(said(&rig.backend), ["5 east, 2 north", "2 north"]);
}

#[test]
fn test_a_turn_is_said_without_interrupting_and_logged_as_the_path() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    step(
        &mut rig,
        &[(91, 175), (92, 175), (93, 175), (94, 175), (95, 175)],
    );
    assert_eq!(
        heard(&rig.backend),
        [
            ("5 east, 2 north".to_string(), true),
            ("2 north".to_string(), false)
        ]
    );
    assert_eq!(*rig.log.borrow(), ["key: 5 east, 2 north", "path: 2 north"]);
}

#[test]
fn test_p_goes_to_whatever_the_navigation_keys_are_on() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        direction: true,
        ..on(street())
    });
    rig.press(SOUTH);
    rig.press(PATHFIND);
    assert_eq!(
        said(&rig.backend),
        ["person, 1 of 1, 10 east 5 south", "10 east, 4 south"]
    );
}

#[test]
fn test_alt_p_drops_the_way_and_nothing_more_of_it_is_said() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    rig.press_key("alt+p");
    assert!(!rig.nav.guiding());
    step(
        &mut rig,
        &[(91, 175), (92, 175), (93, 175), (94, 175), (95, 175)],
    );
    rig.press_key("alt+p");
    assert_eq!(
        said(&rig.backend),
        ["5 east, 2 north", "path off", "no path"]
    );
}

#[test]
fn test_a_step_the_wrong_way_says_the_new_way_from_there() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    step(&mut rig, &[(89, 175)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north", "6 east, 2 north"]);
}

/// North first where the path went east first heard the whole way again at every step
/// (poke_center_pc.ctm, 2026-09-24): going on the way the player walks is taken quietly.
#[test]
fn test_a_way_of_the_players_own_as_short_is_followed_and_its_turn_said() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    step(&mut rig, &[(91, 175), (92, 175), (93, 175), (93, 174)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north"]);
    step(&mut rig, &[(93, 173)]);
    assert_eq!(last(&rig.backend), "2 east");
    step(&mut rig, &[(94, 173), (95, 173)]);
    assert_eq!(from(&rig.backend, 1), ["2 east", "door, 1 north"]);
}

#[test]
fn test_a_wall_in_a_chunk_loaded_since_finds_the_path_again() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    rig.field
        .blocked
        .borrow_mut()
        .extend((92..96).map(|x| (x, 174)));
    step(&mut rig, &[(91, 175)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north", "2 north, 4 east"]);
}

/// Issue 3: a ledge was a wall, walked round. The player's cell goes from before the ledge
/// to after it in one move (omega_ruby_ledges.ctm), which is still on the way.
#[test]
fn test_a_ledge_on_the_way_is_jumped_and_the_jump_followed() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ledges: cells(80..120, 177),
        ..on(street())
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(said(&rig.backend), ["people, 1", "10 east, 4 south"]);
    let way = guidance(&rig);
    let mut walked = cells(91..101, 175);
    walked.extend([(100, 176), (100, 178)]);
    step(&mut rig, &walked);
    assert!(
        Rc::ptr_eq(
            way.as_ref().unwrap(),
            rig.nav.guide.guidance.as_ref().unwrap()
        ),
        "the way was found again"
    );
    step(&mut rig, &[(100, 179)]);
    assert_eq!(from(&rig.backend, 2), ["4 south", "person, 1 south"]);
}

fn sea_band() -> Vec<Cell> {
    (80..120)
        .flat_map(|x| [177, 178].into_iter().map(move |z| (x, z)))
        .collect()
}

/// Route 109: P walked the player into the sea, where the blocked bit is clear
/// (omega_ruby_surf.ctm, the user, 2026-09-28). P goes round the water; Shift and P crosses it,
/// and says the move as the leg on the water begins.
#[test]
fn test_p_keeps_to_land_and_shift_p_says_the_leg_to_surf_when_the_player_reaches_it() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        sea: sea_band(),
        ..on(street())
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    let way = path_of(&guidance(&rig));
    assert!(!way.iter().any(|cell| sea_band().contains(cell)));
    assert!(!last(&rig.backend).contains("Surf"));
    rig.press_key("shift+p");
    assert_eq!(
        last(&rig.backend),
        "10 east, 1 south, Surf 2 south, 1 south"
    );
    let mut walked = cells(91..101, 175);
    walked.push((100, 176));
    step(&mut rig, &walked);
    assert_eq!(tail(&rig.backend, 2), ["1 south", "Surf 2 south"]);
    step(&mut rig, &[(100, 177), (100, 178), (100, 179)]);
    assert_eq!(tail(&rig.backend, 2), ["1 south", "person, 1 south"]);
}

#[test]
fn test_p_from_the_water_crosses_it_without_saying_so() {
    let here = street();
    let water = Field {
        interactables: here.interactables.clone(),
        ..Field::new(here.zone, (100, 177))
    };
    let mut rig = navigator(Want {
        blocked: house_front(),
        sea: sea_band(),
        ..on(water)
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(last(&rig.backend), "2 south");
}

/// Route 122: the shore along the water is blocked, and Surf jumps the player over it, from
/// (22, 8) to (22, 10) and off from (54, 58) to (54, 56); P walled the water off at both
/// (omega_ruby_surf3.ctm, spike 162). Shift and P jumps it both ways, saying the
/// jump with the side it lands on, and P from the water lands.
#[test]
fn test_shift_p_jumps_the_shore_onto_the_water_and_off_it() {
    let here = street();
    let across = Field {
        interactables: here.interactables.clone(),
        ..Field::new(here.zone, (100, 175))
    };
    let sea: Vec<Cell> = cells(80..120, 177);
    let mut rig = navigator(Want {
        blocked: house_front(),
        sea: sea.clone(),
        ..on(across)
    });
    let shore: HashSet<Cell> = (80..120)
        .flat_map(|x| [176, 178].into_iter().map(move |z| (x, z)))
        .collect();
    *rig.field.shore.borrow_mut() = shore.clone();
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    let way = path_of(&guidance(&rig));
    assert!(
        !way.iter()
            .any(|cell| sea.contains(cell) || shore.contains(cell))
    );
    rig.press_key("shift+p");
    assert_eq!(last(&rig.backend), "Surf 2 south, 2 south");
    step(&mut rig, &[(100, 177)]);
    assert_eq!(last(&rig.backend), "2 south");
    step(&mut rig, &[(100, 179)]);
    assert_eq!(last(&rig.backend), "person, 1 south");
    *rig.field.r#where.borrow_mut() = Some(Field {
        interactables: here.interactables.clone(),
        ..Field::new(here.zone, (100, 177))
    });
    rig.press(PATHFIND);
    assert_eq!(last(&rig.backend), "2 south");
}

/// X's Route 8 (x_route-8-3.ctm, spike 235): a mud pit's stones are blocked in the cell table, and
/// the player hops from the launch at the pit's edge onto one two cells on, over the mud, then from
/// stone to stone and onto the launch on the far side, each hop one move. P took the pit for a
/// wall, "can't continue".
#[test]
fn test_p_hops_across_a_mud_pit_from_stone_to_stone() {
    let across = Field {
        interactables: vec![Interactable {
            source: 0x08D1E9A8,
            ..Interactable::new("people", "person", (101, 175))
        }],
        ..Field::new(street().zone, (90, 175))
    };
    let mut blocked: Vec<Cell> = (92..99)
        .flat_map(|x| (160..200).map(move |z| (x, z)))
        .collect();
    blocked.push((101, 175));
    let mut rig = navigator(Want {
        blocked,
        ..on(across)
    });
    let stones: HashSet<Cell> = [(93, 175), (95, 175), (97, 175)].into_iter().collect();
    *rig.field.stones.borrow_mut() = stones.clone();
    *rig.field.launches.borrow_mut() = [(91, 175), (99, 175)].into_iter().collect();
    rig.press(PATHFIND);
    let way = guidance(&rig);
    assert_eq!(last(&rig.backend), "10 east");
    let path = path_of(&way);
    assert!(stones.iter().all(|stone| path.contains(stone)) && path.iter().all(|c| c.1 == 175));
    step(
        &mut rig,
        &[(91, 175), (93, 175), (95, 175), (97, 175), (99, 175)],
    );
    assert!(
        Rc::ptr_eq(
            way.as_ref().unwrap(),
            rig.nav.guide.guidance.as_ref().unwrap()
        ),
        "each hop is on the way"
    );
    step(&mut rig, &[(100, 175)]);
    assert_eq!(last(&rig.backend), "person, 1 east");
}

/// A band of spiky ground across the chunk.
fn spikes() -> HashSet<Cell> {
    (92..98)
        .flat_map(|x| (160..200).map(move |z| (x, z)))
        .collect()
}

/// A band of spiky ground across the chunk, with flat ground along each side.
fn flats() -> HashSet<Cell> {
    [90, 91, 98, 99]
        .into_iter()
        .flat_map(|x| (160..200).map(move |z| (x, z)))
        .collect()
}

/// The person east of the spikes, as Python's tests give it.
fn person_at(cell: Cell) -> Interactable {
    Interactable {
        source: 0x08D1E9A8,
        ..Interactable::new("people", "person", cell)
    }
}

/// A rock the Rhyhorn smashes.
fn rock_at(cell: Cell) -> Interactable {
    Interactable {
        code: ROCK,
        source: 0x08D21F18,
        move_model: true,
        ..Interactable::new("obstacles", "rock", cell)
    }
}

/// X's Route 9 (x_route_9_rhyhorn, spike 238): spiky ground is blocked in the cell table, and a
/// player riding the Rhyhorn, the bike's bits set, crosses it from flat ground to flat ground and
/// gets off onto plain ground. Riding at (31, 61), every way was "no path".
#[test]
fn test_p_riding_crosses_spiky_ground_and_gets_off_onto_plain_ground() {
    let riding = Field {
        interactables: vec![person_at((102, 175))],
        on_bike: true,
        ..Field::new(street().zone, (91, 175))
    };
    let mut rig = navigator(Want {
        blocked: vec![(102, 175)],
        ..on(riding)
    });
    *rig.field.spiky.borrow_mut() = spikes();
    *rig.field.flat.borrow_mut() = flats();
    rig.press(PATHFIND);
    assert_eq!(last(&rig.backend), "8 east, get off 2 east");
}

/// Route 9 (spike 238): the player walked onto flat ground, faced the Rhyhorn waiting there and
/// pressed A, and the game moved them into its cell, riding. P stopped at the spikes, "then can't
/// continue".
#[test]
fn test_p_on_foot_gets_on_the_rhyhorn_waiting_on_flat_ground_to_cross_the_spikes() {
    let beside = Field {
        interactables: vec![
            Interactable {
                code: RHYHORN,
                source: 0x08D21F18,
                move_model: true,
                ..Interactable::new("people", "Rhyhorn", (90, 175))
            },
            person_at((102, 175)),
        ],
        ..Field::new(street().zone, (88, 175))
    };
    let mut rig = navigator(Want {
        blocked: vec![(102, 175)],
        ..on(beside)
    });
    *rig.field.spiky.borrow_mut() = spikes();
    *rig.field.flat.borrow_mut() = flats();
    rig.press(ITEM_NEXT);
    rig.press(PATHFIND);
    assert_eq!(
        last(&rig.backend),
        "1 east, Rhyhorn 1 east, 9 east, get off 2 east"
    );
}

/// Route 9 (spike 238): only a player riding crosses the spikes, so with nothing to ride the way
/// goes round them, here past the chunk, never across.
#[test]
fn test_p_on_foot_with_no_rhyhorn_takes_spiky_ground_for_a_wall() {
    let alone = Field {
        interactables: vec![person_at((102, 175))],
        ..Field::new(street().zone, (88, 175))
    };
    let mut rig = navigator(Want {
        blocked: vec![(102, 175)],
        ..on(alone)
    });
    *rig.field.spiky.borrow_mut() = spikes();
    *rig.field.flat.borrow_mut() = flats();
    rig.press(PATHFIND);
    let way = guidance(&rig);
    let spiky = spikes();
    assert!(way.is_some() && !path_of(&way).iter().any(|cell| spiky.contains(cell)));
}

/// Route 9 (spike 238): two rocks close the only way east across the spikes at x 51, and the
/// player smashed them riding, pressing A at each: they were gone from the list after 103 s, and
/// the player at (58, 59) by 130 s. P riding stopped at them, "then can't continue".
#[test]
fn test_p_riding_smashes_a_rock_standing_in_the_only_way_across_the_spikes() {
    let wall: HashSet<Cell> = (160..200).filter(|&z| z != 175).map(|z| (94, z)).collect();
    let riding = Field {
        interactables: vec![person_at((102, 175)), rock_at((94, 175))],
        on_bike: true,
        ..Field::new(street().zone, (91, 175))
    };
    let mut blocked: Vec<Cell> = vec![(102, 175)];
    blocked.extend(wall.iter().copied());
    let mut rig = navigator(Want {
        blocked,
        ..on(riding)
    });
    *rig.field.spiky.borrow_mut() = spikes().difference(&wall).copied().collect();
    *rig.field.flat.borrow_mut() = flats();
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(
        last(&rig.backend),
        "2 east, smash 1 east, 5 east, get off 2 east"
    );
}

/// Route 9 (spike 238): a rock is faced and smashed with A, so a way round it a step or two longer
/// is taken, as P goes round a cracked boulder.
#[test]
fn test_p_riding_goes_round_a_rock_where_that_costs_less_than_smashing_it() {
    let riding = Field {
        interactables: vec![person_at((102, 175)), rock_at((94, 175))],
        on_bike: true,
        ..Field::new(street().zone, (91, 175))
    };
    let mut rig = navigator(Want {
        blocked: vec![(102, 175)],
        ..on(riding)
    });
    *rig.field.spiky.borrow_mut() = spikes();
    *rig.field.flat.borrow_mut() = flats();
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    let way = guidance(&rig);
    assert!(way.is_some() && !path_of(&way).contains(&(94, 175)));
    assert!(!last(&rig.backend).contains("smash"));
}

/// Route 9 (spike 238): only the Rhyhorn smashes a rock, so on foot one in the gap into a pen is in
/// the way, as before.
#[test]
fn test_p_on_foot_takes_a_rock_for_somebody_in_the_way() {
    let gapped = Field {
        interactables: vec![
            Interactable {
                move_model: true,
                ..person_at((95, 182))
            },
            rock_at((95, 177)),
        ],
        ..Field::new(street().zone, (95, 175))
    };
    let mut blocked = pen(None);
    blocked.push((96, 177));
    let mut rig = navigator(Want {
        blocked,
        ..on(gapped)
    });
    *rig.field.spiky.borrow_mut() = (80..120).map(|x| (x, 199)).collect();
    *rig.field.flat.borrow_mut() = HashSet::from([(95, 177)]);
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(
        last(&rig.backend),
        "1 south, then can't continue, rock in the way"
    );
}

/// Ever Grande City: the waterfall is a row of blocked water, and Waterfall carries the player up
/// over it, from (51, 52) to (51, 50); they came back down over it with no question
/// (omega_ruby_waterfall.ctm, spike 170). Shift and P climbs it, saying the move, and goes down it
/// without; P from the water stops at it either way (the user, 2026-10-05).
#[test]
fn test_shift_p_climbs_a_waterfall_and_goes_down_it_and_p_keeps_off_it() {
    let falls: HashSet<Cell> = (80..120).map(|x| (x, 178)).collect();
    let sea: Vec<Cell> = (80..120)
        .flat_map(|x| (174..186).map(move |z| (x, z)))
        .filter(|cell| !falls.contains(cell))
        .collect();
    let below = Field {
        interactables: vec![Interactable {
            source: 0x08D1E9A8,
            move_model: true,
            ..Interactable::new("people", "person", (100, 172))
        }],
        ..Field::new(0x15, (100, 182))
    };
    let mut rig = navigator(Want {
        sea,
        ..on(below.clone())
    });
    *rig.field.falls.borrow_mut() = falls.clone();
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    let way = guidance(&rig);
    assert!(way.is_some() && !path_of(&way).iter().any(|cell| falls.contains(cell)));
    rig.press_key("shift+p");
    assert_eq!(last(&rig.backend), "3 north, Waterfall 2 north, 4 north");
    let above = Field {
        interactables: vec![Interactable {
            source: 0x08D1E9A8,
            move_model: true,
            ..Interactable::new("people", "person", (100, 187))
        }],
        ..Field::new(0x15, (100, 175))
    };
    *rig.field.r#where.borrow_mut() = Some(above);
    rig.press_key("shift+p");
    assert_eq!(last(&rig.backend), "11 south");
    rig.press(PATHFIND);
    let way = guidance(&rig);
    assert!(way.is_some() && !path_of(&way).iter().any(|cell| falls.contains(cell)));
    *rig.field.r#where.borrow_mut() = Some(Field {
        cell: (100, 177),
        ..below
    });
    rig.press(PATHFIND);
    let spoken = said(&rig.backend).len();
    step(&mut rig, &[(100, 178)]);
    assert_eq!(said(&rig.backend).len(), spoken);
}

fn current_sea() -> HashSet<Cell> {
    (80..120)
        .flat_map(|x| (174..186).map(move |z| (x, z)))
        .collect()
}

/// A row of current carrying the player west, rock at either end, the one way south across it.
fn current() -> HashSet<Cell> {
    (90..106).map(|x| (x, 178)).collect()
}

fn rocks() -> HashSet<Cell> {
    let current = current();
    (80..120)
        .map(|x| (x, 178))
        .filter(|cell| !current.contains(cell))
        .collect()
}

/// Route 132: the player surfing north of the current, a Swimmer south of it (movie43).
fn swimmer() -> Field {
    Field {
        interactables: vec![Interactable {
            source: 0x08D1E9A8,
            move_model: true,
            ..Interactable::new("people", "person", (95, 181))
        }],
        surfing: true,
        ..Field::new(0x3D, (100, 176))
    }
}

/// A navigator on `where` over the current's sea, the current laid.
fn on_the_current(r#where: Field) -> Rig {
    let rocks = rocks();
    let rig = navigator(Want {
        blocked: rocks.iter().copied().collect(),
        sea: current_sea().difference(&rocks).copied().collect(),
        ..on(r#where)
    });
    *rig.field.currents.borrow_mut() = current();
    rig
}

/// Carry the player through `cells`, the runtime following each and its field poll looking
/// at each, as on a current.
fn drift(rig: &mut Rig, cells: &[Cell]) {
    for &cell in cells {
        {
            let mut slot = rig.field.r#where.borrow_mut();
            let was = slot.take().unwrap();
            *slot = Some(Field { cell, ..was });
        }
        let r#where = rig.field.poll().unwrap().unwrap();
        rig.nav.follow(&r#where).unwrap();
        rig.nav.swept(&r#where).unwrap();
    }
}

/// Route 132: a current carries a player surfing on it its own way, a cell at a time, and
/// they cannot turn (movie43, spike 208). P's way says the current as one leg, by where it leaves
/// the player, as they reach it, and nothing while it carries them.
#[test]
fn test_p_rides_a_current_saying_where_it_leaves_the_player_and_nothing_on_it() {
    let mut rig = on_the_current(swimmer());
    rig.press(PATHFIND);
    assert_eq!(
        last(&rig.backend),
        "2 south, ride the current 10 west, 3 south, 4 east"
    );
    drift(&mut rig, &[(100, 177), (100, 178)]);
    let all = said(&rig.backend);
    assert_eq!(
        all[all.len() - 2..],
        [
            "2 south, ride the current 10 west, 3 south, 4 east",
            "ride the current 10 west"
        ]
    );
    let spoken = all.len();
    let carried: Vec<Cell> = (91..100).rev().map(|x| (x, 178)).collect();
    drift(&mut rig, &carried);
    assert_eq!(said(&rig.backend).len(), spoken);
    drift(&mut rig, &[(90, 178)]);
    assert_eq!(last(&rig.backend), "3 south");
    *rig.field.r#where.borrow_mut() = Some(Field {
        cell: (103, 178),
        ..swimmer()
    });
    rig.press(PATHFIND);
    assert_eq!(
        last(&rig.backend),
        "ride the current 13 west, 3 south, 4 east"
    );
}

/// Route 133: P was found again and said at every one of 63 cells a current carried the
/// player (movie43). Off the way, the current is said as it catches the player, by where it
/// leaves them, and the way is found again there.
#[test]
fn test_a_current_off_the_way_is_said_as_it_catches_the_player_and_the_way_found_after() {
    let upstream = Field {
        interactables: vec![Interactable {
            source: 0x08D1E9A8,
            move_model: true,
            ..Interactable::new("people", "person", (100, 173))
        }],
        ..swimmer()
    };
    let mut rig = on_the_current(upstream);
    rig.press(PATHFIND);
    assert_eq!(last(&rig.backend), "2 north");
    drift(&mut rig, &[(100, 177)]);
    let spoken = said(&rig.backend).len();
    drift(&mut rig, &[(100, 178)]);
    assert_eq!(said(&rig.backend)[spoken..], ["ride the current 10 west"]);
    let carried: Vec<Cell> = (91..100).rev().map(|x| (x, 178)).collect();
    drift(&mut rig, &carried);
    assert_eq!(said(&rig.backend)[spoken..], ["ride the current 10 west"]);
    drift(&mut rig, &[(90, 178)]);
    assert_eq!(
        said(&rig.backend)[spoken..],
        ["ride the current 10 west", "5 north, 9 east"]
    );
}

/// A sandy slope going down south, the one way up through a cliff across the chunk at z 170.
fn slope() -> Vec<Cell> {
    (168..173).map(|z| (95, z)).collect()
}

fn cliff() -> Vec<Cell> {
    let mut out: Vec<Cell> = (80..120).filter(|&x| x != 95).map(|x| (x, 170)).collect();
    for x in [94, 96] {
        out.extend((168..173).map(|z| (x, z)));
    }
    out
}

/// Jagged Pass: the Hiker up the slope from the player (omega_ruby_bike.ctm).
fn pass() -> Field {
    Field {
        interactables: vec![Interactable {
            source: 0x08D1E9A8,
            move_model: true,
            ..Interactable::new("people", "person", (95, 160))
        }],
        ..Field::new(0x54, (95, 175))
    }
}

/// Jagged Pass: P to the Hiker up the sandy slope (the user, 2026-10-03). On foot P goes round
/// it; on the bike it rides straight up; Shift and P on foot says the leg up it with the bike, as
/// the player reaches the slope.
#[test]
fn test_p_rides_up_a_slope_only_on_the_bike_and_shift_p_says_the_bike_on_foot() {
    let mut rig = navigator(Want {
        blocked: cliff(),
        slopes: slope(),
        ..on(pass())
    });
    rig.press(PATHFIND);
    let way = path_of(&guidance(&rig));
    assert!(!way.iter().any(|cell| slope().contains(cell)));
    rig.press_key("shift+p");
    assert_eq!(last(&rig.backend), "2 north, Mach Bike 5 north, 7 north");
    step(&mut rig, &[(95, 174), (95, 173)]);
    assert_eq!(last(&rig.backend), "Mach Bike 5 north");
    *rig.field.r#where.borrow_mut() = Some(Field {
        on_bike: true,
        ..pass()
    });
    rig.press(PATHFIND);
    assert_eq!(last(&rig.backend), "14 north");
}

/// Jagged Pass: the player hopped north up the bumpy slope at (49, 30) on the Acro Bike, from
/// (49, 31) to (49, 29) (the user, 2026-10-03). On foot P goes round it; Shift and P hops it.
#[test]
fn test_shift_p_hops_up_a_bumpy_slope_and_says_the_bike() {
    let mut rig = navigator(Want {
        blocked: cliff(),
        ..on(pass())
    });
    *rig.field.bumpy.borrow_mut() = HashSet::from([(95, 170)]);
    rig.press(PATHFIND);
    let way = path_of(&guidance(&rig));
    assert!(!way.contains(&(95, 170)));
    rig.press_key("shift+p");
    assert_eq!(last(&rig.backend), "4 north, bumpy slope 2 north, 8 north");
}

/// A wall across the chunk, its one gap at (95, 177) with a bush or a rock in it.
fn hedge() -> Vec<Cell> {
    (80..120).filter(|&x| x != 95).map(|x| (x, 177)).collect()
}

/// South of the hedge's one-cell gap, an item ball north through it: Seafloor Cavern's gap
/// between two pillars, where the player was refused "2 north" off its line (movie38).
fn gap(place: (f64, f64)) -> Field {
    Field {
        interactables: vec![Interactable {
            source: place_source(FURNITURE_SOURCE, 1).unwrap(),
            ..Interactable::new("items", "item ball", (95, 174))
        }],
        place: Some(place),
        ..Field::new(street().zone, (95, 179))
    }
}

/// movie38 (spike 202): told "2 north" a third of a cell east of the gap's line, the player was
/// refused it for a hundred seconds. Lined up, or off the line with no corner to catch, nothing
/// more is said.
#[test]
fn test_p_says_to_line_up_first_where_a_corner_would_catch_a_player_off_the_line() {
    let mut said_for: Vec<String> = Vec::new();
    for (place, blocked) in [
        ((95.31, 179.0), hedge()),
        ((95.0, 179.0), hedge()),
        ((94.72, 179.0), hedge()),
        ((95.31, 179.0), Vec::new()),
    ] {
        let mut rig = navigator(Want {
            blocked,
            ..on(gap(place))
        });
        rig.press(PATHFIND);
        said_for.extend(said(&rig.backend));
    }
    assert_eq!(
        said_for,
        [
            "a little west, then 5 north",
            "5 north",
            "a little east, then 5 north",
            "5 north"
        ]
    );
}

/// Stopped off the line before the gap: "a little west", once, then "lined up" as soon as
/// the player is on it, sneaking or not (Orion, 2026-10-11).
#[test]
fn test_guiding_says_which_way_to_line_up_once_stopped_and_lined_up_once_there() {
    let now = Rc::new(Slot::new(0.0));
    let clock = now.clone();
    let mut rig = navigator(Want {
        blocked: hedge(),
        clock: Rc::new(move || clock.get()),
        ..on(gap((95.0, 179.0)))
    });
    rig.press(PATHFIND);

    fn stand(rig: &mut Rig, now: &Slot<f64>, x: f64, after: f64) {
        now.set(now.get() + after);
        *rig.field.r#where.borrow_mut() = Some(gap((x, 179.0)));
        let r#where = rig.field.poll().unwrap().unwrap();
        rig.nav.follow(&r#where).unwrap();
    }

    stand(&mut rig, &now, 95.43, 0.1);
    stand(&mut rig, &now, 95.43, 0.1);
    stand(&mut rig, &now, 95.43, 0.3);
    stand(&mut rig, &now, 95.43, 0.3);
    stand(&mut rig, &now, 95.14, 0.1);
    stand(&mut rig, &now, 95.0, 0.1);
    stand(&mut rig, &now, 95.0, 0.5);
    assert_eq!(said(&rig.backend), ["5 north", "a little west", "lined up"]);
}

/// Walls round the cells from (91, 178) to (100, 185), but for a gap two cells wide at (95,
/// 177) for a big boulder and, where asked, a gate.
fn pen(gate: Option<Cell>) -> Vec<Cell> {
    let walls = (90..102).flat_map(|x| [(x, 177), (x, 186)]).chain(
        [90, 101]
            .into_iter()
            .flat_map(|x| (178..186).map(move |z| (x, z))),
    );
    walls
        .filter(|&cell| cell != (95, 177) && cell != (96, 177) && Some(cell) != gate)
        .collect()
}

/// The street with a big boulder in the way into a pen, and an item ball inside it.
fn pen_field() -> Field {
    let here = street();
    let mut interactables = here.interactables.clone();
    interactables.push(Interactable {
        size: (2, 2),
        code: BIG_BOULDER,
        source: 0x08D1F458,
        move_model: true,
        ..Interactable::new("obstacles", "object", (95, 177))
    });
    interactables.push(Interactable {
        source: place_source(FURNITURE_SOURCE, 1).unwrap(),
        ..Interactable::new("items", "item ball", (95, 182))
    });
    Field {
        interactables,
        ..Field::new(here.zone, here.cell)
    }
}

/// The house front and these walls.
fn walled(more: Vec<Cell>) -> Vec<Cell> {
    let mut walls = house_front();
    walls.extend(more);
    walls
}

/// A big boulder on the street that moves north, or east into a hole.
fn loose() -> Field {
    let here = street();
    Field {
        interactables: vec![
            Interactable {
                size: (2, 2),
                code: BIG_BOULDER,
                source: 0x08D1F458,
                move_model: true,
                pushes: Some(vec![("north", false), ("east", true)]),
                ..Interactable::new("obstacles", "object", (95, 177))
            },
            Interactable {
                source: place_source(FURNITURE_SOURCE, 1).unwrap(),
                ..Interactable::new("items", "item ball", (99, 182))
            },
        ],
        ..Field::new(here.zone, here.cell)
    }
}

/// Orion, 2026-10-11: P left him south of a boulder that moved only east, with no way to
/// tell how to get round to its west. P goes to the nearest side it can be pushed from and says
/// the push; comma and period choose another, and P goes there, round the boulder.
#[test]
fn test_p_goes_to_a_side_a_boulder_is_pushed_from_and_comma_and_period_choose_which() {
    let mut rig = navigator(on(loose()));
    rig.press(ITEM_READ);
    rig.press(PATHFIND);
    rig.press(PATH_NEXT);
    rig.press(PATH_NEXT);
    rig.press(PATH_NEXT);
    rig.press(PATHFIND);
    let way = guidance(&rig);
    let end = *path_of(&way).last().unwrap();
    assert!([(95, 179), (96, 179)].contains(&end));
    assert_eq!(way.unwrap().borrow().push, Some("north"));
    rig.press(PATH_PREVIOUS);
    rig.press(ITEM_NEXT);
    rig.press(PATH_NEXT);
    assert_eq!(
        said(&rig.backend),
        [
            "object, 1 of 2, 5 east 2 south, moveable north or east into a hole",
            "to push east: 2 south, 4 east",
            "push north, 1 of 2",
            "push east into a hole, 2 of 2",
            "push north, 1 of 2",
            "to push north: 4 south, 5 east",
            "push east into a hole, 2 of 2",
            "item ball, 2 of 2, 9 east 7 south",
            "one way to it"
        ]
    );
}

/// Off the way to push it north, the way found again still ends south of it, though its west
/// side, pushed east from, is nearer.
#[test]
fn test_a_way_to_push_a_boulder_found_again_keeps_to_its_side() {
    let mut rig = navigator(on(loose()));
    rig.press(ITEM_READ);
    rig.press(PATH_NEXT);
    rig.press(PATHFIND);
    step(&mut rig, &[(90, 174)]);
    let way = guidance(&rig);
    let end = *path_of(&way).last().unwrap();
    assert!([(95, 179), (96, 179)].contains(&end));
    assert_eq!(way.unwrap().borrow().push, Some("north"));
}

/// Route 104's bush stands in the way to an item ball (omega_ruby_bush.ctm), Rusturf
/// Tunnel's rock in the way on (omega_ruby_smash.ctm, spike 137). P walks round either as round
/// anything standing; Shift and P clears it, and says the move as the player reaches it. One
/// chosen itself is walked up to, with HMs or not.
#[test]
fn test_p_goes_round_a_bush_or_a_rock_and_shift_p_clears_it() {
    for (sprite, r#move) in [(BUSH, "Cut"), (CRACKED_BOULDER, "Rock Smash")] {
        let here = street();
        let mut interactables = here.interactables.clone();
        interactables.push(Interactable {
            code: sprite,
            source: 0x08D1F458,
            move_model: true,
            ..Interactable::new("obstacles", "object", (95, 177))
        });
        let blocked = Field {
            interactables,
            ..Field::new(here.zone, here.cell)
        };
        let mut walls = house_front();
        walls.extend(hedge());
        let mut rig = navigator(Want {
            blocked: walls,
            ..on(blocked)
        });
        rig.press(CATEGORY_PREVIOUS);
        rig.press(PATHFIND);
        let way = path_of(&guidance(&rig));
        assert!(!way.contains(&(95, 177)), "{move}");
        rig.press_key("shift+p");
        assert_eq!(
            last(&rig.backend),
            format!("5 east, 1 south, {move} 1 south, 3 south, 4 east"),
            "{move}"
        );
        let mut walked = cells(91..96, 175);
        walked.push((95, 176));
        step(&mut rig, &walked);
        assert_eq!(
            tail(&rig.backend, 2),
            ["1 south".to_string(), format!("{move} 1 south")],
            "{move}"
        );
        rig.press(CATEGORY_NEXT);
        rig.press(ITEM_READ);
        rig.press_key("shift+p");
        assert_eq!(last(&rig.backend), "object, 1 south", "{move}");
    }
}

/// Fiery Path's boulder fills a path two cells wide (omega_ruby_strength-boulders.ctm): P
/// walks round it, and Shift and P goes through both its rows, said with Strength, where it is
/// the one way in.
#[test]
fn test_p_goes_round_a_big_boulder_and_shift_p_pushes_through_all_of_it() {
    let mut rig = navigator(Want {
        blocked: walled(pen(None)),
        ..on(pen_field())
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(
        last(&rig.backend),
        "5 east, 1 south, then can't continue, object in the way"
    );
    rig.press_key("shift+p");
    assert_eq!(
        last(&rig.backend),
        "5 east, 1 south, Strength 2 south, 4 south"
    );
}

/// Team Magma's Hideout listed every teleporter of the place, most in rooms the player was not
/// in (Orion, 2026-10-11). Walled into the pen, the item ball and the person are hidden, and the
/// boulder in its wall is not; pressed again, all are listed, as they are when pa3ds starts.
#[test]
fn test_ctrl_p_hides_what_shift_p_finds_no_way_to_and_shows_it_again() {
    let mut walls = pen(None);
    walls.extend([(95, 177), (96, 177)]);
    let mut rig = navigator(Want {
        blocked: walled(walls),
        ..on(pen_field())
    });
    rig.press_key("ctrl+p");
    rig.press(CATEGORY_PREVIOUS);
    rig.press_key("ctrl+p");
    rig.press(CATEGORY_NEXT);
    assert_eq!(
        said(&rig.backend),
        [
            "unreachable hidden",
            "obstacles, 1",
            "unreachable shown",
            "people, 1"
        ]
    );
}

/// Shift and P goes to a side a big boulder is pushed from, and this one, moved north or east,
/// is pushed from inside the pen alone: its north side, on the street, is no way to it, and
/// Ctrl and P hides it as Shift and P falls short of it.
#[test]
fn test_ctrl_p_hides_a_big_boulder_whose_push_sides_shift_p_finds_no_way_to() {
    let mut rig = navigator(Want {
        blocked: pen(None),
        ..on(loose())
    });
    rig.press_key("shift+p");
    rig.press_key("ctrl+p");
    rig.press(ITEM_READ);
    assert_eq!(
        said(&rig.backend),
        [
            "5 east, 1 south, then can't continue, object in the way",
            "unreachable hidden",
            "nothing here"
        ]
    );
}

/// Seafloor Cavern (movie36): a boulder pushed the wrong way can shut the way it was pushed to
/// open, so Shift and P goes round one however far that is (Orion, 2026-10-11).
#[test]
fn test_shift_p_goes_round_a_big_boulder_wherever_a_way_round_lies() {
    let mut rig = navigator(Want {
        blocked: walled(pen(Some((95, 186)))),
        ..on(pen_field())
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press_key("shift+p");
    assert!(!last(&rig.backend).contains("Strength"));
    let way = path_of(&guidance(&rig));
    assert_eq!(way.last(), Some(&(95, 182)));
    assert!(
        !way.iter()
            .any(|cell| [(95, 177), (96, 177), (95, 178), (96, 178)].contains(cell))
    );
}

/// Victory Road (movie46): a boulder pushed into the foot of a corridor stuck there, and Shift and
/// P went on saying "Strength 2 south" through it. It goes into a boulder only a way a push moves
/// it, and the pen's one way in is south through it.
#[test]
fn test_shift_p_pushes_a_big_boulder_only_a_way_it_moves() {
    let cases: [Vec<(&'static str, bool)>; 3] = [
        vec![],
        vec![("north", false), ("east", false)],
        vec![("south", false)],
    ];
    for pushes in cases {
        let south = pushes.contains(&("south", false));
        let mut stuck = pen_field();
        let boulder = stuck.interactables.len() - 2;
        stuck.interactables[boulder].pushes = Some(pushes);
        let mut rig = navigator(Want {
            blocked: walled(pen(None)),
            ..on(stuck)
        });
        rig.press(CATEGORY_PREVIOUS);
        rig.press(PATHFIND);
        rig.press_key("alt+p");
        rig.press_key("shift+p");
        let stopped = "5 east, 1 south, then can't continue, object in the way";
        assert_eq!(
            tail(&rig.backend, 3),
            [
                stopped,
                "path off",
                if south {
                    "5 east, 1 south, Strength 2 south, 4 south"
                } else {
                    stopped
                }
            ]
        );
    }
}

/// Ever Grande City's way into Victory Road's lower floor, walked into north off the water below
/// it, with the ground running on over it (movie45).
fn cave_mouth() -> Interactable {
    Interactable {
        size: (3, 1),
        zone: Some(0x7D),
        source: place_source(WARP_SOURCE, 3).unwrap(),
        way_in: Some(pathfind::NORTH),
        ..Interactable::new("exits", "passage", (95, 177))
    }
}

fn cave_cliff() -> Vec<Cell> {
    cells(80..120, 177)
}

fn under_cliff() -> Vec<Cell> {
    (80..120).flat_map(|x| [(x, 178), (x, 179)]).collect()
}

/// movie45: P took the player onto the ground over the cave mouth, "passage, 1 south", where no
/// step south is taken. The way to it ends on the water below, facing north.
#[test]
fn test_a_passage_is_walked_up_to_on_the_side_it_is_walked_into_from() {
    let walls = [house_front(), cave_cliff()].concat();
    let above = Field {
        interactables: vec![cave_mouth()],
        ..Field::new(street().zone, street().cell)
    };
    let mut rig = navigator(Want {
        blocked: walls.clone(),
        sea: under_cliff(),
        ..on(above)
    });
    rig.press(PATHFIND);
    let way = guidance(&rig);
    assert!(way.is_none() || path_of(&way).last().unwrap().1 != 176);
    let heard = last(&rig.backend);
    assert!(heard.contains("can't continue") || heard.starts_with("no path"));
    let below = Field {
        interactables: vec![cave_mouth()],
        ..Field::new(street().zone, (100, 178))
    };
    let mut rig = navigator(Want {
        blocked: walls,
        sea: under_cliff(),
        ..on(below)
    });
    rig.press(PATHFIND);
    let way = guidance(&rig);
    assert!(way.is_some() && path_of(&way).last() == Some(&(97, 178)));
}

/// Three of Rusturf Tunnel's rocks have a sign in their cell and a script of their own, and
/// stand across the tunnel between its halves: they are the rocks Rock Smash breaks to join
/// them, and Shift and P found no way to Verdanturf Town while they were walls (2026-09-29).
#[test]
fn test_a_rock_with_a_sign_under_it_is_smashed_all_the_same() {
    let here = street();
    let mut interactables = here.interactables.clone();
    interactables.push(Interactable {
        code: CRACKED_BOULDER,
        source: 0x08D1F458,
        move_model: true,
        ..Interactable::new("obstacles", "object", (95, 177))
    });
    interactables.push(Interactable {
        source: place_source(FURNITURE_SOURCE, 0).unwrap(),
        ..Interactable::new("signs", "sign", (95, 177))
    });
    let scripted = Field {
        interactables,
        ..Field::new(here.zone, here.cell)
    };
    let mut walls = house_front();
    walls.extend(hedge());
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(scripted)
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press_key("shift+p");
    assert_eq!(
        last(&rig.backend),
        "5 east, 1 south, Rock Smash 1 south, 3 south, 4 east"
    );
}

/// The game puts the player in a door's own cell before the warp, and the Pokémon Center's
/// door said a path to the Poké Mart a moment before it (poke_center_pc.ctm, 2026-09-24).
#[test]
fn test_a_door_walked_into_on_the_way_is_left_to_its_warp() {
    let here = street();
    let mut interactables = here.interactables.clone();
    interactables.push(Interactable {
        zone: Some(0xE1),
        source: place_source(WARP_SOURCE, 1).unwrap(),
        ..Interactable::new("exits", "door", (99, 172))
    });
    let street = Field {
        interactables,
        ..Field::new(here.zone, here.cell)
    };
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street)
    });
    rig.press(PATHFIND);
    step(&mut rig, &[(99, 172)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north"]);
    assert!(rig.nav.guiding());
    rig.field.visit.set(Some((0x08D1FF08, 0xE1)));
    step(&mut rig, &[(4, 9)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north"]);
    assert!(!rig.nav.guiding());
}

#[test]
fn test_a_warp_ends_the_path_quietly_and_a_field_rebuilt_in_place_does_not() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    rig.field.visit.set(Some((0x08D1FF08, 0x6)));
    step(&mut rig, &[(90, 175)]);
    step(
        &mut rig,
        &[(91, 175), (92, 175), (93, 175), (94, 175), (95, 175)],
    );
    assert_eq!(said(&rig.backend), ["5 east, 2 north", "2 north"]);
    rig.field.visit.set(Some((0x08D1FF08, 0xDF)));
    step(&mut rig, &[(16, 18)]);
    assert_eq!(said(&rig.backend), ["5 east, 2 north", "2 north"]);
    assert!(!rig.nav.guiding());
}

#[test]
fn test_beside_the_thing_p_says_where_it_is() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    walk_to(&rig, (95, 173));
    rig.press(PATHFIND);
    assert_eq!(said(&rig.backend), ["door, 1 north"]);
    assert!(!rig.nav.guiding());
}

/// The user, 2026-10-03: "find a path to the closest it can be before saying can't
/// continue", where P on Route 119 said "no path to edge" and nothing more.
#[test]
fn test_with_no_way_there_p_goes_as_near_as_it_can_and_says_it_can_t_continue() {
    let ring: Vec<Cell> = (98..103)
        .flat_map(|x| (178..183).map(move |z| (x, z)))
        .filter(|&(x, z)| (x - 100i64).abs().max((z - 180i64).abs()) == 2)
        .collect();
    let mut walls = house_front();
    walls.extend(ring);
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(street())
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(
        said(&rig.backend),
        ["people, 1", "4 south, 7 east, then can't continue"]
    );
    let mut walked: Vec<Cell> = (176..180).map(|z| (90, z)).collect();
    walked.extend((91..98).map(|x| (x, 179)));
    step(&mut rig, &walked);
    assert_eq!(
        from(&rig.backend, 2),
        ["7 east", "can't continue, person, 3 east 1 south"]
    );
    assert!(!rig.nav.guiding());
}

/// Spike 165: under Route 126 the player's light rose into a pond, and the way into Sootopolis
/// City is in a pocket of its own. P goes to the light in theirs whose sea above reaches a dive
/// spot over the thing's pocket, and says that spot from there.
#[test]
fn test_under_the_water_p_goes_to_the_light_whose_sea_reaches_the_things_pocket() {
    let wall: Vec<Cell> = (80..101)
        .flat_map(|x| (160..200).map(move |z| (x, z)))
        .filter(|&(x, z)| x == 80 || x == 100 || z == 160 || z == 199)
        .collect();
    let (pond, sea, theirs) = ((85, 168), (90, 170), (110, 175));
    let under = Field {
        interactables: vec![Interactable {
            size: (5, 1),
            zone: Some(0x4C),
            source: 0,
            ..Interactable::new("exits", "entrance", (115, 180))
        }],
        ..Field::new(0x42, (85, 170))
    };
    let open: Vec<Cell> = (80..120)
        .flat_map(|x| (160..200).map(move |z| (x, z)))
        .filter(|cell| !wall.contains(cell))
        .collect();
    let mut rig = navigator(Want {
        blocked: wall,
        sea: open,
        ..on(under)
    });
    *rig.field.surfaces.borrow_mut() = IndexMap::from([
        (pond, Rc::new(HashSet::from([pond]))),
        (sea, Rc::new(HashSet::from([sea, theirs]))),
        (theirs, Rc::new(HashSet::from([sea, theirs]))),
    ]);
    rig.press(PATHFIND);
    assert_eq!(
        said(&rig.backend),
        ["5 east, then can't continue, surface and go to the dive spot 20 east 5 south"]
    );
    step(&mut rig, &cells(86..91, 170));
    assert_eq!(
        last(&rig.backend),
        "can't continue, surface and go to the dive spot 20 east 5 south, entrance, 25 east 10 south"
    );
}

/// Spike 159: from the foot of Route 119 the way north crosses a bridge high over the path,
/// and its decks walked at the player's height cut it. The way found at the bridge's height is
/// followed on that ground: at the player's own the bridge is a wall, and every step would
/// find the way again.
#[test]
fn test_where_no_way_is_found_at_the_players_height_the_decks_are_walked_at_another() {
    let wall: Vec<Cell> = (93..108)
        .flat_map(|x| (173..188).map(move |z| (x, z)))
        .filter(|&(x, z)| (x - 100i64).abs().max((z - 180i64).abs()) == 7)
        .collect();
    let mut walls = house_front();
    walls.extend(wall);
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(street())
    });
    *rig.field.decks.borrow_mut() = vec![(99.0, HashSet::from([(93, 177)]))];
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(
        said(&rig.backend),
        ["people, 1", "2 south, 10 east, 2 south"]
    );
    step(&mut rig, &[(90, 176), (90, 177), (91, 177), (91, 178)]);
    assert_eq!(
        from(&rig.backend, 2),
        ["10 east", "1 north, 9 east, 2 south"]
    );
    let way = guidance(&rig);
    assert!(way.is_some() && way.unwrap().borrow().height == Some(99.0));
}

/// Spike 167: Sootopolis Gym's lower floor lies under the corridor to its way out, and a stair
/// climbs from it to the entrance's room. From the lower floor P led the player to the cell
/// under the way out, and on the corridor the Lass pacing below stood in the player's way.
#[test]
fn test_floor_over_floor_each_is_walked_as_itself_and_joined_by_the_stairs() {
    let way_out = Interactable {
        zone: Some(0x14),
        source: place_source(WARP_SOURCE, 0).unwrap(),
        height: Some(198.0),
        ..Interactable::new("exits", "way out", (100, 183))
    };
    let lass = Interactable {
        source: 0x08D269E8,
        move_model: true,
        height: Some(-18.0),
        ..Interactable::new("people", "person", (100, 172))
    };
    let mut rig = navigator(Want {
        exits: true,
        ..on(Field {
            interactables: vec![way_out, lass],
            ..Field::new(0x1BC, (100, 175))
        })
    });
    {
        let mut floors = rig.field.floors.borrow_mut();
        for x in 96..101 {
            floors.insert((x, 170), (None, Some(198.0)));
            floors.insert((x, 182), (Some(if x == 96 { -9.0 } else { -18.0 }), None));
        }
        for z in 171..182 {
            floors.insert((96, z), (Some(189.0 - 18.0 * (z - 171) as f64), None));
        }
        for z in 171..183 {
            floors.insert((100, z), (Some(-18.0), Some(198.0)));
        }
        floors.insert((100, 183), (None, Some(198.0)));
    }
    rig.field.level.set(Some(-18.0));
    rig.press(PATHFIND);
    assert_eq!(
        said(&rig.backend),
        ["7 south, 4 west, 12 north, 4 east, 13 south"]
    );
    walk_to(&rig, (100, 171));
    rig.field.level.set(Some(198.0));
    rig.press(PATHFIND);
    assert_eq!(last(&rig.backend), "12 south");
}

/// Spike A: outside Sky Pillar the terraces stand 108 apart on three floors, and the one stair
/// up to Zinnia's goes from floor 0 at 108 onto floor 1 at 216. P went only as far as the sand
/// under her, "then can't continue".
#[test]
fn test_a_ladder_climbs_from_one_floor_onto_another() {
    let zinnia = Interactable {
        source: 0x08D269E8,
        move_model: true,
        height: Some(216.0),
        ..Interactable::new("people", "person", (105, 174))
    };
    let mut rig = navigator(on(Field {
        interactables: vec![zinnia],
        ..Field::new(0xB0, (98, 175))
    }));
    {
        let mut floors = rig.field.floors.borrow_mut();
        for x in 96..101 {
            floors.insert((x, 175), (Some(108.0), None));
        }
        for x in 101..106 {
            floors.insert((x, 175), (None, Some(216.0)));
        }
        floors.insert((105, 174), (None, Some(216.0)));
    }
    rig.field.level.set(Some(108.0));
    rig.press(CATEGORY_NEXT);
    rig.backend.spoken.borrow_mut().clear();
    rig.press(PATHFIND);
    assert_eq!(said(&rig.backend), ["2 east, then can't continue"]);
    rig.field.ladders.borrow_mut().extend([
        (((100, 175), (101, 175)), (108.0, 216.0)),
        (((101, 175), (100, 175)), (216.0, 108.0)),
    ]);
    rig.press(PATHFIND);
    assert_eq!(last(&rig.backend), "7 east");
}

/// movie47 (Orion, 2026-10-15): in Victory Road, a stacked ground, Ctrl and P hid none of the
/// 8 things Shift and P fell short of, its walk taking a step onto any floor of a cell. Outside
/// Sky Pillar Zinnia's terrace stands 108 over the player's, and is hidden until the stair up to
/// it is there (spike 193).
#[test]
fn test_ctrl_p_hides_what_is_on_a_floor_no_step_climbs_to() {
    let zinnia = Interactable {
        source: 0x08D269E8,
        move_model: true,
        height: Some(216.0),
        ..Interactable::new("people", "person", (105, 174))
    };
    let mut rig = navigator(on(Field {
        interactables: vec![zinnia],
        ..Field::new(0xB0, (98, 175))
    }));
    {
        let mut floors = rig.field.floors.borrow_mut();
        for x in 96..101 {
            floors.insert((x, 175), (Some(108.0), None));
        }
        for x in 101..106 {
            floors.insert((x, 175), (None, Some(216.0)));
        }
        floors.insert((105, 174), (None, Some(216.0)));
    }
    rig.field.level.set(Some(108.0));
    rig.press_key("ctrl+p");
    rig.press(ITEM_READ);
    rig.field.ladders.borrow_mut().extend([
        (((100, 175), (101, 175)), (108.0, 216.0)),
        (((101, 175), (100, 175)), (216.0, 108.0)),
    ]);
    walk_to(&rig, (99, 175));
    rig.press(ITEM_READ);
    assert_eq!(
        said(&rig.backend),
        [
            "unreachable hidden",
            "nothing here",
            "person, 1 of 1, 6 east 1 north"
        ]
    );
}

/// Fortree City's bridge at 103 ends on a platform at 103, and under its last cell the ground
/// is open with no height. P went onto the platform and back onto the ground under the bridge,
/// "8 east, 2 west, 9 north" (omega_ruby_tree-city-regression.ctm, 2026-10-06).
#[test]
fn test_a_way_does_not_drop_off_a_bridge_through_a_floor_with_no_height() {
    let girl = Interactable {
        source: 0x08D269E8,
        move_model: true,
        height: Some(0.0),
        ..Interactable::new("people", "person", (99, 172))
    };
    let mut rig = navigator(on(Field {
        interactables: vec![girl],
        ..Field::new(0x11, (97, 175))
    }));
    {
        let mut floors = rig.field.floors.borrow_mut();
        for x in 96..101 {
            floors.insert((x, 175), (Some(0.0), Some(103.0)));
        }
        floors.insert((100, 175), (None, Some(103.0)));
        rig.field.bare.borrow_mut().insert(((100, 175), 0));
        for x in 101..104 {
            floors.insert((x, 175), (Some(103.0), None));
        }
        floors.insert((99, 174), (Some(0.0), None));
        floors.insert((99, 173), (Some(0.0), None));
    }
    rig.field.level.set(Some(103.0));
    rig.press(CATEGORY_NEXT);
    rig.backend.spoken.borrow_mut().clear();
    rig.press(PATHFIND);
    assert_eq!(said(&rig.backend), ["3 east, then can't continue"]);
}

/// Spike A: outside Sky Pillar the sand runs on under the stair from the terrace at 324 down to
/// Zinnia's at 216, and P walked up the sand and onto her terrace by that stair, "6 south".
#[test]
fn test_a_ladder_is_not_climbed_from_the_floor_under_it() {
    let zinnia = Interactable {
        source: 0x08D269E8,
        move_model: true,
        height: Some(216.0),
        ..Interactable::new("people", "person", (105, 174))
    };
    let mut rig = navigator(on(Field {
        interactables: vec![zinnia],
        ..Field::new(0xB0, (105, 177))
    }));
    rig.field.floors.borrow_mut().extend([
        ((105, 177), (Some(0.0), None)),
        ((105, 176), (Some(0.0), Some(324.0))),
        ((105, 175), (Some(0.0), Some(216.0))),
        ((105, 174), (None, Some(216.0))),
    ]);
    rig.field.ladders.borrow_mut().extend([
        (((105, 176), (105, 175)), (324.0, 216.0)),
        (((105, 175), (105, 176)), (216.0, 324.0)),
    ]);
    rig.field.level.set(Some(0.0));
    rig.press(CATEGORY_NEXT);
    rig.backend.spoken.borrow_mut().clear();
    rig.press(PATHFIND);
    assert_eq!(said(&rig.backend), ["2 north, then can't continue"]);
}

/// Spike 167: each of Sootopolis Gym's corridors up to Wallace is shut until the room of thin ice
/// below it is cleared, by stepping on every tile once and last on an exit, and P to him stopped
/// at the shut corridor. The way goes over the ice instead, round the floor in its middle that is
/// no ice, and says the room is cleared where it ends.
#[test]
fn test_a_way_shut_beyond_thin_ice_goes_over_every_tile_and_clears_it() {
    let sign = Interactable {
        source: place_source(FURNITURE_SOURCE, 0).unwrap(),
        ..Interactable::new("signs", "sign", (96, 165))
    };
    let mut ways: HashSet<Cell> = (95..98)
        .flat_map(|x| (170..173).map(move |z| (x, z)))
        .collect();
    ways.extend([
        (95, 173),
        (95, 174),
        (95, 175),
        (96, 169),
        (96, 168),
        (96, 166),
    ]);
    let walls: Vec<Cell> = (80..120)
        .flat_map(|x| (160..200).map(move |z| (x, z)))
        .filter(|cell| !ways.contains(cell))
        .collect();
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(Field {
            interactables: vec![sign],
            ..Field::new(0x1BC, (95, 175))
        })
    });
    *rig.field.ice.borrow_mut() = vec![IceRoom::new(
        (95, 170),
        (3, 3),
        254.0,
        vec![0, 0, 0, 0, 2, 0, 0, 0, 0],
        vec![(95, 170), (96, 170), (97, 170)],
    )];
    rig.press(CATEGORY_NEXT);
    rig.backend.spoken.borrow_mut().clear();
    rig.press(PATHFIND);
    assert_eq!(
        said(&rig.backend),
        ["3 north, 2 east, 1 north, 2 west, 1 north, 2 east, and the ice is cleared"]
    );
    step(
        &mut rig,
        &[
            (95, 174),
            (95, 173),
            (95, 172),
            (96, 172),
            (97, 172),
            (97, 171),
            (96, 171),
            (95, 171),
            (95, 170),
            (96, 170),
            (97, 170),
        ],
    );
    assert_eq!(last(&rig.backend), "ice cleared, sign, 1 west 5 north");
}

/// Shift and P to the sign past the shut corridor goes over the ice and clears it, so Ctrl
/// and P keeps the sign, though no step reaches it before the room is cleared.
#[test]
fn test_ctrl_p_keeps_what_shift_p_reaches_by_clearing_thin_ice() {
    let sign = Interactable {
        source: place_source(FURNITURE_SOURCE, 0).unwrap(),
        ..Interactable::new("signs", "sign", (96, 165))
    };
    let mut ways: HashSet<Cell> = (95..98)
        .flat_map(|x| (170..173).map(move |z| (x, z)))
        .collect();
    ways.extend([
        (95, 173),
        (95, 174),
        (95, 175),
        (96, 169),
        (96, 168),
        (96, 166),
    ]);
    let walls: Vec<Cell> = (80..120)
        .flat_map(|x| (160..200).map(move |z| (x, z)))
        .filter(|cell| !ways.contains(cell))
        .collect();
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(Field {
            interactables: vec![sign],
            ..Field::new(0x1BC, (95, 175))
        })
    });
    *rig.field.ice.borrow_mut() = vec![IceRoom::new(
        (95, 170),
        (3, 3),
        254.0,
        vec![0, 0, 0, 0, 2, 0, 0, 0, 0],
        vec![(95, 170), (96, 170), (97, 170)],
    )];
    rig.press_key("ctrl+p");
    rig.press(ITEM_READ);
    assert_eq!(
        said(&rig.backend),
        ["unreachable hidden", "sign, 1 of 1, 1 east 10 north"]
    );
}

/// Spike 160: Fortree City's invisible Kecleon stands in the one way to the Gym, and the way as
/// near as the player could get as the crow flies ended behind a wall from it, on a walkway 7
/// cells south of the door. The way goes up to whoever stands in it instead, and says them.
#[test]
fn test_a_person_in_the_one_way_is_walked_up_to_and_said() {
    let ring: Vec<Cell> = (98..103)
        .flat_map(|x| (178..183).map(move |z| (x, z)))
        .filter(|&(x, z)| (x - 100i64).abs().max((z - 180i64).abs()) == 2 && (x, z) != (98, 180))
        .collect();
    let guard = Interactable {
        source: 0x08D1F000,
        move_model: true,
        ..Interactable::new("people", "person", (98, 180))
    };
    let here = street();
    let mut interactables = here.interactables.clone();
    interactables.push(guard);
    let street = Field {
        interactables,
        ..here
    };
    let mut walls = house_front();
    walls.extend(ring);
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(street)
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(ITEM_NEXT);
    rig.press(PATHFIND);
    assert_eq!(
        last(&rig.backend),
        "5 south, 7 east, then can't continue, person in the way"
    );
}

/// Route 7: the way west went through Shauna, standing in the open, on its way to the
/// Snorlax filling the road, and P said she was in the way (2026-10-19). The way goes round
/// whoever it can, and says who it cannot.
#[test]
fn test_somebody_the_way_goes_round_is_not_in_the_way() {
    let ring: Vec<Cell> = (98..103)
        .flat_map(|x| (178..183).map(move |z| (x, z)))
        .filter(|&(x, z)| (x - 100i64).abs().max((z - 180i64).abs()) == 2 && (x, z) != (98, 180))
        .collect();
    let guard = Interactable {
        source: 0x08D1F000,
        move_model: true,
        ..Interactable::new("people", "Snorlax", (98, 180))
    };
    let shauna = Interactable {
        source: 0x08D1FAB0,
        move_model: true,
        ..Interactable::new("people", "Shauna", (94, 180))
    };
    let here = street();
    let mut interactables = here.interactables.clone();
    interactables.extend([guard, shauna]);
    let street = Field {
        interactables,
        ..here
    };
    let mut walls = house_front();
    walls.extend(ring);
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(street)
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(ITEM_NEXT);
    rig.press(ITEM_NEXT);
    rig.press(PATHFIND);
    let said = last(&rig.backend);
    assert!(
        said.ends_with("then can't continue, Snorlax in the way"),
        "{said}"
    );
}

/// Spike 161: Fortree Gym's doors stand across its ways, and are walked through by walking into
/// an arm, which turns the door. A wall with one gap, a door's arm standing across it.
#[test]
fn test_p_walks_through_a_rotating_door_and_says_the_push() {
    let wall: Vec<Cell> = (80..120).filter(|&x| x != 94).map(|x| (x, 177)).collect();
    let mut walls = house_front();
    walls.extend(wall);
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(street())
    });
    let door = RotatingDoor::new((95, 178), IndexMap::from([(pathfind::WEST, 1)]));
    *rig.field.edit.borrow_mut() = Some(Rc::new(move |ground: Ground| Ground {
        doors: vec![door.clone()],
        ..ground
    }));
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert!(last(&rig.backend).contains("push 1 south"));
}

/// A Linoone the DexNav found on Route 120 (movie26).
fn hidden() -> Interactable {
    Interactable {
        code: 264,
        source: 0x08D374E4,
        ..Interactable::new("pokémon", "hidden Pokémon", (98, 175))
    }
}

/// `replace(TOWN, interactables=(*TOWN.interactables, HIDDEN))`.
fn town_and_hidden() -> Field {
    let mut where_ = town();
    where_.interactables.push(hidden());
    where_
}

/// Orion, movie26: out of touch mode and along the list to it took long enough to lose it.
#[test]
fn test_a_hidden_pokemon_found_is_guided_to_and_p_is_on_it() {
    let r#where = town_and_hidden();
    let mut rig = navigator(Want {
        r#where: Some(r#where.clone()),
        exits: true,
        ..Want::default()
    });
    rig.nav.found(&r#where, &hidden(), true).unwrap();
    let spoken = rig.backend.spoken.borrow().clone();
    assert_eq!(spoken.len(), 1);
    let (first, interrupted) = &spoken[0];
    assert!(first.starts_with("hidden Pokémon, ") && !interrupted);
    assert!(rig.nav.guiding());
    rig.press(PATHFIND);
    assert_eq!(
        last(&rig.backend),
        first.strip_prefix("hidden Pokémon, ").unwrap()
    );
}

/// Walking away from a hidden Pokémon the DexNav found said a new way at every step
/// (movie26's replay); P asks for it, and is answered.
#[test]
fn test_walking_off_a_way_nobody_asked_for_finds_it_again_quietly() {
    let r#where = town_and_hidden();
    let mut rig = navigator(on(r#where.clone()));
    rig.nav.found(&r#where, &hidden(), true).unwrap();
    step(&mut rig, &[(93, 172), (92, 172)]);
    assert!(rig.backend.spoken.borrow().len() == 1 && rig.nav.guiding());
    assert!(
        rig.log
            .borrow()
            .iter()
            .any(|line| line.starts_with("path, quietly: "))
    );
    rig.press(PATHFIND);
    assert_eq!(rig.backend.spoken.borrow().len(), 2);
}

#[test]
fn test_a_hidden_pokemon_found_is_left_alone_unless_asked() {
    let r#where = town_and_hidden();
    let mut rig = navigator(Want {
        r#where: Some(r#where.clone()),
        navigation: Some(Navigation {
            guide_to_hidden: false,
            ..Navigation::default()
        }),
        ..Want::default()
    });
    rig.nav.found(&r#where, &hidden(), true).unwrap();
    assert!(rig.backend.spoken.borrow().is_empty() && !rig.nav.guiding());
}

#[test]
fn test_with_nowhere_nearer_p_says_there_is_no_path() {
    let mut walls = house_front();
    walls.extend([(89, 175), (91, 175), (90, 174), (90, 176)]);
    let mut rig = navigator(Want {
        blocked: walls,
        ..on(street())
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    assert_eq!(said(&rig.backend), ["people, 1", "no path to person"]);
    assert!(!rig.nav.guiding());
}

/// omega_ruby_movil.ctm (spike 142): P again halfway up Route 110 found no new way, said "no
/// path to path", and left nothing to follow. The way being walked stays, and P says what is
/// left of it.
#[test]
fn test_p_again_that_finds_no_way_keeps_the_way_being_walked_and_says_the_rest() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(PATHFIND);
    step(&mut rig, &[(91, 175), (92, 175), (93, 175)]);
    let held = guidance(&rig).unwrap();
    // Python makes `pathfind.find` answer None. `find` is called by name here, so the door's
    // approaches are made cells a way only goes round (`stepped`, which `open` ignores): the door
    // has no goal left to walk to, and `find` answers None for want of one, with every cell of
    // the way still open for `follow`.
    *rig.field.edit.borrow_mut() = Some(Rc::new(|mut ground: Ground| {
        ground.stepped.extend([(95, 173), (95, 171)]);
        ground
    }));
    rig.press(PATHFIND);
    assert_eq!(said(&rig.backend), ["5 east, 2 north", "2 east, 2 north"]);
    assert!(rig.nav.guiding());
    // Not in the Python: with the real `find` P again finds the same way anew and says the same
    // words, so only the way kept tells the branch the test is for from the one beside it.
    assert!(
        Rc::ptr_eq(&held, rig.nav.guide.guidance.as_ref().unwrap()),
        "the way being walked is kept"
    );
    step(&mut rig, &[(94, 175), (95, 175)]);
    assert_eq!(last(&rig.backend), "2 north");
}

#[test]
fn test_a_thing_gone_while_the_player_was_off_the_path_is_said_gone() {
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(street())
    });
    rig.press(CATEGORY_PREVIOUS);
    rig.press(PATHFIND);
    let here = street();
    *rig.field.r#where.borrow_mut() = Some(Field {
        interactables: here.interactables[..1].to_vec(),
        ..Field::new(here.zone, here.cell)
    });
    step(&mut rig, &[(90, 174)]);
    assert_eq!(last(&rig.backend), "person gone");
    assert!(!rig.nav.guiding());
}

/// `DEFAULT_BINDINGS[mode][key]`.
fn bound(mode: &str, key: &str) -> &'static str {
    // The bindings live as long as the program; a `LazyLock` hands out `&'static` to them.
    let keys = &DEFAULT_BINDINGS[mode];
    keys[key].as_str()
}

#[test]
fn test_the_users_keys_map_to_the_actions() {
    assert_eq!(
        "iojlk"
            .chars()
            .map(|key| bound(LIST, &key.to_string()))
            .collect::<Vec<_>>(),
        [
            CATEGORY_PREVIOUS,
            CATEGORY_NEXT,
            ITEM_PREVIOUS,
            ITEM_NEXT,
            ITEM_READ
        ]
    );
    assert_eq!(
        "o;lk"
            .chars()
            .map(|key| bound(DIRECTION, &key.to_string()))
            .collect::<Vec<_>>(),
        [NORTH, EAST, SOUTH, WEST]
    );
    for mode in [LIST, DIRECTION] {
        assert_eq!(
            "[]'p"
                .chars()
                .map(|key| bound(mode, &key.to_string()))
                .collect::<Vec<_>>(),
            [CATEGORY_PREVIOUS, CATEGORY_NEXT, MODE_TOGGLE, PATHFIND],
            "{mode}"
        );
    }
    let mut expected: HashSet<String> = "iojlk[];'prc,.u"
        .chars()
        .map(|key| key.to_string())
        .collect();
    expected.extend(
        [
            "shift+k", "shift+j", "shift+p", "alt+p", "ctrl+p", "shift+r", "shift+c",
        ]
        .map(str::to_string),
    );
    assert_eq!(bound_keys(&DEFAULT_BINDINGS), expected);
    for mode in [LIST, DIRECTION, TOUCH] {
        assert_eq!(
            [",", "."].map(|key| bound(mode, key)),
            [PATH_PREVIOUS, PATH_NEXT],
            "{mode}"
        );
    }
    assert_eq!(
        ["j", "l", "i", "o", "k", "u"].map(|key| bound(TOUCH, key)),
        [
            ITEM_PREVIOUS,
            ITEM_NEXT,
            ROW_PREVIOUS,
            ROW_NEXT,
            ITEM_READ,
            TAP
        ]
    );
    for mode in [LIST, DIRECTION, TOUCH] {
        assert_eq!(bound(mode, "'"), MODE_TOGGLE, "{mode}");
    }
    for mode in [LIST, DIRECTION] {
        assert_eq!(bound(mode, "shift+k"), RENAME, "{mode}");
        assert_eq!(bound(mode, "shift+j"), RENAME_SPEAKER, "{mode}");
        assert_eq!(bound(mode, "shift+p"), PATHFIND_HM, "{mode}");
        assert_eq!(bound(mode, "r"), OUR_SIDE, "{mode}");
        assert_eq!(bound(mode, "shift+r"), ENEMY_SIDE, "{mode}");
    }
    for mode in [LIST, DIRECTION, TOUCH] {
        assert_eq!(bound(mode, "c"), COORDINATES, "{mode}");
        assert_eq!(bound(mode, "shift+c"), ITEM_COORDINATES, "{mode}");
        assert_eq!(bound(mode, "alt+p"), PATHFIND_OFF, "{mode}");
    }
}

/// M was the mode toggle for a day, and M is this machine's Start (`pa3ds.core.keyfile`): a key
/// pa3ds takes is a button the game never gets. The C-stick, the debug button and GPIO 14 are
/// the ones ORAS does not use, so those are the only overlaps the defaults are allowed: P, the
/// way there, is GPIO 14 here.
#[test]
fn test_no_key_of_the_default_bindings_is_a_button_the_player_presses() {
    // `read_text` reads a file with universal newlines.
    let config = include_str!("fake_qt_config.ini")
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let taken = keyfile::taken(&DEFAULT_BINDINGS, &keyfile::emulator_keys(&config).unwrap());
    let controls: HashSet<String> = taken.into_iter().map(|(control, _, _)| control).collect();
    assert_eq!(
        controls,
        HashSet::from([
            "the C-stick".to_string(),
            "the debug button".to_string(),
            "the GPIO 14 button".to_string()
        ])
    );
}

#[test]
fn test_virtual_keys_are_named_as_their_letters_and_us_punctuation() {
    assert!(key_name(0x4B) == "k" && key_name(0x31) == "1");
    assert_eq!(
        [0xBA, 0xDB, 0xDD].map(key_name),
        [";".to_string(), "[".to_string(), "]".to_string()]
    );
    assert_eq!(key_name(0x70), "vk0x70");
}

/// Spike 102: on Lumiose's boulevards the chunks mark every cell a wall, so a way cannot be
/// found; the thing and its bearing still can, and the player hears why there is no more.
#[test]
fn test_p_on_a_rail_says_where_the_thing_is_and_that_no_route_is_known() {
    let here = street();
    let rail = Field {
        on_rail: true,
        ..here
    };
    let mut rig = navigator(on(rail));
    rig.press_key("p");
    assert_eq!(
        last(&rig.backend),
        "door, 5 east 3 north, no route on these streets"
    );
    assert!(!rig.nav.guiding());
}

/// Spike 105: on Lumiose's boulevards up walks the way the camera looks and a held left or
/// right turns it, so P says the holds and the steps, and the list says where things are from
/// the way the camera looks.
#[test]
fn test_on_a_rail_the_way_and_the_bearing_are_said_from_the_camera() {
    let rail = Field {
        heading: Some("east"),
        bearing: Some(92.0),
        ..street()
    };
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(rail)
    });
    rig.press_key("p");
    assert_eq!(last(&rig.backend), "5 forward, hold left, 2 forward");
    rig.press(ITEM_PREVIOUS);
    assert_eq!(last(&rig.backend), "person, 2 of 2, 10 ahead 5 right");
}

/// Glittering Cave (spike 239): left and right turn the player and the camera swings after them,
/// so P says each turn, then the steps forward.
#[test]
fn test_under_a_camera_that_turns_with_the_player_p_says_the_turns() {
    let cave = Field {
        heading: Some("north"),
        bearing: Some(352.3),
        turning: true,
        ..street()
    };
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(cave)
    });
    rig.press_key("p");
    assert_eq!(
        last(&rig.backend),
        "turn right, 5 forward, turn left, 2 forward"
    );
}

/// A tap of left or right leaves the camera between two ways, and up walks the player across
/// the street at its angle (spike 105).
#[test]
fn test_on_a_rail_a_camera_left_at_an_angle_is_straightened_first() {
    let rail = Field {
        heading: Some("east"),
        bearing: Some(60.0),
        ..street()
    };
    let mut rig = navigator(Want {
        blocked: house_front(),
        ..on(rail)
    });
    rig.press_key("p");
    assert_eq!(
        last(&rig.backend),
        "camera angled, hold left, hold right, 5 forward, hold left, 2 forward"
    );
}

/// A hold turns the camera and a tap leaves it partway: once it has stopped, the player
/// standing still hears the way from where it looks now, and nothing while it still turns
/// (spike 105).
#[test]
fn test_the_way_is_said_again_once_the_camera_stops_somewhere_new() {
    let now = Rc::new(Slot::new(0.0));
    let rail = Field {
        heading: Some("east"),
        bearing: Some(90.0),
        ..street()
    };
    let clock = now.clone();
    let mut rig = navigator(Want {
        blocked: house_front(),
        clock: Rc::new(move || clock.get()),
        ..on(rail.clone())
    });
    rig.press_key("p");

    /// The camera at `bearing` for `seconds`, the runtime polling every twentieth.
    fn camera(rig: &mut Rig, now: &Slot<f64>, rail: &Field, bearing: f64, seconds: f64) {
        *rig.field.r#where.borrow_mut() = Some(Field {
            heading: Some(nearest(bearing).unwrap()),
            bearing: Some(bearing),
            ..rail.clone()
        });
        for _ in 0..py::round(seconds * 20.0).unwrap() {
            now.set(now.get() + 0.05);
            let r#where = rig.field.poll().unwrap().unwrap();
            rig.nav.follow(&r#where).unwrap();
        }
    }

    camera(&mut rig, &now, &rail, 89.0, 1.0);
    assert_eq!(said(&rig.backend), ["5 forward, hold left, 2 forward"]);
    for bearing in [60.0, 30.0, 5.0] {
        camera(&mut rig, &now, &rail, bearing, 0.1);
    }
    assert_eq!(said(&rig.backend).len(), 1);
    camera(&mut rig, &now, &rail, 1.0, 0.5);
    assert_eq!(
        last(&rig.backend),
        "hold right, 5 forward, hold left, 2 forward"
    );
    camera(&mut rig, &now, &rail, 65.0, 0.5);
    assert_eq!(
        last(&rig.backend),
        "camera angled, hold left, hold right, 5 forward, hold left, 2 forward"
    );
    assert_eq!(said(&rig.backend).len(), 3);
}

/// For when the line was missed, and while the HP box is off the screen (ADR-0015; the
/// user's keys, 2026-09-29). A key's answer interrupts.
#[test]
fn test_r_says_our_side_again_and_shift_r_the_enemys() {
    let backend = Rc::new(Recorder::default());
    let sides: Sides = Rc::new(|ours: bool| {
        Ok(Some(
            if ours {
                "Blaze♂ L10 20/30"
            } else {
                "Zigzagoon♂ L2 50%"
            }
            .to_string(),
        ))
    });
    let mut nav = Navigator::new(
        Rc::new(FakeField::new(Some(town()), &[], &[], &[], &[])),
        Rc::new(Speech::plain(backend.clone())),
        Rc::new(|_line: &str| {}),
        Options {
            sides: Some(sides),
            ..Options::default()
        },
    );
    nav.press(OUR_SIDE).unwrap();
    nav.press(ENEMY_SIDE).unwrap();
    assert_eq!(
        heard(&backend),
        [
            ("Blaze♂ L10 20/30".to_string(), true),
            ("Zigzagoon♂ L2 50%".to_string(), true)
        ]
    );
}

/// A press is never silent: the player cannot tell a key that did nothing from one that was
/// not heard.
#[test]
fn test_the_sides_keys_say_no_battle_out_of_one() {
    let mut rig = navigator(Want {
        r#where: None,
        ..Want::default()
    });
    rig.press(OUR_SIDE);
    rig.press(ENEMY_SIDE);
    assert_eq!(said(&rig.backend), ["no battle", "no battle"]);
}
