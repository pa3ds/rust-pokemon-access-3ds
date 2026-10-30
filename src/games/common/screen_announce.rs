//! What to say of a screen: the item under its cursor, what changed, and what came on screen.
//!
//! The screen text reader reports pieces and the cursor reader reports the pieces under each
//! cursor. This decides what is said of them, and in what order, by the rules the user set
//! (2026-09-14):
//!
//! - **A menu is heard one item at a time, under its cursor.** A layout a cursor has been over is
//!   a menu: what comes on screen with it, as it opens or comes back, and its rows as a list
//!   scrolls, whether the list writes new words into them or moves them into view, are left to the
//!   cursor. Words that come later in a menu already up, such as the options screen's "Would you
//!   like to make these changes?", are said.
//! - **Details follow the cursor, whole and in one order.** What changes after a cursor moves is
//!   gathered until it has settled, then every panel that cursor's moves have changed is read, row
//!   by row, in the order the screen built them: "The user growls… Category Power — Accuracy 100".
//!   Saying each change as a poll caught it put a move's description before its panel or after it,
//!   and left the panel out when two moves shared their power and accuracy (the user, 2026-09-14).
//!   A value that changes with no move is said with its row in its own group: "Play time: 0:14".
//!   Words coming on screen while the details gather are part of them: the Mart's count and
//!   description coming back as the cursor leaves Quit, a contest's move panel shown afresh for
//!   each move.
//! - **A screen rewritten is said as an item and its details** (Orion, 2026-10-10). Several rows
//!   changing at once with no cursor moving, as the summary turns to another Pokémon or the
//!   Pokédex to another entry, are a rewrite: its first row is said at once, the one naming what
//!   the screen shows where that is known (`subject`), and the rest is gathered as details are
//!   and said whole as one line once settled: "Umbra", then "Lv.5, HP 19/19, Attack 9...". A row
//!   a line, they were heard in pieces, some mixed with their neighbours', and under `interrupt`
//!   each cut off the one before.
//! - **A screen coming back is read like a screen opening** (the user, 2026-09-14): in full,
//!   unless it is a menu. Words coming back to a layout that kept its other words on screen bring
//!   those with them.
//! - **A letter deleted from a name is heard as the name that is left** (the user, 2026-09-18).
//!   Each letter typed is a TextBox of its own, so deleting one takes a word off the screen and
//!   says nothing at all; what remains is said instead, "R a i" for a name that read "Raiq".
//! - **A screen revealed over several polls is said once it has arrived**, whole: what comes on
//!   screen in a layout's first second waits while more of its words are still being written, up
//!   to a second, and is said as it came the moment any of it changes or leaves, ahead of the
//!   change.
//! - **A clock is said at its marks** (Orion, 2026-10-26): while a game that times the player
//!   runs, Pokémon-Amie's three, a number going one up or down a second after it last changed is
//!   its clock ticking, said each half minute and, counting down, at 10 and each of the last 5,
//!   once while it bounces round one. They said "Time Left" every second for a minute. Every tick
//!   can sound a tone, `[pa3ds.audio] ticks`. While the player plays, a Tile Puzzle with a piece
//!   away from home, its numbers are the play's, unsaid but for the clock's marks (Orion,
//!   2026-10-28).
//! - **Words coming back where words of their panel are leaving come alone**, and so does every
//!   word coming back while a game that times the player runs: neither is a screen coming back
//!   (2026-10-28).
//! - The item under a cursor comes first, then its details, then what changed, then what came on
//!   screen. Nothing interrupts unless the speech policy says so.
//!
//! List and choice box labels belong to their own readers and are left out everywhere.
//!
//! Bound by:
//! - ADR-0010: a cursor's details are one line, after its item, and never interrupt.
//! - ADR-0013: each clause of screen text is from live play: replay the tapes before and after
//!   changing one.

use std::collections::HashSet;
use std::ops::Range;
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::{IndexMap, IndexSet};
use regex::Regex;

use crate::core::speech::{Category, Speech};
use crate::games::common::cursors::shared::CursorPoll;
use crate::games::common::screen::{
    ScreenPoll, ScreenText, by_rows, distinct, in_rows, join, labels_value, reading_order,
};
use crate::py::{self, Log, Result};

/// Polls after a cursor moves before its details are said, when none of them is still being
/// written. The move panel changed one poll after its cursor in the user's log.
pub const DETAIL_POLLS: i64 = 2;

/// The most polls details wait for words still being written, so that a value that never holds
/// still cannot keep them unsaid.
pub const DETAIL_MAX_POLLS: i64 = 8;

/// The most polls words that came on screen wait for the rest of their layout's words to arrive,
/// so that a value that never holds still cannot keep them unsaid.
pub const ARRIVAL_MAX_POLLS: i64 = 5;

/// Glyphs between a label and the value it names: the level up's are five apart, a name's letters
/// one.
pub const LABEL_GAP: f64 = 2.0;

/// Polls within which the same words from another layout are the same news: the location sign
/// came up 0.23 s after AreaNav's words changed.
pub const REPEAT_POLLS: i64 = 5;

/// The root a place's name said from RAM counts as coming from. No layout's root is 0.
pub const FIELD_ROOT: i64 = 0;

/// Units within which two words beside each other are drawn on one line, as a table's label and
/// value are.
pub const ONE_LINE: f64 = 1.0;

/// Polls within which a place's name on screen is the one the field has just said. AreaNav's words
/// changed 0.6 to 0.8 s after the zone did and the location sign came 0.23 s after them (spike
/// 87), which is five polls at the edge of `REPEAT_POLLS`; two seconds leaves room for a slow
/// poll.
pub const PLACE_POLLS: i64 = 10;

/// Polls within which words a cursor said are old news to a choice box opening on them. The Poké
/// Mart's Buy, Sell and No thanks are read by the cursor beside them the poll their box is shown,
/// and by the choice reader once a sweep has found the box, as much as a second later (spike 98);
/// two seconds leaves room for a slow one.
pub const CURSOR_HEARD_POLLS: i64 = 10;

/// The fewest rows changing in one poll, with no cursor moving, that are a screen rewritten: the
/// summary turning to another Pokémon changed 17. One row alone, a play time or a question, is
/// said at once.
pub const REWRITE_ROWS: i64 = 2;

/// Polls from one tick of a clock to the next: a second is five, and Berry Picker's "Time Left"
/// was caught 0.82 to 1.05 s apart; a slow poll moves it a little either way.
pub const CLOCK_POLLS: Range<i64> = 3..9;

static NUMBERS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d+").expect("a fixed pattern"));

/// A layout root and the pane a group of pieces hangs from, or the frame drawn round words hanging
/// from the root itself (`ScreenText.group`).
pub type Panel = (i64, i64);

/// The details of the item a cursor has just moved to, gathered until they settle.
struct Gathering {
    cursor: i64,
    polls: i64,
    /// The panels of the pieces that changed, in the order they changed.
    panels: IndexSet<Panel>,
    /// The panels of the pieces that came on screen meanwhile: said with the details, and not
    /// kept for the moves after, as the panels of a change are. The PC Box's "Potion selected.",
    /// kept, was said after every move of its menu (movie11).
    joined: IndexSet<Panel>,
    /// TextBoxes said meanwhile as they came on screen, left out of the details.
    said: HashSet<i64>,
    /// The item has no details to say: what changes is taken in unsaid (`CursorItem.quiet`).
    quiet: bool,
    /// The item is a count alone, said after the first row of its details, which name what it
    /// counts (`ScreenAnnouncer._say_details`).
    count: String,
    /// The item was not said as the cursor moved: a count, or nameless, said by its details.
    held: bool,
    /// For a screen rewritten, the layout its first row was said from, whose rest comes first,
    /// and that row's words, said once.
    lead: (i64, String),
    /// What the cursor said of the item, left out of its details.
    item: String,
    /// The root of the layout the item is in.
    menu: i64,
    /// The list of rows the item is one of, where its cursor knows it (`CursorItem.rows`): the
    /// list its details are of. The fitting room's Change and Back are a list beside its tabs, and
    /// each move between them read again the card the tabs had changed (Khai,
    /// x_cyllage_boutique_fitting_room).
    rows: i64,
}

impl Gathering {
    /// What the panels its moves change are kept under (`ScreenAnnouncer.detail_panels`).
    fn kept(&self) -> (i64, i64, i64) {
        (self.cursor, self.menu, self.rows)
    }

    /// `_Gathering(cursor, quiet=quiet, count=count, held=held)`: `polls` at 0 and no panels or
    /// words said yet.
    fn new(cursor: i64, quiet: bool, count: String, held: bool) -> Self {
        Gathering {
            cursor,
            polls: 0,
            panels: IndexSet::new(),
            joined: IndexSet::new(),
            said: HashSet::new(),
            quiet,
            count,
            held,
            lead: (0, String::new()),
            item: String::new(),
            menu: 0,
            rows: 0,
        }
    }
}

/// `Callable[[], str | None]`: the name of what a screen shows, asked as it is rewritten. It reads
/// the game, so it can raise.
pub type Subject<'a> = &'a dyn Fn() -> Result<Option<String>>;

