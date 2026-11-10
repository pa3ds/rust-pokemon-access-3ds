//! A Pokémon's contest conditions, said as numbers where the game draws them as a star: an Extra
//! (ADR-0017), `[pa3ds.extras] conditions`, off unless asked for.
//!
//! The summary's contest page and the Pokéblock Case's top screen draw a star whose five points
//! reach as far as the Pokémon's coolness, beauty, cuteness, cleverness and toughness. The layout
//! draws only its outline, `star_base.bclim`; the shape inside is drawn outside every layout, and
//! what draws it was not found (spike 196, movie33). The numbers are the Pokémon's record's, so
//! they are said from it, more exact than the star, and only for a player who asks: "Conditions
//! Coolness 116 Beauty 70 Cuteness 70 Cleverness 70 Toughness 70". Sheen is left out: the game's
//! text never names it, and in ORAS it changes nothing.
//!
//! Whose star it is: the summary points at the Pokémon it shows; the Pokéblock Case at the party,
//! and draws the star of the one under its cursor, named there.
//!
//! Bound by:
//! - ADR-0017: a Pokémon's conditions are an Extra and ship off.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use indexmap::IndexSet;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::{Category, Speech};
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::pokemon;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::RunningApp;
use crate::games::gen6::game::ConditionsReader;
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::screens::partyicons::SLOTS;
use crate::games::gen6::screens::summary;
use crate::py::{Log, Result};

pub const CASE: &str = "N3app10poroc_case4ProcE";
/// The Pokéblock Case's process -> the party's list, six `PokemonParam`s (spike 196).
pub const PARTY: i64 = 0x130;
/// The star's outline, drawn wherever the conditions are (spike 196).
pub const STAR: &str = "star_base.bclim";

/// The conditions as they are said: "Conditions Coolness 116 Beauty 70", or with pauses
/// "Conditions: Coolness 116, Beauty 70".
///
/// Python's `pauses` defaults to False.
pub fn said(heading: Option<&str>, words: &[String], values: &[i64], pauses: bool) -> String {
    let pieces = if pauses { ", " } else { " " };
    let told = words
        .iter()
        .zip(values)
        .map(|(word, value)| format!("{word} {value}"))
        .collect::<Vec<_>>()
        .join(pieces);
    match heading {
        Some(heading) if !heading.is_empty() => {
            format!("{heading}{}{told}", if pauses { ": " } else { " " })
        }
        _ => told,
    }
}

/// Says the conditions of the Pokémon whose star is on screen, once a Pokémon.
pub struct Conditions {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    layouts: Rc<Layouts>,
    files: Rc<LayoutFiles>,
    apps: Rc<RefCell<RunningApp>>,
    /// The game's word for each category, coolness first (`GameText.contest_categories`).
    words: Vec<String>,
    heading: Option<String>,
    speech: Rc<Speech>,
    log: Log,
    pauses: bool,
    /// A layout's pane names -> whether it draws the star.
    stars: HashMap<Vec<String>, bool>,
    /// The Pokémon and its conditions last said, while its star stays on screen.
    said: Option<(i64, Vec<i64>)>,
}

impl Conditions {
    /// Python's `pauses` defaults to False.
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        files: Rc<LayoutFiles>,
        apps: Rc<RefCell<RunningApp>>,
        words: Vec<String>,
        heading: Option<String>,
        speech: Rc<Speech>,
        log: Log,
        pauses: bool,
    ) -> Self {
        Conditions {
            memory,
            names,
            layouts,
            files,
            apps,
            words,
            heading,
            speech,
            log,
            pauses,
            stars: HashMap::new(),
            said: None,
        }
    }

    /// The reader, with the game's words for the categories and its heading from the ROM,
    /// as the gen 6 profile builds it once the Extra is on (`Game.conditions`).
    pub fn of(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        files: Rc<LayoutFiles>,
        apps: Rc<RefCell<RunningApp>>,
        text: &GameText,
        speech: Rc<Speech>,
        log: Log,
        pauses: bool,
    ) -> Box<dyn ConditionsReader> {
        Box::new(Conditions::new(
            memory,
            names,
            layouts,
            files,
            apps,
            text.contest_categories.clone(),
            text.contest_conditions.clone(),
            speech,
            log,
            pauses,
        ))
    }
}

