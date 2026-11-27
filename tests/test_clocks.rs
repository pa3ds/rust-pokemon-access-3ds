//! The clocks' tests: what of a screen's words ticks, turns or scores, and is left unsaid.

use std::cell::Cell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::games::common::clocks::{Clocks, SETTLE_POLLS, Scored};
use crate::games::common::screen::ScreenText;
use crate::py::Result;

const PANEL: i64 = 0x08200000;

/// `piece(box, text, x, y, parent, with_layout)`.
fn piece_at(r#box: i64, text: &str, x: f64, y: f64, parent: i64, with_layout: bool) -> ScreenText {
    ScreenText {
        left: x,
        right: x + (text.chars().count() * 6) as f64,
        row: y,
        glyph: 12.5,
        root: PANEL,
        parent: if parent != 0 { parent } else { PANEL },
        with_layout,
        ..ScreenText::new(r#box, text, false)
    }
}

/// `piece(box, text)`: at (-100, 100), hanging from the root, come with its layout.
fn piece(r#box: i64, text: &str) -> ScreenText {
    piece_at(r#box, text, -100.0, 100.0, 0, true)
}

/// `replace(p, text=text, changed=True, with_layout=False)`.
fn changed_to(p: &ScreenText, text: &str) -> ScreenText {
    ScreenText {
        text: text.to_string(),
        changed: true,
        with_layout: false,
        ..p.clone()
    }
}

/// `clocks(tick, timed, playing, scored, score_said)`.
fn clocks_with(
    tick: Option<Rc<dyn Fn()>>,
    timed: bool,
    playing: Option<Rc<dyn Fn() -> Result<bool>>>,
    scored: Option<Scored>,
    score_said: bool,
) -> Polls {
    Polls {
        clocks: Clocks::with(
            Rc::new(|_line: &str| {}),
            tick,
            Some(Rc::new(move || timed)),
            playing,
            scored,
            score_said,
        ),
        polls: 0,
        opened_at: IndexMap::new(),
    }
}

/// `clocks(timed=timed)`.
fn clocks(timed: bool) -> Polls {
    clocks_with(None, timed, None, None, false)
}

/// A tick counter, and the `tick` that counts.
fn counting() -> (Rc<Cell<i64>>, Option<Rc<dyn Fn()>>) {
    let ticks = Rc::new(Cell::new(0));
    let counted = ticks.clone();
    (ticks, Some(Rc::new(move || counted.set(counted.get() + 1))))
}

/// Clocks asked once a poll, as the announcer asks them: of a poll's words, all of them screen
/// text's, and the poll each layout's words came on screen at, as it was the poll before.
struct Polls {
    clocks: Clocks,
    polls: i64,
    opened_at: IndexMap<i64, i64>,
}

impl Polls {
    /// The words of `texts` left to be said.
    fn ask(&mut self, texts: &[ScreenText], on_screen: &[ScreenText], turns: bool) -> Vec<String> {
        self.polls += 1;
        let unsaid = self
            .clocks
            .unsaid(texts, texts, &self.opened_at, turns)
            .unwrap();
        let mut opened_at: IndexMap<i64, i64> = IndexMap::new();
        for p in on_screen {
            let at = self.opened_at.get(&p.root).copied().unwrap_or(self.polls);
            opened_at.insert(p.root, at);
        }
        self.opened_at = opened_at;
        texts
            .iter()
            .filter(|p| !unsaid.contains(&p.r#box))
            .map(|p| p.text.clone())
            .collect()
    }

    /// `ask(texts, on_screen)`.
    fn on(&mut self, texts: &[ScreenText], on_screen: &[ScreenText]) -> Vec<String> {
        self.ask(texts, on_screen, false)
    }
}

/// What is left said of a clock shown with its first words and changed to each of the rest, a
/// change after each of `gaps` polls.
fn run_clock(polls: &mut Polls, texts: &[String], gaps: &[i64]) -> Vec<String> {
    let mut first = piece(1, &texts[0]);
    let mut said = polls.on(std::slice::from_ref(&first), std::slice::from_ref(&first));
    assert_eq!(gaps.len(), texts.len() - 1);
    for (text, &gap) in texts[1..].iter().zip(gaps) {
        for _ in 0..gap - 1 {
            said.extend(polls.on(&[], std::slice::from_ref(&first)));
        }
        first = changed_to(&first, text);
        said.extend(polls.on(std::slice::from_ref(&first), std::slice::from_ref(&first)));
    }
    said
}

/// `run_clock` with a change every 5 polls, its default.
fn run_clock_5(polls: &mut Polls, texts: &[String]) -> Vec<String> {
    run_clock(polls, texts, &vec![5; texts.len() - 1])
}

fn words(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|text| text.to_string()).collect()
}

/// Berry Picker said "Time Left" every second for its whole minute (a player, 2026-10-26).
#[test]
fn test_a_countdown_is_said_each_half_minute_at_10_and_its_last_5() {
    let (ticks, tick) = counting();
    let countdown: Vec<String> = (0..=60)
        .rev()
        .map(|left| format!("Time Left: {left:02}"))
        .collect();
    let said = run_clock_5(&mut clocks_with(tick, true, None, None, false), &countdown);
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
    assert_eq!(ticks.get(), 60);
}

#[test]
fn test_a_clock_counting_up_is_said_each_half_minute() {
    let up: Vec<String> = (28..33)
        .map(|seconds| format!("Time: 00:{seconds:02}"))
        .collect();
    assert_eq!(
        run_clock_5(&mut clocks(true), &up),
        ["Time: 00:28", "Time: 00:30"]
    );
}

/// Berry Picker's replay read 41 then 39 two seconds later, and 10 then 9 two polls after
/// (as_amie_berry_picker); 31 to 29 passes the half minute, which is said.
#[test]
fn test_a_clock_still_ticks_past_a_missed_second_and_after_a_late_one() {
    let said = run_clock(
        &mut clocks(true),
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
        &mut clocks(true),
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

/// Tile Puzzle's Unlimited gives a second back for a piece put home, and said "Time Remaining:
/// 01:00" twice in a second (or_amie_tile_puzzle_unlimited). A clock jumping, as a finished
/// puzzle's 28 seconds won back do, passes its marks afresh.
#[test]
fn test_a_mark_is_said_once_as_a_second_won_back_bounces_the_clock_round_it() {
    let bouncing: Vec<String> = ["01:02", "01:01", "01:00", "01:01", "01:00", "00:59"]
        .iter()
        .map(|seconds| format!("Time Remaining: {seconds}"))
        .collect();
    let said = run_clock(&mut clocks(true), &bouncing, &[5, 5, 3, 3, 5]);
    assert_eq!(said, ["Time Remaining: 01:02", "Time Remaining: 01:00"]);
    let jumping: Vec<String> = [31, 30]
        .into_iter()
        .chain((30..=58).rev())
        .map(|seconds| format!("Time Remaining: 00:{seconds}"))
        .collect();
    assert_eq!(
        run_clock_5(&mut clocks(true), &jumping),
        [
            "Time Remaining: 00:31",
            "Time Remaining: 00:30",
            "Time Remaining: 00:58",
            "Time Remaining: 00:30"
        ]
    );
}

/// At each piece put home Unlimited said its total and the gain it popped up, "Puzzle No. 2, 102
/// Points", "+2 Points" (or_amie_tile_puzzle_unlimited): nothing is said of a swap, the total only
/// as a puzzle starts and once it is solved. Words are said.
#[test]
fn test_numbers_are_the_plays_while_the_player_plays() {
    let playing = Rc::new(Cell::new(false));
    let asked = playing.clone();
    let mut polls = clocks_with(
        None,
        true,
        Some(Rc::new(move || Ok(asked.get()))),
        None,
        false,
    );
    let puzzle = piece_at(1, "Puzzle No. 2", -100.0, 20.0, 5, true);
    let total = piece_at(2, "100 Points", 0.0, 20.0, 5, true);
    let both = [puzzle.clone(), total.clone()];
    assert_eq!(polls.on(&both, &both), ["Puzzle No. 2", "100 Points"]);
    playing.set(true);
    let more = changed_to(&total, "102 Points");
    let gain = [
        piece_at(3, "+", 44.0, 0.0, 0, false),
        piece_at(5, "2 Points", 50.0, 0.0, 0, false),
    ];
    let question = piece_at(4, "Quit the puzzle?", 0.0, -50.0, 0, true);
    let texts = [
        more.clone(),
        gain[0].clone(),
        gain[1].clone(),
        question.clone(),
    ];
    let on_screen = [
        puzzle.clone(),
        more.clone(),
        gain[0].clone(),
        gain[1].clone(),
        question,
    ];
    assert_eq!(polls.on(&texts, &on_screen), ["Quit the puzzle?"]);
    playing.set(false);
    let solved = ScreenText {
        text: "202 Points".to_string(),
        ..more
    };
    assert_eq!(
        polls.on(std::slice::from_ref(&solved), &[puzzle, solved.clone()]),
        ["202 Points"]
    );
}

/// Head It's score, combo and "MISS!" came at every ball, a second apart, and queued behind the
/// balls' names; its combo, one up each second, ticked as a clock and was said at 30 (spike 273's
/// `--again`). Its chances left are said (players, 2026-11-04).
#[test]
fn test_a_games_score_is_left_unsaid_and_is_never_a_clock() {
    let (ticks, tick) = counting();
    let mut polls = clocks_with(
        tick,
        true,
        None,
        Some(Rc::new(|p: &ScreenText| Ok(p.r#box == 1 || p.r#box == 2))),
        false,
    );
    let combos: Vec<String> = (28..33).map(|hits| format!("{hits}-HIT COMBO")).collect();
    assert_eq!(run_clock_5(&mut polls, &combos), Vec::<String>::new());
    let missed = piece_at(2, "MISS!", 0.0, 0.0, 0, true);
    let chances = piece_at(3, "Chances:2", -100.0, 80.0, 0, true);
    let both = [missed, chances];
    assert_eq!(polls.on(&both, &both), ["Chances:2"]);
    assert_eq!(ticks.get(), 0);
}

/// `[pa3ds.amie] head_it_score`: the combo once while hits keep coming within 3 s, and again
/// after a pause or once it starts over.
#[test]
fn test_a_games_score_asked_for_is_said_once_while_it_keeps_going_up() {
    let (ticks, tick) = counting();
    let mut polls = clocks_with(
        tick,
        true,
        None,
        Some(Rc::new(|p: &ScreenText| Ok(p.r#box == 1))),
        true,
    );
    let said = run_clock(
        &mut polls,
        &words(&[
            "3-HIT COMBO",
            "4-HIT COMBO",
            "5-HIT COMBO",
            "6-HIT COMBO",
            "3-HIT COMBO",
        ]),
        &[5, 5, 20, 5],
    );
    assert_eq!(said, ["3-HIT COMBO", "6-HIT COMBO", "3-HIT COMBO"]);
    assert_eq!(ticks.get(), 0);
}

/// Taken for a clock wherever it ticked, the battle Bag's page, the Mart's counts, money, a save
/// screen's minute and the PC Box's name went unsaid as the player changed them
/// (as_cave_of_origin, or_mart, or_pc_box_buttons and others).
#[test]
fn test_no_number_is_a_clock_outside_a_game_that_times_the_player() {
    let countdown: Vec<String> = (56..=60)
        .rev()
        .map(|left| format!("Time Left: {left}"))
        .collect();
    assert_eq!(run_clock_5(&mut clocks(false), &countdown), countdown);
}

/// O-Power's Active panel counts down each power running, its time a TextBox of its own, and each
/// second of up to four was read (the first or_pss_o_power, archived). Its half minute is no mark
/// either: R says the times. Back from a power used, it came with the times it had left, then
/// jumped them two seconds a poll later and brought one back alone: "Prize Money Power Lv. 1
/// 02:57", "02:43".
#[test]
fn test_a_clock_alone_outside_a_game_is_said_only_as_its_layout_comes() {
    let (ticks, tick) = counting();
    let countdown: Vec<String> = (28..=33)
        .rev()
        .map(|left| format!("02:{left:02}"))
        .collect();
    let said = run_clock_5(&mut clocks_with(tick, false, None, None, false), &countdown);
    assert_eq!(said, ["02:33"]);
    assert_eq!(ticks.get(), 0);
    // Its first tick two seconds on: a row came at "02:59" and next read "02:57".
    assert_eq!(
        run_clock(
            &mut clocks(false),
            &words(&["02:59", "02:57", "02:56"]),
            &[10, 5]
        ),
        ["02:59"]
    );
    let mut polls = clocks(false);
    let power = [
        piece_at(1, "Prize Money Power", -100.0, 60.0, 5, true),
        piece_at(2, "Lv. 1", 20.0, 60.0, 5, true),
        piece_at(3, "02:59", 80.0, 60.0, 5, true),
    ];
    assert_eq!(
        polls.on(&power, &power),
        ["Prize Money Power", "Lv. 1", "02:59"]
    );
    let jumped = changed_to(&power[2], "02:57");
    let back = piece_at(4, "02:43", 80.0, 30.0, 6, false);
    let on_screen = [
        power[0].clone(),
        power[1].clone(),
        jumped.clone(),
        back.clone(),
    ];
    assert_eq!(polls.on(&[jumped, back], &on_screen), Vec::<String>::new());
}

/// A row of O-Power's Active panel coming later in its first second ticked first two polls on,
/// too soon for a clock's beat, and "Encounter Power Lv. 1 02:27" was said again
/// (the first or_pss_o_power, archived). Its clock settles unsaid, and is a clock from then on.
#[test]
fn test_a_clock_alone_settled_in_its_layouts_first_second_ticks_at_any_pace() {
    let mut polls = clocks(false);
    let power = vec![
        piece_at(1, "Prize Money Power", -100.0, 60.0, 5, true),
        piece_at(2, "Lv. 1", 20.0, 60.0, 5, true),
        piece_at(3, "02:59", 80.0, 60.0, 5, true),
    ];
    let mut said = polls.on(&power, &power);
    for _ in 0..SETTLE_POLLS.end - 3 {
        said.extend(polls.on(&[], &power));
    }
    let mut late = vec![
        piece_at(5, "Encounter Power", -100.0, 30.0, 6, false),
        piece_at(6, "Lv. 1", 20.0, 30.0, 6, false),
        piece_at(7, "02:28", 80.0, 30.0, 6, false),
    ];
    let all = |late: &[ScreenText]| -> Vec<ScreenText> { [power.clone(), late.to_vec()].concat() };
    said.extend(polls.on(&late, &all(&late)));
    said.extend(polls.on(&[], &all(&late)));
    late[2] = ScreenText {
        text: "02:27".to_string(),
        changed: true,
        ..late[2].clone()
    };
    said.extend(polls.on(std::slice::from_ref(&late[2]), &all(&late)));
    assert_eq!(
        said,
        [
            "Prize Money Power",
            "Lv. 1",
            "02:59",
            "Encounter Power",
            "Lv. 1"
        ]
    );
}

/// The PSS's profile turns between "Total number of people met: 4" and "Number of people met
/// today: 0" every 3.4 s, its count a poll or two after its label, and each turn was read
/// (or_pss_singleplayer_2). A row turned back off that pace is said: a summary turned back to a
/// Pokémon by the player.
#[test]
fn test_a_row_turned_back_and_forth_by_itself_is_said_once_each_way() {
    let turn = |turns: bool, gaps: &[i64]| -> Vec<String> {
        let mut polls = clocks(false);
        let mut rows = vec![
            piece_at(1, "Total number of people met:", -150.0, 0.0, 5, true),
            piece_at(2, "4", 50.0, 0.0, 5, true),
        ];
        let mut said = polls.ask(&rows, &rows, turns);
        let words = [
            "Total number of people met:",
            "4",
            "Number of people met today:",
            "0",
        ];
        for (i, &gap) in gaps.iter().enumerate() {
            let i = i + 1;
            let mut gap = gap;
            for r#box in [0, 1] {
                for _ in 0..gap - 1 {
                    said.extend(polls.ask(&[], &rows, turns));
                }
                rows[r#box] = changed_to(&rows[r#box], words[2 * (i % 2) + r#box]);
                said.extend(polls.ask(std::slice::from_ref(&rows[r#box]), &rows, turns));
                gap = 2;
            }
        }
        said
    };
    let once = [
        "Total number of people met:",
        "4",
        "Number of people met today:",
        "0",
    ];
    let again = ["Total number of people met:", "4"];
    assert_eq!(turn(true, &[17, 16, 17, 16]), once);
    assert_eq!(turn(false, &[17, 16]), [once.as_slice(), &again].concat());
    assert_eq!(turn(true, &[17, 30]), [once.as_slice(), &again].concat());
}

/// O-Power's list hides its description under Use, Give and Cancel, and the next power picked
/// brings it back with "Required Energy" as before: at the pace of a turn, that row went unsaid
/// with the new power said (y_pss_singleplayer). A page turned back comes back whole.
#[test]
fn test_a_row_back_beside_new_words_is_no_turn() {
    let mut polls = clocks(false);
    let mut rows = vec![
        piece_at(1, "Capture Power Lv. 1", -150.0, 40.0, 5, true),
        piece_at(2, "Required Energy", -150.0, 0.0, 5, true),
    ];
    let mut said = polls.ask(&rows, &rows, true);
    for words in [["", ""], ["HP Restoring Power Lv. 1", "Required Energy"]] {
        for _ in 0..16 {
            said.extend(polls.ask(&[], &rows, true));
        }
        rows = rows
            .iter()
            .zip(words)
            .map(|(row, text)| changed_to(row, text))
            .collect();
        said.extend(polls.ask(&rows, &rows, true));
    }
    assert_eq!(
        said[said.len() - 2..],
        ["HP Restoring Power Lv. 1", "Required Energy"]
    );
}

/// A score going up by one a poll after it came, or a play time a minute after, is no clock.
#[test]
fn test_a_number_changing_by_one_off_a_seconds_beat_is_said() {
    assert_eq!(
        run_clock(&mut clocks(true), &words(&["SCORE 7", "SCORE 8"]), &[1]),
        ["SCORE 7", "SCORE 8"]
    );
    assert_eq!(
        run_clock(
            &mut clocks(true),
            &words(&["Play time: 14", "Play time: 15"]),
            &[300]
        ),
        ["Play time: 14", "Play time: 15"]
    );
}

/// Where no game times anything, Sun's, the clocks say no game runs that does.
#[test]
fn test_clocks_of_no_game_time_nothing() {
    assert!(!Clocks::new(Rc::new(|_line: &str| {})).timed());
    assert!(clocks(true).clocks.timed());
}
