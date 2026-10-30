//! The mod settings: `config.toml`'s settings as controls, opened from pa3ds's window.
//!
//! **What it is** (ADR-0039 as amended 2026-10-30). A modal `#32770` over pa3ds's window: a tab for
//! each page, General, Speech, Audio, Navigation and Extras, one a table of the file, and the page
//! chosen under them, a control a setting. Ctrl+Tab and Ctrl+Shift+Tab change the page from
//! anywhere in it, as Windows' own property sheets do (Orion, 2026-10-25: a list of pages said its
//! item twice, a list box raising focus for itself and for the item). Each control is named by its
//! setting's short name, which is what Tab says. What the setting does is said after it by pa3ds's
//! own speech a tenth of a second after the control has the focus, without interrupting, so the
//! screen reader says the name, the control and its value first (Orion: a description Windows gives
//! the control is read between the control and its value, which is clumsier; and sent the moment
//! the control had the focus, NVDA said pa3ds's first, 2026-10-26).
//!
//! - on or off: a checkbox
//! - the language: a list to choose from, the game's own first, then each language pa3ds has words
//!   in, each by its own name
//! - one of a few words, as the hidden items' (`config.CHOICES`): a list of them, each by its
//!   message
//! - a number: a box to type it into, whose arrows step it; a loudness in percent, 100 as loud as
//!   the game, where the file has 1
//! - the folder of a portable Azahar: a box to type it into, and Browse, the folder picker
//!
//! **OK, Cancel and Apply**, as Windows' own settings dialogs have (Orion): applying writes each
//! setting changed on its own line of the file (`config.change`), then asks whether to start pa3ds
//! again now, for them to take; the new pa3ds takes this one's place (`instance.again`). Otherwise
//! they take at the next start. Nothing the poll loop does asks after the settings (Orion: rather a
//! restart than a check every tick).
//!
//! Bound by:
//! - ADR-0039: a change takes by starting pa3ds again, never in the poll loop, and applying writes
//!   each setting changed on its own line, nothing else in the file.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::config::{self, Config, Value};
use crate::core::keyfile;
use crate::core::words::{ENGLISH, Words, own};
use crate::py::{self, Result};
use crate::ux::dialog::{
    ATOM_BUTTON, ATOM_EDIT, ATOM_STATIC, BS_AUTOCHECKBOX, BS_DEFPUSHBUTTON, Class, DS_MODALFRAME,
    ES_AUTOHSCROLL, IDCANCEL, IDOK, SS_LEFT, WS_BORDER, WS_CHILD, WS_EX_CLIENTEDGE, WS_GROUP,
    WS_SYSMENU, WS_TABSTOP, WS_VISIBLE, frame, item,
};

/// Each page's message and the table of the file it shows, in the tabs' order.
pub const PAGES: [(&str, &str); 7] = [
    ("settings-page-general", "pa3ds"),
    ("settings-page-speech", "pa3ds.ui"),
    ("settings-page-audio", "pa3ds.audio"),
    ("settings-page-navigation", "pa3ds.navigation"),
    ("settings-page-touch", "pa3ds.touch"),
    ("settings-page-amie", "pa3ds.amie"),
    ("settings-page-extras", "pa3ds.extras"),
];
/// Where `config.Tables` keeps each table.
pub const ON_CONFIG: [(&str, &str); 7] = [
    ("pa3ds", "general"),
    ("pa3ds.ui", "ui"),
    ("pa3ds.audio", "audio"),
    ("pa3ds.navigation", "navigation"),
    ("pa3ds.touch", "touch"),
    ("pa3ds.amie", "amie"),
    ("pa3ds.extras", "extras"),
];

pub const CHECK: &str = "check";
pub const CHOICE: &str = "choice";
pub const NUMBER: &str = "number";
pub const PERCENT: &str = "percent";
pub const FOLDER: &str = "folder";
/// Each setting that is not on or off, by its control. The last ones are Python's
/// `**dict.fromkeys(config.CHOICES, CHOICE)`, a const array having no spread.
pub const KINDS: [(&str, &str); 9] = [
    ("pa3ds.ui.language", CHOICE),
    ("pa3ds.azahar_folder", FOLDER),
    ("pa3ds.poll_rate", NUMBER),
    ("pa3ds.audio.tones", PERCENT),
    ("pa3ds.audio.footsteps", PERCENT),
    ("pa3ds.audio.ticks", PERCENT),
    ("pa3ds.audio.feedback_sounds", PERCENT),
    (config::CHOICES[0].0, CHOICE),
    (config::CHOICES[1].0, CHOICE),
];
/// What a number's arrows step between: reads a second, and a loudness in percent, ten times as
/// loud as the game the most anyone will want.
pub const RANGES: [(&str, (i64, i64)); 2] = [(NUMBER, (1, 240)), (PERCENT, (0, 1000))];

