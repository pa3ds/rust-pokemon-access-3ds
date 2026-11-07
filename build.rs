//! macOS only: prism-sys's static build links libc++ but not what its AVSpeech and VoiceOver
//! backends call, the Apple frameworks and clang's runtime (`@available` is
//! `__isPlatformVersionAtLeast` there), so pa3ds links them itself. Nothing on Windows.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    println!("cargo:rustc-link-lib=objc");
    for framework in [
        "Foundation",
        "CoreFoundation",
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