/// Says a screen's text and its cursors' items, one poll at a time.
pub struct ScreenAnnouncer {
    speech: Rc<Speech>,
    log: Log,
    pauses: bool,
    tick: Option<Rc<dyn Fn()>>,
    timed: Option<Rc<dyn Fn() -> bool>>,
    playing: Option<Rc<dyn Fn() -> Result<bool>>>,
    /// Clock's TextBox -> the mark it was said at last, said once while it bounces round it.
    marks: IndexMap<i64, i64>,
    /// TextBox -> the words it came on screen with or changed to lately, and the poll, for
    /// telling a clock's tick (`_ticked`).
    set: IndexMap<i64, (String, i64)>,
    /// TextBoxes that have ticked, for as long as they keep being set.
    clocks: HashSet<i64>,
    /// Roots of the layouts a cursor has been over, for as long as those layouts live.
    menus: HashSet<i64>,
    gathering: Option<Gathering>,
    /// A screen rewritten with no cursor moving, gathered until it settles.
    rewrite: Option<Gathering>,
    /// Cursor -> the words it opened on and the poll it did, for one that only slides into
    /// place the poll after: the Bag's said "Pokéblock Kit" as it opened and again 0.2 s later
    /// (movie30).
    opened: IndexMap<i64, (String, i64)>,
    /// (cursor, layout of its item, its list) -> every panel its moves along that list have
    /// changed. Ember to Scratch changes only the description, and the move's power and accuracy
    /// went unsaid. Kept a layout at a time: the battle's move to forget, changed as the cursor
    /// went onto FORGET, was read again for "return" in the move list beside it
    /// (or_move-learning, 2026-10-18); and a list at a time (`_Gathering.rows`).
    detail_panels: IndexMap<(i64, i64, i64), IndexSet<Panel>>,
    polls: i64,
    /// Words a cursor said lately -> the poll it said them at and the cursor
    /// (`said_by_cursor`).
    cursor_said: IndexMap<String, (i64, i64)>,
    /// Words said lately -> (the poll they were said at, the root of the layout they came from).
    recent: IndexMap<String, (i64, i64)>,
    /// Layout root -> the poll its words came on screen at, for layouts with words on screen.
    on_screen_since: IndexMap<i64, i64>,
    /// Layout root -> the words that came on screen while more of the layout's words were still
    /// being written, each with the poll it came at.
    arriving: IndexMap<i64, Vec<(i64, ScreenText)>>,
    /// (Layout root, panel) -> the poll words last left it at, for `REPEAT_POLLS`.
    left: IndexMap<(i64, i64), i64>,
    /// The place's name the field said last, and the poll it was said at.
    placed: Option<(String, i64)>,
    /// TextBox -> its layout's root and the words last said of it, as screen text or details,
    /// until they change: a title still saying them, the Bag's pocket, is no detail
    /// (`heading`).
    words_said: IndexMap<i64, (i64, String)>,
}

impl ScreenAnnouncer {
    /// `ScreenAnnouncer(speech, log)`: no pauses.
    pub fn new(speech: Rc<Speech>, log: Log) -> Self {
        ScreenAnnouncer::with(speech, log, false)
    }

    /// `ScreenAnnouncer(speech, log, pauses)`: no clock.
    pub fn with(speech: Rc<Speech>, log: Log, pauses: bool) -> Self {
        ScreenAnnouncer::with_clock(speech, log, pauses, None, None, None)
    }

    /// `timed` says whether a game that times the player runs, whose clock is said at its
    /// marks (`_ticked`), none where it is None; `tick` sounds each tick of the clock; `playing`
    /// whether its player is in the middle of play, whose numbers are left unsaid.
    ///
    /// `playing` returns `Result`: it reads the game.
    pub fn with_clock(
        speech: Rc<Speech>,
        log: Log,
        pauses: bool,
        tick: Option<Rc<dyn Fn()>>,
        timed: Option<Rc<dyn Fn() -> bool>>,
        playing: Option<Rc<dyn Fn() -> Result<bool>>>,
    ) -> Self {
        ScreenAnnouncer {
            speech,
            log,
            pauses,
            tick,
            timed,
            playing,
            marks: IndexMap::new(),
            set: IndexMap::new(),
            clocks: HashSet::new(),
            menus: HashSet::new(),
            gathering: None,
            rewrite: None,
            opened: IndexMap::new(),
            detail_panels: IndexMap::new(),
            polls: 0,
            cursor_said: IndexMap::new(),
            recent: IndexMap::new(),
            on_screen_since: IndexMap::new(),
            arriving: IndexMap::new(),
            left: IndexMap::new(),
            placed: None,
            words_said: IndexMap::new(),
        }
    }

    /// Whether the details of the item a cursor has moved to are still to be said.
    pub fn gathering(&self) -> bool {
        self.gathering.is_some()
    }

    /// Did a cursor say these words a moment ago? A choice box or a list opening on them then
    /// has no news: the Mart's boxes are read by the cursor beside their words before the choice
    /// reader has found them, and "Buy" was heard from both (spike 98), as the save's YES was from
    /// its cursor and then its list (movie1, 2026-09-26).
    pub fn said_by_cursor(&mut self, text: &str) -> bool {
        self.cursor_said = self
            .cursor_said
            .iter()
            .filter(|&(_words, &said)| self.polls - said.0 <= CURSOR_HEARD_POLLS)
            .map(|(words, &said)| (words.clone(), said))
            .collect();
        self.cursor_said.contains_key(text)
    }

    /// Say the name of the place the player has just arrived in.
    ///
    /// It comes from RAM and the ROM (`Gen6Profile.poll_field`), 0.6 to 0.8 s before AreaNav's
    /// words and the location sign say the same (spike 87), so those are left unsaid when they
    /// follow it — and it is left unsaid when they came first, as they do before the field has
    /// found the zone table.
    pub fn place(&mut self, name: &str) {
        if !self.fresh(name, FIELD_ROOT) {
            (self.log)(&format!("field place, just said elsewhere: {name}"));
            return;
        }
        self.placed = Some((name.to_string(), self.polls));
        (self.log)(&format!("field place: {name}"));
        self.speech.say(name, Category::Screen);
    }

