//! Touch mode: the bottom screen's touch targets, walked by name, and a tap on the one chosen.
//!
//! The PokéNav Plus is touched, never walked with a cursor (ADR-0009). Each
//! of its buttons is a touch area, an `nw::lyt::Bounding`, which pa3ds finds and places as it does
//! the PC Box's slots (spike 129); so is every other screen's. In touch mode the navigation keys
//! walk the touch areas drawn on the bottom screen, in reading order, row by row, saying each by
//! name; and a key taps the one chosen, through `pa3ds.core.touch` (ADR-0009, amended 2026-10-01:
//! the one thing pa3ds does to the game, on the player's key, on the target it has just named). Or,
//! on a screen that drags, Shift alone holds a press on it, a **drag** (`DRAG`), and the press goes
//! with each target the keys choose after, until it is let go (`drop`): Pokémon-Amie's Poké Puffs
//! are dragged to the Pokémon (ADR-0009). A game's profile says which screens need a touch at all
//! (`TouchTargets`' `offered`): on the rest, which the game's buttons work, touch mode has nothing
//! to touch. What the targets are, and what each is called, is `touch_targets`'.
//!
//! Bound by:
//! - ADR-0009: a tap, or a drag, only on the player's keys, and only on the target just named.

use std::cell::Cell;
use std::rc::Rc;

use crate::core::config::{DRAG_TOGGLE, Touch};
use crate::core::touch::TouchScreen;
use crate::core::words::own;
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::screen::ScreenText;
use crate::py::{self, Log, Result};

pub const WIDTH: i64 = 320;
/// The touch screen in pixels; a layout places its panes from its middle, y up.
pub const HEIGHT: i64 = 240;

pub const ITEM_PREVIOUS: &str = "item previous";
pub const ITEM_NEXT: &str = "item next";
pub const ITEM_READ: &str = "item read";
pub const ROW_PREVIOUS: &str = "row previous";
pub const ROW_NEXT: &str = "row next";
pub const TAP: &str = "tap";
/// Hold a press on the target chosen, carried to each target the keys choose until `drop`: Shift
/// alone's, bound to no key (`Navigator.press_key`).
pub const DRAG: &str = "drag";
/// What the keys do in touch mode; the names are the navigation keys' own where they match.
///
/// A `frozenset` in Python, only ever tested for membership.
pub const ACTIONS: [&str; 7] = [
    ITEM_PREVIOUS,
    ITEM_NEXT,
    ITEM_READ,
    ROW_PREVIOUS,
    ROW_NEXT,
    TAP,
    DRAG,
];
/// Said where the bottom screen has nothing to touch.
pub const NOTHING: &str = "touch-nothing";
/// Said by a key bound to drag where nothing drags.
pub const NO_DRAG: &str = "touch-no-drag";
pub const HOLDING: &str = "touch-holding";
pub const DROPPED: &str = "touch-dropped";
/// How long a drag made as it is let go takes at full speed (`delayed`), a third pressed where it
/// was picked up, a third sliding, a third on the target, and as much longer as the game runs
/// slower (`pace`): a player lowers Azahar's speed limit for a higher score. It was asked for
/// shorter than the first 0.6 s (2026-10-29), and in rounds played live by touch mode's keys
/// (spike 260's `--keys`) it fed 32 berries of 33 drags at 0.6 s, 31 of 35 at 0.3, 42 of 46 at
/// 0.15, the quicker giving the more before each Pokémon left, and 12 of 31 at 0.05, where each
/// third is a game frame and a half.
pub const DROP_SECONDS: f64 = 0.15;
/// How long a tap on a target touched by rubbing rubs; another tap before it ends carries the rub
/// on, never lifting.
pub const RUB_SECONDS: f64 = 1.0;
/// How long a drag held on such a target waits before it rubs: Shift pressed and let go alone,
/// as to quiet a screen reader, touches nothing (ADR-0009).
pub const RUB_DELAY: f64 = 0.25;
/// `[pa3ds.touch]` as it ships.
pub fn default_touch() -> Touch {
    Touch::default()
}

