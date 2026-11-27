//! The Pokémon-Amie page, the PlayNav's in ORAS and the bottom screen's own in X, whose Pokémon is a
//! picture named by its species (spike 198); and Amie itself. Its decorate screen is
//! `amie_decorate`'s, and the stylus's geometry `amie_stylus`'s.
//!
//! After the tutorial the page shows one Pokémon of the player's walking in its ring, a 40 by 30
//! touch area on a picture of its own, and the game names neither: touch mode said "touch: 2 of
//! layout 0x15a024bc has no name" and left it out, so the player could not open Amie at all
//! (omega_ruby_bottom-screen1). The picture draws texture 0 of a `common::resource::PokeIcon`, the
//! class the AreaNav's Pokémon Distribution and the Bag draw their icons with
//! (`gen6.pokeicon`, `screens.partyicons`). The page's object is a class of its module, Game
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
//! the hand it draws: a burn on 15 and 16, a shock on 17 and 18 (spike 287's census of every model in
//! the ROM, and its listing of the script). It counts a pet only while the hand moves 4 pixels a tick
//! and the press stays down, a lift starting its count afresh. So the **stylus**, a point the arrow
//! keys move over the Pokémon while Amie runs (`StylusKeys`), or jump with Ctrl to the next part,
//! says the part it comes onto by a name of pa3ds's own for that Pokémon's or form's part where it
//! has one (`amie.ftl`), else by its kind (`PARTS`); it keeps near the Pokémon (`Stylus.outline`);
//! and touch mode's tap or drag on it rubs back and forth across the part (`Stylus.rub`), carried
//! on as it moves.
//!
//! A Pokémon left alone moves about a good deal: Pikachu's box moved 85 pixels in 40 s, so a part
//! found with the arrows had gone by the next press. So the stylus moves over a **still picture** of
//! it, taken as it appears (`Stylus.settle`), its parts staying where the player found them, and a
//! tap or drag goes to the part chosen wherever the Pokémon has moved it to
//! (`amie_stylus.placed`). A Pokémon too big for the bottom screen stays on the top screen, a strip
//! of it showing, until a press held at the touch screen's top edge brings it down; touch mode offers that as a target of its own (`AmiePage._view`, `BOTH_SCREENS`), and the
//! still picture is taken anew.
//!
//! What the Pokémon is doing is the **love command** running, a routine and a class each, which
//! `Player` points at (spike 287). `Moods` says what a sighted player sees of it, where the player asks
//! (`[pa3ds.amie] descriptions`): its back turned, its watching the hand, its hand raised, its
//! eating. A press held off the Pokémon brings the raised hand, a high five's, waving about two
//! seconds, so Ctrl and any arrow then jump the stylus straight to it (`Stylus.to_hand`).
//!
//! Bound by:
//! - ADR-0009: the stylus the arrows move, tapped, rubbed or dragged only by touch mode's keys, and
//!   the beckon a tap on the Pokémon (as amended 2026-11-05); a jump goes to a hand raised
//!   (2026-11-06), a touch to the part named wherever it has moved, and the Pokémon is brought
//!   between the screens by a target of its own (2026-11-06).
//! - ADR-0038: the moods are descriptions of Amie's own, in sentences, with no narrator (as amended
//!   2026-11-06).

use std::cell::{OnceCell, RefCell};
use std::fmt::Display;
use std::rc::{Rc, Weak};

use indexmap::IndexMap;

