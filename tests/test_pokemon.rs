//! Pokémon records, against two the game really built.
//!
//! Both are wild Wurmple from battles.ctm (spike 65): the first battle's, whose blocks are stored
//! in order 4, and the second's, in order 22. A decoder with those two entries of the order table
//! wrong still passes the checksum, so the names are what is checked.

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::binary::{u16, u32};
use crate::core::rtti::ClassNames;
use crate::games::common::pokemon::{
    self, FEMALE, HP, LEVEL, PARAM_CLASS, PARAM_RECORD, Pokemon, RECORD_IN_PARAM, SPECIES_MAX,
    crypt, decrypt, parse,
};

/// `bytes.fromhex(...)`, the text given in lines.
fn from_hex(lines: &[&str]) -> Vec<u8> {
    let text: String = lines.concat();
    (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&text[at..at + 2], 16).expect("hex digits"))
        .collect()
}

fn first_wurmple() -> Vec<u8> {
    from_hex(&[
        "119107ad0000e1ba9b7f9f1d0e53a73396c0116f6bdae047885afcef1ac3ce82586dfa4a2ef0f19c8b040b1879939652",
        "d04b4efc4631f6885b82814d3641aa116e11f320458d5d31fbbe59d3089fc2dd8c66e13936bdc8e6675cc1d0693d0c7f",
        "ecdea779252e8fd7423ad4a2189ea8c20380e6e0a3fd3b9205062f65ea7347a515e4bab620060c58015d09c7b1ff1c4d",
        "72ce9c3374fc743fde790519ed6d7ad5a9767aa35f999a261a78c63da67b72df3e49ffb995dad8750b23577ed012b028",
        "c940d2436c4b3db2ad561892e1af466ec6ce8c240c630ce01cf09c8ceb34c1ce16545aa5a4f03bb5",
    ])
}

fn first_stats() -> Vec<u8> {
    from_hex(&["927e9f1da38c8ea49dc0016f70d8e747a9e05c0c10c1ce82586dfa4a"])
}

pub(crate) fn second_wurmple() -> Vec<u8> {
    from_hex(&[
        "f7d67e2600003ef88b03233797268b5848557dc0149eb12772c5003836fd90bcb04a8494ace0f10bb6da43db53c91a90",
        "0febe4a893d663432d389b00099708b05c24de5a66fd4129ed09e70296578f8153881b7b23f259d2778cc256dce1b628",
        "f180fca6b1e58c2b891cf420e082d90efa94567356bf5068f0386a7ee1c10e2e7ebcf1662cc12fc1486469bef9dc24ef",
        "30c50ac5fbca45825e9011f14377273136c7d127000ec66990d21e6cd1f1dd7e0ca999ea577a1b56ed3d6f9ebf11c46d",
        "b8b5f265269c3046eef565cfeb43b18f273ed2de241860a4f0df1742741d4a29c750e9dccb88d3de",
    ])
}

fn second_stats() -> Vec<u8> {
    from_hex(&["8b03233795268b58455570c0139eb72774c5043830fd90bcb04ac294"])
}

pub(crate) const WURMPLE: i64 = 265;

fn wurmple() -> Pokemon {
    Pokemon {
        species: WURMPLE,
        nickname: "Wurmple".to_string(),
        gender: FEMALE,
        nicknamed: false,
        form: 0,
    }
}

#[test]
fn test_a_record_stored_in_order_4_decodes_to_its_name() {
    let plain = decrypt(&first_wurmple());
    assert!(plain.is_some());
    assert_eq!(
        parse(&plain.unwrap(), SPECIES_MAX).unwrap(),
        Some(wurmple())
    );
}

#[test]
fn test_a_record_stored_in_order_22_decodes_to_its_name() {
    let plain = decrypt(&second_wurmple());
    assert!(plain.is_some());
    assert_eq!(
        parse(&plain.unwrap(), SPECIES_MAX).unwrap(),
        Some(wurmple())
    );
}

#[test]
fn test_battle_stats_use_the_keystream_started_over() {
    let first = crypt(&first_stats(), u32(&first_wurmple(), 0).unwrap());
    let second = crypt(&second_stats(), u32(&second_wurmple(), 0).unwrap());
    assert_eq!(
        (
            i64::from(first[LEVEL as usize]),
            u16(&first, HP).unwrap(),
            u16(&first, HP + 2).unwrap()
        ),
        (3, 16, 16)
    );
    assert_eq!(
        (
            i64::from(second[LEVEL as usize]),
            u16(&second, HP).unwrap(),
            u16(&second, HP + 2).unwrap()
        ),
        (2, 13, 13)
    );
}

#[test]
fn test_a_record_whose_sum_does_not_hold_is_not_one() {
    let mut damaged = first_wurmple();
    damaged[0x50] ^= 0x01;
    assert_eq!(decrypt(&damaged), None);
}

#[test]
fn test_a_pokemon_is_read_through_the_object_holding_it() {
    let memory = FakeMemory::new();
    let param_class = FakeClasses::new(memory.clone()).add(PARAM_CLASS);
    let param = 0x081FEE88;
    memory.u32(param, &[param_class, param + RECORD_IN_PARAM]);
    memory.write(param + RECORD_IN_PARAM, &second_wurmple());
    let names = ClassNames::new(memory.clone(), CODE);
    assert_eq!(
        pokemon::read(&*memory, &names, param, SPECIES_MAX).unwrap(),
        Some(wurmple())
    );
    memory.u32(param + PARAM_RECORD, &[0x08000000]);
    assert_eq!(
        pokemon::read(&*memory, &names, param, SPECIES_MAX).unwrap(),
        None
    );
}
