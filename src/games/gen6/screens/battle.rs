//! The battle's HP: who is on each side, at what level, and how hurt.
//!
//! The user's rule (2026-09-16): say the player's side whole whenever its HP changes — "Torchic ♀
//! L5 20/20" — and the enemy the same but as a percentage, "Wurmple ♂ L2 80%", because ORAS draws
//! the enemy a bar and never a number, and anything the game does not show is an Extra.
//!
//! Four sources, each measured rather than assumed:
//!
//! - **Each Pokémon on screen is its gauge's** (spike 66): the battle's `btl::BtlvGaugeSelf` for
//!   each of the player's, `btl::BtlvGaugeBase` for each of the rest, which holds a battle record
//!   whose `pml::pokepara::PokemonParam` is the Pokémon, nickname and gender (`pokemon`), and the
//!   panes its reading is drawn in: the **group** that holds them, its level TextBox and its bar.
//!   The gauge's class is in the battle's DLL, which `Discovery.learn` reads as the battle starts,
//!   so the gauges are found by class as the battle builds them, seconds before the box shows
//!   (spike 138). They were found as what held a level TextBox the reader had seen, by a sweep
//!   asked for once the box was up, and a third of the battles said their first reading 1.5 s late.
//! - **A side is read from its gauge, not from the box's panels** (spike 150, movie15, 2026-10-01).
//!   A double battle draws both of the player's Pokémon in one panel and both of the enemy's in
//!   another, each in a group of its own; read a panel at a time, Blaze was said with Beldum's HP,
//!   Beldum never, and the enemies only once one bar had moved. Whose side a gauge is on is its
//!   class: its `side` word is the battle position, 0 and 2 the player's, 1 and 3 the enemy's, and
//!   only 0 was taken as the player's.
//! - **The name as said, sign and all,** comes from a `gfl::str::StrBuf` (spike 57): during a
//!   battle the heap's strings ending in ♂ or ♀ are the battle's name plates, one for each Pokémon
//!   in it. The plate whose name is the gauge's Pokémon's nickname is that Pokémon's. Taking them
//!   in address order instead, the player's first, held only while each side had one Pokémon: with
//!   two in the party the wild Poochyena was said under the second one's name (battles2.ctm, the
//!   user, 2026-09-17). A Pokémon with no gender has no plate, and is said by the nickname in its
//!   own record.
//! - **Whether a battle is up at all** is whether an HP panel on screen has a gauge leading to a
//!   Pokémon. Gendered plates alone said so until 2026-09-18, which left a battle between
//!   genderless Pokémon silent. They still say so before the box arrives, which is a poll or two
//!   earlier.
//! - **The level, and the player's current and maximum HP**, are the HP box's own TextBoxes (spike
//!   54), the level the gauge's and the HP the one in its group.
//! - **The enemy's share** is the width of the gauge's bar over the width of a full one (spike 54).
//!   The bar is a picture, not text, and the pane's own width is the bar: the enemy's read 48, then
//!   30, then 27, then 0 as a Wurmple fainted.
//!
//! **Nothing is said until the gauge has counted to its target and the reading has held still for a
//! poll.** That is not caution, it is correctness: the bar drains as an animation and the HP box
//! counts with it, so a poll lands on whatever frame it happens to catch. The 48, 30, 27, 0 above
//! is one Ember, and speaking it would say 62% and then 0% a moment later (the user, 2026-09-16).
//! Holding for a poll was not enough on its own: a Potion's count from 3 to 19 was said at 9
//! (battles2.ctm). The gauge keeps the HP it shows and the HP it is counting towards side by side
//! (spike 67).
//!
//! **How many Pokémon a trainer has** is the tray of balls the battle draws as it opens (spike 285,
//! x_battle-count.ctm): one top-screen layout, a panel a side, each a `ball_win.bclim` picture and
//! six slots, a ball for each Pokémon in the party and `ball_null.bclim` for each slot without one.
//! Youngster Jayden's held two balls and four empty slots beside the player's six. The opponent's
//! panel is the one further right.
//!
//! Bound by:
//! - ADR-0015: HP comes from the battle's own records, never from the bar, and the enemy's is a
//!   percentage.
//! - ADR-0016: the HP box is the battle reader's alone: screen text leaves its pieces out while a
//!   battle is up.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::{IndexMap, IndexSet};
use regex::Regex;

