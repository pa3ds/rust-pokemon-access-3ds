//! Super Training: the PlayNav's page names its party slots, a round offers its goals to tap, and
//! the field offers the training bags in the page's list (omega_ruby_bottom-screen5).
//!
//! The page shows the party as six pictures, each a 40 by 30 button with a touch area, and the game
//! names none: touch mode said "mapnews icon 02" and "pmn 1pix org" by the address of the image, or
//! left them out. They are the icons of a `common::resource::PokeIcon` (`screens.distribution`,
//! `screens.amie`), in the party's order. The page's object is a class of its module, Game Freak's
//! "training" (Super Training), `field::subscreen::training::FM_Training`, which holds the party's
//! `pml::pokepara::PokemonParam`s a record of 0x20 bytes each:
//!
//! ```text
//! FM_Training  +0xF0             -> common::resource::PokeIcon (6 textures, the party in order)
//!              +0x140            the selected member's party index
//!              +0x14C + 0x20 * r -> the PokemonParam of record r
//!              +0x150 + 0x20 * r    the party index of record r
//! PokeIcon     +0x0C -> LytTexReplaceSystem: +0x04 its textures, +0x08 how many
//! texture i    +0x0C   the image's physical address, as a picture's material keeps it
//! ```
//!
//! X's page keeps the same four 0x28 earlier, its PokeIcon at +0xC8 (`Game.training`, spike 228):
//! read at +0xF0 there was a message window, and touch mode left the six slots out with no name.
//!
//! The page is a carousel: the selected member is shown big in the middle and the other five in a
//! row that runs on from it in party order, wrapping. The textures stay in party order, but the
//! records rotate to start at the selected member. With Latios (member 0) selected they were in
//! party order; with Breloom (member 3) selected, record 0 held Breloom with 3 at +0x150, then
//! Sceptile 4, Groudon 5, Latios 0, Greninja 1 and Rayquaza 2, and +0x140 was 3. A texture is named
//! by the record whose index is its own: read by position, Breloom in the middle was called
//! "Latios". A sweep found three objects of the class and one was the page.
//!
//! A round of Super Training, `app::sparring::Proc`, has no touch area on the bottom screen, and
//! touch mode said "nothing to touch" throughout. Its goals are pictures on the top screen's
//! layout, up to eight panes drawing `target01_7.bclim` and `target01_8.bclim`, the frames of the
//! white goal, 32 by 32; the ones shown are up. The round's image archive names `target01_0` to
//! `target01_8`, and `big_target_0` to `big_target_8` for the big goal, which the coach calls a red
//! goal ("red goals are special bonus goals you earn, and they won't disappear after one hit!");
//! the big ones are not yet seen drawn. A bursting goal, `target01_break` and `big_target_break`,
//! is not offered. A pane's x and y hold its place in its layout already, so a goal's place is its
//! own less its root's, as for a touch area, 200 across and 120 down from the screen's middle:
//! (198, 153) matched the screenshot. A tap at the bottom screen's (x, y) shoots at the top
//! screen's (1.25 x, 1.25 y), measured at eight points to within a pixel or two, so a goal is shot
//! by a tap at 0.8 of its place; a tap below y 192 aims off the screen. A tap on (160, 120) at the
//! goal at (198, 153) scored 26. It is a plain tap, ADR-0009's.
//!
//! The page's `list_button` slides out a list of training bags on the left, up to twelve pictures
//! `sand_bag01.bclim` to `sand_bag26.bclim`, 18 by 30, hidden when empty and off the screen's left
//! edge while the list is closed. They have no touch area, and touch mode offered nothing, though
//! the game says "Pick a training bag from the list on the left, and tap on it!". A Strength Bag was
//! at its own less its root's (20, 44) on the touch screen, and a tap there opened its panel with
//! Use and Back. The bag hanging beside the Pokémon, `sand_bag00.bclim`, 40 by 100, has no line in
//! the game's text, and is never offered.
//!
//! What a round shows besides its goals (measured live in the Magnemite regimen, 2026-10-10, in the
//! top screen's layout, places as for a goal): the score is digit pictures, `score_nom0.bclim` to
//! `score_nom9.bclim` with `score_nomslash.bclim`, a row at y 203 reading "157/300" (x 257 to 387),
//! its leading zeros hidden panes. Each hit's popup (`plus` or `minus` and digits, at y 64, hidden
//! between hits) and a row at y 292, off the screen, are drawn with the same pictures. The clock is
//! `num0_c.bclim` to `num9_c.bclim` to the right of `timer_c.bclim` (x 325, y 30): `num2_c`,
//! `num2_c`, `num8_c` at x 352, 373, 391 read "2:28". Other `num?_c` pictures lie off the screen (y
//! -39) or hidden at the middle. The words `start.bclim`, `clear.bclim`, `cleared.bclim` and
//! `gameover_c.bclim` are pictures shown on the layout as they happen.
//!
//! A Balloon Bot's ball is an `actor::SparringMissileActor`, a class of the game's `actor` module,
//! about fifty of them in a pool at a stride of 0x1094 and two the Bot's. Dumped every half second
//! for 15 s:
//!
//! ```text
//! +0x04   u32: 1 while flying, 2 once landed, 0 unused; 3 once, on one of the pool
//! +0x10   f32 x, y, z: z runs from about 8 at the Bot to 227.5 at the player
//! +0x10C  u32 frames in flight: 3 to 0x41, about 1.1 s
//! +0x110  u32 a pointer: set on the Bot's two, 0 on the pool (the player's shots, presumably)
//! ```
//!
//! One of the Bot's flew three times in 15 s, and the score fell 177, 167, 157 as they hit. The
//! player blocks a ball with L in the second before it lands. A shot of the player's own with
//! +0x110 set would warn as well; not seen.

