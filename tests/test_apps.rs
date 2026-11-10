//! Running app tests, against a process manager and processes laid out as spike 99 read them.

use std::cell::Cell;
use std::rc::Rc;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::gen6::apps::{
    ASKS, EVERY, MANAGER_CLASS, MANAGER_RUNNING, PROCESS_MANAGER, RunningApp,
};

const MANAGER: i64 = 0x08C6A180;
/// Where every process spike 99 saw was built.
const SLOT: i64 = 0x08C65224;
const BAG: &str = "N3app3bag4ProcE";
const FIELD: &str = "N5field12FieldmapProcE";

struct Game {
    memory: Rc<FakeMemory>,
    classes: FakeClasses,
    app: RunningApp,
}

impl Game {
    fn new() -> Game {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        memory.u32(MANAGER, &[classes.add(MANAGER_CLASS)]);
        let mut app = RunningApp::new(
            memory.clone(),
            Rc::new(ClassNames::new(memory.clone(), CODE)),
        );
        app.track([MANAGER]).unwrap();
        Game {
            memory,
            classes,
            app,
        }
    }

    /// Build a process of this class at `SLOT` and point the manager at its state word.
    fn run(&self, mangled: &str, state_at: i64) {
        self.run_at(mangled, state_at, SLOT);
    }

    /// Build a process of this class at `at` and point the manager at its state word.
    fn run_at(&self, mangled: &str, state_at: i64, at: i64) {
        self.memory.u32(at, &[self.classes.add(mangled)]);
        self.memory.u32(at + PROCESS_MANAGER, &[MANAGER]);
        self.memory.u32(MANAGER + MANAGER_RUNNING, &[at + state_at]);
    }
}

#[test]
fn test_the_app_running_when_first_polled_is_news() {
    let mut game = Game::new();
    game.run(FIELD, 0x64);
    assert_eq!(game.app.poll().unwrap().as_deref(), Some(FIELD));
    assert_eq!(game.app.running.as_deref(), Some(FIELD));
}

#[test]
fn test_where_the_running_process_begins_is_kept() {
    let mut game = Game::new();
    game.run(FIELD, 0x64);
    game.app.poll().unwrap();
    assert_eq!(game.app.process, Some(SLOT));
}

#[test]
fn test_an_app_still_running_is_not_news_again() {
    let mut game = Game::new();
    game.run(FIELD, 0x64);
    game.app.poll().unwrap();
    assert_eq!(game.app.poll().unwrap(), None);
}

/// Spike 99 saw the field, the Bag, the Pokédex and a battle all built at one address.
#[test]
fn test_another_app_built_where_the_last_was_is_news() {
    let mut game = Game::new();
    game.run(FIELD, 0x64);
    game.app.poll().unwrap();
    game.run(BAG, 0x58);
    assert_eq!(game.app.poll().unwrap().as_deref(), Some(BAG));
}

/// Super Training's regimen menu and the regimen it starts: one address, one pointer
/// (omega_ruby_bottom-screen5).
#[test]
fn test_another_app_built_where_the_last_was_with_the_manager_at_the_same_word_is_news() {
    let mut game = Game::new();
    game.run(FIELD, 0x64);
    game.app.poll().unwrap();
    game.run(BAG, 0x64);
    assert_eq!(game.app.poll().unwrap().as_deref(), Some(BAG));
    assert_eq!(game.app.poll().unwrap(), None);
}

/// The field coming back after a battle builds its map blocks again where they were, and its
/// whole sweeps found nothing a reader asks for by class (spike 134).
#[test]
fn test_an_app_that_has_run_before_is_coming_back() {
    let mut game = Game::new();
    game.run(FIELD, 0x64);
    game.app.poll().unwrap();
    assert!(!game.app.back);
    game.run(BAG, 0x58);
    game.app.poll().unwrap();
    assert!(!game.app.back);
    game.run(FIELD, 0x64);
    game.app.poll().unwrap();
    assert!(game.app.back);
}

