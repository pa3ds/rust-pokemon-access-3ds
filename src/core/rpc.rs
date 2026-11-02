//! Client for Azahar's RPC server.
//!
//! Written from the wire format in the emulator's own `src/core/rpc/packet.h`, not from the
//! `citra.py` client that ships alongside it. pa3ds only ever reads; `WriteMemory` exists in the
//! protocol and is deliberately not implemented here.
//!
//! ```text
//! header  16 bytes little-endian: version, id, packet type, data size
//! data    up to MAX_READ bytes
//! ```
//!
//! Azahar has spoken two versions of it, and answers a packet of the other with an empty reply:
//! 1 until azahar-emu/azahar#2631 (merged 2026-10-04), 2 since, which also runs every request on the
//! emulation thread and answers none while emulation is paused. `processes` finds which by asking.
//!
//! Per-read size is capped at MAX_READ and round-trip latency is flat in size, so a read should
//! always ask for the full page: packet count is the cost, not bytes. `read_many` pipelines a
//! whole batch and matches replies by the header id, which is roughly three times faster per read
//! than waiting for each reply in turn.
//!
//! Bound by:
//! - ADR-0001: the game is read over Azahar's RPC, and nothing patches the ROM.

use std::cell::{Cell, RefCell};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use indexmap::IndexMap;

use crate::py::{Error, Kind, Result, time};

pub const HEADER_SIZE: usize = 16;
pub const ARGS_SIZE: usize = 8;
pub const PROCESS_INFO_SIZE: usize = 20;

/// The versions the client speaks, the one tried first first.
pub const PROTOCOL_VERSIONS: [u32; 2] = [2, 1];
/// Largest payload the server will return in one packet.
pub const MAX_READ: i64 = 1024;
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 45987;
pub const MAX_PROCESSES: i64 = (MAX_READ - 4) / PROCESS_INFO_SIZE as i64;
/// Windows in a row that may go unanswered before `read_many` gives up on the emulator.
pub const MAX_STALLS: i64 = 6;
/// Seconds to wait per consecutive stall before resending, so a server that has fallen behind is
/// given room to catch up instead of being handed the same backlog again.
pub const BACKOFF: f64 = 0.25;
/// Bounced datagrams tolerated in one `read_many`. High because they cost nothing and come in
/// bursts; finite so a socket refusing everything cannot spin forever.
pub const MAX_RESETS: i64 = 10_000;
/// Seconds without a reply after which the next read first asks whether the game is still the one
/// selected. Restarting the game replaces the RPC server, and the new one serves a read with nothing
/// selected from no process at all, which crashes Azahar (movie13, 2026-09-29). The old server is
/// gone for at least 0.15 s before the new one answers, so a read that follows closer than this on a
/// reply cannot have crossed a restart.
pub const QUIET: f64 = 0.05;

/// These (low, high) spans as (address, size) reads of `MAX_READ` or less.
pub fn reads_of(spans: &[(i64, i64)]) -> Vec<(i64, i64)> {
    spans
        .iter()
        .flat_map(|&(low, high)| {
            (low..high)
                .step_by(MAX_READ as usize)
                .map(move |at| (at, MAX_READ.min(high - at)))
        })
        .collect()
}

/// The span's bytes out of the replies to `reads_of` it, or None where a read of it failed.
pub fn joined(replies: &IndexMap<i64, Vec<u8>>, low: i64, high: i64) -> Option<Vec<u8>> {
    let mut data = Vec::new();
    for at in (low..high).step_by(MAX_READ as usize) {
        if let Some(reply) = replies.get(&at) {
            data.extend_from_slice(reply);
        }
    }
    (data.len() as i64 == high - low).then_some(data)
}

const NO_PROCESS: u32 = 0xFFFF_FFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PacketType {
    ReadMemory = 1,
    WriteMemory = 2,
    ProcessList = 3,
    SetGetProcess = 4,
}

/// One entry from the emulator's process list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Process {
    pub pid: i64,
    pub title_id: i64,
    pub name: String,
}

/// What the runtime asks of the emulator: the public side of the Python `Rpc` class.
///
/// [`UdpRpc`] asks a running Azahar; a tape (`crate::tape`) answers from a recorded run; a test
/// may answer from memory. Methods take `&self` because readers share one connection, as every
/// Python reader held the same `Rpc`.
pub trait Rpc {
    /// Read `size` bytes. Sizes above MAX_READ are refused by the server, not truncated.
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>>;

