//! Pictures named by the image they draw: the ROM's textures indexed, and a picture's image read.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use super::fake_rom::{GarcFile, TmpDir, garc, lz11_literals, rom_image};
use crate::core::memory::Memory;
use crate::core::rom::Rom;
use crate::games::common::garc::{garc_files, garc_variants};
use crate::games::common::{layout_file, lyt, pictures};
use crate::games::gen6::rom::archives::GameText;
use crate::games::gen6::rom::textures;
use crate::py::Result;

const TITLE: i64 = 0x000400000011C400;

fn le16(value: i64) -> Vec<u8> {
    (value as u16).to_le_bytes().to_vec()
}

fn le32(value: i64) -> Vec<u8> {
    (value as u32).to_le_bytes().to_vec()
}

/// A `.bclim`: its image, then the CLIM header and the `imag` block.
fn texture(image: &[u8], width: i64, height: i64, fmt: i64) -> Vec<u8> {
    let mut file = image.to_vec();
    file.extend(b"CLIM");
    file.extend(le16(0xFEFF));
    file.extend(le16(0x14));
    file.extend(le32(0x02020000));
    file.extend(le32(image.len() as i64 + 0x28));
    file.extend(le16(1));
    file.extend(le16(0));
    file.extend(b"imag");
    file.extend(le32(0x10));
    file.extend(le16(width));
    file.extend(le16(height));
    file.extend(le32(fmt));
    file.extend(le32(image.len() as i64));
    file
}

/// `texture(image)`: 8 by 8, format 9.
fn plain_texture(image: &[u8]) -> Vec<u8> {
    texture(image, 8, 8, 9)
}

/// `bytes(-len(data) % to)`.
fn padding(length: usize, to: i64) -> Vec<u8> {
    vec![0; crate::py::modulo(-(length as i64), to) as usize]
}

/// A `darc` holding these files under its root.
fn darc(files: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let count = files.len() as i64 + 1;
    let mut names: Vec<u8> = vec![0, 0];
    let mut offsets: Vec<i64> = Vec::new();
    for (name, _) in files {
        offsets.push(names.len() as i64);
        names.extend(
            format!("{name}\0")
                .encode_utf16()
                .flat_map(u16::to_le_bytes),
        );
    }
    let table_size = count * layout_file::DARC_ENTRY + names.len() as i64;
    let data_at = 0x1C + table_size + crate::py::modulo(-(0x1C + table_size), 0x80);
    let mut table: Vec<u8> = Vec::new();
    table.extend(le32(layout_file::DIRECTORY));
    table.extend(le32(0));
    table.extend(le32(count));
    let mut data: Vec<u8> = Vec::new();
    for (offset, (_, blob)) in offsets.iter().zip(files) {
        table.extend(le32(*offset));
        table.extend(le32(data_at + data.len() as i64));
        table.extend(le32(blob.len() as i64));
        data.extend(blob);
        data.extend(padding(blob.len(), 0x80));
    }
    let mut body = table;
    body.extend(&names);
    body.extend(vec![0; (data_at - 0x1C) as usize - body.len()]);
    let size = data_at + data.len() as i64;
    let mut out = b"darc".to_vec();
    out.extend(le16(0xFEFF));
    out.extend(le16(0x1C));
    out.extend(le32(0x01000000));
    out.extend(le32(size));
    out.extend(le32(0x1C));
    out.extend(le32(table_size));
    out.extend(le32(data_at));
    out.extend(body);
    out.extend(data);
    out
}

fn fire() -> Vec<u8> {
    (0..=255).collect()
}

fn fra() -> Vec<u8> {
    vec![1; 256]
}

fn fre() -> Vec<u8> {
    vec![2; 256]
}

/// A layout pack: a table of names, then its darcs, LZ11-compressed as the ROM keeps them.
fn packed(archives: &[Vec<u8>]) -> Vec<u8> {
    let mut pack = b"\x6c\0\0\0names of the pack\0".to_vec();
    pack.extend([0; 12]);
    for archive in archives {
        pack.extend(archive);
    }
    lz11_literals(&pack)
}

