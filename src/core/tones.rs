//! Tones that say where a cursor is: higher the further up, and panned the further right.
//!
//! The Hoenn map's cursor says a place's name only over a place, and between places it says
//! nothing, so a player cannot tell moving through sea from standing at the map's edge; touch
//! mode's targets are spread over a screen the player cannot see (Orion, 2026-10-01). A short tone
//! at each move says where the cursor now is: its pitch from 880 Hz at the top to 220 Hz at the
//! bottom, an octave a half of the height, and its balance from the left ear to the right. They
//! follow how loud Azahar plays the game (`loudness`), times `[pa3ds.audio] tones`: 1 as loud as
//! they always were beside the game at Azahar's full volume, above 1 louder, 0 none (Orion,
//! 2026-10-18).
//!
//! Touch mode's targets jump, and each is a tone (`play`). The map's cursor glides on every frame
//! for as long as the circle pad is held, and is read every screen poll, a fifth of a second
//! (spike 175): each poll it moved, the tone slides from where it is to where the cursor is
//! (`glide`), and keeps sounding a little past the slide, so the next poll's slide carries on from
//! it without a break. A sound file a slide, played one after another, left a gap between each
//! two, heard as beeps rather than a slide (Orion, 2026-10-02).
//!
//! pa3ds's own sounds, beside the game's (ADR-0025), and nothing like them is said: a tone is
//! never the only way something is told, only where it is. One `Voice`, written a fiftieth of a
//! second at a time to Windows' own waveOut by a thread of its own, so a tone changes without
//! stopping.
//!
//! Bound by:
//! - ADR-0025: a tone is said beside a word, never instead of one.

use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;

use crate::core::loudness::Loudness;
use crate::py::{self, Error, Kind, Result};

pub const RATE: i64 = 22050;
/// Seconds a tone lasts: under the time between two moves of a held D-pad.
pub const LENGTH: f64 = 0.07;
/// Seconds a slide lasts: a screen poll, so a cursor's slides follow on from each other.
pub const GLIDE: f64 = 0.2;
/// Seconds a slide's end keeps sounding, for the next poll's slide to carry on from: a poll comes
/// a little after a fifth of a second, and later still when it waits on a sweep.
pub const HOLD: f64 = 0.1;
/// Seconds faded in and out, so a tone starts and stops without a click.
pub const FADE: f64 = 0.008;
pub const LOW: f64 = 220.0;
pub const HIGH: f64 = 880.0;
pub const VOLUME: f64 = 0.3;
/// Frames written at a time: a fiftieth of a second.
pub const BUFFER: i64 = 441;
/// Buffers queued at once, which is how late a tone can be heard: four fiftieths of a second.
pub const BUFFERS: i64 = 4;

/// `round(RATE * seconds)` of one of the constants above, which is never a NaN.
fn frames(seconds: f64) -> i64 {
    py::round(RATE as f64 * seconds).expect("a constant is finite")
}

/// The tone as it sounds, rendered a buffer at a time: where it is, where it is sliding to, and
/// how long it has left.
#[derive(Debug, Clone)]
pub struct Voice {
    from: (f64, f64),
    to: (f64, f64),
    /// Frames of the slide rendered.
    done: i64,
    /// Frames the slide lasts, 0 for none.
    slide: i64,
    /// Frames left to sound before fading out.
    left: i64,
    level: f64,
    phase: f64,
}

impl Voice {
    pub fn new() -> Self {
        Voice {
            from: (0.5, 0.5),
            to: (0.5, 0.5),
            done: 0,
            slide: 0,
            left: 0,
            level: 0.0,
            phase: 0.0,
        }
    }

    pub fn sounding(&self) -> bool {
        self.left > 0 || self.level > 0.0
    }

    /// A tone at a place: `across` 0 the left edge to 1 the right, `down` 0 the top to 1 the
    /// bottom.
    pub fn play(&mut self, across: f64, down: f64) {
        self.from = (across, down);
        self.to = (across, down);
        self.slide = 0;
        self.left = frames(LENGTH);
    }

    /// A slide to `end`, from where the tone is if it is still sounding, else from `start`.
    pub fn glide(&mut self, start: (f64, f64), end: (f64, f64)) {
        self.from = if self.sounding() { self.at() } else { start };
        self.to = end;
        self.done = 0;
        self.slide = frames(GLIDE);
        self.left = frames(GLIDE + HOLD);
    }

