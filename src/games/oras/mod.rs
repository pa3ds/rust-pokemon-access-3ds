//! Pokémon Omega Ruby and Alpha Sapphire: read by the gen 6 profile and readers (`games.gen6`),
//! with what ORAS keeps where X does not here — where its ROM keeps each kind of file, and who its
//! story's people are — and what ORAS alone has: on its field (`field`) the DexNav's hidden
//! Pokémon, Secret Bases, berry spots, soaring and more, and among its screens (`screens`) its
//! contests.

use std::collections::HashSet;
use std::rc::Rc;
use std::sync::OnceLock;

use indexmap::IndexMap;

use crate::core::memory::Memory;
use crate::core::pathfind::SOUTH;
use crate::games::common::field::ground::{CUT, ROCK_SMASH, STRENGTH};
use crate::games::common::pictures;
use crate::games::gen6::field::features::{
    DoorReader, Feature, HiddenReader, IceReader, SecretBaseReader,
};
use crate::games::gen6::field::mmodel;
use crate::games::gen6::field::reader::{FLAG, PC_TILE};
use crate::games::gen6::game::{Data, FurnitureKind, Game};
use crate::games::gen6::rom::archives::{BOY, GIRL, GameText};
use crate::games::gen6::rom::layout::{RomLayout, TownFacilities};
use crate::games::oras::field::ash::AshPiles;
use crate::games::oras::field::berries::BerrySpots;
use crate::games::oras::field::doors::RotatingDoors;
use crate::games::oras::field::hidden::DexNav;
use crate::games::oras::field::ice::IceFloors;
use crate::games::oras::field::secret_base::{SecretBases, zones as secret_bases_zones};
use crate::games::oras::field::sky;
use crate::games::oras::field::sky::Soaring;
use crate::games::oras::screens::conditions::Conditions;
use crate::games::oras::screens::contest::Standings;

/// Words the game does not say itself, a folder a language: `pa3ds.core.names` reads them.
///
/// The files are `include_str!`ed, so the binary carries them: `Game.data` is this table in place
/// of Python's `Path(__file__).parent / "data"`.
pub const DATA: Data = &[
    ("ja", include_str!("data/ja/sprites.toml")),
    ("en", include_str!("data/en/sprites.toml")),
    ("fr", include_str!("data/fr/sprites.toml")),
    ("it", include_str!("data/it/sprites.toml")),
    ("de", include_str!("data/de/sprites.toml")),
    ("es", include_str!("data/es/sprites.toml")),
    ("ko", include_str!("data/ko/sprites.toml")),
];

/// Each language's `scenes.toml` from the same folder, where there is one
/// (`games.common.scenes.FILE`).
pub const SCENES: Data = &[("en", include_str!("data/en/scenes.toml"))];

/// Each language's `pictures.toml` from the same folder, where there is one
/// (`games.common.pictures.PICTURES_FILE`).
pub const PICTURES: Data = &[("en", include_str!("data/en/pictures.toml"))];

pub const OMEGA_RUBY: i64 = 0x000400000011C400;
pub const ALPHA_SAPPHIRE: i64 = 0x000400000011C500;

/// The text archive a language, as they sit in the ROM (spike 74; `a/0/7/2` is the Japanese in
/// kanji).
pub fn text_archives() -> &'static IndexMap<&'static str, &'static str> {
    static TEXT_ARCHIVES: OnceLock<IndexMap<&'static str, &'static str>> = OnceLock::new();
    TEXT_ARCHIVES.get_or_init(|| {
        IndexMap::from([
            ("ja", "a/0/7/1"),
            ("en", "a/0/7/3"),
            ("fr", "a/0/7/4"),
            ("it", "a/0/7/5"),
            ("de", "a/0/7/6"),
            ("es", "a/0/7/7"),
            ("ko", "a/0/7/8"),
        ])
    })
}

/// The story text archive a language, the zones' scripts' messages (`a/0/8/0` the Japanese in
/// kanji), parallel across languages (spike 91); read for the plates beside doors (spike 109).
pub fn story_archives() -> &'static IndexMap<&'static str, &'static str> {
    static STORY_ARCHIVES: OnceLock<IndexMap<&'static str, &'static str>> = OnceLock::new();
    STORY_ARCHIVES.get_or_init(|| {
        IndexMap::from([
            ("ja", "a/0/7/9"),
            ("en", "a/0/8/1"),
            ("fr", "a/0/8/2"),
            ("it", "a/0/8/3"),
            ("de", "a/0/8/4"),
            ("es", "a/0/8/5"),
            ("ko", "a/0/8/6"),
        ])
    })
}

pub const PLACE_NAMES: i64 = 90;
pub const TRAINER_NAMES: i64 = 22;
pub const CLASS_NAMES: i64 = 21;
/// One small file a trainer, by the same number as the trainer names; the class at +0x02.
pub const TRAINERS: &str = "a/0/3/6";
pub const TRAINER_CLASS: i64 = 0x02;
pub const TRAINER_BATTLE: i64 = 0x06;