fn index_of(pairs: &[(String, &str)]) -> IndexMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| (key.clone(), value.to_string()))
        .collect()
}

#[test]
fn test_every_copy_of_a_garc_entry_is_read_and_the_first_kept_by_garc_files() {
    // The English game draws copy 2 of the Pokédex's tabs, where copy 0 has FRA (spike 147).
    let archive = garc(&[
        GarcFile::Copies(IndexMap::from([(0, b"FRA".to_vec()), (2, b"FRE".to_vec())])),
        GarcFile::One(b"alone".to_vec()),
    ]);
    assert_eq!(
        garc_variants(&archive).unwrap(),
        vec![
            IndexMap::from([(0, b"FRA".to_vec()), (2, b"FRE".to_vec())]),
            IndexMap::from([(0, b"alone".to_vec())])
        ]
    );
    assert_eq!(
        garc_files(&archive).unwrap(),
        vec![b"FRA".to_vec(), b"alone".to_vec()]
    );
}

#[test]
fn test_the_census_names_every_copy_of_a_texture_by_its_image() {
    let tabs = garc(&[GarcFile::Copies(IndexMap::from([
        (
            0,
            packed(&[darc(&[("word_button_fra.bclim", plain_texture(&fra()))])]),
        ),
        (
            2,
            packed(&[darc(&[("word_button_fra.bclim", plain_texture(&fre()))])]),
        ),
    ]))]);
    let icons = garc(&[GarcFile::One(packed(&[darc(&[
        ("type_icon_09_honoo.bclim", plain_texture(&fire())),
        ("a.bclyt", b"x".to_vec()),
    ])]))]);
    let tmp_path = TmpDir::new();
    let path = tmp_path.path().join("or.3ds");
    std::fs::write(
        &path,
        rom_image(
            &[
                ("a/2/3/5", &tabs),
                ("a/2/1/2", &icons),
                ("a/0/0/1", b"not one"),
            ],
            TITLE,
        ),
    )
    .unwrap();
    let index = {
        let rom = Rom::new(&path).unwrap();
        textures::census(&rom, &["a/2/3/5", "a/2/1/2", "a/0/0/1"]).unwrap()
    };
    assert_eq!(
        index,
        index_of(&[
            (textures::digest(&fra()), "word_button_fra"),
            (textures::digest(&fre()), "word_button_fra"),
            (textures::digest(&fire()), "type_icon_09_honoo"),
        ])
    );
}

#[test]
fn test_a_texture_with_no_name_is_not_indexed() {
    // A Pokémon's picture in the Pokédex is a file of its own, named by nothing but its number.
    let tmp_path = TmpDir::new();
    let path = tmp_path.path().join("or.3ds");
    let pictures = garc(&[GarcFile::One(lz11_literals(&plain_texture(&fire())))]);
    std::fs::write(&path, rom_image(&[("a/2/6/3", &pictures)], TITLE)).unwrap();
    let rom = Rom::new(&path).unwrap();
    assert_eq!(
        textures::census(&rom, &["a/2/6/3"]).unwrap(),
        IndexMap::<String, String>::new()
    );
}

#[test]
fn test_the_index_is_kept_as_toml_and_read_back() {
    let tmp_path = TmpDir::new();
    let path = tmp_path
        .path()
        .join("cache")
        .join("textures")
        .join("sango-1.toml");
    let index = index_of(&[(
        textures::digest(&fire()),
        "type_icon_09_honoo s_type_icon_09_honoo",
    )]);
    assert!(
        textures::render(&index, "Pokémon Omega Ruby", "sango-1")
            .starts_with("# The textures of Pokémon Omega Ruby, ROM sango-1")
    );
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    // `Path.write_text` writes each "\n" as the platform's line ending.
    let text = textures::render(&index, "", "sango-1");
    let text = if cfg!(windows) {
        text.replace('\n', "\r\n")
    } else {
        text
    };
    std::fs::write(&path, text).unwrap();
    assert_eq!(textures::load(&path).unwrap(), index);
    assert_eq!(
        textures::load(&tmp_path.path().join("missing.toml")).unwrap(),
        IndexMap::<String, String>::new()
    );
}

