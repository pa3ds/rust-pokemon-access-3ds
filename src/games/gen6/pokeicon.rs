//! The icons of Pokémon a screen loads to draw: a `common::resource::PokeIcon` and the textures
//! its `app::tool::LytTexReplaceSystem` holds, an icon each (spikes 177 and 178).
//!
//! The icons are stored with a palette in the ROM and expanded as they load, so no file holds the
//! image a picture draws and the texture index cannot name it (spikes 176 and 177): a picture draws
//! the n-th texture's image, and whoever loaded the icons knows whose the n-th is. ORAS's AreaNav's
//! Pokémon Distribution and DexNav, the Bag's party, Pokémon-Amie's page and Super Training's are
//! named so.
//!
//! ```text
//! PokeIcon             +0x0C  -> app::tool::LytTexReplaceSystem
//! LytTexReplaceSystem  +0x04  -> its textures, a pointer each; +0x08 how many
//! app::tool::LytTexData +0x0C  the image's physical address, as a picture's material keeps it
//! ```

pub const ICON_CLASS: &str = "N6common8resource8PokeIconE";
pub const REPLACE: i64 = 0x0C;
pub const REPLACE_CLASS: &str = "N3app4tool19LytTexReplaceSystemE";
pub const TEXTURES: i64 = 0x04;
pub const TEXTURE_COUNT: i64 = 0x08;
pub const IMAGE: i64 = 0x0C;
pub const MAX_TEXTURES: i64 = 32;
