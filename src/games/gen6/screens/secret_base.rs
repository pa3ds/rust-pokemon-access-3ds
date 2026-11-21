//! The Secret Base PC's decoration list, Omega Ruby and Alpha Sapphire's: a category with nothing
//! in it is drawn grey, its words at alpha 0x80 where the others' are opaque (spike L on
//! or_secret_base_route_111), and was said as the others are. It is said with the word an empty
//! slot is said with, "Desks, empty". Only the list's own layout is touched.

use indexmap::IndexMap;

use crate::core::words::own;
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::FileMatching;
use crate::games::common::screen::ScreenText;
use crate::games::common::slots::EMPTY_SLOT;

/// The PC's menus and its decoration list, one layout drawn with `s_b_cursor` (spike 154).
pub const LIST: &str = "secret_base_upper.bclyt";

/// The words on the screen, each greyed category of the decoration list said empty.
pub fn emptied(
    found: IndexMap<i64, ScreenText>,
    files: &dyn FileMatching,
    layouts: &dyn TrackedLayouts,
) -> IndexMap<i64, ScreenText> {
    let mut listed: IndexMap<i64, bool> = IndexMap::new();
    let mut out = found.clone();
    for (r#box, piece) in &found {
        if !piece.faded {
            continue;
        }
        if !listed.contains_key(&piece.root) {
            let file = files.matching(&layouts.pane_names(piece.root));
            listed.insert(piece.root, file.is_some_and(|file| file.name == LIST));
        }
        if listed[&piece.root] {
            out.insert(
                *r#box,
                ScreenText {
                    text: own(EMPTY_SLOT, &[("slot", &piece.text)]),
                    ..piece.clone()
                },
            );
        }
    }
    out
}
