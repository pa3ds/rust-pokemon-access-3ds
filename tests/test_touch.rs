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

/// Let go part way to a goal, a Charged Shot flew to between two goals and missed
/// (as_super_training_more_l1_regimens).
#[test]
fn test_a_held_press_let_go_while_it_slides_lets_go_where_it_was_sliding_to() {
    let server = touch::TouchServer::new(Arc::new(|_line: &str| {}), 0).unwrap();
    let port = server.port().unwrap();
    let client = UdpSocket::bind((touch::HOST, 0)).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    let mut ask = b"DSUC".to_vec();
    ask.extend_from_slice(&1001u16.to_le_bytes());
    ask.extend_from_slice(&12u16.to_le_bytes());
    ask.extend_from_slice(&0u32.to_le_bytes());
    ask.extend_from_slice(&1u32.to_le_bytes());
    ask.extend_from_slice(&(touch::PAD_DATA as u32).to_le_bytes());
    ask.extend_from_slice(&[0; 8]);
    client.send_to(&ask, (touch::HOST, port as u16)).unwrap();
    time::sleep(0.05);
    server.hold(95, 23);
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
    assert_eq!(*touches(0.1).last().unwrap(), (1, 0, 95, 23));
    server.r#move(160, 120);
    server.release();
    let touched = touches(touch::MOVE + 0.2);
    let pressed: Vec<(u8, u8, i64, i64)> = touched
        .iter()
        .copied()
        .filter(|state| state.0 != 0)
        .collect();
    assert!(*pressed.last().unwrap() == (1, 0, 160, 120) && touched.last().unwrap().0 == 0);
    let last = touched
        .iter()
        .position(|state| state == pressed.last().unwrap())
        .unwrap();
    assert!(touched[..last].iter().all(|state| state.0 != 0));
    server.close();
}