use crate::core::binary::{f32, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::symbol;
use crate::core::words::own;
use crate::games::common::battle::{Battler, GAUGE_RECORD, MAX_ASKS, REASK, RECORD_PARAM, SIGNS};
use crate::games::common::layout::Layouts;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::ImageOf;
use crate::games::common::{lyt, pokemon, strbuf};
use crate::py::{self, PySet, Result};

/// Halfwords. A nickname is at most 12; this only rejects garbage lengths.
pub const MAX_NAME: i64 = 64;

/// The width of a full HP bar, measured on both sides in two movies (spike 54). Used as a floor:
/// the widest the bar has been seen is preferred, so a battle joined part way still reads
/// sensibly.
pub const BAR_FULL: f64 = 48.0;
/// How tall the bar is, which is what tells it from the other pictures in its panel: the gauge
/// frame is 32 and the panel's other black plate is 2.
pub const BAR_HEIGHT: (f64, f64) = (3.0, 6.0);
pub const BAR_MAX_WIDTH: f64 = 64.0;

/// The player's gauge and every other one (spike 66; spike 150, a double battle's two of each).
pub const GAUGE_CLASSES: [&str; 2] = ["N3btl13BtlvGaugeSelfE", "N3btl13BtlvGaugeBaseE"];
/// u32 the battle position: 0 and 2 the player's, 1 and 3 the enemy's in a double (spike 150).
pub const GAUGE_SIDE: i64 = 0x14;
/// -> the pane its Pokémon's reading hangs from: level, HP, bar, status icon (spike 150).
pub const GAUGE_GROUP: i64 = 0x18;
/// -> the gauge's level TextBox, the one its panel shows.
pub const GAUGE_LEVEL: i64 = 0x2C;
/// -> the bar picture whose width is the HP (spike 150).
pub const GAUGE_BAR: i64 = 0x30;
/// u32 the HP the box shows, counted a point a frame towards `GAUGE_TARGET_HP` (spike 67).
pub const GAUGE_SHOWN_HP: i64 = 0x3C;
pub const GAUGE_TARGET_HP: i64 = 0x40;
pub const GAUGE_HEAD: i64 = 0x44;

/// Screen polls, 2 s, an enemy's first reading of a battle waits for the player's side to be said.
/// Its panel comes on a poll or two ahead of the player's, and the enemy was said first in one
/// battle and second in the next, which heard as a race (Orion, 2026-09-29). Waited out, since a
/// battle whose player's side is never read must still say the enemy.
pub const OURS_FIRST: i64 = 10;

pub static LEVEL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{1,3})$").expect("a valid pattern"));
/// Python's `\s` also takes U+001C to U+001F, which the `regex` crate's does not.
pub static HP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d{1,4})[\s\x1c-\x1f]*/[\s\x1c-\x1f]*(\d{1,4})$").expect("a valid pattern")
});

/// The picture each side's tray of balls is, as a trainer battle opens (spike 285).
pub const TRAY: &str = "ball_win.bclim";
pub const BALL: &str = "ball_";
/// A slot of the tray with no Pokémon in it.
pub const EMPTY: &str = "ball_null.bclim";
/// Screen polls, 2 s, the tray is looked for after the battle's first page: a wild battle has
/// none, and the layouts are not walked for it the rest of the battle.
pub const TRAY_POLLS: i64 = 10;

/// `text.endswith(SIGNS)`.
fn ends_with_sign(text: &str) -> bool {
    SIGNS.iter().any(|sign| text.ends_with(sign))
}

/// What a gauge shows: the Pokémon, whether its HP has finished counting, whose side it is on,
/// and the panes it is drawn in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    pub pokemon: pokemon::Pokemon,
    pub counting: bool,
    pub player: bool,
    pub position: i64,
    pub group: i64,
    pub level: i64,
    pub bar: i64,
}

/// The battle's HP gauges, found by class.
///
/// Shared: the profile tracks them while the battle reader reads them, so the gauges are in a
/// `RefCell` and every method takes `&self`.
pub struct Gauges {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    /// Gauge -> the vtable it had when a sweep found it, and whether it is the player's. Its
    /// class is the battle's DLL's, which the profile's class names, read from the executable
    /// alone, cannot name (spike 138), so the sweep that found it says which it is.
    gauges: RefCell<IndexMap<i64, (i64, bool)>>,
}

