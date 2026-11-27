//! What one game has alone, read by its own package and asked by the gen 6 profile.
//!
//! The profile builds what every gen 6 game is read with: the memory, its classes, the layouts and
//! their files, the app running, the game's text, speech and the log, and the settings tables. A
//! game's own readers are built from those, handed over whole as one `Kit`, by one object a game,
//! its `Own` (`Game.own`), which answers what gen 6 asks at the places it asks it: each slot below
//! is one such place, and does nothing in a game with nothing there. A game's package overrides the
//! slots it has an answer for, and where two of its readers answer one slot, its `Own` says which
//! comes first. ORAS's holds the PokéNav Plus, the contests, the Pokéblock Kit, the credits'
//! Pokémon, the Secret Base PC and soaring (`oras.own`); X's, its opening (`xy.own`).
//!
//! Where gen 6 asks a slot among its own readers, the order is the profile's, and the slot's
//! docstring says where it comes. A consumer handed the `Own` for a few of its slots types it as a
//! protocol of its own, with those alone: the cursor sources (`sources.OwnSources`) and the town map
//! (`townmap.OwnLists`).
//!
//! Bound by:
//! - ADR-0006: gen 6 never names a game's own thing; what one game has alone is built by its own
//!   package.
//!
//! Python's `Own` is a class whose slots do nothing, which a game's subclasses override; here it
//! is a trait whose methods do nothing by default, `Nothing` the game with none of it. The slots the
//! cursor sources and the town map ask are their traits', `OwnSources` and `OwnLists`, which `Own`
//! extends so a game's is handed to them as it is.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexSet;

use crate::core::config::Tables;
use crate::core::discovery::Found;
use crate::core::memory::{Cache, Memory};
use crate::core::rtti::ClassNames;
use crate::core::speech::Speech;
use crate::games::common::announce::Focus;
use crate::games::common::field::things::Field;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::RunningApp;
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::screens::sources::OwnSources;
use crate::py::{Log, Result};

/// What the gen 6 profile builds that a game's own readers are built from.
#[derive(Clone)]
pub struct Kit {
    pub memory: Rc<dyn Memory>,
    /// The executable's classes (`core.rtti`).
    pub names: Rc<ClassNames>,
    pub layouts: Rc<Layouts>,
    /// The layouts' files, which name what their pictures draw.
    pub files: Rc<LayoutFiles>,
    /// Which app runs, and its process.
    pub apps: Rc<RefCell<RunningApp>>,
    /// The game's own words out of the player's ROM, or `NO_TEXT`.
    pub text: Rc<GameText>,
    pub speech: Rc<Speech>,
    pub log: Log,
    /// The settings file's tables (`config.Tables`).
    pub tables: Tables,
    /// A phase of the page cache's to read in (`PageCache.phase`), entered only where a reader
    /// reads: a phase begins by fetching the pages it read last time. Python's is the cache's
    /// bound `phase`; here the cache itself, entered with `memory::phase`.
    pub phase: Rc<dyn Cache>,
}

/// An app only one game has, read by that game's own package: ORAS's soaring
/// (`oras.field.sky`). While it runs, the navigation keys walk what it lists instead of the
/// field's, and it is polled in the field's place.
pub trait OwnApp {
    /// The class its process is (`apps.RunningApp`).
    fn app(&self) -> &'static str;

    /// What the log and the poll's phase call it.
    fn name(&self) -> &'static str;

    /// The navigation keys' categories it lists things in, after the field's.
    fn categories(&self) -> Vec<&'static str>;

    /// Another app has started, this one or any other.
    fn started(&self);

    /// What the navigation keys walk while it runs, read through its process; None while it
    /// says nothing.
    fn listed(&self, process: i64) -> Result<Option<Field>>;

    /// Say what it says of itself, every field poll while it runs.
    fn poll(&self, process: i64) -> Result<()>;
}

