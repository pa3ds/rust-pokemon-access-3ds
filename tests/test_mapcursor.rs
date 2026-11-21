//! The map's cursor sounds a tone as it moves (`gen6.screens.mapcursor`, spikes 152 and 175).

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{FakeClasses, FakeMemory};
use crate::games::common::lyt;
use crate::games::gen6::screens::mapcursor::{CURSOR, MapCursor};

const BASE: i64 = 0x08300000;
const PICTURE: i64 = 0x08301000;
const HOLDER: i64 = 0x08302000;
const POINTER: i64 = 0x08303000;

/// `_place(fake, pane, x, y, width, height)`.
fn place(fake: &FakeLayouts, pane: i64, x: f64, y: f64, width: f64, height: f64) {
    let shape = fake.shape;
    for (at, value) in [
        (shape.x, x),
        (shape.y, y),
        (shape.width, width),
        (shape.height, height),
        (shape.scale_x, 1.0),
        (shape.scale_y, 1.0),
    ] {
        fake.memory
            .write(pane + at, &(value as f32).to_le_bytes());
    }
}

#[test]
fn test_a_move_of_the_cursor_sounds_and_the_map_coming_on_screen_does_not() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let fake = FakeLayouts::new(&memory, &classes);
    let (_, root) = fake.screen_with(BASE, false, false);
    let picture = classes.add(lyt::PICTURE_CLASS);
    fake.pane_with_vtable(PICTURE, root, Some(picture));
    fake.pane(HOLDER, root);
    fake.pane_with_vtable(POINTER, HOLDER, Some(picture));
    place(&fake, root, 0.0, 0.0, 0.0, 0.0);
    place(&fake, PICTURE, 0.0, 0.0, 320.0, 240.0);
    place(&fake, HOLDER, 0.0, 0.0, 0.0, 0.0);
    fake.track().unwrap();
    let images: IndexMap<i64, &str> =
        IndexMap::from([(PICTURE, "poke_navi_map.bclim"), (POINTER, CURSOR)]);
    #[allow(clippy::type_complexity)]
    let tones: Rc<RefCell<Vec<((f64, f64), (f64, f64))>>> = Rc::new(RefCell::new(Vec::new()));
    let mut cursor = MapCursor::new(
        fake.layouts.clone(),
        Rc::new(move |pane: i64| Ok(images.get(&pane).map(|image| image.to_string()))),
        Some({
            let tones = tones.clone();
            Rc::new(move |was: (f64, f64), now: (f64, f64)| tones.borrow_mut().push((was, now)))
        }),
    );
    cursor.poll().unwrap();
    assert_eq!(*tones.borrow(), Vec::new()); // the map coming on screen
    place(&fake, HOLDER, 80.0, -60.0, 0.0, 0.0);
    fake.layouts.refresh();
    cursor.poll().unwrap();
    assert_eq!(*tones.borrow(), vec![((0.5, 0.5), (0.75, 0.75))]);
    cursor.poll().unwrap();
    assert_eq!(tones.borrow().len(), 1); // held still
    // Drawn off the screen, as the field's AreaNav keeps it while its map slides: no tone.
    place(&fake, HOLDER, 190.0, -60.0, 0.0, 0.0);
    fake.layouts.refresh();
    cursor.poll().unwrap();
    assert_eq!(tones.borrow().len(), 1);
}
