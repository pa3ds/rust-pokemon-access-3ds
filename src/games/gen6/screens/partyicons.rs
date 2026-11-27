//! The Bag's party slots, named by the species whose icon each draws (spike 184).
//!
//! The Bag lists the party beside its items, each slot an icon, a level and an HP bar, and no name:
//! touch mode said "L 14" (movie21). The icons come from a `common::resource::PokeIcon`, as
//! the AreaNav's Pokémon Distribution's do (`gen6.pokeicon`): its `LytTexReplaceSystem` holds a
//! texture a party member, in the party's order, and each slot's picture draws its member's. The
//! party is a list of six `pml::pokepara::PokemonParam` pointers and then how many are in it, with
//! no class of its own:
//!
//! ```text
//! party     +0x00  six pointers to PokemonParams    +0x18 u8 how many are in the party
//! PokeIcon  +0x0C  -> LytTexReplaceSystem: +0x04 its textures, +0x08 how many
//! texture   +0x0C  the image's physical address, as a picture's material keeps it
//! ```
//!
//! The count is a byte: the Battle Maison sets the one after it to 2, and the party read as a u32
//! of 518 was no party. Its screen choosing the Battle Box or the party draws both on the top
//! screen from one PokeIcon of twelve textures, the Battle Box's six and then the party's, each
//! slot a level and an icon: "L 73 L 71" (or_multi_battle_partners).
//!
//! There the Maison keeps five more lists of six, one the Battle Box's, counting its two, which
//! was taken for the party: the party's first two were said as the Battle Box's (or_battle_box).
//! The party's list is the one whose members are the save's party, `savedata::PokePartySave`, by
//! each record's encryption constant; the Battle Box's icons are named from `savedata::BattleBox`:
//!
//! ```text
//! PokePartySave  +0x04  six records 0x104 apart, then +0x61C the count, a byte
//! BattleBox      +0x04  six records 0xE8 apart
//! ```
//!
//! Before a link battle the party list's top screen draws the other player's team as icons, a
//! gender sign and "L50" each, and no name: "zen, L50, , L50, , L50" (the PSS between two
//! players, spike 319). Its `app::pokelist::SubDrawer` draws them, as the decompile shows (spike
//! 318, the same in Omega Ruby and X), from what the list was opened with, which the list's
//! process copies into itself:
//!
//! ```text
//! pokelist::Proc  +0x38  what it was opened with, 0x54 bytes:
//!                        +0x08, +0x0C, +0x10 a team each, as the party's list
//! ```
//!
//! One team against the player's, the first empty, is the second, drawn in the first six slots; a
//! multi battle's three are the three in order, six slots each. The process is known as it starts
//! (`RunningApp.head`), where the sweep that finds the SubDrawer came two seconds after the screen
//! was said, and its names were said apart from it. A slot is the n-th icon picture in
//! `pokelist_uppwer.bclyt`: `DllPokeList`'s table of each slot's panes counts up the layout's
//! panes in their order.
//!
//! Only while the Bag, that screen or the party list runs: another screen's icons are not the
//! party's in its order.

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::binary::u32;
use crate::core::memory::Memory;
use crate::core::rtti::ClassNames;
use crate::games::common::layout::Layouts;
use crate::games::common::screen::ScreenText;
use crate::games::common::touch::FileOf;
use crate::games::common::{lyt, pokemon};
use crate::games::gen6::pokeicon;
use crate::py::{self, PySet, Result};

pub const BAG: &str = "N3app3bag4ProcE";
pub const GROUPS: &str = "N3app19battle_party_select4ProcE";
pub const LIST: &str = "N3app8pokelist4ProcE";
/// Where the party list's process keeps what it was opened with.
pub const OPENED_WITH: i64 = 0x38;
/// Where that holds the teams its top screen draws, each a list as the party's: with none at the
/// first, one, at the second, in the first six slots; else all three, six slots each.
pub const TEAMS: [i64; 3] = [0x08, 0x0C, 0x10];
pub const TOP: &str = "pokelist_uppwer.bclyt";
/// The party list's top screen, and the texture its slots' icon pictures are made with.
pub const ICON: &str = "poke_icon.bclim";
/// The game font's gender signs, ♂ and ♀ (`speech.SYMBOLS`).
pub const SIGNS: [&str; 2] = ["\u{E08E}", "\u{E08F}"];
pub const ICON_CLASS: &str = pokeicon::ICON_CLASS;
pub const SLOTS: i64 = 6;

/// App -> the texture holding the party's first member's icon.
pub const FIRST: [(&str, i64); 2] = [(BAG, 0), (GROUPS, SLOTS)];

