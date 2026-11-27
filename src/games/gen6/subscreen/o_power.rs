//! O-Power: the energy the PSS's O-Power list spends, and the powers running, said by R
//! (spike 313).
//!
//! ```text
//!     savedata::AssistPowerData   the save's O-Power record, in the heap from before the title screen
//!         +0x68  u32 the energy, 0 to 10
//! ```
//!
//! Using Hatching, Bargain, Prize Money and Encounter Power took it 10, 8, 6, 2, 0, each as the game
//! charged the power's cost (or_pss_o_power); Y's record keeps it at the same place. The O-Power
//! list draws it as ten orbs along its bottom, pictures whose inner part stays shown, as it empties,
//! for a while after the orb is spent, and named nothing: read from the record, it is the number the
//! orbs settle on. Beside them a clock icon and a digit, a picture of its own, `number_1` with
//! Azahar counting no steps and `number_4` at 9000 an hour: the energy's regen rate, which goes up
//! with the steps counted that day, said after the energy where the list is up.
//!
//! The powers running are words of the top screen's Active panel, `assist_power_upper.bclyt`, up
//! while the PSS is: "Active", the kind of power, and a row for each, "Hatching Power", "Lv. 1",
//! "02:58", its time left ticking each second, left unsaid as a clock's tick
//! (`common.screen_announce`). With field and battle powers both running, the panel turns between
//! a page of each every six seconds by itself, its rows written again into the same TextBoxes
//! (or_pss_o_power_2): each page is kept as it shows (`poll`). R says the energy and then each page
//! while O-Power's list or the panel is up, the energy first, a page not showing as it last showed,
//! its times counted down by the seconds since.
//!
//! Bound by:
//! - ADR-0013: the Active panel's turns are left unsaid, and R says each of its pages.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::discovery::Found;
use crate::core::memory::Memory;
use crate::core::words::own;
use crate::games::common::screen::{ScreenText, distinct, in_rows, join};
use crate::games::common::screen_announce::CLOCK;
use crate::games::common::touch::{FileOf, OnScreen};
use crate::py::{self, Result};

pub const CLASS: &str = "N8savedata15AssistPowerDataE";
pub const ENERGY: i64 = 0x68;
/// The orbs the list draws, the most energy there is.
pub const GAUGE: i64 = 10;
pub const LIST: &str = "assist_power_lower.bclyt";
pub const ACTIVE: &str = "assist_power_upper.bclyt";
pub const ENERGY_WORDS: &str = "o-power-energy";
/// How the texture of the digit beside the list's gauge's clock icon begins, `number_1` to
/// `number_4`: the energy's regen rate, by the steps counted that day (spike 313 again).
pub const REGEN: &str = "number_";
pub const REGEN_WORDS: &str = "o-power-regen";
/// Seconds a page of the Active panel is kept for R after it last showed: two of its turns.
pub const PAGE_SECONDS: f64 = 15.0;

/// `Callable[[], Sequence[int]]`: the roots of the layouts drawn now. It reads memory.
pub type Shown = Rc<dyn Fn() -> Result<Vec<i64>>>;
/// `Callable[[int], Sequence[str]]`: the textures the pictures shown under a layout's root draw.
/// It reads memory.
pub type Pictures = Rc<dyn Fn(i64) -> Result<Vec<String>>>;

/// The save's O-Power record and the screens that show it.
pub struct OPower {
    memory: Rc<dyn Memory>,
    on_screen: OnScreen,
    file_of: FileOf,
    /// The roots of the layouts drawn now, on either screen.
    shown: Shown,
    /// The textures the pictures shown under a layout's root draw.
    pictures: Pictures,
    pauses: bool,
    clock: Rc<dyn Fn() -> f64>,
    records: Vec<i64>,
    names: IndexMap<i64, String>,
    /// The Active panel's pages by their heading, "Active Field Power", in the order first
    /// shown, with when each last showed and its words then.
    pages: IndexMap<String, (f64, Vec<ScreenText>)>,
}

impl OPower {
    /// `OPower(memory, on_screen, file_of, shown, pictures, pauses)`; Python's `pauses` defaults
    /// to False and its `clock` to `time.monotonic`.
    pub fn new(
        memory: Rc<dyn Memory>,
        on_screen: OnScreen,
        file_of: FileOf,
        shown: Shown,
        pictures: Pictures,
        pauses: bool,
    ) -> Self {
        OPower::with_clock(
            memory,
            on_screen,
            file_of,
            shown,
            pictures,
            pauses,
            Rc::new(py::time::monotonic),
        )
    }

    /// `OPower(memory, on_screen, file_of, shown, pictures, pauses, clock)`.
    pub fn with_clock(
        memory: Rc<dyn Memory>,
        on_screen: OnScreen,
        file_of: FileOf,
        shown: Shown,
        pictures: Pictures,
        pauses: bool,
        clock: Rc<dyn Fn() -> f64>,
    ) -> Self {
        OPower {
            memory,
            on_screen,
            file_of,
            shown,
            pictures,
            pauses,
            clock,
            records: Vec::new(),
            names: IndexMap::new(),
            pages: IndexMap::new(),
        }
    }

