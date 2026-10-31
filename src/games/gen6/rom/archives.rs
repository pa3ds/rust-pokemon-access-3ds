//! The game's own words, out of the player's ROM: place names, the rival, the PC and doors.
//!
//! The ROM keeps its text in GARC archives, one a language, and a file of lines each (findings,
//! spike 74). They name what RAM only numbers:
//!
//! - **Place names**, file 90: "Littleroot Town", indexed by the low ten bits of a zone row's +0x1c
//!   (`field.FieldReader.place_index`, spike 85). The index is in RAM; the words are here.
//! - **Trainer names**, file 22: lines 1 to 3 are the boy protagonist and 4 to 6 the girl, in all
//!   seven languages (spike 85). The rival stands in a work object, and the model says the sprite it
//!   was filled with: `0x13a SUPPORT_F_RS` for May, `0x13b SUPPORT_M_RS` for Brendan (spike 89). So
//!   the rival is named by the game's own word in the player's own language, whoever they are in
//!   that scene, and "Rival" is only what is said without a ROM.
//!
//! - **The PC**, file 92: nothing labels it where it stands, but its own lines all name it, and
//!   the one word they share is the game's word for it (`PC_LINES`).
//!
//! - **Door names**, from three things the ROM holds and nothing links: every zone's data file and
//!   its warps (`a/0/1/3`, LZ11), the zone table with each zone's area kind (the same archive's file
//!   536), and the AreaNav's list of what each town has (`a/0/6/6`, its words in text file 33). A
//!   door is named when the pairing can be *derived* (spike 86): a kind behind doors in two towns or
//!   more is the one facility every one of those towns lists — the Pokémon Center, the Poké Mart,
//!   the Contest Spectacular Hall — and a town left with one door and one facility pairs them. 38 of
//!   143 doors in towns and routes, the same in every language because the pairing is made on line
//!   numbers, never on words. A door the ROM cannot pair is left unnamed, not guessed at. The way
//!   into a landmark's building that no plate pairs is named from the facilities its town's list has
//!   left (`plates.entrance_names`, 2026-10-06). A warp into one of X's gates is the gate and the
//!   place beyond it (`plates.gate_names`, 2026-10-28).
//!
//! - **The map's places**, which the Pokédex's habitat draws as touch areas of the AreaNav's map
//!   (`a/0/6/7`): the AreaNav's list gives each row's zone the index of its touch area at +0x04,
//!   and the map's archive opens with its layout's pane names, which the index is into (spike 152).
//!
//! The archives and their text files are read by `common.garc`.
//!
//! What a file number holds is a fact about ORAS, the way a struct offset is; what the lines say is
//! the ROM's, which is the point. The two are checked against each other when loaded, so a ROM that
//! is not what these numbers expect is turned away rather than read as nonsense. X and Y keep the same
//! kinds of file under other numbers, with a Town Map where ORAS has the AreaNav; a `RomLayout` says
//! where one game keeps each, and each game's package has its own (`games.oras.ROM`, `games.xy.ROM`;
//! spikes 100 and 101).
//!
//! Bound by:
//! - ADR-0008: the game's own words come from the player's ROM, never a table of ours; without one
//!   pa3ds carries on and logs it.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;
use unicode_general_category::{GeneralCategory, get_general_category};

use crate::core::binary::{self, u16, u32};
use crate::core::rom::{Rom, lz11};
use crate::games::common::field::ground::{CUT, ROCK_SMASH, STRENGTH, SURF, WATERFALL};
use crate::games::common::garc::{garc_files, message_lines, plain, singular};
use crate::games::common::text::{Token, tokens};
use crate::games::gen6::field::rail::Teleport;
use crate::games::gen6::field::zonedata;
use crate::games::gen6::rom::doors::{door_facilities, town_rows};
use crate::games::gen6::rom::layout::{RomLayout, TownFacilities, zone_table};
use crate::games::gen6::rom::maps::{
    ChunkSource, MapChunks, StreetSource, Streets, rail_teleports,
};
use crate::games::gen6::rom::people::{
    pokemon_models, tagged_species, trainer_classes, trainer_sprites,
};
use crate::games::gen6::rom::plates;
use crate::py::{self, Error, Kind, Result};

/// The protagonists' lines in the trainer name list, each three times over.
pub const BOY: i64 = 1;
pub const GIRL: i64 = 4;

/// The moves whose names a way is said with (`Navigator`).
pub const MOVES: [i64; 5] = [CUT, SURF, STRENGTH, ROCK_SMASH, WATERFALL];

/// The PC's own text: booting it ("booted up the PC!") and the three PCs it offers ("Someone's
/// PC", "Lanette's PC", "Professor's PC"). Nothing labels the PC where it stands — it is a cell of
/// the Center's counter, drawn with the building, not a sprite — but each of these lines names it,
/// and the word they all share is what the game calls it: "PC" in six languages and "パソコン" in
/// Japanese, the same in both games (2026-09-24). Line 1 is left out because the Italian asks
/// which "sistema" instead.
pub const PC_LINES: [i64; 4] = [0, 2, 3, 6];

/// `struct.Struct("<HHhHH")`: a berry spot in the field DLL's table (spike 141): its cell's x and
/// z, the ground's height there, its zone, and a word every spot shares, 0x218 in ORAS.
pub struct BerrySpot;

pub const BERRY_SPOT: BerrySpot = BerrySpot;

impl BerrySpot {
    /// `BERRY_SPOT.size`.
    pub fn size(&self) -> i64 {
        10
    }

    /// `BERRY_SPOT.iter_unpack(blob)`: a buffer that is not a whole number of records is
    /// `struct.error`.
    #[track_caller]
    pub fn iter_unpack(&self, blob: &[u8]) -> Result<Vec<[i64; 5]>> {
        if blob.len() as i64 % self.size() != 0 {
            return Err(Error::unpack(format!(
                "iterative unpacking requires a buffer of a multiple of {} bytes",
                self.size()
            )));
        }
        let mut out = Vec::new();
        for at in (0..blob.len() as i64).step_by(self.size() as usize) {
            out.push([
                u16(blob, at)?,
                u16(blob, at + 2)?,
                binary::s16(blob, at + 4)?,
                u16(blob, at + 6)?,
                u16(blob, at + 8)?,
            ]);
        }
        Ok(out)
    }

