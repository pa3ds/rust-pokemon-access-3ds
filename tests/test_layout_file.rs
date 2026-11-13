//! Layout file tests, against CLYT files built the way spike 50 read the command menu's.

use indexmap::IndexMap;

use super::fake_memory::FakeMemory;
use super::test_textures::darc;
use crate::games::common::layout_file::{LayoutFile, LayoutFiles, find, parse, size_of};
use crate::games::common::pictures::{words, words_in};

fn le16(value: i64) -> [u8; 2] {
    (value as u16).to_le_bytes()
}

fn le32(value: i64) -> [u8; 4] {
    (value as u32).to_le_bytes()
}

/// `name.encode("ascii").ljust(width, b"\0")`.
fn ljust(name: &str, width: usize) -> Vec<u8> {
    let mut bytes = name.as_bytes().to_vec();
    if bytes.len() < width {
        bytes.resize(width, 0);
    }
    bytes
}

fn strings(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

/// A dict of names to the textures they are drawn with.
fn drawn(pairs: &[(&str, &[&str])]) -> IndexMap<String, Vec<String>> {
    pairs
        .iter()
        .map(|&(name, textures)| (name.to_string(), strings(textures)))
        .collect()
}

pub(crate) fn section(magic: &[u8], body: &[u8]) -> Vec<u8> {
    [magic, &le32(8 + body.len() as i64), body].concat()
}

pub(crate) fn names_table(names: &[&str]) -> Vec<u8> {
    let mut offsets: Vec<i64> = Vec::new();
    let mut strings: Vec<u8> = Vec::new();
    for name in names {
        offsets.push(4 * names.len() as i64 + strings.len() as i64);
        strings.extend(name.as_bytes());
        strings.push(0);
    }
    let mut table = le32(names.len() as i64).to_vec();
    for offset in offsets {
        table.extend(le32(offset));
    }
    table.extend(strings);
    table
}

pub(crate) fn material(name: &str, textures: &[i64]) -> Vec<u8> {
    let mut body = ljust(name, 20);
    body.extend(vec![0; 0x30 - 20]);
    body.extend(le32(textures.len() as i64));
    for &index in textures {
        body.extend(le16(index));
        body.extend([0, 0]);
    }
    body
}

pub(crate) fn materials(entries: &[Vec<u8>]) -> Vec<u8> {
    let mut offsets: Vec<i64> = Vec::new();
    let mut blobs: Vec<u8> = Vec::new();
    for entry in entries {
        offsets.push(0x0C + 4 * entries.len() as i64 + blobs.len() as i64);
        blobs.extend(entry);
    }
    let mut table = le32(entries.len() as i64).to_vec();
    for offset in offsets {
        table.extend(le32(offset));
    }
    table.extend(blobs);
    table
}

/// `pane(magic, name)`: `material_index` left to its default, 0.
pub(crate) fn pane(magic: &[u8], name: &str) -> Vec<u8> {
    pane_with(magic, name, 0)
}

pub(crate) fn pane_with(magic: &[u8], name: &str, material_index: i64) -> Vec<u8> {
    let mut body = vec![0; 4];
    body.extend(ljust(name, 16));
    body.extend(vec![0; 0x4C - 8 - 4 - 16]);
    if magic == b"pic1" {
        body.extend(vec![0; 16]);
        body.extend(le16(material_index));
        body.extend(le16(0));
    }
    section(magic, &body)
}

/// A wnd1 section drawn with these materials: its content's, then each frame's.
pub(crate) fn window(name: &str, content: i64, frames: &[i64]) -> Vec<u8> {
    let mut head = vec![0; 4];
    head.extend(ljust(name, 16));
    head.extend(vec![0; 0x4C - 8 - 4 - 16]);
    let table = 0x68 + 0x14;
    let mut body = head;
    body.extend(vec![0; 16]);
    body.extend([frames.len() as u8, 0, 0, 0]);
    body.extend(le32(0x68));
    body.extend(le32(table));
    body.extend(vec![0; 16]);
    body.extend(le16(content));
    body.extend(le16(0));
    for k in 0..frames.len() as i64 {
        body.extend(le32(table + 4 * frames.len() as i64 + 4 * k));
    }
    for &material in frames {
        body.extend(le16(material));
        body.extend([0, 0]);
    }
    section(b"wnd1", &body)
}

pub(crate) fn layout_file(sections: &[Vec<u8>]) -> Vec<u8> {
    let body = sections.concat();
    let mut file = b"CLYT".to_vec();
    file.extend(le16(0xFEFF));
    file.extend(le16(0x14));
    file.extend(le32(0x07020000));
    file.extend(le32(0x14 + body.len() as i64));
    file.extend(le16(sections.len() as i64));
    file.extend(le16(0));
    file.extend(body);
    file
}

pub(crate) fn command_menu() -> Vec<u8> {
    layout_file(&[
        section(
            b"txl1",
            &names_table(&[
                "bar.bclim",
                "fight_button.bclim",
                "fight_text.bclim",
                "bag_button.bclim",
            ]),
        ),
        section(
            b"mat1",
            &materials(&[
                material("16", &[1]),
                material("2q", &[2]),
                material("v", &[3]),
            ]),
        ),
        pane(b"pan1", "RootPane"),
        pane(b"pan1", "23"),
        pane(b"pan1", "2j"),
        pane(b"bnd1", "z"),
        pane_with(b"pic1", "16", 0),
        pane_with(b"pic1", "2q", 1),
        pane(b"pan1", "20"),
        pane_with(b"pic1", "v", 2),
    ])
}

#[test]
fn test_a_layout_file_names_its_panes_in_order_and_its_pictures_textures() {
    let parsed = parse(&command_menu());
    assert_eq!(
        parsed,
        Some(LayoutFile::without_frames(
            strings(&["RootPane", "23", "2j", "z", "16", "2q", "20", "v"]),
            drawn(&[
                ("16", &["fight_button.bclim"]),
                ("2q", &["fight_text.bclim"]),
                ("v", &["bag_button.bclim"]),
            ]),
        ))
    );
}

/// The main menu's cursor is a window whose frames are drawn with `cursor_lt.bclim`
/// (spike 96); its content has no texture of its own.
#[test]
fn test_a_window_is_known_by_the_textures_of_its_content_and_frames() {
    let parsed = parse(&layout_file(&[
        section(
            b"txl1",
            &names_table(&["cursor_lt.bclim", "msgwin_c.bclim"]),
        ),
        section(
            b"mat1",
            &materials(&[
                material("1uC", &[]),
                material("1uLT", &[0]),
                material("2yC", &[1]),
            ]),
        ),
        pane(b"pan1", "RootPane"),
        window("1u", 0, &[1, 1]),
        window("2y", 2, &[]),
    ]));
    let parsed = parsed.expect("a layout file");
    assert_eq!(
        parsed.frames,
        drawn(&[("1u", &["cursor_lt.bclim"]), ("2y", &["msgwin_c.bclim"])])
    );
}

#[test]
fn test_a_window_too_short_to_say_what_draws_it_leaves_the_rest_of_the_file_whole() {
    let parsed = parse(&layout_file(&[
        section(b"txl1", &names_table(&["fight_button.bclim"])),
        section(b"mat1", &materials(&[material("16", &[0])])),
        pane(b"pan1", "RootPane"),
        pane(b"wnd1", "w"),
        pane_with(b"pic1", "16", 0),
    ]));
    let parsed = parsed.expect("a layout file");
    assert_eq!(parsed.textures, drawn(&[("16", &["fight_button.bclim"])]));
    assert_eq!(parsed.frames, drawn(&[("w", &[])]));
}

#[test]
fn test_the_header_gives_the_files_size() {
    let blob = command_menu();
    assert_eq!(size_of(&blob[..0x14]), Some(blob.len() as i64));
}

#[test]
fn test_anything_else_is_not_a_layout_file() {
    let blob = command_menu();
    assert_eq!(size_of(&[b"CLYX", &blob[4..0x14]].concat()), None);
    assert_eq!(
        size_of(&[&blob[..4], b"\xfe\xff", &blob[6..0x14]].concat()),
        None
    );
    assert_eq!(parse(&blob[..blob.len() - 8]), None);
}

#[test]
fn test_layout_files_are_found_in_swept_pages_by_their_headers() {
    let blob = command_menu();
    let mut page = vec![0u8; 0x1000];
    page[0x230..0x230 + blob.len()].copy_from_slice(&blob);
    page[0x800..0x804].copy_from_slice(b"CLYT");
    assert_eq!(
        find(&IndexMap::from([(0x14500000, page)])),
        vec![0x14500230]
    );
}

#[test]
fn test_a_layout_is_matched_to_its_file_by_the_names_of_its_panes() {
    let memory = FakeMemory::new();
    memory.write(0x145EFEB0, &archive(&[command_menu()]));
    let files = LayoutFiles::new(memory);
    files.add_archive(0x145EFEB0).unwrap();
    files.add_archive(0x145EFEB0).unwrap();
    let built = strings(&["RootPane", "23", "2j", "z", "16", "2q", "20", "v"]);
    let found = files.matching(&built);
    assert!(
        found
            .as_ref()
            .is_some_and(|found| found.textures["v"] == ["bag_button.bclim"])
    );
    assert!(files.matching(&built[..built.len() - 1]).is_none());
}

#[test]
fn test_a_textures_name_is_said_as_words() {
    assert_eq!(words("fight_button.bclim"), "fight");
    assert_eq!(words("pokemon_button.bclim"), "pokémon");
    assert_eq!(words("run_button"), "run");
    assert_eq!(words("waza_plate_00.bclim"), "plate 00");
    assert_eq!(words("button.bclim"), "button");
    assert_eq!(words("return_btn.bclim"), "return");
    assert_eq!(words("camera_btn_mini.bclim"), "camera");
}

/// A `darc` holding these files one after another, as a work's resource points at (spike 64).
pub(crate) fn archive(files: &[Vec<u8>]) -> Vec<u8> {
    let body = files.concat();
    let size = 0x1C + body.len() as i64;
    let mut darc = b"darc".to_vec();
    darc.extend(le16(0xFEFF));
    darc.extend(le16(0x1C));
    darc.extend(le32(0x01000000));
    darc.extend(le32(size));
    darc.extend(le32(0x1C));
    darc.extend(le32(0));
    darc.extend(le32(0x1C));
    darc.extend(body);
    darc
}

#[test]
fn test_the_layout_files_in_a_works_archive_are_read_without_a_sweep() {
    let memory = FakeMemory::new();
    memory.write(0x14BACA80, &archive(&[vec![0; 0x40], command_menu()]));
    let files = LayoutFiles::new(memory);
    files.add_archive(0x14BACA80).unwrap();
    let found = files.matching(&strings(&[
        "RootPane", "23", "2j", "z", "16", "2q", "20", "v",
    ]));
    assert!(
        found
            .as_ref()
            .is_some_and(|found| found.textures["16"] == ["fight_button.bclim"])
    );
}

/// The summary's move panel is known by its file, `waza.bclyt`, to put it away while an Egg
/// is shown (or_egg_summary).
#[test]
fn test_a_layout_file_read_from_an_archive_has_its_name() {
    let memory = FakeMemory::new();
    memory.write(0x14BACA80, &darc(&[("command.bclyt", command_menu())]));
    let files = LayoutFiles::new(memory);
    files.add_archive(0x14BACA80).unwrap();
    let found = files.matching(&strings(&[
        "RootPane", "23", "2j", "z", "16", "2q", "20", "v",
    ]));
    assert!(
        found
            .as_ref()
            .is_some_and(|found| found.name == "command.bclyt")
    );
}

#[test]
fn test_something_that_is_not_an_archive_gives_no_files() {
    let memory = FakeMemory::new();
    memory.write(
        0x14BACA80,
        &[b"darx", &archive(&[command_menu()])[4..]].concat(),
    );
    let files = LayoutFiles::new(memory);
    files.add_archive(0x14BACA80).unwrap();
    assert!(
        files
            .matching(&strings(&[
                "RootPane", "23", "2j", "z", "16", "2q", "20", "v"
            ]))
            .is_none()
    );
}

/// The keyboard carries its cursor from key to key, so its panes are walked in a different
/// order from the one its file lists (opening.ctm, 2026-09-18).
#[test]
fn test_a_file_is_found_when_a_layout_has_moved_a_pane() {
    let memory = FakeMemory::new();
    memory.write(
        0x14C00000,
        &archive(&[layout_file(&[
            section(b"txl1", &names_table(&["ok_button.bclim"])),
            section(b"mat1", &materials(&[material("k", &[0])])),
            pane(b"pan1", "RootPane"),
            pane_with(b"pic1", "k", 0),
            pane(b"bnd1", "b"),
            pane_with(b"pic1", "cur", 0),
        ])]),
    );
    let files = LayoutFiles::new(memory);
    files.add_archive(0x14C00000).unwrap();
    let moved = files.matching(&strings(&["RootPane", "k", "cur", "b"]));
    let moved = moved.expect("a file whose panes are the same set");
    assert_eq!(moved.textures["k"], ["ok_button.bclim"]);
}

#[test]
fn test_two_files_of_the_same_panes_in_another_order_match_neither() {
    let memory = FakeMemory::new();
    memory.write(
        0x14C00000,
        &archive(&[
            layout_file(&[
                section(b"txl1", &names_table(&["a.bclim"])),
                section(b"mat1", &materials(&[material("k", &[0])])),
                pane(b"pan1", "RootPane"),
                pane_with(b"pic1", "k", 0),
                pane(b"bnd1", "b"),
            ]),
            layout_file(&[
                section(b"txl1", &names_table(&["b.bclim"])),
                section(b"mat1", &materials(&[material("k", &[0])])),
                pane(b"pan1", "RootPane"),
                pane(b"bnd1", "b"),
                pane_with(b"pic1", "k", 0),
            ]),
        ]),
    );
    let files = LayoutFiles::new(memory);
    files.add_archive(0x14C00000).unwrap();
    assert!(files.matching(&strings(&["RootPane", "k", "b"])).is_some());
    assert!(files.matching(&strings(&["k", "b", "RootPane"])).is_none());
}

/// The name entry draws its Shift key, an arrow up, with `icon_delete.bclim`, and the number
/// entry its Delete key: "delete" named the Shift key (or_name_rater).
#[test]
fn test_a_picture_is_said_by_the_layout_it_is_drawn_in_first() {
    assert_eq!(
        words_in("icon_delete.bclim", "nameinput2_lower.bclyt"),
        "shift"
    );
    assert_eq!(
        words_in("icon_delete.bclim", "number_input_lower.bclyt"),
        "delete"
    );
    assert_eq!(words("icon_delete.bclim"), "delete");
}

#[test]
fn test_a_texture_called_an_icon_is_said_without_the_word() {
    assert_eq!(words("icon_return.bclim"), "return");
    assert_eq!(words("return_icon.bclim"), "return");
    assert_eq!(
        (words("map_big.bclim"), words("map_small.bclim")),
        ("zoom in".to_string(), "zoom out".to_string())
    );
    // The DexNav's crowns, "complete2" by its file's name (movie28).
    assert_eq!(words("complete2.bclim"), "gold crown");
    assert_eq!(words("icon.bclim"), "icon");
    // A move's and a contest's: the move-forget screen's.
    assert_eq!(words("waza_con_change.bclim"), "change");
}

#[test]
fn test_super_trainings_buttons_are_said_by_what_they_do() {
    assert_eq!(words("list_button.bclim"), "training bags");
    // A regimen's Balloon Bot has no words of its own: the game's title for it is the word.
    assert_eq!(words("stage_poke01.bclim"), "stage poke01");
    // The same button once the regimen is cleared (2026-10-10).
    assert_eq!(words("stage_icon00_rock.bclim"), "start");
}

/// Each move's button on the battle's move info screen is a `waza_pos` picture the panel
/// above names as it is chosen; by its name it was "pos 00", all four (or_move-learning).
#[test]
fn test_the_battles_move_info_buttons_are_nameless() {
    assert_eq!(words("waza_pos_00.bclim"), "");
    assert_eq!(words("waza_pos_03.bclim"), "");
}
