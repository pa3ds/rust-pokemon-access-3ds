//! The sound output off Windows, where `tones` and `clips` play on waveOut: the default output
//! device, CoreAudio's on macOS and ALSA's on Linux, which PipeWire and PulseAudio also serve.
//!
//! Both write 16-bit stereo at a rate of their own, as waveOut takes it. A device need not take
//! that rate or that format, so the output opens at the device's own default and converts: drawn
//! straight between the frames to the device's rate (`Resampler`), as `clips.resampled` draws a
//! game's sound, and to its channels and sample format. What is written waits in a queue the
//! device's callback takes from, and plays silence while it is empty, as waveOut does with
//! nothing queued.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, SampleFormat, SizedSample, StreamConfig};

use crate::py::{Error, Kind, Result};

/// Converts 16-bit stereo at one rate to float samples at another, drawn straight between the
/// frames, carrying where it is from one write to the next so a tone written a buffer at a time
/// plays as one.
pub struct Resampler {
    /// Frames of the input a frame of the output moves on.
    step: f64,
    /// Where the next output frame falls, in frames from the start of the next write: -1 is the
    /// last frame of the write before, `last`.
    at: f64,
    last: [f32; 2],
}

impl Resampler {
    pub fn new(from: u32, to: u32) -> Self {
        Resampler {
            step: from as f64 / to as f64,
            at: 0.0,
            last: [0.0; 2],
        }
    }

    /// Starts again, nothing written before.
    pub fn reset(&mut self) {
        self.at = 0.0;
        self.last = [0.0; 2];
    }

    /// `data`, 16-bit stereo frames, little-endian, as `channels` float samples a frame: both
    /// sides' average for one channel, silence on any past the second.
    pub fn convert(&mut self, data: &[u8], channels: usize) -> Vec<f32> {
        let frames: Vec<[f32; 2]> = data
            .as_chunks::<4>()
            .0
            .iter()
            .map(|frame| {
                [
                    i16::from_le_bytes([frame[0], frame[1]]) as f32 / 32768.0,
                    i16::from_le_bytes([frame[2], frame[3]]) as f32 / 32768.0,
                ]
            })
            .collect();
        let count = frames.len() as f64;
        let mut out: Vec<f32> = Vec::with_capacity((count / self.step) as usize * channels + 1);
        while self.at < count - 1.0 {
            let before = self.at.floor();
            let share = (self.at - before) as f32;
            let index = before as isize;
            let from = if index < 0 {
                self.last
            } else {
                frames[index as usize]
            };
            let to = frames[(index + 1) as usize];
            let left = from[0] + (to[0] - from[0]) * share;
            let right = from[1] + (to[1] - from[1]) * share;
            if channels == 1 {
                out.push((left + right) / 2.0);
            } else {
                out.push(left);
                out.push(right);
                out.extend(std::iter::repeat_n(0.0, channels.saturating_sub(2)));
            }
            self.at += self.step;
        }
        if let Some(&last) = frames.last() {
            self.at -= count;
            self.last = last;
        }
        out
    }
}

/// The default output device, open for good, playing what is written to it.
pub struct Output {
    queue: Arc<Mutex<VecDeque<f32>>>,
    resampler: Resampler,
    channels: usize,
    rate: u32,
    _stream: cpal::Stream,
}

impl Output {
    /// The default output at its own default config, taking 16-bit stereo at `rate`. An OSError
    /// where there is no device or it will not open: no sound, as waveOut's.
    pub fn open(rate: i64) -> Result<Self> {
        let failed = |what: String| Error::new(Kind::OsError, format!("no sound device: {what}"));
        let device = cpal::default_host()
            .default_output_device()
            .ok_or_else(|| failed("no default output".to_string()))?;
        let supported = device
            .default_output_config()
            .map_err(|error| failed(error.to_string()))?;
        let format = supported.sample_format();
        let config = supported.config();
        let queue: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::new()));
        // cpal leaves ALSA a tenth of a second buffered by default, more than doubling how late a
        // tone is heard: ask for a twenty-fifth, and take the default where ALSA will not.
        #[cfg(target_os = "linux")]
        let stream = match supported.buffer_size() {
            cpal::SupportedBufferSize::Range { min, max } => {
                let fixed = StreamConfig {
                    buffer_size: cpal::BufferSize::Fixed(
                        (config.sample_rate.0 / 25).clamp(*min, *max),
                    ),
                    ..config.clone()
                };
                build(&device, &fixed, format, &queue)
                    .or_else(|_| build(&device, &config, format, &queue))
            }
            cpal::SupportedBufferSize::Unknown => build(&device, &config, format, &queue),
        };
        #[cfg(not(target_os = "linux"))]
        let stream = build(&device, &config, format, &queue);
        let stream = stream.map_err(failed)?;
        stream.play().map_err(|error| failed(error.to_string()))?;
        Ok(Output {
            queue,
            resampler: Resampler::new(rate as u32, config.sample_rate.0),
            channels: config.channels as usize,
            rate: config.sample_rate.0,
            _stream: stream,
        })
    }

    /// Queue `data`, 16-bit stereo frames, after whatever is still to play.
    pub fn write(&mut self, data: &[u8]) {
        let samples = self.resampler.convert(data, self.channels);
        locked(&self.queue).extend(samples);
    }

    /// Drop whatever is still to play.
    pub fn clear(&mut self) {
        locked(&self.queue).clear();
        self.resampler.reset();
    }

    /// Seconds of what is written still to play.
    pub fn queued(&self) -> f64 {
        locked(&self.queue).len() as f64 / self.channels as f64 / self.rate as f64
    }
}

