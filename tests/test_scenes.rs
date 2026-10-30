//! Descriptions of scenes: found by the line they follow, said a beat at a time (spike 203).

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use indexmap::IndexMap;
use regex::Regex;

use super::fake_rom::TmpDir;
use super::fake_speech::Recorder;
use crate::core::speech::Speech;
use crate::core::words::{self, ENGLISH, Words};
use crate::games::common::announce::{Announcements, Speaker};
use crate::games::common::dialogue::{PageShown, Shown};
use crate::games::common::scenes::{
    Described, FILE, GameClock, Recall, Scene, Scenes, last_page, load, shipped,
};
use crate::games::sm::field;
use crate::py::{Log, Result};

/// A part of a story line: text, or a control sequence given as (code, args...).
enum Part<'a> {
    Text(&'a str),
    Control(&'a [i64]),
}

/// A story line as the ROM has it: text, and control sequences given as (code, args...).
fn line(parts: &[Part]) -> Vec<u8> {
    let mut units: Vec<i64> = Vec::new();
    for part in parts {
        match part {
            Part::Text(text) => units.extend(text.encode_utf16().map(i64::from)),
            Part::Control(control) => {
                units.extend([0x10, control.len() as i64]);
                units.extend(control.iter());
            }
        }
    }
    units
        .iter()
        .flat_map(|unit| (*unit as u16).to_le_bytes())
        .collect()
}

const PAGE: &[i64] = &[0xBE01];
/// An insert: the player's name.
const NAME: &[i64] = &[0x0100, 0];

fn page(raw: &[u8]) -> Result<(i64, Regex)> {
    let found = last_page(raw)?;
    assert!(found.is_some());
    Ok(found.expect("asserted"))
}

#[test]
fn test_a_line_is_matched_by_its_last_page_with_anything_where_an_insert_was() -> Result<()> {
    let (index, words) = page(&line(&[
        Part::Text("This is the time"),
        Part::Control(PAGE),
        Part::Text("Reclaim the possibility\nyou once held!"),
    ]))?;
    assert!(index == 1 && words.is_match("Reclaim the possibility you once held!"));
    let (index, words) = page(&line(&[
        Part::Control(NAME),
        Part::Text(" changed into the Magma Suit!"),
    ]))?;
    assert!(index == 0 && words.is_match("Raiden changed into the Magma Suit!"));
    let (_, words) = page(&line(&[Part::Text("Gurrrooouuuhhh!!!")]))?;
    assert!(!words.is_match("Gurrraaahhhh! Gurrrooouuuhhh!!!"));
    Ok(())
}

/// The player's gender choosing the words after it: "boy", 3 letters, or "girl", 4 after them.
const GENDER: &[i64] = &[0x1100, 255, 0x0403];

/// Norman's "How about our little boy's favorite?" is one line for a boy and a girl, the ROM
/// writing "boygirl" after a control saying how long each is (story 253:147, or_sky_pillar_7).
#[test]
fn test_words_the_players_gender_chooses_are_matched_as_either() -> Result<()> {
    let (_, words) = page(&line(&[
        Part::Text("Hmm.\nHow about our little "),
        Part::Control(GENDER),
        Part::Text("boygirl's favorite?"),
    ]))?;
    assert!(words.is_match("Hmm. How about our little boy's favorite?"));
    assert!(words.is_match("Hmm. How about our little girl's favorite?"));
    assert!(!words.is_match("Hmm. How about our little boygirl's favorite?"));
    Ok(())
}

/// The log a test keeps, and what it logged.
fn logging() -> (Log, Rc<RefCell<Vec<String>>>) {
    let logged: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let kept = logged.clone();
    (
        Rc::new(move |line: &str| kept.borrow_mut().push(line.to_string())),
        logged,
    )
}

/// A log that keeps nothing.
fn quiet() -> Log {
    Rc::new(|_: &str| {})
}

/// `Words(language, game)`, put in place.
fn use_words(language: &str, game: &Path) -> Result<()> {
    words::r#use(Some(Rc::new(Words::new(
        language,
        Some(game),
        IndexMap::new(),
        quiet(),
    )?)));
    Ok(())
}

/// `Described(nothing, nothing)`: no line of the ROM's, no word of the game's, no save.
fn nothing_described() -> Result<Rc<Described>> {
    Ok(Rc::new(Described::new(
        Rc::new(|_: &str, _: i64, _: i64| Ok(None)),
        Rc::new(|_: &str, _: i64| None),
        None,
    )?))
}

/// `Described.said`, as `Scenes` is handed it.
fn said_of(described: &Rc<Described>) -> Rc<dyn Fn(&str) -> Option<String>> {
    let described = described.clone();
    Rc::new(move |message: &str| described.said(message))
}

