//! A contest's standings during its appeals, said on the keys that say a battle's sides.
//!
//! The appeals draw the four entrants as portraits along the bottom screen's top (spike 196,
//! movie33): each a pane holding its background, `contest_player_bg_01.bclim`, its appeal so far
//! as a bar, `appeal_bar_filler.bclim`, and the Pokémon's icon, `poke_icon.bclim`, sorted by
//! appeal, the leader on the left. The places were once read from the bars and the entrants told
//! apart by their icons' images; the game keeps both itself (spike B on or_toughness_contest):
//!
//! ```text
//! contest::MainProc   +0x30  -> contest::Main
//! contest::Main       +0x24  -> the appeals, of no class
//! the appeals          +0x1  u8 the rounds done
//!                     +0xF8  a 0x10-byte block an entrant, the player's first, then the
//!                            computer's from the last entry number to the first:
//!                              +0x4 u16 its appeal so far, written as a round ends
//!                              +0xD u8 its place, 1 to 4: before the appeals the first round's
//!                                   turn order, after each round the next's, at the end the
//!                                   results'
//! contest::Main       +0x30  -> contest::AppealUi
//! contest::AppealUi  +0x24C  u8 the entrant the cursor has picked, its portrait 4 to 7 left to
//!                            right, which stand in place order as a move is chosen
//! contest::Main      +0x260  -> a gfl::str::StrBuf, the Pokémon of the entrant picked
//! ```
//!
//! No word names an entrant but the panel under the move list, "Pokémon: Pikachu, Trainer:
//! Brendan", a second and a half after the cursor comes onto a portrait. The Pokémon the game
//! picked names it at once: its trainer from the contests' own entrants
//! (`GameText.contest_rivals`), the two said as the entry cards wrote them, "Jordyn & Seedottie",
//! the player's by their card alone. The computer's entrants are learned before that too, in the
//! first round, which appeals in place order: each appeal's message names the Pokémon whose turn
//! it is, "Bolt tried to show its appeal with Light Screen!". Each name is kept for its entrant's
//! block.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{f32, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::pictures::PLACES;
use crate::games::common::screen::ScreenText;
use crate::games::common::{lyt, strbuf};
use crate::games::gen6::anchors::follow;
use crate::games::gen6::apps::LOADED_CODE;
use crate::games::gen6::game::StandingsReader;
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::screens::rows::{CONTEST_MAIN, CONTEST_UI, PICKED};
use crate::py::{self, Error, Result};

/// The entry cards' app, then the appeals' (spike 196). The appeals were taken for the cards'
/// until a new app built where the last ran was noticed (2026-10-09, or_broken_contest).
pub const APPS: [&str; 2] = ["N7contest11OhiromeProcE", "N7contest8MainProcE"];
/// The appeals' process -> its `contest::Main` -> its `contest::AppealUi` (spike B).
pub const MAIN: i64 = CONTEST_UI[0];
pub const APPEAL_UI: i64 = CONTEST_UI[1];
/// `contest::Main` -> the appeals: each entrant's appeal and place (spike B).
pub const APPEALS: i64 = 0x24;
/// The appeals -> the rounds done.
pub const ROUNDS: i64 = 0x1;
/// The appeals -> the first entrant's block, the player's.
pub const BLOCKS: i64 = 0xF8;
pub const BLOCK: i64 = 0x10;
/// An entrant's block -> its place, 1 to 4.
pub const PLACE: i64 = 0xD;
/// The player's entrant, the first block.
pub const PLAYER: i64 = 0;
/// The pick of the leftmost portrait, first place's.
pub const FIRST_PICK: i64 = 4;
/// `contest::Main` -> the Pokémon of the entrant the cursor has picked (spike B).
pub const PICKED_NAME: i64 = 0x260;
/// Halfwords past which a Pokémon's name is no name: a nickname is twelve at most.
pub const NAME_LIMIT: i64 = 16;
pub const BACKGROUND: &str = "contest_player_bg_01.bclim";
pub const BAR: &str = "appeal_bar_filler.bclim";
pub const ICON: &str = "poke_icon.bclim";
pub const CURSOR: &str = "cursor.bclim";
pub const ENTRANTS: i64 = 4;
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

/// Each of the computer's entrants, by its block, -> the entrant whose appeal came in its
/// turn: until the first round ends the places are its turn order (spike B). The player's turn
/// names no entrant of `appealed`, which holds the computer's alone.
pub fn by_turn(places: &[i64], appealed: &[(String, String)]) -> IndexMap<i64, (String, String)> {
    let theirs = py::sorted_by_lt(
        (0..places.len() as i64)
            .filter(|&entrant| entrant != PLAYER)
            .collect(),
        |&a, &b| places[a as usize] < places[b as usize],
    );
    theirs.into_iter().zip(appealed.iter().cloned()).collect()
}

