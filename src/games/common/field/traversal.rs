//! Traversals: the ways across ground that walking alone does not cross, each keeping what the
//! player is doing on it, its part of a search's state (GLOSSARY: Traversal).
//!
//! A kind is a module of its own, a `Traversal` built from what one ground has of it, and one line
//! of `ground.TRAVERSALS`: riding (`ride`), climbing walls (`climb`), grind rails (`grind`), vines
//! (`vines`), teleporters (`teleport`), arrow tiles (`arrow`) and Fortree Gym's rotating doors
//! (`rotating`). The search
//! keeps one state for each the ground has, in that order, and the floor after them on a stacked
//! ground, which is the ground's own and read by all (`Ground._walk`). The search gives back each
//! step of a way with its state, and each traversal names its own steps from it, "climb",
//! "push", "grind the rail", and says its own legs (`said`, `legs`). What keeps no state, a
//! current, a belt, ice, a slope, a ledge, the shore, is the search's rules and the guide's words.
//!
//! Bound by:
//! - ADR-0006: a kind of traversal is a module of its own and a line of
//!   `ground.TRAVERSALS`, naming no game's kinds.
//!
//! Python's states are nested tuples, an int a traversal but the rotating doors', a tuple of their
//! turns. The search's state here is one flat `pathfind::State`, each traversal's part a run of as
//! many as its `start` gives, the same for every state of one ground (`width`): two states
//! compare as Python's nested tuples of that one shape do.
//!
//! Python's `Leap`, a leg a traversal carries the player across in one move, a vine's or a
//! teleporter's, a way said as far as its first and then how many more, as up Coumarine Gym,
//! where every press said all 17 vines and the legs between, again at each step (Khai,
//! 2026-10-26), is a `Leg` whose `more` is some (`pathfind::Leg::more`).

use std::collections::HashSet;

use indexmap::IndexMap;

use crate::core::pathfind::{Cell, Leg, State, Step};
use crate::games::common::field::ground::Ground;
use crate::py::Result;

/// What the listing calls whatever stands in some cells, as the Pokémon to ride (`said`'s
/// `name`).
pub type Name<'a> = &'a dyn Fn(&HashSet<Cell>) -> Result<String>;

/// One kind of traversal on one ground, as the search asks it. Each question takes the ground,
/// for what every kind reads alike (its floors, its kinds of cell), the traversal's own state,
/// and the floor the player is on, `ANY_FLOOR` off a stacked ground.
pub trait Traversal {
    /// Whether it carries the player on from a cell, so that no way ends there (`stops`), or in
    /// one move to a cell of its own choosing (`leap`): only then are those asked.
    fn carries(&self) -> bool {
        false
    }

    /// Whether a leap can carry the player further than it costs, so a search across it goes by
    /// cost alone (`pathfind.find`'s `straight`).
    fn far(&self) -> bool {
        false
    }

    /// Whether it has a `carry` of its own: Python asks `type(way).carry is not Traversal.carry`.
    fn carries_on(&self) -> bool {
        false
    }

    /// What a player standing in `cell` on `floor` is doing on it: where a search starts.
    fn start(&self, ground: &Ground, cell: Cell, floor: i64) -> Result<State>;

    /// `state` after a move from `cell` to `step`, or None where the move is not taken.
    fn step(
        &self,
        ground: &Ground,
        state: &[i64],
        cell: Cell,
        step: Cell,
        floor: i64,
    ) -> Result<Option<State>>;

    /// The floor of `step` a move from `cell`, `before` to `after`, puts the player on, where
    /// this traversal decides it rather than the ground's climb (`Ground.climb`); None where it
    /// does not.
    fn lands(
        &self,
        _ground: &Ground,
        _before: &[i64],
        _after: &[i64],
        _cell: Cell,
        _step: Cell,
        _floor: i64,
    ) -> Result<Option<i64>> {
        Ok(None)
    }

    /// Whether a way can end on `cell` in `state`, or at a goal of `ends` it carries the
    /// player on from, as it ends on a teleporter it is walked to.
    fn stops(
        &self,
        _ground: &Ground,
        _state: &[i64],
        _cell: Cell,
        _floor: i64,
        _ends: &HashSet<Cell>,
    ) -> Result<bool> {
        Ok(true)
    }

    /// The way the player in `cell` in `state` is carried on a cell at a time, as a current
    /// carries them, or None: an arrow's push (`arrow`). Asked only where `carries`.
    fn carry(
        &self,
        _ground: &Ground,
        _state: &[i64],
        _cell: Cell,
        _floor: i64,
    ) -> Result<Option<&'static str>> {
        Ok(None)
    }

    /// Where the player in `cell` in `state` is carried in one move, or None.
    fn leap(&self, _ground: &Ground, _state: &[i64], _cell: Cell, _floor: i64) -> Option<Cell> {
        None
    }

    /// How many of the search's state its own part is: one, but the rotating doors' a door each.
    /// Python's states are nested, and need no telling.
    fn width(&self) -> usize {
        1
    }

    /// What each step of `way` this traversal takes is walked with, by the index of the
    /// step's last cell, "climb" or "push": said before the leg it is in, a leg ending where it
    /// changes, and over every word of the ground's own, a ramp's or a current's
    /// (`pathfind.legs`'s `placed`). By place, not by its cells, which a way can walk again on
    /// another level. `mine` is this traversal's own state at each cell, as the search found it,
    /// and `name` what the listing calls whatever stands in some cells, as the Pokémon to ride.
    fn said(
        &self,
        _ground: &Ground,
        _way: &[Step],
        _mine: &[&[i64]],
        _name: Name<'_>,
    ) -> Result<IndexMap<i64, String>> {
        Ok(IndexMap::new())
    }

    /// `legs`, the way's, with its own said as it says them: a climb's handholds with the leg
    /// they are climbed at, a vine's grab and ride as one.
    fn legs(
        &self,
        _ground: &Ground,
        _way: &[Step],
        _mine: &[&[i64]],
        legs: Vec<Leg>,
    ) -> Result<Vec<Leg>> {
        Ok(legs)
    }
}
