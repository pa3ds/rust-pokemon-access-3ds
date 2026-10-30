//! Pokémon-Amie's Berry Picker: each bubble's berry is said with the Pokémon under it, and offered
//! there to let a dragged berry go on.

use std::cell::Cell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_battle::record_of;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::layout::Layouts;
use crate::games::common::touch::Target;
use crate::games::common::{lyt, pokemon, strbuf};
use crate::games::gen6::screens::berry_picker;
use crate::py::{Error, Kind, Result};

const MODEL: i64 = 0x08B00000;
const PARAM: i64 = 0x08B10000;
const DEMO: i64 = 0x08B20000;
const TEXT: i64 = 0x08B21000;
const TREECKO: i64 = 252;

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

/// A bubble on the left showing a Pinap Berry, a spare bubble parked off the screen, a berry
/// in a tree outside any bubble, as the game's opening draws in the bubbles' layout (spike A), and
/// Treecko standing on the left: the picker, the memory, the bubble layout's panes in order (the
/// bubble, its berry, the spare, its berry and the tree's) and the layouts. With `demo`, the
/// opening's demonstration, saying it is over with 1 and not with 0.
fn picker(demo: Option<i64>) -> Picker {
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
    let mut found = berry_picker::BerryPicker::new(
        reads.clone(),
        Rc::new(ClassNames::new(reads.clone(), CODE)),
        fake.layouts.clone(),
        Rc::new(move |pane: i64| Ok(drawn.get(&pane).map(|image| image.to_string()))),
        IndexMap::from([("kinomi02.bclim".to_string(), "Pinap Berry".to_string())]),
        Rc::new(|found: &pokemon::Pokemon| {
            (found.species == TREECKO).then(|| "Treecko".to_string())
        }),
    );
    let mut tracked = IndexMap::from([(MODEL, berry_picker::MODEL.to_string())]);
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

/// ADR-0009: let go at its body, the berry was eaten (spike A, live).
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
    assert_eq!(
        picked.found.poll(app).unwrap(),
        ["Treecko wants Pinap Berry"]
    );
    assert_eq!(picked.found.poll(app).unwrap(), Vec::<String>::new());
    picked
        .memory
        .write(picked.panes[1] + lyt::X, &(-300.0f32).to_le_bytes());
    picked.layouts.refresh();
    assert_eq!(picked.found.poll(app).unwrap(), Vec::<String>::new());
    picked
        .memory
        .write(picked.panes[1] + lyt::X, &(-104.0f32).to_le_bytes());
    picked.layouts.refresh();
    assert_eq!(
        picked.found.poll(app).unwrap(),
        ["Treecko wants Pinap Berry"]
    );
}

/// The opening shows three Pokémon wanting berries before READY: the game demonstrating,
/// silent (Orion, 2026-10-28).
#[test]
fn test_the_openings_demonstration_is_said_nothing_of_and_the_round_is() {
    let app = Some(berry_picker::APP);
    let mut found = picker(Some(0)).found;
    assert_eq!(found.poll(app).unwrap(), Vec::<String>::new());
    assert_eq!(found.targets(app).unwrap(), vec![]);
    assert_eq!(
        picker(Some(1)).found.poll(app).unwrap(),
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
        picked.found.poll(Some(berry_picker::APP)).unwrap(),
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
        picked.found.poll(Some(berry_picker::APP)).unwrap(),
        ["Pinap Berry wanted"]
    );
}
