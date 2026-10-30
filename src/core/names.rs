//! What to call the things the game itself never names.
//!
//! ORAS holds no name for the people on its maps. A speaker is an overworld sprite — Norman `0x13f`
//! wherever he stands, Wally `0xb4` — and nothing in the game turns one into a word (findings, spikes
//! 52, 73, 75, 76 and 77). Signs and doors are the same: the game knows a slot on a map, not "the
//! Mart's sign".
//!
//! So pa3ds carries names of its own, under two rules that keep the "no hardcoded game text"
//! constraint meaning what it was written to mean (AGENTS.md, reopened by the user 2026-09-21):
//!
//! - **A name pa3ds invents never replaces one the game says.** Anything the game shows is read from
//!   the game, as it always was. These names fill silence, nothing else.
//! - **The player's own names win, and the player can always add one.** A name given while playing is
//!   written to the player's own file and takes precedence over the one that shipped.
//!
//! Between the two sits **the game's own word**, when the game has one after all but RAM does not
//! hold it: the rival is a work object in RAM, and the ROM names them in the player's language
//! (`games.gen6.archives`, spike 87); a door's slot is a warp in RAM, and the ROM says which
//! building some of them lead to (spike 86). So a thing is called, in order, what the player has
//! called it, what the game calls it, and what pa3ds shipped — "Rival" is only heard without a ROM.
//!
//! Two things get named, and they are keyed differently because they are different. A **sprite** is
//! one character everywhere in the game, so naming Norman once names him in every scene; the shipped
//! table is a game's `sprites.ftl`, a language a folder under `<pa3ds>/data` (ADR-0040). A **place**
//! is a slot on one map — the zone list's
//! (map, slot) — so a sign named in Petalburg says nothing about a sign in Rustboro. The shipped
//! table names some slots too: furniture each zone shows a message of its own for, but the same
//! thing in many, as the Pokémon Journals and the beds are.
//!
//! The player's names live in `names/<language>/<rom>.toml` under their own folder, keyed as the
//! shipped table is: **TOML, per language, one file a ROM** (ADR-0031), since pa3ds writes it, where
//! the shipped table is Fluent, which a translator opens (ADR-0040). The language is the one pa3ds
//! speaks its own words in. A name is a word in a language, so English names for a ROM played in
//! French belong to neither.
//!
//! The file opens with a comment saying which game, which ROM and which language it is for, because
//! `sango-1` says nothing to somebody who did not choose it and sharing names is a matter of sending
//! the file to somebody who then has to know what they have been sent. The game's own name comes from
//! the emulator's window (`pa3ds.core.window`), which reads it from the ROM, so pa3ds holds no table of
//! game names at all.
//!
//! Nothing here ever raises at a player: a names file that is missing, unreadable or malformed reads
//! as no names at all. Being told the wrong name is a nuisance; a reader that stops mid-game is not.
//!
//! Bound by:
//! - ADR-0023: a name of pa3ds's own never replaces the game's, lives in a data file, and the
//!   player's always wins.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::IndexMap;
use regex::Regex;

use crate::core::toml;
use crate::core::words::{Words, current};
use crate::py::{self, Result};

pub const SPRITES: &str = "sprites";
pub const PLACES: &str = "places";
pub const SECTIONS: [&str; 2] = [SPRITES, PLACES];
/// The shipped table's words for each kind of use tile, by the kind (`words`).
pub const TILES: &str = "tiles";
/// The shipped table's words for furniture by the game's shared script it runs (`words`).
pub const FURNITURE: &str = "furniture";
pub const WORDS: [&str; 2] = [TILES, FURNITURE];
/// How each table's messages begin in a game's `sprites.ftl`: `sprite-0x118`, and a slot's
/// `place-0xaa-furniture-0`, its key with dashes where the player's file has colons.
pub const SHIPPED: [(&str, &str); 4] = [
    (SPRITES, "sprite-"),
    (PLACES, "place-"),
    (TILES, "tile-"),
    (FURNITURE, "furniture-"),
];

/// `SHIPPED[section]`.
fn shipped(section: &str) -> &'static str {
    SHIPPED
        .iter()
        .find(|(held, _)| *held == section)
        .map(|(_, prefix)| *prefix)
        .unwrap_or_default()
}

