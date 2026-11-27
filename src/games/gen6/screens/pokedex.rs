//! The Pokédex lists: Omega Ruby's entry said the tick it is chosen, X's by its number and name,
//! and X's regions by the row under the cursor.
//!
//! Omega Ruby's list keeps its rows in `app::sango_zukan::ZukanLaunch`, the process's +0xC8: nine
//! records of 0x44 bytes from +0x50, each an index and then pointers into a row's panes (spike 298
//! on or_menus). The fifth's index is the entry chosen; the fourth's panes show its number and
//! name (spike 310 on or_pokedex: their words change the tick the index does, or a tick before).
//! The plate over the list is a picture fixed in place while the rows slide a row's height and back
//! under it, and its item was said once it had read the same for two polls, 0.3 to 0.4 s after
//! the index changed. The index is read every tick, and a move is said the poll it changes, in
//! the words of the fourth record's panes (`OrasList`): no slide can put another row's words
//! there. A page jumps seven, passing the entries between over 0.1 s, by one or two a tick: every
//! move is said once the index has held four ticks, as the Mart's jumps are once they hold
//! (`mart.Jumps`).
//!
//! X's list is described below.
//!
//! X's list is a row of numbered icons, and its cursor's words are the number alone, "004"; the
//! entry's name is the top screen's, on the line of its "No." and the same number, with the seen
//! and obtained counts above them and the rest of the entry below (x_pokedex). Said as they were, a
//! move was "004", then "SEEN 2 OBTAINED 1 No. Fennekin Fox Pokémon …": the counts at every move,
//! and "No." with its number left out as the cursor's words. A move is said as the panel names
//! it, "004 Fennekin", once the panel's number agrees with the cursor's, a few polls at most, what
//! the screen changes meanwhile held for its details (`CursorPoll.chased`); the details are the
//! rest of the panel. The line is left to the cursor (`CursorPoll.rows`), not taken off the
//! screen. The counts are still said with each entry: the details' table is told by how far right
//! the words before it reach (`screen_order.columns`), and left out, the counts' "1" no longer
//! held HEIGHT's and WEIGHT's values in their labels' column, "Fire Fox Pokémon TYPE HEIGHT WEIGHT
//! 1'04" 20.7 lbs.".
//!
//! The entry under the cursor is `app::zukan::DataManager` +0x130 (spike 298 on x_pokedex), the
//! process's +0x3C; X said each move the poll it changed, so it is not read.
//!
//! X's Pokédex opens on its regions, Central, Coastal and Mountain, each a row with its count, and
//! no cursor of theirs is drawn as one: only a pane of the layout moves. The region under the
//! cursor is the view's, `app::zukan::top::View` +0x18, a row from the top, through the process's
//! `Graphics` and its view of the screen up now (spike A on x_pokedex: seven of seven against the
//! screenshots). It is read every tick, and said in its row's words, "CENTRAL KALOS POKÉDEX 14",
//! once the view has built as many rows as the screen shows (`XyRegions`): as the screen comes up
//! its words are there a poll or more before any row is lit. They come 3.1 s after the app starts,
//! once the Pokédex's case has opened on the screen, long after the app's whole sweep (spike B).
//!
//! Omega Ruby's Pokédex opens on the Hoenn and National Pokédex, its cursor coming on screen just
//! as the app's whole sweep comes due: the sweep waits for it (`OrasOpening`).
//!
//! Bound by:
//! - ADR-0034: the heap's whole sweep waits for Omega Ruby's choice's cursor as the Pokédex opens.
//! - ADR-0043: the list's entry once its index holds, a jump where it lands; X's regions by the
//!   row under the cursor, as they open and as it moves; search's fields with their labels, the
//!   shapes in pa3ds's words.

use std::collections::HashSet;
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::{IndexMap, IndexSet};
use regex::Regex;

use crate::core::binary::{s32, u32};
use crate::core::memory::Memory;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll, Rect};
use crate::games::common::screen::ScreenText;
use crate::games::common::screen_order::{join, side_by_side};
use crate::games::gen6::anchors::Kept;
use crate::py::{self, Result};

