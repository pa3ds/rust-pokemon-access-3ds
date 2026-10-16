//! Hotkeys: the keys pa3ds answers while the emulator's window is in front.
//!
//! A low-level keyboard hook (`WH_KEYBOARD_LL`) sees every key before the window it is going to,
//! and can keep it from getting there. pa3ds keeps the keys it is bound to when the foreground
//! window is the emulator's, so a key that cycles the map's interactables never reaches the game as
//! a button: the user's Azahar has I, J, K and L on the C-stick and O on the debug button, and
//! rebinding the emulator is designed but unbuilt (ADR-0022). Anywhere else in front, every key
//! passes through untouched, so nothing typed elsewhere is ever swallowed.
//!
//! The hook runs on a thread of its own with the message loop Windows requires of it; the runtime
//! takes the keys pressed at each poll (`pressed`), by name. Nothing here knows what a key means:
//! the same key means one thing in one navigation mode and another in the other, so the lookup is
//! the navigator's (`common.field.navigate`).
//!
//! A name may carry modifiers — `shift+k` — because one key is not enough for both reading a thing
//! and renaming it. Which keys pa3ds is given, and which buttons the emulator is left, is
//! `pa3ds.core.keyfile`'s; this module only turns a key press into the name that was bound.

use std::collections::HashSet;

#[cfg(windows)]
use std::cell::RefCell;
#[cfg(windows)]
use std::collections::VecDeque;
#[cfg(windows)]
use std::ptr::null_mut;
#[cfg(windows)]
use std::sync::atomic::{AtomicU32, Ordering};
#[cfg(windows)]
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
#[cfg(windows)]
use std::time::Duration;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{GetLastError, LPARAM, LRESULT, WPARAM};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
#[cfg(windows)]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, MSG, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
};

use crate::core::window;
#[cfg(windows)]
use crate::py::{Error, Kind, Result};

pub const WH_KEYBOARD_LL: i64 = 13;
pub const WM_KEYDOWN: i64 = 0x0100;
pub const WM_SYSKEYDOWN: i64 = 0x0104;
pub const WM_KEYUP: i64 = 0x0101;
pub const WM_SYSKEYUP: i64 = 0x0105;
pub const WM_QUIT: i64 = 0x0012;

pub const VK_SHIFT: i64 = 0x10;
pub const VK_CONTROL: i64 = 0x11;
/// Alt, which Windows still calls the menu key.
pub const VK_MENU: i64 = 0x12;
/// Insert and caps lock, which NVDA takes as its own modifier, and the two Windows keys: a key
/// pressed while one is held is the screen reader's or Windows's. Insert and R is NVDA's OCR, and
/// pa3ds took it for R and said the battle's side (Orion, 2026-09-29). Whether one is held is asked
/// of Windows and of pa3ds's own hook (`track`): NVDA swallows its modifier in its own hook, so
/// Windows never marks it held, but pa3ds's hook, installed after NVDA's, is called first and sees it
/// go down and up. Asking Windows alone kept the Windows keys only, and insert and R still reached
/// pa3ds as R.
pub const ELSEWHERE: [i64; 4] = [0x2D, 0x14, 0x5B, 0x5C];
/// The bit `GetAsyncKeyState` sets while a key is held.
pub const DOWN: i64 = 0x8000;

/// The process whose window the keys are answered in front of.
pub const EMULATOR: &str = window::EMULATOR;

/// Keys named by a word, as the keys file writes them, so Enter can be bound.
pub const NAMED: [(i64, &str); 1] = [(0x0D, "enter")];

/// The OEM virtual keys by the character each carries on a US layout. Windows numbers these
/// keys by their place, not their character, so on another layout the key in the semicolon's place
/// is still `;` here.
pub const PUNCTUATION: [(i64, &str); 11] = [
    (0xBA, ";"),
    (0xBB, "="),
    (0xBC, ","),
    (0xBD, "-"),
    (0xBE, "."),
    (0xBF, "/"),
    (0xC0, "`"),
    (0xDB, "["),
    (0xDC, "\\"),
    (0xDD, "]"),
    (0xDE, "'"),
];

/// A virtual-key code as the key's name: letters, digits and US-layout punctuation as
/// themselves, letters in lower case.
pub fn key_name(virtual_key: i64) -> String {
    if (0x30..=0x39).contains(&virtual_key) || (0x41..=0x5A).contains(&virtual_key) {
        return (virtual_key as u8 as char).to_lowercase().to_string();
    }
    if let Some((_, character)) = PUNCTUATION.iter().find(|(key, _)| *key == virtual_key) {
        return character.to_string();
    }
    if let Some((_, name)) = NAMED.iter().find(|(key, _)| *key == virtual_key) {
        return name.to_string();
    }
    format!("vk{virtual_key:#x}")
}

pub const SHIFT: &str = "shift";
pub const CTRL: &str = "ctrl";
pub const ALT: &str = "alt";
/// The order modifiers are written in, so one combination has one name. Windows' own order.
pub const ORDER: [&str; 3] = [CTRL, ALT, SHIFT];

