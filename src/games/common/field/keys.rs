//! The navigation keys: the modes, the actions, and which key does which in each mode, as the
//! keys file says (`<pa3ds>\user\keys.toml`, ADR-0022); and the keys of a context, which mean
//! something only while one screen runs (`DEFAULT_CONTEXTS`).
//!
//! Bound by:
//! - ADR-0009: touch mode's drag is a key like any other, Shift alone as it ships, and Shift drags
//!   nothing once it is bound elsewhere; Berry Picker's berries and Pokémon are a key each.
//! - ADR-0022: the navigation keys by mode, list, direction and touch, rebindable in `keys.toml`; a
//!   context's keys are taken only while its screen runs.

use std::collections::HashSet;
use std::sync::LazyLock;

use indexmap::IndexMap;

use crate::core::keys::SHIFT;
use crate::games::common::touch::{DRAG, ROW_NEXT, ROW_PREVIOUS, TAP};

pub const LIST: &str = "list";

pub const DIRECTION: &str = "direction";

/// Touch mode: the navigation keys walk the bottom screen's touch targets, and one taps the one
/// said (`games.common.touch`, ADR-0009).
pub const TOUCH: &str = "touch";

/// What the mode key steps through, in turn (Orion, 2026-10-02: one key for all three).
pub const MODES: [&str; 3] = [LIST, DIRECTION, TOUCH];

pub const CATEGORY_PREVIOUS: &str = "category previous";

pub const CATEGORY_NEXT: &str = "category next";

pub const ITEM_PREVIOUS: &str = "item previous";

pub const ITEM_NEXT: &str = "item next";

pub const ITEM_READ: &str = "item read";

pub const NORTH: &str = "north";

pub const EAST: &str = "east";

pub const SOUTH: &str = "south";

pub const WEST: &str = "west";

pub const DIRECTIONS: [&str; 4] = [NORTH, EAST, SOUTH, WEST];

pub const MODE_TOGGLE: &str = "mode";

/// Name whatever is chosen. One key for both name spaces, because which one a thing belongs to is
/// the thing's business and not the player's (`Interactable.naming`).
pub const RENAME: &str = "rename";

/// Name whoever last spoke. pa3ds's own, not Pokemon Access's, because pa3ds knows who is
/// speaking and Pokemon Access does not — and because the moment a player knows who somebody is
/// is the moment they have just heard them, not the moment they find them in a list.
pub const RENAME_SPEAKER: &str = "rename speaker";

/// Find the way on foot to whatever is chosen, say it, and guide the player along it.
pub const PATHFIND: &str = "pathfind";

/// The same, over water and through bushes, rocks and boulders too, each leg on the water or
/// through one said with the move that crosses it: "Surf 12 south", "Cut 1 east", "Rock Smash 1
/// east", "Strength 2 west". P alone keeps to land and goes round a bush, a rock or a boulder,
/// since a way across water is no way to a player who cannot surf yet (the user, 2026-09-28).
/// Up a sandy slope too, said with the bike that rides up it, "Mach Bike 6 north", where the
/// player is not riding it already; P goes up one only on the bike (the user, 2026-10-03). Up a
/// bumpy slope as well, the Acro Bike's, said with it, "Acro Bike 2 north", by P too on the bike,
/// since the player's model is the same on either (spike 159). Up a waterfall, said with the move,
/// "Waterfall 2 north", and down one; P keeps off it either way (the user, 2026-10-05, spike 170).
pub const PATHFIND_HM: &str = "pathfind with HMs";

/// Stop guiding: the way P or Shift and P found is dropped and nothing more of it is said, "path
/// off", or "no path" when there was none (the user, 2026-10-10).
pub const PATHFIND_OFF: &str = "pathfind off";

/// Leave out of the list, in either mode, everything Shift and P finds no way to from where the
/// player stands, or put it back: "unreachable hidden", "unreachable shown". Off whenever pa3ds
/// starts, as the mode is list mode (Orion, 2026-10-11): sometimes the player wants to know what is
/// there and out of reach, and sometimes, in Team Magma's Hideout with every teleporter of the place
/// listed, not.
pub const HIDE_UNREACHABLE: &str = "hide unreachable";

pub const PATH_PREVIOUS: &str = "path previous";

/// Choose which way P goes to the chosen thing, where there is more than one: for a big boulder,
/// the side it is pushed from, among the ways it can be pushed, "push east, 2 of 2"; P then says
/// "to push east: 3 west, 2 north". With none chosen P takes the nearest. Comma and period in every
/// mode (Orion, 2026-10-11): direction mode has the compass where list mode has I and O, and the
/// brackets stay the category everywhere.
pub const PATH_NEXT: &str = "path next";

