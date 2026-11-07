//! Asking the player for a word, in a box a screen reader reads.
//!
//! One thing pa3ds needs from the player rather than from the game: a name for something the game
//! never names. The console is not it — the plan is to have no console (ADR-0026) — and alt-tabbing
//! out of the game to type is the tedium the feature exists to remove.
//!
//! **A native Win32 dialog**, built from a template in memory with `ctypes` and no new dependency.
//! Not Tk: Tk draws its own controls, so a screen reader sees a picture of a text box. Measured on
//! this machine with NVDA (spike 81): the dialog here is announced as
//! "Sign, 1 north. Called: Signpost 1 — edit — selected Signpost 1", and tabbing names OK and Cancel.
//! The Tk version of the same dialog announced only its class, `TkChild`, **and Windows refused it
//! the foreground** while Azahar was in front, so it opened behind the game. This one took the
//! foreground from behind Azahar every time.
//!
//! Two things make it read: it is a real `#32770`, the system dialog class, which is what MSAA
//! reports as a dialog and what makes a screen reader read the whole box when it opens; and the
//! static label sits immediately before the edit in tab order, which is how Win32 says which text
//! names which control.
//!
//! **The dialog runs on a thread of its own.** The poll loop must not stop while it is up — the game
//! keeps running, and a player who opens it mid-battle should not lose the battle text. The answer
//! comes back the way a keypress does, through a queue the runtime drains at the next poll. One at a
//! time: a second ask while one is open is refused rather than stacking boxes over the game.
//!
//! **On macOS and Linux** the port's own, as an experiment: the box is the system's own dialog,
//! shown by `osascript`, `zenity` or `kdialog` (`native`), on the same thread of its own.
//!
//! Bound by:
//! - ADR-0023: a rename is a native box on a thread of its own, and the new name is said back as
//!   stored.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

#[cfg(windows)]
use std::cell::RefCell;
#[cfg(windows)]
use std::ptr::null_mut;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{GetLastError, HWND, LPARAM, WPARAM};
#[cfg(windows)]
use windows_sys::Win32::UI::Controls::{CheckDlgButton, IsDlgButtonChecked};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DLGTEMPLATE, DialogBoxIndirectParamW, EndDialog, GetDlgItemTextW, SetDlgItemTextW,
    SetForegroundWindow,
};

use crate::core::words::own;
use crate::py::{Error, Kind, Result};

/// The buttons' words of the box asked for last, found on the thread that asked (`Asker.ask`).
static BUTTONS: Mutex<Option<(String, String)>> = Mutex::new(None);

/// "OK" and "Cancel" in the words in place, or as the thread that asked for a box found them.
fn buttons() -> (String, String) {
    lock(&BUTTONS)
        .clone()
        .unwrap_or_else(|| (own("dialog-ok", &[]), own("dialog-cancel", &[])))
}

pub const DS_SETFONT: i64 = 0x0040;
pub const DS_MODALFRAME: i64 = 0x0080;
pub const DS_CENTER: i64 = 0x0800;
pub const WS_POPUP: i64 = 0x80000000;
pub const WS_CAPTION: i64 = 0x00C00000;
pub const WS_SYSMENU: i64 = 0x00080000;
pub const WS_VISIBLE: i64 = 0x10000000;
pub const WS_CHILD: i64 = 0x40000000;
pub const WS_TABSTOP: i64 = 0x00010000;
pub const WS_BORDER: i64 = 0x00800000;
pub const WS_GROUP: i64 = 0x00020000;
pub const ES_AUTOHSCROLL: i64 = 0x0080;
pub const BS_DEFPUSHBUTTON: i64 = 0x0001;
pub const BS_AUTOCHECKBOX: i64 = 0x0003;
pub const BST_CHECKED: i64 = 1;
pub const SS_LEFT: i64 = 0x0000;
pub const WS_EX_CLIENTEDGE: i64 = 0x00000200;

/// How a dialog template names the predefined control classes.
pub const ATOM_BUTTON: i64 = 0x0080;
pub const ATOM_EDIT: i64 = 0x0081;
pub const ATOM_STATIC: i64 = 0x0082;

