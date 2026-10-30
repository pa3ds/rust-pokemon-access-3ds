//! Names for doors and warps out of what the maps show (spike 107): the plate beside a building's
//! door, and on Lumiose's streets the street a warp comes out on.
//!
//! A **plate** is a sign the zone's script shows in the plate's style (`script.PLATE`): "Sycamore
//! Pokémon Lab", "Café Soleil", "Santalune City Pokémon Gym Leader: Viola". It names the door it
//! stands by, which is the nearest door into a building of the town's own, within `NEAR` cells
//! (`FAR` for a plate of one line, which the game puts by a garden gate as far as 12 cells from the
//! house) and with no other door nearly as near. What it says is its first page, the lines run
//! together, since the Japanese and Korean put a building's second word on a line of its own; a
//! plate that says who the player is, or that shows one of two messages as the story goes, names
//! nothing. On a street the plate and the door are on one line, measured in the numbers the game
//! places both by. Over X, 45 doors, every one of them right in English, and in all seven languages
//! the same doors, since the pairing never reads a word.
//!
//! The **street marks** are a table a Lumiose zone (`a/2/5/5` in X), each mark a stretch of a line
//! and a number: 1 to 12 one of Lumiose's street names, two lines a name in their text file, 13 none.
//! A mark covers a whole boulevard, or one cell at an alley's mouth; a warp between two of Lumiose's
//! zones is named for the mark nearest where it comes out, on the line it comes out on, which is
//! the warp its record's +0x02 gives in the zone it goes to.
//!
//! A warp into one of X's **gates** is named for the gate and the place beyond it, "Lumiose Gate to
//! Route 4": the gate's zone has an area kind of its own (`zonetable.GATE`), and the place beyond
//! is the one its warps reach that is neither the gate's nor the warp's own (`gate_names`).

use std::collections::HashSet;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{s16, u16, u32};
use crate::core::pathfind::Cell;
use crate::games::common::text::{self, Token};
use crate::games::gen6::field::{rail, zonedata};
use crate::games::gen6::rom::script;
use crate::py::{self, Error, Kind, Result};

pub const NEAR: i64 = 6;
pub const FAR: i64 = 12;
/// How much further the next door must be than the nearest for a plate to be the nearest's.
pub const APART: f64 = 1.5;
/// The quotation marks a plate puts round a name on a line of its own (`_named`).
pub const QUOTES: &str = "「」『』“”\"";
/// A zone-table row's +0x06: the zone's file in the story text archive, which its script's
/// messages are lines of.
pub const INFO_STORY: i64 = 0x06;
pub const MARK: i64 = 0x18;
pub const MARK_STREET: i64 = 0x02;
pub const MARK_PLACEMENT: i64 = 0x08;
pub const MARK_LINE: i64 = 0x0C;
pub const MARK_PLACE: i64 = 0x10;

/// A zone as the ROM keeps it, decompressed: the file, its furniture and its warps.
pub type Zone = (Vec<u8>, Vec<Vec<u8>>, Vec<Vec<u8>>);

/// `items[index]`: a negative index counts from the end, and one out of range raises.
#[track_caller]
fn item<T>(items: &[T], index: i64) -> Result<&T> {
    let at = if index < 0 {
        index + items.len() as i64
    } else {
        index
    };
    if at < 0 || at >= items.len() as i64 {
        return Err(Error::index("list index out of range"));
    }
    Ok(&items[at as usize])
}

/// The zones the player walks about in as outdoors: each zone of the matrix, its own parent;
/// and one of its parent's place and area kind whose records are placed on streets, as
/// Lumiose's other zones are, or that a building's way out comes out into, as Coumarine's lower
/// town is. A Gym whose puzzle is on rails, as Coumarine's and Anistar's are, has a kind of its
/// own, and so do the rooms of a building.
pub fn outdoors(zones: &IndexMap<i64, Zone>, table: &[u8]) -> Result<HashSet<i64>> {
    let mut out: HashSet<i64> = HashSet::new();
    for &zone in zones.keys() {
        if row(
            table,
            zone,
            crate::games::gen6::field::zonetable::INFO_PARENT,
        )? == zone
        {
            out.insert(zone);
        }
    }

    let of_the_town = |zone: i64| -> Result<bool> {
        let parent = row(
            table,
            zone,
            crate::games::gen6::field::zonetable::INFO_PARENT,
        )?;
        Ok(place(table, zone)? == place(table, parent)?
            && row(table, zone, crate::games::gen6::field::zonetable::INFO_KIND)?
                == row(
                    table,
                    parent,
                    crate::games::gen6::field::zonetable::INFO_KIND,
                )?)
    };

    for (&zone, (_data, furniture, warps)) in zones {
        let mut on_streets = false;
        for record in furniture.iter().chain(warps) {
            if zonedata::on_rail(record)? {
                on_streets = true;
                break;
            }
        }
        if on_streets && of_the_town(zone)? {
            out.insert(zone);
        }
        for record in warps {
            let to = u16(record, zonedata::WARP_TO)?;
            if u16(record, zonedata::WARP_KIND)? == zonedata::WARP_OUT
                && zones.contains_key(&to)
                && of_the_town(to)?
            {
                out.insert(to);
            }
        }
    }
    Ok(out)
}

