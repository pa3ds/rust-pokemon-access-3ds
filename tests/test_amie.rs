//! The Pokémon-Amie page's Pokémon is named by its species, through the icon it is drawn with.

use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::keyfile::data_path;
use crate::core::rtti::ClassNames;
use crate::core::words::{self, ENGLISH, Words};
use crate::games::common::lyt;
use crate::games::common::pictures::named;
use crate::games::gen6::screens::amie;
use crate::games::gen6::screens::distribution as d;
use crate::games::xy;

const PAGE: i64 = 0x08A00000;
const HOLDER: i64 = 0x08A01000;
const ICON: i64 = 0x08A02000;
const REPLACE: i64 = 0x08A03000;
const TABLE: i64 = 0x08A04000;
const TEXTURE: i64 = 0x08A05000;
const WALKER: i64 = 0x08A06000;
const OTHER: i64 = 0x08A07000;
const IMAGE: i64 = 0x18200000;

fn species() -> Vec<String> {
    let mut names = vec![String::new(); 254];
    names.push("Sceptile".to_string());
    names
}

fn page(number: i64) -> (amie::AmiePage, Rc<FakeMemory>) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(PAGE, &[classes.add(amie::CLASS)]);
    memory.u32(PAGE + number, &[254]);
    memory.u32(PAGE + number + amie::HOLDER, &[HOLDER]);
    memory.u32(HOLDER, &[ICON]);
    memory.u32(ICON, &[classes.add(d::ICON_CLASS)]);
    memory.u32(ICON + d::REPLACE, &[REPLACE]);
    memory.u32(REPLACE, &[classes.add(d::REPLACE_CLASS)]);
    memory.u32(REPLACE + d::TEXTURES, &[TABLE, 1]);
    memory.u32(TABLE, &[TEXTURE]);
    memory.u32(TEXTURE + d::IMAGE, &[IMAGE]);
    for (picture, image) in [(WALKER, IMAGE), (OTHER, IMAGE + 0x1000)] {
        let material = picture + 0x400;
        memory.u32(picture + lyt::PICTURE_MATERIAL, &[material]);
        memory.u32(material + lyt::MATERIAL_TEXTURE, &[image]);
    }
    let mut found = amie::AmiePage::with_number(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        species(),
        number,
    );
    found
        .track(&IndexMap::from([(PAGE, amie::CLASS.to_string())]), false)
        .unwrap();
    (found, memory)
}

#[test]
fn test_the_picture_drawing_the_icons_first_texture_is_named_by_its_species() {
    assert_eq!(
        page(amie::NUMBER).0.name(WALKER).unwrap().as_deref(),
        Some("Sceptile")
    );
}

#[test]
fn test_xs_page_is_read_where_x_keeps_its_pokemon() {
    assert_eq!(
        page(xy::game().amie).0.name(WALKER).unwrap().as_deref(),
        Some("Sceptile")
    );
}

/// X's text has no line for PLAY, DECORATE and SWITCH; a picture with no word in the data was
/// taken for part of the menu's close (spike 228, x_super_train3).
#[test]
fn test_xs_menu_buttons_have_a_word_in_every_language_x_is_played_in() {
    let game = data_path().unwrap().join("xy");
    for language in ["en", "de", "es", "fr", "it", "ja", "ko", "zh-CN", "zh-TW"] {
        words::r#use(Some(Rc::new(
            Words::new(
                language,
                Some(&game),
                IndexMap::new(),
                Rc::new(|_line: &str| {}),
            )
            .unwrap(),
        )));
        assert!(
            xy::game()
                .worded
                .iter()
                .all(|texture| named(texture).is_some_and(|word| !word.is_empty())),
            "{language}"
        );
    }
    words::r#use(Some(Rc::new(
        Words::new(
            ENGLISH,
            Some(&game),
            IndexMap::new(),
            Rc::new(|_line: &str| {}),
        )
        .unwrap(),
    )));
    let mut textures: Vec<&String> = xy::game().worded.iter().collect();
    textures.sort();
    assert_eq!(
        textures
            .into_iter()
            .map(|texture| named(texture))
            .collect::<Vec<_>>(),
        [
            Some("PLAY".to_string()),
            Some("DECORATE".to_string()),
            Some("SWITCH".to_string())
        ]
    );
}

#[test]
fn test_a_picture_drawing_another_image_is_not_named() {
    assert_eq!(page(amie::NUMBER).0.name(OTHER).unwrap(), None);
}

#[test]
fn test_amie_itself_offers_its_pokemon_to_stroke_by_the_species_the_page_showed() {
    let (mut found, memory) = page(amie::NUMBER);
    memory.u32(PAGE, &[0]); // the page freed as Amie runs
    assert!(
        found
            .strokes(Some("N5field12FieldmapProcE"))
            .unwrap()
            .is_empty()
    );
    let strokes = found.strokes(Some(amie::AMIE)).unwrap();
    let [stroke] = strokes.as_slice() else {
        panic!("one stroke: {strokes:?}");
    };
    assert_eq!(
        (stroke.name.as_str(), stroke.path.clone(), stroke.seconds),
        (
            "stroke Sceptile",
            amie::STROKE.to_vec(),
            amie::STROKE_SECONDS
        )
    );
    // A puff is dragged to its mouth, not its middle (spike 257).
    assert!(stroke.drop == Some(amie::MOUTH) && stroke.drop != Some((stroke.x, stroke.y)));
}

