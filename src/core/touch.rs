//! Touching the bottom screen, as a cemuhook (DSU) server Azahar reads a touchpad from.
//!
//! pa3ds announces and never acts, with one exception: a tap on the touch target pa3ds has just
//! named, when the player presses the key that asks for it (ADR-0009, amended 2026-10-01). A
//! pointer on the screen would have to land on a spot of a window whose size, layout and full
//! screen are the player's; this is the touch screen's own pixels instead.
//!
//! Azahar reads its touch screen from the mouse and, while the mouse is not pressing, from a
//! second device, `controller_touch_device`, which can be the cemuhook UDP client it keeps from
//! Citra (`input_common/udp`). That client asks a server on this machine, port 26760 by default,
//! for its first pad every three seconds, and takes each pad state the server sends back: a
//! touchpad pressed or not, at an x and a y it scales by the setting's calibration. With the
//! calibration 0 to 320 and 0 to 240 (`DEVICE`), pa3ds's x and y are the touch screen's pixels,
//! from its top left. The protocol is a public one, which DS4Windows and others speak; nothing of
//! Azahar's is used here.
//!
//! A tap is a press held `HOLD` seconds and a release: the game reads the touch screen once a
//! frame, so a press is a run of frames, and it is the change from released to pressed that a
//! button takes (Amethyst's touch module for Black and White does the same through BizHawk).
//! Azahar keeps the last state it was sent, so a finger is never left down: every tap ends in a
//! release, and closing the server sends one. A tap, slide or hold asked for while a press is
//! still down lets go for `LIFT` first: carried on, it was no new touch to the game.
//!
//! A slide is a press that moves, along a path walked evenly in time and let go at its end:
//! Pokémon-Amie is stroked so, and answers it with its hand on the Pokémon (spike 200). A tap is a
//! slide that stays on one point. ADR-0009 allowed pa3ds a tap alone; the slide is an exploration
//! asked for (2026-10-08), to keep or take back.
//!
//! A drag is a press the player holds between keys (`Drag`): held on one point, slid on to the
//! next in `MOVE` seconds each time the player moves it, and let go when they let go (on the
//! target it was sliding to, if it was), or after `HELD` seconds whatever happens, so a finger is
//! never left down for good.
//!
//! Bound by:
//! - ADR-0009: a touch, on the player's keys and on targets pa3ds has just named, is the one input
//!   pa3ds ever sends the game: a tap, or a drag the player holds.

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use flate2::Crc;
use indexmap::IndexMap;

use crate::core::words::own;
use crate::py::{self, Error, Kind, Result, time};

/// Azahar's `udp_input_port` as it ships.
pub const PORT: i64 = 26760;
pub const HOST: &str = "127.0.0.1";
/// Azahar's `controller_touch_device` for pa3ds's taps: the cemuhook client, calibrated so an x
/// and a y are the touch screen's own pixels (Azahar multiplies the share by 320 and 240).
pub const DEVICE: &str = "engine:cemuhookudp,max_x:320,max_y:240,min_x:0,min_y:0";
pub const WIDTH: i64 = 320;
pub const HEIGHT: i64 = 240;
/// How long a tap presses: six frames, a press a button takes as one.
pub const HOLD: f64 = 0.1;
/// How long the stylus is lifted before a tap, slide or hold that comes while a press is still
/// down. Carried on, the press was one touch to the game: Pokémon-Amie's Pokémon, its back to the
/// player, turns only to separate presses, and the stroke key pressed again within a stroke's three
/// seconds kept one press down for 16 s, Ralts turning away all along (as_amie_petting_issue,
/// spike 276).
pub const LIFT: f64 = HOLD;
/// How often the pad's state is sent while a client listens: twice a frame.
pub const TICK: f64 = 1.0 / 120.0;
/// How far the pad's counter climbs a second, by the clock. Azahar takes a state only when its
/// counter is past every one it has taken since it opened, from any server (`Client::OnPadData`,
/// `input_common/udp/client.cpp`): counting from one again, as each game read and each pa3ds
/// started in another's place did, a server had every tap dropped for as long as the one before
/// it had been sending (2026-10-20). Read off the clock, a server counts on from wherever any
/// before it got to; twice the states sent a second (`TICK`), so it never runs ahead of the
/// clock. The packet holds 32 bits of it, which wrap every 207 days: an Azahar open across that
/// moment drops taps until it is opened again.
pub const COUNTS: i64 = 240;
/// How long a client is sent states after its last request; it asks every three seconds.
pub const LISTENING: f64 = 6.0;
/// How long a held press takes to slide from one target to the next: a fifth of a second
/// (ADR-0009), not yet measured against a screen that takes a drop.
pub const MOVE: f64 = 0.2;
/// How long a press let go while it slides stays on its target before it lets go: two frames, so
/// the game reads it there.
pub const LAND: f64 = 2.0 / 60.0;
/// How long a held press is kept down at most, should its letting go never come.
///
/// A server keeps its own (`State.held_for`), which a test shortens where Python's sets the
/// module's `HELD`.
pub const HELD: f64 = 30.0;
/// How fast a rub goes back and forth, in the screen's pixels a second: Pokémon-Amie counts a
/// stroke only while the hand has moved at least 4 pixels since its script's last tick, and a swipe
/// of 30 a tick is something else (spike 287). 300 is 10 a tick at 30 ticks a second, 5 at 60.
pub const RUB_SPEED: f64 = 300.0;

