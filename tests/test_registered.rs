//! What each way of the Y button's menu uses, from the save's Bag.

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use crate::games::common::layout_file::LayoutFile;
use crate::games::gen6::screens::registered;

const BAG: i64 = 0x08C6B828;
const ITEMS: [&str; 5] = ["", "Mach Bike", "Acro Bike", "Dowsing Machine", "Eon Flute"];
const MENU_ROOT: i64 = 0x82F0EE0;
const OTHER_ROOT: i64 = 0x82A1390;

fn items() -> Vec<String> {
    ITEMS.iter().map(|item| item.to_string()).collect()
}

/// A layout file drawing `texture` in its pane `pane`, as the test's `SimpleNamespace` stands in.
fn drawing(pane: &str, texture: &str) -> Rc<LayoutFile> {
    Rc::new(LayoutFile::new(
        Vec::new(),
        IndexMap::from([(pane.to_string(), vec![texture.to_string()])]),
        IndexMap::new(),
    ))
}

fn reader(slots: &[u16]) -> registered::Registered {
    let memory = FakeMemory::new();
    for (i, number) in slots.iter().enumerate() {
        memory.write(BAG + 0x0E + 2 * i as i64, &number.to_le_bytes());
    }
    let menu = drawing(registered::ICON_PANE, registered::ICON[0]);
    let other = drawing("p", "mode01.bclim");
    let mut found = registered::Registered::new(
        memory,
        0x0E,
        items(),
        Rc::new(move |root| {
            Some(if root == MENU_ROOT {
                menu.clone()
            } else {
                other.clone()
            })
        }),
    );
    found.track(&IndexMap::from([(BAG, registered::BAG.to_string())]));
    found
}

#[test]
fn test_the_menu_says_each_item_with_its_way_each_time_it_opens() {
    let mut found = reader(&[1, 2, 3, 4]);
    let said = Some("up Mach Bike, right Acro Bike, down Dowsing Machine, left Eon Flute");
    assert_eq!(
        found.poll([MENU_ROOT, OTHER_ROOT]).unwrap().as_deref(),
        said
    );
    assert_eq!(found.poll([MENU_ROOT, OTHER_ROOT]).unwrap(), None);
    assert_eq!(found.poll([OTHER_ROOT]).unwrap(), None);
    assert_eq!(found.poll([MENU_ROOT]).unwrap().as_deref(), said);
}

#[test]
fn test_a_slot_with_nothing_registered_is_left_out() {
    assert_eq!(
        reader(&[0, 2, 0, 4]).poll([MENU_ROOT]).unwrap().as_deref(),
        Some("right Acro Bike, left Eon Flute")
    );
}

#[test]
fn test_nothing_is_said_of_another_screen_or_before_the_bag_is_found() {
    let mut found = reader(&[1, 2, 3, 4]);
    assert_eq!(found.poll([OTHER_ROOT]).unwrap(), None);
    assert_eq!(found.poll([]).unwrap(), None);
    let mut empty =
        registered::Registered::new(FakeMemory::new(), 0x0E, items(), Rc::new(|_root| None));
    assert_eq!(empty.poll([MENU_ROOT]).unwrap(), None);
}