impl Gauges {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        Gauges {
            memory,
            names,
            gauges: RefCell::new(IndexMap::new()),
        }
    }

    /// Take these gauges, the player's (`btl::BtlvGaugeSelf`) and the rest, found by class: a
    /// whole sweep's in place of those kept, or, `adding` a sweep of new blocks', beside them.
    pub fn track(
        &self,
        ours: impl IntoIterator<Item = i64>,
        theirs: impl IntoIterator<Item = i64>,
        adding: bool,
    ) -> Result<()> {
        let mut found: IndexMap<i64, (i64, bool)> = IndexMap::new();
        let both = ours
            .into_iter()
            .map(|gauge| (true, gauge))
            .chain(theirs.into_iter().map(|gauge| (false, gauge)));
        for (player, gauge) in both {
            let vtable = u32(&self.memory.read(gauge, 4)?, 0)?;
            found.insert(gauge, (vtable, player));
        }
        let mut gauges = self.gauges.borrow_mut();
        if adding {
            gauges.extend(found);
        } else {
            *gauges = found;
        }
        Ok(())
    }

    /// What each gauge whose level TextBox is one of these shows. A sweep can find a word
    /// equal to a gauge's vtable where no gauge is, so only a gauge whose level is on screen has
    /// its record read.
    pub fn shown(&self, levels: &HashSet<i64>) -> Result<Vec<Shown>> {
        let mut out: Vec<Shown> = Vec::new();
        let kept: Vec<(i64, (i64, bool))> = self
            .gauges
            .borrow()
            .iter()
            .map(|(&gauge, &held)| (gauge, held))
            .collect();
        for (gauge, (vtable, player)) in kept {
            let head = self.memory.read(gauge, GAUGE_HEAD)?;
            if u32(&head, 0)? != vtable {
                // Freed, and its memory another object's: a new battle builds new ones.
                self.gauges.borrow_mut().shift_remove(&gauge);
                continue;
            }
            if !levels.contains(&u32(&head, GAUGE_LEVEL)?) {
                continue;
            }
            let param = u32(
                &self
                    .memory
                    .read(u32(&head, GAUGE_RECORD)? + RECORD_PARAM, 4)?,
                0,
            )?;
            let found = pokemon::read(&*self.memory, &self.names, param, pokemon::SPECIES_MAX)?;
            if let Some(found) = found {
                out.push(Shown {
                    pokemon: found,
                    counting: u32(&head, GAUGE_SHOWN_HP)? != u32(&head, GAUGE_TARGET_HP)?,
                    player,
                    position: u32(&head, GAUGE_SIDE)?,
                    group: u32(&head, GAUGE_GROUP)?,
                    level: u32(&head, GAUGE_LEVEL)?,
                    bar: u32(&head, GAUGE_BAR)?,
                });
            }
        }
        Ok(out)
    }
}

/// The name plates of everyone in the battle: nickname and gender sign as one string.
///
/// Shared like [`Gauges`]: tracked by the profile, read by the battle reader.
pub struct Battlers {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    buffers: RefCell<Vec<i64>>,
    /// The buffers whose text ended in a gender sign when last read: the ones worth a poll.
    named: RefCell<Vec<i64>>,
}

