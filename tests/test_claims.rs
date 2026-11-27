//! Whose words (`gen6.screens.claims`): what screen text leaves to the readers that say some of a
//! screen's words themselves, and a button's words to touch mode.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;

use super::test_announce::{Screen, args, piece};
use crate::games::common::cursors::shared::{CursorItem, CursorPoll};
use crate::games::common::screen::{ScreenPoll, ScreenText};
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::screens::claims::{self, ButtonRule, Claims, Owner, PSS, RULES};
use crate::games::gen6::screens::pokedex;
use crate::games::gen6::screens::sources::Polled;
use crate::py::Result;

const PAGE: i64 = 0x08100000;
const MENU: i64 = 0x08200000;
const TOP: i64 = 0x08300000;

/// Words on the bottom screen, come on screen with their layout or changed.
fn bottom(r#box: i64, text: &str, root: i64, x: f64, changed: bool) -> ScreenText {
    ScreenText {
        changed,
        with_layout: !changed,
        ..piece(r#box, text, x, 0.0, root)
    }
}

/// `bottom(box, text, root)`: at x 0, come with its layout.
fn shown(r#box: i64, text: &str, root: i64) -> ScreenText {
    bottom(r#box, text, root, 0.0, false)
}

/// `poll(texts)`: what is on screen is what came.
fn poll(texts: &[ScreenText]) -> ScreenPoll {
    ScreenPoll {
        texts: texts.to_vec(),
        on_screen: texts.to_vec(),
        ..ScreenPoll::default()
    }
}

/// The cursor sources' poll, a cursor over `under` where any is given.
fn polled_with(under: &[ScreenText], unsaid: &[i64], unseen: &[i64], labels: &[&str]) -> Polled {
    let items = if under.is_empty() {
        Vec::new()
    } else {
        vec![CursorItem::new(0x0C, under.to_vec(), true)]
    };
    Polled {
        cursors: CursorPoll {
            items,
            under: if under.is_empty() {
                Vec::new()
            } else {
                vec![under.to_vec()]
            },
            ..CursorPoll::default()
        },
        title: None,
        chosen: IndexMap::new(),
        before: Vec::new(),
        after: Vec::new(),
        unsaid: unsaid.iter().copied().collect(),
        unseen: unseen.iter().copied().collect(),
        labels: labels.iter().map(|label| label.to_string()).collect(),
    }
}

/// `polled(*under)`.
fn polled(under: &[ScreenText]) -> Polled {
    polled_with(under, &[], &[], &[])
}

/// A fake set of buttons: the TextBoxes naming one, and how often they were asked for.
struct Buttons {
    boxes: HashSet<i64>,
    asked: Cell<i64>,
}

impl Buttons {
    fn new(boxes: &[i64]) -> Rc<Buttons> {
        Rc::new(Buttons {
            boxes: boxes.iter().copied().collect(),
            asked: Cell::new(0),
        })
    }

    /// `buttons()`.
    fn call(&self) -> Result<HashSet<i64>> {
        self.asked.set(self.asked.get() + 1);
        Ok(self.boxes.clone())
    }
}

/// `RULES[FIELD]`.
fn field_rule() -> ButtonRule {
    RULES
        .iter()
        .find(|(app, _)| *app == FIELD)
        .map(|(_, rule)| *rule)
        .expect("the field's rule")
}

/// `field(buttons, rule)`, no words' panes named.
fn field(buttons: &Rc<Buttons>, rule: Option<ButtonRule>) -> Claims {
    field_named(buttons, rule, HashMap::new())
}

/// The claims on a screen whose rule is `rule`, its words' panes `named` by their boxes.
fn field_named(
    buttons: &Rc<Buttons>,
    rule: Option<ButtonRule>,
    named: HashMap<i64, (String, String)>,
) -> Claims {
    let buttons = buttons.clone();
    Claims::new(
        Vec::new(),
        Rc::new(|_piece: &ScreenText| false),
        Rc::new(move || buttons.call()),
        Rc::new(move || rule),
        Rc::new(move |piece: &ScreenText| named.get(&piece.r#box).cloned()),
    )
}

fn no_labels() -> HashSet<String> {
    HashSet::new()
}

/// ADR-0013: the PSS's Friends, Acquaintances and Passersby were said each time the field came
/// back (or_pss_singleplayer). The rest of the bottom screen is still said, and the top's.
#[test]
fn test_a_buttons_words_coming_on_screen_are_left_to_touch_mode_and_kept_quiet() -> Result<()> {
    let (friends, passersby) = (
        shown(1, "Friends", PAGE),
        bottom(2, "Passersby", PAGE, 100.0, false),
    );
    let news = bottom(3, "It couldn't be found nearby", PAGE, 200.0, false);
    let up = ScreenText {
        top: true,
        ..shown(4, "Route 101", TOP)
    };
    let words = [friends.clone(), passersby.clone(), news.clone(), up.clone()];
    let heard = field(&Buttons::new(&[1, 2]), Some(field_rule())).poll(
        &poll(&words),
        &polled(&[]),
        &no_labels(),
    )?;
    assert_eq!(heard.texts, [news.clone(), up.clone()]);
    assert_eq!(heard.quiet, HashSet::from([1, 2]));
    assert_eq!(heard.heard, words);
    assert_eq!(
        Screen::new().say(args(&heard.texts, vec![], &heard.heard)),
        ["Route 101", "It couldn't be found nearby"]
    );
    Ok(())
}

/// "Game Chat: Disabled" is said; L and R page O-Power's list, and its seven powers were read as
/// each page came (or_pss_o_power_2).
#[test]
fn test_one_buttons_words_changing_are_said_but_a_page_of_them_is_turned_unsaid() -> Result<()> {
    let chat = bottom(1, "Disabled", PAGE, 0.0, true);
    let news = bottom(5, "Game Chat", PAGE, 100.0, false);
    let words = [chat.clone(), news.clone()];
    assert_eq!(
        field(&Buttons::new(&[1]), Some(field_rule()))
            .poll(&poll(&words), &polled(&[]), &no_labels())?
            .texts,
        words
    );
    let powers: Vec<ScreenText> = [1, 2]
        .iter()
        .map(|&r#box| {
            bottom(
                r#box,
                &format!("Power {}", r#box),
                PAGE,
                50.0 * r#box as f64,
                true,
            )
        })
        .collect();
    assert!(
        field(&Buttons::new(&[1, 2]), Some(field_rule()))
            .poll(&poll(&powers), &polled(&[]), &no_labels())?
            .texts
            .is_empty()
    );
    let paged = field(
        &Buttons::new(&[1, 2]),
        Some(ButtonRule {
            pages: false,
            ..ButtonRule::default()
        }),
    );
    assert_eq!(
        paged
            .poll(&poll(&powers), &polled(&[]), &no_labels())?
            .texts,
        powers
    );
    Ok(())
}

/// One change with nothing new is said as it is, and the walk of the touch areas skipped.
#[test]
fn test_buttons_are_asked_for_only_where_something_comes_or_a_page_may_turn() -> Result<()> {
    let buttons = Buttons::new(&[1]);
    let heard = field(&buttons, Some(field_rule())).poll(
        &poll(&[bottom(1, "Disabled", PAGE, 0.0, true)]),
        &polled(&[]),
        &no_labels(),
    )?;
    assert!(buttons.asked.get() == 0 && heard.texts.len() == 1 && heard.quiet.is_empty());
    Ok(())
}

/// The X menu's words are its items; the PSS's apps have no menu, and the profile's Message row,
/// lit as it came back from its messages, kept its words for the cursor it was taken for
/// (or_pss_singleplayer_2).
#[test]
fn test_a_layout_a_cursor_is_over_keeps_its_buttons_words_on_the_field_alone() -> Result<()> {
    let (bag, save) = (shown(1, "BAG", MENU), bottom(2, "SAVE", MENU, 100.0, false));
    let menu = poll(&[bag.clone(), save.clone()]);
    let heard = field(&Buttons::new(&[1, 2]), Some(field_rule())).poll(
        &menu,
        &polled(std::slice::from_ref(&bag)),
        &no_labels(),
    )?;
    assert!(heard.texts == [bag.clone(), save.clone()] && heard.quiet.is_empty());
    let heard = field(&Buttons::new(&[1, 2]), Some(PSS)).poll(
        &menu,
        &polled(std::slice::from_ref(&bag)),
        &no_labels(),
    )?;
    assert!(heard.texts.is_empty() && heard.quiet == HashSet::from([1, 2]));
    Ok(())
}

#[test]
fn test_a_screen_with_no_rule_keeps_its_buttons_words() -> Result<()> {
    let buttons = Buttons::new(&[1]);
    let heard =
        field(&buttons, None).poll(&poll(&[shown(1, "Next", PAGE)]), &polled(&[]), &no_labels())?;
    assert!(heard.texts.len() == 1 && heard.quiet.is_empty() && buttons.asked.get() == 0);
    Ok(())
}

#[test]
fn test_the_field_takes_its_rule_and_the_pss_its_own() {
    let asked: RefCell<Vec<bool>> = RefCell::new(Vec::new());
    let in_pss = || {
        asked.borrow_mut().push(true);
        true
    };
    assert!(claims::rule(Some(FIELD), &in_pss) == Some(field_rule()) && asked.borrow().is_empty());
    assert_eq!(claims::rule(Some("N3app3pss7ProcE"), &in_pss), Some(PSS));
    assert_eq!(claims::rule(None, &|| false), None);
}

/// The battle's HP box is its reader's to say, and its other panels are read beside it; the
/// AreaNav's Pokémon Distribution's icons are not gathered with the rest of their screen, as a
/// rewrite reads again the groups it has met. So with what the cursor sources say or do not
/// draw; and their labels join the list readers'.
#[test]
fn test_a_readers_words_are_left_unsaid_but_heard_and_the_games_own_neither() -> Result<()> {
    let (hp, name) = (
        shown(1, "18/20", PAGE),
        bottom(2, "Torchic", PAGE, 100.0, false),
    );
    let (icon, said, hidden) = (
        bottom(3, "Surskit", PAGE, 200.0, false),
        shown(4, "BOX 1", MENU),
        shown(5, "Memo", TOP),
    );
    let words = [hp.clone(), name.clone(), icon, said, hidden];
    let heard = Claims::new(
        vec![Rc::new(|p: &ScreenText| Ok(p.r#box == 1)) as Owner],
        Rc::new(|p: &ScreenText| p.r#box == 3),
        {
            let buttons = Buttons::new(&[]);
            Rc::new(move || buttons.call())
        },
        Rc::new(|| None),
        Rc::new(|_piece: &ScreenText| None),
    )
    .poll(
        &poll(&words),
        &polled_with(&[], &[4], &[TOP], &["BOX 1"]),
        &HashSet::from(["YES".to_string()]),
    )?;
    assert_eq!(heard.texts, std::slice::from_ref(&name));
    assert_eq!(heard.heard, [hp, name]);
    assert_eq!(
        heard.labels,
        HashSet::from(["YES".to_string(), "BOX 1".to_string()])
    );
    Ok(())
}

/// ADR-0013, amended 2026-11-24: the choices read "HOENN POKÉDEX Number caught: 36 Number
/// seen: 181 NATIONAL POKÉDEX…" as the Pokédex opened, and its cursor the first again
/// (or_pokedex). Left to the cursor as the field's are, once it is over them.
#[test]
fn test_the_pokedexs_buttons_words_are_left_to_its_cursors() -> Result<()> {
    let hoenn = shown(1, "HOENN POKÉDEX", MENU);
    let national = bottom(2, "NATIONAL POKÉDEX", MENU, 100.0, false);
    let choices_up = [hoenn.clone(), national.clone()];
    let rule = claims::rule(Some(pokedex::ORAS_APP), &|| false);
    assert!(rule.is_some_and(|rule| rule.cursor_keeps));
    let pane = |name: &str| (pokedex::CHOICE.to_string(), name.to_string());
    let choices = field_named(
        &Buttons::new(&[1, 2]),
        rule,
        HashMap::from([(1, pane("6")), (2, pane("l"))]),
    );
    assert!(
        choices
            .poll(&poll(&choices_up), &polled(&[]), &no_labels())?
            .texts
            .is_empty()
    );
    assert_eq!(
        choices
            .poll(
                &poll(&choices_up),
                &polled(std::slice::from_ref(&hoenn)),
                &no_labels()
            )?
            .texts,
        choices_up
    );
    Ok(())
}

/// ADR-0013, amended 2026-11-25: only as long as the item under the cursor is said. More and
/// Habitat, on the bar its cursor walks, are left to it (or_pokedex); a touch area's words in a
/// layout no rule names are said as they come, the More list's rows (its list's labels, left
/// out anyway), and so are words whose layout's file is not known.
#[test]
fn test_the_pokedex_leaves_only_the_layouts_its_cursors_walk() -> Result<()> {
    let (more, habitat) = (
        shown(1, "More", MENU),
        bottom(2, "Habitat", MENU, 100.0, false),
    );
    let row = shown(3, "APPEARANCE/CRY", PAGE);
    let unnamed = shown(4, "Search/Sort", TOP);
    let rule = claims::rule(Some(pokedex::ORAS_APP), &|| false);
    let pane = |file: &str, name: &str| (file.to_string(), name.to_string());
    let named = HashMap::from([
        (1, pane(pokedex::TOUCH_BAR, "69")),
        (2, pane(pokedex::TOUCH_BAR, "6r")),
        (3, pane("menu_app.bclyt", "6")),
    ]);
    let heard = field_named(&Buttons::new(&[1, 2, 3, 4]), rule, named).poll(
        &poll(&[more, habitat, row.clone(), unnamed.clone()]),
        &polled(&[]),
        &no_labels(),
    )?;
    assert_eq!(heard.texts, [row, unnamed]);
    assert_eq!(heard.quiet, HashSet::from([1, 2]));
    Ok(())
}

/// ADR-0013, amended 2026-11-24: the Pokédex's choices may go unsaid only as long as the item
/// under its cursor is said. X's regions were said whole, "CENTRAL KALOS POKÉDEX 14 COASTAL
/// KALOS POKÉDEX 5…", until their source said the one under the cursor (2026-11-25,
/// `pokedex.XyRegions`): they go with the list's numbers.
#[test]
fn test_xs_regions_are_left_to_their_source() -> Result<()> {
    let central = shown(1, "CENTRAL KALOS POKÉDEX", MENU);
    let coastal = bottom(2, "COASTAL KALOS POKÉDEX", MENU, 100.0, false);
    let number = shown(3, "001", PAGE);
    let rule = claims::rule(Some(pokedex::APP), &|| false);
    let pane = |file: &str, name: &str| (file.to_string(), name.to_string());
    let named = HashMap::from([
        (1, pane(pokedex::REGIONS, "f")),
        (2, pane(pokedex::REGIONS, "r")),
        (3, pane(pokedex::LIST, "n")),
    ]);
    let heard = field_named(&Buttons::new(&[1, 2, 3]), rule, named).poll(
        &poll(&[central, coastal, number]),
        &polled(&[]),
        &no_labels(),
    )?;
    assert!(heard.texts.is_empty());
    assert_eq!(heard.quiet, HashSet::from([1, 2, 3]));
    Ok(())
}

/// The registration said "Rotate Zoom Next" after each entry (or_groudon_master_ball): Next is
/// a button, and "Rotate" and "Zoom" hang from pictures over the model's four arrows, touch areas
/// with no words of their own. They go by their panes: the entry beside them is still said, a
/// touch area's words or not.
#[test]
fn test_the_registrations_captions_over_its_arrows_go_with_its_buttons_words() -> Result<()> {
    let (rotate, zoom) = (
        shown(1, "Rotate", PAGE),
        bottom(2, "Zoom", PAGE, 60.0, false),
    );
    let (after, entry) = (
        bottom(3, "Next", PAGE, 120.0, false),
        bottom(4, "Continent Pokémon", PAGE, 0.0, false),
    );
    let rule = claims::rule(Some(pokedex::ORAS_APP), &|| false);
    let pane = |name: &str| (pokedex::REGISTRATION.to_string(), name.to_string());
    let named = HashMap::from([
        (1, pane("3s")),
        (2, pane("3w")),
        (3, pane("2v")),
        (4, pane("1a")),
    ]);
    let heard = field_named(&Buttons::new(&[3, 4]), rule, named).poll(
        &poll(&[rotate, zoom, after, entry.clone()]),
        &polled(&[]),
        &no_labels(),
    )?;
    assert_eq!(heard.texts, [entry]);
    assert_eq!(heard.quiet, HashSet::from([1, 2, 3]));
    Ok(())
}
