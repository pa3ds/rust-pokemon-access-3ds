//! The game's cry table, found in its executable and read by species and form (spike 275).

use crate::games::gen6::rom::cries;

/// A cry GARC of 4000 sequences and then 1000 waves.
const SEQUENCES: i64 = 4000;
const FILES: i64 = 5000;
/// Where the made-up table's first other form is: Venusaur's Mega.
const MEGA: i64 = 722;

/// `struct.pack("<10H", ...)`.
fn halfwords(values: &[i64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&value| (value as u16).to_le_bytes())
        .collect()
}

/// `(sequence, wave) * PAIRS`.
fn four(sequence: i64, wave: i64) -> Vec<i64> {
    [sequence, wave].repeat(cries::PAIRS as usize)
}

fn record(sequence: i64, wave: i64, form: i64) -> Vec<u8> {
    let mut values = vec![form, 0];
    values.extend(four(sequence, SEQUENCES + wave));
    halfwords(&values)
}

/// An executable holding a cry table: something before it, no one, every species' record,
/// Venusaur's pointing at its Mega's after them, and then other data.
fn code(species: i64) -> Vec<u8> {
    let mut body = b"other data".to_vec();
    body.extend(vec![0u8; cries::RECORD as usize]);
    for s in 1..species {
        body.extend(record((s - 1) * 5, s - 1, if s == 3 { MEGA } else { 0 }));
    }
    body.extend(record(15, 3, 0));
    body.extend(halfwords(&[1, 1, 0, 0, 0, 0, 0, 0, 0, 0]));
    body
}

/// Bulbasaur's record has the first sequence, 0, and the first wave, the first file after
/// the sequences, behind species 0's zeros: OR's 3776 and X's 3751 found it alike.
#[test]
fn test_the_table_is_found_by_bulbasaur_and_read_to_its_end() {
    let records = cries::table(&code(cries::SPECIES), SEQUENCES, FILES).unwrap();
    assert_eq!(records.len() as i64, MEGA + 1);
    assert_eq!(records[0], (0, vec![0; 8]));
    assert_eq!(records[25], (0, four(120, SEQUENCES + 24)));
    assert_eq!(records[MEGA as usize], (0, four(15, SEQUENCES + 3)));
}

#[test]
fn test_no_table_where_the_count_of_sequences_is_not_its_first_wave() {
    assert_eq!(
        cries::table(&code(cries::SPECIES), SEQUENCES + 1, FILES),
        None
    );
    // A table cut short is no table.
    assert_eq!(cries::table(&code(400), SEQUENCES, FILES), None);
}

#[test]
fn test_a_form_has_its_own_cry_where_the_table_gives_one() {
    let records = cries::table(&code(cries::SPECIES), SEQUENCES, FILES).unwrap();
    assert_eq!(cries::pairs(&records, 3, 1), Some(four(15, SEQUENCES + 3)));
    assert_eq!(cries::pairs(&records, 3, 0), Some(four(10, SEQUENCES + 2)));
    // A form with no record of its own cries as its species does.
    assert_eq!(
        cries::pairs(&records, 25, 5),
        Some(four(120, SEQUENCES + 24))
    );
    assert_eq!(cries::pairs(&records, 0, 0), None);
    assert_eq!(cries::pairs(&records, cries::SPECIES, 0), None);
}
