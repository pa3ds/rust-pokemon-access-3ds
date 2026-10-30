//! Pokémon-Amie's Berry Picker: each bubble's berry is said with the Pokémon under it, and offered
//! there to let a dragged berry go on.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_battle::record_of;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::field::keys::{BERRIES, BERRY_PICKER, POKEMON};
use crate::games::common::layout::Layouts;
use crate::games::common::touch::Target;
use crate::games::common::{lyt, pokemon, strbuf};
use crate::games::gen6::subscreen::berry_picker;
use crate::py::{Error, Kind, Result};

const MODEL: i64 = 0x08B00000;
const PARAM: i64 = 0x08B10000;
const DEMO: i64 = 0x08B20000;
const TEXT: i64 = 0x08B21000;
const APP: i64 = 0x08B30000;
const TREECKO: i64 = 252;
const TORCHIC: i64 = 255;
const MUDKIP: i64 = 258;

/// `NAMES.get(found.species)`.
fn named(species: i64) -> Option<String> {
    match species {
        TREECKO => Some("Treecko".to_string()),
        TORCHIC => Some("Torchic".to_string()),
        MUDKIP => Some("Mudkip".to_string()),
        _ => None,
    }
}

/// Easy's tree as touch mode offers it (as_amie_berry_picker).
fn tree() -> Vec<Target> {
    ["Razz Berry", "Bluk Berry", "Pinap Berry"]
        .iter()
        .enumerate()
        .map(|(i, name)| Target::new(10 + i as i64, name, 52 + 108 * i as i64, 40, 80.0, 40.0))
        .collect()
}

fn words() -> IndexMap<String, String> {
    [
        ("kinomi00.bclim", "Razz Berry"),
        ("kinomi01.bclim", "Bluk Berry"),
        ("kinomi02.bclim", "Pinap Berry"),
        ("kinomi03.bclim", "Wepear Berry"),
    ]
    .into_iter()
    .map(|(image, word)| (image.to_string(), word.to_string()))
    .collect()
}

/// The fake memory, failing every read past 0xe0000000 once `mapped` is set: Python's test
/// patches `memory.read` so.
struct Mapped {
    memory: Rc<FakeMemory>,
    mapped: Cell<bool>,
}

impl Memory for Mapped {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        if self.mapped.get() && address >= 0xE0000000 {
            return Err(Error::new(
                Kind::RpcError,
                format!("page {:#x} unread", address & !0xFFF),
            ));
        }
        self.memory.read(address, size)
    }
}

struct Picker {
    found: berry_picker::BerryPicker,
    memory: Rc<FakeMemory>,
    reads: Rc<Mapped>,
    panes: Vec<i64>,
    layouts: Rc<Layouts>,
}

/// What Berry Picker says at a poll, the lines alone.
fn heard(found: &mut berry_picker::BerryPicker, running: Option<&str>) -> Vec<String> {
    found
        .poll(running)
        .unwrap()
        .into_iter()
        .map(|(_, line, _)| line)
        .collect()
}

/// A bubble on the left showing a Pinap Berry, a spare bubble parked off the screen, a berry
/// in a tree outside any bubble, as the game's opening draws in the bubbles' layout (spike 260),
/// and Treecko standing on the left: the picker, the memory, the bubble layout's panes in order
/// (the bubble, its berry, the spare, its berry and the tree's) and the layouts. With `demo`, the
/// opening's demonstration, saying it is over with 1 and not with 0.
fn picker(demo: Option<i64>) -> Picker {
    picker_with(demo, Rc::new(|| 0.0))
}

/// `picker(demo, clock)`.
fn picker_with(demo: Option<i64>, clock: Rc<dyn Fn() -> f64>) -> Picker {
    picker_saying(demo, clock, false, false)
}