/// A game's `scenes.toml` and English `scenes.ftl`, its words put in place, and the scenes as the
/// runtime loads them: `lines` out of the ROM by (archive, file, number), `named` the game's
/// words by (kind, number).
#[allow(clippy::too_many_arguments)]
fn described(
    tmp_path: &Path,
    timing: &str,
    words: &str,
    lines: &[((&str, i64, i64), Vec<u8>)],
    named: &[((&str, i64), &str)],
    recall: Option<Recall>,
    log: &Log,
    title: Option<i64>,
) -> Result<(Vec<Scene>, Rc<Described>)> {
    let game = tmp_path.join("data").join("oras");
    fs::create_dir_all(game.join("en")).unwrap();
    fs::write(game.join(FILE), timing).unwrap();
    fs::write(game.join("en").join("scenes.ftl"), words).unwrap();
    use_words(ENGLISH, &game)?;
    let found_lines: Rc<IndexMap<(String, i64, i64), Vec<u8>>> = Rc::new(
        lines
            .iter()
            .map(|((archive, file, number), raw)| {
                ((archive.to_string(), *file, *number), raw.clone())
            })
            .collect(),
    );
    let found_names: IndexMap<(String, i64), String> = named
        .iter()
        .map(|((kind, number), word)| ((kind.to_string(), *number), word.to_string()))
        .collect();
    let lines_of = found_lines.clone();
    let words_of = Rc::new(Described::new(
        Rc::new(move |archive: &str, file: i64, number: i64| {
            Ok(lines_of.get(&(archive.to_string(), file, number)).cloned())
        }),
        Rc::new(move |kind: &str, number: i64| {
            found_names.get(&(kind.to_string(), number)).cloned()
        }),
        recall,
    )?);
    let scenes = load(
        &fs::read_to_string(game.join(FILE)).unwrap(),
        &|archive: &str, file: i64, number: i64| {
            Ok(found_lines
                .get(&(archive.to_string(), file, number))
                .cloned())
        },
        &|message: &str| words_of.known(message),
        log,
        title,
    )?;
    Ok((scenes, words_of))
}

/// Groudon woken in Seafloor Cavern (movie40): Archie's "Augh... You..." in zone 0x6f, then the
/// demo, then ten seconds on; and Tabitha's line, then the arrival on Route 128.
#[test]
fn test_a_scene_is_said_a_beat_at_a_time() -> Result<()> {
    let tmp_path = TmpDir::new();
    let (log, logged) = logging();
    let (scenes, words) = described(
        tmp_path.path(),
        r#"
[[scene]]
after = "story:402:30"
zone = 0x6f
[[scene.beat]]
id = "wakes-1"
[[scene.beat]]
demo = true
id = "wakes-2"
[[scene.beat]]
at = 10
id = "wakes-3"

[[scene]]
after = "story:402:42"
zone = 0x6f
[[scene.beat]]
zone = 0x39
id = "glare-1"

[[scene]]
after = "story:402:43"
zone = 0x6f
[[scene.beat]]
id = "missing-1"
"#,
        "
wakes-1 = { TRAINER(235) } walks to the lava.
wakes-2 = The Orb glows.
wakes-3 = { POKEMON(383) } wakes.
glare-1 = The sky glares.
missing-1 = { ITEM(9999) } is nowhere.
",
        &[
            (("story", 402, 30), line(&[Part::Text("Augh... You...")])),
            (("story", 402, 42), line(&[Part::Text("Child! Hurry up!")])),
            (("story", 402, 43), line(&[Part::Text("Missing.")])),
        ],
        &[(("trainer", 235), "Maxie"), (("pokemon", 383), "Groudon")],
        None,
        &log,
        None,
    )?;
    assert!(
        scenes.len() == 2
            && *logged.borrow()
                == [
                    "scene after story:402:43: left out, the ROM does not have its line or its names"
                ]
    );
    let said: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let now = Rc::new(Cell::new(0.0));
    let clock = now.clone();
    let mut reader = Scenes::new(scenes, said_of(&words), Rc::new(move || Ok(clock.get())));
    let mut say = |text: &str| said.borrow_mut().push(text.to_string());
    let last = || said.borrow().last().cloned().unwrap_or_default();
    reader.heard(0, "Augh... You...", Some(0x6E), &mut say)?;
    assert!(said.borrow().is_empty());
    reader.heard(0, "Augh... You...", Some(0x6F), &mut say)?;
    assert_eq!(*said.borrow(), ["Maxie walks to the lava."]);
    reader.poll(&mut say)?;
    assert_eq!(*said.borrow(), ["Maxie walks to the lava."]);
    now.set(30.0);
    reader.demo(None, &mut say)?;
    assert_eq!(last(), "The Orb glows.");
    now.set(39.0);
    reader.poll(&mut say)?;
    assert_eq!(last(), "The Orb glows.");
    now.set(40.0);
    reader.poll(&mut say)?;
    assert_eq!(last(), "Groudon wakes.");
    reader.heard(0, "Child! Hurry up!", Some(0x6F), &mut say)?;
    assert_eq!(last(), "Groudon wakes.");
    reader.arrived(0x39, &mut say)?;
    assert_eq!(last(), "The sky glares.");
    // Waiting for Route 128 and arriving somewhere else lets the scene go.
    reader.heard(0, "Child! Hurry up!", Some(0x6F), &mut say)?;
    reader.arrived(0x6D, &mut say)?;
    reader.arrived(0x39, &mut say)?;
    assert_eq!(
        said.borrow()
            .iter()
            .filter(|text| *text == "The sky glares.")
            .count(),
        1
    );
    Ok(())
}

