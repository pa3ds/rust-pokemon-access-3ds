//! ORAS control-code tests. The byte layouts follow spike 11; the words are made up.

use crate::games::common::garc::{plain, singular};
use crate::games::common::text::{Control, Token, pages, tokens};
use crate::py::Kind;

fn text(value: &str) -> Vec<u8> {
    value.encode_utf16().flat_map(u16::to_le_bytes).collect()
}

fn units(raw: &[u8]) -> Vec<i64> {
    raw.as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair) as i64)
        .collect()
}

fn halfwords(values: &[i64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|&value| (value as u16).to_le_bytes())
        .collect()
}

fn code(value: i64, args: &[i64]) -> Vec<u8> {
    let mut values = vec![0x0010, 1 + args.len() as i64, value];
    values.extend(args);
    halfwords(&values)
}

fn texts(raw: &[u8]) -> Vec<String> {
    pages(raw)
        .unwrap()
        .into_iter()
        .map(|page| page.text)
        .collect()
}

fn word(value: &str) -> Token {
    Token::Text(value.to_string())
}

#[test]
fn test_plain_text_is_one_token() {
    assert_eq!(
        tokens(&text("Hello there.")).unwrap(),
        [word("Hello there.")]
    );
}

#[test]
fn test_a_control_sequence_splits_the_text() {
    let raw = [text("one"), code(0xBE01, &[]), text("two")].concat();
    assert_eq!(
        tokens(&raw).unwrap(),
        [
            word("one"),
            Token::Control(Control::bare(0xBE01)),
            word("two")
        ]
    );
}

#[test]
fn test_arguments_are_consumed_not_read_as_text() {
    let raw = [
        text("a "),
        code(0xFF00, &[2]),
        text("red"),
        code(0xFF00, &[0]),
        text(" word"),
    ]
    .concat();
    assert_eq!(
        tokens(&raw).unwrap(),
        [
            word("a "),
            Token::Control(Control::new(0xFF00, &[2])),
            word("red"),
            Token::Control(Control::new(0xFF00, &[0])),
            word(" word")
        ]
    );
}

#[test]
fn test_an_argument_that_is_half_a_surrogate_pair_does_not_break_decoding() {
    let raw = [text("x"), code(0x1234, &[0xD800]), text("y")].concat();
    assert_eq!(
        tokens(&raw).unwrap(),
        [
            word("x"),
            Token::Control(Control::new(0x1234, &[0xD800])),
            word("y")
        ]
    );
}

/// A Chinese patch leaves other halfwords after a line's zero (rust-pokemon-access-3ds#34).
#[test]
fn test_a_zero_ends_the_text_but_not_inside_a_sequence() {
    let raw = [text("小"), code(0xFF00, &[0]), text("悠\0비づ")].concat();
    assert_eq!(
        tokens(&raw).unwrap(),
        [
            word("小"),
            Token::Control(Control::new(0xFF00, &[0])),
            word("悠")
        ]
    );
    assert_eq!(texts(&raw), ["小悠"]);
}

#[test]
fn test_text_outside_the_basic_plane_survives() {
    assert_eq!(
        tokens(&text("\u{1f600} ok")).unwrap(),
        [word("\u{1f600} ok")]
    );
}

#[test]
fn test_a_sequence_running_past_the_end_is_an_error() {
    let raw = [text("x"), halfwords(&[0x0010, 4, 0xFF00])].concat();
    let error = tokens(&raw).unwrap_err();
    assert!(error.is(Kind::ValueError) && error.message.contains("claims 4"));
}

#[test]
fn test_a_marker_at_the_very_end_is_an_error() {
    let error = tokens(&[text("x"), halfwords(&[0x0010])].concat()).unwrap_err();
    assert!(error.is(Kind::ValueError) && error.message.contains("cut off"));
}

#[test]
fn test_an_odd_byte_count_is_an_error() {
    let error = tokens(b"abc").unwrap_err();
    assert!(error.is(Kind::ValueError) && error.message.contains("cannot be 3 bytes"));
}

#[test]
fn test_pages_split_at_both_page_breaks() {
    let raw = [
        text("First."),
        code(0xBE01, &[]),
        text("\nSecond."),
        code(0xBE00, &[]),
        text("\nThird."),
    ]
    .concat();
    assert_eq!(texts(&raw), ["First.", "Second.", "Third."]);
}

#[test]
fn test_line_breaks_within_a_page_are_spoken_as_spaces() {
    assert_eq!(
        texts(&text("Wrapped at the\nedge of the box.")),
        ["Wrapped at the edge of the box."]
    );
}

#[test]
fn test_colour_codes_are_not_spoken() {
    let raw = [
        text("Go see "),
        code(0xFF00, &[2]),
        text("the lab"),
        code(0xFF00, &[0]),
        text(" now."),
    ]
    .concat();
    assert_eq!(texts(&raw), ["Go see the lab now."]);
}

#[test]
fn test_every_page_is_kept_so_an_index_matches_the_game() {
    let raw = [text("Only page."), code(0xBE01, &[]), text("\n")].concat();
    assert_eq!(texts(&raw), ["Only page.", ""]);
}

#[test]
fn test_no_text_is_one_empty_page() {
    assert_eq!(texts(b""), [""]);
}

#[test]
fn test_lines_split_at_line_breaks() {
    assert_eq!(
        pages(&text("top\nbottom")).unwrap()[0].lines,
        [units(&text("top")), units(&text("bottom"))]
    );
}

#[test]
fn test_lines_keep_their_control_sequences_as_the_game_types_them() {
    let raw = [text("Go "), code(0xFF00, &[2]), text("red")].concat();
    assert_eq!(pages(&raw).unwrap()[0].lines, [units(&raw)]);
}

#[test]
fn test_the_newline_after_a_page_break_is_not_a_line() {
    let raw = [text("A"), code(0xBE01, &[]), text("\nB")].concat();
    assert_eq!(pages(&raw).unwrap()[1].lines, [units(&text("B"))]);
}

#[test]
fn test_a_newline_inside_control_arguments_is_not_a_line_break() {
    let raw = [text("a"), code(0x1234, &[0x000A]), text("b")].concat();
    assert_eq!(pages(&raw).unwrap()[0].lines, [units(&raw)]);
}

#[test]
fn test_singular_drops_the_plural_ending_after_a_number_form() {
    // X's English interior items, as file 92 stores them: no letters singular, then "s" plural.
    let raw = [
        text("Leaf Cushion"),
        code(0x1101, &[0x00FE, 0x0100]),
        text("s"),
    ]
    .concat();
    assert_eq!(
        (
            plain(&raw),
            singular(&raw),
            singular(&text("Mysterious Eggs"))
        ),
        (
            "Leaf Cushions".to_string(),
            "Leaf Cushion".to_string(),
            "Mysterious Eggs".to_string()
        )
    );
}
