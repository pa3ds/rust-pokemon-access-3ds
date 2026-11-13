//! `field::mmodel::MoveModel`: one slot of the overworld's entity list.
//!
//! ```text
//! +0x04 u32 zero while the slot is free; for the player, bit 0x200000 (+0x06's 0x20) is set
//!           while they ride the Mach Bike (spike 158) or a Pokémon on the sea (spike 227);
//!           bit 0x04 is set on a model the game has hidden, with 0x100 beside it each time
//!           (`hidden`, spike 294)
//! +0x08 u8  bit 0x20 set on a model built and not yet set up, for a poll as a zone loads
//!           (`just_built`)
//! +0x0c u32 zone << 16 | the entity index in that zone's table; the player is index 0xff
//! +0x10 u32 flag << 16 | character — who or what it is: Norman 0x13f, Wally 0xb4, Birch 0x13e,
//!           and 0x2000 for a Poké Ball lying on the ground
//! +0x1c f32 and +0x24 f32 the sine and cosine of the way it faces; on a rail, of the line's
//!           own frame, across and along (spike 103)
//! +0x94 f32 x and +0x9c f32 z, the cell it is standing in, on the grid; on a rail, as on
//!           Lumiose's boulevards, +0x94 is the line's id and +0x9c the cells along it, and
//!           +0xf8 f32 the offset across it in world units (spike 103), both measured by the
//!           line's way as a record placed on it is (`rail.Line.upright`, spike 107)
//! +0xa4 f32 and +0xac f32: on a rail, the sine and cosine of the way the line runs in the
//!           world where the model stands, so the line's frame turned into the world's; X's
//!           (spike 105)
//! +0xec f32 x, +0xf0 y, +0xf4 z: where it is in the world on the grid, as +0x114 has it; on a
//!           rail the rail's own coordinates, the distance along it in world units at +0xf4
//! +0x114 f32 x, +0x118 y, +0x11c z: where it is in the world, 18 units a cell, the cell's
//!           centre at `cell * 18 + 9`, on the grid and on a rail alike (spike 102)
//! +0x150 u8 width, u8 depth: the cells it stands in, east and west, north and south, as it
//!           faces south: 1 and 1 for a person, 2 and 2 for the boulder Strength pushes, 3 and
//!           1 for Mauville Gym's barrier, whichever way it faces. Copied from the ROM's record
//!           for its sprite, which the slot keeps at +0x884; X's are at +0x150 too
//!           (omega_ruby_strength-boulders.ctm, 2026-10-01)
//! +0x898 u32 the sprite it is drawn with: the character again for most, and for a work object
//!           the sprite a script filled it with (spike 89); +0x890 in X and Y (spike 100)
//! +0xa2c u8 for the player, what they move as: 1 on foot, 2 on the bike, 0 on a Pokémon on
//!           the sea or under it (spike 227)
//! ```
//!
//! A sweep for the class finds **32 of them, 0xab0 apart**: a fixed pool the game fills as a zone
//! loads and empties as it leaves, not a list that grows (spike 75). X's are 0xaa8 apart, eight
//! bytes smaller, with everything up to the cell where ORAS has it and the drawn sprite eight
//! bytes sooner; the work objects are the same sixteen ids in both games' sprite lists
//! (spike 100). The zone is the one the model was built in, so walking into the next outdoor zone
//! keeps the model (spike 78).
//!
//! The character holds for a story character and is shared among the nameless. Sixteen ids,
//! `0x7a` to `0x89`, are **work objects**: a slot a script fills with whoever the scene needs,
//! and +0x10 keeps the slot's id, so a name given to it would go to whoever fills it next
//! (spike 88). +0x898 says who did: the rival's model reads `0x7a` at +0x10 in both openings and
//! `0x13a SUPPORT_F_RS` at +0x898 as May, `0x13b SUPPORT_M_RS` as Brendan (spike 89). So a work
//! object is the character it was filled with, and every other model is its +0x10, as it always
//! was. What to call one is `pa3ds.core.names`: the game has no word for anybody on its maps but
//! the rival, whom the ROM names (`archives`).

use std::ops::Range;