    /// Read many equal-sized blocks, pipelined. Missing replies are retried.
    ///
    /// Returns only the addresses that answered; a caller that needs every one should check.
    fn read_many(&self, addresses: &[i64], size: i64) -> Result<IndexMap<i64, Vec<u8>>> {
        let reads: Vec<(i64, i64)> = addresses.iter().map(|&address| (address, size)).collect();
        self.read_each(&reads)
    }

    /// Read each (address, size), pipelined, keyed by address: `read_many` for reads of
    /// different sizes, where reading a byte more than asked is not safe (`games.sm.heaps`).
    fn read_each(&self, reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>>;

    /// How long the emulator took to answer each read of the last request, or of the last
    /// burst: 0.01 to 0.05 ms, and 2 to 20 ms while it loads an area (sun_10.ctm, 2026-09-27).
    fn reply_seconds(&self) -> f64;

    /// Whether memory the GPU draws into can be read safely: a server of version 2 serves
    /// every read on the emulation thread (azahar-emu/azahar#2631, in Azahar 2126.2), where
    /// version 1 flushes the GPU's copy from its own thread and can crash Azahar (spike 149,
    /// azahar-emu/azahar#2619). Known once `processes` has asked.
    fn reads_beside_gpu(&self) -> bool;

    /// Whether the emulator may have lost our selection since it last answered: it stalled,
    /// or has been quiet for `QUIET`. Never, before a process is selected.
    fn unsure(&self) -> bool;

    /// Whether the emulator still has our process selected. Asking is safe with no game
    /// running, which a read is not.
    fn confirm(&self) -> Result<bool>;

    /// Select the only running process and return it.
    fn attach(&self) -> Result<Process>;
}

/// A UDP connection to one running emulator.
pub struct UdpRpc {
    addr: SocketAddr,
    window: usize,
    reply_seconds: Cell<f64>,
    version: Cell<u32>,
    next_id: Cell<u32>,
    pid: Cell<Option<i64>>,
    stalled: Cell<bool>,
    last_reply: Cell<f64>,
    sock: UdpSocket,
    buffer: RefCell<Vec<u8>>,
}

#[track_caller]
fn wire_u32(value: i64) -> Result<u32> {
    u32::try_from(value).map_err(|_| Error::unpack("'I' format requires 0 <= number <= 4294967295"))
}

fn args(first: i64, second: i64) -> Result<[u8; ARGS_SIZE]> {
    let mut out = [0u8; ARGS_SIZE];
    out[..4].copy_from_slice(&wire_u32(first)?.to_le_bytes());
    out[4..].copy_from_slice(&wire_u32(second)?.to_le_bytes());
    Ok(out)
}

impl UdpRpc {
    pub fn new(host: &str, port: u16, timeout: f64, window: usize) -> Result<Self> {
        let addr: SocketAddr = format!("{host}:{port}")
            .parse()
            .map_err(|_| Error::value(format!("not an address: {host}:{port}")))?;
        let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None)?;
        socket.set_recv_buffer_size(1 << 20)?;
        socket.bind(&SocketAddr::from(([0, 0, 0, 0], 0)).into())?;
        let sock: UdpSocket = socket.into();
        sock.set_read_timeout(Some(Duration::from_secs_f64(timeout)))?;
        Ok(UdpRpc {
            addr,
            window,
            reply_seconds: Cell::new(0.0),
            version: Cell::new(PROTOCOL_VERSIONS[0]),
            next_id: Cell::new(1),
            pid: Cell::new(None),
            stalled: Cell::new(false),
            last_reply: Cell::new(0.0),
            sock,
            buffer: RefCell::new(vec![0; HEADER_SIZE + MAX_READ as usize]),
        })
    }

    /// The emulator on this machine, at `port`, with the default timeout and window.
    pub fn connect(port: u16) -> Result<Self> {
        UdpRpc::new(DEFAULT_HOST, port, 0.25, 16)
    }

    // -- packets ----------------------------------------------------------------

    fn send(&self, packet_type: PacketType, payload: &[u8]) -> Result<u32> {
        let request_id = self.next_id.get();
        self.next_id.set(request_id.wrapping_add(1));
        let mut packet = Vec::with_capacity(HEADER_SIZE + payload.len());
        for word in [
            self.version.get(),
            request_id,
            packet_type as u32,
            payload.len() as u32,
        ] {
            packet.extend_from_slice(&word.to_le_bytes());
        }
        packet.extend_from_slice(payload);
        self.sock.send_to(&packet, self.addr)?;
        Ok(request_id)
    }

