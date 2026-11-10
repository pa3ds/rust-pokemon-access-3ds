//! Super Training: its page, the PlayNav's in ORAS and the bottom screen's own in X, names its
//! party slots, a round offers its goals to tap, and the field offers the training bags in the
//! page's list (omega_ruby_bottom-screen5).
//!
//! The page shows the party as six pictures, each a 40 by 30 button with a touch area, and the game
//! names none: touch mode said "mapnews icon 02" and "pmn 1pix org" by the address of the image, or
//! left them out. They are the icons of a `common::resource::PokeIcon` (`screens.distribution`,
//! `subscreen.amie`), in the party's order. The page's object is a class of its module, Game Freak's
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
//! Use and Back. With no bag in use the one hanging beside the Pokémon draws `sand_bag00.bclim`, 40
//! by 100, which has no line in the game's text: offered as "empty bag", as tapping it can find a
//! bag.
//!
//! Core Training (as_super_training_bags, spike A): a bag hangs from its top, so its place is the
//! top of it, and a tap there does nothing; a tap on the bag or on the Pokémon makes it hit the bag.
//! A hit takes one from the Pokémon's record's +0x16 of the bags' hits left (2 for a Strength Bag),
//! and with none left the page's +0x264 goes from 1, idle (6 while it hits), to 4 and the game draws
//! `surprise_icon.bclim`, a "!" over the Pokémon, until a tap on the Pokémon gives the bag's effect
//! and says so. Left alone the Pokémon hits the bag once a minute. The page says nothing else of it.
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
//! +0x04   u32: 1 while flying, 2 once it has hit, 3 once a barrier stopped it, 0 unused
//! +0x10   f32 x, y, z: z runs from about 8 at the Bot to 227.5 at the player
//! +0x10C  u32 frames in flight: 3 to 0x41, about 1.1 s
//! +0x110  u32 a pointer: set on the Bot's two, 0 on the pool (the player's shots, presumably)
//! ```
//!
//! One of the Bot's flew three times in 15 s, and the score fell 177, 167, 157 as they hit. The
//! player blocks a ball with L in the second before it lands. A shot of the player's own with
//! +0x110 set would warn as well; not seen.
//!
//! Super Training's welcome, "Welcome to S.T.!" over its page, is a message the field shows on the
//! bottom screen, waiting for a tap to go on (as_play_nav, spike 253). The page switches every
//! button of its own off under it (spike 228), and the frame round the PlayNav's pages keeps its
//! arrows and the X menu's icons on: touch mode offered those alone, and "next", the frame's ▶, went
//! on with the message and turned the PlayNav to the page after, leaving Super Training behind.
//! While a message waits, the game draws its ▼ in the message's corner, `f_msg_win_c01.bclim`, at
//! (305, 63) for the welcome, on no touch area. Touch mode offers the ▼ as "continue", where a tap
//! lands on no button (`Waiting`): any message of the field's bottom screen that draws one, the
//! welcome the only one met so far.
//!
//! The page's chart is said on a key, from the record's effort values (`pokemon.EFFORT`).
//!
//! Bound by:
//! - ADR-0017: the chart is said by each stat's rough share, its points only with the Extra on.