/// How a dialog is shown: title, label, the name it starts with and its checkbox's words, none
/// for no checkbox; back to the name given and whether the box was ticked, or None for a dialog
/// cancelled. `ask` is the real one; a test passes its own.
///
/// It runs on the dialog's thread, so it is `Send` and `Sync`.
pub type Ask = Arc<dyn Fn(&str, &str, &str, &str) -> Result<Option<(String, bool)>> + Send + Sync>;

pub const IDOK: i64 = 1;
pub const IDCANCEL: i64 = 2;
pub const IDC_LABEL: i64 = 100;
pub const IDC_EDIT: i64 = 101;
pub const IDC_CHECK: i64 = 102;

pub const WM_INITDIALOG: i64 = 0x0110;
pub const WM_COMMAND: i64 = 0x0111;

/// The most characters a name may be. Longer than anything worth hearing, and it stops a name
/// from being a paragraph pasted in by accident.
pub const LIMIT: i64 = 64;

/// What came back from one dialog: the name given, and whatever the asker needs to know what
/// it was for. An empty `said` is a name cleared; a cancelled dialog gives no Answer at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    /// Which question this answers, in the asker's own words.
    pub what: String,
    pub said: String,
    /// Whether the box's checkbox was ticked; False where it had none.
    pub ticked: bool,
}

/// A null-terminated UTF-16 string, as every string in a dialog template is.
pub fn wstr(text: &str) -> Vec<u8> {
    let mut out: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
    out.extend_from_slice(&[0, 0]);
    out
}

/// Each item of a template starts on a 4-byte boundary. Windows does not forgive this: a
/// template one byte out builds the wrong controls, or none.
pub fn align(data: &mut Vec<u8>) {
    while !data.len().is_multiple_of(4) {
        data.push(0);
    }
}

// `struct.pack` raises `struct.error` for a number that does not fit its format, at the line that
// packs it, so these raise for their callers' lines.

/// `struct.pack("<h", value)`.
#[track_caller]
fn short(value: i64) -> Result<[u8; 2]> {
    i16::try_from(value)
        .map(i16::to_le_bytes)
        .map_err(|_| Error::unpack("'h' format requires -32768 <= number <= 32767"))
}

/// `struct.pack("<H", value)`.
#[track_caller]
fn ushort(value: i64) -> Result<[u8; 2]> {
    u16::try_from(value)
        .map(u16::to_le_bytes)
        .map_err(|_| Error::unpack("'H' format requires 0 <= number <= 65535"))
}

/// `struct.pack("<I", value)`.
#[track_caller]
fn uint(value: i64) -> Result<[u8; 4]> {
    u32::try_from(value)
        .map(u32::to_le_bytes)
        .map_err(|_| Error::unpack("'I' format requires 0 <= number <= 4294967295"))
}

/// A control's class in a template, Python's `int | str`: a predefined one's atom, or a class's
/// name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Class {
    Atom(i64),
    Name(String),
}

impl From<i64> for Class {
    fn from(atom: i64) -> Self {
        Class::Atom(atom)
    }
}

impl From<&str> for Class {
    fn from(name: &str) -> Self {
        Class::Name(name.to_string())
    }
}

/// One control: its class a predefined one's atom, or a class's name, as a common
/// control's, `SysTabControl32`, has no atom.
pub fn item(
    data: &mut Vec<u8>,
    style: i64,
    ex_style: i64,
    x: i64,
    y: i64,
    cx: i64,
    cy: i64,
    ident: i64,
    atom: impl Into<Class>,
    text: &str,
) -> Result<()> {
    align(data);
    data.extend_from_slice(&uint(style & 0xFFFFFFFF)?);
    data.extend_from_slice(&uint(ex_style)?);
    for number in [x, y, cx, cy] {
        data.extend_from_slice(&short(number)?);
    }
    data.extend_from_slice(&ushort(ident)?);
    match atom.into() {
        Class::Atom(atom) => {
            data.extend_from_slice(&ushort(0xFFFF)?);
            data.extend_from_slice(&ushort(atom)?);
        }
        Class::Name(name) => data.extend_from_slice(&wstr(&name)),
    }
    data.extend_from_slice(&wstr(text));
    data.extend_from_slice(&ushort(0)?); // no creation data
    Ok(())
}

