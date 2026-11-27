//! What X and Y have alone, as the gen 6 profile asks it (`gen6.own.Own`): the choices their
//! opening draws without a layout, the boy or the girl and their three looks (`xy.intro`).

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::memory::{Cache, phase};
use crate::games::common::announce::Focus;
use crate::games::gen6::apps::RunningApp;
use crate::games::gen6::own::{Kit, Own};
use crate::games::gen6::screens::sources::OwnSources;
use crate::games::gen6::screens::townmap::OwnLists;
use crate::games::xy::intro::{APP, IntroReader};
use crate::py::Result;

/// X's and Y's own reader, asked where the gen 6 profile asks a game's own.
pub struct Xy {
    apps: Rc<RefCell<RunningApp>>,
    phase: Rc<dyn Cache>,
    intro: RefCell<IntroReader>,
}

impl Xy {
    pub fn new(kit: &Kit) -> Self {
        Xy {
            apps: kit.apps.clone(),
            phase: kit.phase.clone(),
            intro: RefCell::new(IntroReader::new(kit.memory.clone())),
        }
    }

    /// `Xy`, as `Game.own` builds it.
    pub fn of(kit: &Kit) -> Rc<dyn Own> {
        Rc::new(Xy::new(kit))
    }
}

impl OwnLists for Xy {}

impl OwnSources for Xy {}

impl Own for Xy {
    fn focus(&self) -> Result<Option<Focus>> {
        // The opening draws its boy or girl and their looks as models, with no word (spike 216).
        let (running, process) = {
            let apps = self.apps.borrow();
            (apps.running.clone(), apps.process)
        };
        let Some(process) = process else {
            return Ok(None);
        };
        if running.as_deref() != Some(APP) {
            return Ok(None);
        }
        phase(&*self.phase, "opening", || {
            self.intro.borrow_mut().poll(process)
        })
    }
}
