//! Touch mode's targets: the touch areas drawn on the bottom screen, found, placed and named, in
//! reading order, on the screens that need a touch (`offered`), for touch mode to walk
//! (`common.touch`).
//!
//! A touch area is named by, in this order (spike 171; `TouchTargets._names`):
//!
//! 1. **the words in its button**: the screen text inside the pane it hangs from, as the AreaNav's
//!    window onto the map is said by the place it shows;
//! 2. **the game's word for the picture drawn over it** (`images`), by the image it draws or the
//!    one its layout file gives it: the PokéNav's app buttons are drawn with
//!    `pmn_screen_zukan.bclim` and the like, and the ROM's text has "DexNav" for them; the
//!    AreaNav's tabs, Places to Berries, draw images of their words that come from no archive
//!    pa3ds reads, and their file names them `map_navi01` to `05` (movie17); or what the game
//!    fills a picture with as it runs: the AreaNav's Pokémon Distribution draws each Pokémon's
//!    icon, named by its species (`oras.subscreen.distribution`, spike 178);
//! 3. **the words of the button drawing the same picture beside words**: the shortcut bar under
//!    the AreaNav draws the X menu's six icons alone, and the X menu draws each beside its word,
//!    POKÉMON, POKÉDEX, BAG, the player's name, SAVE and OPTIONS, hidden until it opens;
//! 4. **what it was called the last time it could be named**, while its button holds the same
//!    panes: the DexNav's button draws a magnifier from another archive while a hidden Pokémon is
//!    near (spike 171);
//! 5. **where it is, as a slot** (`slots.slot`), as the Box's pointer says it: "BOX 1, row 1,
//!    column 1, empty", "Party 2", a mark's "●, on" (movie22);
//! 6. **the words of the picture's own name** (`pictures.words`), a bare "button" in the top bar's
//!    corners and the PSS menu's arrow being the page arrows, previous and next; where the data has
//!    no word for the picture, **the game's words written under it**, as a label under its icon,
//!    come first: the PSS's menu, "Battle" under `pss_taisen_icon` (or_pss_singleplayer); or with
//!    none under it, beside it on its row, as a box's words are: "Add to Favorites List".
//!
//! A touch area none of these names is left out, and the log says so once. So is the AreaNav's
//! map, whose every place is a touch area (spike 152), on the field: it is drawn zoomed in and cut
//! to a window, and its places beyond the window are touch areas on screen all the same, 65 of
//! them over the DexNav's button and the shortcut bar. In the AreaNav itself a tap on a place
//! chooses it, the top screen saying its name and what it is like, and each is offered by the
//! place the ROM names it (`places`), under every other button the AreaNav draws over the map
//! (or_area_nav).
//!
//! A button a manager takes taps on by a window or a picture holding its words, with no touch area,
//! is one too: the PSS Settings' Yes and No, the PSS profile's messages (`Buttons.on`). And the PSS
//! draws its windows itself, off the field's draw list (`Layouts.listed`), each over a backdrop as
//! big as the screen, at the layer of its page under them, which is on the list: a target of a
//! layout on the list, at that layer or under it, is left out while such a window is up, and so is
//! one of a window under a window built after it, the Shout-Out's Yes and No over the menu
//! (`Layouts.order`, spike 312). A touch area under a bar its layout draws over it is hidden, the
//! O-Power list's last row under its gauge; and a button switched off that the game greys, with an
//! animation to grey it and a picture faded, is offered with "unavailable" after its name
//! (`said_unavailable`, `greyed`).
//!
//! Bound by:
//! - ADR-0009: targets only on the screens that need a touch; a button the game shows greyed is
//!   offered, said unavailable.
//! - ADR-0037: the DexNav's icons are offered and said caught or not, though the game switches off
//!   those not caught.
//! - ADR-0013: the words naming a target, a button's own or the label under its icon, are known
//!   (`button_words`), for the field's bottom screen and the PSS's apps to leave them to touch mode,
//!   and stay known by their layout's file and their pane's name; a button switched off and not
//!   offered names no target. Where touch mode offers none, a button's words are its touch area's
//!   alone, for the Pokédex to leave them to its cursors.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{f32, u32};
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::screen::ScreenText;
use crate::games::common::screen_order::{distinct, in_rows, join};
use crate::games::common::touch::{
    After, FileOf, HEIGHT, ImageOf, OnScreen, Target, TextOf, WIDTH, rows,
};
use crate::games::common::{lyt, pictures, slots};
use crate::py::{self, Log, Result};

/// How far down the screen the top bar reaches, whose corners hold the page arrows (spike 198).
pub const TOP_BAR: i64 = 30;
/// How far under a touch area's bottom, in the layout's units, the words labelling its picture may
/// start (`TouchTargets._label`): a glyph. The PSS's menu starts "Battle" 10 above its icon's
/// bottom.
pub const LABEL_GAP: f64 = 12.0;
/// The map pictures whose touch areas are places: Omega Ruby's AreaNav's (spike 152) and X's
/// Town Map's. Not knowing X's, touch mode named its places as other buttons are, by its panes'
/// groups, "Party 2" (x_townmap). A `frozenset`, sorted as the tests walk it.
pub const MAPS: [&str; 2] = ["poke_navi_map.bclim", "townmap_bg1.bclim"];

/// `Callable[[int], str | None]`: what a picture the game fills in as it runs draws. It reads
/// memory.
pub type PictureName = Rc<dyn Fn(i64) -> Result<Option<String>>>;
/// `Callable[[], set[str]]`: the labels of the list windows on screen.
pub type Lists = Rc<dyn Fn() -> HashSet<String>>;
/// `Callable[[], set[int]]`: the works of the layouts whose buttons take no tap. It reads memory.
pub type Idle = Rc<dyn Fn() -> Result<HashSet<i64>>>;
/// `Callable[[], set[tuple[int, str]]]`: (work, pane name) of the buttons switched off. It reads
/// memory.
pub type Off = Rc<dyn Fn() -> Result<HashSet<(i64, String)>>>;
/// `Callable[[int], bool]`: whether a picture's button is offered though switched off. It reads
/// memory.
pub type SaidOff = Rc<dyn Fn(i64) -> Result<bool>>;
/// `Callable[[], Mapping[str, str]]`: the AreaNav's map's touch areas by pane name -> the place
/// each is. It reads memory.
pub type Places = Rc<dyn Fn() -> Result<Rc<IndexMap<String, String>>>>;
/// `Callable[[], list[Target]]`: targets no touch area draws. It reads memory.
pub type Gestures = Rc<dyn Fn() -> Result<Vec<Target>>>;
/// `Callable[[], Mapping[int, str]]`: the pictures a screen offers as targets of their own, by
/// pane with their names. It reads memory.
pub type Pictured = Rc<dyn Fn() -> Result<IndexMap<i64, String>>>;

/// `TouchTargets.__init__`'s keyword arguments, each as the Python defaults it: set what differs
/// with `..Options::default()`.
#[derive(Default)]
pub struct Options {
    /// None is `{}`.
    pub images: Option<IndexMap<String, String>>,
    pub picture_name: Option<PictureName>,
    pub lists: Option<Lists>,
    pub idle: Option<Idle>,
    pub gestures: Option<Gestures>,
    pub pictured: Option<Pictured>,
    pub off: Option<Off>,
    pub on: Option<Off>,
    pub said_off: Option<SaidOff>,
    pub places: Option<Places>,
    pub offered: Option<Rc<dyn Fn() -> bool>>,
    pub caught: HashSet<String>,
    pub said_caught: Option<Rc<dyn Fn(&str) -> String>>,
    pub said_unavailable: Option<Rc<dyn Fn(&str) -> String>>,
    pub greyed: Option<Off>,
}

