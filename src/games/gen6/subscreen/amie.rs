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
//! pets the Pokémon by a stylus moved over it, and shows its hand where the stylus is (spike 200,
//! live in the rig); it can open with the Pokémon's back turned, which turns round to separate
//! presses (spike 276). So while it runs touch mode offers the Pokémon, "beckon Sceptile", whose tap
//! presses for `BECKON_SECONDS`, and where a Poké Puff is let go; and the stylus.
//!
//! Amie keeps a picture of the Pokémon as the bottom screen shows it, a grey a part and only what
//! faces the camera, and its script asks the grey under the stylus (`CheckTouchingColor`, spike 280):
//!
//! ```text
//! kawaigari::Player  +0x22C -> gfl::heap::NwAllocator, the picture's holder
//! holder             +0x164, +0x168 -> two buffers in the linear heap; +0x16C which is current
//! buffer             +0x30000 the picture: column x from +x * 0x400, its 240 pixels from the
//!                    screen's bottom up, 4 bytes each, 0xFF and the part's grey three times
//! ```
//!
//! A grey is a kind of part, the same on every Pokémon, in pairs, the odd one on the head and the
//! even on the body, and Amie's script switches on it for the stroke's sound and pitch, its kind, and
//! the hand it draws: a burn on 15 and 16, a shock on 17 and 18 (spike A's census of every model in
//! the ROM, and its listing of the script). It counts a pet only while the hand moves 4 pixels a tick
//! and the press stays down, a lift starting its count afresh. So the **stylus**, a point the arrow
//! keys move over the Pokémon while Amie runs (`StylusKeys`), says the kind of part it comes onto
//! (`PARTS`), a sweet spot or danger zone by a name of pa3ds's own for that Pokémon's part
//! (`SPOTS`); it stays on that part as the Pokémon moves (`AmiePage.follow`); and touch mode's tap
//! or drag on it rubs back and forth across the part (`AmiePage.rub`), carried on as it moves.
//!
//! Bound by:
//! - ADR-0009: the stylus the arrows move, tapped, rubbed or dragged only by touch mode's keys, and
//!   the beckon a tap on the Pokémon (as amended 2026-11-05).

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::discovery::{HEAP_SPAN, LINEAR_SPAN, within};
use crate::core::heaps;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::core::words::{current, own};
use crate::games::common::field::keys::{STYLUS, STYLUS_MOVES};
use crate::games::common::touch::Target;
use crate::games::common::{lyt, touch};
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::gflheap;
use crate::games::gen6::screens::distribution as d;
use crate::games::gen6::screens::townmap;
use crate::games::gen6::subscreen::berry_picker::Touching;
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
/// Where a drag carries a Poké Puff to on the Pokémon, its mouth: held there, Pikachu leaned in,
/// ate it in about 2.5 s and showed a heart; held at the screen's middle, over its eyes, it frowned
/// and turned away, and 17 above its mouth it ate nothing in 3.5 s (or_amie_dragging_puffs, spike
/// 257). Blaziken and Metagross ate there too, a little slower (Orion, 2026-10-28).
pub const MOUTH: (i64, i64) = (160, 157);
/// The pane the Pokémon stands in for in touch mode, which no pane is at.
pub const BECKON_TARGET: i64 = 1;
/// The Pokémon as touch mode offers it, "beckon Sceptile" (Orion, 2026-11-05).
pub const BECKON_WORD: &str = "amie-beckon";
pub const BECKON: (i64, i64) = (160, 130);
/// Where a beckon presses, and for how long: Ralts, its back turned, turned round in 3.5 s to
/// presses of 0.4 s 0.4 s apart there, where taps of 0.1 s a second apart never turned it, nor one
/// press held (spike 276).
pub const BECKON_SECONDS: f64 = 0.4;
/// The pane the stylus stands in for in touch mode, which no pane is at either.
pub const STYLUS_TARGET: i64 = 2;
pub const STYLUS_WORD: &str = "amie-stylus";
/// The stylus as touch mode offers it, by the part it is on, "stylus, head", or alone where the
/// picture cannot be read.
pub const STYLUS_ON: &str = "amie-stylus-on";
/// The grey `AmiePage` gives a pixel off the Pokémon, where the picture has none.
pub const OFF_PICTURE: i64 = -1;
/// The danger zone's greys and the sweet spot's, which each Pokémon has on a part of its own: said
/// by a name of pa3ds's own for its part, `amie-spot-<national number>-<grey>` in `amie.ftl`,
/// "metal X" on Metagross's 27 (Orion, 2026-11-05: "our own name a Pokémon"), and where it has none
/// as the head or the body (`PARTS`). A `frozenset` in Python, only ever tested for membership.
pub const SPOTS: [i64; 4] = [25, 26, 27, 28];
/// How long the band of the picture round the stylus is taken as read (`WINDOW`): an arrow press
/// asks for it twice, held an arrow repeats some thirty times a second, and the stylus is kept on
/// its part every poll.
pub const FRESH: f64 = 0.1;
/// How many columns of the picture either side of the stylus are read at once: 65 of 1 KB each.
pub const WINDOW: i64 = 32;
/// How far the part the stylus is on is looked for once it has moved off it, and how far either
/// side of the stylus a rub goes at most, on its part.
pub const NEAR: i64 = 20;
/// Half a rub at the least: 12 pixels across at `touch.RUB_SPEED` moves the hand 4 pixels a tick
/// of Amie's script and more, which it counts a pet by (spike A); a part narrower than that is
/// rubbed a little past its edges, and Amie forgives two ticks off a part.
pub const RUB_HALF: i64 = 6;
/// Where the stylus is before the arrows first move it: the middle of the screen, where the
/// Pokémon stands.
pub const STYLUS_START: (i64, i64) = (160, 120);
/// How far an arrow moves the stylus, in the screen's pixels: Pikachu's cheek, the narrowest part
/// spike A stroked, was 9 to 11 across, so no step jumps one; the screen is 53 steps across.
pub const STEP: i64 = 6;
/// Said as the stylus leaves the Pokémon, or comes onto grey 0, which the script takes for off
/// it.
pub const OFF: &str = "amie-part-off";
/// Each grey by the kind of part it is, a message each (spike A): the same on every Pokémon. 1 lies
/// on eye and mouth textures and 2 and 3 on furry Pokémon, not yet seen on screen. 25 to 28, the
/// danger zone and the sweet spot, are each Pokémon's own part, said by its name of pa3ds's own
/// (`SPOTS`), or where it has none as the head and the body, the side of the pair each is: the
/// game's own stroke tells them apart, its pitch down on a danger zone and up on a sweet spot
/// (`SEQ_SE_KW_STROKE` at 108 and 148 against 128). Any other grey is said as a part, `amie-part`.
pub const PARTS: [(i64, &str); 27] = [
    (1, "amie-part-face"),
    (2, "amie-part-fur"),
    (3, "amie-part-fur"),
    (4, "amie-part-right-eye"),
    (5, "amie-part-eye"),
    (6, "amie-part-left-eye"),
    (8, "amie-part-head"),
    (9, "amie-part-body"),
    (10, "amie-part-hand"),
    (11, "amie-part-spikes"),
    (12, "amie-part-spikes"),
    (13, "amie-part-ghostly"),
    (14, "amie-part-ghostly"),
    (15, "amie-part-flame"),
    (16, "amie-part-flame"),
    (17, "amie-part-sparks"),
    (18, "amie-part-sparks"),
    (19, "amie-part-slime"),
    (20, "amie-part-slime"),
    (21, "amie-part-sludge"),
    (22, "amie-part-sludge"),
    (23, "amie-part-ice"),
    (24, "amie-part-ice"),
    (25, "amie-part-head"),
    (26, "amie-part-body"),
    (27, "amie-part-head"),
    (28, "amie-part-body"),
];
pub const PART: &str = "amie-part";
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

