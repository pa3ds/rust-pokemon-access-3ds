//! Navigation: the keys that walk the list of what is on the map, in one of two modes.
//!
//! **List mode** walks the whole map. Two keys step between categories and two between a
//! category's items, nearest first, or in the zone's tables' order with `[pa3ds.navigation]
//! sort_by_distance` off, so a thing keeps its place as the player walks; a fifth says the chosen
//! item again. **Direction mode** looks one way at a time: four keys, one a compass point, each
//! cycling through the category's items on that side of the player within `RANGE` cells each way
//! (a half-box: north is everything with a smaller z, so a thing north-east is heard under north
//! and under east both), nearest first. A
//! press of another direction starts again from the nearest that way. The apostrophe toggles the
//! mode (the user, 2026-09-21; moved off M on 2026-09-22, which is the emulator's Start). The
//! category is one setting under both modes, and "all" is a category of its own, the one both
//! start on.
//!
//! Each press reads the map afresh, so the bearing is from where the player stands now and a
//! person is where they have walked to. The chosen item is kept by what it is, not by its place in
//! the list: walking re-sorts the list, and keeping a place meant K tracked one person and then, a
//! few steps on, another (the user, 2026-09-21). An item that has gone — a ball picked up — gives
//! way to the nearest.
//!
//! Spoken as "exits, 3" on reaching a category and "door, 1 of 3, 1 north" on an item (the user,
//! 2026-09-21: the count on a category, the place on an item), the place counted within the
//! direction in direction mode. A category with nothing in it is skipped over, "nothing here"
//! when the map is empty, "nothing north" or "no exits north" when a direction is, and "no map"
//! off the field. An exit into another place says it, in the game's own words out of the ROM:
//! "edge to Route 101, 2 of 3, 5 north" (the user, 2026-09-24). A door into a town's own house
//! leads nowhere new, as the game has it, and says nothing more; zone numbers stay in the log.
//!
//! Each thing is said by the name it has been given, and by what it is when it has none: "the
//! Mart's sign, 1 of 3, 1 north" rather than "sign, 1 of 3, 1 north". A name is the player's or
//! the shipped table's (`pa3ds.core.names`), and **one key renames whatever is chosen**, through
//! a dialog — the player is the one standing in front of the thing, and nobody else is in a
//! position to name it.
//!
//! Keys come in as names and are looked up in the mode's bindings here, since one key means one
//! thing in list mode and another in direction mode: O steps the category in one and looks north
//! in the other. A key bound in either mode is kept from the emulator in both.
//!
//! **P finds the way to whatever is chosen**, in either mode (the user, 2026-09-24), and says it
//! as its legs: "5 east, 2 north". Then it **guides**: as the player walks the path, the next leg
//! is said the moment they reach each turn ("2 north"), and the thing itself when they get there
//! ("door, 1 north"). A player who steps off the path is given a new one from where they stand,
//! said when it turns them and taken quietly when it goes on the way they are walking; a warp ends
//! the guiding, since the path's cells belong to the map the player has left.
//!
//! Bound by:
//! - ADR-0009: Shift alone drags in touch mode only while one of Pokémon-Amie's screens runs, held
//!   or tapped as `drag` says; elsewhere touch mode keeps its keys with Shift.
//! - ADR-0024: nearest first by category unless `sort_by_distance` is off, all a category, and
//!   direction mode a half-box 10 cells deep, nearest first always.
//! - ADR-0037: a search's find is guided to, and the detector's only with the DexNav open in touch
//!   mode.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::config::{DRAG_TOGGLE, Navigation};
use crate::core::keys::{SHIFT, SHIFT_DOWN, SHIFT_UP};
use crate::core::names::Names;
use crate::core::pathfind::{self, AHEAD, BEHIND, Cell, LEFT, RIGHT};
use crate::core::speech::{Category, Speech};
use crate::core::words::own;
use crate::games::common::field::chunk::CLIMB;
use crate::games::common::field::guide::{Clock, FieldSource, Guide, Key, Place};
use crate::games::common::field::keys::{
    Bindings, CATEGORY_NEXT, CATEGORY_PREVIOUS, COORDINATES, DEFAULT_BINDINGS, DIRECTIONS, EAST,
    ENEMY_SIDE, HIDE_UNREACHABLE, ITEM_COORDINATES, ITEM_NEXT, ITEM_PREVIOUS, LIST, MODE_TOGGLE,
    MODES, NORTH, OUR_SIDE, PATH_NEXT, PATH_PREVIOUS, PATHFIND, PATHFIND_HM, PATHFIND_OFF, RENAME,
    RENAME_SPEAKER, SOUTH, TOUCH, WEST,
};
use crate::games::common::field::things::Naming as Named;
use crate::games::common::field::things::{
    self, ALL, Field, Interactable, SPRITE, category_said, kind_said,
};
use crate::games::common::touch::ACTIONS as TOUCH_ACTIONS;
use crate::games::common::touch::NOTHING as TOUCH_NOTHING;
use crate::games::common::touch::TouchTargets;
use crate::games::common::touch::{DRAG, TAP};
use crate::py::{self, Error, Log, Result};
use crate::ux::dialog::Asker;

/// `(ALL, *things.CATEGORIES)`.
pub const CATEGORIES: [&str; 13] = {
    let mut out = [ALL; 13];
    let mut at = 0;
    while at < things::CATEGORIES.len() {
        out[at + 1] = things::CATEGORIES[at];
        at += 1;
    }
    out
};

