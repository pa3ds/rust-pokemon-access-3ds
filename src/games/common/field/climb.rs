//! Climbing walls: X's Cyllage Gym's rock faces, climbed from the blue spots at their foot, along
//! the row of cells in front of each, and off onto a ledge at a face's side; and the slides down
//! them, which carry the player (spike 245). A traversal (`traversal`): its state is what the
//! player on a face's line may be doing, `FOOT`, `WALL` or `EITHER`.
//!
//! Bound by:
//! - ADR-0006: nothing here names a game's kinds; `gen6.game.Climbing` carries them.
//! - ADR-0024: a climb is said as legs, each covering cells.

use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::pathfind::{self, Cell, State};
use crate::games::common::field::chunk::{BLOCKED, CLIMB, KIND_SHIFT};
use crate::games::common::field::ground::Ground;
use crate::games::common::field::traversal::Traversal;
use crate::py::{self, Result};

/// A handhold, in world units: a cell's height. Up the wall from (25, 32) at 3 the player rose to
/// 13, 38 and 57 for the ledge at 54, three holds (spike 245).
pub const HOLD: f64 = 18.0;

/// What a player on a face's line may be doing, a search's state there: walking its floor, on the
/// wall, or either. Off every line it is `FOOT`. Bits, so `state & WALL` asks one.
pub const FOOT: i64 = 1;
pub const WALL: i64 = 2;
pub const EITHER: i64 = FOOT | WALL;

/// One stretch of a way climbed, by index along the path: got on at `path[on]`, off at
/// `path[off]`, every step between on the wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Climb {
    pub on: i64,
    pub off: i64,
    /// Handholds climbed in place where it gets on, up positive, down negative. A climb that ends
    /// stepping sideways onto a ledge must be at the ledge's height before it, so its whole change
    /// is here. Python's default is 0.
    pub first: i64,
    /// Handholds climbed down in place where it gets off at a blue spot on the floor. Python's
    /// default is 0.
    pub last: i64,
}

/// The matrix's climbing walls, as the search and the list ask them: none on most maps
/// (`Walls::default()`, Python's `NO_WALLS`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Walls {
    /// Each cell of a face's line, with the face's number: the open cells in front of the face,
    /// from its outermost blue spot to its other, which the climber stands in (`mmodel.cell` gave
    /// (25, 32) to (31, 32) all the way along).
    pub lines: IndexMap<Cell, i64>,
    /// Each face's foot, by its number: the height of its line's floor.
    pub feet: Vec<f64>,
    /// Each blue spot, with its height: the face's foot, or a ledge's at the face's side.
    pub pads: IndexMap<Cell, f64>,
    /// Each ledge's blue spot, with the cell of the line beside it that the climber steps off the
    /// face from onto it, sideways, and back on: a cliff on foot.
    pub ledges: IndexMap<Cell, Cell>,
    /// Each cell of a slide, its top and every blocked cell it carries the player through, with the
    /// way it carries them, a cell at a time, to the open cell past it: a current (`Ground.carry`).
    /// In x_gym_2_slide the player stepped south onto (16, 20) at 188 and was at (16, 25) at 111
    /// 0.6 s later, about twice walking speed.
    pub slides: IndexMap<Cell, &'static str>,
}

/// `pathfind.STEPS[way]`, for a way the game's kinds name.
fn step_of(way: &str) -> Cell {
    pathfind::STEPS
        .iter()
        .find(|(known, _)| *known == way)
        .map(|&(_, step)| step)
        .expect("a way of the compass")
}