    /// The next `count` frames, 16-bit stereo, `gain` times `VOLUME` loud.
    ///
    /// Python's `gain` defaults to 1.
    pub fn render(&mut self, count: i64, gain: f64) -> Result<Vec<u8>> {
        let mut out: Vec<u8> = Vec::new();
        let step = 1.0 / (RATE as f64 * FADE);
        for _ in 0..count {
            let (across, down) = self.at();
            self.done += 1;
            // `min(a, b)` keeps `a` unless `b` is strictly less, `max(a, b)` unless `b` is greater.
            self.level = if self.left > 0 {
                let raised = self.level + step;
                if 1.0 < raised { 1.0 } else { raised }
            } else {
                let lowered = self.level - step;
                if 0.0 > lowered { 0.0 } else { lowered }
            };
            self.left = (self.left - 1).max(0);
            self.phase = py::modulo_f(
                self.phase
                    + 2.0 * std::f64::consts::PI * HIGH * (LOW / HIGH).powf(down) / RATE as f64,
                2.0 * std::f64::consts::PI,
            );
            // `max(-32767.0, min(32767.0, value))`, a NaN held at the top as Python holds it.
            let value = 32767.0 * VOLUME * gain * self.level * self.phase.sin();
            let value = if value < 32767.0 { value } else { 32767.0 };
            let value = if value > -32767.0 { value } else { -32767.0 };
            out.extend_from_slice(&short(py::round(
                value * (across * std::f64::consts::PI / 2.0).cos(),
            )?)?);
            out.extend_from_slice(&short(py::round(
                value * (across * std::f64::consts::PI / 2.0).sin(),
            )?)?);
        }
        Ok(out)
    }

    fn at(&self) -> (f64, f64) {
        let share = if self.slide != 0 {
            let share = self.done as f64 / self.slide as f64;
            if 1.0 < share { 1.0 } else { share }
        } else {
            1.0
        };
        (
            clamp(self.from.0 + (self.to.0 - self.from.0) * share),
            clamp(self.from.1 + (self.to.1 - self.from.1) * share),
        )
    }
}

impl Default for Voice {
    fn default() -> Self {
        Voice::new()
    }
}

/// `struct.pack("<h", value)`.
#[track_caller]
fn short(value: i64) -> Result<[u8; 2]> {
    i16::try_from(value)
        .map(i16::to_le_bytes)
        .map_err(|_| Error::unpack("'h' format requires -32768 <= number <= 32767"))
}

/// `threading.Event`: set by whoever wants the thread, waited on and cleared by the thread.
struct Event {
    flag: Mutex<bool>,
    changed: Condvar,
}

impl Event {
    fn new() -> Self {
        Event {
            flag: Mutex::new(false),
            changed: Condvar::new(),
        }
    }

    fn set(&self) {
        *self.flag.lock().unwrap_or_else(PoisonError::into_inner) = true;
        self.changed.notify_all();
    }

    fn clear(&self) {
        *self.flag.lock().unwrap_or_else(PoisonError::into_inner) = false;
    }

    fn wait(&self) {
        let mut flag = self.flag.lock().unwrap_or_else(PoisonError::into_inner);
        while !*flag {
            flag = self
                .changed
                .wait(flag)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }
}

/// What the caller's thread and the tones' share: Python's `_voice` with its `_lock`, and `_wake`.
struct Shared {
    voice: Mutex<Voice>,
    wake: Event,
    loudness: Option<Loudness>,
    level: f64,
}

fn locked(voice: &Mutex<Voice>) -> MutexGuard<'_, Voice> {
    voice.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A game's clock's tick as a tone in the middle, 440 Hz (`ScreenAnnouncer`): the lowest,
/// an octave under it, was hard to hear even at 1.5 (Orion, 2026-10-26).
pub fn ticking(tones: Option<Rc<Tones>>) -> Option<Rc<dyn Fn()>> {
    let tones = tones?;
    Some(Rc::new(move || {
        let _ = tones.play(0.5, 0.5);
    }))
}

/// Plays the voice on a thread of its own, started at the first tone; silent off Windows and
/// without a sound device.
pub struct Tones {
    shared: Arc<Shared>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl Tones {
    /// `loudness` how loud Azahar plays, None as at its full volume; `level` the setting.
    pub fn new(loudness: Option<Loudness>, level: f64) -> Self {
        Tones {
            shared: Arc::new(Shared {
                voice: Mutex::new(Voice::new()),
                wake: Event::new(),
                loudness,
                level,
            }),
            thread: Mutex::new(None),
        }
    }

    /// A tone where the cursor is.
    pub fn play(&self, across: f64, down: f64) -> Result<()> {
        locked(&self.shared.voice).play(across, down);
        self.sound()
    }

    /// A tone sliding from where the cursor was to where it is.
    pub fn glide(&self, start: (f64, f64), end: (f64, f64)) -> Result<()> {
        locked(&self.shared.voice).glide(start, end);
        self.sound()
    }

    fn sound(&self) -> Result<()> {
        if !cfg!(windows) {
            return Ok(());
        }
        let mut thread = self.thread.lock().unwrap_or_else(PoisonError::into_inner);
        if thread.is_none() {
            let shared = self.shared.clone();
            let started = std::thread::Builder::new()
                .name("tones".to_string())
                .spawn(move || run(&shared))
                .map_err(|_| Error::new(Kind::RuntimeError, "can't start new thread"))?;
            *thread = Some(started);
        }
        self.shared.wake.set();
        Ok(())
    }
}

impl Default for Tones {
    fn default() -> Self {
        Tones::new(None, 1.0)
    }
}

#[cfg(windows)]
fn run(shared: &Shared) {
    let Ok(mut out) = waveout::WaveOut::new() else {
        return;
    };
    loop {
        shared.wake.wait();
        shared.wake.clear();
        loop {
            let gain = shared.level
                * shared
                    .loudness
                    .as_ref()
                    .map_or(1.0, |loudness| loudness.azahar());
            let data = {
                let mut voice = locked(&shared.voice);
                if !voice.sounding() {
                    break;
                }
                // Python's thread ends here with the exception, as it would on a NaN.
                match voice.render(BUFFER, gain) {
                    Ok(data) => data,
                    Err(_) => return,
                }
            };
            out.write(&data);
        }
    }
}

#[cfg(not(windows))]
fn run(_shared: &Shared) {}

#[cfg(windows)]
mod waveout {
    use std::ptr::{null_mut, read_volatile};

