//! The subscreens: the field's bottom-screen pages, as the game names them, and the apps they open.
//!
//! A subscreen is a class of the game's `field::subscreen`: ORAS's PokéNav Plus (`pmn`) and the PSS
//! (`pss`) in both games, and Pokémon-Amie's page (`kawaigari`) and Super Training's (`training`),
//! which ORAS's PlayNav shows and X's bottom screen shows itself. Gen 6 reads Amie's and Super
//! Training's, in ORAS and X alike, and the apps they open: Amie itself, its decorate screen, its
//! games, and Super Training's rounds. Each has a reader or a few of its own, and the profile wired
//! each one by hand, Berry Picker alone at eleven places (the deepening round, 2026-10-29). Here
//! they are built from what they share and answered for together. Each reader answers whichever of
//! these it has, by the same name, for the app running, and nothing in any other app;
//! `Subscreens` reads which app runs once a call (`apps.RunningApp`) and hands it to each:
//!
//! - `classes`, what the heap is swept for to find its objects, and `track(found, adding)`, what a
//!   sweep found of them, by class; an app's own process is not swept for, but handed to the
//!   readers that read it while their app runs (`process`, the running app's
//!   `apps.RunningApp.process`);
//! - `touched(picture, running)`, what touch mode calls a picture of its screen's, named apart
//!   from what screen text calls one (`Subscreens.name`), as the three ways of naming a picture
//!   stay apart (ADR-0006, 2026-10-15);
//! - `targets(running)`, the targets it offers touch mode at points no pane places, beside the
//!   touch areas;
//! - `offer(running)`, the pictures no touch area holds that it offers touch mode as targets, by
//!   pane with their names, which touch mode places and checks as it does a touch area;
//! - `poll(running)`, what it says as its screen changes: (what the log calls it, the line, its
//!   category), read with the screen's words, or of None only logged;
//! - `describe(running)`, what it describes of a picture the game shows without words (ADR-0038),
//!   after the screen's words: the narrator's line, `Category.SCENE`, or one only logged, None,
//!   where descriptions are off;
//! - `context(running)` and `press(action, running)`, a screen's own keys: the table of the keys
//!   file they are in while it runs (`field.keys.DEFAULT_CONTEXTS`), and the line a key's action
//!   says;
//! - `ticking(running)` and `tick()`, what it follows every tick rather than every screen poll:
//!   Head It's balls, for its cue;
//! - `holding(running)`, whether its round holds the heap's whole sweep off: Head It's;
//! - `scored(piece, running)`, whether words on its screen are its score, left unsaid as they
//!   come unless the player asks: Head It's;
//! - `reworded(found)`, the screen's words as they are said, which the screen text reader asks of
//!   every poll's in any app, so each reader rewords only what it knows for its own: the Switch
//!   screen's rows, Head It's numbers.
//!
//! A new one is built here and put in the tuples of what it answers, in the order it is asked in:
//! touch mode names a picture by the first reader with a name for it, and offers targets in order.
//! Screen text names a picture of theirs through `Subscreens.name`: the digits of a result in
//! Amie's games.
//! The PSS has no reader of its own: its screens are the field's bottom screen and touch mode's, but
//! for its own apps, its profile and its icon choice, and the friend list's registering after a
//! trade, apps of their own that need a touch (`pss`). Its O-Power's energy and powers running are
//! one of these readers (`o_power`), which R says on O-Power's screens (`Subscreens.sides`), after
//! a battle's sides and a contest's; and O-Power's screens, the field's, turn rows by themselves,
//! as the PSS's own apps do (`Subscreens.turns`).
//!
//! What touch mode does in the subscreens' apps is theirs too: which need a touch (`TOUCHED`,
//! `Subscreens.offered`), where Shift alone drags (`DRAGS`), and where a drag is made only on
//! letting go, at the pace of the game's clock (`Subscreens.delayed`, `Subscreens.pace`). So is
//! what the screen announcer asks of their apps: whether one of Amie's games times the player
//! (`Subscreens.timed`) and is mid-play (`Subscreens.playing`); and what the claims on a screen's
//! words ask (`gen6.screens.claims`): whether one of the PSS's own apps runs, whose buttons' words
//! are touch mode's (`Subscreens.in_pss`).
//!
//! The game's own sounds the readers play, a key's, a cue's, a bag's and a Pokémon's cry, or the
//! words in their place, are one object they are handed (`gen6.feedback.FeedbackSounds`).
//!
//! Bound by:
//! - ADR-0009: touch mode offers targets in the subscreens' apps that need a touch, Shift alone
//!   drags in Pokémon-Amie's and holds a charge in Super Training's rounds, and Berry Picker's drag
//!   is made on letting go.
//! - ADR-0013: a clock of Amie's games is said at its marks (`timed`), in the PSS's own apps a
//!   button's words are touch mode's (`in_pss`), a row they or O-Power's screens turn back and
//!   forth by themselves is said once each way (`turns`), and R says O-Power's pages (`sides`).
//!
//! Python asks each reader by the same name, duck-typed; here each answer is a trait, implemented
//! for the `RefCell` a reader is shared in, and the tuples are lists of them.