/// A dict in Python, read here by `find` on the key.
pub const RELATIVE: [(&str, &str); 4] =
    [(NORTH, AHEAD), (EAST, RIGHT), (SOUTH, BEHIND), (WEST, LEFT)];

/// What a dialog's answer is for, as `dialog.Answer` carries it.
/// The answer to a key with nothing to answer about: an empty map, no category with anything in
/// it, nothing chosen.
pub const NOTHING_HERE: &str = "key-nothing-here";

pub const RENAMING: &str = "interactable";
pub const SPEAKING: &str = "speaker";

/// The rename box's checkbox, for somebody or something that can be named alone, apart from
/// everyone drawn as it: ticked as it opens, since the shipped names already call everybody what
/// they look like (Orion, 2026-10-11).
pub const ONLY: &str = "rename-only-this-one";

/// How far direction mode looks, in cells each way: a box, so a thing 10 north and 10 east is
/// in. The top screen shows about 13 cells across and 9 deep (measured from a rig screenshot,
/// 2026-09-21), so this is a little over a screen sideways and two screens ahead; the user's
/// choice, so the keys do not cycle the whole map.
pub const RANGE: i64 = 10;

/// Whether `cell` is on `direction`'s side of `origin`, no more than `within` cells away
/// either way.
pub fn towards(direction: &str, origin: (i64, i64), cell: (i64, i64), within: i64) -> bool {
    let (dx, dz) = (cell.0 - origin.0, cell.1 - origin.1);
    if dx.abs() > within || dz.abs() > within {
        return false;
    }
    if direction == NORTH {
        return dz < 0;
    }
    if direction == SOUTH {
        return dz > 0;
    }
    if direction == EAST {
        return dx > 0;
    }
    dx < 0
}

/// `Callable[[int, int], str | None]`: what the game calls the place an exit from one zone into
/// another arrives in. It reads the game's words, so it can raise.
pub type LeadsTo = Rc<dyn Fn(i64, i64) -> Result<Option<String>>>;
/// `Callable[[int, int], str | None]`: the street an exit comes out on.
pub type Street = Rc<dyn Fn(i64, i64) -> Result<Option<String>>>;
/// `Callable[[], tuple[int, Naming | None] | None]`: the character that last spoke, and the
/// name of the one model alone.
pub type Speaker = Rc<dyn Fn() -> Option<(i64, Option<Named>)>>;
/// `Callable[[bool], str | None]`: the player's side of the battle as last said, or the enemy's.
/// It reads a contest's portraits (`oras.screens.contest`), so it can raise.
pub type Sides = Rc<dyn Fn(bool) -> Result<Option<String>>>;
/// `Callable[[int, int], bool]`: whether a slot on a map is a sign that says only the name of a
/// door there.
pub type DoorSign = Rc<dyn Fn(i64, i64) -> bool>;

/// `Navigator.__init__`'s arguments after `field`, `speech` and `log`, each as the Python
/// defaults it: set what differs with `..Options::default()`.
pub struct Options {
    /// None is `DEFAULT_BINDINGS`.
    pub bindings: Option<Bindings>,
    pub names: Option<Rc<Names>>,
    pub asker: Option<Asker>,
    pub navigation: Option<Navigation>,
    pub speaker: Option<Speaker>,
    pub leads_to: Option<LeadsTo>,
    pub street: Option<Street>,
    /// None is `py::time::monotonic`.
    pub clock: Option<Clock>,
    pub routes: bool,
    pub slack: i64,
    pub moves: Option<IndexMap<i64, String>>,
    pub bike: Option<String>,
    pub acro: Option<String>,
    pub sides: Option<Sides>,
    pub touch: Option<TouchTargets>,
    /// Those a game's own apps list, after the field's: ORAS's sky's.
    pub categories: Vec<String>,
    /// Whether a slot on a map is a sign that says only the name of a door there, which the
    /// list leaves out with `[pa3ds.navigation] hide_door_signs`.
    pub door_sign: Option<DoorSign>,
    /// What the game calls the place a zone is in, which a leg of P's way into another says.
    pub place: Option<Place>,
    /// None is never.
    pub shift_drags: Option<Rc<dyn Fn() -> bool>>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            bindings: None,
            names: None,
            asker: None,
            navigation: None,
            speaker: None,
            leads_to: None,
            street: None,
            clock: None,
            routes: true,
            slack: 0,
            moves: None,
            bike: None,
            acro: None,
            sides: None,
            touch: None,
            categories: Vec::new(),
            door_sign: None,
            place: None,
            shift_drags: None,
        }
    }
}

/// What a thing is said as and called: the navigator's `label`, `name_of`, `destination`,
/// `called` and `_name`, which read nothing of the navigator but the names and the game's words.
/// Python hands its `Guide` the navigator's bound methods; here the two share this.
pub struct Naming {
    names: Rc<Names>,
    /// What the game calls the place an exit from one zone into another arrives in, or None
    /// where it arrives in the player's own place, as every door of a town does.
    leads_to: LeadsTo,
    /// The street an exit from a zone, by its source, comes out on in the player's own
    /// place, as Lumiose's zones all are one.
    street: Street,
}

impl Naming {
    /// What to call this: the name it has been given, or what it is when it has none.
    pub fn name_of(&self, item: &Interactable, zone: i64) -> Result<String> {
        Ok(match self.name(item, zone)? {
            Some(name) if !name.is_empty() => name,
            _ => kind_said(&item.kind),
        })
    }