pub const IDC_PAGES: i64 = 301;
pub const IDC_APPLY: i64 = 302;
/// Setting n's control is `FIRST + 10 * n`, its label one more, its Browse two and its arrows
/// three.
pub const FIRST: i64 = 1000;

pub const LEFT: i64 = 7;
pub const WIDE: i64 = 286;
pub const WIDTH: i64 = LEFT + WIDE + 7;
/// Where a page starts, under the tabs.
pub const TOP: i64 = 7 + 14 + 6;
pub const PAGE_HEIGHT: i64 = 136;
pub const HEIGHT: i64 = TOP + PAGE_HEIGHT + 7 + 14 + 7;
/// The tabs' class, a common control's, which has no atom.
pub const TABS: &str = "SysTabControl32";

/// A checkbox that says when it has the focus, for its setting to be described.
pub const BS_NOTIFY: i64 = 0x4000;
pub const WS_VSCROLL: i64 = 0x00200000;
pub const CBS_DROPDOWNLIST: i64 = 0x0003;
pub const ES_NUMBER: i64 = 0x2000;
pub const ATOM_COMBOBOX: i64 = 0x0085;

/// One setting as a control: its name in the file (`pa3ds.ui.pauses`), which control, the
/// page it is on, and its control's id.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub kind: &'static str,
    pub page: i64,
    pub ident: i64,
}

/// Every setting, page by page, in the file's order.
pub fn fields() -> Vec<Field> {
    let mut out: Vec<Field> = Vec::new();
    for (page, (_message, table)) in PAGES.iter().enumerate() {
        for (each, _default) in config::defaults(table) {
            let name = format!("{table}.{each}");
            let kind = KINDS
                .iter()
                .find(|(setting, _)| *setting == name)
                .map_or(CHECK, |(_, kind)| *kind);
            let ident = FIRST + 10 * out.len() as i64;
            out.push(Field {
                name,
                kind,
                page: page as i64,
                ident,
            });
        }
    }
    out
}

