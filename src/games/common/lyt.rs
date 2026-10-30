//! NintendoWare layouts as ORAS builds them: the offsets of `nw::lyt` objects and Game Freak's
//! work.
//!
//! Structures only, no reading: `layout.Layouts` answers where a pane is, and each reader takes the
//! fields it needs from the head it read. Read off live objects in spikes 14, 29 to 33, 39 to 41,
//! 48 to 50 and 54.
//!
//! ```text
//! Pane      +0x04 -> the next child's node in its parent's child list, or the list at the end
//!           +0x0c -> parent, 0 at the root     +0x10 u32 child count
//!           +0x14 -> the first child's node, or the list itself if none
//!           (a child's node is 4 bytes into the child)
//!           +0x48 f32 width    +0x4c f32 height
//!           +0x80 f32 global scale x    +0x8c f32 global x    +0x94 f32 global scale y
//!           +0x9c f32 global y
//!           +0xb5 u8 global alpha    +0xb6 u8 anchor, bits 2-3: 0 left, 1 centre, 2 right
//!           +0xb7 u8 flags, bit 0 visible    +0xb8 char[16] name
//! TextBox   +0xd4 -> text    +0xe4 f32 glyph width    +0xe8 f32 glyph height
//!           +0xf4 -> tag processor, 0 for a designer's placeholder
//!           +0xf8 u16 capacity    +0xfa u16 length
//!           +0xfc u8 text position, % 3: left, centre, right
//! Layout    +0x10 -> root pane    +0x18 f32 width: 400 top screen, 320 bottom
//! work      +0x04 u8 alive    +0x07 u8 drawn    +0x10 -> Layout    +0x14 nw::lyt::DrawInfo
//!           +0x28 f32 the DrawInfo's projection: -2 over the width of the screen drawn to
//! ```
//!
//! A work is Game Freak's record for one layout. It has no class of its own; the DrawInfo it embeds
//! does, which is how a sweep finds it. Works sit in pools 0x104 apart (spikes 61 and 64):
//!
//! ```text
//! work      +0xbc -> a resource, whose +0x10 is the `darc` archive holding the layout's files
//! pool      +0x00 -> its table    +0x04 u16 capacity    +0x06 u16 in use
//!           +0x1c -> the end of its table
//!           the table starts 0x48 into the pool, and the works 0x30 after the table's end
//! entry     +0x00 -> the pool    +0x04 -> a work        (8 bytes; a message window holds one)
//! ```

use crate::py::{Error, Result};

/// Every pane class is in `nw::lyt`: Pane, Picture, Window, Bounding, TextBox.
pub const CLASS_PREFIX: &str = "N2nw3lyt";
pub const PANE_CLASS: &str = "N2nw3lyt4PaneE";
pub const PICTURE_CLASS: &str = "N2nw3lyt7PictureE";
pub const TEXTBOX_CLASS: &str = "N2nw3lyt7TextBoxE";
/// A layout pane drawn as a stretchable frame, as a menu cursor is. Not a message window
/// (`msgwin`) and not a list window (`lists.MENU_WINDOW_CLASS`).
pub const WINDOW_CLASS: &str = "N2nw3lyt6WindowE";
/// An invisible pane marking where a touch lands: a key's or a button's.
pub const BOUNDING_CLASS: &str = "N2nw3lyt8BoundingE";
/// A pane standing for a layout built into another, its panes under it: each of Sun's buttons,
/// and the frame of its keyboard's cursor (spike 112).
pub const PARTS_CLASS: &str = "N2nw3lyt5PartsE";
pub const LAYOUT_CLASS: &str = "N2nw3lyt6LayoutE";
pub const DRAWINFO_CLASS: &str = "N2nw3lyt8DrawInfoE";

// nw::lyt::Pane
pub const NODE: i64 = 0x04;
pub const PARENT: i64 = 0x0C;
pub const CHILD_COUNT: i64 = 0x10;
pub const CHILD_LIST: i64 = 0x14;
pub const WIDTH: i64 = 0x48;
pub const HEIGHT: i64 = 0x4C;
pub const SCALE_X: i64 = 0x80;
pub const X: i64 = 0x8C;
pub const SCALE_Y: i64 = 0x94;
pub const Y: i64 = 0x9C;
/// The pane's global alpha: its own with its ancestors' folded in. The summary screen fades the
/// move detail and contest labels it is not showing to 0 and leaves them visible (spike 35).
pub const ALPHA: i64 = 0xB5;
pub const ANCHOR: i64 = 0xB6;
pub const FLAGS: i64 = 0xB7;
pub const VISIBLE: i64 = 0x01;
pub const NAME: i64 = 0xB8;
/// NUL-padded and minified ("2j"), but the name its layout file gives the pane.
pub const NAME_SIZE: i64 = 16;
/// Enough of a pane to take a TextBox's text, length and alignment in one read.
pub const HEAD: i64 = 0x100;

