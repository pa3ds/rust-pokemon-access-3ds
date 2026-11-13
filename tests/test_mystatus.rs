//! The save's trainer record says the game's language, read from what a sweep read (spike 226).

use indexmap::IndexMap;

use crate::core::discovery::Found;
use crate::core::snapshot::PAGE;
use crate::games::gen6::mystatus;

/// Where movie1's `MyStatus` was, 2 s into a new game.
const RECORD: i64 = 0x08C7D348;

fn found(language: u8, at: i64) -> Found {
    found_with(language, at, 0, "")
}

/// `found(language, at, gender, name)`.
fn found_with(language: u8, at: i64, gender: u8, name: &str) -> Found {
    let page = at - at % PAGE;
    let mut data = vec![0u8; (PAGE * 2) as usize];
    data[(at - page + mystatus::LANGUAGE) as usize] = language;
    data[(at - page + mystatus::GENDER) as usize] = gender;
    let raw: Vec<u8> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let start = (at - page + mystatus::NAME) as usize;
    data[start..start + raw.len()].copy_from_slice(&raw);
    let half = PAGE as usize;
    Found {
        pages: IndexMap::from([
            (page, data[..half].to_vec()),
            (page + PAGE, data[half..].to_vec()),
        ]),
        heap: IndexMap::from([(at, mystatus::CLASS.to_string())]),
        partial: false,
    }
}

/// An English save says 2, English; the Chinese fan Omega Ruby's says 1, Japanese, whose text
/// its Chinese replaced, so its game text is read as the Japanese and is Chinese.
#[test]
fn test_the_language_is_the_records_byte() {
    assert_eq!(mystatus::language(&found(2, RECORD)), Some("en"));
    assert_eq!(mystatus::language(&found(1, RECORD)), Some("ja"));
    assert_eq!(mystatus::language(&found(3, RECORD)), Some("fr"));
}

#[test]
fn test_a_language_no_game_has_is_none() {
    assert_eq!(mystatus::language(&found(0, RECORD)), None);
    assert_eq!(mystatus::language(&found(6, RECORD)), None);
}

#[test]
fn test_a_record_the_sweep_did_not_read_says_nothing() {
    let unread = Found {
        pages: IndexMap::new(),
        heap: IndexMap::from([(RECORD, mystatus::CLASS.to_string())]),
        partial: false,
    };
    assert_eq!(mystatus::language(&unread), None);
    let nothing = Found {
        pages: IndexMap::new(),
        heap: IndexMap::new(),
        partial: false,
    };
    assert_eq!(mystatus::language(&nothing), None);
    assert_eq!(mystatus::player(&unread), None);
}

/// Every boy's save reads 0; a record with no trainer yet reads 1 and no name, so a gender is
/// believed only beside a name.
#[test]
fn test_the_player_is_a_boy_or_a_girl_once_they_have_a_name() {
    assert_eq!(
        mystatus::player(&found_with(2, RECORD, 0, "Brendan")),
        Some((false, "Brendan".to_string()))
    );
    assert_eq!(
        mystatus::player(&found_with(2, RECORD, 1, "Ruby")),
        Some((true, "Ruby".to_string()))
    );
    assert_eq!(mystatus::player(&found_with(2, RECORD, 1, "")), None);
    assert_eq!(mystatus::player(&found_with(2, RECORD, 2, "Brendan")), None);
}