impl Battlers {
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>) -> Self {
        Battlers {
            memory,
            names,
            buffers: RefCell::new(Vec::new()),
            named: RefCell::new(Vec::new()),
        }
    }

    /// Consider these `gfl::str::StrBuf` addresses from now on, and check every one for a name.
    ///
    /// Kept once seen, like every other reader's objects: a sweep taken as a battle starts can
    /// miss one that is there a moment later. All are read here, a sweep at a time, and only the
    /// ones ending in a gender sign are read every poll: reading every buffer every poll was 50
    /// reads a poll through menus.ctm, which has no battle at all (spike 62). `adding` the few a
    /// sweep of the blocks just allocated found, only those are read, known or not: a buffer is
    /// taken empty and named a moment later, and the block is swept again. Which of the rest are
    /// still buffers waits for a sweep of the whole heap (spike 110).
    pub fn track(&self, buffers: impl IntoIterator<Item = i64>, adding: bool) -> Result<()> {
        if adding {
            for buffer in buffers {
                let Some(text) = self.text(buffer)? else {
                    continue;
                };
                if !self.buffers.borrow().contains(&buffer) {
                    self.buffers.borrow_mut().push(buffer);
                }
                if ends_with_sign(&text) && !self.named.borrow().contains(&buffer) {
                    self.named.borrow_mut().push(buffer);
                }
            }
            return Ok(());
        }
        for buffer in buffers {
            if !self.buffers.borrow().contains(&buffer) {
                self.buffers.borrow_mut().push(buffer);
            }
        }
        let mut live: Vec<i64> = Vec::new();
        let mut named: Vec<i64> = Vec::new();
        let held = self.buffers.borrow().clone();
        for buffer in held {
            let Some(text) = self.text(buffer)? else {
                continue;
            };
            // Still a StrBuf, so worth reading again. The heap holds a good many, most of them the
            // dialogue system's, and this is what stops the list growing for ever.
            live.push(buffer);
            if ends_with_sign(&text) {
                named.push(buffer);
            }
        }
        *self.buffers.borrow_mut() = live;
        *self.named.borrow_mut() = named;
        Ok(())
    }

    /// Every buffer holding a name that still ends in a gender sign, lowest address first.
    ///
    /// Empty out of a battle, which is how a caller tells there is no battle to read. A buffer
    /// that gains a name is found at the next `track`, a sweep later, before its HP box is up.
    pub fn read(&self) -> Result<Vec<String>> {
        let mut found: Vec<(i64, String)> = Vec::new();
        let named = self.named.borrow().clone();
        for buffer in named {
            let text = self.text(buffer)?;
            if let Some(text) = text
                && ends_with_sign(&text)
            {
                found.push((buffer, text));
            }
        }
        *self.named.borrow_mut() = found.iter().map(|(buffer, _text)| *buffer).collect();
        found.sort();
        Ok(found.into_iter().map(|(_address, text)| text).collect())
    }

    fn text(&self, buffer: i64) -> Result<Option<String>> {
        if self.names.of_object(buffer)?.as_deref() != Some(strbuf::CLASS) {
            return Ok(None);
        }
        let raw = strbuf::raw(&*self.memory, buffer, MAX_NAME)?;
        if raw.is_empty() {
            return Ok(None);
        }
        let text = py::decode_utf16le(&raw);
        // The Chinese fan Omega Ruby writes a plate's sign as the font's private-use glyph, where
        // the official games write the real one, and no plate of its was found (Orion, 2026-10-18).
        Ok(Some(
            text.split('\0')
                .next()
                .unwrap_or_default()
                .chars()
                .map(symbol)
                .collect(),
        ))
    }
}

/// Says each side of a battle when what it shows has changed and settled.
pub struct BattleReader {
    layouts: Rc<Layouts>,
    battlers: Rc<Battlers>,
    gauges: Rc<Gauges>,
    /// Level TextBoxes of the HP box at the last poll whose gauge is not known.
    wanted: PySet,
    /// Level TextBox a sweep has been asked for, this battle -> the asks since.
    asked: IndexMap<i64, i64>,
    /// Bar pane -> the widest it has been, taken for a full bar.
    widest: IndexMap<i64, f64>,
    /// Group -> what it read at the last poll, before it has held still.
    pending: IndexMap<i64, String>,
    /// Group -> what was last announced for it.
    said: IndexMap<i64, String>,
    /// Whether the player's side has been said this battle, before which an enemy waits.
    ours_said: bool,
    /// Screen polls an enemy's reading has waited for the player's side (`OURS_FIRST`).
    enemy_waited: i64,
    in_battle: bool,
    /// Roots of the layouts found to be an HP box.
    boxes: HashSet<i64>,
    /// The pane names of those layouts, which name the layout file: a box built again elsewhere
    /// for the next battle is known by them from its first piece on screen.
    box_layouts: HashSet<Vec<String>>,
}

impl BattleReader {
    pub fn new(layouts: Rc<Layouts>, battlers: Rc<Battlers>, gauges: Rc<Gauges>) -> Self {
        BattleReader {
            layouts,
            battlers,
            gauges,
            wanted: PySet::new(),
            asked: IndexMap::new(),
            widest: IndexMap::new(),
            pending: IndexMap::new(),
            said: IndexMap::new(),
            ours_said: false,
            enemy_waited: 0,
            in_battle: false,
            boxes: HashSet::new(),
            box_layouts: HashSet::new(),
        }
    }

