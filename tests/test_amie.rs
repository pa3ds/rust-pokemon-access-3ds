//! The Pokémon-Amie page's Pokémon is named by its species, through the icon it is drawn with.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_memories::record;
use crate::core::keyfile::data_path;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::core::words::{self, ENGLISH, Words};
use crate::games::common::field::keys::{STYLUS, STYLUS_NUMBERS};
use crate::games::common::layout::Layouts;
use crate::games::common::pictures::named;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::{ImageOf, Target};
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::screens::distribution as d;
use crate::games::gen6::screens::partyicons::COUNT;
use crate::games::gen6::subscreen::berry_picker::Touching;
use crate::games::gen6::subscreen::{amie, amie_switch};
use crate::games::xy;
use crate::py::{self, Result, time};

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
    page_aimed(number, false, 0)
}

/// `page(number, aimed, later)`.
fn page_aimed(number: i64, aimed: bool, later: i64) -> (amie::AmiePage, Rc<FakeMemory>) {
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
        later,
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

/// Orion, 2026-11-05: the Pokémon is offered to beckon, a press of 0.4 s, as presses held
/// apart turned Ralts round (spike 276); a puff is dragged to its mouth (spike 257).
#[test]
fn test_amie_itself_offers_its_pokemon_to_beckon_by_the_species_the_page_showed() {
    let (mut found, memory) = page(amie::NUMBER);
    memory.u32(PAGE, &[0]); // the page freed as Amie runs
    assert!(
        found
            .targets(Some("N5field12FieldmapProcE"))
            .unwrap()
            .is_empty()
    );
    let targets = found.targets(Some(amie::AMIE)).unwrap();
    let [beckon, _stylus] = targets.as_slice() else {
        panic!("a beckon and the stylus: {targets:?}");
    };
    assert_eq!(
        (
            beckon.name.as_str(),
            beckon.path.clone(),
            beckon.seconds,
            beckon.rub
        ),
        (
            "beckon Sceptile",
            vec![amie::BECKON],
            amie::BECKON_SECONDS,
            None
        )
    );
    assert!(beckon.drop == Some(amie::MOUTH) && beckon.drop != Some((beckon.x, beckon.y)));
}

const PLAYER: i64 = 0x08A08000;
const COPY: i64 = 0x08A09000;
const ALLOCATOR: i64 = 0x08A0A000;
const BUFFER: i64 = 0x14100000;

/// A picture of the Pokémon's parts in the current buffer: a grey a rectangle, (left, top,
/// right, bottom), the later over the earlier.
fn drawn(memory: &FakeMemory, parts: &[(u8, (i64, i64, i64, i64))]) {
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
    running(parts, aimed).0
}

/// Amie running, its still picture taken of `drawn`'s: the page and the memory.
fn running(parts: &[(u8, (i64, i64, i64, i64))], aimed: bool) -> (amie::AmiePage, Rc<FakeMemory>) {
    running_later(parts, aimed, 0)
}

/// `running(parts, aimed, later)`.
fn running_later(
    parts: &[(u8, (i64, i64, i64, i64))],
    aimed: bool,
    later: i64,
) -> (amie::AmiePage, Rc<FakeMemory>) {
    let (mut found, memory) = page_aimed(amie::NUMBER, aimed, later);
    let classes = FakeClasses::starting(memory.clone(), CODE.0 + 0x8000);
    let player = classes.add(amie::PLAYER);
    memory.u32(PLAYER, &[player]);
    memory.u32(PLAYER + amie::HOLDS_PICTURE + later, &[ALLOCATOR]);
    memory.u32(ALLOCATOR, &[classes.add(amie::PICTURE_HOLDER)]);
    memory.u32(ALLOCATOR + amie::BUFFERS, &[BUFFER - 0x80000, BUFFER, 1]);
    // A copy of a Player's first words in the module's code, which a sweep finds too (spike 279).
    memory.u32(COPY, &[player]);
    memory.u32(COPY + amie::HOLDS_PICTURE, &[0xE3550000]);
    drawn(&memory, parts);
    found
        .track(
            &IndexMap::from([
                (COPY, amie::PLAYER.to_string()),
                (PLAYER, amie::PLAYER.to_string()),
            ]),
            true,
        )
        .unwrap();
    found.settle().unwrap();
    (found, memory)
}

/// Rectangles of a Pokémon's head and body in its picture of parts, x 100 to 219.
const HEAD: (i64, i64, i64, i64) = (100, 60, 220, 120);
const BODY: (i64, i64, i64, i64) = (100, 120, 220, 230);

/// `monkeypatch.setattr(amie.time, "monotonic", lambda: now[0])`: this thread's clock reads
/// `now` until the guard drops.
struct Frozen(Rc<Cell<f64>>);

impl time::Clock for Frozen {
    fn monotonic(&mut self) -> f64 {
        self.0.get()
    }

    fn sleep(&mut self, _seconds: f64, _ends_tick: bool) {}
}

struct Thawed;

impl Drop for Thawed {
    fn drop(&mut self) {
        time::set_clock(None);
    }
}

fn frozen(now: &Rc<Cell<f64>>) -> Thawed {
    time::set_clock(Some(Box::new(Frozen(now.clone()))));
    Thawed
}

/// Where a touch on the stylus rubs now.
fn rubbed(found: &mut amie::AmiePage) -> ((i64, i64), (i64, i64)) {
    found.targets(Some(amie::AMIE)).unwrap()[1]
        .rub
        .expect("a rub")
}

/// Amie counts a pet only while the hand moves 4 pixels a tick on one kind of part (spike A):
/// a rub stays on the part, `RUB_REACH` at most either side, and `RUB_HALF` at least.
#[test]
fn test_a_rub_goes_across_the_part_the_stylus_is_on_the_way_it_is_longer() {
    let mut found = amie_running(
        &[
            (9, BODY),
            (8, HEAD),
            (27, (140, 60, 180, 70)),
            (17, (180, 100, 186, 140)),
            (25, (100, 60, 108, 66)),
        ],
        true,
    );
    let mut rub = |x: i64, y: i64| {
        found.stylus = (x, y);
        found.keep(None);
        rubbed(&mut found)
    };
    let ((left, y), (right, same_y)) = rub(150, 90); // the head: across, wider than tall
    assert!(y == same_y && right - left == 24 && left >= 100 && right < 180);
    let ((x, top), (same_x, bottom)) = rub(183, 120); // a cheek's strip: down it
    assert!(
        (180..186).contains(&x) && x == same_x && bottom - top == 24 && top >= 100 && bottom < 140
    );
    let ((left, y), (right, same_y)) = rub(160, 65); // a sweet spot's strip: across it
    assert!(
        y == same_y && (60..70).contains(&y) && right - left == 24 && left >= 140 && right < 180
    );
    let ((left, _y), (right, _same_y)) = rub(104, 62); // an ear tip, 8 across: a little past it
    assert_eq!(right - left, 12);
    // Off the Pokémon it holds still: a rub reaching back onto it never raised its hand.
    assert_eq!(rub(20, 90), ((24, 90), (24, 90)));
    // Where the picture cannot be read, before Azahar 2126.2, it rubs where it is all the same.
    assert_eq!(
        amie_running(&[(8, HEAD)], false)
            .targets(Some(amie::AMIE))
            .unwrap()[1]
            .rub,
        Some(((154, 120), (166, 120)))
    );
}

/// A grey is a kind of part, the same on every Pokémon (spike A): 17 Pikachu's cheek, which
/// shocks the hand. A sweet spot (27, 28) or a danger zone (25, 26) is said by the Pokémon's own
/// name for it in `amie.ftl`, Sceptile's 28 its arm leaves; one it has none for, as
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
    assert_eq!(stylus(125, 205), "stylus, arm leaves");
    let (away, grey_0) = (stylus(10, 10), stylus(205, 205));
    assert!(away == grey_0 && grey_0 == "stylus, off Sceptile");
}

