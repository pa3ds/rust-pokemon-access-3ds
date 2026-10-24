//! pa3ds's own window, "Pokémon Access 3DS", opened when pa3ds is double-clicked.
//!
//! **Why it exists.** OBS records a program's sound by one of its windows, and pa3ds had none: its
//! console is Windows Terminal's or conhost's, another process, so a streamer picking it recorded that
//! process instead of pa3ds's tones and footsteps (Orion, 2026-10-24; docs/plans/pa3ds-window.md).
//! This window is pa3ds's own process's. A player who double-clicked pa3ds also gets somewhere to
//! end it and to reach its pages from, and in time its settings.
//!
//! **When.** Only where pa3ds has its console alone (`instance.alone`), as a double-clicked pa3ds
//! has: started from a shell it opens none, nor in a `--silent` run. Its console is hidden as it
//! opens, the window standing in for it, and the window itself is hidden while Azahar is open as the
//! console was (ADR-0039), unless `[pa3ds] hide_window` is false: hidden, it is out of Alt+Tab and the
//! taskbar and goes on running, as NUtils hides a window for a streamer.
//!
//! **What it is.** A modeless dialog, the system's `#32770` as the rename box is, so a screen reader
//! reads it as a dialog and Tab walks its buttons: report an issue, and download Azahar's latest
//! zip or its installer (Orion), each opening the browser, the downloads straight at the file; and
//! Exit, which ends pa3ds as Azahar closing does. Closing the window ends pa3ds too;
//! Escape does nothing, so a stray press never ends it. It runs on a thread of its own with its own
//! message loop, as long as pa3ds does, and the poll loop never waits on it.
//!
//! Bound by:
//! - ADR-0039: opened only by a pa3ds alone on its console, never a silent run; hidden while
//!   Azahar is open unless `hide_window` is false; Exit ends pa3ds as Azahar closing does.

use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::{Arc, LazyLock};

use regex::Regex;

use crate::core::words::own;
use crate::py::Result;
use crate::ux::dialog::{
    ATOM_BUTTON, BS_DEFPUSHBUTTON, WS_CHILD, WS_GROUP, WS_SYSMENU, WS_TABSTOP, WS_VISIBLE, frame,
    item,
};

/// Where a player reports an issue: the port's, which is what players run (ADR-0033).
pub const ISSUES: &str = "https://github.com/pa3ds/rust-pokemon-access-3ds/issues";
/// Azahar's releases, opened where the latest release's file cannot be found.
pub const AZAHAR: &str = "https://github.com/azahar-emu/azahar/releases";
pub const AZAHAR_LATEST: &str = "https://api.github.com/repos/azahar-emu/azahar/releases/latest";
pub static PORTABLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^azahar-windows-msvc-[\d.]+\.zip$").unwrap());
/// Azahar's files for Windows, by name: the build made with Microsoft's compiler, the one its
/// README offers first, as a zip to unpack anywhere and as an installer. Each name carries the
/// version (`azahar-windows-msvc-2126.2.zip`), so the latest release is asked for each time.
pub static INSTALLER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^azahar-windows-msvc-[\d.]+-installer\.exe$").unwrap());

pub const IDC_ISSUES: i64 = 201;
pub const IDC_PORTABLE: i64 = 202;
pub const IDC_INSTALLER: i64 = 203;
pub const IDC_EXIT: i64 = 204;
/// The buttons, in tab order, by their ids and messages.
pub const BUTTONS: [(i64, &str); 4] = [
    (IDC_ISSUES, "window-issues"),
    (IDC_PORTABLE, "window-azahar-portable"),
    (IDC_INSTALLER, "window-azahar-installer"),
    (IDC_EXIT, "window-exit"),
];

pub const WS_MINIMIZEBOX: i64 = 0x00020000;
/// On the taskbar and in Alt+Tab, as a program's main window is, though it has no owner.
pub const WS_EX_APPWINDOW: i64 = 0x00040000;
pub const WM_CLOSE: i64 = 0x0010;
pub const SW_HIDE: i64 = 0;
pub const SW_SHOWNORMAL: i64 = 1;
pub const WIDTH: i64 = 160;
pub const ROW: i64 = 18;

