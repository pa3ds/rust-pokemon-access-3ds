//! Who is speaking a field message, and how a conversation reads when pa3ds knows their names.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use super::fake_speech::Recorder;
use crate::core::names::{Names, sprite_key};
use crate::core::speech::Speech;
use crate::core::words::{ENGLISH, Words};
use crate::games::common::announce::Announcements;
use crate::games::common::dialogue::{PageShown, Shown};
use crate::games::common::field::things::{MODEL_SOURCE, own_name};
use crate::games::gen6::field::speaker::SpeakerReader;
use crate::games::gen6::game::Game;
use crate::games::{gen6::field::mmodel, gen6::field::talk};
use crate::games::{oras, xy};

const WINDOW: i64 = 0x08286988;
const OTHER_WINDOW: i64 = 0x082878B8;
/// What a sweep hands over: the Gra embedded in the talk object (spike 73).
const TALK: i64 = 0x0828613C;
const NORMAN_MODEL: i64 = 0x08D1E9A8;
const WALLY_MODEL: i64 = 0x08D1FF08;
const NORMAN: i64 = 0x13F;
const WALLY: i64 = 0xB4;
const GYM: i64 = 0x1AF;

/// `NAMES`: the shipped table the tests build a `Names` from.
fn shipped_names() -> IndexMap<String, String> {
    IndexMap::from([
        ("0x13f".to_string(), "Norman".to_string()),
        ("0xb4".to_string(), "Wally".to_string()),
    ])
}

/// Lay out a talk object as the field leaves it while a message is up.
fn talking(memory: &FakeMemory, at: i64, window: i64, actor: i64) {
    memory.u32(at + talk::WINDOW, &[window]);
    memory.u32(at + talk::ACTOR, &[actor]);
}

/// Lay out a slot of the entity list, as a zone load fills it.
fn standing(memory: &FakeMemory, at: i64, character: i64, slot: i64) {
    memory.u32(at + mmodel::IN_USE, &[1]);
    memory.u32(at + mmodel::ZONE, &[GYM << 16 | slot]);
    memory.u32(at + mmodel::CHARACTER, &[character]);
}

fn reader(memory: &Rc<FakeMemory>) -> SpeakerReader {
    SpeakerReader::with_default_drawn(
        memory.clone(),
        Rc::new(Names::new(Some(&shipped_names()), None, None, None, None).unwrap()),
    )
}

/// `Names.load("sango-1", words=Words(language, game.DATA))` with no home: the player has named
/// nobody.
fn shipped(game: &Game, language: &str) -> Names {
    let words = Words::new(
        language,
        Some(&(game.data)()),
        IndexMap::new(),
        Rc::new(|_line: &str| {}),
    )
    .unwrap();
    Names::load("sango-1", None, "", Some(&words)).unwrap()
}

#[test]
fn test_the_talk_object_says_which_character_is_speaking() {
    let memory = FakeMemory::new();
    talking(&memory, TALK, WINDOW, NORMAN_MODEL);
    standing(&memory, NORMAN_MODEL, NORMAN, 1);
    let speaker = reader(&memory);
    speaker.track([TALK]);
    assert_eq!(speaker.character_of(WINDOW).unwrap(), Some(NORMAN));
    assert_eq!(speaker.name_for(WINDOW).unwrap().as_deref(), Some("Norman"));
}

/// The rival stands in work object 0x7a in both openings and is drawn as May or Brendan
/// (spike 89): the speaker is who is drawn, so a name given to May is not given to the slot.
#[test]
fn test_a_work_object_speaks_as_whoever_it_was_filled_with() {
    let memory = FakeMemory::new();
    talking(&memory, TALK, WINDOW, NORMAN_MODEL);
    standing(&memory, NORMAN_MODEL, 0x7A, 3);
    memory.u32(NORMAN_MODEL + mmodel::DRAWN, &[0x13A]);
    let named = IndexMap::from([
        ("0x13a".to_string(), "May".to_string()),
        ("0x7a".to_string(), "a slot".to_string()),
    ]);
    let speaker = SpeakerReader::with_default_drawn(
        memory.clone(),
        Rc::new(Names::new(Some(&named), None, None, None, None).unwrap()),
    );
    speaker.track([TALK]);
    assert_eq!(speaker.character_of(WINDOW).unwrap(), Some(0x13A));
    assert_eq!(speaker.name_for(WINDOW).unwrap().as_deref(), Some("May"));
    assert_eq!(speaker.last(), Some((0x13A, own_name(GYM, 3).unwrap())));
}

