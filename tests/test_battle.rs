//! The battle HP reader, against what the movies actually showed.
//!
//! Numbers here are spike 54's, 57's and 66's: the HP box's panels as battles.ctm drew them, the
//! name buffers as `gfl::str::StrBuf` holds them with the gender sign inside the string, and the
//! gauge that says which Pokémon each panel shows, as battles2.ctm built them.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{Counting, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::battle::{GAUGE_RECORD, MAX_ASKS, REASK, RECORD_PARAM};
use crate::games::common::screen::ScreenText;
use crate::games::common::{lyt, pokemon, strbuf};
use crate::games::gen6::screens::battle as hp;
use crate::games::gen6::screens::battle::{BattleReader, Battlers, Gauges};

const CODE: (i64, i64) = (0x00100000, 0x01000000);
/// The HP box's layout root, as battles.ctm built it.
const BOX: i64 = 0x14D48708;
/// Its two panels: the player's and the enemy's (spike 54).
const MINE: i64 = 0x14D4B614;
const THEIRS: i64 = 0x14D4DCD4;
/// The first battler's StrBuf; the others follow it (spike 57).
const BUFFERS: i64 = 0x08554010;
/// The TextBoxes `screen` puts in the panels, which the gauges hold.
const MY_LEVEL: i64 = 1;
const MY_HP: i64 = 2;
const THEIR_LEVEL: i64 = 3;
const GAUGES: i64 = 0x08554654;
const RECORDS: i64 = 0x082041F4;
const PARAMS: i64 = 0x081FD740;

/// `text.encode("utf-16-le")`.
fn utf16le(text: &str) -> Vec<u8> {
    text.encode_utf16().flat_map(u16::to_le_bytes).collect()
}

/// A Pokémon record as the game stores it, with a constant that leaves its blocks in order.
/// Python's default `species` is 255: this is [`record_of`] with it.
pub fn record(nickname: &str, gender: i64) -> Vec<u8> {
    record_of(nickname, gender, 255)
}

/// `record(nickname, gender, species)`.
pub fn record_of(nickname: &str, gender: i64, species: i64) -> Vec<u8> {
    let mut plain = vec![0u8; pokemon::RECORD_SIZE as usize];
    let constant: i64 = 0x00000101;
    plain[0..4].copy_from_slice(&(constant as u32).to_le_bytes());
    plain[8..10].copy_from_slice(&(species as u16).to_le_bytes());
    plain[pokemon::GENDER as usize] = (gender << 1) as u8;
    let name = utf16le(nickname);
    let at = pokemon::NICKNAME as usize;
    plain[at..at + name.len()].copy_from_slice(&name);
    let checksum = plain[pokemon::PLAIN as usize..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|word| u16::from_le_bytes(*word) as i64)
        .sum::<i64>()
        & 0xFFFF;
    let at = pokemon::CHECKSUM as usize;
    plain[at..at + 2].copy_from_slice(&(checksum as u16).to_le_bytes());
    let mut out = plain[..pokemon::PLAIN as usize].to_vec();
    out.extend(pokemon::crypt(&plain[pokemon::PLAIN as usize..], constant));
    out
}