/// What the game itself calls a character, where RAM does not say but the ROM does.
pub type GameSprite = Rc<dyn Fn(i64) -> Option<String>>;
/// What the game itself calls a slot on a map: the building behind a door.
pub type GamePlace = Rc<dyn Fn(i64, i64) -> Option<String>>;

/// `f"{number:#x}"`: Python writes a negative number's sign before the `0x`.
fn hex(number: i64) -> String {
    if number < 0 {
        format!("-{:#x}", number.unsigned_abs())
    } else {
        format!("{number:#x}")
    }
}

/// How a character is written in a names file: `0x13f`.
pub fn sprite_key(character: i64) -> String {
    hex(character)
}

/// What each kind of slot is written as in a names file, by its tag (`things.SOURCE_KIND`): a
/// warp's slot 3 is `warp:3`. A word, so a person editing the file can tell a door from a sign
/// (Orion, 2026-10-11); a kind renamed in code keeps its word here, or its names stop matching.
pub const KINDS: [&str; 21] = [
    "furniture",
    "warp",
    "tile",
    "edge",
    "berry",
    "trigger",
    "base",
    "ash",
    "bike",
    "dive",
    "stairs",
    "app",
    "decoration",
    "model",
    "current",
    "ride",
    "climb",
    "hidden",
    "vine",
    "opening",
    "teleport",
];

/// A slot as names files wrote it until 2026-10-11, its kind's tag and its number one sum:
/// `0x10:4294967299`, Rustboro's warp 3.
static OLD_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(0x[0-9a-f]+):(\d+)$").expect("a fixed pattern"));

/// How a slot on a map is written in a names file: `0x10:warp:3`, map, kind and number. A slot
/// is never negative, so Python's `KINDS[kind]` never counts from the end here.
pub fn place_key(map_id: i64, slot: i64) -> String {
    let (kind, number) = (slot >> 32, slot & 0xFFFFFFFF);
    let word = match usize::try_from(kind).ok().and_then(|at| KINDS.get(at)) {
        Some(word) => word.to_string(),
        None => kind.to_string(),
    };
    format!("{}:{word}:{number}", hex(map_id))
}

/// A key in the form names files use now, from either form.
fn readable(key: &str) -> Result<String> {
    let Some(old) = OLD_KEY.captures(key) else {
        return Ok(key.to_string());
    };
    Ok(place_key(
        py::int_base16(&old[1])?,
        py::int_base10(&old[2])?,
    ))
}

/// `first or second()`: a name that is there but empty is no name at all.
fn either(first: Option<String>, second: impl FnOnce() -> Option<String>) -> Option<String> {
    match first {
        Some(first) if !first.is_empty() => Some(first),
        _ => second(),
    }
}

/// The names pa3ds has for what the game leaves unnamed, the player's ahead of the shipped.
pub struct Names {
    shipped: IndexMap<String, String>,
    words: IndexMap<String, IndexMap<String, String>>,
    /// The shipped table's slots: furniture the game shows alike in many zones, each its own
    /// zone's message, as the 16 Pokémon Journals are (spike 211).
    shipped_places: IndexMap<String, String>,
    path: Option<PathBuf>,
    /// What to write at the top of the player's file: the game and the language, which the
    /// file cannot work out for itself and a person reading it cannot either.
    header: IndexMap<String, String>,
    game_sprite: RefCell<GameSprite>,
    game_place: RefCell<GamePlace>,
    mine: RefCell<IndexMap<String, IndexMap<String, String>>>,
}

impl Default for Names {
    fn default() -> Self {
        Names::empty()
    }
}

