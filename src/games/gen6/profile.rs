//! The gen 6 profile: which of the game's objects to find, the readers they go to, and how each
//! poll runs them.
//!
//! The runtime (`app`) knows when to sweep and when to poll; this knows what a sweep should look for,
//! which reader each object found belongs to, and in what order the readers are polled and their news
//! said. It reads ORAS and Pokémon X and Y, one engine, each game told apart by its `Game`: its
//! numbers, and what it alone has (spike 100).
//!
//! Bound by:
//! - ADR-0041: a replay logs touch mode's targets as they change, so a tape holds them.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::u32;
use crate::core::clips::Clips;
use crate::core::config::{Extras, Navigation, Ui};
use crate::core::discovery::{Discovery, Found, HEAP_SPAN, LINEAR_SPAN, within};
use crate::core::memory::{Cache, Memory, phase};
use crate::core::names::{FURNITURE, GamePlace, GameSprite, Names, TILES};
use crate::core::pathfind::{COMPASS, walked};
use crate::core::rtti::{ClassNames, demangle};
use crate::core::speech::{Category, Speech};
use crate::core::tones::{Tones, ticking};
use crate::core::touch::Drag;
use crate::core::words::own;
use crate::games::common::announce::{Announcements, Speaker};
use crate::games::common::cursors::reader::CursorReader;
use crate::games::common::dialogue::{DialogueReader, Shown};
use crate::games::common::field::ground::Ground;
use crate::games::common::field::guide::{FieldSource, Place};
use crate::games::common::field::keys::Bindings;
use crate::games::common::field::navigate::{
    DoorSign, LeadsTo, Navigator, Options as NavigatorOptions, Sides, Street,
};
use crate::games::common::field::things::{
    FURNITURE_SOURCE, Field, Interactable, WARP_SOURCE, bearing, describe, source_kind,
    source_number,
};
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::{LayoutFile, LayoutFiles};
use crate::games::common::pictures::CAUGHT;
use crate::games::common::pictures::named as named_picture;
use crate::games::common::scenes::{
    Described, GameClock, Lines, Recall, Scenes, Word, load as load_scenes,
    shipped as scenes_shipped,
};
use crate::games::common::screen::{Pictures, ScreenPoll, ScreenText, ScreenTextReader};
use crate::games::common::screen_announce::ScreenAnnouncer;
use crate::games::common::session::Profile;
use crate::games::common::slots::Occupant;
use crate::games::common::touch::{
    self, FileOf, Idle, ImageOf, Lists, OnScreen, PictureName, Tap, TextOf, TouchTargets,
};
use crate::games::common::{lyt, msgwin, pokemon, strbuf};
use crate::games::gen6::apps::{BATTLE, DEMOS, FIELD, MANAGER_CLASS, RunningApp};
use crate::games::gen6::field::dowsing::Lights;
use crate::games::gen6::field::footsteps::Footsteps;
use crate::games::gen6::field::reader::{FieldReader, MOVE_MODEL_CLASS};
use crate::games::gen6::field::speaker::SpeakerReader;
use crate::games::gen6::field::{camera, mapblock, talk};
use crate::games::gen6::game::MemoriesReader;
use crate::games::gen6::game::{ConditionsReader, Game, OpeningReader, OwnApp, StandingsReader};
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::rom::textures;
use crate::games::gen6::rom::textures::Index;
use crate::games::gen6::screens::amie::{AmiePage, Goods, Puffs};
use crate::games::gen6::screens::battle::{BattleReader, Battlers, GAUGE_CLASSES, Gauges};
use crate::games::gen6::screens::choices::{
    BOX_DEVICE_MANAGER, ChoiceReader, DEVICE_MANAGER_CLASS,
};
use crate::games::gen6::screens::distribution::{Distribution, DistributionReader};
use crate::games::gen6::screens::habitat::HabitatReader;
use crate::games::gen6::screens::lists::{CONTROLLER_CLASS, ListReader};
use crate::games::gen6::screens::mapcursor::MapCursor;
use crate::games::gen6::screens::partners::Partners;
use crate::games::gen6::screens::partyicons::PartyIcons;
use crate::games::gen6::screens::puzzle::TilePuzzle;
use crate::games::gen6::screens::training::{Bags, Goals, Round, TrainingPage};
use crate::games::gen6::screens::waiting::Waiting;
use crate::games::gen6::screens::{
    amie, berry_picker, boxes, buttons, distribution, partners, partyicons, puzzle, summary,
    townmap, training,
};
use crate::games::gen6::{frames, gflheap, mystatus};
use crate::py::{self, Error, Log, PySet, Result};

/// What the heap is swept for: message windows, list controllers, the device manager a choice box
/// holds, layout works through their DrawInfo, the battlers' names and HP gauges, the map's move
/// models, the heaps themselves, whose blocks the runtime watches for what they allocate (`app`,
/// ADR-0034), the camera whose heading the D-pad walks by on Lumiose's rails, the process manager
/// that says which app is running, and the button managers, touch bars and the field's menu that
/// say whose buttons take taps (`buttons`), and the AreaNav's navigator, which says whose icon each
/// picture of its Pokémon Distribution draws, and its views, which say which of its layouts it
/// draws; the party and the icons the Bag draws it with; the Pokémon-Amie page, which says whose
/// icon its Pokémon is; the texture systems Amie's decorate screen draws its Poké Puffs and items
/// from, and the layout helpers its item pages point at; its Tile Puzzle's board, which says where
/// each tile's piece belongs; its Berry Picker's Pokémon and opening, which say who wants each
/// berry and whether the round has begun; Super Training's page, which says whose each party slot's
/// is; and the balls of its Balloon Bots, which say when a shot is coming; the Battle Maison's
/// partner screen, which says whose portrait each picture draws; and the save's trainer record, which says
/// the game's language (`mystatus`). A game's own readers add what
/// they find their objects by (`Gen6Profile.classes`): ORAS's DexNav, the player's work its hidden Pokémon's record is found
/// through.
///
/// Not TextBoxes or cursor windows: walking each layout's tree finds every one that can be placed on
/// screen (spike 62). And the linear heap is not swept: the battle builds its layouts there, and its
/// message window leads to their pool (spike 64).
pub const HEAP_CLASSES: [&str; 41] = [
    msgwin::CLASSES[0],
    msgwin::CLASSES[1],
    talk::CLASSES[0],
    CONTROLLER_CLASS,
    DEVICE_MANAGER_CLASS,
    lyt::DRAWINFO_CLASS,
    strbuf::CLASS,
    GAUGE_CLASSES[0],
    GAUGE_CLASSES[1],
    MOVE_MODEL_CLASS,
    gflheap::CLASS,
    mapblock::CLASS,
    camera::CLASS,
    MANAGER_CLASS,
    distribution::CLASS,
    // `*townmap.VIEWS`: the dict's keys, in its order.
    townmap::VIEWS[0].0,
    townmap::VIEWS[1].0,
    townmap::VIEWS[2].0,
    townmap::VIEWS[3].0,
    townmap::VIEWS[4].0,
    amie::CLASS,
    distribution::REPLACE_CLASS,
    amie::G2D_CLASS,
    puzzle::CLASS,
    puzzle::PICTURE,
    puzzle::BACKGROUND,
    berry_picker::MODEL,
    berry_picker::DEMO,
    training::CLASS,
    training::MISSILE,
    partners::CLASS,
    pokemon::PARAM_CLASS,
    partyicons::ICON_CLASS,
    partyicons::PARTY_SAVE,
    partyicons::BATTLE_BOX,
    boxes::BOXES,
    boxes::NAMES,
    buttons::MANAGER_CLASS,
    buttons::BAR_CLASS,
    buttons::MENU_CLASS,
    mystatus::CLASS,
];

/// Without a message window class there is nothing to read: not this game, or not this version.
pub const REQUIRED: [&str; 2] = msgwin::CLASSES;
/// Where the tone of each quarter up walks sounds, across and down as `Tones.play` has them,
/// north first and clockwise: north high, south low, east in the right ear and west in the left, as
/// the Hoenn map's cursor sounds its place (`core.tones`).
pub const FACING_TONES: [(f64, f64); 4] = [(0.5, 0.0), (1.0, 0.5), (0.5, 1.0), (0.0, 0.5)];
/// Seconds after the player arrives in a zone, or the field runs again after another app, that a
/// new facing is taken without a tone: as a zone loads or the Bag opens over the field, the camera
/// looks south for a poll or two in ORAS and X alike, and over the tapes 44 of 133 tones came
/// within 1.5 s of a zone load or an app starting, the compass turned south and back within 0.8 s
/// (spike 251).
pub const ARRIVAL_QUIET: f64 = 1.5;

/// `Gen6Profile.__init__`'s arguments after `memory`, `names`, `speech`, `log` and the keyword
/// `game`, each as the Python defaults it: set what differs with `..Options::default()`.
#[derive(Default)]
pub struct Options {
    /// None is `Names()`.
    pub named: Option<Rc<Names>>,
    /// None is `Ui()`.
    pub ui: Option<Ui>,
    pub navigation: Option<Navigation>,
    /// None is `Extras()`.
    pub extras: Option<Extras>,
    /// None is `DEFAULT_BINDINGS`.
    pub keys: Option<Bindings>,
    pub text: Rc<GameText>,
    pub pictures: Option<Rc<Index>>,
    /// The executable as read from its code, `b""` when there is none.
    pub image: Vec<u8>,
    pub tap: Option<Tap>,
    pub slide: Option<touch::Slide>,
    pub drag: Option<Rc<dyn Drag>>,
    pub tones: Option<Rc<Tones>>,
    pub ticks: Option<Rc<Tones>>,
    pub clips: Option<Rc<Clips>>,
    /// None is `{}`.
    pub swims: Option<IndexMap<String, f64>>,
    pub title: Option<i64>,
    pub census: bool,
}