    /// What this is said as: what it is called, and for an exit into another place, the
    /// place: "edge to Route 101", "path to Route 102", "the north gap to Route 101".
    ///
    /// The place is the game's own word, so a name the player gives an exit comes before it
    /// rather than instead of it (ADR-0023: nothing of ours replaces what the game says); a name
    /// that already says the place is left as it is.
    pub fn label(&self, item: &Interactable, zone: i64) -> Result<String> {
        let called = self.name_of(item, zone)?;
        Ok(leading(&called, self.destination(item, zone)?.as_deref()))
    }

    /// What the game calls the place this exit from `zone` leads to, or None: for anything that
    /// is not an exit, for an exit into the player's own place — a house's door, the mat out of
    /// it, the stairs between two floors, though not the edge onto another zone of the same
    /// outdoors, as Route 119's two are — and without the game's text.
    pub fn destination(&self, item: &Interactable, zone: i64) -> Result<Option<String>> {
        let Some(there) = item.zone else {
            return Ok(None);
        };
        let home = item.home.unwrap_or(zone);
        // The street first: Lumiose's zones are one place, and a warp between two of them on one
        // matrix would say that place where the street says more.
        if let Some(street) = (self.street)(home, item.source)?
            && !street.is_empty()
        {
            return Ok(Some(street));
        }
        (self.leads_to)(zone, there)
    }

    /// The name this has been given, by the player or the game, or None.
    pub fn name(&self, item: &Interactable, zone: i64) -> Result<Option<String>> {
        let Some(naming) = item.naming(zone) else {
            return Ok(None);
        };
        Ok(self.called(naming, item.own()?))
    }

    /// What a thing named by `naming` is called, or the one `own` names alone, ahead of it.
    pub fn called(&self, naming: Named, own: Option<Named>) -> Option<String> {
        if let Some((_, first, second)) = own
            && let Some(called) = self.names.place(first, second)
            && !called.is_empty()
        {
            return Some(called);
        }
        let (what, first, second) = naming;
        if what == SPRITE {
            self.names.sprite(first)
        } else {
            self.names.place(first, second)
        }
    }
}

/// Answers the navigation keys from the field reader's list.
pub struct Navigator {
    pub(crate) field: Rc<dyn FieldSource>,
    categories: Vec<String>,
    /// The bottom screen's touch targets, in a game where they are read.
    ///
    /// The gen 6 profile's `census` walks them too, where the Python's holds the same object as
    /// `self.touch`.
    pub(crate) touch: Option<TouchTargets>,
    /// Whether Shift alone drags in touch mode now: while one of Pokémon-Amie's screens runs
    /// (ADR-0009 as amended 2026-10-28).
    shift_drags: Rc<dyn Fn() -> bool>,
    /// How an item and the way to it are said (`config.toml`'s `[pa3ds.navigation]`).
    wording: Navigation,
    speech: Rc<Speech>,
    log: Log,
    bindings: Bindings,
    names: Rc<Names>,
    naming: Rc<Naming>,
    pub(crate) asker: Asker,
    speaker: Speaker,
    /// The player's side of the battle as last said, or the enemy's, or None out of one.
    sides: Sides,
    door_sign: DoorSign,
    /// What the open dialog is renaming, the one alone where its checkbox may say so, and
    /// what it was called, since the player may have walked on by the time they press OK.
    renaming: Option<(Named, Option<Named>, String)>,
    mode: &'static str,
    /// Whether what Shift and P finds no way to is left out of the list (`HIDE_UNREACHABLE`).
    hiding: bool,
    category: i64,
    /// The key of the item chosen, or None for the nearest.
    chosen: Option<Key>,
    /// The direction the chosen item was found in, so the same key steps on from it and
    /// another starts afresh.
    facing: Option<String>,
    /// The big boulder a way to push it was chosen for, by its key, and that way
    /// (`PATH_NEXT`).
    pushing: Option<(Key, &'static str)>,
    /// P, and guiding the player along the way it finds.
    pub guide: Guide,
}

impl Navigator {
    /// `categories` are those a game's own apps list, after the field's: ORAS's sky's.
    /// `door_sign` says whether a slot on a map is a sign that says only the name of a door
    /// there, which the list leaves out with `[pa3ds.navigation] hide_door_signs`. `place` is
    /// what the game calls the place a zone is in, which a leg of P's way into another says.
    pub fn new(field: Rc<dyn FieldSource>, speech: Rc<Speech>, log: Log, options: Options) -> Self {
        let Options {
            bindings,
            names,
            asker,
            navigation,
            speaker,
            leads_to,
            street,
            clock,
            routes,
            slack,
            moves,
            bike,
            acro,
            sides,
            touch,
            categories,
            door_sign,
            place,
            shift_drags,
        } = options;
        let wording = navigation.unwrap_or_default();
        let names = names.unwrap_or_else(|| Rc::new(Names::default()));
        let naming = Rc::new(Naming {
            names: names.clone(),
            leads_to: leads_to.unwrap_or_else(|| Rc::new(|_here, _there| Ok(None))),
            street: street.unwrap_or_else(|| Rc::new(|_zone, _slot| Ok(None))),
        });
        let tell = {
            let (speech, log) = (speech.clone(), log.clone());
            Rc::new(move |text: &str| {
                log(&format!("path: {text}"));
                speech.say(text, Category::Guidance);
            })
        };
        let guide = Guide::new(
            field.clone(),
            {
                let naming = naming.clone();
                Rc::new(move |item: &Interactable, zone: i64| naming.label(item, zone))
            },
            {
                let naming = naming.clone();
                Rc::new(move |item: &Interactable, zone: i64| naming.name_of(item, zone))
            },
            tell,
            log.clone(),
            wording.clone(),
            clock.unwrap_or_else(|| Rc::new(py::time::monotonic)),
            routes,
            slack,
            moves,
            bike,
            acro,
            place,
        );
        Navigator {
            field,
            categories: CATEGORIES
                .iter()
                .map(|category| category.to_string())
                .chain(categories)
                .collect(),
            touch,
            shift_drags: shift_drags.unwrap_or_else(|| Rc::new(|| false)),
            wording,
            speech,
            log,
            bindings: bindings.unwrap_or_else(|| DEFAULT_BINDINGS.clone()),
            names,
            naming,
            asker: asker.unwrap_or_default(),
            speaker: speaker.unwrap_or_else(|| Rc::new(|| None)),
            sides: sides.unwrap_or_else(|| Rc::new(|_player| Ok(None))),
            door_sign: door_sign.unwrap_or_else(|| Rc::new(|_zone, _slot| false)),
            renaming: None,
            mode: LIST,
            hiding: false,
            category: 0,
            chosen: None,
            facing: None,
            pushing: None,
            guide,
        }
    }

