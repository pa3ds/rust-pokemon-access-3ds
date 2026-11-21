//! The Pokéblock Kit's screen not shown is put away (`oras.screens.pokeblock_kit`, spike B).

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::{FileMatching, LayoutFile};
use crate::games::gen6::apps::RunningApp;
use crate::games::gen6::game::ScreensReader;
use crate::games::oras::screens::conditions::CASE;
use crate::games::oras::screens::pokeblock_kit;
use crate::py::Result;

const PROCESS: i64 = 0x08C652BC;
const FILES: [(i64, &str); 6] = [
    (0x0822A000, "poroc_case_lower.bclyt"),
    (0x0822B000, "poroc_case_status_upper.bclyt"),
    (0x0822C000, "brender_lower.bclyt"),
    (0x0822D000, "brender_upper.bclyt"),
    (0x0822E000, "base.bclyt"),
    (0x0822F000, "base.bclyt"),
];
const CASE_LIST: i64 = FILES[0].0;
const PARTY: i64 = FILES[1].0;
const BLENDER: i64 = FILES[2].0;
const BLENDER_TOP: i64 = FILES[3].0;
const BLENDER_CURSOR: i64 = FILES[4].0;
const CASE_CURSOR: i64 = FILES[5].0;
/// Root -> its work's first word: the cursors built into their lower screens' pools.
const POOLS: [(i64, i64); 6] = [
    (CASE_LIST, 0x08300000),
    (PARTY, 0x08310000),
    (BLENDER, 0x08320000),
    (BLENDER_TOP, 0x08330000),
    (BLENDER_CURSOR, 0x08320000),
    (CASE_CURSOR, 0x08300000),
];

struct FakeLayouts;

impl TrackedLayouts for FakeLayouts {
    fn works(&self) -> IndexMap<i64, i64> {
        FILES
            .iter()
            .map(|&(root, _)| (root - 0x10000, root))
            .collect()
    }

    fn pane_names(&self, root: i64) -> Vec<String> {
        let (_, file) = FILES
            .iter()
            .find(|&&(each, _)| each == root)
            .unwrap_or_else(|| panic!("KeyError: {root}"));
        vec![file.to_string()]
    }

    fn name(&self, _pane: i64) -> Result<Vec<u8>> {
        panic!("SimpleNamespace has no attribute 'name'")
    }
}

struct FakeFiles;

impl FileMatching for FakeFiles {
    fn matching(&self, panes: &[String]) -> Option<Rc<LayoutFile>> {
        Some(Rc::new(LayoutFile {
            name: panes[0].clone(),
            ..LayoutFile::new(Vec::new(), IndexMap::new(), IndexMap::new())
        }))
    }
}

#[test]
fn test_the_screen_not_shown_is_put_away_with_its_cursors() {
    let memory = FakeMemory::new();
    for (root, pool) in POOLS {
        memory.u32(root - 0x10000, &[pool]);
    }
    // `SimpleNamespace(running=CASE, process=PROCESS)`.
    let apps = Rc::new(RefCell::new(RunningApp::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
    )));
    apps.borrow_mut().running = Some(CASE.to_string());
    apps.borrow_mut().process = Some(PROCESS);
    let mut kit = pokeblock_kit::Kit::new(
        memory.clone(),
        apps,
        Rc::new(FakeLayouts),
        Rc::new(FakeFiles),
    );
    let mut away = |sequence: u8| -> HashSet<i64> {
        memory.write(PROCESS + pokeblock_kit::SEQUENCE, &[sequence]);
        kit.put_away()
            .unwrap()
            .into_iter()
            .map(|work| work + 0x10000)
            .collect()
    };
    assert_eq!(
        away(4),
        HashSet::from([BLENDER, BLENDER_TOP, BLENDER_CURSOR])
    );
    assert_eq!(away(7), HashSet::from([CASE_LIST, PARTY, CASE_CURSOR]));
    assert_eq!(
        away(0xB),
        FILES.iter().map(|&(root, _)| root).collect::<HashSet<i64>>()
    );
    assert_eq!(away(2), HashSet::new());
}