impl ConditionsReader for Conditions {
    /// Say the conditions where a star has come on screen or turned to another Pokémon, once
    /// the words drawn with it have been said: polled after them, they come last, and the
    /// Case's "Blaze" does not cut off Blaze's. `under` is the words under the cursors, which
    /// name the Case's Pokémon; `settling` the layouts whose words are still being written;
    /// `waiting` whether a cursor's details are still to be said.
    ///
    /// Python's `waiting` defaults to False.
    fn poll(
        &mut self,
        under: &[String],
        on_screen: &[ScreenText],
        settling: &IndexSet<i64>,
        waiting: bool,
    ) -> Result<()> {
        let Some(root) = self.star_shown()? else {
            self.said = None;
            return Ok(());
        };
        // Words still being written, as a description is when the cursor moves along the moves,
        // put the conditions off without forgetting whose were said.
        if waiting || settling.contains(&root) || !on_screen.iter().any(|piece| piece.root == root)
        {
            return Ok(());
        }
        let param = self.shown(under)?;
        let values = match param {
            None => None,
            Some(param) => pokemon::contest_conditions(&*self.memory, &self.names, param)?,
        };
        let (Some(param), Some(values)) = (param, values) else {
            return Ok(());
        };
        if self.said.as_ref() == Some(&(param, values.clone())) {
            return Ok(());
        }
        self.said = Some((param, values.clone()));
        let text = said(self.heading.as_deref(), &self.words, &values, self.pauses);
        (self.log)(&format!("conditions: {text}"));
        self.speech.say(&text, Category::Screen);
        Ok(())
    }
}

impl Conditions {
    /// The `PokemonParam` whose star is drawn, or None.
    fn shown(&self, under: &[String]) -> Result<Option<i64>> {
        let (process, running) = {
            let apps = self.apps.borrow();
            (apps.process, apps.running.clone())
        };
        let Some(process) = process else {
            return Ok(None);
        };
        if running.as_deref() == Some(summary::SUMMARY) {
            return summary::shown(&*self.memory, &self.apps.borrow());
        }
        if running.as_deref() != Some(CASE) {
            return Ok(None);
        }
        let party = u32(&self.memory.read(process + PARTY, 4)?, 0)?;
        if party == 0 {
            return Ok(None);
        }
        let members = self.memory.read(party, 4 * SLOTS)?;
        // ponytail: by name, so two of the party under one name say nothing; the Case's cursor
        // index would settle it, and was not found.
        let mut named: Vec<i64> = Vec::new();
        for i in 0..SLOTS {
            let param = u32(&members, 4 * i)?;
            let found = pokemon::read(&*self.memory, &self.names, param, pokemon::SPECIES_MAX)?;
            if found.is_some_and(|found| under.contains(&found.nickname)) {
                named.push(param);
            }
        }
        Ok(if named.len() == 1 {
            Some(named[0])
        } else {
            None
        })
    }

    /// The root of the layout on screen that draws the star, or None.
    fn star_shown(&mut self) -> Result<Option<i64>> {
        for root in self.layouts.roots() {
            let names = self.layouts.pane_names(root);
            if !self.stars.contains_key(&names) {
                let Some(file) = self.files.matching(&names) else {
                    continue; // Its file may not have been read yet: asked again next poll.
                };
                let drawn = file
                    .textures
                    .values()
                    .any(|drawn| drawn.iter().any(|texture| texture == STAR));
                self.stars.insert(names.clone(), drawn);
            }
            let placement = self.layouts.placement(root)?;
            if self.stars[&names] && placement.is_some_and(|placement| placement.shown) {
                return Ok(Some(root));
            }
        }
        Ok(None)
    }
}
