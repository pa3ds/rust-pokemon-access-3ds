//! The screens' cursor sources, as the screen poll asks them (ADR-0043): what each screen's own
//! objects say its cursor is on, and what that makes of the poll's cursors and words.
//!
//! A source is read in one of two rhythms. Every tick, `moved` asks each whether its word has
//! changed, a row, a key, a box, a place, and says how the screen poll should follow: once, or
//! each tick until the cursor's word is said (`Chase`). Every screen poll, `put_away` gives the
//! layouts of the views not drawn before the screen is read, and `poll` takes what the cursor
//! reader found and hands back, in one `Polled`, the cursors as each screen's source has them,
//! the title said before the item, the lines said beside the screen's words, and the words left
//! unsaid or unheard. `said` has the last word on the poll's words, once they are filtered.
//!
//! The order the sources are asked in is theirs and is kept here: the Bag's pick-up is taken out
//! before the Mart's jumps, the battle Bag's pockets, a shiny target and the Bag's item see the
//! cursors; and the title is the Bag's pocket, else the Box's box, else the battle Bag's page,
//! else the map's place, though no two run at once. A game's own sources are its `Own`'s
//! (`gen6.own`), asked in their places here through `OwnSources`: its rows after gen 6's, its
//! lines before the screen's words after the Box's mode, and its words unsaid last.
//!
//! Bound by:
//! - ADR-0013: a cursor source says which item is under the cursor and when it moved, the words
//!   still the screen's; a title is said before its item.
//! - ADR-0043: a row, key or ball is read every tick, a change polling the screen at once.
//! - ADR-0044: every cursor source is built and asked here, in the order written above, and the
//!   profile asks this module alone.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ops::{BitOr, BitOrAssign};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::pokemon::Pokemon;
use crate::games::common::screen::{ScreenPoll, ScreenText};
use crate::games::common::screen_announce::Heard;
use crate::games::gen6::apps::RunningApp;
use crate::games::gen6::game::CursorShapes;
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::screens::registered::Registered;
use crate::games::gen6::screens::townmap::OwnLists;
use crate::games::gen6::screens::{
    bag, bag_sort, battle_bag, boxes, counts, mart, partyicons, pokedex, reminder, rows, scrolled,
    shiny, summary, townmap,
};
use crate::py::{Error, Result};

/// How the screen poll follows a source that has moved (`CursorSources.moved`). An `IntFlag` in
/// the Python: `any` is its `&` taken as a truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Chase(pub i64);

impl Chase {
    pub const NONE: Chase = Chase(0);
    /// A title turned, the Box's box or the map's place: the screen polled now, once.
    pub const ONCE: Chase = Chase(1);
    /// A row, a key or an entry: the screen polled now and each tick until the cursor's word is
    /// said (`rows.CHASE`).
    pub const ROW: Chase = Chase(2);
    /// As `ROW`, until the item's details are said too: the Pokéblock Kit's block with its
    /// count.
    pub const DETAILS: Chase = Chase(4);

    /// `bool(self & other)`: whether any of `other`'s flags are set.
    pub fn any(self, other: Chase) -> bool {
        self.0 & other.0 != 0
    }
}

impl BitOr for Chase {
    type Output = Chase;

    fn bitor(self, other: Chase) -> Chase {
        Chase(self.0 | other.0)
    }
}

impl BitOrAssign for Chase {
    fn bitor_assign(&mut self, other: Chase) {
        self.0 |= other.0;
    }
}

/// What the cursor sources make of one screen poll.
#[derive(Debug, Clone, PartialEq)]
pub struct Polled {
    /// The cursors, as each screen's source has them.
    pub cursors: CursorPoll,
    /// A title the objects give, said the poll it changes and before the item.
    pub title: Option<String>,
    /// A cursor -> the name of the item its screen's objects say it is on (`bag.Chosen`).
    pub chosen: IndexMap<i64, String>,
    /// (what the log calls it, the line) said before the screen's words, as a menu's.
    pub before: Vec<(String, String)>,
    /// The same, said after the screen's words, as the screen's.
    pub after: Vec<(String, String)>,
    /// The TextBoxes whose words the sources say, or leave unsaid.
    pub unsaid: HashSet<i64>,
    /// The layout roots of what is on screen but not drawn: the summary's other pages.
    pub unseen: HashSet<i64>,
    /// Words the screen should leave unsaid this poll: the boxes' names the Box has said.
    pub labels: HashSet<String>,
}

