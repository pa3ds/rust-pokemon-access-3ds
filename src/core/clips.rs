//! Short sounds pa3ds plays beside the game's, as loud as the game would play them: its footsteps
//! where the game plays none (ADR-0025, `games.gen6.field.footsteps`).
//!
//! A clip is one or more sounds played together, mixed into one as a game starts (`mix`): the
//! game's own, decoded out of the player's ROM (`bcsar`), and WAV files of pa3ds's own (`wave`). It
//! plays through Windows' waveOut on the default output, where Azahar's sound goes while its output
//! device is Auto, which mixes it with the game's. A clip cuts off the one before it on the same
//! voice, an output of its own: a footstep the last. The Pokémon's cries have a voice a side, so
//! each cuts off only its own Pokémon's, and the three can be heard over each other as the game's
//! sounds are (waiting their turn, a cry would come too late, and cut off, the Pokémon before went
//! unheard). The feedback sounds, cues and a key's, each have a voice of their own the same way,
//! and cut off only themselves. How loud is what the game
//! would make it: the sound's own volume and its sequence's random one; times how loud Azahar
//! plays the game (`loudness`); times `[pa3ds.audio] footsteps`, above 1 louder than the game
//! would. A clip plays in both ears, as the game plays it, but where it is asked toward one side:
//! Head It's cue, and the Pokémon's cries in Amie's games.
//!
//! Bound by:
//! - ADR-0025: the game's own sounds as loud as the game and Azahar would play them; nothing mutes
//!   or skips the game's; none panned but Head It's cue and the cries in Amie's games.

use std::cell::RefCell;
use std::collections::HashSet;
use std::collections::hash_map::RandomState;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::bcsar::Sound;
use crate::core::binary::{u16, u32};
use crate::core::keyfile::data_path;
use crate::core::loudness::Loudness;
use crate::py::{self, Error, Kind, Result};

/// pa3ds's own sounds, WAV files every game's tables share.
pub fn sounds() -> Result<PathBuf> {
    Ok(data_path()?.join("sounds"))
}

/// What a voice plays on: Windows' waveOut, or a test's stand-in where Python's test patches
/// `_WaveOut`.
pub trait Output {
    fn rate(&self) -> i64;
    fn play(&mut self, data: &[u8]);
}

/// Opens an output at a rate: `_WaveOut(rate)`.
pub type Opener = Rc<dyn Fn(i64) -> Result<Box<dyn Output>>>;

/// What an output is kept by: a voice's number, or a sound's name where the clips are `apart`
/// (Python's `int | str` key).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum On {
    Voice(i64),
    Sound(String),
}

/// The clips a game has, by name, and the outputs they play on, a voice each, each opened at
/// its first clip.
pub struct Clips {
    sounds: RefCell<IndexMap<String, Sound>>,
    loudness: Loudness,
    level: f64,
    /// Whether each sound has a voice of its own, as the feedback sounds do: one cue sounded
    /// cut off in a regimen, where the Bot's alert, its "!" and a hit's sound can come close
    /// together.
    apart: bool,
    outs: RefCell<IndexMap<On, Box<dyn Output>>>,
    open: Opener,
}

impl Clips {
    /// `Clips(loudness, level)`; Python's `level` defaults to 1.
    pub fn new(loudness: Loudness, level: f64) -> Self {
        Clips::opening(
            loudness,
            level,
            Rc::new(|rate| Ok(Box::new(waveout::WaveOut::new(rate)?) as Box<dyn Output>)),
        )
    }

    /// `Clips(loudness, level, apart=True)`: each sound on a voice of its own.
    pub fn apart(loudness: Loudness, level: f64) -> Self {
        Clips {
            apart: true,
            ..Clips::new(loudness, level)
        }
    }

    /// `Clips(loudness, level)` playing on what `open` opens: a test's outputs.
    pub fn opening(loudness: Loudness, level: f64, open: Opener) -> Self {
        Clips {
            sounds: RefCell::new(IndexMap::new()),
            loudness,
            level,
            apart: false,
            outs: RefCell::new(IndexMap::new()),
            open,
        }
    }

    pub fn add(&self, name: &str, sound: Sound) {
        self.sounds.borrow_mut().insert(name.to_string(), sound);
    }

    pub fn names(&self) -> HashSet<String> {
        self.sounds.borrow().keys().cloned().collect()
    }

    /// `play(name)`, toward the middle, both ears at full, as Python's `across` defaults.
    pub fn play(&self, name: &str) -> Result<()> {
        self.play_across(name, 0.5)
    }

