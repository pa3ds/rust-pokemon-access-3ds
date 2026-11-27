# Porting a Python module to Rust

This repository is a port of the Python pa3ds (`port.toml` names the upstream commit). A ported
module must do what the Python does, step for step: the same reads of the emulator in the same
order, the same clock reads, the same log lines and the same speech. That is checked, not hoped
for: a **tape** records everything the emulator and the clock answered the Python runtime during
a movie's replay, and the Rust runtime replayed on it must ask the same questions and log and say
the same lines (`tools/tape/README.md`). A port that is "the same idea" fails the tape.

So port **faithfully, not idiomatically**: keep the Python's structure, names, order of operations
and its bugs. A bug noticed while porting is ported as it is and reported, never fixed here: the
fix belongs upstream first.

The reference ports show every rule below in use: `src/core/rpc.rs`, `src/core/memory.rs`,
`src/core/speech.rs`, `src/core/binary.rs`, and their tests in `tests/`.

## Where things go

- `src/pa3ds/<path>.py` is `src/<path>.rs`; a package's `__init__.py` is its `mod.rs`;
  `tests/<name>.py` is `tests/<name>.rs`. `python tools/sync.py layout` made every file and every
  `mod` declaration already. Add, remove or rename a module only where upstream did, in a sync
  (`AGENTS.md`, "Keeping up with upstream").
- Fill in only the files you were given. If a module you depend on needs a small change (an item
  made `pub`, a missing accessor), make the smallest change and list it in your report.
- `src/py.rs` holds what Python does differently from Rust (below). Do not edit it; ask for what
  it lacks in your report, and write a private helper meanwhile.
- `data/` is upstream's, which `tools/sync.py mirror` keeps: a game a folder and a language a
  folder inside it (ADR-0040). Its `.ftl` files, everything a translator edits, are read at run
  time (`core::folders::data_path`): a build run under `target\` reads the checkout's, and the
  release zip carries them beside `pa3ds.exe`. Anything else, a `.toml` table that is no
  translation, is built in by the reader that reads it, with `include_str!` (#16): its `shipped`
  is the table, and its `load` takes the table's text where the Python's takes its path.

## Names and documentation

- Same names: functions, methods, constants, classes (as structs, traits or enums) and fields keep
  their Python names. `_private` loses its underscore and is private in Rust, or `pub(crate)` if a
  test reaches it.
- Docstrings become `///` or `//!` doc comments, and comments `//` comments, word for word: they
  are the reasons the code is the way it is. Wrap at 100 columns. Add no comment of your own except
  where the Rust differs from the Python in a way the next porter would trip on, and then say why.
- A Python name that is a Rust keyword takes `r#`: `r#type`, `r#match`.

## Types

| Python | Rust |
| --- | --- |
| `int` | `i64`, always, addresses included. A byte is still `i64` once it is a number. |
| `float` | `f64` |
| `str` | `String`, `&str` to borrow |
| `bytes` | `Vec<u8>`, `&[u8]` to borrow |
| `list`, `Sequence` | `Vec<T>`, `&[T]` |
| `tuple` | a tuple, or a struct where the Python named its parts |
| `dict` | `IndexMap` (Python iterates a dict in insertion order) |
| `set`, `frozenset` | `py::PySet` if iterated and of ints, `HashSet` if only tested for membership |
| `X \| None` | `Option<X>` |
| `@dataclass` | a struct with `pub` fields, `#[derive(Debug, Clone, PartialEq)]` |
| `Protocol` | a trait |
| `Enum`, `IntEnum`, `StrEnum` | an enum, with `value()` where the value is used |
| `Callable[..., T]` | `Rc<dyn Fn(...) -> T>`; `py::Log` for `log` |
| `Final` constant | `pub const`, or a `fn` returning it where a const cannot hold it |

Keyword arguments with defaults: a constructor taking every argument, plus a short one for the
common call, or a `Default` impl and `..Default::default()`. Follow what the tests need.

## Objects that are shared