/// Said for a tap Azahar is not listening for: its touch device is not pa3ds's.
pub const NOT_LISTENING: &str = "touch-not-listening";
pub const CLIENT: &[u8; 4] = b"DSUC";
pub const SERVER: &[u8; 4] = b"DSUS";
pub const VERSION: i64 = 1001;
pub const VERSION_TYPE: i64 = 0x100000;
pub const PORT_INFO: i64 = 0x100001;
pub const PAD_DATA: i64 = 0x100002;
/// The one pad this server has, as a port's info: slot 0, connected, full gyro, USB, a MAC of its
/// own, battery full, active.
pub const PAD: [u8; 12] = [0, 2, 2, 1, 0, 0, 0, 0, 0, 1, 5, 1];

/// Where the server's thread logs: the thread is the server's own, so the log is shared with it.
pub type SharedLog = Arc<dyn Fn(&str) + Send + Sync>;

/// `struct.pack("<I", value)`.
#[track_caller]
fn pack_u32(value: i64) -> Result<[u8; 4]> {
    u32::try_from(value)
        .map(u32::to_le_bytes)
        .map_err(|_| Error::unpack("'I' format requires 0 <= number <= 4294967295"))
}

/// `struct.pack("<H", value)`.
#[track_caller]
fn pack_u16(value: i64) -> Result<[u8; 2]> {
    u16::try_from(value)
        .map(u16::to_le_bytes)
        .map_err(|_| Error::unpack("'H' format requires 0 <= number <= 65535"))
}

/// A server's packet: its header, the type and the payload, with the CRC-32 of all three.
pub fn message(kind: i64, payload: &[u8], server: i64) -> Result<Vec<u8>> {
    let mut body: Vec<u8> = pack_u32(kind)?.to_vec();
    body.extend_from_slice(payload);
    let mut head: Vec<u8> = SERVER.to_vec();
    head.extend_from_slice(&pack_u16(VERSION)?);
    head.extend_from_slice(&pack_u16(body.len() as i64)?);
    head.extend_from_slice(&pack_u32(0)?);
    head.extend_from_slice(&pack_u32(server)?);
    let mut crc = Crc::new();
    crc.update(&head);
    crc.update(&body);
    let mut out: Vec<u8> = head[..8].to_vec();
    out.extend_from_slice(&crc.sum().to_le_bytes());
    out.extend_from_slice(&head[12..]);
    out.extend_from_slice(&body);
    Ok(out)
}

