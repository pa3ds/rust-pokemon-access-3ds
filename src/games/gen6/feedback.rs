//! The game's own sounds for what a key or a moment has done, out of the player's ROM: touch
//! mode's drag picked up and let go of, a key of pa3ds's with nothing to do, Head It's cue, a hit
//! landing on Super Training's bag and the bag finished, a Balloon Bot's shot and the moment to
//! block it, and the Pokémon's own cries in Amie's games. They play where `[pa3ds.audio]
//! feedback_sounds` is above 0, as loud as the game would play them (`core.clips`); where one
//! cannot be played, a key's line and a round's cue are said in words instead, and the rest go
//! unheard.
//!
//! Their names, their loading as a game starts (`load`), and one object the subscreens and touch
//! mode ask (`FeedbackSounds`): `play` a sound alone, `heard` a key's line as its sound, `cue` Head
//! It's cue toward its Pokémon, `round_cue` a round's cue as `[pa3ds.training]` asks, `dragged`
//! touch mode's drag, and `cry` a Pokémon's own cry toward its side.
//!
//! Bound by:
//! - ADR-0025: the game's own sounds as loud as the game would play them, in both ears but for Head
//!   It's cue and the cries, toward their Pokémon.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::bcsar::{self, Sound};
use crate::core::clips::Clips;
use crate::core::config::{CUE_AS_SOUND, CUE_OFF};
use crate::core::rom::Rom;
use crate::core::speech::{Category, Speech};
use crate::games::common::pokemon::Pokemon;
use crate::games::common::session::Setup;
use crate::games::gen6::rom::cries::{Crier, Cries};
use crate::games::gen6::rom::layout::RomLayout;
use crate::py::{self, Kind, Log, Result};

pub const PICK_SOUND: &str = "SEQ_SE_CATCH";
/// The PC Box's sounds for picking up a Pokémon and putting it down, the game's own for moving
/// something somewhere, played as touch mode picks up a drag and lets go of it in place of
/// "holding" and "let go on": a tenth and an eighth of a second, the same in
/// Omega Ruby and X. Amie's touch sounds, `SEQ_SE_KW_TOUCH` and `SEQ_SE_KW_TOUCH2`, until then.
pub const DROP_SOUND: &str = "SEQ_SE_RELEASE";
/// The game's own sound for a press that does nothing, played for a key of pa3ds's with nothing to
/// do in place of saying why: the beep of Omega Ruby's menus, its
/// `SEQ_SE_SYS_015` too, the same wave 271 samples longer; X's `SEQ_SE_BEEP` is that same wave at
/// the same volume, where X's `SEQ_SE_SYS_015` is another sound, 0.93 s long.
pub const WRONG_SOUND: &str = "SEQ_SE_BEEP";
/// The "!" the field plays over someone's head, played as Head It's cue (in place of a
/// tone): 0.3 s, the same wave and volume in Omega Ruby and X, where `SEQ_SE_FLD_277` is
/// that wave too (spike 274).
pub const CUE_SOUND: &str = "SEQ_SE_FLD_036";
/// Played as a Balloon Bot fires: 0.23 s, the same wave and volume in Omega
/// Ruby, Alpha Sapphire and X.
pub const ALERT_SOUND: &str = "SEQ_SE_KW_ALERT";
/// Played as a hit lands on Super Training's bag, of which the game gives no sound:
/// 0.34 s, the same wave and volume in Omega Ruby, Alpha Sapphire and X.
pub const HIT_SOUND: &str = "SEQ_SE_SP_017";
/// Every one of them, as they are read out of the ROM.
pub const SOUNDS: [&str; 6] = [
    PICK_SOUND,
    DROP_SOUND,
    WRONG_SOUND,
    CUE_SOUND,
    HIT_SOUND,
    ALERT_SOUND,
];

/// What the feedback sounds play on: `Clips`, or a test's stand-in, where Python's test hands a
/// namespace with its `names` and `play`.
pub trait Plays {
    fn names(&self) -> HashSet<String>;
    /// `play(name, across)`.
    fn play(&self, name: &str, across: f64) -> Result<()>;
}