/// Sprite -> trainer, for the people the story names who stand as a sprite of their own (spike
/// 90). Which trainer is which person is read off the ROM's own sprite ids and where each sprite
/// stands; the words are the ROM's, in whichever language is asked for. A sprite shared by a kind
/// of trainer is not here: that is derived from the maps instead (`archives.trainer_sprites`).
pub fn characters() -> &'static IndexMap<i64, i64> {
    static CHARACTERS: OnceLock<IndexMap<i64, i64>> = OnceLock::new();
    CHARACTERS.get_or_init(|| {
        IndexMap::from([
            // The rival, drawn into work object 0x7a (spike 89); the bikes by the ROM's own ids.
            (0x13A, GIRL),
            (0x13C, GIRL), // SUPPORT_F_RS, SUPPORT_F_BIKE_RS: May
            (0x13B, BOY),
            (0x13D, BOY), // SUPPORT_M_RS, SUPPORT_M_BIKE_RS: Brendan
            (0xB4, 518),
            (0xB5, 518), // RIVAL_RS (spike 73), RIVAL_EX_RS (its one trainer): Wally
            (0xBB, 561), // LEADER1_RS, in Rustboro: Roxanne
            (0xBC, 563),
            (0x15D, 563), // LEADER2_RS and 2A, in Dewford: Brawly
            (0xBD, 567),  // LEADER3_RS, in Mauville: Wattson
            (0xBE, 569),  // LEADER4_RS, in Lavaridge: Flannery
            (0xBF, 570),
            (0x13F, 570), // LEADER5_RS in Petalburg, LEADER5A_RS (spike 73): Norman
            (0xC0, 571),
            (0x15F, 571), // LEADER6_RS and 6A, in Fortree: Winona
            (0xC3, 572),  // LEADER8_RS, in Sootopolis: Wallace
            (0xB7, 553),
            (0x160, 553), // BIGFOUR1_RS and 1A: Sidney
            (0xB8, 554),
            (0x15E, 554), // BIGFOUR2_RS (Mt. Pyre) and 2A: Phoebe
            (0xB9, 555),  // BIGFOUR3_RS: Glacia
            (0xBA, 556),  // BIGFOUR4_RS (the S.S. Tidal): Drake
            (0xB6, 557),  // CHAMPION_RS (Granite Cave, Meteor Falls, Mossdeep): Steven
            (0xC4, 235),
            (0xC5, 691),
            (0xC6, 694), // MAGMABOSS_RS, MAGMAKANBU_M_RS, _F_RS: Maxie, Tabitha,
            // Courtney, in Team Magma's hideout
            (0xC9, 178),
            (0xCA, 683),
            (0xCB, 688), // AQUABOSS_RS, AQUAKANBU_M_RS, _F_RS: Archie, Matt, Shelly
            (0xD3, 856), // HIMITU_RS, where the Secret Bases are: Aarune
            (0xD4, 713), // ZAZAKO_RS, outside Sky Pillar and wherever her Whismur
                         // stands beside her (spike 193): Lorekeeper Zinnia
        ])
    })
}

/// The map chunks, a file a chunk by the number the field map lists it under: Route 109's four
/// round the beach are files 43 to 46, each the cell table the game loads (spike 136).
pub const MAP_CHUNKS: &str = "a/0/3/9";
/// The matrices, a file a matrix by the number a zone-table row's +0x04 gives: Route 126's is 28, 8
/// chunks by 8, and the water under it 31, 3 by 3 (spike 165).
pub const MATRICES: &str = "a/0/4/0";
/// The field's DLL, whose table lists all 90 berry spots (spike 141): Route 102's two are the
/// first, at (155, 129) and (156, 129), where the player heard "This is where you planted the Pecha
/// Berry!" and harvested an Oran Berry tree (omega_ruby_berry.ctm).
pub const BERRY_SPOTS: &str = "DllField.cro";

/// The moves' names, a line a move (file 15 holds the same lines).
pub const MOVE_NAMES: i64 = 14;

/// The PC's own text (`archives.PC_LINES`).
pub const PC_TEXT: i64 = 92;

