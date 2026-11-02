//! RpcMemory tests, through the fake emulator.

use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_emulator::FakeEmulator;
use crate::core::memory::{Cache, ExactCache, Memory, PageCache, RpcMemory, phase};
use crate::core::rpc::{MAX_READ, Process, Rpc, UdpRpc};
use crate::py::{Error, Kind, Result};

const M: usize = MAX_READ as usize;

fn connect(emu: &FakeEmulator) -> Rc<dyn Rpc> {
    Rc::new(UdpRpc::connect(emu.port()).unwrap())
}

fn counting() -> Vec<u8> {
    (0..=255u8).cycle().take(M).collect()
}

fn filled(bytes: &[(i64, u8)]) -> HashMap<i64, Vec<u8>> {
    bytes
        .iter()
        .map(|&(base, byte)| (base, vec![byte; M]))
        .collect()
}

#[test]
fn test_a_small_read_is_one_packet() {
    let emu = FakeEmulator::new(HashMap::from([(0x08001000, counting())]));
    assert_eq!(
        RpcMemory::new(connect(&emu)).read(0x08001010, 4).unwrap(),
        [0x10, 0x11, 0x12, 0x13]
    );
    assert_eq!(emu.reads(), 1);
}

#[test]
fn test_a_read_longer_than_one_packet_is_joined_in_order() {
    let emu = FakeEmulator::new(filled(&[(0x08001000, 1), (0x08001400, 2), (0x08001800, 3)]));
    let data = RpcMemory::new(connect(&emu))
        .read(0x08001000, 2 * MAX_READ + 5)
        .unwrap();
    assert_eq!(data, [vec![1; M], vec![2; M], vec![3; 5]].concat());
}

#[test]
fn test_a_read_of_exactly_whole_packets_needs_no_remainder() {
    let emu = FakeEmulator::new(filled(&[(0x08001000, 7), (0x08001400, 8)]));
    let data = RpcMemory::new(connect(&emu))
        .read(0x08001000, 2 * MAX_READ)
        .unwrap();
    assert_eq!(data, [vec![7; M], vec![8; M]].concat());
    assert_eq!(emu.reads(), 2);
}

/// Every read was a packet of its own: the dialogue reader made 12.7 a poll (spike 62).
#[test]
fn test_reads_in_one_page_in_one_phase_fetch_it_once() {
    let emu = FakeEmulator::new(HashMap::from([(0x08001000, counting())]));
    let cache = PageCache::new(connect(&emu));
    phase(&cache, "dialogue", || {
        assert_eq!(cache.read(0x08001010, 4)?, [0x10, 0x11, 0x12, 0x13]);
        assert_eq!(cache.read(0x08001020, 2)?, [0x20, 0x21]);
        Ok(())
    })
    .unwrap();
    assert_eq!(emu.reads(), 1);
}