use std::collections::BTreeSet;
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::IndexMap;
use regex::Regex;

use crate::core::binary::{f32, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::pictures::words as picture_words;
use crate::games::common::touch::{HEIGHT, Idle, ImageOf, Off, Target, WIDTH};
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::screens::distribution as d;
use crate::py::{self, Result};

pub const CLASS: &str = "N5field9subscreen8training11FM_TrainingE";
/// A round of Super Training, the app the goals are offered in.
pub const SPARRING: &str = "N3app8sparring4ProcE";
/// Super Training's list of regimens, an app of its own (or_super_training_practice).
pub const MENU: &str = "N3app13sparring_menu4ProcE";
/// Where ORAS's page keeps its PokeIcon; X's keeps it at 0xC8 (`Game.training`).
pub const ICONS: i64 = 0xF0;
/// How far after the PokeIcon the records start, in ORAS and X alike (spike 228).
pub const PARAMS: i64 = 0x5C;
/// Where a record holds its member's party index, after its PokemonParam: the records rotate to
/// start at the selected member, the icons' textures do not.
pub const INDEX: i64 = 0x04;
pub const PARAM_STRIDE: i64 = 0x20;
pub const SLOTS: i64 = 6;
/// pa3ds's word for a goal: the game's dialogue speaks of "the white goals" and names none.
pub const GOAL: &str = "training-goal";
/// pa3ds's word for the big goal, after the coach's "red goals"; seen only in the round's archive
/// so far.
pub const RED_GOAL: &str = "training-red-goal";
pub const GOAL_IMAGE: &str = "target01";
pub const BIG_GOAL_IMAGE: &str = "big_target";
/// What the texture's name of a goal's picture starts with: the white goal's or the big one's, then
/// the frame's digit. Their `_break` frames are not goals.
pub static GOAL_FRAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"^(?:{GOAL_IMAGE}|{BIG_GOAL_IMAGE})_\d\.")).expect("a fixed pattern")
});
/// What the texture's name of a training bag's picture starts with, in the list and beside the
/// Pokémon alike; the one beside it has no word in the game's text and is not offered.
pub const BAG_IMAGE: &str = "sand_bag";
pub const TOP_WIDTH: i64 = 400;
/// The top screen in pixels, as a layout places its panes from its middle.
pub const TOP_HEIGHT: i64 = 240;
/// How far across and down the bottom screen a tap shoots at the top screen's goal at 1 of it.
pub const AIM: f64 = 0.8;
/// The score's pictures: a digit, or the slash between the score and the round's target.
pub static SCORE_DIGIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^score_nom(\d|slash)\.").expect("a fixed pattern"));
/// The clock's pictures: a digit, or the clock's face the digits lie to the right of.
pub static CLOCK_DIGIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:num(\d)_c|(timer)_c)\.").expect("a fixed pattern"));
/// How far apart down the screen two pictures of one row of digits may be, in pixels.
pub const SAME_ROW: f64 = 4.0;
/// The least seconds between two scores said: a score that moves faster is said once, newest, when
/// they have passed.
pub const SCORE_EVERY: f64 = 2.0;
/// The seconds on the clock that are said: every half minute from 2:30, and 0:10.
pub const CLOCK_MARKS: [i64; 6] = [150, 120, 90, 60, 30, 10];
/// The textures, less a trailing `_c`, of the words the round shows: `gameover_c.bclim` is
/// "game over" (`pictures.SPELLINGS`).
pub const BANNERS: [&str; 4] = ["start", "clear", "cleared", "gameover"];
/// pa3ds's words for a ball of the Balloon Bot's in flight; the game says nothing.
pub const SHOT: &str = "training-shot-coming";
pub const MISSILE: &str = "N5actor20SparringMissileActorE";
pub const MISSILE_STATE: i64 = 0x04;
/// A ball's state in flight; 2 once landed, 0 unused.
pub const FLYING: i64 = 1;
/// A pointer, set on the Bot's balls and 0 on the pool of the player's.
pub const BOT_BALL: i64 = 0x110;
/// How much of a ball is read: through `BOT_BALL`.
pub const MISSILE_HEAD: i64 = 0x114;

