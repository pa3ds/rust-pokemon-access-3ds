//! The settings file: pa3ds's flags as a file, and Azahar's own settings patched in (ADR-0026).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::test_app::ENVIRONMENT;
use crate::core::config::{self, General, Navigation, Table, Ui, Value};
use crate::core::keyfile;
use crate::core::toml as toml_text;
use crate::core::words::own;
use crate::py;

/// pytest's `tmp_path`: a folder of this test's own, gone when the test is.
struct TmpPath(PathBuf);

impl TmpPath {
    fn new() -> Self {
        static MADE: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "pa3ds-test-config-{}-{}",
            std::process::id(),
            MADE.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path).unwrap();
        TmpPath(path)
    }
}

impl Drop for TmpPath {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for TmpPath {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

/// A fresh Azahar's settings, the ones pa3ds ships a value for, CRLF as Azahar writes them.
fn emulator_text() -> String {
    [
        "[Debugging]",
        "enable_rpc_server\\default=true",
        "enable_rpc_server=false",
        "[Miscellaneous]",
        "log_filter\\default=true",
        "log_filter=*:Info",
        "[Renderer]",
        "graphics_api\\default=true",
        "graphics_api=1",
        "async_shader_compilation\\default=true",
        "async_shader_compilation=false",
        "shaders_accurate_mul\\default=true",
        "shaders_accurate_mul=true",
        "use_vsync\\default=true",
        "use_vsync=true",
        "",
    ]
    .join("\r\n")
}

fn emulator(tmp_path: &Path) -> PathBuf {
    let path = tmp_path.join("qt-config.ini");
    fs::write(&path, emulator_text()).unwrap();
    path
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

fn prepare(home: &Path, emulator: &Path, running: bool) -> config::Config {
    config::prepare(Some(home), Some(emulator), true, running).unwrap()
}

/// `dataclasses.asdict(kind())`, as the table a parser reads it back as.
fn as_table<T: Table>() -> toml::Value {
    toml::Value::Table(
        T::default()
            .fields()
            .into_iter()
            .map(|(name, value)| {
                let value = match value {
                    Value::Bool(value) => toml::Value::Boolean(value),
                    Value::Int(value) => toml::Value::Integer(value),
                    Value::Float(value) => toml::Value::Float(value),
                    Value::Str(value) => toml::Value::String(value),
                };
                (name.to_string(), value)
            })
            .collect(),
    )
}

#[test]
fn test_the_first_run_writes_every_default_and_reads_them_back() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    let none = tmp_path.join("none.ini");
    let first = prepare(&home, &none, false);
    let written: toml::Table = read(&home.join("config.toml")).parse().unwrap();
    let ours = written["pa3ds"].as_table().unwrap();
    assert_eq!(ours["keys"], toml::Value::Boolean(true));
    assert_eq!(ours["poll_rate"], toml::Value::Integer(60));
    assert_eq!(ours["ui"], as_table::<Ui>());
    assert_eq!(ours["navigation"], as_table::<Navigation>());
    assert_eq!(
        ours["extras"],
        toml::Value::Table(toml::Table::from_iter([
            ("conditions".to_string(), toml::Value::Boolean(false)),
            (
                "hidden_items".to_string(),
                toml::Value::String("off".to_string())
            ),
        ]))
    );
    let shipped: toml::Table = config::azahar()
        .into_iter()
        .map(|(name, (default, _))| {
            let default = match default {
                Value::Bool(default) => toml::Value::Boolean(default),
                Value::Int(default) => toml::Value::Integer(default),
                Value::Float(default) => toml::Value::Float(default),
                Value::Str(default) => toml::Value::String(default),
            };
            (name.to_string(), default)
        })
        .collect();
    assert_eq!(written["azahar"], toml::Value::Table(shipped));
    let again = prepare(&home, &none, false);
    assert_eq!(
        (again.general, again.ui, again.navigation),
        (first.general, first.ui, first.navigation)
    );
    assert!(
        !again
            .notes
            .iter()
            .any(|note| note.contains("no setting") || note.contains("no ["))
    );
}

/// The RPC server above all: Azahar ships it off, and without it pa3ds hears nothing.
#[test]
fn test_a_fresh_azahar_gets_pa3ds_s_settings_with_their_flags_off() {
    let tmp_path = TmpPath::new();
    let ini = emulator(&tmp_path);
    let settings = prepare(&tmp_path.join("home"), &ini, false);
    let after = read(&ini);
    assert!(after.contains("enable_rpc_server\\default=false\r\nenable_rpc_server=true\r\n"));
    assert!(after.contains("graphics_api=2\r\n"));
    assert!(after.contains("log_filter=\"*:Info RPC_Server:Error\"\r\n"));
    assert_eq!(
        after.matches("\r\n").count(),
        emulator_text().matches("\r\n").count()
    );
    assert_eq!(settings.said, "Azahar settings applied");
    assert_eq!(prepare(&tmp_path.join("home"), &ini, false).said, "");
}

#[test]
fn test_azahar_open_is_left_alone() {
    let tmp_path = TmpPath::new();
    let ini = emulator(&tmp_path);
    let settings = prepare(&tmp_path.join("home"), &ini, true);
    assert_eq!(read(&ini), emulator_text());
    assert!(settings.said.contains("close Azahar") && settings.warning);
}

/// No qt-config.ini yet: nothing can be patched, and the RPC server stays off.
#[test]
fn test_an_azahar_never_started_is_said() {
    let tmp_path = TmpPath::new();
    let settings = prepare(&tmp_path.join("home"), &tmp_path.join("none.ini"), false);
    assert!(settings.said.contains("start Azahar once") && settings.warning);
}

#[test]
fn test_a_setting_of_the_wrong_kind_is_logged_and_left_at_its_default() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("config.toml"),
        "[pa3ds]\npoll_rate = 30\n\n[pa3ds.ui]\ninterrupt = true\npauses = \"yes\"\nwibble = 1\n\
         \n[pa3ds.navigation]\nturn_by_turn = 0\n",
    )
    .unwrap();
    let settings = prepare(&home, &tmp_path.join("none.ini"), false);
    assert_eq!(settings.general.poll_rate, 30.0);
    assert_eq!(
        settings.ui,
        Ui {
            interrupt: true,
            ..Ui::default()
        }
    );
    assert_eq!(settings.navigation, Navigation::default());
    assert!(settings.notes.iter().any(|note| note.contains("pauses")));
    assert!(settings.notes.iter().any(|note| note.contains("wibble")));
    assert!(
        settings
            .notes
            .iter()
            .any(|note| note.contains("turn_by_turn"))
    );
}