/// The two ends a rub goes back and forth between.
type Ends = ((i64, i64), (i64, i64));

/// Columns of the picture of parts, `first` and on, as read at once: the greys in them.
pub struct Band {
    pub first: i64,
    pub data: Vec<u8>,
    pub last: i64,
}

impl Band {
    pub fn new(first: i64, data: Vec<u8>) -> Self {
        let last = first + data.len() as i64 / COLUMN - 1;
        Band { first, data, last }
    }

    /// The grey drawn at a pixel, `OFF_PICTURE` off the Pokémon, or None outside the band.
    pub fn grey(&self, x: i64, y: i64) -> Option<i64> {
        if !((self.first..=self.last).contains(&x) && (0..touch::HEIGHT).contains(&y)) {
            return None;
        }
        let at = ((x - self.first) * COLUMN + (touch::HEIGHT - 1 - y) * 4) as usize;
        let [alpha, grey, green, blue] = self.data[at..at + 4] else {
            unreachable!("four bytes");
        };
        Some(if alpha == 0xFF && grey == green && green == blue {
            i64::from(grey)
        } else {
            OFF_PICTURE
        })
    }

    /// The first and last pixel of the run of one grey through (x, y), across the screen or
    /// down it, within the band and `WINDOW` of (x, y) either way, so that a run across and one
    /// down are measured alike.
    pub fn run(&self, x: i64, y: i64, across: bool) -> (i64, i64) {
        let (grey, mut ends) = (self.grey(x, y), Vec::new());
        for way in [-1, 1] {
            let start = if across { x } else { y };
            let mut at = start;
            while (at + way - start).abs() <= WINDOW
                && (if across {
                    self.grey(at + way, y)
                } else {
                    self.grey(x, at + way)
                }) == grey
            {
                at += way;
            }
            ends.push(at);
        }
        (ends[0], ends[1])
    }