impl Names {
    pub fn new(
        shipped: Option<&IndexMap<String, String>>,
        mine_path: Option<&Path>,
        header: Option<&IndexMap<String, String>>,
        words: Option<&IndexMap<String, IndexMap<String, String>>>,
        places: Option<&IndexMap<String, String>>,
    ) -> Result<Self> {
        let mut mine: IndexMap<String, IndexMap<String, String>> = SECTIONS
            .iter()
            .map(|section| (section.to_string(), IndexMap::new()))
            .collect();
        let mut migrated = false;
        if let Some(mine_path) = mine_path {
            let found = toml::read(mine_path);
            mine = SECTIONS
                .iter()
                .map(|section| (section.to_string(), toml::strings(found.get(*section))))
                .collect();
            let mut places: IndexMap<String, String> = IndexMap::new();
            for (key, said) in &mine[PLACES] {
                places.insert(readable(key)?, said.clone());
            }
            // An IndexMap's `==` ignores order, as a dict's does.
            if places != mine[PLACES] {
                // Written the old way: kept the new way from now on, read once a game is played.
                mine[PLACES] = places;
                migrated = true;
            }
        }
        let names = Names {
            shipped: shipped.cloned().unwrap_or_default(),
            words: words.cloned().unwrap_or_default(),
            shipped_places: places.cloned().unwrap_or_default(),
            path: mine_path.map(Path::to_path_buf),
            header: header.cloned().unwrap_or_default(),
            game_sprite: RefCell::new(Rc::new(|_character| None)),
            game_place: RefCell::new(Rc::new(|_map, _slot| None)),
            mine: RefCell::new(mine),
        };
        if migrated {
            names.write()?;
        }
        Ok(names)
    }

    /// `Names()`: no shipped names, no file, no header.
    pub fn empty() -> Self {
        Names::new(None, None, None, None, None).expect("nothing is read without a file")
    }

    /// The names for one game, one language and one player.
    ///
    /// The game's `sprites.ftl`, in the language of `words` (the words in place) and English
    /// where it has none: its characters' names, `sprite-0x118`, its words for use tiles,
    /// `tile-0xdd`, and furniture, `furniture-0x28f0`, and its names for slots,
    /// `place-0xaa-furniture-0` (ADR-0040). And the player's
    /// `names/<language>/<fingerprint>.toml` under `home`. `title` is what the emulator says it
    /// is running, for the comment at the top of that file; without it the ROM names itself.
    pub fn load(
        fingerprint: &str,
        home: Option<&Path>,
        title: &str,
        words: Option<&Words>,
    ) -> Result<Names> {
        let placed = current();
        let words = words.unwrap_or(&placed);
        let language = words.language.as_str();
        let mine = home.map(|home| {
            home.join("names")
                .join(language)
                .join(format!("{fingerprint}.toml"))
        });
        let header = IndexMap::from([
            (
                "game".to_string(),
                if title.is_empty() { fingerprint } else { title }.to_string(),
            ),
            ("rom".to_string(), fingerprint.to_string()),
            ("language".to_string(), language.to_string()),
        ]);
        let places: IndexMap<String, String> = words
            .prefixed(shipped(PLACES))
            .into_iter()
            .map(|(key, said)| (key.replace('-', ":"), said))
            .collect();
        let tables: IndexMap<String, IndexMap<String, String>> = WORDS
            .iter()
            .map(|section| (section.to_string(), words.prefixed(shipped(section))))
            .collect();
        Names::new(
            Some(&words.prefixed(shipped(SPRITES))),
            mine.as_deref(),
            Some(&header),
            Some(&tables),
            Some(&places),
        )
    }

