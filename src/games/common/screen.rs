//! Screen text: announce the words a screen draws outside any message window.
//!
//! A menu's labels, the options screen's names and values, and a question such as "Would you
//! like to make these changes?" are not messages. Each is a string set straight into an
//! `nw::lyt::TextBox` pane of a NintendoWare layout (spike 28), laid out as `lyt` gives.
//!
//! Layouts hold far more text than is on screen — options this language does not have, questions
//! not asked yet, whole screens not shown yet — so a TextBox's words are heard only when code
//! wrote them (the tag processor is set) and `Layouts` says the TextBox is being drawn (spikes 29
//! to 31), or when they are a designer's separator between two written words, as Sun's "/" is
//! (`separators`).
//!
//! Words are reported once they read the same on two polls, so a string caught halfway through
//! being written is not spoken. A message window's line objects are TextBoxes too; the dialogue
//! reader owns those, and passes them in to be left out.
//!
//! Each piece of a screen is its own TextBox — the party screen draws "L", "5", "19", "/" and
//! "19" as five — so `join` puts pieces drawn side by side back together, from where the pane
//! says its words are: its global place, width and scale, its anchor, and the TextBox's glyph
//! width and text position (spike 33).
//!
//! Bound by:
//! - ADR-0013: words a screen draws are heard as they come on screen and as they change; no commas
//!   but under `pauses`.
//! - ADR-0037: stars counted, Potential's and a result's, one drawn as a shadow not earned and
//!   unsaid.

use std::collections::{BTreeSet, HashSet};
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{f32, u16, u32};
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::pictures::{
    HALF_STAR, HEART, NO_HEART, NO_STAR, PLACES, SPECIAL, STAR, hearts, stars,
};
use crate::games::common::text::pages;
use crate::games::common::{lyt, msgwin};
use crate::py::{self, Error, Kind, PySet, Result, sorted_by_lt};

/// Halfwords. Screen text is labels and short sentences; this only rejects garbage lengths.
pub const MAX_TEXT: i64 = 0x400;

/// Left, centre, right: how far across its box an anchor or an alignment sits.
pub const FRACTION: [f64; 3] = [0.0, 0.5, 1.0];

/// An average glyph's width across, as a share of the TextBox's glyph width. An estimate:
/// "Torchic", seven glyphs at 12.5, drew about 44 units wide in spike 31's party screen picture.
pub const GLYPH_ADVANCE: f64 = 0.5;
/// Glyphs past its entrant's words a contest's place is taken to stand (`_places_last`): past
/// `LABEL_GAP`, the gap at which a value arriving alone is said with its label.
pub const PLACE_AFTER: f64 = 2.5;
/// Units a hidden heart is taken to stand past the words it hangs from, and to be wide and tall:
/// a contest's hearts are 10 by 10, 10 apart (spike 196).
pub const GLYPH_SPOT: f64 = 10.0;
/// Units within which two pieces with the same words are one drawn twice.
pub const SAME_PLACE: f64 = 2.0;
/// Pieces whose words come closer than this many glyph widths are one piece: "L5" and "19/19" are
/// 6 to 8 units apart at a glyph width of 16, and "PP 35/35"'s "PP" and "35" about 15 at 12.5.
pub const JOIN_GAP: f64 = 0.8;
/// What Sun's party screen writes where it draws "Lv.": a TextBox of its own holding a lone "l",
/// drawn as the level mark by the font the level's digits are in, just left of the level in the
/// same group. Read as it is, Rowlet's slot was "Rowlet 22 22 l5" (sun_9.ctm, 2026-09-27).
pub const LEVEL_GLYPH: &str = "l";
/// Polls after its layout's first words within which a piece still comes with its screen: the
/// battle bag's "LAST USED ITEM" was written a poll after the pockets beside it.
pub const ARRIVAL_POLLS: i64 = 2;
pub const SCREEN_HALF_HEIGHT: f64 = 120.0;
/// Half the bottom screen's width, and the top's. A layout's origin is the middle of its screen.
pub const SCREEN_HALF_WIDTH: [f64; 2] = [160.0, 200.0];

/// Words a screen has just shown: in a TextBox newly on screen, or newly written into one.
///
/// Python's `ScreenText(box, text, changed)` leaves the rest at their defaults: [`ScreenText::new`]
/// does, [`ScreenText::placed`] gives the place too, and the other fields are `pub` for a struct
/// update.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenText {
    pub r#box: i64,
    pub text: String,
    /// True when the TextBox was already on screen and its words changed, as a value's do.
    pub changed: bool,
    pub left: f64,
    /// Where the words are estimated to start and end across the screen.
    pub right: f64,
    pub row: f64,
    /// The glyph width; 0 when the pane says nothing usable about where it is.
    pub glyph: f64,
    /// On the top screen.
    pub top: bool,
    /// The root pane of the layout it is in.
    pub root: i64,
    /// The pane it hangs from: the group of a row, a panel, a slot.
    pub parent: i64,
    /// The pane directly under its layout's root that it hangs from. Where `parent` is the row or
    /// slot, this is the part of the screen: the battle's HP box keeps the player's level and HP in
    /// one panel and the wild Pokemon's level in another (spike 53), and words from two panels are
    /// two things to say rather than one run.
    pub panel: i64,
    /// It came on screen in the same poll as the first of its layout's pieces: part of a screen
    /// opening or coming back, not words appearing later in a screen already up.
    pub with_layout: bool,
    /// It came back on screen with the words it left with.
    pub returned: bool,
    /// Its words are what the game calls what a picture draws (`gen6.rom.textures`): a type's icon,
    /// a status condition's.
    pub picture: bool,
    /// Read first in its row, wherever it is drawn in it (`after_the_name`).
    pub leads: bool,
    /// It moved onto its screen with the words it had just outside it: a list's row as the list
    /// scrolls, where the language screen moves its rows rather than writing new words into them.
    pub scrolled: bool,
    /// For words hanging straight from their layout's root, the `nw::lyt::Window` of the layout
    /// drawn round them, or 0. The move-forget screen hangs every word from its root, the Pokémon's
    /// stats and the chosen move's alike, and groups them only by the frames it draws: one round
    /// CATEGORY, POWER and ACCURACY, one round the description (spike 139).
    pub frame: i64,
    /// Its words are what the game's text says a screen draws as a label (`GameText.labels`):
    /// a label of what follows it however it is drawn, under `pauses`.
    pub label: bool,
    /// Its words are a title's or a button's, no label however they are drawn
    /// (`GameText.plain`).
    pub plain: bool,
}

impl ScreenText {
    /// `ScreenText(box, text, changed)`: every other field at its default.
    pub fn new(r#box: i64, text: impl Into<String>, changed: bool) -> Self {
        ScreenText {
            r#box,
            text: text.into(),
            changed,
            left: 0.0,
            right: 0.0,
            row: 0.0,
            glyph: 0.0,
            top: false,
            root: 0,
            parent: 0,
            panel: 0,
            with_layout: false,
            returned: false,
            picture: false,
            leads: false,
            scrolled: false,
            frame: 0,
            label: false,
            plain: false,
        }
    }

