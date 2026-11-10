//! A Pokémon's summary: which Pokémon it shows, by its process (spike 196).
//!
//! The summary's stats are on the top screen and built first, the Pokémon's name and level on the
//! bottom; a screen rewritten is led by the layout naming its Pokémon, said first where the game's
//! order put it in the middle (`ScreenAnnouncer.say`'s `subject`).

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::pokemon;
use crate::games::gen6::apps::RunningApp;
use crate::py::Result;

pub const SUMMARY: &str = "N3app6status7ProcessE";
/// The summary's process -> the `PokemonParam` it shows, a copy of the party member's record
/// (spike 196).
pub const SHOWN: i64 = 0x78;

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
