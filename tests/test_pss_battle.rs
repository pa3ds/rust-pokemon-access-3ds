//! A link battle's timers, drawn a digit a picture (the PSS's multiplayer, spike A).

use std::cell::Cell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{FakeClasses, FakeMemory};
use crate::games::common::field::keys::{BATTLE_TIME, COMMAND_TIME, DEFAULT_CONTEXTS, PSS_BATTLE};
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::lyt;
use crate::games::gen6::apps::{BATTLE, FIELD};
use crate::games::gen6::subscreen::pss_battle;

const COLON: (f64, f64) = (0.0, 0.531);
const LABEL: (f64, f64) = (0.0, 1.0);

/// `CORNERS[digit]`: a digit's corner in the image.
fn corner(digit: i64) -> (f64, f64) {
    pss_battle::DIGITS
        .iter()
        .find(|(_corner, d)| *d == digit)
        .expect("a digit")
        .0
}

/// Y's `btl_time.bclyt` as its link battle showed it: under the root, Battle Time's row, its
/// label, two digits, the colon and two more, and Command Time's, its label and three digits, the
/// hundreds hidden; and a picture under the root itself, off the screen.
struct Battle {
    memory: Rc<FakeMemory>,
    fake: FakeLayouts,
    root: i64,
    pictures: IndexMap<String, i64>,
    timers: pss_battle::BattleTimers,
    ticks: Rc<Cell<usize>>,
}

impl Battle {
    fn new() -> Battle {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        let fake = FakeLayouts::new(&memory, &classes);
        let picture = classes.add(lyt::PICTURE_CLASS);
        let (_work, root) = fake.screen_with(0x08200000, true, false);
        let mut pictures: IndexMap<String, i64> = IndexMap::new();
        let mut at = root + 0x1000;
        let rows: [(&str, &[(&str, f32)]); 2] = [
            (
                "battle",
                &[
                    ("label", 135.0),
                    ("m1", 156.0),
                    ("m2", 166.0),
                    ("colon", 173.0),
                    ("s1", 181.0),
                    ("s2", 191.0),
                ],
            ),
            (
                "command",
                &[("label", 135.0), ("c1", 181.0), ("c2", 191.0), ("c0", 0.0)],
            ),
        ];
        for (row, placed) in rows {
            let group = at;
            fake.pane(group, root);
            for &(name, x) in placed {
                at += 0x400;
                fake.pane_with_vtable(at, group, Some(picture));
                memory.write(at + lyt::X, &x.to_le_bytes());
                pictures.insert(format!("{row} {name}"), at);
            }
            at += 0x400;
        }
        fake.pane_with_vtable(at, root, Some(picture));
        place(&memory, at, corner(0));
        fake.track().unwrap();
        place(&memory, pictures["battle label"], LABEL);
        place(&memory, pictures["battle colon"], COLON);
        place(&memory, pictures["command label"], LABEL);
        fake.visible(pictures["command c0"], false);
        let ticks = Rc::new(Cell::new(0));
        let timers = pss_battle::BattleTimers::new(
            fake.layouts.clone(),
            Rc::new(move |asked| {
                (asked == root).then(|| {
                    Rc::new(LayoutFile {
                        name: pss_battle::FILE.to_string(),
                        ..LayoutFile::new(Vec::new(), IndexMap::new(), IndexMap::new())
                    })
                })
            }),
            Some({
                let ticks = ticks.clone();
                Rc::new(move || ticks.set(ticks.get() + 1))
            }),
        );
        Battle {
            memory,
            fake,
            root,
            pictures,
            timers,
            ticks,
        }
    }

    /// Draw Battle Time, "56:35", and Command Time, "7", hidden where None.
    fn show(&self, battle: &str, command: Option<&str>) {
        let digits: Vec<char> = battle.replace(':', "").chars().collect();
        assert_eq!(digits.len(), 4);
        for (name, digit) in ["m1", "m2", "s1", "s2"].into_iter().zip(digits) {
            place(
                &self.memory,
                self.pictures[&format!("battle {name}")],
                corner(i64::from(digit.to_digit(10).unwrap())),
            );
        }
        self.fake
            .visible(self.pictures["command label"], command.is_some());
        let (tens, ones) = (self.pictures["command c1"], self.pictures["command c2"]);
        self.fake
            .visible(tens, command.is_some_and(|command| command.len() > 1));
        self.fake.visible(ones, command.is_some());
        if let Some(command) = command {
            let digits: Vec<i64> = command
                .chars()
                .map(|c| i64::from(c.to_digit(10).unwrap()))
                .collect();
            place(&self.memory, ones, corner(digits[digits.len() - 1]));
            if digits.len() > 1 {
                place(&self.memory, tens, corner(digits[digits.len() - 2]));
            }
        }
        self.fake.layouts.refresh();
    }
}

