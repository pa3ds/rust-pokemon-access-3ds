//! The gen 6 profile: the field's voices, P's ways and touch mode's targets logged, the rival and
//! where an exit leads named by the game's words, the PC, and what each game sweeps for.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::fake_speech::Recorder;
use super::test_field::{
    FILE, HOLDER, PC_TILE, POOCHYENA, ROUTE_BLOCK, TOWN, TOWN_BLOCK, TOWN_CHUNK, Town, border, game,
};
use crate::core::bcsar::Sound;
use crate::core::clips::Clips;
use crate::core::config::{Amie, Tables};
use crate::core::discovery::Found;
use crate::core::loudness::Loudness;
use crate::core::memory::{Cache, Memory};
use crate::core::names::Names;
use crate::core::rtti::ClassNames;
use crate::core::speech::{Category, Speech};
use crate::games::common::dialogue::{PageShown, Shown, TextWritten};
use crate::games::common::field::keys::{ENEMY_SIDE, ITEM_NEXT, MODE_TOGGLE, OUR_SIDE, PATHFIND};
use crate::games::common::session::Ways;
use crate::games::common::touch::Target;
use crate::games::gen6::apps::{BATTLE, DEMO, FIELD, MANAGER_CLASS};
use crate::games::gen6::field::features::Tracked;
use crate::games::gen6::field::landform::{Here, Landform, gen6};
use crate::games::gen6::feedback::{CUE_SOUND, FeedbackSounds, Plays};
use crate::games::gen6::field::voice::{Said, Speaking};
use crate::games::gen6::field::{dowsing, mapblock};
use crate::games::gen6::game::Game;
use crate::games::gen6::own::{Kit, Own};
use crate::games::gen6::profile::{AfterForget, Gen6Profile, HELD_MOST, Options, STILL};
use crate::games::gen6::rom::archives::{GameText, no_text};
use crate::games::gen6::screens::summary::SUMMARY;
use crate::games::gen6::screens::{partyicons, reminder, rows};
use crate::games::oras::field::hidden::PLAYER_WORK_CLASS;
use crate::games::oras::own::Oras;
use crate::games::{gen6::gflheap, oras::field::sky};
use crate::games::{oras, xy};
use crate::py::{Log, PySet, Result};

const WORK_OBJECT: i64 = 0x7A;
const MAY: i64 = 0x13A;
const BRENDAN: i64 = 0x13B;

/// `PLACES`: the place names as the ROM lists them, Littleroot Town at 170 and Route 101 at 204.
fn places() -> Vec<String> {
    let mut places = vec!["——————".to_string()];
    places.extend(std::iter::repeat_n(String::new(), 169));
    places.push("Littleroot Town".to_string());
    places.extend(std::iter::repeat_n(String::new(), 33));
    places.push("Route 101".to_string());
    places
}

fn trainers() -> Vec<String> {
    ["-", "Brendan", "Brendan", "Brendan", "May", "May", "May"]
        .iter()
        .map(|name| name.to_string())
        .collect()
}

/// The fake town's memory, read in phases as the profile reads the page cache.
struct Phased {
    memory: Rc<FakeMemory>,
}

impl Phased {
    fn new(memory: Rc<FakeMemory>) -> Rc<Phased> {
        Rc::new(Phased { memory })
    }
}

impl Memory for Phased {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        self.memory.read(address, size)
    }
}

impl Cache for Phased {
    fn begin(&self, _name: &str) -> Result<()> {
        Ok(())
    }

    fn end(&self, _name: &str) {}

    /// The Python's `Phased` has no `read_each`: nothing the profile does here asks for one.
    fn read_each(&self, _reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        panic!("Phased has no attribute 'read_each'")
    }
}

pub(crate) fn no_log() -> Log {
    Rc::new(|_: &str| {})
}

fn profile(
    town: &Town,
    text: GameText,
    named: Option<Rc<Names>>,
    log: Log,
) -> (Gen6Profile, Rc<Recorder>) {
    profile_of(town, text, named, log, oras::game())
}

