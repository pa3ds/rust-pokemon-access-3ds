//! The field camera's heading: the way up walks the player on Lumiose's boulevards (spike 105).
//!
//! On a rail the camera is behind the player and the D-pad works from it: up walks the way the camera
//! looks, down walks back from it without turning it, and left and right turn the player a quarter,
//! the camera turning after them only while the key is held. So the camera, not the way the player
//! faces, is what a way there has to be said from, and a tap leaves it pointing anywhere.
//!
//! ```text
//! xy_system::CCameraULCD  +0x64 ptr a block of the heap's own, tagged and sized as every
//!                         block is (`discovery.ALLOCATION`, `BLOCK_SIZE`)
//! that block              +0xc0c f32 x3 where the camera is in the world, +0xc18 x3 what it
//!                         looks at, +0xc24 x3 the unit way from one to the other
//!                         +0xc30 f32 the camera's own heading, in radians, the way a model's
//!                         world facing is: up on the pad walks the world's (sin, cos) of it
//!                         +0xebc f32 the camera's heading where it is behind the player, the
//!                         same way
//! ```
//!
//! Found in seven snapshots taken while the player walked up, each a different way, where +0xebc
//! was the way they walked to a tenth of a degree (spike 105). The block is the upper screen
//! camera's and is there on the grid too, where +0xebc reads 0 and +0xc30 is where the camera
//! looks: pi in Parfum Palace and on Route 7, -1.22 through the palace's back door into its garden,
//! where up walked Khai west, swinging round to -pi as he walked north up it (spike 230,
//! x_route-6-4.ctm, 2026-10-20). The way from where it is to what it looks at turns with it, to the
//! tenth of a degree.
//!
//! Off the rails the camera is behind the player in Glittering Cave too, and +0xebc holds its heading
//! there while +0xc30 reads 0: -172.3 degrees on arrival, looking north. On every other grid map
//! measured, Route 9, Parfum Palace and its garden, Route 7 and a Lumiose building, +0xebc reads
//! exactly 0 (spike 239, x_glittering_cave). So `behind` tells a camera behind the player on the grid
//! from the grid's own, which `looks` reads.

use std::f64::consts::PI;

use crate::core::binary::{f32, u32};
use crate::core::discovery::{ALLOCATION, Found, HEAP_SPAN, within};
use crate::core::memory::Memory;
use crate::py::Result;

pub const CLASS: &str = "N9xy_system11CCameraULCDE";
/// Where the camera object points at the block holding its heading.
pub const BLOCK: i64 = 0x64;
pub const BLOCK_SIZE: i64 = 0x1468;
pub const LOOKS: i64 = 0xC30;
pub const YAW: i64 = 0xEBC;

/// The block holding the camera's heading, through whichever camera object points at one;
/// None when no sweep has found one.
pub fn find(found: &Found, memory: &dyn Memory) -> Result<Option<i64>> {
    let mut cameras = found.of(&[CLASS]);
    cameras.sort();
    for camera in cameras {
        let block = u32(&memory.read(camera + BLOCK, 4)?, 0)?;
        if within(HEAP_SPAN, block) && is_block(memory, block)? {
            return Ok(Some(block));
        }
    }
    Ok(None)
}

/// Whether `block` still starts the camera's block: freed, the heap reuses it.
pub fn is_block(memory: &dyn Memory, block: i64) -> Result<bool> {
    let head = memory.read(block, 8)?;
    Ok(u32(&head, 0)? == ALLOCATION && u32(&head, 4)? == BLOCK_SIZE)
}

/// The camera's heading in the world, in degrees, or None when the block is not the
/// camera's any more or holds no angle.
pub fn heading(memory: &dyn Memory, block: i64) -> Result<Option<f64>> {
    if !is_block(memory, block)? {
        return Ok(None);
    }
    let yaw = angle(memory, block, YAW)?;
    Ok(yaw.map(f64::to_degrees))
}

/// The heading of a camera behind the player, in degrees (`heading`), or None where `YAW`
/// reads 0, as it does wherever the camera is the grid's own (spike 239).
pub fn behind(memory: &dyn Memory, block: i64) -> Result<Option<f64>> {
    // Python's `heading(...) or None`: 0.0, and -0.0, are false.
    Ok(heading(memory, block)?.filter(|&yaw| yaw != 0.0))
}

/// Where the camera looks, in radians the way a model faces (`LOOKS`), or None when the block
/// is not the camera's any more or holds no angle: the way up walks the player off the rails.
pub fn looks(memory: &dyn Memory, block: i64) -> Result<Option<f64>> {
    if is_block(memory, block)? {
        angle(memory, block, LOOKS)
    } else {
        Ok(None)
    }
}

fn angle(memory: &dyn Memory, block: i64, at: i64) -> Result<Option<f64>> {
    let angle = f32(&memory.read(block + at, 4)?, 0)?;
    Ok(if angle.is_finite() && angle.abs() <= PI + 0.01 {
        Some(angle)
    } else {
        None
    })
}
