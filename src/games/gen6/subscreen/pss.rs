//! The PSS's own apps, which its menu opens over the field, and the friend list's registering
//! after a trade: apps that need a touch, where touch mode offers targets (`running`,
//! `subscreen.TOUCHED`). The PSS's menu and its windows are the field's bottom screen.

/// How the PSS's own apps' classes begin, `app::pss::`: its profile, which its menu's Profile
/// opens, the player's card and message, a tap on the message choosing another
/// (or_pss_singleplayer); and the icon chosen for the card, which a tap on it opens, where touch mode
/// had nothing to touch (or_pss_singleplayer_2). Nothing in them is for the game's buttons.
pub const PSS: &str = "N3app3pss";
/// After a trade or a battle with someone not a friend, "Tap the player's icon below" to register
/// them as one, or Do not register: a player online with the PSS had nothing to touch there (a
/// player's log of 2026-11-12).
pub const FRIEND_ENTRY: &str = "N6netapp12friend_entry4ProcE";
/// Where two players who chose to battle from the PSS meet before it: POKÉMON LIST, BAG,
/// BATTLE, START! and the quit question, and in Omega Ruby and Alpha Sapphire facing X or Y, the
/// Kalos Rules' notice and its Next. Both players had nothing to touch there and were stuck
/// (the PSS's multiplayer, two Azahars in one room).
pub const COLOSSEUM: &str = "N3app9colosseum4ProcE";
/// After a trade or a battle in the PSS, "Would you like to do something else with" the other
/// player, Yes or No: only a tap answers Yes, B alone answering No, and both players had nothing to
/// touch there (the PSS's multiplayer, spike A).
pub const DIALOG: &str = "N3app6dialog4ProcE";

/// Is the app one of the PSS's own (`PSS`)?
pub fn running(app: Option<&str>) -> bool {
    app.is_some_and(|app| app.starts_with(PSS))
}
