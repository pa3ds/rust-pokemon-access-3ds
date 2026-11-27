//! What of a screen's words ticks, turns or scores: a timed game's clock, a clock alone, a row the
//! screen turns by itself and a game's score, left unsaid by screen text (`ScreenAnnouncer.say`), by
//! these rules:
//!
//! - **A clock is said at its marks**: while a game that times the player runs,
//!   Pokémon-Amie's three, a number going one up or down a second after it last changed is its clock
//!   ticking, said each half minute and, counting down, at 10 and each of the last 5, once while it
//!   bounces round one. They said "Time Left" every second for a minute. Every tick can sound a
//!   tone, `[pa3ds.audio] ticks`. While the player plays, a Tile Puzzle with a piece away from
//!   home, its numbers are the play's, unsaid but for the clock's marks.
//! - **A clock alone is said as its layout comes**: outside such a game, words that are minutes and
//!   seconds alone, O-Power's "02:58", are left unsaid as they tick and as they settle in their
//!   layout's first second.
//! - **A row a screen turns back and forth by itself is said once each way**, where the screen is
//!   known to: the PSS's profile turns between its counts of people met every 3.4 s (`Heard.turns`).
//!   A row back beside new words in its layout is no turn.
//! - **A game's score is the player's to ask for** (players, 2026-11-04): Head It's score,
//!   combo and "MISS!", which come at every ball, faster than they can be said, are left unsaid and
//!   never taken for a clock, its combo one up a second apart; where the player asks to hear them,
//!   each is said and then left unsaid while it keeps going up within 3 s. Its misses left, counting
//!   down, and its clock are said as ever.
//!
//! They are asked in one order, each of the words the one before leaves (`Clocks.unsaid`): what is
//! a game's score, a clock's ticks, a clock alone settling, a row turned, the numbers of play, and
//! the score left unsaid.

use std::collections::HashSet;
use std::ops::Range;
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::IndexMap;
use regex::Regex;

use crate::games::common::screen::ScreenText;
use crate::py::{self, Log, Result};

/// Polls from one tick of a clock to the next: a second is five, and Berry Picker's "Time Left"
/// was caught 0.82 to 1.05 s apart; a slow poll moves it a little either way.
pub const CLOCK_POLLS: Range<i64> = 3..9;
/// Words that are a clock alone, minutes and seconds: O-Power's "02:58". Python's `fullmatch`:
/// anchored at both ends.
pub static CLOCK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:\d+:\d\d)$").expect("a fixed pattern"));
/// Polls after its layout came on screen, its first second, within which a clock alone settles
/// unsaid, as long as words arriving wait for the rest of their layout's
/// (`screen_announce.ARRIVAL_MAX_POLLS`).
pub const SETTLE_POLLS: Range<i64> = 1..5;

/// The most polls between two turns of a row a screen turns by itself: the PSS's profile turns
/// between "Total number of people met" and "Number of people met today" every 3.2 to 3.8 s.
pub const TURN_MAX_POLLS: i64 = 50;
/// Polls a turn may come early or late on the turn before, at least: a quarter of the turn
/// before where that is more. The profile's count turned 19 polls, then 16, after the one before.
pub const TURN_SLACK_POLLS: i64 = 2;

static NUMBERS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d+").expect("a fixed pattern"));
/// Polls, three seconds, within which a score heard again higher is left unsaid, where the player
/// asks for a game's score as it comes (`score_said`).
pub const AGAIN_POLLS: i64 = 15;

/// `Callable[[ScreenText], bool]`: whether words are a game's score (`Clocks`' `scored`). It
/// reads the game, so it can raise.
pub type Scored = Rc<dyn Fn(&ScreenText) -> Result<bool>>;

