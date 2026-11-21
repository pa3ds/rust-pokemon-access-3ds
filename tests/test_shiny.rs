//! A shiny Pokémon said so under a horde's target cursor (`gen6.screens.shiny`).

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::words::own;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::{FileMatching, LayoutFile};
use crate::games::common::pokemon;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::BATTLE;
use crate::games::gen6::screens::{battle_bag, shiny};
use crate::py::Result;

const PROCESS: i64 = 0x08100000;
const GAUGE: i64 = 0x08210000;
const VIEW: i64 = 0x08220000;
const UI: i64 = 0x14230000;
const SCREEN: i64 = 0x08240000;
const TARGET_ROOT: i64 = 0x1460C294;
const PARTY_ROOT: i64 = 0x1460D914;
const ZIGZAGOON: i64 = 263;
const DOUBLE_CLASS: &str = "N3btl3app16TargetSelect2vs2E";

/// `SHINY`.
fn shiny_zigzagoon() -> pokemon::Pokemon {
    pokemon::Pokemon {
        species: ZIGZAGOON,
        nickname: "Zigzagoon".to_string(),
        gender: pokemon::FEMALE,
        nicknamed: false,
        form: 0,
        egg: false,
        shiny: true,
    }
}

/// `{3: SHINY}`.
fn at_three() -> IndexMap<i64, pokemon::Pokemon> {
    IndexMap::from([(3, shiny_zigzagoon())])
}

struct FakeLayouts;

impl TrackedLayouts for FakeLayouts {
    fn works(&self) -> IndexMap<i64, i64> {
        panic!("_Layouts has no attribute 'works'")
    }

    fn pane_names(&self, root: i64) -> Vec<String> {
        if root == PARTY_ROOT {
            vec![shiny::PARTY_FILE.to_string()]
        } else {
            vec!["target".to_string()]
        }
    }

    fn name(&self, _pane: i64) -> Result<Vec<u8>> {
        panic!("_Layouts has no attribute 'name'")
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

/// `_shiny(memory, targets, running)`: the targets are handed to `poll`.
fn shiny_of(memory: &Rc<FakeMemory>, running: &str) -> shiny::Shiny {
    let running = running.to_string();
    shiny::Shiny::new(
        memory.clone(),
        Rc::new(move || Some(running.clone())),
        Rc::new(|| Some(PROCESS)),
        Rc::new(|| vec![GAUGE]),
        Rc::new(FakeLayouts),
        Rc::new(FakeFiles),
    )
}

/// `_memory(mask, screen)`.
fn memory_of(mask: u8, screen: &str) -> Rc<FakeMemory> {
    let memory = FakeMemory::new();
    let classes = FakeClasses::starting(memory.clone(), CODE.0 + 0x8000);
    memory.u32(GAUGE + battle_bag::VIEW, &[VIEW]);
    memory.u32(VIEW, &[classes.add(battle_bag::VIEW_CLASS)]);
    memory.u32(VIEW + battle_bag::UI, &[UI]);
    memory.write(UI + shiny::TARGETS, &[mask]);
    memory.u32(UI + shiny::SCREEN, &[SCREEN]);
    memory.u32(SCREEN, &[classes.add(screen)]);
    memory
}

fn moved(names: &[&str], root: i64) -> CursorPoll {
    let items = names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let piece = ScreenText {
                root,
                ..ScreenText::new(0x100 + index as i64, &format!("{name} "), false)
            };
            CursorItem::new(0x0C, vec![piece], false)
        })
        .collect();
    CursorPoll {
        items,
        under: Vec::new(),
        lists: IndexMap::new(),
        moving: HashSet::new(),
        rows: HashSet::new(),
        chased: false,
    }
}

fn screen(root: i64) -> Vec<ScreenText> {
    vec![ScreenText {
        root,
        ..ScreenText::new(root + 1, "Zigzagoon", false)
    }]
}

fn said(cursors: &CursorPoll) -> Vec<String> {
    cursors
        .items
        .iter()
        .map(|item| item.pieces[0].text.clone())
        .collect()
}

#[test]
fn test_the_shiny_one_of_five_alike_is_said_so_where_the_target_cursor_is_on_it() {
    let screen = screen(TARGET_ROOT);
    let mut on_it = shiny_of(&memory_of(1 << 3, shiny::HORDE_CLASS), BATTLE);
    assert_eq!(
        said(
            &on_it
                .poll(moved(&["Zigzagoon"], TARGET_ROOT), &screen, &at_three)
                .unwrap()
        ),
        vec![own("shiny-pokemon", &[("name", &"Zigzagoon")])]
    );
    let mut beside = shiny_of(&memory_of(1 << 2, shiny::HORDE_CLASS), BATTLE);
    assert_eq!(
        said(
            &beside
                .poll(moved(&["Zigzagoon"], TARGET_ROOT), &screen, &at_three)
                .unwrap()
        ),
        vec!["Zigzagoon ".to_string()]
    );
    let mut back = shiny_of(&memory_of(0, shiny::HORDE_CLASS), BATTLE);
    assert_eq!(
        said(
            &back
                .poll(moved(&["return"], TARGET_ROOT), &screen, &at_three)
                .unwrap()
        ),
        vec!["return ".to_string()]
    );
}

/// A double battle's few foes need no telling apart, and the party's rows are the player's.
#[test]
fn test_only_a_hordes_target_is_said_shiny() {
    let mut double = shiny_of(&memory_of(1 << 3, DOUBLE_CLASS), BATTLE);
    assert_eq!(
        said(
            &double
                .poll(
                    moved(&["Zigzagoon"], TARGET_ROOT),
                    &screen(TARGET_ROOT),
                    &at_three
                )
                .unwrap()
        ),
        vec!["Zigzagoon ".to_string()]
    );
    let mut party = shiny_of(&memory_of(1 << 3, shiny::HORDE_CLASS), BATTLE);
    assert_eq!(
        said(
            &party
                .poll(
                    moved(&["Zigzagoon"], PARTY_ROOT),
                    &screen(PARTY_ROOT),
                    &at_three
                )
                .unwrap()
        ),
        vec!["Zigzagoon ".to_string()]
    );
}

/// No mask is read: its memory is no tape's, and reading it would be.
#[test]
fn test_with_no_shiny_pokemon_nothing_is_read() {
    let mut targets = shiny_of(&FakeMemory::new(), BATTLE);
    let cursors = moved(&["Zigzagoon"], TARGET_ROOT);
    assert_eq!(
        targets
            .poll(cursors.clone(), &screen(TARGET_ROOT), &IndexMap::new)
            .unwrap(),
        cursors
    );
}
