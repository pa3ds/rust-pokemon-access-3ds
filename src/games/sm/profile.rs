//! The Sun and Moon profile: dialogue with the speaker the game names, menus and screens, and the
//! field.
//!
//! The runtime (`app`) knows when to sweep and when to poll, as for ORAS. A sweep looks for message
//! windows, talk windows and the `G2DUtil`s that hold them, inside the game's heaps only (`heaps`); a
//! poll takes in any window a util has come to hold since, then reads the pages of each window with
//! ORAS's dialogue reader, given Sun's classes and TextBox fields (`msgwin`), and says each page
//! with the name on its plate when the speaker changes (`talk`). Menus and screens are read by
//! ORAS's layout, screen text and cursor readers over Sun's panes (`lyt`), found through the layout
//! works a sweep finds and read only inside the blocks the last walk found; the language choice,
//! whose buttons are pictures, by its own reader, in the words of the player's ROM (`langselect`,
//! `text`). A battle's HP box is said a side at a time from the gauges behind it, the enemy's as a
//! percentage (`battle`). The field is read from its actors and move models
//! and the ROM's event data (`field`, `zones`), and pa3ds's keys walk it as in ORAS, P finding the
//! way on foot through the walls out of the ROM (`walls`).

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::config::{Navigation, Ui};
use crate::core::discovery::{Discovery, Found};
use crate::core::heaps::ReadEach;
use crate::core::memory::{self, Cache, Memory};
use crate::core::names::Names;
use crate::core::rtti::{ClassNames, demangle};
use crate::core::speech::{Category, Speech};
use crate::games::common::announce::Announcements;
use crate::games::common::cursors::reader::CursorReader;
use crate::games::common::dialogue::DialogueReader;
use crate::games::common::field::ground::Ground;
use crate::games::common::field::guide::FieldSource;
use crate::games::common::field::keys::Bindings;
use crate::games::common::field::navigate::{self, Navigator};
use crate::games::common::field::things::{Field, Interactable, describe};
use crate::games::common::layout::Layouts;
use crate::games::common::lyt as common_lyt;
use crate::games::common::scenes::{Scene, Scenes};
use crate::games::common::screen::{ScreenPoll, ScreenText, ScreenTextReader};
use crate::games::common::screen_announce::ScreenAnnouncer;
use crate::games::common::session::Profile;
use crate::games::sm::apps::{self, RunningApp};
use crate::games::sm::battle::{self, SunBattleReader};
use crate::games::sm::field::{self, SunFieldReader, Walls};
use crate::games::sm::heaps::{Blocks, InBlocks};
use crate::games::sm::langselect::{self, LangSelectReader};
use crate::games::sm::talk::{self, Speakers};
use crate::games::sm::text::SunText;
use crate::games::sm::zones::SunZones;
use crate::games::sm::{lyt, msgwin};
use crate::py::{Log, Result};

/// Message windows, the talk windows and utils that name their speakers, the language choice,
/// and layout works through the DrawInfo each embeds, as in ORAS: every TextBox and cursor on screen
/// is under a layout's tree. The field's: the move models' manager, the states that lead to the
/// field, and the actors of its things and doors (`field`). The process manager, which says when
/// another app starts (`apps`). And the battle's HP gauges, whose class its module brings when the
/// battle starts (`battle`).
///
/// `HEAP_CLASSES`: a function, since a constant cannot join the other modules' arrays.
pub fn heap_classes() -> Vec<&'static str> {
    let mut classes: Vec<&'static str> = msgwin::CLASSES.to_vec();
    classes.extend([
        talk::CLASS,
        talk::UTIL_CLASS,
        langselect::CLASS,
        common_lyt::DRAWINFO_CLASS,
    ]);
    classes.extend(field::CLASSES);
    classes.extend([apps::MANAGER_CLASS, battle::GAUGE_CLASS]);
    classes
}

/// Without a message window class there is nothing to read: not this game, or not this version.
pub const REQUIRED: [&str; 2] = msgwin::CLASSES;

/// The zone last listed and what was in it (people by kind alone, since they walk).
type Listed = (i64, Vec<(String, String, Option<(i64, i64)>)>);

