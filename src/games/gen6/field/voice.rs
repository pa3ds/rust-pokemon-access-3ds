//! Field voices: readers polled each field poll, each saying a line as what it reads changes
//! (GLOSSARY: Field voice). Gen 6's own is the Dowsing Machine's lights (`dowsing.Lights`); a
//! game's own come through the field reader (`FieldReader.voices`), ORAS's decorating cursor in a
//! Secret Base. The profile polls them in an order it writes down (`Gen6Profile.poll_field`).

use std::rc::Rc;

use crate::core::speech::Category;
use crate::games::common::field::things::Field;
use crate::py::Result;

/// The line a field voice says.
#[derive(Debug, Clone, PartialEq)]
pub struct Said {
    pub words: String,
    pub category: Category,
    /// What the log has for it after "field": the words, or the words after what says them,
    /// "cursor: blank, here".
    pub logged: String,
}

/// A field voice: the line it says now, given where the player is, or None while nothing says
/// where; None for nothing to say.
pub type Voice = Rc<dyn Fn(Option<&Field>) -> Result<Option<Said>>>;
