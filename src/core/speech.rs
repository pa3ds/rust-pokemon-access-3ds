//! Speech: what gets said, and whether it cuts off what is already being said.
//!
//! The NVDA backend cannot report whether it is still speaking (spike 1), so pa3ds never waits for
//! speech to finish. NVDA queues on its own when `interrupt` is false; all pa3ds decides is the
//! interrupt flag, per speech category. Nothing interrupts by default: a screen reader already cuts
//! speech when a key is pressed in the game's window, and pa3ds interrupting itself cut an item under
//! the cursor off with the description the same move brought up (the user, 2026-09-14). Categories
//! can be made to interrupt, as `[pa3ds.ui] interrupt` does for menus.
//!
//! Everything a reader reports is said, identical or not. Readers report changes, not states, so
//! polling the same state again says nothing; and when the game shows the same words again — a second
//! conversation with the same person — a sighted player sees them again, so they are heard again.
//! This reverses an earlier "identical text is never repeated" rule, which silenced exactly that
//! (the user, 2026-09-13).
//!
//! Bound by:
//! - ADR-0010: nothing interrupts by default: a key's answer and a warning do, and menus under
//!   `interrupt`.
//! - ADR-0020: speech goes to the player's own screen reader through Prism; braille takes the raw
//!   string.
//! - ADR-0021: the game's lines are dropped while another window is in front; pa3ds's own and a
//!   key's answer are said anywhere.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::py::{self, Error, Kind, Result};

/// What a piece of speech is. Decides whether it interrupts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Menu,
    /// A list opening, such as Yes/No. It queues, so it does not cut off the question before it.
    Prompt,
    /// Words a screen draws outside any message window as it comes up: a menu's labels, a question
    /// over a list. It queues, so a screen that opens after a message does not cut the message off.
    Screen,
    Dialogue,
    Battle,
    System,
    /// Something the player must do before pa3ds can work, said as it starts: close Azahar so its
    /// settings can be written. It interrupts and is said whatever window is in front, so it is not
    /// left queued behind the console being read out (Orion, 2026-10-05).
    Warning,
    /// The answer to a key the player pressed. It interrupts: the player asked for it now, and
    /// the key never reaches the game's window, so the screen reader has not cut speech for it.
    Key,
    /// The next leg of a path, said as the player walks up to each turn. It queues: two turns a
    /// step apart are both heard, the second after the first.
    Guidance,
    /// A description of what the game shows without words (`games.common.scenes`). It queues, so
    /// the line a scene follows is heard whole first.
    Scene,
}

impl Category {
    pub const ALL: [Category; 10] = [
        Category::Menu,
        Category::Prompt,
        Category::Screen,
        Category::Dialogue,
        Category::Battle,
        Category::System,
        Category::Warning,
        Category::Key,
        Category::Guidance,
        Category::Scene,
    ];

    /// The category's value, as the settings file names it.
    pub fn value(self) -> &'static str {
        match self {
            Category::Menu => "menu",
            Category::Prompt => "prompt",
            Category::Screen => "screen",
            Category::Dialogue => "dialogue",
            Category::Battle => "battle",
            Category::System => "system",
            Category::Warning => "warning",
            Category::Key => "key",
            Category::Guidance => "guidance",
            Category::Scene => "scene",
        }
    }
}

pub fn default_interrupts() -> HashSet<Category> {
    HashSet::from([Category::Key, Category::Warning])
}

/// Said whether or not the player is in the game's window (ADR-0021): pa3ds's own few lines
/// ("pa3ds ready", a reader failing), said while the player may still be in the window they started
/// pa3ds from, and the answer to a key they pressed, which only comes while the game is in front.
pub const HEARD_ANYWHERE: [Category; 3] = [Category::System, Category::Warning, Category::Key];