/// A Youngster named "Calvin" with the box ticked is Calvin in what he says too, where every
/// other Youngster speaks as the sprite's name (Orion, 2026-10-11).
#[test]
fn test_somebody_named_alone_speaks_as_that_name() {
    let memory = FakeMemory::new();
    talking(&memory, TALK, WINDOW, NORMAN_MODEL);
    standing(&memory, NORMAN_MODEL, 0xDA, 7);
    let shipped = IndexMap::from([("0xda".to_string(), "Youngster".to_string())]);
    let names = Rc::new(Names::new(Some(&shipped), None, None, None, None).unwrap());
    let speaker = SpeakerReader::with_default_drawn(memory.clone(), names.clone());
    speaker.track([TALK]);
    assert_eq!(
        speaker.name_for(WINDOW).unwrap().as_deref(),
        Some("Youngster")
    );
    names.name_place(GYM, MODEL_SOURCE | 7, "Calvin").unwrap();
    assert_eq!(speaker.name_for(WINDOW).unwrap().as_deref(), Some("Calvin"));
    standing(&memory, NORMAN_MODEL, 0xDA, 8);
    assert_eq!(
        speaker.name_for(WINDOW).unwrap().as_deref(),
        Some("Youngster")
    );
}

#[test]
fn test_a_window_no_talk_object_is_showing_has_no_speaker() {
    let memory = FakeMemory::new();
    talking(&memory, TALK, WINDOW, NORMAN_MODEL);
    standing(&memory, NORMAN_MODEL, NORMAN, 1);
    let speaker = reader(&memory);
    speaker.track([TALK]);
    assert_eq!(speaker.character_of(OTHER_WINDOW).unwrap(), None);
    assert_eq!(speaker.name_for(OTHER_WINDOW).unwrap(), None);
}

/// The field writes the actor as a message begins and clears it when it ends (spike 73).
#[test]
fn test_between_messages_the_actor_is_cleared_and_nobody_is_speaking() {
    let memory = FakeMemory::new();
    talking(&memory, TALK, WINDOW, NORMAN_MODEL);
    standing(&memory, NORMAN_MODEL, NORMAN, 1);
    let speaker = reader(&memory);
    speaker.track([TALK]);
    talking(&memory, TALK, WINDOW, 0);
    assert_eq!(speaker.character_of(WINDOW).unwrap(), None);
}

/// Its class is a DLL's while a talk runs, so a sweep cannot find it again when it matters.
#[test]
fn test_a_talk_object_is_kept_once_found() {
    let memory = FakeMemory::new();
    let speaker = reader(&memory);
    speaker.track([TALK]);
    speaker.track([]);
    talking(&memory, TALK, WINDOW, WALLY_MODEL);
    standing(&memory, WALLY_MODEL, WALLY, 2);
    assert_eq!(speaker.name_for(WINDOW).unwrap().as_deref(), Some("Wally"));
}

#[test]
fn test_a_character_nobody_has_named_is_left_unsaid() {
    let memory = FakeMemory::new();
    talking(&memory, TALK, WINDOW, NORMAN_MODEL);
    standing(&memory, NORMAN_MODEL, 0xDD, 9);
    let speaker = reader(&memory);
    speaker.track([TALK]);
    assert_eq!(speaker.character_of(WINDOW).unwrap(), Some(0xDD));
    assert_eq!(speaker.name_for(WINDOW).unwrap(), None);
}

/// An `Announcements` that asks `speaking` who is showing a window, and what it was made to say.
fn announcing(
    speaking: &Rc<RefCell<HashMap<i64, Option<String>>>>,
) -> (Announcements, Rc<Recorder>) {
    let heard = Recorder::new();
    let speaking = speaking.clone();
    let announcements = Announcements::with(
        Rc::new(Speech::plain(heard.clone())),
        Rc::new(|_: &str| {}),
        Some(Rc::new(move |window| {
            Ok(speaking.borrow().get(&window).cloned().flatten())
        })),
        false,
    );
    (announcements, heard)
}