impl Walls {
    /// The walls in `cells`, each cell's word and ground height: a blue spot is an open cell
    /// beside a `face` cell at its height, the face standing the way from the spot to it; spots
    /// facing one way along one row or column are one face, its line the open cells between its
    /// outermost, broken where a cell is shut or has no wall behind it, and a line of one cell
    /// is none. A cell of a `sideways` kind on a line, beside a spot of the line that way above
    /// its foot, is the step off onto that ledge. In Cyllage Gym the foot is the lowest of a
    /// face's spots: every ledge stands 36 to 144 over it (spike 245). A slide is an open cell
    /// of the `slide` kind from which, one way, the cells are blocked, each lower than the one
    /// before, for one or more, and then open lower than it: the Gym's two fall 18 a cell, 171
    /// to 117 onto the floor at 108 from a top at 189, and 81 to 9 onto 0 from 99.
    pub fn found(
        cells: &IndexMap<Cell, (i64, Option<f64>)>,
        face: i64,
        sideways: &IndexMap<i64, &'static str>,
        slide: i64,
    ) -> Walls {
        let shut = |cell: Cell| -> bool {
            match cells.get(&cell) {
                None => true,
                Some(held) => held.0 & BLOCKED != 0,
            }
        };

        let lower = |cell: Cell, than: Cell| -> bool {
            let here = cells.get(&cell).and_then(|held| held.1);
            let there = cells[&than].1;
            match (here, there) {
                (Some(here), Some(there)) => here < there,
                _ => false,
            }
        };

        let mut facing: IndexMap<(&'static str, i64), IndexMap<Cell, f64>> = IndexMap::new();
        for (&(x, z), &(word, height)) in cells {
            let Some(height) = height else {
                continue;
            };
            if word >> KIND_SHIFT != face {
                continue;
            }
            for (way, (dx, dz)) in pathfind::STEPS {
                let spot = (x - dx, z - dz);
                if !shut(spot) && cells[&spot].1 == Some(height) {
                    let along = if way == pathfind::NORTH || way == pathfind::SOUTH {
                        spot.1
                    } else {
                        spot.0
                    };
                    facing.entry((way, along)).or_default().insert(spot, height);
                }
            }
        }
        let mut lines: IndexMap<Cell, i64> = IndexMap::new();
        let mut feet: Vec<f64> = Vec::new();
        let mut pads: IndexMap<Cell, f64> = IndexMap::new();
        let mut ledges: IndexMap<Cell, Cell> = IndexMap::new();
        let mut sorted: Vec<(&(&'static str, i64), &IndexMap<Cell, f64>)> = facing.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));
        for (&(way, _), spots) in sorted {
            let (dx, dz) = step_of(way);
            let first = *spots.keys().min().expect("a face has a spot");
            let last = *spots.keys().max().expect("a face has a spot");
            let mut run: Vec<Cell> = Vec::new();
            for k in 0..last.0 - first.0 + last.1 - first.1 + 2 {
                let cell = (first.0 + k * dz.abs(), first.1 + k * dx.abs());
                if cell <= last && !shut(cell) && shut((cell.0 + dx, cell.1 + dz)) {
                    run.push(cell);
                    continue;
                }
                let held: Vec<usize> = run
                    .iter()
                    .enumerate()
                    .filter(|(_, one)| spots.contains_key(*one))
                    .map(|(i, _)| i)
                    .collect();
                let line: Vec<Cell> = match (held.first(), held.last()) {
                    (Some(&lo), Some(&hi)) => run[lo..hi + 1].to_vec(),
                    _ => Vec::new(),
                };
                run = Vec::new();
                if line.len() < 2 {
                    continue;
                }
                // Python's `min`: a later height replaces the first only by being less.
                let mut foot: Option<f64> = None;
                for one in &line {
                    if let Some(&height) = spots.get(one)
                        && foot.is_none_or(|foot| height < foot)
                    {
                        foot = Some(height);
                    }
                }
                let foot = foot.expect("a line holds a spot");
                for &one in &line {
                    lines.insert(one, feet.len() as i64);
                    if let Some(&height) = spots.get(&one) {
                        pads.insert(one, height);
                    }
                    if let Some(&off) = sideways.get(&(cells[&one].0 >> KIND_SHIFT)) {
                        let (ox, oz) = step_of(off);
                        let ahead = (one.0 + ox, one.1 + oz);
                        if line.contains(&ahead)
                            && spots.get(&ahead).copied().unwrap_or(foot) > foot
                        {
                            ledges.insert(ahead, one);
                        }
                    }
                }
                feet.push(foot);
            }
        }
        let mut slides: IndexMap<Cell, &'static str> = IndexMap::new();
        for (&top, &(word, _)) in cells {
            if word >> KIND_SHIFT != slide || shut(top) {
                continue;
            }
            for (way, (dx, dz)) in pathfind::STEPS {
                let mut run: Vec<Cell> = vec![top];
                let mut ahead;
                loop {
                    let last = run[run.len() - 1];
                    ahead = (last.0 + dx, last.1 + dz);
                    if !(shut(ahead) && lower(ahead, last)) {
                        break;
                    }
                    run.push(ahead);
                }
                if run.len() > 1 && !shut(ahead) && lower(ahead, top) {
                    for cell in run {
                        slides.insert(cell, way);
                    }
                }
            }
        }
        Walls {
            lines,
            feet,
            pads,
            ledges,
            slides,
        }
    }

    /// Whether the step is off a face onto a ledge, or back on: only climbing.
    pub fn sideways(&self, cell: Cell, step: Cell) -> bool {
        self.ledges.get(&step) == Some(&cell) || self.ledges.get(&cell) == Some(&step)
    }

    /// What a step off a face onto its ledge, or back on, costs on top of the step: a step's
    /// worth a handhold climbed in place to the ledge's height, each a press of a player who
    /// counts them (ADR-0024); nothing for any other.
    pub fn toll(&self, cell: Cell, step: Cell) -> Result<i64> {
        if !self.sideways(cell, step) {
            return Ok(0);
        }
        let (ledge, beside) = if self.ledges.contains_key(&step) {
            (step, cell)
        } else {
            (cell, step)
        };
        let rise = self.pads[&ledge] - self.feet[self.lines[&beside] as usize];
        Ok(pathfind::STEP * py::round(rise.abs() / HOLD)?)
    }

    pub fn on(ground: &Ground) -> Option<Rc<dyn Traversal>> {
        if ground.walls.lines.is_empty() {
            None
        } else {
            Some(ground.walls.clone() as Rc<dyn Traversal>)
        }
    }

    /// What a player standing in `cell` at `level` may be doing: either at a blue spot, on
    /// the wall on a line above its foot, on foot anywhere else.
    pub fn doing(&self, cell: Cell, level: Option<f64>) -> i64 {
        if self.pads.contains_key(&cell) {
            return EITHER;
        }
        let face = self.lines.get(&cell);
        if let (Some(&face), Some(level)) = (face, level)
            && level - self.feet[face as usize] > CLIMB
        {
            return WALL;
        }
        FOOT
    }

    /// `state` after the step, or None where it cannot be taken: the search's `through`.
    /// Onto a face's line at a blue spot either is open, and anywhere else only on foot; along
    /// it the state goes on; off it sideways onto a ledge only on the wall, and any other way
    /// only on foot.
    pub fn hold(&self, state: i64, cell: Cell, step: Cell) -> Option<i64> {
        // At a blue spot either is open: a walker climbs up there, a climber climbs down. Taken
        // on the cell left, so a jump that skips `through` (`pathfind._onward`) loses nothing.
        let here = if self.pads.contains_key(&cell) {
            EITHER
        } else {
            state
        };
        let landed = if self.pads.contains_key(&step) {
            Some(EITHER)
        } else {
            None
        };
        if self.sideways(cell, step) {
            return if here & WALL == 0 {
                None
            } else {
                Some(landed.unwrap_or(WALL))
            };
        }
        let face = self.lines.get(&cell);
        if face.is_some() && self.lines.get(&step) == face {
            return Some(landed.unwrap_or(here));
        }
        if face.is_some() && here & FOOT == 0 {
            return None;
        }
        Some(landed.unwrap_or(FOOT))
    }

    /// Where `path` climbs, from a player at `level` on its first cell: each run of it along
    /// one line with a step sideways in it, from the run's first blue spot to its last, and a
    /// first run the player starts on the wall, from where they are. Any other run is walked,
    /// and so is the rest of one along a ledge it steps off onto: Cyllage Gym's (29, 27) and
    /// (29, 28) are one ledge, two spots of their face's line (spike 245). The search's state
    /// (`hold`) puts a spot before a step off sideways and after a step on.
    pub fn climbs(&self, path: &[Cell], level: Option<f64>) -> Result<Vec<Climb>> {
        let mut out: Vec<Climb> = Vec::new();
        let mut i = if self.lines.is_empty() { path.len() } else { 0 };
        while i < path.len() {
            let (face, mut j) = (self.lines.get(&path[i]), i);
            while face.is_some() && j + 1 < path.len() && self.lines.get(&path[j + 1]) == face {
                j += 1;
            }
            let side: Vec<usize> = (i..j)
                .filter(|&k| self.sideways(path[k], path[k + 1]))
                .collect();
            let walled = face.is_some() && i == 0 && self.doing(path[0], level) == WALL;
            if !side.is_empty() || walled {
                let spots: Vec<usize> = (i..j + 1)
                    .filter(|&k| self.pads.contains_key(&path[k]))
                    .collect();
                let on = if walled || spots.is_empty() {
                    i
                } else {
                    spots[0]
                };
                let onto = side
                    .last()
                    .is_some_and(|&last| self.ledges.get(&path[last + 1]) == Some(&path[last]));
                let off = if onto {
                    side[side.len() - 1] + 1
                } else {
                    match spots.last() {
                        Some(&last) if last > side.last().copied().unwrap_or(on) => last,
                        _ => j,
                    }
                };
                let was = match level {
                    Some(level) if on == 0 => Some(level),
                    _ => self.pads.get(&path[on]).copied(),
                };
                let now = self.pads.get(&path[off]).copied();
                let change = match (was, now) {
                    (Some(was), Some(now)) => py::round((now - was) / HOLD)?,
                    _ => 0,
                };
                out.push(Climb {
                    on: on as i64,
                    off: off as i64,
                    first: if onto { change } else { 0 },
                    last: if onto { 0 } else { (-change).max(0) },
                });
            }
            i = j + 1;
        }
        Ok(out)
    }

    /// The blue spots of each face at each height, and whether they stand on a ledge, above
    /// the face's foot: what the list shows, a run of them at a time.
    pub fn spots(&self) -> Vec<(Vec<Cell>, bool)> {
        // Python's dict keyed by (face, height): looked up by `==`, so a NaN height, which no
        // ROM's heights hold, makes a key of its own each time.
        let mut faces: Vec<((i64, f64), Vec<Cell>)> = Vec::new();
        for (&cell, &height) in &self.pads {
            let key = (self.lines[&cell], height);
            match faces.iter_mut().find(|(known, _)| *known == key) {
                Some((_, cells)) => cells.push(cell),
                None => faces.push((key, vec![cell])),
            }
        }
        faces
            .into_iter()
            .map(|((face, height), cells)| (cells, height > self.feet[face as usize]))
            .collect()
    }
}

impl Traversal for Walls {
    fn start(&self, ground: &Ground, cell: Cell, _floor: i64) -> Result<State> {
        Ok(vec![self.doing(cell, ground.level)])
    }

    fn step(
        &self,
        _ground: &Ground,
        state: &[i64],
        cell: Cell,
        step: Cell,
        _floor: i64,
    ) -> Result<Option<State>> {
        Ok(self.hold(state[0], cell, step).map(|held| vec![held]))
    }
}