/// Python's `SunProfile(...)` arguments after `memory`, `names`, `speech` and `log`, each as Python
/// defaults it: set what differs with `..Options::default()`.
///
/// `read_each` is `memory`'s own where it has one, as an `ExactCache` does (`SunFieldReader`
/// looks for it on the memory it is given, which Rust cannot ask a `dyn Memory`).
#[derive(Default)]
pub struct Options {
    pub ui: Option<Ui>,
    pub navigation: Option<Navigation>,
    pub text: Option<Rc<SunText>>,
    pub zones: Option<Rc<SunZones>>,
    pub named: Option<Rc<Names>>,
    /// None is `DEFAULT_BINDINGS`.
    pub keys: Option<Bindings>,
    pub blocks: Option<Blocks>,
    pub walls: Option<Rc<dyn Walls>>,
    /// Python's `phases`: the cache whose `phase` each poll opens. None is `nullcontext`, no phase.
    pub phases: Option<Rc<dyn Cache>>,
    pub read_each: Option<ReadEach>,
    /// pa3ds's own words for the cutscenes, by demo (`scenes::load`).
    pub scenes: Vec<Scene>,
    /// Each beat's words by its id (`Described.said`); None words none.
    pub said: Option<Rc<dyn Fn(&str) -> Option<String>>>,
}

/// Sun's readers, and what each sweep and poll does with them.
///
/// `memory` should read exactly what it is asked for, as `RpcMemory` does, not a page around it:
/// the byte after an object may be the GPU's (`heaps`). A dialogue poll is a dozen reads, sent one
/// after another at about 73 us each (spike 2).
///
/// Every method takes `&self` and each reader sits in a cell, since the runtime re-enters the
/// profile from inside its own calls (a sweep's `between` polls dialogue, answers keys and polls
/// the screen). A cell is borrowed only for the call into its reader, which never comes back into
/// the profile.
pub struct SunProfile {
    memory: Rc<dyn Memory>,
    phase: Option<Rc<dyn Cache>>,
    names: Rc<ClassNames>,
    speech: Rc<Speech>,
    log: Log,
    pub layouts: Rc<Layouts>,
    pub dialogue: RefCell<DialogueReader>,
    pub screen: RefCell<ScreenTextReader>,
    pub cursors: RefCell<CursorReader>,
    pub screen_announcer: RefCell<ScreenAnnouncer>,
    pub speakers: Rc<Speakers>,
    pub announcements: Rc<RefCell<Announcements>>,
    pub langselect: RefCell<LangSelectReader>,
    pub battle: RefCell<SunBattleReader>,
    pub apps: RefCell<RunningApp>,
    pub scenes: Option<RefCell<Scenes>>,
    /// The message windows read: the last sweep's, and any a util has held since.
    windows: RefCell<Vec<i64>>,
    utils: RefCell<Vec<i64>>,
    zones: Option<Rc<SunZones>>,
    pub field: Rc<SunFieldReader>,
    pub navigator: RefCell<Navigator>,
    /// The name of the place the player was last polled in.
    place: RefCell<Option<String>>,
    /// The zone the player was last polled in.
    zone: Cell<Option<i64>>,
    /// The zone last listed and what was in it (people by kind alone, since they walk).
    listed: RefCell<Option<Listed>>,
    /// Whether a sweep has been taken in since the last listing.
    relist: Cell<bool>,
}

/// What the navigator reads the map through: the reader's own four, as Python's structural
/// `FieldSource` takes them.
impl FieldSource for SunFieldReader {
    fn visit(&self) -> Option<(i64, i64)> {
        SunFieldReader::visit(self)
    }

    fn poll(&self) -> Result<Option<Field>> {
        SunFieldReader::poll(self)
    }

