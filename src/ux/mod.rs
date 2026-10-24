//! The windows pa3ds opens for the player: `window`, pa3ds's own, opened when it is
//! double-clicked, and `dialog`, the box asking for a name.
//!
//! Native Windows controls only, which a screen reader reads as it reads any program's (spike 81),
//! each window on a thread of its own so the poll loop never waits on one.

pub mod dialog;
pub mod window;