/// What the cursor sources ask of a game's own (`gen6.own.Own`), the lists its town map
/// draws among them: ORAS's contests, its Pokéblock Kit and its AreaNav's views.
pub trait OwnSources: OwnLists {
    /// Every tick, after gen 6's rows: how the screen poll is to follow its screens' cursors
    /// (`CursorSources.moved`). `windows` are the menus' cursors on screen, and `shown` whether
    /// the last screen poll found a cursor on screen.
    fn moved(&self, _windows: &[i64], _shown: bool) -> Result<Chase> {
        Ok(Chase::NONE)
    }

    /// Every screen poll, before the screen is read: the works of its layouts not drawn now
    /// (`Layouts.put_away`), by `works` a layout's root -> its work.
    fn put_away(&self, _works: &IndexMap<i64, i64>) -> Result<HashSet<i64>> {
        Ok(HashSet::new())
    }

    /// Every screen poll: what it says before the screen's words, as a menu's, after the PC
    /// Box's mode and before the Bag's sort, as (what the log calls it, the line).
    fn before(&self, _on_screen: &[ScreenText]) -> Result<Vec<(String, String)>> {
        Ok(Vec::new())
    }

    /// Every screen poll: the boxes of the words on screen it leaves unsaid, the town map on
    /// `area`.
    fn unsaid(&self, _area: Option<&str>, _on_screen: &[ScreenText]) -> HashSet<i64> {
        HashSet::new()
    }
}

/// Every screen's cursor source a gen 6 game has, asked in their order.
///
/// The Python's `windows`, the menu cursors on screen, and `targets`, the battle's shiny
/// Pokémon by position, are the profile's cursor reader's and battle reader's: handed to
/// `moved` and `poll`, as those readers are the profile's.
pub struct CursorSources {
    apps: Rc<RefCell<RunningApp>>,
    layouts: Rc<Layouts>,
    registered: Option<Rc<RefCell<Registered>>>,
    labels: HashSet<String>,
    own: Rc<dyn OwnSources>,
    box_mode: Option<boxes::Mode>,
    box_titles: Option<boxes::Titles>,
    bag: Option<Rc<RefCell<bag::Chosen>>>,
    bag_sort: bag_sort::BagSort,
    areanav: townmap::AreaNav,
    reminder: reminder::Pages,
    counts: counts::Counts,
    battle_bag: battle_bag::BattleBag,
    shiny: shiny::Shiny,
    rows: rows::Rows,
    /// Whether the last screen poll found a cursor on screen.
    cursor_shown: Rc<Cell<bool>>,
    mart: mart::Jumps,
    xy_list: pokedex::XyList,
    search_rows: scrolled::Scrolled,
    oras_list: pokedex::OrasList,
    summary: summary::Pages,
}