// nw::lyt::Picture and its nw::lyt::Material (spikes 128 and 198)
/// -> the picture's material, which says what it draws now, whatever its layout file says.
pub const PICTURE_MATERIAL: i64 = 0x13C;
/// Its four corners' colours, RGBA a byte each, which tint what it draws: Pokémon-Amie's games
/// draw a star not earned as a full star in black at alpha 0x3c, a faint shadow (or_amie_stars).
pub const PICTURE_COLOURS: i64 = 0x140;
/// The physical address of the image the material's first texture draws: FCRAM, or video memory
/// for some the game builds itself.
pub const MATERIAL_TEXTURE: i64 = 0x84;
/// u16 width, u16 height: the image's size as it is seen.
pub const MATERIAL_SIZE: i64 = 0x88;
/// u16 width, u16 height: the size it is stored at, each rounded up to a power of two.
pub const MATERIAL_STORED: i64 = 0x8C;
/// The GPU's texture format (`textures.BITS`).
pub const MATERIAL_FORMAT: i64 = 0x9C;

// nw::lyt::TextBox
pub const TEXT: i64 = 0xD4;
pub const GLYPH_WIDTH: i64 = 0xE4;
pub const TAG_PROCESSOR: i64 = 0xF4;
pub const CAPACITY: i64 = 0xF8;
/// Capacity comes first. As one word the pair reads `0x00240086`, easy to split the wrong way.
pub const LENGTH: i64 = 0xFA;
pub const TEXT_POSITION: i64 = 0xFC;

/// Where one game's panes keep the fields that moved between versions of NintendoWare.
///
/// The tree's links, the Layout and Game Freak's work are where ORAS has them in Sun too; a
/// pane's size, place, flags and name, and a TextBox's fields, are not (spike 112). The module's
/// constants are ORAS's, and `ORAS` gathers them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneShape {
    pub width: i64,
    pub height: i64,
    pub scale_x: i64,
    pub x: i64,
    pub scale_y: i64,
    pub y: i64,
    pub alpha: i64,
    pub anchor: i64,
    pub flags: i64,
    pub name: i64,
    /// How much of any pane one read takes, and no more than the smallest pane holds.
    pub head: i64,
    pub text: i64,
    pub glyph_width: i64,
    pub tag_processor: i64,
    pub capacity: i64,
    pub length: i64,
    pub text_position: i64,
    /// How much of a TextBox one read takes: through its text position, within the object.
    pub textbox_head: i64,
    /// Alignment codes in NintendoWare's order, centre, left, right, in the low two bits, as Sun
    /// keeps both a pane's anchor and a TextBox's text position; ORAS keeps its anchor in bits 2
    /// and 3 and its text position modulo 3, left, centre, right.
    pub centre_first: bool,
}

impl PaneShape {
    /// The pane's horizontal anchor: 0 left, 1 centre, 2 right.
    ///
    /// Raises `IndexError` where `head` stops short of the anchor, as Python's indexing does.
    pub fn anchored(&self, head: &[u8]) -> Result<i64> {
        if self.centre_first {
            let byte = head
                .get(self.anchor as usize)
                .ok_or_else(|| Error::index("index out of range"))?;
            return Ok(LEFT_FIRST[(byte & 3) as usize]);
        }
        let byte = head
            .get(self.anchor as usize)
            .ok_or_else(|| Error::index("index out of range"))?;
        Ok(i64::from((byte >> 2) & 3).min(2))
    }

    /// Where a TextBox sets its words across its width: 0 left, 1 centre, 2 right.
    ///
    /// Raises `IndexError` where `head` stops short of the text position, as Python's indexing
    /// does.
    pub fn aligned(&self, head: &[u8]) -> Result<i64> {
        if self.centre_first {
            let byte = head
                .get(self.text_position as usize)
                .ok_or_else(|| Error::index("index out of range"))?;
            return Ok(LEFT_FIRST[(byte & 3) as usize]);
        }
        let byte = head
            .get(self.text_position as usize)
            .ok_or_else(|| Error::index("index out of range"))?;
        Ok(i64::from(byte % 3))
    }
}

