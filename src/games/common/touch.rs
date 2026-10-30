//! Touch mode: the bottom screen's touch targets, walked by name, and a tap on the one chosen.
//!
//! The PokéNav Plus is touched, never walked with a cursor (docs/plans/pokenav-and-touch.md). Each
//! of its buttons is a touch area, an `nw::lyt::Bounding`, which pa3ds finds and places as it does
//! the PC Box's slots (spike 129); so is every other screen's. In touch mode the navigation keys
//! walk the touch areas drawn on the bottom screen, in reading order, row by row, saying each by
//! name; and a key taps the one chosen, through `pa3ds.core.touch` (ADR-0009, amended 2026-10-01:
//! the one thing pa3ds does to the game, on the player's key, on the target it has just named).
//!
//! A touch area is named by, in this order (spike 171):
//!
//! 1. **the words in its button**: the screen text inside the pane it hangs from, as the AreaNav's
//!    window onto the map is said by the place it shows;
//! 2. **the game's word for the picture drawn over it** (`images`), by the image it draws or the
//!    one its layout file gives it: the PokéNav's app buttons are drawn with
//!    `pmn_screen_zukan.bclim` and the like, and the ROM's text has "DexNav" for them; the
//!    AreaNav's tabs, Places to Berries, draw images of their words that come from no archive
//!    pa3ds reads, and their file names them `map_navi01` to `05` (movie17); or what the game
//!    fills a picture with as it runs: the AreaNav's Pokémon Distribution draws each Pokémon's
//!    icon, named by its species (`gen6.screens.distribution`, spike 178);
//! 3. **the words of the button drawing the same picture beside words**: the shortcut bar under
//!    the AreaNav draws the X menu's six icons alone, and the X menu draws each beside its word,
//!    POKÉMON, POKÉDEX, BAG, the player's name, SAVE and OPTIONS, hidden until it opens;
//! 4. **what it was called the last time it could be named**, while its button holds the same
//!    panes: the DexNav's button draws a magnifier from another archive while a hidden Pokémon is
//!    near (spike 171);
//! 5. **where it is, as a slot** (`slots.slot`), as the Box's pointer says it: "BOX 1, row 1,
//!    column 1, empty", "Party 2", a mark's "●, on" (movie22);
//! 6. **the words of the picture's own name** (`pictures.words`), a bare "button" in the top bar's
//!    corners being the page arrows, previous and next.
//!
//! A touch area none of these names is left out, and the log says so once. So is the AreaNav's
//! map, whose every place is a touch area (spike 152), on the field: it is drawn zoomed in and cut
//! to a window, and its places beyond the window are touch areas on screen all the same, 65 of
//! them over the DexNav's button and the shortcut bar. In the AreaNav itself a tap on a place
//! chooses it, the top screen saying its name and what it is like, and each is offered by the
//! place the ROM names it (`places`), under every other button the AreaNav draws over the map
//! (or_area_nav).
//!
//! Bound by:
//! - ADR-0009: a tap only on the player's key, and only on the target just named.
//! - ADR-0037: the DexNav's icons are offered and said caught or not, though the game switches off
//!   those not caught.

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::{f32, u32};
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::screen::{ScreenText, distinct, in_rows, join};
use crate::games::common::{lyt, pictures, slots};
use crate::py::{self, Error, Log, Result};

pub const WIDTH: i64 = 320;
/// The touch screen in pixels; a layout places its panes from its middle, y up.
pub const HEIGHT: i64 = 240;
/// How far down the screen the top bar reaches, whose corners hold the page arrows (spike 198).
pub const TOP_BAR: i64 = 30;
/// The AreaNav's map, whose touch areas are places (spike 152).
pub const MAP: &str = "poke_navi_map.bclim";