/// `FIRST.get(app)`.
fn first(app: &str) -> Option<i64> {
    FIRST
        .iter()
        .find(|(running, _)| *running == app)
        .map(|&(_, first)| first)
}
pub const COUNT: i64 = 0x18;
pub const PARTY_SAVE: &str = "N8savedata13PokePartySaveE";
pub const BATTLE_BOX: &str = "N8savedata9BattleBoxE";
/// Where each save object's records start.
pub const RECORDS: i64 = 0x04;
/// How far apart the save's party's records are: a record and the stats a party member has.
pub const SAVED: i64 = 0x104;

/// The party's list among the addresses holding a PokemonParam: six pointers to them, the
/// first `COUNT` of them filled, then the count. Of several, the one whose members' encryption
/// constants are `saved`, the save's party's, in order.
pub fn party_of(
    holders: impl IntoIterator<Item = i64>,
    memory: &dyn Memory,
    names: &ClassNames,
    saved: &[i64],
) -> Result<Option<i64>> {
    let mut found: Vec<i64> = Vec::new();
    for at in holders {
        let head = memory.read(at, COUNT + 4)?;
        let count = head[COUNT as usize] as i64;
        if !(0 < count && count <= SLOTS) {
            continue;
        }
        let mut all = true;
        for i in 0..SLOTS {
            if names.of_object(u32(&head, 4 * i)?)?.as_deref() != Some(pokemon::PARAM_CLASS) {
                all = false;
                break;
            }
        }
        if all {
            if !saved.is_empty() {
                let mut constants: Vec<i64> = Vec::new();
                for i in 0..count {
                    constants.push(u32(
                        &memory.read(u32(&head, 4 * i)? + pokemon::RECORD_IN_PARAM, 4)?,
                        0,
                    )?);
                }
                if constants == saved {
                    return Ok(Some(at));
                }
            }
            found.push(at);
        }
    }
    Ok(found.first().copied())
}

/// The teams the party list's process was opened with, out of its first bytes
/// (`RunningApp.head`), in the order its top screen draws them, six slots each; none out of a
/// link battle. Asked only while the party list runs: another process holds anything there.
pub fn teams(process: &[u8]) -> Result<Vec<i64>> {
    let last = TEAMS[TEAMS.len() - 1];
    let opened = py::slice(process, OPENED_WITH, OPENED_WITH + last + 4);
    if (opened.len() as i64) < last + 4 {
        return Ok(Vec::new());
    }
    let mut listed: Vec<i64> = Vec::new();
    for at in TEAMS {
        listed.push(u32(opened, at)?);
    }
    if listed[0] == 0 {
        listed = listed[1..2].to_vec();
    }
    Ok(if listed.iter().any(|&team| team != 0) {
        listed
    } else {
        Vec::new()
    })
}

/// The encryption constants of the save's party's members, in order.
pub fn saved_party(memory: &dyn Memory, party: i64) -> Result<Vec<i64>> {
    let count = memory.read(party + RECORDS + SAVED * SLOTS, 1)?[0] as i64;
    let mut out = Vec::new();
    for i in 0..std::cmp::min(count, SLOTS) {
        out.push(u32(&memory.read(party + RECORDS + SAVED * i, 4)?, 0)?);
    }
    Ok(out)
}

/// Names a picture of the Bag's party by the species of the member whose icon it draws.
pub struct PartyIcons {
    memory: Rc<dyn Memory>,
    names: Rc<ClassNames>,
    species: Vec<String>,
    running: Rc<dyn Fn() -> Option<String>>,
    process: Rc<dyn Fn() -> Vec<u8>>,
    layouts: Option<Rc<Layouts>>,
    file_of: FileOf,
    /// The party's list, as last found: the Switch screen names its buttons by it.
    pub party: Option<i64>,
    battle_box: Option<i64>,
    icons: PySet,
    /// Image -> the species it draws, until the next sweep: screen text asks of every
    /// picture at every poll, and naming one reads the party member's record.
    named: RefCell<IndexMap<i64, Option<String>>>,
    /// A team and a slot in it -> its member's species, until the next sweep, as `_named`.
    members: RefCell<IndexMap<(i64, i64), Option<String>>>,
}

impl PartyIcons {
    /// `PartyIcons(memory, names, species, running)`: no process, layouts or files, so no
    /// party list's team.
    pub fn new(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        species: Vec<String>,
        running: Rc<dyn Fn() -> Option<String>>,
    ) -> Self {
        PartyIcons::with_list(
            memory,
            names,
            species,
            running,
            Rc::new(Vec::new),
            None,
            Rc::new(|_root| None),
        )
    }