/// A touch area on the bottom screen, by name, and the pixel a tap on it presses.
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub pane: i64,
    pub name: String,
    pub x: i64,
    pub y: i64,
    /// Where its middle is, up the screen, for telling rows apart.
    pub row: f64,
    pub height: f64,
    /// Where its layout is drawn among the screen's others, higher over lower (`Layouts.layer`).
    pub layer: i64,
    /// The root of its layout.
    pub root: i64,
    /// Where the key that taps slides instead, for a target that is a stroke (`TouchTargets`'s
    /// `gestures`); none for a tap.
    pub path: Vec<(i64, i64)>,
    /// How long the slide along `path` takes.
    pub seconds: f64,
    /// Where a drag carries a press to on it, where not its middle: Amie's Pokémon takes a puff
    /// at its mouth (`amie.MOUTH`).
    pub drop: Option<(i64, i64)>,
    /// The two ends a touch on it rubs between, for a target touched by rubbing, Amie's stylus:
    /// a tap rubs `RUB_SECONDS`, a drag until it is let go; none for a tap.
    pub rub: Option<((i64, i64), (i64, i64))>,
    /// Whether a drag presses it at once and keeps the press on it as it moves: a goal of Super
    /// Training's, charged by a press held on it (ADR-0009 as amended 2026-11-09).
    pub held: bool,
    /// Told once a slide along `path` has begun, for a target that moves what the screen shows:
    /// Amie's beckon and its Pokémon brought between the screens, whose still picture is taken anew
    /// (`amie_stylus.Stylus.look_again`).
    pub after: After,
}

/// `Callable[[], None] | None`, left out of a target's comparison as Python's
/// `field(compare=False)` leaves it.
#[derive(Clone, Default)]
pub struct After(pub Option<Rc<dyn Fn()>>);

impl PartialEq for After {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl std::fmt::Debug for After {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.0.is_some() {
            "After(..)"
        } else {
            "None"
        })
    }
}

impl Target {
    /// `Target(pane, name, x, y, row, height)`: `layer` and `root` 0, as Python defaults them.
    pub fn new(pane: i64, name: &str, x: i64, y: i64, row: f64, height: f64) -> Self {
        Target {
            pane,
            name: name.to_string(),
            x,
            y,
            row,
            height,
            layer: 0,
            root: 0,
            path: Vec::new(),
            seconds: 0.0,
            drop: None,
            rub: None,
            held: false,
            after: After::default(),
        }
    }
}

/// `Callable[[], Sequence[ScreenText]]`: what is on screen, as the screen reader last read it.
pub type OnScreen = Rc<dyn Fn() -> Vec<ScreenText>>;
/// `Callable[[int], str]`: a TextBox's words, shown or not. It reads memory, so it can raise.
pub type TextOf = Rc<dyn Fn(i64) -> Result<String>>;
/// `Callable[[int], str | None]`: the file a picture draws now. It reads memory.
pub type ImageOf = Rc<dyn Fn(i64) -> Result<Option<String>>>;
/// `Callable[[int], LayoutFile | None]`: a layout's root -> the file it was built from.
pub type FileOf = Rc<dyn Fn(i64) -> Option<Rc<LayoutFile>>>;
/// `Callable[[float, float], None]`: sound where a target is.
pub type Tone = Rc<dyn Fn(f64, f64)>;

/// `Callable[[], list[Target]]`: the targets on the bottom screen now, in reading order, as touch
/// mode walks them (`TouchTargets.targets`). It reads memory.
pub type Targets = Rc<dyn Fn() -> Result<Vec<Target>>>;

/// `TouchMode.__init__`'s keyword arguments, each as the Python defaults it: set what differs
/// with `..ModeOptions::default()`.
pub struct ModeOptions {
    pub screen: Option<Rc<dyn TouchScreen>>,
    pub tone: Option<Tone>,
    pub settings: Touch,
    /// None is never.
    pub delayed: Option<Rc<dyn Fn() -> bool>>,
    pub sound: Option<Rc<dyn Fn(bool)>>,
    /// None is full speed, 1.
    pub pace: Option<Rc<dyn Fn() -> f64>>,
}

impl Default for ModeOptions {
    fn default() -> Self {
        ModeOptions {
            screen: None,
            tone: None,
            settings: default_touch(),
            delayed: None,
            sound: None,
            pace: None,
        }
    }
}