/// The dialog as a DLGTEMPLATE in memory, rather than a resource in an executable.
///
/// Sizes are dialog units, not pixels: Windows scales them by the dialog's font, which is how a
/// box stays laid out at any DPI or text size. The label is given two lines' worth of room,
/// because it carries both what is being renamed and what it is called now. `check` puts a
/// checkbox of those words after the edit, a Tab away, which a checkbox's own words name. Python's
/// `check` defaults to "" and `width` to 260.
pub fn template(title: &str, label: &str, check: &str, width: i64) -> Result<Vec<u8>> {
    let (ok, cancel) = buttons();
    let mut items = Vec::new();
    // The label comes immediately before the edit, because that is how Win32 says which static
    // names which control: MSAA takes a control's name from the static before it in tab order.
    item(
        &mut items,
        WS_CHILD | WS_VISIBLE | WS_GROUP | SS_LEFT,
        0,
        7,
        7,
        width - 14,
        18,
        IDC_LABEL,
        ATOM_STATIC,
        label,
    )?;
    item(
        &mut items,
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL,
        WS_EX_CLIENTEDGE,
        7,
        29,
        width - 14,
        14,
        IDC_EDIT,
        ATOM_EDIT,
        "",
    )?;
    let mut buttons = 52;
    if !check.is_empty() {
        item(
            &mut items,
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_AUTOCHECKBOX,
            0,
            7,
            49,
            width - 14,
            12,
            IDC_CHECK,
            ATOM_BUTTON,
            check,
        )?;
        buttons = 68;
    }
    item(
        &mut items,
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_DEFPUSHBUTTON,
        0,
        width - 111,
        buttons,
        50,
        14,
        IDOK,
        ATOM_BUTTON,
        &ok,
    )?;
    item(
        &mut items,
        WS_CHILD | WS_VISIBLE | WS_TABSTOP,
        0,
        width - 57,
        buttons,
        50,
        14,
        IDCANCEL,
        ATOM_BUTTON,
        &cancel,
    )?;
    frame(
        DS_MODALFRAME | WS_SYSMENU,
        0,
        if check.is_empty() { 4 } else { 5 },
        width,
        buttons + 21,
        title,
        &items,
    )
}

/// A DLGTEMPLATE round `count` items already built: a visible, captioned, centred popup in
/// Segoe UI, with `style` and `ex_style` added.
pub fn frame(
    style: i64,
    ex_style: i64,
    count: i64,
    width: i64,
    height: i64,
    title: &str,
    items: &[u8],
) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    let style = style | DS_SETFONT | DS_CENTER | WS_POPUP | WS_CAPTION | WS_VISIBLE;
    data.extend_from_slice(&uint(style & 0xFFFFFFFF)?);
    data.extend_from_slice(&uint(ex_style)?);
    data.extend_from_slice(&ushort(count)?);
    for number in [0, 0, width, height] {
        data.extend_from_slice(&short(number)?);
    }
    data.extend_from_slice(&ushort(0)?); // no menu
    data.extend_from_slice(&ushort(0)?); // the default dialog class, which is what makes it a dialog
    data.extend_from_slice(&wstr(title));
    data.extend_from_slice(&ushort(9)?); // DS_SETFONT: point size, then the typeface
    data.extend_from_slice(&wstr("Segoe UI"));
    align(&mut data);
    data.extend_from_slice(items);
    Ok(data)
}

/// What the dialog procedure of the dialog this thread is showing reads and writes. `ask` hands
/// its `current` and its `answer` to a bare function through here, where Python's procedure
/// closed over them.
#[cfg(windows)]
struct Asking {
    current: String,
    check: String,
    answer: Option<(String, bool)>,
}

