//! Pokémon-Amie's Head It: each ball of yarn is said by the Pokémon it falls at, and each Pokémon
//! up is offered in touch mode to be tapped.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_battle::record_of;
use crate::core::rtti::ClassNames;
use crate::games::common::field::keys::{DEFAULT_CONTEXTS, HEAD_IT, HEADERS, SCORE_KEY};
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::{FileOf, OnScreen, Target};
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::rom::cries::Crier;
use crate::games::gen6::subscreen::berry_picker::Touching;
use crate::games::gen6::subscreen::head_it;
use crate::py::{self, Result};

const MODEL: i64 = 0x08B00000;
const PARAM: i64 = 0x08B10000;
const APP: i64 = 0x08B30000;
const POOCHYENA: i64 = 261;
const RALTS: i64 = 280;
const TREECKO: i64 = 252;
const OTHER: &str = "N3app6puzzle7ProcessE";
/// The round's three left to right, Poochyena down on the left.
const LINEUP: &str = "Poochyena, Ralts and rowan";

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

/// `game(cue, clock)`.
fn game_with(cue: Option<head_it::Cue>, clock: Rc<dyn Fn() -> f64>) -> Game {
    game_given(Given {
        cue,
        clock,
        ..Given::default()
    })
}

/// `game(...)`'s keyword arguments, each as the Python defaults it.
struct Given {
    cue: Option<head_it::Cue>,
    clock: Rc<dyn Fn() -> f64>,
    cry: Option<Crier>,
    instead: bool,
    named: bool,
    frames: head_it::Frames,
    sleep: Rc<dyn Fn(f64)>,
    file_of: FileOf,
    on_screen: OnScreen,
    lead: f64,
}

impl Default for Given {
    fn default() -> Self {
        Given {
            cue: None,
            clock: Rc::new(|| 0.0),
            cry: None,
            instead: false,
            named: true,
            frames: Rc::new(|| Ok(None)),
            sleep: Rc::new(|_seconds| {}),
            file_of: Rc::new(|_root| None),
            on_screen: Rc::new(Vec::new),
            lead: head_it::LEAD,
        }
    }
}

