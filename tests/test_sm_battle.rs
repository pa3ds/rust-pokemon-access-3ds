//! Sun's battle HP reader, against what sun_9.ctm showed (spike 124): Hau's Popplio against
//! Rowlet, the HP box one layout with a panel a side, names and numbers as TextBoxes, and a
//! `btl::BtlvGaugeBase` a panel.

use std::rc::Rc;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_battle::record_of;
use crate::core::rtti::ClassNames;
use crate::games::common::battle::{GAUGE_RECORD, MAX_ASKS, RECORD_PARAM};
use crate::games::common::screen::ScreenText;
use crate::games::common::{lyt, pokemon};
use crate::games::sm::battle as hp;
use crate::games::sm::battle::SunBattleReader;
use crate::games::sm::lyt::SHAPE;

const BOX: i64 = 0x30330000;
const MINE: i64 = BOX + 0x1000;
const THEIRS: i64 = BOX + 0x2000;
const GAUGES: i64 = 0x30680000;
const RECORDS: i64 = 0x30002000;
const PARAMS: i64 = 0x3002E000;
/// The TextBoxes of a panel, as `screen` numbers them.
const NAME: i64 = 1;
const LEVEL: i64 = 2;
const CURRENT: i64 = 3;
const SLASH: i64 = 4;
const MAXIMUM: i64 = 5;
/// The maximum HP as the box draws it, in fullwidth digits.
const FULLWIDTH_22: &str = "\u{ff12}\u{ff12}";

struct Battle {
    memory: Rc<FakeMemory>,
    gauge_class: i64,
    param_class: i64,
    fake: FakeLayouts,
    root: i64,
    reader: SunBattleReader,
    mine: i64,
    theirs: i64,
}

impl Battle {
    fn new() -> Battle {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let gauge_class = classes.add(hp::GAUGE_CLASS);
        let param_class = classes.add(pokemon::PARAM_CLASS);
        let fake = FakeLayouts::with_shape(&memory, &classes, SHAPE);
        let (work, root) = fake.screen_with(BOX, true, false);
        fake.project(work, 400.0);
        for panel in [MINE, THEIRS] {
            fake.pane(panel, root);
        }
        fake.track().unwrap();
        let names = Rc::new(ClassNames::new(memory.clone(), CODE));
        let reader = SunBattleReader::new(memory.clone(), names, fake.layouts.clone());
        let mut battle = Battle {
            memory,
            gauge_class,
            param_class,
            fake,
            root,
            reader,
            mine: 0,
            theirs: 0,
        };
        let rowlet = battle.pokemon(0, "Rowlet", pokemon::FEMALE, 22);
        battle.mine = battle.gauge(0, MINE, rowlet, 22);
        let popplio = battle.pokemon(1, "Popplio", pokemon::MALE, 20);
        battle.theirs = battle.gauge(1, THEIRS, popplio, 20);
        battle
            .reader
            .track([battle.mine, battle.theirs], false)
            .unwrap();
        battle
    }

    /// A battle record holding a PokemonParam and its maximum HP. Returns the record.
    fn pokemon(&self, index: i64, nickname: &str, gender: i64, maximum: i64) -> i64 {
        let (param, battle_record) = (PARAMS + index * 0x200, RECORDS + index * 0x320);
        self.memory
            .u32(param, &[self.param_class, param + pokemon::RECORD_IN_PARAM]);
        self.memory.write(
            param + pokemon::RECORD_IN_PARAM,
            &record_of(nickname, gender, 722),
        );
        self.memory.u32(battle_record + RECORD_PARAM, &[param]);
        self.memory.write(
            battle_record + hp::RECORD_MAX_HP,
            &(maximum as u16).to_le_bytes(),
        );
        battle_record
    }

    fn gauge(&self, index: i64, panel: i64, battle_record: i64, shown: i64) -> i64 {
        let gauge = GAUGES + index * 0x100;
        self.memory.u32(gauge, &[self.gauge_class]);
        self.memory.u32(gauge + GAUGE_RECORD, &[battle_record]);
        self.memory.u32(gauge + hp::GAUGE_PANEL, &[panel, index]);
        self.counting(gauge, shown, shown);
        gauge
    }

    fn counting(&self, gauge: i64, shown: i64, target: i64) {
        self.memory
            .u32(gauge + hp::GAUGE_SHOWN_HP, &[shown, target]);
    }

    fn heard(&mut self, pieces: &[ScreenText]) -> Vec<String> {
        self.fake.layouts.refresh();
        self.reader
            .poll(pieces)
            .unwrap()
            .into_iter()
            .map(|side| side.text)
            .collect()
    }
}