/// A piece as the screen text reader makes one: `root` is the layout's real root, which is what
/// ties it to the tree the bar is found in.
fn piece(r#box: i64, text: &str, panel: i64, row: f64, root: i64, top: bool) -> ScreenText {
    ScreenText {
        left: 0.0,
        right: 20.0,
        row,
        glyph: 12.5,
        top,
        root,
        panel,
        parent: panel,
        ..ScreenText::new(r#box, text, false)
    }
}

/// An HP box's layout at `base`, with the pane names its layout file gives it: returns its root.
fn build_box(
    fake: &FakeLayouts,
    picture_class: i64,
    bars: &mut HashMap<i64, i64>,
    base: i64,
    mine: i64,
    theirs: i64,
) -> i64 {
    let memory = &fake.memory;
    let name = |pane: i64, name: &str| {
        let mut text = name.as_bytes().to_vec();
        text.push(0);
        memory.write(pane + lyt::NAME, &text);
    };
    let (work, root) = fake.screen_with(base, false, false);
    fake.project(work, 320.0);
    name(root, "RootPane");
    for (panel, panel_name) in [(mine, "1v"), (theirs, "2g")] {
        fake.pane(panel, root);
        name(panel, panel_name);
        // A pane head is 0x100 bytes, so these are spaced past each other: at 0x40 apart
        // the frame overwrote the bar's height and the bar stopped being findable.
        let bar = panel + 0x200;
        fake.pane_with_vtable(bar, panel, Some(picture_class));
        name(bar, "3a");
        memory.write(bar + lyt::WIDTH, &48.0f32.to_le_bytes());
        memory.write(bar + lyt::HEIGHT, &4.0f32.to_le_bytes());
        bars.insert(panel, bar);
        // The gauge frame, which must not be mistaken for the bar: far too tall.
        let frame = panel + 0x400;
        fake.pane_with_vtable(frame, panel, Some(picture_class));
        name(frame, "3b");
        memory.write(frame + lyt::WIDTH, &128.0f32.to_le_bytes());
        memory.write(frame + lyt::HEIGHT, &32.0f32.to_le_bytes());
    }
    root
}

/// A battle with the HP box's two panels, its bars, and the roster's name buffers.
struct Battle {
    memory: Rc<FakeMemory>,
    strbuf_class: i64,
    param_class: i64,
    picture_class: i64,
    gauge_classes: Vec<i64>,
    fake: FakeLayouts,
    bars: HashMap<i64, i64>,
    root: i64,
    battlers: Rc<Battlers>,
    gauges: Rc<Gauges>,
    reader: BattleReader,
    mine: i64,
    theirs: i64,
}

impl Battle {
    /// `Battle()`: the roster and the Pokémon out of the Python's defaults.
    fn new() -> Battle {
        Battle::with(
            &["Torchic♀", "Wurmple♂"],
            [("Torchic", pokemon::FEMALE), ("Wurmple", pokemon::MALE)],
        )
    }

    fn with(roster: &[&str], out: [(&str, i64); 2]) -> Battle {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let strbuf_class = classes.add("N3gfl3str6StrBufE");
        let param_class = classes.add(pokemon::PARAM_CLASS);
        let picture_class = classes.add(lyt::PICTURE_CLASS);
        // The battle's DLL's, past the executable, where the profile's class names cannot name
        // them: in the tests' executable they were named, and the runtime's gauges never were.
        let dll = FakeClasses::starting(memory.clone(), CODE.1 + 0x1000);
        let gauge_classes: Vec<i64> = hp::GAUGE_CLASSES.iter().map(|name| dll.add(name)).collect();
        let fake = FakeLayouts::new(&memory, &classes);
        let mut bars = HashMap::new();
        let root = build_box(&fake, picture_class, &mut bars, BOX, MINE, THEIRS);
        fake.track().unwrap();

        for (index, name) in roster.iter().enumerate() {
            let buffer = BUFFERS + index as i64 * 0xE8;
            let text = buffer + 0x40;
            memory.u32(buffer, &[strbuf_class]);
            memory.u32(buffer + strbuf::TEXT, &[text]);
            memory.u16(buffer + strbuf::LENGTH, name.chars().count() as i64);
            memory.write(text, &utf16le(name));
        }
        let battlers = Rc::new(Battlers::new(
            memory.clone(),
            Rc::new(ClassNames::new(memory.clone(), CODE)),
        ));
        battlers
            .track((0..roster.len() as i64).map(|i| BUFFERS + i * 0xE8), false)
            .unwrap();
        let gauges = Rc::new(Gauges::new(
            memory.clone(),
            Rc::new(ClassNames::new(memory.clone(), CODE)),
        ));
        let reader = BattleReader::new(fake.layouts.clone(), battlers.clone(), gauges.clone());
        let mut battle = Battle {
            memory,
            strbuf_class,
            param_class,
            picture_class,
            gauge_classes,
            fake,
            bars,
            root,
            battlers,
            gauges,
            reader,
            mine: 0,
            theirs: 0,
        };
        let ours = battle.pokemon(0, out[0].0, out[0].1);
        battle.mine = battle.gauge(0, MY_LEVEL, ours, MINE);
        let theirs = battle.pokemon(1, out[1].0, out[1].1);
        battle.theirs = battle.gauge(1, THEIR_LEVEL, theirs, THEIRS);
        battle
            .gauges
            .track([battle.mine], [battle.theirs], false)
            .unwrap();
        battle
    }

    /// A battle record for a Pokémon, holding its PokemonParam. Returns the record.
    fn pokemon(&self, index: i64, nickname: &str, gender: i64) -> i64 {
        let (param, battle_record) = (PARAMS + index * 0x200, RECORDS + index * 0x244);
        self.memory
            .u32(param, &[self.param_class, param + pokemon::RECORD_IN_PARAM]);
        self.memory
            .write(param + pokemon::RECORD_IN_PARAM, &record(nickname, gender));
        self.memory.u32(battle_record + RECORD_PARAM, &[param]);
        battle_record
    }

    /// `gauge(index, level, battle_record, group)`: shown and target both 20.
    fn gauge(&self, index: i64, level: i64, battle_record: i64, group: i64) -> i64 {
        self.gauge_with(index, level, battle_record, group, 20, 20)
    }

    /// A gauge showing a battle record, holding the group its reading hangs from, its level
    /// TextBox and its bar. The index is its battle position, the player's even and the enemy's
    /// odd, as a double battle numbers them (spike 150). Returns the gauge.
    fn gauge_with(
        &self,
        index: i64,
        level: i64,
        battle_record: i64,
        group: i64,
        shown: i64,
        target: i64,
    ) -> i64 {
        let gauge = GAUGES + index * 0x9C;
        self.memory
            .u32(gauge, &[self.gauge_classes[(index % 2) as usize]]);
        self.memory.u32(gauge + GAUGE_RECORD, &[battle_record]);
        self.memory.u32(gauge + hp::GAUGE_LEVEL, &[level]);
        self.memory.u32(gauge + hp::GAUGE_SIDE, &[index]);
        self.memory.u32(gauge + hp::GAUGE_GROUP, &[group]);
        self.memory.u32(
            gauge + hp::GAUGE_BAR,
            &[self.bars.get(&group).copied().unwrap_or(0)],
        );
        self.counting(gauge, shown, target);
        gauge
    }

    fn counting(&self, gauge: i64, shown: i64, target: i64) {
        self.memory
            .u32(gauge + hp::GAUGE_SHOWN_HP, &[shown, target]);
    }

    /// Build an HP box's layout at `base`, with the pane names its layout file gives it, and
    /// return its root.
    fn r#box(&mut self, base: i64, mine: i64, theirs: i64) -> i64 {
        build_box(
            &self.fake,
            self.picture_class,
            &mut self.bars,
            base,
            mine,
            theirs,
        )
    }

    fn name(&self, pane: i64, name: &str) {
        let mut text = name.as_bytes().to_vec();
        text.push(0);
        self.memory.write(pane + lyt::NAME, &text);
    }

    fn bar(&self, panel: i64, width: f64) {
        self.memory.write(
            self.bars[&panel] + lyt::WIDTH,
            &(width as f32).to_le_bytes(),
        );
    }

    /// A new gauge list and a reader over it, as the tests that begin without gauges do.
    fn without_gauges(&mut self) {
        self.gauges = Rc::new(Gauges::new(
            self.memory.clone(),
            Rc::new(ClassNames::new(self.memory.clone(), CODE)),
        ));
        self.reader = BattleReader::new(
            self.fake.layouts.clone(),
            self.battlers.clone(),
            self.gauges.clone(),
        );
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

/// `screen(battle)`: the player's 20/20 at level 5 and the enemy's level 2.
fn screen(battle: &Battle) -> Vec<ScreenText> {
    screen_with(battle, "20/ 20", "5", "2")
}

fn screen_with(battle: &Battle, mine: &str, my_level: &str, their_level: &str) -> Vec<ScreenText> {
    vec![
        piece(MY_LEVEL, my_level, MINE, 906.0, battle.root, false),
        piece(MY_HP, mine, MINE, 888.0, battle.root, false),
        piece(THEIR_LEVEL, their_level, THEIRS, 1108.0, battle.root, false),
    ]
}

fn settle(battle: &mut Battle, pieces: &[ScreenText]) -> Vec<String> {
    battle.heard(pieces);
    battle.heard(pieces)
}

fn strings(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|text| text.to_string()).collect()
}

/// battles2.ctm: with Emrys and Blaze in the party, the wild Poochyena was said as "Blaze♀ L3
/// 100%", the second name in the heap being the party's second Pokémon.
#[test]
fn test_the_enemy_is_named_by_its_gauge_whatever_the_party_holds() {
    let mut battle = Battle::with(
        &["Emrys♂", "Blaze♀", "Poochyena♂"],
        [("Emrys", pokemon::MALE), ("Poochyena", pokemon::MALE)],
    );
    let pieces = screen(&battle);
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Emrys♂ L5 20/20", "Poochyena♂ L2 100%"])
    );
}