fn said(pages: Vec<PageShown>, speakers: HashMap<i64, Option<String>>) -> Vec<String> {
    let (mut announcements, heard) = announcing(&Rc::new(RefCell::new(speakers)));
    for page in pages {
        announcements.pages(&[Shown::Page(page)]).unwrap();
    }
    heard.spoken()
}

fn page(text: &str) -> PageShown {
    PageShown::new(WINDOW, 0, text)
}

fn say(announcements: &mut Announcements, text: &str) {
    announcements.pages(&[Shown::Page(page(text))]).unwrap();
}

/// Norman and the trainer talk back and forth through one window (wally.ctm, spike 73). A name
/// a line would be noise; a name a speaker is the news.
#[test]
fn test_a_speaker_is_named_when_they_take_over_and_not_before_every_line() {
    let speaking = Rc::new(RefCell::new(HashMap::from([(
        WINDOW,
        Some("Norman".to_string()),
    )])));
    let (mut announcements, heard) = announcing(&speaking);
    say(&mut announcements, "Not bad... It was a good battle.");
    say(
        &mut announcements,
        "Come back and challenge me again anytime.",
    );
    speaking
        .borrow_mut()
        .insert(WINDOW, Some("Wally".to_string()));
    say(&mut announcements, "Th-thank you, sir!");
    speaking
        .borrow_mut()
        .insert(WINDOW, Some("Norman".to_string()));
    say(&mut announcements, "Hm?");
    assert_eq!(
        heard.spoken(),
        [
            "Norman: Not bad... It was a good battle.",
            "Come back and challenge me again anytime.",
            "Wally: Th-thank you, sir!",
            "Norman: Hm?",
        ]
    );
}

#[test]
fn test_dialogue_with_nobody_named_reads_as_it_always_did() {
    assert_eq!(
        said(vec![page("The water is deep, deep blue.")], HashMap::new()),
        ["The water is deep, deep blue."]
    );
}

/// Walking away from a conversation and back should not swallow the name.
#[test]
fn test_a_named_speaker_after_an_unnamed_one_is_named_again() {
    let speaking = Rc::new(RefCell::new(HashMap::from([(
        WINDOW,
        Some("Norman".to_string()),
    )])));
    let (mut announcements, heard) = announcing(&speaking);
    say(&mut announcements, "Hm?");
    speaking.borrow_mut().insert(WINDOW, None);
    say(&mut announcements, "The water is deep, deep blue.");
    speaking
        .borrow_mut()
        .insert(WINDOW, Some("Norman".to_string()));
    say(&mut announcements, "Why, if it isn't wee!");
    assert_eq!(
        heard.spoken(),
        [
            "Norman: Hm?",
            "The water is deep, deep blue.",
            "Norman: Why, if it isn't wee!",
        ]
    );
}

/// The file beside the game's code, read the way the runtime reads it.
#[test]
fn test_the_shipped_table_is_the_one_that_ships() {
    let names = shipped(oras::game(), ENGLISH);
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Norman"));
    assert_eq!(names.sprite(WALLY).as_deref(), Some("Wally"));
    // The rival is whoever work object 0x7a is filled with, May's sprite or Brendan's (spike 89).
    // What ships for them is heard only without a ROM, and the slot itself is nobody.
    assert_eq!(names.sprite(0x13A).as_deref(), Some("May"));
    assert_eq!(names.sprite(0x13B).as_deref(), Some("Brendan"));
    assert_eq!(names.sprite(0x7A), None);
    // A kind of trainer, copied from the ROM's English for when it cannot be read (spike 90).
    assert_eq!(names.sprite(0xDA).as_deref(), Some("Youngster"));
    assert_eq!(names.sprite(0xC7).as_deref(), Some("Team Magma Grunt"));
    assert_eq!(names.sprite(0x12A).as_deref(), Some("Nurse"));
    assert_eq!(names.sprite(0x118).as_deref(), Some("Mom"));
    assert_eq!(names.sprite(0x13E).as_deref(), Some("Professor Birch"));
    // WOMAN_RS is Mrs. Birch in Littleroot but a woman in 27 other zones, Slateport among them.
    assert_eq!(names.sprite(0x128).as_deref(), Some("Woman"));
}

fn keys(table: &IndexMap<String, String>) -> HashSet<String> {
    table.keys().cloned().collect()
}

