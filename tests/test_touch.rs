//! The touches pa3ds sends, as Azahar's cemuhook client reads them (`pa3ds.core.touch`).
//!
//! What the client checks is Azahar's `input_common/udp/protocol.cpp`: the server's magic,
//! protocol 1001, a payload length of the type and the data, and a CRC-32 over the whole packet
//! with the CRC field zeroed. A pad's state is 80 bytes, its first touch 36 bytes in.

use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;

use flate2::Crc;

use crate::core::{tones, touch};
use crate::py::time;

fn u16_at(data: &[u8], at: usize) -> i64 {
    i64::from(u16::from_le_bytes(data[at..at + 2].try_into().unwrap()))
}

fn u32_at(data: &[u8], at: usize) -> i64 {
    i64::from(u32::from_le_bytes(data[at..at + 4].try_into().unwrap()))
}

fn s16_at(data: &[u8], at: usize) -> i64 {
    i64::from(i16::from_le_bytes(data[at..at + 2].try_into().unwrap()))
}

#[test]
fn test_a_packet_is_one_azahar_takes() {
    let packet = touch::message(
        touch::PAD_DATA,
        &touch::pad_state(7, Some((285, 53))).unwrap(),
        0x1234,
    )
    .unwrap();
    assert_eq!(&packet[..4], b"DSUS");
    let (version, length, crc, server, kind) = (
        u16_at(&packet, 4),
        u16_at(&packet, 6),
        u32_at(&packet, 8),
        u32_at(&packet, 12),
        u32_at(&packet, 16),
    );
    assert_eq!((version, server, kind), (1001, 0x1234, touch::PAD_DATA));
    assert!(length == 80 + 4 && packet.len() == 20 + 80);
    let mut zeroed = Crc::new();
    zeroed.update(&packet[..8]);
    zeroed.update(&[0; 4]);
    zeroed.update(&packet[12..]);
    assert_eq!(crc, i64::from(zeroed.sum()));
    let data = &packet[20..];
    assert_eq!(u32_at(data, 12), 7);
    assert_eq!(
        (data[36], data[37], u16_at(data, 38), u16_at(data, 40)),
        (1, 0, 285, 53)
    );
    assert_eq!(touch::pad_state(8, None).unwrap()[36], 0);
}

/// A tap reaches only a client that has asked for the pad, and ends in a release.
#[test]
fn test_a_tap_presses_and_lets_go_for_a_client_that_asked() {
    let server = touch::TouchServer::new(Arc::new(|_line: &str| {}), 0).unwrap();
    let port = server.port().unwrap();
    let client = UdpSocket::bind((touch::HOST, 0)).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    assert!(!server.listened());
    let mut ask = b"DSUC".to_vec();
    ask.extend_from_slice(&1001u16.to_le_bytes());
    ask.extend_from_slice(&12u16.to_le_bytes());
    ask.extend_from_slice(&0u32.to_le_bytes());
    ask.extend_from_slice(&1u32.to_le_bytes());
    ask.extend_from_slice(&(touch::PAD_DATA as u32).to_le_bytes());
    ask.extend_from_slice(&[0; 8]);
    client.send_to(&ask, (touch::HOST, port as u16)).unwrap();
    time::sleep(0.05);
    assert!(server.listened());
    server.tap(400, 10);
    let mut touched: Vec<(u8, u8, i64, i64)> = Vec::new();
    let deadline = time::monotonic() + 0.5;
    let mut buffer = [0u8; 100];
    while time::monotonic() < deadline {
        let size = client.recv(&mut buffer).unwrap();
        let data = &buffer[20..size];
        touched.push((data[36], data[37], u16_at(data, 38), u16_at(data, 40)));
    }
    let pressed: Vec<&(u8, u8, i64, i64)> = touched.iter().filter(|state| state.0 != 0).collect();
    assert!(!pressed.is_empty() && *pressed[0] == (1, 0, 319, 10));
    assert_eq!(touched.last().unwrap().0, 0);
    server.close();
}