#[cfg(windows)]
thread_local! {
    static ASKING: RefCell<Option<Asking>> = const { RefCell::new(None) };
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
unsafe extern "system" fn proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
) -> isize {
    let message = message as i64;
    if message == WM_INITDIALOG {
        // Windows sends the edit's change back through this procedure while the text is set, so
        // `current` is cloned out rather than held borrowed across the call.
        let (current, check) = ASKING.with(|asking| {
            asking
                .borrow()
                .as_ref()
                .map(|asking| (asking.current.clone(), !asking.check.is_empty()))
                .unwrap_or_default()
        });
        // SAFETY: `window` is the dialog Windows is calling this for, and the text is a
        // NUL-terminated UTF-16 string that outlives the call.
        unsafe { SetDlgItemTextW(window, IDC_EDIT as i32, wide(&current).as_ptr()) };
        if check {
            // SAFETY: `window` is the dialog, and IDC_CHECK one of its controls.
            unsafe { CheckDlgButton(window, IDC_CHECK as i32, BST_CHECKED as u32) };
        }
        // pa3ds is not the foreground process when the key is pressed — the hook sees the
        // key but never receives it — so this can be refused, and the box would open
        // behind the game with the player hearing nothing. Measured as working from
        // behind Azahar (spike 81); if it is ever refused, the dialog is still usable by
        // alt-tabbing and nothing is lost but the convenience.
        // SAFETY: SetForegroundWindow only reads the window handle.
        unsafe { SetForegroundWindow(window) };
        return 1; // let the dialog manager put focus on the first tab stop, the edit
    }
    if message == WM_COMMAND {
        let command = wparam as i64 & 0xFFFF;
        if command == IDOK {
            let mut buffer = vec![0u16; LIMIT as usize + 1];
            // SAFETY: `buffer` has room for `LIMIT + 1` UTF-16 units, which is what is passed.
            unsafe {
                GetDlgItemTextW(
                    window,
                    IDC_EDIT as i32,
                    buffer.as_mut_ptr(),
                    LIMIT as i32 + 1,
                )
            };
            let end = buffer
                .iter()
                .position(|&unit| unit == 0)
                .unwrap_or(buffer.len());
            let said = String::from_utf16_lossy(&buffer[..end]);
            ASKING.with(|asking| {
                if let Some(asking) = asking.borrow_mut().as_mut() {
                    // SAFETY: `window` is the dialog, and IDC_CHECK one of its controls.
                    let ticked = !asking.check.is_empty()
                        && unsafe { IsDlgButtonChecked(window, IDC_CHECK as i32) } as i64
                            == BST_CHECKED;
                    asking.answer = Some((said, ticked));
                }
            });
            // SAFETY: `window` is the dialog this procedure was called for.
            unsafe { EndDialog(window, IDOK as isize) };
            return 1;
        }
        if command == IDCANCEL {
            // SAFETY: as above.
            unsafe { EndDialog(window, IDCANCEL as isize) };
            return 1;
        }
    }
    0
}

/// Show the dialog and wait for it. The name given and whether the checkbox was ticked, or None
/// if it was cancelled. The checkbox, where there is one, starts ticked.
///
/// Blocking, and it runs its own message loop, so it belongs on a thread of its own — see
/// `Asker`. An empty string back is a name cleared, which is not the same as None.
#[cfg(windows)]
pub fn ask(title: &str, label: &str, current: &str, check: &str) -> Result<Option<(String, bool)>> {
    let built = template(title, label, check, 260)?;
    // The template must start on a 4-byte boundary, which a byte buffer does not promise and
    // Python's `create_string_buffer` happened to.
    let mut held = vec![0u32; built.len().div_ceil(4)];
    for (word, bytes) in held.iter_mut().zip(built.chunks(4)) {
        let mut padded = [0u8; 4];
        padded[..bytes.len()].copy_from_slice(bytes);
        *word = u32::from_le_bytes(padded);
    }
    ASKING.with(|asking| {
        *asking.borrow_mut() = Some(Asking {
            current: current.to_string(),
            check: check.to_string(),
            answer: None,
        });
    });
    // SAFETY: `held` is a DLGTEMPLATE as `template` lays one out, DWORD-aligned and alive for
    // the call, and `proc` has the signature of a dialog procedure.
    let shown = unsafe {
        DialogBoxIndirectParamW(
            null_mut(),
            held.as_ptr() as *const DLGTEMPLATE,
            null_mut(),
            Some(proc),
            0,
        )
    };
    if shown < 0 {
        // SAFETY: GetLastError takes no arguments.
        let code = unsafe { GetLastError() };
        let text = std::io::Error::from_raw_os_error(code as i32).to_string();
        let text = text
            .strip_suffix(&format!(" (os error {code})"))
            .unwrap_or(&text);
        return Err(Error::new(
            Kind::OsError,
            format!("[WinError {code}] {text}"),
        ));
    }
    Ok(ASKING.with(|asking| asking.borrow_mut().take().and_then(|asking| asking.answer)))
}

