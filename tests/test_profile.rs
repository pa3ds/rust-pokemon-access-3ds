//! The ORAS profile's use of the game's own words: a place said on arriving, the rival, where
//! an exit leads, and the PC.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeMemory};
use super::fake_rom::TmpDir;
use super::fake_speech::Recorder;
use super::fake_words::words_in;
use super::test_field::{
    Given, HOUSE, PC_TILE, POOCHYENA, ROUTE, ROUTE_BLOCK, TABLE, TOWN, TOWN_BLOCK, TOWN_CHUNK,
    Town, border, dex_nav, field_reader, game,
};
use crate::core::bcsar::Sound;
use crate::core::clips::Clips;
use crate::core::config::{Amie, Tables};
use crate::core::loudness::Loudness;
use crate::core::memory::{Cache, Memory};
use crate::core::names::Names;
use crate::core::rtti::ClassNames;
use crate::core::speech::Speech;
use crate::games::common::field::keys::{ENEMY_SIDE, ITEM_NEXT, MODE_TOGGLE, OUR_SIDE, PATHFIND};
use crate::games::common::field::things::{Interactable, WARP_SOURCE};
use crate::games::common::session::Ways;
use crate::games::common::touch::Target;
use crate::games::gen6::apps::{BATTLE, DEMO, FIELD};
use crate::games::gen6::field::mapblock;
use crate::games::gen6::field::mmodel::PLAYER_INDEX;
use crate::games::gen6::field::zonetable::{INFO_NAME, INFO_SIZE};
use crate::games::gen6::game::{Game, OwnApp};
use crate::games::gen6::profile::{CUE_SOUND, Gen6Profile, Options};
use crate::games::gen6::rom::archives::{GameText, no_text};
use crate::games::oras::field::hidden::PLAYER_WORK_CLASS;
use crate::games::{gen6::gflheap, oras::field::sky};
use crate::games::{oras, xy};
use crate::py::{Log, Result};

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

