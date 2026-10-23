//! Pokémon X and Y: read by the gen 6 profile and readers, with only what X keeps elsewhere here.
//!
//! Measured on X in spike 100, on Route 4 by Lumiose Gate: the zone files, the chunks, the zone table
//! and the move models are ORAS's, but for the counts of sections the shared readers take in either
//! game's, and the sprite a move model is drawn with, which sits at +0x890 rather than +0x898. The ROM
//! keeps the same kinds of file under other numbers, each text archive one number on from ORAS's, and
//! the Town Map's list of what each town has where ORAS keeps the AreaNav's (spike 101). Y is X's other
//! version and is taken to be laid out as X is; no Y has been read.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use indexmap::IndexMap;

use crate::core::keyfile::data_path;
use crate::games::common::field::ground::{CUT, ROCK_SMASH, STRENGTH};
use crate::games::gen6::field::footsteps::{RUN, Steps, WALK};
use crate::games::gen6::game::{Data, FurnitureKind, Game, Rides};
use crate::games::gen6::rom::layout::{RomLayout, TownFacilities, owned};
use crate::games::xy::intro::IntroReader;
use crate::py::PySet;

/// X's own words, for what the game does not say itself (ADR-0040): beside its scenes, a folder a
/// language holding `sprites.ftl`, the fallback names for the people on its maps (spike 108), and
/// `pictures.ftl`, its opening's choices (spike 216).
pub const DATA: Data = data;

/// `data_path() / "xy"`, which Python makes as the module loads.
pub fn data() -> PathBuf {
    data_path().unwrap_or_default().join("xy")
}

pub const X: i64 = 0x0004000000055D00;
pub const Y: i64 = 0x0004000000055E00;

/// The text archive a language (`a/0/7/3` is the Japanese in kanji), each told by its PC lines'
/// language in spike 100.
pub fn text_archives() -> &'static IndexMap<&'static str, &'static str> {
    static TEXT_ARCHIVES: OnceLock<IndexMap<&'static str, &'static str>> = OnceLock::new();
    TEXT_ARCHIVES.get_or_init(|| {
        IndexMap::from([
            ("ja", "a/0/7/2"),
            ("en", "a/0/7/4"),
            ("fr", "a/0/7/5"),
            ("it", "a/0/7/6"),
            ("de", "a/0/7/7"),
            ("es", "a/0/7/8"),
            ("ko", "a/0/7/9"),
        ])
    })
}

/// The story text archive a language, the zones' scripts' messages (`a/0/8/1` the Japanese in
/// kanji), each told in spike 107 by its plates saying one building in all seven.
pub fn story_archives() -> &'static IndexMap<&'static str, &'static str> {
    static STORY_ARCHIVES: OnceLock<IndexMap<&'static str, &'static str>> = OnceLock::new();
    STORY_ARCHIVES.get_or_init(|| {
        IndexMap::from([
            ("ja", "a/0/8/0"),
            ("en", "a/0/8/2"),
            ("fr", "a/0/8/3"),
            ("it", "a/0/8/4"),
            ("de", "a/0/8/5"),
            ("es", "a/0/8/6"),
            ("ko", "a/0/8/7"),
        ])
    })
}

