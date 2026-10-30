//! The gen 6 field: where the player is, what is on the map, and the ground under it.
//!
//! `reader` puts the field together for the profile from the structures each module here reads:
//! the move models (`mmodel`), the chunks (`mapblock`), Lumiose's rails (`rail`, `camera`), the
//! Dowsing Machine (`dowsing`), and who is speaking on the field (`talk`, `speaker`).

pub mod camera;
pub mod dowsing;
pub mod features;
pub mod fieldmap;
pub mod hidden_items;
pub mod mapblock;
pub mod mmodel;
pub mod rail;
pub mod reader;
pub mod speaker;
pub mod talk;
pub mod terrain;
pub mod works;
pub mod zonedata;
pub mod zonetable;

pub mod footsteps;