    /// `play(name, across)`, on the voice Python's `voice` of None picks.
    pub fn play_across(&self, name: &str, across: f64) -> Result<()> {
        self.play_voice(name, across, None)
    }

    /// `play(name, across, voice)`.
    pub fn play_on(&self, name: &str, across: f64, voice: i64) -> Result<()> {
        self.play_voice(name, across, Some(voice))
    }

    /// Play the clip `name`, as loud as the game would play it, toward `across` (`stereo`),
    /// cutting off only what `voice` was playing: by default the one voice, or where the clips
    /// are `apart` the sound itself, played again.
    fn play_voice(&self, name: &str, across: f64, voice: Option<i64>) -> Result<()> {
        let Some(sound) = self.sounds.borrow().get(name).cloned() else {
            return Ok(());
        };
        if !cfg!(windows) {
            return Ok(());
        }
        let level =
            sound.volume * uniform(sound.quietest, 1.0) * self.loudness.azahar() * self.level;
        // ponytail: the sequence's random pitch bend is left out; add it if steps sound alike.
        let samples = stereo(&sound.samples, level, across)?;
        let data: Vec<u8> = samples
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect();
        let on = match voice {
            Some(voice) => On::Voice(voice),
            None if self.apart => On::Sound(name.to_string()),
            None => On::Voice(0),
        };
        let mut outs = self.outs.borrow_mut();
        if outs.get(&on).is_none_or(|out| out.rate() != sound.rate) {
            match (self.open)(sound.rate) {
                Ok(out) => {
                    outs.insert(on.clone(), out);
                }
                // No sound device: nothing more to play.
                Err(_) => {
                    self.sounds.borrow_mut().clear();
                    return Ok(());
                }
            }
        }
        if let Some(out) = outs.get_mut(&on) {
            out.play(&data);
        }
        Ok(())
    }
}

impl Output for waveout::WaveOut {
    fn rate(&self) -> i64 {
        self.rate
    }

    fn play(&mut self, data: &[u8]) {
        waveout::WaveOut::play(self, data);
    }
}

/// Mono samples `level` times as loud, as left and right, in turn: `across` 0.5 both ears at
/// full, as a mono sound plays, 0 the left alone and 1 the right, the far ear fading between.
pub fn stereo(samples: &[i16], level: f64, across: f64) -> Result<Vec<i16>> {
    // Python's `min(1.0, x)`: the first unless the second is strictly less.
    let (left, right) = (2.0 - 2.0 * across, 2.0 * across);
    let left = if left < 1.0 { left } else { 1.0 };
    let right = if right < 1.0 { right } else { 1.0 };
    let mut out: Vec<i16> = Vec::with_capacity(samples.len() * 2);
    for &sample in samples {
        let value = sample as f64 * level;
        out.push(py::round(value * left)?.clamp(-32768, 32767) as i16);
        out.push(py::round(value * right)?.clamp(-32768, 32767) as i16);
    }
    Ok(out)
}

/// `random.uniform(a, b)`: `a + (b - a) * random.random()`, the random number 53 random bits of
/// the standard library's own randomly keyed hasher, as Python's has 53 of its generator's.
fn uniform(a: f64, b: f64) -> f64 {
    let bits = RandomState::new().build_hasher().finish() >> 11;
    a + (b - a) * (bits as f64 / (1u64 << 53) as f64)
}

/// `wave.Error`, which `footsteps.load` catches with an OSError and an EOFError: a ValueError
/// here, which nothing else reading a WAV raises.
#[track_caller]
fn wave_error(message: impl Into<String>) -> Error {
    Error::new(Kind::ValueError, message)
}

/// `EOFError`, whose message Python's `wave` leaves empty.
#[track_caller]
fn end_of_file() -> Error {
    Error::new(Kind::ValueError, "")
}

const WAVE_FORMAT_PCM: i64 = 0x0001;
const WAVE_FORMAT_EXTENSIBLE: i64 = 0xFFFE;
/// `KSDATAFORMAT_SUBTYPE_PCM`, the GUID an extensible WAV's format names plain PCM by.
const SUBTYPE_PCM: &[u8] = b"\x01\x00\x00\x00\x00\x00\x10\x00\x80\x00\x00\xaa\x00\x38\x9b\x71";

