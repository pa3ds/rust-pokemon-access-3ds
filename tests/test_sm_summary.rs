//! A summary's types, by the ROM's personal data (`sm.summary`).

use crate::games::sm::summary::{self, Types};

fn entry(first: u8, second: u8, forms: u8, index: u16) -> Vec<u8> {
    let mut data = vec![0u8; 0x54];
    data[summary::TYPE_1 as usize] = first;
    data[summary::TYPE_2 as usize] = second;
    let at = summary::FORM_INDEX as usize;
    data[at..at + 2].copy_from_slice(&index.to_le_bytes());
    data[summary::FORMS as usize] = forms;
    data
}

/// Rowlet is Grass and Flying (11, 2); Rattata Normal, its Alolan form, entry 4 here, Dark
/// and Normal; a single type is said once.
#[test]
fn test_a_species_types_and_a_form_s_own() {
    let types = Types {
        entries: vec![
            entry(0, 0, 1, 0),
            entry(11, 2, 1, 0),
            entry(0, 0, 2, 4),
            entry(9, 9, 1, 0),
            entry(16, 0, 1, 0),
        ],
    };
    assert_eq!(types.of(1, 0).unwrap(), [11, 2]);
    assert_eq!(types.of(2, 0).unwrap(), [0]);
    assert_eq!(types.of(2, 1).unwrap(), [16, 0]);
    assert_eq!(types.of(3, 0).unwrap(), [9]);
    assert!(types.of(0, 0).unwrap().is_empty());
    assert!(types.of(99, 0).unwrap().is_empty());
}
