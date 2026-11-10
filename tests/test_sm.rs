//! Sun and Moon tests: the heaps pa3ds may sweep, and dialogue read and said with its speaker, in a
//! fake laid out as spike 111 read the game.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_emulator::FakeEmulator;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::fake_speech::Recorder;
use crate::core::discovery::Found;
use crate::core::heaps::Block;
use crate::core::memory::{Memory, RpcMemory};
use crate::core::rpc::{MAX_READ, Process, Rpc, UdpRpc};
use crate::core::rtti::ClassNames;
use crate::core::speech::Speech;
use crate::games::common::announce::{Focus, SPEAKER_POLLS};
use crate::games::common::msgwin as common_msgwin;
use crate::games::common::session::Profile;
use crate::games::sm::heaps::{self, Blocks, Heaps};
use crate::games::sm::langselect::{self, LangSelectReader, Shown};
use crate::games::sm::profile::SunProfile;
use crate::games::sm::text::SunText;
use crate::games::sm::{apps, msgwin, talk};
use crate::py::Result;

const TABLE: i64 = 0x08000000;
const WINDOW: i64 = 0x30001000;
const RECORDS: i64 = 0x30002000;
const BOXES: [i64; 2] = [0x30003000, 0x30004000];
const TYPED: [i64; 2] = [0x30005000, 0x30005400];
const TALK: i64 = 0x30006000;
const UTIL: i64 = 0x30006800;
const WINDOW_BUFFER: i64 = 0x30007000;
const NAME_BUFFER: i64 = 0x30007200;
const WINDOW_TEXT: i64 = 0x30008000;
const NAME_TEXT: i64 = 0x30009000;
const SCROLL: &str = "\u{10}\u{1}븀";

type Span = (i64, i64);

/// A fake's pages as the fake emulator serves them, a read's worth each.
fn served(memory: &FakeMemory) -> HashMap<i64, Vec<u8>> {
    let mut pages = HashMap::new();
    for (base, page) in memory.pages() {
        for at in (0..page.len()).step_by(MAX_READ as usize) {
            let end = (at + MAX_READ as usize).min(page.len());
            pages.insert(base + at as i64, page[at..end].to_vec());
        }
    }
    pages
}

fn connect(emu: &FakeEmulator) -> Rc<dyn Rpc> {
    Rc::new(UdpRpc::connect(emu.port()).unwrap())
}

/// A `CTRExpHeap` at `at` with this id and first used block, and its entry in the table.
/// Returns its head.
fn heap(
    memory: &FakeMemory,
    vtable: i64,
    at: i64,
    ident: i64,
    used: i64,
    child: i64,
    sibling: i64,
) -> i64 {
    memory.u32(at, &[vtable]);
    memory.u32(at + heaps::ID, &[ident]);
    let head = at + heaps::HEAD;
    memory.write(head, heaps::EXPH);
    memory.u32(head + heaps::NEXT, &[sibling, child]);
    memory.u32(head + heaps::USED, &[used]);
    memory.u32(TABLE + ident * heaps::ENTRY, &[at]);
    head
}

/// A used block's header at `at`. Returns what it holds, as (start, end).
fn block(memory: &FakeMemory, at: i64, size: i64, following: i64) -> Span {
    memory.u16(at, heaps::USED_BLOCK);
    memory.u32(at + heaps::BLOCK_SIZE, &[size]);
    memory.u32(at + heaps::BLOCK_NEXT, &[following]);
    (at + heaps::BLOCK_HEADER, at + heaps::BLOCK_HEADER + size)
}

fn names(memory: &Rc<FakeMemory>) -> Rc<ClassNames> {
    Rc::new(ClassNames::new(memory.clone(), CODE))
}

