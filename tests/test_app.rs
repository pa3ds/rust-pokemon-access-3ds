//! Runtime tests: waiting for a game to start, and what a reader raising does to the loop.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};

use indexmap::IndexMap;

use super::fake_emulator::{FakeEmulator, NO_PROCESS};
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::fake_rom::{TmpDir, rom_image, rom_image_with};
use super::fake_speech::Recorder;
use super::test_archives::oras_files;
use crate::__main__::{Hooks, KEPT_LOGS, default_log_path, log_path, main, main_with};
use crate::app::{
    Busy, DATA_IN_TEST, Every, Failure, Faults, MISSING_DATA, PATIENCE, Settings, run, run_with,
    wait_for_game,
};
use crate::core::config::Ui;
use crate::core::memory::{self, Memory};
use crate::core::rpc::{MAX_READ, Process, Rpc, UdpRpc};
use crate::core::speech::{Backend, Category, Speech};
use crate::core::{config, keyfile, words};
use crate::games::catalog::{self, Running};
use crate::games::common::announce::Announcements;
use crate::games::common::dialogue::{Shown, TextWritten};
use crate::games::common::msgwin;
use crate::games::common::session::{Profile, Session, Setup};
use crate::games::gen6::mystatus;
use crate::games::gen6::rom::archives::no_text;
use crate::games::gen6::session::game_text;
use crate::games::{oras, sm};
use crate::py::{Error, Kind, Log, Result};

/// The environment is the process's, and the tests run on threads of one process: a test that
/// moves the player's folder holds this until it is done.
pub(super) static ENVIRONMENT: Mutex<()> = Mutex::new(());

/// Every test that reaches the player's folder gets a home of its own. `main` writes the log, the
/// keys file and the player's names there, and a test that reached the real one would rotate the
/// live logs and rewrite the keys somebody is playing with.
struct Home {
    _lock: MutexGuard<'static, ()>,
    root: TmpDir,
    home: PathBuf,
}

impl Home {
    fn new() -> Home {
        let lock = ENVIRONMENT
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let root = TmpDir::new();
        let home = root.path().join("user");
        // SAFETY: every test that touches the environment holds `ENVIRONMENT`.
        unsafe {
            std::env::set_var(keyfile::HOME_ENV, &home);
            // Azahar's own config is looked for under APPDATA, and is not pa3ds's to find in a
            // test.
            std::env::set_var("APPDATA", root.path().join("appdata"));
        }
        Home {
            _lock: lock,
            root,
            home,
        }
    }

    fn tmp(&self) -> &Path {
        self.root.path()
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        // SAFETY: as in `new`.
        unsafe { std::env::remove_var(keyfile::HOME_ENV) };
    }
}

const GAME: (u32, u64, &str) = (11, 0x000400000011C400, "sango-1");

fn game() -> Vec<(u32, u64, String)> {
    vec![(GAME.0, GAME.1, GAME.2.to_string())]
}

fn args(argv: &[&str]) -> Vec<String> {
    argv.iter().map(|arg| arg.to_string()).collect()
}

fn logger() -> (Log, Rc<RefCell<Vec<String>>>) {
    let logged = Rc::new(RefCell::new(Vec::new()));
    let into = logged.clone();
    (
        Rc::new(move |line: &str| into.borrow_mut().push(line.to_string())),
        logged,
    )
}

fn quiet() -> Log {
    Rc::new(|_line: &str| {})
}

fn connect(emu: &FakeEmulator) -> Rc<dyn Rpc> {
    Rc::new(UdpRpc::connect(emu.port()).unwrap())
}

/// The player's `config.toml`, which is where a setting lives now there is no flag for it,
/// with Azahar's settings commented out as a player keeping Azahar's own has them: a table
/// missing would be filled in, and there is no Azahar here to apply it to.
fn settings_file(home: &Path, text: &str) {
    std::fs::create_dir_all(home).unwrap();
    let azahar: String = config::azahar()
        .keys()
        .map(|name| format!("# {name} = 0\n"))
        .collect();
    std::fs::write(
        home.join("config.toml"),
        format!("{text}\n[azahar]\n{azahar}"),
    )
    .unwrap();
}

#[test]
fn test_attaches_at_once_when_the_game_is_already_running() {
    let backend = Recorder::new();
    let emu = FakeEmulator::with(HashMap::new(), game(), 0, 0);
    let process = wait_for_game(
        &*connect(&emu),
        &Speech::plain(backend.clone()),
        &|| false,
        &quiet(),
        &Settings::default(),
    )
    .unwrap();
    assert_eq!(
        process.map(|process| process.name),
        Some("sango-1".to_string())
    );
    assert!(backend.spoken().is_empty());
}

#[test]
fn test_waits_for_the_game_and_says_so_once() {
    let backend = Recorder::new();
    let emu = FakeEmulator::with(HashMap::new(), Vec::new(), 0, 0);
    let attempts = Cell::new(0);
    let until = || {
        attempts.set(attempts.get() + 1);
        if attempts.get() == 4 {
            emu.set_processes(game());
        }
        attempts.get() > 20
    };
    let process = wait_for_game(
        &*connect(&emu),
        &Speech::plain(backend.clone()),
        &until,
        &quiet(),
        &Settings {
            wait: 0.0,
            ..Settings::default()
        },
    )
    .unwrap();
    assert_eq!(
        process.map(|process| process.name),
        Some("sango-1".to_string())
    );
    assert_eq!(backend.spoken(), ["Waiting for the game"]);
}