/// The window as a DLGTEMPLATE: its buttons one under another, the first the default.
pub fn template() -> Result<Vec<u8>> {
    let mut items = Vec::new();
    for (n, (ident, message)) in BUTTONS.iter().enumerate() {
        let first = if n == 0 {
            WS_GROUP | BS_DEFPUSHBUTTON
        } else {
            0
        };
        item(
            &mut items,
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | first,
            0,
            7,
            7 + ROW * n as i64,
            WIDTH - 14,
            14,
            *ident,
            ATOM_BUTTON,
            &own(message, &[]),
        )?;
    }
    frame(
        WS_SYSMENU | WS_MINIMIZEBOX,
        WS_EX_APPWINDOW,
        BUTTONS.len() as i64,
        WIDTH,
        7 + ROW * BUTTONS.len() as i64 + 3,
        &own("window-title", &[]),
        &items,
    )
}

/// The link to the file of GitHub's latest-release JSON whose name is `name`, or None.
///
/// The Python reads the JSON and matches each asset's `name`; with no JSON crate here, each
/// `browser_download_url` is matched by its last part, which GitHub makes the asset's name.
pub fn download(release: &str, name: &Regex) -> Option<String> {
    static URL: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#""browser_download_url"\s*:\s*"([^"]+)""#).unwrap());
    URL.captures_iter(release)
        .map(|found| found[1].to_string())
        .find(|url| name.is_match(url.rsplit('/').next().unwrap_or("")))
}

/// Where to send the player for Azahar's file `name`: the latest release's file, or the
/// releases page where GitHub cannot be reached or has no such file.
///
/// Windows' own `curl.exe` asks, as `update.rs` does, where the Python uses `urllib`.
pub fn azahar(name: &Regex) -> String {
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    let answer = std::process::Command::new(
        std::path::Path::new(&root)
            .join("System32")
            .join("curl.exe"),
    )
    .args([
        "-fsSL",
        "--max-time",
        "10",
        "-H",
        "User-Agent: pa3ds",
        AZAHAR_LATEST,
    ])
    .output();
    answer
        .ok()
        .filter(|answer| answer.status.success())
        .and_then(|answer| download(&String::from_utf8_lossy(&answer.stdout), name))
        .unwrap_or_else(|| AZAHAR.to_string())
}

#[cfg(windows)]
mod win {
    use super::*;
    use std::cell::RefCell;
    use std::ptr::null_mut;