use crate::core::binary::{f32, u32};
use crate::core::memory::Memory;
use crate::py::{self, Error, Result};

pub const CLASS: &str = "N5field6mmodel9MoveModelE";
pub const CLASSES: [&str; 2] = [CLASS, "N5field6mmodel13MoveModelBaseE"];
/// How far apart ORAS's slots of the pool are; X's are `SLOT_X` apart.
pub const SLOT: i64 = 0xAB0;
pub const SLOT_X: i64 = 0xAA8;

pub const IN_USE: i64 = 0x04;
/// `IN_USE`'s bit on a model the game has hidden (`hidden`).
pub const HIDDEN: i64 = 0x04;
pub const SETUP: i64 = 0x08;
pub const NOT_SET_UP: i64 = 0x20;
pub const RIDING: i64 = 0x06;
pub const BIKE: i64 = 0x20;
pub const MOUNT: i64 = 0x07;
/// +0x07's bit on the usual ride on the water, which X's riders are told apart by (`on_bike`).
pub const SURFING: i64 = 0x10;
/// What the player moves as, a u8: 1 on foot, `FORM_BIKE` on the bike, 0 on a Pokémon on the
/// sea or under it, Gyarados, Kyogre or Sharpedo alike (spike 227).
pub const FORM: i64 = 0xA2C;
pub const FORM_BIKE: i64 = 2;
/// Which ride the player is on, a u8 (spike 227): on the bike, `MACH` on the Mach Bike and one of
/// `ACRO` on the Acro Bike, its tricks among them. Switching bikes over and over in the Safari
/// Zone, it went 3, 2, 3, 2 with `field::PlayerWork`'s +0x08 going 2, 1, 2, 1 (or_which_bike, spike
/// 271); 2 all through Jagged Pass on the Mach Bike
/// (or_jagged_pass_mach_bike), 3 to 6 through the Acro Bike's tricks (or_acro_bike_tricks).
pub const RIDE: i64 = 0x59;
pub const MACH: i64 = 2;
pub const ACRO: Range<i64> = 3..7;
pub const ZONE: i64 = 0x0C;
pub const CHARACTER: i64 = 0x10;
pub const CELL_X: i64 = 0x94;
pub const CELL_Z: i64 = 0x9C;
pub const FACING_SIN: i64 = 0x1C;
pub const FACING_COS: i64 = 0x24;
pub const LINE_SIN: i64 = 0xA4;
pub const LINE_COS: i64 = 0xAC;
pub const RAIL_LINE: i64 = 0x94;
pub const RAIL_ALONG: i64 = 0x9C;
pub const RAIL_ACROSS: i64 = 0xF8;
pub const PLACE_X: i64 = 0xEC;
pub const PLACE_Z: i64 = 0xF4;
pub const WORLD_X: i64 = 0x114;
pub const WORLD_Y: i64 = 0x118;
pub const WORLD_Z: i64 = 0x11C;
pub const SIZE: i64 = 0x150;
/// Enough of a slot to hold everything above.
pub const HEAD: i64 = 0x152;

pub const INDEX_MASK: i64 = 0xFFFF;
pub const PLAYER_INDEX: i64 = 0xFF;
/// World units per cell.
pub const CELL: i64 = 18;
/// World units the two places may differ by and still be one: they are copies on the grid.
pub const NEAR: f64 = 0.5;
/// Where ORAS keeps the sprite a model is drawn with; a game's own is `Game.drawn`.
pub const DRAWN: i64 = 0x898;
/// `WKOBJCODE00` to `WKOBJCODE15` in the ROM's sprite list (spike 88).
pub const WORK_OBJECTS: Range<i64> = 0x7A..0x8A;
/// `NONDRAW` in the ROM's sprite list: a model drawn as nothing, standing in for what the player
/// talks to there. ORAS stands 23 in nine zones: on each of Sootopolis City's ten locked doors,
/// and in front of Groudon deep in the Cave of Origin (spike 203).
pub const NONDRAW: i64 = 0x0;
/// The character of a Poké Ball lying on the ground, seen on Route 102 (spike 78).
pub const ITEM_BALL: i64 = 0x2000;
/// The characters of an item ball: `MONSBALL` and `MONSBALL_RARE`, which runs item balls'
/// scripts among `MONSBALL`'s in both games, 26 in X and 39 in ORAS, and on Route 6 was an
/// "object" (spikes 155 and 220, 2026-10-19).
///
/// Python's is a frozenset, only tested.
pub const ITEM_BALLS: [i64; 2] = [ITEM_BALL, 0x2001];
/// Model codes from here up to `PROP_MODEL` are Pokémon: the movers' Machoke `0x103f`.
pub const POKEMON_MODEL: i64 = 0x1000;
/// Model codes from here up are things, not people: a doll in the bedroom, a Poké Ball.
pub const PROP_MODEL: i64 = 0x2000;

