//! pa3ds's own window (`pa3ds.ux.window`), read back from its template: no test opens it.

use super::test_rename::read_template;
use crate::ux::dialog::ATOM_BUTTON;
use crate::ux::window::{
    IDC_EXIT, IDC_INSTALLER, IDC_ISSUES, IDC_PORTABLE, INSTALLER, PORTABLE, WS_EX_APPWINDOW,
    download, template,
};

#[test]
fn test_the_window_is_pa3dss_by_its_title_with_its_buttons_in_tab_order() {
    let (title, items) = read_template(&template().unwrap());
    assert_eq!(title, "Pokémon Access 3DS");
    assert_eq!(
        items,
        [
            (
                IDC_ISSUES,
                ATOM_BUTTON,
                "Report an issue on GitHub".to_string()
            ),
            (
                IDC_PORTABLE,
                ATOM_BUTTON,
                "Download Azahar (portable)".to_string()
            ),
            (
                IDC_INSTALLER,
                ATOM_BUTTON,
                "Download Azahar (installer)".to_string()
            ),
            (IDC_EXIT, ATOM_BUTTON, "Exit".to_string()),
        ]
    );
}

/// A dialog with no owner is left off the taskbar and out of Alt+Tab unless it says
/// otherwise, and a double-clicked pa3ds has nothing else there to find it by.
#[test]
fn test_the_window_is_on_the_taskbar_though_it_has_no_owner() {
    let built = template().unwrap();
    let ex_style = u32::from_le_bytes(built[4..8].try_into().unwrap()) as i64;
    assert!(ex_style & WS_EX_APPWINDOW != 0);
}

fn release() -> String {
    let mut assets = Vec::new();
    for build in ["libretro-windows-x86_64", "windows-msvc", "windows-msys2"] {
        for end in [".zip", "-installer.exe"] {
            assets.push(format!(
                "{{\"name\": \"azahar-{build}-2126.2{end}\", \"browser_download_url\": \
                 \"https://example.org/azahar-{build}-2126.2{end}\"}}"
            ));
        }
    }
    format!(
        "{{\"tag_name\": \"2126.2\", \"assets\": [{}]}}",
        assets.join(", ")
    )
}

/// Azahar's 2126.2 offered three Windows builds, each a zip and an installer, every name
/// carrying the version (2026-10-24).
#[test]
fn test_each_download_is_the_latest_releases_own_file_of_the_build_made_with_msvc() {
    assert_eq!(
        download(&release(), &PORTABLE).as_deref(),
        Some("https://example.org/azahar-windows-msvc-2126.2.zip")
    );
    assert_eq!(
        download(&release(), &INSTALLER).as_deref(),
        Some("https://example.org/azahar-windows-msvc-2126.2-installer.exe")
    );
    assert_eq!(download("{\"assets\": []}", &PORTABLE), None);
    assert_eq!(download("not json", &PORTABLE), None);
}