    fn interactables(&self, r#where: &Field) -> Result<Field> {
        SunFieldReader::interactables(self, r#where)
    }

    fn ground(&self, r#where: &Field) -> Result<Ground> {
        SunFieldReader::ground(self, r#where)
    }
}

impl SunProfile {
    /// `SunProfile(memory, names, speech, log)`: every other argument at its default.
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        speech: Rc<Speech>,
        log: Log,
    ) -> Self {
        SunProfile::with(memory, names, speech, log, Options::default())
    }

    /// `phases` opens a named phase of `memory`'s cache round each poll, as ORAS's page cache
    /// has (`memory.ExactCache.phase`): the reads a poll makes are fetched together at the start
    /// of the next poll of its kind, where each was sent alone (issue 4). `scenes` is pa3ds's
    /// words for the cutscenes, by demo (`scenes.load`), each beat worded by `said`
    /// (`Described.said`).
    pub fn with(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        speech: Rc<Speech>,
        log: Log,
        options: Options,
    ) -> Self {
        let Options {
            ui,
            navigation,
            text,
            zones,
            named,
            keys,
            blocks,
            walls,
            phases,
            read_each,
            scenes,
            said,
        } = options;
        // The layout readers follow pointers down each layout's tree to its words, and read only
        // inside the blocks the last walk found, as the field's do (ADR-0035).
        let panes: Rc<dyn Memory> = match &blocks {
            None => memory.clone(),
            Some(blocks) => Rc::new(InBlocks::new(memory.clone(), blocks.clone())),
        };
        let layouts = Rc::new(Layouts::new(panes.clone(), names.clone(), lyt::SHAPE));
        let dialogue = DialogueReader::with_shapes(
            memory.clone(),
            names.clone(),
            Some(layouts.clone()),
            msgwin::SHAPES,
        );
        // Menus and screens are ORAS's readers over Sun's panes: the words each TextBox on screen
        // shows, and the item under each cursor (spike 112).
        let screen = ScreenTextReader::with(
            panes.clone(),
            names.clone(),
            layouts.clone(),
            msgwin::CLASSES.iter().map(|c| c.to_string()).collect(),
            text.as_ref()
                .and_then(|text| text.line(lyt::LEVEL_WORDS.0, lyt::LEVEL_WORDS.1)),
            None,
            IndexMap::new(),
            HashSet::new(),
        );
        let cursors = CursorReader::with(layouts.clone(), names.clone(), None, true, None);
        let ui = ui.unwrap_or_default();
        let screen_announcer = ScreenAnnouncer::with(speech.clone(), log.clone(), ui.pauses);
        let speakers = Rc::new(Speakers::new(memory.clone(), names.clone()));
        let announcements = Rc::new(RefCell::new(Announcements::with(
            speech.clone(),
            log.clone(),
            Some({
                let speakers = speakers.clone();
                Rc::new(move |window| speakers.name_for(window))
            }),
            true,
        )));
        // The scenes the game shows without words, described where the player asks and Sun has
        // descriptions in their language, each over the demo it plays as (ADR-0038).
        let scenes = (ui.descriptions && !scenes.is_empty()).then(|| {
            RefCell::new(Scenes::with_monotonic(
                scenes,
                said.unwrap_or_else(|| Rc::new(|_message: &str| None)),
            ))
        });
        let langselect = LangSelectReader::new(memory.clone(), names.clone(), text.clone());
        // The HP box is the battle reader's to say, from the gauges behind its panels, and a
        // gauge's pointers are followed only inside the blocks the last walk found (spike 124).
        let battle = SunBattleReader::new(panes, names.clone(), layouts.clone());
        let running = RunningApp::with(memory.clone(), names.clone(), blocks.clone());
        // The field: the player's zone and cell and what is around them (spike 113), walked by
        // ORAS's navigator, and P's way found through the walls out of the ROM (spike 114).
        // Without them, P says where the thing is and that no way is known.
        let routes = walls.is_some();
        let reader = Rc::new(SunFieldReader::with(
            memory.clone(),
            names.clone(),
            zones.clone(),
            blocks,
            walls,
            read_each,
        ));
        let navigator = Navigator::new(
            reader.clone(),
            speech.clone(),
            log.clone(),
            navigate::Options {
                bindings: keys,
                names: named,
                navigation,
                leads_to: Some({
                    let zones = zones.clone();
                    Rc::new(move |here, there| leads_to(zones.as_deref(), here, there))
                }),
                routes,
                slack: field::SLACK,
                sides: Some({
                    let announcements = announcements.clone();
                    Rc::new(move |player| Ok(announcements.borrow().sides(player)))
                }),
                ..navigate::Options::default()
            },
        );
        SunProfile {
            memory,
            phase: phases,
            names,
            speech,
            log,
            layouts,
            dialogue: RefCell::new(dialogue),
            screen: RefCell::new(screen),
            cursors: RefCell::new(cursors),
            screen_announcer: RefCell::new(screen_announcer),
            speakers,
            announcements,
            langselect: RefCell::new(langselect),
            battle: RefCell::new(battle),
            apps: RefCell::new(running),
            scenes,
            windows: RefCell::new(Vec::new()),
            utils: RefCell::new(Vec::new()),
            zones,
            field: reader,
            navigator: RefCell::new(navigator),
            place: RefCell::new(None),
            zone: Cell::new(None),
            listed: RefCell::new(None),
            relist: Cell::new(false),
        }
    }

    /// The announcements' `narrate`, which `Scenes` says each beat through.
    fn narrate(&self) -> impl FnMut(&str) + '_ {
        move |text: &str| self.announcements.borrow_mut().narrate(text)
    }

