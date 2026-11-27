//! The emulator's windows: which one is in front, and what they say is running.
//!
//! The RPC answers ReadMemory, WriteMemory, ProcessList and SetGetProcess, and nothing among them is
//! "what is this game called". A process is `sango-1` and a title id, which is what the console calls
//! the ROM and says nothing to a person. The game's real name — Pokémon Omega Ruby — lives in the
//! ROM's SMDH, which the game itself never maps into memory.
//!
//! **Azahar has already read it.** Its window is titled `Azahar <build> | <game>`, where the game
//! comes from `loader->ReadTitle()`, straight out of that SMDH (`citra_qt/citra_qt.cpp`,
//! `UpdateWindowTitle`). So the name is there for the asking, and asking for it keeps pa3ds free of
//! any table of game names: **the game names itself**, which is the rule the whole project is built
//! on, and this is the one place a game's own title is wanted at all — the comment at the top of a
//! player's names file (`pa3ds.core.names`).
//!
//! Nothing here is ever spoken and nothing depends on it. A title that cannot be found is "", and
//! the ROM names itself instead.
//!
//! Asking **which process a window belongs to** lives here too, because two things want it: this, to
//! find Azahar's window among everybody's, and `pa3ds.core.keys`, to answer a key only while Azahar's
//! window is in front. It was written twice before it was written once.
//!
//! **Which emulator is pa3ds's** where two are open, one per player of the PSS on one machine: the
//! one listening on the RPC port pa3ds reads (`follow`, `own`). Its windows are the ones in front,
//! named and closed with; the other's are somebody else's, as any program's are. Until one listens
//! there, before a game is loaded, every Azahar's windows are.
//!
//! Bound by:
//! - ADR-0021: "in front" is a window showing the game, known by its title.
//! - ADR-0031: the game's name is the emulator's window title: pa3ds keeps no table of game names.
//! - ADR-0039: with two emulators open, pa3ds's is the one on its RPC port.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use crate::py;

pub const EMULATOR: &str = "azahar.exe";
/// What Azahar puts between its own name and the game's. The render windows carry a third part
/// ("Primary Window"), so the main window is the one with exactly two.
pub const SEPARATOR: &str = " | ";

pub const PROCESS_QUERY_LIMITED_INFORMATION: i64 = 0x1000;
pub const UDP_TABLE_OWNER_PID: i32 = 1;
/// Seconds an answer of `own` stands: it is asked on every key and every line said.
pub const LOOK: f64 = 1.0;

/// The RPC port pa3ds reads (`follow`), 0 for none; and the process `own` found, and when it
/// looked. One lock for the two, which the Python keeps as module globals.
static FOLLOWED: Mutex<(u16, (i64, f64))> = Mutex::new((0, (0, -LOOK)));

/// Take the emulator listening on this RPC port for pa3ds's own (`own`).
pub fn follow(port: u16) {
    *FOLLOWED.lock().unwrap_or_else(PoisonError::into_inner) = (port, (0, -LOOK));
}

/// The process listening on the RPC port pa3ds reads, or 0 while none does or none is
/// followed: with two emulators open, the one that is pa3ds's.
pub fn own() -> i64 {
    let mut followed = FOLLOWED.lock().unwrap_or_else(PoisonError::into_inner);
    let (port, (mut pid, looked)) = *followed;
    if port != 0 && py::time::monotonic() - looked >= LOOK {
        pid = listening(port);
        followed.1 = (pid, py::time::monotonic());
    }
    pid
}

#[cfg(windows)]
use std::collections::HashMap;
#[cfg(windows)]
use std::sync::LazyLock;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HWND, LPARAM};
#[cfg(windows)]
use windows_sys::Win32::NetworkManagement::IpHelper::GetExtendedUdpTable;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsWindowVisible,
};
#[cfg(windows)]
use windows_sys::core::BOOL;