/// A key's name with the modifiers held in front of it: `shift+k`, `ctrl+alt+shift+k`.
pub fn modified(name: &str, ctrl: bool, alt: bool, shift: bool) -> String {
    let held = |one: &str| match one {
        CTRL => ctrl,
        ALT => alt,
        _ => shift,
    };
    let front: String = ORDER
        .iter()
        .filter(|one| held(one))
        .map(|one| format!("{one}+"))
        .collect();
    format!("{front}{name}")
}

/// Keep in `held` which keys of `ELSEWHERE` the hook has seen go down and not yet come up.
///
/// A release the hook never sees, as on the secure desktop that Ctrl+Alt+Del opens, leaves the
/// key held and pa3ds's keys dead until that key is pressed and released again.
pub fn track(held: &mut HashSet<i64>, virtual_key: i64, message: i64) {
    if !ELSEWHERE.contains(&virtual_key) {
        return;
    }
    if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
        held.insert(virtual_key);
    } else if message == WM_KEYUP || message == WM_SYSKEYUP {
        held.remove(&virtual_key);
    }
}

/// The name to answer for a press, or "" for one that is nobody's.
///
/// Shift falls back: with only `k` bound, shift and k together still read the item, because a
/// player holding shift from the last thing they typed should not find the key dead. **Ctrl and
/// alt never fall back**, since ctrl and a letter belongs to whatever is in front — ctrl+c must
/// reach the emulator even when `c` is pa3ds's. **Nothing is pa3ds's with a key of `ELSEWHERE`
/// held**, the screen reader's modifier or Windows's.
pub fn bound<S: AsRef<str>>(
    name: &str,
    keys: impl IntoIterator<Item = S> + Clone,
    ctrl: bool,
    alt: bool,
    elsewhere: bool,
) -> String {
    if elsewhere {
        return String::new();
    }
    let has = |wanted: &str| keys.clone().into_iter().any(|key| key.as_ref() == wanted);
    if has(name) {
        return name.to_string();
    }
    if ctrl || alt {
        return String::new();
    }
    let bare = name.strip_prefix(&format!("{SHIFT}+")).unwrap_or(name);
    if has(bare) {
        bare.to_string()
    } else {
        String::new()
    }
}

/// Whether a modifier is down *now*. `GetAsyncKeyState` reads the physical keyboard, so
/// it is right inside a hook; `GetKeyState` reads the calling thread's own idea of the
/// keyboard, and the hook's thread has no window and so no input of its own to shape it.
#[cfg(windows)]
fn held(virtual_key: i64) -> bool {
    // SAFETY: GetAsyncKeyState only reads its argument.
    (unsafe { GetAsyncKeyState(virtual_key as i32) } as i64 & DOWN) != 0
}

#[cfg(windows)]
#[repr(C)]
#[allow(non_snake_case)]
struct KeyboardHookStruct {
    vkCode: u32,
    scanCode: u32,
    flags: u32,
    time: u32,
    dwExtraInfo: *mut u32,
}

/// `threading.Event`: a flag one thread sets and another waits on, for as long as it likes.
#[cfg(windows)]
#[derive(Default)]
struct Event {
    set: Mutex<bool>,
    changed: Condvar,
}

#[cfg(windows)]
impl Event {
    fn set(&self) {
        *lock(&self.set) = true;
        self.changed.notify_all();
    }

    /// Wait up to `timeout` seconds; whether the flag is set.
    fn wait(&self, timeout: f64) -> bool {
        let set = lock(&self.set);
        let (set, _) = self
            .changed
            .wait_timeout_while(set, Duration::from_secs_f64(timeout), |set| !*set)
            .unwrap_or_else(PoisonError::into_inner);
        *set
    }
}

/// `threading.Thread`, as far as `Hotkeys` uses one: started, and joined with a timeout. A
/// `JoinHandle` cannot be joined with one, so the thread says when it is over.
#[cfg(windows)]
struct Thread {
    over: Arc<Event>,
}