Python hands one object to many holders: the RPC client, a memory, the speech, a cache. In Rust
a holder keeps an `Rc<dyn Trait>` (or `Rc<Struct>`), and the methods take `&self`, with the
mutable state inside in `Cell` or `RefCell`. Never keep a `RefCell` borrowed across a call that
might come back into the same object.

- The emulator is `&dyn Rpc` to a function and `Rc<dyn Rpc>` to a holder (`core::rpc`). The UDP
  client is `UdpRpc`; tests use the fake emulator (`tests/fake_emulator.rs`).
- Memory is `&dyn Memory` / `Rc<dyn Memory>` (`core::memory`). Tests use `FakeMemory`, written
  through `&self` so a test can change it while a reader holds it.
- `with cache.phase("name"): ...` is `memory::phase(&cache, "name", || { ...; Ok(()) })?`.
- A Python object one owner mutates stays a plain struct with `&mut self` methods.
- A thread gets `Arc` and `Mutex`, only where the Python has a thread.
- A bound method one object hands another is a closure over what the method reads. Where it
  reads its holder's own state, that part is lifted into a struct both keep an `Rc` of: the
  navigator's naming methods, which its `Guide` is handed, are `navigate::Naming`.
- Where a Python object keeps another to consult as it polls, and both are kept by a third, the
  third hands the one to the other's `poll` instead: the kinds of cursor in `CursorReader`.
