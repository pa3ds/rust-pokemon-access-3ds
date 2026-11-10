//! The Pokédex's habitat page: which places on its map are the Pokémon's, and where the cursor is.
//!
//! The top screen says "Poochyena's Habitat" beside the place under the map's cursor; the bottom
//! screen is the AreaNav's map (`map_lower`), each place a touch area (`nw::lyt::Bounding`), and a
//! place the Pokémon lives in draws a `poke_point.bclim` picture over it, no word anywhere saying
//! which (spike 152). The ROM names each touch area (`archives.map_places`), so the page says the
//! places drawn so as soon as the More/Habitat bar is on Habitat, "Poochyena's Habitat Route 101,
//! Route 102, Route 103", and again only when up and down turn to another Pokémon.
//!
//! The place under the map's cursor is said once the map is open, not while the bar is still on
//! Habitat, where it named a place the player had not moved to. The map is open
//! when the top screen's three icons, grass, water and a fishing rod, have slid onto the screen, which
//! they do only then (spike 153). A move says the place alone, and on one of the Pokémon's places
//! how it is found, by the icons drawn in colour: "Found in Route 101 grass". An
//! icon in colour draws the image its layout file gives it, `map_ico_00.bclim`; one in grey, the
//! game's other image, `map_ico_00_1.bclim`.
//!
//! Both of the top screen's words are this reader's while the map is up (`owns`), as the HP box is
//! the battle reader's: left to screen text, every move of the cursor said the title again.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::f32;
use crate::core::memory::Memory;
use crate::core::speech::{Category, Speech};
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::{LayoutFile, LayoutFiles};
use crate::games::common::lyt;
use crate::games::common::screen::ScreenText;
use crate::games::gen6::rom::textures;
use crate::py::{self, Error, Log, Result};

/// What the map draws over a place the Pokémon lives in: a touch area's own picture, two of them
/// a place and one drawn (spike 152).
pub const POINT: &str = "poke_point.bclim";
/// The top screen's icons for how a Pokémon is found, by the image each draws in colour, and
/// pa3ds's word for each (Orion's, 2026-10-01): the game draws them and never names them. Grass
/// was seen in colour first (spike 153), fishing on Route 111; a Pokémon met
/// in seaweed under water lights grass, as the game shows it.
///
/// Python's `dict`, as (image, word) pairs: it is only ever looked up.
pub const WAYS: [(&str, &str); 3] = [
    ("map_ico_00.bclim", "habitat-grass"),
    ("map_ico_01.bclim", "habitat-water"),
    ("map_ico_02.bclim", "habitat-fishing"),
];
/// Half the top screen's height: the icons' middle is within it only once the map is open.
pub const HALF_HEIGHT: f64 = 120.0;
/// Said before a place drawn as the Pokémon's: the game draws it and never says it (Orion's
/// words, 2026-10-01, for the longer "is a known habitat" first tried).
pub const FOUND: &str = "habitat-found";

/// Says the habitat page's places, and owns its top screen's words while its map is up.
pub struct HabitatReader {
    memory: Rc<dyn Memory>,
    layouts: Rc<Layouts>,
    files: Rc<LayoutFiles>,
    /// A touch area's pane name -> its place, in the AreaNav's order.
    places: IndexMap<String, String>,
    speech: Rc<Speech>,
    log: Log,
    separator: &'static str,
    /// A layout's pane names -> `POINT` for the map, "panel" for the title's, drawn with the
    /// icons of `WAYS`, else None.
    kinds: HashMap<Vec<String>, Option<&'static str>>,
    owned: HashSet<i64>,
    /// The title and its places, or the place and its ways, as last read, said once read the
    /// same twice: the pictures may change a poll after the words.
    pending: Option<Vec<String>>,
    title: Option<String>,
    lit: Vec<String>,
    /// The title, its places and the place under the cursor last said.
    place: Option<String>,
}

