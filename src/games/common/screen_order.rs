//! The order screen text is read in, and its pieces made one line to speak: what
//! `common.screen_announce` says of the pieces `common.screen` reads, and what the cursors, touch
//! mode and the screens' readers join.
//!
//! A screen's TextBoxes are built in an order that keeps its parts together, the summary's two
//! columns of stats, a party slot's name above its level, but not a row's: a move's "/" is built
//! before both of its PP numbers (spikes 33, 47). So pieces are read in the order their layouts
//! built them, each run on a row left to right (`in_rows`), a panel at a time (`panels_of`), a
//! panel that is one column down it, and tables side by side each down its columns (`columns`); a
//! value that came alone is said with the label beside it (`with_row_mates`). Pieces drawn side by
//! side on a row are joined as drawn, the rest a space apart, or under `pauses` a comma, or a colon
//! after a label (`join`, `labels_value`).
//!
//! Two pieces are on one row here by the smaller of their glyphs (`side_by_side`), and in
//! `common.screen`'s reader by the larger: the two stay apart, as one in the other's place changes
//! what is said. What the reader itself sorts and joins by, `reading_order` and `touching`, is
//! `common.screen`'s, which this module reads `ScreenText` from.
//!
//! Bound by:
//! - ADR-0013: no commas but under `pauses`; each rule of the order is from live play: replay the
//!   tapes before and after changing one.

use std::collections::HashSet;

use indexmap::{IndexMap, IndexSet};

use crate::games::common::screen::{ScreenText, reading_order, touching};
use crate::py::{self, sorted_by_lt};

/// Units within which two pieces with the same words are one drawn twice.
pub const SAME_PLACE: f64 = 2.0;
/// Glyphs between a label and the value it names: the level up's are five apart, a name's letters
/// one.
pub const LABEL_GAP: f64 = 2.0;
/// Units within which two words beside each other are drawn on one line, as a table's label and
/// value are.
pub const ONE_LINE: f64 = 1.0;

/// Python's `min(a, b)` on floats: `a`, unless `b < a`. A NaN is never smaller, so it stays
/// whichever side it is on, where `f64::min` would drop it.
fn min_of(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
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
    followed(ordered)
}