    /// `process` is the running process's bytes from its start (`RunningApp.head`), and
    /// `layouts` and `file_of` say which of the party list's top screen's slots a picture is.
    pub fn with_list(
        memory: Rc<dyn Memory>,
        names: Rc<ClassNames>,
        species: Vec<String>,
        running: Rc<dyn Fn() -> Option<String>>,
        process: Rc<dyn Fn() -> Vec<u8>>,
        layouts: Option<Rc<Layouts>>,
        file_of: FileOf,
    ) -> Self {
        PartyIcons {
            memory,
            names,
            species,
            running,
            process,
            layouts,
            file_of,
            party: None,
            battle_box: None,
            icons: PySet::new(),
            named: RefCell::new(IndexMap::new()),
            members: RefCell::new(IndexMap::new()),
        }
    }

    /// What a sweep found: the PokemonParams, to find the party by, the PokeIcons, and the
    /// save's party and Battle Box.
    pub fn track(
        &mut self,
        params: &[i64],
        icons: impl IntoIterator<Item = i64>,
        holders: &dyn Fn(&[i64]) -> Result<Vec<i64>>,
        adding: bool,
        saves: impl IntoIterator<Item = i64>,
    ) -> Result<()> {
        self.icons = if adding {
            self.icons.union(&PySet::of(icons))
        } else {
            PySet::of(icons)
        };
        self.named.borrow_mut().clear();
        self.members.borrow_mut().clear();
        let mut saved: Vec<i64> = Vec::new();
        for save in saves {
            let kind = self.names.of_object(save)?;
            if kind.as_deref() == Some(PARTY_SAVE) {
                saved = saved_party(&*self.memory, save)?;
            } else if kind.as_deref() == Some(BATTLE_BOX) {
                self.battle_box = Some(save);
            }
        }
        if !params.is_empty() && (self.party.is_none() || !adding) {
            let found = party_of(holders(params)?, &*self.memory, &self.names, &saved)?;
            self.party = match found {
                Some(party) if party != 0 => Some(party),
                _ => self.party,
            };
        }
        Ok(())
    }

    /// The party's list, as last found.
    pub fn party(&self) -> Option<i64> {
        self.party
    }

    /// The save's Battle Box, as last found.
    pub fn battle_box(&self) -> Option<i64> {
        self.battle_box
    }

    /// The species of the party member a picture draws the icon of, or None.
    pub fn name(&self, picture: i64) -> Result<Option<String>> {
        let running = (self.running)();
        if self.species.is_empty() {
            return Ok(None);
        }
        if running.as_deref() == Some(LIST) {
            return self.team_drawn(picture);
        }
        if running.as_deref().and_then(first).is_none() || self.party.is_none() {
            return Ok(None);
        }
        let memory = &self.memory;
        let material = u32(&memory.read(picture + lyt::PICTURE_MATERIAL, 4)?, 0)?;
        if material == 0 {
            return Ok(None);
        }
        let image = u32(&memory.read(material + lyt::MATERIAL_TEXTURE, 4)?, 0)?;
        if !self.named.borrow().contains_key(&image) {
            let drawn = self.species_drawn(image)?;
            self.named.borrow_mut().insert(image, drawn);
        }
        Ok(self.named.borrow()[&image].clone())
    }

