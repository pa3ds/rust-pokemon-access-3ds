//! Layouts for reader tests: works, layouts and panes laid out as spikes 29 and 30 read them.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::layout::{Cached, Layouts};
use crate::games::common::lyt;
use crate::py::Result;

/// Writes layouts into fake memory, and holds the `Layouts` that tracks them.
///
/// Written through `&self`, like `FakeMemory`, so a test can add to the game while a reader holds
/// the `Layouts`. Python's keyword arguments with defaults are a short method and a `_with` one:
/// `screen` and `screen_with`, `pane` and `pane_with_vtable`, `draw` and `draw_with`.
pub struct FakeLayouts {
    pub memory: Rc<FakeMemory>,
    pub shape: lyt::PaneShape,
    pub pane_class: i64,
    pub layout_class: i64,
    pub drawinfo: i64,
    pub works: RefCell<Vec<i64>>,
    children: RefCell<IndexMap<i64, Vec<i64>>>,
    pub layouts: Rc<Layouts>,
}

impl FakeLayouts {
    /// `FakeLayouts(memory, classes)`, for ORAS's panes.
    pub fn new(memory: &Rc<FakeMemory>, classes: &FakeClasses) -> Self {
        FakeLayouts::with_shape(memory, classes, lyt::ORAS)
    }

    /// `FakeLayouts(memory, classes, shape)`.
    pub fn with_shape(
        memory: &Rc<FakeMemory>,
        classes: &FakeClasses,
        shape: lyt::PaneShape,
    ) -> Self {
        FakeLayouts::with_cached(memory, classes, shape, None)
    }

    /// `FakeLayouts(memory, classes, shape, cached)`.
    pub fn with_cached(
        memory: &Rc<FakeMemory>,
        classes: &FakeClasses,
        shape: lyt::PaneShape,
        cached: Option<Cached>,
    ) -> Self {
        FakeLayouts {
            memory: memory.clone(),
            shape,
            pane_class: classes.add("N2nw3lyt4PaneE"),
            layout_class: classes.add(lyt::LAYOUT_CLASS),
            drawinfo: classes.add(lyt::DRAWINFO_CLASS),
            works: RefCell::new(Vec::new()),
            children: RefCell::new(IndexMap::new()),
            layouts: Rc::new(Layouts::with_cached(
                memory.clone(),
                Rc::new(ClassNames::new(memory.clone(), CODE)),
                shape,
                cached,
            )),
        }
    }

    /// `screen(base)`: a drawn layout at `base`, for the bottom screen and with a group.
    pub fn screen(&self, base: i64) -> (i64, i64) {
        self.screen_with(base, false, true)
    }

    /// A drawn layout at `base`. Returns (work, group), the group being where panes go: a
    /// pane under the root, or the root itself when not `grouped`.
    pub fn screen_with(&self, base: i64, top: bool, grouped: bool) -> (i64, i64) {
        let (work, layout, root, mut group) = (base, base + 0x200, base + 0x400, base + 0x600);
        self.memory
            .u32(work + lyt::WORK_LAYOUT, &[layout, self.drawinfo]);
        self.memory.u32(layout, &[self.layout_class]);
        self.memory.u32(layout + lyt::LAYOUT_ROOT, &[root]);
        let width: f32 = if top { 400.0 } else { 320.0 };
        self.memory
            .write(layout + lyt::LAYOUT_WIDTH, &width.to_le_bytes());
        self.pane(root, 0);
        if grouped {
            self.pane(group, root);
        } else {
            group = root;
        }
        self.draw(work);
        self.works.borrow_mut().push(work);
        (work, group)
    }

    /// `pane(at, parent)`: a pane of the Pane class.
    pub fn pane(&self, at: i64, parent: i64) {
        self.pane_with_vtable(at, parent, None);
    }

    /// `pane(at, parent, vtable)`: `vtable` None is the Pane class's.
    pub fn pane_with_vtable(&self, at: i64, parent: i64, vtable: Option<i64>) {
        self.memory.u32(at, &[vtable.unwrap_or(self.pane_class)]);
        self.memory.u32(at + lyt::PARENT, &[parent]);
        self.memory
            .u32(at + lyt::CHILD_LIST, &[at + lyt::CHILD_LIST]);
        self.visible(at, true);
        self.fade(at, 0xFF);
        if parent != 0 {
            self.children
                .borrow_mut()
                .entry(parent)
                .or_default()
                .push(at);
            self.link(parent);
        }
    }

    /// Write the parent's child list as the game lays it out: each node 4 bytes into its child,
    /// pointing at the next, the last pointing back at the list. The child count is left to the
    /// tests, which set it to shape a cursor's layout.
    fn link(&self, parent: i64) {
        let nodes: Vec<i64> = self.children.borrow()[&parent]
            .iter()
            .map(|child| child + lyt::NODE)
            .collect();
        self.memory.u32(parent + lyt::CHILD_LIST, &[nodes[0]]);
        let following = nodes[1..].iter().copied().chain([parent + lyt::CHILD_LIST]);
        for (&node, following) in nodes.iter().zip(following) {
            self.memory.u32(node, &[following]);
        }
    }

    pub fn fade(&self, pane: i64, alpha: i64) {
        self.memory.write(pane + self.shape.alpha, &[alpha as u8]);
    }

    /// `draw(work)`: drawn and alive.
    pub fn draw(&self, work: i64) {
        self.draw_with(work, true, true);
    }

    pub fn draw_with(&self, work: i64, drawn: bool, alive: bool) {
        self.memory.write(
            work + lyt::WORK_ALIVE,
            &[u8::from(alive), 0, 0, u8::from(drawn)],
        );
    }

    /// Record the layout as drawn to a screen this wide, as its DrawInfo's projection does.
    pub fn project(&self, work: i64, screen_width: f64) {
        let projection = (-2.0 / screen_width) as f32;
        self.memory
            .write(work + lyt::WORK_PROJECTION, &projection.to_le_bytes());
    }

    pub fn visible(&self, pane: i64, on: bool) {
        self.memory.write(
            pane + self.shape.flags,
            &[if on { lyt::VISIBLE as u8 } else { 0 }],
        );
    }

    pub fn track(&self) -> Result<()> {
        let works = self.works.borrow().clone();
        self.layouts.track(works)
    }
}