/// Moving through two options inside one screen poll said the first one's description after
/// the second option, and never the second's (menus.ctm replay).
#[test]
fn test_written_text_waits_a_whole_poll_and_only_the_last_is_said() {
    let backend = Recorder::new();
    let mut announcements = Announcements::new(Rc::new(Speech::plain(backend.clone())), quiet());
    let written = |window: i64, text: &str| {
        Shown::Text(TextWritten {
            window,
            text: text.to_string(),
        })
    };
    announcements.screen_polled();
    announcements
        .pages(&[written(1, "Move animations.")])
        .unwrap();
    announcements.screen_polled();
    assert!(backend.spoken().is_empty());
    announcements
        .pages(&[written(1, "Switching."), written(2, "Elsewhere.")])
        .unwrap();
    announcements.screen_polled();
    assert_eq!(backend.spoken(), ["Switching."]);
    announcements.screen_polled();
    assert_eq!(backend.spoken(), ["Switching.", "Elsewhere."]);
    announcements.screen_polled();
    assert_eq!(backend.spoken(), ["Switching.", "Elsewhere."]);
}

/// Under `interrupt`, an option's description cut off the option (the audit, 2026-10-10).
#[test]
fn test_written_text_never_cuts_off_the_option_it_describes() {
    let backend = Recorder::new();
    let mut announcements = Announcements::new(
        Rc::new(Speech::new(
            backend.clone(),
            std::collections::HashSet::from([Category::Key, Category::Menu]),
            None,
        )),
        quiet(),
    );
    announcements
        .pages(&[Shown::Text(TextWritten {
            window: 1,
            text: "Move animations.".to_string(),
        })])
        .unwrap();
    announcements.screen_polled();
    announcements.screen_polled();
    assert_eq!(backend.spoken(), ["Move animations."]);
    assert_eq!(backend.interrupting(), Vec::<String>::new());
}

#[test]
fn test_a_task_is_due_every_period_or_sooner_when_asked() {
    let every = Every::new(2.0);
    assert!(every.due(10.0));
    assert!(!every.due(11.0));
    assert!(every.due(12.0));
    every.soon(12.5);
    assert!(every.due(12.5));
    every.wait(13.0);
    assert!(!every.due(14.0));
    every.reset();
    assert!(every.due(14.0));
}

/// An emulator whose answers took as long as `reply_seconds` says, and that is asked nothing
/// else.
struct Answering(Cell<f64>);

impl Rpc for Answering {
    fn read(&self, _address: i64, _size: i64) -> Result<Vec<u8>> {
        unimplemented!("only how long the replies took is asked")
    }

    fn read_each(&self, _reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        unimplemented!("only how long the replies took is asked")
    }

    fn reply_seconds(&self) -> f64 {
        self.0.get()
    }

    fn unsure(&self) -> bool {
        false
    }

    fn confirm(&self) -> Result<bool> {
        Ok(true)
    }

    fn attach(&self) -> Result<Process> {
        unimplemented!("only how long the replies took is asked")
    }
}

/// While an area loads every read takes 2 to 20 ms, and a look or a screen poll then held
/// polling up 0.4 to 1.2 s (sun_10.ctm, 2026-09-27).
#[test]
fn test_work_that_can_wait_is_put_off_while_the_emulator_answers_slowly_but_not_for_long() {
    let rpc = Rc::new(Answering(Cell::new(0.0)));
    let busy = Busy::new(rpc.clone());
    rpc.0.set(0.00002);
    assert!(!busy.now(10.0));
    rpc.0.set(0.005);
    assert!(busy.now(10.1));
    assert!(busy.now(10.1 + PATIENCE - 0.01));
    assert!(!busy.now(10.1 + PATIENCE));
    rpc.0.set(0.00002);
    assert!(!busy.now(12.0));
    rpc.0.set(0.005);
    assert!(busy.now(12.1));
}

/// A player testing from the start of the game restarts pa3ds more than once (2026-09-14).
#[test]
fn test_each_run_logs_afresh_and_keeps_the_runs_before() {
    let home = Home::new();
    for number in 0..KEPT_LOGS + 2 {
        std::fs::write(log_path(None, true).unwrap(), format!("run {number}")).unwrap();
    }
    let logs = home.home.join("logs");
    let read = |name: &str| std::fs::read_to_string(logs.join(name)).unwrap();
    assert_eq!(read("pa3ds.log"), format!("run {}", KEPT_LOGS + 1));
    assert_eq!(read("pa3ds.1.log"), format!("run {KEPT_LOGS}"));
    assert_eq!(read(&format!("pa3ds.{}.log", KEPT_LOGS - 1)), "run 2");
    assert!(!logs.join(format!("pa3ds.{KEPT_LOGS}.log")).exists());
}

/// ADR-0032, the user 2026-09-22: beside pa3ds, never %APPDATA% and never Program Files, so
/// the whole folder copies to another machine with the names and keys in it.
#[test]
fn test_everything_the_player_owns_sits_beside_pa3ds() {
    let home = Home::new();
    assert_eq!(
        default_log_path().unwrap().parent().unwrap(),
        home.home.join("logs")
    );
    assert_eq!(
        keyfile::keys_path(None).unwrap().parent().unwrap(),
        home.home
    );
    assert_eq!(keyfile::home_path().unwrap(), home.home);
}

