//! Which of the summary's pages is shown, learnt from the first time each page's layouts show
//! (`gen6.screens.summary.Pages`).

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::layout::TrackedLayouts;
use crate::games::common::layout_file::{FileMatching, LayoutFile};
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::RunningApp;
use crate::games::gen6::screens::summary;
use crate::py::Result;

const PROCESS: i64 = 0x08C652BC;
/// Two pages' values, addresses in the code.
const STATS: i64 = 0x0073190C;
const CONDITIONS: i64 = 0x007326B8;
const METAGROSS: i64 = 0;
const TORCHIC: i64 = 1;
const WORKS: [(i64, i64); 3] = [
    (0x0821A000, 0x0822A000),
    (0x0821B000, 0x0822B000),
    (0x0821C000, 0x0822C000),
];
const PARAMS: i64 = 0x0822A000;
const WAZA: i64 = 0x0822B000;
const CONTEST: i64 = 0x0822C000;

struct FakeLayouts;

impl TrackedLayouts for FakeLayouts {
    fn works(&self) -> IndexMap<i64, i64> {
        IndexMap::from(WORKS)
    }

    fn pane_names(&self, root: i64) -> Vec<String> {
        let file = match root {
            PARAMS => "params.bclyt",
            WAZA => "waza.bclyt",
            CONTEST => summary::STAR_PAGE,
            _ => panic!("KeyError: {root}"),
        };
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

#[test]
fn test_a_page_is_learnt_as_it_first_shows_and_the_others_are_unheard() {
    let memory = FakeMemory::new();
    let apps = Rc::new(RefCell::new(RunningApp::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
    )));
    apps.borrow_mut().running = Some(summary::SUMMARY.to_string());
    apps.borrow_mut().process = Some(PROCESS);
    let mut pages = summary::Pages::new(
        memory.clone(),
        apps.clone(),
        Rc::new(FakeLayouts),
        Rc::new(FakeFiles),
    );
    let stats = vec![piece(PARAMS, "HP 199/227", 1), piece(WAZA, "Hold Back", 2)];
    let mut both = stats.clone();
    both.extend([piece(CONTEST, "Name", 3), piece(CONTEST, "Coolness", 4)]);

    // `poll(word, on_screen, mon, cursored)`, then what the pages say: hidden and turned.
    let mut poll = |word: i64, on_screen: &[ScreenText], mon: i64, cursored: &[i64]| {
        memory.u32(PROCESS + summary::PAGE, &[word]);
        memory.u32(PROCESS + summary::PLACE, &[mon]);
        pages.poll(on_screen, &roots(cursored)).unwrap();
        (pages.hidden(), pages.turned())
    };

    // As the summary opens its value points into the heap: nothing is learnt.
    assert_eq!(poll(0x08C6A770, &stats, METAGROSS, &[]).0, roots(&[]));
    assert_eq!(poll(STATS, &stats, METAGROSS, &[]).0, roots(&[]));
    // The conditions page's layout shows first under its value, and is its; the first visit is
    // no turn, its words newly on screen.
    let (hidden, turned) = poll(CONDITIONS, &both, METAGROSS, &[]);
    assert_eq!(turned, roots(&[]));
    assert_eq!(hidden, roots(&[PARAMS, WAZA]));
    // Turned back: no page is heard as the game draws it afresh, then the stats page is said,
    // the other unheard.
    let (hidden, turned) = poll(STATS, &both, METAGROSS, &[]);
    assert_eq!(turned, roots(&[]));
    assert_eq!(hidden, roots(&[PARAMS, WAZA, CONTEST]));
    let (hidden, turned) = poll(STATS, &both, METAGROSS, &[]);
    assert_eq!(turned, roots(&[PARAMS, WAZA]));
    assert_eq!(hidden, roots(&[CONTEST]));
    let (_, turned) = poll(STATS, &both, METAGROSS, &[]);
    assert_eq!(turned, roots(&[]));
    // A page changed with the Pokémon is a switch, said as one, not a turn: an Egg's memo.
    let (hidden, turned) = poll(CONDITIONS, &both, TORCHIC, &[]);
    assert_eq!(turned, roots(&[]));
    assert_eq!(hidden, roots(&[PARAMS, WAZA]));
    // Check Moves, a cursor on a page's moves, flips the page under the player: no flip is a
    // turn, and each move is heard in the shown page's words.
    let (hidden, turned) = poll(STATS, &both, TORCHIC, &[WAZA]);
    assert_eq!(turned, roots(&[]));
    assert_eq!(hidden, roots(&[CONTEST]));
    // The star's labels, on the conditions page alone.
    let words: HashSet<String> = ["Coolness", "HP 199/227"]
        .iter()
        .map(|word| word.to_string())
        .collect();
    assert_eq!(pages.star_labels(&both, &words), roots(&[CONTEST + 4]));
    // Out of the summary, every page is forgotten.
    apps.borrow_mut().running = None;
    pages.poll(&[], &roots(&[])).unwrap();
    assert_eq!(pages.hidden(), roots(&[]));
}