/// X's and Y's Pokédex.
pub const APP: &str = "N3app5zukan8MainProcE";
/// Omega Ruby's and Alpha Sapphire's Pokédex, its registration of a Pokémon caught among its
/// screens.
pub const ORAS_APP: &str = "N3app11sango_zukan9ZukanProcE";
/// The registration's bottom screen: the entry, the model of the Pokémon caught, and the buttons
/// under it, four arrows turning and zooming the model and Next (or_groudon_master_ball).
pub const REGISTRATION: &str = "description_lower.bclyt";
/// The registration's "Rotate" and "Zoom", each hanging from a picture over two of the model's
/// arrows, touch areas with no words of their own: no button's words, but said as they came with
/// Next's as "Rotate Zoom Next" (`claims.RULES`).
pub const CAPTIONS: [(&str, &str); 2] = [(REGISTRATION, "3s"), (REGISTRATION, "3w")];
/// Omega Ruby's choice of the Hoenn and National Pokédex, each with its counts, as its Pokédex
/// opens: its words come 1.1 to 1.2 s after the app starts, and its cursor a quarter to a third of
/// a second after them, just as the app's whole sweep comes due (`OrasOpening`, spike B).
pub const CHOICE: &str = "start_lower.bclyt";
/// X's choice of the Central, Coastal and Mountain Pokédex, each with its count, as its Pokédex
/// opens: buttons, whose words are said a row at a time as the cursor comes onto them
/// (`XyRegions`).
pub const REGIONS: &str = "top_lower.bclyt";
/// X's Pokédex's process -> `app::zukan::Graphics`.
pub const GRAPHICS: i64 = 0x40;
pub const GRAPHICS_CLASS: &str = "N3app5zukan8GraphicsE";
/// `Graphics` -> the view of the screen up now: `app::zukan::top::View` on the regions screen,
/// `app::zukan::list::ScrollView` on the list. It points 0xC4 into the view, at the last of its
/// secondary vtables, which RTTI names as the view.
pub const VIEW: i64 = 0x5C;
pub const VIEW_CLASS: &str = "N3app5zukan3top4ViewE";
/// The regions' view, where `VIEW` points -> how many rows it has built, a byte counting up to
/// the three shown as the screen comes up.
pub const BUILT: i64 = 0x14;
/// The regions' view, where `VIEW` points -> the region under the cursor, a u32 counted from the
/// top row: down wraps round, left and right move nothing.
pub const REGION: i64 = 0x18;
/// The Pokédex's process -> `app::sango_zukan::ZukanLaunch`, its list.
pub const LAUNCH: i64 = 0xC8;
pub const LAUNCH_CLASS: &str = "N3app11sango_zukan11ZukanLaunchE";
pub const ROWS: i64 = 0x50;
pub const RECORD: i64 = 0x44;
/// The record whose index, a s32 at its start, is the entry chosen.
pub const CHOSEN: i64 = ROWS + 4 * RECORD;
/// The record whose panes show the entry chosen.
pub const SHOWN: i64 = ROWS + 3 * RECORD;
/// Ticks the index holds before its entry is said: a jump of seven steps it on by one or two
/// every tick or two (spike 310), and a step holds from the first.
pub const STILL_TICKS: i64 = 4;
/// Search by shape's fourteen silhouettes, the same textures in Omega Ruby and X, which the game
/// names in no word: named by `pictures.ftl`, said as the cursor comes onto them and as SHAPE's
/// value once one is chosen (spike 311). A `frozenset` of `search_icon01.bclim` to
/// `search_icon14.bclim`.
pub const SHAPES: [&str; 14] = [
    "search_icon01.bclim",
    "search_icon02.bclim",
    "search_icon03.bclim",
    "search_icon04.bclim",
    "search_icon05.bclim",
    "search_icon06.bclim",
    "search_icon07.bclim",
    "search_icon08.bclim",
    "search_icon09.bclim",
    "search_icon10.bclim",
    "search_icon11.bclim",
    "search_icon12.bclim",
    "search_icon13.bclim",
    "search_icon14.bclim",
];
/// Words with no letter or digit: a search field's or a picker's choice of none, its dashes.
/// Anchored at both ends, as Python's `fullmatch` is.
pub static WORDLESS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[^\w]+$").unwrap());
/// A list row's words: the entry's number in the Pokédex shown, three digits. Anchored at both
/// ends, as Python's `fullmatch` is.
pub static NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{3}$").unwrap());
/// The most polls a move waits for the panel to name its entry, said by its number alone
/// after.
pub const WAIT_POLLS: i64 = 3;
/// Units within which two pieces beside each other are on one line.
pub const ONE_LINE: f64 = 1.0;

