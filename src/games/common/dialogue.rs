//! Dialogue: announce each page of a message the moment the game starts to type it.
//!
//! A message window (`print::MsgWin`, or its base `gfl::str::StrWin`) holds the whole message in
//! a `gfl::str::StrBuf`, several pages of it, and lays out one page at a time into
//! `nw::lyt::TextBox` line objects — one per line, each holding its line as far as the typewriter
//! has got (spike 14).
//! So the whole of a page can be spoken the instant its first character appears, and which page
//! that is can be read rather than guessed: it is the page whose line the typed text is the start
//! of.
//!
//! Several windows hold text at once, most of them left over from earlier screens. A window is live
//! while every line object is still a TextBox (spike 13) and none has been taken by another
//! window. Each line object names the tag processor that lays it out, and a window's own is the
//! `print::XYTagProc` it embeds at +0x9c (spikes 17 and 27).
//!
//! A screen can also write text straight into a window's line objects under a tag processor of its
//! own, leaving the window's message alone: the options screen writes each option's description
//! that way (spike 27). That is announced as written text, whenever it changes.
//!
//! Everything the game shows is heard, every time it shows it. Talking to the same person twice
//! puts identical words in the same buffer, and the second conversation is as audible as the first
//! (decided 2026-09-13, reversing "identical text is never repeated").
//!
//! Bound by:
//! - ADR-0011: a message is said a page at a time, as the page starts to type.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{u16, u32};
use crate::core::heaps::IN_USE;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::layout::{Layouts, Placement};
use crate::games::common::text::{Line, NEWLINE, Page, pages};
use crate::games::common::{lyt, msgwin, strbuf};
use crate::py::{self, Error, Kind, PySet, Result};

/// Halfwords. The longest message seen is under 400; this only rejects garbage lengths.
pub const MAX_MESSAGE: i64 = 0x1000;
/// Polls a box seen showing its page must stay off screen before its return is a new showing. About
/// 50 ms at 60 Hz: a sign's box is off for over a second between reads (spike 58), and Birch's box
/// flickered off for one poll between pages.
pub const AWAY_POLLS: i64 = 3;
/// TextBoxes watched for a new window typing into them. The field's box has two lines, and the
/// opening's windows handed theirs on; this only stops the set growing without end.
pub const MAX_LINE_HOSTS: i64 = 16;

/// What one game calls its message windows, and where its line objects keep what is read.
///
/// ORAS's and X's by default. Sun's message windows are ORAS's field for field under other names,
/// and its TextBoxes hold every field 0x30 earlier (spike 111).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shapes {
    pub windows: &'static [&'static str],
    pub text: i64,
    pub tag_processor: i64,
    pub capacity: i64,
    pub length: i64,
    /// How far before a window the SDK's block header is, which reads 'UD' while the window lives,
    /// or None where that was never measured. A freed window keeps its base class's vtable, so the
    /// class check passes it, and its lines lead into memory the next screen may give the GPU:
    /// reading them crashed Azahar after Groudon's capture (spikes 168 and 169).
    pub block: Option<i64>,
}

impl Shapes {
    /// A line object from its text pointer to its length, in one read.
    pub fn typed_span(&self) -> i64 {
        self.length + 2 - self.text
    }
}

/// The SDK's block header (0x10) and Game Freak's (0x20) sit in front of each window (spike 110).
pub const ORAS_SHAPES: Shapes = Shapes {
    windows: &msgwin::CLASSES,
    text: lyt::TEXT,
    tag_processor: lyt::TAG_PROCESSOR,
    capacity: lyt::CAPACITY,
    length: lyt::LENGTH,
    block: Some(0x30),
};

/// A page the game has started to show, ready to speak.
///
/// Equal when everything but `why` is: Python's `field(compare=False)`.
#[derive(Debug, Clone)]
pub struct PageShown {
    pub window: i64,
    pub index: i64,
    pub text: String,
    /// For the log: why this page was reached. `next` for the page after one already said; for a
    /// first page, what started the showing — a new message, or the same one again and how.
    pub why: String,
}

impl PageShown {
    /// `PageShown(window, index, text)`: `why` left at its default, "".
    pub fn new(window: i64, index: i64, text: impl Into<String>) -> Self {
        PageShown {
            window,
            index,
            text: text.into(),
            why: String::new(),
        }
    }
}