/// The pad's state, 80 bytes: nothing held, the sticks centred, the first touch at `touch` or
/// none, and the pad lying still.
pub fn pad_state(counter: i64, touch: Option<(i64, i64)>) -> Result<Vec<u8>> {
    let (x, y) = touch.unwrap_or((0, 0));
    let mut out: Vec<u8> = PAD.to_vec();
    out.extend_from_slice(&pack_u32(counter)?);
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&[128, 128, 128, 128]);
    out.extend_from_slice(&[0; 12]);
    out.extend_from_slice(&[u8::from(touch.is_some()), 0]);
    out.extend_from_slice(&pack_u16(x)?);
    out.extend_from_slice(&pack_u16(y)?);
    out.extend_from_slice(&[0; 6]);
    out.extend_from_slice(&0u64.to_le_bytes());
    for value in [0.0f32, 0.0, 1.0, 0.0, 0.0, 0.0] {
        out.extend_from_slice(&value.to_le_bytes());
    }
    Ok(out)
}

/// Where a press walking `path` evenly from `start` to `until` is at `now`, or None once it
/// has let go (or before any press).
pub fn along(path: &[(i64, i64)], start: f64, until: f64, now: f64) -> Option<(i64, i64)> {
    if path.is_empty() || now >= until {
        return None;
    }
    if path.len() == 1 {
        return Some(path[0]);
    }
    let last = (path.len() - 1) as f64;
    // Python's `max(0.0, x)` keeps 0.0 unless x is strictly greater.
    let share = (now - start) / (until - start);
    let at = if share > 0.0 { share } else { 0.0 } * last;
    let i = (py::int(at).expect("a finite share") as usize).min(path.len() - 2);
    let ((x0, y0), (x1, y1)) = (path[i], path[i + 1]);
    let share = at - i as f64;
    let x = py::round(x0 as f64 + (x1 - x0) as f64 * share).expect("finite");
    let y = py::round(y0 as f64 + (y1 - y0) as f64 * share).expect("finite");
    Some((x, y))
}

/// Where a rub between two ends is once it has gone `travelled` pixels: from the first to the
/// second and back, over and over.
pub fn rubbed(ends: ((i64, i64), (i64, i64)), travelled: f64) -> (i64, i64) {
    let ((x0, y0), (x1, y1)) = ends;
    // `math.dist`, which is `hypot` of the differences.
    let length = py::hypot((x1 - x0) as f64, (y1 - y0) as f64);
    if length == 0.0 {
        return (x0, y0);
    }
    let along = py::modulo_f(travelled, 2.0 * length);
    let share = if along <= length {
        along
    } else {
        2.0 * length - along
    } / length;
    (
        py::round(x0 as f64 + (x1 - x0) as f64 * share).expect("finite"),
        py::round(y0 as f64 + (y1 - y0) as f64 * share).expect("finite"),
    )
}

/// The emulated touch screen, which touch mode presses on the player's keys (`TouchServer`):
/// a tap, a slide along a path, and a press the player holds between keys. Each says why not
/// where it cannot, as `TouchServer.tap` does.
///
/// A test's stand-in that never rubs has no `rub`, `rub_on` or `rubbing`, as Python duck-types
/// it: here it rubs nothing, and no rub lasts.
pub trait TouchScreen {
    /// Press at (x, y) and let go.
    fn tap(&self, x: i64, y: i64) -> Option<String>;

    /// Press at the path's first point, move along it over `seconds`, and let go.
    fn slide(&self, path: &[(i64, i64)], seconds: f64) -> Option<String>;

    /// Press at (x, y) and keep it down; says why not as `TouchServer.tap` does.
    fn hold(&self, x: i64, y: i64) -> Option<String>;

    /// Slide a held press on to (x, y), still down.
    fn r#move(&self, x: i64, y: i64);

    /// Let a held press go where it is.
    fn release(&self);

    /// Press and rub back and forth between two points, for `seconds`, or until `release`
    /// where None, going down `delay` seconds from now. A rub asked for while one lasts carries
    /// it on between the new ends, for as long as the new one asks, never lifted.
    fn rub(
        &self,
        _ends: ((i64, i64), (i64, i64)),
        _seconds: Option<f64>,
        _delay: f64,
    ) -> Option<String> {
        None
    }

    /// Carry a rub that lasts on between new ends, for the time it has left, never lifted;
    /// nothing where none lasts. Whether it went on between new ends.
    fn rub_on(&self, _ends: ((i64, i64), (i64, i64))) -> bool {
        false
    }