/// `head[index]`: the byte, or the `IndexError` Python raises past the end of `bytes`.
#[track_caller]
fn byte(head: &[u8], index: i64) -> Result<i64> {
    match head.get(index as usize) {
        Some(&byte) => Ok(byte as i64),
        None => Err(Error::index("index out of range")),
    }
}

/// Who or what this slot is, or 0 when it is empty.
pub fn character(head: &[u8]) -> Result<i64> {
    Ok(u32(head, CHARACTER)? & INDEX_MASK)
}

/// Who or what this slot is, a work object as whoever it was filled with.
///
/// Only a work object reads further than the head: every other model's +0x10 is the sprite it
/// is drawn with, and reading `drawn` for all 32 would be a page a model for nothing. A work
/// object whose `drawn` does not hold a sprite yet, as while its looks load, is its own id.
/// X keeps a pointer where ORAS keeps the sprite, so `drawn` is the game's own. Where `drawn`
/// holds a model rather than a sprite, as X's does, `models` says where its things' and its
/// Pokémon's models start (`sprite_of`).
pub fn drawn_as(
    memory: &dyn Memory,
    model: i64,
    head: &[u8],
    drawn: i64,
    models: Option<(i64, i64)>,
) -> Result<i64> {
    let code = character(head)?;
    if !WORK_OBJECTS.contains(&code) {
        return Ok(code);
    }
    let filled = u32(&memory.read(model + drawn, 4)?, 0)? & INDEX_MASK;
    if filled == 0 || WORK_OBJECTS.contains(&filled) {
        return Ok(code);
    }
    Ok(match models {
        None => filled,
        Some(models) => sprite_of(filled, models),
    })
}

/// The sprite a model draws, given where the things' and the Pokémon's models start: a
/// person's model is its sprite, a thing's counts on from `PROP_MODEL`, a Pokémon's from
/// `POKEMON_MODEL` (spikes 218 and 221). X's Rhyhorn by the house is model `0x106`, sprite
/// `0x1014`.
pub fn sprite_of(model: i64, models: (i64, i64)) -> i64 {
    let (things, pokemon) = models;
    if model >= pokemon {
        return POKEMON_MODEL + model - pokemon;
    }
    if model >= things {
        PROP_MODEL + model - things
    } else {
        model
    }
}

/// The cells it stands in, as the north-west one and (width, height), laid round `cell`.
///
/// Its size is as it faces south, and turned a quarter when it faces east or west: a barrier,
/// 3 by 1, lies east and west facing south and north and south facing east (spike 146). An odd
/// side is centred on the model's cell, the barrier's beam one cell either side of it, and an
/// even one runs on east and south of it: Fiery Path's boulder stands at (35, 44) and the player
/// faced it from (37, 44) and (37, 45) (omega_ruby_strength-boulders.ctm). A side of 0, as
/// while a slot is being filled, is taken for 1.
pub fn footprint(head: &[u8], cell: (i64, i64)) -> Result<((i64, i64), (i64, i64))> {
    let mut width = byte(head, SIZE)?.max(1);
    let mut height = byte(head, SIZE + 1)?.max(1);
    if f32(head, FACING_SIN)?.abs() > f32(head, FACING_COS)?.abs() {
        (width, height) = (height, width);
    }
    Ok((
        (
            cell.0 - py::floordiv(width - 1, 2),
            cell.1 - py::floordiv(height - 1, 2),
        ),
        (width, height),
    ))
}

