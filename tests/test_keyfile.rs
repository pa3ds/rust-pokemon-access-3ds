//! The keys file: what pa3ds takes, what it asks the emulator to bind, and what it says about it.
//!
//! The config here is the shape of a real `qt-config.ini`, cut down: the profile chooser, a few
//! buttons with their `\default` flags, and one stick with the escapes Azahar writes inside a
//! nested binding. It is written CRLF, as Azahar writes it, because keeping a file's own line
//! endings is part of what patching has to get right.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use indexmap::IndexMap;

use crate::core::keyfile::{self, Asked, Bindings, Wanted};
use crate::core::keys::{Released, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, bound, modified, track};

const LIST: &str = "list";
const DIRECTION: &str = "direction";

/// pytest's `tmp_path`: a folder of this test's own, gone when the test is.
struct TmpPath(PathBuf);

impl TmpPath {
    fn new() -> Self {
        static MADE: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "pa3ds-test-keyfile-{}-{}",
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

fn mode(keys: &[(&str, &str)]) -> IndexMap<String, String> {
    keys.iter()
        .map(|(key, action)| (key.to_string(), action.to_string()))
        .collect()
}

fn modes(of: &[(&str, &[(&str, &str)])]) -> Bindings {
    of.iter()
        .map(|(name, keys)| (name.to_string(), mode(keys)))
        .collect()
}

/// The apostrophe means the same in both modes, so the file writes it once above them.
fn bindings() -> Bindings {
    modes(&[
        (
            LIST,
            &[
                ("'", "mode"),
                ("i", "category previous"),
                ("k", "item read"),
            ],
        ),
        (
            DIRECTION,
            &[
                ("'", "mode"),
                ("o", "north"),
                (";", "east"),
                ("shift+k", "rename"),
            ],
        ),
    ])
}

fn config() -> String {
    [
        "[Controls]",
        "profile\\default=true",
        "profile=0",
        "profiles\\1\\button_a\\default=true",
        "profiles\\1\\button_a=\"code:65,engine:keyboard\"",
        "profiles\\1\\button_start\\default=true",
        "profiles\\1\\button_start=\"code:77,engine:keyboard\"",
        "profiles\\1\\button_select\\default=true",
        "profiles\\1\\button_select=\"code:78,engine:keyboard\"",
        "profiles\\1\\button_debug\\default=true",
        "profiles\\1\\button_debug=\"code:79,engine:keyboard\"",
        "profiles\\1\\c_stick\\default=false",
        concat!(
            "profiles\\1\\c_stick=\"code$073$1engine$0keyboard:maptype$0guid+port,down:code$075$1",
            "engine$0keyboard,engine:analog_from_button,left:code$074$1engine$0keyboard,right:",
            "code$076$1engine$0keyboard,up:code$073$1engine$0keyboard\""
        ),
        "profiles\\1\\touch_device\\default=true",
        "profiles\\1\\touch_device=engine:emu_window",
        "",
    ]
    .join("\r\n")
}

fn written(tmp_path: &Path) -> PathBuf {
    let path = tmp_path.join("qt-config.ini");
    fs::write(&path, config()).unwrap();
    path
}

/// The file exactly as it is on disk, CRLF and all: reading it any other way hides whether
/// patching kept the line endings.
fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

/// `Path.read_text(encoding="utf-8")`, whose universal newlines read a Windows file's CRLF as LF.
fn read_text(path: &Path) -> String {
    read(path).replace("\r\n", "\n").replace('\r', "\n")
}

fn prepare(
    defaults: &Bindings,
    home: &Path,
    config: &Path,
    write: bool,
    running: bool,
    explained: Option<&IndexMap<String, String>>,
) -> keyfile::Keys {
    keyfile::prepare(
        defaults,
        Some(home),
        Some(config),
        write,
        running,
        explained,
    )
    .unwrap()
}

fn stick_value(text: &str) -> String {
    text.split("c_stick=")
        .nth(1)
        .unwrap()
        .split("\r\n")
        .next()
        .unwrap()
        .trim_matches('"')
        .to_string()
}

fn key(name: &str, action: &str) -> Wanted {
    Wanted::from([(name.to_string(), Asked::Key(action.to_string()))])
}

fn stick(name: &str, settings: &[(&str, &str)]) -> Wanted {
    Wanted::from([(name.to_string(), Asked::Stick(mode(settings)))])
}

// ---------------------------------------------------------------- key codes

/// Azahar records `event->key()`, so `code:` is a Qt key code. Letters and digits are the
/// same number in both, which is why nothing noticed; the semicolon is 59 to Qt and 186 to
/// Windows, and an arrow key is not in the virtual-key range at all.
#[test]
fn test_a_key_is_numbered_the_way_qt_numbers_it_not_the_way_windows_does() {
    assert_eq!(keyfile::qt_code("a").unwrap(), Some(0x41));
    assert_eq!(keyfile::qt_code("1").unwrap(), Some(0x31));
    assert_eq!(keyfile::qt_code(";").unwrap(), Some(0x3B));
    assert_eq!(keyfile::qt_code("'").unwrap(), Some(0x27));
    assert_eq!(keyfile::qt_code("left").unwrap(), Some(16777234));
    assert_eq!(keyfile::qt_code("return").unwrap(), Some(16777220));
}

#[test]
fn test_a_name_no_key_has_is_no_code_at_all() {
    assert_eq!(keyfile::qt_code("wibble").unwrap(), None);
    assert_eq!(keyfile::qt_code("€").unwrap(), None);
    assert_eq!(keyfile::qt_code("").unwrap(), None);
}

/// Qt's `Key_ssharp` is 0xdf; upper case, "ß" is two letters.
#[test]
fn test_a_letter_with_no_single_capital_is_numbered_by_itself() {
    assert_eq!(keyfile::qt_code("ß").unwrap(), Some(0xDF));
}

#[test]
fn test_a_code_that_is_digits_but_no_number_is_no_key() {
    let params = IndexMap::from([
        ("engine".to_string(), "keyboard".to_string()),
        ("code".to_string(), "²".to_string()),
    ]);
    assert_eq!(keyfile::keyboard_code(&params).unwrap(), None);
}

#[test]
fn test_a_stick_scale_that_is_not_a_number_is_written_unknown() {
    let sticks = IndexMap::from([(
        "circle_pad".to_string(),
        IndexMap::from([("modifier_scale".to_string(), "half".to_string())]),
    )]);
    let rendered = keyfile::render(&bindings(), &IndexMap::new(), Some(&sticks), None).unwrap();
    assert!(rendered.contains("#   modifier_scale = ?"));
}

#[test]
fn test_every_code_reads_back_as_the_name_it_was_written_from() {
    for name in [
        "a", "z", "0", "9", ";", "'", "[", "]", "/", "left", "down", "enter", "space",
    ] {
        let code = keyfile::qt_code(name).unwrap();
        assert!(code.is_some(), "{name}");
        assert_eq!(keyfile::qt_name(code.unwrap()), name);
    }
}

/// Qt calls it Return and the keypad's Enter; the hook calls it enter, as the player does, and a
/// key pa3ds takes is compared with Azahar's by that name (touch mode's tap, 2026-10-01).
#[test]
fn test_the_main_enter_key_is_enter_as_pa3ds_s_hook_calls_it() {
    assert_eq!(keyfile::qt_code("return").unwrap(), Some(0x01000004));
    assert_eq!(keyfile::qt_code("enter").unwrap(), Some(0x01000004));
    assert_eq!(keyfile::qt_name(0x01000005), "keypad enter");
}

#[test]
fn test_a_code_nothing_knows_is_said_as_itself() {
    assert_eq!(keyfile::qt_name(0x01FFFFFF), "code 33554431");
}

// ---------------------------------------------------------------- reading the emulator's config

#[test]
fn test_the_emulator_s_own_bindings_are_read_buttons_and_sticks_alike() {
    let found = keyfile::emulator_keys(&config()).unwrap();
    assert_eq!(found["button_a"], ["a"]);
    assert_eq!(found["button_start"], ["m"]);
    assert_eq!(found["c_stick"], ["i", "k", "j", "l"]); // up, down, left, right
}

/// `touch_device=engine:emu_window` has no keyboard behind it, and a binding on a gamepad
/// costs the player no key.
#[test]
fn test_a_control_that_is_not_on_the_keyboard_is_left_out() {
    assert!(
        !keyfile::emulator_keys(&config())
            .unwrap()
            .contains_key("touch_device")
    );
}

#[test]
fn test_the_profile_setting_counts_from_zero_and_the_sections_count_from_one() {
    let other = config().replace("profile=0", "profile=1");
    assert_eq!(
        keyfile::emulator_keys(&other).unwrap(),
        IndexMap::<String, Vec<String>>::new()
    );
}

// ---------------------------------------------------------------- what pa3ds takes

/// The whole point: M was bound to the navigation mode and M is Start, so the player could
/// not open the game's menu while pa3ds was running.
#[test]
fn test_a_key_pa3ds_takes_is_named_with_the_button_it_costs() {
    let emulator = keyfile::emulator_keys(&config()).unwrap();
    let over = keyfile::taken(&modes(&[(LIST, &[("m", "mode")])]), &emulator);
    assert_eq!(over, [("Start".to_string(), "m".to_string(), false)]);
    assert!(keyfile::taken(&bindings(), &emulator).contains(&(
        "the C-stick".to_string(),
        "i".to_string(),
        true
    )));
}

/// The hook swallows shift and T, not T: touch mode is shift and T so that T stays the D-pad
/// (Orion, 2026-10-01), and saying at every start that it sits on the D-pad would be untrue
/// whenever shift is up. Until then `shift+k` was said to take the C-stick's K.
#[test]
fn test_a_key_with_a_modifier_takes_its_button_only_while_the_modifier_is_held() {
    let emulator = keyfile::emulator_keys(&config()).unwrap();
    assert_eq!(
        keyfile::taken(&modes(&[(LIST, &[("shift+k", "rename")])]), &emulator),
        []
    );
    assert_eq!(
        keyfile::taken(&modes(&[(LIST, &[("k", "item read")])]), &emulator),
        [("the C-stick".to_string(), "k".to_string(), true)]
    );
}

/// Taking the C-stick or the debug button costs nothing in this game, so saying it at every
/// start would be noise. Taking Start is worth hearing.
#[test]
fn test_the_controls_oras_never_uses_are_logged_but_not_said() {
    let tmp_path = TmpPath::new();
    let quiet = prepare(
        &modes(&[(LIST, &[("i", "x"), ("o", "y")])]),
        &tmp_path.join("home"),
        &written(&tmp_path),
        true,
        false,
        None,
    );
    assert_eq!(quiet.said, "");
    assert!(quiet.notes.iter().any(|note| note.contains("the C-stick")));
    let loud = prepare(
        &modes(&[(LIST, &[("m", "mode")])]),
        &tmp_path.join("loud"),
        &written(&tmp_path),
        true,
        false,
        None,
    );
    assert_eq!(loud.said, "pa3ds takes Start");
}

// ---------------------------------------------------------------- patching

/// Azahar ignores a stored value while its `\default` flag is true and re-saves the file at
/// startup, which is how a frame limit of 1000 was silently ignored for an afternoon.
#[test]
fn test_a_button_is_bound_with_its_default_flag_turned_off() {
    let (patched, changed) = keyfile::patch(&config(), &key("start", "y")).unwrap();
    assert!(patched.contains("profiles\\1\\button_start=\"code:89,engine:keyboard\""));
    assert!(patched.contains("profiles\\1\\button_start\\default=false"));
    assert_eq!(changed, ["Start is now y"]);
}

#[test]
fn test_patching_keeps_the_file_s_own_line_endings() {
    let (patched, _) = keyfile::patch(&config(), &key("start", "y")).unwrap();
    assert!(!patched.replace("\r\n", "").contains('\n'));
}

/// Patching runs at every start, so it has to be idempotent — and a start that changed
/// nothing must not tell the player to restart the game.
#[test]
fn test_a_button_already_bound_the_way_it_is_asked_for_changes_nothing() {
    let (once, _) = keyfile::patch(&config(), &key("start", "y")).unwrap();
    let (twice, changed) = keyfile::patch(&once, &key("start", "y")).unwrap();
    assert_eq!(twice, once);
    assert_eq!(changed, Vec::<String>::new());
}

/// The value alone is not enough: with the flag true Azahar is not using the value.
#[test]
fn test_a_button_whose_flag_is_still_true_is_rewritten_even_at_the_same_value() {
    let (_, changed) = keyfile::patch(&config(), &key("start", "m")).unwrap();
    assert_eq!(changed, ["Start is now m"]);
}

#[test]
fn test_a_key_that_does_not_exist_is_said_rather_than_written() {
    let (patched, changed) = keyfile::patch(&config(), &key("start", "wibble")).unwrap();
    assert_eq!(patched, config());
    assert_eq!(changed, ["start: no key is called 'wibble'"]);
}

#[test]
fn test_a_button_the_config_does_not_have_is_said_rather_than_invented() {
    let (_, changed) = keyfile::patch(
        &config().replace("button_start", "button_gone"),
        &key("start", "y"),
    )
    .unwrap();
    assert_eq!(
        changed,
        ["start: no profiles\\1\\button_start in the config to patch"]
    );
}

// ---------------------------------------------------------------- the file itself

#[test]
fn test_the_first_run_writes_the_defaults_and_the_second_reads_them_back() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    let config = written(&tmp_path);
    let first = prepare(&bindings(), &home, &config, true, false, None);
    assert!(home.join("keys.toml").exists());
    assert_eq!(first.bindings, bindings());
    assert!(first.notes.iter().any(|note| note.contains("wrote")));
    let again = prepare(&bindings(), &home, &config, true, false, None);
    assert_eq!(again.bindings, bindings());
    assert!(!again.notes.iter().any(|note| note.contains("wrote")));
}

/// What `render` writes has to read back as what it was written from, or the first edit a
/// player makes silently changes their keys.
#[test]
fn test_the_file_written_on_the_first_run_is_the_defaults_and_nothing_else() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    let loaded: toml::Table = read(&home.join("keys.toml")).parse().unwrap();
    assert_eq!(loaded["azahar"], toml::Value::Table(toml::Table::new()));
    // The shared keys sit above the modes, and each mode holds only what is its own.
    let navigation = loaded["pa3ds"]["navigation"].as_table().unwrap();
    assert_eq!(navigation["'"], toml::Value::String("mode".to_string()));
    let table = |pairs: &[(&str, &str)]| {
        toml::Value::Table(
            pairs
                .iter()
                .map(|(key, value)| (key.to_string(), toml::Value::String(value.to_string())))
                .collect(),
        )
    };
    assert_eq!(
        navigation[LIST],
        table(&[("i", "category previous"), ("k", "item read")])
    );
    assert_eq!(
        navigation[DIRECTION],
        table(&[("o", "north"), (";", "east"), ("shift+k", "rename")])
    );
    // And reading it back gives what it was written from.
    let again = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert_eq!(again.bindings, bindings());
}

