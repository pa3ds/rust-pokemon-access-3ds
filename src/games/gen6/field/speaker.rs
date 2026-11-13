//! Who is speaking a field message.
//!
//! The game draws no name plate and keeps no name for the people on its maps, but it does say which
//! of them a message belongs to: a talk object holds the message window it is showing and the model
//! showing it (`talk`), and the model says which character it is (`mmodel`). Spike 73 read a whole
//! conversation off this — Norman, the trainer who had just battled him, and Wally, each line on the
//! right one.
//!
//! A name for a character is `pa3ds.core.names`: the shipped table, or what the player has called
//! them. A character nobody has named is left unsaid, so dialogue reads exactly as it did before.
//!
//! Nothing is read until a page is about to be announced. The actor is written as the message begins
//! and cleared between messages (spike 73), so asking at that moment is asking when it is true, and a
//! field with nobody talking costs nothing.
//!
//! Whoever spoke last is remembered, because that is who a player wants to name: they have just
//! heard a line from somebody the game does not name, and the key is pressed after the line, not
//! during it.
//!
//! Bound by:
//! - ADR-0023: a speaker is said when it changes, and a work object is whoever it was filled with.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::names::Names;
use crate::games::common::field::things::{Naming, own_name};
use crate::games::{gen6::field::mmodel, gen6::field::talk};
use crate::py::Result;

/// Who spoke: the character, and the name of the one model alone (`Interactable.own`).
pub type Speaker = (i64, Naming);

/// Tracks the field's talk objects and says who is speaking a window.
///
/// Shared: the profile hands `last` and `name_for` to the readers that ask, so the mutable parts
/// are in `Cell` and `RefCell` and every method takes `&self`.
pub struct SpeakerReader {
    memory: Rc<dyn Memory>,
    names: Rc<Names>,
    /// Where the game keeps the sprite a move model is drawn with (`mmodel.drawn_as`).
    drawn: i64,
    /// Where the game's things' and Pokémon's models start (`mmodel.drawn_as`).
    models: Option<(Option<i64>, i64)>,
    /// Who last spoke, for a rename: a player names somebody having just heard them, which
    /// is a moment later than the page that named them.
    last: Cell<Option<Speaker>>,
    /// Every talk object found. Kept for good: a talk object's class is a DLL's while a talk
    /// runs, so it cannot be found again exactly when it matters.
    talking: RefCell<Vec<i64>>,
}

impl SpeakerReader {
    /// `SpeakerReader(memory, names, drawn, models)`.
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<Names>,
        drawn: i64,
        models: Option<(Option<i64>, i64)>,
    ) -> Self {
        SpeakerReader {
            memory,
            names,
            drawn,
            models,
            last: Cell::new(None),
            talking: RefCell::new(Vec::new()),
        }
    }

    /// `SpeakerReader(memory, names)`: `drawn` left to its default, `mmodel.DRAWN`.
    pub fn with_default_drawn(memory: Rc<dyn Memory>, names: Rc<Names>) -> Self {
        SpeakerReader::new(memory, names, mmodel::DRAWN, None)
    }

    /// Keep these talk objects, as well as every one kept before.
    pub fn track(&self, objects: impl IntoIterator<Item = i64>) {
        for at in objects {
            let mut talking = self.talking.borrow_mut();
            if !talking.contains(&at) {
                talking.push(at);
            }
        }
    }

    /// Whether a talk object kept is showing this message window, with anyone speaking or not:
    /// a sign's talk has nobody, and a talk object not found yet has no say at all.
    pub fn shows(&self, window: i64) -> Result<bool> {
        for &at in self.talking.borrow().iter() {
            if u32(&self.memory.read(at + talk::WINDOW, 4)?, 0)? == window {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Which character is showing this message window, or None when none is.
    pub fn character_of(&self, window: i64) -> Result<Option<i64>> {
        Ok(self.speaker_of(window)?.map(|speaker| speaker.0))
    }

    /// Who is showing this message window, or None when nobody is.
    pub fn speaker_of(&self, window: i64) -> Result<Option<Speaker>> {
        for &at in self.talking.borrow().iter() {
            let head = self.memory.read(at + talk::WINDOW, talk::HEAD)?;
            if u32(&head, 0)? != window {
                continue;
            }
            let actor = u32(&head, talk::ACTOR - talk::WINDOW)?;
            if actor == 0 {
                continue;
            }
            let head = self.memory.read(actor, mmodel::HEAD)?;
            let character = mmodel::drawn_as(&*self.memory, actor, &head, self.drawn, self.models)?;
            if character != 0 {
                return Ok(Some((
                    character,
                    own_name(mmodel::zone_of(&head)?, mmodel::index_of(&head)?)?,
                )));
            }
        }
        Ok(None)
    }

    /// What to call whoever is speaking this window, or None: their own name, or their
    /// character's.
    ///
    /// Remembers the speaker either way: somebody nameless is exactly who a rename is for, so
    /// the one worth remembering is the one this returns None for.
    pub fn name_for(&self, window: i64) -> Result<Option<String>> {
        let Some(speaker) = self.speaker_of(window)? else {
            return Ok(None);
        };
        self.last.set(Some(speaker));
        let (character, (_, zone, slot)) = speaker;
        Ok(match self.names.place(zone, slot) {
            Some(own) if !own.is_empty() => Some(own),
            _ => self.names.sprite(character),
        })
    }

    /// Who last spoke, or None if nobody has this session.
    pub fn last(&self) -> Option<Speaker> {
        self.last.get()
    }
}
