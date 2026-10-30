//! The navigation keys: the modes, the actions, and which key does which in each mode, as the
//! keys file says (`<pa3ds>\user\keys.toml`, ADR-0022).
//!
//! Bound by:
//! - ADR-0022: the navigation keys by mode, list, direction and touch, rebindable in `keys.toml`.

use std::collections::HashSet;
use std::sync::LazyLock;

use indexmap::IndexMap;

use crate::games::common::touch::{DRAG, ROW_NEXT, ROW_PREVIOUS, TAP};

pub const LIST: &str = "list";

pub const DIRECTION: &str = "direction";

/// Touch mode: the navigation keys walk the bottom screen's touch targets, and one taps the one
/// said (`games.common.touch`, docs/plans/pokenav-and-touch.md).
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

/// What the keys file says above an action's key, where its name does not say it all. The
/// horde's order is Orion's, heard by fainting the bottom left Zigzagoon and the fifth going to 0%
/// (movie16, 2026-10-01): the enemy side is said by battle position (`gen6.screens.battle`).
pub const EXPLAINED: [(&str, &str); 4] = [
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
        "Holds a touch on the button touch mode last said. Keep Shift down and the keys that \
         walk the buttons carry it to each one they say; let go of Shift to let go of it.",
    ),
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
/// bottom screen's buttons, I and O a row at a time, K says the one chosen again and U taps it;
/// shift and U holds a touch on it, carried by the same keys with shift held and let go with shift
/// (Orion, 2026-10-26).
/// C says the player's cell and alt and P stops guiding (2026-10-10), and shift and C the cells of
/// the thing chosen (Orion, 2026-10-15). All of these are the player's to change in
/// `<pa3ds>\user\keys.toml`.
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
                ("shift+u", DRAG),
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