    /// Whether a rub lasts, which only the screen knows: its time, a tap, a slide or a
    /// release ends it.
    fn rubbing(&self) -> bool {
        false
    }
}

/// A point kept to the touch screen's pixels.
fn on_screen(x: i64, y: i64) -> (i64, i64) {
    (x.clamp(0, WIDTH - 1), y.clamp(0, HEIGHT - 1))
}

/// The type of a client's request, or None for anything that is not one.
pub fn request(packet: &[u8]) -> Option<i64> {
    if packet.len() < 20 || &packet[..4] != CLIENT {
        return None;
    }
    let word: [u8; 4] = packet[16..20].try_into().expect("four bytes");
    Some(i64::from(u32::from_le_bytes(word)))
}

/// `time.time()`: seconds since the epoch, by the wall clock. `py::time` has no `time`; the
/// touch server's thread is never replayed from a tape, so the real clock is the one asked.
fn wall_clock() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |since| since.as_secs_f64())
}

/// What an `OSError` out of a socket says as `str(error)`: Windows' error and its text, the text
/// without its closing full stop.
fn os_error(error: &io::Error) -> String {
    let Some(code) = error.raw_os_error() else {
        return error.to_string();
    };
    let words = error.to_string();
    let text = words
        .strip_suffix(&format!(" (os error {code})"))
        .unwrap_or(&words);
    if cfg!(windows) {
        let text = text.trim_end_matches(|c: char| c == '.' || c.is_whitespace());
        format!("[WinError {code}] {text}")
    } else {
        format!("[Errno {code}] {text}")
    }
}

/// What `_lock` guards.
struct State {
    touch: Option<(i64, i64)>,
    path: Vec<(i64, i64)>,
    start: f64,
    until: f64,
    /// When a press held between keys is let go whatever happens, or 0 with none held.
    held: f64,
    /// Whether the held press was let go while it slid, and lets go at `held` for that.
    letting_go: bool,
    /// When the last press let go.
    up: f64,
    /// The ends a press rubs between, while one does (`rub`).
    rub: Option<((i64, i64), (i64, i64))>,
    /// `HELD`, as this server keeps it.
    held_for: f64,
    counter: i64,
    clients: IndexMap<SocketAddr, f64>,
}

impl State {
    /// When a press asked for now goes down: `LIFT` after the last one let go, lifting one
    /// still down now, so the game takes it for a new touch. Called holding the lock.
    fn new_press(&mut self) -> f64 {
        let now = time::monotonic();
        if self.touch.is_some() {
            (self.touch, self.up) = (None, now);
        }
        // Python's `max(now, up + LIFT)`: the first unless the second is strictly greater.
        if self.up + LIFT > now {
            self.up + LIFT
        } else {
            now
        }
    }

    /// Whether a rub lasts, under the lock.
    fn lasts(&self) -> bool {
        self.rub.is_some() && !self.path.is_empty() && time::monotonic() < self.until
    }
}

/// What the server's thread and its caller's share.
struct Inner {
    log: SharedLog,
    id: i64,
    state: Mutex<State>,
    stop: AtomicBool,
    socket: UdpSocket,
}