/// `profile(town, text, named, log, game)`: Python's `game` defaults to `oras.GAME`.
pub(crate) fn profile_of(
    town: &Town,
    text: GameText,
    named: Option<Rc<Names>>,
    log: Log,
    game: &Game,
) -> (Gen6Profile, Rc<Recorder>) {
    let backend = Recorder::new();
    let made = Gen6Profile::new(
        Phased::new(town.memory.clone()),
        Rc::new(ClassNames::new(town.memory.clone(), CODE)),
        Rc::new(Speech::plain(backend.clone())),
        log,
        game,
        Options {
            named,
            text: Rc::new(text),
            ..Options::default()
        },
    );
    made.set_field(town.reader.clone());
    town.reader.track(&town.found()).unwrap();
    (made, backend)
}

fn names(shipped: &[(&str, &str)]) -> Rc<Names> {
    let shipped: IndexMap<String, String> = shipped
        .iter()
        .map(|(key, name)| (key.to_string(), name.to_string()))
        .collect();
    Rc::new(Names::new(Some(&shipped), None, None, None, None).unwrap())
}

/// A landform's watch that speaks, a numbered line each time it is asked: a game's own field
/// voice, as ORAS's decorating cursor is.
struct Talker {
    asked: Cell<i64>,
}

impl Tracked for Talker {
    fn classes(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn track(&self, _found: &Found) -> Result<()> {
        Ok(())
    }

    fn speaking(self: Rc<Self>) -> Option<Rc<dyn Speaking>> {
        Some(self)
    }
}

impl Speaking for Talker {
    fn said(&self, _here: &Here) -> Result<Option<Said>> {
        self.asked.set(self.asked.get() + 1);
        let words = format!("game {}", self.asked.get());
        Ok(Some(Said {
            words: words.clone(),
            category: Category::Menu,
            logged: words,
        }))
    }
}

/// A landform read through `Talker`. Python's keeps the talker it is given; a landform here is
/// shared between threads and keeps none, so it makes one for the one field reader that asks.
#[derive(Debug)]
struct Talking;

impl Landform for Talking {
    fn watch(
        &self,
        _memory: Rc<dyn Memory>,
        _image: &[u8],
        _text: &GameText,
    ) -> Option<Rc<dyn Tracked>> {
        Some(Rc::new(Talker {
            asked: Cell::new(0),
        }))
    }
}

/// The field's voices are polled in the order the profile writes down: gen 6's own, the
/// Dowsing Machine's lights, then the game's own, as ORAS's decorating cursor, each logged and
/// said as it says. The lights are read while nothing says where the player is, as they were
/// before the field's voices, and say a colour read on that poll and the next; the game's say
/// nothing then.
#[test]
fn test_the_field_voices_say_gen_6s_first_then_the_games() {
    let (process, work, record) = (0x08C65224, 0x08248DD4, 0x0828B748);
    let town = Town::of(Game {
        landforms: [gen6(), vec![Arc::new(Talking) as Arc<dyn Landform>]].concat(),
        ..game()
    });
    town.memory.u32(process + dowsing::WORK, &[work]);
    town.memory.u32(work + oras::DOWSING, &[record]);
    town.memory.u32(record + dowsing::PARENT, &[work]);
    town.memory.u32(record + dowsing::LEVEL, &[2]);
    let logged: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let log: Log = {
        let logged = logged.clone();
        Rc::new(move |line: &str| logged.borrow_mut().push(line.to_string()))
    };
    let (made, backend) = profile(&town, GameText::new(places(), trainers()), None, log);
    {
        let mut apps = made.apps.borrow_mut();
        apps.running = Some(FIELD.to_string());
        apps.process = Some(process);
    }
    made.poll_field().unwrap();
    made.poll_field().unwrap();
    town.memory.u32(record + dowsing::LEVEL, &[4]);
    town.memory.u32(HOLDER, &[0]);
    made.poll_field().unwrap();
    town.memory.u32(HOLDER, &[FILE]);
    made.poll_field().unwrap();
    let said = ["game 1", "dowsing green", "game 2", "dowsing red", "game 3"];
    let lines: Vec<String> = logged
        .borrow()
        .iter()
        .filter(|line| line.starts_with("field game") || line.starts_with("field dowsing"))
        .cloned()
        .collect();
    assert_eq!(lines, said.map(|words| format!("field {words}")));
    let spoken: Vec<String> = backend
        .spoken()
        .into_iter()
        .filter(|words| said.contains(&words.as_str()))
        .collect();
    assert_eq!(spoken, said);
}

/// ADR-0041: what P says of each thing listed is logged where a tape replay asks for it, read
/// inside what it gives; a recording and a player's run log none.
#[test]
fn test_a_tape_replay_logs_ps_ways_at_each_listing_and_any_other_run_never() {
    let town = Town::new();
    let entered: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
    let aside: Ways = {
        let entered = entered.clone();
        Rc::new(move |inside: &mut dyn FnMut() -> crate::py::Result<()>| {
            entered.borrow_mut().push(true);
            inside()
        })
    };
    let made = |ways: Option<Ways>| -> Vec<String> {
        let logged: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let log: Log = {
            let logged = logged.clone();
            Rc::new(move |line: &str| logged.borrow_mut().push(line.to_string()))
        };
        let profile = Gen6Profile::new(
            Phased::new(town.memory.clone()),
            Rc::new(ClassNames::new(town.memory.clone(), CODE)),
            Rc::new(Speech::plain(Recorder::new())),
            log,
            oras::game(),
            Options {
                text: Rc::new(GameText::new(places(), trainers())),
                ways,
                ..Options::default()
            },
        );
        profile.set_field(town.reader.clone());
        town.reader.track(&town.found()).unwrap();
        profile.poll_field().unwrap();
        logged
            .borrow()
            .iter()
            .filter(|line| line.starts_with("field way to "))
            .cloned()
            .collect()
    };
    assert!(!made(Some(aside)).is_empty() && *entered.borrow() == [true]);
    assert_eq!(made(None), Vec::<String>::new());
}

/// `[pa3ds.navigation] guide_to_hidden`, on by default (movie26): a find, not every
/// poll it stays up; a faint, then the next search's find, is a find again; and what the
/// detector turns up is no search's, "not a random Poochyena!" (movie27), but for the
/// DexNav's in touch mode.
#[test]
fn test_a_hidden_pokemon_is_guided_to_as_a_search_finds_it_and_once() {
    let town = Town::new();
    let (made, backend) = profile(&town, no_text(), None, no_log());
    made.poll_field().unwrap();
    let finds = || {
        backend
            .spoken()
            .iter()
            .filter(|text| text.starts_with("hidden Pokémon, "))
            .count()
    };
    town.hide_found(POOCHYENA, 5, (97, 175), 0x08, 3);
    made.poll_field().unwrap();
    made.poll_field().unwrap();
    assert_eq!(finds(), 1);
    town.hide_found(POOCHYENA, 5, (97, 175), 0x0D, 3);
    made.poll_field().unwrap();
    town.hide_found(POOCHYENA, 5, (98, 176), 0x08, 3);
    made.poll_field().unwrap();
    assert_eq!(finds(), 2);
    // The detector's, with the DexNav open, and then with the AreaNav: in list mode, neither.
    for (cell, found_by) in [((96, 176), 1), ((95, 176), 2)] {
        town.hide_state(POOCHYENA, 5, cell, 0xFF);
        made.poll_field().unwrap();
        town.hide_found(POOCHYENA, 5, cell, 0x08, found_by);
        made.poll_field().unwrap();
    }
    assert_eq!(finds(), 2);
    // In touch mode, at the DexNav, the DexNav's; still not the AreaNav's.
    made.navigator.borrow_mut().press(MODE_TOGGLE).unwrap();
    made.navigator.borrow_mut().press(MODE_TOGGLE).unwrap();
    for (cell, found_by) in [((95, 175), 2), ((96, 175), 1)] {
        town.hide_state(POOCHYENA, 5, cell, 0xFF);
        made.poll_field().unwrap();
        town.hide_found(POOCHYENA, 5, cell, 0x08, found_by);
        made.poll_field().unwrap();
    }
    assert_eq!(finds(), 3);
}

/// A battle or a scene unloads the map and runs with the field's models still in memory: after
/// Groudon's capture the keys listed it in the cave left behind. While the map
/// is still loaded under another screen the list is read, and P finds no way; and the item chosen
/// is kept for when the field comes back.
#[test]
fn test_there_is_no_map_once_another_screen_has_unloaded_it() {
    let (control, heard) = profile(&Town::new(), no_text(), None, no_log());
    for _ in 0..2 {
        control.navigator.borrow_mut().press(ITEM_NEXT).unwrap();
    }
    let walked = heard.spoken();
    let town = Town::new();
    let (made, backend) = profile(&town, no_text(), None, no_log());
    let running = |name: &str| made.apps.borrow_mut().running = Some(name.to_string());
    made.navigator.borrow_mut().press(ITEM_NEXT).unwrap();
    running(DEMO);
    made.navigator.borrow_mut().press(PATHFIND).unwrap();
    for block in [TOWN_BLOCK, ROUTE_BLOCK] {
        town.memory.u32(block + mapblock::STATE, &[0]);
    }
    running(BATTLE);
    made.navigator.borrow_mut().press(ITEM_NEXT).unwrap();
    for block in [TOWN_BLOCK, ROUTE_BLOCK] {
        town.memory.u32(block + mapblock::STATE, &[mapblock::READY]);
    }
    running(FIELD);
    made.navigator.borrow_mut().press(ITEM_NEXT).unwrap();
    assert_eq!(
        backend.spoken(),
        [
            walked[0].as_str(),
            "can't walk now",
            "no map",
            walked[1].as_str()
        ]
    );
}

/// A battle has begun seconds before the game draws either side, at "Go!", and R said "no
/// battle" through them (a player's report).
#[test]
fn test_r_in_a_battle_before_its_sides_are_drawn_says_so_and_not_no_battle() {
    let (made, backend) = profile(&Town::new(), no_text(), None, no_log());
    made.navigator.borrow_mut().press(OUR_SIDE).unwrap();
    made.apps.borrow_mut().running = Some(BATTLE.to_string());
    made.navigator.borrow_mut().press(OUR_SIDE).unwrap();
    made.navigator.borrow_mut().press(ENEMY_SIDE).unwrap();
    assert_eq!(
        backend.spoken(),
        ["no battle", "nothing shown yet", "nothing shown yet"]
    );
}

/// The summary's Exp. Points, a number beside a bar on the top screen, were taken for an
/// enemy's level whose gauge no sweep finds, and a whole sweep was asked for every poll, each
/// 1.1 s with nothing read meanwhile: 25 in 27 s (or_menus).
#[test]
fn test_the_battle_reader_asks_for_a_sweep_only_while_a_battle_runs() {
    let town = Town::new();
    let (made, _) = profile(&town, no_text(), None, no_log());
    // Python monkeypatches the battle reader to want a sweep, and the apps' to want none: here
    // the reader waits for a level's gauge, and the apps know their manager.
    made.battle.borrow_mut().wanted = PySet::of([0x100]);
    let manager = 0x08F00000;
    let classes = FakeClasses::starting(town.memory.clone(), 0x00180000);
    town.memory.u32(manager, &[classes.add(MANAGER_CLASS)]);
    made.apps.borrow_mut().track([manager]).unwrap();
    made.apps.borrow_mut().running = Some(SUMMARY.to_string());
    assert!(!made.sweep_wanted().unwrap());
    made.apps.borrow_mut().running = Some(BATTLE.to_string());
    assert!(made.sweep_wanted().unwrap());
}

/// ADR-0034: a whole sweep waits while a cursor moves. The Bag's own, 1.5 s after it starts,
/// read nothing else for 1.2 s as the player began moving down its list: the first item of
/// Medicine went without its details (spike 297).
#[test]
fn test_a_moving_cursor_holds_the_whole_sweep_off_until_it_is_still_for_three_seconds_at_most() {
    let (made, _) = profile(&Town::new(), no_text(), None, no_log());
    assert!(!made.sweep_held(10.0));
    made.moving.set(true);
    assert!(made.sweep_held(10.0));
    assert!(made.sweep_held(10.0 + STILL - 0.1));
    assert!(!made.sweep_held(10.0 + STILL));
    let mut held: Vec<bool> = Vec::new();
    for step in 0..10 {
        made.moving.set(true);
        held.push(made.sweep_held(20.0 + step as f64 * 0.5));
    }
    let expected: Vec<bool> = (0..10).map(|step| step as f64 * 0.5 < HELD_MOST).collect();
    assert_eq!(held, expected);
}

/// Spike 89: the rival's work object is drawn as SUPPORT_F_RS or SUPPORT_M_RS, and the ROM
/// names both, in the player's language, ahead of the English that ships; so does a kind of
/// trainer (spike 90). The work object itself is nobody, since a script can fill it with
/// anyone.
#[test]
fn test_the_rival_is_named_by_the_sprite_they_were_filled_with() {
    let town = Town::new();
    let named = names(&[
        ("0x13a", "May"),
        ("0x13b", "Brendan"),
        ("0xda", "Youngster"),
    ]);
    let text = GameText {
        sprites: IndexMap::from([
            (MAY, "ハルカ".to_string()),
            (BRENDAN, "ユウキ".to_string()),
            (0xDA, "たんぱんこぞう".to_string()),
        ]),
        ..GameText::new(places(), trainers())
    };
    profile(&town, text, Some(named.clone()), no_log());
    assert_eq!(named.sprite(MAY).as_deref(), Some("ハルカ"));
    assert_eq!(named.sprite(BRENDAN).as_deref(), Some("ユウキ"));
    assert_eq!(named.sprite(0xDA).as_deref(), Some("たんぱんこぞう"));
    assert_eq!(named.sprite(WORK_OBJECT), None);
}

#[test]
fn test_without_the_rom_the_rival_is_what_shipped() {
    let town = Town::new();
    let named = names(&[("0x13a", "May")]);
    profile(&town, no_text(), Some(named.clone()), no_log());
    assert_eq!(named.sprite(MAY).as_deref(), Some("May"));
}

/// The profile hands the navigator the game's place names (`PlaceNames`), which it says an exit
/// with and the listing's log names it by: the gap north of Littleroot leads to Route 101, and
/// the town's doors to no other place.
#[test]
fn test_the_navigator_and_the_listings_log_say_where_an_exit_leads() {
    let town = Town::new();
    let logged: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let log: Log = {
        let logged = logged.clone();
        Rc::new(move |line: &str| logged.borrow_mut().push(line.to_string()))
    };
    let (made, _) = profile(&town, GameText::new(places(), trainers()), None, log);
    let r#where = town.r#where().unwrap();
    let labels: Vec<String> = r#where
        .of("exits")
        .iter()
        .map(|item| made.navigator.borrow().label(item, TOWN).unwrap())
        .collect();
    assert_eq!(labels, ["door", "door", "path to Route 101"]);
    made.poll_field().unwrap();
    assert!(logged.borrow().iter().any(|line| {
        line.starts_with("field exits: ") && line.contains("edge to zone 0x17 (Route 101) ")
    }));
}

/// The profile hands the field the word the PC's own lines share (`archives.PC_LINES`).
#[test]
fn test_the_pc_is_listed_by_the_roms_word_for_it() {
    let mut town = Town::new();
    town.block_tiles(
        TOWN_BLOCK,
        TOWN_CHUNK,
        (80, 160),
        &border(160),
        &[((95, 170), PC_TILE)],
    );
    let made = Gen6Profile::new(
        Phased::new(town.memory.clone()),
        Rc::new(ClassNames::new(town.memory.clone(), CODE)),
        Rc::new(Speech::plain(Recorder::new())),
        no_log(),
        oras::game(),
        Options {
            text: Rc::new(GameText {
                pc: Some("パソコン".to_string()),
                ..GameText::new(places(), trainers())
            }),
            ..Options::default()
        },
    );
    made.field().track(&town.found()).unwrap();
    let r#where = made.field().poll().unwrap();
    let r#where = r#where.expect("the player's field");
    let signs = made.field().interactables(&r#where).unwrap().of("signs");
    let kinds: Vec<&str> = signs.iter().map(|item| item.kind.as_str()).collect();
    assert_eq!(kinds, ["パソコン", "sign", "object"]);
}

/// ADR-0041: no key of pa3ds's is pressed in a replay, so touch mode's targets are logged for a
/// tape to hold them, once each time they change, and "none" when they go.
#[test]
fn test_a_replay_logs_touch_modes_targets_as_they_change_and_a_players_run_never() {
    let town = Town::new();
    let offered: Rc<RefCell<Vec<&str>>> = Rc::new(RefCell::new(vec![
        "Marill water",
        "Magikarp fishing caught",
    ]));
    let made = |census: bool| {
        let logged: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let log: Log = {
            let logged = logged.clone();
            Rc::new(move |line: &str| logged.borrow_mut().push(line.to_string()))
        };
        let profile = Gen6Profile::new(
            Phased::new(town.memory.clone()),
            Rc::new(ClassNames::new(town.memory.clone(), CODE)),
            Rc::new(Speech::plain(Recorder::new())),
            log,
            oras::game(),
            Options {
                census,
                ..Options::default()
            },
        );
        // Python's test puts its own `touch` in; here the navigator's touch mode is given the
        // targets.
        let offering = offered.clone();
        profile
            .navigator
            .borrow_mut()
            .touch
            .as_mut()
            .expect("touch mode")
            .targets = Rc::new(move || {
            Ok(offering
                .borrow()
                .iter()
                .map(|name| Target::new(0, name, 0, 0, 0.0, 0.0))
                .collect())
        });
        (profile, logged)
    };
    let touched = |logged: &Rc<RefCell<Vec<String>>>| -> Vec<String> {
        logged
            .borrow()
            .iter()
            .filter(|line| line.starts_with("touch"))
            .cloned()
            .collect()
    };
    let (replay, logged) = made(true);
    replay.census().unwrap();
    replay.census().unwrap();
    offered.borrow_mut().clear();
    replay.census().unwrap();
    assert_eq!(
        touched(&logged),
        [
            "touch targets: Marill water | Magikarp fishing caught",
            "touch targets: none"
        ]
    );
    let (player, logged) = made(false);
    player.census().unwrap();
    assert!(touched(&logged).is_empty());
}

/// The runtime watches every heap a sweep finds (ADR-0034), and until 2026-09-28 the heap's
/// class was swept for only because the DexNav's record is found by it: X would have lost its
/// heaps' watch with ORAS's DexNav.
#[test]
fn test_every_gen_6_game_sweeps_for_its_heaps_and_only_oras_for_the_dexnav() {
    let town = Town::new();
    let classes_of = |game| {
        Gen6Profile::new(
            Phased::new(town.memory.clone()),
            Rc::new(ClassNames::new(town.memory.clone(), CODE)),
            Rc::new(Speech::plain(Recorder::new())),
            no_log(),
            game,
            Options::default(),
        )
        .classes
    };
    let oras = classes_of(oras::game());
    let x = classes_of(xy::game());
    let has = |classes: &[String], class: &str| classes.iter().any(|held| held == class);
    assert!(has(&oras, gflheap::CLASS) && has(&x, gflheap::CLASS));
    assert!(has(&oras, PLAYER_WORK_CLASS) && !has(&x, PLAYER_WORK_CLASS));
    let distinct: std::collections::HashSet<&String> = oras.iter().collect();
    assert_eq!(distinct.len(), oras.len());
}

/// As the bottom screen's arrow shows it, and not while the player is still turning
/// (spike 166).
#[test]
fn test_soaring_the_way_the_player_faces_is_said_once_a_turn_stops() {
    let town = Town::new();

    /// Python's `Soaring(Oras)`: ORAS's own, its soaring over Littleroot alone.
    fn soaring(kit: &Kit) -> Rc<dyn Own> {
        let mut oras = Oras::new(kit);
        let littleroot = sky::Place::new(
            "Littleroot Town",
            sky::TOWNS,
            0x6,
            vec![(-4000.0, 8000.0, -2000.0, 10000.0)],
        );
        oras.apps = vec![Rc::new(sky::Soaring::new(
            kit.memory.clone(),
            vec![littleroot],
            kit.speech.clone(),
            kit.log.clone(),
        ))];
        Rc::new(oras)
    }

    let game = Game {
        own: soaring,
        ..oras::game().clone()
    };
    let (made, backend) = profile_of(
        &town,
        GameText::new(places(), trainers()),
        None,
        no_log(),
        &game,
    );
    let (process, camera) = (0x08C65224, 0x0823C960);
    {
        let mut apps = made.apps.borrow_mut();
        apps.running = Some(sky::APP.to_string());
        apps.process = Some(process);
    }
    town.memory.u32(process + sky::CAMERA, &[camera]);
    let located: Vec<u8> = [-3000.0f32, 1900.0, 9000.0]
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect();
    town.memory.write(camera + sky::X, &located);

    let heading = |radians: f64| {
        town.memory
            .write(camera + sky::HEADING, &(radians as f32).to_le_bytes());
        made.poll_field().unwrap();
    };
    heading(std::f64::consts::PI);
    heading(std::f64::consts::PI);
    heading(2.0);
    heading(std::f64::consts::PI / 2.0);
    heading(std::f64::consts::PI / 2.0);
    assert_eq!(backend.spoken(), ["facing north", "facing east"]);
}

/// `[pa3ds.amie] head_it_cue` reaches Head It as the game's "!" (`CUE_SOUND`), in the left
/// ear for the left Pokémon, both for the middle and the right for the right (ADR-0025 as
/// amended 2026-11-01), and nothing is cued with the setting off: the spike that timed the cue
/// turned it on itself, and so never went through the setting ("it didn't cue me
/// at all").
///
/// Python's fakes the sounds and plays the cue toward each column; `Clips` here plays through
/// this machine's sound device, so the cue is found and not played, and where each column's is
/// heard is `stereo`'s, which test_footsteps checks.
#[test]
fn test_head_its_cue_is_the_games_exclamation_once_the_setting_is_on() {
    let sounds = Clips::new(Loudness::full(), 1.0);
    sounds.add(
        CUE_SOUND,
        Sound {
            rate: 32728,
            samples: Vec::new(),
            volume: 1.0,
            quietest: 1.0,
        },
    );
    let sounds = Rc::new(sounds);
    for on in [true, false] {
        let memory = FakeMemory::new();
        let made = Gen6Profile::new(
            Phased::new(memory.clone()),
            Rc::new(ClassNames::new(memory.clone(), CODE)),
            Rc::new(Speech::plain(Recorder::new())),
            no_log(),
            oras::game(),
            Options {
                tables: Some(Tables {
                    amie: Amie {
                        head_it_cue: on,
                        ..Amie::default()
                    },
                    ..Tables::default()
                }),
                sounds: Some(Rc::new(FeedbackSounds::new(
                    no_log(),
                    Rc::new(Speech::plain(Recorder::new())),
                    Some(sounds.clone() as Rc<dyn Plays>),
                    None,
                ))),
                ..Options::default()
            },
        );
        let head_it = made.subscreens.head_it.clone();
        assert_eq!(head_it.borrow().cue.is_some(), on, "on {on}");
    }
}

/// Move to forget gives way to the screen it was opened from, whose opening is left unsaid
/// under the game's question or its news of the move: until the page after that one, or a list
/// or a choice to answer, or the field (or_move_reminder).
#[test]
fn test_back_from_move_to_forget_lasts_until_a_page_after_the_first_or_an_answer() {
    let after = AfterForget::default();
    assert!(!after.back());
    after.started(Some(rows::FORGET), reminder::REMEMBER);
    assert!(after.back());
    // Written text is no page; the question is the first.
    after.paged(&[Shown::Text(TextWritten {
        window: 1,
        text: "Altaria".to_string(),
    })]);
    after.paged(&[Shown::Page(PageShown::new(
        2,
        0,
        "Give up trying to make Altaria remember a move?",
    ))]);
    assert!(after.back());
    after.paged(&[Shown::Page(PageShown::new(
        2,
        1,
        "Altaria did not learn Dragon Pulse.",
    ))]);
    assert!(!after.back());
    after.started(Some(rows::FORGET), partyicons::BAG);
    after.started(Some(partyicons::BAG), BATTLE);
    assert!(after.back());
    after.answered();
    assert!(!after.back());
    for started in [FIELD, rows::FORGET] {
        after.started(Some(rows::FORGET), reminder::REMEMBER);
        after.started(Some(reminder::REMEMBER), started);
        assert!(!after.back(), "{started}");
    }
}