    /// Say one poll's worth: what the `screen` text reader and the `cursors` reader found, less
    /// what other readers own, the list readers' `labels`, the `roots` of every layout still
    /// alive, so a menu whose layout is gone is forgotten, and the `subject`, asked only as a
    /// screen is rewritten, for the name of what it shows where that is known: the summary's
    /// Pokémon's.
    ///
    /// A cursor that knows its list owns the list and no more of its layout. The Poké Mart draws
    /// its list, the money, the count in the bag and the chosen item's description in one
    /// layout, and left to the cursor as a menu's layout is, the description was never heard as
    /// the Mart opened, nor the count as it changed (shop.ctm, 2026-09-24). A cursor over
    /// pictures alone owns those pictures: the AreaNav's tabs are drawn in its bar's layout, and
    /// left to them its View Details was never heard (or_area_nav).
    ///
    /// Python's `roots` and `subject` default to None.
    pub fn say(
        &mut self,
        screen: &ScreenPoll,
        cursors: &CursorPoll,
        labels: &HashSet<String>,
        roots: Option<&IndexSet<i64>>,
        subject: Option<Subject>,
    ) -> Result<()> {
        self.polls += 1;
        let (texts, on_screen, settling, left) = (
            &screen.texts,
            &screen.on_screen,
            &screen.settling,
            &screen.left,
        );
        for piece in texts.iter().chain(left.iter()) {
            self.words_said.shift_remove(&piece.r#box);
        }
        let (items, under, listed, moving, rows) = (
            &cursors.items,
            &cursors.under,
            &cursors.lists,
            &cursors.moving,
            &cursors.rows,
        );
        let mut cursored: HashSet<i64> = rows.clone();
        for pieces in under {
            if !pieces.iter().any(|p| labels.contains(&p.text)) {
                self.menus
                    .extend(pieces.iter().filter(|p| !p.picture).map(|p| p.root));
                cursored.extend(pieces.iter().map(|p| p.r#box));
            }
        }
        for boxes in listed.values() {
            cursored.extend(boxes);
        }
        if let Some(roots) = roots {
            self.menus.retain(|root| roots.contains(root));
            self.detail_panels.retain(|_cursor, panels| {
                panels.retain(|panel| roots.contains(&panel.0));
                !panels.is_empty()
            });
            self.words_said.retain(|_box, said| roots.contains(&said.0));
        }

        let mut spoken: HashSet<String> = HashSet::new();
        let polls = self.polls;
        self.opened.retain(|_cursor, seen| polls - seen.1 <= 1);
        for item in items {
            // A list window draws the same kind of cursor. Over its labels, the cursor is the
            // list's, and what shows through behind the list is not its item: the options
            // confirmation's cursor held "SLOW ON" from the rows under Yes and No.
            if item.pieces.is_empty() || item.pieces.iter().any(|p| labels.contains(&p.text)) {
                continue;
            }
            let text = join(&item.pieces, self.pauses);
            // A count alone names nothing: the Pokéblock Kit's cursor sits on a picture of a
            // berry or a block and its count, and the screen writes what it is elsewhere, "16"
            // then "Cheri Berry Red" (movie30, spike 196). It is said after that, with it.
            // A nameless item, a button of no words, is said by its details alone.
            let held = (self::count(&text) || text.is_empty()) && !item.opened && !item.quiet;
            if text.is_empty() && item.opened {
                continue;
            }
            if item.opened {
                self.opened.insert(item.cursor, (text.clone(), self.polls));
            } else if self.opened.shift_remove(&item.cursor) == Some((text.clone(), self.polls - 1))
            {
                // The same words the poll after it opened: it has slid into place, not moved.
                (self.log)(&format!(
                    "cursor {:#010x} settled where it opened: {text}",
                    item.cursor
                ));
                cursored.extend(item.pieces.iter().map(|p| p.r#box));
                continue;
            }
            if let Some(rewrite) = self.rewrite.take() {
                // A cursor moving, or coming on screen with a screen of its own, drops the rest of
                // a screen being rewritten unsaid, as a move drops an item's details: said as it
                // stood, the Bag's panel, half written for the next item, read the next item's
                // name with the last one's count (movie33). Its panels are kept.
                self.detail_panels
                    .entry((0, 0, 0))
                    .or_default()
                    .extend(rewrite.panels.iter().copied());
                (self.log)(&format!(
                    "screen rewritten, rest unsaid: a cursor {:#010x} moved",
                    item.cursor
                ));
            }
            let count = if held { text.clone() } else { String::new() };
            // Or a poll apart, where one holds an item still being laid out (`MenuCursors`): the
            // summary's battle page drew Hold Back a poll before its contest page
            // (or_summary_moves).
            let by = self.cursor_said.get(&text).copied();
            let again = spoken.contains(&text)
                || by.is_some_and(|by| by.1 != item.cursor && self.polls - by.0 <= 1);
            (self.log)(&format!(
                "cursor {:#010x} {}: {text}{}{}  [layout {:#x}]",
                item.cursor,
                if item.opened { "opened" } else { "moved" },
                if held { " (said with its details)" } else { "" },
                if again {
                    " (just said by another cursor)"
                } else {
                    ""
                },
                item.pieces[0].root
            ));
            if !held && !again {
                // Two cursors over one item say it once: a summary's battle page and contest
                // page each keep one, and both said the move (movie32, the audit, 2026-10-10).
                self.speech.say(
                    &text,
                    if item.opened {
                        Category::Prompt
                    } else {
                        Category::Menu
                    },
                );
                spoken.insert(text.clone());
            }
            self.cursor_said
                .insert(text.clone(), (self.polls, item.cursor));
            cursored.extend(item.pieces.iter().map(|p| p.r#box));
            // Only a move brings up details. A cursor coming on screen comes with whatever else
            // the key changed: the bag's cursor came back with the pocket name, and "MEDICINE"
            // was read as the details of the Potion under it. A move drops the details of the
            // item before it unsaid, as the player has moved on, but not the panels they were in.
            if !item.opened {
                if let Some(gathering) = &self.gathering {
                    self.detail_panels
                        .entry(gathering.kept())
                        .or_default()
                        .extend(gathering.panels.iter().copied());
                }
                self.gathering = Some(Gathering {
                    item: text.clone(),
                    menu: item.pieces[0].root,
                    rows: item.rows,
                    ..Gathering::new(item.cursor, item.quiet, count, held)
                });
            }
        }

        // What a cursor is over is its own, said or not this poll: the bag's list label "Potion"
        // settled a poll after its cursor said it and was heard again. Where the item is a count
        // alone, what changes beside a menu's rows names it: the Pokéblock Case writes the chosen
        // block's name under its grid (movie30, spike 196). Not otherwise: the PC Box rewrites its
        // box's name as its menu's cursor moves, "BOX 1" each time (movie9).
        // Words coming to a panel of the menu's layout that words have just left are its own too:
        // the Bag's party draws "ABLE!" or "UNABLE!" for a TM or a stone in place of each level,
        // and the levels again after it, and both were read at every pocket (movie31, Khai,
        // 2026-10-18); a paralysed Pokémon shows no level, and its "UNABLE!" came alone
        // (hm_tm). A question is said. Not a panel no words left: X's language select draws its
        // question in a panel of its own where its list was, "English" in the place of the
        // list's own, and the language went unsaid, "Play Pokémon X in" (x_movie1_opening). The
        // PC's rows, a panel each, are its cursor's (`MenuCursors.rows`).
        let polls = self.polls;
        self.left
            .retain(|_panel, &mut poll| polls - poll <= REPEAT_POLLS);
        for p in left {
            self.left.insert((p.root, p.panel), self.polls);
        }
        let counting = self
            .gathering
            .as_ref()
            .is_some_and(|gathering| !gathering.count.is_empty());
        // Nor a sentence while no cursor has just moved: the Pokéblock Case asks "Give the
        // Pokéblock to Pikachu?" in the words it asked which Pokémon with, under its list, where
        // going back to it rewrote "1", "Gyarados" and the counts beside it (movie33).
        let menu_rows: HashSet<i64> = on_screen
            .iter()
            .filter(|p| rows.contains(&p.r#box))
            .map(|p| p.root)
            .collect();
        let rowed: HashSet<i64> = if counting {
            menu_rows.clone()
        } else {
            HashSet::new()
        };
        let asking: HashSet<i64> = if self.gathering.is_none() {
            menu_rows
        } else {
            HashSet::new()
        };
        let news = |piece: &ScreenText| {
            rowed.contains(&piece.root) || (asking.contains(&piece.root) && sentence(piece))
        };
        // A panel framed in a menu's layout, holding none of what a cursor is over, is the
        // screen's: the battle's move info and the move to forget, each drawn beside the buttons
        // its cursor walks, were never heard (or_move-learning, 2026-10-18).
        let over: HashSet<Panel> = under.iter().flatten().map(ScreenText::group).collect();
        // `self._menus` as it stands from here on: nothing below changes it.
        let menu_roots = self.menus.clone();
        let menus = |piece: &ScreenText| {
            menu_roots.contains(&piece.root)
                && !listed.contains_key(&piece.root)
                && !(piece.frame != 0 && !over.contains(&piece.group()))
        };
        let heard: Vec<ScreenText> = texts
            .iter()
            .filter(|p| {
                !labels.contains(&p.text)
                    && !cursored.contains(&p.r#box)
                    && !(menus(p)
                        && ((p.changed && !news(p))
                            || p.with_layout
                            || p.scrolled
                            || (!p.changed
                                && !sentence(p)
                                && self.left.contains_key(&(p.root, p.panel)))))
            })
            .cloned()
            .collect();
        let timed = self.timed.as_ref().is_some_and(|timed| timed());
        let mut steps: IndexMap<i64, bool> = IndexMap::new();
        for p in &heard {
            if timed
                && p.changed
                && let Some(tick) = self.ticked(p)?
            {
                steps.insert(p.r#box, tick);
            }
        }
        let mut ticks: HashSet<i64> = steps
            .iter()
            .filter(|(_, tick)| **tick)
            .map(|(&r#box, _)| r#box)
            .collect();
        // Kept for two beats, so a clock whose second the reader missed still ticks.
        let polls = self.polls;
        self.set
            .retain(|_box, set_at| polls - set_at.1 < 2 * CLOCK_POLLS.end);
        for p in texts {
            self.set.insert(p.r#box, (p.text.clone(), self.polls));
        }
        let set = &self.set;
        self.clocks.retain(|r#box| set.contains_key(r#box));
        let clocks = &self.clocks;
        self.marks.retain(|r#box, _| clocks.contains(r#box));
        // While the player plays, the numbers are the play's but for the clock's marks, and the
        // signs beside them: at each piece put home Unlimited said its total and the gain it
        // popped up, "Puzzle No. 2, 102 Points", "+2 Points", its "+" a TextBox of its own, and
        // the second it gave back, "Time Remaining: 00:30" (or_amie_tile_puzzle_unlimited).
        // Nothing is said of a swap, the total as a puzzle starts and once it is solved (Orion,
        // 2026-10-28). The last swap's are said: the board lies solved 0.4 s before its total
        // changes, and the results come a second after that.
        let played: HashSet<i64> = heard
            .iter()
            .filter(|p| {
                !steps.contains_key(&p.r#box)
                    && (NUMBERS.is_match(&p.text) || !p.text.chars().any(py::is_alpha))
            })
            .map(|p| p.r#box)
            .collect();
        let mut heard = heard;
        if timed && !played.is_empty() {
            let playing = match &self.playing {
                Some(playing) => playing()?,
                None => false,
            };
            if playing {
                (self.log)(&format!(
                    "screen played, unsaid: {}",
                    heard
                        .iter()
                        .filter(|p| played.contains(&p.r#box))
                        .map(|p| p.text.as_str())
                        .collect::<Vec<&str>>()
                        .join(", ")
                ));
                heard.retain(|p| !played.contains(&p.r#box));
                ticks.extend(played.iter().copied());
            }
        }
        let mut changed: Vec<ScreenText> = heard
            .iter()
            .filter(|p| p.changed && !ticks.contains(&p.r#box))
            .cloned()
            .collect();
        let mut shown: Vec<ScreenText> = heard.iter().filter(|p| !p.changed).cloned().collect();
        let mut said: HashSet<i64> = cursored.union(&ticks).copied().collect();
        let mut gathering = self.gathering.take();
        if gathering
            .as_ref()
            .is_some_and(|g| moving.contains(&g.cursor))
        {
            // The cursor is on its way elsewhere before its details settled: they are no longer
            // what it is over. The PC Box's pointer, held two polls on each slot, read Party 1's
            // summary once it showed Party 2's Pokémon (spike 129).
            if let Some(moved) = gathering.take() {
                self.detail_panels
                    .entry(moved.kept())
                    .or_default()
                    .extend(moved.panels.iter().copied());
            }
        }
        if gathering.is_none() && self.rewrite.is_none() {
            let rewritten = self::rows(&changed, on_screen, labels);
            if rewritten.len() as i64 >= REWRITE_ROWS {
                said.extend(self.start_rewrite(&rewritten, subject)?);
            }
        } else if let Some(rewrite) = &mut self.rewrite {
            // A first row written again before the rest is said is said again with it: the Sell
            // screen's count went on to 3 after 2 was said.
            for p in &changed {
                rewrite.said.remove(&p.r#box);
            }
        }
        // `gathered = gathering or self._rewrite`, taken out of its place while it is worked on
        // and put back where it was if it has not settled.
        let is_gathering = gathering.is_some();
        let gathered = if is_gathering {
            gathering
        } else {
            self.rewrite.take()
        };
        if let Some(mut gathered) = gathered {
            // Words coming on screen meanwhile are part of it: the Mart's count and description
            // coming back as its cursor leaves Quit were two lines, and a contest's move panel,
            // shown afresh for each move, half of one (the audit, 2026-10-10). Not a layout
            // opening, which is a screen of its own, nor words new to a menu, as a question is.
            let joining: Vec<ScreenText> = shown
                .iter()
                .filter(|p| !p.with_layout && !menus(p))
                .cloned()
                .collect();
            gathered
                .panels
                .extend(changed.iter().map(ScreenText::group));
            gathered
                .joined
                .extend(joining.iter().map(ScreenText::group));
            let joined: HashSet<i64> = joining.iter().map(|p| p.r#box).collect();
            shown.retain(|p| !joined.contains(&p.r#box));
            gathered.said.extend(shown.iter().map(|p| p.r#box));
            changed = Vec::new();
            if !self.settled(&gathered, settling) {
                gathered.polls += 1;
                if is_gathering {
                    self.gathering = Some(gathered);
                } else {
                    self.rewrite = Some(gathered);
                }
            } else if is_gathering {
                said.extend(self.say_details(&gathered, on_screen, labels, &cursored));
            } else {
                said.extend(self.say_rewrite(&gathered, on_screen, labels, &cursored));
            }
        }

        // What arrived before a change is said before it: the level up's gains, then its totals.
        let changed_roots: HashSet<i64> =
            texts.iter().filter(|p| p.changed).map(|p| p.root).collect();
        let (moved_on, ready) = self.arrived(&shown, on_screen, settling, &changed_roots);
        // A layout that came with words a poll before a cursor showed on it is a menu all the
        // same: the Bag coming back after a move was learned read its list a row at a time. A
        // sentence is still said: a Secret Base's "You cannot set it there!".
        let [moved_on, ready] = [moved_on, ready].map(|pieces| {
            pieces
                .into_iter()
                .filter(|p| !(p.with_layout && menus(p) && !sentence(p)))
                .collect::<Vec<ScreenText>>()
        });
        // Words arriving now are said with the rest of this poll, not taken for what moved on.
        let coming: HashSet<i64> = ready.iter().map(|p| p.r#box).collect();
        let kept: HashSet<i64> = cursored.union(&coming).copied().collect();
        self.say_shown(&moved_on, on_screen, &mut said, labels, &kept, left);

        // A changed value's row is what was already there: what comes on screen with it is said
        // after, and was said twice when a panel took it in.
        let arriving: HashSet<i64> = shown.iter().map(|p| p.r#box).collect();
        let around: Vec<ScreenText> = on_screen
            .iter()
            .filter(|p| !arriving.contains(&p.r#box))
            .cloned()
            .collect();
        for group in self::rows(&changed, &around, labels) {
            let text = join(&group, self.pauses);
            said.extend(group.iter().map(|p| p.r#box));
            if !self.fresh(&text, group[0].root) {
                (self.log)(&format!(
                    "screen changed, just said elsewhere: {text}  [layout {:#x}]",
                    group[0].root
                ));
                continue;
            }
            (self.log)(&format!(
                "screen changed: {text}  [layout {:#x}]",
                group[0].root
            ));
            self.speech.say(&text, Category::Menu);
        }

        self.say_shown(&ready, on_screen, &mut said, labels, &cursored, left);
        self.say_what_is_left(left, on_screen, &mut said)
    }

    /// Is a changed piece a clock's tick, left unsaid (True), or one at a mark, said
    /// (False)? None for no tick. Its number has gone one up or down a second after the words
    /// were last set, and nothing else of them has changed: "Time Left: 58", "Time: 00:10".
    /// Asked only while a game that times the player runs (`timed`). Once a clock, a step of
    /// two, a second the reader missed, or one coming soon after a late one, is a tick too:
    /// Berry Picker went 41 then 39, and 10 then 9 at 0.4 s. Every tick sounds `tick`; one
    /// reaching or passing a mark is said, but a mark said last, as a second won back bounces
    /// the clock round it: Unlimited said "Time Remaining: 01:00" twice in a second
    /// (or_amie_tile_puzzle_unlimited). A clock changing on no tick passes its marks afresh.
    fn ticked(&mut self, piece: &ScreenText) -> Result<Option<bool>> {
        let Some(before) = self.set.get(&piece.r#box).cloned() else {
            return Ok(None);
        };
        let (now, then) = (seconds(&piece.text)?, seconds(&before.0)?);
        let (Some(now), Some(then)) = (now, then) else {
            return Ok(None);
        };
        if NUMBERS.replace_all(&piece.text, "#") != NUMBERS.replace_all(&before.0, "#") {
            return Ok(None);
        }
        let step = (now - then).abs();
        if !((step == 1 && CLOCK_POLLS.contains(&(self.polls - before.1)))
            || (self.clocks.contains(&piece.r#box) && 0 < step && step <= 2))
        {
            self.marks.shift_remove(&piece.r#box);
            return Ok(None);
        }
        self.clocks.insert(piece.r#box);
        let down = now < then;
        if let Some(tick) = &self.tick {
            tick();
        }
        let passed = if down { now..then } else { then + 1..now + 1 };
        let marks: Vec<i64> = passed.filter(|&seconds| marked(seconds, down)).collect();
        if !marks.is_empty() {
            // `min(marks, key=lambda s: abs(s - now))`: the first of equals.
            let mut mark = marks[0];
            for &seconds in &marks[1..] {
                if (seconds - now).abs() < (mark - now).abs() {
                    mark = seconds;
                }
            }
            if self.marks.get(&piece.r#box) != Some(&mark) {
                self.marks.insert(piece.r#box, mark);
                return Ok(Some(false));
            }
        }
        (self.log)(&format!(
            "screen ticked: {}  [layout {:#x}]",
            piece.text, piece.root
        ));
        Ok(Some(true))
    }

    /// Say the letters left in a name a letter has just been deleted from.
    ///
    /// A name is typed a letter to a TextBox, so deleting one only takes words off the screen,
    /// which says nothing at all. What is left is said instead (the user, 2026-09-18). A group of
    /// one-character words on one row is a name being spelled out; words leaving anywhere else are
    /// not news of their own.
    fn say_what_is_left(
        &mut self,
        left: &[ScreenText],
        on_screen: &[ScreenText],
        said: &mut HashSet<i64>,
    ) -> Result<()> {
        let groups: IndexSet<(i64, i64)> = left
            .iter()
            .map(|piece| (piece.root, piece.parent))
            .collect();
        for group in groups {
            let rest: Vec<ScreenText> = on_screen
                .iter()
                .filter(|piece| (piece.root, piece.parent) == group && !said.contains(&piece.r#box))
                .cloned()
                .collect();
            if rest.len() < 2 || rest.iter().any(|piece| piece.text.chars().count() != 1) {
                continue;
            }
            let mut rows_of: HashSet<i64> = HashSet::new();
            for piece in &rest {
                rows_of.insert(py::round(piece.row)?);
            }
            if rows_of.len() > 1 {
                continue;
            }
            let text = join(&distinct(in_rows(rest.clone())), self.pauses);
            said.extend(rest.iter().map(|piece| piece.r#box));
            (self.log)(&format!("screen left: {text}  [layout {:#x}]", group.0));
            self.speech.say(&text, Category::Screen);
        }
        Ok(())
    }

    /// Say what came on screen: one utterance per panel, queued in build order, rather than one
    /// run for the whole poll. Two panels are two things, and the battle's HP box proved it: the
    /// player's level and HP and the wild Pokemon's level ran together as "6 22/ 22 2". `left`
    /// is what left the screen this poll (`_with_what_stayed`).
    fn say_shown(
        &mut self,
        shown: &[ScreenText],
        on_screen: &[ScreenText],
        said: &mut HashSet<i64>,
        labels: &HashSet<String>,
        kept: &HashSet<i64>,
        left: &[ScreenText],
    ) {
        if shown.is_empty() {
            return;
        }
        let shown = with_row_mates(shown, on_screen, kept, labels);
        let stayed = self.with_what_stayed(&shown, on_screen, said, labels, left);
        for pieces in panels(&stayed, on_screen) {
            if pieces.is_empty() {
                continue;
            }
            let text = join(&pieces, self.pauses);
            let root = pieces[0].root;
            said.extend(pieces.iter().map(|p| p.r#box));
            self.words_said
                .extend(pieces.iter().map(|p| (p.r#box, (p.root, p.text.clone()))));
            // Nor the item a cursor has just said: the Bag's panel names it as it opens. A letter
            // typed is said, though its key was (`letter`).
            if !self.fresh(&text, root)
                || (self.said_by_cursor(&text) && !pieces.iter().all(letter))
            {
                (self.log)(&format!(
                    "screen shown, just said elsewhere: {text}  [layout {root:#x}]"
                ));
                continue;
            }
            (self.log)(&format!("screen shown: {text}  [layout {root:#x}]"));
            self.speech.say(&text, Category::Screen);
        }
    }

    /// What came on screen and is ready to say, as (what its layout has moved on from, the
    /// rest): the words of layouts no longer being written, with what came before them, or that
    /// have waited their most polls.
    ///
    /// The level up reveals its rows over half a second and writes each value a poll after its
    /// label, and said a row and a half, then a label and values out of step, then the rest: "HP",
    /// "+ 2 ATTACK + 0 DEFENSE SP. ATK SP. DEF", "+ 1 + 1 SPEED + 1" (battles2.ctm, the user,
    /// 2026-09-17). A layout whose words change or leave has moved on, and what arrived is said
    /// at once, as it was: the gains were on screen for under a second before the totals replaced
    /// them, and waiting on, then dropping what had changed, lost them all. `changed` is the roots
    /// of the layouts with words changed this poll.
    fn arrived(
        &mut self,
        shown: &[ScreenText],
        on_screen: &[ScreenText],
        settling: &IndexSet<i64>,
        changed: &HashSet<i64>,
    ) -> (Vec<ScreenText>, Vec<ScreenText>) {
        let mut since: IndexMap<i64, i64> = IndexMap::new();
        for piece in on_screen {
            let at = self
                .on_screen_since
                .get(&piece.root)
                .copied()
                .unwrap_or(self.polls);
            since.insert(piece.root, at);
        }
        self.on_screen_since = since;
        let mut now: IndexMap<i64, &str> = IndexMap::new();
        for piece in on_screen {
            now.insert(piece.r#box, piece.text.as_str());
        }
        let mut moved_on: Vec<ScreenText> = Vec::new();
        let mut ready: Vec<ScreenText> = Vec::new();
        for piece in shown {
            let opening = self.polls
                - self
                    .on_screen_since
                    .get(&piece.root)
                    .copied()
                    .unwrap_or(self.polls);
            if self.arriving.contains_key(&piece.root) || opening < ARRIVAL_MAX_POLLS {
                self.arriving
                    .entry(piece.root)
                    .or_default()
                    .push((self.polls, piece.clone()));
            } else {
                // A screen already up for a while is not being revealed: a letter typed into a
                // name is said as it is typed, where holding it said "w e e" for two quick
                // letters.
                ready.push(piece.clone());
            }
        }
        let roots: Vec<i64> = self.arriving.keys().copied().collect();
        for root in roots {
            let waiting = &self.arriving[&root];
            let since = waiting[0].0;
            let gone = changed.contains(&root)
                || waiting
                    .iter()
                    .any(|(_poll, p)| now.get(&p.r#box).copied() != Some(p.text.as_str()));
            if !gone && settling.contains(&root) && self.polls - since < ARRIVAL_MAX_POLLS - 1 {
                continue;
            }
            let waiting = self.arriving.shift_remove(&root).unwrap_or_default();
            for (poll, piece) in waiting {
                // Words arriving with the change are part of what it brought, and follow it.
                if gone && poll < self.polls {
                    moved_on.push(piece);
                } else {
                    ready.push(piece);
                }
            }
        }
        (
            py::sorted_by_lt(moved_on, |a, b| reading_order(a) < reading_order(b)),
            py::sorted_by_lt(ready, |a, b| reading_order(a) < reading_order(b)),
        )
    }

    /// Is this not what another layout said a moment ago? Remembers it when it is not.
    ///
    /// Walking onto Route 101 said it twice: AreaNav's words changed on the bottom screen, and the
    /// location sign came up on the top with the same words a poll later (the user, 2026-09-14).
    /// The same layout saying the same words again, as a description does when the cursor comes
    /// back to a move, is heard again.
    fn fresh(&mut self, text: &str, root: i64) -> bool {
        if let Some((placed, at)) = &self.placed
            && root != FIELD_ROOT
            && text == placed
            && self.polls - at <= PLACE_POLLS
        {
            return false;
        }
        if let Some(&(poll, where_)) = self.recent.get(text)
            && where_ != root
            && self.polls - poll <= REPEAT_POLLS
        {
            return false;
        }
        self.recent = self
            .recent
            .iter()
            .filter(|&(_words, &(poll, _where))| self.polls - poll <= REPEAT_POLLS)
            .map(|(words, &seen)| (words.clone(), seen))
            .collect();
        self.recent.insert(text.to_string(), (self.polls, root));
        true
    }

    /// Have the details settled: waited their polls, with nothing in them still being written?
    ///
    /// A move's description was written a poll after its panel, and was read after the panel
    /// where it was read before it every other time. Before any panel is known, any words still
    /// being written anywhere may be the details, so they are waited for too.
    fn settled(&self, gathering: &Gathering, settling: &IndexSet<i64>) -> bool {
        if gathering.polls >= DETAIL_MAX_POLLS {
            return true;
        }
        if gathering.polls < DETAIL_POLLS {
            return false;
        }
        let mut panels: IndexSet<Panel> = IndexSet::new();
        if let Some(known) = self.detail_panels.get(&gathering.kept()) {
            panels.extend(known.iter().copied());
        }
        panels.extend(gathering.panels.iter().copied());
        panels.extend(gathering.joined.iter().copied());
        let roots: HashSet<i64> = panels.iter().map(|&(root, _parent)| root).collect();
        if roots.is_empty() {
            settling.is_empty()
        } else {
            !roots.iter().any(|root| settling.contains(root))
        }
    }

    /// Say every panel the cursor's moves have changed, in the order the screen built them,
    /// each row by row, as one line that never interrupts. Returns the TextBoxes said.
    ///
    /// A line a panel, said as menus are, cut off the item they belong to under `interrupt`, and
    /// each other; the next item still cuts them off (Khai and Orion, 2026-10-05).
    fn say_details(
        &mut self,
        gathering: &Gathering,
        on_screen: &[ScreenText],
        labels: &HashSet<String>,
        cursored: &HashSet<i64>,
    ) -> HashSet<i64> {
        self.gathering = None;
        let kept = self.detail_panels.entry(gathering.kept()).or_default();
        kept.extend(gathering.panels.iter().copied());
        let mut panels = kept.clone();
        panels.extend(gathering.joined.iter().copied());
        // Words hung under a label are of the label's panel: the Bag's TM pocket hangs each number
        // under its own, and a number this move left as it was went unsaid, "TYPE Fighting POWER
        // CATEGORY STATUS ACCURACY PP" (or_tm-hm, 2026-10-18).
        let hung: Vec<Panel> = on_screen
            .iter()
            .filter(|p| panels.contains(&p.group()))
            .map(|p| (p.root, p.r#box))
            .collect();
        panels.extend(hung);
        // Only the layouts this move changed: a summary keeps its other page's panel on screen,
        // left as that page last showed it, and Zen Headbutt's details on the contest page were
        // read with Iron Defense's from the battle page (movie32).
        let changed: HashSet<i64> = gathering
            .panels
            .iter()
            .chain(gathering.joined.iter())
            .map(|&(root, _group)| root)
            .collect();
        let mut said: HashSet<i64> = HashSet::new();
        let mut texts: Vec<String> = Vec::new();
        let mut roots: Vec<i64> = Vec::new();
        // A group drawn wholly beside a column of two rows or more is read with it as one table,
        // row by row: the Bag's TM pocket hangs each of a move's numbers from a pane of its own,
        // and they were said after their labels, "TYPE Normal POWER CATEGORY SPECIAL ACCURACY
        // PP, 15, 100, 40" (movie30). Not beside a lone row: the PC Box's name and level, read as
        // one, put the level first, drawn a little higher (movie6). Nor a group of its own rows:
        // a summary's move panel, one row of it beside the contest page's labels, was read in
        // among them (movie7).
        let mut tables: Vec<Vec<ScreenText>> = Vec::new();
        let built: IndexSet<Panel> = on_screen.iter().map(ScreenText::group).collect();
        for panel in built {
            if !panels.contains(&panel) || (!changed.is_empty() && !changed.contains(&panel.0)) {
                continue;
            }
            // Nor the item the cursor said, nor a title still saying what was last said of it:
            // the Bag's pocket and item were the first words of every item's details,
            // "MEDICINE Full Heal", then the count and the description (Khai, 2026-10-18). A
            // name's letter is never the item, though its key was the cursor's (`letter`), nor
            // a label's value (`valued`).
            let pieces: Vec<ScreenText> = on_screen
                .iter()
                .filter(|p| {
                    p.group() == panel
                        && !labels.contains(&p.text)
                        && !gathering.said.contains(&p.r#box)
                        && !cursored.contains(&p.r#box)
                        && (p.text != gathering.item || letter(p) || valued(p, on_screen))
                        && !self.heading(p, on_screen)
                })
                .map(|p| without(p, &gathering.item))
                .collect();
            let beside = tables.iter_mut().find(|table| {
                !table.is_empty()
                    && !pieces.is_empty()
                    && table[0].root == pieces[0].root
                    && row_count(table) >= 2
                    && pieces
                        .iter()
                        .filter(|b| b.glyph > 0.0)
                        .all(|b| table.iter().any(|a| side_by_side(a, b)))
            });
            match beside {
                Some(table) => table.extend(pieces),
                None => tables.push(pieces),
            }
        }
        for table in tables {
            let groups: IndexSet<(i64, i64)> = table.iter().map(|p| p.group()).collect();
            let ordered = if groups.len() > 1 {
                by_rows(table)
            } else {
                columns(&table)
            };
            let group = under_their_labels(distinct(ordered));
            if !group.is_empty() {
                let mut text = join(&group, self.pauses);
                if gathering.held && !gathering.count.is_empty() && texts.is_empty() {
                    let first: Vec<ScreenText> = group
                        .iter()
                        .filter(|p| (p.row - group[0].row).abs() < group[0].glyph / 2.0)
                        .cloned()
                        .collect();
                    let rest = py::slice_from(&group, first.len() as i64).to_vec();
                    text = format!("{} {}", join(&first, self.pauses), gathering.count);
                    if !rest.is_empty() {
                        text = line(&[text, join(&rest, self.pauses)]);
                    }
                }
                if gathering.quiet {
                    (self.log)(&format!(
                        "details, unsaid: {text}  [layout {:#x}]",
                        group[0].root
                    ));
                } else {
                    texts.push(text);
                    roots.push(group[0].root);
                    self.words_said
                        .extend(group.iter().map(|p| (p.r#box, (p.root, p.text.clone()))));
                }
                said.extend(group.iter().map(|p| p.r#box));
            }
        }
        if texts.is_empty() && !gathering.count.is_empty() {
            // Nothing names it: the count is said alone, late rather than never.
            (texts, roots) = (vec![gathering.count.clone()], vec![0]);
        }
        if !texts.is_empty() {
            let line = line(&texts);
            let r#where = roots
                .iter()
                .collect::<IndexSet<_>>()
                .iter()
                .map(|root| format!("{root:#x}"))
                .collect::<Vec<_>>()
                .join(", ");
            (self.log)(&format!("details: {line}  [layout {where}]"));
            self.speech.say(&line, Category::Screen);
        }
        said
    }

    /// Is this a title on a row of its own whose words were said and have not changed since?
    /// A label alone on its row, a move's "Category" beside no icon, is still said.
    fn heading(&self, piece: &ScreenText, on_screen: &[ScreenText]) -> bool {
        piece.plain
            && self.words_said.get(&piece.r#box) == Some(&(piece.root, piece.text.clone()))
            && !on_screen.iter().any(|other| {
                other.r#box != piece.r#box && other.root == piece.root && side_by_side(other, piece)
            })
    }

    /// Say the first row of a screen rewritten at once, as a cursor's item is, and gather the
    /// rest as its details. Returns the TextBoxes said.
    ///
    /// The first row is the one naming the `subject`, its highest where two do, or else the
    /// first in reading order that is no sentence: the summary's stats are on the top screen
    /// and built first, its Pokémon's name on the bottom (Orion, 2026-10-10: the name first,
    /// where it costs no wait; it is written in the same poll as the rest), and the Pokédex's
    /// other languages put the entry's description before its number and name (movie7).
    fn start_rewrite(
        &mut self,
        rows: &[Vec<ScreenText>],
        subject: Option<Subject>,
    ) -> Result<HashSet<i64>> {
        let name = match subject {
            Some(subject) => subject()?,
            None => None,
        };
        let mut named: Vec<(f64, &Vec<ScreenText>)> = Vec::new();
        for row in rows {
            for p in row {
                if let Some(name) = &name
                    && !name.is_empty()
                    && py::strip(&p.text) == *name
                {
                    named.push((p.row, row));
                }
            }
        }
        // `max(named, key=row)`: the first of equals, and a NaN never larger.
        let first: &Vec<ScreenText> = match named.first() {
            Some(&(mut highest, mut row)) => {
                for &(at, other) in &named[1..] {
                    if at > highest {
                        (highest, row) = (at, other);
                    }
                }
                row
            }
            None => rows
                .iter()
                .find(|row| !row.iter().any(sentence))
                .unwrap_or(&rows[0]),
        };
        let text = join(first, self.pauses);
        let root = first[0].root;
        let mut rewrite = Gathering::new(0, false, String::new(), false);
        rewrite.lead = (root, text.clone());
        rewrite.said.extend(first.iter().map(|p| p.r#box));
        self.rewrite = Some(rewrite);
        if self.fresh(&text, root) {
            (self.log)(&format!("screen rewritten: {text}  [layout {root:#x}]"));
            self.speech.say(&text, Category::Menu);
        } else {
            (self.log)(&format!(
                "screen rewritten, just said elsewhere: {text}  [layout {root:#x}]"
            ));
        }
        Ok(first.iter().map(|p| p.r#box).collect())
    }

    /// Say the rest of a screen rewritten once it has settled, as a cursor's details are: every
    /// panel its rewrites have changed in the layouts this one changed, whole, as a screen
    /// opening is read, as one line that never interrupts, its first row's layout first. Returns
    /// the TextBoxes said.
    ///
    /// Words two layouts draw alike are said once: the summary keeps its other pages on screen,
    /// "Name Poochyena" on two of them.
    fn say_rewrite(
        &mut self,
        rewrite: &Gathering,
        on_screen: &[ScreenText],
        labels: &HashSet<String>,
        cursored: &HashSet<i64>,
    ) -> HashSet<i64> {
        self.rewrite = None;
        let kept = self.detail_panels.entry((0, 0, 0)).or_default();
        kept.extend(rewrite.panels.iter().copied());
        let mut panels = kept.clone();
        panels.extend(rewrite.joined.iter().copied());
        let roots: HashSet<i64> = rewrite
            .panels
            .iter()
            .chain(rewrite.joined.iter())
            .map(|&(root, _group)| root)
            .collect();
        let pieces: Vec<ScreenText> = on_screen
            .iter()
            .filter(|p| {
                panels.contains(&p.group())
                    && roots.contains(&p.root)
                    && !labels.contains(&p.text)
                    && !cursored.contains(&p.r#box)
                    && !rewrite.said.contains(&p.r#box)
            })
            .cloned()
            .collect();
        let (lead, first) = &rewrite.lead;
        let mut parts: IndexMap<String, i64> = IndexMap::new();
        for group in self::panels(&pieces, on_screen) {
            let text = join(&group, self.pauses);
            // Nor what a cursor has just said: the Bag's, back on Exp. Share as the pocket
            // changed, and the panel's own "Exp. Share" after it (movie30).
            if text != *first && !self.said_by_cursor(&text) {
                parts.entry(text).or_insert(group[0].root);
            }
        }
        // `sorted(parts, key=lambda text: parts[text] != lead)`: the lead's first, each side in
        // the order it came, as Python's sort is stable.
        let texts: Vec<String> = parts
            .iter()
            .filter(|&(_text, root)| root == lead)
            .chain(parts.iter().filter(|&(_text, root)| root != lead))
            .map(|(text, _root)| text.clone())
            .collect();
        if !texts.is_empty() {
            let line = line(&texts);
            let r#where = parts
                .values()
                .collect::<IndexSet<_>>()
                .iter()
                .map(|root| format!("{root:#x}"))
                .collect::<Vec<_>>()
                .join(", ");
            (self.log)(&format!("screen rewritten, then: {line}  [layout {where}]"));
            self.speech.say(&line, Category::Screen);
        }
        pieces.iter().map(|p| p.r#box).collect()
    }

    /// What came on screen, with the rest of each layout that some of it came back to.
    ///
    /// Returning from the move list brought back only the summary's panel that had faded, "OT wee
    /// ID No. …", where opening the summary read its stats and name as well: the move list keeps
    /// those on screen. A screen coming back is read as it opened (the user, 2026-09-14), so words
    /// coming back to a layout bring what stayed in it. A menu's layout is left to its cursor, and
    /// words new to a layout come alone, as a question does. So do words that came back where
    /// words of their panel were leaving, `left`, and what comes with them: the screen changed
    /// what it shows there, and did not come back. Each of Tile Puzzle's difficulty buttons, one
    /// panel slid or swapped in for the last, read "Select a difficulty level." over it twice
    /// (or_amie_tile_puzzle_play_again). Not words of another panel: the summary's "OT" panel
    /// comes back where the move list leaves, and brings its stats and name (or_menus). A screen
    /// coming back by sliding in is still one coming
    /// back: the decorate screen's counts slide back up as it returns, and bring its wallpaper's
    /// name and "Interior Items Poké Puffs" (or_amie_interior_items). And while a game that times
    /// the player runs every word comes alone, as its pop-ups come back where they were:
    /// Unlimited's "+2 Points", popped up at each piece put home, read its instructions and its
    /// clock again each time (or_amie_tile_puzzle_unlimited).
    fn with_what_stayed(
        &self,
        shown: &[ScreenText],
        on_screen: &[ScreenText],
        said: &HashSet<i64>,
        labels: &HashSet<String>,
        left: &[ScreenText],
    ) -> Vec<ScreenText> {
        if self.timed.as_ref().is_some_and(|timed| timed()) {
            return shown.to_vec();
        }
        let swapped: HashSet<i64> = shown
            .iter()
            .filter(|p| left.iter().any(|gone| p.over(gone)))
            .map(|p| p.root)
            .collect();
        let back: HashSet<i64> = shown
            .iter()
            .filter(|p| p.returned && !self.menus.contains(&p.root) && !swapped.contains(&p.root))
            .map(|p| p.root)
            .collect();
        if back.is_empty() {
            return shown.to_vec();
        }
        let boxes: HashSet<i64> = shown.iter().map(|p| p.r#box).collect();
        let stayed = on_screen.iter().filter(|p| {
            back.contains(&p.root)
                && !boxes.contains(&p.r#box)
                && !said.contains(&p.r#box)
                && !labels.contains(&p.text)
        });
        let all: Vec<ScreenText> = shown.iter().chain(stayed).cloned().collect();
        py::sorted_by_lt(all, |a, b| (!a.top, a.r#box) < (!b.top, b.r#box))
    }
}

/// Panels said as one line, a comma between two where the first ends no sentence, for the pause
/// a line of its own had: "In Bag: 9, A spray-type medicine for treating wounds.".
fn line(parts: &[String]) -> String {
    let mut out = String::new();
    for part in parts {
        if let Some(last) = out.chars().last() {
            out += if ".!?…".contains(last) { " " } else { ", " };
        }
        out += part;
    }
    out
}

/// The piece less the item its cursor has just said, where its words end with it: the TM
/// pocket names "TM04 Calm Mind" under the cursor's "Calm Mind".
fn without(piece: &ScreenText, item: &str) -> ScreenText {
    let suffix = format!(" {item}");
    match piece.text.strip_suffix(&suffix) {
        Some(text) if !item.is_empty() => ScreenText {
            text: text.to_string(),
            ..piece.clone()
        },
        _ => piece.clone(),
    }
}

/// Pieces read row by row, each row's first moved to just after the words above it where it
/// is drawn under them, starting where they end with nothing else of their row over it: the
/// battle's move panel writes PHYSICAL under CATEGORY, and POWER 80 on CATEGORY's row, "CATEGORY
/// POWER 80 PHYSICAL" (or_move-learning, 2026-10-18).
fn under_their_labels(pieces: Vec<ScreenText>) -> Vec<ScreenText> {
    let mut rows: Vec<Vec<ScreenText>> = Vec::new();
    for piece in pieces {
        match rows.last_mut() {
            Some(row) if side_by_side(&row[0], &piece) => row.push(piece),
            _ => rows.push(vec![piece]),
        }
    }
    for at in 1..rows.len() {
        let (before, after) = rows.split_at_mut(at);
        let (above, row) = (&mut before[at - 1], &mut after[0]);
        let Some(first) = row.first().cloned() else {
            continue;
        };
        if first.glyph <= 0.0 {
            continue;
        }
        let label = above.iter().rposition(|p| p.right <= first.left + p.glyph);
        let Some(label) = label else {
            continue;
        };
        if label == above.len() - 1
            || above
                .iter()
                .any(|p| p.left < first.right && p.right > first.left)
        {
            continue;
        }
        row.remove(0);
        above.insert(label + 1, first);
    }
    rows.into_iter().flatten().collect()
}

/// A group's placed pieces row by row, as a table, unless it is tables side by side: each
/// column it built, higher up and to the right of the one before, is read with that one row by
/// row where every row of it is one of that one's, and otherwise after it, as it was built, as
/// a screen opening reads it (`_speaking_order`). A move's panel builds its labels and then its
/// values, "Category Power 40"; the PC Box's summary builds its stats and then its moves, between
/// their rows, and read row by row they came interleaved, "SP. ATK: 68, MOVES LEARNED, SP. DEF:
/// 58, Bubble Beam", where opening it read each down (Orion, or_organize_boxes, 2026-10-24).
fn columns(pieces: &[ScreenText]) -> Vec<ScreenText> {
    let mut columns: Vec<Vec<ScreenText>> = Vec::new();
    for piece in pieces.iter().filter(|p| p.glyph > 0.0) {
        let starts = match columns.last() {
            None => true,
            Some(last) => {
                // Python's `max` over floats: the first unless a later one is strictly greater.
                let mut right = last[0].right;
                for p in &last[1..] {
                    if p.right > right {
                        right = p.right;
                    }
                }
                piece.row - last[last.len() - 1].row >= piece.glyph / 2.0 && piece.left >= right
            }
        };
        if starts {
            columns.push(vec![piece.clone()]);
        } else {
            columns
                .last_mut()
                .expect("a column to add to")
                .push(piece.clone());
        }
    }
    let mut tables: Vec<Vec<Vec<ScreenText>>> = Vec::new();
    for column in columns {
        let beside = tables.last().is_some_and(|table: &Vec<Vec<ScreenText>>| {
            let before = &table[table.len() - 1];
            column
                .iter()
                .all(|b| before.iter().any(|a| side_by_side(a, b)))
        });
        if beside {
            tables.last_mut().expect("checked above").push(column);
        } else {
            tables.push(vec![column]);
        }
    }
    if tables.len() < 2 {
        return by_rows(pieces.to_vec());
    }
    let mut out: Vec<ScreenText> = Vec::new();
    for table in tables {
        if table.len() > 1 {
            out.extend(by_rows(table.into_iter().flatten()));
        } else {
            out.extend(in_rows(table.into_iter().next().expect("one column")));
        }
    }
    out
}

/// Python's `min(a, b)` on floats: `a`, unless `b < a`, so a NaN stays where Python puts it.
fn min_of(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// What came on screen, with the label already there beside a value that came alone. `kept` is
/// what is not to be taken for a label: a cursor's item, and whatever else is coming this poll.
///
/// A value can arrive after its label was said: the level up's "SP. DEF" and "SPEED" were said
/// with the gains, and their totals arrived alone as "12 12" (battles2.ctm replay, 2026-09-17).
/// Each of its rows is a group of a label and a value set well apart, so a piece arriving in a
/// group where one other piece shares its row, at least `LABEL_GAP` glyphs away, is said with that
/// one. Words arriving in a group of their own, as a move's description does, still come alone,
/// and so does a letter typed into a name beside the letters before it: "R a i d" for the fourth
/// letter was the name again each time (opening.ctm replay).
fn with_row_mates(
    shown: &[ScreenText],
    on_screen: &[ScreenText],
    kept: &HashSet<i64>,
    labels: &HashSet<String>,
) -> Vec<ScreenText> {
    let boxes: HashSet<i64> = shown.iter().map(|piece| piece.r#box).collect();
    // Each mate with its place in `on_screen`: `label not in mates` finds the very object it was
    // given before it tries `==`, which a NaN in the object never equals.
    let mut mates: Vec<(usize, &ScreenText)> = Vec::new();
    for piece in shown {
        if piece.glyph <= 0.0 {
            continue;
        }
        let beside: Vec<(usize, &ScreenText)> = on_screen
            .iter()
            .enumerate()
            .filter(|(_at, other)| {
                !boxes.contains(&other.r#box)
                    && other.root == piece.root
                    && other.parent == piece.parent
                    && other.glyph > 0.0
                    && (other.row - piece.row).abs() < min_of(other.glyph, piece.glyph) / 2.0
            })
            .collect();
        if beside.len() != 1 {
            continue;
        }
        let (at, label) = beside[0];
        let apart = LABEL_GAP * piece.glyph;
        if !(label.right <= piece.left - apart || label.left >= piece.right + apart) {
            continue;
        }
        if !kept.contains(&label.r#box)
            && !labels.contains(&label.text)
            && !mates.iter().any(|&(was, mate)| was == at || mate == label)
        {
            mates.push((at, label));
        }
    }
    if mates.is_empty() {
        return shown.to_vec();
    }
    let all: Vec<ScreenText> = shown
        .iter()
        .cloned()
        .chain(mates.into_iter().map(|(_at, label)| label.clone()))
        .collect();
    py::sorted_by_lt(all, |a, b| reading_order(a) < reading_order(b))
}

/// The pieces a panel at a time, in the order the layouts built them, runs on a row read left to
/// right: build order put the battle's "35", "35" and "/" side by side and read "PP 3535 /"
/// (spike 46), and reading rows across a whole layout interleaved the summary's columns (spike
/// 47).
///
/// A panel, not a whole layout (the user, 2026-09-16). The battle draws both Pokemon's HP boxes in
/// one layout, and reading the layout whole said "6 22/ 22 2" — Torchic at level 6 with 22/22 HP,
/// and the wild Pokemon's level 2 on the end as though it were part of the same row. The game
/// itself keeps the two apart, one pane under the root each, and so does this.
///
/// A panel boundary only breaks an utterance where a row boundary does too. Splitting on the panel
/// alone cut labels off their values: the level up read "ATTACK" and "13" as two things, and a
/// move's "PP 25/" and "25" likewise, because a screen can draw a label and its number in
/// different panels while putting them side by side on one row. What the ear wants is the row.
///
/// A panel that is one column is read top to bottom (`speaking_order`).
///
/// Python's `on_screen` defaults to nothing.
fn panels(pieces: &[ScreenText], on_screen: &[ScreenText]) -> Vec<Vec<ScreenText>> {
    let mut layouts: IndexMap<i64, Vec<ScreenText>> = IndexMap::new();
    for piece in pieces {
        layouts.entry(piece.root).or_default().push(piece.clone());
    }
    let mut out: Vec<Vec<ScreenText>> = Vec::new();
    for group in layouts.values() {
        let mut current: Vec<ScreenText> = Vec::new();
        for piece in distinct(speaking_order(group, on_screen)) {
            if let Some(last) = current.last()
                && piece.panel != last.panel
                && !side_by_side(last, &piece)
            {
                out.push(std::mem::take(&mut current));
            }
            current.push(piece);
        }
        if !current.is_empty() {
            out.push(current);
        }
    }
    out
}

/// The pieces of one layout in the order to say them: each panel that is one column top to
/// bottom, and the rest in the order the layout built them, each run on a row left to right.
///
/// The Pokédex's entry is one panel built out of order: the species and gender, then a table of
/// TYPE, HEIGHT and WEIGHT built as a column of labels and a column of values, then the times
/// encountered, and the number and name at the top built last. In build order it read "Chick
/// Pokémon Female TYPE HEIGHT WEIGHT 1'04" 5.5 lbs. Times encountered: 1 004 Torchic"
/// (pokedex2.ctm, the user, 2026-09-24). A panel with no words from two groups side by side on
/// a row is one column, and read down it is "004 Torchic Chick Pokémon Female TYPE HEIGHT 1'04"
/// WEIGHT 5.5 lbs. Times encountered: 1". A panel whose groups stand side by side, as the
/// summary's two columns of stats do (spike 47), or whose words sit beside another panel's, as
/// the level up's labels do beside their values, keeps its build order.
fn speaking_order(pieces: &[ScreenText], on_screen: &[ScreenText]) -> Vec<ScreenText> {
    let everything = if on_screen.is_empty() {
        pieces
    } else {
        on_screen
    };
    let panels: IndexSet<i64> = pieces.iter().map(|piece| piece.panel).collect();
    let mut column: HashSet<i64> = HashSet::new();
    for &panel in &panels {
        let layout: Vec<&ScreenText> = everything
            .iter()
            .filter(|p| p.root == pieces[0].root)
            .collect();
        if one_column(&layout, panel) {
            column.insert(panel);
        }
    }
    if column.is_empty() {
        return in_rows(pieces.iter().cloned());
    }
    let mut ordered: Vec<ScreenText> = Vec::new();
    for &panel in &panels {
        let mine: Vec<ScreenText> = pieces
            .iter()
            .filter(|piece| piece.panel == panel)
            .cloned()
            .collect();
        if column.contains(&panel) {
            ordered.extend(by_rows(mine.iter().cloned()));
            ordered.extend(mine.iter().filter(|piece| piece.glyph <= 0.0).cloned());
        } else {
            ordered.extend(in_rows(mine));
        }
    }
    ordered
}

/// Is this panel one column: no word of it beside a word of another group on its row, in the
/// whole layout, and no two of its words beside each other without sharing a line? A table's
/// label and value are drawn on one line, the entry's HEIGHT and its value both at 10; columns
/// side by side only come near, the summary's Dex No. and HP five units apart. A panel with
/// fewer than two placed is not: a lone piece is no column, and the summary, which hangs every
/// piece from its root as a panel of its own, was read piece by piece in build order, "HP / 30
/// 30", once its "/" was heard (movie7, 2026-09-30).
fn one_column(layout: &[&ScreenText], panel: i64) -> bool {
    let mine: Vec<&ScreenText> = layout
        .iter()
        .copied()
        .filter(|piece| piece.panel == panel && piece.glyph > 0.0)
        .collect();
    if mine.len() < 2 {
        return false;
    }
    if mine.iter().any(|piece| {
        layout.iter().any(|other| {
            other.glyph > 0.0 && other.parent != piece.parent && side_by_side(piece, other)
        })
    }) {
        return false;
    }
    !mine.iter().any(|piece| {
        mine.iter()
            .any(|other| side_by_side(piece, other) && (piece.row - other.row).abs() > ONE_LINE)
    })
}

/// On the same row, so one thing to say however the layout groups them?
/// How many rows the pieces are drawn on.
fn row_count(pieces: &[ScreenText]) -> usize {
    let mut rows: Vec<&ScreenText> = Vec::new();
    for piece in pieces {
        if piece.glyph > 0.0 && !rows.iter().any(|row| side_by_side(row, piece)) {
            rows.push(piece);
        }
    }
    rows.len()
}

fn side_by_side(one: &ScreenText, other: &ScreenText) -> bool {
    if one.glyph <= 0.0 || other.glyph <= 0.0 {
        return false;
    }
    (one.row - other.row).abs() < min_of(one.glyph, other.glyph) / 2.0
}

/// Is this a label's value, written after the label on its row? Never the cursor's item,
/// whatever its words: TM29 Psychic's type is "Psychic", and its details said "TYPE POWER 90"
/// (or_movie54, Orion, 2026-10-19, #13).
fn valued(piece: &ScreenText, on_screen: &[ScreenText]) -> bool {
    on_screen.iter().any(|other| {
        other.root == piece.root
            && other.r#box != piece.r#box
            && other.left < piece.left
            && side_by_side(other, piece)
            && (other.label || labels_value(other, piece))
    })
}

/// Each changed piece with what sits beside it in its group on its row, each row once.
/// The fewest words a sentence has, a question or a message: more than a label's
/// (`LABEL_WORDS`) or an item's.
pub const SENTENCE_WORDS: usize = 4;

/// Is this a sentence rather than a label, a value or an item's name?
fn sentence(piece: &ScreenText) -> bool {
    py::split(&piece.text).len() >= SENTENCE_WORDS
}

/// Is this one letter of words being spelled, as a name is typed a letter a TextBox
/// (`say_what_is_left`)? Never words a cursor said again: the keyboard's cursor says the key a
/// letter is typed with, and the letter went unsaid, alone and in the name (#12).
fn letter(piece: &ScreenText) -> bool {
    piece.text.chars().count() == 1
}

fn rows(
    changed: &[ScreenText],
    on_screen: &[ScreenText],
    labels: &HashSet<String>,
) -> Vec<Vec<ScreenText>> {
    let mut rows: Vec<Vec<ScreenText>> = Vec::new();
    let mut by_layout: IndexMap<i64, Vec<ScreenText>> = IndexMap::new();
    for piece in changed {
        by_layout.entry(piece.root).or_default().push(piece.clone());
    }
    let ordered: Vec<ScreenText> = by_layout
        .values()
        .flat_map(|group| speaking_order(group, on_screen))
        .collect();
    for piece in ordered {
        let mut row = vec![piece.clone()];
        // A sentence is no value of a row: "Give the Pokéblock to Pikachu?" was said with the
        // tabs beside it, "Give a Pokéblock" and "Make" (movie33).
        if piece.glyph > 0.0 && !sentence(&piece) {
            let beside: Vec<ScreenText> = on_screen
                .iter()
                .filter(|other| {
                    other.root == piece.root
                        && other.glyph > 0.0
                        && !labels.contains(&other.text)
                        && (other.row - piece.row).abs() < min_of(other.glyph, piece.glyph) / 2.0
                })
                .cloned()
                .collect();
            let group: Vec<ScreenText> = beside
                .iter()
                .filter(|other| other.parent == piece.parent)
                .cloned()
                .collect();
            let found = distinct(in_rows(if group.len() > 1 { group } else { beside }));
            if !found.is_empty() {
                row = found;
            }
        }
        rows.push(row);
    }
    once(rows)
}

/// The groups, each set of pieces said once however many of its pieces changed.
fn once(groups: Vec<Vec<ScreenText>>) -> Vec<Vec<ScreenText>> {
    let mut out: Vec<Vec<ScreenText>> = Vec::new();
    let mut seen: HashSet<Vec<i64>> = HashSet::new();
    for group in groups {
        let key: Vec<i64> = group.iter().map(|p| p.r#box).collect();
        if seen.insert(key) {
            out.push(group);
        }
    }
    out
}

/// The seconds a clock's words show, "00:10" as minutes and seconds; None for words with no
/// number, or more than a clock's two.
fn seconds(text: &str) -> Result<Option<i64>> {
    let numbers: Vec<&str> = NUMBERS.find_iter(text).map(|m| m.as_str()).collect();
    if !(!numbers.is_empty() && numbers.len() <= 2) {
        return Ok(None);
    }
    let mut total = 0;
    for number in numbers {
        total = total * 60 + py::int_base10(number)?;
    }
    Ok(Some(total))
}

/// Is a clock's tick said? Each half minute, and counting down, at 10 and each of the last 5
/// (Orion, 2026-10-26).
fn marked(seconds: i64, down: bool) -> bool {
    (seconds > 0 && py::modulo(seconds, 30) == 0)
        || (down && (seconds == 10 || (0 < seconds && seconds <= 5)))
}

/// Is this a count, a number alone? Not one with a leading zero: X's Pokédex list numbers its
/// rows "002", and taken for a count it was said after the entry's first row, "SEEN 2 OBTAINED 1
/// 002" (x/movie2, the audit, 2026-10-10).
fn count(text: &str) -> bool {
    py::isdigit(text) && (text.chars().count() == 1 || !text.starts_with('0'))
}
