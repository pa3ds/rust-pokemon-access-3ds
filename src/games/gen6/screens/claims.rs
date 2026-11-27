//! Whose words: what of a screen poll's words screen text leaves to others, decided in one place.
//!
//! Screen text says a screen's words as they come and change (ADR-0013), but some are another's to
//! say, and each is claimed here:
//!
//! - **Readers of their own** say some words themselves, which screen text leaves unsaid but still
//!   hears among the words on screen: the battle's HP boxes, the habitat's title and place
//!   (`owners`).
//! - **The game's own and the cursor sources** take some out of the screen altogether, unsaid and
//!   unheard: the AreaNav's Pokémon Distribution's icons, read again by a rewrite of the groups it
//!   met (`own`), and the TextBoxes and layouts a source says or does not draw (`Polled.unsaid`,
//!   `Polled.unseen`).
//! - **The list readers' labels**, and the words a source has said, are left out everywhere.
//! - **A button's words** are left to whoever says them as the player meets the button, on the
//!   screens whose rule says so (`ButtonRule`, `RULES`): on the field's bottom screen and the PSS's
//!   own apps, touch mode, which says each as its target (`buttons`); in the Pokédex, its cursors,
//!   which say each as a menu's item, its registration's captions going with them, but for X's
//!   regions, which no cursor says (`ButtonRule.said`). They are left out as they come on screen
//!   and kept out of a screen coming back (`Heard.quiet`); words changing in one button are still
//!   said, "Game Chat: Disabled", unless a page of them turns. Only as they come: they are still
//!   heard on the screen, and Amie's Switch screen reads each Pokémon's Fullness and Enjoyment from
//!   them, as details.
//!
//! What is left is a `Heard`, which the announcer says (`ScreenAnnouncer.say`), once the cursor
//! sources have had the last word on it (`CursorSources.said`). The claims are a union, so their
//! order is no part of what is said. The asymmetry is: the readers of their own leave their words
//! among those heard, which the rest of their screen is read beside, and nothing else does.
//!
//! Bound by:
//! - ADR-0013: on the field's bottom screen and the PSS's apps a button's words are touch mode's,
//!   but for words changing in one button, a layout a cursor is over on the field, and a large touch
//!   area, a window onto the screen; a page turned is left to touch mode too. The Pokédex's are its
//!   cursors', the registration's "Rotate" and "Zoom" with them, and X's regions screen text's
//!   until a cursor of theirs is found (amended 2026-11-24).

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::games::common::screen::{ScreenPoll, ScreenText};
use crate::games::common::screen_announce::Heard;
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::screens::pokedex;
use crate::games::gen6::screens::sources::Polled;
use crate::py::Result;

/// Buttons of one layout whose words change in a poll, at least, for a page of them turned rather
/// than one button's words changed: L and R page O-Power's list, its seven at once
/// (or_pss_o_power_2), where a button changing its own words alone is still said: "Game Chat:
/// Disabled".
pub const PAGE_TURNED: i64 = 2;

/// How a screen leaves its buttons' words to others (`Claims`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonRule {
    /// A layout a cursor is over keeps its words: the field's X menu, whose words are its items.
    /// The PSS's apps have no menu: the profile's Message row, lit as it came back from its
    /// messages, kept its words for the cursor it was taken for.
    pub cursor_keeps: bool,
    /// Two or more of a layout's buttons changing in one poll are a page turned, left unsaid too:
    /// L and R page O-Power's list, and its seven powers were read as they came
    /// (or_pss_o_power_2).
    pub pages: bool,
    /// (layout file, pane name) of words no button's that are left unsaid with them, as captions
    /// of the pictures over its buttons.
    pub unsaid: &'static [(&'static str, &'static str)],
    /// Layout files whose buttons' words stay screen text's, where no cursor says them: a cursor
    /// leaves a menu unsaid only while it speaks the item under it.
    pub said: &'static [&'static str],
}

impl Default for ButtonRule {
    fn default() -> Self {
        PSS
    }
}

/// The apps whose buttons' words are another's, by class, with how (ADR-0013).
pub const RULES: [(&str, ButtonRule); 3] = [
    // The field's bottom screen is touched, never walked with a cursor: read as they came, the
    // PSS said "Friends Acquaintances Passersby" each time it came back, and its menu "Battle
    // Trade Battle Spot" (or_pss_singleplayer).
    (
        FIELD,
        ButtonRule {
            cursor_keeps: true,
            ..PSS
        },
    ),
    // The Pokédex's buttons are menu items its cursors say: the choices read
    // "HOENN POKÉDEX Number caught: 36 Number seen: 181 NATIONAL POKÉDEX…" as it opened, and the
    // cursor the first again (or_pokedex), and X its list's numbers before its cursor said the
    // entry (x_pokedex). Its registration's Next, and "Rotate" and "Zoom" over the model's four
    // arrows, followed the entry (or_groudon_master_ball). X's regions, with their counts, have
    // no cursor found, and stay screen text's.
    (
        pokedex::ORAS_APP,
        ButtonRule {
            cursor_keeps: true,
            unsaid: &pokedex::CAPTIONS,
            ..PSS
        },
    ),
    (
        pokedex::APP,
        ButtonRule {
            cursor_keeps: true,
            said: &[pokedex::REGIONS],
            ..PSS
        },
    ),
];