/// The AreaNav's text: a description a place, then the facilities' names from line 348.
pub const FACILITY_NAMES: i64 = 33;
/// Zone N's data file is file N, LZ11-compressed; the zone table follows them.
pub const ZONES: &str = "a/0/1/3";
pub const ZONE_TABLE: i64 = 536;
/// A row a place, 60 bytes: its outdoor zone at +0x00, its facilities' lines from +0x24.
pub const AREANAV: (&str, i64) = ("a/0/6/6", 0);
pub const NAV_ROW: i64 = 60;
pub const NAV_ZONE: i64 = 0x00;
pub const NAV_FIRST: i64 = 0x24;
/// ORAS's list of what each town has: the AreaNav's.
pub fn towns() -> &'static TownFacilities {
    static TOWNS: OnceLock<TownFacilities> = OnceLock::new();
    TOWNS.get_or_init(|| TownFacilities {
        archive: AREANAV.0.to_string(),
        file: AREANAV.1,
        text: FACILITY_NAMES,
        row: NAV_ROW,
        zone: NAV_ZONE,
        first: NAV_FIRST,
    })
}
/// The PokéNav's app buttons, drawn as pictures alone, by the icon each draws, and the line of
/// text file 123 the game names its app with: AreaNav 139, PlayNav 140, BuzzNav 141, DexNav 142
/// (spike 171, and the BuzzNav's in movie24); the AreaNav's tabs, by the images their layout
/// file gives them: Places 143, Pokémon 144, Trainers 145, Bases 146, Berries 147 (movie17); and the
/// Pokémon-Amie page's menu, opened by tapping its Pokémon, images of their words: PLAY 77,
/// DECORATE 78, SWITCH 79, in the middle, on the left and on the right (spike 198,
/// omega_ruby_bottom-screen2); and Super Training's page, whose lower-left button
/// `spar_button.bclim` opens the regimen menu, by line 84, "Super Training"
/// (omega_ruby_bottom-screen5).
pub fn touch_words() -> &'static (i64, IndexMap<&'static str, i64>) {
    static TOUCH_WORDS: OnceLock<(i64, IndexMap<&'static str, i64>)> = OnceLock::new();
    TOUCH_WORDS.get_or_init(|| {
        (
            123,
            IndexMap::from([
                ("pmn_screen_map.bclim", 139),
                ("pmn_sub_screen_pss.bclim", 140),
                ("pmn_screen_tv.bclim", 141),
                ("pmn_screen_zukan.bclim", 142),
                ("map_navi01.bclim", 143),
                ("map_navi02.bclim", 144),
                ("map_navi03.bclim", 145),
                ("map_navi04.bclim", 146),
                ("map_navi05.bclim", 147),
                ("kwg_button01.bclim", 77),
                ("kwg_button02.bclim", 78),
                ("kwg_button03.bclim", 79),
                ("spar_button.bclim", 84),
            ]),
        )
    })
}
/// The DexNav's red magnifier over the middle of its map, 120 by 120, while a hidden Pokémon is
/// up: drawn as a picture alone, and tapped to show that Pokémon's box, so named by its title there,
/// line 12 of text file 84, "Hidden Pokémon". It covers the icons under it, which a tap cannot reach
/// meanwhile (movie27, 279 s: "search get" by its file's name). Its archive has two: `get`, with a
/// Poké Ball on it, seen over a caught Surskit, and `check`, not seen, for one not caught (spike
/// 195).
pub fn dexnav_words() -> &'static (i64, IndexMap<&'static str, i64>) {
    static DEXNAV_WORDS: OnceLock<(i64, IndexMap<&'static str, i64>)> = OnceLock::new();
    DEXNAV_WORDS.get_or_init(|| {
        (
            84,
            IndexMap::from([
                ("search_get_button.bclim", 12),
                ("search_check_button.bclim", 12),
            ]),
        )
    })
}
/// Super Training's bags, drawn as pictures alone in the list its page opens, by the image each
/// draws and the line of text file 24 the game names the bag with: line N is the bag of
/// `sand_bagNN.bclim`, HP Bag S at 1 to Soothing Bag at 26 (omega_ruby_bottom-screen5). The bag
/// hanging beside the Pokémon, `sand_bag00.bclim`, has no line (line 0 is empty) and is left out.
pub fn training_bags() -> &'static (i64, IndexMap<&'static str, i64>) {
    static TRAINING_BAGS: OnceLock<(i64, IndexMap<&'static str, i64>)> = OnceLock::new();
    TRAINING_BAGS.get_or_init(|| {
        let names = (1..27).map(|n| {
            let image: &'static str = Box::leak(format!("sand_bag{n:02}.bclim").into_boxed_str());
            (image, n)
        });
        (24, names.collect())
    })
}
/// Super Training's regimens, drawn as pictures alone in the menu a round is chosen from, by the
/// Balloon Bot each draws and the line of text file 44 the game titles the regimen with: line N-1
/// is the regimen of `stage_pokeNN.bclim`, 'Practice against Scatterbug!' at 0 to 'Watch It!
/// Multiple Mega Evolutions!' at 37. The tutorials drew `stage_poke01` and `stage_poke02`,
/// Scatterbug and Spewpa; Level 1's page drew `stage_poke03` to `stage_poke08` at (64, 105), (160,
/// 105), (256, 105), (64, 178), (160, 178) and (256, 178), which are Magnemite's Sp. Atk, Wailmer's
/// HP, Axew's Attack, Tentacool's Sp. Def, Noibat's Speed and Geodude's Defense (Khai, 2026-10-10).
pub fn training_regimens() -> &'static (i64, IndexMap<&'static str, i64>) {
    static TRAINING_REGIMENS: OnceLock<(i64, IndexMap<&'static str, i64>)> = OnceLock::new();
    TRAINING_REGIMENS.get_or_init(|| {
        let names = (1..39).map(|n| {
            let image: &'static str = Box::leak(format!("stage_poke{n:02}.bclim").into_boxed_str());
            (image, n - 1)
        });
        (44, names.collect())
    })
}
/// Pictures a button draws only while what it shows is caught: said "caught" after its name.
pub const CAUGHT_IMAGES: [&str; 1] = ["search_get_button.bclim"];
/// The DexNav's labels, each over its value or beside it: Name, Search Level, Level, Type,
/// Potential, First Move, Ability, Held Item; and what its red mark says in the group of each it
/// marks (spike 195: in First Move's for an egg move, Helping Hand, movie27).
pub fn dexnav_labels() -> (i64, IndexMap<i64, String>) {
    (
        84,
        [
            (13, ""),
            (15, ""),
            (19, pictures::HIGHER_LEVEL),
            (22, ""),
            (23, ""),
            (24, pictures::EGG_MOVE),
            (26, pictures::HIDDEN_ABILITY),
            (32, ""),
        ]
        .into_iter()
        .map(|(line, mark)| (line, mark.to_string()))
        .collect(),
    )
}
/// The DexNav's titles and buttons, no label of what follows: Nearby, Found, Last seen and Hidden
/// Pokémon, SEARCH, Pokédex. "Found Pokémon: Search Level" and "Pokédex: SEARCH" were said under
/// `pauses` (spike 195).
pub const DEXNAV_PLAIN: (i64, [i64; 6]) = (84, [9, 10, 11, 12, 28, 61]);
/// The Bag's pockets, drawn as icons alone, by the icon each draws (its `_l` copy while it is the
/// pocket open), and the line of text file 163 the game names the pocket with: ITEMS, MEDICINE,
/// TMs & HMs, BERRIES, KEY ITEMS (movie21).
pub fn bag_pockets() -> &'static (i64, IndexMap<&'static str, i64>) {
    static BAG_POCKETS: OnceLock<(i64, IndexMap<&'static str, i64>)> = OnceLock::new();
    BAG_POCKETS.get_or_init(|| {
        (
            163,
            IndexMap::from([
                ("ball_icon.bclim", 0),
                ("ball_icon_l.bclim", 0),
                ("kaifuku_icon.bclim", 1),
                ("kaifuku_icon_l.bclim", 1),
                ("waza_icon.bclim", 2),
                ("waza_icon_l.bclim", 2),
                ("kinomi_icon.bclim", 3),
                ("kinomi_icon_l.bclim", 3),
                ("taisetu_icon.bclim", 4),
                ("taisetu_icon_l.bclim", 4),
            ]),
        )
    })
}
/// The AreaNav's map, `map_lower`, which the Pokédex's habitat draws too: its archive opens
/// with its panes' names, and an AreaNav row keeps its zone's touch area among them at +0x04
/// (spike 152: Route 101 is `m`, Route 102 `1p`).
pub const MAP_PANES: (&str, i64, i64) = ("a/0/6/7", 0, 0x04);

/// The archives holding textures with names: 131 of the 140 with any, the other nine holding only
/// files without, which give nothing to say (spike 147). `a/1/5/2`, the title screen's, is 8.5 s of
/// the 12.7 the census takes.
pub const TEXTURE_ARCHIVES: [&str; 131] = [
    "a/0/0/0", "a/0/0/2", "a/0/0/4", "a/0/0/5", "a/0/0/9", "a/0/1/1", "a/0/1/2", "a/0/1/5",
    "a/0/1/6", "a/0/1/7", "a/0/1/8", "a/0/1/9", "a/0/2/4", "a/0/2/5", "a/0/2/6", "a/0/2/7",
    "a/0/2/8", "a/0/3/0", "a/0/3/3", "a/0/4/1", "a/0/4/2", "a/0/4/3", "a/0/4/4", "a/0/4/5",
    "a/0/4/6", "a/0/4/7", "a/0/4/8", "a/0/4/9", "a/0/5/0", "a/0/5/1", "a/0/5/2", "a/0/5/3",
    "a/0/5/4", "a/0/5/5", "a/0/5/6", "a/0/5/7", "a/0/6/1", "a/0/6/4", "a/0/6/5", "a/0/6/7",
    "a/0/6/8", "a/0/6/9", "a/0/7/0", "a/0/9/0", "a/0/9/3", "a/0/9/4", "a/0/9/5", "a/0/9/6",
    "a/0/9/7", "a/0/9/8", "a/1/0/3", "a/1/0/6", "a/1/0/7", "a/1/0/9", "a/1/1/0", "a/1/1/1",
    "a/1/1/2", "a/1/1/3", "a/1/1/4", "a/1/3/1", "a/1/3/4", "a/1/3/8", "a/1/4/0", "a/1/4/1",
    "a/1/4/2", "a/1/4/4", "a/1/4/7", "a/1/4/8", "a/1/4/9", "a/1/5/0", "a/1/5/2", "a/1/5/3",
    "a/1/5/5", "a/1/5/7", "a/1/5/9", "a/1/6/1", "a/1/6/8", "a/1/7/1", "a/1/7/3", "a/1/7/7",
    "a/1/7/9", "a/1/8/7", "a/1/8/8", "a/1/9/9", "a/2/0/0", "a/2/0/1", "a/2/0/2", "a/2/0/3",
    "a/2/0/4", "a/2/0/5", "a/2/0/6", "a/2/0/7", "a/2/0/8", "a/2/1/2", "a/2/1/3", "a/2/1/4",
    "a/2/1/5", "a/2/1/6", "a/2/1/7", "a/2/1/9", "a/2/2/4", "a/2/2/5", "a/2/2/6", "a/2/2/8",
    "a/2/3/1", "a/2/3/3", "a/2/3/4", "a/2/3/5", "a/2/3/6", "a/2/3/7", "a/2/3/8", "a/2/3/9",
    "a/2/4/0", "a/2/4/1", "a/2/4/3", "a/2/4/4", "a/2/4/5", "a/2/4/6", "a/2/4/7", "a/2/4/8",
    "a/2/4/9", "a/2/5/0", "a/2/5/5", "a/2/5/8", "a/2/5/9", "a/2/6/5", "a/2/7/6", "a/2/7/8",
    "a/2/7/9", "a/2/8/0", "a/2/9/4",
];
/// The language choice's text file (`app::lang_select`), whose lines from 6 name the languages.
pub const LANGUAGE_NAMES: (i64, i64) = (103, 6);
pub const TYPE_NAMES: i64 = 18;
/// The Pokémon's names, "Egg" at 0 and Bulbasaur at 1 (spike 178).
pub const SPECIES_NAMES: i64 = 98;
/// The field's models, a file a model by the id the ROM's sprite list gives it (544 of each), and
/// the first Pokémon's, `EV_POKE01`: the character `0x1000 + n` is drawn with model `0x1BE + n`, as
/// each move model's record of its sprite says at +0x14 (spike 193).
pub const POKEMON_MODELS: (&str, i64) = ("a/0/2/1", 0x1BE);
/// STATUS, PHYSICAL, SPECIAL: the summary's words, in the category icons' order (spike 147).
pub const MOVE_CATEGORIES: (i64, i64) = (150, 17);
/// Coolness, Beauty, Cuteness, Cleverness, Toughness: the contest's own text, in its move icons'
/// order, `contest_icon_00` to `contest_icon_04` (spike 196: Hold Back's is 00, Iron Defense's 04).
pub const CONTEST_CATEGORIES: (i64, i64) = (141, 121);
/// "Pokémon:" and "Trainer:", the contest's labels of the entrant its cursor is on (spike 196).
pub const CONTEST_ENTRANT: (i64, i64) = (141, 138);
/// The story text naming the contests' computer entrants: 69 trainers, Micah to Wallace, then
/// their 69 Pokémon in the same order, Poochin to Milotic (spike 196).
pub const CONTEST_RIVALS: i64 = 445;
/// "Conditions", in the story text (spike 196): the game's word for the five a Pokémon's star
/// draws.
pub const CONTEST_CONDITIONS: (i64, i64) = (446, 83);
/// PARALYSIS, ASLEEP, FROZEN, BURNED, POISONED, FAINTED, POKÉRUS (spike 147).
pub const CONDITIONS: (i64, i64) = (123, 50);
/// A Secret Base's decorations by number, as the save numbers them: Small Desk 0, Small Chair 10,
/// the Welcome Mats 0x13 to 0x17, Your Location 0xad (spike 154).
pub const DECORATION_NAMES: i64 = 56;
/// The items' names, a line an item by its number: the Red Orb at 534 (spike 203). File 116
/// holds the same lines.
pub const ITEM_NAMES: i64 = 114;
/// The Mach Bike's line of the items' names: "Mach Bike" in English and "マッハじてんしゃ" in
/// Japanese, the Acro Bike's the line after (spike 158).
pub const MACH_BIKE: (i64, i64) = (ITEM_NAMES, 718);
/// The Acro Bike's line of the items' names: "Acro Bike", which hops up a bumpy slope
/// (spike 159).
pub const ACRO_BIKE: (i64, i64) = (ITEM_NAMES, 719);

/// The warp kinds that are the way into one building and no other, 16 of them, each used by that
/// building's entrance alone (2026-10-06): `0x0f01` Devon Corporation in Rustboro, `0x1001` the
/// Oceanic Museum in Slateport, `0x1101` TV Mauville, `0x1201` Mauville Gym, `0x1301` Route 112's
/// Cable Car station, `0x1401` the Weather Institute on Route 119, `0x1501` the Safari Zone on Route
/// 121, `0x1601` Lilycove's Department Store, `0x1701` its Museum, `0x1801` Mossdeep's Space Center,
/// `0x1901` the Cave of Origin, from Sootopolis City by two zones, `0x1a01` the Battle Maison at the
/// Battle Resort, `0x3601` Rydel's Cycles, `0x3701` New Mauville off Route 110, `0x3801` Fortree Gym
/// and `0x3901` Mauville's Food Court. Most are named by their plate already; the rest by what their
/// town's list leaves over (`plates.entrance_names`).
pub const ENTRANCES: [i64; 16] = [
    0x0F01, 0x1001, 0x1101, 0x1201, 0x1301, 0x1401, 0x1501, 0x1601, 0x1701, 0x1801, 0x1901, 0x1A01,
    0x3601, 0x3701, 0x3801, 0x3901,
];

pub const LADDER_UP: (i64, i64) = (0x0A00, 0x0A01);
pub const LADDER_DOWN: (i64, i64) = (0x0B00, 0x0B01);
pub const TELEPORTER_PAD: i64 = 0x0601;

/// What the warp kinds that are not a door or stairs are (2026-10-06, read off the ROM's warps).
///
/// Ladders: `0x0a00` and `0x0a01` lead up, `0x0b00` and `0x0b01` down. They pair, `0x0b00` with
/// `0x0a01` and `0x0b01` with `0x0a00`, in Granite Cave, Meteor Falls, Shoal Cave, Sky Pillar, Mirage
/// Cave and Victory Road. Every `0x0b` end goes from the entrance side deeper down, Granite Cave 1F
/// to B1F to B2F, Meteor Falls to B1F, Shoal Cave to its lower room and its ice room; every `0x0a`
/// end goes back up, and Victory Road climbs to the League by `0x0a`. The player went down from zone
/// `0x7c` to `0x7d` by `0x0b00` and back by `0x0a01` (pa3ds.5.log, 2318 to 2330 s).
///
/// Teleporters: `0x0601`, 84 of them, all in Team Aqua's Hideout (zones `0x5c` to `0x61`) and Team
/// Magma's (`0xa9` to `0xae`), most leading to another pad of their own room.
///
/// The entrances are `ENTRANCES`. X and Y have no table: their kinds `0x10xx`, `0x38xx` and `0x39xx`
/// are not measured, and stay "warp".
pub fn warps() -> &'static IndexMap<i64, String> {
    static WARPS: OnceLock<IndexMap<i64, String>> = OnceLock::new();
    WARPS.get_or_init(|| {
        let mut out: IndexMap<i64, String> = IndexMap::new();
        for kind in [LADDER_UP.0, LADDER_UP.1] {
            out.insert(kind, "ladder up".to_string());
        }
        for kind in [LADDER_DOWN.0, LADDER_DOWN.1] {
            out.insert(kind, "ladder down".to_string());
        }
        out.insert(TELEPORTER_PAD, "teleporter".to_string());
        for kind in ENTRANCES {
            out.insert(kind, "entrance".to_string());
        }
        out
    })
}