/// The Chinese fan Omega Ruby writes a plate's ♂ as the font's U+E08E, and its battles said
/// nothing, either side, nor on a key (movie2_route103 on it).
#[test]
fn test_a_plate_signed_with_the_fonts_own_glyph_is_said_with_the_sign() {
    let mut battle = Battle::with(
        &["Blaze\u{e08e}", "刺尾虫\u{e08f}"],
        [("Blaze", pokemon::MALE), ("刺尾虫", pokemon::FEMALE)],
    );
    let pieces = screen(&battle);
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Blaze♂ L5 20/20", "刺尾虫♀ L2 100%"])
    );
}

/// After switching to Blaze, battles2.ctm kept saying Emrys's name with Blaze's HP.
#[test]
fn test_a_switch_is_heard_under_the_name_of_the_pokemon_sent_out() {
    let mut battle = Battle::with(
        &["Emrys♂", "Blaze♀", "Poochyena♂"],
        [("Emrys", pokemon::MALE), ("Poochyena", pokemon::MALE)],
    );
    let pieces = screen(&battle);
    settle(&mut battle, &pieces);
    let blaze = battle.pokemon(2, "Blaze", pokemon::FEMALE);
    battle.memory.u32(battle.mine + GAUGE_RECORD, &[blaze]);
    let pieces = screen_with(&battle, "21/ 21", "5", "2");
    assert_eq!(settle(&mut battle, &pieces), strings(&["Blaze♀ L5 21/21"]));
}

/// A Potion took Emrys from 3/19 to 19/19, and "9/19" was said on the way (battles2.ctm). The
/// gauge counts the HP it shows towards the HP it is going to, a point a frame (spike 67).
#[test]
fn test_hp_is_said_once_the_box_has_counted_to_where_it_is_going() {
    let mut battle = Battle::new();
    battle.counting(battle.mine, 3, 3);
    let pieces = screen_with(&battle, "3/ 20", "5", "2");
    settle(&mut battle, &pieces);
    battle.counting(battle.mine, 9, 20);
    let pieces = screen_with(&battle, "9/ 20", "5", "2");
    assert_eq!(settle(&mut battle, &pieces), strings(&[]));
    assert_eq!(battle.heard(&pieces), strings(&[]));
    battle.counting(battle.mine, 20, 20);
    let pieces = screen(&battle);
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Torchic♀ L5 20/20"])
    );
}