impl PartialEq for PageShown {
    fn eq(&self, other: &Self) -> bool {
        (self.window, self.index, &self.text) == (other.window, other.index, &other.text)
    }
}

/// Text a screen wrote straight into a window's line objects, ready to speak.
#[derive(Debug, Clone, PartialEq)]
pub struct TextWritten {
    pub window: i64,
    pub text: String,
}

/// `PageShown | TextWritten`: what [`DialogueReader::poll`] reports.
#[derive(Debug, Clone, PartialEq)]
pub enum Shown {
    Page(PageShown),
    Text(TextWritten),
}

struct Progress {
    /// True for a window that was already up when reading started, or one back on screen: its
    /// message is joined at the page on screen rather than read from the top, because the player
    /// has seen the rest.
    join: bool,
    message: Option<Vec<u8>>,
    buffer: i64,
    pages: Vec<Page>,
    /// Index of the last page announced in this showing of the message; -1 before the first.
    said: i64,
    /// The lines as last seen with anything typed in them.
    typed: Vec<Line>,
    /// What started the current showing, for the log.
    why: String,
    /// Lines written under another tag processor, as last polled.
    written: Vec<Line>,
    /// The written lines last announced.
    written_said: Vec<Line>,
    /// Whether the box has been seen on screen showing the page last said.
    seen: bool,
    /// Polls in a row the box has been off screen since it was seen showing that page.
    hidden: i64,
}

impl Progress {
    /// `_Progress(join=join)`: every other field at its default.
    fn new(join: bool) -> Self {
        Progress::with_why(join, "first seen")
    }

    /// `_Progress(join=join, why=why)`.
    fn with_why(join: bool, why: &str) -> Self {
        Progress {
            join,
            message: None,
            buffer: 0,
            pages: Vec::new(),
            said: -1,
            typed: Vec::new(),
            why: why.to_string(),
            written: Vec::new(),
            written_said: Vec::new(),
            seen: false,
            hidden: 0,
        }
    }
}

/// `head[at]`, raising `IndexError` where `head` stops short of it.
#[track_caller]
fn byte(head: &[u8], at: i64) -> Result<i64> {
    match usize::try_from(at).ok().and_then(|at| head.get(at)) {
        Some(&found) => Ok(i64::from(found)),
        None => Err(Error::index("index out of range")),
    }
}

/// Tracks message windows and reports each new page, and any text written over them.
pub struct DialogueReader {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    layouts: Option<Rc<Layouts>>,
    shapes: Shapes,
    // A window's progress is held apart from the reader while it is polled, which changes the
    // reader's other fields: the `Rc` is the Python's one object in the dict.
    progress: IndexMap<i64, Rc<RefCell<Progress>>>,
    active: Vec<i64>,
    started: bool,
    /// Window -> the line objects its records pointed at when last polled.
    lines: IndexMap<i64, Vec<i64>>,
    /// TextBoxes some window has typed into, oldest first, watched for a window not found yet.
    /// A window names itself in its lines' tag processor as it starts typing, where the next sweep
    /// found it up to 2 s later: the first sign in signs.ctm was said after its box closed (spike
    /// 60).
    hosts: IndexMap<i64, ()>,
}

