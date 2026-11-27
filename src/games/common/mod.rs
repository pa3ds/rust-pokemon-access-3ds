//! What every game is read the same way in: X and Y, ORAS, and Sun and Moon.
//!
//! Game Freak's two 3DS engines draw with the same NintendoWare layouts, type into message windows
//! laid out alike, keep a Pokémon as the same record, and store their words in the same archives;
//! what differs is names and numbers. So each module here reads one structure, and takes a game's
//! **shapes** where the games differ: gen 6's are the named default (`lyt.ORAS`,
//! `dialogue.ORAS_SHAPES`, `msgwin.CLASSES`), and Sun passes its own (`games.sm.lyt`,
//! `games.sm.msgwin`).
//!
//! - Screens: `lyt` and `layout` (panes, and whether one is on screen), `layout_file` (what a
//!   layout's CLYT names), `screen` (screen text), `cursor` and `slots` (what a cursor is on).
//! - Words: `msgwin`, `dialogue` (a message a page at a time), `strbuf`, `text` (control codes).
//! - `announce`: what is said of all of these, and in what order.
//! - `session`: what the runtime needs of a game, which each engine's package puts together.
//! - `pokemon`: a Pokémon's record, as the save keeps it.
//! - `battle`: a side of a battle as it is said, and what a gauge leads to.
//! - `garc`: the ROM's archives and the text files in them.
//! - `field`: what is on the map, and the ground under it; `navigate`, the keys that walk it
//!   and the way there.
//!
//! A reader that only one engine has lives in its package (`games.gen6`, `games.sm`).

pub mod announce;
pub mod battle;
pub mod cursors;
pub mod dialogue;
pub mod field;
pub mod garc;
pub mod layout;
pub mod layout_file;
pub mod lyt;
pub mod msgwin;
pub mod pictures;
pub mod pokemon;
pub mod screen;
pub mod screen_announce;
pub mod session;
pub mod slots;
pub mod strbuf;
pub mod text;

pub mod touch;

pub mod scenes;

pub mod clocks;

pub mod species_tags;

pub mod touch_targets;
