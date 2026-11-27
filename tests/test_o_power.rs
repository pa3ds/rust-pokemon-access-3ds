//! What R says on O-Power's screens: the energy, then the powers running (spike 313).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use crate::core::discovery::Found;
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::o_power::{ACTIVE, CLASS, ENERGY, LIST, OPower};

const RECORD: i64 = 0x08C7FD3C;
const TOP: i64 = 0x15DCBA14;
const BOTTOM: i64 = 0x15DF062C;
const FIELD: i64 = 0x0860AB84;
/// Textures of the pictures under the list: an orb, and the regen rate beside the clock icon.
const GAUGE: [&str; 2] = ["o_power_soto.bclim", "number_4.bclim"];

fn words(r#box: i64, text: &str, x: f64, y: f64) -> ScreenText {
    ScreenText {
        left: x,
        right: x + (text.chars().count() * 6) as f64,
        row: y,
        glyph: 12.5,
        root: TOP,
        top: true,
        ..ScreenText::new(r#box, text, false)
    }
}

/// `LayoutFile((), {}, name=name)`.
fn named(name: &str) -> Rc<LayoutFile> {
    Rc::new(LayoutFile {
        name: name.to_string(),
        ..LayoutFile::new(Vec::new(), IndexMap::new(), IndexMap::new())
    })
}

fn reader(memory: &Rc<FakeMemory>, shown: Vec<i64>) -> OPower {
    let files: IndexMap<i64, Rc<LayoutFile>> = IndexMap::from([
        (TOP, named(ACTIVE)),
        (BOTTOM, named(LIST)),
        (FIELD, named("menu_lower.bclyt")),
    ]);
    let panel = vec![
        words(1, "Hatching Power", -130.0, 70.0),
        words(2, "Lv. 1", 20.0, 70.0),
        words(3, "02:58", 90.0, 70.0),
    ];
    let mut o_power = OPower::new(
        memory.clone(),
        Rc::new(move || panel.clone()),
        Rc::new(move |root| files.get(&root).cloned()),
        Rc::new(move || Ok(shown.clone())),
        Rc::new(|root| {
            Ok(if root == BOTTOM {
                GAUGE.iter().map(|image| image.to_string()).collect()
            } else {
                Vec::new()
            })
        }),
        false,
    );
    o_power.track(&Found {
        heap: IndexMap::from([(RECORD, CLASS.to_string())]),
        ..Found::default()
    });
    o_power
}

/// Using Hatching, Bargain, Prize Money and Encounter Power took the record's +0x68 10, 8, 6, 2,
/// 0 (or_pss_o_power); the Active panel on the top screen says each power running.
#[test]
fn test_r_says_the_energy_then_the_powers_running_on_o_powers_screens() {
    let memory = FakeMemory::new();
    memory.u32(RECORD + ENERGY, &[8]);
    assert_eq!(
        reader(&memory, vec![TOP]).said().unwrap().as_deref(),
        Some("energy 8 of 10 Hatching Power Lv. 1 02:58")
    );
}

/// The digit beside the list's gauge's clock icon is the energy's regen rate, `number_1` with
/// Azahar counting no steps and `number_4` at 9000 an hour (spike 313 again).
#[test]
fn test_r_says_the_regen_rate_the_list_shows_after_the_energy() {
    let memory = FakeMemory::new();
    memory.u32(RECORD + ENERGY, &[8]);
    assert_eq!(
        reader(&memory, vec![TOP, BOTTOM]).said().unwrap().as_deref(),
        Some("energy 8 of 10 regen rate 4 Hatching Power Lv. 1 02:58")
    );
}

/// With field and battle powers running, the Active panel turns between a page of each every
/// six seconds, its rows written again into the same TextBoxes, and R said the page showing
/// (or_pss_o_power_2): the other is said as it last showed, its times counted down since, until
/// it has not shown for two turns.
#[test]
fn test_r_says_each_page_of_the_active_panel_as_it_turns() {
    let memory = FakeMemory::new();
    memory.u32(RECORD + ENERGY, &[5]);
    let field = vec![
        words(1, "Active", -130.0, 40.0),
        words(2, "Field Power", -60.0, 40.0),
        words(3, "Bargain Power", -130.0, 70.0),
        words(4, "Lv. 1", 20.0, 70.0),
        words(5, "02:36", 90.0, 70.0),
    ];
    let battle = vec![
        words(1, "Active", -130.0, 40.0),
        words(2, "Battle Power", -60.0, 40.0),
        words(3, "Attack Power", -130.0, 70.0),
        words(4, "Lv. 1", 20.0, 70.0),
    ];
    let showing = Rc::new(RefCell::new(field));
    let now = Rc::new(Cell::new(100.0));
    let files: IndexMap<i64, Rc<LayoutFile>> = IndexMap::from([(TOP, named(ACTIVE))]);
    let mut o_power = OPower::with_clock(
        memory.clone(),
        {
            let showing = showing.clone();
            Rc::new(move || showing.borrow().clone())
        },
        Rc::new(move |root| files.get(&root).cloned()),
        Rc::new(|| Ok(vec![TOP])),
        Rc::new(|_root| Ok(Vec::new())),
        false,
        {
            let now = now.clone();
            Rc::new(move || now.get())
        },
    );
    o_power.track(&Found {
        heap: IndexMap::from([(RECORD, CLASS.to_string())]),
        ..Found::default()
    });
    o_power.poll();
    *showing.borrow_mut() = battle;
    now.set(106.4);
    o_power.poll();
    now.set(108.9);
    assert_eq!(
        o_power.said().unwrap().as_deref(),
        Some(
            "energy 5 of 10 Active Field Power Bargain Power Lv. 1 02:28 \
             Active Battle Power Attack Power Lv. 1"
        )
    );
    now.set(130.0);
    assert_eq!(
        o_power.said().unwrap().as_deref(),
        Some("energy 5 of 10 Active Battle Power Attack Power Lv. 1")
    );
}

#[test]
fn test_r_says_nothing_of_o_power_elsewhere() {
    let memory = FakeMemory::new();
    memory.u32(RECORD + ENERGY, &[8]);
    assert_eq!(reader(&memory, vec![FIELD]).said().unwrap(), None);
}
