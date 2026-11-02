//! The Pokémon-Amie page, the PlayNav's in ORAS and the bottom screen's own in X, whose Pokémon is a
//! picture named by its species (spike 198); and Amie itself, and its decorate screen.
//!
//! After the tutorial the page shows one Pokémon of the player's walking in its ring, a 40 by 30
//! touch area on a picture of its own, and the game names neither: touch mode said "touch: 2 of
//! layout 0x15a024bc has no name" and left it out, so the player could not open Amie at all
//! (omega_ruby_bottom-screen1). The picture draws texture 0 of a `common::resource::PokeIcon`, the
//! class the AreaNav's Pokémon Distribution and the Bag draw their icons with
//! (`screens.distribution`, `screens.partyicons`). The page's object is a class of its module, Game
//! Freak's "kawaigari" (Amie), `field::subscreen::kawaigari::FM_Kawaigari`:
//!
//! ```text
//! FM_Kawaigari  +0x14C  the national number of the Pokémon the page shows (254, Sceptile, in the
//!                       replay; a second 0x18-byte record at +0x164 held the same 254)
//!               +0x194  -> a record whose +0x00 -> common::resource::PokeIcon
//! PokeIcon      +0x0C   -> LytTexReplaceSystem: +0x04 its textures, +0x08 how many
//! texture 0     +0x0C   the image's physical address, as a picture's material keeps it
//! ```
//!
//! A picture drawing that image is the Pokémon's. Measured with one Pokémon on one replay: that
//! +0x14C holds the number for any other species is taken from the two records agreeing.
//!
//! X's page keeps the same 0x28 earlier, as its Super Training page does: its number at +0x124,
//! Frogadier's 657, and again at +0x13C (`Game.amie`, spike 228, x_super_train2). Read at +0x14C,
//! touch mode said "2 of layout 0x159e193c has no name" and left the page with its arrows alone.
//!
//! PLAY opens Amie itself, `kawaigari::MainProc`: the Pokémon's model on the bottom screen and no
//! button drawn, its Poké Puffs, minigames and way back hidden (omega_ruby_bottom-screen4). Amie
//! strokes the Pokémon by a slide over it, and shows its hand where the stylus is (spike 200, live in
//! the rig). So while it runs touch mode offers the Pokémon, "stroke Sceptile", whose key slides
//! back and forth over it: an exploration past ADR-0009's tap, which Khai asked for (2026-10-08).
//!
//! The slide goes where the Pokémon is. Amie keeps a picture of it as the bottom screen shows it, a
//! grey a part and only what faces the camera, and its script asks the grey under the stylus
//! (`CheckTouchingColor`, spike 280):
//!
//! ```text
//! kawaigari::Player  +0x22C -> gfl::heap::NwAllocator, the picture's holder
//! holder             +0x164, +0x168 -> two buffers in the linear heap; +0x16C which is current
//! buffer             +0x30000 the picture: column x from +x * 0x400, its 240 pixels from the
//!                    screen's bottom up, 4 bytes each, 0xFF and the part's grey three times
//! ```
//!
//! The slide runs along the longest row of the Pokémon's head, 8 on every Pokémon, inset from its
//! ends, or of its body, 9, where its head shows too little; across the middle as before where there
//! is no picture. Bound by ADR-0009, as amended 2026-11-02.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::discovery::{HEAP_SPAN, LINEAR_SPAN, within};
use crate::core::heaps;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::touch::Target;
use crate::games::common::{lyt, touch};
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::gflheap;
use crate::games::gen6::screens::distribution as d;
use crate::games::gen6::screens::townmap;
use crate::py::{self, Error, Kind, Result};