use std::cell::RefCell;
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
use crate::games::common::field::keys::{ROUND_NUMBERS, TRAINING_PAGE, TRAINING_ROUND};
use crate::games::common::layout::Layouts;
use crate::games::common::pictures::words as picture_words;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::{ImageOf, Target};
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::screens::distribution as d;
use crate::games::gen6::subscreen::amie::{METER, PAGE_APPS};
use crate::games::gen6::subscreen::berry_picker::Touching;
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
/// Pokémon alike.
pub const BAG_IMAGE: &str = "sand_bag";
/// The bag beside the Pokémon when none is in use, which the game's text never names.
pub const EMPTY_BAG: &str = "sand_bag00.bclim";
pub const EMPTY_BAG_WORD: &str = "training-empty-bag";
/// How far above the screen's middle a second empty bag waits, out of sight behind the top bar, to
/// drop in: at (90, 0), where the one hanging is at (50, 56).
pub const WAITING_BAG: f64 = 110.0;
/// Panes of the Super Training page's layout, by which it is told, its file being none pa3ds
/// finds: its two buttons' touch areas, the shot's icon, the mark over the Pokémon, the bag waiting.
///
/// A `frozenset` in Python, only ever tested against.
pub const PAGE_PANES: [&str; 5] = ["2o", "2p", "3v", "4b", "4e"];
/// Where after its PokeIcon the page keeps what its Pokémon is doing, +0x264 in ORAS (spike A).
pub const STATE: i64 = 0x174;
/// Where after its PokeIcon the page says a hit is landing on the bag, +0x128 in ORAS: 1 for about
/// two fifths of a second a hit, and 0 between them (spike A).
pub const HITTING: i64 = 0x38;
/// Which of the record's effort values (`pokemon.EFFORT`) each of the game's six labels is, in the
/// order it writes them: HP, Attack, Defense, Sp. Atk, Sp. Def, Speed.
pub const STAT_ORDER: [usize; 6] = [0, 1, 2, 4, 5, 3];
/// The points the chart holds of one stat.
pub const STAT_FULL: i64 = 252;
/// The points its bar holds of them all: a Fully Trained Pokémon's.
pub const TOTAL_FULL: i64 = 510;
/// What the texture's name of the kind of shot's picture starts with.
pub const SHOT_ICON: &str = "shot_icon0";
/// That state, from when the Pokémon has finished its bag until it is tapped: the "!" over it.
pub const FINISHED: i64 = 4;
/// The states measured: idle, finished, hitting, and showing what a special bag (7) or a stat's
/// (8) has given. Any other is logged, to learn what it is.
pub const STATES_SEEN: [i64; 5] = [1, 4, 6, 7, 8];
pub const BAG_FINISHED: &str = "training-bag-finished";
pub const STAT: &str = "training-stat";
pub const TOTAL: &str = "training-total";
pub const TOTAL_POINTS: &str = "training-total-points";
pub const NO_NUMBERS: &str = "training-no-numbers";
/// A round's energy meter, low in the middle of the top screen: its scale is how full it is, 0 to 1
/// (as_super_training_tutorial2). The first tutorial's round draws none.
pub const ENERGY_IMAGE: &str = "st_gauge_c.bclim";
pub const ENERGY: &str = "training-energy";
/// The aura round the Pokémon while a press held is charged, from about 1.9 s into the press for
/// about 1.5 s: let go in it for a shot worth three times the points (spike A: 21, 63 on Wailmer).
pub const CHARGE_IMAGE: &str = "charge01.bclim";
pub const CHARGED: &str = "training-charged";
pub const FIZZLED: &str = "training-fizzled";
/// The regimen menu's count of the Pokémon's medals, written beside a picture of one.
///
/// Python's `$` also matches before a newline at the end, so `\n?` before it here.
pub static MEDAL_COUNT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^x (\d+)\n?$").expect("a fixed pattern"));
/// The ▼ a bottom-screen message draws while it waits for a tap (`Waiting`).
pub const ARROW: &str = "f_msg_win_c01.bclim";
pub const CONTINUE: &str = "touch-continue";
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
/// How far apart across two digits of the score may be, or its last and the slash, in pixels: 20
/// and 15 as measured.
pub const DIGIT_GAP: f64 = 30.0;
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
/// What the log calls what a round says.
pub const ROUND_LOG: &str = "round";
/// What the log calls what the Super Training page says.
pub const PAGE_LOG: &str = "training";
pub const MISSILE: &str = "N5actor20SparringMissileActorE";
pub const MISSILE_STATE: i64 = 0x04;
/// A ball's state in flight; 2 once it has hit the player, 0 unused.
pub const FLYING: i64 = 1;
/// A ball's state once the player's barrier has stopped it: with L held as it arrived, and no
/// points lost, where one that hit went to 2 and took 8 (as_super_training_tutorial and 2).
pub const STOPPED: i64 = 3;
pub const BLOCKED: &str = "training-blocked";
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

