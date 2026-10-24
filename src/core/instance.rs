//! One pa3ds at a time, and out of sight while Azahar is open (ADR-0039).
//!
//! **Its window.** pa3ds runs in a console, which a player who double-clicked `pa3ds.exe` has no
//! use for once Azahar is open: it sits in Alt+Tab between them and the game. So while Azahar has a
//! window up, pa3ds hides the window its console is shown in, and once Azahar has been open and
//! closes, pa3ds closes too (`Beside`). Unloading a game leaves Azahar's window up, so switching
//! games, loading a state or the game restarting itself never closes pa3ds. Under Windows Terminal,
//! every Windows 11 player's console, the console's own window is a stand-in that `ShowWindow`
//! ignores; the terminal's window owns it, and is the one hidden (measured 2026-10-17). A console a
//! shell shares is never hidden: the shell, and every tab beside it, would go with it, and a player
//! who opened a shell to start pa3ds wants to see it (Orion).
//!
//! **The one before it.** A pa3ds started while another runs ends that one, which may be hung,
//! and takes its place (`replace`): the log carries on in the same file, and the first "pa3ds
//! ready" is "pa3ds restarted". A small file in the player's folder names the pa3ds running by its
//! process and the moment that process started, so a file left by a pa3ds that died never ends a
//! program that has its number since. It is deleted as pa3ds closes, and not when pa3ds is
//! replaced.
//!
//! Bound by:
//! - ADR-0039: one pa3ds at a time, hidden while Azahar is open and closed with it; a `--silent`
//!   run takes no part.

use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;

use crate::core::{toml, window};
use crate::py::{Log, Result, time};

/// The file in the player's folder naming the pa3ds running.
pub const LOCK: &str = "pa3ds.lock";
/// What pa3ds runs as: the Rust port's one executable, or Python under uv. A console with anything
/// else attached to it is a shell's.
pub const OURS: [&str; 3] = ["pa3ds.exe", "python.exe", "uv.exe"];
/// Seconds between looks at Azahar.
pub const LOOK: f64 = 1.0;
/// Milliseconds to wait for the pa3ds replaced to be gone, before taking its place anyway.
pub const PATIENCE: u32 = 5000;

pub const SYNCHRONIZE: u32 = 0x00100000;
pub const PROCESS_TERMINATE: u32 = 0x0001;
pub const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
pub const GA_ROOTOWNER: u32 = 3;
pub const SW_HIDE: i32 = 0;
pub const WAIT_OBJECT_0: u32 = 0;

#[cfg(windows)]
mod win {
    use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, HWND};
    use windows_sys::Win32::System::Console::{GetConsoleProcessList, GetConsoleWindow};
    use windows_sys::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, TerminateProcess, WaitForSingleObject,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetAncestor, IsWindowVisible, ShowWindow};

    use super::*;

    /// A handle on a process, or 0 when there is no such process to open.
    pub fn opened(pid: i64, access: u32) -> i64 {
        // SAFETY: OpenProcess takes plain values and returns a handle or null.
        unsafe { OpenProcess(access, 0, pid as u32) as usize as i64 }
    }

    /// Whether the process a handle is on is still running.
    pub fn alive(handle: i64) -> bool {
        // SAFETY: a zero timeout only asks whether the process has ended.
        unsafe { WaitForSingleObject(handle as usize as HANDLE, 0) != WAIT_OBJECT_0 }
    }

    fn started_of(handle: i64) -> i64 {
        let empty = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut times = [empty; 4];
        let [created, exited, kernel, user] = &mut times;
        // SAFETY: each pointer is a live FILETIME for the call to fill.
        let ok =
            unsafe { GetProcessTimes(handle as usize as HANDLE, created, exited, kernel, user) };
        if ok == 0 {
            return 0;
        }
        (i64::from(times[0].dwHighDateTime) << 32) | i64::from(times[0].dwLowDateTime)
    }

    fn close(handle: i64) {
        // SAFETY: the handle is one this module opened and closes once.
        unsafe { CloseHandle(handle as usize as HANDLE) };
    }

    /// When a process started, as Windows counts time, or 0 for no such process.
    pub fn started(pid: i64) -> i64 {
        let handle = opened(pid, PROCESS_QUERY_LIMITED_INFORMATION);
        if handle == 0 {
            return 0;
        }
        let began = started_of(handle);
        close(handle);
        began
    }

    /// End a process if it is still the one that started at `began`, and wait for it to be
    /// gone. With exit code 0: Windows Terminal keeps a tab open for a program that ends with
    /// any other, which for a pa3ds that had hidden its window would be a window nobody sees.
    pub fn end(pid: i64, began: i64) -> bool {
        let handle = opened(
            pid,
            SYNCHRONIZE | PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION,
        );
        if handle == 0 {
            return false;
        }
        let ended = started_of(handle) == began;
        if ended {
            // SAFETY: the handle was opened with the rights both calls need.
            unsafe {
                TerminateProcess(handle as usize as HANDLE, 0);
                WaitForSingleObject(handle as usize as HANDLE, PATIENCE);
            }
        }
        close(handle);
        ended
    }

    /// Whether pa3ds has its console to itself: nothing attached to it but pa3ds.
    pub fn alone() -> bool {
        let mut pids = [0u32; 64];
        // SAFETY: the buffer holds as many ids as it is said to.
        let count = unsafe { GetConsoleProcessList(pids.as_mut_ptr(), pids.len() as u32) } as usize;
        0 < count
            && count <= pids.len()
            && pids[..count]
                .iter()
                .all(|&pid| OURS.contains(&window::process_name(i64::from(pid)).as_str()))
    }

    /// Hide the window pa3ds's console is shown in, the terminal's where a terminal shows it,
    /// unless a shell shares the console. Asked again at every look while Azahar is open, since
    /// Windows Terminal shows and owns its window a moment after pa3ds has started.
    pub fn hide() {
        // SAFETY: GetConsoleWindow takes nothing and returns a window or null.
        let console: HWND = unsafe { GetConsoleWindow() };
        if console.is_null() || !alone() {
            return;
        }
        // SAFETY: each call only reads or shows the window handle it is given.
        unsafe {
            let owner = GetAncestor(console, GA_ROOTOWNER);
            let shown = if owner.is_null() { console } else { owner };
            if IsWindowVisible(shown) != 0 {
                ShowWindow(shown, SW_HIDE);
            }
        }
    }
}

