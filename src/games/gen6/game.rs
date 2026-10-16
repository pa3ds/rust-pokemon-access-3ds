//! What one game keeps somewhere else than the others, for the games the gen 6 readers read.
//!
//! X and Y are the same engine as ORAS: the same classes, the same message windows and layouts, the
//! same move models and zone files (spike 100). What differs is a handful of numbers — where the ROM
//! keeps each kind of file, and one offset in a move model — and what one game has alone, as ORAS's
//! DexNav; a game package says its own in a `Game`. Everything else is shared.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Speech;
use crate::core::toml;
use crate::games::common::announce::Focus;
use crate::games::common::field::things::Field;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::RunningApp;
use crate::games::gen6::field::features::{
    DoorReader, Feature, HiddenReader, IceReader, SecretBaseReader,
};
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::rom::layout::RomLayout;
use crate::py::{Log, Result};

/// An app only one game has, read by that game's own package: ORAS's soaring
/// (`oras.field.sky`). While it runs, the navigation keys walk what it lists instead of the
/// field's, and it is polled in the field's place.
pub trait OwnApp {
    /// The class its process is (`apps.RunningApp`).
    fn app(&self) -> &'static str;

    /// What the log and the poll's phase call it.
    fn name(&self) -> &'static str;

    /// The navigation keys' categories it lists things in, after the field's.
    fn categories(&self) -> Vec<&'static str>;

    /// Another app has started, this one or any other.
    fn started(&self);

    /// What the navigation keys walk while it runs, read through its process; None while it
    /// says nothing.
    fn listed(&self, process: i64) -> Result<Option<Field>>;

    /// Say what it says of itself, every field poll while it runs.
    fn poll(&self, process: i64) -> Result<()>;
}

/// Who stands where in a competition only one game has, said on the keys that say a battle's
/// sides: ORAS's contests (`oras.screens.contest`).
pub trait StandingsReader {
    /// The dialogue pages just said, which name whose turn it is.
    fn heard(&mut self, texts: &[&str]);

    /// Every screen poll, with every piece of screen text on screen.
    fn poll(&mut self, on_screen: &[ScreenText]) -> Result<()>;

    /// The player's standing, or everyone else's; None out of a competition. Python's `pauses`
    /// defaults to False.
    fn said(&self, player: bool, pauses: bool) -> Result<Option<String>>;
}

/// The choices an opening draws without a layout, said by the game's own package: X's boy or
/// girl and their three looks, models with no word on them (`xy.intro`).
pub trait OpeningReader {
    /// The class of the process the choices are made in (`apps.RunningApp`).
    fn app(&self) -> &'static str;

    /// The choice under the cursor, read through the process, when it has just opened or
    /// moved; else None.
    fn poll(&mut self, process: i64) -> Result<Option<Focus>>;
}

/// A Pokémon's contest conditions, an Extra, said where only one game draws them: ORAS's
/// (`oras.screens.conditions`).
pub trait ConditionsReader {
    /// Every screen poll, once the screen's words are said: the words under the cursors,
    /// everything on screen, the layouts still being written, and whether a cursor's details are
    /// still to be said. Python's `waiting` defaults to False.
    fn poll(
        &mut self,
        under: &[String],
        on_screen: &[ScreenText],
        settling: &IndexSet<i64>,
        waiting: bool,
    ) -> Result<()>;
}

/// `str | int`, what `Game.furniture` says a record is: a word, or the kind of use tile it is
/// called as.
#[derive(Debug, Clone, PartialEq)]
pub enum FurnitureKind {
    Word(String),
    Tile(i64),
}

/// A game's `data` folder as the binary carries it: each language's `sprites.toml`, by the
/// language's code. Python's is a `Path` to the folder and `pa3ds.core.names` reads
/// `<language>/sprites.toml` from it; the Rust binary has no folder beside it, so the files are
/// `include_str!`ed.
pub type Data = &'static [(&'static str, &'static str)];

/// `Callable[[Memory], HiddenReader]`: a plain `fn`, since a `Game` is built once and kept in a
/// `static`, where an `Rc<dyn Fn>` cannot be.
pub type HiddenFactory = fn(Rc<dyn Memory>) -> Rc<dyn HiddenReader>;
/// `Callable[[Memory, bytes], SecretBaseReader]`.
pub type SecretBaseFactory = fn(Rc<dyn Memory>, &[u8]) -> Rc<dyn SecretBaseReader>;
/// `Callable[[Memory, GameText], Feature]`.
pub type FeatureFactory = fn(Rc<dyn Memory>, &GameText) -> Rc<dyn Feature>;
/// `Callable[[Memory, GameText, Speech, Callable[[str], None]], OwnApp | None]`.
pub type AppFactory =
    fn(Rc<dyn Memory>, &GameText, Rc<Speech>, Log) -> Result<Option<Rc<dyn OwnApp>>>;
