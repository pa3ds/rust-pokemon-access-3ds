//! Screen announcing tests: what is said of screen text and cursors, and in what order.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use super::fake_speech::Recorder;
use crate::core::speech::{Category, Speech};
use crate::games::common::announce::{Announcements, FOCUS_HEARD_POLLS, Focus, SPEAKER_POLLS};
use crate::games::common::battle::Battler;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::dialogue::{PageShown, Shown};
use crate::games::common::screen::{ScreenPoll, ScreenText};
use crate::games::common::screen_announce::{
    ARRIVAL_MAX_POLLS, CURSOR_HEARD_POLLS, DETAIL_MAX_POLLS, DETAIL_POLLS, PLACE_POLLS,
    REPEAT_POLLS, ScreenAnnouncer,
};
use crate::py::{Log, Result};

const MENU: i64 = 0x08100000;
const PANEL: i64 = 0x08200000;
const OPTIONS: i64 = 0x08300000;
const SUMMARY: i64 = 0x08400000;

/// `[]`: what a poll said when it said nothing.
const NOTHING: [&str; 0] = [];

/// `piece`'s keyword arguments: `parent` (0 for the root), `with_layout` and `panel`.
#[derive(Clone, Copy)]
struct Kw {
    parent: i64,
    with_layout: bool,
    panel: i64,
}

const PLAIN: Kw = Kw {
    parent: 0,
    with_layout: true,
    panel: 0,
};

