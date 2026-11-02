//! Transport tests, against a fake emulator speaking the real wire format.

use std::collections::{HashMap, HashSet};

use super::fake_emulator::{FakeEmulator, sango};
use crate::core::rpc::{MAX_READ, QUIET, Rpc, UdpRpc};
use crate::py::{Kind, time};

fn page(base: i64, fill: &[u8]) -> HashMap<i64, Vec<u8>> {
    let mut data = fill.to_vec();
    data.resize(MAX_READ as usize, 0);
    HashMap::from([(base, data)])
}

fn rpc(emu: &FakeEmulator) -> UdpRpc {
    UdpRpc::connect(emu.port()).unwrap()
}

fn quick(emu: &FakeEmulator) -> UdpRpc {
    UdpRpc::new("127.0.0.1", emu.port(), 0.05, 16).unwrap()
}

#[test]
fn test_read_returns_the_bytes_at_that_address() {
    let emu = FakeEmulator::new(page(0x08608800, b"hello"));
    assert_eq!(rpc(&emu).read(0x08608800, 5).unwrap(), b"hello");
}

#[test]
fn test_read_crosses_no_pages_and_reads_zeros_where_nothing_is_mapped() {
    let emu = FakeEmulator::new(HashMap::new());
    assert_eq!(rpc(&emu).read(0x20000000, 16).unwrap(), vec![0; 16]);
}

#[test]
fn test_read_refuses_sizes_the_server_would_reject() {
    for size in [0, -1, MAX_READ + 1, 4096] {
        let emu = FakeEmulator::new(HashMap::new());
        let error = rpc(&emu).read(0x08000000, size).unwrap_err();
        assert!(error.is(Kind::ValueError), "{size}: {error:?}");
    }
}

#[test]
fn test_read_at_exactly_the_cap_is_allowed() {
    let emu = FakeEmulator::new(page(0x08000000, &[0xab; MAX_READ as usize]));
    assert_eq!(
        rpc(&emu).read(0x08000000, MAX_READ).unwrap(),
        vec![0xab; MAX_READ as usize]
    );
}

#[test]
fn test_read_many_returns_every_address_and_costs_one_packet_each() {
    let memory: HashMap<i64, Vec<u8>> = (0x100000..0x110000)
        .step_by(MAX_READ as usize)
        .map(|base: i64| (base, vec![(base >> 10 & 0xFF) as u8; MAX_READ as usize]))
        .collect();
    let emu = FakeEmulator::new(memory.clone());
    let bases: Vec<i64> = memory.keys().copied().collect();
    let got = rpc(&emu).read_many(&bases, MAX_READ).unwrap();
    assert_eq!(
        got.keys().copied().collect::<HashSet<_>>(),
        memory.keys().copied().collect()
    );
    assert!(memory.iter().all(|(base, data)| &got[base] == data));
    assert_eq!(emu.reads(), memory.len() as i64);
}

#[test]
fn test_read_many_retries_dropped_replies() {
    let memory: HashMap<i64, Vec<u8>> = (0x100000..0x104000)
        .step_by(MAX_READ as usize)
        .map(|base: i64| (base, vec![1; MAX_READ as usize]))
        .collect();
    let emu = FakeEmulator::with(memory.clone(), sango(), 3, 0);
    let bases: Vec<i64> = memory.keys().copied().collect();
    let got = rpc(&emu).read_many(&bases, MAX_READ).unwrap();
    assert_eq!(
        got.keys().copied().collect::<HashSet<_>>(),
        memory.keys().copied().collect()
    );
    assert!(emu.reads() > memory.len() as i64);
}

/// Stalls were counted in total, so a sweep big enough to drop four packets always died.
#[test]
fn test_read_many_survives_drops_spread_through_a_long_run() {
    let memory: HashMap<i64, Vec<u8>> = (0x100000..0x120000)
        .step_by(MAX_READ as usize)
        .map(|base: i64| (base, Vec::new()))
        .collect();
    let emu = FakeEmulator::with(HashMap::new(), sango(), 0, 7);
    let bases: Vec<i64> = memory.keys().copied().collect();
    let got = quick(&emu).read_many(&bases, MAX_READ).unwrap();
    assert_eq!(
        got.keys().copied().collect::<HashSet<_>>(),
        memory.keys().copied().collect()
    );
    assert!(emu.reads() > memory.len() as i64);
}

/// Windows reports a bounced UDP packet as a reset; it means one loss, not silence.
#[test]
fn test_a_refused_datagram_is_its_own_kind_of_timeout() {
    assert_eq!(Kind::RpcRefusedError.base(), Some(Kind::RpcTimeoutError));
}