/// The computer's entrant whose Pokémon this is, as its entry card wrote them, "Jordyn &
/// Seedottie", or None for the player's own or a name two rivals share and no card tells
/// apart.
pub fn entrant(
    pokemon: &str,
    rivals: &[(String, String)],
    card: &dyn Fn(&str, &str) -> Option<String>,
) -> Option<String> {
    let trainers: Vec<&String> = rivals
        .iter()
        .filter(|(_trainer, theirs)| theirs == pokemon)
        .map(|(trainer, _theirs)| trainer)
        .collect();
    let carded: Vec<String> = trainers
        .iter()
        .filter_map(|trainer| card(trainer, pokemon))
        .collect();
    if carded.len() == 1 {
        return Some(carded[0].clone());
    }
    if trainers.len() == 1 && carded.is_empty() {
        Some(format!("{}{JOIN}{pokemon}", trainers[0]))
    } else {
        None
    }
}

/// The player's place, or each other entrant's in order, `places` each entrant's by its
/// block, the player's first: "Brendan & Pikachu first place".
///
/// Python's `pauses` defaults to False.
pub fn standings(
    places: &[i64],
    name: &dyn Fn(i64) -> Option<String>,
    player: bool,
    pauses: bool,
) -> Option<String> {
    let words: Vec<String> = PLACES.iter().map(|&(_, place)| own(place, &[])).collect();
    let mut lines: Vec<String> = Vec::new();
    for entrant in py::sorted_by_lt((0..places.len() as i64).collect(), |&a, &b| {
        places[a as usize] < places[b as usize]
    }) {
        if (entrant == PLAYER) != player {
            continue;
        }
        let at = places[entrant as usize];
        let place = if 0 < at && at <= words.len() as i64 {
            words[(at - 1) as usize].clone()
        } else {
            at.to_string()
        };
        let named = name(entrant);
        lines.push(match named {
            Some(named) if !named.is_empty() => {
                format!("{named}{}{place}", if pauses { ": " } else { " " })
            }
            _ => place,
        });
    }
    let joined = lines.join(", ");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

/// The appeals as the game keeps them.
#[derive(Debug, Clone, PartialEq)]
pub struct Standing {
    /// The rounds done.
    pub rounds: i64,
    /// Each entrant's place, by its block, the player's first.
    pub places: Vec<i64>,
}

/// The rounds done and each entrant's place, from the appeals' bytes up to the end of the
/// last block, or None where the places are not each of 1 to 4.
pub fn standing(appeals: &[u8]) -> Result<Option<Standing>> {
    let mut places: Vec<i64> = Vec::new();
    for at in 0..ENTRANTS {
        let byte = appeals
            .get((BLOCKS + BLOCK * at + PLACE) as usize)
            .ok_or_else(|| Error::index("index out of range"))?;
        places.push(*byte as i64);
    }
    let mut sorted = places.clone();
    sorted.sort();
    if sorted != (1..ENTRANTS + 1).collect::<Vec<i64>>() {
        return Ok(None);
    }
    let rounds = appeals
        .get(ROUNDS as usize)
        .ok_or_else(|| Error::index("index out of range"))?;
    Ok(Some(Standing {
        rounds: *rounds as i64,
        places,
    }))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Portrait {
    pub x: f64,
    pub y: f64,
}

/// The four portraits, and where the cursor stands.
type Found = (Vec<Portrait>, Option<(f64, f64)>);

/// Who stands where in a contest's appeals, from the game's own record of them.
pub struct Standings {
    memory: Rc<dyn Memory>,
    process: Rc<dyn Fn() -> Option<i64>>,
    /// The appeals' `contest::Main`, once its class is checked.
    ///
    /// A `Cell`: `said` takes `&self` and checks it, as the Python's does.
    main: Cell<Option<i64>>,
    layouts: Rc<Layouts>,
    files: Rc<LayoutFiles>,
    /// The game's "Pokémon:" and "Trainer:".
    labels: Vec<String>,
    running: Rc<dyn Fn() -> Option<String>>,
    /// Each entrant learned, by its block.
    names: IndexMap<i64, String>,
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
    /// The computer's entrants the messages have named, in the order first named.
    appealed: Vec<(String, String)>,
    /// The app running at the last poll.
    app_before: Option<String>,
}

impl Standings {
    /// Python's `rivals` defaults to none, and its `process` to one that is always None.
    pub fn new(
        memory: Rc<dyn Memory>,
        layouts: Rc<Layouts>,
        files: Rc<LayoutFiles>,
        labels: Vec<String>,
        running: Rc<dyn Fn() -> Option<String>>,
        rivals: Vec<(String, String)>,
        process: Rc<dyn Fn() -> Option<i64>>,
    ) -> Self {
        Standings {
            memory,
            process,
            main: Cell::new(None),
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
            appealed: Vec::new(),
            app_before: None,
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
        process: Rc<dyn Fn() -> Option<i64>>,
    ) -> Rc<RefCell<dyn StandingsReader>> {
        Rc::new(RefCell::new(Standings::new(
            memory,
            layouts,
            files,
            text.contest_entrant.clone(),
            running,
            text.contest_rivals.clone(),
            process,
        )))
    }
}

impl StandingsReader for Standings {
    /// Learn the entrant the panel names for the portrait the cursor is on, and the entrant
    /// of a portrait the cursor has just come onto, to say.
    fn poll(&mut self, on_screen: &[ScreenText]) -> Result<Option<String>> {
        let running = (self.running)();
        if running.as_deref() == Some(APPS[0]) && self.app_before.as_deref() != Some(APPS[0]) {
            // A new contest's entry cards. What the last contest taught is forgotten here, not as
            // another app runs: one of no name runs between the cards and the appeals, and the
            // cards were forgotten with it (or_toughness_contest).
            self.names.clear();
            self.written.clear();
            self.appealed.clear();
        }
        self.app_before = running.clone();
        if !running.is_some_and(|app| APPS.contains(&app.as_str())) {
            return Ok(None);
        }
        self.written
            .extend(on_screen.iter().map(|piece| piece.text.clone()));
        let named = self.named(on_screen);
        let found = self.portraits()?;
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
        let mut arrived = None;
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
            if at.is_some() {
                arrived = self.entrant()?;
            }
        }
        // Read the same on two polls: caught as the panel is rewritten, the player's portrait
        // was taken for Gianna & Tailster's (movie33).
        let steady = named == self.last;
        self.last = named.clone();
        if at.is_some() && named.is_some() && named != self.arrived_with && steady {
            if let Some(picked) = self.picked()? {
                self.names.insert(picked, named.clone().unwrap_or_default());
            }
            self.arrived_with = named;
        }
        Ok(arrived)
    }

    /// Learn the computer's entrants from the first round's messages: "Bolt tried to show
    /// its appeal" is the entrant whose turn it is, and the first round appeals in place order.
    /// Only entrants whose entry cards were shown are looked for; the panel's names win.
    fn heard(&mut self, texts: &[&str]) -> Result<()> {
        if (self.running)().as_deref() != Some(APPS[1]) {
            return Ok(());
        }
        let carded: Vec<(String, String)> = self
            .rivals
            .iter()
            .filter(|(trainer, pokemon)| self.card(trainer, pokemon).is_some())
            .cloned()
            .collect();
        let before = self.appealed.len();
        for text in texts {
            for rival in named_in(text, &carded) {
                if !self.appealed.contains(&rival) {
                    self.appealed.push(rival);
                }
            }
        }
        if self.appealed.len() == before {
            return Ok(());
        }
        let Some(now) = self.standing()? else {
            return Ok(());
        };
        if now.rounds != 0 {
            return Ok(());
        }
        for (block, (trainer, pokemon)) in by_turn(&now.places, &self.appealed) {
            let card = self.card(&trainer, &pokemon).unwrap_or_default();
            self.names.entry(block).or_insert(card);
        }
        Ok(())
    }

    /// The player's place, or each other entrant's in order, "Brendan & Pikachu first place";
    /// None outside the appeals or where they are not found.
    ///
    /// Python's `pauses` defaults to False.
    fn said(&self, player: bool, pauses: bool) -> Result<Option<String>> {
        let now = if (self.running)().as_deref() == Some(APPS[1]) {
            self.standing()?
        } else {
            None
        };
        let Some(now) = now else {
            return Ok(None);
        };
        let name = |block: i64| -> Option<String> {
            self.names
                .get(&block)
                .filter(|name| !name.is_empty())
                .cloned()
        };
        Ok(standings(&now.places, &name, player, pauses))
    }
}

impl Standings {
    /// The entrant the cursor has picked, by its Pokémon, learned for its block; the player's
    /// own as the panel taught it, else as their entry card wrote it, else their Pokémon; or
    /// None where no name is read or learned.
    fn entrant(&mut self) -> Result<Option<String>> {
        let block = self.picked()?;
        let learned = match block {
            None => None,
            Some(block) => self
                .names
                .get(&block)
                .filter(|learned| !learned.is_empty())
                .cloned(),
        };
        let Some(pokemon) = self.pokemon()? else {
            return Ok(learned);
        };
        // The player's own came as their Pokémon alone until the panel had named it as their
        // card did (or_toughness_contest).
        let card = |trainer: &str, pokemon: &str| self.card(trainer, pokemon);
        let named = match entrant(&pokemon, &self.rivals, &card).filter(|named| !named.is_empty()) {
            Some(named) => Some(named),
            None => learned.or_else(|| self.own_card(&pokemon)),
        };
        let Some(named) = named else {
            return Ok(Some(pokemon));
        };
        if let Some(block) = block {
            self.names.insert(block, named.clone());
        }
        Ok(Some(named))
    }

    /// The appeals' `contest::Main`, its class checked once, or None out of the appeals.
    fn contest(&self) -> Result<Option<i64>> {
        let Some(process) = (self.process)() else {
            return Ok(None);
        };
        if (self.running)().as_deref() != Some(APPS[1]) {
            return Ok(None);
        }
        let Some(main) = follow(&*self.memory, process, &[MAIN])? else {
            return Ok(None);
        };
        if Some(main) != self.main.get() {
            if ClassNames::new(self.memory.clone(), LOADED_CODE)
                .of_object(main)?
                .as_deref()
                != Some(CONTEST_MAIN)
            {
                return Ok(None);
            }
            self.main.set(Some(main));
        }
        Ok(Some(main))
    }

    /// The appeals as the game keeps them, or None out of them.
    fn standing(&self) -> Result<Option<Standing>> {
        let appeals = match self.contest()? {
            None => None,
            Some(main) => follow(&*self.memory, main, &[APPEALS])?,
        };
        let Some(appeals) = appeals else {
            return Ok(None);
        };
        standing(&self.memory.read(appeals, BLOCKS + BLOCK * ENTRANTS)?)
    }

    /// The block of the entrant the cursor has picked: the portraits stand in place order as
    /// a move is chosen, the only time the cursor is on them. None out of the appeals.
    fn picked(&self) -> Result<Option<i64>> {
        let ui = match self.contest()? {
            None => None,
            Some(main) => follow(&*self.memory, main, &[APPEAL_UI])?,
        };
        let now = match ui {
            None => None,
            Some(_) => self.standing()?,
        };
        let (Some(ui), Some(now)) = (ui, now) else {
            return Ok(None);
        };
        let place = *self
            .memory
            .read(ui + PICKED, 1)?
            .first()
            .ok_or_else(|| Error::index("index out of range"))? as i64
            - FIRST_PICK
            + 1;
        Ok(now
            .places
            .iter()
            .position(|&each| each == place)
            .map(|at| at as i64))
    }

    /// The Pokémon of the entrant the cursor has picked, as the game wrote it for its panel,
    /// or None out of the appeals.
    fn pokemon(&self) -> Result<Option<String>> {
        let buffer = match self.contest()? {
            None => None,
            Some(main) => follow(&*self.memory, main, &[PICKED_NAME])?,
        };
        match buffer {
            None => Ok(None),
            Some(buffer) => strbuf::label_with(&*self.memory, buffer, NAME_LIMIT),
        }
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

    /// The entry card of the entrant whose Pokémon this is, the player's having no trainer
    /// among the contests' entrants: words the contest showed ending with it, a name and a
    /// joiner longer at most, or None.
    fn own_card(&self, pokemon: &str) -> Option<String> {
        let length = pokemon.chars().count();
        let most = length + NAME_LIMIT as usize + JOIN.chars().count();
        self.written
            .iter()
            .find(|words| {
                let count = words.chars().count();
                words.ends_with(pokemon) && length < count && count <= most
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
                if self.is_portrait(root, panel, &texture)? {
                    let head = layouts.pane(panel)?;
                    portraits.push(Portrait {
                        x: f32(&head, shape.x)?,
                        y: f32(&head, shape.y)?,
                    });
                }
            }
            if portraits.len() as i64 != ENTRANTS {
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

    /// Whether a pane is a portrait: its bar, its icon and its background under it, the bar
    /// its own child.
    fn is_portrait(
        &self,
        root: i64,
        panel: i64,
        texture: &dyn Fn(i64) -> Result<Option<String>>,
    ) -> Result<bool> {
        let mut found: IndexMap<String, i64> = IndexMap::new();
        let mut stack: Vec<i64> = self.layouts.children(root, panel);
        while let Some(pane) = stack.pop() {
            let kind = texture(pane)?;
            if let Some(kind) = kind
                && [BAR, ICON, BACKGROUND].contains(&kind.as_str())
            {
                found.insert(kind, pane);
            }
            stack.extend(self.layouts.children(root, pane));
        }
        Ok(found.len() == 3 && u32(&self.layouts.pane(found[BAR])?, lyt::PARENT)? == panel)
    }
}
