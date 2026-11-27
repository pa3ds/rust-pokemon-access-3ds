//! The stylus over Pokémon-Amie's Pokémon, and the picture of its parts it moves over (`amie`).
//!
//! Amie keeps a picture of the Pokémon as the bottom screen shows it, a grey a part and only what
//! faces the camera, and its script asks the grey under the stylus (spike 280). `Picture` reads it
//! at once, a part's runs down the columns, and a part's pixels that touch are a **patch**:
//! Pikachu's two cheeks, or its body and its tail.
//!
//! The stylus moves over a **still picture** of the Pokémon, taken as it appears
//! (`Stylus.settle`), as one left alone moves about a good deal, and a tap or drag goes to the part
//! chosen wherever the Pokémon has moved it to (`placed`). Nothing here reads memory:
//! `amie.AmiePage` hands the stylus the time, the love command and the picture as drawn now, as
//! calls it makes only where it needs them, and a test hands it pictures.
//!
//! Bound by:
//! - ADR-0009: a jump goes to a hand raised (as amended 2026-11-06), and a touch to the part named
//!   wherever it has moved (2026-11-06 again).

use std::cell::OnceCell;
use std::collections::HashMap;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::games::common::touch;
use crate::py::{self, Result};

/// How far apart the picture's columns are in its buffer.
pub const COLUMN: i64 = 0x400;
/// The grey `AmiePage` gives a pixel off the Pokémon, where the picture has none.
pub const OFF_PICTURE: i64 = -1;
/// What every pixel of the picture is before the Pokémon is first drawn, and no part: taken for
/// one, Amie opened with the stylus on a "part" filling the screen, and a Pokémon of any size offered
/// to be brought down (or_amie_interactions, the census's "empty" pictures).
pub const BLANK: i64 = 255;
/// A run of one part down a column, each pixel 0xFF and its grey three times; neither grey 0, off
/// the Pokémon, nor `BLANK`. Python's `RUNS`, `\xff([\x01-\xfe])\1\1(?:\xff\1\1\1)*`, which the
/// `regex` crate cannot write for its back-references: a match at `at`, its grey and its end.
fn run_at(data: &[u8], at: usize) -> Option<(u8, usize)> {
    let pixel = data.get(at..at + 4)?;
    let grey = pixel[1];
    if pixel[0] != 0xFF || !(0x01..=0xFE).contains(&grey) || pixel[2] != grey || pixel[3] != grey {
        return None;
    }
    let mut end = at + 4;
    while data.get(end..end + 4) == Some(&[0xFF, grey, grey, grey]) {
        end += 4;
    }
    Some((grey, end))
}
/// A run down a column: (x, top, bottom).
pub type Run = (i64, i64, i64);
/// Python's `Box`: (left, top, right, bottom).
pub type Box4 = (i64, i64, i64, i64);
/// The two ends a rub goes back and forth between.
type Ends = ((i64, i64), (i64, i64));
/// How long between looks at the picture as the still one is taken (`Stylus.settle`).
pub const LOOK: f64 = 0.5;
/// How far either way a run through a point is measured, so that a run across and one down are
/// measured alike.
pub const LONGEST_RUN: i64 = 32;
/// How far either side of the stylus a rub goes at most, on its part: at 20, rubbing Pikachu's
/// chin reached its cheeks (or_amie_moods).
pub const RUB_REACH: i64 = 12;
/// How far a rub keeps in from its part's edges, as the Pokémon sways under it.
pub const RUB_INSET: i64 = 3;
/// Half a rub at the least: 12 pixels across at `touch.RUB_SPEED` moves the hand 4 pixels a tick
/// of Amie's script and more, which it counts a pet by (spike 287); a part narrower than that is
/// rubbed a little past its edges, and Amie forgives two ticks off a part.
pub const RUB_HALF: i64 = 6;
pub const RICHER: f64 = 10.0;
/// How long after the Pokémon is first drawn the still picture may be replaced, and how many times
/// as many pixels take it there: Pikachu walked up to the player 8 s after Amie opened, a third
/// bigger, and any later a Pokémon idling can be a joyful leap at the camera (or_amie_interactions).
pub const BIGGER: f64 = 1.25;
/// How long after a press moved the Pokémon it is taken to have stopped.
pub const SETTLE: f64 = 1.0;
/// The most pixels of a part left out of the still picture as a speck, where there is more of it:
/// Pikachu Rock Star's head has a dozen of 1 to 4 pixels in its forehead, which a jump landed on and
/// called the head again (spike 287). A Pokémon's eyes can be nothing but specks: those stay.
pub const SPECK: i64 = 8;
/// How far inside its part a touch looks to be from every edge, `RUB_REACH` along it: a part is
/// only its grey to the game, so where on it is touched matters only for staying on it as the
/// Pokémon moves (2026-11-06: petting the head, he kept reaching the cheeks).
pub const DEEP: i64 = 12;
pub const OFF_SIDES: (i64, i64) = (24, 295);
/// Where a press off the Pokémon goes, at the screen's side nearer the stylus and between the
/// corners' buttons. A press beside the Pokémon can touch it as it sways, and raises no hand; held
/// at (40, 120) it always did (spike 287), and worked out from the Pokémon as it swayed the press
/// went back and forth 40 pixels (or_amie_interactions).
pub const OFF_ROWS: (i64, i64) = (60, 180);
/// Where the stylus is before the arrows first move it: the middle of the screen, where the
/// Pokémon stands.
pub const STYLUS_START: (i64, i64) = (160, 120);
/// How far an arrow moves the stylus, in the screen's pixels: Pikachu's cheek, the narrowest part
/// spike 287 stroked, was 9 to 11 across, so no step jumps one; the screen is 53 steps across.
pub const STEP: i64 = 6;
/// The grey a Pokémon high-fives with: a hand or arm on most, a mouth on a few (spike 287).
pub const HAND: i64 = 10;
/// `LoveLookTouch`: a press held off the Pokémon a few seconds raises its hand, longer the lower
/// its affection (Pikachu Rock Star's 60 took 4.8 s), which it waves about 2 s, about a second a
/// wave, and raises again 5 s later while the press stays; the stylus on grey 10 meanwhile is the
/// high five (spike 287, live).
pub const HAND_RAISED: i64 = 24;
/// `LoveAppear`, the ways a Pokémon comes into Amie, chosen as it opens from tables of the
/// script's: 7 with its back turned, and 6 where the Pokémon called the player
/// (`CheckCalledByMonster`). Seen started at will, 1 to 6 each leaned up close, gazing at the stylus
/// (spike 287's `look`). A `frozenset` of `range(1, 8)` in Python.
pub const APPEARING: std::ops::Range<i64> = 1..8;

