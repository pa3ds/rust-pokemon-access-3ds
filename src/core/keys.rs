//! Hotkeys: the keys pa3ds answers while the emulator's window is in front.
//!
//! A low-level keyboard hook (`WH_KEYBOARD_LL`) sees every key before the window it is going to,
//! and can keep it from getting there. pa3ds keeps the keys it is bound to when the foreground
//! window is the emulator's, so a key that cycles the map's interactables never reaches the game as
//! a button: the user's Azahar has I, J, K and L on the C-stick and O on the debug button, and
//! rebinding the emulator is designed but unbuilt (ADR-0022). Anywhere else in front, every key
//! passes through untouched, so nothing typed elsewhere is ever swallowed.
//!
//! macOS has an event tap that does what the hook does, and X11 on Linux has grabs, which come
//! close (`mac`, `x11`). Every platform decides a key as Windows does (`State::key`), by its
//! Windows virtual key, so a key has one name everywhere.
//!
//! The hook runs on a thread of its own with the message loop Windows requires of it; the runtime
//! takes the keys pressed at each poll (`pressed`), by name. Nothing here knows what a key means:
//! the same key means one thing in one navigation mode and another in the other, so the lookup is
//! the navigator's (`common.field.navigate`).
//!
//! A name may carry modifiers — `shift+k` — because one key is not enough for both reading a thing
//! and renaming it. Which keys pa3ds is given, and which buttons the emulator is left, is
//! `pa3ds.core.keyfile`'s; this module only turns a key press into the name that was bound.
//!
//! Bound by:
//! - ADR-0009: a drag held on Shift alone or a key bound to drag is let go as it comes up, in front
//!   of any window.
//! - ADR-0022: pa3ds's keys are taken only while the game's window is in front, and none while the
//!   screen reader's modifier is held.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

#[cfg(windows)]
use std::cell::RefCell;
#[cfg(windows)]
use std::ptr::null_mut;
#[cfg(windows)]
use std::sync::atomic::AtomicU32;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
#[cfg(windows)]
use windows_sys::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
#[cfg(windows)]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, MSG, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
};

use crate::core::window;
use crate::py::{Error, Kind, Result};

pub const WH_KEYBOARD_LL: i64 = 13;
pub const WM_KEYDOWN: i64 = 0x0100;
pub const WM_SYSKEYDOWN: i64 = 0x0104;
pub const WM_KEYUP: i64 = 0x0101;
pub const WM_SYSKEYUP: i64 = 0x0105;
pub const WM_QUIT: i64 = 0x0012;

pub const VK_SHIFT: i64 = 0x10;
/// Shift as a hook sees it: the left one and the right one, and either.
pub const SHIFTS: [i64; 3] = [VK_SHIFT, 0xA0, 0xA1];
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
/// What follows a key's name as it comes up, for a key whose release is said (`Released`).
pub const UP: &str = " up";
/// What is pressed when Shift comes up after a key of pa3ds's taken with it, or after it went down
/// alone: touch mode lets go of a drag held with it (`common.touch`'s drag). Never a key bound.
///
/// `SHIFT + UP` in Python; a const cannot join two here.
pub const SHIFT_UP: &str = "shift up";
/// What is pressed when Shift goes down alone in front of the emulator, once however long it is
/// held, and passed on to the emulator all the same: touch mode drags with it on the screens that
/// drag (ADR-0009 as amended 2026-10-28). Never a key bound.
pub const SHIFT_DOWN: &str = "shift down";
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
/// A release the hook never sees, as while an NVDA dialog opened with Insert+N or the secure
/// desktop that Ctrl+Alt+Del opens has the keyboard, would leave the key held and every key of
/// pa3ds's dead. So `back`, the emulator's window in front again after another window was, forgets
/// every key held (issue 23).
pub fn track(held: &mut HashSet<i64>, virtual_key: i64, message: i64, back: bool) {
    if back {
        held.clear();
    }
    if !ELSEWHERE.contains(&virtual_key) {
        return;
    }
    if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
        held.insert(virtual_key);
    } else if message == WM_KEYUP || message == WM_SYSKEYUP {
        held.remove(&virtual_key);
    }
}

/// The keys whose release is said, `d up`, and whose repeats are not: a key the player binds to
/// touch mode's drag, which lets go as it comes up, and which Windows repeats while it is held,
/// presses that would pick up and let go over and over with `drag` "toggle".
///
/// A release the hook never sees leaves the key held, and its next press is taken for a repeat.
pub struct Released {
    names: HashSet<String>,
    /// The name each key held down was taken as, by its virtual key.
    down: HashMap<i64, String>,
}

impl Released {
    pub fn new<S: AsRef<str>>(names: impl IntoIterator<Item = S>) -> Self {
        Released {
            names: names
                .into_iter()
                .map(|name| name.as_ref().to_lowercase())
                .collect(),
            down: HashMap::new(),
        }
    }

