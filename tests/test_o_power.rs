//! What R says on O-Power's screens: the energy, then the powers running (spike Q).

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
        reader(&memory, vec![TOP, BOTTOM]).said().unwrap().as_deref(),
        Some("energy 8 of 10 Hatching Power Lv. 1 02:58")
    );
}

#[test]
fn test_r_says_nothing_of_o_power_elsewhere() {
    let memory = FakeMemory::new();
    memory.u32(RECORD + ENERGY, &[8]);
    assert_eq!(reader(&memory, vec![FIELD]).said().unwrap(), None);
}
