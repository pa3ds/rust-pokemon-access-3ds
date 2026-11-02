//! Pokémon-Amie's Head It: each ball of yarn is said by the Pokémon it falls at, and each Pokémon
//! up is offered in touch mode to be tapped.

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_battle::record_of;
use crate::core::rtti::ClassNames;
use crate::games::common::layout::Layouts;
use crate::games::common::touch::Target;
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::subscreen::head_it;

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

/// As as_amie_head_it's round went 30 s in: Poochyena down on the left, Ralts up in the
/// middle and rowan up on the right; a ball falling at the middle, the giant ball parked off to
/// the right and a ball past the bottom (spike A). The game, the memory, the balls' panes and the
/// layouts.
fn game() -> Game {
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