impl DialogueReader {
    /// `DialogueReader(memory, names, layouts)`, with Python's default `shapes`, [`ORAS_SHAPES`].
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Option<Rc<Layouts>>,
    ) -> Self {
        DialogueReader::with_shapes(memory, names, layouts, ORAS_SHAPES)
    }

    /// `DialogueReader(memory, names, layouts, shapes)`.
    pub fn with_shapes(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Option<Rc<Layouts>>,
        shapes: Shapes,
    ) -> Self {
        DialogueReader {
            memory,
            names,
            layouts,
            shapes,
            progress: IndexMap::new(),
            active: Vec::new(),
            started: false,
            lines: IndexMap::new(),
            hosts: IndexMap::new(),
        }
    }

    /// Read these windows from now on: in place of those read so far, or, `adding` those a
    /// sweep of the blocks just allocated found, beside them (spike 110).
    ///
    /// Progress is kept for every window ever tracked, not just these. A sweep taken while a scene
    /// is loading can miss a window that is back a moment later, and forgetting where it had got
    /// to would skip or repeat pages when it returns (measured: whole pages lost after a battle).
    pub fn track(&mut self, windows: &[i64], adding: bool) {
        if adding {
            for &window in windows {
                if !self.active.contains(&window) {
                    self.active.push(window);
                    self.progress
                        .entry(window)
                        .or_insert_with(|| Rc::new(RefCell::new(Progress::new(!self.started))));
                }
            }
            return;
        }
        self.active = windows.to_vec();
        for &address in &self.active {
            self.progress
                .entry(address)
                .or_insert_with(|| Rc::new(RefCell::new(Progress::new(!self.started))));
        }
        self.started = true;
        self.lines.retain(|window, _| windows.contains(window));
    }

    /// Watch these TextBoxes, which a message window is typing into, for windows not found yet.
    ///
    /// The screen text reader sees every TextBox a layout holds, so it meets a message box's
    /// lines before any sweep has found the window typing into them.
    pub fn watch(&mut self, boxes: &[i64]) {
        for &r#box in boxes {
            self.hosts.shift_remove(&r#box);
            self.hosts.insert(r#box, ());
        }
        while self.hosts.len() as i64 > MAX_LINE_HOSTS {
            self.hosts.shift_remove_index(0);
        }
    }

    /// Stop watching every TextBox, as an app ends. The field's message box keeps its lines
    /// from one window to the next, but a battle's or the Pokédex's go with the app, into memory
    /// the next app may give the GPU, and reading them crashed Azahar after Groudon's capture
    /// (spikes 168 and 169). The screen text reader hands the new app's over as they are typed
    /// in.
    pub fn forget_hosts(&mut self) {
        self.hosts.clear();
    }

    pub fn poll(&mut self) -> Result<Vec<Shown>> {
        if let Some(layouts) = &self.layouts {
            layouts.refresh();
        }
        self.find_typing()?;
        let mut shown: Vec<Shown> = Vec::new();
        for address in self.active.clone() {
            let progress = self.progress[&address].clone();
            shown.extend(self.poll_window(address, &mut progress.borrow_mut())?);
        }
        Ok(shown)
    }

    /// The message windows read at the last poll.
    pub fn windows(&self) -> Vec<i64> {
        self.active.clone()
    }

    /// The TextBoxes tracked windows lay their pages out in, as of the last poll.
    ///
    /// They are layout TextBoxes like any other, so the screen text reader would see every
    /// character typed into them; it leaves these to this reader, which says them a page at a time.
    ///
    /// Python returns a `set`, iterated by the reader of screen text; `PySet` iterates it as
    /// Python does.
    pub fn line_objects(&self) -> PySet {
        PySet::of(self.lines.values().flat_map(|lines| lines.iter().copied()))
    }

    /// Python's `_poll`, renamed because `poll` is the public one.
    fn poll_window(&mut self, address: i64, progress: &mut Progress) -> Result<Vec<Shown>> {
        let before = self.shapes.block.unwrap_or(0);
        let head = self.memory.read(address - before, before + msgwin::HEAD)?;
        if self.shapes.block.is_some() && u16(&head, 0)? != IN_USE {
            // Its lines are no longer to be read, not even to see whether they are TextBoxes. They
            // stay watched: the field's box keeps its lines for the next window (`forget_hosts`).
            self.lines.shift_remove(&address);
            return Ok(gone(progress, "again after the window closed"));
        }
        let head = head[before as usize..].to_vec();
        // A window freed since a sweep found it is memory something else now holds, and its
        // pointers lead anywhere: a read that fails loses the whole poll.
        let class = self.names.of_vtable(u32(&head, 0)?)?;
        if !class.is_some_and(|class| self.shapes.windows.contains(&class.as_str())) {
            return Ok(Vec::new());
        }
        let count = byte(&head, msgwin::LINE_COUNT)?;
        if !(0 < count && count <= msgwin::MAX_LINES) {
            // Not proof of anything: the count is rebuilt as a page is laid out.
            return Ok(Vec::new());
        }
        let records = self
            .memory
            .read(u32(&head, msgwin::LINES)?, msgwin::LINE_RECORD * count)?;
        let mut boxes: Vec<i64> = Vec::new();
        for i in 0..count {
            boxes.push(u32(&records, i * msgwin::LINE_RECORD)?);
        }
        for &r#box in &boxes {
            if self.names.of_object(r#box)?.as_deref() != Some(lyt::TEXTBOX_CLASS) {
                self.lines.shift_remove(&address);
                return Ok(gone(progress, "again after the window was rebuilt"));
            }
        }
        self.lines.insert(address, boxes.clone());
        self.watch(&boxes);

        let own = address + msgwin::TAG_PROCESSOR;
        let mut typed: Vec<(i64, Line)> = Vec::new();
        for &r#box in &boxes {
            typed.push(self.typed(r#box)?);
        }
        if typed.iter().all(|(tag, _)| *tag == 0) {
            return Ok(gone(progress, "again after the window closed"));
        }
        for (tag, _) in &typed {
            if *tag != 0 && *tag != own && self.window_of(*tag)?.is_some() {
                return Ok(gone(progress, "again after its lines were taken over"));
            }
        }

        let buffer = u32(&head, msgwin::MESSAGE)?;
        let message = strbuf::raw(&*self.memory, buffer, MAX_MESSAGE)?;
        if progress.message.as_deref() != Some(message.as_slice()) || buffer != progress.buffer {
            let pages_of = match pages(&message) {
                Ok(pages_of) => pages_of,
                Err(error) if error.is(Kind::ValueError) => return Ok(Vec::new()),
                Err(error) => return Err(error),
            };
            if let Some(before) = &progress.message {
                progress.join = false;
                let same = &message == before;
                progress.why = if same {
                    "again in a new buffer"
                } else {
                    "new message"
                }
                .to_string();
            }
            progress.pages = pages_of;
            (progress.message, progress.buffer, progress.said) = (Some(message), buffer, -1);
        }

        let written: Vec<Line> = typed
            .iter()
            .filter(|(tag, line)| {
                *tag != 0 && *tag != own && !line.is_empty() && !in_message(&progress.pages, line)
            })
            .map(|(_, line)| line.clone())
            .collect();
        if !written.is_empty() {
            let still_own = typed.iter().any(|(tag, _)| *tag == own);
            return written_text(address, progress, written, still_own);
        }
        progress.written = Vec::new();
        if !progress.written_said.is_empty() {
            // The window's own message has come back over the written text: a showing of its own.
            progress.written_said = Vec::new();
            (progress.said, progress.join, progress.typed) = (-1, false, Vec::new());
            progress.why = "again after text was written over it".to_string();
        }

        let lines: Vec<Line> = typed.iter().map(|(_, line)| line.clone()).collect();
        let own_typing = typed
            .iter()
            .all(|(tag, line)| line.is_empty() || *tag == own);
        // A box that left the screen and came back is showing its message again. A signpost needs
        // this: its words appear whole rather than typed, so the typing never starts over, and
        // reading the same sign twice running was silent (signs.ctm, 2026-09-16). Its box was off
        // screen for over a second between reads, so a poll catches it. Joined at the page on
        // screen, which is where a box that comes back mid-message picks up.
        //
        // Only a box seen showing its page, then off screen for a few polls. Birch's box in the
        // opening flickers off between pages, the next page is typed while it is off, and taking
        // its return for a new showing said "one another out." twice, 20 ms apart (movie3 replay,
        // 2026-09-17).
        //
        // And only with every line off screen. The BuzzNav scrolls a show's words through two
        // lines, each line fading out for a few frames as it leaves the top and coming back at the
        // bottom; read by its first line alone, the box left and came back every two or three
        // seconds, and the whole show was said again each time (spike 192, movie25).
        let mut placements: Vec<Placement> = Vec::new();
        if let Some(layouts) = &self.layouts {
            for &r#box in &boxes {
                if let Some(placement) = layouts.placement(r#box)? {
                    placements.push(placement);
                }
            }
        }
        let shown = placements.iter().any(|placement| placement.shown);
        if !placements.is_empty() && !shown {
            if progress.seen {
                progress.hidden += 1;
            }
        } else if !placements.is_empty() {
            if progress.hidden >= AWAY_POLLS && progress.said >= 0 {
                progress.said = -1;
                progress.join = true;
                progress.why = "again after the box left the screen".to_string();
            }
            progress.hidden = 0;
            progress.seen = progress.said >= 0;
        }
        // Typing that starts over is the game saying the message again, whether or not the box left
        // the screen in between (reversing their own rule of 2026-09-14).
        // That rule was added for the options screen, which retypes the text speed description as a
        // demonstration in a box that never leaves; the cost of it was that reading a signpost
        // twice was sometimes silent, because a box that closes and reopens inside a poll is never
        // caught off screen. Silence is the worse fault: with nothing said, there is no telling
        // pa3ds having hung from the NPC having finished. The options screen repeating its
        // description is tolerable because nothing interrupts by default, so it follows the value
        // rather than cutting it off.
        if own_typing && restarted(&progress.pages, &progress.typed, &lines, progress.said) {
            progress.said = -1;
            progress.join = false;
            progress.why = "again, typed from the start".to_string();
        }
        if lines.iter().any(|line| !line.is_empty()) {
            progress.typed = lines.clone();
        }

        let on_screen: Vec<Line> = lines.into_iter().filter(|line| !line.is_empty()).collect();
        let current = current_page(&progress.pages, &on_screen, progress.said);
        let Some(current) = current.filter(|&current| current > progress.said) else {
            return Ok(Vec::new());
        };
        // Pages before the one on screen were shown too, just faster than a poll: say them all.
        // The exception is a message that was already up when reading started.
        let first = if progress.join && progress.said < 0 {
            current
        } else {
            progress.said + 1
        };
        let opening = progress.said < 0;
        progress.join = false;
        progress.seen = shown;
        progress.hidden = 0;
        progress.said = current;
        Ok((first..=current)
            .filter(|&index| !progress.pages[index as usize].text.is_empty())
            .map(|index| {
                Shown::Page(PageShown {
                    window: address,
                    index,
                    text: progress.pages[index as usize].text.clone(),
                    why: if opening && index == first {
                        progress.why.clone()
                    } else {
                        "next".to_string()
                    },
                })
            })
            .collect())
    }

    /// Read from now on any window typing into a watched TextBox that is not read already.
    ///
    /// Found this way, a window is read from its first page, as one a sweep finds after startup
    /// is. A sweep that does not find it yet does not lose it: its tag processor names it again
    /// at the next poll.
    fn find_typing(&mut self) -> Result<()> {
        for r#box in self.hosts.keys().copied().collect::<Vec<i64>>() {
            let head = self.memory.read(r#box, self.shapes.tag_processor + 4)?;
            if self.names.of_vtable(u32(&head, 0)?)?.as_deref() != Some(lyt::TEXTBOX_CLASS) {
                self.hosts.shift_remove(&r#box);
                continue;
            }
            let window = self.window_of(u32(&head, self.shapes.tag_processor)?)?;
            let Some(window) = window.filter(|window| !self.active.contains(window)) else {
                continue;
            };
            self.active.push(window);
            self.progress.entry(window).or_insert_with(|| {
                Rc::new(RefCell::new(Progress::with_why(false, "first seen typing")))
            });
        }
        Ok(())
    }

    /// (tag processor, halfwords typed so far). The tag processor is zero once closed.
    fn typed(&self, r#box: i64) -> Result<(i64, Line)> {
        let shapes = self.shapes;
        let span = self.memory.read(r#box + shapes.text, shapes.typed_span())?;
        let tag = u32(&span, shapes.tag_processor - shapes.text)?;
        let capacity = u16(&span, shapes.capacity - shapes.text)?;
        let length = u16(&span, shapes.length - shapes.text)?;
        if !(0 < length && length <= capacity.min(MAX_MESSAGE)) {
            return Ok((tag, Vec::new()));
        }
        let raw = self.memory.read(u32(&span, 0)?, length * 2)?;
        if raw.len() as i64 != length * 2 {
            return Err(Error::unpack(format!(
                "unpack requires a buffer of {} bytes",
                length * 2
            )));
        }
        let mut units: Line = Vec::new();
        for i in 0..length {
            units.push(u16(&raw, i * 2)?);
        }
        Ok((tag, units))
    }

    fn window_of(&self, tag: i64) -> Result<Option<i64>> {
        msgwin::window_of(&self.names, tag, self.shapes.windows)
    }
}

/// Nothing of this showing is on screen any more. Whatever the window shows next is heard from
/// its first page, even the same words again.
fn gone(progress: &mut Progress, why: &str) -> Vec<Shown> {
    if progress.said >= 0 {
        progress.why = why.to_string();
    }
    progress.said = -1;
    progress.join = false;
    progress.typed = Vec::new();
    progress.written = Vec::new();
    progress.written_said = Vec::new();
    Vec::new()
}

/// Python's `_written`, renamed because `poll_window` has a local `written`. Announce text written
/// over the window, once it has held for a poll and if it is new.
///
/// Holding for a poll keeps a line caught halfway through being written from being spoken. A
/// window with none of its own lines left is not showing anything of its own to be written over.
fn written_text(
    address: i64,
    progress: &mut Progress,
    written: Vec<Line>,
    still_own: bool,
) -> Result<Vec<Shown>> {
    let held = written == progress.written;
    progress.written = written.clone();
    if !held || !still_own || written == progress.written_said {
        return Ok(Vec::new());
    }
    progress.written_said = written.clone();
    let mut units: Vec<i64> = Vec::new();
    for (i, line) in written.iter().enumerate() {
        if i > 0 {
            units.push(NEWLINE);
        }
        units.extend(line);
    }
    let packed: Vec<u8> = units
        .iter()
        .flat_map(|&unit| (unit as u16).to_le_bytes())
        .collect();
    let text = match pages(&packed) {
        Ok(spoken) => spoken
            .iter()
            .map(|page| page.text.as_str())
            .collect::<Vec<&str>>()
            .join(" "),
        Err(error) if error.is(Kind::ValueError) => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    if py::strip(&text).is_empty() {
        return Ok(Vec::new());
    }
    Ok(vec![Shown::Text(TextWritten {
        window: address,
        text: py::split(&text).join(" "),
    })])
}

/// Could this typed line be the start of some line of the message?
fn in_message(message: &[Page], line: &[i64]) -> bool {
    message
        .iter()
        .any(|page| page.lines.iter().any(|candidate| starts(candidate, line)))
}

/// `candidate[:len(line)] == line`, which Python writes out wherever it needs it.
fn starts(candidate: &[i64], line: &[i64]) -> bool {
    py::slice(candidate, 0, line.len() as i64) == line
}

/// The furthest page any typed line belongs to, never looking back before the last one said.
///
/// Looking back is what would go wrong: "You" typed at the start of a later page is also the
/// start of an earlier one. A line left over from the page before a scroll matches that earlier
/// page and loses to the new line, which is why the furthest match wins.
fn current_page(message: &[Page], typed: &[Line], said: i64) -> Option<i64> {
    let floor = said.max(0);
    let mut best: Option<i64> = None;
    for line in typed {
        for index in floor..message.len() as i64 {
            if message[index as usize]
                .lines
                .iter()
                .any(|candidate| starts(candidate, line))
            {
                best = Some(best.map_or(index, |best| best.max(index)));
                break;
            }
        }
    }
    best
}

/// Is the same message being typed again from the start, in a window that never closed?
///
/// A second conversation with the same person can put identical text in the same buffer of the
/// same open window, so the message looks unchanged and only the typing gives it away. Typing
/// starts over: less is typed than last time, and every typed line is the start of the first line
/// but not all of it. Two things also leave less typed and are ruled out. A scroll empties the
/// recycled line, but what stays is a whole line. A cleared page can begin like the first page,
/// but then its text also matches a page after the one last said, and the page going forward wins.
///
/// Only typing under the window's own tag processor counts. The options screen retypes the text
/// speed description under its own each time the speed changes, as a demonstration, and a player
/// heard that as the description repeating over every choice (spike 27).
fn restarted(message: &[Page], before: &[Line], now: &[Line], said: i64) -> bool {
    if said < 0 || message.is_empty() || message[0].lines.is_empty() {
        return false;
    }
    if now.iter().map(Vec::len).sum::<usize>() >= before.iter().map(Vec::len).sum::<usize>() {
        return false;
    }
    let typed: Vec<&Line> = now.iter().filter(|line| !line.is_empty()).collect();
    let first = &message[0].lines[0];
    let starts_first = typed
        .iter()
        .all(|line| line.len() < first.len() && starts(first, line));
    if typed.is_empty() || !starts_first {
        return false;
    }
    let later: Vec<&Line> = py::slice_from(message, said + 1)
        .iter()
        .flat_map(|page| page.lines.iter())
        .collect();
    !typed
        .iter()
        .any(|line| later.iter().any(|candidate| starts(candidate, line)))
}
