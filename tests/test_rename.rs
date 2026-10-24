//! Naming what the game never names, while playing: the key, the dialog and what sticks.
//!
//! The dialog itself cannot be shown in a test — it is a modal window with its own message loop —
//! so `Asker` is given a stand-in that answers at once. What is tested is everything around it: what
//! is being renamed, which of the two name spaces it belongs to, what the box is labelled with, what
//! is said back, and whether the name is there the next time pa3ds starts.

use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use indexmap::IndexMap;

use super::fake_speech::Recorder;
use crate::core::names::Names;
use crate::core::speech::Speech;
use crate::games::common::field::ground::Ground;
use crate::games::common::field::guide::FieldSource;
use crate::games::common::field::keys::{
    CATEGORY_NEXT, ITEM_NEXT, ITEM_READ, RENAME, RENAME_SPEAKER,
};
use crate::games::common::field::navigate::{LeadsTo, Navigator, Options};
use crate::games::common::field::things::{
    Field, Interactable, MODEL_SOURCE, Naming, PLACE, POKEMON, SPRITE, WARP_SOURCE,
};
use crate::games::oras::field::hidden::HIDDEN_POKEMON;
use crate::py::{self, Result};
use crate::ux::dialog::{
    ATOM_BUTTON, ATOM_EDIT, ATOM_STATIC, Ask, Asker, IDC_CHECK, IDC_EDIT, IDC_LABEL, IDCANCEL,
    IDOK, template,
};

const NORMAN: i64 = 0x13F;
const MART: i64 = 0x1AF;

/// pytest's `tmp_path`: a folder of this test's own, gone when the test is.
struct TmpPath(PathBuf);

impl TmpPath {
    fn new() -> Self {
        static MADE: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "pa3ds-test-rename-{}-{}",
            std::process::id(),
            MADE.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path).unwrap();
        TmpPath(path)
    }
}

impl std::ops::Deref for TmpPath {
    type Target = std::path::Path;