/// Furfrou's trims are drawn by models named `pm0701_12_poodle` and on, past the 12 bytes
/// read; and the sweep finds a copy of the Player's words in the module's code first, which leads
/// to no model (spike A's census).
#[test]
fn test_a_form_is_told_by_its_models_name_though_the_name_runs_on() {
    let (mut found, memory) = running(&[(8, HEAD)], true);
    let (model, resource, records, name) = (0x08A0B000, 0x08A0C000, 0x08A0D000, 0x08A0E000);
    memory.u32(PLAYER + amie::MODEL, &[model]);
    memory.u32(model + amie::MODEL_RESOURCE, &[resource]);
    memory.u32(resource + amie::RECORDS, &[records]);
    memory.u32(records + amie::RECORD_NAME, &[name]);
    memory.write(name, b"pm0701_12_poodle\0");
    found.stylus = (150, 90);
    assert!(
        found.part(None).unwrap().as_deref() == Some("head")
            && found.form().unwrap().as_deref() == Some("12")
    );
}

/// One Player's address serves one Pokémon after another: Pikachu Rock Star, after a
/// Metagross, took its model, had no names of its own, and said its ears, a danger zone, as the
/// head (or_amie_interactions). And its model is not there yet as Amie opens.
#[test]
fn test_a_form_is_read_anew_as_amie_opens_on_another_pokemon() {
    let mut k = keyed(&[(8, HEAD)]);
    let (model, resource, records, name) = (0x08A0B000, 0x08A0C000, 0x08A0D000, 0x08A0E000);
    assert_eq!(k.found.borrow_mut().form().unwrap(), None); // not loaded
    k.memory.u32(PLAYER + amie::MODEL, &[model]);
    k.memory.u32(model + amie::MODEL_RESOURCE, &[resource]);
    k.memory.u32(resource + amie::RECORDS, &[records]);
    k.memory.u32(records + amie::RECORD_NAME, &[name]);
    k.memory.write(name, b"pm0376_00\0");
    assert_eq!(k.found.borrow_mut().form().unwrap().as_deref(), Some("00"));
    k.poll(FIELD);
    k.memory.write(name, b"pm0025_12\0");
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(k.found.borrow_mut().form().unwrap().as_deref(), Some("12"));
}

/// Every pixel is 255 before the Pokémon is first drawn. Taken for a part, Amie opened on a
/// "part" filling the screen, and a Pikachu was offered to be brought down
/// (or_amie_interactions).
#[test]
fn test_a_picture_not_yet_drawn_is_no_part() {
    let mut k = keyed(&[]);
    k.memory.write(
        BUFFER + amie::PICTURE,
        &vec![0xFF; (320 * amie::COLUMN) as usize],
    );
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(k.found.borrow_mut().part(None).unwrap(), None);
    let names: Vec<String> = k
        .found
        .borrow_mut()
        .targets(Some(amie::AMIE))
        .unwrap()
        .into_iter()
        .map(|target| target.name)
        .collect();
    assert_eq!(names, ["beckon Sceptile", "stylus"]);
    drawn(&k.memory, &[(8, HEAD)]);
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(k.found.borrow().outline(), Some((100, 60, 219, 119)));
}

#[test]
fn test_where_the_picture_cannot_be_read_the_stylus_is_offered_alone() {
    let mut found = amie_running(&[(8, (100, 60, 220, 120))], false);
    assert_eq!(found.targets(Some(amie::AMIE)).unwrap()[1].name, "stylus");
}

/// Touch mode as the stylus's keys use it: where the stylus was when each move chose it, and
/// when each poll carried a rub on, choosing nothing.
struct Chosen {
    page: Rc<RefCell<amie::AmiePage>>,
    chosen: Vec<(i64, i64)>,
    carried: Vec<(i64, i64)>,
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
        assert_eq!(pane, amie::STYLUS_TARGET);
        self.chosen.push(self.page.borrow().stylus);
        Ok(String::new())
    }

    fn carry(&mut self, pane: i64) -> Result<()> {
        assert_eq!(pane, amie::STYLUS_TARGET);
        self.carried.push(self.page.borrow().stylus);
        Ok(())
    }
}

