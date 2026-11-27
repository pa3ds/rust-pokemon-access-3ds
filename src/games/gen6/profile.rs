//! The gen 6 profile: which of the game's objects to find, the readers they go to, and how each
//! poll runs them.
//!
//! The runtime (`app`) knows when to sweep and when to poll; this knows what a sweep should look for,
//! which reader each object found belongs to, and in what order the readers are polled and their news
//! said. It reads ORAS and Pokémon X and Y, one engine, each game told apart by its `Game`: its
//! numbers, and what it alone has (spike 100).
//!
//! Bound by:
//! - ADR-0009: touch mode offers targets only in the apps that need a touch (`TOUCHED`, and the
//!   subscreens' own, `Subscreens.offered`), and Shift alone drags only where the subscreens say.
//! - ADR-0013: a menu's row read every tick polls the screen at once as it changes, and each tick
//!   after until the cursor's word is said (`poll_menus`, `rows.CHASE`).
//! - ADR-0034: a whole sweep waits while a cursor moves, until it is still `STILL`, at most
//!   `HELD_MOST`, and for a cursor yet to come as its screen opens, at most `HELD_MOST` too
//!   (`sweep_held`, `Chase.WAIT`).
//! - ADR-0041: a replay logs touch mode's targets as they change, so a tape holds them, and a tape
//!   replay P's ways at each listing (`ways`).
//! - ADR-0044: the screen poll asks the cursor sources through `sources.CursorSources` alone, and
//!   wires none itself (`poll_menus`, `poll_screen`).

