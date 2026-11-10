//! The game's own words for what stands on Sun's field: the kind of trainer a person looks like,
//! each trainer's name, what a sign and a thing say, and the building behind a door (spike A).
//!
//! **People.** Every trainer record names its trainer (`zones.TRAINER_FLAGS`), whose file in
//! `a/1/0/5` gives their class at +0x00: text file 106 the classes, 105 the names. A model two
//! trainers or more stand as, all of one class, is that class wherever it is drawn, as gen 6's
//! sprites are (spike 90): "Youngster", or "Team Skull Grunt" where they share the one name too. The
//! trainer standing on their own record is their class and name, "Youngster Jimmy".
//!
//! **Signs and things** say what their zone's script shows first (`script`): a sign a line of story
//! file 21, its first line its name, "Trainer Tips", "Route 1"; anything else read by facing it a
//! line of its zone's own story file, its first sentence its name, "It's a Wii U!".
//!
//! **Doors** in from outdoors are named for what the **Town Map** (`a/2/4/1` file 0) lists, records
//! of 0x2c bytes:
//!
//! ```text
//! +0x00 u32 the island    +0x04 u32 the zone (0xffff none)    +0x08 u32 how it is marked: 0 a
//! Pokémon Center on the map, 0x0b a facility of the town, 3 a place to fly to
//! +0x0c u32 x, +0x10 u32 z, where the map marks it, in the world's units, or both 0
//! +0x24 u16 its line of system text file 100: "Pokémon Center", "Apparel Shop", "Salon"
//! ```
//!
//! A facility's line is one line of words; the file's lines of sentences are the map's
//! descriptions, which some records of a route hold, and they name no door. In turn:
//!
//! 1. A **marked** facility is the door into a building of its town nearest its mark, where that
//!    door is within `AT` cells (a Pokémon Center's mark is its door's place to the unit, 12 of 16)
//!    or within `MARKED` cells with no other nearly as near (`APART`): the Hau'oli shops' marks are
//!    1.2 to 3.9 cells from their doors.
//! 2. A **kind** of building, its zone's music (`ROW_MUSIC`), behind doors in two towns or more, is
//!    the one facility every one of those towns lists that a kind in more towns has not taken, and
//!    the one its marked doors were, where any were: X's rule (spike 86). 14 is every Pokémon
//!    Center, 25 the Apparel Shops, 24 the Salons and 22 the Malasada Shops.
//! 3. A **plate**, a building's sign (`PLATE`), names the door nearest it within `NEAR` cells with
//!    no other nearly as near, by its first line, unless that line is a place's name: "Ferry
//!    Terminal", "Malie Library". A plate of one line says no more than its door's name, a **door
//!    sign** (`Navigator`'s `door_sign`).
//!
//! The words are the ROM's, in the player's language, and the pairing never reads a word, so it
//! pairs the same doors in every language.
//!
//! Bound by:
//! - ADR-0008: the game's words come out of the player's own ROM, in their language.
//! - ADR-0023: the game's own word comes behind the player's names and ahead of the shipped.

use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;

use crate::core::binary::{u16, u32};
use crate::core::rom::Rom;
use crate::core::words::own;
use crate::games::common::field::things::{
    FURNITURE_SOURCE, MODEL_SOURCE, WARP_SOURCE, source_kind, source_number,
};
use crate::games::common::garc::{garc_files, garc_spans, message_lines, plain};
use crate::games::common::text::{Token, tokens};
use crate::games::sm::field::{CELL, place_of};
use crate::games::sm::script;
use crate::games::sm::text::TEXT_ARCHIVES;
use crate::games::sm::zones::{self as sm_zones, Record, SunZones};
use crate::py::{self, Error, Kind, Result};