impl CursorSources {
    /// `menu` is the field menu (`buttons.Buttons.menu`), `choices` the choice boxes', `gauges`
    /// the battle's HP gauges kept, `occupants` and `box_shown` the PC Box's, `own` the game's
    /// own sources, the lists its town map draws among them, and `registered` the Y button's
    /// menu's, where it has one.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        apps: Rc<RefCell<RunningApp>>,
        layouts: Rc<Layouts>,
        files: Rc<LayoutFiles>,
        shapes: &CursorShapes,
        text: &GameText,
        pauses: bool,
        menu: Rc<dyn Fn() -> Result<Option<i64>>>,
        choices: Rc<dyn Fn() -> Vec<i64>>,
        gauges: Rc<dyn Fn() -> Vec<i64>>,
        occupants: Rc<RefCell<boxes::Occupants>>,
        box_shown: boxes::Shown,
        own: Rc<dyn OwnSources>,
        registered: Option<Rc<RefCell<Registered>>>,
    ) -> Self {
        let running: Rc<dyn Fn() -> Option<String>> = {
            let apps = apps.clone();
            Rc::new(move || apps.borrow().running.clone())
        };
        let process: Rc<dyn Fn() -> Option<i64>> = {
            let apps = apps.clone();
            Rc::new(move || apps.borrow().process)
        };
        let box_process: Rc<dyn Fn() -> Option<i64>> = {
            let apps = apps.clone();
            Rc::new(move || {
                let apps = apps.borrow();
                if apps.running.as_deref() == Some(boxes::APP) {
                    apps.process
                } else {
                    None
                }
            })
        };

        let box_mode = shapes
            .r#box
            .map(|r#box| boxes::Mode::new(memory.clone(), r#box.mode, box_process));
        // The box turned to, said from the Box's byte before its title is written (spike 298).
        let box_titles = shapes
            .r#box
            .map(|_| boxes::Titles::new(box_shown, Rc::new(move || occupants.borrow().names())));
        let bag = shapes.bag.clone().map(|shape| {
            let apps = apps.clone();
            Rc::new(RefCell::new(bag::Chosen::new(
                memory.clone(),
                shape,
                Rc::new(move || {
                    let apps = apps.borrow();
                    if apps.running.as_deref() == Some(partyicons::BAG) {
                        apps.process
                    } else {
                        None
                    }
                }),
                text.items.clone(),
                text.pockets.clone(),
            )))
        });
        let bag_sort = bag_sort::BagSort::new({
            let bag = bag.clone();
            Rc::new(move || match &bag {
                None => Ok(false),
                Some(bag) => bag.borrow_mut().carrying(),
            })
        });
        // The place under the AreaNav's map cursor, said from its MapView before the top screen
        // names it (spike 309), and the lists a game draws over the map (`townmap.OwnLists`).
        let areanav = townmap::AreaNav::new(
            own.clone(),
            shapes.map_view.map(|map_view| {
                let map_areas = text.map_areas.clone();
                townmap::Places::new(
                    memory.clone(),
                    running.clone(),
                    process.clone(),
                    Rc::new(move |index: i64| map_areas.get(&index).cloned()),
                    map_view,
                )
            }),
        );
        // The Move Reminder keeps the page of moves' effects it is not drawing.
        let reminder = reminder::Pages::new(memory.clone(), running.clone(), process.clone());
        // A Poké Mart's count, said whole at each change where the screen held it to a pause.
        let counts = counts::Counts::new(layouts.clone(), files.clone(), pauses);
        let battle_bag = battle_bag::BattleBag::new(
            memory.clone(),
            process.clone(),
            layouts.clone(),
            files.clone(),
            gauges.clone(),
        );
        // A shiny Pokémon under a horde's target cursor said so (spike 307).
        let shiny = shiny::Shiny::new(
            memory.clone(),
            running.clone(),
            process.clone(),
            gauges,
            layouts.clone(),
            files.clone(),
        );
        // The X menu's, the options', the party's, the PC's, the move screens' and the title
        // menu's rows and the name entry's key, read every tick while a cursor is on screen: a
        // move polls the screen at once.
        let cursor_shown = Rc::new(Cell::new(false));
        let rows = rows::Rows::new(
            memory.clone(),
            running.clone(),
            process.clone(),
            menu,
            {
                let cursor_shown = cursor_shown.clone();
                Rc::new(move || cursor_shown.get())
            },
            shapes.rows,
        );
        // A Poké Mart's list jumped a page is said where it lands, not where it passes.
        let mart = mart::Jumps::new(
            memory.clone(),
            names.clone(),
            choices,
            layouts.clone(),
            files.clone(),
        );
        // Omega Ruby's says its entry from the list's own rows, the tick it is chosen (spike 310).
        let oras_list = pokedex::OrasList::new(memory.clone(), running, process);
        // The summary draws one page and keeps the rest as drawn: those not shown go unheard.
        let summary = summary::Pages::new(
            memory,
            names,
            apps.clone(),
            layouts.clone(),
            files,
            shapes.summary,
        );
        CursorSources {
            apps,
            layouts,
            registered,
            labels: text.search_labels.clone(),
            own,
            box_mode,
            box_titles,
            bag,
            bag_sort,
            areanav,
            reminder,
            counts,
            battle_bag,
            shiny,
            rows,
            cursor_shown,
            mart,
            // X's Pokédex list says an entry by its number alone; the top screen's panel names
            // it.
            xy_list: pokedex::XyList::new(),
            // Search and sort's moves along rows reading alike, its dashes after another field's.
            search_rows: scrolled::Scrolled::new(),
            oras_list,
            summary,
        }
    }

    /// Every tick: which sources' words have changed since the last, and how the screen poll
    /// is to follow them. `windows` is the menu cursors on screen.
    pub fn moved(&mut self, windows: &dyn Fn() -> Vec<i64>) -> Result<Chase> {
        let mut chase = Chase::NONE;
        // A menu's row moved: the screen is polled now, and each tick until its word is said.
        if self.rows.moved()? || self.oras_list.moved()? {
            chase |= Chase::ROW;
        }
        // A game's own screens' cursors: ORAS's Pokéblock Kit's.
        chase |= self.own.moved(&windows(), self.cursor_shown.get())?;
        // The Box turned to another box, the AreaNav's map cursor came onto another place: the
        // screen is polled now, once, and the heap's whole sweep waits as it does for a cursor
        // moving: one ran into a turn through the boxes, and BOX 4 and BOX 5 went unsaid
        // (or_pc_box_bugs).
        let turned = match &self.box_titles {
            None => false,
            Some(titles) => titles.turned()?,
        };
        if turned || self.areanav.turned()? {
            chase |= Chase::ONCE;
        }
        Ok(chase)
    }

    /// Every screen poll, before the screen is read: the works of the layouts not drawn now
    /// (`Layouts.put_away`). `works` is each layout's root -> its work.
    pub fn put_away(&mut self, works: &IndexMap<i64, i64>) -> Result<HashSet<i64>> {
        let mut not_drawn = self.own.put_away(works)?;
        not_drawn.extend(self.reminder.put_away()?);
        not_drawn.extend(self.summary.put_away()?);
        Ok(not_drawn)
    }

    /// Every screen poll: what the sources make of the screen's words and its cursors.
    /// `chasing` is whether a source's move is being chased (`moved`); `targets` the battle's
    /// shiny Pokémon by position.
    pub fn poll(
        &mut self,
        screen: &ScreenPoll,
        cursors: CursorPoll,
        chasing: bool,
        targets: &dyn Fn() -> IndexMap<i64, Pokemon>,
    ) -> Result<Polled> {
        let on_screen = &screen.on_screen;
        let running = self.apps.borrow().running.clone();
        let (count, counted) = self.counts.poll(on_screen)?;
        let paged = self.battle_bag.poll(on_screen)?;
        let own = self.own.before(on_screen)?;
        let box_mode = match &mut self.box_mode {
            None => None,
            Some(reader) => reader.poll()?,
        };
        let box_title = match &mut self.box_titles {
            None => None,
            Some(titles) => titles.poll()?,
        };
        let place = self.areanav.title()?;
        let slots = match &self.registered {
            None => None,
            Some(reader) => reader.borrow_mut().poll(self.layouts.roots())?,
        };
        let (carried, cursors) = self.bag_sort.poll(cursors, on_screen)?;
        let mut cursors = self.mart.poll(cursors)?;
        if !paged.pockets.is_empty() {
            // The battle Bag's pockets' menu, behind an open pocket's list, is no cursor's.
            let mut items: Vec<CursorItem> = Vec::new();
            for item in &cursors.items {
                let first = item
                    .pieces
                    .first()
                    .ok_or_else(|| Error::index("tuple index out of range"))?;
                if !paged.pockets.contains(&first.root) {
                    items.push(item.clone());
                }
            }
            cursors = CursorPoll { items, ..cursors };
        }
        let cursors = self.shiny.poll(cursors, on_screen, targets)?;
        let (chosen, pocket) = match &self.bag {
            None => (IndexMap::new(), None),
            Some(bag) => bag.borrow_mut().poll(&cursors)?,
        };
        self.summary.poll()?;
        let cursors = self.summary.cursors(cursors, on_screen)?;
        let cursors = self.reminder.cursors(cursors);
        let cursors = self.oras_list.cursors(cursors, on_screen)?;
        let cursors = self.xy_list.poll(running.as_deref(), cursors, on_screen);
        let cursors = self.search_rows.poll(
            cursors,
            &pokedex::search_roots(running.as_deref(), on_screen, &self.labels),
        );
        let cursors = pokedex::labelled(running.as_deref(), cursors, on_screen, &self.labels);
        let mut cursors = self.areanav.cursors(cursors, on_screen);
        if chasing && cursors.items.is_empty() && running.as_deref() == Some(boxes::APP) {
            // The Box's byte moved, and its pointer has yet to say where: what the screen
            // changes meanwhile is that slot's.
            cursors = CursorPoll {
                chased: true,
                ..cursors
            };
        }
        self.cursor_shown.set(!cursors.under.is_empty());
        let mut before: Vec<(String, String)> = Vec::new();
        if let Some(line) = box_mode {
            before.push(("box mode".to_string(), line));
        }
        before.extend(own);
        for (logged, line) in [
            ("bag sort", carried),
            ("registered items", slots),
            ("count", count),
        ] {
            if let Some(line) = line {
                before.push((logged.to_string(), line));
            }
        }
        // `pocket or box_title or paged.turned or place`.
        let title = [pocket, box_title, paged.turned.clone()]
            .into_iter()
            .flatten()
            .find(|title| !title.is_empty())
            .or(place);
        // A battle Bag's pocket just opened says its page after its first item.
        let after = match &paged.opened {
            None => Vec::new(),
            Some(opened) => vec![("battle bag".to_string(), opened.clone())],
        };
        // The summary's pages it does not draw are not heard.
        let mut unsaid = counted;
        unsaid.extend(paged.unsaid.iter().copied());
        unsaid.extend(pokedex::tabled(
            running.as_deref(),
            &screen.texts,
            on_screen,
            &cursors,
            &self.labels,
        ));
        unsaid.extend(self.own.unsaid(self.areanav.area().as_deref(), on_screen));
        let unseen = self.summary.hidden();
        let labels = match &self.box_titles {
            None => HashSet::new(),
            Some(titles) => titles.unsaid()?,
        };
        Ok(Polled {
            cursors,
            title,
            chosen,
            before,
            after,
            unsaid,
            unseen,
            labels,
        })
    }

    /// The poll's cursors and words, filtered, as the sources have the last word on them, and
    /// how the announcer is to hear them.
    pub fn said(&self, cursors: CursorPoll, mut heard: Heard) -> (CursorPoll, Heard) {
        let mut out = std::mem::take(&mut heard.texts);
        if self.apps.borrow().running.as_deref() == Some(rows::FITTING) {
            // The fitting room's cursor walks its tabs, pictures, and on into the list and the
            // buttons drawn in their layout, which open with it as a menu's do: owning the tab
            // alone, as the AreaNav's cursor does, it read Change, Back and the cards as the
            // room opened (x_cyllage_boutique_fitting_room). Their changes are still heard: the
            // cards a move between categories shows are its details.
            let walked: HashSet<i64> = cursors
                .under
                .iter()
                .flatten()
                .map(|piece| piece.root)
                .collect();
            out.retain(|text| text.changed || !text.with_layout || !walked.contains(&text.root));
        }
        // A summary page turned to is said as it opened, its words as they stand, the first
        // cutting off the page before; the button under it a poll later, queued. A page changed
        // with the Pokémon, an Egg's or the one after it, is said as a switch is: the name, then
        // the rest, its layouts coming back to the screen part of it.
        let turned = self.summary.turned();
        let cursors = self.summary.rowed(cursors, &heard.heard);
        let mut reread = turned.clone();
        reread.extend(self.summary.button());
        let rewritten = self.summary.switched();
        (
            cursors,
            Heard {
                texts: out,
                lead: turned,
                reread,
                rewritten,
                ..heard
            },
        )
    }

    /// The name of what a rewrite shows, where a source knows it: the summary's Pokémon.
    pub fn subject(&self) -> Result<Option<String>> {
        self.summary.subject()
    }

    /// The slot the PC Box's pointer has been sent to (`rows.Rows.box_slot`).
    pub fn box_slot(&self) -> Result<Option<i64>> {
        self.rows.box_slot()
    }
}
