//! What the runtime needs of a game, and what a game is started with.
//!
//! The runtime (`app`) is the schedule: when to sweep, when to look at the heaps, when to poll which
//! reader. Everything it runs of a game comes from the game's package as a `Session` — the profile,
//! the classes a sweep looks for, and how the heaps are watched and a new block is swept — built by
//! that engine's `start` (`gen6.session`, `sm.session`) from a `Setup`. So the runtime names no
//! game's module, and a game with its own way of reading its heaps brings it with it.
//!
//! Bound by:
//! - ADR-0006: each engine puts its game together, one start an engine and not one a game.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::core::clips::Clips;
use crate::core::config::{Amie, Extras, Navigation, Touch, Ui};
use crate::core::discovery::{Discovery, Found, Walk};
use crate::core::heaps::{Block, Heaps};
use crate::core::memory::{Cache, PageCache};
use crate::core::rom::Rom;
use crate::core::rpc::Process;
use crate::core::tones::Tones;
use crate::core::touch::Drag;
use crate::core::{keyfile, rom};
use crate::games::common::field::keys::{Bindings, Contexts, DEFAULT_BINDINGS, DEFAULT_CONTEXTS};
use crate::py::{self, Error, Kind, Log, Result};

/// What the schedule asks of a game's readers, poll by poll (`gen6.profile`, `sm.profile`).
///
/// Every method takes `&self`, and an implementor keeps what it changes in cells, where Python's
/// profile is one object the runtime calls from several places at once: `poll_menus` sweeps, a
/// sweep calls back to the runtime's `between`, and that polls the dialogue and answers keys on
/// the same profile, from inside the call that began it.
pub trait Profile {
    fn track(&self, found: &Found) -> Result<()>;

    fn poll_app(&self) -> Result<bool>;

    fn app_sweep_wanted(&self) -> bool;

    fn poll_dialogue(&self) -> Result<bool>;

    fn poll_menus(&self, discovery: &mut Discovery) -> Result<bool>;

    fn poll_screen(&self) -> Result<()>;

    fn poll_field(&self) -> Result<()>;

    fn poll_guidance(&self) -> Result<()>;

    fn poll_footsteps(&self) -> Result<()>;

    fn poll_pointers(&self) -> Result<()>;

    fn poll_renames(&self) -> Result<()>;

    fn press(&self, key: &str) -> Result<()>;

    /// The screen running now that has keys of its own (`field.keys.DEFAULT_CONTEXTS`), or None:
    /// the keys the hook takes besides the modes'.
    fn context(&self) -> Option<&'static str>;

    fn sweep_wanted(&self) -> Result<bool>;
}

/// One game as the schedule runs it.
pub struct Session {
    pub profile: Rc<dyn Profile>,
    /// What a sweep looks for.
    pub classes: Vec<String>,
    /// Without one of these in the executable there is nothing to read: not this game, or not
    /// this version.
    pub required: Vec<String>,
    /// The heaps' watch: the blocks each heap has allocated since the last look (ADR-0034). It is
    /// shared with `adopt` and `allocated`, as the Python's one object is.
    pub heaps: Rc<RefCell<Heaps>>,
    /// What the heaps' watch reads through, in a phase of its own.
    pub watched: Rc<dyn Cache>,
    /// Has the heaps' watch take in the heaps a whole sweep found. What it returns in Python is
    /// not used.
    pub adopt: Rc<dyn Fn(&Found) -> Result<()>>,
    /// What of a new block to sweep, if any of it.
    pub to_sweep: Rc<dyn Fn(Block) -> Option<(i64, i64)>>,
    /// Takes in the blocks the watch saw allocated.
    pub allocated: Rc<dyn Fn(&[Block])>,
    /// How a whole sweep reads the heap where a game says where it may be read (Sun,
    /// ADR-0035); None for a sweep of the whole heap span.
    pub walk: Option<Walk>,
    /// Classes of `classes` found with every class built on them (`Discovery`).
    pub bases: Vec<String>,
    /// The page cache the readers read through, whose pages are saved when a reader fails
    /// (ADR-0019); None where they read to the byte.
    pub pages: Option<Rc<PageCache>>,
    /// The language the save says the game is in, as the last whole sweep read it, or None where
    /// it is not known: before the sweep, and in Sun, whose save pa3ds cannot reach (spike 226).
    /// The runtime reads the game again in it where it is not the session's.
    pub language: Rc<dyn Fn() -> Option<String>>,
}

