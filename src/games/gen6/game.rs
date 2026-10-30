//! What one game keeps somewhere else than the others, for the games the gen 6 readers read.
//!
//! X and Y are the same engine as ORAS: the same classes, the same message windows and layouts, the
//! same move models and zone files (spike 100). What differs is a handful of numbers — where the ROM
//! keeps each kind of file, and one offset in a move model — and what one game has alone, as ORAS's
//! DexNav; a game package says its own in a `Game`. Everything else is shared.
//!
//! Bound by:
//! - ADR-0006: gen 6 never names a game's own thing; what one game has alone is said in its Game.

use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::discovery::Found;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::core::speech::Speech;
use crate::games::common::announce::Focus;
use crate::games::common::field::things::Field;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::apps::RunningApp;
use crate::games::gen6::field::camera;
use crate::games::gen6::field::features::{
    DoorReader, Feature, HiddenReader, IceReader, SecretBaseReader,
};
use crate::games::gen6::field::footsteps::{ORAS, Steps};
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::rom::layout::RomLayout;
use crate::games::gen6::screens::amie::NUMBER;
use crate::games::gen6::screens::buttons::MENU;
use crate::games::gen6::screens::training::ICONS;
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

/// The Pokémon one game's save keeps for its credits' moments, which scenes name as
/// `{memory:N}`: ORAS's (`oras.memories`).
pub trait MemoriesReader {
    /// What the heap is swept for to find the save's object.
    fn classes(&self) -> Vec<String>;
    /// What a sweep found.
    fn track(&self, found: &Found);
    /// The species and form the save keeps for a moment, or None where it keeps none.
    fn pokemon(&self, moment: i64) -> Result<Option<(i64, i64)>>;
    /// The nickname the player gave the Pokémon of a moment, or None where they gave none
    /// or no longer have it.
    fn nickname(&self, moment: i64) -> Result<Option<String>>;
}

/// `Callable[[Memory], MemoriesReader]`.
pub type MemoriesFactory = fn(Rc<dyn Memory>) -> Rc<dyn MemoriesReader>;

/// `str | int | tuple[str, str, int]`, what `Game.furniture` says a record is: a word, the kind
/// of use tile it is called as, or, for a record the player leaves by, the category it is listed
/// under, its kind and the zone it leads to.
#[derive(Debug, Clone, PartialEq)]
pub enum FurnitureKind {
    Word(String),
    Tile(i64),
    Exit(String, String, i64),
}

/// A game's own words' folder beside pa3ds, `data/oras` (ADR-0040). Python's `DATA` is a `Path`
/// made as the module loads; a `Game` lives in a `static`, so here it is the function making it.
pub type Data = fn() -> PathBuf;

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

/// The ground only a Pokémon the player rides crosses, as X's Rhyhorn crosses Route 9's spiky
/// ground: a wall on foot, got onto from the flat ground beside it, where the Pokémon waits to be
/// ridden and is got off (`Ground.mounted`, spike 238).
///
/// Its `frozenset`s are `HashSet`s: they are only tested.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rides {
    /// The cell kinds only a player riding crosses.
    pub ground: HashSet<i64>,
    /// The kinds of the flat ground it is got on and off on, walked on foot and riding.
    pub stops: HashSet<i64>,
    /// The characters of the Pokémon ridden.
    pub mounts: HashSet<i64>,
    /// The sprites of the things the ridden Pokémon smashes with A, in the way on foot.
    pub smashes: HashSet<i64>,
}

/// The cell kinds of the walls a player climbs, as X's Cyllage Gym's (`climb.Walls`).
#[derive(Debug, Clone, PartialEq)]
pub struct Climbing {
    /// Where a face meets the floor, blocked.
    pub face: i64,
    /// The kinds of the step off a face sideways onto a ledge, each with its way.
    pub sideways: IndexMap<i64, &'static str>,
    /// The top of a slide down a wall, open: a slide only where the cells past it fall.
    pub slide: i64,
}

/// One zone's conveyor belts that carry the player (`Ground.conveyors`), and the switch that
/// turns some of them round, as X's Poké Ball Factory's (spike A).
#[derive(Debug, Clone, PartialEq)]
pub struct Conveyors {
    /// Each cell of a belt with the way it carries the player while the switch has turned none.
    pub ways: IndexMap<(i64, i64), &'static str>,
    /// The cells the switch turns round, each with the way it carries the player once turned.
    pub turned: IndexMap<(i64, i64), &'static str>,
    /// The script of the furniture record that is the switch, pressed with A at it.
    pub switch: i64,
    /// The class of the object, and where in it, that points at the byte saying whether the
    /// switch has turned the belts, 0 for not: the save keeps it.
    pub state: (&'static str, i64),
}

/// The cell kinds of the rails a player on roller skates grinds along, as X's Shalour City
/// Gym's (`grind.GrindRails`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grinding {
    /// A cell of a rail, blocked.
    pub rail: i64,
    /// The cell past a rail's end that it is skated onto from, open.
    pub end: i64,
}

