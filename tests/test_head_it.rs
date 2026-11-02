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
    frames: head_it::Frames,
    sleep: Rc<dyn Fn(f64)>,
}

impl Default for Given {
    fn default() -> Self {
        Given {
            cue: None,
            clock: Rc::new(|| 0.0),
            cry: None,
            instead: false,
            frames: Rc::new(|| Ok(None)),
            sleep: Rc::new(|_seconds| {}),
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
        frames,
        sleep,
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
        Rc::new(|found: &pokemon::Pokemon| named(found.species)),
        cue,
        head_it::LEAD,
        clock,
        cry,
        instead,
        frames,
        sleep,
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

/// A player's request (Orion, 2026-11-02): the Pokémon's own cry where its name is said.
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

/// As Berry Picker's are (Orion, 2026-11-01: "the same precedent as berry picker"): its keys'
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

/// Live, a tick read a ball about 300 too low once in a while, the next back in line, and the
/// cue sounded at once, 0.66 s early (spike 273); a ball read after a stall, far lower but no
/// lower than it could have fallen, is followed on.
#[test]
fn test_a_height_misread_far_below_the_ball_is_skipped() {
    let (cued, cue, now, clock) = cued();
    let mut game = game_with(Some(cue), clock.clone());
    heard(&mut game.found, head_it::APP);
    // 300 above `SWEET` at a steady 500 a second, due 0.4 s ahead at 0.2; read 300 too low at 0.1.
    for tick in 0..36 {
        now.set(tick as f64 / 60.0);
        let height = head_it::SWEET + 300.0 - 500.0 * now.get();
        place(
            &game.memory,
            game.panes[0],
            0.0,
            (if tick == 6 { height - 300.0 } else { height }) as f32,
        );
        game.found.tick().unwrap();
    }
    assert_eq!(cued.borrow().len(), 1);
    let when = cued.borrow()[0].0;
    assert!((0.2..=0.2 + 1.0 / 60.0).contains(&when), "{when}");
    let mut stalled = game_with(Some(Rc::new(|_column: i64| {})), clock);
    heard(&mut stalled.found, head_it::APP);
    for (at, height) in [(0.0, 400.0f32), (0.03, 385.0), (0.7, 35.0)] {
        now.set(at);
        place(&stalled.memory, stalled.panes[0], 0.0, height);
        stalled.found.tick().unwrap();
    }
    place(&stalled.memory, stalled.panes[0], 10500.0, 480.0);
    stalled.layouts.refresh();
    // Gone once missing from two polls.
    assert_eq!(stalled.found.poll(Some(head_it::APP)).unwrap(), []);
    assert_eq!(
        stalled.found.poll(Some(head_it::APP)).unwrap(),
        [(
            head_it::LOG.to_string(),
            "no cue for a ball gone by, its last heights 400 at 0.00, 385 at 0.03, 35 at 0.70"
                .to_string(),
            None
        )]
    );
}

/// Live, a height changes once a frame, a thirtieth of a second, and is read up to a frame
/// later, and a ball's first read can be the spare balls' 480 (spike 273): its whole fall is
/// fitted, that read left out, and the cue sounds at the read nearest its moment.
#[test]
fn test_a_ball_read_at_uneven_times_with_a_stale_first_height_is_cued_within_a_frame() {
    let (cued, cue, now, clock) = cued();
    let mut game = game_with(Some(cue), clock);
    heard(&mut game.found, head_it::APP);
    // From 400 at 120 a second, 450 faster each second, as the live runs' balls fell: at `SWEET`
    // at 1.035 s, so due 0.4 s ahead at 0.635. Each frame's height read 0 to 25 ms after it was
    // drawn, in a pattern that repeats every 5; the first read 480.
    for frame in 0..40 {
        let drawn = frame as f64 / 30.0;
        let height = if frame == 0 {
            480.0
        } else {
            400.0 - 120.0 * drawn - 225.0 * drawn.powf(2.0)
        };
        place(&game.memory, game.panes[0], 0.0, height as f32);
        now.set(drawn + [0.0, 0.025, 0.01, 0.02, 0.005][frame % 5]);
        game.found.tick().unwrap();
    }
    assert_eq!(cued.borrow().len(), 1);
    let when = cued.borrow()[0].0;
    assert!((when - 0.635).abs() <= 1.0 / 30.0, "{when}");
}

/// Spike B: a cue sounded at the read nearest its moment was up to half a tick off. Each
/// height is timed by the frame the ball stepped on, by the game's count of frames, and the cue
/// waits for its moment: timed by the frame each height was read in, this one was 25 ms
/// early.
#[test]
fn test_a_ball_timed_by_the_games_frames_is_cued_at_its_moment_however_late_it_is_read() {
    let (cued, cue, now, clock) = cued();
    let counted = Rc::new(Cell::new(0i64));
    let mut game = game_given(Given {
        cue: Some(cue),
        clock,
        frames: {
            let counted = counted.clone();
            Rc::new(move || Ok(Some(counted.get())))
        },
        sleep: {
            let now = now.clone();
            Rc::new(move |seconds: f64| now.set(now.get() + seconds))
        },
        ..Given::default()
    });
    heard(&mut game.found, head_it::APP);
    // The ball moves every other frame, from 400 at 120 a second, 450 faster each second: at
    // `SWEET` at 1.0346 s, so due 0.4 s ahead at 0.6346. Each tick reads 10 to 40 ms after the
    // last, 0 to 25 ms into a frame, in patterns that repeat.
    let mut tick = 0.0;
    for step in 0..60 {
        tick += [0.01, 0.04, 0.025, 0.03][step % 4];
        counted.set(py::int(tick * head_it::FRAME_RATE).unwrap());
        let moved = (counted.get() / 2 * 2) as f64 / head_it::FRAME_RATE;
        place(
            &game.memory,
            game.panes[0],
            0.0,
            (400.0 - 120.0 * moved - 225.0 * moved.powf(2.0)) as f32,
        );
        // Python's `max(now, x)`: the first unless the second is strictly greater.
        let read = tick + [0.0, 0.02, 0.005, 0.015, 0.025][step % 5];
        if read > now.get() {
            now.set(read);
        }
        game.found.tick().unwrap();
    }
    assert_eq!(cued.borrow().len(), 1);
    let when = cued.borrow()[0].0;
    assert!((when - 0.6346).abs() <= 0.003, "{when}");
}

/// Bigger, it is headed higher (spike 273, live): cued for the others' `SWEET`, its "!" came
/// late, and Orion's tap the lead after it missed (or_amie_head_it_normal).
#[test]
fn test_the_giant_ball_is_cued_the_lead_before_its_own_sweet_spot() {
    let (cued, cue, now, clock) = cued();
    let mut game = game_with(Some(cue), clock);
    // Time up: the ball down the middle gone, the giant ball falling there at a steady 500 a
    // second, at `GIANT_SWEET` at 0.6 s, so due 0.4 s ahead at 0.2.
    place(&game.memory, game.panes[0], 10500.0, 480.0);
    place(
        &game.memory,
        game.panes[1],
        0.0,
        (head_it::GIANT_SWEET + 300.0) as f32,
    );
    game.layouts.refresh();
    heard(&mut game.found, head_it::APP);
    for tick in 0..36 {
        now.set(tick as f64 / 60.0);
        place(
            &game.memory,
            game.panes[1],
            0.0,
            (head_it::GIANT_SWEET + 300.0 - 500.0 * now.get()) as f32,
        );
        game.found.tick().unwrap();
    }
    assert_eq!(cued.borrow().len(), 1);
    let when = cued.borrow()[0].0;
    assert!((0.2..=0.2 + 1.0 / 60.0).contains(&when), "{when}");
}

/// A ball gone by with no cue is logged, never said, with its last heights.
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
    let heard = || heard(&mut found.borrow_mut(), head_it::APP);
    heard();
    let mut start = 0.0;
    for delay in [0.30, 0.40, 0.55, 1.5] {
        // The ball down the middle again from 300 above `SWEET`, cued 0.4 s ahead, 0.2 s in.
        place(&memory, panes[0], 0.0, (head_it::SWEET + 300.0) as f32);
        layouts.refresh();
        heard();
        for tick in 0..15 {
            let at = tick as f64 / 60.0;
            now.set(start + at);
            place(
                &memory,
                panes[0],
                0.0,
                (head_it::SWEET + 300.0 - 200.0 * at - 500.0 * at * at) as f32,
            );
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
