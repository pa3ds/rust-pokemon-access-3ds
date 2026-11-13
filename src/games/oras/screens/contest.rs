//! A contest's standings during its appeals, said on the keys that say a battle's sides.
//!
//! The appeals draw the four entrants as portraits along the bottom screen's top (spike 196,
//! movie33): each a pane holding its background, `contest_player_bg_01.bclim`, its appeal so far
//! as a bar, `appeal_bar_filler.bclim`, as wide as the appeal, and the Pokémon's icon,
//! `poke_icon.bclim`. The game keeps them sorted by appeal, the leader on the left, moving the
//! icons from portrait to portrait as the order changes, and draws the player's portrait on a
//! background image of its own. No word names an entrant but on the panel under the move list,
//! "Pokémon: Pikachu, Trainer: Brendan", for the portrait the cursor is on; each name is learned
//! so, by the image of the icon in that portrait, and said as the entry cards wrote it,
//! "Brendan & Pikachu", where they did. The computer's entrants are learned before that, in the
//! first round: its portraits stand in its turn order and hold still until all four have
//! appealed, and each appeal's message names the Pokémon whose turn it is, "Bolt tried to show
//! its appeal with Light Screen!", one of the contests' own entrants (`GameText.contest_rivals`).

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{f32, u32};
use crate::core::memory::Memory;
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::lyt;
use crate::games::common::pictures::PLACES;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::game::StandingsReader;
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::rom::textures;
use crate::py::{self, Result};

/// The entry cards' app, then the appeals' (spike 196). The appeals were taken for the cards'
/// until a new app built where the last ran was noticed (2026-10-09, or_broken_contest).
pub const APPS: [&str; 2] = ["N7contest11OhiromeProcE", "N7contest8MainProcE"];
pub const BACKGROUND: &str = "contest_player_bg_01.bclim";
pub const BAR: &str = "appeal_bar_filler.bclim";
pub const ICON: &str = "poke_icon.bclim";
pub const CURSOR: &str = "cursor.bclim";
pub const ENTRANTS: usize = 4;
/// Where the cursor stands on a portrait from its middle: on an empty pane at its top right
/// (spike 196).
pub const CURSOR_AT: (f64, f64) = (10.0, 20.0);
/// Units within which the cursor stands where `CURSOR_AT` says.
pub const NEAR: f64 = 6.0;
/// Between a trainer and a Pokémon, where no entry card wrote the two together.
pub const JOIN: &str = " & ";

/// A picture's first texture, by its pane's name in its layout's file.
fn texture_of(
    layouts: &Layouts,
    drawn: &IndexMap<String, Vec<String>>,
    pane: i64,
) -> Result<Option<String>> {
    let raw = layouts.name(pane)?;
    let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
    let name = py::decode_ascii(&raw[..end]);
    Ok(drawn.get(&name).and_then(|found| found.first().cloned()))
}

/// Python's `\w`: a letter, a digit or an underscore.
fn word_char(c: char) -> bool {
    py::is_alnum(c) || c == '_'
}

/// The entrants whose Pokémon a message names, in the order it names them: "Bolt tried to
/// show its appeal with Light Screen!" names Camden & Bolt.
///
/// Python searches `(?<!\w)<the name>(?!\w)`; the regex crate has no look-around, so each place
/// the name starts is tried in turn and its neighbours looked at.
pub fn named_in(text: &str, rivals: &[(String, String)]) -> Vec<(String, String)> {
    let chars: Vec<char> = text.chars().collect();
    let mut found: Vec<(usize, (String, String))> = Vec::new();
    for rival in rivals {
        let name: Vec<char> = rival.1.chars().collect();
        let matched = (0..=chars.len()).find(|&start| {
            start + name.len() <= chars.len()
                && chars[start..start + name.len()] == name[..]
                && !(start > 0 && word_char(chars[start - 1]))
                && !chars.get(start + name.len()).is_some_and(|&c| word_char(c))
        });
        if let Some(start) = matched {
            found.push((start, rival.clone()));
        }
    }
    found.sort();
    found.into_iter().map(|(_, rival)| rival).collect()
}

/// Each icon of the first round's portraits, left to right with whether it is the
/// player's, -> the entrant whose appeal came in its turn: the first round appeals left to
/// right and nothing moves until all four have (spike 196, movie33). The player's turn names
/// no entrant of `appealed`, which holds the computer's alone.
pub fn by_turn(
    turns: &[(String, bool)],
    appealed: &[(String, String)],
) -> IndexMap<String, (String, String)> {
    let theirs = turns
        .iter()
        .filter(|(_icon, mine)| !mine)
        .map(|(icon, _mine)| icon.clone());
    theirs.zip(appealed.iter().cloned()).collect()
}