/// Where ORAS keeps what `archives.GameText` reads.
pub fn rom() -> &'static RomLayout {
    static ROM: OnceLock<RomLayout> = OnceLock::new();
    ROM.get_or_init(|| RomLayout {
        towns: Some(towns().clone()),
        characters: characters().clone(),
        protagonists: true,
        story: crate::games::gen6::rom::layout::owned(story_archives()),
        moves: Some(MOVE_NAMES),
        map_chunks: Some(MAP_CHUNKS.to_string()),
        matrices: Some(MATRICES.to_string()),
        berries: Some(BERRY_SPOTS.to_string()),
        textures: TEXTURE_ARCHIVES
            .iter()
            .map(|path| path.to_string())
            .collect(),
        languages: Some(LANGUAGE_NAMES),
        types: Some(TYPE_NAMES),
        species: Some(SPECIES_NAMES),
        pokemon_models: Some((POKEMON_MODELS.0.to_string(), POKEMON_MODELS.1)),
        categories: Some(MOVE_CATEGORIES),
        contest_categories: Some(CONTEST_CATEGORIES),
        contest_entrant: Some(CONTEST_ENTRANT),
        contest_rivals: Some(CONTEST_RIVALS),
        contest_conditions: Some(CONTEST_CONDITIONS),
        conditions: Some(CONDITIONS),
        map_panes: Some((MAP_PANES.0.to_string(), MAP_PANES.1, MAP_PANES.2)),
        decorations: Some(DECORATION_NAMES),
        bike: Some(MACH_BIKE),
        acro: Some(ACRO_BIKE),
        items: Some(ITEM_NAMES),
        files: vec![sky::ARCHIVE.to_string()],
        touch_words: [
            touch_words(),
            bag_pockets(),
            dexnav_words(),
            training_bags(),
            training_regimens(),
        ]
        .into_iter()
        .map(|(file, lines)| {
            (
                *file,
                lines
                    .iter()
                    .map(|(image, &line)| (image.to_string(), line))
                    .collect(),
            )
        })
        .collect(),
        entrances: ENTRANCES.into_iter().collect(),
        caught: CAUGHT_IMAGES
            .iter()
            .map(|image| image.to_string())
            .collect(),
        labels: vec![dexnav_labels()],
        plain: vec![(DEXNAV_PLAIN.0, DEXNAV_PLAIN.1.to_vec())],
        ..RomLayout::new(
            "ORAS",
            text_archives(),
            PLACE_NAMES,
            TRAINER_NAMES,
            CLASS_NAMES,
            PC_TEXT,
            ZONES,
            ZONE_TABLE,
            TRAINERS,
            (TRAINER_CLASS, "<H"),
            TRAINER_BATTLE,
        )
    })
}