impl Plays for Clips {
    fn names(&self) -> HashSet<String> {
        Clips::names(self)
    }

    fn play(&self, name: &str, across: f64) -> Result<()> {
        self.play_across(name, across)
    }
}

/// The game's feedback sounds as a game plays them, or the words in their place.
pub struct FeedbackSounds {
    log: Log,
    speech: Rc<Speech>,
    /// Where they play; None where `[pa3ds.audio] feedback_sounds` is 0, or pa3ds takes no
    /// keys, as in a replay.
    clips: Option<Rc<dyn Plays>>,
    /// Plays a Pokémon's own cry toward a side, 0 the left and 1 the right, and says whether
    /// it played (`load`); None where the game's cries are not read. Which game plays one is
    /// the subscreens', by its style.
    pub cry: Option<Crier>,
}

impl FeedbackSounds {
    /// Python's `clips` and `cry` default to None.
    pub fn new(
        log: Log,
        speech: Rc<Speech>,
        clips: Option<Rc<dyn Plays>>,
        cry: Option<Crier>,
    ) -> Self {
        FeedbackSounds {
            log,
            speech,
            clips,
            cry,
        }
    }

    /// Whether each of `sounds` can be played.
    pub fn has(&self, sounds: &[&str]) -> bool {
        match &self.clips {
            None => false,
            Some(clips) => {
                let names = clips.names();
                sounds.iter().all(|sound| names.contains(*sound))
            }
        }
    }

    /// `sound` toward `across` (`clips.stereo`), and nothing where it cannot be played. Python's
    /// `across` defaults to 0.5.
    pub fn play(&self, sound: &str, across: f64) {
        if let Some(clips) = &self.clips {
            // The sound thread not starting, raised to the caller in Python.
            let _ = clips.play(sound, across);
        }
    }

    /// A screen's own key's line heard as the game's `sound`, as the drag's own sounds are and
    /// the game's error sound, and logged as a key's line is: "" then, and the line to say where
    /// the sound cannot be played.
    pub fn heard(&self, sound: &str, said: &str) -> String {
        if self.has(&[sound]) {
            (self.log)(&format!("key: {said}, heard as {sound}"));
            self.play(sound, 0.5);
            return String::new();
        }
        said.to_string()
    }

    /// Head It's cue, in the ear of the Pokémon's side, the middle's in both (ADR-0025 as
    /// amended); logged so a round's log shows each beside the key pressed for it.
    pub fn cue(&self, column: i64) {
        (self.log)(&format!(
            "head it: cue, {}",
            ["left", "middle", "right"][column as usize]
        ));
        self.play(CUE_SOUND, column as f64 / 2.0);
    }

    /// A cue of a Super Training round, as `[pa3ds.training]` asks: the game's sound, or the
    /// words where it asks for them or the sound cannot be played; logged either way.
    pub fn round_cue(&self, mode: &str, sound: &str, said: &str) {
        if mode == CUE_OFF {
            return;
        }
        if mode == CUE_AS_SOUND && self.has(&[sound]) {
            (self.log)(&format!("round: {said}, heard as {sound}"));
            self.play(sound, 0.5);
            return;
        }
        (self.log)(&format!("round: {said}"));
        self.speech.say(said, Category::Warning);
    }

    /// Touch mode's sound for picking up a drag, with True, and letting go of it, the PC Box's
    /// (`PICK_SOUND`); None where either is missing, and the drag is said in words.
    pub fn dragged(self: &Rc<Self>) -> Option<Rc<dyn Fn(bool)>> {
        if !self.has(&[PICK_SOUND, DROP_SOUND]) {
            return None;
        }
        let me = self.clone();
        Some(Rc::new(move |up: bool| {
            me.play(if up { PICK_SOUND } else { DROP_SOUND }, 0.5)
        }))
    }
}