pub const CLASS: &str = "N5field9subscreen9kawaigari12FM_KawaigariE";
/// The apps a subscreen page's Pokémon are named in: the field, whose bottom screen the subscreens
/// are, before any app is known, and the AreaNav, as touch mode asks of every icon in the field's
/// other app (Super Training's page too, `subscreen.super_training`).
pub const PAGE_APPS: [Option<&str>; 3] = [Some(FIELD), None, Some(townmap::APP)];
/// Amie's decorate screen, the app DECORATE starts, where its Poké Puffs are named (`Puffs`).
pub const ARRANGE: &str = "N3app17kawaigari_arrange4ProcE";
/// Pokémon-Amie itself, the app PLAY starts.
pub const AMIE: &str = "N9kawaigari8MainProcE";
/// Amie's three games, Berry Picker, Head It and Tile Puzzle, each an app of its own, whose clock
/// is said only at its marks (`ScreenAnnouncer`; as_amie_berry_picker, as_amie_head_it and
/// as_amie_tile_puzzle).
pub const GAMES: [&str; 3] = [
    "N3app5nuts27ProcessE",
    "N3app7heading7ProcessE",
    "N3app6puzzle7ProcessE",
];
/// Pokémon-Amie's own screens, where touch mode drags with Shift alone (ADR-0009 as amended
/// 2026-10-28). A `frozenset` in Python, only ever tested for membership.
pub const SCREENS: [&str; 5] = [AMIE, ARRANGE, GAMES[0], GAMES[1], GAMES[2]];
/// A digit of a count the games' results draw a picture a digit: Berry Picker's berries and Head
/// It's total (or_amie_stars). Python's `DIGIT`, `(\d)\.bclim` matched whole.
pub fn digit(image: Option<&str>) -> Option<String> {
    let image = image.filter(|image| !image.is_empty())?;
    let stem = image.strip_suffix(".bclim")?;
    let mut chars = stem.chars();
    let first = chars.next()?;
    (chars.next().is_none() && py::is_decimal(first)).then(|| first.to_string())
}

/// Back and forth across the middle of the screen, over the Pokémon's body: Sceptile stood from
/// about 110 to 200 across and 60 to 170 down, and the hand followed every pass (spike 200). Where
/// the Pokémon's picture cannot be read.
pub const STROKE: [(i64, i64); 6] = [
    (110, 110),
    (210, 110),
    (110, 130),
    (210, 130),
    (110, 110),
    (210, 110),
];
/// Amie's own object, which keeps the stylus and leads to the picture of the Pokémon's parts. The
/// module's code holds copies of its first words, which a sweep finds as well (spike 279).
pub const PLAYER: &str = "N9kawaigari6PlayerE";
pub const HOLDS_PICTURE: i64 = 0x22C;
/// The class `Player` +0x22C is, whose `BUFFERS` hold the picture.
pub const PICTURE_HOLDER: &str = "N3gfl4heap11NwAllocatorE";
/// The picture's two buffers, and after them which is current, 0 or 1.
pub const BUFFERS: i64 = 0x164;
pub const PICTURE: i64 = 0x30000;
/// Where the screen's picture starts in a buffer, and how far apart its columns are.
pub const COLUMN: i64 = 0x400;
pub const HEAD: i64 = 8;
/// The greys of a Pokémon's head and body, the same on 17 Pokémon; the others mark its eyes, a
/// hand, or a part a Pokémon of its own: ear tips, a horn, wings, a beak (spike 280's census).
pub const BODY: i64 = 9;
/// The narrowest row of head the slide goes along before the body instead. A 12-pixel slide made
/// no stroke, and a 33-pixel drag did (spike 279, heard).
pub const LONG_ENOUGH: i64 = 40;
/// Every how many rows the picture is looked along.
pub const ROW_STEP: i64 = 4;
/// How much of a row the slide keeps off at each end, as the Pokémon moves while it lasts.
pub const INSET: f64 = 0.15;
/// How far an aimed slide goes in all, back and forth along its row: as far as `STROKE`'s, five
/// passes of 100 pixels in `STROKE_SECONDS`. Five passes of Pikachu's head, 41 pixels, made Amie
/// pet it as often (spike 280, `--compare`), but Orion heard it stroked fewer times (2026-11-03).
pub const REACH: i64 = 500;
/// Where a drag carries a Poké Puff to on the Pokémon, its mouth: held there, Pikachu leaned in,
/// ate it in about 2.5 s and showed a heart; held at the screen's middle, over its eyes, it frowned
/// and turned away, and 17 above its mouth it ate nothing in 3.5 s (or_amie_dragging_puffs, spike
/// 257). Blaziken and Metagross ate there too, a little slower (Orion, 2026-10-28).
pub const MOUTH: (i64, i64) = (160, 157);
pub const STROKE_SECONDS: f64 = 3.0;
/// The pane a stroke stands in for in touch mode, which no pane is at.
pub const STROKE_TARGET: i64 = 1;
/// pa3ds's word for the slide, the manual's: "you can stroke your Pokémon".
pub const STROKE_WORD: &str = "amie-stroke";
/// Where ORAS's page keeps its Pokémon's number; X's keeps it at 0x124 (`Game.amie`).
pub const NUMBER: i64 = 0x14C;
/// How far after the number the holder is, in ORAS and X alike (spike 228).
pub const HOLDER: i64 = 0x48;
/// Seconds before a replace system's textures are read again for an image not among them.
pub const REREAD: f64 = 1.0;

