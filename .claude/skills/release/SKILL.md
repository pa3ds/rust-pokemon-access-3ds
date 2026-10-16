---
name: release
description: Cut a pa3ds release. Reads the commits since the last version tag, writes what changed for players into CHANGELOG.md, bumps the version, commits and tags the release candidate, and on the user's word pushes the tag, which builds and publishes the GitHub release. Use when the user runs /release or asks to cut, tag or publish a release.
disable-model-invocation: true
---

# Release

A release is a commit on `main` that sets `version` in `Cargo.toml` and adds that version's
section to `CHANGELOG.md`, tagged `v<version>`. Pushing the tag runs
`.github/workflows/release.yml`, which builds the zip on Windows and publishes the GitHub release
with the section as its description. Every packaged pa3ds.exe then offers the update to its player
on its next start (`src/update.rs`). So the section is written for a player, and pushing is
publishing.

## 1. Start from main, level with origin

```
git fetch origin
git status -sb
```

Stop and say why if the branch is not `main`, the tree is not clean, or `main` is not level with
`origin/main`.

## 2. Read what changed

```
git describe --tags --abbrev=0 --match "v[0-9]*"
git log --reverse --format="%h %s%n%b" <tag>..HEAD
```

No tag means the first release. If the range is empty, stop: there is nothing to release.

## 3. Keep what a player notices

A change is user-facing when it changes what a player hears, presses, sets, installs or reads in
`README.md`. Most `feat(port)` and `fix(port)` commits are: each brings upstream's behaviour to
players. So are changes to `README.md`, to packaging and to updating.

Leave out `chore(port): level with upstream`, tests, refactors, tapes, CI, `AGENTS.md`,
`docs/porting.md` and this skill. When a subject does not say what a player would notice, read the
commit (`git show <sha>`) before deciding.

## 4. Write the section

Add it to `CHANGELOG.md` above the newest section, below the introduction:

```
## [0.2.0] - 2026-10-16

### Added

- Shift+C says where the chosen thing is.

### Fixed

- Ctrl+P walks a stacked ground one floor at a time.
```

- `Added`, `Changed`, `Fixed`, in that order, each only when it has an entry.
- One bullet per change a player would tell apart. Commits that make one change together are one
  bullet.
- In the README's words and voice: plain, present tense, keys and settings named as the README
  names them. No ADR, spike, finding, file path, commit or upstream hash, and no "port".
- First release: instead of a list, one short paragraph saying what pa3ds does and which games it
  reads, from the README's opening and its Games section.

## 5. Choose the version

Before 1.0: anything under Added or Changed bumps the minor version (0.1.0 to 0.2.0); only Fixed
bumps the patch (0.2.0 to 0.2.1). The first release is 0.1.0. 1.0 is the user's call. Set
`version` in `Cargo.toml` and put the same version in the section's heading.

## 6. Run the checks

```
cargo fmt --check
cargo clippy --all-targets
cargo test --lib
python tools/sync.py tests
powershell -ExecutionPolicy Bypass -File tools\package.ps1 -SkipTests
```

Clippy must report no warnings. The last builds the release exactly as the workflow will and
checks it starts. The builds also update `Cargo.lock` to the new version. If anything fails,
stop and report it: a release is not cut on red.

## 7. Commit and tag the release candidate

```
git add CHANGELOG.md Cargo.toml Cargo.lock
git commit -m "chore(release): v<version>"
git tag -a v<version> -m "pa3ds <version>"
```

The commit and the tag are still only on this machine.

## 8. Publish on the user's word

Show the user the version and the section exactly as written, then ask with `AskUserQuestion`
whether to publish. Never push without that answer.

- **Publish:** `git push --atomic origin main v<version>`.
- **Change something:** edit the section, `git commit --amend --no-edit`,
  `git tag -f -a v<version> -m "pa3ds <version>"`, and ask again.
- **Hold:** leave both local. To drop the candidate: `git tag -d v<version>` and
  `git reset --hard HEAD~1`.

## 9. Watch it publish

```
gh run list --workflow release.yml --limit 1
gh run watch <run id> --exit-status
gh release view v<version> --json url,assets
```

If the run fails, the tag is public with no release behind it. Read the log
(`gh run view <run id> --log-failed`). If the fault is in the workflow's environment, re-run it
(`gh run rerun <run id>`). If it needs a change on `main`, delete the tag
(`git push origin :refs/tags/v<version>`, `git tag -d v<version>`), fix it, and run `/release`
again.

Report the release's URL, its version and its section.
