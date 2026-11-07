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

#[cfg(windows)]
use std::path::Path;
use std::path::PathBuf;

use crate::py;

#[cfg(windows)]
pub const EMULATOR: &str = "azahar.exe";
/// Azahar off Windows: the `azahar` a build unpacks, an AppImage or a Flatpak runs, and
/// `Azahar.app/Contents/MacOS` holds. A process's name is compared in lower case.
#[cfg(not(windows))]
pub const EMULATOR: &str = "azahar";
/// What Azahar puts between its own name and the game's. The render windows carry a third part
/// ("Primary Window"), so the main window is the one with exactly two.
pub const SEPARATOR: &str = " | ";

pub const PROCESS_QUERY_LIMITED_INFORMATION: i64 = 0x1000;

use std::collections::HashMap;
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

// macOS and Linux: a process is asked of the system by its number, and the windows of the window
// server, Quartz on macOS and X on Linux.

/// The executable behind a process, in lower case, or "" when it cannot be asked: the name the
/// system keeps for it where its path cannot be read, as another user's cannot.
#[cfg(unix)]
pub fn process_name(pid: i64) -> String {
    let path = image_path(pid);
    match path.rsplit('/').next() {
        Some(name) if !name.is_empty() => name.to_lowercase(),
        _ => short_name(pid),
    }
}

/// `process_name`, kept, as `in_front` is asked on every key.
#[cfg(unix)]
fn named(pid: i64) -> String {
    NAMES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .entry(pid)
        .or_insert_with(|| process_name(pid))
        .clone()
}

/// The process last found running the emulator, asked about first: finding it otherwise reads
/// every process there is.
#[cfg(unix)]
static FOUND: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);

/// The process running the emulator, or 0 while none runs.
#[cfg(unix)]
pub fn emulator_pid(emulator: &str) -> i64 {
    use std::sync::atomic::Ordering;
    let last = FOUND.load(Ordering::Relaxed);
    if last != 0 && process_name(last) == emulator {
        return last;
    }
    let found = pids()
        .into_iter()
        .find(|&pid| process_name(pid) == emulator)
        .unwrap_or(0);
    FOUND.store(found, Ordering::Relaxed);
    found
}

/// Nothing off Windows asks which process a window is.
#[cfg(not(windows))]
pub fn process_of(_window: i64) -> String {
    String::new()
}

/// The full path of a process's executable, or "" where it cannot be asked. An AppImage's is
/// inside the image, mounted, which is why a portable Azahar is found by its working folder.
#[cfg(target_os = "linux")]
pub fn image_path(pid: i64) -> String {
    if pid <= 0 {
        return String::new();
    }
    std::fs::read_link(format!("/proc/{pid}/exe"))
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// `comm`: the name the executable was started by, cut to 15 characters, in lower case.
#[cfg(target_os = "linux")]
fn short_name(pid: i64) -> String {
    if pid <= 0 {
        return String::new();
    }
    std::fs::read_to_string(format!("/proc/{pid}/comm"))
        .map(|name| name.trim_end_matches('\n').to_lowercase())
        .unwrap_or_default()
}

#[cfg(target_os = "linux")]
fn pids() -> Vec<i64> {
    std::fs::read_dir("/proc")
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|entry| entry.file_name().to_str()?.parse().ok())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(target_os = "linux")]
fn working_folder(pid: i64) -> Option<PathBuf> {
    std::fs::read_link(format!("/proc/{pid}/cwd")).ok()
}