/// "Welcome home, [name]!" is said to a boy and a girl alike, and their rooms are zones of their
/// own (or_ and as_sky_pillar_7): the room's beat comes in either, and nowhere else.
#[test]
fn test_a_beat_waiting_for_several_zones_comes_in_any_one() -> Result<()> {
    let tmp_path = TmpDir::new();
    let (scenes, words) = described(
        tmp_path.path(),
        r#"
[[scene]]
after = "story:66:35"
zone = 0x6
[[scene.beat]]
zone = [0xe0, 0xe2]
id = "room-1"
"#,
        "room-1 = Your room.\n",
        &[(
            ("story", 66, 35),
            line(&[
                Part::Text("Welcome home, "),
                Part::Control(NAME),
                Part::Text("!"),
            ]),
        )],
        &[],
        None,
        &quiet(),
        None,
    )?;
    let said: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let mut reader = Scenes::with_monotonic(scenes, said_of(&words));
    let mut say = |text: &str| said.borrow_mut().push(text.to_string());
    for room in [0xE0, 0xE2] {
        reader.heard(0, "Welcome home, Raiden!", Some(0x6), &mut say)?;
        reader.arrived(room, &mut say)?;
    }
    reader.heard(0, "Welcome home, Raiden!", Some(0x6), &mut say)?;
    reader.arrived(0xDF, &mut say)?;
    reader.arrived(0xE0, &mut say)?;
    assert_eq!(*said.borrow(), ["Your room.", "Your room."]);
    Ok(())
}

/// Kyogre caught or knocked out plays the same scene (test_kyogre_scene): either line in zone 0x79
/// starts it, and a scene one of whose lines the ROM lacks is left out.
#[test]
fn test_a_scene_after_several_lines_follows_any_one_of_them() -> Result<()> {
    let tmp_path = TmpDir::new();
    let (log, logged) = logging();
    let (scenes, words) = described(
        tmp_path.path(),
        r#"
[[scene]]
after = ["text:147:67", "text:148:1"]
zone = 0x79
[[scene.beat]]
id = "clears-1"

[[scene]]
after = ["text:147:67", "text:148:9"]
zone = 0x79
[[scene.beat]]
id = "clears-1"
"#,
        "clears-1 = The rain is gone.\n",
        &[
            (
                ("text", 147, 67),
                line(&[
                    Part::Text("Gotcha! "),
                    Part::Control(&[0x0101, 0]),
                    Part::Text(" was caught!"),
                ]),
            ),
            (
                ("text", 148, 1),
                line(&[
                    Part::Text("The wild "),
                    Part::Control(&[0x0101, 0]),
                    Part::Text(" fainted!"),
                ]),
            ),
        ],
        &[],
        None,
        &log,
        None,
    )?;
    assert!(
        scenes.len() == 1
            && *logged.borrow()
                == [
                    "scene after text:147:67 or text:148:9: left out, the ROM does not have its \
                     line or its names"
                ]
    );
    let said: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let mut reader = Scenes::with_monotonic(scenes, said_of(&words));
    let mut say = |text: &str| said.borrow_mut().push(text.to_string());
    reader.heard(0, "The wild Kyogre fainted!", Some(0x79), &mut say)?;
    reader.heard(0, "Gotcha! Kyogre was caught!", Some(0x79), &mut say)?;
    reader.heard(0, "The wild Zubat fainted!", Some(0x70), &mut say)?;
    assert_eq!(*said.borrow(), ["The rain is gone.", "The rain is gone."]);
    Ok(())
}

