//! Game text as ORAS stores it: UTF-16LE with control sequences inline.
//!
//! ```text
//! u16  0x0010   marker
//! u16  count    halfwords after this one
//! u16  code
//! u16  args     count - 1 of them
//! ```
//!
//! Read from the game's own text walker (spike 11). Parse on halfwords, before decoding: an argument
//! can be any value, including half a surrogate pair or zero, and either breaks a decode of the
//! whole string.
//!
//! By the time text reaches a message window the player's name and other inserts are already
//! substituted, so what is left is formatting — `0xff00` is colour — and the two page breaks, where
//! the box stops and waits for A. `0xbe01` then clears the box; `0xbe00` scrolls it up one line
//! (spike 14).

use crate::py::{self, Error, Result};

pub const MARKER: i64 = 0x0010;
pub const END: i64 = 0x0000;
pub const NEWLINE: i64 = 0x000A;
pub const PAGE_CLEAR: i64 = 0xBE01;
pub const PAGE_SCROLL: i64 = 0xBE00;
pub const PAGE_BREAKS: [i64; 2] = [PAGE_CLEAR, PAGE_SCROLL];

/// One control sequence: its code and its arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct Control {
    pub code: i64,
    pub args: Vec<i64>,
}

impl Control {
    pub fn new(code: i64, args: &[i64]) -> Self {
        Control {
            code,
            args: args.to_vec(),
        }
    }

    /// `Control(code)`: no arguments.
    pub fn bare(code: i64) -> Self {
        Control::new(code, &[])
    }
}

/// `type Token = str | Control`.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Text(String),
    Control(Control),
}

pub type Line = Vec<i64>;

/// What the box shows between two waits for A.
///
/// `lines` are halfwords with control sequences left in, which is exactly how the game types
/// each one into a TextBox. `text` is the page ready to speak.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub lines: Vec<Line>,
    pub text: String,
}

/// Split UTF-16LE bytes into runs of text and the control sequences between them.
///
/// Raises ValueError for text that is not whole halfwords or a sequence that runs past the end;
/// either means the read was wrong, and a guess at the rest would be spoken as if it were right.
pub fn tokens(raw: &[u8]) -> Result<Vec<Token>> {
    let units = units(raw)?;
    Ok(walk(&units)?
        .into_iter()
        .map(|(s, e, c)| match c {
            None => Token::Text(decode(py::slice(&units, s, e))),
            Some(c) => Token::Control(c),
        })
        .collect())
}

/// The message split where the box waits for A, in the game's own order.
///
/// Every page is kept, even one with nothing to say, so an index here is an index into what the
/// game shows. Empty lines at either end of a page are dropped — the newline that follows a page
/// break, mostly — because the box starts each page on a fresh line regardless.
pub fn pages(raw: &[u8]) -> Result<Vec<Page>> {
    let units = units(raw)?;
    let mut out: Vec<Page> = Vec::new();
    let mut lines: Vec<Vec<i64>> = vec![Vec::new()];
    let mut spoken: Vec<String> = Vec::new();
    for (start, end, control) in walk(&units)? {
        match control {
            None => {
                for &unit in py::slice(&units, start, end) {
                    if unit == NEWLINE {
                        lines.push(Vec::new());
                    } else {
                        lines.last_mut().expect("a page has a line").push(unit);
                    }
                }
                spoken.push(decode(py::slice(&units, start, end)));
            }
            Some(control) if PAGE_BREAKS.contains(&control.code) => {
                out.push(page(&lines, &spoken));
                lines = vec![Vec::new()];
                spoken = Vec::new();
            }
            Some(_) => lines
                .last_mut()
                .expect("a page has a line")
                .extend_from_slice(py::slice(&units, start, end)),
        }
    }
    out.push(page(&lines, &spoken));
    Ok(out)
}

fn page(lines: &[Vec<i64>], spoken: &[String]) -> Page {
    let mut kept: Vec<Vec<i64>> = lines.to_vec();
    while !kept.is_empty() && kept[0].is_empty() {
        kept.remove(0);
    }
    while !kept.is_empty() && kept[kept.len() - 1].is_empty() {
        kept.pop();
    }
    Page {
        lines: kept,
        text: py::split(&spoken.concat()).join(" "),
    }
}

fn units(raw: &[u8]) -> Result<Vec<i64>> {
    if raw.len() % 2 == 1 {
        return Err(Error::value(format!(
            "UTF-16 text cannot be {} bytes long",
            raw.len()
        )));
    }
    Ok(raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair) as i64)
        .collect())
}

/// (start, end, control) for each run: control is None for a run of text. A zero outside a
/// control sequence ends the text, as it ends it for the game: a Chinese patch of ORAS leaves
/// other halfwords after it in a line, a different one after each of the six trainer names that
/// are otherwise the same two (pa3ds/rust-pokemon-access-3ds#34).
fn walk(units: &[i64]) -> Result<Vec<(i64, i64, Option<Control>)>> {
    let len = units.len() as i64;
    let mut runs = Vec::new();
    let mut start = 0;
    let mut at = 0;
    while at < len {
        if units[at as usize] == END {
            if at > start {
                runs.push((start, at, None));
            }
            return Ok(runs);
        }
        if units[at as usize] != MARKER {
            at += 1;
            continue;
        }
        if at > start {
            runs.push((start, at, None));
        }
        if at + 2 >= len {
            return Err(Error::value(format!(
                "control sequence at halfword {at} is cut off"
            )));
        }
        let count = units[(at + 1) as usize];
        let end = at + 2 + count;
        if count == 0 || end > len {
            return Err(Error::value(format!(
                "control sequence at halfword {at} claims {count} halfwords"
            )));
        }
        runs.push((
            at,
            end,
            Some(Control {
                code: units[(at + 2) as usize],
                args: py::slice(units, at + 3, end).to_vec(),
            }),
        ));
        start = end;
        at = end;
    }
    if start < len {
        runs.push((start, len, None));
    }
    Ok(runs)
}

fn decode(units: &[i64]) -> String {
    let bytes: Vec<u8> = units
        .iter()
        .flat_map(|&unit| (unit as u16).to_le_bytes())
        .collect();
    py::decode_utf16le(&bytes)
}