use crate::core::binary::{s16, u16, u32};
use crate::core::discovery::{HEAP_SPAN, LINEAR_SPAN, within};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::core::words::{Values, current, own};
use crate::games::common::field::keys::{STYLUS, STYLUS_JUMPS, STYLUS_MOVES, STYLUS_NUMBERS};
use crate::games::common::touch::{After, Target};
use crate::games::common::{lyt, pokemon, touch};
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::pokeicon;
use crate::games::gen6::screens::townmap;
use crate::games::gen6::subscreen::amie_stylus::{
    COLUMN, HAND, HAND_RAISED, LOOK, OFF_PICTURE, Picture, SETTLE, Stylus,
};
use crate::games::gen6::subscreen::berry_picker::{Name, Touching};
use crate::py::{self, Error, Kind, PySet, Result};

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
/// module's code holds copies of its first words, which a sweep finds as well (spike 279). Where it
/// keeps each field read is a game's own (`PlayerFields`).
pub const PLAYER: &str = "N9kawaigari6PlayerE";
/// The class `PlayerFields.holds_picture` points at, whose `BUFFERS` hold the picture.
pub const PICTURE_HOLDER: &str = "N3gfl4heap11NwAllocatorE";
/// The picture's two buffers, and after them which is current, 0 or 1.
pub const BUFFERS: i64 = 0x164;
/// Where the screen's picture starts in a buffer, its columns `amie_stylus.COLUMN` apart.
pub const PICTURE: i64 = 0x30000;
/// Where a drag carries a Poké Puff to on the Pokémon, its mouth: held there, Pikachu leaned in,
/// ate it in about 2.5 s and showed a heart; held at the screen's middle, over its eyes, it frowned
/// and turned away, and 17 above its mouth it ate nothing in 3.5 s (or_amie_dragging_puffs, spike
/// 257). Blaziken and Metagross ate there too, a little slower.
pub const MOUTH: (i64, i64) = (160, 157);
/// The pane the Pokémon stands in for in touch mode, which no pane is at.
pub const BECKON_TARGET: i64 = 1;
/// The Pokémon as touch mode offers it, "beckon Sceptile".
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
/// A female, in `PlayerFields.shown`'s third word.
pub const FEMALE: i64 = 1;
/// A female moving unlike her male in a message's key, `amie-motion-592-f-happyB`.
pub const FEMALE_FORM: &str = "f";
/// How long the picture is taken as read while a touch goes on, and as the arrows move the stylus,
/// which held repeat some thirty times a second; a whole picture is 20 ms to read (spike 287).
pub const FRESH: f64 = 0.1;
pub const BOTH_SCREENS: [i64; 21] = [
    130, 143, 248, 249, 250, 306, 321, 383, 384, 483, 484, 486, 487, 493, 623, 643, 644, 646, 716,
    717, 718,
];
/// The Pokémon too big for the bottom screen, by species, and Hoopa Unbound: each census Pokémon
/// reaching the top of the screen with no eyes drawn was pressed at the top edge, and these came down
/// (spike 287, 2026-11-07). The other 9 were only standing close: Bayleef, Spinda, Flygon, Trubbish,
/// female Jellicent, Accelgor and the biggest Pumpkaboo and Gourgeist. Nothing read says which are
/// which: `Player`'s size, the word 0x18 after `PlayerFields.shown`, has Snorlax at 1 and Charizard
/// at 2.
pub const UNBOUND: (i64, i64) = (720, 1);
/// The pane the Pokémon brought between the screens stands in for in touch mode.
pub const VIEW_TARGET: i64 = 3;
pub const VIEW_TOP: (i64, i64) = (160, 2);
pub const VIEW_BOTTOM: (i64, i64) = (160, 237);
/// Where a press brings a Pokémon too big for the bottom screen down onto it, where it sends it
/// back to the top screen, and how long: Wailord, Snorlax and Lugia showed their eyes about 2 s into
/// a press held at the top edge, stayed there, and went back to one at the bottom edge (spike 287's
/// `watch`).
pub const VIEW_SECONDS: f64 = 3.0;
pub const VIEW_DOWN: &str = "amie-view-down";
pub const VIEW_UP: &str = "amie-view-up";
pub const VIEW_CAME_DOWN: &str = "amie-view-came-down";
/// The two as touch mode offers them, and what is said once each is done.
pub const VIEW_WENT_UP: &str = "amie-view-went-up";
/// What the log calls the line said as a Pokémon has moved between the screens.
pub const VIEW_LOG: &str = "amie view";
/// Said as the stylus leaves the Pokémon, or comes onto grey 0, which the script takes for off
/// it.
pub const OFF: &str = "amie-part-off";
/// Each grey by the kind of part it is, a message each (spike 287), where the Pokémon has no name of
/// pa3ds's own for it in `amie.ftl` ("our own name a Pokémon", then "cheeks for
/// Pikachu, tail for the Charmander line"). 2 and 3 are the ears, the Pokémon's own right and left,
/// as 4 and 6 are its eyes; 10, where it high-fives, a hand or paw on most, a mouth on a few
/// (spike 287's census). 25 to 28, the danger zone and the sweet spot, are said as the head and the
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
/// Where the love command running (`PlayerFields.love`) keeps its number, an s16 (spike 287).
pub const LOVE_NUMBER: i64 = 0xC;
pub const NEXT_BITE: i64 = 15;
/// `LoveFoodIdle`: a puff held out, and where eating goes to when more of it is left; and the last
/// of `LoveFoodEat`'s.
pub const LAST_BITE: i64 = 19;
/// What the Pokémon is seen doing as a love command starts, a message each ("describe the other
/// moods too, even if nothing reaches them yet"). Which class each number builds
/// is the module's own table, read from the module loaded (spike 287), each started at will on the
/// research rig and pictured (`look --loves`): 1 to 7 `LoveAppear`; 8 and 9 `LoveMotionRepeat`, a
/// sleepy nod and lying asleep; 14 `LoveHate`, which the script sets only for a puff held out to a
/// Pokémon whose fullness, cell 64, is 255; 15 and 16 `LoveFoodIdle`, 16 after waiting, its head
/// tilted and a paw coming up, read as reaching for the puff; 17 to 19 `LoveFoodEat`; 20
/// `LoveFoodFall`, set as a puff is let go or swiped away, which draws nothing without one; 21
/// `LovePicky`, frowning with its head turned; 22 `LoveLook`, 23 `LoveLookAround`, 24
/// `LoveLookTouch`; 27 `LoveCamera`, leaning in with its eyes shut, set as the camera sees a face,
/// which Azahar has none of. Left unsaid: 0 `LoveMotion`, idle, 13 `LoveTouch`, a pet, the player's
/// own doing, and 18 and 19, the rest of eating. 10, 11, 12, 25 and 26 build nothing.
pub const MOODS: [(i64, &str); 19] = [
    (1, "amie-mood-coming"),
    (2, "amie-mood-coming"),
    (3, "amie-mood-coming"),
    (4, "amie-mood-coming"),
    (5, "amie-mood-coming"),
    (6, "amie-mood-coming"),
    (7, "amie-mood-back-turned"),
    (8, "amie-mood-dozing"),
    (9, "amie-mood-asleep"),
    (14, "amie-mood-too-full"),
    (NEXT_BITE, "amie-mood-puff-seen"),
    (16, "amie-mood-reaching"),
    (17, "amie-mood-eating"),
    (20, "amie-mood-puff-fell"),
    (21, "amie-mood-picky"),
    (22, "amie-mood-watching"),
    (23, "amie-mood-looking-around"),
    (HAND_RAISED, "amie-mood-hand-raised"),
    (27, "amie-mood-your-face"),
];
/// What is seen as a love command ends: turned round, awake, and at the end of eating a puff,
/// where `NEXT_BITE` does not follow.
pub const MOODS_OVER: [(i64, &str); 4] = [
    (7, "amie-mood-turned-round"),
    (8, "amie-mood-awake"),
    (9, "amie-mood-awake"),
    (LAST_BITE, "amie-mood-eaten"),
];
/// The moods saying the puff by its kind, as the game's text has no one word for a puff in every
/// language: its label is the plural in English and German. Left unsaid where nothing says which puff
/// it is, one not picked up by touch mode (2026-11-06: "use words from the ROM"). A
/// `frozenset` in Python, only ever tested for membership.
pub const OF_THE_PUFF: [&str; 4] = [
    "amie-mood-puff-seen",
    "amie-mood-reaching",
    "amie-mood-eaten",
    "amie-mood-puff-fell",
];
/// What the log calls a line of `Moods`.
pub const MOOD_LOG: &str = "amie mood";
/// The animations a sentence describes, by `PlayerFields.playing`: the models' `kw10_respond` to
/// `kw60_touch`, the three of eating, the three of nodding off and the two of sleep one action each.
/// A Pokémon's sentence is `amie-motion-<number>-<this>` in `amie-motions-*.ftl`, one a form moving
/// unlike its species adding its form (`AmiePage.form`), `amie-motion-25-1-happyB`, as `amie.ftl`'s
/// names do.
pub const MOTIONS: [(i64, &str); 25] = [
    (3, "respond"),
    (4, "turnA"),
    (5, "turnB"),
    (6, "drowse"),
    (7, "drowse"),
    (8, "drowse"),
    (9, "sleep"),
    (10, "sleep"),
    (11, "hate"),
    (12, "question"),
    (13, "happyA"),
    (14, "happyB"),
    (15, "happyC"),
    (16, "moveA"),
    (17, "moveB"),
    (18, "moveC"),
    (19, "moveD"),
    (20, "lonely"),
    (21, "playA"),
    (22, "playB"),
    (23, "mad"),
    (24, "eat"),
    (25, "eat"),
    (26, "eat"),
    (27, "touch"),
];
/// The animations a Pokémon plays by itself every few seconds, said only where the player asks
/// (`[pa3ds.amie] fidgets`): all but the reactions run under love command 0 (`moods --words`), so they
/// are told by name. A `frozenset` in Python, only ever tested for membership.
pub const FIDGETS: [&str; 7] = [
    "question", "moveA", "moveB", "moveC", "moveD", "turnA", "turnB",
];
/// The moods left unsaid where the Pokémon has a sentence for the animation under them, which says
/// more (2026-11-07: "just have the animations instead"). A hand held up has a cry of its own
/// as well, and a press held is what brought it.
pub const SHOWN_BY: [(&str, &str); 4] = [
    ("amie-mood-eating", "eat"),
    ("amie-mood-dozing", "drowse"),
    ("amie-mood-asleep", "sleep"),
    ("amie-mood-hand-raised", "touch"),
];
pub const TASK_SCRIPT: i64 = 0x04;
pub const SCRIPT_MAGIC: i64 = 0xF1E0;
pub const SCRIPT_MAGIC_AT: i64 = 0x04;
/// `Player`'s `kawaigari::PawnTask` (`PlayerFields.task`) +0x04 -> Amie's script as it runs, its
/// magic at +0x04 and at +0x10 how far in its data starts (spike 287).
pub const SCRIPT_DATA: i64 = 0x10;
pub const NUMBERS: [i64; 3] = [56, 64, 60];
pub const AFFECTION: usize = 0;
pub const FULLNESS: usize = 1;
/// Where the script's data keeps the Pokémon's three numbers, 0 to 255 each, in the order the
/// Switch screen shows them, as they stand: the game writes them to its record as Amie closes.
pub const ENJOYMENT: usize = 2;
/// The one number that fills a row's fifth icon.
pub const METER_FULL: i64 = 255;
/// The least number filling each of a row's five icons (`meter`).
pub const METER_STEPS: [i64; 5] = [1, 50, 100, 150, METER_FULL];
/// One of the numbers said as its row of icons, by the game's word for it: "Affection 2 of 5".
pub const METER: &str = "amie-meter";
pub const HIGH_FIVE: i64 = 10;
/// What a high five adds to Enjoyment, and how long after the hand was up it lands: the script
/// adds the same for other things (spike 287), and on or_amie_interactions 2 s after 24 ended.
pub const HIGH_FIVE_SECONDS: f64 = 4.0;
/// Where ORAS's page keeps its Pokémon's number; X's keeps it at 0x124 (`Game.amie`).
pub const NUMBER: i64 = 0x14C;
/// How far after the number the holder is, in ORAS and X alike (spike 228).
pub const HOLDER: i64 = 0x48;
pub const WALKERS: i64 = 0x04;
pub const WALKER: i64 = 0x14;
pub const WALKER_NUMBER: i64 = 0x08;
/// Who walks in the page's ring, in its holder from +0x04: a record each of `WALKER` bytes, its
/// picture first and its national number `WALKER_NUMBER` in. The player's Pokémon is first, and a
/// Pokémon of a friend's or a passerby's visiting from the PSS has a picture of its own after it, as
/// a visitor's Sceptile did (or_amie_affection_increase). Who it belongs to the game says on a tap.
pub const WALKERS_MOST: i64 = 6;
/// A Pokémon visiting the page, as touch mode offers it.
pub const VISITOR: &str = "amie-visitor";

