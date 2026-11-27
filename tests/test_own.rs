//! A game's own (`gen6.own.Own`) is asked where gen 6 asks it, in gen 6's order around it: each
//! fake here overrides a slot or two of a game that has none, and the profile says the rest as
//! before.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::fake_speech::Recorder;
use super::test_field::Town;
use super::test_profile::{no_log, profile_of};
use crate::core::config::Tables;
use crate::core::memory::{Cache, Memory};
use crate::core::rtti::ClassNames;
use crate::core::speech::{Category, Speech};
use crate::games::common::field::keys::OUR_SIDE;
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::{BATTLE, FIELD, RunningApp};
use crate::games::gen6::game::Game;
use crate::games::gen6::own::{Kit, Own};
use crate::games::gen6::profile::HEAP_CLASSES;
use crate::games::gen6::rom::archives::{GameText, no_text};
use crate::games::gen6::screens::sources::OwnSources;
use crate::games::gen6::screens::townmap::OwnLists;
use crate::games::xy;
use crate::games::xy::intro;
use crate::games::xy::own::Xy;
use crate::py::Result;

/// A fake memory read in phases, each entered kept: Python's `phase` context managers, the one
/// `kit` gives entering nothing that is looked at.
pub(crate) struct Phases {
    memory: Rc<FakeMemory>,
    pub(crate) entered: RefCell<Vec<String>>,
}

impl Memory for Phases {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        self.memory.read(address, size)
    }
}

impl Cache for Phases {
    fn begin(&self, name: &str) -> Result<()> {
        self.entered.borrow_mut().push(name.to_string());
        Ok(())
    }

    fn end(&self, _name: &str) {}

    fn read_each(&self, _reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        panic!("a game's own reads no batch")
    }
}

/// What the profile would build a game's own from, over fake memory, `running` the app.
pub(crate) fn kit(memory: Rc<FakeMemory>, running: Option<&str>, process: Option<i64>) -> Kit {
    let layouts = FakeLayouts::new(
        &memory,
        &FakeClasses::starting(memory.clone(), CODE.0 + 0x8000),
    )
    .layouts
    .clone();
    let names = Rc::new(ClassNames::new(memory.clone(), CODE));
    let apps = Rc::new(RefCell::new(RunningApp::new(memory.clone(), names.clone())));
    apps.borrow_mut().running = running.map(String::from);
    apps.borrow_mut().process = process;
    Kit {
        memory: memory.clone(),
        names,
        layouts,
        files: Rc::new(LayoutFiles::new(memory.clone())),
        apps,
        text: Rc::new(GameText::new(Vec::new(), Vec::new())),
        speech: Rc::new(Speech::plain(Recorder::new())),
        log: Rc::new(|_: &str| {}),
        tables: Tables::default(),
        phase: Rc::new(Phases {
            memory,
            entered: RefCell::new(Vec::new()),
        }),
    }
}

/// `class Swept(Own)`: a class of its own.
struct Swept;

impl OwnLists for Swept {}
impl OwnSources for Swept {}
impl Own for Swept {
    fn classes(&self) -> Vec<String> {
        vec!["N4mine5FirstE".to_string()]
    }
}

fn swept(_kit: &Kit) -> Rc<dyn Own> {
    Rc::new(Swept)
}

/// ADR-0006: what a game has alone is swept for before `HEAP_CLASSES`, as ORAS's subscreen's came
/// before its touch bars inside it: the log names the classes a module brings in in the order
/// asked.
#[test]
fn test_a_games_own_classes_are_swept_for_first() {
    let game = Game {
        own: swept,
        ..xy::game().clone()
    };
    let (made, _) = profile_of(&Town::new(), no_text(), None, no_log(), &game);
    let heap: IndexSet<&str> = HEAP_CLASSES.into_iter().collect();
    let mut first = vec!["N4mine5FirstE".to_string()];
    first.extend(heap.iter().map(|class| class.to_string()));
    assert_eq!(made.classes[..first.len()], first[..]);
}