/// What Super Training's page says of its selected Pokémon.
#[derive(Debug, Clone, PartialEq)]
pub struct PageNow {
    /// What it is doing (`STATE`).
    pub state: i64,
    /// Whether a hit is landing on the bag (`HITTING`).
    pub hitting: bool,
    /// Its `PokemonParam`.
    pub param: i64,
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
    /// What the heap is swept for: the page, which says whose each party slot's icon is.
    pub const CLASSES: [&'static str; 1] = [CLASS];

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

    /// What the page's Pokémon is doing and who it is, or None with no page.
    pub fn selected(&self) -> Result<Option<PageNow>> {
        let params = self.icons + PARAMS;
        for (&page, &vtable) in &self.pages {
            let head = self.memory.read(page, self.icons + STATE + 4)?;
            // Of the objects of this class only the page's holds a PokeIcon.
            let (icons, param) = (u32(&head, self.icons)?, u32(&head, params)?);
            if u32(&head, 0)? == vtable
                && icons != 0
                && param != 0
                && self.names.of_object(icons)?.as_deref() == Some(d::ICON_CLASS)
            {
                return Ok(Some(PageNow {
                    state: u32(&head, self.icons + STATE)?,
                    hitting: u32(&head, self.icons + HITTING)? != 0,
                    param,
                }));
            }
        }
        Ok(None)
    }

    /// The Pokémon the page has selected.
    pub fn who(&self, now: &PageNow) -> Result<Option<pokemon::Pokemon>> {
        pokemon::read(
            self.memory.as_ref(),
            &self.names,
            now.param,
            pokemon::SPECIES_MAX,
        )
    }

    /// The selected Pokémon's effort values, in the order the game labels them (`STAT_ORDER`),
    /// or None with no page.
    pub fn efforts(&self) -> Result<Option<Vec<i64>>> {
        let points = match self.selected()? {
            None => None,
            Some(now) => pokemon::efforts(self.memory.as_ref(), &self.names, now.param)?,
        };
        Ok(points.map(|points| STAT_ORDER.iter().map(|&i| points[i]).collect()))
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

    /// Keep a press held on a goal on it as it drifts: nothing to say.
    ///
    /// Touch mode, which the navigator owns, is handed here, where Python's `Goals` keeps a
    /// callable returning it; None is Python's without one.
    pub fn poll(
        &self,
        running: Option<&str>,
        touch: Option<&mut dyn Touching>,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        if running == Some(SPARRING)
            && let Some(touch) = touch
            && touch.holding()
        {
            touch.follow()?;
        }
        Ok(Vec::new())
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
                found.push(Target {
                    held: true,
                    ..Target::new(
                        pane,
                        &word,
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

/// The training bags the page's list shows, pictures with no touch area of their own, offered
/// by the game's own words.
pub struct Bags {
    layouts: Rc<Layouts>,
    image_of: ImageOf,
    /// An image -> the game's word for it (`touch_words`).
    words: IndexMap<String, String>,
}

impl Bags {
    pub fn new(layouts: Rc<Layouts>, image_of: ImageOf, words: IndexMap<String, String>) -> Self {
        Bags {
            layouts,
            image_of,
            words,
        }
    }

    /// The bags shown on the bottom screen while the field runs, by pane, with their words.
    /// Touch mode places each and leaves it out as it does a touch area: under a menu, under the
    /// page's Yes and No, which switch each bag's picture off (spike 228), and off the screen.
    pub fn offer(&self, running: Option<&str>) -> Result<IndexMap<i64, String>> {
        if running != Some(FIELD) {
            return Ok(IndexMap::new());
        }
        let mut out: IndexMap<i64, String> = IndexMap::new();
        for (pane, _root, image, _across, up) in
            pictures(&self.layouts, &self.image_of, false, &|image| {
                image.starts_with(BAG_IMAGE)
            })?
        {
            if let Some(word) = self.word(&image, up).filter(|word| !word.is_empty()) {
                out.insert(pane, word);
            }
        }
        Ok(out)
    }

    fn word(&self, image: &str, up: f64) -> Option<String> {
        if image == EMPTY_BAG {
            return (up < WAITING_BAG).then(|| own(EMPTY_BAG_WORD, &[]));
        }
        self.words.get(image).cloned()
    }
}

/// Says when the Pokémon on the Super Training page has finished its bag, which the game shows
/// only as a "!" over it, and a tap on it answers. Read only while the page is drawn.
pub struct CoreTraining {
    page: Rc<RefCell<TrainingPage>>,
    layouts: Rc<Layouts>,
    called: Rc<dyn Fn(&pokemon::Pokemon) -> Option<String>>,
    /// Played as each hit lands on the bag: nothing else says one has (Orion, 2026-11-09).
    hit: Rc<dyn Fn()>,
    /// Played as the "!" comes up, easily missed among the taps (Orion, 2026-11-09).
    finished: Rc<dyn Fn()>,
    /// Whether this "!" has been said.
    said: bool,
    /// Whether the last poll saw the Pokémon finished: a tap on the bag that finishes it goes
    /// straight on to what it gave, with the state there for one poll and no "!" to tap.
    waiting: bool,
    /// Whether a hit was landing at the last poll.
    hitting: bool,
    /// The last state logged as not measured.
    logged: Option<i64>,
}

impl CoreTraining {
    /// `hit` and `finished` play nothing.
    pub fn new(
        page: Rc<RefCell<TrainingPage>>,
        layouts: Rc<Layouts>,
        called: Rc<dyn Fn(&pokemon::Pokemon) -> Option<String>>,
    ) -> Self {
        CoreTraining::with(page, layouts, called, Rc::new(|| {}), Rc::new(|| {}))
    }

    pub fn with(
        page: Rc<RefCell<TrainingPage>>,
        layouts: Rc<Layouts>,
        called: Rc<dyn Fn(&pokemon::Pokemon) -> Option<String>>,
        hit: Rc<dyn Fn()>,
        finished: Rc<dyn Fn()>,
    ) -> Self {
        CoreTraining {
            page,
            layouts,
            called,
            hit,
            finished,
            said: false,
            waiting: false,
            hitting: false,
            logged: None,
        }
    }

    /// Is the page's layout drawn on the bottom screen?
    pub fn drawn(&self) -> Result<bool> {
        for root in self.layouts.roots() {
            let names = self.layouts.pane_names(root);
            if names.iter().any(|name| name == "4b")
                && PAGE_PANES
                    .iter()
                    .all(|pane| names.iter().any(|name| name == pane))
            {
                let placement = self.layouts.placement(root)?;
                if placement.is_some_and(|placement| placement.shown && !placement.top) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// The line for a bag just finished, with its sound, and each hit's sound; a hit and any
    /// state not measured to the log alone.
    pub fn poll(
        &mut self,
        running: Option<&str>,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        let now = if running == Some(FIELD) && self.drawn()? {
            self.page.borrow().selected()?
        } else {
            None
        };
        if now.as_ref().is_none_or(|now| now.state != FINISHED) {
            self.said = false;
        }
        let Some(now) = now else {
            self.hitting = false;
            self.waiting = false;
            return Ok(Vec::new());
        };
        let mut lines = Vec::new();
        if now.hitting && !self.hitting {
            (self.hit)();
            lines.push((PAGE_LOG.to_string(), "hit".to_string(), None));
        }
        self.hitting = now.hitting;
        if !STATES_SEEN.contains(&now.state) && Some(now.state) != self.logged {
            lines.push((PAGE_LOG.to_string(), format!("state {}", now.state), None));
        }
        self.logged = Some(now.state);
        let waited = self.waiting;
        self.waiting = now.state == FINISHED;
        if waited && now.state == FINISHED && !self.said {
            self.said = true;
            (self.finished)();
            let who = self.page.borrow().who(&now)?;
            let name = who.as_ref().and_then(|who| (self.called)(who));
            if let Some(name) = name.filter(|name| !name.is_empty()) {
                lines.push((
                    PAGE_LOG.to_string(),
                    own(BAG_FINISHED, &[("pokemon", &name)]),
                    Some(Category::Screen),
                ));
            }
        }
        Ok(lines)
    }
}

/// What R says on Super Training's page and in its rounds, the numbers a screen draws and does
/// not say, as it does Head It's score and a contest's places (Orion, 2026-11-09). The page's
/// chart is said by each stat's rough share, or with the Extra on by its points (ADR-0017).
pub struct TrainingKeys {
    core: Rc<RefCell<CoreTraining>>,
    page: Rc<RefCell<TrainingPage>>,
    round: Rc<RefCell<Round>>,
    /// What the game calls the six stats, in the order it writes them.
    stats: Vec<String>,
    exact: bool,
    layouts: Rc<Layouts>,
    image_of: ImageOf,
    wrong: Rc<dyn Fn(&str) -> String>,
}

impl TrainingKeys {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        core: Rc<RefCell<CoreTraining>>,
        page: Rc<RefCell<TrainingPage>>,
        sparring: Rc<RefCell<Round>>,
        stats: Vec<String>,
        exact: bool,
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        wrong: Rc<dyn Fn(&str) -> String>,
    ) -> Self {
        TrainingKeys {
            core,
            page,
            round: sparring,
            stats,
            exact,
            layouts,
            image_of,
            wrong,
        }
    }

    /// The keys' table in the keys file on the page or in a round, else None.
    ///
    /// Python's reads the page's layouts with no exception to catch; a read that fails here is
    /// no table, as `Keyed.context` cannot raise.
    pub fn context(&self, running: Option<&str>) -> Result<Option<&'static str>> {
        if running == Some(SPARRING) {
            return Ok(Some(TRAINING_ROUND));
        }
        Ok((running == Some(FIELD) && self.core.borrow().drawn()?).then_some(TRAINING_PAGE))
    }

    /// The numbers of the screen running, with the line to say.
    pub fn press(&self, action: &str, _running: Option<&str>) -> Result<String> {
        let said = if action == ROUND_NUMBERS {
            self.round.borrow().numbers()?
        } else {
            self.numbers()?
        };
        // `running` only ever asked on the screen its table is for (`context`)
        if said.is_empty() {
            return Ok((self.wrong)(&own(NO_NUMBERS, &[])));
        }
        Ok(said)
    }

    fn stat(&self, stat: &str, points: i64) -> Result<String> {
        Ok(if self.exact {
            own(STAT, &[("stat", &stat), ("points", &points.to_string())])
        } else {
            own(
                METER,
                &[("label", &stat), ("count", &fifths(points, STAT_FULL)?)],
            )
        })
    }

    fn numbers(&self) -> Result<String> {
        let Some(efforts) = self.page.borrow().efforts()? else {
            return Ok(String::new());
        };
        if efforts.len() != self.stats.len() {
            return Ok(String::new());
        }
        let total: i64 = efforts.iter().sum();
        let mut said = Vec::new();
        for (stat, &points) in self.stats.iter().zip(&efforts) {
            said.push(self.stat(stat, points)?);
        }
        said.push(if self.exact {
            own(
                TOTAL_POINTS,
                &[
                    ("points", &total.to_string()),
                    ("full", &TOTAL_FULL.to_string()),
                ],
            )
        } else {
            own(
                METER,
                &[
                    ("label", &own(TOTAL, &[])),
                    ("count", &fifths(total, TOTAL_FULL)?),
                ],
            )
        });
        for (_pane, _root, image, _x, _y) in
            pictures(&self.layouts, &self.image_of, false, &|image| {
                image.starts_with(SHOT_ICON)
            })?
        {
            said.push(picture_words(&image));
        }
        Ok(said.join(", "))
    }
}

/// How many fifths of a stat's chart its points fill, what the chart shows at a glance: "HP 2
/// of 5" (Orion, 2026-11-09, over words such as "a lot"). 0 only with none, 5 only when full.
fn fifths(points: i64, full: i64) -> Result<i64> {
    if points <= 0 {
        return Ok(0);
    }
    Ok(if points >= full {
        5
    } else {
        py::ceil((points * 5) as f64 / full as f64)?.min(4)
    })
}

/// The regimen menu's count of the Pokémon's medals, which the game writes "x 0" beside a
/// picture of a medal, in words.
pub struct Medals {
    running: Rc<dyn Fn() -> Option<String>>,
}

impl Medals {
    pub fn new(running: Rc<dyn Fn() -> Option<String>>) -> Self {
        Medals { running }
    }

    /// The words on the screen, the medal count's said as one.
    pub fn reworded(&self, found: IndexMap<i64, ScreenText>) -> Result<IndexMap<i64, ScreenText>> {
        if (self.running)().as_deref() != Some(MENU) {
            return Ok(found);
        }
        let mut out = IndexMap::new();
        for (r#box, piece) in found {
            let count = match MEDAL_COUNT.captures(&piece.text) {
                Some(m) => Some(py::int_base10(&m[1])?),
                None => None,
            };
            out.insert(
                r#box,
                match count {
                    Some(count) => ScreenText {
                        text: own("training-medals", &[("count", &count)]),
                        ..piece
                    },
                    None => piece,
                },
            );
        }
        Ok(out)
    }
}

/// `sorted((x, what) ...)`: Python's tuple order, x first and then the words.
fn by_place(items: Vec<(f64, String)>) -> Vec<(f64, String)> {
    py::sorted_by_lt(items, |a, b| if a.0 != b.0 { a.0 < b.0 } else { a.1 < b.1 })
}

/// The bottom screen's message waiting for a tap, offered at its ▼: Super Training's
/// welcome.
pub struct Waiting {
    layouts: Rc<Layouts>,
    image_of: ImageOf,
}

impl Waiting {
    pub fn new(layouts: Rc<Layouts>, image_of: ImageOf) -> Self {
        Waiting { layouts, image_of }
    }

    /// The ▼ of each message waiting on the field's bottom screen, "continue".
    pub fn offer(&self, running: Option<&str>) -> Result<IndexMap<i64, String>> {
        if running != Some(FIELD) {
            return Ok(IndexMap::new());
        }
        let mut out: IndexMap<i64, String> = IndexMap::new();
        for (pane, ..) in pictures(&self.layouts, &self.image_of, false, &|image| {
            image == ARROW
        })? {
            out.insert(pane, own(CONTINUE, &[]));
        }
        Ok(out)
    }
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
    let score = running_up_to(
        slash.0,
        &row.iter()
            .filter(|d| d.0 < slash.0)
            .cloned()
            .collect::<Vec<_>>(),
    );
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

/// The digits running unbroken up to the slash, the score's right-aligned against it, left to
/// right. A shot taking 8 points pops "-8" up at y 200, level with the row at y 203 but 100 pixels
/// from it, and read "80 of 50" for a score of 0 (as_super_training_tutorial).
fn running_up_to(slash: f64, digits: &[(f64, String)]) -> String {
    let mut run: Vec<&str> = Vec::new();
    let mut at = slash;
    for (x, what) in digits.iter().rev() {
        if at - x > DIGIT_GAP {
            break;
        }
        run.push(what);
        at = *x;
    }
    run.into_iter().rev().collect()
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
    /// How full the energy meter was at the last poll, 0 to 1; None where none is drawn.
    energy: Option<f64>,
    /// Whether the aura of a press held was drawn at the last poll.
    charged: bool,
    /// What is said of a Bot's shot, "" where it was heard as a sound instead.
    warned: Rc<dyn Fn(&str) -> String>,
    /// Whether the player's press is still down on a goal (`TouchMode.holding`).
    holding: Rc<dyn Fn() -> bool>,
}

/// `Round.__init__`'s keyword arguments after `memory`, each as the Python defaults it: set what
/// differs with `..RoundOptions::default()`.
#[derive(Default)]
pub struct RoundOptions {
    /// None is `time.monotonic`.
    pub now: Option<Rc<dyn Fn() -> f64>>,
    /// None says each shot as it is.
    pub warned: Option<Rc<dyn Fn(&str) -> String>>,
    /// None is never.
    pub holding: Option<Rc<dyn Fn() -> bool>>,
}

impl Round {
    /// What the heap is swept for: the Balloon Bots' balls, which say when a shot is coming.
    pub const CLASSES: [&'static str; 1] = [MISSILE];

    /// `now` None is `time.monotonic`.
    pub fn new(
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        memory: Rc<dyn Memory>,
        now: Option<Rc<dyn Fn() -> f64>>,
    ) -> Self {
        Round::with(
            layouts,
            image_of,
            memory,
            RoundOptions {
                now,
                ..RoundOptions::default()
            },
        )
    }

    pub fn with(
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        memory: Rc<dyn Memory>,
        options: RoundOptions,
    ) -> Self {
        let RoundOptions {
            now,
            warned,
            holding,
        } = options;
        let mut round = Round {
            layouts,
            image_of,
            memory,
            now: now.unwrap_or_else(|| Rc::new(py::time::monotonic)),
            warned: warned.unwrap_or_else(|| Rc::new(|said: &str| said.to_string())),
            holding: holding.unwrap_or_else(|| Rc::new(|| false)),
            missiles: IndexMap::new(),
            score: None,
            score_at: None,
            reading: None,
            left: None,
            banners: BTreeSet::new(),
            states: IndexMap::new(),
            energy: None,
            charged: false,
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
        self.energy = None;
        self.charged = false;
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

    /// What to say now, with what the log calls it and its category, while a round runs:
    /// nothing otherwise.
    pub fn poll(
        &mut self,
        running: Option<&str>,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        let holding = self.holding.clone();
        self.poll_with(running, &*holding)
    }

    /// `poll`, asking `holding` whether the press is down where Python asks its own: touch mode,
    /// which the navigator owns, is handed to the subscreens' poll.
    pub fn poll_with(
        &mut self,
        running: Option<&str>,
        holding: &dyn Fn() -> bool,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        if running != Some(SPARRING) {
            self.forget();
            return Ok(Vec::new());
        }
        let layouts = self.layouts.clone();
        let mut seen: Vec<Seen> = Vec::new();
        let mut charged = false;
        for (pane, _root, image, across, up) in
            pictures(&layouts, &self.image_of, true, &|image| {
                image == ENERGY_IMAGE
                    || image == CHARGE_IMAGE
                    || SCORE_DIGIT.is_match(image)
                    || CLOCK_DIGIT.is_match(image)
                    || banner(image).is_some()
            })?
        {
            if image == ENERGY_IMAGE {
                self.energy = Some(f32(&layouts.pane(pane)?, layouts.shape.scale_x)?);
            } else if image == CHARGE_IMAGE {
                charged = true;
            } else {
                seen.push((
                    image,
                    TOP_WIDTH as f64 / 2.0 + across,
                    TOP_HEIGHT as f64 / 2.0 - up,
                ));
            }
        }
        // The warning first: it interrupts, and the lines after it queue behind it. With the
        // game's "!" played in its place it is left to the log (Orion, 2026-11-09).
        let mut said: Vec<(String, String, Option<Category>)> = Vec::new();
        let (launched, blocked) = self.balls()?;
        if launched {
            let category = if (self.warned)(&own(SHOT, &[])).is_empty() {
                None
            } else {
                Some(Category::Warning)
            };
            said.push((ROUND_LOG.to_string(), own(SHOT, &[]), category));
        }
        if blocked {
            said.push((
                ROUND_LOG.to_string(),
                own(BLOCKED, &[]),
                Some(Category::Screen),
            ));
        }
        said.extend(self.say_charge(charged, holding));
        for (line, category) in self.hear(&seen)? {
            said.push((ROUND_LOG.to_string(), line, Some(category)));
        }
        Ok(said)
    }

    /// "charged" as the aura comes, for the player to let go in it, and "fizzled" if it goes
    /// with the press still down; let go in it, it is gone as the shot flies, with nothing said.
    fn say_charge(
        &mut self,
        charged: bool,
        holding: &dyn Fn() -> bool,
    ) -> Vec<(String, String, Option<Category>)> {
        let was = self.charged;
        self.charged = charged;
        if charged && !was {
            return vec![(
                ROUND_LOG.to_string(),
                own(CHARGED, &[]),
                Some(Category::Warning),
            )];
        }
        if was && !charged && holding() {
            return vec![(
                ROUND_LOG.to_string(),
                own(FIZZLED, &[]),
                Some(Category::Warning),
            )];
        }
        Vec::new()
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

    /// The score, the clock and the energy meter as last read, "34 of 50, 1:28 left, energy 4
    /// of 5", or "" before they show; the first regimen draws no meter.
    pub fn numbers(&self) -> Result<String> {
        let mut said: Vec<String> = Vec::new();
        if let Some(reading) = self.reading {
            said.push(format!("{} of {}", reading.0, reading.1));
        }
        if let Some(left) = self.left {
            said.push(own(
                "training-left",
                &[(
                    "time",
                    &format!("{}:{:02}", py::floordiv(left, 60), py::modulo(left, 60)),
                )],
            ));
        }
        if let Some(energy) = self.energy {
            let fifths = fifths(py::round(energy * 100.0)?, 100)?;
            said.push(own(
                METER,
                &[("label", &own(ENERGY, &[])), ("count", &fifths)],
            ));
        }
        Ok(said.join(", "))
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

    /// Has a ball of the Bot's gone into flight since the last poll, and has one been stopped
    /// by the L button's barrier?
    fn balls(&mut self) -> Result<(bool, bool)> {
        let (mut launched, mut blocked) = (false, false);
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
            if u32(&head, BOT_BALL)? != 0 {
                launched |= state == FLYING && before.is_some_and(|before| before != FLYING);
                blocked |= state == STOPPED && before == Some(FLYING);
            }
        }
        Ok((launched, blocked))
    }
}