/// The DSU server Azahar's touch device reads, on a thread of its own.
pub struct TouchServer {
    inner: Arc<Inner>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl TouchServer {
    /// A server on `port` (Python's default is `PORT`), or the `OSError` that says why not.
    ///
    /// `os.urandom(4)` is a `RandomState`'s key hashed, which std seeds from the operating
    /// system's random numbers.
    pub fn new(log: SharedLog, port: i64) -> Result<Self> {
        use std::hash::BuildHasher;
        let id = i64::from(std::collections::hash_map::RandomState::new().hash_one(0u8) as u32);
        let Ok(bound) = u16::try_from(port) else {
            return Err(Error::new(
                Kind::OverflowError,
                "bind(): port must be 0-65535.",
            ));
        };
        let socket = match UdpSocket::bind((HOST, bound)) {
            Ok(socket) => socket,
            Err(error) => {
                return Err(Error::new(
                    Kind::OsError,
                    format!(
                        "port {port} is taken, so pa3ds cannot touch: {}",
                        os_error(&error)
                    ),
                ));
            }
        };
        socket.set_read_timeout(Some(Duration::from_secs_f64(TICK)))?;
        let inner = Arc::new(Inner {
            log,
            id,
            state: Mutex::new(State {
                touch: None,
                path: Vec::new(),
                start: 0.0,
                until: 0.0,
                held: 0.0,
                letting_go: false,
                up: 0.0,
                rub: None,
                held_for: HELD,
                counter: 0,
                clients: IndexMap::new(),
            }),
            stop: AtomicBool::new(false),
            socket,
        });
        let running = inner.clone();
        let thread = std::thread::Builder::new()
            .name("pa3ds touch".to_string())
            .spawn(move || running.run())
            .map_err(|_| Error::new(Kind::RuntimeError, "can't start new thread"))?;
        Ok(TouchServer {
            inner,
            thread: Mutex::new(Some(thread)),
        })
    }

    /// The port the server took: `PORT`, or whichever was free when it was asked for 0.
    pub fn port(&self) -> Result<i64> {
        Ok(i64::from(self.inner.socket.local_addr()?.port()))
    }

    /// Whether Azahar has asked for the pad lately: a tap reaches nobody otherwise.
    pub fn listened(&self) -> bool {
        let now = time::monotonic();
        let state = self.inner.locked();
        state.clients.values().any(|&when| now - when < LISTENING)
    }

    /// Press the touch screen at (x, y), its pixels from the top left, and let go. Says why not
    /// where Azahar has not asked for the pad lately, which is Azahar without pa3ds's touch
    /// settings (`config.AZAHAR`): closed and opened again after pa3ds patched them, it asks.
    pub fn tap(&self, x: i64, y: i64) -> Option<String> {
        self.slide(&[(x, y)], HOLD)
    }

    /// Press the touch screen at the path's first point, move along it evenly over `seconds`,
    /// and let go at its last. Says why not as `tap` does.
    pub fn slide(&self, path: &[(i64, i64)], seconds: f64) -> Option<String> {
        if !self.listened() {
            return Some(own(NOT_LISTENING, &[]));
        }
        let points: Vec<(i64, i64)> = path.iter().map(|&(x, y)| on_screen(x, y)).collect();
        if points.is_empty() {
            return None;
        }
        let mut state = self.inner.locked();
        (state.path, state.start) = (points, state.new_press());
        state.until = state.start + seconds;
        (state.held, state.rub) = (0.0, None);
        None
    }

    /// Press the touch screen at (x, y) and keep it down, until `release` or `HELD` seconds.
    /// Says why not as `tap` does.
    pub fn hold(&self, x: i64, y: i64) -> Option<String> {
        if !self.listened() {
            return Some(own(NOT_LISTENING, &[]));
        }
        let point = on_screen(x, y);
        let mut state = self.inner.locked();
        (state.path, state.start) = (vec![point], state.new_press());
        state.until = state.start;
        (state.held, state.rub, state.letting_go) = (state.start + state.held_for, None, false);
        None
    }

    /// Press and rub back and forth between two points at `RUB_SPEED`, for `seconds`, or
    /// until `release` where None (and `HELD` at most), going down `delay` seconds from now.
    /// Asked for while a rub lasts, it carries that one on between the new ends for as long as
    /// this one asks, never lifting: Pokémon-Amie starts counting a pet afresh at every press
    /// (spike 287). Says why not as `tap` does.
    pub fn rub(
        &self,
        ends: ((i64, i64), (i64, i64)),
        seconds: Option<f64>,
        delay: f64,
    ) -> Option<String> {
        if !self.listened() {
            return Some(own(NOT_LISTENING, &[]));
        }
        let (a, b) = (on_screen(ends.0.0, ends.0.1), on_screen(ends.1.0, ends.1.1));
        let now = time::monotonic();
        let mut state = self.inner.locked();
        if state.rub.is_none() || state.path.is_empty() {
            // Python's `max(new_press, now + delay)`: the first unless the second is greater.
            let pressed = state.new_press();
            state.start = if now + delay > pressed {
                now + delay
            } else {
                pressed
            };
        }
        (state.rub, state.path) = (Some((a, b)), vec![a]);
        state.until = match seconds {
            None => state.start + state.held_for,
            // Python's `max(now, start)`: the first unless the second is greater.
            Some(seconds) => (if state.start > now { state.start } else { now }) + seconds,
        };
        state.held = 0.0;
        None
    }

    /// Carry a rub that lasts on between new ends, for the time it has left, never lifting:
    /// Amie's stylus moved by an arrow in a tap's second, or as its part moves. Where none
    /// lasts, ended by its time, a tap, a slide or a release, nothing: only the player's tap or
    /// drag starts a press (ADR-0009), and an arrow after a beckon, or after Shift let go, had
    /// pressed again. Whether it went on between new ends.
    pub fn rub_on(&self, ends: ((i64, i64), (i64, i64))) -> bool {
        let new = (on_screen(ends.0.0, ends.0.1), on_screen(ends.1.0, ends.1.1));
        let mut state = self.inner.locked();
        if !state.lasts() || Some(new) == state.rub {
            return false;
        }
        (state.rub, state.path) = (Some(new), vec![new.0]);
        true
    }

    /// Whether a rub lasts: touch mode carries one on as its target moves only while it
    /// does, and keeps no clock of its own, as its own had outlived a rub this ended.
    pub fn rubbing(&self) -> bool {
        self.inner.locked().lasts()
    }

    /// Slide a held press from where it is on to (x, y) over `MOVE` seconds, still down.
    pub fn r#move(&self, x: i64, y: i64) {
        let mut state = self.inner.locked();
        if state.held == 0.0 || state.path.is_empty() {
            return;
        }
        let here = state.touch.unwrap_or(state.path[state.path.len() - 1]);
        state.path = vec![here, on_screen(x, y)];
        // Python's `max(start, now)`: the first unless the second is strictly greater.
        let now = time::monotonic();
        state.start = if now > state.start { now } else { state.start };
        state.until = state.start + MOVE;
    }