pub const ITEM_PREVIOUS: &str = "item previous";
pub const ITEM_NEXT: &str = "item next";
pub const ITEM_READ: &str = "item read";
pub const ROW_PREVIOUS: &str = "row previous";
pub const ROW_NEXT: &str = "row next";
pub const TAP: &str = "tap";
/// What the keys do in touch mode; the names are the navigation keys' own where they match.
///
/// A `frozenset` in Python, only ever tested for membership.
pub const ACTIONS: [&str; 6] = [
    ITEM_PREVIOUS,
    ITEM_NEXT,
    ITEM_READ,
    ROW_PREVIOUS,
    ROW_NEXT,
    TAP,
];
/// Said where the bottom screen has nothing to touch.
pub const NOTHING: &str = "touch-nothing";
/// How far across its width a pane's place is, by its anchor: left, centre, right.
pub const ANCHOR: [f64; 3] = [0.0, 0.5, 1.0];

/// A touch area on the bottom screen, by name, and the pixel a tap on it presses.
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub pane: i64,
    pub name: String,
    pub x: i64,
    pub y: i64,
    /// Where its middle is, up the screen, for telling rows apart.
    pub row: f64,
    pub height: f64,
    /// Where its layout is drawn among the screen's others, higher over lower (`Layouts.layer`).
    pub layer: i64,
    /// The root of its layout.
    pub root: i64,
    /// Where the key that taps slides instead, for a target that is a stroke (`TouchTargets`'s
    /// `gestures`); none for a tap.
    pub path: Vec<(i64, i64)>,
    /// How long the slide along `path` takes.
    pub seconds: f64,
}

impl Target {
    /// `Target(pane, name, x, y, row, height)`: `layer` and `root` 0, as Python defaults them.
    pub fn new(pane: i64, name: &str, x: i64, y: i64, row: f64, height: f64) -> Self {
        Target {
            pane,
            name: name.to_string(),
            x,
            y,
            row,
            height,
            layer: 0,
            root: 0,
            path: Vec::new(),
            seconds: 0.0,
        }
    }
}

/// `Callable[[], Sequence[ScreenText]]`: what is on screen, as the screen reader last read it.
pub type OnScreen = Rc<dyn Fn() -> Vec<ScreenText>>;
/// `Callable[[int], str]`: a TextBox's words, shown or not. It reads memory, so it can raise.
pub type TextOf = Rc<dyn Fn(i64) -> Result<String>>;
/// `Callable[[int], str | None]`: the file a picture draws now. It reads memory.
pub type ImageOf = Rc<dyn Fn(i64) -> Result<Option<String>>>;
/// `Callable[[int], LayoutFile | None]`: a layout's root -> the file it was built from.
pub type FileOf = Rc<dyn Fn(i64) -> Option<Rc<LayoutFile>>>;
/// `Callable[[int, int], str | None]`: tap a pixel of the touch screen.
pub type Tap = Rc<dyn Fn(i64, i64) -> Option<String>>;
/// `Callable[[float, float], None]`: sound where a target is.
pub type Tone = Rc<dyn Fn(f64, f64)>;
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
/// `Callable[[Sequence[tuple[int, int]], float], str | None]`: drag a press along a path.
pub type Slide = Rc<dyn Fn(&[(i64, i64)], f64) -> Option<String>>;
/// `Callable[[], list[Target]]`: targets no touch area draws. It reads memory.
pub type Gestures = Rc<dyn Fn() -> Result<Vec<Target>>>;
/// `Callable[[Target, Target], str | None]`: what a key moving from one target to another says.
/// It reads memory.
pub type Moved = Rc<dyn Fn(&Target, &Target) -> Result<Option<String>>>;

/// `TouchTargets.__init__`'s keyword arguments, each as the Python defaults it: set what differs
/// with `..Options::default()`.
#[derive(Default)]
pub struct Options {
    /// None is `{}`.
    pub images: Option<IndexMap<String, String>>,
    pub tap: Option<Tap>,
    pub tone: Option<Tone>,
    pub picture_name: Option<PictureName>,
    pub lists: Option<Lists>,
    pub idle: Option<Idle>,
    pub slide: Option<Slide>,
    pub gestures: Option<Gestures>,
    pub off: Option<Off>,
    pub said_off: Option<SaidOff>,
    pub places: Option<Places>,
    pub moved: Option<Moved>,
}