/// A screen's layout helper, which the decorate screen's item pages point at (`Goods`).
pub const G2D_CLASS: &str = "N3app4util7G2DUtilE";
/// A page of Interior Items' slots, and textures in its replace system.
pub const SLOTS: i64 = 15;
pub const G2D: i64 = 0x04;
pub const PAGE: i64 = 0x40;
pub const LISTS: i64 = 0x44;
/// Where each page's items start in the lists: the cushions, the objects and the furniture.
pub const PAGE_LISTS: [i64; 3] = [0, 120, 165];
pub const ITEMS: i64 = 172;
pub const PUFFS: i64 = 0x124;
/// The game's text names Amie's 34 wallpapers before its items (`RomLayout.goods`).
pub const ITEMS_FROM: i64 = 34;
/// Heap blocks walked back from a G2DUtil for the pages' block, which came 0x768 before it.
pub const WALK: i64 = 32;
/// A heap block's link to the block before it (`pa3ds.core.heaps`).
pub const PREVIOUS: i64 = 0x08;

/// A stroke's path: (x, y) after (x, y).
type Path = Vec<(i64, i64)>;

/// Where a stroke slides over the Pokémon in its picture of parts (`PICTURE` on): back and
/// forth along the longest row of its head, or of its body where no row of head is
/// `LONG_ENOUGH`, or of any of it, kept `INSET` off the row's ends, as many times as go `REACH`,
/// five at least. None where none of it is drawn.
pub fn aim(picture: &[u8]) -> Result<Option<Path>> {
    // By grey, None any: the longest row's length, its first pixel across, and how far down.
    let mut longest: IndexMap<Option<i64>, (i64, i64, i64)> = IndexMap::from([
        (Some(HEAD), (0, 0, 0)),
        (Some(BODY), (0, 0, 0)),
        (None, (0, 0, 0)),
    ]);
    for y in (ROW_STEP / 2..touch::HEIGHT).step_by(ROW_STEP as usize) {
        let mut row: Vec<Option<i64>> = Vec::new();
        for x in 0..touch::WIDTH {
            row.push(grey(picture, x * COLUMN + (touch::HEIGHT - 1 - y) * 4)?);
        }
        for (wanted, best) in longest.iter_mut() {
            let on = |grey: Option<i64>| grey.is_some() && (wanted.is_none() || *wanted == grey);
            // `itertools.groupby` of whether each pixel is the part wanted: its runs in turn.
            let (mut x, mut at) = (0, 0);
            while at < row.len() {
                let state = on(row[at]);
                let mut length = 0;
                while at < row.len() && on(row[at]) == state {
                    length += 1;
                    at += 1;
                }
                if state && length > best.0 {
                    *best = (length, x, y);
                }
                x += length;
            }
        }
    }
    let (length, left, y) = [Some(HEAD), Some(BODY)]
        .iter()
        .map(|wanted| longest[wanted])
        .find(|found| found.0 >= LONG_ENOUGH)
        .unwrap_or(longest[&None]);
    if length == 0 {
        return Ok(None);
    }
    let inset = py::round(length as f64 * INSET)?;
    let ends = [(left + inset, y), (left + length - 1 - inset, y)];
    // Python's `max(1, ...)` and `max(5, ...)`: the first unless the second is greater.
    let span = ends[1].0 - ends[0].0;
    let reached = py::round(REACH as f64 / if span > 1 { span } else { 1 } as f64)?;
    let passes = if reached > 5 { reached } else { 5 };
    Ok(Some(
        (0..passes + 1).map(|k| ends[(k % 2) as usize]).collect(),
    ))
}