    /// Whether a press taken as `name` is new, rather than Windows repeating one held.
    pub fn down(&mut self, virtual_key: i64, name: &str) -> bool {
        if !self.names.contains(name) {
            return true;
        }
        if self.down.contains_key(&virtual_key) {
            return false;
        }
        self.down.insert(virtual_key, name.to_string());
        true
    }

    /// What is pressed as a key comes up: its name and `UP`, or "" for a key not said.
    pub fn up(&mut self, virtual_key: i64) -> String {
        match self.down.remove(&virtual_key) {
            None => String::new(),
            Some(name) => name + UP,
        }
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
pub const EVENT_SYSTEM_FOREGROUND: i64 = 0x0003;
#[cfg(windows)]
pub const WINEVENT_OUTOFCONTEXT: i64 = 0x0000;

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
#[derive(Default)]
struct Event {
    set: Mutex<bool>,
    changed: Condvar,
}

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
struct Thread {
    over: Arc<Event>,
}

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

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the hook thread and the thread that started it share.
struct State {
    keys: HashSet<String>,
    emulator: String,
    released: Mutex<Released>,
    /// The keys of the screen running now that has its own (`keyfile`'s contexts), taken besides
    /// `keys` until the runtime says another screen runs: a key that is one of Azahar's buttons
    /// stays that button everywhere else (Orion, 2026-10-29).
    contextual: Mutex<HashSet<String>>,
    pressed: Mutex<VecDeque<String>>,
    elsewhere: Mutex<HashSet<i64>>,
    /// Whether the emulator's window was in front at the last key or change of window in front
    /// the hook saw. Keys typed into NVDA's own windows never reach the hook: NVDA runs at a
    /// higher integrity than pa3ds, and Windows calls a hook for no window above its own. So the
    /// change of window is heard too, or a modifier whose release went to an NVDA dialog stays
    /// held after it closes, until Alt+Tab (Khai, 2026-10-31).
    front: AtomicBool,
    /// Whether a key of pa3ds's has been taken with Shift since Shift last came up, or Shift went
    /// down alone in front of the emulator.
    shifted: AtomicBool,
    /// Whether Shift is down: Windows repeats its press while it is held.
    shift_down: AtomicBool,
    #[cfg(windows)]
    thread_id: AtomicU32,
    /// Set by `stop`: the thread's loop ends at its next turn.
    #[cfg(not(windows))]
    stopping: AtomicBool,
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
        // On this thread, so the change of window and the keys are never heard at once.
        // SAFETY: `foreground_proc` has the signature of a WinEventProc and is never unloaded.
        let on_front = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND as u32,
                EVENT_SYSTEM_FOREGROUND as u32,
                null_mut(),
                Some(foreground_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT as u32,
            )
        };
        self.ready.set();
        // SAFETY: MSG is plain data that GetMessageW fills in.
        let mut message: MSG = unsafe { std::mem::zeroed() };
        // SAFETY: `message` is live for each call, and a null window takes this thread's messages.
        while unsafe { GetMessageW(&mut message, null_mut(), 0, 0) } > 0 {}
        // SAFETY: `hook` was returned by SetWindowsHookExW above and is removed once.
        unsafe { UnhookWindowsHookEx(hook) };
        if !on_front.is_null() {
            // SAFETY: `on_front` was returned by SetWinEventHook above and is removed once.
            unsafe { UnhookWinEvent(on_front) };
        }
    }

    fn hook(&self, code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code >= 0 {
            // SAFETY: for a code of 0 or more, a WH_KEYBOARD_LL hook's lParam points at a
            // KBDLLHOOKSTRUCT that lives until the hook returns.
            let event = unsafe { &*(lparam as *const KeyboardHookStruct) };
            let message = wparam as i64;
            // A low-level hook is called for these four messages only.
            let down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
            if self.key(event.vkCode as i64, down, held) {
                return 1;
            }
        }
        // SAFETY: the arguments are the ones this hook was called with.
        unsafe { CallNextHookEx(null_mut(), code, wparam, lparam) }
    }
}

impl State {
    /// Whether the emulator's window is in front again after another was, and so whether it is
    /// now in front, which this keeps.
    fn back(&self) -> bool {
        #[cfg(not(target_os = "linux"))]
        let front = window::in_front(&self.emulator);
        // A key grabbed is taken from whatever program is in front, so only where X says.
        #[cfg(target_os = "linux")]
        let front = window::front_known(&self.emulator) == Some(true);
        let back = front && !self.front.load(Ordering::SeqCst);
        self.front.store(front, Ordering::SeqCst);
        back
    }

    #[cfg(any(windows, target_os = "linux"))]
    fn foreground(&self) {
        if self.back() {
            lock(&self.elsewhere).clear();
        }
    }