/// `ask` on macOS and Linux: the system's own dialog (`native`), with the checkbox as a second,
/// yes-or-no question asked after the name, since neither has a text box with a checkbox in it.
#[cfg(not(windows))]
pub fn ask(title: &str, label: &str, current: &str, check: &str) -> Result<Option<(String, bool)>> {
    native::ask(title, label, current, check).map_err(|error| Error::new(Kind::OsError, error))
}

/// The dialogs of macOS and Linux, each one a run of the program the system has for it:
/// `osascript` on macOS, `zenity` or else `kdialog` on Linux. VoiceOver and Orca read them as
/// they read any program's. Every word goes to the program as an argument of its own, never into
/// a script's text, so a name is never read as AppleScript. A dialog cancelled is None; Err is a
/// dialog that could not be shown.
#[cfg(not(windows))]
pub mod native {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::LIMIT;

    /// `osascript` running `script`'s `on run argv` with `argv`.
    #[cfg(target_os = "macos")]
    fn osascript(script: &str, argv: &[&str]) -> Command {
        let mut command = Command::new("/usr/bin/osascript");
        command.arg("-e").arg(script).args(argv);
        command
    }

    /// The Linux dialog program installed, zenity before kdialog.
    #[cfg(not(target_os = "macos"))]
    fn tool() -> Result<&'static str, String> {
        let path = std::env::var_os("PATH").unwrap_or_default();
        ["zenity", "kdialog"]
            .into_iter()
            .find(|tool| std::env::split_paths(&path).any(|dir| dir.join(tool).is_file()))
            .ok_or_else(|| "neither zenity nor kdialog is installed".to_string())
    }

    /// What the dialog printed, its last newline off, or None when it was cancelled.
    fn run(command: &mut Command) -> Result<Option<String>, String> {
        let output = command.output().map_err(|error| error.to_string())?;
        outcome(output.status.code(), &output.stdout, &output.stderr)
    }

    /// `run`'s answer from how the dialog ended. zenity and kdialog end 1 when cancelled;
    /// osascript ends 1 for any error, and says -128 for the one that is a cancel.
    pub(super) fn outcome(
        code: Option<i32>,
        stdout: &[u8],
        stderr: &[u8],
    ) -> Result<Option<String>, String> {
        let said = String::from_utf8_lossy(stderr);
        match code {
            Some(0) => {
                let printed = String::from_utf8_lossy(stdout);
                Ok(Some(
                    printed.strip_suffix('\n').unwrap_or(&printed).to_string(),
                ))
            }
            Some(1) if !cfg!(target_os = "macos") || said.contains("(-128)") => Ok(None),
            _ => Err(match said.trim() {
                "" => format!("the dialog ended with {code:?}"),
                said => said.to_string(),
            }),
        }
    }

    /// The name given, cut to `LIMIT` characters as the Windows box's edit is.
    fn name(said: &str) -> String {
        said.chars().take(LIMIT as usize).collect()
    }

    /// A name, starting as `current`, and whether `check` was answered yes, where there is one.
    #[cfg(target_os = "macos")]
    pub fn ask(
        title: &str,
        label: &str,
        current: &str,
        check: &str,
    ) -> Result<Option<(String, bool)>, String> {
        // Printed as the checkbox's answer, 1 or 0, then the name: a cancel of the second
        // dialog is a box left unticked, not a rename cancelled.
        const SCRIPT: &str = r#"on run argv
    activate
    set answer to text returned of (display dialog (item 2 of argv) with title (item 1 of argv) default answer (item 3 of argv))
    set ticked to "0"
    if (item 4 of argv) is not "" then
        try
            display dialog (item 4 of argv) with title (item 1 of argv)
            set ticked to "1"
        end try
    end if
    return ticked & answer
end run"#;
        let printed = run(&mut osascript(SCRIPT, &[title, label, current, check]))?;
        Ok(printed.map(|printed| {
            let (ticked, said) = printed.split_at(printed.len().min(1));
            (name(said), ticked == "1")
        }))
    }

    #[cfg(not(target_os = "macos"))]
    pub fn ask(
        title: &str,
        label: &str,
        current: &str,
        check: &str,
    ) -> Result<Option<(String, bool)>, String> {
        let tool = tool()?;
        let mut entry = Command::new(tool);
        match tool {
            "zenity" => entry.args(["--entry", "--title", title, "--text", label]),
            _ => entry.args(["--title", title, "--inputbox", label]),
        };
        match tool {
            "zenity" => entry.args(["--entry-text", current]),
            _ => entry.arg(current),
        };
        let Some(said) = run(&mut entry)? else {
            return Ok(None);
        };
        let ticked = !check.is_empty() && confirm(title, check).unwrap_or(false);
        Ok(Some((name(&said), ticked)))
    }

    /// Whether the player answers yes to `text`. Cancel, or No, is false.
    pub fn confirm(title: &str, text: &str) -> Result<bool, String> {
        #[cfg(target_os = "macos")]
        let mut command = osascript(
            "on run argv\nactivate\ndisplay dialog (item 2 of argv) with title (item 1 of argv)\nend run",
            &[title, text],
        );
        #[cfg(not(target_os = "macos"))]
        let mut command = match tool()? {
            "zenity" => {
                let mut command = Command::new("zenity");
                command.args([
                    "--question",
                    "--no-markup",
                    "--title",
                    title,
                    "--text",
                    text,
                ]);
                command
            }
            tool => {
                let mut command = Command::new(tool);
                command.args(["--title", title, "--yesno", text]);
                command
            }
        };
        run(&mut command).map(|answer| answer.is_some())
    }

    /// `text` told to the player, until they close it.
    pub fn tell(title: &str, text: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        let mut command = osascript(
            "on run argv\nactivate\ndisplay alert (item 1 of argv) message (item 2 of argv)\nend run",
            &[title, text],
        );
        #[cfg(not(target_os = "macos"))]
        let mut command = match tool()? {
            "zenity" => {
                let mut command = Command::new("zenity");
                command.args(["--info", "--no-markup", "--title", title, "--text", text]);
                command
            }
            tool => {
                let mut command = Command::new(tool);
                command.args(["--title", title, "--msgbox", text]);
                command
            }
        };
        run(&mut command).map(drop)
    }

    /// The folder the player picks, starting in `initial` where it is one and the home folder
    /// otherwise; None when they cancel.
    pub fn folder(title: &str, initial: &Path) -> Result<Option<PathBuf>, String> {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let start = match initial.is_dir() {
            true => initial.to_path_buf(),
            false => home.unwrap_or_else(|| PathBuf::from("/")),
        };
        let start = start.to_string_lossy();
        #[cfg(target_os = "macos")]
        let mut command = osascript(
            "on run argv\nactivate\nreturn POSIX path of (choose folder with prompt (item 1 of argv) default location (POSIX file (item 2 of argv)))\nend run",
            &[title, &start],
        );
        #[cfg(not(target_os = "macos"))]
        let mut command = match tool()? {
            "zenity" => {
                let mut command = Command::new("zenity");
                // The trailing slash opens the picker inside the folder rather than beside it.
                let start = format!("{}/", start.trim_end_matches('/'));
                command.args(["--file-selection", "--directory", "--title", title]);
                command.args(["--filename", &start]);
                command
            }
            tool => {
                let mut command = Command::new(tool);
                command.args(["--title", title, "--getexistingdirectory", &start]);
                command
            }
        };
        Ok(run(&mut command)?.map(PathBuf::from))
    }
}