fn locked(queue: &Mutex<VecDeque<f32>>) -> MutexGuard<'_, VecDeque<f32>> {
    queue.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A stream on `device` in its sample format, filled from `queue`.
fn build(
    device: &Device,
    config: &StreamConfig,
    format: SampleFormat,
    queue: &Arc<Mutex<VecDeque<f32>>>,
) -> std::result::Result<cpal::Stream, String> {
    fn of<T: SizedSample + FromSample<f32>>(
        device: &Device,
        config: &StreamConfig,
        queue: &Arc<Mutex<VecDeque<f32>>>,
    ) -> std::result::Result<cpal::Stream, String> {
        let queue = queue.clone();
        device
            .build_output_stream(
                config,
                move |out: &mut [T], _: &cpal::OutputCallbackInfo| {
                    let mut queue = locked(&queue);
                    for sample in out.iter_mut() {
                        *sample = T::from_sample(queue.pop_front().unwrap_or(0.0));
                    }
                },
                |_| {},
                None,
            )
            .map_err(|error| error.to_string())
    }
    match format {
        SampleFormat::F32 => of::<f32>(device, config, queue),
        SampleFormat::F64 => of::<f64>(device, config, queue),
        SampleFormat::I8 => of::<i8>(device, config, queue),
        SampleFormat::I16 => of::<i16>(device, config, queue),
        SampleFormat::I32 => of::<i32>(device, config, queue),
        SampleFormat::U8 => of::<u8>(device, config, queue),
        SampleFormat::U16 => of::<u16>(device, config, queue),
        SampleFormat::U32 => of::<u32>(device, config, queue),
        other => Err(format!("sample format {other} not played")),
    }
}

#[cfg(test)]
mod checks {
    use super::*;
    use cpal::Sample;

    fn stereo(frames: &[(i16, i16)]) -> Vec<u8> {
        frames
            .iter()
            .flat_map(|&(left, right)| [left.to_le_bytes(), right.to_le_bytes()].concat())
            .collect()
    }

    #[test]
    fn a_rate_doubled_draws_a_frame_between_each_two_and_carries_on_across_writes() {
        let mut resampler = Resampler::new(22050, 44100);
        let first = resampler.convert(&stereo(&[(0, 16384), (16384, 0)]), 2);
        assert_eq!(first, [0.0, 0.5, 0.25, 0.25]);
        // The next write starts between the last frame of the one before and its own first.
        let second = resampler.convert(&stereo(&[(-16384, 0)]), 2);
        assert_eq!(second, [0.5, 0.0, 0.0, 0.0]);
        let third = resampler.convert(&stereo(&[(0, 0)]), 2);
        assert_eq!(third, [-0.5, 0.0, -0.25, 0.0]);
        resampler.reset();
        assert_eq!(
            resampler.convert(&stereo(&[(16384, 16384)]), 2),
            [] as [f32; 0]
        );
    }

    #[test]
    fn a_device_s_channels_take_both_sides_averaged_or_silence_past_two() {
        let frame = stereo(&[(16384, -32768), (16384, -32768)]);
        assert_eq!(Resampler::new(48000, 48000).convert(&frame, 1), [-0.25]);
        assert_eq!(
            Resampler::new(48000, 48000).convert(&frame, 4),
            [0.5, -1.0, 0.0, 0.0]
        );
    }

    #[test]
    fn a_rate_lowered_takes_as_many_frames_as_the_device_plays() {
        let mut resampler = Resampler::new(48000, 44100);
        let mut total = 0;
        for _ in 0..100 {
            total += resampler.convert(&stereo(&[(0, 0); 480]), 2).len() / 2;
        }
        // A second at 48000 as a second at 44100, but for the last frame, held for a write to come.
        assert!((44099..=44101).contains(&total), "{total}");
    }

    #[test]
    fn silence_is_each_format_s_own() {
        assert_eq!(u16::from_sample(0.0f32), 32768);
        assert_eq!(i16::from_sample(-1.0f32), -32768);
        assert_eq!(u8::from_sample(0.0f32), 128);
    }
}