/// The walk over the bottom screen's touch targets: the one chosen, a drag picked up and the
/// press held, and a tap, a slide or a drag on the target just named (ADR-0009).
pub struct TouchMode {
    /// The targets on the bottom screen now, in reading order (`TouchTargets.targets`). A test
    /// puts its own in, as Python's puts its own touch mode in the profile.
    pub(crate) targets: Targets,
    log: Log,
    /// The touch screen a tap, a slide and a drag press, or None where pa3ds takes no keys
    /// and touches nothing, as in a replay.
    screen: Option<Rc<dyn TouchScreen>>,
    /// Sound where a target is, across and down the screen as shares of it (`core.tones`).
    tone: Option<Tone>,
    /// Whether the screen drags only as the drag is let go, straight from where it was picked up
    /// to where the keys chose: Berry Picker, where a berry carried over a Pokémon wanting
    /// another sends it off (spike 260). Elsewhere the press goes with the keys, as a Poké Puff
    /// must, held at the Pokémon's mouth until it has been eaten (spike 257).
    delayed: Rc<dyn Fn() -> bool>,
    /// A sound for picking up a drag, with True, and letting go of it, played in place of saying
    /// "holding" and "let go on", which slowed a timed game (`[pa3ds.audio]
    /// feedback_sounds`); None says them.
    sound: Option<Rc<dyn Fn(bool)>>,
    /// How fast the game runs beside full speed, which a drag made on letting go is timed by
    /// (`DROP_SECONDS`): Berry Picker's clock says (`berry_picker.BerryPicker.speed`).
    pace: Rc<dyn Fn() -> f64>,
    /// `[pa3ds.touch]`: whether the keys go round from the last target to the first and the
    /// bottom row to the top, or stop at the ends (`wrap`), and whether J and L go on into the
    /// next row at a row's end, or stay in their row (`next_row`).
    settings: Touch,
    /// The target picked up to drag, while one is.
    picked: Option<Target>,
    /// The target the keys have chosen to drag it to, while one is picked up.
    over: Option<Target>,
    /// Whether the press is down: from the first move away from the target picked up, where the
    /// screen does not drag only on letting go. Whether a rub goes on, and between which ends,
    /// only the screen knows (`TouchScreen.rubbing`).
    pressed: bool,
    chosen: Option<i64>,
    /// When the last tap was, on `time.monotonic`'s clock: Super Training hears a hit on its
    /// bag only after one (`super_training.TAPPED_HIT`). A `Cell`, as a tap takes `&self`.
    tapped: Cell<f64>,
}

impl TouchMode {
    pub fn new(targets: Targets, log: Log, options: ModeOptions) -> Self {
        let ModeOptions {
            screen,
            tone,
            settings,
            delayed,
            sound,
            pace,
        } = options;
        TouchMode {
            targets,
            log,
            screen,
            tone,
            delayed: delayed.unwrap_or_else(|| Rc::new(|| false)),
            sound,
            pace: pace.unwrap_or_else(|| Rc::new(|| 1.0)),
            settings,
            picked: None,
            over: None,
            pressed: false,
            chosen: None,
            tapped: Cell::new(f64::NEG_INFINITY),
        }
    }

    /// The targets on the bottom screen now, in reading order, as the walk has them.
    pub fn targets(&self) -> Result<Vec<Target>> {
        (self.targets)()
    }

    /// Touch mode coming on: the first target, as a menu opening says the item it is on.
    pub fn start(&mut self) -> Result<String> {
        let found = (self.targets)()?;
        self.chosen = found.first().map(|target| target.pane);
        Ok(match found.first() {
            Some(first) => self.said(first),
            None => own(NOTHING, &[]),
        })
    }