/// A gauge is found by a sweep, so a box can be up before its gauge is known. Until then
/// nothing is said rather than a guess.
#[test]
fn test_a_side_waits_for_its_gauge_to_be_found() {
    let mut battle = Battle::new();
    battle.without_gauges();
    let pieces = screen(&battle);
    assert_eq!(settle(&mut battle, &pieces), strings(&[]));
    battle.gauges.track([battle.mine], [], true).unwrap();
    battle.gauges.track([], [battle.theirs], true).unwrap();
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Torchic♀ L5 20/20", "Wurmple♂ L2 100%"])
    );
    assert!(!battle.reader.sweep_wanted());
}

/// A sweep taken as the HP box came on screen can come before the game has built the gauge:
/// the wild Zigzagoon's first reading was lost when nothing asked again (movie2, spike 99).
#[test]
fn test_a_sweep_is_asked_for_when_a_gauge_is_first_wanted_and_again_while_it_is_missing() {
    let mut battle = Battle::new();
    battle.without_gauges();
    assert!(!battle.reader.sweep_wanted());
    let pieces = screen(&battle);
    battle.heard(&pieces);
    assert!(battle.reader.sweep_wanted());
    let mut asks: Vec<bool> = Vec::new();
    for _ in 0..=REASK {
        battle.heard(&pieces);
        asks.push(battle.reader.sweep_wanted());
    }
    let mut expected = vec![false; REASK as usize];
    expected.push(true);
    assert_eq!(asks, expected);
}

/// Asked for as long as the box was up, a battle whose gauge class was never learned cost Sun
/// 75 whole walks (sun_9.ctm).
#[test]
fn test_a_gauge_no_sweep_finds_is_asked_for_three_times_at_most() {
    let mut battle = Battle::new();
    battle.without_gauges();
    let pieces = screen(&battle);
    let mut asks = 0;
    for _ in 0..(MAX_ASKS + 2) * (REASK + 1) {
        battle.heard(&pieces);
        asks += i64::from(battle.reader.sweep_wanted());
    }
    assert_eq!(asks, MAX_ASKS);
}

/// A new battle builds new gauges, and the last one's memory is somebody else's.
#[test]
fn test_a_gauge_whose_memory_another_object_has_taken_is_dropped() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    settle(&mut battle, &pieces);
    battle.memory.u32(battle.mine, &[battle.strbuf_class]);
    battle.reader.begin();
    let mut heard: Vec<String> = Vec::new();
    for _ in 0..hp::OURS_FIRST + 2 {
        heard.extend(battle.heard(&pieces));
    }
    // The enemy waits for the player's side, which no gauge now says, and is said once it has
    // waited long enough.
    assert_eq!(heard, strings(&["Wurmple♂ L2 100%"]));
    assert!(battle.reader.sweep_wanted());
}

/// The enemy's panel comes on a poll or two ahead of the player's, and the enemy was said
/// first in one battle and second in the next, which heard as a race.
#[test]
fn test_our_side_is_said_before_the_enemys_as_a_battle_opens() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    settle(&mut battle, &pieces);
    battle.reader.begin();
    let theirs_alone = [piece(THEIR_LEVEL, "2", THEIRS, 1108.0, battle.root, false)];
    let mut heard = battle.heard(&theirs_alone);
    heard.extend(battle.heard(&theirs_alone));
    heard.extend(battle.heard(&pieces));
    heard.extend(battle.heard(&pieces));
    assert_eq!(heard, strings(&["Torchic♀ L5 20/20", "Wurmple♂ L2 100%"]));
}

/// The next battle said only the side whose HP had changed: the player's "Blaze♂ L10 20/30"
/// was said as one battle ended and never as the next began (spike 138, movie4). The battle's own
/// start is what says a new one has begun, since the name plates keep their text.
#[test]
fn test_a_new_battle_says_both_sides_again() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Torchic♀ L5 20/20", "Wurmple♂ L2 100%"])
    );
    assert_eq!(settle(&mut battle, &pieces), strings(&[]));
    battle.reader.begin();
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Torchic♀ L5 20/20", "Wurmple♂ L2 100%"])
    );
}

