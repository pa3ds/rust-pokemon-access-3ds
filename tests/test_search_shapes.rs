//! The Pokédex search's shapes, which the game names in no word, named in Omega Ruby's data.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::keyfile::data_path;
use crate::core::words::{self, Words};
use crate::games::common::pictures::named;
use crate::games::oras::WORDED;

/// Spike O: `search_icon01` to `search_icon14`, one a body shape.
#[test]
fn test_every_shape_has_words_and_each_its_own() {
    words::r#use(Some(Rc::new(
        Words::new(
            "en",
            Some(&data_path().unwrap().join("oras")),
            IndexMap::new(),
            Rc::new(|_line: &str| {}),
        )
        .unwrap(),
    )));
    let said: IndexMap<&str, Option<String>> =
        WORDED.iter().map(|&texture| (texture, named(texture))).collect();
    words::r#use(None);
    assert_eq!(said.len(), 14);
    assert!(said.values().all(|word| word.as_ref().is_some_and(|word| !word.is_empty())));
    assert_eq!(said.values().collect::<HashSet<_>>().len(), 14);
    assert_eq!(said["search_icon06.bclim"].as_deref(), Some("Bodiless"));
}