fn place(memory: &FakeMemory, picture: i64, corner: (f64, f64)) {
    let mut uv = (corner.0 as f32).to_le_bytes().to_vec();
    uv.extend((corner.1 as f32).to_le_bytes());
    memory.write(picture + lyt::PICTURE_UV, &uv);
}

/// Y's top screen drew "Battle Time: 56:35" and "Command Time: 7", its tens hidden.
#[test]
fn test_c_says_command_time_and_shift_c_battle_time_as_the_top_screen_draws_them() {
    let battle = Battle::new();
    battle.show("56:35", Some("7"));
    let timers = &battle.timers;
    assert_eq!(timers.context(Some(BATTLE)), Some(PSS_BATTLE));
    assert_eq!(timers.context(Some(FIELD)), None);
    assert_eq!(timers.press(COMMAND_TIME, Some(BATTLE)).unwrap(), "7");
    assert_eq!(timers.press(BATTLE_TIME, Some(BATTLE)).unwrap(), "56:35");
    battle.show("55:58", Some("81"));
    assert_eq!(timers.press(COMMAND_TIME, Some(BATTLE)).unwrap(), "81");
    assert_eq!(timers.press(BATTLE_TIME, Some(BATTLE)).unwrap(), "55:58");
}

/// Command Time goes once the player has chosen, Battle Time staying (Alpha Sapphire's top
/// screen); a battle with no time limit draws neither.
#[test]
fn test_a_timer_not_shown_is_no_timer() {
    let battle = Battle::new();
    battle.show("56:34", None);
    assert_eq!(
        battle.timers.press(COMMAND_TIME, Some(BATTLE)).unwrap(),
        "no timer"
    );
    assert_eq!(
        battle.timers.press(BATTLE_TIME, Some(BATTLE)).unwrap(),
        "56:34"
    );
    battle.fake.visible(battle.root, false);
    battle.fake.layouts.refresh();
    assert_eq!(
        battle.timers.press(BATTLE_TIME, Some(BATTLE)).unwrap(),
        "no timer"
    );
}

/// Each second down a tick; 60, 10 and each of the last 5 said; a new turn, Command Time
/// shown again from 90, no tick, and its marks said afresh.
#[test]
fn test_command_time_ticks_every_second_and_is_said_at_its_marks() {
    let mut battle = Battle::new();

    fn polls(battle: &mut Battle, seconds: &[Option<i64>]) -> Vec<String> {
        let mut said: Vec<String> = Vec::new();
        for second in seconds {
            let shown = second.map(|second| second.to_string());
            battle.show("50:00", shown.as_deref());
            said.extend(
                battle
                    .timers
                    .poll(Some(BATTLE))
                    .unwrap()
                    .into_iter()
                    .map(|(_, line, _)| line),
            );
        }
        said
    }

    assert_eq!(
        polls(
            &mut battle,
            &[Some(62), Some(61), Some(60), Some(59), Some(58)]
        ),
        ["60"]
    );
    assert_eq!(battle.ticks.get(), 4);
    // Two seconds down is a tick a slow poll missed one of; 45 is no tick.
    assert_eq!(
        polls(
            &mut battle,
            &[Some(58), Some(56), Some(11), Some(10), Some(9)]
        ),
        ["10"]
    );
    assert_eq!(battle.ticks.get(), 7);
    assert_eq!(
        polls(&mut battle, &[None, Some(90), Some(89), Some(61), Some(60)]),
        ["60"]
    );
    assert_eq!(polls(&mut battle, &[Some(5), Some(4)]), ["4"]);
    // A battle from the field, as every battle of the tapes: nothing followed, though C asks.
    battle.timers = pss_battle::BattleTimers::with_before(
        battle.fake.layouts.clone(),
        Rc::new(|_root| None),
        Some({
            let ticks = battle.ticks.clone();
            Rc::new(move || ticks.set(ticks.get() + 1))
        }),
        Rc::new(|| Some(FIELD.to_string())),
    );
    let ticks = battle.ticks.get();
    assert!(polls(&mut battle, &[Some(62), Some(61), Some(60)]).is_empty());
    assert_eq!(battle.ticks.get(), ticks);
}

#[test]
fn test_the_keys_file_gives_the_timers_their_table() {
    assert_eq!(
        DEFAULT_CONTEXTS[PSS_BATTLE],
        IndexMap::from([
            ("c".to_string(), COMMAND_TIME.to_string()),
            ("shift+c".to_string(), BATTLE_TIME.to_string()),
        ])
    );
}
