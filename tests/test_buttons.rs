//! The game's button managers, touch bars and field menu: whose buttons take taps now (spikes 189
//! and B).

use std::collections::HashSet;
use std::rc::Rc;

use super::fake_memory::FakeMemory;
use crate::games::gen6::screens::buttons::{
    BAR_MANAGER, BUTTONS, Buttons, HANDLE, INPUT, LISTENER, LOCKED, ON, PANE, PASSIVE, WORK,
    pane_name,
};
use crate::games::oras;

const MANAGER_VTABLE: i64 = 0x005DCFFC;
const BAR_VTABLE: i64 = 0x00705B64;
const MENU_VTABLE: i64 = 0x007FE07C;
/// The field's menu round the PlayNav, as or_super_train found it (spike 228).
const MENU: i64 = 0x086039B0;
/// The party's managers: its list's, its held items' corners', and its bottom bar's.
const LIST: i64 = 0x082DFEFC;
const CORNERS: i64 = 0x082E0348;
const BAR: i64 = 0x08399548;
const PARTY_WORK: i64 = 0x0825C208;
const BAR_WORK: i64 = 0x0825C820;
const TOUCH_BAR: i64 = 0x08369574;

/// A manager whose buttons are all in the layout of this work.
fn manager(memory: &FakeMemory, at: i64, work: i64, locked: bool, buttons: i64) {
    let (table, handle) = (at + 0x100, at + 0x200);
    memory.u32(at, &[MANAGER_VTABLE]);
    memory.u32(at + BUTTONS, &[table, buttons]);
    memory.write(at + LOCKED, &[locked as u8]);
    for i in 0..buttons {
        let button = at + 0x300 + 0x60 * i;
        memory.u32(table + 4 * i, &[button]);
        memory.u32(button + HANDLE, &[handle]);
    }
    memory.u32(handle + WORK, &[work]);
}

fn touch_bar(memory: &FakeMemory, taking: bool) {
    memory.u32(TOUCH_BAR, &[BAR_VTABLE]);
    memory.u32(TOUCH_BAR + BAR_MANAGER, &[BAR]);
    memory.u32(TOUCH_BAR + LISTENER, &[TOUCH_BAR + 0x74]);
    memory.write(TOUCH_BAR + INPUT, &[taking as u8]);
}

fn party(locked: bool, taking: bool) -> Rc<FakeMemory> {
    let memory = FakeMemory::new();
    manager(&memory, LIST, PARTY_WORK, locked, 2);
    manager(&memory, CORNERS, PARTY_WORK, true, 2);
    manager(&memory, BAR, BAR_WORK, false, 4);
    touch_bar(&memory, taking);
    memory
}

fn set(works: &[i64]) -> HashSet<i64> {
    works.iter().copied().collect()
}

/// Under "Do what with Blaze?" the party's list's managers are locked, and its touch bar takes
/// no input though its own manager is open (movie22, 131 s); at the list, only the corners'
/// manager is locked, and the list takes taps through the other (110 s).
#[test]
fn test_a_layout_whose_managers_are_all_locked_takes_no_tap() {
    let memory = party(false, true);
    let mut buttons = Buttons::new(memory.clone());
    buttons
        .track([LIST, CORNERS, BAR], [TOUCH_BAR], [], false)
        .unwrap();
    assert_eq!(buttons.idle().unwrap(), set(&[]));
    memory.write(LIST + LOCKED, b"\x01");
    memory.write(TOUCH_BAR + INPUT, b"\x00");
    assert_eq!(buttons.idle().unwrap(), set(&[PARTY_WORK, BAR_WORK]));
}

/// A manager whose memory holds something else now, and a word of a module's file kept in the
/// heap, which names a touch bar's vtable with code where its listener would be (spike 189).
#[test]
fn test_a_freed_manager_and_a_touch_bar_with_no_listener_say_nothing() {
    let memory = party(true, true);
    let mut buttons = Buttons::new(memory.clone());
    buttons
        .track([LIST, CORNERS, BAR], [TOUCH_BAR], [], false)
        .unwrap();
    assert_eq!(buttons.idle().unwrap(), set(&[PARTY_WORK]));
    memory.u32(LIST, &[0]);
    memory.u32(CORNERS, &[0]);
    assert_eq!(buttons.idle().unwrap(), set(&[]));
    memory.write(TOUCH_BAR + INPUT, b"\x00");
    memory.u32(TOUCH_BAR + LISTENER, &[0xE1A06001]);
    assert_eq!(buttons.idle().unwrap(), set(&[]));
}