#[cfg(windows)]
pub use win::{alive, alone, end, hide, opened, started};

// The runtime is Windows; this keeps the import working elsewhere.

#[cfg(not(windows))]
pub fn opened(_pid: i64, _access: u32) -> i64 {
    0
}

#[cfg(not(windows))]
pub fn alive(_handle: i64) -> bool {
    false
}

#[cfg(not(windows))]
pub fn started(_pid: i64) -> i64 {
    0
}

#[cfg(not(windows))]
pub fn end(_pid: i64, _began: i64) -> bool {
    false
}

#[cfg(not(windows))]
pub fn alone() -> bool {
    false
}

#[cfg(not(windows))]
pub fn hide() {}

/// Take the place of the pa3ds running from `home`, ending it, and say whether one was.
pub fn replace(home: &Path) -> Result<bool> {
    let path = home.join(LOCK);
    let held = toml::read(&path);
    let own = i64::from(std::process::id());
    let replaced = match (held.get("pid"), held.get("started")) {
        (Some(::toml::Value::Integer(pid)), Some(::toml::Value::Integer(began))) => {
            *pid != own && end(*pid, *began)
        }
        _ => false,
    };
    toml::write(
        &path,
        &format!(
            "# The pa3ds running now, for one started after it to replace. Deleted as it \
             closes.\npid = {own}\nstarted = {}\n",
            started(own)
        ),
    )?;
    Ok(replaced)
}

/// Delete the file naming the pa3ds running, while it still names this one.
pub fn release(home: &Path) {
    let path = home.join(LOCK);
    let own = i64::from(std::process::id());
    if toml::read(&path).get("pid") == Some(&::toml::Value::Integer(own)) {
        let _ = std::fs::remove_file(&path);
    }
}

/// Azahar, looked at once a second: pa3ds's console is hidden while Azahar is open, or from
/// the start where pa3ds's own window stands in for it, and that window while Azahar is open;
/// `closed` is true once Azahar has been open and has closed, for pa3ds to close with it.
pub struct Beside {
    log: Log,
    look: f64,
    next: Cell<f64>,
    /// A handle on Azahar's process, once it has been seen: whether it is running is then
    /// asked of the process, which a window coming and going cannot fool.
    azahar: Cell<i64>,
    /// `window.emulator_pid` and `hide`, which the Python's test monkeypatched.
    find: Rc<dyn Fn() -> i64>,
    hide: Rc<dyn Fn()>,
    /// Hide the console from the start, not only while Azahar is open: pa3ds's own window
    /// stands in for it (`pa3ds.ux.window`).
    early: bool,
    /// What else to hide while Azahar is open: pa3ds's own window, where it has one.
    also: Option<Rc<dyn Fn()>>,
}

impl Beside {
    pub fn new(log: Log, look: f64) -> Self {
        Beside::with(
            log,
            look,
            Rc::new(|| window::emulator_pid(window::EMULATOR)),
            Rc::new(hide),
        )
    }

    /// `new`, with how Azahar is found and how pa3ds hides given: a test's own.
    pub fn with(log: Log, look: f64, find: Rc<dyn Fn() -> i64>, hide: Rc<dyn Fn()>) -> Self {
        Beside {
            log,
            look,
            next: Cell::new(0.0),
            azahar: Cell::new(0),
            find,
            hide,
            early: false,
            also: None,
        }
    }

    /// Python's `early` and `also`, which default to False and None.
    pub fn own_window(mut self, early: bool, also: Option<Rc<dyn Fn()>>) -> Self {
        self.early = early;
        self.also = also;
        self
    }

    pub fn closed(&self) -> bool {
        let now = time::monotonic();
        if now < self.next.get() {
            return false;
        }
        self.next.set(now + self.look);
        if self.early {
            (self.hide)();
        }
        if self.azahar.get() == 0 {
            let pid = (self.find)();
            if pid != 0 {
                self.azahar.set(opened(pid, SYNCHRONIZE));
            }
        }
        if self.azahar.get() == 0 {
            return false;
        }
        if !alive(self.azahar.get()) {
            (self.log)("Azahar has closed, and pa3ds closes with it");
            return true;
        }
        (self.hide)();
        if let Some(also) = &self.also {
            also();
        }
        false
    }
}
