//! pa3ds's own words, in the player's language with English behind them (ADR-0040).
//!
//! Whatever pa3ds says that the game does not is a message in a Fluent file under `<pa3ds>/data`:
//! code asks for `cant-continue`, and `data/common/<language>/words.ftl` says what that is. A game's
//! folder, `data/<game>/<language>/`, is read before `common`'s, so a game's own message wins where
//! both have one. A message no file of the player's language has is looked for in English's, so a
//! language half translated is half English; one neither has is said as its id, which a test keeps
//! from happening (`tests/test_words.py`).
//!
//! Two of Fluent's defaults are off, or the port, which reads the same files through Rust's library,
//! would say other words: the marks it puts round each value inserted (U+2068 and U+2069), and
//! grouping a number's digits ("1,234"), which Python's library does by the language and Rust's
//! never does.
//!
//! pa3ds holds the words of one game at a time, as it reads one game at a time: a session puts its
//! own in place as it starts (`use`), and every reader asks through `own`. Before then, and in a test
//! that puts none in place, the words are English, every game's alike.
//!
//! Bound by:
//! - ADR-0040: code asks for a message by id, and its words are in `data/`, the player's language
//!   before English.

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use fluent_bundle::types::{FluentNumber, FluentNumberOptions};
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use fluent_syntax::ast;
use indexmap::IndexMap;
use unic_langid::LanguageIdentifier;
use unicode_normalization::UnicodeNormalization;

use crate::py::{self, Log, Result};

/// The language every message is written in first, and the one a message is looked for in when
/// the player's has none.
pub const ENGLISH: &str = "en";
/// The folder of the words every game says alike: the core's, `games.common`'s and gen 6's.
pub const COMMON: &str = "common";

/// What a message may call by name, as a scene's description calls `POKEMON(383)`: the values it
/// is given, and its words, or None where it finds nothing, which Python's raised.
pub type Function = Rc<dyn Fn(&[Value]) -> Option<String>>;

/// A value as a message is given it: text, or a number, which Fluent selects a plural by.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
}

/// What `own` and `Words::get` take a value as: Python's keyword arguments of any type.
pub trait IntoValue {
    fn value(&self) -> Value;
}

impl IntoValue for str {
    fn value(&self) -> Value {
        Value::Text(self.to_string())
    }
}

impl IntoValue for &str {
    fn value(&self) -> Value {
        Value::Text(self.to_string())
    }
}

impl IntoValue for String {
    fn value(&self) -> Value {
        Value::Text(self.clone())
    }
}

impl IntoValue for i64 {
    fn value(&self) -> Value {
        Value::Number(*self as f64)
    }
}

impl IntoValue for usize {
    fn value(&self) -> Value {
        Value::Number(*self as f64)
    }
}

impl IntoValue for f64 {
    fn value(&self) -> Value {
        Value::Number(*self)
    }
}

impl IntoValue for bool {
    /// As Python's `str(True)`: Fluent is never given a bool as a number.
    fn value(&self) -> Value {
        Value::Text(if *self { "True" } else { "False" }.to_string())
    }
}

impl IntoValue for Value {
    fn value(&self) -> Value {
        self.clone()
    }
}

/// A message's values by name, `&[("count", &3)]`.
pub type Values<'a> = &'a [(&'a str, &'a dyn IntoValue)];

thread_local! {
    /// The functions of the `Words` formatting a message now, which each name's trampoline in a
    /// bundle calls: Rust's library keeps a function only if it can cross threads, and these
    /// close over a session's own state. Python's bundle holds the functions themselves.
    static CALLING: RefCell<Option<Rc<IndexMap<String, Function>>>> =
        const { RefCell::new(None) };
    /// Whether a function called while formatting found nothing. Rust's library writes `NAME()`
    /// for a function that fails and records no error, where Python's records the exception.
    static FAILED: Cell<bool> = const { Cell::new(false) };
}

/// The messages of one game in one language, English behind them.
pub struct Words {
    pub language: String,
    game: Option<PathBuf>,
    log: Log,
    made: RefCell<HashSet<String>>,
    bundles: Vec<FluentBundle<FluentResource>>,
    /// Each bundle's messages, in the order its files hold them.
    ids: Vec<Vec<String>>,
    functions: Rc<IndexMap<String, Function>>,
}