    use windows_sys::Win32::Media::Audio::{
        HWAVEOUT, WAVE_FORMAT_PCM, WAVE_MAPPER, WAVEFORMATEX, WAVEHDR, WHDR_DONE, waveOutOpen,
        waveOutPrepareHeader, waveOutWrite,
    };

    use super::{BUFFER, BUFFERS, RATE};
    use crate::py::{self, Error, Kind, Result};

    /// Windows' waveOut, open for good, with `BUFFERS` buffers written in turn.
    pub struct WaveOut {
        handle: HWAVEOUT,
        buffers: Vec<Box<[u8]>>,
        headers: Vec<WAVEHDR>,
        next: usize,
    }

    impl WaveOut {
        pub fn new() -> Result<Self> {
            let mut handle: HWAVEOUT = null_mut();
            let format = WAVEFORMATEX {
                wFormatTag: WAVE_FORMAT_PCM as u16,
                nChannels: 2,
                nSamplesPerSec: RATE as u32,
                nAvgBytesPerSec: (RATE * 4) as u32,
                nBlockAlign: 4,
                wBitsPerSample: 16,
                cbSize: 0,
            };
            // SAFETY: `handle` and `format` are live for the call, and no callback is given.
            let failed = unsafe { waveOutOpen(&mut handle, WAVE_MAPPER, &format, 0, 0, 0) } != 0;
            if failed {
                return Err(Error::new(Kind::OsError, "no sound device for the tones"));
            }
            let mut buffers: Vec<Box<[u8]>> = (0..BUFFERS)
                .map(|_| vec![0u8; (BUFFER * 4) as usize].into_boxed_slice())
                .collect();
            let mut headers: Vec<WAVEHDR> = Vec::new();
            for buffer in buffers.iter_mut() {
                headers.push(WAVEHDR {
                    lpData: buffer.as_mut_ptr(),
                    dwBufferLength: (BUFFER * 4) as u32,
                    ..Default::default()
                });
            }
            for header in headers.iter_mut() {
                // SAFETY: the headers are in place for good, and so are the buffers they name:
                // neither list grows again, and a move of the list moves no element.
                unsafe {
                    waveOutPrepareHeader(handle, header, std::mem::size_of::<WAVEHDR>() as u32);
                }
                header.dwFlags |= WHDR_DONE;
            }
            Ok(WaveOut {
                handle,
                buffers,
                headers,
                next: 0,
            })
        }

        /// Queue `data`, waiting for the oldest buffer to finish playing first.
        pub fn write(&mut self, data: &[u8]) {
            let header: *mut WAVEHDR = &mut self.headers[self.next];
            // SAFETY: the driver sets the flag from another thread while the header is queued, so
            // it is read through the pointer each time.
            while unsafe { read_volatile(&raw const (*header).dwFlags) } & WHDR_DONE == 0 {
                py::time::sleep(0.004);
            }
            self.buffers[self.next][..data.len()].copy_from_slice(data);
            // SAFETY: the buffer is done playing, so nothing else touches the header or the
            // buffer until it is written again.
            unsafe {
                (*header).dwBufferLength = data.len() as u32;
                waveOutWrite(self.handle, header, std::mem::size_of::<WAVEHDR>() as u32);
            }
            self.next = (self.next + 1) % BUFFERS as usize;
        }
    }
}

fn clamp(value: f64) -> f64 {
    let raised = if 0.0 > value { 0.0 } else { value };
    if 1.0 < raised { 1.0 } else { raised }
}