    fn recv(&self) -> Result<(u32, Vec<u8>)> {
        let mut buffer = self.buffer.borrow_mut();
        let got = match self.sock.recv(&mut buffer) {
            Ok(got) => got,
            Err(error) => {
                return Err(match error.kind() {
                    // Windows reports an ICMP port-unreachable for an *earlier* datagram as a
                    // reset on the next receive, even though UDP has no connection to reset. It
                    // means one packet went nowhere, not that the emulator has gone, and they
                    // arrive in bursts: counting them as silence made a long sweep give up three
                    // packets into the first burst.
                    std::io::ErrorKind::ConnectionReset => {
                        Error::new(Kind::RpcRefusedError, "a datagram was refused")
                    }
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
                        self.stalled.set(true);
                        Error::new(Kind::RpcTimeoutError, "no reply from the emulator")
                    }
                    _ => Error::from(error),
                });
            }
        };
        self.last_reply.set(time::perf_counter());
        if got < HEADER_SIZE {
            return Err(Error::new(
                Kind::RpcError,
                format!("short packet: {got} bytes"),
            ));
        }
        let word = |at: usize| u32::from_le_bytes(buffer[at..at + 4].try_into().expect("4 bytes"));
        let (request_id, size) = (word(4), word(12) as usize);
        let end = (HEADER_SIZE + size).min(got);
        Ok((request_id, buffer[HEADER_SIZE..end].to_vec()))
    }

    fn call(&self, packet_type: PacketType, payload: &[u8]) -> Result<Vec<u8>> {
        let began = time::perf_counter();
        let sent = self.send(packet_type, payload)?;
        loop {
            let (request_id, data) = self.recv()?;
            if request_id == sent {
                self.reply_seconds.set(time::perf_counter() - began);
                return Ok(data);
            }
        }
    }

    // -- processes --------------------------------------------------------------

    pub fn processes(&self) -> Result<Vec<Process>> {
        let mut data = self.call(PacketType::ProcessList, &args(0, MAX_PROCESSES)?)?;
        if data.is_empty() {
            // A list holds its count even when empty: nothing at all is a server of the other
            // version refusing the packet. Every exchange after this speaks its version.
            let other = PROTOCOL_VERSIONS
                .into_iter()
                .find(|&v| v != self.version.get())
                .expect("two versions");
            self.version.set(other);
            data = self.call(PacketType::ProcessList, &args(0, MAX_PROCESSES)?)?;
        }
        if data.len() < 4 {
            return Err(Error::new(Kind::RpcError, "process list reply too short"));
        }
        let count = u32::from_le_bytes(data[..4].try_into().expect("4 bytes")) as usize;
        let mut out = Vec::new();
        for i in 0..count {
            let at = 4 + i * PROCESS_INFO_SIZE;
            let Some(entry) = data.get(at..at + PROCESS_INFO_SIZE) else {
                return Err(Error::unpack(format!(
                    "unpack_from requires a buffer of at least {} bytes for unpacking \
                     {PROCESS_INFO_SIZE} bytes at offset {at} (actual buffer size is {})",
                    at + PROCESS_INFO_SIZE,
                    data.len()
                )));
            };
            let pid = u32::from_le_bytes(entry[..4].try_into().expect("4 bytes"));
            let title_id = u64::from_le_bytes(entry[4..12].try_into().expect("8 bytes"));
            let name = &entry[12..20];
            let name = &name[..name.iter().rposition(|&b| b != 0).map_or(0, |end| end + 1)];
            out.push(Process {
                pid: pid as i64,
                title_id: title_id as i64,
                name: crate::py::decode_latin1(name),
            });
        }
        Ok(out)
    }

    /// The pid reads are served from, or None when the emulator has not been told.
    pub fn selected_process(&self) -> Result<Option<i64>> {
        let data = self.call(PacketType::SetGetProcess, &args(0, 0)?)?;
        if data.len() < 4 {
            return Err(Error::new(Kind::RpcError, "process reply too short"));
        }
        let pid = u32::from_le_bytes(data[..4].try_into().expect("4 bytes"));
        Ok(if pid == NO_PROCESS {
            None
        } else {
            Some(pid as i64)
        })
    }

    pub fn select_process(&self, pid: i64) -> Result<()> {
        self.call(PacketType::SetGetProcess, &args(1, pid)?)?;
        self.pid.set(Some(pid));
        self.stalled.set(false);
        Ok(())
    }

    fn check(&self) -> Result<()> {
        if self.unsure() && !self.confirm()? {
            return Err(Error::new(
                Kind::RpcLostError,
                "the emulator no longer has the game selected",
            ));
        }
        Ok(())
    }
}

