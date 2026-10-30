//! What to say of a screen: the item under its cursor, what changed, and what came on screen.
//!
//! The screen itself, what comes on it and what is under its cursors, is `screen_announce`'s.
//!
//! `Announcements` says what the other readers report: dialogue pages and text written over a
//! window, the option under a list's or a choice box's cursor, and each side of a battle.
//!
//! Bound by:
//! - ADR-0013: within a poll, the item under a cursor, then the rows changed, then what came on
//!   screen.
//! - ADR-0023: a speaker, the narrator among them, is named as they take over and again once a
//!   screen's words have come between their lines.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::speech::{Category, Speech};
use crate::core::words::own;
use crate::games::common::battle::Battler;
use crate::games::common::dialogue::{PageShown, Shown, TextWritten};
use crate::py::{Log, Result};

/// Screen polls within which words a list or a choice box said are theirs, not a cursor's: the
/// cursor beside a choice box's words opened on "Yes" after the box had been answered and its
/// labels were gone, while its layout was still on screen (movie1, spike 99).
pub const FOCUS_HEARD_POLLS: i64 = 10;

/// The option under a cursor: on opening, or after the cursor moved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Focus {
    /// What it came from: a list window's controller, a choice box, a language choice.
    pub source: i64,
    pub label: String,
    pub index: i64,
    pub count: i64,
    pub opened: bool,
}

/// Dialogue polls a message's first page waits for the name on its plate, where the game names its
/// speakers: Sun builds the talk window after its first page has begun to type, and the heaps'
/// watch finds it a look or two on, so the name came a moment after the page and 20 of sun_4.ctm's
/// 23 pages were named (issue 4). A fifth of a second at the runtime's tick.
pub const SPEAKER_POLLS: i64 = 12;
/// Who a scene's description is said as, a speaker of pa3ds's own (ADR-0038): the message
/// saying its name, "Narration".
pub const NARRATOR: &str = "narration";

/// `speaker: Callable[[int], str | None]`: who is showing a window's page. It reads the game's
/// memory (`sm.talk.Speakers.name_for`), so it can raise, and the Rust one returns a `Result` for
/// that: the raise reaches the caller of `pages`, as it does in Python.
pub type Speaker = Rc<dyn Fn(i64) -> Result<Option<String>>>;

/// Says what the dialogue, list, choice box and battle readers report, and logs each.
pub struct Announcements {
    speech: Rc<Speech>,
    log: Log,
    speaker: Option<Speaker>,
    hold: bool,
    /// Who was named on the last page said, so a conversation names each speaker as it takes over
    /// rather than before every line.
    said_speaker: Option<String>,
    /// Screen lines spoken by then (`Speech.spoken`): a screen's words coming between two of a
    /// speaker's lines name them again (Orion, 2026-10-27).
    screens: i64,
    /// First pages with no name yet, each with the polls it has waited (`SPEAKER_POLLS`).
    unnamed: Vec<(i64, PageShown)>,
    /// Text written over a message window, each with the number of screen text polls before it,
    /// held until a whole poll has passed. The options screen writes an option's description as
    /// the cursor lands on it, and dialogue is polled every frame while the cursor is polled every
    /// 0.2 s, so without holding it the description was said before the option it describes.
    written: Vec<(i64, TextWritten)>,
    screen_polls: i64,
    /// Panel -> what was last said for it in this battle, for the keys that say it again.
    sides: IndexMap<i64, Battler>,
    /// Words a list or choice box said lately -> the screen poll they were said at.
    focused: IndexMap<String, i64>,
}

impl Announcements {
    /// `Announcements(speech, log)`: no speaker, and no holding for one.
    pub fn new(speech: Rc<Speech>, log: Log) -> Self {
        Announcements::with(speech, log, None, false)
    }

    /// `speaker` names who is showing a window's page; with `hold_for_speaker`, a first page
    /// with no name yet waits `SPEAKER_POLLS` for one. Sun's game names its speakers a moment
    /// after the page begins; ORAS's names are pa3ds's own, from a talk object a sweep finds,
    /// and a page there is said as it comes.
    pub fn with(
        speech: Rc<Speech>,
        log: Log,
        speaker: Option<Speaker>,
        hold_for_speaker: bool,
    ) -> Self {
        let hold = hold_for_speaker && speaker.is_some();
        Announcements {
            speech,
            log,
            speaker,
            hold,
            said_speaker: None,
            screens: 0,
            unnamed: Vec::new(),
            written: Vec::new(),
            screen_polls: 0,
            sides: IndexMap::new(),
            focused: IndexMap::new(),
        }
    }