/// The feedback sounds of the game starting: its sounds read into `setup.feedback_sounds`
/// (`sounds`), and its cries read as each is first wanted into `setup.cries` (`cry_player`).
pub fn load(
    setup: &Setup,
    layout: &RomLayout,
    image: &[u8],
    speech: Rc<Speech>,
) -> Result<FeedbackSounds> {
    let feedback = setup.feedback_sounds.clone();
    if let Some(feedback) = &feedback {
        for (name, sound) in sounds(setup, layout)? {
            feedback.add(&name, sound);
        }
    }
    Ok(FeedbackSounds::new(
        setup.log.clone(),
        speech,
        feedback.map(|feedback| feedback as Rc<dyn Plays>),
        cry_player(setup, layout, image)?,
    ))
}

/// The game's own sounds for what a key has done, out of the player's ROM: the PC Box's
/// touch mode picks up and lets go of a drag with (`PICK_SOUND`), the game's error
/// sound (`WRONG_SOUND`), and the "!" of Head It's cue (`CUE_SOUND`); none, and
/// the log says why, where it cannot be read, and each but the cue is said in words.
pub fn sounds(setup: &Setup, layout: &RomLayout) -> Result<IndexMap<String, Sound>> {
    let rom = setup.rom_path(None)?;
    let (Some(archive), Some(rom)) = (layout.sounds.as_deref(), rom) else {
        return Ok(IndexMap::new());
    };
    if archive.is_empty() {
        return Ok(IndexMap::new());
    }
    let read = Rom::new(&rom).and_then(|image| bcsar::sounds(&image, archive, SOUNDS));
    match read {
        Ok(sounds) => Ok(sounds),
        Err(error)
            if [
                Kind::RomError,
                Kind::OsError,
                Kind::StructError,
                Kind::KeyError,
                Kind::IndexError,
            ]
            .iter()
            .any(|&kind| error.is(kind)) =>
        {
            let name = rom
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            (setup.log)(&format!(
                "feedback sounds: the game's own are off, {name}: {}",
                error.message
            ));
            Ok(IndexMap::new())
        }
        Err(error) => Err(error),
    }
}

/// Plays a Pokémon's own cry toward a side, 0 the left and 1 the right, over the other sides'
/// cries, and says whether it played: read out of the player's ROM the first time each is wanted
/// and kept (`rom.cries`).
/// None where cries are off or the game's are not read; once the ROM fails, none plays, and
/// the log says why.
pub fn cry_player(setup: &Setup, layout: &RomLayout, image: &[u8]) -> Result<Option<Crier>> {
    let (clips, rom) = (setup.cries.clone(), setup.rom_path(None)?);
    let (Some(clips), Some(archive), Some(rom)) = (clips, layout.cries.as_deref(), rom) else {
        return Ok(None);
    };
    if image.is_empty() {
        return Ok(None);
    }
    let cries = RefCell::new(Cries::new(&rom, archive, image));
    let failed: RefCell<Vec<String>> = RefCell::new(Vec::new());
    let log = setup.log.clone();
    let rom_name = rom
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(Some(Rc::new(move |found: &Pokemon, across: f64| {
        let name = format!("{}/{}", found.species, found.form);
        if !clips.names().contains(&name) {
            if !failed.borrow().is_empty() {
                return Ok(false);
            }
            let read = cries.borrow_mut().cry(found.species, found.form);
            let sound = match read {
                Ok(sound) => sound,
                Err(error)
                    if [
                        Kind::RomError,
                        Kind::OsError,
                        Kind::StructError,
                        Kind::IndexError,
                    ]
                    .iter()
                    .any(|&kind| error.is(kind)) =>
                {
                    failed.borrow_mut().push(error.message.clone());
                    log(&format!("cries: off, {rom_name}: {error}"));
                    return Ok(false);
                }
                Err(error) => return Err(error),
            };
            let Some(sound) = sound else {
                return Ok(false);
            };
            // ponytail: read as it is first wanted, 10 to 45 ms; read at the lineup if it shows.
            clips.add(&name, sound);
        }
        log(&format!("cry: {name} toward {}", py::fixed(across, 1)));
        // A voice a side, so a cry cuts off only its own Pokémon's last.
        clips.play_on(&name, across, py::round(across * 2.0)?)?;
        Ok(true)
    })))
}
