//! How loud Azahar plays the game, so pa3ds's own sounds are as loud beside it: its footsteps
//! (`clips`) and its tones (`tones`), each times its own `[pa3ds.audio]` setting.
//!
//! Azahar's volume slider cubed, as Azahar scales its output (`audio_core/dsp_interface.cpp`),
//! times its slider in Windows' volume mixer (`mixer`), which macOS and Linux have none of, as
//! Windows has none where COM finds no session. Read on a thread of its own once a second
//! from the moment it is made, as the game starts, so the first sound is already as loud as the
//! rest; the slider out of Azahar's `qt-config.ini` whenever that changes.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use crate::core::mixer;
use crate::py;

pub const AZAHAR: &str = "azahar.exe";
/// Seconds between reads.
pub const LOOK: f64 = 1.0;

/// Azahar's two sliders, kept up to date. Clones share one reading: the thread writes it, the
/// tones' thread and the footsteps read it.
#[derive(Debug, Clone)]
pub struct Loudness {
    azahar: Arc<AtomicU64>,
}

impl Loudness {
    pub fn new(qt_config: &Path) -> Self {
        let loudness = Loudness {
            azahar: Arc::new(AtomicU64::new(1.0f64.to_bits())),
        };
        let (shared, qt_config) = (loudness.azahar.clone(), qt_config.to_path_buf());
        // A thread that cannot start leaves the game's loudness at full, as before it read.
        let _ = std::thread::Builder::new()
            .name("loudness".to_string())
            .spawn(move || watch(&qt_config, &shared));
        loudness
    }

    /// The game at full loudness, with no Azahar watched: a test's, where Python's fakes the
    /// sounds whole and `new`'s thread would read this machine's mixer.
    #[cfg(test)]
    pub(crate) fn full() -> Self {
        Loudness {
            azahar: Arc::new(AtomicU64::new(1.0f64.to_bits())),
        }
    }

    /// How loud Azahar plays the game, 0 to 1.
    pub fn azahar(&self) -> f64 {
        f64::from_bits(self.azahar.load(Ordering::Relaxed))
    }
}

fn watch(qt_config: &PathBuf, azahar: &AtomicU64) {
    let (mut changed, mut slider): (Option<SystemTime>, f64) = (None, 1.0);
    loop {
        match fs::metadata(qt_config).and_then(|found| found.modified()) {
            Ok(stamp) => {
                if changed != Some(stamp) {
                    match fs::read(qt_config) {
                        Ok(text) => {
                            (changed, slider) =
                                (Some(stamp), azahar_volume(&String::from_utf8_lossy(&text)));
                        }
                        Err(_) => slider = 1.0,
                    }
                }
            }
            Err(_) => slider = 1.0,
        }
        let mixed = mixer::slider(AZAHAR);
        // ponytail: Azahar's mute key and its slider moved without saving are not read.
        let loud = slider.powi(3) * mixed.unwrap_or(1.0);
        azahar.store(loud.to_bits(), Ordering::Relaxed);
        py::time::sleep(LOOK);
    }
}

/// `[Audio] volume` of Azahar's `qt-config.ini`, 0 to 1; 1 where it says none.
pub fn azahar_volume(config: &str) -> f64 {
    for line in py::splitlines(config) {
        if let Some(value) = line.strip_prefix("volume=") {
            return match py::float_of(value) {
                // `min(max(value, 0.0), 1.0)`, a NaN kept where Python keeps it.
                Ok(value) => {
                    let value = if 0.0 > value { 0.0 } else { value };
                    if 1.0 < value { 1.0 } else { value }
                }
                Err(_) => 1.0,
            };
        }
    }
    1.0
}
