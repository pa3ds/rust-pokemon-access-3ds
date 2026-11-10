//! The Pokémon-Amie page's Pokémon is named by its species, through the icon it is drawn with.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_memories::record;
use crate::core::keyfile::data_path;
use crate::core::rtti::ClassNames;
use crate::core::speech::Category;
use crate::core::words::{self, ENGLISH, Words};
use crate::games::common::field::keys::STYLUS;
use crate::games::common::pictures::named;
use crate::games::common::touch::Target;
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::screens::distribution as d;
use crate::games::gen6::subscreen::amie;
use crate::games::gen6::subscreen::berry_picker::Touching;
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
    page_aimed(number, false, false, 0)
}

/// `page(number, aimed, still_picture, later)`.
fn page_aimed(
    number: i64,
    aimed: bool,
    still_picture: bool,
    later: i64,
) -> (amie::AmiePage, Rc<FakeMemory>) {
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
        still_picture,
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

/// Amie running, its picture of parts as `drawn` draws it: the page and the memory.
fn running(parts: &[(u8, (i64, i64, i64, i64))], aimed: bool) -> (amie::AmiePage, Rc<FakeMemory>) {
    running_with(parts, aimed, false, 0)
}

/// `running(parts, aimed, still_picture, later)`.
fn running_with(
    parts: &[(u8, (i64, i64, i64, i64))],
    aimed: bool,
    still_picture: bool,
    later: i64,
) -> (amie::AmiePage, Rc<FakeMemory>) {
    let (mut found, memory) = page_aimed(amie::NUMBER, aimed, still_picture, later);
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

/// Amie counts a pet only while the hand moves 4 pixels a tick on one kind of part (spike A):
/// a rub stays on the part, `NEAR` at most either side of the stylus, and 12 pixels at least.
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
        found.targets(Some(amie::AMIE)).unwrap()[1].rub
    };
    assert_eq!(rub(150, 90), Some(((138, 90), (162, 90)))); // the head: across, wider than tall
    assert_eq!(rub(183, 120), Some(((183, 108), (183, 132)))); // a cheek's strip: down it
    assert_eq!(rub(160, 65), Some(((148, 65), (172, 65)))); // a sweet spot's strip: across it
    assert_eq!(rub(104, 62), Some(((97, 62), (109, 62)))); // an ear tip, 8 across: a little past it
    // Off the Pokémon it holds still: a rub reaching back onto it never raised its hand.
    assert_eq!(rub(20, 20), Some(((20, 20), (20, 20))));
    // Where the picture cannot be read, before Azahar 2126.2, it rubs where it is all the same.
    assert_eq!(
        amie_running(&[(8, HEAD)], false)
            .targets(Some(amie::AMIE))
            .unwrap()[1]
            .rub,
        Some(((154, 120), (166, 120)))
    );
}

/// Orion, 2026-11-05: Pikachu moved, and the stylus left its chin. Kept on its part by the
/// middle of its row, a step off the head was the head again as Pikachu bounced (2026-11-06):
/// "if the pokemon moved 3 pixels, the stylus moves 3 pixels in the same direction".
#[test]
fn test_the_stylus_moves_with_its_part_as_far_as_the_part_moves() {
    let now = Rc::new(Cell::new(100.0));
    let _thawed = frozen(&now);
    let (mut found, memory) = running(&[(9, BODY), (8, HEAD), (17, (180, 100, 190, 110))], true);
    // The head and body moved by (dx, dy), and the cheek where given.
    let shifted = |dx: i64, dy: i64, cheek: (i64, i64, i64, i64)| {
        drawn(
            &memory,
            &[
                (9, (100 + dx, 120 + dy, 220 + dx, 230 + dy)),
                (8, (100 + dx, 60 + dy, 220 + dx, 120 + dy)),
                (17, cheek),
            ],
        );
        now.set(now.get() + 1.0);
    };
    found.stylus = (185, 105);
    found.keep(None).unwrap();
    assert!(!found.follow().unwrap());
    shifted(3, -2, (183, 98, 193, 108)); // all of it 3 right and 2 up
    assert!(found.follow().unwrap() && found.stylus == (188, 103));
    assert!(!found.follow().unwrap());
    shifted(3, -2, (173, 102, 183, 112)); // the cheek alone 10 left and 4 down
    assert!(found.follow().unwrap() && found.stylus == (178, 107));
    shifted(3, -2, (100, 200, 110, 210)); // the cheek gone far off
    assert!(!found.follow().unwrap() && found.stylus == (178, 107));
    found.stylus = (20, 20);
    found.keep(None).unwrap(); // off the Pokémon, and far from it: kept on nothing, and where it is
    shifted(0, 0, (180, 100, 190, 110));
    assert!(!found.follow().unwrap());
    shifted(5, 0, (185, 100, 195, 110));
    assert!(!found.follow().unwrap() && found.stylus == (20, 20));
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
}