    /// `with self._phase(name): body`.
    fn phase<R>(&self, name: &str, body: impl FnOnce() -> Result<R>) -> Result<R> {
        match &self.phase {
            Some(cache) => memory::phase(&**cache, name, body),
            None => body(),
        }
    }

    fn _poll_screen(&self) -> Result<()> {
        let revived = self.layouts.revive()?;
        if self.layouts.poll_pools()? || revived {
            let windows = self.layouts.panes(common_lyt::WINDOW_CLASS);
            self.cursors.borrow_mut().track(windows)?;
        }
        let line_objects: HashSet<i64> = self.dialogue.borrow().line_objects().iter().collect();
        let screen = self.screen.borrow_mut().poll(&line_objects, true)?;
        // A message box's lines lead to a window typing into them before any sweep finds it.
        let lines: Vec<i64> = screen.message_lines.iter().copied().collect();
        self.dialogue.borrow_mut().watch(&lines);
        // The HP box is the battle reader's to say, whole and once it holds still.
        let sides = self.battle.borrow_mut().poll(&screen.on_screen)?;
        let cursors = self.cursors.borrow_mut().poll(&screen.on_screen)?;
        // The language choice's buttons are pictures, which only its own reader can name; the
        // other readers keep up with its screen but leave it to that reader.
        if !self.langselect.borrow().showing() {
            let texts: Vec<ScreenText> = screen
                .texts
                .iter()
                .filter(|text| !self.battle.borrow().owns(text))
                .cloned()
                .collect();
            let texts = self._without_plates(texts)?;
            // A layout built again where it was is the one that was there, and a menu stays a
            // menu, as in ORAS.
            let mut roots = self.layouts.roots();
            roots.extend(self.layouts.buried());
            self.screen_announcer.borrow_mut().say(
                &ScreenPoll { texts, ..screen },
                &cursors,
                &HashSet::new(),
                Some(&roots),
                None,
            )?;
        }
        self.announcements.borrow_mut().battlers(&sides);
        // Text written over a message window is held for a screen poll before it is said.
        self.announcements.borrow_mut().screen_polled();
        Ok(())
    }

    /// The pieces, but for the name on the plate over a message window, which is said with
    /// the window's page (`talk`).
    fn _without_plates(&self, texts: Vec<ScreenText>) -> Result<Vec<ScreenText>> {
        let windows = self.windows.borrow().clone();
        let mut plates: HashSet<String> = HashSet::new();
        for window in windows {
            plates.extend(self.speakers.name_for(window)?);
        }
        if plates.is_empty() {
            return Ok(texts);
        }
        let mut roots: HashSet<i64> = HashSet::new();
        let lines = self.dialogue.borrow().line_objects();
        for line in lines.iter() {
            if let Some(placement) = self.layouts.placement(line)? {
                roots.insert(placement.root);
            }
        }
        Ok(texts
            .into_iter()
            .filter(|text| !roots.contains(&text.root) || !plates.contains(&text.text))
            .collect())
    }

