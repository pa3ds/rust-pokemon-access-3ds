//! A battle's timers, where the battle has them: a battle with another player through the PSS
//! shows Battle Time, the minutes and seconds the battle has left, and Command Time, the seconds
//! left to choose, on the top screen, `btl_time.bclyt`. Each digit is a picture of its own showing
//! its part of one image of all ten, `battle_timer.bclim`, so no TextBox says either: Command Time
//! counted down unheard (the PSS's multiplayer, spike A).
//!
//! - **C says Command Time and Shift and C Battle Time** (`[pa3ds.pss.battle]` in the keys file),
//!   in any battle: "no timer" where none is shown, a battle with no time limit, or Command Time
//!   once the player has chosen.
//! - **Command Time is timed** as the clocks of Amie's games are (`common.clocks`): every second a
//!   tick, `[pa3ds.audio] ticks`, and said at its marks, each half minute and at 10 and each of the
//!   last 5. Battle Time, running the whole battle, is the player's to ask for.
//!
//! Each timer is a row of its own, a pane under the layout's root holding its label and its
//! digits: Battle Time's with a colon among them, Command Time's without. A digit is told by where
//! in the image its picture's corner is (`DIGITS`); a digit hidden is not shown, as Command Time's
//! tens below 10, "Command Time: 7".
//!
//! Bound by:
//! - ADR-0013: a timed screen's clock is said only at its marks.
//! - ADR-0022: a context's keys are taken only while its screen runs.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::speech::Category;
use crate::core::words::own;
use crate::games::common::clocks::marked;
use crate::games::common::field::keys::{BATTLE_TIME, COMMAND_TIME, PSS_BATTLE};
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::games::common::touch::FileOf;
use crate::games::gen6::apps::BATTLE;
use crate::py::{self, Result, sorted_by_lt};

pub const FILE: &str = "btl_time.bclyt";
/// Each digit by its top left corner's place in `battle_timer.bclim`, u and v, 0 to 4 along one
/// row and 5 to 9 along the other: Y's link battle counted each down through all ten (spike A).
pub const DIGITS: [((f64, f64), i64); 10] = [
    ((0.075, 0.518), 0),
    ((0.215, 0.518), 1),
    ((0.365, 0.518), 2),
    ((0.535, 0.518), 3),
    ((0.695, 0.518), 4),
    ((0.045, 0.278), 5),
    ((0.215, 0.278), 6),
    ((0.375, 0.278), 7),
    ((0.535, 0.278), 8),
    ((0.695, 0.278), 9),
];
/// How far a corner may sit from its digit's, f32 against the three places written above.
pub const CLOSE: f64 = 0.01;
pub const NO_TIMER: &str = "pss-no-timer";
/// The most seconds one tick goes down: a poll can come late and miss one. More is a new turn.
pub const TICK: i64 = 2;

/// C and Shift and C in a battle, and Command Time's ticks and marks.
pub struct BattleTimers {
    layouts: Rc<Layouts>,
    file_of: FileOf,
    tick: Option<Rc<dyn Fn()>>,
    /// Command Time at the last poll, None while none was shown.
    last: Option<i64>,
    /// The mark last said, which a second won back cannot say again.
    mark: Option<i64>,
}

/// Python's `min(values)` of floats: the first unless a later one is strictly less.
fn least(values: &[f64]) -> f64 {
    let mut found = values[0];
    for &value in &values[1..] {
        if value < found {
            found = value;
        }
    }
    found
}

/// Python's `max(values)` of floats: the first unless a later one is strictly greater.
fn most(values: &[f64]) -> f64 {
    let mut found = values[0];
    for &value in &values[1..] {
        if value > found {
            found = value;
        }
    }
    found
}

impl BattleTimers {
    /// Python's `tick` defaults to None.
    pub fn new(layouts: Rc<Layouts>, file_of: FileOf, tick: Option<Rc<dyn Fn()>>) -> Self {
        BattleTimers {
            layouts,
            file_of,
            tick,
            last: None,
            mark: None,
        }
    }