/// (zone, warp slot) -> what the plate beside that door says, for every door a plate names.
/// `story` gives a file of the story text archive, in the player's language, as its lines.
/// `facilities` is place -> what its town's list calls each facility: a plate holding one of
/// them is named by it alone (`_facility`). `every` is every facility the list's text names,
/// a town's or not: a plate with a line that is one of them, and that does not name the
/// building otherwise, is named by it (`_named`).
pub fn plate_names(
    zones: &IndexMap<i64, Zone>,
    table: &[u8],
    story: &dyn Fn(i64) -> Result<Vec<Vec<u8>>>,
    outside: &HashSet<i64>,
    facilities: Option<&IndexMap<i64, Vec<String>>>,
    every: &[String],
) -> Result<IndexMap<(i64, i64), String>> {
    let no_facilities = IndexMap::new();
    let facilities = facilities.unwrap_or(&no_facilities);
    let mut named: IndexMap<(i64, i64), (i64, String)> = IndexMap::new();
    let mut inside: Vec<i64> = outside.iter().copied().collect();
    inside.sort();
    for zone in inside {
        let Some((data, furniture, warps)) = zones.get(&zone) else {
            continue;
        };
        let code = script_of(data)?;
        let lines = story(row(table, zone, INFO_STORY)?)?;
        let Some(code) = code.filter(|_| !lines.is_empty()) else {
            continue;
        };
        let mut doors: Vec<(i64, &Vec<u8>)> = Vec::new();
        for (slot, record) in warps.iter().enumerate() {
            let to = u16(record, zonedata::WARP_TO)?;
            if zones.contains_key(&to) && !outside.contains(&to) {
                doors.push((slot as i64, record));
            }
        }
        if doors.is_empty() {
            continue;
        }
        let listed = facilities
            .get(&place(table, zone)?)
            .map_or(&[][..], Vec::as_slice);
        let mut found: Vec<(&Vec<u8>, String, i64)> = Vec::new();
        for (_index, record, line) in plates_of(&code, furniture, &lines)? {
            let words = first_page(item(&lines, line)?)?;
            let Some(mut words) = words.filter(|words| !words.is_empty()) else {
                continue;
            };
            let said = words_of(item(&lines, line)?)?;
            let name = self::named(item(&lines, line)?, every)?;
            if first_lines(item(&lines, line)?)?.contains('\n') {
                words = facility(&said, listed)
                    .or_else(|| name.clone().filter(|name| !name.is_empty()))
                    .unwrap_or(words);
            } else if let Some(name) = name.filter(|name| !name.is_empty())
                && !holds(&words, &name)
            {
                words = name;
            }
            found.push((record, words, if !said.contains('\n') { FAR } else { NEAR }));
        }
        for (record, name) in story_plates(&code, furniture, &lines, listed)? {
            found.push((record, name, NEAR));
        }
        for (record, words, far) in found {
            let mut nearest: Vec<(i64, i64)> = Vec::new();
            for &(slot, door) in &doors {
                if let Some(d) = apart(record, door)? {
                    nearest.push((d, slot));
                }
            }
            nearest.sort();
            if nearest.is_empty() || nearest[0].0 > far {
                continue;
            }
            let ((d, slot), rest) = (nearest[0], &nearest[1..]);
            if !rest.is_empty() && (rest[0].0 as f64) < APART * d as f64 {
                continue;
            }
            let to = u16(item(warps, slot)?, zonedata::WARP_TO)?;
            if place(table, to)? != place(table, zone)? {
                continue;
            }
            if !named.contains_key(&(zone, slot)) || named[&(zone, slot)].0 > d {
                named.insert((zone, slot), (d, words));
            }
        }
    }
    Ok(named
        .into_iter()
        .map(|(door, (_d, words))| (door, words))
        .collect())
}

