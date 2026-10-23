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
//! Bound by:
//! - ADR-0021: "in front" is a window showing the game, known by its title.
//! - ADR-0031: the game's name is the emulator's window title: pa3ds keeps no table of game names.

use std::path::{Path, PathBuf};

use crate::py;

pub const EMULATOR: &str = "azahar.exe";
/// What Azahar puts between its own name and the game's. The render windows carry a third part
/// ("Primary Window"), so the main window is the one with exactly two.
pub const SEPARATOR: &str = " | ";

pub const PROCESS_QUERY_LIMITED_INFORMATION: i64 = 0x1000;

#[cfg(windows)]
use std::collections::HashMap;
#[cfg(windows)]
use std::sync::{LazyLock, Mutex, PoisonError};

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HWND, LPARAM};
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

/// The executable behind a window, in lower case, or "" when it cannot be asked.
#[cfg(windows)]
pub fn process_of(window: i64) -> String {
    let mut pid: u32 = 0;
    // SAFETY: `pid` is a live u32 for the call to write the process id into; an invalid window
    // leaves it 0.
    unsafe { GetWindowThreadProcessId(window as usize as HWND, &mut pid) };
    process_name(pid as i64)
}

/// Whether the window in front is one of the emulator's showing the game: its main
/// window or a render window, and not one of its dialogs. What the keyboard hook takes a
/// key by (ADR-0022), and what speech keeps quiet by while the player is reading something
/// else (ADR-0021).
#[cfg(windows)]
pub fn in_front(emulator: &str) -> bool {
    // SAFETY: GetForegroundWindow takes no arguments.
    let front = unsafe { GetForegroundWindow() };
    if front.is_null() {
        return false;
    }
    let mut pid: u32 = 0;
    // SAFETY: as in `process_of`.
    unsafe { GetWindowThreadProcessId(front, &mut pid) };
    let name = NAMES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .entry(pid as i64)
        .or_insert_with(|| process_name(pid as i64))
        .clone();
    name == emulator && shows_game(&text(front as usize as i64))
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
    let Some(&window) = visible(emulator).first() else {
        return 0;
    };
    let mut pid: u32 = 0;
    // SAFETY: `pid` is a live u32 for the call to write the process id into.
    unsafe { GetWindowThreadProcessId(window as usize as HWND, &mut pid) };
    pid as i64
}

#[cfg(windows)]
fn visible(emulator: &str) -> Vec<i64> {
    struct Each<'a> {
        emulator: &'a str,
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
        {
            this.found.push(window);
        }
        1
    }

    let mut every = Each {
        emulator,
        found: Vec::new(),
    };
    // SAFETY: `each` matches the callback signature, and `every` outlives the call.
    unsafe { EnumWindows(Some(each), &mut every as *mut Each as LPARAM) };
    every.found
}

// The runtime is Windows; this keeps the import working elsewhere.

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
/// Azahar's settings until 2026-10-01 (Orion).
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
/// portable Azahar keeps its settings (`keyfile.found_portable`).
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
