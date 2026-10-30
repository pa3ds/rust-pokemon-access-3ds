# Tapes: the port checked against the Python runtime

A **tape** is everything the emulator and the clock answered the Python runtime during one replay
of a movie, in the order it asked: each `Rpc` call with its reply, and each `time.monotonic` and
`time.sleep` made from pa3ds's own modules. Replaying a tape needs no emulator and no ROM rig.
The Rust runtime is answered by the rules the Python's `spikes/tape_replay.py replay` answers
the Python runtime by, so a faithful port gets exactly the answers the Python's replay got and
logs and says exactly the same lines, even on a tape neither asks just as its recording did.

Two live runs of the same commit differ by tens of lines (`docs/runbooks/rig.md`), which is why a
live comparison cannot prove a port line for line and a tape can.

Tapes hold raw game RAM, which holds the player's save data (ADR-0029). They live under
`%LOCALAPPDATA%\pa3ds\tapes`, never in the repository.

## Recording, and the Python's references

Tapes are recorded, checked and replayed through the Python runtime by the Python checkout's own
`spikes/tape_replay.py`, from that checkout (`docs/runbooks/rig.md` there, "Before and after a
change"):

```
uv run python spikes/tape_replay.py record <replay>...
uv run python spikes/tape_replay.py verify <replay>...
uv run python spikes/tape_replay.py replay <label> [<replay>...] [--covers <tag>...]
```

`record` plays a replay as `spikes/regress.py run` plays it: one of the movies Orion and Khai
share in `regression/replays.toml`, the `pa3ds/regression` checkout beside the two repositories
(ADR-0041), or a `[[replay]]` of the Python checkout's `spikes/rig.toml` for a movie not shared
yet. It writes `<replay>.tape.gz` with, beside it, what the runtime logged and said and the commit
it ran. `--covers` takes every replay covering one of the tags of `regression/tags.toml`. `verify`
replays each tape strictly against its recording, and holds only at that commit. `replay` plays
each tape through whatever Python is checked out and writes what it logged and said to
`%LOCALAPPDATA%\pa3ds\captures\regress\<label>\<name>.log` and `.said`: these are the references
the Rust runtime is held to. Its `summary.txt` says of each whether its reads were all exact; an
approximate one read memory the tape never held, and only a tape recorded again at that commit
can prove the port there.

A replay plays with the default settings, where `[pa3ds.ui] descriptions` is off and no scene is
described, but for `census`: both runtimes log touch mode's targets as they change, "touch
targets: ...", since no movie presses the key that would say them (ADR-0041). A tape recorded
before 6b42273c holds none of the reads that takes, and is recorded again. Name it
`<replay>_described` to play `<replay>` with descriptions on: both runtimes turn them on for a
tape of that name.

`tools/tape/record_all.sh <replay>...` records each replay, makes its references under the label
`port`, and replays it through the Rust runtime against them.

## Replaying through the Rust runtime

```
cargo build --release
target\release\tape.exe --against %LOCALAPPDATA%\pa3ds\captures\regress\<label> %LOCALAPPDATA%\pa3ds\tapes\*.tape.gz
```

writes `<name>.rust.log` and `<name>.rust.said` beside each tape and compares them with the
Python replay's `<name>.log`, its seconds left out, and `<name>.said` under the label, saying for
each tape either "identical to the Python runtime's" or the first line each logged or said
differently. Each tape's line starts as the Python's `summary.txt` line does, with how its
questions were answered, "exact 180734, covered 106, stale 0, missing 0, unused 0", and
"approximate" where a stale or missing answer made it so: the two must agree.

The tape is read a tick at a time, a tick ending at the poll loop's own sleep, which the runtime
makes through `py::time::sleep_ending_tick` where the Python's replay knows it by its caller.
Each question is answered from its own tick: **exact**, the tick's oldest unanswered record of
that question; **covered**, bytes the tick read under another question, the latest at or before
the last exact answer, else the first after it; **stale**, bytes only an earlier tick read, the
newest; **missing**, none, which fails a lone read as "not on the tape" and is left out of a
`read_each`. A clock, `unsure` or `reply_seconds` asked more often than recorded gets the last
value given; what a tick recorded and nobody asked is counted unused (`src/tape.rs`).

## Live runs

`tools/tape/live.py <replay>...` replays a movie with the shipped `pa3ds.exe` reading the running
emulator, and with `--python` the Python runtime as the control; `spikes/regress.py compare
python-live rust-live` in the Python checkout diffs the two. A live run is judged against the noise
between two Python runs, as any change upstream is (`docs/runbooks/rig.md`), not line for line.

## What is compared, and what is not

Every log line and every spoken line, in order. Two kinds of line are compared by their first
line only: a reader's failure, whose traceback names Python files, and the snapshot path written
for it.

A `read_each` batch is the same question in any order: the Python sends some batches in the
order of a `set`, which no other language reproduces, and a batch's order changes only the order
its replies come back in.

## Format

gzip of: the magic `PA3DSTAPE1\n`, then records, each a one-byte kind and its fields,
little-endian. A string is a `u32` byte length and UTF-8; a blob a `u32` length and bytes. An
outcome is a `u8`: 0 for a reply, which follows, or the class of the exception raised, followed by
its message: 1 `RpcError`, 2 `RpcTimeoutError`, 3 `RpcRefusedError`, 4 `RpcLostError`,
5 `ValueError`.

| kind | record | fields |
| --- | --- | --- |
| 1 | `time.monotonic()` | `f64` the time |
| 2 | `time.sleep(s)` | `f64` s |
| 3 | `read(address, size)` | `u32` address, `u32` size, outcome, blob |
| 4 | `read_each(reads)` | `u32` n, n × (`u32` address, `u32` size), outcome, `u32` m, m × (`u32` address, blob) |
| 5 | `unsure` | `u8` |
| 6 | `reply_seconds` | `f64` |
| 7 | `confirm()` | outcome, `u8` |
| 8 | `attach()` | outcome, `u32` pid, `u64` title id, string name |

`read_many` is `read_each` underneath and is recorded as one. Calls the client makes of itself,
as `read` asking `unsure` and `confirm` before it reads, are inside the outer call and not
recorded. `src/tape.rs` reads the same format.