pub const DEWFORD_GYM: i64 = 0x1AC;

/// Dewford Gym's three conveyor belts across its lit corridor, at z 29, 22 and 14, by their
/// triggers' scripts (spike 132): walking north onto one carries the player back south
/// (omega_ruby_fighting_gym.ctm). That walking south onto one is let through is assumed, not seen.
pub fn belts() -> &'static IndexMap<(i64, i64), &'static str> {
    static BELTS: OnceLock<IndexMap<(i64, i64), &'static str>> = OnceLock::new();
    BELTS.get_or_init(|| {
        IndexMap::from([
            ((DEWFORD_GYM, 0x9), SOUTH),
            ((DEWFORD_GYM, 0xA), SOUTH),
            ((DEWFORD_GYM, 0xB), SOUTH),
        ])
    })
}

pub const MAUVILLE_GYM: i64 = 0x1B8;
pub const STOPPER3: i64 = 0x138;

/// Mauville Gym's electric beams, each between two posts four cells apart, by the sprite of their
/// move models (spike 146): `STOPPER3`, an invisible blocker of one cell in the eight other zones that
/// stand one, and in the Gym the beam's middle. A switch turns two beams a quarter about the post
/// between them, and the model with them (omega_ruby_movil-gym.ctm).
pub fn barriers() -> &'static HashSet<(i64, i64)> {
    static BARRIERS: OnceLock<HashSet<(i64, i64)>> = OnceLock::new();
    BARRIERS.get_or_init(|| HashSet::from([(MAUVILLE_GYM, STOPPER3)]))
}

