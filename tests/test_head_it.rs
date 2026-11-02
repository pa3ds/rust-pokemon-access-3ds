//! Pokémon-Amie's Head It: each ball of yarn is said by the Pokémon it falls at, and each Pokémon
//! up is offered in touch mode to be tapped.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_battle::record_of;
use crate::core::rtti::ClassNames;
use crate::games::common::field::keys::{DEFAULT_CONTEXTS, HEAD_IT, HEADERS};
use crate::games::common::layout::Layouts;
use crate::games::common::touch::Target;
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::subscreen::berry_picker::Touching;
use crate::games::gen6::subscreen::head_it;
use crate::py::Result;

const MODEL: i64 = 0x08B00000;
const PARAM: i64 = 0x08B10000;
const APP: i64 = 0x08B30000;
const POOCHYENA: i64 = 261;
const RALTS: i64 = 280;
const TREECKO: i64 = 252;
const OTHER: &str = "N3app6puzzle7ProcessE";

/// `NAMES.get(found.species)`.
fn named(species: i64) -> Option<String> {
    match species {
        POOCHYENA => Some("Poochyena".to_string()),
        RALTS => Some("Ralts".to_string()),
        TREECKO => Some("rowan".to_string()),
        _ => None,
    }
}

struct Game {
    found: head_it::HeadIt,
    memory: Rc<FakeMemory>,
    panes: Vec<i64>,
    layouts: Rc<Layouts>,
}

fn game() -> Game {
    game_with(None, Rc::new(|| 0.0))
}

/// As as_amie_head_it's round went 30 s in: Poochyena down on the left, Ralts up in the
/// middle and rowan up on the right; a ball falling at the middle, the giant ball parked off to
/// the right and a ball past the bottom (spike A). The game, the memory, the balls' panes and the
/// layouts. With `cue`, the cue is on, sounding through it, timed by `clock`.
fn game_with(cue: Option<head_it::Cue>, clock: Rc<dyn Fn() -> f64>) -> Game {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let picture = classes.add(lyt::PICTURE_CLASS);
    let fake = FakeLayouts::new(&memory, &classes);
    let (_, group) = fake.screen(0x14B00000);
    memory.u32(group - 0x200 + lyt::CHILD_COUNT, &[1]);
    memory.u32(group + lyt::CHILD_COUNT, &[3]);
    let panes: Vec<i64> = (1..4).map(|i| group + 0x100 * i).collect();
    for (&pane, (x, y)) in panes
        .iter()
        .zip([(0.0f32, 452.0f32), (10500.0, 480.0), (100.0, -137.0)])
    {
        fake.pane_with_vtable(pane, group, Some(picture));
        place(&memory, pane, x, y);
    }
    fake.track().unwrap();
    let mut drawn: IndexMap<i64, &str> = panes
        .iter()
        .map(|&pane| (pane, "ball00_01.bclim"))
        .collect();
    drawn.insert(panes[1], "giant01.bclim");
    memory.u32(MODEL, &[classes.add(head_it::MODEL)]);
    let param_class = classes.add(pokemon::PARAM_CLASS);
    for (i, (species, x, y)) in [
        (POOCHYENA, -120.0f32, -200.0f32),
        (RALTS, 0.0, 25.0),
        (TREECKO, 120.0, 25.0),
    ]
    .into_iter()
    .enumerate()
    {
        let param = PARAM + 0x1000 * i as i64;
        memory.u32(param, &[param_class, param + pokemon::RECORD_IN_PARAM]);
        memory.write(param + pokemon::RECORD_IN_PARAM, &record_of("", 0, species));
        let at = MODEL + head_it::RECORDS + i as i64 * head_it::SIZE;
        memory.u32(at + head_it::PARAM, &[param]);
        let mut place = x.to_le_bytes().to_vec();
        place.extend(y.to_le_bytes());
        memory.write(at + head_it::PLACE, &place);
    }
    let mut found = head_it::HeadIt::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        fake.layouts.clone(),
        Rc::new(move |pane: i64| Ok(drawn.get(&pane).map(|image| image.to_string()))),
        Rc::new(|found: &pokemon::Pokemon| named(found.species)),
        cue,
        head_it::LEAD,
        clock,
    );
    memory.u32(APP, &[classes.add(head_it::APP)]);
    memory.u32(APP + head_it::PHASE, &[head_it::FALLING]);
    found
        .track(
            &IndexMap::from([
                (MODEL, head_it::MODEL.to_string()),
                (APP, head_it::APP.to_string()),
            ]),
            false,
        )
        .unwrap();
    Game {
        found,
        memory,
        panes,
        layouts: fake.layouts.clone(),
    }
}

