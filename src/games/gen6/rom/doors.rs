//! Door names out of the ROM alone (spike 86): every zone's warps, the zone table's area kinds,
//! and each town's list of what it has, paired where the pairing can be derived; never guessed.

use std::collections::HashSet;

use indexmap::IndexMap;

use crate::core::binary::{u16, u32};
use crate::core::rom::{Rom, lz11};
use crate::games::common::garc::garc_files;
use crate::games::gen6::field::{zonedata, zonetable};
use crate::games::gen6::rom::layout::{RomLayout, TownFacilities, zone_table};
use crate::py::{self, Error, Result};

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

/// The area kind of a Gym's rooms, in ORAS and in X alike (spikes 85, 101).
pub const GYM: i64 = 0x11;

/// (outdoor zone, warp slot) -> the line of the game's list of what each town has
/// (`TownFacilities`) naming the building behind that door, for every door the ROM's own tables
/// pair with a facility (spike 86; X's Town Map, spike 101). `outside` is the zones walked about
/// in as outdoors besides the towns' own (`plates.outdoors`): Lumiose's streets, whose doors
/// count as the town's (spike 107).
///
/// 1. A door whose zone has the area kind of the town's own outdoors leads to a plain house: a
///    town and its houses share one kind. Some facilities do too (Dewford Hall, the Trainers'
///    School), and they are left unnamed with the houses.
/// 2. A kind behind doors in two towns or more is a shared facility, named for the one facility
///    every one of those towns lists that no kind in more towns has already taken — most towns
///    first, so the Pokémon Center's kind takes "Pokémon Center" before the Gyms' kind can.
/// 3. A town left with one unnamed door and one unnamed facility pairs them.
/// 4. A door off a street, or in a zone of the town that is not its own parent, is named by the
///    kind as a shared facility only where its town lists that facility: Lumiose lists
///    "Boutique Couture" where five towns list "Boutique" (spike 107). What kind is which
///    facility is worked out from the towns' own doors alone, as before.
/// 5. A warp into a Gym's room is named for the first facility its town's row lists that no
///    other town's does. In X's Gym towns the Gym comes first, before the Pokémon Center (spike
///    107); in ORAS's after the Center and the Poké Mart, where a town has one, and before
///    whatever else only that town has: "Mauville City Gym" ahead of "Rydel's Cycles", where
/// the Gym's plate had named it "Mauville City Pokémon Gym Leader: Wattson".
pub fn door_facilities(
    rom: &Rom,
    zones: Option<&[Vec<u8>]>,
    layout: &RomLayout,
    outside: &HashSet<i64>,
) -> Result<IndexMap<(i64, i64), i64>> {
    let Some(towns) = &layout.towns else {
        return Ok(IndexMap::new());
    };
    let read_zones;
    let zones: &[Vec<u8>] = match zones {
        Some(zones) => zones,
        None => {
            read_zones = garc_files(&rom.read(&layout.zones)?)?;
            &read_zones
        }
    };
    let table = zone_table(zones, layout)?;
    let count = table.len() as i64 / zonetable::INFO_SIZE;

    let row = |zone: i64| -> &[u8] {
        py::slice(
            &table,
            zone * zonetable::INFO_SIZE,
            (zone + 1) * zonetable::INFO_SIZE,
        )
    };
    let kind = |zone: i64| -> Result<i64> { u16(row(zone), zonetable::INFO_KIND) };
    let parent = |zone: i64| -> Result<i64> { u16(row(zone), zonetable::INFO_PARENT) };
    let place = |zone: i64| -> Result<i64> {
        Ok(u16(row(zone), zonetable::INFO_NAME)? & zonetable::NAME_MASK)
    };

    let mut listed: IndexMap<i64, Vec<i64>> = IndexMap::new();
    // A row's zone -> its facilities in the row's order.
    let mut rows: IndexMap<i64, Vec<i64>> = IndexMap::new();
    for (zone, ordered) in town_rows(rom, towns, count)? {
        let lines = listed.entry(place(zone)?).or_default();
        let kept = rows.entry(zone).or_default();
        for line in ordered {
            if !lines.contains(&line) {
                lines.push(line);
            }
            if !kept.contains(&line) {
                kept.push(line);
            }
        }
    }

    // Place -> (outdoor zone, slot, zone behind) for every door in from its outdoors.
    let mut doors: IndexMap<i64, Vec<(i64, i64, i64)>> = IndexMap::new();
    // The same for doors off a street, and in a town's zones that are not their own parent.
    let mut street_doors: IndexMap<i64, Vec<(i64, i64, i64)>> = IndexMap::new();
    // (outdoor zone, slot) of every warp into a Gym's room.
    let mut gyms: Vec<(i64, i64)> = Vec::new();
    let mut own: IndexMap<i64, HashSet<i64>> = IndexMap::new();
    for zone in 0..count {
        let walked = parent(zone)? == zone || outside.contains(&zone);
        if !walked
            || !listed.contains_key(&place(zone)?)
            || zone >= zones.len() as i64
            || zones[zone as usize].is_empty()
        {
            continue;
        }
        own.entry(place(zone)?).or_default().insert(kind(zone)?);
        for (slot, to, how) in zonedata::warps(&lz11(&zones[zone as usize])?)? {
            if to >= count || outside.contains(&to) {
                continue;
            }
            if kind(to)? == GYM {
                gyms.push((zone, slot));
            }
            if how == zonedata::WARP_DOOR && parent(zone)? == zone {
                doors
                    .entry(place(zone)?)
                    .or_default()
                    .push((zone, slot, to));
            } else if how == zonedata::WARP_DOOR || how >> 8 == zonedata::WARP_STREET {
                street_doors
                    .entry(place(zone)?)
                    .or_default()
                    .push((zone, slot, to));
            }
        }
    }

    let mut behind: IndexMap<i64, HashSet<i64>> = IndexMap::new();
    for (town, entries) in &doors {
        for &(_zone, _slot, to) in entries {
            if !own[town].contains(&kind(to)?) {
                behind.entry(kind(to)?).or_default().insert(*town);
            }
        }
    }
    let mut shared: IndexMap<i64, i64> = IndexMap::new();
    let mut taken: HashSet<i64> = HashSet::new();
    let mut by_towns: Vec<(&i64, &HashSet<i64>)> = behind.iter().collect();
    by_towns.sort_by_key(|(_building, towns)| std::cmp::Reverse(towns.len()));
    for (&building, towns) in by_towns {
        if towns.len() < 2 {
            continue;
        }
        let first = *towns.iter().next().expect("two towns or more");
        let mut common: HashSet<i64> = listed[&first].iter().copied().collect();
        for town in towns {
            let lines: HashSet<i64> = listed[town].iter().copied().collect();
            common = common.intersection(&lines).copied().collect();
        }
        common.retain(|line| !taken.contains(line));
        if common.len() == 1 {
            let line = common.into_iter().next().expect("one facility is common");
            shared.insert(building, line);
            taken.insert(line);
        }
    }

    let mut named: IndexMap<(i64, i64), i64> = IndexMap::new();
    for (town, entries) in &doors {
        let mut left: Vec<(i64, i64)> = Vec::new();
        for &(zone, slot, to) in entries {
            if !own[town].contains(&kind(to)?) && !shared.contains_key(&kind(to)?) {
                left.push((zone, slot));
            }
        }
        for &(zone, slot, to) in entries {
            if let Some(&line) = shared.get(&kind(to)?) {
                named.insert((zone, slot), line);
            }
        }
        let mut claimed: HashSet<i64> = HashSet::new();
        for &(_z, _s, to) in entries {
            if let Some(&line) = shared.get(&kind(to)?) {
                claimed.insert(line);
            }
        }
        let mut unclaimed: HashSet<i64> = listed[town].iter().copied().collect();
        unclaimed.retain(|line| !claimed.contains(line));
        if left.len() == 1 && unclaimed.len() == 1 {
            named.insert(
                left[0],
                unclaimed.into_iter().next().expect("one facility is left"),
            );
        }
    }
    for (town, entries) in &street_doors {
        for &(zone, slot, to) in entries {
            if let Some(&line) = shared.get(&kind(to)?)
                && listed[town].contains(&line)
            {
                named.insert((zone, slot), line);
            }
        }
    }
    for &(zone, slot) in &gyms {
        let ordered: Vec<i64> = match rows.get(&zone) {
            Some(ordered) if !ordered.is_empty() => ordered.clone(),
            _ => rows.get(&parent(zone)?).cloned().unwrap_or_default(),
        };
        let here = place(zone)?;
        let own_only: Vec<i64> = ordered
            .iter()
            .copied()
            .filter(|line| {
                !listed
                    .iter()
                    .any(|(town, lines)| *town != here && lines.contains(line))
            })
            .collect();
        if let Some(&first) = own_only.first() {
            named.entry((zone, slot)).or_insert(first);
        }
    }
    Ok(named)
}

/// (outdoor zone, its facilities' lines in the row's order) of each row of the game's list of
/// what each town has, for every row whose zone is one of the zone table's `count`.
pub fn town_rows(rom: &Rom, towns: &TownFacilities, count: i64) -> Result<Vec<(i64, Vec<i64>)>> {
    let mut out: Vec<(i64, Vec<i64>)> = Vec::new();
    let files = garc_files(&rom.read(&towns.archive)?)?;
    let nav = item(&files, towns.file)?;
    let mut at = 0;
    while at < nav.len() as i64 - towns.row + 1 {
        let zone = u16(nav, at + towns.zone)?;
        if zone < count {
            let mut ordered: Vec<i64> = Vec::new();
            let mut offset = towns.first;
            while offset < towns.row {
                let line = u32(nav, at + offset)?;
                if line != 0 && !ordered.contains(&line) {
                    ordered.push(line);
                }
                offset += 4;
            }
            out.push((zone, ordered));
        }
        at += towns.row;
    }
    Ok(out)
}
