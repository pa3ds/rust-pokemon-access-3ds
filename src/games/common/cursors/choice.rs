//! The opening's choice of boy or girl draws a fifth (spike 71): a window the game slides from one
//! choice's place onto another's inside its own layout, each choice a plain pane beside it holding
//! that choice's pictures. A window with no panes of its own, beside two or more plain panes of one
//! size in places of their own, is a cursor from the poll it is seen on, and the choice is the pane
//! it sits exactly on: the frame round each choice sits on its own pictures, not on a pane beside it.
//! The choice is said by its own words, or, having none, by the words of the textures the other
//! choices do not share — both portraits sit on the same white backing, and only `intro_male.bclim`
//! and `intro_famale.bclim` tell them apart. A texture pa3ds has words of its own for is said by
//! those, whole: each portrait as the boy or the girl it shows (`pictures::described`).

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::f32;
use crate::games::common::cursors::menu::MenuCursors;
use crate::games::common::cursors::shared::{CursorItem, Panes, Said, Where, class_of, near};
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::games::common::pictures::words;
use crate::games::common::screen::{ScreenText, distinct, in_rows};
use crate::py::{self, Result};

/// (root, the choice it sits on, the other choices).
pub type Choice = (i64, Option<i64>, Vec<i64>);

/// Windows the game slides from one choice's place onto another's.
///
/// Python's keeps the menu cursors (`menus`), which are cursors of their own kind; here they are
/// handed to `poll`, the reader holding both.
pub struct ChoiceFrames {
    panes: Rc<Panes>,
    layouts: Rc<Layouts>,
    said: Rc<Said>,
    /// Windows reported as the cursor over one of the choices beside them.
    choices_seen: IndexSet<i64>,
    /// (root, choice) -> the words of the textures of the pictures under it, once known.
    choice_words: IndexMap<(i64, i64), Vec<String>>,
    /// pa3ds's own words for a picture, by its texture's name less `.bclim`, said whole in
    /// place of the texture's words: Omega Ruby's boy and girl, `intro_male` and
    /// `intro_famale`.
    described: IndexMap<String, String>,
}

impl ChoiceFrames {
    pub fn new(panes: Rc<Panes>, said: Rc<Said>, described: IndexMap<String, String>) -> Self {
        ChoiceFrames {
            layouts: panes.layouts.clone(),
            panes,
            said,
            choices_seen: IndexSet::new(),
            choice_words: IndexMap::new(),
            described,
        }
    }

    /// Report the choice each choice frame sits on, and return the frames shown.
    pub fn poll(
        &mut self,
        on_screen: &[ScreenText],
        items: &mut Vec<CursorItem>,
        menus: &MenuCursors,
    ) -> Result<HashSet<i64>> {
        let choices = self.choices(menus)?;
        let gone: Vec<i64> = self
            .choices_seen
            .iter()
            .copied()
            .filter(|window| !choices.contains_key(window))
            .collect();
        for window in gone {
            self.choices_seen.shift_remove(&window);
            self.said.forget(window);
        }
        for (&window, (root, on, others)) in &choices {
            let Some(on) = *on else {
                continue;
            };
            self.choices_seen.insert(window);
            let item = self.choice_item(*root, on, others, on_screen)?;
            self.said
                .report(window, item, items, Where::Pane(on), false);
        }
        Ok(choices.keys().copied().collect())
    }

