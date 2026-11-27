//! Pokémon Sun and Moon: another engine, gfl2, read by ORAS's dialogue reader where the two agree
//! and by readers of its own where they do not (spike 111).
//!
//! What agrees: the ROM's text files, a message window's fields, the string buffer's and the
//! control sequences that break a message into pages. What does not: the classes' names, a
//! TextBox's fields, where the heap is, and a speaker the game names itself. Only dialogue is read
//! so far; Moon is Sun's other version and is taken to be laid out as Sun is, since no Moon has
//! been read.

use std::collections::HashSet;
use std::sync::OnceLock;

pub const SUN: i64 = 0x0004000000164800;
pub const MOON: i64 = 0x0004000000175E00;
pub const ULTRA_SUN: i64 = 0x00040000001B5000;
pub const ULTRA_MOON: i64 = 0x00040000001B5100;

/// Where a pair of versions keeps what Sun's readers read, where it is not where Sun keeps it.
/// The readers name text files by Sun's numbers and ask this for their own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionLayout {
    /// Sun's file of system text -> this pair's, for those that moved. A `Mapping` only looked
    /// up.
    pub system_files: &'static [(i64, i64)],
    /// The story text file of the signs' words (`gametext`).
    pub sign_text: i64,
    /// The trainer table's archive, a file a trainer (`gametext`).
    pub trainers: &'static str,
    /// The trainer classes' archive, a file a class (`gametext`).
    pub classes: &'static str,
    /// A move model's flag, 1 while it is drawn (`field`).
    pub model_shown: i64,
    /// Where a move model stands now, x, y and z (`field`).
    pub model_place: i64,
    /// Where the area loaded keeps its number in the event archive (`field`).
    pub area_number: i64,
    /// The field's models' archive, model n file n (`gametext`).
    pub field_models: &'static str,
    /// The Town Map's archive, whose marks name doors (`gametext`).
    pub town_map: &'static str,
    /// The language choice's language under the cursor, the choice after it (`langselect`).
    pub chosen_language: i64,
    /// Where a Poké Mart list's `ShopDraw` keeps the item under the cursor (`shop`), None where it
    /// is not measured.
    pub shop_item: Option<i64>,
    /// Festival Plaza's zones, which the zone table gives the place of a Pokémon Center: named by
    /// the X menu's own line instead (`zones.PLAZA`). A `frozenset` only looked in.
    pub plaza: &'static [i64],
    /// Where the save keeps its trainer record, `Savedata::MyStatus` (`savedata`): 0xea0 in Sun
    /// and 0xee8 in Ultra Sun, as 68 and 73 of their functions reach it.
    pub my_status: i64,
}

impl VersionLayout {
    /// This pair's file of system text that Sun numbers `number`.
    pub fn system(&self, number: i64) -> i64 {
        self.system_files
            .iter()
            .find(|&&(sun, _)| sun == number)
            .map_or(number, |&(_, own)| own)
    }
}

/// Sun or Moon, as the catalog tells it from the title running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SunMoon {
    /// What the log calls it.
    pub name: String,
    /// The title ids it runs as, one a version.
    pub titles: HashSet<i64>,
    pub layout: VersionLayout,
}

pub const SUN_LAYOUT: VersionLayout = VersionLayout {
    system_files: &[],
    sign_text: 21,
    trainers: "a/1/0/5",
    classes: "a/1/0/4",
    model_shown: 0x308,
    model_place: 0x3A4,
    area_number: 0x00,
    field_models: "a/1/9/6",
    town_map: "a/2/4/1",
    chosen_language: 0x12C,
    shop_item: Some(0xC4),
    plaza: &[0x119, 0x11A],
    my_status: 0xEA0,
};
/// Ultra Sun's, read against Sun's in its ROM and its bedroom (spike P, 2026-11-23): every class
/// the readers find has Sun's name, and every field of each is where Sun's is but a move model's,
/// eight bytes on from +0x2f4 and twelve by its place, and the area loaded's, its number at +0x04
/// with a pointer before it, and the language choice's, its language and choice four bytes on.
/// The archives are at Sun's paths but the trainers', one on, the field's models' at `a/2/0/0`
/// (whose 316 models with a name keep Sun's numbers) and the Town Map's at `a/2/4/5`, where Sun's
/// paths hold others. System text files from 36 on (the level's 21 is Sun's) are four or five on,
/// the item names' 36 at 40, the language choice's 51 at 55 and the place names' 67 at 72; a
/// Mart's list keeps its item at `ShopDraw` +0xc4, as Sun's does.
pub const ULTRA_LAYOUT: VersionLayout = VersionLayout {
    system_files: &[
        (36, 40),
        (51, 55),
        (55, 60),
        (67, 72),
        (100, 105),
        (105, 110),
        (106, 111),
    ],
    sign_text: 23,
    trainers: "a/1/0/6",
    classes: "a/1/0/5",
    model_shown: 0x310,
    model_place: 0x3B0,
    area_number: 0x04,
    field_models: "a/2/0/0",
    town_map: "a/2/4/5",
    chosen_language: 0x130,
    shop_item: Some(0xC4),
    plaza: &[0x13E, 0x13F],
    my_status: 0xEE8,
};

/// `GAME`: a function, since a `String` and a `HashSet` cannot be built in a constant.
pub fn game() -> &'static SunMoon {
    static GAME: OnceLock<SunMoon> = OnceLock::new();
    GAME.get_or_init(|| SunMoon {
        name: "Sun and Moon".into(),
        titles: HashSet::from([SUN, MOON]),
        layout: SUN_LAYOUT,
    })
}

/// Read as Sun, the same engine a year on, through its own `VersionLayout`. Ultra Moon is taken
/// to be laid out as Ultra Sun, as Moon is as Sun, since none has been read.
///
/// `ULTRA`: a function for the same reason as [`game`].
pub fn ultra() -> &'static SunMoon {
    static ULTRA: OnceLock<SunMoon> = OnceLock::new();
    ULTRA.get_or_init(|| SunMoon {
        name: "Ultra Sun and Ultra Moon".into(),
        titles: HashSet::from([ULTRA_SUN, ULTRA_MOON]),
        layout: ULTRA_LAYOUT,
    })
}

/// The layout of the versions a ROM or a process of `title` is: Sun's for any not Ultra.
pub fn layout_of(title: i64) -> VersionLayout {
    if ultra().titles.contains(&title) {
        ULTRA_LAYOUT
    } else {
        SUN_LAYOUT
    }
}

pub mod apps;
pub mod battle;
pub mod camera;
pub mod field;
pub mod gametext;
pub mod heaps;
pub mod langselect;
pub mod lyt;
pub mod msgwin;
pub mod profile;
pub mod script;
pub mod session;
pub mod talk;
pub mod text;
pub mod walls;
pub mod zones;

pub mod shop;

pub mod r#box;

pub mod savedata;
