//! The Pokémon-Amie page's Pokémon is named by its species, through the icon it is drawn with.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::keyfile::data_path;
use crate::core::rtti::ClassNames;
use crate::core::words::{self, ENGLISH, Words};
use crate::games::common::field::keys::STYLUS;
use crate::games::common::lyt;
use crate::games::common::pictures::named;
use crate::games::common::touch::Target;
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::screens::distribution as d;
use crate::games::gen6::subscreen::amie;
use crate::games::gen6::subscreen::berry_picker::Touching;
use crate::games::xy;
use crate::py::Result;

pub(crate) const PAGE: i64 = 0x08A00000;
const HOLDER: i64 = 0x08A01000;
const ICON: i64 = 0x08A02000;
const REPLACE: i64 = 0x08A03000;
const TABLE: i64 = 0x08A04000;
const TEXTURE: i64 = 0x08A05000;
pub(crate) const WALKER: i64 = 0x08A06000;
const OTHER: i64 = 0x08A07000;
const IMAGE: i64 = 0x18200000;

pub(crate) fn species() -> Vec<String> {
    let mut names = vec![String::new(); 254];
    names.push("Sceptile".to_string());
    names
}

pub(crate) fn page(number: i64) -> (amie::AmiePage, Rc<FakeMemory>) {
    page_aimed(number, false)
}

/// `page(number, aimed)`.
fn page_aimed(number: i64, aimed: bool) -> (amie::AmiePage, Rc<FakeMemory>) {
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
    let mut found = amie::AmiePage::with_aim(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        species(),
        number,
        aimed,
    );
    found
        .track(&IndexMap::from([(PAGE, amie::CLASS.to_string())]), false)
        .unwrap();
    (found, memory)
}

#[test]
fn test_the_picture_drawing_the_icons_first_texture_is_named_by_its_species() {
    assert_eq!(
        page(amie::NUMBER)
            .0
            .name(WALKER, Some(FIELD))
            .unwrap()
            .as_deref(),
        Some("Sceptile")
    );
}