#[test]
fn test_read_many_gives_up_rather_than_hanging_when_nothing_answers() {
    let emu = FakeEmulator::with(HashMap::new(), sango(), 10_000, 0);
    let error = quick(&emu)
        .read_many(&[0x08000000, 0x08000400], MAX_READ)
        .unwrap_err();
    assert!(error.is(Kind::RpcTimeoutError), "{error:?}");
}

#[test]
fn test_processes_decodes_pid_title_id_and_name() {
    let emu = FakeEmulator::new(HashMap::new());
    let processes = rpc(&emu).processes().unwrap();
    let [proc] = processes.as_slice() else {
        panic!("{processes:?}")
    };
    assert_eq!(proc.pid, 11);
    assert_eq!(proc.title_id, 0x000400000011C400);
    assert_eq!(proc.name, "sango-1");
}

#[test]
fn test_selected_process_is_none_until_told() {
    let emu = FakeEmulator::new(HashMap::new());
    let rpc = rpc(&emu);
    assert_eq!(rpc.selected_process().unwrap(), None);
    rpc.select_process(11).unwrap();
    assert_eq!(rpc.selected_process().unwrap(), Some(11));
}

#[test]
fn test_attach_selects_the_only_process() {
    let emu = FakeEmulator::new(HashMap::new());
    let rpc = rpc(&emu);
    assert_eq!(rpc.attach().unwrap().name, "sango-1");
    assert_eq!(rpc.selected_process().unwrap(), Some(11));
}

/// Azahar's RPC went from version 1 to 2 (azahar-emu/azahar#2631), and each answers the
/// other's packets with nothing: players' Azahar speaks 1, the rig's fork 2.
#[test]
fn test_attach_speaks_the_emulators_protocol_version() {
    for version in [1, 2] {
        let emu = FakeEmulator::with_version(page(0x0800_0000, b"hi"), sango(), 0, 0, version);
        let rpc = rpc(&emu);
        rpc.attach().unwrap();
        assert_eq!(emu.selected(), 11, "version {version}");
        assert_eq!(
            rpc.read(0x0800_0000, 2).unwrap(),
            b"hi",
            "version {version}"
        );
        // Only version 2 reads on the emulation thread, beside the GPU (spike 149).
        assert_eq!(rpc.reads_beside_gpu(), version == 2, "version {version}");
    }
}

#[test]
fn test_attach_refuses_to_guess_between_several_processes() {
    let two = vec![(11, 1, "sango-1".to_string()), (12, 2, "other".to_string())];
    let emu = FakeEmulator::with(HashMap::new(), two, 0, 0);
    let error = rpc(&emu).attach().unwrap_err();
    assert!(
        error.is(Kind::RpcError) && error.message.contains("found 2"),
        "{error:?}"
    );
}

#[test]
fn test_attach_reports_an_empty_emulator() {
    let emu = FakeEmulator::with(HashMap::new(), Vec::new(), 0, 0);
    let error = rpc(&emu).attach().unwrap_err();
    assert!(
        error.is(Kind::RpcError) && error.message.contains("no process"),
        "{error:?}"
    );
}

/// Request ids must not let one reply satisfy the next read.
#[test]
fn test_consecutive_reads_each_get_their_own_answer() {
    let emu = FakeEmulator::new(page(0x08000000, b"right"));
    let rpc = rpc(&emu);
    assert_eq!(rpc.read(0x08000000, 5).unwrap(), b"right");
    assert_eq!(rpc.read(0x08000000, 5).unwrap(), b"right");
    assert_eq!(emu.reads(), 2);
}

/// Leaving Mystery Gift or Live Competition restarts the game, and Azahar starts a new RPC
/// server with no process selected; a read reaching it before the game's process exists crashed
/// the emulator (movie13, 2026-09-29). `read_many` resent its stalled reads to it.
#[test]
fn test_a_retry_is_not_sent_to_an_emulator_that_restarted_the_game() {
    let emu = FakeEmulator::new(HashMap::new());
    let rpc = quick(&emu);
    rpc.attach().unwrap();
    rpc.read_many(&[0x08000000], MAX_READ).unwrap();
    emu.restart(1);
    let error = rpc.read_many(&[0x08000000], MAX_READ).unwrap_err();
    assert!(error.is(Kind::RpcTimeoutError), "{error:?}");
    assert_eq!(emu.unselected_reads(), 0);
}

/// A restart between two reads leaves no timeout behind to warn of it.
#[test]
fn test_no_read_is_sent_after_a_quiet_spell_without_asking_first() {
    let emu = FakeEmulator::new(HashMap::new());
    let rpc = rpc(&emu);
    rpc.attach().unwrap();
    rpc.read(0x08000000, 4).unwrap();
    emu.restart(0);
    time::sleep(QUIET * 2.0);
    let error = rpc.read(0x08000000, 4).unwrap_err();
    assert!(error.is(Kind::RpcTimeoutError), "{error:?}");
    assert_eq!(emu.unselected_reads(), 0);
}