/// The bottom screen's touch targets, and the one touch mode is on.
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
    /// Tap a pixel of the touch screen; a line saying why not, or None once sent.
    tap: Option<Tap>,
    /// Sound where a target is, across and down the screen as shares of it (`core.tones`).
    tone: Option<Tone>,
    /// The game's word for what a picture the game fills in as it runs draws, or None.
    picture_name: Option<PictureName>,
    /// The labels of the list windows on screen: a Yes and No the player must answer first.
    lists: Option<Lists>,
    /// The works of the layouts whose buttons take no tap now (`Buttons.idle`).
    idle: Option<Idle>,
    /// Drag a press along a path of the touch screen over so many seconds.
    slide: Option<Slide>,
    /// Targets a screen offers that no touch area draws, each a stroke: Pokémon-Amie's
    /// Pokémon, stroked by sliding over it (spike 200).
    gestures: Option<Gestures>,
    /// (work, pane name) of the buttons switched off now, which take no tap (`Buttons.off`).
    off: Option<Off>,
    /// Whether a picture's button is offered though switched off: the DexNav switches off the
    /// icon of a species not caught, which it cannot search for, and the species is said all the
    /// same (ADR-0037).
    said_off: Option<SaidOff>,
    /// The AreaNav's map's touch areas by pane name -> the place each is, while the map is the
    /// screen's and not the field's window onto it; none otherwise.
    places: Option<Places>,
    /// What a key moving from one target to another says in place of the second's name, or
    /// None for its name: Tile Puzzle's tiles say only the row or column that changed.
    moved: Option<Moved>,
    chosen: Option<i64>,
    /// Touch area -> what it was last called (rule 4).
    named: IndexMap<i64, (String, HashSet<i64>)>,
    unnamed: HashSet<i64>,
    groups: IndexMap<i64, Vec<i64>>,
    /// A walk's buttons and panes' middles (None where not laid out), each found once: the Box's
    /// touch areas hanging from its root measured its every pane each, 34,000 times a walk
    /// (movie22).
    middles: IndexMap<i64, Option<(f64, f64)>>,
    /// The targets a test gives in place of `targets`' walk, as Python's test overrides the
    /// method in a subclass.
    #[cfg(test)]
    pub(crate) given: Option<Rc<dyn Fn() -> Vec<Target>>>,
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
            tap,
            tone,
            picture_name,
            lists,
            idle,
            slide,
            gestures,
            off,
            said_off,
            places,
            moved,
        } = options;
        TouchTargets {
            layouts,
            on_screen,
            text_of,
            image_of,
            file_of,
            log,
            images: images.unwrap_or_default(),
            tap,
            tone,
            picture_name,
            lists,
            idle,
            slide,
            gestures,
            off,
            said_off,
            places,
            moved,
            chosen: None,
            named: IndexMap::new(),
            unnamed: HashSet::new(),
            groups: IndexMap::new(),
            middles: IndexMap::new(),
            #[cfg(test)]
            given: None,
        }
    }

    /// Touch mode coming on: the first target, as a menu opening says the item it is on.
    pub fn start(&mut self) -> Result<String> {
        let found = self.targets()?;
        self.chosen = found.first().map(|target| target.pane);
        Ok(match found.first() {
            Some(first) => self.said(first, None),
            None => own(NOTHING, &[]),
        })
    }

    /// Answer a key of touch mode, with the line to say.
    ///
    /// Nothing is tapped that has not just been named: with nothing chosen, or the one chosen
    /// gone from the screen, any key chooses the first target and says it.
    pub fn press(&mut self, action: &str) -> Result<String> {
        let found = self.targets()?;
        if found.is_empty() {
            self.chosen = None;
            return Ok(own(NOTHING, &[]));
        }
        let Some(mut at) = found
            .iter()
            .position(|target| Some(target.pane) == self.chosen)
        else {
            self.chosen = Some(found[0].pane);
            return Ok(self.said(&found[0], None));
        };
        if action == TAP {
            let target = &found[at];
            if !target.path.is_empty() {
                let Some(slide) = &self.slide else {
                    return Ok(own("touch-off", &[]));
                };
                let path: Vec<String> = target
                    .path
                    .iter()
                    .map(|(x, y)| format!("({x}, {y})"))
                    .collect();
                (self.log)(&format!(
                    "touch: slide {} along [{}]",
                    target.name,
                    path.join(", ")
                ));
                return Ok(slide(&target.path, target.seconds).unwrap_or_default());
            }
            let Some(tap) = &self.tap else {
                return Ok(own("touch-off", &[]));
            };
            (self.log)(&format!(
                "touch: tap {} at ({}, {})",
                target.name, target.x, target.y
            ));
            return Ok(tap(target.x, target.y).unwrap_or_default());
        }
        let before = found[at].clone();
        if action == ITEM_NEXT {
            at = py::modulo(at as i64 + 1, found.len() as i64) as usize;
        } else if action == ITEM_PREVIOUS {
            at = py::modulo(at as i64 - 1, found.len() as i64) as usize;
        } else if action == ROW_NEXT || action == ROW_PREVIOUS {
            let lines = rows(&found);
            let row = lines
                .iter()
                .position(|members| members.contains(&at))
                .expect("every target is in a row");
            // Straight up or down, to the nearest across: a grid's column kept (Orion,
            // 2026-10-26, Tile Puzzle).
            let step = if action == ROW_NEXT { 1 } else { -1 };
            let x = found[at].x;
            at = *lines[py::modulo(row as i64 + step, lines.len() as i64) as usize]
                .iter()
                .min_by_key(|&&i| (found[i].x - x).abs())
                .expect("a row has a target");
        }
        self.chosen = Some(found[at].pane);
        let moved = match &self.moved {
            Some(moved) => moved(&before, &found[at])?,
            None => None,
        };
        Ok(self.said(&found[at], moved))
    }

    /// A target's name, or `line` in its place, with its tone, as the key chooses it.
    fn said(&self, target: &Target, line: Option<String>) -> String {
        if let Some(tone) = &self.tone {
            tone(
                target.x as f64 / WIDTH as f64,
                target.y as f64 / HEIGHT as f64,
            );
        }
        match line {
            Some(line) if !line.is_empty() => line,
            _ => target.name.clone(),
        }
    }

    /// Every touch area drawn on the bottom screen that can be named, in reading order.
    pub fn targets(&mut self) -> Result<Vec<Target>> {
        #[cfg(test)]
        if let Some(given) = &self.given {
            return Ok(given());
        }
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
        let works: IndexMap<i64, i64> = layouts
            .works()
            .into_iter()
            .map(|(work, root)| (root, work))
            .collect();
        let mut roots: Vec<i64> = layouts.roots().into_iter().collect();
        roots.sort();
        for root in roots {
            let placement = layouts.placement(root)?;
            // A layout whose buttons the game has locked takes no tap however it is drawn: the
            // party's list under "Do what with Blaze?", washed out by a veil (spike 189).
            let Some(placement) = placement else {
                continue;
            };
            if !placement.shown
                || placement.top
                || layouts.taken_off(root)?
                || works.get(&root).is_some_and(|work| idle.contains(work))
            {
                continue;
            }
            let tree = layouts.tree(root);
            let layer = layouts.layer(root)?;
            // A button switched off takes no tap however it is drawn: Super Training's party
            // under its Use panel (spike 228). But for a picture said all the same: the DexNav's
            // Pokémon not caught were left out with it.
            let mut touches: Vec<i64> = Vec::new();
            for (&pane, kind) in &tree {
                if kind == lyt::BOUNDING_CLASS && shown(&layouts, pane)? {
                    let name = self.name(pane)?;
                    if !works
                        .get(&root)
                        .is_some_and(|&work| off.contains(&(work, name)))
                        || self.said_though_off(root, pane, &tree)?
                    {
                        touches.push(pane);
                    }
                }
            }
            if touches.is_empty() {
                continue;
            }
            let head = layouts.pane(root)?;
            let (root_x, root_y) = (f32(&head, shape.x)?, f32(&head, shape.y)?);
            let file = (self.file_of)(root);
            let empty = IndexMap::new();
            let textures = file.as_ref().map_or(&empty, |file| &file.textures);
            let mut drawn: IndexMap<i64, Option<String>> = IndexMap::new();
            for (&pane, kind) in &tree {
                if kind == lyt::PICTURE_CLASS && shown(&layouts, pane)? {
                    drawn.insert(pane, (self.image_of)(pane)?);
                }
            }
            if drawn.values().any(|image| image.as_deref() == Some(MAP)) {
                found.extend(self.map_places(root, &touches, layer, root_x, root_y)?);
                continue;
            }
            let mut names: IndexMap<i64, String> = IndexMap::new();
            let mut groups: IndexMap<i64, Vec<i64>> = IndexMap::new();
            for &touch in &touches {
                let group = self.group(root, touch, &tree)?;
                groups.insert(touch, group);
            }
            let owned = own_parts(&groups);
            let mut filled: Option<IndexMap<i64, String>> = None;
            // Touch area -> the words on screen in its button, or with none the game's word for
            // its picture: the Pokémon-Amie page's PLAY, DECORATE and SWITCH lie within the touch
            // area behind its menu, and are buttons of their own (spike 198). A button with words
            // keeps to them: the Secret Base's DexNav panel, words and an app's icon, was offered
            // as well as the window it lies in.
            let mut spoken: IndexMap<i64, HashSet<String>> = IndexMap::new();
            for &touch in &touches {
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
                let mut image: Option<String> = None;
                'image: for &p in &owned[&touch] {
                    for i in self.drawn_or_textured(p, &drawn, textures)? {
                        if let Some(word) = self.images.get(&i) {
                            image = Some(word.clone());
                            break 'image;
                        }
                    }
                }
                if let Some(image) = &image
                    && !image.is_empty()
                    && said.is_empty()
                {
                    spoken
                        .get_mut(&touch)
                        .expect("inserted above")
                        .insert(image.clone());
                }
                if let Some(picture_name) = self.picture_name.clone() {
                    // What the game fills a picture with comes first, as it is drawn: the Bag's
                    // party slot is its Pokémon's icon, then its level (movie21).
                    let mut pictured: Option<String> = None;
                    for &p in group {
                        if tree.get(&p).map(String::as_str) == Some(lyt::PICTURE_CLASS)
                            && let Some(word) = picture_name(p)?
                            && !word.is_empty()
                        {
                            pictured = Some(word);
                            break;
                        }
                    }
                    let parent = u32(&layouts.pane(touch)?, lyt::PARENT)?;
                    let nothing = pictured.is_none()
                        && said.is_empty()
                        && image.as_deref().is_none_or(str::is_empty);
                    let mut beside = false;
                    if nothing {
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
                        // a picture beside it, moved away: the Amie menu's backdrop, holding
                        // none, took the Pokémon lying within it for its name, not "close".
                        if filled.is_none() {
                            let mut found: IndexMap<i64, String> = IndexMap::new();
                            for (&p, kind) in &tree {
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
                        pictured = None;
                        for (&p, word) in filled.as_ref().expect("filled above") {
                            if self.lies_in(p, touch)? {
                                pictured = Some(word.clone());
                                break;
                            }
                        }
                    }
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
            // Touch area -> its middle in pixels and its size.
            let mut placed: IndexMap<i64, (i64, i64, f64, f64)> = IndexMap::new();
            let first = found.len();
            for &touch in &touches {
                let group = &groups[&touch];
                let pane = layouts.pane(touch)?;
                let (x, y) = pixel(&pane, &shape, root_x, root_y)?;
                // A button with words of its own is named by them or not at all: the Bag's rows
                // all draw `bar1.bclim`, and an empty row was said by the one row's item
                // (movie21).
                let worded = group
                    .iter()
                    .any(|p| tree.get(p).map(String::as_str) == Some(lyt::TEXTBOX_CLASS));
                if !names.contains_key(&touch) && !worded {
                    if by_picture.is_none() {
                        by_picture = Some(self.by_picture(root, &tree, textures)?);
                    }
                    let table = by_picture.as_ref().expect("made just above");
                    let mut alike: Option<String> = None;
                    'alike: for &p in group {
                        if tree.get(&p).map(String::as_str) != Some(lyt::PICTURE_CLASS) {
                            continue;
                        }
                        for i in self.drawn_or_textured(p, &drawn, textures)? {
                            if let Some(word) = table.get(&i) {
                                alike = Some(word.clone());
                                break 'alike;
                            }
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
                        file.as_deref(),
                        root,
                        x,
                        y,
                        &(self.on_screen)(),
                        None,
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
                } else if !worded && u32(&layouts.pane(touch)?, lyt::PARENT)? != root {
                    // A touch area hanging straight from the root has no button of its own, and
                    // the pictures over it are the screen's: the Battle Box's slots, left on
                    // under BOX 1, were said by its wallpaper, "line pattern" (movie22).
                    // Python's `next` over a generator: the first picture's texture ends the look,
                    // and no picture after it has its name read.
                    let mut texture: Option<String> = None;
                    for &p in &owned[&touch] {
                        if tree.get(&p).map(String::as_str) == Some(lyt::PICTURE_CLASS)
                            && shown(&layouts, p)?
                        {
                            let image = drawn.get(&p).cloned().flatten();
                            let t = match image {
                                Some(image) if !image.is_empty() => Some(image),
                                _ => textures
                                    .get(&self.name(p)?)
                                    .and_then(|textures| textures.first().cloned()),
                            };
                            if let Some(t) = t.filter(|t| !t.is_empty()) {
                                texture = Some(t);
                                break;
                            }
                        }
                    }
                    let mut picture = texture.as_deref().map(pictures::words);
                    if let Some(texture) = &texture
                        && texture.split('.').next() == Some(pictures::BUTTON)
                        && y < TOP_BAR
                    {
                        picture = Some(own(
                            if (x as f64) < WIDTH as f64 / 2.0 {
                                pictures::PREVIOUS
                            } else {
                                pictures::NEXT
                            },
                            &[],
                        ));
                    }
                    if let Some(picture) = picture
                        && !picture.is_empty()
                    {
                        names.insert(touch, picture);
                    }
                }
                if !((0..WIDTH).contains(&x) && (0..HEIGHT).contains(&y)) {
                    continue;
                }
                let (width, height) = (f32(&pane, shape.width)?, f32(&pane, shape.height)?);
                areas.push((layer, x, y, width, height));
                placed.insert(touch, (x, y, width, height));
                let name = names.get(&touch).filter(|name| !name.is_empty()).cloned();
                let Some(name) = name else {
                    if !self.unnamed.contains(&touch) {
                        self.unnamed.insert(touch);
                        (self.log)(&format!(
                            "touch: {} of layout {root:#x} has no name",
                            self.name(touch)?
                        ));
                    }
                    continue;
                };
                found.push(Target {
                    pane: touch,
                    name,
                    x,
                    y,
                    row: f32(&pane, shape.y)? - root_y,
                    height,
                    layer,
                    root,
                    path: Vec::new(),
                    seconds: 0.0,
                });
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
        if let Some(lists) = &self.lists {
            found = answered_first(&found, &lists());
        }
        Ok(rows(&found)
            .into_iter()
            .flatten()
            .map(|i| found[i].clone())
            .collect())
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
            let Some(place) = places.get(&self.name(touch)?) else {
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
                });
            }
        }
        Ok(found)
    }

    /// `(drawn.get(p), *textures.get(self._name(p), ()))`, the images a pane draws now or its
    /// layout file gives it, less the empty: what the first rules look a button's pictures up by.
    fn drawn_or_textured(
        &self,
        p: i64,
        drawn: &IndexMap<i64, Option<String>>,
        textures: &IndexMap<String, Vec<String>>,
    ) -> Result<Vec<String>> {
        let mut out: Vec<String> = drawn.get(&p).cloned().flatten().into_iter().collect();
        out.extend(textures.get(&self.name(p)?).cloned().unwrap_or_default());
        out.retain(|image| !image.is_empty());
        Ok(out)
    }

    /// Rule 3's table: an image -> the words of the one button of the layout, shown or not, whose
    /// words are written beside a picture drawing it. The X menu's six buttons each draw their
    /// icon beside their word, and the shortcut bar under the AreaNav draws the same icons alone
    /// (movie18); an image several buttons draw, as the X menu's backing `menu_rgb` is, names none.
    /// The table that came before paired the bar's icons with the layout's groups of words by
    /// count, and the count changed with the X menu open: the bar was said "menu icon1" to "menu
    /// icon6" from then on (Orion, 2026-10-02).
    fn by_picture(
        &mut self,
        root: i64,
        tree: &IndexMap<i64, String>,
        textures: &IndexMap<String, Vec<String>>,
    ) -> Result<IndexMap<String, String>> {
        // A `set[str]` in Python: only its size and its one member are read.
        let mut named: IndexMap<String, IndexSet<String>> = IndexMap::new();
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
            if said.is_empty() {
                continue;
            }
            for &pane in &group {
                if tree.get(&pane).map(String::as_str) == Some(lyt::PICTURE_CLASS) {
                    let mut images: Vec<String> = (self.image_of)(pane)?.into_iter().collect();
                    images.extend(textures.get(&self.name(pane)?).cloned().unwrap_or_default());
                    for image in images {
                        if !image.is_empty() {
                            named.entry(image).or_default().insert(said.clone());
                        }
                    }
                }
            }
        }
        Ok(named
            .into_iter()
            .filter(|(_, said)| said.len() == 1)
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

    /// Does the pane's middle lie within the touch area, as `group` tests it? A pane not laid
    /// out lies nowhere.
    fn lies_in(&mut self, pane: i64, touch: i64) -> Result<bool> {
        let shape = self.layouts.shape;
        let head = self.layouts.pane(touch)?;
        let (across, down) = middle(&head, &shape)?;
        let width = f32(&head, shape.width)? * f32(&head, shape.scale_x)?;
        let height = f32(&head, shape.height)? * f32(&head, shape.scale_y)?;
        let middle = self.middle_of(pane)?;
        Ok(width > 0.0
            && middle.is_some_and(|middle| {
                (middle.0 - across).abs() * 2.0 <= width && (middle.1 - down).abs() * 2.0 <= height
            }))
    }

    fn name(&self, pane: i64) -> Result<String> {
        let raw = self.layouts.name(pane)?;
        let end = raw.iter().position(|&byte| byte == 0).unwrap_or(raw.len());
        Ok(py::decode_ascii(&raw[..end]))
    }
}

/// The touch screen's pixel a pane's place is, from its root's middle, y down.
fn pixel(head: &[u8], shape: &lyt::PaneShape, root_x: f64, root_y: f64) -> Result<(i64, i64)> {
    Ok((
        py::round(WIDTH as f64 / 2.0 + f32(head, shape.x)? - root_x)?,
        py::round(HEIGHT as f64 / 2.0 - (f32(head, shape.y)? - root_y))?,
    ))
}

/// Where a pane's middle is across and up its screen, from its place and its anchor.
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

fn middle(head: &[u8], shape: &lyt::PaneShape) -> Result<(f64, f64)> {
    let width = f32(head, shape.width)? * f32(head, shape.scale_x)?;
    let anchor = usize::try_from(shape.anchored(head)?)
        .ok()
        .and_then(|at| ANCHOR.get(at))
        .ok_or_else(|| Error::index("tuple index out of range"))?;
    Ok((
        f32(head, shape.x)? + (0.5 - anchor) * width,
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

/// Indexes of `found` row by row, top first, each left to right: a target is in a row when its
/// middle is within half its height of the row's first.
pub fn rows(found: &[Target]) -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let order = py::sorted_by_lt((0..found.len()).collect(), |&a, &b| {
        -found[a].row < -found[b].row
    });
    for i in order {
        let mut joined = false;
        for row in &mut rows {
            let first = &found[row[0]];
            if (found[i].row - first.row).abs() < min_of(found[i].height, first.height) / 2.0 {
                row.push(i);
                joined = true;
                break;
            }
        }
        if !joined {
            rows.push(vec![i]);
        }
    }
    rows.into_iter()
        .map(|row| py::sorted_by_lt(row, |&a, &b| found[a].x < found[b].x))
        .collect()
}

/// Python's `min(a, b)` on floats: `a`, unless `b < a`, so a NaN stays where Python puts it.
fn min_of(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}