impl Words {
    /// `language` is a folder's name, `zh-CN`; `game` the game's folder, `data/oras`, beside
    /// which `common` is, or None for the words every game shares alone, `data/common`'s.
    /// `functions` are what a message may call by name, as a scene's description calls
    /// `POKEMON(383)` for the game's own word for Groudon.
    pub fn new(
        language: &str,
        game: Option<&Path>,
        functions: IndexMap<String, Function>,
        log: Log,
    ) -> Result<Words> {
        let common = match game {
            None => crate::core::keyfile::data_path()?,
            Some(game) => game.parent().map(Path::to_path_buf).unwrap_or_default(),
        }
        .join(COMMON);
        let folders: Vec<PathBuf> = match game {
            None => vec![common],
            Some(game) => vec![game.to_path_buf(), common],
        };
        let functions = Rc::new(functions);
        let mut bundles = Vec::new();
        let mut ids = Vec::new();
        let mut codes = vec![language.to_string()];
        if language != ENGLISH {
            codes.push(ENGLISH.to_string());
        }
        for code in codes {
            let locale: LanguageIdentifier = code.parse().unwrap_or_default();
            let mut bundle = FluentBundle::new(vec![locale]);
            bundle.set_use_isolating(false);
            for name in functions.keys() {
                let called = name.clone();
                let _ = bundle.add_function(name, move |positional, _named| {
                    trampoline(&called, positional)
                });
            }
            let mut held: Vec<String> = Vec::new();
            for folder in &folders {
                for path in ftl_files(&folder.join(&code))? {
                    let text = std::fs::read_to_string(&path)?;
                    let resource = match FluentResource::try_new(text) {
                        Ok(resource) => resource,
                        Err((resource, _junk)) => resource,
                    };
                    held.extend(resource.entries().filter_map(|entry| match entry {
                        ast::Entry::Message(message) => Some(message.id.name.to_string()),
                        _ => None,
                    }));
                    // A message a file before this one has keeps that one's words, as Python's
                    // `add_resource` keeps the first.
                    let _ = bundle.add_resource(resource);
                }
            }
            bundles.push(bundle);
            ids.push(held);
        }
        Ok(Words {
            language: language.to_string(),
            game: game.map(Path::to_path_buf),
            log,
            made: RefCell::new(HashSet::new()),
            bundles,
            ids,
            functions,
        })
    }

    /// Every game's English, as Python's `Words()`.
    pub fn english() -> Result<Words> {
        Words::new(ENGLISH, None, IndexMap::new(), Rc::new(|_line: &str| {}))
    }

    /// The message in the player's language, or English, with `values` in it; None where
    /// neither has it, or where a function it calls finds nothing (a scene naming a Pokémon the
    /// save has not kept).
    pub fn get(&self, message: &str, values: Values<'_>) -> Option<String> {
        for bundle in &self.bundles {
            let Some(found) = bundle.get_message(message) else {
                continue;
            };
            let pattern = found.value()?;
            let mut args = FluentArgs::new();
            for (name, value) in values {
                args.set(name.to_string(), fluent(value.value()));
            }
            // A function may ask for a message of its own, as YOURS does `scene-yours`: what the
            // message being formatted had found so far is put back after.
            let was = CALLING.with(|calling| calling.replace(Some(self.functions.clone())));
            let outer = FAILED.with(|failed| failed.replace(false));
            let mut errors = Vec::new();
            let said = bundle
                .format_pattern(pattern, Some(&args), &mut errors)
                .into_owned();
            let broke = FAILED.with(|failed| failed.replace(outer));
            CALLING.with(|calling| calling.replace(was));
            return if errors.is_empty() && !broke {
                Some(said)
            } else {
                None
            };
        }
        None
    }

    /// These words again, their messages able to call `functions` by name.
    pub fn with_functions(&self, functions: IndexMap<String, Function>) -> Result<Words> {
        Words::new(
            &self.language,
            self.game.as_deref(),
            functions,
            self.log.clone(),
        )
    }

    /// Every message whose id starts with `prefix`, by the rest of its id, in the player's
    /// language and English where it has none: a game's `sprite-0x118`, "Mom", is `0x118`.
    pub fn prefixed(&self, prefix: &str) -> IndexMap<String, String> {
        let mut found = IndexMap::new();
        for ids in self.ids.iter().rev() {
            for message in ids {
                if let Some(rest) = message.strip_prefix(prefix)
                    && let Some(said) = self.get(message, &[])
                {
                    found.insert(rest.to_string(), said);
                }
            }
        }
        found
    }

    /// Python's `__call__`: the message, or its id where no language has it.
    pub fn say(&self, message: &str, values: Values<'_>) -> String {
        self.get(message, values)
            .unwrap_or_else(|| message.to_string())
    }

    /// Log, once, a word made from a name rather than read from the data, as a texture's
    /// name makes a picture's: what the English table wants an entry for.
    pub fn made(&self, what: &str, said: &str) {
        if self.made.borrow_mut().insert(what.to_string()) {
            (self.log)(&format!(
                "no word in data for {what}; said {}",
                py::repr(said)
            ));
        }
    }
}