    /// The words on screen with the party list's top screen's gender signs each straight
    /// after its slot's species, as a battle says "Talonflame♂ L50": drawn right of the icon
    /// over the level, it was said after the level, "Talonflame, L50, ♂" (Alpha Sapphire, live).
    pub fn reworded(&self, found: IndexMap<i64, ScreenText>) -> IndexMap<i64, ScreenText> {
        if (self.running)().as_deref() != Some(LIST) {
            return found;
        }
        let mut named: IndexMap<i64, i64> = IndexMap::new();
        for (&r#box, piece) in &found {
            if piece.picture
                && !piece.text.is_empty()
                && (self.file_of)(piece.root).is_some_and(|file| file.name == TOP)
            {
                named.insert(piece.parent, r#box);
            }
        }
        found
            .into_iter()
            .map(|(r#box, piece)| {
                if SIGNS.contains(&py::strip(&piece.text))
                    && let Some(&follows) = named.get(&piece.parent)
                    && !piece.picture
                {
                    (r#box, ScreenText { follows, ..piece })
                } else {
                    (r#box, piece)
                }
            })
            .collect()
    }

    /// The species of the party member whose icon is drawn from this image, or None.
    fn species_drawn(&self, image: i64) -> Result<Option<String>> {
        let Some(party_at) = self.party else {
            return Ok(None);
        };
        for icon in self.icons.iter() {
            let Some(slot) = self.slot(icon, image)? else {
                continue;
            };
            let first = first((self.running)().as_deref().unwrap_or("")).unwrap_or(0);
            if slot < first {
                return self.boxed(slot);
            }
            return self.member(party_at, slot - first);
        }
        Ok(None)
    }

    /// The species of the team member whose icon this picture of the party list's top screen
    /// draws, or None: another player's team before a link battle, drawn as icons alone. Out of
    /// a link battle the list was opened with no team, and nothing is read.
    fn team_drawn(&self, picture: i64) -> Result<Option<String>> {
        let against = teams(&(self.process)())?;
        if against.is_empty() {
            return Ok(None);
        }
        let Some(slot) = self.slot_drawn(picture)? else {
            return Ok(None);
        };
        let (team, member) = (py::floordiv(slot, SLOTS), py::modulo(slot, SLOTS));
        if team >= against.len() as i64 {
            return Ok(None);
        }
        let key = (against[team as usize], member);
        if !self.members.borrow().contains_key(&key) {
            let found = self.member(key.0, member)?;
            self.members.borrow_mut().insert(key, found);
        }
        Ok(self.members.borrow()[&key].clone())
    }

    /// Which of the party list's top screen's slots a picture is the icon of, or None.
    fn slot_drawn(&self, picture: i64) -> Result<Option<i64>> {
        let Some(layouts) = &self.layouts else {
            return Ok(None);
        };
        let Some(placement) = layouts.placement(picture)? else {
            return Ok(None);
        };
        let file = (self.file_of)(placement.root);
        let Some(file) = file.filter(|file| file.name == TOP) else {
            return Ok(None);
        };
        let mut icons: Vec<i64> = Vec::new();
        for (pane, kind) in layouts.tree(placement.root) {
            if kind == lyt::PICTURE_CLASS
                && file
                    .textures
                    .get(&layouts.pane_name(pane)?)
                    .is_some_and(|textures| textures.iter().any(|texture| texture == ICON))
            {
                icons.push(pane);
            }
        }
        Ok(icons
            .iter()
            .position(|&icon| icon == picture)
            .map(|slot| slot as i64))
    }

    /// Which of a PokeIcon's textures is this image, or None where it is none of them, or the
    /// icon no PokeIcon.
    fn slot(&self, icon: i64, image: i64) -> Result<Option<i64>> {
        let memory = &self.memory;
        if icon == 0 || self.names.of_object(icon)?.as_deref() != Some(ICON_CLASS) {
            return Ok(None);
        }
        let replace = u32(&memory.read(icon + pokeicon::REPLACE, 4)?, 0)?;
        if replace == 0
            || self.names.of_object(replace)?.as_deref() != Some(pokeicon::REPLACE_CLASS)
        {
            return Ok(None);
        }
        let head = memory.read(replace + pokeicon::TEXTURES, 8)?;
        let (listed, count) = (u32(&head, 0)?, u32(&head, 4)?);
        if listed == 0 || !(0 < count && count <= pokeicon::MAX_TEXTURES) {
            return Ok(None);
        }
        let table = memory.read(listed, 4 * count)?;
        for i in 0..count {
            if u32(&memory.read(u32(&table, 4 * i)? + pokeicon::IMAGE, 4)?, 0)? == image {
                return Ok(Some(i));
            }
        }
        Ok(None)
    }

    /// The species of a party list's member, or None past its count.
    fn member(&self, party: i64, slot: i64) -> Result<Option<String>> {
        if party == 0 {
            return Ok(None);
        }
        let memory = &self.memory;
        let head = memory.read(party, COUNT + 4)?;
        // The list found in an earlier Bag may be another object's now, counting past six.
        if !(0 <= slot && slot < std::cmp::min(head[COUNT as usize] as i64, SLOTS)) {
            return Ok(None);
        }
        let found = pokemon::read(
            &**memory,
            &self.names,
            u32(&head, 4 * slot)?,
            pokemon::SPECIES_MAX,
        )?;
        let Some(found) = found else {
            return Ok(None);
        };
        if found.species >= self.species.len() as i64 {
            return Ok(None);
        }
        Ok(Some(self.species[found.species as usize].clone()).filter(|name| !name.is_empty()))
    }

    /// The species of the Battle Box's member in this slot, or None.
    fn boxed(&self, slot: i64) -> Result<Option<String>> {
        let Some(r#box) = self.battle_box else {
            return Ok(None);
        };
        if self.names.of_object(r#box)?.as_deref() != Some(BATTLE_BOX) {
            return Ok(None);
        }
        let found = pokemon::stored(
            &*self.memory,
            r#box + RECORDS + pokemon::RECORD_SIZE * slot,
            pokemon::SPECIES_MAX,
        )?;
        let Some(found) = found else {
            return Ok(None);
        };
        if found.species >= self.species.len() as i64 {
            return Ok(None);
        }
        Ok(Some(self.species[found.species as usize].clone()).filter(|name| !name.is_empty()))
    }
}