#[cfg(target_os = "macos")]
pub fn image_path(pid: i64) -> String {
    let Ok(pid @ 1..) = i32::try_from(pid) else {
        return String::new();
    };
    let mut buffer = vec![0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
    // SAFETY: `buffer` holds as many bytes as it is said to; a process that cannot be asked
    // writes nothing and gives 0 or less.
    let length =
        unsafe { libc::proc_pidpath(pid, buffer.as_mut_ptr().cast(), buffer.len() as u32) };
    if 0 >= length {
        return String::new();
    }
    String::from_utf8_lossy(&buffer[..length as usize]).into_owned()
}

/// `proc_name`: the process's name as the system keeps it, in lower case.
#[cfg(target_os = "macos")]
fn short_name(pid: i64) -> String {
    let Ok(pid @ 1..) = i32::try_from(pid) else {
        return String::new();
    };
    let mut buffer = [0u8; 64];
    // SAFETY: as in `image_path`.
    let length = unsafe { libc::proc_name(pid, buffer.as_mut_ptr().cast(), buffer.len() as u32) };
    if 0 >= length {
        return String::new();
    }
    String::from_utf8_lossy(&buffer[..length as usize]).to_lowercase()
}

#[cfg(target_os = "macos")]
fn pids() -> Vec<i64> {
    // SAFETY: with no buffer, proc_listallpids says how many processes there are.
    let count = unsafe { libc::proc_listallpids(std::ptr::null_mut(), 0) };
    if count <= 0 {
        return Vec::new();
    }
    // Room for those started since.
    let mut pids: Vec<libc::pid_t> = vec![0; count as usize + 64];
    let bytes = (pids.len() * std::mem::size_of::<libc::pid_t>()) as i32;
    // SAFETY: `pids` holds `bytes` bytes, which is what is passed.
    let found = unsafe { libc::proc_listallpids(pids.as_mut_ptr().cast(), bytes) };
    pids.truncate(found.max(0) as usize);
    pids.into_iter().map(i64::from).collect()
}

#[cfg(target_os = "macos")]
fn working_folder(pid: i64) -> Option<PathBuf> {
    let pid = i32::try_from(pid).ok()?;
    // SAFETY: proc_vnodepathinfo is plain integers and bytes, for which zero is a value.
    let mut info: libc::proc_vnodepathinfo = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<libc::proc_vnodepathinfo>() as i32;
    // SAFETY: `info` is a live proc_vnodepathinfo of `size` bytes for the call to fill.
    let filled = unsafe {
        libc::proc_pidinfo(
            pid,
            libc::PROC_PIDVNODEPATHINFO,
            0,
            (&raw mut info).cast(),
            size,
        )
    };
    if filled != size {
        return None;
    }
    let path: Vec<u8> = info
        .pvi_cdir
        .vip_path
        .as_flattened()
        .iter()
        .take_while(|&&byte| byte != 0)
        .map(|&byte| byte as u8)
        .collect();
    Some(PathBuf::from(String::from_utf8_lossy(&path).into_owned()))
}

/// Whether the window in front is one of the emulator's showing the game (ADR-0021, ADR-0022),
/// as on Windows. The front window is the first ordinary one, at layer 0, in Quartz's list,
/// which runs front to back. Its title is there only for a program the player has given Screen
/// Recording: without it, a window of Azahar's in front is taken as showing the game, dialogs
/// too.
#[cfg(target_os = "macos")]
pub fn in_front(emulator: &str) -> bool {
    let Some((pid, title)) = quartz::windows(true).into_iter().next() else {
        return false;
    };
    named(pid) == emulator && (title.is_empty() || shows_game(&title))
}

/// Every window of the emulator's at layer 0, by its title, shown or not: "" for each where
/// macOS keeps titles back (`in_front`), so that Azahar is still known to be open.
#[cfg(target_os = "macos")]
pub fn titles(emulator: &str) -> Vec<String> {
    quartz::windows(false)
        .into_iter()
        .filter(|(pid, _)| named(*pid) == emulator)
        .map(|(_, title)| title)
        .collect()
}

/// Quartz's window list, through CoreGraphics and CoreFoundation's plain C.
#[cfg(target_os = "macos")]
mod quartz {
    use std::ffi::{CStr, c_char, c_void};

    type Object = *const c_void;

    const ON_SCREEN_ONLY: u32 = 1 << 0;
    const EXCLUDE_DESKTOP_ELEMENTS: u32 = 1 << 4;
    const NULL_WINDOW: u32 = 0;
    const SINT64: isize = 4;
    const UTF8: u32 = 0x0800_0100;

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFArrayGetCount(array: Object) -> isize;
        fn CFArrayGetValueAtIndex(array: Object, index: isize) -> Object;
        fn CFDictionaryGetValue(dictionary: Object, key: Object) -> Object;
        fn CFNumberGetValue(number: Object, kind: isize, value: *mut c_void) -> u8;
        fn CFStringGetLength(string: Object) -> isize;
        fn CFStringGetMaximumSizeForEncoding(length: isize, encoding: u32) -> isize;
        fn CFStringGetCString(
            string: Object,
            buffer: *mut c_char,
            size: isize,
            encoding: u32,
        ) -> u8;
        fn CFRelease(object: Object);
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGWindowListCopyWindowInfo(option: u32, relative_to: u32) -> Object;
        static kCGWindowLayer: Object;
        static kCGWindowOwnerPID: Object;
        static kCGWindowName: Object;
    }

    /// The owner and title of each window at layer 0, front to back: only those on screen, or
    /// every one there is.
    pub fn windows(on_screen: bool) -> Vec<(i64, String)> {
        let option = if on_screen {
            ON_SCREEN_ONLY | EXCLUDE_DESKTOP_ELEMENTS
        } else {
            EXCLUDE_DESKTOP_ELEMENTS
        };
        // SAFETY: the call takes plain values and returns an array this function owns, or null.
        let list = unsafe { CGWindowListCopyWindowInfo(option, NULL_WINDOW) };
        if list.is_null() {
            return Vec::new();
        }
        let mut found = Vec::new();
        // SAFETY: `list` is the array of dictionaries CGWindowListCopyWindowInfo returns, alive
        // until it is released below; the keys are CoreGraphics' own constants.
        unsafe {
            for index in 0..CFArrayGetCount(list) {
                let window = CFArrayGetValueAtIndex(list, index);
                if number(window, kCGWindowLayer) != Some(0) {
                    continue;
                }
                if let Some(pid) = number(window, kCGWindowOwnerPID) {
                    found.push((pid, text(window, kCGWindowName)));
                }
            }
            CFRelease(list);
        }
        found
    }

    /// A number in a window's dictionary.
    ///
    /// SAFETY: `window` is a live CFDictionary and `key` a CFString.
    unsafe fn number(window: Object, key: Object) -> Option<i64> {
        let value = unsafe { CFDictionaryGetValue(window, key) };
        let mut out: i64 = 0;
        if value.is_null() || unsafe { CFNumberGetValue(value, SINT64, (&raw mut out).cast()) } == 0
        {
            return None;
        }
        Some(out)
    }

    /// A string in a window's dictionary, or "" where it has none.
    ///
    /// SAFETY: as for `number`.
    unsafe fn text(window: Object, key: Object) -> String {
        let value = unsafe { CFDictionaryGetValue(window, key) };
        if value.is_null() {
            return String::new();
        }
        let size = unsafe { CFStringGetMaximumSizeForEncoding(CFStringGetLength(value), UTF8) } + 1;
        let mut buffer = vec![0 as c_char; size.max(1) as usize];
        if unsafe { CFStringGetCString(value, buffer.as_mut_ptr(), size, UTF8) } == 0 {
            return String::new();
        }
        // SAFETY: CFStringGetCString wrote a terminated string into `buffer`.
        unsafe { CStr::from_ptr(buffer.as_ptr()) }
            .to_string_lossy()
            .into_owned()
    }
}

