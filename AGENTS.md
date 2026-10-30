# pa3ds, in Rust

pa3ds speaks 3DS Pokemon games to blind players: it reads Azahar's emulated memory and sends text
to a screen reader. Two people work on it, Orion and Khai, each on a machine of their own, sharing
the movies regression runs on in `regression/` beside the two checkouts (`pa3ds/regression`,
ADR-0041); each records the tapes of those movies on their own machine. Orion is blind and cannot
check your work by looking at the game, and you cannot see the screen either: tests, snapshots,
tapes and screenshots are the only shared evidence.

**This repository is a port.** The Python pa3ds (https://github.com/pa3ds/pokemon-access-3ds)
is upstream: features are found and built there first, with its spikes and movie rig, and arrive
here as a port of the commit `port.toml` names. Each Python file has one Rust file at the same
path (`docs/porting.md`), so an upstream change is a change to the files it maps to.

The Python checkout sits beside this one, at `../pokemon-access-3ds` (or `PA3DS_PYTHON`). Its
glossary and docs are this repository's too, and are not copied here: before the first change in
a session, read its `GLOSSARY.md` in full — the vocabulary, which code and conversation both use
— and the **ADRs** for the area you are working in (`docs/adr/README.md` there is grouped by
area). Before changing a module, read every ADR its module doc lists under **Bound by**, copied
from its Python docstring: a rule every screen goes through, as `common.touch` and
`common.screen_announce` are, keeps decisions made for screens far from the one you are changing.
An ADR is binding: contradict one only by saying so. A path such as `docs/runbooks/rig.md`
in a comment or doc here means the Python checkout's. This repository's only doc is
`docs/porting.md`.

## Talking to upstream

- **What to port** is the open issues in this repository labelled `upstream`, kept up to date by
  `.github/workflows/upstream.yml`: each push to upstream's main rewrites the one nobody has
  claimed with `sync.py status`, or opens one starting where the claimed one ends, and a push
  here that brings `port.toml` to an issue's commit closes it. An issue is ported up to the
  commit its title names, never further, and never closed by hand: a claimed issue rewritten
  under Khai while he ported nearly closed at the commit his agent had read, leaving what was
  pushed meanwhile in no issue (Orion, 2026-10-25). Several open, port the oldest first.
- **Claim an issue before working on it**, the `upstream` one or any other. Orion and Khai port
  from machines of their own, and once nearly cut the same release twice (Orion, 2026-10-18).
  Look first (`gh issue view <n>`): an issue labelled `in progress` is someone else's, so stop and
  ask. Otherwise mark it: `gh issue edit <n> --add-label "in progress" --add-assignee @me`. The
  mark stays until the work reaches the default branch, which closes the issue; if you stop
  before then, take it off (`--remove-label "in progress" --remove-assignee @me`).
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
  approved exception is a touch on the touch target pa3ds has just named, on the player's keys:
  a tap, or a drag the player holds from target to target (ADR-0009, amended 2026-10-01: through
  Azahar's cemuhook touch input, not the mouse; and 2026-10-28, the drag).
- **No hardcoded game text.** Every name, label and message *the game itself shows* comes from
  the ROM or RAM. Hardcoded struct layouts and anchor definitions are fine; hardcoded strings are
  not. What the game **never** names — a speaker, a sign, a door — may carry a name of pa3ds's own
  under three rules (Orion, 2026-09-21, reopening this): it never replaces something the game
  says, it lives in a data file a game and a language rather than in code, and the player's own
  names always win. `pa3ds.core.names`. Every other word pa3ds says of its own is a message in
  `data/` too: code asks for `own("cant-continue", &[])`, and the English goes in a `.ftl` file
  under `data/common/en/` or the game's (ADR-0040, `core::words`). `data/` is upstream's, copied
  by `tools/sync.py mirror`: a word changes there first.
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
of whoever is at the machine, over whatever they are doing, and ends the pa3ds they are playing
with (ADR-0039). `--no-log` keeps a run from rotating
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

0. Claim the oldest open `upstream` issue (*Talking to upstream*, above), and check out the
   commit its title names in the Python checkout: that is HEAD below, whatever upstream's main
   has moved on to since. `git fetch` this repository and branch from `origin/main`: its
   `port.toml` must name the commit the issue's body says port.toml matches. Where it does not,
   your main is stale or a port has not landed yet, so stop and ask. A sync begun on a main two
   days stale ported again what Khai's PR #32 had merged (Orion, 2026-10-30); `status` now
   warns where origin's `port.toml` is ahead of yours.
1. `status`, then for each file listed to port by hand, read its upstream diff
   (`git -C ../pokemon-access-3ds diff <port.toml's commit>..HEAD -- <path>`) and make the same
   change to its Rust file, following `docs/porting.md`. Port the change to its tests too.
   Go an upstream commit at a time (`git -C ../pokemon-access-3ds log --reverse <port.toml's
   commit>..HEAD`), and commit each one, or each coherent group, on its own on a `sync/` branch:
   twenty small commits are easier to dissect than one of 15,000 lines (Orion, 2026-10-08).
   A rename upstream (`R` in `status`) is a `git mv` here. A file split into several shows only as
   new files, so read the commit that split it; after the split, the new Rust files' lines add up
   to the old file's, and `cargo check` passes before the next split begins.
2. `mirror` for the data files; rewrite this file, `README.md`, `manual/` or `CLAUDE.md` by hand where
   upstream's changed in a way that applies here.
3. `cargo test --lib`, `cargo clippy --all-targets`, `sync.py tests`.
4. With the upstream commit checked out there, replay every tape through it to make the
   references (`uv run python spikes/tape_replay.py replay port-<short sha>` in that checkout),
   then through this runtime against them (`target\release\tape.exe --against
   %LOCALAPPDATA%\pa3ds\captures\regress\port-<short sha> %LOCALAPPDATA%\pa3ds\tapes\*.tape.gz`;
   `tools/tape/README.md`). Every tape must say "identical to the Python runtime's". This is the
   proof; the tests alone are not. A tape the Python replays as approximate reads what it never
   recorded: record it again at that commit (`tools/tape/record_all.sh <replay>`), and record one
   of a movie that reaches the change where none does.
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

[README.md](README.md) and the pages it links to in [`manual/`](manual/) are what a player reads,
and they ship in the release zip: the README the games, what is said, setup and updating;
`keys.md` the default keys; `settings.md` every setting, the Azahar settings pa3ds writes and the
player's files; `languages.md` languages and translating; `known-issues.md` every known problem
and gap a player meets, there and not in the other pages (Orion, 2026-10-24). Upstream's are the
same manual for the Python pa3ds (Orion, 2026-10-06); a sync that changes them changes these to match, in the same
commit, but for running `pa3ds.exe` where upstream runs uv, and for the Updating section, which
only the port has. Keep them short, as upstream's AGENTS.md says (Orion, 2026-10-18). They name
no ADR, finding or spike. Building and releasing are in [BUILDING.md](BUILDING.md), which does
not ship.

## Runbooks

Upstream's runbooks are in its `docs/runbooks/` and apply as written to the game and the emulator;
where they run `uv run` or a spike, run them in the Python checkout. Spikes and the movie rig stay
upstream and are not ported (the user, 2026-10-03): they record how a finding was reached, and
drive the emulator rather than ship.