/// Until 2026-10-18 `[pa3ds.ui] tones` was on or off; `[pa3ds.audio]` takes how loud, and a
/// player's false is filled in there as 0 and their footsteps as they had them.
#[test]
fn test_the_tones_turned_off_stay_off_as_they_move_to_the_audio_table() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("config.toml"),
        "[pa3ds.ui]\ntones = false\nfootsteps = 0.5\n",
    )
    .unwrap();
    let none = tmp_path.join("none.ini");
    let settings = config::prepare(Some(&home), Some(&none), true, false).unwrap();
    assert_eq!(
        settings.audio,
        config::Audio {
            tones: 0.0,
            footsteps: 0.5,
            ticks: 0.0
        }
    );
    assert!(
        settings
            .notes
            .iter()
            .any(|note| note.contains("[pa3ds.ui] tones is [pa3ds.audio] tones now")),
        "{:?}",
        settings.notes
    );
    let text = read_text(&home.join("config.toml"));
    assert!(text.contains("\ntones = 0\n") && text.contains("\nfootsteps = 0.5\n"));
    assert_eq!(
        config::prepare(Some(&tmp_path.join("new")), Some(&none), true, false)
            .unwrap()
            .audio,
        config::Audio {
            tones: 1.0,
            footsteps: 1.0,
            ticks: 0.0
        }
    );
}