/// The natives the zone script's case of a vine's end hands one constant to, as X's Coumarine
/// City Gym's (`vines.Vines`, `Script.native`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vining {
    /// A vine swung on across a gap.
    pub swing: i64,
    /// A vine climbed, up from its lower end and down from its upper.
    pub climb: i64,
}

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
    /// Its own words, for what the game never names, beside pa3ds: its scenes' timing, and a
    /// folder a language (ADR-0040).
    pub data: Data,
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
    /// not been measured.
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
    /// flag, Mossdeep Gym's switches; or the kind of use tile it is called as, a Secret Base's PC; or,
    /// as `triggers` has it, a category, a kind and the zone it leads to, for a record the player
    /// leaves by: the pole in Santalune Gym's lobby, which the game slides the player down to the
    /// web room (spike 224).
    pub furniture: IndexMap<(i64, i64), FurnitureKind>,
    /// The reader of its Secret Bases, given the executable's image to find every spot in, or None
    /// in a game with none: ORAS's (`games.oras.field.secret_base`).
    pub secret_bases: Option<SecretBaseFactory>,
    /// The event flag before the first hidden item's, each item's its number past it: `0x44c` in
    /// Omega Ruby's Dowsing Machine's getter, `0x4e49e4`, and in X's, `0x4ab888`
    /// (`gen6.field.hidden_items`, spike 249).
    pub hidden_items: i64,
    /// (zone, a trigger's script) of each trigger that gives an item, listed as a hidden item
    /// while it is armed: ORAS's Scanner. Listed wherever the hidden items are, under the sea or
    /// everywhere, since the player dives to it in a zone the zone table does not put under the
    /// sea.
    pub hidden_triggers: HashSet<(i64, i64)>,
    /// (zone, a trigger's script) of the listed triggers P walks over on its way, where any other
    /// is stepped round: Dewford Gym's switches, in the corridor its belts carry the player down,
    /// which light its side passages and turn nothing (spike 132). A belt is walked over too.
    pub crossed: HashSet<(i64, i64)>,
    /// Zone -> its conveyor belts that carry the player, and the switch that turns some round:
    /// X's Poké Ball Factory, whose belts a native of the game's runs with nothing in the zone, its
    /// chunks or the trigger saying where (spike A).
    pub conveyors: IndexMap<i64, Conveyors>,
    /// The readers of what else it has on its field to list, each in its own words: ORAS's berry
    /// spots and Route 113's ash piles (`games.oras.field.berries`, `games.oras.field.ash`).
    pub features: Vec<FeatureFactory>,
    /// The reader of its rotating doors, or None in a game with none: ORAS's, in Fortree Gym
    /// (`games.oras.field.doors`).
    pub doors: Option<DoorFactory>,
    /// The reader of its rooms of thin ice, or None in a game with none: ORAS's, in Sootopolis
    /// Gym (`games.oras.field.ice`).
    pub ice: Option<IceFactory>,
    /// The sprites of the spots a wild Pokémon leaps out of as the player walks near, listed
    /// under pokémon rather than as a thing to use: X's grass on Routes 6 and 18, `TOBI_KUSA`
    /// (2026-10-19).
    pub ambushes: HashSet<i64>,
    /// The sprites of the spots a wild Pokémon waits at unseen, a battle as the player comes up
    /// to one and nothing in the way after: listed under pokémon, and walked through. X's
    /// Glittering Cave, `ENEMY3D` (spike 239).
    pub lurkers: HashSet<i64>,
    /// The sprites, other than an item ball's, of the things the player finds an item at, listed
    /// under items: ORAS's Mega Stones, `KIRAKIRA`, an "object" among the signs in Petalburg Woods
    /// (Orion, or_mossy_rock, 2026-10-25), and X's, `KIRAKIRA2`.
    pub pickups: HashSet<i64>,
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
    /// Zones that are several rooms side by side with no floor between them, which the game
    /// moves the player between by triggers (`triggers`): only the room the player stands in is
    /// listed. X's house, its bedroom and its ground floor.
    pub rooms: HashSet<i64>,
    /// Where its things' and its Pokémon's models start, where `drawn` holds a model rather than
    /// a sprite (`mmodel.sprite_of`): X's; None where it holds the sprite.
    pub models: Option<(i64, i64)>,
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
    /// The reader of the Pokémon its save keeps for its credits, or None in a game whose credits
    /// have not been looked at: ORAS's (`oras.memories`, spike 222).
    pub memories: Option<MemoriesFactory>,
    /// The bit of a cell's word that marks a strand the player walks diagonally between, or 0 in a
    /// game with none: X's Santalune Gym is a web of them, and ORAS's chunks use the same bit for
    /// other things (`Ground.strands`, spike 223).
    pub strands: i64,
    /// What the Pokémon it lets the player ride cross, or None in a game with none: X's Rhyhorn on
    /// Route 9 (spike 238).
    pub rides: Option<Rides>,
    /// The walls it lets the player climb, or None in a game with none: X's Cyllage Gym
    /// (spike 245).
    pub climbing: Option<Climbing>,
    /// The rails it lets the player grind along on roller skates, or None in a game with none:
    /// X's Shalour City Gym (spike 247).
    pub grinding: Option<Grinding>,
    /// The vines it lets the player grab, or None in a game with none: X's Coumarine City Gym
    /// (spike 250).
    pub vining: Option<Vining>,
    /// Zone -> the native its teleporters' cases hand where they put the player, x, the floor in
    /// squares of height and z (`plates.teleports`): X's Laverre City Gym (spike 258). By zone, since a
    /// native's number is its zone script's own: Coumarine Gym's 56 swings a vine.
    pub teleporting: IndexMap<i64, i64>,
    /// Whether a row of models drawn as nothing, two or more with one script that no drawn record
    /// of their zone has, is an invisible gate: not listed, and in the way. X's Santalune Gym has
    /// five behind Viola, and AZ's room five (spike 223).
    pub gates: bool,
    /// Where the player's move model says what they move as, on foot, on the bike or on a
    /// Pokémon (`mmodel.FORM`), or None where it was not read: X's.
    pub form: Option<i64>,
    /// Where its move model keeps a step's moment and which ground it plays no footstep on
    /// (`footsteps.Steps`): ORAS's, or X's (spike 231).
    pub steps: Steps,
    /// Where its Super Training page keeps the PokeIcon of its party's icons, its records following
    /// (`screens.training`): ORAS's 0xF0, X's 0xC8 (spike 228).
    pub training: i64,
    /// Where its Pokémon-Amie page keeps its Pokémon's number, its icon's holder following
    /// (`screens.amie`): ORAS's 0x14C, X's 0x124 (spike 228).
    pub amie: i64,
    /// Textures a game draws as a button's words with no line of its text to name them, named
    /// instead by pa3ds's own words for the picture (`pictures.named`) and offered as a button of
    /// their own, as the game's word would be: X's Pokémon-Amie menu (spike 228).
    pub worded: IndexSet<String>,
    /// Where the field's menu round the bottom screen's apps keeps its button manager and the byte
    /// set while an app has the input (`buttons`): ORAS's +0x7C and +0x9C, X's +0x70 and +0x90
    /// (spike 228).
    pub field_menu: (i64, i64),
    /// Where its field camera's block keeps where the camera looks (`camera.Shape`): ORAS's, or
    /// X's (spikes 230 and 239).
    pub camera: camera::Shape,
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
            hidden_items: 0x44C,
            hidden_triggers: HashSet::new(),
            crossed: HashSet::new(),
            conveyors: IndexMap::new(),
            features: Vec::new(),
            doors: None,
            ice: None,
            ambushes: HashSet::new(),
            lurkers: HashSet::new(),
            pickups: HashSet::new(),
            blockers: HashSet::new(),
            elevators: HashSet::new(),
            player_looks: HashSet::new(),
            stairs: HashSet::new(),
            rooms: HashSet::new(),
            models: None,
            apps: Vec::new(),
            warps: IndexMap::new(),
            standings: None,
            conditions: None,
            opening: None,
            memories: None,
            strands: 0,
            rides: None,
            climbing: None,
            grinding: None,
            vining: None,
            teleporting: IndexMap::new(),
            gates: false,
            form: None,
            steps: ORAS,
            training: ICONS,
            amie: NUMBER,
            worded: IndexSet::new(),
            field_menu: MENU,
            camera: camera::ORAS,
        }
    }
}