/// Between two apps the manager can point where no process has been built yet.
#[test]
fn test_a_pointer_into_no_process_is_waited_out() {
    let mut game = Game::new();
    game.run(FIELD, 0x64);
    game.app.poll().unwrap();
    game.memory.u32(MANAGER + MANAGER_RUNNING, &[0x08D00000]);
    assert_eq!(game.app.poll().unwrap(), None);
    assert_eq!(game.app.running.as_deref(), Some(FIELD));
    game.run_at(BAG, 0x58, 0x08D00000 - 0x58);
    assert_eq!(game.app.poll().unwrap().as_deref(), Some(BAG));
}

#[test]
fn test_a_process_that_does_not_point_back_at_the_manager_is_not_one() {
    let mut game = Game::new();
    game.run(BAG, 0x58);
    game.memory.u32(SLOT + PROCESS_MANAGER, &[0x08000000]);
    assert_eq!(game.app.poll().unwrap(), None);
}

#[test]
fn test_nothing_is_news_before_the_manager_is_found() {
    let game = Game::new();
    game.run(FIELD, 0x64);
    let mut app = RunningApp::new(
        game.memory.clone(),
        Rc::new(ClassNames::new(game.memory.clone(), CODE)),
    );
    assert_eq!(app.poll().unwrap(), None);
    app.track([0x08000000, MANAGER]).unwrap();
    assert_eq!(app.poll().unwrap().as_deref(), Some(FIELD));
}

/// Only a sweep of the whole heap finds the manager, and the first can come before the game
/// has built it: every key said "no map" until the scheduled sweep 30 s later (a session
/// of 2026-09-27 23:48).
#[test]
fn test_a_sweep_is_asked_for_every_few_seconds_until_the_manager_is_found() {
    let game = Game::new();
    let now = Rc::new(Cell::new(0.0));
    let clock = now.clone();
    let mut app = RunningApp::with(
        game.memory.clone(),
        Rc::new(ClassNames::new(game.memory.clone(), CODE)),
        Rc::new(move || clock.get()),
    );
    assert!(app.sweep_wanted());
    assert!(!app.sweep_wanted());
    now.set(now.get() + EVERY);
    assert!(app.sweep_wanted());
    app.track([MANAGER]).unwrap();
    now.set(now.get() + EVERY);
    assert!(!app.sweep_wanted());
}

#[test]
fn test_a_manager_no_sweep_finds_is_left_to_the_schedule() {
    let game = Game::new();
    let now = Rc::new(Cell::new(0.0));
    let clock = now.clone();
    let mut app = RunningApp::with(
        game.memory.clone(),
        Rc::new(ClassNames::new(game.memory.clone(), CODE)),
        Rc::new(move || clock.get()),
    );
    for _ in 0..ASKS {
        assert!(app.sweep_wanted());
        now.set(now.get() + EVERY);
    }
    assert!(!app.sweep_wanted());
}

/// Every app's class is in the code module the game loads for it, past where the executable
/// ends, and the class names the runtime reads from the executable do not reach it: the first
/// runtime to read the manager saw no app start at all (2026-09-25).
#[test]
fn test_an_app_whose_class_is_in_a_module_loaded_later_is_still_named() {
    let mut game = Game::new();
    let module = FakeClasses::starting(game.memory.clone(), 0x007F0000);
    game.memory.u32(SLOT, &[module.add(FIELD)]);
    game.memory.u32(SLOT + PROCESS_MANAGER, &[MANAGER]);
    game.memory.u32(MANAGER + MANAGER_RUNNING, &[SLOT + 0x64]);
    assert_eq!(game.app.poll().unwrap().as_deref(), Some(FIELD));
}

#[test]
fn test_a_manager_that_is_gone_is_forgotten() {
    let mut game = Game::new();
    game.run(FIELD, 0x64);
    game.app.poll().unwrap();
    game.memory.u32(MANAGER, &[0]);
    game.run(BAG, 0x58);
    assert_eq!(game.app.poll().unwrap(), None);
}
