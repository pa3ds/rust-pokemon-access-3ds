//! The Move Reminder's two pages, its moves' battle effects and, in Omega Ruby and Alpha
//! Sapphire, their contest effects, of which it draws one (spike B on or_move_reminder_bugs).
//!
//! Each page is a pair of views, its upper screen's and its lower's, and a view draws the layouts
//! of the `G2DUtil` at its +0x5C only while a byte of its own is 1, as the AreaNav's do
//! (`townmap`). The page not shown keeps every flag of its layouts as when it was last drawn, its
//! list's cursor parked on the row it was left on: read as shown, the contest page's cursor said
//! that row as the battle page's list scrolled under it, a move ahead of the cursor, and its
//! details said Hyper Beam's appeal for every move after (or_move_reminder_bugs). The layouts of
//! the views not drawn are put away. X and Y's Move Reminder has the one page, its `ParentView`
//! holding other things where Omega Ruby's holds the contest effects' views (spike B on
//! x_move_reminder): nothing is put away there.
//!
//! ```text
//! app::wazaomoidashi::SelectWazaProc  +0x40  -> app::wazaomoidashi::ParentView
//! ParentView  +0x78 -> UpperView   +0x7C -> LowerView, the battle effects'
//!             +0x80 -> UpperContestView   +0x84 -> LowerContestView, the contest effects'
//!             +0x90 u8 the page shown, 0 the battle effects', 1 the contest effects'
//! each view   +0x5C -> app::util::G2DUtil   +0x68 u8 1 while it is drawn
//! LowerView +0xA4, LowerContestView +0xA8 -> app::tool::MenuCursor, its list's cursor
//! MenuCursor  +0x10 -> its layout's handle, +0x04 its work (a probe on or_move_reminder_bugs)
//! ```

use std::collections::HashSet;
use std::rc::Rc;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::shared::CursorPoll;
use crate::games::gen6::anchors::follow;
use crate::games::gen6::apps::LOADED_CODE;
use crate::games::gen6::screens::rows::{
    CONTEST_LIST, LOWER_CONTEST_VIEW, PAGE, PARENT_VIEW, REMEMBER, REMEMBER_LIST,
};
use crate::games::gen6::screens::townmap::{UTIL, UTIL_CLASS, WORK, util_works};
use crate::py::{Error, Result};

/// `ParentView` -> its views: the battle effects' upper and lower, then the contest effects'.
pub const VIEWS: [i64; 4] = [0x78, 0x7C, 0x80, 0x84];
/// The views of the contest effects' page, shown while `ParentView` +0x90 is 1 (`rows.PAGE`).
pub const CONTEST_VIEWS: [i64; 2] = [0x80, 0x84];
/// A view -> 1 while it is drawn (spike B).
pub const DRAWN: i64 = 0x68;
/// Each lower view, by where `ParentView` keeps it -> its list's `app::tool::MenuCursor`, whose
/// layout is no `G2DUtil`'s: put away with its view's alone, its cursor still said a move. A dict
/// in the Python: `cursors` looks a view up in it.
pub const CURSORS: [(i64, i64); 2] = [(0x7C, 0xA4), (0x84, 0xA8)];
pub const CURSOR_CLASS: &str = "N3app4tool10MenuCursorE";
/// A `MenuCursor` -> its layout's handle, as a `G2DUtil` lists one (`townmap.WORK`).
pub const CURSOR_LAYOUT: i64 = 0x10;

/// `CURSORS[at]`: where a lower view keeps its list's cursor.
pub fn cursors(at: i64) -> Option<i64> {
    CURSORS
        .iter()
        .find(|&&(view, _)| view == at)
        .map(|&(_, cursor)| cursor)
}

/// The layouts of the Move Reminder's page not shown.
pub struct Pages {
    memory: Rc<dyn Memory>,
    running: Rc<dyn Fn() -> Option<String>>,
    process: Rc<dyn Fn() -> Option<i64>>,
    /// The process, its `ParentView`, once its class is checked, and the names of the classes
    /// loaded with it.
    parent: Option<(i64, i64, Rc<ClassNames>)>,
}

impl Pages {
    pub fn new(
        memory: Rc<dyn Memory>,
        running: Rc<dyn Fn() -> Option<String>>,
        process: Rc<dyn Fn() -> Option<i64>>,
    ) -> Self {
        Pages {
            memory,
            running,
            process,
            parent: None,
        }
    }