    /// `BERRY_SPOT.pack(*values)`: a value out of its field's range is `struct.error`.
    #[track_caller]
    pub fn pack(&self, values: [i64; 5]) -> Result<Vec<u8>> {
        const FIELDS: [(char, i64, i64); 5] = [
            ('H', 0, 0xFFFF),
            ('H', 0, 0xFFFF),
            ('h', -0x8000, 0x7FFF),
            ('H', 0, 0xFFFF),
            ('H', 0, 0xFFFF),
        ];
        let mut out = Vec::new();
        for (value, (code, least, most)) in values.into_iter().zip(FIELDS) {
            if !(least..=most).contains(&value) {
                return Err(Error::unpack(format!(
                    "'{code}' format requires {least} <= number <= {most}"
                )));
            }
            out.extend_from_slice(&value.to_le_bytes()[..2]);
        }
        Ok(out)
    }
}

/// Fewer records of the shape than this are not the table. ORAS's has 90, and the next longest
/// run of the shape in its DLL is 26, the same table read out of step.
pub const BERRY_SPOTS_LEAST: i64 = 32;

/// Where a species' stats say how many forms it has, its first among them.
pub const FORM_COUNT: i64 = 0x20;

/// (species, form) -> what the game calls the form, for each form but a species' first: the
/// forms' text names each species' first form on the line of its national number, then every
/// other form of every species in national order, after a line between (spike 222: Omega Ruby's
/// 723 "Mega Venusaur" to 912 "Hoopa Unbound"). A form whose name is its first form's says
/// nothing more, as Unown's every form is "One form". None where the lines do not come out
/// even: a list laid out otherwise is not read.
pub fn form_names(names: &[String], counts: &[i64]) -> IndexMap<(i64, i64), String> {
    let mut line = counts.len() as i64;
    let mut out: IndexMap<(i64, i64), String> = IndexMap::new();
    if names.len() as i64 <= line {
        return out;
    }
    for (species, &count) in counts.iter().enumerate() {
        for form in 1..count {
            if line + form < names.len() as i64 {
                let named = &names[(line + form) as usize];
                if !named.is_empty() && *named != names[species] {
                    out.insert((species as i64, form), named.clone());
                }
            }
        }
        line += (count - 1).max(0);
    }
    if line == names.len() as i64 - 1 {
        out
    } else {
        IndexMap::new()
    }
}
/// More cells than any map matrix is wide or tall.
pub const MATRIX_CELLS: i64 = 0x400;

/// The longest run of characters every line holds, with the spaces and punctuation around it
/// taken off, or None when that leaves less than two characters: the one word a handful of
/// lines all say, whatever language they say it in and whether it spaces its words or not.
pub fn shared_word(lines: &[String]) -> Option<String> {
    if lines.is_empty() {
        return None;
    }
    let shortest: Vec<char> = lines
        .iter()
        .min_by_key(|line| line.chars().count())
        .expect("lines is not empty")
        .chars()
        .collect();
    for size in (2..=shortest.len()).rev() {
        for start in 0..=shortest.len() - size {
            let run: String = shortest[start..start + size].iter().collect();
            if lines.iter().all(|line| line.contains(&run)) {
                let word = trimmed(&run);
                if word.chars().count() >= 2 {
                    return Some(word);
                }
            }
        }
    }
    None
}

/// `run` without the spaces and punctuation at either end.
fn trimmed(run: &str) -> String {
    use GeneralCategory::*;
    fn edge(character: char) -> bool {
        matches!(
            get_general_category(character),
            ConnectorPunctuation
                | DashPunctuation
                | ClosePunctuation
                | FinalPunctuation
                | InitialPunctuation
                | OtherPunctuation
                | OpenPunctuation
                | SpaceSeparator
                | LineSeparator
                | ParagraphSeparator
                | Control
                | Format
                | Surrogate
                | PrivateUse
                | Unassigned
        )
    }

    let run: Vec<char> = run.chars().collect();
    let (mut start, mut end) = (0, run.len());
    while start < end && edge(run[start]) {
        start += 1;
    }
    while end > start && edge(run[end - 1]) {
        end -= 1;
    }
    run[start..end].iter().collect()
}

/// `items[index]`: a negative index counts from the end, and one out of range raises.
#[track_caller]
fn item<T>(items: &[T], index: i64) -> Result<&T> {
    let at = if index < 0 {
        index + items.len() as i64
    } else {
        index
    };
    if at < 0 || at >= items.len() as i64 {
        return Err(Error::index("list index out of range"));
    }
    Ok(&items[at as usize])
}

