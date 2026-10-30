//! The PlayNav's screens: Pokémon-Amie's and Super Training's readers, answered for as one.
//!
//! The PlayNav's pages, and the apps they open, Amie itself, its decorate screen, its games and
//! Super Training's rounds, each have a reader or a few of their own, and the profile wired each one
//! by hand: Berry Picker alone at eleven places (the deepening round, 2026-10-29). Here they are
//! built from what they share and answered for together: what a sweep found of their objects
//! (`track`), what touch mode calls a picture of theirs (`name`), the targets they offer it beside
//! the touch areas (`targets`), and what each says as its screen changes (`poll`). The profile
//! reaches two by name: Berry Picker, whose clock times touch mode's drag, and Tile Puzzle, whose
//! play the screen announcer's clock waits on.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::config::Amie;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::games::common::layout::Layouts;
use crate::games::common::pokemon;
use crate::games::common::touch::{Idle, ImageOf, Off, Target};
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::game::Game;
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::screens::amie::{self, AmiePage, Goods, Puffs};
use crate::games::gen6::screens::berry_picker::BerryPicker;
use crate::games::gen6::screens::puzzle::{self, TilePuzzle};
use crate::games::gen6::screens::townmap;
use crate::games::gen6::screens::training::{Bags, Goals, Round, TrainingPage};
use crate::games::gen6::screens::waiting::Waiting;
use crate::py::{self, Result};

/// The PlayNav's screens' readers, and what each answers for the app running.
pub struct PlayNav {
    image_of: ImageOf,
    pub amie: RefCell<AmiePage>,
    pub puffs: RefCell<Puffs>,
    pub goods: RefCell<Goods>,
    pub puzzle: RefCell<TilePuzzle>,
    pub training: RefCell<TrainingPage>,
    pub goals: Goals,
    pub waiting: Waiting,
    pub bags: Bags,
    pub berry_picker: Rc<RefCell<BerryPicker>>,
    pub sparring: RefCell<Round>,
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

/// A name of "" goes on to the next, as Python's `if name := ...` does.
fn named(name: Option<String>) -> Option<String> {
    name.filter(|name| !name.is_empty())
}

impl PlayNav {
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
        idle: Idle,
        off: Off,
    ) -> PlayNav {
        // The PlayNav's Pokémon-Amie page draws its Pokémon's icon, and says nothing of whose.
        let amie = AmiePage::with_number(
            memory.clone(),
            names.clone(),
            text.species.clone(),
            game.amie,
        );
        // Amie's decorate screen draws its Poké Puffs, and says nothing of which.
        let puffs = Puffs::new(memory.clone(), names.clone(), text.puffs.clone());
        // And its Interior Items, of which it says only how many.
        let goods = Goods::new(memory.clone(), names.clone(), text.goods.clone());
        // Its Tile Puzzle draws each tile's piece in 3D, and says nothing of where it belongs.
        let puzzle = TilePuzzle::new(memory.clone(), layouts.clone());
        // So does Super Training's, with the party; and its rounds' goals are on the top screen.
        let training = TrainingPage::with_icons(
            memory.clone(),
            names.clone(),
            text.species.clone(),
            game.training,
        );
        let goals = Goals::new(layouts.clone(), image_of.clone());
        // A message on the bottom screen waits for a tap on no button: Super Training's welcome.
        let waiting = Waiting::new(layouts.clone(), image_of.clone());
        // And the training bags in its page's list, pictures with no touch area of their own.
        let bags = Bags::new(
            layouts.clone(),
            image_of.clone(),
            images.clone(),
            Some(idle),
            Some(off),
        );
        // Amie's Berry Picker draws the berry each Pokémon wants in a bubble, and says nothing of
        // who wants which.
        let berry_picker = BerryPicker::with_settings(
            memory.clone(),
            names,
            layouts.clone(),
            image_of.clone(),
            images,
            Rc::new(move |found: &pokemon::Pokemon| short(&text, found)),
            Rc::new(py::time::monotonic),
            settings.berry_first,
            settings.one_at_a_time,
        );
        // A round says its score, clock and banners, and warns of a Bot's shot.
        let sparring = Round::new(layouts, image_of.clone(), memory, None);
        PlayNav {
            image_of,
            amie: RefCell::new(amie),
            puffs: RefCell::new(puffs),
            goods: RefCell::new(goods),
            puzzle: RefCell::new(puzzle),
            training: RefCell::new(training),
            goals,
            waiting,
            bags,
            berry_picker: Rc::new(RefCell::new(berry_picker)),
            sparring: RefCell::new(sparring),
        }
    }

    /// Hand each reader what a sweep found, by class: all of it, or with `adding` the new.
    pub fn track(&self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        self.amie.borrow_mut().track(found, adding)?;
        self.puffs.borrow_mut().track(found, adding)?;
        self.goods.borrow_mut().track(found, adding)?;
        self.puzzle.borrow_mut().track(found, adding)?;
        self.berry_picker.borrow_mut().track(found, adding)?;
        self.training.borrow_mut().track(found, adding)?;
        self.sparring.borrow_mut().track(found, adding)
    }

    /// What touch mode calls a picture of these screens', or None.
    pub fn name(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        if running.is_none_or(|running| running == FIELD || running == townmap::APP) {
            if let Some(name) = named(self.amie.borrow().name(picture)?) {
                return Ok(Some(name));
            }
            if let Some(name) = named(self.training.borrow().name(picture)?) {
                return Ok(Some(name));
            }
        }
        if running == Some(amie::ARRANGE) || running == Some(amie::AMIE) {
            if let Some(name) = named(self.puffs.borrow_mut().name(picture)?) {
                return Ok(Some(name));
            }
            return self.goods.borrow_mut().name(picture);
        }
        if running == Some(puzzle::APP) {
            return if (self.image_of)(picture)?.as_deref() == Some(puzzle::FRAME) {
                self.puzzle.borrow().name(picture)
            } else {
                Ok(None)
            };
        }
        Ok(None)
    }

    /// The targets these screens offer touch mode beside its touch areas.
    pub fn targets(&self, running: Option<&str>) -> Result<Vec<Target>> {
        let mut found = self.amie.borrow_mut().strokes(running)?;
        found.extend(self.goals.targets(running)?);
        found.extend(self.berry_picker.borrow().targets(running)?);
        found.extend(self.waiting.targets(running)?);
        found.extend(self.bags.targets(running)?);
        Ok(found)
    }

    /// What each screen says now, as (what the log calls it, the line, its category).
    pub fn poll(&self, running: Option<&str>) -> Result<Vec<(String, String, Category)>> {
        let mut said: Vec<(String, String, Category)> = self
            .sparring
            .borrow_mut()
            .poll(running)?
            .into_iter()
            .map(|(line, category)| ("round".to_string(), line, category))
            .collect();
        said.extend(
            self.berry_picker
                .borrow_mut()
                .poll(running)?
                .into_iter()
                .map(|line| ("berry picker".to_string(), line, Category::Screen)),
        );
        Ok(said)
    }
}