/// The gen 6 readers, and what each sweep and poll does with them, for ORAS or for X and Y.
///
/// A reader that Python mutates in place sits in a `RefCell`, and a reader that several others hold
/// is an `Rc`; none is borrowed across a call into another reader or into the discovery, which the
/// runtime's `between` can re-enter the profile from (a sweep's `between` polls dialogue and
/// answers keys).
pub struct Gen6Profile {
    memory: Rc<dyn Cache>,
    names: Rc<ClassNames>,
    /// Touch mode's targets as last logged, or None where they are not (`census`).
    census: RefCell<Option<String>>,
    /// The game's own words out of the player's ROM, or `NO_TEXT` when it could not be read.
    text: Rc<GameText>,
    /// The place-name index of the zone the player was last polled in.
    place: Cell<Option<i64>>,
    pub layouts: Rc<Layouts>,
    /// Which screen the game is running, and the moment another starts (spike 99).
    ///
    /// Shared with the own apps' `running`, which the Python hands as a bound method.
    pub apps: Rc<RefCell<RunningApp>>,
    pub dialogue: RefCell<DialogueReader>,
    /// Shared with touch mode's `lists`, which the Python hands as a bound method.
    pub lists: Rc<RefCell<ListReader>>,
    pub choices: RefCell<ChoiceReader>,
    /// Shared with touch mode's `on_screen` and `text_of`, which the Python hands as bound
    /// methods.
    pub screen: Rc<RefCell<ScreenTextReader>>,
    /// Layout files name the textures a screen's pictures use, which is all the battle's
    /// command menu has to say what its buttons are (spike 50). They live in both heaps.
    pub files: Rc<LayoutFiles>,
    pub cursors: RefCell<CursorReader>,
    /// The battle's name plates: the StrBufs whose text ends in a gender sign, which exist only
    /// while a battle is up (spike 57). Its gauges say which Pokémon each HP panel shows.
    pub battlers: Rc<Battlers>,
    pub gauges: Rc<Gauges>,
    pub battle: RefCell<BattleReader>,
    /// The field says which model is showing a message; what to call it is pa3ds's own, since
    /// the game keeps no name for anyone on its maps (spikes 73 to 77).
    pub speaker: Rc<SpeakerReader>,
    /// The map: the player's cell, and the zone data and models around it (spike 78), listed
    /// in the log as zones are entered and walked through by the navigation keys.
    ///
    /// Python's attribute is assigned to (a test puts its own reader in); the navigator keeps the
    /// one it was made with, as it does there, and what is read through the profile is read
    /// through whichever this holds (`field`, `set_field`).
    field: Rc<RefCell<Rc<FieldReader>>>,
    /// The Pokémon its save keeps for its credits, which their descriptions name (spike 222).
    pub memories: Option<Rc<dyn MemoriesReader>>,
    /// What the heap is swept for in this game: `HEAP_CLASSES`, and what the game's own
    /// readers find their objects by.
    pub classes: Vec<String>,
    pub distribution: Rc<RefCell<Distribution>>,
    pub views: Rc<RefCell<townmap::Views>>,
    pub amie: Rc<RefCell<AmiePage>>,
    pub puffs: Rc<RefCell<Puffs>>,
    pub goods: Rc<RefCell<Goods>>,
    pub puzzle: Rc<RefCell<TilePuzzle>>,
    pub training: Rc<RefCell<TrainingPage>>,
    pub goals: Rc<Goals>,
    pub waiting: Rc<Waiting>,
    pub bags: Rc<Bags>,
    pub berry_picker: Rc<RefCell<berry_picker::BerryPicker>>,
    pub sparring: RefCell<Round>,
    pub buttons: Rc<RefCell<buttons::Buttons>>,
    pub party_icons: Rc<RefCell<PartyIcons>>,
    pub partners: Rc<RefCell<Partners>>,
    pub occupants: Rc<RefCell<boxes::Occupants>>,
    pub standings: Option<Rc<RefCell<dyn StandingsReader>>>,
    pub map_cursor: RefCell<MapCursor>,
    pub footsteps: Option<RefCell<Footsteps>>,
    /// Python's `self.touch` is the navigator's `touch`, which it owns here.
    pub navigator: RefCell<Navigator>,
    log: Log,
    speech: Rc<Speech>,
    /// The Dowsing Machine's lights, in a game where they have been measured.
    pub dowsing: Option<RefCell<Lights>>,
    /// The zone last listed and what was in it (people by kind alone, since they walk), to
    /// list again only when something comes or goes.
    listed: RefCell<Option<(i64, Vec<(String, String, Option<(i64, i64)>)>)>>,
    /// Whether a sweep has been taken in since the last listing.
    relist: Cell<bool>,
    /// The chunks the last listing was narrowed by, as logged.
    chunks: RefCell<Vec<(i64, i64)>>,
    /// The hidden Pokémon last seen, species and cell, to list again when one comes or goes
    /// between sweeps.
    hiding: Cell<Option<(i64, (i64, i64))>>,
    /// The record's raw words as last logged, so a change is logged whether or not it
    /// passes as a Pokémon: the user heard a rustle the list did not have (2026-09-22).
    hidden_words: RefCell<Option<String>>,
    /// Whether the player was riding the bike at the last field poll, to log each change.
    on_bike: Cell<bool>,
    /// The quarters from north up walks the player, as the compass was last turned.
    turn: Cell<i64>,
    /// The quarter of the world up walks the player, as its tone last said (`facing`).
    facing: Cell<Option<i64>>,
    /// Python's `tones.play`, kept for the facing's tone.
    tone: Option<Rc<Tones>>,
    /// When the facing may next be toned, `ARRIVAL_QUIET` after the player arrives in a
    /// zone or the field last ran under another app.
    quiet: Cell<f64>,
    /// The language the save says the game is in, as the last whole sweep read it.
    pub save_language: RefCell<Option<String>>,
    pub announcements: Rc<RefCell<Announcements>>,
    pub screen_announcer: RefCell<ScreenAnnouncer>,
    /// The scenes the game shows without words, described where the player asks and the game has
    /// descriptions in their language (ADR-0038).
    pub scenes: Option<RefCell<Scenes>>,
    /// Tile Puzzle's picture is described as a scene is, by the narrator (spike 256).
    describing: bool,
    /// The Pokédex's habitat map draws the Pokémon's places and never names them (spike 152).
    pub habitat: RefCell<HabitatReader>,
    /// And the AreaNav's Pokémon Distribution draws a place's Pokémon and names none.
    pub distribution_reader: RefCell<DistributionReader>,
    /// An Extra: the conditions the game draws as a star, said as numbers (ADR-0017).
    pub conditions: Option<RefCell<Box<dyn ConditionsReader>>>,
    /// X's opening draws its boy or girl and their looks as models, with no word (spike 216).
    pub opening: Option<RefCell<Box<dyn OpeningReader>>>,
    /// (work, root) of every layout whose files have been looked for since an app started.
    archived: RefCell<HashSet<(i64, i64)>>,
    /// The zone the player was last polled in.
    zone: Cell<Option<i64>>,
    /// Message windows the field has shown a page in that no talk object kept was showing,
    /// swept for once each, and again after the player enters another zone.
    unheard: RefCell<HashSet<i64>>,
    /// Whether a sweep is wanted for the talk object showing a page.
    speaker_wanted: Cell<bool>,
    /// Every device manager a sweep has found. It lives all session, so a choice box built
    /// later is found in the blocks just allocated by the manager it holds. Iterated by
    /// `Found.holders`, whose result follows its order, so a `PySet`. Shared with the scenes'
    /// clock, which reads the game's frames through them (`frames`).
    managers: Rc<RefCell<PySet>>,
    /// The cell the Secret Base's decorating cursor was last said on, and what was there.
    decorating: RefCell<Option<((i64, i64), String)>>,
    /// The game's own apps, ORAS's soaring: while one runs, the keys walk what it lists, which
    /// the field's stale lists know nothing of (spike 166).
    own: Rc<Vec<Rc<dyn OwnApp>>>,
}

/// `FieldReader` is what a `Navigator` reads the map through in the gen 6 games.
impl FieldSource for FieldReader {
    fn visit(&self) -> Option<(i64, i64)> {
        FieldReader::visit(self)
    }

    fn poll(&self) -> Result<Option<Field>> {
        FieldReader::poll(self)
    }