/// The scripts of a Secret Base's way out, a trigger on the cell the player arrives in, walked onto
/// going south, and of its PC and the flag beside it, furniture records, in every one of the 51 zones
/// (spike 154): the player came into Route 111's at (18, 18) and booted the PC from in front of (12,
/// 12) (omega_ruby_secret-base.ctm).
pub const WAY_OUT: i64 = 0x2A71;
pub const BASE_PC: i64 = 0x2A72;
pub const BASE_FLAG: i64 = 0x2A73;
pub const LAVARIDGE_GYM: i64 = 0x1AD;
pub const MOSSDEEP_GYM: i64 = 0x1BB;
pub const SOOTOPOLIS_GYM: i64 = 0x1BC;
pub const SWITCH: (&str, &str) = ("signs", "switch");
pub const PASSAGE: (&str, &str) = ("exits", "passage");
pub const TRAPDOOR: (&str, &str) = ("exits", "trapdoor");
pub const GEYSER: (&str, &str) = ("exits", "geyser");
pub const LIFT: (&str, &str) = ("exits", "lift");
pub const TELEPORTER: (&str, &str) = ("exits", "teleporter");
pub const UNDERWATER_128: i64 = 0x44;
pub const SEAFLOOR_CAVERN: i64 = 0x62;
pub const UNDERWATER_126: i64 = 0x42;
pub const UNDER_SOOTOPOLIS: i64 = 0x4C;
pub const UNDERWATER_134: i64 = 0xA0;
pub const UNDERWATER_134_SOUTH: i64 = 0xA1;