pub mod amie;
pub mod amie_decorate;
pub mod amie_stylus;
pub mod amie_switch;
pub mod berry_picker;
pub mod head_it;
pub mod o_power;
pub mod pss;
pub mod puzzle;
pub mod super_training;

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::config::{CRYING, STYLE_CRY, STYLE_NONE, Tables};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::{FileOf, HOLDING, ImageOf, OnScreen, Target};
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::apps::{FIELD, RunningApp};
use crate::games::gen6::feedback::{
    ALERT_SOUND, CUE_SOUND, FeedbackSounds, HIT_SOUND, PICK_SOUND, WRONG_SOUND,
};
use crate::games::gen6::game::Game;
use crate::games::gen6::rom::archives::GameText;
use crate::py::{self, Result};
use amie::{AmiePage, Moods, StylusKeys};
use amie_decorate::{Goods, Puffs};
use amie_switch::SwitchScreen;
use berry_picker::{BerryKeys, BerryPicker, Process, Touching};
use head_it::{Cue, Frames, HeadIt, HeadItKeys};
use o_power::OPower;
use pss::{COLOSSEUM, DIALOG, FRIEND_ENTRY};
use puzzle::TilePuzzle;
use super_training::{
    Bags, CoreTraining, Gains, Goals, Medals, Round, RoundOptions, TrainingKeys, TrainingPage,
    Waiting,
};

/// The subscreens' apps touch mode offers targets in (ADR-0009 as amended 2026-10-29), beside the
/// field whose bottom screen they are, and beside the PSS's own (`pss.running`): Pokémon-Amie, its
/// decorate screen and its games, Super Training's regimen menu and rounds, the friend list's
/// registering, the Colosseum a battle chosen in the PSS meets in, and the question after a trade
/// or battle there. A `frozenset` in Python, only ever tested for membership.
pub const TOUCHED: [&str; 10] = [
    amie::SCREENS[0],
    amie::SCREENS[1],
    amie::SCREENS[2],
    amie::SCREENS[3],
    amie::SCREENS[4],
    super_training::MENU,
    super_training::SPARRING,
    FRIEND_ENTRY,
    COLOSSEUM,
    DIALOG,
];
/// The apps where Shift alone drags (ADR-0009): Pokémon-Amie's, and Super Training's rounds, where
/// it is the stylus held on a goal to charge a shot. A `frozenset` in Python, only ever tested for
/// membership.
pub const DRAGS: [&str; 6] = [
    amie::SCREENS[0],
    amie::SCREENS[1],
    amie::SCREENS[2],
    amie::SCREENS[3],
    amie::SCREENS[4],
    super_training::SPARRING,
];

/// A reader handed what a sweep found of its objects, by class.
pub trait Tracked {
    /// What the heap is swept for to find its objects.
    fn classes(&self) -> &'static [&'static str];
    fn track(&self, found: &IndexMap<i64, String>, adding: bool) -> Result<()>;
}

/// A reader naming a picture of its screen's for touch mode.
pub trait Touched {
    fn touched(&self, picture: i64, running: Option<&str>) -> Result<Option<String>>;
}

/// A reader offering touch mode targets at points no pane places, beside the touch areas.
pub trait Offering {
    fn targets(&self, running: Option<&str>) -> Result<Vec<Target>>;
}

/// A reader offering touch mode pictures no touch area holds, by pane with their names.
pub trait Pictured {
    fn offer(&self, running: Option<&str>) -> Result<IndexMap<i64, String>>;
}

