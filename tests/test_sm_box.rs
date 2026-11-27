//! Sun's PC Box: the slot under the cursor and the Pokémon in it, from the Box's own objects, laid
//! out as Sun's Route 1 Pokémon Center has them.

use std::rc::Rc;

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_battle::record_of;
use crate::core::rtti::ClassNames;
use crate::games::common::pokemon;
use crate::games::sm::r#box::{self, BoxReader};

const FRAME: i64 = 0x30000BCC;
const STATUS: i64 = 0x30001BFC;
const PARAM: i64 = 0x30000258;

struct Box {
    memory: Rc<FakeMemory>,
    reader: BoxReader,
}

impl Box {
    fn new() -> Box {
        let memory = FakeMemory::new();
        let classes = FakeClasses::new(memory.clone());
        memory.u32(FRAME, &[classes.add(r#box::FRAME)]);
        memory.u32(STATUS, &[classes.add(r#box::STATUS)]);
        memory.u32(STATUS + r#box::PANEL, &[PARAM]);
        memory.u32(
            PARAM,
            &[
                classes.add(pokemon::PARAM_CLASS),
                PARAM + pokemon::RECORD_IN_PARAM,
            ],
        );
        let mut reader = BoxReader::new(
            memory.clone(),
            Rc::new(ClassNames::new(memory.clone(), CODE)),
        );
        reader.track([FRAME, STATUS], false);
        Box { memory, reader }
    }

    /// Put the cursor on `slot` over `nickname`, or an empty slot, and poll till it settles.
    fn at(&mut self, slot: i64, nickname: Option<&str>) -> Vec<String> {
        self.memory.u32(FRAME + r#box::SLOT, &[slot]);
        let held = match nickname {
            None => vec![0u8; pokemon::RECORD_SIZE as usize],
            Some(nickname) => record_of(nickname, pokemon::FEMALE, 79),
        };
        self.memory.write(PARAM + pokemon::RECORD_IN_PARAM, &held);
        (0..r#box::STEADY + 1)
            .filter_map(|_| self.reader.poll().unwrap())
            .map(|focus| focus.label)
            .collect()
    }
}

#[test]
fn test_a_slot_is_said_by_its_row_and_column_with_the_pokemon_in_it_or_empty() {
    let mut shown = Box::new();
    assert_eq!(shown.at(5, Some("Slowpoke")), ["row 1, column 6, Slowpoke"]);
    assert_eq!(shown.at(4, None), ["row 1, column 5, empty"]);
    assert_eq!(shown.at(30, Some("Rowlet")), ["Party 1, Rowlet"]);
    assert!(shown.at(30, Some("Rowlet")).is_empty());
}

/// The panel showed the slot before's Pokémon a poll or two after the cursor moved.
#[test]
fn test_a_slot_is_not_said_until_the_panel_has_caught_up_with_the_cursor() {
    let mut shown = Box::new();
    shown.at(30, Some("Rowlet"));
    shown.memory.u32(FRAME + r#box::SLOT, &[31]);
    assert_eq!(
        shown.reader.poll().unwrap(),
        None,
        "Party 2 still showing Rowlet"
    );
    assert_eq!(shown.at(31, Some("Wingull")), ["Party 2, Wingull"]);
}

/// Carrying Rowlet over the box, the panel showed Rowlet on every slot and each was said as
/// Rowlet's (spike 318). A slot passed before says what it held, one never passed its place, and
/// a turn of the box forgets them.
#[test]
fn test_while_carrying_a_slot_says_what_it_held_when_last_passed() {
    let mut shown = Box::new();
    shown.at(5, None);
    shown.at(4, Some("Slowpoke"));
    shown.at(30, Some("Rowlet"));
    shown.memory.write(FRAME + r#box::STATE, &[r#box::CARRYING]);
    assert_eq!(shown.at(5, Some("Rowlet")), ["row 1, column 6, empty"]);
    assert_eq!(shown.at(4, Some("Rowlet")), ["row 1, column 5, Slowpoke"]);
    assert_eq!(shown.at(3, Some("Rowlet")), ["row 1, column 4"]);
    shown.memory.write(FRAME + r#box::STATE, &[r#box::TURNING]);
    shown.reader.poll().unwrap();
    shown.memory.write(FRAME + r#box::STATE, &[r#box::CARRYING]);
    assert_eq!(shown.at(5, Some("Rowlet")), ["row 1, column 6"]);
}