/// Orion, 2026-10-01: the keys file says the order the enemy side is said in.
#[test]
fn test_an_action_explained_is_explained_above_its_key() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    let explained = mode(&[("item read", "Says the item chosen again.")]);
    prepare(
        &bindings(),
        &home,
        &written(&tmp_path),
        true,
        false,
        Some(&explained),
    );
    let text = read_text(&home.join("keys.toml"));
    assert!(text.contains("# Says the item chosen again.\n\"k\" = \"item read\""));
    let again = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert_eq!(again.bindings, bindings());
}

#[test]
fn test_an_edit_the_player_makes_is_what_is_answered() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("keys.toml"),
        "[pa3ds.navigation.list]\nz = \"item read\"\n",
    )
    .unwrap();
    let keys = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert_eq!(keys.bindings[LIST]["z"], "item read");
    assert!(!keys.bindings[LIST].contains_key("k"));
}

/// Falling through to the navigator, a typo would not be quiet: an unknown action lands in
/// the branch that reads an item.
#[test]
fn test_an_action_pa3ds_does_not_do_is_said_and_dropped() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("keys.toml"),
        "[pa3ds.navigation.list]\nz = \"item reed\"\nk = \"item read\"\n",
    )
    .unwrap();
    let keys = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert!(!keys.bindings[LIST].contains_key("z"));
    assert_eq!(keys.bindings[LIST]["k"], "item read");
    assert!(
        keys.notes
            .iter()
            .any(|note| note.contains("'item reed', which pa3ds does not do"))
    );
}