#[test]
fn test_only_small_blocks_under_the_dialogue_heaps_are_swept() {
    // The GPU's buffers sit between Sun's heaps and inside them, as one of the field's 529 KB
    // blocks: reading them took the emulator down (spike 111).
    let memory = FakeMemory::new();
    let ctr = FakeClasses::new(memory.clone()).add(heaps::CLASS);
    let window = block(&memory, 0x30001000, 0x114, 0x30002000);
    block(&memory, 0x30002000, 0x81280, 0x30100000); // the GPU's
    block(&memory, 0x30100000, 0x200, 0); // holds the child heap below
    let talk_window = block(&memory, 0x30200000, 0xE4, 0);
    let elsewhere = block(&memory, 0x30300000, 0x40, 0);
    let sibling = heap(&memory, ctr, 0x302FFF00, 0x21, 0x30300000, 0, 0);
    let child = heap(&memory, ctr, 0x30100010, 0x30, 0x30200000, 0, 0);
    heap(&memory, ctr, 0x30000000, 5, 0x30001000, child, sibling);
    heap(&memory, ctr, 0x31000000, 7, 0x30300000, 0, 0);
    memory.u32(TABLE + 0x0B * heaps::ENTRY, &[0x31000000]); // its own id says 7
    let emu = FakeEmulator::new(served(&memory));
    let rpc = connect(&emu);
    let mut spans = Heaps::new(rpc, names(&memory)).spans().unwrap();
    spans.sort();
    assert_eq!(spans, [window, talk_window]);
    assert!(!spans.contains(&elsewhere));
}

/// An `Rpc` that keeps every read it is asked for.
struct Reads {
    rpc: Rc<dyn Rpc>,
    asked: RefCell<Vec<Span>>,
}

impl Reads {
    fn new(rpc: Rc<dyn Rpc>) -> Rc<Reads> {
        Rc::new(Reads {
            rpc,
            asked: RefCell::new(Vec::new()),
        })
    }
}

impl Rpc for Reads {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        self.asked.borrow_mut().push((address, size));
        self.rpc.read(address, size)
    }

    fn read_each(&self, reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        self.asked.borrow_mut().extend_from_slice(reads);
        self.rpc.read_each(reads)
    }

    fn reply_seconds(&self) -> f64 {
        self.rpc.reply_seconds()
    }

    fn reads_beside_gpu(&self) -> bool {
        self.rpc.reads_beside_gpu()
    }

    fn unsure(&self) -> bool {
        self.rpc.unsure()
    }

    fn confirm(&self) -> Result<bool> {
        self.rpc.confirm()
    }

    fn attach(&self) -> Result<Process> {
        self.rpc.attach()
    }
}

#[test]
fn test_a_walk_reads_each_small_block_with_the_header_after_it() {
    // Reading each header and then each body was two reads a block, 27,000 every 2 s on Route 1,
    // and about 1.1 s of each 2 s (sun_4.ctm, 2026-09-26). A body is read with the header that
    // follows it, and nothing is read that is not a small block or a header.
    let memory = FakeMemory::new();
    let ctr = FakeClasses::new(memory.clone()).add(heaps::CLASS);
    let first = block(&memory, 0x30001000, 0x30, 0x30001040);
    let second = block(&memory, 0x30001040, 0x500, 0x30001550);
    block(&memory, 0x30001550, 0x81280, 0x30082800); // the GPU's
    let last = block(&memory, 0x30082800, 0x20, 0);
    memory.write(first.0, b"first");
    memory.write(second.0 + 0x4F0, b"second's end");
    heap(&memory, ctr, 0x30000000, 5, 0x30001000, 0, 0);
    let emu = FakeEmulator::new(served(&memory));
    let reads = Reads::new(connect(&emu));
    let walked = Heaps::new(reads.clone(), names(&memory))
        .walk(None, heaps::CHUNK)
        .unwrap();
    assert_eq!(walked.spans, [first, second, last]);
    assert!(walked.bodies[&first.0].starts_with(b"first"));
    assert_eq!(walked.bodies[&second.0][0x4F0..0x4FC], b"second's end"[..]);
    let sizes: IndexMap<i64, usize> = walked
        .bodies
        .iter()
        .map(|(&start, body)| (start, body.len()))
        .collect();
    assert_eq!(
        sizes,
        IndexMap::from([(first.0, 0x30), (second.0, 0x500), (last.0, 0x20)])
    );
    let blocks = [
        (first.0 - 0x10, first.1 + 0x10),
        (second.0, second.1 + 0x10),
        (0x30001550, 0x30001560),
        (last.0 - 0x10, last.1),
    ];
    let walk_reads: Vec<Span> = reads
        .asked
        .borrow()
        .iter()
        .copied()
        .filter(|read| read.0 >= 0x30001000)
        .collect();
    assert!(walk_reads.iter().all(|&(at, size)| {
        blocks
            .iter()
            .any(|&(low, high)| low <= at && at + size <= high)
    }));
    // a header, its body with the next header in two, the GPU's header with the body before it,
    // the last header, and the last body
    assert_eq!(walk_reads.len(), 6);
}

