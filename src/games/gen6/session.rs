//! A gen 6 game — ORAS, X or Y — put together for the runtime: its readers, and its heaps watched
//! for what they allocate (spike 110).
//!
//! Bound by:
//! - ADR-0025: the game's own sounds pa3ds plays, its footsteps, the keys' sounds and the Pokémon's
//!   cries, as loud as the game would play them; the cries panned toward their Pokémon.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::bcsar::{self, Sound};
use crate::core::clips::sounds;
use crate::core::discovery::{Found, HEAP_SPAN};
use crate::core::heaps::{Block, Heaps};
use crate::core::memory::{Cache, PageCache};
use crate::core::names::Names;
use crate::core::rom::Rom;
use crate::core::rpc::Rpc;
use crate::core::rtti::ClassNames;
use crate::core::speech::Speech;
use crate::core::window;
use crate::core::words::{self, Words};
use crate::games::common::pokemon::Pokemon;
use crate::games::common::session::{Profile, Session, Setup};
use crate::games::gen6::field::footsteps;
use crate::games::gen6::game::Game;
use crate::games::gen6::profile::{
    CUE_SOUND, DROP_SOUND, Gen6Profile, Options, PICK_SOUND, REQUIRED, WRONG_SOUND,
};
use crate::games::gen6::rom::archives::{GameText, no_text};
use crate::games::gen6::rom::cries::{Crier, Cries};
use crate::games::gen6::rom::layout::RomLayout;
use crate::games::{gen6::gflheap, gen6::rom::textures, gen6::screens::buttons};
use crate::py::{self, Kind, Result};

/// `game` read through a page cache, every poll in a phase of its own: one pipelined fetch of
/// the pages it read last time, and every read inside a page answered from it (spike 62).
/// `image` is the executable as read from `code`, where the game keeps tables no ROM file has.
pub fn start(
    game: &'static Game,
    rpc: Rc<dyn Rpc>,
    speech: Rc<Speech>,
    code: (i64, i64),
    setup: Setup,
    image: &[u8],
) -> Result<Session> {
    let memory = Rc::new(PageCache::new(rpc.clone()));
    let language = if setup.tables.ui.language.is_empty() {
        &setup.language
    } else {
        &setup.tables.ui.language
    };
    words::r#use(Some(Rc::new(Words::new(
        language,
        Some(&(game.data)()),
        IndexMap::new(),
        setup.log.clone(),
    )?)));
    // The game's own words come from the game; these are for what it never names at all.
    let title = window::game_title(window::EMULATOR);
    let named = Rc::new(Names::load(
        &setup.process.name,
        setup.names_home.as_deref(),
        &title,
        None,
    )?);
    let text = Rc::new(game_text_with(&setup, &game.rom, &game.teleporting)?);
    let mut pictures = None;
    if !game.rom.textures.is_empty()
        && let Some(home) = &setup.names_home
    {
        let cache = home
            .join("cache")
            .join("textures")
            .join(format!("{}.toml", setup.process.name));
        pictures = textures::start(
            &cache,
            setup.rom_path(None)?.as_deref(),
            &game.rom.textures,
            &title,
            &setup.process.name,
            text.clone(),
            &*setup.log,
        )?
        .map(Rc::new);
    }
    let mut clips = setup.clips.clone();
    let (mut found, mut swims) = (IndexMap::new(), IndexMap::new());
    if let (Some(playing), Some(archive)) = (&clips, &game.rom.sounds)
        && !archive.is_empty()
    {
        (found, swims) = footsteps::load(
            footsteps::FILE,
            footsteps::shipped(&(game.data)()),
            setup.rom_path(None)?.as_deref(),
            archive,
            &sounds()?,
            &setup.log,
        )?;
        for (name, sound) in &found {
            playing.add(name, sound.clone());
        }
    }
    if found.is_empty() {
        clips = None;
    }
    let feedback = setup.feedback_sounds.clone();
    if let Some(feedback) = &feedback {
        for (name, sound) in feedback_sounds(&setup, &game.rom)? {
            feedback.add(&name, sound);
        }
    }
    let profile = Rc::new(Gen6Profile::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), code)),
        speech,
        setup.log.clone(),
        game,
        Options {
            named: Some(named),
            tables: Some(setup.tables.clone()),
            keys: Some(setup.keys.clone()),
            contexts: Some(setup.contexts.clone()),
            text,
            pictures,
            image: image.to_vec(),
            touch_screen: setup.touch_screen.clone(),
            tones: setup.tones.clone(),
            ticks: setup.ticks.clone(),
            clips,
            feedback_sounds: feedback,
            swims: Some(swims),
            cry: cry_player(&setup, &game.rom, image)?,
            title: Some(setup.process.title_id),
            census: setup.census,
            ways: setup.ways.clone(),
            reads_beside_gpu: rpc.reads_beside_gpu(),
        },
    ));
    // What the game allocates is found where it was allocated, the look after (spike 110).
    let read_each = {
        let memory = memory.clone();
        Rc::new(move |reads: &[(i64, i64)]| memory.read_each(reads))
    };
    let heaps = Rc::new(RefCell::new(Heaps::with(
        memory.clone(),
        gflheap::HEAD,
        HEAP_SPAN,
        Some(gflheap::COUNT),
        None,
        Rc::new(|_heap| false),
        Some(read_each),
    )));
    let adopting = heaps.clone();
    let saved = profile.clone();
    Ok(Session {
        classes: profile.classes.clone(),
        profile: profile as Rc<dyn Profile>,
        required: REQUIRED.iter().map(|class| class.to_string()).collect(),
        heaps,
        watched: memory.clone() as Rc<dyn Cache>,
        adopt: Rc::new(move |found: &Found| {
            adopting
                .borrow_mut()
                .adopt(&found.of(&[gflheap::CLASS]), false)?;
            Ok(())
        }),
        to_sweep: Rc::new(|block: Block| Some(gflheap::to_sweep(block))),
        allocated: Rc::new(|_blocks: &[Block]| {}),
        walk: None,
        bases: vec![buttons::BAR_CLASS.to_string()],
        pages: Some(memory),
        language: Rc::new(move || saved.save_language.borrow().clone()),
    })
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

