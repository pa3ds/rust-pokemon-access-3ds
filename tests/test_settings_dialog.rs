//! The settings dialog (`pa3ds.ux.settings`), read back from its template: no test opens it.

use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use super::test_rename::read_template;
use crate::core::config::{self, Config, Value};
use crate::ux::dialog::{ATOM_BUTTON, ATOM_EDIT, Class, IDCANCEL, IDOK};
use crate::ux::settings::{
    ATOM_COMBOBOX, CHECK, IDC_APPLY, IDC_PAGES, NUMBER, PERCENT, TABS, changed, choices, fields,
    languages, shown, template, typed, values,
};

#[test]
fn test_every_setting_is_one_control_on_its_tables_page() {
    let every = fields();
    let expected: Vec<String> = config::TABLES
        .iter()
        .flat_map(|table| {
            config::defaults(table)
                .into_iter()
                .map(move |(each, _)| format!("{table}.{each}"))
        })
        .collect();
    assert_eq!(
        every
            .iter()
            .map(|each| each.name.clone())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        every
            .iter()
            .map(|each| each.ident)
            .collect::<HashSet<_>>()
            .len(),
        every.len()
    );
    let kinds: HashMap<&str, &str> = every
        .iter()
        .filter(|each| each.kind != CHECK)
        .map(|each| (each.name.as_str(), each.kind))
        .collect();
    assert_eq!(
        kinds,
        HashMap::from([
            ("pa3ds.poll_rate", NUMBER),
            ("pa3ds.azahar_folder", "folder"),
            ("pa3ds.ui.language", "choice"),
            ("pa3ds.audio.tones", PERCENT),
            ("pa3ds.audio.footsteps", PERCENT),
            ("pa3ds.audio.ticks", PERCENT),
            ("pa3ds.audio.drag_sounds", PERCENT),
            ("pa3ds.navigation.drag", "choice"),
            ("pa3ds.extras.hidden_items", "choice"),
        ])
    );
}

/// Tab's order: the tabs, a page's controls, OK, Cancel and Apply. A checkbox is
/// named by its words, every other control by the static just before it (`ux.dialog`).
#[test]
fn test_the_dialog_has_a_tab_a_page_then_names_each_control_and_ends_with_its_buttons() {
    let (title, items) = read_template(&template().unwrap());
    assert_eq!(title, "Mod settings");
    assert_eq!(items[0], (IDC_PAGES, Class::from(TABS), String::new()));
    assert_eq!(
        items[items.len() - 3..],
        [
            (IDOK, Class::Atom(ATOM_BUTTON), "OK".to_string()),
            (IDCANCEL, Class::Atom(ATOM_BUTTON), "Cancel".to_string()),
            (IDC_APPLY, Class::Atom(ATOM_BUTTON), "&Apply".to_string()),
        ]
    );
    let by_id: HashMap<i64, (Class, String)> = items
        .iter()
        .map(|(ident, atom, text)| (*ident, (atom.clone(), text.clone())))
        .collect();
    let named: HashMap<String, i64> = fields()
        .into_iter()
        .map(|each| (each.name, each.ident))
        .collect();
    assert_eq!(
        by_id[&named["pa3ds.ui.pauses"]],
        (
            Class::Atom(ATOM_BUTTON),
            "Pauses in screen text".to_string()
        )
    );
    let tones = named["pa3ds.audio.tones"];
    let idents: Vec<i64> = items.iter().map(|(ident, _, _)| *ident).collect();
    let at = |ident: i64| idents.iter().position(|each| *each == ident).unwrap();
    assert_eq!(at(tones + 1) + 1, at(tones));
    assert_eq!(by_id[&(tones + 1)].1, "Relative volume for tones");
    assert_eq!(by_id[&tones].0, Class::Atom(ATOM_EDIT));
    assert_eq!(
        by_id[&named["pa3ds.ui.language"]].0,
        Class::Atom(ATOM_COMBOBOX)
    );
}

/// Orion, 2026-10-25: a spin box in percent, 100 as loud as the game, from 0 up.
#[test]
fn test_a_loudness_is_shown_in_percent_and_written_as_the_file_has_it() {
    assert_eq!(shown(PERCENT, &Value::Float(1.0)).unwrap(), "100");
    assert_eq!(shown(PERCENT, &Value::Float(0.5)).unwrap(), "50");
    assert_eq!(typed(PERCENT, "150", &Value::Float(1.0)), Value::Float(1.5));
    assert_eq!(typed(PERCENT, "0", &Value::Float(1.0)), Value::Float(0.0));
    assert_eq!(shown(NUMBER, &Value::Float(60.0)).unwrap(), "60");
    assert_eq!(
        typed(NUMBER, " 30 ", &Value::Float(60.0)),
        Value::Float(30.0)
    );
}

#[test]
fn test_a_box_that_says_nothing_a_setting_can_take_keeps_what_it_was() {
    assert_eq!(typed(NUMBER, "0", &Value::Float(60.0)), Value::Float(60.0));
    assert_eq!(typed(NUMBER, "", &Value::Float(60.0)), Value::Float(60.0));
    assert_eq!(
        typed(PERCENT, "loud", &Value::Float(1.0)),
        Value::Float(1.0)
    );
}

#[test]
fn test_only_the_settings_changed_are_written() {
    let before = values(&Config::default());
    let mut after = before.clone();
    after.insert("pa3ds.ui.pauses".to_string(), Value::Bool(true));
    after.insert("pa3ds.audio.tones".to_string(), Value::Float(1.0));
    assert_eq!(
        changed(&before, &after),
        IndexMap::from([("pa3ds.ui.pauses".to_string(), Value::Bool(true))])
    );
}

/// Orion, 2026-10-26: hidden items off, under the sea alone, or everywhere.
#[test]
fn test_a_setting_of_a_few_words_lists_each_by_its_message_the_default_first() {
    let pair = |code: &str, said: &str| (code.to_string(), said.to_string());
    assert_eq!(
        choices("pa3ds.extras.hidden_items").unwrap(),
        vec![
            pair("off", "Off"),
            pair("underwater", "Under the sea"),
            pair("all", "Everywhere")
        ]
    );
    assert_eq!(choices("pa3ds.ui.language").unwrap(), languages().unwrap());
}

#[test]
fn test_the_languages_are_the_games_own_first_then_each_by_its_own_name() {
    let found = languages().unwrap();
    let pair = |code: &str, said: &str| (code.to_string(), said.to_string());
    assert_eq!(found[0], pair("", "The game's own language"));
    assert!(found.contains(&pair("en", "English")));
    assert!(found.contains(&pair("fr", "Français")));
    assert!(found.contains(&pair("zh-TW", "繁體中文")));
}