/// `Callable[[Memory], DoorReader]`.
pub type DoorFactory = fn(Rc<dyn Memory>) -> Rc<dyn DoorReader>;
/// `Callable[[Memory], IceReader]`.
pub type IceFactory = fn(Rc<dyn Memory>) -> Rc<dyn IceReader>;
/// `Callable[[Memory, Layouts, LayoutFiles, GameText, Callable[[], str | None]],
/// StandingsReader]`.
pub type StandingsFactory = fn(
    Rc<dyn Memory>,
    Rc<Layouts>,
    Rc<LayoutFiles>,
    &GameText,
    Rc<dyn Fn() -> Option<String>>,
) -> Rc<RefCell<dyn StandingsReader>>;
/// `Callable[[Memory, ClassNames, Layouts, LayoutFiles, RunningApp, GameText, Speech,
/// Callable[[str], None], bool], ConditionsReader]`.
pub type ConditionsFactory = fn(
    Rc<dyn Memory>,
    Rc<ClassNames>,
    Rc<Layouts>,
    Rc<LayoutFiles>,
    Rc<RefCell<RunningApp>>,
    &GameText,
    Rc<Speech>,
    Log,
    bool,
) -> Box<dyn ConditionsReader>;
/// `Callable[[Memory, str], OpeningReader]`.
pub type OpeningFactory = fn(Rc<dyn Memory>, &str) -> Box<dyn OpeningReader>;

/// One game as the gen 6 readers read it.
///
/// Not `PartialEq`: the dataclass compares its callables by identity, which a `fn` pointer does
/// not do reliably.
#[derive(Debug, Clone)]
pub struct Game {
    /// What the log calls it.
    pub name: String,
    /// The title ids it runs as, one a version.
    pub titles: HashSet<i64>,
    /// Its words for what the game never names, a folder a language (`pa3ds.core.names`).
    pub data: Data,
    /// Its descriptions of scenes, a `scenes.toml` a language from the same folder as `data`
    /// (`games.common.scenes`): Python reads `data / language / "scenes.toml"`.
    pub scenes: Data,
    /// Its words for the pictures the game draws and never names, a `pictures.toml` a language
    /// from the same folder as `data` (`games.common.pictures.described`): Python reads
    /// `data / language / "pictures.toml"`.
    pub pictures: Data,
    /// Where its ROM keeps the game's own words (`archives`).
    pub rom: RomLayout,
    /// Where a move model keeps the sprite it is drawn with (`mmodel.drawn_as`).
    pub drawn: i64,
    /// The reader of its hidden Pokémon, for the field to list, or None in a game with none: ORAS's
    /// DexNav (`games.oras.field.hidden`).
    pub hidden: Option<HiddenFactory>,
    /// (zone, a trigger's script) -> the way the conveyor belt that trigger is runs (`field`,
    /// spike 132). Nothing in a trigger's record says it is a belt, so each is known one by one.
    pub belts: IndexMap<(i64, i64), &'static str>,
    /// Sprite -> the move that clears the thing drawn as it, which Shift and P's way goes through:
    /// a bush Cut clears, a rock Rock Smash does, a boulder Strength pushes.
    pub clears: IndexMap<i64, i64>,
    /// Where the field's work holds the Dowsing Machine's record (`dowsing`), or None where it has
    /// not been measured, as in X and Y.
    pub dowsing: Option<i64>,
    /// (zone, sprite) of the move models that are a barrier's beam, three cells across the way the
    /// model faces (`field`, spike 146). The sprite is an invisible blocker of one cell elsewhere, so
    /// each zone is known one by one.
    pub barriers: HashSet<(i64, i64)>,
    /// (zone, a trigger's script) -> the category and kind it is listed as: a switch, the way
    /// between two rooms that share no floor, a trapdoor or a geyser between a floor and the hot
    /// spring under it, a lift or a teleporter between platforms, a Secret Base's way out, or the way
    /// into Seafloor Cavern, with the zone it leads to. As with the belts, nothing in the record says
    /// which.
    ///
    /// Python's value is `tuple[str, str] | tuple[str, str, int]`: the zone is the third item, None
    /// where the tuple has two.
    pub triggers: IndexMap<(i64, i64), (String, String, Option<i64>)>,
    /// (zone, a furniture record's script) -> what it is, where it is not a sign: a Secret Base's
    /// flag, Mossdeep Gym's switches; or the kind of use tile it is called as, a Secret Base's PC.
    pub furniture: IndexMap<(i64, i64), FurnitureKind>,
    /// The reader of its Secret Bases, given the executable's image to find every spot in, or None
    /// in a game with none: ORAS's (`games.oras.field.secret_base`).
    pub secret_bases: Option<SecretBaseFactory>,
    /// The readers of what else it has on its field to list, each in its own words: ORAS's berry
    /// spots and Route 113's ash piles (`games.oras.field.berries`, `games.oras.field.ash`).
    pub features: Vec<FeatureFactory>,
    /// The reader of its rotating doors, or None in a game with none: ORAS's, in Fortree Gym
    /// (`games.oras.field.doors`).
    pub doors: Option<DoorFactory>,
    /// The reader of its rooms of thin ice, or None in a game with none: ORAS's, in Sootopolis
    /// Gym (`games.oras.field.ice`).
    pub ice: Option<IceFactory>,
    /// The sprites of its invisible blockers, move models drawn as nothing that stand in the way:
    /// listed as nobody, and solid but where they stand in for a rotating door's arm. Not where a
    /// sprite is a barrier (`barriers`).
    pub blockers: HashSet<i64>,
    /// The scripts of the furniture records an elevator's door is, one a cell: side by side, they
    /// are one thing.
    pub elevators: HashSet<i64>,
    /// The sprites the player is drawn as: a model drawn as one, standing in for the player in a
    /// scene, is the player and not listed.
    pub player_looks: HashSet<i64>,
    /// Zones whose chunks' floors are joined by stairs, each listed under exits where it meets
    /// the floor above: Sootopolis Gym's two, the way back up from its lower floor.
    pub stairs: HashSet<i64>,
    /// Its own apps' readers, each built from the memory, the ROM's words, and where to say and
    /// log, or None where what it needs is not read: ORAS's soaring (`oras.field.sky`).
    pub apps: Vec<AppFactory>,
    /// Warp kind -> what it is, where the kind says more than a door or stairs: ladders, a
    /// teleporter pad, the way into one building alone. A kind not here is "warp".
    pub warps: IndexMap<i64, String>,
    /// The reader of who stands where in its competitions, built from the memory, the layouts
    /// and their files, the ROM's words and the app running, or None in a game with none: ORAS's
    /// contests (`oras.screens.contest`).
    pub standings: Option<StandingsFactory>,
    /// The reader of a Pokémon's contest conditions, built as the Extra is turned on, or None in
    /// a game that draws none: ORAS's (`oras.screens.conditions`).
    pub conditions: Option<ConditionsFactory>,
    /// The reader of the choices its opening draws without a layout, built from the memory and
    /// the player's language, or None where the opening's choices are layouts, as ORAS's are: X's
    /// (`xy.intro`).
    pub opening: Option<OpeningFactory>,
}

