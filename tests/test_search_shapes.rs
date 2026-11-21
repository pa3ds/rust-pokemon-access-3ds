//! The Pokédex search's shapes, which the game names in no word, named in pa3ds's data.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::keyfile::data_path;
use crate::core::words::{self, Words};
use crate::games::common::pictures::named;
use crate::games::gen6::screens::pokedex::SHAPES;
use crate::games::{oras, xy};

/// Spike O: `search_icon01` to `search_icon14`, one a body shape, the same in both games.
#[test]
fn test_every_shape_has_words_and_each_its_own() {
    for (game, worded) in [("oras", oras::WORDED.to_vec()), ("xy", xy::WORDED.to_vec())] {
        assert!(SHAPES.iter().all(|shape| worded.contains(shape)), "{game}");
        words::r#use(Some(Rc::new(
            Words::new(
                "en",
                Some(&data_path().unwrap().join(game)),
                IndexMap::new(),
                Rc::new(|_line: &str| {}),
            )
            .unwrap(),
        )));
        let said: IndexMap<&str, Option<String>> =
            SHAPES.iter().map(|&texture| (texture, named(texture))).collect();
        words::r#use(None);
        assert_eq!(said.len(), 14, "{game}");
        assert!(
            said.values()
                .all(|word| word.as_ref().is_some_and(|word| !word.is_empty())),
            "{game}"
        );
        assert_eq!(said.values().collect::<HashSet<_>>().len(), 14, "{game}");
        assert_eq!(
            said["search_icon06.bclim"].as_deref(),
            Some("Bodiless"),
            "{game}"
        );
    }
}
