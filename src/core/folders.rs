//! pa3ds's folders: where pa3ds itself is, the player's own folder beside it, and the words it
//! ships beside that. Every file pa3ds reads or writes of its own is under one of them.
//!
//! Bound by:
//! - ADR-0032: everything the player owns is under `<pa3ds>/user`, which `PA3DS_HOME` moves; never
//!   `%APPDATA%`.

use std::path::{Path, PathBuf};

use crate::py::Result;

/// Somewhere else to keep the player's things, for a test and for an agent's session: a run from
/// a checkout would otherwise write into whatever the person at this machine is really playing
/// with.
pub const HOME_ENV: &str = "PA3DS_HOME";

/// The folder pa3ds itself is in: beside the executable once there is one, and the checkout
/// otherwise.
///
/// There is always an executable here, where Python's `sys.frozen` is only set in a build.
pub fn app_path() -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    Ok(executable
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default())
}

/// `<pa3ds>/user`: the folder pa3ds keeps the player's own things in — the log, the keys file,
/// and the names they have given (`pa3ds.core.names`).
///
/// **Beside pa3ds, never in `%APPDATA%` and never under Program Files** (ADR-0032):
/// pa3ds is meant to sit next to its emulator in a folder that can be copied to
/// another machine whole, names and keys and all. `PA3DS_HOME` moves it, which is what a test
/// wants and what keeps an agent's run off the player's real files.
pub fn home_path() -> Result<PathBuf> {
    if let Some(moved) = std::env::var_os(HOME_ENV).filter(|moved| !moved.is_empty()) {
        return Ok(PathBuf::from(moved));
    }
    Ok(app_path()?.join("user"))
}

/// `<pa3ds>/data`: the words pa3ds ships, beside the player's folder, a game a folder and a
/// language a folder inside it, and `common` for what every game says alike (ADR-0040). Shipped
/// whole with each release, and replaced whole by an update: nothing in it is the player's.
///
/// A build run where it was built, `cargo run`, `cargo test` or `tape.exe` under `target\`, has
/// no folder beside it and reads the checkout's, which `tools/sync.py mirror` keeps as upstream's:
/// Python's `app_path` is the checkout's when there is no executable.
pub fn data_path() -> Result<PathBuf> {
    let beside = app_path()?.join("data");
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    Ok(if !beside.is_dir() && checkout.is_dir() {
        checkout
    } else {
        beside
    })
}