    /// Is this piece part of an HP box, and so this reader's to say rather than screen text's?
    ///
    /// The box's pieces said as screen text came out as "5 18/ 20 3" as it slid in, on top of
    /// this reader's "Torchic ♀ L5 18/20" (the user, 2026-09-16). Only during a battle: out of
    /// one, a layout built where a box used to be is somebody else's.
    pub fn owns(&self, piece: &ScreenText) -> Result<bool> {
        if !self.in_battle {
            return Ok(false);
        }
        let names = self.layouts.pane_names(piece.root);
        if self.boxes.contains(&piece.root)
            || (!names.is_empty() && self.box_layouts.contains(&names))
        {
            return Ok(true);
        }
        // Before the box is known, the enemy's panel can come on screen a poll ahead of the
        // player's, which is what makes it known: battles.ctm said "3" as the first battle opened.
        // A level alone on the top screen beside a bar is the enemy's. The party screen and the
        // summary, which also show a level beside a bar, are on the bottom screen.
        if !piece.top || !LEVEL.is_match(py::strip(&piece.text)) {
            return Ok(false);
        }
        Ok(self.bar(piece.root, piece.panel)?.is_some())
    }

    /// A battle has started: say both sides once they settle, whatever the last battle said,
    /// and ask afresh for any gauge not found. Kept from one battle to the next, a second battle
    /// said only the side whose HP had changed, and the asks counted in the last one held the new
    /// one's sweep back 2.3 s (spike 138).
    pub fn begin(&mut self) {
        self.pending.clear();
        self.said.clear();
        self.asked.clear();
        self.ours_said = false;
        self.enemy_waited = 0;
    }