/// A picture whose material draws the image at `physical`.
fn picture(memory: &FakeMemory, physical: i64, stored: (i64, i64), fmt: i64) -> i64 {
    let (pane, material) = (0x08000000, 0x08001000);
    memory.u32(pane + lyt::PICTURE_MATERIAL, &[material]);
    memory.u32(material + lyt::MATERIAL_TEXTURE, &[physical]);
    memory.u16(material + lyt::MATERIAL_STORED, stored.0);
    memory.u16(material + lyt::MATERIAL_STORED + 2, stored.1);
    memory.u32(material + lyt::MATERIAL_FORMAT, &[fmt]);
    pane
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| word.to_string()).collect()
}

#[test]
fn test_a_picture_is_named_by_the_image_its_material_draws_from_fcram_or_video_memory() {
    let index = index_of(&[(
        textures::digest(&fire()),
        "type_icon_09_honoo s_type_icon_09_honoo",
    )]);
    let memory = FakeMemory::new();
    memory.write(0x14100000, &fire());
    // 16 by 16 at 8 bits a pixel, ETC1A4, is 256 bytes, FCRAM seen as the linear heap.
    let pane = picture(&memory, 0x20100000, (16, 16), 0xD);
    assert_eq!(
        textures::names(&index, &*memory, pane).unwrap(),
        strings(&["type_icon_09_honoo", "s_type_icon_09_honoo"])
    );
    memory.write(0x1F520000, &fire());
    let pane = picture(&memory, 0x18520000, (8, 8), 0x0);
    assert_eq!(
        textures::names(&index, &*memory, pane).unwrap(),
        strings(&["type_icon_09_honoo", "s_type_icon_09_honoo"])
    );
}

/// Memory that keeps where each read began.
struct Reads {
    memory: Rc<FakeMemory>,
    at: RefCell<Vec<i64>>,
}

impl Reads {
    fn new() -> Reads {
        Reads {
            memory: FakeMemory::new(),
            at: RefCell::new(Vec::new()),
        }
    }
}

impl Memory for Reads {
    fn read(&self, address: i64, size: i64) -> Result<Vec<u8>> {
        self.at.borrow_mut().push(address);
        self.memory.read(address, size)
    }
}

#[test]
fn test_an_image_larger_than_any_icon_is_never_read() {
    // Reading the GPU's memory makes Azahar flush its copy from the RPC's thread, and a battle's
    // start crashed it 5 times in 12 while pa3ds read a 128 KB image in video memory the GPU was
    // drawing into (spike 149, 2026-09-30). Every texture a rule names is 4 KB or less.
    let memory = Reads::new();
    let pane = picture(&memory.memory, 0x185A0000, (256, 128), 0x0);
    let index = index_of(&[(textures::digest(&vec![0; 0x20000]), "x")]);
    assert_eq!(
        textures::names(&index, &memory, pane).unwrap(),
        Vec::<String>::new()
    );
    assert!(
        !memory
            .at
            .borrow()
            .iter()
            .any(|at| (0x1F5A0000..0x1F5C0000).contains(at))
    );
}

#[test]
fn test_a_picture_drawing_nothing_known_has_no_name() {
    let memory = FakeMemory::new();
    assert_eq!(
        textures::names(&IndexMap::new(), &*memory, 0x08000000).unwrap(),
        Vec::<String>::new()
    );
    let pane = picture(&memory, 0x20100000, (16, 16), 0x77);
    let index = index_of(&[(textures::digest(&[0; 256]), "x")]);
    assert_eq!(
        textures::names(&index, &*memory, pane).unwrap(),
        Vec::<String>::new()
    );
}