use std::cell::{Cell, OnceCell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::app::most_common;
use crate::core::binary::u32;
use crate::core::clips::Clips;
use crate::core::config::{DRAG_TOGGLE, PARTY_BEEPS, PARTY_OFF, Tables};
use crate::core::discovery::{Found, LINEAR_SPAN, within};
use crate::core::keys::shifted;
use crate::core::memory::{Cache, Memory, phase};
use crate::core::names::{FURNITURE, GamePlace, GameSprite, Names, TILES};
use crate::core::pathfind::{COMPASS, walked};
use crate::core::rtti::{ClassNames, demangle};
use crate::core::speech::{Category, Speech};
use crate::core::tones::{Tones, beeps, sine, ticking, triangle};
use crate::core::touch::TouchScreen;
use crate::core::words::own;
use crate::games::common::announce::{Announcements, Speaker};
use crate::games::common::clocks::Clocks;
use crate::games::common::cursors::menu::Blank;
use crate::games::common::cursors::reader::CursorReader;
use crate::games::common::cursors::shared::{Rect, Slot};
use crate::games::common::dialogue::{DialogueReader, ORAS_SHAPES, Shown};
use crate::games::common::field::ground::Ground;
use crate::games::common::field::guide::FieldSource;
use crate::games::common::field::keys::{Bindings, Contexts, DEFAULT_CONTEXTS};
use crate::games::common::field::navigate::{Navigator, Options as NavigatorOptions, Sides};
use crate::games::common::field::things::{Field, Interactable, describe};
use crate::games::common::layout::{Cached, Layouts};
use crate::games::common::layout_file::{LayoutFile, LayoutFiles};
use crate::games::common::pictures::named as named_picture;
use crate::games::common::pictures::{CAUGHT, UNAVAILABLE};
use crate::games::common::scenes::{
    Described, GameClock, Lines, Recall, Scenes, Word, load as load_scenes,
    shipped as scenes_shipped,
};
use crate::games::common::screen::{Pictures, ScreenText, ScreenTextReader};
use crate::games::common::screen_announce::{Heard, ScreenAnnouncer};
use crate::games::common::screen_order::{in_rows, join};
use crate::games::common::session::{Profile, Ways};
use crate::games::common::slots::Occupant;
use crate::games::common::touch::{self, FileOf, ImageOf, OnScreen, TextOf, TouchMode};
use crate::games::common::touch_targets::{
    self, Idle, Lists, PictureName, TouchTargets, drawn_with,
};
use crate::games::common::{lyt, msgwin, pokemon, strbuf};
use crate::games::gen6::anchors::Anchors;
use crate::games::gen6::apps::{BATTLE, DEMOS, FIELD, MANAGER_CLASS, RunningApp, TUTORIAL};
use crate::games::gen6::feedback::FeedbackSounds;
use crate::games::gen6::field::dowsing::Lights;
use crate::games::gen6::field::features::Sighting;
use crate::games::gen6::field::footsteps::Footsteps;
use crate::games::gen6::field::places::PlaceNames;
use crate::games::gen6::field::reader::{FieldReader, MOVE_MODEL_CLASS};
use crate::games::gen6::field::speaker::SpeakerReader;
use crate::games::gen6::field::voice::Voice;
use crate::games::gen6::field::{camera, mapblock, talk};
use crate::games::gen6::game::Game;
use crate::games::gen6::own::{Kit, Own, OwnApp};
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::rom::textures;
use crate::games::gen6::rom::textures::Index;
use crate::games::gen6::screens::battle::{
    BattleReader, Battlers, GAUGE_CLASSES, Gauges, OpponentsParty,
};
use crate::games::gen6::screens::choices::{
    BOX_DEVICE_MANAGER, ChoiceReader, DEVICE_MANAGER_CLASS,
};
use crate::games::gen6::screens::claims::{Claims, Owner};
use crate::games::gen6::screens::habitat::HabitatReader;
use crate::games::gen6::screens::lists::{CONTROLLER_CLASS, ListReader};
use crate::games::gen6::screens::mapcursor::MapCursor;
use crate::games::gen6::screens::partners::Partners;
use crate::games::gen6::screens::partyicons::PartyIcons;
use crate::games::gen6::screens::sources::{Chase, CursorSources};
use crate::games::gen6::screens::{
    boxes, buttons, claims, partners, partyicons, pokedex, registered, rows, townmap,
};
use crate::games::gen6::subscreen::{self, Subscreens, pss};
use crate::games::gen6::{frames, gflheap, mystatus};
use crate::py::{self, Error, Kind, Log, PySet, Result};

/// What the heap is swept for: message windows, list controllers, the device manager a choice box
/// holds, layout works through their DrawInfo, the battlers' names and HP gauges, the map's move
/// models, the heaps themselves, whose blocks the runtime watches for what they allocate (`app`,
/// ADR-0034), the camera whose heading the D-pad walks by on Lumiose's rails, the process manager
/// that says which app is running, and the button managers, touch bars and the field's menu that
/// say whose buttons take taps (`buttons`); the party and the icons the Bag draws it with; the
/// Battle Maison's partner screen, which says whose portrait each picture draws; and the save's
/// trainer record, which says the game's language (`mystatus`). The subscreens add what they find
/// their objects by (`Subscreens.classes`), and a game's own subscreen and readers theirs
/// (`Gen6Profile.classes`): ORAS's AreaNav's navigator, which says whose icon each picture of its
/// Pokémon Distribution draws, and its views, which say which of its layouts it draws; and ORAS's
/// DexNav, the player's work its hidden Pokémon's record is found through.
///
/// Not TextBoxes or cursor windows: walking each layout's tree finds every one that can be placed on
/// screen (spike 62). And the linear heap is not swept: the battle builds its layouts there, and its
/// message window leads to their pool (spike 64).
pub const HEAP_CLASSES: [&str; 26] = [
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
    partners::CLASS,
    pokemon::PARAM_CLASS,
    partyicons::ICON_CLASS,
    partyicons::PARTY_SAVE,
    partyicons::BATTLE_BOX,
    boxes::BOXES,
    boxes::NAMES,
    registered::BAG,
    buttons::MANAGER_CLASS,
    buttons::BAR_CLASS,
    buttons::MENU_CLASS,
    mystatus::CLASS,
];

/// Without a message window class there is nothing to read: not this game, or not this version.
pub const REQUIRED: [&str; 2] = msgwin::CLASSES;
/// Seconds a cursor holds still before the heap's whole sweep, held while it moves, runs. A sweep
/// reads 13 MB in one tick of 1.0 to 1.3 s with nothing else read, and the Bag's own, 1.5 s after
/// it starts, came as the player began moving down its list: the first item of Medicine went
/// without its details (findings/cursors.md, spike 297).
pub const STILL: f64 = 1.0;
/// The longest seconds a moving cursor holds the whole sweep off, so what only a sweep finds is
/// never more than that late.
pub const HELD_MOST: f64 = 3.0;
/// The apps touch mode offers targets in (ADR-0009 as amended 2026-10-29): the field, whose
/// bottom screen is the PokéNav Plus and its DexNav, BuzzNav and PlayNav, or X's own, with
/// Pokémon-Amie's and Super Training's pages; the subscreens' tutorials; the AreaNav; and beside
/// them the subscreens' own apps, Pokémon-Amie's and Super Training's, and the PSS's, which the
/// subscreens say (`Subscreens.offered`). Every other
/// screen, the Bag, the PC, a battle, is worked with the game's buttons, which do all its touches
/// do and faster, and touch mode says there is nothing to touch. A
/// `frozenset` in Python, only ever tested for membership.
pub const TOUCHED: [&str; 3] = [FIELD, TUTORIAL, townmap::APP];
/// Where the tone of each quarter up walks sounds, across and down as `Tones.play` has them,
/// north first and clockwise: north high, south low, east in the right ear and west in the left, as
/// the Hoenn map's cursor sounds its place (`core.tones`).
pub const FACING_TONES: [(f64, f64); 4] = [(0.5, 0.0), (1.0, 0.5), (0.5, 1.0), (0.0, 0.5)];
/// Seconds between follows of the running app's chains (`gen6.anchors`), and at once as an app
/// starts: what it keeps is built over its first second or two, and rebuilt as the field loads a
/// zone.
pub const ANCHOR_EVERY: f64 = 0.2;
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
    /// None is `DEFAULT_BINDINGS`.
    pub keys: Option<Bindings>,
    pub text: Rc<GameText>,
    pub pictures: Option<Rc<Index>>,
    /// The executable as read from its code, `b""` when there is none.
    pub image: Vec<u8>,
    pub touch_screen: Option<Rc<dyn TouchScreen>>,
    pub tones: Option<Rc<Tones>>,
    pub ticks: Option<Rc<Tones>>,
    pub clips: Option<Rc<Clips>>,
    pub sounds: Option<Rc<FeedbackSounds>>,
    /// None is `{}`.
    pub swims: Option<IndexMap<String, f64>>,
    pub title: Option<i64>,
    pub census: bool,
    pub ways: Option<Ways>,
    /// None is `DEFAULT_CONTEXTS`.
    pub contexts: Option<Contexts>,
    /// None is `Tables()`.
    pub tables: Option<Tables>,
    pub drawn: Option<Rc<dyn Memory>>,
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
    /// What P's ways at each listing are read inside, or None where they are not logged
    /// (`app.Settings.ways`).
    ways: Option<Ways>,
    pub layouts: Rc<Layouts>,
    /// Which screen the game is running, and the moment another starts (spike 99); and what
    /// its process leads to, from that moment (spike 282).
    ///
    /// Shared with the own apps' `running`, which the Python hands as a bound method.
    pub apps: Rc<RefCell<RunningApp>>,
    pub anchors: RefCell<Anchors>,
    /// When the running app's chains were last followed (`ANCHOR_EVERY`).
    anchored: Cell<f64>,
    pub dialogue: RefCell<DialogueReader>,
    /// Shared with touch mode's `lists`, which the Python hands as a bound method.
    pub lists: Rc<RefCell<ListReader>>,
    pub choices: Rc<RefCell<ChoiceReader>>,
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
    pub battle: Rc<RefCell<BattleReader>>,
    /// How many Pokémon a trainer has, from the tray of balls its battle opens with (spike 285),
    /// said or beeped as the player chose.
    pub opponents: Option<RefCell<OpponentsParty>>,
    party_beeped: bool,
    tones: Option<Rc<Tones>>,
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
    /// What the game calls where the player arrives, where an exit leads and a slot on a map.
    pub places: Rc<PlaceNames>,
    /// What the heap is swept for in this game: the classes of what it alone has,
    /// `HEAP_CLASSES`, the subscreens' classes, and what the field's own readers find their
    /// objects by. Its own come first, before the touch bars, as ORAS's subscreen's did inside
    /// `HEAP_CLASSES`: the log names the classes a module brings in that order, the AreaNav's
    /// views and then its touch bar.
    pub classes: Vec<String>,
    /// What the game has alone: ORAS's PokéNav Plus and contests, its credits' Pokémon, which
    /// their descriptions name (spike 222), and soaring, while which the keys walk what it
    /// lists, which the field's stale lists know nothing of (spike 166); X's opening.
    pub own: Rc<dyn Own>,
    /// The subscreens, Pokémon-Amie's and Super Training's and the apps they open, which name
    /// what they draw, offer what no touch area does, and answer keys of their own.
    pub subscreens: Rc<Subscreens>,
    /// A screen's own keys, answered before the navigator's while it runs (`context`).
    contexts: Contexts,
    pub buttons: Rc<RefCell<buttons::Buttons>>,
    pub party_icons: Rc<RefCell<PartyIcons>>,
    pub partners: Rc<RefCell<Partners>>,
    registered: Rc<RefCell<registered::Registered>>,
    pub occupants: Rc<RefCell<boxes::Occupants>>,
    /// What each screen's own objects say its cursor is on (ADR-0043).
    pub sources: Rc<RefCell<CursorSources>>,
    /// Ticks left to poll the screen at, until a row's move is said (`rows.CHASE`).
    chasing: Cell<i64>,
    /// Whether the move chased is said once its details are too (`Chase.DETAILS`).
    chasing_details: Cell<bool>,
    /// Whether the last screen poll said a cursor's move or opening.
    cursor_said: Cell<bool>,
    pub map_cursor: RefCell<MapCursor>,
    pub footsteps: Option<RefCell<Footsteps>>,
    /// Python's `self.touch` is the navigator's `touch`, which it owns here.
    pub navigator: RefCell<Navigator>,
    log: Log,
    speech: Rc<Speech>,
    /// Gen 6's own field voices: the Dowsing Machine's lights, in a game where they have been
    /// measured.
    voices: Vec<Voice>,
    /// The zone last listed and what was in it (people by kind alone, since they walk), to
    /// list again only when something comes or goes.
    listed: RefCell<Option<(i64, Vec<(String, String, Option<(i64, i64)>)>)>>,
    /// Whether a sweep has been taken in since the last listing.
    relist: Cell<bool>,
    /// The chunks the last listing was narrowed by, as logged.
    chunks: RefCell<Vec<(i64, i64)>>,
    /// What each watched feature showed at the last poll (`FieldReader.watch`): what was up,
    /// to list again when one comes or goes between sweeps, and its record as last logged, so a
    /// change is logged whether or not it passes as what it is: a rustle was heard that the
    /// list did not have.
    seen: RefCell<Vec<Sighting>>,
    /// Whether the player was riding the bike at the last field poll, to log each change.
    on_bike: Cell<bool>,
    /// The quarters from north up walks the player, as the compass was last turned.
    turn: Cell<i64>,
    /// The quarter of the world up walks the player, as its tone last said (`facing`).
    facing: Cell<Option<i64>>,
    /// Python's `partial(tones.play, wave=triangle)`, kept for the facing's tone.
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
    /// The Pokédex's habitat map draws the Pokémon's places and never names them (spike 152).
    pub habitat: Rc<RefCell<HabitatReader>>,
    /// Whose words (`gen6.screens.claims`).
    pub claims: Claims,
    /// The zone the player was last polled in.
    zone: Cell<Option<i64>>,
    /// Message windows the field has shown a page in that no talk object kept was showing,
    /// swept for once each, and again after the player enters another zone.
    unheard: RefCell<HashSet<i64>>,
    /// Whether a sweep is wanted for the talk object showing a page.
    speaker_wanted: Cell<bool>,
    after_forget: AfterForget,
    /// Whether a cursor has moved or opened since `sweep_held` last asked.
    pub(crate) moving: Cell<bool>,
    /// The tick a cursor was last seen to move or open at.
    moved: Cell<Option<f64>>,
    /// The tick a moving cursor began holding the whole sweep off at, while it does.
    holding: Cell<Option<f64>>,
    /// Whether a cursor source said at the last tick that a cursor is yet to come
    /// (`Chase.WAIT`).
    waiting: Cell<bool>,
    /// The tick a cursor yet to come began holding the whole sweep off at, while one is.
    waited: Cell<Option<f64>>,
    /// Every device manager a sweep has found. It lives all session, so a choice box built
    /// later is found in the blocks just allocated by the manager it holds. Iterated by
    /// `Found.holders`, whose result follows its order, so a `PySet`. Shared with the scenes'
    /// clock, which reads the game's frames through them (`frames`).
    managers: Rc<RefCell<PySet>>,
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
            keys,
            text,
            pictures,
            image,
            touch_screen,
            tones,
            ticks,
            clips,
            sounds,
            swims,
            title,
            census,
            ways,
            contexts,
            tables,
            drawn,
        } = options;
        // The settings file's tables, each handed to the readers it is for (`config.Tables`).
        let tables = tables.unwrap_or_default();
        let (ui, touch_settings) = (tables.ui.clone(), tables.touch.clone());
        let contexts = contexts.unwrap_or_else(|| DEFAULT_CONTEXTS.clone());
        let tone = tones.clone();
        let beeping = tones.clone();
        let cache = memory;
        let memory: Rc<dyn Memory> = cache.clone();
        let named = named.unwrap_or_else(|| Rc::new(Names::empty()));
        // The page cache's pages, which a layout pool gone is looked for again in; none from a
        // snapshot or a fake, which cache none.
        let cached: Option<Cached> = cache.has_cached().then(|| {
            let cache = cache.clone();
            Rc::new(move |page: i64| cache.cached(page)) as Cached
        });
        let layouts = Rc::new(Layouts::with_cached(
            memory.clone(),
            names.clone(),
            lyt::ORAS,
            cached,
        ));
        let apps = Rc::new(RefCell::new(RunningApp::new(memory.clone(), names.clone())));
        // The options screen types TEXT SPEED's description again to show the speed chosen.
        let dialogue = DialogueReader::with(
            memory.clone(),
            names.clone(),
            Some(layouts.clone()),
            ORAS_SHAPES,
            {
                let apps = apps.clone();
                Rc::new(move || apps.borrow().running.as_deref() != Some(rows::OPTIONS))
            },
        );
        let lists = Rc::new(RefCell::new(ListReader::new(memory.clone(), names.clone())));
        let choices = Rc::new(RefCell::new(ChoiceReader::new(
            memory.clone(),
            names.clone(),
        )));
        // A picture whose texture the game swaps as it runs is named by the image it draws: the
        // Pokédex's language tabs and its type (spike 147).
        let named_by_image: Option<Pictures> = pictures.map(|pictures| {
            let memory = memory.clone();
            Rc::new(move |pane: i64| pictures.said(&*memory, pane)) as Pictures
        });
        // The Bag draws each party member's icon, and says nothing of whose it is. Made here,
        // before the screen text reader that asks it, where Python makes it later and reaches it
        // through `self`; making it reads nothing.
        let party_icons = Rc::new(RefCell::new(PartyIcons::new(
            memory.clone(),
            names.clone(),
            text.species.clone(),
            {
                let apps = apps.clone();
                Rc::new(move || apps.borrow().running.clone())
            },
        )));
        // The Battle Maison's partners are portraits over names drawn with no TextBox. Made here,
        // before the screen text reader that asks it, where Python makes it later and reaches it
        // through `self`; making it reads nothing.
        let partners = Rc::new(RefCell::new(Partners::new(memory.clone())));
        // The Bag's party draws each member as an icon beside its level, or, for an evolution
        // stone, beside "ABLE!" or "UNABLE!": "Metagross UNABLE!" (movie31). A game's own
        // names its pictures for its readers to say whole: ORAS's AreaNav's Pokémon
        // Distribution, whose icons' buttons take no tap (`Own.named`).
        // Made here, before the screen text reader, which names pictures by it: Python's closure
        // reaches `self.files` once it is made, later. Making either reads nothing.
        let files = Rc::new(LayoutFiles::new(memory.clone()));
        // What a game's own readers are built from (`gen6.own`).
        let kit = Kit {
            memory: memory.clone(),
            names: names.clone(),
            layouts: layouts.clone(),
            files: files.clone(),
            apps: apps.clone(),
            text: text.clone(),
            speech: speech.clone(),
            log: log.clone(),
            tables: tables.clone(),
            phase: cache.clone(),
        };
        // What the game has alone: ORAS's PokéNav Plus and contests, its credits' Pokémon,
        // which their descriptions name (spike 222), and soaring, while which the keys walk what
        // it lists, which the field's stale lists know nothing of (spike 166); X's opening. Made
        // here, before the screen text reader that asks it, where Python makes it after; making
        // it reads nothing. `alone` here, as `own` is pa3ds's own words.
        let alone = (game.own)(&kit);
        let image_of: ImageOf = {
            let (memory, files) = (memory.clone(), files.clone());
            Rc::new(move |pane: i64| {
                let at = textures::drawn_at(&*memory, pane)?;
                Ok(at.and_then(|at| files.image_at(at)))
            })
        };
        // The subscreens name theirs: Pokémon-Amie's games draw a result's count a digit a
        // picture (`Subscreens.name`). The subscreens are made below, after the screen text reader
        // whose words they read: Python's closure reaches `self.subscreens` once it is made, and
        // this one through the cell it is put in then.
        let subscreens_made: Rc<OnceCell<Rc<Subscreens>>> = Rc::new(OnceCell::new());
        let on_screen_named: Pictures = {
            let (named_by_image, party_icons) = (named_by_image.clone(), party_icons.clone());
            let subscreens = subscreens_made.clone();
            let (alone, partners) = (alone.clone(), partners.clone());
            Rc::new(move |pane: i64| {
                let by_image = match &named_by_image {
                    Some(named) => named(pane)?,
                    None => None,
                };
                // Python's `or`: a name of "" or None goes on to the subscreens', then to the
                // party's icons, then to the game's own, and then to the Maison's partners.
                if let Some(name) = by_image
                    && !name.is_empty()
                {
                    return Ok(Some(name));
                }
                if let Some(subscreens) = subscreens.get()
                    && let Some(name) = subscreens.name(pane)?
                    && !name.is_empty()
                {
                    return Ok(Some(name));
                }
                if let Some(name) = party_icons.borrow().name(pane)?
                    && !name.is_empty()
                {
                    return Ok(Some(name));
                }
                if let Some(name) = alone.named(pane)?
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
        // How many Pokémon a trainer has, from the tray of balls its battle opens with (spike 285),
        // said or beeped as the player chose.
        let opponents = (ui.trainer_party != PARTY_OFF)
            .then(|| RefCell::new(OpponentsParty::new(layouts.clone(), image_of.clone())));
        let party_beeped = ui.trainer_party == PARTY_BEEPS;
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
            &tables.extras.hidden_items,
        );
        let announcements = Rc::new(RefCell::new(Announcements::with(
            speech.clone(),
            log.clone(),
            Some(name_for(&speaker)),
            false,
        )));
        let field = Rc::new(RefCell::new(field));
        // What the game calls where the player arrives, where an exit leads and a slot on a map.
        let places = Rc::new(PlaceNames::new(text.clone(), game.items_at.clone(), {
            let field = field.clone();
            Rc::new(move || field.borrow().clone())
        }));

        // Touch mode walks the bottom screen's touch areas, and taps one on the player's key
        // (`games.common.touch`, ADR-0009 as amended 2026-10-01).

        let file_of: FileOf = {
            let (files, layouts) = (files.clone(), layouts.clone());
            Rc::new(move |root: i64| -> Option<Rc<LayoutFile>> {
                files.matching(&layouts.pane_names(root))
            })
        };

        // The game's button managers say whose buttons take taps (spike 189).
        let buttons = Rc::new(RefCell::new(buttons::Buttons::with_menu(
            memory.clone(),
            game.field_menu,
        )));
        let pauses = ui.pauses;
        let sides: Sides = {
            let (alone, announcements, apps, subscreens) = (
                alone.clone(),
                announcements.clone(),
                apps.clone(),
                subscreens_made.clone(),
            );
            Rc::new(move |player: bool| {
                // ORAS's contests' appeals draw who stands where as portraits, said first.
                let standing = alone.sides(player)?;
                // Python's `or`: a standing of "" or None goes on to the battle's sides.
                let said = match standing {
                    Some(standing) if !standing.is_empty() => Some(standing),
                    _ => announcements.borrow().sides(player),
                };
                // A battle has begun before the game draws either side: R was "no battle" for the
                // seconds before "Go!".
                if said.is_none() && apps.borrow().running.as_deref() == Some(BATTLE) {
                    return Ok(Some(own("key-battle-not-shown", &[])));
                }
                // On the subscreens', what R says there: O-Power's energy and the powers running.
                if said.is_none()
                    && let Some(subscreens) = subscreens.get()
                {
                    return subscreens.sides(player);
                }
                Ok(said)
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
        // The DexNav draws a Pokémon caught in colour, and only those are said so: "Ralts",
        // "Magikarp fishing caught", with commas under pauses (ADR-0037, `Distribution.said`).
        let gap = if ui.pauses { ", " } else { " " };
        let mut images: IndexMap<String, String> = text.touch_words.clone();
        // Pictures of words the game's text has no line for are named by pa3ds's own (X's Amie
        // menu), and keep to them as a button with the game's word does.
        for image in &game.worded {
            if let Some(said) = named_picture(image)
                && !said.is_empty()
            {
                images.insert(image.clone(), said);
            }
        }

        let box_process: Rc<dyn Fn() -> Option<i64>> = {
            let apps = apps.clone();
            Rc::new(move || {
                let apps = apps.borrow();
                if apps.running.as_deref() == Some(boxes::APP) {
                    apps.process
                } else {
                    None
                }
            })
        };
        let registered = Rc::new(RefCell::new(registered::Registered::new(
            memory.clone(),
            text.items.clone(),
            file_of.clone(),
        )));
        // Carrying a Pokémon in the PC Box, the slot it is over says who is in it, from the save.
        // Made before the block the cursors are made in, where Python makes it inside, just before
        // them; making it reads nothing.
        let box_shown = boxes::shown(
            memory.clone(),
            game.cursors.r#box.map(|r#box| r#box.shown),
            box_process.clone(),
        );
        let occupants = Rc::new(RefCell::new(boxes::Occupants::with(
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
            box_shown.clone(),
        )));
        // What each screen's own objects say its cursor is on (ADR-0043). Made here, before the
        // cursor reader that asks it the Box's slot, where Python makes it after and reaches it
        // through `self`; making it reads nothing.
        let sources = Rc::new(RefCell::new(CursorSources::new(
            memory.clone(),
            names.clone(),
            apps.clone(),
            layouts.clone(),
            files.clone(),
            &game.cursors,
            &text,
            pauses,
            {
                let buttons = buttons.clone();
                Rc::new(move || buttons.borrow().menu())
            },
            {
                let choices = choices.clone();
                Rc::new(move || choices.borrow().boxes())
            },
            {
                let gauges = gauges.clone();
                Rc::new(move || gauges.kept())
            },
            occupants.clone(),
            box_shown.clone(),
            alone.clone(),
            Some(registered.clone()),
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
                    let none = IndexMap::new();
                    let textures = file.as_ref().map_or(&none, |file| &file.textures);
                    for image in drawn_with([pane], &*image_of, textures, &|pane| {
                        layouts.pane_name(pane)
                    }) {
                        if let Some(word) = images.get(&image?) {
                            return Ok(Some(word.clone()));
                        }
                    }
                    Ok(None)
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
                // The Box's pointer is said on the slot the game sends it to, not once it rests
                // there.
                Some({
                    let sources = sources.clone();
                    Rc::new(move || sources.borrow().box_slot()) as Slot
                }),
                Some({
                    let (apps, labels) = (apps.clone(), text.search_labels.clone());
                    Rc::new(
                        move |rect: &Rect, on_screen: &[ScreenText], framed: &[ScreenText]| {
                            pokedex::blank(
                                apps.borrow().running.as_deref(),
                                rect,
                                on_screen,
                                framed,
                                &labels,
                            )
                        },
                    ) as Blank
                }),
                // A trade's slots show no summary: each is said with who is in it.
                {
                    let apps = apps.clone();
                    Rc::new(move || apps.borrow().running.as_deref() == Some(pss::TRADE_LIST))
                },
            )
        };
        let off: touch_targets::Off = {
            let buttons = buttons.clone();
            Rc::new(move || buttons.borrow().off())
        };
        let on: touch_targets::Off = {
            let buttons = buttons.clone();
            Rc::new(move || buttons.borrow().on())
        };
        let greyed: touch_targets::Off = {
            let buttons = buttons.clone();
            Rc::new(move || buttons.borrow().greyed())
        };
        // The game's own sounds for what a key or a moment has done, or the words in their place.
        let sounds = sounds.unwrap_or_else(|| {
            Rc::new(FeedbackSounds::new(log.clone(), speech.clone(), None, None))
        });
        // The input device managers `_frames` reads the game's count of frames through, made here
        // where Python's are made below: Head It's cue is handed `_frames`, as the scenes are.
        let managers = Rc::new(RefCell::new(PySet::new()));
        // The clocks' TextBoxes found ticking, which the screen announcer's clocks keep below and
        // C reads here, Python's `_clock` reading the one `Clocks` the announcer is handed.
        let kept_clocks: Rc<RefCell<IndexSet<i64>>> = Rc::new(RefCell::new(IndexSet::new()));
        // The row of the clock last found ticking that is still on the screen, as C says it:
        // Berry Picker's "Time Left: 58", or "" with none (`Clocks.clocks`).
        let clock: Rc<dyn Fn() -> String> = {
            let (on_screen, kept, pauses) = (on_screen.clone(), kept_clocks.clone(), ui.pauses);
            Rc::new(move || {
                let on_screen = on_screen();
                let clocks: Vec<i64> = kept.borrow().iter().copied().collect();
                let clock = clocks
                    .iter()
                    .rev()
                    .find_map(|&r#box| on_screen.iter().find(|piece| piece.r#box == r#box))
                    .cloned();
                let Some(clock) = clock else {
                    return String::new();
                };
                // Python's `max(clock.glyph, 1)`: the glyph unless 1 is greater.
                let least = if 1.0 > clock.glyph { 1.0 } else { clock.glyph };
                let row: Vec<ScreenText> = on_screen
                    .into_iter()
                    .filter(|p| p.root == clock.root && (p.row - clock.row).abs() < least)
                    .collect();
                join(&in_rows(row), pauses)
            })
        };
        // The subscreens, Pokémon-Amie's and Super Training's and the apps they open, which name
        // what they draw, offer what no touch area does, and answer keys of their own, Berry
        // Picker's.
        let subscreens = Rc::new(Subscreens::new(
            memory.clone(),
            names.clone(),
            layouts.clone(),
            image_of.clone(),
            images.clone(),
            text.clone(),
            game,
            apps.clone(),
            &tables,
            sounds.clone(),
            frames_of(memory.clone(), managers.clone()),
            drawn,
            file_of.clone(),
            on_screen.clone(),
            {
                let party_icons = party_icons.clone();
                Rc::new(move || party_icons.borrow().party)
            },
            ticking(ticks.clone()),
            clock,
        ));
        subscreens_made
            .set(subscreens.clone())
            .unwrap_or_else(|_| unreachable!("the subscreens are made once"));
        screen.borrow_mut().set_reworded({
            let (subscreens, alone) = (subscreens.clone(), alone.clone());
            Rc::new(move |found| Ok(alone.reworded(subscreens.reworded(found)?)))
        });
        let classes: IndexSet<String> = alone
            .classes()
            .into_iter()
            .chain(
                HEAP_CLASSES
                    .iter()
                    .chain(subscreens.classes.iter())
                    .copied()
                    .chain(field.borrow().classes())
                    .map(String::from),
            )
            .collect();
        let picture_name: PictureName = {
            let (apps, alone, subscreens) = (apps.clone(), alone.clone(), subscreens.clone());
            let partners = partners.clone();
            Rc::new(move |pane: i64| {
                // The game's own names its pictures first: the DexNav's icons on the field, the
                // Pokémon Distribution's in the AreaNav (`Own.touched`).
                // A name of "" or None goes on to the next.
                let running = apps.borrow().running.clone();
                if let Some(name) = alone.touched(pane)?
                    && !name.is_empty()
                {
                    return Ok(Some(name));
                }
                if let Some(name) = subscreens.touched(pane)?
                    && !name.is_empty()
                {
                    return Ok(Some(name));
                }
                // The Battle Maison's partners, asked where the AreaNav's navigators are, on the
                // field and in the AreaNav.
                let near = running
                    .as_deref()
                    .is_none_or(|running| running == FIELD || running == townmap::APP);
                if near {
                    return Ok(partners
                        .borrow()
                        .name(pane)?
                        .filter(|name| !name.is_empty()));
                }
                Ok(None)
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
        let targets = Rc::new(RefCell::new(TouchTargets::new(
            layouts.clone(),
            on_screen,
            text_of,
            image_of.clone(),
            file_of,
            log.clone(),
            touch_targets::Options {
                images: Some(images.clone()),
                picture_name: Some(picture_name),
                caught: text.caught.clone(),
                said_caught: Some(Rc::new(move |words: &str| {
                    format!("{words}{gap}{}", own(CAUGHT, &[]))
                })),
                said_unavailable: Some(Rc::new(move |words: &str| {
                    format!("{words}{gap}{}", own(UNAVAILABLE, &[]))
                })),
                lists: Some(lists_labels),
                idle: Some(idle),
                off: Some(off),
                on: Some(on),
                greyed: Some(greyed),
                offered: Some({
                    let (apps, subscreens) = (apps.clone(), subscreens.clone());
                    Rc::new(move || {
                        let running = apps.borrow().running.clone();
                        running.is_none()
                            || running
                                .as_deref()
                                .is_some_and(|running| TOUCHED.contains(&running))
                            || subscreens.offered()
                    })
                }),
                said_off: Some({
                    let alone = alone.clone();
                    Rc::new(move |pane: i64| alone.said_off(pane))
                }),
                places: Some({
                    let (apps, alone) = (apps.clone(), alone.clone());
                    let map_places = Rc::new(text.map_places.clone());
                    let none = Rc::new(IndexMap::new());
                    Rc::new(move || {
                        let running = apps.borrow().running.clone();
                        Ok(
                            if running.as_deref() == Some(townmap::APP) && !alone.drawing()? {
                                map_places.clone()
                            } else {
                                none.clone()
                            },
                        )
                    })
                }),
                gestures: Some({
                    let subscreens = subscreens.clone();
                    Rc::new(move || subscreens.targets())
                }),
                pictured: Some({
                    let subscreens = subscreens.clone();
                    Rc::new(move || subscreens.offer())
                }),
            },
        )));
        let touch_targets = targets.clone();
        let touch = TouchMode::new(
            Rc::new(move || targets.borrow_mut().targets()),
            log.clone(),
            touch::ModeOptions {
                screen: touch_screen,
                tone: tones.clone().map(|tones| {
                    Rc::new(move |across: f64, down: f64| {
                        // The sound thread not starting, raised to the key's handler in Python.
                        let _ = tones.play(across, down, sine);
                    }) as touch::Tone
                }),
                settings: touch_settings.clone(),
                delayed: Some({
                    let subscreens = subscreens.clone();
                    Rc::new(move || subscreens.delayed())
                }),
                pace: Some({
                    let subscreens = subscreens.clone();
                    Rc::new(move || subscreens.pace())
                }),
                sound: sounds.dragged(),
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
        // The navigator says what it finds by the name it has been given, and renames it on a
        // key: the player standing in front of a sign is the only one who can name it.
        let navigator = Navigator::new(
            Rc::new(OwnApps {
                field: field.clone(),
                running: {
                    let (apps, alone) = (apps.clone(), alone.clone());
                    Rc::new(move || own_running(&apps, &alone.apps()))
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
                navigation: Some(tables.navigation.clone()),
                speaker: Some({
                    let speaker = speaker.clone();
                    Rc::new(move || {
                        speaker
                            .last()
                            .map(|(character, own)| (character, Some(own)))
                    })
                }),
                leads_to: Some({
                    let places = places.clone();
                    Rc::new(move |here, there| places.leads_to(here, there))
                }),
                place: Some({
                    let places = places.clone();
                    Rc::new(move |zone| places.place(zone))
                }),
                part: Some({
                    let places = places.clone();
                    Rc::new(move |item: &Interactable, zone| places.part(item, zone))
                }),
                street: Some({
                    let places = places.clone();
                    Rc::new(move |zone, slot| Ok(places.street(zone, slot)))
                }),
                door_sign: Some({
                    let places = places.clone();
                    Rc::new(move |zone, slot| places.door_sign(zone, slot))
                }),
                moves: Some(text.moves.clone()),
                bike: text.bike.clone(),
                acro: text.acro.clone(),
                sides: Some(sides),
                touch: Some(touch),
                categories: alone
                    .apps()
                    .iter()
                    .flat_map(|app| app.categories())
                    .map(String::from)
                    .collect(),
                drags: Some({
                    let apps = apps.clone();
                    Rc::new(move || {
                        apps.borrow()
                            .running
                            .as_deref()
                            .is_some_and(|running| subscreen::DRAGS.contains(&running))
                    })
                }),
                toggle: touch_settings.drag == DRAG_TOGGLE,
                ..NavigatorOptions::default()
            },
        );
        let voices: Vec<Voice> = match game.dowsing {
            None => Vec::new(),
            Some(offset) => {
                let lights = RefCell::new(Lights::with_process(
                    memory.clone(),
                    offset,
                    field_process(apps.clone()),
                ));
                vec![
                    Rc::new(move |r#where: Option<&Field>| lights.borrow_mut().heard(r#where))
                        as Voice,
                ]
            }
        };
        let screen_announcer = ScreenAnnouncer::with_clocks(
            speech.clone(),
            log.clone(),
            ui.pauses,
            Some(Box::new(
                Clocks::with(
                    log.clone(),
                    ticking(ticks),
                    Some({
                        let subscreens = subscreens.clone();
                        Rc::new(move || subscreens.timed())
                    }),
                    Some({
                        let subscreens = subscreens.clone();
                        Rc::new(move || subscreens.playing())
                    }),
                    Some({
                        let subscreens = subscreens.clone();
                        Rc::new(move |piece: &ScreenText| subscreens.scored(piece))
                    }),
                    tables.amie.head_it_score,
                )
                .keeping(kept_clocks),
            )),
        );
        // The scenes the game shows without words, described where the player asks and the game
        // has descriptions in their language (ADR-0038), those of the other version left out.
        let scenes = if ui.descriptions && text.loaded() {
            // Python raises out of `__init__` here; `new` returns no `Result`, and nothing the
            // file or the ROM holds raises in `load` but a line number in digits no int reads.
            let described = Rc::new(
                Described::new(
                    line_of(text.clone()),
                    word_of(text.clone()),
                    Some(recall(named.clone(), alone.clone())),
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
        let habitat = HabitatReader::new(
            memory.clone(),
            layouts.clone(),
            files.clone(),
            text.map_places.clone(),
            speech.clone(),
            log.clone(),
            ui.pauses,
        );
        let battle = Rc::new(RefCell::new(battle));
        let habitat = Rc::new(RefCell::new(habitat));
        // Whose words: the HP boxes are the battle reader's to say, whole and once they hold
        // still; the AreaNav's Pokémon Distribution's icons the game's own; a button's the
        // touch target's.
        let claims = Claims::new(
            vec![
                {
                    let battle = battle.clone();
                    Rc::new(move |text: &ScreenText| battle.borrow().owns(text)) as Owner
                },
                {
                    let habitat = habitat.clone();
                    Rc::new(move |text: &ScreenText| Ok(habitat.borrow().owns(text))) as Owner
                },
            ],
            {
                let alone = alone.clone();
                Rc::new(move |text: &ScreenText| alone.owns(text))
            },
            {
                let targets = touch_targets.clone();
                Rc::new(move || targets.borrow_mut().button_words())
            },
            {
                let (apps, subscreens) = (apps.clone(), subscreens.clone());
                Rc::new(move || {
                    claims::rule(apps.borrow().running.as_deref(), &|| subscreens.in_pss())
                })
            },
            {
                let (layouts, files) = (layouts.clone(), files.clone());
                Rc::new(move |piece: &ScreenText| pane_named(&layouts, &files, piece))
            },
        );
        // The rival is a work object in RAM and a name in the ROM, and so are some doors' buildings:
        // the game's own word, behind whatever the player has called them and ahead of pa3ds's.
        let sprite: GameSprite = {
            let text = text.clone();
            Rc::new(move |character| game_word(&text, character))
        };
        let place: GamePlace = {
            let places = places.clone();
            Rc::new(move |zone, slot| places.slot(zone, slot))
        };
        named.use_game(Some(sprite), Some(place));
        Gen6Profile {
            memory: cache,
            names: names.clone(),
            census: RefCell::new(census.then(String::new)),
            ways,
            layouts,
            apps,
            anchors: RefCell::new(Anchors::new(memory.clone(), game.anchors.clone())),
            anchored: Cell::new(0.0),
            dialogue: RefCell::new(dialogue),
            lists,
            choices,
            screen,
            files,
            cursors: RefCell::new(cursors),
            battlers,
            gauges,
            battle,
            opponents,
            party_beeped,
            tones: beeping,
            speaker,
            field,
            places,
            classes: classes.into_iter().collect(),
            own: alone,
            subscreens,
            contexts,
            buttons,
            party_icons,
            partners,
            registered,
            occupants,
            sources,
            chasing: Cell::new(0),
            chasing_details: Cell::new(false),
            cursor_said: Cell::new(false),
            map_cursor: RefCell::new(map_cursor),
            footsteps,
            navigator: RefCell::new(navigator),
            log,
            speech,
            voices,
            listed: RefCell::new(None),
            relist: Cell::new(false),
            chunks: RefCell::new(Vec::new()),
            seen: RefCell::new(Vec::new()),
            on_bike: Cell::new(false),
            turn: Cell::new(0),
            facing: Cell::new(None),
            tone,
            quiet: Cell::new(0.0),
            save_language: RefCell::new(None),
            announcements,
            screen_announcer: RefCell::new(screen_announcer),
            scenes,
            habitat,
            claims,
            zone: Cell::new(None),
            unheard: RefCell::new(HashSet::new()),
            speaker_wanted: Cell::new(false),
            after_forget: AfterForget::default(),
            moving: Cell::new(false),
            moved: Cell::new(None),
            holding: Cell::new(None),
            waiting: Cell::new(false),
            waited: Cell::new(None),
            managers,
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
        self.own.track(found)?;
        self.subscreens.track(&found.heap, adding)?;
        self.partners.borrow_mut().track(&found.heap, adding)?;
        self.occupants.borrow_mut().track(&found.heap);
        self.registered.borrow_mut().track(&found.heap);
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
        if !found.partial
            && let Some(language) = mystatus::language(found)
        {
            *self.save_language.borrow_mut() = Some(language.to_string());
        }
        if !found.partial
            && let Some(player) = mystatus::player(found)
        {
            *self.places.player.borrow_mut() = Some(player);
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
        // A number beside a bar on the top screen is an enemy's level only in a battle: the
        // summary's Exp. Points were taken for one whose gauge no sweep finds, and a whole sweep
        // was asked for every poll, 1.1 s each with nothing read meanwhile, 25 in 27 s (or_menus).
        let battle = self.apps.borrow().running.as_deref() == Some(BATTLE)
            && self.battle.borrow_mut().sweep_wanted();
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

    /// Whether the heap's whole sweep waits: while a subscreen's round would lose a ball to
    /// the second it takes (`Subscreens.holding`), while a cursor moves, until it has held
    /// still `STILL` or held the sweep off `HELD_MOST`, and for a cursor yet to come as its
    /// screen opens, `HELD_MOST` at most, counted from its app's start: a cursor that moved
    /// just before had begun the other count already (or_menus).
    pub fn sweep_held(&self, now: f64) -> bool {
        if self.moving.get() {
            self.moved.set(Some(now));
            self.moving.set(false);
        }
        if !self.waiting.get() {
            self.waited.set(None);
        } else if self.waited.get().is_none() {
            self.waited.set(Some(now));
        }
        let held = self.subscreens.holding()
            || self
                .waited
                .get()
                .is_some_and(|waited| now - waited < HELD_MOST);
        if self.moved.get().is_none_or(|moved| now - moved >= STILL) {
            self.holding.set(None);
            return held;
        }
        if self.holding.get().is_none() {
            self.holding.set(Some(now));
        }
        held || now - self.holding.get().expect("set above") < HELD_MOST
    }

    /// Whether another app has started since the last poll: the field, the Bag, the Pokédex, a
    /// battle. One read a poll, and the moment to sweep for the new screen's objects.
    pub fn poll_app(&self) -> Result<bool> {
        let was = self.apps.borrow().running.clone();
        let started = phase(&*self.memory, "app", || self.apps.borrow_mut().poll())?;
        let Some(started) = started else {
            self.anchor(false)?;
            return Ok(false);
        };
        (self.log)(&format!("app: {}", demangle(&started)));
        self.after_forget.started(was.as_deref(), &started);
        self.anchors.borrow_mut().started();
        self.anchor(true)?;
        self.dialogue.borrow_mut().forget_hosts();
        // A screen built again where it was may load its archive elsewhere: the PokéNav's came
        // back from the PlayNav's tutorial in another, and its pictures went unnamed (movie24).
        self.files.look_again();
        // A battle's sides are said again on a key only while it lasts.
        self.announcements.borrow_mut().forget_sides();
        self.subscreens.started();
        if started == BATTLE {
            self.battle.borrow_mut().begin();
            if let Some(opponents) = &self.opponents {
                opponents.borrow_mut().begin();
            }
        }
        if let Some(scenes) = &self.scenes {
            scenes.borrow_mut().started(
                &demangle(&started),
                self.zone.get(),
                &mut self.narrate(),
            )?;
            if DEMOS.contains(&started.as_str()) {
                scenes.borrow_mut().demo(None, None, &mut self.narrate())?;
            }
        }
        for app in self.own.apps() {
            app.started();
        }
        Ok(true)
    }

    /// Follow the running app's chains, `ANCHOR_EVERY` or `at_once`, and hand the readers
    /// what they lead to that is new (`gen6.anchors`); on the field, its zone's data, the zone
    /// table and the field map too (`FieldReader.anchor`).
    fn anchor(&self, at_once: bool) -> Result<()> {
        let now = py::time::monotonic();
        if !at_once && now - self.anchored.get() < ANCHOR_EVERY {
            return Ok(());
        }
        self.anchored.set(now);
        let (running, process) = {
            let apps = self.apps.borrow();
            (apps.running.clone(), apps.process)
        };
        let anchored = phase(&*self.memory, "anchors", || {
            let anchored = self
                .anchors
                .borrow_mut()
                .poll(running.as_deref(), process)?;
            if running.as_deref() == Some(FIELD)
                && let Some(process) = process
            {
                let taken = self.field().anchor(process)?;
                if !taken.is_empty() {
                    self.relist.set(true);
                    (self.log)(&format!("anchors: the field's {}", taken.join(", ")));
                }
            }
            Ok(anchored)
        })?;
        if let Some(anchored) = anchored {
            let names: Vec<&String> = anchored.heap.values().collect();
            let counted: Vec<String> = most_common(&names)
                .into_iter()
                .map(|(name, count)| format!("{count} {}", demangle(&name)))
                .collect();
            (self.log)(&format!("anchors: {}", counted.join(", ")));
            self.track(&anchored)?;
        }
        Ok(())
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
        let texts: Vec<&str> = shown
            .iter()
            .filter_map(|page| match page {
                Shown::Page(page) => Some(page.text.as_str()),
                Shown::Text(_) => None,
            })
            .collect();
        self.own.heard(&texts)?;
        self.after_forget.paged(&shown);
        let said = self.announcements.borrow_mut().pages(&shown)?;
        if said && let Some(opponents) = &self.opponents {
            opponents.borrow_mut().paged();
        }
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
    pub fn poll_menus(&self) -> Result<bool> {
        let focuses = phase(&*self.memory, "lists", || self.lists.borrow_mut().poll())?;
        let mut screen_polled = false;
        if focuses.iter().any(|focus| focus.opened) {
            // The question comes before its prompt: "YES, 1 of 2" was heard 1.6 s before "So
            // you're wee?" while a new window waited for a sweep. One is found by its lines as it
            // types now (spike 60), and a sweep of the heap here held every other reader up a
            // second at each list and found no window the dialogue reader lacked (spike 300).
            self.poll_dialogue()?;
            // A question the screen draws itself, such as "Would you like to make these changes?",
            // comes first too, without waiting for it to hold.
            self.poll_screen_with(false, false)?;
            screen_polled = true;
            // The answer is the player's to give now, and the screen behind is up.
            self.after_forget.answered();
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
        if chosen.iter().any(|focus| focus.opened) {
            self.after_forget.answered();
        }
        // X's opening draws its boy or girl and their looks as models, with no word.
        if let Some(opening) = self.own.focus()? {
            self.announcements
                .borrow_mut()
                .focus(&opening, "opening", false);
        }
        // A cursor source moved: the screen is polled now, once or each tick until its word is
        // said (`Chase`).
        let chase = phase(&*self.memory, "rows", || {
            self.sources
                .borrow_mut()
                .moved(&|| self.cursors.borrow().windows())
        })?;
        if chase.any(Chase::ROW | Chase::DETAILS) {
            self.chasing.set(rows::CHASE);
        }
        if chase.any(Chase::DETAILS) {
            self.chasing_details.set(true);
        }
        let turned = chase.any(Chase::ONCE);
        self.moving.set(self.moving.get() || turned);
        self.waiting.set(chase.any(Chase::WAIT));
        if (self.chasing.get() != 0 || turned) && !screen_polled {
            let gathering = self.screen_announcer.borrow().gathering();
            self.poll_screen()?;
            let mut said = self.cursor_said.get();
            if self.chasing_details.get() {
                said = (said || gathering) && !self.screen_announcer.borrow().gathering();
            }
            self.chasing.set(if said {
                0
            } else {
                (self.chasing.get() - 1).max(0)
            });
            self.chasing_details
                .set(self.chasing_details.get() && self.chasing.get() != 0);
            screen_polled = true;
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
        let polled = phase(&*self.memory, "screen", || {
            // The field's layouts come back where they were when a menu screen closes, and a
            // layout found before it was built is taken in once it is.
            let revived = self.layouts.revive()?;
            let works_of: IndexMap<i64, i64> = self
                .layouts
                .works()
                .into_iter()
                .map(|(work, root)| (root, work))
                .collect();
            let put_away = self.sources.borrow_mut().put_away(&works_of)?;
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
            let party = match &self.opponents {
                None => None,
                Some(opponents) => opponents.borrow_mut().poll()?,
            };
            self.habitat.borrow_mut().poll(on_screen)?;
            let listed = self.own.after(on_screen)?;
            self.map_cursor.borrow_mut().poll()?;
            let played = {
                let mut navigator = self.navigator.borrow_mut();
                let touch = navigator
                    .touch
                    .as_mut()
                    .expect("the gen 6 navigator is given touch targets");
                self.subscreens.poll(touch)?
            };
            let found = self.cursors.borrow_mut().poll(on_screen)?;
            let polled =
                self.sources
                    .borrow_mut()
                    .poll(&screen, found, self.chasing.get() != 0, &|| {
                        self.battle.borrow().shiny()
                    })?;
            Ok((screen, sides, party, listed, played, polled))
        })?;
        let (screen, sides, party, listed, played, polled) = polled;
        self.moving
            .set(self.moving.get() || !polled.cursors.items.is_empty());
        self.cursor_said.set(!polled.cursors.items.is_empty());
        for (logged, line, category) in &played {
            (self.log)(&format!("{logged}: {line}"));
            if let Some(category) = category {
                self.speech.say(line, *category);
            }
        }
        for (logged, line) in &polled.before {
            (self.log)(&format!("{logged}: {line}"));
            self.speech.say(line, Category::Menu);
        }
        let mut labels = self.lists.borrow().labels();
        labels.extend(self.announcements.borrow_mut().recent_labels());
        if with_choices {
            labels.extend(self.choices.borrow().labels());
        }
        let mut heard = self.claims.poll(&screen, &polled, &labels)?;
        let mut cursors = polled.cursors;
        let mut title = polled.title;
        if self.after_forget.back() {
            // Back from Move to forget, the Bag or the Move Reminder is built again as the player
            // left it, under the game's question or its news of the move, and its opening is
            // left unsaid: its stats, moves and pocket were read between "Give up trying to make
            // Altaria remember a move?" and its YES (or_move_reminder, a player's log of the
            // Bag). Its cursors still count as said for the chase.
            heard.texts.retain(|text| text.changed || !text.with_layout);
            cursors.items.retain(|item| !item.opened);
            title = None;
        }
        let (cursors, heard) = self.sources.borrow().said(cursors, heard);
        // O-Power's Active panel turns between its field and battle powers by itself, as the
        // PSS's profile turns its counts (or_pss_o_power): the subscreens say where.
        let heard = Heard {
            turns: self.subscreens.turns(),
            ..heard
        };
        // A layout built again where it was is the one that was there, and a menu stays a menu:
        // the battle's move list, forgotten while it was gone, was read out whole the moment it
        // showed.
        let mut roots = self.layouts.roots();
        roots.extend(self.layouts.buried());
        let subject = || self.sources.borrow().subject();
        self.screen_announcer.borrow_mut().say(
            &screen,
            &cursors,
            &heard,
            Some(&roots),
            Some(&subject),
            Some(&polled.chosen),
            title.as_deref(),
        )?;
        for (logged, line) in &polled.after {
            (self.log)(&format!("{logged}: {line}"));
            self.speech.say(line, Category::Screen);
        }
        // After the screen's words, which name the place first: the Pokémon Distribution's.
        for (logged, line) in &listed {
            (self.log)(&format!("{logged}: {line}"));
            self.speech.say(line, Category::Screen);
        }
        // A description after the screen's words, as it follows what it describes, and what is
        // only logged beside it: Tile Puzzle's picture. Read outside the screen's phase, which
        // would fetch its pages every poll after.
        for (logged, line, category) in self.subscreens.describe()? {
            if category == Some(Category::Scene) {
                self.announcements.borrow_mut().narrate(&line);
            } else if category.is_none() {
                (self.log)(&format!("{logged}: {line}"));
            }
        }
        // After the screen's words, and once a cursor's details are said, so they come last:
        // ORAS's contest conditions.
        let under: Vec<String> = cursors
            .under
            .iter()
            .flatten()
            .map(|piece| piece.text.clone())
            .collect();
        let waiting = self.screen_announcer.borrow().gathering();
        self.own
            .last(&under, &heard.heard, &screen.settling, waiting)?;
        if let Some(party) = party {
            (self.log)(&format!("battle party: {party}"));
            if !self.party_beeped {
                self.speech
                    .say(&own("battle-party", &[("count", &party)]), Category::Battle);
            } else if let Some(tones) = &self.tones {
                beeps(tones, party);
            }
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

    /// Answer a navigation key, by name: the screen's own first, while one with keys of its own
    /// runs (`context`), in every mode, Shift falling back there before anywhere else
    /// (`keys.bound`): Super Training's charge holds Shift, and R under it was the battle's
    /// `shift+r`, the enemy's side, where the round's numbers were wanted.
    pub fn press(&self, key: &str) -> Result<()> {
        phase(&*self.memory, "keys", || {
            let context = self.context()?;
            let action = context.and_then(|context| {
                self.contexts
                    .get(context)
                    .and_then(|keys| shifted(keys, key))
            });
            let Some(action) = action else {
                return self.navigator.borrow_mut().press_key(key);
            };
            let said = {
                let mut navigator = self.navigator.borrow_mut();
                let touch = navigator
                    .touch
                    .as_mut()
                    .expect("the gen 6 navigator is given touch targets");
                self.subscreens.press(&action, touch)?
            };
            if !said.is_empty() {
                (self.log)(&format!("key: {said}"));
                self.speech.say(&said, Category::Key);
            }
            Ok(())
        })
    }

    /// The table of the keys file of the screen running now, where it has keys of its own:
    /// Berry Picker's (`Subscreens.context`).
    pub fn context(&self) -> Result<Option<&'static str>> {
        self.subscreens.context()
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
    /// And what a subscreen follows every tick: Head It's balls, while its cue is on.
    pub fn poll_pointers(&self) -> Result<()> {
        if self.cursors.borrow().sampling() {
            phase(&*self.memory, "pointers", || {
                self.cursors.borrow_mut().sample(&*self.memory)
            })?;
        }
        if self.subscreens.ticking() {
            phase(&*self.memory, "ticked", || self.subscreens.tick())?;
        }
        Ok(())
    }

    /// Keep any name the player has just given. Costs one empty queue read a poll, and the
    /// dialog it answers has been up on a thread of its own while the game kept running. And
    /// press again a key that waited for the zone the player arrived in to be set up.
    pub fn poll_renames(&self) -> Result<()> {
        self.navigator.borrow_mut().poll_renames()?;
        let held = self.navigator.borrow().held();
        if let Some(action) = held {
            phase(&*self.memory, "keys", || {
                self.navigator.borrow_mut().press(&action)
            })?;
        }
        Ok(())
    }

    /// Poll where the player is, and log what is around them when it changes.
    ///
    /// The listing reads every model in the pool, so it is made only on a change of zone and
    /// after each sweep, not every poll.
    pub fn poll_field(&self) -> Result<()> {
        if let Some(scenes) = &self.scenes {
            scenes.borrow_mut().poll(&mut self.narrate())?;
        }
        if let Some((app, process)) = own_running(&self.apps, &self.own.apps()) {
            return phase(&*self.memory, app.name(), || app.poll(process));
        }
        let field = self.field();
        // `None` is the Python's `return` from inside the `with`.
        let polled = phase(&*self.memory, "field", || -> Result<Option<Field>> {
            let polled = field.poll()?;
            // The field's voices, in this order: gen 6's own, the Dowsing Machine's lights, then
            // the game's (`FieldReader.voices`), ORAS's decorating cursor. Each is asked even
            // while nothing says where the player is, the lights needing two polls alike.
            for voice in self.voices.iter().chain(field.voices.iter()) {
                if let Some(said) = voice(polled.as_ref())? {
                    (self.log)(&format!("field {}", said.logged));
                    self.speech.say(&said.words, said.category);
                }
            }
            let Some(mut at) = polled else {
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
                        let _ = tones.play(across, down, triangle);
                    }
                }
                self.facing.set(facing);
            }
            self.navigator.borrow_mut().swept(&at)?;
            if let Some((name, shown)) = self.places.arrived(at.zone)? {
                self.screen_announcer
                    .borrow_mut()
                    .place(&name, shown.as_deref());
            }
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
            // A watched feature, as ORAS's DexNav's find, shows and goes between sweeps, so it is
            // read every poll: its record logged as it changes, the field listed again as what is
            // up does, and a find the player went looking for guided to as it first shows. The
            // navigator knows whether touch mode is on.
            let seen = field.watch(at.zone)?;
            // `zip_longest(seen, self._seen, fillvalue=Sighting())`.
            let pairs: Vec<(Sighting, Sighting)> = {
                let was = self.seen.borrow();
                (0..seen.len().max(was.len()))
                    .map(|i| {
                        (
                            seen.get(i).cloned().unwrap_or_default(),
                            was.get(i).cloned().unwrap_or_default(),
                        )
                    })
                    .collect()
            };
            for (now, was) in &pairs {
                if now.record != was.record {
                    (self.log)(&format!(
                        "field {}",
                        now.record.as_deref().unwrap_or("None")
                    ));
                }
            }
            let finds: Vec<(Interactable, bool)> = pairs
                .iter()
                .filter(|(now, was)| now.find.is_some() && was.up.is_none())
                .filter_map(|(now, _)| now.find.clone().map(|find| (find, now.searched)))
                .collect();
            if pairs.iter().any(|(now, was)| now.up != was.up) {
                self.relist.set(true);
            }
            *self.seen.borrow_mut() = seen;
            if !self.relist.get()
                && let Some(listed) = &*self.listed.borrow()
                && listed.0 == at.zone
            {
                return Ok(None);
            }
            at = field.interactables(&at)?;
            for (find, searched) in &finds {
                self.navigator.borrow_mut().found(&at, find, *searched)?;
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
        if let Some(ways) = &self.ways {
            // A replay presses no key of pa3ds's, so what P says is logged for its tape's diff to
            // show (ADR-0041): Route 120's bridges taken for the pond under them went unheard for
            // 23 days (801d7b8). Outside any phase, which would read its pages every poll after.
            ways(&mut || {
                match self.navigator.borrow_mut().ways(&at) {
                    Ok(lines) => {
                        for line in lines {
                            (self.log)(&format!("field {line}"));
                        }
                    }
                    Err(error) if error.is(Kind::RpcError) => {
                        (self.log)(&format!("field ways: not read, {error}"));
                    }
                    Err(error) => return Err(error),
                }
                Ok(())
            })?;
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

    /// Find the layout files of every layout tracked since the last look, outside any phase,
    /// since each archive is read once (`LayoutFiles.read_archives`).
    fn read_archives(&self) -> Result<()> {
        self.files.read_archives(&self.layouts.works(), &self.names)
    }
}

/// `Gen6Profile._pane_named`: the file a word's layout was built from and its pane's name there,
/// or None where the file is not known: by the layout's tree as last walked, reading nothing.
fn pane_named(
    layouts: &Layouts,
    files: &LayoutFiles,
    piece: &ScreenText,
) -> Option<(String, String)> {
    let names = layouts.pane_names(piece.root);
    let file = files.matching(&names)?;
    let name = layouts
        .tree(piece.root)
        .keys()
        .zip(&names)
        .find(|(pane, _)| **pane == piece.r#box)
        .map_or(String::new(), |(_, name)| name.clone());
    Some((file.name.clone(), name))
}

/// `Gen6Profile._field_process`: the field's process while the field runs, None otherwise: what
/// the Dowsing Machine's lights are read through.
fn field_process(apps: Rc<RefCell<RunningApp>>) -> Rc<dyn Fn() -> Option<i64>> {
    Rc::new(move || {
        let apps = apps.borrow();
        if apps.running.as_deref() == Some(FIELD) {
            apps.process
        } else {
            None
        }
    })
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

/// Back from Move to forget: the pages the game has said since Move to forget gave way to the
/// screen it was opened from, the Bag or the Move Reminder, while that screen opening again is
/// left unsaid (`Gen6Profile.poll_screen`). The question or the news of the move is the first
/// page, and a page after it is said once the screen behind is up; so is a list or a choice
/// opening, the player's to answer.
#[derive(Default)]
pub(crate) struct AfterForget {
    /// The pages said since, or None when not back from Move to forget.
    pages: Cell<Option<i64>>,
}

impl AfterForget {
    /// Is the screen Move to forget was opened from opening again?
    pub(crate) fn back(&self) -> bool {
        self.pages.get().is_some()
    }

    /// Another app has started after `was`.
    pub(crate) fn started(&self, was: Option<&str>, started: &str) {
        if was == Some(rows::FORGET) && started != FIELD {
            self.pages.set(Some(0));
        } else if started == FIELD || started == rows::FORGET {
            self.pages.set(None);
        }
    }

    /// The dialogue reader has read these.
    pub(crate) fn paged(&self, shown: &[Shown]) {
        if let Some(pages) = self.pages.get() {
            let pages = pages
                + shown
                    .iter()
                    .filter(|page| matches!(page, Shown::Page(_)))
                    .count() as i64;
            self.pages.set(Some(pages));
            if pages > 1 {
                self.pages.set(None);
            }
        }
    }

    /// A list or a choice has opened: the answer is the player's, and the screen is up.
    pub(crate) fn answered(&self) {
        self.pages.set(None);
    }
}

/// What the navigator reads the map through: what one of the game's own apps lists while it
/// runs (`OwnApp.listed`), and the field's while the field runs or its map is still loaded under
/// another screen, where the player cannot walk (`Field.walkable`). A battle or a scene unloads
/// the map and runs with the field's models still in memory as they were: after Groudon's
/// capture the keys listed it in the cave left behind. A map loaded is not
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

/// `Gen6Profile._recall`: the word for what a scene names as it is said: `memory`, the Pokémon
/// the player's own save keeps for the credits' moment (spike 222), None where it keeps none;
/// `yours`, the nickname the player gave that Pokémon, None where they gave none or no longer
/// have it; `sprite`, what a speaker drawn as it is called.
fn recall(named: Rc<Names>, own: Rc<dyn Own>) -> Recall {
    Rc::new(move |kind: &str, number: i64| {
        if kind == "sprite" {
            return Ok(named.sprite(number));
        }
        own.recall(kind, number)
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

    fn poll_menus(&self) -> Result<bool> {
        Gen6Profile::poll_menus(self)
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

    fn context(&self) -> Result<Option<&'static str>> {
        Gen6Profile::context(self)
    }

    fn sweep_wanted(&self) -> Result<bool> {
        Gen6Profile::sweep_wanted(self)
    }

    fn sweep_held(&self, now: f64) -> bool {
        Gen6Profile::sweep_held(self, now)
    }
}
