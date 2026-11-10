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
//! keys move over the Pokémon while Amie runs (`StylusKeys`), or jump with Ctrl to the next part,
//! says the part it comes onto by a name of pa3ds's own for that Pokémon's or form's part where it
//! has one (`amie.ftl`), else by its kind (`PARTS`); it moves with that part as the Pokémon moves, as
//! far as the part does (`AmiePage.follow`), and keeps near the Pokémon (`AmiePage.outline`); and
//! touch mode's tap or drag on it rubs back and forth across the part (`AmiePage.rub`), carried on as
//! it moves.
//!
//! What the Pokémon is doing is the **love command** running, a routine and a class each, which
//! `Player` points at (spike A). `Moods` says what a sighted player sees of it: its back turned, its
//! looking at the stylus, its hand raised, its eating. A press held off the Pokémon brings the
//! raised hand, a high five's, waving about two seconds, so Ctrl and any arrow then jump the stylus
//! straight to it (`AmiePage.raised_hand`).
//!
//! Bound by:
//! - ADR-0009: the stylus the arrows move, tapped, rubbed or dragged only by touch mode's keys, and
//!   the beckon a tap on the Pokémon (as amended 2026-11-05); a jump goes to a hand raised
//!   (2026-11-06).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::IndexMap;

use crate::core::binary::{s16, u32};
use crate::core::discovery::{HEAP_SPAN, LINEAR_SPAN, within};
use crate::core::heaps;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::core::words::{current, own};
use crate::games::common::field::keys::{STYLUS, STYLUS_JUMPS, STYLUS_MOVES};
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
pub const COLUMN: i64 = 0x400;
/// Where the screen's picture starts in a buffer, how far apart its columns are, and how many
/// pixels that is.
pub const PER_COLUMN: i64 = COLUMN / 4;
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
pub const MODEL: i64 = 0x84;
pub const MODEL_RESOURCE: i64 = 0x28;
pub const RECORDS: i64 = 0x70;
/// `Player` +0x84 -> the Pokémon's shader model, +0x28 -> its resource, +0x70 -> its records,
/// the first's +0x5C -> the model's name (spike 279).
pub const RECORD_NAME: i64 = 0x5C;
/// A Pokémon's model by name, `pm0025_12`: its species' own number, then its number among that
/// species' models, which tells a form drawn unlike the rest, Pikachu Rock Star's 12. Some names run
/// on past the 12 bytes read, `pm0701_12_poodle`, Furfrou Heart Trim's. Forms drawn alike but for
/// colour share a name, Arceus's every type, and so do a few drawn unlike, Burmy's cloaks and East
/// Shellos: those are named by their species alone (spike A's census). Python's `MODEL_NAME`,
/// `pm\d{4}_(\d{2})` matched at the start: the model's number, or None.
pub fn model_name(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let digits = |from: usize, to: usize| chars[from..to].iter().all(|&c| py::is_decimal(c));
    (chars.len() >= 9
        && chars[0] == 'p'
        && chars[1] == 'm'
        && digits(2, 6)
        && chars[6] == '_'
        && digits(7, 9))
    .then(|| chars[7..9].iter().collect())
}
/// How long the band of the picture round the stylus is taken as read (`WINDOW`): an arrow press
/// asks for it twice, held an arrow repeats some thirty times a second, and the stylus is kept on
/// its part every poll.
pub const FRESH: f64 = 0.1;
/// How many columns of the picture either side of the stylus are read at once: 65 of 1 KB each.
pub const WINDOW: i64 = 32;
/// How far the part the stylus is on is looked for once it has moved off it.
pub const NEAR: i64 = 20;
/// How far round the stylus the edges of its part are compared from one read of the picture to
/// the next, to tell how far the part moved (`moved`).
pub const TRACK: i64 = 20;
pub const SHIFT: i64 = 10;
/// How far a part is looked for having moved between two reads, a tenth of a second apart
/// (`FRESH`), either way; the nearest first, which a tie goes to.
pub static SHIFTS: LazyLock<Vec<(i64, i64)>> = LazyLock::new(|| {
    let mut shifts: Vec<(i64, i64)> = (-SHIFT..SHIFT + 1)
        .flat_map(|dx| (-SHIFT..SHIFT + 1).map(move |dy| (dx, dy)))
        .collect();
    // Python's `sorted` by an int key, which is stable, as `sort_by_key` is.
    shifts.sort_by_key(|d| d.0.pow(2) + d.1.pow(2));
    shifts
});
/// How far either side of the stylus a rub goes at most, on its part: at 20, rubbing Pikachu's
/// chin reached its cheeks (Orion, or_amie_moods).
pub const RUB_REACH: i64 = 12;
/// How far a rub keeps in from its part's edges, as the Pokémon sways under it.
pub const RUB_INSET: i64 = 3;
/// How long the whole picture is taken as read, for the Pokémon's outline and a jump.
pub const WHOLE: f64 = 0.5;
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
/// Each grey by the kind of part it is, a message each (spike A), where the Pokémon has no name of
/// pa3ds's own for it in `amie.ftl` (Orion, 2026-11-05: "our own name a Pokémon", then "cheeks for
/// Pikachu, tail for the Charmander line"). 2 and 3 are the ears, the Pokémon's own right and left,
/// as 4 and 6 are its eyes; 10, where it high-fives, a hand or paw on most, a mouth on a few
/// (spike A's census). 25 to 28, the danger zone and the sweet spot, are said as the head and the
/// body, the side of the pair each is: the game's own sound tells them apart, its pitch down on a
/// danger zone and up on a sweet spot (`SEQ_SE_KW_STROKE` at 108 and 148 against 128). Any other
/// grey is said as a part, `amie-part`: 1 among them, seen only as specks at Victini's ears and
/// under Manaphy's head.
pub const PARTS: [(i64, &str); 26] = [
    (2, "amie-part-right-ear"),
    (3, "amie-part-left-ear"),
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
/// The grey a Pokémon high-fives with: a hand or arm on most, a mouth on a few (spike A).
pub const HAND: i64 = 10;
pub const LOVE: i64 = 0x1B8;
/// `Player` +0x1B8 -> the love command running, none for a moment between two, and its number, an
/// s16 at +0xC (spike A).
pub const LOVE_NUMBER: i64 = 0xC;
/// `LoveLookTouch`: a press held off the Pokémon a few seconds raises its hand, longer the lower
/// its affection (Pikachu Rock Star's 60 took 4.8 s), which it waves about 2 s, about a second a
/// wave, and raises again 5 s later while the press stays; the stylus on grey 10 meanwhile is the
/// high five (spike A, live).
pub const HAND_RAISED: i64 = 24;
/// `LoveFoodIdle`: a puff held out, and where eating goes to when more of it is left.
pub const NEXT_BITE: i64 = 15;
/// What the Pokémon is seen doing as a love command starts, a message each: `LoveAppear`, its back
/// to the player until beckoned round; `LoveFoodEat`'s first; `LoveLook`, watching a press held;
/// `LoveLookAround`. Left unsaid: 0 `LoveMotion`, the idle one, 13 `LoveTouch`, a pet, the player's
/// own doing, and 15, 18 and 19, a puff held out and the rest of eating. 14, 16, 20 and 27 have
/// not been seen, nor is it known which of `LoveHate`, `LovePicky`, `LoveFoodFall`, `LoveHome`,
/// `LoveCamera` and `LoveMotionRepeat` each is.
pub const MOODS: [(i64, &str); 5] = [
    (7, "amie-mood-back-turned"),
    (17, "amie-mood-eating"),
    (22, "amie-mood-watching"),
    (23, "amie-mood-looking-around"),
    (HAND_RAISED, "amie-mood-hand-raised"),
];
/// What is seen as a love command ends: turned round, and at the end of eating a puff, the last
/// of `LoveFoodEat`'s, where `NEXT_BITE` does not follow.
pub const MOODS_OVER: [(i64, &str); 2] = [(7, "amie-mood-turned-round"), (19, "amie-mood-eaten")];
/// What the log calls a line of `Moods`.
pub const MOOD_LOG: &str = "amie mood";
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
    /// A byte a pixel, its grey, column by column from the bottom up as `data` is: (x, y)
    /// at `at`.
    pub greys: Vec<u8>,
}