/// Touch mode's drag is Shift alone's, and a key of the player's too where they bind one (Orion,
/// 2026-10-29); the file pa3ds writes binds no key to it.
#[test]
fn test_an_action_no_default_key_does_is_the_players_to_bind() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("keys.toml"),
        "[pa3ds.navigation.list]\nd = \"drag\"\n",
    )
    .unwrap();
    let keys = keyfile::prepare_with(
        &bindings(),
        Some(&home),
        Some(&written(&tmp_path)),
        true,
        false,
        None,
        &["drag"],
    )
    .unwrap();
    assert_eq!(keys.bindings[LIST]["d"], "drag");
    let fresh = tmp_path.join("fresh");
    fs::create_dir(&fresh).unwrap();
    keyfile::prepare_with(
        &bindings(),
        Some(&fresh),
        Some(&written(&tmp_path)),
        true,
        false,
        None,
        &["drag"],
    )
    .unwrap();
    assert!(!read_text(&fresh.join("keys.toml")).contains("drag"));
}

/// Windows repeats a key held down, which would pick a drag up and let it go over and over; a key
/// bound to drag is taken once and said as it comes up, and other keys repeat as ever.
#[test]
fn test_a_key_held_is_said_once_and_its_release_after() {
    let (d, j) = (0x44, 0x4A);
    let mut released = Released::new(["d", "shift+d"]);
    assert!(released.down(d, "d") && !released.down(d, "d"));
    assert!(released.down(j, "j") && released.down(j, "j"));
    assert!(released.up(j).is_empty() && released.up(d) == "d up" && released.up(d).is_empty());
    assert!(released.down(d, "shift+d") && released.up(d) == "shift+d up");
}

