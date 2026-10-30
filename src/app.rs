//! The runtime: wait for the game, find what to read, poll it, and say what changed.
//!
//! This is the schedule: when to measure the heaps, sweep them and poll the readers. What to look
//! for, what the readers are and how the heaps are read is the game's, handed over as a session by
//! its engine's package (`games.catalog.start`); finding objects is `discovery`'s.
//!
//! Bound by:
//! - ADR-0004: the poll runs at 60 Hz and nothing holds it up for long: a sweep reads a chunk at a
//!   time, polling between.
//! - ADR-0019: a reader that throws is logged with a snapshot, and says "Reader error" once a bug a
//!   minute.
//! - ADR-0041: a replay logs touch mode's targets as they change (`Settings.census`), a player's
//!   run never; and only a tape replay P's ways (`Settings.ways`).

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::panic::{self, AssertUnwindSafe, Location};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, mpsc};

use indexmap::IndexMap;

use crate::core::clips::Clips;
use crate::core::config::{Amie, Audio, Extras, Navigation, Touch, Ui};
use crate::core::discovery::{CODE_SPAN, Discovery};
use crate::core::heaps::Block;
use crate::core::keyfile::{data_path, emulator_config};
use crate::core::keys::Hotkeys;
use crate::core::loudness::Loudness;
use crate::core::memory::{self, Memory};
use crate::core::rpc::{Process, Rpc};
use crate::core::rtti::demangle;
use crate::core::snapshot::{self, PAGE};
use crate::core::speech::{Category, Speech};
use crate::core::tones::Tones;
use crate::core::touch::{self, SharedLog, TouchServer};
use crate::core::words::own;
use crate::games::catalog;
use crate::games::common::field::keys::{
    Bindings, Contexts, DEFAULT_BINDINGS, DEFAULT_CONTEXTS, bound_keys, context_keys, drag_keys,
};
use crate::games::common::session::{Session, Setup, Ways};
use crate::py::{Error, Kind, Log, Result, time};
use crate::tape;

#[derive(Clone)]
pub struct Settings {
    /// Seconds between polls: the game's frame rate by default, as ADR-0004 has always said. The
    /// runtime shipped at 30 Hz and managed 20.6 polls a second (spike 59), because sweeps held it
    /// up; a dialogue poll itself takes under a millisecond. `poll_rate = 30` for a slower machine.
    pub tick: f64,
    /// Pages a sweep reads before polling again. A heap sweep read whole held polling up for about
    /// 300 ms every couple of seconds (spike 59); 512 pages is about 13 ms.
    pub sweep_chunk: i64,
    /// Seconds between looks at the game's heaps for the blocks they have allocated since the last,
    /// which are swept at once for the objects the readers want (`heaps`, spike 110). Every object a
    /// sweep of the whole heap found over movies 4 and 5 was in one of them first, a second or two
    /// sooner than a sweep every 2 s would have found it.
    pub heap_tick: f64,
    /// Seconds after another app starts — the field, the Bag, the Pokédex, a battle — to sweep the
    /// whole heap, whatever the schedule says: a net for what the app builds into memory taken
    /// before, as the field finds its zone data through a holder no new block holds (movie4,
    /// 2026-09-26). One read a poll says when one starts (`Gen6Profile.poll_app`, spike 99), and
    /// the profile whether it wants the sweep: ORAS's field coming back does not (spike 134).
    pub app_sweep: f64,
    /// Seconds after a block appears to sweep it again. A pool is taken whole and built into after:
    /// swept only as it appeared, the move models, the talk windows and the camera waited for the
    /// whole sweep, and the field said nothing for 18 s after the game was continued (movie4,
    /// 2026-09-26).
    pub sweep_again: Vec<f64>,
    /// Seconds between sweeps of the whole heap: the net under the heaps' own watch, and the moment
    /// readers forget what has gone. A whole sweep is made at once whenever a reader is waiting for
    /// one. Until spike 110 the whole heap was swept every 8 s and around what had been found every
    /// 2 s, and after every dialogue page; then every 24 and 6 s, and when a screen changed.
    pub whole_rescan: f64,
    /// Seconds between polls of screen text. Each reads every TextBox a sweep found and every pane
    /// above it, several times the reads of a dialogue poll, and screens change at the player's
    /// pace.
    pub screen_tick: f64,
    /// Seconds between attempts to reach an emulator that is not running a game yet.
    pub wait: f64,
    /// How menus, lists and screens are said (`config.toml`'s `[pa3ds.ui]`). Pauses are off unless
    /// asked for, since a screen reader reads "Torchic, ♂, L, 5" more clumsily than "Torchic ♂ L 5"
    /// (the user, 2026-09-14).
    pub ui: Ui,
    /// How loud pa3ds's own tones and footsteps are (`[pa3ds.audio]`).
    pub audio: Audio,
    /// How the map's list and the way to a thing are said (`[pa3ds.navigation]`).
    pub navigation: Navigation,
    pub touch: Touch,
    pub amie: Amie,
    /// What the game shows in a way pa3ds cannot read, said from its numbers
    /// (`[pa3ds.extras]`).
    pub extras: Extras,
    /// The language a game is read in until its save says another, which it is read in from then
    /// on (ADR-0040): its text out of the ROM, and pa3ds's own words where `[pa3ds.ui] language`
    /// does not say otherwise.
    pub language: String,
    /// Where the player's own names live, and where a rename is written: beside pa3ds itself from
    /// the command line (ADR-0032). None keeps them for the run only, which is what a replay wants —
    /// a replay must say the same thing on any machine, and a name given on this one would change
    /// it.
    pub names_home: Option<PathBuf>,
    /// Navigation mode -> key -> action, answered while the emulator's window is in front, or
    /// None for no hook at all, as a replay in the rig wants.
    pub keys: Option<Bindings>,
    /// Context -> key -> action: a screen's own keys, taken only while it runs.
    pub contexts: Contexts,
    /// The player's ROM, for the game's own words RAM only numbers: a place's name, the
    /// rival's. None finds it among the ROMs Azahar has opened, by the running game's title id.
    pub rom: Option<PathBuf>,
    /// Where a snapshot of a reader's failure is written (`Faults`). None is `capture_dir()` on
    /// C:, where raw captures live, because the pages a reader was reading can hold save data
    /// (ADR-0029).
    pub captures: Option<PathBuf>,
    /// Whether this run replaced a pa3ds that was running (`instance.replace`): its first "pa3ds
    /// ready" is "pa3ds restarted", interrupting, so a player who restarted a pa3ds that had hung
    /// hears that it took, in whatever window they are in (Orion, 2026-10-17).
    pub restarted: bool,
    /// Azahar's `qt-config.ini`, where it lists the ROMs it has opened (`config.Config.emulator`).
    /// None is the one in AppData.
    pub emulator: Option<PathBuf>,
    /// Log touch mode's targets as they change, every screen poll: a replay presses no key of
    /// pa3ds's, so without it nothing touch mode offers is on a tape (ADR-0041). On for replays,
    /// never for a player.
    pub census: bool,
    /// Log what P and Shift and P say of the way to each thing listed, at each listing, read
    /// inside this: a tape replay's, which answers from what the tape holds and takes nothing from
    /// the replay's own answers. None logs none: a player's run, and a recording, where the
    /// searches would only make the replays drift (ADR-0041, Orion, 2026-10-29).
    pub ways: Option<Ways>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            tick: 1.0 / 60.0,
            sweep_chunk: 512,
            heap_tick: 0.1,
            app_sweep: 1.5,
            sweep_again: vec![0.5, 2.0],
            whole_rescan: 30.0,
            screen_tick: 0.2,
            wait: 1.0,
            ui: Ui::default(),
            audio: Audio::default(),
            navigation: Navigation::default(),
            touch: Touch::default(),
            amie: Amie::default(),
            extras: Extras::default(),
            language: "en".to_string(),
            names_home: None,
            keys: None,
            contexts: DEFAULT_CONTEXTS.clone(),
            rom: None,
            captures: None,
            restarted: false,
            emulator: None,
            census: false,
            ways: None,
        }
    }
}