/// Whether the window in front is one of the emulator's showing the game (ADR-0021, ADR-0022),
/// as on Windows, through X's `_NET_ACTIVE_WINDOW`. An Azahar with no X window, as a Wayland
/// one has, is never the active X window, and pa3ds cannot tell whether it is in front: it is
/// taken to be while it runs. With no X display at all, it always is, so that speech is never
/// lost to a question pa3ds cannot ask.
#[cfg(target_os = "linux")]
pub fn in_front(emulator: &str) -> bool {
    x11::with(|x| {
        let active = x.windows(x.root, x.active)?.first().copied().unwrap_or(0);
        if active != 0 && x.is_emulator(active, emulator) {
            return Ok(shows_game(&x.title(active)));
        }
        let clients = x.windows(x.root, x.clients)?;
        let on_x = clients
            .into_iter()
            .any(|window| x.is_emulator(window, emulator));
        Ok(!on_x && emulator_pid(emulator) != 0)
    })
    .unwrap_or(true)
}

/// Every titled X window of the emulator's: `_NET_CLIENT_LIST`, the windows the window manager
/// manages, by `_NET_WM_NAME`. An Azahar running with none, as a Wayland one does, is one ""
/// so that it is still known to be open.
#[cfg(target_os = "linux")]
pub fn titles(emulator: &str) -> Vec<String> {
    let found = x11::with(|x| {
        Ok(x.windows(x.root, x.clients)?
            .into_iter()
            .filter(|&window| x.is_emulator(window, emulator))
            .map(|window| x.title(window))
            .filter(|title| !title.is_empty())
            .collect::<Vec<String>>())
    })
    .unwrap_or_default();
    if found.is_empty() && emulator_pid(emulator) != 0 {
        return vec![String::new()];
    }
    found
}