/// How many of its row's five icons one of Amie's numbers fills: 1 to 49 one, 50 to 99 two,
/// 100 to 149 three, 150 to 254 four, and only 255 five (spike 287's `--meters`, the record
/// rewritten and the Switch screen read, alike for all three).
pub fn meter(number: i64) -> i64 {
    // `bisect.bisect_right`: past every step at most the number.
    METER_STEPS.partition_point(|&step| step <= number) as i64
}

/// Where a game's `Player` keeps what is read of it, each by its offset (`Game.amie_player`),
/// in one place: every read of it goes through `PlayerRead`, where reads adding a game's offset
/// each for itself had read X's wrong (6eae83a3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerFields {
    /// -> the love command running, none for a moment between two (`LOVE_NUMBER`).
    pub love: i64,
    /// The animation its model plays, its number in the model's list plus 2, and 2 while it
    /// stands, which it goes back to as each ends (`MOTIONS`).
    pub playing: i64,
    /// -> its `kawaigari::PawnTask`, which leads to Amie's script and its numbers (`NUMBERS`).
    pub task: i64,
    /// -> the picture of parts' holder (`PICTURE_HOLDER`).
    pub holds_picture: i64,
    /// The Pokémon it draws, a word each: its species, its form, its sex and the encryption
    /// constant its record starts with, which says whose it is. Found with the lead rewritten as
    /// Burmy's three cloaks on the research rig, and checked against Metagross's, Rock Star
    /// Pikachu's and Froakie's records.
    pub shown: i64,
}

/// Omega Ruby's and Alpha Sapphire's (spike 287); X's are `xy.AMIE_PLAYER`.
pub const ORAS_PLAYER: PlayerFields = PlayerFields {
    love: 0x1B8,
    playing: 0x146,
    task: 0x1BC,
    holds_picture: 0x22C,
    shown: 0x5FC,
};

/// A read's value, or None where the emulator could not answer it: Python's `except RpcError`.
fn answered<T>(read: Result<T>) -> Result<Option<T>> {
    match read {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.is(Kind::RpcError) => Ok(None),
        Err(error) => Err(error),
    }
}

/// `functools.cached_property`: the value kept once it is worked out; an error keeps nothing.
fn cached<T: Clone>(cell: &OnceCell<T>, work: impl FnOnce() -> Result<T>) -> Result<T> {
    if let Some(value) = cell.get() {
        return Ok(value.clone());
    }
    let value = work()?;
    Ok(cell.get_or_init(|| value).clone())
}

/// Amie's `Player` as read at one poll (`AmiePage.poll`): the first one running, each field
/// read the first time it is asked and kept until the next poll, so the stylus's keys, the moods
/// and touch mode's targets, which ask a few times a poll, read it once, and only what they
/// ask; anything that cannot be read is None.
pub struct PlayerRead {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    players: IndexMap<i64, i64>,
    fields: PlayerFields,
    found: OnceCell<Option<(i64, i64)>>,
    mood: OnceCell<Option<i64>>,
    motion: OnceCell<Option<&'static str>>,
    shown: OnceCell<Option<(i64, i64, i64, i64)>>,
    numbers: OnceCell<Option<(i64, i64, i64)>>,
}