/// A double battle draws both of the player's Pokémon in one panel and both enemies in another,
/// each in a group of its own that its gauge holds (spike 150, movie15): read a panel at a time,
/// Blaze was said with Beldum's HP and Beldum never, and both levels of a side were screen text.
#[test]
fn test_a_double_battle_says_each_pokemon_from_its_own_gauge() {
    let mut battle = Battle::with(
        &["Blaze♂", "Lotad♂", "Seedot♂"],
        [("Blaze", pokemon::MALE), ("Lotad", pokemon::MALE)],
    );
    let groups = [(MINE, MINE + 0x800), (THEIRS, THEIRS + 0x800)];
    for (panel, group) in groups {
        battle.fake.pane(group, panel);
        let bar = group + 0x200;
        battle
            .fake
            .pane_with_vtable(bar, group, Some(battle.picture_class));
        battle
            .memory
            .write(bar + lyt::WIDTH, &48.0f32.to_le_bytes());
        battle
            .memory
            .write(bar + lyt::HEIGHT, &4.0f32.to_le_bytes());
        battle.bars.insert(group, bar);
    }
    battle.fake.track().unwrap();
    let beldum_record = battle.pokemon(2, "Beldum", 2);
    let beldum = battle.gauge(2, 11, beldum_record, groups[0].1);
    let seedot_record = battle.pokemon(3, "Seedot", pokemon::MALE);
    let seedot = battle.gauge(3, 13, seedot_record, groups[1].1);
    battle
        .gauges
        .track([battle.mine, beldum], [battle.theirs, seedot], false)
        .unwrap();
    battle.bar(groups[1].1, 24.0);

    let root = battle.root;
    let at = |r#box: i64, text: &str, panel: i64, group: i64, row: f64| ScreenText {
        left: 0.0,
        right: 20.0,
        row,
        glyph: 12.5,
        top: true,
        root,
        panel,
        parent: group,
        ..ScreenText::new(r#box, text, false)
    };

    let pieces = [
        at(MY_LEVEL, "13", MINE, MINE, 907.0),
        at(MY_HP, "29/ 36", MINE, MINE, 889.0),
        at(11, "9", MINE, groups[0].1, 907.0),
        at(12, "26/ 26", MINE, groups[0].1, 889.0),
        at(THEIR_LEVEL, "9", THEIRS, THEIRS, 1108.0),
        at(13, "9", THEIRS, groups[1].1, 1108.0),
    ];
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&[
            "Blaze♂ L13 29/36",
            "Beldum L9 26/26",
            "Lotad♂ L9 100%",
            "Seedot♂ L9 50%"
        ])
    );
    assert!(
        pieces
            .iter()
            .all(|piece| battle.reader.owns(piece).unwrap())
    );
}

#[test]
fn test_two_names_alike_are_told_apart_by_gender() {
    let mut battle = Battle::with(
        &["Poochyena♂", "Poochyena♀"],
        [("Poochyena", pokemon::MALE), ("Poochyena", pokemon::FEMALE)],
    );
    let pieces = screen(&battle);
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Poochyena♂ L5 20/20", "Poochyena♀ L2 100%"])
    );
}

#[test]
fn test_a_pokemon_with_no_gender_is_said_by_its_name_alone() {
    let mut battle = Battle::with(
        &["Torchic♀"],
        [("Torchic", pokemon::FEMALE), ("Magnemite", 2)],
    );
    let pieces = screen(&battle);
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Torchic♀ L5 20/20", "Magnemite L2 100%"])
    );
}

#[test]
fn test_each_side_is_said_once_it_has_held_still() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    assert_eq!(battle.heard(&pieces), strings(&[]));
    assert_eq!(
        battle.heard(&pieces),
        strings(&["Torchic♀ L5 20/20", "Wurmple♂ L2 100%"])
    );
}

#[test]
fn test_a_settled_reading_is_not_said_twice() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    battle.heard(&pieces);
    battle.heard(&pieces);
    assert_eq!(battle.heard(&pieces), strings(&[]));
}

#[test]
fn test_the_player_is_said_again_when_its_hp_changes() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    battle.heard(&pieces);
    battle.heard(&pieces);
    let hurt = screen_with(&battle, "18/ 20", "5", "2");
    assert_eq!(battle.heard(&hurt), strings(&[]));
    assert_eq!(battle.heard(&hurt), strings(&["Torchic♀ L5 18/20"]));
}

/// The player's panel leaves the screen while the player's Pokémon attacks and the enemy's
/// stays, draining: the reading waited for both sides' turns (spike 144).
#[test]
fn test_the_enemy_is_said_as_the_hit_lands_while_the_players_panel_is_away() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    settle(&mut battle, &pieces);
    let theirs_alone = [piece(THEIR_LEVEL, "2", THEIRS, 1108.0, battle.root, false)];
    battle.bar(THEIRS, 24.0);
    assert_eq!(
        settle(&mut battle, &theirs_alone),
        strings(&["Wurmple♂ L2 50%"])
    );
}

/// Sliding in, or levelling up, the player's panel shows its level and not its HP, as an
/// enemy's does: "Blaze♂ L9 75%" was said (movie4, spike 144). Its gauge says whose side it is.
#[test]
fn test_the_players_panel_with_its_level_alone_is_not_an_enemy() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    settle(&mut battle, &pieces);
    let mine_alone = [piece(MY_LEVEL, "5", MINE, 906.0, battle.root, false)];
    assert_eq!(settle(&mut battle, &mine_alone), strings(&[]));
}