    /// The pixel of a grey nearest (x, y) within `reach`, or None.
    pub fn nearest(&self, x: i64, y: i64, grey: i64, reach: i64) -> Option<(i64, i64)> {
        let mut found: Option<(i64, i64, i64)> = None;
        for dx in -reach..reach + 1 {
            for dy in -reach..reach + 1 {
                let distance = (dx * dx + dy * dy, x + dx, y + dy);
                if dx * dx + dy * dy <= reach * reach
                    && self.grey(x + dx, y + dy) == Some(grey)
                    && found.is_none_or(|best| distance < best)
                {
                    found = Some(distance);
                }
            }
        }
        found.map(|(_, x, y)| (x, y))
    }
}

/// Names the picture of the Pokémon the Pokémon-Amie page shows by its species, and offers
/// Amie's Pokémon to beckon and its stylus to rub with.
pub struct AmiePage {
    memory: Rc<dyn Memory>,
    /// Whether the picture of parts is read: the GPU draws it, and Azahar before 2126.2 can
    /// crash when memory the GPU draws into is read (`Rpc.reads_beside_gpu`).
    aim: bool,
    names: Rc<ClassNames>,
    species: Vec<String>,
    /// Where the page keeps its Pokémon's number, its holder `HOLDER` after it.
    at: i64,
    /// Page object -> the vtable it was found with.
    pages: IndexMap<i64, i64>,
    last: Option<String>,
    /// The species the page last showed, which Amie opens on, and its national number.
    number: i64,
    /// `Player` object -> the vtable it was found with.
    players: IndexMap<i64, i64>,
    /// Where the stylus is, as the arrows have moved it (`StylusKeys`), or as it has been
    /// kept on its part (`follow`).
    pub stylus: (i64, i64),
    /// The grey of the part the arrows left the stylus on, which it is kept on.
    kept: Option<i64>,
    /// When the band of the picture round the stylus was read, and the band.
    band: (f64, Option<Rc<Band>>),
}