fn place(memory: &FakeMemory, pane: i64, x: f32, y: f32) {
    memory.write(pane + lyt::X, &x.to_le_bytes());
    memory.write(pane + lyt::Y, &y.to_le_bytes());
}

/// What Head It says at a poll, the lines alone.
fn heard(found: &mut head_it::HeadIt, running: &str) -> Vec<String> {
    found
        .poll(Some(running))
        .unwrap()
        .into_iter()
        .map(|(_, line, _)| line)
        .collect()
}

#[test]
fn test_a_ball_is_said_as_it_comes_by_the_pokemon_it_falls_at() {
    let mut game = game();
    assert_eq!(heard(&mut game.found, head_it::APP), ["Ralts"]);
    assert_eq!(heard(&mut game.found, head_it::APP), Vec::<String>::new());
    // Fallen past the bottom, it is parked; the next ball down the right column is rowan's.
    place(&game.memory, game.panes[0], 400.0, 480.0);
    place(&game.memory, game.panes[2], 100.0, 478.0);
    game.layouts.refresh();
    assert_eq!(heard(&mut game.found, head_it::APP), ["rowan"]);
    // After time up, the giant ball down the middle.
    place(&game.memory, game.panes[1], 0.0, 470.0);
    game.layouts.refresh();
    assert_eq!(heard(&mut game.found, head_it::APP), ["Ralts"]);
    assert_eq!(heard(&mut game.found, OTHER), Vec::<String>::new());
}

#[test]
fn test_a_ball_at_no_pokemon_that_can_be_read_is_said_a_ball() {
    let mut game = game();
    game.memory.u32(
        MODEL + head_it::RECORDS + head_it::SIZE + head_it::PARAM,
        &[0],
    );
    assert_eq!(heard(&mut game.found, head_it::APP), ["ball of yarn"]);
}

/// ADR-0009: touch mode taps the Pokémon it has just named, where the game's instructions say,
/// "Tap Pokémon to knock back the balls of yarn!"; one not yet up is not offered.
#[test]
fn test_each_pokemon_up_is_offered_at_its_body_by_its_name() {
    let game = game();
    let (row, tall) = (120.0 - head_it::BODY as f64, head_it::TALL);
    assert_eq!(
        game.found.targets(Some(head_it::APP)).unwrap(),
        vec![
            Target::new(PARAM + 0x1000, "Ralts", 160, head_it::BODY, row, tall),
            Target::new(PARAM + 0x2000, "rowan", 267, head_it::BODY, row, tall),
        ]
    );
    assert_eq!(game.found.targets(Some(OTHER)).unwrap(), vec![]);
}

/// Its opening demonstrates a ball headed back, and its Pokémon stand on behind the results
/// and Play Again; the last ball falls after time up (spike A, live).
#[test]
fn test_nothing_is_said_or_offered_before_balls_fall_or_once_they_have() {
    for (phase, playing) in [
        (1, false),
        (head_it::LAST, true),
        (5, false),
        (head_it::LIST, false),
    ] {
        let mut game = game();
        game.memory.u32(APP + head_it::PHASE, &[phase]);
        assert_eq!(
            !heard(&mut game.found, head_it::APP).is_empty(),
            playing,
            "phase {phase}"
        );
        assert_eq!(
            !game.found.targets(Some(head_it::APP)).unwrap().is_empty(),
            playing,
            "phase {phase}"
        );
    }
}

/// The cue's stand-in, keeping when each cue sounded and for which column, and the clock it is
/// timed by.
fn cued() -> (
    Rc<RefCell<Vec<(f64, i64)>>>,
    head_it::Cue,
    Rc<Cell<f64>>,
    Rc<dyn Fn() -> f64>,
) {
    let cued: Rc<RefCell<Vec<(f64, i64)>>> = Rc::new(RefCell::new(Vec::new()));
    let now = Rc::new(Cell::new(0.0));
    let cue: head_it::Cue = {
        let (cued, now) = (cued.clone(), now.clone());
        Rc::new(move |column: i64| cued.borrow_mut().push((now.get(), column)))
    };
    let clock: Rc<dyn Fn() -> f64> = {
        let now = now.clone();
        Rc::new(move || now.get())
    };
    (cued, cue, now, clock)
}