/// What a game is started with, besides the emulator and speech: the process running, the log,
/// and the settings its readers read by (`app.Settings`).
///
/// Python's defaults are `DEFAULT_BINDINGS`, `Ui()`, `Navigation()`, `Extras()`, `"en"`, no
/// `names_home`, no `census`, no `rom`, no `emulator`, a `sweep_chunk` of 512, and no `tap`,
/// `tones` or `clips`:
/// [`Setup::new`] has them, and the rest is set with `Setup { language, ..Setup::new(process, log) }`.
#[derive(Clone)]
pub struct Setup {
    pub process: Process,
    pub log: Log,
    pub keys: Bindings,
    pub ui: Ui,
    pub navigation: Navigation,
    pub touch: Touch,
    pub amie: Amie,
    pub extras: Extras,
    pub language: String,
    pub names_home: Option<PathBuf>,
    /// Log touch mode's targets as they change (`app.Settings.census`).
    pub census: bool,
    pub rom: Option<PathBuf>,
    /// Azahar's `qt-config.ini`, where `rom_path` finds the ROMs it opened; None is AppData's.
    pub emulator: Option<PathBuf>,
    pub sweep_chunk: i64,
    /// Tap the touch screen at a pixel, saying why not where it cannot (`pa3ds.core.touch`); None
    /// where pa3ds takes no keys, as in a replay.
    pub tap: Option<Rc<dyn Fn(i64, i64) -> Option<String>>>,
    /// A press dragged along a path over so many seconds (`TouchServer.slide`).
    pub slide: Option<Rc<dyn Fn(&[(i64, i64)], f64) -> Option<String>>>,
    /// A press the player holds between keys (`TouchServer`).
    pub drag: Option<Rc<dyn Drag>>,
    /// The tones that say where a cursor is (`pa3ds.core.tones`); None where pa3ds takes no keys,
    /// so a replay makes no sound, or where `[pa3ds.audio] tones` is 0.
    pub tones: Option<Rc<Tones>>,
    /// The tone at each tick of a game's clock (`ScreenAnnouncer`); None as `tones` is, or where
    /// `[pa3ds.audio] ticks` is 0, as it ships.
    pub ticks: Option<Rc<Tones>>,
    /// Where the game's own sounds pa3ds plays go, its footsteps (`pa3ds.core.clips`); None where
    /// pa3ds takes no keys, as in a replay, or where `[pa3ds.audio] footsteps` is 0.
    pub clips: Option<Rc<Clips>>,
    /// Where the game's own sounds for what a key has done go, a drag picked up and let go of and
    /// a key with nothing to do; None as `clips` is, or where `[pa3ds.audio] feedback_sounds` is
    /// 0, and they are said in words.
    pub feedback_sounds: Option<Rc<Clips>>,
    /// A screen's own keys, which win over the modes' while it runs (`Profile.context`).
    pub contexts: Contexts,
}

impl Setup {
    pub fn new(process: Process, log: Log) -> Self {
        Setup {
            process,
            log,
            keys: DEFAULT_BINDINGS.clone(),
            ui: Ui::default(),
            navigation: Navigation::default(),
            touch: Touch::default(),
            amie: Amie::default(),
            extras: Extras::default(),
            language: "en".to_string(),
            names_home: None,
            census: false,
            rom: None,
            emulator: None,
            sweep_chunk: 512,
            tap: None,
            slide: None,
            drag: None,
            tones: None,
            ticks: None,
            clips: None,
            feedback_sounds: None,
            contexts: DEFAULT_CONTEXTS.clone(),
        }
    }

    /// The player's ROM of the game running: `rom`, or the one Azahar has opened. Where
    /// none is, `notes` has which `qt-config.ini` was read and why each ROM it lists is not.
    pub fn rom_path(&self, mut notes: Option<&mut Vec<String>>) -> Result<Option<PathBuf>> {
        if let Some(rom) = &self.rom {
            return Ok(Some(rom.clone()));
        }
        let path = match &self.emulator {
            Some(emulator) => emulator.clone(),
            None => keyfile::emulator_config("")?,
        };
        // `read_text(encoding="utf-8", errors="replace")`, which also turns a CRLF into a LF:
        // `azahar_roms` splits the text into lines, so the difference is never seen.
        let config = match std::fs::read(&path) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(error) => {
                if let Some(notes) = notes.as_deref_mut() {
                    notes.push(format!(
                        "{} could not be read: {}",
                        py::path_str(&path),
                        keyfile::os_error(&error, &path, true)
                    ));
                }
                String::new()
            }
        };
        let candidates = rom::azahar_roms(&config);
        if let Some(notes) = notes.as_deref_mut()
            && candidates.is_empty()
        {
            notes.push(format!("{} lists no ROM", py::path_str(&path)));
        }
        rom::find(self.process.title_id, candidates, notes)
    }

    /// What `load` reads out of the player's ROM in their language, or None; the log says
    /// which, and why, and `missing` what is lost without it. The ROM is `rom`, or the one
    /// Azahar has opened that is the game running.
    pub fn words<T>(
        &self,
        load: impl FnOnce(&Rom, &str) -> Result<T>,
        missing: &str,
    ) -> Result<Option<T>> {
        let mut notes: Vec<String> = Vec::new();
        let Some(path) = self.rom_path(Some(&mut notes))? else {
            (self.log)(&format!(
                "no decrypted ROM of title {:016x} among those Azahar has opened; {missing}",
                self.process.title_id
            ));
            for note in &notes {
                (self.log)(note);
            }
            return Ok(None);
        };
        // Python's `with rom.Rom(path) as image:`: the image is closed as the block ends.
        let words = (|| {
            let image = Rom::new(&path)?;
            if image.title_id != self.process.title_id {
                return Err(Error::new(
                    Kind::RomError,
                    format!(
                        "it is title {:016x}, and the game running is {:016x}",
                        image.title_id, self.process.title_id
                    ),
                ));
            }
            load(&image, &self.language)
        })();
        let words = match words {
            Ok(words) => words,
            Err(error) if error.is(Kind::RomError) || error.is(Kind::OsError) => {
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                (self.log)(&format!("the game's own words are off, {name}: {error}"));
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        (self.log)(&format!(
            "the game's own words from {} ({})",
            py::path_str(&path),
            self.language
        ));
        Ok(Some(words))
    }
}