/// Sprite -> trainer, for the people X's story names who stand as a sprite of their own, as
/// ORAS's `archives.CHARACTERS` is (spike 108). Each sprite's id in X's list (text file 76), the
/// zones it stands in and a line of the story that names that person all agree; the word is the
/// ROM's trainer name, in whichever language is asked for. The rival stands in no zone file: they
/// are drawn into a work object (+0x890, spike 100), and their sprites are named by their ids and
/// the story's lines, not yet seen drawn in a replay.
pub fn characters() -> &'static IndexMap<i64, i64> {
    static CHARACTERS: OnceLock<IndexMap<i64, i64>> = OnceLock::new();
    CHARACTERS.get_or_init(|| {
        IndexMap::from([
            // FRIEND1: alone of the three in Vaniville, on Route 2 and in the Flare HQ,
            // whose lines name Shauna and neither of the others: Shauna
            (0x03, 321),
            // FRIEND2, beside FRIEND3 where "Uh, my name is Tierno." (Tower of Mastery,
            // story 281:7): Tierno
            (0x04, 324),
            // FRIEND3, alone of the three in Frost Cavern, where "My name is Trevor."
            // (story 191:6): Trevor
            (0x05, 325),
            // HAKASE1: "I'm Professor Sycamore!" in his Lumiose lab (story 389:2): Sycamore
            (0x06, 327),
            // CHAMPION: Lumiose, Coumarine, the Radiant Chamber and the League: Diantha
            (0x0A, 276),
            // BIGFOUR1, in the Ironworks Chamber, who says he is Wikstrom: Wikstrom
            (0x0B, 187),
            // BIGFOUR2, in the Blazing Chamber: "I am Malva": Malva
            (0x0C, 269),
            // BIGFOUR3, in the Dragonmark Chamber: Drasna, the one left (spike 108)
            (0x0D, 270),
            // BIGFOUR4, in the Flood Chamber, who talks as a Chef: Siebold (spike 108)
            (0x0E, 271),
            // LEADER1, in Santalune's Gym: Viola
            (0x0F, 6),
            // LEADER2, in Cyllage's Gym: Grant
            (0x10, 76),
            // LEADER3, in Shalour's Gym and the Tower of Mastery: Korrina
            (0x11, 21),
            // LEADER4, in Coumarine's Gym: Ramos
            (0x12, 22),
            // LEADER5, in Prism Tower: Clemont
            (0x13, 23),
            // LEADER6, in Laverre's Gym: Valerie
            (0x14, 24),
            // LEADER7, in Anistar's Gym: Olympia
            (0x15, 25),
            // LEADER8, in Snowbelle's Gym: Wulfric
            (0x16, 26),
            // FLAREBOSS: Lumiose, Lysandre Labs and the Flare HQ: Lysandre
            (0x17, 303),
            // FLAREBOSS2, in the Flare HQ alone: Lysandre
            (0x9C, 303),
            // AZ: Geosenge, Route 13, and Lysandre Labs, where "That is AZ.": AZ
            (0x21, 602),
            // RIVALM and RIVALM_EX, drawn into a work object: Calem, "our son"
            (0x93, 130),
            (0x9F, 130),
            // RIVALW and RIVALW_EX, likewise: Serena, "our daughter"
            (0x94, 575),
            (0xA0, 575),
        ])
    })
}

/// Where X's things' and Pokémon's models start, `MONSBALL` and `EV_POKE01` (spikes 100 and 218): a
/// work object's `DRAWN` holds the model it was filled with, which is a person's sprite but not a
/// thing's or a Pokémon's. Mom's Rhyhorn by the house is `0x106`, sprite `0x1014` (spike 221).
pub const MODELS: (i64, i64) = (0xAB, 0xF2);