fn keyed(parts: &[(u8, (i64, i64, i64, i64))]) -> Keyed {
    keyed_still(parts, false)
}

/// `keyed(parts, still_picture)`.
fn keyed_still(parts: &[(u8, (i64, i64, i64, i64))], still_picture: bool) -> Keyed {
    let now = Rc::new(Cell::new(100.0));
    let thawed = frozen(&now);
    let (found, memory) = running_with(parts, true, still_picture, 0);
    let found = found.shared();
    let chosen = Chosen {
        page: found.clone(),
        chosen: Vec::new(),
        carried: Vec::new(),
    };
    let keys = amie::StylusKeys::new(found.clone(), Rc::new(|line: &str| format!("! {line}")));
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
/// mode chooses it as it goes, to rub there; a poll keeps it on its part, choosing nothing in
/// touch mode; and the arrows keep it within a step of the Pokémon.
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
    drawn(&k.memory, &[(8, (180, 60, 300, 120))]); // the head moved off the stylus, 80 right
    k.now.set(k.now.get() + 1.0);
    assert!(k.poll(FIELD).is_empty() && k.chosen.carried.len() == 1); // Amie not running
    assert!(k.poll(amie::AMIE).is_empty() && k.chosen.carried.last() == Some(&(189, 108)));
    assert_eq!(k.chosen.chosen.len(), 4); // the poll chose nothing
    k.found.borrow_mut().stylus = (180, 60); // the head's top left corner
    assert_eq!(k.press("stylus left"), "off Sceptile");
    assert_eq!(k.press("stylus left"), "! edge"); // a step off the Pokémon at most
    assert_eq!(k.press("stylus up"), "");
    assert!(k.press("stylus up") == "! edge" && k.found.borrow().stylus == (174, 54));
}

/// Orion could not find a hand or fur a step at a time (2026-11-05): a jump goes past the
/// part the stylus is on and any gap, to the middle of the next part's run, and says it.
#[test]
fn test_ctrl_and_an_arrow_jump_the_stylus_to_the_next_part_that_way() {
    let mut k = keyed(&[
        (9, BODY),
        (8, HEAD),
        (17, (180, 100, 186, 140)),
        (27, (140, 60, 180, 70)),
    ]);
    k.found.borrow_mut().stylus = (120, 105);
    assert_eq!(k.press("stylus jump right"), "sparks");
    assert_eq!(k.found.borrow().stylus, (182, 105));
    assert_eq!(k.press("stylus jump right"), "head");
    assert_eq!(k.found.borrow().stylus, (202, 105));
    k.found.borrow_mut().stylus = (160, 100);
    assert!(k.press("stylus jump up") == "head" && k.found.borrow().stylus == (160, 64));
    k.found.borrow_mut().stylus = (110, 90);
    assert!(k.press("stylus jump left") == "! edge" && k.found.borrow().stylus == (110, 90));
}