/// Each furniture record the zone's script shows a plate for in one of several lines as the
/// story goes, with the one facility of its town's list that every one of them holds: Stern's
/// Shipyard's plate is one of three notices, each under "Stern's Shipyard", and no other plate
/// or table names its door (a user, 2026-10-26).
fn story_plates<'a>(
    code: &script::Script,
    furniture: &'a [Vec<u8>],
    lines: &[Vec<u8>],
    listed: &[String],
) -> Result<Vec<(&'a Vec<u8>, String)>> {
    let mut out: Vec<(&Vec<u8>, String)> = Vec::new();
    for record in furniture {
        let shown: HashSet<i64> =
            script::sign_lines(code, u16(record, zonedata::FURNITURE_SCRIPT)?)?
                .into_iter()
                .filter(|&(_, style)| style == script::PLATE)
                .map(|(line, _)| line)
                .collect();
        if shown.len() < 2 || shown.iter().any(|&line| line >= lines.len() as i64) {
            continue;
        }
        let refs: Vec<&String> = listed.iter().collect();
        let mut held: HashSet<&String> = refs.iter().copied().collect();
        for &line in &shown {
            let said = words_of(item(lines, line)?)?;
            let one = held_in(&said, &refs);
            held.retain(|name| one.contains(name));
        }
        if held.len() == 1 {
            out.push((record, held.into_iter().next().cloned().expect("one")));
        }
    }
    Ok(out)
}

/// (zone, warp slot) -> the facility of its town's list a landmark's entrance is, for the
/// entrances no plate or table has named. `entrances` is the warp kinds each used by one
/// building's way in alone (`RomLayout.entrances`), `listed` is place -> what its town's list
/// calls each facility, `doors` the names already found (`plate_names`, `GameText.doors`).
///
/// A place with exactly one such entrance in from its outdoors into a zone of its own place, and
/// no name for it yet, has it named from the facilities left over. A facility is **claimed** when
/// a door name of the place holds it, spaces left out (`_held`). Then:
///
/// 1. The plates of the place, the furniture its zones' scripts show in a plate's style, that
///    each hold exactly one unclaimed facility: if together they name one facility, it is the
///    entrance's. Devon Corporation's plate, "The Devon Corporation / For all your living
///    needs...", stands 18 cells from its doors, past the Gym's, so `plate_names` cannot pair it
///    by distance, but holds "Devon Corporation" and no other facility.
/// 2. Only where no plate holds exactly one: the one facility of the list neither claimed nor
///    held by any plate. Route 112's list has "Cable Car Base Station" and no plate holds it.
///
/// Rules that find two facilities, or none, leave the entrance unnamed: Lilycove's museum has no
/// plate but the town's sign, which holds "Harbor" and "Lilycove Museum" both, so each is held
/// and neither is left over. An entrance into another place, Cave of Origin's and New Mauville's,
/// is left to the place it leads to. Against the ROM (2026-10-06) Rustboro's entrance is "Devon
/// Corporation" and Route 112's "Cable Car Base Station", Lilycove's has none, and every
/// entrance the plates named is as it was.
#[allow(clippy::too_many_arguments)]
pub fn entrance_names(
    zones: &IndexMap<i64, Zone>,
    table: &[u8],
    story: &dyn Fn(i64) -> Result<Vec<Vec<u8>>>,
    outside: &HashSet<i64>,
    listed: &IndexMap<i64, Vec<String>>,
    doors: &IndexMap<(i64, i64), String>,
    entrances: &HashSet<i64>,
) -> Result<IndexMap<(i64, i64), String>> {
    let mut outdoors: Vec<i64> = outside.iter().copied().collect();
    outdoors.sort();
    let mut entering: IndexMap<i64, Vec<(i64, i64)>> = IndexMap::new();
    for &zone in &outdoors {
        let Some(found) = zones.get(&zone) else {
            continue;
        };
        for (slot, record) in found.2.iter().enumerate() {
            let to = u16(record, zonedata::WARP_TO)?;
            if entrances.contains(&u16(record, zonedata::WARP_KIND)?)
                && zones.contains_key(&to)
                && !outside.contains(&to)
                && place(table, to)? == place(table, zone)?
                && !doors.contains_key(&(zone, slot as i64))
            {
                entering
                    .entry(place(table, zone)?)
                    .or_default()
                    .push((zone, slot as i64));
            }
        }
    }
    let mut out: IndexMap<(i64, i64), String> = IndexMap::new();
    for (&here, found) in &entering {
        let facilities: HashSet<&String> = listed
            .get(&here)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .filter(|name| !name.is_empty())
            .collect();
        if found.len() != 1 || facilities.is_empty() {
            continue;
        }
        let mut names: Vec<&String> = Vec::new();
        for (&(zone, _slot), name) in doors {
            if place(table, zone)? == here {
                names.push(name);
            }
        }
        let claimed: HashSet<&String> = facilities
            .iter()
            .copied()
            .filter(|name| names.iter().any(|said| holds(said, name)))
            .collect();
        let mut held: Vec<HashSet<&String>> = Vec::new();
        for &zone in &outdoors {
            let Some((data, furniture, _warps)) = zones.get(&zone) else {
                continue;
            };
            if place(table, zone)? != here {
                continue;
            }
            let code = script_of(data)?;
            let lines = story(row(table, zone, INFO_STORY)?)?;
            if let Some(code) = code {
                for (_index, _record, line) in plates_of(&code, furniture, &lines)? {
                    held.push(held_in(&words_of(item(&lines, line)?)?, &facilities));
                }
            }
        }
        let unclaimed: Vec<HashSet<&String>> = held
            .iter()
            .map(|plate| plate.difference(&claimed).copied().collect())
            .collect();
        let mut named: HashSet<&String> = unclaimed
            .iter()
            .filter(|left| left.len() == 1)
            .flatten()
            .copied()
            .collect();
        if named.is_empty() {
            named = facilities
                .iter()
                .copied()
                .filter(|name| {
                    !claimed.contains(name) && !held.iter().any(|plate| plate.contains(name))
                })
                .collect();
        }
        if named.len() == 1 {
            let name = named.into_iter().next().expect("one facility");
            out.insert(found[0], name.clone());
        }
    }
    Ok(out)
}