/// The Poké Puffs' names, line for line ORAS's file 81 (`oras.PUFF_NAMES`; spike 228).
pub const PUFF_NAMES: i64 = 67;
/// Pokémon-Amie's wallpapers and interior items, line for line ORAS's file 110
/// (`oras.GOODS_NAMES`), the English with a plural's ending after each (`garc.singular`).
pub const GOODS_NAMES: i64 = 92;
/// Super Training's bags in the list its page opens, as ORAS's (`oras.TRAINING_BAGS`): line N of
/// text file 23 is the bag of `sand_bagNN.bclim`, HP Bag S at 1, Strength Bag at 19 (spike 228).
pub fn training_bags() -> &'static (i64, IndexMap<&'static str, i64>) {
    static TRAINING_BAGS: OnceLock<(i64, IndexMap<&'static str, i64>)> = OnceLock::new();
    TRAINING_BAGS.get_or_init(|| {
        let names = (1..27).map(|n| {
            let image: &'static str = Box::leak(format!("sand_bag{n:02}.bclim").into_boxed_str());
            (image, n)
        });
        (23, names.collect())
    })
}
/// Super Training's regimens in its regimen menu, as ORAS's (`oras.TRAINING_REGIMENS`): line N-1 of
/// text file 41 is the regimen of `stage_pokeNN.bclim`, line for line ORAS's file 44 but for X's own
/// two, "Kick Out! Get the Dragalge Corps!" and "The Battle for the Best: Version X!". The tutorial's
/// menu drew `stage_poke02`, "Get Spewpa with an Energy Shot!" (spike 228, x_super_train2).
pub fn training_regimens() -> &'static (i64, IndexMap<&'static str, i64>) {
    static TRAINING_REGIMENS: OnceLock<(i64, IndexMap<&'static str, i64>)> = OnceLock::new();
    TRAINING_REGIMENS.get_or_init(|| {
        let names = (1..39).map(|n| {
            let image: &'static str = Box::leak(format!("stage_poke{n:02}.bclim").into_boxed_str());
            (image, n - 1)
        });
        (41, names.collect())
    })
}
/// The Super Training page's lower-left button, `spar_button.bclim`, which opens the regimen menu:
/// "Super Training", line 178 of text file 55, the PSS questionnaire's answer, the one line of X's
/// text that is that name alone; X has no PokéNav, whose app list names it in ORAS (spike 228).
pub fn training_button() -> &'static (i64, IndexMap<&'static str, i64>) {
    static TRAINING_BUTTON: OnceLock<(i64, IndexMap<&'static str, i64>)> = OnceLock::new();
    TRAINING_BUTTON.get_or_init(|| (55, IndexMap::from([("spar_button.bclim", 178)])))
}

/// Where X keeps a step's moment (spike 231): the footstep check at `0x00719af8` looks the model's
/// animation up in 23 movements of three ids (`0x007933b0`), walking 0x04, 0x5c and 0x6f, running
/// 0x05, 0x5d and 0x70, and plays at the frames of `0x0079443c`, 6 and 13 walking, 0 and 8
/// running. The model keeps its animation at +0x97c, which holds its frame at +0x18 and its id at
/// +0x4e, four on from ORAS's; the ground word at +0x148 as ORAS. The footstep is bits 11 to 15 of
/// the word (`0x003e0a34`), and the labels' table (`0x00793804`, 17 footsteps of 23 movements of
/// two feet) has no walk or run for 0, 1, 14 and 15.
pub const STEPS: Steps = Steps {
    animation: 0x97C,
    animation_id: 0x4E,
    frame: 0x18,
    movements: &[
        (0x04, WALK),
        (0x5C, WALK),
        (0x6F, WALK),
        (0x05, RUN),
        (0x5D, RUN),
        (0x70, RUN),
    ],
    falls: &[(WALK, &[6, 13]), (RUN, &[0, 8])],
    shift: 11,
    silent: &[0, 1, 14, 15],
};
/// Every sound the game plays, named, its footsteps by Omega Ruby's names (spike 231).
pub const SOUND_ARCHIVE: &str = "sound/xy_sound.bcsar";

/// Place names in file 72 ("Route 4" at the index Route 4's row holds), trainer classes 20 and
/// names 21, the PC's lines 74 in the order ORAS has them; zone N's file is `a/0/1/2` file N and the
/// zone table file 360; a trainer's class is a byte at +0x01 and its battle at +0x02 (spike 100).
/// The Town Map's list is `a/0/6/8`: a row of 0x40 bytes a town, its zone at +0x00, its description's
/// line at +0x24 and its facilities' lines from +0x28, all lines of text file 31 (spike 101). The map
/// chunks are `a/0/4/1`, each file's section 4 the streets of Lumiose's that run through it
/// (spike 106). Lumiose's street marks are `a/2/5/5`, a file a zone of its streets, and its street
/// names text file 71, two lines a name (spike 107). The story's people are `CHARACTERS` (spike
/// 108). The moves' names are text file 13, as file 14 (spike 136); the Pokémon's names file 80
/// (spike 178). The Pokémon on the map are the field's models from `EV_POKE01`, `0xf2` of `a/0/2/1`,
/// each tagged as its battle models in `a/0/0/7` are, Fletchling's `pm0753` (spike 218).
pub fn rom() -> &'static RomLayout {
    static ROM: OnceLock<RomLayout> = OnceLock::new();
    ROM.get_or_init(|| RomLayout {
        towns: Some(TownFacilities {
            archive: "a/0/6/8".to_string(),
            file: 0,
            text: 31,
            row: 0x40,
            zone: 0x00,
            first: 0x28,
        }),
        chunks: Some("a/0/4/1".to_string()),
        story: owned(story_archives()),
        street_marks: Some("a/2/5/5".to_string()),
        street_names: Some(71),
        characters: characters().clone(),
        moves: Some(13),
        map_chunks: Some("a/0/4/1".to_string()),
        species: Some(80),
        pokemon_models: Some(("a/0/2/1".to_string(), MODELS.1)),
        battle_models: Some("a/0/0/7".to_string()),
        puffs: Some(PUFF_NAMES),
        goods: Some(GOODS_NAMES),
        sounds: Some(SOUND_ARCHIVE.to_string()),
        touch_words: [training_bags(), training_regimens(), training_button()]
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
        ..RomLayout::new(
            "X and Y",
            text_archives(),
            72,
            21,
            20,
            74,
            "a/0/1/2",
            360,
            "a/0/3/8",
            (0x01, "<B"),
            0x02,
        )
    })
}