/// What the ROM names that RAM only numbers, in one language.
///
/// `streets`, `street_records` and `map_chunks` are left out of the comparison, as the
/// dataclass leaves them (`compare=False`).
#[derive(Debug, Clone)]
pub struct GameText {
    pub places: Vec<String>,
    pub trainers: Vec<String>,
    pub language: String,
    /// (outdoor zone, warp slot) -> what the game calls the building behind that door.
    pub doors: IndexMap<(i64, i64), String>,
    /// Sprite -> what the game calls whoever is drawn with it: a person, or a kind of trainer.
    pub sprites: IndexMap<i64, String>,
    /// What the game calls the PC, out of the PC's own lines (`PC_LINES`), or None.
    pub pc: Option<String>,
    /// Where Lumiose's streets are read from, in a game that has any (`RomLayout.chunks`).
    pub streets: Option<Rc<dyn StreetSource>>,
    /// (zone, warp slot) -> the street of Lumiose's a warp between two of its zones comes out on
    /// (`plates.street_names`).
    pub street_to: IndexMap<(i64, i64), String>,
    /// The zones walked about in as outdoors (`plates.outdoors`).
    pub outdoors: HashSet<i64>,
    /// Zone -> its furniture and warps, for every zone with records on streets: listed with
    /// the zone the player is in when both are on one map (`FieldReader`, spike 107).
    pub street_records: IndexMap<i64, (Vec<Vec<u8>>, Vec<Vec<u8>>)>,
    /// What the game calls each of `MOVES`, by its number; none where the ROM's move names are
    /// not read.
    pub moves: IndexMap<i64, String>,
    /// Where the cell tables of chunks the game has not loaded are read from
    /// (`RomLayout.map_chunks`).
    pub map_chunks: Option<Rc<dyn ChunkSource>>,
    /// (zone, cell) of every berry spot, in the game's own order (`berry_spots`).
    pub berries: Vec<(i64, (i64, i64))>,
    /// What the game calls each language it carries, in the language choice's order
    /// (`RomLayout.languages`); none where the ROM's are not read.
    pub languages: Vec<String>,
    /// What the game calls each type, by its number; none where they are not read.
    pub types: Vec<String>,
    /// What the game calls each move category, in `RomLayout.categories`' order.
    pub categories: Vec<String>,
    /// What the game calls each status condition, in `RomLayout.conditions`' order.
    pub conditions: Vec<String>,
    /// What the game calls each contest category, in `RomLayout.contest_categories`' order.
    pub contest_categories: Vec<String>,
    /// A contest's labels of the entrant under its cursor, the Pokémon's then the trainer's
    /// (`RomLayout.contest_entrant`).
    pub contest_entrant: Vec<String>,
    /// The game's word for a Pokémon's contest conditions (`RomLayout.contest_conditions`).
    pub contest_conditions: Option<String>,
    /// The contests' computer entrants, each trainer with their Pokémon
    /// (`RomLayout.contest_rivals`).
    pub contest_rivals: Vec<(String, String)>,
    /// An image a button is drawn with -> the game's word for the button
    /// (`RomLayout.touch_words`).
    pub touch_words: IndexMap<String, String>,
    /// A label's words -> what a mark in its group says, or "" (`RomLayout.labels`).
    pub labels: IndexMap<String, String>,
    /// Words a screen draws as no label (`RomLayout.plain`).
    pub plain: HashSet<String>,
    /// Images a button draws only while what it shows is caught (`RomLayout.caught`).
    pub caught: HashSet<String>,
    /// A touch area of the map, by its pane's name -> what the game calls the place, in the
    /// order the AreaNav lists them (`map_places`); none where the map is not read.
    pub map_places: IndexMap<String, String>,
    /// What the game calls each Secret Base decoration, by its number (`RomLayout.decorations`).
    pub decorations: Vec<String>,
    /// What the game calls each kind of Poké Puff, in order (`RomLayout.puffs`).
    pub puffs: Vec<String>,
    /// The words before and after the name in what the game calls another player's Secret Base,
    /// by its owner's name and then by its team's: ("", "'s Base"), ("Team ", "'s Base")
    /// (`RomLayout.base_names`).
    pub base_names: Vec<(String, String)>,
    /// What the game calls each of Pokémon-Amie's wallpapers and then each interior item, in the
    /// singular (`RomLayout.goods`).
    pub goods: Vec<String>,
    /// What the game calls the Mach Bike (`RomLayout.bike`), or None.
    pub bike: Option<String>,
    /// What the game calls the Acro Bike (`RomLayout.acro`), or None.
    pub acro: Option<String>,
    /// Each of `RomLayout.files` as the ROM holds it, for a game's own readers; one the ROM does
    /// not have is left out. Left out of the comparison, as the dataclass leaves it.
    pub files: IndexMap<String, Vec<u8>>,
    /// What the game calls each Pokémon, by its national number; none where they are not read.
    pub species: Vec<String>,
    /// (species, form) -> what the game calls the form (`form_names`); none where they are not
    /// read.
    pub forms: IndexMap<(i64, i64), String>,
    /// The ROM's zone table, which the game's in RAM is byte for byte (`FieldReader`); empty
    /// where it is not read. Left out of the comparison, as the dataclass leaves it.
    pub zone_table: Vec<u8>,
    /// What the game calls each item, by its number; none where they are not read.
    pub items: Vec<String>,
    /// A file of the "story" text or the "text" as its lines, control sequences in, each as the
    /// ROM has it: the lines a scene follows (`games.common.scenes`). Left out of the comparison,
    /// as the dataclass leaves it.
    pub lines: IndexMap<String, TextLines>,
    /// (outdoor zone, furniture index) of every plate that says only a door's name
    /// (`plates.door_signs`).
    pub door_signs: HashSet<(i64, i64)>,
    /// (zone, a furniture record's script) -> the game's shared script it runs
    /// (`plates.shared_furniture`).
    pub shared: IndexMap<(i64, i64), i64>,
    /// (zone, a furniture record's script) -> the native its case hands one constant to, and the
    /// constant (`plates.native_furniture`): a vine's swing or climb.
    pub natives: IndexMap<(i64, i64), (i64, i64)>,
    /// What the game calls itself, "Pokémon Omega Ruby", out of the ROM's SMDH (`Rom.title`).
    pub title: Option<String>,
    /// (zone, warp slot) -> the place indices of the gate a warp goes into and of the place
    /// beyond it (`plates.gate_names`).
    pub gates: IndexMap<(i64, i64), (i64, i64)>,
    /// (zone, a trigger's script) -> (x, how high in world units, z) of where the teleporter it
    /// is puts the player (`plates.teleports`): Laverre Gym's panels.
    pub teleports: IndexMap<(i64, i64), (i64, i64, i64)>,
    /// The teleports the game's Gym modules place on streets (`RomLayout.gym_modules`):
    /// Anistar City Gym's.
    pub rail_teleports: Vec<Teleport>,
}

/// `Callable[[int], Sequence[bytes]]`: a text archive's file as its lines (`lines_of`).
#[derive(Clone)]
pub struct TextLines(pub Rc<dyn Fn(i64) -> Result<Vec<Vec<u8>>>>);

impl std::fmt::Debug for TextLines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TextLines")
    }
}

impl PartialEq for GameText {
    fn eq(&self, other: &Self) -> bool {
        self.places == other.places
            && self.trainers == other.trainers
            && self.language == other.language
            && self.doors == other.doors
            && self.sprites == other.sprites
            && self.pc == other.pc
            && self.street_to == other.street_to
            && self.outdoors == other.outdoors
            && self.moves == other.moves
            && self.berries == other.berries
            && self.languages == other.languages
            && self.types == other.types
            && self.categories == other.categories
            && self.conditions == other.conditions
            && self.contest_categories == other.contest_categories
            && self.contest_entrant == other.contest_entrant
            && self.contest_conditions == other.contest_conditions
            && self.contest_rivals == other.contest_rivals
            && self.touch_words == other.touch_words
            && self.labels == other.labels
            && self.plain == other.plain
            && self.caught == other.caught
            && self.map_places == other.map_places
            && self.decorations == other.decorations
            && self.base_names == other.base_names
            && self.bike == other.bike
            && self.acro == other.acro
            && self.species == other.species
            && self.forms == other.forms
            && self.items == other.items
            && self.door_signs == other.door_signs
            && self.shared == other.shared
            && self.natives == other.natives
            && self.title == other.title
    }
}