fn english() -> GameText {
    GameText {
        languages: strings(&[
            "English", "Spanish", "French", "German", "Italian", "Japanese", "Korean",
        ]),
        types: strings(&[
            "Normal", "Fighting", "Flying", "Poison", "Ground", "Rock", "Bug", "Ghost", "Steel",
            "Fire",
        ]),
        categories: strings(&["STATUS", "PHYSICAL", "SPECIAL"]),
        conditions: strings(&[
            "PARALYSIS",
            "ASLEEP",
            "FROZEN",
            "BURNED",
            "POISONED",
            "FAINTED",
            "POKÉRUS",
        ]),
        contest_categories: strings(&["Coolness", "Beauty", "Cuteness", "Cleverness", "Toughness"]),
        ..GameText::default()
    }
}

#[test]
fn test_a_contest_moves_category_is_said_as_the_contest_names_it() {
    // Hold Back draws `contest_icon_00`, Iron Defense `contest_icon_04`, a Toughness move (spike
    // 196); a contest's heart is its sign, counted by the screen text reader.
    assert_eq!(
        textures::said(&["contest_icon_00"], &english()).unwrap(),
        Some("Coolness".to_string())
    );
    assert_eq!(
        textures::said(&["contest_icon_04"], &english()).unwrap(),
        Some("Toughness".to_string())
    );
    assert_eq!(
        textures::said(&["contest_icon_05"], &english()).unwrap(),
        None
    );
    assert_eq!(
        textures::said(&["heart"], &english()).unwrap(),
        Some(pictures::HEART.to_string())
    );
    assert_eq!(textures::said(&["mini_heart"], &english()).unwrap(), None);
    assert_eq!(
        textures::said(&["three"], &english()).unwrap(),
        Some("third place".to_string())
    );
}

#[test]
fn test_a_language_tab_is_said_as_the_game_names_its_language() {
    // Both of a tab's pictures name its language, in the player's language (the game's
    // own words where it has them).
    assert_eq!(
        textures::said(&["p_zkn_word_button_fra"], &english()).unwrap(),
        Some("French".to_string())
    );
    assert_eq!(
        textures::said(&["p_zkn_word_button_kor2"], &english()).unwrap(),
        Some("Korean".to_string())
    );
    assert_eq!(
        textures::said(&["p_zkn_word_button_xyz"], &english()).unwrap(),
        None
    );
}

#[test]
fn test_a_type_icon_is_said_as_the_game_names_the_type_by_its_number() {
    assert_eq!(
        textures::said(&["type_icon_09_honoo", "s_type_icon_09_honoo"], &english()).unwrap(),
        Some("Fire".to_string())
    );
    assert_eq!(
        textures::said(&["type_icon_17_yousei"], &english()).unwrap(),
        None
    );
    assert_eq!(
        textures::said(&["exit_button02"], &english()).unwrap(),
        None
    );
}

#[test]
fn test_a_condition_numbered_zero_is_no_condition() {
    assert_eq!(
        textures::said(&["sick_icon_00_x"], &english()).unwrap(),
        None
    );
    assert_eq!(
        textures::said(&["sick_icon_00_x"], &GameText::new(Vec::new(), Vec::new())).unwrap(),
        None
    );
}

#[test]
fn test_a_darc_cut_short_holds_no_textures() {
    let blob = [textures::DARC, textures::DARC_HEAD, &[0; 8]].concat();
    let mut found: Vec<String> = Vec::new();
    textures::named_textures(&blob, 0, &mut |name, _| found.push(name.to_string())).unwrap();
    assert!(found.is_empty());
}

#[test]
fn test_a_move_category_and_a_status_condition_are_said_in_the_games_words() {
    let said = |texture: &str| textures::said(&[texture], &english()).unwrap();
    assert_eq!(
        said("damage_type_icon_01_buturi"),
        Some("PHYSICAL".to_string())
    );
    assert_eq!(
        said("damage_type_icon_00_henka"),
        Some("STATUS".to_string())
    );
    assert_eq!(said("sick_icon_01_mahi"), Some("PARALYSIS".to_string()));
    assert_eq!(said("sick_icon_04_yakedo"), Some("BURNED".to_string()));
    // Badly poisoned has an icon of its own, and the game has no word for it but poisoned.
    assert_eq!(said("sick_icon_06_dokudoku"), Some("POISONED".to_string()));
    assert_eq!(said("pokerus_icon"), Some("POKÉRUS".to_string()));
}
