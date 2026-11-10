//! Sun's battle HP: who is on each side, at what level, and how hurt, said as ORAS's is (ADR-0015):
//! the player's side "Rowlet♀ L5 22/22" whenever its HP changes, the enemy's as a percentage,
//! "Popplio♂ L5 60%", since a bar is all the game shows for it.
//!
//! Sun's HP box is one layout with a panel a side, and every word of it is screen text where ORAS
//! drew the names as pictures (spike 124, sun_9.ctm, Hau's Popplio against Rowlet in Iki Town):
//!
//! ```text
//! panel  HPndBall_LB (the player's), HPndBall_RT (the enemy's): a pane directly under the root
//!        TextBox  the name, "Rowlet"     TextBox  the level, "5", on the name's row
//!        TextBox  the HP, "22", and the maximum, "22" in fullwidth digits (U+FF12), on a row
//!                 of their own: the player's panel only
//!        Picture  Icon_Gender, 12 by 12: the sign, a texture pa3ds cannot read
//!        Picture  the gauge's fill, 16 tall and anchored left, 104.8 wide when full; its two
//!                 siblings, the gauge's frame and backing, 112 wide and centred
//! ```
//!
//! Nothing there says which Pokémon a panel shows, how far a count has got, or what fraction of its
//! HP the enemy has left, so all three come from the gauge behind it, `btl::BtlvGaugeBase`, one a
//! panel, in the battle's heaps under heap 5:
//!
//! ```text
//! gauge  +0x0c -> the battle record of the Pokémon shown, whose +0x00 is its PokemonParam, as
//!                 ORAS's (spike 66); +0x0c u16 species, +0x0e u16 maximum HP, +0x10 current HP
//!        +0x20 -> its panel    +0x24 u32 0 the player's side, 1 the enemy's
//!        +0x28 u32 the HP the box shows    +0x2c u32 the HP it is counting towards
//! ```
//!
//! The count runs a few points a screen poll, the fill shrinking with it: Popplio's fill read
//! 104.8, 97.9, 91.9, 85.0, 79.0, 73.1 and 65.2 a poll apart. So nothing is said until the gauge
//! has counted to its target and the reading has held for a poll, as in ORAS (spike 67). The
//! enemy's percentage is the HP the gauge shows over the record's maximum, not the fill's width
//! over a full one: the fill is 5.8 + 99 x HP/max wide, a sliver at 0 HP, so 12 of 20 HP drew 65.2
//! and read 62%. The gender sign is the Pokémon record's, since the game draws it as a picture: the
//! record decodes in Sun as in ORAS, with species up to 802.
//!
//! Bound by:
//! - ADR-0015: HP comes from the battle's own records, never from the bar, and the enemy's is a
//!   percentage.
//! - ADR-0016: the HP box is the battle reader's alone: screen text leaves its pieces out while a
//!   battle is up.

use std::rc::Rc;

use indexmap::IndexMap;
use unicode_general_category::{GeneralCategory, get_general_category};