#[test]
fn test_the_field_s_own_blocks_are_not_walked_but_the_heaps_inside_it_are() {
    // Heap 0x21's own blocks are the rendering engine's, four of every five round trips of a walk
    // on Route 1, and its actors are in 0x45 inside it (sun_4.ctm, 2026-09-26).
    let memory = FakeMemory::new();
    let ctr = FakeClasses::new(memory.clone()).add(heaps::CLASS);
    block(&memory, 0x30101000, 0x40, 0x30101050);
    block(&memory, 0x30101050, 0x200, 0x30102000); // holds the actors' heap
    let actor = block(&memory, 0x30200000, 0x24, 0);
    let actors = heap(&memory, ctr, 0x30101060, 0x45, 0x30200000, 0, 0);
    let field = heap(&memory, ctr, 0x30100000, 0x21, 0x30101000, actors, 0);
    heap(&memory, ctr, 0x30000000, 5, 0, field, 0);
    let emu = FakeEmulator::new(served(&memory));
    let reads = Reads::new(connect(&emu));
    let spans = Heaps::new(reads.clone(), names(&memory)).spans().unwrap();
    assert_eq!(spans, [actor]);
    let asked: Vec<i64> = reads.asked.borrow().iter().map(|read| read.0).collect();
    assert!(!asked.contains(&0x30101000) && !asked.contains(&0x30101050));
}

#[test]
fn test_a_walk_polls_as_it_goes() {
    let memory = FakeMemory::new();
    let ctr = FakeClasses::new(memory.clone()).add(heaps::CLASS);
    for at in (0x30001000..0x30001400).step_by(0x40) {
        let following = if at < 0x300013C0 { at + 0x40 } else { 0 };
        block(&memory, at, 0x30, following);
    }
    heap(&memory, ctr, 0x30000000, 5, 0x30001000, 0, 0);
    let polls = RefCell::new(Vec::new());
    let emu = FakeEmulator::new(served(&memory));
    let rpc = connect(&emu);
    let walked = Heaps::new(rpc, names(&memory))
        .walk(Some(&|| polls.borrow_mut().push(1)), 4)
        .unwrap();
    assert_eq!(walked.spans.len(), 16);
    assert_eq!(polls.borrow().len(), 4);
}

fn buffer(memory: &FakeMemory, buffer: i64, text_at: i64, text: &str) -> i64 {
    let raw: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
    memory.write(text_at, &[raw.clone(), vec![0, 0]].concat());
    memory.u32(buffer + 4, &[text_at]);
    memory.u16(buffer + 0x0A, raw.len() as i64 / 2);
    buffer
}

/// Sun's field box: a two-line message window, and the talk window with the speaker's plate.
struct Scene {
    memory: Rc<FakeMemory>,
    recorder: Rc<Recorder>,
}