/// Pokémon-Amie's Pokémon turns round only to separate presses, and the stroke key pressed
/// again mid-stroke had carried one press on for 16 s (spike 276): a new slide lets go for
/// `LIFT` before it presses.
#[test]
fn test_a_press_asked_for_while_one_is_down_lifts_first() {
    let server = touch::TouchServer::new(Arc::new(|_line: &str| {}), 0).unwrap();
    let port = server.port().unwrap();
    let client = UdpSocket::bind((touch::HOST, 0)).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    let mut ask = b"DSUC".to_vec();
    ask.extend_from_slice(&1001u16.to_le_bytes());
    ask.extend_from_slice(&12u16.to_le_bytes());
    ask.extend_from_slice(&0u32.to_le_bytes());
    ask.extend_from_slice(&1u32.to_le_bytes());
    ask.extend_from_slice(&(touch::PAD_DATA as u32).to_le_bytes());
    ask.extend_from_slice(&[0; 8]);
    client.send_to(&ask, (touch::HOST, port as u16)).unwrap();
    time::sleep(0.05);
    server.slide(&[(100, 100), (200, 100)], 1.0);
    time::sleep(0.2);
    server.slide(&[(50, 50)], 0.5);
    let mut pressed: Vec<u8> = Vec::new();
    let deadline = time::monotonic() + 0.4;
    let mut buffer = [0u8; 100];
    while time::monotonic() < deadline {
        let size = client.recv(&mut buffer).unwrap();
        pressed.push(buffer[20..size][36]);
    }
    // `itertools.groupby`: each run of one state, with its length.
    let mut runs: Vec<(u8, usize)> = Vec::new();
    for down in pressed {
        match runs.last_mut() {
            Some((state, count)) if *state == down => *count += 1,
            _ => runs.push((down, 1)),
        }
    }
    let lifted: Vec<usize> = runs
        .iter()
        .filter(|(down, _)| *down == 0)
        .map(|&(_, count)| count)
        .collect();
    assert!(
        runs.last().unwrap().0 == 1 && !lifted.is_empty(),
        "{runs:?}"
    );
    assert!(
        *lifted.iter().max().unwrap() as f64 >= touch::LIFT / touch::TICK * 0.6,
        "{runs:?}"
    );
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

#[test]
fn test_a_rub_goes_back_and_forth_between_its_ends() {
    let ends = ((100, 50), (120, 50));
    assert_eq!(
        [0.0, 10.0, 20.0, 30.0, 40.0, 45.0]
            .iter()
            .map(|&travelled| touch::rubbed(ends, travelled))
            .collect::<Vec<_>>(),
        [
            (100, 50),
            (110, 50),
            (120, 50),
            (110, 50),
            (100, 50),
            (105, 50)
        ]
    );
    assert_eq!(touch::rubbed(((7, 7), (7, 7)), 99.0), (7, 7));
}

/// Pokémon-Amie counts a pet afresh at every press (spike 287): a tap's rub lasts its second,
/// another before it ends carries it on between the new ends, still down, and a rub held until
/// let go is let go by `release`.
#[test]
fn test_a_rub_asked_for_while_one_lasts_carries_it_on_never_lifting() {
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
    assert_eq!(server.rub(((100, 50), (130, 50)), Some(0.3), 0.0), None);
    let first = touches(0.2);
    server.rub(((200, 80), (230, 80)), Some(0.3), 0.0);
    let second = touches(0.4);
    let both: Vec<(u8, u8, i64, i64)> = first.iter().chain(&second).copied().collect();
    let pressed: Vec<&(u8, u8, i64, i64)> = both.iter().filter(|state| state.0 != 0).collect();
    let downs: std::collections::HashSet<i64> = pressed.iter().map(|state| state.3).collect();
    assert_eq!(downs, std::collections::HashSet::from([50, 80]));
    let across: std::collections::HashSet<i64> = pressed
        .iter()
        .filter(|state| state.3 == 50)
        .map(|state| state.2)
        .collect();
    assert!(across.len() > 3);
    // Down from the first press on to the end of the second, and up after it.
    let down: Vec<u8> = both.iter().map(|state| state.0).collect();
    let from = down.iter().position(|&state| state == 1).unwrap();
    assert!(*down.last().unwrap() == 0 && !down[from..first.len() + second.len() / 2].contains(&0));
    server.rub(((10, 10), (40, 10)), None, 0.1);
    assert_eq!(touches(0.05).last().unwrap().0, 0); // not yet down
    assert_eq!(touches(0.3).last().unwrap().0, 1);
    server.release();
    assert_eq!(touches(0.1).last().unwrap().0, 0);
    server.close();
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

/// A client that has asked `server` for the pad, as Azahar does.
fn listened(server: &touch::TouchServer) -> UdpSocket {
    let client = UdpSocket::bind((touch::HOST, 0)).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    let mut ask = b"DSUC".to_vec();
    ask.extend_from_slice(&1001u16.to_le_bytes());
    ask.extend_from_slice(&12u16.to_le_bytes());
    ask.extend_from_slice(&0u32.to_le_bytes());
    ask.extend_from_slice(&1u32.to_le_bytes());
    ask.extend_from_slice(&(touch::PAD_DATA as u32).to_le_bytes());
    ask.extend_from_slice(&[0; 8]);
    client
        .send_to(&ask, (touch::HOST, server.port().unwrap() as u16))
        .unwrap();
    time::sleep(0.05);
    client
}

/// A tap on Pokémon-Amie's stylus rubs a second, and an arrow moving the stylus in that second
/// carries the rub on; but a rub something else ended stays ended. Tapping "beckon" in the
/// second, or pressing Shift and letting it go as to quiet a screen reader, ended it, and the
/// next arrow pressed the screen again with no tap (ADR-0009: a touch only on the player's tap
/// or drag; a review of feat/amie-greys, 2026-11-08).
#[test]
fn test_an_arrow_never_starts_again_a_rub_something_else_ended() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::games::common::touch::{DRAG, ITEM_PREVIOUS, ModeOptions, TAP, Target, TouchMode};

    fn stylus(x: i64) -> Target {
        Target {
            rub: Some(((x - 10, 100), (x + 10, 100))),
            ..Target::new(2, "stylus, head", x, 100, 0.0, 240.0)
        }
    }

    let beckon = Target {
        path: vec![(160, 130)],
        seconds: 0.4,
        ..Target::new(1, "beckon", 160, 120, 0.0, 240.0)
    };
    for ended_by in ["beckon", "shift"] {
        let server = Rc::new(touch::TouchServer::new(Arc::new(|_line: &str| {}), 0).unwrap());
        let client = listened(&server);
        let found = Rc::new(RefCell::new(vec![beckon.clone(), stylus(160)]));
        let given = found.clone();
        let mut mode = TouchMode::new(
            Rc::new(move || Ok(given.borrow().clone())),
            Rc::new(|_line: &str| {}),
            ModeOptions {
                screen: Some(server.clone()),
                ..ModeOptions::default()
            },
        );
        mode.choose(2).unwrap();
        assert_eq!(mode.press(TAP).unwrap(), "");
        if ended_by == "beckon" {
            mode.press(ITEM_PREVIOUS).unwrap();
            mode.press(TAP).unwrap();
        } else {
            mode.press(DRAG).unwrap();
            mode.drop();
        }
        assert_eq!(server.rub_until().0, None);
        found.borrow_mut()[1] = stylus(166);
        mode.choose(2).unwrap(); // an arrow, as `amie.StylusKeys` moves the stylus
        assert_eq!(server.rub_until().0, None, "{ended_by}");
        server.close();
        drop(client);
    }
}

/// Carried on, a rub keeps its press down and the time it has left (Amie counts a pet afresh
/// at every press); with none lasting, carrying one on presses nothing.
#[test]
fn test_a_rub_that_lasts_is_carried_on_between_new_ends() {
    let server = touch::TouchServer::new(Arc::new(|_line: &str| {}), 0).unwrap();
    let client = listened(&server);
    assert!(!server.rub_on(((10, 10), (30, 10))));
    assert!(!server.rubbing());
    server.rub(((10, 10), (30, 10)), Some(1.0), 0.0);
    assert!(server.rubbing());
    let until = server.rub_until().1;
    assert!(server.rub_on(((20, 10), (40, 10))));
    assert!(!server.rub_on(((20, 10), (40, 10)))); // the same ends: nothing new
    assert_eq!(server.rub_until().0, Some(((20, 10), (40, 10))));
    assert_eq!(server.rub_until().1, until);
    server.release();
    assert!(!server.rubbing());
    server.close();
    drop(client);
}
