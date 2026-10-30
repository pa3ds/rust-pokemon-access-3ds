//! Words for what the game draws without any: buttons, icons and signs that are pictures alone.
//!
//! What a picture is called is pa3ds's own, in the player's language: `texture-fight_button` in a
//! `pictures.ftl` under `<pa3ds>/data`, the game's own file before every game's (ADR-0040). Sun's
//! buttons of pictures are named by their pane, `button-Btn_Fight`, since Sun's layout files are not
//! read (spike 124). A picture no file names, in the player's language or in English, is said in
//! words made from its texture's name, `fight_button.bclim` "fight", or its pane's, as every picture
//! was until 2026-10-17; the log names each one so said, which is how the English table grows. A
//! sign the game never names, a Box mark, is said as the Unicode sign it draws, which the screen
//! reader names in the player's own language.
//!
//! What pa3ds says of a picture rather than what it is, "empty", "on", "third place", "2 stars", is a
//! message too, by the id here (`EMPTY`, `ON`, `PLACES`): a reader says `own(EMPTY)`.
//!
//! Bound by:
//! - ADR-0036: a picture's words are a message of `pictures.ftl`, made from its texture's name only
//!   where none has it.
//! - ADR-0037: the DexNav's pictures in words: stars counted, the red mark after what it marks,
//!   crowns by colour; and a result's stars in Pokémon-Amie's games.

use std::sync::OnceLock;

use indexmap::IndexMap;

use crate::core::words::{current, own};
use crate::py;

/// The word a texture name ends in to say its picture is a button, or `btn` short for it, as the
/// PlayNav's `return_btn` and the Pokédex's `up_btn` (movie24). Left unsaid.
pub const BUTTON: &str = "button";
pub const ICON: &str = "icon";
/// Words a texture name starts with to say what its picture is or belongs to, left unsaid: an
/// icon, a move's ("waza", Japanese for move) or a contest's. The name entry's confirm key is
/// `icon_return`, and reads "return" (opening.ctm, the user, 2026-09-18); the move-forget screen's
/// `waza_con_change` reads "change" (Orion, 2026-10-01), not "waza con change".
pub const KINDS: [&str; 3] = [ICON, "waza", "con"];
/// Words a texture name ends in to say which size of a picture it is, after its kind: the
/// contest photo's `camera_btn_mini` is "camera" (Orion, 2026-10-09, movie34). Left unsaid.
pub const SIZES: [&str; 1] = ["mini"];
/// The word a designer starts a button's pane name with, as the battle's `Btn_Fight` (spike 124):
/// what the button is, not what it says.
pub const BUTTON_PREFIX: &str = "btn";

/// A picture's message, by its texture's name less `.bclim`: `texture-map_big`, "zoom in".
pub const TEXTURE: &str = "texture-";
/// A Sun button's message, by its pane's name, `button-Hero_00`, or that name less the number of
/// one of a row, `button-Btn_Pokemon`.
pub const PANE: &str = "button-";
/// A picture with no layout, by what the reader that finds it calls it: X's mirrors and looks,
/// `model-boy` (`games.xy.intro`).
pub const MODEL: &str = "model-";
/// Textures named by how their name starts: Super Training's cleared regimens draw
/// `stage_icon00_rock` and every `stage_icon` after it, each `texture-stage_icon` (Khai's session of
/// 2026-10-09); the battle's move info screen draws its move buttons `waza_pos_00` and on
/// (or_move-learning, 2026-10-18).
pub const PREFIXED: [&str; 2] = ["stage_icon", "waza_pos"];
pub const PREVIOUS: &str = "top-bar-previous";
/// What the ◀ and ▶ in the top bar's corners do: the PlayNav's pages and the sub-screen
/// tutorial's. Both draw `button.bclim`, a ◀, and the right one is mirrored by its material, not
/// by any pane float (spike 198, omega_ruby_bottom-screen1), so the picture's words are `BUTTON` for
/// both and only its side of the screen tells them apart (`common.touch`). The ROM's other
/// `button.bclim` are plates, a battle's 250 by 40 and a rounded corner in Super Training's
/// archive, and a `button` anywhere but that bar stays one.
pub const NEXT: &str = "top-bar-next";