    /// Window -> (root, the choice it sits on, the other choices), for every window shown
    /// beside two or more plain panes of one size in places of their own: the opening's choice of
    /// boy or girl, whose cursor is a window the game slides from one choice's place to another's.
    ///
    /// The choice is None while the window is between two, where there is nothing new to say. A
    /// window directly under its layout's root is a frame, read as one; a window holding panes of
    /// its own is a box the screen is drawn in, as a message window is, not a cursor; and the
    /// frame round a choice of its own stands among that choice's pictures, with no panes beside
    /// it to sit on.
    fn choices(&self, menus: &MenuCursors) -> Result<IndexMap<i64, Choice>> {
        let mut found: IndexMap<i64, Choice> = IndexMap::new();
        for root in self.layouts.roots() {
            let tree = self.layouts.tree(root);
            for (&pane, name) in &tree {
                if name != lyt::WINDOW_CLASS || menus.windows.contains(&pane) {
                    continue;
                }
                let parent = self.panes.parent(pane)?;
                if parent == 0 || parent == root || !self.layouts.children(root, pane).is_empty() {
                    continue;
                }
                let choices: Vec<i64> = self
                    .layouts
                    .children(root, parent)
                    .into_iter()
                    .filter(|&child| class_of(&tree, child) == Some(lyt::PANE_CLASS))
                    .collect();
                if choices.len() < 2 {
                    continue;
                }
                let mut sizes: Vec<(f64, f64)> = Vec::new();
                for &choice in &choices {
                    sizes.push(self.panes.size(choice)?);
                }
                if distinct_count(&sizes) != 1 {
                    continue;
                }
                let mut places: Vec<(f64, f64)> = Vec::new();
                for &choice in &choices {
                    places.push(self.panes.place(choice)?);
                }
                if distinct_count(&places) != places.len() {
                    continue;
                }
                let placement = self.layouts.placement(pane)?;
                if !placement.is_some_and(|placement| placement.shown) {
                    continue;
                }
                let head = self.layouts.pane(pane)?;
                let (x, y) = (
                    f32(&head, self.layouts.shape.x)?,
                    f32(&head, self.layouts.shape.y)?,
                );
                let on_it: Vec<i64> = choices
                    .iter()
                    .zip(&places)
                    .filter(|&(_choice, &(at_x, at_y))| near(at_x, x) && near(at_y, y))
                    .map(|(&choice, _place)| choice)
                    .collect();
                if on_it.len() > 1 {
                    continue;
                }
                found.insert(
                    pane,
                    (
                        root,
                        on_it.first().copied(),
                        choices
                            .iter()
                            .copied()
                            .filter(|choice| !on_it.contains(choice))
                            .collect(),
                    ),
                );
            }
        }
        Ok(found)
    }

    /// What a choice under a sliding window is: the words under it, or the words of the
    /// textures the other choices do not have.
    ///
    /// The first picture's texture, which names a battle's buttons, would say the white backing
    /// both portraits sit on. What tells the choices apart is what one has and the others do not.
    fn choice_item(
        &mut self,
        root: i64,
        choice: i64,
        others: &[i64],
        on_screen: &[ScreenText],
    ) -> Result<Vec<ScreenText>> {
        let mut under: Vec<ScreenText> = Vec::new();
        for piece in on_screen {
            if self.panes.below(piece.r#box, choice)? {
                under.push(piece.clone());
            }
        }
        let inside = distinct(in_rows(under));
        if !inside.is_empty() {
            return Ok(inside);
        }
        let mut theirs: HashSet<String> = HashSet::new();
        for &other in others {
            theirs.extend(self.words_under(root, other)?);
        }
        let mut said: Vec<String> = Vec::new();
        for word in self.words_under(root, choice)? {
            if !theirs.contains(&word) && !said.contains(&word) {
                said.push(word);
            }
        }
        if said.is_empty() {
            return Ok(Vec::new());
        }
        let placement = self.layouts.placement(choice)?;
        Ok(vec![ScreenText {
            top: placement.is_some_and(|placement| placement.top),
            root,
            parent: choice,
            ..ScreenText::new(choice, said.join(" "), false)
        }])
    }

    /// The words of the textures of every picture under a choice, from its layout's file.
    fn words_under(&mut self, root: i64, choice: i64) -> Result<Vec<String>> {
        if let Some(known) = self.choice_words.get(&(root, choice)) {
            return Ok(known.clone());
        }
        let Some(file) = self.panes.file(root) else {
            return Ok(Vec::new());
        };
        let mut said: Vec<String> = Vec::new();
        for (&pane, name) in &self.layouts.tree(root) {
            if name != lyt::PICTURE_CLASS || !self.panes.below(pane, choice)? {
                continue;
            }
            let Some(texture) = self.panes.texture(pane, &file)? else {
                continue;
            };
            let stem = texture
                .rsplit_once('.')
                .map_or(texture.as_str(), |(stem, _)| stem);
            match self
                .described
                .get(stem)
                .filter(|described| !described.is_empty())
            {
                Some(described) => said.push(described.clone()),
                None => said.extend(py::split(&words(&texture)).into_iter().map(String::from)),
            }
        }
        self.choice_words.insert((root, choice), said.clone());
        Ok(said)
    }
}

/// `len(set(pairs))` of floats: a pair is the same as another by `==`, which a NaN never is.
fn distinct_count(pairs: &[(f64, f64)]) -> usize {
    let mut seen: Vec<(f64, f64)> = Vec::new();
    for pair in pairs {
        if !seen.contains(pair) {
            seen.push(*pair);
        }
    }
    seen.len()
}