impl Default for GameText {
    /// `GameText((), ())`: nothing named, in English.
    fn default() -> Self {
        GameText {
            places: Vec::new(),
            trainers: Vec::new(),
            language: "en".to_string(),
            doors: IndexMap::new(),
            sprites: IndexMap::new(),
            pc: None,
            streets: None,
            street_to: IndexMap::new(),
            outdoors: HashSet::new(),
            street_records: IndexMap::new(),
            moves: IndexMap::new(),
            map_chunks: None,
            berries: Vec::new(),
            languages: Vec::new(),
            types: Vec::new(),
            categories: Vec::new(),
            conditions: Vec::new(),
            contest_categories: Vec::new(),
            contest_entrant: Vec::new(),
            contest_conditions: None,
            contest_rivals: Vec::new(),
            touch_words: IndexMap::new(),
            labels: IndexMap::new(),
            plain: HashSet::new(),
            caught: HashSet::new(),
            map_places: IndexMap::new(),
            decorations: Vec::new(),
            puffs: Vec::new(),
            base_names: Vec::new(),
            goods: Vec::new(),
            bike: None,
            acro: None,
            files: IndexMap::new(),
            species: Vec::new(),
            forms: IndexMap::new(),
            zone_table: Vec::new(),
            items: Vec::new(),
            lines: IndexMap::new(),
            door_signs: HashSet::new(),
            shared: IndexMap::new(),
            natives: IndexMap::new(),
            title: None,
            gates: IndexMap::new(),
            teleports: IndexMap::new(),
            rail_teleports: Vec::new(),
        }
    }
}

impl GameText {
    /// `GameText(places, trainers)`: the rest left to its default, set with
    /// `GameText { pc, ..GameText::new(places, trainers) }`.
    pub fn new(places: Vec<String>, trainers: Vec<String>) -> Self {
        GameText {
            places,
            trainers,
            ..GameText::default()
        }
    }

    /// Whether this was read out of the player's ROM: `NO_TEXT` is none, which every lookup
    /// finds nothing in.
    pub fn loaded(&self) -> bool {
        !self.places.is_empty()
    }

    /// `load(rom, language, layout)`: no zone's teleporters.
    pub fn load(rom: &Rom, language: &str, layout: &RomLayout) -> Result<GameText> {
        GameText::load_with(rom, language, layout, &IndexMap::new())
    }