/// The Wurmple's gauge read 0 for 2.5 s before its panel left (movie2), and shift and R said
/// it at 29% on the victory screen, the last reading said.
#[test]
fn test_the_enemys_drop_to_nothing_is_said_before_its_panel_goes() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    settle(&mut battle, &pieces);
    battle.bar(THEIRS, 0.0);
    battle.counting(battle.theirs, 0, 0);
    let theirs_alone = [piece(THEIR_LEVEL, "2", THEIRS, 1108.0, battle.root, false)];
    assert_eq!(
        settle(&mut battle, &theirs_alone),
        strings(&["Wurmple♂ L2 0%"])
    );
}

/// The enemy's bar read 48, 30, 27 twenty milliseconds later, then 0, for one Ember (spike 54).
/// Speaking those would say 62% and then 0% a moment later.
#[test]
fn test_a_draining_bar_is_only_said_where_it_stops() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    battle.heard(&pieces);
    battle.heard(&pieces);
    let mut said: Vec<String> = Vec::new();
    for width in [30.0, 27.0, 0.0, 0.0] {
        battle.bar(THEIRS, width);
        said.extend(battle.heard(&pieces));
    }
    assert_eq!(said, strings(&["Wurmple♂ L2 0%"]));
}

#[test]
fn test_the_enemy_is_a_percentage_of_the_widest_bar_seen() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    battle.heard(&pieces);
    battle.heard(&pieces);
    battle.bar(THEIRS, 24.0);
    battle.heard(&pieces);
    assert_eq!(battle.heard(&pieces), strings(&["Wurmple♂ L2 50%"]));
}

/// The plate carries the sign, and saying the name without it now and with it a sweep later
/// would say the side twice.
#[test]
fn test_a_gendered_pokemon_waits_for_its_name_plate() {
    let mut battle = Battle::with(
        &["Torchic", "Wurmple"],
        [("Torchic", pokemon::FEMALE), ("Wurmple", pokemon::MALE)],
    );
    let pieces = screen(&battle);
    assert_eq!(battle.heard(&pieces), strings(&[]));
    assert_eq!(battle.heard(&pieces), strings(&[]));
}

/// No plate ends in a sign then, and the whole battle went unsaid until 2026-09-18.
#[test]
fn test_a_battle_of_pokemon_with_no_gender_is_read_from_its_gauges() {
    let mut battle = Battle::with(&[], [("Magnemite", 2), ("Voltorb", 2)]);
    let pieces = screen(&battle);
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Magnemite L5 20/20", "Voltorb L2 100%"])
    );
    assert!(battle.reader.owns(&screen(&battle)[0]).unwrap());
}

/// The party screen's slot shows a bare level, and its HP as three pieces rather than one, so
/// it looked like a side with no numbers: mid-battle it was announced as "Wurmple L6 100%"
/// (battles.ctm replay, 2026-09-16). Only the layout holding the player's panel is the battle's.
#[test]
fn test_a_panel_outside_the_hp_box_is_not_an_enemy() {
    let mut battle = Battle::new();
    let elsewhere = 0x14611FC0;
    let mut pieces = screen(&battle);
    pieces.push(piece(9, "6", elsewhere, 74.0, 0x1460D914, false));
    pieces.push(piece(10, "22", elsewhere, 74.0, 0x1460D914, false));
    assert_eq!(battle.heard(&pieces), strings(&[]));
    assert_eq!(
        battle.heard(&pieces),
        strings(&["Torchic♀ L5 20/20", "Wurmple♂ L2 100%"])
    );
}

/// Screen text said the box as it slid in, "5 18/ 20 3", on top of this reader's own line.
/// A level arriving alone later, as the second battle's "6" did, is still the
/// box's.
#[test]
fn test_the_hp_box_is_left_to_this_reader_once_found() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    battle.heard(&pieces);
    assert!(
        pieces
            .iter()
            .all(|piece| battle.reader.owns(piece).unwrap())
    );
    assert!(
        battle
            .reader
            .owns(&piece(4, "6", MINE, 906.0, battle.root, false))
            .unwrap()
    );
}

#[test]
fn test_nothing_is_left_to_this_reader_before_a_battle_is_seen() {
    let battle = Battle::new();
    assert!(!battle.reader.owns(&screen(&battle)[0]).unwrap());
}

/// The enemy's panel came on screen a poll before the player's, and "3" was said as the first
/// battle of battles.ctm opened (2026-09-16 replay).
#[test]
fn test_the_enemys_level_ahead_of_the_players_panel_is_left_to_this_reader() {
    let mut battle = Battle::new();
    let level = piece(3, "3", THEIRS, 1108.0, battle.root, true);
    assert_eq!(battle.heard(std::slice::from_ref(&level)), strings(&[]));
    assert!(battle.reader.owns(&level).unwrap());
}

/// The party screen and the summary show a level beside a bar, and are on the bottom screen.
#[test]
fn test_a_level_beside_a_bar_on_the_bottom_screen_is_not_the_enemys() {
    let mut battle = Battle::new();
    let level = piece(30, "6", THEIRS, 74.0, battle.root, false);
    battle.heard(std::slice::from_ref(&level));
    assert!(!battle.reader.owns(&level).unwrap());
}

