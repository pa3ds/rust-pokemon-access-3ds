//! The subscreens: the field's bottom-screen pages, as the game names them, and the apps they open.
//!
//! A subscreen is a class of the game's `field::subscreen`: ORAS's PokéNav Plus (`pmn`) and the PSS
//! (`pss`) in both games, and Pokémon-Amie's page (`kawaigari`) and Super Training's (`training`),
//! which ORAS's PlayNav shows and X's bottom screen shows itself. Gen 6 reads Amie's and Super
//! Training's, in ORAS and X alike, and the apps they open: Amie itself, its decorate screen, its
//! games, and Super Training's rounds. Each has a reader or a few of its own, and the profile wired
//! each one by hand, Berry Picker alone at eleven places (the deepening round, 2026-10-29). Here
//! they are built from what they share and answered for together. Each reader answers whichever of
//! these it has, by the same name, for the app running, and nothing in any other app:
//!
//! - `classes`, what the heap is swept for to find its objects, and `track(found, adding)`, what a
//!   sweep found of them, by class;
//! - `name(picture, running)`, what touch mode calls a picture of its screen's;
//! - `targets(running)`, the targets it offers touch mode at points no pane places, beside the
//!   touch areas;
//! - `offer(running)`, the pictures no touch area holds that it offers touch mode as targets, by
//!   pane with their names, which touch mode places and checks as it does a touch area;
//! - `poll(running)`, what it says as its screen changes: (what the log calls it, the line, its
//!   category), read with the screen's words;
//! - `describe(running)`, what it describes of a picture the game shows without words (ADR-0038),
//!   after the screen's words: the narrator's line, `Category.SCENE`, or one only logged, None,
//!   where descriptions are off;
//! - `context(running)` and `press(action, running)`, a screen's own keys: the table of the keys
//!   file they are in while it runs (`field.keys.DEFAULT_CONTEXTS`), and the line a key's action
//!   says.
//!
//! A new one is built here and put in the tuples of what it answers, in the order it is asked in:
//! touch mode names a picture by the first reader with a name for it, and offers targets in order.
//! What touch mode does in the subscreens' apps is theirs too: which need a touch (`TOUCHED`), where
//! Shift alone drags (`DRAGS`), and where a drag is made only on letting go, at the pace of the
//! game's clock (`Subscreens.delayed`, `Subscreens.pace`). The profile reaches Tile Puzzle by name,
//! whose play the screen announcer's clock waits on.
//!
//! Bound by:
//! - ADR-0009: touch mode offers targets in the subscreens' apps that need a touch, Shift alone
//!   drags only in Pokémon-Amie's, and Berry Picker's drag is made on letting go.
//!
//! Python asks each reader by the same name, duck-typed; here each answer is a trait, implemented
//! for the `RefCell` a reader is shared in, and the tuples are lists of them.

pub mod amie;
pub mod berry_picker;
pub mod puzzle;
pub mod super_training;

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::config::Amie;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::games::common::layout::Layouts;
use crate::games::common::pokemon;
use crate::games::common::touch::{ImageOf, Target};
use crate::games::gen6::game::Game;
use crate::games::gen6::rom::archives::GameText;
use crate::py::{self, Result};
use amie::{AmiePage, Goods, Puffs};
use berry_picker::{BerryKeys, BerryPicker, Touching};
use puzzle::TilePuzzle;
use super_training::{Bags, Goals, Round, TrainingPage, Waiting};

/// The subscreens' apps touch mode offers targets in (ADR-0009 as amended 2026-10-29), beside the
/// field whose bottom screen they are: Pokémon-Amie, its decorate screen and its games, and Super
/// Training's regimen menu and rounds. A `frozenset` in Python, only ever tested for membership.
pub const TOUCHED: [&str; 7] = [
    amie::SCREENS[0],
    amie::SCREENS[1],
    amie::SCREENS[2],
    amie::SCREENS[3],
    amie::SCREENS[4],
    super_training::MENU,
    super_training::SPARRING,
];
/// The apps where Shift alone drags, Pokémon-Amie's, the only ones that drag (ADR-0009).
pub const DRAGS: [&str; 5] = amie::SCREENS;