/// `class Standing(Own)`: says what a test has it say.
struct Standing {
    said: RefCell<Option<String>>,
}

impl OwnLists for Standing {}
impl OwnSources for Standing {}
impl Own for Standing {
    fn sides(&self, _player: bool) -> Result<Option<String>> {
        Ok(self.said.borrow().clone())
    }
}

thread_local! {
    /// The `Standing` the profile made, which a test reaches as Python's does through
    /// `made.own`.
    static STANDING: RefCell<Option<Rc<Standing>>> = const { RefCell::new(None) };
}

fn standing(_kit: &Kit) -> Rc<dyn Own> {
    let made = Rc::new(Standing {
        said: RefCell::new(None),
    });
    STANDING.with(|kept| *kept.borrow_mut() = Some(made.clone()));
    made
}

/// A contest's standing is said on R first; where the game's own says nothing, R is the
/// battle's, here a battle whose sides are not drawn yet.
#[test]
fn test_r_says_a_games_own_standing_before_a_battles_sides() {
    let game = Game {
        own: standing,
        ..xy::game().clone()
    };
    let (made, backend) = profile_of(&Town::new(), no_text(), None, no_log(), &game);
    made.apps.borrow_mut().running = Some(BATTLE.to_string());
    made.navigator.borrow_mut().press(OUR_SIDE).unwrap();
    let own = STANDING.with(|kept| kept.borrow().clone()).expect("made");
    *own.said.borrow_mut() = Some("Brendan & Pikachu first place".to_string());
    made.navigator.borrow_mut().press(OUR_SIDE).unwrap();
    assert_eq!(
        backend.spoken(),
        ["nothing shown yet", "Brendan & Pikachu first place"]
    );
}

/// `class Lines(Own)`: a line after the screen's words, and one said last.
struct Lines {
    speech: Rc<Speech>,
}

impl OwnLists for Lines {}
impl OwnSources for Lines {}
impl Own for Lines {
    fn after(&self, _on_screen: &[ScreenText]) -> Result<Vec<(String, String)>> {
        Ok(vec![(
            "mine".to_string(),
            "said after the screen".to_string(),
        )])
    }

    fn last(
        &self,
        _under: &[String],
        _on_screen: &[ScreenText],
        _settling: &IndexSet<i64>,
        _waiting: bool,
    ) -> Result<()> {
        self.speech.say("said last", Category::Screen);
        Ok(())
    }
}

fn lines(kit: &Kit) -> Rc<dyn Own> {
    Rc::new(Lines {
        speech: kit.speech.clone(),
    })
}

/// ADR-0006: the lines after the screen's words are the cursor sources', then the game's
/// own (ORAS's Pokémon Distribution), then the subscreens' descriptions, then what the game's
/// own says last (ORAS's contest conditions).
///
/// The Python patches the subscreens' `describe` in place to say a description between the two;
/// the port has no such patch, and checks the game's own lines' order alone.
#[test]
fn test_a_games_own_lines_come_after_the_screens_words_around_the_descriptions() {
    let game = Game {
        own: lines,
        ..xy::game().clone()
    };
    let (made, backend) = profile_of(&Town::new(), no_text(), None, no_log(), &game);
    made.poll_screen().unwrap();
    assert_eq!(backend.spoken(), ["said after the screen", "said last"]);
}

/// A phase begins by fetching the pages it read last time: entered every tick, X's opening
/// would read the intro's pages once more after the intro ended.
#[test]
fn test_xs_opening_reads_in_a_phase_of_its_own_only_while_it_runs() {
    let process = 0x08C60000;
    let memory = FakeMemory::new();
    let phases = Rc::new(Phases {
        memory: memory.clone(),
        entered: RefCell::new(Vec::new()),
    });
    for running in [FIELD, intro::APP] {
        let kit = Kit {
            phase: phases.clone(),
            ..kit(memory.clone(), Some(running), Some(process))
        };
        assert!(Xy::new(&kit).focus().unwrap().is_none());
    }
    assert_eq!(*phases.entered.borrow(), ["opening"]);
}