/// The level up's stats are numbers on the top screen too, with no bar.
#[test]
fn test_a_number_with_no_bar_beside_it_is_not_the_enemys_level() {
    let mut battle = Battle::new();
    let stats = 0x14D60000;
    battle.fake.pane(stats, battle.root);
    battle.fake.track().unwrap();
    let value = piece(9, "13", stats, 1050.0, battle.root, true);
    battle.heard(std::slice::from_ref(&value));
    assert!(!battle.reader.owns(&value).unwrap());
}

#[test]
fn test_the_party_screen_is_not_the_hp_box() {
    let mut battle = Battle::new();
    let party = piece(9, "6", 0x14611FC0, 74.0, 0x1460D914, false);
    let mut pieces = screen(&battle);
    pieces.push(party.clone());
    battle.heard(&pieces);
    assert!(!battle.reader.owns(&party).unwrap());
}

/// A layout built where the box used to be, once the battle is over, is somebody else's: the
/// plates lose their signs and the gauges their Pokémon.
#[test]
fn test_nothing_is_left_to_this_reader_out_of_a_battle() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    battle.heard(&pieces);
    battle
        .memory
        .u16(BUFFERS + strbuf::LENGTH, "Torchic".chars().count() as i64);
    battle.memory.u16(
        BUFFERS + strbuf::LENGTH + 0xE8,
        "Wurmple".chars().count() as i64,
    );
    for gauge in [battle.mine, battle.theirs] {
        battle.memory.u32(gauge + GAUGE_RECORD, &[0]);
    }
    battle.heard(&pieces);
    assert!(!battle.reader.owns(&pieces[0]).unwrap());
}

/// The next battle can build the box at another address, and its pieces can arrive one at a
/// time, a level before the HP (battles.ctm's second battle). The layout's pane names are the same
/// file's, so its first piece is known.
#[test]
fn test_a_box_built_again_elsewhere_is_known_by_its_pane_names() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    battle.heard(&pieces);
    let again = 0x14E00000;
    let root = battle.r#box(again, again + 0x2000, again + 0x3000);
    battle.fake.track().unwrap();
    battle.heard(&pieces);
    assert!(
        battle
            .reader
            .owns(&piece(4, "6", again + 0x2000, 906.0, root, false))
            .unwrap()
    );
    let other = 0x14F00000;
    let root = battle.r#box(other, other + 0x2000, other + 0x3000);
    battle.name(other + 0x2000, "5q");
    battle.fake.track().unwrap();
    assert!(
        !battle
            .reader
            .owns(&piece(5, "6", other + 0x2000, 906.0, root, false))
            .unwrap()
    );
}

#[test]
fn test_a_panel_with_no_level_is_not_a_side() {
    let mut battle = Battle::new();
    let pieces = [piece(
        1,
        "Choose a Pokémon.",
        MINE,
        906.0,
        battle.root,
        false,
    )];
    assert_eq!(battle.heard(&pieces), strings(&[]));
    assert_eq!(battle.heard(&pieces), strings(&[]));
}

/// Reading every StrBuf every poll was 50 reads a poll through menus.ctm, which has no battle
/// (spike 62). They are checked when a sweep hands them over, and only names are read after.
#[test]
fn test_a_buffer_holding_no_name_is_not_read_every_poll() {
    let battle = Battle::with(
        &["Torchic", "Wurmple"],
        [("Torchic", pokemon::FEMALE), ("Wurmple", pokemon::MALE)],
    );
    let counting = Counting::new(battle.memory.clone());
    let battlers = Battlers::new(
        counting.clone(),
        Rc::new(ClassNames::new(battle.memory.clone(), CODE)),
    );
    battlers
        .track((0..2).map(|i| BUFFERS + i * 0xE8), false)
        .unwrap();
    let before = counting.reads.get();
    for _ in 0..5 {
        assert_eq!(battlers.read().unwrap(), strings(&[]));
    }
    assert_eq!(counting.reads.get(), before);
}

#[test]
fn test_a_buffer_that_gains_a_name_is_read_from_the_next_sweep() {
    let battle = Battle::with(
        &["Torchic", "Wurmple"],
        [("Torchic", pokemon::FEMALE), ("Wurmple", pokemon::MALE)],
    );
    let battlers = &battle.battlers;
    assert_eq!(battlers.read().unwrap(), strings(&[]));
    battle.memory.write(BUFFERS + 0x40, &utf16le("Torchic♀"));
    battle
        .memory
        .u16(BUFFERS + strbuf::LENGTH, "Torchic♀".chars().count() as i64);
    assert_eq!(battlers.read().unwrap(), strings(&[]));
    battlers.track([], false).unwrap();
    assert_eq!(battlers.read().unwrap(), strings(&["Torchic♀"]));
}