impl Band {
    pub fn new(first: i64, data: Vec<u8>) -> Self {
        let last = first + data.len() as i64 / COLUMN - 1;
        let greys = data.iter().skip(1).step_by(4).copied().collect();
        Band {
            first,
            data,
            last,
            greys,
        }
    }

    /// Where (x, y) is in `greys`, which `PER_COLUMN` apart a column.
    pub fn at(&self, x: i64, y: i64) -> i64 {
        (x - self.first) * PER_COLUMN + touch::HEIGHT - 1 - y
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

/// How far the part at (x, y) in one read of the picture moved by the next: the shift within
/// `SHIFT` that lines up the most of its edges within `TRACK` of (x, y), the pixels on both sides
/// of them, with the later read; (0, 0) where no shift lines up more than none, or no edge of the
/// part is that near (Orion, 2026-11-06: "if the pokemon moved 3 pixels, the stylus moves 3
/// pixels in the same direction").
pub fn moved(before: &Band, after: &Band, x: i64, y: i64) -> (i64, i64) {
    let (old, new) = (&before.greys, &after.greys);
    let mine = old[before.at(x, y) as usize];
    let mut edges: Vec<(i64, u8)> = Vec::new();
    let across = (x - TRACK).max(before.first + 1).max(after.first + SHIFT)
        ..(x + TRACK).min(before.last - 1).min(after.last - SHIFT) + 1;
    for px in across {
        for py in (y - TRACK).max(SHIFT)..(y + TRACK).min(touch::HEIGHT - 1 - SHIFT) + 1 {
            let at = before.at(px, py);
            let grey = old[at as usize];
            let around = [at + 1, at - 1, at + PER_COLUMN, at - PER_COLUMN];
            if around
                .iter()
                .any(|&near| (old[near as usize] == mine) != (grey == mine))
            {
                edges.push((after.at(px, py), grey));
            }
        }
    }
    if edges.is_empty() {
        return (0, 0);
    }
    let lined_up = |shift: (i64, i64)| -> usize {
        let off = shift.0 * PER_COLUMN - shift.1;
        edges
            .iter()
            .filter(|&&(at, grey)| new[(at + off) as usize] == grey)
            .count()
    };
    if lined_up((0, 0)) == edges.len() {
        return (0, 0);
    }
    // Python's `max(SHIFTS, key=lined_up)`: the first of equal bests.
    let mut best = (SHIFTS[0], lined_up(SHIFTS[0]));
    for &shift in &SHIFTS[1..] {
        let count = lined_up(shift);
        if count > best.1 {
            best = (shift, count);
        }
    }
    best.0
}

/// Names the picture of the Pokémon the Pokémon-Amie page shows by its species, and offers
/// Amie's Pokémon to beckon and its stylus to rub with.
pub struct AmiePage {
    memory: Rc<dyn Memory>,
    /// How much later than ORAS's `Player` keeps its fields (`Game.amie_player`).
    later: i64,
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
    /// The grey of the part the stylus is kept on, `OFF_PICTURE` for none, None before it
    /// has been on any.
    kept: Option<i64>,
    /// The Player the Pokémon's model was read through, and its number (`form`).
    form: (Option<i64>, Option<String>),
    /// The Player the picture was last read through (`_buffer`): the sweep finds a copy of
    /// its words in the module's code as well, which leads to no model.
    live: Option<i64>,
    /// When the whole picture was read, for the outline and a jump, and the picture.
    whole: (f64, Option<Rc<Band>>),
    /// When the band of the picture round the stylus was read, and the band.
    band: (f64, Option<Rc<Band>>),
    /// The band the stylus was last moved with its part by (`follow`).
    tracked: Option<Rc<Band>>,
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
        AmiePage::with_aim(memory, names, species, number, false, 0)
    }

    /// `AmiePage(...)` with every argument.
    pub fn with_aim(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        species: Vec<String>,
        number: i64,
        aimed: bool,
        later: i64,
    ) -> Self {
        AmiePage {
            memory,
            later,
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
            form: (None, None),
            live: None,
            whole: (f64::NEG_INFINITY, None),
            band: (f64::NEG_INFINITY, None),
            tracked: None,
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
        let part = self.part(None)?;
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

    /// A grey's part, or the part the stylus is on, in pa3ds's words: the Pokémon's own name
    /// for it, its form's first (`amie.ftl`, `form`), else the kind of part (`PARTS`); "off"
    /// where it is off the Pokémon, or None where the picture cannot be read.
    pub fn part(&mut self, grey: Option<i64>) -> Result<Option<String>> {
        let grey = match grey {
            None => self.grey()?,
            grey => grey,
        };
        let Some(grey) = grey else {
            return Ok(None);
        };
        if grey == 0 || grey == OFF_PICTURE {
            return Ok(Some(own(OFF, &[])));
        }
        let (words, form) = (current(), self.form()?);
        let named = form
            .and_then(|form| words.get(&format!("amie-spot-{}-{form}-{grey}", self.number), &[]));
        Ok(Some(
            named
                .filter(|named| !named.is_empty())
                .or_else(|| words.get(&format!("amie-spot-{}-{grey}", self.number), &[]))
                .filter(|named| !named.is_empty())
                .unwrap_or_else(|| {
                    own(
                        PARTS
                            .iter()
                            .find(|(kind, _)| *kind == grey)
                            .map_or(PART, |(_, word)| word),
                        &[],
                    )
                }),
        ))
    }

    /// The model Amie draws the Pokémon with, by its number among its species' own,
    /// `"12"` for Pikachu Rock Star's `pm0025_12`: `Player` +0x84 leads to its shader model,
    /// whose resource's +0x70 lists its records, the first named for the model (spike 279).
    /// Read once a Player, the one the picture is read through; None where it cannot be.
    pub fn form(&mut self) -> Result<Option<String>> {
        let Some(player) = self.live else {
            return Ok(None);
        };
        if self.form.0 != Some(player) {
            let memory = &self.memory;
            let read = || -> Result<Option<String>> {
                let model = u32(&memory.read(player + MODEL + self.later, 4)?, 0)?;
                let resource = if model != 0 {
                    u32(&memory.read(model + MODEL_RESOURCE, 4)?, 0)?
                } else {
                    0
                };
                let records = if resource != 0 {
                    u32(&memory.read(resource + RECORDS, 4)?, 0)?
                } else {
                    0
                };
                let named = if records != 0 {
                    u32(&memory.read(records + RECORD_NAME, 4)?, 0)?
                } else {
                    0
                };
                let text = if named != 0 {
                    let raw = memory.read(named, 12)?;
                    raw.split(|&byte| byte == 0)
                        .next()
                        .unwrap_or_default()
                        .to_vec()
                } else {
                    Vec::new()
                };
                Ok(model_name(&py::decode_latin1(&text)))
            };
            let name = match read() {
                Ok(name) => name,
                Err(error) if error.is(Kind::RpcError) => None,
                Err(error) => return Err(error),
            };
            self.form = (Some(player), name);
        }
        Ok(self.form.1.clone())
    }

    /// Keep the stylus on a grey (`follow`), or on the part it is on now, as the arrows leave
    /// it; on nothing where it is off the Pokémon.
    pub fn keep(&mut self, grey: Option<i64>) -> Result<()> {
        let grey = match grey {
            None => self.grey()?,
            grey => grey,
        };
        if let Some(grey) = grey {
            self.kept = Some(if grey == 0 || grey == OFF_PICTURE {
                OFF_PICTURE
            } else {
                grey
            });
        }
        Ok(())
    }

    /// Move the stylus with the part under it as the Pokémon moves, as far as the part moved
    /// since the picture was last read (`moved`); and where the part has gone from under it all
    /// the same, onto the part's pixel nearest it within `NEAR`, the middle of its run across.
    /// The part it is kept on is the one the arrows last left it on, or before any, the first it
    /// is found on: chosen by touch mode's keys alone, it had kept to nothing (or_amie_moods).
    /// Whether it moved.
    pub fn follow(&mut self) -> Result<bool> {
        if self.kept.is_none() {
            self.keep(None)?;
        }
        let (before, band) = (self.tracked.clone(), self.read()?);
        self.tracked = band.clone();
        let Some(band) = band else {
            return Ok(false);
        };
        let start = self.stylus;
        let (mut x, mut y) = start;
        if let Some(before) = before
            && !Rc::ptr_eq(&band, &before)
            && before.grey(x, y).is_some()
        {
            let (dx, dy) = moved(&before, &band, x, y);
            x = (x + dx).clamp(0, touch::WIDTH - 1);
            y = (y + dy).clamp(0, touch::HEIGHT - 1);
            self.stylus = (x, y);
        }
        if let Some(kept) = self.kept
            && kept != OFF_PICTURE
            && band.grey(x, y) != Some(kept)
            && let Some(found) = band.nearest(x, y, kept, NEAR)
        {
            let (left, right) = band.run(found.0, found.1, true);
            self.stylus = (py::floordiv(left + right, 2), found.1);
        }
        Ok(self.stylus != start)
    }

    /// Where a rub with the stylus goes: back and forth across the part it is on, the way the
    /// part is longer, `RUB_REACH` at most either side of it and `RUB_HALF` at least; where the
    /// picture cannot be read, `RUB_HALF` either side of where it is. Off the Pokémon it stays
    /// where it is: a press held there raises its hand, and any rub reaching back onto it was
    /// taken for a press on it, which never does (spike A, live).
    pub fn rub(&mut self) -> Result<Ends> {
        let band = self.read()?;
        let (x, y) = self.stylus;
        let band = match band {
            Some(band) if band.grey(x, y).is_some() => band,
            _ => return Ok(((x - RUB_HALF, y), (x + RUB_HALF, y))),
        };
        if matches!(band.grey(x, y), Some(0 | OFF_PICTURE)) {
            return Ok(((x, y), (x, y)));
        }
        let (across, down) = (band.run(x, y, true), band.run(x, y, false));
        let sideways = across.1 - across.0 >= down.1 - down.0;
        let ((low, high), at) = if sideways { (across, x) } else { (down, y) };
        let half = RUB_REACH.min(RUB_HALF.max(py::floordiv(high - low, 2) - RUB_INSET));
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

    /// The box the Pokémon is drawn in, (left, top, right, bottom), from the whole picture;
    /// None where it cannot be read or nothing is drawn. The arrows keep the stylus within a
    /// step of it: across the whole screen it was too many presses (Orion, 2026-11-05).
    pub fn outline(&mut self) -> Result<Option<(i64, i64, i64, i64)>> {
        let Some(whole) = self.read_whole()? else {
            return Ok(None);
        };
        let (mut columns, mut rows) = (Vec::new(), Vec::new());
        for x in 0..touch::WIDTH {
            let column = py::slice(&whole.data, x * COLUMN, x * COLUMN + touch::HEIGHT * 4);
            let on: Vec<i64> = column
                .as_chunks::<4>()
                .0
                .iter()
                .enumerate()
                .filter(|(_, [alpha, grey, green, blue])| {
                    *alpha == 0xFF && grey == green && green == blue && *grey != 0
                })
                .map(|(i, _)| touch::HEIGHT - 1 - i as i64)
                .collect();
            if !on.is_empty() {
                columns.push(x);
                rows.extend(on);
            }
        }
        let (Some(&first), Some(&last)) = (columns.first(), columns.last()) else {
            return Ok(None);
        };
        Ok(Some((
            first,
            *rows.iter().min().expect("a column drawn"),
            last,
            *rows.iter().max().expect("a column drawn"),
        )))
    }

    /// The love command running, by its number; None between two, or where no Player runs.
    pub fn mood(&self) -> Result<Option<i64>> {
        for (&player, &vtable) in &self.players {
            if self.holder(player, vtable)?.is_none() {
                continue;
            }
            let command = u32(&self.memory.read(player + LOVE + self.later, 4)?, 0)?;
            if within(HEAP_SPAN, command) {
                return Ok(Some(s16(&self.memory.read(command + LOVE_NUMBER, 2)?, 0)?));
            }
        }
        Ok(None)
    }

    /// Where a jump goes while the Pokémon's hand is raised, whichever way: a player cannot
    /// see where it is, and it is up about 2 s (`HAND_RAISED`). The middle of the run across of
    /// the hand's pixel nearest the stylus, in the picture as it is now, as the hand waves;
    /// None where it is not raised or not drawn, or the picture cannot be read.
    pub fn raised_hand(&mut self) -> Result<Option<(i64, i64)>> {
        if self.mood()? != Some(HAND_RAISED) {
            return Ok(None);
        }
        self.whole = (f64::NEG_INFINITY, None);
        let whole = self.read_whole()?;
        let Some(whole) = whole.filter(|whole| whole.greys.contains(&(HAND as u8))) else {
            return Ok(None);
        };
        let (x, y) = self.stylus;
        let mut drawn: Option<(i64, i64, i64)> = None;
        for px in 0..touch::WIDTH {
            for py in 0..touch::HEIGHT {
                let found = ((px - x).pow(2) + (py - y).pow(2), px, py);
                if i64::from(whole.greys[whole.at(px, py) as usize]) == HAND
                    && whole.grey(px, py) == Some(HAND)
                    && drawn.is_none_or(|best| found < best)
                {
                    drawn = Some(found);
                }
            }
        }
        let Some((_distance, x, y)) = drawn else {
            return Ok(None);
        };
        let (left, right) = whole.run(x, y, true);
        Ok(Some((py::floordiv(left + right, 2), y)))
    }

    /// Where the stylus jumps to from where it is, the way given: past the part it is on and
    /// any gap, to the middle of the next part's run that way, as Ctrl and an arrow move it;
    /// None where no other part lies that way, or the picture cannot be read.
    pub fn jump(&mut self, across: i64, down: i64) -> Result<Option<(i64, i64)>> {
        let Some(whole) = self.read_whole()? else {
            return Ok(None);
        };
        let (mut x, mut y) = self.stylus;
        let here = whole.grey(x, y);
        while let Some(grey) = whole.grey(x + across, y + down) {
            (x, y) = (x + across, y + down);
            if Some(grey) != here && grey != 0 && grey != OFF_PICTURE {
                let first = if across != 0 { x } else { y };
                while whole.grey(x + across, y + down) == Some(grey) {
                    (x, y) = (x + across, y + down);
                }
                let middle = py::floordiv(first + if across != 0 { x } else { y }, 2);
                return Ok(Some(if across != 0 {
                    (middle, y)
                } else {
                    (x, middle)
                }));
            }
        }
        Ok(None)
    }

    /// The whole picture, read again once `WHOLE` has gone by; None where it cannot be.
    fn read_whole(&mut self) -> Result<Option<Rc<Band>>> {
        let now = py::time::monotonic();
        if let (when, Some(whole)) = &self.whole
            && now - when < WHOLE
        {
            return Ok(Some(whole.clone()));
        }
        let buffer = self.buffer()?;
        let mut whole = None;
        if let Some(buffer) = buffer {
            whole = match self.memory.read(buffer + PICTURE, touch::WIDTH * COLUMN) {
                Ok(data) => Some(Rc::new(Band::new(0, data))),
                Err(error) if error.is(Kind::RpcError) => None,
                Err(error) => return Err(error),
            };
        }
        self.whole = (now, whole.clone());
        Ok(whole)
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
    fn buffer(&mut self) -> Result<Option<i64>> {
        if !self.aim {
            return Ok(None);
        }
        let players: Vec<(i64, i64)> = self.players.iter().map(|(&p, &v)| (p, v)).collect();
        for (player, vtable) in players {
            let Some(holder) = self.holder(player, vtable)? else {
                continue;
            };
            let buffers = self.memory.read(holder + BUFFERS, 12)?;
            let which = u32(&buffers, 8)?;
            let buffer = if which == 0 || which == 1 {
                u32(&buffers, 4 * which)?
            } else {
                0
            };
            if within(LINEAR_SPAN, buffer) {
                self.live = Some(player);
                return Ok(Some(buffer));
            }
        }
        Ok(None)
    }

    /// Each Player running, with its picture's holder. Python's `_holders` is a generator over
    /// the Players; here each caller walks them and asks this of each in turn, so the reads are
    /// made as lazily.
    fn holder(&self, player: i64, vtable: i64) -> Result<Option<i64>> {
        let head = self.memory.read(player, HOLDS_PICTURE + self.later + 4)?;
        let holder = u32(&head, HOLDS_PICTURE + self.later)?;
        // A Player freed, or a copy of its words in the module's code, leads nowhere.
        Ok((u32(&head, 0)? == vtable
            && within(HEAP_SPAN, holder)
            && self.names.of_object(holder)?.as_deref() == Some(PICTURE_HOLDER))
        .then_some(holder))
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

/// Says what Amie's Pokémon is seen doing as it starts and as it stops (`MOODS`,
/// `MOODS_OVER`), which a sighted player sees and the game's sounds tell only in part.
pub struct Moods {
    page: Rc<RefCell<AmiePage>>,
    /// The love command at the last poll.
    mood: Option<i64>,
}

impl Moods {
    pub fn new(page: Rc<RefCell<AmiePage>>) -> Self {
        Moods { page, mood: None }
    }

    /// What the Pokémon is seen doing, where its love command changed since the last poll.
    pub fn poll(
        &mut self,
        running: Option<&str>,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        let before = self.mood;
        self.mood = if running == Some(AMIE) {
            self.page.borrow().mood()?
        } else {
            None
        };
        if running != Some(AMIE) || self.mood == before {
            return Ok(Vec::new());
        }
        let word = |table: &[(i64, &'static str)], number: i64| {
            table
                .iter()
                .find(|(key, _)| *key == number)
                .map(|(_, word)| *word)
        };
        let over = if self.mood == Some(NEXT_BITE) {
            None
        } else {
            word(&MOODS_OVER, before.unwrap_or(0))
        };
        Ok([over, word(&MOODS, self.mood.unwrap_or(0))]
            .into_iter()
            .flatten()
            .map(|word| (MOOD_LOG.to_string(), own(word, &[]), Some(Category::Screen)))
            .collect())
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

    /// Move the stylus a step, or jump it to the next part, with the line to say.
    pub fn press(
        &mut self,
        action: &str,
        _running: Option<&str>, // only ever asked while Amie runs (`context`)
        touch: &mut dyn Touching,
    ) -> Result<String> {
        let page = self.page.clone();
        page.borrow_mut().follow()?; // a step from where the part has moved it to since the last poll
        let jump = STYLUS_JUMPS
            .iter()
            .find(|(name, _)| *name == action)
            .map(|(_, way)| *way);
        let hand = if jump.is_some() {
            page.borrow_mut().raised_hand()?
        } else {
            None
        };
        let grey = hand.map(|_| HAND); // what it lands on, gone by the time it is asked
        let moved = if let Some((across, down)) = jump {
            let moved = match hand {
                Some(hand) => Some(hand),
                None => page.borrow_mut().jump(across, down)?,
            };
            let Some(moved) = moved else {
                return Ok((self.wrong)(&own(EDGE, &[])));
            };
            self.said = None; // a jump always says where it lands
            moved
        } else {
            let (_, (across, down)) = *STYLUS_MOVES
                .iter()
                .find(|(name, _)| *name == action)
                .ok_or_else(|| Error::key(py::repr(action)))?;
            let (x, y) = page.borrow().stylus;
            let (left, top, right, bottom) =
                page.borrow_mut()
                    .outline()?
                    .unwrap_or((0, 0, touch::WIDTH - 1, touch::HEIGHT - 1));
            let moved = (
                (x + across * STEP)
                    .max(left - STEP)
                    .max(0)
                    .min(right + STEP)
                    .min(touch::WIDTH - 1),
                (y + down * STEP)
                    .max(top - STEP)
                    .max(0)
                    .min(bottom + STEP)
                    .min(touch::HEIGHT - 1),
            );
            if moved == (x, y) {
                return Ok((self.wrong)(&own(EDGE, &[])));
            }
            moved
        };
        page.borrow_mut().stylus = moved;
        page.borrow_mut().keep(grey)?;
        touch.choose(STYLUS_TARGET)?;
        let part = page.borrow_mut().part(grey)?;
        if part == self.said {
            return Ok(String::new());
        }
        self.said = part.clone();
        Ok(part.unwrap_or_default())
    }

    /// Keep the stylus on its part as the Pokémon moves, carrying a rub on with it, and its
    /// ends as the part moves under it; says nothing, and chooses nothing in touch mode.
    pub fn poll(
        &self,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        if running == Some(AMIE) {
            self.page.borrow_mut().follow()?;
            touch.carry(STYLUS_TARGET)?;
        }
        Ok(Vec::new())
    }
}
