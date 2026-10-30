//! Where one gen 6 game keeps each kind of file in its ROM (`RomLayout`), a game's package saying
//! its own (`games.oras.ROM`, `games.xy.ROM`; spikes 100 and 101).

use std::collections::HashSet;

use indexmap::IndexMap;

use crate::core::binary::u16;
use crate::games::gen6::field::zonetable;
use crate::py::{Error, Kind, Result};

/// A game's list of what each town has, a row a town naming each facility by its line of a
/// text file: ORAS's AreaNav, X's Town Map. Both are rows of u32s, the town's outdoor zone and
/// then its facilities' lines to the end of the row, under numbers of each game's own.
#[derive(Debug, Clone, PartialEq)]
pub struct TownFacilities {
    pub archive: String,
    pub file: i64,
    /// The text file whose lines the facilities are.
    pub text: i64,
    pub row: i64,
    pub zone: i64,
    /// Where in a row the town's zone is, and where its facilities' lines begin.
    pub first: i64,
}

/// Where one game keeps what `GameText` reads: the same kinds of file in every Gen 6 game,
/// under each game's own numbers.
#[derive(Debug, Clone, PartialEq)]
pub struct RomLayout {
    /// What the game is called in a message saying its ROM is not what was expected.
    pub name: String,
    /// The text archive a language.
    pub text: IndexMap<String, String>,
    pub places: i64,
    pub trainer_names: i64,
    pub class_names: i64,
    pub pc: i64,
    /// Zone N's data file is file N, LZ11-compressed.
    pub zones: String,
    pub zone_table: i64,
    /// One small file a trainer, by the same number as the trainer names.
    pub trainers: String,
    /// Where a trainer's record keeps its class, and the struct format it is read with.
    pub trainer_class: (i64, String),
    /// Where a trainer's record says the battle it fights, `DOUBLE` for doubles.
    pub trainer_battle: i64,
    /// The list of what each town has, which names the buildings behind doors; None in a game
    /// without one.
    pub towns: Option<TownFacilities>,
    /// Sprite -> trainer, for the people the story names (`CHARACTERS`).
    pub characters: IndexMap<i64, i64>,
    /// Whether the trainer names open with the boy three times and the girl three times, which is
    /// how the text archive is checked. Without it, the place names are checked against the zone
    /// table instead.
    pub protagonists: bool,
    /// The map chunks' files, LZ11-compressed, by the number a map's field map lists each chunk
    /// under, where a game has streets placed on rails in them (`Streets`); None in one without.
    pub chunks: Option<String>,
    /// The sound archive, whose footsteps pa3ds plays where the game plays none (`footsteps`);
    /// None in a game whose footsteps are not measured.
    pub sounds: Option<String>,
    /// The story text archive a language, whose file a zone-table row's +0x06 gives is what the
    /// zone's script shows, its plates' words among it (`plates.plate_names`); none in a game
    /// whose plates are not read.
    pub story: IndexMap<String, String>,
    /// Where Lumiose's street marks are, a file a zone of its streets (`plates.street_names`).
    pub street_marks: Option<String>,
    /// The text file of Lumiose's street names, two lines a name.
    pub street_names: Option<i64>,
    /// The text file of the moves' names, a line a move by its number (`SURF`).
    pub moves: Option<i64>,
    /// The map chunks' files, LZ11-compressed, by the number a map's field map lists each chunk
    /// under, whose cell tables P's ground reads where the game has not loaded them (`MapChunks`);
    /// None where they are not read.
    pub map_chunks: Option<String>,
    /// The matrices' files, by the number a zone-table row's +0x04 gives (`MapChunks.matrix`);
    /// None where they are not read.
    pub matrices: Option<String>,
    /// The field's DLL, whose table lists every berry spot (`berry_spots`); None in a game
    /// whose berry spots are not read.
    pub berries: Option<String>,
    /// The archives holding textures with names, which `textures.census` indexes; none in a game
    /// whose pictures are not named by their image.
    pub textures: Vec<String>,
    /// The text file whose lines name the languages, and the line they start at: in the order
    /// English, Spanish, French, German, Italian, Japanese, Korean (`textures.LANGUAGES`).
    pub languages: Option<(i64, i64)>,
    /// The text file of the types' names, a line a type by its number.
    pub types: Option<i64>,
    /// The text file of the Pokémon's names, a line a species by its national number.
    pub species: Option<i64>,
    /// The text file of the Pokémon's forms' names, and the archive of their stats, a file a
    /// species by its national number saying how many forms it has (`form_names`); None where
    /// they are not read.
    pub forms: Option<(i64, String)>,
    /// The field's models' archive, a file a model, and the model a character of
    /// `field.POKEMON_MODEL` draws, from which the Pokémon on the map are named
    /// (`pokemon_models`); None in a game whose are not read.
    pub pokemon_models: Option<(String, i64)>,
    /// The battle models' archive, which says the Pokémon each model's tag is, in a game that tags
    /// some apart from their national numbers (`tagged_species`); None where a tag is the number.
    pub battle_models: Option<String>,
    /// The text file naming the moves' categories and the line they start at: status, physical,
    /// special, as the category icons number them (`textures`).
    pub categories: Option<(i64, i64)>,
    /// The text file naming the contest categories and the line they start at: coolness, beauty,
    /// cuteness, cleverness, toughness, as a contest's move icons number them (`textures`).
    pub contest_categories: Option<(i64, i64)>,
    /// The text file and line of a contest's labels of the entrant under its cursor, "Pokémon:"
    /// and "Trainer:" (`oras.screens.contest`).
    pub contest_entrant: Option<(i64, i64)>,
    /// The story text file and line of the game's word for a Pokémon's contest conditions, said
    /// before them (`oras.screens.conditions`).
    pub contest_conditions: Option<(i64, i64)>,
    /// The story text file naming the contests' computer entrants, the trainers in its first
    /// half and each one's Pokémon in the second, in the same order (`oras.screens.contest`).
    pub contest_rivals: Option<i64>,
    /// The text file naming the status conditions and the line they start at: paralysis, asleep,
    /// frozen, burned, poisoned, fainted, Pokérus, as the status icons number them (`textures`).
    pub conditions: Option<(i64, i64)>,
    /// Text files, and the line of each that names what each of the game's pictures that no word
    /// on screen names is a button for, by the image's file: the PokéNav's app buttons, the Bag's
    /// pockets (`games.common.touch`); none in a game whose are not read.
    pub touch_words: Vec<(i64, IndexMap<String, i64>)>,
    /// Text files, and the lines of each a screen draws as a label above or beside its value,
    /// which where they are drawn does not tell: the DexNav's "First Move" over its move. Each
    /// with what a mark drawn in its group says (`pictures.EGG_MOVE`), or "" (`games.common.screen`).
    pub labels: Vec<(i64, IndexMap<i64, String>)>,
    /// Text files, and the lines of each a screen draws as no label: a title, a button.
    pub plain: Vec<(i64, Vec<i64>)>,
    /// Images a button draws only while the Pokémon it shows is caught (`pictures.CAUGHT`).
    pub caught: HashSet<String>,
    /// The map's archive and file, which opens with the names of its layout's panes, and where a
    /// row of the list of what each town has keeps the index of its zone's touch area among them
    /// (`map_places`); None in a game whose map is not read.
    pub map_panes: Option<(String, i64, i64)>,
    /// The text file of a Secret Base's decorations' names, a line a decoration by its number;
    /// None in a game without Secret Bases.
    pub decorations: Option<i64>,
    /// The text file of the Poké Puffs' names, a line a puff by its kind, Sweet Poké Puff first
    /// and Supreme Winter Poké Puff last; None where they are not read.
    pub puffs: Option<i64>,
    /// The text file of Pokémon-Amie's wallpapers' names and then its interior items', Normal
    /// Wallpaper first and Wooden Bed last; None where they are not read.
    pub goods: Option<i64>,
    /// The text file of the items' names and the Mach Bike's line in it, which a way up a
    /// sandy slope is said with; None in a game without sandy slopes.
    pub bike: Option<(i64, i64)>,
    /// The same file and the Acro Bike's line, which a hop up a bumpy slope is said with; None in
    /// a game without bumpy slopes.
    pub acro: Option<(i64, i64)>,
    /// The text file of the items' names, a line an item by its number: the Red Orb is 534 in
    /// ORAS. Named in the descriptions of scenes (`games.common.scenes`).
    pub items: Option<i64>,
    /// Archives a game's own readers read for themselves, handed over whole (`GameText.files`):
    /// ORAS's sky's map of places (`oras.field.sky`).
    pub files: Vec<String>,
    /// The warp kinds each used by the way into one building alone, whose building the town's list
    /// names where no plate does (`plates.entrance_names`); none in a game whose are not measured.
    pub entrances: HashSet<i64>,
}