    /// One key going down or coming up, `virtual_key` as Windows numbers it, and `held` whether
    /// a key is held now: whether it is kept from the window in front. Every platform's keys come
    /// through here, so a key means one thing on all of them.
    fn key(&self, virtual_key: i64, down: bool, held: impl Fn(i64) -> bool) -> bool {
        let message = if down { WM_KEYDOWN } else { WM_KEYUP };
        let back = self.back();
        track(&mut lock(&self.elsewhere), virtual_key, message, back);
        let front = self.front.load(Ordering::SeqCst);
        if !down {
            let up = lock(&self.released).up(virtual_key);
            if !up.is_empty() {
                // In front of any window, as Shift's: a press held is let go wherever the
                // player has gone.
                lock(&self.pressed).push_back(up);
            }
        }
        if !down && SHIFTS.contains(&virtual_key) {
            self.shift_down.store(false, Ordering::SeqCst);
        }
        if down && SHIFTS.contains(&virtual_key) {
            // Shift alone, passed on all the same and said once however long it is held:
            // touch mode drags with it on the screens that drag (ADR-0009).
            if !self.shift_down.load(Ordering::SeqCst)
                && !held(VK_CONTROL)
                && !held(VK_MENU)
                && lock(&self.elsewhere).is_empty()
                && !ELSEWHERE.iter().any(|&key| held(key))
                && front
            {
                self.shift_down.store(true, Ordering::SeqCst);
                self.shifted.store(true, Ordering::SeqCst);
                lock(&self.pressed).push_back(SHIFT_DOWN.to_string());
            }
            return false;
        }
        if !down && SHIFTS.contains(&virtual_key) && self.shifted.load(Ordering::SeqCst) {
            // Passed on all the same, and in front of any window: a press held is let go
            // wherever the player has gone.
            self.shifted.store(false, Ordering::SeqCst);
            lock(&self.pressed).push_back(SHIFT_UP.to_string());
        }
        if down {
            let (ctrl, alt, shift) = (held(VK_CONTROL), held(VK_MENU), held(VK_SHIFT));
            let name = modified(&key_name(virtual_key), ctrl, alt, shift);
            let elsewhere =
                !lock(&self.elsewhere).is_empty() || ELSEWHERE.iter().any(|&key| held(key));
            let keys: Vec<String> = self
                .keys
                .iter()
                .chain(lock(&self.contextual).iter())
                .cloned()
                .collect();
            let key = bound(&name, &keys, ctrl, alt, elsewhere);
            if !key.is_empty() && front {
                if lock(&self.released).down(virtual_key, &key) {
                    if shift {
                        self.shifted.store(true, Ordering::SeqCst);
                    }
                    lock(&self.pressed).push_back(key);
                }
                return true;
            }
        }
        false
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

/// Windows' call on a change of window in front: on to the `State` this thread heard it for, as
/// `hook_proc` is. An out-of-context event comes on the thread that asked for it.
#[cfg(windows)]
unsafe extern "system" fn foreground_proc(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    let state = HOOKED.with(|hooked| hooked.borrow().clone());
    if let Some(state) = state {
        state.foreground();
    }
}

/// Keys taken, by name, only while the emulator's window is in front.
pub struct Hotkeys {
    state: Arc<State>,
    thread: Option<Thread>,
}

impl Hotkeys {
    pub fn new<S: AsRef<str>, R: AsRef<str>>(
        keys: impl IntoIterator<Item = S>,
        emulator: &str,
        released: impl IntoIterator<Item = R>,
    ) -> Self {
        // Python keeps the hook's ctypes callback on the object, because ctypes frees it with the
        // last reference and the hook would call freed code; a Rust function is never freed.
        Hotkeys {
            state: Arc::new(State {
                keys: keys
                    .into_iter()
                    .map(|key| key.as_ref().to_lowercase())
                    .collect(),
                emulator: emulator.to_string(),
                released: Mutex::new(Released::new(released)),
                contextual: Mutex::new(HashSet::new()),
                pressed: Mutex::new(VecDeque::new()),
                elsewhere: Mutex::new(HashSet::new()),
                front: AtomicBool::new(true),
                shifted: AtomicBool::new(false),
                shift_down: AtomicBool::new(false),
                #[cfg(windows)]
                thread_id: AtomicU32::new(0),
                #[cfg(not(windows))]
                stopping: AtomicBool::new(false),
                ready: Event::default(),
                error: Mutex::new(String::new()),
            }),
            thread: None,
        }
    }

    /// Hotkeys for the emulator pa3ds is made for, `released` the keys whose release is said.
    pub fn of<S: AsRef<str>, R: AsRef<str>>(
        keys: impl IntoIterator<Item = S>,
        released: impl IntoIterator<Item = R>,
    ) -> Self {
        Hotkeys::new(keys, EMULATOR, released)
    }

    /// Install the hook on a thread of its own. Returns why it could not, or "".
    pub fn start(&mut self) -> Result<String> {
        let state = self.state.clone();
        self.thread = Some(Thread::start("hotkeys", move || state.run())?);
        self.state.ready.wait(5.0);
        Ok(lock(&self.state.error).clone())
    }

    pub fn stop(&self) {
        #[cfg(windows)]
        {
            let thread_id = self.state.thread_id.load(Ordering::SeqCst);
            if thread_id != 0 {
                // SAFETY: PostThreadMessageW only reads its arguments; a thread that has ended is
                // a failed call.
                unsafe { PostThreadMessageW(thread_id, WM_QUIT as u32, 0, 0) };
            }
        }
        #[cfg(not(windows))]
        self.state.stopping.store(true, Ordering::SeqCst);
        if let Some(thread) = &self.thread {
            thread.join(2.0);
        }
    }

    /// `hotkeys.contextual = keys`: the keys of the screen running now that has its own, taken
    /// besides the rest until the next call.
    pub fn set_contextual(&self, keys: HashSet<String>) {
        *lock(&self.state.contextual) = keys;
    }

    /// The keys pressed since the last call, in order.
    pub fn pressed(&self) -> Vec<String> {
        lock(&self.state.pressed).drain(..).collect()
    }
}

/// macOS's keycodes (`kVK_ANSI_*`, numbered by their place on a US keyboard) as the Windows
/// virtual keys of the same place, so a key has one name on both and the keys file is the same.
pub const MAC_KEYS: [(u16, i64); 49] = [
    (0x00, 0x41), // A
    (0x0B, 0x42),
    (0x08, 0x43),
    (0x02, 0x44),
    (0x0E, 0x45),
    (0x03, 0x46),
    (0x05, 0x47),
    (0x04, 0x48),
    (0x22, 0x49),
    (0x26, 0x4A),
    (0x28, 0x4B),
    (0x25, 0x4C),
    (0x2E, 0x4D),
    (0x2D, 0x4E),
    (0x1F, 0x4F),
    (0x23, 0x50),
    (0x0C, 0x51),
    (0x0F, 0x52),
    (0x01, 0x53),
    (0x11, 0x54),
    (0x20, 0x55),
    (0x09, 0x56),
    (0x0D, 0x57),
    (0x07, 0x58),
    (0x10, 0x59),
    (0x06, 0x5A), // Z
    (0x1D, 0x30), // 0
    (0x12, 0x31),
    (0x13, 0x32),
    (0x14, 0x33),
    (0x15, 0x34),
    (0x17, 0x35),
    (0x16, 0x36),
    (0x1A, 0x37),
    (0x1C, 0x38),
    (0x19, 0x39), // 9
    (0x29, 0xBA), // ;
    (0x18, 0xBB), // =
    (0x2B, 0xBC), // ,
    (0x1B, 0xBD), // -
    (0x2F, 0xBE), // .
    (0x2C, 0xBF), // /
    (0x32, 0xC0), // `
    (0x21, 0xDB), // [
    (0x2A, 0xDC), // \
    (0x1E, 0xDD), // ]
    (0x27, 0xDE), // '
    (0x24, 0x0D), // Return
    (0x4C, 0x0D), // the keypad's Enter, which Windows numbers as Return too
];

/// macOS's modifier keys, which come as a change of the event's flags rather than as a key going
/// down or up: keycode, the Windows virtual key a low-level hook names it by, the flag of that
/// one key, and the flag of either of its pair. Command is the Windows keys, so that like them it
/// keeps pa3ds's keys from being taken (`ELSEWHERE`).
pub const MAC_MODIFIERS: [(u16, i64, u64, u64); 8] = [
    (0x38, 0xA0, 0x02, MAC_SHIFT),
    (0x3C, 0xA1, 0x04, MAC_SHIFT),
    (0x3B, 0xA2, 0x01, MAC_CONTROL),
    (0x3E, 0xA3, 0x2000, MAC_CONTROL),
    (0x3A, 0xA4, 0x20, MAC_OPTION),
    (0x3D, 0xA5, 0x40, MAC_OPTION),
    (0x37, 0x5B, 0x08, MAC_COMMAND),
    (0x36, 0x5C, 0x10, MAC_COMMAND),
];
pub const MAC_SHIFT: u64 = 0x2_0000;
pub const MAC_CONTROL: u64 = 0x4_0000;
pub const MAC_OPTION: u64 = 0x8_0000;
pub const MAC_COMMAND: u64 = 0x10_0000;
/// Every flag of one key of a pair; a keyboard that sets none of them sets the pair's alone.
pub const MAC_ONE_KEY: u64 = 0x207F;
/// Caps lock, which is Windows' 0x14 here. Its flag says whether it is locked, not held, so
/// whether it is held is asked of the keyboard (`CGEventSourceKeyState`).
pub const MAC_CAPS_LOCK: u16 = 0x39;

/// A change of macOS's modifier flags as the Windows virtual key that went down or came up, and
/// whether it went down; none for a key that is not one of `MAC_MODIFIERS`.
pub fn mac_modifier(keycode: u16, flags: u64) -> Option<(i64, bool)> {
    let (_, virtual_key, one, pair) = MAC_MODIFIERS.iter().find(|key| key.0 == keycode)?;
    let flag = if flags & MAC_ONE_KEY != 0 { one } else { pair };
    Some((*virtual_key, flags & flag != 0))
}

/// An X keysym, as a US layout's first level gives it, as the Windows virtual key that carries
/// it: letters, digits and `PUNCTUATION` (whose keysyms are their ASCII codes), Return, and the
/// keys of `ELSEWHERE`.
pub fn vk_of_keysym(keysym: u32) -> Option<i64> {
    match keysym {
        0x61..=0x7A => Some(keysym as i64 - 0x20),
        0x30..=0x39 => Some(keysym as i64),
        // Return and the keypad's Enter.
        0xFF0D | 0xFF8D => Some(0x0D),
        // Insert and the keypad's, Orca's modifier.
        0xFF63 | 0xFF9E => Some(0x2D),
        0xFFE5 => Some(0x14),
        0xFFEB => Some(0x5B),
        0xFFEC => Some(0x5C),
        _ => PUNCTUATION
            .iter()
            .find(|(_, character)| character.as_bytes() == [keysym as u8] && keysym < 0x80)
            .map(|(virtual_key, _)| *virtual_key),
    }
}

/// No keys anywhere else.
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
impl State {
    fn run(self: &Arc<Self>) {
        *lock(&self.error) = "keys need Windows, macOS or Linux".to_string();
        self.ready.set();
    }
}

/// macOS: an event tap, which sees every key on its way to the window in front and can keep it
/// from getting there, as Windows' low-level hook does. It runs on a thread of its own with the
/// run loop it is called from. macOS gives one only to a program the player has allowed to
/// control the computer (Accessibility); for a pa3ds started from a terminal, that is the
/// terminal.
#[cfg(target_os = "macos")]
mod mac {
    use std::cell::Cell;
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use super::{
        MAC_CAPS_LOCK, MAC_COMMAND, MAC_CONTROL, MAC_KEYS, MAC_OPTION, MAC_SHIFT, State,
        VK_CONTROL, VK_MENU, VK_SHIFT, lock, mac_modifier,
    };

    type Ref = *mut c_void;

    const SESSION_TAP: u32 = 1;
    const HEAD_INSERT: u32 = 0;
    const DEFAULT_TAP: u32 = 0;
    const KEY_DOWN: u32 = 10;
    const KEY_UP: u32 = 11;
    const FLAGS_CHANGED: u32 = 12;
    const DISABLED_BY_TIMEOUT: u32 = 0xFFFF_FFFE;
    const DISABLED_BY_USER_INPUT: u32 = 0xFFFF_FFFF;
    const KEYCODE: u32 = 9;
    const HID_SYSTEM_STATE: i32 = 1;

    type Callback = extern "C" fn(Ref, u32, Ref, *mut c_void) -> Ref;

    unsafe extern "C" {
        fn CGEventTapCreate(
            tap: u32,
            place: u32,
            options: u32,
            events: u64,
            callback: Callback,
            user: *mut c_void,
        ) -> Ref;
        fn CGEventTapEnable(tap: Ref, enable: bool);
        fn CGEventGetIntegerValueField(event: Ref, field: u32) -> i64;
        fn CGEventGetFlags(event: Ref) -> u64;
        fn CGEventSourceKeyState(state: i32, key: u16) -> bool;
        fn CFMachPortCreateRunLoopSource(allocator: Ref, port: Ref, order: isize) -> Ref;
        fn CFMachPortInvalidate(port: Ref);
        fn CFRunLoopGetCurrent() -> Ref;
        fn CFRunLoopAddSource(run_loop: Ref, source: Ref, mode: Ref);
        fn CFRunLoopRunInMode(mode: Ref, seconds: f64, after_one: u8) -> i32;
        // `CFTypeRef` is `const void *`, as `window`'s declaration has it.
        fn CFRelease(object: *const c_void);
        static kCFRunLoopDefaultMode: Ref;
    }

    /// What the tap is called with: the keys' state, and the tap itself to turn back on.
    struct Tap {
        state: Arc<State>,
        port: Cell<Ref>,
    }

    impl State {
        pub(super) fn run(self: &Arc<Self>) {
            let tap = Tap {
                state: self.clone(),
                port: Cell::new(null_mut()),
            };
            let events = (1 << KEY_DOWN) | (1 << KEY_UP) | (1 << FLAGS_CHANGED);
            // SAFETY: `on_key` has the callback's signature, and `tap` outlives the run loop, the
            // only caller of `on_key`.
            let port = unsafe {
                CGEventTapCreate(
                    SESSION_TAP,
                    HEAD_INSERT,
                    DEFAULT_TAP,
                    events,
                    on_key,
                    &tap as *const Tap as *mut c_void,
                )
            };
            if port.is_null() {
                *lock(&self.error) = "keys need Accessibility: allow the terminal pa3ds runs in \
                    under System Settings, Privacy & Security, Accessibility, and start pa3ds again"
                    .to_string();
                self.ready.set();
                return;
            }
            tap.port.set(port);
            // SAFETY: `port` is the tap made above, and the source is made, added and released
            // once, on this thread, whose run loop it is added to.
            unsafe {
                let source = CFMachPortCreateRunLoopSource(null_mut(), port, 0);
                CFRunLoopAddSource(CFRunLoopGetCurrent(), source, kCFRunLoopDefaultMode);
                self.ready.set();
                // A turn at a time, so `stop` is heard within one.
                while !self.stopping.load(Ordering::SeqCst) {
                    CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.25, 0);
                }
                CFMachPortInvalidate(port);
                CFRelease(source);
                CFRelease(port);
            }
        }
    }

    /// macOS's call for each key: a null event keeps it from the window in front.
    extern "C" fn on_key(_proxy: Ref, kind: u32, event: Ref, user: *mut c_void) -> Ref {
        // SAFETY: `user` is the `Tap` that `run` keeps alive while its run loop calls this.
        let tap = unsafe { &*(user as *const Tap) };
        if kind == DISABLED_BY_TIMEOUT || kind == DISABLED_BY_USER_INPUT {
            // macOS turns off a tap that answered too slowly; pa3ds's keys stay pa3ds's.
            // SAFETY: the port is the live tap this callback belongs to.
            unsafe { CGEventTapEnable(tap.port.get(), true) };
            return event;
        }
        // SAFETY: for a key event, `event` is a live CGEvent until the callback returns.
        let (keycode, flags) = unsafe {
            (
                CGEventGetIntegerValueField(event, KEYCODE) as u16,
                CGEventGetFlags(event),
            )
        };
        let key = if kind == FLAGS_CHANGED {
            mac_modifier(keycode, flags)
        } else {
            MAC_KEYS
                .iter()
                .find(|(code, _)| *code == keycode)
                .map(|(_, virtual_key)| (*virtual_key, kind == KEY_DOWN))
        };
        let Some((virtual_key, down)) = key else {
            return event;
        };
        let (control, option) = (flags & MAC_CONTROL != 0, flags & MAC_OPTION != 0);
        let held = |key: i64| match key {
            VK_SHIFT => flags & MAC_SHIFT != 0,
            VK_CONTROL => control,
            VK_MENU => option,
            0x5B | 0x5C => flags & MAC_COMMAND != 0,
            // SAFETY: CGEventSourceKeyState only reads its arguments.
            0x14 => unsafe { CGEventSourceKeyState(HID_SYSTEM_STATE, MAC_CAPS_LOCK) },
            // Insert, NVDA's modifier: VoiceOver's is Control and Option held together.
            0x2D => control && option,
            _ => false,
        };
        if tap.state.key(virtual_key, down, held) {
            null_mut()
        } else {
            event
        }
    }
}

/// Linux, on X11: X gives no program every key, as Windows' hook and macOS's tap do, so while the
/// emulator's window is in front pa3ds grabs each key it is bound to, as it is bound and with
/// Shift (`bound`'s fallback), and with caps lock and num lock either way. A grabbed key comes to
/// pa3ds with the keyboard held still: one pa3ds takes is let go of at once, so the keys pressed
/// while it is held still reach the emulator, and one it does not take goes on to the window in
/// front as if never grabbed. Shift is never grabbed, so Shift alone, "shift down" and "shift
/// up", is not heard here, and touch mode drags only with a key bound to drag. Wayland gives no
/// program keys it has not been given, so there pa3ds binds none; and with an Azahar X cannot
/// show in front, a Wayland window or one under a window manager that names no active window,
/// pa3ds grabs none (`window::front_known`).
#[cfg(target_os = "linux")]
mod x11 {
    use std::collections::HashSet;
    use std::os::unix::io::AsRawFd;
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use x11rb::connection::Connection;
    use x11rb::protocol::Event;
    use x11rb::protocol::xproto::{
        Allow, Atom, ChangeWindowAttributesAux, ConnectionExt as _, EventMask, Grab, GrabMode,
        KeyButMask, Keycode, ModMask, Window,
    };
    use x11rb::rust_connection::RustConnection;