/// Found from where pa3ds itself is. In Python, run from a checkout, that was the checkout; the
/// Rust pa3ds is always an executable, and its folder is where the player's folder goes beside
/// (ADR-0032).
#[test]
fn test_pa3ds_run_from_a_checkout_is_the_checkout() {
    let executable = std::env::current_exe().unwrap();
    assert_eq!(keyfile::app_path().unwrap(), executable.parent().unwrap());
}

#[test]
fn test_a_log_path_given_keeps_its_own_runs_beside_it() {
    let tmp = TmpDir::new();
    let target = tmp.path().join("logs").join("test.log");
    for number in 0..3 {
        std::fs::write(
            log_path(Some(&target), true).unwrap(),
            format!("run {number}"),
        )
        .unwrap();
    }
    let read = |path: PathBuf| std::fs::read_to_string(path).unwrap();
    assert_eq!(read(target.clone()), "run 2");
    assert_eq!(read(tmp.path().join("logs").join("test.1.log")), "run 1");
    assert_eq!(read(tmp.path().join("logs").join("test.2.log")), "run 0");
}

#[test]
fn test_a_folder_given_as_the_log_path_gets_the_usual_name() {
    let tmp = TmpDir::new();
    assert_eq!(
        log_path(Some(tmp.path()), true).unwrap(),
        tmp.path().join("pa3ds.log")
    );
}

fn logs_in(folder: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(folder)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|path| path.extension().is_some_and(|extension| extension == "log"))
                .collect()
        })
        .unwrap_or_default()
}

/// The user, 2026-09-14: test without writing logs, or write them elsewhere. The keys file
/// is written beside where the log would have gone, which is not a log.
#[test]
fn test_no_log_writes_no_file() {
    let home = Home::new();
    assert_eq!(main(&args(&["--silent", "--seconds", "0", "--no-log"])), 0);
    assert!(logs_in(&home.home).is_empty());
}

#[test]
fn test_a_run_logs_where_it_is_told() {
    let home = Home::new();
    let target = home.tmp().join("elsewhere.log");
    let given = target.to_string_lossy().into_owned();
    assert_eq!(
        main(&args(&["--silent", "--seconds", "0", "--log-path", &given])),
        0
    );
    assert!(target.exists());
    assert!(logs_in(&home.home).is_empty());
}

#[test]
fn test_no_log_and_a_log_path_cannot_both_be_asked_for() {
    let tmp = TmpDir::new();
    let given = tmp.path().join("x.log").to_string_lossy().into_owned();
    assert_eq!(main(&args(&["--no-log", "--log-path", &given])), 2);
}

/// Hooks whose runtime only keeps the settings it is handed and runs `then` on the speech.
fn keeping(
    seen: Rc<RefCell<Vec<Settings>>>,
    then: impl Fn(&Speech) + 'static,
) -> Box<dyn Fn(Rc<dyn Rpc>, Rc<Speech>, Settings, &dyn Fn() -> bool, Log) -> Result<()>> {
    Box::new(move |_rpc, speech, settings, _until, _log| {
        seen.borrow_mut().push(settings);
        then(&speech);
        Ok(())
    })
}

/// Without this the player's half of `pa3ds.core.names` was unreachable: `names_home` defaults
/// to None, which keeps a name for the run only, and nothing was passing anything else.
#[test]
fn test_a_run_keeps_the_players_own_names_beside_pa3ds() {
    let home = Home::new();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let hooks = Hooks {
        run: keeping(seen.clone(), |_speech| {}),
        ..Hooks::default()
    };
    main_with(&args(&["--silent", "--seconds", "0", "--no-log"]), &hooks);
    assert_eq!(
        seen.borrow()[0].names_home.as_deref(),
        Some(home.home.as_path())
    );
}

/// The rig calls `run` with the settings as they come, and a name given on this machine must
/// not change what a replay says on another.
#[test]
fn test_a_replay_keeps_no_names_of_its_own() {
    assert_eq!(Settings::default().names_home, None);
}

/// A backend that keeps whether each thing said cut off what came before it.
#[derive(Default)]
struct Interrupts(RefCell<Vec<(String, bool)>>);

impl Backend for Interrupts {
    fn speak(&self, text: &str, interrupt: bool) {
        self.0.borrow_mut().push((text.to_string(), interrupt));
    }

    fn braille(&self, _text: &str) {}
}

/// ADR-0010 and ADR-0022: a key's answer interrupts, because the key never reached Azahar's
/// window for the screen reader to cut speech on. `main` built its speech with an empty set of
/// interrupting categories until 2026-09-24, so the policy the tests checked was never the one
/// the player heard: cycling the list read every item out whole (the user, 2026-09-24).
/// `interrupt` adds menus to the key, never replaces it.
#[test]
fn test_the_runtime_cuts_speech_off_for_a_key_and_for_nothing_else() {
    for menus in [false, true] {
        let home = Home::new();
        let heard = Rc::new(Interrupts::default());
        let hooks = Hooks {
            run: keeping(Rc::new(RefCell::new(Vec::new())), |speech| {
                speech.say("door, 1 of 3, 1 north", Category::Key);
                speech.say("Bag", Category::Menu);
                speech.say("Welcome.", Category::Dialogue);
            }),
            silent: heard.clone(),
            in_front: Rc::new(|| true),
        };
        settings_file(&home.home, &format!("[pa3ds.ui]\ninterrupt = {menus}\n"));
        main_with(&args(&["--silent", "--seconds", "0", "--no-log"]), &hooks);
        assert_eq!(
            *heard.0.borrow(),
            [
                ("door, 1 of 3, 1 north".to_string(), true),
                ("Bag".to_string(), menus),
                ("Welcome.".to_string(), false)
            ],
            "menus {menus}"
        );
    }
}