/// A reader saying what it has as its screen changes. Touch mode, which the navigator owns, is
/// handed to `poll` for a reader that moves a target of its own: Amie's stylus.
pub trait Polled {
    fn poll(
        &self,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>>;
}

/// A reader describing a picture the game shows without words.
pub trait Described {
    fn describe(&self, running: Option<&str>) -> Result<Vec<(String, String, Option<Category>)>>;
}

/// A reader following something every tick rather than every screen poll: Head It's balls and
/// a Balloon Bot's.
pub trait Ticked {
    fn ticking(&self, running: Option<&str>) -> bool;
    fn tick(&self) -> Result<()>;
}

/// A reader whose round holds the heap's whole sweep off while it plays.
pub trait Holding {
    fn holding(&self, running: Option<&str>) -> bool;
}

macro_rules! ticked {
    ($($reader:ty),*) => {$(
        impl Ticked for RefCell<$reader> {
            fn ticking(&self, running: Option<&str>) -> bool {
                self.borrow().ticking(running)
            }
            fn tick(&self) -> Result<()> {
                self.borrow_mut().tick()
            }
        }
        impl Holding for RefCell<$reader> {
            fn holding(&self, running: Option<&str>) -> bool {
                self.borrow().holding(running)
            }
        }
    )*};
}
ticked!(HeadIt, Round);

/// A reader rewording the screen's words as they are said, only what it knows for its own.
pub trait Rewording {
    fn reworded(&self, found: IndexMap<i64, ScreenText>) -> Result<IndexMap<i64, ScreenText>>;
}

impl Rewording for RefCell<SwitchScreen> {
    fn reworded(&self, found: IndexMap<i64, ScreenText>) -> Result<IndexMap<i64, ScreenText>> {
        self.borrow().reworded(found)
    }
}

impl Rewording for RefCell<HeadIt> {
    fn reworded(&self, found: IndexMap<i64, ScreenText>) -> Result<IndexMap<i64, ScreenText>> {
        Ok(self.borrow().reworded(found))
    }
}

impl Rewording for Medals {
    fn reworded(&self, found: IndexMap<i64, ScreenText>) -> Result<IndexMap<i64, ScreenText>> {
        Medals::reworded(self, found)
    }
}

impl Rewording for Gains {
    fn reworded(&self, found: IndexMap<i64, ScreenText>) -> Result<IndexMap<i64, ScreenText>> {
        Gains::reworded(self, found)
    }
}

/// A screen's own keys: the table of the keys file they are in while it runs, and the line a
/// key's action says. Touch mode, which the navigator owns, is handed to `press`.
pub trait Keyed {
    fn context(&self, running: Option<&str>) -> Result<Option<&'static str>>;
    fn press(
        &self,
        action: &str,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<String>;
}

impl Keyed for RefCell<BerryKeys> {
    fn context(&self, running: Option<&str>) -> Result<Option<&'static str>> {
        Ok(self.borrow().context(running))
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

impl Keyed for RefCell<StylusKeys> {
    fn context(&self, running: Option<&str>) -> Result<Option<&'static str>> {
        Ok(self.borrow().context(running))
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

impl Keyed for HeadItKeys {
    fn context(&self, running: Option<&str>) -> Result<Option<&'static str>> {
        Ok(HeadItKeys::context(self, running))
    }

    fn press(
        &self,
        action: &str,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<String> {
        HeadItKeys::press(self, action, running, touch)
    }
}

impl Keyed for TrainingKeys {
    fn context(&self, running: Option<&str>) -> Result<Option<&'static str>> {
        TrainingKeys::context(self, running)
    }

