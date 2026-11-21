//! The battle Bag's page, said as it turns before its item (`gen6.screens.battle_bag`).

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::{FileMatching, LayoutFile};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::screens::battle_bag;
use crate::py::Result;

const PROCESS: i64 = 0x08100000;
const FALSE_GAUGE: i64 = 0x08200000;
const GAUGE: i64 = 0x08210000;
const VIEW: i64 = 0x08220000;
const UI: i64 = 0x14230000;
const ITEMS: i64 = 0x14240000;
const POCKETS_MENU: i64 = 0x14250000;
const LIST_ROOT: i64 = 0x1468E494;
const POCKETS_ROOT: i64 = 0x1460C294;
const TITLE_PANES: [(i64, &str); 5] = [
    (LIST_ROOT + 1, "1k"),
    (LIST_ROOT + 2, "1o"),
    (LIST_ROOT + 3, "1q"),
    (LIST_ROOT + 4, "1s"),
    (LIST_ROOT + 5, "4"),
];

struct FakeLayouts;

impl TrackedLayouts for FakeLayouts {
    fn works(&self) -> IndexMap<i64, i64> {
        panic!("_Layouts has no attribute 'works'")
    }

    fn pane_names(&self, root: i64) -> Vec<String> {
        let file = match root {
            LIST_ROOT => battle_bag::LIST_FILE,
            POCKETS_ROOT => battle_bag::POCKETS_FILE,
            _ => panic!("KeyError: {root}"),
        };
        vec![file.to_string()]
    }

    fn name(&self, address: i64) -> Result<Vec<u8>> {
        let name = TITLE_PANES
            .iter()
            .find(|&&(pane, _)| pane == address)
            .map_or("?", |&(_, name)| name);
        Ok([name.as_bytes(), b"\0"].concat())
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

fn bag() -> (Rc<FakeMemory>, battle_bag::BattleBag) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::starting(memory.clone(), CODE.0 + 0x8000);
    // A word a sweep took for a gauge points nowhere; the real gauge points at the view.
    memory.u32(FALSE_GAUGE + battle_bag::VIEW, &[0xE58D8004]);
    memory.u32(GAUGE + battle_bag::VIEW, &[VIEW]);
    memory.u32(VIEW, &[classes.add(battle_bag::VIEW_CLASS)]);
    memory.u32(VIEW + battle_bag::UI, &[UI]);
    memory.u32(UI + battle_bag::BAG, &[classes.add(battle_bag::BAG_CLASS)]);
    memory.u32(UI + battle_bag::BAG + battle_bag::LIST, &[ITEMS]);
    memory.u32(ITEMS, &[classes.add(battle_bag::LIST_CLASS)]);
    memory.u32(POCKETS_MENU, &[classes.add("N3btl3app12ItemCategoryE")]);
    let bag = battle_bag::BattleBag::new(
        memory.clone(),
        Rc::new(|| Some(PROCESS)),
        Rc::new(FakeLayouts),
        Rc::new(FakeFiles),
        Rc::new(|| vec![FALSE_GAUGE, GAUGE]),
    );
    (memory, bag)
}

fn screen(pocket: &str, page: &str, item: &str) -> Vec<ScreenText> {
    let words = [(1, pocket), (2, page), (3, "/"), (4, "2"), (5, item)];
    let mut pieces: Vec<ScreenText> = words
        .iter()
        .map(|&(r#box, text)| ScreenText {
            root: LIST_ROOT,
            ..ScreenText::new(LIST_ROOT + r#box, text, false)
        })
        .collect();
    pieces.push(ScreenText {
        root: POCKETS_ROOT,
        ..ScreenText::new(POCKETS_ROOT + 1, "HP/PP RESTORE", false)
    });
    pieces
}

fn page(memory: &FakeMemory, page: u8, pocket: u8) {
    memory.write(ITEMS + battle_bag::POCKET, &[pocket]);
    memory.write(ITEMS + battle_bag::PAGE, &[page]);
}

#[test]
fn test_a_page_is_said_as_it_turns_and_the_title_never_by_the_screen() {
    let (memory, mut bag) = bag();
    page(&memory, 0, 0);
    // A pocket opened: its page after its first item, the list's words its cursor's, and the
    // pockets' menu behind it no cursor's.
    let heard = bag.poll(&screen("HP/PP RESTORE", "1", "Ether")).unwrap();
    assert_eq!(heard.opened.as_deref(), Some("HP/PP RESTORE 1/2"));
    assert_eq!(heard.turned, None);
    assert_eq!(
        heard.unsaid,
        (1..6)
            .map(|r#box| LIST_ROOT + r#box)
            .collect::<HashSet<i64>>()
    );
    assert_eq!(heard.pockets, HashSet::from([POCKETS_ROOT]));
    assert_eq!(
        bag.poll(&screen("HP/PP RESTORE", "1", "Soda Pop")).unwrap(),
        battle_bag::Heard {
            turned: None,
            opened: None,
            unsaid: heard.unsaid.clone(),
            pockets: heard.pockets.clone(),
        }
    );
    // Turned, its number from the byte, the screen's title a poll behind.
    page(&memory, 1, 0);
    assert_eq!(
        bag.poll(&screen("HP/PP RESTORE", "1", "Leppa Berry"))
            .unwrap()
            .turned
            .as_deref(),
        Some("HP/PP RESTORE 2/2")
    );
    assert_eq!(
        bag.poll(&screen("HP/PP RESTORE", "2", "Oran Berry"))
            .unwrap()
            .turned,
        None
    );
    // Another pocket is no turn: opened, said after its item.
    page(&memory, 0, 1);
    let heard = bag.poll(&screen("STATUS RESTORE", "1", "Revive")).unwrap();
    assert_eq!(
        (heard.turned, heard.opened.as_deref()),
        (None, Some("STATUS RESTORE 1/2"))
    );
}

#[test]
fn test_with_no_pocket_open_nothing_is_the_bags() {
    let (memory, mut bag) = bag();
    memory.u32(UI + battle_bag::BAG + battle_bag::LIST, &[POCKETS_MENU]);
    assert_eq!(
        bag.poll(&screen("HP/PP RESTORE", "1", "Ether")).unwrap(),
        battle_bag::nothing()
    );
    // Nor with its list off screen, where no memory is read.
    memory.u32(UI + battle_bag::BAG + battle_bag::LIST, &[ITEMS]);
    assert_eq!(bag.poll(&[]).unwrap(), battle_bag::nothing());
}