/// Mauville Gym's triggers, by their scripts (spike 146): its five floor switches, each one cell,
/// the two by the entrance at (58, 38) and (66, 38), the three by Wattson at (5, 40), (13, 36) and
/// (13, 44); and the ends of the way between its two rooms, which share no floor, three cells wide
/// at the top of the entrance's room, (61, 31), and the bottom of Wattson's, (8, 49). That each end
/// leads to the other is read off the map, not seen.
///
/// Lavaridge Gym's 24, each one cell (spike 157): a trapdoor in the floor of the room the player
/// comes in by, which drops them into the hot spring below, and a geyser under a raft in the hot
/// spring, which shoots it back up through the trapdoor over it. The game names neither. Twelve of
/// each, in grids of three columns at x 12, 21 and 30 above and 90, 96 and 102 below, rows at z 53,
/// 43, 33 and 23 above (the middle column's last at 20) and 50, 40, 30 and 20 below, each trapdoor
/// over the geyser in its place in the other grid. The player stepped on the trapdoor at (21, 53),
/// came down at (96, 50), and rode the geyser at (102, 50) up to (30, 53)
/// (omega_ruby_fire-gym.ctm); that the rest pair the same way is read off the grids, not seen.
///
/// Mossdeep Gym's 11, each one cell (spike 163): seven platforms joined only by tracks of light, and
/// a **lift** at the edge of each platform where a track starts, which carries the player along it
/// to another platform. Each lift's script, 5 to 0xe, has the Gym's module ride the track of its
/// number, 0 to 9, so all ten are lifts though only the first was ridden: from the entrance's at
/// (19, 32) the player was carried 22 cells north and up to Liza and Tate. The **teleporter** beside
/// them at (21, 6), script 0xf, put the player back at the entrance at (19, 35)
/// (omega_ruby_psychic-gym.ctm). The lifts on the four middle platforms are armed by their
/// platform's switch, and the teleporter once all four are pressed; the game names none of them.
///
/// The way into Seafloor Cavern, under Route 128 (spike 164): no warp record leads there from
/// anywhere, and the underwater zone's only trigger, script 5, seven cells wide at (305, 184), is
/// the gap in the south wall of a ring of rock, round the light under the surface's dive square at
/// (307, 175). Its script ends in the zone's one call into Seafloor Cavern's first room, `0x62`, at
/// (21, 35). A third item is the zone an exit leads to, said after it: "entrance to Seafloor
/// Cavern". The player dived at (307, 190), south of the ring, swam 6 north into the gap and stood
/// in `0x62` at (21, 35) (omega_ruby_route-128-sea-floor2.ctm); from inside the ring, diving at
/// (307, 175), P's way stopped 5 short of the gap.
///
/// The way into Sootopolis City, under Route 126 (spike 165), is the same: the underwater zone's
/// only trigger, script 2, five cells wide at (55, 95), ends in its one call into `0x4c`, the
/// water under the city, at (20, 36), where the light into the city's lake is 3 north. `0x4c`'s
/// only trigger, script 2, seven wide at (17, 37), leads back to (57, 96). The listing was
/// replayed (omega_ruby_route-126-dive.ctm); neither was swum through.
///
/// The way south under Route 134 to the Sealed Chamber is two zones of sea floor joined the same
/// way, with no warp in either: `0xa0`'s only trigger, script 2, seven wide at (16, 36), puts the
/// player in `0xa1` at (19, 6), and `0xa1`'s at (16, 5) puts them back at (19, 35). Diving down
/// into `0xa0`, the list held nothing but the dive spot; the player swam south and came up in
/// `0xa1` at (19, 6) (movie44, 2026-10-13).
///
/// Python's value is `tuple[str, str] | tuple[str, str, int]`: the zone is the third item, None
/// where the tuple has two.
pub fn triggers() -> &'static IndexMap<(i64, i64), (String, String, Option<i64>)> {
    static TRIGGERS: OnceLock<IndexMap<(i64, i64), (String, String, Option<i64>)>> =
        OnceLock::new();
    TRIGGERS.get_or_init(|| {
        let owned = |(category, kind): (&str, &str)| (category.to_string(), kind.to_string(), None);
        let mut out = IndexMap::new();
        for script in 0x4..0x9 {
            out.insert((MAUVILLE_GYM, script), owned(SWITCH));
        }
        out.insert((MAUVILLE_GYM, 0x9), owned(PASSAGE));
        out.insert((MAUVILLE_GYM, 0xA), owned(PASSAGE));
        for script in 0x4..0x10 {
            out.insert((LAVARIDGE_GYM, script), owned(TRAPDOOR));
        }
        for script in 0x10..0x1C {
            out.insert((LAVARIDGE_GYM, script), owned(GEYSER));
        }
        for script in 0x5..0xF {
            out.insert((MOSSDEEP_GYM, script), owned(LIFT));
        }
        out.insert((MOSSDEEP_GYM, 0xF), owned(TELEPORTER));
        out.insert(
            (UNDERWATER_128, 0x5),
            (
                "exits".to_string(),
                "entrance".to_string(),
                Some(SEAFLOOR_CAVERN),
            ),
        );
        out.insert(
            (UNDERWATER_126, 0x2),
            (
                "exits".to_string(),
                "entrance".to_string(),
                Some(UNDER_SOOTOPOLIS),
            ),
        );
        out.insert(
            (UNDER_SOOTOPOLIS, 0x2),
            (
                "exits".to_string(),
                "way out".to_string(),
                Some(UNDERWATER_126),
            ),
        );
        out.insert(
            (UNDERWATER_134, 0x2),
            (
                PASSAGE.0.to_string(),
                PASSAGE.1.to_string(),
                Some(UNDERWATER_134_SOUTH),
            ),
        );
        out.insert(
            (UNDERWATER_134_SOUTH, 0x2),
            (
                PASSAGE.0.to_string(),
                PASSAGE.1.to_string(),
                Some(UNDERWATER_134),
            ),
        );
        for &zone in secret_bases_zones() {
            out.insert((zone, WAY_OUT), owned(("exits", "way out")));
        }
        out
    })
}

/// Where the field's work holds the Dowsing Machine's record (`gen6.field.dowsing`), found on Route 110
/// (omega_ruby_item-finder.ctm, 2026-09-29).
pub const DOWSING: i64 = 0x1D0;

/// The sprite of the thorny bush Cut clears (spike 137): Route 104's at (149, 274), where the player
/// facing it heard "You can't pass because of prickly thorns, but a Pokémon may be able to cut the
/// tree." (omega_ruby_bush.ctm). The ROM stands 42 of them in 16 zones, none with a script of its
/// own.
pub const BUSH: i64 = 0x200B;
/// The sprite of the rock Rock Smash breaks (spike 137): Rusturf Tunnel's at (41, 12), where the
/// player facing it heard "It's a cracked boulder. A Pokémon may be able to break it."
/// (omega_ruby_smash.ctm). The ROM stands 108 of them in 20 zones, ten in Rusturf Tunnel; three of
/// those have a script, 0x2a4, and the rest none: the three across the tunnel that Rock Smash breaks
/// to join its halves, as the others are broken.
pub const CRACKED_BOULDER: i64 = 0x200C;
/// The sprite of the boulder Strength pushes: Fiery Path's at (35, 44), where the player facing it
/// from (37, 44) and from (37, 45) heard "It's a big boulder, but a Pokémon may be able to push it
/// aside." (omega_ruby_strength-boulders.ctm, 2026-10-01). It stands in four cells, as its model
/// says (`mmodel.footprint`), filling the path's two rows over open floor. The ROM stands 43 of them
/// in 13 zones, none with a script of its own.
pub const BIG_BOULDER: i64 = 0x2009;