/// Whether the player is riding the bike, `form` the model's byte at `FORM`, or None in a
/// game it is not known in. In Jagged Pass +0x06 read 0x01 on foot and 0x21 from the moment the
/// player got on to the moment they got off, twice, the pictures showing them riding between,
/// and +0xa2c 1 to 2 with it (omega_ruby_bike.ctm, spike 158). Riding a Pokémon on the sea or
/// under it sets the same bit, and +0x07 tells only the usual ride on the water from the bike,
/// 0x13 where the bike has 0x03, and not Sharpedo, Kyogre or anyone under the water, who have
/// 0x03 too (spike 227); X's are told apart by it still, their +0xa2c not having been read.
pub fn on_bike(head: &[u8], form: Option<i64>) -> Result<bool> {
    let riding = byte(head, RIDING)? & BIKE != 0;
    match form {
        None => Ok(riding && byte(head, MOUNT)? & SURFING == 0),
        Some(form) => Ok(riding && form == FORM_BIKE),
    }
}

/// On the bike, whether it is the Acro Bike, not the Mach Bike (`RIDE`); None off the bike
/// and in a game where `form` is not known, X having one bike, its rides not read.
pub fn acro(head: &[u8], form: Option<i64>) -> Result<Option<bool>> {
    if form.is_none() || !on_bike(head, form)? {
        return Ok(None);
    }
    let ride = byte(head, RIDE)?;
    Ok(if ride == MACH {
        Some(false)
    } else if ACRO.contains(&ride) {
        Some(true)
    } else {
        None
    })
}

/// Whether the player rides a Pokémon, on the sea or under it (`on_bike`). On Routes 132 and
/// 133 +0x06 read 0x21 surfing and 0x01 once a current had carried the player onto a sandbar
/// (movie43, spike 208). Only the bike is ruled out, not a Pokémon's 0 asked for: just after a
/// save is loaded on the water, +0xa2c reads 1 until the ride's animation begins (spike 227).
pub fn surfing(head: &[u8], form: Option<i64>) -> Result<bool> {
    Ok(byte(head, RIDING)? & BIKE != 0 && !on_bike(head, form)?)
}

/// Which zone the slot was built in.
pub fn zone_of(head: &[u8]) -> Result<i64> {
    Ok(u32(head, ZONE)? >> 16)
}

/// Which entity of that zone's table it is; `PLAYER_INDEX` for the player.
pub fn index_of(head: &[u8]) -> Result<i64> {
    Ok(u32(head, ZONE)? & INDEX_MASK)
}

/// Whether the game has anything in this slot.
pub fn in_use(head: &[u8]) -> Result<bool> {
    Ok(u32(head, IN_USE)? != 0)
}

/// Whether the game has hidden the model: built, but not drawn, and walked through. The
/// Secret Base Guild in Fortree builds work objects round Aarune filled with the pals of the
/// player's own Secret Base, and hides them; drawn ones read `03 00 01 08` from +0x04, those
/// `07 01 01 09` (or_secret_base_guild_bug). So do the Vigoroth movers once the move is done,
/// Wally once he has left Petalburg Gym, and the Contest Spectacular Halls' unfilled slots
/// (spike 294). +0x07's 0x01 is no such bit: every model in Santalune Gym has it, Viola too.
pub fn hidden(head: &[u8]) -> Result<bool> {
    Ok(byte(head, IN_USE)? & HIDDEN != 0)
}

/// Whether the game has built the model and not yet set it up: true of every model of a zone
/// the player has just arrived in, theirs included, for the first poll there, `22 00` from
/// +0x08 where `02 40` is usual, and of none after (six tapes, 2026-11-13). Whether it is hidden
/// is not set yet: the Guild's pals read as drawn, and its list counted 13, then 9
/// (or_secret_base_guild_bug).
pub fn just_built(head: &[u8]) -> Result<bool> {
    Ok(byte(head, SETUP)? & NOT_SET_UP != 0)
}

