//! A message the field shows on the bottom screen, waiting for a tap to go on: Super Training's
//! welcome, "Welcome to S.T.!" over its PlayNav page (as_play_nav, spike 253).
//!
//! The page switches every button of its own off under it (spike 228), and the frame round the
//! PlayNav's pages keeps its arrows and the X menu's icons on: touch mode offered those alone, and
//! "next", the frame's ▶, went on with the message and turned the PlayNav to the page after,
//! leaving Super Training behind. While a message waits, the game draws its ▼ in the message's
//! corner, `f_msg_win_c01.bclim`, at (305, 63) for the welcome, on no touch area. Touch mode
//! offers the ▼ as "continue", where a tap lands on no button.

use std::rc::Rc;

use crate::core::binary::f32;
use crate::core::words::own;
use crate::games::common::layout::Layouts;
use crate::games::common::touch::{HEIGHT, ImageOf, Target, WIDTH};
use crate::games::gen6::apps::FIELD;
use crate::games::gen6::screens::training::pictures;
use crate::py::{self, Result};

/// The ▼ a bottom-screen message draws while it waits for a tap.
pub const ARROW: &str = "f_msg_win_c01.bclim";
pub const CONTINUE: &str = "touch-continue";

/// The bottom screen's message waiting for a tap, as a touch target.
pub struct Waiting {
    layouts: Rc<Layouts>,
    image_of: ImageOf,
}

impl Waiting {
    pub fn new(layouts: Rc<Layouts>, image_of: ImageOf) -> Self {
        Waiting { layouts, image_of }
    }

    /// The ▼ of each message waiting on the bottom screen of the field, "continue".
    pub fn targets(&self, running: Option<&str>) -> Result<Vec<Target>> {
        if running != Some(FIELD) {
            return Ok(Vec::new());
        }
        let layouts = &self.layouts;
        let mut found = Vec::new();
        for (pane, root, _image, across, up) in
            pictures(layouts, &self.image_of, false, &|image| image == ARROW)?
        {
            let x = py::round(WIDTH as f64 / 2.0 + across)?;
            let y = py::round(HEIGHT as f64 / 2.0 - up)?;
            let height = f32(&layouts.pane(pane)?, layouts.shape.height)?;
            let layer = layouts.layer(root)?;
            found.push(Target {
                layer,
                root,
                ..Target::new(pane, &own(CONTINUE, &[]), x, y, up, height)
            });
        }
        Ok(found)
    }
}
