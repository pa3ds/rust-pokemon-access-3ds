//! Sun's field: where the player is, and what is on the map worth walking to (spike 113).
//!
//! Sun has no zone file in the memory pa3ds may read, no pool of models at a fixed stride and no
//! chunks of cells: the field is objects, each a small allocation (ADR-0035), and the ROM's event
//! data (`zones`). Everything here is read to the byte, and only inside a block the last sweep read.
//!
//! ```text
//! Field::MoveModel::FieldMoveModelManager  (heap 3)
//!     +0x0c -> the player's model, +0x10 to +0x88 -> the other 31 models
//! Field::MoveModel::FieldMoveModel, FieldMoveModelPlayer
//!     +0x20 -> its scene node, zero while the slot is free
//!     +0x30 f32 x, y, z: where the save put it, and zero all through a new game
//!     +0x6c u32 the model it is drawn as: 0x77, 0x04, 0x21 for the bedroom's three
//!     +0x3a4 f32 x, +0x3a8 y, +0x3ac z: where it stands now; +0x3b0 where it stood a frame
//!            before
//! Field::Fieldmap::State  (heap 32)
//!     +0x14 -> the field, which has no class of its own, in a block of 0x1a0 of heap 0x21,
//!              whose own blocks a walk passes over (`heaps.UNREAD`):
//!         +0x78 -> the area loaded, which has none either: +0x00 u32 its number in the ROM's
//!                  event data, 21 for the player's house
//!         +0x7c u16 the zone the player is in: 0x1d in the living room, 0x1a in the bedroom;
//!               +0x7e is 1 on Route 1's second zone and 0 elsewhere so far, not known
//! Field::FieldBgEventActor, FieldMapJumpActor, FieldItemActor  (heap 0x45)
//!     +0x14 f32 x, +0x18 y, +0x1c z: where the record it was built from places it, the
//!     record's own floats; +0x20 -> the record, in the area's event data, a block of 8 KB
//!     that is not read: the ROM has the same file (`zones`)
//! ```
//!
//! What each is comes from its record: a warp's type says a door from the mat out or the path into
//! the next area, a thing's kind a signpost from anything else, and an item actor is a ball still
//! on the ground, the ones picked up having none.
//!
//! x is east and z south, as in ORAS: the bedroom's TV is north of the player and the globe on the
//! desk east of it, as the screenshots have them. Sun has no grid: people and the player stand
//! anywhere, and a thing reaches as far as its shape. So a **cell** here is `CELL` world units,
//! about the width of the player: a thing is read from 40 to 100 units from its centre, so a cell
//! that size keeps "here" within reach of it, and the bedroom is 14 cells across, as an ORAS room
//! is about as many. The player walked about 400 units a second across the bedroom in sun_1.ctm,
//! eight cells a second.
//!
//! Measured over sun_1.ctm from 340 s, as the player got out of bed and walked to the desk, and
//! over sun_3.ctm, standing still; the items over sun_4.ctm on Route 1. What a trainer keeps is not
//! measured. Walls are the ROM's meshes (`walls`).
//!
//! Bound by:
//! - ADR-0035: a pointer is followed only into a small block the last walk found, and no further
//!   than its end.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{self, f32, u16, u32};
use crate::core::discovery::Found;
use crate::core::heaps::ReadEach;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::field::ground::{Ground, Of};
use crate::games::common::field::pad::quarter;
use crate::games::common::field::things::{
    FURNITURE_SOURCE, Field, Interactable, WARP_SOURCE, place_source,
};
use crate::games::sm::camera::{self, turned, turned_box, unturned};
use crate::games::sm::heaps::{self, Blocks};
use crate::games::sm::walls::{AreaWalls, SunWalls};
use crate::games::sm::zones::{self as sm_zones, Record, SunZones};
use crate::py::{self, Result};

pub const MANAGER: &str = "N5Field9MoveModel21FieldMoveModelManagerE";
pub const MODEL: &str = "N5Field9MoveModel14FieldMoveModelE";
pub const PLAYER: &str = "N5Field9MoveModel20FieldMoveModelPlayerE";
pub const STATE: &str = "N5Field8Fieldmap5StateE";
pub const THING: &str = "N5Field17FieldBgEventActorE";
pub const WARP: &str = "N5Field17FieldMapJumpActorE";
pub const ITEM: &str = "N5Field14FieldItemActorE";
/// The actors listed. An item actor is built for each ball still on the ground: seven on Route 1's
/// second zone in sun_4.ctm, each at its record's place to the bit.
pub const ACTORS: [&str; 3] = [THING, WARP, ITEM];
pub const PROP: &str = "11StaticActor";
pub const BOX: &str = "8BoxShape";
/// The collision's objects for what stands still on the field: a prop's body, a box or a
/// cylinder, round a place. The floor and the walls are the ROM's (`walls`); a flower pot, a
/// pillar or a tree is one of these, and wider than the record its thing is read by (sun_5.ctm).
pub const CYLINDER: &str = "13CylinderShape";
/// What a sweep looks for. The models are reached from their manager, whose list says which is
/// the player's.
pub const CLASSES: [&str; 7] = [MANAGER, STATE, THING, WARP, ITEM, PROP, camera::CLASS];
/// The heap the move models, their manager and the area loaded are in, beside the dialogue's: the
/// spike 112 capture read every small block of every heap every 2 s for 100 s of sun_3.ctm, and
/// nothing went down.
pub const HEAPS: [i64; 1] = [3];