/// Amie running with the stylus's keys, its clock the cell's one number, to move on: the page,
/// the memory, the keys, touch mode as they use it, the clock, and the clock's guard.
struct Keyed {
    found: Rc<RefCell<amie::AmiePage>>,
    memory: Rc<FakeMemory>,
    keys: amie::StylusKeys,
    chosen: Chosen,
    now: Rc<Cell<f64>>,
    _thawed: Thawed,
}

impl Keyed {
    fn press(&mut self, action: &str) -> String {
        self.keys
            .press(action, Some(amie::AMIE), &mut self.chosen)
            .unwrap()
    }

    fn poll(&mut self, running: &str) -> Vec<(String, String, Option<Category>)> {
        self.keys.poll(Some(running), &mut self.chosen).unwrap()
    }

    fn rubbed(&self) -> ((i64, i64), (i64, i64)) {
        rubbed(&mut self.found.borrow_mut())
    }

    fn part(&self) -> Option<String> {
        self.found.borrow_mut().part(None).unwrap()
    }
}

fn keyed(parts: &[(u8, (i64, i64, i64, i64))]) -> Keyed {
    let now = Rc::new(Cell::new(100.0));
    let thawed = frozen(&now);
    let (found, memory) = running(parts, true);
    let found = found.shared();
    let chosen = Chosen {
        page: found.clone(),
        chosen: Vec::new(),
        carried: Vec::new(),
    };
    let keys = amie::StylusKeys::new(
        found.clone(),
        Rc::new(|line: &str| format!("! {line}")),
        Vec::new(),
    );
    Keyed {
        found,
        memory,
        keys,
        chosen,
        now,
        _thawed: thawed,
    }
}

/// Orion, 2026-11-05: the arrows move the stylus over the Pokémon, a step a press, and touch
/// mode chooses it as it goes, to rub there; a poll chooses nothing in touch mode; and the arrows
/// keep it within a step of the Pokémon.
#[test]
fn test_the_arrows_move_the_stylus_saying_a_part_as_it_comes_onto_one() {
    let mut k = keyed(&[(8, HEAD)]);
    assert!(
        k.keys.context(Some(amie::AMIE)) == Some(STYLUS) && k.keys.context(Some(FIELD)).is_none()
    );
    k.found.borrow_mut().stylus = (160, 126);
    assert_eq!(k.press("stylus up"), "off Sceptile");
    assert_eq!(k.press("stylus up"), "head");
    assert_eq!(k.press("stylus up"), ""); // still the head
    assert_eq!(k.press("stylus right"), "");
    assert_eq!(
        k.chosen.chosen,
        [(160, 120), (160, 114), (160, 108), (166, 108)]
    );
    assert!(k.poll(amie::AMIE).is_empty() && k.chosen.carried == [(166, 108)]);
    assert!(k.poll(FIELD).is_empty() && k.chosen.carried.len() == 1); // Amie not running
    assert_eq!(k.chosen.chosen.len(), 4); // the poll chose nothing
    drop(k);
    let mut k = keyed(&[(8, HEAD)]);
    k.found.borrow_mut().stylus = (100, 60); // the head's top left corner
    assert_eq!(k.press("stylus left"), "off Sceptile");
    assert_eq!(k.press("stylus left"), "! edge"); // a step off the Pokémon at most
    assert_eq!(k.press("stylus up"), "");
    assert!(k.press("stylus up") == "! edge" && k.found.borrow().stylus == (94, 54));
}

/// Orion could not find a hand or fur a step at a time (2026-11-05): a jump goes past the
/// part the stylus is on and any gap, to the middle of the next part's run, and says it.
#[test]
fn test_ctrl_and_an_arrow_jump_the_stylus_to_the_next_part_that_way() {
    let mut k = keyed(&[(9, BODY), (8, HEAD), (17, (180, 100, 186, 140))]);
    k.found.borrow_mut().stylus = (120, 105);
    assert_eq!(k.press("stylus jump right"), "sparks");
    assert_eq!(k.found.borrow().stylus, (182, 105));
    assert_eq!(k.press("stylus jump right"), "head");
    assert_eq!(k.found.borrow().stylus, (202, 105));
    k.found.borrow_mut().stylus = (110, 90);
    assert!(k.press("stylus jump left") == "! edge" && k.found.borrow().stylus == (110, 90));
}

/// A sweet spot with no name of its own is said as the head, and a jump went from the head to
/// the head (Orion, 2026-11-06).
#[test]
fn test_a_jump_goes_past_a_part_said_as_the_one_it_left() {
    let mut k = keyed(&[(9, BODY), (8, HEAD), (27, (140, 80, 180, 100))]);
    k.found.borrow_mut().stylus = (160, 110);
    assert_eq!(k.press("stylus jump down"), "body");
    k.found.borrow_mut().stylus = (160, 110);
    assert_eq!(k.press("stylus jump up"), "! edge");
}

/// Pikachu Rock Star's head has specks in its forehead, which a step or a jump landed on and
/// called the head again (spike A); the same pixels stay where they are all of a part, as a
/// Metagross's eyes are.
#[test]
fn test_a_speck_of_a_part_is_left_out_of_the_still_picture() {
    let mut k = keyed(&[(8, HEAD)]);
    drawn(
        &k.memory,
        &[
            (8, HEAD),
            (17, (140, 70, 180, 100)),
            (4, (120, 70, 122, 72)),
        ],
    );
    for y in [85, 86, 87] {
        k.memory.write(
            BUFFER + amie::PICTURE + 160 * amie::COLUMN + (239 - y) * 4,
            &[0xFF, 8, 8, 8],
        );
    }
    k.found.borrow_mut().look_again(0.0);
    k.now.set(k.now.get() + 2.0);
    k.poll(amie::AMIE);
    k.found.borrow_mut().stylus = (160, 86);
    assert_eq!(k.part().as_deref(), Some("sparks"));
    k.found.borrow_mut().stylus = (120, 70);
    assert_eq!(k.part().as_deref(), Some("right eye"));
}

const COMMAND: i64 = 0x08A0F000;
const PARAM: i64 = 0x08A10000;

/// Touch mode as the moods use it: what is picked up.
struct Held(Option<String>);

