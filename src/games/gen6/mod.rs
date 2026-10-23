//! The gen 6 engine: Pokémon X and Y, Omega Ruby and Alpha Sapphire.
//!
//! The two pairs are one engine: the same classes, message windows, layouts, move models and zone
//! files, told apart by a handful of numbers — where the ROM keeps each kind of file, how many
//! sections a zone file has, where a move model keeps its sprite — which each game's package gives
//! in a `game.Game` (spike 100). So one profile (`profile`) and one set of readers read all four.
//!
//! - The runtime's side: `session` (a game put together for the runtime), `profile` (what to
//!   sweep for, and how each poll runs), `apps` (which screen is running), `gflheap` (Game Freak's
//!   heap).
//! - Menus and dialogue: `lists` (list windows), `choices` (the field's choice box), `battle` (the
//!   HP box), `talk` and `speaker` (who is speaking).
//! - The field: `field` (where the player is and what is around them), `mmodel` (move models),
//!   `mapblock` (the chunks' cells), `script` and `plates` (what a sign or a door's plate says),
//!   and X's Lumiose: `rail` and `camera`.
//! - The ROM: `archives`, the game's own words for places, the rival, the PC and doors.

pub mod apps;
pub mod field;
pub mod frames;
pub mod game;
pub mod gflheap;
pub mod mystatus;
pub mod profile;
pub mod rom;
pub mod screens;
pub mod session;
