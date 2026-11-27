//! The game's own feedback sounds, or the words in their place (`gen6.feedback`): what the profile
//! wired by hand before the untangling round's step 9 put it in one object.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use super::fake_speech::Recorder;
use crate::core::config::{CUE_AS_SOUND, CUE_AS_SPEECH, CUE_OFF};
use crate::core::speech::Speech;
use crate::games::gen6::feedback::{
    ALERT_SOUND, CUE_SOUND, DROP_SOUND, FeedbackSounds, PICK_SOUND, Plays, WRONG_SOUND,
};
use crate::py::{Log, Result};

/// `SimpleNamespace(names=frozenset(loaded), play=play)`: clips that hold `loaded` and keep what
/// they played.
struct Fake {
    names: HashSet<String>,
    played: Rc<RefCell<Vec<(String, f64)>>>,
}

impl Plays for Fake {
    fn names(&self) -> HashSet<String> {
        self.names.clone()
    }

    fn play(&self, name: &str, across: f64) -> Result<()> {
        self.played.borrow_mut().push((name.to_string(), across));
        Ok(())
    }
}

/// What a test's sounds played, logged and said.
type Heard = (
    Rc<RefCell<Vec<(String, f64)>>>,
    Rc<RefCell<Vec<String>>>,
    Rc<Recorder>,
);

/// Feedback sounds with `loaded` among their clips, and what they played, logged and said.
fn sounds(loaded: &[&str]) -> (Rc<FeedbackSounds>, Heard) {
    let played = Rc::new(RefCell::new(Vec::new()));
    let logged: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let backend = Recorder::new();
    let clips = Fake {
        names: loaded.iter().map(|name| name.to_string()).collect(),
        played: played.clone(),
    };
    let log: Log = {
        let logged = logged.clone();
        Rc::new(move |line: &str| logged.borrow_mut().push(line.to_string()))
    };
    let found = FeedbackSounds::new(
        log,
        Rc::new(Speech::plain(backend.clone())),
        Some(Rc::new(clips) as Rc<dyn Plays>),
        None,
    );
    (Rc::new(found), (played, logged, backend))
}

fn played(heard: &Heard) -> Vec<(String, f64)> {
    heard.0.borrow().clone()
}

fn logged(heard: &Heard) -> Vec<String> {
    heard.1.borrow().clone()
}

#[test]
fn test_a_keys_line_is_heard_as_its_sound_or_said_without_it() {
    let (found, heard) = sounds(&[WRONG_SOUND]);
    assert_eq!(found.heard(WRONG_SOUND, "nothing to drag"), "");
    assert_eq!(
        found.heard(PICK_SOUND, "holding Pinap Berry"),
        "holding Pinap Berry"
    );
    assert_eq!(played(&heard), [(WRONG_SOUND.to_string(), 0.5)]);
    assert_eq!(
        logged(&heard),
        [format!("key: nothing to drag, heard as {WRONG_SOUND}")]
    );
    let alone = FeedbackSounds::new(
        {
            let logged = heard.1.clone();
            Rc::new(move |line: &str| logged.borrow_mut().push(line.to_string()))
        },
        Rc::new(Speech::plain(Recorder::new())),
        None,
        None,
    );
    assert_eq!(alone.heard(WRONG_SOUND, "nothing to drag"), "nothing to drag");
    assert!(!alone.has(&[WRONG_SOUND]));
}

#[test]
fn test_a_rounds_cue_is_its_sound_its_words_or_nothing_as_asked() {
    let (found, heard) = sounds(&[ALERT_SOUND]);
    found.round_cue(CUE_AS_SOUND, ALERT_SOUND, "shot coming");
    found.round_cue(CUE_AS_SPEECH, ALERT_SOUND, "shot coming");
    found.round_cue(CUE_AS_SOUND, CUE_SOUND, "now");
    found.round_cue(CUE_OFF, ALERT_SOUND, "shot coming");
    assert_eq!(played(&heard), [(ALERT_SOUND.to_string(), 0.5)]);
    assert_eq!(
        logged(&heard),
        [
            format!("round: shot coming, heard as {ALERT_SOUND}"),
            "round: shot coming".to_string(),
            "round: now".to_string(),
        ]
    );
    assert_eq!(heard.2.spoken(), ["shot coming", "now"]);
}

#[test]
fn test_head_its_cue_is_toward_its_pokemon_and_logged() {
    let (found, heard) = sounds(&[CUE_SOUND]);
    for column in 0..3 {
        found.cue(column);
    }
    assert_eq!(
        played(&heard),
        [
            (CUE_SOUND.to_string(), 0.0),
            (CUE_SOUND.to_string(), 0.5),
            (CUE_SOUND.to_string(), 1.0),
        ]
    );
    assert_eq!(
        logged(&heard),
        [
            "head it: cue, left",
            "head it: cue, middle",
            "head it: cue, right"
        ]
    );
}

#[test]
fn test_a_drag_is_heard_only_where_both_its_sounds_are() {
    assert!(sounds(&[PICK_SOUND]).0.dragged().is_none());
    let (found, heard) = sounds(&[PICK_SOUND, DROP_SOUND]);
    let drag = found.dragged().expect("both sounds are there");
    drag(true);
    drag(false);
    assert_eq!(
        played(&heard),
        [(PICK_SOUND.to_string(), 0.5), (DROP_SOUND.to_string(), 0.5)]
    );
}
