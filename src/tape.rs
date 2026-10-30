//! A tape's answers, for checking the port against the Python runtime (`tools/tape/README.md`).
//!
//! A tape is what the emulator and the clock answered the Python runtime through one replay of a
//! movie, in the order it asked. [`TapeRpc`] and [`TapeClock`] give the Rust runtime the answers
//! the Python's `spikes/tape_replay.py replay` gives the Python runtime, by the same rules, so the
//! two can be held to each other's lines on a tape neither asks just as its recording did.
//!
//! The tape is read a tick at a time, a tick ending at the sleep that ends a pass of the poll
//! loop (`time::sleep_ending_tick`). Each question is answered from its own tick, and counted as
//! one of:
//!
//! - **exact**: the tick's oldest unanswered record of that question, as recorded, a raised error
//!   and the order a `read_each`'s replies came in included (its reads may be asked in any order);
//! - **covered**: bytes the tick read under another question: the latest read at or before the
//!   last exact answer, else the first after it;
//! - **stale**: bytes only an earlier tick read, the newest;
//! - **missing**: none, or only across two replies, which are never joined.
//!
//! A clock, `unsure` or `reply_seconds` asked more often than recorded gets the last value given.
//! Stale bytes are served, a missing read is left out of a `read_each` and fails a lone one, and
//! the tape is then approximate. What a tick recorded and never asked for is counted unused.
//!
//! Running out of tape unwinds with a [`Stop`]: the Python raised a `BaseException` there, which
//! no reader's `except Exception` takes for a failure, and the runtime passes a `Stop` on in the
//! same way (`is_stop`).

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::io::{BufReader, Read};
use std::path::Path;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::rpc::{Process, Rpc};
use crate::py::{Error, Kind, Result, time::Clock};

const MAGIC: &[u8] = b"PA3DSTAPE1\n";

const CLOCK: u8 = 1;
const SLEEP: u8 = 2;
const READ: u8 = 3;
const READ_EACH: u8 = 4;
const UNSURE: u8 = 5;
const REPLY_SECONDS: u8 = 6;
const CONFIRM: u8 = 7;
const ATTACH: u8 = 8;

/// How a question was answered, best first.
const CLASSES: [&str; 4] = ["exact", "covered", "stale", "missing"];
const EXACT: usize = 0;
const COVERED: usize = 1;
const STALE: usize = 2;
const MISSING: usize = 3;

/// The read cap (`core::rpc::MAX_READ`), the grain the bytes read are found by.
const PAGE: i64 = 1024;

fn name(kind: u8) -> &'static str {
    match kind {
        CLOCK => "clock",
        SLEEP => "sleep",
        READ => "read",
        READ_EACH => "read_each",
        UNSURE => "unsure",
        REPLY_SECONDS => "reply_seconds",
        CONFIRM => "confirm",
        ATTACH => "attach",
        _ => "record",
    }
}

/// Why a replay stopped: the payload of the unwind that stops it.
#[derive(Debug, Clone, PartialEq)]
pub enum Stop {
    /// Every answer on the tape was asked for: the replay ran as far as the recording did.
    End,
    /// The replay stopped short of the tape's end: the tape ends inside a record, or the runtime
    /// stopped or raised before it did.
    Diverged(String),
}

/// Whether a caught panic is a tape stopping the replay, which must unwind on past whatever
/// caught it.
pub fn is_stop(payload: &(dyn Any + Send)) -> bool {
    payload.is::<Stop>()
}

/// What a recorded call gave back.
#[derive(Clone)]
enum Reply {
    Float(f64),
    Flag(bool),
    Bytes(Rc<[u8]>),
    Each(Vec<(i64, Rc<[u8]>)>),
    Process(i64, i64, String),
    Raised(Kind, String),
}

/// A record's kind and question: (address, size) for a read, a `read_each`'s reads sorted, as a
/// batch built from a set is the same question in another order, and nothing for the rest.
type Question = (u8, Vec<(i64, i64)>);