pub const OUR_SIDE: &str = "our side";

/// Say a side of the battle again, as it was last said: "Blaze♂ L10 20/30", "Zigzagoon♂ L2 50%".
/// For when the line was missed, and while the HP box is off the screen, in the move list or the
/// Bag (ADR-0015). "no battle" outside one.
pub const ENEMY_SIDE: &str = "enemy side";

/// Say the cell the player stands in, "x 381, z 117", as the zone's cells count it: for finding a
/// place again, and for telling a spike where something is (the user, 2026-10-10).
pub const COORDINATES: &str = "coordinates";

/// Say the cells of the thing the navigation keys are on, as `COORDINATES` says the player's,
/// "x 62, z 29", and an axis a thing spans by its first and last, "x 80 to 119, z 159": for
/// picturing the map (Orion, 2026-10-15). `[pa3ds.navigation] show_y` adds how high, to both.
pub const ITEM_COORDINATES: &str = "item coordinates";

/// Say again the line pa3ds spoke last, whatever said it: for a line missed or cut off (Khai,
/// 2026-11-08).
pub const REPEAT: &str = "repeat";

/// What the keys file says above an action's key, where its name does not say it all. The
/// horde's order is Orion's, heard by fainting the bottom left Zigzagoon and the fifth going to 0%
/// (movie16, 2026-10-01): the enemy side is said by battle position (`gen6.screens.battle`).
/// Berry Picker's berries, a key each, in the game's own order of their pictures
/// (`berry_picker.BERRIES`), left to right on the tree on every difficulty: picked up, for a
/// Pokémon's key to give.
pub const BERRIES: [&str; 7] = [
    "first berry",
    "second berry",
    "third berry",
    "fourth berry",
    "fifth berry",
    "sixth berry",
    "seventh berry",
];
/// Berry Picker's three Pokémon, a key each, in the game's own order of them, in which each keeps
/// its place all round wherever it pops up (spike 262): given the berry picked up.
pub const POKEMON: [&str; 3] = ["first pokemon", "second pokemon", "third pokemon"];
/// Head It's three Pokémon, a key each, by where each stands all round: tapped, to head the ball
/// falling at it (Orion, 2026-10-31; `head_it.HeadItKeys`).
pub const HEADERS: [&str; 3] = ["left pokemon", "middle pokemon", "right pokemon"];
/// Say Head It's score, its combo and its clock or chances left, which are left unsaid as they
/// come unless the player asks (Orion, 2026-11-04; `head_it.HeadIt.score`).
pub const SCORE_KEY: &str = "score";
/// Amie's stylus moved a step over the Pokémon, which way across and down the screen each goes
/// (Orion, 2026-11-05; `amie.StylusKeys`).
pub const STYLUS_MOVES: [(&str, (i64, i64)); 4] = [
    ("stylus up", (0, -1)),
    ("stylus down", (0, 1)),
    ("stylus left", (-1, 0)),
    ("stylus right", (1, 0)),
];

pub const EXPLAINED: [(&str, &str); 10] = [
    (
        ENEMY_SIDE,
        "The enemy's Pokémon are said in the order the battle places them. In a horde of five that \
         is round a half circle from the bottom right: bottom right, top right, top middle, top \
         left, bottom left.",
    ),
    (
        MODE_TOGGLE,
        "Steps through list mode, direction mode and touch mode, which walks the buttons of the \
         bottom screen, the PokéNav's among them, and taps the one said.",
    ),
    (
        TAP,
        "Touches the button touch mode last said, and nothing else.",
    ),
    (
        DRAG,
        "In Pokémon-Amie, picks up the button touch mode last said to drag it, held or pressed \
         twice as the drag setting says. \"shift\" is Shift alone; bound to another key, Shift \
         drags nothing.",
    ),
    (
        BERRIES[0],
        "In Berry Picker, picks up a berry on the tree, the first, second and on in the game's own \
         order, left to right; a Pokémon's key then gives it, and another berry's key picks that \
         one up instead.",
    ),
    (
        POKEMON[0],
        "In Berry Picker, gives the berry picked up to one of the round's three Pokémon, in the \
         game's own order of them; with none picked up, says what it wants.",
    ),
    (
        HEADERS[0],
        "In Head It, taps the Pokémon standing on the left, in the middle or on the right, to head \
         the ball falling at it.",
    ),
    (
        SCORE_KEY,
        "In Head It, says the score, the combo and the time or the chances left.",
    ),
    (
        "stylus up",
        "In Pokémon-Amie, moves the stylus over your Pokémon, saying the part it comes onto; touch \
         mode's drag key picks it up there, and these keys then stroke.",
    ),
    (REPEAT, "Says again the last thing pa3ds said."),
];

