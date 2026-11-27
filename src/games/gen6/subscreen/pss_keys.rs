//! The keys of the screens two players meet on through the PSS, which say what the top screen
//! draws and nothing reads out: `[pa3ds.pss.trade]` and `[pa3ds.pss.colosseum]` in the keys file.
//!
//! - **A trade** (`pss.TRADE_LIST`): R says the Pokémon the player has offered and Shift and R the
//!   other player's, as its top screen writes each, `trade_upper.bclyt`: a name, its gender sign,
//!   "Lv.", the level and the held item in five boxes, the player's on the left half and the other
//!   player's on the right (spike A: the other player's Treecko at x 70 to 170, and the
//!   player's five boxes hidden while nothing was offered; the left half unchecked live).
//! - **The team choice before a battle chosen in the PSS** (the party opened from
//!   `pss.COLOSSEUM`): Shift and R says the team the other player chose, as its top screen draws
//!   it, `pokelist_uppwer.bclyt`: their name, then each Pokémon's gender sign and level, but for
//!   its icon, which nothing names yet (spike A); its "TIME REMAINING" row is the clock's, said at
//!   its marks (`Subscreens.timed`), and left out: C says it.
//!
//! A key with nothing drawn to say says so.

use std::rc::Rc;

use crate::core::words::own;
use crate::games::common::clocks::{CLOCK, NO_TIMER};
use crate::games::common::field::keys::{
    PSS_COLOSSEUM, PSS_TRADE, THEIR_OFFER, THEIR_TEAM, TIME, YOUR_OFFER,
};
use crate::games::common::screen::ScreenText;
use crate::games::common::screen_order::{in_rows, join};
use crate::games::common::touch::{FileOf, OnScreen};
use crate::games::gen6::screens::rows::PARTY;
use crate::games::gen6::subscreen::pss::{COLOSSEUM, TRADE_LIST};

pub const OFFERS: &str = "trade_upper.bclyt";
/// The layout files of the trade's top screen and the team choice's, by the names the game gave
/// them, "uppwer" its own spelling.
pub const TEAM: &str = "pokelist_uppwer.bclyt";
pub const NO_OFFER: &str = "pss-no-offer";
pub const NO_TEAM: &str = "pss-no-team";

/// `Callable[[], str | None]`: the app that ran before the one running.
pub type Before = Rc<dyn Fn() -> Option<String>>;

/// R and Shift and R on the screens two players meet on: a trade's offers, the other player's
/// team. `before` is the app that ran before the one running (`Subscreens.started`).
pub struct PssKeys {
    on_screen: OnScreen,
    file_of: FileOf,
    before: Before,
    pauses: bool,
}

impl PssKeys {
    /// Python's `pauses` defaults to False.
    pub fn new(on_screen: OnScreen, file_of: FileOf, before: Before, pauses: bool) -> Self {
        PssKeys {
            on_screen,
            file_of,
            before,
            pauses,
        }
    }

    /// The keys' table in the keys file while a trade or the team choice runs, else None.
    pub fn context(&self, running: Option<&str>) -> Option<&'static str> {
        if running == Some(TRADE_LIST) {
            return Some(PSS_TRADE);
        }
        if running == Some(PARTY) && (self.before)().as_deref() == Some(COLOSSEUM) {
            return Some(PSS_COLOSSEUM);
        }
        None
    }

    /// Say an offer or the other player's team, as the top screen draws it.
    pub fn press(&self, action: &str, _running: Option<&str>) -> String {
        if action == YOUR_OFFER || action == THEIR_OFFER {
            let left = action == YOUR_OFFER;
            let offer: Vec<ScreenText> = self
                .drawn(OFFERS)
                .into_iter()
                .filter(|p| ((p.left + p.right) / 2.0 < 0.0) == left)
                .collect();
            return or_own(join(&in_rows(offer), self.pauses), NO_OFFER);
        }
        if action == THEIR_TEAM || action == TIME {
            let drawn = self.drawn(TEAM);
            // The clock's row, "TIME REMAINING: 1:24", is C's, and said at its marks.
            let clocks: Vec<f64> = drawn
                .iter()
                .filter(|p| CLOCK.is_match(&p.text))
                .map(|p| p.row)
                .collect();
            let timed: Vec<ScreenText> = drawn
                .iter()
                .filter(|p| {
                    // Python's `max(p.glyph, 1)`: the glyph unless 1 is greater.
                    let least = if 1.0 > p.glyph { 1.0 } else { p.glyph };
                    clocks.iter().any(|row| (p.row - row).abs() < least)
                })
                .cloned()
                .collect();
            if action == TIME {
                return or_own(join(&in_rows(timed), self.pauses), NO_TIMER);
            }
            let team: Vec<ScreenText> = drawn.into_iter().filter(|p| !timed.contains(p)).collect();
            return or_own(join(&in_rows(team), self.pauses), NO_TEAM);
        }
        String::new()
    }

    /// The words on the top screen in the layout built from the file of this name.
    fn drawn(&self, name: &str) -> Vec<ScreenText> {
        (self.on_screen)()
            .into_iter()
            .filter(|p| {
                p.top && !p.picture && (self.file_of)(p.root).is_some_and(|file| file.name == name)
            })
            .collect()
    }
}

/// Python's `said or own(key)`: the words, or the message where they are none.
fn or_own(said: String, key: &str) -> String {
    if said.is_empty() { own(key, &[]) } else { said }
}
