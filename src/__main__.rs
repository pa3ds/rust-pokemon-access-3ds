//! `pa3ds` (or `python -m pa3ds`): announce what the game shows. The executable is `src/main.rs`,
//! which calls `main` here, so the tests can reach what the Python's reached.

use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::app::{self, Settings};
use crate::core::rpc::{DEFAULT_PORT, Rpc, UdpRpc};
use crate::core::speech::{self, Backend, Category, Speech};
use crate::core::{config, instance, keyfile, window};
use crate::games::common::field::keys::{DEFAULT_BINDINGS, EXPLAINED};
use crate::py::{self, Log, time};

/// Runs whose logs are kept, this one included.
pub const KEPT_LOGS: usize = 10;
const LOG_NAME: &str = "pa3ds.log";
const LOG_FOLDER: &str = "logs";

/// A backend that says nothing, for runs where the log is the output.
pub struct Silent;

impl Backend for Silent {
    fn speak(&self, _text: &str, _interrupt: bool) {}

    fn braille(&self, _text: &str) {}
}

/// The command line: a flag is only what a developer passes for one run; settings are the
/// file's (Orion, 2026-09-30).
#[derive(Default)]
struct Args {
    silent: bool,
    seconds: Option<f64>,
    port: u16,
    no_keys: bool,
    rom: Option<PathBuf>,
    no_log: bool,
    log_path: Option<PathBuf>,
}

fn usage() -> String {
    "usage: pa3ds [-h] [--silent] [--seconds SECONDS] [--port PORT] [--no-keys] [--rom PATH]\n\
     \x20            [--no-log | --log-path PATH]"
        .to_string()
}

fn help() -> Result<String, String> {
    let config = config::config_path(None).map_err(|error| error.to_string())?;
    Ok(format!(
        "{}\n\nAnnounce what the running game shows.\n\noptions:\n\
         \x20 -h, --help       show this help message and exit\n\
         \x20 --silent         log what would be said, say nothing\n\
         \x20 --seconds SECONDS\n\
         \x20                  stop after this many seconds\n\
         \x20 --port PORT      the emulator's RPC port\n\
         \x20 --no-keys        answer no keys and patch no buttons, whatever the settings say\n\
         \x20 --rom PATH       your decrypted ROM, for the game's own names of places and the\n\
         \x20                  rival; by default pa3ds finds it among the ROMs Azahar has opened\n\
         \x20 --no-log         write no log file; what is said is still printed\n\
         \x20 --log-path PATH  write the log to this file, or to pa3ds.log in this folder,\n\
         \x20                  keeping earlier runs beside it\n\n\
         Each run is logged to {} unless --no-log or --log-path says otherwise, and the {} runs \
         before it are kept beside it as pa3ds.1.log (the newest) to pa3ds.{}.log. Everything \
         else is set in {}.",
        usage(),
        default_log_path()
            .map_err(|error| error.to_string())?
            .display(),
        KEPT_LOGS - 1,
        KEPT_LOGS - 1,
        config.display(),
    ))
}

fn parse(argv: &[String]) -> Result<Args, String> {
    let mut args = Args {
        port: DEFAULT_PORT,
        ..Args::default()
    };
    let mut at = 0;
    let value = |at: &mut usize, flag: &str| -> Result<String, String> {
        *at += 1;
        argv.get(*at)
            .cloned()
            .ok_or_else(|| format!("argument {flag}: expected one argument"))
    };
    while at < argv.len() {
        let arg = argv[at].as_str();
        let (flag, inline) = match arg.split_once('=') {
            Some((flag, inline)) if flag.starts_with("--") => (flag, Some(inline.to_string())),
            _ => (arg, None),
        };
        let take = |at: &mut usize| match &inline {
            Some(inline) => Ok(inline.clone()),
            None => value(at, flag),
        };
        match flag {
            "-h" | "--help" => return Err(help()?),
            "--silent" => args.silent = true,
            "--no-keys" => args.no_keys = true,
            "--no-log" => args.no_log = true,
            "--seconds" => {
                let text = take(&mut at)?;
                args.seconds =
                    Some(py::float_of(&text).map_err(|_| {
                        format!("argument --seconds: invalid float value: '{text}'")
                    })?);
            }
            "--port" => {
                let text = take(&mut at)?;
                args.port = text
                    .trim()
                    .parse()
                    .map_err(|_| format!("argument --port: invalid int value: '{text}'"))?;
            }
            "--rom" => args.rom = Some(PathBuf::from(take(&mut at)?)),
            "--log-path" => args.log_path = Some(PathBuf::from(take(&mut at)?)),
            other => return Err(format!("unrecognized arguments: {other}")),
        }
        at += 1;
    }
    if args.no_log && args.log_path.is_some() {
        return Err("argument --log-path: not allowed with argument --no-log".to_string());
    }
    Ok(args)
}