/// The grey of the part drawn at a pixel of the picture, or None off the Pokémon.
fn grey(picture: &[u8], at: i64) -> Result<Option<i64>> {
    let pixel = py::slice(picture, at, at + 4);
    let [alpha, grey, green, blue] = pixel else {
        return Err(Error::value(format!(
            "not enough values to unpack (expected 4, got {})",
            pixel.len()
        )));
    };
    Ok((*alpha == 0xFF && grey == green && green == blue).then_some(i64::from(*grey)))
}

/// Names the picture of the Pokémon the Pokémon-Amie page shows by its species.
pub struct AmiePage {
    memory: Rc<dyn Memory>,
    /// Whether the stroke is aimed by the picture: the GPU draws it, and Azahar before
    /// 2126.2 can crash when memory the GPU draws into is read (`Rpc.reads_beside_gpu`).
    aim: bool,
    names: Rc<ClassNames>,
    species: Vec<String>,
    /// Where the page keeps its Pokémon's number, its holder `HOLDER` after it.
    number: i64,
    /// Page object -> the vtable it was found with.
    pages: IndexMap<i64, i64>,
    /// The species the page last showed, which Amie opens on.
    last: Option<String>,
    /// `Player` object -> the vtable it was found with.
    players: IndexMap<i64, i64>,
    /// When the stroke was last aimed, and where: the picture is 320 KB, and a replay asks
    /// for the targets every poll.
    aimed: (f64, Option<Path>),
}

impl AmiePage {
    /// What the heap is swept for: the page, which says whose icon its Pokémon is, and Amie's
    /// `Player`, which leads to the picture its stroke is aimed by.
    pub const CLASSES: [&'static str; 2] = [CLASS, PLAYER];

    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>, species: Vec<String>) -> Self {
        AmiePage::with_number(memory, names, species, NUMBER)
    }

    /// `AmiePage(memory, names, species, number)`; Python's `aimed` defaults to False.
    pub fn with_number(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        species: Vec<String>,
        number: i64,
    ) -> Self {
        AmiePage::with_aim(memory, names, species, number, false)
    }

    /// `AmiePage(...)` with every argument.
    pub fn with_aim(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        species: Vec<String>,
        number: i64,
        aimed: bool,
    ) -> Self {
        AmiePage {
            memory,
            aim: aimed,
            names,
            species,
            number,
            pages: IndexMap::new(),
            last: None,
            players: IndexMap::new(),
            aimed: (-REREAD, None),
        }
    }