impl Scene {
    fn new() -> Scene {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let textbox = classes.add("N2nw3lyt7TextBoxE");
        let window_class = classes.add(msgwin::CLASSES[0]);
        let talk_class = classes.add(talk::CLASS);
        let util_class = classes.add(talk::UTIL_CLASS);
        memory.u32(WINDOW, &[window_class]);
        memory.u32(WINDOW + common_msgwin::LINES, &[RECORDS]);
        memory.u32(WINDOW + common_msgwin::MESSAGE, &[WINDOW_BUFFER]);
        memory.write(WINDOW + common_msgwin::LINE_COUNT, &[2, 1]);
        for (i, r#box) in BOXES.into_iter().enumerate() {
            memory.u32(RECORDS + i as i64 * common_msgwin::LINE_RECORD, &[r#box]);
            memory.u32(r#box, &[textbox]);
            memory.u32(r#box + msgwin::TEXT, &[TYPED[i]]);
            memory.u32(
                r#box + msgwin::TAG_PROCESSOR,
                &[WINDOW + common_msgwin::TAG_PROCESSOR],
            );
            memory.u16(r#box + msgwin::CAPACITY, 55);
        }
        memory.u32(TALK, &[talk_class]);
        memory.u32(TALK + talk::UTIL, &[UTIL]);
        memory.u32(TALK + talk::SPEAKER, &[NAME_BUFFER]);
        memory.u32(UTIL, &[util_class]);
        memory.u32(UTIL + talk::UTIL_WINDOW, &[WINDOW]);
        Scene {
            memory,
            recorder: Recorder::new(),
        }
    }

    fn message(&self, text: &str, speaker: &str) {
        buffer(&self.memory, WINDOW_BUFFER, WINDOW_TEXT, text);
        buffer(&self.memory, NAME_BUFFER, NAME_TEXT, speaker);
    }

    fn r#type(&self, line: usize, text: &str) {
        let raw: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
        self.memory
            .write(TYPED[line], &[raw.clone(), vec![0, 0]].concat());
        self.memory
            .u16(BOXES[line] + msgwin::LENGTH, raw.len() as i64 / 2);
    }

    /// Type these lines, then poll a profile reading the scene from its first frame, whose
    /// sweep found `swept`, and a sweep of the blocks just allocated `then`.
    fn said(&self, typing: &[(usize, &str)], swept: &[i64], then: &[i64]) -> Vec<String> {
        for &(line, text) in typing {
            self.r#type(line, text);
        }
        let classes = HashMap::from([
            (WINDOW, msgwin::CLASSES[0]),
            (TALK, talk::CLASS),
            (UTIL, talk::UTIL_CLASS),
        ]);
        let found = |at: &[i64], partial: bool| Found {
            heap: at.iter().map(|at| (*at, classes[at].to_string())).collect(),
            partial,
            ..Found::default()
        };
        let emu = FakeEmulator::new(served(&self.memory));
        let rpc = connect(&emu);
        let memory: Rc<dyn Memory> = Rc::new(RpcMemory::new(rpc));
        let names = Rc::new(ClassNames::new(memory.clone(), CODE));
        let profile = SunProfile::new(
            memory,
            names,
            Rc::new(Speech::plain(self.recorder.clone())),
            Rc::new(|_line: &str| {}),
        );
        profile.track(&found(swept, false)).unwrap();
        if !then.is_empty() {
            profile.track(&found(then, true)).unwrap();
        }
        // A first page with no name waits `SPEAKER_POLLS` for one: polled through them.
        for _ in 0..SPEAKER_POLLS + 1 {
            profile.poll_dialogue().unwrap();
        }
        self.recorder.spoken()
    }
}

#[test]
fn test_a_page_is_said_with_the_name_on_its_plate() {
    let scene = Scene::new();
    scene.message(
        &format!("Ahhh! Can't you just feel that warmth?{SCROLL}\nIt's so warm and bright here!"),
        "Mom",
    );
    assert_eq!(
        scene.said(&[(0, "Ahhh!")], &[WINDOW, TALK], &[]),
        ["Mom: Ahhh! Can't you just feel that warmth?"]
    );
}

/// A scrolled page leaves the line before it in the other box: "The first day spent under
/// Alola's sun!" stayed while "It's so warm and bright here!" was typed (spike 111). A window up
/// before reading starts is joined at the page on screen.
#[test]
fn test_the_page_on_screen_is_the_one_its_typed_line_begins() {
    let scene = Scene::new();
    scene.message(
        &format!(
            "Ahhh! Can't you just feel that warmth?\nThe first day spent!{SCROLL}\nIt's so warm and bright here!"
        ),
        "",
    );
    assert_eq!(
        scene.said(
            &[(0, "It's so"), (1, "The first day spent!")],
            &[WINDOW, TALK],
            &[]
        ),
        ["It's so warm and bright here!"]
    );
}

#[test]
fn test_a_talk_window_showing_another_window_names_nobody() {
    let scene = Scene::new();
    scene.message("Hang on, now. Gimme just a sec...", "Mom");
    scene
        .memory
        .u32(UTIL + talk::UTIL_WINDOW, &[WINDOW + 0x1000]);
    assert_eq!(
        scene.said(&[(0, "Hang")], &[WINDOW, TALK], &[]),
        ["Hang on, now. Gimme just a sec..."]
    );
}

/// The field's util outlives each conversation, and points at the window the moment it is
/// built; the sweeps reached Mom's first window six seconds after its first page (spike 111).
#[test]
fn test_a_window_its_util_holds_is_read_before_any_sweep_finds_it() {
    let scene = Scene::new();
    scene.message("Ahhh! Can't you just feel that warmth?", "Mom");
    assert_eq!(
        scene.said(&[(0, "Ahhh!")], &[TALK, UTIL], &[]),
        ["Mom: Ahhh! Can't you just feel that warmth?"]
    );
}

const FILE_51: [&str; 19] = [
    "",
    "Use the +Control Pad to select a language.",
    "",
    "Would you like to start the game?",
    "Play the game in",
    "Selected language",
    "",
    "",
    "English",
    "Spanish",
    "French",
    "German",
    "Italian",
    "Japanese",
    "Korean",
    "Simplified Chinese Characters",
    "Traditional Chinese Characters",
    "Begin game",
    "Select language",
];
const DRAW: i64 = 0x302EF5B0;

/// The language choice's upper screen, and a reader of it with file 51's words.
struct Choosing {
    memory: Rc<FakeMemory>,
    reader: LangSelectReader,
}

impl Choosing {
    fn new() -> Choosing {
        let memory = FakeMemory::new();
        let draw_class = FakeClasses::new(memory.clone()).add(langselect::CLASS);
        memory.u32(DRAW, &[draw_class]);
        let text = SunText {
            files: IndexMap::from([(
                langselect::TEXT_FILE,
                FILE_51.iter().map(|line| line.to_string()).collect(),
            )]),
        };
        let mut reader = LangSelectReader::new(
            memory.clone(),
            Rc::new(ClassNames::new(memory.clone(), CODE)),
            Some(Rc::new(text)),
        );
        reader.track([DRAW], false);
        Choosing { memory, reader }
    }

    fn show(&mut self, screen: i64, language: i64, choice: i64) -> Option<Shown> {
        self.memory
            .write(DRAW + langselect::SCREEN, &[screen as u8]);
        self.memory
            .write(DRAW + langselect::LANGUAGE, &[language as u8, choice as u8]);
        self.reader.poll().unwrap()
    }
}

fn focus(label: &str, index: i64, count: i64, opened: bool) -> Focus {
    Focus {
        source: DRAW,
        label: label.to_string(),
        index,
        count,
        opened,
    }
}

#[test]
fn test_the_language_list_says_its_title_the_language_under_its_cursor_and_its_help() {
    let mut choosing = Choosing::new();
    let shown = choosing.show(langselect::LIST, 2, 0);
    assert_eq!(
        shown,
        Some(Shown {
            title: Some("Play the game in".to_string()),
            focus: focus("French", 2, 9, true),
            help: Some("Use the +Control Pad to select a language.".to_string()),
        })
    );
    assert_eq!(choosing.show(langselect::LIST, 2, 0), None);
    assert_eq!(
        choosing.show(langselect::LIST, 8, 0),
        Some(Shown {
            title: None,
            focus: focus("Traditional Chinese Characters", 8, 9, false),
            help: None,
        })
    );
}

#[test]
fn test_the_confirmation_says_the_language_chosen_and_then_each_choice_moved_to() {
    let mut choosing = Choosing::new();
    choosing.show(langselect::LIST, 0, 0);
    let shown = choosing.show(langselect::CONFIRM, 0, 0);
    assert_eq!(
        shown,
        Some(Shown {
            title: Some("Selected language: English".to_string()),
            focus: focus("Begin game", 0, 2, true),
            help: Some("Would you like to start the game?".to_string()),
        })
    );
    assert_eq!(
        choosing.show(langselect::CONFIRM, 0, 1),
        Some(Shown {
            title: None,
            focus: focus("Select language", 1, 2, false),
            help: None,
        })
    );
}

#[test]
fn test_nothing_is_said_once_the_choice_is_gone() {
    let mut choosing = Choosing::new();
    choosing.show(langselect::LIST, 0, 0);
    choosing.memory.u32(DRAW, &[0]);
    assert_eq!(choosing.show(langselect::LIST, 1, 0), None);
}

/// A sweep of the blocks just allocated says what is new and nothing of what is gone: taken as
/// everything, it would forget the talk window, and the page its speaker (spike 123).
#[test]
fn test_a_window_the_heaps_just_allocated_is_read_beside_the_talk_window_a_sweep_found() {
    let scene = Scene::new();
    scene.message("Ahhh! Can't you just feel that warmth?", "Mom");
    assert_eq!(
        scene.said(&[(0, "Ahhh!")], &[TALK], &[WINDOW]),
        ["Mom: Ahhh! Can't you just feel that warmth?"]
    );
}

/// A small block freed and its memory taken again as part of a large one, which may be the
/// GPU's, is read no more from the look the watch sees it (spike 123).
#[test]
fn test_a_small_block_is_out_of_bounds_once_a_larger_one_is_taken_over_it() {
    let live = heaps::SmallBlocks::new();
    live.walked(&[(0x30000010, 0x30000110), (0x30000130, 0x30000230)]);
    let taken = Block {
        heap: 0,
        address: 0x30000120,
        size: 0x40000,
    };
    let fresh = Block {
        heap: 0,
        address: 0x30100000,
        size: 0x80,
    };
    assert_eq!(live.allocated(&[taken, fresh]), [0x30000120]);
    assert_eq!(
        *live.spans(),
        [(0x30000010, 0x30000110), (0x30100010, 0x30100090)]
    );
    assert!(!live.holds((0x30000130, 0x30000230)));
    assert_eq!(heaps::to_sweep(taken), None);
}

/// A block freed and taken again as it was is new to the watch only if it was forgotten
/// (sun_3.ctm, 2026-09-27).
#[test]
fn test_a_walk_names_the_blocks_it_no_longer_found_for_the_watch_to_forget() {
    let live = heaps::SmallBlocks::new();
    live.walked(&[(0x30000010, 0x30000110), (0x30000130, 0x30000230)]);
    assert_eq!(live.walked(&[(0x30000130, 0x30000230)]), [0x30000000]);
    assert_eq!(*live.spans(), [(0x30000130, 0x30000230)]);
}

/// GameSys::GameProcManager's +0x10 is zero between two apps, and elsewhere once the next has
/// started: the title, the intro and the field over sun_4.ctm (spike 123).
#[test]
fn test_another_app_is_said_to_start_when_the_manager_points_somewhere_new() {
    let memory = FakeMemory::new();
    let manager_class = FakeClasses::new(memory.clone()).add(apps::MANAGER_CLASS);
    let manager = 0x30010000;
    memory.u32(manager, &[manager_class]);
    let mut running = apps::RunningApp::new(memory.clone(), names(&memory));
    running.track([manager]).unwrap();
    memory.u32(manager + apps::RUNNING, &[0x3417E7A4]);
    assert_eq!(running.poll().unwrap(), Some((0x3417E7A4, None)));
    assert_eq!(running.poll().unwrap(), None);
    memory.u32(manager + apps::RUNNING, &[0]);
    assert_eq!(running.poll().unwrap(), None);
    memory.u32(manager + apps::RUNNING, &[0x3417E790]);
    assert_eq!(running.poll().unwrap(), Some((0x3417E790, None)));
}

/// The manager points at a record in heap 6, whose +0x0c is the process, whose vtable names it;
/// a pointer into no small block names nothing rather than read past a block (ADR-0035).
#[test]
fn test_the_app_started_is_named_through_the_record_the_manager_points_at() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let manager_class = classes.add(apps::MANAGER_CLASS);
    let proc_class = classes.add("N5Field9FieldProcE");
    let (manager, process, record) = (0x30010000, 0x3417E714, 0x3417E790);
    memory.u32(manager, &[manager_class]);
    memory.u32(process, &[proc_class]);
    memory.u32(record, &[2]);
    memory.u32(record + apps::RECORD_PROCESS, &[process]);
    let all: Blocks = Rc::new(|| {
        Rc::new(vec![
            (0x30010000, 0x30010040),
            (0x3417E6F4, 0x3417E760),
            (0x3417E770, 0x3417E7A0),
        ])
    });
    let mut running = apps::RunningApp::with(memory.clone(), names(&memory), Some(all));
    running.track([manager]).unwrap();
    memory.u32(manager + apps::RUNNING, &[record]);
    assert_eq!(
        running.poll().unwrap(),
        Some((record, Some("N5Field9FieldProcE".to_string())))
    );
    // Started before the watch has handed its blocks over: named the poll they can be read.
    let blocks: Rc<RefCell<Rc<Vec<Span>>>> =
        Rc::new(RefCell::new(Rc::new(vec![(0x30010000, 0x30010040)])));
    let held: Blocks = {
        let blocks = blocks.clone();
        Rc::new(move || blocks.borrow().clone())
    };
    let mut late = apps::RunningApp::with(memory.clone(), names(&memory), Some(held));
    late.track([manager]).unwrap();
    memory.u32(manager + apps::RUNNING, &[record]);
    assert_eq!(late.poll().unwrap(), Some((record, None)));
    assert_eq!(late.late_name().unwrap(), None);
    *blocks.borrow_mut() = Rc::new(vec![
        (0x30010000, 0x30010040),
        (0x3417E6F4, 0x3417E760),
        (0x3417E770, 0x3417E7A0),
    ]);
    assert_eq!(
        late.late_name().unwrap(),
        Some("N5Field9FieldProcE".to_string())
    );
    assert_eq!(late.late_name().unwrap(), None);
    memory.u32(manager + apps::RUNNING, &[0x33000000]);
    assert_eq!(running.poll().unwrap(), Some((0x33000000, None)));
}

/// Every cutscene is an `AppDemoProc`, and its +0x24 says which: 6 the chase after Kukui's call
/// (sun_1.ctm). None before it is named, or once another app runs.
#[test]
fn test_a_demo_says_which_it_plays_once_it_is_named() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let manager = 0x30010000;
    memory.u32(manager, &[classes.add(apps::MANAGER_CLASS)]);
    let (process, record) = (0x3417E714, 0x3417E790);
    memory.u32(process, &[classes.add(apps::DEMO)]);
    memory.u32(process + apps::DEMO_NUMBER, &[6]);
    memory.u32(record + apps::RECORD_PROCESS, &[process]);
    let blocks: Rc<RefCell<Rc<Vec<Span>>>> =
        Rc::new(RefCell::new(Rc::new(vec![(0x30010000, 0x30010040)])));
    let held: Blocks = {
        let blocks = blocks.clone();
        Rc::new(move || blocks.borrow().clone())
    };
    let mut running = apps::RunningApp::with(memory.clone(), names(&memory), Some(held));
    running.track([manager]).unwrap();
    memory.u32(manager + apps::RUNNING, &[record]);
    assert_eq!(running.poll().unwrap(), Some((record, None)));
    assert_eq!(running.demo().unwrap(), None);
    *blocks.borrow_mut() = Rc::new(vec![
        (0x30010000, 0x30010040),
        (0x3417E6F4, 0x3417E760),
        (0x3417E770, 0x3417E7A0),
    ]);
    assert_eq!(running.late_name().unwrap(), Some(apps::DEMO.to_string()));
    assert_eq!(running.demo().unwrap(), Some(6));
    memory.u32(manager + apps::RUNNING, &[0x33000000]);
    running.poll().unwrap();
    assert_eq!(running.demo().unwrap(), None);
}
