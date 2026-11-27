//! ORAS's own subscreen, the PokéNav Plus: what its AreaNav and DexNav draw that X has none of,
//! handed to the gen 6 profile through the game's `Game` (`Game.subscreen`,
//! `gen6.game.OwnSubscreen`) as its contests are.
//!
//! The PokéNav Plus is the field's bottom screen in ORAS (`field::subscreen`), and its apps are the
//! AreaNav, the DexNav, the BuzzNav and the PlayNav. The PlayNav's pages, Pokémon-Amie's and Super
//! Training's, are gen 6's, which X's bottom screen shows itself (`gen6.subscreen`); the BuzzNav's
//! shows are screen text and its buttons touch mode's, with no reader of their own. The AreaNav is
//! an app of its own, `app::townmap`, which the Bag's Town Map and the Fly map open too, and X's
//! Town Map is the same app: the place under its map's cursor, its places for touch mode and the
//! map cursor's tone are gen 6's (`gen6.screens.townmap`, `gen6.screens.mapcursor`). What is ORAS's
//! alone is read here:
//!
//! - the AreaNav's views, which say which of its layouts it draws, and the lists drawn over its
//!   map, Berry Trees', Trainer's Eye's and Secret Bases' (`areanav`);
//! - its navigator's Pokémon (`distribution`): the Pokémon Distribution's in the AreaNav, said a
//!   line a way, and the DexNav's on the field, named for touch mode.
//!
//! `PokeNav` answers for them all, for the app running, and for nothing in any other app.
//!
//! Bound by:
//! - ADR-0037: the DexNav's icons are named with how each is met and "caught", never "not caught",
//!   and offered though the DexNav switches off a species not caught.
//! - ADR-0043: a list over the AreaNav's map reads its rows without the area the map is on, said as
//!   it changes, and Berry Trees' times only with their berries; a view not drawn is put away.

pub mod areanav;
pub mod distribution;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::CursorPoll;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::game::OwnSubscreen;
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::screens::townmap;
use crate::py::Result;
use areanav::{ListRows, Views};
use distribution::{Distribution, DistributionReader};

/// What the log calls the Pokémon Distribution's lines.
pub const LOGGED: &str = "distribution";

/// The PokéNav Plus's readers, ORAS's own, and what each answers for the app running.
pub struct PokeNav {
    distribution: Rc<RefCell<Distribution>>,
    views: RefCell<Views>,
    rows: RefCell<ListRows>,
    reader: RefCell<DistributionReader>,
    gap: &'static str,
}

impl PokeNav {
    /// `PokeNav(memory, names, text, pauses)`.
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        text: &GameText,
        pauses: bool,
    ) -> Self {
        // The AreaNav's Pokémon Distribution and the DexNav draw each Pokémon's icon, which no
        // file names.
        let distribution = Rc::new(RefCell::new(Distribution::new(
            memory.clone(),
            names.clone(),
            text.species.clone(),
        )));
        // The AreaNav keeps the layouts of a screen it has left as they were; its views say.
        let views = RefCell::new(Views::new(memory, names));
        let rows = RefCell::new(ListRows::new());
        // And the AreaNav's Pokémon Distribution draws a place's Pokémon and names none.
        let reader = RefCell::new(DistributionReader::new(distribution.clone(), pauses));
        // The DexNav draws a Pokémon caught in colour, and only those are said so: "Ralts",
        // "Magikarp fishing caught", with commas under pauses (ADR-0037, `Distribution.said`).
        let gap = if pauses { ", " } else { " " };
        PokeNav {
            distribution,
            views,
            rows,
            reader,
            gap,
        }
    }

    /// `PokeNav` as `Game.subscreen` builds it.
    pub fn of(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        text: &GameText,
        pauses: bool,
    ) -> Rc<dyn OwnSubscreen> {
        Rc::new(PokeNav::new(memory, names, text, pauses))
    }
}