    use super::vk_of_keysym;
    use super::{ALT, CTRL, SHIFT, State, VK_CONTROL, VK_MENU, VK_SHIFT, key_name, lock};

    type Failed = Box<dyn std::error::Error>;

    /// Caps lock and num lock, which X counts as modifiers: a key is grabbed under each.
    // shortcut: num lock is taken to be Mod2, as on every common X setup; read the modifier
    // mapping if a player's is not.
    const LOCKS: [u16; 4] = [0, 1 << 1, 1 << 4, (1 << 1) | (1 << 4)];

    struct Keyboard {
        connection: RustConnection,
        root: Window,
        active: Atom,
        first: Keycode,
        per: usize,
        // shortcut: the keyboard's mapping is read once; read it again on MappingNotify if
        // players change layouts while pa3ds runs.
        keysyms: Vec<u32>,
        /// What the grabs were made for: whether the emulator was in front, and the contextual
        /// keys then.
        made_for: Option<(bool, HashSet<String>)>,
        grabbed: HashSet<(Keycode, u16)>,
        /// The keys taken and not yet up. The grab is let go as a key is taken, so its release
        /// goes to the window in front, and is read off the keyboard instead.
        down: Vec<(Keycode, i64)>,
    }

    impl State {
        pub(super) fn run(self: &Arc<Self>) {
            let mut keyboard = match Keyboard::open() {
                Ok(keyboard) => keyboard,
                Err(reason) => {
                    *lock(&self.error) = reason;
                    self.ready.set();
                    return;
                }
            };
            self.foreground();
            self.ready.set();
            // Over at `stop`, or with the X server.
            let _ = keyboard.listen(self);
        }
    }