/// ADR-0021: reading another window is never talked over, unless `unfocused`; pa3ds's
/// own lines are said either way.
#[test]
fn test_the_runtime_is_quiet_while_another_window_is_in_front() {
    for unfocused in [false, true] {
        let home = Home::new();
        let heard = Rc::new(Interrupts::default());
        let hooks = Hooks {
            run: keeping(Rc::new(RefCell::new(Vec::new())), |speech| {
                speech.say("pa3ds ready", Category::System);
                speech.say("Welcome.", Category::Dialogue);
            }),
            silent: heard.clone(),
            in_front: Rc::new(|| false),
        };
        settings_file(
            &home.home,
            &format!("[pa3ds.ui]\nunfocused = {unfocused}\n"),
        );
        main_with(
            &args(&["--silent", "--seconds", "0", "--no-log", "--no-keys"]),
            &hooks,
        );
        let said: Vec<String> = heard
            .0
            .borrow()
            .iter()
            .map(|(text, _)| text.clone())
            .collect();
        let expected: &[&str] = if unfocused {
            &["pa3ds ready", "Welcome."]
        } else {
            &["pa3ds ready"]
        };
        assert_eq!(said, expected, "unfocused {unfocused}");
    }
}

/// The user, 2026-09-16: 60 Hz, with 30 as a setting for machines that cannot keep up.
#[test]
fn test_the_poll_rate_is_the_frame_rate_unless_asked_for_less() {
    let home = Home::new();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let hooks = Hooks {
        run: keeping(seen.clone(), |_speech| {}),
        ..Hooks::default()
    };
    let argv = args(&["--silent", "--seconds", "0", "--no-log"]);
    main_with(&argv, &hooks);
    settings_file(&home.home, "[pa3ds]\npoll_rate = 30\n");
    main_with(&argv, &hooks);
    settings_file(&home.home, "[pa3ds]\npoll_rate = 0\n");
    main_with(&argv, &hooks);
    let ticks: Vec<f64> = seen.borrow().iter().map(|settings| settings.tick).collect();
    for (tick, expected) in ticks.iter().zip([1.0 / 60.0, 1.0 / 30.0, 1.0 / 60.0]) {
        assert!((tick - expected).abs() < 1e-9, "{ticks:?}");
    }
    assert_eq!(ticks.len(), 3);
}

/// ADR-0026: set once in a file rather than at every start.
#[test]
fn test_the_settings_file_reaches_the_readers() {
    let home = Home::new();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let hooks = Hooks {
        run: keeping(seen.clone(), |_speech| {}),
        ..Hooks::default()
    };
    settings_file(
        &home.home,
        "[pa3ds.ui]\npauses = true\n\n[pa3ds.navigation]\nturn_by_turn = false\n",
    );
    main_with(
        &args(&["--silent", "--seconds", "0", "--no-log", "--no-keys"]),
        &hooks,
    );
    let seen = seen.borrow();
    assert!(seen[0].ui.pauses && !seen[0].ui.interrupt);
    assert!(!seen[0].navigation.turn_by_turn);
}

/// An exception raised here, as one caught in the loop is.
fn raising(place: &str) -> Failure {
    Failure::of_error(&Error::value(place))
}

/// ADR-0019. A player cannot act on a traceback; they can act on knowing this screen is
/// broken, and the silence before this was indistinguishable from a crash.
#[test]
fn test_a_reader_that_raises_is_logged_and_said_once() {
    let tmp = TmpDir::new();
    let backend = Recorder::new();
    let (log, logged) = logger();
    let written = tmp.path().join("20260923-000000_reader-error.idx");
    let faults = Faults::new(
        Rc::new(Speech::plain(backend.clone())),
        log,
        Some(Box::new(move || Ok(Some(written.clone())))),
        Faults::QUIET,
        Faults::CAPTURES,
    );
    let failure = raising("a reader");
    assert!(faults.caught(&failure, 100.0).is_some());
    assert_eq!(backend.spoken(), ["Reader error"]);
    assert!(logged.borrow().join("\n").contains("ValueError: a reader"));
    assert!(logged.borrow().join("\n").contains("raised at"));
    // The same line failing the next poll, and every poll after: already said, and already logged.
    assert_eq!(faults.caught(&failure, 100.02), None);
    assert_eq!(faults.caught(&failure, 130.0), None);
    assert_eq!(backend.spoken(), ["Reader error"]);
}

/// The same line failing on a screen the player has walked to since is news. A quiet spell is
/// the only "another screen" the runtime can tell apart without a screen of its own.
#[test]
fn test_a_fault_that_comes_back_after_a_quiet_spell_is_said_again() {
    let backend = Recorder::new();
    let faults = Faults::new(
        Rc::new(Speech::plain(backend.clone())),
        quiet(),
        None,
        10.0,
        Faults::CAPTURES,
    );
    let failure = raising("a reader");
    faults.caught(&failure, 100.0);
    faults.caught(&failure, 105.0);
    faults.caught(&failure, 120.0);
    assert_eq!(backend.spoken(), ["Reader error", "Reader error"]);
}