    /// The pages and Players a sweep found, by class: all of them, or with `adding` the new
    /// ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut kept: IndexMap<&str, IndexMap<i64, i64>> =
            [(CLASS, &self.pages), (PLAYER, &self.players)]
                .into_iter()
                .map(|(kind, have)| {
                    (
                        kind,
                        if adding {
                            have.clone()
                        } else {
                            IndexMap::new()
                        },
                    )
                })
                .collect();
        for (&at, name) in found {
            if let Some(objects) = kept.get_mut(name.as_str()) {
                objects.insert(at, u32(&self.memory.read(at, 4)?, 0)?);
            }
        }
        self.pages = kept.shift_remove(CLASS).unwrap_or_default();
        self.players = kept.shift_remove(PLAYER).unwrap_or_default();
        self.species()?;
        Ok(())
    }

    /// The species a picture draws the Amie page's Pokémon as, in the game's words, or None.
    pub fn name(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        if !PAGE_APPS.contains(&running) || self.pages.is_empty() || self.species.is_empty() {
            return Ok(None);
        }
        let memory = &self.memory;
        let material = u32(&memory.read(picture + lyt::PICTURE_MATERIAL, 4)?, 0)?;
        let image = if material != 0 {
            u32(&memory.read(material + lyt::MATERIAL_TEXTURE, 4)?, 0)?
        } else {
            0
        };
        if image == 0 {
            return Ok(None);
        }
        for (&page, &vtable) in &self.pages {
            let head = memory.read(page, self.number + HOLDER + 4)?;
            // A page freed and its memory taken by something else says nothing.
            if u32(&head, 0)? != vtable {
                continue;
            }
            let (number, holder) = (u32(&head, self.number)?, u32(&head, self.number + HOLDER)?);
            if holder == 0 || !(0 < number && number < self.species.len() as i64) {
                continue;
            }
            if self.first_image(u32(&memory.read(holder, 4)?, 0)?)? == Some(image) {
                let name = &self.species[number as usize];
                return Ok(if name.is_empty() {
                    None
                } else {
                    Some(name.clone())
                });
            }
        }
        Ok(None)
    }

    /// The species the page shows, or the one it last showed once it has gone, as it may
    /// while Amie runs.
    pub fn species(&mut self) -> Result<Option<String>> {
        for (&page, &vtable) in &self.pages {
            let head = self.memory.read(page, self.number + 4)?;
            let number = u32(&head, self.number)?;
            if u32(&head, 0)? == vtable
                && 0 < number
                && number < self.species.len() as i64
                && !self.species[number as usize].is_empty()
            {
                self.last = Some(self.species[number as usize].clone());
                break;
            }
        }
        Ok(self.last.clone())
    }

    /// The stroke touch mode offers while Amie runs, by the Pokémon's species.
    pub fn targets(&mut self, running: Option<&str>) -> Result<Vec<Target>> {
        let species = if running == Some(AMIE) {
            self.species()?
        } else {
            None
        };
        let Some(species) = species else {
            return Ok(Vec::new());
        };
        let name = own(STROKE_WORD, &[("pokemon", &species)]);
        Ok(vec![Target {
            path: self.stroke()?.unwrap_or_else(|| STROKE.to_vec()),
            seconds: STROKE_SECONDS,
            drop: Some(MOUTH),
            ..Target::new(STROKE_TARGET, &name, 160, 120, 0.0, touch::HEIGHT as f64)
        }])
    }

    /// The stroke aimed by the Pokémon's picture of parts, again at most every `REREAD`
    /// seconds; None where no Player leads to a picture.
    fn stroke(&mut self) -> Result<Option<Path>> {
        let now = py::time::monotonic();
        if !self.aim || now - self.aimed.0 < REREAD {
            return Ok(self.aimed.1.clone());
        }
        let (memory, mut path) = (&self.memory, None);
        for (&player, &vtable) in &self.players {
            let head = memory.read(player, HOLDS_PICTURE + 4)?;
            let holder = u32(&head, HOLDS_PICTURE)?;
            // A Player freed, or a copy of its words in the module's code, leads nowhere.
            if u32(&head, 0)? != vtable
                || !within(HEAP_SPAN, holder)
                || self.names.of_object(holder)?.as_deref() != Some(PICTURE_HOLDER)
            {
                continue;
            }
            let buffers = memory.read(holder + BUFFERS, 12)?;
            let current = u32(&buffers, 8)?;
            let buffer = if current == 0 || current == 1 {
                u32(&buffers, 4 * current)?
            } else {
                0
            };
            if within(LINEAR_SPAN, buffer) {
                path = match memory.read(buffer + PICTURE, touch::WIDTH * COLUMN) {
                    Ok(picture) => aim(&picture)?,
                    // A page unanswered: the stroke crosses the middle this time.
                    Err(error) if error.is(Kind::RpcError) => None,
                    Err(error) => return Err(error),
                };
                break;
            }
        }
        self.aimed = (now, path.clone());
        Ok(path)
    }

    /// The image of texture 0 of a PokeIcon, or None where a link of the chain is not the
    /// class it should be.
    fn first_image(&self, icons: i64) -> Result<Option<i64>> {
        let (memory, names) = (&self.memory, &self.names);
        if icons == 0 || names.of_object(icons)?.as_deref() != Some(d::ICON_CLASS) {
            return Ok(None);
        }
        let replace = u32(&memory.read(icons + d::REPLACE, 4)?, 0)?;
        if replace == 0 || names.of_object(replace)?.as_deref() != Some(d::REPLACE_CLASS) {
            return Ok(None);
        }
        let head = memory.read(replace + d::TEXTURES, 8)?;
        let (listed, count) = (u32(&head, 0)?, u32(&head, 4)?);
        if listed == 0 || !(0 < count && count <= d::MAX_TEXTURES) {
            return Ok(None);
        }
        let texture = u32(&memory.read(listed, 4)?, 0)?;
        Ok(if texture != 0 {
            Some(u32(&memory.read(texture + d::IMAGE, 4)?, 0)?)
        } else {
            None
        })
    }
}