/// Sun's own words, for what it never names (ADR-0040): its scenes' timing, and a folder a
/// language. No character's name ships yet, only the words for the passport's photos and the
/// battle's buttons (`pictures.PANE`); the player's own names are kept all the same.
///
/// A function: Python's `NAMES` is a `Path` made as the module loads.
pub fn names() -> PathBuf {
    crate::core::keyfile::data_path()
        .unwrap_or_default()
        .join("sm")
}

/// The manager to the end of its models: the player at +0x0c, then 31 more.
pub const MANAGER_SLOTS: i64 = 0x88;
pub const MANAGER_PLAYER: i64 = 0x0C;
pub const MANAGER_FIRST: i64 = 0x10;
pub const MODEL_NODE: i64 = 0x20;
pub const MODEL_CODE: i64 = 0x6C;
/// 1 while the model is drawn, 0 while it stands on the field unseen, its model not loaded
/// (+0x30c 0xffff). The living room held five such, one in the player's own cell, and the porch
/// outside four, and P led the user to them (sun_4.ctm, spike 117, 2026-09-26).
pub const MODEL_SHOWN: i64 = 0x308;
/// Where a model stands now, x, y and z. Its +0x30 is where the save put it: in sun_1.ctm it
/// stayed zero while +0x3a4 walked from (2411, 2418) to (2126, 1978), and the field's own copy at
/// its +0x104 followed a frame ahead.
pub const MODEL_PLACE: i64 = 0x3A4;
pub const PLACE_SIZE: i64 = 12;
/// A model from its start to where it stands, in one read.
pub const MODEL_HEAD: i64 = MODEL_PLACE + PLACE_SIZE;
pub const STATE_FIELD: i64 = 0x14;
pub const FIELD_AREA: i64 = 0x78;
pub const FIELD_ZONE: i64 = 0x7C;
pub const FIELD_HEAD: i64 = 0x80;
/// What gfl2 puts before an object in its block: the field is 0x20 into its block's body.
pub const OBJECT: i64 = 0x20;
pub const ACTOR_X: i64 = 0x14;
pub const ACTOR_Z: i64 = 0x1C;
pub const ACTOR_HEAD: i64 = 0x20;
/// The collision's group the prop is in, zero while it is in none: the people's bodies and the
/// flower pots are one, the ranges they are talked to from another, warps' lines another
/// (`prop_cells`). The shapes round two people nobody sees were in none, and the player walked
/// through one of them (sun_5.ctm).
pub const PROP_SCENE: i64 = 0x0C;
pub const PROP_SHAPE: i64 = 0x28;
pub const PROP_X: i64 = 0x30;
pub const PROP_Z: i64 = 0x38;
pub const PROP_HEAD: i64 = 0x3C;
/// A shape's offset from its actor's place: x at +0x04, z at +0x0c.
pub const SHAPE_X: i64 = 0x04;
pub const SHAPE_Z: i64 = 0x0C;
/// A box's two axes on the ground, each x, y, z: along its width, then its depth.
pub const BOX_AXES: i64 = 0x10;
/// Half the box's width and half its depth, along its axes, then half its height.
pub const BOX_HALVES: i64 = 0x28;
pub const BOX_HEAD: i64 = 0x34;
pub const CYLINDER_RADIUS: i64 = 0x10;
pub const CYLINDER_HEAD: i64 = 0x18;
/// The most cells one prop may close: a shape bigger than a room is not a prop.
pub const PROP_CELLS: i64 = 400;
/// Cells either way off a path the player may be and still be walking it (`Navigator`): a
/// body a cell and a half across, walked by a stick, not a grid.
pub const SLACK: i64 = 2;
/// World units a body's cylinder may be from the person it is round and be theirs.
pub const SAME_PLACE: f64 = 5.0;
/// Cells out from a thing a path may end (`Ground.reach`): a flower pot's box is 104 units a
/// side round a record of a unit, and the player stood 42 units from its box to check it.
pub const REACH: i64 = 3;
/// An item ball's source: `ITEM_SOURCE | x << 16 | z` of its cell, as a thing's is its cell under
/// `FURNITURE_SOURCE`.
pub const ITEM_SOURCE: i64 = 4 << 32;
/// More zones than Sun has (295): a zone past it is not a zone, and the field is not up.
pub const ZONES: i64 = 0x400;
pub const AREAS: i64 = 0x400;

/// World units a cell (the module's docstring says why this size).
pub const CELL: f64 = 50.0;

/// What the reader asks of Sun's walls: `SunWalls`, or a stand-in in a test (Python's tests hand
/// it any object with these two methods).
pub trait Walls {
    /// Start reading the area's walls, so the first P in it does not wait for them.
    fn prefetch(&self, area: i64);
    /// The area's walls, or None where they cannot be read.
    fn area(&self, area: i64) -> Result<Option<Arc<AreaWalls>>>;
}

impl Walls for SunWalls {
    fn prefetch(&self, area: i64) {
        SunWalls::prefetch(self, area)
    }

    fn area(&self, area: i64) -> Result<Option<Arc<AreaWalls>>> {
        SunWalls::area(self, area)
    }
}

/// (zone, area, the player's model, where it stands).
type Where = (i64, i64, i64, (f64, f64));

/// An item's `key`: (category, kind, source, home).
type Key = (String, String, i64, Option<i64>);

/// A model's head and where it stands, (x, z).
type Standing = (Vec<u8>, (f64, f64));

/// Where the blocks that may be read are, and the collection they were taken from.
#[derive(Default)]
struct Bounds {
    /// The collection `blocks` was last taken from, to tell it is not the one `walked` gives now
    /// (`blocks() is not source`).
    source: Option<Rc<Vec<(i64, i64)>>>,
    /// The small blocks the last sweep read, sorted: nothing is read outside one.
    blocks: Vec<(i64, i64)>,
    starts: Vec<i64>,
}