    pub fn mode(&self) -> &str {
        self.mode
    }

    /// Whether a path is being walked, and the player's every step wants following, or a way
    /// waits at the belts' switch for it to be pressed (`Guide.waiting`).
    pub fn guiding(&self) -> bool {
        self.guide.guidance.is_some() || self.guide.waiting()
    }

    /// Answer a key by what it means in the current mode; a key unbound there is quiet.
    ///
    /// While touch mode holds a press, Shift is held all the while (`SHIFT_UP` lets it go), so
    /// a key with Shift means what it means without: Shift and J walks back as J does, where
    /// Shift and J would rename the speaker (ADR-0009).
    pub fn press_key(&mut self, key: &str) -> Result<()> {
        let holding = self.touch.as_ref().is_some_and(TouchTargets::holding);
        let toggle = self.wording.drag == DRAG_TOGGLE;
        if key == SHIFT_DOWN || key == SHIFT_UP {
            return self.shift(key == SHIFT_DOWN, holding, toggle);
        }
        let prefix = format!("{SHIFT}+");
        let key = if holding {
            key.strip_prefix(&prefix).unwrap_or(key)
        } else {
            key
        };
        let action = self
            .bindings
            .get(self.mode)
            .and_then(|keys| keys.get(&key.to_lowercase()))
            .cloned();
        if holding
            && toggle
            && action
                .as_deref()
                .is_some_and(|action| action == DRAG || action == TAP)
            && let Some(touch) = &mut self.touch
        {
            // Tapped to pick up, tapped again to let go (`[pa3ds.navigation] drag`).
            let dropped = touch.drop();
            self.say(&dropped);
        } else if let Some(action) = action {
            self.press(&action)?;
        }
        Ok(())
    }

    /// Shift alone. In touch mode while one of Pokémon-Amie's screens runs it drags: held, it
    /// picks up the target said and lets go as it comes up; or, with `[pa3ds.navigation] drag`
    /// "toggle", one press picks up and the next lets go. Elsewhere it lets go of a drag Shift
    /// and U picked up, as it comes up (ADR-0009 as amended 2026-10-28).
    fn shift(&mut self, down: bool, holding: bool, toggle: bool) -> Result<()> {
        if self.touch.is_none() {
            return Ok(());
        }
        let alone = self.mode == TOUCH && (self.shift_drags)();
        let touch = self.touch.as_mut().expect("checked above");
        if down && alone && !holding {
            let said = touch.press(DRAG)?;
            self.say(&said);
        } else if holding && ((down && alone && toggle) || (!down && !toggle)) {
            let dropped = touch.drop();
            self.say(&dropped);
        }
        Ok(())
    }