    fn press(
        &self,
        action: &str,
        running: Option<&str>,
        _touch: &mut dyn Touching,
    ) -> Result<String> {
        TrainingKeys::press(self, action, running)
    }
}

impl Tracked for RefCell<OPower> {
    fn classes(&self) -> &'static [&'static str] {
        &OPower::CLASSES
    }
    fn track(&self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        self.borrow_mut().track(found, adding);
        Ok(())
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
    HeadIt,
    TrainingPage,
    Round
);

impl Touched for RefCell<AmiePage> {
    fn touched(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        self.borrow().touched(picture, running)
    }
}

impl Touched for RefCell<TrainingPage> {
    fn touched(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        self.borrow().touched(picture, running)
    }
}

impl Touched for RefCell<Puffs> {
    fn touched(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        self.borrow_mut().touched(picture, running)
    }
}

impl Touched for RefCell<Goods> {
    fn touched(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        self.borrow_mut().touched(picture, running)
    }
}

impl Touched for RefCell<TilePuzzle> {
    fn touched(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        self.borrow().touched(picture, running)
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

impl Offering for RefCell<HeadIt> {
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

/// A reader whose every line is said: each with its category.
fn said(lines: Vec<(String, String, Category)>) -> Vec<(String, String, Option<Category>)> {
    lines
        .into_iter()
        .map(|(logged, line, category)| (logged, line, Some(category)))
        .collect()
}

impl Polled for RefCell<Round> {
    fn poll(
        &self,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        self.borrow_mut().poll_with(running, &|| touch.holding())
    }
}

impl Polled for Goals {
    fn poll(
        &self,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        Goals::poll(self, running, Some(touch))
    }
}

impl Polled for RefCell<CoreTraining> {
    fn poll(
        &self,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        self.borrow_mut().poll_with(running, &|| touch.tapped())
    }
}

impl Polled for RefCell<BerryPicker> {
    fn poll(
        &self,
        running: Option<&str>,
        _touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        Ok(said(self.borrow_mut().poll(running)?))
    }
}

impl Polled for RefCell<HeadIt> {
    fn poll(
        &self,
        running: Option<&str>,
        _touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        self.borrow_mut().poll(running)
    }
}

impl Polled for RefCell<StylusKeys> {
    fn poll(
        &self,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        self.borrow().poll(running, touch)
    }
}

impl Polled for RefCell<Moods> {
    fn poll(
        &self,
        running: Option<&str>,
        touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        self.borrow_mut().poll(running, touch)
    }
}

impl Polled for RefCell<AmiePage> {
    fn poll(
        &self,
        running: Option<&str>,
        _touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        Ok(self.borrow_mut().poll(running))
    }
}

impl Touched for RefCell<SwitchScreen> {
    fn touched(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        self.borrow().touched(picture, running)
    }
}

/// The subscreens' readers, and what each answers for the app running.
pub struct Subscreens {
    apps: Rc<RefCell<RunningApp>>,
    image_of: ImageOf,
    pub(crate) puzzle: Rc<RefCell<TilePuzzle>>,
    pub(crate) o_power: Rc<RefCell<OPower>>,
    berry_picker: Rc<RefCell<BerryPicker>>,
    /// What the heap is swept for, each reader's `classes`.
    pub classes: Vec<&'static str>,
    tracked: Vec<Rc<dyn Tracked>>,
    touching: Vec<Rc<dyn Touched>>,
    offering: Vec<Rc<dyn Offering>>,
    pictured: Vec<Rc<dyn Pictured>>,
    polled: Vec<Rc<dyn Polled>>,
    described: Vec<Rc<dyn Described>>,
    keyed: Vec<Rc<dyn Keyed>>,
    ticked: Vec<Rc<dyn Ticked>>,
    holding: Vec<Rc<dyn Holding>>,
    rewording: Vec<Rc<dyn Rewording>>,
    /// Head It, which a test reaches its cue through, as Python's does through `_ticked[0]`.
    pub(crate) head_it: Rc<RefCell<HeadIt>>,
}

/// `yours`: a Pokémon of the player's by the nickname they gave it, or else as the game calls its
/// species and form.
fn yours(text: &GameText, found: &pokemon::Pokemon) -> Option<String> {
    if found.nicknamed && !found.nickname.is_empty() {
        return Some(found.nickname.clone());
    }
    text.pokemon(found.species, found.form)
}

/// `short`: a Pokémon of the player's by its nickname, or else its species alone, with no form:
/// Berry Picker's, where "Pikachu Rock Star wants Pinap Berry" took half a second against a
/// Pokémon's two, and Head It's, against a ball's fall.
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
        apps: Rc<RefCell<RunningApp>>,
        tables: &Tables,
        sounds: Rc<FeedbackSounds>,
        frames: Frames,
        drawn: Option<Rc<dyn Memory>>,
        file_of: FileOf,
        on_screen: OnScreen,
        party: Rc<dyn Fn() -> Option<i64>>,
    ) -> Subscreens {
        // The settings file's tables, each reader taking its own (`config.Tables`).
        let settings = &tables.amie;
        let image_of_kept = image_of.clone();
        // A screen's own keys are heard as the drag's own sounds are and the game's error sound,
        // or said where they are off; logged either way, as a key's line is.
        let picked: Rc<dyn Fn(&str) -> String> = {
            let sounds = sounds.clone();
            Rc::new(move |berry: &str| {
                sounds.heard(PICK_SOUND, &own(HOLDING, &[("target", &berry)]))
            })
        };
        let wrong: Rc<dyn Fn(&str) -> String> = {
            let sounds = sounds.clone();
            Rc::new(move |why: &str| sounds.heard(WRONG_SOUND, why))
        };
        let (memory_kept, layouts_kept) = (memory.clone(), layouts.clone());
        let running: Rc<dyn Fn() -> Option<String>> = {
            let apps = apps.clone();
            Rc::new(move || apps.borrow().running.clone())
        };
        let process: Process = {
            let apps = apps.clone();
            // Where `app`'s process begins while it is the app running, else None: one read of
            // the manager says, from the moment it starts, where a sweep found it seconds on.
            Rc::new(move |app: &str| {
                let apps = apps.borrow();
                if apps.running.as_deref() == Some(app) {
                    apps.process
                } else {
                    None
                }
            })
        };
        let short: berry_picker::Name = {
            let text = text.clone();
            Rc::new(move |found: &pokemon::Pokemon| short(&text, found))
        };
        // The Pokémon-Amie page draws its Pokémon's icon, and says nothing of whose; Amie itself
        // has its Pokémon beckoned, and petted with a stylus over a picture of its parts the GPU
        // draws, read through `drawn` where the emulator lets that be read.
        let page = AmiePage::with_aim(
            memory.clone(),
            names.clone(),
            text.species.clone(),
            short.clone(),
            game.amie,
            drawn,
            game.amie_player,
        )
        .shared();
        // Its own keys, the arrows, move the stylus, saying the part it comes onto, and keep it
        // there as the Pokémon moves, for touch mode to rub there.
        let stylus_keys = Rc::new(RefCell::new(StylusKeys::new(
            page.clone(),
            wrong.clone(),
            text.amie_numbers.clone(),
        )));
        // Its Switch screen draws the three numbers as rows of icons, and names each row alone.
        // and the party's buttons draw each member's icon, and say nothing of whose.
        let switch = Rc::new(RefCell::new(SwitchScreen::new(
            memory.clone(),
            names.clone(),
            layouts.clone(),
            image_of.clone(),
            text.amie_numbers.clone(),
            party,
            {
                let text = text.clone();
                Rc::new(move |found: &pokemon::Pokemon| yours(&text, found))
            },
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
            tables.ui.descriptions,
        )));
        // So does Super Training's, with the party; and its rounds' goals are on the top screen.
        let training = Rc::new(RefCell::new(TrainingPage::with_icons(
            memory.clone(),
            names.clone(),
            text.species.clone(),
            game.training,
        )));
        // It draws a "!" over its Pokémon when a bag is finished, and says nothing of it; and its
        // regimen menu writes the Pokémon's medals "x 0" beside a picture of one.
        let core = Rc::new(RefCell::new(CoreTraining::with(
            training.clone(),
            layouts.clone(),
            {
                let text = text.clone();
                Rc::new(move |found: &pokemon::Pokemon| yours(&text, found))
            },
            {
                let sounds = sounds.clone();
                Rc::new(move || sounds.play(HIT_SOUND, 0.5))
            },
            {
                let sounds = sounds.clone();
                Rc::new(move || sounds.play(CUE_SOUND, 0.5))
            },
            // When touch mode last tapped is asked of it as the page is polled (`Polled`).
            Rc::new(|| f64::NEG_INFINITY),
        )));
        let medals = Rc::new(Medals::new(running.clone()));
        // A message on the bottom screen waits for a tap on no button: Super Training's welcome.
        let waiting = Rc::new(Waiting::new(layouts.clone(), image_of.clone()));
        // And the training bags in its page's list, pictures with no touch area of their own.
        let bags = Rc::new(Bags::new(layouts.clone(), image_of.clone(), images.clone()));
        // Both of Amie's games below can say a Pokémon by its own cry, toward its side, in its
        // name's place or with it, each as the player asks (`[pa3ds.amie] berry_picker_style`,
        // `head_it_style`, a player's request; a style a game, 2026-11-04).
        let (berry, head) = (
            settings.berry_picker_style.as_str(),
            settings.head_it_style.as_str(),
        );
        // Amie's Berry Picker draws the berry each Pokémon wants in a bubble, and says nothing of
        // who wants which.
        let berry_picker = Rc::new(RefCell::new(BerryPicker::with_settings(
            memory.clone(),
            names.clone(),
            layouts.clone(),
            image_of.clone(),
            images.clone(),
            short.clone(),
            Rc::new(py::time::monotonic),
            settings.berry_first,
            settings.one_at_a_time,
            if CRYING.contains(&berry) {
                sounds.cry.clone()
            } else {
                None
            },
            berry == STYLE_CRY,
            process.clone(),
        )));
        // Its own keys, a berry's and a Pokémon's, heard as the drag's own sounds are and the
        // game's error sound, or said where they are off.
        let berry_keys = Rc::new(RefCell::new(BerryKeys::new(
            berry_picker.clone(),
            images,
            picked,
            wrong.clone(),
        )));
        // Its Head It drops balls of yarn at the party's Pokémon, and names neither the Pokémon nor
        // which one each ball falls at; the game's "!" says when to tap, where the player asks.
        let head_it = Rc::new(RefCell::new(HeadIt::new(
            memory.clone(),
            names,
            layouts.clone(),
            image_of.clone(),
            short,
            if settings.head_it_cue && sounds.has(&[CUE_SOUND]) {
                let sounds = sounds.clone();
                Some(Rc::new(move |column: i64| sounds.cue(column)) as Cue)
            } else {
                None
            },
            settings.head_it_lead,
            Rc::new(py::time::monotonic),
            if CRYING.contains(&head) {
                sounds.cry.clone()
            } else {
                None
            },
            head == STYLE_CRY,
            head != STYLE_NONE,
            frames,
            Rc::new(py::time::sleep),
            process,
            file_of.clone(),
            on_screen.clone(),
        )));
        // And its own keys, a Pokémon's, which tap it, and the score's.
        let head_keys = Rc::new(HeadItKeys::new(head_it.clone(), wrong.clone()));
        // A round says its score, clock and banners, and warns of a Bot's shot. Whether the
        // press is held is asked of touch mode as the round is polled (`Polled`).
        let sparring = Rc::new(RefCell::new(Round::with(
            layouts.clone(),
            image_of.clone(),
            memory,
            RoundOptions {
                shot_fired: Some({
                    let (sounds, mode) = (sounds.clone(), tables.training.shot_cue.clone());
                    Rc::new(move |said: &str| sounds.round_cue(&mode, ALERT_SOUND, said))
                }),
                block_now: Some({
                    let (sounds, mode) = (sounds.clone(), tables.training.block_cue.clone());
                    Rc::new(move |said: &str| sounds.round_cue(&mode, CUE_SOUND, said))
                }),
                lead: tables.training.block_lead,
                ..RoundOptions::default()
            },
        )));
        // A barrier is offered where a shot will meet it, by how far the round's Pokémon stands.
        let goals = Rc::new(Goals::with_flight(layouts.clone(), image_of.clone(), {
            let sparring = sparring.clone();
            Rc::new(move || sparring.borrow().flight())
        }));
        // And R says what the page and a round draw and do not say.
        let training_keys = Rc::new(TrainingKeys::new(
            core.clone(),
            training.clone(),
            sparring.clone(),
            text.training_stats.clone(),
            tables.extras.base_stats,
            layouts,
            image_of,
            wrong,
        ));

        // O-Power's energy and the powers running, which R says on its screens (spike 313): the
        // PSS's list on the bottom screen, with its gauge's pictures, and the Active panel on the
        // top.
        let shown_roots: o_power::Shown = {
            let layouts = layouts_kept.clone();
            Rc::new(move || {
                let mut shown: Vec<i64> = Vec::new();
                for root in layouts.roots() {
                    if layouts
                        .placement(root)?
                        .is_some_and(|placement| placement.shown)
                    {
                        shown.push(root);
                    }
                }
                Ok(shown)
            })
        };

        let drawn_images: o_power::Pictures = {
            let (layouts, image_of) = (layouts_kept, image_of_kept.clone());
            Rc::new(move |root: i64| {
                let mut images: Vec<String> = Vec::new();
                for (pane, kind) in layouts.tree(root) {
                    if kind == lyt::PICTURE_CLASS
                        && layouts
                            .placement(pane)?
                            .is_some_and(|placement| placement.shown)
                        && let Some(image) = image_of(pane)?
                        && !image.is_empty()
                    {
                        images.push(image);
                    }
                }
                Ok(images)
            })
        };

        let o_power = Rc::new(RefCell::new(OPower::new(
            memory_kept,
            on_screen,
            file_of,
            shown_roots,
            drawn_images,
            tables.ui.pauses,
        )));
        // O-Power first: the log names the classes swept for in the order asked, its record's
        // straight after the profile's own (`gen6.profile.HEAP_CLASSES`).
        let tracked: Vec<Rc<dyn Tracked>> = vec![
            o_power.clone(),
            page.clone(),
            puffs.clone(),
            goods.clone(),
            puzzle.clone(),
            berry_picker.clone(),
            head_it.clone(),
            training.clone(),
            sparring.clone(),
        ];
        let classes: IndexSet<&'static str> = tracked
            .iter()
            .flat_map(|reader| reader.classes().iter().copied())
            .collect();
        let gains = Rc::new(Gains::new(
            running,
            &text.training_stats,
            &text.training_gains,
        ));
        // The page first: Amie's `Player` is read afresh for each poll as the others ask.
        let mut polled: Vec<Rc<dyn Polled>> = vec![
            page.clone(),
            core,
            goals.clone(),
            sparring.clone(),
            berry_picker.clone(),
            head_it.clone(),
            stylus_keys.clone(),
        ];
        if settings.descriptions {
            polled.push(Rc::new(RefCell::new(Moods::new(
                page.clone(),
                settings.fidgets,
            ))));
        }
        Subscreens {
            apps,
            image_of: image_of_kept,
            classes: classes.into_iter().collect(),
            tracked,
            touching: vec![
                switch.clone(),
                page.clone(),
                training,
                puffs,
                goods,
                puzzle.clone(),
            ],
            offering: vec![page, goals, berry_picker.clone(), head_it.clone()],
            pictured: vec![waiting, bags],
            polled,
            described: vec![puzzle.clone()],
            keyed: vec![berry_keys, head_keys, stylus_keys, training_keys],
            ticked: vec![head_it.clone(), sparring.clone()],
            holding: vec![head_it.clone(), sparring.clone()],
            rewording: vec![switch, head_it.clone(), medals, gains],
            head_it,
            puzzle,
            o_power,
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

    /// The app running, as `RunningApp` last found it.
    fn running(&self) -> Option<String> {
        self.apps.borrow().running.clone()
    }

    /// What screen text calls a picture of these screens', or None: a digit of a count the
    /// results of Amie's games draw a picture a digit, `0.bclim` to `9.bclim`, too large for the
    /// index (or_amie_stars), named by its file.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        let running = self.running();
        if running
            .as_deref()
            .is_some_and(|running| amie::GAMES.contains(&running))
        {
            Ok(amie::digit((self.image_of)(picture)?.as_deref()))
        } else {
            Ok(None)
        }
    }

    /// What touch mode calls a picture of these screens', or None.
    pub fn touched(&self, picture: i64) -> Result<Option<String>> {
        let running = self.running();
        let running = running.as_deref();
        for reader in &self.touching {
            if let Some(name) = reader.touched(picture, running)?
                && !name.is_empty()
            {
                return Ok(Some(name));
            }
        }
        Ok(None)
    }

    /// The targets these screens offer touch mode at points no pane places.
    pub fn targets(&self) -> Result<Vec<Target>> {
        let running = self.running();
        let running = running.as_deref();
        let mut found = Vec::new();
        for reader in &self.offering {
            found.extend(reader.targets(running)?);
        }
        Ok(found)
    }

    /// The pictures these screens offer touch mode as targets of their own, by pane, with
    /// their names.
    pub fn offer(&self) -> Result<IndexMap<i64, String>> {
        let running = self.running();
        let running = running.as_deref();
        let mut found: IndexMap<i64, String> = IndexMap::new();
        for reader in &self.pictured {
            found.extend(reader.offer(running)?);
        }
        Ok(found)
    }

    /// What each screen describes now, as (what the log calls it, the line, its category):
    /// a `Category.SCENE` line the narrator's, and one of None only logged.
    pub fn describe(&self) -> Result<Vec<(String, String, Option<Category>)>> {
        let running = self.running();
        let running = running.as_deref();
        let mut said = Vec::new();
        for reader in &self.described {
            said.extend(reader.describe(running)?);
        }
        Ok(said)
    }

    /// The table of the keys file of the screen running, where it has keys of its own.
    pub fn context(&self) -> Result<Option<&'static str>> {
        let running = self.running();
        let running = running.as_deref();
        for keys in &self.keyed {
            if let Some(context) = keys.context(running)? {
                return Ok(Some(context));
            }
        }
        Ok(None)
    }

    /// Answer one of the screen's own keys, with the line to say.
    pub fn press(&self, action: &str, touch: &mut dyn Touching) -> Result<String> {
        let running = self.running();
        let running = running.as_deref();
        for keys in &self.keyed {
            if keys.context(running)?.is_some() {
                return keys.press(action, running, touch);
            }
        }
        Ok(String::new())
    }

    /// What each screen says now, as (what the log calls it, the line, its category), one
    /// of None only logged.
    pub fn poll(
        &self,
        touch: &mut dyn Touching,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        let running = self.running();
        let running = running.as_deref();
        let mut said = Vec::new();
        for reader in &self.polled {
            said.extend(reader.poll(running, touch)?);
        }
        Ok(said)
    }

    /// Whether a screen follows something every tick now (`tick`).
    pub fn ticking(&self) -> bool {
        let running = self.running();
        self.ticked
            .iter()
            .any(|reader| reader.ticking(running.as_deref()))
    }

    /// Follow what each screen follows every tick: Head It's balls, for its cue.
    pub fn tick(&self) -> Result<()> {
        let running = self.running();
        let running = running.as_deref();
        for reader in &self.ticked {
            if reader.ticking(running) {
                reader.tick()?;
            }
        }
        Ok(())
    }

    /// Another app has started: Tile Puzzle's next picture is new, even one of the same
    /// Pokémon.
    pub fn started(&self) {
        self.puzzle.borrow().started();
    }

    /// Whether touch mode offers targets in the app running, as one of the subscreens' apps
    /// that need a touch (`TOUCHED`) or one of the PSS's own (`in_pss`).
    pub fn offered(&self) -> bool {
        self.running()
            .as_deref()
            .is_some_and(|running| TOUCHED.contains(&running))
            || self.in_pss()
    }

    /// Whether one of the PSS's own apps runs (`pss.running`): its screens are the field's
    /// bottom screen, where a button's words are touch mode's (ADR-0013).
    pub fn in_pss(&self) -> bool {
        pss::running(self.running().as_deref())
    }

    /// Whether the screen turns rows back and forth by itself (`Heard.turns`): one of the
    /// PSS's own apps, whose profile turns its counts of people met, or O-Power's list or
    /// Active panel up on the field, the panel turning between its field
    /// powers and its battle powers, the page showing kept there for R (`OPower.poll`). Asked
    /// once a screen poll, outside the screen's phase, which would fetch O-Power's pages every
    /// poll after.
    pub fn turns(&self) -> bool {
        if self.running().as_deref() == Some(FIELD) && self.o_power.borrow_mut().up() {
            self.o_power.borrow_mut().poll();
            return true;
        }
        self.in_pss()
    }

    /// What R says on these screens where no battle or contest has a side to say: O-Power's
    /// energy and the powers running, on its screens (spike 313); None elsewhere, and for
    /// Shift and R, the other side.
    pub fn sides(&self, player: bool) -> Result<Option<String>> {
        if player {
            self.o_power.borrow_mut().said()
        } else {
            Ok(None)
        }
    }

    /// Whether one of Amie's games runs, each timing the player, whose clock is said only at
    /// its marks (`common.clocks`).
    pub fn timed(&self) -> bool {
        self.running()
            .as_deref()
            .is_some_and(|running| amie::GAMES.contains(&running))
    }

    /// Whether one of Amie's games is mid-play, its numbers the play's and left unsaid
    /// (`common.clocks`): a Tile Puzzle with a piece away from home.
    pub fn playing(&self) -> Result<bool> {
        Ok(self.running().as_deref() == Some(puzzle::APP) && self.puzzle.borrow().playing()?)
    }

    /// Whether words on the screen are a game's score, what comes at every hit or miss, left
    /// unsaid unless the player asks and never a clock (`common.clocks`): Head It's score, its
    /// "79-HIT COMBO" and its "MISS!", which at a long Unlimited round's pace, a ball a second,
    /// queued up behind the balls' names, so that the names came later and later (players,
    /// 2026-11-04; spike 273's `--again`). Not its chances left, "Chances:2", nor its clock.
    pub fn scored(&self, piece: &ScreenText) -> Result<bool> {
        self.head_it
            .borrow()
            .scored(piece, self.running().as_deref())
    }

    /// The words on the screen as they are said, each reader rewording them in turn: the
    /// Switch screen's labels with their rows, and Head It's numbers without the zeros it pads
    /// them with.
    pub fn reworded(
        &self,
        mut found: IndexMap<i64, ScreenText>,
    ) -> Result<IndexMap<i64, ScreenText>> {
        for reader in &self.rewording {
            found = reader.reworded(found)?;
        }
        Ok(found)
    }

    /// Whether a screen's round holds the heap's whole sweep off: a second of nothing else
    /// read, every 30 s, left a ball of Head It's without its cue and a key pressed for one
    /// tapping late (spike 273, live).
    pub fn holding(&self) -> bool {
        let running = self.running();
        self.holding
            .iter()
            .any(|reader| reader.holding(running.as_deref()))
    }

    /// Whether touch mode's drag is made only as it is let go, straight from where it was
    /// picked up to where the keys chose: in Berry Picker, where a berry carried over a Pokémon
    /// wanting another sends it off (spike 260).
    pub fn delayed(&self) -> bool {
        self.running().as_deref() == Some(berry_picker::APP)
    }

    /// How fast the game runs beside full speed, which a drag made on letting go is timed
    /// by: Berry Picker's clock says (`BerryPicker.speed`).
    pub fn pace(&self) -> f64 {
        self.berry_picker.borrow().speed()
    }
}