/// Tracks the `LytTexReplaceSystem`s a sweep finds, and says which of them a picture draws a
/// texture of.
pub struct Replaced {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    /// Replace system -> the vtable it was found with.
    systems: IndexMap<i64, i64>,
    /// Replace system -> its table and count as last read, when, and its textures' images.
    images: IndexMap<i64, (i64, i64, f64, Vec<i64>)>,
}

impl Replaced {
    fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        Replaced {
            memory,
            names,
            systems: IndexMap::new(),
            images: IndexMap::new(),
        }
    }

    /// The replace systems a sweep found, by class: all of them, or with `adding` the new
    /// ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut systems = if adding {
            self.systems.clone()
        } else {
            IndexMap::new()
        };
        for (&system, name) in found {
            if name == d::REPLACE_CLASS {
                systems.insert(system, u32(&self.memory.read(system, 4)?, 0)?);
            }
        }
        self.systems = systems;
        let systems = &self.systems;
        self.images.retain(|at, _| systems.contains_key(at));
        Ok(())
    }

    /// The system of `count` textures whose texture i's image the picture draws, and i.
    fn texture(&mut self, picture: i64, count: i64) -> Result<Option<(i64, i64)>> {
        let memory = self.memory.clone();
        let material = u32(&memory.read(picture + lyt::PICTURE_MATERIAL, 4)?, 0)?;
        let image = if material != 0 {
            u32(&memory.read(material + lyt::MATERIAL_TEXTURE, 4)?, 0)?
        } else {
            0
        };
        if image == 0 {
            return Ok(None);
        }
        let now = py::time::monotonic();
        let systems: Vec<(i64, i64)> = self.systems.iter().map(|(&s, &v)| (s, v)).collect();
        for (system, vtable) in systems {
            let head = memory.read(system, d::TEXTURES + 8)?;
            let (listed, count_now) = (u32(&head, d::TEXTURES)?, u32(&head, d::TEXTURES + 4)?);
            // A system freed and its memory taken by something else says nothing.
            if u32(&head, 0)? != vtable || count_now != count {
                continue;
            }
            // Read again where the picture's image is not among them, once a while: the Interior
            // Items' textures are filled as the screen opens, and their pages' last stay empty.
            let stale = match self.images.get(&system) {
                None => true,
                Some(known) => {
                    (known.0, known.1) != (listed, count)
                        || (!known.3.contains(&image) && now - known.2 >= REREAD)
                }
            };
            if stale {
                let table = memory.read(listed, 4 * count)?;
                let mut images: Vec<i64> = Vec::new();
                for i in 0..count {
                    images.push(u32(&memory.read(u32(&table, 4 * i)? + d::IMAGE, 4)?, 0)?);
                }
                self.images.insert(system, (listed, count, now, images));
            }
            if let Some(i) = self.images[&system].3.iter().position(|&x| x == image) {
                return Ok(Some((system, i as i64)));
            }
        }
        Ok(None)
    }
}