/// (zone, warp slot) -> the street a warp between two zones of one place comes out on, where
/// the zone it goes to has marks on the line it comes out on. `marks` is the street marks'
/// files, a zone's each, and `names` the lines of their text file.
pub fn street_names(
    zones: &IndexMap<i64, Zone>,
    table: &[u8],
    marks: &[Vec<u8>],
    names: &[String],
) -> Result<IndexMap<(i64, i64), String>> {
    let by_zone = marks_by_zone(zones, marks)?;
    let mut out: IndexMap<(i64, i64), String> = IndexMap::new();
    for (&zone, (_data, _furniture, warps)) in zones {
        for (slot, record) in warps.iter().enumerate() {
            let to = u16(record, zonedata::WARP_TO)?;
            if !by_zone.contains_key(&to)
                || !zones.contains_key(&to)
                || place(table, to)? != place(table, zone)?
            {
                continue;
            }
            let arrivals = &zones[&to].2;
            let back = u16(record, crate::games::gen6::field::zonedata::WARP_TO_SLOT)?;
            if back >= arrivals.len() as i64 || !zonedata::on_rail(&arrivals[back as usize])? {
                continue;
            }
            let (line, a, x, width, height) = rail::placed(&arrivals[back as usize])?;
            let place_at = [a, x, width, height];
            let named: Vec<&(i64, i64, [i64; 4])> = by_zone[&to]
                .iter()
                .filter(|(_on, street, _mark)| 0 < *street && *street <= names.len() as i64 / 2)
                .collect();
            let mut near: Vec<(i64, i64)> = Vec::new();
            for &&(on, street, mark) in &named {
                if on == line {
                    near.push((gap(&place_at, &mark), street));
                }
            }
            near.sort();
            // North Boulevard's warps come out on a line with no mark of its own, in a zone
            // whose marks all name that one street.
            let streets: HashSet<i64> = named.iter().map(|(_on, street, _mark)| *street).collect();
            if near.is_empty() && streets.len() == 1 {
                near = vec![(0, named[0].1)];
            }
            if !near.is_empty() {
                let street = near[0].1;
                let words = py::slice(names, 2 * street - 2, 2 * street)
                    .iter()
                    .filter(|word| !word.is_empty())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" ");
                if !words.is_empty() {
                    out.insert((zone, slot as i64), words);
                }
            }
        }
    }
    Ok(out)
}