/// A buffer is taken empty and named a moment later, and its block is swept again then; the
/// rest wait for a whole sweep to be read again (spike 110).
#[test]
fn test_a_name_buffer_found_again_by_a_sweep_of_new_blocks_is_read_again() {
    let battle = Battle::with(
        &["Torchic♀"],
        [("Torchic", pokemon::FEMALE), ("Wurmple", pokemon::MALE)],
    );
    let later = BUFFERS + 0xE8;
    battle.memory.u32(later, &[battle.strbuf_class]);
    battle.memory.u32(later + strbuf::TEXT, &[later + 0x40]);
    battle.battlers.track([later], true).unwrap();
    assert_eq!(battle.battlers.read().unwrap(), strings(&["Torchic♀"]));
    battle
        .memory
        .u16(later + strbuf::LENGTH, "Ralts♂".chars().count() as i64);
    battle.memory.write(later + 0x40, &utf16le("Ralts♂"));
    battle.battlers.track([later], true).unwrap();
    assert_eq!(
        battle.battlers.read().unwrap(),
        strings(&["Torchic♀", "Ralts♂"])
    );
}

/// A status condition is an icon in the HP box, named by its image (spike 147): paralysed, the
/// player's side is "Torchic♀ L5 20/20 PARALYSIS", and said again without it once cured.
#[test]
fn test_a_status_condition_in_a_panel_is_said_after_its_reading() {
    let mut battle = Battle::new();
    let pieces = screen(&battle);
    settle(&mut battle, &pieces);
    let icon = ScreenText {
        left: 30.0,
        right: 50.0,
        row: 906.0,
        glyph: 10.0,
        root: battle.root,
        panel: MINE,
        parent: MINE,
        picture: true,
        ..ScreenText::new(0x0A00, "PARALYSIS", false)
    };
    let mut paralysed = pieces.clone();
    paralysed.push(icon);
    assert_eq!(
        settle(&mut battle, &paralysed),
        strings(&["Torchic♀ L5 20/20 PARALYSIS"])
    );
    assert_eq!(
        settle(&mut battle, &pieces),
        strings(&["Torchic♀ L5 20/20"])
    );
}

const TRAY_ROOT: i64 = 0x14CE8000;

/// The tray of balls a trainer battle opens with (spike 285): a top-screen layout, the player's
/// panel on the left and the opponent's on the right, each a tray picture and six slots.
fn tray(theirs: i64, mine: i64) -> (FakeLayouts, HashMap<i64, String>) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let picture = classes.add(lyt::PICTURE_CLASS);
    let fake = FakeLayouts::new(&memory, &classes);
    let (work, root) = fake.screen_with(TRAY_ROOT, true, false);
    fake.project(work, 400.0);
    let mut images: HashMap<i64, String> = HashMap::new();
    for (side, (x, balls)) in [(-120.0f32, mine), (120.0f32, theirs)]
        .into_iter()
        .enumerate()
    {
        let panel = TRAY_ROOT + 0x1000 + side as i64 * 0x1000;
        fake.pane(panel, root);
        memory.write(panel + fake.shape.x, &x.to_le_bytes());
        for slot in 0..7 {
            let pane = panel + 0x100 * (slot + 1);
            fake.pane_with_vtable(pane, panel, Some(picture));
            let image = if slot == 0 {
                hp::TRAY
            } else if slot <= balls {
                "ball_normal.bclim"
            } else {
                hp::EMPTY
            };
            images.insert(pane, image.to_string());
        }
    }
    fake.track().unwrap();
    (fake, images)
}

/// Spike 285, x_battle-count.ctm: Youngster Jayden's tray held two balls beside the player's
/// six, told once, after "You are challenged by Youngster Jayden!" and once it has held for a
/// poll.
#[test]
fn test_a_trainers_count_is_the_balls_in_the_right_hand_tray_after_the_first_page() {
    let (fake, images) = tray(2, 6);
    let mut party = hp::OpponentsParty::new(
        fake.layouts.clone(),
        Rc::new(move |pane| Ok(images.get(&pane).cloned())),
    );
    party.begin();
    assert_eq!(party.poll().unwrap(), None);
    party.paged();
    let polled: Vec<Option<i64>> = (0..3).map(|_| party.poll().unwrap()).collect();
    assert_eq!(polled, [None, Some(2), None]);
}

/// A wild battle draws no tray: nothing is told, and the layouts are walked for it only
/// `TRAY_POLLS` screen polls after the first page.
#[test]
fn test_a_battle_with_no_tray_is_looked_at_only_for_a_while() {
    let (fake, _images) = tray(1, 6);
    let calls: Rc<RefCell<Vec<i64>>> = Rc::new(RefCell::new(Vec::new()));
    let mut party = hp::OpponentsParty::new(fake.layouts.clone(), {
        let calls = calls.clone();
        Rc::new(move |pane| {
            calls.borrow_mut().push(pane);
            Ok(None)
        })
    });
    party.begin();
    party.paged();
    let polled: Vec<Option<i64>> = (0..hp::TRAY_POLLS + 5)
        .map(|_| party.poll().unwrap())
        .collect();
    assert_eq!(polled, vec![None; (hp::TRAY_POLLS + 5) as usize]);
    let looked = calls.borrow().len();
    party.paged();
    party.poll().unwrap();
    assert!(looked != 0 && calls.borrow().len() == looked);
}