// ---------------------------------------------------------------- keys pa3ds gains

/// Orion, 2026-10-17, reopening ADR-0022: players should not have to delete their file to get a
/// new key. It goes after the table's last key, not after the comment that follows it.
#[test]
fn test_a_key_pa3ds_gained_is_added_to_its_table_and_nothing_else_changes() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    let mut older = bindings();
    older[DIRECTION].shift_remove("shift+k");
    let before = keyfile::render(
        &older,
        &keyfile::emulator_keys(&config()).unwrap(),
        None,
        None,
    )
    .unwrap();
    fs::write(home.join("keys.toml"), &before).unwrap();
    let keys = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    let after = read_text(&home.join("keys.toml"));
    assert_eq!(
        after,
        before.replace(
            "\";\" = \"east\"\n",
            "\";\" = \"east\"\n\"shift+k\" = \"rename\"\n"
        )
    );
    assert_eq!(keys.bindings, bindings());
    assert!(keys.said.is_empty() && !keys.warning);
    let again = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert!(!again.notes.iter().any(|note| note.contains("added")));
}

#[test]
fn test_a_key_every_mode_lacks_is_added_once_above_them() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("keys.toml"),
        "[pa3ds.navigation.list]\ni = \"category previous\"\nk = \"item read\"\n",
    )
    .unwrap();
    let keys = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    let loaded: toml::Table = read(&home.join("keys.toml")).parse().unwrap();
    let navigation = loaded["pa3ds"]["navigation"].as_table().unwrap();
    assert_eq!(navigation["'"], toml::Value::String("mode".to_string()));
    assert!(!navigation[LIST].as_table().unwrap().contains_key("'"));
    assert!(!navigation[DIRECTION].as_table().unwrap().contains_key("'"));
    assert_eq!(keys.bindings, bindings());
}