/// The command line `argv` (without the program's name), run: 0 when it ran, 2 when the arguments
/// were wrong, 1 when it failed.
pub fn main(argv: &[String]) -> i32 {
    main_with(argv, &Hooks::default())
}

/// What `main` hands its run to: the runtime, the backend `--silent` says nothing through, and
/// whether the game's window is in front. The Python's tests patched each of these in place.
pub struct Hooks {
    pub run:
        Box<dyn Fn(Rc<dyn Rpc>, Rc<Speech>, Settings, &dyn Fn() -> bool, Log) -> py::Result<()>>,
    pub silent: Rc<dyn Backend>,
    pub in_front: Rc<dyn Fn() -> bool>,
}

impl Default for Hooks {
    fn default() -> Self {
        Hooks {
            run: Box::new(app::run),
            silent: Rc::new(Silent),
            in_front: Rc::new(|| window::in_front(window::EMULATOR)),
        }
    }
}

/// `main`, run through `hooks`.
pub fn main_with(argv: &[String], hooks: &Hooks) -> i32 {
    let args = match parse(argv) {
        Ok(args) => args,
        Err(message) if message.starts_with("usage:") => {
            println!("{message}");
            return 0;
        }
        Err(message) => {
            eprintln!(
                "{}
pa3ds: error: {message}",
                usage()
            );
            return 2;
        }
    };
    match run(args, hooks) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("pa3ds: {}: {}", error.kind.name(), error.message);
            1
        }
    }
}

/// `atexit.register(instance.release, home)`: the file naming this pa3ds goes when `run` ends, by
/// returning, raising or panicking, and not when another pa3ds ends this process.
struct Release(PathBuf);

impl Drop for Release {
    fn drop(&mut self) {
        instance::release(&self.0);
    }
}

