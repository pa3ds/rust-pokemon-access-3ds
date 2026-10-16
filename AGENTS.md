# pa3ds, in Rust

pa3ds speaks 3DS Pokemon games to blind players: it reads Azahar's emulated memory and sends text
to a screen reader. Two people work on it, Orion and Khai, each on a machine of their own, with
their own movies and saves. Orion is blind and cannot check your work by looking at the game, and
you cannot see the screen either: tests, snapshots, tapes and screenshots are the only shared
evidence.

**This repository is a port.** The Python pa3ds (https://github.com/pa3ds/pokemon-access-3ds)
is upstream: features are found and built there first, with its spikes and movie rig, and arrive
here as a port of the commit `port.toml` names. Each Python file has one Rust file at the same
path (`docs/porting.md`), so an upstream change is a change to the files it maps to.

The Python checkout sits beside this one, at `../pokemon-access-3ds` (or `PA3DS_PYTHON`). Its
glossary and docs are this repository's too, and are not copied here: before the first change in
a session, read its `GLOSSARY.md` in full — the vocabulary, which code and conversation both use
— and the **ADRs** for the area you are working in (`docs/adr/README.md` there is grouped by
area). An ADR is binding: contradict one only by saying so. A path such as `docs/runbooks/rig.md`
in a comment or doc here means the Python checkout's. This repository's only doc is
`docs/porting.md`.

## Talking to upstream

- **What to port** is an open issue in this repository labelled `upstream`, kept up to date by
  `.github/workflows/upstream.yml`: each push to upstream's main rewrites it with
  `sync.py status`, and a push here that brings `port.toml` level with upstream closes it.
- **A bug found in the Python while porting** is ported as it is (below) and filed as an issue in
  the Python repository, labelled `from-port`:
  `gh issue create --repo pa3ds/pokemon-access-3ds --label from-port --title ... --body ...`,
  naming the file, the upstream commit it was found at, and what goes wrong. Upstream fixes it
  there; the fix then arrives here like any other change. Search first
  (`gh issue list --repo pa3ds/pokemon-access-3ds --label from-port --state all`) so a bug is
  filed once.

## Hard constraints

Settled. Reopen one only by asking Orion first.

- **Announce, never act.** pa3ds never presses buttons and never writes to game memory. The one
  approved exception is a tap on the touch target pa3ds has just named, on the player's key
  (ADR-0009, amended 2026-10-01: a tap through Azahar's cemuhook touch input, not the mouse).
- **No hardcoded game text.** Every name, label and message *the game itself shows* comes from
  the ROM or RAM. Hardcoded struct layouts and anchor definitions are fine; hardcoded strings are
  not. What the game **never** names — a speaker, a sign, a door — may carry a name of pa3ds's own
  under three rules (Orion, 2026-09-21, reopening this): it never replaces something the game
  says, it lives in a data file a game and a language rather than in code, and the player's own
  names always win. `pa3ds.core.names`.
- **Anchors, not addresses.** Find structures by scanning and validating at runtime, then cache
  the result against a ROM fingerprint. A bare address is never the way to reach data.
- **Clean-room.** pokemon-access and PKHeX are GPL and Azahar is GPL-2. Read their behaviour and
  their formats; never copy their code.
- **Parity by default.** Anything the game does not show on screen (IVs, exact HP, hidden items)
  lives behind a per-item toggle that ships off.

And one of this repository's own: **the port behaves as upstream does.** Same reads, same clock
reads, same log lines, same speech. A bug found while porting is ported as it is and fixed
upstream first.

## Commands

```
cargo build --release
cargo test --lib
cargo clippy --all-targets
cargo fmt
cargo run --release -- --silent --no-log --seconds 60   # the runtime, printing what it would say
powershell -ExecutionPolicy Bypass -File tools\package.ps1   # the release zip, in target\package
```

`--silent` is mandatory from an agent session: without it pa3ds speaks through the screen reader
of whoever is at the machine, over whatever they are doing. `--no-log` keeps a run from rotating
the live logs in `user\logs\` beside pa3ds.

`pa3ds.exe` needs nothing installed: Prism, which reaches the screen reader, is built into it by
the `prismer` crate's static build. Building pa3ds therefore needs Rust's MSVC toolchain, Visual
Studio 2022 or newer with its C and C++ compiler and ATL, and CMake 4.2 or newer on the path
(Visual Studio's own 3.31 is too old for Prism's C23); the first build compiles Prism, about a
minute. Without ATL, Prism's SAPI, JAWS, System Access and ZoomText backends do not compile.

Change files with the Edit tool, which keeps a file's line endings. The checkout is LF
(`.gitattributes`), and on Windows a Python script's `write_text` rewrites a whole file as CRLF.

## Keeping up with upstream

```
python tools/sync.py status     # what changed upstream since port.toml's commit, file by file
python tools/sync.py mirror     # copy the data files and the licence across
python tools/sync.py layout     # create the Rust file for a new Python file
python tools/sync.py tests      # every Python test has a Rust test of its name
python tools/sync.py mark       # port.toml now matches upstream's HEAD
```

The Python checkout is `PA3DS_PYTHON`, or `../pokemon-access-3ds` beside this one. To sync:

1. `status`, then for each file listed to port by hand, read its upstream diff
   (`git -C ../pokemon-access-3ds diff <port.toml's commit>..HEAD -- <path>`) and make the same
   change to its Rust file, following `docs/porting.md`. Port the change to its tests too.
   Go an upstream commit at a time (`git -C ../pokemon-access-3ds log --reverse <port.toml's
   commit>..HEAD`), and commit each one, or each coherent group, on its own on a `sync/` branch:
   twenty small commits are easier to dissect than one of 15,000 lines (Orion, 2026-10-08).
   A rename upstream (`R` in `status`) is a `git mv` here. A file split into several shows only as
   new files, so read the commit that split it; after the split, the new Rust files' lines add up
   to the old file's, and `cargo check` passes before the next split begins.
2. `mirror` for the data files; rewrite this file, `README.md` or `CLAUDE.md` by hand where
   upstream's changed in a way that applies here.
3. `cargo test --lib`, `cargo clippy --all-targets`, `sync.py tests`.
4. With the upstream commit checked out there, record a tape of a movie that reaches the change
   and replay it here, with the tapes already recorded (`tools/tape/record_all.sh <replay>`, then
   `target\release\tape.exe %LOCALAPPDATA%\pa3ds\tapes\*.tape.gz`; `tools/tape/README.md`).
   Every tape must say "identical to the Python runtime's". This is the proof; the tests alone
   are not. Tapes recorded on an older upstream commit are re-referenced first by replaying them
   through the new Python (`tools/tape/py.py replay`), since what the Python says has changed.
5. `mark`, in a commit of its own once every tape is identical. Each port commit reads
   `feat(port): <what upstream did> (upstream <short sha>)`.

## Commits

Conventional Commits: `type(scope): summary`, with `feat`, `fix`, `docs`, `refactor`, `test`,
`chore`, and a scope naming the area (`battle`, `rig`, `speech`, `port`).

Nothing runs the checks on push, so run them before anything reaches the default branch:
`cargo test --lib`, `cargo clippy --all-targets` with no warnings, `cargo fmt --check` and
`python tools/sync.py tests`.

## Releases

`/release` (`.claude/skills/release/SKILL.md`) cuts one from `main`: it writes the player-facing
changes since the last `v*` tag into `CHANGELOG.md`, sets `version` in `Cargo.toml`, commits
`chore(release): v<version>` and tags it. Pushing the tag is publishing:
`.github/workflows/release.yml` builds the zip and makes the GitHub release, its description the
changelog's section, and every packaged pa3ds.exe offers it to its player on the next start.

That update check is `src/update.rs`, the port's own like `src/tape.rs`, with no Python
counterpart. It runs only in a build `tools/package.ps1` made (`PA3DS_RELEASE`), and never under
`--silent`, so `cargo run` and an agent's session never see its dialog.

## The README is the player's manual

[README.md](README.md) is what a player reads, and it ships in the release zip: the games, what is
said, setup, the default keys, every setting, the Azahar settings pa3ds writes. Upstream's README
is the same manual for the Python pa3ds (Orion, 2026-10-06); a sync that changes it changes this
one to match, in the same commit, but for running `pa3ds.exe` where upstream runs uv, and for
its Updating section, which only the port has. It names no ADR, finding or spike, except in its
section for developers.

## Runbooks

Upstream's runbooks are in its `docs/runbooks/` and apply as written to the game and the emulator;
where they run `uv run` or a spike, run them in the Python checkout. Spikes and the movie rig stay
upstream and are not ported (the user, 2026-10-03): they record how a finding was reached, and
drive the emulator rather than ship.
