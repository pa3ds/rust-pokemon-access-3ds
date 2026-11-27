//! Layout tests: layouts taken in from a layout pool, as spike 64 read the battle's.

use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{FakeClasses, FakeMemory};
use crate::core::memory::{Memory, PAGE};
use crate::games::common::layout::{self, BUILT_POLLS, SETTLE_WALKS};
use crate::games::common::lyt;

const POOL: i64 = 0x14A9F008;
const TABLE: i64 = POOL + lyt::POOL_TABLE_OFFSET;

/// `pool(memory, works, capacity)`, at `POOL`.
fn pool(memory: &FakeMemory, works: &[i64], capacity: i64) {
    pool_at(memory, works, capacity, POOL);
}

/// A pool of `capacity` slots whose table lists these works, the rest of it empty slots.
fn pool_at(memory: &FakeMemory, works: &[i64], capacity: i64, at: i64) {
    let table = at + lyt::POOL_TABLE_OFFSET;
    memory.u32(at + lyt::POOL_TABLE, &[table]);
    memory.u16(at + lyt::POOL_CAPACITY, capacity);
    memory.u16(at + lyt::POOL_IN_USE, works.len() as i64);
    memory.u32(at + lyt::POOL_SELF, &[at]);
    memory.u32(
        at + lyt::POOL_TABLE_END,
        &[table + capacity * lyt::ENTRY_SIZE],
    );
    for index in 0..capacity {
        let work = works
            .get(index as usize)
            .copied()
            .unwrap_or(0x14AA0000 + index * lyt::WORK_SIZE);
        memory.u32(table + index * lyt::ENTRY_SIZE, &[at, work]);
    }
}

/// `memory.pages()`, a dict in the order the pages were first written.
fn pages(memory: &FakeMemory) -> IndexMap<i64, Vec<u8>> {
    memory.pages()
}

/// `{work: root}`.
fn works(pairs: &[(i64, i64)]) -> IndexMap<i64, i64> {
    pairs.iter().copied().collect()
}

#[test]
fn test_a_layout_built_into_a_watched_pool_is_taken_in_at_the_next_look() {
    let memory = FakeMemory::new();
    let fake = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()));
    let (work, _group) = fake.screen(0x14B00000);
    pool(&memory, &[], 4);
    fake.layouts.watch_pools([TABLE + lyt::ENTRY_SIZE]).unwrap();
    assert!(!fake.layouts.poll_pools().unwrap());
    pool(&memory, &[work], 4);
    assert!(fake.layouts.poll_pools().unwrap());
    assert_eq!(fake.layouts.works(), works(&[(work, 0x14B00400)]));
    assert!(!fake.layouts.poll_pools().unwrap());
}

#[test]
fn test_a_handle_into_something_that_is_not_a_pool_leads_nowhere() {
    let memory = FakeMemory::new();
    let fake = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()));
    let (work, _group) = fake.screen(0x14B00000);
    pool(&memory, &[work], 4);
    memory.u32(POOL + lyt::POOL_TABLE, &[TABLE + 4]);
    fake.layouts.watch_pools([TABLE]).unwrap();
    assert!(!fake.layouts.poll_pools().unwrap());
    assert_eq!(fake.layouts.works(), works(&[]));
}

#[test]
fn test_a_pool_that_stops_being_one_is_no_longer_read() {
    let memory = FakeMemory::new();
    let fake = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()));
    let (work, _group) = fake.screen(0x14B00000);
    pool(&memory, &[], 4);
    fake.layouts.watch_pools([TABLE]).unwrap();
    memory.u16(POOL + lyt::POOL_CAPACITY, 0);
    for _ in 0..layout::REMADE_POLLS + 1 {
        assert!(!fake.layouts.poll_pools().unwrap());
    }
    pool(&memory, &[work], 4);
    assert!(!fake.layouts.poll_pools().unwrap());
}

/// The PSS builds each screen's pool where the last one's lay, in a block its heap's watch knew
/// already: O-Power's list, opened from the PSS menu, offered the menu's buttons until the whole
/// sweep (or_pss_o_power). Only in pages the poll has read already, as it reads the gone
/// pool's works there: read afresh, every replay recorded before went its own way.
#[test]
fn test_a_pool_made_where_a_watched_one_went_is_watched() {
    let made = POOL + 0x9E8;
    for (held, found) in [(true, true), (false, false)] {
        let memory = FakeMemory::new();
        let kept = memory.clone();
        let cached: layout::Cached = Rc::new(move |page: i64| {
            if held || page != made & -PAGE {
                kept.read(page, PAGE).ok()
            } else {
                None
            }
        });
        let fake = FakeLayouts::with_cached(
            &memory,
            &FakeClasses::new(memory.clone()),
            lyt::ORAS,
            Some(cached),
        );
        let (work, _group) = fake.screen(0x14B00000);
        pool(&memory, &[], 10);
        fake.layouts.watch_pools([TABLE]).unwrap();
        fake.layouts.poll_pools().unwrap();
        memory.u16(POOL + lyt::POOL_CAPACITY, 0);
        assert!(!fake.layouts.poll_pools().unwrap());
        assert!(!fake.layouts.poll_pools().unwrap());
        pool_at(&memory, &[work], 10, made);
        assert_eq!(fake.layouts.poll_pools().unwrap(), found);
        assert_eq!(
            fake.layouts.works(),
            if found {
                works(&[(work, 0x14B00400)])
            } else {
                works(&[])
            }
        );
    }
}

