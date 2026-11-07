//! What prism-sys's static build leaves out of the link, off Windows.
//!
//! macOS: prism-sys links libc++ but not what its AVSpeech and VoiceOver backends call, the Apple
//! frameworks and clang's runtime (`@available` is `__isPlatformVersionAtLeast` there), so pa3ds
//! links them itself, and CoreGraphics for its keys' event tap.
//!
//! Linux: Prism needs GCC 13's libstdc++, which Ubuntu 22.04 and Debian 12 do not have, so it is
//! linked in whole and the binary asks only for glibc. prism-sys asks for `-lstdc++` by name, and
//! the linker takes the first folder that has either library, so a folder holding the static one
//! alone is put ahead of the compiler's own.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("macos") => macos(),
        Ok("linux") => linux(),
        _ => {}
    }
}

fn macos() {
    println!("cargo:rustc-link-lib=objc");
    for framework in [
        "Foundation",
        "CoreFoundation",
        "CoreGraphics",
        "AppKit",
        "AVFoundation",
        "IOKit",
    ] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
    let resources = Command::new("xcrun")
        .args(["clang", "--print-resource-dir"])
        .output()
        .expect("xcrun clang, from the Xcode command line tools");
    let resources = String::from_utf8_lossy(&resources.stdout)
        .trim()
        .to_string();
    println!("cargo:rustc-link-search=native={resources}/lib/darwin");
    println!("cargo:rustc-link-lib=static=clang_rt.osx");
}

fn linux() {
    println!("cargo:rerun-if-env-changed=CXX");
    let compiler = std::env::var("CXX").unwrap_or_else(|_| "c++".to_string());
    let Ok(found) = Command::new(&compiler)
        .arg("-print-file-name=libstdc++.a")
        .output()
    else {
        return;
    };
    let archive = PathBuf::from(String::from_utf8_lossy(&found.stdout).trim());
    // A compiler without the static library prints the bare name back.
    if !archive.is_absolute() || !archive.is_file() {
        return;
    }
    let folder = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("libstdc++");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::copy(&archive, folder.join("libstdc++.a")).unwrap();
    println!("cargo:rustc-link-search=native={}", folder.display());
}