    impl Keyboard {
        fn open() -> Result<Keyboard, String> {
            if std::env::var_os("DISPLAY").is_none() {
                return Err(
                    "keys need X11, and there is no DISPLAY: Wayland gives a program \
                    no keys of its own"
                        .to_string(),
                );
            }
            Keyboard::connect().map_err(|error| format!("keys need X11: {error}"))
        }

        fn connect() -> Result<Keyboard, Failed> {
            let (connection, screen) = RustConnection::connect(None)?;
            let setup = connection.setup();
            let (root, first, last) = (
                setup.roots[screen].root,
                setup.min_keycode,
                setup.max_keycode,
            );
            let mapping = connection
                .get_keyboard_mapping(first, last - first + 1)?
                .reply()?;
            let active = connection
                .intern_atom(false, b"_NET_ACTIVE_WINDOW")?
                .reply()?
                .atom;
            let changes = ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE);
            connection
                .change_window_attributes(root, &changes)?
                .check()?;
            Ok(Keyboard {
                connection,
                root,
                active,
                first,
                per: usize::from(mapping.keysyms_per_keycode.max(1)),
                keysyms: mapping.keysyms,
                made_for: None,
                grabbed: HashSet::new(),
                down: Vec::new(),
            })
        }

        fn listen(&mut self, state: &State) -> Result<(), Failed> {
            let socket = self.connection.stream().as_raw_fd();
            while !state.stopping.load(Ordering::SeqCst) {
                self.release(state)?;
                while let Some(event) = self.connection.poll_for_event()? {
                    self.event(state, event)?;
                }
                self.grab(state)?;
                self.connection.flush()?;
                let mut ready = libc::pollfd {
                    fd: socket,
                    events: libc::POLLIN,
                    revents: 0,
                };
                // A turn at least every 50 ms, so `stop` is heard and a key taken is seen up.
                // SAFETY: one pollfd, live for the call.
                unsafe { libc::poll(&mut ready, 1, 50) };
            }
            Ok(())
        }