/// Mode -> key -> action. Keys are single characters, as `pa3ds.core.keys.key_name` gives them.
pub type Bindings = IndexMap<String, IndexMap<String, String>>;

fn keys(pairs: &[(&str, &str)]) -> IndexMap<String, String> {
    pairs
        .iter()
        .map(|(key, action)| (key.to_string(), action.to_string()))
        .collect()
}

/// The user's choice of keys: I, O, J, L and K for the list (2026-09-21); O, semicolon, L and K
/// laid out as a compass for the directions; the brackets for the category, and the apostrophe for
/// the mode, beside the semicolon that is already east; P for the way there in both (2026-09-24),
/// Pokemon Access's mnemonic. The mode was M until 2026-09-22, when `pa3ds.core.keyfile` reported
/// what M costs: it is the emulator's Start button, and a key pa3ds takes is a button the player
/// has lost.
///
/// Shift and K renames what K reads, as Pokemon Access does it, and shift and J renames whoever
/// last spoke. Shift and P is the way there with HMs (2026-09-28). R says the player's side of a
/// battle again and shift and R the enemy's (2026-09-29). The apostrophe steps on to touch mode
/// after direction mode (Orion, 2026-10-02; it was shift and T for a day), where J and L walk the
/// bottom screen's buttons, I and O a row at a time, K says the one chosen again and U taps it.
/// Shift alone drags it, in Pokémon-Amie: Shift and U did from 2026-10-26 to 2026-10-29, when Shift
/// alone took its place (Orion, ADR-0009), and from 2026-11-01 Shift alone is drag's key as any
/// other key is an action's, so a player who binds drag elsewhere has Shift drag nothing (Orion:
/// players who never read of a key added beside it were left with Shift).
/// C says the player's cell and alt and P stops guiding (2026-10-10), and shift and C the cells of
/// the thing chosen (Orion, 2026-10-15). Y says the last line again (Khai, 2026-11-08). All of
/// these are the player's to change in `<pa3ds>\user\keys.toml`.
pub static DEFAULT_BINDINGS: LazyLock<Bindings> = LazyLock::new(|| {
    IndexMap::from([
        (
            LIST.to_string(),
            keys(&[
                ("i", CATEGORY_PREVIOUS),
                ("o", CATEGORY_NEXT),
                ("j", ITEM_PREVIOUS),
                ("l", ITEM_NEXT),
                ("k", ITEM_READ),
                ("[", CATEGORY_PREVIOUS),
                ("]", CATEGORY_NEXT),
                ("'", MODE_TOGGLE),
                ("shift+k", RENAME),
                ("shift+j", RENAME_SPEAKER),
                ("p", PATHFIND),
                ("shift+p", PATHFIND_HM),
                ("alt+p", PATHFIND_OFF),
                ("ctrl+p", HIDE_UNREACHABLE),
                (",", PATH_PREVIOUS),
                (".", PATH_NEXT),
                ("r", OUR_SIDE),
                ("shift+r", ENEMY_SIDE),
                ("c", COORDINATES),
                ("shift+c", ITEM_COORDINATES),
                ("y", REPEAT),
            ]),
        ),
        (
            DIRECTION.to_string(),
            keys(&[
                ("o", NORTH),
                (";", EAST),
                ("l", SOUTH),
                ("k", WEST),
                ("[", CATEGORY_PREVIOUS),
                ("]", CATEGORY_NEXT),
                ("'", MODE_TOGGLE),
                ("shift+k", RENAME),
                ("shift+j", RENAME_SPEAKER),
                ("p", PATHFIND),
                ("shift+p", PATHFIND_HM),
                ("alt+p", PATHFIND_OFF),
                ("ctrl+p", HIDE_UNREACHABLE),
                (",", PATH_PREVIOUS),
                (".", PATH_NEXT),
                ("r", OUR_SIDE),
                ("shift+r", ENEMY_SIDE),
                ("c", COORDINATES),
                ("shift+c", ITEM_COORDINATES),
                ("y", REPEAT),
            ]),
        ),
        (
            TOUCH.to_string(),
            keys(&[
                ("j", ITEM_PREVIOUS),
                ("l", ITEM_NEXT),
                ("i", ROW_PREVIOUS),
                ("o", ROW_NEXT),
                ("k", ITEM_READ),
                ("u", TAP),
                (SHIFT, DRAG),
                ("[", CATEGORY_PREVIOUS),
                ("]", CATEGORY_NEXT),
                ("'", MODE_TOGGLE),
                ("shift+k", RENAME),
                ("shift+j", RENAME_SPEAKER),
                ("p", PATHFIND),
                ("shift+p", PATHFIND_HM),
                ("alt+p", PATHFIND_OFF),
                ("ctrl+p", HIDE_UNREACHABLE),
                (",", PATH_PREVIOUS),
                (".", PATH_NEXT),
                ("r", OUR_SIDE),
                ("shift+r", ENEMY_SIDE),
                ("c", COORDINATES),
                ("shift+c", ITEM_COORDINATES),
                ("y", REPEAT),
            ]),
        ),
    ])
});

