//! A contest's standings, from its portraits as spike 196 read them (movie33).

use crate::games::oras::screens::contest::{Portrait, by_turn, named_in, standings};
use crate::py::Result;

/// The images the portraits' backgrounds draw: the player's is unlike the others'.
const THEIRS: i64 = 0x15681D80;
const MINE: i64 = 0x15679D00;

fn portrait(x: f64, appeal: f64, icon: i64, background: i64) -> Portrait {
    Portrait {
        x,
        y: 76.0,
        appeal,
        icon,
        background: Some(background),
    }
}

/// Round 4's portraits left to right, sorted by appeal, the player's first.
fn round_four() -> Vec<Portrait> {
    vec![
        portrait(-120.0, 63.0, 1, MINE),
        portrait(-40.0, 47.0, 2, THEIRS),
        portrait(40.0, 24.0, 3, THEIRS),
        portrait(120.0, 20.0, 4, THEIRS),
    ]
}

/// `NAMES.get(p.icon)`.
fn names(portrait: &Portrait) -> Result<Option<String>> {
    Ok(match portrait.icon {
        1 => Some("Brendan & Pikachu".to_string()),
        2 => Some("Molly & Ronnie".to_string()),
        4 => Some("Camden & Bolt".to_string()),
        _ => None,
    })
}

#[test]
fn test_the_players_place_is_their_portraits_by_the_bars() {
    assert_eq!(
        standings(&round_four(), &names, true, false).unwrap(),
        Some("Brendan & Pikachu first place".to_string())
    );
    assert_eq!(
        standings(&round_four(), &names, true, true).unwrap(),
        Some("Brendan & Pikachu: first place".to_string())
    );
}

#[test]
fn test_the_others_are_said_in_order_unnamed_until_learned() {
    assert_eq!(
        standings(&round_four(), &names, false, false).unwrap(),
        Some("Molly & Ronnie second place, third place, Camden & Bolt fourth place".to_string())
    );
}

/// At 428 s the bars were 29, 26, 12, 14, the fourth ahead of the third (movie33).
#[test]
fn test_bars_caught_mid_turn_are_ranked_by_their_widths() {
    let mid = vec![
        portrait(-120.0, 29.0, 1, THEIRS),
        portrait(-40.0, 26.0, 2, MINE),
        portrait(40.0, 12.0, 3, THEIRS),
        portrait(120.0, 14.0, 4, THEIRS),
    ];
    assert_eq!(
        standings(&mid, &|_p| Ok(None), false, false).unwrap(),
        Some("first place, third place, fourth place".to_string())
    );
}

/// Some of the contests' entrants, as Omega Ruby's story file 445 pairs them.
fn rivals() -> Vec<(String, String)> {
    [
        ("Gianna", "Tailster"),
        ("Molly", "Ronnie"),
        ("Camden", "Bolt"),
        ("Adeline", "Win"),
    ]
    .iter()
    .map(|&(trainer, pokemon)| (trainer.to_string(), pokemon.to_string()))
    .collect()
}

fn rival(trainer: &str, pokemon: &str) -> (String, String) {
    (trainer.to_string(), pokemon.to_string())
}

#[test]
fn test_a_message_names_the_entrants_whose_pokemon_it_names_in_order() {
    assert_eq!(
        named_in(
            "Bolt tried to show its appeal with Light Screen!",
            &rivals()
        ),
        [rival("Camden", "Bolt")]
    );
    assert_eq!(
        named_in("Ronnie and then Tailster", &rivals()),
        [rival("Molly", "Ronnie"), rival("Gianna", "Tailster")]
    );
}

#[test]
fn test_a_name_inside_another_word_is_no_name() {
    assert_eq!(
        named_in(
            "Pikachu tried to show its appeal with Thunder Wave!",
            &rivals()
        ),
        []
    );
    assert_eq!(
        named_in("It tried to startle the other Pokémon!", &rivals()),
        []
    );
}

/// Movie33's first round: Pikachu (the player's), Bolt, Tailster, Ronnie.
#[test]
fn test_the_first_rounds_appeals_name_its_portraits_left_to_right_but_the_players() {
    let turns: Vec<(String, bool)> = [
        ("pikachu", true),
        ("bolt", false),
        ("tailster", false),
        ("ronnie", false),
    ]
    .iter()
    .map(|&(icon, mine)| (icon.to_string(), mine))
    .collect();
    let appealed = [rival("Camden", "Bolt"), rival("Gianna", "Tailster")];
    assert_eq!(
        by_turn(&turns, &appealed),
        indexmap::IndexMap::from([
            ("bolt".to_string(), rival("Camden", "Bolt")),
            ("tailster".to_string(), rival("Gianna", "Tailster")),
        ])
    );
}
