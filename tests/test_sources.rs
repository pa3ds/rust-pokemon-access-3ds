//! The screens' cursor sources as the screen poll asks them (`gen6.screens.sources`): what the
//! profile once stitched itself, line by line, now asked of one module.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::pokemon::Pokemon;
use crate::games::common::screen::{ScreenPoll, ScreenText};
use crate::games::gen6::apps::RunningApp;
use crate::games::gen6::game::CursorShapes;
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::screens::sources::{Chase, CursorSources};
use crate::games::gen6::screens::{boxes, rows, townmap};

const ROOT: i64 = 0x08300400;
const OTHER: i64 = 0x08310400;

/// `_sources(running, box)`: `r#box` is the box the PC Box's grid shows.
fn sources_of(running: &str, r#box: Rc<Cell<Option<i64>>>) -> CursorSources {
    let memory = FakeMemory::new();
    let names = Rc::new(ClassNames::new(memory.clone(), CODE));
    let layouts = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()))
        .layouts
        .clone();
    let apps = Rc::new(RefCell::new(RunningApp::new(memory.clone(), names.clone())));
    apps.borrow_mut().running = Some(running.to_string());
    apps.borrow_mut().process = None;
    let shown: boxes::Shown = Rc::new(move || Ok(r#box.get()));
    let occupants = Rc::new(RefCell::new(boxes::Occupants::with(
        memory.clone(),
        names.clone(),
        Rc::new(|| None),
        Rc::new(|| None),
        shown.clone(),
    )));
    CursorSources::new(
        memory.clone(),
        names.clone(),
        apps,
        layouts,
        Rc::new(LayoutFiles::new(memory.clone())),
        &CursorShapes {
            r#box: Some(boxes::GEN6),
            ..CursorShapes::default()
        },
        &GameText::new(Vec::new(), Vec::new()),
        false,
        Rc::new(|| Ok(None)),
        Rc::new(Vec::new),
        Rc::new(Vec::new),
        occupants,
        shown,
        Rc::new(RefCell::new(townmap::Views::new(memory.clone(), names))),
        None,
        None,
        None,
    )
}

/// `_screen(*texts)`.
fn screen(texts: &[ScreenText]) -> ScreenPoll {
    ScreenPoll {
        texts: texts.to_vec(),
        on_screen: texts.to_vec(),
        left: Vec::new(),
        settling: IndexSet::new(),
        message_lines: IndexSet::new(),
    }
}

/// `_cursors(*items, under=...)`.
fn cursors_of(items: Vec<CursorItem>, under: &[ScreenText]) -> CursorPoll {
    CursorPoll {
        items,
        under: if under.is_empty() {
            Vec::new()
        } else {
            vec![under.to_vec()]
        },
        lists: IndexMap::new(),
        moving: HashSet::new(),
        rows: HashSet::new(),
        chased: false,
    }
}

/// No menu cursors on screen (`windows`).
fn windows() -> Vec<i64> {
    Vec::new()
}

/// No shiny Pokémon in battle (`targets`).
fn targets() -> IndexMap<i64, Pokemon> {
    IndexMap::new()
}

/// ADR-0043: the Box's box is said as it turns, the screen polled at once.
#[test]
fn test_a_box_turned_to_polls_the_screen_once() {
    let r#box = Rc::new(Cell::new(Some(2)));
    let mut sources = sources_of(boxes::APP, r#box);
    assert_eq!(sources.moved(&windows).unwrap(), Chase::ONCE);
    sources
        .poll(&screen(&[]), cursors_of(Vec::new(), &[]), false, &targets)
        .unwrap();
    assert_eq!(sources.moved(&windows).unwrap(), Chase::NONE);
}

/// The Box's byte moved and its pointer has yet to say where: the poll is the slot's.
#[test]
fn test_a_box_slot_chased_owns_what_the_screen_changes_meanwhile() {
    let mut sources = sources_of(boxes::APP, Rc::new(Cell::new(Some(0))));
    assert!(
        sources
            .poll(&screen(&[]), cursors_of(Vec::new(), &[]), true, &targets)
            .unwrap()
            .cursors
            .chased
    );
    assert!(
        !sources
            .poll(&screen(&[]), cursors_of(Vec::new(), &[]), false, &targets)
            .unwrap()
            .cursors
            .chased
    );
    let mut elsewhere = sources_of(rows::OPTIONS, Rc::new(Cell::new(None)));
    assert!(
        !elsewhere
            .poll(&screen(&[]), cursors_of(Vec::new(), &[]), true, &targets)
            .unwrap()
            .cursors
            .chased
    );
}

/// Its tabs, list and buttons open with it as a menu's do: only their changes are heard.
#[test]
fn test_the_fitting_rooms_cursor_owns_the_layouts_it_walks_as_they_open() {
    let tab = ScreenText {
        root: ROOT,
        ..ScreenText::new(ROOT + 1, "hat", false)
    };
    let card = ScreenText {
        root: ROOT,
        with_layout: true,
        ..ScreenText::new(ROOT + 2, "Two-Tone Bag", false)
    };
    let changed = ScreenText {
        root: ROOT,
        with_layout: true,
        ..ScreenText::new(ROOT + 3, "Change", true)
    };
    let other = ScreenText {
        root: OTHER,
        with_layout: true,
        ..ScreenText::new(OTHER + 1, "Back", false)
    };
    let cursors = cursors_of(
        vec![CursorItem::new(1, vec![tab.clone()], true)],
        std::slice::from_ref(&tab),
    );
    let fitting = sources_of(rows::FITTING, Rc::new(Cell::new(None)));
    let said = fitting.said(
        cursors.clone(),
        vec![card.clone(), changed.clone(), other.clone()],
        &[],
    );
    assert_eq!(said.texts, vec![changed.clone(), other.clone()]);
    assert!(
        said.lead == said.reread && said.reread == said.rewritten && said.rewritten.is_empty()
    );
    let elsewhere = sources_of(rows::OPTIONS, Rc::new(Cell::new(None)));
    assert_eq!(
        elsewhere
            .said(
                cursors,
                vec![card.clone(), changed.clone(), other.clone()],
                &[]
            )
            .texts,
        vec![card, changed, other]
    );
}