impl HabitatReader {
    pub fn new(
        memory: Rc<dyn Memory>,
        layouts: Rc<Layouts>,
        files: Rc<LayoutFiles>,
        places: IndexMap<String, String>,
        speech: Rc<Speech>,
        log: Log,
        pauses: bool,
    ) -> Self {
        HabitatReader {
            memory,
            layouts,
            files,
            places,
            speech,
            log,
            separator: if pauses { ": " } else { " " },
            kinds: HashMap::new(),
            owned: HashSet::new(),
            pending: None,
            title: None,
            lit: Vec::new(),
            place: None,
        }
    }

    pub fn owns(&self, piece: &ScreenText) -> bool {
        self.owned.contains(&piece.r#box)
    }

    pub fn poll(&mut self, on_screen: &[ScreenText]) -> Result<()> {
        self.owned = HashSet::new();
        if self.places.is_empty() {
            return Ok(());
        }
        let mut words: Vec<&ScreenText> = Vec::new();
        for piece in on_screen {
            if piece.top && self.kind(piece.root) == Some("panel") {
                words.push(piece);
            }
        }
        let lit = if words.len() == 2 {
            self.drawn()?
        } else {
            None
        };
        let Some(lit) = lit else {
            self.pending = None;
            self.title = None;
            self.place = None;
            return Ok(());
        };
        // `sorted(words, key=lambda p: p.left)` of two: Python swaps them only when the second's
        // key is strictly less, which is not what comparing floats totally does for a NaN.
        let (title, place) = if words[1].left < words[0].left {
            (words[1], words[0])
        } else {
            (words[0], words[1])
        };
        self.owned = HashSet::from([title.r#box, place.r#box]);
        if self.title.as_ref() != Some(&title.text) {
            if self.held(
                std::iter::once(title.text.clone())
                    .chain(lit.iter().cloned())
                    .collect(),
            ) {
                let text = if !lit.is_empty() {
                    format!("{}{}{}", title.text, self.separator, lit.join(", "))
                } else {
                    title.text.clone()
                };
                (self.log)(&format!("habitat: {text}  [layout {:#x}]", title.root));
                self.speech.say(&text, Category::Screen);
                self.title = Some(title.text.clone());
                self.lit = lit;
                self.place = None;
            }
            return Ok(());
        }
        let ways = self.ways(title.root)?;
        match ways {
            None => self.place = None,
            Some(ways) => {
                if self.place.as_ref() != Some(&place.text)
                    && self.held(
                        std::iter::once(place.text.clone())
                            .chain(ways.iter().cloned())
                            .collect(),
                    )
                {
                    self.place = Some(place.text.clone());
                    let mut text = place.text.clone();
                    if self.lit.contains(&place.text) {
                        text = own(FOUND, &[("place", &place.text)])
                            + &if !ways.is_empty() {
                                format!("{}{}", self.separator, ways.join(", "))
                            } else {
                                String::new()
                            };
                    }
                    (self.log)(&format!(
                        "habitat place: {text}  [layout {:#x}]",
                        place.root
                    ));
                    self.speech.say(&text, Category::Menu);
                }
            }
        }
        Ok(())
    }

    /// Has this read the same as the poll before? Remembers it when it has not.
    fn held(&mut self, read: Vec<String>) -> bool {
        if self.pending.as_ref() == Some(&read) {
            return true;
        }
        self.pending = Some(read);
        false
    }

    /// How the Pokémon is found at the place under the map's cursor, by the icons in colour,
    /// or None while the icons are off the screen, the map not open.
    fn ways(&self, root: i64) -> Result<Option<Vec<String>>> {
        let Some(file) = self.files.matching(&self.layouts.pane_names(root)) else {
            return Ok(None);
        };
        let root_y = f32(&self.layouts.pane(root)?, lyt::Y)?;
        let mut icons: Vec<(i64, String)> = Vec::new();
        for (pane, kind) in self.layouts.tree(root) {
            if kind == lyt::PICTURE_CLASS
                && let Some(icon) = drawn_with(&file, &self.name(pane)?)
            {
                icons.push((pane, icon));
            }
        }
        if icons.is_empty() {
            return Ok(None);
        }
        for (pane, _icon) in &icons {
            let near = (f32(&self.layouts.pane(*pane)?, lyt::Y)? - root_y).abs() < HALF_HEIGHT;
            if !near {
                return Ok(None);
            }
        }
        let mut found: Vec<String> = Vec::new();
        for (pane, icon) in &icons {
            let at = textures::drawn_at(&*self.memory, *pane)?;
            if let Some(at) = at
                && self.files.image_at(at).as_deref() == Some(icon.as_str())
            {
                found.push(own(way(icon)?, &[]));
            }
        }
        Ok(Some(found))
    }

    /// The places the map on screen draws as the Pokémon's, in the AreaNav's order, or None
    /// when no map is on screen.
    fn drawn(&mut self) -> Result<Option<Vec<String>>> {
        for root in self.layouts.roots() {
            if self.kind(root) != Some(POINT) {
                continue;
            }
            let placement = self.layouts.placement(root)?;
            if !placement.is_some_and(|placement| placement.shown) {
                continue;
            }
            let file = self.files.matching(&self.layouts.pane_names(root));
            let tree = self.layouts.tree(root);
            let mut lit: HashSet<String> = HashSet::new();
            for (pane, kind) in &tree {
                let name = self.name(*pane)?;
                if kind != lyt::BOUNDING_CLASS || !self.places.contains_key(&name) {
                    continue;
                }
                for child in self.layouts.children(root, *pane) {
                    let shown = self.layouts.placement(child)?;
                    if tree.get(&child).map(String::as_str) == Some(lyt::PICTURE_CLASS)
                        && let Some(file) = &file
                        && file
                            .textures
                            .get(&self.name(child)?)
                            .is_some_and(|held| held.len() == 1 && held[0] == POINT)
                        && shown.is_some_and(|shown| shown.shown)
                    {
                        lit.insert(name.clone());
                    }
                }
            }
            // `dict.fromkeys(...)`: each place once, in the order its pane comes in `places`.
            let mut drawn: Vec<String> = Vec::new();
            for (pane, place) in &self.places {
                if lit.contains(pane) && !drawn.contains(place) {
                    drawn.push(place.clone());
                }
            }
            return Ok(Some(drawn));
        }
        Ok(None)
    }

    fn kind(&mut self, root: i64) -> Option<&'static str> {
        let names = self.layouts.pane_names(root);
        if !self.kinds.contains_key(&names) {
            let Some(file) = self.files.matching(&names) else {
                return None; // Its file may not have been read yet: asked again next poll.
            };
            let drawn: HashSet<&String> = file.textures.values().flatten().collect();
            let kind = if drawn.iter().any(|texture| texture.as_str() == POINT) {
                Some(POINT)
            } else if WAYS
                .iter()
                .any(|(image, _)| drawn.iter().any(|texture| texture.as_str() == *image))
            {
                Some("panel")
            } else {
                None
            };
            self.kinds.insert(names.clone(), kind);
        }
        self.kinds[&names]
    }

    fn name(&self, pane: i64) -> Result<String> {
        let name = self.layouts.name(pane)?;
        let end = name
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(name.len());
        Ok(py::decode_ascii(&name[..end]))
    }
}

/// The icon of `WAYS` a picture of the top screen's panel is drawn with in colour, or None.
fn drawn_with(file: &LayoutFile, picture: &str) -> Option<String> {
    let icons = file.textures.get(picture).map_or(&[][..], Vec::as_slice);
    if icons.len() == 1 && WAYS.iter().any(|(image, _)| *image == icons[0]) {
        Some(icons[0].clone())
    } else {
        None
    }
}

/// `WAYS[icon]`.
fn way(icon: &str) -> Result<&'static str> {
    WAYS.iter()
        .find(|(image, _)| *image == icon)
        .map(|&(_, word)| word)
        .ok_or_else(|| Error::key(py::repr(icon)))
}