impl RomLayout {
    /// `RomLayout(name, text, places, trainer_names, class_names, pc, zones, zone_table, trainers,
    /// trainer_class, trainer_battle)`: everything after those is at its default, and is set
    /// with `RomLayout { towns, ..RomLayout::new(...) }`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        text: &IndexMap<&str, &str>,
        places: i64,
        trainer_names: i64,
        class_names: i64,
        pc: i64,
        zones: &str,
        zone_table: i64,
        trainers: &str,
        trainer_class: (i64, &str),
        trainer_battle: i64,
    ) -> Self {
        RomLayout {
            name: name.to_string(),
            text: owned(text),
            places,
            trainer_names,
            class_names,
            pc,
            zones: zones.to_string(),
            zone_table,
            trainers: trainers.to_string(),
            trainer_class: (trainer_class.0, trainer_class.1.to_string()),
            trainer_battle,
            towns: None,
            characters: IndexMap::new(),
            protagonists: false,
            chunks: None,
            sounds: None,
            story: IndexMap::new(),
            street_marks: None,
            street_names: None,
            moves: None,
            map_chunks: None,
            matrices: None,
            berries: None,
            textures: Vec::new(),
            languages: None,
            types: None,
            species: None,
            forms: None,
            pokemon_models: None,
            battle_models: None,
            categories: None,
            contest_categories: None,
            contest_entrant: None,
            contest_conditions: None,
            contest_rivals: None,
            conditions: None,
            touch_words: Vec::new(),
            labels: Vec::new(),
            plain: Vec::new(),
            caught: HashSet::new(),
            map_panes: None,
            decorations: None,
            puffs: None,
            goods: None,
            bike: None,
            acro: None,
            items: None,
            files: Vec::new(),
            entrances: HashSet::new(),
        }
    }
}

