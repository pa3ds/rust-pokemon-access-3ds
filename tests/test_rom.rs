//! The player's ROM: reading files out of it, and finding it among the ones Azahar has opened.

use std::path::{Path, PathBuf};

use super::fake_rom::{TmpDir, lz11_literals, rom_image, rom_image_named, rom_image_with};
use crate::core::rom::{self, Rom};
use crate::py::{self, Kind};

const OMEGA_RUBY: i64 = 0x000400000011C400;
const ALPHA_SAPPHIRE: i64 = 0x000400000011C500;

fn write(folder: &Path, name: &str, image: &[u8]) -> PathBuf {
    let path = folder.join(name);
    std::fs::write(&path, image).unwrap();
    path
}

#[test]
fn test_files_are_read_by_their_path() {
    let tmp_path = TmpDir::new();
    let files: Vec<(&str, &[u8])> = vec![
        ("a/0/7/3", b"english"),
        ("a/0/1/3", b"zones"),
        ("a/0/7/1", b"japanese"),
        ("top", b"t"),
    ];
    let opened = Rom::new(&write(
        tmp_path.path(),
        "or.3ds",
        &rom_image(&files, OMEGA_RUBY),
    ))
    .unwrap();
    assert_eq!(opened.title_id, OMEGA_RUBY);
    let mut sorted: Vec<String> = files.iter().map(|(name, _)| name.to_string()).collect();
    sorted.sort();
    assert_eq!(opened.paths(), sorted);
    for (name, data) in &files {
        assert_eq!(opened.read(name).unwrap(), *data);
    }
    assert!(opened.read("a/9/9/9").unwrap_err().is(Kind::RomError));
}

/// ADR-0038: the title screen's logo is said as the game's own name, out of the ROM.
#[test]
fn test_the_game_names_itself_in_its_smdh_in_the_players_language() {
    let tmp_path = TmpDir::new();
    let image = rom_image_named(
        &[("top", b"t")],
        OMEGA_RUBY,
        &[
            "ポケットモンスター オメガルビー",
            "Pokémon Omega Ruby",
            "Pokémon Rubis Oméga",
        ],
    );
    let opened = Rom::new(&write(tmp_path.path(), "or.3ds", &image)).unwrap();
    assert_eq!(
        opened.title("fr").unwrap().as_deref(),
        Some("Pokémon Rubis Oméga")
    );
    assert_eq!(
        opened.title("ko").unwrap().as_deref(),
        Some("Pokémon Omega Ruby")
    );
    let bare = Rom::new(&write(
        tmp_path.path(),
        "bare.3ds",
        &rom_image(&[("top", b"t")], OMEGA_RUBY),
    ))
    .unwrap();
    assert_eq!(bare.title("en").unwrap(), None);
}

/// The player's Alpha Sapphire on this machine is one: pa3ds needs keys to read it, and will
/// never ship them (ADR-0008).
#[test]
fn test_an_encrypted_image_is_turned_away_with_a_reason() {
    let tmp_path = TmpDir::new();
    let path = write(
        tmp_path.path(),
        "sapphire.3ds",
        &rom_image_with(&[("a", b"x")], ALPHA_SAPPHIRE, true),
    );
    let error = Rom::new(&path).unwrap_err();
    assert!(error.is(Kind::RomError) && error.message.contains("encrypted"));
    assert_eq!(rom::title_id(&path).unwrap(), None);
}

#[test]
fn test_what_is_not_a_rom_has_no_title() {
    let tmp_path = TmpDir::new();
    assert_eq!(
        rom::title_id(&write(tmp_path.path(), "notes.3ds", b"not a rom at all")).unwrap(),
        None
    );
    assert_eq!(
        rom::title_id(&tmp_path.path().join("gone.3ds")).unwrap(),
        None
    );
    let error = Rom::new(&write(tmp_path.path(), "short.3ds", &[0; 0x400])).unwrap_err();
    assert!(error.is(Kind::RomError));
}