/// The game font's private-use gender glyphs, as the Unicode signs they draw. A screen reader skips
/// a private-use character but says ♂ and ♀ in the player's own words (the user, 2026-09-14). U+E08E
/// was read beside Torchic's ♂ on the party screen (spike 32); U+E08F, the ♀ beside it in the font,
/// has not been seen in a replay yet.
pub fn symbol(c: char) -> char {
    match c {
        '\u{e08e}' => '♂',
        '\u{e08f}' => '♀',
        c => c,
    }
}

/// Where speech goes: the part of a Prism backend pa3ds uses.
pub trait Backend {
    fn speak(&self, text: &str, interrupt: bool);
    fn braille(&self, text: &str);
}

/// Whether the player is in the game's window.
pub type Listening = Rc<dyn Fn() -> bool>;

/// Says things through one backend, applying the interrupt policy.
pub struct Speech {
    backend: Rc<dyn Backend>,
    interrupts: HashSet<Category>,
    listening: Option<Listening>,
    /// Lines spoken so far of each category, so a speaker can be named again after a screen's
    /// words come between their lines.
    spoken: RefCell<HashMap<Category, i64>>,
}

impl Speech {
    /// `listening` says whether the player is in the game's window; while it is false only
    /// `HEARD_ANYWHERE` is said, so reading another window is never talked over (ADR-0021).
    /// None says everything, as a replay, whose emulator is never in front, wants.
    pub fn new(
        backend: Rc<dyn Backend>,
        interrupts: HashSet<Category>,
        listening: Option<Listening>,
    ) -> Self {
        Speech {
            backend,
            interrupts,
            listening,
            spoken: RefCell::new(HashMap::new()),
        }
    }

    /// `spoken[category]`: the lines of a category spoken so far.
    pub fn spoken(&self, category: Category) -> i64 {
        self.spoken.borrow().get(&category).copied().unwrap_or(0)
    }

    /// Speech with the default interrupts, heard in any window.
    pub fn plain(backend: Rc<dyn Backend>) -> Self {
        Speech::new(backend, default_interrupts(), None)
    }

    /// Speak `text` unless there is nothing in it, or the player is in another window.
    /// Returns whether it was spoken.
    ///
    /// Whitespace is collapsed first: a line break in game text is where a box ran out of
    /// width, and a screen reader pauses on it as though a sentence ended.
    pub fn say(&self, text: &str, category: Category) -> bool {
        let translated: String = text.chars().map(symbol).collect();
        let spoken = py::split(&translated).join(" ");
        if spoken.is_empty() {
            return false;
        }
        if let Some(listening) = &self.listening
            && !HEARD_ANYWHERE.contains(&category)
            && !listening()
        {
            return false;
        }
        self.backend
            .speak(&spoken, self.interrupts.contains(&category));
        self.backend.braille(&spoken);
        *self.spoken.borrow_mut().entry(category).or_insert(0) += 1;
        true
    }
}

/// The best backend Prism can find on this machine, and its name.
///
/// Prism is built into pa3ds (`prismer`'s static build), so there is no library to look for: the
/// context is started here, at the first call, and kept for the rest of the run, since the backend
/// borrows from it and is said through until pa3ds stops.
pub fn screen_reader() -> Result<(Rc<dyn Backend>, String)> {
    let failed = |what: &str, error: prismer::Error| {
        Error::new(Kind::RuntimeError, format!("prism {what}: {error}"))
    };
    let context: &'static prismer::Prism = Box::leak(Box::new(
        prismer::Prism::new().map_err(|error| failed("could not start", error))?,
    ));
    let backend = context
        .acquire_best()
        .map_err(|error| failed("found no backend", error))?;
    let name = backend.name();
    Ok((Rc::new(Prism(backend)), name))
}

/// A Prism backend, as pa3ds says through it.
struct Prism(prismer::Backend<'static>);

impl Backend for Prism {
    fn speak(&self, text: &str, interrupt: bool) {
        let _ = self.0.speak(text, interrupt);
    }

    fn braille(&self, text: &str) {
        let _ = self.0.braille(text);
    }
}