    /// `ScreenText(box, text, changed, left, right, row, glyph)`: the rest at their defaults.
    pub fn placed(
        r#box: i64,
        text: impl Into<String>,
        changed: bool,
        left: f64,
        right: f64,
        row: f64,
        glyph: f64,
    ) -> Self {
        ScreenText {
            left,
            right,
            row,
            glyph,
            ..ScreenText::new(r#box, text, changed)
        }
    }

    /// The layout's root and the pane or frame its words are grouped by.
    pub fn group(&self) -> (i64, i64) {
        (
            self.root,
            if self.frame != 0 {
                self.frame
            } else {
                self.parent
            },
        )
    }

    /// Do the two stand in one place in one panel, one over the other?
    pub fn over(&self, other: &ScreenText) -> bool {
        self.root == other.root
            && self.panel == other.panel
            && self.left < other.right
            && other.left < self.right
            && (self.row - other.row).abs() < py_max(self.glyph, other.glyph)
    }
}

/// What one screen text poll found (`ScreenTextReader.poll`).
///
/// Python's two `set`s are `IndexSet`s here, in the order Python iterates them.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScreenPoll {
    /// What has come on screen or changed since the last poll, the top screen first.
    pub texts: Vec<ScreenText>,
    /// Every piece on screen, reported or not, the top screen first.
    pub on_screen: Vec<ScreenText>,
    /// The pieces that left the screen, as they last read: a letter deleted from a name is one
    /// of them.
    pub left: Vec<ScreenText>,
    /// The roots of layouts whose words on screen were not all reported, because some had not
    /// yet read the same for a poll: words still being written.
    pub settling: IndexSet<i64>,
    /// The TextBoxes a message window was typing into, whether or not the dialogue reader has
    /// found that window yet: handed to it, they lead to the window the moment it types, where a
    /// sweep took up to 2 s (spike 60).
    pub message_lines: IndexSet<i64>,
}

