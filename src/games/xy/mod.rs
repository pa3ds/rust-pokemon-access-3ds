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
use crate::core::pathfind;
use crate::games::common::field::ground::{CUT, ROCK_SMASH, STRENGTH};
use crate::games::gen6::field::camera;
use crate::games::gen6::field::footsteps::{RUN, Steps, WALK};
use crate::games::gen6::game::{Climbing, Data, FurnitureKind, Game, Grinding, Rides, Vining};
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
/// The items' names, a line an item by its number as ORAS's file 114 (`oras.ITEM_NAMES`): the Poké
/// Ball at 4, which the title screen's film throws (x_intro_and_title_screen). File 98 holds the same
/// lines, and 95 and 97 the plurals.
pub const ITEM_NAMES: i64 = 96;
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
/// The fitting room's tabs, pictures of what each lists: line N of text file 143 is the tab of the
/// Nth, "hat", "top", "bottom", "dress", "socks", "shoes", "bag" and "hat accessory", and the tab
/// lit while its list is shown draws `_wh` (x_cyllage_boutique_fitting_room). The boy's dress tab,
/// `tab_onepiece`, is drawn dimmed and never lit, so its `_wh` is the others' pattern, unseen.
pub fn fitting_tabs() -> &'static (i64, IndexMap<&'static str, i64>) {
    static FITTING_TABS: OnceLock<(i64, IndexMap<&'static str, i64>)> = OnceLock::new();
    FITTING_TABS.get_or_init(|| {
        let tabs = [
            "hat", "tops", "bottoms", "onepiece", "socks", "boots", "bag", "onepoint",
        ];
        let mut names = IndexMap::new();
        for (line, tab) in tabs.iter().enumerate() {
            for lit in ["", "_wh"] {
                let image: &'static str =
                    Box::leak(format!("tab_{tab}{lit}.bclim").into_boxed_str());
                names.insert(image, line as i64);
            }
        }
        (143, names)
    })
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
        items: Some(ITEM_NAMES),
        sounds: Some(SOUND_ARCHIVE.to_string()),
        touch_words: [
            training_bags(),
            training_regimens(),
            training_button(),
            fitting_tabs(),
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
/// A sparkle in each of twelve zones whose case hands over one Mega Stone, items 665 to 683 (text
/// 96): an item found, listed under items, as ORAS's `KIRAKIRA` is.
pub const KIRAKIRA2: i64 = 0x2042;
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

/// Cyllage City Gym's rock walls (zone 0xa0, chunk file 354 of `a/0/4/1`), climbed "if you start
/// from one of those blue spots", the Gym guide says. Kind 0x59, blocked (`0x59000001`), is where a
/// face meets the floor: 28 cells, all in that file, at the face's foot. A blue spot is an open cell
/// beside one at its height, the face standing the way from it; the spots facing one way along a row
/// or a column are one face, and the open cells between its outermost spots its line. Kinds 0x51 to
/// 0x54, open (`0x5N000020`), east, west, north and south, are the step off a face sideways onto a
/// ledge at its side: each of the 10 is on a line at its foot, beside a spot that way 36 to 144
/// higher. In the tape of x_gym_2 the player walked from the entrance to (25, 32), a spot at 3, and
/// climbed in place to 13, 38 and 57 in about 2 s; went east along the face at 54, the cell they
/// were in (26, 32) to (30, 32), and stepped off east from `0x51` at (30, 32) onto the ledge (31,
/// 32) at 54; then back on, west along it and down at (25, 32) to 3, and off west onto (24, 32). The
/// step from (26, 17) onto its ledge rises 36, a step's worth on foot, so the kinds, not the heights,
/// say where a ledge is. X has 0x51 to 0x54 in chunk files 458 and 594 too, never seen (spike 245).
///
/// Kind 0x8a, open (`0x8a000020`), is the top of a slide down a wall: two rows of three south of
/// Grant, (14..16, 20) at 189 and (14..16, 27) at 99, each with blocked cells (`0x01000021`) south
/// of it falling 18 a cell, to 117 and to 9, then the floor, at 108 and at 0, walls either side. In
/// the tape of x_gym_2_slide the player stepped south from (16, 19) onto (16, 20) and was carried
/// down to (16, 25) in 0.6 s, walked onto (16, 27) and was carried to (16, 33). X has 0x8a in chunk
/// files 79, 886 and 1023 too, a row each with one or two blocked cells past it falling to lower
/// floor, so taken for slides too; never seen.
///
/// Python's `CLIMBING` constant: a function, since an `IndexMap` is no const.
pub fn climbing() -> Climbing {
    Climbing {
        face: 0x59,
        sideways: IndexMap::from([
            (0x51, pathfind::EAST),
            (0x52, pathfind::WEST),
            (0x53, pathfind::NORTH),
            (0x54, pathfind::SOUTH),
        ]),
        slide: 0x8A,
    }
}

/// Shalour City Gym's rails (zone 0xaf), skated along on roller skates: "charge at it", the
/// walkthroughs say, and the player is carried across. Its four chunks, at (0, 0), (0, 40), (40, 0)
/// and (40, 40), have three floors each, the ground table floor 0's. Kind 0x8c, blocked
/// (`0x8c000021`), is a rail's cell: 74 on floor 0 and 42 on floor 1, each rail a chain of them a cell
/// beside the next or at a corner, bending a cell at a time. Kind 0x8b, open (`0x8b000020`), is the
/// cell past a rail's end, in a row of three across it, 32 cells, all on floor 0; the one meant is
/// the middle, in line with the rail. In the tape of x_gym_3 the player skated north from (67, 64)
/// onto (67, 62), 0x8b, along (67, 61) to (67, 53), 0x8c, rising from 108 to 120 and falling to 102,
/// and onto (67, 52), 0x8b, and the floor at 102, in about 1.7 s: the one rail ridden there. The
/// cells either side of it at row 61 are ledges dropping south, so it is the one way north. Floor 0
/// has four more: west into the centre from (17, 40) along row 40 to (34, 40), east from (46, 40) to
/// (63, 40), and two bending diagonally, from (35, 26) to (50, 33) and from (30, 49) to (50, 55).
/// Floor 1 has the way out, from (40, 47) south to (40, 66) and bending east along row 71 to
/// (62, 71), falling from 117 to 18: an end only at its top, (40, 46) on floor 0 by the centre, and
/// open floor at 0 past its other, (63, 71), by the entrance; after Korrina "take the south rail
/// out", the walkthroughs say. It is taken as ridden one way, from its end, never seen. The rail from
/// (30, 49) crosses it at (40, 55), a cell of both on two floors. Six invisible blockers stand on
/// the end cells (17, 39..41) and (63, 39..41), the bridge that is put together as the four trainers
/// are beaten, never seen. X has 0x8b and 0x8c on Route 5, Route 13, Route 18, Dendemille Town and
/// Couriway Town too, never seen; Route 5's rail over the water, (695..707, 455), is broken in the
/// middle by (701, 455), `0x90000021`, blocked, and is no rail: 0x90 is in one other cell of X's,
/// mid-rail on Route 13 (spike 247).
pub const GRINDING: Grinding = Grinding {
    rail: 0x8C,
    end: 0x8B,
};

pub const SHALOUR_GYM: i64 = 0xAF;
/// The script of Shalour Gym's two triggers at the centre's edges, (35, 38) and (45, 38), each 1
/// wide and 5 deep, work 0x40f7: where Korrina's scene starts, her model, sprite 0x11, loaded only
/// then (spike 247).
pub const KORRINA: i64 = 0x7;

pub const COUMARINE_GYM: i64 = 0xBF;
/// The script of Coumarine Gym's one trigger, on its top level's east street at 720 by (65,
/// 73.7): walked onto east, it drops the player to the ground at (63.6, 73.8), as it did twice in
/// x_gym_4_climbed. Its case calls native 58 with 3 and then native 59, with no place of its own,
/// so the landing is the game's and P walks to it as a way out of the top (spike 250).
pub const WAY_DOWN: i64 = 0x2B;

pub const PRISM_LOBBY: i64 = 0x4D;
pub const CLEMONT_ROOM: i64 = 0x4E;
/// Prism Tower's Gym: its lobby, Clemont's room at the top, and Bonnie's four quiz rooms under it,
/// joined by no warp at all, each lift a trigger whose script moves the player (spike 252).
pub const QUIZ_ROOMS: std::ops::Range<i64> = 0x4F..0x53;
/// The script of the lobby's one trigger, at (19, 22) before the door of the lift in the pillar
/// at its middle: stepped onto and then walked north from, it carried the player up to the first
/// quiz room at (19, 26) (spike 252). Nothing was listed that led up to the Gym.
pub const LOBBY_LIFT: i64 = 0x3;
/// The script of each quiz room's "small square" at (19, 23), which asks Bonnie's question and
/// asks it again ("If you want to hear the question again, step on the small panel!"), script 6 on
/// the same cell once asked (spike 252).
pub const QUIZ_PANEL: i64 = 0x5;
pub const QUIZ_LIFT: i64 = 0x7;
/// The scripts of each quiz room's lift at (19, 26), which the player arrives on and which asks
/// "Which floor would you like to go to?", and of Clemont's room's, at (19, 26) too (spike 252).
pub const CLEMONT_LIFT: i64 = 0x5;

/// Coumarine City Gym's vines (zone 0xbf), a tree climbed from 0 to Ramos at 720, its levels 0,
/// 145, 290, 432, 576 and 720 platforms round the trunk on the Gym's streets, laid as floors (spike
/// 248). 42 of its furniture records are placed on streets, each on one cell of the grid
/// (`Rails.on_grid`), as high as its line's start point; each one's zone script case calls a
/// function that pushes one constant and calls a native, 56 for scripts 1 to 0x1a and 57 for 0x1b
/// to 0x2a, the constant the end's number among them. Scripts n, odd, and n + 1 are a vine's two
/// ends: 56's at the two sides of a gap between neighbouring platforms of one level, swung across,
/// and 57's at one place a level apart, climbed, all 21 pairs. An end's cell is solid at its level:
/// in the tape of x_gym_4 the player walked north up to (59, 72.9) and was stopped by the vine at
/// (59, 72), at 0, never pressing A there; its other end, script 0x1b, is at (59, 73) at 145, the
/// floor at 145 on (59, 74). No movie rides a vine: a ride is taken to land beside the other end,
/// on its open cell side by side with it at its level nearest the end grabbed, but for that end's
/// own cell (spike 250).
pub const VINING: Vining = Vining {
    swing: 56,
    climb: 57,
};

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

/// Where X's field's work holds the Dowsing Machine's record (`gen6.field.dowsing`): 0x24 before
/// ORAS's, the record laid out as ORAS's is (spike 243, x_route_8_dowsing).
pub const DOWSING: i64 = 0x1AC;

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
        dowsing: Some(DOWSING),
        blockers: HashSet::from([STOPPER]),
        ambushes: HashSet::from([TOBI_KUSA]),
        lurkers: HashSet::from([ENEMY3D]),
        pickups: HashSet::from([KIRAKIRA2]),
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
            (
                (SHALOUR_GYM, KORRINA),
                ("signs".to_string(), "gym leader spot".to_string(), None),
            ),
            (
                (COUMARINE_GYM, WAY_DOWN),
                ("exits".to_string(), "way down".to_string(), None),
            ),
            (
                (PRISM_LOBBY, LOBBY_LIFT),
                (
                    "exits".to_string(),
                    "elevator".to_string(),
                    Some(QUIZ_ROOMS.start),
                ),
            ),
            (
                (CLEMONT_ROOM, CLEMONT_LIFT),
                ("exits".to_string(), "elevator".to_string(), None),
            ),
        ])
        .into_iter()
        .chain(QUIZ_ROOMS.map(|room| {
            (
                (room, QUIZ_PANEL),
                ("signs".to_string(), "quiz panel".to_string(), None),
            )
        }))
        .chain(QUIZ_ROOMS.map(|room| {
            (
                (room, QUIZ_LIFT),
                ("exits".to_string(), "elevator".to_string(), None),
            )
        }))
        .collect(),
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
        climbing: Some(climbing()),
        grinding: Some(GRINDING),
        vining: Some(VINING),
        gates: true,
        steps: STEPS,
        training: TRAINING,
        amie: AMIE,
        field_menu: FIELD_MENU,
        worded: WORDED.iter().map(|texture| texture.to_string()).collect(),
        camera: camera::X,
        opening: Some(|memory, _language| Box::new(IntroReader::new(memory))),
        ..Game::new("X and Y", [X, Y], DATA, rom().clone(), DRAWN)
    })
}

pub mod intro;
