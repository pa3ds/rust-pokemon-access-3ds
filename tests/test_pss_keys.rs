//! The keys of the screens two players meet on through the PSS (spike 319).

use std::rc::Rc;

use indexmap::IndexMap;

use crate::games::common::field::keys::{
    DEFAULT_CONTEXTS, PSS_COLOSSEUM, PSS_TRADE, THEIR_OFFER, THEIR_TEAM, TIME, YOUR_OFFER,
};
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::screens::partyicons;
use crate::games::gen6::screens::rows::PARTY;
use crate::games::gen6::subscreen::{pss, pss_keys};

const OFFERS: i64 = 0x08400000;
const TEAM: i64 = 0x08500000;

fn word(r#box: i64, text: &str, x: f64, row: f64, root: i64) -> ScreenText {
    ScreenText {
        left: x,
        right: x + 8.0 * text.chars().count() as f64,
        row,
        glyph: 13.0,
        top: true,
        root,
        ..ScreenText::new(r#box, text, false)
    }
}

fn keys(drawn: Vec<ScreenText>, before: Option<&str>, process: Vec<u8>) -> pss_keys::PssKeys {
    let files: IndexMap<i64, &str> =
        IndexMap::from([(OFFERS, pss_keys::OFFERS), (TEAM, pss_keys::TEAM)]);
    let before = before.map(String::from);
    pss_keys::PssKeys::with_process(
        Rc::new(move || drawn.clone()),
        Rc::new(move |root| {
            files.get(&root).map(|name| {
                Rc::new(LayoutFile {
                    name: name.to_string(),
                    ..LayoutFile::new(Vec::new(), IndexMap::new(), IndexMap::new())
                })
            })
        }),
        Rc::new(move || before.clone()),
        false,
        Rc::new(move || process.clone()),
    )
}

/// The other player's Treecko written on the right half (spike 319).
#[test]
fn test_a_trades_keys_say_each_offer_by_its_half_of_the_top_screen() {
    let theirs = vec![
        word(1, "Treecko", 50.0, 110.0, OFFERS),
        word(2, "Lv.", 135.0, 100.0, OFFERS),
        word(3, "9", 170.0, 100.0, OFFERS),
        word(4, "None", 30.0, -55.0, OFFERS),
    ];
    let trade = keys(theirs.clone(), None, Vec::new());
    assert_eq!(
        trade.context(Some(pss::TRADE_LIST)).unwrap(),
        Some(PSS_TRADE)
    );
    assert_eq!(
        trade.press(THEIR_OFFER, Some(pss::TRADE_LIST)),
        "Treecko Lv. 9 None"
    );
    assert_eq!(trade.press(YOUR_OFFER, Some(pss::TRADE_LIST)), "no offer");
    let mut both = theirs;
    both.push(word(5, "Pansage", -130.0, 110.0, OFFERS));
    assert_eq!(
        keys(both, None, Vec::new()).press(YOUR_OFFER, Some(pss::TRADE_LIST)),
        "Pansage"
    );
}

/// The party opened from the Colosseum draws the other player's team on the top screen, and
/// its clock under it (spike 319): its name, each icon named by its species, and level. Started
/// on that screen, pa3ds never saw the Colosseum run before it, and Shift and R said "no battle"
/// (Alpha Sapphire and Y, live): the list was opened with a team, which says it too.
#[test]
fn test_the_team_choice_says_the_other_players_team_but_its_clock() {
    let drawn = vec![
        word(1, "Brendan", -40.0, 100.0, TEAM),
        ScreenText {
            picture: true,
            ..word(6, "Talonflame", -40.0, 65.0, TEAM)
        },
        word(2, "L", -50.0, 45.0, TEAM),
        word(3, "50", -25.0, 45.0, TEAM),
        word(4, "TIME REMAINING:", -60.0, -105.0, TEAM),
        word(5, "1:24", 80.0, -105.0, TEAM),
    ];
    let team = keys(drawn.clone(), Some(pss::COLOSSEUM), Vec::new());
    assert_eq!(team.context(Some(PARTY)).unwrap(), Some(PSS_COLOSSEUM));
    assert!(!team.press(THEIR_TEAM, Some(PARTY)).contains("TIME"));
    assert_eq!(
        team.press(THEIR_TEAM, Some(PARTY)),
        "Brendan Talonflame L 50"
    );
    let mut opened = vec![0u8; 0x154];
    let at = (partyicons::OPENED_WITH + partyicons::TEAMS[1]) as usize;
    opened[at..at + 4].copy_from_slice(&0x08CF7200u32.to_le_bytes());
    assert_eq!(
        keys(drawn.clone(), None, opened)
            .context(Some(PARTY))
            .unwrap(),
        Some(PSS_COLOSSEUM)
    );
    assert_eq!(
        keys(drawn.clone(), None, vec![0u8; 0x154])
            .context(Some(PARTY))
            .unwrap(),
        None
    );
    // C the clock, as on every screen with one.
    assert_eq!(team.press(TIME, Some(PARTY)), "TIME REMAINING: 1:24");
    assert_eq!(
        keys(drawn, Some(FIELD), Vec::new())
            .context(Some(PARTY))
            .unwrap(),
        None
    );
    assert_eq!(
        keys(Vec::new(), Some(pss::COLOSSEUM), Vec::new()).press(THEIR_TEAM, Some(PARTY)),
        "no team shown"
    );
    assert_eq!(
        keys(Vec::new(), Some(pss::COLOSSEUM), Vec::new()).press(TIME, Some(PARTY)),
        "no timer"
    );
}

#[test]
fn test_the_keys_file_gives_each_its_table() {
    assert_eq!(
        DEFAULT_CONTEXTS[PSS_TRADE],
        IndexMap::from([
            ("r".to_string(), YOUR_OFFER.to_string()),
            ("shift+r".to_string(), THEIR_OFFER.to_string()),
        ])
    );
    assert_eq!(
        DEFAULT_CONTEXTS[PSS_COLOSSEUM],
        IndexMap::from([
            ("shift+r".to_string(), THEIR_TEAM.to_string()),
            ("c".to_string(), TIME.to_string()),
        ])
    );
}
