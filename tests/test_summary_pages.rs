//! Which of the summary's pages is shown, by the flag of each page's list
//! (`gen6.screens.summary.Pages`).

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::{FileMatching, LayoutFile};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::RunningApp;
use crate::games::gen6::screens::summary;
use crate::py::Result;

const PROCESS: i64 = 0x08C652BC;
/// The pages' lists, in `summary.LISTS`'s order: the stats page, the conditions page, the memo.
const MOVES: i64 = 0x08C70000;
const CONTEST_MOVES: i64 = 0x08C71000;
const RIBBONS: i64 = 0x08C72000;
const METAGROSS: i64 = 0;
const EGG: i64 = 1;
const TORCHIC: i64 = 2;
const IRON_HEAD: i64 = 442;
const FILES: [(i64, &str); 6] = [
    (0x0822A000, "params.bclyt"),
    (0x0822B000, "waza.bclyt"),
    (0x0822C000, summary::STAR_PAGE),
    (0x0822D000, "memo.bclyt"),
    (0x0822E000, summary::BUTTON),
    (0x0822F000, summary::INFO),
];
const PARAMS: i64 = FILES[0].0;
const WAZA: i64 = FILES[1].0;
const CONTEST: i64 = FILES[2].0;
const MEMO: i64 = FILES[3].0;
const BUTTON: i64 = FILES[4].0;
const INFO: i64 = FILES[5].0;

struct FakeLayouts;

impl TrackedLayouts for FakeLayouts {
    fn works(&self) -> IndexMap<i64, i64> {
        FILES
            .iter()
            .map(|&(root, _)| (root - 0x10000, root))
            .collect()
    }

    fn pane_names(&self, root: i64) -> Vec<String> {
        let (_, file) = FILES
            .iter()
            .find(|&&(each, _)| each == root)
            .unwrap_or_else(|| panic!("KeyError: {root}"));
        vec![file.to_string()]
    }

    fn name(&self, _pane: i64) -> Result<Vec<u8>> {
        panic!("_Layouts has no attribute 'name'")
    }
}

struct FakeFiles;

impl FileMatching for FakeFiles {
    fn matching(&self, panes: &[String]) -> Option<Rc<LayoutFile>> {
        panes.first().map(|name| {
            Rc::new(LayoutFile {
                name: name.clone(),
                ..LayoutFile::new(Vec::new(), IndexMap::new(), IndexMap::new())
            })
        })
    }
}

fn piece(root: i64, text: &str, r#box: i64) -> ScreenText {
    ScreenText {
        top: true,
        root,
        ..ScreenText::new(root + r#box, text, false)
    }
}

fn roots(roots: &[i64]) -> HashSet<i64> {
    roots.iter().copied().collect()
}

fn summary_of() -> (Rc<FakeMemory>, Rc<RefCell<RunningApp>>, summary::Pages) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::starting(memory.clone(), CODE.0 + 0x8000);
    for (&(at, name, _, _), each) in summary::LISTS.iter().zip([MOVES, CONTEST_MOVES, RIBBONS]) {
        memory.u32(PROCESS + at, &[each]);
        memory.u32(each, &[classes.add(name)]);
    }
    let apps = Rc::new(RefCell::new(RunningApp::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
    )));
    apps.borrow_mut().running = Some(summary::SUMMARY.to_string());
    apps.borrow_mut().process = Some(PROCESS);
    let pages = summary::Pages::new(
        memory.clone(),
        apps.clone(),
        Rc::new(FakeLayouts),
        Rc::new(FakeFiles),
        summary::ORAS,
    );
    (memory, apps, pages)
}