fn piece(r#box: i64, text: &str, panel: i64, row: f64, left: f64, root: i64) -> ScreenText {
    ScreenText {
        left,
        right: left + 20.0,
        row,
        glyph: 12.0,
        top: true,
        root,
        panel,
        parent: panel,
        ..ScreenText::new(r#box, text, false)
    }
}

/// `screen(battle)`: the player's 22 of 22 at level 5 and the enemy's level 5.
fn screen(battle: &Battle) -> Vec<ScreenText> {
    screen_with(battle, "22", FULLWIDTH_22, "5", "5")
}

/// The box as sun_9 drew it: the name and level on one row, the HP on another with the
/// layout file's "/", the maximum in fullwidth digits; the enemy's panel with the name and level
/// alone.
fn screen_with(
    battle: &Battle,
    mine: &str,
    maximum: &str,
    my_level: &str,
    their_level: &str,
) -> Vec<ScreenText> {
    let root = battle.root;
    vec![
        piece(NAME, "Rowlet", MINE, -85.0, -157.0, root),
        piece(LEVEL, my_level, MINE, -86.0, -93.0, root),
        piece(CURRENT, mine, MINE, -105.0, -168.0, root),
        piece(SLASH, "/", MINE, -105.0, -164.0, root),
        piece(MAXIMUM, maximum, MINE, -105.0, -160.0, root),
        piece(10 + NAME, "Popplio", THEIRS, 108.0, 106.0, root),
        piece(10 + LEVEL, their_level, THEIRS, 107.0, 170.0, root),
    ]
}

fn settle(battle: &mut Battle, pieces: &[ScreenText]) -> Vec<String> {
    battle.heard(pieces);
    battle.heard(pieces)
}

fn strings(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|text| text.to_string()).collect()
}

/// The player's numbers, the enemy's share of its HP, and the gender from the record, since
/// the game draws it as a picture.
#[test]
fn test_each_side_is_said_from_its_panel_with_the_sign_of_the_pokemon_its_gauge_shows() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Rowlet♀ L5 22/22", "Popplio♂ L5 100%"])
    );
}

/// Leafage took Popplio from 20 HP to 12 a point or two a poll (spike 124). Its fill drew
/// 65.2 of 104.8, which is 62%: the fill keeps a sliver at 0 HP, so the HP is what is said.
#[test]
fn test_the_enemy_is_said_as_a_percentage_once_its_gauge_has_counted_down() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    settle(&mut battle, &pieces);
    battle.counting(battle.theirs, 16, 12);
    assert_eq!(settle(&mut battle, &pieces), strings(&[]));
    battle.counting(battle.theirs, 12, 12);
    assert_eq!(settle(&mut battle, &pieces), strings(&["Popplio♂ L5 60%"]));
    battle.counting(battle.theirs, 0, 0);
    assert_eq!(settle(&mut battle, &pieces), strings(&["Popplio♂ L5 0%"]));
}

/// The whole gauge hides through each attack and comes back; a side is said again only when
/// its reading has changed.
#[test]
fn test_a_side_is_not_said_again_when_its_gauge_comes_back_from_an_attacks_animation() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    settle(&mut battle, &pieces);
    assert_eq!(battle.heard(&[]), strings(&[]));
    assert_eq!(settle(&mut battle, &pieces), strings(&[]));
    battle.reader.track([], false).unwrap();
    battle.heard(&[]);
    battle
        .reader
        .track([battle.mine, battle.theirs], false)
        .unwrap();
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Rowlet♀ L5 22/22", "Popplio♂ L5 100%"])
    );
}

#[test]
fn test_the_players_hp_is_said_once_the_count_has_held_for_a_poll() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    settle(&mut battle, &pieces);
    battle.counting(battle.mine, 18, 18);
    let hurt = screen_with(&battle, "18", FULLWIDTH_22, "5", "5");
    assert_eq!(battle.heard(&hurt), strings(&[]));
    assert_eq!(battle.heard(&hurt), strings(&["Rowlet♀ L5 18/22"]));
}

/// Screen text said "Rowlet 5 22 22" and each step of the count.
#[test]
fn test_the_box_is_the_battle_readers_while_a_gauge_shows_its_panel() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    assert!(!battle.reader.owns(&pieces[0]));
    battle.heard(&pieces);
    assert!(pieces.iter().all(|p| battle.reader.owns(p)));
    let other = piece(20, "HP", BOX + 0x3000, 50.0, 0.0, battle.root);
    assert!(!battle.reader.owns(&other));
}

#[test]
fn test_a_sweep_is_asked_for_while_the_players_panel_has_no_gauge() {
    let mut battle = Battle::new();
    battle.reader.track([], false).unwrap();
    let pieces = screen(&battle);
    assert_eq!(battle.heard(&pieces), strings(&[]));
    assert!(battle.reader.sweep_wanted());
    assert!(!battle.reader.sweep_wanted());
    // Asked again every REASK polls, MAX_ASKS times, then no more: a gauge no sweep can find must
    // not cost a whole walk a second for the rest of the battle.
    let asks: i64 = (0..100)
        .map(|_| i64::from(battle.reader.sweep_wanted()))
        .sum();
    assert_eq!(asks, MAX_ASKS - 1);
    battle
        .reader
        .track([battle.mine, battle.theirs], false)
        .unwrap();
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Rowlet♀ L5 22/22", "Popplio♂ L5 100%"])
    );
    assert!(!battle.reader.sweep_wanted());
}

/// Through the intro the enemy's panel read as shown, its gauge full, while the screen showed
/// no box: its name carried `UNPLACED` until the box came on after Rowlet was sent out.
#[test]
fn test_a_side_is_not_said_before_its_box_is_first_drawn() {
    let mut battle = Battle::new();
    let name = 10 + NAME;
    battle
        .memory
        .write(name + SHAPE.flags, &[(hp::UNPLACED | lyt::VISIBLE) as u8]);
    let pieces = screen(&battle);
    assert_eq!(settle(&mut battle, &pieces), strings(&["Rowlet♀ L5 22/22"]));
    battle
        .memory
        .write(name + SHAPE.flags, &[lyt::VISIBLE as u8]);
    assert_eq!(settle(&mut battle, &pieces), strings(&["Popplio♂ L5 100%"]));
}
