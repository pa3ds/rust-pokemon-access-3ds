//! Gen 6's readers of its screens other than the field: the battle's HP box, list windows and
//! choice boxes, the button managers, the town map's place under its cursor, the Pokédex's
//! habitat map, the Hoenn map's cursor and the party's icons. ORAS's AreaNav's views and its
//! Pokémon Distribution are ORAS's own (`oras.subscreen`).

pub mod battle;
pub mod buttons;
pub mod choices;
pub mod habitat;
pub mod lists;
pub mod mapcursor;
pub mod partyicons;
pub mod summary;
pub mod townmap;

pub mod boxes;
pub mod partners;

pub mod bag_sort;
pub mod registered;

pub mod bag;
pub mod battle_bag;
pub mod counts;
pub mod mart;

pub mod shiny;

pub mod rows;

pub mod reminder;

pub mod pokedex;
pub mod scrolled;

pub mod sources;

pub mod claims;