/// A shown picture of the top screen: its image, and its place across and down from the corner.
pub type Seen = (String, f64, f64);

/// Is a place within the top screen?
pub fn on_top(x: f64, y: f64) -> bool {
    0.0 <= x && x < TOP_WIDTH as f64 && 0.0 <= y && y < TOP_HEIGHT as f64
}

/// The pixel of the bottom screen whose tap shoots at the top screen's (x, y), or None where
/// that is off the top screen.
pub fn aim(x: f64, y: f64) -> Result<Option<(i64, i64)>> {
    if !on_top(x, y) {
        return Ok(None);
    }
    Ok(Some((py::round(x * AIM)?, py::round(y * AIM)?)))
}

/// Names the pictures of the Super Training page's party slots by their members' species.
pub struct TrainingPage {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    species: Vec<String>,
    /// Where the page keeps its PokeIcon, its records `PARAMS` after it.
    icons: i64,
    /// Page object -> the vtable it was found with.
    pages: IndexMap<i64, i64>,
}

impl TrainingPage {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>, species: Vec<String>) -> Self {
        TrainingPage::with_icons(memory, names, species, ICONS)
    }

    pub fn with_icons(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        species: Vec<String>,
        icons: i64,
    ) -> Self {
        TrainingPage {
            memory,
            names,
            species,
            icons,
            pages: IndexMap::new(),
        }
    }

    /// The pages a sweep found, by class: all of them, or with `adding` the new ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut pages = if adding {
            self.pages.clone()
        } else {
            IndexMap::new()
        };
        for (&page, name) in found {
            if name == CLASS {
                pages.insert(page, u32(&self.memory.read(page, 4)?, 0)?);
            }
        }
        self.pages = pages;
        Ok(())
    }

    /// The species of the party member whose icon a picture draws, or None.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        if self.pages.is_empty() || self.species.is_empty() {
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
        let params = self.icons + PARAMS;
        for (&page, &vtable) in &self.pages {
            let head = memory.read(page, params + PARAM_STRIDE * SLOTS)?;
            // A page freed and its memory taken by something else says nothing.
            if u32(&head, 0)? != vtable {
                continue;
            }
            let Some(slot) = self.slot(u32(&head, self.icons)?, image)? else {
                continue;
            };
            let mut record = None;
            for r in 0..SLOTS {
                if u32(&head, params + PARAM_STRIDE * r + INDEX)? == slot {
                    record = Some(r);
                    break;
                }
            }
            let Some(record) = record else {
                return Ok(None);
            };
            let found = pokemon::read(
                memory.as_ref(),
                &self.names,
                u32(&head, params + PARAM_STRIDE * record)?,
                pokemon::SPECIES_MAX,
            )?;
            let Some(found) = found else {
                return Ok(None);
            };
            if found.species >= self.species.len() as i64 {
                return Ok(None);
            }
            let name = &self.species[found.species as usize];
            return Ok(if name.is_empty() {
                None
            } else {
                Some(name.clone())
            });
        }
        Ok(None)
    }

    /// Which of a PokeIcon's textures, as far as the party goes, draws this image, or None
    /// where it draws none or a link of the chain is not the class it should be.
    fn slot(&self, icons: i64, image: i64) -> Result<Option<i64>> {
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
        let table = memory.read(listed, 4 * count.min(SLOTS))?;
        for i in 0..(table.len() as i64 / 4) {
            if u32(&memory.read(u32(&table, 4 * i)? + d::IMAGE, 4)?, 0)? == image {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }
}

/// (pane, root, image, across, up) of every shown picture of a shown layout on the top or the
/// bottom screen whose image `wanted` says yes to: its own place less its root's, as touch mode
/// places a touch area, from the screen's middle.
pub fn pictures(
    layouts: &Layouts,
    image_of: &ImageOf,
    top: bool,
    wanted: &dyn Fn(&str) -> bool,
) -> Result<Vec<(i64, i64, String, f64, f64)>> {
    let shape = layouts.shape;
    let mut out = Vec::new();
    let mut roots: Vec<i64> = layouts.roots().into_iter().collect();
    roots.sort();
    for root in roots {
        let Some(placement) = layouts.placement(root)? else {
            continue;
        };
        if !placement.shown || placement.top != top {
            continue;
        }
        let head = layouts.pane(root)?;
        for (pane, kind) in layouts.tree(root) {
            let shown = layouts.placement(pane)?;
            if kind != lyt::PICTURE_CLASS || !shown.is_some_and(|shown| shown.shown) {
                continue;
            }
            let image = image_of(pane)?.unwrap_or_default();
            if wanted(&image) {
                let here = layouts.pane(pane)?;
                out.push((
                    pane,
                    root,
                    image,
                    f32(&here, shape.x)? - f32(&head, shape.x)?,
                    f32(&here, shape.y)? - f32(&head, shape.y)?,
                ));
            }
        }
    }
    Ok(out)
}

/// The goals a round of Super Training shows, as touch targets on the bottom screen.
pub struct Goals {
    layouts: Rc<Layouts>,
    image_of: ImageOf,
}

impl Goals {
    pub fn new(layouts: Rc<Layouts>, image_of: ImageOf) -> Self {
        Goals { layouts, image_of }
    }

    /// The goals up on the top screen while a round runs, each at the pixel that shoots it.
    pub fn targets(&self, running: Option<&str>) -> Result<Vec<Target>> {
        if running != Some(SPARRING) {
            return Ok(Vec::new());
        }
        let layouts = &self.layouts;
        let mut found: Vec<Target> = Vec::new();
        for (pane, _root, image, across, up) in pictures(layouts, &self.image_of, true, &|image| {
            GOAL_FRAME.is_match(image)
        })? {
            if let Some((x, y)) = aim(
                TOP_WIDTH as f64 / 2.0 + across,
                TOP_HEIGHT as f64 / 2.0 - up,
            )? {
                let word = own(
                    if image.starts_with(BIG_GOAL_IMAGE) {
                        RED_GOAL
                    } else {
                        GOAL
                    },
                    &[],
                );
                found.push(Target::new(
                    pane,
                    &word,
                    x,
                    y,
                    up,
                    f32(&layouts.pane(pane)?, layouts.shape.height)?,
                ));
            }
        }
        Ok(found)
    }
}

/// The training bags the page's list shows, as touch targets named by the game's own words.
pub struct Bags {
    layouts: Rc<Layouts>,
    image_of: ImageOf,
    /// An image -> the game's word for it (`touch_words`).
    words: IndexMap<String, String>,
    /// The works of the layouts whose buttons take no tap now (`Buttons.idle`).
    idle: Option<Idle>,
    /// (work, pane name) of the buttons switched off now (`Buttons.off`): each bag is one, by
    /// its picture, off under the page's Yes and No (spike 228).
    off: Option<Off>,
}

impl Bags {
    pub fn new(
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        words: IndexMap<String, String>,
        idle: Option<Idle>,
        off: Option<Off>,
    ) -> Self {
        Bags {
            layouts,
            image_of,
            words,
            idle,
            off,
        }
    }

    /// The bags on the touch screen while the field runs, each where it is drawn.
    pub fn targets(&self, running: Option<&str>) -> Result<Vec<Target>> {
        if running != Some(FIELD) {
            return Ok(Vec::new());
        }
        let layouts = &self.layouts;
        let mut found: Vec<Target> = Vec::new();
        let idle = match &self.idle {
            Some(idle) => idle()?,
            None => Default::default(),
        };
        let off = match &self.off {
            Some(off) => off()?,
            None => Default::default(),
        };
        let works: IndexMap<i64, i64> = layouts
            .works()
            .into_iter()
            .map(|(work, root)| (root, work))
            .collect();
        for (pane, root, image, across, up) in pictures(layouts, &self.image_of, false, &|image| {
            image.starts_with(BAG_IMAGE)
        })? {
            // The list is a layout like any other: under a menu it takes no tap (`TouchTargets`).
            let work = works.get(&root).copied();
            let raw = layouts.name(pane)?;
            let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
            let name = py::decode_ascii(&raw[..end]);
            if layouts.taken_off(root)?
                || work.is_some_and(|work| idle.contains(&work))
                || work.is_some_and(|work| off.contains(&(work, name)))
            {
                continue;
            }
            let word = self.words.get(&image).filter(|word| !word.is_empty());
            let (x, y) = (
                py::round(WIDTH as f64 / 2.0 + across)?,
                py::round(HEIGHT as f64 / 2.0 - up)?,
            );
            if let Some(word) = word
                && (0..WIDTH).contains(&x)
                && (0..HEIGHT).contains(&y)
            {
                found.push(Target {
                    layer: layouts.layer(root)?,
                    root,
                    ..Target::new(
                        pane,
                        word,
                        x,
                        y,
                        up,
                        f32(&layouts.pane(pane)?, layouts.shape.height)?,
                    )
                });
            }
        }
        Ok(found)
    }
}

/// `sorted((x, what) ...)`: Python's tuple order, x first and then the words.
fn by_place(items: Vec<(f64, String)>) -> Vec<(f64, String)> {
    py::sorted_by_lt(items, |a, b| if a.0 != b.0 { a.0 < b.0 } else { a.1 < b.1 })
}

/// The score and the target of the row of digits around the shown slash, or None: the digits
/// on the screen within `SAME_ROW` of the slash's y, those left of it the score and those right
/// of it the target, each in x order.
pub fn read_score(seen: &[Seen]) -> Result<Option<(i64, i64)>> {
    let marks: Vec<(String, f64, f64)> = seen
        .iter()
        .filter(|(_, x, y)| on_top(*x, *y))
        .filter_map(|(image, x, y)| {
            SCORE_DIGIT
                .captures(image)
                .map(|m| (m[1].to_string(), *x, *y))
        })
        .collect();
    let Some(slash) = marks
        .iter()
        .find(|(what, _, _)| what == "slash")
        .map(|(_, x, y)| (*x, *y))
    else {
        return Ok(None);
    };
    let row = by_place(
        marks
            .iter()
            .filter(|(what, _, y)| what != "slash" && (y - slash.1).abs() <= SAME_ROW)
            .map(|(what, x, _)| (*x, what.clone()))
            .collect(),
    );
    let score: String = row
        .iter()
        .filter(|(x, _)| *x < slash.0)
        .map(|(_, what)| what.as_str())
        .collect();
    let target: String = row
        .iter()
        .filter(|(x, _)| *x > slash.0)
        .map(|(_, what)| what.as_str())
        .collect();
    if score.is_empty() || target.is_empty() {
        return Ok(None);
    }
    Ok(Some((py::int_base10(&score)?, py::int_base10(&target)?)))
}

/// The seconds left on the clock, or None: the digits on the screen within `SAME_ROW` of the
/// clock face's y and right of it, in x order, the first the minutes and the rest the seconds.
pub fn read_clock(seen: &[Seen]) -> Result<Option<i64>> {
    let marks: Vec<(String, f64, f64)> = seen
        .iter()
        .filter(|(_, x, y)| on_top(*x, *y))
        .filter_map(|(image, x, y)| {
            CLOCK_DIGIT.captures(image).map(|m| {
                let what = m.get(1).or_else(|| m.get(2)).map_or("", |g| g.as_str());
                (what.to_string(), *x, *y)
            })
        })
        .collect();
    let Some(face) = marks
        .iter()
        .find(|(what, _, _)| what == "timer")
        .map(|(_, x, y)| (*x, *y))
    else {
        return Ok(None);
    };
    let digits: String = by_place(
        marks
            .iter()
            .filter(|(what, x, y)| what != "timer" && *x > face.0 && (y - face.1).abs() <= SAME_ROW)
            .map(|(what, x, _)| (*x, what.clone()))
            .collect(),
    )
    .into_iter()
    .map(|(_, what)| what)
    .collect();
    let chars: Vec<char> = digits.chars().collect();
    if chars.len() <= 1 {
        return Ok(None);
    }
    let minutes = py::int_base10(&chars[0].to_string())?;
    let seconds = py::int_base10(&chars[1..].iter().collect::<String>())?;
    Ok(Some(60 * minutes + seconds))
}

/// The words of a banner's picture, "game over" for `gameover_c.bclim`, or None for any
/// other.
pub fn banner(image: &str) -> Option<String> {
    let stem = match image.rfind('.') {
        Some(at) => &image[..at],
        None => image,
    };
    let stem = stem.strip_suffix("_c").unwrap_or(stem);
    BANNERS.contains(&stem).then(|| picture_words(stem))
}

/// What a round of Super Training says as it goes: the score as it changes, the clock every
/// half minute, its banners, and a warning as a Balloon Bot's ball leaves.
pub struct Round {
    layouts: Rc<Layouts>,
    image_of: ImageOf,
    memory: Rc<dyn Memory>,
    now: Rc<dyn Fn() -> f64>,
    /// Ball -> the vtable it was found with.
    missiles: IndexMap<i64, i64>,
    /// The score last said, or first seen.
    score: Option<(i64, i64)>,
    /// When a score was last said.
    score_at: Option<f64>,
    /// The score the last poll read.
    reading: Option<(i64, i64)>,
    /// The seconds the clock read last.
    left: Option<i64>,
    /// The banners shown at the last poll.
    banners: BTreeSet<String>,
    /// Ball -> its state at the last poll.
    states: IndexMap<i64, i64>,
}

impl Round {
    /// `now` None is `time.monotonic`.
    pub fn new(
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        memory: Rc<dyn Memory>,
        now: Option<Rc<dyn Fn() -> f64>>,
    ) -> Self {
        let mut round = Round {
            layouts,
            image_of,
            memory,
            now: now.unwrap_or_else(|| Rc::new(py::time::monotonic)),
            missiles: IndexMap::new(),
            score: None,
            score_at: None,
            reading: None,
            left: None,
            banners: BTreeSet::new(),
            states: IndexMap::new(),
        };
        round.forget();
        round
    }

    /// Start a round over: nothing is known of the last.
    fn forget(&mut self) {
        self.score = None;
        self.score_at = None;
        self.reading = None;
        self.left = None;
        self.banners = BTreeSet::new();
        self.states = IndexMap::new();
    }

    /// The balls a sweep found, by class: all of them, or with `adding` the new ones.
    pub fn track(&mut self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        let mut missiles = if adding {
            self.missiles.clone()
        } else {
            IndexMap::new()
        };
        for (&ball, name) in found {
            if name == MISSILE {
                missiles.insert(ball, u32(&self.memory.read(ball, 4)?, 0)?);
            }
        }
        self.missiles = missiles;
        Ok(())
    }

    /// What to say now, with its category, while a round runs: nothing otherwise.
    pub fn poll(&mut self, running: Option<&str>) -> Result<Vec<(String, Category)>> {
        if running != Some(SPARRING) {
            self.forget();
            return Ok(Vec::new());
        }
        let seen: Vec<Seen> = pictures(&self.layouts, &self.image_of, true, &|image| {
            SCORE_DIGIT.is_match(image) || CLOCK_DIGIT.is_match(image) || banner(image).is_some()
        })?
        .into_iter()
        .map(|(_pane, _root, image, across, up)| {
            (
                image,
                TOP_WIDTH as f64 / 2.0 + across,
                TOP_HEIGHT as f64 / 2.0 - up,
            )
        })
        .collect();
        // The warning first: it interrupts, and the lines after it queue behind it.
        let mut out = Vec::new();
        if self.launched()? {
            out.push((own(SHOT, &[]), Category::Warning));
        }
        out.extend(self.hear(&seen)?);
        Ok(out)
    }

    /// The lines for the pictures shown now, said as the screen's words.
    pub fn hear(&mut self, seen: &[Seen]) -> Result<Vec<(String, Category)>> {
        let mut lines = self.say_score(read_score(seen)?);
        lines.extend(self.say_clock(read_clock(seen)?));
        lines.extend(self.say_banners(seen));
        Ok(lines
            .into_iter()
            .map(|line| (line, Category::Screen))
            .collect())
    }

    fn say_score(&mut self, score: Option<(i64, i64)>) -> Vec<String> {
        // Read the same on two polls first: a hit from 0 to 177 showed its last digit a frame
        // before the others, and was said "7 of 300" (spike 200, live).
        let steady = score == self.reading;
        self.reading = score;
        let Some(score) = score.filter(|_| steady) else {
            return Vec::new();
        };
        if self.score.is_none() {
            self.score = Some(score);
            return Vec::new();
        }
        let now = (self.now)();
        if Some(score) == self.score
            || self
                .score_at
                .is_some_and(|score_at| now - score_at < SCORE_EVERY)
        {
            return Vec::new();
        }
        self.score = Some(score);
        self.score_at = Some(now);
        vec![format!("{} of {}", score.0, score.1)]
    }

    /// The latest mark the clock has reached or passed since it was last read.
    fn say_clock(&mut self, left: Option<i64>) -> Vec<String> {
        let Some(left) = left else {
            return Vec::new();
        };
        let last = self.left;
        self.left = Some(left);
        let passed: Vec<i64> = CLOCK_MARKS
            .iter()
            .copied()
            .filter(|&mark| last.is_some_and(|last| left <= mark && mark < last))
            .collect();
        let Some(&mark) = passed.iter().min() else {
            return Vec::new();
        };
        vec![own(
            "training-left",
            &[(
                "time",
                &format!("{}:{:02}", py::floordiv(mark, 60), py::modulo(mark, 60)),
            )],
        )]
    }

    fn say_banners(&mut self, seen: &[Seen]) -> Vec<String> {
        let shown: BTreeSet<String> = seen
            .iter()
            .filter_map(|(image, _x, _y)| banner(image).filter(|said| !said.is_empty()))
            .collect();
        let new: Vec<String> = shown.difference(&self.banners).cloned().collect();
        self.banners = shown;
        new
    }

    /// Has a ball of the Bot's gone into flight since the last poll?
    fn launched(&mut self) -> Result<bool> {
        let mut launched = false;
        let missiles: Vec<(i64, i64)> = self.missiles.iter().map(|(&b, &v)| (b, v)).collect();
        for (ball, vtable) in missiles {
            let head = self.memory.read(ball, MISSILE_HEAD)?;
            // A ball freed and its memory taken by something else says nothing.
            if u32(&head, 0)? != vtable {
                self.states.shift_remove(&ball);
                continue;
            }
            let (state, before) = (u32(&head, MISSILE_STATE)?, self.states.get(&ball).copied());
            self.states.insert(ball, state);
            if state == FLYING
                && before.is_some_and(|before| before != FLYING)
                && u32(&head, BOT_BALL)? != 0
            {
                launched = true;
            }
        }
        Ok(launched)
    }
}