/// What of a screen's words ticks, turns or scores, left unsaid, one poll at a time.
pub struct Clocks {
    log: Log,
    tick: Option<Rc<dyn Fn()>>,
    timed: Option<Rc<dyn Fn() -> bool>>,
    playing: Option<Rc<dyn Fn() -> Result<bool>>>,
    scored: Option<Scored>,
    score_said: bool,
    /// Polls so far, a poll each time `unsaid` is asked.
    polls: i64,
    /// A score's words -> the poll they were last heard at, and their first number then.
    numbered: IndexMap<i64, (i64, i64)>,
    /// Clock's TextBox -> the mark it was said at last, said once while it bounces round it.
    marks: IndexMap<i64, i64>,
    /// TextBox -> the words it came on screen with or changed to lately, and the poll, for
    /// telling a clock's tick (`_ticked`).
    set: IndexMap<i64, (String, i64)>,
    /// TextBoxes that have ticked, for as long as they keep being set.
    ticking: HashSet<i64>,
    /// TextBox -> the last two words it came on screen with or changed to, each with its poll,
    /// while a screen may turn its rows by itself (`_turned`).
    writes: IndexMap<i64, Vec<(String, i64)>>,
}

impl Clocks {
    /// `Clocks(log)`: no game's, which leave clocks alone and rows turned unsaid all the same.
    pub fn new(log: Log) -> Self {
        Clocks::with(log, None, None, None, None, false)
    }

    /// `timed` says whether a game that times the player runs, whose clock is said at its
    /// marks (`_ticked`), none where it is None; `tick` sounds each tick of the clock; `playing`
    /// whether its player is in the middle of play, whose numbers are left unsaid; `scored`
    /// whether words are a game's score, left unsaid unless `score_said`, and then unsaid when
    /// heard again higher within `AGAIN_POLLS`. With none of them, clocks alone and rows turned
    /// are still left unsaid, as they are in any game.
    ///
    /// `playing` and `scored` return `Result`: they read the game.
    pub fn with(
        log: Log,
        tick: Option<Rc<dyn Fn()>>,
        timed: Option<Rc<dyn Fn() -> bool>>,
        playing: Option<Rc<dyn Fn() -> Result<bool>>>,
        scored: Option<Scored>,
        score_said: bool,
    ) -> Self {
        Clocks {
            log,
            tick,
            timed,
            playing,
            scored,
            score_said,
            polls: 0,
            numbered: IndexMap::new(),
            marks: IndexMap::new(),
            set: IndexMap::new(),
            ticking: HashSet::new(),
            writes: IndexMap::new(),
        }
    }

    /// Whether a game that times the player runs: every word coming back then comes alone
    /// (`ScreenAnnouncer._with_what_stayed`).
    pub fn timed(&self) -> bool {
        self.timed.as_ref().is_some_and(|timed| timed())
    }

