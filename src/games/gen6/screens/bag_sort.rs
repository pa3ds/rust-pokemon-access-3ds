//! The Bag's own sort: an item picked up with Y, said as it is carried up and down its list.
//!
//! Y on an item in the Bag picks it up, and the list is drawn with it where it would land: each step
//! of the D-pad moves it a place and draws the items shifted round it, so a drop puts it between the
//! two now beside it (or_bag_manual_sorting). Its cursor stays on the carried item, so as read it
//! said the item again and again, or, as the list scrolled under a cursor that did not move,
//! nothing.
//!
//! Nothing in the layout says an item is being carried, and the words "moving" come from the first
//! step after it is picked up: a cursor that stays on the same item while the items beside it
//! change is carrying it. Item names are unique in a pocket, as each stack is one row.
//!
//! Said: the first step, "moving Venusaurite, between Metronome and Soothe Bell"; every other step
//! "between Soothe Bell and Rocky Helmet", or at an end of the list "above Metronome".

use std::collections::HashSet;
use std::rc::Rc;

use crate::core::words::own;
use crate::games::common::cursors::shared::CursorPoll;
use crate::games::common::screen::ScreenText;

/// The fewest rows a column of words has to have to be a list: the Bag's draws up to nine.
pub const ROWS: usize = 3;
/// Units a row's words can start from the cursor's, for the two to be one column.
pub const SAME_COLUMN: f64 = 1.0;
/// Polls after a cursor lands on an item in which the rows round it are still being drawn.
pub const SETTLE: i64 = 2;

/// The item under a cursor, the words on the rows above and below it, and every row's.
type Placed = (ScreenText, String, String, HashSet<String>);

/// Says an item carried in the Bag, and takes the cursor's repeats of it out of a poll.
pub struct BagSort {
    running: Rc<dyn Fn() -> bool>,
    said: Option<(String, String, String)>,
    /// The rows drawn when `said` was: a step keeps most of them, another pocket none.
    ///
    /// A `frozenset` only tested.
    column: HashSet<String>,
    held: Option<(String, String, String)>,
    carrying: bool,
    settling: i64,
}

impl BagSort {
    pub fn new(running: Rc<dyn Fn() -> bool>) -> Self {
        BagSort {
            running,
            said: None,
            column: HashSet::new(),
            held: None,
            carrying: false,
            settling: 0,
        }
    }

    /// What to say of an item being carried, if anything, and the poll without the cursor's
    /// own saying of it.
    pub fn poll(
        &mut self,
        poll: CursorPoll,
        on_screen: &[ScreenText],
    ) -> (Option<String>, CursorPoll) {
        let placed = if !(self.running)() {
            None
        } else {
            poll.under.iter().find_map(|under| placed(under, on_screen))
        };
        let Some((item, above, below, column)) = placed else {
            self.said = None;
            self.held = None;
            self.carrying = false;
            return (None, poll);
        };
        let now = (item.text.clone(), above.clone(), below.clone());
        if self.settling != 0
            && let Some(said) = &self.said
            && now.0 == said.0
        {
            // A pocket turned to is still being drawn for a poll or two after its cursor lands.
            self.settling -= 1;
            self.said = Some(now);
            self.column = column;
            return (None, poll);
        }
        let another = match &self.said {
            None => true,
            Some(said) => now.0 != said.0,
        };
        if another
            || (self.said.as_ref() != Some(&now)
                && ![&above, &below]
                    .into_iter()
                    .any(|row| !row.is_empty() && self.column.contains(row)))
        {
            // A new item under the cursor, or the same words on the first row of another pocket,
            // whose rows are none of the last's: "moving Amulet Coin, below Max Repel"
            // (or_bag_and_battle_bag). An item still carried past the list's end stays carried.
            self.carrying = self.carrying && !another;
            self.settling = if another { SETTLE } else { 0 };
            self.said = Some(now);
            self.column = column;
            self.held = None;
            return (None, poll);
        }
        let items = poll
            .items
            .iter()
            .filter(|one| one.opened || !one.pieces.iter().any(|p| p.r#box == item.r#box))
            .cloned()
            .collect();
        let same = CursorPoll { items, ..poll };
        if self.said.as_ref() == Some(&now) {
            self.held = None;
            return (None, same);
        }
        if self.held.as_ref() != Some(&now) {
            // A list being drawn is half old and half new for a poll.
            self.held = Some(now);
            return (None, same);
        }
        self.said = Some(now);
        self.column = column;
        self.held = None;
        let mut line = between(&above, &below);
        if !self.carrying {
            self.carrying = true;
            line = format!("{}, {line}", own("bag-moving", &[("item", &item.text)]));
        }
        (Some(line), same)
    }
}

/// The one piece a cursor is over, if it is a row of a list, with the words on the rows above
/// and below it, "" where it has none, and every row's.
fn placed(under: &[ScreenText], on_screen: &[ScreenText]) -> Option<Placed> {
    if under.len() != 1 || under[0].glyph <= 0.0 {
        return None;
    }
    let item = &under[0];
    let column: Vec<&ScreenText> = on_screen
        .iter()
        .filter(|piece| {
            piece.root == item.root
                && !piece.picture
                && piece.glyph == item.glyph
                && (piece.left - item.left).abs() < SAME_COLUMN
        })
        .collect();
    if column.len() < ROWS {
        return None;
    }
    // Python's `min` and `max` with a key: the first kept unless a later one is strictly less
    // (or greater).
    let mut above: Option<&ScreenText> = None;
    let mut below: Option<&ScreenText> = None;
    for p in &column {
        if p.row > item.row && above.is_none_or(|best| p.row < best.row) {
            above = Some(p);
        }
        if p.row < item.row && below.is_none_or(|best| p.row > best.row) {
            below = Some(p);
        }
    }
    Some((
        item.clone(),
        above.map_or(String::new(), |p| p.text.clone()),
        below.map_or(String::new(), |p| p.text.clone()),
        column.iter().map(|piece| piece.text.clone()).collect(),
    ))
}

fn between(above: &str, below: &str) -> String {
    if !above.is_empty() && !below.is_empty() {
        return own("bag-between", &[("above", &above), ("below", &below)]);
    }
    if !below.is_empty() {
        own("bag-above", &[("below", &below)])
    } else {
        own("bag-below", &[("above", &above)])
    }
}