    /// The keys' table in the keys file while a battle runs, else None.
    pub fn context(&self, running: Option<&str>) -> Option<&'static str> {
        if running == Some(BATTLE) {
            Some(PSS_BATTLE)
        } else {
            None
        }
    }

    /// Say Command Time or Battle Time as the top screen shows it, or that it shows none.
    pub fn press(&self, action: &str, _running: Option<&str>) -> Result<String> {
        let (command, battle) = self.timers()?;
        let said = if action == COMMAND_TIME {
            command
        } else if action == BATTLE_TIME {
            battle
        } else {
            Some(String::new())
        };
        Ok(said.unwrap_or_else(|| own(NO_TIMER, &[])))
    }

    /// Command Time's tick, and the time at a mark, as (the log's name, the line, its
    /// category).
    pub fn poll(
        &mut self,
        running: Option<&str>,
    ) -> Result<Vec<(String, String, Option<Category>)>> {
        let command = if running == Some(BATTLE) {
            self.timers()?.0
        } else {
            None
        };
        let now = match command {
            None => None,
            Some(command) => Some(py::int_base10(&command)?),
        };
        let last = std::mem::replace(&mut self.last, now);
        // A turn's Command Time passes its marks afresh: once chosen, and the next begun.
        if now.is_none() || last.is_some_and(|last| now.is_some_and(|now| now > last)) {
            self.mark = None;
        }
        let (Some(now), Some(last)) = (now, last) else {
            return Ok(Vec::new());
        };
        if !(0 < last - now && last - now <= TICK) {
            return Ok(Vec::new());
        }
        if let Some(tick) = &self.tick {
            tick();
        }
        let marks: Vec<i64> = (now..last)
            .filter(|&seconds| marked(seconds, true))
            .collect();
        // `min(marks, key=...)`: the first of the nearest.
        let Some(mark) = marks
            .iter()
            .copied()
            .min_by_key(|seconds| (seconds - now).abs())
        else {
            return Ok(Vec::new());
        };
        if Some(mark) == self.mark {
            return Ok(Vec::new());
        }
        self.mark = Some(mark);
        Ok(vec![(
            "battle command time".to_string(),
            now.to_string(),
            Some(Category::Battle),
        )])
    }

    /// Command Time, "7", and Battle Time, "56:35", as shown now; None for one not shown.
    pub fn timers(&self) -> Result<(Option<String>, Option<String>)> {
        let layouts = &self.layouts;
        for root in layouts.roots() {
            let file = (self.file_of)(root);
            let placement = if file.is_some_and(|file| file.name == FILE) {
                layouts.placement(root)?
            } else {
                None
            };
            if !placement.is_some_and(|placement| placement.shown) {
                continue;
            }
            let (mut command, mut battle): (Option<String>, Option<String>) = (None, None);
            for row in self.rows(root)?.into_values() {
                let digits: String = row
                    .iter()
                    .filter_map(|&(_x, d)| d.map(|d| d.to_string()))
                    .collect();
                if digits.is_empty() {
                    continue;
                }
                let places: Vec<f64> = row
                    .iter()
                    .filter(|(_x, d)| d.is_some())
                    .map(|&(x, _d)| x)
                    .collect();
                if row
                    .iter()
                    .any(|&(x, d)| d.is_none() && least(&places) < x && x < most(&places))
                {
                    if battle.is_none() {
                        let cut = digits.len().saturating_sub(2);
                        let minutes = if cut == 0 { "0" } else { &digits[..cut] };
                        battle = Some(format!("{minutes}:{}", &digits[cut..]));
                    }
                } else if command.is_none() {
                    command = Some(py::int_base10(&digits)?.to_string());
                }
            }
            return Ok((command, battle));
        }
        Ok((None, None))
    }

    /// Each row's pictures shown, left to right, by the pane under the root they are in: each
    /// its x and its digit, None for another picture, a label or the colon.
    fn rows(&self, root: i64) -> Result<IndexMap<i64, Vec<(f64, Option<i64>)>>> {
        let layouts = &self.layouts;
        let mut rows: IndexMap<i64, Vec<(f64, Option<i64>)>> = IndexMap::new();
        for (pane, kind) in layouts.tree(root) {
            let placement = layouts.placement(pane)?;
            if kind != lyt::PICTURE_CLASS || !placement.is_some_and(|placement| placement.shown) {
                continue;
            }
            let head = layouts.pane(pane)?;
            let row = u32(&head, lyt::PARENT)?;
            if row == root {
                continue;
            }
            let (u, v) = (
                f32(&head, lyt::PICTURE_UV)?,
                f32(&head, lyt::PICTURE_UV + 4)?,
            );
            let digit = DIGITS
                .iter()
                .find(|&&((du, dv), _d)| (u - du).abs() < CLOSE && (v - dv).abs() < CLOSE)
                .map(|&(_corner, d)| d);
            rows.entry(row)
                .or_default()
                .push((f32(&head, layouts.shape.x)?, digit));
        }
        Ok(rows
            .into_iter()
            .map(|(row, pictures)| (row, sorted_by_lt(pictures, |a, b| a.0 < b.0)))
            .collect())
    }
}