/// The player's shift and K is east, and pa3ds gains shift and K for rename: the player's key
/// keeps its meaning, the new one waits behind a #, and the player is told, once.
#[test]
fn test_a_new_key_on_a_key_the_player_has_is_added_commented_out_and_said() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("keys.toml"),
        concat!(
            "[pa3ds.navigation]\n\"'\" = \"mode\"\n\n[pa3ds.navigation.list]\n",
            "i = \"category previous\"\nk = \"item read\"\n\n[pa3ds.navigation.direction]\n",
            "o = \"north\"\n\"shift+k\" = \"east\"\n"
        ),
    )
    .unwrap();
    let keys = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    let text = read_text(&home.join("keys.toml"));
    assert!(text.ends_with("# \"shift+k\" = \"rename\"\n"), "{text}");
    assert!(text.contains("shift+k is already \"east\""));
    assert_eq!(keys.bindings[DIRECTION]["shift+k"], "east");
    assert!(
        !keys.bindings[DIRECTION]
            .values()
            .any(|action| action == "rename")
    );
    assert!(keys.warning);
    assert!(keys.said.contains(
        "new key in keys.toml commented out: shift+k for rename, because shift+k is already \
         east. edit keys.toml to choose"
    ));
    let again = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert!(again.said.is_empty() && !again.warning);
    assert_eq!(read_text(&home.join("keys.toml")), text);
}

/// Commenting a key out is how a player unbinds one, now that a key deleted comes back.
#[test]
fn test_a_key_the_player_commented_out_stays_unbound() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    let path = home.join("keys.toml");
    let text = read_text(&path).replace("\"shift+k\" = \"rename\"", "# \"shift+k\" = \"rename\"");
    fs::write(&path, text).unwrap();
    let keys = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert!(!keys.bindings[DIRECTION].contains_key("shift+k"));
    assert!(!keys.notes.iter().any(|note| note.contains("added")));
}

#[test]
fn test_a_mode_pa3ds_does_not_have_is_said_and_dropped() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("keys.toml"),
        "[pa3ds.navigation.compass]\nz = \"north\"\n",
    )
    .unwrap();
    let keys = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert_eq!(keys.bindings, bindings());
    assert!(
        keys.notes
            .iter()
            .any(|note| note.contains("no mode called 'compass'"))
    );
}

/// Being told the wrong key is a nuisance; a reader with no keys at all is not.
#[test]
fn test_a_file_that_will_not_parse_costs_the_edit_not_the_keys() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("keys.toml"),
        "[pa3ds.navigation.list]\nz = \"unclosed\n",
    )
    .unwrap();
    let keys = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert_eq!(keys.bindings, bindings());
    assert!(
        keys.notes
            .iter()
            .any(|note| note.contains("could not be read"))
    );
    // Said, interrupting, with where it breaks: a player who edited it would otherwise wonder why
    // nothing they did took (Orion, 2026-10-17). And left alone, for them to fix.
    assert!(keys.warning);
    assert!(keys.said.starts_with("keys.toml could not be read: "));
    assert!(keys.said.contains("line 2"), "{}", keys.said);
    assert!(
        keys.said
            .ends_with("delete it for pa3ds to write a new one")
    );
    assert!(read_text(&home.join("keys.toml")).ends_with("\"unclosed\n"));
}