const COMMAND: i64 = 0x08A0F000;
const PARAM: i64 = 0x08A10000;

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
    let mut moods = amie::Moods::new(Rc::new(RefCell::new(found)));
    let mut said = |numbers: &[Option<i64>]| -> Vec<String> {
        let mut lines = Vec::new();
        for &number in numbers {
            mood(&memory, number);
            lines.extend(
                moods
                    .poll(Some(amie::AMIE))
                    .unwrap()
                    .into_iter()
                    .map(|(_log, line, _category)| line),
            );
        }
        lines
    };
    assert_eq!(said(&[Some(7), Some(7)]), ["Sceptile has its back turned"]);
    assert_eq!(said(&[Some(0)]), ["Sceptile turned round"]);
    assert!(said(&[Some(13), None, Some(0)]).is_empty()); // a pet is the player's own doing
    assert_eq!(
        said(&[Some(22), Some(24), None, Some(24)]),
        [
            "Sceptile is watching your hand",
            "Sceptile is holding up its hand",
            "Sceptile is holding up its hand"
        ]
    );
    assert_eq!(said(&[Some(23)]), ["Sceptile is looking around"]);
    assert_eq!(
        said(&[15, 17, 18, 19, 15, 17, 18, 19, 0].map(Some)),
        [
            "Sceptile is eyeing the puff",
            "Sceptile is eating",
            "Sceptile is eating",
            "Sceptile finished the puff"
        ]
    );
    assert_eq!(said(&[Some(22)]), ["Sceptile is watching your hand"]);
    assert!(moods.poll(Some(FIELD)).unwrap().is_empty());
    let mut said = |number: i64| -> Vec<String> {
        mood(&memory, Some(number));
        moods
            .poll(Some(amie::AMIE))
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
    let (mut found, memory) = running_with(&[(8, HEAD)], true, false, later);
    memory.u32(PLAYER + amie::LOVE + later, &[COMMAND]);
    memory.write(COMMAND + amie::LOVE_NUMBER, &23u16.to_le_bytes());
    assert_eq!(found.mood().unwrap(), Some(23));
    assert_eq!(found.outline().unwrap(), Some((100, 60, 219, 119)));
}

