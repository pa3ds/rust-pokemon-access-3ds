//! Names for what the game does not name: the shipped table, the player's own, and precedence.

use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use indexmap::IndexMap;

use crate::core::names::{FURNITURE, KINDS, Names, TILES};
use crate::core::words::{ENGLISH, Words};
use crate::games::common::field::things;
use crate::games::{oras, xy};

const NORMAN: i64 = 0x13F;
const WALLY: i64 = 0xB4;
const GYM: i64 = 0x1AF;

/// pytest's `tmp_path`: a folder of this test's own, gone when the test is.
struct TmpPath(PathBuf);

impl TmpPath {
    fn new() -> Self {
        static MADE: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "pa3ds-test-names-{}-{}",
            std::process::id(),
            MADE.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path).unwrap();
        TmpPath(path)
    }
}

impl Drop for TmpPath {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for TmpPath {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

/// `Path.read_text(encoding="utf-8")`, whose universal newlines read a Windows file's CRLF as LF.
fn read_text(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap()
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

/// A game's English `sprites.ftl` holding `body`, and its words.
fn shipped(tmp_path: &Path, body: &str) -> Words {
    let folder = tmp_path.join("data").join("oras").join("en");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("sprites.ftl"), body).unwrap();
    words(ENGLISH, &tmp_path.join("data").join("oras"))
}

/// `Words(language, game)`.
fn words(language: &str, game: &Path) -> Words {
    Words::new(
        language,
        Some(game),
        IndexMap::new(),
        Rc::new(|_line: &str| {}),
    )
    .unwrap()
}

fn load(words: &Words, fingerprint: &str, home: &Path) -> Names {
    Names::load(fingerprint, Some(home), "", Some(words)).unwrap()
}

/// A sprite is one character everywhere in the game: Norman named once is Norman in the Gym
/// and in the street (spike 75).
#[test]
fn test_a_shipped_sprite_is_said_for_every_map_it_walks_on() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "sprite-0x13f = Norman\n");
    let names = load(&data, "sango-1", &tmp_path.join("home"));
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Norman"));
    assert_eq!(names.sprite(WALLY), None);
}

#[test]
fn test_a_name_the_player_gives_wins_and_is_kept() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "sprite-0x13f = Norman\n");
    let home = tmp_path.join("home");
    let names = load(&data, "sango-1", &home);
    names.name_sprite(NORMAN, "Dad").unwrap();
    names.name_sprite(WALLY, "Wally").unwrap();
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Dad"));
    assert_eq!(names.sprite(WALLY).as_deref(), Some("Wally"));
    let again = load(&data, "sango-1", &home);
    assert_eq!(again.sprite(NORMAN).as_deref(), Some("Dad"));
    assert_eq!(again.sprite(WALLY).as_deref(), Some("Wally"));
}

#[test]
fn test_an_empty_name_gives_the_character_back_to_the_shipped_table() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "sprite-0x13f = Norman\n");
    let names = load(&data, "sango-1", &tmp_path.join("home"));
    names.name_sprite(NORMAN, "Dad").unwrap();
    names.name_sprite(NORMAN, "  ").unwrap();
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Norman"));
}

/// May's sprite, which the rival's work object is filled with in the boy's game (spike 89).
const RIVAL: i64 = 0x13A;

/// The rival is "Rival" in the shipped table and May in the ROM: a word pa3ds invented never
/// replaces one the game says (ADR-0023).
#[test]
fn test_the_games_own_word_comes_before_the_shipped_one() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "sprite-0x13a = Rival\nsprite-0x13f = Norman\n");
    let names = load(&data, "sango-1", &tmp_path.join("home"));
    assert_eq!(names.sprite(RIVAL).as_deref(), Some("Rival"));
    names.use_game(
        Some(Rc::new(|character| {
            (character == RIVAL).then(|| "May".to_string())
        })),
        None,
    );
    assert_eq!(names.sprite(RIVAL).as_deref(), Some("May"));
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Norman"));
}

#[test]
fn test_the_players_own_name_comes_before_the_games() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "sprite-0x13a = Rival\n");
    let names = load(&data, "sango-1", &tmp_path.join("home"));
    names.use_game(
        Some(Rc::new(|character| {
            (character == RIVAL).then(|| "May".to_string())
        })),
        None,
    );
    names.name_sprite(RIVAL, "Neighbour").unwrap();
    assert_eq!(names.sprite(RIVAL).as_deref(), Some("Neighbour"));
    names.name_sprite(RIVAL, "").unwrap();
    assert_eq!(names.sprite(RIVAL).as_deref(), Some("May"));
}