/// As as_amie_head_it's round went 30 s in: Poochyena down on the left, Ralts up in the
/// middle and rowan up on the right; a ball falling at the middle, the giant ball parked off to
/// the right and a ball past the bottom (spike 273). The game, the memory, the balls' panes and the
/// layouts. With `cue`, the cue is on, sounding through it, timed by `clock`, the game's
/// `frames` and `sleep`; with `cry`, the Pokémon's cries, `instead` of their names or not.
fn game_given(given: Given) -> Game {
    let Given {
        cue,
        clock,
        cry,
        instead,
        named,
        frames,
        sleep,
        file_of,
        on_screen,
        lead,
    } = given;
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
        Rc::new(|found: &pokemon::Pokemon| self::named(found.species)),
        cue,
        lead,
        clock,
        cry,
        instead,
        named,
        frames,
        sleep,
        Rc::new(|app: &str| (app == head_it::APP).then_some(APP)),
        file_of,
        on_screen,
    );
    memory.u32(APP, &[classes.add(head_it::APP)]);
    memory.u32(APP + head_it::PHASE, &[head_it::FALLING]);
    found
        .track(
            &IndexMap::from([(MODEL, head_it::MODEL.to_string())]),
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

/// Live, a ball went missing from a poll mid-fall and came back at the next, and was said
/// again and cued too late (spike 273).
#[test]
fn test_a_ball_missing_from_one_poll_is_the_same_ball_and_from_two_is_gone() {
    let mut game = game();
    assert_eq!(heard(&mut game.found, head_it::APP), ["Ralts"]);
    let ralts: &[&str] = &["Ralts"];
    for (x, y, said) in [
        // Back, lower: the same.
        (10500.0f32, 300.0f32, &[] as &[&str]),
        (0.0, 250.0, &[]),
        (10500.0, 300.0, &[]),
        (10500.0, 300.0, &[]),
        (0.0, 470.0, ralts),
        // Back higher: the next.
        (10500.0, 480.0, &[]),
        (0.0, 475.0, ralts),
    ] {
        place(&game.memory, game.panes[0], x, y);
        game.layouts.refresh();
        assert_eq!(heard(&mut game.found, head_it::APP), said, "{x} {y}");
    }
}

/// A player's request: the Pokémon's own cry where its name is said.
#[test]
fn test_a_ball_plays_its_pokemons_cry_toward_its_column_with_its_name_or_in_its_place() {
    let cried: Rc<RefCell<Vec<(i64, f64)>>> = Rc::new(RefCell::new(Vec::new()));
    let cry: Crier = {
        let cried = cried.clone();
        Rc::new(move |found: &pokemon::Pokemon, across: f64| {
            cried.borrow_mut().push((found.species, across));
            Ok(true)
        })
    };
    let mut game = game_given(Given {
        cry: Some(cry.clone()),
        ..Given::default()
    });
    assert_eq!(heard(&mut game.found, head_it::APP), ["Ralts"]);
    assert_eq!(*cried.borrow(), [(RALTS, 0.5)]);
    place(&game.memory, game.panes[2], 100.0, 478.0);
    game.layouts.refresh();
    assert_eq!(heard(&mut game.found, head_it::APP), ["rowan"]);
    assert_eq!(cried.borrow().last(), Some(&(TREECKO, 1.0)));
    let mut instead = game_given(Given {
        cry: Some(cry),
        instead: true,
        ..Given::default()
    })
    .found;
    assert_eq!(heard(&mut instead, head_it::APP), Vec::<String>::new());
    assert_eq!(cried.borrow().last(), Some(&(RALTS, 0.5)));
    // A cry that cannot play leaves the name to say it.
    let mut silent = game_given(Given {
        cry: Some(Rc::new(|_found: &pokemon::Pokemon, _across: f64| Ok(false))),
        instead: true,
        ..Given::default()
    })
    .found;
    assert_eq!(heard(&mut silent, head_it::APP), ["Ralts"]);
}

/// Players, 2026-11-04: `head_it_style` "none", the "!" and nothing else.
#[test]
fn test_a_ball_is_only_logged_for_a_player_who_asks_for_the_cue_alone() {
    let mut found = game_given(Given {
        named: false,
        ..Given::default()
    })
    .found;
    assert_eq!(
        found.poll(Some(head_it::APP)).unwrap(),
        [(head_it::LOG.to_string(), "Ralts".to_string(), None)]
    );
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
/// and Play Again; the last ball falls after time up (spike 273, live). The opening says the
/// lineup alone. While they fall the heap's whole sweep waits, a second of reading nothing else
/// (spike 273, live).
#[test]
fn test_no_ball_is_said_nor_pokemon_offered_before_balls_fall_or_once_they_have() {
    for (phase, playing) in [
        (1, false),
        (head_it::LAST, true),
        (5, false),
        (head_it::LIST, false),
    ] {
        let mut game = game();
        game.memory.u32(APP + head_it::PHASE, &[phase]);
        let said = heard(&mut game.found, head_it::APP);
        assert_eq!(
            !(said.is_empty() || said == [LINEUP]),
            playing,
            "phase {phase}"
        );
        assert_eq!(
            !game.found.targets(Some(head_it::APP)).unwrap().is_empty(),
            playing,
            "phase {phase}"
        );
        assert_eq!(
            game.found.holding(Some(head_it::APP)),
            playing,
            "phase {phase}"
        );
        assert!(!game.found.holding(Some(OTHER)), "phase {phase}");
    }
}

/// As Berry Picker's are (2026-11-01: "the same precedent as berry picker"): its keys'
/// order, 8, 9 and 0, the one not yet up among them, said once in the opening and again in each
/// Play Again's.
#[test]
fn test_the_rounds_three_pokemon_are_said_left_to_right_as_its_opening_starts() {
    let mut game = game();
    game.memory.u32(APP + head_it::PHASE, &[1]);
    assert_eq!(heard(&mut game.found, head_it::APP), [LINEUP]);
    assert_eq!(heard(&mut game.found, head_it::APP), Vec::<String>::new());
    game.memory.u32(APP + head_it::PHASE, &[head_it::FALLING]);
    assert_eq!(heard(&mut game.found, head_it::APP), ["Ralts"]);
    game.memory.u32(APP + head_it::PHASE, &[0]);
    assert_eq!(heard(&mut game.found, head_it::APP), [LINEUP]);
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

const BALLS: i64 = 0x08B40000;
/// The round's `Ball`, and the record of the ball falling in the tests.
const RECORD: i64 = BALLS + head_it::BALL_RECORDS + 3 * head_it::BALL_SIZE;

/// `struct.pack("<2f", a, b)`.
fn pair(a: f64, b: f64) -> Vec<u8> {
    let mut packed = (a as f32).to_le_bytes().to_vec();
    packed.extend((b as f32).to_le_bytes());
    packed
}

/// A ball and its record `n` steps into a fall down the middle from `top`, where it fell
/// `speed`, each step `rise` faster: by default as Unlimited's fastest fell (`step(memory, pane,
/// n)`).
fn step(memory: &FakeMemory, pane: i64, n: i64, top: f64, speed: f64, rise: f64) {
    let k = n as f64;
    let height = top - speed * k - rise * k * (k + 1.0) / 2.0;
    place(memory, pane, 0.0, height as f32);
    memory.write(RECORD + head_it::SPEED, &pair(speed + rise * k, rise));
    memory.write(RECORD + head_it::ACROSS, &pair(0.0, height));
}

/// `step`'s defaults: from 477.6 at 2.4 a step, 1 faster each.
const FASTEST: (f64, f64, f64) = (477.6, 2.4, 1.0);

/// `falling(...)`'s keyword arguments, each as the Python defaults it.
struct Fall {
    idle: bool,
    giant: bool,
    as_read: bool,
    gaps: Vec<f64>,
    delays: Vec<f64>,
}

impl Default for Fall {
    fn default() -> Self {
        Fall {
            idle: false,
            giant: false,
            as_read: false,
            gaps: vec![0.01, 0.04, 0.025, 0.03],
            delays: vec![0.0, 0.02, 0.005, 0.015, 0.025],
        }
    }
}

/// A ball down the middle whose record the round's `Ball` holds, cued `lead` ahead: when its
/// cue sounded and for which column, each wait for it, and its moment. From 477.6, 2.4 a step
/// and 1 faster each, a step every other frame, as Unlimited's fastest fell: 490.6 above
/// `SWEET`, it is there 28.56 steps on, at 0.955 s. With `giant`, the giant ball after time up
/// as Omega Ruby's and X's fell, from 469.2 at 3.6 a step, 0.6 faster each: at `GIANT_SWEET`
/// 28.99 steps on, at 0.969 s. Each tick reads `gaps` after the last, `delays` into a frame, by
/// default 10 to 40 ms and 0 to 25; with `idle`, a tick half a second before read it as the spare
/// balls' 480; with `as_read`, each height is timed by the frame it was read in.
fn falling(lead: f64, fall: Fall) -> (Vec<(f64, i64)>, Vec<f64>, f64) {
    let cued: Rc<RefCell<Vec<(f64, i64)>>> = Rc::new(RefCell::new(Vec::new()));
    let now = Rc::new(Cell::new(-0.5));
    let slept: Rc<RefCell<Vec<f64>>> = Rc::new(RefCell::new(Vec::new()));
    let counted = Rc::new(Cell::new(-30i64));
    let Game {
        mut found,
        memory,
        panes,
        layouts,
    } = game_given(Given {
        cue: {
            let (cued, now) = (cued.clone(), now.clone());
            Some(Rc::new(move |column: i64| {
                cued.borrow_mut().push((now.get(), column))
            }))
        },
        clock: {
            let now = now.clone();
            Rc::new(move || now.get())
        },
        frames: {
            let counted = counted.clone();
            Rc::new(move || Ok(Some(counted.get())))
        },
        sleep: {
            let (slept, now) = (slept.clone(), now.clone());
            Rc::new(move |seconds: f64| {
                slept.borrow_mut().push(seconds);
                now.set(now.get() + seconds);
            })
        },
        lead,
        ..Given::default()
    });
    memory.u32(RECORD, &[3]);
    found
        .track(&IndexMap::from([(BALLS, head_it::BALL.to_string())]), true)
        .unwrap();
    let (mut pane, mut falls) = (panes[0], FASTEST);
    if fall.giant {
        // Time up: the ball down the middle gone, the giant ball falling there.
        place(&memory, panes[0], 10500.0, 480.0);
        (pane, falls) = (panes[1], (469.2, 3.6, 0.6));
        place(&memory, pane, 0.0, 469.2);
        layouts.refresh();
    }
    heard(&mut found, head_it::APP);
    if fall.idle {
        place(&memory, pane, 0.0, 480.0);
        found.tick().unwrap();
    }
    let mut tick = 0.0;
    for tock in 0..36 {
        tick += fall.gaps[tock % fall.gaps.len()];
        counted.set(py::int(tick * head_it::FRAME_RATE).unwrap());
        let (top, speed, rise) = falls;
        step(
            &memory,
            pane,
            py::floordiv(counted.get(), 2),
            top,
            speed,
            rise,
        );
        // Python's `max(now, x)`: the first unless the second is strictly greater.
        let read = tick + fall.delays[tock % fall.delays.len()];
        if read > now.get() {
            now.set(read);
        }
        if fall.as_read {
            found.parity = [0, 0];
        }
        found.tick().unwrap();
    }
    let expected = (if fall.giant { 28.9906 } else { 28.5581 }) * head_it::STEP - lead;
    (cued.take(), slept.take(), expected)
}

/// The game's "!", `LEAD` before the ball reaches `SWEET`, once a ball, with its column
/// (Orion, 2026-10-31: an aid, off as it ships; 2026-11-01, the game's sound and a lead a player
/// can tap in). The game's own record of the ball says how far it falls at its next step and how
/// much faster each makes it (spike 273): past about 100 hits a ball falls in under a second,
/// and fitted to the heights read of its first 0.2 s, as the cue was timed before, a third or
/// more of such a ball's cues came early.
#[test]
fn test_the_cue_sounds_the_lead_before_a_ball_reaches_its_sweet_spot_by_its_own_record() {
    let (cued, _, expected) = falling(0.4, Fall::default());
    assert_eq!(
        cued.iter().map(|&(_, column)| column).collect::<Vec<_>>(),
        [1]
    );
    assert!((cued[0].0 - expected).abs() <= 0.002, "{cued:?} {expected}");
}

/// Spike 277: a cue sounded at the read nearest its moment was up to half a tick off. Each
/// height is timed by the frame the ball stepped on, every other one on the parity most of the
/// round's steps were first read on (spike 273), and the cue waits for its moment: timed by the
/// frame each height was read in, this one, read 17 to 33 ms apart, was a frame late. A ball is
/// cued from its second read, 0.8 s ahead too, within the frame the clock knows its first by.
#[test]
fn test_a_ball_is_timed_by_the_frame_it_stepped_on_however_late_it_is_read() {
    let uneven = || Fall {
        gaps: vec![0.017, 0.033, 0.029],
        delays: vec![0.0],
        ..Fall::default()
    };
    let (cued, _, expected) = falling(0.4, uneven());
    assert!((cued[0].0 - expected).abs() <= 0.002, "{cued:?} {expected}");
    let (cued, _, expected) = falling(
        0.4,
        Fall {
            as_read: true,
            ..uneven()
        },
    );
    assert!(cued[0].0 - expected >= 0.016, "{cued:?} {expected}");
    let (cued, _, expected) = falling(0.8, Fall::default());
    let late = cued[0].0 - expected;
    assert!((0.0..=1.0 / 60.0).contains(&late), "{cued:?} {expected}");
}

/// A ball cued from its first read, the tick before half a second back, waited for its cue's
/// moment in the tick, 0.29 s, and live 0.3 to 0.7 s, the screen waiting with it (spike 273):
/// the next tick is taken to come no later than `TICK`.
#[test]
fn test_a_cue_due_further_off_than_a_tick_is_not_waited_for_in_the_tick() {
    let (cued, slept, expected) = falling(
        0.6,
        Fall {
            idle: true,
            ..Fall::default()
        },
    );
    assert!((cued[0].0 - expected).abs() <= 0.002, "{cued:?} {expected}");
    // Python's `max(slept)`, which raises with nothing slept.
    let most = slept
        .iter()
        .copied()
        .reduce(|most, each| if each > most { each } else { most })
        .expect("a wait");
    assert!(most <= head_it::TICK, "{slept:?}");
}

/// Two balls falling together, both due before the next tick, the one found second due
/// first: cued in the order they were found, the first was waited for and the second sounded
/// 15 ms late.
#[test]
fn test_balls_due_in_one_tick_are_cued_in_the_order_they_are_due() {
    let (cued, cue, now, clock) = cued();
    let Game {
        mut found,
        memory,
        panes,
        layouts,
    } = game_given(Given {
        cue: Some(cue),
        clock,
        sleep: {
            let now = now.clone();
            Rc::new(move |seconds: f64| now.set(now.get() + seconds))
        },
        ..Given::default()
    });
    let right = BALLS + head_it::BALL_RECORDS + 4 * head_it::BALL_SIZE;
    memory.u32(RECORD, &[3]);
    memory.u32(right, &[4]);
    found
        .track(&IndexMap::from([(BALLS, head_it::BALL.to_string())]), true)
        .unwrap();
    place(&memory, panes[2], 100.0, 320.3);
    layouts.refresh();
    heard(&mut found, head_it::APP);
    // 20 a step, steady: at the second tick the middle one is 254.2 above `SWEET`, the right
    // one 245.3, so due 15 ms after and before the next tick.
    for (at, above) in [(0.0, 20.0), (0.03, 0.0)] {
        now.set(at);
        for (pane, at_record, x, height) in [
            (panes[0], RECORD, 0.0, head_it::SWEET + 254.2 + above),
            (panes[2], right, 100.0, head_it::SWEET + 245.3 + above),
        ] {
            place(&memory, pane, x as f32, height as f32);
            memory.write(at_record + head_it::SPEED, &pair(20.0, 0.0));
            memory.write(at_record + head_it::ACROSS, &pair(x, height));
        }
        found.tick().unwrap();
    }
    let due: Vec<f64> = [245.3, 254.2]
        .iter()
        .map(|left| 0.03 + left / 20.0 * head_it::STEP - head_it::LEAD)
        .collect();
    let cued = cued.borrow();
    assert_eq!(
        cued.iter().map(|&(_, column)| column).collect::<Vec<_>>(),
        [2, 1]
    );
    assert_eq!(cued.len(), due.len());
    assert!(
        cued.iter()
            .zip(&due)
            .all(|(&(at, _), moment)| (at - moment).abs() < 1e-6),
        "{cued:?} {due:?}"
    );
}

/// Bigger, it is headed higher (spike 273, live): cued for the others' `SWEET`, its "!" came
/// late, and Orion's tap the lead after it missed (or_amie_head_it_normal). Its record is one of
/// the round's twenty, stepping as the others do (spike 273, live on Hard).
#[test]
fn test_the_giant_ball_is_cued_the_lead_before_its_own_sweet_spot() {
    let (cued, _, expected) = falling(
        0.4,
        Fall {
            giant: true,
            ..Fall::default()
        },
    );
    assert_eq!(
        cued.iter().map(|&(_, column)| column).collect::<Vec<_>>(),
        [1]
    );
    assert!((cued[0].0 - expected).abs() <= 0.002, "{cued:?} {expected}");
}

/// Headed back up, its record's speed stands where it was, 22.2 the giant ball's (spike 273,
/// live). A ball gone by with no cue is logged, never said, with its last heights.
#[test]
fn test_a_ball_headed_back_up_is_not_cued_and_without_the_cue_nothing_is_followed() {
    let (cued, cue, now, clock) = cued();
    let mut on = game_with(Some(cue), clock);
    on.memory.u32(RECORD, &[3]);
    on.found
        .track(&IndexMap::from([(BALLS, head_it::BALL.to_string())]), true)
        .unwrap();
    heard(&mut on.found, head_it::APP);
    for (at, height) in [(0.0, 60.0f32), (0.1, 120.0), (0.2, 180.0)] {
        now.set(at);
        place(&on.memory, on.panes[0], 0.0, height);
        on.memory.write(RECORD + head_it::SPEED, &pair(22.2, 0.6));
        on.memory
            .write(RECORD + head_it::ACROSS, &pair(0.0, f64::from(height)));
        on.found.tick().unwrap();
    }
    assert!(cued.borrow().is_empty());
    place(&on.memory, on.panes[0], 10500.0, 480.0);
    on.layouts.refresh();
    assert_eq!(on.found.poll(Some(head_it::APP)).unwrap(), []);
    assert_eq!(
        on.found.poll(Some(head_it::APP)).unwrap(),
        [(
            head_it::LOG.to_string(),
            "no cue for a ball gone by, its last heights 180 at 0.20".to_string(),
            None
        )]
    );
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
            ("r".to_string(), SCORE_KEY.to_string()),
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

const SCORE_ROOT: i64 = 0x14C00000;
const STATUS_ROOT: i64 = 0x14D00000;
const HIT_ROOT: i64 = 0x14E00000;

/// Head It's three layouts of words, known by a picture each, as or_amie_head_it_normal's tape
/// drew them: `FILES.get`.
fn files(root: i64) -> Option<Rc<LayoutFile>> {
    let picture = match root {
        SCORE_ROOT => ("v", "scorewindow.bclim"),
        STATUS_ROOT => ("0", "win_heading.bclim"),
        HIT_ROOT => ("b", "hit01.bclim"),
        _ => return None,
    };
    Some(Rc::new(LayoutFile::without_frames(
        vec!["RootPane".to_string()],
        IndexMap::from([(picture.0.to_string(), vec![picture.1.to_string()])]),
    )))
}

/// Words in a pane of that name, hanging from a pane of the parent's name.
#[allow(clippy::too_many_arguments)]
fn words(
    memory: &FakeMemory,
    r#box: i64,
    name: &str,
    text: &str,
    root: i64,
    parent: i64,
    parent_name: &str,
    left: f64,
) -> ScreenText {
    for (pane, called) in [(r#box, name), (parent, parent_name)] {
        if pane != 0 {
            let mut raw = called.as_bytes().to_vec();
            raw.resize(lyt::NAME_SIZE as usize, 0);
            memory.write(pane + lyt::NAME, &raw);
        }
    }
    ScreenText {
        root,
        parent,
        ..ScreenText::placed(
            r#box,
            text,
            false,
            left,
            left + (8 * text.chars().count()) as f64,
            0.0,
            10.0,
        )
    }
}

/// Players and Orion, 2026-11-04: unsaid as they come, said on R with the clock or the
/// chances left; not the results' "Poké Puffs received!" in the score's layout, nor the high
/// score beside the clock. The combo is shown for a second after each hit, and said until a
/// miss ends it.
#[test]
fn test_the_score_its_combo_and_a_miss_are_the_rounds_score_and_a_key_says_them() {
    let shown: Rc<RefCell<Vec<ScreenText>>> = Rc::new(RefCell::new(Vec::new()));
    let Game { found, memory, .. } = game_given(Given {
        file_of: Rc::new(files),
        on_screen: {
            let shown = shown.clone();
            Rc::new(move || shown.borrow().clone())
        },
        ..Given::default()
    });
    let score = words(
        &memory,
        SCORE_ROOT + 0x100,
        "2",
        "819",
        SCORE_ROOT,
        0,
        "",
        0.0,
    );
    let combo = words(
        &memory,
        SCORE_ROOT + 0x200,
        "7",
        "98-HIT COMBO",
        SCORE_ROOT,
        0,
        "",
        0.0,
    );
    let puffs = words(
        &memory,
        SCORE_ROOT + 0x300,
        "2t",
        "Poké Puffs received!",
        SCORE_ROOT,
        0,
        "",
        0.0,
    );
    let clock = words(
        &memory,
        STATUS_ROOT + 0x100,
        "4",
        "Time Left: 23",
        STATUS_ROOT,
        STATUS_ROOT + 0x80,
        "5m",
        0.0,
    );
    let high = words(
        &memory,
        STATUS_ROOT + 0x200,
        "3h",
        "0",
        STATUS_ROOT,
        STATUS_ROOT + 0x180,
        "3a",
        0.0,
    );
    let missed = words(
        &memory,
        HIT_ROOT + 0x100,
        "23",
        "MISS!",
        HIT_ROOT,
        0,
        "",
        0.0,
    );
    let app = Some(head_it::APP);
    assert_eq!(
        [&score, &combo, &puffs, &clock, &high, &missed]
            .map(|each| found.scored(each, app).unwrap()),
        [true, true, false, false, false, true]
    );
    assert!(!found.scored(&score, Some(OTHER)).unwrap());
    let found = Rc::new(RefCell::new(found));
    let keys = head_it::HeadItKeys::new(found.clone(), Rc::new(|why: &str| why.to_string()));
    let mut touched = Touched::default();
    assert_eq!(
        keys.press(SCORE_KEY, app, &mut touched).unwrap(),
        "no score"
    );
    *shown.borrow_mut() = vec![high.clone(), clock.clone(), combo, score.clone(), puffs];
    heard(&mut found.borrow_mut(), head_it::APP);
    assert_eq!(
        keys.press(SCORE_KEY, app, &mut touched).unwrap(),
        "819, 98-HIT COMBO, Time Left: 23"
    );
    *shown.borrow_mut() = vec![high.clone(), clock.clone(), score.clone()];
    heard(&mut found.borrow_mut(), head_it::APP);
    assert_eq!(
        keys.press(SCORE_KEY, app, &mut touched).unwrap(),
        "819, 98-HIT COMBO, Time Left: 23"
    );
    *shown.borrow_mut() = vec![high, clock, score.clone(), missed];
    heard(&mut found.borrow_mut(), head_it::APP);
    assert_eq!(
        keys.press(SCORE_KEY, app, &mut touched).unwrap(),
        "819, Time Left: 23"
    );
    *shown.borrow_mut() = vec![
        words(
            &memory,
            STATUS_ROOT + 0x300,
            "0",
            "Chances:",
            STATUS_ROOT,
            STATUS_ROOT + 0x280,
            "5j",
            0.0,
        ),
        words(
            &memory,
            STATUS_ROOT + 0x400,
            "5k",
            "2",
            STATUS_ROOT,
            STATUS_ROOT + 0x280,
            "5j",
            64.0,
        ),
        score,
    ];
    assert_eq!(found.borrow().score(app).unwrap(), "819, Chances:2");
    assert_eq!(found.borrow().score(Some(OTHER)).unwrap(), "");
}

/// A screen reader said "053" as "zero five three": the score, the clock
/// and the results' points are unpadded, and words of other layouts, a trainer's "ID No. 01234",
/// are left as they are.
#[test]
fn test_the_rounds_numbers_are_said_without_the_zeros_they_are_padded_with() {
    let Game { found, memory, .. } = game_given(Given {
        file_of: Rc::new(files),
        ..Given::default()
    });
    let at = |r#box: i64, name: &str, text: &str, root: i64| {
        words(&memory, r#box, name, text, root, 0, "", 0.0)
    };
    let score = at(SCORE_ROOT + 0x100, "2", "000", SCORE_ROOT);
    let points = at(STATUS_ROOT + 0x100, "6c", "053", STATUS_ROOT);
    let clock = at(STATUS_ROOT + 0x200, "4", "Time Left: 01", STATUS_ROOT);
    let high = at(STATUS_ROOT + 0x300, "3h", "HIGH SCORE:0", STATUS_ROOT);
    let total = at(STATUS_ROOT + 0x400, "6t", "TOTAL: 100", STATUS_ROOT);
    let other = ScreenText {
        root: 0x14F00000,
        ..ScreenText::new(0x500, "ID No. 01234", false)
    };
    let reworded = found.reworded(
        [score, points, clock, high, total, other]
            .into_iter()
            .map(|p| (p.r#box, p))
            .collect(),
    );
    let texts: Vec<String> = reworded.into_values().map(|p| p.text).collect();
    assert_eq!(
        texts,
        [
            "0",
            "53",
            "Time Left: 1",
            "HIGH SCORE:0",
            "TOTAL: 100",
            "ID No. 01234"
        ]
    );
}

/// Orion, 2026-11-01: learnt while playing, but never used in the round, as a lead changed
/// mid-round throws off the player's own sense of it; said at the results as pa3ds's, for the
/// setting. A key answers its column's last cue within `ANSWER`: one later, or a second press,
/// is no answer.
#[test]
fn test_at_the_results_pa3ds_says_its_estimate_of_the_lead_from_each_answered_cue() {
    let (cued, cue, now, clock) = cued();
    let Game {
        found,
        memory,
        panes,
        layouts,
    } = game_with(Some(cue), clock);
    let found = Rc::new(RefCell::new(found));
    let keys = head_it::HeadItKeys::new(found.clone(), Rc::new(|why: &str| why.to_string()));
    let mut touched = Touched::default();
    let app = Some(head_it::APP);
    memory.u32(RECORD, &[3]);
    found
        .borrow_mut()
        .track(&IndexMap::from([(BALLS, head_it::BALL.to_string())]), true)
        .unwrap();
    let heard = || heard(&mut found.borrow_mut(), head_it::APP);
    heard();
    let (top, speed, rise) = FASTEST;
    let mut start = 0.0;
    for delay in [0.30, 0.40, 0.55, 1.5] {
        // The ball down the middle again from the top, cued 0.4 s ahead, 0.5 s in.
        step(&memory, panes[0], 0, top, speed, rise);
        layouts.refresh();
        heard();
        for tick in 0..36 {
            now.set(start + tick as f64 / 60.0);
            step(&memory, panes[0], tick / 2, top, speed, rise);
            found.borrow_mut().tick().unwrap();
        }
        now.set(cued.borrow().last().unwrap().0 + delay);
        keys.press(HEADERS[1], app, &mut touched).unwrap();
        keys.press(HEADERS[1], app, &mut touched).unwrap();
        place(&memory, panes[0], 10500.0, 480.0);
        layouts.refresh();
        heard();
        heard();
        start = now.get() + 1.0;
    }
    assert_eq!(cued.borrow().len(), 4);
    memory.u32(APP + head_it::PHASE, &[head_it::LAST]);
    assert_eq!(heard(), Vec::<String>::new());
    memory.u32(APP + head_it::PHASE, &[5]);
    assert_eq!(
        heard(),
        ["pa3ds estimates your cue lead at 0.40 seconds, from 3 taps"]
    );
    assert_eq!(heard(), Vec::<String>::new());
    memory.u32(APP + head_it::PHASE, &[1]);
    heard();
    memory.u32(APP + head_it::PHASE, &[5]);
    assert_eq!(heard(), Vec::<String>::new());
}