/// The timing names a beat no `scenes.ftl` has: said as its id, it would be gibberish.
#[test]
fn test_a_beat_with_no_words_in_any_language_leaves_its_scene_out() -> Result<()> {
    let tmp_path = TmpDir::new();
    let (log, logged) = logging();
    let (scenes, _) = described(
        tmp_path.path(),
        "[[scene]]\nzone = 0x76\n[[scene.beat]]\nid = \"gone-1\"\n",
        "",
        &[],
        &[],
        None,
        &log,
        None,
    )?;
    assert!(
        scenes.is_empty()
            && *logged.borrow()
                == ["scene after arriving: left out, the ROM does not have its line or its names"]
    );
    Ok(())
}

/// ADR-0040: a translation half done is half English, never silent.
#[test]
fn test_a_beat_the_language_lacks_is_said_in_english() -> Result<()> {
    let tmp_path = TmpDir::new();
    let game = tmp_path.path().join("data").join("oras");
    fs::create_dir_all(game.join("fr")).unwrap();
    fs::create_dir_all(game.join("en")).unwrap();
    fs::write(
        game.join(FILE),
        "[[scene]]\nzone = 0x76\n[[scene.beat]]\nid = \"glow-1\"\n\
         [[scene.beat]]\nid = \"glow-2\"\n",
    )
    .unwrap();
    fs::write(
        game.join("en").join("scenes.ftl"),
        "glow-1 = Magma glows.\nglow-2 = It is hot.\n",
    )
    .unwrap();
    fs::write(
        game.join("fr").join("scenes.ftl"),
        "glow-1 = Le magma luit.\n",
    )
    .unwrap();
    use_words("fr", &game)?;
    let words = nothing_described()?;
    let mut said: Vec<String> = Vec::new();
    let scenes = load(
        &fs::read_to_string(game.join(FILE)).unwrap(),
        &|_: &str, _: i64, _: i64| Ok(None),
        &|message: &str| words.known(message),
        &quiet(),
        None,
    )?;
    Scenes::with_monotonic(scenes, said_of(&words))
        .arrived(0x76, &mut |text: &str| said.push(text.to_string()))?;
    assert_eq!(said, ["Le magma luit.", "It is hot."]);
    Ok(())
}

/// Steven's "Good luck to you." on Route 128 is one line in one zone in both versions, and he
/// flies off into Omega Ruby's glare (movie40) but Alpha Sapphire's rain (Orion's
/// movie1_kyogre_awakens, 2026-10-14): a scene with a title is the running title's alone.
#[test]
fn test_a_scene_of_one_version_is_left_out_of_the_other() -> Result<()> {
    let tmp_path = TmpDir::new();
    let timing = r#"
[[scene]]
after = "story:148:46"
zone = 0x39
title = 0x000400000011C400
[[scene.beat]]
id = "glare-1"

[[scene]]
after = "story:148:46"
zone = 0x39
[[scene.beat]]
id = "off-1"
"#;
    let words = "glare-1 = Steven flies off into the glare.\noff-1 = Steven flies off.\n";
    let lines = [(("story", 148, 46), line(&[Part::Text("Good luck to you.")]))];
    let said_in = |title: Option<i64>| -> Result<Vec<String>> {
        let (log, logged) = logging();
        let (scenes, described_) = described(
            tmp_path.path(),
            timing,
            words,
            &lines,
            &[],
            None,
            &log,
            title,
        )?;
        assert!(logged.borrow().is_empty());
        let mut said: Vec<String> = Vec::new();
        let mut reader = Scenes::with_monotonic(scenes, said_of(&described_));
        reader.heard(0, "Good luck to you.", Some(0x39), &mut |text: &str| {
            said.push(text.to_string())
        })?;
        Ok(said)
    };
    assert_eq!(
        said_in(Some(0x000400000011C400))?,
        ["Steven flies off into the glare."]
    );
    assert_eq!(said_in(Some(0x000400000011C500))?, ["Steven flies off."]);
    assert_eq!(said_in(None)?, ["Steven flies off into the glare."]);
    Ok(())
}

/// The Cave of Origin's deepest room, `0x76`: nothing shows the way on but the room itself
/// (Orion, 2026-10-12), so it is described each time the player comes down into it.
#[test]
fn test_a_scene_with_no_line_is_said_on_arriving_in_its_zone() -> Result<()> {
    let tmp_path = TmpDir::new();
    let (scenes, words) = described(
        tmp_path.path(),
        "[[scene]]\nzone = 0x76\n[[scene.beat]]\nid = \"glow-1\"\n",
        "glow-1 = Magma glows to the north.\n",
        &[],
        &[],
        None,
        &quiet(),
        None,
    )?;
    let mut said: Vec<String> = Vec::new();
    let mut reader = Scenes::with_monotonic(scenes, said_of(&words));
    let mut say = |text: &str| said.push(text.to_string());
    reader.arrived(0x74, &mut say)?;
    reader.arrived(0x76, &mut say)?;
    reader.arrived(0x74, &mut say)?;
    reader.arrived(0x76, &mut say)?;
    assert_eq!(
        said,
        ["Magma glows to the north.", "Magma glows to the north."]
    );
    Ok(())
}