/// (zone, warp slot) -> the place index of the gate a warp goes into and of the one place
/// beyond it: the one its warps reach besides its own and the warp's own zone's. X's five gates
/// out of Lumiose are all "Lumiose Gate", and only what lies beyond tells them apart.
pub fn gate_names(
    zones: &IndexMap<i64, Zone>,
    table: &[u8],
) -> Result<IndexMap<(i64, i64), (i64, i64)>> {
    let mut out: IndexMap<(i64, i64), (i64, i64)> = IndexMap::new();
    for (&zone, (_data, _furniture, warps)) in zones {
        let here = place(table, zone)?;
        for (slot, record) in warps.iter().enumerate() {
            let to = u16(record, zonedata::WARP_TO)?;
            if !zones.contains_key(&to)
                || row(table, to, crate::games::gen6::field::zonetable::INFO_KIND)?
                    != crate::games::gen6::field::zonetable::GATE
            {
                continue;
            }
            let gate = place(table, to)?;
            if gate == here {
                continue;
            }
            let mut beyond: HashSet<i64> = HashSet::new();
            for onward in &zones[&to].2 {
                beyond.insert(place(table, u16(onward, zonedata::WARP_TO)?)?);
            }
            beyond.remove(&gate);
            beyond.remove(&here);
            if beyond.len() == 1 {
                let only = *beyond.iter().next().expect("one left");
                out.insert((zone, slot as i64), (gate, only));
            }
        }
    }
    Ok(out)
}

/// zone -> (line, street, place) of each mark, each file given to the zone whose records
/// stand on the most of the lines it marks: a file a zone, in the zones' order (spike 107).
#[allow(clippy::type_complexity)]
fn marks_by_zone(
    zones: &IndexMap<i64, Zone>,
    marks: &[Vec<u8>],
) -> Result<IndexMap<i64, Vec<(i64, i64, [i64; 4])>>> {
    let mut on_lines: IndexMap<i64, HashSet<i64>> = IndexMap::new();
    for (&zone, (_data, furniture, warps)) in zones {
        let mut lines: HashSet<i64> = HashSet::new();
        for r in furniture.iter().chain(warps) {
            if zonedata::on_rail(r)? {
                lines.insert(rail::placed(r)?.0);
            }
        }
        if !lines.is_empty() {
            on_lines.insert(zone, lines);
        }
    }
    let mut out: IndexMap<i64, Vec<(i64, i64, [i64; 4])>> = IndexMap::new();
    for blob in marks {
        let read = marks_of(blob)?;
        let lines: HashSet<i64> = read.iter().map(|(line, _street, _place)| *line).collect();
        let mut best: Option<i64> = None;
        let mut most = 0;
        for (&zone, held) in &on_lines {
            let shared = held.intersection(&lines).count();
            if best.is_none() || shared > most {
                best = Some(zone);
                most = shared;
            }
        }
        if let Some(best) = best
            && on_lines[&best].intersection(&lines).next().is_some()
        {
            out.entry(best).or_default().extend(read);
        }
    }
    Ok(out)
}

fn marks_of(blob: &[u8]) -> Result<Vec<(i64, i64, [i64; 4])>> {
    let mut out = Vec::new();
    if blob.len() < 4 {
        return Ok(out);
    }
    let count = u32(blob, 0)?;
    for i in 0..count {
        let at = 4 + i * MARK;
        if at + MARK > blob.len() as i64
            || u16(blob, at + MARK_PLACEMENT)? != crate::games::gen6::field::zonedata::ON_RAIL
        {
            continue;
        }
        out.push((
            u32(blob, at + MARK_LINE)?,
            u16(blob, at + MARK_STREET)?,
            [
                s16(blob, at + MARK_PLACE)?,
                s16(blob, at + MARK_PLACE + 2)?,
                s16(blob, at + MARK_PLACE + 4)?,
                s16(blob, at + MARK_PLACE + 6)?,
            ],
        ));
    }
    Ok(out)
}

/// Cells between two stretches of one line as the game measures them, (a, x, cells a,
/// cells x) each: the way a line is measured turns or swaps both alike, which keeps it.
fn gap(first: &[i64], second: &[i64]) -> i64 {
    let mut total = 0;
    for axis in 0..2 {
        let one = rail::span(first[axis], first[axis + 2]);
        let two = rail::span(second[axis], second[axis + 2]);
        total += 0i64
            .max(two.start - one.end + 1)
            .max(one.start - two.end + 1);
    }
    total
}

/// Cells between a sign and a door at their nearest, on the grid or on one line of the
/// streets; None where one is on the grid and the other on a street, or they are on two lines.
fn apart(sign: &[u8], door: &[u8]) -> Result<Option<i64>> {
    if zonedata::on_rail(sign)? != zonedata::on_rail(door)? {
        return Ok(None);
    }
    if !zonedata::on_rail(sign)? {
        return Ok(nearest(&grid_cells(sign, true)?, &grid_cells(door, false)?));
    }
    let (on, a, x, width, height) = rail::placed(sign)?;
    let (other, a2, x2, width2, height2) = rail::placed(door)?;
    Ok(if on == other {
        Some(gap(&[a, x, width, height], &[a2, x2, width2, height2]))
    } else {
        None
    })
}