/// Where a move model keeps the sprite it is drawn with: +0x10's sprite again for everybody, and
/// MONSBALL for an item ball (spike 100).
pub const DRAWN: i64 = 0x890;

/// The player's house in Vaniville Town: the bedroom and the ground floor, side by side on one
/// chunk of one zone with no floor between them. Its stairs are triggers whose scripts move the
/// player from one room to the other, the bedroom's at (9, 15) and the ground floor's at (31, 35),
/// the only triggers of either game that do (spike 217).
pub const HOUSE: i64 = 0x2;

/// The thorny bush Cut clears, `ITREE01`, as ORAS's (`oras.BUSH`): its shared script, 0x2712 of
/// `a/0/3/1` file 27, shows "You can't pass because of prickly thorns, but a Pokémon may be able to
/// cut the tree." (story 130:3). 25 in 12 zones, Route 3 to Route 22 and the Pokémon Village
/// (spike 219).
pub const BUSH: i64 = 0x200B;
/// The rock Rock Smash breaks, `IROCK01`: shared script 0x2710, "It's a cracked boulder. A Pokémon
/// may be able to break it." (story 130:1). 61 in 13 zones (spike 219).
pub const CRACKED_BOULDER: i64 = 0x200C;
/// The boulder Strength pushes, `KROCK01`: shared script 0x2714, "It's a big boulder, but a
/// Pokémon may be able to push it aside." (story 130:5). 25 in 12 zones (spike 219).
pub const BIG_BOULDER: i64 = 0x2009;
/// The wall Rock Smash breaks, `IWALL01`, which ORAS has not: shared script 0x2716, "It's a cracked
/// wall. A Pokémon may be able to break it." (story 130:9). Three on Route 15, two in Victory Road,
/// seven in the Lost Hotel (spike 219).
pub const CRACKED_WALL: i64 = 0x2034;
/// The invisible blocker, `STOPPER`, ORAS's id and model: six in Shalour City and four in the Team
/// Flare Secret HQ, each running shared script 0x7d0, which shows nothing (spikes 218 and 219).
pub const STOPPER: i64 = 0x8A;
/// Grass a wild Pokémon leaps out of as the player walks near, rustling till it does: three on
/// Route 6 and three on Route 18, with no script; "Whoa! A wild Audino leaped out of the tall
/// grass!" (text 124:261, x_route-6.ctm, 2026-10-19).
pub const TOBI_KUSA: i64 = 0x2002;
/// A spot a wild Pokémon waits at unseen, `ENEMY3D`: nine in Glittering Cave (zone 0x12f), records
/// 3 to 11, all running shared script 0x7d0, drawn as nothing. Walking up the corridor from (32, 72)
/// the player stopped at (32, 50) under a "!" and a wild battle began, "A wild Onix appeared!"; after
/// fleeing, up walked them straight through (32, 49) to (32, 37) (spike 239, x_glittering_cave).
pub const ENEMY3D: i64 = 0x95;

