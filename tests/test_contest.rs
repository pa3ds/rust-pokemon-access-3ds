//! A contest's standings, from the game's own record of its appeals (spike 298 on
//! or_toughness_contest): its four entrants' blocks, the player's Metagross first, then Seedottie,
//! Visikoth and Poochin.

use crate::games::oras::screens::contest::{
    BLOCK, BLOCKS, ENTRANTS, PLACE, ROUNDS, by_turn, entrant, named_in, standing, standings,
};

/// Each block's place before the first appeal: the first round's turn order.
const BEFORE: [i64; 4] = [4, 2, 1, 3];
/// Each block's place at the end: Asher & Visikoth first place, Seedottie fourth.
const RESULTS: [i64; 4] = [3, 4, 1, 2];

/// `NAMES.get(block)`.
fn names(block: i64) -> Option<String> {
    match block {
        0 => Some("Brendan & Metagross".to_string()),
        2 => Some("Asher & Visikoth".to_string()),
        1 => Some("Jordyn & Seedottie".to_string()),
        _ => None,
    }
}

/// The appeals' bytes as the game keeps them, to the end of the last block.
fn appeals(rounds: u8, places: [u8; 4]) -> Vec<u8> {
    let mut memory = vec![0u8; (BLOCKS + BLOCK * ENTRANTS) as usize];
    memory[ROUNDS as usize] = rounds;
    for (at, place) in places.into_iter().enumerate() {
        memory[(BLOCKS + BLOCK * at as i64 + PLACE) as usize] = place;
    }
    memory
}

#[test]
fn test_the_places_and_rounds_are_read_as_the_game_keeps_them() {
    let now = standing(&appeals(5, RESULTS.map(|place| place as u8)))
        .unwrap()
        .unwrap();
    assert!(now.rounds == 5 && now.places == RESULTS);
    // Not yet written, or not the appeals: no place is each of 1 to 4.
    assert_eq!(standing(&appeals(0, [0, 0, 0, 0])).unwrap(), None);
    assert_eq!(standing(&appeals(0, [1, 1, 3, 4])).unwrap(), None);
}

#[test]
fn test_the_players_place_is_their_blocks() {
    assert_eq!(
        standings(&RESULTS, &names, true, false),
        Some("Brendan & Metagross third place".to_string())
    );
    assert_eq!(
        standings(&RESULTS, &names, true, true),
        Some("Brendan & Metagross: third place".to_string())
    );
}

#[test]
fn test_the_others_are_said_in_order_unnamed_until_learned() {
    assert_eq!(
        standings(&RESULTS, &names, false, false),
        Some(
            "Asher & Visikoth first place, second place, Jordyn & Seedottie fourth place"
                .to_string()
        )
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

/// Visikoth, Seedottie, Poochin, then the player's Metagross appealed, as the places stood
/// before the first appeal.
#[test]
fn test_the_first_rounds_appeals_name_the_computers_entrants_in_place_order() {
    let appealed = [rival("Asher", "Visikoth"), rival("Jordyn", "Seedottie")];
    assert_eq!(
        by_turn(&BEFORE, &appealed),
        indexmap::IndexMap::from([
            (2, rival("Asher", "Visikoth")),
            (1, rival("Jordyn", "Seedottie")),
        ])
    );
}

/// The game names the Pokémon of the portrait the cursor picks the moment it is picked (spike
/// 298 on or_toughness_contest): its trainer from the contests' entrants, the two as the card
/// wrote them.
#[test]
fn test_a_portrait_is_named_by_its_pokemon_as_its_entry_card_wrote_it() {
    let rivals = [
        rival("Micah", "Poochin"),
        rival("Jordyn", "Seedottie"),
        rival("Ali", "Macherie"),
        rival("Gina", "Macherie"),
    ];
    let cards = indexmap::IndexMap::from([
        (rival("Jordyn", "Seedottie"), "Jordyn & Seedottie"),
        (rival("Gina", "Macherie"), "Gina & Macherie"),
    ]);
    let card = |trainer: &str, pokemon: &str| -> Option<String> {
        cards
            .get(&rival(trainer, pokemon))
            .map(|words| words.to_string())
    };
    assert_eq!(
        entrant("Seedottie", &rivals, &card),
        Some("Jordyn & Seedottie".to_string())
    );
    // No card shown for it: the trainer and the Pokémon joined.
    assert_eq!(
        entrant("Poochin", &rivals, &card),
        Some("Micah & Poochin".to_string())
    );
    // A name two rivals share is told apart by the card shown.
    assert_eq!(
        entrant("Macherie", &rivals, &card),
        Some("Gina & Macherie".to_string())
    );
    assert_eq!(entrant("Macherie", &rivals, &|_t, _p| None), None);
    // The player's own is no rival's.
    assert_eq!(entrant("Metagross", &rivals, &card), None);
}