impl PlayerRead {
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        players: IndexMap<i64, i64>,
        fields: PlayerFields,
    ) -> Self {
        PlayerRead {
            memory,
            names,
            players,
            fields,
            found: OnceCell::new(),
            mood: OnceCell::new(),
            motion: OnceCell::new(),
            shown: OnceCell::new(),
            numbers: OnceCell::new(),
        }
    }

    /// The Player running and its picture's holder, or None.
    pub fn found(&self) -> Result<Option<(i64, i64)>> {
        cached(&self.found, || {
            let fields = self.fields;
            let read = || -> Result<Option<(i64, i64)>> {
                for (&player, &vtable) in &self.players {
                    let head = self.memory.read(player, fields.holds_picture + 4)?;
                    let holder = u32(&head, fields.holds_picture)?;
                    // A Player freed, or a copy of its words in the module's code, leads nowhere.
                    if u32(&head, 0)? == vtable
                        && within(HEAP_SPAN, holder)
                        && self.names.of_object(holder)?.as_deref() == Some(PICTURE_HOLDER)
                    {
                        return Ok(Some((player, holder)));
                    }
                }
                Ok(None)
            };
            Ok(answered(read())?.flatten())
        })
    }

    /// `size` bytes of the Player running from `at`, or None.
    fn field(&self, at: i64, size: i64) -> Result<Option<Vec<u8>>> {
        let Some((player, _holder)) = self.found()? else {
            return Ok(None);
        };
        answered(self.memory.read(player + at, size))
    }

    /// The love command running, by its number; None between two.
    pub fn mood(&self) -> Result<Option<i64>> {
        cached(&self.mood, || {
            let head = self.field(self.fields.love, 4)?;
            let command = match head {
                Some(head) if !head.is_empty() => u32(&head, 0)?,
                _ => 0,
            };
            if !within(HEAP_SPAN, command) {
                return Ok(None);
            }
            answered(self.memory.read(command + LOVE_NUMBER, 2))?
                .map(|raw| s16(&raw, 0))
                .transpose()
        })
    }

    /// The animation its model plays, by `MOTIONS`' word; None while it stands.
    pub fn motion(&self) -> Result<Option<&'static str>> {
        cached(&self.motion, || {
            let Some(head) = self.field(self.fields.playing, 2)? else {
                return Ok(None);
            };
            let playing = u16(&head, 0)?;
            Ok(MOTIONS
                .iter()
                .find(|(number, _)| *number == playing)
                .map(|(_, word)| *word))
        })
    }

    /// The species, form, sex and record of the Pokémon it draws.
    pub fn shown(&self) -> Result<Option<(i64, i64, i64, i64)>> {
        cached(&self.shown, || {
            let Some(head) = self.field(self.fields.shown, 16)? else {
                return Ok(None);
            };
            Ok(Some((
                u32(&head, 0)?,
                u32(&head, 4)?,
                u32(&head, 8)?,
                u32(&head, 12)?,
            )))
        })
    }

    /// The Pokémon's affection, fullness and enjoyment as they stand.
    pub fn numbers(&self) -> Result<Option<(i64, i64, i64)>> {
        cached(&self.numbers, || {
            let memory = &self.memory;
            let head = self.field(self.fields.task, 4)?;
            let task = match head {
                Some(head) if !head.is_empty() => u32(&head, 0)?,
                _ => 0,
            };
            let read = || -> Result<Option<(i64, i64, i64)>> {
                let script = if within(HEAP_SPAN, task) {
                    u32(&memory.read(task + TASK_SCRIPT, 4)?, 0)?
                } else {
                    0
                };
                let head = if within(HEAP_SPAN, script) {
                    memory.read(script, SCRIPT_DATA + 4)?
                } else {
                    Vec::new()
                };
                if head.is_empty() || u16(&head, SCRIPT_MAGIC_AT)? != SCRIPT_MAGIC {
                    return Ok(None);
                }
                let data = script + u32(&head, SCRIPT_DATA)?;
                Ok(Some((
                    u32(&memory.read(data + NUMBERS[AFFECTION], 4)?, 0)?,
                    u32(&memory.read(data + NUMBERS[FULLNESS], 4)?, 0)?,
                    u32(&memory.read(data + NUMBERS[ENJOYMENT], 4)?, 0)?,
                )))
            };
            Ok(answered(read())?.flatten())
        })
    }

    /// The buffer of the picture of parts drawn last, read afresh as it is asked, as the two
    /// take turns.
    pub fn buffer(&self) -> Result<Option<i64>> {
        let Some((_player, holder)) = self.found()? else {
            return Ok(None);
        };
        let Some(buffers) = answered(self.memory.read(holder + BUFFERS, 12))? else {
            return Ok(None);
        };
        let which = u32(&buffers, 8)?;
        let buffer = if which == 0 || which == 1 {
            u32(&buffers, 4 * which)?
        } else {
            0
        };
        Ok(within(LINEAR_SPAN, buffer).then_some(buffer))
    }
}

/// Names the picture of the Pokémon the Pokémon-Amie page shows by its species, and offers
/// Amie's Pokémon to beckon and its stylus to rub with.
pub struct AmiePage {
    memory: Rc<dyn Memory>,
    /// A party member as the player knows it, by nickname or species (`Subscreens.short`).
    call: Name,
    /// Where this game's `Player` keeps what is read of it (`Game.amie_player`).
    fields: PlayerFields,
    /// `Player` as read at this poll, none until it is first asked of.
    now: OnceCell<PlayerRead>,
    /// What the picture of parts is read through, none where it is not read: the GPU draws it,
    /// and Azahar before 2126.2 can crash when that memory is read (`Rpc.reads_beside_gpu`).
    /// Reading it flushes it from the GPU's copy, right only from its own first byte on: from the
    /// pages round it, `PageCache`'s, it came back wrapped as far round as the buffer lies off a
    /// page, 4 to 8 rows on Omega Ruby and half a column on X, and up to half of it gone.
    drawn: Option<Rc<dyn Memory>>,
    /// The stylus over the still picture (`StylusKeys`).
    pub stylus: Stylus,
    /// When the Pokémon will have moved between the screens, and what is said then
    /// (`view_said`).
    moving: Option<(f64, String)>,
    /// Whether a Pokémon too big for the bottom screen has been brought down (`_view`).
    down: bool,
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
    /// The party's records, which say the Pokémon's nickname (`called`).
    params: PySet,
    /// The number and record `called` last looked up, and what it found.
    called: Option<(i64, Option<i64>, String)>,
    /// When the picture was last read, and the picture as it was: in a cell, as `picture` is
    /// asked through a shared page as well (`with_stylus`).
    whole: RefCell<(f64, Option<Rc<Picture>>)>,
    /// `motion_line` as a test replaces it, where Python's test sets the method on the page.
    pub(crate) motion_line_of: Option<fn(&str) -> Option<String>>,
    /// `BOTH_SCREENS` and `UNBOUND`, as a test replaces them, where Python's monkeypatches the
    /// module's.
    pub(crate) both_screens: &'static [i64],
    pub(crate) unbound: (i64, i64),
    /// The page itself, as it is shared (`shared`), for a target's `after` to reach it, where
    /// Python's closes over `self`; nothing where it is not shared.
    this: Weak<RefCell<AmiePage>>,
}