impl AmiePage {
    /// What the heap is swept for: the page, which says whose icon its Pokémon is, and Amie's
    /// `Player`, which leads to the picture of its parts.
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
            at: number,
            pages: IndexMap::new(),
            last: None,
            number: 0,
            players: IndexMap::new(),
            stylus: STYLUS_START,
            kept: None,
            band: (f64::NEG_INFINITY, None),
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
            let head = memory.read(page, self.at + HOLDER + 4)?;
            // A page freed and its memory taken by something else says nothing.
            if u32(&head, 0)? != vtable {
                continue;
            }
            let (number, holder) = (u32(&head, self.at)?, u32(&head, self.at + HOLDER)?);
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
            let head = self.memory.read(page, self.at + 4)?;
            let number = u32(&head, self.at)?;
            if u32(&head, 0)? == vtable
                && 0 < number
                && number < self.species.len() as i64
                && !self.species[number as usize].is_empty()
            {
                (self.last, self.number) = (Some(self.species[number as usize].clone()), number);
                break;
            }
        }
        Ok(self.last.clone())
    }

    /// The Pokémon touch mode offers to beckon while Amie runs, by its species, and the
    /// stylus, by the part it is on, with where a rub on it goes (`rub`).
    pub fn targets(&mut self, running: Option<&str>) -> Result<Vec<Target>> {
        let species = if running == Some(AMIE) {
            self.species()?
        } else {
            None
        };
        let Some(species) = species else {
            return Ok(Vec::new());
        };
        let (x, y) = self.stylus;
        let part = self.part()?;
        let beckon = Target {
            path: vec![BECKON],
            seconds: BECKON_SECONDS,
            drop: Some(MOUTH),
            ..Target::new(
                BECKON_TARGET,
                &own(BECKON_WORD, &[("pokemon", &species)]),
                160,
                120,
                0.0,
                touch::HEIGHT as f64,
            )
        };
        let stylus = match part.as_deref() {
            Some(part) if !part.is_empty() => own(STYLUS_ON, &[("part", &part)]),
            _ => own(STYLUS_WORD, &[]),
        };
        Ok(vec![
            beckon,
            Target {
                rub: Some(self.rub()?),
                ..Target::new(STYLUS_TARGET, &stylus, x, y, 0.0, touch::HEIGHT as f64)
            },
        ])
    }

    /// The kind of part the stylus is on, in pa3ds's words (`PARTS`), "off" where it is off
    /// the Pokémon, or None where the picture cannot be read.
    pub fn part(&mut self) -> Result<Option<String>> {
        let Some(grey) = self.grey()? else {
            return Ok(None);
        };
        if grey == 0 || grey == OFF_PICTURE {
            return Ok(Some(own(OFF, &[])));
        }
        let spot = if SPOTS.contains(&grey) {
            current().get(&format!("amie-spot-{}-{grey}", self.number), &[])
        } else {
            None
        };
        Ok(Some(match spot {
            Some(spot) if !spot.is_empty() => spot,
            _ => own(
                PARTS
                    .iter()
                    .find(|(kind, _)| *kind == grey)
                    .map_or(PART, |(_, word)| word),
                &[],
            ),
        }))
    }

    /// Keep the stylus on the part it is on now, as the arrows leave it (`follow`); on
    /// none where it is off the Pokémon.
    pub fn keep(&mut self) -> Result<()> {
        let grey = self.grey()?;
        self.kept = match grey {
            None | Some(0 | OFF_PICTURE) => None,
            grey => grey,
        };
        Ok(())
    }

    /// Keep the stylus on its part as the Pokémon moves: where the part has gone from under
    /// it, onto the part's pixel nearest it within `NEAR`, the middle of its run across. Whether
    /// it moved.
    pub fn follow(&mut self) -> Result<bool> {
        let band = self.read()?;
        let (x, y) = self.stylus;
        let (Some(kept), Some(band)) = (self.kept, band) else {
            return Ok(false);
        };
        if band.grey(x, y) == Some(kept) {
            return Ok(false);
        }
        let Some(found) = band.nearest(x, y, kept, NEAR) else {
            return Ok(false);
        };
        let (left, right) = band.run(found.0, found.1, true);
        self.stylus = (py::floordiv(left + right, 2), found.1);
        Ok(true)
    }

    /// Where a rub with the stylus goes: back and forth across the part it is on, the way the
    /// part is longer, `NEAR` at most either side of it and `RUB_HALF` at least; off the
    /// Pokémon, or where the picture cannot be read, `RUB_HALF` either side of where it is.
    pub fn rub(&mut self) -> Result<Ends> {
        let band = self.read()?;
        let (x, y) = self.stylus;
        let band = match band {
            Some(band) if !matches!(band.grey(x, y), None | Some(0 | OFF_PICTURE)) => band,
            _ => return Ok(((x - RUB_HALF, y), (x + RUB_HALF, y))),
        };
        let (across, down) = (band.run(x, y, true), band.run(x, y, false));
        let sideways = across.1 - across.0 >= down.1 - down.0;
        let ((low, high), at) = if sideways { (across, x) } else { (down, y) };
        let half = NEAR.min(RUB_HALF.max(py::floordiv(high - low, 2) - 2));
        let middle = if high - low < 2 * half {
            py::floordiv(low + high, 2)
        } else {
            at.max(low + half).min(high - half)
        };
        if sideways {
            return Ok(((middle - half, y), (middle + half, y)));
        }
        Ok(((x, middle - half), (x, middle + half)))
    }

    /// The grey under the stylus, `OFF_PICTURE` off the Pokémon, or None unread.
    fn grey(&mut self) -> Result<Option<i64>> {
        let band = self.read()?;
        Ok(band.and_then(|band| band.grey(self.stylus.0, self.stylus.1)))
    }

    /// The band of the picture round the stylus, `WINDOW` columns either side, read again
    /// once `FRESH` has gone by or the stylus has come within `NEAR` of its edge; None where
    /// no picture can be read.
    fn read(&mut self) -> Result<Option<Rc<Band>>> {
        let now = py::time::monotonic();
        let (when, band) = &self.band;
        let x = self.stylus.0;
        // Within reach of its edge, but where the band is at the screen's own.
        if let Some(band) = band
            && now - when < FRESH
            && (band.first == 0 || x >= band.first + NEAR)
            && (band.last == touch::WIDTH - 1 || x <= band.last - NEAR)
        {
            return Ok(Some(band.clone()));
        }
        let buffer = self.buffer()?;
        let mut band = None;
        if let Some(buffer) = buffer {
            let first = (x - WINDOW).clamp(0, touch::WIDTH - 1 - 2 * WINDOW);
            band = match self
                .memory
                .read(buffer + PICTURE + first * COLUMN, (2 * WINDOW + 1) * COLUMN)
            {
                Ok(data) => Some(Rc::new(Band::new(first, data))),
                Err(error) if error.is(Kind::RpcError) => None,
                Err(error) => return Err(error),
            };
        }
        self.band = (now, band.clone());
        Ok(band)
    }

    /// The buffer of the picture of parts drawn last, or None where no Player leads to one,
    /// or the emulator cannot read it safely (`aimed`).
    fn buffer(&self) -> Result<Option<i64>> {
        if !self.aim {
            return Ok(None);
        }
        let memory = &self.memory;
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
            let which = u32(&buffers, 8)?;
            let buffer = if which == 0 || which == 1 {
                u32(&buffers, 4 * which)?
            } else {
                0
            };
            if within(LINEAR_SPAN, buffer) {
                return Ok(Some(buffer));
            }
        }
        Ok(None)
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