#[test]
fn test_an_extra_is_on_only_when_asked_for() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("config.toml"),
        "[pa3ds.extras]\nconditions = true\n",
    )
    .unwrap();
    assert!(
        prepare(&home, &tmp_path.join("none.ini"), false)
            .extras
            .conditions
    );
    let other = config::prepare(
        Some(&tmp_path.join("other")),
        Some(&tmp_path.join("none.ini")),
        false,
        false,
    )
    .unwrap();
    assert!(!other.extras.conditions);
}

/// The first file was flat, `positions = true` at the top; it is never overwritten.
#[test]
fn test_a_file_from_before_the_tables_says_what_it_has_lost() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(home.join("config.toml"), "positions = true\n").unwrap();
    let settings = prepare(&home, &tmp_path.join("none.ini"), false);
    assert_eq!(settings.ui, Ui::default());
    assert!(
        settings
            .notes
            .iter()
            .any(|note| note.contains("no [positions] table"))
    );
}

#[test]
fn test_a_line_commented_out_of_azahar_s_table_leaves_that_setting_alone() {
    // A line deleted comes back at its default, so a # is how a player keeps Azahar's own.
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("config.toml"),
        "[azahar]\nuse_vsync = false\n# graphics_api = 2\n",
    )
    .unwrap();
    let ini = emulator(&tmp_path);
    prepare(&home, &ini, false);
    let after = read(&ini);
    assert!(after.contains("use_vsync=false"));
    assert!(after.contains("graphics_api=1\r\n"));
    assert!(after.contains("enable_rpc_server=true\r\n"));
}

#[test]
fn test_a_setting_pa3ds_gained_is_added_and_nothing_else_changes() {
    // Orion, 2026-10-17: players should not have to delete their file to get a new setting. It
    // goes in at its default, explained, at the end of its table.
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    let full = config::render().unwrap();
    let gained = full.find("\n\n# List mode's list").unwrap();
    let line = full.find("sort_by_distance = ").unwrap();
    let end = line + full[line..].find('\n').unwrap();
    let older = format!("{}{}", &full[..gained], &full[end..]);
    let older = older.replace("pauses = false", "pauses = true");
    fs::write(home.join("config.toml"), older).unwrap();
    let settings = prepare(&home, &tmp_path.join("none.ini"), false);
    assert_eq!(
        read(&home.join("config.toml")).replace("\r\n", "\n"),
        full.replace("pauses = false", "pauses = true")
    );
    assert!(settings.ui.pauses);
    assert!(
        settings
            .notes
            .iter()
            .any(|note| note.ends_with("[pa3ds.navigation] sort_by_distance"))
    );
    assert!(!settings.said.contains("could not be read"));
}

#[test]
fn test_a_file_that_will_not_parse_is_said_and_left_alone() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(home.join("config.toml"), "[pa3ds.ui]\npauses = yes\n").unwrap();
    let settings = prepare(&home, &emulator(&tmp_path), false);
    assert!(settings.warning);
    assert!(settings.said.starts_with("config.toml could not be read: "));
    assert!(settings.said.contains("line 2"), "{}", settings.said);
    assert_eq!(
        read(&home.join("config.toml")),
        "[pa3ds.ui]\npauses = yes\n"
    );
}

#[test]
fn test_a_name_every_table_has_and_no_setting_is_logged() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(home.join("config.toml"), "[pa3ds]\n__doc__ = \"x\"\n").unwrap();
    let settings = prepare(&home, &tmp_path.join("none.ini"), false);
    assert_eq!(settings.general, General::default());
    assert!(settings.notes.iter().any(|note| note.contains("__doc__")));
}

#[test]
fn test_files_that_are_not_utf_8_read_as_nothing() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(home.join("config.toml"), b"[pa3ds]\nkeys = false # \xff\n").unwrap();
    let ini = tmp_path.join("qt-config.ini");
    fs::write(&ini, b"enable_rpc_server=\xff\n").unwrap();
    let settings = config::prepare(Some(&home), Some(&ini), false, false).unwrap();
    assert_eq!(settings.general, General::default());
    assert!(
        settings
            .notes
            .iter()
            .any(|note| note.contains("could not be read"))
    );
}