/// A name's function, called from its bundle with the values Fluent gives it.
fn trampoline<'a>(name: &str, positional: &[FluentValue<'a>]) -> FluentValue<'a> {
    let function = CALLING.with(|calling| {
        calling
            .borrow()
            .as_ref()
            .and_then(|functions| functions.get(name).cloned())
    });
    let values: Vec<Value> = positional
        .iter()
        .map(|value| match value {
            FluentValue::Number(number) => Value::Number(number.value),
            FluentValue::String(text) => Value::Text(text.to_string()),
            _ => Value::Text(String::new()),
        })
        .collect();
    match function.and_then(|function| function(&values)) {
        Some(said) => FluentValue::String(Cow::Owned(said)),
        None => {
            FAILED.with(|failed| failed.set(true));
            FluentValue::Error
        }
    }
}

/// A value as Fluent should insert it: a number without its digits grouped, which the Rust
/// library never groups, and anything else as text. A number keeps the library's default options:
/// it compares a number with a variant's number key, `[0]`, options and all, and a key's are the
/// defaults.
fn fluent(value: Value) -> FluentValue<'static> {
    match value {
        Value::Text(text) => FluentValue::String(Cow::Owned(text)),
        Value::Number(number) => {
            FluentValue::Number(FluentNumber::new(number, FluentNumberOptions::default()))
        }
    }
}

/// `sorted((folder / code).glob("*.ftl"))`: none where the folder is not there.
fn ftl_files(folder: &Path) -> Result<Vec<PathBuf>> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Ok(Vec::new());
    };
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "ftl") && path.is_file() {
            files.push(path);
        }
    }
    // Python sorts the paths as Windows compares them, its case aside.
    files.sort_by_key(|path| py::path_str(path).to_lowercase());
    Ok(files)
}

/// A word that may be pa3ds's own or the game's, as it is said: pa3ds's in the player's
/// language, `kind-stairs-up` for "stairs up", or the word itself where no message has it, the
/// game's "PC" and the data's "trash can" already being the player's.
pub fn said(prefix: &str, word: &str) -> String {
    current()
        .get(&format!("{prefix}-{}", slug(word)), &[])
        .filter(|said| !said.is_empty())
        .unwrap_or_else(|| word.to_string())
}

/// A word as a message's id spells it: "hidden Pokémon" is `hidden-Pokemon`. Its case is
/// kept, so the game's "Waterfall", a move's name, is never taken for pa3ds's own "waterfall".
pub fn slug(word: &str) -> String {
    let bare: String = word.nfkd().filter(char::is_ascii).collect();
    py::split(&bare).join("-")
}

thread_local! {
    /// The words in place. Unset, a test reads the words `tests/conftest.py` puts in place for
    /// every test, Omega Ruby's English; out of a test, every game's English.
    static PLACED: RefCell<Option<Option<Rc<Words>>>> = const { RefCell::new(None) };
}

/// Put a game's words in place: every reader asks them from here on. None puts back every
/// game's English, read again when next asked.
pub fn r#use(words: Option<Rc<Words>>) {
    *LAST
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = words
        .as_ref()
        .map(|words| (words.language.clone(), words.game.clone()));
    PLACED.with(|placed| *placed.borrow_mut() = Some(words));
}

/// The language and the game's folder of the words put in place last, by any thread. The words
/// in place are a thread's own, where Python's are the module's, every thread's alike: a window
/// opened on its own thread puts the same in place there (`placed_last`).
static LAST: std::sync::Mutex<Option<(String, Option<PathBuf>)>> = std::sync::Mutex::new(None);

/// The words put in place last, by whichever thread, made again for this one: none where every
/// game's English was put back, or none was put at all. Without the functions a message may call,
/// which close over the thread that put them in place, and logging nothing.
pub fn placed_last() -> Result<Option<Rc<Words>>> {
    let last = LAST
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let Some((language, game)) = last else {
        return Ok(None);
    };
    Ok(Some(Rc::new(Words::new(
        &language,
        game.as_deref(),
        IndexMap::new(),
        Rc::new(|_line: &str| {}),
    )?)))
}

/// The words in place, or every game's English where none has been put.
pub fn current() -> Rc<Words> {
    if let Some(Some(words)) = PLACED.with(|placed| placed.borrow().clone()) {
        return words;
    }
    let unset = PLACED.with(|placed| placed.borrow().is_none());
    let words = Rc::new(if unset { default_words() } else { english() });
    PLACED.with(|placed| *placed.borrow_mut() = Some(Some(words.clone())));
    words
}

#[cfg(not(test))]
fn default_words() -> Words {
    english()
}

#[cfg(test)]
fn default_words() -> Words {
    crate::tests::conftest::english()
}

/// Every game's English, or none where the data folder cannot be read, as a missing folder holds
/// no message.
fn english() -> Words {
    Words::english().unwrap_or_else(|_| Words {
        language: ENGLISH.to_string(),
        game: None,
        log: Rc::new(|_line: &str| {}),
        made: RefCell::new(HashSet::new()),
        bundles: Vec::new(),
        ids: Vec::new(),
        functions: Rc::new(IndexMap::new()),
    })
}

/// One of pa3ds's own messages, as it is said in the words in place.
pub fn own(message: &str, values: Values<'_>) -> String {
    current().say(message, values)
}
