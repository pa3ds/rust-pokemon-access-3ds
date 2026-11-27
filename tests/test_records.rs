//! A zone's records made interactables, a zone file's bytes in (`gen6.field.records`): its warps,
//! the zone loaded's and another part's out of the ROM through one loop, its furniture, and its
//! triggers.

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use super::test_field::{HOUSE, ROUTE, TOWN, ZoneSpec, zone_file};
use crate::games::common::field::things::TELEPORTER;
use crate::games::gen6::field::records::{Records, never, off_the_rails};
use crate::games::gen6::field::terrain::Terrain;
use crate::games::gen6::field::works::Works;
use crate::games::gen6::field::zonedata::{
    self, FURNITURE, ON_RAIL, PLACEMENT, WARP, WARP_DOOR, read_records, read_triggers, read_zone,
};
use crate::games::gen6::field::zonetable::ZoneTable;
use crate::games::gen6::game::{FurnitureKind, Game};
use crate::games::gen6::rom::archives::GameText;
use crate::games::oras;
use crate::py::Result;

/// ORAS's teleporter pad (`Game.warps`), and a kind only a pad puts the player at, as X's.
const PAD: i64 = 0x0601;
const LANDING: i64 = 0x0001;

/// `GAME`: ORAS's with a landing, a ladder out of the town and a lift.
fn game() -> Game {
    Game {
        landings: [LANDING].into_iter().collect(),
        furniture: IndexMap::from([(
            (TOWN, 0x30),
            FurnitureKind::Exit("exits".into(), "ladder".into(), ROUTE),
        )]),
        triggers: IndexMap::from([(
            (TOWN, 0x40),
            ("exits".into(), "lift".into(), Some(ROUTE)),
        )]),
        ..oras::game().clone()
    }
}

/// The records of `GAME`, with no zone table found, no matrix out of the ROM, and a vending
/// machine's shared script in the shipped table.
fn records(text: Option<GameText>) -> Records {
    Records::new(
        Rc::new(game()),
        Rc::new(text.unwrap_or_default()),
        Rc::new(ZoneTable::new(FakeMemory::new(), Vec::new())),
        Rc::new(Terrain::new(None, None, None, None, None)),
        IndexMap::new(),
        IndexMap::from([(0x28F0, "vending machine".to_string())]),
    )
}

/// The town's warps: two doors side by side into the house, a pad into the town itself
/// coming out at the fourth warp, as high as it, a landing, and a door on a rail.
fn warps() -> Result<Vec<Vec<u8>>> {
    let data = zone_file(&ZoneSpec {
        warps: &[
            (HOUSE, WARP_DOOR, 94, 171, None, None),
            (HOUSE, WARP_DOOR, 95, 171, None, None),
            (TOWN, PAD, 100, 160, Some(0), Some(3)),
            (TOWN, LANDING, 104, 160, Some(36), None),
            (HOUSE, WARP_DOOR, 90, 160, None, None),
        ],
        ..ZoneSpec::default()
    });
    let memory = zonedata::Bytes(&data);
    let found = read_zone(&memory, 0)?.expect("a zone file");
    let mut out = read_records(&memory, &found, WARP)?;
    let at = PLACEMENT as usize;
    out[4].splice(at..at + 2, (ON_RAIL as u16).to_le_bytes());
    Ok(out)
}

/// Out of the ROM, a part's warps are listed as the zone loaded's are: side by side into one
/// zone one exit, a pad into the zone itself a teleporter landing where it comes out, the kinds
/// only a pad puts the player at left out, and with no rails to lay it, a warp on a rail too.
#[test]
fn test_another_parts_warps_are_the_zone_loadeds_with_nothing_live_asked() -> Result<()> {
    let listed = records(None).warps(
        &warps()?,
        Some(TOWN),
        &|zone| Ok(Some(zone)),
        None,
        &off_the_rails,
        &|_| Ok(false),
        &never,
    )?;
    let got: Vec<_> = listed
        .iter()
        .map(|item| (item.kind.as_str(), item.cell, item.size, item.zone, item.home))
        .collect();
    assert_eq!(
        got,
        [
            ("door", (94, 171), (2, 1), Some(HOUSE), Some(TOWN)),
            (TELEPORTER, (100, 160), (1, 1), Some(TOWN), Some(TOWN)),
        ]
    );
    assert_eq!(
        (listed[1].arrival, listed[1].landing),
        (Some((104, 160)), Some(36.0))
    );
    Ok(())
}