#[test]
fn test_xs_page_is_read_where_x_keeps_its_pokemon() {
    assert_eq!(
        page(xy::game().amie)
            .0
            .name(WALKER, Some(FIELD))
            .unwrap()
            .as_deref(),
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
    assert_eq!(page(amie::NUMBER).0.name(OTHER, Some(FIELD)).unwrap(), None);
}

#[test]
fn test_amie_itself_offers_its_pokemon_to_stroke_by_the_species_the_page_showed() {
    let (mut found, memory) = page(amie::NUMBER);
    memory.u32(PAGE, &[0]); // the page freed as Amie runs
    assert!(
        found
            .targets(Some("N5field12FieldmapProcE"))
            .unwrap()
            .is_empty()
    );
    let strokes = found.targets(Some(amie::AMIE)).unwrap();
    let [stroke, _stylus] = strokes.as_slice() else {
        panic!("a stroke and the stylus: {strokes:?}");
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

const PLAYER: i64 = 0x08A08000;
const COPY: i64 = 0x08A09000;
const ALLOCATOR: i64 = 0x08A0A000;
const BUFFER: i64 = 0x14100000;

/// A picture of the Pokémon's parts in the current buffer: a grey a rectangle, (left, top,
/// right, bottom), the later over the earlier.
fn pokemon(memory: &FakeMemory, parts: &[(u8, (i64, i64, i64, i64))]) {
    let mut raw = vec![0u8; (320 * amie::COLUMN) as usize];
    for &(grey, (left, top, right, bottom)) in parts {
        for x in left..right {
            for y in top..bottom {
                let at = (x * amie::COLUMN + (239 - y) * 4) as usize;
                raw[at..at + 4].copy_from_slice(&[0xFF, grey, grey, grey]);
            }
        }
    }
    memory.write(BUFFER + amie::PICTURE, &raw);
}

fn amie_running(parts: &[(u8, (i64, i64, i64, i64))], aimed: bool) -> amie::AmiePage {
    let (mut found, memory) = page_aimed(amie::NUMBER, aimed);
    let classes = FakeClasses::starting(memory.clone(), CODE.0 + 0x8000);
    let player = classes.add(amie::PLAYER);
    memory.u32(PLAYER, &[player]);
    memory.u32(PLAYER + amie::HOLDS_PICTURE, &[ALLOCATOR]);
    memory.u32(ALLOCATOR, &[classes.add(amie::PICTURE_HOLDER)]);
    memory.u32(ALLOCATOR + amie::BUFFERS, &[BUFFER - 0x80000, BUFFER, 1]);
    // A copy of a Player's first words in the module's code, which a sweep finds too (spike 279).
    memory.u32(COPY, &[player]);
    memory.u32(COPY + amie::HOLDS_PICTURE, &[0xE3550000]);
    pokemon(&memory, parts);
    found
        .track(
            &IndexMap::from([
                (COPY, amie::PLAYER.to_string()),
                (PLAYER, amie::PLAYER.to_string()),
            ]),
            true,
        )
        .unwrap();
    found
}

/// The game tells the Pokémon's parts by a picture of it a grey a part, 8 its head and 9 its
/// body on every Pokémon (spike 280); the stroke stays on plain head, off its eyes (4 and 6).
#[test]
fn test_the_stroke_goes_along_the_widest_row_of_the_pokemons_head_in_the_games_picture() {
    let mut found = amie_running(
        &[
            (9, (110, 120, 210, 230)),
            (8, (100, 60, 220, 120)),
            (4, (130, 60, 140, 120)),
            (6, (180, 80, 190, 100)),
        ],
        true,
    );
    let strokes = found.targets(Some(amie::AMIE)).unwrap();
    let [stroke, _stylus] = strokes.as_slice() else {
        panic!("a stroke and the stylus: {strokes:?}");
    };
    // Rows 80 to 100 are broken by both eyes; above, only the right eye: 140 to 219 is 80 wide.
    // Back and forth as far as the old stroke went, 500 pixels: 9 passes of 55.
    assert_eq!(stroke.path, [(152, 62), (207, 62)].repeat(5));
    assert_eq!(
        (stroke.name.as_str(), stroke.seconds, stroke.drop),
        ("stroke Sceptile", amie::STROKE_SECONDS, Some(amie::MOUTH))
    );
}

#[test]
fn test_a_head_too_narrow_to_stroke_leaves_the_stroke_on_the_body() {
    let mut found = amie_running(&[(9, (100, 120, 220, 230)), (8, (150, 90, 170, 120))], true);
    assert_eq!(
        found.targets(Some(amie::AMIE)).unwrap()[0].path,
        [(118, 122), (201, 122)].repeat(4)[..7]
    );
}

#[test]
fn test_with_no_picture_the_stroke_crosses_the_middle_as_before() {
    let mut found = amie_running(&[], true);
    assert_eq!(
        found.targets(Some(amie::AMIE)).unwrap()[0].path,
        amie::STROKE
    );
}

/// Reading what the GPU draws into crashed Orion's Azahar 2126.1.1 twice as he stroked
/// Pikachu (spike 149's race); before 2126.2 the stroke crosses the middle as it always did.
#[test]
fn test_an_azahar_that_cannot_read_beside_the_gpu_has_its_picture_left_alone() {
    let mut found = amie_running(&[(8, (100, 60, 220, 120))], false);
    assert_eq!(
        found.targets(Some(amie::AMIE)).unwrap()[0].path,
        amie::STROKE
    );
}

/// A grey is a kind of part, the same on every Pokémon (spike A): 17 Pikachu's cheek, which
/// shocks the hand. A sweet spot (27, 28) or a danger zone (25, 26) is said by the Pokémon's own
/// name for it in `amie.ftl`, Sceptile's 28 the leaves on its arms; one it has none for, as
/// the head or the body, the side of the pair each is (Orion, 2026-11-05).
#[test]
fn test_the_stylus_is_offered_by_the_kind_of_part_it_is_on() {
    let mut found = amie_running(
        &[
            (9, (100, 120, 220, 230)),
            (8, (100, 60, 220, 120)),
            (17, (180, 100, 190, 110)),
            (27, (140, 60, 180, 70)),
            (25, (100, 60, 110, 70)),
            (28, (120, 200, 130, 210)),
            (0, (200, 200, 210, 210)),
        ],
        true,
    );
    let mut stylus = |x: i64, y: i64| -> String {
        found.stylus = (x, y);
        let target = found.targets(Some(amie::AMIE)).unwrap()[1].clone();
        assert_eq!(
            (target.pane, target.x, target.y),
            (amie::STYLUS_TARGET, x, y)
        );
        target.name
    };
    assert_eq!(stylus(150, 90), "stylus, head");
    assert_eq!(stylus(185, 105), "stylus, sparks");
    let (top, corner) = (stylus(150, 65), stylus(105, 65));
    assert!(top == corner && corner == "stylus, head");
    assert_eq!(stylus(150, 200), "stylus, body");
    assert_eq!(stylus(125, 205), "stylus, leaves on its arms");
    let (away, grey_0) = (stylus(10, 10), stylus(205, 205));
    assert!(away == grey_0 && grey_0 == "stylus, off");
}

#[test]
fn test_where_the_picture_cannot_be_read_the_stylus_is_offered_alone() {
    let mut found = amie_running(&[(8, (100, 60, 220, 120))], false);
    assert_eq!(found.targets(Some(amie::AMIE)).unwrap()[1].name, "stylus");
}

/// Touch mode as the stylus's keys use it: where it was when each move chose it.
struct Chosen {
    page: Rc<RefCell<amie::AmiePage>>,
    chosen: Vec<(i64, (i64, i64))>,
}

impl Touching for Chosen {
    fn targets(&mut self) -> Result<Vec<Target>> {
        Ok(Vec::new())
    }

    fn give(&mut self, _picked: &Target, _over: &Target) -> String {
        String::new()
    }

    fn tap(&mut self, _target: &Target) -> String {
        String::new()
    }

    fn choose(&mut self, pane: i64) -> Result<String> {
        self.chosen.push((pane, self.page.borrow().stylus));
        Ok(String::new())
    }
}

/// Orion, 2026-11-05: the arrows move the stylus over the Pokémon, a step a press, and touch
/// mode chooses it as it goes, to tap or drag there.
#[test]
fn test_the_arrows_move_the_stylus_saying_a_part_as_it_comes_onto_one() {
    let found = Rc::new(RefCell::new(amie_running(
        &[(8, (100, 60, 220, 120))],
        true,
    )));
    let mut chosen = Chosen {
        page: found.clone(),
        chosen: Vec::new(),
    };
    let mut keys = amie::StylusKeys::new(found.clone(), Rc::new(|line: &str| format!("! {line}")));
    assert!(keys.context(Some(amie::AMIE)) == Some(STYLUS) && keys.context(Some(FIELD)).is_none());
    found.borrow_mut().stylus = (160, 126);
    let mut press =
        |action: &str, chosen: &mut Chosen| keys.press(action, Some(amie::AMIE), chosen).unwrap();
    assert_eq!(press("stylus up", &mut chosen), "off");
    assert_eq!(press("stylus up", &mut chosen), "head");
    assert_eq!(press("stylus up", &mut chosen), ""); // still the head
    assert_eq!(press("stylus right", &mut chosen), "");
    assert_eq!(
        chosen.chosen,
        [
            (amie::STYLUS_TARGET, (160, 120)),
            (amie::STYLUS_TARGET, (160, 114)),
            (amie::STYLUS_TARGET, (160, 108)),
            (amie::STYLUS_TARGET, (166, 108))
        ]
    );
    found.borrow_mut().stylus = (2, 0);
    assert_eq!(press("stylus left", &mut chosen), "off");
    assert_eq!(press("stylus left", &mut chosen), "! edge");
    assert!(press("stylus up", &mut chosen) == "! edge" && found.borrow().stylus == (0, 0));
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
        puffs.name(citrus, Some(amie::ARRANGE)).unwrap().as_deref(),
        Some("Citrus Poké Puff")
    );
    assert_eq!(puffs.name(item, Some(amie::ARRANGE)).unwrap(), None);
    assert_eq!(puffs.name(citrus, Some(FIELD)).unwrap(), None);
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
        goods.name(picture, Some(amie::ARRANGE)).unwrap().as_deref(),
        Some("Leaf Cushion")
    );
    memory.write(block + amie::PAGE, &[1]);
    assert_eq!(
        goods.name(picture, Some(amie::ARRANGE)).unwrap().as_deref(),
        Some("Pink Star Cloud")
    );
}