/// The language list's choices, (folder, what it is called): the game's own first, "", then
/// each language pa3ds has words in, by its own name.
pub fn languages() -> Result<Vec<(String, String)>> {
    let common = keyfile::data_path()?.join("common");
    let found: Vec<String> = if common.is_dir() {
        let mut found = Vec::new();
        for path in std::fs::read_dir(&common)? {
            let path = path?.path();
            if path.join("words.ftl").is_file() {
                found.push(
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
        found.sort();
        found
    } else {
        vec![ENGLISH.to_string()]
    };
    let mut out = vec![(String::new(), own("settings-language-game", &[]))];
    for code in found {
        let said = Words::new(&code, None, IndexMap::new(), Rc::new(|_line: &str| {}))?
            .say("language-name", &[]);
        out.push((code, said));
    }
    Ok(out)
}

/// A list's choices, (what the file has, what it is called): the languages, or one of
/// `config.CHOICES`' words each by its message, `setting-pa3ds-extras-hidden_items-off`.
pub fn choices(name: &str) -> Result<Vec<(String, String)>> {
    let Some((_, words)) = config::CHOICES.iter().find(|(setting, _)| *setting == name) else {
        return languages();
    };
    Ok(words
        .iter()
        .map(|word| {
            (
                word.to_string(),
                own(&format!("{}-{word}", config::message(name)), &[]),
            )
        })
        .collect())
}

/// Each setting's value, by its name in the file.
pub fn values(settings: &Config) -> IndexMap<String, Value> {
    use crate::core::config::Table;
    let tables: IndexMap<&str, Vec<(&str, Value)>> = ON_CONFIG
        .iter()
        .map(|(table, on)| {
            let held = match *on {
                "general" => settings.tables.general.fields(),
                "ui" => settings.tables.ui.fields(),
                "audio" => settings.tables.audio.fields(),
                "navigation" => settings.tables.navigation.fields(),
                "touch" => settings.tables.touch.fields(),
                "amie" => settings.tables.amie.fields(),
                _ => settings.tables.extras.fields(),
            };
            (*table, held)
        })
        .collect();
    fields()
        .into_iter()
        .map(|each| {
            let (table, setting) = each.name.rsplit_once('.').unwrap_or(("", &each.name));
            let value = tables[table]
                .iter()
                .find(|(name, _)| *name == setting)
                .map(|(_, value)| value.clone())
                .expect("every field is one of its table's");
            (each.name, value)
        })
        .collect()
}

/// A value as a number, where Python's `isinstance(value, int | float)` holds: a bool among them.
fn number(value: &Value) -> Option<f64> {
    match value {
        Value::Bool(value) => Some(f64::from(u8::from(*value))),
        Value::Int(value) => Some(*value as f64),
        Value::Float(value) => Some(*value),
        Value::Str(_) => None,
    }
}

/// A number's value as its box shows it: a loudness in percent.
pub fn shown(kind: &str, value: &Value) -> Result<String> {
    if kind == PERCENT
        && let Some(value) = number(value)
    {
        return Ok(py::round(value * 100.0)?.to_string());
    }
    if kind == NUMBER
        && let Some(value) = number(value)
    {
        return Ok(py::format_g(value));
    }
    Ok(match value {
        Value::Bool(value) => if *value { "True" } else { "False" }.to_string(),
        Value::Int(value) => value.to_string(),
        Value::Float(value) => py::float_repr(*value),
        Value::Str(value) => value.clone(),
    })
}

/// What a number's box says, as the file has it, or what it was where the box says nothing a
/// setting can take: a reads-a-second of 0 never reads the game.
pub fn typed(kind: &str, text: &str, was: &Value) -> Value {
    let Ok(number) = py::int_base10(py::strip(text)) else {
        return was.clone();
    };
    if kind == PERCENT {
        return if number >= 0 {
            Value::Float(number as f64 / 100.0)
        } else {
            was.clone()
        };
    }
    if number > 0 {
        Value::Float(number as f64)
    } else {
        was.clone()
    }
}

/// The settings `after` gives another value than `before`, for `config.change`.
pub fn changed(
    before: &IndexMap<String, Value>,
    after: &IndexMap<String, Value>,
) -> IndexMap<String, Value> {
    after
        .iter()
        .filter(|(name, value)| before.get(*name) != Some(*value))
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
}

/// The dialog as a DLGTEMPLATE: the tabs, each page's controls under one another in the
/// space under them, only the first page's shown, and OK, Cancel and Apply under them all. The
/// order is Tab's: a page's controls, those shown, between the tabs and the buttons.
pub fn template() -> Result<Vec<u8>> {
    let mut items = Vec::new();
    let mut count = 0;

    let mut add = |style: i64,
                   ex_style: i64,
                   x: i64,
                   y: i64,
                   cx: i64,
                   cy: i64,
                   ident: i64,
                   atom: Class,
                   text: &str|
     -> Result<()> {
        item(
            &mut items,
            WS_CHILD | style,
            ex_style,
            x,
            y,
            cx,
            cy,
            ident,
            atom,
            text,
        )?;
        count += 1;
        Ok(())
    };

    add(
        WS_VISIBLE | WS_TABSTOP | WS_GROUP,
        0,
        LEFT,
        7,
        WIDE,
        14,
        IDC_PAGES,
        TABS.into(),
        "",
    )?;
    let mut y = vec![TOP; PAGES.len()];
    for each in fields() {
        let seen = if each.page == 0 { WS_VISIBLE } else { 0 };
        let at = y[each.page as usize];
        let name = own(&config::message(&each.name), &[]);
        if each.kind == CHECK {
            add(
                seen | WS_TABSTOP | WS_GROUP | BS_AUTOCHECKBOX | BS_NOTIFY,
                0,
                LEFT,
                at,
                WIDE,
                12,
                each.ident,
                ATOM_BUTTON.into(),
                &name,
            )?;
            y[each.page as usize] = at + 16;
            continue;
        }
        // The label immediately before its control, which is how Win32 says which static names
        // which control (`ux.dialog`).
        add(
            seen | SS_LEFT,
            0,
            LEFT,
            at,
            WIDE,
            9,
            each.ident + 1,
            ATOM_STATIC.into(),
            &name,
        )?;
        if each.kind == CHOICE {
            add(
                seen | WS_TABSTOP | WS_GROUP | WS_VSCROLL | CBS_DROPDOWNLIST,
                0,
                LEFT,
                at + 10,
                WIDE,
                100,
                each.ident,
                ATOM_COMBOBOX.into(),
                "",
            )?;
        } else if each.kind == FOLDER {
            add(
                seen | WS_TABSTOP | WS_GROUP | WS_BORDER | ES_AUTOHSCROLL,
                WS_EX_CLIENTEDGE,
                LEFT,
                at + 10,
                WIDE - 57,
                14,
                each.ident,
                ATOM_EDIT.into(),
                "",
            )?;
            add(
                seen | WS_TABSTOP,
                0,
                LEFT + WIDE - 53,
                at + 10,
                53,
                14,
                each.ident + 2,
                ATOM_BUTTON.into(),
                &own("settings-browse", &[]),
            )?;
        } else {
            add(
                seen | WS_TABSTOP | WS_GROUP | WS_BORDER | ES_NUMBER,
                WS_EX_CLIENTEDGE,
                LEFT,
                at + 10,
                50,
                14,
                each.ident,
                ATOM_EDIT.into(),
                "",
            )?;
        }
        y[each.page as usize] = at + 28;
    }
    let buttons = TOP + PAGE_HEIGHT + 7;
    for (n, (ident, message)) in [
        (IDOK, "dialog-ok"),
        (IDCANCEL, "dialog-cancel"),
        (IDC_APPLY, "dialog-apply"),
    ]
    .into_iter()
    .enumerate()
    {
        let n = n as i64;
        add(
            WS_VISIBLE
                | WS_TABSTOP
                | if n == 0 {
                    BS_DEFPUSHBUTTON | WS_GROUP
                } else {
                    0
                },
            0,
            WIDTH - 7 - 54 * (3 - n),
            buttons,
            50,
            14,
            ident,
            ATOM_BUTTON.into(),
            &own(message, &[]),
        )?;
    }
    frame(
        DS_MODALFRAME | WS_SYSMENU,
        0,
        count,
        WIDTH,
        HEIGHT,
        &own("settings-title", &[]),
        &items,
    )
}

pub const TCM_INSERTITEMW: i64 = 0x133E;
pub const TCM_GETCURSEL: i64 = 0x130B;
pub const TCM_SETCURSEL: i64 = 0x130C;
pub const TCIF_TEXT: i64 = 0x0001;
/// `TCN_FIRST - 1`, -551, as the notification's unsigned code.
pub const TCN_SELCHANGE: i64 = 0xFFFFFDD9;
pub const WM_NOTIFY: i64 = 0x004E;
pub const WM_NEXTDLGCTL: i64 = 0x0028;
pub const WM_KEYDOWN: i64 = 0x0100;
pub const WH_MSGFILTER: i64 = -1;
pub const MSGF_DIALOGBOX: i64 = 0;
pub const VK_TAB: i64 = 0x09;
pub const VK_SHIFT: i64 = 0x10;
pub const VK_CONTROL: i64 = 0x11;
pub const ICC_TAB_CLASSES: i64 = 0x0008;
pub const CB_ADDSTRING: i64 = 0x0143;
pub const CB_GETCURSEL: i64 = 0x0147;
pub const CB_SETCURSEL: i64 = 0x014E;
pub const UDM_SETBUDDY: i64 = 0x0469;
pub const UDM_SETRANGE32: i64 = 0x046F;
pub const UDM_SETPOS32: i64 = 0x0471;
pub const UDS_SETBUDDYINT: i64 = 0x0002;
pub const UDS_ALIGNRIGHT: i64 = 0x0004;
pub const UDS_ARROWKEYS: i64 = 0x0020;
pub const UDS_NOTHOUSANDS: i64 = 0x0080;
pub const ICC_UPDOWN_CLASS: i64 = 0x0010;
pub const MB_OK: i64 = 0x0;
pub const MB_YESNO: i64 = 0x4;
pub const MB_ICONQUESTION: i64 = 0x20;
pub const MB_ICONWARNING: i64 = 0x30;
pub const IDYES: i64 = 6;
pub const BN_SETFOCUS: i64 = 6;
pub const CBN_SETFOCUS: i64 = 3;
pub const EN_SETFOCUS: i64 = 0x0100;
/// How each kind of control says it has the focus.
pub const FOCUSED: [i64; 3] = [BN_SETFOCUS, CBN_SETFOCUS, EN_SETFOCUS];
pub const SW_HIDE: i64 = 0;
pub const SW_SHOW: i64 = 5;
pub const BUFFER: usize = 1024;
pub const WM_TIMER: i64 = 0x0113;
/// The timer a setting's description waits on.
pub const DESCRIBE: usize = 1;
/// How long it waits, in milliseconds, for the screen reader to start on the control.
pub const DESCRIBE_AFTER: u32 = 100;

/// How the dialog speaks a setting's description, without interrupting, and starts pa3ds again
/// for settings applied to take. Python's are any callables; these live on the window's thread.
pub type Say = Rc<dyn Fn(&str)>;
pub type Restart = Rc<dyn Fn() -> Result<()>>;

#[cfg(windows)]
mod win {
    use std::cell::{Cell, RefCell};
    use std::path::{Path, PathBuf};
    use std::ptr::{null, null_mut};
    use std::rc::Rc;

    use indexmap::IndexMap;
    use windows_sys::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::Controls::{
        CheckDlgButton, INITCOMMONCONTROLSEX, InitCommonControlsEx, IsDlgButtonChecked, NMHDR,
        TCITEMW,
    };
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, CreateWindowExW, DLGTEMPLATE, DialogBoxIndirectParamW, EndDialog,
        GetDlgItem, GetDlgItemTextW, KillTimer, MSG, MessageBoxW, SendMessageW, SetDlgItemTextW,
        SetForegroundWindow, SetTimer, SetWindowsHookExW, ShowWindow, UnhookWindowsHookEx,
    };

    use super::*;
    use crate::core::words;
    use crate::py::{Error, Kind};
    use crate::ux::dialog::{BST_CHECKED, WM_COMMAND, WM_INITDIALOG};
    use crate::ux::folder;

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// What Python's `show` closes over, for its dialog procedure and its hook, which are bare
    /// functions here.
    struct Showing {
        every: Vec<Field>,
        lists: IndexMap<String, Vec<(String, String)>>,
        before: RefCell<IndexMap<String, Value>>,
        /// Each page's windows, labels, arrows and Browse among them, to show and hide.
        controls: RefCell<IndexMap<i64, Vec<isize>>>,
        about: IndexMap<i64, String>,
        say: Say,
        restart: Restart,
        home: Option<PathBuf>,
        /// The dialog, once open, for Ctrl+Tab's hook.
        dialog: Cell<isize>,
        /// The description the timer will say.
        waiting: RefCell<String>,
    }

    thread_local! {
        static SHOWING: RefCell<Option<Rc<Showing>>> = const { RefCell::new(None) };
    }

    /// The dialog open on this thread, cloned out so that nothing is borrowed while Windows
    /// calls back into the procedure.
    fn showing() -> Option<Rc<Showing>> {
        SHOWING.with(|showing| showing.borrow().clone())
    }

    /// `ctypes.WinError(ctypes.get_last_error())`, as `dialog::ask` raises it.
    fn win_error() -> Error {
        // SAFETY: GetLastError takes no arguments.
        let code = unsafe { GetLastError() };
        let text = std::io::Error::from_raw_os_error(code as i32).to_string();
        let text = text
            .strip_suffix(&format!(" (os error {code})"))
            .unwrap_or(&text);
        Error::new(Kind::OsError, format!("[WinError {code}] {text}"))
    }

    fn item_text(window: HWND, ident: i64) -> String {
        let mut buffer = vec![0u16; BUFFER];
        // SAFETY: `buffer` has room for BUFFER UTF-16 units, which is what is passed.
        unsafe { GetDlgItemTextW(window, ident as i32, buffer.as_mut_ptr(), BUFFER as i32) };
        let end = buffer
            .iter()
            .position(|&unit| unit == 0)
            .unwrap_or(buffer.len());
        String::from_utf16_lossy(&buffer[..end])
    }

    fn set_text(window: HWND, ident: i64, text: &str) {
        // SAFETY: the text is NUL-terminated and outlives the call.
        unsafe { SetDlgItemTextW(window, ident as i32, wide(text).as_ptr()) };
    }

    fn send(window: HWND, message: i64, wparam: usize, lparam: isize) -> isize {
        // SAFETY: SendMessageW takes any window, and fails for one that is gone.
        unsafe { SendMessageW(window, message as u32, wparam, lparam) }
    }

    fn message_box(window: HWND, text: &str, title: &str, style: i64) -> i64 {
        // SAFETY: both strings are NUL-terminated and outlive the call.
        unsafe {
            MessageBoxW(
                window,
                wide(text).as_ptr(),
                wide(title).as_ptr(),
                style as u32,
            ) as i64
        }
    }

    fn dialog_item(window: HWND, ident: i64) -> HWND {
        // SAFETY: GetDlgItem only reads the handles.
        unsafe { GetDlgItem(window, ident as i32) }
    }

    fn read(showing: &Showing, window: HWND) -> IndexMap<String, Value> {
        let before = showing.before.borrow().clone();
        let mut out: IndexMap<String, Value> = IndexMap::new();
        for each in &showing.every {
            let was = before[&each.name].clone();
            let value = if each.kind == CHECK {
                // SAFETY: `window` is the dialog, and the id one of its controls.
                Value::Bool(
                    unsafe { IsDlgButtonChecked(window, each.ident as i32) } as i64 == BST_CHECKED,
                )
            } else if each.kind == CHOICE {
                let chosen = send(dialog_item(window, each.ident), CB_GETCURSEL, 0, 0) as i64;
                let listed = &showing.lists[&each.name];
                if 0 <= chosen && (chosen as usize) < listed.len() {
                    Value::Str(listed[chosen as usize].0.clone())
                } else {
                    was
                }
            } else {
                let text = item_text(window, each.ident);
                if each.kind == FOLDER {
                    Value::Str(py::strip(&text).to_string())
                } else {
                    typed(each.kind, &text, &was)
                }
            };
            out.insert(each.name.clone(), value);
        }
        out
    }

    fn apply(showing: &Showing, window: HWND) -> Result<()> {
        let after = read(showing, window);
        let changes = changed(&showing.before.borrow(), &after);
        if changes.is_empty() {
            return Ok(());
        }
        let home = showing.home.as_deref();
        let listed: Vec<(String, Value)> = changes.clone().into_iter().collect();
        if !config::change(&listed, home)? {
            let path = py::path_str(&config::config_path(home)?);
            message_box(
                window,
                &own("settings-unwritten", &[("path", &path)]),
                &own("settings-title", &[]),
                MB_OK | MB_ICONWARNING,
            );
            return Ok(());
        }
        showing.before.borrow_mut().extend(changes);
        if message_box(
            window,
            &own("settings-restart", &[]),
            &own("settings-title", &[]),
            MB_YESNO | MB_ICONQUESTION,
        ) == IDYES
        {
            (showing.restart)()?;
        }
        Ok(())
    }

    fn page(showing: &Showing, chosen: i64) {
        let controls = showing.controls.borrow().clone();
        for (number, windows) in &controls {
            for each in windows {
                let shown = if *number == chosen { SW_SHOW } else { SW_HIDE };
                // SAFETY: ShowWindow takes any window, and fails for one that is gone.
                unsafe { ShowWindow(*each as HWND, shown as i32) };
            }
        }
    }

    /// Ctrl+Tab: the next page, or the one before, and the focus on its tab, as a
    /// property sheet has it, for the screen reader to say which.
    fn turn(showing: &Showing, window: HWND, by: i64) {
        let tabs = dialog_item(window, IDC_PAGES);
        let chosen = py::modulo(
            send(tabs, TCM_GETCURSEL, 0, 0) as i64 + by,
            PAGES.len() as i64,
        );
        send(tabs, TCM_SETCURSEL, chosen as usize, 0);
        page(showing, chosen);
        send(window, WM_NEXTDLGCTL, tabs as usize, 1);
    }

    fn start(showing: &Showing, window: HWND) -> Result<()> {
        let tabs = dialog_item(window, IDC_PAGES);
        for (n, (message, _table)) in PAGES.iter().enumerate() {
            let mut said = wide(&own(message, &[]));
            let tab = TCITEMW {
                mask: TCIF_TEXT as u32,
                dwState: 0,
                dwStateMask: 0,
                pszText: said.as_mut_ptr(),
                cchTextMax: 0,
                iImage: 0,
                lParam: 0,
            };
            send(tabs, TCM_INSERTITEMW, n, &tab as *const TCITEMW as isize);
        }
        showing.dialog.set(window as isize);
        let before = showing.before.borrow().clone();
        for each in &showing.every {
            let control = dialog_item(window, each.ident);
            let mut windows = vec![control];
            if each.kind == CHECK {
                let checked = if before[&each.name] == Value::Bool(true) {
                    BST_CHECKED
                } else {
                    0
                };
                // SAFETY: `window` is the dialog, and the id one of its controls.
                unsafe { CheckDlgButton(window, each.ident as i32, checked as u32) };
            } else if each.kind == CHOICE {
                for (_code, said) in &showing.lists[&each.name] {
                    let said = wide(said);
                    send(control, CB_ADDSTRING, 0, said.as_ptr() as isize);
                }
                let at = showing.lists[&each.name]
                    .iter()
                    .position(|(code, _said)| before[&each.name] == Value::Str(code.clone()))
                    .unwrap_or(0);
                send(control, CB_SETCURSEL, at, 0);
            } else {
                set_text(window, each.ident, &shown(each.kind, &before[&each.name])?);
            }
            if let Some((_, (low, high))) = RANGES.iter().find(|(kind, _)| *kind == each.kind) {
                let seen = if each.page == 0 { WS_VISIBLE } else { 0 };
                let class = wide("msctls_updown32");
                // SAFETY: the class name is NUL-terminated and outlives the call, and the
                // arrows are a child of the dialog, which owns them from here on.
                let arrows = unsafe {
                    CreateWindowExW(
                        0,
                        class.as_ptr(),
                        null(),
                        (WS_CHILD
                            | UDS_SETBUDDYINT
                            | UDS_ALIGNRIGHT
                            | UDS_ARROWKEYS
                            | UDS_NOTHOUSANDS
                            | seen) as u32,
                        0,
                        0,
                        0,
                        0,
                        window,
                        (each.ident + 3) as usize as _,
                        null_mut(),
                        null(),
                    )
                };
                if !arrows.is_null() {
                    send(arrows, UDM_SETBUDDY, control as usize, 0);
                    send(arrows, UDM_SETRANGE32, *low as usize, *high as isize);
                    windows.push(arrows);
                }
            }
            if each.kind != CHECK {
                windows.push(dialog_item(window, each.ident + 1));
            }
            if each.kind == FOLDER {
                windows.push(dialog_item(window, each.ident + 2));
            }
            showing
                .controls
                .borrow_mut()
                .entry(each.page)
                .or_default()
                .extend(
                    windows
                        .into_iter()
                        .filter(|w| !w.is_null())
                        .map(|w| w as isize),
                );
        }
        // SAFETY: SetForegroundWindow only reads the window handle.
        unsafe { SetForegroundWindow(window) };
        Ok(())
    }

    fn browse(window: HWND, each: &Field) -> Result<()> {
        let text = item_text(window, each.ident);
        let typed_in = if py::strip(&text).is_empty() {
            None
        } else {
            Some(py::to_path(py::strip(&text)))
        };
        let initial = match typed_in {
            Some(typed_in) if typed_in.is_dir() => typed_in,
            _ => {
                let app = keyfile::app_path()?;
                app.parent().map(Path::to_path_buf).unwrap_or(app)
            }
        };
        let mut notes: Vec<String> = Vec::new();
        if let Some(chosen) = folder::choose(
            &own(&config::message(&each.name), &[]),
            &initial,
            &mut notes,
        ) {
            set_text(window, each.ident, &py::path_str(&chosen));
        }
        Ok(())
    }

    fn handle(
        showing: &Showing,
        window: HWND,
        message: i64,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> Result<isize> {
        if message == WM_INITDIALOG {
            start(showing, window)?;
            return Ok(1);
        }
        if message == WM_NOTIFY && lparam != 0 {
            // SAFETY: a WM_NOTIFY's lparam is the NMHDR of the control notifying.
            let header = unsafe { &*(lparam as *const NMHDR) };
            if header.idFrom as i64 == IDC_PAGES && header.code as i64 == TCN_SELCHANGE {
                page(showing, send(header.hwndFrom, TCM_GETCURSEL, 0, 0) as i64);
            }
            return Ok(0);
        }
        if message == WM_TIMER && wparam == DESCRIBE {
            // SAFETY: `window` is the dialog the timer was set on.
            unsafe { KillTimer(window, DESCRIBE) };
            let waiting = showing.waiting.borrow().clone();
            (showing.say)(&waiting);
            return Ok(1);
        }
        if message == WM_COMMAND {
            let (command, note) = (wparam as i64 & 0xFFFF, wparam as i64 >> 16 & 0xFFFF);
            if FOCUSED.contains(&note)
                && let Some(about) = showing.about.get(&command)
            {
                // Queued behind what the screen reader has started to say of the control; a
                // control left before the timer runs out is not described.
                *showing.waiting.borrow_mut() = about.clone();
                // SAFETY: `window` is the dialog; a timer with no procedure posts WM_TIMER to it.
                unsafe { SetTimer(window, DESCRIBE, DESCRIBE_AFTER, None) };
                return Ok(0);
            }
            if command == IDOK || command == IDC_APPLY {
                apply(showing, window)?;
                if command == IDOK {
                    // SAFETY: `window` is the dialog this procedure was called for.
                    unsafe { EndDialog(window, IDOK as isize) };
                }
                return Ok(1);
            }
            if command == IDCANCEL {
                // SAFETY: as above.
                unsafe { EndDialog(window, IDCANCEL as isize) };
                return Ok(1);
            }
            if let Some(each) = showing
                .every
                .iter()
                .find(|each| each.kind == FOLDER && command == each.ident + 2)
            {
                browse(window, each)?;
                return Ok(1);
            }
        }
        Ok(0)
    }

    unsafe extern "system" fn proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> isize {
        let Some(showing) = showing() else {
            return 0;
        };
        match handle(&showing, window, message as i64, wparam, lparam) {
            Ok(answer) => answer,
            Err(error) => {
                // What ctypes writes for a callback that raised, which then answers 0.
                eprintln!(
                    "Exception ignored on calling ctypes callback function:\n{}: {}",
                    error.kind.name(),
                    error.message
                );
                0
            }
        }
    }

    unsafe extern "system" fn keys(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // Every message the dialog's own loop takes passes here first (`WH_MSGFILTER`):
        // Ctrl+Tab is taken for the pages before Tab would move the focus.
        if let Some(showing) = showing()
            && code as i64 == MSGF_DIALOGBOX
            && lparam != 0
            && showing.dialog.get() != 0
        {
            // SAFETY: a WH_MSGFILTER hook's lparam is the MSG being filtered.
            let message = unsafe { &*(lparam as *const MSG) };
            // SAFETY: GetKeyState takes a key and reads this thread's keyboard state.
            if message.message as i64 == WM_KEYDOWN
                && message.wParam as i64 == VK_TAB
                && unsafe { GetKeyState(VK_CONTROL as i32) } < 0
            {
                let by = if unsafe { GetKeyState(VK_SHIFT as i32) } < 0 {
                    -1
                } else {
                    1
                };
                turn(&showing, showing.dialog.get() as HWND, by);
                return 1;
            }
        }
        // SAFETY: the hook's own arguments, handed on.
        unsafe { CallNextHookEx(null_mut(), code, wparam, lparam) }
    }

    pub fn show(owner: isize, say: Say, restart: Restart, home: Option<&Path>) -> Result<()> {
        // The words in place are a thread's own, and this is the window's: the words the
        // session put in place are put here too, as Python's are the one module's.
        words::r#use(words::placed_last()?);
        let every = fields();
        let mut lists: IndexMap<String, Vec<(String, String)>> = IndexMap::new();
        for each in every.iter().filter(|each| each.kind == CHOICE) {
            lists.insert(each.name.clone(), choices(&each.name)?);
        }
        let before = values(&config::prepare(home, None, false, true)?);
        let about = every
            .iter()
            .map(|each| {
                (
                    each.ident,
                    own(&format!("{}-about", config::message(&each.name)), &[]),
                )
            })
            .collect();
        SHOWING.with(|slot| {
            *slot.borrow_mut() = Some(Rc::new(Showing {
                every,
                lists,
                before: RefCell::new(before),
                controls: RefCell::new(IndexMap::new()),
                about,
                say,
                restart,
                home: home.map(Path::to_path_buf),
                dialog: Cell::new(0),
                waiting: RefCell::new(String::new()),
            }))
        });

        let controls_init = INITCOMMONCONTROLSEX {
            dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: (ICC_UPDOWN_CLASS | ICC_TAB_CLASSES) as u32,
        };
        // SAFETY: the structure is filled in and outlives the call.
        unsafe { InitCommonControlsEx(&controls_init) };
        let built = template()?;
        // The template must start on a 4-byte boundary, as in `dialog::ask`.
        let mut held = vec![0u32; built.len().div_ceil(4)];
        for (word, bytes) in held.iter_mut().zip(built.chunks(4)) {
            let mut padded = [0u8; 4];
            padded[..bytes.len()].copy_from_slice(bytes);
            *word = u32::from_le_bytes(padded);
        }
        // SAFETY: `keys` has the signature of a hook procedure, hooked on this thread alone.
        let hook = unsafe {
            SetWindowsHookExW(
                WH_MSGFILTER as i32,
                Some(keys),
                null_mut(),
                GetCurrentThreadId(),
            )
        };
        // SAFETY: `held` is a DLGTEMPLATE as `template` lays one out, DWORD-aligned and alive
        // for the call, and `proc` has the signature of a dialog procedure.
        let shown = unsafe {
            DialogBoxIndirectParamW(
                null_mut(),
                held.as_ptr() as *const DLGTEMPLATE,
                owner as HWND,
                Some(proc),
                0,
            )
        };
        let failed = if shown < 0 { Some(win_error()) } else { None };
        if !hook.is_null() {
            // SAFETY: `hook` is the hook set above.
            unsafe { UnhookWindowsHookEx(hook) };
        }
        SHOWING.with(|slot| *slot.borrow_mut() = None);
        match failed {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

/// Open the dialog over `owner`, pa3ds's window, and wait for it to close: modal, on the
/// window's thread, whose loop runs meanwhile. `say` speaks a setting's description without
/// interrupting; `restart` starts pa3ds again, for settings applied to take. Python's `home`
/// defaults to None.
#[cfg(windows)]
pub fn show(
    owner: isize,
    say: Say,
    restart: Restart,
    home: Option<&std::path::Path>,
) -> Result<()> {
    win::show(owner, say, restart, home)
}

// The runtime is Windows; this keeps the import working elsewhere.

#[cfg(not(windows))]
pub fn show(
    _owner: isize,
    _say: Say,
    _restart: Restart,
    _home: Option<&std::path::Path>,
) -> Result<()> {
    Err(crate::py::Error::new(
        crate::py::Kind::OsError,
        "a dialog needs Windows",
    ))
}
