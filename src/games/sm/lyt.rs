//! Sun's NintendoWare panes: where its `nw::lyt` keeps the fields ORAS's keeps elsewhere.
//!
//! Everything else about a layout is ORAS's (`games.common.lyt`): a pane's tree links at +0x04 to
//! +0x14, a Layout's root at +0x10 and width at +0x18, and Game Freak's work, found through the
//! `nw::lyt::DrawInfo` it embeds at +0x14, alive at +0x04, drawn at +0x07, its Layout at +0x10 and
//! the DrawInfo's projection at +0x28, -0.005 on the top screen and -0.00625 on the bottom. A work
//! is a heap block of its own, 0x134 bytes. What moved (spike 112, sun_3.ctm, over the title's
//! Continue, the X menu, the options and the save prompt, each piece checked against a picture):
//!
//! ```text
//! Pane      +0x3c f32 width    +0x40 f32 height
//!           +0x44 u8 flags, bit 0 visible    +0x45 u8 alpha    +0x46 u8 global alpha
//!           +0x47 u8 anchor, the low two bits: 0 centre, 1 left, 2 right
//!           +0x48 the global matrix: +0x48 f32 scale x, +0x54 f32 x, +0x5c f32 scale y,
//!           +0x64 f32 y
//!           +0x80 char[16] name
//! TextBox   +0xa4 -> text    +0xb8 f32 glyph width    +0xc8 -> tag processor
//!           +0xcc u16 capacity    +0xce u16 length
//!           +0xd1 u8 text position, the low two bits: 0 centre, 1 left, 2 right
//! ```
//!
//! The objects are smaller than ORAS's, and every read of one stops at its end, since the block
//! after it may be the GPU's (ADR-0035): a Pane or a Bounding is 0xa4 bytes, the smallest pane, and
//! a TextBox 0x100, each 0x20 bytes into a heap block of its own.
//!
//! Bound by:
//! - ADR-0035: a pane is read only as far as the smallest pane goes, and only inside a small block.

use crate::games::common::lyt::PaneShape;

pub const SHAPE: PaneShape = PaneShape {
    width: 0x3C,
    height: 0x40,
    scale_x: 0x48,
    x: 0x54,
    scale_y: 0x5C,
    y: 0x64,
    alpha: 0x46,
    anchor: 0x47,
    flags: 0x44,
    name: 0x80,
    head: 0xA4,
    text: 0xA4,
    glyph_width: 0xB8,
    tag_processor: 0xC8,
    capacity: 0xCC,
    length: 0xCE,
    text_position: 0xD1,
    textbox_head: 0xD4,
    text_colours: None,
    centre_first: true,
};
/// The system text file and line holding the game's own words for a level, "Lv." in English: the
/// file's one line with words (sun_9.ctm, 2026-09-27). The party screen draws them as a glyph, a
/// lone "l" in the digits' font (`screen.LEVEL_GLYPH`), and they are said from here.
pub const LEVEL_WORDS: (i64, i64) = (21, 1);
