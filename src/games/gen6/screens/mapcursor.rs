//! A tone for where the Hoenn map's cursor is, at each move (`pa3ds.core.tones`).
//!
//! The map is `map_lower`, drawn by the AreaNav, the Pokédex's habitat and the Fly map: a picture of
//! Hoenn (`poke_navi_map.bclim`) and a cursor (`cursol.bclim`) in a pane of its own (spikes 152 and
//! 173). Between places the cursor says nothing, so a player cannot tell moving through sea from
//! standing at an edge. Each poll the cursor's place is taken as a share of the
//! map, across from its left and down from its top, allowing for the map's zoom and slide (the
//! AreaNav draws it at scales up to 2, moved so the place in view stays put), and a tone is played
//! when that changes while the cursor is shown: a move, never the map coming on screen. The tone
//! slides from where the cursor was at the last poll to where it is (`Tones.glide`), and the next
//! poll's slide carries on from it, since a held circle pad glides the cursor on every frame (spike
//! 175).

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::{f32, u32};
use crate::games::common::layout::Layouts;
use crate::games::common::lyt;
use crate::py::{self, Error, Kind, Result};

pub const MAP: &str = "poke_navi_map.bclim";
pub const CURSOR: &str = "cursol.bclim";
pub const HALF_WIDTH: f64 = 160.0;
/// Half the bottom screen, from its middle, where a layout places its panes.
pub const HALF_HEIGHT: f64 = 120.0;

/// `Callable[[int], str | None]`: the file a picture draws now. It reads memory.
pub type ImageOf = Rc<dyn Fn(i64) -> Result<Option<String>>>;
/// `Callable[[tuple[float, float], tuple[float, float]], None]`: slide a tone from one place to
/// another.
pub type Tone = Rc<dyn Fn((f64, f64), (f64, f64))>;

/// Plays a tone as the map's cursor moves.
pub struct MapCursor {
    layouts: Rc<Layouts>,
    image_of: ImageOf,
    tone: Option<Tone>,
    /// (root, its panes' names) -> its map picture and cursor picture, or None for a layout
    /// without both: found once, since finding reads every picture's material.
    found: IndexMap<(i64, Vec<String>), Option<(i64, i64)>>,
    /// The map's root and the cursor's place on it at the last poll.
    last: Option<(i64, (f64, f64))>,
}

impl MapCursor {
    pub fn new(layouts: Rc<Layouts>, image_of: ImageOf, tone: Option<Tone>) -> Self {
        MapCursor {
            layouts,
            image_of,
            tone,
            found: IndexMap::new(),
            last: None,
        }
    }

    pub fn poll(&mut self) -> Result<()> {
        let Some(tone) = self.tone.clone() else {
            return Ok(());
        };
        let layouts = self.layouts.clone();
        for root in layouts.roots() {
            let placement = layouts.placement(root)?;
            let Some(placement) = placement else {
                continue;
            };
            if !placement.shown || placement.top {
                continue;
            }
            let key = (root, layouts.pane_names(root));
            let parts = match self.found.get(&key) {
                Some(&parts) => parts,
                None => self.parts(root)?,
            };
            let Some((picture, cursor)) = parts else {
                continue;
            };
            let shown = layouts.placement(cursor)?;
            let pane = u32(&layouts.pane(cursor)?, lyt::PARENT)?;
            let Some(shown) = shown else {
                continue;
            };
            if !shown.shown || !self.on_screen(root, pane)? {
                continue;
            }
            let place = self.share(picture, pane)?;
            // A move, not a map coming on screen: the field's AreaNav sounded at every return
            // to the field (movie7).
            if let Some(last) = self.last
                && last.0 == root
                && last.1 != place
            {
                tone(last.1, place);
            }
            self.last = Some((root, place));
            return Ok(());
        }
        self.last = None;
        Ok(())
    }

    /// The cursor's place on the map, across from the left and down from the top, each from 0
    /// to 1, to a hundredth.
    pub fn share(&self, picture: i64, cursor: i64) -> Result<(f64, f64)> {
        let shape = &self.layouts.shape;
        let (head, at) = (self.layouts.pane(picture)?, self.layouts.pane(cursor)?);
        let width = f32(&head, shape.width)? * f32(&head, shape.scale_x)?;
        let height = f32(&head, shape.height)? * f32(&head, shape.scale_y)?;
        if width == 0.0 {
            return Err(Error::new(Kind::ZeroDivisionError, "division by zero"));
        }
        let across = (f32(&at, shape.x)? - f32(&head, shape.x)?) / width + 0.5;
        if height == 0.0 {
            return Err(Error::new(Kind::ZeroDivisionError, "division by zero"));
        }
        let down = (f32(&head, shape.y)? - f32(&at, shape.y)?) / height + 0.5;
        Ok((py::round_to(across, 2), py::round_to(down, 2)))
    }

    /// Whether a pane's middle is on the bottom screen: the field's AreaNav keeps the cursor
    /// drawn off it, at x 190, while its map slides under the player's walk (spike 173).
    fn on_screen(&self, root: i64, pane: i64) -> Result<bool> {
        let shape = &self.layouts.shape;
        let (head, at) = (self.layouts.pane(root)?, self.layouts.pane(pane)?);
        Ok(
            (f32(&at, shape.x)? - f32(&head, shape.x)?).abs() <= HALF_WIDTH
                && (f32(&at, shape.y)? - f32(&head, shape.y)?).abs() <= HALF_HEIGHT,
        )
    }

    /// The layout's map picture and cursor picture, or None; kept once any of its pictures
    /// is named, since until its archive has been read none is, and the habitat's map was taken
    /// for no map for good the poll it came on screen (movie7, 2026-10-02).
    fn parts(&mut self, root: i64) -> Result<Option<(i64, i64)>> {
        let tree = self.layouts.tree(root);
        let mut drawn: IndexMap<Option<String>, i64> = IndexMap::new();
        for (&pane, kind) in &tree {
            if kind == lyt::PICTURE_CLASS {
                drawn.insert((self.image_of)(pane)?, pane);
            }
        }
        let (picture, cursor) = (
            drawn.get(&Some(MAP.to_string())).copied(),
            drawn.get(&Some(CURSOR.to_string())).copied(),
        );
        let parts = match (picture, cursor) {
            (Some(picture), Some(cursor)) => Some((picture, cursor)),
            _ => None,
        };
        // `any(drawn.keys() - {None})`: a name, and not an empty one.
        if drawn
            .keys()
            .any(|image| image.as_deref().is_some_and(|image| !image.is_empty()))
        {
            self.found
                .insert((root, self.layouts.pane_names(root)), parts);
        }
        Ok(parts)
    }
}