/// The bit of a cell's word on a strand of Santalune Gym's web (zone 0x38, chunk file 344 of
/// `a/0/4/1`): the floor is open cells along strands, many running diagonally, two cells touching
/// only at a corner with both cells between them walls. Strand cells are `0x00006828` where a
/// diagonal joins and `0x00006820` along a straight run, walls `0x01006821`. The player walked
/// (17, 65) to (18, 66), and (17, 63) to (18, 62) to (17, 61), in one move each, in a replay of
/// the player's movie. Across X's chunks, two open cells touching only at a corner, both between them
/// walls, with the bit on both cells occur 111 times (90 in the Gym's chunk, five each in files 327,
/// 328, 330 and 331, one in 733), and in Omega Ruby never; without the bit they are the corners of
/// ordinary walls, 177 in X and 307 in ORAS, and stay shut. ORAS uses the bit for other things, so
/// it is X's alone (spike 223).
pub const STRAND: i64 = 0x8;

/// Route 9 (zone 0x10b, chunk files 187 to 222 of `a/0/4/1`), crossed on a Rhyhorn. Its spiky
/// ground, kind 0x32 with the blocked bit set (`0x32000025`), 490 cells, the executable counts as tall
/// grass (it tests 0x32 with 0x20 to 0x23 at `0x003e1054`), so wild Pokémon are met on it. Its flat
/// ground, kind 0x31 and open (`0x31000020`), 161 cells in five patches, always lies between the spikes
/// and plain ground (tested alone, with 0x33, at `0x003e0f30`). In the tape of x_route9.ctm the player
/// walked onto flat ground at (15, 60), faced the Rhyhorn, character `0x100f`, standing in (19, 60) and
/// (19, 61), pressed A, and was moved into its cell, riding: +0x06 0x01 to 0x21 and +0x07 0x00 to 0x03,
/// the bike's bits. They rode east from (20, 61), flat, onto the spikes at (21, 61). The Rhyhorn's
/// model stays where it was got on, and is put at the player's cell whenever the map's models are
/// built again: at (19, 61) while they rode to (31, 61), at (32, 61) from 39.37 s, as the chunks east
/// loaded, and at (46, 59), the player's cell, after a battle. Route 9's ten rocks, sprite `0x200d`
/// (`IROCK02`), the Rhyhorn smashes with A ("press A button to smash any boulders in your path", the
/// walkthroughs say): two at (51, 58) and (51, 60) close the only way east across the spikes, listed
/// "18 north" and "18 north 2 west" from (33, 61) at 91.76 s, gone from the list after 103 s, and the
/// player was at (58, 59) by 130 s. Route 17's Mamoswine, whose kinds 0x33 and 0x34 the executable
/// tests beside these, is left out: no recording shows it (spike 238).
///
/// Python's `RIDES` constant: a function, since a `HashSet` is no const.
pub fn rides() -> Rides {
    Rides {
        ground: HashSet::from([0x32]),
        stops: HashSet::from([0x31]),
        mounts: HashSet::from([0x100F]),
        smashes: HashSet::from([0x200D]),
    }
}