/// A drag: Poké Puffs are dragged to the Pokémon (ADR-0009). Held, a press is sent until it is
/// let go; moved, it slides on to its new point still down; and one never let go is let go after
/// `HELD` seconds.
#[test]
fn test_a_held_press_stays_down_moves_and_lets_go_when_released_or_held_too_long() {
    let server = touch::TouchServer::new(Arc::new(|_line: &str| {}), 0).unwrap();
    let port = server.port().unwrap();
    let client = UdpSocket::bind((touch::HOST, 0)).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    let touches = |seconds: f64| -> Vec<(u8, u8, i64, i64)> {
        let mut out: Vec<(u8, u8, i64, i64)> = Vec::new();
        let deadline = time::monotonic() + seconds;
        let mut buffer = [0u8; 100];
        while time::monotonic() < deadline {
            let size = client.recv(&mut buffer).unwrap();
            let data = &buffer[20..size];
            out.push((data[36], data[37], u16_at(data, 38), u16_at(data, 40)));
        }
        out
    };
    let mut ask = b"DSUC".to_vec();
    ask.extend_from_slice(&1001u16.to_le_bytes());
    ask.extend_from_slice(&12u16.to_le_bytes());
    ask.extend_from_slice(&0u32.to_le_bytes());
    ask.extend_from_slice(&1u32.to_le_bytes());
    ask.extend_from_slice(&(touch::PAD_DATA as u32).to_le_bytes());
    ask.extend_from_slice(&[0; 8]);
    client.send_to(&ask, (touch::HOST, port as u16)).unwrap();
    time::sleep(0.05);
    assert_eq!(server.hold(95, 23), None);
    touches(0.05);
    let held: std::collections::HashSet<(u8, u8, i64, i64)> = touches(0.3).into_iter().collect();
    assert_eq!(held, std::collections::HashSet::from([(1, 0, 95, 23)]));
    server.r#move(160, 120);
    let moving = touches(touch::MOVE + 0.2);
    assert!(moving.iter().all(|state| state.0 != 0) && *moving.last().unwrap() == (1, 0, 160, 120));
    let across: Vec<i64> = moving.iter().map(|state| state.2).collect();
    let mut sorted = across.clone();
    sorted.sort();
    let distinct: std::collections::HashSet<i64> = across.iter().copied().collect();
    assert!(across == sorted && across[0] < 120 && distinct.len() > 3);
    server.release();
    assert_eq!(touches(0.1).last().unwrap().0, 0);
    server.r#move(50, 50);
    assert_eq!(touches(0.1).last().unwrap().0, 0);
    server.set_held(0.2);
    server.hold(10, 10);
    assert_eq!(touches(0.4).last().unwrap().0, 0);
    server.close();
}

/// Azahar takes a pad's state only when its counter is past every one it has taken since it
/// opened, from any server: one game read after another, or a pa3ds started in another's place,
/// had its taps dropped while it counted up from one again (2026-10-20).
#[test]
fn test_a_server_counts_on_past_the_one_before_it() {
    let mut ask = b"DSUC".to_vec();
    ask.extend_from_slice(&1001u16.to_le_bytes());
    ask.extend_from_slice(&12u16.to_le_bytes());
    ask.extend_from_slice(&0u32.to_le_bytes());
    ask.extend_from_slice(&1u32.to_le_bytes());
    ask.extend_from_slice(&(touch::PAD_DATA as u32).to_le_bytes());
    ask.extend_from_slice(&[0; 8]);

    let counters = |seconds: f64| -> Vec<i64> {
        let server = touch::TouchServer::new(Arc::new(|_line: &str| {}), 0).unwrap();
        let client = UdpSocket::bind((touch::HOST, 0)).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        client
            .send_to(&ask, (touch::HOST, server.port().unwrap() as u16))
            .unwrap();
        let mut out: Vec<i64> = Vec::new();
        let deadline = time::monotonic() + seconds;
        let mut buffer = [0u8; 100];
        while time::monotonic() < deadline {
            let size = client.recv(&mut buffer).unwrap();
            out.push(u32_at(&buffer[..size], 20 + 12));
        }
        server.close();
        out
    };

    let (first, second) = (counters(0.5), counters(0.2));
    let mut ordered = first.clone();
    ordered.sort();
    ordered.dedup();
    assert_eq!(first, ordered);
    assert!(second[0] > *first.last().unwrap());
}

fn tone(across: f64, down: f64) -> Vec<u8> {
    let mut voice = tones::Voice::new();
    voice.play(across, down);
    voice
        .render(
            crate::py::round(tones::RATE as f64 * tones::LENGTH).unwrap(),
            1.0,
        )
        .unwrap()
}

/// `[pa3ds.audio] tones` above 1 is louder (Orion, 2026-10-18); far above, the wave stops at
/// full scale instead of wrapping into noise.
#[test]
fn test_a_tone_is_as_loud_as_asked_and_never_wraps_round() {
    fn peak(gain: f64) -> i64 {
        let mut voice = tones::Voice::new();
        voice.play(0.5, 0.5);
        let data = voice
            .render(
                crate::py::round(tones::RATE as f64 * tones::LENGTH).unwrap(),
                gain,
            )
            .unwrap();
        (0..data.len())
            .step_by(2)
            .map(|at| s16_at(&data, at).abs())
            .max()
            .unwrap()
    }

    let ratio = peak(1.5) as f64 / peak(1.0) as f64;
    assert!(1.4 < ratio && ratio < 1.6, "{ratio}");
    assert_eq!(peak(0.0), 0);
    // full scale, panned to the middle
    assert_eq!(
        peak(20.0),
        crate::py::round(32767.0 * (std::f64::consts::PI / 4.0).cos()).unwrap()
    );
}