/// Process id -> executable name, since `in_front` is asked on every key and every line said.
#[cfg(windows)]
static NAMES: LazyLock<Mutex<HashMap<i64, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// The process with a UDP socket on this port, which Azahar's RPC server is, or 0.
#[cfg(windows)]
pub fn listening(port: u16) -> i64 {
    // socket.AF_INET.
    let af_inet: u32 = 2;
    let mut size: u32 = 0;
    // SAFETY: a null table asks only for the size, written into `size`.
    unsafe {
        GetExtendedUdpTable(
            std::ptr::null_mut(),
            &mut size,
            0,
            af_inet,
            UDP_TABLE_OWNER_PID,
            0,
        )
    };
    // With room for sockets opened between the two asks.
    size += 12 * 16;
    // Words rather than bytes, for the table's alignment.
    let mut table = vec![0u32; (size as usize).div_ceil(4)];
    // SAFETY: `table` holds at least `size` bytes, which is what `size` tells the call.
    if unsafe {
        GetExtendedUdpTable(
            table.as_mut_ptr().cast(),
            &mut size,
            0,
            af_inet,
            UDP_TABLE_OWNER_PID,
            0,
        )
    } != 0
    {
        return 0;
    }
    // MIB_UDPTABLE_OWNER_PID: a count, then rows of address, port and process, the port in
    // network order in its first two bytes.
    for row in 0..table[0] as usize {
        let (_address, at, pid) = (table[1 + 3 * row], table[2 + 3 * row], table[3 + 3 * row]);
        if u16::from_be((at & 0xFFFF) as u16) == port {
            return i64::from(pid);
        }
    }
    0
}

#[cfg(windows)]
fn pid(window: i64) -> i64 {
    let mut pid: u32 = 0;
    // SAFETY: `pid` is a live u32 for the call to write the process id into; an invalid window
    // leaves it 0.
    unsafe { GetWindowThreadProcessId(window as usize as HWND, &mut pid) };
    i64::from(pid)
}

/// The executable behind a window, in lower case, or "" when it cannot be asked.
#[cfg(windows)]
pub fn process_of(window: i64) -> String {
    process_name(pid(window))
}

/// Whether the window in front is one of the emulator's showing the game, pa3ds's own
/// emulator's where it is known (`own`): its main window or a render window, and not one of
/// its dialogs. What the keyboard hook takes a
/// key by (ADR-0022), and what speech keeps quiet by while the player is reading something
/// else (ADR-0021).
#[cfg(windows)]
pub fn in_front(emulator: &str) -> bool {
    // SAFETY: GetForegroundWindow takes no arguments.
    let front = unsafe { GetForegroundWindow() };
    if front.is_null() {
        return false;
    }
    let pid = pid(front as usize as i64);
    let name = NAMES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .entry(pid)
        .or_insert_with(|| process_name(pid))
        .clone();
    let mine = own();
    name == emulator && (mine == 0 || pid == mine) && shows_game(&text(front as usize as i64))
}

#[cfg(windows)]
pub fn process_name(pid: i64) -> String {
    basename(&image_path(pid)).to_lowercase()
}

/// The full path of a process's executable, or "" where it cannot be asked.
#[cfg(windows)]
pub fn image_path(pid: i64) -> String {
    // SAFETY: OpenProcess only reads its arguments; a null handle is a refusal.
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION as u32, 0, pid as u32) };
    if handle.is_null() {
        return String::new();
    }
    let mut size: u32 = 1024;
    let mut buffer = vec![0u16; size as usize];
    // SAFETY: `handle` is open, `buffer` holds `size` UTF-16 units and `size` is where the call
    // reads how many it has room for and writes how many it wrote.
    let queried =
        unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut size) } != 0;
    let name = if queried {
        let end = buffer
            .iter()
            .position(|&unit| unit == 0)
            .unwrap_or(buffer.len());
        Some(String::from_utf16_lossy(&buffer[..end]))
    } else {
        None
    };
    // SAFETY: `handle` was opened above and is closed once.
    unsafe { CloseHandle(handle) };
    name.unwrap_or_default()
}

/// `os.path.basename` as `ntpath` has it: what follows the last separator, after a drive letter.
#[cfg(windows)]
fn basename(path: &str) -> &str {
    let mut chars = path.chars();
    let rest = if chars.nth(1) == Some(':') {
        chars.as_str()
    } else {
        path
    };
    rest.rsplit(['\\', '/']).next().unwrap_or("")
}