/// A reader handed what a sweep found of its objects, by class.
pub trait Tracked {
    /// What the heap is swept for to find its objects.
    fn classes(&self) -> &'static [&'static str];
    fn track(&self, found: &IndexMap<i64, String>, adding: bool) -> Result<()>;
}

/// A reader naming a picture of its screen's for touch mode.
pub trait Named {
    fn name(&self, picture: i64, running: Option<&str>) -> Result<Option<String>>;
}

/// A reader offering touch mode targets at points no pane places, beside the touch areas.
pub trait Offering {
    fn targets(&self, running: Option<&str>) -> Result<Vec<Target>>;
}

/// A reader offering touch mode pictures no touch area holds, by pane with their names.
pub trait Pictured {
    fn offer(&self, running: Option<&str>) -> Result<IndexMap<i64, String>>;
}

/// A reader saying what it has as its screen changes.
pub trait Polled {
    fn poll(&self, running: Option<&str>) -> Result<Vec<(String, String, Category)>>;
}

/// A reader describing a picture the game shows without words.
pub trait Described {
    fn describe(&self, running: Option<&str>) -> Result<Vec<(String, String, Option<Category>)>>;
}

/// A screen's own keys: the table of the keys file they are in while it runs, and the line a
/// key's action says. Touch mode, which the navigator owns, is handed to `press`.
pub trait Keyed {
    fn context(&self, running: Option<&str>) -> Option<&'static str>;
    fn press(
        &self,
        action: &str,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<String>;
}

impl Keyed for RefCell<BerryKeys> {
    fn context(&self, running: Option<&str>) -> Option<&'static str> {
        self.borrow().context(running)
    }

    fn press(
        &self,
        action: &str,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<String> {
        self.borrow_mut().press(action, running, touch)
    }
}

impl Described for RefCell<TilePuzzle> {
    fn describe(&self, running: Option<&str>) -> Result<Vec<(String, String, Option<Category>)>> {
        self.borrow().describe(running)
    }
}

macro_rules! tracked {
    ($($reader:ty),*) => {$(
        impl Tracked for RefCell<$reader> {
            fn classes(&self) -> &'static [&'static str] {
                &<$reader>::CLASSES
            }
            fn track(&self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
                self.borrow_mut().track(found, adding)
            }
        }
    )*};
}
tracked!(
    AmiePage,
    Puffs,
    Goods,
    TilePuzzle,
    BerryPicker,
    TrainingPage,
    Round
);

impl Named for RefCell<AmiePage> {
    fn name(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        self.borrow().name(picture, running)
    }
}

impl Named for RefCell<TrainingPage> {
    fn name(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        self.borrow().name(picture, running)
    }
}

impl Named for RefCell<Puffs> {
    fn name(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        self.borrow_mut().name(picture, running)
    }
}

impl Named for RefCell<Goods> {
    fn name(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        self.borrow_mut().name(picture, running)
    }
}

impl Named for RefCell<TilePuzzle> {
    fn name(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        self.borrow().name(picture, running)
    }
}

impl Offering for RefCell<AmiePage> {
    fn targets(&self, running: Option<&str>) -> Result<Vec<Target>> {
        self.borrow_mut().targets(running)
    }
}

impl Offering for RefCell<BerryPicker> {
    fn targets(&self, running: Option<&str>) -> Result<Vec<Target>> {
        self.borrow().targets(running)
    }
}

impl Offering for Goals {
    fn targets(&self, running: Option<&str>) -> Result<Vec<Target>> {
        Goals::targets(self, running)
    }
}

impl Pictured for Waiting {
    fn offer(&self, running: Option<&str>) -> Result<IndexMap<i64, String>> {
        Waiting::offer(self, running)
    }
}

impl Pictured for Bags {
    fn offer(&self, running: Option<&str>) -> Result<IndexMap<i64, String>> {
        Bags::offer(self, running)
    }
}

impl Polled for RefCell<Round> {
    fn poll(&self, running: Option<&str>) -> Result<Vec<(String, String, Category)>> {
        self.borrow_mut().poll(running)
    }
}

impl Polled for RefCell<BerryPicker> {
    fn poll(&self, running: Option<&str>) -> Result<Vec<(String, String, Category)>> {
        self.borrow_mut().poll(running)
    }
}