/// A 16-bit WAV file as a sound at full volume, its channels averaged into one. A
/// `wave.Error` or an OSError where it is not one.
///
/// Python's `wave` module reads the file: a RIFF header, then chunks until the `data` one,
/// each padded to an even size, the `fmt ` one before it.
pub fn wave(path: &Path) -> Result<Sound> {
    let file = fs::read(path)?;
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let chunk = |at: usize| -> Option<(&[u8], usize)> {
        let head = file.get(at..at + 8)?;
        Some((
            &head[..4],
            u32::from_le_bytes(head[4..8].try_into().ok()?) as usize,
        ))
    };
    let Some((riff, _size)) = chunk(0) else {
        return Err(end_of_file());
    };
    if riff != b"RIFF" {
        return Err(wave_error("file does not start with RIFF id"));
    }
    if file.get(8..12) != Some(b"WAVE".as_slice()) {
        return Err(wave_error("not a WAVE file"));
    }
    let mut format: Option<(i64, i64, i64)> = None;
    let mut data: Option<&[u8]> = None;
    let mut at = 12;
    while let Some((id, size)) = chunk(at) {
        let body = &file[(at + 8).min(file.len())..(at + 8 + size).min(file.len())];
        if id == b"fmt " {
            if body.len() < 14 {
                return Err(end_of_file());
            }
            let tag = u16(body, 0)?;
            let (channels, rate) = (u16(body, 2)?, u32(body, 4)?);
            if tag != WAVE_FORMAT_PCM && tag != WAVE_FORMAT_EXTENSIBLE {
                return Err(wave_error(format!("unknown format: {tag}")));
            }
            if body.len() < 16 {
                return Err(end_of_file());
            }
            let width = u16(body, 14)?;
            if tag == WAVE_FORMAT_EXTENSIBLE {
                if body.len() < 40 {
                    return Err(end_of_file());
                }
                if &body[24..40] != SUBTYPE_PCM {
                    return Err(wave_error("unknown extended format"));
                }
            }
            let width = (width + 7) / 8;
            if width == 0 {
                return Err(wave_error("bad sample width"));
            }
            if channels == 0 {
                return Err(wave_error("bad # of channels"));
            }
            format = Some((channels, rate, width));
        } else if id == b"data" {
            if format.is_none() {
                return Err(wave_error("data chunk before fmt chunk"));
            }
            data = Some(body);
            break;
        }
        at += 8 + size + size % 2;
    }
    let (Some((channels, rate, width)), Some(data)) = (format, data) else {
        return Err(wave_error("fmt chunk and/or data chunk missing"));
    };
    if width != 2 {
        return Err(wave_error(format!("{name} is not 16-bit")));
    }
    let frames = (data.len() as i64 / (channels * width)) as usize;
    let mut samples: Vec<i16> = data[..frames * (channels * width) as usize]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&pair| i16::from_le_bytes(pair))
        .collect();
    if channels > 1 {
        let channels = channels as usize;
        samples = samples
            .chunks_exact(channels)
            .map(|frame| {
                let total: i64 = frame.iter().map(|&sample| sample as i64).sum();
                py::floordiv(total, channels as i64) as i16
            })
            .collect();
    }
    Ok(Sound {
        rate,
        samples,
        volume: 1.0,
        quietest: 1.0,
    })
}

/// Sounds played together as one, each at its own volume times `level`, at the first one's
/// rate; the random volume as wide as the widest of theirs.
pub fn mix(layers: &[(Sound, f64)]) -> Result<Sound> {
    let rate = layers[0].0.rate;
    let mut tracks: Vec<Vec<f64>> = Vec::new();
    for (sound, level) in layers {
        tracks.push(
            resampled(sound, rate)?
                .into_iter()
                .map(|sample| sample * sound.volume * level)
                .collect(),
        );
    }
    let length = tracks.iter().map(Vec::len).max().unwrap_or(0);
    let mut out: Vec<i16> = Vec::with_capacity(length);
    for at in 0..length {
        let total = sum(tracks.iter().filter_map(|track| track.get(at).copied()));
        out.push(py::round(total)?.clamp(-32768, 32767) as i16);
    }
    let mut quietest = layers[0].0.quietest;
    for (sound, _level) in &layers[1..] {
        if sound.quietest < quietest {
            quietest = sound.quietest;
        }
    }
    Ok(Sound {
        rate,
        samples: out,
        volume: 1.0,
        quietest,
    })
}

/// Python's `sum` of floats: the first added to the int 0 as it is, and the rest by Neumaier's
/// compensated sum, as CPython has added floats since 3.12.
pub(crate) fn sum(values: impl Iterator<Item = f64>) -> f64 {
    let (mut total, mut compensation) = (0.0f64, 0.0f64);
    for (index, value) in values.enumerate() {
        if index == 0 {
            total = value;
            continue;
        }
        let next = total + value;
        if total.abs() >= value.abs() {
            compensation += (total - next) + value;
        } else {
            compensation += (value - next) + total;
        }
        total = next;
    }
    if compensation != 0.0 && compensation.is_finite() {
        total += compensation;
    }
    total
}