pub const SANTALUNE_LOBBY: i64 = 0x37;
pub const SANTALUNE_WEB: i64 = 0x38;
/// Santalune Gym's two poles, a furniture record each way: the lobby's at (20, 24) and the web
/// room's at (20, 62), both running script 1. The lobby script's case 1 moves the player to the web
/// room at (20, 64), as a trigger with script 1 at (20, 25) does walking north onto it; the web
/// room's case 1 moves the player to the lobby at (20, 25). The guide's line (story 420:1) says "Jump
/// on that pole there", and the Italian and German say you slide down it. Both zones listed the poles
/// as signs (spike 224).
pub const POLE: i64 = 0x1;
/// The stairs beyond Viola's tent in the web room, the zone's only trigger: script 4, 3 wide and 1
/// deep at (19, 22), walked onto going north, whose case moves the player to the lobby at (23, 21).
/// Viola says "Take the stairs that you find beyond it, and you'll be back up to the entrance" (story
/// 419:5), and the player's movie walked north from (20, 44) onto it and arrived there. The room had
/// no exit listed (spike 224).
pub const WEB_STAIRS: i64 = 0x4;

/// The shared scripts of the elevators' doors, a furniture record a cell, two side by side, in 24
/// zones of Lumiose City and Lysandre Labs: file 14 of `a/0/3/1`, whose messages are story 145's,
/// "Which floor would you like to go to?" (spike 220).
///
/// Python's is a frozenset of the range; a `Range` here, collected where a set is wanted.
pub const ELEVATORS: std::ops::Range<i64> = 0x27DA..0x280A;

pub const ENTRANCES: [i64; 6] = [0x1001, 0x1101, 0x1601, 0x3701, 0x3800, 0x3901];
pub const TELEPORTER_PADS: [i64; 2] = [0x0001, 0x0601];
pub const WAYS_OUT: [i64; 5] = [0x0400, 0x0800, 0x0901, 0x0902, 0x0903];
pub const DOORS: [i64; 3] = [0x0201, 0x0F01, 0x3600];

/// What X's warp kinds that are not a door from outdoors, the mat out or stairs are, each told by
/// where its warps lead and the kind of the warp each arrives at (spike 220). Teleporters: `0x0601`
/// and `0x0001`, Lysandre Labs' 26 pads, each to a pad of its own zone. Ways out: `0x0800` out of
/// each Gym but Prism Tower's, `0x0901` to `0x0903` Prism Tower's, and `0x0400` out of a Santalune and
/// a Lumiose building. Doors: `0x0201`, the Elite Four's and the Radiant Chamber's inner doors and two
/// into Lysandre Labs, `0x0f01` the Tower of Mastery's, `0x3600` one in the Team Flare Secret HQ,
/// each arriving at a mat out. Entrances: ways in from outdoors to one building's mat or Gym, the
/// Gyms of Santalune, Laverre, Anistar, Snowbelle, Shalour, Cyllage and Coumarine, Prism Tower's
/// four, Shabboneau Castle, Ambrette Aquarium, Kiloude Station, the Battle Maison, the Battle Chateau
/// and the Poké Ball Factory's inner building. Left "warp": `0x0202`, `0x0203`, `0x0300`, `0x0302`
/// and `0x0303`, which lead into houses and between outdoor zones alike.
///
/// Python builds it with `dict.fromkeys` over each frozenset, so its order is each set's as
/// CPython lays it out (`PySet`).
pub fn warps() -> &'static IndexMap<i64, String> {
    static WARPS: OnceLock<IndexMap<i64, String>> = OnceLock::new();
    WARPS.get_or_init(|| {
        let mut out: IndexMap<i64, String> = IndexMap::new();
        for (kinds, word) in [
            (&TELEPORTER_PADS[..], "teleporter"),
            (&WAYS_OUT[..], "way out"),
            (&DOORS[..], "door"),
            (&ENTRANCES[..], "entrance"),
        ] {
            for kind in PySet::of(kinds.iter().copied()).iter() {
                out.insert(kind, word.to_string());
            }
        }
        out
    })
}

