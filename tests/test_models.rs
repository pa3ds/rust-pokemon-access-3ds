//! Who stands on the map (`gen6.field.models`): what a model is to the field by its sprite, and
//! another part's NPC record as the model the game would build from it.

use std::rc::Rc;

use super::fake_memory::FakeMemory;
use super::test_field::{ZoneSpec, zone_file};
use crate::games::gen6::field::models::{Models, Role};
use crate::games::gen6::field::works::Works;
use crate::games::gen6::field::zonedata::{self, NPC, read_records, read_zone};
use crate::games::oras;
use crate::py::Result;

/// Mauville Gym's beams are drawn as the invisible blocker the game stands elsewhere (spike
/// 146): a model built there is a barrier, listed under obstacles; another part's record drawn
/// so is that blocker, never listed, as a person's record is listed.
#[test]
fn test_a_barrier_is_one_in_its_own_zone_and_elsewhere_the_blocker_it_is_drawn_as() -> Result<()> {
    let models = Models::new(
        FakeMemory::new(),
        Rc::new(oras::game().clone()),
        Rc::new(Works::new(FakeMemory::new())),
        None,
        Rc::new(|| Ok(None)),
    );
    assert_eq!(
        models.role(oras::STOPPER3, Some(oras::MAUVILLE_GYM)),
        Role::Barrier
    );
    assert_eq!(models.role(oras::STOPPER3, None), Role::Blocker);
    let data = zone_file(&ZoneSpec {
        npcs: &[
            (oras::STOPPER3, 10, 10, 0x30, None),
            (0x119, 12, 10, 0x31, None),
        ],
        ..ZoneSpec::default()
    });
    let memory = zonedata::Bytes(&data);
    let found = read_zone(&memory, 0)?.expect("a zone file");
    let records = read_records(&memory, &found, NPC)?;
    let [barrier, person] = records.as_slice() else {
        panic!("two records");
    };
    assert!(models.record(oras::MAUVILLE_GYM, barrier)?.is_none());
    let listed = models
        .record(oras::MAUVILLE_GYM, person)?
        .expect("a person is listed");
    assert_eq!(
        (
            listed.category.as_str(),
            listed.kind.as_str(),
            listed.cell,
            listed.record
        ),
        ("people", "person", (12, 10), Some((oras::MAUVILLE_GYM, 1)))
    );
    Ok(())
}