/// Each fault is keyed by what was raised and where. Python's second fault was a `KeyError` with
/// no traceback, keyed by its class alone; a Rust error always knows where it was raised.
#[test]
fn test_each_bug_is_said_for_itself() {
    let backend = Recorder::new();
    let faults = Faults::new(
        Rc::new(Speech::plain(backend.clone())),
        quiet(),
        None,
        Faults::QUIET,
        Faults::CAPTURES,
    );
    let first = faults.caught(&raising("a reader"), 100.0);
    let second = faults.caught(&Failure::of_error(&Error::key("shape")), 100.0);
    assert_eq!(backend.spoken(), ["Reader error", "Reader error"]);
    assert_ne!(first, second);
    assert!(first.unwrap().contains("test_app.rs"));
    assert!(second.unwrap().starts_with("KeyError"));
}

/// The snapshot is what a test replays the failure through, so one is taken with each fault
/// said; a run left going overnight on a broken screen must not fill the disk with them.
#[test]
fn test_a_snapshot_of_the_failure_is_written_and_then_capped() {
    let (log, logged) = logger();
    let taken = Rc::new(Cell::new(0));
    let counter = taken.clone();
    let faults = Faults::new(
        Rc::new(Speech::plain(Recorder::new())),
        log,
        Some(Box::new(move || {
            counter.set(counter.get() + 1);
            Ok(Some(PathBuf::from(format!(
                "capture-{}.idx",
                counter.get()
            ))))
        })),
        0.0,
        2,
    );
    for tick in 0..4 {
        faults.caught(&raising("a reader"), tick as f64);
    }
    assert_eq!(taken.get(), 2);
    let logged = logged.borrow();
    let snapshots: Vec<&String> = logged
        .iter()
        .filter(|line| line.starts_with("snapshot"))
        .collect();
    assert_eq!(
        snapshots,
        [
            "snapshot of the failure: capture-1.idx",
            "snapshot of the failure: capture-2.idx"
        ]
    );
    assert!(
        logged
            .iter()
            .any(|line| line == "no snapshot: this run has written all it will")
    );
}

/// The emulator having stopped answering is a likely reason the reader failed at all.
#[test]
fn test_a_snapshot_that_cannot_be_taken_is_not_a_second_failure() {
    let backend = Recorder::new();
    let (log, logged) = logger();
    let faults = Faults::new(
        Rc::new(Speech::plain(backend.clone())),
        log,
        Some(Box::new(|| {
            Err(Error::new(Kind::OsError, "the emulator is gone"))
        })),
        Faults::QUIET,
        Faults::CAPTURES,
    );
    assert!(faults.caught(&raising("a reader"), 100.0).is_some());
    assert_eq!(backend.spoken(), ["Reader error"]);
    assert!(
        logged
            .borrow()
            .iter()
            .any(|line| line.contains("no snapshot of the failure"))
    );
}

/// An executable naming the message window classes, as the emulator serves it.
fn emulator_running_a_game() -> HashMap<i64, Vec<u8>> {
    let memory = FakeMemory::new();
    // `regions` probes the first page of every 16 KiB, so a code image whose first page is blank
    // is a code image the runtime cannot find at all.
    memory.write(CODE.0, b"fake executable");
    let classes = FakeClasses::new(memory.clone());
    for mangled in msgwin::CLASSES {
        classes.add(mangled);
    }
    let image = memory.read(CODE.0, 0x2000).unwrap();
    (0..image.len())
        .step_by(MAX_READ as usize)
        .map(|at| {
            (
                CODE.0 + at as i64,
                image[at..at + MAX_READ as usize].to_vec(),
            )
        })
        .collect()
}

/// A profile that is the catalog's in all but its dialogue poll, which `poll` stands in for:
/// where the Python's tests patched `DialogueReader.poll`.
struct Patched {
    real: Rc<dyn Profile>,
    poll: Box<dyn Fn(&dyn Profile) -> Result<bool>>,
}

impl Profile for Patched {
    fn track(&self, found: &crate::core::discovery::Found) -> Result<()> {
        self.real.track(found)
    }
    fn poll_app(&self) -> Result<bool> {
        self.real.poll_app()
    }
    fn app_sweep_wanted(&self) -> bool {
        self.real.app_sweep_wanted()
    }
    fn poll_dialogue(&self) -> Result<bool> {
        (self.poll)(&*self.real)
    }
    fn poll_menus(&self, discovery: &mut crate::core::discovery::Discovery) -> Result<bool> {
        self.real.poll_menus(discovery)
    }
    fn poll_screen(&self) -> Result<()> {
        self.real.poll_screen()
    }
    fn poll_field(&self) -> Result<()> {
        self.real.poll_field()
    }
    fn poll_guidance(&self) -> Result<()> {
        self.real.poll_guidance()
    }
    fn poll_footsteps(&self) -> Result<()> {
        self.real.poll_footsteps()
    }
    fn poll_pointers(&self) -> Result<()> {
        self.real.poll_pointers()
    }
    fn poll_renames(&self) -> Result<()> {
        self.real.poll_renames()
    }
    fn press(&self, key: &str) -> Result<()> {
        self.real.press(key)
    }
    fn context(&self) -> Option<&'static str> {
        self.real.context()
    }
    fn sweep_wanted(&self) -> Result<bool> {
        self.real.sweep_wanted()
    }
}