#[test]
fn test_a_button_nobody_has_is_said_and_left_out() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(home.join("keys.toml"), "[azahar]\nwibble = \"y\"\n").unwrap();
    let keys = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert!(
        keys.notes
            .iter()
            .any(|note| note.contains("no button called 'wibble'"))
    );
}

#[test]
fn test_the_azahar_section_is_written_into_the_emulator_s_config() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    let config = written(&tmp_path);
    fs::write(
        home.join("keys.toml"),
        "[pa3ds.navigation.list]\nk = \"item read\"\n\n[azahar]\nstart = \"y\"\n",
    )
    .unwrap();
    let keys = prepare(&bindings(), &home, &config, true, false, None);
    assert!(read(&config).contains("profiles\\1\\button_start=\"code:89,engine:keyboard\""));
    assert!(keys.said.contains("keys applied"));
    assert_eq!(
        keyfile::emulator_keys(&read(&config)).unwrap()["button_start"],
        ["y"]
    );
}

/// Reading and writing text on Windows quietly translates line endings both ways, so a patch
/// of one button would have handed Azahar its whole config rewritten as LF.
#[test]
fn test_patching_through_the_whole_run_leaves_the_config_crlf() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    let config = written(&tmp_path);
    fs::write(home.join("keys.toml"), "[azahar]\nstart = \"y\"\n").unwrap();
    prepare(&bindings(), &home, &config, true, false, None);
    let after = read(&config);
    assert!(!after.replace("\r\n", "").contains('\n'));
    assert_eq!(
        after.matches("\r\n").count(),
        self::config().matches("\r\n").count()
    );
}

/// Azahar saves its own settings over the file when it closes (`GMainWindow::closeEvent`),
/// so a patch written while it runs is lost, and telling the player to restart the game was
/// telling them to throw it away.
#[test]
fn test_nothing_is_patched_while_azahar_is_open() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    let config = written(&tmp_path);
    fs::write(home.join("keys.toml"), "[azahar]\nstart = \"y\"\n").unwrap();
    let keys = prepare(&bindings(), &home, &config, true, true, None);
    assert_eq!(read(&config), self::config());
    assert!(keys.said.contains("close Azahar and start pa3ds again"));
}

#[test]
fn test_a_key_that_changes_nothing_says_nothing_was_applied() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(home.join("keys.toml"), "[azahar]\nstart = \"nosuchkey\"\n").unwrap();
    let keys = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert!(!keys.said.contains("applied"));
}

#[test]
fn test_a_dry_run_reads_everything_and_writes_nothing() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    let config = written(&tmp_path);
    fs::write(home.join("keys.toml"), "[azahar]\nstart = \"y\"\n").unwrap();
    let keys = prepare(&bindings(), &home, &config, false, false, None);
    assert_eq!(read(&config), self::config());
    assert!(keys.said.contains("keys applied"));
}

#[test]
fn test_a_missing_emulator_config_is_said_and_nothing_else_stops() {
    let tmp_path = TmpPath::new();
    let keys = prepare(
        &bindings(),
        &tmp_path.join("home"),
        &tmp_path.join("nothing.ini"),
        true,
        false,
        None,
    );
    assert_eq!(keys.bindings, bindings());
    assert!(
        keys.notes
            .iter()
            .any(|note| note.contains("could not be read"))
    );
    assert_eq!(keys.said, "");
}

#[test]
fn test_a_portable_azahar_is_the_folder_with_azahar_exe_and_a_user_folder() {
    let tmp_path = TmpPath::new();
    fs::write(tmp_path.join("azahar.exe"), b"").unwrap();
    assert!(keyfile::portable(&tmp_path).is_err());
    fs::create_dir(tmp_path.join("user")).unwrap();
    assert_eq!(keyfile::portable(&tmp_path), Ok(tmp_path.to_path_buf()));
    assert_eq!(
        keyfile::portable(&tmp_path.join("user")),
        Ok(tmp_path.to_path_buf())
    );
    assert_eq!(
        keyfile::emulator_config(&crate::py::path_str(&tmp_path)).unwrap(),
        tmp_path.join("user").join("config").join("qt-config.ini")
    );
}