/// Attach as soon as the emulator is running a game, so pa3ds can be started before it.
///
/// Azahar's RPC server answers only once a game is running. Started after the game instead,
/// pa3ds means switching windows during the opening, and its first line is gone by then.
pub fn wait_for_game(
    rpc: &dyn Rpc,
    speech: &Speech,
    until: &dyn Fn() -> bool,
    log: &Log,
    settings: &Settings,
) -> Result<Option<Process>> {
    let mut told = false;
    while !until() {
        match rpc.attach() {
            Ok(process) => return Ok(Some(process)),
            Err(error) if error.is(Kind::RpcError) => {
                if !told {
                    log("waiting for the game");
                    speech.say(&own("app-waiting", &[]), Category::System);
                    told = true;
                }
                time::sleep(settings.wait);
            }
            Err(error) => return Err(error),
        }
    }
    Ok(None)
}

/// When a task is next due: every `period` seconds, or sooner when asked.
pub struct Every {
    period: f64,
    next: Cell<f64>,
}

impl Every {
    pub fn new(period: f64) -> Self {
        Every {
            period,
            next: Cell::new(0.0),
        }
    }

    /// Whether the task is due now. If it is, the next time is set from now.
    pub fn due(&self, now: f64) -> bool {
        if now < self.next.get() {
            return false;
        }
        self.next.set(now + self.period);
        true
    }

    /// Be due by `at` at the latest.
    pub fn soon(&self, at: f64) {
        // `min(self._next, at)`: the first unless the second is less.
        if at < self.next.get() {
            self.next.set(at);
        }
    }

    /// Be due a whole period from now, as though the task had just run.
    pub fn wait(&self, now: f64) {
        self.next.set(now + self.period);
    }

    /// Be due at once.
    pub fn reset(&self) {
        self.next.set(0.0);
    }
}

/// Seconds a read takes the emulator to answer past which it is busy: 0.01 to 0.05 ms normally,
/// 2 to 20 ms while it loads an area.
pub const SLOW_REPLY: f64 = 0.001;
/// The longest the work that can wait is put off for, so an emulator slow all the time is read
/// as it was before.
pub const PATIENCE: f64 = 1.5;
/// Said once as pa3ds starts without `<pa3ds>/data` beside it, as when only `pa3ds.exe` was copied
/// somewhere: pa3ds runs on, but every word of its own, the shipped names and the scenes are gone.
pub const MISSING_DATA: &str = "pa3ds data folder not found";