/// Words a texture name spells otherwise, for a picture no file names. A file name is ASCII, so
/// `pokemon_button.bclim` loses the accent the game itself writes everywhere else ("Choose a
/// Pokémon."), and a screen reader says the two differently (the user, 2026-09-16);
/// `intro_famale.bclim`, which names the girl on the opening's choice of boy or girl, is Game Freak's
/// own misspelling, and a screen reader reads it aloud as written (the user, 2026-09-21); Super
/// Training's `gameover_c.bclim` runs two words together and is said "game over".
pub fn spellings() -> &'static IndexMap<&'static str, &'static str> {
    static SPELLINGS: OnceLock<IndexMap<&'static str, &'static str>> = OnceLock::new();
    SPELLINGS.get_or_init(|| {
        IndexMap::from([
            ("pokemon", "pokémon"),
            ("famale", "female"),
            ("gameover", "game over"),
        ])
    })
}

/// Said of a button that is only the plate other buttons' words are written on, and of a slot of
/// the PC Box holding no Pokémon.
pub const EMPTY: &str = "picture-empty";
/// Said of a battle's button drawn with no words, by its place among those drawn like it: the
/// move info's four moves.
pub const MOVE: &str = "picture-move";

/// The sign each mark's picture shows, which a screen reader names in the player's own words, as
/// it does the gender signs. The game draws the marks and never names them.
pub fn marks() -> &'static IndexMap<&'static str, &'static str> {
    static MARKS: OnceLock<IndexMap<&'static str, &'static str>> = OnceLock::new();
    MARKS.get_or_init(|| {
        IndexMap::from([
            ("box_mark_01", "●"),
            ("box_mark_02", "▲"),
            ("box_mark_03", "■"),
            ("box_mark_04", "♥"),
            ("box_mark_05", "★"),
            ("box_mark_06", "◆"),
        ])
    })
}
pub const ON: &str = "mark-on";
/// A mark set or not: the game says so only by drawing it solid.
pub const OFF: &str = "mark-off";

pub const STAR: &str = "★";
pub const NO_STAR: &str = "☆";
pub const HALF_STAR: &str = "⯨";
pub const SPECIAL: &str = "!";
pub const HEART: &str = "♥";
/// A heart hidden under the words of what it counts: one point of it not had (`screen`).
pub const NO_HEART: &str = "♡";
/// The colours of the stars Pokémon-Amie's games rate a result with, a difficulty's each: Easy's
/// yellow, Hard's pink (or_amie_stars).
pub const RATINGS: [&str; 4] = ["easy", "normal", "hard", "infinite"];
/// What a picture shows, where the DexNav draws it in place of words: a star of Potential filled
/// or not, three in a row, and the red mark beside what is better than usual, its level, its first
/// move, its ability (spike 195); where a contest does, a heart of a move's appeal, one a point
/// (spike 196); and where Pokémon-Amie's games rate a result out of five, a star or the left half
/// of one, an empty one drawn as a full one's faint shadow (or_amie_stars). Signs for the screen
/// text reader to say in words of its own (`stars`, `hearts`, the `SPECIAL` marks), never said as
/// signs.
///
/// Python builds the `star_<rating>` and `star_<rating>h` entries from `RATINGS`; they are
/// written out here, in the same order.
pub const DRAWN: [(&str, &str); 14] = [
    ("star01", STAR),
    ("star02", NO_STAR),
    ("sp_mark", SPECIAL),
    ("heart", HEART),
    ("heart00", HEART),
    ("heart01", HEART),
    ("star_easy", STAR),
    ("star_normal", STAR),
    ("star_hard", STAR),
    ("star_infinite", STAR),
    ("star_easyh", HALF_STAR),
    ("star_normalh", HALF_STAR),
    ("star_hardh", HALF_STAR),
    ("star_infiniteh", HALF_STAR),
];
/// A contest's places, which its results draw as pictures beside each entrant, `one` to `four`
/// (spike 196), said after the entrant: "Raiden & Metagross third place" (Orion, 2026-10-09).
pub const PLACES: [(&str, &str); 4] = [
    ("one", "contest-place-one"),
    ("two", "contest-place-two"),
    ("three", "contest-place-three"),
    ("four", "contest-place-four"),
];
pub const HIGHER_LEVEL: &str = "dexnav-higher-level";
pub const EGG_MOVE: &str = "dexnav-egg-move";
/// What the DexNav's mark says beside a level above the area's, a first move that is an egg move,
/// and the hidden ability, as the guides read it (docs/plans/dexnav.md); the game says none of it.
pub const HIDDEN_ABILITY: &str = "dexnav-hidden-ability";
/// Said after a Pokémon the DexNav shows in colour, caught, where the rest are its silhouettes:
/// only the caught are said so (Orion, 2026-10-06).
pub const CAUGHT: &str = "dexnav-caught";