/// What the sweeps found.
#[derive(Default)]
struct Tracked {
    managers: Vec<i64>,
    states: Vec<i64>,
    actors: IndexMap<i64, String>,
    props: Vec<i64>,
    cameras: Vec<i64>,
}

/// The area's things as last seen, by key, in the world's cells (`things`).
#[derive(Default)]
struct Kept {
    items: IndexMap<Key, Interactable>,
    area: Option<i64>,
}

/// The player's zone and cell, and the people, things and exits around them.
pub struct SunFieldReader {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    zones: Option<Rc<SunZones>>,
    walls: Option<Rc<dyn Walls>>,
    walked: Option<Blocks>,
    /// Python looks for `read_each` on the memory it is given and uses it where it has one, as
    /// `ExactCache` does and `RpcMemory` does not; here it is given on its own.
    read_each: Option<ReadEach>,
    bounds: RefCell<Bounds>,
    tracked: RefCell<Tracked>,
    kept: RefCell<Kept>,
    /// The collision's group that holds people's and props' bodies, once a drawn person has
    /// shown which it is: the same object from one area to the next (`prop_cells`).
    bodies: Cell<Option<i64>>,
    /// The area and the player's place when last read, to tell a model not yet placed in
    /// an area just loaded (`r#where`).
    stood: Cell<Option<(i64, (f64, f64))>>,
    /// The quarters the camera looked from north as the player's place was last read: every
    /// cell said is turned by it (`camera`).
    turn: Cell<i64>,
    visit: Cell<Option<(i64, i64)>>,
    /// The area the player was last polled in, whose walls have been asked for.
    area: Cell<Option<i64>>,
}

