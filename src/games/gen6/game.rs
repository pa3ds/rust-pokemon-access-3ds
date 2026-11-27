//! What one game keeps somewhere else than the others, for the games the gen 6 readers read.
//!
//! X and Y are the same engine as ORAS: the same classes, the same message windows and layouts, the
//! same move models and zone files (spike 100). What differs is a handful of numbers — where the ROM
//! keeps each kind of file, and one offset in a move model — and what one game has alone, as ORAS's
//! DexNav; a game package says its own in a `Game`, and builds the readers of what it alone has
//! beyond the field in its `Own` (`gen6.own`). Everything else is shared.
//!
//! Bound by:
//! - ADR-0006: gen 6 never names a game's own thing; what one game has alone is said in its Game.

use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};

use crate::core::memory::Memory;
use crate::games::gen6::anchors::Chain;
use crate::games::gen6::field::camera;
use crate::games::gen6::field::features::Feature;
use crate::games::gen6::field::footsteps::{ORAS, Steps};
use crate::games::gen6::field::landform::{Landform, gen6};
use crate::games::gen6::own::{self, Kit, Own};
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::rom::layout::RomLayout;
use crate::games::gen6::screens::buttons::MENU;
use crate::games::gen6::screens::{bag, boxes, rows, summary, townmap};
use crate::games::gen6::subscreen::amie::{NUMBER, ORAS_PLAYER, PlayerFields};
use crate::games::gen6::subscreen::super_training::ICONS;

/// Where one game keeps its screens' cursor sources (ADR-0043), each its screen's own
/// `Shape`, measured in that game. None where not measured: that screen is then its layout's
/// alone, as it was before its source was found.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CursorShapes {
    /// The X menu's button, the options' row and the party's slot (`rows.Rows`, spikes 298 and
    /// 306); the other rows `rows` reads are the same in Omega Ruby and X.
    pub rows: Option<rows::Shape>,
    /// The Bag's pocket, item and mode (`bag.Chosen`, spike 297).
    pub bag: Option<bag::Shape>,
    /// The Pokémon the summary shows and its pages' lists (`summary.Pages`, spikes 303 and
    /// 304).
    pub summary: Option<summary::Shape>,
    /// The PC Box's mode and the box its grid shows (`boxes.Mode`, `boxes.shown`, spikes 288
    /// and 298).
    pub r#box: Option<boxes::Shape>,
    /// The place under the AreaNav's map cursor (`townmap.Places`, spike 309).
    pub map_view: Option<townmap::Shape>,
}

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