#[test]
fn test_a_folder_that_is_not_a_portable_azahar_says_why() {
    let tmp_path = TmpPath::new();
    let empty = tmp_path.join("empty");
    fs::create_dir(&empty).unwrap();
    assert!(
        keyfile::portable(&empty)
            .unwrap_err()
            .contains("no azahar.exe")
    );
    fs::write(empty.join("azahar.exe"), b"").unwrap();
    assert!(keyfile::portable(&empty).unwrap_err().contains("AppData"));
}

// ---------------------------------------------------------------- modifiers

#[test]
fn test_modifiers_are_written_in_one_order_so_one_press_has_one_name() {
    assert_eq!(modified("k", false, false, true), "shift+k");
    assert_eq!(modified("k", true, true, true), "ctrl+alt+shift+k");
    assert_eq!(modified("k", false, false, false), "k");
}

/// A player holding shift from the last thing they typed should not find K dead.
#[test]
fn test_shift_falls_back_to_the_key_on_its_own() {
    assert_eq!(bound("shift+k", ["k"], false, false, false), "k");
    assert_eq!(
        bound("shift+k", ["shift+k", "k"], false, false, false),
        "shift+k"
    );
}

/// Ctrl and a letter belongs to whatever is in front: ctrl+c must reach the emulator even
/// when C is one of pa3ds's keys.
#[test]
fn test_ctrl_and_alt_never_fall_back() {
    assert_eq!(bound("ctrl+c", ["c"], true, false, false), "");
    assert_eq!(bound("alt+k", ["k"], false, true, false), "");
    assert_eq!(bound("ctrl+k", ["ctrl+k"], true, false, false), "ctrl+k");
}

/// Insert and R is NVDA's OCR, and pa3ds took it for R and said the battle's side (Orion,
/// 2026-09-29).
#[test]
fn test_nothing_is_pa3ds_s_while_the_screen_reader_s_key_is_held() {
    assert_eq!(bound("r", ["r"], false, false, true), "");
    assert_eq!(bound("shift+r", ["shift+r"], false, false, true), "");
    assert_eq!(bound("r", ["r"], false, false, false), "r");
}

#[test]
fn test_a_key_nobody_bound_is_nobody_s() {
    assert_eq!(bound("q", ["k"], false, false, false), "");
}

// ---------------------------------------------------------------- the sticks

/// A stick holds a parameter package Azahar keeps in a sorted map, with annotations beside
/// the four directions. Changing one direction must not disturb the rest, so the parser has to
/// give the untouched value back exactly.
#[test]
fn test_a_sticks_value_reads_and_writes_back_byte_for_byte() {
    let stick = keyfile::emulator_keys(&config()).unwrap();
    assert_eq!(stick["c_stick"], ["i", "k", "j", "l"]);
    let value = stick_value(&config());
    assert_eq!(keyfile::pack(&keyfile::unpack(&value)), value);
}

/// The keys file has to show a stick the way a table sets it, or the block it writes as a
/// comment cannot be uncommented and edited.
#[test]
fn test_a_stick_is_also_read_as_what_each_of_its_settings_is_on() {
    assert_eq!(
        keyfile::emulator_sticks(&config()).unwrap()["c_stick"],
        mode(&[("up", "i"), ("down", "k"), ("left", "j"), ("right", "l")])
    );
}

/// Against this machine's own config, since that is the layout the defaults were chosen
/// against: the circle pad on the arrows and the C-stick on i/j/k/l, both held halfway by D.
#[test]
fn test_the_real_layout_reads_back_the_way_the_keys_file_shows_it() {
    let config = include_str!("fake_qt_config.ini")
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let sticks = keyfile::emulator_sticks(&config).unwrap();
    assert_eq!(
        sticks["circle_pad"],
        mode(&[
            ("up", "up"),
            ("down", "down"),
            ("left", "left"),
            ("right", "right"),
            ("modifier", "d"),
            ("modifier_scale", "0.500000")
        ])
    );
    assert_eq!(sticks["c_stick"]["up"], "i");
    assert_eq!(sticks["c_stick"]["modifier"], "d");
}

/// The user, 2026-09-22: w/a/s/d to walk, with the modifier somewhere else, because the
/// modifier is what sneaking and DexNav need and w/a/s/d is what some people prefer.
#[test]
fn test_a_stick_can_be_moved_onto_other_keys() {
    let (patched, changed) = keyfile::patch(
        &config(),
        &stick(
            "c_stick",
            &[
                ("up", "w"),
                ("left", "a"),
                ("down", "s"),
                ("right", "d"),
                ("modifier", "shift"),
            ],
        ),
    )
    .unwrap();
    assert_eq!(
        keyfile::emulator_keys(&patched).unwrap()["c_stick"],
        ["w", "s", "a", "d", "shift"]
    );
    assert_eq!(
        changed,
        ["the C-stick is now up w, left a, down s, right d, modifier shift"]
    );
}

