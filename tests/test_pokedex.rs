//! X's Pokédex list names each entry by the top screen's panel (x_pokedex).

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use super::fake_memory::{FakeClasses, FakeMemory};
use crate::games::common::cursors::shared::{CursorItem, CursorPoll, Rect};
use crate::games::common::screen::ScreenText;
use crate::games::common::screen_order::join;
use crate::games::gen6::screens::pokedex;
use crate::py::Result;

const LIST: i64 = 0x0831EEBC;
const PANEL: i64 = 0x082D14F8;

/// A piece of the screen: `ScreenText(box, text, changed, row, left, glyph=12.0, root)`.
fn piece(r#box: i64, text: &str, changed: bool, row: f64, left: f64, root: i64) -> ScreenText {
    ScreenText {
        row,
        left,
        glyph: 12.0,
        root,
        ..ScreenText::new(r#box, text, changed)
    }
}

/// Python's `ROW`.
fn list_row() -> ScreenText {
    piece(0x08322FD4, "004", true, 50.0, 49.5, LIST)
}

/// The top screen's line naming an entry: "No.", its number and its name.
fn panel(number: &str, name: &str) -> Vec<ScreenText> {
    vec![
        piece(0x082DAE70, "No.", false, 64.0, -3.8, PANEL),
        piece(0x082DBA08, number, true, 64.0, 15.0, PANEL),
        piece(0x082DC158, name, true, 64.0, 80.0, PANEL),
    ]
}

/// `ROW` followed by the panel's line.
fn with_panel(number: &str, name: &str) -> Vec<ScreenText> {
    [vec![list_row()], panel(number, name)].concat()
}

fn moved() -> CursorPoll {
    CursorPoll {
        items: vec![CursorItem::new(0x083615B4, vec![list_row()], false)],
        under: vec![vec![list_row()]],
        ..CursorPoll::default()
    }
}

/// `CursorPoll([], [(ROW,)], {}, frozenset(), frozenset())`.
fn nothing() -> CursorPoll {
    CursorPoll {
        under: vec![vec![list_row()]],
        ..CursorPoll::default()
    }
}

#[test]
fn test_a_move_is_said_with_the_name_the_panel_gives_its_number() {
    let cursors =
        pokedex::XyList::new().poll(Some(pokedex::APP), moved(), &with_panel("004", "Fennekin"));
    let [item] = &cursors.items[..] else {
        panic!("one item: {:?}", cursors.items);
    };
    assert_eq!(join(&item.pieces, false), "004 Fennekin");
    assert_eq!(
        cursors.rows,
        HashSet::from([0x082DAE70, 0x082DBA08, 0x082DC158])
    );
}

#[test]
fn test_a_move_waits_while_the_panel_names_the_entry_before() {
    let mut entries = pokedex::XyList::new();
    let cursors = entries.poll(Some(pokedex::APP), moved(), &with_panel("003", "?????"));
    assert!(cursors.items.is_empty());
    assert!(cursors.chased);
    let items = entries
        .poll(
            Some(pokedex::APP),
            nothing(),
            &with_panel("004", "Fennekin"),
        )
        .items;
    let [item] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(join(&item.pieces, false), "004 Fennekin");
}

#[test]
fn test_a_move_the_panel_never_names_is_said_by_its_number() {
    let mut entries = pokedex::XyList::new();
    entries.poll(Some(pokedex::APP), moved(), &[list_row()]);
    for _ in 0..pokedex::WAIT_POLLS - 1 {
        assert!(
            entries
                .poll(Some(pokedex::APP), nothing(), &[list_row()])
                .items
                .is_empty()
        );
    }
    let items = entries
        .poll(Some(pokedex::APP), nothing(), &[list_row()])
        .items;
    let [item] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(join(&item.pieces, false), "004");
}

#[test]
fn test_another_app_is_left_as_it_is() {
    let cursors = moved();
    assert_eq!(
        pokedex::XyList::new().poll(Some("N3app3bag4ProcE"), cursors.clone(), &[list_row()]),
        cursors
    );
}

/// `frozenset({"NAME", "COLOR", "SHAPE", "TYPE"})`.
fn labels() -> HashSet<String> {
    ["NAME", "COLOR", "SHAPE", "TYPE"]
        .iter()
        .map(|label| label.to_string())
        .collect()
}

/// Search and sort frames a field's value alone: one not set said only its dashes, one set
/// its letter alone (or_pokedex). A word beside it that is no label, as in the type grid, is
/// left out.
#[test]
fn test_a_search_field_is_said_with_its_label() {
    let other_row = piece(9, "NAME", false, 38.0, -150.0, PANEL);
    let label = piece(1, "COLOR", false, -22.0, -150.0, PANEL);
    let normal = ScreenText {
        picture: true,
        ..piece(4, "Normal", false, -22.0, -60.0, PANEL)
    };
    let unset = piece(2, "---", false, -22.0, 35.6, PANEL);
    let red = piece(3, "Red", false, -22.0, 35.6, PANEL);
    let one = |value: &ScreenText, on_screen: &[ScreenText]| -> String {
        let cursors = CursorPoll {
            items: vec![CursorItem::new(7, vec![value.clone()], false)],
            ..CursorPoll::default()
        };
        let items = pokedex::labelled(Some(pokedex::ORAS_APP), cursors, on_screen, &labels()).items;
        let [item] = &items[..] else {
            panic!("one item: {items:?}");
        };
        join(&item.pieces, false)
    };
    for (value, said) in [(&unset, "COLOR ---"), (&red, "COLOR Red")] {
        assert_eq!(
            one(
                value,
                &[
                    label.clone(),
                    other_row.clone(),
                    normal.clone(),
                    value.clone()
                ]
            ),
            said
        );
    }
    assert_eq!(one(&red, &[normal.clone(), red.clone()]), "Red");
}

const PROCESS: i64 = 0x08100000;
const LAUNCH: i64 = 0x08200000;
const NUMBER_BOX: i64 = 0x08300010;
const NAME_BOX: i64 = 0x08300020;
const PLATE_ROOT: i64 = 0x082EC748;

/// Omega Ruby's list: `ZukanLaunch` with the chosen entry in its fifth record and the panes
/// showing it in its fourth (spike 310).
fn oras_list(index: i64) -> (Rc<FakeMemory>, pokedex::OrasList) {
    let memory = FakeMemory::new();
    memory.u32(PROCESS + pokedex::LAUNCH, &[LAUNCH]);
    memory.u32(
        LAUNCH,
        &[FakeClasses::new(memory.clone()).add(pokedex::LAUNCH_CLASS)],
    );
    memory.u32(LAUNCH + pokedex::CHOSEN, &[index]);
    memory.u32(LAUNCH + pokedex::SHOWN + 4, &[NUMBER_BOX, NAME_BOX]);
    let entries = pokedex::OrasList::new(
        memory.clone(),
        Rc::new(|| Some(pokedex::ORAS_APP.to_string())),
        Rc::new(|| Some(PROCESS)),
    );
    (memory, entries)
}

fn row(number: &str, name: &str) -> Vec<ScreenText> {
    let at = |r#box: i64, text: &str, left: f64| ScreenText {
        left,
        root: PLATE_ROOT,
        ..ScreenText::new(r#box, text, true)
    };
    vec![at(NUMBER_BOX, number, 2.0), at(NAME_BOX, name, 36.0)]
}

/// The plate's own poll: its item, where it says one, from the rows sliding under it.
fn plate(pieces: &[ScreenText], moved: bool) -> CursorPoll {
    let items = if moved {
        vec![CursorItem::new(5, vec![pieces[0].clone()], false)]
    } else {
        Vec::new()
    };
    CursorPoll {
        items,
        under: vec![pieces.to_vec()],
        ..CursorPoll::default()
    }
}

/// The plate said the row sliding under it, a poll or two late; the row the game shows the
/// entry in is said once the index has held still, a jump's passing entries never.
#[test]
fn test_an_entry_is_said_in_its_row_s_words_once_the_index_holds() {
    let (memory, mut entries) = oras_list(3);
    for _ in 0..pokedex::STILL_TICKS + 1 {
        entries.moved().unwrap();
    }
    let items = entries
        .cursors(plate(&row("004", "Torchic"), false), &row("004", "Torchic"))
        .unwrap()
        .items;
    let [opened] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert!(opened.opened && join(&opened.pieces, false) == "004 Torchic");
    memory.u32(LAUNCH + pokedex::CHOSEN, &[5]);
    assert!(entries.moved().unwrap());
    let held = entries
        .cursors(
            plate(&row("005", "Combusken"), true),
            &row("006", "Blaziken"),
        )
        .unwrap();
    assert!(held.items.is_empty() && held.chased);
    memory.u32(LAUNCH + pokedex::CHOSEN, &[10]);
    entries.moved().unwrap();
    for _ in 0..pokedex::STILL_TICKS {
        assert!(!entries.moved().unwrap());
    }
    let items = entries
        .cursors(
            plate(&row("010", "Poochyena"), true),
            &row("011", "Mightyena"),
        )
        .unwrap()
        .items;
    let [item] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert!(join(&item.pieces, false) == "011 Mightyena" && !item.opened);
}

#[test]
fn test_an_index_moved_with_its_row_s_words_not_yet_waits() {
    let (memory, mut entries) = oras_list(3);
    for _ in 0..pokedex::STILL_TICKS + 1 {
        entries.moved().unwrap();
    }
    entries
        .cursors(plate(&row("004", "Torchic"), false), &row("004", "Torchic"))
        .unwrap();
    memory.u32(LAUNCH + pokedex::CHOSEN, &[4]);
    entries.moved().unwrap();
    for _ in 0..pokedex::STILL_TICKS {
        entries.moved().unwrap();
    }
    assert!(
        entries
            .cursors(plate(&row("004", "Torchic"), true), &row("004", "Torchic"))
            .unwrap()
            .items
            .is_empty()
    );
    let items = entries
        .cursors(
            plate(&row("005", "Combusken"), true),
            &row("005", "Combusken"),
        )
        .unwrap()
        .items;
    let [item] = &items[..] else {
        panic!("one item: {items:?}");
    };
    assert_eq!(join(&item.pieces, false), "005 Combusken");
}

/// Opened before its cursor came, search and sort was read whole, and its bar's "Reset
/// Confirm" came back with every picker closed (or_pokedex); a picker's label coming back is
/// said, and an entry's page, TYPE its one label, is no table.
#[test]
fn test_search_s_table_coming_on_screen_is_left_to_its_cursor() {
    let label = ScreenText {
        with_layout: true,
        ..piece(1, "NAME", true, 68.0, -150.0, PANEL)
    };
    let unset = ScreenText {
        with_layout: true,
        ..piece(2, "---", true, 68.0, 36.0, PANEL)
    };
    let color = ScreenText {
        with_layout: true,
        ..piece(5, "COLOR", true, -22.0, -150.0, PANEL)
    };
    let picker = ScreenText {
        panel: 9,
        returned: true,
        ..piece(3, "COLOR", true, 105.0, -160.0, PANEL)
    };
    let bar = ScreenText {
        returned: true,
        ..piece(4, "Reset", false, -105.0, -125.0, LIST)
    };
    let cursors = CursorPoll::default();
    let texts = [label, unset, color, picker, bar.clone()];
    assert_eq!(
        pokedex::tabled(Some(pokedex::ORAS_APP), &texts, &texts, &cursors, &labels()),
        HashSet::from([1, 2, 4, 5])
    );
    assert_eq!(
        pokedex::tabled(Some("N3app4bag4ProcE"), &texts, &texts, &cursors, &labels()),
        HashSet::new()
    );
    assert_eq!(
        pokedex::search_roots(Some(pokedex::ORAS_APP), &texts, &labels()),
        HashSet::from([PANEL])
    );
    let page = [
        ScreenText {
            root: LIST,
            with_layout: true,
            ..ScreenText::new(6, "NAME", true)
        },
        bar,
    ];
    assert_eq!(
        pokedex::tabled(Some(pokedex::ORAS_APP), &page, &page, &cursors, &labels()),
        HashSet::new()
    );
}

/// Coming on screen, the cursor framed "Reset Confirm" a poll before the table.
#[test]
fn test_the_cursor_framing_the_bar_s_buttons_both_says_nothing() {
    let at = |r#box: i64, text: &str, row: f64, left: f64, root: i64, parent: i64| ScreenText {
        row,
        left,
        root,
        parent,
        ..ScreenText::new(r#box, text, false)
    };
    let label = at(1, "NAME", 68.0, -150.0, PANEL, 0);
    let color = at(5, "COLOR", -22.0, -150.0, PANEL, 0);
    let reset = at(7, "Reset", 0.0, 0.0, LIST, 0x10);
    let confirm = at(8, "Confirm", 0.0, 0.0, LIST, 0x20);
    let both = CursorPoll {
        items: vec![CursorItem::new(
            7,
            vec![reset.clone(), confirm.clone()],
            true,
        )],
        ..CursorPoll::default()
    };
    let on_screen = [label, color, reset, confirm.clone()];
    assert!(
        pokedex::labelled(Some(pokedex::ORAS_APP), both, &on_screen, &labels())
            .items
            .is_empty()
    );
    let one = CursorPoll {
        items: vec![CursorItem::new(7, vec![confirm], false)],
        ..CursorPoll::default()
    };
    assert!(
        !pokedex::labelled(Some(pokedex::ORAS_APP), one, &on_screen, &labels())
            .items
            .is_empty()
    );
}

/// TYPE and SHAPE are blank until set, and the cursor framing them said nothing.
#[test]
fn test_a_blank_field_is_said_by_its_label() {
    let label = ScreenText {
        right: -110.0,
        ..piece(1, "COLOR", false, 8.0, -150.0, PANEL)
    };
    let other = ScreenText {
        right: -110.0,
        ..piece(2, "NAME", false, 38.0, -150.0, PANEL)
    };
    let unset = ScreenText {
        right: 30.0,
        ..piece(3, "---", false, 38.0, 10.0, PANEL)
    };
    let rect = Rect {
        top: false,
        left: 0.0,
        right: 120.0,
        low: -5.0,
        high: 20.0,
    };
    let said = pokedex::blank(
        Some(pokedex::ORAS_APP),
        &rect,
        &[label.clone(), other.clone(), unset],
        &[],
        &labels(),
    );
    assert_eq!(said, vec![label.clone()]);
    assert_eq!(
        pokedex::blank(Some(pokedex::ORAS_APP), &rect, &[other], &[], &labels()),
        Vec::new()
    );
    assert_eq!(
        pokedex::blank(Some("N3app4bag4ProcE"), &rect, &[label], &[], &labels()),
        Vec::new()
    );
}

/// SHAPE set said its shape alone, "Finned" (heard in play); a picker's label stands beside
/// whichever choice scrolls onto its row, and is said only for its blank row.
#[test]
fn test_a_picture_framed_takes_the_table_s_label_alone() {
    let at = |r#box: i64, text: &str, row: f64, left: f64, right: f64, root: i64| ScreenText {
        right,
        ..piece(r#box, text, false, row, left, root)
    };
    let shape = at(1, "SHAPE", 8.0, -150.0, -110.0, PANEL);
    let color = at(2, "COLOR", 38.0, -150.0, -110.0, PANEL);
    let picker = at(3, "TYPE", 8.0, -160.0, -120.0, PANEL + 0x100);
    let finned = ScreenText {
        root: PANEL,
        picture: true,
        ..ScreenText::new(4, "Finned", false)
    };
    let red = at(5, "Red", 38.0, 10.0, 30.0, PANEL);
    let rect = Rect {
        top: false,
        left: 0.0,
        right: 120.0,
        low: -5.0,
        high: 20.0,
    };
    let framed = [finned];
    assert_eq!(
        pokedex::blank(
            Some(pokedex::ORAS_APP),
            &rect,
            &[shape.clone(), color, red],
            &framed,
            &labels()
        ),
        vec![shape]
    );
    assert_eq!(
        pokedex::blank(
            Some(pokedex::ORAS_APP),
            &rect,
            std::slice::from_ref(&picker),
            &framed,
            &labels()
        ),
        Vec::new()
    );
    assert_eq!(
        pokedex::blank(
            Some(pokedex::ORAS_APP),
            &rect,
            std::slice::from_ref(&picker),
            &[],
            &labels()
        ),
        vec![picker]
    );
}

/// X draws its shapes in a column beside the table, level with its rows, and the cursor on
/// them said "ORDER Bodiless" and "COLOR Finned" (x_pokedex_search_shapes): a picture is the
/// value of a row with none in words, while the table shows its values.
#[test]
fn test_a_picture_beside_a_row_with_a_value_takes_no_label() {
    let at = |r#box: i64, text: &str, row: f64, left: f64, right: f64| ScreenText {
        right,
        ..piece(r#box, text, false, row, left, PANEL)
    };
    let name = at(1, "NAME", 8.0, -150.0, -110.0);
    let unset = at(3, "---", 8.0, 10.0, 30.0);
    let color = at(2, "COLOR", 38.0, -150.0, -110.0);
    let shape = [ScreenText {
        root: PANEL,
        picture: true,
        ..ScreenText::new(4, "Finned", false)
    }];
    let rect = Rect {
        top: false,
        left: 40.0,
        right: 120.0,
        low: -5.0,
        high: 20.0,
    };
    let red = at(5, "Red", 38.0, 10.0, 30.0);
    let blank = |on_screen: &[ScreenText], framed: &[ScreenText]| {
        pokedex::blank(Some(pokedex::APP), &rect, on_screen, framed, &labels())
    };
    assert_eq!(
        blank(&[name.clone(), unset, color.clone(), red.clone()], &shape),
        Vec::new()
    );
    assert_eq!(
        blank(&[name.clone(), color.clone(), red], &shape),
        vec![name.clone()]
    );
    // The picker shown, the table's values hidden: a label for no picture, nor its blank.
    assert_eq!(blank(&[name.clone(), color.clone()], &shape), Vec::new());
    assert_eq!(blank(&[name, color], &[]), Vec::new());
}

const GRAPHICS: i64 = 0x08400000;
const VIEW: i64 = 0x085000C4;
const REGIONS_ROOT: i64 = 0x082D56B8;
const BAR_ROOT: i64 = 0x08600000;

/// The layouts on screen's files: the regions' root alone, `top_lower.bclyt`.
fn regions_files() -> pokedex::Files {
    Box::new(|_roots| [(REGIONS_ROOT, pokedex::REGIONS.to_string())].into())
}

/// X's regions: the process's `Graphics`, its view of the screen up now the regions', the
/// rows it has built and the region under the cursor (spike A on x_pokedex). Python's `built`
/// defaults to 3.
fn regions(region: i64, built: i64) -> (Rc<FakeMemory>, pokedex::XyRegions) {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    memory.u32(PROCESS + pokedex::GRAPHICS, &[GRAPHICS]);
    memory.u32(GRAPHICS, &[classes.add(pokedex::GRAPHICS_CLASS)]);
    memory.u32(GRAPHICS + pokedex::VIEW, &[VIEW]);
    memory.u32(VIEW, &[classes.add(pokedex::VIEW_CLASS)]);
    memory.u32(VIEW + pokedex::BUILT, &[built, region]);
    let source = pokedex::XyRegions::new(
        memory.clone(),
        Rc::new(|| Some(pokedex::APP.to_string())),
        Rc::new(|| Some(PROCESS)),
        regions_files(),
    );
    (memory, source)
}

/// The regions' rows, each a name and its count, top first, with a word of another layout on
/// the bottom screen below them.
fn screen() -> Vec<ScreenText> {
    let rows = [
        ("CENTRAL KALOS POKÉDEX", "14", 25.0),
        ("COASTAL KALOS POKÉDEX", "5", 0.0),
        ("MOUNTAIN KALOS POKÉDEX", "4", -25.0),
    ];
    // Each count listed before its name: a row is read left to right.
    let mut words: Vec<ScreenText> = Vec::new();
    for (n, (name, count, row)) in rows.into_iter().enumerate() {
        for (half, (left, text)) in [(80.0, count), (-100.0, name)].into_iter().enumerate() {
            words.push(piece(
                0x100 + 2 * n as i64 + half as i64,
                text,
                false,
                row,
                left,
                REGIONS_ROOT,
            ));
        }
    }
    words.push(ScreenText {
        picture: true,
        ..piece(0x200, "Back", false, -100.0, 120.0, BAR_ROOT)
    });
    words
}

/// Python's `nothing()` of the regions' tests: `CursorPoll([], [], {}, frozenset(),
/// frozenset())`.
fn no_cursors() -> CursorPoll {
    CursorPoll::default()
}

/// X's regions were said whole, as no cursor of theirs is drawn (x_pokedex). The view's
/// region is said in its row's words once it has built the rows shown, as they open, then each
/// move, and nothing where the region holds: left and right move nothing.
#[test]
fn test_the_region_under_the_cursor_is_said_in_its_row_s_words() -> Result<()> {
    let (memory, mut source) = regions(0, 1);
    assert!(source.moved()?);
    assert!(source.cursors(no_cursors(), &screen()).items.is_empty());
    memory.u32(VIEW + pokedex::BUILT, &[3]);
    assert!(source.moved()?);
    let items = source.cursors(no_cursors(), &screen()).items;
    let [opened] = items.as_slice() else {
        panic!("one item: {items:?}");
    };
    assert!(opened.opened && join(&opened.pieces, false) == "CENTRAL KALOS POKÉDEX 14");
    for (region, said) in [
        (1, "COASTAL KALOS POKÉDEX 5"),
        (2, "MOUNTAIN KALOS POKÉDEX 4"),
        (0, "CENTRAL KALOS POKÉDEX 14"),
    ] {
        memory.u32(VIEW + pokedex::REGION, &[region]);
        assert!(source.moved()?);
        let items = source.cursors(no_cursors(), &screen()).items;
        let [item] = items.as_slice() else {
            panic!("one item: {items:?}");
        };
        assert!(!item.opened && join(&item.pieces, false) == said, "{said}");
    }
    assert!(!source.moved()?);
    assert!(source.cursors(no_cursors(), &screen()).items.is_empty());
    Ok(())
}

/// The list's view in the regions' place: back to them, the region is said as they open.
#[test]
fn test_the_regions_are_said_afresh_as_they_come_back() -> Result<()> {
    let (memory, mut source) = regions(1, 3);
    source.moved()?;
    source.cursors(no_cursors(), &screen());
    memory.u32(
        VIEW,
        &[FakeClasses::starting(memory.clone(), 0x00180000).add("N3app5zukan4list10ScrollViewE")],
    );
    assert!(!source.moved()?);
    assert!(source.cursors(no_cursors(), &[]).items.is_empty());
    memory.u32(
        VIEW,
        &[FakeClasses::starting(memory.clone(), 0x00190000).add(pokedex::VIEW_CLASS)],
    );
    assert!(source.moved()?);
    let items = source.cursors(no_cursors(), &screen()).items;
    let [item] = items.as_slice() else {
        panic!("one item: {items:?}");
    };
    assert!(item.opened && join(&item.pieces, false) == "COASTAL KALOS POKÉDEX 5");
    Ok(())
}

#[test]
fn test_the_regions_are_read_in_x_s_pokedex_alone() -> Result<()> {
    let (memory, _source) = regions(0, 3);
    let mut source = pokedex::XyRegions::new(
        memory,
        Rc::new(|| Some(pokedex::ORAS_APP.to_string())),
        Rc::new(|| Some(PROCESS)),
        regions_files(),
    );
    let cursors = no_cursors();
    assert!(!source.moved()?);
    assert_eq!(source.cursors(cursors.clone(), &screen()), cursors);
    Ok(())
}

const CHOICE_ROOT: i64 = 0x082EA34C;
const REGISTRATION_ROOT: i64 = 0x082E83EC;

/// Omega Ruby's opening, its choice's layout and the registration's known by their files.
fn opening(running: Rc<RefCell<String>>) -> pokedex::OrasOpening {
    pokedex::OrasOpening::new(
        Rc::new(move || Some(running.borrow().clone())),
        Box::new(|_roots| {
            [
                (CHOICE_ROOT, pokedex::CHOICE.to_string()),
                (REGISTRATION_ROOT, pokedex::REGISTRATION.to_string()),
            ]
            .into()
        }),
    )
}

/// The choice's cursor came as the app's whole sweep came due, and its first choice was said
/// after the sweep (spike B): the opening waits from the app's start, through the choice's words
/// coming, until a cursor is on screen, and not again until the Pokédex starts again.
#[test]
fn test_the_sweep_waits_for_omega_ruby_s_choice_s_cursor() {
    let running = Rc::new(RefCell::new(pokedex::ORAS_APP.to_string()));
    let mut source = opening(running.clone());
    assert!(source.waiting());
    source.poll(&no_cursors(), &[]);
    let hoenn = ScreenText {
        root: CHOICE_ROOT,
        ..ScreenText::new(0x100, "HOENN POKÉDEX", false)
    };
    source.poll(&no_cursors(), std::slice::from_ref(&hoenn));
    assert!(source.waiting());
    source.poll(
        &CursorPoll {
            items: vec![CursorItem::new(0x08331B94, vec![hoenn.clone()], true)],
            under: vec![vec![hoenn.clone()]],
            ..CursorPoll::default()
        },
        std::slice::from_ref(&hoenn),
    );
    assert!(!source.waiting());
    *running.borrow_mut() = "N5field14FieldmapProcE".to_string();
    assert!(!source.waiting());
    *running.borrow_mut() = pokedex::ORAS_APP.to_string();
    assert!(source.waiting());
}

/// The registration is the same app, with no cursor until its nickname's YES and NO: its
/// words end the wait, so its sweep comes as it did (or_groudon_master_ball).
#[test]
fn test_the_registration_after_a_catch_does_not_wait() {
    let mut source = opening(Rc::new(RefCell::new(pokedex::ORAS_APP.to_string())));
    assert!(source.waiting());
    source.poll(
        &no_cursors(),
        &[ScreenText {
            root: REGISTRATION_ROOT,
            top: true,
            ..ScreenText::new(0x100, "Registration completed!", false)
        }],
    );
    assert!(!source.waiting());
    assert!(!opening(Rc::new(RefCell::new(pokedex::APP.to_string()))).waiting());
}