/// A sign is a slot on one map, so naming it says nothing about the sign in the next town.
#[test]
fn test_a_place_is_named_on_its_own_map() {
    let tmp_path = TmpPath::new();
    let names = load(&shipped(&tmp_path, ""), "sango-1", &tmp_path.join("home"));
    names.name_place(GYM, 1, "the Gym's sign").unwrap();
    assert_eq!(names.place(GYM, 1).as_deref(), Some("the Gym's sign"));
    assert_eq!(names.place(GYM, 2), None);
    assert_eq!(names.place(0x000D, 1), None);
}

/// One player, two games: a sprite in one is nobody in the other.
#[test]
fn test_names_are_kept_apart_by_fingerprint() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "");
    let home = tmp_path.join("home");
    load(&data, "sango-1", &home)
        .name_sprite(NORMAN, "Norman")
        .unwrap();
    assert_eq!(load(&data, "sapphire-1", &home).sprite(NORMAN), None);
}

/// Being told the wrong name is a nuisance; a reader that stops mid-game is not.
#[test]
fn test_a_names_file_that_cannot_be_read_is_no_names_at_all() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir_all(home.join("names").join("en")).unwrap();
    fs::write(
        home.join("names").join("en").join("sango-1.toml"),
        "{not toml",
    )
    .unwrap();
    let data = shipped(&tmp_path, "sprite-0x13f = Norman\n");
    let names = load(&data, "sango-1", &home);
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Norman"));
    names.name_sprite(WALLY, "Wally").unwrap();
    assert_eq!(names.sprite(WALLY).as_deref(), Some("Wally"));
}

#[test]
fn test_a_shipped_table_that_is_not_there_is_no_names_at_all() {
    let tmp_path = TmpPath::new();
    let nowhere = words(ENGLISH, &tmp_path.join("nowhere").join("oras"));
    let names = load(&nowhere, "sango-1", &tmp_path.join("home"));
    assert_eq!(names.sprite(NORMAN), None);
    assert_eq!(names.words(TILES), IndexMap::<i64, String>::new());
}

/// A use tile by its kind, a trash can wherever it stands, and furniture by the shared script
/// it runs, a vending machine (spike 210); a key that is no number is a typo, left out, and the
/// rest of the table read.
#[test]
fn test_the_shipped_table_says_what_each_kind_of_use_tile_and_furniture_is() {
    let tmp_path = TmpPath::new();
    let data = shipped(
        &tmp_path,
        "tile-0xdd = trash can\ntile-trash = bin\nfurniture-0x28f0 = vending machine\n",
    );
    let names = Names::load("sango-1", None, "", Some(&data)).unwrap();
    assert_eq!(
        names.words(TILES),
        IndexMap::from([(0xDD, "trash can".to_string())])
    );
    assert_eq!(
        names.words(FURNITURE),
        IndexMap::from([(0x28F0, "vending machine".to_string())])
    );
}

/// The PC's word was "PC" in the code until 2026-10-14, heard in every language where the
/// ROM could not be read; it is each table's now.
#[test]
fn test_every_shipped_table_says_what_the_pc_is() {
    for game in [oras::game(), xy::game()] {
        for table in sprite_tables(&(game.data)()) {
            let language = table
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            let words = words(language, &(game.data)());
            let names = Names::load("rom", None, "", Some(&words)).unwrap();
            assert!(!names.words(TILES)[&0xD5].is_empty(), "{language}");
        }
    }
}

/// `game.DATA.glob("*/sprites.ftl")`.
fn sprite_tables(data: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(data)
        .unwrap()
        .map(|entry| entry.unwrap().path().join("sprites.ftl"))
        .filter(|table| table.is_file())
        .collect();
    found.sort();
    assert!(!found.is_empty(), "{}", data.display());
    found
}

/// The Magma Hideout's first bed: its zone's furniture 0, furniture's tag being 0 (spike 211).
const BED: (i64, i64) = (0xAA, 0);

/// A bed is its zone's own message in each of three zones, so the shipped table names it by
/// its slot; the player's name for the slot and the game's own word still come first.
#[test]
fn test_the_shipped_table_names_a_slot_behind_the_player_and_the_game() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "place-0xaa-furniture-0 = bed\n");
    let names = load(&data, "sango-1", &tmp_path.join("home"));
    assert_eq!(names.place(BED.0, BED.1).as_deref(), Some("bed"));
    names.use_game(
        None,
        Some(Rc::new(|zone, slot| {
            ((zone, slot) == BED).then(|| "Hideout".to_string())
        })),
    );
    assert_eq!(names.place(BED.0, BED.1).as_deref(), Some("Hideout"));
    names.name_place(BED.0, BED.1, "Maxie's bed").unwrap();
    assert_eq!(names.place(BED.0, BED.1).as_deref(), Some("Maxie's bed"));
}