/// The story text, a language an archive beside its system text's: `a/0/4/2` English.
///
/// `STORY_ARCHIVES`: a function, since a constant cannot hold the archives' new names.
pub fn story_archives() -> IndexMap<&'static str, String> {
    TEXT_ARCHIVES
        .iter()
        .map(|&(language, archive)| (language, archive.replace("a/0/3/", "a/0/4/")))
        .collect()
}
pub const FACILITIES: i64 = 100;
pub const TRAINER_NAMES: i64 = 105;
pub const TRAINER_CLASSES: i64 = 106;
/// Story text file 21, the signs': "Route 1", "Trainer Tips", "Pokémon Research Lab".
pub const SIGN_TEXT: i64 = 21;
pub const TRAINERS: &str = "a/1/0/5";
pub const TRAINER_CLASS: i64 = 0x00;
pub const TOWN_MAP: &str = "a/2/4/1";
pub const TOWN_MAP_RECORD: i64 = 0x2C;
pub const TOWN_MAP_LINE: i64 = 0x24;
pub const MARKED_CENTER: i64 = 0x00;
pub const MARKED_FACILITY: i64 = 0x0B;
pub const NO_ZONE: i64 = 0xFFFF;
/// A zone-table row's +0x00 u16, the music the zone plays: shops of one kind share theirs, and a
/// town's plain houses play the town's.
pub const ROW_MUSIC: i64 = 0x00;
/// A zone-table row's +0x0c u16, the zone's own file of story text.
pub const ROW_STORY: i64 = 0x0C;
/// An area's file of zone scripts, 'ZS', counted from its events (`zones.AREA_FILES`).
pub const SCRIPTS: i64 = 7;
/// A sign's kind (`zones.THING_KIND`) for a building's or a place's sign, beside 0x0e a route's
/// and 0x15 a board of tips.
pub const PLATE: i64 = 0x12;
/// Cells a Pokémon Center's mark, a facility's mark and a plate may be from the door they name.
/// The plates measured stand 5.4 to 15.8 cells from their doors, the next door 2.4 to 13 times as
/// far.
pub const AT: f64 = 1.0;
/// Cells a Pokémon Center's mark, a facility's mark and a plate may be from the door they name.
/// The plates measured stand 5.4 to 15.8 cells from their doors, the next door 2.4 to 13 times as
/// far.
pub const MARKED: f64 = 4.0;
/// Cells a Pokémon Center's mark, a facility's mark and a plate may be from the door they name.
/// The plates measured stand 5.4 to 15.8 cells from their doors, the next door 2.4 to 13 times as
/// far.
pub const NEAR: f64 = 16.0;
/// How much further the next door must be than the nearest for a mark or a plate to be the
/// nearest's, as X's plates (`gen6.rom.plates`).
pub const APART: f64 = 1.5;
/// A sentence's stops: ".", "!", "?", "…", or the full-width stops Japanese and Chinese end one
/// with (`SENTENCE`).
const STOPS: [char; 7] = [
    '.', '!', '?', '\u{2026}', '\u{3002}', '\u{ff01}', '\u{ff1f}',
];
/// The controls that put a value the game fills in into a line: 0x100 a name, 0x101 a Pokémon's,
/// 0x102 a number, opening 1,200 of the story's lines in English and Japanese (spike A).
pub const VALUES: (i64, i64) = (0x100, 0x200);
/// What a name may open on besides a letter or a digit: a quotation, or a pause, "...Heh".
pub const OPENING: &str = "“「『«\"(¡¿….";
pub const FULL_STOPS: &str = ".。";
/// Characters a thing's first sentence may run to and be its name: a longer one is a passage,
/// not a name, and the thing stays an object.
pub const LONGEST: usize = 100;

/// (zone, source): a place, as the names key one.
pub type Slot = (i64, i64);

/// `SENTENCE.match(text)`: a sentence to its stop, `.+?[stops](?=\s|$)` with `re.DOTALL`, as the
/// characters it runs to. The `regex` crate has no look-ahead, so the pattern is walked by hand:
/// at least one character, then the first stop followed by whitespace or the end.
fn sentence_match(chars: &[char]) -> Option<usize> {
    (1..chars.len()).find_map(|i| {
        (STOPS.contains(&chars[i]) && chars.get(i + 1).is_none_or(|&next| py::is_space(next)))
            .then_some(i + 1)
    })
}

/// What the game's own words call Sun's people, signs, things and doors.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SunGameText {
    /// Model -> the kind of trainer drawn as it.
    pub sprites: IndexMap<i64, String>,
    /// (zone, record) -> the trainer standing on it, class and name.
    pub trainers: IndexMap<(i64, i64), String>,
    pub places: IndexMap<Slot, String>,
    /// A `frozenset` only tested.
    pub door_signs: HashSet<Slot>,
}

