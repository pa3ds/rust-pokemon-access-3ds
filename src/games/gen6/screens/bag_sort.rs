//! The Bag's own sort: an item picked up with Y, said as it is carried up and down its list.
//!
//! Y on an item in the Bag picks it up, and the list is drawn with it where it would land: each step
//! of the D-pad moves it a place and draws the items shifted round it, so a drop puts it between the
//! two now beside it (or_bag_manual_sorting). Its cursor stays on the carried item, so as read it
//! said the item again and again, or, as the list scrolled under a cursor that did not move,
//! nothing.
//!
//! Nothing in the layout says an item is being carried. The Bag's own mode does, from the pick-up
//! to the drop (`bag.Chosen.carrying`, spike 298). pa3ds had told it from the steps, a cursor
//! staying on one item while the items beside it changed: a pick-up was said only once a step had
//! settled, the first steps missed (or_bag_manual_sorting, "moving Venusaurite, between Shell Bell
//! and Float Stone", two steps in), and a drop and pick-up again said the cursor's lines, "Soothe
//! Bell", "Venusaurite", before "moving" at the next step.
//!
//! Said: the pick-up, "moving Venusaurite", again if the item is dropped and picked straight back
//! up; each step "between Soothe Bell and Rocky Helmet", or at an end of the list "above
//! Metronome". The drop says nothing: the last step said where it lands. While carried, the list's
//! cursor says nothing of its own: it is on the carried item.
//!
//! Bound by:
//! - ADR-0013: a cursor source says when the item under the cursor changes, the words still the
//!   screen's.

use std::rc::Rc;

use crate::core::words::own;
use crate::games::common::cursors::shared::CursorPoll;
use crate::games::common::screen::ScreenText;
use crate::py::Result;

/// The fewest rows a column of words has to have to be a list: the Bag's draws up to nine.
pub const ROWS: usize = 3;
/// Units a row's words can start from the cursor's, for the two to be one column.
pub const SAME_COLUMN: f64 = 1.0;

/// The item under a cursor, and the words on the rows above and below it.
type Placed = (ScreenText, String, String);

/// Says an item carried in the Bag, and takes the cursor's lines out of a poll while it is.
pub struct BagSort {
    carrying: Rc<dyn Fn() -> Result<bool>>,
    /// The item carried, once said.
    item: Option<String>,
    /// The rows above and below it when last said.
    said: (String, String),
    held: Option<(String, String)>,
}

impl BagSort {
    /// `carrying` is whether the Bag has an item picked up with Y.
    pub fn new(carrying: Rc<dyn Fn() -> Result<bool>>) -> Self {
        BagSort {
            carrying,
            item: None,
            said: (String::new(), String::new()),
            held: None,
        }
    }

    /// What to say of an item being carried, if anything, and the poll without the cursor's
    /// own saying of it.
    pub fn poll(
        &mut self,
        poll: CursorPoll,
        on_screen: &[ScreenText],
    ) -> Result<(Option<String>, CursorPoll)> {
        if !(self.carrying)()? {
            self.item = None;
            self.held = None;
            return Ok((None, poll));
        }
        let placed = poll.under.iter().find_map(|under| placed(under, on_screen));
        let quiet = CursorPoll {
            items: Vec::new(),
            ..poll
        };
        let Some((item, above, below)) = placed else {
            return Ok((None, quiet));
        };
        let Some(carried) = &self.item else {
            self.item = Some(item.text.clone());
            self.said = (above, below);
            self.held = None;
            return Ok((Some(own("bag-moving", &[("item", &item.text)])), quiet));
        };
        let now = (above, below);
        if item.text != *carried || now == self.said {
            self.held = None;
            return Ok((None, quiet));
        }
        if self.held.as_ref() != Some(&now) {
            // A list being drawn is half old and half new for a poll.
            self.held = Some(now);
            return Ok((None, quiet));
        }
        let line = between(&now.0, &now.1);
        self.said = now;
        self.held = None;
        Ok((Some(line), quiet))
    }
}

/// The one piece a cursor is over, if it is a row of a list, with the words on the rows above
/// and below it, "" where it has none.
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