    /// Let a held press go where it is, or, while it slides on to a target, once it gets
    /// there: let go part way, a Charged Shot flew to between two goals and missed
    /// (as_super_training_more_l1_regimens).
    pub fn release(&self) {
        let mut state = self.inner.locked();
        if state.held != 0.0 && state.path.len() > 1 && time::monotonic() < state.until {
            (state.held, state.letting_go) = (state.until + LAND, true);
            return;
        }
        if state.touch.is_some() {
            state.up = time::monotonic();
        }
        (state.touch, state.path, state.held, state.rub) = (None, Vec::new(), 0.0, None);
    }

    /// `touch.HELD = seconds` for this server, as a test shortens it.
    pub fn set_held(&self, seconds: f64) {
        self.inner.locked().held_for = seconds;
    }

    /// `server._rub` and `server._until`, as a test reads them.
    #[cfg(test)]
    pub(crate) fn rub_until(&self) -> (Option<((i64, i64), (i64, i64))>, f64) {
        let state = self.inner.locked();
        (state.rub, state.until)
    }

    pub fn close(&self) {
        {
            let mut state = self.inner.locked();
            (state.touch, state.path, state.held, state.rub) = (None, Vec::new(), 0.0, None);
        }
        self.inner.send();
        self.inner.stop.store(true, Ordering::SeqCst);
        let taken = self
            .thread
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(thread) = taken {
            // `thread.join(timeout=1)`: a thread that has not stopped in a second is left behind.
            let waited = Instant::now();
            while !thread.is_finished() && waited.elapsed() < Duration::from_secs(1) {
                std::thread::sleep(Duration::from_millis(1));
            }
            if thread.is_finished() {
                let _ = thread.join();
            }
        }
        // `socket.close()` is the socket dropping with the server.
    }
}

impl TouchScreen for TouchServer {
    fn tap(&self, x: i64, y: i64) -> Option<String> {
        TouchServer::tap(self, x, y)
    }

    fn slide(&self, path: &[(i64, i64)], seconds: f64) -> Option<String> {
        TouchServer::slide(self, path, seconds)
    }

