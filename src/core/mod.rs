//! The core: everything pa3ds does that knows nothing about Pokémon (ADR-0006).
//!
//! - Reaching the emulator: `rpc`, `memory` (reads, and the page caches a poll reads through),
//!   `binary`, and `snapshot`, memory read in bulk, which every sweep is and a reader's failure is
//!   saved as (ADR-0019).
//! - Finding the game's objects: `rtti` (a class's name from its vtable), `discovery` (sweeping for
//!   them) and `heaps` (the blocks just allocated, ADR-0034).
//! - The ROM on disk: `rom`, its file system, for the game's own words (ADR-0008).
//! - The player's side: `speech`, `keys` and `keyfile`, `window`,
//!   `names` and `toml`, the files the player owns (ADR-0031).
//! - `pathfind`: a way on foot over any game's cells.
//!
//! What the game is, and what its memory means, is a game package's (`pa3ds.games`).

pub mod binary;
pub mod config;
pub mod discovery;
pub mod heaps;
pub mod keyfile;
pub mod keys;
pub mod memory;
pub mod modules;
pub mod names;
pub mod pathfind;
pub mod rom;
pub mod rpc;
pub mod rtti;
pub mod snapshot;
pub mod speech;
pub mod toml;
pub mod window;
pub mod words;

pub mod tones;
pub mod touch;

pub mod instance;

pub mod bcsar;
pub mod clips;
pub mod loudness;
pub mod mixer;