impl Rpc for UdpRpc {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        if !(0 < size && size <= MAX_READ) {
            return Err(Error::value(format!(
                "size must be 1..{MAX_READ}, got {size}"
            )));
        }
        self.check()?;
        self.call(PacketType::ReadMemory, &args(address, size)?)
    }

    fn read_each(&self, reads: &[(i64, i64)]) -> Result<IndexMap<i64, Vec<u8>>> {
        let mut todo: Vec<(i64, i64)> = reads.to_vec();
        let (began, count) = (time::perf_counter(), todo.len());
        for &(_address, size) in &todo {
            if !(0 < size && size <= MAX_READ) {
                return Err(Error::value(format!(
                    "size must be 1..{MAX_READ}, got {size}"
                )));
            }
        }
        let mut out = IndexMap::new();
        let mut in_flight: IndexMap<u32, (i64, i64)> = IndexMap::new();
        let (mut stalls, mut resets) = (0, 0);
        while !todo.is_empty() || !in_flight.is_empty() {
            let step = (|| -> Result<(u32, Vec<u8>)> {
                if in_flight.is_empty() {
                    self.check()?;
                }
                while !todo.is_empty() && in_flight.len() < self.window {
                    let read = todo.pop().expect("not empty");
                    let id = self.send(PacketType::ReadMemory, &args(read.0, read.1)?)?;
                    in_flight.insert(id, read);
                }
                self.recv()
            })();
            let (request_id, data) = match step {
                Ok(reply) => reply,
                Err(error) if error.kind == Kind::RpcLostError => return Err(error),
                Err(error) if error.kind == Kind::RpcRefusedError => {
                    // That one datagram is gone; its reply never comes. Say nothing and carry
                    // on — the window's other replies are still on their way, and if the whole
                    // window was lost the timeout below picks it up and resends.
                    resets += 1;
                    if resets > MAX_RESETS {
                        return Err(error);
                    }
                    continue;
                }
                Err(error) if error.is(Kind::RpcTimeoutError) => {
                    // A stall usually means the server is behind, not gone: it queues requests
                    // and drains them on its own thread, and a long sweep can outrun it.
                    // Resending at once makes that worse — the queue it could not keep up with
                    // gets longer — so back off first and give it room. Stalls are counted in a
                    // row rather than in total, because any sweep this size loses a packet
                    // eventually.
                    stalls += 1;
                    if stalls > MAX_STALLS {
                        return Err(error);
                    }
                    // The real clock: the client's own waits are inside the calls a tape answers
                    // whole, so a tape never sees them.
                    std::thread::sleep(Duration::from_secs_f64(BACKOFF * stalls as f64));
                    todo.extend(in_flight.values().copied());
                    in_flight.clear();
                    continue;
                }
                Err(error) => return Err(error),
            };
            stalls = 0;
            if let Some(read) = in_flight.shift_remove(&request_id) {
                out.insert(read.0, data);
            }
        }
        if count > 0 {
            self.reply_seconds
                .set((time::perf_counter() - began) / count as f64);
        }
        Ok(out)
    }

    fn reply_seconds(&self) -> f64 {
        self.reply_seconds.get()
    }

    fn reads_beside_gpu(&self) -> bool {
        self.version.get() >= 2
    }

    fn unsure(&self) -> bool {
        self.pid.get().is_some()
            && (self.stalled.get() || time::perf_counter() - self.last_reply.get() > QUIET)
    }

    fn confirm(&self) -> Result<bool> {
        if self.selected_process()? != self.pid.get() {
            return Ok(false);
        }
        self.stalled.set(false);
        Ok(true)
    }

    /// Reads happen to work with nothing selected, but that is undefined behaviour we do not
    /// lean on.
    fn attach(&self) -> Result<Process> {
        let running = self.processes()?;
        if running.is_empty() {
            return Err(Error::new(
                Kind::RpcError,
                "no process is running in the emulator",
            ));
        }
        if running.len() > 1 {
            let names: Vec<&str> = running.iter().map(|p| p.name.as_str()).collect();
            return Err(Error::new(
                Kind::RpcError,
                format!(
                    "expected one process, found {}: {}",
                    running.len(),
                    names.join(", ")
                ),
            ));
        }
        self.select_process(running[0].pid)?;
        Ok(running[0].clone())
    }
}