/// `DRAWN.get(texture)`.
pub fn drawn(texture: &str) -> Option<&'static str> {
    DRAWN
        .iter()
        .find(|(held, _)| *held == texture)
        .map(|(_, sign)| *sign)
}

/// `PLACES.get(texture)`.
pub fn place(texture: &str) -> Option<&'static str> {
    PLACES
        .iter()
        .find(|(held, _)| *held == texture)
        .map(|(_, words)| *words)
}

/// Stars as words: "2 stars", "1 star", "0 stars"; with `half` after them, "2 and a half
/// stars", "half a star".
pub fn stars(filled: i64, half: bool) -> String {
    let id = if half {
        "stars-and-a-half"
    } else {
        "dexnav-stars"
    };
    own(id, &[("count", &filled)])
}

/// A contest move's hearts as words: "4 hearts", "1 heart".
pub fn hearts(count: i64) -> String {
    own("contest-hearts", &[("count", &count)])
}

/// What the data calls a texture's picture, `map_big.bclim` "zoom in", or None where no file
/// of the player's language or English's names it.
pub fn named(texture: &str) -> Option<String> {
    let stem = stem(texture);
    let words = current();
    let mut said = words.get(&format!("{TEXTURE}{stem}"), &[]);
    if said.is_none()
        && let Some(prefix) = PREFIXED.iter().find(|prefix| stem.starts_with(**prefix))
    {
        said = words.get(&format!("{TEXTURE}{prefix}"), &[]);
    }
    said
}

/// A texture's picture in words: what the data calls it (`named`), or else words made from its
/// name, `fight_button.bclim` "fight" (`made`), which the log notes.
pub fn words(texture: &str) -> String {
    match named(texture) {
        Some(said) => said,
        None => {
            let said = made(stem(texture));
            current().made(&format!("texture {}", stem(texture)), &said);
            said
        }
    }
}

/// A texture's name as words: `fight_button` is "fight".
///
/// A trailing "button", or a leading or trailing "icon" (`KINDS`), is the designer saying what
/// the picture is, not part of the item's name. The battle's command menu reads "fight bag
/// pokémon run" as any menu would (the user, 2026-09-16), the name entry's confirm key
/// `icon_return.bclim` reads "return", and so does the AreaNav's `return_icon.bclim` (movie17).
/// A texture called only `button` or `icon` keeps the word: dropping it leaves nothing.
pub fn made(stem: &str) -> String {
    let replaced = stem.replace('_', " ");
    let mut said = py::split(&replaced);
    if said.len() > 1 && SIZES.contains(&said[said.len() - 1]) {
        said.pop();
    }
    if said.len() > 1 && [BUTTON, BUTTON_PREFIX, ICON].contains(&said[said.len() - 1]) {
        said.pop();
    }
    while said.len() > 1 && KINDS.contains(&said[0]) {
        said.remove(0);
    }
    said.iter()
        .map(|word| spellings().get(*word).copied().unwrap_or(word))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A button's pane in words: what the data calls it, by its whole name and then by its name
/// less the number of one of a row, or else words made from its name, `Btn_Fight` "fight",
/// `Btn_Pokemon_00` "pokémon", which the log notes. None for a name with no words left.
pub fn pane_words(name: &str) -> Option<String> {
    let row = name
        .trim_end_matches(|c: char| c.is_ascii_digit())
        .trim_end_matches('_');
    let mut keys = vec![name];
    if row != name {
        keys.push(row);
    }
    for key in keys {
        if let Some(said) = current().get(&format!("{PANE}{key}"), &[]) {
            return Some(said);
        }
    }
    let mut parts: Vec<String> = name
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| part.to_lowercase())
        .collect();
    if !parts.is_empty() && parts[0] == BUTTON_PREFIX {
        parts.remove(0);
    }
    while parts.last().is_some_and(|last| py::isdigit(last)) {
        parts.pop();
    }
    if parts.is_empty() {
        return None;
    }
    let said = made(&parts.join("_"));
    current().made(&format!("button {name}"), &said);
    Some(said)
}

/// The sign of a Box mark's texture, or None for any other texture.
pub fn mark(texture: &str) -> Option<&'static str> {
    marks().get(stem(texture)).copied()
}

/// A texture's name less its `.bclim`.
fn stem(texture: &str) -> &str {
    match texture.rsplit_once('.') {
        Some((stem, _)) => stem,
        None => texture,
    }
}