/// Whether to put off the work that can wait, because the emulator is answering slowly.
///
/// While an area loads every read takes it 2 to 20 ms, pipelined or not, and whichever poll ran
/// then held the loop up: the heaps' look, the field's listing, the screen poll's reads fetched
/// ahead, 0.4 to 1.2 s each as Route 1 loaded, with asynchronous reads as with deterministic
/// ones (sun_10.ctm, 2026-09-27). Afterwards the same reads take a hundredth of a millisecond.
/// Dialogue is polled all the same.
pub struct Busy {
    rpc: Rc<dyn Rpc>,
    since: Cell<Option<f64>>,
}

impl Busy {
    pub fn new(rpc: Rc<dyn Rpc>) -> Self {
        Busy {
            rpc,
            since: Cell::new(None),
        }
    }

    pub fn now(&self, now: f64) -> bool {
        if self.rpc.reply_seconds() < SLOW_REPLY {
            self.since.set(None);
            return false;
        }
        let since = match self.since.get() {
            Some(since) => since,
            None => {
                self.since.set(Some(now));
                now
            }
        };
        now - since < PATIENCE
    }
}

/// A reader's failure as the runtime catches it: what was raised and where, as one line (the key
/// that makes one bug one fault), and what the log shows of it.
pub struct Failure {
    pub fault: String,
    pub detail: String,
}

impl Failure {
    /// A raised exception: Python's `_fault` keys it by the traceback's last frame, which is where
    /// `py::Error` records it was raised. A short read was raised in `core::binary` for every
    /// reader alike, so the line that called it is logged too, as Python's traceback names the
    /// reader (#18): a player's two failures in the Bag said only `binary.rs`.
    pub fn of_error(error: &Error) -> Self {
        let mut detail = format!(
            "{}: {}\n  raised at {}:{}",
            error.kind.name(),
            error.message,
            error.location.file(),
            error.location.line()
        );
        if let Some(caller) = error.caller {
            detail += &format!("\n  called from {}:{}", caller.file(), caller.line());
        }
        Failure {
            fault: error.fault(),
            detail,
        }
    }

    /// A panic: a Rust reader's way of failing where Python's would have raised, as indexing past
    /// a list's end does.
    fn of_panic(payload: &(dyn Any + Send)) -> Self {
        let message = payload
            .downcast_ref::<&str>()
            .map(|text| text.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "a panic".to_string());
        let (file, line) = PANICKED_AT
            .with(|at| at.borrow().clone())
            .unwrap_or_default();
        let name = file.rsplit(['/', '\\']).next().unwrap_or("").to_string();
        Failure {
            fault: format!("panic at {name}:{line}"),
            detail: format!("panic: {message}\n  raised at {file}:{line}"),
        }
    }
}

thread_local! {
    /// Where the last panic on this thread came from, for the fault it becomes.
    static PANICKED_AT: RefCell<Option<(String, u32)>> = const { RefCell::new(None) };
}

/// Remember where each panic comes from, for `Failure::of_panic`, and say nothing of the ones the
/// runtime catches: a reader's failure is the log's to show (ADR-0019), and a tape's stop is not a
/// failure at all. Any other panic is shown as Rust shows it.
fn hook_panics() {
    let shown = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let at: Option<&Location<'_>> = info.location();
        PANICKED_AT.with(|cell| {
            *cell.borrow_mut() = at.map(|at| (at.file().to_string(), at.line()));
        });
        if CATCHING.with(Cell::get) || tape::is_stop(info.payload()) {
            return;
        }
        shown(info);
    }));
}

thread_local! {
    /// Whether the runtime is inside a poll, where a panic is a reader's failure and caught.
    static CATCHING: Cell<bool> = const { Cell::new(false) };
}

/// What a reader raising does: log it with a snapshot, and say so once (ADR-0019).
///
/// A player mid-battle cannot act on a traceback, but they can act on knowing this screen is
/// broken — silence that might be a crash is worse than two words. The traceback and the snapshot
/// are for the agent.
///
/// One bug is one fault, keyed by what was raised and the line that raised it, so a reader failing
/// every poll says nothing after the first time. A fault that goes away and comes back later is
/// said again: the same line failing on a screen the player has walked to since is news, and the
/// only "screen" the runtime can tell apart cheaply is one separated by a quiet spell.
pub struct Faults {
    speech: Rc<Speech>,
    log: Log,
    capture: Option<Box<dyn Fn() -> Result<Option<PathBuf>>>>,
    quiet: f64,
    left: Cell<i64>,
    /// Fault -> when it was last raised.
    seen: RefCell<IndexMap<String, f64>>,
}

impl Faults {
    /// Seconds a fault must stay away before it is worth saying again.
    pub const QUIET: f64 = 60.0;
    /// Snapshots one run writes. Enough to see the first faults, few enough that a run left going
    /// overnight on a broken screen cannot fill the disk.
    pub const CAPTURES: i64 = 10;

    pub fn new(
        speech: Rc<Speech>,
        log: Log,
        capture: Option<Box<dyn Fn() -> Result<Option<PathBuf>>>>,
        quiet: f64,
        captures: i64,
    ) -> Self {
        Faults {
            speech,
            log,
            capture,
            quiet,
            left: Cell::new(captures),
            seen: RefCell::new(IndexMap::new()),
        }
    }