fn grid_cells(record: &[u8], furniture: bool) -> Result<Vec<Cell>> {
    let ((x, z), (wide, deep)) = if furniture {
        zonedata::furniture_box(record)?
    } else {
        zonedata::warp_box(record)?
    };
    let mut cells = Vec::new();
    for i in 0..wide.max(1) {
        for j in 0..deep.max(1) {
            cells.push((x + i, z + j));
        }
    }
    Ok(cells)
}

fn nearest(first: &[Cell], second: &[Cell]) -> Option<i64> {
    if first.is_empty() || second.is_empty() {
        return None;
    }
    let mut best: Option<i64> = None;
    for a in first {
        for b in second {
            let apart = (a.0 - b.0).abs() + (a.1 - b.1).abs();
            if best.is_none_or(|best| apart < best) {
                best = Some(apart);
            }
        }
    }
    best
}

/// A message's first page, its lines run together, or None for one that says something
/// only the game fills in, as the player's house says whose it is.
fn first_page(raw: &[u8]) -> Result<Option<String>> {
    let tokens = match text::tokens(raw) {
        Ok(tokens) => tokens,
        Err(error) if error.is(Kind::ValueError) => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut first: Vec<String> = Vec::new();
    for token in tokens {
        match token {
            Token::Text(words) => first.push(words),
            Token::Control(control) if text::PAGE_BREAKS.contains(&control.code) => break,
            Token::Control(_) => return Ok(None),
        }
    }
    let words = py::split(&first.concat()).join(" ");
    Ok(if words.is_empty() { None } else { Some(words) })
}

/// The one facility of its town's list a plate's words hold, or None where they hold none or
/// two. ORAS's plates put a slogan on a line before or after the name, "“Ride in Gravel and
/// Shake Up Your Soul!” Rydel's Cycles", and the Trick House's is directions to it, so a plate
/// whose first page is more than one line is named by what its town's list calls it (spike
/// 109). A first page of one line is the name alone and may say more than the list does:
/// "Move Tutor House", "Pokémon Trainers' School". Spaces are left out of the match, since
/// the Japanese and Korean break a name over two lines.
fn facility(said: &str, listed: &[String]) -> Option<String> {
    let listed: Vec<&String> = listed.iter().collect();
    let held = held_in(said, &listed);
    if held.len() == 1 {
        held.into_iter().next().cloned()
    } else {
        None
    }
}

/// The one facility of `every` that a line of the plate is, on any page, or two lines run
/// together, since the Japanese and Korean break a name over two; None for none or two. A town's
/// list is full at six, and Slateport's leaves out the Oceanic Museum, whose plate says "The
/// endless sea sustains all life." first and its name on the next page: the list's text still
/// names it. A whole line, not words within one, as `_facility` takes: X's "Autumnal Avenue
/// Ahead" plate holds "Boutique" in "Poké Ball Boutique". A line's quotation marks are left out:
/// the Japanese and Korean plates put the museum's name in them, 「うみの かがく はくぶつかん」,
/// and named the door by its motto (a user's, in a Chinese translation, 2026-10-26).
fn named(raw: &[u8], every: &[String]) -> Result<Option<String>> {
    let tokens = match text::tokens(raw) {
        Ok(tokens) => tokens,
        Err(error) if error.is(Kind::ValueError) => return Ok(None),
        Err(error) => return Err(error),
    };
    let joined: String = tokens
        .into_iter()
        .map(|token| match token {
            Token::Text(words) => words,
            Token::Control(_) => "\n".to_string(),
        })
        .collect();
    let lines: Vec<String> = joined
        .split('\n')
        .map(|line| {
            py::split(line)
                .concat()
                .trim_matches(|c| QUOTES.contains(c))
                .to_string()
        })
        .collect();
    let mut runs: HashSet<String> = lines.iter().cloned().collect();
    for pair in lines.windows(2) {
        runs.insert(format!("{}{}", pair[0], pair[1]));
    }
    let named: HashSet<&String> = every
        .iter()
        .filter(|name| !name.is_empty() && runs.contains(&py::split(name).concat()))
        .collect();
    Ok(if named.len() == 1 {
        named.into_iter().next().cloned()
    } else {
        None
    })
}

/// The facilities of a town's list a plate's words hold, spaces left out of the match.
/// (Python's `_held`.)
fn held_in<'a>(
    said: &str,
    listed: impl IntoIterator<Item = &'a &'a String>,
) -> HashSet<&'a String> {
    let held: HashSet<&String> = listed
        .into_iter()
        .copied()
        .filter(|name| !name.is_empty() && holds(said, name))
        .collect();
    // "Pokémon Day Care" is inside "Pokémon Day Care Annex": the longer one is the plate's.
    held.iter()
        .copied()
        .filter(|name| !held.iter().any(|other| name != other && holds(other, name)))
        .collect()
}