/// Azahar keys a per-direction note by that direction's own binding, so a direction that
/// moves and leaves its note behind would leave a stale pair in the value for ever.
#[test]
fn test_moving_a_direction_takes_its_annotation_with_it() {
    let (patched, _) = keyfile::patch(&config(), &stick("c_stick", &[("up", "w")])).unwrap();
    let params = keyfile::unpack(&stick_value(&patched));
    assert_eq!(params["up"], "code:87,engine:keyboard");
    assert!(params.contains_key("code:87,engine:keyboard"));
    assert!(!params.contains_key("code:73,engine:keyboard"));
}

/// A player gives 0.5 and Azahar stores 0.500000.
#[test]
fn test_the_modifier_scale_is_written_the_way_the_config_keeps_it() {
    let (patched, _) = keyfile::patch(
        &config(),
        &stick("c_stick", &[("modifier_scale", "0.250000")]),
    )
    .unwrap();
    assert_eq!(
        keyfile::unpack(&stick_value(&patched))["modifier_scale"],
        "0.250000"
    );
}

#[test]
fn test_a_stick_setting_that_does_not_exist_is_said_rather_than_written() {
    let (_, changed) = keyfile::patch(&config(), &stick("c_stick", &[("diagonal", "w")])).unwrap();
    assert_eq!(changed, ["c_stick: a stick has no 'diagonal'"]);
}

#[test]
fn test_a_stick_is_read_from_the_file_with_its_scale_as_a_number() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    let config = written(&tmp_path);
    fs::write(
        home.join("keys.toml"),
        "[azahar.c_stick]\nup = \"w\"\nmodifier = \"shift\"\nmodifier_scale = 0.5\n",
    )
    .unwrap();
    let keys = prepare(&bindings(), &home, &config, true, false, None);
    let params = keyfile::unpack(&stick_value(&read(&config)));
    assert_eq!(params["up"], "code:87,engine:keyboard");
    assert_eq!(params["modifier"], "code:16777248,engine:keyboard");
    assert_eq!(params["modifier_scale"], "0.500000");
    assert!(keys.said.contains("keys applied"));
}

#[test]
fn test_a_stick_nobody_has_is_said_and_left_out() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(home.join("keys.toml"), "[azahar.d_stick]\nup = \"w\"\n").unwrap();
    let keys = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert!(
        keys.notes
            .iter()
            .any(|note| note.contains("no stick called 'd_stick'"))
    );
}

// ---------------------------------------------------------------- shared keys

#[test]
fn test_a_key_that_means_the_same_in_both_modes_is_written_once() {
    assert_eq!(keyfile::shared(&bindings()), mode(&[("'", "mode")]));
    assert_eq!(
        keyfile::shared(&modes(&[(LIST, &[("k", "a")]), (DIRECTION, &[("k", "b")])])),
        IndexMap::<String, String>::new()
    );
}

#[test]
fn test_a_modes_own_key_wins_over_the_one_above_it() {
    let tmp_path = TmpPath::new();
    let home = tmp_path.join("home");
    fs::create_dir(&home).unwrap();
    fs::write(
        home.join("keys.toml"),
        "[pa3ds.navigation]\nk = \"item read\"\n\n[pa3ds.navigation.direction]\nk = \"north\"\n",
    )
    .unwrap();
    let keys = prepare(&bindings(), &home, &written(&tmp_path), true, false, None);
    assert_eq!(keys.bindings[LIST]["k"], "item read");
    assert_eq!(keys.bindings[DIRECTION]["k"], "north");
}

/// NVDA swallows insert in its own hook, so Windows never says it is held, and insert and R,
/// NVDA's OCR, reached pa3ds as R (Orion, 2026-09-29). pa3ds's hook is called first and sees
/// insert go down and up.
#[test]
fn test_nvdas_modifier_is_held_from_the_hooks_own_events() {
    let (insert, caps_lock, r) = (0x2D, 0x14, 0x52);
    let mut held: HashSet<i64> = HashSet::new();
    track(&mut held, insert, WM_KEYDOWN);
    track(&mut held, r, WM_KEYDOWN);
    assert_eq!(held, HashSet::from([insert]));
    track(&mut held, caps_lock, WM_SYSKEYDOWN);
    track(&mut held, insert, WM_KEYUP);
    assert_eq!(held, HashSet::from([caps_lock]));
    track(&mut held, caps_lock, WM_KEYUP);
    assert!(held.is_empty());
}