impl OwnSubscreen for PokeNav {
    /// What the heap is swept for: the AreaNav's navigator, which says whose icon each
    /// picture of its Pokémon Distribution draws, and its views, which say which of its layouts
    /// it draws.
    fn classes(&self) -> Vec<&'static str> {
        let mut classes = vec![distribution::CLASS];
        classes.extend(areanav::VIEWS.iter().map(|(view, _)| *view));
        classes
    }

    /// Hand each reader what a sweep found, by class: all of it, or with `adding` the new.
    fn track(&self, found: &IndexMap<i64, String>, adding: bool) -> Result<()> {
        self.distribution.borrow_mut().track(
            found
                .iter()
                .filter(|(_, name)| *name == distribution::CLASS)
                .map(|(&at, _)| at),
            adding,
        );
        self.views.borrow_mut().track(found, adding)
    }

    /// What screen text calls a picture of these screens': the Pokémon Distribution draws the
    /// place's Pokémon as icons whose buttons take no tap, named for its reader to say whole
    /// (or_area_nav, spike 232).
    fn name(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        if running == Some(townmap::APP) {
            self.distribution.borrow().name(picture)
        } else {
            Ok(None)
        }
    }

    /// What touch mode calls a picture of these screens', or None. The AreaNav's navigators
    /// are the field's and its own app's: elsewhere they are gone, and asking cost a touch walk
    /// 1,400 reads (movie22). On the field its icons are the DexNav's; in the AreaNav, its
    /// Pokémon Distribution's (or_area_nav).
    fn touched(&self, picture: i64, running: Option<&str>) -> Result<Option<String>> {
        if running == Some(FIELD) {
            return self.distribution.borrow().said(picture, self.gap);
        }
        if running.is_none_or(|running| running == townmap::APP) {
            return self.distribution.borrow().name(picture);
        }
        Ok(None)
    }

    /// Whether touch mode offers a picture though its button is switched off: the DexNav
    /// switches off the icon of a species not caught, which it cannot search for, and the
    /// species is said all the same (ADR-0037).
    fn said_off(&self, picture: i64, running: Option<&str>) -> Result<bool> {
        Ok(running.is_none_or(|running| running == FIELD)
            && self.distribution.borrow().name(picture)?.is_some())
    }

    /// Whether the AreaNav draws one of its views over its map now, where a tap reaches the
    /// view's own buttons, not a place (`Views.drawing`).
    fn drawing(&self) -> Result<bool> {
        self.views.borrow().drawing()
    }

    /// The works of the layouts of every view the AreaNav is not drawing now
    /// (`Views.put_away`), by `works_of` a layout's root -> its work.
    fn put_away(&self, works_of: &IndexMap<i64, i64>) -> Result<HashSet<i64>> {
        self.views.borrow().put_away(works_of)
    }

    /// Whether the AreaNav drew one of its lists over its map at the last `put_away`.
    fn listing(&self) -> bool {
        self.views.borrow().listing()
    }

    /// The cursors, a list over the map read row by row (`ListRows`), each row without the
    /// `area` the map is on: said as the area as the map moves to it, and not again with its
    /// rows.
    fn cursors(
        &self,
        cursors: CursorPoll,
        area: Option<&str>,
        on_screen: &[ScreenText],
    ) -> CursorPoll {
        let drawn = self.views.borrow().list_drawn();
        let mut cursors = self.rows.borrow_mut().poll(cursors, drawn, area, on_screen);
        if let Some(area) = area
            && self.views.borrow().listing()
        {
            cursors = areanav::unplaced(cursors, area);
        }
        cursors
    }

    /// The boxes of the words the lists over the map leave unsaid, with the map on `area`:
    /// Berry Trees' times, said with their berries, and Trainer's Eye's area, said as it
    /// changes, its words above the card no more.
    fn unsaid(&self, area: Option<&str>, on_screen: &[ScreenText]) -> HashSet<i64> {
        let drawn = self.views.borrow().list_drawn();
        let mut unsaid = areanav::berry_times(on_screen, drawn);
        if let Some(area) = area
            && drawn == Some(areanav::TRAINERS)
        {
            unsaid.extend(areanav::place_block(on_screen, area));
        }
        unsaid
    }

    /// What these screens say after the screen's words, as (what the log calls it, the
    /// line): the Pokémon Distribution, a line a way, in the AreaNav's own app, where its
    /// navigators are.
    fn poll(
        &self,
        on_screen: &[ScreenText],
        running: Option<&str>,
    ) -> Result<Vec<(String, String)>> {
        Ok(self
            .reader
            .borrow_mut()
            .poll(on_screen, running == Some(townmap::APP))?
            .into_iter()
            .map(|line| (LOGGED.to_string(), line))
            .collect())
    }

    /// Whether words on screen are these screens' to say, not screen text's: the
    /// Distribution's icons, which left to it ran together as one word.
    fn owns(&self, piece: &ScreenText) -> bool {
        self.reader.borrow().owns(piece)
    }
}