/// Orion, 2026-11-06: "describe the other moods too, even if nothing reaches them yet"; seen
/// started at will on the research rig (spike A's `look`).
#[test]
fn test_the_moods_no_movie_reached_are_said_too() {
    let (found, memory) = running(&[(8, HEAD)], false);
    let mut moods = amie::Moods::new(Rc::new(RefCell::new(found)));
    let mut said = |number: i64| -> Vec<String> {
        mood(&memory, Some(number));
        moods
            .poll(Some(amie::AMIE))
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
    assert_eq!(said(16), ["Sceptile is reaching for the puff"]);
    assert_eq!(said(20), ["the puff fell"]);
    assert_eq!(said(21), ["Sceptile frowns and turns away"]);
    assert_eq!(said(27), ["Sceptile leans in to your face"]);
}

/// ADR-0009: the hand is up about 2 s and a player cannot see where, so a jump goes straight
/// to it, in the picture as it is now; before it, a jump is a jump, and so it is for a Pokémon
/// with no hand drawn.
#[test]
fn test_while_its_hand_is_raised_a_jump_goes_to_the_hand_whichever_way() {
    let mut k = keyed(&[(9, BODY), (8, HEAD), (10, (120, 140, 130, 150))]);
    k.found.borrow_mut().stylus = (40, 100);
    assert_eq!(k.press("stylus jump right"), "head");
    k.found.borrow_mut().stylus = (40, 100);
    mood(&k.memory, Some(amie::HAND_RAISED));
    assert_eq!(k.press("stylus jump left"), "hand");
    assert!(k.found.borrow().stylus == (124, 140) && k.chosen.chosen.last() == Some(&(124, 140)));
    drawn(&k.memory, &[(9, BODY), (8, HEAD), (10, (150, 70, 160, 80))]); // it rose, within WHOLE
    assert!(k.press("stylus jump up") == "hand" && k.found.borrow().stylus == (154, 79));
    drawn(&k.memory, &[(9, BODY), (8, HEAD)]);
    k.now.set(k.now.get() + 1.0);
    k.found.borrow_mut().stylus = (120, 105);
    assert_eq!(k.press("stylus jump up"), "! edge");
}

/// Amie running with the stylus over a still picture, taken of `parts`.
fn still(parts: &[(u8, (i64, i64, i64, i64))]) -> Keyed {
    let mut k = keyed_still(parts, true);
    k.poll(amie::AMIE);
    k
}

/// Where a touch on the stylus rubs now.
fn rubbed(found: &Rc<RefCell<amie::AmiePage>>) -> ((i64, i64), (i64, i64)) {
    found.borrow_mut().targets(Some(amie::AMIE)).unwrap()[1]
        .rub
        .expect("a rub")
}

/// Orion, 2026-11-06: the stylus moves over a picture taken as the Pokémon appears, however it
/// moves, and a tap or drag goes to the part chosen wherever it has gone. Touch mode's tone stays
/// where the player put it.
#[test]
fn test_over_a_still_picture_a_part_stays_where_it_was_found_and_a_touch_finds_it() {
    let mut k = still(&[(9, BODY), (8, HEAD)]);
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
    let ((left, y), (right, same_y)) = rubbed(&k.found);
    assert!(y == same_y && (120..180).contains(&y) && 160 <= left && left < right && right < 280);
}

/// A part the picture no longer draws, an eye shut, is rubbed where it would be on the
/// Pokémon: as far across and down it as it was.
#[test]
fn test_a_touch_on_a_part_gone_from_view_goes_where_it_would_be() {
    let mut k = still(&[(9, BODY), (8, HEAD), (4, (150, 80, 170, 100))]);
    k.found.borrow_mut().stylus = (160, 90);
    k.press("stylus left");
    drawn(
        &k.memory,
        &[(9, (160, 120, 280, 230)), (8, (160, 60, 280, 120))],
    );
    k.now.set(k.now.get() + 1.0);
    let ((left, y), (right, _y)) = rubbed(&k.found);
    assert!((195..=235).contains(&py::floordiv(left + right, 2)) && (70..=100).contains(&y));
}

/// A press held off the Pokémon raises its hand, and one touching it never does: the still
/// picture's "off" may be on the Pokémon now (spike A).
#[test]
fn test_a_press_off_the_pokemon_keeps_clear_of_it_wherever_it_has_gone() {
    let mut k = still(&[(9, BODY), (8, HEAD)]);
    k.found.borrow_mut().stylus = (98, 100);
    k.press("stylus left"); // off it, by the still picture
    assert_eq!(rubbed(&k.found), ((82, 100), (82, 100))); // a step off it is not clear
    drawn(
        &k.memory,
        &[(9, (60, 120, 180, 230)), (8, (60, 60, 180, 120))],
    ); // now under the stylus
    k.now.set(k.now.get() + 1.0);
    assert_eq!(rubbed(&k.found), ((42, 100), (42, 100)));
}

/// Its first frames may catch the Pokémon looking away; after `RICHER` it stays, and it is
/// taken again only once the Pokémon comes anew.
#[test]
fn test_the_still_picture_takes_in_more_parts_at_first_and_then_stays() {
    let mut k = still(&[(8, HEAD)]);
    k.found.borrow_mut().stylus = (160, 90);
    let part = |k: &Keyed| k.found.borrow_mut().part(None).unwrap();
    assert_eq!(part(&k).as_deref(), Some("head"));
    drawn(&k.memory, &[(8, HEAD), (4, (150, 80, 170, 100))]);
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(part(&k).as_deref(), Some("right eye"));
    drawn(
        &k.memory,
        &[(8, HEAD), (4, (150, 80, 170, 100)), (6, (155, 85, 165, 95))],
    );
    k.now.set(k.now.get() + amie::RICHER + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(part(&k).as_deref(), Some("right eye")); // settled
    mood(&k.memory, Some(7));
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(part(&k), None); // back turned, no picture
    mood(&k.memory, Some(0));
    drawn(&k.memory, &[(8, HEAD)]);
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(part(&k).as_deref(), Some("head"));
}

/// It comes closer to a beckon, and a far picture has parts narrower than a step. The stylus
/// stays on its part.
#[test]
fn test_a_beckon_takes_the_still_picture_again_once_the_pokemon_has_come_close() {
    let mut k = still(&[(8, (150, 100, 170, 120))]);
    k.found.borrow_mut().stylus = (160, 110);
    k.press("stylus up");
    let beckon = k.found.borrow_mut().targets(Some(amie::AMIE)).unwrap()[0].clone();
    let after = beckon.after.0.expect("told after");
    after();
    drawn(&k.memory, &[(8, (100, 60, 220, 180))]);
    k.now.set(k.now.get() + 0.5);
    k.poll(amie::AMIE);
    assert_eq!(
        k.found.borrow_mut().outline().unwrap(),
        Some((150, 100, 169, 119))
    ); // not stopped yet
    k.now.set(k.now.get() + amie::BECKON_SECONDS + amie::SETTLE);
    k.poll(amie::AMIE);
    assert_eq!(
        k.found.borrow_mut().outline().unwrap(),
        Some((100, 60, 219, 179))
    );
    let part = k.found.borrow_mut().part(None).unwrap();
    let x = k.found.borrow().stylus.0;
    assert!(part.as_deref() == Some("head") && (100..220).contains(&x));
}

#[test]
fn test_amie_closing_leaves_the_still_picture_behind() {
    let mut k = still(&[(8, HEAD)]);
    k.poll(FIELD);
    assert_eq!(k.found.borrow_mut().outline().unwrap(), None);
    drawn(&k.memory, &[(9, BODY)]);
    k.now.set(k.now.get() + 1.0);
    k.poll(amie::AMIE);
    assert_eq!(
        k.found.borrow_mut().outline().unwrap(),
        Some((100, 120, 219, 229))
    );
}

/// Wailord stays on the top screen, a strip of belly on the bottom one, until a press held at
/// the top edge brings it down, and one at the bottom edge sends it back (spike A's `watch`).
/// Once it has moved the still picture is taken anew, and that is said.
#[test]
fn test_a_pokemon_too_big_for_the_bottom_screen_is_offered_to_bring_down_and_send_back() {
    let mut k = still(&[(9, (40, 0, 160, 6))]);
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
    assert_eq!(targets(still(&[(9, BODY), (8, HEAD)])), 2);
    assert_eq!(
        targets(still(&[(8, (100, 0, 220, 100)), (4, (120, 20, 130, 30))])),
        2
    );
}

/// The stylus goes to the hand in the still picture, and the touch to the hand as it waves.
#[test]
fn test_while_its_hand_is_raised_a_jump_over_a_still_picture_touches_the_hand_as_it_is_now() {
    let mut k = still(&[(9, BODY), (8, HEAD), (10, (120, 140, 130, 150))]);
    k.found.borrow_mut().stylus = (40, 100);
    mood(&k.memory, Some(amie::HAND_RAISED));
    drawn(&k.memory, &[(9, BODY), (8, HEAD), (10, (180, 60, 200, 80))]);
    k.now.set(k.now.get() + 1.0);
    assert_eq!(k.press("stylus jump left"), "hand");
    let (x, y) = k.found.borrow().stylus;
    assert!((120..130).contains(&x) && (140..150).contains(&y));
    let ((left, y), (right, _y)) = rubbed(&k.found);
    assert!((180..200).contains(&py::floordiv(left + right, 2)) && (60..80).contains(&y));
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
