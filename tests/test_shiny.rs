//! A shiny Pokémon said so under a battle's cursors (`gen6.screens.shiny`).

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::{FileMatching, LayoutFile};
use crate::games::common::pokemon;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::BATTLE;
use crate::games::gen6::screens::{battle_bag, partyicons, shiny};
use crate::py::Result;

const PROCESS: i64 = 0x08100000;
const GAUGE: i64 = 0x08210000;
const VIEW: i64 = 0x08220000;
const UI: i64 = 0x14230000;
const PARTY: i64 = 0x08300000;
const TARGET_ROOT: i64 = 0x1460C294;
const PARTY_ROOT: i64 = 0x1460D914;
const ZIGZAGOON: i64 = 263;

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

/// `_shiny(memory, targets, party, running)`: the targets are handed to `poll`.
fn shiny_of(memory: &Rc<FakeMemory>, party: Option<i64>, running: &str) -> shiny::Shiny {
    let running = running.to_string();
    shiny::Shiny::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        Rc::new(move || Some(running.clone())),
        Rc::new(|| Some(PROCESS)),
        Rc::new(|| vec![GAUGE]),
        Rc::new(move || party),
        Rc::new(FakeLayouts),
        Rc::new(FakeFiles),
    )
}

fn memory_of(mask: u8) -> Rc<FakeMemory> {
    let memory = FakeMemory::new();
    let classes = FakeClasses::starting(memory.clone(), CODE.0 + 0x8000);
    memory.u32(GAUGE + battle_bag::VIEW, &[VIEW]);
    memory.u32(VIEW, &[classes.add(battle_bag::VIEW_CLASS)]);
    memory.u32(VIEW + battle_bag::UI, &[UI]);
    memory.write(UI + shiny::TARGETS, &[mask]);
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

fn shiny_word(name: &str) -> String {
    own("shiny-pokemon", &[("name", &name)])
}

#[test]
fn test_the_shiny_one_of_five_alike_is_said_so_where_the_target_cursor_is_on_it() {
    let targets = || IndexMap::from([(3, shiny_zigzagoon())]);
    let screen = screen(TARGET_ROOT);
    let mut on_it = shiny_of(&memory_of(1 << 3), None, BATTLE);
    assert_eq!(
        said(
            &on_it
                .poll(moved(&["Zigzagoon"], TARGET_ROOT), &screen, &targets)
                .unwrap()
        ),
        vec![shiny_word("Zigzagoon")]
    );
    let mut beside = shiny_of(&memory_of(1 << 2), None, BATTLE);
    assert_eq!(
        said(
            &beside
                .poll(moved(&["Zigzagoon"], TARGET_ROOT), &screen, &targets)
                .unwrap()
        ),
        vec!["Zigzagoon ".to_string()]
    );
    let mut back = shiny_of(&memory_of(0), None, BATTLE);
    assert_eq!(
        said(
            &back
                .poll(moved(&["return"], TARGET_ROOT), &screen, &targets)
                .unwrap()
        ),
        vec!["return ".to_string()]
    );
}

/// No mask is read: its memory is no tape's, and reading it would be.
#[test]
fn test_with_no_shiny_pokemon_nothing_is_read() {
    let mut targets = shiny_of(&FakeMemory::new(), None, BATTLE);
    let cursors = moved(&["Zigzagoon"], TARGET_ROOT);
    assert_eq!(
        targets
            .poll(cursors.clone(), &screen(TARGET_ROOT), &IndexMap::new)
            .unwrap(),
        cursors
    );
}

#[test]
fn test_a_shiny_party_member_is_said_so_in_the_battles_party_list() {
    let memory = memory_of(0);
    let classes = FakeClasses::new(memory.clone());
    memory.write(PARTY + partyicons::COUNT, &[2]);
    let blank = vec![0u8; pokemon::RECORD_SIZE as usize];
    for (index, (name, is_shiny)) in [("Groudon", false), ("Mario", true)].into_iter().enumerate() {
        let param = 0x08400000 + 0x1000 * index as i64;
        memory.u32(PARTY + 4 * index as i64, &[param]);
        memory.u32(param, &[classes.add(pokemon::PARAM_CLASS)]);
        memory.u32(
            param + pokemon::PARAM_RECORD,
            &[param + pokemon::RECORD_IN_PARAM],
        );
        memory.write(
            param + pokemon::RECORD_IN_PARAM,
            &record(&blank, name, is_shiny),
        );
    }
    let mut party = shiny_of(&memory, Some(PARTY), BATTLE);
    let screen = screen(PARTY_ROOT);
    assert_eq!(
        said(
            &party
                .poll(
                    moved(&["Groudon", "Mario"], PARTY_ROOT),
                    &screen,
                    &IndexMap::new
                )
                .unwrap()
        ),
        vec!["Groudon ".to_string(), shiny_word("Mario")]
    );
    // Out of a battle, the field's party list says no more than it did.
    let mut field = shiny_of(&memory, Some(PARTY), "N3app8pokelist4ProcE");
    assert_eq!(
        said(
            &field
                .poll(moved(&["Mario"], PARTY_ROOT), &screen, &IndexMap::new)
                .unwrap()
        ),
        vec!["Mario ".to_string()]
    );
}

/// A record as the game stores it of a Pokémon of this nickname, shiny or not, with a constant
/// that leaves its blocks in order (as `test_battle.record`).
fn record(blank: &[u8], name: &str, is_shiny: bool) -> Vec<u8> {
    let mut plain = blank.to_vec();
    let constant: i64 = 0x00000101;
    plain[0..4].copy_from_slice(&(constant as u32).to_le_bytes());
    plain[8..10].copy_from_slice(&383u16.to_le_bytes());
    let at = pokemon::TRAINER_ID as usize;
    plain[at..at + 2].copy_from_slice(&16285u16.to_le_bytes());
    plain[at + 2..at + 4].copy_from_slice(&6019u16.to_le_bytes());
    let high: u32 = 16285 ^ 6019 ^ if is_shiny { 0 } else { 0x100 };
    let at = pokemon::PID as usize;
    plain[at..at + 4].copy_from_slice(&(high << 16).to_le_bytes());
    let nickname: Vec<u8> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let at = pokemon::NICKNAME as usize;
    plain[at..at + nickname.len()].copy_from_slice(&nickname);
    let checksum = plain[pokemon::PLAIN as usize..]
        .as_chunks::<2>()
        .0
        .iter()
        .take(112)
        .map(|word| u16::from_le_bytes(*word) as i64)
        .sum::<i64>()
        & 0xFFFF;
    let at = pokemon::CHECKSUM as usize;
    plain[at..at + 2].copy_from_slice(&(checksum as u16).to_le_bytes());
    let mut out = plain[..pokemon::PLAIN as usize].to_vec();
    out.extend(pokemon::crypt(&plain[pokemon::PLAIN as usize..], constant));
    out
}