    use windows_sys::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateDialogIndirectParamW, DLGTEMPLATE, DestroyWindow, DispatchMessageW, GetMessageW,
        IsDialogMessageW, MSG, PostQuitMessage, ShowWindowAsync, TranslateMessage,
    };

    use crate::ux::dialog::{IDCANCEL, WM_COMMAND, WM_INITDIALOG};

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Open `url` in the player's browser.
    pub fn browse(url: &str) {
        // SAFETY: every string is NUL-terminated and outlives the call.
        unsafe {
            ShellExecuteW(
                null_mut(),
                wide("open").as_ptr(),
                wide(url).as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL as i32,
            )
        };
    }

    pub fn hide(window: isize) {
        // Async: the window's thread does it, and the poll loop never waits on that thread.
        // SAFETY: ShowWindowAsync only posts to the window, which may be gone, and then fails.
        unsafe { ShowWindowAsync(window as HWND, SW_HIDE as i32) };
    }

    thread_local! {
        /// What the window's procedure sets as the player ends pa3ds: Python's procedure closed
        /// over it.
        static EXITED: RefCell<Option<Arc<AtomicBool>>> = const { RefCell::new(None) };
    }

    fn end(window: HWND) {
        EXITED.with(|exited| {
            if let Some(exited) = exited.borrow().as_ref() {
                exited.store(true, Ordering::SeqCst);
            }
        });
        // SAFETY: `window` is the dialog this thread made.
        unsafe {
            DestroyWindow(window);
            PostQuitMessage(0);
        }
    }

    unsafe extern "system" fn proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> isize {
        let message = message as i64;
        if message == WM_INITDIALOG {
            return 1; // focus on the first button
        }
        if message == WM_CLOSE {
            end(window);
            return 1;
        }
        if message == WM_COMMAND {
            let command = wparam as i64 & 0xFFFF;
            if command == IDC_ISSUES {
                browse(ISSUES);
            } else if command == IDC_PORTABLE || command == IDC_INSTALLER {
                // Asking GitHub takes a moment: not on the window's thread, which would
                // stop answering the screen reader meanwhile.
                let name: &'static Regex = if command == IDC_PORTABLE {
                    &PORTABLE
                } else {
                    &INSTALLER
                };
                let _ = std::thread::Builder::new()
                    .name("azahar".to_string())
                    .spawn(move || browse(&azahar(name)));
            } else if command == IDC_EXIT {
                end(window);
            }
            let known = BUTTONS.iter().any(|(ident, _)| *ident == command) || command == IDCANCEL;
            return known as isize;
        }
        0
    }

    /// Open the window and run its message loop until it is closed.
    pub fn show(built: Vec<u8>, exited: Arc<AtomicBool>, opened: Arc<AtomicIsize>) {
        EXITED.with(|slot| *slot.borrow_mut() = Some(exited));
        // The template must start on a 4-byte boundary, as in `dialog::ask`.
        let mut held = vec![0u32; built.len().div_ceil(4)];
        for (word, bytes) in held.iter_mut().zip(built.chunks(4)) {
            let mut padded = [0u8; 4];
            padded[..bytes.len()].copy_from_slice(bytes);
            *word = u32::from_le_bytes(padded);
        }
        // SAFETY: `held` is a DLGTEMPLATE as `template` lays one out, DWORD-aligned and alive
        // while the window is, and `proc` has the signature of a dialog procedure.
        let window = unsafe {
            CreateDialogIndirectParamW(
                null_mut(),
                held.as_ptr() as *const DLGTEMPLATE,
                null_mut(),
                Some(proc),
                0,
            )
        };
        if window.is_null() {
            // What `threading.excepthook` writes for a thread that raised.
            eprintln!(
                "Exception in thread window:\nOSError: {}",
                std::io::Error::last_os_error()
            );
            return;
        }
        opened.store(window as isize, Ordering::SeqCst);
        // SAFETY: MSG is plain data, and every call is given this thread's own message.
        let mut message: MSG = unsafe { std::mem::zeroed() };
        while unsafe { GetMessageW(&mut message, null_mut(), 0, 0) } > 0 {
            if unsafe { IsDialogMessageW(window, &message) } == 0 {
                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }
    }
}

/// pa3ds's window, open from the moment it is made, on a thread of its own.
#[derive(Clone)]
pub struct Window {
    exited: Arc<AtomicBool>,
    opened: Arc<AtomicIsize>,
}

impl Window {
    pub fn new() -> Result<Self> {
        let window = Window {
            exited: Arc::new(AtomicBool::new(false)),
            opened: Arc::new(AtomicIsize::new(0)),
        };
        // The words in place are this thread's, not the window's: the template is built here,
        // where the Python's thread builds it from the one module's words.
        let built = template()?;
        #[cfg(windows)]
        {
            let (exited, opened) = (window.exited.clone(), window.opened.clone());
            let _ = std::thread::Builder::new()
                .name("window".to_string())
                .spawn(move || win::show(built, exited, opened));
        }
        #[cfg(not(windows))]
        {
            let _ = built;
            eprintln!("Exception in thread window:\nOSError: a window needs Windows");
        }
        Ok(window)
    }

    /// Whether the player has ended pa3ds from the window.
    pub fn exited(&self) -> bool {
        self.exited.load(Ordering::SeqCst)
    }

    /// Hide the window, once it is open. It goes on running, and OBS goes on recording.
    pub fn hide(&self) {
        let opened = self.opened.load(Ordering::SeqCst);
        if opened != 0 && !self.exited() {
            #[cfg(windows)]
            win::hide(opened);
        }
    }
}
