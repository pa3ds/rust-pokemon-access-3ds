//! A PSS profile's values, drawn into a picture a row (spike A).

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::lyt;
use crate::games::gen6::subscreen::pss_profile;

const ROWS: i64 = 0x08660000;

/// Another player's profile drew "Nice to meet ya!", History's icons, "United States" and
/// "District of Columbia" a picture a row, in five frames under the layout's root, its rows at
/// 45, -5, -55 and -105 down the list: the n-th frame is the n-th row down, scrolled or not, and
/// the texture drawn in a frame elsewhere, hidden at 0, is none of the list's. History's row of
/// icons names nothing (Y, spike A).
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
    let named: Vec<Option<String>> = pictures.iter().map(|&p| values.name(p).unwrap()).collect();
    assert_eq!(
        named,
        [
            Some("Nice to meet ya!".to_string()),
            None,
            Some("United States".to_string()),
            Some("District of Columbia".to_string()),
            None,
            None,
            None
        ]
    );
}