impl SunFieldReader {
    /// `SunFieldReader(memory, names)`: no `zones`, `blocks`, `walls` or `read_each`.
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        SunFieldReader::with(memory, names, None, None, None, None)
    }

    /// `memory` should read exactly what it is asked for (`heaps`); `zones` is the ROM's,
    /// without which exits lead nowhere named and things are a cell wide. `blocks` says every
    /// small block that may be read, (start, end): the heaps' last walk's and those allocated
    /// since (`heaps.SmallBlocks`); without it, only the blocks a
    /// sweep read may be read, and a sweep between whole sweeps, as there was then, read only
    /// those near what was found before: the field and the area loaded were not among them, and
    /// the keys said "no map" standing in the bedroom (sun_3.ctm, 2026-09-26).
    ///
    /// `read_each` is `memory`'s own where it has one (see the field).
    pub fn with(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        zones: Option<Rc<SunZones>>,
        blocks: Option<Blocks>,
        walls: Option<Rc<dyn Walls>>,
        read_each: Option<ReadEach>,
    ) -> Self {
        SunFieldReader {
            memory,
            names,
            zones,
            walls,
            walked: blocks,
            read_each,
            bounds: RefCell::new(Bounds::default()),
            tracked: RefCell::new(Tracked::default()),
            kept: RefCell::new(Kept::default()),
            bodies: Cell::new(None),
            stood: Cell::new(None),
            turn: Cell::new(0),
            visit: Cell::new(None),
            area: Cell::new(None),
        }
    }

    /// Take in what a sweep found, and the blocks that may be read. A sweep of the blocks just
    /// allocated says what is new and nothing of what is gone, so what it found is added to what
    /// is known; every object is checked for its class before it is read.
    pub fn track(&self, found: &Found) {
        {
            let mut bounds = self.bounds.borrow_mut();
            if let Some(walked) = &self.walked {
                let source = walked();
                let mut blocks = source.to_vec();
                blocks.sort();
                bounds.source = Some(source);
                bounds.blocks = blocks;
            } else if !found.partial {
                let mut blocks: Vec<(i64, i64)> = found
                    .pages
                    .iter()
                    .map(|(&start, data)| (start, start + data.len() as i64))
                    .collect();
                blocks.sort();
                bounds.blocks = blocks;
            }
            bounds.starts = bounds.blocks.iter().map(|&(start, _end)| start).collect();
        }
        let mut tracked = self.tracked.borrow_mut();
        let Tracked {
            managers,
            states,
            actors,
            props,
            cameras,
        } = &mut *tracked;
        let mut known: [(&str, &mut Vec<i64>); 4] = [
            (MANAGER, managers),
            (STATE, states),
            (PROP, props),
            (camera::CLASS, cameras),
        ];
        if !found.partial {
            actors.clear();
            for (_name, kept) in known.iter_mut() {
                kept.clear();
            }
        }
        for (&address, name) in &found.heap {
            if ACTORS.contains(&name.as_str()) {
                actors.insert(address, name.clone());
            }
        }
        for (name, kept) in known.iter_mut() {
            let new: Vec<i64> = found
                .of(&[*name])
                .into_iter()
                .filter(|at| !kept.contains(at))
                .collect();
            kept.extend(new);
        }
    }

    /// The player's model and the zone, as the last poll read them.
    pub fn visit(&self) -> Option<(i64, i64)> {
        self.visit.get()
    }

    /// The player's zone and cell, or None while the field is not up.
    pub fn poll(&self) -> Result<Option<Field>> {
        let Some((zone, area, player, (x, z))) = self.r#where()? else {
            return Ok(None);
        };
        // A warp is a new area, or the player's model built again: Route 1's two zones are one
        // area, and walking from one to the other left the way home as though through a door
        // (sun_6.ctm, 2026-09-26).
        self.visit.set(Some((player, area)));
        if Some(area) != self.area.get()
            && let Some(walls) = &self.walls
        {
            self.area.set(Some(area));
            walls.prefetch(area);
        }
        Ok(Some(Field::new(zone, turned(cell(x, z)?, self.turn.get()))))
    }

    /// Where the player can walk in the area loaded, out of the ROM's meshes (`walls`), with
    /// the cells they cannot walk to from where they stand closed: floor runs on under a bed
    /// and a house, walled all round. Nothing is known without the ROM's walls. Always at the
    /// player's own height.
    pub fn ground(&self, r#where: &Field) -> Result<Ground> {
        let found = self.r#where()?;
        let (Some(found), Some(walls)) = (found, &self.walls) else {
            return Ground::of(r#where, Of::default());
        };
        let Some(walls) = walls.area(found.1)? else {
            return Ground::of(r#where, Of::default());
        };
        let turn = self.turn.get();
        let standing = unturned(r#where.cell, turn);
        // The player's height says which of a cell's floors they are on: a deck over the ground,
        // the top of the hill outside the house or the path below it (spike 120).
        let model = self.standing(found.2)?;
        let mut height = match &model {
            None => None,
            Some(model) => Some(f32(&model.0, MODEL_PLACE + 4)?),
        };
        if let Some(value) = height
            && !value.is_finite()
        {
            height = None;
        }
        let chunks = walls.chunks(None, 0, Some(standing), turn, height)?;
        let ground = Ground::of(
            r#where,
            Of {
                chunks: chunks.into_iter().map(Rc::new).collect(),
                ..Of::default()
            },
        )?;
        let drawn: Vec<(f64, f64)> = self
            .drawn(found.2)?
            .into_iter()
            .map(|(_model, _head, place)| place)
            .collect();
        let props: IndexSet<(i64, i64)> = self
            .prop_cells(standing, &drawn)?
            .into_iter()
            .map(|cell| turned(cell, turn))
            .collect();
        let mut closed = ground.standing.clone();
        closed.extend(props);
        Ok(Ground {
            standing: closed,
            reach: REACH,
            ..ground
        })
    }

    /// The cells a solid prop's body stands in: those whose centre is less than half a cell
    /// from its box or its cylinder, as the walls' cells are closed (`walls`), but for a prop
    /// round the player's own cell, `standing`, which cannot be solid to them.
    ///
    /// A prop is solid when it is in the bodies' group, the collision's group that holds a
    /// cylinder where a drawn person stands, the smallest there. The other groups hold the
    /// range a person is talked to from, the lines of warps, the camera's boxes, and a box 950
    /// units long across Route 1 that the player had walked into to start Kukui's scene: taken
    /// as solid, they walled the user in at their own door and sent them round Route 1 to get
    /// home (sun_6.ctm, 2026-09-26). Until a drawn person has shown which group it is, no prop
    /// is taken as solid.
    fn prop_cells(
        &self,
        standing: (i64, i64),
        drawn: &[(f64, f64)],
    ) -> Result<IndexSet<(i64, i64)>> {
        // (group, x, z, kind, data)
        let mut shapes: Vec<(i64, f64, f64, String, Vec<u8>)> = Vec::new();
        let props = self.tracked.borrow().props.clone();
        for prop in props {
            if self.class(prop)?.as_deref() != Some(PROP) {
                continue;
            }
            let Some(head) = self.read(prop, PROP_HEAD)? else {
                continue;
            };
            let group = u32(&head, PROP_SCENE)?;
            if group == 0 {
                continue;
            }
            let shape = u32(&head, PROP_SHAPE)?;
            if shape == 0 {
                continue;
            }
            let kind = self.class(shape)?;
            let size = match kind.as_deref() {
                Some(BOX) => Some(BOX_HEAD),
                Some(CYLINDER) => Some(CYLINDER_HEAD),
                _ => None,
            };
            let data = match size {
                None => None,
                Some(size) => self.read(shape, size)?,
            };
            if let (Some(kind), Some(data)) = (kind, data) {
                shapes.push((group, f32(&head, PROP_X)?, f32(&head, PROP_Z)?, kind, data));
            }
        }
        let mut bodies: Vec<(f64, i64)> = Vec::new();
        for (group, x, z, kind, data) in &shapes {
            if kind == CYLINDER
                && drawn
                    .iter()
                    .any(|&(px, pz)| (x - px).abs() < SAME_PLACE && (z - pz).abs() < SAME_PLACE)
            {
                bodies.push((f32(data, CYLINDER_RADIUS)?, *group));
            }
        }
        // `min(bodies)`: tuples compare by their first float, which a NaN never ties or beats.
        let mut smallest: Option<(f64, i64)> = None;
        for body in bodies {
            smallest = match smallest {
                Some(held) if !tuple_less(body, held) => Some(held),
                _ => Some(body),
            };
        }
        if let Some(smallest) = smallest {
            self.bodies.set(Some(smallest.1));
        }
        let mut closed: IndexSet<(i64, i64)> = IndexSet::new();
        for (group, x, z, kind, data) in &shapes {
            if Some(*group) != self.bodies.get() {
                continue;
            }
            let cells = if kind == BOX {
                box_cells(*x, *z, data)?
            } else {
                cylinder_cells(*x, *z, data)?
            };
            if let Some(cells) = cells
                && cells.len() as i64 <= PROP_CELLS
                && !cells.contains(&standing)
            {
                closed.extend(cells);
            }
        }
        Ok(closed)
    }

    /// `where` with the people, the things read by facing them and the exits of the field
    /// listed.
    pub fn interactables(&self, r#where: &Field) -> Result<Field> {
        let Some((_zone, area, player, _at)) = self.r#where()? else {
            return Ok(r#where.clone());
        };
        let mut items = self.people(player)?;
        items.extend(self.things(area, r#where.zone)?);
        Ok(Field {
            interactables: items,
            ..r#where.clone()
        })
    }

    /// The things, exits and item balls of the area, turned as the pad walks them: those
    /// of the zones loaded now, and outdoors those last seen in the area's other zones and the
    /// ROM's records for the rest.
    ///
    /// The field keeps only the actors of the zone the player is in, and Route 1's two zones
    /// are one area, its line between them walked across without a door: stepping over it, the
    /// thing P was leading to was "gone" (sun_6.ctm, 2026-09-26). What is kept is dropped with
    /// the area, and a thing of a zone loaded now that is no longer there is gone: a ball
    /// picked up.
    fn things(&self, area: i64, zone: i64) -> Result<Vec<Interactable>> {
        let live = self.placed(area, zone)?;
        let mut kept = self.kept.borrow_mut();
        if Some(area) != kept.area {
            kept.items = IndexMap::new();
            kept.area = Some(area);
        }
        // A zone's exits, or its things or its balls, are loaded while any of them is: the
        // actors change over a sweep before the field's zone does, and the game keeps some of a
        // zone's actors and not others, so a zone taken as loaded whole had its paths "gone".
        let loaded: HashSet<(String, Option<i64>)> = live
            .iter()
            .map(|item| (item.category.clone(), item.home))
            .collect();
        let keys: HashSet<Key> = live.iter().map(|item| item.key()).collect();
        kept.items.retain(|key, item| {
            keys.contains(key) || !loaded.contains(&(item.category.clone(), item.home))
        });
        for item in &live {
            kept.items.insert(item.key(), item.clone());
        }
        // Only an outdoor area is one ground. The player's house is an area too, its rooms four
        // zones laid out far apart, and from the porch the other rooms' mats were listed 150 to
        // 330 cells away (the user, 2026-09-27): indoors, the player's own zone is the map.
        let outdoors = match &self.zones {
            Some(zones) => zones.outdoors(zone)?,
            None => false,
        };
        let shown: IndexMap<Key, Interactable> = if outdoors {
            let mut shown = kept.items.clone();
            let mut seen: HashSet<(String, Option<i64>)> = shown
                .values()
                .map(|item| (item.category.clone(), item.home))
                .collect();
            seen.extend(loaded);
            for item in self.recorded(area, &seen)? {
                shown.entry(item.key()).or_insert(item);
            }
            shown
        } else {
            kept.items
                .iter()
                .filter(|(_key, item)| item.home == Some(zone))
                .map(|(key, item)| (key.clone(), item.clone()))
                .collect()
        };
        let mut out: Vec<Interactable> = Vec::new();
        for item in shown.values() {
            let (first, size) = turned_box(item.cell, item.size, self.turn.get());
            out.push(Interactable {
                cell: first,
                size,
                ..item.clone()
            });
        }
        Ok(out)
    }

    pub fn sweep_wanted(&self) -> bool {
        false
    }

    /// (zone, area, the player's model, where it stands), or None.
    fn r#where(&self) -> Result<Option<Where>> {
        let zone_area = self.zone()?;
        let player = self.player()?;
        let (Some(zone_area), Some(player)) = (zone_area, player) else {
            return Ok(None);
        };
        let Some(standing) = self.standing(player)? else {
            return Ok(None);
        };
        // Through a door the area loaded changes before the player's model is placed in it: it
        // stood where it had in the house for two seconds, and everything was said from there,
        // "39 west 219 north" (sun_4.ctm, 2026-09-26). The field is up once it has moved.
        let (area, place) = (zone_area.1, standing.1);
        if let Some(stood) = self.stood.get()
            && area != stood.0
            && place == stood.1
        {
            return Ok(None);
        }
        self.stood.set(Some((area, place)));
        self.turn.set(self.quarter(self.turn.get())?);
        Ok(Some((zone_area.0, area, player, place)))
    }

    /// The quarters from north the camera looks now (`camera`), north without one; `current`
    /// while the camera has no angle yet.
    fn quarter(&self, current: i64) -> Result<i64> {
        let cameras = self.tracked.borrow().cameras.clone();
        for translator in cameras {
            if self.class(translator)?.as_deref() != Some(camera::CLASS) {
                continue;
            }
            let Some(head) = self.read(translator, camera::HEAD)? else {
                continue;
            };
            if u32(&head, camera::SET)? == 0 {
                return Ok(current);
            }
            return quarter(f32(&head, camera::ANGLE)?, Some(current));
        }
        Ok(0)
    }

    /// The zone and the area loaded, through the first state that leads to a field.
    fn zone(&self) -> Result<Option<(i64, i64)>> {
        let states = self.tracked.borrow().states.clone();
        for state in states {
            if self.class(state)?.as_deref() != Some(STATE) {
                continue;
            }
            let link = self.read(state + STATE_FIELD, 4)?;
            let head = match link {
                None => None,
                Some(link) => self.field(u32(&link, 0)?)?,
            };
            let Some(head) = head else {
                continue;
            };
            let zone = u16(&head, FIELD_ZONE)?;
            let area_link = u32(&head, FIELD_AREA)?;
            let area = if area_link != 0 {
                self.read(area_link, 4)?
            } else {
                None
            };
            if zone < ZONES
                && let Some(area) = area
                && u32(&area, 0)? < AREAS
            {
                return Ok(Some((zone, u32(&area, 0)?)));
            }
        }
        Ok(None)
    }

    /// The field's head, read inside the block its own header says it is in.
    ///
    /// The field is in heap 0x21, whose own blocks a walk no longer reads, so its block is not
    /// among the walk's. Its pointer is a live state's, and its block's header is 0x30 before
    /// it: read to the byte, a 'UD' header of a small block that holds the head is the block
    /// the walk would have found (sun_4.ctm, 2026-09-26).
    fn field(&self, at: i64) -> Result<Option<Vec<u8>>> {
        if let Some(head) = self.read(at, FIELD_HEAD)? {
            return Ok(Some(head));
        }
        let header = at - OBJECT - heaps::BLOCK_HEADER;
        if at == 0 || header < 0 {
            return Ok(None);
        }
        let data = self.memory.read(header, heaps::BLOCK_HEADER)?;
        let size = u32(&data, heaps::BLOCK_SIZE)?;
        // `size > SMALL or size < OBJECT + FIELD_HEAD`, as a range for clippy.
        if u16(&data, 0)? != heaps::USED_BLOCK
            || !(OBJECT + FIELD_HEAD..=heaps::SMALL).contains(&size)
        {
            return Ok(None);
        }
        Ok(Some(self.memory.read(at, FIELD_HEAD)?))
    }

    fn player(&self) -> Result<Option<i64>> {
        let managers = self.tracked.borrow().managers.clone();
        for manager in managers {
            if self.class(manager)?.as_deref() != Some(MANAGER) {
                continue;
            }
            let Some(slots) = self.read(manager, MANAGER_SLOTS)? else {
                continue;
            };
            let player = u32(&slots, MANAGER_PLAYER)?;
            if self.class(player)?.as_deref() == Some(PLAYER) {
                return Ok(Some(player));
            }
        }
        Ok(None)
    }

    /// Every model standing on the field but the player's: its model code names it.
    fn people(&self, player: i64) -> Result<Vec<Interactable>> {
        let mut out: Vec<Interactable> = Vec::new();
        for (model, head, (x, z)) in self.drawn(player)? {
            out.push(Interactable {
                code: u32(&head, MODEL_CODE)?,
                source: model,
                move_model: true,
                ..Interactable::new("people", "person", turned(cell(x, z)?, self.turn.get()))
            });
        }
        Ok(out)
    }

    /// Every model drawn on the field but the player's: its address, head and place.
    fn drawn(&self, player: i64) -> Result<Vec<(i64, Vec<u8>, (f64, f64))>> {
        let managers = self.tracked.borrow().managers.clone();
        let mut out: Vec<(i64, Vec<u8>, (f64, f64))> = Vec::new();
        for manager in managers {
            let Some(slots) = self.read(manager, MANAGER_SLOTS)? else {
                continue;
            };
            let mut starts: Vec<i64> = Vec::new();
            for at in (MANAGER_FIRST..MANAGER_SLOTS).step_by(4) {
                starts.push(u32(&slots, at)?);
            }
            self.fetch(&starts, MODEL_HEAD)?;
            for at in (MANAGER_FIRST..MANAGER_SLOTS).step_by(4) {
                let model = u32(&slots, at)?;
                if model == player || self.class(model)?.as_deref() != Some(MODEL) {
                    continue;
                }
                if let Some((head, place)) = self.standing(model)?
                    && u32(&head, MODEL_SHOWN)? != 0
                {
                    out.push((model, head, place));
                }
            }
            return Ok(out);
        }
        Ok(out)
    }

    /// A model's head and where it stands, (x, z), or None for a free slot or a place that
    /// is not a number.
    fn standing(&self, model: i64) -> Result<Option<Standing>> {
        let Some(head) = self.read(model, MODEL_HEAD)? else {
            return Ok(None);
        };
        if u32(&head, MODEL_NODE)? == 0 {
            return Ok(None);
        }
        let (x, z) = (f32(&head, MODEL_PLACE)?, f32(&head, MODEL_PLACE + 8)?);
        if !(x.is_finite() && z.is_finite()) {
            return Ok(None);
        }
        // A model is at the world's corner until the area it is in has placed it: walking out of
        // the house, every person was "39 west 219 north" for a listing (sun_4.ctm, 2026-09-26).
        if x == 0.0 && z == 0.0 {
            return Ok(None);
        }
        Ok(Some((head, (x, z))))
    }

    /// The things, exits and item balls the field has built actors for, where their records
    /// put them in the world and as wide as the ROM's shapes say. Two things at one place are
    /// one thing: Iki Town has five of its things twice over, one for each of two scripts.
    fn placed(&self, area: i64, zone: i64) -> Result<Vec<Interactable>> {
        let mut said: HashSet<(String, (i64, i64))> = HashSet::new();
        let starts: Vec<i64> = self.tracked.borrow().actors.keys().copied().collect();
        self.fetch(&starts, ACTOR_HEAD)?;
        let mut actors: Vec<(i64, String)> = self
            .tracked
            .borrow()
            .actors
            .iter()
            .map(|(&actor, name)| (actor, name.clone()))
            .collect();
        actors.sort();
        let mut out: Vec<Interactable> = Vec::new();
        for (actor, name) in actors {
            if self.class(actor)?.as_deref() != Some(name.as_str()) {
                continue;
            }
            let Some(head) = self.read(actor, ACTOR_HEAD)? else {
                continue;
            };
            let (x, z) = (f32(&head, ACTOR_X)?, f32(&head, ACTOR_Z)?);
            if !(x.is_finite() && z.is_finite()) || said.contains(&(name.clone(), cell(x, z)?)) {
                continue;
            }
            said.insert((name.clone(), cell(x, z)?));
            let record = match &self.zones {
                None => None,
                Some(zones) => {
                    let section = match name.as_str() {
                        WARP => sm_zones::WARPS,
                        ITEM => sm_zones::ITEMS,
                        _ => sm_zones::THINGS,
                    };
                    zones.record(area, section, x, z)?
                }
            };
            // An actor the loaded area has no record of is the last area's, still there until the
            // next sweep: walking back into the house, Route 1's doors were listed there as six
            // warps (sun_1.ctm, 2026-09-26).
            if self.zones.is_some() && record.is_none() {
                continue;
            }
            if let Some(item) = made(&name, x, z, record.as_ref(), zone)? {
                out.push(item);
            }
        }
        Ok(out)
    }

    /// The exits and things the ROM places in the area's zones where the field has built
    /// none of that category's actors, `seen` holding (category, zone) where it has: it builds
    /// only the zone the player is in, and Route 1's two are one ground with a line between
    /// them the player cannot see (sun_7.ctm, 2026-09-26). Item balls are left to their
    /// actors, since a ball in the ROM may have been picked up.
    fn recorded(
        &self,
        area: i64,
        seen: &HashSet<(String, Option<i64>)>,
    ) -> Result<Vec<Interactable>> {
        let found = match &self.zones {
            None => None,
            Some(zones) => zones.areas.get(&area),
        };
        let Some(found) = found else {
            return Ok(Vec::new());
        };
        let mut said: HashSet<(&str, (i64, i64))> = HashSet::new();
        let mut out: Vec<Interactable> = Vec::new();
        let sections = [
            ("exits", WARP, &found.warps),
            ("signs", THING, &found.things),
        ];
        for (category, actor, lists) in sections {
            for records in lists {
                for record in records {
                    let at = cell(record.x, record.z)?;
                    let Some(zone) = record.zone else {
                        continue;
                    };
                    if seen.contains(&(category.to_string(), Some(zone)))
                        || said.contains(&(actor, at))
                    {
                        continue;
                    }
                    said.insert((actor, at));
                    if let Some(item) = made(actor, record.x, record.z, Some(record), zone)? {
                        out.push(item);
                    }
                }
            }
        }
        Ok(out)
    }

    /// The class `address` was built as, read only if its vtable is inside a block read.
    fn class(&self, address: i64) -> Result<Option<String>> {
        if self.inside(address, 4) {
            self.names.of_object(address)
        } else {
            Ok(None)
        }
    }

    /// Read `size` bytes from each of these starts in one burst, where the memory can, for
    /// the reads after it to be answered from: an area loading answers each read in about 5 ms,
    /// and the listing on arriving on Route 1, a read or two an actor and a model, held polling
    /// up 1.35 s (sun_10.ctm, spike 60).
    fn fetch(&self, starts: &[i64], size: i64) -> Result<()> {
        if let Some(read_each) = &self.read_each {
            let reads: Vec<(i64, i64)> = starts
                .iter()
                .filter(|&&start| self.inside(start, size))
                .map(|&start| (start, size))
                .collect();
            read_each(&reads)?;
        }
        Ok(())
    }

    /// These bytes, or None where they are not all inside one block the last sweep read.
    fn read(&self, address: i64, size: i64) -> Result<Option<Vec<u8>>> {
        if self.inside(address, size) {
            Ok(Some(self.memory.read(address, size)?))
        } else {
            Ok(None)
        }
    }

    fn inside(&self, address: i64, size: i64) -> bool {
        // The heaps' watch takes a block out of bounds the look a larger one is taken over it,
        // with no sweep to follow (`heaps.SmallBlocks`).
        let mut bounds = self.bounds.borrow_mut();
        if let Some(walked) = &self.walked {
            let current = walked();
            let changed = match &bounds.source {
                Some(source) => !Rc::ptr_eq(&current, source),
                None => true,
            };
            if changed {
                let source = walked();
                let mut blocks = source.to_vec();
                blocks.sort();
                bounds.starts = blocks.iter().map(|&(start, _end)| start).collect();
                bounds.blocks = blocks;
                bounds.source = Some(source);
            }
        }
        let at = bounds.starts.partition_point(|&start| start <= address) as i64 - 1;
        at >= 0 && address + size <= bounds.blocks[at as usize].1
    }
}

/// `a < b` on two `(float, int)` tuples as Python compares them: by the first items unless they
/// differ by `!=`, which a NaN always does, and then by whether the first is less.
fn tuple_less(a: (f64, i64), b: (f64, i64)) -> bool {
    if a.0 != b.0 { a.0 < b.0 } else { a.1 < b.1 }
}

/// What an actor, or a record without one, of the class `name` is, in the world's cells: an
/// exit, a thing or an item ball, as wide as its record's shape, keyed by its cell. None for a
/// hidden item.
fn made(
    name: &str,
    x: f64,
    z: f64,
    record: Option<&Record>,
    zone: i64,
) -> Result<Option<Interactable>> {
    let at = cell(x, z)?;
    let key = (at.0 & 0xFFFF) << 16 | at.1 & 0xFFFF;
    let (first, size) = reach(at, record)?;
    // A thing's zone is its record's, not the player's: Route 1's two zones are one area, and
    // walking from one to the other made the thing P led to "gone", and would have lost the name
    // given it (sun_6.ctm, 2026-09-26).
    let home = match record {
        Some(Record {
            zone: Some(zone), ..
        }) => *zone,
        _ => zone,
    };
    if name == WARP {
        return Ok(Some(Interactable {
            size,
            zone: record.and_then(|record| record.leads_to),
            code: record.map_or(0, |record| record.kind),
            source: place_source(WARP_SOURCE, key)?,
            home: Some(home),
            ..Interactable::new("exits", exit(record), first)
        }));
    }
    if name == ITEM {
        // The field builds an actor for a hidden item too, where nothing is seen: three of the
        // seven beside the house, and P led the user onto one (sun_4.ctm). Hidden items are an
        // extra the game does not show (parity), so only balls are listed.
        if let Some(record) = record
            && record.kind != sm_zones::ITEM_BALL
        {
            return Ok(None);
        }
        return Ok(Some(Interactable {
            size,
            code: record.map_or(0, |record| record.code),
            source: place_source(ITEM_SOURCE, key)?,
            home: Some(home),
            ..Interactable::new("items", "item ball", first)
        }));
    }
    Ok(Some(Interactable {
        size,
        code: record.map_or(0, |record| record.script),
        source: place_source(FURNITURE_SOURCE, key)?,
        home: Some(home),
        ..Interactable::new("signs", thing(record), first)
    }))
}

/// What a warp is, in ORAS's words, by its type: a door, the mat out of a building, stairs, or
/// the path into the next area, walked through where no door is. "warp" for a type not seen.
fn exit(record: Option<&Record>) -> &'static str {
    let Some(record) = record else {
        return "warp";
    };
    match record.kind {
        sm_zones::WARP_DOOR => "door",
        sm_zones::WARP_OUT => "way out",
        sm_zones::WARP_EDGE => "path",
        sm_zones::WARP_OPENING => "path",
        sm_zones::WARP_UP => "stairs up",
        sm_zones::WARP_DOWN => "stairs down",
        _ => "warp",
    }
}

/// A signpost as a sign, anything else read by facing it as an object, as ORAS says them.
fn thing(record: Option<&Record>) -> &'static str {
    if let Some(record) = record
        && record.script == 0
        && sm_zones::SIGNS.contains(&record.kind)
    {
        return "sign";
    }
    "object"
}

/// A little-endian f32 each four bytes of what `struct.unpack_from` took, as the floats they are.
fn floats(raw: &[u8]) -> Vec<f64> {
    raw.as_chunks::<4>()
        .0
        .iter()
        .map(|word| f32::from_le_bytes(*word) as f64)
        .collect()
}

/// The cells within half a cell of a box shape round (x, z), or None for one that is not a
/// number.
fn box_cells(x: f64, z: f64, data: &[u8]) -> Result<Option<IndexSet<(i64, i64)>>> {
    let values = floats(&binary::take::<48>(data, SHAPE_X)?);
    if !values.iter().all(|value| value.is_finite()) {
        return Ok(None);
    }
    let (cx, cz) = (x + values[0], z + values[2]);
    let axes = [(values[3], values[5]), (values[6], values[8])];
    let halves = [values[9] + CELL / 2.0, values[10] + CELL / 2.0];
    let reach = py::hypot(halves[0], halves[1]);
    if reach > CELL * PROP_CELLS as f64 {
        return Ok(None);
    }
    let mut out: IndexSet<(i64, i64)> = IndexSet::new();
    let (low, high) = (
        py::floor((cx - reach) / CELL)?,
        py::floor((cx + reach) / CELL)?,
    );
    for gx in low..=high {
        let (low, high) = (
            py::floor((cz - reach) / CELL)?,
            py::floor((cz + reach) / CELL)?,
        );
        for gz in low..=high {
            let (dx, dz) = ((gx as f64 + 0.5) * CELL - cx, (gz as f64 + 0.5) * CELL - cz);
            if axes
                .iter()
                .zip(halves.iter())
                .all(|(&(ax, az), &half)| (dx * ax + dz * az).abs() < half)
            {
                out.insert((gx, gz));
            }
        }
    }
    Ok(Some(out))
}

/// The cells within half a cell of a cylinder shape round (x, z), or None.
fn cylinder_cells(x: f64, z: f64, data: &[u8]) -> Result<Option<IndexSet<(i64, i64)>>> {
    let values = floats(&binary::take::<16>(data, SHAPE_X)?);
    let (ox, _oy, oz, radius) = (values[0], values[1], values[2], values[3]);
    if !(ox.is_finite() && oz.is_finite() && radius.is_finite()) || radius > CELL * 20.0 {
        return Ok(None);
    }
    let (cx, cz, reach) = (x + ox, z + oz, radius + CELL / 2.0);
    let mut out: IndexSet<(i64, i64)> = IndexSet::new();
    let (low, high) = (
        py::floor((cx - reach) / CELL)?,
        py::floor((cx + reach) / CELL)?,
    );
    for gx in low..=high {
        let (low, high) = (
            py::floor((cz - reach) / CELL)?,
            py::floor((cz + reach) / CELL)?,
        );
        for gz in low..=high {
            if py::hypot((gx as f64 + 0.5) * CELL - cx, (gz as f64 + 0.5) * CELL - cz) < reach {
                out.insert((gx, gz));
            }
        }
    }
    Ok(Some(out))
}

/// The first cell and the size of what a record's shape covers, or the actor's own cell.
fn reach(at: (i64, i64), record: Option<&Record>) -> Result<((i64, i64), (i64, i64))> {
    let Some(shape) = record.and_then(|record| record.shape.as_ref()) else {
        return Ok((at, (1, 1)));
    };
    let (west, north) = cell(shape.west, shape.north)?;
    let (east, south) = cell(shape.east, shape.south)?;
    Ok(((west, north), (east - west + 1, south - north + 1)))
}

fn cell(x: f64, z: f64) -> Result<(i64, i64)> {
    Ok((py::floor(x / CELL)?, py::floor(z / CELL)?))
}