/// Pixels of a part that touch, as runs down the columns: (x, top, bottom).
#[derive(Debug, Clone, PartialEq)]
pub struct Patch {
    pub runs: Vec<Run>,
    pub size: i64,
    pub r#box: Box4,
}

impl Patch {
    pub fn new(runs: Vec<Run>) -> Patch {
        let size = runs.iter().map(|&(_x, top, bottom)| bottom - top + 1).sum();
        let r#box = (
            runs.iter().map(|run| run.0).min().expect("a run"),
            runs.iter().map(|run| run.1).min().expect("a run"),
            runs.iter().map(|run| run.0).max().expect("a run"),
            runs.iter().map(|run| run.2).max().expect("a run"),
        );
        Patch { runs, size, r#box }
    }

    pub fn middle(&self) -> (i64, i64) {
        let (left, top, right, bottom) = self.r#box;
        (py::floordiv(left + right, 2), py::floordiv(top + bottom, 2))
    }
}

/// The picture of parts as read at once: its runs in one pass of a regular expression, as a
/// touch reads it ten times a second, and a Python loop over its pixels took 20 ms.
pub struct Picture {
    pub data: Vec<u8>,
    /// `runs`, `size` and `outline`, Python's `cached_property`s.
    runs: OnceCell<IndexMap<i64, Vec<Run>>>,
    size: OnceCell<i64>,
    outline: OnceCell<Option<Box4>>,
}

impl Picture {
    pub fn new(data: Vec<u8>) -> Self {
        Picture {
            data,
            runs: OnceCell::new(),
            size: OnceCell::new(),
            outline: OnceCell::new(),
        }
    }