/// The subscreens' readers, and what each answers for the app running.
pub struct Subscreens {
    pub puzzle: Rc<RefCell<TilePuzzle>>,
    berry_picker: Rc<RefCell<BerryPicker>>,
    /// What the heap is swept for, each reader's `classes`.
    pub classes: Vec<&'static str>,
    tracked: Vec<Rc<dyn Tracked>>,
    named: Vec<Rc<dyn Named>>,
    offering: Vec<Rc<dyn Offering>>,
    pictured: Vec<Rc<dyn Pictured>>,
    polled: Vec<Rc<dyn Polled>>,
    described: Vec<Rc<dyn Described>>,
    keyed: Vec<Rc<dyn Keyed>>,
}

/// `yours`: a Pokémon of the player's by the nickname they gave it, or else as the game calls its
/// species and form (Orion, 2026-10-16).
fn yours(text: &GameText, found: &pokemon::Pokemon) -> Option<String> {
    if found.nicknamed && !found.nickname.is_empty() {
        return Some(found.nickname.clone());
    }
    text.pokemon(found.species, found.form)
}

/// `short`: a Pokémon of the player's by its nickname, or else its species alone, with no form:
/// Berry Picker's, where "Pikachu Rock Star wants Pinap Berry" took half a second against a
/// Pokémon's two (Orion, 2026-10-29).
fn short(text: &GameText, found: &pokemon::Pokemon) -> Option<String> {
    if found.nicknamed && !found.nickname.is_empty() {
        return Some(found.nickname.clone());
    }
    text.pokemon(found.species, 0)
}