    /// Whether to sweep now rather than on schedule: a panel's gauge is not known, first when
    /// seen and again every `REASK` polls, `MAX_ASKS` times. The gauges are found by class as the
    /// battle builds them (spike 138), so this is the net for one missed. A sweep taken as the box
    /// came on screen can be early for its gauge, and asked for only once, the wild Zigzagoon's
    /// first reading came only after its HP had fallen (movie2, spike 99).
    pub fn sweep_wanted(&mut self) -> bool {
        let mut due = false;
        for r#box in self.wanted.iter() {
            let asks = self.asked.get(&r#box).copied().unwrap_or(0);
            if asks < MAX_ASKS * (REASK + 1) {
                due = due || asks % (REASK + 1) == 0;
                self.asked.insert(r#box, asks + 1);
            }
        }
        due
    }

    /// The sides whose reading has changed, finished counting and held still for a poll.
    pub fn poll(&mut self, on_screen: &[ScreenText]) -> Result<Vec<Battler>> {
        let plates = self.battlers.read()?;
        let pieces: IndexMap<i64, &ScreenText> =
            on_screen.iter().map(|piece| (piece.r#box, piece)).collect();
        let levels: HashSet<i64> = pieces.keys().copied().collect();
        let shown = self.gauges.shown(&levels)?;
        for gauge in &shown {
            // Every layout a gauge's level is in is the HP box's, so its words are this reader's:
            // a double battle's second level of each side was said as screen text, "13, 9".
            let root = pieces[&gauge.level].root;
            self.boxes.insert(root);
            let names = self.layouts.pane_names(root);
            if !names.is_empty() {
                self.box_layouts.insert(names);
            }
        }
        let held: HashSet<i64> = shown.iter().map(|gauge| gauge.level).collect();
        let mut wanted: Vec<i64> = Vec::new();
        for piece in on_screen {
            if !held.contains(&piece.r#box) && self.unread_level(piece, on_screen)? {
                wanted.push(piece.r#box);
            }
        }
        self.wanted = PySet::of(wanted);
        let mut out: Vec<Battler> = Vec::new();
        let mut enemy_held = false;
        // The player's side first, then the rest in battle position order.
        let mut ordered: Vec<&Shown> = shown.iter().collect();
        ordered.sort_by_key(|gauge| (!gauge.player, gauge.position));
        for gauge in ordered {
            let level = py::strip(&pieces[&gauge.level].text);
            if !LEVEL.is_match(level) {
                continue;
            }
            let group: Vec<&ScreenText> = on_screen
                .iter()
                .filter(|piece| piece.parent == gauge.group)
                .collect();
            // A status condition is an icon in the group, named by its image: "PARALYSIS" after
            // the HP, as the box draws it beside the Pokémon (spike 147).
            let marks: String = group
                .iter()
                .filter(|piece| piece.picture)
                .map(|piece| format!(" {}", piece.text))
                .collect();
            let detail = if gauge.player {
                // As the player's panel slides in or levels up its level is up and its HP not,
                // and it is said once both are (spike 144, movie4).
                let hp = group.iter().find_map(|piece| {
                    HP.captures(py::strip(&piece.text))
                        .map(|hp| (hp[1].to_string(), hp[2].to_string()))
                });
                hp.map(|(current, maximum)| format!("{current}/{maximum}{marks}"))
            } else {
                self.share(gauge.bar)?
                    .map(|share| own("battle-share", &[("percent", &share)]) + &marks)
            };
            let Some(detail) = detail else {
                continue;
            };
            let Some(name) = name(&plates, &gauge.pokemon) else {
                continue;
            };
            if gauge.counting {
                self.pending.shift_remove(&gauge.group);
                continue;
            }
            let text = own(
                "battle-battler",
                &[("name", &name), ("level", &level), ("hp", &detail)],
            );
            if self.pending.get(&gauge.group) != Some(&text) {
                // Held for one poll before it counts: the bar drains as an animation and the HP
                // box counts down with it, so a single poll catches a frame rather than a value.
                self.pending.insert(gauge.group, text);
                continue;
            }
            if self.said.get(&gauge.group) == Some(&text) {
                continue;
            }
            if !gauge.player && !self.ours_said && self.enemy_waited < OURS_FIRST {
                enemy_held = true;
                continue;
            }
            self.ours_said = self.ours_said || gauge.player;
            self.said.insert(gauge.group, text.clone());
            out.push(Battler {
                panel: gauge.group,
                text,
                player: gauge.player,
            });
        }
        if enemy_held {
            self.enemy_waited += 1;
        }
        self.in_battle = !shown.is_empty() || !plates.is_empty();
        if !self.in_battle {
            // Seldom: the name plates keep their text after a battle, which is why the battle's
            // own start says when a new one begins.
            self.begin();
        }
        Ok(out)
    }

    /// Is this a level of the HP box that no gauge known holds? One in a box already known, or
    /// beside the player's HP in its group, or the enemy's alone on the top screen beside a bar,
    /// as the box comes on before a sweep has found its gauges.
    fn unread_level(&self, piece: &ScreenText, on_screen: &[ScreenText]) -> Result<bool> {
        if !LEVEL.is_match(py::strip(&piece.text)) {
            return Ok(false);
        }
        if self.boxes.contains(&piece.root)
            || on_screen
                .iter()
                .any(|other| other.parent == piece.parent && HP.is_match(py::strip(&other.text)))
        {
            return Ok(true);
        }
        if !piece.top {
            return Ok(false);
        }
        Ok(self.bar(piece.root, piece.panel)?.is_some())
    }

    /// The enemy's HP as a percentage of a full bar, or None when the bar says nothing.
    fn share(&mut self, bar: i64) -> Result<Option<i64>> {
        let width = f32(&self.layouts.pane(bar)?, self.layouts.shape.width)?;
        if width < 0.0 {
            return Ok(None);
        }
        let widest = self.widest.get(&bar).copied().unwrap_or(BAR_FULL);
        // Python's max(), which keeps the first unless the second is greater.
        let full = if width > widest { width } else { widest };
        self.widest.insert(bar, full);
        if full > 0.0 {
            Ok(Some(py::round(width / full * 100.0)?))
        } else {
            Ok(None)
        }
    }

    /// The bar pane in this panel: the one picture as tall as a bar and no wider than one.
    fn bar(&self, root: i64, panel: i64) -> Result<Option<i64>> {
        let mut found: Vec<i64> = Vec::new();
        for (pane, class) in self.layouts.tree(root) {
            if class != lyt::PICTURE_CLASS {
                continue;
            }
            let placement = self.layouts.placement(pane)?;
            let Some(placement) = placement else {
                continue;
            };
            if !placement.shown || placement.panel != panel {
                continue;
            }
            let head = self.layouts.pane(pane)?;
            let shape = self.layouts.shape;
            let (height, width) = (f32(&head, shape.height)?, f32(&head, shape.width)?);
            if (BAR_HEIGHT.0..=BAR_HEIGHT.1).contains(&height)
                && (0.0..=BAR_MAX_WIDTH).contains(&width)
            {
                found.push(pane);
            }
        }
        Ok(if found.len() == 1 {
            Some(found[0])
        } else {
            None
        })
    }
}

/// The name plate of the Pokémon shown: its nickname with its sign, as the game wrote it.
///
/// Two plates can share a nickname, a wild Poochyena against the player's, and then the gender
/// tells them apart. A Pokémon with no gender has no plate among those with a sign, and is said by
/// its nickname alone. None while a gendered Pokémon's plate is not found yet, rather than saying
/// it without its sign now and with it a sweep later.
fn name(plates: &[String], shown: &pokemon::Pokemon) -> Option<String> {
    // Two equal plates are one, as in the Python set; only a lone match is ever taken, so the
    // order the set is iterated in cannot show.
    let mut matching: IndexSet<&String> = plates
        .iter()
        .filter(|plate| {
            let stem = plate
                .char_indices()
                .next_back()
                .map_or("", |(last, _)| &plate[..last]);
            stem == shown.nickname
        })
        .collect();
    if shown.gender != pokemon::MALE && shown.gender != pokemon::FEMALE {
        return if shown.nickname.is_empty() {
            None
        } else {
            Some(shown.nickname.clone())
        };
    }
    if matching.len() > 1 {
        let sign = SIGNS[shown.gender as usize];
        matching = matching
            .into_iter()
            .filter(|plate| plate.ends_with(sign))
            .collect();
    }
    if matching.len() == 1 {
        matching.into_iter().next().cloned()
    } else {
        None
    }
}

/// How many balls the opponent's tray shows, or None while two trays are not on the top
/// screen: the opponent's is the one further right.
pub fn opponents_party(layouts: &Layouts, image_of: &ImageOf) -> Result<Option<i64>> {
    for root in layouts.roots() {
        let Some(placement) = layouts.placement(root)? else {
            continue;
        };
        if !placement.shown || !placement.top {
            continue;
        }
        let mut trays = PySet::new();
        let mut balls: IndexMap<i64, i64> = IndexMap::new();
        for (pane, kind) in layouts.tree(root) {
            let r#where = layouts.placement(pane)?;
            let Some(r#where) =
                r#where.filter(|r#where| kind == lyt::PICTURE_CLASS && r#where.shown)
            else {
                continue;
            };
            let image = image_of(pane)?.unwrap_or_default();
            if image == TRAY {
                trays.add(r#where.panel);
            } else if image.starts_with(BALL) && image != EMPTY {
                *balls.entry(r#where.panel).or_insert(0) += 1;
            }
        }
        if trays.len() == 2 {
            // `max(trays, key=...)`: the first of equal bests, in the set's order.
            let mut right: Option<(i64, f64)> = None;
            for panel in trays.iter() {
                let x = f32(&layouts.pane(panel)?, layouts.shape.x)?;
                if right.is_none_or(|(_, best)| x > best) {
                    right = Some((panel, x));
                }
            }
            return Ok(right.and_then(|(panel, _)| balls.get(&panel).copied()));
        }
    }
    Ok(None)
}

/// How many Pokémon a trainer has, told once a battle after the battle's first page, which in
/// a trainer battle is "You are challenged by ..." (Khai, 2026-11-05).
pub struct OpponentsParty {
    layouts: Rc<Layouts>,
    image_of: ImageOf,
    /// Screen polls left to look for the tray, `TRAY_POLLS` from the battle's first page.
    looks: i64,
    due: bool,
    last: Option<i64>,
}

impl OpponentsParty {
    pub fn new(layouts: Rc<Layouts>, image_of: ImageOf) -> Self {
        OpponentsParty {
            layouts,
            image_of,
            looks: 0,
            due: false,
            last: None,
        }
    }

    /// A battle has started.
    pub fn begin(&mut self) {
        (self.due, self.looks, self.last) = (true, 0, None);
    }

    /// A page was said: in a battle not yet told, the tray is looked for from now.
    pub fn paged(&mut self) {
        if self.due && self.looks == 0 {
            self.looks = TRAY_POLLS;
        }
    }

    /// The opponent's count once it has held for a poll, once a battle; else None. Held, as
    /// the balls come into the tray one after another.
    pub fn poll(&mut self) -> Result<Option<i64>> {
        if !self.due || self.looks == 0 {
            return Ok(None);
        }
        self.looks -= 1;
        let count = opponents_party(&self.layouts, &self.image_of)?;
        if count.is_none() || count != self.last {
            self.last = count;
            self.due = self.looks > 0;
            return Ok(None);
        }
        self.due = false;
        Ok(count)
    }
}