    /// Keep the record a sweep found, while a sweep finds one.
    pub fn track(&mut self, found: &Found) {
        let records = found.of(&[CLASS]);
        if !records.is_empty() {
            self.records = records;
        }
    }

    /// Whether the Active panel or O-Power's list is on screen.
    pub fn up(&mut self) -> bool {
        let on_screen = (self.on_screen)();
        self.files(&on_screen)
            .values()
            .any(|name| name == LIST || name == ACTIVE)
    }

    /// Keep the page of the Active panel showing, from the words on screen.
    pub fn poll(&mut self) {
        let on_screen = (self.on_screen)();
        let files = self.files(&on_screen).clone();
        let panel = distinct(in_rows(
            on_screen
                .into_iter()
                .filter(|piece| files.get(&piece.root).map(String::as_str) == Some(ACTIVE)),
        ));
        let Some(first) = panel.first() else {
            return;
        };
        // `max(first.glyph, 1)`: the glyph unless 1 is greater.
        let glyph = if 1.0 > first.glyph { 1.0 } else { first.glyph };
        let heading: Vec<&str> = panel
            .iter()
            .filter(|piece| (piece.row - first.row).abs() < glyph / 2.0)
            .map(|piece| piece.text.as_str())
            .collect();
        let heading = heading.join(" ");
        let now = (self.clock)();
        self.pages.insert(heading, (now, panel));
    }

    /// The energy, the regen rate, then each page of the Active panel, while O-Power's list or
    /// the panel is up; None elsewhere.
    pub fn said(&mut self) -> Result<Option<String>> {
        let mut files: IndexMap<i64, String> = IndexMap::new();
        for root in (self.shown)()? {
            if let Some(file) = (self.file_of)(root) {
                files.insert(root, file.name.clone());
            }
        }
        if !files.values().any(|name| name == LIST || name == ACTIVE) {
            return Ok(None);
        }
        let mut parts: Vec<String> = Vec::new();
        if let Some(energy) = self.energy()? {
            parts.push(own(ENERGY_WORDS, &[("count", &energy)]));
        }
        let mut rates: Vec<i64> = Vec::new();
        for (&root, name) in &files {
            if name != LIST {
                continue;
            }
            for image in (self.pictures)(root)? {
                let Some(rest) = image.strip_prefix(REGEN) else {
                    continue;
                };
                // `partition(".")[0]`: up to the first dot, or all of it.
                let digits = rest.split('.').next().unwrap_or_default();
                if py::isdigit(digits) {
                    rates.push(py::int_base10(digits)?);
                }
            }
        }
        if let Some(rate) = rates.first() {
            parts.push(own(REGEN_WORDS, &[("rate", rate)]));
        }
        self.poll();
        let now = (self.clock)();
        for (seen, panel) in self.pages.values() {
            if now - seen <= PAGE_SECONDS {
                parts.push(join(
                    &counted_down(panel, py::int(now - seen)?)?,
                    self.pauses,
                ));
            }
        }
        let said = parts.join(if self.pauses { ", " } else { " " });
        Ok(if said.is_empty() { None } else { Some(said) })
    }

    /// The energy the record holds, or None where no record is known or it holds more than
    /// the gauge.
    pub fn energy(&self) -> Result<Option<i64>> {
        for &record in &self.records {
            let energy = u32(&self.memory.read(record + ENERGY, 4)?, 0)?;
            if energy <= GAUGE {
                return Ok(Some(energy));
            }
        }
        Ok(None)
    }

    /// The name of the layout file of each piece's root where it is known, kept while the
    /// root is on screen, as `FilesOnScreen` keeps them: asked every poll, for `up`.
    ///
    /// Python's `roots` is a set, its order reaching only the order names are kept in, which
    /// nothing reads: a `HashSet`.
    fn files(&mut self, pieces: &[ScreenText]) -> &IndexMap<i64, String> {
        let roots: HashSet<i64> = pieces.iter().map(|piece| piece.root).collect();
        self.names.retain(|root, _name| roots.contains(root));
        for root in roots {
            if self.names.contains_key(&root) {
                continue;
            }
            if let Some(file) = (self.file_of)(root) {
                self.names.insert(root, file.name.clone());
            }
        }
        &self.names
    }
}

/// The panel's words with each time left a clock alone shows, "02:24", counted down by
/// `seconds`, to no less than none.
fn counted_down(panel: &[ScreenText], seconds: i64) -> Result<Vec<ScreenText>> {
    if seconds == 0 {
        return Ok(panel.to_vec());
    }
    let mut out: Vec<ScreenText> = Vec::new();
    for piece in panel {
        let mut piece = piece.clone();
        if CLOCK.is_match(&piece.text) {
            let (minutes, left) = piece
                .text
                .split_once(':')
                .expect("a clock has its colon");
            let total = 0.max(py::int_base10(minutes)? * 60 + py::int_base10(left)? - seconds);
            piece.text = format!(
                "{:0width$}:{:02}",
                total / 60,
                total % 60,
                width = minutes.chars().count()
            );
        }
        out.push(piece);
    }
    Ok(out)
}