    /// The words lists and choice boxes said within `FOCUS_HEARD_POLLS`, which a cursor over
    /// the same words has no news in.
    pub fn recent_labels(&mut self) -> HashSet<String> {
        self.focused = self
            .focused
            .iter()
            .filter(|&(_label, &poll)| self.screen_polls - poll <= FOCUS_HEARD_POLLS)
            .map(|(label, &poll)| (label.clone(), poll))
            .collect();
        self.focused.keys().cloned().collect()
    }

    /// Say pages as they come, and hold written text for the screen poll. Returns whether a
    /// page was said.
    pub fn pages(&mut self, shown: &[Shown]) -> Result<bool> {
        let mut said = false;
        let mut fresh: Vec<PageShown> = Vec::new();
        for item in shown {
            match item {
                Shown::Text(item) => self.written.push((self.screen_polls, item.clone())),
                Shown::Page(item) => fresh.push(item.clone()),
            }
        }
        for item in self.ready(fresh)? {
            let window = item.window;
            self.say(&item)?;
            // A page covers whatever was written over its window before it.
            self.written.retain(|held| held.1.window != window);
            said = true;
        }
        Ok(said)
    }

    /// The pages to say now, in order: those held for a name that has come, or that have
    /// waited `SPEAKER_POLLS`, or whose window has moved on to another page; then the new ones,
    /// less a first page with no name yet, which is held.
    fn ready(&mut self, fresh: Vec<PageShown>) -> Result<Vec<PageShown>> {
        let speaker = match &self.speaker {
            Some(speaker) if self.hold => speaker.clone(),
            _ => return Ok(fresh),
        };
        let mut out: Vec<PageShown> = Vec::new();
        let moved_on: HashSet<i64> = fresh.iter().map(|item| item.window).collect();
        let mut kept: Vec<(i64, PageShown)> = Vec::new();
        // The list is replaced only once the loop has run, as in Python: a speaker that raises
        // leaves the pages as they were held.
        for (waited, item) in self.unnamed.clone() {
            let waited = waited + 1;
            if moved_on.contains(&item.window)
                || waited >= SPEAKER_POLLS
                || speaker(item.window)?.is_some()
            {
                out.push(item);
            } else {
                kept.push((waited, item));
            }
        }
        self.unnamed = kept;
        for item in fresh {
            if item.index == 0 && speaker(item.window)?.is_none() {
                self.unnamed.push((0, item));
            } else {
                out.push(item);
            }
        }
        Ok(out)
    }

    /// Say the last text written over each window that has waited a whole screen text poll.
    ///
    /// Only the last text is said: each covers the one before it, and moving through two options
    /// inside one poll read both. It waits a whole poll because the cursor is read once a poll:
    /// moving from BATTLE EFFECTS to BATTLE STYLE inside one poll said BATTLE STYLE with BATTLE
    /// EFFECTS' description, and BATTLE STYLE's own, written just after the poll, was never said
    /// (menus.ctm replay, 2026-09-14).
    pub fn screen_polled(&mut self) {
        let ready: HashSet<i64> = self
            .written
            .iter()
            .filter(|(came, _item)| *came < self.screen_polls)
            .map(|(_came, item)| item.window)
            .collect();
        let mut latest: IndexMap<i64, TextWritten> = IndexMap::new();
        for (_came, item) in &self.written {
            if ready.contains(&item.window) {
                latest.insert(item.window, item.clone());
            }
        }
        for item in latest.values() {
            self.say_text(item);
        }
        self.written
            .retain(|(_came, item)| !ready.contains(&item.window));
        self.screen_polls += 1;
    }

