//! pa3ds's own footsteps: a footfall at the game's frames, each at full volume, the clip the
//! table has for the ground (spike 227).

use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use crate::core::bcsar::Sound;
use crate::core::clips::{mix, stereo};
use crate::core::loudness::azahar_volume;
use crate::games::common::field::chunk::footstep;
use crate::games::gen6::field::footsteps::{self, Footsteps, Steps, came_down, load};
use crate::games::gen6::field::mmodel;
use crate::games::xy;
use crate::py::Log;

const MODEL: i64 = 0x08D209B8;
const ANIMATION: i64 = 0x08299EB4;
const STREET: i64 = 0x00000020;
const GRASS: i64 = 0x20004004;
const CAVE: i64 = 0x24000024;
const WALKING: i64 = 0x04;
const STANDING: i64 = 0x02;
const RUNNING: i64 = 0x06;

fn silent() -> HashSet<String> {
    ["walk silent", "run silent"]
        .into_iter()
        .map(String::from)
        .collect()
}

/// Spike 225's cells: Petalburg's street, Route 113's grass, the beach's sand.
#[test]
fn test_the_ground_is_read_as_the_game_reads_it() {
    assert_eq!(footstep(STREET), 0);
    assert_eq!(footstep(GRASS), 4);
    assert_eq!(footstep(0x025A8020), 8);
    assert_eq!(footstep(0xFFFFFFFF), 0);
    assert_eq!(footstep(0xFF00A000), 0);
}

#[test]
fn test_a_foot_comes_down_once_as_the_frame_passes_it() {
    let falls = footsteps::falls(footsteps::WALK);
    assert!(came_down(falls, 5.0, 6.0));
    // the same frame read on two ticks
    assert!(!came_down(falls, 6.0, 6.0));
    assert!(!came_down(falls, 6.0, 12.0));
    // 13 passed round the end of the cycle
    assert!(came_down(falls, 12.0, 0.0));
    assert!(!came_down(falls, 13.0, 1.0));
}

struct Walk {
    memory: Rc<FakeMemory>,
    layout: Steps,
    played: Rc<RefCell<Vec<String>>>,
    steps: Footsteps,
}

impl Walk {
    fn new(grounds: HashSet<String>) -> Walk {
        Walk::with(grounds, None)
    }

    /// `Walk(grounds, layout)`: where a game other than ORAS keeps a step.
    fn laid(grounds: HashSet<String>, layout: Steps) -> Walk {
        Walk::built(grounds, None, layout)
    }

    /// `Walk(grounds)`, or with `swim` the `Swim(grounds, every)` built on it: a zone `BELOW`
    /// under the sea.
    fn with(grounds: HashSet<String>, swim: Option<IndexMap<String, f64>>) -> Walk {
        Walk::built(grounds, swim, footsteps::ORAS)
    }

    fn built(grounds: HashSet<String>, swim: Option<IndexMap<String, f64>>, layout: Steps) -> Walk {
        let memory = FakeMemory::new();
        memory.u32(MODEL + mmodel::IN_USE, &[1]);
        memory.u32(MODEL + mmodel::ZONE, &[0x00170000 | mmodel::PLAYER_INDEX]);
        memory.u32(MODEL + layout.animation, &[ANIMATION]);
        let played: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let play: footsteps::Play = {
            let played = played.clone();
            Rc::new(move |name: &str| {
                played.borrow_mut().push(name.to_string());
                Ok(())
            })
        };
        let steps = match swim {
            None => Footsteps::with_steps(
                memory.clone(),
                play,
                grounds,
                &IndexMap::new(),
                Rc::new(|_zone: i64| Ok(false)),
                layout,
                if layout != footsteps::ORAS {
                    None
                } else {
                    Some(mmodel::FORM)
                },
            ),
            Some(every) => Footsteps::with(
                memory.clone(),
                play,
                grounds,
                &every,
                Rc::new(|zone: i64| Ok(zone == BELOW)),
            ),
        };
        Walk {
            memory,
            layout,
            played,
            steps,
        }
    }