fn no_log() -> Log {
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
fn profile_of(
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

/// The game turns the place's name when the place changes, not the zone: walking out of a house
/// into its own town arrives nowhere (spike 87).
#[test]
fn test_the_place_is_said_on_arriving_and_not_on_walking_into_a_building() {
    let town = Town::new();
    let (made, backend) = profile(&town, GameText::new(places(), trainers()), None, no_log());
    made.poll_field().unwrap();
    assert_eq!(backend.spoken(), ["Littleroot Town"]);
    town.model(0, HOUSE, PLAYER_INDEX, 0xAC, (5, 5));
    made.poll_field().unwrap();
    town.model(0, TOWN, PLAYER_INDEX, 0xAC, (94, 172));
    made.poll_field().unwrap();
    assert_eq!(backend.spoken(), ["Littleroot Town"]);
    town.model(0, TOWN, PLAYER_INDEX, 0xAC, (99, 150));
    made.poll_field().unwrap();
    assert_eq!(backend.spoken(), ["Littleroot Town", "Route 101"]);
}

/// ADR-0042: a part of a place in parts is said as the player walks into it, from another
/// place or the next part, and not on walking out of a house back into it; and an exit into a
/// part says the part, the way out of a house of the place too ("way out to
/// South Side" said nothing of where). Here the town's two zones are given parts of their own.
#[test]
fn test_a_part_of_a_place_is_said_walking_into_it_and_by_the_exits_into_it() {
    let tmp_path = TmpDir::new();
    words_in(
        tmp_path.path(),
        &format!(
            "part-{TOWN:#x} = Northern {{ $place }}\npart-{ROUTE:#x} = Southern {{ $place }}\n"
        ),
        "oras",
        "en",
        "zones.ftl",
    );
    let town = Town::new();
    let (made, backend) = profile(&town, GameText::new(places(), trainers()), None, no_log());
    made.poll_field().unwrap();
    assert_eq!(backend.spoken(), ["Northern Littleroot Town"]);
    town.model(0, HOUSE, PLAYER_INDEX, 0xAC, (5, 5));
    made.poll_field().unwrap();
    let door = Interactable {
        zone: Some(TOWN),
        ..Interactable::new("exits", "door", (5, 6))
    };
    assert_eq!(
        made.navigator
            .borrow()
            .destination(&door, HOUSE)
            .unwrap()
            .as_deref(),
        Some("Northern Littleroot Town")
    );
    town.model(0, TOWN, PLAYER_INDEX, 0xAC, (94, 172));
    made.poll_field().unwrap();
    assert_eq!(backend.spoken(), ["Northern Littleroot Town"]);
    let edge = Interactable {
        zone: Some(ROUTE),
        ..Interactable::new("exits", "edge", (95, 159))
    };
    assert_eq!(
        made.navigator
            .borrow()
            .destination(&edge, TOWN)
            .unwrap()
            .as_deref(),
        Some("Southern Route 101")
    );
    town.model(0, TOWN, PLAYER_INDEX, 0xAC, (99, 150));
    made.poll_field().unwrap();
    assert_eq!(
        backend.spoken(),
        ["Northern Littleroot Town", "Southern Route 101"]
    );
}

/// ADR-0042: a thing standing in another part of the player's place says the part; one in the
/// player's own part, an exit into that part, and a place of no parts say nothing more.
#[test]
fn test_a_thing_of_another_part_is_said_with_its_part() {
    let tmp_path = TmpDir::new();
    words_in(
        tmp_path.path(),
        &format!(
            "part-{TOWN:#x} = Northern {{ $place }}\npart-{ROUTE:#x} = Southern {{ $place }}\n"
        ),
        "oras",
        "en",
        "zones.ftl",
    );
    let mut town = Town::new();
    let both = vec![TOWN, ROUTE];
    town.reader = field_reader(
        town.memory.clone(),
        Given {
            parts: IndexMap::from([(TOWN, both.clone()), (ROUTE, both.clone())]),
            ..Given::default()
        },
        Game {
            hidden: Some(dex_nav),
            ..game()
        },
    );
    town.memory.u16(TABLE + ROUTE * INFO_SIZE + INFO_NAME, 170);
    let (made, _) = profile(&town, GameText::new(places(), trainers()), None, no_log());
    made.poll_field().unwrap();
    let ball = Interactable::new("items", "item ball", (95, 140));
    assert_eq!(
        made.part_of(&ball, TOWN).unwrap().as_deref(),
        Some("Southern Littleroot Town")
    );
    let here = Interactable {
        cell: (95, 170),
        ..ball.clone()
    };
    assert_eq!(made.part_of(&here, TOWN).unwrap(), None);
    let edge = Interactable {
        zone: Some(ROUTE),
        ..Interactable::new("exits", "edge", (99, 159))
    };
    assert_eq!(made.part_of(&edge, TOWN).unwrap(), None);
    town.reader = field_reader(
        town.memory.clone(),
        Given::default(),
        Game {
            hidden: Some(dex_nav),
            ..game()
        },
    );
    made.set_field(town.reader.clone());
    assert_eq!(made.part_of(&ball, TOWN).unwrap(), None);
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

/// AreaNav's words still say it, a moment later, as they always have.
#[test]
fn test_without_the_rom_no_place_is_said_from_the_field() {
    let town = Town::new();
    let (made, backend) = profile(&town, no_text(), None, no_log());
    made.poll_field().unwrap();
    assert!(backend.spoken().is_empty());
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

/// The ROM names the building behind warp 0; sign 0 on the same map is not that building
/// (the wally.ctm replay of 2026-09-23 read Petalburg's signs as its Gym, Mart and Center).
#[test]
#[allow(clippy::identity_op)]
fn test_the_roms_door_name_is_a_doors_and_never_a_signs() {
    let town = Town::new();
    let named = names(&[]);
    let text = GameText {
        doors: IndexMap::from([((TOWN, 0), "Pokémon Center".to_string())]),
        ..GameText::new(places(), trainers())
    };
    profile(&town, text, Some(named.clone()), no_log());
    assert_eq!(
        named.place(TOWN, WARP_SOURCE | 0).as_deref(),
        Some("Pokémon Center")
    );
    assert_eq!(named.place(TOWN, 0), None);
}

/// Lumiose's five gates were each "warp to Lumiose Gate".
#[test]
#[allow(clippy::identity_op)]
fn test_a_warp_into_a_gate_is_called_the_gate_and_the_place_beyond_it() {
    let town = Town::new();
    let named = names(&[]);
    let text = GameText {
        gates: IndexMap::from([((TOWN, 0), (170, 204))]),
        ..GameText::new(places(), trainers())
    };
    profile(&town, text, Some(named.clone()), no_log());
    assert_eq!(
        named.place(TOWN, WARP_SOURCE | 0).as_deref(),
        Some("Littleroot Town to Route 101")
    );
}

/// Nothing said the gap north of Littleroot leads to Route 101. The
/// place is the zone table's index for each side and the ROM's word for it; the house behind the
/// first door shares the town's place, and the second door's zone has no place at all.
#[test]
fn test_an_exit_into_another_place_is_said_with_it_and_a_door_of_the_town_is_not() {
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

#[test]
fn test_without_the_rom_an_exit_is_only_what_it_is() {
    let town = Town::new();
    let (made, _) = profile(&town, no_text(), None, no_log());
    let labels: Vec<String> = town
        .r#where()
        .unwrap()
        .of("exits")
        .iter()
        .map(|item| made.navigator.borrow().label(item, TOWN).unwrap())
        .collect();
    assert_eq!(labels, ["door", "door", "path"]);
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

    fn soaring(
        memory: Rc<dyn Memory>,
        _text: &GameText,
        speech: Rc<Speech>,
        log: Log,
    ) -> Result<Option<Rc<dyn OwnApp>>> {
        let littleroot = sky::Place::new(
            "Littleroot Town",
            sky::TOWNS,
            0x6,
            vec![(-4000.0, 8000.0, -2000.0, 10000.0)],
        );
        Ok(Some(Rc::new(sky::Soaring::new(
            memory,
            vec![littleroot],
            speech,
            log,
        ))))
    }

    let game = Game {
        apps: vec![soaring],
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
                feedback_sounds: Some(sounds.clone()),
                ..Options::default()
            },
        );
        let head_it = made.subscreens.head_it.clone();
        assert_eq!(head_it.borrow().cue.is_some(), on, "on {on}");
    }
}

/// One model draws both of Route 111's fossils, and the game names them only as the item
/// taken: west the Root Fossil, east the Claw Fossil, listed under items (a player's report,
/// spike 233). Their names are the ROM's, in the player's language.
#[test]
fn test_route_111s_fossils_are_called_the_items_they_are_from_the_rom() -> Result<()> {
    use crate::games::common::field::things::{MODEL_SOURCE, place_source};
    use crate::games::gen6::profile::game_place;

    let mut items = vec![String::new(); 101];
    items[99] = "Wurzelfossil".to_string();
    items[100] = "Klauenfossil".to_string();
    let text = GameText {
        items,
        ..GameText::default()
    };
    let mut called = Vec::new();
    for record in [3, 4, 5] {
        called.push(game_place(
            &text,
            &oras::game().items_at,
            None,
            0x23,
            place_source(MODEL_SOURCE, record)?,
        ));
    }
    assert_eq!(
        called,
        [
            Some("Wurzelfossil".to_string()),
            Some("Klauenfossil".to_string()),
            None
        ]
    );
    assert!(oras::game().pickups.contains(&oras::FOSSIL));
    Ok(())
}
