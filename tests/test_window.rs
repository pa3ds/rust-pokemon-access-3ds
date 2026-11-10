//! The emulator's own window, and the game name read out of its title.
//!
//! The windows themselves cannot be made in a test, so what is tested is the parsing: which of
//! Azahar's several windows is the one carrying a game name, and what happens to every title that is
//! not one. Whether the format is still the one Azahar writes is a thing to check against a running
//! emulator, not here.

use crate::core::window;

/// `UpdateWindowTitle` in the fork writes `Azahar <build> | <game>`, where the game came from
/// `loader->ReadTitle()` — the ROM's own SMDH.
#[test]
fn test_the_main_windows_title_carries_the_game() {
    assert_eq!(
        window::game_of("Azahar 2116 | Pokémon Omega Ruby"),
        "Pokémon Omega Ruby"
    );
}

/// The render windows carry a third part. Their game name is the same, so nothing is lost by
/// passing them over, and taking exactly two parts is what tells them apart.
#[test]
fn test_a_render_window_is_not_mistaken_for_the_main_one() {
    assert_eq!(
        window::game_of("Azahar 2116 | Pokémon Omega Ruby | Primary Window"),
        ""
    );
    assert_eq!(
        window::game_of("Azahar 2116 | Pokémon Omega Ruby | Secondary Window"),
        ""
    );
}

#[test]
fn test_the_emulator_with_no_game_in_it_names_nothing() {
    assert_eq!(window::game_of("Azahar 2116"), "");
    assert_eq!(window::game_of(""), "");
}

/// Azahar joins on " | " with spaces, and a game may hold a bare pipe of its own: the SMDH
/// this machine's save carries reads "Pokemon OR / AS", so a slash or a pipe is not far off.
#[test]
fn test_a_game_whose_name_holds_a_pipe_is_left_alone() {
    assert_eq!(
        window::game_of("Azahar 2116 | Pokemon OR|AS"),
        "Pokemon OR|AS"
    );
}

/// `game_title` with a stand-in for the window list, which a test cannot make.
fn found(titles: &[&str]) -> impl Fn() -> String {
    let titles: Vec<String> = titles.iter().map(|title| title.to_string()).collect();
    move || window::game_title_from(window::EMULATOR, |_emulator| Ok(titles.clone()))
}

#[test]
fn test_the_first_window_that_names_a_game_is_the_one_taken() {
    let title = found(&[
        "Azahar 2116",
        "Azahar 2116 | Pokémon Omega Ruby | Primary Window",
        "Azahar 2116 | Pokémon Omega Ruby",
    ]);
    assert_eq!(title(), "Pokémon Omega Ruby");
}

/// A names file opened with the game closed still has to be written, and pa3ds holds no table
/// of game names to fall back on: the ROM names itself instead (`pa3ds.core.names`).
#[test]
fn test_no_emulator_window_at_all_is_no_title_and_no_trouble() {
    assert_eq!(found(&[])(), "");
}

/// Whatever is on this desktop, listing it must not throw: the answer is only ever a comment
/// at the top of a file, and nothing is allowed to stop a rename being saved over it.
#[test]
fn test_the_windows_of_this_machine_can_be_asked_without_raising() {
    let _title: String = window::game_title(window::EMULATOR);
}

/// Azahar's settings were typed into with pa3ds running, and I and O never reached them:
/// any window of the emulator's counted as the game's.
#[test]
fn test_keys_are_taken_in_the_game_windows_and_not_in_the_emulators_dialogs() {
    assert!(window::shows_game("Azahar 2116 | Pokémon Omega Ruby"));
    assert!(window::shows_game(
        "Azahar 2116 | Pokémon Omega Ruby | Primary Window"
    ));
    assert!(window::shows_game(
        "Azahar 2116 | Pokémon Omega Ruby | Secondary Window"
    ));
    assert!(!window::shows_game("Configure"));
    assert!(!window::shows_game("Azahar 2116"));
}