        /// The Windows virtual key of the key with this keycode, by its first keysym.
        fn virtual_key(&self, keycode: Keycode) -> Option<i64> {
            let at = usize::from(keycode.checked_sub(self.first)?) * self.per;
            vk_of_keysym(*self.keysyms.get(at)?)
        }

        fn keycodes(&self, virtual_key: i64) -> impl Iterator<Item = Keycode> + '_ {
            (0..self.keysyms.len() / self.per)
                .map(|at| self.first.wrapping_add(at as u8))
                .filter(move |&keycode| self.virtual_key(keycode) == Some(virtual_key))
        }

        fn event(&mut self, state: &State, event: Event) -> Result<(), Failed> {
            match event {
                Event::PropertyNotify(event) if event.atom == self.active => state.foreground(),
                Event::KeyPress(event) => {
                    let virtual_key = self.virtual_key(event.detail);
                    let taken = match virtual_key {
                        Some(virtual_key) => {
                            let keys = self.connection.query_keymap()?.reply()?.keys;
                            let held = |key: i64| match key {
                                VK_SHIFT => event.state.contains(KeyButMask::SHIFT),
                                VK_CONTROL => event.state.contains(KeyButMask::CONTROL),
                                VK_MENU => event.state.contains(KeyButMask::MOD1),
                                _ => self.keycodes(key).any(|keycode| pressed(&keys, keycode)),
                            };
                            state.key(virtual_key, true, held)
                        }
                        None => false,
                    };
                    if let (true, Some(virtual_key)) = (taken, virtual_key) {
                        self.connection
                            .allow_events(Allow::ASYNC_KEYBOARD, event.time)?;
                        self.connection.ungrab_keyboard(event.time)?;
                        if !self
                            .down
                            .iter()
                            .any(|(keycode, _)| *keycode == event.detail)
                        {
                            self.down.push((event.detail, virtual_key));
                        }
                    } else {
                        self.connection
                            .allow_events(Allow::REPLAY_KEYBOARD, event.time)?;
                    }
                }
                _ => {}
            }
            Ok(())
        }