#[test]
fn test_a_new_phase_reads_the_game_as_it_is_now() {
    let emu = FakeEmulator::new(filled(&[(0x08001000, 1)]));
    let cache = PageCache::new(connect(&emu));
    phase(&cache, "dialogue", || {
        assert_eq!(cache.read(0x08001000, 1)?, [1]);
        Ok(())
    })
    .unwrap();
    emu.set_page(0x08001000, vec![2; M]);
    phase(&cache, "dialogue", || {
        assert_eq!(cache.read(0x08001000, 1)?, [2]);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_a_phase_fetches_the_pages_it_used_last_time_before_it_reads() {
    let pages: Vec<(i64, u8)> = (0..4)
        .map(|i| (0x08001000 + i * MAX_READ, i as u8 + 1))
        .collect();
    let emu = FakeEmulator::new(filled(&pages));
    let cache = PageCache::new(connect(&emu));
    phase(&cache, "screen", || {
        for i in 0..4 {
            cache.read(0x08001000 + i * MAX_READ, 1)?;
        }
        Ok(())
    })
    .unwrap();
    phase(&cache, "screen", || {
        let fetched = emu.reads();
        for i in 0..4 {
            assert_eq!(cache.read(0x08001000 + i * MAX_READ, 1)?, [i as u8 + 1]);
        }
        assert_eq!(emu.reads(), fetched);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_a_read_across_pages_is_joined_in_order() {
    let emu = FakeEmulator::new(filled(&[(0x08001000, 1), (0x08001400, 2), (0x08001800, 3)]));
    let cache = PageCache::new(connect(&emu));
    let data = phase(&cache, "dialogue", || {
        cache.read(0x08001000 + MAX_READ - 2, MAX_READ + 4)
    })
    .unwrap();
    assert_eq!(data, [vec![1; 2], vec![2; M], vec![3; 2]].concat());
}

#[test]
fn test_a_read_outside_any_phase_is_never_served_stale() {
    let emu = FakeEmulator::new(filled(&[(0x08001000, 1)]));
    let cache = PageCache::new(connect(&emu));
    assert_eq!(cache.read(0x08001000, 1).unwrap(), [1]);
    emu.set_page(0x08001000, vec![2; M]);
    assert_eq!(cache.read(0x08001000, 1).unwrap(), [2]);
}

/// A dead pane's parent pointed near the top of the address space, and the read of it could
/// not even be packed into a request (movie3 replay, 2026-09-17).
#[test]
fn test_a_read_past_the_address_space_reads_as_zeros_without_being_sent() {
    let emu = FakeEmulator::new(HashMap::new());
    let rpc = connect(&emu);
    assert_eq!(
        PageCache::new(rpc.clone()).read(0xFFFFFFF8, 0x100).unwrap(),
        vec![0; 0x100]
    );
    phase(&PageCache::new(rpc.clone()), "screen", || {
        assert_eq!(
            PageCache::new(rpc.clone()).read(0xFFFFFFF8, 0x100)?,
            vec![0; 0x100]
        );
        Ok(())
    })
    .unwrap();
    assert_eq!(emu.reads(), 0);
}

/// Sun's heads, a read each, were 12.5 ms a look sent one after another (spike 123).
#[test]
fn test_an_exact_phase_fetches_the_reads_it_made_last_time_and_no_byte_more() {
    let emu = FakeEmulator::new(HashMap::from([(0x08001000, counting())]));
    let cache = ExactCache::new(connect(&emu), None);
    phase(&cache, "heaps", || {
        assert_eq!(cache.read(0x08001010, 4)?, [0x10, 0x11, 0x12, 0x13]);
        Ok(())
    })
    .unwrap();
    emu.set_page(0x08001000, vec![0xAA; M]);
    phase(&cache, "heaps", || {
        let before = emu.reads();
        assert_eq!(cache.read(0x08001010, 4)?, [0xAA; 4]);
        assert_eq!(cache.read(0x08001010, 2)?, [0xAA; 2]);
        assert_eq!(emu.reads(), before);
        assert_eq!(cache.read(0x08001014, 4)?, [0xAA; 4]);
        assert_eq!(emu.reads(), before + 1);
        Ok(())
    })
    .unwrap();
    assert_eq!(cache.read(0x08001010, 4).unwrap(), [0xAA; 4]);
}

/// A read of last poll's may since lie in a block that may not be read (ADR-0035).
#[test]
fn test_an_exact_phase_fetches_ahead_only_the_reads_still_within_bounds() {
    let emu = FakeEmulator::new(filled(&[(0x08001000, 0x11), (0x08002000, 0x22)]));
    let allowed = HashSet::from([0x08001010]);
    let cache = ExactCache::new(
        connect(&emu),
        Some(Rc::new(move |at, _size| allowed.contains(&at))),
    );
    phase(&cache, "screen", || {
        cache.read(0x08001010, 4)?;
        cache.read(0x08002010, 4)?;
        Ok(())
    })
    .unwrap();
    let before = emu.reads();
    phase(&cache, "screen", || {
        assert_eq!(emu.reads(), before + 1);
        assert_eq!(cache.read(0x08001010, 4)?, [0x11; 4]);
        assert_eq!(emu.reads(), before + 1);
        assert_eq!(cache.read(0x08002010, 4)?, [0x22; 4]);
        assert_eq!(emu.reads(), before + 2);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_reads_in_a_burst_come_back_by_address_from_either_cache() {
    let emu = FakeEmulator::new(filled(&[(0x08001000, 0x11), (0x08002000, 0x22)]));
    let rpc = connect(&emu);
    let expected: HashMap<i64, Vec<u8>> =
        HashMap::from([(0x08001010, vec![0x11; 2]), (0x080023F0, vec![0x22; 4])]);
    let as_map = |got: IndexMap<i64, Vec<u8>>| got.into_iter().collect::<HashMap<_, _>>();
    let reads = [(0x08001010, 2), (0x080023F0, 4)];
    assert_eq!(
        as_map(PageCache::new(rpc.clone()).read_each(&reads).unwrap()),
        expected
    );
    assert_eq!(
        as_map(
            ExactCache::new(rpc.clone(), None)
                .read_each(&reads)
                .unwrap()
        ),
        expected
    );
}

/// The dialogue reader followed a null pointer, and its phase fetched page 0 ahead every poll
/// after: Azahar logged an unmapped read every 0.2 s through every session (2026-09-29).
#[test]
fn test_a_null_pointer_reads_as_zeros_and_is_never_fetched_ahead() {
    let emu = FakeEmulator::new(HashMap::new());
    let rpc = connect(&emu);
    let caches: [Box<dyn Cache>; 2] = [
        Box::new(PageCache::new(rpc.clone())),
        Box::new(ExactCache::new(rpc.clone(), None)),
    ];
    for cache in &caches {
        for _ in 0..2 {
            phase(cache.as_ref(), "dialogue", || {
                assert_eq!(cache.read(0, 0x10)?, vec![0; 0x10]);
                assert!(cache.read_each(&[(0x20, 4)])?.is_empty());
                Ok(())
            })
            .unwrap();
        }
    }
    assert_eq!(RpcMemory::new(rpc).read(0, 4).unwrap(), [0; 4]);
    assert_eq!(emu.reads(), 0);
}

/// An emulator answering every batch of reads but those sent while `stalled`.
struct Stalling {
    stalled: Cell<bool>,
    answered: Cell<i64>,
}

impl Rpc for Stalling {
    fn read_many(&self, addresses: &[i64], size: i64) -> Result<IndexMap<i64, Vec<u8>>> {
        if self.stalled.get() {
            return Err(Error::new(Kind::RpcError, "stalled"));
        }
        self.answered.set(self.answered.get() + 1);
        Ok(addresses
            .iter()
            .map(|&address| (address, vec![0; size as usize]))
            .collect())
    }

    fn read(&self, _address: i64, _size: i64) -> Result<Vec<u8>> {
        unreachable!("the Python's Stalling has no `read`")
    }

    fn read_each(&self, _reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        unreachable!("the Python's Stalling has no `read_each`")
    }

    fn reply_seconds(&self) -> f64 {
        unreachable!("the Python's Stalling has no `reply_seconds`")
    }

    fn reads_beside_gpu(&self) -> bool {
        unreachable!("the Python's Stalling has no `reads_beside_gpu`")
    }

    fn unsure(&self) -> bool {
        unreachable!("the Python's Stalling has no `unsure`")
    }

    fn confirm(&self) -> Result<bool> {
        unreachable!("the Python's Stalling has no `confirm`")
    }

    fn attach(&self) -> Result<Process> {
        unreachable!("the Python's Stalling has no `attach`")
    }
}

#[test]
fn test_a_phase_the_emulator_stalls_as_it_starts_leaves_nothing_cached() {
    let rpc = Rc::new(Stalling {
        stalled: Cell::new(false),
        answered: Cell::new(0),
    });
    let cache = PageCache::new(rpc.clone());
    phase(&cache, "screen", || cache.read(0x08001000, 4)).unwrap();
    rpc.stalled.set(true);
    let error = phase(&cache, "screen", || Ok(())).unwrap_err();
    assert!(error.is(Kind::RpcError));
    rpc.stalled.set(false);
    cache.read(0x08001000, 4).unwrap();
    cache.read(0x08001000, 4).unwrap();
    assert_eq!(rpc.answered.get(), 3);
}