impl AmiePage {
    /// What the heap is swept for: the page, which says whose icon its Pokémon is, Amie's
    /// `Player`, which leads to the picture of its parts, and the party's records.
    pub const CLASSES: [&'static str; 3] = [CLASS, PLAYER, pokemon::PARAM_CLASS];

    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        species: Vec<String>,
        call: Name,
    ) -> Self {
        AmiePage::with_number(memory, names, species, call, NUMBER)
    }

    /// `AmiePage(memory, names, species, call, number)`; Python's `drawn` defaults to None.
    pub fn with_number(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        species: Vec<String>,
        call: Name,
        number: i64,
    ) -> Self {
        AmiePage::with_aim(memory, names, species, call, number, None, ORAS_PLAYER)
    }

    /// `AmiePage(...)` with every argument.
    pub fn with_aim(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        species: Vec<String>,
        call: Name,
        number: i64,
        drawn: Option<Rc<dyn Memory>>,
        fields: PlayerFields,
    ) -> Self {
        AmiePage {
            memory,
            call,
            fields,
            now: OnceCell::new(),
            drawn,
            stylus: Stylus::new(),
            moving: None,
            down: false,
            names,
            species,
            at: number,
            pages: IndexMap::new(),
            last: None,
            number: 0,
            players: IndexMap::new(),
            params: PySet::new(),
            called: None,
            whole: RefCell::new((f64::NEG_INFINITY, None)),
            motion_line_of: None,
            both_screens: &BOTH_SCREENS,
            unbound: UNBOUND,
            this: Weak::new(),
        }
    }

    /// The page shared, as the subscreens keep it, its targets' `after` able to reach it.
    pub fn shared(self) -> Rc<RefCell<AmiePage>> {
        let page = Rc::new(RefCell::new(self));
        page.borrow_mut().this = Rc::downgrade(&page);
        page
    }

