//! Updating pa3ds.exe from GitHub's latest release, on the player's say.
//!
//! The port's own, with no Python counterpart: pa3ds ships as this binary (upstream ADR-0033 as
//! amended), and the player's files survive an update because they are in `user\` beside the exe
//! (ADR-0032). The prompt is a plain message box, which is what a screen reader reads; the
//! network and the unzip are Windows' own `curl.exe` and `tar.exe`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IDYES, MB_ICONQUESTION, MB_ICONWARNING, MB_OK, MB_SETFOREGROUND, MB_TOPMOST, MB_YESNO,
    MessageBoxW,
};

// A test build points this at a local server.
const API: &str = match option_env!("PA3DS_UPDATE_API") {
    Some(url) => url,
    None => "https://api.github.com/repos/pa3ds/rust-pokemon-access-3ds/releases/latest",
};

const TITLE: &str = "pa3ds update";

#[derive(Debug, PartialEq)]
pub struct Release {
    pub version: String,
    pub zip: String,
}

/// The release in GitHub's "latest release" JSON: its tag `vX.Y.Z` and the asset ending in
/// `-windows.zip`.
pub fn parse(json: &str) -> Option<Release> {
    let tag = Regex::new(r#""tag_name"\s*:\s*"v([^"]+)""#).unwrap();
    let zip = Regex::new(r#""browser_download_url"\s*:\s*"([^"]+-windows\.zip)""#).unwrap();
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
    let _ = fs::remove_file(dir.join("pa3ds.exe.old"));

    let current = env!("CARGO_PKG_VERSION");
    let release = match latest() {
        Ok(release) => release,
        Err(error) => {
            println!("could not check for a pa3ds update: {error}");
            return None;
        }
    };
    if !newer(current, &release.version) {
        return None;
    }
    let question = format!(
        "pa3ds {} is out. You have {current}.\n\nUpdate now? pa3ds downloads it, replaces \
         itself and starts again. Everything in your user folder is kept.",
        release.version
    );
    if message(&question, MB_YESNO | MB_ICONQUESTION) != IDYES {
        return None;
    }
    if let Err(error) = update(&exe, dir, &release) {
        message(
            &format!(
                "The update to pa3ds {} failed: {error}\n\npa3ds carries on as {current}.",
                release.version
            ),
            MB_OK | MB_ICONWARNING,
        );
        return None;
    }
    match Command::new(dir.join("pa3ds.exe")).args(argv).status() {
        Ok(status) => Some(status.code().unwrap_or(1)),
        Err(error) => {
            println!("could not start the updated pa3ds: {error}");
            Some(1)
        }
    }
}

fn latest() -> Result<Release, String> {
    let json = run(Command::new(system32("curl.exe")).args([
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
    parse(&String::from_utf8_lossy(&json)).ok_or_else(|| "no Windows release found".to_string())
}

/// Replaces the files of `dir` with the release's. A running exe can be renamed but not
/// overwritten, so it steps aside as `pa3ds.exe.old` (deleted at the next start).
fn update(exe: &Path, dir: &Path, release: &Release) -> Result<(), String> {
    let zip = std::env::temp_dir().join(format!("pa3ds-{}-windows.zip", release.version));
    let result = swap(exe, dir, &release.zip, &zip);
    let _ = fs::remove_file(&zip);
    result
}

fn swap(exe: &Path, dir: &Path, url: &str, zip: &Path) -> Result<(), String> {
    run(Command::new(system32("curl.exe"))
        .args(["-fsSL", "--connect-timeout", "5", "-o"])
        .arg(zip)
        .arg(url))?;
    let old = dir.join("pa3ds.exe.old");
    fs::rename(exe, &old).map_err(|error| format!("could not move pa3ds.exe aside: {error}"))?;
    // The zip holds one folder, pa3ds-<version>-windows, around the files.
    let unpacked = run(Command::new(system32("tar.exe"))
        .arg("-xf")
        .arg(zip)
        .arg("-C")
        .arg(dir)
        .args(["--strip-components", "1"]));
    let new = dir.join("pa3ds.exe");
    if unpacked.is_err() && !new.exists() {
        let _ = fs::rename(&old, exe);
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

fn system32(program: &str) -> PathBuf {
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    Path::new(&root).join("System32").join(program)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    const LATEST: &str = r#"{"url":"https://api.github.com/repos/pa3ds/rust-pokemon-access-3ds/releases/1",
        "tag_name":"v0.10.2","name":"pa3ds 0.10.2","assets":[
        {"name":"pa3ds-0.10.2-linux.tar.gz","browser_download_url":"https://github.com/pa3ds/rust-pokemon-access-3ds/releases/download/v0.10.2/pa3ds-0.10.2-linux.tar.gz"},
        {"name":"pa3ds-0.10.2-windows.zip","browser_download_url":"https://github.com/pa3ds/rust-pokemon-access-3ds/releases/download/v0.10.2/pa3ds-0.10.2-windows.zip"}]}"#;

    #[test]
    fn parse_reads_the_version_and_the_windows_zip() {
        assert_eq!(
            parse(LATEST),
            Some(Release {
                version: "0.10.2".to_string(),
                zip: "https://github.com/pa3ds/rust-pokemon-access-3ds/releases/download/v0.10.2/pa3ds-0.10.2-windows.zip".to_string(),
            })
        );
    }

    #[test]
    fn parse_finds_nothing_without_a_windows_zip() {
        let json = r#"{"tag_name":"v1.0.0","assets":[{"browser_download_url":"https://x/pa3ds-1.0.0-linux.tar.gz"}]}"#;
        assert_eq!(parse(json), None);
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