    /// Answer a key of touch mode, with the line to say.
    ///
    /// Nothing is tapped that has not just been named: with nothing chosen, or the one chosen
    /// gone from the screen, any key chooses the first target and says it.
    pub fn press(&mut self, action: &str) -> Result<String> {
        let found = (self.targets)()?;
        if found.is_empty() {
            self.chosen = None;
            return Ok(own(NOTHING, &[]));
        }
        let Some(mut at) = found
            .iter()
            .position(|target| Some(target.pane) == self.chosen)
        else {
            self.chosen = Some(found[0].pane);
            self.carry_to(&found[0]);
            return Ok(self.said(&found[0]));
        };
        if action == DRAG {
            let target = found[at].clone();
            if self.picked.is_some() {
                return Ok(String::new());
            }
            if self.screen.is_none() {
                return Ok(own("touch-off", &[]));
            }
            // Nothing is pressed until the keys choose another target: Shift pressed and let go
            // alone, as to pause a screen reader, touches nothing (ADR-0009). But a target
            // touched by rubbing rubs at once, held a moment first where the key is held.
            if let Some(ends) = target.rub {
                let delay = if self.settings.drag == DRAG_TOGGLE {
                    0.0
                } else {
                    RUB_DELAY
                };
                let screen = self.screen.as_ref().expect("checked above");
                if let Some(why) = screen.rub(ends, None, delay)
                    && !why.is_empty()
                {
                    return Ok(why);
                }
                self.pressed = true;
            } else if target.held {
                let screen = self.screen.as_ref().expect("checked above");
                if let Some(why) = screen.hold(target.x, target.y)
                    && !why.is_empty()
                {
                    return Ok(why);
                }
                self.pressed = true;
            }
            self.picked = Some(target.clone());
            self.over = Some(target.clone());
            (self.log)(&format!(
                "touch: pick up {} at ({}, {})",
                target.name, target.x, target.y
            ));
            if let Some(sound) = &self.sound {
                sound(true);
                return Ok(String::new());
            }
            return Ok(own(HOLDING, &[("target", &target.name)]));
        }
        if action == TAP && self.picked.is_some() {
            return Ok(String::new());
        }
        if action == TAP {
            let target = &found[at];
            if target.rub.is_some() {
                return Ok(self.rub(target));
            }
            if !target.path.is_empty() {
                let Some(screen) = &self.screen else {
                    return Ok(own("touch-off", &[]));
                };
                let path: Vec<String> = target
                    .path
                    .iter()
                    .map(|(x, y)| format!("({x}, {y})"))
                    .collect();
                (self.log)(&format!(
                    "touch: slide {} along [{}]",
                    target.name,
                    path.join(", ")
                ));
                let why = screen.slide(&target.path, target.seconds);
                if why.as_deref().is_none_or(str::is_empty)
                    && let Some(after) = &target.after.0
                {
                    after();
                }
                return Ok(why.unwrap_or_default());
            }
            return Ok(self.tap(target));
        }
        let before = found[at].clone();
        let lines = rows(&found);
        let row = lines
            .iter()
            .position(|members| members.contains(&at))
            .expect("every target is in a row");
        if action == ITEM_NEXT || action == ITEM_PREVIOUS {
            // The whole list in reading order, or with `next_row` off the row alone; past its
            // end, round to the other or, with `wrap` off, stay (`[pa3ds.navigation]`).
            let walked: Vec<usize> = if self.settings.next_row {
                (0..found.len()).collect()
            } else {
                lines[row].clone()
            };
            let step = walked
                .iter()
                .position(|&i| i == at)
                .expect("the target chosen is among those walked") as i64
                + if action == ITEM_NEXT { 1 } else { -1 };
            at = walked[stepped(step, walked.len() as i64, self.settings.wrap) as usize];
        } else if action == ROW_NEXT || action == ROW_PREVIOUS {
            // Straight up or down, to the nearest across: a grid's column kept (Tile Puzzle).
            let x = found[at].x;
            let row = stepped(
                row as i64 + if action == ROW_NEXT { 1 } else { -1 },
                lines.len() as i64,
                self.settings.wrap,
            );
            at = *lines[row as usize]
                .iter()
                .min_by_key(|&&i| (found[i].x - x).abs())
                .expect("a row has a target");
        }
        self.chosen = Some(found[at].pane);
        if found[at].pane != before.pane {
            self.carry_to(&found[at]);
        }
        Ok(self.said(&found[at]))
    }

    /// Choose a target a screen's own key has moved, with its name to say, carrying on to
    /// where it is now what goes on with it (`carry`): Pokémon-Amie's stylus
    /// (`amie.StylusKeys`).
    pub fn choose(&mut self, pane: i64) -> Result<String> {
        let Some(target) = (self.targets)()?.into_iter().find(|t| t.pane == pane) else {
            return Ok(String::new());
        };
        self.chosen = Some(pane);
        self.carry_on(&target);
        Ok(self.said(&target))
    }