fn holds(said: &str, name: &str) -> bool {
    // Case left out: one of the French shipyard's three notices says "Chantier Naval De Poupe".
    casefold(&py::split(said).concat()).contains(&casefold(&py::split(name).concat()))
}

/// Python's `str.casefold`, for the letters the games' languages write: lowercase, and the
/// letters whose fold is not their lowercase.
///
/// shortcut: only ß, ẞ, ſ, µ, ς and the Latin ligatures fold differently from `to_lowercase`
/// here; the rest of CaseFolding.txt's full foldings (Greek with iota, Armenian, Cherokee) if a
/// game's text ever holds them.
fn casefold(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            'ß' | 'ẞ' => out.push_str("ss"),
            'ſ' => out.push('s'),
            'µ' => out.push('μ'),
            'ς' => out.push('σ'),
            'ﬀ' => out.push_str("ff"),
            'ﬁ' => out.push_str("fi"),
            'ﬂ' => out.push_str("fl"),
            'ﬃ' => out.push_str("ffi"),
            'ﬄ' => out.push_str("ffl"),
            'ﬅ' | 'ﬆ' => out.push_str("st"),
            _ => out.extend(c.to_lowercase()),
        }
    }
    out
}

/// (zone, furniture index) of every plate outdoors that says nothing but the name of a door
/// of its zone, `doors` being every door's name: in Mauville Hills "Apt. 13 / Wattson's Home"
/// stands by the door it names, and listed both said it twice (Orion, 2026-10-13). A plate
/// that says more is kept, a Gym's with its leader, the Oceanic Museum's with its slogan.
pub fn door_signs(
    zones: &IndexMap<i64, Zone>,
    table: &[u8],
    story: &dyn Fn(i64) -> Result<Vec<Vec<u8>>>,
    outside: &HashSet<i64>,
    doors: &IndexMap<(i64, i64), String>,
) -> Result<HashSet<(i64, i64)>> {
    let mut out: HashSet<(i64, i64)> = HashSet::new();
    let mut inside: Vec<i64> = outside.iter().copied().collect();
    inside.sort();
    for zone in inside {
        let Some((data, furniture, warps)) = zones.get(&zone) else {
            continue;
        };
        let code = script_of(data)?;
        let names: HashSet<&String> = (0..warps.len() as i64)
            .filter_map(|slot| doors.get(&(zone, slot)))
            .collect();
        let Some(code) = code.filter(|_| !names.is_empty()) else {
            continue;
        };
        let lines = story(row(table, zone, INFO_STORY)?)?;
        for (index, _record, line) in plates_of(&code, furniture, &lines)? {
            let said = py::split(&words_of(item(&lines, line)?)?).join(" ");
            if names.contains(&&said) {
                out.insert((zone, index));
            }
        }
    }
    Ok(out)
}

/// (zone, a furniture record's script) -> the shared script its case runs (`Script.runs`),
/// out of `native_furniture`: what the record is wherever it stands, as every vending machine is
/// one of five scripts.
pub fn shared_furniture(natives: &IndexMap<(i64, i64), (i64, i64)>) -> IndexMap<(i64, i64), i64> {
    natives
        .iter()
        .filter(|(_, (_, constant))| *constant >= script::SHARED)
        .map(|(&key, &(_, constant))| (key, constant))
        .collect()
}

/// (zone, a furniture record's script) -> the native its case hands one constant to, and the
/// constant (`Script.native`): a vending machine's shared script, or a vine's swing or climb.
pub fn native_furniture(zones: &IndexMap<i64, Zone>) -> Result<IndexMap<(i64, i64), (i64, i64)>> {
    let mut out: IndexMap<(i64, i64), (i64, i64)> = IndexMap::new();
    for (&zone, (data, furniture, _warps)) in zones {
        let code = if furniture.is_empty() {
            None
        } else {
            script_of(data)?
        };
        let Some(code) = code else {
            continue;
        };
        for record in furniture {
            let number = u16(record, zonedata::FURNITURE_SCRIPT)?;
            if number < script::SHARED
                && let Some(found) = code.native(number)?
            {
                out.insert((zone, number), found);
            }
        }
    }
    Ok(out)
}