    fn interactables(&self, r#where: &Field) -> Result<Field> {
        FieldReader::interactables(self, r#where)
    }

    fn ground(&self, r#where: &Field) -> Result<Ground> {
        FieldReader::ground(self, r#where)
    }
}

impl Gen6Profile {
    pub fn new(
        memory: Rc<dyn Cache>,
        names: Rc<ClassNames>,
        speech: Rc<Speech>,
        log: Log,
        game: &Game,
        options: Options,
    ) -> Gen6Profile {
        let Options {
            named,
            ui,
            navigation,
            extras,
            keys,
            text,
            pictures,
            image,
            tap,
            slide,
            drag,
            tones,
            ticks,
            clips,
            swims,
            title,
            census,
        } = options;
        let tone = tones.clone();
        let cache = memory;
        let memory: Rc<dyn Memory> = cache.clone();
        let named = named.unwrap_or_else(|| Rc::new(Names::empty()));
        let layouts = Rc::new(Layouts::new(memory.clone(), names.clone(), lyt::ORAS));
        let apps = Rc::new(RefCell::new(RunningApp::new(memory.clone(), names.clone())));
        let dialogue = DialogueReader::new(memory.clone(), names.clone(), Some(layouts.clone()));
        let lists = Rc::new(RefCell::new(ListReader::new(memory.clone(), names.clone())));
        let choices = ChoiceReader::new(memory.clone(), names.clone());
        // A picture whose texture the game swaps as it runs is named by the image it draws: the
        // Pokédex's language tabs and its type (spike 147).
        let named_by_image: Option<Pictures> = pictures.map(|pictures| {
            let memory = memory.clone();
            Rc::new(move |pane: i64| pictures.said(&*memory, pane)) as Pictures
        });
        let species: Vec<String> = text.species.clone();
        // The Bag draws each party member's icon, and says nothing of whose it is. Made here,
        // before the screen text reader that asks it, where Python makes it later and reaches it
        // through `self`; making it reads nothing.
        let party_icons = Rc::new(RefCell::new(PartyIcons::new(
            memory.clone(),
            names.clone(),
            species.clone(),
            {
                let apps = apps.clone();
                Rc::new(move || apps.borrow().running.clone())
            },
        )));
        // The Battle Maison's partners are portraits over names drawn with no TextBox. Made here,
        // before the screen text reader that asks it, where Python makes it later and reaches it
        // through `self`; making it reads nothing.
        let partners = Rc::new(RefCell::new(Partners::new(memory.clone())));
        // The AreaNav's Pokémon Distribution draws each Pokémon's icon, which no file names. Made
        // here, before the screen text reader that asks it, where Python makes it later and
        // reaches it through `self`; making it reads nothing.
        let distribution = Rc::new(RefCell::new(Distribution::new(
            memory.clone(),
            names.clone(),
            species,
        )));
        // The Bag's party draws each member as an icon beside its level, or, for an evolution
        // stone, beside "ABLE!" or "UNABLE!": "Metagross UNABLE!" (movie31). The AreaNav's
        // Pokémon Distribution draws the place's Pokémon as icons whose buttons take no tap, so
        // they are named here for its reader to say whole (or_area_nav, spike 232).
        // Made here, before the screen text reader, which names pictures by it: Python's closure
        // reaches `self.files` once it is made, later. Making either reads nothing.
        let files = Rc::new(LayoutFiles::new(memory.clone()));
        let image_of: ImageOf = {
            let (memory, files) = (memory.clone(), files.clone());
            Rc::new(move |pane: i64| {
                let at = textures::drawn_at(&*memory, pane)?;
                Ok(at.and_then(|at| files.image_at(at)))
            })
        };
        // Pokémon-Amie's games draw a result's count a digit a picture, `0.bclim` to `9.bclim`, too
        // large for the index (or_amie_stars): named by their files.
        let on_screen_named: Pictures = {
            let (named_by_image, party_icons) = (named_by_image.clone(), party_icons.clone());
            let image_of = image_of.clone();
            let (apps, distribution, partners) =
                (apps.clone(), distribution.clone(), partners.clone());
            Rc::new(move |pane: i64| {
                let by_image = match &named_by_image {
                    Some(named) => named(pane)?,
                    None => None,
                };
                // Python's `or`: a name of "" or None goes on to the party's icons, then to the
                // Distribution's, and then to the Maison's partners.
                if let Some(name) = by_image
                    && !name.is_empty()
                {
                    return Ok(Some(name));
                }
                let running = apps.borrow().running.clone();
                if running
                    .as_deref()
                    .is_some_and(|app| amie::GAMES.contains(&app))
                    && let Some(digit) = amie::digit(image_of(pane)?.as_deref())
                {
                    return Ok(Some(digit));
                }
                if let Some(name) = party_icons.borrow().name(pane)?
                    && !name.is_empty()
                {
                    return Ok(Some(name));
                }
                if apps.borrow().running.as_deref() == Some(townmap::APP)
                    && let Some(name) = distribution.borrow().name(pane)?
                    && !name.is_empty()
                {
                    return Ok(Some(name));
                }
                partners.borrow().name(pane)
            })
        };
        let screen = Rc::new(RefCell::new(ScreenTextReader::with(
            memory.clone(),
            names.clone(),
            layouts.clone(),
            msgwin::CLASSES
                .iter()
                .map(|class| class.to_string())
                .collect(),
            None,
            Some(on_screen_named),
            text.labels.clone(),
            text.plain.clone(),
        )));
        let battlers = Rc::new(Battlers::new(memory.clone(), names.clone()));
        let gauges = Rc::new(Gauges::new(memory.clone(), names.clone()));
        let battle = BattleReader::new(layouts.clone(), battlers.clone(), gauges.clone());
        let speaker = Rc::new(SpeakerReader::new(
            memory.clone(),
            named.clone(),
            game.drawn,
            game.models,
        ));
        // Each game's own readers come with it: ORAS's DexNav, its Secret Bases, whose every spot
        // is listed in the executable (spike 154), Route 113's ash piles, and Fortree and
        // Sootopolis Gyms' doors and ice, kept by the Gyms' own modules (spikes 155, 161, 167).
        // An Extra: the hidden items, under the sea or everywhere (ADR-0017, spike 249).
        let field = FieldReader::new(
            memory.clone(),
            game,
            &text,
            &image,
            Rc::new(py::time::monotonic),
            Some(&named.words(TILES)),
            Some(&named.words(FURNITURE)),
            &extras.clone().unwrap_or_default().hidden_items,
        );
        // The Pokémon its save keeps for its credits, which their descriptions name (spike 222).
        let memories = game.memories.map(|make| make(memory.clone()));
        let classes: IndexSet<String> = HEAP_CLASSES
            .iter()
            .copied()
            .chain(field.classes())
            .map(String::from)
            .chain(memories.iter().flat_map(|memories| memories.classes()))
            .collect();
        // The navigator says what it finds by the name it has been given, and renames it on a
        // key: the player standing in front of a sign is the only one who can name it.
        let ui = ui.unwrap_or_default();
        let announcements = Rc::new(RefCell::new(Announcements::with(
            speech.clone(),
            log.clone(),
            Some(name_for(&speaker)),
            false,
        )));
        let field = Rc::new(RefCell::new(field));
        // The game's own apps, ORAS's soaring: while one runs, the keys walk what it lists, which
        // the field's stale lists know nothing of (spike 166).
        let mut owned: Vec<Rc<dyn OwnApp>> = Vec::new();
        for make in &game.apps {
            // Python raises out of the profile's making, which ends the runtime.
            match make(memory.clone(), &text, speech.clone(), log.clone()) {
                Ok(Some(app)) => owned.push(app),
                Ok(None) => {}
                Err(error) => panic!("{error}"),
            }
        }
        let owned = Rc::new(owned);

        // Touch mode walks the bottom screen's touch areas, and taps one on the player's key
        // (`games.common.touch`, ADR-0009 as amended 2026-10-01).

        let file_of: FileOf = {
            let (files, layouts) = (files.clone(), layouts.clone());
            Rc::new(move |root: i64| -> Option<Rc<LayoutFile>> {
                files.matching(&layouts.pane_names(root))
            })
        };

        // The AreaNav keeps the layouts of a screen it has left as they were; its views say.
        let views = Rc::new(RefCell::new(townmap::Views::new(
            memory.clone(),
            names.clone(),
        )));
        // The game's button managers say whose buttons take taps (spike 189).
        let buttons = Rc::new(RefCell::new(buttons::Buttons::with_menu(
            memory.clone(),
            game.field_menu,
        )));
        // ORAS's contests' appeals draw who stands where as portraits, said on the keys that say
        // a battle's sides (Orion, 2026-10-09).
        let standings: Option<Rc<RefCell<dyn StandingsReader>>> = game.standings.map(|make| {
            make(memory.clone(), layouts.clone(), files.clone(), &text, {
                let apps = apps.clone();
                Rc::new(move || apps.borrow().running.clone())
            })
        });
        let pauses = ui.pauses;
        let sides: Sides = {
            let (standings, announcements) = (standings.clone(), announcements.clone());
            Rc::new(move |player: bool| {
                let standing = match &standings {
                    None => None,
                    Some(standings) => standings.borrow().said(player, pauses)?,
                };
                // Python's `or`: a standing of "" or None goes on to the battle's sides.
                match standing {
                    Some(standing) if !standing.is_empty() => Ok(Some(standing)),
                    _ => Ok(announcements.borrow().sides(player)),
                }
            })
        };
        let on_screen: OnScreen = {
            let screen = screen.clone();
            Rc::new(move || screen.borrow().on_screen())
        };
        let text_of: TextOf = {
            let screen = screen.clone();
            Rc::new(move |r#box: i64| screen.borrow().text_of(r#box))
        };
        // The PlayNav's Pokémon-Amie page draws its Pokémon's icon, and says nothing of whose.
        let amie = Rc::new(RefCell::new(AmiePage::with_number(
            memory.clone(),
            names.clone(),
            text.species.clone(),
            game.amie,
        )));
        // Amie's decorate screen draws its Poké Puffs, and says nothing of which.
        let puffs = Rc::new(RefCell::new(Puffs::new(
            memory.clone(),
            names.clone(),
            text.puffs.clone(),
        )));
        // And its Interior Items, of which it says only how many.
        let goods = Rc::new(RefCell::new(Goods::new(
            memory.clone(),
            names.clone(),
            text.goods.clone(),
        )));
        // Its Tile Puzzle draws each tile's piece in 3D, and says nothing of where it belongs.
        let puzzle = Rc::new(RefCell::new(TilePuzzle::new(
            memory.clone(),
            layouts.clone(),
        )));
        // So does Super Training's, with the party; and its rounds' goals are on the top screen.
        let training = Rc::new(RefCell::new(TrainingPage::with_icons(
            memory.clone(),
            names.clone(),
            text.species.clone(),
            game.training,
        )));
        let goals = Rc::new(Goals::new(layouts.clone(), image_of.clone()));
        // A message on the bottom screen waits for a tap on no button: Super Training's welcome.
        let waiting = Rc::new(Waiting::new(layouts.clone(), image_of.clone()));
        // The DexNav draws a Pokémon caught in colour, and only those are said so: "Ralts",
        // "Magikarp fishing caught", with commas under pauses (ADR-0037, `Distribution.said`).
        let gap = if ui.pauses { ", " } else { " " };
        let caught = format!("{gap}{}", own(CAUGHT, &[]));
        let mut images: IndexMap<String, String> = text
            .touch_words
            .iter()
            .map(|(image, words)| {
                let said = if text.caught.contains(image) {
                    format!("{words}{caught}")
                } else {
                    words.clone()
                };
                (image.clone(), said)
            })
            .collect();
        // Pictures of words the game's text has no line for are named by pa3ds's own (X's Amie
        // menu), and keep to them as a button with the game's word does.
        for image in &game.worded {
            if let Some(said) = named_picture(image)
                && !said.is_empty()
            {
                images.insert(image.clone(), said);
            }
        }

        // Carrying a Pokémon in the PC Box, the slot it is over says who is in it, from the save.
        // Made before the block the cursors are made in, where Python makes it inside, just before
        // them; making it reads nothing.
        let occupants = Rc::new(RefCell::new(boxes::Occupants::new(
            memory.clone(),
            names.clone(),
            {
                let party_icons = party_icons.clone();
                Rc::new(move || party_icons.borrow().party())
            },
            {
                let party_icons = party_icons.clone();
                Rc::new(move || party_icons.borrow().battle_box())
            },
        )));
        // Omega Ruby's boy and girl are portraits the game never names, said in pa3ds's words
        // (`pictures.named`).
        // A cursor over pictures alone says what the game calls them, by the image or by the
        // game's word for the texture, as touch mode does: the AreaNav's tabs, Places to
        // Berries, which left and right walk (or_area_nav2); and the Battle Maison's partners by
        // their portraits (or_multi_battle_partners). Made here, after the words it looks
        // pictures up in, where Python makes it before them and reaches them as it runs; in a
        // block of its own, since its `game_word` would hide the module's.
        let cursors = {
            // The game's word for a picture, by the image it draws or its layout file's texture.
            let game_word: Pictures = {
                let (layouts, file_of, image_of) =
                    (layouts.clone(), file_of.clone(), image_of.clone());
                let images = images.clone();
                Rc::new(move |pane: i64| {
                    let placement = layouts.placement(pane)?;
                    let file = placement.and_then(|placement| file_of(placement.root));
                    let raw = layouts.name(pane)?;
                    let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
                    let name = py::decode_ascii(&raw[..end]);
                    let filed: Vec<String> = file
                        .and_then(|file| file.textures.get(&name).cloned())
                        .unwrap_or_default();
                    let mut drawn: Vec<String> = image_of(pane)?.into_iter().collect();
                    drawn.extend(filed);
                    Ok(drawn
                        .iter()
                        .find(|image| !image.is_empty() && images.contains_key(*image))
                        .map(|image| images[image].clone()))
                })
            };
            let partners = partners.clone();
            let cursor_named: Pictures = Rc::new(move |pane: i64| {
                let by_image = match &named_by_image {
                    Some(named) => named(pane)?,
                    None => None,
                };
                // Python's `or`: a name of "" or None goes on to the partners, and then to the
                // game's word.
                if let Some(name) = by_image
                    && !name.is_empty()
                {
                    return Ok(Some(name));
                }
                if let Some(name) = partners.borrow().name(pane)?
                    && !name.is_empty()
                {
                    return Ok(Some(name));
                }
                game_word(pane)
            });
            CursorReader::with_occupant(
                layouts.clone(),
                names.clone(),
                Some(files.clone()),
                false,
                Some(cursor_named),
                Some({
                    let occupants = occupants.clone();
                    Rc::new(move |kind: &str, title: Option<&str>, index: i64| {
                        occupants.borrow().call(kind, title, index)
                    }) as Occupant
                }),
            )
        };
        // And the training bags in its page's list, pictures with no touch area of their own.
        let off: touch::Off = {
            let buttons = buttons.clone();
            Rc::new(move || buttons.borrow().off())
        };
        let bags = Rc::new(Bags::new(
            layouts.clone(),
            image_of.clone(),
            images.clone(),
            Some({
                let buttons = buttons.clone();
                Rc::new(move || buttons.borrow_mut().idle()) as Idle
            }),
            Some(off.clone()),
        ));
        // Amie's Berry Picker draws the berry each Pokémon wants in a bubble, and says nothing of
        // who wants which.
        let berry_picker = Rc::new(RefCell::new(berry_picker::BerryPicker::new(
            memory.clone(),
            names.clone(),
            layouts.clone(),
            image_of.clone(),
            images.clone(),
            {
                let text = text.clone();
                Rc::new(move |found: &pokemon::Pokemon| yours_in(&text, found))
            },
        )));
        // A round says its score, clock and banners, and warns of a Bot's shot.
        let sparring = Round::new(layouts.clone(), image_of.clone(), memory.clone(), None);
        let picture_name: PictureName = {
            let (apps, distribution, amie, training, party_icons) = (
                apps.clone(),
                distribution.clone(),
                amie.clone(),
                training.clone(),
                party_icons.clone(),
            );
            let partners = partners.clone();
            let (puffs, goods) = (puffs.clone(), goods.clone());
            let (puzzle, image_of) = (puzzle.clone(), image_of.clone());
            Rc::new(move |pane: i64| {
                // The AreaNav's navigators are the field's and its own app's: elsewhere they are
                // gone, and asking cost a touch walk 1,400 reads (movie22). On the field its icons
                // are the DexNav's; in the AreaNav, its Pokémon Distribution's (or_area_nav).
                // A name of "" or None goes on to the party's icons.
                let running = apps.borrow().running.clone();
                if running
                    .as_deref()
                    .is_none_or(|running| running == FIELD || running == townmap::APP)
                {
                    let said = if running.as_deref() == Some(FIELD) {
                        distribution.borrow().said(pane, gap)?
                    } else {
                        distribution.borrow().name(pane)?
                    };
                    if let Some(name) = said
                        && !name.is_empty()
                    {
                        return Ok(Some(name));
                    }
                    if let Some(name) = amie.borrow().name(pane)?
                        && !name.is_empty()
                    {
                        return Ok(Some(name));
                    }
                    if let Some(name) = training.borrow().name(pane)?
                        && !name.is_empty()
                    {
                        return Ok(Some(name));
                    }
                    if let Some(name) = partners.borrow().name(pane)?
                        && !name.is_empty()
                    {
                        return Ok(Some(name));
                    }
                }
                if running.as_deref() == Some(amie::ARRANGE)
                    || running.as_deref() == Some(amie::AMIE)
                {
                    if let Some(name) = puffs.borrow_mut().name(pane)?
                        && !name.is_empty()
                    {
                        return Ok(Some(name));
                    }
                    return goods.borrow_mut().name(pane);
                }
                if running.as_deref() == Some(puzzle::APP) {
                    return if image_of(pane)?.as_deref() == Some(puzzle::FRAME) {
                        puzzle.borrow().name(pane)
                    } else {
                        Ok(None)
                    };
                }
                party_icons.borrow().name(pane)
            })
        };
        let lists_labels: Lists = {
            let lists = lists.clone();
            Rc::new(move || lists.borrow().lit())
        };
        let idle: Idle = {
            let buttons = buttons.clone();
            Rc::new(move || buttons.borrow_mut().idle())
        };
        let touch = TouchTargets::new(
            layouts.clone(),
            on_screen,
            text_of,
            image_of.clone(),
            file_of,
            log.clone(),
            touch::Options {
                images: Some(images),
                tap,
                tone: tones.clone().map(|tones| {
                    Rc::new(move |across: f64, down: f64| {
                        // The sound thread not starting, raised to the key's handler in Python.
                        let _ = tones.play(across, down);
                    }) as touch::Tone
                }),
                picture_name: Some(picture_name),
                lists: Some(lists_labels),
                idle: Some(idle),
                off: Some(off),
                slide,
                drag,
                wrap: navigation
                    .as_ref()
                    .is_none_or(|navigation| navigation.touch_wrap),
                next_row: navigation
                    .as_ref()
                    .is_none_or(|navigation| navigation.touch_next_row),
                said_off: Some({
                    let (apps, distribution) = (apps.clone(), distribution.clone());
                    Rc::new(move |pane: i64| {
                        let running = apps.borrow().running.clone();
                        Ok(running.as_deref().is_none_or(|running| running == FIELD)
                            && distribution.borrow().name(pane)?.is_some())
                    })
                }),
                places: Some({
                    let (apps, views) = (apps.clone(), views.clone());
                    let map_places = Rc::new(text.map_places.clone());
                    let none = Rc::new(IndexMap::new());
                    Rc::new(move || {
                        let running = apps.borrow().running.clone();
                        Ok(
                            if running.as_deref() == Some(townmap::APP)
                                && !views.borrow().drawing()?
                            {
                                map_places.clone()
                            } else {
                                none.clone()
                            },
                        )
                    })
                }),
                moved: Some({
                    let (apps, puzzle) = (apps.clone(), puzzle.clone());
                    Rc::new(move |before: &touch::Target, now: &touch::Target| {
                        if apps.borrow().running.as_deref() == Some(puzzle::APP) {
                            puzzle.borrow().moved(before, now)
                        } else {
                            Ok(None)
                        }
                    })
                }),
                gestures: Some({
                    let (amie, goals, waiting, bags, apps) = (
                        amie.clone(),
                        goals.clone(),
                        waiting.clone(),
                        bags.clone(),
                        apps.clone(),
                    );
                    let berry_picker = berry_picker.clone();
                    Rc::new(move || {
                        let running = apps.borrow().running.clone();
                        let mut found = amie.borrow_mut().strokes(running.as_deref())?;
                        found.extend(goals.targets(running.as_deref())?);
                        found.extend(berry_picker.borrow().targets(running.as_deref())?);
                        found.extend(waiting.targets(running.as_deref())?);
                        found.extend(bags.targets(running.as_deref())?);
                        Ok(found)
                    })
                }),
            },
        );
        // The Hoenn map's cursor says nothing between places: a tone says where it is.
        let map_cursor = MapCursor::new(
            layouts.clone(),
            image_of,
            tones.map(|tones| {
                Rc::new(move |start: (f64, f64), end: (f64, f64)| {
                    // The sound thread not starting, raised to the screen's poll in Python.
                    let _ = tones.glide(start, end);
                }) as crate::games::gen6::screens::mapcursor::Tone
            }),
        );
        // A footstep of pa3ds's own where the game plays none, at the game's own moment, and a
        // sound every few cells swum (ADR-0025, spike 227).
        let footsteps = clips.map(|clips| {
            let reader = field.borrow().clone();
            let grounds = clips.names();
            RefCell::new(Footsteps::with_steps(
                memory.clone(),
                Rc::new(move |name: &str| clips.play(name)),
                grounds,
                &swims.unwrap_or_default(),
                Rc::new(move |zone: i64| reader.zones.underwater(zone)),
                game.steps,
                game.form,
            ))
        });
        let navigator = Navigator::new(
            Rc::new(OwnApps {
                field: field.clone(),
                running: {
                    let (apps, owned) = (apps.clone(), owned.clone());
                    Rc::new(move || own_running(&apps, &owned))
                },
                on_field: {
                    let apps = apps.clone();
                    Rc::new(move || matches!(apps.borrow().running.as_deref(), Some(FIELD) | None))
                },
            }),
            speech.clone(),
            log.clone(),
            NavigatorOptions {
                bindings: keys,
                names: Some(named.clone()),
                navigation,
                speaker: Some({
                    let speaker = speaker.clone();
                    Rc::new(move || {
                        speaker
                            .last()
                            .map(|(character, own)| (character, Some(own)))
                    })
                }),
                leads_to: Some(leads_to_of(text.clone(), field.clone())),
                place: Some(place_of(text.clone(), field.clone())),
                street: Some(street_of(text.clone())),
                door_sign: Some(door_sign_of(text.clone())),
                moves: Some(text.moves.clone()),
                bike: text.bike.clone(),
                acro: text.acro.clone(),
                sides: Some(sides),
                touch: Some(touch),
                categories: owned
                    .iter()
                    .flat_map(|app| app.categories())
                    .map(String::from)
                    .collect(),
                ..NavigatorOptions::default()
            },
        );
        let dowsing = game
            .dowsing
            .map(|offset| RefCell::new(Lights::new(memory.clone(), offset)));
        let screen_announcer = ScreenAnnouncer::with_clock(
            speech.clone(),
            log.clone(),
            ui.pauses,
            ticking(ticks),
            Some({
                let apps = apps.clone();
                Rc::new(move || {
                    apps.borrow()
                        .running
                        .as_deref()
                        .is_some_and(|running| amie::GAMES.contains(&running))
                })
            }),
            Some({
                let (apps, puzzle) = (apps.clone(), puzzle.clone());
                Rc::new(move || {
                    Ok(apps.borrow().running.as_deref() == Some(puzzle::APP)
                        && puzzle.borrow().playing()?)
                })
            }),
        );
        // The scenes the game shows without words, described where the player asks and the game
        // has descriptions in their language (ADR-0038), those of the other version left out.
        let managers = Rc::new(RefCell::new(PySet::new()));
        let scenes = if ui.descriptions && text.loaded() {
            // Python raises out of `__init__` here; `new` returns no `Result`, and nothing the
            // file or the ROM holds raises in `load` but a line number in digits no int reads.
            let described = Rc::new(
                Described::new(
                    line_of(text.clone()),
                    word_of(text.clone()),
                    Some(recall(named.clone(), memories.clone(), text.clone())),
                )
                .unwrap_or_else(|error| panic!("{error:?}")),
            );
            let known = {
                let described = described.clone();
                move |message: &str| described.known(message)
            };
            let loaded = load_scenes(
                scenes_shipped(&(game.data)()),
                &*line_of(text.clone()),
                &known,
                &log,
                title,
            )
            .unwrap_or_else(|error| panic!("{error:?}"));
            let clock = GameClock::with_monotonic(frames_of(memory.clone(), managers.clone()));
            Some(RefCell::new(Scenes::new(
                loaded,
                Rc::new(move |message: &str| described.said(message)),
                Rc::new(move || clock.call()),
            )))
        } else {
            None
        };
        // Tile Puzzle's picture is described as a scene is, by the narrator (spike 256).
        let describing = ui.descriptions && text.loaded();
        let habitat = HabitatReader::new(
            memory.clone(),
            layouts.clone(),
            files.clone(),
            text.map_places.clone(),
            speech.clone(),
            log.clone(),
            ui.pauses,
        );
        // And the AreaNav's Pokémon Distribution draws a place's Pokémon and names none.
        let distribution_reader = DistributionReader::new(distribution.clone(), ui.pauses);
        // An Extra: the conditions the game draws as a star, said as numbers (ADR-0017).
        let conditions = match game.conditions {
            Some(make) if extras.unwrap_or_default().conditions && text.loaded() => {
                Some(RefCell::new(make(
                    memory.clone(),
                    names.clone(),
                    layouts.clone(),
                    files.clone(),
                    apps.clone(),
                    &text,
                    speech.clone(),
                    log.clone(),
                    ui.pauses,
                )))
            }
            _ => None,
        };
        // X's opening draws its boy or girl and their looks as models, with no word (spike 216).
        let opening = game
            .opening
            .map(|make| RefCell::new(make(memory.clone(), &text.language)));
        // The rival is a work object in RAM and a name in the ROM, and so are some doors' buildings:
        // the game's own word, behind whatever the player has called them and ahead of pa3ds's.
        let sprite: GameSprite = {
            let text = text.clone();
            Rc::new(move |character| game_word(&text, character))
        };
        let place: GamePlace = {
            let text = text.clone();
            Rc::new(move |zone, slot| game_place(&text, zone, slot))
        };
        named.use_game(Some(sprite), Some(place));
        Gen6Profile {
            memory: cache,
            names: names.clone(),
            census: RefCell::new(census.then(String::new)),
            text,
            place: Cell::new(None),
            layouts,
            apps,
            dialogue: RefCell::new(dialogue),
            lists,
            choices: RefCell::new(choices),
            screen,
            files,
            cursors: RefCell::new(cursors),
            battlers,
            gauges,
            battle: RefCell::new(battle),
            speaker,
            field,
            memories,
            classes: classes.into_iter().collect(),
            distribution,
            views,
            amie,
            puffs,
            goods,
            puzzle,
            training,
            goals,
            waiting,
            bags,
            berry_picker,
            sparring: RefCell::new(sparring),
            buttons,
            party_icons,
            partners,
            occupants,
            standings,
            map_cursor: RefCell::new(map_cursor),
            footsteps,
            navigator: RefCell::new(navigator),
            log,
            speech,
            dowsing,
            listed: RefCell::new(None),
            relist: Cell::new(false),
            chunks: RefCell::new(Vec::new()),
            hiding: Cell::new(None),
            hidden_words: RefCell::new(None),
            on_bike: Cell::new(false),
            turn: Cell::new(0),
            facing: Cell::new(None),
            tone,
            quiet: Cell::new(0.0),
            save_language: RefCell::new(None),
            announcements,
            screen_announcer: RefCell::new(screen_announcer),
            scenes,
            describing,
            habitat: RefCell::new(habitat),
            distribution_reader: RefCell::new(distribution_reader),
            conditions,
            opening,
            archived: RefCell::new(HashSet::new()),
            zone: Cell::new(None),
            unheard: RefCell::new(HashSet::new()),
            speaker_wanted: Cell::new(false),
            managers,
            decorating: RefCell::new(None),
            own: owned,
        }
    }

    /// `profile.field`: the reader the profile reads the map through now.
    pub fn field(&self) -> Rc<FieldReader> {
        self.field.borrow().clone()
    }

    /// `profile.field = field`.
    pub fn set_field(&self, field: Rc<FieldReader>) {
        *self.field.borrow_mut() = field;
    }

    /// Hand each reader what a sweep found of its classes.
    ///
    /// A sweep of the blocks the game has just allocated is partial: it says what is new and
    /// nothing of what is gone, so readers add what it found to what they know, and the layouts
    /// are looked over only when it brings new ones (spike 110).
    pub fn track(&self, found: &Found) -> Result<()> {
        let adding = found.partial;
        self.dialogue
            .borrow_mut()
            .track(&found.of(&msgwin::CLASSES), adding);
        self.apps.borrow_mut().track(found.of(&[MANAGER_CLASS]))?;
        // A talk object's class is a DLL's while a talk runs, so every one found is kept.
        self.speaker.track(found.of(&talk::CLASSES));
        self.distribution
            .borrow_mut()
            .track(found.of(&[distribution::CLASS]), adding);
        self.views.borrow_mut().track(&found.heap, adding)?;
        self.amie.borrow_mut().track(&found.heap, adding)?;
        self.puffs.borrow_mut().track(&found.heap, adding)?;
        self.goods.borrow_mut().track(&found.heap, adding)?;
        self.puzzle.borrow_mut().track(&found.heap, adding)?;
        self.berry_picker.borrow_mut().track(&found.heap, adding)?;
        self.training.borrow_mut().track(&found.heap, adding)?;
        self.partners.borrow_mut().track(&found.heap, adding)?;
        self.occupants.borrow_mut().track(&found.heap);
        self.sparring.borrow_mut().track(&found.heap, adding)?;
        self.buttons.borrow_mut().track(
            found.of(&[buttons::MANAGER_CLASS]),
            found.of(&[buttons::BAR_CLASS]),
            found.of(&[buttons::MENU_CLASS]),
            adding,
        )?;
        self.party_icons.borrow_mut().track(
            &found.of(&[pokemon::PARAM_CLASS]),
            found.of(&[partyicons::ICON_CLASS]),
            &|targets: &[i64]| found.holders(targets),
            adding,
            found
                .of(&[partyicons::PARTY_SAVE])
                .into_iter()
                .chain(found.of(&[partyicons::BATTLE_BOX])),
        )?;
        self.lists.borrow_mut().track(found.of(&[CONTROLLER_CLASS]));
        if let Some(memories) = &self.memories {
            memories.track(found);
        }
        if !found.partial
            && let Some(language) = mystatus::language(found)
        {
            *self.save_language.borrow_mut() = Some(language.to_string());
        }
        // A choice box has no class of its own; the device manager it holds does.
        for manager in found.of(&[DEVICE_MANAGER_CLASS]) {
            self.managers.borrow_mut().add(manager);
        }
        let managers: Vec<i64> = self.managers.borrow().iter().collect();
        let holders = found.holders(&managers)?;
        let field = self.field();
        phase(&*self.memory, "tracking", || {
            // A layout's work has no class of its own; the DrawInfo embedded in it does.
            let works: Vec<i64> = found
                .of(&[lyt::DRAWINFO_CLASS])
                .into_iter()
                .map(|at| at - lyt::WORK_DRAWINFO)
                .collect();
            let watched = self.layouts.pools();
            if adding {
                // A screen's pool is made before its layouts are built into it.
                self.layouts.watch_pools_in(&found.pages)?;
            }
            if !works.is_empty() || !adding {
                self.layouts.track(works.iter().copied())?;
            }
            for (pool, capacity) in self.layouts.pools() {
                if !watched.contains_key(&pool) {
                    (self.log)(&format!(
                        "layouts: watching the pool of {capacity} at {pool:#010x}"
                    ));
                }
            }
            if !works.is_empty() || !adding {
                // Every TextBox and cursor window worth reading is under a layout's tree.
                self.screen.borrow_mut().track([]);
                self.cursors
                    .borrow_mut()
                    .track(self.layouts.panes(lyt::WINDOW_CLASS))?;
            }
            self.battlers.track(found.of(&[strbuf::CLASS]), adding)?;
            let [ours, theirs] = GAUGE_CLASSES;
            self.gauges
                .track(found.of(&[ours]), found.of(&[theirs]), adding)?;
            self.choices
                .borrow_mut()
                .track(holders.iter().map(|holder| holder - BOX_DEVICE_MANAGER))?;
            field.track(found)
        })?;
        // Listing the field reads every model in the pool: after a whole sweep, or when the
        // blocks just allocated held the field's own.
        if !adding || !found.of(&[MOVE_MODEL_CLASS, mapblock::CLASS]).is_empty() {
            self.relist.set(true);
        }
        self.read_archives()
    }

    /// Whether a reader needs something only a sweep finds, sooner than the schedule has it.
    pub fn sweep_wanted(&self) -> Result<bool> {
        let battle = self.battle.borrow_mut().sweep_wanted();
        let running = self.apps.borrow().running.clone();
        let field = self
            .field()
            .sweep_wanted(running.as_deref().is_none_or(|running| running == FIELD))?;
        if let Some(field) = field
            && !field.is_empty()
        {
            (self.log)(&format!("field: {field}"));
        }
        let app = self.apps.borrow_mut().sweep_wanted();
        if app {
            (self.log)("app: no process manager, sweeping for it");
        }
        let speaker = self.speaker_wanted.replace(false);
        if speaker {
            (self.log)("speaker: no talk object shows the page, sweeping for it");
        }
        Ok(battle || field.is_some() || speaker || app)
    }

    /// Whether another app has started since the last poll: the field, the Bag, the Pokédex, a
    /// battle. One read a poll, and the moment to sweep for the new screen's objects.
    pub fn poll_app(&self) -> Result<bool> {
        let started = phase(&*self.memory, "app", || self.apps.borrow_mut().poll())?;
        let Some(started) = started else {
            return Ok(false);
        };
        (self.log)(&format!("app: {}", demangle(&started)));
        self.dialogue.borrow_mut().forget_hosts();
        // A screen built again where it was may load its archive elsewhere: the PokéNav's came
        // back from the PlayNav's tutorial in another, and its pictures went unnamed (movie24).
        self.archived.borrow_mut().clear();
        // A battle's sides are said again on a key only while it lasts.
        self.announcements.borrow_mut().forget_sides();
        self.puzzle.borrow().started();
        if started == BATTLE {
            self.battle.borrow_mut().begin();
        }
        if let Some(scenes) = &self.scenes {
            scenes.borrow_mut().started(
                &demangle(&started),
                self.zone.get(),
                &mut self.narrate(),
            )?;
            if DEMOS.contains(&started.as_str()) {
                scenes.borrow_mut().demo(None, &mut self.narrate())?;
            }
        }
        for app in self.own.iter() {
            app.started();
        }
        Ok(true)
    }

    /// Whether the app just started wants the whole heap swept as it builds: not the field
    /// coming back after a battle or a menu, which builds its map blocks again where they were
    /// and keeps the rest; those sweeps found nothing a reader asks for by class, 16 of 65 over
    /// movies 4 and 7 (spike 134).
    pub fn app_sweep_wanted(&self) -> bool {
        let apps = self.apps.borrow();
        !(apps.running.as_deref() == Some(FIELD) && apps.back)
    }

    /// Poll dialogue. Returns whether a page was said.
    pub fn poll_dialogue(&self) -> Result<bool> {
        let shown = phase(&*self.memory, "dialogue", || {
            self.dialogue.borrow_mut().poll()
        })?;
        if let Some(standings) = &self.standings {
            let texts: Vec<&str> = shown
                .iter()
                .filter_map(|page| match page {
                    Shown::Page(page) => Some(page.text.as_str()),
                    Shown::Text(_) => None,
                })
                .collect();
            standings.borrow_mut().heard(&texts);
        }
        let said = self.announcements.borrow_mut().pages(&shown)?;
        if said && self.apps.borrow().running.as_deref() == Some(FIELD) {
            self.look_for_speakers(&shown)?;
        }
        if said && let Some(scenes) = &self.scenes {
            for page in &shown {
                if let Shown::Page(page) = page {
                    scenes.borrow_mut().heard(
                        page.index,
                        &page.text,
                        self.zone.get(),
                        &mut self.narrate(),
                    )?;
                }
            }
        }
        Ok(said)
    }

    /// `self.announcements.narrate`, as the scenes are handed it to say a beat with.
    fn narrate(&self) -> impl FnMut(&str) + '_ {
        move |line: &str| self.announcements.borrow_mut().narrate(line)
    }

    /// Ask for a sweep when the field shows a page no talk object kept is showing. A door
    /// builds the next zone's talk windows into memory taken as the field loaded, a few seconds
    /// on, where no sweep of new blocks looks, and the first pages Mom said in the house had no
    /// name to them (movie1, 2026-09-26). Asked as the talk starts, the sweep finds it for the
    /// pages after; swept for at every door instead, the replays went their own way.
    /// A Pokémon of the player's by the nickname they gave it, or else as the game calls its
    /// species and form (Orion, 2026-10-16, for the credits).
    fn yours(&self, found: &pokemon::Pokemon) -> Option<String> {
        yours_in(&self.text, found)
    }

    fn look_for_speakers(&self, shown: &[Shown]) -> Result<()> {
        for page in shown {
            if let Shown::Page(page) = page
                && !self.unheard.borrow().contains(&page.window)
                && !self.speaker.shows(page.window)?
            {
                self.unheard.borrow_mut().insert(page.window);
                self.speaker_wanted.set(true);
            }
        }
        Ok(())
    }

    /// Poll lists and choice boxes. Returns whether screen text was polled along the way.
    pub fn poll_menus(&self, discovery: &mut Discovery) -> Result<bool> {
        let focuses = phase(&*self.memory, "lists", || self.lists.borrow_mut().poll())?;
        let mut screen_polled = false;
        if focuses.iter().any(|focus| focus.opened) {
            // The question comes before its prompt, but its window can be new and not yet swept:
            // at real speed "YES, 1 of 2" was heard 1.6 s before "So you're wee?". Sweep now and
            // say the question first.
            let swept = discovery.sweep_for(&msgwin::CLASSES)?;
            self.dialogue.borrow_mut().track(&swept, false);
            self.poll_dialogue()?;
            // A question the screen draws itself, such as "Would you like to make these changes?",
            // comes first too, without waiting for it to hold.
            self.poll_screen_with(false, false)?;
            screen_polled = true;
        }
        for focus in &focuses {
            // The save's YES was read by the cursor over it before a sweep found its list, and
            // heard again from the list a second later (movie1, 2026-09-26).
            let heard = focus.opened
                && self
                    .screen_announcer
                    .borrow_mut()
                    .said_by_cursor(&focus.label);
            self.announcements.borrow_mut().focus(focus, "list", heard);
        }
        // A choice box opens only after A on its question, so the question is already said.
        let chosen = phase(&*self.memory, "choices", || {
            self.choices.borrow_mut().poll()
        })?;
        for focus in &chosen {
            // A box that draws a cursor picture beside its words is the cursor reader's from the
            // poll it is shown, and this reader's only once a sweep has found it (spike 98).
            let heard = focus.opened
                && self
                    .screen_announcer
                    .borrow_mut()
                    .said_by_cursor(&focus.label);
            self.announcements
                .borrow_mut()
                .focus(focus, "choice", heard);
        }
        let process = self.apps.borrow().process;
        if let Some(opening) = &self.opening
            && let Some(process) = process
            && self.apps.borrow().running.as_deref() == Some(opening.borrow().app())
        {
            let focus = phase(&*self.memory, "opening", || {
                opening.borrow_mut().poll(process)
            })?;
            if let Some(focus) = focus {
                self.announcements
                    .borrow_mut()
                    .focus(&focus, "opening", false);
            }
        }
        Ok(screen_polled)
    }

    /// `poll_screen()`: `hold` and `with_choices` both True, which is what the runtime asks.
    pub fn poll_screen(&self) -> Result<()> {
        self.poll_screen_with(true, true)
    }

    /// `poll_screen(hold, with_choices)`. Named apart from `poll_screen` because the runtime's
    /// `Profile::poll_screen` takes neither.
    pub fn poll_screen_with(&self, hold: bool, with_choices: bool) -> Result<()> {
        let (screen, sides, listed, cursors, round_lines, wants) =
            phase(&*self.memory, "screen", || {
                // The field's layouts come back where they were when a menu screen closes, and a
                // layout found before it was built is taken in once it is.
                let revived = self.layouts.revive()?;
                let put_away = self.views.borrow().put_away()?;
                self.layouts.put_away(put_away);
                // The battle's layouts are built into a pool its message window leads to.
                let failed: RefCell<Option<Error>> = RefCell::new(None);
                self.layouts.watch_pools(self.linear_handles(&failed))?;
                if let Some(error) = failed.into_inner() {
                    return Err(error);
                }
                if self.layouts.poll_pools()? || revived {
                    self.cursors
                        .borrow_mut()
                        .track(self.layouts.panes(lyt::WINDOW_CLASS))?;
                }
                let exclude: HashSet<i64> = self.dialogue.borrow().line_objects().iter().collect();
                let screen = self.screen.borrow_mut().poll(&exclude, hold)?;
                // A message box's lines lead to a window typing into them before any sweep finds it.
                let lines: Vec<i64> = screen.message_lines.iter().copied().collect();
                self.dialogue.borrow_mut().watch(&lines);
                let on_screen = &screen.on_screen;
                // The HP box is the battle reader's to say, whole and once it holds still.
                let sides = self.battle.borrow_mut().poll(on_screen)?;
                self.habitat.borrow_mut().poll(on_screen)?;
                let in_townmap = self.apps.borrow().running.as_deref() == Some(townmap::APP);
                let listed = self
                    .distribution_reader
                    .borrow_mut()
                    .poll(on_screen, in_townmap)?;
                if let Some(standings) = &self.standings {
                    standings.borrow_mut().poll(on_screen)?;
                }
                self.map_cursor.borrow_mut().poll()?;
                let running = self.apps.borrow().running.clone();
                let round_lines = self.sparring.borrow_mut().poll(running.as_deref())?;
                let wants = self.berry_picker.borrow_mut().poll(running.as_deref())?;
                let cursors = self.cursors.borrow_mut().poll(on_screen)?;
                Ok((screen, sides, listed, cursors, round_lines, wants))
            })?;
        for (line, category) in &round_lines {
            (self.log)(&format!("round: {line}"));
            self.speech.say(line, *category);
        }
        for line in &wants {
            (self.log)(&format!("berry picker: {line}"));
            self.speech.say(line, Category::Screen);
        }
        let mut labels = self.lists.borrow().labels();
        labels.extend(self.announcements.borrow_mut().recent_labels());
        if with_choices {
            labels.extend(self.choices.borrow().labels());
        }
        let mut texts: Vec<ScreenText> = Vec::new();
        for text in &screen.texts {
            if !self.battle.borrow().owns(text)?
                && !self.habitat.borrow().owns(text)
                && !self.distribution_reader.borrow().owns(text)
            {
                texts.push(text.clone());
            }
        }
        // The Distribution's icons are not gathered with the rest of their screen either: a
        // rewrite reads again the groups it has met.
        let heard: Vec<ScreenText> = screen
            .on_screen
            .iter()
            .filter(|piece| !self.distribution_reader.borrow().owns(piece))
            .cloned()
            .collect();
        // A layout built again where it was is the one that was there, and a menu stays a menu:
        // the battle's move list, forgotten while it was gone, was read out whole the moment it
        // showed.
        let mut roots = self.layouts.roots();
        roots.extend(self.layouts.buried());
        let subject = || summary::name(&*self.memory, &self.names, &self.apps.borrow());
        self.screen_announcer.borrow_mut().say(
            &ScreenPoll {
                texts,
                on_screen: heard,
                ..screen.clone()
            },
            &cursors,
            &labels,
            Some(&roots),
            Some(&subject),
        )?;
        // After the screen's words, which name the place first.
        for line in &listed {
            (self.log)(&format!("distribution: {line}"));
            self.speech.say(line, Category::Screen);
        }
        if self.text.loaded() && self.apps.borrow().running.as_deref() == Some(puzzle::APP) {
            let shown = self.puzzle.borrow().new_picture()?;
            let mut background = None;
            let said = match shown {
                Some(shown) => {
                    background = self.puzzle.borrow().background()?;
                    puzzle::described(&shown, &|found| self.yours(found), background)
                }
                None => None,
            };
            if let Some(said) = said {
                // The number too, as a background with no words yet is left out.
                (self.log)(&format!(
                    "puzzle background: {}",
                    background.map_or("None".to_string(), |number| number.to_string())
                ));
                if self.describing {
                    self.announcements.borrow_mut().narrate(&said);
                } else {
                    // Drawn afresh each time the puzzle opens, so a replay's log says which three
                    // it drew, which proves it followed its movie.
                    (self.log)(&format!("puzzle picture: {said}"));
                }
            }
        }
        if let Some(conditions) = &self.conditions {
            // After the screen's words, and once a cursor's details are said, so they come last.
            let under: Vec<String> = cursors
                .under
                .iter()
                .flatten()
                .map(|piece| piece.text.clone())
                .collect();
            let waiting = self.screen_announcer.borrow().gathering();
            conditions
                .borrow_mut()
                .poll(&under, &screen.on_screen, &screen.settling, waiting)?;
        }
        self.announcements.borrow_mut().battlers(&sides);
        self.announcements.borrow_mut().screen_polled();
        self.read_archives()?;
        self.census()
    }

    /// Log touch mode's targets whenever they have changed, where `census` asked for it: a replay
    /// presses no key of pa3ds's, so nothing touch mode offers was on any tape, and the DexNav's
    /// Pokémon not caught went missing with every tape the same (ADR-0041).
    pub fn census(&self) -> Result<()> {
        if self.census.borrow().is_none() {
            return Ok(());
        }
        let listed = phase(&*self.memory, "census", || {
            let mut navigator = self.navigator.borrow_mut();
            let touch = navigator
                .touch
                .as_mut()
                .expect("the gen 6 navigator is given touch targets");
            let names: Vec<String> = touch
                .targets()?
                .into_iter()
                .map(|target| target.name)
                .collect();
            Ok(names.join(" | "))
        })?;
        if self.census.borrow().as_deref() != Some(listed.as_str()) {
            *self.census.borrow_mut() = Some(listed.clone());
            (self.log)(&format!(
                "touch targets: {}",
                if listed.is_empty() { "none" } else { &listed }
            ));
        }
        Ok(())
    }

    /// Answer a navigation key, by name.
    pub fn press(&self, key: &str) -> Result<()> {
        phase(&*self.memory, "keys", || {
            self.navigator.borrow_mut().press_key(key)
        })
    }

    /// Play a footstep where the player's foot came down on silent ground, every tick: the
    /// game's own come a frame of its walk apart, a thirtieth of a second. Three small reads,
    /// and none without a player.
    pub fn poll_footsteps(&self) -> Result<()> {
        let (Some(footsteps), Some(player)) = (&self.footsteps, self.field().player()) else {
            return Ok(());
        };
        phase(&*self.memory, "footsteps", || {
            footsteps.borrow_mut().poll(player)
        })
    }

    /// Follow the player along a path they are walking, every poll while there is one: the
    /// field's own poll is every fifth of a second, and a turn said that late is a step the
    /// player has already taken past it. Nothing is read while there is none.
    pub fn poll_guidance(&self) -> Result<()> {
        if !self.navigator.borrow().guiding() {
            return Ok(());
        }
        phase(&*self.memory, "guidance", || {
            let at = self.field().poll()?;
            if let Some(at) = at {
                self.navigator.borrow_mut().follow(&at)?;
            }
            Ok(())
        })
    }

    /// Sample where each named cursor on screen stands, every tick while there is one, so the
    /// screen poll can tell one at rest from one gliding past (spike 129). One small read each.
    pub fn poll_pointers(&self) -> Result<()> {
        if !self.cursors.borrow().sampling() {
            return Ok(());
        }
        phase(&*self.memory, "pointers", || {
            self.cursors.borrow_mut().sample(&*self.memory)
        })
    }

    /// Keep any name the player has just given. Costs one empty queue read a poll, and the
    /// dialog it answers has been up on a thread of its own while the game kept running.
    pub fn poll_renames(&self) -> Result<()> {
        self.navigator.borrow_mut().poll_renames()
    }

    /// Poll where the player is, and log what is around them when it changes.
    ///
    /// The listing reads every model in the pool, so it is made only on a change of zone and
    /// after each sweep, not every poll.
    pub fn poll_field(&self) -> Result<()> {
        if let Some(scenes) = &self.scenes {
            scenes.borrow_mut().poll(&mut self.narrate())?;
        }
        if let Some((app, process)) = own_running(&self.apps, &self.own) {
            return phase(&*self.memory, app.name(), || app.poll(process));
        }
        let field = self.field();
        // `None` is the Python's `return` from inside the `with`.
        let polled = phase(&*self.memory, "field", || -> Result<Option<Field>> {
            self.poll_dowsing()?;
            let Some(mut at) = field.poll()? else {
                return Ok(None);
            };
            let arrived = Some(at.zone) != self.zone.get();
            if arrived {
                self.zone.set(Some(at.zone));
                self.unheard.borrow_mut().clear();
            }
            if at.on_bike != self.on_bike.get() {
                self.on_bike.set(at.on_bike);
                (self.log)(&format!(
                    "field: {} the bike",
                    if at.on_bike { "on" } else { "off" }
                ));
            }
            // Off the rails the compass turns with the camera. On Lumiose's rails a way is said
            // from the camera already, and a cell is the corridor's, not the world's.
            let turn = field.turn(self.turn.get())?;
            if turn != self.turn.get() {
                self.turn.set(turn);
                walked(turn);
                (self.log)(&format!("field: up walks {}", COMPASS[turn as usize]));
            }
            // A tone as the way up walks turns a quarter, where nothing says it has: round Prism
            // Tower and down Lumiose's streets the player cannot see the camera turn (spike 251).
            if arrived || self.apps.borrow().running.as_deref() != Some(FIELD) {
                self.quiet.set(py::time::monotonic() + ARRIVAL_QUIET);
            }
            let facing = field.facing(self.facing.get())?;
            if facing != self.facing.get() {
                if let (Some(_), Some(now)) = (self.facing.get(), facing)
                    && py::time::monotonic() >= self.quiet.get()
                {
                    (self.log)(&format!("field: tone for facing {}", COMPASS[now as usize]));
                    if let Some(tones) = &self.tone {
                        let (across, down) = FACING_TONES[now as usize];
                        // The sound thread not starting, raised to the field's poll in Python.
                        let _ = tones.play(across, down);
                    }
                }
                self.facing.set(facing);
            }
            self.navigator.borrow_mut().swept(&at)?;
            self.arrive(at.zone)?;
            // After the place's name, which a scene across Hoenn's towns follows town by town.
            if arrived && let Some(scenes) = &self.scenes {
                scenes.borrow_mut().arrived(at.zone, &mut self.narrate())?;
            }
            if let Some(scenes) = &self.scenes {
                let wanted = scenes.borrow().triggers(at.zone);
                if !wanted.is_empty()
                    && let Some(trigger) = field.trigger_at(at.zone, at.cell, &wanted)?
                {
                    scenes
                        .borrow_mut()
                        .stepped(at.zone, trigger, &mut self.narrate())?;
                }
            }
            self.poll_decorating(&at)?;
            // A hidden Pokémon shows and goes between sweeps, so its record is read every poll.
            let hiding = field.hidden(at.zone)?;
            let key = hiding.as_ref().map(|hiding| (hiding.code, hiding.cell));
            let words = field.hidden_words()?;
            if words != *self.hidden_words.borrow() {
                (self.log)(&format!(
                    "field hidden record: {}",
                    words.as_deref().unwrap_or("None")
                ));
                *self.hidden_words.borrow_mut() = words;
            }
            // Found by a search, or by the detector with the DexNav open: never one it turns up
            // while the AreaNav is, "I'm looking for the youngster there, not a random Poochyena!"
            // (Orion, 2026-10-06 and 07). The navigator knows whether touch mode is on.
            let found = field.hiding();
            let appeared = hiding.is_some()
                && self.hiding.get().is_none()
                && found
                    .as_ref()
                    .is_some_and(|found| found.searched || found.on_dexnav);
            if key != self.hiding.get() {
                self.hiding.set(key);
                self.relist.set(true);
            }
            if !self.relist.get()
                && let Some(listed) = &*self.listed.borrow()
                && listed.0 == at.zone
            {
                return Ok(None);
            }
            at = field.interactables(&at)?;
            if appeared
                && let Some(hiding) = &hiding
                && let Some(found) = &found
            {
                self.navigator
                    .borrow_mut()
                    .found(&at, hiding, found.searched)?;
            }
            // Which chunks the edges were narrowed by, for the log: the blocks come and go
            // with the map, and a border listed whole is a border no chunk covers.
            let mut chunks: Vec<(i64, i64)> = field
                .chunks_loaded()?
                .iter()
                .map(|chunk| chunk.origin)
                .collect();
            chunks.sort();
            if chunks != *self.chunks.borrow() {
                let shown = if chunks.is_empty() {
                    "none".to_string()
                } else {
                    format!(
                        "[{}]",
                        chunks
                            .iter()
                            .map(|(x, z)| format!("({x}, {z})"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                *self.chunks.borrow_mut() = chunks;
                (self.log)(&format!("field chunks: {shown}"));
            }
            Ok(Some(at))
        })?;
        let Some(at) = polled else {
            return Ok(());
        };
        self.relist.set(false);
        let mut items: Vec<(String, String, Option<(i64, i64)>)> = at
            .interactables
            .iter()
            .map(|item| {
                (
                    item.category.clone(),
                    item.kind.clone(),
                    if item.category == "people" {
                        None
                    } else {
                        Some(item.cell)
                    },
                )
            })
            .collect();
        items.sort();
        let listed = (at.zone, items);
        if *self.listed.borrow() == Some(listed.clone()) {
            return Ok(());
        }
        *self.listed.borrow_mut() = Some(listed);
        (self.log)(&format!(
            "field: zone {:#x} at ({}, {})",
            at.zone, at.cell.0, at.cell.1
        ));
        let zone = at.zone;
        let called = |item: &Interactable| -> Result<Option<String>> {
            self.navigator.borrow().called(item, zone)
        };
        let leads = |item: &Interactable| -> Result<Option<String>> {
            self.navigator.borrow().destination(item, zone)
        };
        for line in describe(&at, Some(&called), Some(&leads)) {
            (self.log)(&format!("field {}", line?));
        }
        Ok(())
    }

    /// Say what the Dowsing Machine's lights show as it changes: three small reads a field
    /// poll, and nothing while no field runs.
    fn poll_dowsing(&self) -> Result<()> {
        let (process, running) = {
            let apps = self.apps.borrow();
            (apps.process, apps.running.clone())
        };
        let (Some(dowsing), Some(process)) = (&self.dowsing, process) else {
            return Ok(());
        };
        if running.as_deref() != Some(FIELD) {
            return Ok(());
        }
        let said = dowsing.borrow_mut().poll(process)?;
        if let Some(said) = said {
            (self.log)(&format!("field {said}"));
            self.speech.say(&said, Category::Screen);
        }
        Ok(())
    }

    /// Say what the Secret Base's decorating cursor is on and where, each time it moves or
    /// what is there changes: a desk set down under it is said as it lands.
    fn poll_decorating(&self, r#where: &Field) -> Result<()> {
        let on = self.field().decorating(r#where)?;
        if on == *self.decorating.borrow() {
            return Ok(());
        }
        *self.decorating.borrow_mut() = on.clone();
        if let Some((cell, words)) = on {
            let said = format!("{words}, {}", bearing(r#where.cell, cell));
            (self.log)(&format!("field cursor: {said}"));
            self.speech.say(&said, Category::Menu);
        }
        Ok(())
    }

    /// Say where the player is when the place changes, not the zone: a town and every
    /// building in it share a place, so walking out of a house arrives nowhere, as the game has
    /// it (spike 87). The name comes from RAM's index and the ROM's words, ahead of AreaNav's.
    fn arrive(&self, zone: i64) -> Result<()> {
        let text = &self.text;
        if !text.loaded() {
            return Ok(());
        }
        let index = self.field().zones.place_index(zone)?;
        let Some(index) = index else {
            return Ok(());
        };
        if Some(index) == self.place.get() {
            return Ok(());
        }
        self.place.set(Some(index));
        if let Some(name) = text.place(index) {
            self.screen_announcer.borrow_mut().place(&name);
        }
        Ok(())
    }

    /// The layout handles of the message windows read, where they lead out of the heap.
    ///
    /// Only pools out of the heap are read this way. The heap's layouts wait for the sweep, which
    /// finds them with the objects that own them: a list's layout taken in before its list has
    /// its labels read as screen text (`Layouts.revive`).
    ///
    /// Python's is a generator, so each handle is read as `watch_pools` asks for it, between its
    /// own reads. A read that fails ends the iteration and is left in `failed` for the caller to
    /// raise, where the generator raised through `watch_pools`.
    fn linear_handles<'a>(
        &'a self,
        failed: &'a RefCell<Option<Error>>,
    ) -> impl Iterator<Item = i64> + 'a {
        let mut windows = self.dialogue.borrow().windows().into_iter();
        std::iter::from_fn(move || {
            loop {
                let window = windows.next()?;
                let entry = self
                    .memory
                    .read(window + msgwin::LAYOUT_HANDLE, 4)
                    .and_then(|read| u32(&read, 0));
                match entry {
                    Err(error) => {
                        *failed.borrow_mut() = Some(error);
                        return None;
                    }
                    Ok(entry) if within(LINEAR_SPAN, entry) => return Some(entry),
                    Ok(_) => {}
                }
            }
        })
    }

    /// Find the layout files of every layout tracked since the last look, in its own archive.
    ///
    /// Read outside any phase, since each archive is read once.
    fn read_archives(&self) -> Result<()> {
        let mut walked: HashSet<i64> = HashSet::new();
        for (work, root) in self.layouts.works() {
            if self.archived.borrow().contains(&(work, root)) {
                continue;
            }
            self.archived.borrow_mut().insert((work, root));
            let resource = u32(&self.memory.read(work + lyt::WORK_RESOURCE, 4)?, 0)?;
            if !in_a_heap(resource) || walked.contains(&resource) {
                continue;
            }
            walked.insert(resource);
            let archive = u32(&self.memory.read(resource + lyt::RESOURCE_ARCHIVE, 4)?, 0)?;
            if in_a_heap(archive) {
                self.files.add_archive(archive)?;
            }
            for step in [lyt::RESOURCE_SIZE, -lyt::RESOURCE_SIZE] {
                for count in 1..lyt::MAX_RESOURCES {
                    let beside = resource + step * count;
                    let head = self.memory.read(beside, lyt::RESOURCE_ACCESSOR + 4)?;
                    let accessor = u32(&head, lyt::RESOURCE_ACCESSOR)?;
                    if !in_a_heap(accessor)
                        || self.names.of_object(accessor)?.as_deref() != Some(lyt::ACCESSOR_CLASS)
                    {
                        break;
                    }
                    walked.insert(beside);
                    let archive = u32(&head, lyt::RESOURCE_ARCHIVE)?;
                    if in_a_heap(archive) {
                        self.files.add_archive(archive)?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// `Gen6Profile._own_running`: the game's own app running now, and its process, or None.
fn own_running(
    apps: &RefCell<RunningApp>,
    own: &[Rc<dyn OwnApp>],
) -> Option<(Rc<dyn OwnApp>, i64)> {
    let apps = apps.borrow();
    let process = apps.process;
    let app = own
        .iter()
        .find(|app| apps.running.as_deref() == Some(app.app()))?;
    Some((app.clone(), process?))
}

/// What the navigator reads the map through: what one of the game's own apps lists while it
/// runs (`OwnApp.listed`), and the field's while the field runs or its map is still loaded under
/// another screen, where the player cannot walk (`Field.walkable`). A battle or a scene unloads
/// the map and runs with the field's models still in memory as they were: after Groudon's
/// capture the keys listed it in the cave left behind (Orion, 2026-10-12). A map loaded is not
/// enough alone: arriving in a new place, the field runs a second or so before a sweep finds its
/// chunks (movie41, 751.7 s in Littleroot).
struct OwnApps {
    /// The profile's field reader.
    field: Rc<RefCell<Rc<FieldReader>>>,
    running: Rc<dyn Fn() -> Option<(Rc<dyn OwnApp>, i64)>>,
    /// Whether the field is the app running.
    on_field: Rc<dyn Fn() -> bool>,
}

impl OwnApps {
    fn field(&self) -> Rc<FieldReader> {
        self.field.borrow().clone()
    }
}

impl FieldSource for OwnApps {
    fn visit(&self) -> Option<(i64, i64)> {
        self.field().visit()
    }

    fn poll(&self) -> Result<Option<Field>> {
        if let Some((app, process)) = (self.running)() {
            return app.listed(process);
        }
        let field = self.field();
        let r#where = field.poll()?;
        let Some(r#where) = r#where else {
            return Ok(None);
        };
        if (self.on_field)() {
            return Ok(Some(r#where));
        }
        Ok(if field.chunks_loaded()?.is_empty() {
            None
        } else {
            Some(Field {
                walkable: false,
                ..r#where
            })
        })
    }

    fn interactables(&self, r#where: &Field) -> Result<Field> {
        // An app's listing is whole, and says the way to each thing itself.
        if r#where.steer.is_some() {
            Ok(r#where.clone())
        } else {
            self.field().interactables(r#where)
        }
    }

    fn ground(&self, r#where: &Field) -> Result<Ground> {
        self.field().ground(r#where)
    }
}

/// `Gen6Profile._frames`: the game's count of frames, which a scene's beats are timed by
/// (`gen6.frames`): read in whatever phase asks, since a scene starts inside the field's.
fn frames_of(
    memory: Rc<dyn Memory>,
    managers: Rc<RefCell<PySet>>,
) -> Rc<dyn Fn() -> Result<Option<i64>>> {
    Rc::new(move || {
        let managers: Vec<i64> = managers.borrow().iter().collect();
        frames::frames(&*memory, &managers)
    })
}

/// A line of the "story" text or the "text" by its file and number, as the ROM has it, or
/// None.
fn line_of(text: Rc<GameText>) -> Lines {
    Rc::new(move |archive: &str, file: i64, number: i64| {
        let lines = match text.lines.get(archive) {
            Some(lines) => (lines.0)(file)?,
            None => Vec::new(),
        };
        Ok(if number < lines.len() as i64 {
            Some(lines[number as usize].clone())
        } else {
            None
        })
    })
}

/// `Gen6Profile._yours` over the game's text, which the readers made before the profile are
/// handed: a Pokémon of the player's by its nickname, or else its species and form.
fn yours_in(text: &GameText, found: &pokemon::Pokemon) -> Option<String> {
    if found.nicknamed && !found.nickname.is_empty() {
        return Some(found.nickname.clone());
    }
    text.pokemon(found.species, found.form)
}

/// `Gen6Profile._recall`: the word for what a scene names as it is said: `memory`, the Pokémon
/// the player's own save keeps for the credits' moment (spike 222), None where it keeps none;
/// `yours`, the nickname the player gave that Pokémon, None where they gave none or no longer
/// have it; `sprite`, what a speaker drawn as it is called.
fn recall(
    named: Rc<Names>,
    memories: Option<Rc<dyn MemoriesReader>>,
    text: Rc<GameText>,
) -> Recall {
    Rc::new(move |kind: &str, number: i64| {
        if kind == "sprite" {
            return Ok(named.sprite(number));
        }
        let Some(memories) = &memories else {
            return Ok(None);
        };
        if kind == "yours" {
            return memories.nickname(number);
        }
        let found = if kind == "memory" {
            memories.pokemon(number)?
        } else {
            None
        };
        Ok(found.and_then(|(species, form)| text.pokemon(species, form)))
    })
}

/// The game's own word for what a description names, `{ POKEMON(383) }` a Pokémon by its
/// number and `{ GAME() }` the game itself, or None.
fn word_of(text: Rc<GameText>) -> Word {
    Rc::new(move |kind: &str, number: i64| {
        if kind == "place" {
            return text.place(number);
        }
        if kind == "game" {
            return text.title.clone();
        }
        let said: &[String] = match kind {
            "pokemon" => &text.species,
            "trainer" => &text.trainers,
            "item" => &text.items,
            _ => &[],
        };
        if number < said.len() as i64 && !said[number as usize].is_empty() {
            Some(said[number as usize].clone())
        } else {
            None
        }
    })
}

/// `speaker.name_for`, as the `Speaker` that `Announcements` asks who is showing a window.
fn name_for(speaker: &Rc<SpeakerReader>) -> Speaker {
    let speaker = speaker.clone();
    Rc::new(move |window| speaker.name_for(window))
}

/// What the game calls a character that RAM gives only a number: the rival, by the
/// sprite a work object was filled with (spike 89).
fn game_word(text: &GameText, character: i64) -> Option<String> {
    text.character(character)
}

/// What the game calls a slot on a map: the building behind a door in from outdoors,
/// where the ROM's own tables pair the two (spike 86). Only a warp's slot is a door's: a sign
/// is numbered in a table of its own. A warp into one of X's gates is the gate and the place
/// beyond it, "Lumiose Gate to Route 4": Lumiose's five gates are all "Lumiose Gate" (Khai,
/// 2026-10-28).
fn game_place(text: &GameText, zone: i64, slot: i64) -> Option<String> {
    if source_kind(slot) != WARP_SOURCE {
        return None;
    }
    if let Some((gate, beyond)) = text.gate(zone, source_number(slot)) {
        return Some(own("leads-to", &[("thing", &gate), ("place", &beyond)]));
    }
    text.door(zone, source_number(slot))
}

/// Whether a slot on a map is a sign that says only the name of a door there
/// (`plates.door_signs`).
fn door_sign_of(text: Rc<GameText>) -> DoorSign {
    Rc::new(move |zone, slot| {
        source_kind(slot) == FURNITURE_SOURCE
            && text.door_signs.contains(&(zone, source_number(slot)))
    })
}

/// What the game calls the place an exit from `here` into `there` arrives in, or None when
/// it is the place the player is in already.
///
/// A place, not a zone, for the reason `arrive` gives: Littleroot and every house in it are
/// one place, so the door of a house leads nowhere new and says nothing more, while the gap
/// at the top of the town leads onto Route 101 and says so (the user, 2026-09-24). Both
/// indices are the zone table's in RAM and the words the ROM's, as for arriving.
///
/// The other zone of the player's own matrix is said all the same: Route 119 is two zones
/// the game calls by one name, and the edge between them was the only exit said bare, "edge"
/// (the user, 2026-10-03, spike 159).
fn leads_to_of(text: Rc<GameText>, field: Rc<RefCell<Rc<FieldReader>>>) -> LeadsTo {
    Rc::new(move |here, there| {
        if !text.loaded() {
            return Ok(None);
        }
        let field = field.borrow().clone();
        let (ours, theirs) = (
            field.zones.place_index(here)?,
            field.zones.place_index(there)?,
        );
        let (Some(ours), Some(theirs)) = (ours, theirs) else {
            return Ok(None);
        };
        if ours == theirs && !field.zones.one_matrix(here, there)? {
            return Ok(None);
        }
        Ok(text.place(theirs))
    })
}

/// What the game calls the place `zone` is in, or None.
fn place_of(text: Rc<GameText>, field: Rc<RefCell<Rc<FieldReader>>>) -> Place {
    Rc::new(move |zone| {
        let field = field.borrow().clone();
        let index = field.zones.place_index(zone)?;
        match index {
            Some(index) if text.loaded() => Ok(text.place(index)),
            _ => Ok(None),
        }
    })
}

/// The street of Lumiose's a warp between two of its zones comes out on, in the game's
/// own words (spike 107): its zones are all "Lumiose City", so the place says nothing.
fn street_of(text: Rc<GameText>) -> Street {
    Rc::new(move |zone, slot| {
        if source_kind(slot) != WARP_SOURCE {
            return Ok(None);
        }
        Ok(text.street(zone, source_number(slot)))
    })
}

/// Is this in the heap or the linear heap, where a work's resource and archive have been?
fn in_a_heap(address: i64) -> bool {
    within(HEAP_SPAN, address) || within(LINEAR_SPAN, address)
}

impl Profile for Gen6Profile {
    fn track(&self, found: &Found) -> Result<()> {
        Gen6Profile::track(self, found)
    }

    fn poll_app(&self) -> Result<bool> {
        Gen6Profile::poll_app(self)
    }

    fn app_sweep_wanted(&self) -> bool {
        Gen6Profile::app_sweep_wanted(self)
    }

    fn poll_dialogue(&self) -> Result<bool> {
        Gen6Profile::poll_dialogue(self)
    }

    fn poll_menus(&self, discovery: &mut Discovery) -> Result<bool> {
        Gen6Profile::poll_menus(self, discovery)
    }

    fn poll_screen(&self) -> Result<()> {
        Gen6Profile::poll_screen(self)
    }

    fn poll_field(&self) -> Result<()> {
        Gen6Profile::poll_field(self)
    }

    fn poll_footsteps(&self) -> Result<()> {
        Gen6Profile::poll_footsteps(self)
    }

    fn poll_guidance(&self) -> Result<()> {
        Gen6Profile::poll_guidance(self)
    }

    fn poll_pointers(&self) -> Result<()> {
        Gen6Profile::poll_pointers(self)
    }

    fn poll_renames(&self) -> Result<()> {
        Gen6Profile::poll_renames(self)
    }

    fn press(&self, key: &str) -> Result<()> {
        Gen6Profile::press(self, key)
    }

    fn sweep_wanted(&self) -> Result<bool> {
        Gen6Profile::sweep_wanted(self)
    }
}