impl Game {
    /// `Game(name, titles, data, rom, drawn)`: every other field at its default, set with
    /// `Game { hidden, ..Game::new(...) }`.
    pub fn new(
        name: &str,
        titles: impl IntoIterator<Item = i64>,
        data: Data,
        rom: RomLayout,
        drawn: i64,
    ) -> Self {
        Game {
            name: name.to_string(),
            titles: titles.into_iter().collect(),
            data,
            scenes: &[],
            pictures: &[],
            rom,
            drawn,
            hidden: None,
            belts: IndexMap::new(),
            clears: IndexMap::new(),
            dowsing: None,
            barriers: HashSet::new(),
            triggers: IndexMap::new(),
            furniture: IndexMap::new(),
            secret_bases: None,
            features: Vec::new(),
            doors: None,
            ice: None,
            blockers: HashSet::new(),
            elevators: HashSet::new(),
            player_looks: HashSet::new(),
            stairs: HashSet::new(),
            apps: Vec::new(),
            warps: IndexMap::new(),
            standings: None,
            conditions: None,
            opening: None,
        }
    }

    /// The names the game's package ships for a language, read as `Names.load` reads
    /// `data / language / "sprites.toml"`: a language the package has no file for, or a file that
    /// does not parse, is no names at all. Written for the runtime's `Names.load`, which reads a
    /// folder the Rust binary does not have.
    pub fn sprites(&self, language: &str) -> IndexMap<String, String> {
        toml::strings(self.table(language).get(crate::core::names::SPRITES))
    }

    /// `toml.read(data / language / "sprites.toml")` of the table the binary carries: every
    /// section, the sprites' names, the use tiles' and furniture's words and the slots'.
    pub fn table(&self, language: &str) -> IndexMap<String, ::toml::Value> {
        let Some((_, text)) = self.data.iter().find(|(held, _)| *held == language) else {
            return IndexMap::new();
        };
        if text.starts_with('\u{feff}') {
            IndexMap::new()
        } else {
            match text.parse::<::toml::Table>() {
                Ok(loaded) => toml::table(Some(&::toml::Value::Table(loaded))),
                Err(_) => IndexMap::new(),
            }
        }
    }
}
