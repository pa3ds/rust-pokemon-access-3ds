//! Hidden items out of the executable's table (`pa3ds.games.gen6.field.hidden_items`, spike
//! 249).

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use crate::core::config;
use crate::core::discovery::Found;
use crate::games::gen6::field::hidden_items::{HiddenItems, ITEMS_LEAST, Only, ROW, rows};
use crate::games::gen6::field::works::{EVENT_CLASS, FLAGS, Works};
use crate::py::{self, Result};

/// Where the run of omega_ruby's Route 110 replay found `field::EventWork`.
const EVENT: i64 = 0x08C7DAE8;
const BASE: i64 = 0x44C;
const ROUTE_110: i64 = 0x21;
const UNDER_126: i64 = 0x42;

/// An executable with junk before the table, the table, and the second table's first row,
/// which is numbered otherwise. Rows 25 to 27 are Route 110's, (57, 67), (50, 63) and the
/// Poké Ball at (10, 59); row 1 lies under Route 126.
fn image() -> Vec<u8> {
    let places = |number: i64| match number {
        25 => (ROUTE_110, 57, 67),
        26 => (ROUTE_110, 50, 63),
        27 => (ROUTE_110, 10, 59),
        1 => (UNDER_126, 39, 12),
        _ => (9, number, number),
    };
    let row = |number: i64| {
        let (zone, x, z) = places(number);
        ROW.pack([4, number, x, 2, z, zone, 0x80]).unwrap()
    };
    let mut out = [1u8, 0].repeat(40);
    for number in 1..ITEMS_LEAST + 11 {
        out.extend(row(number));
    }
    out.extend(py::fromhex("d301ac80080038001f005d005d00").unwrap());
    out
}

/// `Found({}, {EVENT: EVENT_CLASS})`.
fn found() -> Found {
    Found {
        pages: IndexMap::new(),
        heap: IndexMap::from([(EVENT, EVENT_CLASS.to_string())]),
        partial: false,
    }
}

/// ADR-0017: listing hidden items is an Extra, and ships off.
#[test]
fn test_hidden_items_are_an_extra_and_ship_off() {
    assert_eq!(config::Extras::default().hidden_items, config::HIDDEN_OFF);
    let (name, words) = config::CHOICES[0];
    assert_eq!(name, "pa3ds.extras.hidden_items");
    assert_eq!(words[0], config::HIDDEN_OFF);
}

#[test]
fn test_the_table_is_found_by_its_rows_numbered_from_one() {
    let found = rows(&image());
    assert_eq!(found.len() as i64, ITEMS_LEAST + 10);
    assert_eq!(found[26], (ROUTE_110, (10, 59), 2, 4));
    assert_eq!(rows(&image()[..200]), Vec::new());
}

#[test]
fn test_an_item_is_listed_until_its_flag_is_set_and_under_the_sea_alone_where_asked() -> Result<()>
{
    let memory = FakeMemory::new();
    let nowhere = |_cell: (i64, i64)| -> Result<Option<i64>> { Ok(None) };
    // Rows 25 and 26 found already, as the replay's save had them: flags 0x465 and 0x466.
    memory.write(EVENT + FLAGS + 0x467 / 8, &[0x60]);
    let works = Rc::new(Works::new(memory.clone()));
    let items = HiddenItems::new(memory.clone(), works.clone(), &image(), BASE, None);
    assert_eq!(items.listed(ROUTE_110, &nowhere)?, Vec::new());
    works.track(&found())?;
    let listed: Vec<(String, (i64, i64), Option<f64>)> = items
        .listed(ROUTE_110, &nowhere)?
        .into_iter()
        .map(|item| (item.kind, item.cell, item.height))
        .collect();
    assert_eq!(
        listed,
        vec![("hidden item".to_string(), (10, 59), Some(2.0))]
    );
    memory.write(EVENT + FLAGS + 0x467 / 8, &[0xE0]);
    assert_eq!(items.listed(ROUTE_110, &nowhere)?, Vec::new());
    let only: Only = Rc::new(|zone: i64| Ok(zone == UNDER_126));
    let under = HiddenItems::new(memory.clone(), works, &image(), BASE, Some(only));
    assert_eq!(under.listed(ROUTE_110, &nowhere)?, Vec::new());
    let cells: Vec<(i64, i64)> = under
        .listed(UNDER_126, &nowhere)?
        .into_iter()
        .map(|item| item.cell)
        .collect();
    assert_eq!(cells, vec![(39, 12)]);
    Ok(())
}