    /// The boxes of one poll's words left unsaid as ticking, turning, played or scored:
    /// `heard` is what screen text would say of them, `texts` every word that came on screen or
    /// changed, `opened_at` the poll each layout's words came on screen at, polls counted as
    /// this is asked, once a poll, and `turns` whether the screen turns rows by itself
    /// (`Heard.turns`). Each is asked of the words the one before leaves, in the order the
    /// module's docstring writes, and logged as it is left.
    pub fn unsaid(
        &mut self,
        heard: &[ScreenText],
        texts: &[ScreenText],
        opened_at: &IndexMap<i64, i64>,
        turns: bool,
    ) -> Result<HashSet<i64>> {
        self.polls += 1;
        let timed = self.timed();
        let mut heard: Vec<ScreenText> = heard.to_vec();
        // Head It's combo, one up a second apart, passed for a clock: `ticks` sounded at every
        // hit, and a combo of 30 was said as a clock's mark (spike 273's `--again`).
        let mut scored: HashSet<i64> = HashSet::new();
        for p in &heard {
            if let Some(is_score) = &self.scored
                && is_score(p)?
            {
                scored.insert(p.r#box);
            }
        }
        let mut steps: IndexMap<i64, bool> = IndexMap::new();
        for p in &heard {
            if (timed || CLOCK.is_match(&p.text))
                && p.changed
                && !scored.contains(&p.r#box)
                && let Some(tick) = self.ticked(p, timed)?
            {
                steps.insert(p.r#box, tick);
            }
        }
        let mut ticks: HashSet<i64> = steps
            .iter()
            .filter(|(_, tick)| **tick)
            .map(|(&r#box, _)| r#box)
            .collect();
        // And settles with its layout: O-Power's Active panel, back from a power used, came with
        // the times it had left, then jumped them a second or two a poll later and brought one
        // back alone, "Prize Money Power Lv. 1 02:57", "02:43" (or_pss_o_power). A clock from
        // then on: a row coming later in that second ticked first two polls on, too soon for a
        // clock's beat, and said "Encounter Power Lv. 1 02:27" again.
        // And comes back with a page the screen turns to by itself: the Active panel hides its
        // times with its battle powers and shows them again with its field powers, its rows
        // turned unsaid and "02:24" said alone (or_pss_o_power_2).
        let settled: HashSet<i64> = if timed {
            HashSet::new()
        } else {
            heard
                .iter()
                .filter(|p| {
                    let since = opened_at.get(&p.root).copied().unwrap_or(self.polls);
                    CLOCK.is_match(&p.text)
                        && !ticks.contains(&p.r#box)
                        && (SETTLE_POLLS.contains(&(self.polls - since))
                            || (turns && !p.changed && !p.with_layout))
                })
                .map(|p| p.r#box)
                .collect()
        };
        for p in &heard {
            if settled.contains(&p.r#box) {
                (self.log)(&format!(
                    "screen ticked: {}  [layout {:#x}]",
                    p.text, p.root
                ));
            }
        }
        heard.retain(|p| !settled.contains(&p.r#box));
        ticks.extend(settled.iter().copied());
        self.ticking.extend(settled.iter().copied());
        let turned = self.turned(texts, turns);
        // A page turned back comes back whole, but for its clocks: O-Power's list brought its
        // description back with the next power picked, "Required Energy" as before, at the pace
        // of a turn, and left it unsaid (y_pss_singleplayer).
        let fresh: HashSet<i64> = texts
            .iter()
            .filter(|p| !turned.contains(&p.r#box) && !ticks.contains(&p.r#box))
            .map(|p| p.root)
            .collect();
        let turned: HashSet<i64> = texts
            .iter()
            .filter(|p| turned.contains(&p.r#box) && !fresh.contains(&p.root))
            .map(|p| p.r#box)
            .collect();
        for p in &heard {
            if turned.contains(&p.r#box) {
                (self.log)(&format!(
                    "screen turned: {}  [layout {:#x}]",
                    p.text, p.root
                ));
            }
        }
        heard.retain(|p| !turned.contains(&p.r#box));
        ticks.extend(turned.iter().copied());
        // Kept for two beats, so a clock whose second the reader missed still ticks.
        let polls = self.polls;
        self.set
            .retain(|_box, set_at| polls - set_at.1 < 2 * CLOCK_POLLS.end);
        for p in texts {
            self.set.insert(p.r#box, (p.text.clone(), self.polls));
        }
        let set = &self.set;
        self.ticking.retain(|r#box| set.contains_key(r#box));
        let ticking = &self.ticking;
        self.marks.retain(|r#box, _| ticking.contains(r#box));
        // While the player plays, the numbers are the play's but for the clock's marks, and the
        // signs beside them: at each piece put home Unlimited said its total and the gain it
        // popped up, "Puzzle No. 2, 102 Points", "+2 Points", its "+" a TextBox of its own, and
        // the second it gave back, "Time Remaining: 00:30" (or_amie_tile_puzzle_unlimited).
        // Nothing is said of a swap, the total as a puzzle starts and once it is solved.
        // The last swap's are said: the board lies solved 0.4 s before its total
        // changes, and the results come a second after that.
        let played: HashSet<i64> = heard
            .iter()
            .filter(|p| {
                !steps.contains_key(&p.r#box)
                    && (NUMBERS.is_match(&p.text) || !p.text.chars().any(py::is_alpha))
            })
            .map(|p| p.r#box)
            .collect();
        if timed && !played.is_empty() {
            let playing = match &self.playing {
                Some(playing) => playing()?,
                None => false,
            };
            if playing {
                (self.log)(&format!(
                    "screen played, unsaid: {}",
                    heard
                        .iter()
                        .filter(|p| played.contains(&p.r#box))
                        .map(|p| p.text.as_str())
                        .collect::<Vec<&str>>()
                        .join(", ")
                ));
                heard.retain(|p| !played.contains(&p.r#box));
                ticks.extend(played.iter().copied());
            }
        }
        // Head It's score, "79-HIT COMBO" and "MISS!" came at every ball, a second apart, and
        // queued behind the balls' names, which came later and later (players, 2026-11-04; spike
        // 273's `--again`). Asked for as they come, a score is said once and not while it keeps
        // going up.
        let mut unsaid = scored.clone();
        if !scored.is_empty() && self.score_said {
            let mut numbered: IndexMap<i64, i64> = IndexMap::new();
            for p in &heard {
                if scored.contains(&p.r#box)
                    && let Some(found) = NUMBERS.find(&p.text)
                {
                    numbered.insert(p.r#box, py::int_base10(found.as_str())?);
                }
            }
            unsaid = numbered
                .iter()
                .filter(|&(r#box, &number)| {
                    self.numbered
                        .get(r#box)
                        .is_some_and(|last| self.polls - last.0 < AGAIN_POLLS && number > last.1)
                })
                .map(|(&r#box, _)| r#box)
                .collect();
            for (&r#box, &number) in &numbered {
                self.numbered.insert(r#box, (self.polls, number));
            }
        }
        if !unsaid.is_empty() {
            (self.log)(&format!(
                "screen scored, unsaid: {}",
                heard
                    .iter()
                    .filter(|p| unsaid.contains(&p.r#box))
                    .map(|p| p.text.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            ));
            ticks.extend(unsaid.iter().copied());
        }
        Ok(ticks)
    }

    /// The boxes of `texts` back to the words they had before their last change, about as
    /// long after it as it came after the one before (`Heard.turns`).
    fn turned(&mut self, texts: &[ScreenText], turns: bool) -> HashSet<i64> {
        if !turns {
            self.writes.clear();
            return HashSet::new();
        }
        let polls = self.polls;
        self.writes.retain(|_box, writes| {
            polls - writes.last().expect("a write each").1 <= TURN_MAX_POLLS
        });
        let mut turned: HashSet<i64> = HashSet::new();
        for piece in texts {
            let writes = self.writes.get(&piece.r#box).cloned().unwrap_or_default();
            if let [(words, at), (_, last)] = writes.as_slice() {
                let (pace, gap) = (last - at, self.polls - last);
                if *words == piece.text
                    && (gap - pace).abs() <= TURN_SLACK_POLLS.max(py::floordiv(pace, 4))
                {
                    turned.insert(piece.r#box);
                }
            }
            let mut kept: Vec<(String, i64)> = writes.last().cloned().into_iter().collect();
            kept.push((piece.text.clone(), self.polls));
            self.writes.insert(piece.r#box, kept);
        }
        turned
    }

    /// Is a changed piece a clock's tick, left unsaid (True), or one at a mark, said
    /// (False)? None for no tick. Its number has gone one up or down a second after the words
    /// were last set, and nothing else of them has changed: "Time Left: 58", "Time: 00:10".
    /// Asked only while a game that times the player runs (`timed`). Once a clock, a step of
    /// two, a second the reader missed, or one coming soon after a late one, is a tick too:
    /// Berry Picker went 41 then 39, and 10 then 9 at 0.4 s. Every tick sounds `tick`; one
    /// reaching or passing a mark is said, but a mark said last, as a second won back bounces
    /// the clock round it: Unlimited said "Time Remaining: 01:00" twice in a second
    /// (or_amie_tile_puzzle_unlimited). A clock changing on no tick passes its marks afresh.
    ///
    /// Outside such a game, only words that are a clock alone, minutes and seconds, are asked,
    /// and every tick is left unsaid with no sound and no mark: O-Power's Active panel counts
    /// down each power running, "02:58", and read each second of up to four at once
    /// (or_pss_o_power); R says them (`gen6.subscreen.o_power`).
    ///
    /// Python's `timed` defaults to True.
    fn ticked(&mut self, piece: &ScreenText, timed: bool) -> Result<Option<bool>> {
        let Some(before) = self.set.get(&piece.r#box).cloned() else {
            return Ok(None);
        };
        let (now, then) = (seconds(&piece.text)?, seconds(&before.0)?);
        let (Some(now), Some(then)) = (now, then) else {
            return Ok(None);
        };
        if NUMBERS.replace_all(&piece.text, "#") != NUMBERS.replace_all(&before.0, "#") {
            return Ok(None);
        }
        let step = (now - then).abs();
        // Words a clock alone are one from their first tick, two seconds on two seconds after: the
        // Active panel's row came at "02:59" and ticked first to "02:57" (or_pss_o_power).
        let gap = self.polls - before.1;
        if !((step == 1 && CLOCK_POLLS.contains(&gap))
            || (self.ticking.contains(&piece.r#box) && 0 < step && step <= 2)
            || (!timed && step == 2 && (CLOCK_POLLS.start..2 * CLOCK_POLLS.end).contains(&gap)))
        {
            self.marks.shift_remove(&piece.r#box);
            return Ok(None);
        }
        self.ticking.insert(piece.r#box);
        if !timed {
            (self.log)(&format!(
                "screen ticked: {}  [layout {:#x}]",
                piece.text, piece.root
            ));
            return Ok(Some(true));
        }
        let down = now < then;
        if let Some(tick) = &self.tick {
            tick();
        }
        let passed = if down { now..then } else { then + 1..now + 1 };
        let marks: Vec<i64> = passed.filter(|&seconds| marked(seconds, down)).collect();
        if !marks.is_empty() {
            // `min(marks, key=lambda s: abs(s - now))`: the first of equals.
            let mut mark = marks[0];
            for &seconds in &marks[1..] {
                if (seconds - now).abs() < (mark - now).abs() {
                    mark = seconds;
                }
            }
            if self.marks.get(&piece.r#box) != Some(&mark) {
                self.marks.insert(piece.r#box, mark);
                return Ok(Some(false));
            }
        }
        (self.log)(&format!(
            "screen ticked: {}  [layout {:#x}]",
            piece.text, piece.root
        ));
        Ok(Some(true))
    }
}

/// The seconds a clock's words show, "00:10" as minutes and seconds; None for words with no
/// number, or more than a clock's two.
fn seconds(text: &str) -> Result<Option<i64>> {
    let numbers: Vec<&str> = NUMBERS.find_iter(text).map(|m| m.as_str()).collect();
    if !(!numbers.is_empty() && numbers.len() <= 2) {
        return Ok(None);
    }
    let mut total = 0;
    for number in numbers {
        total = total * 60 + py::int_base10(number)?;
    }
    Ok(Some(total))
}

/// Is a clock's tick said? Each half minute, and counting down, at 10 and each of the last 5.
fn marked(seconds: i64, down: bool) -> bool {
    (seconds > 0 && py::modulo(seconds, 30) == 0)
        || (down && (seconds == 10 || (0 < seconds && seconds <= 5)))
}