    /// Say the place when the player arrives in another: the house and the route outside are
    /// both "Route 1", so walking out of the door arrives nowhere new, as the game has it. By
    /// its name, not its line: Route 1's two zones have lines of their own, both "Route 1", and
    /// "Route 1" was said at every step over the line between them (sun_7.ctm, 2026-09-26).
    fn _arrive(&self, zone: i64) -> Result<()> {
        let Some(zones) = &self.zones else {
            return Ok(());
        };
        let Some(name) = zones.place(zone)? else {
            return Ok(());
        };
        if self.place.borrow().as_deref() == Some(name.as_str()) {
            return Ok(());
        }
        *self.place.borrow_mut() = Some(name.clone());
        self.screen_announcer.borrow_mut().place(&name);
        Ok(())
    }
}

/// What the game calls the place an exit from `here` into `there` arrives in, or None when
/// it is the place the player is in already.
fn leads_to(zones: Option<&SunZones>, here: i64, there: i64) -> Result<Option<String>> {
    let Some(zones) = zones else {
        return Ok(None);
    };
    // By name: from Route 1's second zone the house's door was "door to Route 1", its
    // zone's place line not the second zone's, though both are "Route 1" (sun_7.ctm).
    let (ours, theirs) = (zones.place(here)?, zones.place(there)?);
    if ours.is_none() || theirs.is_none() || ours == theirs {
        return Ok(None);
    }
    Ok(theirs)
}

impl Profile for SunProfile {
    /// Take in what a sweep found.
    ///
    /// A sweep of the blocks the game has just allocated is partial: it says what is new and
    /// nothing of what is gone, so readers add what it found to what they know, and the layouts
    /// are looked over only when it brings new ones, as in ORAS (ADR-0034).
    fn track(&self, found: &Found) -> Result<()> {
        let adding = found.partial;
        let windows = found.of(&msgwin::CLASSES);
        let utils = found.of(&[talk::UTIL_CLASS]);
        {
            let mut held = self.windows.borrow_mut();
            *held = if adding {
                let fresh: Vec<i64> = windows
                    .iter()
                    .copied()
                    .filter(|window| !held.contains(window))
                    .collect();
                [held.clone(), fresh].concat()
            } else {
                windows.clone()
            };
        }
        {
            let mut held = self.utils.borrow_mut();
            *held = if adding {
                let fresh: Vec<i64> = utils
                    .iter()
                    .copied()
                    .filter(|util| !held.contains(util))
                    .collect();
                [held.clone(), fresh].concat()
            } else {
                utils
            };
        }
        self.dialogue.borrow_mut().track(&windows, adding);
        self.speakers.track(found.of(&[talk::CLASS]), adding);
        self.langselect
            .borrow_mut()
            .track(found.of(&[langselect::CLASS]), adding);
        self.battle
            .borrow_mut()
            .track(found.of(&[battle::GAUGE_CLASS]), adding)?;
        self.apps
            .borrow_mut()
            .track(found.of(&[apps::MANAGER_CLASS]))?;
        // A layout's work has no class of its own; the DrawInfo embedded in it does.
        let works: Vec<i64> = found
            .of(&[common_lyt::DRAWINFO_CLASS])
            .into_iter()
            .map(|at| at - common_lyt::WORK_DRAWINFO)
            .collect();
        if adding {
            // A screen's pool is made before its layouts are built into it.
            self.layouts.watch_pools_in(&found.pages)?;
        }
        if !works.is_empty() || !adding {
            self.layouts.track(works)?;
            self.screen.borrow_mut().track(Vec::<i64>::new());
            let windows = self.layouts.panes(common_lyt::WINDOW_CLASS);
            self.cursors.borrow_mut().track(windows)?;
        }
        self.field.track(found);
        // Listing the field reads every model and actor: after a whole sweep, or when the blocks
        // just allocated held the field's own.
        if !adding || !found.of(&field::CLASSES).is_empty() {
            self.relist.set(true);
        }
        Ok(())
    }