#[cfg(all(test, not(windows)))]
mod native_tests {
    use super::native::outcome;

    #[test]
    fn a_dialog_answered_is_what_it_printed_without_its_newline() {
        assert_eq!(
            outcome(Some(0), b"Sign\n", b""),
            Ok(Some("Sign".to_string()))
        );
    }

    #[test]
    fn a_dialog_cancelled_is_none() {
        let said: &[u8] = match cfg!(target_os = "macos") {
            true => b"execution error: User canceled. (-128)\n",
            false => b"",
        };
        assert_eq!(outcome(Some(1), b"", said), Ok(None));
    }

    #[test]
    fn a_dialog_that_failed_says_why() {
        assert_eq!(
            outcome(Some(255), b"", b"cannot open display\n"),
            Err("cannot open display".to_string())
        );
    }
}

/// `threading.Thread`, as far as `Asker` uses one: started, asked if it is alive, and joined with
/// a timeout. A `JoinHandle` can do neither of the last two, so the thread says when it is over.
struct Thread {
    over: Arc<(Mutex<bool>, Condvar)>,
}

impl Thread {
    fn start(name: &str, target: impl FnOnce() + Send + 'static) -> Result<Thread> {
        struct Finish(Arc<(Mutex<bool>, Condvar)>);
        impl Drop for Finish {
            fn drop(&mut self) {
                *lock(&self.0.0) = true;
                self.0.1.notify_all();
            }
        }

        let over = Arc::new((Mutex::new(false), Condvar::new()));
        let finish = Finish(over.clone());
        std::thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let _finish = finish;
                target();
            })
            .map_err(|_| Error::new(Kind::RuntimeError, "can't start new thread"))?;
        Ok(Thread { over })
    }

    fn is_alive(&self) -> bool {
        !*lock(&self.over.0)
    }

    fn join(&self, timeout: f64) {
        let over = lock(&self.over.0);
        drop(
            self.over
                .1
                .wait_timeout_while(over, Duration::from_secs_f64(timeout), |over| !*over)
                .unwrap_or_else(PoisonError::into_inner),
        );
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// One dialog at a time, on a thread of its own, answering through a queue.
///
/// The runtime never waits for a dialog: it asks, carries on reading the game, and picks the
/// answer up at a later poll. `open` says whether a box is already up, so a second press of the
/// key does not stack one over the other.
pub struct Asker {
    show: Ask,
    answers: Arc<Mutex<VecDeque<Answer>>>,
    thread: Option<Thread>,
}

impl Default for Asker {
    fn default() -> Self {
        Asker::new(None)
    }
}

impl Asker {
    pub fn new(show: Option<Ask>) -> Self {
        Asker {
            show: show.unwrap_or_else(|| Arc::new(ask)),
            answers: Arc::new(Mutex::new(VecDeque::new())),
            thread: None,
        }
    }

    /// Whether a dialog is up now.
    pub fn open(&self) -> bool {
        self.thread.as_ref().is_some_and(Thread::is_alive)
    }

    /// Put a dialog up, with a checkbox of `check`'s words where it says any. False when one
    /// is already open, and nothing happens. Python's `check` defaults to "".
    pub fn ask(
        &mut self,
        what: &str,
        title: &str,
        label: &str,
        current: &str,
        check: &str,
    ) -> Result<bool> {
        if self.open() {
            return Ok(false);
        }
        let (show, answers) = (self.show.clone(), self.answers.clone());
        // The box is built on its own thread, where the words in place are not: its buttons'
        // words are found here, as Python's thread reads the one module's.
        *lock(&BUTTONS) = Some((own("dialog-ok", &[]), own("dialog-cancel", &[])));
        let (what, title, label, current, check) = (
            what.to_string(),
            title.to_string(),
            label.to_string(),
            current.to_string(),
            check.to_string(),
        );
        self.thread = Some(Thread::start("dialog", move || {
            Asker::run(&show, &answers, &what, &title, &label, &current, &check)
        })?);
        Ok(true)
    }

    /// Wait for the open dialog to close; True once none is.
    ///
    /// For a test, which cannot ask a box to close and must not sleep for a guess. The runtime
    /// never waits: quitting with a dialog up ends the process, its thread being a daemon, and
    /// holding the exit while somebody finishes typing would be the wrong way round.
    pub fn wait(&self, timeout: f64) -> bool {
        if let Some(thread) = &self.thread {
            thread.join(timeout);
        }
        !self.open()
    }

    /// `wait` for the default five seconds.
    pub fn wait_default(&self) -> bool {
        self.wait(5.0)
    }

    /// Every dialog answered since the last call. A cancelled one is not an answer.
    pub fn answers(&self) -> Vec<Answer> {
        lock(&self.answers).drain(..).collect()
    }

    fn run(
        show: &Ask,
        answers: &Mutex<VecDeque<Answer>>,
        what: &str,
        title: &str,
        label: &str,
        current: &str,
        check: &str,
    ) {
        match show(title, label, current, check) {
            Ok(Some((said, ticked))) => lock(answers).push_back(Answer {
                what: what.to_string(),
                said,
                ticked,
            }),
            Ok(None) => {}
            // What `threading.excepthook` writes for a thread that raised.
            Err(error) => eprintln!(
                "Exception in thread dialog:\n{}: {}",
                error.kind.name(),
                error.message
            ),
        }
    }
}
