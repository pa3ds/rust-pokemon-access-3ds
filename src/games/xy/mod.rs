//! Pokémon X and Y: read by the gen 6 profile and readers, with only what X keeps elsewhere here.
//!
//! Measured on X in spike 100, on Route 4 by Lumiose Gate: the zone files, the chunks, the zone table
//! and the move models are ORAS's, but for the counts of sections the shared readers take in either
//! game's, and the sprite a move model is drawn with, which sits at +0x890 rather than +0x898. The ROM
//! keeps the same kinds of file under other numbers, each text archive one number on from ORAS's, and
//! the Town Map's list of what each town has where ORAS keeps the AreaNav's (spike 101). Y is X's other
//! version and is taken to be laid out as X is; no Y has been read.

use std::sync::OnceLock;

use indexmap::IndexMap;

use crate::games::gen6::game::{Data, Game};
use crate::games::gen6::rom::layout::{RomLayout, TownFacilities, owned};
use crate::games::xy::intro::{IntroReader, looks};

/// Words the game does not say itself, a folder a language: `sprites.toml`, X's fallback names
/// for the people on its maps (spike 108), and `looks.toml`, its opening's choices (spike 216).
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

/// Each language's `looks.toml` from the same folder, where there is one (`intro::looks`).
pub const LOOKS: Data = &[("en", include_str!("data/en/looks.toml"))];

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

/// Place names in file 72 ("Route 4" at the index Route 4's row holds), trainer classes 20 and
/// names 21, the PC's lines 74 in the order ORAS has them; zone N's file is `a/0/1/2` file N and the
/// zone table file 360; a trainer's class is a byte at +0x01 and its battle at +0x02 (spike 100).
/// The Town Map's list is `a/0/6/8`: a row of 0x40 bytes a town, its zone at +0x00, its description's
/// line at +0x24 and its facilities' lines from +0x28, all lines of text file 31 (spike 101). The map
/// chunks are `a/0/4/1`, each file's section 4 the streets of Lumiose's that run through it
/// (spike 106). Lumiose's street marks are `a/2/5/5`, a file a zone of its streets, and its street
/// names text file 71, two lines a name (spike 107). The story's people are `CHARACTERS` (spike
/// 108). The moves' names are text file 13, as file 14 (spike 136); the Pokémon's names file 80
/// (spike 178).
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

/// X and Y as the gen 6 readers read them.
pub fn game() -> &'static Game {
    static GAME: OnceLock<Game> = OnceLock::new();
    GAME.get_or_init(|| Game {
        opening: Some(|memory, language| {
            Box::new(IntroReader::new(memory, looks(LOOKS, language)))
        }),
        ..Game::new("X and Y", [X, Y], DATA, rom().clone(), DRAWN)
    })
}

pub mod intro;