    /// Read one language's names out of the ROM, and where the teleporters of the zones
    /// `teleporting` lists put the player (`Game.teleporting`). A RomError when the language is
    /// not one the game carries or the files are not what the layout says they are.
    pub fn load_with(
        rom: &Rom,
        language: &str,
        layout: &RomLayout,
        teleporting: &IndexMap<i64, i64>,
    ) -> Result<GameText> {
        let Some(archive) = layout.text.get(language) else {
            return Err(Error::new(
                Kind::RomError,
                format!(
                    "no text in {}; {} carries {}",
                    py::repr(language),
                    layout.name,
                    layout.text.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            ));
        };
        let files = garc_files(&rom.read(archive)?)?;
        if files.len() as i64 <= layout.places.max(layout.trainer_names) {
            return Err(Error::new(
                Kind::RomError,
                format!(
                    "{archive} holds {} files, too few to be {}'s text",
                    files.len(),
                    layout.name
                ),
            ));
        }
        let places: Vec<String> = message_lines(item(&files, layout.places)?)?
            .iter()
            .map(|line| plain(line))
            .collect();
        let trainers: Vec<String> = message_lines(item(&files, layout.trainer_names)?)?
            .iter()
            .map(|line| plain(line))
            .collect();
        let classes: Vec<String> = message_lines(item(&files, layout.class_names)?)?
            .iter()
            .map(|line| plain(line))
            .collect();
        let zones = garc_files(&rom.read(&layout.zones)?)?;
        let table = zone_table(&zones, layout)?;
        let read = read_zones(&zones, layout)?;
        let outside = plates::outdoors(&read, &table)?;
        let mut doors: IndexMap<(i64, i64), String> = IndexMap::new();
        // Place -> what its town's row calls each facility, which a plate may hold.
        let mut listed: IndexMap<i64, Vec<String>> = IndexMap::new();
        // Every facility the list's text names, from the first line a town's row lists: the
        // lines before are the places' descriptions and the list's own words, "No" among them.
        let mut every: Vec<String> = Vec::new();
        if let Some(towns) = &layout.towns {
            let facilities: Vec<String> = message_lines(item(&files, towns.text)?)?
                .iter()
                .map(|line| plain(line))
                .collect();
            for (door, line) in door_facilities(rom, Some(&zones), layout, &outside)? {
                if line < facilities.len() as i64 && !facilities[line as usize].is_empty() {
                    doors.insert(door, facilities[line as usize].clone());
                }
            }
            let size = crate::games::gen6::field::zonetable::INFO_SIZE;
            let rows = town_rows(rom, towns, table.len() as i64 / size)?;
            for (zone, lines) in &rows {
                let (zone, lines) = (*zone, lines);
                let place = u16(
                    &table,
                    zone * size + crate::games::gen6::field::zonetable::INFO_NAME,
                )? & crate::games::gen6::field::zonetable::NAME_MASK;
                listed.entry(place).or_default().extend(
                    lines
                        .iter()
                        .filter(|&&line| line < facilities.len() as i64)
                        .map(|&line| facilities[line as usize].clone()),
                );
            }
            let first = rows
                .iter()
                .flat_map(|(_zone, lines)| lines.iter().copied())
                .min()
                .unwrap_or(facilities.len() as i64);
            every = py::slice_from(&facilities, first).to_vec();
        }
        // The plates and the streets name what is otherwise said by its kind, so a ROM without
        // their files, as a version laid out otherwise might be, is read for the rest.
        let mut story: Vec<Vec<u8>> = Vec::new();
        if let Some(story_archive) = layout.story.get(language) {
            story = match rom.read(story_archive).and_then(|blob| garc_files(&blob)) {
                Ok(story) => story,
                Err(error) if error.is(Kind::RomError) => Vec::new(),
                Err(error) => return Err(error),
            };
            let some = !story.is_empty();
            let lines = lines_of(story.clone());
            let named =
                plates::plate_names(&read, &table, &lines, &outside, Some(&listed), &every)?;
            for (door, words) in named {
                doors.entry(door).or_insert(words);
            }
            if some {
                let named = plates::entrance_names(
                    &read,
                    &table,
                    &lines,
                    &outside,
                    &listed,
                    &doors,
                    &layout.entrances,
                )?;
                for (door, words) in named {
                    doors.entry(door).or_insert(words);
                }
            }
        }
        let door_signs =
            plates::door_signs(&read, &table, &lines_of(story.clone()), &outside, &doors)?;
        let mut street_to: IndexMap<(i64, i64), String> = IndexMap::new();
        if let Some(street_marks) = layout
            .street_marks
            .as_ref()
            .filter(|marks| !marks.is_empty())
            && let Some(street_names) = layout.street_names
            && street_names < files.len() as i64
        {
            let loaded = (|| -> Result<(Vec<String>, Vec<Vec<u8>>)> {
                let names = message_lines(item(&files, street_names)?)?
                    .iter()
                    .map(|line| plain(line))
                    .collect();
                let marks = garc_files(&rom.read(street_marks)?)?;
                Ok((names, marks))
            })();
            let (names, marks) = match loaded {
                Ok(loaded) => loaded,
                Err(error) if error.is(Kind::RomError) => (Vec::new(), Vec::new()),
                Err(error) => return Err(error),
            };
            street_to = plates::street_names(&read, &table, &marks, &names)?;
        }
        let mut sprites: IndexMap<i64, String> = IndexMap::new();
        for (&sprite, &trainer) in &layout.characters {
            if trainer < trainers.len() as i64 && !trainers[trainer as usize].is_empty() {
                sprites.insert(sprite, trainers[trainer as usize].clone());
            }
        }
        let (class_of, pairs) = trainer_classes(rom, layout)?;
        for (sprite, words) in
            trainer_sprites(&zones, &class_of, &pairs, &classes, &trainers, layout)?
        {
            sprites.entry(sprite).or_insert(words);
        }
        let species = match layout.species {
            Some(file) => lines_of_file(&files, file)?,
            None => Vec::new(),
        };
        let mut forms: IndexMap<(i64, i64), String> = IndexMap::new();
        if let Some((names, stats)) = &layout.forms {
            // The names run on in blank lines after the last species: 812 lines, 722 named.
            let named = species
                .iter()
                .enumerate()
                .filter(|(_, name)| !name.is_empty())
                .map(|(number, _)| number as i64)
                .max()
                .unwrap_or(-1)
                + 1;
            let read = || -> Result<IndexMap<(i64, i64), String>> {
                let stats = garc_files(&rom.read(stats)?)?;
                let counts: Vec<i64> = py::slice(&stats, 0, named)
                    .iter()
                    .map(|stat| {
                        stat.get(FORM_COUNT as usize)
                            .map_or(1, |&count| count as i64)
                    })
                    .collect();
                Ok(form_names(&lines_of_file(&files, *names)?, &counts))
            };
            match read() {
                Ok(read) => forms = read,
                Err(error) if error.is(Kind::RomError) => {}
                Err(error) => return Err(error),
            }
        }
        if let Some((archive, first)) = &layout.pokemon_models {
            let drawn = match layout
                .battle_models
                .as_deref()
                .map(|models| tagged_species(rom, models))
                .transpose()
                .and_then(|tags| pokemon_models(rom, archive, *first, tags.as_ref()))
            {
                Ok(drawn) => drawn,
                Err(error) if error.is(Kind::RomError) => IndexMap::new(),
                Err(error) => return Err(error),
            };
            for (sprite, number) in drawn {
                if 0 < number
                    && number < species.len() as i64
                    && !species[number as usize].is_empty()
                {
                    sprites
                        .entry(sprite)
                        .or_insert_with(|| species[number as usize].clone());
                }
            }
        }
        let mut pc = None;
        if files.len() as i64 > layout.pc && !files[layout.pc as usize].is_empty() {
            let lines = message_lines(&files[layout.pc as usize])?;
            if lines.len() as i64 > PC_LINES[PC_LINES.len() - 1] {
                let said: Vec<String> = PC_LINES
                    .iter()
                    .map(|&line| plain(&lines[line as usize]))
                    .collect();
                pc = shared_word(&said);
            }
        }
        let mut moves: IndexMap<i64, String> = IndexMap::new();
        if let Some(file) = layout.moves
            && file < files.len() as i64
            && !files[file as usize].is_empty()
        {
            let names = message_lines(&files[file as usize])?;
            for r#move in MOVES {
                if r#move < names.len() as i64 {
                    let word = plain(&names[r#move as usize]);
                    if !word.is_empty() {
                        moves.insert(r#move, word);
                    }
                }
            }
        }
        let streets = layout
            .chunks
            .as_ref()
            .filter(|chunks| !chunks.is_empty())
            .map(|chunks| Rc::new(Streets::new(&rom.path, chunks)) as Rc<dyn StreetSource>);
        let mut on_streets: IndexMap<i64, (Vec<Vec<u8>>, Vec<Vec<u8>>)> = IndexMap::new();
        for (&zone, (_data, furniture, warps)) in &read {
            let mut any = false;
            for record in furniture.iter().chain(warps) {
                if zonedata::on_rail(record)? {
                    any = true;
                    break;
                }
            }
            if any {
                on_streets.insert(zone, (furniture.clone(), warps.clone()));
            }
        }
        let mut berries: Vec<(i64, (i64, i64))> = Vec::new();
        if let Some(file) = &layout.berries {
            berries = match rom.read(file) {
                Ok(dll) => berry_spots(&dll),
                Err(error) if error.is(Kind::RomError) => Vec::new(),
                Err(error) => return Err(error),
            };
        }
        let lines_from = |from: Option<(i64, i64)>| -> Result<Vec<String>> {
            match from {
                None => Ok(Vec::new()),
                Some((file, start)) => {
                    Ok(py::slice_from(&lines_of_file(&files, file)?, start).to_vec())
                }
            }
        };

        let types = match layout.types {
            Some(file) => lines_of_file(&files, file)?,
            None => Vec::new(),
        };
        let languages = lines_from(layout.languages)?;
        let categories = lines_from(layout.categories)?;
        let conditions = lines_from(layout.conditions)?;
        let natives = plates::native_furniture(&read)?;
        let gates = plates::gate_names(&read, &table)?;
        let text = GameText {
            places: places.clone(),
            trainers,
            language: language.to_string(),
            doors,
            sprites,
            pc,
            streets,
            street_to,
            outdoors: outside,
            street_records: on_streets,
            moves,
            map_chunks: layout
                .map_chunks
                .as_ref()
                .filter(|chunks| !chunks.is_empty())
                .map(|chunks| {
                    Rc::new(MapChunks::with_matrices(
                        &rom.path,
                        chunks,
                        layout.matrices.as_deref(),
                    )) as Rc<dyn ChunkSource>
                }),
            berries,
            languages,
            types,
            categories,
            conditions,
            map_places: map_places_of(rom, layout, &table, &places)?,
            touch_words: touch_words(&files, layout)?,
            labels: {
                let mut labels: IndexMap<String, String> = IndexMap::new();
                for (file, lines) in &layout.labels {
                    let said = lines_of_file(&files, *file)?;
                    for (&line, mark) in lines {
                        if line < said.len() as i64 && !said[line as usize].is_empty() {
                            labels.insert(said[line as usize].clone(), mark.clone());
                        }
                    }
                }
                labels
            },
            plain: {
                let mut plain: HashSet<String> = HashSet::new();
                for (file, lines) in &layout.plain {
                    let said = lines_of_file(&files, *file)?;
                    for &line in lines {
                        if line < said.len() as i64 && !said[line as usize].is_empty() {
                            plain.insert(said[line as usize].clone());
                        }
                    }
                }
                plain
            },
            caught: layout.caught.clone(),
            contest_categories: lines_from(layout.contest_categories)?,
            contest_entrant: py::slice(&lines_from(layout.contest_entrant)?, 0, 2).to_vec(),
            contest_rivals: match layout.contest_rivals {
                Some(file) => halves(&lines_of_file(&story, file)?),
                None => Vec::new(),
            },
            contest_conditions: match layout.contest_conditions {
                Some((file, line)) => py::slice_from(&lines_of_file(&story, file)?, line)
                    .first()
                    .filter(|word| !word.is_empty())
                    .cloned(),
                None => None,
            },
            decorations: match layout.decorations {
                Some(file) => lines_of_file(&files, file)?,
                None => Vec::new(),
            },
            puffs: match layout.puffs {
                Some(file) => lines_of_file(&files, file)?,
                None => Vec::new(),
            },
            base_names: match layout.base_names {
                Some(file) => lines_of(files.clone())(file)?
                    .iter()
                    .map(|line| around_name(line))
                    .collect(),
                None => Vec::new(),
            },
            goods: match layout.goods {
                Some(file) => lines_of_file_read(&files, file, singular)?,
                None => Vec::new(),
            },
            bike: lines_from(layout.bike)?
                .into_iter()
                .next()
                .filter(|line| !line.is_empty()),
            acro: lines_from(layout.acro)?
                .into_iter()
                .next()
                .filter(|line| !line.is_empty()),
            files: files_of(rom, layout)?,
            species,
            forms,
            zone_table: table.clone(),
            items: match layout.items {
                Some(file) => lines_of_file(&files, file)?,
                None => Vec::new(),
            },
            lines: IndexMap::from([
                (
                    "story".to_string(),
                    TextLines(Rc::new(lines_of(story.clone()))),
                ),
                (
                    "text".to_string(),
                    TextLines(Rc::new(lines_of(files.clone()))),
                ),
            ]),
            door_signs,
            shared: plates::shared_furniture(&natives),
            natives,
            gates,
            title: rom.title(language)?,
            teleports: plates::teleports(&read, teleporting)?,
            rail_teleports: rail_teleports_of(rom, layout)?,
        };
        if layout.protagonists && !text.protagonists_hold() {
            return Err(Error::new(
                Kind::RomError,
                format!(
                    "{archive} file {} is not the trainer names this version of pa3ds knows",
                    layout.trainer_names
                ),
            ));
        }
        if !layout.protagonists && !text.places_hold(&table)? {
            return Err(Error::new(
                Kind::RomError,
                format!(
                    "{archive} file {} is not the place names this version of pa3ds knows",
                    layout.places
                ),
            ));
        }
        Ok(text)
    }

    /// What the game calls a zone's place, by the ROM's zone table, or None.
    pub fn zone_place(&self, zone: i64) -> Option<String> {
        place_of(&self.zone_table, &self.places)(zone)
    }

    /// What the game calls a place, or None for the placeholder at 0 and an empty line.
    pub fn place(&self, index: i64) -> Option<String> {
        if !(0 < index && index < self.places.len() as i64) {
            return None;
        }
        Some(self.places[index as usize].clone()).filter(|place| !place.is_empty())
    }

    /// What the game calls the building behind a door in from this zone's outdoors, or None
    /// when the ROM does not say which building it is.
    pub fn door(&self, zone: i64, slot: i64) -> Option<String> {
        self.doors.get(&(zone, slot)).cloned()
    }

    /// The street of Lumiose's this zone's warp comes out on, or None.
    pub fn street(&self, zone: i64, slot: i64) -> Option<String> {
        self.street_to.get(&(zone, slot)).cloned()
    }

    /// What the game calls the gate this zone's warp goes into and the place beyond it, or
    /// None.
    pub fn gate(&self, zone: i64, slot: i64) -> Option<(String, String)> {
        let &(gate, beyond) = self.gates.get(&(zone, slot))?;
        Some((self.place(gate)?, self.place(beyond)?))
    }

    /// What the game calls a Pokémon in a form: the form's own name where it holds the
    /// species' ("Mega Latios"), the form's before the species' where it does not ("Primal
    /// Reversion Groudon", as most of the game's own put the form first; Orion, 2026-10-16),
    /// the species' alone for a first form or one the game gives no name of its own; None for
    /// no species.
    pub fn pokemon(&self, species: i64, form: i64) -> Option<String> {
        let name = if 0 < species && species < self.species.len() as i64 {
            self.species[species as usize].as_str()
        } else {
            ""
        };
        if name.is_empty() {
            return None;
        }
        let named = self.forms.get(&(species, form)).map_or("", String::as_str);
        if named.is_empty() {
            return Some(name.to_string());
        }
        Some(if named.contains(name) {
            named.to_string()
        } else {
            format!("{named} {name}")
        })
    }

    /// What the game calls whoever is drawn with this sprite, or None: a person the story
    /// names, or the kind of trainer everyone drawn as it is.
    pub fn character(&self, sprite: i64) -> Option<String> {
        self.sprites.get(&sprite).cloned()
    }

    /// Is the place list what it should be: long enough for every zone's place, with a name
    /// at the place of every outdoor zone. That holds of X's list in every language it carries
    /// (spike 100), and of no other file of its archive.
    fn places_hold(&self, table: &[u8]) -> Result<bool> {
        let size = crate::games::gen6::field::zonetable::INFO_SIZE;
        for zone in 0..table.len() as i64 / size {
            let (parent, index) = (
                u16(
                    table,
                    zone * size + crate::games::gen6::field::zonetable::INFO_PARENT,
                )?,
                u16(
                    table,
                    zone * size + crate::games::gen6::field::zonetable::INFO_PARENT + 2,
                )?,
            );
            let index = index & crate::games::gen6::field::zonetable::NAME_MASK;
            if index >= self.places.len() as i64
                || (parent == zone && self.places[index as usize].is_empty())
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Is the trainer list what it should be: the boy three times, then the girl three
    /// times, two different words. That is true in every language ORAS carries (spike 85) and
    /// of nothing else in the archive.
    fn protagonists_hold(&self) -> bool {
        let names = &self.trainers;
        if names.len() as i64 <= GIRL + 2 {
            return false;
        }
        let boy = py::slice(names, BOY, BOY + 3);
        let girl = py::slice(names, GIRL, GIRL + 3);
        let one = |words: &[String]| words.iter().collect::<HashSet<_>>().len() == 1;
        one(boy)
            && one(girl)
            && boy[0] != girl[0]
            && boy.iter().chain(girl).all(|name| !name.is_empty())
    }
}

/// The game's words where the player's ROM could not be read: none, so every lookup finds
/// nothing and the readers that need the ROM are off (`GameText.loaded`).
///
/// Python's `NO_TEXT`, a constant; a `GameText` holds maps, which no `const` can.
pub fn no_text() -> GameText {
    GameText::new(Vec::new(), Vec::new())
}

/// A touch area of the map, by its pane's name -> what the game calls its place.
///
/// The map's archive opens with a u32 count of its files and their names, 64 bytes each, then a
/// u32 count of its layout's panes and their names, 32 bytes each; a row of the list of what
/// each town has keeps its zone's touch area as an index into those (spike 152). Rows sharing a
/// touch area, as a cave's floors do, name it by the first.
pub fn map_places(
    head: &[u8],
    nav: &[u8],
    towns: &TownFacilities,
    pane_at: i64,
    place: &dyn Fn(i64) -> Option<String>,
) -> Result<IndexMap<String, String>> {
    let files = u32(head, 0)?;
    let at = 4 + 64 * files;
    let count = if at + 4 <= head.len() as i64 {
        u32(head, at)?
    } else {
        0
    };
    if !(0 < count && count <= 4096) || at + 4 + 32 * count > head.len() as i64 {
        return Err(Error::new(
            Kind::RomError,
            "the map's archive does not open with its panes' names",
        ));
    }
    let names: Vec<String> = (0..count)
        .map(|i| {
            let name = py::slice(head, at + 4 + 32 * i, at + 36 + 32 * i);
            let end = name
                .iter()
                .position(|&byte| byte == 0)
                .unwrap_or(name.len());
            py::decode_ascii(&name[..end])
        })
        .collect();
    let mut out: IndexMap<String, String> = IndexMap::new();
    let mut row = 0;
    while row < nav.len() as i64 - towns.row + 1 {
        let zone = u16(nav, row + towns.zone)?;
        let pane = u16(nav, row + pane_at)?;
        if pane < count
            && let Some(name) = place(zone).filter(|name| !name.is_empty())
        {
            out.entry(names[pane as usize].clone()).or_insert(name);
        }
        row += towns.row;
    }
    Ok(out)
}

/// What the game calls a zone's place, by the zone table's row for it, or None.
fn place_of<'a>(table: &'a [u8], places: &'a [String]) -> impl Fn(i64) -> Option<String> + 'a {
    let size = crate::games::gen6::field::zonetable::INFO_SIZE;

    move |zone: i64| -> Option<String> {
        if (zone + 1) * size > table.len() as i64 {
            return None;
        }
        let index = (u16(
            table,
            zone * size + crate::games::gen6::field::zonetable::INFO_NAME,
        )
        .ok()?)
            & crate::games::gen6::field::zonetable::NAME_MASK;
        if 0 < index && index < places.len() as i64 && !places[index as usize].is_empty() {
            Some(places[index as usize].clone())
        } else {
            None
        }
    }
}

/// `map_places` out of the ROM, or none where the game's map is not read.
fn map_places_of(
    rom: &Rom,
    layout: &RomLayout,
    table: &[u8],
    places: &[String],
) -> Result<IndexMap<String, String>> {
    let (Some((archive, file, pane_at)), Some(towns)) = (&layout.map_panes, &layout.towns) else {
        return Ok(IndexMap::new());
    };
    let place = place_of(table, places);

    let loaded = (|| -> Result<IndexMap<String, String>> {
        let map_files = garc_files(&rom.read(archive)?)?;
        let head = item(&map_files, *file)?.clone();
        let nav_files = garc_files(&rom.read(&towns.archive)?)?;
        let nav = item(&nav_files, towns.file)?.clone();
        let head = if py::slice(&head, 0, 1) == [0x11] {
            lz11(&head)?
        } else {
            head
        };
        map_places(&head, &nav, towns, *pane_at, &place)
    })();
    match loaded {
        Ok(places) => Ok(places),
        Err(error)
            if error.is(Kind::RomError)
                || error.is(Kind::IndexError)
                || error.is(Kind::StructError) =>
        {
            Ok(IndexMap::new())
        }
        Err(error) => Err(error),
    }
}

/// `RomLayout.files` out of the ROM, less any it cannot read. (Python's `_files`.)
fn files_of(rom: &Rom, layout: &RomLayout) -> Result<IndexMap<String, Vec<u8>>> {
    let mut out: IndexMap<String, Vec<u8>> = IndexMap::new();
    for name in &layout.files {
        match rom.read(name) {
            Ok(blob) => {
                out.insert(name.clone(), blob);
            }
            Err(error) if error.is(Kind::RomError) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(out)
}

/// The teleports of `RomLayout.gym_modules` (`rail_teleports`), less any module the ROM cannot
/// give. (Python's `_rail_teleports`.)
fn rail_teleports_of(rom: &Rom, layout: &RomLayout) -> Result<Vec<Teleport>> {
    let mut out: Vec<Teleport> = Vec::new();
    for module in &layout.gym_modules {
        match rom.read(module).and_then(|blob| rail_teleports(&blob)) {
            Ok(found) => out.extend(found),
            Err(error) if error.is(Kind::RomError) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(out)
}

/// Every zone's data file the ROM holds, decompressed, with its furniture and its warps.
fn read_zones(zones: &[Vec<u8>], layout: &RomLayout) -> Result<IndexMap<i64, plates::Zone>> {
    let mut out: IndexMap<i64, plates::Zone> = IndexMap::new();
    for (number, packed) in zones.iter().enumerate() {
        let number = number as i64;
        if number == layout.zone_table || packed.is_empty() {
            continue;
        }
        let data = match lz11(packed) {
            Ok(data) => data,
            Err(error) if error.is(Kind::RomError) => continue,
            Err(error) => return Err(error),
        };
        let Some(zone) = crate::games::gen6::field::zonedata::zone_file(&data)? else {
            continue;
        };
        let furniture = crate::games::gen6::field::zonedata::records(
            &data,
            &zone,
            crate::games::gen6::field::zonedata::FURNITURE,
        )?;
        let warps = crate::games::gen6::field::zonedata::records(
            &data,
            &zone,
            crate::games::gen6::field::zonedata::WARP,
        )?;
        out.insert(number, (data, furniture, warps));
    }
    Ok(out)
}

/// `RomLayout.touch_words` read: an image -> its line of the text file, where it has one.
fn touch_words(files: &[Vec<u8>], layout: &RomLayout) -> Result<IndexMap<String, String>> {
    let mut words: IndexMap<String, String> = IndexMap::new();
    for (file, lines) in &layout.touch_words {
        let said = lines_of_file(files, *file)?;
        for (image, &line) in lines {
            if line < said.len() as i64 && !said[line as usize].is_empty() {
                words.insert(image.clone(), said[line as usize].clone());
            }
        }
    }
    Ok(words)
}

/// A text file's lines, or none where the archive has no such file or it is not text: a ROM
/// laid out otherwise loses only what that file names.
fn lines_of_file(files: &[Vec<u8>], file: i64) -> Result<Vec<String>> {
    lines_of_file_read(files, file, plain)
}

/// `_lines_of_file` with its `read`: each line as `read` gives it.
fn lines_of_file_read(
    files: &[Vec<u8>],
    file: i64,
    read: fn(&[u8]) -> String,
) -> Result<Vec<String>> {
    if !(file < files.len() as i64) || files[file as usize].is_empty() {
        return Ok(Vec::new());
    }
    match message_lines(&files[file as usize]) {
        Ok(lines) => Ok(lines.iter().map(|line| read(line)).collect()),
        Err(error) if error.is(Kind::RomError) => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

/// A line's words before its first variable and after it, as `plain` reads words: the name
/// the game puts there is the player's or a team's, which only RAM has.
fn around_name(line: &[u8]) -> (String, String) {
    let parts = match tokens(line) {
        Ok(parts) => parts,
        Err(error) if error.is(Kind::ValueError) => return (String::new(), String::new()),
        Err(error) => panic!("{error:?}"),
    };
    let at = parts
        .iter()
        .position(|part| matches!(part, Token::Control(_)))
        .unwrap_or(parts.len());
    let words = |parts: &[Token]| -> String {
        parts
            .iter()
            .filter_map(|part| match part {
                Token::Text(text) => Some(text.as_str()),
                Token::Control(_) => None,
            })
            .collect()
    };
    (
        words(&parts[..at]),
        words(py::slice_from(&parts, at as i64 + 1)),
    )
}

/// A file's first half beside its second, line for line: a contest's trainers and their
/// Pokémon (Omega Ruby's story file 445, Gianna on line 4 and Tailster on 73, as Bulbapedia
/// lists them, 2026-10-09).
fn halves(lines: &[String]) -> Vec<(String, String)> {
    let half = lines.len() / 2;
    lines[..half]
        .iter()
        .cloned()
        .zip(lines[half..half * 2].iter().cloned())
        .collect()
}

/// A text archive's file as its lines, each split once when first asked for.
fn lines_of(files: Vec<Vec<u8>>) -> impl Fn(i64) -> Result<Vec<Vec<u8>>> {
    let split: RefCell<HashMap<i64, Vec<Vec<u8>>>> = RefCell::new(HashMap::new());

    move |file: i64| -> Result<Vec<Vec<u8>>> {
        if !split.borrow().contains_key(&file) {
            let lines = if file < files.len() as i64 {
                match message_lines(item(&files, file)?) {
                    Ok(lines) => lines,
                    Err(error) if error.is(Kind::RomError) => Vec::new(),
                    Err(error) => return Err(error),
                }
            } else {
                Vec::new()
            };
            split.borrow_mut().insert(file, lines);
        }
        Ok(split.borrow()[&file].clone())
    }
}

/// (zone, cell) of every berry spot the field's DLL lists, in its order, or none where it
/// lists none (spike 141).
///
/// No zone's data has them: the soil on Route 102 is a blocked cell like a tree's, with no
/// record and no move model at it, so the list never had one. The DLL keeps them all in one
/// table, found here by its shape, since a DLL's layout is the build's: the longest run of
/// `BERRY_SPOT` records whose zone and cells are in range and whose last word is one value
/// throughout, at any even offset.
pub fn berry_spots(dll: &[u8]) -> Vec<(i64, (i64, i64))> {
    let mut best: Vec<(i64, (i64, i64))> = Vec::new();
    let size = BERRY_SPOT.size();
    for start in (0..size).step_by(2) {
        let mut run: Vec<(i64, (i64, i64))> = Vec::new();
        let mut shared = 0;
        let end = start + py::floordiv(dll.len() as i64 - start, size) * size;
        for [x, z, _height, zone, word] in BERRY_SPOT
            .iter_unpack(py::slice(dll, start, end))
            .expect("the slice is whole records")
        {
            if word != 0
                && 0 < zone
                && zone < crate::games::gen6::field::zonetable::ZONE_LIMIT
                && x < MATRIX_CELLS
                && z < MATRIX_CELLS
            {
                if !run.is_empty() && word != shared {
                    if run.len() > best.len() {
                        best = run;
                    }
                    run = Vec::new();
                }
                shared = word;
                run.push((zone, (x, z)));
            } else if !run.is_empty() {
                if run.len() > best.len() {
                    best = run;
                }
                run = Vec::new();
            }
        }
        if run.len() > best.len() {
            best = run;
        }
    }
    if best.len() as i64 >= BERRY_SPOTS_LEAST {
        best
    } else {
        Vec::new()
    }
}