/// What a game has alone, as gen 6 asks it: nothing, in a game with none of it.
pub trait Own: OwnSources {
    /// What the heap is swept for to find its readers' objects: first of the game's classes,
    /// before `profile.HEAP_CLASSES`, as the log names a module's classes in the order asked.
    fn classes(&self) -> Vec<String> {
        Vec::new()
    }

    /// Its own apps, which the navigation keys walk and the field poll polls while one runs.
    fn apps(&self) -> Vec<Rc<dyn OwnApp>> {
        Vec::new()
    }

    /// What a sweep found of its classes, after the dialogue's, the app's and the speakers',
    /// before the subscreens'.
    fn track(&self, _found: &Found) -> Result<()> {
        Ok(())
    }

    /// What screen text calls a picture of its screens', after the subscreens' and the
    /// party's names and before the Battle Maison's partners'; None where it has none.
    fn named(&self, _picture: i64) -> Result<Option<String>> {
        Ok(None)
    }

    /// What touch mode calls a picture of its screens', first, before the subscreens'; None
    /// where it has none. Named apart from screen text's, as touch mode names the icons the
    /// player walks to (ADR-0006, 2026-10-15).
    fn touched(&self, _picture: i64) -> Result<Option<String>> {
        Ok(None)
    }

    /// Whether touch mode offers a picture of its screens' though its button is switched
    /// off.
    fn said_off(&self, _picture: i64) -> Result<bool> {
        Ok(false)
    }

    /// Whether it draws something over the town map now, where a tap reaches that and not a
    /// place: touch mode offers the map's places only while it does not.
    fn drawing(&self) -> Result<bool> {
        Ok(false)
    }

    /// Whether words on screen are its own to say, left out of screen text's.
    fn owns(&self, _piece: &ScreenText) -> bool {
        false
    }

    /// What the keys that say a battle's sides say, the player's side or everyone else's,
    /// before the battle's own and the subscreens'; None where it says nothing.
    fn sides(&self, _player: bool) -> Result<Option<String>> {
        Ok(None)
    }

    /// The dialogue pages just said.
    fn heard(&self, _texts: &[&str]) -> Result<()> {
        Ok(())
    }

    /// Every tick, after the lists and choice boxes and before the cursor sources: the
    /// choice under a cursor its screens draw without a layout, when it has just opened or
    /// moved; else None. Read in a phase of its own (`Kit.phase`), entered only while its
    /// screen runs.
    fn focus(&self) -> Result<Option<Focus>> {
        Ok(None)
    }

    /// Every screen poll, in its phase, with every word on screen: what it says after the
    /// screen's words and the cursor sources' lines after them, and before the subscreens'
    /// descriptions, as (what the log calls it, the line).
    fn after(&self, _on_screen: &[ScreenText]) -> Result<Vec<(String, String)>> {
        Ok(Vec::new())
    }

    /// Every screen poll, once the screen's words and the subscreens' descriptions are said:
    /// what it says last, itself. `under` is the words under the cursors, `on_screen` every
    /// word to be heard, `settling` the layouts still being written, and `waiting` whether a
    /// cursor's details are still to be said.
    fn last(
        &self,
        _under: &[String],
        _on_screen: &[ScreenText],
        _settling: &IndexSet<i64>,
        _waiting: bool,
    ) -> Result<()> {
        Ok(())
    }

    /// The word for what a scene names of the player's own save (`scenes.Described`):
    /// `memory`, the Pokémon the save keeps for the credits' moment `number`; `yours`, the
    /// nickname the player gave it. None where it keeps none.
    fn recall(&self, _kind: &str, _number: i64) -> Result<Option<String>> {
        Ok(None)
    }
}

/// `Own(kit)`: a game with nothing of its own.
pub struct Nothing;

impl OwnSources for Nothing {}
impl crate::games::gen6::screens::townmap::OwnLists for Nothing {}
impl Own for Nothing {}

/// `Own`, as `Game.own` builds it by default.
pub fn nothing(_kit: &Kit) -> Rc<dyn Own> {
    Rc::new(Nothing)
}