/// The Pokémon-Amie menu's PLAY, DECORATE and SWITCH, pictures of their words that X's text has no
/// line for: named by `pictures.ftl` and offered beside the menu's close, as ORAS's text names them.
/// Named only by a picture's words, they were taken for part of close and left out (spike 228,
/// x_super_train3).
///
/// Python's is a frozenset of strings, which it iterates in an order of its own: an `IndexSet` here
/// (`docs/porting.md`); it only adds to a dict looked up by image.
pub const WORDED: [&str; 3] = [
    "kwg_button01.bclim",
    "kwg_button02.bclim",
    "kwg_button03.bclim",
];
/// Where X's Pokémon-Amie page, `FM_Kawaigari`, keeps its Pokémon's number: 0x28 before ORAS's,
/// as its Super Training page is, Frogadier's 657 (spike 228, x_super_train2).
pub const AMIE: i64 = 0x124;
/// Where X's Super Training page, `FM_Training`, keeps its party's PokeIcon: 0x28 before ORAS's,
/// and its records with it, Frogadier's at +0x124 (spike 228).
pub const TRAINING: i64 = 0xC8;
/// Where X's `field::subscreen::FieldMenu` keeps its button manager and the byte set while an app
/// of the bottom screen has the input: ORAS's +0x7C and +0x9C, 0xC earlier (spike 228). It read 0
/// on the PSS's page and Super Training's, and 1 from the bags list on.
pub const FIELD_MENU: (i64, i64) = (0x70, 0x90);

/// X and Y as the gen 6 readers read them.
pub fn game() -> &'static Game {
    static GAME: OnceLock<Game> = OnceLock::new();
    GAME.get_or_init(|| Game {
        clears: IndexMap::from([
            (BUSH, CUT),
            (CRACKED_BOULDER, ROCK_SMASH),
            (BIG_BOULDER, STRENGTH),
            (CRACKED_WALL, ROCK_SMASH),
        ]),
        blockers: HashSet::from([STOPPER]),
        ambushes: HashSet::from([TOBI_KUSA]),
        lurkers: HashSet::from([ENEMY3D]),
        elevators: ELEVATORS.collect(),
        warps: warps().clone(),
        triggers: IndexMap::from([
            (
                (HOUSE, 0x6),
                ("exits".to_string(), "stairs down".to_string(), None),
            ),
            (
                (HOUSE, 0x5),
                ("exits".to_string(), "stairs up".to_string(), None),
            ),
            (
                (SANTALUNE_WEB, WEB_STAIRS),
                (
                    "exits".to_string(),
                    "stairs up".to_string(),
                    Some(SANTALUNE_LOBBY),
                ),
            ),
        ]),
        furniture: IndexMap::from([
            (
                (SANTALUNE_LOBBY, POLE),
                FurnitureKind::Exit("exits".to_string(), "warp".to_string(), SANTALUNE_WEB),
            ),
            (
                (SANTALUNE_WEB, POLE),
                FurnitureKind::Exit("exits".to_string(), "warp".to_string(), SANTALUNE_LOBBY),
            ),
        ]),
        rooms: HashSet::from([HOUSE]),
        models: Some(MODELS),
        strands: STRAND,
        rides: Some(rides()),
        gates: true,
        steps: STEPS,
        training: TRAINING,
        amie: AMIE,
        field_menu: FIELD_MENU,
        worded: WORDED.iter().map(|texture| texture.to_string()).collect(),
        opening: Some(|memory, _language| Box::new(IntroReader::new(memory))),
        ..Game::new("X and Y", [X, Y], DATA, rom().clone(), DRAWN)
    })
}

pub mod intro;