/// The X display, through x11rb, connected once and kept.
#[cfg(target_os = "linux")]
mod x11 {
    use std::sync::{Mutex, PoisonError};

    use x11rb::connection::Connection;
    use x11rb::errors::ReplyError;
    use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt, GetPropertyReply, Window};
    use x11rb::rust_connection::RustConnection;

    pub struct Display {
        connection: RustConnection,
        pub root: Window,
        pub active: Atom,
        pub clients: Atom,
        pid: Atom,
        name: Atom,
        utf8: Atom,
    }

    static DISPLAY: Mutex<Option<Display>> = Mutex::new(None);

    /// An answer from the display, or None where there is no display. A display that has
    /// gone is let go of, and connected to again at the next question.
    pub fn with<T>(ask: impl FnOnce(&Display) -> Result<T, ReplyError>) -> Option<T> {
        let mut held = DISPLAY.lock().unwrap_or_else(PoisonError::into_inner);
        if held.is_none() {
            *held = Display::open();
        }
        let asked = ask(held.as_ref()?);
        if let Err(ReplyError::ConnectionError(_)) = asked {
            *held = None;
        }
        asked.ok()
    }

    impl Display {
        fn open() -> Option<Display> {
            let (connection, screen) = x11rb::connect(None).ok()?;
            let root = connection.setup().roots.get(screen)?.root;
            let atom = |name: &[u8]| -> Option<Atom> {
                Some(connection.intern_atom(false, name).ok()?.reply().ok()?.atom)
            };
            let (active, clients) = (atom(b"_NET_ACTIVE_WINDOW")?, atom(b"_NET_CLIENT_LIST")?);
            let (pid, name, utf8) = (
                atom(b"_NET_WM_PID")?,
                atom(b"_NET_WM_NAME")?,
                atom(b"UTF8_STRING")?,
            );
            Some(Display {
                connection,
                root,
                active,
                clients,
                pid,
                name,
                utf8,
            })
        }

        fn property(
            &self,
            window: Window,
            property: impl Into<Atom>,
            kind: impl Into<Atom>,
        ) -> Result<GetPropertyReply, ReplyError> {
            self.connection
                .get_property(false, window, property, kind, 0, 1024)?
                .reply()
        }

        /// A property of a window's that lists windows.
        pub fn windows(&self, window: Window, property: Atom) -> Result<Vec<Window>, ReplyError> {
            let reply = self.property(window, property, AtomEnum::WINDOW)?;
            Ok(reply
                .value32()
                .map(|windows| windows.collect())
                .unwrap_or_default())
        }

        /// A window's title, or "" where it has none or has gone.
        pub fn title(&self, window: Window) -> String {
            self.property(window, self.name, self.utf8)
                .map(|reply| String::from_utf8_lossy(&reply.value).into_owned())
                .unwrap_or_default()
        }

        /// Whether a window is the emulator's: by the process `_NET_WM_PID` names, or by its
        /// `WM_CLASS`, for a Flatpak's, whose process number is its sandbox's own.
        pub fn is_emulator(&self, window: Window, emulator: &str) -> bool {
            let pid = self
                .property(window, self.pid, AtomEnum::CARDINAL)
                .ok()
                .and_then(|reply| reply.value32()?.next());
            if pid.is_some_and(|pid| super::named(i64::from(pid)) == emulator) {
                return true;
            }
            self.property(window, AtomEnum::WM_CLASS, AtomEnum::STRING)
                .is_ok_and(|reply| {
                    reply
                        .value
                        .split(|&byte| byte == 0)
                        .any(|part| part.eq_ignore_ascii_case(emulator.as_bytes()))
                })
        }
    }
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
#[cfg(windows)]
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

/// The folder the running emulator was started in, or None while none runs: where Azahar on
/// macOS and Linux looks for the `user` folder of a portable one (`keyfile.found_portable`).
#[cfg(not(windows))]
pub fn emulator_folder(emulator: &str) -> Option<PathBuf> {
    match emulator_pid(emulator) {
        0 => None,
        pid => working_folder(pid),
    }
}
