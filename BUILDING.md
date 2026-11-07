# Building pa3ds

This is a port of [the Python pa3ds](https://github.com/pa3ds/pokemon-access-3ds) to Rust, so
that a player runs one `pa3ds.exe` with nothing to install. The Python repository is where
features are built first; this one follows it module for module (`port.toml` names the commit it
matches) and is checked against it line for line on recorded replays (`tools/tape/`). The
decisions, the evidence behind them and the vocabulary are the Python repository's, in its
`docs/` and `GLOSSARY.md`.

```
cargo build --release
```

Building needs Rust's MSVC toolchain (rustup's default on Windows), Visual Studio 2022 or newer
with its C and C++ compiler and ATL (the Build Tools are enough), and CMake 4.2 or newer on the
path, since Prism is compiled into pa3ds; the first build takes about a minute longer for it.
Visual Studio's own CMake is 3.31, which is too old.
`target\release\pa3ds.exe` is the whole program. To pack it with its data, its manual and its
licences into one zip a player unpacks and runs:

```
powershell -ExecutionPolicy Bypass -File tools\package.ps1
```

(or `tools\package.bat`, which runs the same), which runs the tests, builds the release, checks it
starts, and writes `target\package\pa3ds-<version>-windows.zip`. Only a pa3ds built this way
checks for updates; `cargo build` makes one that does not.

## macOS and Linux

An experiment, built and tested on every push to an `os/...` branch
(`.github/workflows/unix.yml`). Building needs Rust, a C and C++ compiler (Xcode's command line
tools on macOS) and CMake 4.2 or newer on the path (`brew install cmake`, or `pipx install
"cmake>=4.2"`); on Linux also ALSA's and speech-dispatcher's headers and pkg-config, on Debian and
Ubuntu `sudo apt-get install libasound2-dev libspeechd-dev pkg-config`. `cargo build --release`
makes `target/release/pa3ds`. Set `MACOSX_DEPLOYMENT_TARGET=13.0`, as CI does, for a Mac binary
that runs on macOS 13 and later.

```
sh tools/package.sh
```

runs the tests, builds the release, checks it starts, and writes
`target/package/pa3ds-<version>-macos.zip` (signed ad hoc, not notarised) or
`target/package/pa3ds-<version>-linux.tar.gz`, with the same contents as the Windows zip. A tag's
release workflow builds both after the Windows zip and adds them to the release.

A release is cut with `/release` in Claude Code (`.claude/skills/release/SKILL.md`): it writes what
changed for players into `CHANGELOG.md`, sets the version, and tags it. Pushing a `v<version>` tag
builds the zip on GitHub and publishes it as that version's release
(`.github/workflows/release.yml`).