/// `ordered` with each piece that `follows` another present moved straight after it.
fn followed(ordered: Vec<ScreenText>) -> Vec<ScreenText> {
    // Python's `list.remove` and `list.index` find a piece by identity before equality: `out`
    // holds each piece by its place in `ordered`, so a piece is found as itself.
    let mut out: Vec<usize> = (0..ordered.len()).collect();
    let find = |out: &[usize], i: usize| {
        out.iter()
            .position(|&k| k == i || ordered[k] == ordered[i])
            .expect("in the list")
    };
    for (i, piece) in ordered.iter().enumerate() {
        if piece.follows != 0 {
            let target = out
                .iter()
                .copied()
                .find(|&k| ordered[k].r#box == piece.follows);
            if let Some(target) = target {
                out.remove(find(&out, i));
                let at = find(&out, target) + 1;
                out.insert(at, i);
            }
        }
    }
    out.into_iter().map(|k| ordered[k].clone()).collect()
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
    followed(ordered)
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
/// by a space, or when asked for by a comma, or a colon after a label
/// (`labels_value`): "OT: Brendan, ID No.: 16285". A label with a colon of its
/// own is followed by a space: "Badges: 0", not "Badges:, 0". A value is no label of what
/// follows it: "ITEM: Potion, Sand Attack", "Name: Brendan, ID No.: 16285".
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
                out.push_str(if labelled {
                    ": "
                } else if abbreviates(before, piece) {
                    " "
                } else {
                    ", "
                });
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

/// The most letters of a word cut short before its full stop: "Lv.".
pub const ABBREVIATION: usize = 3;

/// Is `first` a word cut short and `second` the number it names, after it on its row? A trade
/// draws the offered Pokémon's "Lv." and its level apart, and under `pauses` it read "Lv., 9"
/// (the PSS's multiplayer).
fn abbreviates(first: &ScreenText, second: &ScreenText) -> bool {
    let glyph = min_of(first.glyph, second.glyph);
    let word: Vec<char> = first
        .text
        .rsplit(' ')
        .next()
        .unwrap_or("")
        .chars()
        .collect();
    glyph > 0.0
        && (first.row - second.row).abs() < glyph
        && first.left < second.left
        && word.last() == Some(&'.')
        && 0 < word.len() - 1
        && word.len() - 1 <= ABBREVIATION
        && word[..word.len() - 1].iter().all(|&c| py::is_alpha(c))
        && second.text.chars().next().is_some_and(py::is_digit)
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

/// Pieces read row by row, each row's first moved to just after the words above it where it
/// is drawn under them, starting where they end with nothing else of their row over it: the
/// battle's move panel writes PHYSICAL under CATEGORY, and POWER 80 on CATEGORY's row, "CATEGORY
/// POWER 80 PHYSICAL" (or_move-learning, 2026-10-18).
pub fn under_their_labels(pieces: Vec<ScreenText>) -> Vec<ScreenText> {
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
/// 58, Bubble Beam", where opening it read each down (or_organize_boxes).
pub fn columns(pieces: &[ScreenText]) -> Vec<ScreenText> {
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
pub fn with_row_mates(
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
/// A panel, not a whole layout. The battle draws both Pokemon's HP boxes in
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
pub fn panels_of(pieces: &[ScreenText], on_screen: &[ScreenText]) -> Vec<Vec<ScreenText>> {
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
/// (pokedex2.ctm). A panel with no words from two groups side by side on
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
    // A word no other word of the panel reaches over or under is a column of its own: Head It's
    // results draw "TOTAL:" and the total right of the score and bonuses, and read down as one
    // column they came between "SCORE:" and its points (or_amie_head_it_hard).
    if mine.iter().any(|piece| {
        !mine.iter().any(|other| {
            !std::ptr::eq(*other, *piece) && other.left < piece.right && piece.left < other.right
        })
    }) {
        return false;
    }
    !mine.iter().any(|piece| {
        mine.iter()
            .any(|other| side_by_side(piece, other) && (piece.row - other.row).abs() > ONE_LINE)
    })
}

/// How many rows the pieces are drawn on.
pub fn row_count(pieces: &[ScreenText]) -> usize {
    let mut rows: Vec<&ScreenText> = Vec::new();
    for piece in pieces {
        if piece.glyph > 0.0 && !rows.iter().any(|row| side_by_side(row, piece)) {
            rows.push(piece);
        }
    }
    rows.len()
}

/// On the same row, so one thing to say however the layout groups them?
pub fn side_by_side(one: &ScreenText, other: &ScreenText) -> bool {
    if one.glyph <= 0.0 || other.glyph <= 0.0 {
        return false;
    }
    (one.row - other.row).abs() < min_of(one.glyph, other.glyph) / 2.0
}

/// The fewest words a sentence has, a question or a message: more than a label's
/// (`LABEL_WORDS`) or an item's.
pub const SENTENCE_WORDS: usize = 4;
/// How a shorter sentence ends, from two words on: the Box's "Bye-bye, Seedot!"
/// (or_releasing_pokemon). A word alone is a mark, as the Bag's "ABLE!" beside a Pokémon is.
pub const SENTENCE_ENDS: [&str; 6] = ["!", "?", "\u{ff01}", "\u{ff1f}", "\u{3002}", "\u{2026}"];

/// Is this a sentence rather than a label, a value or an item's name? Four words, or fewer
/// ending as a sentence does: from two words on with "!" or "?", from three with a full stop,
/// since two are as often a label shortened, "Dex No.", "ID No.". The Box said nothing of
/// "Seedot was released." and "Bye-bye, Seedot!", taken for the menu's own words
/// (or_releasing_pokemon).
pub fn sentence(piece: &ScreenText) -> bool {
    let words = py::split(&piece.text).len();
    words >= SENTENCE_WORDS
        || (words >= 2 && SENTENCE_ENDS.iter().any(|end| piece.text.ends_with(end)))
        || (words >= 3 && piece.text.ends_with('.'))
}

/// Each changed piece with what sits beside it in its group on its row, each row once.
pub fn rows_of(
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