    fn played(&self) -> Vec<String> {
        self.played.borrow().clone()
    }

    fn frame(&mut self, animation: i64, frame: f64, ground: i64) {
        self.memory.u32(MODEL + footsteps::GROUND, &[ground]);
        self.memory
            .u16(ANIMATION + self.layout.animation_id, animation);
        self.memory
            .write(ANIMATION + self.layout.frame, &(frame as f32).to_le_bytes());
        self.steps.poll(MODEL).unwrap();
    }

    fn cycle(&mut self, animation: i64, ground: i64) {
        for frame in 0..14 {
            self.frame(animation, frame as f64, ground);
        }
    }

    /// `Swim.at`: the player on a Pokémon, carried along x: Route 124's sea and the sea under it.
    fn at(&mut self, x: f64, ground: i64, zone: i64, form: u8) {
        self.memory
            .u32(MODEL + mmodel::ZONE, &[zone << 16 | mmodel::PLAYER_INDEX]);
        self.memory.write(MODEL + mmodel::RIDING, &[0x21, 0x03]);
        self.memory.write(MODEL + mmodel::FORM, &[form]);
        self.memory.write(
            MODEL + mmodel::WORLD_X,
            &((x * mmodel::CELL as f64 + 9.0) as f32).to_le_bytes(),
        );
        self.frame(if form == ON_BIKE { 0x09 } else { 0x29 }, 0.0, ground);
    }
}

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

fn times(name: &str, count: usize) -> Vec<String> {
    vec![name.to_string(); count]
}

#[test]
fn test_a_walk_plays_a_step_a_footfall_on_silent_ground_and_none_on_grass() {
    let mut walk = Walk::new(silent());
    for _ in 0..4 {
        walk.cycle(WALKING, STREET);
    }
    walk.cycle(WALKING, GRASS);
    assert_eq!(walk.played(), times("walk silent", 8));
}

#[test]
fn test_running_plays_the_run() {
    let mut walk = Walk::new(silent());
    walk.cycle(WALKING, STREET);
    walk.frame(STANDING, 0.0, STREET);
    for frame in 0..16 {
        walk.frame(RUNNING, frame as f64, STREET);
    }
    assert_eq!(
        walk.played(),
        names(&["walk silent", "walk silent", "run silent", "run silent"])
    );
}

/// Spike 231: X's animation at the model's +0x97c, its id at +0x4e and frame at +0x18; its
/// walk 0x5c and run 0x5d; its footstep bits 11 to 15, silent at 0, 1, 14 and 15. Its paths'
/// word is ORAS's street, 0x20; 0x6828, Santalune Gym's strand, has footstep 13, which plays.
#[test]
fn test_x_keeps_its_step_where_its_code_does_and_its_own_silent_ground() {
    let mut walk = Walk::laid(silent(), xy::STEPS);
    walk.cycle(0x5C, STREET);
    walk.cycle(WALKING, 0x00006828);
    // footstep 14
    walk.cycle(WALKING, 0x00007020);
    for frame in 0..16 {
        walk.frame(0x5D, frame as f64, STREET);
    }
    // The strand plays the game's own; the run's first tick passes no footfall, then 8 does.
    let mut played = times("walk silent", 4);
    played.push("run silent".to_string());
    assert_eq!(walk.played(), played);
}

#[test]
fn test_a_kind_of_its_own_wins_over_silent_ground_and_plays_over_the_game() {
    let mut grounds = silent();
    grounds.extend(["walk 0x24".to_string(), "walk 0x20".to_string()]);
    let mut walk = Walk::new(grounds);
    walk.cycle(WALKING, CAVE);
    walk.cycle(WALKING, GRASS);
    walk.cycle(WALKING, STREET);
    assert_eq!(
        walk.played(),
        [
            times("walk 0x24", 2),
            times("walk 0x20", 2),
            times("walk silent", 2)
        ]
        .concat()
    );
}