/// The rule of the PSS's own apps (`Subscreens.in_pss`), which are the field's bottom screen too:
/// its profile said "Message: Nice to meet ya!" as it opened (or_pss_singleplayer_2).
///
/// `ButtonRule()`: every field at its default.
pub const PSS: ButtonRule = ButtonRule {
    cursor_keeps: false,
    pages: true,
    unsaid: &[],
    said: &[],
};

/// The rule of the app running, or None where its buttons' words are screen text's.
pub fn rule(running: Option<&str>, in_pss: &dyn Fn() -> bool) -> Option<ButtonRule> {
    let running = running.unwrap_or("");
    RULES
        .iter()
        .find(|(app, _)| *app == running)
        .map(|(_, rule)| *rule)
        .or_else(|| in_pss().then_some(PSS))
}

/// `Callable[[ScreenText], bool]` of a reader that says words of the screen itself, which can
/// raise as it reads.
pub type Owner = Rc<dyn Fn(&ScreenText) -> Result<bool>>;

/// What screen text leaves to others of each screen poll's words.
pub struct Claims {
    owners: Vec<Owner>,
    own: Rc<dyn Fn(&ScreenText) -> bool>,
    buttons: Rc<dyn Fn() -> Result<HashSet<i64>>>,
    rules: Rc<dyn Fn() -> Option<ButtonRule>>,
    named: Named,
}

/// `Callable[[ScreenText], tuple[str, str] | None]`: words' layout file and pane name, where
/// known.
pub type Named = Rc<dyn Fn(&ScreenText) -> Option<(String, String)>>;

impl Claims {
    /// `owners` are the readers that say words of the screen themselves, `own` the game's own's
    /// words, `buttons` the TextBoxes naming a button on the bottom screen now
    /// (`TouchTargets.button_words`), asked only where a rule applies, `rules` the rule of the
    /// screen up now, or None, and `named` words' layout file and pane name, where known.
    pub fn new(
        owners: Vec<Owner>,
        own: Rc<dyn Fn(&ScreenText) -> bool>,
        buttons: Rc<dyn Fn() -> Result<HashSet<i64>>>,
        rules: Rc<dyn Fn() -> Option<ButtonRule>>,
        named: Named,
    ) -> Self {
        Claims {
            owners,
            own,
            buttons,
            rules,
            named,
        }
    }

    /// The poll's words as the others leave them, the cursor sources' `polled` among them, and
    /// the list readers' `labels`.
    pub fn poll(
        &self,
        screen: &ScreenPoll,
        polled: &Polled,
        labels: &HashSet<String>,
    ) -> Result<Heard> {
        let (unsaid, unseen) = (&polled.unsaid, &polled.unseen);
        let mut texts: Vec<ScreenText> = Vec::new();
        'texts: for text in &screen.texts {
            for owns in &self.owners {
                if owns(text)? {
                    continue 'texts;
                }
            }
            if !(self.own)(text) && !unsaid.contains(&text.r#box) && !unseen.contains(&text.root)
            {
                texts.push(text.clone());
            }
        }
        let mut quiet: HashSet<i64> = HashSet::new();
        // `Counter(not t.changed for t in texts if not t.top)`: the bottom screen's words come or
        // came back (True), and changed (False).
        let bottom = |shown: bool| -> i64 {
            texts
                .iter()
                .filter(|t| !t.top && !t.changed == shown)
                .count() as i64
        };
        if let Some(rule) = (self.rules)()
            && (bottom(true) > 0 || bottom(false) >= PAGE_TURNED)
        {
            let mut buttons = (self.buttons)()?;
            if !rule.unsaid.is_empty() {
                for piece in &screen.on_screen {
                    if !piece.top
                        && let Some((file, pane)) = (self.named)(piece)
                        && rule.unsaid.iter().any(|&(f, p)| f == file && p == pane)
                    {
                        buttons.insert(piece.r#box);
                    }
                }
            }
            if !rule.said.is_empty() {
                for piece in &screen.on_screen {
                    if let Some((file, _pane)) = (self.named)(piece)
                        && rule.said.iter().any(|said| *said == file)
                    {
                        buttons.remove(&piece.r#box);
                    }
                }
            }
            let cursored: HashSet<i64> = if rule.cursor_keeps {
                polled
                    .cursors
                    .under
                    .iter()
                    .flatten()
                    .map(|piece| piece.root)
                    .collect()
            } else {
                HashSet::new()
            };
            let mut changed: HashMap<i64, i64> = HashMap::new();
            for text in &texts {
                if !text.top && text.changed && buttons.contains(&text.r#box) {
                    *changed.entry(text.root).or_default() += 1;
                }
            }
            texts.retain(|text| {
                text.top
                    || !buttons.contains(&text.r#box)
                    || (text.changed
                        && (!rule.pages
                            || changed.get(&text.root).copied().unwrap_or(0) < PAGE_TURNED))
                    || cursored.contains(&text.root)
            });
            quiet = screen
                .on_screen
                .iter()
                .filter(|piece| buttons.contains(&piece.r#box) && !cursored.contains(&piece.root))
                .map(|piece| piece.r#box)
                .collect();
        }
        // The Distribution's icons are not gathered with the rest of their screen either: a
        // rewrite reads again the groups it has met.
        let heard: Vec<ScreenText> = screen
            .on_screen
            .iter()
            .filter(|piece| {
                !(self.own)(piece) && !unsaid.contains(&piece.r#box) && !unseen.contains(&piece.root)
            })
            .cloned()
            .collect();
        Ok(Heard {
            labels: labels.union(&polled.labels).cloned().collect(),
            quiet,
            ..Heard::new(texts, heard)
        })
    }
}