/// The bottom screen's touch targets, in reading order.
pub struct TouchTargets {
    layouts: Rc<Layouts>,
    on_screen: OnScreen,
    /// A TextBox's words, shown or not.
    text_of: TextOf,
    /// The file of its layout's archive a picture draws now, or None.
    image_of: ImageOf,
    /// A layout's root -> the file it was built from, or None where it is not known.
    file_of: FileOf,
    log: Log,
    images: IndexMap<String, String>,
    /// The game's word for what a picture the game fills in as it runs draws, or None.
    picture_name: Option<PictureName>,
    /// The labels of the list windows on screen: a Yes and No the player must answer first.
    lists: Option<Lists>,
    /// The works of the layouts whose buttons take no tap now (`Buttons.idle`).
    idle: Option<Idle>,
    /// Targets a screen offers at points no pane of the bottom screen places: Pokémon-Amie's
    /// Pokémon, stroked by sliding over it (spike 200), a round's goals, shot from the bottom
    /// screen, and Berry Picker's Pokémon.
    gestures: Option<Gestures>,
    /// The pictures a screen offers as targets of their own, where no touch area holds them,
    /// by pane with their names: Super Training's training bags and a message's ▼ waiting for a
    /// tap. Placed and left out as a touch area is (`_pictures`).
    pictured: Option<Pictured>,
    /// (work, pane name) of the buttons switched off now, which take no tap (`Buttons.off`).
    off: Option<Off>,
    /// (work, pane name) of the buttons switched on now (`Buttons.on`): a window holding its
    /// words with no touch area of its own is a touch area, as the PSS Settings' Yes and No
    /// are.
    on: Option<Off>,
    /// Whether a picture's button is offered though switched off: the DexNav switches off the
    /// icon of a species not caught, which it cannot search for, and the species is said all the
    /// same (ADR-0037).
    said_off: Option<SaidOff>,
    /// The AreaNav's map's touch areas by pane name -> the place each is, while the map is the
    /// screen's and not the field's window onto it; none otherwise.
    places: Option<Places>,
    /// Whether the screen up now needs a touch: none is offered on one the game's buttons work
    /// (`gen6.profile.TOUCHED`); None offers them everywhere.
    offered: Option<Rc<dyn Fn() -> bool>>,
    /// Images a button named by the game's word for its picture shows only while what it
    /// stands for is caught: the DexNav's magnifier holds a Poké Ball over a hidden Pokémon
    /// caught, and over one never caught draws the same magnifier with none (spike 195).
    caught: HashSet<String>,
    /// A button's words said caught.
    said_caught: Option<Rc<dyn Fn(&str) -> String>>,
    /// A button's words said greyed; None offers no greyed button.
    said_unavailable: Option<Rc<dyn Fn(&str) -> String>>,
    /// (work, pane name) of the buttons switched off that have an animation to grey them
    /// (`Buttons.greyed`), offered where a picture of theirs is faded (`_faded`).
    greyed: Option<Off>,
    /// A layout's root -> the names of its buttons that were a list's labels while it was up.
    listed: IndexMap<i64, HashSet<String>>,
    /// Touch area -> what it was last called (rule 4), and the panes its button held then.
    named: IndexMap<i64, (String, HashSet<i64>)>,
    unnamed: HashSet<i64>,
    /// (layout file, pane name) of every TextBox found a button's words (`button_words`).
    worded: HashSet<(String, String)>,
    /// A layout's root -> the panes of other classes than a touch area's that its manager has
    /// listed as buttons switched on (`_windows`).
    windowed: IndexMap<i64, HashSet<i64>>,
    groups: IndexMap<i64, Vec<i64>>,
    /// A walk's buttons and panes' middles (None where not laid out), each found once: the Box's
    /// touch areas hanging from its root measured its every pane each, 34,000 times a walk
    /// (movie22).
    middles: IndexMap<i64, Option<(f64, f64)>>,
}

impl TouchTargets {
    pub fn new(
        layouts: Rc<Layouts>,
        on_screen: OnScreen,
        text_of: TextOf,
        image_of: ImageOf,
        file_of: FileOf,
        log: Log,
        options: Options,
    ) -> Self {
        let Options {
            images,
            picture_name,
            lists,
            idle,
            gestures,
            pictured,
            off,
            on,
            said_off,
            places,
            offered,
            caught,
            said_caught,
            said_unavailable,
            greyed,
        } = options;
        TouchTargets {
            layouts,
            on_screen,
            text_of,
            image_of,
            file_of,
            log,
            images: images.unwrap_or_default(),
            picture_name,
            lists,
            idle,
            gestures,
            pictured,
            off,
            on,
            said_off,
            places,
            offered,
            caught,
            said_caught,
            said_unavailable,
            greyed,
            listed: IndexMap::new(),
            named: IndexMap::new(),
            unnamed: HashSet::new(),
            worded: HashSet::new(),
            windowed: IndexMap::new(),
            groups: IndexMap::new(),
            middles: IndexMap::new(),
        }
    }

    /// Every touch area drawn on the bottom screen that can be named, in reading order, on a
    /// screen that needs a touch.
    pub fn targets(&mut self) -> Result<Vec<Target>> {
        if let Some(offered) = &self.offered
            && !offered()
        {
            return Ok(Vec::new());
        }
        Ok(self.walk()?.0)
    }

    /// Every touch area drawn on the bottom screen that can be named, in reading order; a
    /// layout's root -> the TextBoxes whose words named its buttons; and the touch areas shown
    /// and left out as switched off (`button_words`). What is offered is decided here, where each
    /// lies and what lies over it, and each is called what `_names` calls it.
    #[allow(clippy::type_complexity)]
    fn walk(&mut self) -> Result<(Vec<Target>, IndexMap<i64, HashSet<i64>>, HashSet<i64>)> {
        let layouts = self.layouts.clone();
        let shape = layouts.shape;
        self.groups.clear();
        self.middles.clear();
        let mut words: IndexMap<i64, ScreenText> = IndexMap::new();
        for piece in (self.on_screen)() {
            if !piece.top {
                words.insert(piece.r#box, piece);
            }
        }
        let mut found: Vec<Target> = Vec::new();
        // Every touch area on the touch screen, named or not: (its layout's layer, x, y, width,
        // height).
        let mut areas: Vec<(i64, i64, i64, f64, f64)> = Vec::new();
        let idle = match &self.idle {
            Some(idle) => idle()?,
            None => HashSet::new(),
        };
        let off = match &self.off {
            Some(off) => off()?,
            None => HashSet::new(),
        };
        let on = match &self.on {
            Some(on) => on()?,
            None => HashSet::new(),
        };
        let greyable = match &self.greyed {
            Some(greyed) if self.said_unavailable.is_some() => greyed()?,
            _ => HashSet::new(),
        };
        let works: IndexMap<i64, i64> = layouts
            .works()
            .into_iter()
            .map(|(work, root)| (root, work))
            .collect();
        let mut boxes: IndexMap<i64, HashSet<i64>> = IndexMap::new();
        let mut unoffered: HashSet<i64> = HashSet::new();
        // Only a layout drawn on the bottom screen now: the field's stay on the list while an app
        // of their own runs, Pokémon-Amie's, which draws all its layouts itself.
        let mut listed: HashSet<i64> = HashSet::new();
        for root in layouts.roots() {
            if layouts.listed(root)?
                && let Some(shown) = layouts.placement(root)?
                && shown.shown
                && !shown.top
            {
                listed.insert(root);
            }
        }
        // (root, layer) of the layouts off the draw list that draw a backdrop (`_backdrop`).
        let mut covering: Vec<(i64, i64)> = Vec::new();
        let mut roots: Vec<i64> = layouts.roots().into_iter().collect();
        roots.sort();
        for root in roots {
            let placement = layouts.placement(root)?;
            let Some(placement) = placement else {
                continue;
            };
            if !placement.shown || placement.top || layouts.taken_off(root)? {
                continue;
            }
            let tree = layouts.tree(root);
            let layer = layouts.layer(root)?;
            // The PSS draws its windows itself, over its page on the field's draw list: its menu,
            // its Yes and No, its O-Power and its settings, each at the page's layer 1, and the
            // page's Friends, Acquaintances and Passersby were offered under every one, and a tap
            // on them did nothing (or_pss_singleplayer). A window built over another of them
            // covers it: the Shout-Out's Yes and No, over the menu.
            if !listed.is_empty() && !listed.contains(&root) && self.backdrop(root, &tree)? {
                covering.push((root, layer));
            }
            // A layout whose buttons the game has locked takes no tap however it is drawn: the
            // party's list under "Do what with Blaze?", washed out by a veil (spike 189).
            if works.get(&root).is_some_and(|work| idle.contains(work)) {
                continue;
            }
            // A button switched off takes no tap however it is drawn: Super Training's party
            // under its Use panel (spike 228). But for a picture said all the same: the DexNav's
            // Pokémon not caught were left out with it. And one the game shows greyed is offered
            // said so: the PSS's Battle, Trade and O-Power for another player, offline, were said
            // as screen text and could not be found among the targets (or_pss_singleplayer_2).
            let mut touches: Vec<i64> = Vec::new();
            let mut greyed: HashSet<i64> = HashSet::new();
            for (&area, kind) in &tree {
                if kind != lyt::BOUNDING_CLASS || !shown(&layouts, area)? {
                    continue;
                }
                let name = self.layouts.pane_name(area)?;
                if !works
                    .get(&root)
                    .is_some_and(|&work| off.contains(&(work, name)))
                    || self.said_though_off(root, area, &tree)?
                {
                    touches.push(area);
                    continue;
                }
                let name = self.layouts.pane_name(area)?;
                if works
                    .get(&root)
                    .is_some_and(|&work| greyable.contains(&(work, name)))
                    && self.faded(root, area, &tree)?
                {
                    touches.push(area);
                    greyed.insert(area);
                } else {
                    unoffered.insert(area);
                }
            }
            let names_on: HashSet<String> = on
                .iter()
                .filter(|(work, _)| Some(work) == works.get(&root))
                .map(|(_, name)| name.clone())
                .collect();
            touches.extend(self.windows(root, &tree, &names_on)?);
            if touches.is_empty() {
                continue;
            }
            let head = layouts.pane(root)?;
            let (root_x, root_y) = (f32(&head, shape.x)?, f32(&head, shape.y)?);
            let file = (self.file_of)(root);
            let mut drawn: IndexMap<i64, Option<String>> = IndexMap::new();
            for (&pane, kind) in &tree {
                if kind == lyt::PICTURE_CLASS && shown(&layouts, pane)? {
                    drawn.insert(pane, (self.image_of)(pane)?);
                }
            }
            if drawn
                .values()
                .any(|image| image.as_deref().is_some_and(|image| MAPS.contains(&image)))
            {
                found.extend(self.map_places(root, &touches, layer, root_x, root_y)?);
                continue;
            }
            let mut groups: IndexMap<i64, Vec<i64>> = IndexMap::new();
            for &touch in &touches {
                let group = self.group(root, touch, &tree)?;
                groups.insert(touch, group);
            }
            let bars = self.bars(root, &tree, root_x, root_y)?;
            let order: IndexMap<i64, i64> = if bars.is_empty() {
                IndexMap::new()
            } else {
                tree.keys()
                    .enumerate()
                    .map(|(at, &pane)| (pane, at as i64))
                    .collect()
            };
            // Touch area -> its middle in pixels, where no bar hides it.
            let mut seen: IndexMap<i64, (i64, i64)> = IndexMap::new();
            for &touch in &touches {
                let (x, y) = pixel(&layouts.pane(touch)?, &shape, root_x, root_y)?;
                // A bar drawn over a touch area's middle hides it: the O-Power list's seventh row
                // lies under its gauge until the list scrolls, and a tap on it did nothing
                // (or_pss_singleplayer_2).
                if !bars.iter().any(|&(bar, at, top, bottom)| {
                    at > order[&touch]
                        && top <= y as f64
                        && y as f64 <= bottom
                        && !groups[&touch].contains(&bar)
                }) {
                    seen.insert(touch, (x, y));
                }
            }
            let (names, spoken) = self.names(
                root,
                &tree,
                &touches,
                &groups,
                &seen,
                &words,
                &drawn,
                file.as_deref(),
                &greyed,
                &mut boxes,
            )?;
            // Touch area -> its middle in pixels and its size.
            let mut placed: IndexMap<i64, (i64, i64, f64, f64)> = IndexMap::new();
            let first = found.len();
            for (touch, name) in names {
                let (x, y) = seen[&touch];
                if !((0..WIDTH).contains(&x) && (0..HEIGHT).contains(&y)) {
                    continue;
                }
                let pane = layouts.pane(touch)?;
                let (width, height) = (f32(&pane, shape.width)?, f32(&pane, shape.height)?);
                areas.push((layer, x, y, width, height));
                placed.insert(touch, (x, y, width, height));
                if let Some(name) = name.filter(|name| !name.is_empty()) {
                    // Tapped in the middle of what is on screen of it: another player's delete
                    // in the PSS hangs half off the left edge, and a tap on its middle, x 0, did
                    // nothing (the PSS's multiplayer).
                    found.push(Target {
                        pane: touch,
                        name,
                        x: on_screen(x, width, WIDTH)?,
                        y: on_screen(y, height, HEIGHT)?,
                        row: f32(&pane, shape.y)? - root_y,
                        height,
                        layer,
                        root,
                        path: Vec::new(),
                        seconds: 0.0,
                        drop: None,
                        rub: None,
                        held: false,
                        after: After::default(),
                    });
                }
            }
            // A touch area hanging straight from the root is the screen's, and the buttons inside
            // it are their own: the BuzzNav's NEXT lies within one over its whole panel
            // (movie24).
            let mut screens: HashSet<i64> = HashSet::new();
            for &touch in placed.keys() {
                if u32(&layouts.pane(touch)?, lyt::PARENT)? == root {
                    screens.insert(touch);
                }
            }
            // And a screen's touch area whose middle is another's taps that other: the bar under
            // Amie's decorate screen's two tabs, offered at its middle, opened Poké Puffs.
            let kept: Vec<Target> = found
                .split_off(first)
                .into_iter()
                .filter(|target| {
                    !within_another(target.pane, &placed, &spoken, &screens)
                        && !(screens.contains(&target.pane) && on_another(target.pane, &placed))
                })
                .collect();
            found.extend(kept);
        }
        // The pictures a screen offers come after the touch areas: where one lies under a touch
        // area's pixel, the tap is the touch area's.
        if let Some(pictured) = &self.pictured {
            let offered = pictured()?;
            found.extend(self.pictures(&offered, &works, &idle, &off)?);
        }
        // A window covers the draw list whatever its layers: another player's challenge over
        // X's PSS hides the bars above and below the page, at layer 10 to its 1, and touch mode
        // offered the bar's POKÉMON to OPTIONS around its Yes and No, which nothing locked
        // (the PSS's multiplayer). Another window it covers only at its layer or under.
        if !covering.is_empty() {
            let mut kept: Vec<Target> = Vec::new();
            'found: for target in found {
                for &(root, layer) in &covering {
                    if root != target.root
                        && (listed.contains(&target.root)
                            || (layer >= target.layer
                                && layouts.order(root) > layouts.order(target.root)))
                    {
                        continue 'found;
                    }
                }
                kept.push(target);
            }
            found = kept;
        }
        // Two touch areas under one pixel are one target: a tap cannot tell them apart. The party
        // list lays a second over each Pokémon's plate, and each was said twice (movie22).
        let mut once: IndexMap<(i64, i64), Target> = IndexMap::new();
        for target in found {
            once.entry((target.x, target.y)).or_insert(target);
        }
        let mut found = uncovered(&once.into_values().collect::<Vec<_>>(), &areas);
        if let Some(gestures) = &self.gestures {
            found.extend(gestures()?);
        }
        if let Some(lists) = self.lists.clone() {
            let labels = lists();
            found = answered_first(&self.unlisted(&found, &labels), &labels);
        }
        Ok((
            rows(&found)
                .into_iter()
                .flatten()
                .map(|i| found[i].clone())
                .collect(),
            boxes,
            unoffered,
        ))
    }