/// Names a picture of a Poké Puff in Amie's decorate screen, or in Amie's own bar of them, by
/// its kind, in the game's words.
///
/// The screen draws each puff, in its grid and set out before the Pokémon, with an image of one
/// `LytTexReplaceSystem` that holds a texture for every kind, in the order the game's text names
/// them: Sweet Poké Puff's first, Supreme Winter Poké Puff's twenty-sixth. A picture drawing its
/// texture i's image is a puff of kind i. Read off or_pokemon-me's grid, enlarged: Sweet, Mint,
/// Citrus, Mocha and Spice, then Frosted Sweet and Fancy Sweet, textures 0 to 5 and 10 (spike 228).
/// Its other replace systems hold fewer textures, the Interior Items' only those owned, and none
/// as many as there are kinds. Amie itself draws the bar its Poké Puffs button opens the same
/// way, a puff a picture of `okashi_00.bclim`, said "okashi 00" by its file: "Sweet Poké Puff",
/// "Mint Poké Puff" and on (or_amie_pokepuffs). Its button's icon draws the first kind too.
pub struct Puffs {
    replaced: Replaced,
    puffs: Vec<String>,
}

impl Puffs {
    /// What the heap is swept for: the texture systems the puffs are drawn from.
    pub const CLASSES: [&'static str; 1] = [d::REPLACE_CLASS];

    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>, puffs: Vec<String>) -> Self {
        Puffs {
            replaced: Replaced::new(memory, names),
            puffs,
        }
    }

    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        self.replaced.track(found, adding)
    }

    /// The kind of puff a picture draws in Amie or its decorate screen, or None where it draws
    /// none.
    pub fn name(&mut self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        if running != Some(ARRANGE) && running != Some(AMIE) {
            return Ok(None);
        }
        let found = if !self.puffs.is_empty() {
            self.replaced.texture(picture, self.puffs.len() as i64)?
        } else {
            None
        };
        Ok(found.map(|(_, i)| self.puffs[i as usize].clone()))
    }
}

/// Names a picture of an interior item in Amie's decorate screen's grid, in the game's words.
///
/// The grid has three pages, the player's cushions, objects and furniture, each drawn with fifteen
/// pictures and a `LytTexReplaceSystem` of fifteen textures, texture i the page's slot i, and the
/// next page's slides in as the arrows are tapped (spike 228, live from or_pokemon_me2's end). A
/// plain block of the screen's, the object of a heap block, says what the pages hold:
///
/// ```text
/// block  +0x04   -> the screen's `app::util::G2DUtil`
///        +0x40   the page shown    +0x42 how many there are, 3
///        +0x44   a byte an item, by its number, each page's in turn: the cushions' 120
///                places, the objects' 45 from +0xBC, the furniture's 7 from +0xE9
///        +0x124  -> the Poké Puffs' replace system (`Puffs`)
/// ```
///
/// Its +0x04 is its anchor: the block comes a few heap blocks before the G2DUtil it points at,
/// 0x770 bytes in X and 0x768 in Omega Ruby, so the blocks before each G2DUtil found are walked
/// back to the one pointing at it. A picture drawing texture i of a fifteen-texture system is
/// the shown page's slot i, as only the shown page's pictures lie under the grid's touch areas.
/// The game's text names wallpapers first and then the items by number (`RomLayout.goods`).
pub struct Goods {
    replaced: Replaced,
    goods: Vec<String>,
    /// G2DUtil -> the vtable it was found with.
    g2ds: IndexMap<i64, i64>,
    block: Option<i64>,
}

