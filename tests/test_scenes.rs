//! Descriptions of scenes: found by the line they follow, said a beat at a time (spike 203).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;
use regex::Regex;

use super::fake_speech::Recorder;
use crate::core::speech::Speech;
use crate::games::common::announce::{Announcements, Speaker};
use crate::games::common::dialogue::{PageShown, Shown};
use crate::games::common::scenes::{Scenes, last_page, load};
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

/// The log a test keeps, and what it logged.
fn logging() -> (Log, Rc<RefCell<Vec<String>>>) {
    let logged: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let kept = logged.clone();
    (
        Rc::new(move |line: &str| kept.borrow_mut().push(line.to_string())),
        logged,
    )
}

/// Groudon woken in Seafloor Cavern (movie40): Archie's "Augh... You..." in zone 0x6f, then the
/// demo, then ten seconds on; and Tabitha's line, then the arrival on Route 128.
#[test]
fn test_a_scene_is_said_a_beat_at_a_time() -> Result<()> {
    let file = r#"
[[scene]]
after = "story:402:30"
zone = 0x6f
[[scene.beat]]
text = "{trainer:235} walks to the lava."
[[scene.beat]]
demo = true
text = "The Orb glows."
[[scene.beat]]
at = 10
text = "{pokemon:383} wakes."

[[scene]]
after = "story:402:42"
zone = 0x6f
[[scene.beat]]
zone = 0x39
text = "The sky glares."

[[scene]]
after = "story:402:43"
zone = 0x6f
[[scene.beat]]
text = "{item:9999} is nowhere."
"#;
    let lines: IndexMap<(i64, i64), Vec<u8>> = IndexMap::from([
        ((402, 30), line(&[Part::Text("Augh... You...")])),
        ((402, 42), line(&[Part::Text("Child! Hurry up!")])),
        ((402, 43), line(&[Part::Text("Missing.")])),
    ]);
    let words: IndexMap<(&str, i64), &str> =
        IndexMap::from([(("trainer", 235), "Maxie"), (("pokemon", 383), "Groudon")]);
    let (log, logged) = logging();
    let scenes = load(
        Some(file),
        &|archive: &str, file: i64, number: i64| {
            Ok(if archive == "story" {
                lines.get(&(file, number)).cloned()
            } else {
                None
            })
        },
        &|kind: &str, number: i64| words.get(&(kind, number)).map(|word| word.to_string()),
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
    let mut reader = Scenes::new(scenes, Rc::new(move || clock.get()));
    let mut say = |text: &str| said.borrow_mut().push(text.to_string());
    let last = || said.borrow().last().cloned().unwrap_or_default();
    reader.heard(0, "Augh... You...", Some(0x6E), &mut say);
    assert!(said.borrow().is_empty());
    reader.heard(0, "Augh... You...", Some(0x6F), &mut say);
    assert_eq!(*said.borrow(), ["Maxie walks to the lava."]);
    reader.poll(&mut say);
    assert_eq!(*said.borrow(), ["Maxie walks to the lava."]);
    now.set(30.0);
    reader.demo(None, &mut say);
    assert_eq!(last(), "The Orb glows.");
    now.set(39.0);
    reader.poll(&mut say);
    assert_eq!(last(), "The Orb glows.");
    now.set(40.0);
    reader.poll(&mut say);
    assert_eq!(last(), "Groudon wakes.");
    reader.heard(0, "Child! Hurry up!", Some(0x6F), &mut say);
    assert_eq!(last(), "Groudon wakes.");
    reader.arrived(0x39, &mut say);
    assert_eq!(last(), "The sky glares.");
    // Waiting for Route 128 and arriving somewhere else lets the scene go.
    reader.heard(0, "Child! Hurry up!", Some(0x6F), &mut say);
    reader.arrived(0x6D, &mut say);
    reader.arrived(0x39, &mut say);
    assert_eq!(
        said.borrow()
            .iter()
            .filter(|text| *text == "The sky glares.")
            .count(),
        1
    );
    Ok(())
}

/// Steven's "Good luck to you." on Route 128 is one line in one zone in both versions, and he
/// flies off into Omega Ruby's glare (movie40) but Alpha Sapphire's rain (Orion's
/// movie1_kyogre_awakens, 2026-10-14): a scene with a title is the running title's alone.
#[test]
fn test_a_scene_of_one_version_is_left_out_of_the_other() -> Result<()> {
    let file = r#"
[[scene]]
after = "story:148:46"
zone = 0x39
title = 0x000400000011C400
[[scene.beat]]
text = "Steven flies off into the glare."

[[scene]]
after = "story:148:46"
zone = 0x39
[[scene.beat]]
text = "Steven flies off."
"#;
    let lines: IndexMap<(i64, i64), Vec<u8>> =
        IndexMap::from([((148, 46), line(&[Part::Text("Good luck to you.")]))]);
    let said_in = |title: Option<i64>| -> Result<Vec<String>> {
        let (log, logged) = logging();
        let scenes = load(
            Some(file),
            &|archive: &str, file: i64, number: i64| {
                Ok(if archive == "story" {
                    lines.get(&(file, number)).cloned()
                } else {
                    None
                })
            },
            &|_: &str, _: i64| None,
            &log,
            title,
        )?;
        assert!(logged.borrow().is_empty());
        let mut said: Vec<String> = Vec::new();
        let mut reader = Scenes::with_monotonic(scenes);
        reader.heard(0, "Good luck to you.", Some(0x39), &mut |text: &str| {
            said.push(text.to_string())
        });
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
    let file = r#"
[[scene]]
zone = 0x76
[[scene.beat]]
text = "Magma glows to the north."
"#;
    let mut said: Vec<String> = Vec::new();
    let (log, _) = logging();
    let scenes = load(
        Some(file),
        &|_: &str, _: i64, _: i64| Ok(None),
        &|_: &str, _: i64| None,
        &log,
        None,
    )?;
    let mut reader = Scenes::with_monotonic(scenes);
    let mut say = |text: &str| said.push(text.to_string());
    reader.arrived(0x74, &mut say);
    reader.arrived(0x76, &mut say);
    reader.arrived(0x74, &mut say);
    reader.arrived(0x76, &mut say);
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
    let file = r#"
[[scene]]
after = "text:118:60"
[[scene.beat]]
text = "The doors swing wide."
[[scene.beat]]
zone = 0x18f
text = "You stand in the truck."
"#;
    let lines: IndexMap<(i64, i64), Vec<u8>> = IndexMap::from([(
        (118, 60),
        line(&[Part::Text("Come see me in my Pokémon Lab!")]),
    )]);
    let (log, logged) = logging();
    let scenes = load(
        Some(file),
        &|archive: &str, file: i64, number: i64| {
            Ok(if archive == "text" {
                lines.get(&(file, number)).cloned()
            } else {
                None
            })
        },
        &|_: &str, _: i64| None,
        &log,
        None,
    )?;
    assert!(logged.borrow().is_empty());
    let said: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let mut reader = Scenes::with_monotonic(scenes);
    let mut say = |text: &str| said.borrow_mut().push(text.to_string());
    reader.heard(0, "Come see me in my Pokémon Lab!", Some(0x10), &mut say);
    assert!(said.borrow().is_empty());
    reader.heard(0, "Come see me in my Pokémon Lab!", None, &mut say);
    reader.arrived(0x18F, &mut say);
    assert_eq!(
        *said.borrow(),
        ["The doors swing wide.", "You stand in the truck."]
    );
    Ok(())
}

/// Sun numbers its demos, 55 Kukui's call and 6 the chase after it (Khai, sun_1.ctm,
/// 2026-09-30): the chase is described from Sun's own file as it starts, a beat as each second
/// comes, and the arrival after it once the field has come, in Route 1's zone 0x0.
#[test]
fn test_a_scene_of_suns_is_said_over_the_demo_it_plays_as() -> Result<()> {
    let (log, logged) = logging();
    let scenes = load(
        field::in_language(field::SCENES, "en"),
        &|_: &str, _: i64, _: i64| Ok(None),
        &|_: &str, _: i64| None,
        &log,
        None,
    )?;
    assert!(logged.borrow().is_empty());
    let said: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let now = Rc::new(Cell::new(100.0));
    let clock = now.clone();
    let mut reader = Scenes::new(scenes, Rc::new(move || clock.get()));
    let mut say = |text: &str| said.borrow_mut().push(text.to_string());
    reader.demo(Some(55), &mut say);
    reader.demo(None, &mut say);
    assert!(said.borrow().is_empty());
    reader.demo(Some(6), &mut say);
    assert!(said.borrow().len() == 1 && said.borrow()[0].starts_with("After the call"));
    now.set(101.0);
    reader.poll(&mut say);
    assert_eq!(said.borrow().len(), 1);
    now.set(102.0);
    reader.poll(&mut say);
    assert!(
        said.borrow()
            .last()
            .is_some_and(|last| last.starts_with("Somewhere else, a girl"))
    );
    now.set(500.0);
    reader.poll(&mut say);
    assert_eq!(said.borrow().len(), 10);
    reader.arrived(0x0, &mut say);
    now.set(501.0);
    reader.poll(&mut say);
    assert!(said.borrow().len() == 11 && said.borrow()[10].starts_with("On the ground"));
    now.set(506.0);
    reader.poll(&mut say);
    assert_eq!(said.borrow().len(), 12);
    Ok(())
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