#[cfg(windows)]
impl Thread {
    fn start(name: &str, target: impl FnOnce() + Send + 'static) -> Result<Thread> {
        struct Finish(Arc<Event>);
        impl Drop for Finish {
            fn drop(&mut self) {
                self.0.set();
            }
        }

        let over = Arc::new(Event::default());
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

    fn join(&self, timeout: f64) {
        self.over.wait(timeout);
    }
}

#[cfg(windows)]
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the hook thread and the thread that started it share.
#[cfg(windows)]
struct State {
    keys: HashSet<String>,
    emulator: String,
    pressed: Mutex<VecDeque<String>>,
    elsewhere: Mutex<HashSet<i64>>,
    thread_id: AtomicU32,
    ready: Event,
    error: Mutex<String>,
}

#[cfg(windows)]
thread_local! {
    /// The `State` whose hook this thread runs. A hook procedure is a bare function that Windows
    /// calls on the thread that installed the hook, with nothing to say which `Hotkeys` it is
    /// for, where Python gave Windows a bound method.
    static HOOKED: RefCell<Option<Arc<State>>> = const { RefCell::new(None) };
}

#[cfg(windows)]
impl State {
    fn run(self: &Arc<Self>) {
        self.thread_id.store(
            // SAFETY: GetCurrentThreadId takes no arguments.
            unsafe { GetCurrentThreadId() },
            Ordering::SeqCst,
        );
        HOOKED.with(|hooked| *hooked.borrow_mut() = Some(self.clone()));
        // SAFETY: `hook_proc` has the signature of a hook procedure and is never unloaded.
        let hook =
            unsafe { SetWindowsHookExW(WH_KEYBOARD_LL as i32, Some(hook_proc), null_mut(), 0) };
        if hook.is_null() {
            // SAFETY: GetLastError takes no arguments.
            let last_error = unsafe { GetLastError() };
            *lock(&self.error) = format!("keyboard hook refused: error {last_error}");
            self.ready.set();
            return;
        }
        self.ready.set();
        // SAFETY: MSG is plain data that GetMessageW fills in.
        let mut message: MSG = unsafe { std::mem::zeroed() };
        // SAFETY: `message` is live for each call, and a null window takes this thread's messages.
        while unsafe { GetMessageW(&mut message, null_mut(), 0, 0) } > 0 {}
        // SAFETY: `hook` was returned by SetWindowsHookExW above and is removed once.
        unsafe { UnhookWindowsHookEx(hook) };
    }

    fn hook(&self, code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            // SAFETY: for a code of 0 or more, a WH_KEYBOARD_LL hook's lParam points at a
            // KBDLLHOOKSTRUCT that lives until the hook returns.
            let event = unsafe { &*(lparam as *const KeyboardHookStruct) };
            let message = wparam as i64;
            track(&mut lock(&self.elsewhere), event.vkCode as i64, message);
            if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
                let (ctrl, alt) = (held(VK_CONTROL), held(VK_MENU));
                let name = modified(&key_name(event.vkCode as i64), ctrl, alt, held(VK_SHIFT));
                let elsewhere =
                    !lock(&self.elsewhere).is_empty() || ELSEWHERE.iter().any(|&key| held(key));
                let key = bound(&name, &self.keys, ctrl, alt, elsewhere);
                if !key.is_empty() && window::in_front(&self.emulator) {
                    lock(&self.pressed).push_back(key);
                    return 1;
                }
            }
        }
        // SAFETY: the arguments are the ones this hook was called with.
        unsafe { CallNextHookEx(null_mut(), code, wparam, lparam) }
    }
}

/// Windows' call into the hook: on to the `State` this thread installed it for.
#[cfg(windows)]
unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let state = HOOKED.with(|hooked| hooked.borrow().clone());
    match state {
        Some(state) => state.hook(code, wparam, lparam),
        // SAFETY: the arguments are the ones this hook was called with.
        None => unsafe { CallNextHookEx(null_mut(), code, wparam, lparam) },
    }
}

/// Keys taken, by name, only while the emulator's window is in front.
#[cfg(windows)]
pub struct Hotkeys {
    state: Arc<State>,
    thread: Option<Thread>,
}

#[cfg(windows)]
impl Hotkeys {
    pub fn new<S: AsRef<str>>(keys: impl IntoIterator<Item = S>, emulator: &str) -> Self {
        // Python keeps the hook's ctypes callback on the object, because ctypes frees it with the
        // last reference and the hook would call freed code; a Rust function is never freed.
        Hotkeys {
            state: Arc::new(State {
                keys: keys
                    .into_iter()
                    .map(|key| key.as_ref().to_lowercase())
                    .collect(),
                emulator: emulator.to_string(),
                pressed: Mutex::new(VecDeque::new()),
                elsewhere: Mutex::new(HashSet::new()),
                thread_id: AtomicU32::new(0),
                ready: Event::default(),
                error: Mutex::new(String::new()),
            }),
            thread: None,
        }
    }

    /// Hotkeys for the emulator pa3ds is made for.
    pub fn of<S: AsRef<str>>(keys: impl IntoIterator<Item = S>) -> Self {
        Hotkeys::new(keys, EMULATOR)
    }

    /// Install the hook on a thread of its own. Returns why it could not, or "".
    pub fn start(&mut self) -> Result<String> {
        let state = self.state.clone();
        self.thread = Some(Thread::start("hotkeys", move || state.run())?);
        self.state.ready.wait(5.0);
        Ok(lock(&self.state.error).clone())
    }

    pub fn stop(&self) {
        let thread_id = self.state.thread_id.load(Ordering::SeqCst);
        if thread_id != 0 {
            // SAFETY: PostThreadMessageW only reads its arguments; a thread that has ended is a
            // failed call.
            unsafe { PostThreadMessageW(thread_id, WM_QUIT as u32, 0, 0) };
        }
        if let Some(thread) = &self.thread {
            thread.join(2.0);
        }
    }

    /// The keys pressed since the last call, in order.
    pub fn pressed(&self) -> Vec<String> {
        lock(&self.state.pressed).drain(..).collect()
    }
}