    /// Carry on to where a target is now what goes on with it, choosing nothing and saying
    /// nothing, for a target the screen moves itself: the stylus kept on its part as the
    /// Pokémon moves, which had chosen the stylus again and taken the player's choice from them.
    /// What goes on is a drag over it, or, while it is chosen, a tap's rub
    /// the screen says lasts; the targets are listed only then, as a moving part's ends are a
    /// read of the picture.
    pub fn carry(&mut self, pane: i64) -> Result<()> {
        let dragged = self.over.as_ref().is_some_and(|over| over.pane == pane);
        let rubbed = self.chosen == Some(pane)
            && self.screen.as_ref().is_some_and(|screen| screen.rubbing());
        if !(dragged || rubbed) {
            return Ok(());
        }
        if let Some(target) = (self.targets)()?.into_iter().find(|t| t.pane == pane) {
            self.carry_on(&target);
        }
        Ok(())
    }

    /// Keep a press held on a target that moves on it, wherever it has gone: a goal drifts
    /// while it is charged, and the shot goes where the press is let go.
    pub fn follow(&mut self) -> Result<()> {
        if let Some(over) = &self.over
            && over.held
        {
            let pane = over.pane;
            self.carry(pane)?;
        }
        Ok(())
    }

    /// Carry a drag picked up (`_carry`), or a tap's rub, on to a target as it is now: the
    /// rub between its ends, where the screen's lasts and they have moved.
    fn carry_on(&mut self, target: &Target) {
        self.carry_to(target);
        if self.picked.is_none()
            && let Some(ends) = target.rub
            && let Some(screen) = &self.screen
            && screen.rub_on(ends)
        {
            (self.log)(&format!(
                "touch: rub on {} between [{:?}, {:?}]",
                target.name, ends.0, ends.1
            ));
        }
    }

    /// Rub a target touched by rubbing for `RUB_SECONDS`, with the line to say: a tap's, which
    /// another before it ends carries on, never lifting.
    pub fn rub(&mut self, target: &Target) -> String {
        let (Some(screen), Some(ends)) = (&self.screen, target.rub) else {
            return own("touch-off", &[]);
        };
        (self.log)(&format!(
            "touch: rub {} between [{:?}, {:?}]",
            target.name, ends.0, ends.1
        ));
        screen.rub(ends, Some(RUB_SECONDS), 0.0).unwrap_or_default()
    }

    /// The name of what is picked up to drag, while something is.
    pub fn picked(&self) -> Option<String> {
        self.picked.as_ref().map(|picked| picked.name.clone())
    }

    /// Whether a target is picked up to drag (`DRAG`), pressed yet or not.
    pub fn holding(&self) -> bool {
        self.picked.is_some()
    }

    /// Let go of the target `DRAG` picked up, where the keys chose, with the line to say; ""
    /// with none. Where the screen drags only on letting go (`delayed`), the drag is made now,
    /// from the target picked up straight to the one chosen, or not at all on the same one.
    pub fn drop(&mut self) -> String {
        let (picked, over, pressed) = (self.picked.take(), self.over.take(), self.pressed);
        self.pressed = false;
        let (Some(picked), Some(over)) = (picked, over) else {
            return String::new();
        };
        if pressed && let Some(screen) = &self.screen {
            screen.release();
            (self.log)(&format!("touch: let go on {}", over.name));
        } else if (self.delayed)() && over.pane != picked.pane {
            return self.give(&picked, &over);
        }
        self.dropped(&over)
    }

    /// Tap a target, with the line to say: the one touch mode has just said, or one a
    /// screen's own key names, Head It's Pokémon's (ADR-0009 as amended 2026-10-31).
    pub fn tap(&self, target: &Target) -> String {
        let Some(screen) = &self.screen else {
            return own("touch-off", &[]);
        };
        (self.log)(&format!(
            "touch: tap {} at ({}, {})",
            target.name, target.x, target.y
        ));
        self.tapped.set(py::time::monotonic());
        screen.tap(target.x, target.y).unwrap_or_default()
    }

    /// When the last tap was, on `time.monotonic`'s clock: Python's `tapped` attribute.
    pub fn tapped(&self) -> f64 {
        self.tapped.get()
    }

