//! pa3ds: a screen reader accessibility layer for 3DS Pokemon games.
//!
//! A port of the Python pa3ds (`port.toml` names the commit it matches): each Python module has
//! a Rust module at the same path, so a change upstream is a change to one file here.

// Lints that would bend a 1:1 port away from the Python it mirrors: a reader takes the arguments
// its Python took, and a constructor is named as the class was called.
#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::new_without_default,
    clippy::module_inception,
    clippy::needless_range_loop,
    clippy::collapsible_else_if
)]

#[allow(non_snake_case)]
pub mod __main__;
pub mod app;
pub mod core;
pub mod debug;
pub mod games;
pub mod py;
pub mod tape;
pub mod update;
pub mod ux;

#[cfg(test)]
#[path = "../tests/mod.rs"]
mod tests;