struct State {
    source: Box<dyn Read>,
    ahead: VecDeque<(u8, Vec<(i64, i64)>, Reply)>,
    tick: i64,
    records: i64,
    /// Ticks the tape ends with a sleep before the loop's top: what `sleeps` must come to.
    boundaries: i64,
    /// The poll loop's sleeps, which end the replay's ticks.
    sleeps: i64,
    /// The tick's unanswered records by question, oldest first, each with its place in the tick.
    queue: HashMap<Question, VecDeque<(i64, Reply)>>,
    /// The bytes the tick read, by page, in tick order.
    fresh: HashMap<i64, Vec<(i64, i64, Rc<[u8]>)>>,
    /// The newest bytes earlier ticks read at each (address, length), by page, newest last.
    held: HashMap<i64, IndexMap<(i64, i64), Rc<[u8]>>>,
    /// The seconds of the sleep that ends the tape's tick, or None at the end of the tape.
    end: Option<f64>,
    /// The place of the last record answered exactly: where a covered read is judged from.
    cursor: i64,
    /// (tick, value) of the last clock, `unsure` and `reply_seconds` given; tick 0 is none.
    last: HashMap<u8, (i64, Reply)>,
    ended: bool,
    counts: HashMap<(&'static str, usize), i64>,
    unused: HashMap<&'static str, i64>,
    /// The first few stale or missing answers.
    problems: Vec<String>,
    first: Option<f64>,
    /// Seconds on the tape's clock since its first reading.
    now: f64,
}

/// One tape, read a tick at a time.
pub struct Tape {
    state: RefCell<State>,
    stopped: RefCell<Option<Stop>>,
    /// Whether what is asked now is the census of P's ways, answered aside (`aside`).
    aside: Cell<bool>,
}

impl Tape {
    pub fn open(path: &Path) -> Result<Rc<Tape>> {
        let mut source: Box<dyn Read> = Box::new(BufReader::new(flate2::read::GzDecoder::new(
            std::fs::File::open(path)?,
        )));
        let mut magic = vec![0; MAGIC.len()];
        if source.read_exact(&mut magic).is_err() || magic != MAGIC {
            return Err(Error::value(format!("{} is not a tape", path.display())));
        }
        let last = HashMap::from([
            (CLOCK, (0, Reply::Float(0.0))),
            (UNSURE, (0, Reply::Flag(false))),
            (REPLY_SECONDS, (0, Reply::Float(0.0))),
        ]);
        let tape = Rc::new(Tape {
            state: RefCell::new(State {
                source,
                ahead: VecDeque::new(),
                tick: 0,
                records: 0,
                boundaries: 0,
                sleeps: 0,
                queue: HashMap::new(),
                fresh: HashMap::new(),
                held: HashMap::new(),
                end: None,
                cursor: -1,
                last,
                ended: false,
                counts: HashMap::new(),
                unused: HashMap::new(),
                problems: Vec::new(),
                first: None,
                now: 0.0,
            }),
            stopped: RefCell::new(None),
            aside: Cell::new(false),
        });
        tape.load();
        Ok(tape)
    }

    /// Answer what is asked inside from what the tape holds, taking no record and counting
    /// nothing: the census of P's ways, which only a tape replay logs (`Settings.ways`,
    /// ADR-0041), so the replay's own answers stay its own. A read the tape holds nowhere raises
    /// as one missing does, and the clock stands where the replay left it.
    pub fn aside(&self, inside: &mut dyn FnMut() -> Result<()>) -> Result<()> {
        self.aside.set(true);
        let done = inside();
        self.aside.set(false);
        done
    }

    /// Records read off the tape.
    pub fn count(&self) -> i64 {
        self.state.borrow().records
    }

    /// Why the replay stopped, once it has.
    pub fn stopped(&self) -> Option<Stop> {
        self.stopped.borrow().clone()
    }

    /// "1234 ticks (1234 on the tape), 5678 records": the head of the Python's summary line.
    pub fn ticks(&self) -> String {
        let state = self.state.borrow();
        format!(
            "{} ticks ({} on the tape), {} records, {} s of tape",
            state.sleeps,
            state.boundaries,
            state.records,
            crate::py::fixed(state.now, 1)
        )
    }