/// Every key bound in any mode: what the hook keeps from the emulator.
pub fn bound_keys(bindings: &Bindings) -> HashSet<String> {
    bindings
        .values()
        .flat_map(|keys| keys.keys().cloned())
        .collect()
}

/// The keys bound to drag in any mode: those whose release the hook says.
pub fn drag_keys(bindings: &Bindings) -> HashSet<String> {
    bindings
        .values()
        .flat_map(|keys| {
            keys.iter()
                .filter(|(_, action)| *action == DRAG)
                .map(|(key, _)| key.clone())
        })
        .collect()
}

/// Pokémon-Amie's Berry Picker while it runs: `[pa3ds.amie.berry_picker]` in the keys file.
pub const BERRY_PICKER: &str = "amie.berry_picker";
/// Pokémon-Amie's Head It while it runs: `[pa3ds.amie.head_it]` in the keys file.
pub const HEAD_IT: &str = "amie.head_it";
/// Pokémon-Amie itself while it runs: `[pa3ds.amie.stylus]` in the keys file.
pub const STYLUS: &str = "amie.stylus";

/// Context -> key -> action: keys that mean something only while one screen runs, in every mode,
/// and are taken from the emulator only then, so a key that is a button of Azahar's elsewhere stays
/// one there (Orion, 2026-10-29).
pub type Contexts = IndexMap<String, IndexMap<String, String>>;

/// The number row in Berry Picker: 1 to 7 the berries, which no difficulty has more of, and 8, 9
/// and 0 the Pokémon (Orion, 2026-10-29), for a player who would rather not walk to each with touch
/// mode's keys, which still drag there as ever. In Head It 8, 9 and 0 the Pokémon too, left to right
/// (Orion, 2026-10-31), none of them a button of Azahar's as it ships; and R the score (Orion,
/// 2026-11-04), pa3ds's key everywhere else, a battle's side: S and T, which he offered beside it,
/// are his B and his D-pad's up, which the list after a round may want. In Amie itself the arrows
/// move the stylus over the Pokémon (Orion, 2026-11-05): they are the circle pad as Orion binds it,
/// and Amie's script reads no stick, only that some button is pressed.
pub static DEFAULT_CONTEXTS: LazyLock<Contexts> = LazyLock::new(|| {
    let mut keys: IndexMap<String, String> = BERRIES
        .iter()
        .enumerate()
        .map(|(i, berry)| ((i + 1).to_string(), berry.to_string()))
        .collect();
    keys.insert("8".to_string(), POKEMON[0].to_string());
    keys.insert("9".to_string(), POKEMON[1].to_string());
    keys.insert("0".to_string(), POKEMON[2].to_string());
    IndexMap::from([
        (BERRY_PICKER.to_string(), keys),
        (
            HEAD_IT.to_string(),
            IndexMap::from([
                ("8".to_string(), HEADERS[0].to_string()),
                ("9".to_string(), HEADERS[1].to_string()),
                ("0".to_string(), HEADERS[2].to_string()),
                ("r".to_string(), SCORE_KEY.to_string()),
            ]),
        ),
        (
            STYLUS.to_string(),
            IndexMap::from([
                ("up".to_string(), "stylus up".to_string()),
                ("down".to_string(), "stylus down".to_string()),
                ("left".to_string(), "stylus left".to_string()),
                ("right".to_string(), "stylus right".to_string()),
            ]),
        ),
    ])
});

/// The keys a context binds: those the hook takes while its screen runs.
pub fn context_keys(contexts: &Contexts, context: Option<&str>) -> HashSet<String> {
    match context {
        None => HashSet::new(),
        Some(context) => contexts
            .get(context)
            .map(|keys| keys.keys().cloned().collect())
            .unwrap_or_default(),
    }
}
