//! The menu cursors' reader: say the item under each cursor on screen, as the menu opens and
//! as the cursor moves, whatever kind of cursor a screen draws (`games.common.cursors`).

use std::collections::HashSet;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::cursors::choice::ChoiceFrames;
use crate::games::common::cursors::frame::MovedFrames;
use crate::games::common::cursors::key::KeyCursors;
use crate::games::common::cursors::menu::MenuCursors;
use crate::games::common::cursors::named::ListCursors;
use crate::games::common::cursors::pane::CursorPanes;
use crate::games::common::cursors::picture::PictureCursors;
use crate::games::common::cursors::plate::plates;
use crate::games::common::cursors::pointer::Pointers;
use crate::games::common::cursors::shared::{CursorItem, CursorPoll, Panes, Said};
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::screen::{Pictures, ScreenText};
use crate::py::{PySet, Result};

/// Tracks menu cursors and reports the screen text under each.
///
/// Python's kinds of cursor keep the kinds they consult (`MovedFrames`, `ChoiceFrames` and
/// `ListCursors` the menu cursors, `CursorPanes` the picture cursors); here the reader holds
/// them all and hands those to each `poll`.
pub struct CursorReader {
    panes: Rc<Panes>,
    said: Rc<Said>,
    menus: MenuCursors,
    pictures: PictureCursors,
    cursor_panes: CursorPanes,
    keys: KeyCursors,
    frames: MovedFrames,
    choices: ChoiceFrames,
    lists: ListCursors,
    pointers: Pointers,
}

impl CursorReader {
    /// `CursorReader(layouts, names)`: no layout files, no cursor panes, no picture names.
    pub fn new(layouts: Rc<Layouts>, names: Rc<ClassNames>) -> Self {
        CursorReader::with(layouts, names, None, false, None, IndexMap::new())
    }

    /// `CursorReader(layouts, names, files)`.
    pub fn with_files(layouts: Rc<Layouts>, names: Rc<ClassNames>, files: Rc<LayoutFiles>) -> Self {
        CursorReader::with(layouts, names, Some(files), false, None, IndexMap::new())
    }

    /// `cursor_panes` reads Sun's cursor pane and its boxes' pointers, which ORAS and X have
    /// none of. `pictures` says what the game calls what a picture draws now, by its image
    /// (`gen6.rom.textures`), or None. `buttons` is pa3ds's own words for a button of pictures
    /// alone, by its pane's name, said in place of that name: the passport's photos.
    pub fn with(
        layouts: Rc<Layouts>,
        names: Rc<ClassNames>,
        files: Option<Rc<LayoutFiles>>,
        cursor_panes: bool,
        pictures: Option<Pictures>,
        buttons: IndexMap<String, String>,
    ) -> Self {
        CursorReader::with_textures(
            layouts,
            names,
            files,
            cursor_panes,
            pictures,
            buttons,
            IndexMap::new(),
        )
    }

    /// `with`, and `textures`: pa3ds's own words for a choice's picture, by its texture's name:
    /// Omega Ruby's boy and girl.
    pub fn with_textures(
        layouts: Rc<Layouts>,
        names: Rc<ClassNames>,
        files: Option<Rc<LayoutFiles>>,
        cursor_panes: bool,
        pictures: Option<Pictures>,
        buttons: IndexMap<String, String>,
        textures: IndexMap<String, String>,
    ) -> Self {
        let panes = Rc::new(Panes::new(layouts, files, pictures, cursor_panes));
        let said = Rc::new(Said::default());
        CursorReader {
            menus: MenuCursors::new(panes.clone(), said.clone(), names),
            pictures: PictureCursors::new(panes.clone(), said.clone()),
            cursor_panes: CursorPanes::new(panes.clone(), said.clone(), buttons),
            keys: KeyCursors::new(panes.clone(), said.clone()),
            frames: MovedFrames::new(panes.clone(), said.clone()),
            choices: ChoiceFrames::new(panes.clone(), said.clone(), textures),
            lists: ListCursors::new(panes.clone(), said.clone()),
            pointers: Pointers::new(panes.clone(), said.clone()),
            panes,
            said,
        }
    }

    /// Read the cursors among these windows from now on, as well as every one found before.
    ///
    /// Most windows are not cursors, so only those shaped as one when found are kept. A cursor
    /// once found is read on every poll after: menus are rebuilt where they were, and a sweep
    /// takes seconds to find a new one. Cursor pictures are found in the layouts themselves.
    pub fn track(&mut self, windows: impl IntoIterator<Item = i64>) -> Result<()> {
        self.menus.track(windows)
    }

    /// Is there a named cursor on screen whose place each tick should sample?
    pub fn sampling(&self) -> bool {
        self.lists.sampling()
    }

    /// Note where each named cursor stands now (`ListCursors.sample`).
    pub fn sample(&mut self, memory: &dyn Memory) -> Result<()> {
        self.lists.sample(memory)
    }

    /// What each cursor on screen is over, whenever that changed, and the rest of what this
    /// poll found (`CursorPoll`).
    ///
    /// `on_screen` is every piece of screen text on screen, from a screen text poll in the same
    /// pass: this reads panes through the same `Layouts`, so the two agree on the frame. Each
    /// kind is read in turn, and a picture or a layout one kind claims is not another's.
    pub fn poll(&mut self, on_screen: &[ScreenText]) -> Result<CursorPoll> {
        let mut items: Vec<CursorItem> = Vec::new();
        self.menus.poll(on_screen, &mut items)?;
        let markers = self.pictures.poll(on_screen, &mut items)?;
        self.cursor_panes
            .poll(on_screen, &mut items, &self.pictures)?;
        let keys = self.keys.poll(on_screen, &mut items)?;
        self.frames.poll(on_screen, &mut items, &self.menus)?;
        let choices = self.choices.poll(on_screen, &mut items, &self.menus)?;
        // A picture another kind already reads is that kind's: the name entry's key cursor is
        // textured as a cursor too, and read twice it was said again every poll (opening.ctm).
        // So is a keyboard's whole layout: its second cursor picture said "“" as it opened.
        let plates = plates(&self.panes, &self.menus, on_screen, &choices)?;
        let taken: HashSet<i64> = markers.keys().chain(keys.keys()).copied().collect();
        let keyboards = PySet::of(keys.values().map(|&(root, _key)| root));
        self.lists.poll(
            on_screen,
            &mut items,
            plates,
            &taken,
            &keyboards,
            &self.menus,
        )?;
        self.pointers.poll(on_screen, &mut items)?;
        Ok(CursorPoll {
            items,
            under: self.said.under.borrow().values().cloned().collect(),
            lists: self.lists.lists(),
            moving: self.lists.moving(),
            rows: self.menus.rows.clone(),
        })
    }
}
