//! Updating pa3ds.exe from GitHub's latest release, on the player's say: offered as pa3ds starts
//! (`offer`), and whenever the player asks from pa3ds's window (`check`).
//!
//! The port's own, with no Python counterpart: pa3ds ships as this binary (upstream ADR-0033 as
//! amended), and the player's files survive an update because they are in `user\` beside the exe
//! (ADR-0032). The prompt is a plain message box, which is what a screen reader reads; the
//! network and the unzip are Windows' own `curl.exe` and `tar.exe`. On macOS and Linux, as an
//! experiment, the prompt is the system's own dialog (`dialog::native`) and the network and the
//! unpacking the system's `curl` and `tar`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IDYES, MB_ICONINFORMATION, MB_ICONQUESTION, MB_ICONWARNING, MB_OK, MB_YESNO,
};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{MB_SETFOREGROUND, MB_TOPMOST, MessageBoxW};

// A test build points this at a local server.
const API: &str = match option_env!("PA3DS_UPDATE_API") {
    Some(url) => url,
    None => "https://api.github.com/repos/pa3ds/rust-pokemon-access-3ds/releases/latest",
};

const TITLE: &str = "pa3ds update";

/// The end of the name of this platform's asset in a release, the system it names and the
/// program in it.
#[cfg(windows)]
const ASSET: (&str, &str, &str) = ("-windows.zip", "Windows", "pa3ds.exe");
#[cfg(target_os = "macos")]
const ASSET: (&str, &str, &str) = ("-macos.zip", "macOS", "pa3ds");
#[cfg(all(not(windows), not(target_os = "macos")))]
const ASSET: (&str, &str, &str) = ("-linux.tar.gz", "Linux", "pa3ds");
const EXE: &str = ASSET.2;

#[derive(Debug, PartialEq)]
pub struct Release {
    pub version: String,
    pub zip: String,
}