/// Each game's tables name the same things in every language (Orion, 2026-10-14): a word
/// the ROM has none for in one language is pa3ds's own there, as "Nurse" is in English.
#[test]
fn test_every_language_names_what_english_does() {
    for game in [oras::game(), xy::game()] {
        let english = ids(&(game.data)().join("en").join("sprites.ftl"));
        for table in sprite_tables(&(game.data)()) {
            assert_eq!(ids(&table), english, "{}", table.display());
        }
    }
}

fn ids(path: &Path) -> std::collections::HashSet<String> {
    let source = fs::read_to_string(path).unwrap();
    let resource = match fluent_syntax::parser::parse(source.as_str()) {
        Ok(resource) => resource,
        Err((resource, _)) => resource,
    };
    resource
        .body
        .iter()
        .filter_map(|entry| match entry {
            fluent_syntax::ast::Entry::Message(message) => Some(message.id.name.to_string()),
            _ => None,
        })
        .collect()
}

/// Sharing names is copying the file, so it is written sorted and as words, not as a dump,
/// and it opens by saying which game and language it is for — `sango-1` says nothing to somebody
/// who has just been sent one.
#[test]
fn test_the_players_names_are_a_file_worth_sending_to_someone() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    let names = Names::load(
        "sango-1",
        Some(&home),
        "Pokémon Omega Ruby",
        Some(&shipped(&tmp_path, "")),
    )
    .unwrap();
    names.name_sprite(WALLY, "Wally").unwrap();
    names.name_sprite(NORMAN, "Norman").unwrap();
    let body = read_text(&home.join("names").join("en").join("sango-1.toml"));
    assert!(
        body.starts_with(
            "# Names you have given in Pokémon Omega Ruby.\n# ROM sango-1. Language en."
        ),
        "{body}"
    );
    let loaded: ::toml::Table = body.parse().unwrap();
    let sprites = loaded["sprites"].as_table().unwrap();
    assert_eq!(sprites.len(), 2);
    assert_eq!(sprites["0x13f"].as_str(), Some("Norman"));
    assert_eq!(sprites["0xb4"].as_str(), Some("Wally"));
    assert!(body.find("\"0x13f\"").unwrap() < body.find("\"0xb4\"").unwrap());
}

/// The title comes from the emulator's own window, which is not up in a test and will not be
/// up if somebody opens a names file with the game closed. pa3ds holds no table of game names to
/// fall back on, by design (ADR-0031), so the ROM says what it is called.
#[test]
fn test_a_rom_the_emulator_does_not_name_names_itself() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    let names = load(&shipped(&tmp_path, ""), "sango-1", &home);
    names.name_sprite(NORMAN, "Norman").unwrap();
    let body = read_text(&home.join("names").join("en").join("sango-1.toml"));
    assert!(
        body.starts_with("# Names you have given in sango-1.\n# ROM sango-1. Language en."),
        "{body}"
    );
}

/// English names for a ROM played in French belong to neither, and the shipped table has been
/// laid out this way all along.
#[test]
fn test_a_name_is_kept_per_language_because_a_name_is_a_word_in_one() {
    let tmp_path = TmpPath::new();
    let data = shipped(&tmp_path, "");
    let home = tmp_path.join("home");
    load(&data, "sango-1", &home)
        .name_sprite(NORMAN, "Dad")
        .unwrap();
    let french = words("fr", &tmp_path.join("data").join("oras"));
    assert_eq!(load(&french, "sango-1", &home).sprite(NORMAN), None);
    assert_eq!(
        load(&data, "sango-1", &home).sprite(NORMAN).as_deref(),
        Some("Dad")
    );
}

/// A quotation mark or a backslash written into the file unescaped would take every name
/// after it with the line it broke.
#[test]
fn test_a_name_may_hold_anything_a_player_can_type() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    let awkward = "the \"old\" man \\ upstairs";
    let names = load(&shipped(&tmp_path, ""), "sango-1", &home);
    names.name_sprite(NORMAN, awkward).unwrap();
    names
        .name_place(GYM, 2, "caf\u{e9}\u{3001} \u{30dd}\u{30b1}\u{30e2}\u{30f3}")
        .unwrap();
    let again = load(&shipped(&tmp_path, ""), "sango-1", &home);
    assert_eq!(again.sprite(NORMAN).as_deref(), Some(awkward));
    assert_eq!(
        again.place(GYM, 2).as_deref(),
        Some("caf\u{e9}\u{3001} \u{30dd}\u{30b1}\u{30e2}\u{30f3}")
    );
}