/// NintendoWare's centre, left, right as left, centre, right; the unused fourth code as left.
const LEFT_FIRST: [i64; 4] = [1, 0, 2, 0];

pub const ORAS: PaneShape = PaneShape {
    width: WIDTH,
    height: HEIGHT,
    scale_x: SCALE_X,
    x: X,
    scale_y: SCALE_Y,
    y: Y,
    alpha: ALPHA,
    anchor: ANCHOR,
    flags: FLAGS,
    name: NAME,
    head: HEAD,
    text: TEXT,
    glyph_width: GLYPH_WIDTH,
    tag_processor: TAG_PROCESSOR,
    capacity: CAPACITY,
    length: LENGTH,
    text_position: TEXT_POSITION,
    textbox_head: HEAD,
    centre_first: false,
};

// nw::lyt::Layout
pub const LAYOUT_ROOT: i64 = 0x10;
pub const LAYOUT_WIDTH: i64 = 0x18;
pub const TOP_SCREEN_WIDTH: f64 = 400.0;

// Game Freak's layout work
pub const WORK_ALIVE: i64 = 0x04;
/// 1 while the layout is on the system's list of layouts to draw, which the field and the battle
/// draw theirs through, walked each frame (0x0012aa3c, a work's next at +0xC4); 0 for a layout an
/// app draws itself, as the PC Box and the party do theirs. The battle takes its command buttons
/// off the list while its move list is up, and draws them no more: their root is not read once a
/// frame (spike 188, movie23).
pub const WORK_LISTED: i64 = 0x05;
/// 1 once the layout has been drawn, written by the draw itself (0x0014d98c) and never cleared:
/// a layout no longer drawn keeps it.
pub const WORK_DRAWN: i64 = 0x07;
/// u16: where the layout is drawn among its screen's others, higher over lower (movie21).
pub const WORK_LAYER: i64 = 0x0A;
pub const WORK_LAYOUT: i64 = 0x10;
pub const WORK_DRAWINFO: i64 = 0x14;
pub const WORK_HEAD: i64 = WORK_DRAWINFO + 4;
/// Filled in when the layout is drawn: -2 divided by the width of the screen it was drawn to,
/// -0.00625 below and -0.005 on top (spikes 40 and 41). A layout's own width does not say which
/// screen it is on: the menu cursor's layouts are 400 wide and drawn below.
pub const WORK_PROJECTION: i64 = 0x28;
/// Works sit in pools, one after another (spike 61).
pub const WORK_SIZE: i64 = 0x104;
/// -> the resource the layout was built from; its +0x10 is the `darc` archive of its files.
pub const WORK_RESOURCE: i64 = 0xBC;
pub const RESOURCE_ARCHIVE: i64 = 0x10;
/// -> the `nw::lyt::ArcResourceAccessor` of the archive, whose +0x04 is the archive again.
pub const RESOURCE_ACCESSOR: i64 = 0x14;
pub const ACCESSOR_CLASS: &str = "N2nw3lyt19ArcResourceAccessorE";
/// Resources sit in a table, one after another: the Pokémon-Amie page's layout is built from one
/// and draws its PLAY, DECORATE and SWITCH from the next, an archive of their images alone, which
/// no work points at (spike 198).
pub const RESOURCE_SIZE: i64 = 0x48;
pub const MAX_RESOURCES: i64 = 32;

// Game Freak's layout pool (spike 64): a table of 8-byte entries, each the pool and a work
pub const POOL_TABLE: i64 = 0x00;
/// u16. The battle's pool holds 24, the field's 48.
pub const POOL_CAPACITY: i64 = 0x04;
/// u16: the slots holding a layout. A layout built into the pool changes it.
pub const POOL_IN_USE: i64 = 0x06;
/// -> the pool itself: a validator (spike 99).
pub const POOL_SELF: i64 = 0x08;
pub const POOL_TABLE_END: i64 = 0x1C;
pub const POOL_HEAD: i64 = 0x20;
/// Where the table starts inside the pool, on both pools read: a validator.
pub const POOL_TABLE_OFFSET: i64 = 0x48;
/// Where the slots start, less the table: 0x30 past the table's end, then a slot of 0x104 each,
/// and the pool's own work after the last (spike 99).
pub const POOL_SLOTS_OFFSET: i64 = POOL_TABLE_OFFSET + 0x30;
pub const MAX_POOL: i64 = 512;
pub const ENTRY_POOL: i64 = 0x00;
pub const ENTRY_WORK: i64 = 0x04;
pub const ENTRY_SIZE: i64 = 0x08;