/// A sound's samples at `rate`, drawn straight between its own.
pub fn resampled(sound: &Sound, rate: i64) -> Result<Vec<f64>> {
    if sound.rate == rate || sound.samples.is_empty() {
        return Ok(sound.samples.iter().map(|&sample| sample as f64).collect());
    }
    let (step, last) = (sound.rate as f64 / rate as f64, sound.samples.len() - 1);
    let mut out: Vec<f64> = Vec::new();
    for at in 0..py::int(last as f64 / step)? + 1 {
        let r#where = at as f64 * step;
        let before = py::int(r#where)? as usize;
        let after = (before + 1).min(last);
        let share = r#where - before as f64;
        out.push(
            sound.samples[before] as f64 * (1.0 - share) + sound.samples[after] as f64 * share,
        );
    }
    Ok(out)
}

#[cfg(windows)]
mod waveout {
    use std::ptr::null_mut;

    use windows_sys::Win32::Media::Audio::{
        HWAVEOUT, WAVE_FORMAT_PCM, WAVE_MAPPER, WAVEFORMATEX, WAVEHDR, waveOutOpen,
        waveOutPrepareHeader, waveOutReset, waveOutUnprepareHeader, waveOutWrite,
    };

    use crate::py::{Error, Kind, Result};

    /// Windows' waveOut, stereo 16-bit at one rate, open for good: a clip resets it and plays.
    pub struct WaveOut {
        pub rate: i64,
        handle: HWAVEOUT,
        buffer: Option<Box<[u8]>>,
        header: Option<Box<WAVEHDR>>,
    }

    impl WaveOut {
        pub fn new(rate: i64) -> Result<Self> {
            let mut handle: HWAVEOUT = null_mut();
            let format = WAVEFORMATEX {
                wFormatTag: WAVE_FORMAT_PCM as u16,
                nChannels: 2,
                nSamplesPerSec: rate as u32,
                nAvgBytesPerSec: (rate * 4) as u32,
                nBlockAlign: 4,
                wBitsPerSample: 16,
                cbSize: 0,
            };
            // SAFETY: `handle` and `format` are live for the call, and no callback is given.
            let failed = unsafe { waveOutOpen(&mut handle, WAVE_MAPPER, &format, 0, 0, 0) } != 0;
            if failed {
                return Err(Error::new(Kind::OsError, "no sound device for clips"));
            }
            Ok(WaveOut {
                rate,
                handle,
                buffer: None,
                header: None,
            })
        }

        pub fn play(&mut self, data: &[u8]) {
            let size = std::mem::size_of::<WAVEHDR>() as u32;
            // SAFETY: the reset returns the header queued, which is then the driver's no longer;
            // the buffer and the header it points into are boxed, so neither moves while queued.
            unsafe {
                waveOutReset(self.handle);
                if let Some(header) = self.header.as_mut() {
                    waveOutUnprepareHeader(self.handle, &mut **header, size);
                }
                let mut buffer: Box<[u8]> = data.into();
                let mut header = Box::new(WAVEHDR {
                    lpData: buffer.as_mut_ptr(),
                    dwBufferLength: data.len() as u32,
                    ..Default::default()
                });
                waveOutPrepareHeader(self.handle, &mut *header, size);
                waveOutWrite(self.handle, &mut *header, size);
                self.buffer = Some(buffer);
                self.header = Some(header);
            }
        }
    }

    impl Drop for WaveOut {
        /// The clip playing is stopped before its buffer is freed: Python's garbage collector
        /// frees the old output's buffer as a new rate opens another, under the driver.
        fn drop(&mut self) {
            // SAFETY: as `play`'s.
            unsafe {
                waveOutReset(self.handle);
                if let Some(header) = self.header.as_mut() {
                    waveOutUnprepareHeader(
                        self.handle,
                        &mut **header,
                        std::mem::size_of::<WAVEHDR>() as u32,
                    );
                }
            }
        }
    }
}

#[cfg(not(windows))]
mod waveout {
    use crate::py::{Error, Kind, Result};

    pub struct WaveOut {
        pub rate: i64,
    }

    impl WaveOut {
        pub fn new(_rate: i64) -> Result<Self> {
            Err(Error::new(Kind::OsError, "no sound device for clips"))
        }

        pub fn play(&mut self, _data: &[u8]) {}
    }
}