/// `Path.read_text(encoding="utf-8")`, which reads with universal newlines.
fn read_text(path: &Path) -> String {
    read(path).replace("\r\n", "\n").replace('\r', "\n")
}

fn quoted(folder: &Path) -> String {
    toml_text::quote(&py::path_str(folder))
}

#[test]
fn test_a_portable_azahar_s_folder_is_where_its_settings_are_patched() {
    let tmp_path = TmpPath::new();
    let folder = tmp_path.join("Azahar");
    let ini = folder.join("user").join("config").join("qt-config.ini");
    fs::create_dir_all(ini.parent().unwrap()).unwrap();
    fs::write(&ini, emulator_text()).unwrap();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("config.toml"),
        format!("[pa3ds]\nazahar_folder = {}\n", quoted(&folder)),
    )
    .unwrap();
    let settings = config::prepare(Some(&home), None, true, false).unwrap();
    assert_eq!(settings.emulator, ini);
    assert!(read(&ini).contains("enable_rpc_server=true\r\n"));
    assert_eq!(settings.said, "Azahar settings applied");
}

/// A player with pa3ds in the folder of a portable Azahar and an older Azahar's settings
/// left in AppData: those were read, and no ROM was found among the ones they list (#14). The
/// running Azahar's own folder comes first, wherever pa3ds is.
#[test]
fn test_a_portable_azahar_beside_pa3ds_is_found_over_an_old_appdata_config() {
    fn azahar(folder: &Path) -> PathBuf {
        let ini = folder.join("user").join("config").join("qt-config.ini");
        fs::create_dir_all(ini.parent().unwrap()).unwrap();
        fs::write(&ini, emulator_text()).unwrap();
        fs::write(folder.join("azahar.exe"), b"").unwrap();
        ini
    }

    let tmp_path = TmpPath::new();
    let beside = azahar(&tmp_path.join("Azahar"));
    azahar(&tmp_path.join("appdata").join("Azahar"));
    let _environment = ENVIRONMENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let appdata = std::env::var_os("APPDATA");
    // SAFETY: every test that touches the environment holds `ENVIRONMENT`.
    unsafe { std::env::set_var("APPDATA", tmp_path.join("appdata")) };
    let home = tmp_path.join("home");
    keyfile::LOOKED_IN.with(|looked_in| {
        *looked_in.borrow_mut() = Some([None, Some(tmp_path.join("Azahar"))]);
    });
    let found = config::prepare(Some(&home), None, true, false).unwrap();
    let running = azahar(&tmp_path.join("elsewhere"));
    keyfile::LOOKED_IN.with(|looked_in| {
        *looked_in.borrow_mut() = Some([
            Some(tmp_path.join("elsewhere")),
            Some(tmp_path.join("Azahar")),
        ]);
    });
    let found_running = config::prepare(Some(&home), None, true, false).unwrap();
    keyfile::LOOKED_IN.with(|looked_in| *looked_in.borrow_mut() = None);
    // SAFETY: as above.
    unsafe {
        match appdata {
            Some(appdata) => std::env::set_var("APPDATA", appdata),
            None => std::env::remove_var("APPDATA"),
        }
    }
    assert_eq!(found.emulator, beside);
    assert_eq!(found_running.emulator, running);
}

#[test]
fn test_choosing_a_folder_sets_it_and_changes_nothing_else() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    prepare(&home, &tmp_path.join("none.ini"), false);
    let path = home.join("config.toml");
    let before = read_text(&path);
    let first = tmp_path.join("Azahar One");
    assert!(config::remember_azahar(&first, Some(&home)).unwrap());
    assert_eq!(
        read_text(&path),
        before.replace(
            "azahar_folder = \"\"",
            &format!("azahar_folder = {}", quoted(&first))
        )
    );
    let second = PathBuf::from(r#"C:\Games\Azahar "2""#);
    assert!(config::remember_azahar(&second, Some(&home)).unwrap());
    assert_eq!(
        read_text(&path),
        before.replace(
            "azahar_folder = \"\"",
            &format!("azahar_folder = {}", quoted(&second))
        )
    );
    assert_eq!(
        prepare(&home, &tmp_path.join("none.ini"), false)
            .general
            .azahar_folder,
        py::path_str(&second)
    );
}

