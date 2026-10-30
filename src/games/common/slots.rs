//! Slots: the places a named cursor rests on where no words are, said by where they are.
//!
//! The PC Box's pointer, `cursol01.bclim`, walks its grid of Pokémon and its party as it walks its
//! menus, but a slot holds an icon and no words (spike 129). Each slot is an `nw::lyt::Bounding`
//! touch area, and the pointer comes to rest exactly on the touch area's place:
//!
//! - **A grid**, the box: touch areas of one size under the layout's root, thirty of them in six
//!   columns and five rows, 30 units apart. A slot is said by the words just above the grid, the
//!   box's name, and its row and column: "BOX 1, row 2, column 3".
//! - **A list**, the party: the touch areas a panel holds, six in two columns, the second column 20
//!   units lower than the first, numbered in reading order: "Party 2". The Battle Box's six lie so
//!   under the root, beside the box's thirty still shown behind its picture, and are said by the
//!   words above them: "BATTLE BOX 2" (or_battle_box).
//! - **A mark**, of the six a Pokémon can be given: a touch area beside a picture textured
//!   `box_mark_01` to `box_mark_06`, said by the sign the picture shows and whether it is set: a
//!   mark set is drawn solid, one not set half-transparent (or/movie11_pokedex_and_box, where
//!   setting ▲ took its picture from alpha 127 to 255, and the save holds it set).
//!
//! A slot holding a Pokémon has its icon (`poke_icon.bclim`) exactly on its place; the one the
//! player carries rides 10 units above it, so an empty slot is one no icon sits exactly on. The
//! top screen's summary of what is under the pointer changes as it moves, and is read after the
//! slot as its details (the user, 2026-09-27: where, then the summary; where, then "empty").
//! While a Pokémon is carried the summary is the carried one's, whatever slot the pointer is on,
//! so a slot is said alone, with the name of the Pokémon in it where the game's own reader knows
//! it: "BOX 1, row 1, column 2, erza" (Orion, or_organize_boxes).