- A factory a `Game` holds (`Game.features`, `Game.apps`) is a plain `fn`, since a `Game` lives
  in a `static`. A test that hands over a reader it made parks it in a thread-local for the
  factory to return (`test_field`'s `given_feature`).
- What Python duck-types is a trait: `GameText.map_chunks` and `GameText.streets` are
  `ChunkSource` and `StreetSource`, so a test gives its own as Python's tests subclass. A method a
  test's subclass inherits becomes a function both call (`secret_base::spots_in`).

## Exceptions

`py::Error` is a raised exception: its class (`py::Kind`), its message, and the line that raised
it. `py::Result<T>` is `Result<T, py::Error>`.

- A function that can raise returns `Result`; one that cannot does not.
- `raise RomError("x")` is `return Err(Error::new(Kind::RomError, "x"))`. Keep the message word
  for word: it is logged.
- `except (A, B):` is a `match` on `error.is(Kind::A) || error.is(Kind::B)` (`is` is
  `isinstance`, bases included); anything else is returned on with `?` or `return Err(e)`.
- What Python raises without a `raise`:
  - `struct.unpack_from` on too few bytes is `struct.error`: `core::binary`'s functions return
    `Result` and raise it, so use them, with `?`.
  - Where the Python catches `IndexError` or `KeyError`, produce that error at the same spot
    (`Error::index`, `Error::key`) with `.get()`. Where it does not catch it, returning the error
    is still better than a panic, but a panic is caught by the runtime as Python's was.
  - `text.encode("ascii")` is `py::encode_ascii(text)?`, which raises `UnicodeEncodeError` (a
    `ValueError`) with Python's message for the first non-ASCII run. Never test `is_ascii()` and
    raise your own.
  - Never turn a raise into a default value: a reader that failed must fail.
- The runtime keys a fault by where it was raised (`Error::fault`): construct errors where the
  Python raised them, not in a shared helper (`#[track_caller]` on a helper that raises for its
  caller).

## Arithmetic

Rust's operators differ from Python's on negatives and halves. Use `py`:

| Python | Rust |
| --- | --- |
| `a // b` on ints | `py::floordiv(a, b)` |
| `a % b` on ints | `py::modulo(a, b)` |
| `a // b`, `a % b` on floats | `py::floordiv_f`, `py::modulo_f` |
| `round(x)` | `py::round(x)?`: halves to even |
| `round(x, n)` | `py::round_to(x, n)` |
| `int(x)` of a float | `py::int(x)?` |
| `math.floor(x)`, `math.ceil(x)` | `py::floor(x)?`, `py::ceil(x)?` |
| `a / b` on ints | `a as f64 / b as f64` |
| `~x` | `!x` |
| a signed 32-bit field (`<i`) | `binary::s32(blob, offset)?`, never `u32(..) as u32 as i32` |

`round`, `int`, `floor` and `ceil` return `Result` because Python raises on a NaN (`ValueError`)
and an infinity (`OverflowError`), and a float read out of freed memory is either; `as i64` would
quietly make it a number. Never cast a float to an int with `as` where Python called one of
these.

`math.hypot` is `py::hypot`: Rust's differs from CPython's in the last bit on about one pair in
eight. `min(a, b)` and `max(a, b)` of floats keep `a` unless `b` is strictly less (or greater), so
a NaN goes where Python puts it: never `f64::min`/`f64::max`, which drop a NaN. `sin`, `cos`,
`atan2`, `sqrt` and the rest are the same C library's in both on Windows.

`a // b` and `a % b` with both sides known non-negative may use `/` and `%`, but `py::` is never
wrong. A division Python could make by zero raises `ZeroDivisionError`; make it do so in Rust only
where it can happen.

## Text

- `len(text)` counts characters: `text.chars().count()`. Indexing or slicing a `str` is by
  character: collect a `Vec<char>` first.
- `text.split()` is `py::split`; `strip`, `lstrip`, `rstrip` are `py::strip` and the rest
  (Python's whitespace is wider than Rust's). `split(sep)` with a separator is Rust's `split`.
- f-strings: an `int` formats as itself. A `float` is **never** `{x}`: Python writes `1.0` where
  Rust writes `1`, so use `py::float_repr(x)`. `{x:.Nf}` is `py::fixed(x, n)` (Rust's `{x:.1}`
  writes `NaN` and `inf` where Python writes `nan` and `inf`), and `{x:g}` is `py::format_g(x)`.
  `{x:#010x}` and `{x:x}` are the same in Rust (on `i64`, mind a negative number: Python writes
  `-0x10`, Rust two's complement; where it can be negative, format the sign yourself).
- Reading text as a number: `int(text)` is `py::int_base10(text)?` and `int(text, 16)` is
  `py::int_base16(text)?` (a sign, outer whitespace, underscores and any script's digits, and
  Python's `ValueError` message otherwise), `float(text)` is `py::float_of(text)?`, and
  `bytes.fromhex(text)` is `py::fromhex(text)?`. Never `str::parse` or `from_str_radix`, which
  accept and refuse different texts.
- Character classes: `c.isdigit()` is `py::is_digit(c)` and `text.isdigit()` is `py::isdigit(text)`
  (it covers superscripts and circled numbers, which are digits and not decimals); `isdecimal()`
  and a regex `\d` are `py::is_decimal(c)`; `isalpha()` and `isalnum()` are `py::is_alpha(c)` and
  `py::is_alnum(c)`; `isspace()` is `py::is_space(c)`. Rust's `char::is_numeric`, `is_alphabetic`
  and `is_alphanumeric` are none of them.
- `text.splitlines()` and `text.splitlines(keepends=True)` are `py::splitlines` and
  `py::splitlines_keepends`: Python ends a line at `\v`, `\f`, `\x1c` to `\x1e`, `\x85`, ` `
  and ` ` as well as at `\n` and `\r`, and Rust's `lines()` does not.
- `textwrap.wrap(text, width)` is `py::wrap(text, width)?`, with `TextWrapper`'s defaults.
- Paths as text, on Windows: `str(path)` is `py::path_str(path)`, `Path(text)` is
  `py::to_path(text)`, and `os.path.normcase` and `os.path.normpath` are `py::normcase` and
  `py::normpath`. `PathBuf::display()` writes none of them.
- Anything that `encode("ascii")`s: see `py::encode_ascii` under Exceptions.
- `{x!r}` and `repr(text)` are `py::repr(text)`. `str(list)` writes each element's repr:
  `['a', 'b']`, `[(0, 0), (40, 0)]`. Write what Python writes.
- `bytes.decode(...)` is `py::decode_utf16le` (`errors="replace"`), `py::decode_utf16le_strict`
  (which raises `UnicodeDecodeError` with Python's message), `py::decode_ascii`,
  `py::decode_latin1`.
- `re` is the `regex` crate. It has no look-around or back-references: if a pattern needs them,
  say so in the report.

## Slices and order

- `data[a:b]` clamps where Rust panics: `py::slice(data, a, b)` and `py::slice_from`. Index
  directly only where the Python would raise.
- `sorted(items, key=...)` is `py::sorted_by_lt(items, |a, b| key(a) < key(b))` and `.sort()` is
  `py::sort_by_lt(&mut items, lt)`. Python's sort compares with `<` alone, so the order it makes
  depends on timsort's exact steps as soon as a key is NaN (a float read out of freed memory often
  is), and only CPython's own algorithm, which `py` follows step for step, gives that order: never
  `sort_by`, `sort_by_key` or `sort_unstable` where a key can be a float. For a tuple key, `lt`
  compares the first parts that differ with `!=` and then with `<`, as Python's tuple comparison
  does. With `reverse=True`, Python reverses, sorts and reverses again; do the same.
- `max(..., key=f)` returns the **first** of equal bests, and Rust's `max_by_key` the **last**:
  use `.rev().max_by_key(f)` or a loop where ties can happen. `min` and `min_by_key` both return
  the first. A float key needs `max_by`/`min_by` with `total_cmp`.
- Python iterates a `set` in hash order, not insertion order, and that order does reach what
  pa3ds reads: an insertion-ordered set made `layout.track` read its works in another order. A
  set of **ints** that is iterated is `py::PySet`, which lays the set out as CPython does
  (`PySet::of` for `set(iterable)` and `{a, b}`, `of_dict` and `update_dict` for `set(a_dict)`,
  `set.update(a_dict)` and `{*a_dict}`; `copy` for `set(s)` and `s.copy()`, `difference` for
  `s - t`, `union` for `s | t`, `intersection` for `s & t`, `update`, `difference_update`,
  `intersection_update`, `add`, `discard`, `remove`, `clear`). Each is written from CPython's
  `Objects/setobject.c` and the table is CPython's slot for slot, discarded keys included, so use
  the method rather than rebuilding a set from another one's iteration order: a copy and a
  difference lay a set out differently from a fresh `set(...)` of the same keys. If you need an
  operation it lacks (`symmetric_difference`, `pop`), write it from `setobject.c` beside the
  others, check it against CPython on many random sequences, and report it. A set of
  strings Python iterates in an order that changes every run, so pa3ds cannot depend on it:
  `IndexSet`, and list the spot in your report.

## Time, files, threads

- `time.monotonic()` is `py::time::monotonic()` and `time.sleep(s)` is `py::time::sleep(s)`, so
  a tape can answer them. Never `std::time` or `std::thread::sleep` directly; the RPC client is
  the one exception, being below the tape. A `clock` parameter stays a parameter,
  `Rc<dyn Fn() -> f64>`, defaulting to `py::time::monotonic`.
- `pathlib.Path` is `PathBuf`; an `OSError` from the file system comes from `?` on `io::Error`.
- TOML is read with the `toml` crate (insertion order kept). `pa3ds.core.toml` writes it.
- Windows calls are `windows-sys`. A `ctypes` structure is a `#[repr(C)]` struct.

## Tests

- Port every test in the test files you were given, each as a `#[test] fn` of the **same name**:
  `python tools/sync.py tests` checks the names.
- A parametrized test is one Rust test looping over its cases, naming the case in its assertion
  message. A fixture is a helper function.
- `pytest.raises(X, match="m")` is `let error = ....unwrap_err(); assert!(error.is(Kind::X) &&
  error.message.contains("m"))` (or a regex where `match` used one).
- Assert against the same literal values the Python asserts. A test that would still pass with
  every function returning a default proves nothing.
- A test that cannot be ported at all (it tests Python itself) goes in `port.toml`'s
  `[untestable]` with the reason, and in your report. This should be rare.

## Before you report

1. `cargo build` and `cargo test --lib <your test file names>` pass.
2. `cargo clippy --all-targets` shows no warning in your files.
3. `cargo fmt` has been run.
4. `python tools/sync.py tests` shows none of your test names missing.
5. Commit on your branch: `feat(port): <modules>`, ending with the attribution line you were
   given.

Report: each module ported (Python lines to Rust lines), tests ported and passing, every place the
Rust behaves differently from the Python (the target is none), every set whose order you guessed,
every edit outside your files, any crate added, and any upstream bug you ported as it is.

## Where the port knowingly differs

Kept here so the next porter, and anyone reading a tape that differs, knows what is deliberate.

- **Integers past `i64`.** Python's ints never overflow. A float made an int past about 9.2e18
  saturates (`py::int`, `round`, `floor`, `ceil`); a product of two words out of freed memory is
  saturated where a port saw it (`lists`, `rtti`, `walls`, `azahar`) and widened to `i128` in
  `mapblock`; anything else overflows, and since release builds check overflow (`Cargo.toml`) it
  panics and the runtime takes it for a reader's failure ("Reader error") where Python would have
  read on with a big number. Only garbage reaches these sizes.
- **A reader's failure is logged with where it was raised, not a traceback**, and a fault is
  keyed by that line, always with a file and line where Python's could be a bare class name. A
  short read is raised in `core::binary` for every reader alike, so its log adds the line that
  called the helper, "called from" (#18), the frame Python's traceback would have named last
  but one; the fault is still keyed by `core::binary`'s line.
- **Tests adapted where Python patched code in place.** `test_app` reaches the runtime through
  `__main__::main_with` and `app::run_with`; `test_navigate`'s "P again finds no way" uses a
  `FakeField` hook; and `test_sm_zones`' "areas read file by file" lost its guard that the event
  archive is never read whole, which needed `Rom.read` patched.
- **pa3ds's own threads.** The touch server (`core::touch`) logs from its thread, which cannot
  call the runtime's `Rc` log: its lines go through a channel the session loop drains each poll,
  so a line can land a poll later than Python's. A tone that cannot start its sound thread is
  dropped where Python would raise to the key's handler. Neither exists in a replay.
- **A settings or keys file that will not parse** is said with the `toml` crate's words for what
  is wrong, not `tomllib`'s: "Invalid string (at line 2, column 10)" where the Python says
  "Invalid value (at line 2, column 10)". The line and the column are the same (`core::toml`'s
  `parse`). No tape starts with a broken file.
- **One pa3ds at a time** (`core::instance`): `Beside` is handed how it finds Azahar and how it
  hides, which the Python's test monkeypatched, and the lock is let go by a guard dropped at the
  end of `__main__`'s `run`, where the Python registers `atexit`. The tests' stand-in for the pa3ds
  replaced is `ping`, where the Python's is its own interpreter asleep.
- **pa3ds's own words** (`core::words`). The `fluent-bundle` crate stands in for Python's
  `fluent.runtime`. A function a message calls, a scene's `POKEMON(383)`, is looked up on the
  thread that formats it, since the crate wants functions it can send to another thread; one that
  finds nothing marks the message failed, as Python's catching its exception does. Values are
  integers or text: the crate writes a float as "1.5" where Python's writes it in the language's
  own way, and no message is given one. The words in place are a thread's own, where Python's are
  the module's: the rename box is built on a thread of its own, so its OK and Cancel are looked up
  on the thread that asks for it and handed over. Under `cargo test` a thread that has put no
  words in place reads Omega Ruby's English, where pytest's autouse fixture puts them in place
  before each test.
- **Tables built in** (#16). `scenes.toml` and `footsteps.toml` are part of `pa3ds.exe`, where
  the Python reads them from `data/` as it starts: a player editing a copy beside the port's
  `pa3ds.exe` changes nothing, and a table edited in the checkout takes a rebuild. Tuning them by
  ear stays upstream's.
- **pa3ds's own sounds.** The footsteps' random volume, `random.uniform`, draws on the standard
  library's randomly keyed hasher, not Python's generator; `wave.Error` and `EOFError` are
  ValueErrors, which `footsteps::load` catches with an OSError. A sound plays only where pa3ds
  takes keys, so no tape hears one.
- **What the tapes cannot check.** A movie presses no pa3ds keys, so the navigation keys are
  checked by their tests alone; and no tape so far holds a reader's failure.