    /// 'exact 33501, covered 0, stale 0, missing 0, unused 0', and each kind not exact.
    pub fn summary(&self) -> String {
        let state = self.state.borrow();
        let mut totals = [0i64; 4];
        for (&(_kind, how), &n) in &state.counts {
            totals[how] += n;
        }
        let mut odd: Vec<((&str, &str), i64)> = state
            .counts
            .iter()
            .filter(|&(&(_kind, how), _)| how != EXACT)
            .map(|(&(kind, how), &n)| ((kind, CLASSES[how]), n))
            .collect();
        odd.sort();
        let mut odd: Vec<String> = odd
            .into_iter()
            .map(|((kind, how), n)| format!("{kind} {how} {n}"))
            .collect();
        let mut unused: Vec<(&str, i64)> = state
            .unused
            .iter()
            .filter(|&(_, &n)| n != 0)
            .map(|(&kind, &n)| (kind, n))
            .collect();
        unused.sort();
        odd.extend(unused.iter().map(|(kind, n)| format!("{kind} unused {n}")));
        let mut out: Vec<String> = CLASSES
            .iter()
            .zip(totals)
            .map(|(how, n)| format!("{how} {n}"))
            .collect();
        out.push(format!("unused {}", state.unused.values().sum::<i64>()));
        let mut text = out.join(", ");
        if !odd.is_empty() {
            text += &format!(" ({})", odd.join(", "));
        }
        text
    }

    /// Whether any answer was stale or missing.
    pub fn approximate(&self) -> bool {
        self.state
            .borrow()
            .counts
            .keys()
            .any(|&(_kind, how)| how == STALE || how == MISSING)
    }

    /// The first few stale or missing answers.
    pub fn problems(&self) -> Vec<String> {
        self.state.borrow().problems.clone()
    }

    fn stop(&self, why: Stop) -> ! {
        *self.stopped.borrow_mut() = Some(why.clone());
        std::panic::panic_any(why)
    }

    // -- the tick ------------------------------------------------------------------------------

    /// End the tick, what it left unasked counted unused and its reads joined to the earlier
    /// ticks', and read the next one off the tape up to the sleep that ends it.
    fn load(&self) {
        let mut state = self.state.borrow_mut();
        let state = &mut *state;
        for ((kind, _key), left) in &state.queue {
            *state.unused.entry(name(*kind)).or_insert(0) += left.len() as i64;
        }
        for (page, holds) in std::mem::take(&mut state.fresh) {
            let slot = state.held.entry(page).or_default();
            for (_place, address, data) in holds {
                // Put back last: a slot's order is its age.
                let key = (address, data.len() as i64);
                slot.shift_remove(&key);
                slot.insert(key, data);
            }
        }
        state.queue.clear();
        state.end = None;
        state.cursor = -1;
        state.tick += 1;
        let mut place = 0;
        while let Some((kind, key, reply)) = self.next(state) {
            state.records += 1;
            if kind == SLEEP && self.ends_tick(state) {
                if let Reply::Float(seconds) = reply {
                    state.end = Some(seconds);
                }
                state.boundaries += 1;
                break;
            }
            match &reply {
                Reply::Bytes(data) => hold(&mut state.fresh, place, key[0].0, data.clone()),
                Reply::Each(replies) => {
                    for (address, data) in replies {
                        hold(&mut state.fresh, place, *address, data.clone());
                    }
                }
                _ => {}
            }
            state
                .queue
                .entry((kind, key))
                .or_default()
                .push_back((place, reply));
            place += 1;
        }
        state.ended = place == 0 && state.end.is_none();
    }

    fn next(&self, state: &mut State) -> Option<(u8, Vec<(i64, i64)>, Reply)> {
        match state.ahead.pop_front() {
            Some(record) => Some(record),
            None => self.record(state),
        }
    }

    /// Whether the sleep just read is the poll loop's: the loop's top follows it, `started` and
    /// then `rpc.unsure`, or `confirm` while the emulator is silent. The tape's last sleep too.
    fn ends_tick(&self, state: &mut State) -> bool {
        while state.ahead.len() < 2 {
            match self.record(state) {
                Some(record) => state.ahead.push_back(record),
                None => break,
            }
        }
        let kinds: Vec<u8> = state.ahead.iter().map(|(kind, _, _)| *kind).collect();
        kinds.is_empty()
            || (kinds[0] == CLOCK && (kinds[1..] == [UNSURE] || kinds[1..] == [CONFIRM]))
    }

