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
//! release, and closing the server sends one.
//!
//! A slide is a press that moves, along a path walked evenly in time and let go at its end:
//! Pokémon-Amie is stroked so, and answers it with its hand on the Pokémon (spike 200). A tap is a
//! slide that stays on one point. ADR-0009 allowed pa3ds a tap alone; the slide is an exploration
//! Khai asked for (2026-10-08), for Orion to keep or take back.
//!
//! A drag is a press the player holds between keys (`Drag`): held on one point, slid on to the
//! next in `MOVE` seconds each time the player moves it, and let go when they let go, or after
//! `HELD` seconds whatever happens, so a finger is never left down for good.
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
/// How long a held press is kept down at most, should its letting go never come.
///
/// A server keeps its own (`State.held_for`), which a test shortens where Python's sets the
/// module's `HELD`.
pub const HELD: f64 = 30.0;

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

/// A press the player holds between keys (`TouchServer`).
pub trait Drag {
    /// Press at (x, y) and keep it down; says why not as `TouchServer.tap` does.
    fn hold(&self, x: i64, y: i64) -> Option<String>;

    /// Slide a held press on to (x, y), still down.
    fn r#move(&self, x: i64, y: i64);

    /// Let a held press go where it is.
    fn release(&self);
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
    /// `HELD`, as this server keeps it.
    held_for: f64,
    counter: i64,
    clients: IndexMap<SocketAddr, f64>,
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
        state.start = time::monotonic();
        state.until = state.start + seconds;
        state.touch = Some(points[0]);
        state.path = points;
        state.held = 0.0;
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
        state.start = time::monotonic();
        state.path = vec![point];
        state.until = state.start;
        state.touch = Some(point);
        state.held = state.start + state.held_for;
        None
    }

    /// Slide a held press from where it is on to (x, y) over `MOVE` seconds, still down.
    pub fn r#move(&self, x: i64, y: i64) {
        let mut state = self.inner.locked();
        let Some(touch) = state.touch.filter(|_| state.held != 0.0) else {
            return;
        };
        state.path = vec![touch, on_screen(x, y)];
        state.start = time::monotonic();
        state.until = state.start + MOVE;
    }

    /// Let a held press go where it is.
    pub fn release(&self) {
        let mut state = self.inner.locked();
        (state.touch, state.path, state.held) = (None, Vec::new(), 0.0);
    }

    /// `touch.HELD = seconds` for this server, as a test shortens it.
    pub fn set_held(&self, seconds: f64) {
        self.inner.locked().held_for = seconds;
    }

    pub fn close(&self) {
        {
            let mut state = self.inner.locked();
            (state.touch, state.path, state.held) = (None, Vec::new(), 0.0);
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

impl Drag for TouchServer {
    fn hold(&self, x: i64, y: i64) -> Option<String> {
        TouchServer::hold(self, x, y)
    }

    fn r#move(&self, x: i64, y: i64) {
        TouchServer::r#move(self, x, y);
    }

    fn release(&self) {
        TouchServer::release(self);
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
                let mut touch = along(&state.path, state.start, state.until, now);
                if touch.is_none() && !state.path.is_empty() && now < state.held {
                    touch = state.path.last().copied();
                } else if touch.is_none() && state.held != 0.0 {
                    (self.log)(&format!(
                        "touch: a press held {} s let go",
                        py::fixed(state.held_for, 0)
                    ));
                    state.held = 0.0;
                }
                state.touch = touch;
                if touch.is_none() {
                    state.path = Vec::new();
                }
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