/// Whether it is placed on a rail rather than the grid. On the grid +0xec is where it is in
/// the world, the same as +0x114 even mid-step; on a rail it is the rail's own coordinates, and
/// the two differ (every model in 28 captures of both games, spike 102).
pub fn on_rail(head: &[u8]) -> Result<bool> {
    let (x, z) = (f32(head, WORLD_X)?, f32(head, WORLD_Z)?);
    if x == 0.0 && z == 0.0 {
        return Ok(false);
    }
    Ok((f32(head, PLACE_X)? - x).abs() > NEAR || (f32(head, PLACE_Z)? - z).abs() > NEAR)
}

/// Where it is on the rail it is on: the line's id and the two numbers the game measures it
/// by, in cells, which the line turns into along it and across it (`rail.Line.upright`).
pub fn rail(head: &[u8]) -> Result<(i64, f64, f64)> {
    Ok((
        u32(head, RAIL_LINE)?,
        f32(head, RAIL_ALONG)?,
        f32(head, RAIL_ACROSS)? / CELL as f64,
    ))
}

/// The way it faces on a rail, as the game measures it: (x, a), the parts `rail` gives as
/// across and along before the line turns them (`rail.Line.direction`).
pub fn rail_facing(head: &[u8]) -> Result<(f64, f64)> {
    Ok((f32(head, FACING_SIN)?, f32(head, FACING_COS)?))
}

/// The way the line it is on runs in the world where it stands, in degrees, as a world
/// heading is measured: x over z. It is the way the rows rise, and on a track, which the game
/// measures from its joint, the way they fall (spike 107).
pub fn rail_way(head: &[u8]) -> Result<f64> {
    Ok(f32(head, LINE_SIN)?
        .atan2(f32(head, LINE_COS)?)
        .to_degrees())
}

/// Where it is in the world, in cells, a cell's centre whole: (20.31, 24.0). The circle pad
/// moves the player a seventh of a cell a frame and stops them anywhere; the D-pad keeps them to
/// the centres (movie38 and movie39, spike 202).
pub fn place(head: &[u8]) -> Result<(f64, f64)> {
    let cell = CELL as f64;
    Ok((
        (f32(head, WORLD_X)? - cell / 2.0) / cell,
        (f32(head, WORLD_Z)? - cell / 2.0) / cell,
    ))
}

/// The cell it is standing in, (x, z), or None for a model the game has not placed yet.
///
/// Where it is in the world, +0x114, says which cell it is in. On the grid +0x94 is the cell a
/// step is heading for, one cell on at most, and it is what guiding says each turn by as the
/// step begins (spike 92), so it stands where it agrees. On a rail +0x94 holds the rail's id and
/// a distance along it, which read as a cell put the player at x = 0 and everyone else hundreds
/// of cells off (spike 102); it is nowhere near, and the world's cell stands.
pub fn cell(head: &[u8]) -> Result<Option<(i64, i64)>> {
    let (x, z) = (f32(head, WORLD_X)?, f32(head, WORLD_Z)?);
    // Not placed yet: the player's model, built as X's field loads, stood at 0, 0, 0 and then at
    // 3, 3, 3 for half a second before the game placed it, and was read as cell (0, 0) once the
    // zone's data was known that soon (spike 282, x_lumiose_centrico_plaza_and_avenues).
    if x == z && z == f32(head, WORLD_Y)? {
        return Ok(None);
    }
    let half = CELL as f64 / 2.0;
    let world = (
        py::round((x - half) / CELL as f64)?,
        py::round((z - half) / CELL as f64)?,
    );
    let heading = (
        py::round(f32(head, CELL_X)?)?,
        py::round(f32(head, CELL_Z)?)?,
    );
    if (heading.0 - world.0).abs() <= 1 && (heading.1 - world.1).abs() <= 1 {
        return Ok(Some(heading));
    }
    Ok(Some(world))
}

/// How high it stands in the world, +0x118, or None where that is not a number: 110 on Mt.
/// Chimney's summit, whose floor and catwalk the chunk's meshes put at 108 (spike 156).
pub fn height(head: &[u8]) -> Result<Option<f64>> {
    let y = f32(head, WORLD_Y)?;
    Ok(if y.is_finite() { Some(y) } else { None })
}