/// Azahar remembers every ROM it has opened: other games, and encrypted copies of this one.
/// The title id is what says which is being played, and it is checked, not assumed.
#[test]
fn test_the_rom_found_is_the_decrypted_one_of_the_game_running() {
    let tmp_path = TmpDir::new();
    let other = write(
        tmp_path.path(),
        "x.3ds",
        &rom_image(&[("a", b"x")], 0x0004000000055D00),
    );
    let locked = write(
        tmp_path.path(),
        "or-locked.3ds",
        &rom_image_with(&[("a", b"x")], OMEGA_RUBY, true),
    );
    let wanted = write(
        tmp_path.path(),
        "or.3ds",
        &rom_image(&[("a", b"x")], OMEGA_RUBY),
    );
    let installer = write(
        tmp_path.path(),
        "or.cia",
        &rom_image(&[("a", b"x")], OMEGA_RUBY),
    );
    assert_eq!(
        rom::find(
            OMEGA_RUBY,
            vec![
                other.clone(),
                installer.clone(),
                locked.clone(),
                wanted.clone()
            ],
            None
        )
        .unwrap(),
        Some(wanted.clone())
    );
    assert_eq!(
        rom::find(
            ALPHA_SAPPHIRE,
            vec![other.clone(), locked.clone(), wanted],
            None
        )
        .unwrap(),
        None
    );
    let mut notes: Vec<String> = Vec::new();
    let gone = tmp_path.path().join("gone.3ds");
    assert_eq!(
        rom::find(
            ALPHA_SAPPHIRE,
            vec![
                installer.clone(),
                gone.clone(),
                locked.clone(),
                other.clone()
            ],
            Some(&mut notes)
        )
        .unwrap(),
        None
    );
    assert_eq!(
        notes,
        vec![
            format!("{} is not a ROM image", py::path_str(&installer)),
            format!("{} is not there", py::path_str(&gone)),
            format!(
                "{} cannot be read as a decrypted ROM image",
                py::path_str(&locked)
            ),
            format!("{} is title 0004000000055d00", py::path_str(&other)),
        ]
    );
}

/// As Azahar writes it: a list with commas inside quotes, backslashes doubled, a path given
/// twice in two spellings, and the folder it opens ROMs from.
#[test]
fn test_azahar_says_which_roms_it_has_opened() {
    let tmp_path = TmpDir::new();
    let roms = tmp_path.path().join("roms");
    std::fs::create_dir(&roms).unwrap();
    write(&roms, "b.3ds", b"");
    write(&roms, "notes.txt", b"");
    let folder = roms.to_string_lossy().replace('\\', "/");
    let doubled = roms.join("b.3ds").to_string_lossy().replace('\\', "\\\\");
    let config = [
        "[UI]".to_string(),
        format!("Paths\\recentFiles=\"{folder}/a (En,Ja).3ds\", {folder}/b.3ds, {doubled}"),
        format!("Paths\\romsPath={folder}"),
        "Paths\\recentFiles\\default=false".to_string(),
    ]
    .join("\n");
    let found = rom::azahar_roms(&config);
    let names: Vec<String> = found
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["a (En,Ja).3ds", "b.3ds"]);
}

#[test]
fn test_no_config_names_no_rom() {
    assert_eq!(rom::azahar_roms(""), Vec::<PathBuf>::new());
}

#[test]
fn test_lz11_undoes_literals_and_references_back() {
    assert_eq!(
        rom::lz11(&lz11_literals(b"ZO in the clear")).unwrap(),
        b"ZO in the clear"
    );
    // "ab" as literals, then one reference three bytes long, one back: "abbbb".
    let mut packed = vec![0x11, 5, 0, 0, 0b00100000];
    packed.extend(b"ab");
    packed.extend([0x20, 0x00]);
    assert_eq!(rom::lz11(&packed).unwrap(), b"abbbb");
}

/// Spike 193: a field model names its Pokémon in its first 12 KB, and only that is unpacked.
#[test]
fn test_lz11_stops_at_its_limit() {
    assert_eq!(
        rom::lz11_limit(&lz11_literals(b"ZO in the clear"), Some(5)).unwrap(),
        b"ZO in"
    );
    let mut packed = vec![0x11, 5, 0, 0, 0b00100000];
    packed.extend(b"ab");
    packed.extend([0x20, 0x00]);
    assert_eq!(rom::lz11_limit(&packed, Some(3)).unwrap(), b"abb");
    assert_eq!(rom::lz11_limit(&packed, Some(99)).unwrap(), b"abbbb");
}

#[test]
fn test_lz11_that_does_not_hold_together_is_a_rom_error() {
    assert!(
        rom::lz11(b"ZO not compressed")
            .unwrap_err()
            .is(Kind::RomError)
    );
    let mut cut = vec![0x11, 0x10, 0, 0, 0];
    cut.extend(b"ab");
    assert!(rom::lz11(&cut).unwrap_err().is(Kind::RomError));
    assert!(
        rom::lz11(&[0x11, 5, 0, 0, 0b10000000, 0x20, 0x05])
            .unwrap_err()
            .is(Kind::RomError)
    );
}