impl Goods {
    /// What the heap is swept for: the texture systems the items are drawn from, and the
    /// layout helpers the pages' block points at.
    pub const CLASSES: [&'static str; 2] = [d::REPLACE_CLASS, G2D_CLASS];

    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>, goods: Vec<String>) -> Self {
        Goods {
            replaced: Replaced::new(memory, names),
            goods,
            g2ds: IndexMap::new(),
            block: None,
        }
    }

    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        self.replaced.track(found, adding)?;
        let mut g2ds = if adding {
            self.g2ds.clone()
        } else {
            IndexMap::new()
        };
        for (&g2d, name) in found {
            if name == G2D_CLASS {
                g2ds.insert(g2d, u32(&self.replaced.memory.read(g2d, 4)?, 0)?);
            }
        }
        self.g2ds = g2ds;
        Ok(())
    }

    /// The item a picture of the grid draws, or None where it draws none.
    pub fn name(&mut self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        if running != Some(ARRANGE) && running != Some(AMIE) {
            return Ok(None);
        }
        let found = if self.goods.len() as i64 > ITEMS_FROM {
            self.replaced.texture(picture, SLOTS)?
        } else {
            None
        };
        let block = if found.is_some() { self.find()? } else { None };
        let (Some(found), Some(block)) = (found, block) else {
            return Ok(None);
        };
        let head = self.replaced.memory.read(block, LISTS + ITEMS)?;
        let page = head[PAGE as usize] as usize;
        if page >= PAGE_LISTS.len() {
            return Ok(None);
        }
        let at = (LISTS + PAGE_LISTS[page] + found.1) as usize;
        let item = *head
            .get(at)
            .ok_or_else(|| Error::index("index out of range"))?;
        let line = ITEMS_FROM + item as i64;
        Ok(if line < self.goods.len() as i64 {
            Some(self.goods[line as usize].clone())
        } else {
            None
        })
    }

    /// The block saying what the pages hold, as last found where it still points at a
    /// G2DUtil, else walked back to from one.
    fn find(&mut self) -> Result<Option<i64>> {
        let memory = self.replaced.memory.clone();
        if let Some(block) = self.block
            && self.points(block)?
        {
            return Ok(Some(block));
        }
        self.block = None;
        let g2ds: Vec<(i64, i64)> = self.g2ds.iter().map(|(&g, &v)| (g, v)).collect();
        for (g2d, vtable) in g2ds {
            if u32(&memory.read(g2d, 4)?, 0)? != vtable {
                continue;
            }
            let mut header = g2d - gflheap::HEADER - heaps::BLOCK;
            for _ in 0..WALK {
                header = u32(&memory.read(header + PREVIOUS, 4)?, 0)?;
                if header == 0 {
                    break;
                }
                let block = header + heaps::BLOCK + gflheap::HEADER;
                if u32(&memory.read(block + G2D, 4)?, 0)? == g2d && self.points(block)? {
                    self.block = Some(block);
                    return Ok(Some(block));
                }
            }
        }
        Ok(None)
    }

    /// Does the block point at a G2DUtil and the Poké Puffs' replace system, as the pages'
    /// does?
    fn points(&self, block: i64) -> Result<bool> {
        let memory = &self.replaced.memory;
        let head = memory.read(block, PUFFS + 4)?;
        let g2d = u32(&head, G2D)?;
        Ok(self.g2ds.contains_key(&g2d)
            && u32(&memory.read(g2d, 4)?, 0)? == self.g2ds[&g2d]
            && self
                .replaced
                .names
                .of_object(u32(&head, PUFFS)?)?
                .as_deref()
                == Some(d::REPLACE_CLASS))
    }
}