    /// Each part's runs down the columns, by its grey.
    pub fn runs(&self) -> &IndexMap<i64, Vec<Run>> {
        self.runs.get_or_init(|| {
            let mut found: IndexMap<i64, Vec<Run>> = IndexMap::new();
            // `RUNS.finditer`: the leftmost match on from where the last one ended.
            let mut at = 0;
            while at < self.data.len() {
                let Some((grey, end)) = run_at(&self.data, at) else {
                    at += 1;
                    continue;
                };
                if at % 4 == 0 {
                    let (x, low) = (at as i64 / COLUMN, at as i64 % COLUMN);
                    let high = (low + (end - at) as i64).min(touch::HEIGHT * 4) - 4;
                    found.entry(i64::from(grey)).or_default().push((
                        x,
                        touch::HEIGHT - 1 - py::floordiv(high, 4),
                        touch::HEIGHT - 1 - py::floordiv(low, 4),
                    ));
                }
                at = end;
            }
            found
        })
    }

    /// How many pixels of the Pokémon are drawn.
    pub fn size(&self) -> i64 {
        *self.size.get_or_init(|| {
            self.runs()
                .values()
                .flatten()
                .map(|&(_x, top, bottom)| bottom - top + 1)
                .sum()
        })
    }

    /// The box the Pokémon is drawn in, (left, top, right, bottom), or None.
    pub fn outline(&self) -> Option<Box4> {
        *self.outline.get_or_init(|| {
            let runs: Vec<Run> = self.runs().values().flatten().copied().collect();
            (!runs.is_empty()).then(|| Patch::new(runs).r#box)
        })
    }

    /// The grey drawn at a pixel, `OFF_PICTURE` off the Pokémon, or None off the screen.
    pub fn grey(&self, x: i64, y: i64) -> Option<i64> {
        if !((0..touch::WIDTH).contains(&x) && (0..touch::HEIGHT).contains(&y)) {
            return None;
        }
        let at = (x * COLUMN + (touch::HEIGHT - 1 - y) * 4) as usize;
        let [alpha, grey, green, blue] = self.data[at..at + 4] else {
            unreachable!("four bytes");
        };
        let grey = i64::from(grey);
        Some(
            if alpha == 0xFF && grey == i64::from(green) && green == blue && grey != BLANK {
                grey
            } else {
                OFF_PICTURE
            },
        )
    }

    /// The first and last pixel of the run of one grey through (x, y), across the screen or
    /// down it, `LONGEST_RUN` either way at most.
    pub fn run(&self, x: i64, y: i64, across: bool) -> (i64, i64) {
        let (grey, mut ends) = (self.grey(x, y), Vec::new());
        for way in [-1, 1] {
            let start = if across { x } else { y };
            let mut at = start;
            while (at + way - start).abs() <= LONGEST_RUN
                && (if across {
                    self.grey(at + way, y)
                } else {
                    self.grey(x, at + way)
                }) == grey
            {
                at += way;
            }
            ends.push(at);
        }
        (ends[0], ends[1])
    }

    /// The pixel of a grey nearest (x, y), or None where none is drawn.
    #[allow(clippy::manual_clamp)] // a run's top can lie below its bottom, which `clamp` refuses
    pub fn closest(&self, x: i64, y: i64, grey: i64) -> Option<(i64, i64)> {
        let mut found: Option<((i64, i64), i64)> = None;
        for &(at, top, bottom) in self.runs().get(&grey).map_or(&[][..], Vec::as_slice) {
            let spot = (at, y.max(top).min(bottom));
            let distance = (spot.0 - x).pow(2) + (spot.1 - y).pow(2);
            if found.is_none_or(|(_, best)| distance < best) {
                found = Some((spot, distance));
            }
        }
        found.map(|(spot, _)| spot)
    }

    /// A part's patches, the largest first: Pikachu's two cheeks, or its body and its tail.
    pub fn patches(&self, grey: i64) -> Vec<Patch> {
        let mut runs: Vec<Run> = self.runs().get(&grey).cloned().unwrap_or_default();
        runs.sort();
        let mut parent: Vec<usize> = (0..runs.len()).collect();

        fn root(parent: &mut [usize], mut at: usize) -> usize {
            while parent[at] != at {
                parent[at] = parent[parent[at]];
                at = parent[at];
            }
            at
        }

        let mut first = 0;
        for at in 0..runs.len() {
            let (x, top, bottom) = runs[at];
            while runs[first].0 < x - 1 {
                first += 1;
            }
            for other in first..at {
                let (beside, other_top, other_bottom) = runs[other];
                if beside == x - 1 && other_top <= bottom && top <= other_bottom {
                    // Python's `parent[root(other)] = root(at)`: the right-hand side first.
                    let to = root(&mut parent, at);
                    let from = root(&mut parent, other);
                    parent[from] = to;
                }
            }
        }
        let mut groups: IndexMap<usize, Vec<Run>> = IndexMap::new();
        for (at, &run) in runs.iter().enumerate() {
            groups.entry(root(&mut parent, at)).or_default().push(run);
        }
        let mut patches: Vec<Patch> = groups.into_values().map(Patch::new).collect();
        // Python's `sorted` by an int key, which is stable, as `sort_by_key` is.
        patches.sort_by_key(|patch| -patch.size);
        patches
    }

    /// A part's patches without its specks, or with them where it is nothing else.
    pub fn solid(&self, grey: i64) -> Vec<Patch> {
        let found = self.patches(grey);
        let solid: Vec<Patch> = found
            .iter()
            .filter(|patch| patch.size > SPECK)
            .cloned()
            .collect();
        if solid.is_empty() { found } else { solid }
    }

    /// The patch of a grey at (x, y), or None where that grey is not drawn there.
    pub fn patch_at(&self, x: i64, y: i64, grey: i64) -> Option<Patch> {
        self.patches(grey).into_iter().find(|patch| {
            patch
                .runs
                .iter()
                .any(|&(at, top, bottom)| at == x && top <= y && y <= bottom)
        })
    }

    /// Where on a patch a touch goes: `DEEP` inside it, or as far as it gets, nearest (x, y).
    pub fn heart(&self, patch: &Patch, x: i64, y: i64) -> (i64, i64) {
        let mut best: Option<((i64, i64), (i64, i64))> = None;
        for &(at, top, bottom) in patch.runs.iter().step_by(3) {
            let spot = (at, py::floordiv(top + bottom, 2));
            let key = (
                self.depth(spot.0, spot.1),
                -(spot.0 - x).pow(2) - (spot.1 - y).pow(2),
            );
            // Python's `max`: the first of equal bests.
            if best.is_none_or(|(_, most)| key > most) {
                best = Some((spot, key));
            }
        }
        best.expect("a patch has a run").0
    }

    /// How far (x, y) is inside its part every way, `DEEP` at most.
    fn depth(&self, x: i64, y: i64) -> i64 {
        let grey = self.grey(x, y);
        for reach in 1..DEEP + 1 {
            if [
                (x - reach, y),
                (x + reach, y),
                (x, y - reach),
                (x, y + reach),
            ]
            .iter()
            .any(|&(px, py)| self.grey(px, py) != grey)
            {
                return reach - 1;
            }
        }
        DEEP
    }

    /// The picture without specks of a part with more to it, each given what lies round it.
    pub fn cleaned(&self) -> Picture {
        let mut data = self.data.clone();
        for &grey in self.runs().keys() {
            let found = self.patches(grey);
            if found[0].size <= SPECK {
                continue;
            }
            for patch in &found {
                if patch.size > SPECK {
                    continue;
                }
                for &(x, top, bottom) in &patch.runs {
                    let around = [
                        self.grey(x - 1, top),
                        self.grey(x + 1, top),
                        self.grey(x, top - 1),
                        self.grey(x, bottom + 1),
                    ]
                    .into_iter()
                    .flatten()
                    .find(|&other| other != OFF_PICTURE && other != grey)
                    .unwrap_or(0) as u8;
                    for y in top..bottom + 1 {
                        let at = (x * COLUMN + (touch::HEIGHT - 1 - y) * 4) as usize;
                        data[at..at + 4].copy_from_slice(&[0xFF, around, around, around]);
                    }
                }
            }
        }
        Picture::new(data)
    }
}

/// (x, y) in one box as far across and down another.
pub fn scaled(x: i64, y: i64, old: Box4, new: Box4) -> (i64, i64) {
    (
        new.0 + py::floordiv((x - old.0) * (new.2 - new.0), (old.2 - old.0).max(1)),
        new.1 + py::floordiv((y - old.1) * (new.3 - new.1), (old.3 - old.1).max(1)),
    )
}

/// Where (x, y) on a part in one picture is in another, wherever the part has gone: on the
/// patch of it lying most nearly where that one lay on the Pokémon, the other cheek not being the
/// cheek, as far across and down it, and well inside it; where the other picture has none of it,
/// as far across and down the whole Pokémon. On or_amie_interactions, where a part's box took in
/// both its patches, a rub on Pikachu's second cheek stayed on it 72% of the time, and so 95%.
pub fn placed(before: &Picture, after: &Picture, x: i64, y: i64, grey: i64) -> (i64, i64) {
    let (Some(old), Some(new)) = (before.outline(), after.outline()) else {
        return (x, y);
    };
    let patches = after.solid(grey);
    if patches.is_empty() {
        return scaled(x, y, old, new);
    }
    let was = before.patch_at(x, y, grey);
    let (wx, wy) = was.as_ref().map_or((x, y), Patch::middle);
    let here = scaled(wx, wy, old, new);
    // Python's `min` by a key: the first of equal bests.
    let mut nearest: Option<(&Patch, i64)> = None;
    for found in &patches {
        let middle = found.middle();
        let distance = (middle.0 - here.0).pow(2) + (middle.1 - here.1).pow(2);
        if nearest.is_none_or(|(_, best)| distance < best) {
            nearest = Some((found, distance));
        }
    }
    let patch = nearest.expect("a patch").0;
    let (tx, ty) = match &was {
        Some(was) => scaled(x, y, was.r#box, patch.r#box),
        None => here,
    };
    after.heart(patch, tx, ty)
}

/// The stylus over the still picture of Amie's Pokémon (ADR-0009 as amended 2026-11-06):
/// where it is, the still picture it moves over, and the part a touch on it goes to, wherever
/// the Pokémon has moved that since (`placed`). It reads nothing: it is handed the time, and the
/// love command and the picture as it is now as calls it makes only where it needs them, so a
/// poll reads what it read before it had a module of its own, and a test hands it pictures.
pub struct Stylus {
    /// Where it is on the still picture, as the arrows have moved it (`StylusKeys`).
    pub at: (i64, i64),
    /// The grey of the part a touch goes to, `OFF_PICTURE` for none, None before the stylus
    /// has been on any.
    kept: Option<i64>,
    /// The still picture it moves over, none before the Pokémon is drawn.
    still: Option<Rc<Picture>>,
    /// Whether the next picture taken replaces the still one, whatever it shows.
    pub anew: bool,
    /// Until when a picture showing more of the Pokémon replaces the still one; `inf` until
    /// it has been drawn.
    richer_until: f64,
    /// When the still picture is next looked at.
    next_look: f64,
}

impl Default for Stylus {
    fn default() -> Self {
        Stylus::new()
    }
}

impl Stylus {
    pub fn new() -> Self {
        Stylus {
            at: STYLUS_START,
            kept: None,
            still: None,
            anew: false,
            richer_until: f64::INFINITY,
            next_look: 0.0,
        }
    }

    /// The grey under it, `OFF_PICTURE` off the Pokémon, or None before a picture is taken.
    pub fn grey(&self) -> Option<i64> {
        self.still
            .as_ref()
            .and_then(|still| still.grey(self.at.0, self.at.1))
    }

    /// The box the Pokémon is drawn in on the still picture, (left, top, right, bottom), or
    /// None. The arrows keep the stylus within a step of it: across the whole screen it was too
    /// many presses.
    pub fn outline(&self) -> Option<Box4> {
        self.still.as_ref().and_then(|still| still.outline())
    }

    /// Put it at a point, a touch going to a grey there, or to the part under it; to nothing
    /// where it is off the Pokémon. Python's `grey` defaults to None.
    pub fn place(&mut self, at: (i64, i64), grey: Option<i64>) {
        self.at = at;
        let grey = match grey {
            None => self.grey(),
            grey => grey,
        };
        if let Some(grey) = grey {
            self.kept = Some(if grey == 0 || grey == OFF_PICTURE {
                OFF_PICTURE
            } else {
                grey
            });
        }
    }

    /// Move it `STEP` pixels that way, kept within a step of the Pokémon (`outline`) and on
    /// the screen; False where that moves it nowhere.
    pub fn step(&mut self, across: i64, down: i64) -> bool {
        let (x, y) = self.at;
        let (left, top, right, bottom) =
            self.outline()
                .unwrap_or((0, 0, touch::WIDTH - 1, touch::HEIGHT - 1));
        let moved = (
            (x + across * STEP)
                .max(left - STEP)
                .max(0)
                .min(right + STEP)
                .min(touch::WIDTH - 1),
            (y + down * STEP)
                .max(top - STEP)
                .max(0)
                .min(bottom + STEP)
                .min(touch::HEIGHT - 1),
        );
        if moved == (x, y) {
            return false;
        }
        self.place(moved, None);
        true
    }

    /// Jump it the way given past the part it is on, any part said as that one is (`said`,
    /// each grey's words), and any gap, to the middle of the next part's run that way, as Ctrl
    /// and an arrow move it; False where no other part lies that way, or no picture is taken.
    /// Sweet spots and danger zones with no name of their own are said as the head or the body,
    /// which a jump went from and to, saying "head" twice.
    pub fn jump(
        &mut self,
        across: i64,
        down: i64,
        said: &mut dyn FnMut(Option<i64>) -> Result<Option<String>>,
    ) -> Result<bool> {
        let Some(whole) = self.still.clone() else {
            return Ok(false);
        };
        let mut words: HashMap<Option<i64>, Option<String>> = HashMap::new();
        let mut part = |grey: Option<i64>| -> Result<Option<String>> {
            if let Some(found) = words.get(&grey) {
                return Ok(found.clone());
            }
            let found = said(grey)?;
            words.insert(grey, found.clone());
            Ok(found)
        };
        let (mut x, mut y) = self.at;
        let here = part(whole.grey(x, y))?;
        while let Some(grey) = whole.grey(x + across, y + down) {
            (x, y) = (x + across, y + down);
            if grey != 0 && grey != OFF_PICTURE && part(Some(grey))? != here {
                let first = if across != 0 { x } else { y };
                while whole.grey(x + across, y + down) == Some(grey) {
                    (x, y) = (x + across, y + down);
                }
                let middle = py::floordiv(first + if across != 0 { x } else { y }, 2);
                self.place(
                    if across != 0 {
                        (middle, y)
                    } else {
                        (x, middle)
                    },
                    None,
                );
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Jump it to the Pokémon's hand while it is raised, whichever way: a player cannot see
    /// where it is, and it is up about 2 s (`HAND_RAISED`). The middle of the run across of the
    /// hand's pixel nearest the stylus, in the picture as it is now, as the hand waves, or on the
    /// still picture where that shows a hand, the touch going to the hand as it is now
    /// (`placed`); False where it is not raised or not drawn, or the picture cannot be read.
    pub fn to_hand(
        &mut self,
        mood: impl FnOnce() -> Result<Option<i64>>,
        live: impl FnOnce() -> Result<Option<Rc<Picture>>>,
    ) -> Result<bool> {
        if mood()? != Some(HAND_RAISED) {
            return Ok(false);
        }
        let (x, y) = self.at;
        let Some((whole, found)) =
            live()?.and_then(|whole| whole.closest(x, y, HAND).map(|found| (whole, found)))
        else {
            return Ok(false);
        };
        let (left, right) = whole.run(found.0, found.1, true);
        let hand = (py::floordiv(left + right, 2), found.1);
        let at = self
            .still
            .as_ref()
            .and_then(|still| still.closest(x, y, HAND))
            .unwrap_or(hand);
        self.place(at, Some(HAND));
        Ok(true)
    }

    /// Take the still picture, a `LOOK` apart: the first drawing the Pokémon, then for
    /// `RICHER` any showing more of its parts, or more of it, as it idles, as its first frames
    /// may catch it looking away or still walking up. It is left until the Pokémon comes anew,
    /// with its back turned, or `look_again` asks. Left alone Pikachu's box moved 85 pixels in
    /// 40 s, so that a part found with the arrows had gone by the next press (spike 287's
    /// `watch`); this holds them still, and a touch finds its part wherever it has
    /// gone (`placed`).
    pub fn settle(
        &mut self,
        now: f64,
        mood: impl FnOnce() -> Result<Option<i64>>,
        live: impl FnOnce() -> Result<Option<Rc<Picture>>>,
    ) -> Result<()> {
        if now < self.next_look {
            return Ok(());
        }
        self.next_look = now + LOOK;
        let love = mood()?;
        if love.is_some_and(|love| APPEARING.contains(&love)) {
            (self.still, self.richer_until) = (None, f64::INFINITY);
            return Ok(());
        }
        let still = self.still.clone();
        if now > self.richer_until || (still.is_some() && love != Some(0) && !self.anew) {
            return Ok(());
        }
        let Some(whole) = live()? else {
            return Ok(());
        };
        if whole.runs().is_empty() {
            return Ok(());
        }
        if self.anew
            || still.as_ref().is_none_or(|still| {
                whole.runs().len() > still.runs().len()
                    || whole.size() as f64 > still.size() as f64 * BIGGER
            })
        {
            self.take(Rc::new(whole.cleaned()));
            if self.richer_until == f64::INFINITY {
                self.richer_until = now + RICHER;
            }
        }
        Ok(())
    }

    /// Make a picture the still one, the stylus staying on its part.
    fn take(&mut self, whole: Rc<Picture>) {
        if let Some(still) = &self.still
            && let Some(kept) = self.kept
            && kept != OFF_PICTURE
        {
            self.at = placed(still, &whole, self.at.0, self.at.1, kept);
        }
        (self.still, self.anew) = (Some(whole), false);
        self.place(self.at, None);
    }

    /// Take the still picture anew once a press of `seconds` has moved the Pokémon and it has
    /// stopped: a beckon brings it close, and a press at an edge brings it between the screens.
    /// Python's `seconds` defaults to 0.
    pub fn look_again(&mut self, now: f64, seconds: f64) {
        (self.anew, self.richer_until) = (true, f64::INFINITY);
        self.next_look = now + seconds + SETTLE;
    }

    /// Leave the still picture behind as Amie closes, or one of its games opens.
    pub fn forget(&mut self) {
        (self.still, self.anew, self.richer_until) = (None, false, f64::INFINITY);
    }

    /// Where a rub with it goes: to its part as it is now (`placed`), back and forth across it
    /// the way it is longer, `RUB_REACH` at most either side and `RUB_HALF` at least; where the
    /// part cannot be found, `RUB_HALF` either side of where it is. Off the Pokémon it stays
    /// still: a press held there raises its hand, and any rub reaching back onto it was taken for
    /// a press on it, which never does (spike 287, live).
    pub fn rub(&self, live: impl FnOnce() -> Result<Option<Rc<Picture>>>) -> Result<Ends> {
        let (x, y) = self.at;
        let seen = if self.still.is_some() { live()? } else { None };
        let (Some(seen), Some(still), Some(kept)) = (seen, &self.still, self.kept) else {
            return Ok(((x - RUB_HALF, y), (x + RUB_HALF, y)));
        };
        if kept == OFF_PICTURE {
            let point = self.off(&seen);
            return Ok((point, point));
        }
        let (x, y) = placed(still, &seen, x, y, kept);
        if matches!(seen.grey(x, y), None | Some(0 | OFF_PICTURE)) {
            return Ok(((x - RUB_HALF, y), (x + RUB_HALF, y)));
        }
        let (across, down) = (seen.run(x, y, true), seen.run(x, y, false));
        let sideways = across.1 - across.0 >= down.1 - down.0;
        let ((low, high), at) = if sideways { (across, x) } else { (down, y) };
        let half = RUB_REACH.min(RUB_HALF.max(py::floordiv(high - low, 2) - RUB_INSET));
        let middle = if high - low < 2 * half {
            py::floordiv(low + high, 2)
        } else {
            at.max(low + half).min(high - half)
        };
        if sideways {
            return Ok(((middle - half, y), (middle + half, y)));
        }
        Ok(((x, middle - half), (x, middle + half)))
    }

    /// Where a press off the Pokémon goes: the screen's side nearer the stylus, or the other
    /// where the Pokémon stands, and where the stylus is where it stands at both.
    fn off(&self, live: &Picture) -> (i64, i64) {
        let (x, y) = self.at;
        let row = y.clamp(OFF_ROWS.0, OFF_ROWS.1);
        let mut sides = [OFF_SIDES.0, OFF_SIDES.1];
        // Python's `sorted` by an int key, which is stable, as `sort_by_key` is.
        sides.sort_by_key(|side| (side - x).abs());
        sides
            .into_iter()
            .find(|&side| matches!(live.grey(side, row), Some(0 | OFF_PICTURE)))
            .map_or((x, y), |side| (side, row))
    }
}