#[cfg(windows)]
fn text(window: i64) -> String {
    // SAFETY: GetWindowTextLengthW only reads the window handle, valid or not.
    let length = unsafe { GetWindowTextLengthW(window as usize as HWND) };
    if length <= 0 {
        return String::new();
    }
    let mut buffer = vec![0u16; length as usize + 1];
    // SAFETY: `buffer` has room for `length + 1` UTF-16 units, which is what is passed.
    unsafe { GetWindowTextW(window as usize as HWND, buffer.as_mut_ptr(), length + 1) };
    let end = buffer
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

/// Every visible top-level window of the emulator, by its title.
#[cfg(windows)]
pub fn titles(emulator: &str) -> Vec<String> {
    visible(emulator).into_iter().map(text).collect()
}

/// The process behind the emulator's visible windows, or 0 while it has none up.
#[cfg(windows)]
pub fn emulator_pid(emulator: &str) -> i64 {
    visible(emulator).first().map_or(0, |&window| pid(window))
}

/// The emulator's visible windows, pa3ds's own (`own`) where it is known.
#[cfg(windows)]
fn visible(emulator: &str) -> Vec<i64> {
    struct Each<'a> {
        emulator: &'a str,
        mine: i64,
        found: Vec<i64>,
    }

    unsafe extern "system" fn each(window: HWND, lparam: LPARAM) -> BOOL {
        // SAFETY: `lparam` is the `Each` that `visible` passed to EnumWindows, which stays alive
        // and is used by nothing else until EnumWindows returns.
        let this = unsafe { &mut *(lparam as *mut Each) };
        let window = window as usize as i64;
        // SAFETY: IsWindowVisible only reads the window handle.
        if unsafe { IsWindowVisible(window as usize as HWND) } != 0
            && !text(window).is_empty()
            && process_of(window) == this.emulator
            && (this.mine == 0 || pid(window) == this.mine)
        {
            this.found.push(window);
        }
        1
    }

    let mut every = Each {
        emulator,
        mine: own(),
        found: Vec::new(),
    };
    // SAFETY: `each` matches the callback signature, and `every` outlives the call.
    unsafe { EnumWindows(Some(each), &mut every as *mut Each as LPARAM) };
    every.found
}

// The runtime is Windows; this keeps the import working elsewhere.

#[cfg(not(windows))]
pub fn listening(_port: u16) -> i64 {
    0
}

#[cfg(not(windows))]
pub fn process_name(_pid: i64) -> String {
    String::new()
}

#[cfg(not(windows))]
pub fn image_path(_pid: i64) -> String {
    String::new()
}

#[cfg(not(windows))]
pub fn process_of(_window: i64) -> String {
    String::new()
}

#[cfg(not(windows))]
pub fn in_front(_emulator: &str) -> bool {
    true
}

#[cfg(not(windows))]
pub fn titles(_emulator: &str) -> Vec<String> {
    Vec::new()
}

#[cfg(not(windows))]
pub fn emulator_pid(_emulator: &str) -> i64 {
    0
}

/// Whether a window of the emulator's, by its title, is one the game is played in.
///
/// Azahar titles its main window and both render windows with the game's name after a
/// separator; its dialogs are "Configure", "Cheats" and the like. A key typed into one of those
/// is the player's, for a text field or a list's first letter: pa3ds took I and O out of
/// Azahar's settings until 2026-10-01.
pub fn shows_game(title: &str) -> bool {
    title.contains(SEPARATOR)
}

/// The game's name out of one window title, or "" when that title is not one.
///
/// `Azahar 2116 | Pokémon Omega Ruby` gives the game; `Azahar 2116` alone is the emulator with
/// no game loaded, and `Azahar 2116 | Pokémon Omega Ruby | Primary Window` is a render window,
/// whose name is the same but which is not worth preferring over the main one.
pub fn game_of(title: &str) -> String {
    let parts: Vec<&str> = title.split(SEPARATOR).collect();
    if parts.len() == 2 {
        py::strip(parts[1]).to_string()
    } else {
        String::new()
    }
}

/// What the emulator says it is running, or "" when it says nothing useful.
///
/// Read when a names file is about to be written, which is while the player is playing, so the
/// window is up and a game is in it.
pub fn game_title(emulator: &str) -> String {
    game_title_from(emulator, |emulator| Ok(titles(emulator)))
}

/// `game_title` with the window list as a parameter: a test monkeypatches `titles`, which Rust
/// cannot, so the one place `game_title` asks for it is a seam.
pub(crate) fn game_title_from(
    emulator: &str,
    titles: impl FnOnce(&str) -> std::io::Result<Vec<String>>,
) -> String {
    match titles(emulator) {
        Ok(found) => found
            .iter()
            .map(|title| game_of(title))
            .find(|game| !game.is_empty())
            .unwrap_or_default(),
        Err(_) => String::new(),
    }
}

/// The folder of the running emulator's executable, or None while none is up: where a
/// portable Azahar keeps its settings (`azahar.found_portable`).
pub fn emulator_folder(emulator: &str) -> Option<PathBuf> {
    let pid = emulator_pid(emulator);
    let path = if pid != 0 {
        image_path(pid)
    } else {
        String::new()
    };
    if path.is_empty() {
        return None;
    }
    // `Path(path).parent`, which is the path itself at a root.
    let path = PathBuf::from(path);
    Some(path.parent().map(Path::to_path_buf).unwrap_or(path))
}