/// The game's "!", `LEAD` before the ball reaches `SWEET`, once a ball, with its column
/// (Orion, 2026-10-31: an aid, off as it ships; 2026-11-01, the game's sound and a lead a player
/// can tap in). A ball speeds up as it falls, as on as_amie_head_it's tape, so it is cued by how
/// fast it speeds up too: taken to fall on at its speed, this one was cued at 0.29 s, 0.1 s late,
/// half the time a tap heads it in.
#[test]
fn test_the_cue_sounds_the_lead_before_a_ball_falling_faster_and_faster_reaches_it() {
    let (cued, cue, now, clock) = cued();
    let mut game = game_with(Some(cue), clock);
    heard(&mut game.found, head_it::APP);
    assert!(game.found.ticking(Some(head_it::APP)));
    // 300 above `SWEET` at 200 a second, 1,000 faster each second: there at 0.6 s, so due 0.4 s
    // ahead at 0.2. Ticks of a sixtieth, each height read twice, as a tick between two frames
    // reads one again, which times nothing.
    for tick in 0..36 {
        let at = tick as f64 / 60.0;
        place(
            &game.memory,
            game.panes[0],
            0.0,
            (head_it::SWEET + 300.0 - 200.0 * at - 500.0 * at * at) as f32,
        );
        for again in [0.0, 0.005] {
            now.set(at + again);
            game.found.tick().unwrap();
        }
    }
    assert_eq!(cued.borrow().len(), 1);
    let (when, column) = cued.borrow()[0];
    assert!(
        (0.2..=0.2 + 1.0 / 60.0).contains(&when) && column == 1,
        "{when} {column}"
    );
}

#[test]
fn test_a_ball_headed_back_up_is_not_cued_and_without_the_cue_nothing_is_followed() {
    let (cued, cue, now, clock) = cued();
    let mut on = game_with(Some(cue), clock);
    heard(&mut on.found, head_it::APP);
    for (at, height) in [(0.0, 60.0f32), (0.1, 120.0), (0.2, 180.0)] {
        now.set(at);
        place(&on.memory, on.panes[0], 0.0, height);
        on.found.tick().unwrap();
    }
    assert!(cued.borrow().is_empty());
    let mut off = game().found;
    heard(&mut off, head_it::APP);
    assert!(!off.ticking(Some(head_it::APP)));
}

/// Touch mode as Head It's keys ask it: each tap made.
#[derive(Default)]
struct Touched {
    tapped: Vec<String>,
}

impl Touching for Touched {
    fn targets(&mut self) -> Result<Vec<Target>> {
        unreachable!("Head It's keys ask for no targets")
    }

    fn give(&mut self, _picked: &Target, _over: &Target) -> String {
        unreachable!("Head It's keys never drag")
    }

    fn tap(&mut self, target: &Target) -> String {
        self.tapped.push(target.name.clone());
        String::new()
    }
}

/// ADR-0009 as amended 2026-10-31: Head It's keys tap a Pokémon touch mode offers, by where it
/// stands, 8, 9 and 0 left to right; a key where none stands, or before balls fall, plays the
/// game's error sound or says why.
#[test]
fn test_a_pokemons_key_taps_it() {
    let game = game();
    let memory = game.memory.clone();
    let mut touched = Touched::default();
    let keys = head_it::HeadItKeys::new(
        Rc::new(RefCell::new(game.found)),
        Rc::new(|why: &str| why.to_string()),
    );
    assert_eq!(
        DEFAULT_CONTEXTS[HEAD_IT],
        IndexMap::from([
            ("8".to_string(), HEADERS[0].to_string()),
            ("9".to_string(), HEADERS[1].to_string()),
            ("0".to_string(), HEADERS[2].to_string()),
        ])
    );
    let app = Some(head_it::APP);
    assert_eq!(keys.context(app), Some(HEAD_IT));
    assert_eq!(keys.context(Some(OTHER)), None);
    assert_eq!(keys.press(HEADERS[1], app, &mut touched).unwrap(), "");
    assert_eq!(keys.press(HEADERS[2], app, &mut touched).unwrap(), "");
    assert_eq!(touched.tapped, ["Ralts", "rowan"]);
    assert_eq!(
        keys.press(HEADERS[0], app, &mut touched).unwrap(),
        "no Pokémon 1"
    );
    memory.u32(APP + head_it::PHASE, &[1]);
    assert_eq!(
        keys.press(HEADERS[1], app, &mut touched).unwrap(),
        "no Pokémon 2"
    );
    assert_eq!(touched.tapped, ["Ralts", "rowan"]);
}