    /// Say the option under a list's or a choice box's cursor, as it opens or moves. An
    /// opening already `heard` from a cursor over the same words is logged and not said again.
    ///
    /// Its place in the list is logged and never said (Orion, 2026-09-30): the settings that said
    /// it reached only the lists that know their length, and a cursor's menu would have needed
    /// the length and the layout of every menu in every game.
    pub fn focus(&mut self, focus: &Focus, kind: &str, heard: bool) {
        self.focused.insert(focus.label.clone(), self.screen_polls);
        let text = &focus.label;
        let place = format!("({} of {})", focus.index + 1, focus.count);
        if focus.opened {
            if heard {
                (self.log)(&format!(
                    "{kind} {:#010x} opened, just said by a cursor: {text} {place}",
                    focus.source
                ));
                return;
            }
            (self.log)(&format!(
                "{kind} {:#010x} opened: {text} {place}",
                focus.source
            ));
            self.speech.say(text, Category::Prompt);
        } else {
            (self.log)(&format!(
                "{kind} {:#010x} moved: {text} {place}",
                focus.source
            ));
            self.speech.say(text, Category::Menu);
        }
    }

    /// Say each side of a battle whose reading has changed and settled.
    ///
    /// Queued, like every battle message, so it follows "The wild Zigzagoon used Tackle!" rather
    /// than cutting it off, which is the order the user asked for (2026-09-16).
    pub fn battlers(&mut self, sides: &[Battler]) {
        for side in sides {
            (self.log)(&format!(
                "battle {}: {}",
                if side.player { "ours" } else { "theirs" },
                side.text
            ));
            self.speech.say(&side.text, Category::Battle);
            self.sides.insert(side.panel, side.clone());
        }
    }

    /// The player's side as last said, or the enemy's, each Pokémon on it in the order the
    /// battle said them; None before anything was said this battle, or out of one.
    pub fn sides(&self, player: bool) -> Option<String> {
        let said: Vec<&str> = self
            .sides
            .values()
            .filter(|side| side.player == player)
            .map(|side| side.text.as_str())
            .collect();
        let said = said.join(", ");
        if said.is_empty() { None } else { Some(said) }
    }

    /// Another app has started: a battle beginning, or the field coming back after one.
    pub fn forget_sides(&mut self) {
        self.sides.clear();
    }

    /// Say a scene's description as the narrator's line, named as a speaker is (Orion,
    /// 2026-10-12 and 2026-10-27). A speaker after it is named again.
    pub fn narrate(&mut self, text: &str) {
        let said = if self.takes_over(NARRATOR) {
            format!("{}: {text}", own(NARRATOR, &[]))
        } else {
            text.to_string()
        };
        (self.log)(&format!("scene [{}]: {text}", own(NARRATOR, &[])));
        self.speech.say(&said, Category::Scene);
    }

    /// Whether `name` is said before their line: when they take over from whoever spoke last,
    /// a speaker or the game's own unnamed text, or when a screen's words have come since their
    /// last line; not while they go on.
    fn takes_over(&mut self, name: &str) -> bool {
        let screens = self.speech.spoken(Category::Screen);
        let named = self.said_speaker.as_deref() != Some(name) || screens != self.screens;
        (self.said_speaker, self.screens) = (Some(name.to_string()), screens);
        named
    }

    /// Python's `_say` of a `TextWritten`, apart from the page's: it names nobody, so it cannot
    /// raise, where the page's speaker can.
    fn say_text(&mut self, item: &TextWritten) {
        // Written text is the option's details, said after it and never cutting it off: as a
        // menu's, under `interrupt` the description cut off the option it describes (ADR-0010
        // as amended 2026-10-05, ADR-0012 as amended 2026-10-10).
        (self.log)(&format!(
            "text {:#010x} (written over the window): {}",
            item.window, item.text
        ));
        self.speech.say(&item.text, Category::Screen);
    }

    /// Python's `_say` of a `PageShown`.
    fn say(&mut self, item: &PageShown) -> Result<()> {
        let name = match &self.speaker {
            Some(speaker) => speaker(item.window)?,
            None => None,
        };
        let said = match &name {
            None => {
                self.said_speaker = None;
                item.text.clone()
            }
            Some(name) => {
                if self.takes_over(name) {
                    format!("{name}: {}", item.text)
                } else {
                    item.text.clone()
                }
            }
        };
        let who = match &name {
            None => String::new(),
            Some(name) => format!(" [{name}]"),
        };
        (self.log)(&format!(
            "dialogue {:#010x} page {} ({}){who}: {}",
            item.window, item.index, item.why, item.text
        ));
        self.speech.say(&said, Category::Dialogue);
        Ok(())
    }
}