    /// Drag from one target straight to another and let go, with the line to say, as a screen
    /// that drags only on letting go has it (`delayed`): Berry Picker's keys, a berry's and then a
    /// Pokémon's, give it this way (ADR-0009 as amended 2026-10-29).
    pub fn give(&self, picked: &Target, over: &Target) -> String {
        let Some(screen) = &self.screen else {
            return own("touch-off", &[]);
        };
        let (x, y) = point(over);
        let path = [(picked.x, picked.y), (picked.x, picked.y), (x, y), (x, y)];
        let listed: Vec<String> = path.iter().map(|(x, y)| format!("({x}, {y})")).collect();
        (self.log)(&format!(
            "touch: drag {} to {} along [{}]",
            picked.name,
            over.name,
            listed.join(", ")
        ));
        if let Some(why) = screen.slide(&path, DROP_SECONDS / (self.pace)())
            && !why.is_empty()
        {
            return why;
        }
        self.dropped(over)
    }

    /// The sound of letting go, or where there is none the words.
    fn dropped(&self, over: &Target) -> String {
        if let Some(sound) = &self.sound {
            sound(false);
            return String::new();
        }
        own(DROPPED, &[("target", &over.name)])
    }

    /// Take what was picked up on to the target the keys have just chosen: pressed on the first
    /// such move, and slid from target to target after; or, where the screen drags only on
    /// letting go (`delayed`), nothing touched until then. A target the keys move, as the
    /// stylus, is carried on to wherever it has gone (`choose`).
    ///
    /// Python's `_carry`: `carry` is its public one's name.
    fn carry_to(&mut self, target: &Target) {
        let (Some(picked), Some(over), Some(screen)) = (&self.picked, &self.over, &self.screen)
        else {
            return;
        };
        let (picked, screen) = (picked.clone(), screen.clone());
        if picked.rub.is_some() {
            // Rubbed between its ends as they are now, wherever they have gone, while it lasts.
            let point = point(target);
            let ends = target.rub.unwrap_or((point, point));
            self.over = Some(target.clone());
            if screen.rub_on(ends) {
                (self.log)(&format!(
                    "touch: rub on {} between [{:?}, {:?}]",
                    target.name, ends.0, ends.1
                ));
            }
            return;
        }
        if target.pane == over.pane && point(target) == point(over) {
            return;
        }
        self.over = Some(target.clone());
        if (self.delayed)() {
            (self.log)(&format!("touch: choose {}", target.name));
            return;
        }
        if !self.pressed {
            if let Some(why) = screen.hold(picked.x, picked.y)
                && !why.is_empty()
            {
                (self.log)(&format!("touch: could not hold {}: {why}", picked.name));
                return;
            }
            self.pressed = true;
            (self.log)(&format!(
                "touch: hold {} at ({}, {})",
                picked.name, picked.x, picked.y
            ));
        }
        let (x, y) = point(target);
        screen.r#move(x, y);
        (self.log)(&format!("touch: carry to {} at ({x}, {y})", target.name));
    }

    /// A target's name, with its tone, as the key chooses it.
    fn said(&self, target: &Target) -> String {
        if let Some(tone) = &self.tone {
            tone(
                target.x as f64 / WIDTH as f64,
                target.y as f64 / HEIGHT as f64,
            );
        }
        target.name.clone()
    }
}

/// Where a drag carries a press to on a target: its `drop`, or its middle.
fn point(target: &Target) -> (i64, i64) {
    target.drop.unwrap_or((target.x, target.y))
}

/// A step to `at` among `count`: round to the other end past either, or with `wrap` off stay
/// at the end.
fn stepped(at: i64, count: i64, wrap: bool) -> i64 {
    if wrap {
        py::modulo(at, count)
    } else {
        at.max(0).min(count - 1)
    }
}

/// Indexes of `found` row by row, top first, each left to right: a target is in a row when its
/// middle is within half its height of the row's first.
pub fn rows(found: &[Target]) -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let order = py::sorted_by_lt((0..found.len()).collect(), |&a, &b| {
        -found[a].row < -found[b].row
    });
    for i in order {
        let mut joined = false;
        for row in &mut rows {
            let first = &found[row[0]];
            if (found[i].row - first.row).abs() < min_of(found[i].height, first.height) / 2.0 {
                row.push(i);
                joined = true;
                break;
            }
        }
        if !joined {
            rows.push(vec![i]);
        }
    }
    rows.into_iter()
        .map(|row| py::sorted_by_lt(row, |&a, &b| found[a].x < found[b].x))
        .collect()
}

/// Python's `min(a, b)` on floats: `a`, unless `b < a`, so a NaN stays where Python puts it.
fn min_of(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}