        /// Say each key taken that has come up since the last turn.
        fn release(&mut self, state: &State) -> Result<(), Failed> {
            if self.down.is_empty() {
                return Ok(());
            }
            let keys = self.connection.query_keymap()?.reply()?.keys;
            let (held, up): (Vec<_>, Vec<_>) = self
                .down
                .drain(..)
                .partition(|(keycode, _)| pressed(&keys, *keycode));
            self.down = held;
            for (_, virtual_key) in up {
                state.key(virtual_key, false, |_| false);
            }
            Ok(())
        }

        /// Grab the keys pa3ds is bound to while the emulator is in front, and none otherwise.
        fn grab(&mut self, state: &State) -> Result<(), Failed> {
            let front = state.front.load(Ordering::SeqCst);
            let contextual = lock(&state.contextual).clone();
            let made_for = Some((front, contextual));
            if made_for == self.made_for {
                return Ok(());
            }
            let wanted: HashSet<(Keycode, u16)> = match &made_for {
                Some((true, contextual)) => state
                    .keys
                    .iter()
                    .chain(contextual)
                    .flat_map(|name| self.combinations(name))
                    .collect(),
                _ => HashSet::new(),
            };
            self.made_for = made_for;
            if wanted == self.grabbed {
                return Ok(());
            }
            self.connection
                .ungrab_key(Grab::ANY, self.root, ModMask::ANY)?;
            for &(keycode, modifiers) in &wanted {
                // Errors, a key another program has grabbed, come as events and are let be.
                self.connection.grab_key(
                    false,
                    self.root,
                    ModMask::from(modifiers),
                    keycode,
                    GrabMode::ASYNC,
                    GrabMode::SYNC,
                )?;
            }
            self.grabbed = wanted;
            Ok(())
        }

