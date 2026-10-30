//! Sun and Moon put together for the runtime: its readers, read to the byte and only in the small
//! allocations (ADR-0035), its heaps walked for a whole sweep and watched between walks (spike 123),
//! and its own words out of the player's ROM.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::discovery::{CODE_SPAN, Walk};
use crate::core::heaps::{Block, Heaps};
use crate::core::memory::{Cache, ExactCache, Memory};
use crate::core::names::Names;
use crate::core::rpc::Rpc;
use crate::core::rtti::ClassNames;
use crate::core::speech::Speech;
use crate::core::window;
use crate::core::words::{self, Words};
use crate::games::common::scenes;
use crate::games::common::session::{Profile, Session, Setup};
use crate::games::sm::heaps::Blocks;
use crate::games::sm::profile::{Options, REQUIRED, SunProfile, heap_classes};
use crate::games::sm::text::SunText;
use crate::games::sm::walls::SunWalls;
use crate::games::sm::zones::SunZones;
use crate::games::sm::{apps, field, heaps, langselect, lyt};
use crate::py::{PySet, Result};

/// Sun, read to the byte, never a page around what is asked for: in Sun the byte after an
/// object may be the GPU's, and reading that took the emulator down (`games.sm.heaps`). A poll's
/// reads are fetched together at the next poll of its kind, kept to the small blocks that may be
/// read now.
pub fn start(rpc: Rc<dyn Rpc>, speech: Rc<Speech>, setup: Setup) -> Result<Session> {
    let setup = if setup.tables.ui.language.is_empty() {
        setup
    } else {
        Setup {
            language: setup.tables.ui.language.clone(),
            ..setup
        }
    };
    words::r#use(Some(Rc::new(Words::new(
        &setup.language,
        Some(&field::names()),
        IndexMap::new(),
        setup.log.clone(),
    )?)));
    let live = Rc::new(heaps::SmallBlocks::new());
    let exact = Rc::new(ExactCache::new(
        rpc.clone(),
        Some({
            let live = live.clone();
            Rc::new(move |address, size| live.may_read(address, size))
        }),
    ));
    // The whole span code can be in, not what the executable was at the start: an app's classes
    // are in a module it loads later, as the language choice's are (0x006b2700).
    let names = Rc::new(ClassNames::new(exact.clone(), CODE_SPAN));
    let text = setup
        .words(
            |image, language| {
                SunText::load(
                    image,
                    language,
                    &[langselect::TEXT_FILE, lyt::LEVEL_WORDS.0],
                )
            },
            "the menus' words are off",
        )?
        .map(Rc::new);
    let zones = setup
        .words(
            SunZones::load,
            "places go unsaid and exits lead nowhere named",
        )?
        .map(Rc::new);
    let walls = setup
        .words(
            |image, _language| SunWalls::from_rom(image, field::CELL),
            "P finds no way",
        )?
        .map(Rc::new);
    let named = Rc::new(Names::load(
        &setup.process.name,
        setup.names_home.as_deref(),
        &window::game_title(window::EMULATOR),
        None,
    )?);
    // The field's move models are in a heap of their own, beside the dialogue's, and the field
    // reads its pointers only inside a small block the last walk found (spike 113). The running
    // process is in heap 6, two small blocks (`sm.apps`, spike 116).
    //
    // `DIALOGUE_HEAPS | field.HEAPS | {apps.HEAP}` is a set, read in the order Python iterates it.
    let wanted: Vec<i64> = PySet::of(heaps::DIALOGUE_HEAPS)
        .union(&PySet::of(field::HEAPS))
        .union(&PySet::of([apps::HEAP]))
        .iter()
        .collect();
    let walker = heaps::Heaps::with(
        rpc.clone(),
        names.clone(),
        setup.log.clone(),
        &wanted,
        &heaps::UNREAD,
    );
    let roots: Rc<RefCell<Vec<i64>>> = Rc::new(RefCell::new(Vec::new()));

    // Watched as ORAS's are, the look after an allocation, keeping to the heaps under the walk's
    // and to their small blocks, read to the byte: its 170 heads, a read each, in one pipelined
    // burst (spike 123). Made ahead of `walk`, which forgets in it; nothing is read in making it.
    let watched = Rc::new(ExactCache::new(rpc.clone(), None));
    let watch = Rc::new(RefCell::new(Heaps::with(
        watched.clone() as Rc<dyn Memory>,
        heaps::HEAD,
        heaps::SPAN,
        Some(heaps::COUNT),
        Some(heaps::SMALL),
        heaps::unread(exact.clone(), &heaps::UNREAD),
        Some({
            let watched = watched.clone();
            Rc::new(move |reads: &[(i64, i64)]| watched.read_each(reads))
        }),
    )));

    let walk: Walk = {
        let (live, watch, roots) = (live.clone(), watch.clone(), roots.clone());
        let chunk = setup.sweep_chunk;
        Rc::new(move |between: &dyn Fn()| {
            let walked = walker.walk(Some(between), chunk)?;
            watch.borrow_mut().forget(&live.walked(&walked.spans));
            *roots.borrow_mut() = walked.roots;
            Ok(walked.bodies)
        })
    };

    let blocks: Blocks = {
        let live = live.clone();
        Rc::new(move || live.spans())
    };
    let described = Rc::new(scenes::Described::new(
        Rc::new(|_: &str, _: i64, _: i64| Ok(None)),
        Rc::new(|_: &str, _: i64| None),
        None,
    )?);
    // Sun's scenes are found by their demo's number, so none follows a line and none names a
    // thing the game does (ADR-0038).
    let loaded = if setup.tables.ui.descriptions {
        let known = {
            let described = described.clone();
            move |message: &str| described.known(message)
        };
        scenes::load(
            scenes::shipped(&field::names()),
            &|_: &str, _: i64, _: i64| Ok(None),
            &known,
            &setup.log,
            Some(setup.process.title_id),
        )?
    } else {
        Vec::new()
    };
    let profile: Rc<dyn Profile> = Rc::new(SunProfile::with(
        exact.clone(),
        names,
        speech,
        setup.log.clone(),
        Options {
            ui: Some(setup.tables.ui.clone()),
            navigation: Some(setup.tables.navigation.clone()),
            text,
            zones,
            named: Some(named),
            keys: Some(setup.keys.clone()),
            blocks: Some(blocks),
            walls: walls.map(|walls| walls as Rc<dyn field::Walls>),
            phases: Some(exact.clone() as Rc<dyn Cache>),
            read_each: Some({
                let exact = exact.clone();
                Rc::new(move |reads: &[(i64, i64)]| exact.read_each(reads))
            }),
            scenes: loaded,
            said: Some(Rc::new(move |message: &str| described.said(message))),
        },
    ));

    let small_block = {
        let live = live.clone();
        move |block: Block| {
            // A block owed another sweep may have been taken over since by one that may not be
            // read.
            let span = heaps::to_sweep(block);
            span.filter(|&span| live.holds(span))
        }
    };

    Ok(Session {
        profile,
        classes: heap_classes().into_iter().map(String::from).collect(),
        required: REQUIRED.iter().map(|class| class.to_string()).collect(),
        heaps: watch.clone(),
        watched,
        adopt: {
            let (watch, roots) = (watch.clone(), roots.clone());
            Rc::new(move |_found| {
                let roots = roots.borrow().clone();
                watch.borrow_mut().adopt(&roots, true)?;
                Ok(())
            })
        },
        to_sweep: Rc::new(small_block),
        allocated: {
            let live = live.clone();
            Rc::new(move |blocks: &[Block]| watch.borrow_mut().forget(&live.allocated(blocks)))
        },
        walk: Some(walk),
        language: Rc::new(|| None),
        bases: Vec::new(),
        pages: None,
    })
}