/// A Secret Base's PC, called what the game calls a PC, and its flag (spike 154). Mossdeep Gym's
/// four switches, one on each middle platform at (11, 25), (28, 27), (32, 13) and (6, 13), pressed by
/// facing them (spike 163): each sets a work value of its own to 1, which arms the lifts on its
/// platform, and the entrance's lift goes to Liza and Tate once all four are set.
pub fn furniture() -> &'static IndexMap<(i64, i64), FurnitureKind> {
    static FURNITURE: OnceLock<IndexMap<(i64, i64), FurnitureKind>> = OnceLock::new();
    FURNITURE.get_or_init(|| {
        let mut out = IndexMap::new();
        for &zone in secret_bases_zones() {
            out.insert((zone, BASE_PC), FurnitureKind::Tile(PC_TILE));
        }
        for &zone in secret_bases_zones() {
            out.insert((zone, BASE_FLAG), FurnitureKind::Word(FLAG.to_string()));
        }
        for script in 0x1..0x5 {
            out.insert(
                (MOSSDEEP_GYM, script),
                FurnitureKind::Word(SWITCH.1.to_string()),
            );
        }
        out
    })
}

/// `DexNav`, as the factory `Game.hidden` holds.
fn dex_nav(memory: Rc<dyn Memory>) -> Rc<dyn HiddenReader> {
    Rc::new(DexNav::new(memory))
}

/// `SecretBases`, as the factory `Game.secret_bases` holds.
fn secret_bases(memory: Rc<dyn Memory>, image: &[u8]) -> Rc<dyn SecretBaseReader> {
    Rc::new(SecretBases::new(memory, image))
}

/// `lambda memory, _text: AshPiles(memory)`, as a factory `Game.features` holds.
fn ash_piles(memory: Rc<dyn Memory>, _text: &GameText) -> Rc<dyn Feature> {
    Rc::new(AshPiles::new(memory))
}

/// `RotatingDoors`, as the factory `Game.doors` holds.
fn rotating_doors(memory: Rc<dyn Memory>) -> Rc<dyn DoorReader> {
    Rc::new(RotatingDoors::new(memory))
}

/// `IceFloors`, as the factory `Game.ice` holds.
fn ice_floors(memory: Rc<dyn Memory>) -> Rc<dyn IceReader> {
    Rc::new(IceFloors::new(memory))
}

/// The sprite of the game's invisible blocker, `STOPPER` among the ROM's sprite ids, drawn as
/// nothing: in Fortree Gym the stand-ins for its doors' arms (spike 161), and in Sootopolis Gym
/// nine in rows of three across the corridors up from its rooms, at (17, 45), (17, 34) and (17,
/// 23) on the floor above the lower one, which the list called nameless people (spike 167).
/// Seafloor Cavern stands six more. Their records sit in a square off the map, (36, 70) in
/// Sootopolis Gym, and the Gym's module sets them in place.
pub const STOPPER: i64 = 0x8A;

/// `STOPPER`, and `STOPPER3` everywhere but Mauville Gym, where it is the barriers: in Seafloor
/// Cavern ten round the submarine, whose furniture records are what the player reads; on each Regi
/// chamber's sealed door (Desert Ruins, Island Cave, Ancient Tomb) and on the Sealed Chamber's wall
/// the player digs through, each on the warp it shuts, the wall's three cells wide at (40, 25); five
/// each in Dewford Town, Route 104 and Route 109, and one on Route 120. Every one was listed as a
/// person, and as "Submarine Explorer 1" once the player named one so (Orion, movie44,
/// 2026-10-13).
///
/// Python's is a frozenset; an array here, collected where a set is wanted.
pub const BLOCKERS: [i64; 2] = [STOPPER, STOPPER3];

/// The shared scripts of the elevators' doors, a furniture record a cell, two side by side: two
/// doors on each of Mauville City's floors, `0x1bd` and `0x1be`, Mauville Hills' among them, and one
/// on each of the Lilycove Department Store's five, `0x143` to `0x147`, its first at (8, 9) and
/// (9, 9). Facing either cell runs the elevator (Orion, 2026-10-13); each was listed as two signs.
///
/// Python's is a frozenset of the range; a `Range` here, collected where a set is wanted.
pub const ELEVATORS: std::ops::Range<i64> = 0x27DA..0x27EC;

/// The player's looks among the ROM's sprite ids, `HEROINE_RS` and `HERO_RS` through the contest
/// outfits to `MAGMA_HERO_RS` and `AQUA_HERO_RS` (spike 87): putting the Magma Suit on in the
/// Cave of Origin fills a work object at the player's own cell with `MAGMA_HERO_RS` (spike 203).
///
/// Python's is a frozenset of the range; a `Range` here, collected where a set is wanted.
pub const PLAYER_LOOKS: std::ops::Range<i64> = 0xAB..0xB3;

/// Omega Ruby and Alpha Sapphire as the gen 6 readers read them.
pub fn game() -> &'static Game {
    static GAME: OnceLock<Game> = OnceLock::new();
    GAME.get_or_init(|| Game {
        hidden: Some(dex_nav),
        belts: belts().clone(),
        clears: IndexMap::from([
            (BUSH, CUT),
            (CRACKED_BOULDER, ROCK_SMASH),
            (BIG_BOULDER, STRENGTH),
        ]),
        dowsing: Some(DOWSING),
        barriers: barriers().clone(),
        triggers: triggers().clone(),
        furniture: furniture().clone(),
        secret_bases: Some(secret_bases),
        features: vec![BerrySpots::of, ash_piles],
        doors: Some(rotating_doors),
        ice: Some(ice_floors),
        blockers: HashSet::from(BLOCKERS),
        elevators: ELEVATORS.collect(),
        player_looks: PLAYER_LOOKS.collect(),
        scenes: SCENES,
        pictures: PICTURES,
        stairs: HashSet::from([SOOTOPOLIS_GYM]),
        apps: vec![Soaring::of],
        warps: warps().clone(),
        standings: Some(Standings::of),
        conditions: Some(Conditions::of),
        ..Game::new(
            "Omega Ruby and Alpha Sapphire",
            [OMEGA_RUBY, ALPHA_SAPPHIRE],
            DATA,
            rom().clone(),
            mmodel::DRAWN,
        )
    })
}

pub mod field;
pub mod screens;
