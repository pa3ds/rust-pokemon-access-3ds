//! Sun's save language, followed from the GameManager the executable names (`sm.savedata`), and
//! the names it exports (`core.cro`).

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::cro;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::sm::savedata::{CLASS, INSTANCE, LANGUAGE_GETTER, SaveLanguage};
use crate::games::sm::{SUN_LAYOUT, ULTRA_LAYOUT};

const INSTANCE_AT: i64 = 0x0067206C;
const GETTER_AT: i64 = 0x004A85C4;
const MANAGER: i64 = 0x30100000;
const GAME_DATA: i64 = 0x30200000;
const SAVE: i64 = 0x30300000;

fn put(data: &mut [u8], at: usize, values: &[u32]) {
    for (n, value) in values.iter().enumerate() {
        data[at + 4 * n..at + 4 * n + 4].copy_from_slice(&value.to_le_bytes());
    }
}

/// A static.crs naming `named`, its one segment from 0x00100000 holding them all.
fn crs(named: &IndexMap<String, i64>) -> Vec<u8> {
    let mut data = vec![0u8; 0x200];
    data[0x80..0x84].copy_from_slice(cro::MAGIC);
    let (segments_at, named_at, strings_at) = (0x138, 0x150, 0x180);
    put(&mut data, segments_at, &[0x00100000, 0x00700000, 0]);
    put(&mut data, cro::SEGMENTS as usize, &[segments_at as u32, 1]);
    put(
        &mut data,
        cro::NAMED as usize,
        &[named_at as u32, named.len() as u32],
    );
    let mut at = strings_at;
    for (i, (name, address)) in named.iter().enumerate() {
        let mut raw = name.as_bytes().to_vec();
        raw.push(0);
        data[at..at + raw.len()].copy_from_slice(&raw);
        put(
            &mut data,
            named_at + 8 * i,
            &[at as u32, ((address - 0x00100000) << 4) as u32],
        );
        at += raw.len();
    }
    data
}

fn game(language: u8, my_status: i64, class_name: &str) -> (Rc<dyn Memory>, Rc<ClassNames>) {
    let memory = FakeMemory::new();
    let vtable = FakeClasses::new(memory.clone()).add(class_name);
    memory.u32(GETTER_AT, &[0xE5D0003D]); // ldrb r0, [r0, #0x3d]
    memory.u32(INSTANCE_AT, &[MANAGER]);
    memory.u32(MANAGER + 0x24, &[GAME_DATA]);
    memory.u32(GAME_DATA + 0x04, &[SAVE]);
    memory.u32(SAVE + my_status, &[vtable]);
    memory.write(SAVE + my_status + 0x3D, &[language]);
    let memory: Rc<dyn Memory> = memory;
    let names = Rc::new(ClassNames::new(memory.clone(), CODE));
    (memory, names)
}

/// `_game(language)`: Sun's record, of its own class.
fn sun(language: u8) -> (Rc<dyn Memory>, Rc<ClassNames>) {
    game(language, SUN_LAYOUT.my_status, CLASS)
}

fn exports() -> IndexMap<String, i64> {
    IndexMap::from([
        (INSTANCE.to_string(), INSTANCE_AT),
        (LANGUAGE_GETTER.to_string(), GETTER_AT),
    ])
}

#[test]
fn test_the_crs_names_each_export_at_its_segments_address() {
    assert_eq!(cro::exports(&crs(&exports())).unwrap(), exports());
    assert_eq!(cro::exports(&[0u8; 0x200]).unwrap(), IndexMap::new());
}

/// Read at the getter's own offset, 0x3d, as Sun's and Ultra Sun's saves said English live.
#[test]
fn test_the_language_is_the_records_byte_down_the_game_managers_pointers() {
    let (memory, names) = sun(3);
    assert_eq!(
        SaveLanguage::new(memory, names, &exports(), SUN_LAYOUT)
            .language()
            .unwrap()
            .as_deref(),
        Some("fr")
    );
}

#[test]
fn test_ultra_sun_keeps_its_record_further_into_the_save() {
    let (memory, names) = game(2, ULTRA_LAYOUT.my_status, CLASS);
    assert_eq!(
        SaveLanguage::new(memory.clone(), names.clone(), &exports(), ULTRA_LAYOUT)
            .language()
            .unwrap()
            .as_deref(),
        Some("en")
    );
    assert_eq!(
        SaveLanguage::new(memory, names, &exports(), SUN_LAYOUT)
            .language()
            .unwrap(),
        None
    );
}

#[test]
fn test_a_chain_that_leads_to_another_class_reads_no_language() {
    let (memory, names) = game(2, SUN_LAYOUT.my_status, "N8Savedata9MyItemE");
    assert_eq!(
        SaveLanguage::new(memory, names, &exports(), SUN_LAYOUT)
            .language()
            .unwrap(),
        None
    );
}

#[test]
fn test_a_rom_without_the_names_reads_nothing() {
    let (memory, names) = sun(2);
    assert_eq!(
        SaveLanguage::new(memory, names, &IndexMap::new(), SUN_LAYOUT)
            .language()
            .unwrap(),
        None
    );
}
