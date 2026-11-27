//! ADR-0006's imports go one way: the core, then `common`, then `gen6` or `sm`, then `oras` or
//! `xy`, with `catalog` and `app` above them all. Nothing checked it, and Sun's text came to import
//! gen 6's, a sibling's (the untangling round, step 0).
//!
//! The Python reads each module's imports with `ast`; here each Rust file's `crate::` paths are
//! read, its `use` lines and the paths in its code alike, comments left out. `py` is the port's
//! stand-in for Python itself, its standard library, and is no module of pa3ds's, as `tests` is
//! none of the runtime's.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use regex::Regex;

/// Each group of modules by its place: a module imports its own group's and lower places'
/// alone, never a sibling's at its own place. `pa3ds` and `games` are the packages themselves, a
/// docstring each. `ux`, the windows pa3ds opens, knows no game, as the core does not, and the
/// navigator asks a name with its dialog. `__main__` starts `app`; and `debug` is no runtime's, so
/// nothing imports it (ADR-0002).
///
/// The port's own beside them: `update`, its update check, which imports nothing of pa3ds's and
/// which the window offers; `tape`, which answers the core's RPC from a recorded run and which
/// `app` replays; and the executables, `main` and `bin`, above them all.
const LAYERS: [(&str, i64); 17] = [
    ("pa3ds", 0),
    ("games", 0),
    ("core", 0),
    ("update", 0),
    ("ux", 1),
    ("tape", 1),
    ("games.common", 2),
    ("games.gen6", 3),
    ("games.sm", 3),
    ("games.oras", 4),
    ("games.xy", 4),
    ("games.catalog", 5),
    ("app", 6),
    ("__main__", 7),
    ("debug", 7),
    ("main", 8),
    ("bin", 8),
];

/// Imports against the order, each until the step that ends it.
const KNOWN: [(&str, &str, &str); 4] = [
    (
        "pa3ds.games.sm.gametext",
        "pa3ds.games.gen6.rom.people",
        "the untangling round's step 9 moves the tagged species and model tags to games.common",
    ),
    (
        "pa3ds.core.pathfind",
        "pa3ds.games.common.field.climb",
        "the port's: Python's `Held`, a leg climbed in place, subclasses `Leg` in \
         `common.field.climb`, where the port's `Leg` holds it as a field",
    ),
    (
        "pa3ds.core.pathfind",
        "pa3ds.games.common.field.things",
        "the port's: Python's `Teleported` leg says itself in `things`' words in \
         `common.field`, where the port's `Leg` says it as a field says it is one",
    ),
    (
        "pa3ds.games.gen6.profile",
        "pa3ds.app",
        "the port's: Python's `Counter.most_common` is the port's `app.most_common`, which the \
         profile's anchors' log counts with as `app`'s sweeps' does",
    ),
];

fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every `.rs` file under `dir`.
fn files_in(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files_in(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

/// `src/games/gen6/profile.rs` -> `pa3ds.games.gen6.profile`, a package by its own `mod.rs`,
/// and the crate by `lib.rs`.
fn module_of(path: &Path) -> String {
    let relative = path.strip_prefix(src()).unwrap().with_extension("");
    let mut parts: Vec<String> = relative
        .iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect();
    if matches!(parts.last().map(String::as_str), Some("mod" | "lib")) {
        parts.pop();
    }
    let mut module = vec!["pa3ds".to_string()];
    module.extend(parts);
    module.join(".")
}

/// The group a module of pa3ds's is in: `games.gen6` for `pa3ds.games.gen6.profile`.
fn group(module: &str) -> String {
    let parts: Vec<&str> = module.split('.').collect();
    if parts.get(1) == Some(&"games") && parts.len() > 2 {
        return format!("games.{}", parts[2]);
    }
    if parts.len() > 1 {
        parts[1].to_string()
    } else {
        parts[0].to_string()
    }
}

/// Every module of pa3ds's a file reaches, in a `use` or in its code: `crate::a::b::C` is `a.b`
/// where that is a module, else `a`; the executables reach the crate as `pa3ds::`.
fn imports(path: &Path, modules: &HashSet<String>) -> HashSet<String> {
    let text = std::fs::read_to_string(path).unwrap();
    let comment = Regex::new(r"//[^\n]*").unwrap();
    let code = comment.replace_all(&text, "");
    let braced = Regex::new(r"\b(?:crate|pa3ds)::((?:\w+::)*)\{([^{}]*)\}").unwrap();
    let pathed = Regex::new(r"\b(?:crate|pa3ds)::(\w+(?:::\w+)*)").unwrap();
    let mut paths: Vec<String> = Vec::new();
    for found in braced.captures_iter(&code) {
        for item in found[2].split(',') {
            let item = item.trim();
            if !item.is_empty() {
                paths.push(format!("{}{item}", &found[1]));
            }
        }
    }
    paths.extend(pathed.captures_iter(&code).map(|found| found[1].to_string()));
    let mut found: HashSet<String> = HashSet::new();
    for path in paths {
        let parts: Vec<&str> = path
            .split("::")
            .map(|part| part.split_whitespace().next().unwrap_or(""))
            .collect();
        // The longest of its prefixes that is a module.
        let module = (1..=parts.len())
            .rev()
            .map(|end| format!("pa3ds.{}", parts[..end].join(".")))
            .find(|module| modules.contains(module));
        if let Some(module) = module
            && !matches!(group(&module).as_str(), "py" | "tests")
        {
            found.insert(module);
        }
    }
    found
}

#[test]
fn test_imports_go_one_way() {
    let mut paths = Vec::new();
    files_in(&src(), &mut paths);
    let files: HashMap<String, PathBuf> = paths
        .into_iter()
        .map(|path| (module_of(&path), path))
        .collect();
    let layers: HashMap<&str, i64> = LAYERS.into_iter().collect();
    let modules: HashSet<String> = files.keys().cloned().collect();
    let unplaced: BTreeSet<String> = modules
        .iter()
        .map(|module| group(module))
        .filter(|group| !layers.contains_key(group.as_str()) && group != "py")
        .collect();
    assert!(unplaced.is_empty(), "place these in LAYERS: {unplaced:?}");
    let mut against: BTreeSet<(String, String)> = BTreeSet::new();
    for (module, path) in &files {
        if group(module) == "py" {
            continue;
        }
        for imported in imports(path, &modules) {
            let (mine, theirs) = (group(module), group(&imported));
            if theirs != mine && layers[theirs.as_str()] >= layers[mine.as_str()] {
                against.insert((module.clone(), imported));
            }
        }
    }
    let known: BTreeSet<(String, String)> = KNOWN
        .iter()
        .map(|(module, imported, _)| (module.to_string(), imported.to_string()))
        .collect();
    let new: Vec<_> = against.difference(&known).collect();
    assert!(new.is_empty(), "imports against ADR-0006's order: {new:?}");
    let gone: Vec<_> = known.difference(&against).collect();
    assert!(
        gone.is_empty(),
        "exceptions that no longer import against it: {gone:?}"
    );
}