impl SunGameText {
    pub fn sprite(&self, model: i64) -> Option<String> {
        self.sprites.get(&model).cloned()
    }

    /// What the game calls a slot on a map: a trainer's own record, a sign, a thing, or the
    /// building behind a door.
    pub fn place(&self, zone: i64, slot: i64) -> Option<String> {
        if source_kind(slot) == MODEL_SOURCE {
            return self.trainers.get(&(zone, source_number(slot))).cloned();
        }
        self.places.get(&(zone, slot)).cloned()
    }

    /// Whether a slot is a plate that says only the name of its door.
    pub fn door_sign(&self, zone: i64, slot: i64) -> bool {
        self.door_signs.contains(&(zone, slot))
    }

    /// Every name, out of the ROM in `language`. A RomError where Sun has no text in it or a
    /// table is not what Sun has.
    pub fn load(rom: &Rom, language: &str, zones: &SunZones) -> Result<SunGameText> {
        let system = TEXT_ARCHIVES
            .iter()
            .find(|(known, _)| *known == language)
            .map(|&(_, archive)| archive);
        let story = story_archives().get(language).cloned();
        let (Some(system), Some(story)) = (system, story) else {
            return Err(Error::new(
                Kind::RomError,
                format!("Sun has no text in {}", py::repr(language)),
            ));
        };
        let texts = garc_files(&rom.read(system)?)?;
        let stories = garc_files(&rom.read(&story)?)?;
        let mut read: Vec<Vec<String>> = Vec::new();
        for number in [FACILITIES, TRAINER_NAMES, TRAINER_CLASSES] {
            let found = match texts.get(number as usize) {
                None => Err(Error::index("list index out of range")),
                Some(blob) => lines(blob),
            };
            match found {
                Ok(found) => read.push(found),
                Err(error) if error.is(Kind::IndexError) => {
                    return Err(Error::new(
                        Kind::RomError,
                        format!("{system} has too few files"),
                    ));
                }
                Err(error) => return Err(error),
            }
        }
        let classes = read.pop().expect("three files read");
        let names = read.pop().expect("three files read");
        let facilities = read.pop().expect("three files read");
        let trainers: Vec<i64> = garc_files(&rom.read(TRAINERS)?)?
            .iter()
            .map(|data| {
                data.get(TRAINER_CLASS as usize)
                    .map_or(-1, |&class| class as i64)
            })
            .collect();
        let town_map = garc_files(&rom.read(TOWN_MAP)?)?;
        let Some(first) = town_map.first() else {
            return Err(Error::new(Kind::RomError, format!("{TOWN_MAP} is empty")));
        };
        let mut text = Story::new(stories, zones);
        let (sprites, own_names) = people(zones, &trainers, &classes, &names);
        let (signs, things) = signs_and_things(rom, zones, &mut text)?;
        let mut places: IndexMap<Slot, String> = things;
        let (doors, door_signs) = self::doors(zones, first, &facilities, &signs)?;
        for (slot, said) in signs_named(&signs)? {
            places.insert(slot, said);
        }
        places.extend(doors);
        Ok(SunGameText {
            sprites,
            trainers: own_names,
            places,
            door_signs,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Sign {
    pub area: i64,
    pub record: Record,
    /// Its first page's words, its lines kept.
    pub said: String,
}

/// The story text, its files read as they are asked for.
struct Story<'a> {
    files: Vec<Vec<u8>>,
    zones: &'a SunZones,
    read: HashMap<i64, Vec<Vec<u8>>>,
}

impl<'a> Story<'a> {
    fn new(files: Vec<Vec<u8>>, zones: &'a SunZones) -> Self {
        Story {
            files,
            zones,
            read: HashMap::new(),
        }
    }

    fn raw(&mut self, number: i64, index: Option<i64>) -> Result<Option<Vec<u8>>> {
        if !self.read.contains_key(&number) {
            let found = match self.files.get(number as usize) {
                None => Ok(Vec::new()),
                Some(file) => match message_lines(file) {
                    Err(error) if error.is(Kind::IndexError) || error.is(Kind::RomError) => {
                        Ok(Vec::new())
                    }
                    other => other,
                },
            }?;
            self.read.insert(number, found);
        }
        let lines = &self.read[&number];
        Ok(match index {
            Some(index) if 0 <= index && index < lines.len() as i64 => {
                Some(lines[index as usize].clone())
            }
            _ => None,
        })
    }

    fn line(&mut self, number: i64, index: Option<i64>) -> Result<Option<String>> {
        let raw = self.raw(number, index)?;
        Ok(raw.map(|raw| plain(&raw)).filter(|said| !said.is_empty()))
    }

    /// A line of a zone's own story file, or None where it holds a value the game fills in,
    /// a name or a number, which the words alone leave a gap for: "'s House", "の いえ".
    fn of_zone(&mut self, zone: Option<i64>, index: Option<i64>) -> Result<Option<String>> {
        let Some(zone) = zone.filter(|&zone| 0 <= zone && zone < self.zones.zones()) else {
            return Ok(None);
        };
        let number = u16(&self.zones.rows, zone * sm_zones::ZONE_ROW + ROW_STORY)?;
        let Some(raw) = self.raw(number, index)? else {
            return Ok(None);
        };
        match tokens(&raw) {
            Err(error) if error.is(Kind::ValueError) => return Ok(None),
            Err(error) => return Err(error),
            Ok(found) => {
                if found.iter().any(|token| {
                    matches!(token, Token::Control(control)
                        if VALUES.0 <= control.code && control.code < VALUES.1)
                }) {
                    return Ok(None);
                }
            }
        }
        let said = plain(&raw);
        Ok(if said.is_empty() { None } else { Some(said) })
    }
}

fn lines(blob: &[u8]) -> Result<Vec<String>> {
    Ok(message_lines(blob)?
        .iter()
        .map(|line| plain(line))
        .collect())
}

/// Model -> the kind of trainer it is, and (zone, record) -> each trainer record's own.
pub(crate) fn people(
    zones: &SunZones,
    trainers: &[i64],
    classes: &[String],
    names: &[String],
) -> (IndexMap<i64, String>, IndexMap<(i64, i64), String>) {
    let mut standing: IndexMap<i64, Vec<i64>> = IndexMap::new();
    let mut own_names: IndexMap<(i64, i64), String> = IndexMap::new();
    for area in zones.areas.values() {
        for record in area.trainers.iter().flatten() {
            if !(0 < record.kind && record.kind < trainers.len() as i64)
                || !(0 <= trainers[record.kind as usize]
                    && trainers[record.kind as usize] < classes.len() as i64)
            {
                continue;
            }
            standing.entry(record.code).or_default().push(record.kind);
            let (kind, name) = (
                &classes[trainers[record.kind as usize] as usize],
                at(names, record.kind),
            );
            if !kind.is_empty()
                && !name.is_empty()
                && let Some(zone) = record.zone
            {
                own_names.insert((zone, record.number), format!("{kind} {name}"));
            }
        }
    }
    let mut sprites: IndexMap<i64, String> = IndexMap::new();
    for (&model, numbers) in &standing {
        let kinds: HashSet<i64> = numbers
            .iter()
            .map(|&number| trainers[number as usize])
            .collect();
        if numbers.len() < 2 || kinds.len() != 1 {
            continue;
        }
        let kind = &classes[*kinds.iter().next().expect("one kind") as usize];
        let who: HashSet<&str> = numbers.iter().map(|&number| at(names, number)).collect();
        if !kind.is_empty() {
            let said = if who.len() == 1 && who.iter().all(|name| !name.is_empty()) {
                format!("{kind} {}", who.iter().next().expect("one name"))
            } else {
                kind.clone()
            };
            sprites.insert(model, said);
        }
    }
    (sprites, own_names)
}

/// Every sign with its words, and every other thing named by its first sentence.
fn signs_and_things(
    rom: &Rom,
    zones: &SunZones,
    text: &mut Story,
) -> Result<(Vec<Sign>, IndexMap<Slot, String>)> {
    let (start, spans) = garc_spans(rom, sm_zones::EVENTS)?;
    let mut signs: Vec<Sign> = Vec::new();
    let mut things: IndexMap<Slot, String> = IndexMap::new();
    for (&number, area) in &zones.areas {
        let at = number * sm_zones::AREA_FILES + SCRIPTS;
        if at >= spans.len() as i64 {
            continue;
        }
        let (begin, end) = spans[at as usize];
        let packed =
            sm_zones::unpacked(&rom.read_part(sm_zones::EVENTS, start + begin, end - begin)?)?;
        let zone_scripts = script::scripts(&packed)?;
        for (part, records) in area.things.iter().enumerate() {
            let found = zone_scripts.get(part).and_then(Option::as_ref);
            for record in records {
                let (Some(found), Some(zone)) = (found, record.zone) else {
                    continue;
                };
                let slot = (zone, place_of(FURNITURE_SOURCE, record.x, record.z)?);
                if record.script == 0 && sm_zones::SIGNS.contains(&record.kind) {
                    let line = found.line(record.code, Some(1))?;
                    let said = text.line(SIGN_TEXT, line)?;
                    if let Some(said) = said {
                        signs.push(Sign {
                            area: number,
                            record: record.clone(),
                            said,
                        });
                    }
                    continue;
                }
                let line = found.line(record.code, None)?;
                let sentence = sentence(text.of_zone(record.zone, line)?.as_deref());
                if let Some(sentence) = sentence {
                    things.entry(slot).or_insert(sentence);
                }
            }
        }
    }
    Ok((signs, things))
}

/// Each sign by its first line, as a sign: "Route 1 sign", "Trainer Tips sign".
pub(crate) fn signs_named(signs: &[Sign]) -> Result<Vec<(Slot, String)>> {
    let mut out: Vec<(Slot, String)> = Vec::new();
    for sign in signs {
        let head = py::strip(sign.said.split('\n').next().unwrap_or(""));
        if !head.is_empty()
            && let Some(zone) = sign.record.zone
        {
            out.push((
                (
                    zone,
                    place_of(FURNITURE_SOURCE, sign.record.x, sign.record.z)?,
                ),
                own("sign-saying", &[("text", &head)]),
            ));
        }
    }
    Ok(out)
}

/// What a thing is called by what it says: its first sentence, its lines run together, or
/// its first line where the next is a slogan in quotation marks, as a building's board has,
/// "Hau'oli Police Station" over "“Keep our roads safe!”". None where that is too long to be a
/// name, says no word, or opens on a value the game fills in, a name: "'s House".
pub(crate) fn sentence(said: Option<&str>) -> Option<String> {
    let lines: Vec<&str> = said
        .unwrap_or("")
        .split('\n')
        .map(py::strip)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.is_empty() {
        return None;
    }
    let first: Vec<char> = lines[0].chars().collect();
    let mut sentence: Vec<char> = if lines.len() > 1
        && lines[1]
            .chars()
            .next()
            .is_some_and(|opening| OPENING.contains(opening))
        && sentence_match(&first).is_none()
    {
        first
    } else {
        let words: Vec<char> = py::split(&lines.join(" ")).join(" ").chars().collect();
        let found: Vec<char> = match sentence_match(&words) {
            Some(end) => words[..end].to_vec(),
            None => words,
        };
        py::strip(&found.iter().collect::<String>())
            .chars()
            .collect()
    };
    if sentence.len() > LONGEST
        || !sentence.iter().any(|&letter| py::is_alnum(letter))
        || !(py::is_alnum(sentence[0]) || OPENING.contains(sentence[0]))
    {
        return None;
    }
    // A name is said in a list, "It's Meowth's bed, 2 of 9": its one full stop goes, "!" stays.
    let last = sentence[sentence.len() - 1];
    let before = sentence.len().checked_sub(2).map(|at| sentence[at]);
    if FULL_STOPS.contains(last) && before.is_some_and(|before| !FULL_STOPS.contains(before)) {
        sentence.pop();
    }
    Some(sentence.into_iter().collect())
}

#[derive(Debug, Clone, PartialEq)]
struct Door {
    area: i64,
    record: Record,
}

impl Door {
    fn slot(&self) -> Result<Slot> {
        let zone = self.record.zone.expect("a door's zone");
        Ok((zone, place_of(WARP_SOURCE, self.record.x, self.record.z)?))
    }
}

/// The building behind each door in from outdoors, and the plates that say only that.
pub(crate) fn doors(
    zones: &SunZones,
    town_map: &[u8],
    facilities: &[String],
    signs: &[Sign],
) -> Result<(IndexMap<Slot, String>, HashSet<Slot>)> {
    let mut doors: Vec<Door> = Vec::new();
    for (&number, area) in &zones.areas {
        for record in area.warps.iter().flatten() {
            if into_building(zones, record)? {
                doors.push(Door {
                    area: number,
                    record: record.clone(),
                });
            }
        }
    }
    let mut music: IndexMap<Slot, i64> = IndexMap::new();
    for door in &doors {
        music.insert(door.slot()?, self::music(zones, door.record.leads_to)?);
    }
    let mut town: IndexMap<Slot, Option<i64>> = IndexMap::new();
    for door in &doors {
        town.insert(
            door.slot()?,
            zones.place_index(door.record.zone.unwrap_or(0))?,
        );
    }
    let mut listed: HashMap<Option<i64>, HashSet<i64>> = HashMap::new();
    let mut marks: Vec<(i64, i64, i64, i64, i64)> = Vec::new();
    let records = (town_map.len() as i64 - TOWN_MAP_RECORD + 1).max(0);
    for at in (0..records).step_by(TOWN_MAP_RECORD as usize) {
        let (_island, zone, marked, x, z) = (
            u32(town_map, at)?,
            u32(town_map, at + 4)?,
            u32(town_map, at + 8)?,
            u32(town_map, at + 12)?,
            u32(town_map, at + 16)?,
        );
        let line = u16(town_map, at + TOWN_MAP_LINE)?;
        let zone = zone & 0xFFFF;
        if !(marked == MARKED_CENTER || marked == MARKED_FACILITY)
            || zone == NO_ZONE
            || zone >= zones.zones()
            || !facility(facilities, line)
        {
            continue;
        }
        listed
            .entry(zones.place_index(zone)?)
            .or_default()
            .insert(line);
        if x != 0 || z != 0 {
            marks.push((line, zones.place_index(zone)?.unwrap_or(0), marked, x, z));
        }
    }
    let mut named: IndexMap<Slot, i64> = IndexMap::new();
    for &(line, place, marked, x, z) in &marks {
        let mut here: Vec<&Door> = Vec::new();
        for door in &doors {
            if town[&door.slot()?] == Some(place) {
                here.push(door);
            }
        }
        let within = if marked == MARKED_CENTER { AT } else { MARKED };
        for door in nearest(&here, x as f64, z as f64, within) {
            named.entry(door.slot()?).or_insert(line);
        }
    }
    // X's rule: a kind behind doors in two towns or more is the facility every one of them lists.
    let mut towns: IndexMap<i64, HashSet<Option<i64>>> = IndexMap::new();
    for door in &doors {
        let slot = door.slot()?;
        towns.entry(music[&slot]).or_default().insert(town[&slot]);
    }
    let mut taken: HashSet<i64> = HashSet::new();
    let kinds: Vec<i64> = py::sorted_by_lt(towns.keys().copied().collect(), |a, b| {
        let (a, b) = (
            (-(towns[a].len() as i64), *a),
            (-(towns[b].len() as i64), *b),
        );
        a < b
    });
    for kind in kinds {
        if towns[&kind].len() < 2 {
            continue;
        }
        let empty: HashSet<i64> = HashSet::new();
        let lists: Vec<&HashSet<i64>> = towns[&kind]
            .iter()
            .map(|place| listed.get(place).unwrap_or(&empty))
            .collect();
        let mut common: HashSet<i64> = lists[0]
            .iter()
            .copied()
            .filter(|line| lists[1..].iter().all(|other| other.contains(line)))
            .filter(|line| !taken.contains(line))
            .collect();
        let mut marked_as: HashSet<i64> = HashSet::new();
        for door in &doors {
            let slot = door.slot()?;
            if music[&slot] == kind
                && let Some(&line) = named.get(&slot)
            {
                marked_as.insert(line);
            }
        }
        if !marked_as.is_empty() {
            common.retain(|line| marked_as.contains(line));
        }
        if common.len() != 1 {
            continue;
        }
        let line = *common.iter().next().expect("one line");
        taken.insert(line);
        for door in &doors {
            let slot = door.slot()?;
            if music[&slot] == kind {
                named.entry(slot).or_insert(line);
            }
        }
    }
    let mut out: IndexMap<Slot, String> = IndexMap::new();
    for (&slot, &line) in &named {
        let said = facilities
            .get(line as usize)
            .ok_or_else(|| Error::index("list index out of range"))?;
        out.insert(slot, said.clone());
    }
    // Plates name what the Town Map leaves.
    let place_names: HashSet<&str> = zones
        .places
        .files
        .get(&sm_zones::PLACE_NAMES)
        .map(|lines| {
            lines
                .iter()
                .map(String::as_str)
                .filter(|said| !said.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let mut door_signs: HashSet<Slot> = HashSet::new();
    for sign in signs {
        let lines: Vec<&str> = sign
            .said
            .split('\n')
            .map(py::strip)
            .filter(|line| !line.is_empty())
            .collect();
        if sign.record.kind != PLATE || lines.is_empty() || place_names.contains(lines[0]) {
            continue;
        }
        let here: Vec<&Door> = doors.iter().filter(|door| door.area == sign.area).collect();
        let building = nearest(&here, sign.record.x, sign.record.z, NEAR);
        if building.is_empty() {
            continue;
        }
        for door in building {
            out.entry(door.slot()?)
                .or_insert_with(|| lines[0].to_string());
        }
        if lines.len() == 1
            && let Some(zone) = sign.record.zone
        {
            door_signs.insert((
                zone,
                place_of(FURNITURE_SOURCE, sign.record.x, sign.record.z)?,
            ));
        }
    }
    Ok((out, door_signs))
}

fn into_building(zones: &SunZones, record: &Record) -> Result<bool> {
    let (Some(zone), Some(leads_to)) = (record.zone, record.leads_to) else {
        return Ok(false);
    };
    Ok(record.kind == sm_zones::WARP_DOOR && zones.outdoors(zone)? && !zones.outdoors(leads_to)?)
}

fn music(zones: &SunZones, zone: Option<i64>) -> Result<i64> {
    let Some(zone) = zone.filter(|&zone| 0 <= zone && zone < zones.zones()) else {
        return Ok(-1);
    };
    u16(&zones.rows, zone * sm_zones::ZONE_ROW + ROW_MUSIC)
}

/// Whether a line of file 100 is a facility's name: one line of words, not a description.
fn facility(facilities: &[String], line: i64) -> bool {
    let said = at(facilities, line);
    !said.is_empty() && !said.contains('\n')
}

/// The doors into the building whose door is nearest (x, z), where it is within `within`
/// cells and the nearest door into any other building is `APART` times as far or more; none
/// otherwise. A building's doors side by side are one: Royal Avenue's store has two.
fn nearest<'a>(doors: &[&'a Door], x: f64, z: f64, within: f64) -> Vec<&'a Door> {
    let cells = |door: &Door| py::hypot(door.record.x - x, door.record.z - z) / CELL;
    let ranked: Vec<&Door> = py::sorted_by_lt(doors.to_vec(), |a, b| cells(a) < cells(b));
    if ranked.is_empty() || cells(ranked[0]) > within {
        return Vec::new();
    }
    let inside = (ranked[0].area, ranked[0].record.leads_to);
    let other = ranked
        .iter()
        .find(|door| (door.area, door.record.leads_to) != inside);
    if let Some(other) = other
        && cells(ranked[0]) > AT
        && cells(other) < cells(ranked[0]) * APART
    {
        return Vec::new();
    }
    ranked
        .into_iter()
        .filter(|door| (door.area, door.record.leads_to) == inside)
        .collect()
}

fn at(lines: &[String], index: i64) -> &str {
    if 0 <= index && index < lines.len() as i64 {
        &lines[index as usize]
    } else {
        ""
    }
}