/// The player's place, or each other entrant's in order, by the bars, the player's being
/// the portrait whose background is unlike the others': "Brendan & Pikachu first place".
///
/// Python's `pauses` defaults to False. `name` returns `Result`, as Python's raises where it
/// reads the icon's image.
pub fn standings(
    portraits: &[Portrait],
    name: &dyn Fn(&Portrait) -> Result<Option<String>>,
    player: bool,
    pauses: bool,
) -> Result<Option<String>> {
    let backgrounds: Vec<Option<i64>> = portraits.iter().map(|p| p.background).collect();
    let mine = portraits.iter().position(|p| {
        backgrounds
            .iter()
            .filter(|&&background| background == p.background)
            .count()
            == 1
    });
    // `sorted(portraits, key=lambda p: (-p.appeal, p.x))`, kept with each one's index so that
    // `portrait is mine` is asked of the same one.
    let ranked = py::sorted_by_lt(
        portraits.iter().enumerate().collect::<Vec<_>>(),
        |(_, a), (_, b)| {
            if -a.appeal != -b.appeal {
                return -a.appeal < -b.appeal;
            }
            a.x < b.x
        },
    );
    let places: Vec<String> = PLACES.iter().map(|&(_, place)| own(place, &[])).collect();
    let mut lines: Vec<String> = Vec::new();
    for (at, (index, portrait)) in ranked.into_iter().enumerate() {
        if (Some(index) == mine) != player {
            continue;
        }
        let place = if at < places.len() {
            places[at].to_string()
        } else {
            (at + 1).to_string()
        };
        let named = name(portrait)?;
        lines.push(match named {
            Some(named) if !named.is_empty() => {
                format!("{named}{}{place}", if pauses { ": " } else { " " })
            }
            _ => place,
        });
    }
    let joined = lines.join(", ");
    Ok(if joined.is_empty() {
        None
    } else {
        Some(joined)
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Portrait {
    pub x: f64,
    pub y: f64,
    /// The bar's width: the appeal so far.
    pub appeal: f64,
    /// The picture of the Pokémon's icon.
    pub icon: i64,
    /// The image the background draws: the player's is the one unlike the others'.
    pub background: Option<i64>,
}

/// The four portraits, and where the cursor stands.
type Found = (Vec<Portrait>, Option<(f64, f64)>);

/// Who stands where in a contest's appeals, from its portraits.
pub struct Standings {
    memory: Rc<dyn Memory>,
    layouts: Rc<Layouts>,
    files: Rc<LayoutFiles>,
    /// The game's "Pokémon:" and "Trainer:".
    labels: Vec<String>,
    running: Rc<dyn Fn() -> Option<String>>,
    /// Icon image digest -> the entrant it is, learned from the panel.
    names: IndexMap<String, String>,
    /// Words the contest has shown, to say an entrant as its entry card did.
    ///
    /// Python's is a set of strings, which it iterates in an order that changes every run: kept
    /// here in the order the words were first shown.
    written: IndexSet<String>,
    /// The portrait the cursor is on, by its place across, or None.
    at: Option<f64>,
    /// What the panel named as the cursor came to that portrait.
    arrived_with: Option<String>,
    /// What the panel named at the last poll.
    last: Option<String>,
    /// The computer's entrants, each trainer with their Pokémon (`GameText.contest_rivals`).
    rivals: Vec<(String, String)>,
    /// The first round's portraits' icons, left to right, and whether each is the player's.
    turns: Vec<(String, bool)>,
    /// Whether no bar has filled yet: the first round's appeals are still to come or going.
    first_round: bool,
    /// The computer's entrants the first round's messages have named, in the order named.
    appealed: Vec<(String, String)>,
}

impl Standings {
    /// Python's `rivals` defaults to none.
    pub fn new(
        memory: Rc<dyn Memory>,
        layouts: Rc<Layouts>,
        files: Rc<LayoutFiles>,
        labels: Vec<String>,
        running: Rc<dyn Fn() -> Option<String>>,
        rivals: Vec<(String, String)>,
    ) -> Self {
        Standings {
            memory,
            layouts,
            files,
            labels,
            running,
            names: IndexMap::new(),
            written: IndexSet::new(),
            at: None,
            arrived_with: None,
            last: None,
            rivals,
            turns: Vec::new(),
            first_round: true,
            appealed: Vec::new(),
        }
    }

    /// The standings, with the contest's labels and entrants from the ROM's words, as the
    /// gen 6 profile builds them (`Game.standings`).
    pub fn of(
        memory: Rc<dyn Memory>,
        layouts: Rc<Layouts>,
        files: Rc<LayoutFiles>,
        text: &GameText,
        running: Rc<dyn Fn() -> Option<String>>,
    ) -> Rc<RefCell<dyn StandingsReader>> {
        Rc::new(RefCell::new(Standings::new(
            memory,
            layouts,
            files,
            text.contest_entrant.clone(),
            running,
            text.contest_rivals.clone(),
        )))
    }
}

impl StandingsReader for Standings {
    /// Learn the entrant the panel names, by the icon in the portrait the cursor is on.
    fn poll(&mut self, on_screen: &[ScreenText]) -> Result<()> {
        if !(self.running)().is_some_and(|app| APPS.contains(&app.as_str())) {
            self.names.clear();
            self.written.clear();
            self.turns.clear();
            self.appealed.clear();
            self.first_round = true;
            return Ok(());
        }
        self.written
            .extend(on_screen.iter().map(|piece| piece.text.clone()));
        let named = self.named(on_screen);
        let found = self.portraits()?;
        if let Some((portraits, _at)) = &found
            && self.first_round
        {
            if portraits.iter().any(|p| p.appeal > 0.0) {
                self.first_round = false;
            } else {
                let backgrounds: Vec<Option<i64>> =
                    portraits.iter().map(|p| p.background).collect();
                let mut turns: Vec<(Option<String>, bool)> = Vec::new();
                for p in py::sorted_by_lt(portraits.iter().collect(), |a, b| a.x < b.x) {
                    let mine = backgrounds
                        .iter()
                        .filter(|&&background| background == p.background)
                        .count()
                        == 1;
                    turns.push((self.icon(p)?, mine));
                }
                self.turns = turns
                    .into_iter()
                    .filter_map(|(icon, mine)| icon.map(|icon| (icon, mine)))
                    .collect();
            }
        }
        let mut on: Vec<Portrait> = Vec::new();
        if let Some((portraits, Some((x, y)))) = &found {
            on = portraits
                .iter()
                .filter(|p| {
                    (p.x + CURSOR_AT.0 - x).abs() <= NEAR && (p.y + CURSOR_AT.1 - y).abs() <= NEAR
                })
                .copied()
                .collect();
        }
        let at = if on.len() == 1 { Some(on[0].x) } else { None };
        if at != self.at {
            // The panel names the portrait the cursor left for a second and a half after it
            // moves (movie33): only words it shows once the cursor is here are this one's. Come
            // up from the moves, where the panel is not shown, what it shows is.
            self.arrived_with = if self.at.is_some() {
                named.clone()
            } else {
                None
            };
            self.at = at;
        }
        // Read the same on two polls: caught as the panel is rewritten, the player's portrait
        // was taken for Gianna & Tailster's (movie33).
        let steady = named == self.last;
        self.last = named.clone();
        if at.is_some() && named.is_some() && named != self.arrived_with && steady {
            if let Some(icon) = self.icon(&on[0])? {
                self.names.insert(icon, named.clone().unwrap_or_default());
            }
            self.arrived_with = named;
        }
        Ok(())
    }

    /// Learn the computer's entrants from the first round's messages: "Bolt tried to show
    /// its appeal" is the portrait whose turn it is, and the first round appeals left to right.
    /// Only entrants whose entry cards were shown are looked for; the panel's names win.
    fn heard(&mut self, texts: &[&str]) {
        if !(self.running)().is_some_and(|app| APPS.contains(&app.as_str()))
            || !self.first_round
            || self.turns.len() != ENTRANTS
        {
            return;
        }
        let carded: Vec<(String, String)> = self
            .rivals
            .iter()
            .filter(|(trainer, pokemon)| self.card(trainer, pokemon).is_some())
            .cloned()
            .collect();
        for text in texts {
            for rival in named_in(text, &carded) {
                if !self.appealed.contains(&rival) {
                    self.appealed.push(rival);
                }
            }
        }
        for (icon, (trainer, pokemon)) in by_turn(&self.turns, &self.appealed) {
            let card = self.card(&trainer, &pokemon).unwrap_or_default();
            self.names.entry(icon).or_insert(card);
        }
    }

    /// The player's place, or each other entrant's in order, "Brendan & Pikachu first place";
    /// None outside the appeals or where the portraits are not found.
    ///
    /// Python's `pauses` defaults to False.
    fn said(&self, player: bool, pauses: bool) -> Result<Option<String>> {
        if !(self.running)().is_some_and(|app| APPS.contains(&app.as_str())) {
            return Ok(None);
        }
        let Some((portraits, _at)) = self.portraits()? else {
            return Ok(None);
        };
        let name = |portrait: &Portrait| -> Result<Option<String>> {
            let icon = self.icon(portrait)?.unwrap_or_default();
            Ok(self.names.get(&icon).cloned())
        };
        standings(&portraits, &name, player, pauses)
    }
}

impl Standings {
    /// The digest of the image the portrait's icon draws, or None.
    fn icon(&self, portrait: &Portrait) -> Result<Option<String>> {
        let image = textures::image(&*self.memory, portrait.icon)?;
        Ok(image
            .filter(|image| !image.is_empty())
            .map(|image| textures::digest(&image)))
    }

    /// The entrant the panel names now, as its entry card wrote it, or None.
    fn named(&self, on_screen: &[ScreenText]) -> Option<String> {
        if self.labels.len() < 2 {
            return None;
        }
        let mut values: Vec<String> = Vec::new();
        for label in &self.labels[..2] {
            let labelled = on_screen.iter().find(|p| &p.text == label)?;
            let value = on_screen.iter().find(|p| {
                p.root == labelled.root
                    && p.parent == labelled.parent
                    && p.left > labelled.right
                    && (p.row - labelled.row).abs() < labelled.glyph / 2.0
                    && !self.labels.contains(&p.text)
            })?;
            values.push(value.text.clone());
        }
        let (pokemon, trainer) = (&values[0], &values[1]);
        Some(match self.card(trainer, pokemon) {
            Some(words) if !words.is_empty() => words,
            _ => format!("{trainer}{JOIN}{pokemon}"),
        })
    }

    /// The two as the contest's entry card wrote them, or None where it wrote no such card.
    fn card(&self, trainer: &str, pokemon: &str) -> Option<String> {
        let most = trainer.chars().count() + pokemon.chars().count() + JOIN.chars().count() + 2;
        self.written
            .iter()
            .find(|words| {
                words.starts_with(trainer)
                    && words.ends_with(pokemon)
                    && words.chars().count() <= most
            })
            .cloned()
    }

    /// The four portraits, and where the cursor stands, or None where they are not shown.
    fn portraits(&self) -> Result<Option<Found>> {
        let (layouts, shape) = (&self.layouts, self.layouts.shape);
        for root in layouts.roots() {
            let placement = layouts.placement(root)?;
            if !placement.is_some_and(|placement| placement.shown) {
                continue;
            }
            let Some(file) = self.files.matching(&layouts.pane_names(root)) else {
                continue;
            };
            let texture = |pane: i64| texture_of(layouts, &file.textures, pane);
            let mut portraits: Vec<Portrait> = Vec::new();
            for panel in layouts.children(root, root) {
                let Some((bar, icon, background)) = self.parts(root, panel, &texture)? else {
                    continue;
                };
                let (head, place) = (layouts.pane(bar)?, layouts.pane(panel)?);
                portraits.push(Portrait {
                    x: f32(&place, shape.x)?,
                    y: f32(&place, shape.y)?,
                    appeal: f32(&head, shape.width)? * f32(&head, shape.scale_x)?,
                    icon,
                    background: textures::drawn_at(&*self.memory, background)?,
                });
            }
            if portraits.len() != ENTRANTS {
                continue;
            }
            let mut cursor = None;
            for (pane, kind) in layouts.tree(root) {
                if kind == lyt::PICTURE_CLASS && texture(pane)?.as_deref() == Some(CURSOR) {
                    cursor = Some(pane);
                    break;
                }
            }
            let mut at = None;
            if let Some(cursor) = cursor {
                let head = layouts.pane(cursor)?;
                at = Some((f32(&head, shape.x)?, f32(&head, shape.y)?));
            }
            return Ok(Some((portraits, at)));
        }
        Ok(None)
    }

    /// A portrait's bar, icon and background, or None for a pane that is no portrait.
    fn parts(
        &self,
        root: i64,
        panel: i64,
        texture: &dyn Fn(i64) -> Result<Option<String>>,
    ) -> Result<Option<(i64, i64, i64)>> {
        let (mut bar, mut icon, mut background) = (None, None, None);
        let mut stack: Vec<i64> = self.layouts.children(root, panel);
        while let Some(pane) = stack.pop() {
            let kind = texture(pane)?;
            match kind.as_deref() {
                Some(BAR) => bar = Some(pane),
                Some(ICON) => icon = Some(pane),
                Some(BACKGROUND) => background = Some(pane),
                _ => {}
            }
            stack.extend(self.layouts.children(root, pane));
        }
        let (Some(bar), Some(icon), Some(background)) = (bar, icon, background) else {
            return Ok(None);
        };
        if u32(&self.layouts.pane(bar)?, lyt::PARENT)? != panel {
            return Ok(None);
        }
        Ok(Some((bar, icon, background)))
    }
}