impl Touching for Held {
    fn targets(&mut self) -> Result<Vec<Target>> {
        Ok(Vec::new())
    }

    fn give(&mut self, _picked: &Target, _over: &Target) -> String {
        String::new()
    }

    fn tap(&mut self, _target: &Target) -> String {
        String::new()
    }

    fn picked(&self) -> Option<String> {
        self.0.clone()
    }
}

/// Amie's love command as `number`, or none running.
fn mood(memory: &FakeMemory, number: Option<i64>) {
    memory.u32(
        PLAYER + amie::LOVE,
        &[if number.is_none() { 0 } else { COMMAND }],
    );
    memory.write(
        COMMAND + amie::LOVE_NUMBER,
        &(number.unwrap_or(0) as u16).to_le_bytes(),
    );
}

/// Orion, 2026-11-05: the moods a sighted player sees, read off the love command, which needs
/// no picture and so reads on any Azahar; a press held off the Pokémon and a puff eaten in two
/// bites, as the script has it, with a moment of no love command between two.
#[test]
fn test_what_the_pokemon_is_seen_doing_is_said_as_it_starts_and_stops() {
    let (found, memory) = running(&[(8, HEAD)], false);
    let mut held = Held(None);
    let mut moods = amie::Moods::new(Rc::new(RefCell::new(found)));
    let mut said = |numbers: &[Option<i64>], held: &mut Held| -> Vec<String> {
        let mut lines = Vec::new();
        for &number in numbers {
            mood(&memory, number);
            lines.extend(
                moods
                    .poll(Some(amie::AMIE), held)
                    .unwrap()
                    .into_iter()
                    .map(|(_log, line, _category)| line),
            );
        }
        lines
    };
    assert_eq!(
        said(&[Some(7), Some(7)], &mut held),
        ["Sceptile has its back turned"]
    );
    assert_eq!(said(&[Some(0)], &mut held), ["Sceptile turned round"]);
    // a pet is the player's own doing
    assert!(said(&[Some(13), None, Some(0)], &mut held).is_empty());
    assert_eq!(
        said(&[Some(22), Some(24), None, Some(24)], &mut held),
        [
            "Sceptile is watching your hand",
            "Sceptile is holding up its hand",
            "Sceptile is holding up its hand"
        ]
    );
    assert_eq!(said(&[Some(23)], &mut held), ["Sceptile is looking around"]);
    assert_eq!(
        said(&[15, 17, 18, 19, 15, 17, 18, 19, 0].map(Some), &mut held),
        ["Sceptile is eating", "Sceptile is eating"]
    ); // a puff no key of pa3ds's picked up
    held.0 = Some("Mint Poké Puff".to_string());
    assert_eq!(
        said(&[15, 17, 18, 19, 15, 17, 18, 19].map(Some), &mut held),
        [
            "Sceptile is eyeing the Mint Poké Puff",
            "Sceptile is eating",
            "Sceptile is eating"
        ]
    );
    held.0 = None; // let go as it was eaten
    assert_eq!(
        said(&[Some(0)], &mut held),
        ["Sceptile finished the Mint Poké Puff"]
    );
    assert_eq!(
        said(&[Some(22)], &mut held),
        ["Sceptile is watching your hand"]
    );
    assert!(moods.poll(Some(FIELD), &mut held).unwrap().is_empty());
    let mut said = |number: i64| -> Vec<String> {
        mood(&memory, Some(number));
        moods
            .poll(Some(amie::AMIE), &mut held)
            .unwrap()
            .into_iter()
            .map(|(_log, line, _category)| line)
            .collect()
    };
    assert_eq!(said(22), ["Sceptile is watching your hand"]); // Amie opened again
}

/// Orion, 2026-10-16: a Pokémon of the player's by the nickname they gave it. Nothing says
/// which of the party Amie shows, so the party's records are matched by its species.
#[test]
fn test_the_pokemon_is_called_by_its_nickname_unless_two_of_the_party_share_its_species() {
    let (mut found, memory) = running(&[(8, HEAD)], false);
    let classes = FakeClasses::starting(memory.clone(), CODE.0 + 0x9000);
    let vtable = classes.add(pokemon::PARAM_CLASS);
    for (param, blob) in [
        (PARAM, record(0x101, 254, "Blaze", true)),
        (PARAM + 0x400, record(0x202, 25, "Pikachu", false)),
        (PARAM + 0x800, record(0x101, 254, "Blaze", true)), // a copy of the first
    ] {
        memory.u32(param, &[vtable]);
        memory.u32(
            param + pokemon::PARAM_RECORD,
            &[param + pokemon::RECORD_IN_PARAM],
        );
        memory.write(param + pokemon::RECORD_IN_PARAM, &blob);
    }
    found
        .track(
            &IndexMap::from([
                (PARAM, pokemon::PARAM_CLASS.to_string()),
                (PARAM + 0x400, pokemon::PARAM_CLASS.to_string()),
                (PARAM + 0x800, pokemon::PARAM_CLASS.to_string()),
            ]),
            true,
        )
        .unwrap();
    assert_eq!(found.called().unwrap(), "Blaze");
    assert_eq!(
        found.targets(Some(amie::AMIE)).unwrap()[0].name,
        "beckon Blaze"
    );
    memory.write(
        PARAM + 0x800 + pokemon::RECORD_IN_PARAM,
        &record(0x303, 254, "Treecko", false),
    );
    found.track(&IndexMap::new(), true).unwrap();
    assert_eq!(found.called().unwrap(), "Sceptile");
}

/// Read where ORAS keeps them, X's stylus said no parts and its Pokémon no moods
/// (x_amie_head_it_hard).
#[test]
fn test_xs_player_keeps_its_love_command_and_picture_four_bytes_later() {
    let later = xy::game().amie_player;
    let (found, memory) = running_later(&[(8, HEAD)], true, later);
    memory.u32(PLAYER + amie::LOVE + later, &[COMMAND]);
    memory.write(COMMAND + amie::LOVE_NUMBER, &23u16.to_le_bytes());
    assert_eq!(found.mood().unwrap(), Some(23));
    assert_eq!(found.outline(), Some((100, 60, 219, 119)));
}