impl Subscreens {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        image_of: ImageOf,
        images: IndexMap<String, String>,
        text: Rc<GameText>,
        game: &Game,
        settings: &Amie,
        picked: Rc<dyn Fn(&str) -> String>,
        wrong: Rc<dyn Fn(&str) -> String>,
        describing: bool,
    ) -> Subscreens {
        // The Pokémon-Amie page draws its Pokémon's icon, and says nothing of whose; Amie itself is
        // stroked by a slide over it.
        let page = Rc::new(RefCell::new(AmiePage::with_number(
            memory.clone(),
            names.clone(),
            text.species.clone(),
            game.amie,
        )));
        // Amie's decorate screen draws its Poké Puffs, and says nothing of which.
        let puffs = Rc::new(RefCell::new(Puffs::new(
            memory.clone(),
            names.clone(),
            text.puffs.clone(),
        )));
        // And its Interior Items, of which it says only how many.
        let goods = Rc::new(RefCell::new(Goods::new(
            memory.clone(),
            names.clone(),
            text.goods.clone(),
        )));
        // Its Tile Puzzle draws each tile's piece in 3D, and says nothing of where it belongs; its
        // picture is described as a scene is, by the narrator, where the player asks for
        // descriptions and the game's text is read (spike 256).
        let puzzle = Rc::new(RefCell::new(TilePuzzle::with(
            memory.clone(),
            layouts.clone(),
            image_of.clone(),
            text.loaded().then(|| {
                let text = text.clone();
                Rc::new(move |found: &pokemon::Pokemon| yours(&text, found)) as _
            }),
            describing,
        )));
        // So does Super Training's, with the party; and its rounds' goals are on the top screen.
        let training = Rc::new(RefCell::new(TrainingPage::with_icons(
            memory.clone(),
            names.clone(),
            text.species.clone(),
            game.training,
        )));
        let goals = Rc::new(Goals::new(layouts.clone(), image_of.clone()));
        // A message on the bottom screen waits for a tap on no button: Super Training's welcome.
        let waiting = Rc::new(Waiting::new(layouts.clone(), image_of.clone()));
        // And the training bags in its page's list, pictures with no touch area of their own.
        let bags = Rc::new(Bags::new(layouts.clone(), image_of.clone(), images.clone()));
        // Amie's Berry Picker draws the berry each Pokémon wants in a bubble, and says nothing of
        // who wants which.
        let berry_picker = Rc::new(RefCell::new(BerryPicker::with_settings(
            memory.clone(),
            names,
            layouts.clone(),
            image_of.clone(),
            images.clone(),
            Rc::new(move |found: &pokemon::Pokemon| short(&text, found)),
            Rc::new(py::time::monotonic),
            settings.berry_first,
            settings.one_at_a_time,
        )));
        // Its own keys, a berry's and a Pokémon's, heard as the drag's own sounds are and the
        // game's error sound, or said where they are off.
        let berry_keys = Rc::new(RefCell::new(BerryKeys::new(
            berry_picker.clone(),
            images,
            picked,
            wrong,
        )));
        // A round says its score, clock and banners, and warns of a Bot's shot.
        let sparring = Rc::new(RefCell::new(Round::new(layouts, image_of, memory, None)));
        let tracked: Vec<Rc<dyn Tracked>> = vec![
            page.clone(),
            puffs.clone(),
            goods.clone(),
            puzzle.clone(),
            berry_picker.clone(),
            training.clone(),
            sparring.clone(),
        ];
        let classes: IndexSet<&'static str> = tracked
            .iter()
            .flat_map(|reader| reader.classes().iter().copied())
            .collect();
        Subscreens {
            classes: classes.into_iter().collect(),
            tracked,
            named: vec![page.clone(), training, puffs, goods, puzzle.clone()],
            offering: vec![page, goals, berry_picker.clone()],
            pictured: vec![waiting, bags],
            polled: vec![sparring, berry_picker.clone()],
            described: vec![puzzle.clone()],
            keyed: vec![berry_keys],
            puzzle,
            berry_picker,
        }
    }

    /// Hand each reader what a sweep found, by class: all of it, or with `adding` the new.
    pub fn track(&self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        for reader in &self.tracked {
            reader.track(found, adding)?;
        }
        Ok(())
    }

    /// What touch mode calls a picture of these screens', or None.
    pub fn name(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        for reader in &self.named {
            if let Some(name) = reader.name(picture, running)?
                && !name.is_empty()
            {
                return Ok(Some(name));
            }
        }
        Ok(None)
    }

    /// The targets these screens offer touch mode at points no pane places.
    pub fn targets(&self, running: Option<&str>) -> Result<Vec<Target>> {
        let mut found = Vec::new();
        for reader in &self.offering {
            found.extend(reader.targets(running)?);
        }
        Ok(found)
    }

    /// The pictures these screens offer touch mode as targets of their own, by pane, with
    /// their names.
    pub fn offer(&self, running: Option<&str>) -> Result<IndexMap<i64, String>> {
        let mut found: IndexMap<i64, String> = IndexMap::new();
        for reader in &self.pictured {
            found.extend(reader.offer(running)?);
        }
        Ok(found)
    }

    /// What each screen describes now, as (what the log calls it, the line, its category):
    /// a `Category.SCENE` line the narrator's, and one of None only logged.
    pub fn describe(
        &self,
        running: Option<&str>,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        let mut said = Vec::new();
        for reader in &self.described {
            said.extend(reader.describe(running)?);
        }
        Ok(said)
    }

    /// The table of the keys file of the screen running, where it has keys of its own.
    pub fn context(&self, running: Option<&str>) -> Option<&'static str> {
        self.keyed.iter().find_map(|keys| keys.context(running))
    }

    /// Answer one of the screen's own keys, with the line to say.
    pub fn press(
        &self,
        action: &str,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<String> {
        match self
            .keyed
            .iter()
            .find(|keys| keys.context(running).is_some())
        {
            Some(keys) => keys.press(action, running, touch),
            None => Ok(String::new()),
        }
    }

    /// What each screen says now, as (what the log calls it, the line, its category).
    pub fn poll(&self, running: Option<&str>) -> Result<Vec<(String, String, Category)>> {
        let mut said = Vec::new();
        for reader in &self.polled {
            said.extend(reader.poll(running)?);
        }
        Ok(said)
    }

    /// Whether touch mode's drag is made only as it is let go, straight from where it was
    /// picked up to where the keys chose: in Berry Picker, where a berry carried over a Pokémon
    /// wanting another sends it off (spike 260).
    pub fn delayed(&self, running: Option<&str>) -> bool {
        running == Some(berry_picker::APP)
    }

    /// How fast the game runs beside full speed, which a drag made on letting go is timed
    /// by: Berry Picker's clock says (`BerryPicker.speed`).
    pub fn pace(&self) -> f64 {
        self.berry_picker.borrow().speed()
    }
}