/// `game_text(setup, layout)`: no zone's teleporters.
pub fn game_text(setup: &Setup, layout: &RomLayout) -> Result<GameText> {
    game_text_with(setup, layout, &IndexMap::new())
}

/// The game's own words out of the player's ROM, or `NO_TEXT`, with where the teleporters of
/// the zones `teleporting` lists put the player; the log says which, and why.
///
/// The game's own sounds for what a key has done, out of the player's ROM: the PC Box's touch
/// mode picks up and lets go of a drag with (`profile.PICK_SOUND`), the game's error sound
/// (`profile.WRONG_SOUND`), and the "!" of Head It's cue (`profile.CUE_SOUND`); none, and the log
/// says why, where it cannot be read, and each but the cue is said in words.
pub fn feedback_sounds(setup: &Setup, layout: &RomLayout) -> Result<IndexMap<String, Sound>> {
    let rom = setup.rom_path(None)?;
    let (Some(archive), Some(rom)) = (layout.sounds.as_deref(), rom) else {
        return Ok(IndexMap::new());
    };
    if archive.is_empty() {
        return Ok(IndexMap::new());
    }
    let read = Rom::new(&rom).and_then(|image| {
        bcsar::sounds(
            &image,
            archive,
            [PICK_SOUND, DROP_SOUND, WRONG_SOUND, CUE_SOUND],
        )
    });
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

/// Not having them costs a place's name said a moment early and the rival's name, both of which
/// have a fallback — AreaNav's words and "Rival" — so a ROM that cannot be read is logged, not
/// spoken at every start.
pub fn game_text_with(
    setup: &Setup,
    layout: &RomLayout,
    teleporting: &IndexMap<i64, i64>,
) -> Result<GameText> {
    Ok(setup
        .words(
            |image, language| GameText::load_with(image, language, layout, teleporting),
            "the game's own words for places and the rival are off",
        )?
        .unwrap_or_else(no_text))
}