/// Orion, 2026-11-06: "describe the other moods too, even if nothing reaches them yet"; seen
/// started at will on the research rig (spike A's `look`).
#[test]
fn test_the_moods_no_movie_reached_are_said_too() {
    let (found, memory) = running(&[(8, HEAD)], false);
    let mut moods = amie::Moods::new(Rc::new(RefCell::new(found)));
    let mut held = Held(Some("Sweet Poké Puff".to_string()));
    let mut said = |number: i64| -> Vec<String> {
        mood(&memory, Some(number));
        moods
            .poll(Some(amie::AMIE), &mut held)
            .unwrap()
            .into_iter()
            .map(|(_log, line, _category)| line)
            .collect()
    };
    assert_eq!(said(3), ["Sceptile comes up to you"]);
    assert_eq!(said(9), ["Sceptile is asleep"]);
    assert_eq!(said(0), ["Sceptile woke up"]);
    assert_eq!(said(8), ["Sceptile is dozing off"]);
    assert_eq!(
        said(14),
        ["Sceptile woke up", "Sceptile is too full to eat"]
    );
    assert_eq!(said(16), ["Sceptile is reaching for the Sweet Poké Puff"]);
    assert_eq!(said(20), ["the Sweet Poké Puff fell"]);
    assert_eq!(said(21), ["Sceptile frowns and turns away"]);
    assert_eq!(said(27), ["Sceptile leans in to your face"]);
}

/// ADR-0009: the hand is up about 2 s and a player cannot see where, so a jump goes straight
/// to it, the stylus to the hand on the still picture and the touch to it as it is now; before
/// it, a jump is a jump, and so it is for a Pokémon with no hand drawn.
#[test]
fn test_while_its_hand_is_raised_a_jump_goes_to_the_hand_whichever_way() {
    let mut k = keyed(&[(9, BODY), (8, HEAD), (10, (120, 140, 130, 150))]);
    k.found.borrow_mut().stylus = (40, 100);
    assert_eq!(k.press("stylus jump right"), "head");
    k.found.borrow_mut().stylus = (40, 100);
    mood(&k.memory, Some(amie::HAND_RAISED));
    drawn(&k.memory, &[(9, BODY), (8, HEAD), (10, (180, 60, 200, 80))]); // it rose
    k.now.set(k.now.get() + 1.0);
    assert_eq!(k.press("stylus jump left"), "hand");
    let (x, y) = k.found.borrow().stylus;
    assert!((120..130).contains(&x) && (140..150).contains(&y));
    assert!((120..130).contains(&k.chosen.chosen.last().unwrap().0));
    let ((left, y), (right, _same_y)) = k.rubbed();
    assert!((180..200).contains(&py::floordiv(left + right, 2)) && (60..80).contains(&y));
    mood(&k.memory, Some(0));
    drawn(&k.memory, &[(9, BODY), (8, HEAD)]);
    k.now.set(k.now.get() + 1.0);
    k.found.borrow_mut().stylus = (120, 105);
    assert_eq!(k.press("stylus jump up"), "! edge");
}

#[test]
fn test_a_hand_the_still_picture_does_not_show_is_jumped_to_all_the_same() {
    let mut k = keyed(&[(9, BODY), (8, HEAD)]);
    k.found.borrow_mut().stylus = (40, 100);
    mood(&k.memory, Some(amie::HAND_RAISED));
    drawn(&k.memory, &[(9, BODY), (8, HEAD), (10, (180, 60, 200, 80))]);
    k.now.set(k.now.get() + 1.0);
    assert_eq!(k.press("stylus jump up"), "hand");
    let ((left, y), (right, _same_y)) = k.rubbed();
    assert!((180..200).contains(&py::floordiv(left + right, 2)) && (60..80).contains(&y));
}

/// Orion, 2026-11-06: the stylus moves over a picture taken as the Pokémon appears, however it
/// moves, and a tap or drag goes to the part chosen wherever it has gone. Touch mode's tone stays
/// where the player put it.
#[test]
fn test_over_a_still_picture_a_part_stays_where_it_was_found_and_a_touch_finds_it() {
    let mut k = keyed(&[(9, BODY), (8, HEAD)]);
    k.found.borrow_mut().stylus = (160, 120);
    assert_eq!(k.press("stylus up"), "head");
    drawn(
        &k.memory,
        &[(9, (160, 180, 280, 230)), (8, (160, 120, 280, 180))],
    ); // 60 right, 60 down
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(k.found.borrow().stylus, (160, 114));
    assert_eq!(k.press("stylus up"), ""); // still the head, where it was
    let stylus = k.found.borrow_mut().targets(Some(amie::AMIE)).unwrap()[1].clone();
    assert_eq!(
        (stylus.name.as_str(), stylus.x, stylus.y),
        ("stylus, head", 160, 108)
    );
    let ((left, y), (right, same_y)) = k.rubbed();
    assert!(y == same_y && (120..180).contains(&y) && 160 <= left && left < right && right < 280);
}

/// A part the picture no longer draws, an eye shut, is rubbed where it would be on the
/// Pokémon: as far across and down it as it was.
#[test]
fn test_a_touch_on_a_part_gone_from_view_goes_where_it_would_be() {
    let mut k = keyed(&[(9, BODY), (8, HEAD), (4, (150, 80, 170, 100))]);
    k.found.borrow_mut().stylus = (160, 90);
    k.press("stylus left");
    drawn(
        &k.memory,
        &[(9, (160, 120, 280, 230)), (8, (160, 60, 280, 120))],
    );
    k.now.set(k.now.get() + 1.0);
    let ((left, y), (right, _y)) = k.rubbed();
    assert!((190..=235).contains(&py::floordiv(left + right, 2)) && (70..=100).contains(&y));
}