/// `piece(box, text, x, y, root, parent, with_layout, panel)`.
fn piece_kw(r#box: i64, text: &str, x: f64, y: f64, root: i64, kw: Kw) -> ScreenText {
    ScreenText {
        left: x,
        right: x + (text.chars().count() * 6) as f64,
        row: y,
        glyph: 12.5,
        root,
        parent: if kw.parent != 0 { kw.parent } else { root },
        with_layout: kw.with_layout,
        panel: kw.panel,
        ..ScreenText::new(r#box, text, false)
    }
}

fn piece(r#box: i64, text: &str, x: f64, y: f64, root: i64) -> ScreenText {
    piece_kw(r#box, text, x, y, root, PLAIN)
}

/// `replace(p, changed=True)`.
fn changed(p: &ScreenText) -> ScreenText {
    ScreenText {
        changed: true,
        ..p.clone()
    }
}

/// `replace(p, text=text, changed=True)`.
fn rewritten(p: &ScreenText, text: &str) -> ScreenText {
    ScreenText {
        text: text.to_string(),
        changed: true,
        ..p.clone()
    }
}

/// `{a, b}` of ints.
fn set(items: &[i64]) -> IndexSet<i64> {
    items.iter().copied().collect()
}

/// `say`'s arguments, each Python's default unless a test gives it.
#[derive(Default)]
struct Args {
    texts: Vec<ScreenText>,
    items: Vec<CursorItem>,
    on_screen: Vec<ScreenText>,
    labels: HashSet<String>,
    roots: Option<IndexSet<i64>>,
    settling: IndexSet<i64>,
    left: Vec<ScreenText>,
    moving: HashSet<i64>,
    rows: HashSet<i64>,
    subject: Option<Box<dyn Fn() -> Result<Option<String>>>>,
}

fn args(texts: &[ScreenText], items: Vec<CursorItem>, on_screen: &[ScreenText]) -> Args {
    Args {
        texts: texts.to_vec(),
        items,
        on_screen: on_screen.to_vec(),
        ..Args::default()
    }
}

fn labelled(words: &[&str]) -> HashSet<String> {
    words.iter().map(|word| word.to_string()).collect()
}

struct Screen {
    backend: Rc<Recorder>,
    announcer: ScreenAnnouncer,
}

impl Screen {
    fn new() -> Self {
        Screen::with(false)
    }

    fn with(pauses: bool) -> Self {
        Screen::with_clock(pauses, None, false)
    }

    /// `Screen(pauses, tick, timed)`.
    fn with_clock(pauses: bool, tick: Option<Rc<dyn Fn()>>, timed: bool) -> Self {
        let backend = Recorder::new();
        let announcer = ScreenAnnouncer::with_clock(
            Rc::new(Speech::plain(backend.clone())),
            Rc::new(|_line: &str| {}),
            pauses,
            tick,
            Some(Rc::new(move || timed)),
        );
        Screen { backend, announcer }
    }

    fn say(&mut self, given: Args) -> Vec<String> {
        let before = self.backend.spoken().len();
        let under: Vec<Vec<ScreenText>> =
            given.items.iter().map(|item| item.pieces.clone()).collect();
        self.announcer
            .say(
                &ScreenPoll {
                    texts: given.texts.clone(),
                    on_screen: given.on_screen.clone(),
                    left: given.left.clone(),
                    settling: given.settling.clone(),
                    message_lines: IndexSet::new(),
                },
                &CursorPoll {
                    items: given.items.clone(),
                    under,
                    lists: IndexMap::new(),
                    moving: given.moving.clone(),
                    rows: given.rows.clone(),
                },
                &given.labels,
                given.roots.as_ref(),
                given.subject.as_deref(),
            )
            .unwrap();
        self.backend.spoken()[before..].to_vec()
    }

    /// A poll with nothing in it.
    fn quiet(&mut self) -> Vec<String> {
        self.say(Args::default())
    }

    /// A poll with only what is on screen.
    fn on(&mut self, on_screen: &[ScreenText]) -> Vec<String> {
        self.say(Args {
            on_screen: on_screen.to_vec(),
            ..Args::default()
        })
    }

    /// A poll with only what is on screen and what is being written.
    fn on_settling(&mut self, on_screen: &[ScreenText], settling: &[i64]) -> Vec<String> {
        self.say(Args {
            on_screen: on_screen.to_vec(),
            settling: set(settling),
            ..Args::default()
        })
    }
}

#[test]
fn test_a_question_written_in_a_menu_while_no_cursor_moves_is_said() {
    // The Pokéblock Case asks "Give the Pokéblock to Pikachu?" in the words that asked which
    // Pokémon, under its list, as the player chooses one, on the row of its tabs (movie33). A
    // row's word that changes is not news so: going back to the case rewrote its counts.
    let mut screen = Screen::new();
    let pikachu = piece_kw(1, "Pikachu", -115.0, -15.0, MENU, Kw { parent: 4, ..PLAIN });
    let asked = piece_kw(
        2,
        "Which Pokémon would you like to give it to?",
        -155.0,
        105.0,
        MENU,
        Kw { parent: 5, ..PLAIN },
    );
    let tab = piece_kw(3, "Make", 62.0, 105.0, MENU, Kw { parent: 6, ..PLAIN });
    let count = piece_kw(4, "5", 66.0, 18.0, MENU, Kw { parent: 7, ..PLAIN });
    let everything = vec![pikachu.clone(), asked.clone(), tab.clone(), count.clone()];
    let rows = HashSet::from([pikachu.r#box]);
    screen.say(Args {
        rows: rows.clone(),
        ..args(
            &everything,
            vec![CursorItem::new(0x0C, vec![pikachu.clone()], true)],
            &everything,
        )
    });
    let give = ScreenText {
        text: "Give the Pokéblock to Pikachu?".to_string(),
        changed: true,
        with_layout: false,
        ..asked.clone()
    };
    let four = ScreenText {
        text: "4".to_string(),
        changed: true,
        with_layout: false,
        ..count.clone()
    };
    assert_eq!(
        screen.say(Args {
            rows,
            ..args(
                &[give.clone(), four.clone()],
                vec![],
                &[pikachu.clone(), give.clone(), tab.clone(), four.clone()]
            )
        }),
        [give.text.as_str()]
    );
}

#[test]
fn test_a_nameless_item_is_said_by_its_details() {
    // A contest's portraits draw no words, and the screen names the entrant elsewhere as the
    // cursor lands, "Pokémon: Bolt, Trainer: Camden" (movie33).
    let mut screen = Screen::new();
    let portrait = vec![ScreenText {
        root: MENU,
        parent: MENU,
        ..ScreenText::new(1, "", false)
    }];
    let names = vec![
        piece_kw(
            2,
            "Pokémon: Bolt",
            -140.0,
            -70.0,
            PANEL,
            Kw { parent: 9, ..PLAIN },
        ),
        piece_kw(
            3,
            "Trainer: Camden",
            40.0,
            -70.0,
            PANEL,
            Kw { parent: 9, ..PLAIN },
        ),
    ];
    let changes: Vec<ScreenText> = names.iter().map(changed).collect();
    let mut said = screen.say(args(
        &changes,
        vec![CursorItem::new(0x0C, portrait, false)],
        &names,
    ));
    for _ in 0..DETAIL_POLLS {
        said.extend(screen.on(&names));
    }
    assert_eq!(said, ["Pokémon: Bolt Trainer: Camden"]);
}

fn x_menu() -> Vec<ScreenText> {
    vec![
        piece(1, "POKÉMON", -70.0, 60.0, MENU),
        piece(2, "BAG", 90.0, 60.0, MENU),
        piece(3, "SAVE", 90.0, 0.0, MENU),
    ]
}

/// `[p]`.
fn one(p: &ScreenText) -> Vec<ScreenText> {
    vec![p.clone()]
}

/// `[*a, *b]`.
fn cat(parts: &[&[ScreenText]]) -> Vec<ScreenText> {
    parts.concat()
}

#[test]
fn test_a_menu_opening_says_only_the_item_under_its_cursor() {
    let mut screen = Screen::new();
    let menu = x_menu();
    let item = CursorItem::new(0x0C, vec![menu[1].clone()], true);
    assert_eq!(screen.say(args(&menu, vec![item], &menu)), ["BAG"]);
}

#[test]
fn test_a_menu_coming_back_without_its_cursor_says_nothing() {
    // Backing out of a menu read "POKÉMON BAG wee SAVE OPTIONS" (the user, 2026-09-14).
    let mut screen = Screen::new();
    let menu = x_menu();
    screen.say(args(
        &menu,
        vec![CursorItem::new(0x0C, vec![menu[1].clone()], true)],
        &menu,
    ));
    assert_eq!(screen.say(args(&menu, vec![], &menu)), NOTHING);
}

#[test]
fn test_a_menu_whose_layout_is_gone_is_forgotten() {
    let mut screen = Screen::new();
    let menu = x_menu();
    screen.say(args(
        &menu,
        vec![CursorItem::new(0x0C, vec![menu[1].clone()], true)],
        &menu,
    ));
    screen.say(Args {
        roots: Some(set(&[])),
        ..Args::default()
    });
    assert_eq!(screen.say(args(&menu, vec![], &menu)), ["POKÉMON BAG SAVE"]);
}

#[test]
fn test_words_that_come_later_in_a_menu_already_up_are_said() {
    // "Would you like to make these changes?" went unsaid once menus stayed menus (2026-09-14).
    let mut screen = Screen::new();
    let rows = vec![
        piece(1, "TEXT SPEED", -85.0, 60.0, OPTIONS),
        piece(2, "SLOW", 60.0, 60.0, OPTIONS),
    ];
    screen.say(args(
        &rows,
        vec![CursorItem::new(0x0C, rows.clone(), true)],
        &rows,
    ));
    let question = piece_kw(
        3,
        "Would you like to make these changes?",
        0.0,
        100.0,
        OPTIONS,
        Kw {
            with_layout: false,
            ..PLAIN
        },
    );
    assert_eq!(
        screen.say(args(
            &one(&question),
            vec![],
            &cat(&[&rows, &one(&question)])
        )),
        [question.text.as_str()]
    );
}

#[test]
fn test_a_screen_with_no_cursor_is_said_at_once() {
    let mut screen = Screen::new();
    let card = vec![
        piece(1, "Name:", -150.0, 80.0, PANEL),
        piece(2, "wee", -60.0, 80.0, PANEL),
    ];
    assert_eq!(screen.say(args(&card, vec![], &card)), ["Name: wee"]);
}

#[test]
fn test_a_panel_revealed_over_several_polls_is_said_whole_once_its_words_stop_arriving() {
    // The level up reveals its rows over half a second and writes each value a poll after its
    // label, and was heard as "HP", "+ 2 ATTACK + 0 DEFENSE SP. ATK SP. DEF", "+ 1 + 1 SPEED + 1"
    // (battles2.ctm, the user, 2026-09-17). Each row is a group of label and value (spike 68).
    let mut screen = Screen::new();
    let (hp, hp_value) = (
        piece(1, "HP", 59.0, 1074.0, PANEL),
        piece(2, "+ 2", 160.0, 1074.0, PANEL),
    );
    let (attack, attack_value) = (
        piece(3, "ATTACK", 59.0, 1054.0, PANEL),
        piece(4, "+ 2", 160.0, 1054.0, PANEL),
    );
    assert_eq!(
        screen.say(Args {
            settling: set(&[PANEL]),
            ..args(&one(&hp), vec![], &[hp.clone(), hp_value.clone()])
        }),
        NOTHING
    );
    let all = vec![
        hp.clone(),
        hp_value.clone(),
        attack.clone(),
        attack_value.clone(),
    ];
    assert_eq!(
        screen.say(Args {
            settling: set(&[PANEL]),
            ..args(&[hp_value.clone(), attack.clone()], vec![], &all)
        }),
        NOTHING
    );
    assert_eq!(
        screen.say(args(&one(&attack_value), vec![], &all)),
        ["HP + 2 ATTACK + 2"]
    );
}

#[test]
fn test_what_arrived_is_said_as_it_came_before_the_change_that_moves_it_on() {
    // Blaze's gains were on screen for under a second before its totals replaced them, and were
    // lost when what arrived waited for the panel to settle (battles2.ctm replay, 2026-09-17).
    let mut screen = Screen::new();
    let (hp, hp_value) = (
        piece(1, "HP", 59.0, 1074.0, PANEL),
        piece(2, "+ 2", 160.0, 1074.0, PANEL),
    );
    let speed = piece(3, "SPEED", 83.0, 977.0, PANEL);
    assert_eq!(
        screen.say(Args {
            settling: set(&[PANEL]),
            ..args(
                &[hp.clone(), hp_value.clone()],
                vec![],
                &[hp.clone(), hp_value.clone(), speed.clone()]
            )
        }),
        NOTHING
    );
    let total = rewritten(&hp_value, "23");
    assert_eq!(
        screen.say(Args {
            settling: set(&[PANEL]),
            ..args(
                &one(&total),
                vec![],
                &[hp.clone(), total.clone(), speed.clone()]
            )
        }),
        ["HP + 2", "HP 23"]
    );
}

#[test]
fn test_a_letter_deleted_from_a_name_says_the_letters_left() {
    // opening.ctm: deleting the q from "Raiq" said nothing at all (the user, 2026-09-18).
    let mut screen = Screen::new();
    let name = 0x30;
    let letters: Vec<ScreenText> = "Raiq"
        .chars()
        .enumerate()
        .map(|(index, letter)| {
            piece_kw(
                index as i64,
                &letter.to_string(),
                -90.0 + 18.0 * index as f64,
                100.0,
                PANEL,
                Kw {
                    parent: name,
                    ..PLAIN
                },
            )
        })
        .collect();
    screen.say(args(&letters, vec![], &letters));
    assert_eq!(
        screen.say(Args {
            on_screen: letters[..3].to_vec(),
            left: vec![letters[3].clone()],
            ..Args::default()
        }),
        ["R a i"]
    );
}

#[test]
fn test_a_word_leaving_a_screen_that_is_not_a_name_says_nothing() {
    let mut screen = Screen::new();
    let kw = Kw {
        parent: 0x30,
        ..PLAIN
    };
    let row = vec![
        piece_kw(1, "TEXT SPEED", -85.0, 60.0, OPTIONS, kw),
        piece_kw(2, "SLOW", 60.0, 60.0, OPTIONS, kw),
    ];
    screen.say(args(&row, vec![], &row));
    assert_eq!(
        screen.say(Args {
            on_screen: row[..1].to_vec(),
            left: vec![row[1].clone()],
            ..Args::default()
        }),
        NOTHING
    );
}

#[test]
fn test_a_value_arriving_beside_its_label_already_said_is_said_with_it() {
    let mut screen = Screen::new();
    let label = piece_kw(
        1,
        "SP. DEF",
        59.0,
        997.0,
        PANEL,
        Kw {
            parent: 0x10,
            ..PLAIN
        },
    );
    screen.say(args(&one(&label), vec![], &one(&label)));
    let value = piece_kw(
        2,
        "12",
        168.0,
        997.0,
        PANEL,
        Kw {
            parent: 0x10,
            with_layout: false,
            ..PLAIN
        },
    );
    assert_eq!(
        screen.say(args(&one(&value), vec![], &[label.clone(), value.clone()])),
        ["SP. DEF 12"]
    );
}

#[test]
fn test_a_letter_typed_into_a_name_is_said_alone() {
    // opening.ctm: typing "raiden" said "R a", "R a i", "R a i d", the name so far each time.
    let mut screen = Screen::new();
    let name = 0x30;
    let mut typed = vec![piece_kw(
        1,
        "R",
        -90.0,
        100.0,
        PANEL,
        Kw {
            parent: name,
            ..PLAIN
        },
    )];
    screen.say(args(&typed, vec![], &typed));
    for (index, letter) in "ai".chars().enumerate().map(|(i, c)| (i as i64 + 2, c)) {
        let new = piece_kw(
            index,
            &letter.to_string(),
            -90.0 + 18.0 * (index - 1) as f64,
            100.0,
            PANEL,
            Kw {
                parent: name,
                with_layout: false,
                ..PLAIN
            },
        );
        typed.push(new.clone());
        assert_eq!(
            screen.say(args(&[new], vec![], &typed)),
            [letter.to_string().as_str()]
        );
    }
}

#[test]
fn test_a_letter_typed_from_a_keyboard_is_said_though_its_key_was_just_said() {
    // X's name entry: the keyboard's cursor says each key, and the letter it types was left out
    // as words the cursor had just said, alone, "R", and in the name read with the next move,
    // "R a" for "R a i" (x_movie1_opening, #12).
    let mut screen = Screen::new();
    let name = 0x30;
    let key = |letter: &str| vec![piece(10, letter, 0.0, -60.0, MENU)];
    let moved = |letter: &str| vec![CursorItem::new(0x0C, key(letter), false)];
    assert_eq!(screen.say(args(&[], moved("R"), &key("R"))), ["R"]);
    let mut typed = vec![piece_kw(
        1,
        "R",
        -90.0,
        100.0,
        PANEL,
        Kw {
            parent: name,
            ..PLAIN
        },
    )];
    assert_eq!(
        screen.say(args(&typed, moved("r"), &cat(&[&key("r"), &typed]))),
        ["r", "R"]
    );
    for (index, letter) in ["a", "i"].into_iter().enumerate() {
        let index = index as i64 + 2;
        let mut heard = screen.say(args(&[], moved(letter), &cat(&[&key(letter), &typed])));
        let new = piece_kw(
            index,
            letter,
            -90.0 + 18.0 * (index - 1) as f64,
            100.0,
            PANEL,
            Kw {
                parent: name,
                with_layout: false,
                ..PLAIN
            },
        );
        typed.push(new.clone());
        heard.extend(screen.say(args(&[new], vec![], &cat(&[&key(letter), &typed]))));
        for _ in 0..DETAIL_POLLS {
            heard.extend(screen.on(&cat(&[&key(letter), &typed])));
        }
        let name: Vec<&str> = typed.iter().map(|p| p.text.as_str()).collect();
        assert_eq!(heard, [letter.to_string(), name.join(" ")], "{letter}");
    }
}

#[test]
fn test_words_arriving_in_a_screen_up_a_while_are_not_held() {
    // movie3.ctm's name entry said "w e e" for two quick letters, held while the name box, up for
    // seconds, still had words being written.
    let mut screen = Screen::new();
    let title = piece(1, "Your name?", 0.0, 100.0, PANEL);
    for _ in 0..ARRIVAL_MAX_POLLS {
        screen.say(args(&one(&title), vec![], &one(&title)));
    }
    let letter = piece_kw(
        2,
        "w",
        -90.0,
        60.0,
        PANEL,
        Kw {
            parent: 0x30,
            with_layout: false,
            ..PLAIN
        },
    );
    assert_eq!(
        screen.say(Args {
            settling: set(&[PANEL]),
            ..args(&one(&letter), vec![], &[title.clone(), letter.clone()])
        }),
        ["w"]
    );
}

#[test]
fn test_words_arriving_with_a_change_follow_it() {
    let mut screen = Screen::new();
    let (hp, hp_value) = (
        piece_kw(
            1,
            "HP",
            59.0,
            1074.0,
            PANEL,
            Kw {
                parent: 0x10,
                ..PLAIN
            },
        ),
        piece_kw(
            2,
            "+ 2",
            160.0,
            1074.0,
            PANEL,
            Kw {
                parent: 0x10,
                ..PLAIN
            },
        ),
    );
    let speed = piece_kw(
        3,
        "SPEED",
        59.0,
        977.0,
        PANEL,
        Kw {
            parent: 0x20,
            ..PLAIN
        },
    );
    let first = vec![hp.clone(), hp_value.clone(), speed.clone()];
    screen.say(Args {
        settling: set(&[PANEL]),
        ..args(&first, vec![], &first)
    });
    let total = rewritten(&hp_value, "23");
    let speed_value = piece_kw(
        4,
        "12",
        168.0,
        977.0,
        PANEL,
        Kw {
            parent: 0x20,
            with_layout: false,
            ..PLAIN
        },
    );
    assert_eq!(
        screen.say(args(
            &[total.clone(), speed_value.clone()],
            vec![],
            &[
                hp.clone(),
                total.clone(),
                speed.clone(),
                speed_value.clone()
            ]
        )),
        ["HP + 2 SPEED", "HP 23", "SPEED 12"]
    );
}

#[test]
fn test_words_still_arriving_are_waited_for_no_longer_than_their_most_polls() {
    let mut screen = Screen::new();
    let clock = piece(1, "Time: 0:30", 0.0, 0.0, PANEL);
    let mut said: Vec<String> = Vec::new();
    for _ in 0..ARRIVAL_MAX_POLLS {
        let texts = if said.is_empty() {
            vec![clock.clone()]
        } else {
            vec![]
        };
        said.extend(screen.say(Args {
            settling: set(&[PANEL]),
            ..args(&texts, vec![], &one(&clock))
        }));
    }
    assert_eq!(said, ["Time: 0:30"]);
}

#[test]
fn test_what_comes_on_screen_is_read_in_place_a_panel_at_a_time() {
    // The battle's move list was read "Scratch PP 3535 /" in build order (spike 46).
    //
    // Two layouts are two things to say. Joining every layout that arrived in one poll into a
    // single run is what put a move list and a level in one breath.
    let mut screen = Screen::new();
    let moves = vec![
        piece(1, "Scratch", 50.0, 100.0, PANEL),
        piece(2, "PP", 40.0, 80.0, PANEL),
        piece(3, "/", 99.0, 80.0, PANEL),
        piece(4, "35", 80.0, 80.0, PANEL),
        piece(5, "35", 108.0, 80.0, PANEL),
    ];
    let level = vec![piece(6, "5", -150.0, 100.0, OPTIONS)];
    let all = cat(&[&moves, &level]);
    assert_eq!(
        screen.say(args(&all, vec![], &all)),
        ["Scratch PP 35/35", "5"]
    );
}

#[test]
fn test_two_panels_of_one_layout_are_two_things_to_say() {
    // The battle draws both Pokemon's HP boxes in one layout, one pane under the root each
    // (spike 53). Read as a layout it said "6 22/ 22 2": Torchic at level 6 with 22/22 HP, and the
    // wild Pokemon's level 2 on the end as though it were part of the same row.
    let mut screen = Screen::new();
    let panelled = |panel| Kw { panel, ..PLAIN };
    let mine = vec![
        piece_kw(1, "6", -76.0, 906.0, PANEL, panelled(0x100)),
        piece_kw(2, "22/ 22", -84.0, 888.0, PANEL, panelled(0x100)),
    ];
    let theirs = vec![piece_kw(3, "2", 174.0, 1108.0, PANEL, panelled(0x200))];
    let all = cat(&[&mine, &theirs]);
    assert_eq!(screen.say(args(&all, vec![], &all)), ["6 22/ 22", "2"]);
}

#[test]
fn test_a_label_and_its_value_stay_together_across_panels() {
    // A panel boundary only breaks an utterance where a row boundary does too. Splitting on the
    // panel alone read the level up's "ATTACK" and "13" as two things, and a move's "PP 25/" and
    // "25" likewise: a screen can draw a label and its number in different panels and still put
    // them side by side on one row.
    let mut screen = Screen::new();
    let row = vec![
        piece_kw(
            1,
            "ATTACK",
            -150.0,
            60.0,
            PANEL,
            Kw {
                panel: 0x100,
                ..PLAIN
            },
        ),
        piece_kw(
            2,
            "13",
            -40.0,
            60.0,
            PANEL,
            Kw {
                panel: 0x200,
                ..PLAIN
            },
        ),
    ];
    assert_eq!(screen.say(args(&row, vec![], &row)), ["ATTACK 13"]);
}

#[test]
fn test_two_columns_built_one_after_the_other_are_not_interleaved() {
    // Reading the summary row by row read "Dex No. 004 HP 19 19 Name Torchic Attack 11" (spike
    // 47).
    let mut screen = Screen::new();
    let right = vec![
        piece(1, "HP", 50.0, 85.0, SUMMARY),
        piece(2, "19", 110.0, 85.0, SUMMARY),
        piece(3, "Attack", 55.0, 50.0, SUMMARY),
        piece(4, "11", 130.0, 50.0, SUMMARY),
    ];
    let left = vec![
        piece(5, "Name", -165.0, 60.0, SUMMARY),
        piece(6, "Torchic", -80.0, 60.0, SUMMARY),
        piece(7, "Dex No.", -155.0, 80.0, SUMMARY),
        piece(8, "004", -45.0, 80.0, SUMMARY),
    ];
    let all = cat(&[&right, &left]);
    let said = screen.say(args(&all, vec![], &all));
    assert_eq!(said, ["HP 19 Attack 11 Name Torchic Dex No. 004"]);
}

#[test]
fn test_a_row_of_pieces_each_a_panel_of_its_own_is_read_across() {
    // The summary hangs every piece from its root, each a panel of its own, and builds HP's "/"
    // before its two numbers: read a panel at a time, as columns, it was "HP / 30 30" (movie7,
    // 2026-09-30). Places are spike 148's.
    let pieces: Vec<ScreenText> = [
        ("HP", 30.0, 42.0, 12.5),
        ("/", 126.0, 134.0, 14.0),
        ("30", 103.0, 117.0, 14.0),
        ("30", 143.0, 157.0, 14.0),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (text, left, right, glyph))| {
        let r#box = index as i64 + 1;
        ScreenText {
            right,
            glyph,
            ..piece_kw(
                r#box,
                text,
                left,
                85.0,
                SUMMARY,
                Kw {
                    panel: r#box,
                    ..PLAIN
                },
            )
        }
    })
    .collect();
    assert_eq!(
        Screen::new().say(args(&pieces, vec![], &pieces)),
        ["HP 30/30"]
    );
}

#[test]
fn test_part_of_a_screen_coming_back_brings_back_the_rest_of_it() {
    // Returning from the move list read only "OT wee ID No. …" of the summary, whose stats stay on
    // screen beside the move panel (spike 48).
    let mut screen = Screen::new();
    let stats = vec![
        piece(1, "HP", 50.0, 85.0, SUMMARY),
        piece(2, "20", 110.0, 85.0, SUMMARY),
    ];
    let alone = Kw {
        with_layout: false,
        ..PLAIN
    };
    let back = vec![
        ScreenText {
            returned: true,
            ..piece_kw(3, "OT", -150.0, -40.0, SUMMARY, alone)
        },
        ScreenText {
            returned: true,
            ..piece_kw(4, "wee", -60.0, -40.0, SUMMARY, alone)
        },
    ];
    assert_eq!(
        screen.say(args(&back, vec![], &cat(&[&stats, &back]))),
        ["HP 20 OT wee"]
    );
}

#[test]
fn test_words_new_to_a_screen_already_up_come_alone() {
    let mut screen = Screen::new();
    let stats = vec![
        piece(1, "HP", 50.0, 85.0, SUMMARY),
        piece(2, "20", 110.0, 85.0, SUMMARY),
    ];
    let new = piece_kw(
        3,
        "Hard claws.",
        -150.0,
        -40.0,
        SUMMARY,
        Kw {
            with_layout: false,
            ..PLAIN
        },
    );
    assert_eq!(
        screen.say(args(&one(&new), vec![], &cat(&[&stats, &one(&new)]))),
        ["Hard claws."]
    );
}

#[test]
fn test_words_coming_back_to_a_menu_come_alone() {
    let mut screen = Screen::new();
    let rows = vec![
        piece(1, "TEXT SPEED", -85.0, 60.0, OPTIONS),
        piece(2, "SLOW", 60.0, 60.0, OPTIONS),
    ];
    screen.say(args(
        &rows,
        vec![CursorItem::new(0x0C, rows.clone(), true)],
        &rows,
    ));
    let question = ScreenText {
        returned: true,
        ..piece_kw(
            3,
            "Would you like to make these changes?",
            0.0,
            100.0,
            OPTIONS,
            Kw {
                with_layout: false,
                ..PLAIN
            },
        )
    };
    assert_eq!(
        screen.say(args(
            &one(&question),
            vec![],
            &cat(&[&rows, &one(&question)])
        )),
        [question.text.as_str()]
    );
}

#[test]
fn test_words_coming_to_a_menu_in_place_of_words_just_gone_are_its_own() {
    // The Bag's party draws "ABLE!" or "UNABLE!" for a TM or a stone in place of each level,
    // and the levels again as the cursor moves on, and both were read at every pocket (movie31,
    // Khai, 2026-10-18). A question in their place is said.
    let mut screen = Screen::new();
    let stone = vec![piece(1, "Sun Stone", 77.0, 0.0, MENU)];
    let level = piece_kw(3, "L45", -91.0, 68.0, MENU, Kw { parent: 5, ..PLAIN });
    screen.say(args(
        &cat(&[&stone, &one(&level)]),
        vec![CursorItem::new(0x0C, stone.clone(), true)],
        &cat(&[&stone, &one(&level)]),
    ));
    let unable = piece_kw(
        2,
        "UNABLE!",
        -91.0,
        68.0,
        MENU,
        Kw {
            parent: 5,
            with_layout: false,
            ..PLAIN
        },
    );
    assert_eq!(
        screen.say(Args {
            left: vec![level.clone()],
            ..args(&one(&unable), vec![], &cat(&[&stone, &one(&unable)]))
        }),
        NOTHING
    );
    let back = ScreenText {
        with_layout: false,
        ..level.clone()
    };
    assert_eq!(
        screen.say(Args {
            left: vec![unable.clone()],
            ..args(&one(&back), vec![], &cat(&[&stone, &one(&back)]))
        }),
        NOTHING
    );
    // A paralysed Pokémon's slot shows no level, and its "UNABLE!" comes as another's leaves.
    let paralysed = piece_kw(
        5,
        "UNABLE!",
        -91.0,
        35.0,
        MENU,
        Kw {
            parent: 6,
            with_layout: false,
            ..PLAIN
        },
    );
    assert_eq!(
        screen.say(Args {
            left: vec![back.clone()],
            ..args(
                &[unable.clone(), paralysed.clone()],
                vec![],
                &cat(&[&stone, &[unable.clone(), paralysed.clone()]])
            )
        }),
        NOTHING
    );
    let asked = piece_kw(
        4,
        "Use it on which Pokémon now?",
        -91.0,
        68.0,
        MENU,
        Kw {
            parent: 5,
            with_layout: false,
            ..PLAIN
        },
    );
    assert_eq!(
        screen.say(Args {
            left: vec![unable.clone()],
            ..args(&one(&asked), vec![], &cat(&[&stone, &one(&asked)]))
        }),
        [asked.text.as_str()]
    );
}

/// X's language select draws its question in a panel of its own where its list was,
/// "English" where the list's "English" was, and the language went unsaid as the menu's own
/// words come back: "Play Pokémon X in" (x_movie1_opening).
#[test]
fn test_words_coming_to_a_panel_no_words_left_are_said() {
    let mut screen = Screen::new();
    let rows: Vec<ScreenText> = ["English", "Spanish", "French"]
        .iter()
        .enumerate()
        .map(|(at, language)| {
            let at = at as i64;
            piece_kw(
                10 + at,
                language,
                -145.0,
                (-10 - 30 * at) as f64,
                MENU,
                Kw {
                    parent: 0x20 + at,
                    panel: 0x10,
                    ..PLAIN
                },
            )
        })
        .collect();
    screen.say(args(
        &rows,
        vec![CursorItem::new(0x0C, vec![rows[0].clone()], true)],
        &rows,
    ));
    let alone = |r#box: i64, text: &str, y: f64, parent: i64| {
        piece_kw(
            r#box,
            text,
            -145.0,
            y,
            MENU,
            Kw {
                parent,
                with_layout: false,
                panel: 0x40,
            },
        )
    };
    let question = alone(1, "Play Pokémon X in", 15.0, 0x30);
    let language = alone(2, "English", -10.0, 0x40);
    let begin = vec![alone(3, "Begin game", -55.0, 0x32)];
    let shown = cat(&[&[question.clone(), language.clone()], &begin]);
    assert_eq!(
        screen.say(Args {
            left: rows.clone(),
            ..args(
                &shown,
                vec![CursorItem::new(0x0C, begin.clone(), false)],
                &shown
            )
        }),
        ["Begin game", "Play Pokémon X in English"]
    );
}

#[test]
fn test_a_place_revealed_after_its_entrant_is_said_with_it() {
    // The contest's results reveal each place a while after the entrants' names, and said alone
    // it was "third place", nobody's (movie30, Orion, 2026-10-09).
    let mut screen = Screen::new();
    let names = piece_kw(
        1,
        "Raiden & Metagross",
        -75.0,
        -45.0,
        PANEL,
        Kw { parent: 7, ..PLAIN },
    );
    screen.say(args(&one(&names), vec![], &one(&names)));
    for _ in 0..ARRIVAL_MAX_POLLS {
        screen.on(&one(&names));
    }
    let place = ScreenText {
        root: PANEL,
        parent: 7,
        picture: true,
        ..ScreenText::placed(
            2,
            "third place",
            false,
            names.right + 2.5 * 12.5,
            names.right + 3.5 * 12.5,
            -45.0,
            12.5,
        )
    };
    let said = screen.say(args(
        &one(&place),
        vec![],
        &cat(&[&one(&names), &one(&place)]),
    ));
    assert_eq!(said, ["Raiden & Metagross third place"]);
}

#[test]
fn test_a_changed_value_is_said_with_its_row() {
    let mut screen = Screen::new();
    let parent = |parent| Kw { parent, ..PLAIN };
    let rows = [
        piece_kw(1, "TEXT SPEED", -85.0, 60.0, OPTIONS, parent(7)),
        piece_kw(2, "SLOW", 60.0, 60.0, OPTIONS, parent(7)),
        piece_kw(3, "BATTLE EFFECTS", -85.0, 30.0, OPTIONS, parent(8)),
    ];
    let fast = rewritten(&rows[1], "FAST");
    let on_screen = vec![rows[0].clone(), fast.clone(), rows[2].clone()];
    assert_eq!(
        screen.say(args(&one(&fast), vec![], &on_screen)),
        ["TEXT SPEED FAST"]
    );
}

#[test]
fn test_a_changed_value_takes_its_row_from_its_own_group() {
    // Scratch's power was read "Power 40 Sp. Def 11", a stat from the page behind it.
    let mut screen = Screen::new();
    let power = vec![
        piece_kw(
            1,
            "Power",
            -200.0,
            -10.0,
            SUMMARY,
            Kw { parent: 5, ..PLAIN },
        ),
        changed(&piece_kw(
            2,
            "40",
            -120.0,
            -10.0,
            SUMMARY,
            Kw { parent: 5, ..PLAIN },
        )),
    ];
    let stat = vec![
        piece(3, "Sp. Def", 40.0, -10.0, SUMMARY),
        piece(4, "11", 125.0, -10.0, SUMMARY),
    ];
    assert_eq!(
        screen.say(args(&one(&power[1]), vec![], &cat(&[&power, &stat]))),
        ["Power 40"]
    );
}

#[test]
fn test_a_changed_value_alone_in_its_group_takes_its_whole_row() {
    let mut screen = Screen::new();
    let label = piece_kw(
        1,
        "Play time:",
        -150.0,
        0.0,
        PANEL,
        Kw { parent: 5, ..PLAIN },
    );
    let minutes = changed(&piece_kw(
        2,
        "14",
        -40.0,
        0.0,
        PANEL,
        Kw { parent: 6, ..PLAIN },
    ));
    assert_eq!(
        screen.say(args(
            &one(&minutes),
            vec![],
            &[label.clone(), minutes.clone()]
        )),
        ["Play time: 14"]
    );
}

/// What is said of a clock shown with its first words and changed to each of the rest, a
/// change every `polls` polls, or after each of them.
///
/// Python's `polls` is an int or a list; here it is always a list, `vec![5; n]` for the int.
fn run_clock(screen: &mut Screen, texts: &[String], polls: &[usize]) -> Vec<String> {
    let mut first = piece(1, &texts[0], -100.0, 100.0, PANEL);
    let mut said = screen.say(args(&one(&first), vec![], &one(&first)));
    assert_eq!(polls.len(), texts.len() - 1);
    for (text, &gap) in texts[1..].iter().zip(polls) {
        for _ in 0..gap - 1 {
            said.extend(screen.on(&one(&first)));
        }
        first = ScreenText {
            text: text.clone(),
            changed: true,
            with_layout: false,
            ..first.clone()
        };
        said.extend(screen.say(args(&one(&first), vec![], &one(&first))));
    }
    said
}

/// `run_clock` with a change every 5 polls, its default.
fn run_clock_5(screen: &mut Screen, texts: &[String]) -> Vec<String> {
    run_clock(screen, texts, &vec![5; texts.len() - 1])
}

fn words(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|text| text.to_string()).collect()
}

#[test]
fn test_a_countdown_is_said_each_half_minute_at_10_and_its_last_5() {
    // Berry Picker said "Time Left" every second for its whole minute (a player, 2026-10-26).
    let ticks = Rc::new(RefCell::new(Vec::<i64>::new()));
    let counting = ticks.clone();
    let mut screen = Screen::with_clock(
        false,
        Some(Rc::new(move || counting.borrow_mut().push(1))),
        true,
    );
    let countdown: Vec<String> = (0..=60)
        .rev()
        .map(|left| format!("Time Left: {left:02}"))
        .collect();
    let said = run_clock_5(&mut screen, &countdown);
    assert_eq!(
        said,
        [
            "Time Left: 60",
            "Time Left: 30",
            "Time Left: 10",
            "Time Left: 05",
            "Time Left: 04",
            "Time Left: 03",
            "Time Left: 02",
            "Time Left: 01"
        ]
    );
    assert_eq!(ticks.borrow().len(), 60);
}

#[test]
fn test_a_clock_counting_up_is_said_each_half_minute() {
    let up: Vec<String> = (28..33)
        .map(|seconds| format!("Time: 00:{seconds:02}"))
        .collect();
    let said = run_clock_5(&mut Screen::with_clock(false, None, true), &up);
    assert_eq!(said, ["Time: 00:28", "Time: 00:30"]);
}

#[test]
fn test_a_clock_still_ticks_past_a_missed_second_and_after_a_late_one() {
    // Berry Picker's replay read 41 then 39 two seconds later, and 10 then 9 two polls after
    // (as_amie_berry_picker); 31 to 29 passes the half minute, which is said.
    let said = run_clock(
        &mut Screen::with_clock(false, None, true),
        &words(&[
            "Time Left: 42",
            "Time Left: 41",
            "Time Left: 39",
            "Time Left: 38",
            "Time Left: 37",
            "Time Left: 36",
        ]),
        &[5, 10, 5, 7, 2],
    );
    assert_eq!(said, ["Time Left: 42"]);
    let said = run_clock(
        &mut Screen::with_clock(false, None, true),
        &words(&[
            "Time Left: 33",
            "Time Left: 32",
            "Time Left: 31",
            "Time Left: 29",
        ]),
        &[5, 5, 10],
    );
    assert_eq!(said, ["Time Left: 33", "Time Left: 29"]);
}

#[test]
fn test_no_number_is_a_clock_outside_a_game_that_times_the_player() {
    // Taken for a clock wherever it ticked, the battle Bag's page, the Mart's counts, money, a
    // save screen's minute and the PC Box's name went unsaid as the player changed them
    // (as_cave_of_origin, or_mart, or_pc_box_buttons and others).
    let countdown: Vec<String> = (56..=60)
        .rev()
        .map(|left| format!("Time Left: {left}"))
        .collect();
    assert_eq!(run_clock_5(&mut Screen::new(), &countdown), countdown);
}

#[test]
fn test_a_number_changing_by_one_off_a_seconds_beat_is_said() {
    // A score going up by one a poll after it came, or a play time a minute after, is no clock.
    assert_eq!(
        run_clock(
            &mut Screen::with_clock(false, None, true),
            &words(&["SCORE 7", "SCORE 8"]),
            &[1]
        ),
        ["SCORE 7", "SCORE 8"]
    );
    assert_eq!(
        run_clock(
            &mut Screen::with_clock(false, None, true),
            &words(&["Play time: 14", "Play time: 15"]),
            &[300]
        ),
        ["Play time: 14", "Play time: 15"]
    );
}

/// A move's detail panel: labels and values under one pane, as the summary builds it.
fn move_panel(power: &str) -> Vec<ScreenText> {
    let under = Kw { parent: 5, ..PLAIN };
    vec![
        piece_kw(10, "Category", -210.0, 13.0, SUMMARY, under),
        piece_kw(11, "Power", -210.0, -10.0, SUMMARY, under),
        piece_kw(12, "Accuracy", -210.0, -33.0, SUMMARY, under),
        piece_kw(13, power, -120.0, -10.0, SUMMARY, under),
        piece_kw(14, "100", -120.0, -33.0, SUMMARY, under),
        piece(15, "Sp. Def", 40.0, -10.0, SUMMARY),
    ]
}

#[test]
fn test_details_that_change_with_the_cursor_are_said_whole_once_settled() {
    // Arrowing between moves read only "Power 40": the three moves share an accuracy of 100, so it
    // never changed, and the user heard it sometimes and not others (2026-09-14).
    let mut screen = Screen::new();
    let growl = vec![
        piece(1, "Growl", -120.0, 70.0, MENU),
        piece(2, "PP", -130.0, 50.0, MENU),
    ];
    let panel = move_panel("40");
    let power = changed(&panel[3]);
    let everything = cat(&[&growl, &panel]);
    let said = screen.say(args(
        &[power],
        vec![CursorItem::new(0x0C, growl.clone(), false)],
        &everything,
    ));
    assert_eq!(said, ["Growl PP"]);
    assert_eq!(screen.on(&everything), NOTHING);
    assert_eq!(screen.on(&everything), ["Category Power 40 Accuracy 100"]);
}

/// With menus set to interrupt, a move's power cut off the move it belonged to, and the Mart's
/// description its count in the bag (Khai, 2026-10-04): the item interrupts, its details queue.
#[test]
fn test_details_are_one_line_that_never_cuts_off_the_item() {
    let mut screen = Screen::new();
    screen.announcer = ScreenAnnouncer::new(
        Rc::new(Speech::new(
            screen.backend.clone(),
            HashSet::from([Category::Key, Category::Menu]),
            None,
        )),
        Rc::new(|_line: &str| {}),
    );
    let growl = vec![piece(1, "Growl", -120.0, 70.0, MENU)];
    let panel = move_panel("40");
    let words = description("The user growls.");
    let everything = cat(&[&growl, &one(&words), &panel]);
    screen.say(args(
        &[changed(&words), changed(&panel[3])],
        vec![CursorItem::new(0x0C, growl.clone(), false)],
        &everything,
    ));
    for _ in 0..DETAIL_POLLS {
        screen.on(&everything);
    }
    assert_eq!(
        screen.backend.spoken(),
        ["Growl", "The user growls. Category Power 40 Accuracy 100"]
    );
    assert_eq!(screen.backend.interrupting(), ["Growl"]);
}

#[test]
fn test_a_count_alone_under_a_cursor_is_said_after_what_names_it() {
    // The Berry Blender's cursor sits on a berry's picture and its count, and the top screen
    // names the berry: "16", then "Cheri Berry Red" and its description, was all numbers and
    // colours (movie30, Orion, 2026-10-08).
    let mut screen = Screen::new();
    let grouped = Kw {
        parent: 0x200,
        ..PLAIN
    };
    let count = vec![piece(1, "16", -118.0, 45.0, MENU)];
    let name = vec![
        piece_kw(2, "Cheri Berry", -190.0, -35.0, PANEL, grouped),
        piece_kw(3, "Red", -14.0, -35.0, PANEL, grouped),
    ];
    let words = piece_kw(
        4,
        "A Berry to be consumed by Pokémon.",
        -180.0,
        -83.0,
        PANEL,
        grouped,
    );
    let everything = cat(&[&count, &name, &one(&words)]);
    let mut said = screen.say(args(
        &[changed(&name[0])],
        vec![CursorItem::new(0x0C, count.clone(), false)],
        &everything,
    ));
    assert_eq!(said, NOTHING);
    for _ in 0..DETAIL_POLLS {
        said.extend(screen.on(&everything));
    }
    assert_eq!(
        said,
        ["Cheri Berry Red 16, A Berry to be consumed by Pokémon."]
    );
}

#[test]
fn test_a_cursor_on_its_way_elsewhere_drops_its_details_unsaid() {
    // The Box's pointer is held two polls on each slot before it is said, and Party 1's details
    // were read once the summary showed Party 2's Pokémon, the pointer already there (spike 129).
    let mut screen = Screen::new();
    let slot = vec![piece(1, "Party 1", -120.0, 70.0, MENU)];
    let panel = move_panel("40");
    let power = changed(&panel[3]);
    let everything = cat(&[&slot, &panel]);
    screen.say(args(
        &[power],
        vec![CursorItem::new(0x0C, slot.clone(), false)],
        &everything,
    ));
    assert_eq!(
        screen.say(Args {
            moving: HashSet::from([0x0C]),
            ..args(&[], vec![], &everything)
        }),
        NOTHING
    );
    assert_eq!(screen.on(&everything), NOTHING);
    assert_eq!(screen.on(&everything), NOTHING);
}

#[test]
fn test_a_quiet_items_details_are_taken_in_unsaid() {
    // An empty slot of the PC Box, over which the summary keeps showing the Pokémon carried:
    // "where, then empty" (the user, 2026-09-27).
    let mut screen = Screen::new();
    let slot = vec![piece(1, "Party 3, empty", -120.0, 70.0, MENU)];
    let panel = move_panel("40");
    let power = changed(&panel[3]);
    let everything = cat(&[&slot, &panel]);
    let said = screen.say(args(
        &[power],
        vec![CursorItem {
            quiet: true,
            ..CursorItem::new(0x0C, slot.clone(), false)
        }],
        &everything,
    ));
    assert_eq!(said, ["Party 3, empty"]);
    assert_eq!(screen.on(&everything), NOTHING);
    assert_eq!(screen.on(&everything), NOTHING);
}

#[test]
fn test_details_that_change_a_poll_after_the_cursor_still_follow_it() {
    let mut screen = Screen::new();
    let growl = vec![piece(1, "Growl", -120.0, 70.0, MENU)];
    screen.say(args(
        &[],
        vec![CursorItem::new(0x0C, growl.clone(), false)],
        &growl,
    ));
    let panel = move_panel("—");
    let power = changed(&panel[3]);
    assert_eq!(screen.say(args(&[power], vec![], &panel)), NOTHING);
    assert_eq!(screen.on(&panel), ["Category Power — Accuracy 100"]);
}

/// A move's description, built before its panel and hanging from a pane of its own.
fn description(text: &str) -> ScreenText {
    piece_kw(9, text, -150.0, 45.0, SUMMARY, Kw { parent: 6, ..PLAIN })
}

#[test]
fn test_details_are_said_in_the_order_the_screen_built_them_however_they_changed() {
    // The description came a poll after the panel, and was read after it where every other move
    // read it first (the user, 2026-09-14).
    let mut screen = Screen::new();
    let growl = vec![piece(1, "Growl", -120.0, 70.0, MENU)];
    let panel = move_panel("—");
    let words = description("The user growls.");
    let everything = cat(&[&growl, &one(&words), &panel]);
    screen.say(args(
        &[changed(&panel[3])],
        vec![CursorItem::new(0x0C, growl.clone(), false)],
        &everything,
    ));
    assert_eq!(screen.on_settling(&everything, &[SUMMARY]), NOTHING);
    assert_eq!(screen.on_settling(&everything, &[SUMMARY]), NOTHING);
    assert_eq!(
        screen.say(args(&[changed(&words)], vec![], &everything)),
        ["The user growls. Category Power — Accuracy 100"]
    );
}

#[test]
fn test_a_moves_details_read_every_panel_its_moves_have_changed() {
    // Ember to Scratch changes only the description, and power and accuracy went unsaid.
    let mut screen = Screen::new();
    let first = vec![piece(1, "Ember", -120.0, 70.0, MENU)];
    let panel = move_panel("40");
    let words = description("Small flames.");
    let with_words = cat(&[&first, &one(&words), &panel]);
    screen.say(args(
        &[changed(&words), changed(&panel[3])],
        vec![CursorItem::new(0x0C, first.clone(), false)],
        &with_words,
    ));
    for _ in 0..DETAIL_POLLS {
        screen.on(&with_words);
    }
    let second = vec![piece(1, "Scratch", -120.0, 70.0, MENU)];
    let claws = description("Sharp claws.");
    let with_claws = cat(&[&second, &one(&claws), &panel]);
    screen.say(args(
        &[changed(&claws)],
        vec![CursorItem::new(0x0C, second.clone(), false)],
        &with_claws,
    ));
    screen.on(&with_claws);
    assert_eq!(
        screen.on(&with_claws),
        ["Sharp claws. Category Power 40 Accuracy 100"]
    );
}

/// The Bag's panel: its pocket's name, the item's and its description, each a panel of its own.
fn bag_panel(pocket: &str, name: &str, words: &str) -> Vec<ScreenText> {
    vec![
        ScreenText {
            plain: true,
            ..piece_kw(20, pocket, -20.0, 110.0, PANEL, Kw { panel: 20, ..PLAIN })
        },
        piece_kw(21, name, -28.0, 75.0, PANEL, Kw { panel: 21, ..PLAIN }),
        piece_kw(22, words, -150.0, -15.0, PANEL, Kw { panel: 22, ..PLAIN }),
    ]
}

/// The Bag's cursor moving to `item`, with `changed` rewritten, and what is said until the
/// details settle.
fn bag_move(
    screen: &mut Screen,
    item: &str,
    now: &[ScreenText],
    changed: &[ScreenText],
) -> Vec<String> {
    let cursor = vec![piece(1, item, 8.0, 90.0, MENU)];
    let texts: Vec<ScreenText> = changed
        .iter()
        .map(|p| ScreenText {
            changed: true,
            with_layout: false,
            ..p.clone()
        })
        .collect();
    let on_screen = cat(&[&cursor, now]);
    let mut said = screen.say(args(
        &texts,
        vec![CursorItem::new(0x0C, cursor.clone(), false)],
        &on_screen,
    ));
    for _ in 0..DETAIL_POLLS {
        said.extend(screen.on(&on_screen));
    }
    said
}

#[test]
fn test_an_items_details_leave_out_the_item_and_a_title_said_already() {
    // The Bag hangs its pocket's name, the item's and its description from one root, and every
    // item's details began with both, "MEDICINE Full Heal" (or_tm-hm, Khai, 2026-10-18). The
    // pocket is said again where the move changes it.
    let mut screen = Screen::new();
    let potion = vec![piece(1, "Potion", 8.0, 90.0, MENU)];
    let shown = bag_panel("MEDICINE", "Potion", "A spray for wounds.");
    assert_eq!(
        screen.say(args(
            &cat(&[&potion, &shown]),
            vec![CursorItem::new(0x0C, potion.clone(), true)],
            &cat(&[&potion, &shown]),
        )),
        ["Potion", "MEDICINE", "A spray for wounds."]
    );
    let heal = bag_panel("MEDICINE", "Full Heal", "Heals all status conditions.");
    assert_eq!(
        bag_move(&mut screen, "Full Heal", &heal, &heal[1..]),
        ["Full Heal", "Heals all status conditions."]
    );
    let coin = bag_panel("ITEMS", "Amulet Coin", "Doubles prize money.");
    assert_eq!(
        bag_move(&mut screen, "Amulet Coin", &coin, &coin),
        ["Amulet Coin", "ITEMS Doubles prize money."]
    );
    let tm = bag_panel("TMs & HMs", "TM04 Calm Mind", "Calms its spirit.");
    assert_eq!(
        bag_move(&mut screen, "Calm Mind", &tm, &tm),
        ["Calm Mind", "TMs & HMs TM04 Calms its spirit."]
    );
}

/// TM29 Psychic's type is "Psychic", the cursor's item, and its details said "TYPE POWER 90"
/// (or_movie54, Orion, 2026-10-19, #13). A value beside its label is never the item's name.
#[test]
fn test_a_labels_value_in_the_items_words_is_said() {
    let mut screen = Screen::new();
    let first = vec![piece(1, "Flamethrower", 8.0, 90.0, MENU)];
    screen.say(args(
        &first,
        vec![CursorItem::new(0x0C, first.clone(), true)],
        &first,
    ));
    let cursor = vec![piece(1, "Psychic", 8.0, 90.0, MENU)];
    let in_panel = |parent: i64, panel: i64| Kw {
        parent,
        panel,
        ..PLAIN
    };
    let now = vec![
        piece_kw(21, "TM29 Psychic", -28.0, 75.0, PANEL, in_panel(0, 21)),
        ScreenText {
            label: true,
            ..piece_kw(30, "TYPE", -144.0, -70.0, PANEL, in_panel(5, 5))
        },
        piece_kw(35, "Psychic", -54.0, -70.0, PANEL, in_panel(5, 5)),
    ];
    let texts: Vec<ScreenText> = now
        .iter()
        .map(|p| ScreenText {
            changed: true,
            with_layout: false,
            ..p.clone()
        })
        .collect();
    let on_screen = cat(&[&cursor, &now]);
    let mut said = screen.say(args(
        &texts,
        vec![CursorItem::new(0x0C, cursor.clone(), false)],
        &on_screen,
    ));
    for _ in 0..DETAIL_POLLS {
        said.extend(screen.on(&on_screen));
    }
    assert_eq!(said, ["Psychic", "TM29, TYPE Psychic"]);
}

#[test]
fn test_a_value_hung_under_its_label_is_read_with_it_when_it_has_not_changed() {
    // The Bag's TM pocket hangs each number under its label, and moving from Calm Mind to Bulk
    // Up read "TYPE Fighting POWER CATEGORY STATUS ACCURACY PP" (or_tm-hm, 2026-10-18).
    let mut screen = Screen::new();
    let under = |r#box: i64, text: &str, x: f64, y: f64, parent: i64| {
        piece_kw(r#box, text, x, y, PANEL, Kw { parent, ..PLAIN })
    };
    let labels = vec![
        under(30, "TYPE", -144.0, -70.0, 5),
        under(31, "POWER", 18.0, -70.0, 5),
        under(32, "ACCURACY", 18.0, -90.0, 5),
    ];
    let values = vec![
        under(33, "—", 141.0, -70.0, 31),
        under(34, "—", 141.0, -90.0, 32),
    ];
    let kind = |word: &str| ScreenText {
        picture: true,
        ..under(35, word, -54.0, -70.0, 5)
    };
    // The panel comes on screen as the cursor moves into the pocket, and only the type changes.
    let shown: Vec<ScreenText> = cat(&[&labels, &values])
        .into_iter()
        .map(|p| ScreenText {
            with_layout: false,
            ..p
        })
        .collect();
    let mut said = Vec::new();
    for (item, word, came) in [
        ("Calm Mind", "Psychic", shown),
        ("Bulk Up", "Fighting", vec![]),
    ] {
        let cursor = vec![piece(1, item, 8.0, 90.0, MENU)];
        let now = cat(&[&labels, &values, &one(&kind(word))]);
        let on_screen = cat(&[&cursor, &now]);
        let texts = cat(&[
            &came,
            &one(&ScreenText {
                changed: true,
                with_layout: false,
                ..kind(word)
            }),
        ]);
        said = screen.say(args(
            &texts,
            vec![CursorItem::new(0x0C, cursor.clone(), false)],
            &on_screen,
        ));
        for _ in 0..DETAIL_POLLS {
            said.extend(screen.on(&on_screen));
        }
    }
    assert_eq!(said, ["Bulk Up", "TYPE Fighting POWER — ACCURACY —"]);
}

#[test]
fn test_details_of_words_hanging_from_the_root_are_the_frames_their_changes_are_in() {
    // The move-forget screen hangs every word from its root, and each move read the Pokémon's
    // number, name and six stats before its power (omega_ruby_hm-tm.ctm, spike 139). Its frames
    // are the only groups it draws.
    let mut screen = Screen::new();
    let stat = vec![
        piece(20, "ATTACK", 55.0, 50.0, SUMMARY),
        piece(21, "15", 130.0, 50.0, SUMMARY),
    ];
    let moves: Vec<ScreenText> = move_panel("40")[..5]
        .iter()
        .map(|p| ScreenText {
            parent: SUMMARY,
            frame: 0x70,
            ..p.clone()
        })
        .collect();
    let everything = cat(&[&stat, &moves]);
    let growl = vec![piece(1, "Growl", -120.0, 70.0, MENU)];
    let all = cat(&[&growl, &everything]);
    screen.say(args(
        &[changed(&moves[3])],
        vec![CursorItem::new(0x0C, growl.clone(), false)],
        &all,
    ));
    for _ in 0..DETAIL_POLLS - 1 {
        screen.on(&all);
    }
    assert_eq!(screen.on(&all), ["Category Power 40 Accuracy 100"]);
}

/// The battle's move info, framed beside the buttons its cursor walks, in their layout: a
/// category written under its label, the power beside its own (or_move-learning).
fn battle_move_panel(r#move: &str, category: &str, power: &str, words: &str) -> Vec<ScreenText> {
    [
        piece(40, r#move, -108.0, 70.0, MENU),
        piece(41, "CATEGORY", -150.0, 40.0, MENU),
        piece(42, category, -100.0, 20.0, MENU),
        piece(43, "POWER", 15.0, 40.0, MENU),
        piece(44, power, 131.0, 40.0, MENU),
        piece(45, "ACCURACY", 15.0, 20.0, MENU),
        piece(46, "100", 124.0, 20.0, MENU),
        piece(47, words, -157.0, -30.0, MENU),
    ]
    .into_iter()
    .map(|p| ScreenText {
        frame: 0x70,
        panel: p.r#box,
        ..p
    })
    .collect()
}

#[test]
fn test_a_panel_framed_beside_a_menus_buttons_is_the_screens_and_its_items_details() {
    // The battle's move info and its move to forget each frame a move's panel in the layout of
    // the buttons the cursor walks, and as the menu's own it was never heard. The move info's
    // panel changes only as A chooses the move a button stands for (or_move-learning,
    // 2026-10-18).
    let mut screen = Screen::new();
    let under = |r#box: i64, text: &str| {
        piece_kw(r#box, text, 0.0, -100.0, MENU, Kw { parent: 9, ..PLAIN })
    };
    let first = vec![under(1, "move 1")];
    let shown = battle_move_panel("Waterfall", "PHYSICAL", "80", "Charges at the target.");
    let said = screen.say(args(
        &shown,
        vec![CursorItem::new(0x0C, first.clone(), true)],
        &cat(&[&first, &shown]),
    ));
    assert!(
        said[0] == "move 1"
            && said
                .iter()
                .any(|line| line.contains("Charges at the target.")),
        "{said:?}"
    );
    let second = vec![under(1, "move 2")];
    let mut heard = screen.say(args(
        &[],
        vec![CursorItem::new(0x0C, second.clone(), false)],
        &cat(&[&second, &shown]),
    ));
    for _ in 0..DETAIL_POLLS + 1 {
        heard.extend(screen.on(&cat(&[&second, &shown])));
    }
    let picked = battle_move_panel("Dragon Pulse", "SPECIAL", "85", "A shock wave.");
    let rewritten: Vec<ScreenText> = picked
        .iter()
        .map(|p| ScreenText {
            changed: true,
            with_layout: false,
            ..p.clone()
        })
        .collect();
    heard.extend(screen.say(args(&rewritten, vec![], &cat(&[&second, &picked]))));
    for _ in 0..DETAIL_POLLS {
        heard.extend(screen.on(&cat(&[&second, &picked])));
    }
    assert_eq!(
        heard,
        [
            "move 2",
            "Dragon Pulse",
            "CATEGORY, SPECIAL, POWER 85, ACCURACY 100, A shock wave."
        ]
    );
}

/// The AreaNav's tabs are pictures in its bar's layout, and the bar's View Details, coming
/// back with it, went unheard once the tabs' cursor had been over them (or_area_nav).
#[test]
fn test_a_cursor_over_pictures_alone_leaves_the_rest_of_their_layout_to_the_screen() {
    let mut screen = Screen::new();
    let tab = ScreenText {
        picture: true,
        glyph: 0.0,
        ..piece_kw(
            1,
            "Trainers",
            -60.0,
            -110.0,
            MENU,
            Kw { parent: 9, ..PLAIN },
        )
    };
    screen.say(args(&[], vec![CursorItem::new(0x0C, vec![tab], true)], &[]));
    let details = [piece_kw(
        2,
        "View Details",
        80.0,
        -110.0,
        MENU,
        Kw { parent: 8, ..PLAIN },
    )];
    assert_eq!(
        screen.say(args(&details, vec![], &details)),
        ["View Details"]
    );
}

#[test]
fn test_a_panel_one_layouts_moves_changed_is_not_read_for_another_layouts_items() {
    // The battle's move to forget stays on screen as the cursor goes back to "return" in the
    // move list beside it, and was read again there (or_move-learning, 2026-10-18).
    let mut screen = Screen::new();
    let forget = vec![piece_kw(
        1,
        "FORGET",
        0.0,
        -90.0,
        PANEL,
        Kw { parent: 9, ..PLAIN },
    )];
    let words = description("Doubles damage on a switch.");
    let back = vec![piece_kw(
        2,
        "return",
        120.0,
        -100.0,
        MENU,
        Kw { parent: 8, ..PLAIN },
    )];
    screen.say(args(
        &[changed(&words)],
        vec![CursorItem::new(0x0C, forget.clone(), false)],
        &cat(&[&forget, &one(&words)]),
    ));
    for _ in 0..DETAIL_POLLS {
        screen.on(&cat(&[&forget, &one(&words)]));
    }
    let everything = cat(&[&forget, &back, &one(&words)]);
    let mut said = screen.say(args(
        &[],
        vec![CursorItem::new(0x0C, back.clone(), false)],
        &everything,
    ));
    for _ in 0..DETAIL_POLLS {
        said.extend(screen.on(&everything));
    }
    assert_eq!(said, ["return"]);
}

#[test]
fn test_details_are_read_only_from_the_layouts_this_move_changed() {
    // A summary keeps its other page's panel on screen as that page last showed it, and Zen
    // Headbutt's details on the contest page were read with Iron Defense's (movie32).
    let mut screen = Screen::new();
    let battle = description("The user hardens its body.");
    let contest = piece_kw(
        30,
        "Quite an appealing move.",
        -150.0,
        45.0,
        PANEL,
        Kw { parent: 8, ..PLAIN },
    );
    let headbutt = vec![piece(1, "Zen Headbutt", -120.0, 70.0, MENU)];
    let everything = cat(&[&headbutt, &one(&battle), &one(&contest)]);
    screen.say(args(
        &[changed(&battle)],
        vec![CursorItem::new(0x0C, headbutt.clone(), false)],
        &everything,
    ));
    for _ in 0..DETAIL_POLLS {
        screen.on(&everything);
    }
    let mut said = screen.say(args(
        &[changed(&contest)],
        vec![CursorItem::new(0x0C, headbutt.clone(), false)],
        &everything,
    ));
    for _ in 0..DETAIL_POLLS {
        said.extend(screen.on(&everything));
    }
    assert_eq!(said, ["Zen Headbutt", "Quite an appealing move."]);
}

#[test]
fn test_moving_on_drops_the_last_items_details_unsaid() {
    let mut screen = Screen::new();
    let panel = move_panel("40");
    let growl = vec![piece(1, "Growl", -120.0, 70.0, MENU)];
    screen.say(args(
        &[changed(&panel[3])],
        vec![CursorItem::new(0x0C, growl.clone(), false)],
        &cat(&[&growl, &panel]),
    ));
    let ember = vec![piece(1, "Ember", -120.0, 70.0, MENU)];
    let everything = cat(&[&ember, &panel]);
    assert_eq!(
        screen.say(args(
            &[],
            vec![CursorItem::new(0x0C, ember.clone(), false)],
            &everything
        )),
        ["Ember"]
    );
    assert_eq!(screen.on(&everything), NOTHING);
    assert_eq!(screen.on(&everything), ["Category Power 40 Accuracy 100"]);
}

/// The fitting room's tabs change the card beside them, and each move between Change and Back,
/// a list of their own, read the card again (Khai, 2026-10-24, x_cyllage_boutique_fitting_room).
#[test]
fn test_an_item_of_another_list_keeps_no_details_from_this_ones_moves() {
    let mut screen = Screen::new();
    let panel = move_panel("40");
    let tab = vec![piece(1, "bag", -120.0, 70.0, MENU)];
    let everything = cat(&[&tab, &panel]);
    screen.say(args(
        &[changed(&panel[3])],
        vec![CursorItem::new(0x0C, tab.clone(), false)],
        &everything,
    ));
    assert_eq!(screen.on(&everything), NOTHING);
    assert_eq!(screen.on(&everything), ["Category Power 40 Accuracy 100"]);
    for word in ["Change", "Back"] {
        let moved = vec![piece(2, word, -120.0, 70.0, MENU)];
        let everything = cat(&[&moved, &panel]);
        assert_eq!(
            screen.say(args(
                &[],
                vec![CursorItem {
                    rows: 0x40,
                    ..CursorItem::new(0x0C, moved.clone(), false)
                }],
                &everything
            )),
            [word]
        );
        let heard: Vec<Vec<String>> = (0..3).map(|_| screen.on(&everything)).collect();
        assert_eq!(heard, vec![Vec::<String>::new(); 3], "{word}");
    }
}

#[test]
fn test_details_wait_for_words_being_written_no_longer_than_their_most_polls() {
    let mut screen = Screen::new();
    let growl = vec![piece(1, "Growl", -120.0, 70.0, MENU)];
    let panel = move_panel("40");
    let everything = cat(&[&growl, &panel]);
    screen.say(args(
        &[changed(&panel[3])],
        vec![CursorItem::new(0x0C, growl.clone(), false)],
        &everything,
    ));
    let heard: Vec<Vec<String>> = (0..DETAIL_MAX_POLLS)
        .map(|_| screen.on_settling(&everything, &[SUMMARY]))
        .collect();
    let silent: Vec<Vec<String>> = vec![vec![]; (DETAIL_MAX_POLLS - 1) as usize];
    assert_eq!(heard[..heard.len() - 1], silent[..]);
    assert_eq!(heard[heard.len() - 1], ["Category Power 40 Accuracy 100"]);
}

#[test]
fn test_a_value_that_changes_long_after_the_cursor_is_said_with_its_row() {
    let mut screen = Screen::new();
    let growl = vec![piece(1, "Growl", -120.0, 70.0, MENU)];
    screen.say(args(
        &[],
        vec![CursorItem::new(0x0C, growl.clone(), false)],
        &growl,
    ));
    for _ in 0..DETAIL_POLLS {
        screen.on(&growl);
    }
    let panel = move_panel("40");
    let power = changed(&panel[3]);
    assert_eq!(screen.say(args(&[power], vec![], &panel)), ["Power 40"]);
}

#[test]
fn test_what_changes_as_a_cursor_comes_on_screen_is_not_its_details() {
    // Changing pocket brought the bag's cursor back with the pocket name, and "MEDICINE" was read
    // with the item panel beside it (menus.ctm replay, 2026-09-14).
    let mut screen = Screen::new();
    let potion = vec![piece(1, "Potion", 80.0, 90.0, MENU)];
    screen.say(args(
        &[],
        vec![CursorItem::new(0x0C, potion.clone(), true)],
        &potion,
    ));
    let under = Kw { parent: 5, ..PLAIN };
    let pocket = changed(&piece_kw(2, "MEDICINE", -100.0, 110.0, PANEL, under));
    let count = piece_kw(3, "x1", -40.0, 80.0, PANEL, under);
    assert_eq!(
        screen.say(args(
            &one(&pocket),
            vec![],
            &cat(&[&potion, &[pocket.clone(), count.clone()]])
        )),
        ["MEDICINE"]
    );
}

#[test]
fn test_words_coming_on_screen_as_details_gather_are_part_of_them() {
    // The Mart's count and description came back as its cursor left Quit and were said as two
    // lines, and a contest's move panel, shown afresh for each move, half written (the audit,
    // 2026-10-10).
    let mut screen = Screen::new();
    let growl = vec![piece(1, "Growl", -120.0, 70.0, MENU)];
    let power = changed(&move_panel("40")[3]);
    let old: Vec<ScreenText> = move_panel("40")
        .into_iter()
        .filter(|p| p.r#box == 10 || p.r#box == 11)
        .collect();
    let new = piece_kw(
        16,
        "Hard claws.",
        -210.0,
        40.0,
        SUMMARY,
        Kw {
            parent: 5,
            with_layout: false,
            ..PLAIN
        },
    );
    let everything = cat(&[&growl, &old, &one(&power), &one(&new)]);
    let said = screen.say(args(
        &[power.clone(), new.clone()],
        vec![CursorItem::new(0x0C, growl.clone(), false)],
        &everything,
    ));
    assert_eq!(said, ["Growl"]);
    assert_eq!(screen.on(&everything), NOTHING);
    assert_eq!(screen.on(&everything), ["Hard claws. Category Power 40"]);
}

#[test]
fn test_words_under_a_cursor_that_settle_after_it_spoke_are_not_said_again() {
    // The bag's "Potion" settled a poll after its cursor said it, and was heard twice.
    let mut screen = Screen::new();
    let nothing = piece(1, "Nothing", 80.0, 60.0, MENU);
    screen.say(args(
        &one(&nothing),
        vec![CursorItem::new(0x0C, vec![nothing.clone()], true)],
        &one(&nothing),
    ));
    let potion = piece_kw(
        2,
        "Potion",
        80.0,
        90.0,
        MENU,
        Kw {
            with_layout: false,
            ..PLAIN
        },
    );
    let items = vec![CursorItem::new(0x0C, vec![potion.clone()], false)];
    screen.say(args(&[], items, &one(&potion)));
    let under = vec![vec![potion.clone()]];
    let before = screen.backend.spoken().len();
    screen
        .announcer
        .say(
            &ScreenPoll {
                texts: one(&potion),
                on_screen: one(&potion),
                ..ScreenPoll::default()
            },
            &CursorPoll {
                under,
                ..CursorPoll::default()
            },
            &HashSet::new(),
            None,
            None,
        )
        .unwrap();
    assert_eq!(screen.backend.spoken()[before..], NOTHING);
}

#[test]
fn test_the_cursor_item_comes_before_what_changed_with_it() {
    // Moving through the moves brings up each move's description as well as the move.
    let mut screen = Screen::new();
    let moved = vec![
        piece(1, "Growl", -120.0, 70.0, MENU),
        piece(2, "PP", -130.0, 50.0, MENU),
    ];
    let description = changed(&piece(
        9,
        "The user growls in an endearing way.",
        0.0,
        40.0,
        PANEL,
    ));
    let items = vec![CursorItem::new(0x0C, moved.clone(), false)];
    let everything = cat(&[&moved, &one(&description)]);
    let said = screen.say(args(&one(&description), items, &everything));
    assert_eq!(said, ["Growl PP"]);
    assert_eq!(screen.on(&everything), NOTHING);
    assert_eq!(screen.on(&everything), [description.text.as_str()]);
}

#[test]
fn test_rows_that_change_in_a_menu_are_left_to_its_cursor() {
    // A list scrolling rewrites every row it shows; only the one under the cursor is heard.
    let mut screen = Screen::new();
    let rows = [
        piece(1, "Potion", 80.0, 90.0, MENU),
        piece(2, "Antidote", 80.0, 60.0, MENU),
    ];
    let scrolled = vec![
        rewritten(&rows[0], "Antidote"),
        rewritten(&rows[1], "Burn Heal"),
    ];
    let items = vec![CursorItem::new(0x0C, vec![scrolled[0].clone()], false)];
    assert_eq!(screen.say(args(&scrolled, items, &scrolled)), ["Antidote"]);
}

#[test]
fn test_rows_that_scroll_into_view_in_a_menu_are_left_to_its_cursor() {
    // The language screen said "Spanish" and then "Japanese": its list moves its rows, and the row
    // moving up into view was taken for words new to the menu (opening.ctm, 2026-09-17).
    let mut screen = Screen::new();
    let rows = vec![
        piece(1, "English", -145.0, -10.0, MENU),
        piece(2, "Spanish", -145.0, -40.0, MENU),
    ];
    screen.say(args(
        &rows,
        vec![CursorItem::new(0x0C, vec![rows[0].clone()], true)],
        &rows,
    ));
    let moved = vec![
        ScreenText {
            row: 20.0,
            ..rows[0].clone()
        },
        ScreenText {
            row: -10.0,
            ..rows[1].clone()
        },
    ];
    let japanese = ScreenText {
        scrolled: true,
        ..piece_kw(
            3,
            "Japanese",
            -145.0,
            -100.0,
            MENU,
            Kw {
                with_layout: false,
                ..PLAIN
            },
        )
    };
    let items = vec![CursorItem::new(0x0C, vec![moved[1].clone()], false)];
    assert_eq!(
        screen.say(args(
            &one(&japanese),
            items,
            &cat(&[&moved, &one(&japanese)])
        )),
        ["Spanish"]
    );
}

#[test]
fn test_list_labels_are_left_to_the_list_readers() {
    let mut screen = Screen::new();
    let labels = vec![
        piece(1, "Yes", 100.0, 30.0, MENU),
        piece(2, "No", 100.0, 0.0, MENU),
    ];
    let items = vec![CursorItem::new(0x0C, vec![labels[0].clone()], true)];
    assert_eq!(
        screen.say(Args {
            labels: labelled(&["Yes", "No"]),
            ..args(&labels, items, &labels)
        }),
        NOTHING
    );
}

#[test]
fn test_a_cursor_over_a_lists_labels_is_the_lists_and_says_nothing() {
    // The options confirmation's cursor held the rows behind its Yes, and read "SLOW ON".
    let mut screen = Screen::new();
    let behind = vec![
        piece(1, "SLOW", 60.0, 60.0, OPTIONS),
        piece(2, "ON", 60.0, 30.0, OPTIONS),
    ];
    let yes = piece(3, "Yes", 60.0, 45.0, MENU);
    let items = vec![CursorItem::new(
        0x0C,
        vec![behind[0].clone(), yes.clone(), behind[1].clone()],
        true,
    )];
    assert_eq!(
        screen.say(Args {
            labels: labelled(&["Yes", "No"]),
            ..args(&[], items, &cat(&[&behind, &one(&yes)]))
        }),
        NOTHING
    );
}

#[test]
fn test_a_lists_cursor_does_not_make_what_is_behind_it_a_menu() {
    let mut screen = Screen::new();
    let behind = vec![piece(1, "SLOW", 60.0, 60.0, OPTIONS)];
    let yes = piece(3, "Yes", 60.0, 45.0, MENU);
    screen.say(Args {
        labels: labelled(&["Yes"]),
        ..args(
            &[],
            vec![CursorItem::new(
                0x0C,
                vec![behind[0].clone(), yes.clone()],
                true,
            )],
            &cat(&[&behind, &one(&yes)]),
        )
    });
    let question = piece(
        4,
        "Would you like to make these changes?",
        0.0,
        100.0,
        OPTIONS,
    );
    assert_eq!(
        screen.say(args(&one(&question), vec![], &one(&question))),
        [question.text.as_str()]
    );
}

const AREA_NAV: i64 = 0x15A5CCDC;
const SIGN: i64 = 0x082E57F4;

#[test]
fn test_words_another_layout_has_just_said_are_not_said_again() {
    // Walking onto Route 101 said it twice: AreaNav's words changed, and the location sign came up
    // with the same words a poll later (the user, 2026-09-14).
    let mut screen = Screen::new();
    let area = changed(&piece(1, "Route 101", 0.0, 100.0, AREA_NAV));
    assert_eq!(
        screen.say(args(&one(&area), vec![], &one(&area))),
        ["Route 101"]
    );
    let sign = piece(2, "Route 101", -120.0, 100.0, SIGN);
    assert_eq!(
        screen.say(args(&one(&sign), vec![], &[area.clone(), sign.clone()])),
        NOTHING
    );
}

#[test]
fn test_words_another_layout_says_in_the_same_poll_are_said_once() {
    let mut screen = Screen::new();
    let area = changed(&piece(1, "Littleroot Town", 0.0, 100.0, AREA_NAV));
    let sign = piece(2, "Littleroot Town", -120.0, 100.0, SIGN);
    let both = vec![area.clone(), sign.clone()];
    assert_eq!(screen.say(args(&both, vec![], &both)), ["Littleroot Town"]);
}

#[test]
fn test_the_same_words_from_another_layout_a_while_later_are_said() {
    let mut screen = Screen::new();
    let area = changed(&piece(1, "Route 101", 0.0, 100.0, AREA_NAV));
    screen.say(args(&one(&area), vec![], &one(&area)));
    for _ in 0..REPEAT_POLLS {
        screen.on(&one(&area));
    }
    let sign = piece(2, "Route 101", -120.0, 100.0, SIGN);
    assert_eq!(
        screen.say(args(&one(&sign), vec![], &[area.clone(), sign.clone()])),
        ["Route 101"]
    );
}

#[test]
fn test_a_place_said_from_the_field_is_not_said_again_by_areanav_or_the_sign() {
    // The zone is in RAM 0.6 to 0.8 s before AreaNav's words change, and the sign comes 0.23 s
    // after those (spike 87): the name is said once, first, and the screen's copies are left.
    let mut screen = Screen::new();
    screen.quiet();
    screen.announcer.place("Route 101");
    assert_eq!(screen.backend.spoken(), ["Route 101"]);
    for _ in 0..3 {
        assert_eq!(screen.quiet(), NOTHING);
    }
    let area = changed(&piece(1, "Route 101", 0.0, 100.0, AREA_NAV));
    assert_eq!(screen.say(args(&one(&area), vec![], &one(&area))), NOTHING);
    assert_eq!(screen.on(&one(&area)), NOTHING);
    let sign = piece(2, "Route 101", -120.0, 100.0, SIGN);
    assert_eq!(
        screen.say(args(&one(&sign), vec![], &[area.clone(), sign.clone()])),
        NOTHING
    );
}

#[test]
fn test_a_place_the_screen_said_first_is_not_said_again_from_the_field() {
    // Before a sweep has found the zone table, AreaNav can be first.
    let mut screen = Screen::new();
    let area = changed(&piece(1, "Littleroot Town", 0.0, 100.0, AREA_NAV));
    assert_eq!(
        screen.say(args(&one(&area), vec![], &one(&area))),
        ["Littleroot Town"]
    );
    screen.announcer.place("Littleroot Town");
    assert_eq!(screen.backend.spoken(), ["Littleroot Town"]);
}

#[test]
fn test_the_same_place_on_screen_long_after_the_field_said_it_is_heard() {
    let mut screen = Screen::new();
    screen.announcer.place("Littleroot Town");
    for _ in 0..PLACE_POLLS + 1 {
        screen.quiet();
    }
    let area = changed(&piece(1, "Littleroot Town", 0.0, 100.0, AREA_NAV));
    assert_eq!(
        screen.say(args(&one(&area), vec![], &one(&area))),
        ["Littleroot Town"]
    );
}

#[test]
fn test_other_words_come_on_screen_beside_words_just_said_elsewhere() {
    let mut screen = Screen::new();
    let area = changed(&piece(1, "Route 101", 0.0, 100.0, AREA_NAV));
    screen.say(args(&one(&area), vec![], &one(&area)));
    let sign = piece(2, "Route 101", -120.0, 100.0, SIGN);
    let card = piece(3, "Name: wee", -150.0, 80.0, PANEL);
    assert_eq!(
        screen.say(args(
            &[sign.clone(), card.clone()],
            vec![],
            &[area.clone(), sign.clone(), card.clone()]
        )),
        ["Name: wee"]
    );
}

#[test]
fn test_pauses_go_between_pieces_said_together() {
    let mut screen = Screen::with(true);
    let card = vec![
        piece(1, "Name", -150.0, 80.0, PANEL),
        piece(2, "wee", -60.0, 80.0, PANEL),
        piece(3, "Level:", -150.0, 60.0, PANEL),
        piece(4, "5", -60.0, 60.0, PANEL),
    ];
    assert_eq!(
        screen.say(args(&card, vec![], &card)),
        ["Name: wee, Level: 5"]
    );
}

const MART: i64 = 0x08500000;

/// The Poké Mart as it opens, in one layout: its list, then the money, the count in the bag and
/// the chosen item's description, each hanging from a pane of its own (spike 95).
fn mart() -> (Vec<ScreenText>, Vec<ScreenText>) {
    let at = |r#box: i64, text: &str, x: f64, y: f64, panel: i64| {
        piece_kw(
            r#box,
            text,
            x,
            y,
            MART,
            Kw {
                parent: panel,
                panel,
                ..PLAIN
            },
        )
    };
    let rows = vec![
        at(1, "Poké Ball", -55.0, 90.0, 10),
        at(2, "$ 200", 137.0, 90.0, 10),
        at(3, "Potion", -55.0, 70.0, 10),
        at(4, "$ 300", 137.0, 70.0, 10),
    ];
    let rest = vec![
        at(5, "Money", -135.0, 104.0, 11),
        at(6, "$ 3,440", -131.0, 80.0, 11),
        at(7, "In Bag:", 55.0, 110.0, 12),
        at(8, "10", 140.0, 110.0, 12),
        at(9, "A device for catching wild Pokémon.", 15.0, -87.0, 13),
    ];
    (rows, rest)
}

/// What is said of a poll where the cursor reader knows the list its cursor walks.
fn listed(
    screen: &mut Screen,
    texts: &[ScreenText],
    items: &[CursorItem],
    on_screen: &[ScreenText],
    rows: &[ScreenText],
) -> Vec<String> {
    let before = screen.backend.spoken().len();
    let under: Vec<Vec<ScreenText>> = items.iter().map(|item| item.pieces.clone()).collect();
    let lists: IndexMap<i64, HashSet<i64>> =
        IndexMap::from([(MART, rows.iter().map(|p| p.r#box).collect())]);
    screen
        .announcer
        .say(
            &ScreenPoll {
                texts: texts.to_vec(),
                on_screen: on_screen.to_vec(),
                ..ScreenPoll::default()
            },
            &CursorPoll {
                items: items.to_vec(),
                under,
                lists,
                ..CursorPoll::default()
            },
            &HashSet::new(),
            None,
            None,
        )
        .unwrap();
    screen.backend.spoken()[before..].to_vec()
}

#[test]
fn test_a_cursor_that_knows_its_list_leaves_the_rest_of_its_layout_to_be_heard() {
    // The Mart read its whole list as it opened, and never which item was chosen (shop.ctm, the
    // user, 2026-09-24). Its list is the cursor's; its money and description are the screen's.
    let mut screen = Screen::new();
    let (rows, rest) = mart();
    let item = CursorItem::new(0x0C, rows[..2].to_vec(), true);
    let all = cat(&[&rows, &rest]);
    let said = listed(&mut screen, &all, &[item], &all, &rows);
    assert_eq!(
        said,
        [
            "Poké Ball $ 200",
            "Money $ 3,440",
            "In Bag: 10",
            "A device for catching wild Pokémon."
        ]
    );
}

#[test]
fn test_what_changes_as_a_list_cursor_moves_is_said_as_its_details() {
    let mut screen = Screen::new();
    let (rows, rest) = mart();
    let all = cat(&[&rows, &rest]);
    listed(
        &mut screen,
        &all,
        &[CursorItem::new(0x0C, rows[..2].to_vec(), true)],
        &all,
        &rows,
    );
    let count = rewritten(&rest[3], "9");
    let words = rewritten(&rest[4], "A spray-type medicine for treating wounds.");
    let now = cat(&[&rows, &rest[..3], &[count.clone(), words.clone()]]);
    let moved = vec![CursorItem::new(0x0C, rows[2..].to_vec(), false)];
    assert_eq!(
        listed(&mut screen, &[count, words], &moved, &now, &rows),
        ["Potion $ 300"]
    );
    assert_eq!(listed(&mut screen, &[], &[], &now, &rows), NOTHING);
    assert_eq!(
        listed(&mut screen, &[], &[], &now, &rows),
        ["In Bag: 9, A spray-type medicine for treating wounds."]
    );
}

#[test]
fn test_a_row_of_a_cursors_list_coming_into_view_is_left_to_the_cursor() {
    // The Pokédex shows rows it had hidden as it scrolls away from the end of its list.
    let mut screen = Screen::new();
    let (rows, rest) = mart();
    let all = cat(&[&rows, &rest]);
    listed(
        &mut screen,
        &all,
        &[CursorItem::new(0x0C, rows[..2].to_vec(), true)],
        &all,
        &rows,
    );
    let antidote = piece_kw(
        20,
        "Antidote",
        -55.0,
        50.0,
        MART,
        Kw {
            parent: 10,
            with_layout: false,
            panel: 10,
        },
    );
    let on_screen = cat(&[&rows, &one(&antidote), &rest]);
    let with_antidote = cat(&[&rows, &one(&antidote)]);
    assert_eq!(
        listed(
            &mut screen,
            &one(&antidote),
            &[],
            &on_screen,
            &with_antidote
        ),
        NOTHING
    );
}

#[test]
fn test_words_coming_back_to_a_list_cursors_layout_come_alone() {
    // Up from the Mart's top item is Quit, which hides the count in the bag; coming back down
    // brought it back, and with it the whole screen, list and all, every time (shop.ctm).
    let mut screen = Screen::new();
    let (rows, rest) = mart();
    let all = cat(&[&rows, &rest]);
    listed(
        &mut screen,
        &all,
        &[CursorItem::new(0x0C, rows[..2].to_vec(), true)],
        &all,
        &rows,
    );
    let back: Vec<ScreenText> = rest[2..4]
        .iter()
        .map(|p| ScreenText {
            with_layout: false,
            returned: true,
            ..p.clone()
        })
        .collect();
    assert_eq!(listed(&mut screen, &back, &[], &all, &rows), ["In Bag: 10"]);
}

/// The Pokédex entry's layout: one panel under its root, `1y` (pokedex2.ctm, spike 96).
const ENTRY: i64 = 0x08600000;

/// The entry in the order the game builds it: species and gender, then the table of TYPE, HEIGHT
/// and WEIGHT as a column of labels and a column of values, the times encountered, and the number
/// and name at the top built last.
fn entry(species: &str, height: &str, name: (&str, &str)) -> Vec<ScreenText> {
    let at = |r#box: i64, text: &str, x: f64, y: f64, parent: i64| {
        piece_kw(
            r#box,
            text,
            x,
            y,
            ENTRY,
            Kw {
                parent,
                panel: 10,
                ..PLAIN
            },
        )
    };
    vec![
        at(1, species, 5.0, 70.0, 2),
        at(2, "Female", 5.0, 50.0, 2),
        at(3, "TYPE", -25.0, 30.0, 15),
        at(4, "HEIGHT", -25.0, 10.0, 15),
        at(5, "WEIGHT", -25.0, -10.0, 15),
        at(6, height, 26.0, 10.0, 15),
        at(7, "5.5 lbs.", 26.0, -10.0, 15),
        at(8, "Times encountered:", -90.0, -35.0, 2),
        at(9, "1", 40.0, -35.0, 2),
        at(10, name.0, 5.0, 100.0, 20),
        at(11, name.1, 40.0, 100.0, 20),
    ]
}

/// `entry()`: every argument at its default.
fn torchic_entry() -> Vec<ScreenText> {
    entry("Chick Pokémon", "1'04\"", ("004", "Torchic"))
}

#[test]
fn test_an_entry_of_one_column_is_read_top_to_bottom() {
    // It read "Chick Pokémon Female TYPE HEIGHT WEIGHT 1'04" 5.5 lbs. Times encountered: 1 004
    // Torchic" (pokedex2.ctm, the user, 2026-09-24).
    let mut screen = Screen::new();
    let shown = torchic_entry();
    assert_eq!(
        screen.say(args(&shown, vec![], &shown)),
        [concat!(
            "004 Torchic Chick Pokémon Female TYPE HEIGHT 1'04\" WEIGHT 5.5 lbs. ",
            "Times encountered: 1"
        )]
    );
}

#[test]
fn test_an_entry_rewritten_says_its_first_row_then_the_rest_as_one_line() {
    // Up and down in the Pokédex said the entry a row a line, "007 Mudkip", "????? Pokémon",
    // each cutting off the one before under `interrupt` (the audit, 2026-10-10).
    let mut screen = Screen::new();
    let shown = torchic_entry();
    screen.say(args(&shown, vec![], &shown));
    let now = entry("????? Pokémon", "???'??\"", ("007", "Mudkip"));
    let altered = altered_from(&shown, &now);
    assert_eq!(screen.say(args(&altered, vec![], &now)), ["007 Mudkip"]);
    assert_eq!(screen.on(&now), NOTHING);
    assert_eq!(
        screen.on(&now),
        ["????? Pokémon Female TYPE HEIGHT ???'??\" WEIGHT 5.5 lbs. Times encountered: 1"]
    );
}

/// `[replace(p, changed=True) for p in now if p.text != shown[p.box - 1].text]`.
fn altered_from(shown: &[ScreenText], now: &[ScreenText]) -> Vec<ScreenText> {
    now.iter()
        .filter(|p| p.text != shown[(p.r#box - 1) as usize].text)
        .map(changed)
        .collect()
}

/// `piece(box, text, x, y, root, parent=parent, with_layout=False)`.
fn later(r#box: i64, text: &str, x: f64, y: f64, root: i64, parent: i64) -> ScreenText {
    piece_kw(
        r#box,
        text,
        x,
        y,
        root,
        Kw {
            parent,
            with_layout: false,
            ..PLAIN
        },
    )
}

/// One cursor move to `item` with `texts` changed, then `DETAIL_POLLS` polls of `everything`:
/// what was said.
fn moved_and_settled(
    screen: &mut Screen,
    texts: &[ScreenText],
    item: &[ScreenText],
    everything: &[ScreenText],
) -> Vec<String> {
    let mut said = screen.say(args(
        texts,
        vec![CursorItem::new(0x0C, item.to_vec(), false)],
        everything,
    ));
    for _ in 0..DETAIL_POLLS {
        said.extend(screen.on(everything));
    }
    said
}

#[test]
fn test_details_drawn_side_by_side_in_two_groups_are_read_as_one_table() {
    // The Bag's TM pocket hangs a move's numbers apart from their labels, and they were said
    // after them, "TYPE Normal POWER CATEGORY SPECIAL ACCURACY PP, 15, 100, 40" (movie30).
    let mut screen = Screen::new();
    let tm = vec![piece(1, "Echoed Voice", 80.0, 90.0, MENU)];
    let names = vec![
        later(10, "POWER", 20.0, 40.0, PANEL, 5),
        later(11, "PP", 20.0, 20.0, PANEL, 5),
    ];
    let values = vec![
        later(12, "40", 120.0, 40.0, PANEL, 6),
        later(13, "15", 120.0, 20.0, PANEL, 7),
    ];
    let everything = cat(&[&tm, &names, &values]);
    let said = moved_and_settled(&mut screen, &cat(&[&names, &values]), &tm, &everything);
    assert_eq!(said, ["Echoed Voice", "POWER 40 PP 15"]);
}

#[test]
fn test_a_group_of_its_own_rows_is_not_read_into_a_table_beside_one_of_them() {
    // A summary's move panel, one row of it beside the contest page's labels, was read in among
    // them, "Type Dark Effect Appeal 4 hearts Jamming Cleverness Cuteness" (movie7).
    let mut screen = Screen::new();
    let r#move = vec![piece(1, "Ember", 80.0, 90.0, MENU)];
    let page = vec![
        later(10, "Type", -195.0, 40.0, PANEL, 5),
        later(11, "Cuteness", 142.0, -34.0, PANEL, 5),
    ];
    let panel = vec![
        later(12, "Effect", -195.0, 13.0, PANEL, 6),
        later(13, "Jamming", -195.0, -33.0, PANEL, 6),
    ];
    let everything = cat(&[&r#move, &page, &panel]);
    let said = moved_and_settled(&mut screen, &cat(&[&page, &panel]), &r#move, &everything);
    assert_eq!(said, ["Ember", "Type Cuteness, Effect Jamming"]);
}

#[test]
fn test_details_of_a_screen_of_two_columns_are_read_as_it_was_shown() {
    // The PC Box's summary is one group of two columns, the stats and the moves, and its details
    // were read across both, "SP. ATK: 68, MOVES LEARNED, SP. DEF: 58, Bubble Beam", and
    // "Intimidate: Sweet Scent, ABILITY", where opening it read each column down (Orion,
    // or_organize_boxes, 2026-10-24). Places are the summary's.
    let mut screen = Screen::with(true);
    let slot = vec![piece(1, "BATTLE BOX 1", 80.0, 90.0, MENU)];
    let kw = |parent: i64| Kw {
        parent,
        with_layout: false,
        ..PLAIN
    };
    let mut summary = vec![piece_kw(10, "bubbles", -170.0, 110.0, PANEL, kw(6))];
    for (r#box, (text, x, y)) in [
        ("♀", -70.0, 110.0),
        ("Lv.33", -34.0, 120.0),
        ("SP. ATK", -190.0, -10.0),
        ("68", -74.0, -10.0),
        ("SP. DEF", -190.0, -30.0),
        ("58", -74.0, -30.0),
        ("ABILITY", -190.0, -90.0),
        ("Intimidate", -110.0, -80.0),
        ("MOVES LEARNED", 40.0, -20.0),
        ("Bubble Beam", 50.0, -40.0),
        ("Sweet Scent", 50.0, -80.0),
    ]
    .into_iter()
    .enumerate()
    {
        summary.push(piece_kw(r#box as i64 + 11, text, x, y, PANEL, kw(5)));
    }
    let everything = cat(&[&slot, &summary]);
    let texts: Vec<ScreenText> = summary.iter().map(changed).collect();
    let mut said = screen.say(args(
        &texts,
        vec![CursorItem::new(0x0C, slot.clone(), false)],
        &everything,
    ));
    for _ in 0..DETAIL_POLLS {
        said.extend(screen.on(&everything));
    }
    assert_eq!(
        said,
        [
            "BATTLE BOX 1",
            "bubbles, ♀, Lv.33, SP. ATK: 68, SP. DEF: 58, ABILITY: Intimidate, MOVES LEARNED, \
             Bubble Beam, Sweet Scent"
        ]
    );
}

#[test]
fn test_two_groups_beside_each_other_on_one_row_are_read_apart() {
    // The PC Box's summary draws a Pokémon's name and its level, a little higher, side by side,
    // and read as one table it put the level first, "Lv.5 Umbra" (movie6).
    let mut screen = Screen::new();
    let slot = vec![piece(1, "Party 2", 80.0, 90.0, MENU)];
    let name = later(10, "Umbra", -150.0, 100.0, PANEL, 5);
    let level = later(11, "Lv.5", -60.0, 104.0, PANEL, 6);
    let everything = cat(&[&slot, &one(&name), &one(&level)]);
    let said = moved_and_settled(&mut screen, &[name, level], &slot, &everything);
    assert_eq!(said, ["Party 2", "Umbra, Lv.5"]);
}

#[test]
fn test_a_rewrite_leaves_out_what_a_cursor_has_just_said() {
    // Changing pocket brought the Bag's cursor back on Exp. Share, and the panel's rewrite said
    // it again (movie30).
    let mut screen = Screen::new();
    let at = |r#box: i64, text: &str, y: f64, parent: i64| {
        piece_kw(
            r#box,
            text,
            0.0,
            y,
            PANEL,
            Kw {
                parent,
                panel: parent,
                ..PLAIN
            },
        )
    };
    let shown = vec![
        at(1, "BERRIES", 110.0, 2),
        at(3, "Pecha Berry", 75.0, 4),
        at(5, "A Berry.", -15.0, 6),
    ];
    screen.say(args(&shown, vec![], &shown));
    let item = vec![piece(9, "Exp. Share", 80.0, 90.0, MENU)];
    let now = vec![
        rewritten(&shown[0], "KEY ITEMS"),
        rewritten(&shown[1], "Exp. Share"),
        rewritten(&shown[2], "A device."),
    ];
    let unchanged: Vec<ScreenText> = now
        .iter()
        .map(|p| ScreenText {
            changed: false,
            ..p.clone()
        })
        .collect();
    let everything = cat(&[&item, &unchanged]);
    let mut said = screen.say(args(
        &now,
        vec![CursorItem::new(0x0C, item.clone(), true)],
        &everything,
    ));
    for _ in 0..DETAIL_POLLS {
        said.extend(screen.on(&everything));
    }
    assert_eq!(said, ["Exp. Share", "KEY ITEMS", "A device."]);
}

#[test]
fn test_words_that_came_on_screen_are_details_of_that_move_alone() {
    // The PC Box's "Potion selected." came back as its menu's cursor moved, joined the move's
    // details, and was said again after every move of the menu (movie11).
    let mut screen = Screen::new();
    let r#move = vec![piece(1, "MOVE", 100.0, 50.0, MENU)];
    let bag = vec![piece(2, "BAG", 100.0, 20.0, MENU)];
    let words = later(3, "Potion selected.", -100.0, -100.0, PANEL, 0);
    let everything = cat(&[&r#move, &bag, &one(&words)]);
    let mut said = moved_and_settled(&mut screen, &one(&words), &r#move, &everything);
    said.extend(moved_and_settled(&mut screen, &[], &bag, &everything));
    assert_eq!(said, ["MOVE", "Potion selected.", "BAG"]);
}

#[test]
fn test_a_rewrite_s_first_row_written_again_is_said_with_the_rest() {
    // The Sell screen's count and price change together; a second press before the price was
    // said would have left the new count out.
    let times = "\u{00d7}";
    let mut screen = Screen::new();
    let shown = vec![
        piece_kw(
            1,
            &format!("{times}1"),
            -50.0,
            0.0,
            PANEL,
            Kw { parent: 2, ..PLAIN },
        ),
        piece_kw(3, "$150", 50.0, -20.0, PANEL, Kw { parent: 4, ..PLAIN }),
    ];
    screen.say(args(&shown, vec![], &shown));
    let two = vec![
        rewritten(&shown[0], &format!("{times}2")),
        rewritten(&shown[1], "$300"),
    ];
    let mut said = screen.say(args(&two, vec![], &two));
    let three = vec![
        rewritten(&shown[0], &format!("{times}3")),
        rewritten(&shown[1], "$450"),
    ];
    said.extend(screen.say(args(&three, vec![], &three)));
    for _ in 0..DETAIL_POLLS {
        said.extend(screen.on(&three));
    }
    assert_eq!(said, [format!("{times}2"), format!("{times}3 $450")]);
}

#[test]
fn test_a_cursor_moving_drops_the_rest_of_a_rewrite_unsaid() {
    // The Bag's cursor moved on while its panel was rewritten, and the rest, said as it stood,
    // read the next item's name with the last one's count (movie33).
    let mut screen = Screen::new();
    let shown = torchic_entry();
    screen.say(args(&shown, vec![], &shown));
    let now = entry("????? Pokémon", "1'04\"", ("007", "Mudkip"));
    let altered = altered_from(&shown, &now);
    let mut said = screen.say(args(&altered, vec![], &now));
    let row = vec![piece(30, "Ether", 80.0, 90.0, MENU)];
    let everything = cat(&[&now, &row]);
    said.extend(moved_and_settled(&mut screen, &[], &row, &everything));
    assert_eq!(said, ["007 Mudkip", "Ether"]);
}

#[test]
fn test_a_sentence_never_leads_a_rewrite() {
    // The Pokédex's other languages put the entry's description first, and the rewrite led with
    // it rather than the Pokémon's number and name (movie7).
    let mut screen = Screen::new();
    let words = piece(30, "Torchic sticks with its Trainer.", 0.0, 100.0, MENU);
    let shown = cat(&[&one(&words), &torchic_entry()]);
    screen.say(args(&shown, vec![], &shown));
    let now = cat(&[
        &one(&ScreenText {
            text: "At first sight, Poochyena takes a bite.".to_string(),
            ..words.clone()
        }),
        &entry("Bite Pokémon", "1'04\"", ("010", "Poochyena")),
    ]);
    let before: std::collections::HashMap<i64, String> =
        shown.iter().map(|p| (p.r#box, p.text.clone())).collect();
    let altered: Vec<ScreenText> = now
        .iter()
        .filter(|p| p.text != before[&p.r#box])
        .map(changed)
        .collect();
    assert_eq!(screen.say(args(&altered, vec![], &now)), ["010 Poochyena"]);
}

#[test]
fn test_the_rest_of_a_rewrite_never_cuts_off_its_first_row() {
    let mut screen = Screen::new();
    screen.announcer = ScreenAnnouncer::new(
        Rc::new(Speech::new(
            screen.backend.clone(),
            HashSet::from([Category::Key, Category::Menu]),
            None,
        )),
        Rc::new(|_line: &str| {}),
    );
    let shown = torchic_entry();
    screen.say(args(&shown, vec![], &shown));
    let now = entry("????? Pokémon", "1'04\"", ("007", "Mudkip"));
    let altered = altered_from(&shown, &now);
    screen.say(args(&altered, vec![], &now));
    for _ in 0..DETAIL_POLLS {
        screen.on(&now);
    }
    assert_eq!(screen.backend.interrupting(), ["007 Mudkip"]);
}

/// `subject=lambda: name`.
fn subject_of(name: &'static str) -> Option<Box<dyn Fn() -> Result<Option<String>>>> {
    Some(Box::new(move || Ok(Some(name.to_string()))))
}

#[test]
fn test_a_screen_rewritten_is_led_by_what_names_it() {
    // The summary's stats are on the top screen and built first, its Pokémon's name and level on
    // the bottom: the name leads (Orion, 2026-10-10).
    let mut screen = Screen::new();
    let stats = vec![
        piece(1, "HP", -100.0, 85.0, SUMMARY),
        piece(2, "19/19", 0.0, 85.0, SUMMARY),
    ];
    let name = vec![
        piece(3, "Umbra", -150.0, 110.0, MENU),
        piece(4, "Lv.5", -150.0, 90.0, MENU),
    ];
    let unchanged = |p: &ScreenText, text: &str| ScreenText {
        text: text.to_string(),
        ..p.clone()
    };
    let old = vec![
        stats[0].clone(),
        unchanged(&stats[1], "30/30"),
        unchanged(&name[0], "Blaze"),
        unchanged(&name[1], "Lv.10"),
    ];
    screen.say(args(&old, vec![], &old));
    let altered: Vec<ScreenText> = [&stats[1], &name[0], &name[1]]
        .into_iter()
        .map(changed)
        .collect();
    let everything = cat(&[&stats, &name]);
    let mut said = screen.say(Args {
        subject: subject_of("Umbra"),
        ..args(&altered, vec![], &everything)
    });
    for _ in 0..DETAIL_POLLS {
        said.extend(screen.say(Args {
            on_screen: everything.clone(),
            subject: subject_of("Umbra"),
            ..Args::default()
        }));
    }
    assert_eq!(said, ["Umbra", "Lv.5, HP 19/19"]);
}

#[test]
fn test_a_screen_rewritten_says_words_two_layouts_draw_alike_once() {
    // The summary keeps its other pages on screen, and both said "Name Poochyena".
    let mut screen = Screen::new();
    let one_page = vec![
        piece(1, "Name", -195.0, 60.0, SUMMARY),
        piece(2, "Torchic", -86.0, 60.0, SUMMARY),
    ];
    let two_page = vec![
        piece(3, "Name", -195.0, 60.0, PANEL),
        piece(4, "Torchic", -86.0, 60.0, PANEL),
    ];
    let both = cat(&[&one_page, &two_page]);
    screen.say(args(&both, vec![], &both));
    let now = vec![
        one_page[0].clone(),
        ScreenText {
            text: "Poochyena".to_string(),
            ..one_page[1].clone()
        },
        two_page[0].clone(),
        ScreenText {
            text: "Poochyena".to_string(),
            ..two_page[1].clone()
        },
    ];
    let mut said = screen.say(args(&[changed(&now[1]), changed(&now[3])], vec![], &now));
    for _ in 0..DETAIL_POLLS {
        said.extend(screen.on(&now));
    }
    assert_eq!(said, ["Name Poochyena"]);
}

#[test]
fn test_one_row_changing_alone_is_said_at_once() {
    let mut screen = Screen::new();
    let time = vec![
        piece(1, "Play time:", -100.0, 0.0, OPTIONS),
        piece(2, "0:13", 50.0, 0.0, OPTIONS),
    ];
    screen.say(args(&time, vec![], &time));
    let now = vec![time[0].clone(), rewritten(&time[1], "0:14")];
    assert_eq!(
        screen.say(args(&one(&now[1]), vec![], &now)),
        ["Play time: 0:14"]
    );
}

#[test]
fn test_a_rewrite_takes_in_words_coming_on_screen_a_poll_later() {
    // The Pokédex wrote its entry and then its description, said apart a poll later.
    let mut screen = Screen::new();
    let shown = torchic_entry();
    screen.say(args(&shown, vec![], &shown));
    let now = entry("Mud Fish Pokémon", "1'04\"", ("258", "Mudkip"));
    let altered = altered_from(&shown, &now);
    let words = later(30, "Mudkip's fin.", 0.0, -80.0, PANEL, 0);
    let mut said = screen.say(Args {
        settling: set(&[PANEL]),
        ..args(&altered, vec![], &now)
    });
    let everything = cat(&[&now, &one(&words)]);
    said.extend(screen.say(args(&one(&words), vec![], &everything)));
    said.extend(screen.on(&everything));
    said.extend(screen.on(&everything));
    assert_eq!(
        said,
        [
            "258 Mudkip",
            "Mud Fish Pokémon Female Times encountered: 1, Mudkip's fin."
        ]
    );
}

#[test]
fn test_a_rewrite_reads_every_panel_its_rewrites_have_changed() {
    // Two Pokémon of one height and weight leave the entry's table as it was, and it went
    // unsaid, as a move's power did between two moves of one power (2026-09-14).
    let mut screen = Screen::new();
    let mut shown = torchic_entry();
    screen.say(args(&shown, vec![], &shown));
    let mut said: Vec<String> = Vec::new();
    for now in [
        entry("Mud Fish Pokémon", "1'08\"", ("258", "Mudkip")),
        entry("Chick Pokémon", "1'08\"", ("255", "Torchic")),
    ] {
        let altered = altered_from(&shown, &now);
        said = screen.say(args(&altered, vec![], &now));
        for _ in 0..DETAIL_POLLS {
            said.extend(screen.on(&now));
        }
        shown = now;
    }
    assert_eq!(
        said,
        [
            "255 Torchic",
            "Chick Pokémon Female TYPE HEIGHT 1'08\" WEIGHT 5.5 lbs. Times encountered: 1"
        ]
    );
}

#[test]
fn test_two_cursors_over_one_item_say_it_once() {
    // A summary's battle page and contest page each keep a cursor, and both said the move
    // (movie32, the audit, 2026-10-10).
    let mut screen = Screen::new();
    let r#move = vec![piece(1, "Hold Back", -1.0, 100.0, MENU)];
    let said = screen.say(args(
        &[],
        vec![
            CursorItem::new(0x0C, r#move.clone(), false),
            CursorItem::new(0x0D, r#move.clone(), true),
        ],
        &r#move,
    ));
    assert_eq!(said, ["Hold Back"]);
}

/// A menu cursor holds an item still being laid out a poll, and the summary's battle page
/// said Hold Back a poll after its contest page had (or_summary_moves).
#[test]
fn test_two_cursors_over_one_item_a_poll_apart_say_it_once() {
    let mut screen = Screen::new();
    let r#move = vec![piece(1, "Hold Back", -1.0, 100.0, MENU)];
    assert_eq!(
        screen.say(args(
            &[],
            vec![CursorItem::new(0x0D, r#move.clone(), true)],
            &r#move
        )),
        ["Hold Back"]
    );
    assert_eq!(
        screen.say(args(
            &[],
            vec![CursorItem::new(0x0C, r#move.clone(), false)],
            &r#move
        )),
        Vec::<String>::new()
    );
}

#[test]
fn test_a_cursor_sliding_into_place_as_it_opens_is_said_once() {
    // The Bag's said "Pokéblock Kit" as it opened, and again 0.2 s later (movie30).
    let mut screen = Screen::new();
    let kit = vec![piece(1, "Pokéblock Kit", 80.0, 90.0, MENU)];
    assert_eq!(
        screen.say(args(
            &[],
            vec![CursorItem::new(0x0C, kit.clone(), true)],
            &kit
        )),
        ["Pokéblock Kit"]
    );
    assert_eq!(
        screen.say(args(
            &[],
            vec![CursorItem::new(0x0C, kit.clone(), false)],
            &kit
        )),
        NOTHING
    );
    let moved = vec![ScreenText {
        text: "Contest Costume".to_string(),
        ..kit[0].clone()
    }];
    assert_eq!(
        screen.say(args(
            &[],
            vec![CursorItem::new(0x0C, moved.clone(), false)],
            &moved
        )),
        ["Contest Costume"]
    );
}

#[test]
fn test_a_number_with_a_leading_zero_is_no_count() {
    // X's Pokédex numbers its rows "002", and taken for a count it was said after the entry's
    // first row, "SEEN 2 OBTAINED 1 002" (x/movie2, the audit, 2026-10-10).
    let mut screen = Screen::new();
    let row = vec![piece(1, "002", 80.0, 90.0, MENU)];
    assert_eq!(
        screen.say(args(
            &[],
            vec![CursorItem::new(0x0C, row.clone(), false)],
            &row
        )),
        ["002"]
    );
}

#[test]
fn test_what_a_cursor_said_a_moment_ago_is_known_to_have_been_said() {
    // The Mart's Buy, Sell and No thanks are read by the cursor beside them the poll the box is
    // shown, and by the choice reader only once a sweep has found its box: "Buy" was heard from
    // both (movie5, the user, 2026-09-25; spike 98).
    let mut screen = Screen::new();
    let buy = piece(1, "Buy", 130.0, 10.0, MENU);
    screen.say(args(
        &[],
        vec![CursorItem::new(0x0C, vec![buy.clone()], true)],
        &one(&buy),
    ));
    assert!(screen.announcer.said_by_cursor("Buy"));
    assert!(!screen.announcer.said_by_cursor("Sell"));
}

#[test]
fn test_what_a_cursor_said_a_while_ago_is_news_again() {
    let mut screen = Screen::new();
    let buy = piece(1, "Buy", 130.0, 10.0, MENU);
    screen.say(args(
        &[],
        vec![CursorItem::new(0x0C, vec![buy.clone()], true)],
        &one(&buy),
    ));
    for _ in 0..CURSOR_HEARD_POLLS + 1 {
        screen.on(&one(&buy));
    }
    assert!(!screen.announcer.said_by_cursor("Buy"));
}

/// A log that keeps its lines.
fn logging() -> (Log, Rc<RefCell<Vec<String>>>) {
    let logged: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let kept = logged.clone();
    let log: Log = Rc::new(move |line: &str| kept.borrow_mut().push(line.to_string()));
    (log, logged)
}

fn silent() -> Log {
    Rc::new(|_line: &str| {})
}

#[test]
fn test_an_option_opening_that_was_just_heard_is_logged_and_not_said() {
    let backend = Recorder::new();
    let (log, logged) = logging();
    let mut announcements = Announcements::new(Rc::new(Speech::plain(backend.clone())), log);
    announcements.focus(
        &Focus {
            source: 0x08803C84,
            label: "Buy".to_string(),
            index: 0,
            count: 3,
            opened: true,
        },
        "choice",
        true,
    );
    announcements.focus(
        &Focus {
            source: 0x08803C84,
            label: "Sell".to_string(),
            index: 1,
            count: 3,
            opened: false,
        },
        "choice",
        false,
    );
    assert_eq!(backend.spoken(), ["Sell"]);
    assert_eq!(
        *logged.borrow(),
        [
            "choice 0x08803c84 opened, just said by a cursor: Buy (1 of 3)",
            "choice 0x08803c84 moved: Sell (2 of 3)"
        ]
    );
}

#[test]
fn test_a_list_s_place_is_logged_and_never_said() {
    // Orion, 2026-09-30: no setting says a list's place any more.
    let backend = Recorder::new();
    let (log, logged) = logging();
    let mut announcements = Announcements::new(Rc::new(Speech::plain(backend.clone())), log);
    announcements.focus(
        &Focus {
            source: 0x08804680,
            label: "YES".to_string(),
            index: 0,
            count: 2,
            opened: true,
        },
        "list",
        false,
    );
    announcements.focus(
        &Focus {
            source: 0x08804680,
            label: "NO".to_string(),
            index: 1,
            count: 2,
            opened: false,
        },
        "list",
        false,
    );
    assert_eq!(backend.spoken(), ["YES", "NO"]);
    assert_eq!(
        *logged.borrow(),
        [
            "list 0x08804680 opened: YES (1 of 2)",
            "list 0x08804680 moved: NO (2 of 2)"
        ]
    );
}

#[test]
fn test_what_a_choice_box_said_is_its_own_for_a_moment_after() {
    // The cursor beside a choice box's words opened on "Yes" once the box had been answered and
    // its labels were gone, while its layout was still on screen (movie1, spike 99).
    let mut announcements = Announcements::new(Rc::new(Speech::plain(Recorder::new())), silent());
    announcements.focus(
        &Focus {
            source: 0x08804680,
            label: "Yes".to_string(),
            index: 0,
            count: 2,
            opened: true,
        },
        "choice",
        false,
    );
    assert_eq!(announcements.recent_labels(), labelled(&["Yes"]));
    for _ in 0..FOCUS_HEARD_POLLS + 1 {
        announcements.screen_polled();
    }
    assert_eq!(announcements.recent_labels(), HashSet::new());
}

#[test]
fn test_a_first_page_waits_a_moment_for_the_name_on_its_plate() {
    // Sun builds the talk window after its first page has begun, so the name came a moment after
    // the page (issue 4): the page is held until the name comes, or its window moves on, or
    // `SPEAKER_POLLS` have passed; a page after the first is not held.
    let names: Rc<RefCell<HashMap<i64, String>>> = Rc::new(RefCell::new(HashMap::new()));
    let backend = Recorder::new();
    let known = names.clone();
    let mut told = Announcements::with(
        Rc::new(Speech::plain(backend.clone())),
        silent(),
        Some(Rc::new(move |window| {
            Ok(known.borrow().get(&window).cloned())
        })),
        true,
    );
    let page = |window, index, text: &str| Shown::Page(PageShown::new(window, index, text));
    assert!(!told.pages(&[page(1, 0, "Hello")]).unwrap());
    assert_eq!(backend.spoken(), NOTHING);
    names.borrow_mut().insert(1, "Mom".to_string());
    assert!(told.pages(&[]).unwrap());
    assert_eq!(backend.spoken(), ["Mom: Hello"]);
    assert!(told.pages(&[page(1, 1, "Dear")]).unwrap());
    assert_eq!(backend.spoken(), ["Mom: Hello", "Dear"]);
    told.pages(&[page(2, 0, "Ring ring")]).unwrap();
    for _ in 0..SPEAKER_POLLS - 1 {
        assert!(!told.pages(&[]).unwrap());
    }
    assert!(told.pages(&[]).unwrap());
    assert_eq!(backend.spoken().last().unwrap(), "Ring ring");
    told.pages(&[page(3, 0, "One")]).unwrap();
    assert!(told.pages(&[page(3, 1, "Two")]).unwrap());
    let spoken = backend.spoken();
    assert_eq!(spoken[spoken.len() - 2..], ["One", "Two"]);
}

#[test]
fn test_each_side_is_kept_as_last_said_until_another_app_starts() {
    // R and shift and R say a side again (ADR-0015): what was last said for each panel, the
    // player's Pokémon together, and nothing once the battle is over.
    let backend = Recorder::new();
    let mut announcements = Announcements::new(Rc::new(Speech::plain(backend.clone())), silent());
    let side = |panel, text: &str, player| Battler {
        panel,
        text: text.to_string(),
        player,
    };
    assert_eq!(announcements.sides(true), None);
    announcements.battlers(&[
        side(1, "Blaze♂ L10 20/30", true),
        side(2, "Zigzagoon♂ L2 100%", false),
    ]);
    announcements.battlers(&[side(2, "Zigzagoon♂ L2 50%", false)]);
    assert_eq!(
        announcements.sides(true),
        Some("Blaze♂ L10 20/30".to_string())
    );
    assert_eq!(
        announcements.sides(false),
        Some("Zigzagoon♂ L2 50%".to_string())
    );
    announcements.battlers(&[side(3, "Umbra♂ L5 19/19", true)]);
    assert_eq!(
        announcements.sides(true),
        Some("Blaze♂ L10 20/30, Umbra♂ L5 19/19".to_string())
    );
    announcements.forget_sides();
    assert_eq!(announcements.sides(true), None);
    assert_eq!(announcements.sides(false), None);
}