    /// Whether another app has started since the last poll: the moment to sweep for its
    /// objects and learn the classes its module brings. One read a poll, and one more as a demo
    /// starts, for its number: the moment its scene's description starts.
    fn poll_app(&self) -> Result<bool> {
        let (started, late, demo) = self.phase("app", || {
            let started = self.apps.borrow_mut().poll()?;
            let late = if started.is_none() {
                self.apps.borrow_mut().late_name()?
            } else {
                None
            };
            let named = match &started {
                None => late.as_deref(),
                Some((_, name)) => name.as_deref(),
            };
            let demo = if named == Some(apps::DEMO) {
                self.apps.borrow().demo()?
            } else {
                None
            };
            Ok((started, late, demo))
        })?;
        if let Some(late) = late {
            (self.log)(&format!("app: the one started is {}", demangle(&late)));
        }
        if let Some(demo) = demo {
            (self.log)(&format!("app: demo {demo}"));
            if let Some(scenes) = &self.scenes {
                scenes.borrow_mut().demo(Some(demo), &mut self.narrate())?;
            }
        }
        let Some((pointer, name)) = started else {
            return Ok(false);
        };
        let what = match name {
            None => format!("the manager pointing at {pointer:#010x}"),
            Some(name) => demangle(&name),
        };
        (self.log)(&format!("app: another started, {what}"));
        // A battle's sides are said again on a key only while it lasts.
        self.announcements.borrow_mut().forget_sides();
        Ok(true)
    }

    /// Every app Sun starts is swept for as it builds: what ORAS's field coming back leaves
    /// alone was measured in ORAS alone (spike 134).
    fn app_sweep_wanted(&self) -> bool {
        true
    }

    /// Poll dialogue. Returns whether a page was said.
    fn poll_dialogue(&self) -> Result<bool> {
        self.phase("dialogue", || {
            let utils = self.utils.borrow().clone();
            let held = talk::windows(&*self.memory, &self.names, utils, &msgwin::CLASSES)?;
            let new: Vec<i64> = {
                let windows = self.windows.borrow();
                held.into_iter()
                    .filter(|window| !windows.contains(window))
                    .collect()
            };
            if !new.is_empty() {
                let windows = {
                    let mut windows = self.windows.borrow_mut();
                    windows.extend(new);
                    windows.clone()
                };
                self.dialogue.borrow_mut().track(&windows, false);
            }
            let shown = self.dialogue.borrow_mut().poll()?;
            self.announcements.borrow_mut().pages(&shown)
        })
    }

    /// Poll the language choice. Returns whether it said anything.
    fn poll_menus(&self, _discovery: &mut Discovery) -> Result<bool> {
        let shown = self.phase("menus", || self.langselect.borrow_mut().poll())?;
        let Some(shown) = shown else {
            return Ok(false);
        };
        if let Some(title) = &shown.title {
            (self.log)(&format!("screen shown: {title}"));
            self.speech.say(title, Category::Screen);
        }
        self.announcements
            .borrow_mut()
            .focus(&shown.focus, "menu", false);
        if let Some(help) = &shown.help {
            (self.log)(&format!("screen shown: {help}"));
            self.speech.say(help, Category::Screen);
        }
        Ok(true)
    }

    /// Say what the screens show outside message windows: menus, their cursors, and the
    /// words a screen sets out, as ORAS's are said.
    fn poll_screen(&self) -> Result<()> {
        // A screen opened from the X menu frees the field's layouts and builds them again where
        // they were as it closes, and a layout found before it was built is taken in once it is,
        // its cursor with it: between the heaps' whole walks the save prompt's Yes and No, and the
        // X menu's, were read out whole with no cursor over them (sun_3.ctm, spike 123).
        self.phase("screen", || self._poll_screen())
    }