use std::collections::{BTreeSet, HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::{LayoutFile, LayoutFiles};
use crate::games::common::lyt;
use crate::games::common::pictures::{OFF, ON, mark, words};
use crate::games::common::screen::ScreenText;
use crate::py::{self, Error, PySet, Result, sorted_by_lt};

pub const PARTY: &str = "slot-party";
pub const GRID: &str = "slot-grid";
pub const TITLED_GRID: &str = "slot-titled-grid";
pub const TITLED_LIST: &str = "slot-titled-list";
pub const EMPTY_SLOT: &str = "slot-empty";
/// pa3ds's own words for a slot, "Party 2", "BOX 1, row 2, column 3", "BATTLE BOX 2", one
/// holding no Pokémon, and one with who is in it: the game names neither the party's slots nor a
/// box's rows.
pub const HELD_SLOT: &str = "slot-held";
/// Which slot: its words' kind (`PARTY`, `GRID`, `TITLED_GRID`, `TITLED_LIST`), the title above
/// its group, and its place in the group from 0 in reading order.
pub type Held = (&'static str, Option<String>, i64);
/// The name of the Pokémon in a slot (`Held`), or None.
pub type Occupant = Rc<dyn Fn(&str, Option<&str>, i64) -> Result<Option<String>>>;
/// The alpha from which a mark's picture is drawn as set: 255 set, 127 not (spike 129).
pub const SET_ALPHA: i64 = 192;
/// The texture a slot's Pokémon is drawn with.
pub const ICON: &str = "poke_icon";
/// Units within which a pointer rests on a touch area, and an icon sits on a slot: both exact.
pub const SAME_SPOT: f64 = 1.0;
/// How far above the slot the pointer rests on the icon of the Pokémon carried rides (spike 129).
pub const CARRIED: f64 = 10.0;

/// `list.index(item)`, raising `ValueError` where the item is not in the list.
#[track_caller]
fn index_of(items: &[i64], item: i64) -> Result<usize> {
    items
        .iter()
        .position(|&each| each == item)
        .ok_or_else(|| Error::value(format!("{item} is not in list")))
}

/// The touch area a cursor resting at (x, y) is on, what to say of it, and whether it is said
/// alone, the summary left unsaid: an empty slot, or any while a Pokémon is carried, said with
/// who is in it by `occupant`. None where it rests on no slot, as on a lone button's touch area;
/// no words for a slot behind the Battle Box's, which nothing reaches.
pub fn slot(
    layouts: &Layouts,
    file: Option<&LayoutFile>,
    root: i64,
    x: f64,
    y: f64,
    on_screen: &[ScreenText],
    occupant: Option<&Occupant>,
) -> Result<Option<(i64, String, bool)>> {
    let tree = layouts.tree(root);
    let mut touch: Option<i64> = None;
    for (pane, kind) in &tree {
        if kind == lyt::BOUNDING_CLASS
            && on(place(layouts, *pane)?, (x, y))
            && shown(layouts, *pane)?
        {
            touch = Some(*pane);
            break;
        }
    }
    let Some(touch) = touch else {
        return Ok(None);
    };
    let parent = u32(&layouts.pane(touch)?, lyt::PARENT)?;
    if let Some(file) = file {
        let sign = mark_of(layouts, file, root, parent, touch)?;
        if let Some(sign) = sign {
            return Ok(Some((touch, sign, false)));
        }
    }
    let size = size(layouts, touch)?;
    let mut group: Vec<i64> = Vec::new();
    for pane in layouts.children(root, parent) {
        if tree.get(&pane).map(String::as_str) == Some(lyt::BOUNDING_CLASS)
            && shown(layouts, pane)?
            && on(self::size(layouts, pane)?, size)
        {
            group.push(pane);
        }
    }
    let mut places: IndexMap<i64, (f64, f64)> = IndexMap::new();
    for &pane in &group {
        places.insert(pane, place(layouts, pane)?);
    }
    let grids = grids(&places)?;
    let icons = match file {
        None => Vec::new(),
        Some(file) => icons(layouts, file, root)?,
    };
    let grid_of = |pane: i64| -> Result<Option<Rc<PySet>>> {
        grids
            .get(&pane)
            .cloned()
            .ok_or_else(|| Error::key(pane.to_string()))
    };
    let mut found: Option<(String, Held)> = None;
    let mut listed: Vec<i64> = group.clone();
    if parent != root {
        found = in_grid(
            layouts,
            touch,
            grid_of(touch)?.as_deref(),
            &places,
            root,
            on_screen,
        )?;
    } else {
        // The Battle Box's six lie under the root beside the box's thirty, and both stay shown
        // whichever is drawn, overlapping: a tap on one was taken for the other (or_battle_box).
        // The one whose slots hold the icons is on screen. ponytail: an empty Battle Box on
        // screen reads as the box behind it; the wallpaper drawn, `box_wp_battle_box`, would tell.
        let mut loose: Vec<i64> = Vec::new();
        for &pane in &group {
            if grid_of(pane)?.is_none() {
                loose.push(pane);
            }
        }
        let battle = loose.iter().any(|pane| holds_icon(&icons, places[pane]));
        if !loose.contains(&touch) {
            if battle {
                return Ok(Some((touch, String::new(), true)));
            }
            found = in_grid(
                layouts,
                touch,
                grid_of(touch)?.as_deref(),
                &places,
                root,
                on_screen,
            )?;
        }
        listed = if battle { loose } else { Vec::new() };
    }
    if found.is_none() && listed.len() > 1 && listed.contains(&touch) {
        // The key is `(-y, x)`: a tuple is smaller by its first element that differs.
        let order = sorted_by_lt(listed.clone(), |a, b| {
            if -places[a].1 != -places[b].1 {
                return -places[a].1 < -places[b].1;
            }
            places[a].0 < places[b].0
        });
        let number = index_of(&order, touch)? + 1;
        let title = if parent != root {
            None
        } else {
            let at: Vec<(f64, f64)> = listed.iter().map(|pane| places[pane]).collect();
            title(&at, self::size(layouts, touch)?.0 / 2.0, root, on_screen)
        };
        let index = number as i64 - 1;
        found = if parent != root {
            Some((own(PARTY, &[("number", &number)]), (PARTY, None, index)))
        } else {
            title.filter(|title| !title.is_empty()).map(|title| {
                (
                    own(TITLED_LIST, &[("title", &title), ("number", &number)]),
                    (TITLED_LIST, Some(title), index),
                )
            })
        };
    }
    let Some((r#where, held)) = found else {
        return Ok(None);
    };
    let place = place(layouts, touch)?;
    if !holds_icon(&icons, place) {
        return Ok(Some((touch, own(EMPTY_SLOT, &[("slot", &r#where)]), true)));
    }
    if !holds_icon(&icons, (place.0, place.1 + CARRIED)) {
        return Ok(Some((touch, r#where, false)));
    }
    // The summary is the carried one's, so the Pokémon in the slot is named from the save.
    let name = match occupant {
        None => None,
        Some(occupant) => occupant(held.0, held.1.as_deref(), held.2)?,
    };
    Ok(Some((
        touch,
        match name.filter(|name| !name.is_empty()) {
            Some(name) => own(HELD_SLOT, &[("slot", &r#where), ("name", &name)]),
            None => r#where,
        },
        true,
    )))
}

/// A lone button of another layout on the cursor's screen whose touch area the cursor rests
/// on the middle of, what to say of it, and that it is said alone: the words of the picture
/// beside its touch area. The Box's pointer, down from its grid, rests on the bottom bar's
/// `change_box.bclim`, in `touch_bar_lower`, a layout many screens share, where the Box's own
/// leaves an empty pane (spike 143, movie12). A lone button of the cursor's own layout stays
/// unsaid: the Box's modes are pictures named `mode01` to `mode03`, and the pointer never went to
/// them.
pub fn button(
    layouts: &Layouts,
    files: &LayoutFiles,
    root: i64,
    x: f64,
    y: f64,
    top: bool,
) -> Result<Option<(i64, String, bool)>> {
    for other in layouts.roots_set().difference(&PySet::of([root])).iter() {
        let tree = layouts.tree(other);
        for (touch, kind) in &tree {
            let touch = *touch;
            if kind != lyt::BOUNDING_CLASS || !on(place(layouts, touch)?, (x, y)) {
                continue;
            }
            let placement = layouts.placement(touch)?;
            let Some(placement) = placement else {
                continue;
            };
            if !placement.shown || placement.top != top {
                continue;
            }
            let Some(file) = files.matching(&layouts.pane_names(other)) else {
                continue;
            };
            let parent = u32(&layouts.pane(touch)?, lyt::PARENT)?;
            for pane in layouts.children(other, parent) {
                if tree.get(&pane).map(String::as_str) != Some(lyt::PICTURE_CLASS)
                    || !shown(layouts, pane)?
                {
                    continue;
                }
                if let Some(textures) = file.textures.get(&name(layouts, pane)?)
                    && !textures.is_empty()
                {
                    return Ok(Some((touch, words(&textures[0]), true)));
                }
            }
        }
    }
    Ok(None)
}

/// The slot's row and column, in its grid (`grids`), after the words just above the grid, and
/// which slot it is; None outside one.
fn in_grid(
    layouts: &Layouts,
    touch: i64,
    grid: Option<&PySet>,
    places: &IndexMap<i64, (f64, f64)>,
    root: i64,
    on_screen: &[ScreenText],
) -> Result<Option<(String, Held)>> {
    let Some(grid) = grid else {
        return Ok(None);
    };
    let mut xs: BTreeSet<i64> = BTreeSet::new();
    for pane in grid.iter() {
        xs.insert(py::round(places[&pane].0)?);
    }
    let mut ys: BTreeSet<i64> = BTreeSet::new();
    for pane in grid.iter() {
        ys.insert(py::round(places[&pane].1)?);
    }
    let xs: Vec<i64> = xs.into_iter().collect();
    let ys: Vec<i64> = ys.into_iter().rev().collect();
    let (x, y) = places[&touch];
    let row = index_of(&ys, py::round(y)?)? + 1;
    let column = index_of(&xs, py::round(x)?)? + 1;
    let at: Vec<(f64, f64)> = grid.iter().map(|pane| places[&pane]).collect();
    let title = title(&at, size(layouts, touch)?.0 / 2.0, root, on_screen);
    let titled = title.as_deref().is_some_and(|text| !text.is_empty());
    let held: Held = (
        if titled { TITLED_GRID } else { GRID },
        title.clone(),
        (row as i64 - 1) * xs.len() as i64 + column as i64 - 1,
    );
    Ok(Some(match title.filter(|text| !text.is_empty()) {
        Some(title) => (
            own(
                TITLED_GRID,
                &[("title", &title), ("row", &row), ("column", &column)],
            ),
            held,
        ),
        None => (own(GRID, &[("row", &row), ("column", &column)]), held),
    }))
}

/// Each touch area -> the touch areas sharing rows and columns with it, where they fill a grid
/// two by two or more, or None where they do not. Each grid is walked once for all its members:
/// walked again for each of the Box's 36 under its root, as each of its slots was named, it made
/// touch mode's every key there wait a quarter of a second (or_drag_attempts).
///
/// Only looked up, never iterated, so a `HashMap`; each grid is one `Rc` its members share, as
/// Python's dict holds one set for them all.
fn grids(places: &IndexMap<i64, (f64, f64)>) -> Result<HashMap<i64, Option<Rc<PySet>>>> {
    let mut grids: HashMap<i64, Option<Rc<PySet>>> = HashMap::new();
    for &touch in places.keys() {
        if grids.contains_key(&touch) {
            continue;
        }
        // `grid` is a set of ints iterated by its callers, so it is kept as the order its members
        // were added in and laid out as Python lays it out.
        let mut grid: Vec<i64> = vec![touch];
        let mut frontier: Vec<i64> = vec![touch];
        while let Some(next) = frontier.pop() {
            let here = *places
                .get(&next)
                .ok_or_else(|| Error::key(next.to_string()))?;
            for (&pane, place) in places {
                if !grid.contains(&pane)
                    && ((place.0 - here.0).abs() <= SAME_SPOT
                        || (place.1 - here.1).abs() <= SAME_SPOT)
                {
                    grid.push(pane);
                    frontier.push(pane);
                }
            }
        }
        let members = Rc::new(PySet::of(grid.iter().copied()));
        let mut xs: HashSet<i64> = HashSet::new();
        let mut ys: HashSet<i64> = HashSet::new();
        for pane in members.iter() {
            xs.insert(py::round(places[&pane].0)?);
        }
        for pane in members.iter() {
            ys.insert(py::round(places[&pane].1)?);
        }
        let filled = xs.len() >= 2 && ys.len() >= 2 && xs.len() * ys.len() == grid.len();
        for pane in members.iter() {
            grids.insert(pane, filled.then(|| members.clone()));
        }
    }
    Ok(grids)
}

/// The words just above these slots, between their first and last column: the box's name.
fn title(places: &[(f64, f64)], width: f64, root: i64, on_screen: &[ScreenText]) -> Option<String> {
    // Python's `min` and `max`: the first unless a later one is strictly less, or greater.
    let mut left = places[0].0;
    let mut right = places[0].0;
    let mut top = places[0].1;
    for &(x, y) in &places[1..] {
        if x < left {
            left = x;
        }
        if x > right {
            right = x;
        }
        if y > top {
            top = y;
        }
    }
    let (left, right) = (left - width, right + width);
    let mut above: Option<&ScreenText> = None;
    for piece in on_screen {
        if piece.root == root
            && piece.glyph > 0.0
            && piece.row > top
            && left <= (piece.left + piece.right) / 2.0
            && (piece.left + piece.right) / 2.0 <= right
            && above.is_none_or(|first| piece.row < first.row)
        {
            above = Some(piece);
        }
    }
    above.map(|piece| piece.text.clone())
}

/// The sign of the mark whose picture lies within this touch area, if one does: beside it in the
/// Box, and in the summary the picture it hangs from (movie22).
fn mark_of(
    layouts: &Layouts,
    file: &LayoutFile,
    root: i64,
    parent: i64,
    touch: i64,
) -> Result<Option<String>> {
    let (x, y) = place(layouts, touch)?;
    let (width, height) = size(layouts, touch)?;
    for pane in std::iter::once(parent).chain(layouts.children(root, parent)) {
        if layouts.tree(root).get(&pane).map(String::as_str) != Some(lyt::PICTURE_CLASS) {
            continue;
        }
        let (px, py) = place(layouts, pane)?;
        if (px - x).abs() > width / 2.0 || (py - y).abs() > height / 2.0 {
            continue;
        }
        let Some(textures) = file.textures.get(&name(layouts, pane)?) else {
            continue;
        };
        for texture in textures {
            if let Some(sign) = mark(texture) {
                let alpha = *layouts
                    .pane(pane)?
                    .get(layouts.shape.alpha as usize)
                    .ok_or_else(|| Error::index("index out of range"))?;
                return Ok(Some(own(
                    if i64::from(alpha) >= SET_ALPHA {
                        ON
                    } else {
                        OFF
                    },
                    &[("mark", &sign)],
                )));
            }
        }
    }
    Ok(None)
}

/// Where the layout draws a Pokémon's icon, looked for once a slot.
fn icons(layouts: &Layouts, file: &LayoutFile, root: i64) -> Result<Vec<(f64, f64)>> {
    let mut out: Vec<(f64, f64)> = Vec::new();
    for (pane, kind) in &layouts.tree(root) {
        let pane = *pane;
        if kind != lyt::PICTURE_CLASS {
            continue;
        }
        let name = name(layouts, pane)?;
        if !file
            .textures
            .get(&name)
            .is_some_and(|textures| textures.iter().any(|texture| texture.starts_with(ICON)))
        {
            continue;
        }
        if shown(layouts, pane)? {
            out.push(place(layouts, pane)?);
        }
    }
    Ok(out)
}

/// Does a Pokémon's icon sit exactly on this place?
fn holds_icon(icons: &[(f64, f64)], at: (f64, f64)) -> bool {
    icons.iter().any(|&icon| on(icon, at))
}

fn shown(layouts: &Layouts, pane: i64) -> Result<bool> {
    Ok(layouts
        .placement(pane)?
        .is_some_and(|placement| placement.shown))
}

fn place(layouts: &Layouts, pane: i64) -> Result<(f64, f64)> {
    let head = layouts.pane(pane)?;
    Ok((f32(&head, layouts.shape.x)?, f32(&head, layouts.shape.y)?))
}

fn size(layouts: &Layouts, pane: i64) -> Result<(f64, f64)> {
    let head = layouts.pane(pane)?;
    Ok((
        f32(&head, layouts.shape.width)?,
        f32(&head, layouts.shape.height)?,
    ))
}

fn on(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() <= SAME_SPOT && (a.1 - b.1).abs() <= SAME_SPOT
}

fn name(layouts: &Layouts, pane: i64) -> Result<String> {
    let raw = layouts.name(pane)?;
    let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
    Ok(py::decode_ascii(&raw[..end]))
}