/// Under Super Training's Yes and No the page's buttons read 0 at +0x35 and Yes, its touch
/// area 0x125, pane "85", reads 1 (spike 228).
#[test]
fn test_a_button_switched_off_is_said_by_its_layouts_work_and_its_panes_name() {
    let memory = FakeMemory::new();
    manager(&memory, LIST, PARTY_WORK, false, 2);
    let (yes, party_slot) = (LIST + 0x300, LIST + 0x360);
    memory.u32(yes + PANE, &[0x125]);
    memory.write(yes + ON, b"\x01");
    memory.u32(party_slot + PANE, &[0x62]);
    let mut buttons = Buttons::new(memory.clone());
    buttons.track([LIST], [], [], false).unwrap();
    assert_eq!(
        buttons.off().unwrap(),
        HashSet::from([(PARTY_WORK, "2q".to_string())])
    );
    assert_eq!(
        buttons.on().unwrap(),
        HashSet::from([(PARTY_WORK, "85".to_string())])
    );
    assert_eq!(pane_name(0x111), "7l");
}

/// Another player's Battle in the PSS, offline, is switched off with an animation to grey it
/// at +0x28, 0x2e, and drawn faded; Super Training's buttons have no animations, 0xFFFFFFFF
/// each, and are drawn as bright as ever switched off (or_pss_singleplayer_2).
#[test]
fn test_a_button_switched_off_is_greyed_where_it_has_an_animation_to_grey_it() {
    let memory = FakeMemory::new();
    manager(&memory, LIST, PARTY_WORK, false, 3);
    let (back, battle, slot) = (LIST + 0x300, LIST + 0x360, LIST + 0x3C0);
    for (button, pane, passive) in [
        (back, 0x01, 0x33),
        (battle, 0x0A, 0x2E),
        (slot, 0x62, 0xFFFFFFFF),
    ] {
        memory.u32(button + PANE, &[pane]);
        memory.u32(button + PASSIVE, &[passive]);
    }
    memory.write(back + ON, b"\x01");
    let mut buttons = Buttons::new(memory.clone());
    buttons.track([LIST], [], [], false).unwrap();
    assert_eq!(
        buttons.off().unwrap(),
        HashSet::from([
            (PARTY_WORK, "a".to_string()),
            (PARTY_WORK, "2q".to_string())
        ])
    );
    assert_eq!(
        buttons.greyed().unwrap(),
        HashSet::from([(PARTY_WORK, "a".to_string())])
    );
}

/// X's regimen menu keeps two buttons on its regimen's touch area "ca", 0x1ba, one of them
/// off, and the regimen was left out (spike 228, x_super_train2).
#[test]
fn test_a_touch_area_one_of_whose_buttons_is_on_is_not_off() {
    let memory = FakeMemory::new();
    manager(&memory, LIST, PARTY_WORK, false, 2);
    for (button, on) in [(LIST + 0x300, 0u8), (LIST + 0x360, 1)] {
        memory.u32(button + PANE, &[0x1BA]);
        memory.write(button + ON, &[on]);
    }
    let mut buttons = Buttons::new(memory.clone());
    buttons.track([LIST], [], [], false).unwrap();
    assert_eq!(buttons.off().unwrap(), HashSet::new());
}

/// An app leaves its buttons off as it closes, and its layout's slot is taken by the next.
#[test]
fn test_a_manager_with_no_button_on_says_nothing_of_its_buttons() {
    let memory = FakeMemory::new();
    manager(&memory, LIST, PARTY_WORK, false, 2);
    memory.u32(LIST + 0x300 + PANE, &[0x24]);
    let mut buttons = Buttons::new(memory.clone());
    buttons.track([LIST], [], [], false).unwrap();
    assert_eq!(buttons.off().unwrap(), HashSet::new());
}

/// The PlayNav's frame takes no tap from Super Training's bags list on, though its buttons
/// read 1 (spike 228); the menu is found by its other vtables too, and those hold no manager.
#[test]
fn test_the_field_menus_manager_is_locked_while_an_app_of_the_bottom_screen_has_the_input() {
    let memory = party(false, true);
    let (held, busy) = oras::game().field_menu;
    memory.u32(MENU, &[MENU_VTABLE]);
    memory.u32(MENU + held, &[BAR]);
    memory.u32(MENU + 0x24, &[MENU_VTABLE]);
    let mut buttons = Buttons::with_menu(memory.clone(), oras::game().field_menu);
    buttons
        .track([LIST, CORNERS, BAR], [], [MENU, MENU + 0x24], false)
        .unwrap();
    assert_eq!(buttons.idle().unwrap(), set(&[]));
    memory.write(MENU + busy, b"\x01");
    assert_eq!(buttons.idle().unwrap(), set(&[BAR_WORK]));
}