/// A table of a game's package, `str` to `str`, as a layout holds it.
pub fn owned(table: &IndexMap<&str, &str>) -> IndexMap<String, String> {
    table
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// The zone table, checked: rows of 0x38 bytes, every parent a zone in the table, and many of
/// them their own parent, as the outdoors are (spike 85).
pub fn zone_table(zones: &[Vec<u8>], layout: &RomLayout) -> Result<Vec<u8>> {
    if zones.len() as i64 <= layout.zone_table {
        return Err(Error::new(
            Kind::RomError,
            format!(
                "{} holds {} files, too few to hold the zone table",
                layout.zones,
                zones.len()
            ),
        ));
    }
    let table = &zones[layout.zone_table as usize];
    let size = zonetable::INFO_SIZE;
    let count = table.len() as i64 / size;
    if count < 100 || table.len() as i64 % size != 0 {
        return Err(Error::new(
            Kind::RomError,
            format!(
                "{} file {} is not the zone table",
                layout.zones, layout.zone_table
            ),
        ));
    }
    let mut parents: Vec<i64> = Vec::new();
    for zone in 0..count {
        parents.push(u16(table, zone * size + zonetable::INFO_PARENT)?);
    }
    if parents.iter().any(|&p| p >= count)
        || (parents
            .iter()
            .enumerate()
            .filter(|&(z, &p)| p == z as i64)
            .count())
            < 20
    {
        return Err(Error::new(
            Kind::RomError,
            format!(
                "{} file {} is not the zone table",
                layout.zones, layout.zone_table
            ),
        ));
    }
    Ok(table.clone())
}