/// `picker(demo, clock, berry_first, one_at_a_time)`.
fn picker_saying(
    demo: Option<i64>,
    clock: Rc<dyn Fn() -> f64>,
    berry_first: bool,
    one_at_a_time: bool,
) -> Picker {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let picture = classes.add(lyt::PICTURE_CLASS);
    let fake = FakeLayouts::new(&memory, &classes);
    let (_, group) = fake.screen(0x14B00000);
    memory.u32(group - 0x200 + lyt::CHILD_COUNT, &[1]);
    memory.u32(group + lyt::CHILD_COUNT, &[5]);
    let panes: Vec<i64> = (1..6).map(|i| group + 0x100 * i).collect();
    for (&pane, (x, y)) in panes.iter().zip([
        (-104.0f32, -80.0f32),
        (-104.0, -30.0),
        (-110.0, -210.0),
        (-110.0, -160.0),
        (0.0, 80.0),
    ]) {
        fake.pane_with_vtable(pane, group, Some(picture));
        memory.write(pane + lyt::X, &x.to_le_bytes());
        memory.write(pane + lyt::Y, &y.to_le_bytes());
    }
    fake.track().unwrap();
    let drawn: IndexMap<i64, &str> = panes
        .iter()
        .copied()
        .zip([
            "balloon00.bclim",
            "kinomi02.bclim",
            "balloon00.bclim",
            "kinomi00.bclim",
            "kinomi01.bclim",
        ])
        .collect();
    memory.u32(MODEL, &[classes.add(berry_picker::MODEL)]);
    let param_class = classes.add(pokemon::PARAM_CLASS);
    memory.u32(PARAM, &[param_class, PARAM + pokemon::RECORD_IN_PARAM]);
    memory.write(PARAM + pokemon::RECORD_IN_PARAM, &record_of("", 0, TREECKO));
    for (i, (x, y)) in [(0.0f32, -480.0f32), (-120.0, 15.0), (120.0, -255.0)]
        .into_iter()
        .enumerate()
    {
        let at = MODEL + berry_picker::RECORDS + i as i64 * berry_picker::SIZE;
        memory.u32(
            at + berry_picker::PARAM,
            &[if i == 1 { PARAM } else { PARAM + 0x1000 }],
        );
        let mut place = x.to_le_bytes().to_vec();
        place.extend(y.to_le_bytes());
        memory.write(at + berry_picker::PLACE, &place);
    }
    let reads = Rc::new(Mapped {
        memory: memory.clone(),
        mapped: Cell::new(false),
    });
    let mut found = berry_picker::BerryPicker::with_settings(
        reads.clone(),
        Rc::new(ClassNames::new(reads.clone(), CODE)),
        fake.layouts.clone(),
        Rc::new(move |pane: i64| Ok(drawn.get(&pane).map(|image| image.to_string()))),
        words(),
        Rc::new(|found: &pokemon::Pokemon| named(found.species)),
        clock,
        berry_first,
        one_at_a_time,
    );
    memory.u32(APP, &[classes.add(berry_picker::APP)]);
    memory.u32(APP + berry_picker::CLOCK, &[1800]);
    let mut tracked = IndexMap::from([
        (MODEL, berry_picker::MODEL.to_string()),
        (APP, berry_picker::APP.to_string()),
    ]);
    if let Some(demo) = demo {
        memory.u32(DEMO, &[classes.add(berry_picker::DEMO), TEXT]);
        memory.u32(TEXT, &[classes.add(strbuf::CLASS)]);
        memory.u32(DEMO + berry_picker::OVER, &[demo]);
        tracked.insert(DEMO, berry_picker::DEMO.to_string());
    }
    found.track(&tracked, false).unwrap();
    Picker {
        found,
        memory,
        reads,
        panes,
        layouts: fake.layouts.clone(),
    }
}

/// ADR-0009: let go at its body, the berry was eaten (spike 260, live).
#[test]
fn test_a_bubbles_berry_is_offered_at_the_pokemon_under_it_by_the_games_names() {
    let picked = picker(None);
    assert_eq!(
        picked.found.targets(Some(berry_picker::APP)).unwrap(),
        vec![Target::new(
            picked.panes[1],
            "Treecko wants Pinap Berry",
            56,
            berry_picker::BODY,
            -95.0,
            50.0
        )]
    );
    assert_eq!(
        picked.found.targets(Some("N3app6puzzle7ProcessE")).unwrap(),
        vec![]
    );
}