    /// Take a reader's failure. Returns the fault, or None if it was already being said.
    pub fn caught(&self, failure: &Failure, now: f64) -> Option<String> {
        let last = self.seen.borrow_mut().insert(failure.fault.clone(), now);
        if let Some(last) = last
            && now - last < self.quiet
        {
            return None;
        }
        (self.log)(&format!(
            "a reader failed, carrying on:\n{}",
            failure.detail
        ));
        self.snapshot();
        self.speech
            .say(&own("app-reader-error", &[]), Category::System);
        Some(failure.fault.clone())
    }

    /// Capture what the readers were reading, for a test to replay the failure through.
    fn snapshot(&self) {
        let Some(capture) = &self.capture else { return };
        if self.left.get() <= 0 {
            (self.log)("no snapshot: this run has written all it will");
            return;
        }
        self.left.set(self.left.get() - 1);
        // The emulator may be the reason the reader failed at all.
        let taken = panic::catch_unwind(AssertUnwindSafe(capture));
        match taken {
            Ok(Ok(Some(written))) => {
                (self.log)(&format!("snapshot of the failure: {}", written.display()))
            }
            Ok(Ok(None)) => (self.log)("no snapshot of the failure: nothing had been read yet"),
            Ok(Err(failed)) => (self.log)(&format!("no snapshot of the failure: {failed:?}")),
            Err(payload) if tape::is_stop(&*payload) => panic::resume_unwind(payload),
            Err(payload) => (self.log)(&format!(
                "no snapshot of the failure: {}",
                Failure::of_panic(&*payload).detail
            )),
        }
    }
}

/// Announce what the game shows until `until()` is true, from one game to the next: a game the
/// emulator loses — closed, or another emulator started in its place — is waited for again.
pub fn run(
    rpc: Rc<dyn Rpc>,
    speech: Rc<Speech>,
    settings: Settings,
    until: &dyn Fn() -> bool,
    log: Log,
) -> Result<()> {
    run_with(rpc, speech, settings, until, log, &catalog::start)
}

/// How a game is started: `catalog::start`, or a test's own, where the Python's tests patched a
/// reader the catalog's profile builds.
pub type Start<'a> = &'a dyn Fn(
    catalog::Running,
    Rc<dyn Rpc>,
    Rc<Speech>,
    (i64, i64),
    Setup,
    &[u8],
) -> Result<Session>;

/// `data_path` as `run` checks it, which a test moves where Python's monkeypatch `app.data_path`.
fn data_folder() -> Result<PathBuf> {
    #[cfg(test)]
    if let Some(moved) = DATA_IN_TEST.with(|moved| moved.borrow().clone()) {
        return Ok(moved);
    }
    data_path()
}

