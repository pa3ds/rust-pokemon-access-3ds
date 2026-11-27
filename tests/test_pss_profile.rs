//! A PSS profile's values, drawn into a picture a row (spike 319).

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::lyt;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::subscreen::pss_profile;

const ROWS: i64 = 0x08660000;

/// Another player's profile drew "Nice to meet ya!", History's icons, "United States" and
/// "District of Columbia" a picture a row, in five frames under the layout's root, its rows at
/// 45, -5, -55 and -105 down the list: the n-th frame is the n-th row down, scrolled or not, and
/// the texture drawn in a frame elsewhere, hidden at 0, is none of the list's. History's row of
/// icons names nothing (Y, spike 319).
#[test]
fn test_a_profiles_rows_are_said_in_their_pictures_by_their_order() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let picture = classes.add(lyt::PICTURE_CLASS);
    let (message, history) = (
        classes.add(pss_profile::ROWS[0]),
        classes.add(pss_profile::ROWS[1]),
    );
    let (_work, root) = fake.screen_with(0x08200000, false, false);
    let mut pictures: Vec<i64> = Vec::new();

    let drawn = |at: i64, parent: i64| -> i64 {
        fake.pane_with_vtable(at, parent, Some(picture));
        let mut name = b"v".to_vec();
        name.resize(lyt::NAME_SIZE as usize, 0);
        memory.write(at + lyt::NAME, &name);
        at
    };

    // The list's five frames under the root, scrolled 46 up, the fifth's picture hidden at 0.
    for (n, y) in [91.0f32, 41.0, -9.0, -59.0, 0.0].into_iter().enumerate() {
        let frame = root + 0x1000 * (n as i64 + 1);
        fake.pane(frame, root);
        memory.write(frame + lyt::Y, &y.to_le_bytes());
        pictures.push(drawn(frame + 0x200, frame));
    }
    fake.visible(pictures[pictures.len() - 1], false);
    // The texture drawn elsewhere: in a frame a pane down, at 0, and in the root itself.
    let elsewhere = root + 0x8000;
    fake.pane(elsewhere, root);
    fake.pane(elsewhere + 0x200, elsewhere);
    pictures.extend([
        drawn(elsewhere + 0x400, elsewhere + 0x200),
        drawn(root + 0x9000, root),
    ]);
    fake.track().unwrap();
    let mut rows: Vec<i64> = Vec::new();
    for (n, (place, words)) in [
        (45i16, Some("Nice to meet ya!")),
        (-5, None),
        (-55, Some("United States")),
        (-105, Some("District of Columbia")),
    ]
    .into_iter()
    .enumerate()
    {
        let row = ROWS + 0x100 * n as i64;
        memory.u32(row, &[if words.is_some() { message } else { history }]);
        memory.write(row + pss_profile::PLACE, &place.to_le_bytes());
        if let Some(words) = words {
            let (buffer, text) = (row + 0x40, row + 0x80);
            memory.u32(row + pss_profile::WORDS, &[buffer]);
            memory.u32(buffer + 4, &[text]);
            memory.write(
                buffer + 0x0A,
                &(words.encode_utf16().count() as u16).to_le_bytes(),
            );
            let encoded: Vec<u8> = words.encode_utf16().flat_map(u16::to_le_bytes).collect();
            memory.write(text, &encoded);
        }
        rows.push(row);
    }
    let file = Rc::new(LayoutFile {
        name: pss_profile::FILE.to_string(),
        ..LayoutFile::new(
            Vec::new(),
            IndexMap::from([("v".to_string(), vec![pss_profile::TEXTURE.to_string()])]),
            IndexMap::new(),
        )
    });
    let mut values = pss_profile::ProfileValues::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        fake.layouts.clone(),
        Rc::new(move |_root| Some(file.clone())),
    );
    let found: IndexMap<i64, String> = rows
        .iter()
        .map(|&row| (row, pss_profile::ROWS[0].to_string()))
        .collect();
    values.track(&found, false).unwrap();
    let said = [
        Some("Nice to meet ya!".to_string()),
        None,
        Some("United States".to_string()),
        Some("District of Columbia".to_string()),
        None,
        None,
        None,
    ];
    let named: Vec<Option<String>> = pictures.iter().map(|&p| values.name(p).unwrap()).collect();
    assert_eq!(named, said);
    // A sweep of the blocks just allocated found three of the rows, and Location 1 took
    // Location 2's words (Omega Ruby, live): they join those kept.
    let some: IndexMap<i64, String> = rows
        .iter()
        .skip(1)
        .step_by(2)
        .map(|&row| (row, pss_profile::ROWS[0].to_string()))
        .collect();
    values.track(&some, true).unwrap();
    let named: Vec<Option<String>> = pictures.iter().map(|&p| values.name(p).unwrap()).collect();
    assert_eq!(named, said);
    // Another profile's rows at the same places take the closed one's.
    let later = ROWS + 0x1000;
    memory.u32(later, &[message]);
    memory.write(later + pss_profile::PLACE, &(-55i16).to_le_bytes());
    memory.write(
        later + pss_profile::WORDS,
        &memory.read(rows[3] + pss_profile::WORDS, 4).unwrap(),
    );
    values
        .track(
            &IndexMap::from([(later, pss_profile::ROWS[0].to_string())]),
            true,
        )
        .unwrap();
    assert_eq!(
        values.name(pictures[2]).unwrap().as_deref(),
        Some("District of Columbia")
    );
}