/// Amie's decorate screen draws its puffs with the images of a system holding a texture for
/// each kind, in the text's order; another holding fewer, the Interior Items', says nothing.
#[test]
fn test_a_puff_is_named_by_the_texture_of_the_replace_system_holding_every_kind() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let kinds: Vec<String> = ["Sweet Poké Puff", "Mint Poké Puff", "Citrus Poké Puff"]
        .iter()
        .map(|kind| kind.to_string())
        .collect();
    let replace = classes.add(d::REPLACE_CLASS);
    let mut systems: IndexMap<i64, String> = IndexMap::new();
    for (system, count, base) in [(0x08B00000, 3, 0x20700000), (0x08B10000, 2, 0x20800000)] {
        memory.u32(system, &[replace]);
        memory.u32(system + d::TEXTURES, &[system + 0x100, count]);
        for i in 0..count {
            memory.u32(system + 0x100 + 4 * i, &[system + 0x200 + 0x40 * i]);
            memory.u32(system + 0x200 + 0x40 * i + d::IMAGE, &[base + 0x1000 * i]);
        }
        systems.insert(system, d::REPLACE_CLASS.to_string());
    }
    let (citrus, item) = (0x08B20000, 0x08B21000);
    for (picture, image) in [(citrus, 0x20702000), (item, 0x20801000)] {
        memory.u32(picture + lyt::PICTURE_MATERIAL, &[picture + 0x400]);
        memory.u32(picture + 0x400 + lyt::MATERIAL_TEXTURE, &[image]);
    }
    let mut puffs = amie::Puffs::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        kinds,
    );
    puffs.track(&systems, false).unwrap();
    assert_eq!(
        (puffs.name(citrus).unwrap(), puffs.name(item).unwrap()),
        (Some("Citrus Poké Puff".to_string()), None)
    );
}

/// The grid's slot i draws texture i of its page's system of fifteen, and the pages' block,
/// found a heap block back from the G2DUtil it points at, lists each page's items by number:
/// or_pokemon_me2's cushions page held the Leaf Cushion first, its objects page the Pink Star
/// Cloud (spike 228).
#[test]
fn test_an_interior_item_is_named_by_its_slot_on_the_page_shown() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let (g2d, between, block, system, picture) =
        (0x08C10030, 0x08C08000, 0x08C00030, 0x08C20000, 0x08C30000);
    memory.u32(g2d, &[classes.add(amie::G2D_CLASS)]);
    memory.u32(g2d - 0x30 + amie::PREVIOUS, &[between]);
    memory.u32(between + amie::PREVIOUS, &[block - 0x30]);
    let (puffs, replace) = (0x08C40000, classes.add(d::REPLACE_CLASS));
    memory.u32(puffs, &[replace]);
    memory.u32(block + amie::G2D, &[g2d]);
    memory.u32(block + amie::PUFFS, &[puffs]);
    memory.write(block + amie::LISTS, &[24]);
    memory.write(block + amie::LISTS + amie::PAGE_LISTS[1], &[146]);
    memory.u32(system, &[replace]);
    memory.u32(system + d::TEXTURES, &[system + 0x100, amie::SLOTS]);
    memory.u32(system + 0x100, &[system + 0x200]);
    memory.u32(system + 0x200 + d::IMAGE, &[0x20900000]);
    memory.u32(picture + lyt::PICTURE_MATERIAL, &[picture + 0x400]);
    memory.u32(picture + 0x400 + lyt::MATERIAL_TEXTURE, &[0x20900000]);
    let mut text: Vec<String> = vec!["Wallpaper".to_string(); amie::ITEMS_FROM as usize];
    text.extend(vec!["Cushion".to_string(); amie::ITEMS as usize]);
    text[(amie::ITEMS_FROM + 24) as usize] = "Leaf Cushion".to_string();
    text[(amie::ITEMS_FROM + 146) as usize] = "Pink Star Cloud".to_string();
    let mut goods = amie::Goods::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        text,
    );
    goods
        .track(
            &IndexMap::from([
                (g2d, amie::G2D_CLASS.to_string()),
                (puffs, d::REPLACE_CLASS.to_string()),
                (system, d::REPLACE_CLASS.to_string()),
            ]),
            false,
        )
        .unwrap();
    assert_eq!(
        goods.name(picture).unwrap().as_deref(),
        Some("Leaf Cushion")
    );
    memory.write(block + amie::PAGE, &[1]);
    assert_eq!(
        goods.name(picture).unwrap().as_deref(),
        Some("Pink Star Cloud")
    );
}
