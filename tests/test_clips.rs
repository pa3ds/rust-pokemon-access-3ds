//! The game's own sounds pa3ds plays: a clip cuts off only what its own voice was playing.

use std::cell::RefCell;
use std::rc::Rc;

use crate::core::bcsar::Sound;
use crate::core::clips::{Clips, Output};
use crate::core::loudness::Loudness;

/// A waveOut that keeps what it was given.
struct FakeOut {
    rate: i64,
    played: Rc<RefCell<Vec<Vec<u8>>>>,
}

impl Output for FakeOut {
    fn rate(&self) -> i64 {
        self.rate
    }

    fn play(&mut self, data: &[u8]) {
        self.played.borrow_mut().push(data.to_vec());
    }
}

/// One Pokémon's cry cut off another's, and waiting their turn they would
/// come too late. A voice a side: a cry cuts off its own Pokémon's last, not the others'.
#[test]
fn test_each_voice_has_an_output_of_its_own() {
    // What each output opened has played, in the order they were opened.
    let opened: Rc<RefCell<Vec<Rc<RefCell<Vec<Vec<u8>>>>>>> = Rc::new(RefCell::new(Vec::new()));
    let played = Clips::opening(Loudness::full(), 1.0, {
        let opened = opened.clone();
        Rc::new(move |rate| {
            let played = Rc::new(RefCell::new(Vec::new()));
            opened.borrow_mut().push(played.clone());
            Ok(Box::new(FakeOut { rate, played }) as Box<dyn Output>)
        })
    });
    played.add(
        "cry",
        Sound {
            rate: 32728,
            samples: vec![1000; 8],
            volume: 1.0,
            quietest: 1.0,
        },
    );
    played.play_on("cry", 0.0, 0).unwrap();
    played.play_on("cry", 1.0, 2).unwrap();
    played.play_on("cry", 0.0, 0).unwrap();
    assert_eq!(
        opened
            .borrow()
            .iter()
            .map(|out| out.borrow().len())
            .collect::<Vec<_>>(),
        [2, 1]
    );
    // One voice, as every other clip has: the same output each time.
    played.play("cry").unwrap();
    assert_eq!(opened.borrow().len(), 2);
}