    /// The works of the layouts of every view of the Move Reminder's not drawn now.
    pub fn put_away(&mut self) -> Result<HashSet<i64>> {
        let Some((parent, names)) = self.parent_view()? else {
            return Ok(HashSet::new());
        };
        // A view is drawn while on the page shown too: the page turns some 0.4 s before the views'
        // bytes do, and a turn's first poll read both pages' lists, "Thrash Normal PP 10/10
        // Toughness", then the page's own, "Thrash Toughness" (or_move_reminder_bugs).
        let contest = *self
            .memory
            .read(parent + PAGE, 1)?
            .first()
            .ok_or_else(|| Error::index("index out of range"))?
            != 0;
        let most = CURSORS
            .iter()
            .fold(DRAWN, |most, &(_, cursor)| most.max(cursor));
        let mut works: HashSet<i64> = HashSet::new();
        for at in VIEWS {
            let Some(view) = follow(&*self.memory, parent, &[at])? else {
                continue;
            };
            let head = self.memory.read(view, most + 4)?;
            let drawn = *head
                .get(DRAWN as usize)
                .ok_or_else(|| Error::index("index out of range"))?;
            if drawn != 0 && CONTEST_VIEWS.contains(&at) == contest {
                continue;
            }
            let util = u32(&head, UTIL)?;
            if util != 0 && names.of_object(util)?.as_deref() == Some(UTIL_CLASS) {
                works.extend(util_works(&*self.memory, util)?);
            }
            let cursor = match cursors(at) {
                Some(cursor) => u32(&head, cursor)?,
                None => 0,
            };
            if cursor != 0 && names.of_object(cursor)?.as_deref() == Some(CURSOR_CLASS) {
                let handle = follow(&*self.memory, cursor, &[CURSOR_LAYOUT])?;
                let work = match handle {
                    None => None,
                    Some(handle) => follow(&*self.memory, handle, &[WORK])?,
                };
                if let Some(work) = work {
                    works.insert(work);
                }
            }
        }
        Ok(works)
    }

    /// The cursors, each layout a cursor is over in the Move Reminder known for a list's: what
    /// else it draws, the chosen move's details, is the screen's own, as the Bag's description is
    /// (`ListCursors.lists`). Each page draws its list and its details in one layout, and taken
    /// for the menu's, no move's details were said.
    ///
    /// Python's roots are a set, added in its order: nothing reads `lists` in order.
    pub fn cursors(&self, cursors: CursorPoll) -> CursorPoll {
        if (self.running)().as_deref() != Some(REMEMBER) {
            return cursors;
        }
        let mut roots: Vec<i64> = Vec::new();
        for p in cursors.under.iter().flatten() {
            if !cursors.lists.contains_key(&p.root) && !roots.contains(&p.root) {
                roots.push(p.root);
            }
        }
        if roots.is_empty() {
            return cursors;
        }
        let mut lists = cursors.lists.clone();
        for root in roots {
            lists.insert(root, HashSet::new());
        }
        CursorPoll { lists, ..cursors }
    }

    /// The Move Reminder's `ParentView` and the names of its module's classes, or None
    /// elsewhere and in a game whose Move Reminder has one page (`rows.LOWER_CONTEST_VIEW`).
    fn parent_view(&mut self) -> Result<Option<(i64, Rc<ClassNames>)>> {
        let Some(process) = (self.process)() else {
            return Ok(None);
        };
        if (self.running)().as_deref() != Some(REMEMBER) {
            return Ok(None);
        }
        if self.parent.as_ref().is_none_or(|parent| parent.0 != process) {
            let Some(parent) = follow(&*self.memory, process, &REMEMBER_LIST[..1])? else {
                return Ok(None);
            };
            // Named afresh for each screen: its classes are in a module (`rows.Rows._found`).
            let names = Rc::new(ClassNames::new(self.memory.clone(), LOADED_CODE));
            let contest = follow(&*self.memory, parent, &CONTEST_LIST[..1])?;
            if names.of_object(parent)?.as_deref() != Some(PARENT_VIEW) {
                return Ok(None);
            }
            let Some(contest) = contest else {
                return Ok(None);
            };
            if names.of_object(contest)?.as_deref() != Some(LOWER_CONTEST_VIEW) {
                return Ok(None);
            }
            self.parent = Some((process, parent, names));
        }
        let (_, parent, names) = self.parent.as_ref().expect("found above");
        Ok(Some((*parent, names.clone())))
    }
}