    /// Poll where the player is, say the place when it changes, and log what is around them
    /// when that changes. The listing reads every model and actor, so it is made only on a
    /// change of zone and after each sweep, not every poll.
    fn poll_field(&self) -> Result<()> {
        if let Some(scenes) = &self.scenes {
            scenes.borrow_mut().poll(&mut self.narrate())?;
        }
        let listing = self.phase("field", || {
            let Some(at) = self.field.poll()? else {
                return Ok(None);
            };
            self._arrive(at.zone)?;
            // The chase after Kukui's call ends on the field, which its last beats wait for.
            if self.zone.get() != Some(at.zone) {
                self.zone.set(Some(at.zone));
                if let Some(scenes) = &self.scenes {
                    scenes.borrow_mut().arrived(at.zone, &mut self.narrate())?;
                }
            }
            let same_zone = self
                .listed
                .borrow()
                .as_ref()
                .is_some_and(|listed| listed.0 == at.zone);
            if !self.relist.get() && same_zone {
                return Ok(None);
            }
            self.relist.set(false);
            Ok(Some(self.field.interactables(&at)?))
        })?;
        let Some(at) = listing else {
            return Ok(());
        };
        let mut items: Vec<(String, String, Option<(i64, i64)>)> = at
            .interactables
            .iter()
            .map(|item| {
                (
                    item.category.clone(),
                    item.kind.clone(),
                    if item.category == "people" {
                        None
                    } else {
                        Some(item.cell)
                    },
                )
            })
            .collect();
        // Strings and cells compare as Python's tuples do, and only a person's third is None, so
        // two keys that tie on the first two never put a None against a cell.
        items.sort();
        let listed = (at.zone, items);
        if self.listed.borrow().as_ref() == Some(&listed) {
            return Ok(());
        }
        *self.listed.borrow_mut() = Some(listed);
        (self.log)(&format!(
            "field: zone {:#x} at ({}, {})",
            at.zone, at.cell.0, at.cell.1
        ));
        let zone = at.zone;
        let called = |item: &Interactable| -> Result<Option<String>> {
            self.navigator.borrow().called(item, zone)
        };
        let destination = |item: &Interactable| self.navigator.borrow().destination(item, zone);
        for line in describe(&at, Some(&called), Some(&destination)) {
            (self.log)(&format!("field {}", line?));
        }
        Ok(())
    }

    /// Follow the player along a path they are walking, every poll while there is one, as in
    /// ORAS: a turn said at the field's fifth of a second is a step the player has taken past
    /// it. Nothing is read while there is none.
    fn poll_guidance(&self) -> Result<()> {
        if !self.navigator.borrow().guiding() {
            return Ok(());
        }
        let at = self.phase("guidance", || self.field.poll())?;
        if let Some(at) = at {
            self.navigator.borrow_mut().follow(&at)?;
        }
        Ok(())
    }

    /// Sample where each named cursor stands, every tick while there is one, as in ORAS. Sun's
    /// Nothing: Sun's footsteps are not measured (`gen6.field.footsteps`).
    fn poll_footsteps(&self) -> Result<()> {
        Ok(())
    }

    /// cursor reader has no layout files, so it knows no named cursor and nothing is read.
    fn poll_pointers(&self) -> Result<()> {
        if !self.cursors.borrow().sampling() {
            return Ok(());
        }
        self.phase("pointers", || {
            self.cursors.borrow_mut().sample(&*self.memory)
        })
    }

    /// Keep any name the player has just given, from a dialog on a thread of its own.
    fn poll_renames(&self) -> Result<()> {
        self.navigator.borrow_mut().poll_renames()
    }

    /// Whether a reader needs something only a sweep finds: the gauge behind an HP box on
    /// screen, as in ORAS.
    fn sweep_wanted(&self) -> Result<bool> {
        Ok(self.battle.borrow_mut().sweep_wanted())
    }

    /// Answer a navigation key, by name. In a phase of its own, so the field's burst of heads
    /// answers the listing's reads (`SunFieldReader._fetch`).
    fn press(&self, key: &str) -> Result<()> {
        self.phase("keys", || self.navigator.borrow_mut().press_key(key))
    }

    /// No screen of Sun and Moon has keys of its own yet.
    fn context(&self) -> Option<&'static str> {
        None
    }
}