    fn hold(&self, x: i64, y: i64) -> Option<String> {
        TouchServer::hold(self, x, y)
    }

    fn r#move(&self, x: i64, y: i64) {
        TouchServer::r#move(self, x, y);
    }

    fn release(&self) {
        TouchServer::release(self);
    }

    fn rub(
        &self,
        ends: ((i64, i64), (i64, i64)),
        seconds: Option<f64>,
        delay: f64,
    ) -> Option<String> {
        TouchServer::rub(self, ends, seconds, delay)
    }

    fn rub_on(&self, ends: ((i64, i64), (i64, i64))) -> bool {
        TouchServer::rub_on(self, ends)
    }

    fn rubbing(&self) -> bool {
        TouchServer::rubbing(self)
    }
}

impl Inner {
    fn locked(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn run(&self) {
        let mut buffer = [0u8; 100];
        while !self.stop.load(Ordering::SeqCst) {
            let (packet, client) = match self.socket.recv_from(&mut buffer) {
                Ok((size, client)) => (buffer[..size].to_vec(), Some(client)),
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
                {
                    (Vec::new(), None)
                }
                Err(_) => {
                    if self.stop.load(Ordering::SeqCst) {
                        return;
                    }
                    continue;
                }
            };
            if let Some(client) = client {
                // A send that fails here ends the thread, as the exception ended Python's.
                if self.answer(&packet, client).is_err() {
                    return;
                }
            }
            {
                let mut state = self.locked();
                let now = time::monotonic();
                let touch = if now < state.start {
                    None // lifted before the next press (`LIFT`)
                } else if let Some(ends) = state.rub {
                    let touch =
                        (now < state.until).then(|| rubbed(ends, (now - state.start) * RUB_SPEED));
                    if touch.is_none() {
                        (state.rub, state.path) = (None, Vec::new());
                    }
                    touch
                } else {
                    let mut touch = along(&state.path, state.start, state.until, now);
                    if touch.is_none() && !state.path.is_empty() && now < state.held {
                        touch = state.path.last().copied();
                    } else if touch.is_none() && state.held != 0.0 && !state.letting_go {
                        (self.log)(&format!(
                            "touch: a press held {} s let go",
                            py::fixed(state.held_for, 0)
                        ));
                    }
                    if touch.is_none() {
                        (state.path, state.held, state.letting_go) = (Vec::new(), 0.0, false);
                    }
                    touch
                };
                if touch.is_none() && state.touch.is_some() {
                    state.up = now;
                }
                state.touch = touch;
            }
            self.send();
        }
    }

    fn answer(&self, packet: &[u8], client: SocketAddr) -> io::Result<()> {
        let kind = request(packet);
        if kind == Some(VERSION_TYPE) {
            let reply = message(VERSION_TYPE, &pack_u16(VERSION).expect("in range"), self.id)
                .expect("in range");
            self.socket.send_to(&reply, client)?;
        } else if kind == Some(PORT_INFO) {
            let reply = message(PORT_INFO, &PAD, self.id).expect("in range");
            self.socket.send_to(&reply, client)?;
        } else if kind == Some(PAD_DATA) {
            let mut state = self.locked();
            if !state.clients.contains_key(&client) {
                (self.log)(&format!(
                    "touch: Azahar is listening, at {}:{}",
                    client.ip(),
                    client.port()
                ));
            }
            state.clients.insert(client, time::monotonic());
        }
        Ok(())
    }

    fn send(&self) {
        let now = time::monotonic();
        let (state, clients) = {
            let mut held = self.locked();
            held.clients.retain(|_, when| now - *when < LISTENING);
            if held.clients.is_empty() {
                return;
            }
            let clock = py::int(wall_clock() * COUNTS as f64).expect("a finite clock");
            held.counter = (held.counter + 1).max(clock);
            let state = message(
                PAD_DATA,
                &pad_state(py::modulo(held.counter, 1 << 32), held.touch).expect("in range"),
                self.id,
            )
            .expect("in range");
            (state, held.clients.keys().copied().collect::<Vec<_>>())
        };
        for client in clients {
            let _ = self.socket.send_to(&state, client);
        }
    }
}