/// `_poll(memory, pages, shown, mon, chosen)`: `chosen` is the move each list of moves holds,
/// the stats page's and the conditions page's.
fn poll_with(
    memory: &FakeMemory,
    pages: &mut summary::Pages,
    shown: Option<i64>,
    mon: i64,
    chosen: (i64, i64),
) {
    for each in [MOVES, CONTEST_MOVES, RIBBONS] {
        memory.write(each + summary::SHOWING, &[(Some(each) == shown) as u8]);
    }
    for (each, r#move) in [(MOVES, chosen.0), (CONTEST_MOVES, chosen.1)] {
        memory.u16(each + summary::CHOSEN, r#move);
    }
    memory.u32(PROCESS + summary::PLACE, &[mon]);
    pages.poll().unwrap();
}

/// `_poll(memory, pages, shown)`: Metagross, and no move chosen.
fn poll(memory: &FakeMemory, pages: &mut summary::Pages, shown: Option<i64>) {
    poll_with(memory, pages, shown, METAGROSS, (0, 0));
}

#[test]
fn test_the_page_shown_is_the_one_whose_list_is_flagged_and_the_others_are_unheard() {
    let (memory, apps, mut pages) = summary_of();
    let pages = &mut pages;
    // As the summary opens no list is flagged: nothing is unheard.
    poll(&memory, pages, None);
    assert_eq!(pages.hidden(), roots(&[]));
    poll(&memory, pages, Some(MOVES));
    assert_eq!(pages.turned(), roots(&[]));
    assert_eq!(pages.hidden(), roots(&[CONTEST, MEMO]));
    // A turn: no page is heard as the game draws it afresh, then the page turned to is said, the
    // others unheard, and its button a poll after it, once a memo with none has hidden it.
    poll(&memory, pages, Some(CONTEST_MOVES));
    assert_eq!(pages.turned(), roots(&[]));
    assert_eq!(
        pages.hidden(),
        roots(&[PARAMS, WAZA, CONTEST, MEMO, BUTTON])
    );
    poll(&memory, pages, Some(CONTEST_MOVES));
    assert_eq!(pages.turned(), roots(&[CONTEST]));
    assert_eq!(pages.button(), roots(&[]));
    assert_eq!(pages.hidden(), roots(&[PARAMS, WAZA, MEMO, BUTTON]));
    poll(&memory, pages, Some(CONTEST_MOVES));
    assert_eq!(pages.turned(), roots(&[]));
    assert_eq!(pages.button(), roots(&[BUTTON]));
    assert_eq!(pages.hidden(), roots(&[PARAMS, WAZA, MEMO]));
    poll(&memory, pages, Some(CONTEST_MOVES));
    assert_eq!(pages.button(), roots(&[]));
    // No list flagged for a poll: the page before holds.
    poll(&memory, pages, None);
    assert_eq!(pages.hidden(), roots(&[PARAMS, WAZA, MEMO]));
    // Check Moves flips the page under the player, both lists holding the move chosen: no flip
    // is a turn, and each move is heard in the shown page's words.
    poll_with(
        &memory,
        pages,
        Some(CONTEST_MOVES),
        METAGROSS,
        (0, IRON_HEAD),
    );
    poll_with(
        &memory,
        pages,
        Some(MOVES),
        METAGROSS,
        (IRON_HEAD, IRON_HEAD),
    );
    assert_eq!(pages.turned(), roots(&[]));
    assert_eq!(pages.hidden(), roots(&[CONTEST, MEMO]));
    poll_with(
        &memory,
        pages,
        Some(MOVES),
        METAGROSS,
        (IRON_HEAD, IRON_HEAD),
    );
    assert_eq!(pages.turned(), roots(&[]));
    poll_with(
        &memory,
        pages,
        Some(CONTEST_MOVES),
        METAGROSS,
        (IRON_HEAD, IRON_HEAD),
    );
    poll_with(
        &memory,
        pages,
        Some(CONTEST_MOVES),
        METAGROSS,
        (IRON_HEAD, IRON_HEAD),
    );
    assert_eq!(pages.turned(), roots(&[]));
    assert_eq!(pages.hidden(), roots(&[PARAMS, WAZA, MEMO]));
    // Ended on the conditions page, Check Moves leaves the stats page's list its move: a turn
    // after it is a turn.
    poll_with(
        &memory,
        pages,
        Some(CONTEST_MOVES),
        METAGROSS,
        (IRON_HEAD, 0),
    );
    poll_with(&memory, pages, Some(MOVES), METAGROSS, (IRON_HEAD, 0));
    poll_with(&memory, pages, Some(MOVES), METAGROSS, (IRON_HEAD, 0));
    assert_eq!(pages.turned(), roots(&[PARAMS, WAZA]));
    // The star's labels, on the conditions page alone.
    let star = [
        piece(CONTEST, "Coolness", 4),
        piece(PARAMS, "HP 199/227", 1),
    ];
    let words: HashSet<String> = ["Coolness", "HP 199/227"]
        .iter()
        .map(|word| word.to_string())
        .collect();
    assert_eq!(pages.star_labels(&star, &words), roots(&[CONTEST + 4]));
    // Out of the summary, every page is forgotten.
    apps.borrow_mut().running = None;
    pages.poll().unwrap();
    assert_eq!(pages.hidden(), roots(&[]));
}

#[test]
fn test_an_egg_and_the_pokemon_after_it_are_switches_not_turns() {
    let (memory, _, mut pages) = summary_of();
    let pages = &mut pages;
    poll(&memory, pages, Some(MOVES));
    // An Egg turns the summary to its memo as the Pokémon changes: said as a switch, the name
    // first, over two polls.
    poll_with(&memory, pages, Some(RIBBONS), EGG, (0, 0));
    assert_eq!(pages.turned(), roots(&[]));
    assert_eq!(pages.switched(), roots(&[MEMO, INFO, BUTTON]));
    poll_with(&memory, pages, Some(RIBBONS), EGG, (0, 0));
    assert_eq!(pages.switched(), roots(&[MEMO, INFO, BUTTON]));
    poll_with(&memory, pages, Some(RIBBONS), EGG, (0, 0));
    assert_eq!(pages.switched(), roots(&[]));
    // Leaving it, the stats page's list is flagged a poll before the place changes: heard as
    // nothing for that poll, then as a switch, not a turn.
    poll_with(&memory, pages, Some(MOVES), EGG, (0, 0));
    assert_eq!(pages.switched(), roots(&[]));
    assert_eq!(
        pages.hidden(),
        roots(&[PARAMS, WAZA, CONTEST, MEMO, BUTTON])
    );
    poll_with(&memory, pages, Some(MOVES), TORCHIC, (0, 0));
    assert_eq!(pages.turned(), roots(&[]));
    assert_eq!(pages.button(), roots(&[]));
    assert_eq!(pages.switched(), roots(&[PARAMS, WAZA, INFO, BUTTON]));
    assert_eq!(pages.hidden(), roots(&[CONTEST, MEMO]));
}

#[test]
fn test_lists_not_of_their_class_say_no_page() {
    let (memory, _, mut pages) = summary_of();
    memory.u32(PROCESS + summary::LISTS[1].0, &[RIBBONS]);
    poll(&memory, &mut pages, Some(MOVES));
    assert_eq!(pages.hidden(), roots(&[]));
}

#[test]
fn test_x_has_two_pages_whose_lists_and_place_sit_sooner() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::starting(memory.clone(), CODE.0 + 0x8000);
    for (&(at, name, _, _), each) in summary::XY.lists.iter().zip([MOVES, RIBBONS]) {
        memory.u32(PROCESS + at, &[each]);
        memory.u32(each, &[classes.add(name)]);
    }
    let apps = Rc::new(RefCell::new(RunningApp::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
    )));
    apps.borrow_mut().running = Some(summary::SUMMARY.to_string());
    apps.borrow_mut().process = Some(PROCESS);
    let mut pages = summary::Pages::new(
        memory.clone(),
        apps,
        Rc::new(FakeLayouts),
        Rc::new(FakeFiles),
        summary::XY,
    );
    for (shown, mon) in [(MOVES, METAGROSS), (RIBBONS, METAGROSS), (RIBBONS, TORCHIC)] {
        for each in [MOVES, RIBBONS] {
            memory.write(each + summary::SHOWING, &[(each == shown) as u8]);
        }
        memory.u32(PROCESS + summary::XY.place, &[mon]);
        pages.poll().unwrap();
    }
    // The memo shown, the stats page's layouts unheard; the switch to Torchic said as one.
    assert_eq!(pages.hidden(), roots(&[PARAMS, WAZA, CONTEST]));
    assert_eq!(pages.switched(), roots(&[MEMO, INFO, BUTTON]));
}