    fn deref(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TmpPath {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A warp's slot: its index with the bit of its own that says it is one.
fn warp(index: i64) -> i64 {
    WARP_SOURCE | index
}

/// One of each kind that matters: a warp and a sign from the zone's table, keyed by their slot,
/// and a person who is a move model, keyed by the character they are.
fn town() -> Field {
    Field {
        interactables: vec![
            Interactable {
                zone: Some(0xDF),
                source: warp(0),
                ..Interactable::new("exits", "door", (94, 171))
            },
            Interactable {
                source: 1,
                ..Interactable::new("signs", "sign", (97, 171))
            },
            Interactable {
                code: NORMAN,
                source: 0x08D1E9A8,
                move_model: true,
                ..Interactable::new("people", "person", (104, 179))
            },
        ],
        ..Field::new(MART, (94, 172))
    }
}

/// A DexNav find, as `DexNav.listed` builds one: its `source` is the record's address, because
/// there is no table it holds a slot in.
fn hiding() -> Interactable {
    Interactable {
        code: 0x105,
        source: 0x08D1E9A8,
        ..Interactable::new(POKEMON, HIDDEN_POKEMON, (96, 170))
    }
}

struct FakeField {
    r#where: Field,
}

impl FieldSource for FakeField {
    // Python's `FakeField` has no `visit` or `ground`, which no test here reaches.
    fn visit(&self) -> Option<(i64, i64)> {
        None
    }

    fn poll(&self) -> Result<Option<Field>> {
        Ok(Some(Field::new(self.r#where.zone, self.r#where.cell)))
    }

    fn interactables(&self, r#where: &Field) -> Result<Field> {
        Ok(Field {
            interactables: self.r#where.interactables.clone(),
            ..Field::new(r#where.zone, r#where.cell)
        })
    }

    fn ground(&self, _where: &Field, _height: Option<f64>) -> Result<Ground> {
        unimplemented!("FakeField has no ground")
    }
}

type Asked = (String, String, String, String);

/// Stands in for the box: records what it was asked and answers with what it was told to.
struct FakeDialog {
    answer: Mutex<Option<String>>,
    /// Whether the checkbox is left ticked, where the box has one: it opens ticked.
    ticked: bool,
    asked: Mutex<Vec<Asked>>,
}

impl FakeDialog {
    fn new(answer: Option<&str>, ticked: bool) -> Arc<Self> {
        Arc::new(FakeDialog {
            answer: Mutex::new(answer.map(str::to_string)),
            ticked,
            asked: Mutex::new(Vec::new()),
        })
    }

    fn call(
        &self,
        title: &str,
        label: &str,
        current: &str,
        check: &str,
    ) -> Result<Option<(String, bool)>> {
        self.asked
            .lock()
            .unwrap()
            .push(box_with(title, label, current, check));
        Ok(self
            .answer
            .lock()
            .unwrap()
            .clone()
            .map(|answer| (answer, !check.is_empty() && self.ticked)))
    }

    fn asked(&self) -> Vec<Asked> {
        self.asked.lock().unwrap().clone()
    }

    fn shown(self: &Arc<Self>) -> Ask {
        let dialog = self.clone();
        Arc::new(move |title, label, current, check| dialog.call(title, label, current, check))
    }
}

/// One of `FakeDialog.asked`'s entries, for a box with no checkbox.
fn box_of(title: &str, label: &str, current: &str) -> Asked {
    box_with(title, label, current, "")
}

/// One of `FakeDialog.asked`'s entries.
fn box_with(title: &str, label: &str, current: &str, check: &str) -> Asked {
    (
        title.to_string(),
        label.to_string(),
        current.to_string(),
        check.to_string(),
    )
}

/// `navigator`'s arguments, each as the Python defaults it.
struct Wanted {
    answer: Option<String>,
    names: Option<Rc<Names>>,
    r#where: Field,
    /// The last speaker's character.
    spoke: Option<i64>,
    leads_to: Option<LeadsTo>,
    /// The speaker's name alone.
    own: Option<Naming>,
    ticked: bool,
}

impl Default for Wanted {
    fn default() -> Self {
        Wanted {
            answer: Some("the Mart's sign".to_string()),
            names: None,
            r#where: town(),
            spoke: None,
            leads_to: None,
            own: None,
            ticked: true,
        }
    }
}

fn navigator(wanted: Wanted) -> (Navigator, Rc<Recorder>, Arc<FakeDialog>, Rc<Names>) {
    let backend = Recorder::new();
    let dialog = FakeDialog::new(wanted.answer.as_deref(), wanted.ticked);
    let named = wanted.names.unwrap_or_else(|| Rc::new(Names::default()));
    let (spoke, own) = (wanted.spoke, wanted.own);
    let nav = Navigator::new(
        Rc::new(FakeField {
            r#where: wanted.r#where,
        }),
        Rc::new(Speech::plain(backend.clone())),
        Rc::new(|_line: &str| {}),
        Options {
            names: Some(named.clone()),
            asker: Some(Asker::new(Some(dialog.shown()))),
            speaker: Some(Rc::new(move || spoke.map(|spoke| (spoke, own)))),
            leads_to: wanted.leads_to,
            ..Options::default()
        },
    );
    (nav, backend, dialog, named)
}

/// Press the key, let the dialog's thread finish, and take the answer at the next poll.
fn rename(nav: &mut Navigator) {
    nav.press(RENAME).unwrap();
    nav.asker.wait_default();
    nav.poll_renames().unwrap();
}

/// `{"sprites": {}, "places": {}}`.
fn nothing_named() -> IndexMap<String, IndexMap<String, String>> {
    IndexMap::from([
        ("sprites".to_string(), IndexMap::new()),
        ("places".to_string(), IndexMap::new()),
    ])
}

fn shipped(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
    pairs
        .iter()
        .map(|&(key, name)| (key.to_string(), name.to_string()))
        .collect()
}

// ---------------------------------------------------------------- which name a thing gets

/// A sign named in Petalburg says nothing about a sign in Rustboro, so a place is keyed by
/// the map it is on and the slot it holds there.
#[test]
fn test_a_sign_is_named_by_its_slot_on_this_map() {
    assert_eq!(town().interactables[1].naming(MART), Some((PLACE, MART, 1)));
}

/// A move model is one character everywhere in the game, and the same number the speaker is
/// read as, so naming the person in the Gym names Norman in the street and in dialogue.
#[test]
fn test_a_person_is_named_by_the_character_they_are() {
    assert_eq!(
        town().interactables[2].naming(MART),
        Some((SPRITE, NORMAN, 0))
    );
}

/// A gen 6 model's source is `zone << 16 | index` over its address, so the index's low four
/// bits sit where a place's kind does, and 0xb is the sky's: such a person was "person" in the
/// list and could not be renamed (movie42, Mauville's Street Thug).
#[test]
fn test_a_person_whose_record_index_ends_in_b_is_still_named() {
    let thug = Interactable {
        code: NORMAN,
        source: (0xF << 16 | 0x2B) << 32 | 0x08D1E9A8,
        move_model: true,
        ..Interactable::new("people", "person", (0, 0))
    };
    assert_eq!(thug.naming(MART), Some((SPRITE, NORMAN, 0)));
}

/// It is a different Pokémon each time and gone in a minute, its source is the record's
/// address rather than a slot in any table, and the DexNav names the species itself once it is
/// registered. A name here would be a key that never matched again.
#[test]
fn test_a_hidden_pokemon_has_no_name_of_its_own() {
    assert_eq!(hiding().naming(MART), None);
}

/// Without this the key wrote the record's heap address into the player's file as a slot —
/// `0x1af:147974568` — which is junk the next time the game is started. Neither branch was wrong
/// on its own: renaming did not know about a category whose source is an address.
#[test]
fn test_the_rename_key_on_a_hidden_pokemon_opens_no_box() {
    let at = Field {
        interactables: vec![hiding()],
        ..Field::new(MART, (94, 172))
    };
    let (mut nav, backend, dialog, names) = navigator(Wanted {
        r#where: at,
        ..Wanted::default()
    });
    nav.press(ITEM_READ).unwrap();
    backend.spoken.borrow_mut().clear();
    nav.press(RENAME).unwrap();
    assert_eq!(dialog.asked(), []);
    assert_eq!(backend.spoken(), ["hidden Pokémon cannot be renamed"]);
    assert_eq!(names.mine(), nothing_named());
}

/// Not renameable is not unnameable: the list still has to say one is there.
#[test]
fn test_a_hidden_pokemon_is_still_said_by_what_it_is() {
    let at = Field {
        interactables: vec![hiding()],
        ..Field::new(MART, (94, 172))
    };
    let (mut nav, backend, _, _) = navigator(Wanted {
        r#where: at,
        ..Wanted::default()
    });
    nav.press(ITEM_READ).unwrap();
    assert_eq!(backend.spoken(), ["hidden Pokémon, 1 of 1, 2 east 2 north"]);
}

// ---------------------------------------------------------------- the dialog

/// A blind player has only the label to tell them which of three signs they are naming.
#[test]
fn test_the_box_says_what_is_being_renamed_and_where_it_is() {
    let (mut nav, _, dialog, _) = navigator(Wanted::default());
    nav.press(CATEGORY_NEXT).unwrap(); // exits
    nav.press(ITEM_READ).unwrap();
    nav.press(RENAME).unwrap();
    nav.asker.wait_default();
    assert_eq!(
        dialog.asked(),
        [box_of("Rename", "door, 1 north. Not named yet", "")]
    );
}

/// And what it is said as afterwards keeps the place after the new name (the user,
/// 2026-09-24).
#[test]
fn test_the_box_for_an_exit_into_another_place_says_the_place() {
    let (mut nav, backend, dialog, _) = navigator(Wanted {
        answer: Some("the Mart".to_string()),
        leads_to: Some(Rc::new(|_here, _there| Ok(Some("Route 101".to_string())))),
        ..Wanted::default()
    });
    nav.press(CATEGORY_NEXT).unwrap(); // exits
    rename(&mut nav);
    nav.press(ITEM_READ).unwrap();
    assert_eq!(
        dialog.asked(),
        [box_of(
            "Rename",
            "door to Route 101, 1 north. Not named yet",
            ""
        )]
    );
    let spoken = backend.spoken();
    assert_eq!(
        spoken[spoken.len() - 2..],
        [
            "door to Route 101 is now the Mart",
            "the Mart to Route 101, 1 of 1, 1 north"
        ]
    );
}

/// So a correction is a few keystrokes, and so the player hears what it is called now.
#[test]
fn test_a_thing_already_named_starts_the_box_with_the_name_it_has() {
    let names = Rc::new(
        Names::new(
            Some(&shipped(&[("0x13f", "Norman")])),
            None,
            None,
            None,
            None,
        )
        .unwrap(),
    );
    let (mut nav, _, dialog, _) = navigator(Wanted {
        names: Some(names),
        ..Wanted::default()
    });
    nav.press(CATEGORY_NEXT).unwrap();
    nav.press(CATEGORY_NEXT).unwrap();
    nav.press(CATEGORY_NEXT).unwrap(); // people
    nav.press(ITEM_READ).unwrap();
    nav.press(RENAME).unwrap();
    nav.asker.wait_default();
    assert_eq!(
        dialog.asked(),
        [box_of(
            "Rename",
            "person, 10 east 7 south. Called: Norman",
            "Norman"
        )]
    );
}

/// The common case for a mistyped key. A window left in front of the game would be worse
/// than saying nothing.
#[test]
fn test_nothing_chosen_opens_no_box_at_all() {
    let (mut nav, backend, dialog, _) = navigator(Wanted {
        r#where: Field::new(MART, (94, 172)),
        ..Wanted::default()
    });
    nav.press(RENAME).unwrap();
    assert_eq!(dialog.asked(), []);
    assert_eq!(backend.spoken(), ["nothing to rename"]);
}

/// A threading.Event: set once, waited on with a timeout.
struct Event(Mutex<bool>, Condvar);

impl Event {
    fn new() -> Arc<Self> {
        Arc::new(Event(Mutex::new(false), Condvar::new()))
    }

    fn wait(&self, timeout: f64) {
        let set = self.0.lock().unwrap();
        drop(
            self.1
                .wait_timeout_while(set, Duration::from_secs_f64(timeout), |set| !*set)
                .unwrap(),
        );
    }

    fn set(&self) {
        *self.0.lock().unwrap() = true;
        self.1.notify_all();
    }
}

/// Two boxes over the game, with the player unable to see either, is the worst outcome
/// here.
#[test]
fn test_a_second_press_while_the_box_is_up_does_not_stack_another() {
    let holding = Event::new();
    let asked: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let never: Ask = {
        let (holding, asked) = (holding.clone(), asked.clone());
        Arc::new(move |_title, label, _current, _check| {
            asked.lock().unwrap().push(label.to_string());
            holding.wait(2.0);
            Ok(None)
        })
    };

    let (mut nav, backend, _, _) = navigator(Wanted::default());
    nav.asker = Asker::new(Some(never));
    nav.press(ITEM_READ).unwrap();
    nav.press(RENAME).unwrap();
    // The box is up once its thread has reached the stand-in.
    while asked.lock().unwrap().is_empty() {
        std::hint::spin_loop();
    }
    backend.spoken.borrow_mut().clear();
    nav.press(RENAME).unwrap();
    assert_eq!(backend.spoken(), ["already renaming"]);
    assert_eq!(asked.lock().unwrap().len(), 1);
    holding.set();
}

// ---------------------------------------------------------------- what sticks

/// The box closing is not evidence to a blind player that anything was saved.
#[test]
fn test_a_name_given_is_kept_and_said_back() {
    let (mut nav, backend, _, names) = navigator(Wanted::default());
    nav.press(ITEM_NEXT).unwrap(); // the sign
    backend.spoken.borrow_mut().clear();
    rename(&mut nav);
    assert_eq!(names.place(MART, 1).as_deref(), Some("the Mart's sign"));
    assert_eq!(backend.spoken(), ["sign is now the Mart's sign"]);
}

/// Renaming a sign and coming back to it: the list says what the player called it.
#[test]
fn test_the_navigation_keys_say_the_new_name_afterwards() {
    let (mut nav, backend, _, _) = navigator(Wanted::default());
    nav.press(ITEM_NEXT).unwrap();
    rename(&mut nav);
    backend.spoken.borrow_mut().clear();
    nav.press(ITEM_READ).unwrap();
    assert_eq!(
        backend.spoken(),
        ["the Mart's sign, 2 of 3, 3 east 1 north"]
    );
}

#[test]
fn test_naming_a_person_names_the_character_everywhere() {
    let (mut nav, _, _, names) = navigator(Wanted {
        answer: Some("Dad".to_string()),
        ..Wanted::default()
    });
    nav.press(ITEM_NEXT).unwrap();
    nav.press(ITEM_NEXT).unwrap(); // the person
    rename(&mut nav);
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Dad"));
}

#[test]
fn test_a_cancelled_box_changes_nothing() {
    let (mut nav, backend, _, names) = navigator(Wanted {
        answer: None,
        ..Wanted::default()
    });
    nav.press(ITEM_NEXT).unwrap();
    backend.spoken.borrow_mut().clear();
    rename(&mut nav);
    assert_eq!(names.place(MART, 1), None);
    assert_eq!(backend.spoken(), [] as [&str; 0]);
}

/// "Rival" comes back after the player's "May" is cleared, and the player is told it has.
#[test]
fn test_an_empty_name_gives_the_thing_back_to_the_shipped_table() {
    let names = Rc::new(
        Names::new(
            Some(&shipped(&[("0x13f", "Norman")])),
            None,
            None,
            None,
            None,
        )
        .unwrap(),
    );
    let (mut nav, backend, _, _) = navigator(Wanted {
        answer: Some(String::new()),
        names: Some(names.clone()),
        ..Wanted::default()
    });
    nav.press(ITEM_NEXT).unwrap();
    nav.press(ITEM_NEXT).unwrap();
    backend.spoken.borrow_mut().clear();
    rename(&mut nav);
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Norman"));
    assert_eq!(backend.spoken(), ["Norman is now Norman"]);
}

#[test]
fn test_an_empty_name_with_nothing_shipped_says_it_has_no_name() {
    let (mut nav, backend, _, names) = navigator(Wanted {
        answer: Some(String::new()),
        ..Wanted::default()
    });
    nav.press(ITEM_NEXT).unwrap();
    rename(&mut nav);
    nav.press(RENAME).unwrap();
    nav.asker.wait_default();
    nav.poll_renames().unwrap();
    assert_eq!(names.place(MART, 1), None);
    assert_eq!(backend.spoken().last().unwrap(), "sign has no name");
}

/// The whole point of writing it down rather than keeping it for the run.
#[test]
fn test_a_name_is_there_again_the_next_time_pa3ds_starts() {
    let tmp_path = TmpPath::new();
    let path = tmp_path.join("sango-1.toml");
    let names = Rc::new(Names::new(None, Some(&path), None, None, None).unwrap());
    let (mut nav, _, _, _) = navigator(Wanted {
        names: Some(names),
        ..Wanted::default()
    });
    nav.press(ITEM_NEXT).unwrap();
    rename(&mut nav);
    let again = Names::new(None, Some(&path), None, None, None).unwrap();
    assert_eq!(again.place(MART, 1).as_deref(), Some("the Mart's sign"));
}

// ---------------------------------------------------------------- the last speaker

/// A player knows who somebody is the moment they have heard them, which is after the line
/// and often after the person has walked off.
#[test]
fn test_naming_the_last_speaker_needs_no_map_and_no_list() {
    let (mut nav, _, dialog, names) = navigator(Wanted {
        answer: Some("Dad".to_string()),
        spoke: Some(NORMAN),
        ..Wanted::default()
    });
    nav.press(RENAME_SPEAKER).unwrap();
    nav.asker.wait_default();
    nav.poll_renames().unwrap();
    assert_eq!(
        dialog.asked(),
        [box_of("Rename", "The last speaker. Not named yet", "")]
    );
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Dad"));
}

/// Both are the character, so naming one names the other. That is the answer to the question
/// the plan left open: which key space a person belongs to is the thing's business.
#[test]
fn test_the_speaker_and_the_person_on_the_map_are_the_same_name() {
    let (mut nav, _, dialog, names) = navigator(Wanted {
        answer: Some("Dad".to_string()),
        spoke: Some(NORMAN),
        ..Wanted::default()
    });
    nav.press(ITEM_NEXT).unwrap();
    nav.press(ITEM_NEXT).unwrap(); // the person on the map
    rename(&mut nav);
    assert_eq!(names.sprite(NORMAN).as_deref(), Some("Dad"));
    nav.press(RENAME_SPEAKER).unwrap();
    nav.asker.wait_default();
    assert_eq!(
        dialog.asked().last().unwrap(),
        &box_of("Rename", "The last speaker. Called: Dad", "Dad")
    );
}

#[test]
fn test_nobody_having_spoken_opens_no_box() {
    let (mut nav, backend, dialog, _) = navigator(Wanted::default());
    nav.press(RENAME_SPEAKER).unwrap();
    assert_eq!(dialog.asked(), []);
    assert_eq!(backend.spoken(), ["nobody has spoken"]);
}

// ---------------------------------------------------------------- one of many drawn alike

const YOUNGSTER: i64 = 0xDA;
const ROUTE: i64 = 0x10E;

fn calvin() -> Interactable {
    Interactable {
        code: YOUNGSTER,
        source: 0x08D1E9A8,
        move_model: true,
        record: Some((ROUTE, 7)),
        ..Interactable::new("people", "person", (95, 172))
    }
}

fn other() -> Interactable {
    Interactable {
        code: YOUNGSTER,
        source: 0x08D1F458,
        move_model: true,
        record: Some((ROUTE, 8)),
        ..Interactable::new("people", "person", (96, 172))
    }
}

/// Two Youngsters on Route 102, one sprite and two records: Calvin is trainer 7 (spike 90).
fn youngsters() -> Field {
    Field {
        interactables: vec![calvin(), other()],
        ..Field::new(ROUTE, (94, 172))
    }
}

/// Orion, 2026-10-11: the shipped names already call everybody what they look like, so a
/// rename is more likely for one than for all, and the box opens ticked. Calvin's name is a slot
/// on his record's map, and the other Youngster keeps the sprite's.
#[test]
fn test_one_youngster_is_named_alone_while_the_box_stays_ticked() {
    let names = Rc::new(
        Names::new(
            Some(&shipped(&[("0xda", "Youngster")])),
            None,
            None,
            None,
            None,
        )
        .unwrap(),
    );
    let (mut nav, backend, dialog, _) = navigator(Wanted {
        answer: Some("Youngster Calvin".to_string()),
        names: Some(names.clone()),
        r#where: youngsters(),
        ..Wanted::default()
    });
    nav.press(ITEM_READ).unwrap();
    backend.spoken.borrow_mut().clear();
    rename(&mut nav);
    assert_eq!(
        dialog.asked(),
        [box_with(
            "Rename",
            "person, 1 east. Called: Youngster",
            "Youngster",
            "Only this one"
        )]
    );
    assert_eq!(
        names.place(ROUTE, MODEL_SOURCE | 7).as_deref(),
        Some("Youngster Calvin")
    );
    assert_eq!(names.sprite(YOUNGSTER).as_deref(), Some("Youngster"));
    nav.press(ITEM_NEXT).unwrap();
    nav.press(ITEM_NEXT).unwrap();
    assert_eq!(
        backend.spoken(),
        [
            "Youngster is now Youngster Calvin",
            "Youngster, 2 of 2, 2 east",
            "Youngster Calvin, 1 of 2, 1 east"
        ]
    );
}

#[test]
fn test_the_box_unticked_names_everyone_drawn_alike_and_one_named_alone_keeps_theirs() {
    let names = Rc::new(Names::default());
    names.name_place(ROUTE, MODEL_SOURCE | 7, "Calvin").unwrap();
    let (mut nav, backend, _, _) = navigator(Wanted {
        answer: Some("Youngster".to_string()),
        names: Some(names.clone()),
        r#where: youngsters(),
        ticked: false,
        ..Wanted::default()
    });
    nav.press(ITEM_NEXT).unwrap(); // the other Youngster
    rename(&mut nav);
    assert_eq!(names.sprite(YOUNGSTER).as_deref(), Some("Youngster"));
    backend.spoken.borrow_mut().clear();
    nav.press(ITEM_READ).unwrap();
    nav.press(ITEM_NEXT).unwrap();
    assert_eq!(
        backend.spoken(),
        ["Youngster, 2 of 2, 2 east", "Calvin, 1 of 2, 1 east"]
    );
}

#[test]
fn test_clearing_a_name_given_alone_gives_the_one_back_to_the_sprites() {
    let names = Rc::new(
        Names::new(
            Some(&shipped(&[("0xda", "Youngster")])),
            None,
            None,
            None,
            None,
        )
        .unwrap(),
    );
    names.name_place(ROUTE, MODEL_SOURCE | 7, "Calvin").unwrap();
    let (mut nav, backend, _, _) = navigator(Wanted {
        answer: Some(String::new()),
        names: Some(names.clone()),
        r#where: youngsters(),
        ..Wanted::default()
    });
    nav.press(ITEM_READ).unwrap();
    backend.spoken.borrow_mut().clear();
    rename(&mut nav);
    assert_eq!(names.place(ROUTE, MODEL_SOURCE | 7), None);
    assert_eq!(backend.spoken(), ["Calvin is now Youngster"]);
}

/// Naming him from what he said names the one on the map, and the other way round.
#[test]
fn test_the_last_speaker_is_named_alone_as_the_one_on_the_map_is() {
    let (mut nav, _, dialog, names) = navigator(Wanted {
        answer: Some("Calvin".to_string()),
        r#where: youngsters(),
        spoke: Some(YOUNGSTER),
        own: calvin().own().unwrap(),
        ..Wanted::default()
    });
    nav.press(RENAME_SPEAKER).unwrap();
    nav.asker.wait_default();
    nav.poll_renames().unwrap();
    assert_eq!(
        dialog.asked(),
        [box_with(
            "Rename",
            "The last speaker. Not named yet",
            "",
            "Only this one"
        )]
    );
    assert_eq!(
        names.place(ROUTE, MODEL_SOURCE | 7).as_deref(),
        Some("Calvin")
    );
    assert_eq!(names.sprite(YOUNGSTER), None);
}

/// Sun's people are read without the record they were built from.
#[test]
fn test_a_person_with_no_record_has_no_box_to_tick() {
    let (mut nav, _, dialog, _) = navigator(Wanted {
        r#where: Field {
            interactables: vec![town().interactables[2].clone()],
            ..Field::new(MART, (94, 172))
        },
        ..Wanted::default()
    });
    nav.press(ITEM_READ).unwrap();
    nav.press(RENAME).unwrap();
    nav.asker.wait_default();
    assert_eq!(dialog.asked()[0].3, "");
}

// ---------------------------------------------------------------- the template

/// A dialog template read back: its title, and each item as (id, class atom, text).
///
/// Windows does not forgive a template that is a byte out — it builds the wrong controls, or
/// none — and nothing here can see the window that comes out. Reading the bytes back the way
/// Windows reads them is the next best thing: if this parser finds four items with the right
/// classes in the right order, so will `DialogBoxIndirectParamW`.
pub(super) fn read_template(built: &[u8]) -> (String, Vec<(i64, i64, String)>) {
    fn wstr(built: &[u8], at: &mut usize) -> String {
        let mut end = *at;
        while built[end..end + 2] != [0, 0] {
            end += 2;
        }
        let text = py::decode_utf16le_strict(&built[*at..end]).unwrap();
        *at = end + 2;
        text
    }
    let word = |at: usize| i64::from(u16::from_le_bytes([built[at], built[at + 1]]));

    let mut at = 18; // the fixed DLGTEMPLATE header
    let count = word(8);

    assert_eq!(built[at..at + 2], [0, 0]); // no menu
    assert_eq!(built[at + 2..at + 4], [0, 0]); // the default class: what makes it a #32770
    at += 4;
    let title = wstr(built, &mut at);
    at += 2; // DS_SETFONT point size
    wstr(built, &mut at); // the typeface
    let mut items = Vec::new();
    for _ in 0..count {
        at += py::modulo(-(at as i64), 4) as usize; // every item starts on a 4-byte boundary
        let ident = word(at + 16);
        assert_eq!(built[at + 18..at + 20], [0xff, 0xff]); // a predefined class, by atom
        let atom = word(at + 20);
        at += 22;
        items.push((ident, atom, wstr(built, &mut at)));
        at += 2; // no creation data
    }
    (title, items)
}

/// The label must come immediately before the edit, because that is how Win32 says which
/// static names which control — it is why NVDA reads "Sign, 1 north. Called: Signpost 1, edit"
/// and not just "edit".
#[test]
fn test_the_dialog_template_builds_a_real_dialog() {
    let built = template("Rename", "sign, 1 north. Called: Signpost 1", "", 260).unwrap();
    let (title, items) = read_template(&built);
    assert_eq!(title, "Rename");
    assert_eq!(
        items,
        [
            (
                IDC_LABEL,
                ATOM_STATIC,
                "sign, 1 north. Called: Signpost 1".to_string()
            ),
            (IDC_EDIT, ATOM_EDIT, String::new()),
            (IDOK, ATOM_BUTTON, "OK".to_string()),
            (IDCANCEL, ATOM_BUTTON, "Cancel".to_string()),
        ]
    );
}

/// A Tab from the name, and named by its own words, as a checkbox is.
#[test]
fn test_the_checkbox_comes_after_the_edit_and_before_the_buttons() {
    let (_, items) =
        read_template(&template("Rename", "person, 1 east", "Only this one", 260).unwrap());
    assert_eq!(
        items,
        [
            (IDC_LABEL, ATOM_STATIC, "person, 1 east".to_string()),
            (IDC_EDIT, ATOM_EDIT, String::new()),
            (IDC_CHECK, ATOM_BUTTON, "Only this one".to_string()),
            (IDOK, ATOM_BUTTON, "OK".to_string()),
            (IDCANCEL, ATOM_BUTTON, "Cancel".to_string()),
        ]
    );
}

// ---------------------------------------------------------------- the game's own word for a door

/// Names that know what the game calls the building behind this map's door, slot 0.
fn game_names() -> Rc<Names> {
    let names = Names::default();
    names.use_game(
        None,
        Some(Rc::new(|zone, slot| {
            if (zone, slot) == (MART, warp(0)) {
                Some("Pokémon Center".to_string())
            } else {
                None
            }
        })),
    );
    Rc::new(names)
}

/// The user, 2026-09-23: a named door replaces the word "door", as a rename does.
#[test]
fn test_a_door_the_rom_names_is_said_by_that_name() {
    let (mut nav, backend, _, _) = navigator(Wanted {
        names: Some(game_names()),
        ..Wanted::default()
    });
    nav.press(ITEM_READ).unwrap();
    assert_eq!(backend.spoken(), ["Pokémon Center, 1 of 3, 1 north"]);
}

#[test]
fn test_the_players_own_name_for_a_door_wins_over_the_games() {
    let (mut nav, backend, dialog, names) = navigator(Wanted {
        answer: Some("the Center".to_string()),
        names: Some(game_names()),
        ..Wanted::default()
    });
    nav.press(ITEM_READ).unwrap();
    rename(&mut nav);
    assert_eq!(dialog.asked()[0].2, "Pokémon Center");
    assert_eq!(names.place(MART, warp(0)).as_deref(), Some("the Center"));
    backend.spoken.borrow_mut().clear();
    nav.press(ITEM_READ).unwrap();
    assert_eq!(backend.spoken(), ["the Center, 1 of 3, 1 north"]);
}

#[test]
fn test_clearing_a_doors_name_gives_it_back_to_the_game() {
    let (mut nav, backend, dialog, names) = navigator(Wanted {
        answer: Some("the Center".to_string()),
        names: Some(game_names()),
        ..Wanted::default()
    });
    nav.press(ITEM_READ).unwrap();
    rename(&mut nav);
    *dialog.answer.lock().unwrap() = Some(String::new());
    rename(&mut nav);
    assert_eq!(
        names.place(MART, warp(0)).as_deref(),
        Some("Pokémon Center")
    );
    assert_eq!(
        backend.spoken().last().unwrap(),
        "the Center is now Pokémon Center"
    );
}

/// Sign 0 and door 0 on one map shared a name until 2026-09-23: renaming the sign renamed the
/// door, and the ROM's name for the building behind the door was said for the sign. They are
/// numbered in tables of their own, and a warp's slot carries a bit of its own to say so.
#[test]
fn test_a_sign_and_a_door_numbered_alike_keep_names_of_their_own() {
    let both = Field {
        interactables: vec![
            Interactable {
                zone: Some(0xDF),
                source: warp(0),
                ..Interactable::new("exits", "door", (94, 171))
            },
            Interactable {
                source: 0,
                ..Interactable::new("signs", "sign", (97, 171))
            },
        ],
        ..Field::new(MART, (94, 172))
    };
    let (mut nav, backend, _, names) = navigator(Wanted {
        names: Some(game_names()),
        r#where: both,
        ..Wanted::default()
    });
    nav.press(ITEM_NEXT).unwrap(); // the sign
    backend.spoken.borrow_mut().clear();
    rename(&mut nav);
    assert_eq!(names.place(MART, 0).as_deref(), Some("the Mart's sign"));
    assert_eq!(
        names.place(MART, warp(0)).as_deref(),
        Some("Pokémon Center")
    );
    backend.spoken.borrow_mut().clear();
    nav.press(ITEM_READ).unwrap();
    nav.press(ITEM_NEXT).unwrap();
    assert_eq!(
        backend.spoken(),
        [
            "the Mart's sign, 2 of 2, 3 east 1 north",
            "Pokémon Center, 1 of 2, 1 north"
        ]
    );
}