/// Python's `min(a, b)` on floats: `a`, unless `b < a`. A NaN is never smaller, so it stays
/// whichever side it is on, where `f64::min` would drop it.
fn min_of(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Python's `max(a, b)` on floats: `a`, unless `b > a`.
fn max_of(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// The pieces in the order given, with each run of them on one row read left to right.
///
/// The order the game built its TextBoxes in keeps a screen's parts together — the summary's two
/// columns of stats, a party slot's name above its level — and reading the whole screen row by row
/// interleaved those columns (spike 47). Inside a row that order is not reading order: a move's
/// "/" is built before both of its PP numbers (spike 33). A run is consecutive pieces less than
/// half a glyph apart up and down; pieces whose place is unknown end a run and stay where they are.
pub fn in_rows(pieces: impl IntoIterator<Item = ScreenText>) -> Vec<ScreenText> {
    let mut ordered: Vec<ScreenText> = Vec::new();
    let mut run: Vec<ScreenText> = Vec::new();
    for piece in pieces {
        if !run.is_empty()
            && (piece.glyph <= 0.0 || (run[0].row - piece.row).abs() >= piece.glyph / 2.0)
        {
            ordered.extend(sorted_across(std::mem::take(&mut run)));
        }
        if piece.glyph <= 0.0 {
            ordered.push(piece);
        } else {
            run.push(piece);
        }
    }
    ordered.extend(sorted_across(run));
    ordered
}

/// `sorted(run, key=_across)`.
fn sorted_across(run: Vec<ScreenText>) -> Vec<ScreenText> {
    // The key is `(not leads, left)`: a tuple is smaller by its first element that differs.
    sorted_by_lt(run, |a, b| {
        if a.leads != b.leads {
            return a.leads && !b.leads;
        }
        a.left < b.left
    })
}

/// The pieces row by row from the top, each row left to right: a panel of labels and values, as
/// a move's Category, Power and Accuracy, read as a table. A row is pieces less than half a glyph
/// apart up and down. Not for a whole screen, whose columns this would interleave (spike 47).
pub fn by_rows(pieces: impl IntoIterator<Item = ScreenText>) -> Vec<ScreenText> {
    let placed: Vec<ScreenText> = pieces.into_iter().filter(|p| p.glyph > 0.0).collect();
    let placed = sorted_by_lt(placed, |a, b| -a.row < -b.row);
    let mut ordered: Vec<ScreenText> = Vec::new();
    let mut row: Vec<ScreenText> = Vec::new();
    for piece in placed {
        if !row.is_empty() && row[0].row - piece.row >= piece.glyph / 2.0 {
            ordered.extend(sorted_across(std::mem::take(&mut row)));
        }
        row.push(piece);
    }
    ordered.extend(sorted_across(row));
    ordered
}

/// The pieces without the same words drawn twice in one place.
///
/// The X menu draws each label in two TextBoxes a unit apart (spike 38). The same words somewhere
/// else are kept: a move's PP reads "40 / 40", and dropping the second "40" lost the PP left.
pub fn distinct(pieces: impl IntoIterator<Item = ScreenText>) -> Vec<ScreenText> {
    let mut out: Vec<ScreenText> = Vec::new();
    for piece in pieces {
        if !out.iter().any(|other| {
            other.text == piece.text
                && (other.left - piece.left).abs() <= SAME_PLACE
                && (other.row - piece.row).abs() <= SAME_PLACE
        }) {
            out.push(piece);
        }
    }
    out
}

/// The pieces as one line to speak, in their order.
///
/// Pieces drawn side by side on one row are joined as drawn ("L5", "19/19"). Others are separated
/// by a space, or when asked for (the user, 2026-09-14) by a comma, or a colon after a label
/// (`labels_value`): "OT: Raiden, ID No.: 16285" (Orion, 2026-09-30). A label with a colon of its
/// own is followed by a space: "Badges: 0", not "Badges:, 0". A value is no label of what
/// follows it: "ITEM: Potion, Sand Attack", "Name: Raiden, ID No.: 16285".
///
/// Words the game's text names a label (`ScreenText.label`) are one wherever they are drawn,
/// touching their value or over it, and a title's or a button's (`ScreenText.plain`) never are:
/// the DexNav's "Found Pokémon, Search Level: 64, Name: Zigzagoon" was "Found Pokémon: Search
/// Level, 64, Name, Zigzagoon", and "Search Level5" is "Search Level 5" (spike 195).
///
/// Python's `pauses` defaults to False.
pub fn join(pieces: &[ScreenText], pauses: bool) -> String {
    let mut out = String::new();
    let mut valued = false;
    for (index, piece) in pieces.iter().enumerate() {
        let before = if index != 0 {
            Some(&pieces[index - 1])
        } else {
            None
        };
        let mut labelled = false;
        match before {
            None => out = piece.text.clone(),
            Some(before)
                if (touching(before, piece) || counted(before, piece)) && !before.label =>
            {
                out.push_str(&piece.text)
            }
            Some(before) if !pauses || before.text.ends_with(':') => {
                out.push(' ');
                out.push_str(&piece.text);
                labelled = pauses;
            }
            Some(before) => {
                labelled = !valued
                    && !before.plain
                    && !piece.label
                    && (before.label || labels_value(before, piece));
                out.push_str(if labelled { ": " } else { ", " });
                out.push_str(&piece.text);
            }
        }
        valued = labelled;
    }
    out
}

/// The most words a label has: "To Next Lv." has three.
pub const LABEL_WORDS: usize = 3;
/// Glyph widths past a label's end a value ends at, at least: a table's values are drawn to its
/// right edge.
pub const VALUE_BEYOND: f64 = 2.0;
/// Glyph widths below a label its value may start: the summary draws Exp. Points' number on the
/// row under it.
pub const VALUE_BELOW: f64 = 2.5;

/// Is `first` a label and `second` its value, as a table draws them?
///
/// The summary names none of its panes for what they hold (spike 148: "0", "4", "1k"), so this is
/// told by place: a label is a few words with a letter and no digit, and its value starts no
/// further left, on its row, a little above it or just below, and ends well past it, as a table's
/// right-aligned column does. A value below its label starts past the label's end, as Exp.
/// Points' number does: the next row of a list starts under the one above it ("Ember", "Sand
/// Attack"). A label's words are the screen's own, so words that changed are a value's, and a
/// picture's words are what it draws, as a type's icon, "Fire", is (movie11, 2026-10-01).
pub fn labels_value(first: &ScreenText, second: &ScreenText) -> bool {
    // ponytail: judged by place alone; a screen this gets wrong wants its label panes listed.
    let glyph = min_of(first.glyph, second.glyph);
    if glyph <= 0.0
        || first.changed
        || first.picture
        || first.text.ends_with(':')
        || first.text.chars().any(py::is_digit)
        || !first.text.chars().any(py::is_alpha)
    {
        return false;
    }
    let below = first.row - second.row > glyph / 2.0;
    // A sign is no value: the party slot draws the gender sign at a place of its own, far past a
    // short name. A dash is one: a status move's Power and Accuracy, "Power, —" (movie32).
    let valued = second.text.chars().any(|c| {
        py::is_alnum(c)
            || unicode_general_category::get_general_category(c)
                == unicode_general_category::GeneralCategory::DashPunctuation
    });
    py::split(&first.text).len() <= LABEL_WORDS
        && valued
        && second.left
            >= (if below {
                first.right
            } else {
                first.left - glyph / 2.0
            })
        && -glyph <= first.row - second.row
        && first.row - second.row <= glyph * VALUE_BELOW
        && second.right >= first.right + glyph * VALUE_BEYOND
}

/// Is `first` a sign and `second` the number it is drawn before on its row? The battle's Bag
/// draws an item's times sign and its count in boxes that touch, but the words are estimated
/// narrower than their boxes, and under `pauses` it read "Poké Ball, times, 10" (movie3,
/// spike 196).
fn counted(first: &ScreenText, second: &ScreenText) -> bool {
    let glyph = min_of(first.glyph, second.glyph);
    glyph > 0.0
        && (first.row - second.row).abs() < glyph
        && first.left < second.left
        && !first.text.chars().any(py::is_alnum)
        && second.text.chars().next().is_some_and(py::is_digit)
}

fn touching(first: &ScreenText, second: &ScreenText) -> bool {
    let glyph = min_of(first.glyph, second.glyph);
    if glyph <= 0.0 || (first.row - second.row).abs() >= glyph {
        return false;
    }
    -glyph < second.left - first.right && second.left - first.right <= glyph * JOIN_GAP
}

/// What `poll` keeps of each window shown in a layout: the pane, then its left, right, low and
/// high edges.
type Frame = (i64, f64, f64, f64, f64);

/// `pictures: Callable[[int], str | None]`: what the game calls what a picture draws now, by its
/// image (`gen6.rom.textures`). It reads the emulator, so it can raise, and the Rust one returns a
/// `Result` for that.
pub type Pictures = Rc<dyn Fn(i64) -> Result<Option<String>>>;

/// `dict[key]`, raising `KeyError` where the key is not there.
#[track_caller]
fn get(map: &IndexMap<i64, i64>, key: i64) -> Result<i64> {
    map.get(&key)
        .copied()
        .ok_or_else(|| Error::key(key.to_string()))
}

/// Tracks layout TextBoxes and reports the words that come on screen.
pub struct ScreenTextReader {
    level_mark: Option<String>,
    pictures: Option<Pictures>,
    /// A label's words -> what a mark in its group says, or "" (`GameText.labels`).
    labels: IndexMap<String, String>,
    /// Words a screen draws as no label (`GameText.plain`).
    plain: HashSet<String>,
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    layouts: Rc<Layouts>,
    windows: Vec<String>,
    boxes: Vec<i64>,
    /// TextBox -> its words at the last poll and where they started and stood, for TextBoxes
    /// that were on screen.
    seen: IndexMap<i64, (String, f64, f64)>,
    /// TextBoxes held at the last poll as words of their layout slid (`poll`).
    moving: HashSet<i64>,
    /// TextBox -> the words last reported, for TextBoxes still on screen.
    said: IndexMap<i64, String>,
    /// TextBox -> the piece last reported for it, to say what a word that left was.
    last_said: IndexMap<i64, ScreenText>,
    /// TextBox -> the words it left the screen with, for TextBoxes off screen and tracked.
    gone: IndexMap<i64, String>,
    /// TextBox -> its piece at the last poll, for TextBoxes counted on screen then.
    last: IndexMap<i64, ScreenText>,
    /// TextBoxes counted on screen at the last poll only because they had been the poll before.
    carried: HashSet<i64>,
    on_screen: Vec<ScreenText>,
    polls: i64,
    /// Layout root -> the poll its pieces came on screen at, for layouts on screen.
    root_since: IndexMap<i64, i64>,
    /// TextBox -> the poll it came on screen at, for TextBoxes on screen.
    box_since: IndexMap<i64, i64>,
    /// Tag processor -> whether it is one a message window embeds, until the next `track`.
    window_tags: IndexMap<i64, bool>,
    /// TextBoxes a message window is typing into, as this poll finds them.
    message_lines: PySet,
    /// TextBoxes of a layout still tracked that were hidden at the last poll.
    hidden: HashSet<i64>,
    /// TextBox -> its words, for TextBoxes drawn outside their screen at the last poll.
    outside: IndexMap<i64, String>,
    /// TextBox -> its words, for TextBoxes that moved onto their screen and are not reported
    /// yet: words are reported a poll after they arrive, when they are no longer outside.
    scrolled: IndexMap<i64, String>,
    /// Layout root -> (window, left, right, low, high) for each window shown in it, at this
    /// poll.
    frames: IndexMap<i64, Vec<Frame>>,
}

impl ScreenTextReader {
    /// `ScreenTextReader(memory, names, layouts)`: the message window classes of `msgwin`, no
    /// level mark and no pictures.
    pub fn new(memory: Rc<dyn Memory>, names: Rc<ClassNames>, layouts: Rc<Layouts>) -> Self {
        ScreenTextReader::with(
            memory,
            names,
            layouts,
            msgwin::CLASSES.iter().map(|c| c.to_string()).collect(),
            None,
            None,
            IndexMap::new(),
            HashSet::new(),
        )
    }

    /// `windows` are the game's message window classes, whose lines this leaves out.
    /// `level_mark` is the game's own words for the level mark Sun draws as a glyph
    /// (`LEVEL_GLYPH`), which are said in its place. `pictures` says what the game calls what a
    /// picture draws now, by its image (`gen6.rom.textures`), or None. `labels` are the words the
    /// game's text draws as labels, each with what a mark in its group says, and `plain` its
    /// titles and buttons (`GameText.labels`, `GameText.plain`).
    #[allow(clippy::too_many_arguments)]
    pub fn with(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        layouts: Rc<Layouts>,
        windows: Vec<String>,
        level_mark: Option<String>,
        pictures: Option<Pictures>,
        labels: IndexMap<String, String>,
        plain: HashSet<String>,
    ) -> Self {
        ScreenTextReader {
            level_mark,
            pictures,
            labels,
            plain,
            memory,
            names,
            layouts,
            windows,
            boxes: Vec::new(),
            seen: IndexMap::new(),
            moving: HashSet::new(),
            said: IndexMap::new(),
            last_said: IndexMap::new(),
            gone: IndexMap::new(),
            last: IndexMap::new(),
            carried: HashSet::new(),
            on_screen: Vec::new(),
            polls: 0,
            root_since: IndexMap::new(),
            box_since: IndexMap::new(),
            window_tags: IndexMap::new(),
            message_lines: PySet::new(),
            hidden: HashSet::new(),
            outside: IndexMap::new(),
            scrolled: IndexMap::new(),
            frames: IndexMap::new(),
        }
    }

    /// Every piece on screen at the last poll, reported or not, the top screen first: what touch
    /// mode names its targets by, as a key asks.
    pub fn on_screen(&self) -> Vec<ScreenText> {
        self.on_screen.clone()
    }

    /// Read these TextBoxes from now on.
    ///
    /// Each sweep replaces the last, except that whatever is on screen stays tracked: a sweep taken
    /// while a scene loads can miss objects that are back a moment later, and losing them would
    /// announce a whole screen again.
    pub fn track(&mut self, boxes: impl IntoIterator<Item = i64>) {
        let every: BTreeSet<i64> = boxes.into_iter().chain(self.said.keys().copied()).collect();
        self.boxes = every.into_iter().collect();
        let tracked: HashSet<i64> = self
            .boxes
            .iter()
            .copied()
            .chain(self.layouts.panes(lyt::TEXTBOX_CLASS))
            .collect();
        self.gone.retain(|r#box, _| tracked.contains(r#box));
        self.window_tags.clear();
    }

    /// What has come on screen since the last poll, the top screen first, and the rest of what
    /// this poll found (`ScreenPoll`).
    ///
    /// `exclude` is the TextBoxes message windows lay their pages out in. With `hold`, words are
    /// reported only once they read the same on two polls in a row; without it, at once. Words
    /// that moved more than a glyph since the last poll hold their layout's words back a poll,
    /// so that what slides in together is heard together, once it has stopped: Tile Puzzle's
    /// difficulty buttons slide in from the screen's edge over two polls, and heard as their
    /// words held, the button sliding out was heard with them, "Normal 5 stars Hard 2 stars", the
    /// two of its stars still on screen (or_amie_tile_puzzle_play_again). A word only bobbing, as
    /// "MISS!" does over the two polls it is up (as_amie_head_it), is heard at once, and so are
    /// words sliding as their layout opens, which arrive as a screen does (`ARRIVAL_POLLS`): a
    /// level up's stats cascade in, and held, they were gone before their values were heard
    /// (or_victory_road_boulders).
    ///
    /// Python's `exclude` defaults to no TextBoxes and `hold` to True.
    pub fn poll(&mut self, exclude: &HashSet<i64>, hold: bool) -> Result<ScreenPoll> {
        self.layouts.refresh();
        self.polls += 1;
        self.message_lines.clear();
        self.hidden.clear();
        self.frames.clear();
        let outside = std::mem::take(&mut self.outside);
        let mut found: IndexMap<i64, ScreenText> = IndexMap::new();
        let mut designed: IndexMap<i64, ScreenText> = IndexMap::new();
        // A layout's own tree finds its TextBoxes as soon as its work is known, where a sweep of
        // the linear heap took seconds: AreaNav's "Littleroot Town" (spike 49).
        let every: BTreeSet<i64> = self
            .boxes
            .iter()
            .copied()
            .chain(self.layouts.panes(lyt::TEXTBOX_CLASS))
            .collect();
        for r#box in every {
            if !exclude.contains(&r#box) {
                let (piece, written) = self.shown(r#box)?;
                if let Some(piece) = piece {
                    if written {
                        found.insert(r#box, piece);
                    } else {
                        designed.insert(r#box, piece);
                    }
                }
            }
        }
        for (r#box, piece) in separators(&designed, &found) {
            found.insert(r#box, piece);
        }
        // `{p.root for p in (*found.values(), *designed.values())}`, a set iterated in its order.
        let roots: Vec<i64> = PySet::of(found.values().chain(designed.values()).map(|p| p.root))
            .iter()
            .collect();
        let named = self.named_pictures(&roots)?;
        // A picture naming what its group writes says nothing new: the Bag's select bar draws a
        // Pokémon's icon beside its name (movie31).
        let written: HashSet<(i64, i64, String)> = found
            .values()
            .map(|p| (p.root, p.parent, p.text.clone()))
            .collect();
        for (r#box, piece) in named {
            if !written.contains(&(piece.root, piece.parent, piece.text.clone())) {
                found.insert(r#box, piece);
            }
        }
        found = after_the_name(&found);
        // After it: a group's one picture is read first in its row, and the DexNav's mark is no
        // such picture: "First Move: egg move, Helping Hand" (spike 195).
        found = by_their_labels(&found, &self.labels);
        found = places_last(&found);
        if let Some(level_mark) = &self.level_mark {
            found = level_marks(&found, level_mark);
        }
        // A pane read while the game rewrites it can sit 1,000 units off for one poll (spike 50),
        // and the move list read "Growl PP 40/" and then "3535" as its pieces came back. Words
        // missing for a single poll are taken as still there, but not words hidden as their row
        // is rewritten: the Mart's list, wrapping to its end, wrote "Quit" into a row and hid the
        // row's price in one frame, and carried, the price was read with it, "Quit $ 2,500"
        // (movie35).
        let rewritten: Vec<&ScreenText> = found
            .iter()
            .filter(|&(r#box, piece)| {
                self.last
                    .get(r#box)
                    .is_some_and(|last| last.text != piece.text)
            })
            .map(|(_, piece)| piece)
            .collect();
        let carried: IndexMap<i64, ScreenText> = self
            .last
            .iter()
            .filter(|&(r#box, piece)| {
                !found.contains_key(r#box)
                    && !exclude.contains(r#box)
                    && !self.carried.contains(r#box)
                    && !(self.hidden.contains(r#box)
                        && rewritten.iter().any(|other| {
                            // `max(other.glyph, piece.glyph)`: the first unless the second is
                            // greater.
                            let glyph = if piece.glyph > other.glyph {
                                piece.glyph
                            } else {
                                other.glyph
                            };
                            other.root == piece.root && (other.row - piece.row).abs() < glyph / 2.0
                        }))
            })
            .map(|(&r#box, piece)| (r#box, piece.clone()))
            .collect();
        self.carried = carried.keys().copied().collect();
        let mut now: IndexMap<i64, ScreenText> = found.clone();
        now.extend(carried);
        self.last = now.clone();
        self.on_screen = now.values().cloned().collect();
        self.on_screen.sort_by_key(reading_order);
        let mut root_since: IndexMap<i64, i64> = IndexMap::new();
        for p in now.values() {
            let since = self.root_since.get(&p.root).copied().unwrap_or(self.polls);
            root_since.insert(p.root, since);
        }
        self.root_since = root_since;
        let mut box_since: IndexMap<i64, i64> = IndexMap::new();
        for r#box in now.keys() {
            let since = self.box_since.get(r#box).copied().unwrap_or(self.polls);
            box_since.insert(*r#box, since);
        }
        self.box_since = box_since;
        let mut scrolled = std::mem::take(&mut self.scrolled);
        scrolled.extend(outside);
        self.scrolled = scrolled
            .into_iter()
            .filter(|(r#box, text)| found.get(r#box).is_some_and(|piece| &piece.text == text))
            .collect();

        let mut out: Vec<ScreenText> = Vec::new();
        let seen: IndexMap<i64, (String, f64, f64)> = now
            .iter()
            .map(|(&r#box, piece)| (r#box, (piece.text.clone(), piece.left, piece.row)))
            .collect();
        let ready: IndexMap<i64, &ScreenText> = found
            .iter()
            .filter(|(r#box, piece)| {
                self.said.get(*r#box) != Some(&piece.text)
                    && (!hold
                        || self.seen.get(*r#box).map_or("", |seen| seen.0.as_str()) == piece.text)
            })
            .map(|(&r#box, piece)| (r#box, piece))
            .collect();
        let mut sliding: HashSet<i64> = HashSet::new();
        for (r#box, piece) in &ready {
            if hold
                && moved(
                    self.seen
                        .get(r#box)
                        .ok_or_else(|| Error::key(r#box.to_string()))?,
                    piece,
                )
                && !self.moving.contains(r#box)
                && self.polls - get(&self.root_since, piece.root)? > ARRIVAL_POLLS
            {
                sliding.insert(piece.root);
            }
        }
        let moving: HashSet<i64> = ready
            .iter()
            .filter(|(r#box, piece)| sliding.contains(&piece.root) && !self.moving.contains(*r#box))
            .map(|(&r#box, _)| r#box)
            .collect();
        for (r#box, piece) in &ready {
            if moving.contains(r#box) {
                continue;
            }
            let with_layout =
                get(&self.box_since, *r#box)? - get(&self.root_since, piece.root)? <= ARRIVAL_POLLS;
            let returned = !self.said.contains_key(r#box)
                && self.gone.shift_remove(r#box).as_ref() == Some(&piece.text);
            let scrolled = self.scrolled.shift_remove(r#box).is_some();
            out.push(ScreenText {
                changed: self.said.contains_key(r#box),
                with_layout,
                returned,
                scrolled,
                ..(*piece).clone()
            });
            self.said.insert(*r#box, piece.text.clone());
            self.last_said.insert(*r#box, out[out.len() - 1].clone());
        }
        let going: Vec<i64> = self
            .said
            .keys()
            .copied()
            .filter(|r#box| !now.contains_key(r#box))
            .collect();
        let left: Vec<ScreenText> = going
            .iter()
            .filter_map(|r#box| self.last_said.get(r#box).cloned())
            .collect();
        for r#box in going {
            let words = self
                .said
                .shift_remove(&r#box)
                .ok_or_else(|| Error::key(r#box.to_string()))?;
            self.gone.insert(r#box, words);
            self.last_said.shift_remove(&r#box);
        }
        let settling = PySet::of(
            now.iter()
                .filter(|(r#box, p)| self.said.get(*r#box) != Some(&p.text))
                .map(|(_, p)| p.root),
        );
        self.seen = seen;
        self.moving = moving;
        out.sort_by_key(reading_order);
        Ok(ScreenPoll {
            texts: out,
            on_screen: self.on_screen.clone(),
            left,
            settling: settling.iter().collect(),
            message_lines: self.message_lines.copy().iter().collect(),
        })
    }

    /// The words of a TextBox being drawn and where they are, or None; and whether code wrote
    /// them, which a designer's words left as the layout file has them were not.
    fn shown(&mut self, r#box: i64) -> Result<(Option<ScreenText>, bool)> {
        if self
            .names
            .of_vtable(u32(&self.layouts.pane(r#box)?, 0)?)?
            .as_deref()
            != Some(lyt::TEXTBOX_CLASS)
        {
            return Ok((None, false));
        }
        let shape = self.layouts.shape;
        let head = self.layouts.textbox(r#box)?;
        let tag = u32(&head, shape.tag_processor)?;
        if tag != 0 && self.is_window_tag(tag)? {
            self.message_lines.add(r#box);
            return Ok((None, false));
        }
        let placement = self.layouts.placement(r#box)?;
        let Some(placement) = placement.filter(|placement| placement.shown) else {
            if placement.is_some() {
                self.hidden.insert(r#box);
            }
            return Ok((None, false));
        };
        let text = self.text(&head)?;
        if text.is_empty() {
            return Ok((None, false));
        }
        let placed = placed(r#box, &text, &head, &shape)?;
        let parent = self.group_of(u32(&head, lyt::PARENT)?, true)?;
        let mut piece = ScreenText {
            top: placement.top,
            root: placement.root,
            panel: placement.panel,
            parent,
            label: self.labels.contains_key(&text),
            plain: self.plain.contains(&text),
            ..placed
        };
        let root = self.layouts.pane(placement.root)?;
        if off_screen(&piece, f32(&root, shape.x)?, f32(&root, shape.y)?) {
            self.outside.insert(r#box, text);
            return Ok((None, false));
        }
        if piece.parent == piece.root && piece.glyph > 0.0 {
            piece.frame = self.frame_of(&piece)?;
        }
        Ok((Some(piece), tag != 0))
    }

    /// Picture -> what the game calls what it draws, as a piece where the picture is, for the
    /// pictures shown in layouts with words on screen: the Pokédex entry's type, drawn as an icon
    /// beside the word TYPE (spike 147; Orion, 2026-09-29), "Male TYPE Fire".
    fn named_pictures(&mut self, roots: &[i64]) -> Result<IndexMap<i64, ScreenText>> {
        let Some(pictures) = &self.pictures else {
            return Ok(IndexMap::new());
        };
        let shape = self.layouts.shape;
        let mut out: IndexMap<i64, ScreenText> = IndexMap::new();
        for &root in roots {
            let root_head = self.layouts.pane(root)?;
            for (pane, kind) in self.layouts.tree(root) {
                let placement = self.layouts.placement(pane)?;
                let Some(placement) = placement.filter(|_| kind == lyt::PICTURE_CLASS) else {
                    continue;
                };
                let mut head = self.layouts.pane(pane)?;
                let mut parent = u32(&head, lyt::PARENT)?;
                let words: String;
                if !placement.shown {
                    // A contest hides the hearts of Jamming a move does not have, and said bare,
                    // "Jamming" sounded cut off: hidden under words shown, they are counted empty
                    // (spike 196).
                    if !placement.drawn || self.group_of(parent, false)? == parent {
                        continue;
                    }
                    let above = self.layouts.placement(parent)?;
                    if !above.is_some_and(|above| above.shown)
                        || pictures(pane)?.as_deref() != Some(HEART)
                    {
                        continue;
                    }
                    words = NO_HEART.to_string();
                    head = self.layouts.pane(parent)?;
                } else {
                    match pictures(pane)?.filter(|words| !words.is_empty()) {
                        Some(found) => words = found,
                        None => continue,
                    }
                }
                let star = [STAR, NO_STAR, HALF_STAR].contains(&words.as_str());
                if (words == STAR || words == HALF_STAR)
                    && shadow(&self.memory.read(pane + lyt::PICTURE_COLOURS, 16)?)
                {
                    // Pokémon-Amie's results draw each star not earned as a full one's faint
                    // shadow, where an earned one lands over it (or_amie_stars), and are said only
                    // with stars earned, as "caught" is never "not caught": shown seconds before
                    // Berry Picker's rank, under "Berries", they read "Berries 0 stars".
                    continue;
                }
                if star
                    && self
                        .names
                        .of_vtable(u32(&self.layouts.pane(parent)?, 0)?)?
                        .as_deref()
                        == Some(lyt::BOUNDING_CLASS)
                {
                    // Tile Puzzle's difficulty buttons box each star of the best result alone, and
                    // counted by its box each read "Easy 1 star1 star1 star1 star1 star"
                    // (as_amie_tile_puzzle_hard): counted by the button instead.
                    parent = u32(&self.layouts.pane(parent)?, lyt::PARENT)?;
                }
                let mut width = f32(&head, shape.width)? * f32(&head, shape.scale_x)?;
                let mut height = f32(&head, shape.height)? * f32(&head, shape.scale_y)?;
                if width <= 0.0 || height <= 0.0 {
                    continue;
                }
                let (mut x, y) = (f32(&head, shape.x)?, f32(&head, shape.y)?);
                if words == NO_HEART {
                    // Hidden, its own place is not kept up: it stands just past its words.
                    (x, width, height) = (x + width / 2.0 + GLYPH_SPOT, GLYPH_SPOT, GLYPH_SPOT);
                }
                // Wholly off its screen is not on it: the summary keeps a fifth move's type icon
                // 140 above the bottom screen's middle, 130 to 150, its edge at 120 (movie7). A
                // star is on it only wholly: the next difficulty button's first star peeks in at
                // the edge, 152 to 170, and Easy's read "Easy 5 stars 1 star".
                let outside = if star { -1.0 } else { 1.0 };
                let across = (x - f32(&root_head, shape.x)?).abs() - outside * width / 2.0;
                let up = (y - f32(&root_head, shape.y)?).abs() - outside * height / 2.0;
                if up >= SCREEN_HALF_HEIGHT
                    || across >= SCREEN_HALF_WIDTH[usize::from(placement.top)]
                {
                    continue;
                }
                // Its height stands in for a glyph's width: what keeps it in its row.
                out.insert(
                    pane,
                    ScreenText {
                        top: placement.top,
                        root: placement.root,
                        panel: placement.panel,
                        parent,
                        picture: true,
                        ..ScreenText::placed(
                            pane,
                            words,
                            false,
                            x - width / 2.0,
                            x + width / 2.0,
                            y,
                            height,
                        )
                    },
                );
            }
        }
        // Counted under the words they hang from, then grouped as those words are, by the frame
        // round them too: the move-forget screen's category icon, grouped with the stats, read
        // them all as each move's details (or_tm-hm, 2026-10-18).
        let counted = side_by_side(&counted_stars(&out)?);
        let mut grouped: IndexMap<i64, ScreenText> = IndexMap::new();
        for (r#box, piece) in counted {
            let parent = self.group_of(piece.parent, false)?;
            grouped.insert(r#box, ScreenText { parent, ..piece });
        }
        let mut framed: IndexMap<i64, ScreenText> = IndexMap::new();
        for (r#box, piece) in grouped {
            let piece = if piece.parent == piece.root {
                ScreenText {
                    frame: self.frame_of(&piece)?,
                    ..piece
                }
            } else {
                piece
            };
            framed.insert(r#box, piece);
        }
        Ok(framed)
    }

    /// The pane what hangs from `parent` is grouped by: its parent, or where that is a
    /// TextBox, the TextBox's. A contest hangs a move's hearts under "Appeal" (movie30, spike
    /// 196). Other `words` only under a TextBox hanging from its layout's root: the Bag hangs an
    /// item's count under its times sign, and as a group of its own it was read after the
    /// description (movie29); the PC Box's list, regrouped under words of its rows, read its rows
    /// as new (movie9).
    ///
    /// Python's `words` defaults to False.
    fn group_of(&self, parent: i64, words: bool) -> Result<i64> {
        let head = self.layouts.pane(parent)?;
        if self.names.of_vtable(u32(&head, 0)?)?.as_deref() != Some(lyt::TEXTBOX_CLASS) {
            return Ok(parent);
        }
        let above = u32(&head, lyt::PARENT)?;
        if words && u32(&self.layouts.pane(above)?, lyt::PARENT)? != 0 {
            return Ok(parent);
        }
        Ok(above)
    }

    /// The window shown in the piece's layout whose rectangle holds the words' middle, or 0.
    fn frame_of(&mut self, piece: &ScreenText) -> Result<i64> {
        if !self.frames.contains_key(&piece.root) {
            let shape = self.layouts.shape;
            let mut frames: Vec<Frame> = Vec::new();
            for (pane, kind) in self.layouts.tree(piece.root) {
                let placement = self.layouts.placement(pane)?;
                let Some(_) = placement.filter(|placement| placement.shown) else {
                    continue;
                };
                if kind != lyt::WINDOW_CLASS {
                    continue;
                }
                let head = self.layouts.pane(pane)?;
                let half_width = f32(&head, shape.width)? * f32(&head, shape.scale_x)? / 2.0;
                let half_height = f32(&head, shape.height)? * f32(&head, shape.scale_y)? / 2.0;
                let (x, y) = (f32(&head, shape.x)?, f32(&head, shape.y)?);
                frames.push((
                    pane,
                    x - half_width,
                    x + half_width,
                    y - half_height,
                    y + half_height,
                ));
            }
            self.frames.insert(piece.root, frames);
        }
        let middle = (piece.left + piece.right) / 2.0;
        Ok(self.frames[&piece.root]
            .iter()
            .find(|&&(_, left, right, low, high)| {
                left <= middle && middle <= right && low <= piece.row && piece.row <= high
            })
            .map_or(0, |frame| frame.0))
    }

    /// Is this the tag processor a message window embeds, so the line is a message's?
    ///
    /// A message window's line objects are left to the dialogue reader, which says a page at a
    /// time. `exclude` names them once that reader has found the window, but whatever else finds
    /// TextBoxes can find a window's lines first: the linear heap's rolling sweep did, and Birch's
    /// "Whew..." was heard twice (spike 46). The window's own tag processor names them from the
    /// start.
    fn is_window_tag(&mut self, tag: i64) -> Result<bool> {
        if !self.window_tags.contains_key(&tag) {
            let classes: Vec<&str> = self.windows.iter().map(String::as_str).collect();
            let is_window = msgwin::window_of(&self.names, tag, &classes)?.is_some();
            self.window_tags.insert(tag, is_window);
        }
        Ok(self.window_tags[&tag])
    }

    /// A TextBox's words whether or not it is on screen, where code wrote them: the X menu keeps
    /// its words hidden under the shortcut bar's icons (`games.common.touch`). A designer's words,
    /// a placeholder of full-width digits left as the layout file has it, are "": the AreaNav's
    /// tabs were named with them (movie17, Orion, 2026-10-01).
    pub fn text_of(&self, r#box: i64) -> Result<String> {
        let head = self.layouts.textbox(r#box)?;
        if u32(&head, self.layouts.shape.tag_processor)? != 0 {
            self.text(&head)
        } else {
            Ok(String::new())
        }
    }

    /// The TextBox's words, control codes dropped and whitespace collapsed.
    fn text(&self, head: &[u8]) -> Result<String> {
        let shape = self.layouts.shape;
        let (capacity, length) = (u16(head, shape.capacity)?, u16(head, shape.length)?);
        if !(0 < length && length <= capacity.min(MAX_TEXT)) {
            return Ok(String::new());
        }
        let spoken = (|| {
            let raw = self.memory.read(u32(head, shape.text)?, length * 2)?;
            pages(&raw)
        })();
        let spoken = match spoken {
            Ok(spoken) => spoken,
            Err(error) if error.is(Kind::ValueError) => return Ok(String::new()),
            Err(error) => return Err(error),
        };
        let pages: Vec<&str> = spoken.iter().map(|page| page.text.as_str()).collect();
        Ok(py::split(&pages.join(" ")).join(" "))
    }
}

/// Have the piece's words moved more than a glyph since they were `seen`? A difficulty button
/// slides 40 to 160 a poll (or_amie_tile_puzzle_play_again), a Pokédex entry's panel 17; Berry
/// Picker's last seconds bob 1 to 3, "MISS!" and "START!" 5 (as_amie_berry_picker,
/// as_amie_head_it).
fn moved(seen: &(String, f64, f64), piece: &ScreenText) -> bool {
    let (_text, left, row) = seen;
    piece.glyph > 0.0 && py_max((piece.left - left).abs(), (piece.row - row).abs()) > piece.glyph
}

/// `max(a, b)` of floats: `a` unless `b` is strictly greater, as Python's.
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Is the piece wholly outside its screen, judged from its layout's root at (root_x, root_y)?
///
/// The X menu, closed, keeps its labels visible and drawn 189 units below the middle of the bottom
/// screen, whose edge is 120 below it (spike 48). Counted as shown, they were read whenever the
/// menu's layout was built again, as the summary, the bag or the trainer card closed to the field.
///
/// A layout's global places include a view offset from its work (+0x74): 290 on the field's bottom
/// screen, 1,000 in battle, where the top screen's roots come out at (0, 1000) and its words about
/// 1,000 up. Judged from (0, 0), every battle HP box and the level up's stats were left out
/// (battles.ctm, spikes 47 and 48); the X menu's root stays at (0, 0) as it parks its labels.
/// A piece whose place is unknown is kept.
///
/// Python's `root_x` and `root_y` default to 0.0.
fn off_screen(piece: &ScreenText, root_x: f64, root_y: f64) -> bool {
    if piece.glyph <= 0.0 {
        return false;
    }
    let half_width = SCREEN_HALF_WIDTH[usize::from(piece.top)];
    (piece.row - root_y).abs() > SCREEN_HALF_HEIGHT + piece.glyph
        || piece.right - root_x < -half_width
        || piece.left - root_x > half_width
}

/// The designer's pieces holding no letter or digit that touch a written piece of their panel
/// on each side: the "/" Sun's party screen and HP box leave as the layout file has it between
/// the current HP and the maximum, which code writes. Read as written words only, the party slot
/// ran the two numbers together (sun_9.ctm). The designer's other words are placeholders, "999"
/// and fullwidth "123", in panes built for what this screen does not show, and are left out.
/// Of the panel, not the layout: as the HP box comes on, all its panes sit at one place, and a
/// double battle's panel's "/" touched the player's numbers.
fn separators(
    designed: &IndexMap<i64, ScreenText>,
    written: &IndexMap<i64, ScreenText>,
) -> IndexMap<i64, ScreenText> {
    let beside = |piece: &ScreenText, left: bool| -> bool {
        // A piece hanging straight from the root is a panel of its own, so the summary's "/",
        // between HP's numbers, was of nobody's panel and "HP 30 30" was said (spike 148).
        written.values().any(|other| {
            (other.panel == piece.panel || other.parent == piece.parent)
                && (if left {
                    touching(other, piece)
                } else {
                    touching(piece, other)
                })
        })
    };
    designed
        .iter()
        .filter(|(_, piece)| {
            !piece.text.chars().any(py::is_alnum) && beside(piece, true) && beside(piece, false)
        })
        .map(|(&r#box, piece)| (r#box, piece.clone()))
        .collect()
}

/// These pieces, with each lone `LEVEL_GLYPH` just left of a number in its own group, on its
/// row, read as `words`.
fn level_marks(found: &IndexMap<i64, ScreenText>, words: &str) -> IndexMap<i64, ScreenText> {
    let mut out = found.clone();
    for (&r#box, piece) in found {
        if piece.text == LEVEL_GLYPH
            && found.values().any(|other| {
                other.parent == piece.parent
                    && py::isdigit(&other.text)
                    && other.left > piece.left
                    && (other.row - piece.row).abs() < max_of(piece.glyph, other.glyph) / 2.0
            })
        {
            out.insert(
                r#box,
                ScreenText {
                    text: words.to_string(),
                    ..piece.clone()
                },
            );
        }
    }
    out
}

/// A group's one picture read straight after the one piece above it, as a move's type after
/// its name. The battle's move list mirrors its two columns, the type icon left of the PP on one
/// and right of it on the other, and read as drawn the right one's was "Growl PP 40/40 Normal";
/// where the mirroring cannot be seen it only sounds lopsided (Orion, 2026-09-29), so it is
/// "Growl Normal PP 40/40". Only a group of two rows with one piece on the upper is shaped so:
/// the Pokédex entry's TYPE is one of three rows, and stays "TYPE Fire".
pub fn after_the_name(found: &IndexMap<i64, ScreenText>) -> IndexMap<i64, ScreenText> {
    let mut groups: IndexMap<i64, Vec<&ScreenText>> = IndexMap::new();
    for piece in found.values() {
        groups.entry(piece.parent).or_default().push(piece);
    }
    let mut out = found.clone();
    for pieces in groups.values() {
        let pictures: Vec<&ScreenText> = pieces.iter().copied().filter(|p| p.picture).collect();
        if pictures.len() != 1 {
            continue;
        }
        let icon = pictures[0];
        let words: Vec<&ScreenText> = pieces.iter().copied().filter(|p| !p.picture).collect();
        let above: Vec<&ScreenText> = words
            .iter()
            .copied()
            .filter(|p| p.row - icon.row >= icon.glyph / 2.0)
            .collect();
        let beside: Vec<&ScreenText> = words
            .iter()
            .copied()
            .filter(|p| (p.row - icon.row).abs() < icon.glyph / 2.0)
            .collect();
        if above.len() == 1 && !beside.is_empty() && above.len() + beside.len() == words.len() {
            out.insert(
                icon.r#box,
                ScreenText {
                    leads: true,
                    ..icon.clone()
                },
            );
        }
    }
    out
}

/// Whether a picture's four corners tint it a faint black (`lyt.PICTURE_COLOURS`).
pub(crate) fn shadow(colours: &[u8]) -> bool {
    (0..16)
        .step_by(4)
        .all(|at| py::slice(colours, at, at + 3) == b"\0\0\0" && colours[at as usize + 3] < 0x80)
}

/// Each group's stars as one piece, how many are filled: the DexNav's Potential draws three,
/// filled or not, "Potential 2 stars" (Orion, 2026-10-06; spike 195). Side by side, three stars
/// alike were one picture drawn three times. A contest move's hearts likewise, as many as are
/// drawn: "Appeal 4 hearts" (spike 196). Pokémon-Amie's results rate out of five in halves, a
/// half star where the whole ones end, under the fifth when all five are earned (or_amie_stars):
/// a half star counts where no whole one covers it.
///
/// Returns `Result` where Python's `int` of a row read out of freed memory would raise.
pub(crate) fn counted_stars(
    pictures: &IndexMap<i64, ScreenText>,
) -> Result<IndexMap<i64, ScreenText>> {
    let counted =
        |piece: &ScreenText| [STAR, NO_STAR, HALF_STAR, HEART, NO_HEART].contains(&&*piece.text);
    let mut groups: IndexMap<(i64, i64, bool, i64), Vec<&ScreenText>> = IndexMap::new();
    for piece in pictures.values() {
        if counted(piece) {
            let heart = piece.text == HEART || piece.text == NO_HEART;
            // A row each: a summary hangs Appeal's hearts and Jamming's in one window (movie33).
            let row = if heart && piece.glyph > 0.0 {
                py::int(py::floordiv_f(piece.row, 2.0 * piece.glyph))?
            } else {
                0
            };
            groups
                .entry((piece.root, piece.parent, heart, row))
                .or_default()
                .push(piece);
        }
    }
    let mut out: IndexMap<i64, ScreenText> = pictures
        .iter()
        .filter(|(_, piece)| !counted(piece))
        .map(|(&r#box, piece)| (r#box, piece.clone()))
        .collect();
    for (&(_root, _parent, heart, _row), group) in &groups {
        // `min(group, key=left)` and `max(right ...)`: the first of equals, and a NaN never
        // smaller or larger.
        let mut first = group[0];
        let mut right = group[0].right;
        for &piece in &group[1..] {
            if piece.left < first.left {
                first = piece;
            }
            if piece.right > right {
                right = piece.right;
            }
        }
        let whole: Vec<&ScreenText> = group
            .iter()
            .copied()
            .filter(|piece| piece.text == STAR)
            .collect();
        let half = group.iter().any(|p| {
            p.text == HALF_STAR && !whole.iter().any(|w| p.left < w.right && w.left < p.right)
        });
        let text = if heart {
            hearts(group.iter().filter(|piece| piece.text == HEART).count() as i64)
        } else {
            stars(whole.len() as i64, half)
        };
        out.insert(
            first.r#box,
            ScreenText {
                text,
                right,
                ..first.clone()
            },
        );
    }
    Ok(out)
}

/// The pictures of a group whose label the game's text names, and nothing else, read straight
/// after the label: the DexNav's Potential and its stars, "Potential 2 stars", which as built
/// were read last of its box. And the DexNav's red mark as what it marks, by its group's label,
/// read straight after the group's value: "First Move Helping Hand egg move" (spike 195); a mark
/// in a group whose label has no word for it is left unsaid.
///
/// A piece moved is put just after the one it follows in the order pieces are read, apart from
/// it on its row: a pane's address is a multiple of four, so the next few are no pane's.
pub(crate) fn by_their_labels(
    found: &IndexMap<i64, ScreenText>,
    labels: &IndexMap<String, String>,
) -> IndexMap<i64, ScreenText> {
    let mut out = found.clone();
    for (&r#box, picture) in found {
        if !picture.picture {
            continue;
        }
        let group: Vec<&ScreenText> = found
            .iter()
            .filter(|&(&other, piece)| {
                piece.parent == picture.parent && piece.root == picture.root && other != r#box
            })
            .map(|(_, piece)| piece)
            .collect();
        let label = group.iter().copied().find(|piece| piece.label);
        let (after, moved): (&ScreenText, ScreenText);
        // A picture's alone: "!" is a key of Sun's name entry keyboard too.
        if picture.text == SPECIAL {
            out.shift_remove(&r#box);
            let mut values: Vec<&ScreenText> = group
                .iter()
                .copied()
                .filter(|piece| !piece.label && !piece.picture)
                .collect();
            values.sort_by_key(|piece| piece.r#box);
            let word = label
                .and_then(|label| labels.get(&label.text))
                .filter(|word| !word.is_empty());
            let (Some(word), Some(&last)) = (word, values.last()) else {
                continue;
            };
            after = last;
            moved = ScreenText {
                text: own(word, &[]),
                ..picture.clone()
            };
        } else if let Some(label) = label
            && group.iter().all(|piece| piece.label || piece.picture)
        {
            // A label its pictures alone are the value of: the Pokédex entry's group of Name
            // holds Name's value and the type icon, "Name Fire Torchic" once moved (movie7).
            out.shift_remove(&r#box);
            after = label;
            moved = picture.clone();
        } else {
            continue;
        }
        let mut at = after.r#box + 1;
        while out.contains_key(&at) {
            at += 1;
        }
        out.insert(
            at,
            ScreenText {
                r#box: at,
                row: after.row,
                left: after.right + 2.0 * after.glyph,
                right: after.right + 3.0 * after.glyph,
                glyph: after.glyph,
                leads: false,
                ..moved
            },
        );
    }
    out
}

/// Pictures drawn side by side as one piece, their words between slashes: a Pokémon of two
/// types is "Bug/Water" (Orion, 2026-09-29), where joined as drawn they read "BugWater". A
/// picture saying what the one beside it says is one thing drawn twice, as each Pokédex language
/// tab is.
/// A contest's place, drawn left of its entrant, read after the entrant's words: "Raiden &
/// Metagross third place" (Orion, 2026-10-09; spike 196). The results reveal the places one at
/// a time after the names, so a place stands far enough past its entrant's words to be said
/// with them as a value is with its label (`LABEL_GAP`).
pub(crate) fn places_last(found: &IndexMap<i64, ScreenText>) -> IndexMap<i64, ScreenText> {
    let places: HashSet<String> = PLACES.iter().map(|&(_, place)| own(place, &[])).collect();
    let mut out = found.clone();
    for (&r#box, piece) in found {
        if !piece.picture || !places.contains(piece.text.as_str()) {
            continue;
        }
        let words: Vec<&ScreenText> = found
            .values()
            .filter(|p| {
                p.root == piece.root && p.parent == piece.parent && !p.picture && p.glyph > 0.0
            })
            .collect();
        // `max(words, key=right)`: the first of equals.
        let Some(&first) = words.first() else {
            continue;
        };
        let mut after = first;
        for &p in &words[1..] {
            if p.right > after.right {
                after = p;
            }
        }
        out.insert(
            r#box,
            ScreenText {
                row: after.row,
                glyph: after.glyph,
                left: after.right + PLACE_AFTER * after.glyph,
                right: after.right + (PLACE_AFTER + 1.0) * after.glyph,
                ..piece.clone()
            },
        );
    }
    out
}

pub(crate) fn side_by_side(pictures: &IndexMap<i64, ScreenText>) -> IndexMap<i64, ScreenText> {
    let mut out: IndexMap<i64, ScreenText> = IndexMap::new();
    let mut last: Option<ScreenText> = None;
    // The key is `(not top, root, -row, left)`: a tuple is smaller by its first element that
    // differs.
    let ordered = sorted_by_lt(
        pictures.values().collect(),
        |a: &&ScreenText, b: &&ScreenText| {
            if a.top != b.top {
                return a.top && !b.top;
            }
            if a.root != b.root {
                return a.root < b.root;
            }
            if -a.row != -b.row {
                return -a.row < -b.row;
            }
            a.left < b.left
        },
    );
    for piece in ordered {
        // Of one group: the Pokéblock Case's star draws "TOUGH" in its layout touching a move's
        // icon on its row, and Electro Ball read "Coolness/Toughness" (movie33).
        if let Some(before) = last.clone()
            && before.root == piece.root
            && before.parent == piece.parent
            && touching(&before, piece)
        {
            // A count drawn a digit a picture is one number: Pokémon-Amie's games' results.
            if py::isdigit(&before.text) && py::isdigit(&piece.text) {
                let joined = ScreenText {
                    text: format!("{}{}", before.text, piece.text),
                    right: piece.right,
                    ..before
                };
                out.insert(joined.r#box, joined.clone());
                last = Some(joined);
            } else if !before.text.split('/').any(|part| part == piece.text) {
                let joined = ScreenText {
                    text: format!("{}/{}", before.text, piece.text),
                    right: piece.right,
                    ..before
                };
                out.insert(joined.r#box, joined.clone());
                last = Some(joined);
            }
            continue;
        }
        out.insert(piece.r#box, piece.clone());
        last = Some(piece.clone());
    }
    out
}

/// The top screen first, then the order the layouts built their panes in.
pub fn reading_order(piece: &ScreenText) -> (bool, i64) {
    (!piece.top, piece.r#box)
}

/// The words with where they are estimated to be drawn. A pane's global matrix is filled in when
/// it is drawn, so a scale of 0 means nothing about its place can be believed yet.
fn placed(r#box: i64, text: &str, head: &[u8], shape: &lyt::PaneShape) -> Result<ScreenText> {
    let scale = f32(head, shape.scale_x)?;
    if scale <= 0.0 {
        return Ok(ScreenText::new(r#box, text, false));
    }
    let width = f32(head, shape.width)? * scale;
    let glyph = f32(head, shape.glyph_width)? * scale;
    let box_left = f32(head, shape.x)? - FRACTION[shape.anchored(head)? as usize] * width;
    let words = min_of(width, text.chars().count() as f64 * glyph * GLYPH_ADVANCE);
    let left = box_left + (width - words) * FRACTION[shape.aligned(head)? as usize];
    Ok(ScreenText::placed(
        r#box,
        text,
        false,
        left,
        left + words,
        f32(head, shape.y)?,
        glyph,
    ))
}