    /// The pages and Players a sweep found, by class: all of them, or with `adding` the new
    /// ones; and the party's records.
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
        let mut params = if adding {
            self.params.copy()
        } else {
            PySet::new()
        };
        for (&at, name) in found {
            if let Some(objects) = kept.get_mut(name.as_str()) {
                objects.insert(at, u32(&self.memory.read(at, 4)?, 0)?);
            } else if name == pokemon::PARAM_CLASS {
                params.add(at);
            }
        }
        self.pages = kept.shift_remove(CLASS).unwrap_or_default();
        self.players = kept.shift_remove(PLAYER).unwrap_or_default();
        self.params = params;
        self.called = None;
        self.now = OnceCell::new();
        self.species()?;
        Ok(())
    }

    /// Start a poll, saying nothing: `Player` is read afresh, once, by what asks of it. The
    /// first of the subscreens' polled readers (`Subscreens`), before the stylus's keys and the
    /// moods ask.
    pub fn poll(&mut self, running: Option<&str>) -> Vec<(String, String, Option<Category>)> {
        let _ = running; // read only as asked, and only Amie's readers ask
        self.now = OnceCell::new();
        Vec::new()
    }

    /// `Player` as read at this poll.
    fn player(&self) -> &PlayerRead {
        self.now.get_or_init(|| {
            PlayerRead::new(
                self.memory.clone(),
                self.names.clone(),
                self.players.clone(),
                self.fields,
            )
        })
    }

    /// The Pokémon by the nickname its player gave it, or else its species:
    /// the party's record `Player` says it draws (`PlayerFields.shown`), of its
    /// species as well, as a save edited by hand can have two Pokémon sharing a constant. With no
    /// `Player` yet it is looked for by its species, and a species two of the party share is said
    /// as that.
    pub fn called(&mut self) -> Result<String> {
        let species = self.species()?.unwrap_or_default();
        let shown = self.player().shown()?;
        let key = shown.map(|shown| shown.3);
        if self
            .called
            .as_ref()
            .is_none_or(|called| (called.0, called.1) != (self.number, key))
        {
            let mut found = Vec::new();
            for param in self.params.iter() {
                let read = || -> Result<(i64, Option<pokemon::Pokemon>)> {
                    Ok((
                        u32(&self.memory.read(param + pokemon::RECORD_IN_PARAM, 4)?, 0)?,
                        pokemon::read(&*self.memory, &self.names, param, pokemon::SPECIES_MAX)?,
                    ))
                };
                match read() {
                    Ok(pair) => found.push(pair),
                    Err(error) if error.is(Kind::RpcError) => return Ok(species),
                    Err(error) => return Err(error),
                }
            }
            let ours: Vec<(i64, pokemon::Pokemon)> = found
                .into_iter()
                .filter_map(|(opens, one)| one.map(|one| (opens, one)))
                .filter(|(_, one)| one.species == self.number)
                .collect();
            let mine: Vec<&(i64, pokemon::Pokemon)> =
                ours.iter().filter(|pair| Some(pair.0) == key).collect();
            let chosen: Vec<&(i64, pokemon::Pokemon)> = if mine.is_empty() {
                ours.iter().collect()
            } else {
                mine
            };
            let mut names: Vec<Option<String>> = Vec::new();
            for (_opens, one) in chosen {
                let name = (self.call)(one);
                if !names.contains(&name) {
                    names.push(name);
                }
            }
            let name = if names.len() == 1 {
                names.remove(0)
            } else {
                None
            };
            let name = name.filter(|name| !name.is_empty()).unwrap_or(species);
            self.called = Some((self.number, key, name));
        }
        Ok(self.called.as_ref().expect("looked up above").2.clone())
    }

    /// The species a picture draws the Amie page's Pokémon as, in the game's words, or None.
    pub fn touched(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
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
            if let Some(visitor) = self.visitor(picture, holder)? {
                return Ok(Some(visitor));
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

    /// What a picture is called if it draws a Pokémon visiting the page, or None.
    fn visitor(&self, picture: i64, holder: i64) -> Result<Option<String>> {
        let walkers = self.memory.read(holder, WALKERS + WALKER * WALKERS_MOST)?;
        let own_picture = u32(&walkers, WALKERS)?;
        for at in (WALKERS..walkers.len() as i64).step_by(WALKER as usize) {
            let walker = u32(&walkers, at)?;
            let number = u16(&walkers, at + WALKER_NUMBER)?;
            if walker == picture
                && walker != own_picture
                && 0 < number
                && number < self.species.len() as i64
            {
                return Ok(Some(own(
                    VISITOR,
                    &[("pokemon", &self.species[number as usize])],
                )));
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
        if running != Some(AMIE) || self.species()?.is_none() {
            return Ok(Vec::new());
        }
        let (x, y) = self.stylus.at;
        let part = self.part(None)?;
        let name = self.called()?;
        let this = self.this.clone();
        let beckon = Target {
            path: vec![BECKON],
            seconds: BECKON_SECONDS,
            drop: Some(MOUTH),
            after: After(Some(Rc::new(move || {
                if let Some(page) = this.upgrade() {
                    page.borrow_mut()
                        .stylus
                        .look_again(py::time::monotonic(), BECKON_SECONDS);
                }
            }))),
            ..Target::new(
                BECKON_TARGET,
                &own(BECKON_WORD, &[("pokemon", &name)]),
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
        let mut found = vec![
            beckon,
            Target {
                rub: Some(self.with_stylus(|stylus, page| {
                    let page = &*page;
                    stylus.rub(|| page.picture(FRESH))
                })?),
                ..Target::new(STYLUS_TARGET, &stylus, x, y, 0.0, touch::HEIGHT as f64)
            },
        ];
        found.extend(self.view(&name)?);
        Ok(found)
    }

    /// The Pokémon too big for the bottom screen (`BOTH_SCREENS`), offered to bring down from
    /// the top screen by a press held at the touch screen's top edge, or send back up by one at
    /// its bottom edge. Nothing read says which screen it is on, and Regigigas and Zygarde have
    /// no eyes to tell by, so `_down` remembers the edge last pressed, Amie
    /// opening with it on the top screen.
    fn view(&mut self, name: &str) -> Result<Vec<Target>> {
        let shown = self.player().shown()?;
        if !self.both_screens.contains(&self.number)
            && shown.is_none_or(|shown| (shown.0, shown.1) != self.unbound)
        {
            return Ok(Vec::new());
        }
        let (point, word, moved) = if self.down {
            (VIEW_BOTTOM, VIEW_UP, VIEW_WENT_UP)
        } else {
            (VIEW_TOP, VIEW_DOWN, VIEW_CAME_DOWN)
        };
        let (this, called) = (self.this.clone(), name.to_string());
        let moving = move || {
            if let Some(page) = this.upgrade() {
                let mut page = page.borrow_mut();
                page.down = !page.down;
                page.moving = Some((
                    py::time::monotonic() + VIEW_SECONDS + SETTLE,
                    own(moved, &[("pokemon", &called)]),
                ));
                page.stylus.look_again(py::time::monotonic(), VIEW_SECONDS);
            }
        };
        Ok(vec![Target {
            path: vec![point],
            seconds: VIEW_SECONDS,
            after: After(Some(Rc::new(moving))),
            ..Target::new(
                VIEW_TARGET,
                &own(word, &[("pokemon", &name)]),
                point.0,
                point.1,
                0.0,
                touch::HEIGHT as f64,
            )
        }])
    }

    /// What is said now the Pokémon has moved between the screens, once, or None.
    pub fn view_said(&mut self) -> Option<String> {
        let (when, _) = self.moving.as_ref()?;
        if py::time::monotonic() < *when || self.stylus.anew {
            return None;
        }
        self.moving.take().map(|(_, said)| said)
    }

    /// A grey's part, or the part the stylus is on, in pa3ds's words: the Pokémon's own name
    /// for it, its form's first (`amie.ftl`, `form`), else the kind of part (`PARTS`); "off" the
    /// Pokémon where it is off it, or None where the picture cannot be read.
    pub fn part(&mut self, grey: Option<i64>) -> Result<Option<String>> {
        let grey = match grey {
            None => self.stylus.grey(),
            grey => grey,
        };
        let Some(grey) = grey else {
            return Ok(None);
        };
        if grey == 0 || grey == OFF_PICTURE {
            return Ok(Some(own(OFF, &[("pokemon", &self.called()?)])));
        }
        Ok(Some(
            self.word("spot", &grey, &[])?
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

    /// This Pokémon's own message of a kind, `amie-<kind>-<number>-<what>`, its form's
    /// first (`form`), or None where it has none.
    fn word(&self, kind: &str, what: &dyn Display, values: Values<'_>) -> Result<Option<String>> {
        let (words, form) = (current(), self.form()?);
        let named = form.and_then(|form| {
            words.get(
                &format!("amie-{kind}-{}-{form}-{what}", self.number),
                values,
            )
        });
        Ok(named
            .filter(|named| !named.is_empty())
            .or_else(|| words.get(&format!("amie-{kind}-{}-{what}", self.number), values)))
    }

    /// How the messages name the form Amie draws the Pokémon in (`PlayerFields.shown`): its
    /// number, `"1"` for Pikachu Rock Star, `FEMALE_FORM` for a first form's female, and None for
    /// a first form's male or where no Player runs, which take the species' own.
    pub fn form(&self) -> Result<Option<String>> {
        let Some((_species, form, sex, _record)) = self.player().shown()? else {
            return Ok(None);
        };
        Ok(if form != 0 {
            Some(form.to_string())
        } else if sex == FEMALE {
            Some(FEMALE_FORM.to_string())
        } else {
            None
        })
    }

    /// Take the still picture as the stylus wants it (`Stylus.settle`), from `Player`'s love
    /// command and the picture as drawn now.
    pub fn settle(&mut self) -> Result<()> {
        let now = py::time::monotonic();
        self.with_stylus(|stylus, page| {
            let page = &*page;
            stylus.settle(now, || page.mood(), || page.picture(LOOK))
        })
    }

    /// `self.stylus.<call>(...)` handed calls of the page's own, Python's `self.mood` or
    /// `self.picture`: the stylus is lent out of the page for the call, which the page's
    /// borrow would otherwise hold, and put back.
    pub fn with_stylus<T>(&mut self, call: impl FnOnce(&mut Stylus, &mut AmiePage) -> T) -> T {
        let mut stylus = std::mem::take(&mut self.stylus);
        let done = call(&mut stylus, self);
        self.stylus = stylus;
        done
    }

    /// Leave what is known of the Pokémon behind as Amie closes, or one of its games opens.
    pub fn forget(&mut self) {
        self.stylus.forget();
        (self.moving, self.down) = (None, false);
    }

    /// The love command running, by its number; None between two, or where no Player runs.
    pub fn mood(&self) -> Result<Option<i64>> {
        self.player().mood()
    }

    /// The animation its model plays, by `MOTIONS`' word; None while it stands, or where no
    /// Player runs.
    pub fn motion(&self) -> Result<Option<&'static str>> {
        self.player().motion()
    }

    /// The sentence for an animation of this Pokémon, its form's first, or None.
    pub fn motion_line(&mut self, motion: &str) -> Result<Option<String>> {
        if let Some(replaced) = self.motion_line_of {
            return Ok(replaced(motion));
        }
        let name = self.called()?;
        self.word("motion", &motion, &[("pokemon", &name)])
    }

    /// The Pokémon's affection, fullness and enjoyment as they stand, or None where no Player
    /// runs.
    pub fn numbers(&self) -> Result<Option<(i64, i64, i64)>> {
        self.player().numbers()
    }

    /// The picture of parts as drawn, read again once `fresh` has gone by; None where it
    /// cannot be. Python's `fresh` defaults to `LOOK`.
    pub fn picture(&self, fresh: f64) -> Result<Option<Rc<Picture>>> {
        let now = py::time::monotonic();
        if let (when, Some(whole)) = &*self.whole.borrow()
            && now - when < fresh
        {
            return Ok(Some(whole.clone()));
        }
        let (buffer, drawn) = (self.buffer()?, self.drawn.clone());
        let mut whole = None;
        if let (Some(buffer), Some(drawn)) = (buffer, drawn) {
            whole = match drawn.read(buffer + PICTURE, touch::WIDTH * COLUMN) {
                Ok(data) => Some(Rc::new(Picture::new(data))),
                Err(error) if error.is(Kind::RpcError) => None,
                Err(error) => return Err(error),
            };
        }
        *self.whole.borrow_mut() = (now, whole.clone());
        Ok(whole)
    }

    /// The buffer of the picture of parts drawn last, or None where no Player leads to one,
    /// or the emulator cannot read it safely (`_drawn`).
    fn buffer(&self) -> Result<Option<i64>> {
        if self.drawn.is_none() {
            return Ok(None);
        }
        self.player().buffer()
    }

    /// The image of texture 0 of a PokeIcon, or None where a link of the chain is not the
    /// class it should be.
    fn first_image(&self, icons: i64) -> Result<Option<i64>> {
        let (memory, names) = (&self.memory, &self.names);
        if icons == 0 || names.of_object(icons)?.as_deref() != Some(pokeicon::ICON_CLASS) {
            return Ok(None);
        }
        let replace = u32(&memory.read(icons + pokeicon::REPLACE, 4)?, 0)?;
        if replace == 0 || names.of_object(replace)?.as_deref() != Some(pokeicon::REPLACE_CLASS) {
            return Ok(None);
        }
        let head = memory.read(replace + pokeicon::TEXTURES, 8)?;
        let (listed, count) = (u32(&head, 0)?, u32(&head, 4)?);
        if listed == 0 || !(0 < count && count <= pokeicon::MAX_TEXTURES) {
            return Ok(None);
        }
        let texture = u32(&memory.read(listed, 4)?, 0)?;
        Ok(if texture != 0 {
            Some(u32(&memory.read(texture + pokeicon::IMAGE, 4)?, 0)?)
        } else {
            None
        })
    }
}

/// Says what Amie's Pokémon is seen doing as it starts and as it stops (`MOODS`,
/// `MOODS_OVER`), which a sighted player sees and the game's sounds tell only in part. A
/// description of Amie's own, where the player asks (`[pa3ds.amie] descriptions`, ADR-0038 as
/// amended 2026-11-06): a whole sentence, not Narration's, as it is said over play.
pub struct Moods {
    page: Rc<RefCell<AmiePage>>,
    /// Whether `FIDGETS` are said.
    fidgets: bool,
    /// The animation at the last poll.
    playing: Option<&'static str>,
    /// The puff last carried to the Pokémon. Python's keeps touch mode, which says the puff
    /// carried by its kind; here touch mode is handed to `poll`.
    puff: String,
    /// The love command at the last poll.
    mood: Option<i64>,
    /// The Pokémon's numbers at the last poll.
    numbers: Option<(i64, i64, i64)>,
    /// When its hand was last seen up.
    raised: f64,
}

impl Moods {
    /// `Moods(page, touch, fidgets)`.
    pub fn new(page: Rc<RefCell<AmiePage>>, fidgets: bool) -> Self {
        Moods {
            page,
            fidgets,
            playing: None,
            puff: String::new(),
            mood: None,
            numbers: None,
            raised: f64::NEG_INFINITY,
        }
    }

    /// What the Pokémon is seen doing, where its love command or its animation changed since
    /// the last poll, and what a pet or a high five showed.
    pub fn poll(
        &mut self,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        let before = self.mood;
        self.mood = if running == Some(AMIE) {
            self.page.borrow().mood()?
        } else {
            None
        };
        if running != Some(AMIE) {
            (self.numbers, self.playing) = (None, None);
            return Ok(Vec::new());
        }
        if self.mood == Some(HAND_RAISED) {
            self.raised = py::time::monotonic();
        }
        if let Some(picked) = touch.picked().filter(|picked| !picked.is_empty()) {
            self.puff = picked;
        }
        let mut moods: Vec<String> = Vec::new();
        if self.mood != before && !(before == Some(LAST_BITE) && self.mood == Some(NEXT_BITE)) {
            let word = |table: &[(i64, &'static str)], number: i64| {
                table
                    .iter()
                    .find(|(key, _)| *key == number)
                    .map(|(_, word)| *word)
            };
            for word in [
                word(&MOODS_OVER, before.unwrap_or(0)),
                word(&MOODS, self.mood.unwrap_or(0)),
            ]
            .into_iter()
            .flatten()
            {
                if self.puff.is_empty() && OF_THE_PUFF.contains(&word) {
                    continue;
                }
                if let Some((_, shown)) = SHOWN_BY.iter().find(|(mood, _)| *mood == word)
                    && self
                        .page
                        .borrow_mut()
                        .motion_line(shown)?
                        .is_some_and(|line| !line.is_empty())
                {
                    continue;
                }
                let called = self.page.borrow_mut().called()?;
                moods.push(own(word, &[("pokemon", &called), ("puff", &self.puff)]));
            }
        }
        moods.extend(self.moving()?);
        let mut said: Vec<(String, String, Option<Category>)> = moods
            .into_iter()
            .map(|line| (MOOD_LOG.to_string(), line, Some(Category::Screen)))
            .collect();
        said.extend(self.shown()?);
        Ok(said)
    }

    /// The sentence for the animation just begun, if any: seen after a puff, a pleased pet, a
    /// dislike and Affection rising, and every few seconds by itself (`FIDGETS`).
    fn moving(&mut self) -> Result<Vec<String>> {
        let before = self.playing;
        self.playing = self.page.borrow().motion()?;
        let Some(playing) = self.playing else {
            return Ok(Vec::new());
        };
        if Some(playing) == before || (FIDGETS.contains(&playing) && !self.fidgets) {
            return Ok(Vec::new());
        }
        let line = self.page.borrow_mut().motion_line(playing)?;
        Ok(line.filter(|line| !line.is_empty()).into_iter().collect())
    }

    /// What the Pokémon's numbers moving shows: a pet takes from Enjoyment and gives the hearts
    /// seen to Affection, and one it dislikes takes and gives none, Pikachu's ears 20 on
    /// or_amie_interactions, where a puff eaten gave Affection 2, took nothing, and brought no
    /// hearts on either screen, its dance said by `_moving`. At full Affection a pet looks like a
    /// dislike, so says nothing. Affection filling another of its row's icons brings an animation
    /// of its own and a fountain of hearts, and the last of them a longer one, the Pokémon
    /// tumbling round the screen for 8 s (or_amie_affection_increase, or_amie_affection_max).
    fn shown(&mut self) -> Result<Vec<(String, String, Option<Category>)>> {
        let before = self.numbers;
        self.numbers = self.page.borrow().numbers()?;
        let (Some(before), Some(numbers)) = (before, self.numbers) else {
            return Ok(Vec::new());
        };
        let gained = numbers.0 - before.0;
        let enjoyed = numbers.2 - before.2;
        let name = self.page.borrow_mut().called()?;
        let mut lines: Vec<String> = Vec::new();
        if enjoyed < 0 && gained > 0 {
            lines.push(own("amie-hearts", &[("count", &gained)]));
        } else if enjoyed < 0 && numbers.0 < METER_FULL {
            lines.push(own("amie-mood-disliked", &[("pokemon", &name)]));
        } else if enjoyed == HIGH_FIVE && py::time::monotonic() - self.raised < HIGH_FIVE_SECONDS {
            lines.push(own("amie-high-five", &[]));
        }
        if meter(numbers.0) > meter(before.0) {
            lines.push(own(
                if numbers.0 >= METER_FULL {
                    "amie-mood-best-friends"
                } else {
                    "amie-mood-closer"
                },
                &[("pokemon", &name)],
            ));
        }
        Ok(lines
            .into_iter()
            .map(|line| (MOOD_LOG.to_string(), line, Some(Category::Screen)))
            .collect())
    }
}

/// Heard for an arrow that would take the stylus off the screen: the game's error sound, or the
/// words where that is off.
pub const EDGE: &str = "amie-stylus-edge";

/// Amie's own keys, the arrows: each moves the stylus `STEP` pixels over
/// its still picture (`Stylus`), saying the kind of part it comes onto (`AmiePage.part`), and
/// nothing more while it stays on that kind, touch mode's tone sounding where it is. Touch mode
/// chooses the stylus as it moves, so touch mode's tap rubs that part a second and its drag rubs
/// it until let go, wherever it has gone, carried on with each move (`Stylus.rub`,
/// `TouchMode.choose`).
///
/// `wrong` is what is heard for an arrow at the screen's edge: a sound, and "", or the words.
///
/// Python's holds touch mode, which is built after the subscreens, as it offers their targets;
/// here touch mode, which the navigator owns, is handed to `press` and `poll`.
pub struct StylusKeys {
    page: Rc<RefCell<AmiePage>>,
    /// What the game calls Amie's three numbers, none where its text is not read.
    labels: Vec<String>,
    wrong: Rc<dyn Fn(&str) -> String>,
    /// The kind of part said last, which a move staying on it does not say again.
    said: Option<String>,
}

impl StylusKeys {
    pub fn new(
        page: Rc<RefCell<AmiePage>>,
        wrong: Rc<dyn Fn(&str) -> String>,
        labels: Vec<String>,
    ) -> Self {
        StylusKeys {
            page,
            labels,
            wrong,
            said: None,
        }
    }

    /// The keys' table in the keys file while Amie runs, `[pa3ds.amie.stylus]`, else None.
    pub fn context(&self, running: Option<&str>) -> Option<&'static str> {
        (running == Some(AMIE)).then_some(STYLUS)
    }

    /// Move the stylus a step, or jump it to the next part, with the line to say; or say the
    /// Pokémon's three numbers, as the Switch screen would show them now.
    pub fn press(
        &mut self,
        action: &str,
        _running: Option<&str>, // only ever asked while Amie runs (`context`)
        touch: &mut dyn Touching,
    ) -> Result<String> {
        let page = self.page.clone();
        if action == STYLUS_NUMBERS {
            let numbers = page.borrow().numbers()?;
            let Some((affection, fullness, enjoyment)) =
                numbers.filter(|_| !self.labels.is_empty())
            else {
                return Ok((self.wrong)(&own(EDGE, &[])));
            };
            // Python's `zip(..., strict=True)`, the game's labels against the three numbers.
            if self.labels.len() < 3 {
                return Err(Error::value("zip() argument 2 is longer than argument 1"));
            }
            return Ok(self
                .labels
                .iter()
                .zip([affection, fullness, enjoyment])
                .map(|(label, number)| own(METER, &[("label", label), ("count", &meter(number))]))
                .collect::<Vec<_>>()
                .join(", "));
        }
        page.borrow_mut().settle()?;
        let mut grey = None;
        if let Some(&(_, (across, down))) = STYLUS_JUMPS.iter().find(|(name, _)| *name == action) {
            let landed = page
                .borrow_mut()
                .with_stylus(|stylus, page| -> Result<bool> {
                    let shared = &*page;
                    if stylus.to_hand(|| shared.mood(), || shared.picture(0.0))? {
                        grey = Some(HAND); // what it lands on, gone from the still picture as likely as not
                        return Ok(true);
                    }
                    stylus.jump(across, down, &mut |grey| page.part(grey))
                })?;
            if !landed {
                return Ok((self.wrong)(&own(EDGE, &[])));
            }
            self.said = None; // a jump always says where it lands
        } else {
            let (_, (across, down)) = *STYLUS_MOVES
                .iter()
                .find(|(name, _)| *name == action)
                .ok_or_else(|| Error::key(py::repr(action)))?;
            if !page.borrow_mut().stylus.step(across, down) {
                return Ok((self.wrong)(&own(EDGE, &[])));
            }
        }
        touch.choose(STYLUS_TARGET)?;
        let part = page.borrow_mut().part(grey)?;
        if part == self.said {
            return Ok(String::new());
        }
        self.said = part.clone();
        Ok(part.unwrap_or_default())
    }

    /// Take the still picture, and carry a rub on to its part as that moves, choosing nothing
    /// in touch mode; and says a Pokémon too big for the bottom screen has moved between the
    /// screens.
    pub fn poll(
        &self,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        if running != Some(AMIE) {
            self.page.borrow_mut().forget();
            return Ok(Vec::new());
        }
        self.page.borrow_mut().settle()?;
        touch.carry(STYLUS_TARGET)?;
        let said = self.page.borrow_mut().view_said();
        Ok(match said {
            None => Vec::new(),
            Some(said) => vec![(VIEW_LOG.to_string(), said, Some(Category::Screen))],
        })
    }
}
