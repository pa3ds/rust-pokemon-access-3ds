//! ADR-0034: the running app's process leads to the objects it keeps, each checked by its class,
//! handed over once an app start.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{FakeClasses, FakeMemory};
use crate::games::gen6::anchors::{Anchors, Chain};
use crate::py::Result;

const APP: &str = "N3app3foo4ProcE";
const OTHER: &str = "N3app3bar4ProcE";
const POOLED: &str = "N3app3foo5ModelE";
const LISTED: &str = "N3app3foo5BlockE";
const PROCESS: i64 = 0x08100000;
const WORK: i64 = 0x08200000;
const POOL: i64 = 0x08300000;
const ARRAY: i64 = 0x08400000;
const BLOCKS: [i64; 2] = [0x08500000, 0x08600000];
const STRIDE: i64 = 0x100;

/// A process whose work leads to the second of a pool of three models, as the field's leads to
/// the fifth of its 32, and an array of two blocks, the array's third word empty. The anchors,
/// the memory and the model's vtable.
fn game() -> (Anchors, Rc<FakeMemory>, i64) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let (model, block) = (classes.add(POOLED), classes.add(LISTED));
    memory.u32(PROCESS + 0x24, &[WORK]);
    memory.u32(WORK + 0x10, &[POOL + STRIDE]); // lands on the second of the pool
    memory.u32(WORK + 0x14, &[ARRAY]);
    for i in 0..3 {
        memory.u32(POOL + STRIDE * i, &[model]);
    }
    memory.u32(ARRAY + 0x8, &BLOCKS);
    for at in BLOCKS {
        memory.u32(at, &[block]);
    }
    let chains = IndexMap::from([(
        APP.to_string(),
        vec![
            Chain {
                count: 3,
                stride: STRIDE,
                ..Chain::new(POOLED, &[0x24, 0x10])
            },
            Chain {
                count: 3,
                ..Chain::new(LISTED, &[0x24, 0x14, 0x8])
            },
        ],
    )]);
    (Anchors::new(memory.clone(), chains), memory, model)
}

/// The classes a find holds, as a set.
fn classes_of(heap: &IndexMap<i64, String>) -> HashSet<&str> {
    heap.values().map(String::as_str).collect()
}

#[test]
fn test_the_chains_lead_to_a_pool_and_an_arrays_objects_once_an_app_start() -> Result<()> {
    let (mut anchors, _, _) = game();
    let found = anchors.poll(Some(APP), Some(PROCESS))?;
    let found = found.expect("a find");
    assert!(found.partial);
    assert_eq!(
        found.heap,
        IndexMap::from([
            (POOL, POOLED.to_string()),
            (POOL + STRIDE, POOLED.to_string()),
            (POOL + 2 * STRIDE, POOLED.to_string()),
            (BLOCKS[0], LISTED.to_string()),
            (BLOCKS[1], LISTED.to_string()),
        ])
    );
    assert_eq!(anchors.poll(Some(APP), Some(PROCESS))?, None);
    anchors.started();
    assert_eq!(anchors.poll(Some(APP), Some(PROCESS))?, Some(found));
    Ok(())
}

#[test]
fn test_nothing_for_another_app_nor_a_pool_not_all_of_its_class() -> Result<()> {
    let (mut anchors, memory, _) = game();
    assert_eq!(anchors.poll(Some(OTHER), Some(PROCESS))?, None);
    assert_eq!(anchors.poll(Some(APP), None)?, None);
    memory.u32(POOL + STRIDE, &[0]);
    let found = anchors.poll(Some(APP), Some(PROCESS))?.expect("a find");
    assert_eq!(classes_of(&found.heap), HashSet::from([LISTED]));
    Ok(())
}

#[test]
fn test_a_pool_short_of_its_count_hands_nothing_over() -> Result<()> {
    let (mut anchors, memory, _) = game();
    memory.u32(POOL + 2 * STRIDE, &[0]);
    let found = anchors.poll(Some(APP), Some(PROCESS))?.expect("a find");
    assert_eq!(classes_of(&found.heap), HashSet::from([LISTED]));
    Ok(())
}

/// Walking out of a Secret Base the field's map blocks went back to the route's, which the
/// reader had let go for the base's (or_secret_base_objects).
#[test]
fn test_objects_come_back_as_news_when_a_chain_leads_to_them_again() -> Result<()> {
    let (mut anchors, memory, _) = game();
    assert!(anchors.poll(Some(APP), Some(PROCESS))?.is_some());
    memory.u32(ARRAY + 0x8, &[0, 0]);
    assert_eq!(anchors.poll(Some(APP), Some(PROCESS))?, None);
    memory.u32(ARRAY + 0x8, &BLOCKS);
    let found = anchors.poll(Some(APP), Some(PROCESS))?.expect("a find");
    assert_eq!(
        found.heap,
        IndexMap::from([
            (BLOCKS[0], LISTED.to_string()),
            (BLOCKS[1], LISTED.to_string()),
        ])
    );
    Ok(())
}