    /// What each of a layout's touch areas is called, by the rules in their order (the
    /// module's): the words in its button and the game's word for its picture, for every one;
    /// then for each `seen`, no bar over it, the words of the button drawing the same picture,
    /// what it was last called, its place as a slot, and its picture's own words or the label
    /// under it. Gives each of those left in the walk, a slot behind another's not, by what it
    /// is called, None where nothing names it, which the log says once; and the words on screen
    /// in each button (`_within_another`). The TextBoxes whose words name a button go into
    /// `boxes` (`button_words`).
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn names(
        &mut self,
        root: i64,
        tree: &IndexMap<i64, String>,
        touches: &[i64],
        groups: &IndexMap<i64, Vec<i64>>,
        seen: &IndexMap<i64, (i64, i64)>,
        words: &IndexMap<i64, ScreenText>,
        drawn: &IndexMap<i64, Option<String>>,
        file: Option<&LayoutFile>,
        greyed: &HashSet<i64>,
        boxes: &mut IndexMap<i64, HashSet<i64>>,
    ) -> Result<(
        IndexMap<i64, Option<String>>,
        IndexMap<i64, HashSet<String>>,
    )> {
        let layouts = self.layouts.clone();
        let empty = IndexMap::new();
        let textures = file.map_or(&empty, |file| &file.textures);
        let mut names: IndexMap<i64, String> = IndexMap::new();
        let owned = own_parts(groups);
        let mut filled: Option<IndexMap<i64, String>> = None;
        // Touch area -> the words on screen in its button, or with none the game's word for
        // its picture: the Pokémon-Amie page's PLAY, DECORATE and SWITCH lie within the touch
        // area behind its menu, and are buttons of their own (spike 198). A button with words
        // keeps to them: the Secret Base's DexNav panel, words and an app's icon, was offered
        // as well as the window it lies in.
        let mut spoken: IndexMap<i64, HashSet<String>> = IndexMap::new();
        for &touch in touches {
            let group = &groups[&touch];
            // As a cursor reads its item: the X menu's words drawn twice in one place once,
            // "BAG BAG" (movie20), and a row left to right, so a move's PP is "35/35" where
            // each word once and in build order made "PP / 35" (movie22).
            let mut said = join(
                &distinct(in_rows(group.iter().filter_map(|p| words.get(p)).cloned())),
                false,
            );
            spoken.insert(
                touch,
                group
                    .iter()
                    .filter_map(|p| words.get(p))
                    .map(|piece| piece.text.clone())
                    .collect(),
            );
            // A button's words, however it is named: the O-Power panel's Use, Give and Cancel
            // came on screen as their buttons were named as they had been, and were said
            // (or_pss_singleplayer).
            if self.button(touch)? {
                boxes.entry(root).or_default().extend(
                    group
                        .iter()
                        .copied()
                        .filter(|p| tree.get(p).map(String::as_str) == Some(lyt::TEXTBOX_CLASS)),
                );
            }
            let mut image: Option<String> = None;
            for i in drawn_with(
                owned[&touch].iter().copied(),
                &|p| Ok(drawn.get(&p).cloned().flatten()),
                textures,
                &|p| self.layouts.pane_name(p),
            ) {
                if let Some(word) = self.images.get(&i?) {
                    image = Some(word.clone());
                    break;
                }
            }
            if let Some(shown_image) = image.clone()
                && !shown_image.is_empty()
                && said.is_empty()
            {
                spoken
                    .get_mut(&touch)
                    .expect("inserted above")
                    .insert(shown_image.clone());
                // By what it draws now, never its layout file's texture: the ball is there,
                // hidden, over a Pokémon not caught.
                if let Some(said_caught) = &self.said_caught
                    && owned[&touch].iter().any(|p| {
                        drawn
                            .get(p)
                            .and_then(Option::as_deref)
                            .is_some_and(|i| self.caught.contains(i))
                    })
                {
                    image = Some(said_caught(&shown_image));
                }
            }
            if let Some(picture_name) = self.picture_name.clone() {
                // What the game fills a picture with comes first, as it is drawn: the Bag's
                // party slot is its Pokémon's icon, then its level (movie21). Not a picture of
                // a button inside it, shown or not: Amie's strip across its top bar took the
                // Sweet Poké Puff of the bar of puffs inside it for its name, each puff a
                // button of its own, and kept it with the bar hidden (or_amie_pokepuffs).
                let inner = self.inner(root, touch, tree)?;
                let mut named: Vec<i64> = Vec::new();
                for &p in group {
                    if tree.get(&p).map(String::as_str) == Some(lyt::PICTURE_CLASS)
                        && !inner.contains(&p)
                        && picture_name(p)?.is_some_and(|word| !word.is_empty())
                    {
                        named.push(p);
                    }
                }
                let parent = u32(&layouts.pane(touch)?, lyt::PARENT)?;
                let mut beside = said.is_empty() && image.as_deref().is_none_or(str::is_empty);
                if beside {
                    for &p in &named {
                        if self.lies_in(p, touch)? {
                            beside = false;
                            break;
                        }
                    }
                }
                if beside {
                    beside = false;
                    for p in layouts.children(root, parent) {
                        if tree.get(&p).map(String::as_str) == Some(lyt::PICTURE_CLASS)
                            && picture_name(p)?.is_some_and(|word| !word.is_empty())
                        {
                            beside = true;
                            break;
                        }
                    }
                }
                if beside {
                    // Super Training's carousel moves its party's pictures between touch
                    // areas that stay put, so a tap selects what is drawn at the place, not
                    // the picture the area's own pane holds (2026-10-10, live from
                    // omega_ruby_bottom-screen5's end). Only an area whose own pane holds such
                    // a picture beside it: the Amie menu's backdrop, holding none, took the
                    // Pokémon lying within it for its name, not "close". And only where none
                    // of its own pictures lies in it: the big slot's own, the row's icon 53
                    // off its middle, still fell in its group, and the slot took its name over
                    // the Pokémon drawn big in it every second or so; the
                    // DexNav's icons each lie in their own, and naming every picture of its
                    // panel cost 180 reads a poll. The picture nearest its middle, its own or
                    // not.
                    if filled.is_none() {
                        let mut found: IndexMap<i64, String> = IndexMap::new();
                        for (&p, kind) in tree {
                            if kind == lyt::PICTURE_CLASS
                                && shown(&layouts, p)?
                                && let Some(word) = picture_name(p)?
                                && !word.is_empty()
                            {
                                found.insert(p, word);
                            }
                        }
                        filled = Some(found);
                    }
                    let mut lying: Vec<i64> = Vec::new();
                    for &p in filled.as_ref().expect("filled above").keys() {
                        if !inner.contains(&p) && self.lies_in(p, touch)? {
                            lying.push(p);
                        }
                    }
                    // `sorted(dict.fromkeys([*named, *lying]), key=...)`: each key found
                    // once, in order, before the sort.
                    let mut keyed: Vec<(i64, f64)> = Vec::new();
                    for p in named
                        .iter()
                        .chain(&lying)
                        .copied()
                        .collect::<IndexSet<i64>>()
                    {
                        keyed.push((p, self.off_middle(p, touch)?));
                    }
                    named = py::sorted_by_lt(keyed, |a, b| a.1 < b.1)
                        .into_iter()
                        .map(|(p, _key)| p)
                        .collect();
                }
                let pictured = match named.first() {
                    Some(&first) => picture_name(first)?,
                    None => None,
                };
                // Screen text names the picture too where it names pictures: the Bag's party
                // slots were "Torchic Torchic L14", the AreaNav's Pokémon "Surskit Surskit"
                // and the Battle Maison's partners "May May" (or_multi_battle_partners).
                if let Some(pictured) = pictured
                    && !pictured.is_empty()
                    && !spoken[&touch].contains(&pictured)
                {
                    said = if said.is_empty() {
                        pictured
                    } else {
                        format!("{pictured} {said}")
                    };
                }
            }
            if !said.is_empty() || image.as_deref().is_some_and(|image| !image.is_empty()) {
                names.insert(
                    touch,
                    if !said.is_empty() {
                        said
                    } else {
                        image.unwrap_or_default()
                    },
                );
            }
        }
        let mut by_picture: Option<IndexMap<String, String>> = None;
        let mut called: IndexMap<i64, Option<String>> = IndexMap::new();
        for (&touch, &(x, y)) in seen {
            let group = &groups[&touch];
            let on_screen = (0..WIDTH).contains(&x) && (0..HEIGHT).contains(&y);
            // A button with words of its own is named by them or not at all: the Bag's rows
            // all draw `bar1.bclim`, and an empty row was said by the one row's item
            // (movie21).
            let worded = group
                .iter()
                .any(|p| tree.get(p).map(String::as_str) == Some(lyt::TEXTBOX_CLASS));
            if !names.contains_key(&touch) && !worded {
                if by_picture.is_none() {
                    by_picture = Some(self.by_picture(root, tree, textures, drawn)?);
                }
                let table = by_picture.as_ref().expect("made just above");
                let mut alike: Option<String> = None;
                for i in drawn_with(
                    group
                        .iter()
                        .copied()
                        .filter(|p| tree.get(p).map(String::as_str) == Some(lyt::PICTURE_CLASS)),
                    &|p| Ok(drawn.get(&p).cloned().flatten()),
                    textures,
                    &|p| self.layouts.pane_name(p),
                ) {
                    if let Some(word) = table.get(&i?) {
                        alike = Some(word.clone());
                        break;
                    }
                }
                if let Some(alike) = alike
                    && !alike.is_empty()
                {
                    names.insert(touch, alike);
                }
            }
            let panes: HashSet<i64> = group.iter().copied().collect();
            if let Some(name) = names.get(&touch) {
                // Remembered only from the rules above: a picture's own words are a guess.
                self.named.insert(touch, (name.clone(), panes));
            } else if let Some((name, held)) = self.named.get(&touch)
                && *held == panes
            {
                // And only while its button holds what it held: Amie's Interior Items move
                // each page's pictures under touch areas that stay put, and a slot empty on
                // the next page was still said by the item it had held (spike 228,
                // or_pokemon_me2).
                names.insert(touch, name.clone());
            } else if let Some(slot) = {
                let (x, y) = place(&layouts, touch)?;
                slots::slot(
                    &layouts,
                    file,
                    root,
                    x,
                    y,
                    &(self.on_screen)(),
                    None,
                    None,
                    false,
                )?
            } && slot.0 == touch
            {
                // A slot is said as the Box's pointer says it, by its place in the grid, its
                // number in the party or the mark it shows: by its pictures, the Box's grid
                // was its wallpaper's, "wave pattern", and a mark "box mark 01". Two of the
                // grid's slots lie under a placeholder TextBox of another panel (movie22).
                if slot.1.is_empty() {
                    // Behind the Battle Box's, and in the way of its taps (or_battle_box).
                    continue;
                }
                names.insert(touch, slot.1);
            } else if !worded && on_screen && u32(&layouts.pane(touch)?, lyt::PARENT)? != root {
                // A touch area hanging straight from the root has no button of its own, and
                // the pictures over it are the screen's: the Battle Box's slots, left on
                // under BOX 1, were said by its wallpaper, "line pattern" (movie22). One off
                // the screen is not offered, so not guessed at: X's field menu keeps a
                // button at (340, 70) until it slides in, and every run logged a word made
                // for its `shadow.bclim`.
                // The picture lying in it first: the PSS's Favorites List hangs its return
                // from the bar along the bottom, and was said "touch bar" (or_pss_singleplayer).
                let mut drawn_in: Vec<i64> = Vec::new();
                for &p in &owned[&touch] {
                    if tree.get(&p).map(String::as_str) == Some(lyt::PICTURE_CLASS)
                        && shown(&layouts, p)?
                    {
                        drawn_in.push(p);
                    }
                }
                // `sorted(..., key=lambda p: not self._lies_in(p, touch))`: every key found
                // first, in order, then a stable sort, False first.
                let mut keyed: Vec<(i64, bool)> = Vec::new();
                for p in drawn_in {
                    keyed.push((p, !self.lies_in(p, touch)?));
                }
                keyed.sort_by_key(|&(_p, key)| key);
                let mut found: Vec<String> = Vec::new();
                for (p, _key) in keyed {
                    let image = drawn.get(&p).cloned().flatten();
                    let t = match image {
                        Some(image) if !image.is_empty() => Some(image),
                        _ => textures
                            .get(&self.layouts.pane_name(p)?)
                            .and_then(|textures| textures.first().cloned()),
                    };
                    if let Some(t) = t.filter(|t| !t.is_empty()) {
                        found.push(t);
                    }
                }
                let layout_name = file.map_or("", |file| file.name.as_str());
                // Of those, one the data has a word for: the Colosseum's return draws the end
                // of the bar it sits on before its arrow, and was said "topbar" (the PSS's
                // multiplayer).
                let texture = found
                    .iter()
                    .find(|t| pictures::named_in(t, layout_name).is_some_and(|w| !w.is_empty()))
                    .or(found.first())
                    .cloned();
                let stem = texture
                    .as_deref()
                    .map(|texture| texture.split('.').next().unwrap_or_default());
                let mut picture: Option<String> = None;
                if stem == Some(pictures::ARROW) || (stem == Some(pictures::BUTTON) && y < TOP_BAR)
                {
                    picture = Some(own(
                        if (x as f64) < WIDTH as f64 / 2.0 {
                            pictures::PREVIOUS
                        } else {
                            pictures::NEXT
                        },
                        &[],
                    ));
                } else if let Some(named) = texture
                    .as_deref()
                    .and_then(|texture| pictures::named_in(texture, layout_name))
                {
                    picture = Some(named);
                } else {
                    // Where the data has no word for the picture, the game's own written
                    // under it: the PSS's menu draws Battle, Trade and the rest as icons,
                    // each with its word below its touch area, said "pss taisen".
                    let label = self.label(root, touch, words, groups)?;
                    if let Some(label) = label {
                        picture = Some(label.text.clone());
                        boxes.entry(root).or_default().insert(label.r#box);
                    } else if let Some(texture) = &texture {
                        picture = Some(pictures::words_in(texture, layout_name));
                    }
                }
                if let Some(picture) = picture
                    && !picture.is_empty()
                {
                    names.insert(touch, picture);
                }
            }
            let mut name = names.get(&touch).filter(|name| !name.is_empty()).cloned();
            if on_screen && name.is_none() && !self.unnamed.contains(&touch) {
                self.unnamed.insert(touch);
                (self.log)(&format!(
                    "touch: {} of layout {root:#x} has no name",
                    self.layouts.pane_name(touch)?
                ));
            }
            if on_screen
                && greyed.contains(&touch)
                && let Some(said) = &name
                && let Some(said_unavailable) = &self.said_unavailable
            {
                name = Some(said_unavailable(said));
            }
            called.insert(touch, name);
        }
        Ok((called, spoken))
    }

    /// The TextBoxes whose words name a button on the bottom screen now, a button's own words
    /// or the label under its picture, which touch mode says as its target: offered or not, as a
    /// layout drawn over it may cover it. The field came back with the PSS's menu over its page,
    /// and its Friends, Acquaintances and Passersby were said (or_pss_singleplayer). A button
    /// hidden is one all the same, while its words stay on screen: the PSS Settings' rows, under
    /// its question, came back with it. Only one laid out, whose words are its own; looked for
    /// here alone, as no walk needs it. Not one shown and switched off, which touch mode does not
    /// offer: the PSS profile's History and Locations are said as screen text, or never would be
    /// (or_pss_singleplayer_2). And once a button's words, always,
    /// by its layout's file and its pane's name: an O-Power's panel grows from nothing as it
    /// opens, its buttons not laid out, and Y said its "Use: Give, Cancel" at every opening but
    /// the first (y_pss_singleplayer); the PSS's profile, built again as it came back from its
    /// messages, said its rows.
    ///
    /// Walked as touch mode walks them where it offers them. Where it offers none, it names none
    /// and switches none off, so a button is its touch area's words alone: the Pokédex's
    /// registration's Next (ADR-0013, amended 2026-11-24), where naming each touch area, as a walk
    /// would, logs every picture it cannot name and reads what nothing there needs.
    ///
    /// Python's sets of boxes are only tested and joined here, every name read of a box already
    /// read in the walk: `HashSet`.
    pub fn button_words(&mut self) -> Result<HashSet<i64>> {
        let offered = match &self.offered {
            None => true,
            Some(offered) => offered(),
        };
        let (named, switched_off) = if offered {
            let (_found, named, switched_off) = self.walk()?;
            (named, switched_off)
        } else {
            self.groups.clear();
            self.middles.clear();
            (IndexMap::new(), HashSet::new())
        };
        let layouts = self.layouts.clone();
        let mut boxes: HashSet<i64> = HashSet::new();
        for root in layouts.roots() {
            let Some(placement) = layouts.placement(root)? else {
                continue;
            };
            if !placement.shown || placement.top {
                continue;
            }
            let tree = layouts.tree(root);
            let mut found: HashSet<i64> = named.get(&root).cloned().unwrap_or_default();
            let mut unoffered: HashSet<i64> = HashSet::new();
            for (&pane, kind) in &tree {
                if kind != lyt::BOUNDING_CLASS
                    || self.middle_of(pane)?.is_none()
                    || !self.button(pane)?
                {
                    continue;
                }
                let mut words: HashSet<i64> = HashSet::new();
                for p in self.group(root, pane, &tree)? {
                    if tree.get(&p).map(String::as_str) == Some(lyt::TEXTBOX_CLASS) {
                        words.insert(p);
                    }
                }
                if switched_off.contains(&pane) {
                    unoffered.extend(words);
                } else {
                    found.extend(words);
                }
            }
            if let Some(file) = (self.file_of)(root)
                && !file.name.is_empty()
            {
                // Learnt from the targets offered alone, laid out: a layout just built has every
                // pane in one place, and Y's O-Power list took its panel's every word for a
                // button's, never to say them.
                if !layouts.fresh(root) {
                    let learnt: Vec<i64> = named
                        .get(&root)
                        .map(|boxes| boxes.iter().copied().collect())
                        .unwrap_or_default();
                    for r#box in learnt {
                        self.worded
                            .insert((file.name.clone(), self.layouts.pane_name(r#box)?));
                    }
                }
                for (&pane, kind) in &tree {
                    if kind == lyt::TEXTBOX_CLASS
                        && self
                            .worded
                            .contains(&(file.name.clone(), self.layouts.pane_name(pane)?))
                    {
                        found.insert(pane);
                    }
                }
            }
            boxes.extend(found.difference(&unoffered));
        }
        Ok(boxes)
    }

    /// Is the touch area a button, smaller than half the screen? One as large is a window onto
    /// the screen, its words the screen's: the AreaNav's window on the field, saying the place
    /// as the field comes back, the DexNav's panel, saying "It couldn't be found nearby", and the
    /// BuzzNav's, its news (movie24).
    fn button(&self, touch: i64) -> Result<bool> {
        let shape = self.layouts.shape;
        let head = self.layouts.pane(touch)?;
        Ok(size(&head, shape.width, shape.scale_x)?
            * size(&head, shape.height, shape.scale_y)?
            * 2.0
            < (WIDTH * HEIGHT) as f64)
    }

    /// The windows and pictures of a layout that are buttons switched on (`on`, their names)
    /// holding words and no touch area: the game takes a tap on the pane itself. The PSS
    /// Settings' Yes and No are windows, `2w` and `34`, each holding its word, and were never
    /// offered (spike 312), nor the profile's messages to choose from, pictures, `prof_btn`, each
    /// holding its message (or_pss_singleplayer_2). Holding words: Tile Puzzle's cursor, a
    /// picture its manager lists, was offered by its texture, "cursol".
    ///
    /// A layout whose manager is not known now has the windows it had: a manager freed and
    /// built again in its block reads as it did, and is found only at the next whole sweep
    /// (`core.heaps`), and the PSS Settings opened a second time offered no Enable Game Chat and
    /// Disable Game Chat for 11 s.
    fn windows(
        &mut self,
        root: i64,
        tree: &IndexMap<i64, String>,
        on: &HashSet<String>,
    ) -> Result<Vec<i64>> {
        let layouts = self.layouts.clone();
        let classes = [lyt::WINDOW_CLASS, lyt::PICTURE_CLASS];
        if !on.is_empty() {
            let windowed = self.windowed.entry(root).or_default();
            for ((&pane, kind), name) in tree.iter().zip(layouts.pane_names(root)) {
                if on.contains(&name) && classes.contains(&kind.as_str()) {
                    windowed.insert(pane);
                }
            }
        }
        let mut found: Vec<i64> = Vec::new();
        for ((&pane, kind), name) in tree.iter().zip(layouts.pane_names(root)) {
            let switched = if on.is_empty() {
                self.windowed
                    .get(&root)
                    .is_some_and(|windowed| windowed.contains(&pane))
            } else {
                on.contains(&name)
            };
            if !switched || !classes.contains(&kind.as_str()) || !shown(&layouts, pane)? {
                continue;
            }
            // No touch area under it at any depth: the profile's bar along the bottom, a picture
            // its manager lists, holds its count's words and return's picture, return's touch
            // area under that.
            let mut kinds: HashSet<&str> = HashSet::new();
            let mut todo: Vec<i64> = layouts.children(root, pane);
            while let Some(here) = todo.pop() {
                kinds.insert(tree.get(&here).map_or("", String::as_str));
                todo.extend(layouts.children(root, here));
            }
            if kinds.contains(lyt::TEXTBOX_CLASS) && !kinds.contains(lyt::BOUNDING_CLASS) {
                found.push(pane);
            }
        }
        Ok(found)
    }

    /// Does the layout draw a picture over the whole touch screen, solid or a veil? The PSS
    /// draws one behind each of its windows, its menu's 640 wide for its two pages.
    fn backdrop(&self, root: i64, tree: &IndexMap<i64, String>) -> Result<bool> {
        let (layouts, shape) = (&self.layouts, self.layouts.shape);
        let head = layouts.pane(root)?;
        let (root_x, root_y) = (f32(&head, shape.x)?, f32(&head, shape.y)?);
        for (&pane, kind) in tree {
            if kind != lyt::PICTURE_CLASS || !shown(layouts, pane)? {
                continue;
            }
            let head = layouts.pane(pane)?;
            let (x, y) = drawn(&head, &shape, root_x, root_y)?;
            let width = size(&head, shape.width, shape.scale_x)?;
            let height = size(&head, shape.height, shape.scale_y)?;
            let (x, y) = (x as f64, y as f64);
            if x - width / 2.0 <= 0.0
                && x + width / 2.0 >= WIDTH as f64
                && y - height / 2.0 <= 0.0
                && y + height / 2.0 >= HEIGHT as f64
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The bars a layout draws across the touch screen, each (pane, its place in the layout's
    /// order, top, bottom): an opaque picture as wide as the screen and under half as tall, as
    /// the O-Power list's gauge is, `assist_bg` 320 by 24. Under half: a frame as big as the
    /// screen may be clear in its middle.
    fn bars(
        &self,
        _root: i64,
        tree: &IndexMap<i64, String>,
        root_x: f64,
        root_y: f64,
    ) -> Result<Vec<(i64, i64, f64, f64)>> {
        let (layouts, shape) = (&self.layouts, self.layouts.shape);
        let mut found: Vec<(i64, i64, f64, f64)> = Vec::new();
        for (at, (&pane, kind)) in tree.iter().enumerate() {
            if kind != lyt::PICTURE_CLASS || !shown(layouts, pane)? {
                continue;
            }
            let head = layouts.pane(pane)?;
            let width = size(&head, shape.width, shape.scale_x)?;
            let height = size(&head, shape.height, shape.scale_y)?;
            if *head
                .get(shape.alpha as usize)
                .ok_or_else(|| py::Error::index("index out of range"))?
                != 0xFF
                || height * 2.0 >= HEIGHT as f64
            {
                continue;
            }
            let (x, y) = drawn(&head, &shape, root_x, root_y)?;
            let (x, y) = (x as f64, y as f64);
            if x - width / 2.0 <= 0.0 && x + width / 2.0 >= WIDTH as f64 {
                found.push((pane, at as i64, y - height / 2.0, y + height / 2.0));
            }
        }
        Ok(found)
    }

    /// The words written under a touch area, as a label is under its icon, that are no
    /// button's own: the nearest of the layout's on screen, hanging beside the icon the touch
    /// area hangs from, whose middle is within its width and whose top is no more than
    /// `LABEL_GAP` below its bottom. The PSS's menu writes "Battle" over the bottom 10 of its
    /// icon's touch area, both hanging from the menu's backdrop; another player's window hangs
    /// "Battle" beside the touch area itself, with the icon (or_pss_singleplayer_2). Words not
    /// beside it are another's: Amie's games menu writes "Head It" under its first button from a
    /// pane of its own, and the Fly map's Pokémon Distribution names icons drawn under others.
    /// With none under it, the words starting no more than `LABEL_GAP` past its right edge
    /// whose middle is within its height, as a box's words are beside it: another player's
    /// profile ticks a box beside "Add to Favorites List", said "checkmark2" (the PSS's
    /// multiplayer).
    fn label(
        &mut self,
        root: i64,
        touch: i64,
        words: &IndexMap<i64, ScreenText>,
        groups: &IndexMap<i64, Vec<i64>>,
    ) -> Result<Option<ScreenText>> {
        let layouts = self.layouts.clone();
        let shape = layouts.shape;
        let head = layouts.pane(touch)?;
        let parent = u32(&head, lyt::PARENT)?;
        let beside = [parent, u32(&layouts.pane(parent)?, lyt::PARENT)?];
        let (across, up) = middle(&head, &shape)?;
        let width = size(&head, shape.width, shape.scale_x)?;
        let height = size(&head, shape.height, shape.scale_y)?;
        let bottom = up - height / 2.0;
        let right = across + width / 2.0;
        let buttons: HashSet<i64> = groups.values().flatten().copied().collect();
        let mut best: Option<(f64, &ScreenText)> = None;
        let mut nearest: Option<(f64, &ScreenText)> = None;
        for (&r#box, piece) in words {
            if piece.root != root
                || piece.picture
                || buttons.contains(&r#box)
                || !beside.contains(&u32(&layouts.pane(r#box)?, lyt::PARENT)?)
            {
                continue;
            }
            let Some(middle) = self.middle_of(r#box)? else {
                continue;
            };
            let pane = layouts.pane(r#box)?;
            let top = middle.1 + size(&pane, shape.height, shape.scale_y)? / 2.0;
            if (middle.0 - across).abs() * 2.0 <= width
                && middle.1 < up
                && bottom - top <= LABEL_GAP
            {
                if best.is_none_or(|(y, _piece)| middle.1 > y) {
                    best = Some((middle.1, piece));
                }
                continue;
            }
            let gap = (middle.0 - size(&pane, shape.width, shape.scale_x)? / 2.0 - right).abs();
            if (middle.1 - up).abs() * 2.0 <= height
                && gap <= LABEL_GAP
                && nearest.is_none_or(|(near, _piece)| gap < near)
            {
                nearest = Some((gap, piece));
            }
        }
        let found = best.or(nearest);
        Ok(found.map(|(_y, piece)| piece.clone()))
    }

    /// The targets, less a list's buttons while the list is not up. A list's layout can stay
    /// drawn after the list has gone, every flag of its panes the same: Tile Puzzle's finished
    /// menu, View Picture, Play Again and Quit, was offered over the picture it showed
    /// (or_amie_tile_puzzle_easy) and over the next puzzle's Begin, where a tap on View Picture
    /// began the puzzle (spike 255). Its list's items say it has gone (`ListReader.lit`). So a
    /// button once a list's label, while the list was up, is left out while the list is not.
    pub(crate) fn unlisted(&mut self, found: &[Target], labels: &HashSet<String>) -> Vec<Target> {
        let roots = self.layouts.roots();
        self.listed.retain(|root, _names| roots.contains(root));
        for target in found {
            if labels.contains(&target.name) {
                self.listed
                    .entry(target.root)
                    .or_default()
                    .insert(target.name.clone());
            }
        }
        found
            .iter()
            .filter(|target| {
                labels.contains(&target.name)
                    || !self
                        .listed
                        .get(&target.root)
                        .is_some_and(|names| names.contains(&target.name))
            })
            .cloned()
            .collect()
    }

    /// The AreaNav's map's places on screen, each a layer under the layout it is in: the
    /// AreaNav draws its every layout at one layer, and its bar, its zoom and Trainer's Eye's rows
    /// lie over the map, taking a tap on a place under them (or_area_nav).
    fn map_places(
        &self,
        root: i64,
        touches: &[i64],
        layer: i64,
        root_x: f64,
        root_y: f64,
    ) -> Result<Vec<Target>> {
        let places = match &self.places {
            Some(places) => places()?,
            None => Rc::new(IndexMap::new()),
        };
        let shape = self.layouts.shape;
        let mut found: Vec<Target> = Vec::new();
        for &touch in touches {
            let Some(place) = places.get(&self.layouts.pane_name(touch)?) else {
                continue;
            };
            let pane = self.layouts.pane(touch)?;
            let (x, y) = pixel(&pane, &shape, root_x, root_y)?;
            if (0..WIDTH).contains(&x) && (0..HEIGHT).contains(&y) {
                found.push(Target {
                    pane: touch,
                    name: place.clone(),
                    x,
                    y,
                    row: f32(&pane, shape.y)? - root_y,
                    height: f32(&pane, shape.height)?,
                    layer: layer - 1,
                    root,
                    path: Vec::new(),
                    seconds: 0.0,
                    drop: None,
                    rub: None,
                    held: false,
                    after: After::default(),
                });
            }
        }
        Ok(found)
    }

    /// Rule 3's table: an image -> the words of the one button of the layout, shown or not, whose
    /// words are written beside a picture drawing it. The X menu's six buttons each draw their
    /// icon beside their word, and the shortcut bar under the AreaNav draws the same icons alone
    /// (movie18); an image several buttons draw, as the X menu's backing `menu_rgb` is, names none.
    /// The table that came before paired the bar's icons with the layout's groups of words by
    /// count, and the count changed with the X menu open: the bar was said "menu icon1" to "menu
    /// icon6" from then on. Nor does an image two buttons without words draw, which tells neither
    /// apart: another player's Battle, Trade, Trainer PR Video and O-Power in X and Y's PSS each
    /// sit on `pss_icon_base.bclim`, as does its hidden Game Chat, and all four were said "Game
    /// Chat" (the PSS's multiplayer).
    fn by_picture(
        &mut self,
        root: i64,
        tree: &IndexMap<i64, String>,
        textures: &IndexMap<String, Vec<String>>,
        drawn: &IndexMap<i64, Option<String>>,
    ) -> Result<IndexMap<String, String>> {
        // A `set[str]` in Python: only its size and its one member are read.
        let mut named: IndexMap<String, IndexSet<String>> = IndexMap::new();
        // A `Counter` in Python, only looked up.
        let mut bare: HashMap<String, i64> = HashMap::new();
        for (&touch, kind) in tree {
            // A touch area hanging straight from the root is the screen's, not a button beside
            // words: Amie's decorate screen's bar under its two tabs named its exit by both tabs'
            // words, "Interior Items Poké Puffs" (x_pokemon-me).
            if kind != lyt::BOUNDING_CLASS || u32(&self.layouts.pane(touch)?, lyt::PARENT)? == root
            {
                continue;
            }
            let group = self.group(root, touch, tree)?;
            let said = self.written(&group, tree)?;
            let drawing: Vec<i64> = group
                .iter()
                .copied()
                .filter(|p| tree.get(p).map(String::as_str) == Some(lyt::PICTURE_CLASS))
                .collect();
            if !said.is_empty() {
                for image in drawn_with(drawing, &*self.image_of, textures, &|p| {
                    self.layouts.pane_name(p)
                }) {
                    named.entry(image?).or_default().insert(said.clone());
                }
                continue;
            }
            // Only images already read, `drawn` the shown pictures': reading a hidden one's was
            // a read no poll had made, and 20 of 137 tapes no longer had their answers.
            let mut images: HashSet<String> = HashSet::new();
            for image in drawn_with(
                drawing,
                &|p| Ok(drawn.get(&p).cloned().flatten()),
                textures,
                &|p| self.layouts.pane_name(p),
            ) {
                images.insert(image?);
            }
            for image in images {
                *bare.entry(image).or_default() += 1;
            }
        }
        Ok(named
            .into_iter()
            .filter(|(image, said)| said.len() == 1 && bare.get(image).copied().unwrap_or(0) < 2)
            .map(|(image, said)| (image, said[0].clone()))
            .collect())
    }

    /// The words code wrote into a button's TextBoxes, shown or not, each once.
    fn written(&self, group: &[i64], tree: &IndexMap<i64, String>) -> Result<String> {
        let mut said: Vec<String> = Vec::new();
        for &pane in group {
            if tree.get(&pane).map(String::as_str) == Some(lyt::TEXTBOX_CLASS) {
                let text = (self.text_of)(pane)?;
                if !text.is_empty() && !said.contains(&text) {
                    said.push(text);
                }
            }
        }
        Ok(said.join(" "))
    }

    /// Does the touch area's button draw a picture faded, neither clear nor solid? The PSS
    /// draws another player's Battle, Trade and O-Power icons at alpha 100 while they are
    /// switched off, and at 255 on its menu. Faded alone is not greyed: Super Training's party
    /// under its panels draws a picture at alpha 200, with no animation to grey it, and was
    /// said "Ralts unavailable" (the round after or_pss_singleplayer_2).
    fn faded(&mut self, root: i64, touch: i64, tree: &IndexMap<i64, String>) -> Result<bool> {
        let layouts = self.layouts.clone();
        for p in self.group(root, touch, tree)? {
            if tree.get(&p).map(String::as_str) == Some(lyt::PICTURE_CLASS)
                && shown(&layouts, p)?
                && (1..0xFF).contains(
                    layouts
                        .pane(p)?
                        .get(layouts.shape.alpha as usize)
                        .ok_or_else(|| py::Error::index("index out of range"))?,
                )
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Does the touch area's button draw a picture offered though the button is off?
    fn said_though_off(
        &mut self,
        root: i64,
        touch: i64,
        tree: &IndexMap<i64, String>,
    ) -> Result<bool> {
        let Some(said_off) = self.said_off.clone() else {
            return Ok(false);
        };
        for p in self.group(root, touch, tree)? {
            if tree.get(&p).map(String::as_str) == Some(lyt::PICTURE_CLASS) && said_off(p)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The touch area's button: the pane it hangs from and everything under it that lies within
    /// it, in the layout's own order. Collected last child first, the Pokédex's button was heard
    /// "13 Number seen: 5 Number caught: HOENN POKÉDEX"; and the Bag's and the Pokédex's lists
    /// hang their scrolling strips from the layout's root, whose every word on screen they were
    /// said by (movie21), as the AreaNav's off-screen one named every picture of its layout
    /// (movie17). A pane not laid out yet, as a hidden button's may not be, is taken for inside.
    fn group(&mut self, root: i64, touch: i64, _tree: &IndexMap<i64, String>) -> Result<Vec<i64>> {
        if let Some(group) = self.groups.get(&touch) {
            return Ok(group.clone());
        }
        let layouts = self.layouts.clone();
        let shape = layouts.shape;
        let head = layouts.pane(touch)?;
        let (across, down) = middle(&head, &shape)?;
        let width = f32(&head, shape.width)? * f32(&head, shape.scale_x)?;
        let height = f32(&head, shape.height)? * f32(&head, shape.scale_y)?;
        let parent = match u32(&head, lyt::PARENT)? {
            0 => touch,
            parent => parent,
        };
        let mut out: Vec<i64> = Vec::new();
        let mut todo: Vec<i64> = vec![parent];
        while let Some(here) = todo.pop() {
            todo.extend(layouts.children(root, here).into_iter().rev());
            let middle = self.middle_of(here)?;
            if width > 0.0
                && let Some(middle) = middle
                && here != parent
                && ((middle.0 - across).abs() * 2.0 > width
                    || (middle.1 - down).abs() * 2.0 > height)
            {
                continue;
            }
            out.push(here);
        }
        self.groups.insert(touch, out.clone());
        Ok(out)
    }

    /// The panes of the buttons of the touch areas inside the touch area's own, shown or
    /// not.
    fn inner(
        &mut self,
        root: i64,
        touch: i64,
        tree: &IndexMap<i64, String>,
    ) -> Result<HashSet<i64>> {
        let group: HashSet<i64> = self.group(root, touch, tree)?.into_iter().collect();
        let mut inner: HashSet<i64> = HashSet::new();
        for (&other, kind) in tree {
            if kind == lyt::BOUNDING_CLASS && other != touch && group.contains(&other) {
                let button: HashSet<i64> = self.group(root, other, tree)?.into_iter().collect();
                // `button < group`: a proper subset.
                if button.len() < group.len() && button.is_subset(&group) {
                    inner.extend(button);
                }
            }
        }
        Ok(inner)
    }

    /// A pane's middle, or None where it is not laid out, found once per walk.
    fn middle_of(&mut self, pane: i64) -> Result<Option<(f64, f64)>> {
        if !self.middles.contains_key(&pane) {
            let head = self.layouts.pane(pane)?;
            let shape = self.layouts.shape;
            let found = if f32(&head, shape.scale_x)? > 0.0 {
                Some(middle(&head, &shape)?)
            } else {
                None
            };
            self.middles.insert(pane, found);
        }
        Ok(self.middles[&pane])
    }

    /// Does the pane's middle lie within the touch area, whatever either's scale? Super
    /// Training's resting Pokémon is drawn swinging its scale through 0 to mirrored and back,
    /// its touch area with it, and in that half it lay in no slot.
    fn lies_in(&self, pane: i64, touch: i64) -> Result<bool> {
        let shape = self.layouts.shape;
        let head = self.layouts.pane(touch)?;
        let (across, down) = middle(&head, &shape)?;
        let middle = middle(&self.layouts.pane(pane)?, &shape)?;
        Ok(
            (middle.0 - across).abs() * 2.0 <= size(&head, shape.width, shape.scale_x)?
                && (middle.1 - down).abs() * 2.0 <= size(&head, shape.height, shape.scale_y)?,
        )
    }

    /// How far a pane's middle is from the touch area's, whatever either's scale.
    fn off_middle(&self, pane: i64, touch: i64) -> Result<f64> {
        let shape = self.layouts.shape;
        let here = middle(&self.layouts.pane(pane)?, &shape)?;
        let (across, up) = middle(&self.layouts.pane(touch)?, &shape)?;
        Ok(py::hypot(here.0 - across, here.1 - up))
    }

    /// The pictures a screen offers (`pictured`), each where it is drawn, as a touch area is
    /// placed, and left out as one is: where its layout is not drawn on the bottom screen or its
    /// buttons are locked, as under a menu (spike 189); where its button is switched off, as a
    /// training bag's is under Super Training's Yes and No (spike 228); and off the screen, as
    /// the bags are while their list is closed.
    fn pictures(
        &self,
        offered: &IndexMap<i64, String>,
        works: &IndexMap<i64, i64>,
        idle: &HashSet<i64>,
        off: &HashSet<(i64, String)>,
    ) -> Result<Vec<Target>> {
        let (layouts, shape) = (&self.layouts, self.layouts.shape);
        let mut found: Vec<Target> = Vec::new();
        for (&pane, name) in offered {
            let Some(placement) = layouts.placement(pane)? else {
                continue;
            };
            if !placement.shown || placement.top {
                continue;
            }
            let root = placement.root;
            let work = works.get(&root).copied();
            if layouts.taken_off(root)? || work.is_some_and(|work| idle.contains(&work)) {
                continue;
            }
            // Python builds `(work, self._name(pane))` before asking `off`, a None work too.
            let pane_name = self.layouts.pane_name(pane)?;
            if work.is_some_and(|work| off.contains(&(work, pane_name))) {
                continue;
            }
            let (head, top) = (layouts.pane(pane)?, layouts.pane(root)?);
            let root_y = f32(&top, shape.y)?;
            let (x, y) = drawn(&head, &shape, f32(&top, shape.x)?, root_y)?;
            if (0..WIDTH).contains(&x) && (0..HEIGHT).contains(&y) {
                found.push(Target {
                    layer: layouts.layer(root)?,
                    root,
                    ..Target::new(
                        pane,
                        name,
                        x,
                        y,
                        f32(&head, shape.y)? - root_y,
                        f32(&head, shape.height)?,
                    )
                });
            }
        }
        Ok(found)
    }
}

/// The images pictures are drawn with, a picture at a time: the one it draws now, then those
/// its layout file's `textures` give its pane by name. A table of the game's words by image
/// names a picture by the first of them it has: touch mode's buttons, and the profile's
/// pictures under a cursor.
///
/// Python's generator asks `image_of` and `name_of` of a picture as it comes to it; so does this
/// iterator, each picture's images handed out before the next is asked of.
pub fn drawn_with<'a>(
    panes: impl IntoIterator<Item = i64> + 'a,
    image_of: &'a dyn Fn(i64) -> Result<Option<String>>,
    textures: &'a IndexMap<String, Vec<String>>,
    name_of: &'a dyn Fn(i64) -> Result<String>,
) -> impl Iterator<Item = Result<String>> + 'a {
    panes.into_iter().flat_map(move |pane| {
        let images = image_of(pane).and_then(|drawn| {
            let mut images: Vec<String> = drawn.into_iter().collect();
            images.extend(textures.get(&name_of(pane)?).cloned().unwrap_or_default());
            Ok(images)
        });
        let out: Vec<Result<String>> = match images {
            Err(error) => vec![Err(error)],
            Ok(images) => images
                .into_iter()
                .filter(|image| !image.is_empty())
                .map(Ok)
                .collect(),
        };
        out
    })
}

/// The touch screen's pixel a pane's place is, from its root's middle, y down.
fn pixel(head: &[u8], shape: &lyt::PaneShape, root_x: f64, root_y: f64) -> Result<(i64, i64)> {
    Ok((
        py::round(WIDTH as f64 / 2.0 + f32(head, shape.x)? - root_x)?,
        py::round(HEIGHT as f64 / 2.0 - (f32(head, shape.y)? - root_y))?,
    ))
}

/// The touch screen's pixel in the middle of a picture as it is drawn. A pane's place is the
/// point it hangs from: Super Training's bag hangs from its top, at (50, 56), and over 300 taps
/// there landed above it and hit nothing, where one at (50, 110) hit it every time (spike 290).
fn drawn(head: &[u8], shape: &lyt::PaneShape, root_x: f64, root_y: f64) -> Result<(i64, i64)> {
    let (across, down) = shape.origin(head)?;
    let (x, y) = pixel(head, shape, root_x, root_y)?;
    Ok((
        py::round(x as f64 + (0.5 - across) * f32(head, shape.width)? * f32(head, shape.scale_x)?)?,
        py::round(y as f64 + (0.5 - down) * f32(head, shape.height)? * f32(head, shape.scale_y)?)?,
    ))
}

/// A pane's width or height as drawn, mirrored or not, and as laid out where its scale is 0.
fn size(head: &[u8], size: i64, scale: i64) -> Result<f64> {
    let scaled = f32(head, scale)?.abs();
    Ok(f32(head, size)? * if scaled == 0.0 { 1.0 } else { scaled })
}

/// Each touch area's button less the buttons of the touch areas inside it, whose pictures are
/// their own: the Pokémon-Amie page's menu lies within the touch area behind it, which took
/// PLAY's word and tapped the menu shut, and PLAY, adding no word, was left out as part of it
/// (spike 198, omega_ruby_bottom-screen2). Words on screen stay the outer button's too: taken from
/// it, the Bag's strips and the Secret Base's panels were said by what was left, "/".
fn own_parts(groups: &IndexMap<i64, Vec<i64>>) -> IndexMap<i64, Vec<i64>> {
    let sets: IndexMap<i64, HashSet<i64>> = groups
        .iter()
        .map(|(&touch, group)| (touch, group.iter().copied().collect()))
        .collect();
    // Python's `<` on sets: a proper subset.
    let within = |inner: &HashSet<i64>, outer: &HashSet<i64>| {
        inner.len() < outer.len() && inner.is_subset(outer)
    };
    groups
        .iter()
        .map(|(&touch, group)| {
            let kept = group
                .iter()
                .copied()
                .filter(|pane| {
                    !groups.keys().any(|&other| {
                        other != touch
                            && sets[&other].contains(pane)
                            && within(&sets[&other], &sets[&touch])
                    })
                })
                .collect();
            (touch, kept)
        })
        .collect()
}

/// Where a pane's middle is across and up its screen, from its place and its anchor.
fn middle(head: &[u8], shape: &lyt::PaneShape) -> Result<(f64, f64)> {
    let width = f32(head, shape.width)? * f32(head, shape.scale_x)?;
    Ok((
        f32(head, shape.x)? + (0.5 - shape.origin(head)?.0) * width,
        f32(head, shape.y)?,
    ))
}

/// Does this touch area lie wholly inside a larger one of its layout, not one of `screens`, with
/// no words on screen the larger's button lacks? It is part of that button: the party list's
/// plates each hold a corner where a held item is drawn, said "pokelist 0" or by part of the
/// plate's words, and a tap there did nothing (movie22).
pub(crate) fn within_another(
    touch: i64,
    placed: &IndexMap<i64, (i64, i64, f64, f64)>,
    own: &IndexMap<i64, HashSet<String>>,
    screens: &HashSet<i64>,
) -> bool {
    let none = HashSet::new();
    let (x, y, width, height) = placed[&touch];
    placed.iter().any(|(&other, &(ox, oy, w, h))| {
        other != touch
            && !screens.contains(&other)
            && w * h > width * height
            && ((x - ox).abs() * 2) as f64 <= w - width + 1.0
            && ((y - oy).abs() * 2) as f64 <= h - height + 1.0
            && own
                .get(&touch)
                .unwrap_or(&none)
                .is_subset(own.get(&other).unwrap_or(&none))
    })
}

/// Does this touch area's middle, where a tap on it presses, lie within another of its
/// layout's?
pub(crate) fn on_another(touch: i64, placed: &IndexMap<i64, (i64, i64, f64, f64)>) -> bool {
    let (x, y, _, _) = placed[&touch];
    placed.iter().any(|(&other, &(ox, oy, w, h))| {
        other != touch && (((x - ox).abs() * 2) as f64) < w && (((y - oy).abs() * 2) as f64) < h
    })
}

/// The middle of the part of a touch area's span, `middle` and `size` along one side of the
/// screen, that lies between 0 and `end`.
fn on_screen(middle: i64, size: f64, end: i64) -> Result<i64> {
    // Python's max and min: the first unless the second is strictly greater (or less).
    let low = middle as f64 - size / 2.0;
    let low = if 0.0 > low { 0.0 } else { low };
    let high = middle as f64 + size / 2.0;
    let high = if (end as f64) < high {
        end as f64
    } else {
        high
    };
    py::round((low + high) / 2.0)
}

fn place(layouts: &Layouts, pane: i64) -> Result<(f64, f64)> {
    let head = layouts.pane(pane)?;
    Ok((f32(&head, layouts.shape.x)?, f32(&head, layouts.shape.y)?))
}

fn shown(layouts: &Layouts, pane: i64) -> Result<bool> {
    Ok(layouts
        .placement(pane)?
        .is_some_and(|placement| placement.shown))
}

/// The targets a tap reaches: a tap lands on the layout drawn on top, so a target under a touch
/// area (layer, x, y, width, height) of a layout drawn over its own taps that instead. The X menu
/// is drawn over the PokéNav, which the field goes on drawing, and a tap on the PokéNav's DexNav
/// opened the Pokédex (movie21).
pub fn uncovered(found: &[Target], areas: &[(i64, i64, i64, f64, f64)]) -> Vec<Target> {
    found
        .iter()
        .filter(|target| {
            !areas.iter().any(|&(layer, x, y, width, height)| {
                layer > target.layer
                    && (((target.x - x).abs() * 2) as f64) < width
                    && (((target.y - y).abs() * 2) as f64) < height
            })
        })
        .cloned()
        .collect()
}

/// The targets a tap reaches while a list window is up: its own, and any of a layout drawn at a
/// higher layer. A list window takes every key until it is answered, and the screen under it is
/// left as it was: the PC Box under "Continue operations?" drew its grid as bright as ever, and
/// touch mode offered it with YES and NO (movie22). A list's layout is the one holding its labels
/// as targets; a list that holds none of them on the bottom screen changes nothing.
pub fn answered_first(found: &[Target], labels: &HashSet<String>) -> Vec<Target> {
    let lists: HashSet<i64> = found
        .iter()
        .filter(|target| labels.contains(&target.name))
        .map(|target| target.root)
        .collect();
    if lists.is_empty() {
        return found.to_vec();
    }
    let top = found
        .iter()
        .filter(|target| lists.contains(&target.root))
        .map(|target| target.layer)
        .max()
        .expect("a list holds a target");
    found
        .iter()
        .filter(|target| lists.contains(&target.root) || target.layer > top)
        .cloned()
        .collect()
}
