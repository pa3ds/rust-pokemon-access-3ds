//! The count chosen at a Poké Mart, to buy or to sell, said whole each time it changes (spike F,
//! or_pokemart_during_normal_play).
//!
//! The count's words are a group of their own in its layout, the sign, the number and the price,
//! written together in one poll as the count moves. The screen text reader reports words once they
//! read the same on two polls, which keeps a sentence being typed from being said, and held, the
//! count moves every poll: it was said only when the player paused, 14 counts of about 60. To sell,
//! the rest of a rewrite said a poll after its first row took the next count's number with it: a
//! count of 1, then "8, $80,000". Here the group is said whole at every change, each count cutting
//! the last off, and the screen's own reading of it is left unsaid.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::FileMatching;
use crate::games::common::screen::{ScreenText, join};
use crate::py::{self, Result};

/// The count's group, by its layout's file and the group pane's name: the Mart's own list, and the
/// Bag's window for how many to sell. Measured in Omega Ruby; nowhere else matches. A `frozenset`
/// only tested.
pub const GROUPS: [(&str, &str); 2] = [("shop_lower.bclyt", "2t"), ("bag_item_lower.bclyt", "24")];

/// The words of a Poké Mart's count, each time they change.
pub struct Counts {
    layouts: Rc<dyn TrackedLayouts>,
    files: Rc<dyn FileMatching>,
    pauses: bool,
    /// Layout root -> its file's name, for the roots on screen.
    file: IndexMap<i64, String>,
    /// Count group -> its words, as last said, for the groups on screen.
    said: IndexMap<i64, String>,
}

impl Counts {
    pub fn new(layouts: Rc<dyn TrackedLayouts>, files: Rc<dyn FileMatching>, pauses: bool) -> Self {
        Counts {
            layouts,
            files,
            pauses,
            file: IndexMap::new(),
            said: IndexMap::new(),
        }
    }

    /// The count's words where they have changed since they were last said, or None, and
    /// the TextBoxes that are the count's, which the screen's own reading leaves unsaid.
    pub fn poll(&mut self, on_screen: &[ScreenText]) -> Result<(Option<String>, HashSet<i64>)> {
        // A set Python iterates below; what is done for each root reads nothing.
        let roots: IndexSet<i64> = on_screen.iter().map(|piece| piece.root).collect();
        // A layout's file is known only once its archive has been read, which can be polls after
        // the layout shows: a miss is asked again.
        self.file.retain(|root, _| roots.contains(root));
        for &root in &roots {
            if self.file.contains_key(&root) {
                continue;
            }
            if let Some(found) = self.files.matching(&self.layouts.pane_names(root)) {
                self.file.insert(root, found.name.clone());
            }
        }
        let mut groups: IndexMap<i64, Vec<ScreenText>> = IndexMap::new();
        let names: HashSet<&str> = GROUPS.iter().map(|&(name, _)| name).collect();
        for piece in on_screen {
            if self
                .file
                .get(&piece.root)
                .is_some_and(|file| names.contains(file.as_str()))
            {
                groups.entry(piece.parent).or_default().push(piece.clone());
            }
        }
        let mut kept: IndexMap<i64, Vec<ScreenText>> = IndexMap::new();
        for (parent, pieces) in groups {
            let file = self.file[&pieces[0].root].clone();
            let name = self::name(&self.layouts.name(parent)?);
            if GROUPS.contains(&(file.as_str(), name.as_str())) {
                kept.insert(parent, pieces);
            }
        }
        let groups = kept;
        let words: IndexMap<i64, String> = groups
            .iter()
            .map(|(&parent, pieces)| (parent, join(pieces, self.pauses)))
            .collect();
        let changed: Vec<String> = words
            .iter()
            .filter(|&(parent, said)| self.said.get(parent) != Some(said))
            .map(|(_, said)| said.clone())
            .collect();
        self.said = words;
        Ok((
            changed.into_iter().next(),
            groups.values().flatten().map(|piece| piece.r#box).collect(),
        ))
    }
}

/// A pane's name: its bytes up to the first zero.
fn name(raw: &[u8]) -> String {
    let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
    py::decode_ascii(&raw[..end])
}