#[test]
fn test_choosing_a_folder_brings_back_a_line_commented_out() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("config.toml"),
        "[pa3ds]\nkeys = true\n# azahar_folder = \"\"\n\n[pa3ds.ui]\npauses = true\n",
    )
    .unwrap();
    assert!(config::remember_azahar(&tmp_path, Some(&home)).unwrap());
    assert_eq!(
        read_text(&home.join("config.toml")),
        format!(
            "[pa3ds]\nkeys = true\nazahar_folder = {}\n\n[pa3ds.ui]\npauses = true\n",
            quoted(&tmp_path)
        )
    );
}

/// Applying writes the settings changed and keeps the player's comments, order and lines
/// commented out (docs/plans/pa3ds-window.md), a comment after a value on its line too, but not
/// a # inside a value; a setting the file lacks joins its table.
#[test]
fn test_the_settings_dialog_changes_each_setting_on_its_own_line_and_nothing_else() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("config.toml"),
        "# mine\n[pa3ds]\npoll_rate = 30 # slow laptop\nazahar_folder = \"C:/#games\"\n\n\
         [pa3ds.ui]\n# pauses = true\nlanguage = \"fr\"\n\n[pa3ds.audio]\ntones = 1\n",
    )
    .unwrap();
    let settings = [
        ("pa3ds.poll_rate", Value::Float(60.0)),
        ("pa3ds.azahar_folder", Value::Str("D:/Azahar".to_string())),
        ("pa3ds.ui.pauses", Value::Bool(true)),
        ("pa3ds.audio.footsteps", Value::Float(0.5)),
    ]
    .map(|(name, value)| (name.to_string(), value));
    assert!(config::change(&settings, Some(&home)).unwrap());
    assert_eq!(
        read_text(&home.join("config.toml")),
        "# mine\n[pa3ds]\npoll_rate = 60 # slow laptop\nazahar_folder = \"D:/Azahar\"\n\n\
         [pa3ds.ui]\npauses = true\nlanguage = \"fr\"\n\n[pa3ds.audio]\ntones = 1\n\
         footsteps = 0.5\n"
    );
}

#[test]
fn test_a_line_separator_in_a_comment_does_not_end_the_line() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    // TOML ends a line only at LF or CRLF, so this is one comment and no setting commented out.
    let before = "[pa3ds]\n# azahar_folder = \"\"\u{2028}kept apart\nkeys = true\n";
    fs::write(home.join("config.toml"), before).unwrap();
    assert!(config::remember_azahar(&tmp_path, Some(&home)).unwrap());
    let text = read_text(&home.join("config.toml"));
    assert_eq!(
        text,
        format!("{before}azahar_folder = {}\n", quoted(&tmp_path))
    );
    let parsed: toml::Table = text.parse().unwrap();
    assert_eq!(
        parsed["pa3ds"]["azahar_folder"].as_str(),
        Some(py::path_str(&tmp_path).as_str())
    );
}

#[test]
fn test_a_folder_is_not_kept_in_a_file_that_will_not_parse() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(home.join("config.toml"), "[pa3ds.ui]\npauses = yes\n").unwrap();
    assert!(!config::remember_azahar(&tmp_path, Some(&home)).unwrap());
    assert_eq!(
        read_text(&home.join("config.toml")),
        "[pa3ds.ui]\npauses = yes\n"
    );
}

/// The settings dialog names each setting and describes it, and the file explains it, all in
/// pa3ds's own words (docs/plans/pa3ds-window.md): a message missing is said as its id.
#[test]
fn test_every_setting_has_a_name_and_says_what_it_does_in_english() {
    for table in config::TABLES {
        for (name, _) in config::defaults(table) {
            let message = config::message(&format!("{table}.{name}"));
            assert_ne!(own(&message, &[]), message);
            assert!(own(&format!("{message}-about"), &[]).ends_with('.'));
        }
    }
}