/// The sprites one language's `sprites.ftl` names, by key, without English behind it.
fn sprites(game: &Game, language: &str) -> IndexMap<String, String> {
    let path = (game.data)().join(language).join("sprites.ftl");
    let resource = fluent_bundle::FluentResource::try_new(std::fs::read_to_string(path).unwrap())
        .unwrap_or_else(|(resource, _)| resource);
    let id: unic_langid::LanguageIdentifier = language.parse().unwrap();
    let mut bundle = fluent_bundle::FluentBundle::<&fluent_bundle::FluentResource>::new(vec![id]);
    bundle.set_use_isolating(false);
    // A message the file names twice keeps its first words, as Python's `add_resource` does.
    let _ = bundle.add_resource(&resource);
    let mut table = IndexMap::new();
    for entry in resource.entries() {
        let fluent_syntax::ast::Entry::Message(message) = entry else {
            continue;
        };
        let Some(key) = message.id.name.strip_prefix("sprite-") else {
            continue;
        };
        let Some(pattern) = bundle
            .get_message(message.id.name)
            .and_then(|found| found.value())
        else {
            continue;
        };
        let mut errors = Vec::new();
        let said = bundle.format_pattern(pattern, None, &mut errors);
        table.insert(key.to_string(), said.to_string());
    }
    table
}

/// The user, 2026-09-23: a fallback in every language the game carries, each the ROM's own
/// words in that language (spikes 90 and 91). Since 2026-10-14 each names the same sprites as
/// the English (Orion): where the ROM has no word, as for the Nurse, it is pa3ds's own.
#[test]
fn test_every_language_ships_a_table_of_the_same_people() {
    let english = keys(&sprites(oras::game(), ENGLISH));
    for language in ["ja", "fr", "it", "de", "es", "ko"] {
        let table = sprites(oras::game(), language);
        assert_eq!(keys(&table), english, "{language}");
        assert!(table.values().all(|name| !name.is_empty()), "{language}");
    }
    assert_eq!(
        shipped(oras::game(), "fr").sprite(0x13A).as_deref(),
        Some("Flora")
    );
    assert_eq!(
        shipped(oras::game(), "ja").sprite(0x118).as_deref(),
        Some("ママ")
    );
    assert_eq!(
        shipped(oras::game(), "de").sprite(0x204E).as_deref(),
        Some("Relaxo-Puppe")
    );
}

/// Spike 108: X's fallback names are the ROM's own words in each of its seven languages, the
/// story's people among them. The Nurse is pa3ds's own word in every language, and so is the
/// Spanish Mom, whom the Spanish text calls only "tu madre".
#[test]
fn test_x_ships_a_table_in_every_language_it_carries() {
    let english = keys(&sprites(xy::game(), ENGLISH));
    let characters: HashSet<String> = xy::characters()
        .keys()
        .map(|&sprite| sprite_key(sprite))
        .collect();
    assert!(characters.is_subset(&english));
    for language in ["ja", "fr", "it", "de", "es", "ko"] {
        let table = sprites(xy::game(), language);
        assert_eq!(keys(&table), english, "{language}");
        assert!(table.values().all(|name| !name.is_empty()), "{language}");
    }
    assert_eq!(
        shipped(xy::game(), "en").sprite(0x0F).as_deref(),
        Some("Viola")
    );
    assert_eq!(
        shipped(xy::game(), "en").sprite(0x3B).as_deref(),
        Some("Nurse")
    );
    assert_eq!(
        shipped(xy::game(), "fr").sprite(0x93).as_deref(),
        Some("Kalem")
    );
    assert_eq!(
        shipped(xy::game(), "ja").sprite(0x07).as_deref(),
        Some("ママ")
    );
}

/// A page no talk object kept shows asks for a sweep: a door builds the next zone's talk
/// windows where no sweep of new blocks looks (movie1, 2026-09-26). A sign's talk has nobody.
#[test]
fn test_a_window_is_shown_by_a_talk_object_kept_whether_or_not_anyone_speaks() {
    let memory = FakeMemory::new();
    talking(&memory, TALK, WINDOW, 0);
    let speaker = reader(&memory);
    assert!(!speaker.shows(WINDOW).unwrap());
    speaker.track([TALK]);
    assert!(speaker.shows(WINDOW).unwrap());
    assert!(!speaker.shows(OTHER_WINDOW).unwrap());
}