    /// The next record off the tape, or None at its end.
    fn record(&self, state: &mut State) -> Option<(u8, Vec<(i64, i64)>, Reply)> {
        let mut kind = [0u8];
        if state.source.read(&mut kind).unwrap_or(0) == 0 {
            return None;
        }
        let kind = kind[0];
        let mut stream = Stream {
            source: &mut *state.source,
            tape: self,
        };
        let record = match kind {
            CLOCK | SLEEP | REPLY_SECONDS => (kind, vec![], Reply::Float(stream.f64())),
            UNSURE => (kind, vec![], Reply::Flag(stream.u8() != 0)),
            READ => {
                let key = (stream.u32(), stream.u32());
                let reply = match stream.outcome() {
                    Some(raised) => raised,
                    None => Reply::Bytes(stream.blob().into()),
                };
                (kind, vec![key], reply)
            }
            READ_EACH => {
                let count = stream.u32() as usize;
                let mut key: Vec<(i64, i64)> =
                    (0..count).map(|_| (stream.u32(), stream.u32())).collect();
                key.sort_unstable();
                let reply = match stream.outcome() {
                    Some(raised) => raised,
                    None => {
                        let replies = stream.u32() as usize;
                        Reply::Each(
                            (0..replies)
                                .map(|_| (stream.u32(), stream.blob().into()))
                                .collect(),
                        )
                    }
                };
                (kind, key, reply)
            }
            CONFIRM => {
                let reply = match stream.outcome() {
                    Some(raised) => raised,
                    None => Reply::Flag(stream.u8() != 0),
                };
                (kind, vec![], reply)
            }
            ATTACH => {
                let reply = match stream.outcome() {
                    Some(raised) => raised,
                    None => {
                        let pid = stream.u32();
                        let title = stream.u64();
                        let name = String::from_utf8_lossy(&stream.blob()).into_owned();
                        Reply::Process(pid, title, name)
                    }
                };
                (kind, vec![], reply)
            }
            other => self.stop(Stop::Diverged(format!(
                "record kind {other}: a tape of a newer format"
            ))),
        };
        Some(record)
    }

    // -- the lookup ----------------------------------------------------------------------------

    /// (place, reply) of the tick's oldest unanswered record of this question, or None:
    /// always None aside, where nothing is taken.
    fn take(&self, kind: u8, key: Vec<(i64, i64)>) -> Option<(i64, Reply)> {
        if self.state.borrow().ended {
            self.stop(Stop::End);
        }
        if self.aside.get() {
            return None;
        }
        let mut state = self.state.borrow_mut();
        let taken = state.queue.get_mut(&(kind, key))?.pop_front()?;
        state.cursor = taken.0;
        Some(taken)
    }

    /// (covered or stale, bytes) of [address, address + size) out of one reply, or None.
    fn find(&self, address: i64, size: i64) -> Option<(usize, Vec<u8>)> {
        let state = self.state.borrow();
        let end = address + size;
        let page = crate::py::floordiv(address, PAGE);
        let slice =
            |at: i64, data: &[u8]| data[(address - at) as usize..(end - at) as usize].to_vec();
        let fresh: Vec<&(i64, i64, Rc<[u8]>)> = state
            .fresh
            .get(&page)
            .map(|holds| {
                holds
                    .iter()
                    .filter(|(_, at, data)| *at <= address && end <= at + data.len() as i64)
                    .collect()
            })
            .unwrap_or_default();
        if !fresh.is_empty() {
            let before = fresh.iter().rev().find(|hold| hold.0 <= state.cursor);
            let (_place, at, data) = before.copied().unwrap_or(fresh[0]);
            return Some((COVERED, slice(*at, data)));
        }
        if let Some(slot) = state.held.get(&page) {
            for (&(at, length), data) in slot.iter().rev() {
                if at <= address && end <= at + length {
                    return Some((STALE, slice(at, data)));
                }
            }
        }
        None
    }