use crate::core::binary::{u16, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::battle::{Battler, GAUGE_RECORD, MAX_ASKS, REASK, RECORD_PARAM, SIGNS};
use crate::games::common::layout::Layouts;
use crate::games::common::pokemon;
use crate::games::common::screen::ScreenText;
use crate::py::{self, Error, Result};

pub const GAUGE_CLASS: &str = "N3btl13BtlvGaugeBaseE";
pub const GAUGE_PANEL: i64 = 0x20;
pub const GAUGE_SIDE: i64 = 0x24;
pub const GAUGE_SHOWN_HP: i64 = 0x28;
pub const GAUGE_TARGET_HP: i64 = 0x2C;
pub const GAUGE_HEAD: i64 = 0x30;
pub const ENEMY: i64 = 1;
/// Sun and Moon's last species, Marshadow.
pub const SPECIES_MAX: i64 = 802;
/// The battle record's u16 maximum HP (spike 124). The gauge's own, +0x30, is 0 until its first
/// count.
pub const RECORD_MAX_HP: i64 = 0x0E;
/// A pane flag the box's TextBoxes carry until the box is first drawn. Through the battle's intro,
/// Hau's challenge and his sending Popplio out, the enemy's panel read as shown, its words and
/// gauge full, while the screen showed no box at all; its name's flags were 0x11 and its place
/// (0, 0) until the box came on screen after Rowlet was sent out, 0x01 and (106, 108) from then
/// (sun_9.ctm, 2026-09-27). Read then, "Popplio♂ L5 100%" was said before "You are challenged".
pub const UNPLACED: i64 = 0x10;

/// A panel's words as `_read` makes them: the name, the level, the HP and the name's TextBox.
type Reading = (String, String, Option<String>, i64);

/// Says each side of a battle when what its panel shows has changed and settled.
pub struct SunBattleReader {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    layouts: Rc<Layouts>,
    gauges: Vec<i64>,
    /// Panel -> its gauge, as last polled.
    panels: IndexMap<i64, i64>,
    /// Battle record -> the Pokémon it holds, decrypted once rather than every poll.
    shown: IndexMap<i64, Option<pokemon::Pokemon>>,
    pending: IndexMap<i64, String>,
    said: IndexMap<i64, String>,
    /// Whether a panel with the player's numbers on it had no gauge at the last poll.
    unread: bool,
    /// Screen polls since a sweep was asked for such a panel.
    asks: i64,
}

impl SunBattleReader {
    /// `memory` should read only inside the blocks the last walk found, as the layouts do: a
    /// gauge's pointers are followed into the battle's records (ADR-0035).
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>, layouts: Rc<Layouts>) -> Self {
        SunBattleReader {
            memory,
            names,
            layouts,
            gauges: Vec::new(),
            panels: IndexMap::new(),
            shown: IndexMap::new(),
            pending: IndexMap::new(),
            said: IndexMap::new(),
            unread: false,
            asks: 0,
        }
    }

    /// Take these `btl::BtlvGaugeBase` addresses: a whole sweep's in place of those kept, or,
    /// `adding` a sweep of new blocks', beside them.
    pub fn track(&mut self, gauges: impl IntoIterator<Item = i64>, adding: bool) -> Result<()> {
        let mut found: Vec<i64> = Vec::new();
        for gauge in gauges {
            if self.names.of_object(gauge)?.as_deref() == Some(GAUGE_CLASS) {
                found.push(gauge);
            }
        }
        if adding {
            // Each is tested against the gauges kept before it, not against those added with it.
            let fresh: Vec<i64> = found
                .into_iter()
                .filter(|gauge| !self.gauges.contains(gauge))
                .collect();
            self.gauges.extend(fresh);
        } else {
            self.gauges = found;
        }
        Ok(())
    }

    /// Is this piece in a panel a gauge shows? Then it is this reader's to say, whole and once
    /// it holds still, not screen text's: "Rowlet 5 22 22" and then "21 22", "20 22" as the
    /// count went, the maximum in fullwidth digits (sun_9.ctm).
    pub fn owns(&self, piece: &ScreenText) -> bool {
        self.panels.contains_key(&piece.panel)
    }

    /// Whether a panel drawn as the HP box is on the top screen with no gauge found for it,
    /// first when seen and again every `REASK` polls, since only a sweep finds a gauge.
    pub fn sweep_wanted(&mut self) -> bool {
        if !self.unread || self.asks >= MAX_ASKS * (REASK + 1) {
            return false;
        }
        let due = self.asks % (REASK + 1) == 0;
        self.asks += 1;
        due
    }

    /// The sides whose reading has changed, finished counting and held still for a poll.
    pub fn poll(&mut self, on_screen: &[ScreenText]) -> Result<Vec<Battler>> {
        let mut panels: IndexMap<i64, Vec<&ScreenText>> = IndexMap::new();
        for piece in on_screen {
            if piece.top && piece.panel != 0 {
                panels.entry(piece.panel).or_default().push(piece);
            }
        }
        let mut readings: IndexMap<i64, Option<Reading>> = IndexMap::new();
        for (&panel, pieces) in &panels {
            readings.insert(panel, read(pieces)?);
        }
        self.panels = IndexMap::new();
        let mut out: Vec<Battler> = Vec::new();
        for gauge in self.gauges.clone() {
            let head = self.memory.read(gauge, GAUGE_HEAD)?;
            if self.names.of_vtable(u32(&head, 0)?)?.as_deref() != Some(GAUGE_CLASS) {
                // `list.remove`: the first gauge of that address.
                if let Some(at) = self.gauges.iter().position(|&held| held == gauge) {
                    self.gauges.remove(at);
                }
                continue;
            }
            let panel = u32(&head, GAUGE_PANEL)?;
            // Known from the gauge alone: the box's first words came as screen text, "Rowlet
            // 22 22 5", in the poll they slid on, before the panel had a reading (sun_9.ctm).
            self.panels.insert(panel, gauge);
            if !panels.contains_key(&panel) {
                continue;
            }
            let player = u32(&head, GAUGE_SIDE)? != ENEMY;
            let Some((name, level, hp, r#box)) = &readings[&panel] else {
                continue;
            };
            let flags = byte(&self.layouts.pane(*r#box)?, self.layouts.shape.flags)?;
            if flags & UNPLACED != 0 {
                continue;
            }
            let shown = self.pokemon(u32(&head, GAUGE_RECORD)?)?;
            let detail = if player {
                hp.clone()
            } else {
                self.share(&head)?
            };
            let (Some(shown), Some(detail)) = (shown, detail) else {
                continue;
            };
            if u32(&head, GAUGE_SHOWN_HP)? != u32(&head, GAUGE_TARGET_HP)? {
                self.pending.shift_remove(&panel);
                continue;
            }
            let sign = if shown.gender == pokemon::MALE || shown.gender == pokemon::FEMALE {
                SIGNS[shown.gender as usize]
            } else {
                ""
            };
            let text = own(
                "battle-battler",
                &[
                    ("name", &format!("{name}{sign}")),
                    ("level", level),
                    ("hp", &detail),
                ],
            );
            if self.pending.get(&panel) != Some(&text) {
                // Held for one poll before it counts: the box's words and its fill are rewritten
                // as the count runs, and a single poll catches a frame rather than a value.
                self.pending.insert(panel, text);
                continue;
            }
            if self.said.get(&panel) == Some(&text) {
                continue;
            }
            self.said.insert(panel, text.clone());
            out.push(Battler {
                panel,
                text,
                player,
            });
        }
        // A panel with the player's numbers on it and no gauge yet: ask for a sweep.
        self.unread = readings.iter().any(|(panel, reading)| {
            !self.panels.contains_key(panel)
                && reading.as_ref().is_some_and(|reading| reading.2.is_some())
        });
        if !self.unread {
            self.asks = 0;
        }
        // Forgotten with the gauges, not with the box: the whole gauge hides through each attack's
        // animation and comes back, and "Popplio♂ L5 100%" was said again each time (sun_9.ctm).
        // The battle's heaps go with the battle, and its gauges' blocks with them.
        if self.gauges.is_empty() {
            self.pending.clear();
            self.said.clear();
            self.shown.clear();
        }
        Ok(out)
    }

    fn pokemon(&mut self, record: i64) -> Result<Option<pokemon::Pokemon>> {
        if record == 0 {
            return Ok(None);
        }
        if !self.shown.contains_key(&record) {
            let param = u32(&self.memory.read(record + RECORD_PARAM, 4)?, 0)?;
            let found = if param != 0 {
                pokemon::read(&*self.memory, &self.names, param, SPECIES_MAX)?
            } else {
                None
            };
            self.shown.insert(record, found);
        }
        Ok(self.shown[&record].clone())
    }

    /// The HP the gauge shows as a percentage of the maximum, or None with no maximum.
    fn share(&self, head: &[u8]) -> Result<Option<String>> {
        let maximum = u16(
            &self
                .memory
                .read(u32(head, GAUGE_RECORD)? + RECORD_MAX_HP, 2)?,
            0,
        )?;
        if maximum == 0 {
            return Ok(None);
        }
        let shown = u32(head, GAUGE_SHOWN_HP)?;
        // Rounded, 1 HP of 201 would be 0%, which is what fainting sounds like.
        let percent = if shown != 0 {
            py::round(shown as f64 / maximum as f64 * 100.0)?.max(1)
        } else {
            0
        };
        Ok(Some(own("battle-share", &[("percent", &percent)])))
    }
}

/// A panel's words as (name, level, current/maximum HP or None, the name's TextBox): the name
/// is its one piece that is not a number, the HP the row holding two numbers, the level the number
/// left over. None where the panel is not laid out so.
fn read(pieces: &[&ScreenText]) -> Result<Option<Reading>> {
    // A number is told by its text alone, so `piece not in numbers` of the Python, which compares
    // whole pieces, is the same as not being one.
    let is_number = |piece: &ScreenText| {
        let text = py::strip(&piece.text);
        !text.is_empty() && text.chars().all(isdigit)
    };
    let numbers: Vec<&ScreenText> = pieces
        .iter()
        .copied()
        .filter(|piece| is_number(piece))
        .collect();
    // The "/" between the HP's numbers is a piece of its own, and no name.
    let names: Vec<&ScreenText> = pieces
        .iter()
        .copied()
        .filter(|piece| !is_number(piece) && piece.text.chars().any(isalnum))
        .collect();
    if names.len() != 1 || !(1..=3).contains(&numbers.len()) {
        return Ok(None);
    }
    let name = names[0];
    let beside = |piece: &ScreenText| (piece.row - name.row).abs() < name.glyph / 2.0;
    let on_row: Vec<&ScreenText> = numbers
        .iter()
        .copied()
        .filter(|piece| beside(piece))
        .collect();
    let off_row: Vec<&ScreenText> = numbers
        .iter()
        .copied()
        .filter(|piece| !beside(piece))
        .collect();
    if on_row.len() != 1 || (!off_row.is_empty() && off_row.len() != 2) {
        return Ok(None);
    }
    let mut hp = None;
    if !off_row.is_empty() {
        // int() reads fullwidth digits, which the maximum is drawn in. Sorted by `left` as
        // Python's sort of two does, which swaps only when the second is strictly less.
        let (current, maximum) = if off_row[1].left < off_row[0].left {
            (off_row[1], off_row[0])
        } else {
            (off_row[0], off_row[1])
        };
        hp = Some(format!(
            "{}/{}",
            int_text(&current.text)?,
            int_text(&maximum.text)?
        ));
    }
    Ok(Some((
        py::strip(&name.text).to_string(),
        int_text(&on_row[0].text)?,
        hp,
        name.r#box,
    )))
}

/// The characters `str.isdigit()` accepts that are not decimal digits, which `int` then refuses:
/// superscripts, subscripts and the circled and parenthesised digits.
const OTHER_DIGITS: [(u32, u32); 20] = [
    (0xB2, 0xB3),
    (0xB9, 0xB9),
    (0x1369, 0x1371),
    (0x19DA, 0x19DA),
    (0x2070, 0x2070),
    (0x2074, 0x2079),
    (0x2080, 0x2089),
    (0x2460, 0x2468),
    (0x2474, 0x247C),
    (0x2488, 0x2490),
    (0x24EA, 0x24EA),
    (0x24F5, 0x24FD),
    (0x24FF, 0x24FF),
    (0x2776, 0x277E),
    (0x2780, 0x2788),
    (0x278A, 0x2792),
    (0x10A40, 0x10A43),
    (0x10E60, 0x10E68),
    (0x11052, 0x1105A),
    (0x1F100, 0x1F10A),
];

/// `str.isdigit()` for one character.
fn isdigit(c: char) -> bool {
    py::is_digit(c)
        || OTHER_DIGITS
            .iter()
            .any(|&(low, high)| (low..=high).contains(&(c as u32)))
}

/// `str.isalnum()` for one character: a letter or a number, by Unicode category.
fn isalnum(c: char) -> bool {
    use GeneralCategory::*;
    matches!(
        get_general_category(c),
        UppercaseLetter
            | LowercaseLetter
            | TitlecaseLetter
            | ModifierLetter
            | OtherLetter
            | DecimalNumber
            | LetterNumber
            | OtherNumber
    )
}

/// The value of a decimal digit of any script, as `int` reads one.
fn decimal(c: char) -> Option<u32> {
    let is_decimal = |c: char| get_general_category(c) == GeneralCategory::DecimalNumber;
    if !is_decimal(c) {
        return None;
    }
    // Decimal digits come in runs of ten from zero up, and one run can follow another directly.
    let mut start = c as u32;
    while start > 0 && char::from_u32(start - 1).is_some_and(is_decimal) {
        start -= 1;
    }
    Some((c as u32 - start) % 10)
}

/// `str(int(text))`: the number the text reads as, which the digits of any script may spell and
/// whitespace may surround, written in ASCII with no leading zeros.
fn int_text(text: &str) -> Result<String> {
    let digits: Option<String> = py::strip(text)
        .chars()
        .map(|c| decimal(c).and_then(|value| char::from_digit(value, 10)))
        .collect();
    match digits {
        Some(digits) if !digits.is_empty() => {
            let digits = digits.trim_start_matches('0');
            Ok(if digits.is_empty() { "0" } else { digits }.to_string())
        }
        _ => Err(Error::value(format!(
            "invalid literal for int() with base 10: {}",
            py::repr(text)
        ))),
    }
}

/// `head[at]`, raising `IndexError` where `head` stops short of it.
#[track_caller]
fn byte(head: &[u8], at: i64) -> Result<i64> {
    match usize::try_from(at).ok().and_then(|at| head.get(at)) {
        Some(&found) => Ok(i64::from(found)),
        None => Err(Error::index("index out of range")),
    }
}