    pub fn press(&mut self, action: &str) -> Result<()> {
        // Naming a speaker asks the field nothing: they may have walked off, or the conversation
        // may be over, and the player still knows who they were talking to.
        if action == RENAME_SPEAKER {
            return self.rename_speaker();
        }
        // Off the map too: a way kept from before a menu or a warp is still the player's to drop.
        if action == PATHFIND_OFF {
            self.say(&own(
                if self.guiding() {
                    "key-path-off"
                } else {
                    "key-no-path"
                },
                &[],
            ));
            self.guide.stop();
            return Ok(());
        }
        // A battle's sides are the battle's, whatever the field says.
        if action == OUR_SIDE || action == ENEMY_SIDE {
            let text = match (self.sides)(action == OUR_SIDE)? {
                Some(text) if !text.is_empty() => text,
                _ => own("key-no-battle", &[]),
            };
            self.say(&text);
            return Ok(());
        }
        if self.mode == TOUCH && TOUCH_ACTIONS.contains(&action) {
            let said = match &mut self.touch {
                Some(touch) => touch.press(action)?,
                None => String::new(),
            };
            if !said.is_empty() {
                self.say(&said);
            }
            return Ok(());
        }
        if action == MODE_TOGGLE {
            if let Some(touch) = &mut self.touch {
                let dropped = touch.drop();
                if !dropped.is_empty() {
                    self.say(&dropped);
                }
            }
            let at = MODES
                .iter()
                .position(|&mode| mode == self.mode)
                .ok_or_else(|| Error::value("tuple.index(x): x not in tuple"))?;
            self.mode = MODES[(at + 1) % MODES.len()];
            self.chosen = None;
            self.facing = None;
            if self.mode == TOUCH {
                let first = match &mut self.touch {
                    Some(touch) => touch.start()?,
                    None => own(TOUCH_NOTHING, &[]),
                };
                self.say(&own("key-touch-mode", &[("first", &first)]));
            } else {
                self.say(&own(&format!("key-{}-mode", self.mode), &[]));
            }
            return Ok(());
        }
        if action == HIDE_UNREACHABLE {
            self.hiding = !self.hiding;
            self.say(&own(
                if self.hiding {
                    "key-unreachable-hidden"
                } else {
                    "key-unreachable-shown"
                },
                &[],
            ));
            return Ok(());
        }
        let Some(r#where) = self.field.poll()? else {
            self.say(&own("key-no-map", &[]));
            return Ok(());
        };
        if action == COORDINATES {
            let said = coordinates(&[r#where.cell], r#where.height, self.wording.show_y)?;
            self.say(&said);
            return Ok(());
        }
        let r#where = self.field.interactables(&r#where)?;
        if action == ITEM_COORDINATES {
            let item = self.chosen_item(&r#where)?;
            let said = match item {
                None => own(NOTHING_HERE, &[]),
                Some(item) => coordinates(&item.cells(), item.height, self.wording.show_y)?,
            };
            self.say(&said);
            return Ok(());
        }
        if action == RENAME {
            return self.rename(&r#where);
        }
        if action == PATHFIND || action == PATHFIND_HM {
            if !r#where.walkable {
                self.say(&own("key-cant-walk-now", &[]));
                return Ok(());
            }
            let Some(item) = self.chosen_item(&r#where)? else {
                self.say(&own(NOTHING_HERE, &[]));
                return Ok(());
            };
            if let Some(steer) = &r#where.steer {
                let said = format!(
                    "{}, {}",
                    self.label(&item, r#where.zone)?,
                    (steer.0)(&item)?
                );
                self.say(&said);
                return Ok(());
            }
            let held = self.guide.guidance.clone();
            let (speech, log) = (self.speech.clone(), self.log.clone());
            let say = move |text: &str| {
                log(&format!("key: {text}"));
                speech.say(text, Category::Key);
            };
            return self.guide.to(
                &r#where,
                &item,
                &say,
                "",
                action == PATHFIND_HM,
                held,
                false,
                true,
                self.push(&item),
            );
        }
        if action == PATH_PREVIOUS || action == PATH_NEXT {
            return self.choose_push(&r#where, if action == PATH_PREVIOUS { -1 } else { 1 });
        }
        if action == CATEGORY_PREVIOUS || action == CATEGORY_NEXT {
            let step = if action == CATEGORY_PREVIOUS { -1 } else { 1 };
            self.chosen = None;
            self.facing = None;
            let was = self.category;
            for _ in 0..self.categories.len() - 1 {
                self.category = py::modulo(self.category + step, self.categories.len() as i64);
                let items = self.items(&r#where, false)?;
                if !items.is_empty() {
                    self.chosen = Some(items[0].key());
                    self.say(&own(
                        "key-category",
                        &[
                            (
                                "category",
                                &category_said(&self.categories[self.category as usize]),
                            ),
                            ("count", &(items.len() as i64)),
                        ],
                    ));
                    return Ok(());
                }
            }
            // Nowhere to go: the category stays, so an empty map's "nothing north" does not
            // turn into "no items north" a press later.
            self.category = was;
            self.say(&own(NOTHING_HERE, &[]));
            return Ok(());
        }
        if DIRECTIONS.contains(&action) {
            return self.look(&r#where, action);
        }
        let items = self.items(&r#where, false)?;
        if items.is_empty() {
            self.say(&own(NOTHING_HERE, &[]));
            return Ok(());
        }
        let keys: Vec<Key> = items.iter().map(Interactable::key).collect();
        let mut at = match &self.chosen {
            Some(chosen) => keys.iter().position(|key| key == chosen).unwrap_or(0) as i64,
            None => 0,
        };
        if action == ITEM_PREVIOUS || action == ITEM_NEXT {
            let step = if action == ITEM_PREVIOUS { -1 } else { 1 };
            at = py::modulo(at + step, items.len() as i64);
        }
        self.chosen = Some(keys[at as usize].clone());
        self.said(&r#where, &items, at)
    }

    /// Say the next item on `direction`'s side of the player, nearest first.
    fn look(&mut self, r#where: &Field, direction: &str) -> Result<()> {
        let items: Vec<Interactable> = self
            .items(r#where, true)?
            .into_iter()
            .filter(|item| {
                towards(
                    direction,
                    r#where.cell,
                    item.nearest(r#where.cell),
                    r#where.reach.filter(|&reach| reach != 0).unwrap_or(RANGE),
                )
            })
            .collect();
        if items.is_empty() {
            self.chosen = None;
            self.facing = None;
            let category = self.categories[self.category as usize].as_str();
            // In the sky the list is in the player's own frame, north the way they face.
            let way = pathfind::way(if r#where.steer.is_some() {
                match RELATIVE.iter().find(|(key, _)| *key == direction) {
                    Some((_, word)) => word,
                    None => return Err(Error::key(py::repr(direction))),
                }
            } else {
                direction
            });
            self.say(&if category == ALL {
                own("key-nothing-way", &[("way", &way)])
            } else {
                own(
                    "key-none-way",
                    &[("category", &category_said(category)), ("way", &way)],
                )
            });
            return Ok(());
        }
        let keys: Vec<Key> = items.iter().map(Interactable::key).collect();
        let mut at = 0;
        let kept = self
            .chosen
            .as_ref()
            .and_then(|chosen| keys.iter().position(|key| key == chosen));
        if self.facing.as_deref() == Some(direction)
            && let Some(kept) = kept
        {
            at = py::modulo(kept as i64 + 1, items.len() as i64);
        }
        self.chosen = Some(keys[at as usize].clone());
        self.facing = Some(direction.to_string());
        self.said(r#where, &items, at)
    }

    /// Choose the next or previous way the chosen big boulder can be pushed, for P to go to
    /// its side: "push east, 2 of 2". Anything else has one way to it.
    fn choose_push(&mut self, r#where: &Field, step: i64) -> Result<()> {
        let Some(item) = self.chosen_item(r#where)? else {
            self.say(&own(NOTHING_HERE, &[]));
            return Ok(());
        };
        let Some(pushes) = &item.pushes else {
            self.say(&own("key-one-way", &[]));
            return Ok(());
        };
        if pushes.is_empty() {
            self.say(&own("boulder-immoveable", &[]));
            return Ok(());
        }
        let ways: Vec<&'static str> = pushes.iter().map(|&(way, _)| way).collect();
        let count = ways.len() as i64;
        let at = match self
            .push(&item)
            .and_then(|now| ways.iter().position(|&way| way == now))
        {
            Some(now) => py::modulo(now as i64 + step, count),
            None if step > 0 => 0,
            None => count - 1,
        };
        let (way, drops) = pushes[at as usize];
        self.chosen = Some(item.key());
        self.pushing = Some((item.key(), way));
        let mut text = own("key-push", &[("way", &pushed(way, drops))]);
        if self.wording.positions {
            text += &format!(
                ", {}",
                own("position", &[("number", &(at + 1)), ("count", &count)])
            );
        }
        self.say(&text);
        Ok(())
    }

    /// The way chosen for this big boulder to be pushed, or None.
    fn push(&self, item: &Interactable) -> Option<&'static str> {
        match &self.pushing {
            Some((key, way)) if *key == item.key() => Some(*way),
            _ => None,
        }
    }

    /// The category's items, nearest first unless `[pa3ds.navigation] sort_by_distance` is
    /// off, where they keep the zone's tables' order; `nearest` true sorts them regardless.
    fn items(&self, r#where: &Field, nearest: bool) -> Result<Vec<Interactable>> {
        let mut items = r#where.of_with(
            &self.categories[self.category as usize],
            nearest || self.wording.sort_by_distance,
        );
        if self.wording.hide_door_signs {
            // A person's source has a record's index where a sign's has its kind.
            items.retain(|item| {
                item.move_model || !(self.door_sign)(item.home.unwrap_or(r#where.zone), item.source)
            });
        }
        if !self.hiding {
            return Ok(items);
        }
        let reached = self.guide.reachable(r#where)?;
        Ok(items
            .into_iter()
            .filter(|item| reached.contains(&item.key()))
            .collect())
    }

    fn said(&self, r#where: &Field, items: &[Interactable], at: i64) -> Result<()> {
        let item = &items[at as usize];
        let mut way = item.bearing(r#where.cell, r#where.heading)?;
        if let Some(rise) = rise(r#where, item.height)? {
            way += &format!(", {rise}");
        }
        if let Some(arrival) = item.arrival {
            way += &format!(
                ", {}",
                own(
                    "out-at",
                    &[("bearing", &things::bearing(r#where.cell, arrival))]
                )
            );
        }
        if let Some(rise) = rise(r#where, item.landing)? {
            way += &format!(", {rise}");
        }
        let start = if self.wording.track_boulders {
            item.start
        } else {
            None
        };
        if let Some(start) = start
            && start != item.cell
        {
            way += &format!(
                ", {}",
                own(
                    "boulder-moved",
                    &[("bearing", &things::bearing(start, item.cell))]
                )
            );
        }
        if let Some(pushes) = &item.pushes {
            way += &format!(", {}", moveable(pushes));
        }
        let mut parts = vec![self.label(item, r#where.zone)?, way];
        if self.wording.positions {
            parts.insert(
                if self.wording.position_first { 1 } else { 2 },
                own(
                    "position",
                    &[("number", &(at + 1)), ("count", &(items.len() as i64))],
                ),
            );
        }
        self.say(&parts.join(", "));
        Ok(())
    }

    /// Keep up with the player along the way being walked (`Guide.follow`).
    pub fn follow(&mut self, r#where: &Field) -> Result<()> {
        self.guide.follow(r#where)
    }

    /// Say where a current takes the player as it catches them (`Guide.swept`).
    pub fn swept(&mut self, r#where: &Field) -> Result<()> {
        self.guide.swept(r#where)
    }

    /// What to call this: the name it has been given, or what it is when it has none.
    pub fn name_of(&self, item: &Interactable, zone: i64) -> Result<String> {
        self.naming.name_of(item, zone)
    }

    /// What this is said as: what it is called, and for an exit into another place, the
    /// place: "edge to Route 101", "path to Route 102", "the north gap to Route 101".
    ///
    /// The place is the game's own word, so a name the player gives an exit comes before it
    /// rather than instead of it (ADR-0023: nothing of ours replaces what the game says); a name
    /// that already says the place is left as it is.
    pub fn label(&self, item: &Interactable, zone: i64) -> Result<String> {
        self.naming.label(item, zone)
    }

    /// What the game calls the place this exit from `zone` leads to, or None: for anything that
    /// is not an exit, for an exit into the player's own place — a house's door, the mat out of
    /// it, the stairs between two floors, though not the edge onto another zone of the same
    /// outdoors, as Route 119's two are — and without the game's text.
    pub fn destination(&self, item: &Interactable, zone: i64) -> Result<Option<String>> {
        self.naming.destination(item, zone)
    }

    /// The name this has been given, by the player or the game, or None.
    pub fn called(&self, item: &Interactable, zone: i64) -> Result<Option<String>> {
        self.naming.name(item, zone)
    }

    fn name(&self, item: &Interactable, zone: i64) -> Result<Option<String>> {
        self.naming.name(item, zone)
    }

    /// The thing the navigation keys are on now, or None when they are on nothing.
    fn chosen_item(&self, r#where: &Field) -> Result<Option<Interactable>> {
        let mut items = self.items(r#where, false)?;
        if items.is_empty() {
            return Ok(None);
        }
        let at = match &self.chosen {
            Some(chosen) => items
                .iter()
                .position(|item| &item.key() == chosen)
                .unwrap_or(0),
            None => 0,
        };
        Ok(Some(items.swap_remove(at)))
    }

    /// A hidden Pokémon the DexNav has just found, `item` of the list `where` holds: by its
    /// search, the game asked for it by the player, or by its detector with the DexNav open,
    /// which is taken only while the player is in touch mode, at the DexNav, and not at an
    /// AreaNav they forgot to bring back (Orion, 2026-10-07); what the detector turns up with
    /// the AreaNav open never comes here (`gen6.profile`). With
    /// `[pa3ds.navigation] guide_to_hidden`, P is on it from now, in the category of all where
    /// the one chosen does not hold it, and the way to it is found as P finds it, said after what
    /// it leads to and queued, since no key asked for it: getting out of touch mode, where the
    /// search is made, and walking the list to it took Orion long enough to lose it (movie26).
    /// Python's `searched` defaults to True.
    pub fn found(&mut self, r#where: &Field, item: &Interactable, searched: bool) -> Result<()> {
        if !self.wording.guide_to_hidden || !(searched || self.mode == TOUCH) {
            return Ok(());
        }
        let key = item.key();
        if !self
            .items(r#where, false)?
            .iter()
            .any(|listed| listed.key() == key)
        {
            self.category = self
                .categories
                .iter()
                .position(|category| category == ALL)
                .ok_or_else(|| Error::value("tuple.index(x): x not in tuple"))?
                as i64;
        }
        self.chosen = Some(key);
        self.facing = None;
        let tell = self.guide.tell.clone();
        self.guide
            .to(r#where, item, &*tell, "", false, None, true, false, None)
    }

    /// Put the rename dialog up for whatever is chosen.
    ///
    /// Nothing chosen is nothing to rename, and no dialog opens: that is the common case for a
    /// mistyped key, and it must not leave a window in front of the game.
    fn rename(&mut self, r#where: &Field) -> Result<()> {
        let Some(item) = self.chosen_item(r#where)? else {
            self.say(&own("rename-nothing", &[]));
            return Ok(());
        };
        let Some(naming) = item.naming(r#where.zone) else {
            // A hidden Pokémon: a different one each time, and the game names the species itself.
            self.say(&own("rename-cannot", &[("thing", &kind_said(&item.kind))]));
            return Ok(());
        };
        if self.asker.open() {
            self.say(&own("rename-already", &[]));
            return Ok(());
        }
        let called = self.name(&item, r#where.zone)?;
        let alone = item.own()?;
        self.renaming = Some((naming, alone, self.label(&item, r#where.zone)?));
        let label = own(
            "rename-label",
            &[
                (
                    "thing",
                    &leading(
                        &kind_said(&item.kind),
                        self.destination(&item, r#where.zone)?.as_deref(),
                    ),
                ),
                ("bearing", &item.bearing(r#where.cell, r#where.heading)?),
                ("name", &named(called.as_deref())),
            ],
        );
        self.asker.ask(
            RENAMING,
            &own("rename-title", &[]),
            &label,
            called.as_deref().unwrap_or_default(),
            &if alone.is_some() {
                own(ONLY, &[])
            } else {
                String::new()
            },
        )?;
        Ok(())
    }

    /// Put the dialog up for whoever last spoke.
    fn rename_speaker(&mut self) -> Result<()> {
        let Some(speaker) = (self.speaker)() else {
            self.say(&own("rename-nobody-spoke", &[]));
            return Ok(());
        };
        if self.asker.open() {
            self.say(&own("rename-already", &[]));
            return Ok(());
        }
        let (character, alone) = speaker;
        let naming: Named = (SPRITE, character, 0);
        let called = self.naming.called(naming, alone);
        let called = called.filter(|called| !called.is_empty());
        self.renaming = Some((
            naming,
            alone,
            called
                .clone()
                .unwrap_or_else(|| own("rename-the-last-speaker", &[])),
        ));
        let label = own(
            "rename-speaker-label",
            &[("name", &named(called.as_deref()))],
        );
        self.asker.ask(
            SPEAKING,
            &own("rename-title", &[]),
            &label,
            called.as_deref().unwrap_or_default(),
            &if alone.is_some() {
                own(ONLY, &[])
            } else {
                String::new()
            },
        )?;
        Ok(())
    }

    /// Keep any name given since the last poll, and say it back.
    ///
    /// Said back because the dialog closing is not evidence to a blind player that anything was
    /// saved. An empty name clears theirs and gives the thing back to the shipped table, so what
    /// is said afterwards is read from the store rather than assumed.
    pub fn poll_renames(&mut self) -> Result<()> {
        for answer in self.asker.answers() {
            if (answer.what != RENAMING && answer.what != SPEAKING) || self.renaming.is_none() {
                continue;
            }
            let Some((naming, alone, was)) = self.renaming.take() else {
                continue;
            };
            let (what, first, second) = match alone {
                Some(alone) if answer.ticked => alone,
                _ => naming,
            };
            if what == SPRITE {
                self.names.name_sprite(first, &answer.said)?;
            } else {
                self.names.name_place(first, second, &answer.said)?;
            }
            let now = self.naming.called(naming, alone);
            match now {
                Some(now) if !now.is_empty() => {
                    self.say(&own("rename-now", &[("thing", &was), ("name", &now)]))
                }
                _ => self.say(&own("rename-no-name", &[("thing", &was)])),
            }
        }
        Ok(())
    }

    fn say(&self, text: &str) {
        (self.log)(&format!("key: {text}"));
        self.speech.say(text, Category::Key);
    }
}

/// The ways a big boulder can be pushed, as a reading says them: "moveable north into a hole
/// or east", or "immoveable" (Orion's words, 2026-10-11: what it can do, not what it is doing).
fn moveable(ways: &[(&str, bool)]) -> String {
    if ways.is_empty() {
        return own("boulder-immoveable", &[]);
    }
    let pushed: Vec<String> = ways
        .iter()
        .map(|&(way, drops)| pushed(way, drops))
        .collect();
    let ways = if pushed.len() == 1 {
        pushed[0].clone()
    } else {
        own(
            "either",
            &[
                ("these", &pushed[..pushed.len() - 1].join(", ")),
                ("last", &pushed[pushed.len() - 1]),
            ],
        )
    };
    own("boulder-moveable", &[("ways", &ways)])
}

/// A way a big boulder can be pushed, "north", or "north into a hole".
fn pushed(way: &str, drops: bool) -> String {
    if drops {
        own("boulder-into-hole", &[("way", &pathfind::way(way))])
    } else {
        pathfind::way(way)
    }
}

/// What the rename box says a thing is called now, or that nothing is.
fn named(called: Option<&str>) -> String {
    match called {
        Some(called) if !called.is_empty() => own("rename-called", &[("name", &called)]),
        _ => own("rename-not-named", &[]),
    }
}

/// How far above or below the player `height` is, "104 below", in the game's units, where
/// both heights are known and further apart than a thing is walked up to from (`Ground.open`):
/// an item ball on a path under a cliff, said "1 north" alone, left a player on the cliff top
/// against it and unable to pick it up (Orion, 2026-10-24). A thing's own height, and where a
/// teleporter puts the player (`Interactable.landing`).
fn rise(r#where: &Field, height: Option<f64>) -> Result<Option<String>> {
    let (Some(here), Some(height)) = (r#where.height, height) else {
        return Ok(None);
    };
    let rise = py::round(height - here)?;
    if rise.abs() as f64 <= 2.0 * CLIMB {
        return Ok(None);
    }
    Ok(Some(own(
        if rise > 0 { "above" } else { "below" },
        &[("count", &rise.abs())],
    )))
}

/// Where `cells` are, "x 94, z 171", an axis they span by its first and last, "x 80 to 119";
/// with `show_y`, how high too, last, "x 94, z 171, y 36", where it is known: the flat
/// axes, then the height, as length, width and height go (Orion, 2026-10-15).
fn coordinates(cells: &[Cell], height: Option<f64>, show_y: bool) -> Result<String> {
    fn span(values: &[i64]) -> String {
        let (low, high) = (values.iter().min().unwrap(), values.iter().max().unwrap());
        if low == high {
            format!("{low}")
        } else {
            own("span", &[("low", low), ("high", high)])
        }
    }

    let mut parts = vec![
        format!(
            "x {}",
            span(&cells.iter().map(|&(x, _)| x).collect::<Vec<_>>())
        ),
        format!(
            "z {}",
            span(&cells.iter().map(|&(_, z)| z).collect::<Vec<_>>())
        ),
    ];
    if let (true, Some(height)) = (show_y, height) {
        parts.push(format!("y {}", py::round(height)?));
    }
    Ok(parts.join(", "))
}

/// `called` with the place it leads to after it, unless it already says it.
fn leading(called: &str, place: Option<&str>) -> String {
    match place {
        Some(place) if !called.contains(place) => {
            own("leads-to", &[("thing", &called), ("place", &place)])
        }
        _ => called.to_string(),
    }
}
