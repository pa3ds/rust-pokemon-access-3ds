//! The AreaNav's Pokémon Distribution names each picture by the species whose icon it draws.

use std::cell::RefCell;
use std::rc::Rc;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::lyt;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::screens::distribution as d;

const NAVI: i64 = 0x08200000;
const ICONS: i64 = 0x08210000;
const REPLACE: i64 = 0x08220000;
const LISTED: i64 = 0x08230000;
const TEXTURES: i64 = 0x08240000;
const PICTURES: i64 = 0x08300000;
const MATERIALS: i64 = 0x08310000;
const GREY: i64 = 0x20710F00;

fn species() -> Vec<String> {
    let mut names = vec!["Egg".to_string()];
    names.extend(std::iter::repeat_n(String::new(), 260));
    names.extend(
        ["Poochyena", "", "Zigzagoon", "", "Wurmple"]
            .iter()
            .map(|name| name.to_string()),
    );
    names
}

/// Route 101 as spike 178 read it: Wurmple, Zigzagoon and Poochyena in textures 0 to 2, and
/// the grey shape a place's icons are drawn as while they load in texture 20.
fn route_101() -> (Rc<FakeMemory>, d::Distribution) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(NAVI, &[classes.add(d::CLASS)]);
    memory.u32(NAVI + d::ICONS, &[ICONS]);
    memory.u32(ICONS, &[classes.add(d::ICON_CLASS)]);
    memory.u32(ICONS + d::REPLACE, &[REPLACE]);
    memory.u32(REPLACE, &[classes.add(d::REPLACE_CLASS)]);
    memory.u32(REPLACE + d::TEXTURES, &[LISTED, 21]);
    for slot in 0..21 {
        memory.u32(LISTED + 4 * slot, &[TEXTURES + 0x6C * slot]);
        memory.u32(
            TEXTURES + 0x6C * slot + d::IMAGE,
            &[if slot == 20 {
                GREY
            } else {
                0x2077D480 + 0x1100 * slot
            }],
        );
    }
    for (slot, species) in [265, 263, 261].into_iter().enumerate() {
        memory.u16(NAVI + d::RECORDS + slot as i64 * d::RECORD, species);
    }
    memory.u32(NAVI + d::RECORD_COUNT, &[3]);
    let mut found = d::Distribution::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        species(),
    );
    found.track([NAVI], false);
    (memory, found)
}

fn picture(memory: &FakeMemory, index: i64, image: i64) -> i64 {
    let (pane, material) = (PICTURES + 0x200 * index, MATERIALS + 0x100 * index);
    memory.u32(pane + lyt::PICTURE_MATERIAL, &[material]);
    memory.u32(material + lyt::MATERIAL_TEXTURE, &[image]);
    pane
}

#[test]
fn test_a_picture_is_named_by_the_record_of_the_texture_it_draws() {
    let (memory, found) = route_101();
    let named: Vec<Option<String>> = (0..3)
        .map(|i| {
            found
                .name(picture(&memory, i, 0x2077D480 + 0x1100 * i))
                .unwrap()
        })
        .collect();
    assert_eq!(
        named,
        vec![
            Some("Wurmple".to_string()),
            Some("Zigzagoon".to_string()),
            Some("Poochyena".to_string())
        ]
    );
}

/// ADR-0037: "caught" after a species caught, 2, and nothing after one only seen, 3, never
/// "not caught", as the save's Pokédex had Zigzagoon and Ralts (spike 195, movie27).
#[test]
fn test_only_a_species_caught_is_said_so() {
    let (memory, found) = route_101();
    memory.write(NAVI + d::RECORDS + d::STATE, &[3]);
    memory.write(NAVI + d::RECORDS + d::RECORD + d::STATE, &[d::CAUGHT as u8]);
    let said: Vec<Option<String>> = (0..3)
        .map(|i| {
            found
                .said(picture(&memory, i, 0x2077D480 + 0x1100 * i), " ")
                .unwrap()
        })
        .collect();
    assert_eq!(
        said,
        [
            Some("Wurmple".to_string()),
            Some("Zigzagoon caught".to_string()),
            Some("Poochyena".to_string())
        ]
    );
    assert_eq!(found.said(picture(&memory, 0, GREY), " ").unwrap(), None);
}

/// ADR-0037: 1 water, 2 fishing, in the habitat page's words, before "caught", which would join it
/// as "caught fishing" (movie27 on Route 102); 0, walking in the grass or on a cave's floor, says
/// none, as the test above has it, and 3, the DexNav's search alone, none. A
/// comma between each under `pauses`.
#[test]
fn test_each_species_says_how_it_is_met_before_caught() {
    let (memory, found) = route_101();
    for (slot, way) in [1u8, 2, 3].into_iter().enumerate() {
        memory.write(NAVI + d::RECORDS + slot as i64 * d::RECORD + d::WAY, &[way]);
    }
    memory.write(NAVI + d::RECORDS + d::RECORD + d::STATE, &[d::CAUGHT as u8]);
    let said: Vec<Option<String>> = (0..3)
        .map(|i| {
            found
                .said(picture(&memory, i, 0x2077D480 + 0x1100 * i), " ")
                .unwrap()
        })
        .collect();
    assert_eq!(
        said,
        [
            Some("Wurmple water".to_string()),
            Some("Zigzagoon fishing caught".to_string()),
            Some("Poochyena".to_string())
        ]
    );
    assert_eq!(
        found
            .said(picture(&memory, 1, 0x2077D480 + 0x1100), ", ")
            .unwrap()
            .as_deref(),
        Some("Zigzagoon, fishing, caught")
    );
}

