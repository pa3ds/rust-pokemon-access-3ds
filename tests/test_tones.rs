//! pa3ds's own tones: a cursor's a sine, a moment's a triangle.

use crate::core::tones::{FADE, LENGTH, RATE, Voice, Wave, sine, triangle};
use crate::py;

/// How loud a tone is on average against its peak, past its fade in: a sine's is 0.71, a
/// triangle's 0.58, its sound spread over more of its harmonics.
fn evenness(wave: Wave) -> f64 {
    let mut voice = Voice::new();
    voice.play(0.0, 0.5, wave);
    let rendered = voice
        .render(py::round(RATE as f64 * LENGTH).unwrap(), 1.0)
        .unwrap();
    let left: Vec<f64> = rendered
        .chunks_exact(4)
        .map(|frame| f64::from(i16::from_le_bytes([frame[0], frame[1]])))
        .collect();
    let left = &left[py::round(RATE as f64 * FADE).unwrap() as usize + 1..];
    let loudest = left.iter().map(|s| s.abs()).fold(0.0, f64::max);
    (left.iter().map(|s| s * s).sum::<f64>() / left.len() as f64).sqrt() / loudest
}

/// A clock's tick, the way up walks turning and a trainer's count as beeps were hard to hear
/// as sines, which sound softer than they are.
#[test]
fn test_a_moment_is_a_triangle_and_a_cursor_a_sine() {
    assert!(evenness(sine) > 0.68);
    assert!(evenness(triangle) < 0.62);
}