/// Heard for an arrow that would take the stylus off the screen: the game's error sound, or the
/// words where that is off.
pub const EDGE: &str = "amie-stylus-edge";

/// Amie's own keys, the arrows (Orion, 2026-11-05): each moves the stylus `STEP` pixels over
/// the Pokémon, saying the kind of part it comes onto (`AmiePage.part`), and nothing more while
/// it stays on that kind, touch mode's tone sounding where it is. The part it is left on it is
/// kept on as the Pokémon moves (`poll`). Touch mode chooses the stylus as it moves, so touch
/// mode's tap rubs there a second and its drag rubs there until let go, carried on with each
/// move (`AmiePage.rub`, `TouchMode.choose`).
///
/// `wrong` is what is heard for an arrow at the screen's edge: a sound, and "", or the words.
///
/// Python's holds touch mode, which is built after the subscreens, as it offers their targets;
/// here touch mode, which the navigator owns, is handed to `press` and `poll`.
pub struct StylusKeys {
    page: Rc<RefCell<AmiePage>>,
    wrong: Rc<dyn Fn(&str) -> String>,
    /// The kind of part said last, which a move staying on it does not say again.
    said: Option<String>,
}

impl StylusKeys {
    pub fn new(page: Rc<RefCell<AmiePage>>, wrong: Rc<dyn Fn(&str) -> String>) -> Self {
        StylusKeys {
            page,
            wrong,
            said: None,
        }
    }

    /// The keys' table in the keys file while Amie runs, `[pa3ds.amie.stylus]`, else None.
    pub fn context(&self, running: Option<&str>) -> Option<&'static str> {
        (running == Some(AMIE)).then_some(STYLUS)
    }

    /// Move the stylus, with the line to say.
    pub fn press(
        &mut self,
        action: &str,
        _running: Option<&str>, // only ever asked while Amie runs (`context`)
        touch: &mut dyn Touching,
    ) -> Result<String> {
        let (_, (across, down)) = *STYLUS_MOVES
            .iter()
            .find(|(name, _)| *name == action)
            .ok_or_else(|| Error::key(py::repr(action)))?;
        let (x, y) = self.page.borrow().stylus;
        let moved = (
            (x + across * STEP).clamp(0, touch::WIDTH - 1),
            (y + down * STEP).clamp(0, touch::HEIGHT - 1),
        );
        if moved == (x, y) {
            return Ok((self.wrong)(&own(EDGE, &[])));
        }
        self.page.borrow_mut().stylus = moved;
        self.page.borrow_mut().keep()?;
        touch.choose(STYLUS_TARGET, false)?;
        let part = self.page.borrow_mut().part()?;
        if part == self.said {
            return Ok(String::new());
        }
        self.said = part.clone();
        Ok(part.unwrap_or_default())
    }

    /// Keep the stylus on its part as the Pokémon moves, carrying a rub on with it; says
    /// nothing.
    pub fn poll(
        &self,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        if running == Some(AMIE) && self.page.borrow_mut().follow()? {
            touch.choose(STYLUS_TARGET, true)?;
        }
        Ok(Vec::new())
    }
}