    /// What the shipped table says each of a kind of thing is, by its number: in `TILES` a
    /// kind of use tile, `0xdd` a trash can wherever it stands; in `FURNITURE` the shared script
    /// a record runs, every vending machine one of five (Orion, 2026-10-14; spike 210). So each is
    /// said as what it is before anyone has named it. A word for what a thing is rather than a
    /// name, so a name the player gives one slot still comes first, and the game's own word for
    /// a kind, the PC's, ahead of it (`FieldReader`). A key that is not a number is a typo, and
    /// left out.
    pub fn words(&self, section: &str) -> IndexMap<i64, String> {
        static HEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new("^0x[0-9a-fA-F]+$").expect("a valid pattern"));
        let mut out = IndexMap::new();
        if let Some(table) = self.words.get(section) {
            for (key, said) in table {
                if HEX.is_match(key) {
                    let number = py::int_base16(key).expect("a key of hexadecimal digits");
                    out.insert(number, said.clone());
                }
            }
        }
        out
    }

    /// Ask the game for its own word for a character and for a slot on a map, ahead of the
    /// shipped table and behind the player's own names. Set once the game's text has been
    /// read.
    pub fn use_game(&self, sprite: Option<GameSprite>, place: Option<GamePlace>) {
        if let Some(sprite) = sprite {
            *self.game_sprite.borrow_mut() = sprite;
        }
        if let Some(place) = place {
            *self.game_place.borrow_mut() = place;
        }
    }

    /// What to call this character, or None when nobody has named it: the player's name, the
    /// game's own word, or the shipped table's, in that order.
    pub fn sprite(&self, character: i64) -> Option<String> {
        let key = sprite_key(character);
        let mine = self.mine.borrow()[SPRITES].get(&key).cloned();
        either(mine, || {
            let game_sprite = self.game_sprite.borrow().clone();
            either(game_sprite(character), || self.shipped.get(&key).cloned())
        })
    }

    /// What to call this slot on this map, or None: the player's name, the game's, then the
    /// shipped table's.
    pub fn place(&self, map_id: i64, slot: i64) -> Option<String> {
        let key = place_key(map_id, slot);
        let mine = self.mine.borrow()[PLACES].get(&key).cloned();
        either(
            either(mine, || {
                let game_place = self.game_place.borrow().clone();
                game_place(map_id, slot)
            }),
            || self.shipped_places.get(&key).cloned(),
        )
    }

    /// Give this character a name, and keep it. An empty name gives it back to the game's own
    /// word, the shipped table, or silence.
    pub fn name_sprite(&self, character: i64, said: &str) -> Result<()> {
        self.keep(SPRITES, &sprite_key(character), said)
    }

    /// Give this slot on this map a name, and keep it. An empty name gives it back to the
    /// game's own word, the shipped table, or silence.
    pub fn name_place(&self, map_id: i64, slot: i64, said: &str) -> Result<()> {
        self.keep(PLACES, &place_key(map_id, slot), said)
    }

    /// The player's own names, as they would be written.
    pub fn mine(&self) -> IndexMap<String, IndexMap<String, String>> {
        self.mine.borrow().clone()
    }

    fn keep(&self, section: &str, key: &str, said: &str) -> Result<()> {
        let said = crate::py::strip(said);
        {
            let mut mine = self.mine.borrow_mut();
            if !said.is_empty() {
                mine[section].insert(key.to_string(), said.to_string());
            } else {
                mine[section].shift_remove(key);
            }
        }
        self.write()
    }

    /// The player's names as the file they are written to, comment and all.
    pub fn render(&self) -> String {
        let header = |key: &str| self.header.get(key).map(String::as_str).unwrap_or("");
        let rom = header("rom");
        let language = header("language");
        let game = header("game");
        let mut lines = toml::comment([
            format!(
                "Names you have given in {}.",
                if game.is_empty() { "this game" } else { game }
            ),
            [
                if rom.is_empty() {
                    String::new()
                } else {
                    format!("ROM {rom}.")
                },
                if language.is_empty() {
                    String::new()
                } else {
                    format!("Language {language}.")
                },
            ]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
            String::new(),
            "pa3ds writes this file when you rename something while playing. You can edit it by"
                .to_string(),
            "hand too, and sending it to somebody else gives them your names.".to_string(),
            String::new(),
            "[sprites] is a character: everyone drawn alike everywhere in the game, by the number"
                .to_string(),
            "the game counts them with. [places] is a slot on one map, written map:kind:number,"
                .to_string(),
            "so a sign named here says nothing about a sign on another map; one person or"
                .to_string(),
            "boulder named alone is a slot of its map too, of the kind model.".to_string(),
        ]);
        for section in SECTIONS {
            lines.push(String::new());
            lines.push(format!("[{section}]"));
            lines.extend(toml::pairs(&self.mine.borrow()[section]));
        }
        lines.join("\n") + "\n"
    }

    fn write(&self) -> Result<()> {
        if let Some(path) = &self.path {
            toml::write(path, &self.render())?;
        }
        Ok(())
    }
}