/// (zone, a trigger's script) -> (x, how high in world units, z) of where it puts the
/// player, for each trigger of a zone `teleporting` lists whose case hands that zone's native
/// three constants, x, the floor in squares of height and z (`Script.call`, `Game.teleporting`).
/// A native's number is its zone script's own, so only the zones listed are read.
pub fn teleports(
    zones: &IndexMap<i64, Zone>,
    teleporting: &IndexMap<i64, i64>,
) -> Result<IndexMap<(i64, i64), (i64, i64, i64)>> {
    let mut out: IndexMap<(i64, i64), (i64, i64, i64)> = IndexMap::new();
    for (&zone, &native) in teleporting {
        let Some((data, _furniture, _warps)) = zones.get(&zone) else {
            continue;
        };
        let found = zonedata::zone_file(data)?;
        let code = script_of(data)?;
        let (Some(found), Some(code)) = (found, code) else {
            continue;
        };
        for record in zonedata::read_triggers(&zonedata::Bytes(data), &found)? {
            let number = u16(&record, zonedata::TRIGGER_SCRIPT)?;
            if let Some((called, constants)) = code.call(number)?
                && called == native
                && constants.len() == 3
            {
                let (x, squares, z) = (constants[0], constants[1], constants[2]);
                out.insert((zone, number), (x, squares * zonedata::CELL, z));
            }
        }
    }
    Ok(out)
}

/// Each furniture record the zone's script shows a plate for, by its index, with the line of
/// the story text it shows: a record that shows one line in a plate's style, and no other.
fn plates_of<'a>(
    code: &script::Script,
    furniture: &'a [Vec<u8>],
    lines: &[Vec<u8>],
) -> Result<Vec<(i64, &'a Vec<u8>, i64)>> {
    let mut out = Vec::new();
    for (index, record) in furniture.iter().enumerate() {
        let mut shown: IndexSet<i64> = IndexSet::new();
        for (line, style) in script::sign_lines(code, u16(record, zonedata::FURNITURE_SCRIPT)?)? {
            if style == script::PLATE {
                shown.insert(line);
            }
        }
        if shown.len() == 1 {
            let line = shown.pop().expect("one line is shown");
            if line < lines.len() as i64 {
                out.push((index as i64, record, line));
            }
        }
    }
    Ok(out)
}

/// A message's first page as it is laid out, its line breaks kept.
fn first_lines(raw: &[u8]) -> Result<String> {
    let tokens = match text::tokens(raw) {
        Ok(tokens) => tokens,
        Err(error) if error.is(Kind::ValueError) => return Ok(String::new()),
        Err(error) => return Err(error),
    };
    let mut first: Vec<String> = Vec::new();
    for token in tokens {
        match token {
            Token::Text(words) => first.push(words),
            Token::Control(control) if text::PAGE_BREAKS.contains(&control.code) => break,
            Token::Control(_) => {}
        }
    }
    Ok(py::strip(&first.concat()).to_string())
}

fn words_of(raw: &[u8]) -> Result<String> {
    match text::tokens(raw) {
        Ok(tokens) => Ok(tokens
            .into_iter()
            .filter_map(|token| match token {
                Token::Text(words) => Some(words),
                Token::Control(_) => None,
            })
            .collect()),
        Err(error) if error.is(Kind::ValueError) => Ok(String::new()),
        Err(error) => Err(error),
    }
}

fn script_of(data: &[u8]) -> Result<Option<script::Script>> {
    let Some(zone) = crate::games::gen6::field::zonedata::zone_file(data)? else {
        return Ok(None);
    };
    if zone.offsets.len() as i64 <= crate::games::gen6::field::zonedata::ZONE_SCRIPT + 1 {
        return Ok(None);
    }
    let (start, end) = (
        zone.offsets[crate::games::gen6::field::zonedata::ZONE_SCRIPT as usize],
        zone.offsets[crate::games::gen6::field::zonedata::ZONE_SCRIPT as usize + 1],
    );
    script::read(py::slice(data, start, end))
}

fn row(table: &[u8], zone: i64, at: i64) -> Result<i64> {
    let offset = zone * crate::games::gen6::field::zonetable::INFO_SIZE + at;
    if offset + 2 <= table.len() as i64 {
        u16(table, offset)
    } else {
        Ok(0)
    }
}

fn place(table: &[u8], zone: i64) -> Result<i64> {
    Ok(
        row(table, zone, crate::games::gen6::field::zonetable::INFO_NAME)?
            & crate::games::gen6::field::zonetable::NAME_MASK,
    )
}