/// `--no-log` style: a run that writes nothing still answers while it lasts.
#[test]
fn test_names_without_a_home_are_kept_for_this_run_only() {
    let shipped = IndexMap::from([("0x13f".to_string(), "Norman".to_string())]);
    let names = Names::new(Some(&shipped), None, None, None, None).unwrap();
    names.name_sprite(WALLY, "Wally").unwrap();
    assert_eq!(names.sprite(WALLY).as_deref(), Some("Wally"));
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Norman"));
}

/// The `[places]` of a names file, as (key, name) in any order.
fn places(path: &Path) -> Vec<(String, String)> {
    let loaded: ::toml::Table = fs::read_to_string(path).unwrap().parse().unwrap();
    let mut out: Vec<(String, String)> = loaded["places"]
        .as_table()
        .unwrap()
        .iter()
        .map(|(key, name)| (key.clone(), name.as_str().unwrap().to_string()))
        .collect();
    out.sort();
    out
}

/// Orion, 2026-10-11: a person editing the file can tell a door from a sign from a boulder,
/// where the kind and the number were one sum, Rustboro's warp 3 `0x10:4294967299`.
#[test]
fn test_a_place_is_written_map_kind_number() {
    let tmp_path = TmpPath::new();
    let mine = tmp_path.join("sango-1.toml");
    let names = Names::new(None, Some(&mine), None, None, None).unwrap();
    names
        .name_place(0x10, 1 << 32 | 3, "Devon Corporation")
        .unwrap();
    names
        .name_place(0x69, 13 << 32 | 5, "the last one")
        .unwrap();
    names.name_place(GYM, 1, "the Gym's sign").unwrap();
    assert_eq!(
        places(&mine),
        [
            ("0x10:warp:3".to_string(), "Devon Corporation".to_string()),
            (
                "0x1af:furniture:1".to_string(),
                "the Gym's sign".to_string()
            ),
            ("0x69:model:5".to_string(), "the last one".to_string()),
        ]
    );
}

#[test]
fn test_a_file_written_the_old_way_is_read_and_kept_the_new_way() {
    let tmp_path = TmpPath::new();
    let mine = tmp_path.join("sango-1.toml");
    fs::write(
        &mine,
        "[places]\n\"0x10:4294967299\" = \"Devon Corporation\"\n\"0x10:warp:4\" = \"House\"\n",
    )
    .unwrap();
    let names = Names::new(None, Some(&mine), None, None, None).unwrap();
    assert_eq!(
        names.place(0x10, 1 << 32 | 3).as_deref(),
        Some("Devon Corporation")
    );
    assert_eq!(names.place(0x10, 1 << 32 | 4).as_deref(), Some("House"));
    assert_eq!(
        places(&mine),
        [
            ("0x10:warp:3".to_string(), "Devon Corporation".to_string()),
            ("0x10:warp:4".to_string(), "House".to_string()),
        ]
    );
}

/// A kind added to the field without a word here would be written as its bare tag. Python reads
/// the tags off the module; here they are listed, and a new one is added to this list too.
#[test]
fn test_every_kind_of_slot_has_a_word() {
    let mut tags: Vec<i64> = [
        things::FURNITURE_SOURCE,
        things::WARP_SOURCE,
        things::TILE_SOURCE,
        things::EDGE_SOURCE,
        things::BERRY_SOURCE,
        things::TRIGGER_SOURCE,
        things::SECRET_BASE_SOURCE,
        things::ASH_SOURCE,
        things::BIKE_SOURCE,
        things::DIVE_SOURCE,
        things::STAIRS_SOURCE,
        things::APP_SOURCE,
        things::DECORATION_SOURCE,
        things::MODEL_SOURCE,
        things::CURRENT_SOURCE,
        things::RIDE_SOURCE,
        things::CLIMB_SOURCE,
        things::HIDDEN_SOURCE,
        things::VINE_SOURCE,
        things::OPENING_SOURCE,
    ]
    .map(|value| value >> 32)
    .to_vec();
    tags.sort();
    assert_eq!(tags, (0..KINDS.len() as i64).collect::<Vec<_>>());
}