/// Route 101's records met each its (way, state).
fn seen(memory: &FakeMemory, records: &[(u8, u8)]) {
    for (slot, (way, state)) in records.iter().enumerate() {
        memory.write(
            NAVI + d::RECORDS + slot as i64 * d::RECORD + d::WAY,
            &[*way],
        );
        memory.write(
            NAVI + d::RECORDS + slot as i64 * d::RECORD + d::STATE,
            &[*state],
        );
    }
}

fn utf16(text: &str) -> Vec<u8> {
    text.encode_utf16()
        .flat_map(|unit| unit.to_le_bytes())
        .collect()
}

fn lines(said: &[&str]) -> Vec<String> {
    said.iter().map(|line| line.to_string()).collect()
}

/// ADR-0037 (spike 232): in the navigator's order, those met walking first and each way said once
/// before its own, the grey shape of an icon loading ignored; nothing while a species seen is still
/// to be drawn, or a species not seen is drawn, as the last place's pictures are a moment after the
/// place changes.
#[test]
fn test_the_areanav_says_its_distribution_by_way_once_all_is_drawn() {
    let (memory, found) = route_101();
    memory.write(NAVI + d::PLACE, &utf16("Route 101\0"));
    let (caught, shown) = (d::CAUGHT as u8, d::SEEN as u8);
    seen(&memory, &[(2, caught), (0, shown), (1, shown)]);
    let icons: Vec<i64> = (0..3)
        .map(|i| picture(&memory, i, 0x2077D480 + 0x1100 * i))
        .collect();
    assert_eq!(found.listed(icons[..2].to_vec(), false).unwrap(), None);
    let loading = [
        vec![picture(&memory, 3, GREY)],
        icons.iter().rev().copied().collect(),
    ]
    .concat();
    assert_eq!(
        found.listed(loading, false).unwrap(),
        Some((
            "Route 101".to_string(),
            lines(&["Zigzagoon", "water Poochyena", "fishing Wurmple caught"])
        ))
    );
    assert_eq!(
        found.listed(icons.clone(), true).unwrap(),
        Some((
            "Route 101".to_string(),
            lines(&["Zigzagoon", "water: Poochyena", "fishing: Wurmple caught"])
        ))
    );
    seen(&memory, &[(2, caught), (0, shown), (3, 1)]);
    assert_eq!(found.listed(icons, false).unwrap(), None);
}

/// Spike 232: the icons can all be drawn before the top screen titles their place, and are said
/// once the title has read the same two polls, as screen text reports it; or, titled otherwise,
/// after `TITLE_POLLS`.
#[test]
fn test_the_reader_says_a_place_once_titled_and_keeps_its_icons_from_screen_text() {
    let (memory, found) = route_101();
    memory.write(NAVI + d::PLACE, &utf16("Route 101\0"));
    let shown = d::SEEN as u8;
    seen(&memory, &[(0, shown), (0, shown), (0, shown)]);
    let icons: Vec<ScreenText> = (0..3)
        .map(|i| ScreenText {
            picture: true,
            ..ScreenText::new(picture(&memory, i, 0x2077D480 + 0x1100 * i), "", false)
        })
        .collect();
    let titled = [
        vec![ScreenText {
            top: true,
            ..ScreenText::new(1, "Route 101", false)
        }],
        icons.clone(),
    ]
    .concat();
    let mut reader = d::DistributionReader::new(Rc::new(RefCell::new(found)), false);
    let said = lines(&["Wurmple Zigzagoon Poochyena"]);
    assert_eq!(reader.poll(&icons, true).unwrap(), Vec::<String>::new());
    assert_eq!(reader.poll(&titled, true).unwrap(), Vec::<String>::new());
    assert_eq!(reader.poll(&titled, true).unwrap(), said);
    assert_eq!(reader.poll(&titled, true).unwrap(), Vec::<String>::new());
    assert!(icons.iter().all(|piece| reader.owns(piece)));
    reader.poll(&titled[..2], true).unwrap();
    assert_eq!(reader.poll(&titled, true).unwrap(), said);
    reader.poll(&icons[..1], true).unwrap();
    let polled: Vec<Vec<String>> = (0..d::TITLE_POLLS)
        .map(|_| reader.poll(&icons, true).unwrap())
        .collect();
    assert!(polled.last() == Some(&said) && polled[..polled.len() - 1].iter().all(Vec::is_empty));
    reader.poll(&icons, false).unwrap();
    assert!(!icons.iter().any(|piece| reader.owns(piece)));
}

#[test]
fn test_the_grey_shape_and_an_image_of_no_texture_are_not_named() {
    let (memory, found) = route_101();
    assert_eq!(found.name(picture(&memory, 0, GREY)).unwrap(), None);
    assert_eq!(found.name(picture(&memory, 1, 0x20000000)).unwrap(), None);
}

/// Its memory taken by something else: the chain's first link is no navigator.
#[test]
fn test_a_navigator_gone_names_nothing() {
    let (memory, found) = route_101();
    let pane = picture(&memory, 0, 0x2077D480);
    memory.u32(NAVI, &[0]);
    assert_eq!(found.name(pane).unwrap(), None);
}
