//! A gen 6 game — ORAS, X or Y — put together for the runtime: its readers, and its heaps watched
//! for what they allocate (spike 110).

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
use crate::games::common::session::{Profile, Session, Setup};
use crate::games::gen6::field::footsteps;
use crate::games::gen6::game::Game;
use crate::games::gen6::profile::{
    DROP_SOUND, Gen6Profile, Options, PICK_SOUND, REQUIRED, WRONG_SOUND,
};
use crate::games::gen6::rom::archives::{GameText, no_text};
use crate::games::gen6::rom::layout::RomLayout;
use crate::games::{gen6::gflheap, gen6::rom::textures, gen6::screens::buttons};
use crate::py::{Kind, Result};

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
    let memory = Rc::new(PageCache::new(rpc));
    let language = if setup.ui.language.is_empty() {
        &setup.language
    } else {
        &setup.ui.language
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
            ui: Some(setup.ui.clone()),
            navigation: Some(setup.navigation.clone()),
            extras: Some(setup.extras.clone()),
            keys: Some(setup.keys.clone()),
            contexts: Some(setup.contexts.clone()),
            text,
            pictures,
            image: image.to_vec(),
            tap: setup.tap.clone(),
            slide: setup.slide.clone(),
            drag: setup.drag.clone(),
            tones: setup.tones.clone(),
            ticks: setup.ticks.clone(),
            clips,
            feedback_sounds: feedback,
            swims: Some(swims),
            title: Some(setup.process.title_id),
            census: setup.census,
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

/// `game_text(setup, layout)`: no zone's teleporters.
pub fn game_text(setup: &Setup, layout: &RomLayout) -> Result<GameText> {
    game_text_with(setup, layout, &IndexMap::new())
}

/// The game's own words out of the player's ROM, or `NO_TEXT`, with where the teleporters of
/// the zones `teleporting` lists put the player; the log says which, and why.
///
/// The game's own sounds for what a key has done, out of the player's ROM: the PC Box's touch
/// mode picks up and lets go of a drag with (`profile.PICK_SOUND`), and the game's error sound
/// (`profile.WRONG_SOUND`); none, and the log says why, where it cannot be read, and each is said
/// in words.
pub fn feedback_sounds(setup: &Setup, layout: &RomLayout) -> Result<IndexMap<String, Sound>> {
    let rom = setup.rom_path(None)?;
    let (Some(archive), Some(rom)) = (layout.sounds.as_deref(), rom) else {
        return Ok(IndexMap::new());
    };
    if archive.is_empty() {
        return Ok(IndexMap::new());
    }
    let read = Rom::new(&rom)
        .and_then(|image| bcsar::sounds(&image, archive, [PICK_SOUND, DROP_SOUND, WRONG_SOUND]));
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