fn run(args: Args, hooks: &Hooks) -> py::Result<()> {
    // One pa3ds at a time (ADR-0039). Not a silent run, which is an agent's or a developer's: it
    // would end the player's own pa3ds mid-game. The file naming this one goes when it closes,
    // and not when another ends it.
    let mut replaced = false;
    let mut _release = None;
    if !args.silent {
        replaced = instance::replace(&keyfile::home_path()?)?;
        _release = Some(Release(keyfile::home_path()?));
    }
    // A patch of Azahar's config is lost when an open Azahar closes, so none is written while
    // it is open.
    let running = !window::titles(window::EMULATOR).is_empty();
    let mut settings = config::prepare(None, None, true, running)?;
    let mut asked: Vec<String> = Vec::new();
    // A silent run is an agent's or a developer's, with nobody to answer; and with Azahar open the
    // player has already started it, so its settings are somewhere pa3ds can look.
    if !args.silent && !running && !settings.emulator.is_file() {
        settings = ask_for_azahar(settings, &mut asked)?;
    }
    let ui = settings.ui.clone();

    let backend: Rc<dyn Backend> = if args.silent {
        hooks.silent.clone()
    } else {
        let (backend, name) = speech::screen_reader()?;
        println!("speaking through {name}");
        backend
    };

    let log_file: Option<Rc<RefCell<File>>> = if args.no_log {
        println!("not logging to a file");
        None
    } else {
        // A pa3ds replaced is the same session to the player, so its log carries on.
        let path = log_path(args.log_path.as_deref(), !replaced)?;
        println!("logging to {}", path.display());
        let file = if replaced {
            File::options().append(true).create(true).open(&path)?
        } else {
            File::create(&path)?
        };
        Some(Rc::new(RefCell::new(file)))
    };
    let started = time::monotonic();
    let deadline = args.seconds.map(|seconds| started + seconds);
    let rpc: Rc<dyn Rpc> = Rc::new(UdpRpc::connect(args.port)?);

    let log: Log = {
        let log_file = log_file.clone();
        Rc::new(move |line: &str| {
            let stamped = format!("{:8.2} {line}", time::monotonic() - started);
            println!("{stamped}");
            if let Some(file) = &log_file {
                let mut file = file.borrow_mut();
                let _ = writeln!(file, "{stamped}");
                let _ = file.flush();
            }
        })
    };

    if replaced {
        log("replaced the pa3ds that was running, whose log this carries on");
    }
    let beside = instance::Beside::new(log.clone(), instance::LOOK);

    // A key's answer always interrupts (ADR-0010): `interrupt` adds menus to it. Until
    // 2026-09-24 this passed menus or nothing, and so no key ever cut speech off.
    let mut interrupts = speech::default_interrupts();
    if ui.interrupt {
        interrupts.insert(Category::Menu);
    }
    // Quiet while another window is in front, so reading it is never talked over (ADR-0021).
    let listening: Option<speech::Listening> = if ui.unfocused {
        None
    } else {
        Some(hooks.in_front.clone())
    };
    let speech = Rc::new(Speech::new(backend, interrupts, listening));
    // Before the game, not after: what a key does and which buttons it costs is about the
    // keyboard, and there is nothing to wait for. A patched binding needs the emulator
    // restarted anyway, so the player wants to hear about it before they start playing.
    // One sentence for both files: a second warning would interrupt the first.
    for note in settings.notes.iter().chain(&asked) {
        log(note);
    }
    let mut said = vec![settings.said.clone()];
    let mut warning = settings.warning;
    let mut bindings = None;
    if settings.general.keys && !args.no_keys {
        let explained: IndexMap<String, String> = EXPLAINED
            .iter()
            .map(|(key, why)| (key.to_string(), why.to_string()))
            .collect();
        let keys = keyfile::prepare(
            &DEFAULT_BINDINGS,
            None,
            Some(&settings.emulator),
            true,
            running,
            Some(&explained),
        )?;
        for note in &keys.notes {
            log(note);
        }
        said.push(keys.said);
        warning = warning || keys.warning;
        bindings = Some(keys.bindings);
    }
    let sentence = said
        .iter()
        .filter(|part| !part.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(". ");
    if !sentence.is_empty() {
        speech.say(
            &sentence,
            if warning {
                Category::Warning
            } else {
                Category::System
            },
        );
    }

    // Out of sight while Azahar is open, and closed with it (ADR-0039).
    let until =
        move || deadline.is_some_and(|deadline| time::monotonic() >= deadline) || beside.closed();
    (hooks.run)(
        rpc,
        speech,
        Settings {
            tick: 1.0 / settings.general.poll_rate,
            ui,
            navigation: settings.navigation.clone(),
            extras: settings.extras.clone(),
            keys: bindings,
            names_home: Some(keyfile::home_path()?),
            rom: args.rom,
            restarted: replaced,
            emulator: Some(settings.emulator.clone()),
            ..Settings::default()
        },
        &until,
        log,
    )
}

/// Ask for the folder of a portable Azahar, whose settings pa3ds could not find, and record
/// it in the settings file. The settings again, read with the folder, once it is recorded; the
/// same ones when the player cancels. What was asked and answered goes to `notes`, for the log
/// that does not exist yet.
fn ask_for_azahar(settings: config::Config, notes: &mut Vec<String>) -> py::Result<config::Config> {
    let mut title = "Azahar's settings were not found. Choose the folder azahar.exe is in, or \
                     cancel if Azahar is installed"
        .to_string();
    loop {
        println!("{title}");
        let Some(chosen) = choose_folder(&title, notes)? else {
            notes.push("no Azahar folder chosen".to_string());
            return Ok(settings);
        };
        notes.push(format!("Azahar folder chosen: {}", py::path_str(&chosen)));
        let found = match keyfile::portable(&chosen) {
            Ok(found) => found,
            Err(found) => {
                title = format!("{found}, choose another folder or cancel");
                notes.push(found);
                continue;
            }
        };
        let path = py::path_str(&config::config_path(None)?);
        if !config::remember_azahar(&found, None)? {
            notes.push(format!(
                "{path} could not be written, so {} is not kept",
                py::path_str(&found)
            ));
            return Ok(settings);
        }
        notes.push(format!(
            "recorded {} as azahar_folder in {path}",
            py::path_str(&found)
        ));
        let again = config::prepare(None, None, true, false)?;
        return Ok(config::Config {
            notes: settings.notes.into_iter().chain(again.notes).collect(),
            ..again
        });
    }
}

/// A Windows folder picker, which a screen reader reads out with the title; None when the
/// player cancels or no picker can open.
fn choose_folder(title: &str, notes: &mut Vec<String>) -> py::Result<Option<PathBuf>> {
    let app = keyfile::app_path()?;
    let initial = app.parent().unwrap_or(&app);
    Ok(match picker::choose(title, initial) {
        Ok(chosen) => chosen,
        Err(error) => {
            notes.push(format!("no folder picker could open: {error}"));
            None
        }
    })
}

/// Tk's `askdirectory`, which on Windows is the shell's `IFileOpenDialog` picking folders: the
/// same dialog, reached through COM. windows-sys declares no COM interfaces, so the two used are
/// declared here, each vtable in `shobjidl_core.h`'s order up to the last slot called.
#[cfg(windows)]
mod picker {
    use std::ffi::{OsString, c_void};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::path::{Path, PathBuf};
    use std::ptr::{null, null_mut};

    use windows_sys::Win32::Foundation::{ERROR_CANCELLED, HWND};
    use windows_sys::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
        CoInitializeEx, CoTaskMemFree, CoUninitialize,
    };
    use windows_sys::Win32::UI::Shell::{
        FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, FileOpenDialog,
        SHCreateItemFromParsingName, SIGDN_FILESYSPATH,
    };
    use windows_sys::core::{GUID, HRESULT, PCWSTR, PWSTR};

    const IID_IFILEOPENDIALOG: GUID = GUID::from_u128(0xd57c7288_d4ad_4768_be02_9d969532d960);
    const IID_ISHELLITEM: GUID = GUID::from_u128(0x43826d1e_e718_42ee_bc55_a1e261c37bfe);
    /// `HRESULT_FROM_WIN32(ERROR_CANCELLED)`, what `Show` returns when the player cancels.
    const CANCELLED: HRESULT = (0x8007_0000 | ERROR_CANCELLED) as HRESULT;

    type This = *mut c_void;

    #[repr(C)]
    struct UnknownVtbl {
        _query_interface: usize,
        _add_ref: usize,
        release: unsafe extern "system" fn(This) -> u32,
    }

    #[repr(C)]
    struct FileDialogVtbl {
        _unknown: UnknownVtbl,
        show: unsafe extern "system" fn(This, HWND) -> HRESULT,
        _set_file_types: usize,
        _set_file_type_index: usize,
        _get_file_type_index: usize,
        _advise: usize,
        _unadvise: usize,
        set_options: unsafe extern "system" fn(This, u32) -> HRESULT,
        get_options: unsafe extern "system" fn(This, *mut u32) -> HRESULT,
        _set_default_folder: usize,
        set_folder: unsafe extern "system" fn(This, This) -> HRESULT,
        _get_folder: usize,
        _get_current_selection: usize,
        _set_file_name: usize,
        _get_file_name: usize,
        set_title: unsafe extern "system" fn(This, PCWSTR) -> HRESULT,
        _set_ok_button_label: usize,
        _set_file_name_label: usize,
        get_result: unsafe extern "system" fn(This, *mut This) -> HRESULT,
    }

    #[repr(C)]
    struct ShellItemVtbl {
        _unknown: UnknownVtbl,
        _bind_to_handler: usize,
        _get_parent: usize,
        get_display_name: unsafe extern "system" fn(This, i32, *mut PWSTR) -> HRESULT,
    }

    /// A COM object, released when dropped.
    struct Com(This);

    impl Com {
        /// # Safety
        /// `V` must be the vtable of the interface this pointer was asked for.
        unsafe fn vtbl<V>(&self) -> &V {
            unsafe { &**(self.0 as *const *const V) }
        }
    }

    impl Drop for Com {
        fn drop(&mut self) {
            unsafe { (self.vtbl::<UnknownVtbl>().release)(self.0) };
        }
    }

    /// COM for this thread, let go when dropped where this call is what took it.
    struct Apartment(bool);

    impl Drop for Apartment {
        fn drop(&mut self) {
            if self.0 {
                unsafe { CoUninitialize() };
            }
        }
    }

    fn check(result: HRESULT) -> Result<(), String> {
        if result < 0 {
            return Err(std::io::Error::from_raw_os_error(result).to_string());
        }
        Ok(())
    }

    fn wide(text: &std::ffi::OsStr) -> Vec<u16> {
        text.encode_wide().chain([0]).collect()
    }

    /// The folder the player picked, starting in `initial`; None when they cancel.
    pub fn choose(title: &str, initial: &Path) -> Result<Option<PathBuf>, String> {
        let flags = (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32;
        let _apartment = Apartment(unsafe { CoInitializeEx(null(), flags) } >= 0);
        let mut made: This = null_mut();
        check(unsafe {
            CoCreateInstance(
                &FileOpenDialog,
                null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_IFILEOPENDIALOG,
                &mut made,
            )
        })?;
        let dialog = Com(made);
        let calls = unsafe { dialog.vtbl::<FileDialogVtbl>() };
        let mut options = 0;
        check(unsafe { (calls.get_options)(dialog.0, &mut options) })?;
        let options = options | FOS_PICKFOLDERS | FOS_PATHMUSTEXIST | FOS_FORCEFILESYSTEM;
        check(unsafe { (calls.set_options)(dialog.0, options) })?;
        let title = wide(title.as_ref());
        check(unsafe { (calls.set_title)(dialog.0, title.as_ptr()) })?;
        // Tk starts wherever it can when the initial folder will not do, and so does this.
        let mut folder: This = null_mut();
        let path = wide(initial.as_os_str());
        let found = unsafe {
            SHCreateItemFromParsingName(path.as_ptr(), null_mut(), &IID_ISHELLITEM, &mut folder)
        };
        if found >= 0 {
            let folder = Com(folder);
            unsafe { (calls.set_folder)(dialog.0, folder.0) };
        }
        let shown = unsafe { (calls.show)(dialog.0, null_mut()) };
        if shown == CANCELLED {
            return Ok(None);
        }
        check(shown)?;
        let mut picked: This = null_mut();
        check(unsafe { (calls.get_result)(dialog.0, &mut picked) })?;
        let picked = Com(picked);
        let mut name: PWSTR = null_mut();
        check(unsafe {
            (picked.vtbl::<ShellItemVtbl>().get_display_name)(
                picked.0,
                SIGDN_FILESYSPATH,
                &mut name,
            )
        })?;
        let chosen = unsafe {
            let length = (0..).take_while(|&at| *name.add(at) != 0).count();
            let chosen = OsString::from_wide(std::slice::from_raw_parts(name, length));
            CoTaskMemFree(name as *const c_void);
            chosen
        };
        Ok(Some(PathBuf::from(chosen)))
    }
}

#[cfg(not(windows))]
mod picker {
    use std::path::{Path, PathBuf};

    pub fn choose(_title: &str, _initial: &Path) -> Result<Option<PathBuf>, String> {
        Ok(None)
    }
}

/// `<pa3ds>/user/logs/pa3ds.log`, among everything else the player owns (ADR-0032), in a folder
/// of its own so ten runs of it do not bury the keys file and the names (the user, 2026-09-24).
pub fn default_log_path() -> py::Result<PathBuf> {
    Ok(keyfile::home_path()?.join(LOG_FOLDER).join(LOG_NAME))
}

/// Where this run's log goes, with the runs before it moved aside to make room.
///
/// `where` is a file, or a folder to put `pa3ds.log` in; by default `default_log_path()`. The runs
/// before are kept beside it with a number before the extension, `pa3ds.1.log` the last to
/// `pa3ds.9.log`. A live session's log is what shows why something was said, or not said, when no
/// replay reproduces it, and a player testing from the start of the game restarts pa3ds more than
/// once. `rotate` off moves nothing, for a run carrying on the log of the pa3ds it replaced.
pub fn log_path(place: Option<&Path>, rotate: bool) -> py::Result<PathBuf> {
    let mut path = match place {
        None => default_log_path()?,
        Some(place) => place.to_path_buf(),
    };
    if path.is_dir() {
        path = path.join(LOG_NAME);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if rotate {
        self::rotate(&path)?;
    }
    Ok(path)
}

/// `RotatingFileHandler(path, backupCount=KEPT_LOGS - 1).doRollover()`, with the number put
/// before the extension: each kept run moves one number up, the oldest is dropped, and the last
/// run becomes `.1`.
fn rotate(path: &Path) -> py::Result<()> {
    let numbered = |n: usize| -> PathBuf {
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name = match path.extension() {
            Some(extension) => format!("{stem}.{n}.{}", extension.to_string_lossy()),
            None => format!("{stem}.{n}"),
        };
        path.with_file_name(name)
    };
    for n in (1..KEPT_LOGS - 1).rev() {
        let (source, target) = (numbered(n), numbered(n + 1));
        if source.exists() {
            if target.exists() {
                std::fs::remove_file(&target)?;
            }
            std::fs::rename(&source, &target)?;
        }
    }
    let first = numbered(1);
    if first.exists() {
        std::fs::remove_file(&first)?;
    }
    if path.exists() {
        std::fs::rename(path, &first)?;
    }
    Ok(())
}