    /// A clock, `unsure` or `reply_seconds`: the tick's next, or, asked more often than
    /// recorded, the last given: covered from this tick, as no time passes between two looks at
    /// it, stale from an earlier one.
    fn reading(&self, kind: u8) -> Reply {
        if let Some((_place, value)) = self.take(kind, vec![]) {
            self.tally(kind, EXACT, "");
            let mut state = self.state.borrow_mut();
            let tick = state.tick;
            state.last.insert(kind, (tick, value.clone()));
            return value;
        }
        let (tick, value) = self.state.borrow().last[&kind].clone();
        let now = self.state.borrow().tick;
        let how = if tick == now {
            COVERED
        } else if tick != 0 {
            STALE
        } else {
            MISSING
        };
        self.tally(kind, how, "");
        value
    }

    /// `confirm` and `attach`, exact or missing: neither is a reading to be guessed again.
    fn answer(&self, kind: u8) -> Result<Reply> {
        match self.take(kind, vec![]) {
            None => {
                self.tally(kind, MISSING, "");
                Err(not_on_the_tape())
            }
            Some((_place, reply)) => {
                self.tally(kind, EXACT, "");
                served(reply)
            }
        }
    }

    /// Count an answer; a stale or missing one is kept. Aside, none is.
    fn tally(&self, kind: u8, how: usize, what: &str) {
        if self.aside.get() {
            return;
        }
        let mut state = self.state.borrow_mut();
        *state.counts.entry((name(kind), how)).or_insert(0) += 1;
        if how != STALE && how != MISSING {
            return;
        }
        let asked = if what.is_empty() {
            name(kind).to_string()
        } else {
            format!("{} {what}", name(kind))
        };
        let problem = format!("tick {}: {asked}: {}", state.tick, CLASSES[how]);
        if state.problems.len() < 5 {
            state.problems.push(problem);
        }
    }
}

fn hold(
    fresh: &mut HashMap<i64, Vec<(i64, i64, Rc<[u8]>)>>,
    place: i64,
    address: i64,
    data: Rc<[u8]>,
) {
    let first = crate::py::floordiv(address, PAGE);
    let last = crate::py::floordiv(address + data.len() as i64 - 1, PAGE);
    for page in first..=last {
        fresh
            .entry(page)
            .or_default()
            .push((place, address, data.clone()));
    }
}

/// A tape taken a field at a time.
struct Stream<'a> {
    source: &'a mut dyn Read,
    tape: &'a Tape,
}

impl Stream<'_> {
    fn take(&mut self, n: usize) -> Vec<u8> {
        let mut chunk = vec![0; n];
        if self.source.read_exact(&mut chunk).is_err() {
            self.tape
                .stop(Stop::Diverged("the tape ends inside a record".into()));
        }
        chunk
    }

    fn u8(&mut self) -> u8 {
        self.take(1)[0]
    }

    fn u32(&mut self) -> i64 {
        u32::from_le_bytes(self.take(4).try_into().expect("4 bytes")) as i64
    }

    fn u64(&mut self) -> i64 {
        u64::from_le_bytes(self.take(8).try_into().expect("8 bytes")) as i64
    }

    fn f64(&mut self) -> f64 {
        f64::from_le_bytes(self.take(8).try_into().expect("8 bytes"))
    }

    fn blob(&mut self) -> Vec<u8> {
        let n = self.u32() as usize;
        self.take(n)
    }

    /// A record's outcome: None for a reply, which follows, or the error it raised.
    fn outcome(&mut self) -> Option<Reply> {
        let kind = match self.u8() {
            0 => return None,
            1 => Kind::RpcError,
            2 => Kind::RpcTimeoutError,
            3 => Kind::RpcRefusedError,
            4 => Kind::RpcLostError,
            _ => Kind::ValueError,
        };
        Some(Reply::Raised(
            kind,
            String::from_utf8_lossy(&self.blob()).into_owned(),
        ))
    }
}

/// A missing read, raised as memory that is not there, which the runtime already carries on from.
fn not_on_the_tape() -> Error {
    Error::new(Kind::RpcError, "not on the tape")
}

fn served(reply: Reply) -> Result<Reply> {
    match reply {
        Reply::Raised(kind, message) => Err(Error::new(kind, message)),
        reply => Ok(reply),
    }
}

/// The emulator, as the tape remembers it answering.
pub struct TapeRpc {
    tape: Rc<Tape>,
}