/// The left channel of rendered 16-bit stereo frames.
fn lefts(data: &[u8]) -> Vec<i64> {
    (0..data.len())
        .step_by(4)
        .map(|at| s16_at(data, at))
        .collect()
}

fn crossings(part: &[i64]) -> i64 {
    part.windows(2)
        .filter(|pair| pair[0] < 0 && 0 <= pair[1])
        .count() as i64
}

/// A stroke over Pokémon-Amie's Pokémon is a press moving along a path (spike 200).
#[test]
fn test_a_slide_walks_its_path_evenly_and_lets_go_at_its_end() {
    let path = [(110, 110), (210, 110), (210, 130)];
    assert_eq!(touch::along(&path, 10.0, 12.0, 10.0), Some((110, 110)));
    assert_eq!(touch::along(&path, 10.0, 12.0, 10.5), Some((160, 110)));
    assert_eq!(touch::along(&path, 10.0, 12.0, 11.5), Some((210, 120)));
    assert_eq!(touch::along(&path, 10.0, 12.0, 12.0), None);
    assert_eq!(touch::along(&[(5, 6)], 0.0, 1.0, 0.5), Some((5, 6)));
}

/// `pa3ds.core.tones`: a tone at the top left is in the left ear alone and higher than one at
/// the bottom right, in the right ear alone.
#[test]
fn test_a_tone_is_higher_up_the_screen_and_louder_on_its_side() {
    fn ears(data: &[u8]) -> (i64, i64) {
        let pairs: Vec<(i64, i64)> = (0..data.len())
            .step_by(4)
            .map(|at| (s16_at(data, at), s16_at(data, at + 2)))
            .collect();
        (
            pairs.iter().map(|(left, _)| left.abs()).max().unwrap(),
            pairs.iter().map(|(_, right)| right.abs()).max().unwrap(),
        )
    }

    let (top_left, bottom_right) = (tone(0.0, 0.0), tone(1.0, 1.0));
    assert!(ears(&top_left).1 == 0 && 0 < ears(&top_left).0);
    assert!(ears(&bottom_right).0 == 0 && 0 < ears(&bottom_right).1);
    let (centre_top, centre_bottom) = (tone(0.5, 0.0), tone(0.5, 1.0));
    // two octaves
    assert!((crossings(&lefts(&centre_top)) - 4 * crossings(&lefts(&centre_bottom))).abs() <= 2);
}

/// A slide from the top down drops two octaves across it: four times the cycles in its first
/// tenth as in its last.
#[test]
fn test_a_glide_slides_from_its_start_to_its_end() {
    let mut voice = tones::Voice::new();
    voice.glide((0.5, 0.0), (0.5, 1.0));
    let data = voice
        .render(
            crate::py::round(tones::RATE as f64 * tones::GLIDE).unwrap(),
            1.0,
        )
        .unwrap();
    let lefts = lefts(&data);
    let tenth = lefts.len() / 10;
    let (first, last) = (
        crossings(&lefts[..tenth]),
        crossings(&lefts[lefts.len() - tenth..]),
    );
    assert!(3 * last <= first && first <= 5 * last);
}

/// The map's cursor glides on while the circle pad is held, and a slide comes every screen poll,
/// a little over a fifth of a second (spike 175): the tone never drops out between them, and
/// fades once the slides stop (Orion, 2026-10-02).
#[test]
fn test_slides_a_poll_apart_sound_without_a_break() {
    let mut voice = tones::Voice::new();
    let poll = crate::py::round(tones::RATE as f64 * 0.25).unwrap();
    let mut data: Vec<u8> = Vec::new();
    for i in 0..4 {
        voice.glide((0.1 * i as f64, 0.5), (0.1 * (i + 1) as f64, 0.5));
        data.extend(voice.render(poll, 1.0).unwrap());
    }
    let lefts = lefts(&data);
    let cycle = crate::py::round(tones::RATE as f64 / tones::LOW).unwrap() as usize;
    let start = crate::py::round(tones::RATE as f64 * tones::FADE).unwrap() as usize;
    let loudest: Vec<i64> = (start..lefts.len() - cycle)
        .map(|i| lefts[i..i + cycle].iter().map(|v| v.abs()).max().unwrap())
        .collect();
    assert!(*loudest.iter().min().unwrap() as f64 > 0.8 * 32767.0 * tones::VOLUME);
    voice
        .render(
            crate::py::round(tones::RATE as f64 * (tones::HOLD + tones::FADE)).unwrap(),
            1.0,
        )
        .unwrap();
    assert!(!voice.sounding());
}