/// What only the zone loaded has comes in through the same loop: a warp on a rail where the
/// rails lay it on the grid, a warp the save's works shut by its slot, and one walled in.
#[test]
fn test_the_zone_loaded_asks_the_rails_the_works_and_the_chunks() -> Result<()> {
    let laid = vec![(90, 158), (91, 159)];
    let listed = records(None).warps(
        &warps()?,
        Some(TOWN),
        &|zone| Ok(Some(zone)),
        None,
        &|record| Ok(zonedata::on_rail(record)?.then(|| laid.clone())),
        &|slot| Ok(slot == 0),
        &|warp| Ok(warp.kind == TELEPORTER),
    )?;
    let got: Vec<_> = listed
        .iter()
        .map(|item| (item.kind.as_str(), item.cell, item.size, item.shape.clone()))
        .collect();
    assert_eq!(
        got,
        [
            ("door", (95, 171), (1, 1), vec![]),
            ("door", (90, 158), (2, 2), laid.clone()),
        ]
    );
    Ok(())
}

/// A furniture record is a sign; one running a shared script the shipped table names is what
/// it says, a vending machine; and one the game's table sends the player through is an exit
/// into the zone it leads to.
#[test]
fn test_furniture_is_a_sign_the_shipped_table_or_an_exit_the_game_says() -> Result<()> {
    let data = zone_file(&ZoneSpec {
        furniture: &[(0x13, 97, 171), (0x28F0, 100, 171), (0x30, 105, 171)],
        ..ZoneSpec::default()
    });
    let memory = zonedata::Bytes(&data);
    let found = read_zone(&memory, 0)?;
    assert!(
        found.is_some()
            && read_records(&memory, found.as_ref().unwrap(), FURNITURE)?.len() == 3
    );
    let found = found.unwrap();
    let listed = records(None).furniture(&memory, &found, Some(TOWN))?;
    let got: Vec<_> = listed
        .iter()
        .map(|item| (item.category.as_str(), item.kind.as_str(), item.zone, item.home))
        .collect();
    assert_eq!(
        got,
        [
            ("signs", "sign", None, Some(TOWN)),
            ("signs", "vending machine", None, Some(TOWN)),
            ("exits", "ladder", Some(ROUTE), Some(TOWN)),
        ]
    );
    Ok(())
}

/// A trigger the game's table knows is listed as it says, those of one exit side by side one;
/// a teleporter the ROM knows is a sign landing the player where it puts them, while its work
/// arms it; and a trigger neither knows is nothing to walk to.
#[test]
fn test_a_trigger_is_listed_where_the_game_knows_it_and_a_teleporter_while_armed() -> Result<()> {
    let data = zone_file(&ZoneSpec {
        triggers: &[
            (0x40, 10, 10, 1, None, None),
            (0x40, 11, 10, 1, None, None),
            (0x41, 20, 20, 1, None, None),
            (0x42, 30, 30, 1, Some(0), Some((1, 0x4001))),
            (0x43, 40, 40, 1, None, None),
        ],
        ..ZoneSpec::default()
    });
    let memory = zonedata::Bytes(&data);
    let found = read_zone(&memory, 0)?.expect("a zone file");
    let triggers = read_triggers(&memory, &found)?;
    let text = GameText {
        teleports: IndexMap::from([((TOWN, 0x41), (50, 9, 60)), ((TOWN, 0x42), (1, 2, 3))]),
        ..GameText::default()
    };
    let listed = records(Some(text)).triggers(
        &triggers,
        TOWN,
        &Works::new(FakeMemory::new()),
        false,
        &|_place| Ok(None),
    )?;
    let got: Vec<_> = listed
        .iter()
        .map(|item| {
            (
                item.category.as_str(),
                item.kind.as_str(),
                item.cell,
                item.size,
                item.zone,
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            ("exits", "lift", (10, 10), (2, 1), Some(ROUTE)),
            ("signs", TELEPORTER, (20, 20), (1, 1), None),
        ]
    );
    assert_eq!(
        (listed[1].arrival, listed[1].landing),
        (Some((50, 60)), Some(9.0))
    );
    Ok(())
}
