//! What ORAS has alone, as the gen 6 profile asks it (`gen6.own.Own`): its subscreen, the PokéNav
//! Plus (`oras.subscreen`); its contests' standings and a Pokémon's conditions
//! (`oras.screens.contest`, `oras.screens.conditions`); the Pokéblock Kit's two screens
//! (`oras.screens.pokeblock_kit`); the Secret Base PC's decoration list
//! (`oras.screens.secret_base`); the Pokémon its save keeps for the credits (`oras.memories`); and
//! soaring (`oras.field.sky`). Each is built once, here, and answers for its own apps alone.
//!
//! Where two answer one question, the order is this:
//!
//! - the classes swept for: the PokéNav Plus's, then the credits' save's;
//! - what a sweep found: the PokéNav Plus, then the credits' save;
//! - the cursors moved: a contest's portrait picked, a row's move, then the Pokéblock Kit's cursors,
//!   chased until the block's details are said;
//! - the layouts put away: the AreaNav's views not drawn, then the Pokéblock Kit's screen not shown;
//! - the words left unsaid: the AreaNav's lists', then the conditions star's labels.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::discovery::Found;
use crate::games::common::cursors::shared::CursorPoll;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::RunningApp;
use crate::games::gen6::own::{Kit, Own, OwnApp};
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::screens::sources::{Chase, OwnSources};
use crate::games::gen6::screens::townmap::OwnLists;
use crate::games::oras::field::sky;
use crate::games::oras::memories::Memories;
use crate::games::oras::screens::{conditions, contest, pokeblock_kit, secret_base};
use crate::games::oras::subscreen::PokeNav;
use crate::py::Result;

/// ORAS's own readers, asked where the gen 6 profile asks a game's own.
pub struct Oras {
    /// The app running: Python's `_apps`, named apart from `apps`, ORAS's own apps.
    running: Rc<RefCell<RunningApp>>,
    text: Rc<GameText>,
    pauses: bool,
    nav: PokeNav,
    standings: RefCell<contest::Standings>,
    picked: RefCell<contest::Picked>,
    conditions: Option<RefCell<conditions::Conditions>>,
    layouts: Rc<Layouts>,
    files: Rc<LayoutFiles>,
    /// The star's labels, the game's words for the contest categories.
    star_labels: HashSet<String>,
    kit: RefCell<pokeblock_kit::Kit>,
    memories: Memories,
    classes: Vec<String>,
    /// Its own apps: soaring, where the ROM's places are read. A test's sets its own.
    pub(crate) apps: Vec<Rc<dyn OwnApp>>,
}

impl Oras {
    pub fn new(kit: &Kit) -> Self {
        let (apps, text) = (kit.apps.clone(), kit.text.clone());
        let pauses = kit.tables.ui.pauses;
        // The PokéNav Plus: its AreaNav's Pokémon Distribution and its DexNav draw each
        // Pokémon's icon, which no file names, and its AreaNav keeps the layouts of a screen it
        // has left as they were.
        let nav = PokeNav::new(kit.memory.clone(), kit.names.clone(), &text, pauses);
        // A contest's appeals draw who stands where as portraits, said on the keys that say a
        // battle's sides, and the entrant the cursor picks is read every tick. Python raises out
        // of the profile's making, which ends the runtime.
        let words = contest::words(&text).unwrap_or_else(|error| panic!("{error}"));
        let running: Rc<dyn Fn() -> Option<String>> = {
            let apps = apps.clone();
            Rc::new(move || apps.borrow().running.clone())
        };
        let process: Rc<dyn Fn() -> Option<i64>> = {
            let apps = apps.clone();
            Rc::new(move || apps.borrow().process)
        };
        let standings = contest::Standings::new(
            kit.memory.clone(),
            kit.layouts.clone(),
            kit.files.clone(),
            words.labels.clone(),
            running.clone(),
            words.rivals.clone(),
            process.clone(),
        );
        let picked = contest::Picked::new(kit.memory.clone(), running, process);
        // An Extra: the conditions the game draws as a star, said as numbers (ADR-0017).
        let conditions = (kit.tables.extras.conditions && text.loaded()).then(|| {
            RefCell::new(conditions::Conditions::new(
                kit.memory.clone(),
                kit.names.clone(),
                kit.layouts.clone(),
                kit.files.clone(),
                apps.clone(),
                words.categories.clone(),
                words.conditions.clone(),
                kit.speech.clone(),
                kit.log.clone(),
                pauses,
            ))
        });
        let star_labels: HashSet<String> = words.categories.iter().cloned().collect();
        // The Pokéblock Kit keeps the Case's and the Berry Blender's screens on screen at once.
        let pokeblocks = pokeblock_kit::Kit::new(
            kit.memory.clone(),
            apps.clone(),
            kit.layouts.clone(),
            kit.files.clone(),
        );
        // The Pokémon its save keeps for its credits, which their descriptions name (spike 222).
        let memories = Memories::new(kit.memory.clone());
        let mut classes: Vec<String> = nav.classes().into_iter().map(String::from).collect();
        classes.extend(memories.classes());
        // Soaring: while it runs, the keys walk what it lists, which the field's stale lists
        // know nothing of (spike 166); where the ROM's places are not read, the field's lists are
        // walked as they were. Python raises out of the profile's making, which ends the runtime.
        let places = sky::places_of(&text).unwrap_or_else(|error| panic!("{error}"));
        let own_apps: Vec<Rc<dyn OwnApp>> = if places.is_empty() {
            Vec::new()
        } else {
            vec![Rc::new(sky::Soaring::new(
                kit.memory.clone(),
                places,
                kit.speech.clone(),
                kit.log.clone(),
            ))]
        };
        Oras {
            running: apps,
            text,
            pauses,
            nav,
            standings: RefCell::new(standings),
            picked: RefCell::new(picked),
            conditions,
            layouts: kit.layouts.clone(),
            files: kit.files.clone(),
            star_labels,
            kit: RefCell::new(pokeblocks),
            memories,
            classes,
            apps: own_apps,
        }
    }