/// A press held off the Pokémon raises its hand, and one touching it never does: beside it, as
/// it sways, a press can. Worked out from the Pokémon as it swayed, the press went back and forth
/// 40 pixels (or_amie_interactions); at the side of the screen it stays put.
#[test]
fn test_a_press_off_the_pokemon_goes_to_the_side_of_the_screen_clear_of_it() {
    let mut k = keyed(&[(9, BODY), (8, HEAD)]);
    k.found.borrow_mut().stylus = (98, 100);
    k.press("stylus left"); // off it, by the still picture
    assert_eq!(k.rubbed(), ((24, 100), (24, 100)));
    k.found.borrow_mut().stylus = (250, 20);
    k.found.borrow_mut().keep(None);
    assert_eq!(k.rubbed(), ((295, 60), (295, 60))); // below the corner's button
    drawn(
        &k.memory,
        &[(9, (0, 120, 120, 230)), (8, (0, 60, 120, 120))],
    ); // now at the left
    k.now.set(k.now.get() + 1.0);
    k.found.borrow_mut().stylus = (20, 100);
    k.found.borrow_mut().keep(None);
    assert_eq!(k.rubbed(), ((295, 100), (295, 100)));
}

/// Its first frames may catch the Pokémon looking away, or still walking up; after `RICHER` it
/// stays, for a joyful leap at the camera, and it is taken again only once the Pokémon comes
/// anew.
#[test]
fn test_the_still_picture_takes_in_more_of_the_pokemon_at_first_and_then_stays() {
    let mut k = keyed(&[(8, HEAD)]);
    mood(&k.memory, Some(0));
    k.found.borrow_mut().stylus = (160, 90);
    k.found.borrow_mut().keep(None);
    assert_eq!(k.part().as_deref(), Some("head"));
    drawn(&k.memory, &[(8, HEAD), (4, (150, 80, 170, 100))]);
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert!(
        k.found.borrow().outline() == Some((100, 60, 219, 119))
            && k.part().as_deref() == Some("head")
    );
    k.found.borrow_mut().stylus = (160, 90);
    assert_eq!(k.part().as_deref(), Some("right eye"));
    drawn(
        &k.memory,
        &[(8, (60, 60, 260, 200)), (4, (150, 80, 170, 100))],
    ); // it walked up
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(k.found.borrow().outline(), Some((60, 60, 259, 199)));
    k.found.borrow_mut().stylus = (160, 90);
    drawn(
        &k.memory,
        &[
            (8, (60, 60, 260, 200)),
            (4, (150, 80, 170, 100)),
            (6, (155, 85, 165, 95)),
        ],
    );
    mood(&k.memory, Some(13)); // being petted, it is no picture to keep
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(k.part().as_deref(), Some("right eye"));
    mood(&k.memory, Some(0));
    k.now.set(k.now.get() + amie::RICHER + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(k.part().as_deref(), Some("right eye")); // settled
    mood(&k.memory, Some(7));
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(k.part(), None); // back turned, no picture
    mood(&k.memory, Some(0));
    drawn(&k.memory, &[(8, HEAD)]);
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(k.part().as_deref(), Some("head"));
}

/// It comes closer to a beckon, and a far picture has parts narrower than a step. The stylus
/// stays on its part.
#[test]
fn test_a_beckon_takes_the_still_picture_again_once_the_pokemon_has_come_close() {
    let mut k = keyed(&[(8, (150, 100, 170, 120))]);
    k.found.borrow_mut().stylus = (160, 110);
    k.press("stylus up");
    let beckon = k.found.borrow_mut().targets(Some(amie::AMIE)).unwrap()[0].clone();
    let after = beckon.after.0.expect("told after");
    after();
    drawn(&k.memory, &[(8, (100, 60, 220, 180))]);
    k.now.set(k.now.get() + 0.5);
    k.poll(amie::AMIE);
    assert_eq!(k.found.borrow().outline(), Some((150, 100, 169, 119))); // not stopped yet
    k.now.set(k.now.get() + amie::BECKON_SECONDS + amie::SETTLE);
    k.poll(amie::AMIE);
    assert_eq!(k.found.borrow().outline(), Some((100, 60, 219, 179)));
    let x = k.found.borrow().stylus.0;
    assert!(k.part().as_deref() == Some("head") && (100..220).contains(&x));
}

#[test]
fn test_amie_closing_leaves_the_still_picture_behind() {
    let mut k = keyed(&[(8, HEAD)]);
    k.poll(FIELD);
    assert_eq!(k.found.borrow().outline(), None);
    drawn(&k.memory, &[(9, BODY)]);
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(k.found.borrow().outline(), Some((100, 120, 219, 229)));
}

/// Wailord stays on the top screen, a strip of belly on the bottom one, until a press held at
/// the top edge brings it down, and one at the bottom edge sends it back (spike A's `watch`).
/// Once it has moved the still picture is taken anew, and that is said.
#[test]
fn test_a_pokemon_too_big_for_the_bottom_screen_is_offered_to_bring_down_and_send_back() {
    let mut k = keyed(&[(9, (40, 0, 160, 6))]);
    let view = k.found.borrow_mut().targets(Some(amie::AMIE)).unwrap()[2].clone();
    assert_eq!(
        (view.name.as_str(), view.path.clone(), view.seconds),
        (
            "bring Sceptile down",
            vec![amie::VIEW_TOP],
            amie::VIEW_SECONDS
        )
    );
    let after = view.after.0.expect("told after");
    after();
    drawn(
        &k.memory,
        &[
            (9, (0, 0, 320, 240)),
            (8, (60, 20, 260, 120)),
            (4, (90, 50, 110, 70)),
        ],
    );
    assert!(k.poll(amie::AMIE).is_empty());
    k.now.set(k.now.get() + amie::VIEW_SECONDS + amie::SETTLE);
    let said: Vec<String> = k
        .poll(amie::AMIE)
        .into_iter()
        .map(|(_log, line, _category)| line)
        .collect();
    assert_eq!(said, ["Sceptile came down"]);
    assert!(k.poll(amie::AMIE).is_empty());
    let view = k.found.borrow_mut().targets(Some(amie::AMIE)).unwrap()[2].clone();
    assert_eq!(
        (view.name.as_str(), view.path),
        ("send Sceptile back up", vec![amie::VIEW_BOTTOM])
    );
}

/// Up close, Butterfree reaches the top of the screen, and shows its eyes (the census).
#[test]
fn test_a_pokemon_the_screen_shows_whole_is_offered_no_way_between_the_screens() {
    let targets = |k: Keyed| {
        k.found
            .borrow_mut()
            .targets(Some(amie::AMIE))
            .unwrap()
            .len()
    };
    assert_eq!(targets(keyed(&[(9, BODY), (8, HEAD)])), 2);
    assert_eq!(
        targets(keyed(&[(8, (100, 0, 220, 100)), (4, (120, 20, 130, 30))])),
        2
    );
}

const TASK: i64 = 0x08A11000;
const SCRIPT: i64 = 0x08A12000;

/// Amie's script as it runs, holding the Pokémon's numbers.
fn numbers(memory: &FakeMemory, affection: i64, fullness: i64, enjoyment: i64) {
    memory.u32(PLAYER + amie::TASK, &[TASK]);
    memory.u32(TASK + amie::TASK_SCRIPT, &[SCRIPT]);
    memory.write(
        SCRIPT + amie::SCRIPT_MAGIC_AT,
        &(amie::SCRIPT_MAGIC as u16).to_le_bytes(),
    );
    memory.u32(SCRIPT + amie::SCRIPT_DATA, &[0x400]);
    for (at, number) in amie::NUMBERS.iter().zip([affection, fullness, enjoyment]) {
        memory.u32(SCRIPT + 0x400 + at, &[number]);
    }
}

/// Measured with the record rewritten and the screen read, alike for all three (spike A).
#[test]
fn test_each_number_fills_its_row_as_the_switch_screen_does() {
    assert_eq!(
        [0, 1, 49, 50, 99, 100, 149, 150, 200, 254, 255].map(amie::meter),
        [0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 5]
    );
}

/// Orion, 2026-11-06: the game shows them only on its Switch screen, so a key says them in
/// Amie itself, by the game's own words for them.
#[test]
fn test_a_key_says_the_numbers_as_the_switch_screen_would_show_them() {
    let mut k = keyed(&[(8, HEAD)]);
    let mut keys = amie::StylusKeys::new(
        k.found.clone(),
        Rc::new(|line: &str| format!("! {line}")),
        ["Affection", "Fullness", "Enjoyment"]
            .map(String::from)
            .to_vec(),
    );
    let mut press =
        |action: &str, k: &mut Keyed| keys.press(action, Some(amie::AMIE), &mut k.chosen).unwrap();
    assert_eq!(press(STYLUS_NUMBERS, &mut k), "! edge"); // no script running
    numbers(&k.memory, 60, 0, 255);
    assert_eq!(
        press(STYLUS_NUMBERS, &mut k),
        "Affection 2 of 5, Fullness 0 of 5, Enjoyment 5 of 5"
    );
}

/// or_amie_interactions: a pet took 50 from Enjoyment and gave 3 hearts; Pikachu's ears took
/// 20 and gave none, displeased; a high five added 10; a puff gave Affection 2 and took nothing,
/// and "finished the puff" says that.
#[test]
fn test_what_a_pet_or_a_high_five_shows_is_said_as_the_numbers_move() {
    let k = keyed(&[(8, HEAD)]);
    let mut moods = amie::Moods::new(k.found.clone());
    let mut held = Held(None);
    let mut said = |affection: i64, enjoyment: i64, love: Option<i64>| -> Vec<String> {
        numbers(&k.memory, affection, 0, enjoyment);
        mood(&k.memory, love);
        k.now.set(k.now.get() + 0.2);
        moods
            .poll(Some(amie::AMIE), &mut held)
            .unwrap()
            .into_iter()
            .map(|(_log, line, _category)| line)
            .collect()
    };
    assert!(said(60, 255, Some(0)).is_empty());
    assert_eq!(said(63, 205, Some(0)), ["3 hearts"]);
    assert_eq!(said(64, 185, Some(0)), ["1 heart"]);
    assert_eq!(said(64, 165, Some(0)), ["Sceptile didn't like that..."]);
    // a puff, whose dance shows no hearts
    assert_eq!(said(66, 165, Some(0)), ["Sceptile dances happily"]);
    assert!(said(66, 175, Some(0)).is_empty()); // Enjoyment rising by itself, no hand up
    said(66, 175, Some(amie::HAND_RAISED));
    assert_eq!(said(66, 185, Some(0)), ["High five! A burst of pink dots"]);
    k.now.set(k.now.get() + amie::HIGH_FIVE_SECONDS);
    assert!(said(66, 195, Some(0)).is_empty());
    assert_eq!(said(255, 195, Some(0)), ["Sceptile dances happily"]);
    assert!(said(255, 145, Some(0)).is_empty()); // at full Affection a pet gives no hearts to count
    assert!(moods.poll(Some(FIELD), &mut held).unwrap().is_empty());
    let mut said = |affection: i64, enjoyment: i64| -> Vec<String> {
        numbers(&k.memory, affection, 0, enjoyment);
        mood(&k.memory, Some(0));
        k.now.set(k.now.get() + 0.2);
        moods
            .poll(Some(amie::AMIE), &mut held)
            .unwrap()
            .into_iter()
            .map(|(_log, line, _category)| line)
            .collect()
    };
    assert!(said(60, 255).is_empty()); // Amie opened again, on another Pokémon
}

/// `monkeypatch.setattr(amie_switch, "pictures", ...)`: the Switch screen's rows as
/// or_amie_interactions drew them.
fn drawn_rows(
    _layouts: &Layouts,
    _image_of: &ImageOf,
    _top: bool,
    _wanted: &dyn Fn(&str) -> bool,
) -> Result<Vec<(i64, i64, String, f64, f64)>> {
    let mut rows: Vec<(&str, f64)> = Vec::new();
    rows.extend([("heart_00.bclim", 30.0); 2]);
    rows.extend([("heart_01.bclim", 30.0); 3]);
    rows.push(("heart_00.bclim", 76.0));
    rows.extend([("ic_hungry_black.bclim", 5.0); 5]);
    rows.extend([("ic_asobu.bclim", -20.0); 4]);
    rows.push(("ic_asobu_black.bclim", -20.0));
    Ok(rows
        .into_iter()
        .map(|(image, up)| (0, 0, image.to_string(), 54.0, up))
        .collect())
}

/// Panes of the Switch screen by their places: the root, three of the party's buttons and the
/// chosen Pokémon's picture. Python's are panes 0 to 4 of a fake `Layouts`; here panes of
/// `FakeLayouts` at these addresses, under one root.
const PLACES: [(i64, (f32, f32)); 4] = [
    (0x08A31000, (-125.0, 66.0)),
    (0x08A32000, (-75.0, 70.0)),
    (0x08A33000, (-25.0, 66.0)),
    (0x08A34000, (-87.0, 15.0)),
];

/// They said "button pokémon", the icon they draw saying nothing of whose (Orion, 2026-11-06).
/// Nothing but the Switch screen's own pictures, and only the party's row, has a name here.
#[test]
fn test_the_switch_screens_buttons_are_named_by_the_party_member_their_place_holds() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let vtable = classes.add(pokemon::PARAM_CLASS);
    let party = 0x08A20000;
    for (slot, blob) in [
        record(0x101, 254, "Blaze", true),
        record(0x202, 25, "Pikachu", false),
    ]
    .into_iter()
    .enumerate()
    {
        let param = PARAM + 0x400 * slot as i64;
        memory.u32(param, &[vtable]);
        memory.u32(
            param + pokemon::PARAM_RECORD,
            &[param + pokemon::RECORD_IN_PARAM],
        );
        memory.write(param + pokemon::RECORD_IN_PARAM, &blob);
        memory.u32(party + 4 * slot as i64, &[param]);
    }
    memory.write(party + COUNT, &[2]);
    let fake = FakeLayouts::new(&memory, &classes);
    let (_work, root) = fake.screen_with(0x08A30000, false, false);
    for (pane, (x, y)) in PLACES {
        fake.pane(pane, root);
        memory.write(pane + fake.shape.x, &x.to_le_bytes());
        memory.write(pane + fake.shape.y, &y.to_le_bytes());
    }
    fake.track().unwrap();
    let mut screen = amie_switch::SwitchScreen::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        fake.layouts.clone(),
        Rc::new(|_pane| Ok(None)),
        ["Affection", "Fullness", "Enjoyment"]
            .map(String::from)
            .to_vec(),
        Rc::new(move || Some(party)),
        Rc::new(|found: &crate::games::common::pokemon::Pokemon| {
            Some(if found.nicknamed {
                found.nickname.clone()
            } else {
                "Pikachu".to_string()
            })
        }),
    );
    let [first, second, third, chosen] = PLACES.map(|(pane, _)| pane);
    assert_eq!(screen.name(first, Some(FIELD)).unwrap(), None); // not that screen
    screen
        .reworded(
            ["Affection", "Fullness", "Enjoyment"]
                .iter()
                .enumerate()
                .map(|(r#box, text)| (r#box as i64, ScreenText::new(r#box as i64, *text, false)))
                .collect(),
        )
        .unwrap();
    assert_eq!(
        screen.name(first, Some(FIELD)).unwrap().as_deref(),
        Some("Blaze")
    );
    assert_eq!(
        screen.name(second, Some(FIELD)).unwrap().as_deref(),
        Some("Pikachu")
    );
    assert_eq!(screen.name(third, Some(FIELD)).unwrap(), None); // past the party's end
    // the chosen Pokémon's own picture, lower down
    assert_eq!(screen.name(chosen, Some(FIELD)).unwrap(), None);
    assert_eq!(screen.name(first, Some(amie::AMIE)).unwrap(), None);
}

/// The game's words and how many of the five pictures beside them are filled, the heart over
/// the Pokémon chosen no part of a row (or_amie_interactions).
#[test]
fn test_the_switch_screen_says_each_row_by_its_label() {
    let memory = FakeMemory::new();
    let layouts = FakeLayouts::new(&memory, &FakeClasses::new(memory.clone()))
        .layouts
        .clone();
    let mut screen = amie_switch::SwitchScreen::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        layouts,
        Rc::new(|_pane| Ok(None)),
        ["Affection", "Fullness", "Enjoyment"]
            .map(String::from)
            .to_vec(),
        Rc::new(|| None),
        Rc::new(|_found: &crate::games::common::pokemon::Pokemon| None),
    );
    screen.pictures = drawn_rows;
    let mut words = |texts: &[&str]| -> Vec<String> {
        let found: IndexMap<i64, ScreenText> = texts
            .iter()
            .enumerate()
            .map(|(r#box, text)| (r#box as i64, ScreenText::new(r#box as i64, *text, false)))
            .collect();
        screen
            .reworded(found)
            .unwrap()
            .into_values()
            .map(|piece| piece.text)
            .collect()
    };
    assert_eq!(
        words(&["Pikachu", "Affection", "Fullness", "Enjoyment"]),
        [
            "Pikachu",
            "Affection 2 of 5",
            "Fullness 0 of 5",
            "Enjoyment 4 of 5"
        ]
    );
    assert_eq!(words(&["Affection", "Pikachu"]), ["Affection", "Pikachu"]); // not that screen
    let found: IndexMap<i64, ScreenText> = [
        ("Switch Pokémon!", 104.0),
        ("Pikachu", -10.0),
        ("Affection", 30.0),
        ("Fullness", 5.0),
        ("Enjoyment", -20.0),
    ]
    .iter()
    .enumerate()
    .map(|(r#box, (text, row))| {
        (
            r#box as i64,
            ScreenText {
                row: *row,
                ..ScreenText::new(r#box as i64, *text, false)
            },
        )
    })
    .collect();
    let rows: IndexMap<String, f64> = screen
        .reworded(found)
        .unwrap()
        .into_values()
        .map(|piece| (piece.text, piece.row))
        .collect();
    assert_eq!(
        rows,
        IndexMap::from([
            ("Switch Pokémon!".to_string(), 104.0),
            ("Pikachu".to_string(), 31.0),
            ("Affection 2 of 5".to_string(), 30.0),
            ("Fullness 0 of 5".to_string(), 5.0),
            ("Enjoyment 4 of 5".to_string(), -20.0)
        ])
    ); // the name first
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