#[cfg(test)]
thread_local! {
    /// Where a test says the data folder is, for `run`'s check alone.
    pub static DATA_IN_TEST: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// `run`, with the game started by `start`.
pub fn run_with(
    rpc: Rc<dyn Rpc>,
    speech: Rc<Speech>,
    settings: Settings,
    until: &dyn Fn() -> bool,
    log: Log,
    start: Start<'_>,
) -> Result<()> {
    hook_panics();
    let mut restarted = settings.restarted;
    let data = data_folder()?;
    if !data.is_dir() {
        // In English, and in code: the words that would say it in the player's language are in
        // the folder that is missing (ADR-0040).
        log(&format!(
            "no data folder at {}: pa3ds's own words, names and scenes are off",
            crate::py::path_str(&data)
        ));
        speech.say(MISSING_DATA, Category::Warning);
    }
    let mut language = settings.language.clone();
    loop {
        let Some(process) = wait_for_game(&*rpc, &speech, until, &log, &settings)? else {
            return Ok(());
        };
        let mut ended = session(
            rpc.clone(),
            speech.clone(),
            &settings,
            until,
            &log,
            process.clone(),
            start,
            restarted,
            &language,
            false,
        )?;
        while let Ended::Language(said) = ended {
            // The save says another language than the game was read in: read it again in that,
            // saying nothing of it (ADR-0040).
            language = said;
            ended = session(
                rpc.clone(),
                speech.clone(),
                &settings,
                until,
                &log,
                process.clone(),
                start,
                false,
                &language,
                true,
            )?;
        }
        if let Ended::Gone(false) = ended {
            return Ok(());
        }
        restarted = false;
        log("the game the emulator was running has gone; waiting for a game again");
    }
}

/// What reading one game came to, Python's `bool | str`.
enum Ended {
    /// Whether the game went away, for `run` to wait for the next one.
    Gone(bool),
    /// The language the save says, another than the game was read in, to read it again in.
    Language(String),
}

/// What one poll's failure is, the way Python's `except` clauses tell them apart.
enum Outcome {
    Done,
    Raised(Error),
    Panicked(Failure),
}

/// Run one poll's work, catching what Python's `except Exception` caught: an error returned, an
/// error a callback that cannot return one raised as a panic, or a reader's panic. A tape stopping
/// goes on unwinding, as Python's `BaseException` did.
fn attempt(work: impl FnOnce() -> Result<()>) -> Outcome {
    let was = CATCHING.with(|catching| catching.replace(true));
    let outcome = panic::catch_unwind(AssertUnwindSafe(work));
    CATCHING.with(|catching| catching.set(was));
    match outcome {
        Ok(Ok(())) => Outcome::Done,
        Ok(Err(error)) => Outcome::Raised(error),
        Err(payload) if tape::is_stop(&*payload) => panic::resume_unwind(payload),
        Err(payload) => match payload.downcast::<Error>() {
            Ok(error) => Outcome::Raised(*error),
            Err(payload) => Outcome::Panicked(Failure::of_panic(&*payload)),
        },
    }
}

/// Raise from inside a callback that cannot return an error, for `attempt` to catch as Python
/// caught the exception the callback raised.
fn raise(result: Result<()>) {
    if let Err(error) = result {
        panic::panic_any(error);
    }
}

/// Read one game in `language` until `until()` is true. Returns whether the game went away, for
/// `run` to wait for the next one, or the language its save says where that is another, to be read
/// again in. `restarted` says "pa3ds restarted" where "pa3ds ready" goes; `quiet` says neither,
/// for a game read again in its own language.
#[allow(clippy::too_many_arguments)]
fn session(
    rpc: Rc<dyn Rpc>,
    speech: Rc<Speech>,
    settings: &Settings,
    until: &dyn Fn() -> bool,
    log: &Log,
    process: Process,
    start: Start<'_>,
    restarted: bool,
    language: &str,
    quiet: bool,
) -> Result<Ended> {
    let began = time::monotonic();
    let code_spans = snapshot::regions(
        &*rpc,
        CODE_SPAN,
        snapshot::PROBE_STEP,
        snapshot::DEFAULT_GAP,
        None,
        None,
    )?;
    let Some(&code) = code_spans.first() else {
        log("the game's code is not readable");
        return Ok(Ended::Gone(false));
    };
    let image = snapshot::sweep(
        &*rpc,
        &[code],
        true,
        snapshot::CHUNK_PAGES,
        snapshot::CHUNK_PAUSE,
        None,
    )?
    .read(code.0, code.1 - code.0)?;
    // The title says which game is running, and its engine's package puts it together.
    let game = catalog::for_title(process.title_id);
    // The server logs from a thread of its own, where the runtime's log, an `Rc`, cannot go: its
    // lines come over a channel, into the log at each poll and once more after it closes.
    let (told, telling) = mpsc::channel::<String>();
    let touching = touch_server(settings, log, told)?.map(Rc::new);
    let hear_touch: Rc<dyn Fn()> = {
        let log = log.clone();
        Rc::new(move || {
            for line in telling.try_iter() {
                log(&line);
            }
        })
    };
    // pa3ds's own sounds, as loud as Azahar plays the game; none in a replay, which takes no keys.
    let audio = if settings.keys.is_some() {
        settings.audio.clone()
    } else {
        Audio {
            tones: 0.0,
            footsteps: 0.0,
            ticks: 0.0,
            feedback_sounds: 0.0,
        }
    };
    // `max(...)` of the four, keeping the first of equals as Python's does.
    let loudest = [audio.footsteps, audio.ticks, audio.feedback_sounds]
        .into_iter()
        .fold(
            audio.tones,
            |most, each| if each > most { each } else { most },
        );
    let loudness = if loudest > 0.0 {
        Some(Loudness::new(&match &settings.emulator {
            Some(emulator) => emulator.clone(),
            None => emulator_config("")?,
        }))
    } else {
        None
    };
    let session = start(
        game,
        rpc.clone(),
        speech.clone(),
        code,
        Setup {
            keys: settings
                .keys
                .clone()
                .unwrap_or_else(|| DEFAULT_BINDINGS.clone()),
            contexts: settings.contexts.clone(),
            ui: settings.ui.clone(),
            navigation: settings.navigation.clone(),
            touch: settings.touch.clone(),
            amie: settings.amie.clone(),
            extras: settings.extras.clone(),
            language: language.to_string(),
            names_home: settings.names_home.clone(),
            census: settings.census,
            ways: settings.ways.clone(),
            rom: settings.rom.clone(),
            emulator: settings.emulator.clone(),
            sweep_chunk: settings.sweep_chunk,
            touch_screen: touching
                .clone()
                .map(|touching| touching as Rc<dyn crate::core::touch::TouchScreen>),
            tones: match &loudness {
                Some(loudness) if audio.tones > 0.0 => {
                    Some(Rc::new(Tones::new(Some(loudness.clone()), audio.tones)))
                }
                _ => None,
            },
            ticks: match &loudness {
                Some(loudness) if audio.ticks > 0.0 => {
                    Some(Rc::new(Tones::new(Some(loudness.clone()), audio.ticks)))
                }
                _ => None,
            },
            clips: match &loudness {
                Some(loudness) if audio.footsteps > 0.0 => {
                    Some(Rc::new(Clips::new(loudness.clone(), audio.footsteps)))
                }
                _ => None,
            },
            feedback_sounds: match &loudness {
                Some(loudness) if audio.feedback_sounds > 0.0 => {
                    Some(Rc::new(Clips::new(loudness.clone(), audio.feedback_sounds)))
                }
                _ => None,
            },
            ..Setup::new(process.clone(), log.clone())
        },
        &image,
    )?;
    let profile = session.profile.clone();
    let heaps = session.heaps.clone();
    let heap_walk = session.walk.clone();
    let sweeping = Rc::new(Every::new(settings.whole_rescan));
    let looking = Every::new(settings.heap_tick);
    // Every object the last whole sweep found, and every one found in a new block since: what the
    // next whole sweep finds beyond them, the heaps' watch missed.
    let mut swept: HashSet<i64> = HashSet::new();
    // (when, block) for the blocks owed another sweep, soonest first.
    let mut again: Vec<(f64, Block)> = Vec::new();
    let screen = Rc::new(Every::new(settings.screen_tick));
    let busy = Rc::new(Busy::new(rpc.clone()));
    let hotkeys: Rc<RefCell<Option<Hotkeys>>> = Rc::new(RefCell::new(None));

    let poll_dialogue: Rc<dyn Fn() -> Result<()>> = {
        let profile = profile.clone();
        Rc::new(move || profile.poll_dialogue().map(|_| ()))
    };
    let answer_keys: Rc<dyn Fn() -> Result<()>> = {
        let (profile, hotkeys) = (profile.clone(), hotkeys.clone());
        let contexts = settings.contexts.clone();
        Rc::new(move || {
            let pressed = hotkeys.borrow().as_ref().map(Hotkeys::pressed);
            if let Some(pressed) = pressed {
                for key in pressed {
                    profile.press(&key)?;
                }
                // The screen's own keys are taken only while it runs: told every poll, so the hook
                // lets them through to Azahar the moment another screen comes.
                if let Some(hotkeys) = hotkeys.borrow().as_ref() {
                    hotkeys.set_contextual(context_keys(&contexts, profile.context()));
                }
                // A rename dialog answers on its own thread, so its answer is picked up here
                // rather than waited for: the game kept running while it was open.
                profile.poll_renames()?;
            }
            Ok(())
        })
    };
    let poll_screen: Rc<dyn Fn() -> Result<()>> = {
        let (profile, busy, screen, sweeping) = (
            profile.clone(),
            busy.clone(),
            screen.clone(),
            sweeping.clone(),
        );
        Rc::new(move || {
            let now = time::monotonic();
            if !busy.now(now) && screen.due(now) {
                profile.poll_screen()?;
                profile.poll_field()?;
                if profile.sweep_wanted()? {
                    sweeping.reset();
                }
            }
            Ok(())
        })
    };
    let between: Rc<dyn Fn()> = {
        let (poll_dialogue, answer_keys, poll_screen) = (
            poll_dialogue.clone(),
            answer_keys.clone(),
            poll_screen.clone(),
        );
        let walking = heap_walk.is_some();
        Rc::new(move || {
            raise(poll_dialogue());
            // Sun's sweep is a walk of its heaps, half a second on Route 1, and the navigation
            // keys, the menus' cursors and the field waited it out every 2 s (sun_4.ctm,
            // 2026-09-26). It reads nothing a poll reads by until it has finished, so a poll
            // halfway through reads what the last walk found, as it would just before this one.
            if walking {
                raise(answer_keys());
                raise(poll_screen());
            }
        })
    };

    // Sweeps read a chunk at a time and poll dialogue between chunks: read in one go, the heap
    // sweep held polling up for about 300 ms every couple of seconds (spike 59). In ORAS and X,
    // only dialogue. Screen text read halfway through a sweep sees what the last sweep found, and
    // the options screen's confirmation cursor was read as "FAST ON Yes", the rows behind it,
    // because the sweep that would find its list, whose labels keep a cursor over them quiet, had
    // not finished (menus.ctm replay, 2026-09-16).
    let classes: Vec<&str> = session.classes.iter().map(String::as_str).collect();
    let bases: Vec<&str> = session.bases.iter().map(String::as_str).collect();
    let mut discovery = Discovery::with(
        rpc.clone(),
        &image,
        code.0,
        &classes,
        between.clone(),
        settings.sweep_chunk,
        heap_walk.clone(),
        &bases,
    )?;
    let required: Vec<&str> = session.required.iter().map(String::as_str).collect();
    if discovery.missing(&required).len() == required.len() {
        log("no message window classes in this executable");
        return Ok(Ended::Gone(false));
    }
    log(&format!(
        "reading {} as {}: code {:#010x}-{:#010x}, ready in {:.1} s",
        process.name,
        game.name(),
        code.0,
        code.1,
        time::monotonic() - began
    ));
    if restarted {
        speech.say(&own("app-restarted", &[]), Category::Warning);
    } else if !quiet {
        speech.say(&own("app-ready", &[]), Category::System);
    }

    // The pages the readers had been reading, saved where raw captures live (ADR-0029).
    let capture_failure: Box<dyn Fn() -> Result<Option<PathBuf>>> = {
        let (pages, rpc, captures) = (
            session.pages.clone(),
            rpc.clone(),
            settings.captures.clone(),
        );
        Box::new(move || {
            let recent = pages
                .as_ref()
                .map(|pages| pages.recent())
                .unwrap_or_default();
            if recent.is_empty() {
                return Ok(None);
            }
            let spans: Vec<(i64, i64)> = recent.iter().map(|&base| (base, base + PAGE)).collect();
            let taken = snapshot::sweep(
                &*rpc,
                &spans,
                false,
                snapshot::CHUNK_PAGES,
                snapshot::CHUNK_PAUSE,
                None,
            )?;
            Ok(Some(taken.save("reader-error", captures.as_deref())?))
        })
    };

    *hotkeys.borrow_mut() = start_hotkeys(settings, log);
    let faults = Faults::new(
        speech.clone(),
        log.clone(),
        Some(capture_failure),
        Faults::QUIET,
        Faults::CAPTURES,
    );
    let mut last_error = String::new();
    // Whether the emulator has stopped answering. Nothing is read again until it says the game is
    // still the one selected: a new emulator on the same port has none selected, and a read with
    // none selected crashed it (2026-09-25, a session left running while the rig started another).
    // `rpc.unsure` catches the stalls a reader swallowed: the game restarting itself replaces the
    // RPC server just as a new emulator does (movie13, 2026-09-29).
    let mut silent = false;
    // Python's `finally`: the keyboard hook is let go and the touch server closed however the
    // session ends.
    struct Finally {
        hotkeys: Rc<RefCell<Option<Hotkeys>>>,
        touching: Option<Rc<TouchServer>>,
        hear_touch: Rc<dyn Fn()>,
    }
    impl Drop for Finally {
        fn drop(&mut self) {
            if let Some(hotkeys) = self.hotkeys.borrow().as_ref() {
                hotkeys.stop();
            }
            if let Some(touching) = &self.touching {
                touching.close();
                (self.hear_touch)();
            }
        }
    }
    let _finally = Finally {
        hotkeys: hotkeys.clone(),
        touching: touching.clone(),
        hear_touch: hear_touch.clone(),
    };

    while !until() {
        hear_touch();
        let started = time::monotonic();
        if silent || rpc.unsure() {
            match still_there(&*rpc)? {
                Some(false) => return Ok(Ended::Gone(true)),
                None => {
                    silent = true;
                    // `_session`'s sleeps end a tick of a tape's replay, which the Python's tells
                    // by its caller (`time::sleep_ending_tick`).
                    time::sleep_ending_tick(settings.wait);
                    continue;
                }
                Some(true) => {}
            }
            if silent {
                log("the emulator is answering again");
            }
            silent = false;
        }
        let mut reread: Option<String> = None;
        let outcome = attempt(|| {
            answer_keys()?;
            let app_started = profile.poll_app()?;
            if app_started && profile.app_sweep_wanted() {
                sweeping.soon(started + settings.app_sweep);
            }
            let whole = sweeping.due(started);
            // A class in a DLL has no vtable until the game loads it, and an app brings the
            // code it runs: the map blocks' is in the field's, and in Sun each app's classes
            // are in a module of its own, as the language choice's are.
            if app_started || whole || discovery.unsettled {
                let learned = discovery.learn(CODE_SPAN)?;
                if !learned.is_empty() {
                    log(&format!(
                        "classes loaded since the executable was read: {}",
                        learned.join(", ")
                    ));
                    // A module is learned a look after it loads, and the party's touch bar's
                    // was learned after the party's sweep: nothing found it for 30 s (movie23).
                    // Swept for as an app's are, once it has built them: at once, the touch bar
                    // was not built yet, and the app's own sweep went with it.
                    if !whole {
                        sweeping.soon(started + settings.app_sweep);
                    }
                }
            }
            if whole {
                discovery.measure_heap()?;
                let found = discovery.sweep()?;
                if !swept.is_empty() {
                    let missed: Vec<&String> = found
                        .heap
                        .iter()
                        .filter(|(at, _)| !swept.contains(at))
                        .map(|(_, name)| name)
                        .collect();
                    if !missed.is_empty() {
                        log(&format!(
                            "sweep: found as the heaps did not, {}",
                            most_common(&missed)
                                .iter()
                                .map(|(name, count)| format!("{count} {}", demangle(name)))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }
                swept = found.heap.keys().copied().collect();
                profile.track(&found)?;
                (session.adopt)(&found)?;
                if let Some(said) = (session.language)()
                    && said != language
                {
                    log(&format!(
                        "the save says the game is in {said}, not {language}: reading it again \
                         in that"
                    ));
                    reread = Some(said);
                    return Ok(());
                }
            }
            if !busy.now(started) && looking.due(started) {
                let mut blocks = memory::phase(&*session.watched, "heaps", || {
                    // Sun's looks read a header at a time, thousands as an area loads; ORAS's
                    // read cached pages, and polling inside their phase would share it.
                    heaps.borrow_mut().poll(if heap_walk.is_some() {
                        Some(between.clone())
                    } else {
                        None
                    })
                })?;
                (session.allocated)(&blocks);
                for block in &blocks {
                    for &delay in &settings.sweep_again {
                        again.push((started + delay, *block));
                    }
                }
                crate::py::sort_by_lt(&mut again, |a, b| a.0 < b.0);
                let due = again
                    .iter()
                    .position(|(when, _block)| *when > started)
                    .unwrap_or(again.len());
                blocks.extend(again.drain(..due).map(|(_when, block)| block));
                let spans: Vec<(i64, i64)> = blocks
                    .into_iter()
                    .filter_map(|block| (session.to_sweep)(block))
                    .collect();
                if !spans.is_empty() {
                    let found = discovery.sweep_spans(&spans)?;
                    swept.extend(found.heap.keys().copied());
                    profile.track(&found)?;
                }
            }
            poll_dialogue()?;
            profile.poll_footsteps()?;
            profile.poll_guidance()?;
            profile.poll_pointers()?;
            if profile.poll_menus(&mut discovery)? {
                screen.wait(time::monotonic());
            }
            poll_screen()
        });
        if let Some(said) = reread {
            return Ok(Ended::Language(said));
        }
        match outcome {
            Outcome::Done => {}
            Outcome::Raised(error) if error.is(Kind::RpcTimeoutError) => {
                log("the emulator stopped answering; retrying");
                sweeping.reset();
                silent = true;
            }
            Outcome::Raised(error) if error.is(Kind::RpcError) => {
                // A read of memory that is not there, through a pointer out of freed memory or a
                // torn read. One bad pointer must not silence pa3ds for the rest of the session: a
                // replay of menus.ctm stopped partway through on 2026-09-17. Logged once while it
                // repeats.
                if error.message != last_error {
                    log(&format!("a read failed, carrying on: {}", error.message));
                    last_error = error.message.clone();
                }
            }
            Outcome::Raised(error) => {
                // Anything else a reader raised. The rest of this poll is lost; the next one runs.
                // Not caught, before 2026-09-23, so one bad read of a shape a reader did not
                // expect took pa3ds down mid-game (ADR-0019).
                faults.caught(&Failure::of_error(&error), time::monotonic());
            }
            Outcome::Panicked(failure) => {
                faults.caught(&failure, time::monotonic());
            }
        }
        let left = settings.tick - (time::monotonic() - started);
        time::sleep_ending_tick(if left > 0.0 { left } else { 0.0 });
    }
    Ok(Ended::Gone(false))
}

/// `Counter(names).most_common()`: each name with how often it came, most first, a tie in the
/// order the names first came.
fn most_common(names: &[&String]) -> Vec<(String, i64)> {
    let mut counts: IndexMap<String, i64> = IndexMap::new();
    for &name in names {
        *counts.entry(name.clone()).or_insert(0) += 1;
    }
    let mut counted: Vec<(String, i64)> = counts.into_iter().collect();
    crate::py::sort_by_lt(&mut counted, |a, b| a.1 > b.1);
    counted
}

/// Whether the game is still the one the emulator has selected, or None while it does not
/// answer. A hiccup keeps the selection; a new emulator, or a state loaded, drops it, and the
/// game is then read afresh.
fn still_there(rpc: &dyn Rpc) -> Result<Option<bool>> {
    match rpc.confirm() {
        Ok(answer) => Ok(Some(answer)),
        Err(error) if error.is(Kind::RpcError) => Ok(None),
        Err(error) => Err(error),
    }
}

/// The server touch mode taps through, when keys are bound; none in a replay, and none, said
/// in the log, where its port is taken.
///
/// The server's log is `told`, a channel's end, since it logs from a thread of its own.
fn touch_server(
    settings: &Settings,
    log: &Log,
    told: mpsc::Sender<String>,
) -> Result<Option<TouchServer>> {
    if settings.keys.is_none() {
        return Ok(None);
    }
    let shared: SharedLog = Arc::new(move |line: &str| {
        // A line sent once the session has ended and dropped its end has no one to read it.
        let _ = told.send(line.to_string());
    });
    match TouchServer::new(shared, touch::PORT) {
        Ok(server) => Ok(Some(server)),
        Err(error) if error.is(Kind::OsError) => {
            log(&error.message);
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

/// The keyboard hook, when keys are bound and this is Windows; otherwise none, and said so.
fn start_hotkeys(settings: &Settings, log: &Log) -> Option<Hotkeys> {
    let keys = settings.keys.as_ref()?;
    if !cfg!(windows) {
        log("keys need Windows; none bound");
        return None;
    }
    let mut hotkeys = Hotkeys::of(bound_keys(keys), drag_keys(keys));
    match hotkeys.start() {
        Ok(error) if !error.is_empty() => {
            log(&format!("{error}; no keys bound"));
            return None;
        }
        Ok(_) => {}
        Err(error) => {
            log(&format!("{error}; no keys bound"));
            return None;
        }
    }
    for (mode, bound) in keys {
        log(&format!(
            "keys, {mode} mode: {}",
            bound
                .iter()
                .map(|(key, action)| format!("{key} {action}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    Some(hotkeys)
}

#[cfg(test)]
mod checks {
    use super::*;

    /// A short read is one fault for every reader, keyed by `core::binary`, and its log names the
    /// line that read it (#18): a player's Bag said only `binary.rs:31`, twice.
    #[test]
    fn a_short_reads_detail_names_the_line_that_read_it() {
        let error = crate::core::binary::u32(&[0; 2], 0).unwrap_err();
        let failure = Failure::of_error(&error);
        assert!(failure.fault.contains("binary.rs"), "{}", failure.fault);
        let called = failure.detail.lines().last().unwrap_or_default();
        assert!(
            called.starts_with("  called from") && called.contains("app.rs"),
            "{}",
            failure.detail
        );
        let other = crate::core::binary::u16(&[0; 1], 0).unwrap_err();
        assert_ne!(Failure::of_error(&other).detail, failure.detail);
        assert!(Failure::of_error(&Error::value("x")).detail.lines().count() == 2);
    }
}