/// The catalog's game, with its dialogue poll made `patch(session, real profile)`.
fn patched(
    patch: impl Fn(&Session) -> Box<dyn Fn(&dyn Profile) -> Result<bool>> + 'static,
) -> impl Fn(Running, Rc<dyn Rpc>, Rc<Speech>, (i64, i64), Setup, &[u8]) -> Result<Session> {
    move |game, rpc, speech, code, setup, image| {
        let mut session = catalog::start(game, rpc, speech, code, setup, image)?;
        let poll = patch(&session);
        session.profile = Rc::new(Patched {
            real: session.profile.clone(),
            poll,
        });
        Ok(session)
    }
}

fn ticks_up_to(count: i64) -> impl Fn() -> bool {
    let ticks = Cell::new(0);
    move || {
        ticks.set(ticks.get() + 1);
        ticks.get() > count
    }
}

/// The bug ADR-0019 was written for and did not cover: anything a reader raised that was not
/// an RpcError left the loop altogether, so one unexpected shape took pa3ds down mid-game.
#[test]
fn test_a_reader_that_raises_does_not_stop_the_loop() {
    let tmp = TmpDir::new();
    let backend = Recorder::new();
    let (log, logged) = logger();
    let ticks = Rc::new(Cell::new(0));
    let counted = ticks.clone();
    let start = patched(|session| {
        let pages = session
            .pages
            .clone()
            .expect("gen 6 reads through a page cache");
        Box::new(move |_real| {
            // Read before raising, as a reader that got partway through a structure has. The
            // snapshot is of what was read, so a reader that failed before reading anything
            // leaves none.
            memory::phase(&*pages, "dialogue", || pages.read(CODE.0, 4))?;
            Err(Error::value("a shape no reader expected"))
        })
    });
    let emu = FakeEmulator::with(emulator_running_a_game(), game(), 0, 0);
    let until = move || {
        counted.set(counted.get() + 1);
        counted.get() > 20
    };
    run_with(
        connect(&emu),
        Rc::new(Speech::plain(backend.clone())),
        Settings {
            tick: 0.0,
            captures: Some(tmp.path().to_path_buf()),
            ..Settings::default()
        },
        &until,
        log,
        &start,
    )
    .unwrap();
    assert!(ticks.get() > 20, "the loop gave up at the first exception");
    assert_eq!(
        backend
            .spoken()
            .iter()
            .filter(|said| *said == "Reader error")
            .count(),
        1
    );
    assert!(
        logged
            .borrow()
            .join("\n")
            .contains("a shape no reader expected")
    );
    assert!(std::fs::read_dir(tmp.path()).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with("_reader-error.idx")
    }));
}

/// Have the emulator stop answering once, a few polls in: as another emulator started in its
/// place with `restart`, which has no process selected, or as a hiccup without.
fn lose_the_emulator(
    emu: Rc<FakeEmulator>,
    restart: bool,
) -> impl Fn(Running, Rc<dyn Rpc>, Rc<Speech>, (i64, i64), Setup, &[u8]) -> Result<Session> {
    let polls = Rc::new(Cell::new(0));
    patched(move |_session| {
        let (emu, polls) = (emu.clone(), polls.clone());
        Box::new(move |real| {
            polls.set(polls.get() + 1);
            if polls.get() == 3 {
                if restart {
                    emu.restart(0);
                }
                return Err(Error::new(Kind::RpcTimeoutError, "no reply"));
            }
            real.poll_dialogue()
        })
    })
}

/// 2026-09-25: a session left running retried its reads into the rig's new emulator before
/// anything had selected a process there, and the emulator crashed. The game is waited for and
/// attached again instead, and nothing is read in between.
#[test]
fn test_a_new_emulator_in_the_old_ones_place_is_attached_before_it_is_read() {
    let (log, logged) = logger();
    let emu = Rc::new(FakeEmulator::with(emulator_running_a_game(), game(), 0, 0));
    run_with(
        connect(&emu),
        Rc::new(Speech::plain(Recorder::new())),
        Settings {
            tick: 0.0,
            wait: 0.0,
            ..Settings::default()
        },
        &ticks_up_to(40),
        log,
        &lose_the_emulator(emu.clone(), true),
    )
    .unwrap();
    assert_eq!(emu.unselected_reads(), 0);
    assert_eq!(emu.selected(), GAME.0);
    assert!(
        logged
            .borrow()
            .iter()
            .any(|line| line.contains("waiting for a game again"))
    );
}

/// A hiccup keeps the process selected, so the game is read on without starting over.
#[test]
fn test_the_same_emulator_answering_again_is_read_on() {
    let (log, logged) = logger();
    let emu = Rc::new(FakeEmulator::with(emulator_running_a_game(), game(), 0, 0));
    run_with(
        connect(&emu),
        Rc::new(Speech::plain(Recorder::new())),
        Settings {
            tick: 0.0,
            wait: 0.0,
            ..Settings::default()
        },
        &ticks_up_to(40),
        log,
        &lose_the_emulator(emu.clone(), false),
    )
    .unwrap();
    let logged = logged.borrow();
    assert!(
        logged
            .iter()
            .any(|line| line == "the emulator is answering again")
    );
    assert!(
        !logged
            .iter()
            .any(|line| line.contains("waiting for a game again"))
    );
}