/// The release in GitHub's "latest release" JSON: its tag `vX.Y.Z` and the asset ending in
/// `asset`, this platform's `ASSET`.
pub fn parse(json: &str, asset: &str) -> Option<Release> {
    let tag = Regex::new(r#""tag_name"\s*:\s*"v([^"]+)""#).unwrap();
    let url = format!(
        r#""browser_download_url"\s*:\s*"([^"]+{})""#,
        regex::escape(asset)
    );
    let zip = Regex::new(&url).unwrap();
    Some(Release {
        version: tag.captures(json)?[1].to_string(),
        zip: zip.captures(json)?[1].to_string(),
    })
}

fn triple(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.').map(|part| part.parse::<u64>().ok());
    let found = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(found)
}

/// Whether `latest` is a later `X.Y.Z` than `current`. Anything else is not.
pub fn newer(current: &str, latest: &str) -> bool {
    match (triple(current), triple(latest)) {
        (Some(current), Some(latest)) => latest > current,
        _ => false,
    }
}

/// Offers the player the latest release when this is a release build and a newer one is out.
/// `Some(code)` when it updated and ran the new pa3ds, which exited with `code`; `None` to carry
/// on with this one.
pub fn offer(argv: &[String]) -> Option<i32> {
    option_env!("PA3DS_RELEASE")?;
    // An agent session runs --silent, and a modal dialog would hang it.
    if argv
        .iter()
        .any(|arg| matches!(arg.as_str(), "--silent" | "-h" | "--help"))
    {
        return None;
    }
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let _ = fs::remove_file(dir.join(format!("{EXE}.old")));

    let release = match latest() {
        Ok(release) => release,
        Err(error) => {
            println!("could not check for a pa3ds update: {error}");
            return None;
        }
    };
    if !newer(CURRENT, &release.version) || !install(&exe, dir, &release) {
        return None;
    }
    match Command::new(dir.join(EXE)).args(argv).status() {
        Ok(status) => Some(status.code().unwrap_or(1)),
        Err(error) => {
            println!("could not start the updated pa3ds: {error}");
            Some(1)
        }
    }
}

/// Checks for a newer release on the player's say, from pa3ds's window, and says what it found:
/// that this is the latest, or the question `offer` asks. Once updated, the new pa3ds starts in a
/// console of its own, where it is alone and opens its window, and replaces this one (upstream
/// ADR-0039).
pub fn check() {
    if option_env!("PA3DS_RELEASE").is_none() {
        message(
            &format!("This pa3ds, {CURRENT}, is a test build, which does not update itself."),
            MB_OK | MB_ICONINFORMATION,
        );
        return;
    }
    let Some((exe, dir)) = std::env::current_exe()
        .ok()
        .and_then(|exe| Some((exe.clone(), exe.parent()?.to_path_buf())))
    else {
        return;
    };
    let release = match latest() {
        Ok(release) => release,
        Err(error) => {
            message(
                &format!("pa3ds could not check for an update: {error}"),
                MB_OK | MB_ICONWARNING,
            );
            return;
        }
    };
    if !newer(CURRENT, &release.version) {
        message(
            &format!("You have the latest pa3ds, {CURRENT}."),
            MB_OK | MB_ICONINFORMATION,
        );
        return;
    }
    if !install(&exe, &dir, &release) {
        return;
    }
    let argv: Vec<String> = std::env::args().skip(1).collect();
    #[cfg(windows)]
    let started = {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_CONSOLE: u32 = 0x10;
        Command::new(dir.join(EXE))
            .args(&argv)
            .creation_flags(CREATE_NEW_CONSOLE)
            .spawn()
    };
    #[cfg(not(windows))]
    let started = Command::new(dir.join(EXE)).args(&argv).spawn();
    if let Err(error) = started {
        message(
            &format!("pa3ds is updated, but could not start again: {error}"),
            MB_OK | MB_ICONWARNING,
        );
    }
}

const CURRENT: &str = env!("CARGO_PKG_VERSION");

/// Asks the player whether to update to `release`, and updates if they say yes: whether pa3ds is
/// now `release`'s. A failed update is said, and pa3ds carries on as it is.
fn install(exe: &Path, dir: &Path, release: &Release) -> bool {
    let question = format!(
        "pa3ds {} is out. You have {CURRENT}.\n\nUpdate now? pa3ds downloads it, replaces \
         itself and starts again. Everything in your user folder is kept.",
        release.version
    );
    if message(&question, MB_YESNO | MB_ICONQUESTION) != IDYES {
        return false;
    }
    if let Err(error) = update(exe, dir, release) {
        message(
            &format!(
                "The update to pa3ds {} failed: {error}\n\npa3ds carries on as {CURRENT}.",
                release.version
            ),
            MB_OK | MB_ICONWARNING,
        );
        return false;
    }
    true
}

fn latest() -> Result<Release, String> {
    let json = run(Command::new(curl()).args([
        "-fsSL",
        "--connect-timeout",
        "5",
        "--max-time",
        "15",
        "-H",
        "Accept: application/vnd.github+json",
        "-H",
        "User-Agent: pa3ds",
        API,
    ]))?;
    parse(&String::from_utf8_lossy(&json), ASSET.0)
        .ok_or_else(|| format!("no {} release found", ASSET.1))
}

/// Replaces the files of `dir` with the release's. A running exe can be renamed but not
/// overwritten, so it steps aside as `pa3ds.exe.old` (deleted at the next start), or on macOS and
/// Linux `pa3ds` as `pa3ds.old`. The `data`
/// folder is pa3ds's and replaced whole, so a file the release has dropped goes with it (upstream
/// ADR-0040): it steps aside as `data.old` until the release is unpacked.
fn update(exe: &Path, dir: &Path, release: &Release) -> Result<(), String> {
    let zip = std::env::temp_dir().join(format!("pa3ds-{}{}", release.version, ASSET.0));
    let result = swap(exe, dir, &release.zip, &zip);
    let _ = fs::remove_file(&zip);
    result
}

fn swap(exe: &Path, dir: &Path, url: &str, zip: &Path) -> Result<(), String> {
    run(Command::new(curl())
        .args(["-fsSL", "--connect-timeout", "5", "-o"])
        .arg(zip)
        .arg(url))?;
    let old = dir.join(format!("{EXE}.old"));
    fs::rename(exe, &old).map_err(|error| format!("could not move {EXE} aside: {error}"))?;
    let (data, old_data) = (dir.join("data"), dir.join("data.old"));
    let _ = fs::remove_dir_all(&old_data);
    let moved = fs::rename(&data, &old_data).is_ok();
    // The zip holds one folder, pa3ds-<version>-windows, around the files, and each platform's
    // package its own. macOS's tar reads a zip, and Linux's a .tar.gz.
    let unpacked = run(Command::new(tar())
        .arg("-xf")
        .arg(zip)
        .arg("-C")
        .arg(dir)
        .args(["--strip-components", "1"]));
    let new = dir.join(EXE);
    #[cfg(not(windows))]
    {
        // An unpacked file need not keep its executable bit, and pa3ds without it cannot start.
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&new, fs::Permissions::from_mode(0o755));
    }
    if unpacked.is_err() && !new.exists() {
        let _ = fs::rename(&old, exe);
        if moved {
            let _ = fs::remove_dir_all(&data);
            let _ = fs::rename(&old_data, &data);
        }
    } else {
        let _ = fs::remove_dir_all(&old_data);
    }
    unpacked.map(drop)
}

/// `command`'s output, or what it said when it failed.
fn run(command: &mut Command) -> Result<Vec<u8>, String> {
    let output = command.output().map_err(|error| error.to_string())?;
    if output.status.success() {
        return Ok(output.stdout);
    }
    let said = String::from_utf8_lossy(&output.stderr);
    Err(match said.trim() {
        "" => output.status.to_string(),
        said => said.to_string(),
    })
}