/// Omega Ruby's opening (OR_character-desc.ctm): the professor's lines come before the player
/// stands in any zone, and the last beat waits for the truck, where the field begins.
#[test]
fn test_a_line_in_no_zone_is_said_before_the_player_is_anywhere() -> Result<()> {
    let tmp_path = TmpDir::new();
    let (log, logged) = logging();
    let (scenes, words) = described(
        tmp_path.path(),
        r#"
[[scene]]
after = "text:118:60"
[[scene.beat]]
id = "truck-1"
[[scene.beat]]
zone = 0x18f
id = "truck-2"
"#,
        "truck-1 = The doors swing wide.\ntruck-2 = You stand in the truck.\n",
        &[(
            ("text", 118, 60),
            line(&[Part::Text("Come see me in my Pokémon Lab!")]),
        )],
        &[],
        None,
        &log,
        None,
    )?;
    assert!(logged.borrow().is_empty());
    let said: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let mut reader = Scenes::with_monotonic(scenes, said_of(&words));
    let mut say = |text: &str| said.borrow_mut().push(text.to_string());
    reader.heard(0, "Come see me in my Pokémon Lab!", Some(0x10), &mut say)?;
    assert!(said.borrow().is_empty());
    reader.heard(0, "Come see me in my Pokémon Lab!", None, &mut say)?;
    reader.arrived(0x18F, &mut say)?;
    assert_eq!(
        *said.borrow(),
        ["The doors swing wide.", "You stand in the truck."]
    );
    Ok(())
}

/// The Pokémon League's corridors play their scenes as the player walks across the trigger
/// halfway along each, with no line before or after (movie49, Orion 2026-10-16): said once since
/// arriving, and the room beyond after it, as the player arrives there.
#[test]
fn test_a_scene_of_a_trigger_is_said_as_the_player_first_steps_onto_it() -> Result<()> {
    let tmp_path = TmpDir::new();
    let (scenes, words) = described(
        tmp_path.path(),
        r#"
[[scene]]
zone = 0x16b
trigger = 0
[[scene.beat]]
id = "doors-1"
[[scene.beat]]
zone = 0x162
id = "doors-2"
"#,
        "doors-1 = The doors open.\ndoors-2 = A square room.\n",
        &[],
        &[],
        None,
        &quiet(),
        None,
    )?;
    let said: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let mut reader = Scenes::with_monotonic(scenes, said_of(&words));
    let mut say = |text: &str| said.borrow_mut().push(text.to_string());
    reader.arrived(0x16B, &mut say)?;
    assert!(said.borrow().is_empty());
    assert_eq!(reader.triggers(0x16B), HashSet::from([0]));
    assert_eq!(reader.triggers(0x162), HashSet::new());
    reader.stepped(0x16B, 0, &mut say)?;
    reader.stepped(0x16B, 0, &mut say)?;
    assert_eq!(*said.borrow(), ["The doors open."]);
    assert_eq!(reader.triggers(0x16B), HashSet::new());
    reader.arrived(0x162, &mut say)?;
    assert_eq!(*said.borrow(), ["The doors open.", "A square room."]);
    reader.arrived(0x16B, &mut say)?;
    reader.stepped(0x16B, 0, &mut say)?;
    assert_eq!(said.borrow().last().unwrap(), "The doors open.");
    Ok(())
}

