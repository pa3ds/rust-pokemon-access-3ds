//! The Bag's party is found by its shape: six PokemonParam pointers, then how many are filled.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_layouts::FakeLayouts;
use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use super::test_pokemon::{WURMPLE, second_wurmple};
use crate::core::rtti::ClassNames;
use crate::games::common::layout_file::LayoutFile;
use crate::games::common::screen::ScreenText;
use crate::games::common::screen_order::{in_rows, join};
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::pokeicon;
use crate::games::gen6::screens::partyicons;

const PARTY: i64 = 0x08CF7200;
const PARAMS: i64 = 0x08CF723C;

#[test]
fn test_the_party_is_six_params_and_a_count() {
    let memory = FakeMemory::new();
    let param = FakeClasses::new(memory.clone()).add(pokemon::PARAM_CLASS);
    for i in 0..6 {
        memory.u32(PARAMS + 0x1E4 * i, &[param]);
        memory.u32(PARTY + 4 * i, &[PARAMS + 0x1E4 * i]);
    }
    memory.u32(PARTY + partyicons::COUNT, &[5]);
    let names = ClassNames::new(memory.clone(), CODE);
    // A word holding a param on its own is not the party; the list is.
    assert_eq!(
        partyicons::party_of([PARTY + 4, PARTY], &*memory, &names, &[]).unwrap(),
        Some(PARTY)
    );
    memory.u32(PARTY + partyicons::COUNT, &[7]);
    assert_eq!(
        partyicons::party_of([PARTY], &*memory, &names, &[]).unwrap(),
        None
    );
    // A byte: the Battle Maison sets the one after it (or_multi_battle_partners).
    memory.u32(PARTY + partyicons::COUNT, &[0x206]);
    assert_eq!(
        partyicons::party_of([PARTY], &*memory, &names, &[]).unwrap(),
        Some(PARTY)
    );
}