/// Names X's Pokédex list's entries by the top screen's panel.
#[derive(Default)]
pub struct XyList {
    /// A move waiting for the panel to name its entry, and the polls it has waited.
    held: Option<(CursorItem, i64)>,
    /// The number last said with its name, the list's root and the panel's.
    named: Option<(String, i64, i64)>,
}

impl XyList {
    pub fn new() -> Self {
        XyList::default()
    }

    /// The cursors' poll with a move's entry named by the panel, or held for it, and the
    /// panel's line naming the entry the cursor's own, unsaid.
    pub fn poll(
        &mut self,
        running: Option<&str>,
        cursors: CursorPoll,
        on_screen: &[ScreenText],
    ) -> CursorPoll {
        if running != Some(APP) {
            (self.held, self.named) = (None, None);
            return cursors;
        }
        if let Some((_, _, panel)) = &self.named
            && !on_screen.iter().any(|p| p.root == *panel)
        {
            // The panel gone, for an entry's pages: back on the list it is said whole.
            self.named = None;
        }
        let moves: Vec<CursorItem> = cursors
            .items
            .iter()
            .filter(|item| !item.opened && !number(item).is_empty())
            .cloned()
            .collect();
        let mut items: Vec<CursorItem> = cursors
            .items
            .iter()
            .filter(|item| !moves.contains(item))
            .cloned()
            .collect();
        if let Some(last) = moves.last() {
            self.held = Some((last.clone(), 0));
        }
        let mut chased = cursors.chased;
        if let Some((item, waited)) = self.held.clone() {
            let (number, root) = (number(&item), item.pieces[0].root);
            if let Some((_, said, name)) = line(&number, root, on_screen) {
                (self.held, self.named) = (None, Some((number, root, said.root)));
                let mut pieces = item.pieces.clone();
                pieces.push(name);
                items.push(CursorItem { pieces, ..item });
            } else if waited >= WAIT_POLLS {
                self.held = None;
                items.push(item);
            } else {
                self.held = Some((item, waited + 1));
                chased = true;
            }
        }
        let mut quiet: HashSet<i64> = HashSet::new();
        if let Some((number, root, panel)) = &self.named {
            let shown: Vec<ScreenText> = on_screen
                .iter()
                .filter(|p| p.root == *panel)
                .cloned()
                .collect();
            if let Some((before, said, name)) = line(number, *root, &shown) {
                quiet = before
                    .iter()
                    .chain([&said, &name])
                    .map(|p| p.r#box)
                    .collect();
            }
        }
        let rows = cursors.rows.union(&quiet).copied().collect();
        CursorPoll {
            items,
            chased,
            rows,
            ..cursors
        }
    }
}

/// The entry number a list's row says, or "" for another cursor's item.
fn number(item: &CursorItem) -> String {
    let text = join(&item.pieces, false);
    if NUMBER.is_match(&text) {
        text
    } else {
        String::new()
    }
}

/// The panel's line naming this entry, from a layout other than the list's: the words before
/// its number, the number, and the name after it; None while the panel names another.
fn line(
    number: &str,
    root: i64,
    on_screen: &[ScreenText],
) -> Option<(Vec<ScreenText>, ScreenText, ScreenText)> {
    for said in on_screen {
        if said.root == root || said.text != number {
            continue;
        }
        let beside: Vec<&ScreenText> = on_screen
            .iter()
            .filter(|p| {
                p.root == said.root && p.r#box != said.r#box && (p.row - said.row).abs() < ONE_LINE
            })
            .collect();
        let after = py::sorted_by_lt(
            beside
                .iter()
                .filter(|p| p.left > said.left)
                .copied()
                .collect(),
            |a: &&ScreenText, b: &&ScreenText| a.left < b.left,
        );
        if let Some(first) = after.first() {
            return Some((
                beside
                    .iter()
                    .filter(|p| p.left < said.left)
                    .map(|p| (*p).clone())
                    .collect(),
                said.clone(),
                (*first).clone(),
            ));
        }
    }
    None
}

/// The cursors' poll with a search field named by its label, one of `labels`.
///
/// Search and sort in both games is a table, each field's name at its row's left and its value
/// beside it, and the cursor frames the value alone: a field not set was its dashes alone, and
/// which of NAME, COLOR, APPEARANCE or ENCOUNTERS it was went unsaid (or_pokedex, x_pokedex);
/// one set, its letter alone, "A". The labels are the game's text's (`GameText.search_labels`),
/// the table's, and a picker's for its choice of none alone, its dashes on the label's row,
/// "APPEARANCE" and its dashes: the type grid's "Fighting" has "Normal" beside it, no label, and a
/// picker's label stands beside whichever choice scrolls onto its row, "TYPE Psychic".
pub fn labelled(
    running: Option<&str>,
    cursors: CursorPoll,
    on_screen: &[ScreenText],
    labels: &HashSet<String>,
) -> CursorPoll {
    if running != Some(APP) && running != Some(ORAS_APP) {
        return cursors;
    }
    let roots = table_roots(on_screen, labels);
    let mut items: Vec<CursorItem> = Vec::new();
    for item in &cursors.items {
        let first = item.pieces.first();
        if let Some(first) = first
            && !roots.is_empty()
            && !roots.contains(&first.root)
            && item
                .pieces
                .iter()
                .map(|p| p.parent)
                .collect::<HashSet<i64>>()
                .len()
                > 1
        {
            // The cursor coming on screen frames the bar's buttons both, a poll before it is on
            // the table: "Reset Confirm", then "SORT By Number" (or_pokedex).
            continue;
        }
        let Some(first) = first.filter(|first| !labels.contains(&first.text)) else {
            items.push(item.clone());
            continue;
        };
        let boxes: HashSet<i64> = item.pieces.iter().map(|p| p.r#box).collect();
        // `max(before, key=left)`: the first of equal bests, as Python's.
        let mut label: Option<&ScreenText> = None;
        for p in on_screen {
            if p.root == first.root
                && !boxes.contains(&p.r#box)
                && (p.row - first.row).abs() < ONE_LINE
                && p.left < first.left
                && labels.contains(&p.text)
                && (roots.contains(&p.root) || WORDLESS.is_match(&join(&item.pieces, false)))
                && label.is_none_or(|best| p.left > best.left)
            {
                label = Some(p);
            }
        }
        items.push(match label {
            None => item.clone(),
            Some(label) => CursorItem {
                pieces: [vec![label.clone()], item.pieces.clone()].concat(),
                ..item.clone()
            },
        });
    }
    CursorPoll { items, ..cursors }
}

/// The label of the search field a menu cursor frames with no words in it (`menu.Blank`):
/// TYPE's two and SHAPE's are blank until set, and the cursor on them said nothing
/// (or_pokedex); SHAPE set, its shape alone, "Finned" (heard in play). Its label is left of it
/// on its row: for a picture `framed`, the table's, of a row with no words for its value, which
/// the picture is: X draws its shapes in a column beside the table, level with its rows, and
/// "ORDER Bodiless" and "COLOR Finned" were said for them, the table's values hidden while
/// they are shown, its labels not (x_pokedex_search_shapes), and its choice of none was said
/// as COLOR; for nothing framed, a picker's too, its blank row beside its label, the choice of
/// none.
pub fn blank(
    running: Option<&str>,
    rect: &Rect,
    on_screen: &[ScreenText],
    framed: &[ScreenText],
    labels: &HashSet<String>,
) -> Vec<ScreenText> {
    if running != Some(APP) && running != Some(ORAS_APP) {
        return Vec::new();
    }
    let roots = table_roots(on_screen, labels);
    let valued = |label: &ScreenText| {
        on_screen.iter().any(|q| {
            q.root == label.root
                && (q.row - label.row).abs() < ONE_LINE
                && q.left > label.left
                && !labels.contains(&q.text)
                && !q.picture
        })
    };
    // The table showing its values, not a picker over them.
    let shown = on_screen
        .iter()
        .any(|p| labels.contains(&p.text) && roots.contains(&p.root) && valued(p));
    // `max(left, key=left)`: the first of equal bests, as Python's.
    let mut best: Option<&ScreenText> = None;
    for p in on_screen {
        if labels.contains(&p.text)
            && p.top == rect.top
            && rect.low <= p.row
            && p.row <= rect.high
            && p.right <= rect.left
            && (shown || !roots.contains(&p.root))
            && (framed.is_empty() || (roots.contains(&p.root) && !valued(p)))
            && best.is_none_or(|best| p.left > best.left)
        {
            best = Some(p);
        }
    }
    best.into_iter().cloned().collect()
}

/// The boxes of search and sort's table, and of the bar under it, that come on screen with
/// them or back to them, menus the cursor says an item of at a time: opened before its cursor
/// came, the table was read whole, "SORT By Number NAME", each field's dashes and on, then
/// "Reset Confirm" (or_pokedex); back from a picker, it was the cursor's next field's details,
/// and the bar's "Reset Confirm" came back with each picker closed. The table is the panel
/// holding two labels or more: an entry's page has TYPE alone, and a picker's panel its own
/// label, left to say which picker opened.
pub fn tabled(
    running: Option<&str>,
    texts: &[ScreenText],
    on_screen: &[ScreenText],
    cursors: &CursorPoll,
    labels: &HashSet<String>,
) -> HashSet<i64> {
    if running != Some(APP) && running != Some(ORAS_APP) {
        return HashSet::new();
    }
    let tables = tables(on_screen, labels);
    if tables.is_empty() {
        return HashSet::new();
    }
    let roots: HashSet<i64> = tables.iter().map(|&(root, _panel)| root).collect();
    let under: HashSet<i64> = cursors.under.iter().flatten().map(|p| p.r#box).collect();
    texts
        .iter()
        .filter(|p| {
            !under.contains(&p.r#box)
                && (p.with_layout || p.returned)
                && (tables.contains(&(p.root, p.panel)) || (!p.top && !roots.contains(&p.root)))
        })
        .map(|p| p.r#box)
        .collect()
}

/// The layouts of search and sort's table on screen, whose rows scroll under its cursor
/// (`scrolled`): APPEARANCE's dashes scrolled away for ENCOUNTERS' were no move (or_pokedex).
pub fn search_roots(
    running: Option<&str>,
    on_screen: &[ScreenText],
    labels: &HashSet<String>,
) -> HashSet<i64> {
    if running != Some(APP) && running != Some(ORAS_APP) {
        return HashSet::new();
    }
    table_roots(on_screen, labels)
}

/// The layouts showing search and sort's table: two labels or more on screen. A picker opened
/// hides the table and shows its own one label, and the table scrolled puts APPEARANCE and
/// ENCOUNTERS in a panel of their own, one of them now and then alone in it (or_pokedex).
fn table_roots(on_screen: &[ScreenText], labels: &HashSet<String>) -> HashSet<i64> {
    let mut held: IndexMap<i64, HashSet<&str>> = IndexMap::new();
    for p in on_screen {
        if labels.contains(&p.text) {
            held.entry(p.root).or_default().insert(p.text.as_str());
        }
    }
    held.iter()
        .filter(|(_, words)| words.len() > 1)
        .map(|(&root, _)| root)
        .collect()
}

/// The (layout, panel) of each panel holding two of search and sort's labels or more.
fn tables(on_screen: &[ScreenText], labels: &HashSet<String>) -> HashSet<(i64, i64)> {
    let mut held: IndexMap<(i64, i64), HashSet<&str>> = IndexMap::new();
    for p in on_screen {
        if labels.contains(&p.text) {
            held.entry((p.root, p.panel))
                .or_default()
                .insert(p.text.as_str());
        }
    }
    held.iter()
        .filter(|(_, words)| words.len() > 1)
        .map(|(&panel, _)| panel)
        .collect()
}

/// `files: Callable[[Iterable[int]], Mapping[int, str]]`: the layouts on screen, by root, the
/// file each was built from (`layout_file.FilesOnScreen.poll`).
pub type Files = Box<dyn FnMut(Vec<i64>) -> IndexMap<i64, String>>;

/// X's Pokédex's region under the cursor, said in its row's words as the cursor comes onto
/// it. Asked beside Omega Ruby's list (`sources.CursorSources`), before X's list and search:
/// a row naming no entry's number and no search label, it passes them as it is.
pub struct XyRegions {
    memory: Rc<dyn Memory>,
    running: Rc<dyn Fn() -> Option<String>>,
    kept: Kept,
    files: Files,
    /// The view, the rows it has built and the region under the cursor, at the last tick.
    read: Option<(i64, i64, i64)>,
    /// The region last said, while the regions are up.
    said: Option<i64>,
}

impl XyRegions {
    /// `files` gives the layouts on screen the file each was built from
    /// (`layout_file.FilesOnScreen.poll`).
    pub fn new(
        memory: Rc<dyn Memory>,
        running: Rc<dyn Fn() -> Option<String>>,
        process: Rc<dyn Fn() -> Option<i64>>,
        files: Files,
    ) -> Self {
        XyRegions {
            kept: Kept::new(memory.clone(), process, running.clone()),
            memory,
            running,
            files,
            read: None,
            said: None,
        }
    }

    /// Has the region under the cursor, or the rows built, changed since the last tick?
    /// Asked every tick, three reads while X's Pokédex runs.
    pub fn moved(&mut self) -> Result<bool> {
        let read = self.now()?;
        let moved = read.is_some() && read != self.read;
        self.read = read;
        Ok(moved)
    }

    /// The cursors' poll with the region under the cursor, the poll it changes, in its
    /// row's words.
    pub fn cursors(&mut self, cursors: CursorPoll, on_screen: &[ScreenText]) -> CursorPoll {
        let Some((view, built, region)) = self.read else {
            // Not the regions: they are said afresh when they come back.
            self.said = None;
            return cursors;
        };
        // A set Python iterates: what is done for each root reads nothing.
        let roots: IndexSet<i64> = on_screen
            .iter()
            .filter(|p| !p.top)
            .map(|p| p.root)
            .collect();
        let files = (self.files)(roots.into_iter().collect());
        let mut rows: Vec<Vec<ScreenText>> = Vec::new();
        let regions: Vec<ScreenText> = on_screen
            .iter()
            .filter(|p| !p.top && files.get(&p.root).map(String::as_str) == Some(REGIONS))
            .cloned()
            .collect();
        for piece in py::sorted_by_lt(regions, |a, b| -a.row < -b.row) {
            match rows.last_mut() {
                Some(row) if side_by_side(&row[0], &piece) => row.push(piece),
                _ => rows.push(vec![piece]),
            }
        }
        let count = rows.len() as i64;
        if Some(region) == self.said || !(region < count && count <= built) {
            return cursors;
        }
        let opened = self.said.is_none();
        self.said = Some(region);
        let pieces = py::sorted_by_lt(rows.swap_remove(region as usize), |a, b| a.left < b.left);
        let mut items = cursors.items.clone();
        items.push(CursorItem::new(view, pieces, opened));
        CursorPoll { items, ..cursors }
    }

    /// The view, the rows it has built and the region under the cursor, while the regions
    /// are up.
    fn now(&mut self) -> Result<Option<(i64, i64, i64)>> {
        if (self.running)().as_deref() != Some(APP) {
            self.kept.forget();
            return Ok(None);
        }
        let graphics = self.kept.find(&[GRAPHICS], Some(GRAPHICS_CLASS), &[])?;
        let view = match graphics {
            None => None,
            Some(graphics) => self.kept.checked(graphics, &[VIEW], Some(VIEW_CLASS))?,
        };
        let Some(view) = view else {
            return Ok(None);
        };
        let state = self.memory.read(view + BUILT, REGION + 4 - BUILT)?;
        Ok(Some((view, state[0] as i64, u32(&state, REGION - BUILT)?)))
    }
}

/// Whether Omega Ruby's Pokédex is opening on its choice, its cursor yet to come: the heap's
/// whole sweep waits meanwhile, as it does for a cursor moving (ADR-0034, amended 2026-11-25).
///
/// The app's own sweep comes due 1.5 s after it starts, and the choice's cursor came on screen
/// 1.45 to 1.5 s after: the screen poll waited the sweep out, 0.5 to 0.6 s live and 1.1 s on the
/// tapes, and the first choice was said that late (spike B, or_pokedex and or_menus). The wait
/// starts as the app does, before the choice's words come, and ends with a cursor on screen or
/// words of another layout, the registration's after a catch, whose sweep is as it was.
pub struct OrasOpening {
    running: Rc<dyn Fn() -> Option<String>>,
    files: Files,
    /// Whether the Pokédex is still opening, since it started; None while it does not run.
    waiting: Option<bool>,
}

impl OrasOpening {
    /// `files` gives the layouts on screen the file each was built from
    /// (`layout_file.FilesOnScreen.poll`).
    pub fn new(running: Rc<dyn Fn() -> Option<String>>, files: Files) -> Self {
        OrasOpening {
            running,
            files,
            waiting: None,
        }
    }

    /// Every tick: is the Pokédex opening, its cursor yet to come?
    pub fn waiting(&mut self) -> bool {
        if (self.running)().as_deref() != Some(ORAS_APP) {
            self.waiting = None;
        } else if self.waiting.is_none() {
            self.waiting = Some(true);
        }
        self.waiting == Some(true)
    }

    /// Every screen poll: a cursor on screen, or words not the choice's, end the wait. A
    /// layout whose file is not known yet is taken for the choice's.
    pub fn poll(&mut self, cursors: &CursorPoll, on_screen: &[ScreenText]) {
        if self.waiting != Some(true) {
            return;
        }
        // A set Python iterates: what is done for each root reads nothing.
        let roots: IndexSet<i64> = on_screen.iter().map(|p| p.root).collect();
        let files = (self.files)(roots.into_iter().collect());
        if !cursors.under.is_empty()
            || on_screen
                .iter()
                .any(|p| files.get(&p.root).map_or(CHOICE, String::as_str) != CHOICE)
        {
            self.waiting = Some(false);
        }
    }
}

/// Omega Ruby's Pokédex list's entry, said the tick it is chosen in its row's own words.
pub struct OrasList {
    memory: Rc<dyn Memory>,
    running: Rc<dyn Fn() -> Option<String>>,
    kept: Kept,
    /// Its `ZukanLaunch` at the last tick, once found and checked by its class.
    launch: Option<i64>,
    /// The panes the record showing the entry chosen points at. A `frozenset` only tested.
    boxes: HashSet<i64>,
    index: Option<i64>,
    /// Ticks the index has held, after a step or a jump.
    still: i64,
    /// The entry last said, and its words, while the list is up.
    said: Option<(i64, String)>,
}

impl OrasList {
    pub fn new(
        memory: Rc<dyn Memory>,
        running: Rc<dyn Fn() -> Option<String>>,
        process: Rc<dyn Fn() -> Option<i64>>,
    ) -> Self {
        OrasList {
            kept: Kept::new(memory.clone(), process, running.clone()),
            memory,
            running,
            launch: None,
            boxes: HashSet::new(),
            index: None,
            still: 0,
            said: None,
        }
    }

    /// Has the entry chosen changed since the last tick? Asked every tick, one read.
    pub fn moved(&mut self) -> Result<bool> {
        let Some(index) = self.read()? else {
            self.index = None;
            return Ok(false);
        };
        if Some(index) == self.index {
            self.still += 1;
            return Ok(false);
        }
        (self.index, self.still) = (Some(index), 0);
        Ok(true)
    }

    /// The cursors' poll with the plate's items replaced by the entry chosen, said the poll
    /// it changes in its row's words, a jump once it holds still.
    pub fn cursors(&mut self, cursors: CursorPoll, on_screen: &[ScreenText]) -> Result<CursorPoll> {
        let launch = match self.launch {
            Some(launch) if (self.running)().as_deref() == Some(ORAS_APP) => launch,
            _ => {
                self.said = None;
                return Ok(cursors);
            }
        };
        if !on_screen.iter().any(|p| self.boxes.contains(&p.r#box)) {
            // Its panes are set once the list is built, after the Pokédex starts.
            let record = self.memory.read(launch + SHOWN, RECORD)?;
            let mut boxes = HashSet::new();
            for at in (4..RECORD).step_by(4) {
                boxes.insert(u32(&record, at)?);
            }
            self.boxes = boxes;
        }
        let pieces: Vec<ScreenText> = py::sorted_by_lt(
            on_screen
                .iter()
                .filter(|p| self.boxes.contains(&p.r#box))
                .cloned()
                .collect(),
            |a: &ScreenText, b: &ScreenText| a.left < b.left,
        );
        let root = pieces.first().map(|p| p.root);
        let Some(root) = root.filter(|root| {
            cursors
                .under
                .iter()
                .any(|under| under.first().is_some_and(|first| first.root == *root))
        }) else {
            // No plate over the list: its entry is said afresh when it comes back.
            self.said = None;
            return Ok(cursors);
        };
        let plate: Vec<CursorItem> = cursors
            .items
            .iter()
            .filter(|item| item.pieces.first().is_some_and(|first| first.root == root))
            .cloned()
            .collect();
        let items: Vec<CursorItem> = cursors
            .items
            .iter()
            .filter(|item| !plate.contains(item))
            .cloned()
            .collect();
        let (index, words) = (self.index, join(&pieces, false));
        let Some(index) =
            index.filter(|index| self.said.as_ref().is_none_or(|(said, _)| said != index))
        else {
            return Ok(CursorPoll { items, ..cursors });
        };
        if self.still < STILL_TICKS {
            return Ok(CursorPoll {
                items,
                chased: true,
                ..cursors
            });
        }
        if self.said.as_ref().is_some_and(|(_, said)| *said == words) {
            // The index has moved and its row's words not yet.
            return Ok(CursorPoll {
                items,
                chased: true,
                ..cursors
            });
        }
        let cursor = plate.first().map_or(launch, |item| item.cursor);
        let opened = self.said.is_none();
        self.said = Some((index, words));
        let mut items = items;
        items.push(CursorItem::new(cursor, pieces, opened));
        Ok(CursorPoll { items, ..cursors })
    }

    /// The entry chosen, while the Pokédex runs and its list is built.
    fn read(&mut self) -> Result<Option<i64>> {
        if (self.running)().as_deref() != Some(ORAS_APP) {
            self.kept.forget();
            self.launch = None;
            return Ok(None);
        }
        self.launch = self.kept.find(&[LAUNCH], Some(LAUNCH_CLASS), &[])?;
        let Some(launch) = self.launch else {
            return Ok(None);
        };
        s32(&self.memory.read(launch + CHOSEN, 4)?, 0).map(Some)
    }
}