/// History shows each of its icons as a picture of its kind's own, hung from the layout's
/// root, the layout listing a kind's six together, and places those shown 28 apart in the
/// record's order: the first says them all, left to right, a repeat again, and they are said
/// after History's label in its row's frame. Named one at a time, two alike side by side were
/// one thing drawn twice, the rest were joined with slashes, and all came after both Locations
/// (or_pss_singleplayer).
#[test]
fn test_history_says_its_icons_in_their_order_after_its_label() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let picture = classes.add(lyt::PICTURE_CLASS);
    let (message, history) = (
        classes.add(pss_profile::ROWS[0]),
        classes.add(pss_profile::ROWS[1]),
    );
    let (_work, root) = fake.screen_with(0x08200000, false, false);

    let drawn = |at: i64, parent: i64, name: &str| -> i64 {
        fake.pane_with_vtable(at, parent, Some(picture));
        let mut named = name.as_bytes().to_vec();
        named.resize(lyt::NAME_SIZE as usize, 0);
        memory.write(at + lyt::NAME, &named);
        at
    };

    // The rows' frames under the root, each with its label and the picture its words go in.
    let mut frames: Vec<i64> = Vec::new();
    for n in 0..2 {
        let frame = root + 0x1000 * (n + 1);
        frames.push(frame);
        fake.pane(frame, root);
        drawn(frame + 0x200, frame, "v");
        fake.pane(frame + 0x400, frame);
    }
    let mut icons: IndexMap<&str, i64> = IndexMap::new();
    for (n, (name, x, shown)) in [
        ("27", -40.0f32, true),
        ("35", 0.0, false),
        ("47", 16.0, true),
        ("86", 44.0, true),
        ("8s", -12.0, true),
    ]
    .into_iter()
    .enumerate()
    {
        let icon = drawn(root + 0x8000 + 0x200 * n as i64, root, name);
        icons.insert(name, icon);
        memory.write(icon + lyt::X, &x.to_le_bytes());
        fake.visible(icon, shown);
    }
    fake.track().unwrap();
    let rows = [ROWS, ROWS + 0x100];
    for (row, kind, place) in [(rows[0], message, 45i16), (rows[1], history, -5)] {
        memory.u32(row, &[kind]);
        memory.write(row + pss_profile::PLACE, &place.to_le_bytes());
    }
    let textures: IndexMap<String, Vec<String>> = [
        ("v", pss_profile::TEXTURE),
        ("27", "prof_battle.bclim"),
        ("35", "prof_battle.bclim"),
        ("47", "prof_trade.bclim"),
        ("86", "prof_trade.bclim"),
        ("8s", "prof_asist.bclim"),
    ]
    .into_iter()
    .map(|(pane, texture)| (pane.to_string(), vec![texture.to_string()]))
    .collect();
    let file = Rc::new(LayoutFile {
        name: pss_profile::FILE.to_string(),
        ..LayoutFile::new(Vec::new(), textures, IndexMap::new())
    });
    let words: IndexMap<String, String> = [
        ("prof_battle", "Battle"),
        ("prof_trade", "Trade"),
        ("prof_asist", "O-Power"),
    ]
    .into_iter()
    .map(|(texture, word)| (texture.to_string(), word.to_string()))
    .collect();
    let mut values = pss_profile::ProfileValues::with_words(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        fake.layouts.clone(),
        Rc::new(move |_root| Some(file.clone())),
        words,
        ", ".to_string(),
    );
    let found: IndexMap<i64, String> = rows
        .iter()
        .map(|&row| (row, pss_profile::ROWS[0].to_string()))
        .collect();
    values.track(&found, false).unwrap();
    let said = "Battle, O-Power, Trade, Trade";
    let named: IndexMap<&str, Option<String>> = icons
        .iter()
        .map(|(&name, &pane)| (name, values.name(pane).unwrap()))
        .collect();
    assert_eq!(
        named,
        IndexMap::from([
            ("27", Some(said.to_string())),
            ("35", None),
            ("47", Some(String::new())),
            ("86", Some(String::new())),
            ("8s", Some(String::new())),
        ])
    );
    let (label, location) = (frames[1] + 0x400, frames[0] + 0x400);
    let on = |r#box: i64, text: &str, picture: bool| ScreenText {
        root,
        picture,
        ..ScreenText::new(r#box, text, false)
    };
    let found: IndexMap<i64, ScreenText> = IndexMap::from([
        (location, on(location, "Message", false)),
        (label, on(label, "History", false)),
        (icons["27"], on(icons["27"], said, true)),
    ]);
    assert_eq!(
        values.reworded(found.clone()).unwrap(),
        IndexMap::from([
            (location, found[&location].clone()),
            (label, on(label, &format!("History {said}"), false)),
        ])
    );
}