/// Choosing the Battle Box or the party, the top screen draws both from twelve textures, the
/// Battle Box's six first: "L 73 L 71" said no species (or_multi_battle_partners).
#[test]
fn test_the_maisons_group_screen_draws_the_party_after_the_battle_box() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let param = classes.add(pokemon::PARAM_CLASS);
    for i in 0..6 {
        let at = PARAMS + 0x1E4 * i;
        memory.u32(at, &[param, at + pokemon::RECORD_IN_PARAM]);
        memory.u32(PARTY + 4 * i, &[at]);
    }
    memory.write(PARAMS + 0x1E4 + pokemon::RECORD_IN_PARAM, &second_wurmple());
    memory.u32(PARTY + partyicons::COUNT, &[0x206]);
    let (icon, replace, table, picture) = (0x08647000, 0x08647100, 0x08647200, 0x08648000);
    memory.u32(icon, &[classes.add(pokeicon::ICON_CLASS)]);
    memory.u32(icon + pokeicon::REPLACE, &[replace]);
    memory.u32(replace, &[classes.add(pokeicon::REPLACE_CLASS)]);
    memory.u32(replace + pokeicon::TEXTURES, &[table, 12]);
    for i in 0..12 {
        memory.u32(table + 4 * i, &[0x08649000 + 0x10 * i]);
        memory.u32(
            0x08649000 + 0x10 * i + pokeicon::IMAGE,
            &[if i < 6 { 0 } else { 0x1000 + i }],
        );
    }
    memory.u32(picture + lyt::PICTURE_MATERIAL, &[picture + 0x200]);
    memory.u32(picture + 0x200 + lyt::MATERIAL_TEXTURE, &[0x1007]);
    let running = Rc::new(RefCell::new(partyicons::GROUPS.to_string()));
    let mut species = vec![String::new(); WURMPLE as usize];
    species.push("Wurmple".to_string());
    let mut icons = partyicons::PartyIcons::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        species,
        {
            let running = running.clone();
            Rc::new(move || Some(running.borrow().clone()))
        },
    );
    icons
        .track(
            &[PARAMS],
            [icon],
            &|_params: &[i64]| Ok(vec![PARTY]),
            false,
            [],
        )
        .unwrap();
    assert_eq!(icons.name(picture).unwrap().as_deref(), Some("Wurmple"));
    // In the Bag the seventh texture is no party member's.
    *running.borrow_mut() = partyicons::BAG.to_string();
    icons
        .track(
            &[PARAMS],
            [icon],
            &|_params: &[i64]| Ok(vec![PARTY]),
            false,
            [],
        )
        .unwrap();
    assert_eq!(icons.name(picture).unwrap(), None);
    // The Battle Box's are its first six, named from the save's Battle Box.
    *running.borrow_mut() = partyicons::GROUPS.to_string();
    let r#box = 0x08C6E33C;
    memory.u32(r#box, &[classes.add(partyicons::BATTLE_BOX)]);
    memory.write(r#box + partyicons::RECORDS, &second_wurmple());
    memory.u32(0x08649000 + pokeicon::IMAGE, &[0x2000]);
    memory.u32(picture + 0x200 + lyt::MATERIAL_TEXTURE, &[0x2000]);
    icons
        .track(
            &[PARAMS],
            [icon],
            &|_params: &[i64]| Ok(vec![PARTY]),
            false,
            [r#box],
        )
        .unwrap();
    assert_eq!(icons.name(picture).unwrap().as_deref(), Some("Wurmple"));
    // The Colosseum's BATTLE POKÉMON draws them so too, where only genders and levels were said
    // (Y, a link battle; spike 318's decompile).
    *running.borrow_mut() = partyicons::COLOSSEUM.to_string();
    icons
        .track(
            &[PARAMS],
            [icon],
            &|_params: &[i64]| Ok(vec![PARTY]),
            false,
            [r#box],
        )
        .unwrap();
    assert_eq!(icons.name(picture).unwrap().as_deref(), Some("Wurmple"));
    memory.u32(picture + 0x200 + lyt::MATERIAL_TEXTURE, &[0x1007]);
    icons
        .track(
            &[PARAMS],
            [icon],
            &|_params: &[i64]| Ok(vec![PARTY]),
            false,
            [r#box],
        )
        .unwrap();
    assert_eq!(icons.name(picture).unwrap().as_deref(), Some("Wurmple"));
}

/// Before a link battle the party list's top screen drew the other player's team as icons,
/// "zen, L50, , L50" (spike 319), from the teams the list's process was opened with: one team,
/// the second, in the first six slots, a multi battle's three six slots apiece, a slot the n-th
/// icon picture of the layout (spike 318's decompile). Out of a link battle there is none.
#[test]
fn test_the_party_lists_top_screen_names_the_other_players_team() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let param = classes.add(pokemon::PARAM_CLASS);
    let team = PARTY;
    for i in 0..6 {
        let at = PARAMS + 0x1E4 * i;
        memory.u32(at, &[param, at + pokemon::RECORD_IN_PARAM]);
        memory.u32(team + 4 * i, &[at]);
    }
    memory.write(PARAMS + 0x1E4 + pokemon::RECORD_IN_PARAM, &second_wurmple());
    memory.u32(team + partyicons::COUNT, &[3]);
    let fake = FakeLayouts::new(&memory, &classes);
    let picture = classes.add(lyt::PICTURE_CLASS);
    let (_work, root) = fake.screen_with(0x08200000, false, false);
    // Eighteen slots, a pane each holding its icon; and a picture of another texture.
    let mut icons: Vec<i64> = Vec::new();
    for slot in 0..18 {
        let pane = root + 0x1000 * (slot + 1);
        fake.pane(pane, root);
        fake.pane_with_vtable(pane + 0x200, pane, Some(picture));
        let mut name = b"i".to_vec();
        name.resize(lyt::NAME_SIZE as usize, 0);
        memory.write(pane + 0x200 + lyt::NAME, &name);
        icons.push(pane + 0x200);
    }
    fake.pane_with_vtable(root + 0x20000, root, Some(picture));
    fake.track().unwrap();
    let file = Rc::new(LayoutFile {
        name: partyicons::TOP.to_string(),
        ..LayoutFile::new(
            Vec::new(),
            IndexMap::from([("i".to_string(), vec![partyicons::ICON.to_string()])]),
            IndexMap::new(),
        )
    });
    let process = Rc::new(RefCell::new(vec![0u8; 0x154]));

    let opened = |teams: &[i64]| {
        let mut process = process.borrow_mut();
        *process = vec![0u8; 0x154];
        for (&at, &listed) in partyicons::TEAMS.iter().zip(teams) {
            let start = (partyicons::OPENED_WITH + at) as usize;
            process[start..start + 4].copy_from_slice(&(listed as u32).to_le_bytes());
        }
    };

    let mut species = vec![String::new(); WURMPLE as usize];
    species.push("Wurmple".to_string());
    let mut names = partyicons::PartyIcons::with_list(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        species,
        Rc::new(|| Some(partyicons::LIST.to_string())),
        {
            let process = process.clone();
            Rc::new(move || process.borrow().clone())
        },
        Some(fake.layouts.clone()),
        Rc::new(move |_root| Some(file.clone())),
    );
    // The field's party menu: no team, and nothing read.
    assert_eq!(names.name(icons[1]).unwrap(), None);
    opened(&[0, team]);
    assert_eq!(
        [icons[1], icons[7], root + 0x20000]
            .iter()
            .map(|&p| names.name(p).unwrap())
            .collect::<Vec<_>>(),
        [Some("Wurmple".to_string()), None, None]
    );
    // Three teams: the second six slots are the second.
    opened(&[team + 0x100, team, team + 0x200]);
    names
        .track(
            &[],
            Vec::<i64>::new(),
            &|_params: &[i64]| Ok(Vec::new()),
            false,
            Vec::<i64>::new(),
        )
        .unwrap();
    assert_eq!(
        [icons[1], icons[7]]
            .iter()
            .map(|&p| names.name(p).unwrap())
            .collect::<Vec<_>>(),
        [None, Some("Wurmple".to_string())]
    );
    // Its gender sign, drawn right of the icon over the level, said after the species, as a
    // battle says "Wurmple♂ L50", not after the level.
    let slot = icons[7] - 0x200;
    let drawn: IndexMap<i64, ScreenText> = [
        (icons[7], "Wurmple", -5.0, 5.0),
        (slot + 0x300, "L", -15.0, -15.0),
        (slot + 0x400, "50", -5.0, -15.0),
        (slot + 0x500, "\u{E08E}", 22.0, 10.0),
    ]
    .into_iter()
    .map(|(r#box, text, x, y)| {
        (
            r#box,
            ScreenText {
                left: x,
                right: x + 10.0,
                row: y,
                glyph: 13.0,
                top: true,
                root,
                parent: slot,
                picture: r#box == icons[7],
                ..ScreenText::new(r#box, text, false)
            },
        )
    })
    .collect();
    let mut worded: Vec<ScreenText> = names.reworded(drawn).into_values().collect();
    worded.sort_by_key(|p| p.r#box);
    let in_order = in_rows(worded);
    assert_eq!(join(&in_order, false), "Wurmple \u{E08E} L50");
}

/// At the Maison five more lists of six are kept, and the Battle Box's, counting its two, was
/// taken for the party (or_battle_box). The save's party names its members by their records'
/// encryption constants.
#[test]
fn test_the_party_is_the_list_holding_the_saves_party() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let param = classes.add(pokemon::PARAM_CLASS);
    let (copy, party) = (PARTY - 0x1000, PARTY);
    for (holder, constants) in [(copy, vec![7, 8]), (party, vec![1, 2, 3])] {
        for i in 0..6 {
            let at = holder + 0x40 + 0x1E4 * i;
            memory.u32(at, &[param, at + pokemon::RECORD_IN_PARAM]);
            memory.u32(
                at + pokemon::RECORD_IN_PARAM,
                &[constants.get(i as usize).copied().unwrap_or(0)],
            );
            memory.u32(holder + 4 * i, &[at]);
        }
        memory.u32(holder + partyicons::COUNT, &[constants.len() as i64]);
    }
    let save = 0x08C7D4C8;
    for (i, constant) in [1, 2, 3].into_iter().enumerate() {
        memory.u32(
            save + partyicons::RECORDS + partyicons::SAVED * i as i64,
            &[constant],
        );
    }
    memory.write(save + partyicons::RECORDS + partyicons::SAVED * 6, &[3]);
    let names = ClassNames::new(memory.clone(), CODE);
    let saved = partyicons::saved_party(&*memory, save).unwrap();
    assert_eq!(saved, [1, 2, 3]);
    assert_eq!(
        partyicons::party_of([copy, party], &*memory, &names, &[]).unwrap(),
        Some(copy)
    );
    assert_eq!(
        partyicons::party_of([copy, party], &*memory, &names, &saved).unwrap(),
        Some(party)
    );
}

/// Another screen's icons, 21 textures, hold an empty texture at slot 16, as a picture drawing
/// nothing does; the party's list, found in an earlier Bag, now counts past six. Its seven words
/// hold no slot 16 (a player's Bag, twice, 2026-10-18).
#[test]
fn test_a_slot_past_six_is_no_party_member() {
    let memory = FakeMemory::new();
    let classes = FakeClasses::new(memory.clone());
    let param = classes.add(pokemon::PARAM_CLASS);
    for i in 0..6 {
        memory.u32(PARAMS + 0x1E4 * i, &[param]);
        memory.u32(PARTY + 4 * i, &[PARAMS + 0x1E4 * i]);
    }
    memory.u32(PARTY + partyicons::COUNT, &[5]);
    let (icon, replace, table, picture) = (0x08647000, 0x08647100, 0x08647200, 0x08648000);
    memory.u32(icon, &[classes.add(pokeicon::ICON_CLASS)]);
    memory.u32(icon + pokeicon::REPLACE, &[replace]);
    memory.u32(replace, &[classes.add(pokeicon::REPLACE_CLASS)]);
    memory.u32(replace + pokeicon::TEXTURES, &[table, 21]);
    for i in 0..21 {
        memory.u32(table + 4 * i, &[0x08649000 + 0x10 * i]);
        memory.u32(
            0x08649000 + 0x10 * i + pokeicon::IMAGE,
            &[if i >= 16 { 0 } else { 0x1000 + i }],
        );
    }
    memory.u32(picture + lyt::PICTURE_MATERIAL, &[picture + 0x200]);
    let mut icons = partyicons::PartyIcons::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), CODE)),
        vec![String::new(), "Bulbasaur".to_string()],
        Rc::new(|| Some(partyicons::BAG.to_string())),
    );
    icons
        .track(
            &[PARAMS],
            [icon],
            &|_params: &[i64]| Ok(vec![PARTY]),
            false,
            [],
        )
        .unwrap();
    memory.u32(PARTY + partyicons::COUNT, &[0x20]);
    assert_eq!(icons.name(picture).unwrap(), None);
}
