//! A Pokémon's summary: which Pokémon it shows, by its process (spike 196).
//!
//! The summary's stats are on the top screen and built first, the Pokémon's name and level on the
//! bottom; a screen rewritten is led by the layout naming its Pokémon, said first where the game's
//! order put it in the middle (`ScreenAnnouncer.say`'s `subject`).
//!
//! An Egg's summary is its memo alone. Turned to from a Pokémon, the game writes the Egg's moves
//! into the move panel's boxes and stops drawing the panel, every flag of it still saying drawn:
//! its moves were read out (or_egg_summary). While an Egg is shown, its pages of a Pokémon's are
//! put away (`Layouts.put_away`), as the AreaNav's views are.

use std::collections::HashSet;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::pokemon;
use crate::games::gen6::apps::RunningApp;
use crate::py::Result;

pub const SUMMARY: &str = "N3app6status7ProcessE";
/// The summary's process -> the `PokemonParam` it shows, a copy of the party member's record
/// (spike 196).
pub const SHOWN: i64 = 0x78;
/// The layout files of the summary's pages an Egg has none of: its stats and moves, their
/// contest kinds, and its ribbons.
pub const POKEMON_PAGES: [&str; 5] = [
    "params.bclyt",
    "waza.bclyt",
    "params_contest.bclyt",
    "waza_contest.bclyt",
    "ribbon.bclyt",
];

/// The `PokemonParam` the summary shows, or None while no summary runs.
pub fn shown(memory: &dyn Memory, apps: &RunningApp) -> Result<Option<i64>> {
    if apps.running.as_deref() != Some(SUMMARY) {
        return Ok(None);
    }
    let Some(process) = apps.process else {
        return Ok(None);
    };
    Ok(Some(u32(&memory.read(process + SHOWN, 4)?, 0)?))
}

/// The name of the Pokémon the summary shows, its nickname or its species', or None.
pub fn name(memory: &dyn Memory, names: &ClassNames, apps: &RunningApp) -> Result<Option<String>> {
    let found = match shown(memory, apps)? {
        Some(param) if param != 0 => pokemon::read(memory, names, param, pokemon::SPECIES_MAX)?,
        _ => None,
    };
    Ok(found.map(|found| found.nickname))
}

/// The works of the pages of a Pokémon's while the summary shows an Egg, else none.
pub fn put_away(
    memory: &dyn Memory,
    names: &ClassNames,
    apps: &RunningApp,
    layouts: &Layouts,
    files: &LayoutFiles,
) -> Result<HashSet<i64>> {
    let mon = match shown(memory, apps)? {
        Some(param) if param != 0 => pokemon::read(memory, names, param, pokemon::SPECIES_MAX)?,
        _ => None,
    };
    if !mon.is_some_and(|mon| mon.egg) {
        return Ok(HashSet::new());
    }
    Ok(layouts
        .works()
        .into_iter()
        .filter(|&(_work, root)| {
            files
                .matching(&layouts.pane_names(root))
                .is_some_and(|file| POKEMON_PAGES.contains(&file.name.as_str()))
        })
        .map(|(work, _root)| work)
        .collect())
}