/// As the player starts walking or turns, the new animation's first frame may be past a
/// footfall; the game counts only the frames it passes (movie2: the first step and a turn).
#[test]
fn test_a_walk_begun_mid_stride_plays_nothing_until_it_passes_a_footfall() {
    let mut walk = Walk::new(silent());
    walk.frame(STANDING, 0.0, STREET);
    walk.frame(WALKING, 8.0, STREET);
    walk.frame(WALKING, 13.0, STREET);
    walk.frame(0x6A, 9.0, STREET);
    walk.frame(0x6A, 10.0, STREET);
    assert_eq!(walk.played().len(), 1);
}

const SEA: i64 = 0x3D1A0006;
const SEAWEED: i64 = 0x42000006;
const ABOVE: i64 = 0x35;
const BELOW: i64 = 0x41;
const ON_POKEMON: u8 = 0;
const ON_BIKE: u8 = 2;

fn swim(grounds: &[&str], movement: &str, every: f64) -> Walk {
    Walk::with(
        grounds.iter().map(|ground| ground.to_string()).collect(),
        Some(IndexMap::from([(movement.to_string(), every)])),
    )
}

#[test]
fn test_surfing_plays_the_first_cell_carried_and_then_every_few() {
    let mut swim = swim(&["surf silent"], footsteps::SURF, 2.0);
    for tick in 0..10 {
        swim.at(tick as f64 * 0.5, SEA, ABOVE, ON_POKEMON);
    }
    // at 0.5, 2.5 and 4.5 cells
    assert_eq!(swim.played(), times("surf silent", 3));
}

#[test]
fn test_a_swimmer_still_or_warped_hears_nothing() {
    let mut swim = swim(&["surf silent"], footsteps::SURF, 2.0);
    for _ in 0..5 {
        swim.at(3.0, SEA, ABOVE, ON_POKEMON);
    }
    // dived: a new place, no swim
    swim.at(30.0, SEA, ABOVE, ON_POKEMON);
    assert!(swim.played().is_empty());
    swim.at(30.25, SEA, ABOVE, ON_POKEMON);
    assert_eq!(swim.played(), names(&["surf silent"]));
}

/// The bike sets the riding bit a Pokémon does, and goes as fast as Sharpedo (spike 227).
#[test]
fn test_the_bike_is_no_swim() {
    let mut swim = swim(&["surf silent"], footsteps::SURF, 2.0);
    for tick in 0..10 {
        swim.at(tick as f64 * 0.5, SEA, ABOVE, ON_BIKE);
    }
    assert!(swim.played().is_empty());
}

#[test]
fn test_diving_plays_its_own_and_the_seaweed_as_soon_as_it_is_entered() {
    let mut swim = swim(
        &["dive silent", "dive 0x42", "surf silent"],
        footsteps::DIVE,
        3.0,
    );
    for (x, ground) in [
        (0, SEA),
        (1, SEA),
        (2, SEA),
        (3, SEAWEED),
        (4, SEAWEED),
        (5, SEA),
    ] {
        swim.at(x as f64, ground, BELOW, ON_POKEMON);
    }
    assert_eq!(
        swim.played(),
        names(&["dive silent", "dive 0x42", "dive silent"])
    );
}

#[test]
fn test_nothing_is_played_for_a_model_that_is_no_longer_the_player() {
    let mut walk = Walk::new(silent());
    walk.memory.u32(MODEL + mmodel::ZONE, &[0x00170003]);
    walk.cycle(WALKING, STREET);
    assert!(walk.played().is_empty());
}

#[test]
fn test_sounds_mix_into_one_at_the_first_ones_rate() {
    let step = Sound {
        rate: 4,
        samples: vec![1000, 1000, 1000, 1000],
        volume: 0.5,
        quietest: 0.7,
    };
    let splash = Sound {
        rate: 2,
        samples: vec![2000, 4000],
        volume: 1.0,
        quietest: 1.0,
    };
    let mixed = mix(&[(step, 1.0), (splash, 0.5)]).unwrap();
    assert_eq!(mixed.rate, 4);
    assert_eq!(mixed.samples, vec![1500, 2000, 2500, 500]);
    assert!(mixed.volume == 1.0 && mixed.quietest == 0.7);
}

