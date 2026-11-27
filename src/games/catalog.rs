//! Which game is running, by its title id: the game packages pa3ds has, what each keeps where the
//! others do not (`games.gen6.game`), and which engine's package puts it together (`start`).
//!
//! Bound by:
//! - ADR-0006: imports go one way: the core, then common, then gen6 and sm, then oras and xy, and
//!   catalog and app above them.

use std::rc::Rc;

use crate::core::rpc::Rpc;
use crate::core::speech::Speech;
use crate::games::common::session::{Session, Setup};
use crate::games::gen6::game::Game;
use crate::games::sm::SunMoon;
use crate::games::{gen6, oras, sm, xy};
use crate::py::Result;

/// The games the gen 6 profile reads.
pub fn games() -> [&'static Game; 2] {
    [oras::game(), xy::game()]
}

/// A game the catalog knows: one the gen 6 profile reads, or Sun and Moon (Python's
/// `Game | SunMoon`).
#[derive(Clone, Copy)]
pub enum Running {
    Gen6(&'static Game),
    SunMoon(&'static SunMoon),
}

impl Running {
    /// What the log calls it.
    pub fn name(&self) -> &str {
        match self {
            Running::Gen6(game) => &game.name,
            Running::SunMoon(game) => &game.name,
        }
    }
}

/// The game running as `title`: Sun and Moon, one the gen 6 profile reads, or ORAS for one
/// pa3ds does not know, since it was the only game read before there were two, and a guess that
/// reads a menu is better than none.
pub fn for_title(title: i64) -> Running {
    for game in [sm::game(), sm::ultra()] {
        if game.titles.contains(&title) {
            return Running::SunMoon(game);
        }
    }
    Running::Gen6(
        games()
            .into_iter()
            .find(|game| game.titles.contains(&title))
            .unwrap_or(oras::game()),
    )
}

/// `game` put together by its engine's package: its readers, and how its heaps are read.
/// `image` is the executable as read from `code`.
pub fn start(
    game: Running,
    rpc: Rc<dyn Rpc>,
    speech: Rc<Speech>,
    code: (i64, i64),
    setup: Setup,
    image: &[u8],
) -> Result<Session> {
    match game {
        Running::SunMoon(game) => sm::session::start(game, rpc, speech, setup),
        Running::Gen6(game) => gen6::session::start(game, rpc, speech, code, setup, image),
    }
}