    /// `Oras`, as `Game.own` builds it.
    pub fn of(kit: &Kit) -> Rc<dyn Own> {
        Rc::new(Oras::new(kit))
    }

    fn app(&self) -> Option<String> {
        self.running.borrow().running.clone()
    }
}

impl OwnLists for Oras {
    fn listing(&self) -> bool {
        self.nav.listing()
    }

    fn lists(
        &self,
        cursors: CursorPoll,
        area: Option<&str>,
        on_screen: &[ScreenText],
    ) -> CursorPoll {
        self.nav.cursors(cursors, area, on_screen)
    }
}

impl OwnSources for Oras {
    fn moved(&self, windows: &[i64], shown: bool) -> Result<Chase> {
        let mut chase = Chase::NONE;
        // A contest's cursor came onto another portrait: a row's move.
        if self.picked.borrow_mut().moved(shown)? {
            chase |= Chase::ROW;
        }
        // The Pokéblock Kit keeps no index of what its cursor is over, and its cursor's place
        // is read instead. The block's count is said with its name, which the game writes as
        // the cursor moves: the screen is polled until both are said (or_pokeblock_kit).
        if self.kit.borrow_mut().moved(windows)? {
            chase |= Chase::DETAILS;
        }
        Ok(chase)
    }

    fn put_away(&self, works: &IndexMap<i64, i64>) -> Result<HashSet<i64>> {
        let mut not_drawn = self.nav.put_away(works)?;
        not_drawn.extend(self.kit.borrow_mut().put_away()?);
        Ok(not_drawn)
    }

    fn before(&self, on_screen: &[ScreenText]) -> Result<Vec<(String, String)>> {
        // A contest's portrait draws no words, and the panel names it a second and a half after
        // the cursor comes onto it.
        let entrant = self.standings.borrow_mut().poll(on_screen)?;
        Ok(match entrant {
            None => Vec::new(),
            Some(entrant) => vec![("contest entrant".to_string(), entrant)],
        })
    }

    fn unsaid(&self, area: Option<&str>, on_screen: &[ScreenText]) -> HashSet<i64> {
        // The star's labels have no values: the conditions Extra says them with theirs.
        let mut unsaid = self.nav.unsaid(area, on_screen);
        unsaid.extend(conditions::star_labels(
            &*self.layouts,
            &*self.files,
            on_screen,
            &self.star_labels,
        ));
        unsaid
    }
}

impl Own for Oras {
    fn classes(&self) -> Vec<String> {
        self.classes.clone()
    }

    fn apps(&self) -> Vec<Rc<dyn OwnApp>> {
        self.apps.clone()
    }

    fn track(&self, found: &Found) -> Result<()> {
        self.nav.track(&found.heap, found.partial)?;
        self.memories.track(found);
        Ok(())
    }

    fn named(&self, picture: i64) -> Result<Option<String>> {
        self.nav.name(picture, self.app().as_deref())
    }

    fn touched(&self, picture: i64) -> Result<Option<String>> {
        self.nav.touched(picture, self.app().as_deref())
    }

    fn said_off(&self, picture: i64) -> Result<bool> {
        self.nav.said_off(picture, self.app().as_deref())
    }

    fn drawing(&self) -> Result<bool> {
        self.nav.drawing()
    }

    fn owns(&self, piece: &ScreenText) -> bool {
        self.nav.owns(piece)
    }

    fn reworded(&self, found: IndexMap<i64, ScreenText>) -> IndexMap<i64, ScreenText> {
        // The Secret Base PC's greyed categories, said empty.
        secret_base::emptied(found, &*self.files, &*self.layouts)
    }

    fn sides(&self, player: bool) -> Result<Option<String>> {
        self.standings.borrow().said(player, self.pauses)
    }

    fn heard(&self, texts: &[&str]) -> Result<()> {
        self.standings.borrow_mut().heard(texts)
    }

    fn after(&self, on_screen: &[ScreenText]) -> Result<Vec<(String, String)>> {
        // After the screen's words, which name the place first: the Pokémon Distribution's.
        self.nav.poll(on_screen, self.app().as_deref())
    }

    fn last(
        &self,
        under: &[String],
        on_screen: &[ScreenText],
        settling: &IndexSet<i64>,
        waiting: bool,
    ) -> Result<()> {
        // The words of the summary's page shown alone: its other pages keep the star on screen.
        if let Some(conditions) = &self.conditions {
            conditions
                .borrow_mut()
                .poll(under, on_screen, settling, waiting)?;
        }
        Ok(())
    }

    fn recall(&self, kind: &str, number: i64) -> Result<Option<String>> {
        if kind == "yours" {
            return self.memories.nickname(number);
        }
        let found = if kind == "memory" {
            self.memories.pokemon(number)?
        } else {
            None
        };
        Ok(found.and_then(|(species, form)| self.text.pokemon(species, form)))
    }
}