/// `recall(kind, number)` out of a table, as the tests' lambdas over a dict are.
fn recalling(recorded: Vec<((&'static str, i64), &'static str)>) -> Recall {
    let recorded: IndexMap<(String, i64), String> = recorded
        .into_iter()
        .map(|((kind, number), said)| ((kind.to_string(), number), said.to_string()))
        .collect();
    Rc::new(move |kind: &str, number: i64| Ok(recorded.get(&(kind.to_string(), number)).cloned()))
}

/// The credits draw the Pokémon that won each Gym and great battle, which the save records
/// (spike 222): `MEMORY(N)` is read as the beat is said, and a moment the save never recorded is
/// not described.
#[test]
fn test_the_players_own_pokemon_are_named_from_the_save_as_each_beat_is_said() -> Result<()> {
    let tmp_path = TmpDir::new();
    let (scenes, words) = described(
        tmp_path.path(),
        r#"
[[scene]]
zone = 0x166
[[scene.beat]]
id = "credits-1"
[[scene.beat]]
id = "credits-2"
[[scene.beat]]
id = "credits-3"
[[scene.beat]]
id = "credits-4"
[[scene.beat]]
id = "credits-5"
"#,
        "
credits-1 = { TRAINER(4) } watches.
credits-2 = Your { MEMORY(24) } faces Wally's Gallade.
credits-3 = Your { MEMORY(4) } is nowhere.
credits-4 = { TEXT(123, 91) } shines among the stars.
credits-5 = { SPRITE(179) } walks in.
",
        &[(("text", 123, 91), line(&[Part::Text("THE\nEND")]))],
        &[(("trainer", 4), "May")],
        Some(recalling(vec![
            (("memory", 24), "Latios"),
            (("sprite", 179), "Professor Birch"),
        ])),
        &quiet(),
        None,
    )?;
    let said: RefCell<Vec<String>> = RefCell::new(Vec::new());
    Scenes::with_monotonic(scenes, said_of(&words)).arrived(0x166, &mut |text: &str| {
        said.borrow_mut().push(text.to_string())
    })?;
    assert_eq!(
        *said.borrow(),
        [
            "May watches.",
            "Your Latios faces Wally's Gallade.",
            "THE END shines among the stars.",
            "Professor Birch walks in."
        ]
    );
    Ok(())
}

/// `YOURS(N)` is the nickname the player gave a moment's Pokémon (Orion, 2026-10-16), or, where
/// they gave it none, `scene-yours` around its name; the rival's stays `MEMORY(N)`.
#[test]
fn test_the_players_own_pokemon_are_called_by_their_nicknames_or_the_files_yours() -> Result<()> {
    let tmp_path = TmpDir::new();
    let (scenes, words) = described(
        tmp_path.path(),
        r#"
[[scene]]
zone = 0x166
[[scene.beat]]
id = "battle-1"
[[scene.beat]]
id = "battle-2"
[[scene.beat]]
id = "battle-3"
"#,
        "
scene-yours = your { $pokemon }
battle-1 = May with her { MEMORY(2) }, and you with { YOURS(1) }.
battle-2 = You and { YOURS(25) } face Steven.
battle-3 = You and { YOURS(4) } are nowhere.
",
        &[],
        &[],
        Some(recalling(vec![
            (("memory", 1), "Torchic"),
            (("yours", 1), "Blaze"),
            (("memory", 2), "Mudkip"),
            (("memory", 25), "Primal Reversion Groudon"),
        ])),
        &quiet(),
        None,
    )?;
    let said: RefCell<Vec<String>> = RefCell::new(Vec::new());
    Scenes::with_monotonic(scenes, said_of(&words)).arrived(0x166, &mut |text: &str| {
        said.borrow_mut().push(text.to_string())
    })?;
    assert_eq!(
        *said.borrow(),
        [
            "May with her Mudkip, and you with Blaze.",
            "You and your Primal Reversion Groudon face Steven."
        ]
    );
    Ok(())
}

/// ADR-0038: Omega Ruby's intro is two apps before the title screen, and A skips straight to the
/// title screen (spike 236): what the skipped app had left to say goes unsaid.
#[test]
fn test_a_scene_of_an_app_is_said_as_it_starts_and_let_go_as_another_starts() -> Result<()> {
    let tmp_path = TmpDir::new();
    let (scenes, words) = described(
        tmp_path.path(),
        r#"
[[scene]]
app = "movie::MovieProc"
[[scene.beat]]
at = 1
id = "movie-1"
[[scene.beat]]
at = 6
id = "movie-2"
[[scene]]
app = "title::TitleProc"
[[scene.beat]]
id = "title"
[[scene]]
after = "text:118:60"
[[scene.beat]]
id = "line-1"
[[scene.beat]]
demo = true
id = "line-2"
"#,
        "movie-1 = A Poké Ball rushes at you.\nmovie-2 = Fire.\ntitle = The title screen.\n\
         line-1 = He waves.\nline-2 = The ground shakes.\n",
        &[(
            ("text", 118, 60),
            line(&[Part::Text("Come see me in my Pokémon Lab!")]),
        )],
        &[],
        None,
        &quiet(),
        None,
    )?;
    let said: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let now = Rc::new(Cell::new(0.0));
    let clock = now.clone();
    let mut reader = Scenes::new(scenes, said_of(&words), Rc::new(move || Ok(clock.get())));
    let mut say = |text: &str| said.borrow_mut().push(text.to_string());
    reader.started("movie::MovieProc", Some(0x10), &mut say)?;
    now.set(2.0);
    reader.poll(&mut say)?;
    assert!(said.borrow().is_empty());
    reader.started("movie::MovieProc", None, &mut say)?;
    now.set(4.0);
    reader.poll(&mut say)?;
    assert_eq!(*said.borrow(), ["A Poké Ball rushes at you."]);
    reader.started("app::title_menu::Proc", None, &mut say)?;
    now.set(20.0);
    reader.poll(&mut say)?;
    assert_eq!(*said.borrow(), ["A Poké Ball rushes at you."]);
    reader.started("title::TitleProc", None, &mut say)?;
    assert_eq!(said.borrow().last().unwrap(), "The title screen.");
    reader.heard(0, "Come see me in my Pokémon Lab!", None, &mut say)?;
    reader.started("app::field_demo::Proc", None, &mut say)?;
    reader.demo(None, &mut say)?;
    assert_eq!(
        said.borrow()[said.borrow().len() - 2..],
        ["He waves.", "The ground shakes."]
    );
    Ok(())
}

/// ADR-0038: the title screen's logo is the game's own name out of its ROM, and its prompt a
/// picture's words, as `pictures.ftl` calls it or as its texture's name says (spike 236).
#[test]
fn test_words_drawn_as_pictures_are_said_as_the_picture_and_the_rom_name_them() -> Result<()> {
    let tmp_path = TmpDir::new();
    let english = tmp_path.path().join("data").join("oras").join("en");
    fs::create_dir_all(&english).unwrap();
    fs::write(
        english.join("pictures.ftl"),
        "texture-push_start = Press Start\n",
    )
    .unwrap();
    let (scenes, words) = described(
        tmp_path.path(),
        "[[scene]]\napp = \"title::TitleProc\"\n[[scene.beat]]\nid = \"title\"\n",
        "title = { GAME() }. { PICTURE(\"push_start\") }, { PICTURE(\"gf_logo\") }.\n",
        &[],
        &[(("game", 0), "Pokémon Omega Ruby")],
        None,
        &quiet(),
        None,
    )?;
    let mut said: Vec<String> = Vec::new();
    Scenes::with_monotonic(scenes, said_of(&words)).started(
        "title::TitleProc",
        None,
        &mut |text: &str| said.push(text.to_string()),
    )?;
    assert_eq!(said, ["Pokémon Omega Ruby. Press Start, gf logo."]);
    Ok(())
}

/// ADR-0038: a paused emulator holds a scene's beats, and the wall clock stands in only while the
/// game's count is not to be had or leaps (spike 236).
#[test]
fn test_a_beat_is_timed_by_the_games_frames_where_they_can_be_read() -> Result<()> {
    let count: Rc<Cell<Option<i64>>> = Rc::new(Cell::new(Some(1000)));
    let wall = Rc::new(Cell::new(0.0));
    let (counted, walled) = (count.clone(), wall.clone());
    let clock = GameClock::new(
        Rc::new(move || Ok(counted.get())),
        Rc::new(move || walled.get()),
    );
    let start = clock.call()?;
    let at = |seconds: f64, frames: Option<i64>| -> Result<f64> {
        wall.set(seconds);
        count.set(frames);
        Ok(clock.call()? - start)
    };
    assert_eq!(at(10.0, Some(1060))?, 1.0);
    assert_eq!(at(20.0, Some(1060))?, 1.0);
    assert_eq!(at(21.0, None)?, 2.0);
    assert_eq!(at(22.0, Some(1120))?, 3.0);
    assert_eq!(at(23.0, Some(900_000))?, 4.0);
    assert_eq!(at(24.0, Some(899_000))?, 5.0);
    assert_eq!(at(30.0, Some(899_120))?, 7.0);
    Ok(())
}

/// Sun numbers its demos, 55 Kukui's call and 6 the chase after it (Khai, sun_1.ctm,
/// 2026-09-30): the chase is described from Sun's own file as it starts, a beat as each second
/// comes, and the arrival after it once the field has come, in Route 1's zone 0x0.
#[test]
fn test_a_scene_of_suns_is_said_over_the_demo_it_plays_as() -> Result<()> {
    let (log, logged) = logging();
    use_words(ENGLISH, &field::names())?;
    let words = nothing_described()?;
    let scenes = load(
        shipped(&field::names()),
        &|_: &str, _: i64, _: i64| Ok(None),
        &|message: &str| words.known(message),
        &log,
        None,
    )?;
    assert!(logged.borrow().is_empty());
    let said: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let now = Rc::new(Cell::new(100.0));
    let clock = now.clone();
    let mut reader = Scenes::new(scenes, said_of(&words), Rc::new(move || Ok(clock.get())));
    let mut say = |text: &str| said.borrow_mut().push(text.to_string());
    reader.demo(Some(55), &mut say)?;
    reader.demo(None, &mut say)?;
    assert!(said.borrow().is_empty());
    reader.demo(Some(6), &mut say)?;
    assert!(said.borrow().len() == 1 && said.borrow()[0].starts_with("After the call"));
    now.set(101.0);
    reader.poll(&mut say)?;
    assert_eq!(said.borrow().len(), 1);
    now.set(102.0);
    reader.poll(&mut say)?;
    assert!(
        said.borrow()
            .last()
            .is_some_and(|last| last.starts_with("Somewhere else, a girl"))
    );
    now.set(500.0);
    reader.poll(&mut say)?;
    assert_eq!(said.borrow().len(), 10);
    reader.arrived(0x0, &mut say)?;
    now.set(501.0);
    reader.poll(&mut say)?;
    assert!(said.borrow().len() == 11 && said.borrow()[10].starts_with("On the ground"));
    now.set(506.0);
    reader.poll(&mut say)?;
    assert_eq!(said.borrow().len(), 12);
    Ok(())
}

/// The timing and the words are two files: each beat's id in one is a message in the other, and
/// no message is left over (ADR-0040).
#[test]
fn test_every_beat_of_every_scene_shipped_has_english_words() {
    let sun = field::names();
    for game in [
        sun.parent().unwrap().join("oras"),
        sun.parent().unwrap().join("xy"),
        sun.clone(),
    ] {
        let timing: ::toml::Table = fs::read_to_string(game.join(FILE))
            .unwrap()
            .parse()
            .unwrap();
        let beats: Vec<String> = timing["scene"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|scene| scene["beat"].as_array().unwrap().iter())
            .map(|beat| beat["id"].as_str().unwrap().to_string())
            .collect();
        let source = fs::read_to_string(game.join("en").join("scenes.ftl")).unwrap();
        let body = match fluent_syntax::parser::parse(source.as_str()) {
            Ok(resource) => resource.body,
            Err((resource, _)) => resource.body,
        };
        let mut messages: HashSet<String> = body
            .iter()
            .filter_map(|entry| match entry {
                fluent_syntax::ast::Entry::Message(message) => Some(message.id.name.to_string()),
                _ => None,
            })
            .collect();
        messages.remove("scene-yours");
        let unique: HashSet<String> = beats.iter().cloned().collect();
        assert_eq!(beats.len(), unique.len(), "{}", game.display());
        assert_eq!(unique, messages, "{}", game.display());
    }
}

/// Orion, 2026-10-12: "Narration:" before a description that follows someone speaking, and not
/// before one that follows another; a speaker after it is named again.
#[test]
fn test_a_description_is_the_narrators_named_as_a_speaker_is() -> Result<()> {
    let backend = Recorder::new();
    let names: IndexMap<i64, Option<&str>> = IndexMap::from([(1, Some("Maxie")), (2, None)]);
    let speaker: Speaker = Rc::new(move |window| {
        Ok(names
            .get(&window)
            .copied()
            .flatten()
            .map(|name| name.to_string()))
    });
    let mut announcements = Announcements::with(
        Rc::new(Speech::plain(backend.clone())),
        Rc::new(|_: &str| {}),
        Some(speaker),
        false,
    );
    let shown = |window: i64, text: &str| vec![Shown::Page(PageShown::new(window, 0, text))];
    announcements.pages(&shown(1, "Behold!"))?;
    announcements.narrate("Groudon rises.");
    announcements.narrate("It roars.");
    announcements.pages(&shown(1, "Splendid."))?;
    announcements.narrate("The sky burns.");
    announcements.pages(&shown(2, "Groudon appeared!"))?;
    announcements.narrate("It towers over you.");
    assert_eq!(
        backend.spoken(),
        [
            "Maxie: Behold!",
            "Narration: Groudon rises.",
            "It roars.",
            "Maxie: Splendid.",
            "Narration: The sky burns.",
            "Groudon appeared!",
            "Narration: It towers over you."
        ]
    );
    Ok(())
}

#[test]
fn test_a_description_among_a_screens_words_names_the_narrator_every_time() {
    // ADR-0038: Tile Puzzle's picture came after "Easy", the title screen's beat the last
    // narrated, and plain it was heard as one more of the screen's lines.
    let backend = Recorder::new();
    let mut announcements = Announcements::with(
        Rc::new(Speech::plain(backend.clone())),
        Rc::new(|_: &str| {}),
        Some(Rc::new(|_| Ok(None))),
        false,
    );
    announcements.narrate("The title screen.");
    announcements.narrate_named("Metagross on the left.", true);
    assert_eq!(
        backend.spoken(),
        [
            "Narration: The title screen.",
            "Narration: Metagross on the left."
        ]
    );
}