/// Orion, 2026-10-17: a player who started pa3ds again over one that hung may be in another
/// window, so the restart is said where "pa3ds ready" goes, cutting off whatever was being said.
/// A game that goes and comes back afterwards is ready, not restarted.
#[test]
fn test_a_restarted_run_says_so_interrupting_and_only_the_first_time() {
    let backend = Recorder::new();
    let emu = Rc::new(FakeEmulator::with(emulator_running_a_game(), game(), 0, 0));
    run_with(
        connect(&emu),
        Rc::new(Speech::plain(backend.clone())),
        Settings {
            tick: 0.0,
            wait: 0.0,
            restarted: true,
            ..Settings::default()
        },
        &ticks_up_to(40),
        Rc::new(|_: &str| {}),
        &lose_the_emulator(emu.clone(), true),
    )
    .unwrap();
    let said: Vec<String> = backend
        .spoken()
        .into_iter()
        .filter(|said| said.starts_with("pa3ds"))
        .collect();
    assert_eq!(said, ["pa3ds restarted", "pa3ds ready"]);
    assert!(
        backend
            .interrupting()
            .contains(&"pa3ds restarted".to_string())
    );
}

/// Orion, 2026-10-17: someone who copied only pa3ds.exe hears why pa3ds says so little, and pa3ds
/// still reads the game.
#[test]
fn test_a_missing_data_folder_is_said_once_and_pa3ds_runs_on() {
    let tmp_path = TmpDir::new();
    DATA_IN_TEST.with(|moved| *moved.borrow_mut() = Some(tmp_path.path().join("nowhere")));
    let backend = Recorder::new();
    let (log, logged) = logger();
    let emu = FakeEmulator::with(emulator_running_a_game(), game(), 0, 0);
    run(
        connect(&emu),
        Rc::new(Speech::plain(backend.clone())),
        Settings {
            tick: 0.0,
            wait: 0.0,
            ..Settings::default()
        },
        &ticks_up_to(40),
        log,
    )
    .unwrap();
    let spoken = backend.spoken();
    assert_eq!(
        spoken.iter().filter(|said| *said == MISSING_DATA).count(),
        1
    );
    assert!(backend.interrupting().contains(&MISSING_DATA.to_string()));
    assert!(spoken.contains(&"pa3ds ready".to_string()));
    assert!(
        logged
            .borrow()
            .iter()
            .any(|line| line.contains("no data folder"))
    );
}

#[test]
fn test_a_data_folder_beside_pa3ds_says_nothing_of_it() {
    let backend = Recorder::new();
    let emu = FakeEmulator::with(emulator_running_a_game(), game(), 0, 0);
    run(
        connect(&emu),
        Rc::new(Speech::plain(backend.clone())),
        Settings {
            tick: 0.0,
            wait: 0.0,
            ..Settings::default()
        },
        &ticks_up_to(40),
        Rc::new(|_: &str| {}),
    )
    .unwrap();
    assert!(!backend.spoken().contains(&MISSING_DATA.to_string()));
}

/// `emulator_running_a_game`, with the save's trainer record in the heap saying `language`
/// (`mystatus`).
fn emulator_running_a_save_in(language: u8) -> HashMap<i64, Vec<u8>> {
    let memory = FakeMemory::new();
    memory.write(CODE.0, b"fake executable");
    let classes = FakeClasses::new(memory.clone());
    for mangled in msgwin::CLASSES {
        classes.add(mangled);
    }
    let record = 0x08100000;
    memory.u32(record, &[classes.add(mystatus::CLASS)]);
    memory.write(record + mystatus::LANGUAGE, &[language]);
    let image = memory.read(CODE.0, 0x2000).unwrap();
    let mut served: HashMap<i64, Vec<u8>> = (0..image.len())
        .step_by(MAX_READ as usize)
        .map(|at| {
            (
                CODE.0 + at as i64,
                image[at..at + MAX_READ as usize].to_vec(),
            )
        })
        .collect();
    served.insert(record, memory.read(record, MAX_READ).unwrap());
    served
}

/// `run(rpc, Speech(backend), Settings(tick=0.0, wait=0.0, ui=ui), ticks_up_to(40), log)` on an
/// emulator whose save says `language`.
fn run_a_save_in(language: u8, backend: &Rc<Recorder>, ui: Ui, log: Log) {
    let emu = FakeEmulator::with(emulator_running_a_save_in(language), game(), 0, 0);
    run(
        connect(&emu),
        Rc::new(Speech::plain(backend.clone())),
        Settings {
            tick: 0.0,
            wait: 0.0,
            ui,
            ..Settings::default()
        },
        &ticks_up_to(40),
        log,
    )
    .unwrap();
}

/// ADR-0040: English until the save says otherwise, then the game's text, the names and pa3ds's
/// own words in the save's language, saying nothing of it: one "pa3ds ready".
#[test]
fn test_a_game_whose_save_says_another_language_is_read_again_in_it_quietly() {
    let backend = Recorder::new();
    let (log, logged) = logger();
    run_a_save_in(3, &backend, Ui::default(), log);
    assert_eq!(
        backend
            .spoken()
            .iter()
            .filter(|said| *said == "pa3ds ready")
            .count(),
        1
    );
    assert!(
        logged
            .borrow()
            .iter()
            .any(|line| line.contains("the save says the game is in fr, not en"))
    );
    assert_eq!(words::current().language, "fr");
}

#[test]
fn test_a_game_whose_save_says_english_is_read_once() {
    let backend = Recorder::new();
    let (log, logged) = logger();
    run_a_save_in(2, &backend, Ui::default(), log);
    assert!(
        !logged
            .borrow()
            .iter()
            .any(|line| line.contains("the save says"))
    );
    assert_eq!(words::current().language, "en");
}