        /// The keycodes and modifiers a key bound as `name` is pressed with: as written, and
        /// with Shift where `bound` would fall back to it, each with and without the locks.
        fn combinations(&self, name: &str) -> Vec<(Keycode, u16)> {
            let mut parts: Vec<&str> = name.split('+').collect();
            let key = parts.pop().unwrap_or_default();
            let Some(virtual_key) = (0..=0xFF).find(|&virtual_key| key_name(virtual_key) == key)
            else {
                return Vec::new();
            };
            let mut modifiers = 0;
            for part in parts {
                modifiers |= match part {
                    SHIFT => u16::from(ModMask::SHIFT),
                    CTRL => u16::from(ModMask::CONTROL),
                    ALT => u16::from(ModMask::M1),
                    _ => return Vec::new(),
                };
            }
            let mut sets = vec![modifiers];
            // Shift and a key bound alone is the key (`bound`).
            if modifiers == 0 {
                sets.push(u16::from(ModMask::SHIFT));
            }
            let mut out = Vec::new();
            for keycode in self.keycodes(virtual_key) {
                for set in &sets {
                    for lock in LOCKS {
                        out.push((keycode, set | lock));
                    }
                }
            }
            out
        }
    }

    fn pressed(keys: &[u8; 32], keycode: Keycode) -> bool {
        keys[usize::from(keycode / 8)] & (1 << (keycode % 8)) != 0
    }
}

#[cfg(test)]
mod checks {
    use super::*;

    #[test]
    fn macs_keys_are_named_as_windows_names_them() {
        let name = |keycode: u16| {
            let (_, virtual_key) = MAC_KEYS.iter().find(|key| key.0 == keycode).unwrap();
            key_name(*virtual_key)
        };
        assert_eq!(name(0x00), "a");
        assert_eq!(name(0x28), "k");
        assert_eq!(name(0x1D), "0");
        assert_eq!(name(0x29), ";");
        assert_eq!(name(0x24), "enter");
        for (virtual_key, _) in PUNCTUATION {
            assert!(
                MAC_KEYS.iter().any(|key| key.1 == virtual_key),
                "{virtual_key:#x}"
            );
        }
        for letter in 0x41..=0x5A {
            assert!(MAC_KEYS.iter().any(|key| key.1 == letter), "{letter:#x}");
        }
    }

    #[test]
    fn macs_modifiers_go_down_and_up_by_their_own_flag() {
        assert_eq!(mac_modifier(0x38, MAC_SHIFT | 0x02), Some((0xA0, true)));
        assert_eq!(mac_modifier(0x38, MAC_SHIFT | 0x04), Some((0xA0, false)));
        assert_eq!(mac_modifier(0x3C, MAC_SHIFT), Some((0xA1, true)));
        assert_eq!(mac_modifier(0x37, 0), Some((0x5B, false)));
        assert_eq!(mac_modifier(MAC_CAPS_LOCK, 0x1_0000), None);
        for (_, virtual_key, _, _) in MAC_MODIFIERS {
            assert!(
                SHIFTS.contains(&virtual_key)
                    || ELSEWHERE.contains(&virtual_key)
                    || (0xA2..=0xA5).contains(&virtual_key)
            );
        }
    }

    #[test]
    fn x_keysyms_are_named_as_windows_names_them() {
        let name = |keysym: u32| vk_of_keysym(keysym).map(key_name);
        assert_eq!(name(0x6B).as_deref(), Some("k"));
        assert_eq!(name(0x37).as_deref(), Some("7"));
        assert_eq!(name(0x3B).as_deref(), Some(";"));
        assert_eq!(name(0x5C).as_deref(), Some("\\"));
        assert_eq!(name(0xFF0D).as_deref(), Some("enter"));
        assert_eq!(name(0x4B), None);
        for (virtual_key, character) in PUNCTUATION {
            let keysym = u32::from(character.as_bytes()[0]);
            assert_eq!(vk_of_keysym(keysym), Some(virtual_key), "{character}");
        }
        for key in ELSEWHERE {
            assert!(
                (0xFF00..=0xFFFF).any(|keysym| vk_of_keysym(keysym) == Some(key)),
                "{key:#x}"
            );
        }
    }
}