/// `Callable[[Memory, bytes, GameText], Feature]`: a plain `fn`, since a `Game` is built once
/// and kept in a `static`, where an `Rc<dyn Fn>` cannot be.
pub type FeatureFactory = fn(Rc<dyn Memory>, &[u8], &GameText) -> Rc<dyn Feature>;
/// `Callable[[Kit], Own]`.
pub type OwnFactory = fn(&Kit) -> Rc<dyn Own>;

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
    /// What it has alone, built once from the profile's kit and asked where gen 6 asks it
    /// (`gen6.own`): ORAS's PokéNav Plus, contests, Pokéblock Kit, credits' Pokémon, Secret Base PC
    /// and soaring (`oras.own`), X's opening (`xy.own`); nothing, in a game with none of them.
    pub own: OwnFactory,
    /// The readers of what else it has on its field to list, each in its own words, given the
    /// executable's image where its table is, and the game's words: ORAS's berry spots, Route
    /// 113's ash piles, its Secret Bases' spots and the DexNav's find
    /// (`games.oras.field.berries`, `games.oras.field.ash`, `games.oras.field.secret_base`,
    /// `games.oras.field.hidden`). One that shows and goes between sweeps is read at every poll
    /// (`features.Watched`).
    pub features: Vec<FeatureFactory>,
    /// Its kinds of special ground in P's way, each saying what it lists and how it is walked,
    /// worked out once a map (`gen6.field.landform`): gen 6's own, which every game's chunks can
    /// hold, and its own after them: X's ridden Pokémon, climbing walls, grind rails, vines and
    /// Poké Ball Factory belts, and Dewford Gym's belts, Fortree Gym's rotating doors, Sootopolis
    /// Gym's thin ice and a Secret Base's floor, each of the last three read through a watch of
    /// its own as the player walks (`games.oras.field.doors`, `games.oras.field.ice`,
    /// `games.oras.field.secret_base`).
    pub landforms: Vec<Arc<dyn Landform>>,
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
    /// (or_mossy_rock), and X's, `KIRAKIRA2`.
    pub pickups: HashSet<i64>,
    /// The sprites of its invisible blockers, move models drawn as nothing that stand in the way:
    /// listed as nobody, and solid but where they stand in for a rotating door's arm. Not where a
    /// sprite is a barrier (`barriers`). With them the things drawn that answer nothing, as the
    /// things on Café Soleil's tables (`xy.PROPS`).
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
    /// a sprite, as X's does, or a sprite numbering a model, as ORAS's Pokémon's do; None among
    /// them where the field's models hold no things (`mmodel.sprite_of`). None where `drawn` holds
    /// the sprite alone.
    pub models: Option<(Option<i64>, i64)>,
    /// Warp kind -> what it is, where the kind says more than a door or stairs: ladders, a
    /// teleporter pad, the way into one building alone. A kind not here is "warp".
    pub warps: IndexMap<i64, String>,
    /// The warp kinds that are only where a teleporter pad puts the player, stepped onto doing
    /// nothing: neither listed nor walked round. X's `0x0001` (spike 265).
    ///
    /// A `frozenset` only tested.
    pub landings: HashSet<i64>,
    /// (zone, a warp's slot) -> the work and the value it is taken at, where a zone's script
    /// moves warps on one cell in and out of the way as a work changes: listed only while the work
    /// holds that value. The Team Flare Secret HQ's lift (spike 266).
    pub armed_warps: IndexMap<(i64, i64), (i64, i64)>,
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
    /// (`subscreen.super_training`): ORAS's 0xF0, X's 0xC8 (spike 228).
    pub training: i64,
    /// Where its Pokémon-Amie page keeps its Pokémon's number, its icon's holder following
    /// (`subscreen.amie`): ORAS's 0x14C, X's 0x124 (spike 228).
    pub amie: i64,
    /// Where Amie's `Player` keeps what is read of it (`subscreen.amie`): ORAS's, or X's
    /// (`xy.AMIE_PLAYER`), most of them 4 later (spike 287).
    pub amie_player: PlayerFields,
    /// Textures a game draws as a button's words with no line of its text to name them, named
    /// instead by pa3ds's own words for the picture (`pictures.named`) and offered as a button of
    /// their own, as the game's word would be: X's Pokémon-Amie menu (spike 228).
    pub worded: IndexSet<String>,
    /// Where the field's menu round the bottom screen's apps keeps its button manager and the byte
    /// set while an app has the input (`buttons`): ORAS's +0x7C and +0x9C, X's +0x70 and +0x90
    /// (spike 228).
    pub field_menu: (i64, i64),
    /// Where its screens keep what their cursors are on (ADR-0043).
    pub cursors: CursorShapes,
    /// Where its field camera's block keeps where the camera looks (`camera.Shape`): ORAS's, or
    /// X's (spikes 230 and 239).
    pub camera: camera::Shape,
    /// Where the save's Bag keeps the key items of the Y button's menu (`screens.registered`,
    /// spike 289): +0x0E in ORAS and X. None in a game that has no such menu.
    pub registered: Option<i64>,
    /// The zones of the places in parts listed whole, the other parts' things out of the ROM
    /// (ADR-0042); every other place in parts is listed a part at a time.
    pub whole: HashSet<i64>,
    /// (zone, record) of a thing the player takes an item at -> the item, whose name out of the
    /// ROM the thing is called by: the game names it only as the item taken, and one model draws
    /// both of Route 111's fossils (`PlaceNames.slot`).
    pub items_at: IndexMap<(i64, i64), i64>,
    /// Where the field's work holds the current zone's record, which leads to its data and the
    /// zone table: ORAS's +0xF4, X's +0xE4 (`FieldReader.anchor`, spike 282).
    pub zone_record: i64,
    /// The save's flags saying which version's story it plays: the flag of the version whose
    /// rooms are the higher zone of each pair of twins, then the other's. Of two warps on one cell
    /// into two places, the one into the room of the story played is listed (`reader._exits`). None
    /// in a game whose versions share every room.
    pub version_flags: Option<(i64, i64)>,
    /// By app, the chains from its process to the objects it keeps, found from the moment it
    /// starts (`gen6.anchors`, spike 282).
    pub anchors: IndexMap<String, Vec<Chain>>,
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
            clears: IndexMap::new(),
            dowsing: None,
            barriers: HashSet::new(),
            triggers: IndexMap::new(),
            furniture: IndexMap::new(),
            hidden_items: 0x44C,
            hidden_triggers: HashSet::new(),
            crossed: HashSet::new(),
            own: own::nothing,
            features: Vec::new(),
            landforms: gen6(),
            ambushes: HashSet::new(),
            lurkers: HashSet::new(),
            pickups: HashSet::new(),
            blockers: HashSet::new(),
            elevators: HashSet::new(),
            player_looks: HashSet::new(),
            stairs: HashSet::new(),
            rooms: HashSet::new(),
            models: None,
            warps: IndexMap::new(),
            landings: HashSet::new(),
            armed_warps: IndexMap::new(),
            teleporting: IndexMap::new(),
            gates: false,
            form: None,
            steps: ORAS,
            training: ICONS,
            amie: NUMBER,
            amie_player: ORAS_PLAYER,
            worded: IndexSet::new(),
            field_menu: MENU,
            cursors: CursorShapes::default(),
            camera: camera::ORAS,
            registered: None,
            whole: HashSet::new(),
            items_at: IndexMap::new(),
            zone_record: 0xF4,
            version_flags: None,
            anchors: IndexMap::new(),
        }
    }
}
