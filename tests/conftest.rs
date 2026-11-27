//! What every test starts from.
//!
//! pytest puts these words in place before each test with an autouse fixture; here `words::current`
//! reads them in a test that has put none in place, each test running on a thread of its own.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::folders::data_path;
use crate::core::words::{ENGLISH, Words};

/// pa3ds's own words, Omega Ruby's English and every game's behind them, whatever a test before
/// put in place: a session puts its game's in place as it starts, and most tests read Omega Ruby
/// without starting one.
pub fn english() -> Words {
    let oras = data_path().unwrap().join("oras");
    Words::new(
        ENGLISH,
        Some(&oras),
        IndexMap::new(),
        Rc::new(|_line: &str| {}),
    )
    .unwrap()
}