#[test]
fn test_a_bubble_is_said_as_it_comes_and_again_once_it_has_gone_and_come_back() {
    let mut picked = picker(None);
    let app = Some(berry_picker::APP);
    assert_eq!(heard(&mut picked.found, app), ["Treecko wants Pinap Berry"]);
    assert_eq!(heard(&mut picked.found, app), Vec::<String>::new());
    picked
        .memory
        .write(picked.panes[1] + lyt::X, &(-300.0f32).to_le_bytes());
    picked.layouts.refresh();
    assert_eq!(heard(&mut picked.found, app), Vec::<String>::new());
    picked
        .memory
        .write(picked.panes[1] + lyt::X, &(-104.0f32).to_le_bytes());
    picked.layouts.refresh();
    assert_eq!(heard(&mut picked.found, app), ["Treecko wants Pinap Berry"]);
}

/// The opening shows three Pokémon wanting berries before READY: the game demonstrating,
/// silent (Orion, 2026-10-28).
#[test]
fn test_the_openings_demonstration_is_said_nothing_of_and_the_round_is() {
    let app = Some(berry_picker::APP);
    let mut found = picker(Some(0)).found;
    assert_eq!(heard(&mut found, app), Vec::<String>::new());
    assert_eq!(found.targets(app).unwrap(), vec![]);
    assert_eq!(
        heard(&mut picker(Some(1)).found, app),
        ["Treecko wants Pinap Berry"]
    );
}