const FIELD_POOL: i64 = 0x08300000;
const CAPACITY: i64 = 4;
const SLOTS: i64 = FIELD_POOL + lyt::POOL_SLOTS_OFFSET + CAPACITY * lyt::ENTRY_SIZE;
const OWN: i64 = SLOTS + CAPACITY * lyt::WORK_SIZE;

/// A pool laid out whole as spike 99 read the field's: its header pointing at itself, its
/// table, its slots, and its own work after them, every slot's first word pointing 0x104 into
/// the own work. A layout is built into each slot in `used`. Returns the slots' works.
fn field_pool(memory: &FakeMemory, fake: &FakeLayouts, used: &[i64]) -> Vec<i64> {
    let works: Vec<i64> = (0..CAPACITY)
        .map(|index| SLOTS + index * lyt::WORK_SIZE)
        .collect();
    let table = FIELD_POOL + lyt::POOL_TABLE_OFFSET;
    memory.u32(FIELD_POOL + lyt::POOL_TABLE, &[table]);
    memory.u16(FIELD_POOL + lyt::POOL_CAPACITY, CAPACITY);
    memory.u16(FIELD_POOL + lyt::POOL_IN_USE, used.len() as i64);
    memory.u32(FIELD_POOL + lyt::POOL_SELF, &[FIELD_POOL]);
    memory.u32(
        FIELD_POOL + lyt::POOL_TABLE_END,
        &[table + CAPACITY * lyt::ENTRY_SIZE],
    );
    for (index, &work) in works.iter().enumerate() {
        memory.u32(table + index as i64 * lyt::ENTRY_SIZE, &[FIELD_POOL, work]);
        memory.u32(work, &[OWN + lyt::WORK_SIZE]);
    }
    for &index in used {
        // Each layout's own objects go far from the pool, where they cannot land on its slots.
        let layout = 0x08400000 + index * 0x1000;
        memory.u32(
            works[index as usize] + lyt::WORK_LAYOUT,
            &[layout, fake.drawinfo],
        );
        memory.u32(layout, &[fake.layout_class]);
        memory.u32(layout + lyt::LAYOUT_ROOT, &[layout + 0x400]);
        fake.pane(layout + 0x400, 0);
    }
    works
}

/// `set(fake.layouts.works())`.
fn tracked(fake: &FakeLayouts) -> IndexSet<i64> {
    fake.layouts.works().into_keys().collect()
}

/// Spike 99: every layout's first word leads to the pool it was built in, so a layout built
/// into the same pool later is taken in without waiting for a sweep.
#[test]
fn test_a_layouts_own_pool_is_found_from_it_and_watched() {
    let memory = FakeMemory::new();
    let fake = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()));
    let works = field_pool(&memory, &fake, &[1]);
    fake.layouts.track([works[1]]).unwrap();
    fake.layouts.poll_pools().unwrap();
    let works = field_pool(&memory, &fake, &[1, 2]);
    assert!(fake.layouts.poll_pools().unwrap());
    assert_eq!(tracked(&fake), IndexSet::from([works[1], works[2]]));
}

#[test]
fn test_a_watched_pool_whose_count_holds_is_not_read_again() {
    let memory = FakeMemory::new();
    let fake = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()));
    let works = field_pool(&memory, &fake, &[1]);
    fake.layouts.track([works[1]]).unwrap();
    fake.layouts.poll_pools().unwrap();
    field_pool(&memory, &fake, &[1, 2]);
    memory.u16(FIELD_POOL + lyt::POOL_IN_USE, 1);
    assert!(!fake.layouts.poll_pools().unwrap());
}