impl TapeRpc {
    pub fn new(tape: Rc<Tape>) -> Self {
        TapeRpc { tape }
    }
}

impl Rpc for TapeRpc {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        let tape = &self.tape;
        if let Some((_place, reply)) = tape.take(READ, vec![(address, size)]) {
            tape.tally(READ, EXACT, "");
            return match served(reply)? {
                Reply::Bytes(data) => Ok(data.to_vec()),
                _ => Ok(vec![]),
            };
        }
        let found = tape.find(address, size);
        let how = found.as_ref().map_or(MISSING, |(how, _)| *how);
        tape.tally(READ, how, &format!("{address:#x} {size}"));
        match found {
            None => Err(not_on_the_tape()),
            Some((_how, data)) => Ok(data),
        }
    }

    /// The batch is the same question as the recorded one in any order: the Python sends some
    /// batches in the order of a `set`, which Rust does not reproduce, and the order of a batch
    /// changes nothing but the order its replies arrive in.
    fn read_each(&self, reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        let tape = &self.tape;
        let mut key = reads.to_vec();
        key.sort_unstable();
        if let Some((_place, reply)) = tape.take(READ_EACH, key) {
            tape.tally(READ_EACH, EXACT, "");
            return match served(reply)? {
                Reply::Each(replies) => Ok(replies
                    .into_iter()
                    .map(|(address, data)| (address, data.to_vec()))
                    .collect()),
                _ => Ok(IndexMap::new()),
            };
        }
        let mut out = IndexMap::new();
        let (mut worst, mut what) = (COVERED, String::new());
        // A live burst sends `todo.pop()` first, so its replies come in the reverse of the
        // request.
        for &(address, size) in reads.iter().rev() {
            let found = tape.find(address, size);
            let how = found.as_ref().map_or(MISSING, |(how, _)| *how);
            if let Some((_how, data)) = found {
                out.insert(address, data);
            }
            if how > worst {
                worst = how;
                what = format!(", first {address:#x} {size}");
            }
        }
        tape.tally(READ_EACH, worst, &format!("of {}{what}", reads.len()));
        Ok(out)
    }

    fn reply_seconds(&self) -> f64 {
        match self.tape.reading(REPLY_SECONDS) {
            Reply::Float(value) => value,
            _ => 0.0,
        }
    }

    fn unsure(&self) -> bool {
        matches!(self.tape.reading(UNSURE), Reply::Flag(true))
    }

    fn confirm(&self) -> Result<bool> {
        Ok(matches!(self.tape.answer(CONFIRM)?, Reply::Flag(true)))
    }

    fn attach(&self) -> Result<Process> {
        match self.tape.answer(ATTACH)? {
            Reply::Process(pid, title_id, name) => Ok(Process {
                pid,
                title_id,
                name,
            }),
            _ => Err(not_on_the_tape()),
        }
    }
}

/// The clock, as the tape remembers it reading.
pub struct TapeClock {
    tape: Rc<Tape>,
}

impl TapeClock {
    pub fn new(tape: Rc<Tape>) -> Self {
        TapeClock { tape }
    }
}

impl Clock for TapeClock {
    fn monotonic(&mut self) -> f64 {
        let value = match self.tape.reading(CLOCK) {
            Reply::Float(value) => value,
            _ => 0.0,
        };
        let mut state = self.tape.state.borrow_mut();
        let first = *state.first.get_or_insert(value);
        state.now = value - first;
        value
    }

    /// Never waits. The poll loop's sleep ends the tick, against the one that ended the tape's;
    /// any other takes the tick's next sleep.
    fn sleep(&mut self, seconds: f64, ends_tick: bool) {
        let tape = &self.tape;
        if !ends_tick {
            let taken = tape.take(SLEEP, vec![]);
            let exact = matches!(taken, Some((_, Reply::Float(recorded))) if recorded == seconds);
            tape.tally(SLEEP, if exact { EXACT } else { COVERED }, "");
            return;
        }
        if tape.state.borrow().ended {
            tape.stop(Stop::End);
        }
        tape.state.borrow_mut().sleeps += 1;
        let exact = tape.state.borrow().end == Some(seconds);
        tape.tally(SLEEP, if exact { EXACT } else { COVERED }, "");
        tape.load();
    }
}