/// The sweep found two more Models in the module's code, holding instructions where a
/// record's pointers are, and a read through 0xebffe000 failed every poll of the round
/// (as_amie_berry_picker's tape).
#[test]
fn test_a_model_found_in_the_modules_code_is_not_read_through() {
    let mut picked = picker(None);
    let memory = &picked.memory;
    let r#false = MODEL + 0x1000;
    memory.write(r#false, &memory.read(MODEL, 4).unwrap());
    let record = r#false + berry_picker::RECORDS;
    memory.u32(record + berry_picker::PARAM, &[0xEBFFE000]);
    let mut place = (-120.0f32).to_le_bytes().to_vec();
    place.extend(20.0f32.to_le_bytes());
    memory.write(record + berry_picker::PLACE, &place);
    picked.reads.mapped.set(true);
    picked
        .found
        .track(
            &IndexMap::from([
                (MODEL, berry_picker::MODEL.to_string()),
                (r#false, berry_picker::MODEL.to_string()),
            ]),
            false,
        )
        .unwrap();
    assert_eq!(
        heard(&mut picked.found, Some(berry_picker::APP)),
        ["Treecko wants Pinap Berry"]
    );
}

#[test]
fn test_a_bubble_over_no_pokemon_that_can_be_read_says_its_berry_alone() {
    let mut picked = picker(None);
    picked.memory.u32(
        MODEL + berry_picker::RECORDS + berry_picker::SIZE + berry_picker::PARAM,
        &[0],
    );
    assert_eq!(
        heard(&mut picked.found, Some(berry_picker::APP)),
        ["Pinap Berry wanted"]
    );
}

/// Touch mode as Berry Picker's keys ask it: the tree's berries and the wants, and each drag given.
struct Touched {
    picker: Rc<RefCell<berry_picker::BerryPicker>>,
    given: Vec<(String, String)>,
}

impl berry_picker::Touching for Touched {
    fn targets(&mut self) -> Result<Vec<Target>> {
        let mut found = tree();
        found.extend(self.picker.borrow().targets(Some(berry_picker::APP))?);
        Ok(found)
    }

    fn give(&mut self, picked: &Target, over: &Target) -> String {
        self.given.push((picked.name.clone(), over.name.clone()));
        format!("let go on {}", over.name)
    }
}

struct Keys {
    pressing: berry_picker::BerryKeys,
    touched: Touched,
    played: Rc<RefCell<Vec<String>>>,
    memory: Rc<FakeMemory>,
    panes: Vec<i64>,
    layouts: Rc<Layouts>,
}

impl Keys {
    fn press(&mut self, action: &str) -> String {
        self.pressing
            .press(action, Some(berry_picker::APP), &mut self.touched)
            .unwrap()
    }
}

/// Berry Picker's keys over `picker`'s round, Treecko the second of its three and the other two
/// not to be read: the keys, touch mode, the sounds played, and the round's memory, panes and
/// layouts. With `sounds`, the game's own are heard in place of the words.
fn keys(sounds: bool) -> Keys {
    let picked = picker(None);
    let found = Rc::new(RefCell::new(picked.found));
    let touched = Touched {
        picker: found.clone(),
        given: Vec::new(),
    };
    let played: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let heard = {
        let played = played.clone();
        Rc::new(move |sound: &str, said: String| -> String {
            if sounds {
                played.borrow_mut().push(sound.to_string());
                return String::new();
            }
            said
        })
    };
    let pressing = berry_picker::BerryKeys::new(
        found,
        words(),
        {
            let heard = heard.clone();
            Rc::new(move |berry: &str| heard("pick", format!("holding {berry}")))
        },
        Rc::new(move |why: &str| heard("wrong", why.to_string())),
    );
    Keys {
        pressing,
        touched,
        played,
        memory: picked.memory,
        panes: picked.panes,
        layouts: picked.layouts,
    }
}

/// ADR-0009 as amended 2026-10-29 (Orion): a key a berry, in the game's own order of their
/// pictures, and a key a Pokémon, in the game's own order of the round's three; the second makes
/// the drag, from the tree straight to the Pokémon.
#[test]
fn test_a_berrys_key_then_a_pokemons_gives_it_the_berry() {
    let mut keys = keys(false);
    assert_eq!(
        keys.pressing.context(Some(berry_picker::APP)),
        Some(BERRY_PICKER)
    );
    assert_eq!(keys.pressing.context(Some("N3app6puzzle7ProcessE")), None);
    assert_eq!(keys.press(BERRIES[2]), "holding Pinap Berry");
    assert!(keys.touched.given.is_empty());
    assert_eq!(
        keys.press(POKEMON[1]),
        "let go on Treecko wants Pinap Berry"
    );
    assert_eq!(
        keys.touched.given,
        [(
            "Pinap Berry".to_string(),
            "Treecko wants Pinap Berry".to_string()
        )]
    );
    assert_eq!(keys.press(POKEMON[1]), "Treecko wants Pinap Berry");
    assert_eq!(keys.touched.given.len(), 1);
}

/// Pressed 1, then 2, the player means to give the second (Orion, 2026-10-29).
#[test]
fn test_another_berrys_key_picks_that_one_up_instead() {
    let mut keys = keys(false);
    keys.press(BERRIES[0]);
    keys.press(BERRIES[2]);
    keys.press(POKEMON[1]);
    assert_eq!(
        keys.touched.given,
        [(
            "Pinap Berry".to_string(),
            "Treecko wants Pinap Berry".to_string()
        )]
    );
}

/// A berry the round has not, as 7 on Easy, a Pokémon not to be read, or one not up with a berry
/// picked up, which stays picked up: the game's own beep, `SEQ_SE_BEEP`,
/// with `feedback_sounds` on (Orion, 2026-10-29).
#[test]
fn test_a_key_with_nothing_to_do_plays_the_games_error_sound_or_says_why() {
    let mut keys = keys(false);
    assert_eq!(keys.press(BERRIES[3]), "no berry 4");
    assert_eq!(keys.press(BERRIES[6]), "no berry 7");
    assert_eq!(keys.press(POKEMON[0]), "no Pokémon 1");
    keys.press(BERRIES[2]);
    keys.memory
        .write(keys.panes[1] + lyt::X, &(-300.0f32).to_le_bytes());
    keys.layouts.refresh();
    assert_eq!(keys.press(POKEMON[1]), "Treecko is not up");
    keys.memory
        .write(keys.panes[1] + lyt::X, &(-104.0f32).to_le_bytes());
    keys.layouts.refresh();
    assert_eq!(
        keys.press(POKEMON[1]),
        "let go on Treecko wants Pinap Berry"
    );
    assert!(
        keys.touched.given
            == [(
                "Pinap Berry".to_string(),
                "Treecko wants Pinap Berry".to_string()
            )]
            && keys.played.borrow().is_empty()
    );
    let mut sounding = self::keys(true);
    assert!(sounding.press(BERRIES[6]).is_empty());
    assert!(sounding.press(BERRIES[2]).is_empty());
    assert_eq!(*sounding.played.borrow(), ["wrong", "pick"]);
}

/// A player lowers Azahar's speed limit for a higher score, and a drag is timed by the game's
/// frames: the round's clock counts 30 a second at full speed (spike 262).
#[test]
fn test_the_rounds_clock_says_how_fast_the_game_runs() {
    let now = Rc::new(Cell::new(0.0));
    let clock: Rc<dyn Fn() -> f64> = {
        let now = now.clone();
        Rc::new(move || now.get())
    };
    let mut picked = picker_with(None, clock);
    let app = Some(berry_picker::APP);
    assert_eq!(picked.found.speed(), 1.0);
    heard(&mut picked.found, app);
    now.set(1.0);
    picked.memory.u32(APP + berry_picker::CLOCK, &[1800 - 9]);
    heard(&mut picked.found, app);
    assert_eq!(picked.found.speed(), 0.3);
    now.set(2.0);
    heard(&mut picked.found, app);
    assert_eq!(picked.found.speed(), 0.3);
    // A copy in the module's code, found first, holds an instruction where the clock is.
    let r#false = APP - 0x1000;
    picked
        .memory
        .write(r#false, &picked.memory.read(APP, 4).unwrap());
    picked
        .memory
        .u32(r#false + berry_picker::CLOCK, &[0xE59F0010]);
    picked
        .found
        .track(
            &IndexMap::from([
                (MODEL, berry_picker::MODEL.to_string()),
                (r#false, berry_picker::APP.to_string()),
                (APP, berry_picker::APP.to_string()),
            ]),
            false,
        )
        .unwrap();
    for (at, frames) in [(3.0, 1800 - 9 - 15), (4.0, 1800 - 9 - 45)] {
        now.set(at);
        picked.memory.u32(APP + berry_picker::CLOCK, &[frames]);
        heard(&mut picked.found, app);
    }
    assert_eq!(picked.found.speed(), 1.0);
}

/// Orion, 2026-10-29: pressing the keys as a round begins learns them too late, so the three are
/// said in the game's own order, the keys', once while the clock stands full before START!, and
/// again before a Play Again's round; with one of them not to be read, not at all.
#[test]
fn test_the_rounds_three_pokemon_are_said_in_their_keys_order_before_it_starts() {
    let mut picked = picker(None);
    let app = Some(berry_picker::APP);
    assert!(
        !heard(&mut picked.found, app)
            .iter()
            .any(|line| line.contains(" and "))
    );
    let memory = picked.memory.clone();
    for (i, species) in [(0, TORCHIC), (2, MUDKIP)] {
        let param = PARAM + 0x1000 * (i + 1);
        memory.write(param, &memory.read(PARAM, 4).unwrap());
        memory.u32(param + 4, &[param + pokemon::RECORD_IN_PARAM]);
        memory.write(param + pokemon::RECORD_IN_PARAM, &record_of("", 0, species));
        memory.u32(
            MODEL + berry_picker::RECORDS + i * berry_picker::SIZE + berry_picker::PARAM,
            &[param],
        );
    }
    let lineup = "Torchic, Treecko and Mudkip".to_string();
    memory.u32(APP + berry_picker::CLOCK, &[1799]);
    heard(&mut picked.found, app);
    memory.u32(APP + berry_picker::CLOCK, &[1800]);
    assert_eq!(
        &heard(&mut picked.found, app)[..1],
        std::slice::from_ref(&lineup)
    );
    assert!(!heard(&mut picked.found, app).contains(&lineup));
    memory.u32(APP + berry_picker::CLOCK, &[1200]);
    assert!(!heard(&mut picked.found, app).contains(&lineup));
    memory.u32(APP + berry_picker::CLOCK, &[1800]);
    assert_eq!(
        &heard(&mut picked.found, app)[..1],
        std::slice::from_ref(&lineup)
    );
    assert_eq!(
        heard(&mut picked.found, Some("N3app6puzzle7ProcessE")),
        Vec::<String>::new()
    );
}

/// Torchic up in the middle wanting a Razz Berry, the spare bubble brought on, the round under way.
fn second_want(memory: &FakeMemory, panes: &[i64], layouts: &Layouts) {
    let param = PARAM + 0x1000;
    memory.write(param, &memory.read(PARAM, 4).unwrap());
    memory.u32(param + 4, &[param + pokemon::RECORD_IN_PARAM]);
    memory.write(param + pokemon::RECORD_IN_PARAM, &record_of("", 0, TORCHIC));
    let mut place = 0.0f32.to_le_bytes().to_vec();
    place.extend(15.0f32.to_le_bytes());
    memory.write(MODEL + berry_picker::RECORDS + berry_picker::PLACE, &place);
    for (pane, y) in [(panes[2], -80.0f32), (panes[3], -30.0f32)] {
        memory.write(pane + lyt::X, &0.0f32.to_le_bytes());
        memory.write(pane + lyt::Y, &y.to_le_bytes());
    }
    layouts.refresh();
    memory.u32(APP + berry_picker::CLOCK, &[1700]);
}

/// Orion, 2026-10-29: a berry is given by its key and then the Pokémon's (`[pa3ds.amie]
/// berry_first`).
#[test]
fn test_berry_first_says_a_want_in_the_order_its_berry_is_given() {
    let mut picked = picker_saying(None, Rc::new(|| 0.0), true, false);
    assert_eq!(
        heard(&mut picked.found, Some(berry_picker::APP)),
        ["Pinap Berry for Treecko"]
    );
}

/// Orion, 2026-10-29: three wants said together took 1.5 s, the first all a player can act on
/// (`[pa3ds.amie] one_at_a_time`); without it each is said as it comes.
#[test]
fn test_one_at_a_time_the_next_is_said_once_a_berry_is_given_or_the_one_said_goes() {
    let app = Some(berry_picker::APP);
    let mut picked = picker(None);
    second_want(&picked.memory, &picked.panes, &picked.layouts);
    assert_eq!(
        heard(&mut picked.found, app),
        ["Treecko wants Pinap Berry", "Torchic wants Razz Berry"]
    );
    let mut picked = picker_saying(None, Rc::new(|| 0.0), false, true);
    second_want(&picked.memory, &picked.panes, &picked.layouts);
    assert_eq!(heard(&mut picked.found, app), ["Treecko wants Pinap Berry"]);
    assert_eq!(heard(&mut picked.found, app), Vec::<String>::new());
    picked.found.given();
    assert_eq!(heard(&mut picked.found, app), ["Torchic wants Razz Berry"]);
    assert_eq!(heard(&mut picked.found, app), Vec::<String>::new());
    let mut picked = picker_saying(None, Rc::new(|| 0.0), false, true);
    second_want(&picked.memory, &picked.panes, &picked.layouts);
    heard(&mut picked.found, app);
    picked
        .memory
        .write(picked.panes[1] + lyt::X, &(-300.0f32).to_le_bytes());
    picked.layouts.refresh();
    assert_eq!(heard(&mut picked.found, app), ["Torchic wants Razz Berry"]);
}