#[cfg(windows)]
fn system32(program: &str) -> PathBuf {
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    Path::new(&root).join("System32").join(program)
}

#[cfg(windows)]
fn curl() -> PathBuf {
    system32("curl.exe")
}

#[cfg(windows)]
fn tar() -> PathBuf {
    system32("tar.exe")
}

#[cfg(not(windows))]
fn curl() -> PathBuf {
    PathBuf::from("/usr/bin/curl")
}

#[cfg(not(windows))]
fn tar() -> PathBuf {
    PathBuf::from("/usr/bin/tar")
}

#[cfg(windows)]
fn message(text: &str, style: u32) -> i32 {
    let wide = |text: &str| -> Vec<u16> { text.encode_utf16().chain(Some(0)).collect() };
    // SAFETY: both strings are NUL-terminated and outlive the call.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            wide(text).as_ptr(),
            wide(TITLE).as_ptr(),
            style | MB_SETFOREGROUND | MB_TOPMOST,
        )
    }
}

/// `message` on macOS and Linux: the system's own dialog, a yes-or-no question where `style`
/// asks one. Where no dialog can open, the text goes to the terminal pa3ds runs in, and the
/// question is answered no.
#[cfg(not(windows))]
fn message(text: &str, style: u32) -> i32 {
    use crate::ux::dialog::native;
    let shown = match style & 0xF == MB_YESNO {
        true => native::confirm(TITLE, text).map(|yes| if yes { IDYES } else { 0 }),
        false => native::tell(TITLE, text).map(|()| 0),
    };
    shown.unwrap_or_else(|error| {
        println!("{text}");
        println!("(no dialog could open: {error})");
        0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LATEST: &str = r#"{"url":"https://api.github.com/repos/pa3ds/rust-pokemon-access-3ds/releases/1",
        "tag_name":"v0.10.2","name":"pa3ds 0.10.2","assets":[
        {"name":"pa3ds-0.10.2-macos.zip","browser_download_url":"https://github.com/pa3ds/rust-pokemon-access-3ds/releases/download/v0.10.2/pa3ds-0.10.2-macos.zip"},
        {"name":"pa3ds-0.10.2-linux.tar.gz","browser_download_url":"https://github.com/pa3ds/rust-pokemon-access-3ds/releases/download/v0.10.2/pa3ds-0.10.2-linux.tar.gz"},
        {"name":"pa3ds-0.10.2-windows.zip","browser_download_url":"https://github.com/pa3ds/rust-pokemon-access-3ds/releases/download/v0.10.2/pa3ds-0.10.2-windows.zip"}]}"#;

    #[test]
    fn parse_reads_the_version_and_the_windows_zip() {
        assert_eq!(
            parse(LATEST, "-windows.zip"),
            Some(Release {
                version: "0.10.2".to_string(),
                zip: "https://github.com/pa3ds/rust-pokemon-access-3ds/releases/download/v0.10.2/pa3ds-0.10.2-windows.zip".to_string(),
            })
        );
    }

    #[test]
    fn parse_reads_the_macos_zip() {
        assert_eq!(
            parse(LATEST, "-macos.zip").map(|release| release.zip),
            Some("https://github.com/pa3ds/rust-pokemon-access-3ds/releases/download/v0.10.2/pa3ds-0.10.2-macos.zip".to_string())
        );
    }

    #[test]
    fn parse_reads_the_linux_tarball() {
        assert_eq!(
            parse(LATEST, "-linux.tar.gz").map(|release| release.zip),
            Some("https://github.com/pa3ds/rust-pokemon-access-3ds/releases/download/v0.10.2/pa3ds-0.10.2-linux.tar.gz".to_string())
        );
    }

    #[test]
    fn parse_finds_nothing_without_a_windows_zip() {
        let json = r#"{"tag_name":"v1.0.0","assets":[{"browser_download_url":"https://x/pa3ds-1.0.0-linux.tar.gz"}]}"#;
        assert_eq!(parse(json, "-windows.zip"), None);
    }

    #[test]
    fn newer_compares_numbers_not_text() {
        assert!(newer("0.9.0", "0.10.0"));
    }

    #[test]
    fn newer_is_false_for_the_same_version() {
        assert!(!newer("1.2.3", "1.2.3"));
    }

    #[test]
    fn newer_is_false_for_an_older_version() {
        assert!(!newer("1.2.3", "1.2.2"));
    }

    #[test]
    fn newer_is_false_for_what_is_not_a_version() {
        assert!(!newer("1.2.3", "garbage"));
        assert!(!newer("garbage", "1.2.3"));
        assert!(!newer("1.2.3", "1.2.4.5"));
    }
}