#[test]
fn test_a_work_whose_first_word_leads_to_no_pool_is_tracked_alone() {
    let memory = FakeMemory::new();
    let fake = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()));
    let works = field_pool(&memory, &fake, &[1]);
    memory.u32(FIELD_POOL + lyt::POOL_SELF, &[0]);
    fake.layouts.track([works[1]]).unwrap();
    field_pool(&memory, &fake, &[1, 2]);
    memory.u32(FIELD_POOL + lyt::POOL_SELF, &[0]);
    assert!(!fake.layouts.poll_pools().unwrap());
    assert_eq!(tracked(&fake), IndexSet::from([works[1]]));
}

/// A screen makes its pool first and builds its layouts into the pool's slots: the save's
/// details were never read when only new blocks were swept (movie4, spike 110).
#[test]
fn test_a_pool_in_memory_just_allocated_is_watched_before_any_layout_is_built_into_it() {
    let memory = FakeMemory::new();
    let fake = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()));
    let (work, _group) = fake.screen(0x14B00000);
    pool(&memory, &[], 4);
    assert_eq!(
        fake.layouts.watch_pools_in(&pages(&memory)).unwrap(),
        vec![POOL]
    );
    assert_eq!(
        fake.layouts.watch_pools_in(&pages(&memory)).unwrap(),
        Vec::<i64>::new()
    );
    fake.layouts.poll_pools().unwrap();
    pool(&memory, &[work], 4);
    assert!(fake.layouts.poll_pools().unwrap());
    assert_eq!(fake.layouts.works(), works(&[(work, 0x14B00400)]));
}

#[test]
fn test_words_that_only_look_like_a_pool_header_are_not_one() {
    let memory = FakeMemory::new();
    let fake = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()));
    memory.u32(POOL, &[TABLE]);
    assert_eq!(
        fake.layouts.watch_pools_in(&pages(&memory)).unwrap(),
        Vec::<i64>::new()
    );
}

/// Taken in the moment its work was allocated, Sun's save prompt's cursor had no panes yet,
/// its tree was walked empty and never again, and the Yes and No were read out whole (sun_3.ctm,
/// spike 123).
#[test]
fn test_a_layout_taken_in_before_its_panes_are_built_has_them_read_as_they_come() {
    let memory = FakeMemory::new();
    let fake = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()));
    let (_work, group) = fake.screen_with(0x14B00000, false, false);
    fake.track().unwrap();
    assert_eq!(
        fake.layouts.tree(group).into_keys().collect::<Vec<_>>(),
        vec![group]
    );
    fake.pane(group + 0x200, group);
    assert!(fake.layouts.revive().unwrap());
    assert!(fake.layouts.tree(group).contains_key(&(group + 0x200)));
    for _ in 0..SETTLE_WALKS {
        fake.layouts.revive().unwrap();
    }
    fake.pane(group + 0x400, group);
    assert!(!fake.layouts.revive().unwrap());
}

/// The battle takes its command buttons off the system's list of layouts to draw while its move
/// list is up, and every flag of their own still says drawn (spike 188, movie23); a layout an app
/// draws itself is never on the list, and is drawn all the same.
#[test]
fn test_a_layout_taken_off_the_list_of_layouts_to_draw_is_drawn_no_more() {
    let memory = FakeMemory::new();
    let fake = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()));
    let (work, root) = fake.screen_with(0x14B00000, false, false);
    fake.track().unwrap();

    let drawn = || -> bool {
        fake.layouts.refresh();
        fake.layouts.revive().unwrap();
        let placement = fake.layouts.placement(root).unwrap();
        placement.is_some_and(|placement| placement.drawn) && !fake.layouts.taken_off(root).unwrap()
    };

    for _ in 0..BUILT_POLLS {
        assert!(drawn());
    }
    memory.write(work + lyt::WORK_LISTED, &[1]);
    assert!(drawn());
    memory.write(work + lyt::WORK_LISTED, &[0]);
    assert!(!drawn());
    memory.write(work + lyt::WORK_LISTED, &[1]);
    assert!(drawn());
}

/// The AreaNav draws its layouts itself, and one of its works read listed once as it was built
/// and never again: taken off the list from then on, its bar's tabs and return were left out of
/// touch mode until pa3ds closed (or_area_nav).
#[test]
fn test_a_layout_read_listed_as_it_is_built_is_not_taken_off() {
    let memory = FakeMemory::new();
    let fake = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()));
    let (work, root) = fake.screen_with(0x14B00000, false, false);
    memory.write(work + lyt::WORK_LISTED, &[1]);
    fake.track().unwrap();
    fake.layouts.refresh();
    fake.layouts.placement(root).unwrap();
    memory.write(work + lyt::WORK_LISTED, &[0]);
    for _ in 0..BUILT_POLLS + 1 {
        fake.layouts.refresh();
        fake.layouts.revive().unwrap();
        assert!(!fake.layouts.taken_off(root).unwrap());
    }
}