/// The Chinese fan Omega Ruby's save says Japanese, whose text its Chinese replaced: the game's
/// words are read as the Japanese, and pa3ds's own are the setting's.
#[test]
fn test_the_language_setting_says_pa3ds_words_and_leaves_the_games_to_the_save() {
    run_a_save_in(
        1,
        &Recorder::new(),
        Ui {
            language: "zh-CN".to_string(),
            ..Ui::default()
        },
        Rc::new(|_: &str| {}),
    );
    assert_eq!(words::current().language, "zh-CN");
}

#[test]
fn test_gives_up_when_told_to_stop() {
    let emu = FakeEmulator::with(HashMap::new(), Vec::new(), 0, 0);
    let attempts = Cell::new(0);
    let until = || {
        attempts.set(attempts.get() + 1);
        attempts.get() > 3
    };
    let process = wait_for_game(
        &*connect(&emu),
        &Speech::plain(Recorder::new()),
        &until,
        &quiet(),
        &Settings {
            wait: 0.0,
            ..Settings::default()
        },
    )
    .unwrap();
    assert_eq!(process, None);
}

fn game_rom(folder: &Path, title: i64, encrypted: bool) -> PathBuf {
    let path = folder.join(if encrypted { "locked.3ds" } else { "or.3ds" });
    let files = oras_files(None);
    let files: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(name, data)| (name.as_str(), data.as_slice()))
        .collect();
    let image = if encrypted {
        rom_image_with(&files, title, true)
    } else {
        rom_image(&files, title)
    };
    std::fs::write(&path, image).unwrap();
    path
}

fn process() -> Process {
    Process {
        pid: 11,
        title_id: GAME.1 as i64,
        name: "sango-1".to_string(),
    }
}

/// Nothing for the player to set: Azahar already knows which ROMs it has opened, and the
/// title id says which of them is running.
#[test]
fn test_the_games_own_words_come_from_the_rom_azahar_last_opened() {
    let home = Home::new();
    let locked = game_rom(home.tmp(), GAME.1 as i64, true);
    let wanted = game_rom(home.tmp(), GAME.1 as i64, false);
    let config = home
        .tmp()
        .join("appdata")
        .join("Azahar")
        .join("config")
        .join("qt-config.ini");
    std::fs::create_dir_all(config.parent().unwrap()).unwrap();
    let posix = |path: &Path| path.to_string_lossy().replace('\\', "/");
    std::fs::write(
        &config,
        format!(
            "Paths\\recentFiles={}, {}\n",
            posix(&locked),
            posix(&wanted)
        ),
    )
    .unwrap();
    let (log, logged) = logger();
    let text = game_text(&Setup::new(process(), log), oras::rom()).unwrap();
    assert_eq!(text.place(3), Some("Littleroot Town".to_string()));
    assert_eq!(
        *logged.borrow(),
        [format!(
            "the game's own words from {} (en)",
            crate::py::path_str(&wanted)
        )]
    );
}

#[test]
fn test_a_rom_of_another_game_is_refused() {
    let home = Home::new();
    let (log, logged) = logger();
    let other = game_rom(home.tmp(), 0x0004000000055D00, false);
    let setup = Setup {
        rom: Some(other),
        ..Setup::new(process(), log)
    };
    assert_eq!(game_text(&setup, oras::rom()).unwrap(), no_text());
    assert!(logged.borrow()[0].contains("the game running is 000400000011c400"));
}

#[test]
fn test_without_a_rom_the_game_runs_on_without_its_own_words() {
    let _home = Home::new();
    let (log, logged) = logger();
    assert_eq!(
        game_text(&Setup::new(process(), log), oras::rom()).unwrap(),
        no_text()
    );
    assert!(logged.borrow()[0].starts_with("no decrypted ROM of title 000400000011c400"));
}

/// The runtime names no game's module: the game's engine hands it a session. Gen 6 reads
/// through a page cache, whose pages a reader's failure saves, and sweeps the heap span; Sun reads
/// to the byte and walks its heaps (ADR-0035).
#[test]
fn test_each_engine_puts_its_own_game_together() {
    let _home = Home::new();
    let emu = FakeEmulator::new(HashMap::new());
    let rpc = connect(&emu);
    let gen6 = catalog::start(
        Running::Gen6(oras::game()),
        rpc.clone(),
        Rc::new(Speech::plain(Recorder::new())),
        CODE,
        Setup::new(
            Process {
                pid: 11,
                title_id: oras::OMEGA_RUBY,
                name: "sango-1".into(),
            },
            quiet(),
        ),
        &[],
    )
    .unwrap();
    let sun = catalog::start(
        Running::SunMoon(sm::game()),
        rpc,
        Rc::new(Speech::plain(Recorder::new())),
        CODE,
        Setup::new(
            Process {
                pid: 11,
                title_id: sm::SUN,
                name: "kujira-1".into(),
            },
            quiet(),
        ),
        &[],
    )
    .unwrap();
    assert!(gen6.pages.is_some() && gen6.walk.is_none());
    assert!(gen6.classes.iter().any(|class| class == msgwin::CLASSES[0]));
    assert!(sun.pages.is_none() && sun.walk.is_some());
    // A NO_PROCESS emulator is the one that has been asked for nothing yet.
    assert_eq!(emu.selected(), NO_PROCESS);
}