/// pytest's `tmp_path`: a folder of this test's own, gone when the test is.
struct TmpPath(std::path::PathBuf);

impl TmpPath {
    fn new() -> Self {
        static MADE: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "pa3ds-test-footsteps-{}-{}",
            std::process::id(),
            MADE.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path).unwrap();
        TmpPath(path)
    }
}

impl Drop for TmpPath {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `wave.open(path, "wb")` with 16-bit samples: a RIFF file of a `fmt ` and a `data` chunk.
fn wav(path: &Path, samples: &[i16], rate: u32, channels: u16) {
    let data: Vec<u8> = samples
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect();
    let mut out: Vec<u8> = b"RIFF".to_vec();
    out.extend((36 + data.len() as u32).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(channels.to_le_bytes());
    out.extend(rate.to_le_bytes());
    out.extend((rate * u32::from(channels) * 2).to_le_bytes());
    out.extend((channels * 2).to_le_bytes());
    out.extend(16u16.to_le_bytes());
    out.extend(b"data");
    out.extend((data.len() as u32).to_le_bytes());
    out.extend(data);
    fs::write(path, out).unwrap();
}

#[test]
fn test_the_table_mixes_wavs_of_our_own_and_leaves_out_what_it_cannot_play() {
    let tmp_path = TmpPath::new();
    let sounds = tmp_path.0.join("sounds");
    fs::create_dir(&sounds).unwrap();
    wav(&sounds.join("weed.wav"), &[100, 200, 300, 400], 32728, 2);
    wav(&sounds.join("water.wav"), &[1000, 1000], 32728, 1);
    let table = "[walk]\n\"0x42\" = [\"weed.wav\", { sound = \"water.wav\", volume = 0.5 }]\n\
                 silent = [\"SEQ_SE_FT_03\"]\n[run]\n\"cave\" = [\"x.wav\"]\n[dive]\nevery = 3\n\
                 [surf]\nevery = \"far\"\n";
    let said: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let log: Log = {
        let said = said.clone();
        Rc::new(move |line: &str| said.borrow_mut().push(line.to_string()))
    };
    let (found, every) = load(
        footsteps::FILE,
        table,
        None,
        "sound/sango_sound.bcsar",
        &sounds,
        &log,
    )
    .unwrap();
    let said = said.borrow();
    assert_eq!(
        found.keys().cloned().collect::<HashSet<String>>(),
        HashSet::from(["walk 0x42".to_string()])
    );
    assert_eq!(every, IndexMap::from([(footsteps::DIVE.to_string(), 3.0)]));
    assert!(
        said.iter().any(|line| line.contains("surf every = 'far'")),
        "{said:?}"
    );
    assert_eq!(found["walk 0x42"].samples, vec![650, 850]);
    assert!(
        said.iter()
            .any(|line| line.contains("walk silent left out"))
    );
    assert!(
        said.iter()
            .any(|line| line.contains("'cave' is not a ground"))
    );
}

#[test]
fn test_azahar_volume_is_read_from_its_audio_settings() {
    assert_eq!(
        azahar_volume("[Audio]\nvolume\\default=false\nvolume=0.4\n"),
        0.4
    );
    assert_eq!(azahar_volume("[Audio]\n"), 1.0);
    assert_eq!(azahar_volume("volume=2\n"), 1.0);
}

/// ADR-0025 as amended 2026-11-01: a footstep sounds in both ears at full, as the mono sound
/// the game plays; Head It's cue the left ear alone for the left Pokémon, the right for the
/// right.
#[test]
fn test_a_clip_plays_in_both_ears_as_the_game_does_unless_asked_toward_one() {
    let samples = [1000i16, -2000];
    assert_eq!(
        stereo(&samples, 0.5, 0.5).unwrap(),
        [500, 500, -1000, -1000]
    );
    assert_eq!(stereo(&samples, 1.0, 0.0).unwrap(), [1000, 0, -2000, 0]);
    assert_eq!(stereo(&samples, 1.0, 1.0).unwrap(), [0, 1000, 0, -2000]);
}
