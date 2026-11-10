//! Which way the pad walks the player where the camera is turned, from the camera's own angle.
//!
//! Up on the pad walks the player the way the camera looks, whichever that is, in Sun (spike 119)
//! and in X off Lumiose's rails (spike 230): in Parfum Palace's garden the camera looks west, a
//! little south, and up walked the player west, where pa3ds had said the stairs back "1 south"
//! (x_route-6-4.ctm, 2026-10-20). Each game keeps the angle in an object of its own (`sm.camera`,
//! `gen6.field.camera`), in the way a model's world facing is: up walks the world's (sin a, cos a),
//! x east and z south.

use crate::py::{self, Result};

pub const QUARTERS: i64 = 4;
/// Degrees past a quarter's edge (45 from its centre) the camera looks before the compass turns.
pub const HYSTERESIS: f64 = 10.0;

/// The quarter turns clockwise from north of the way up walks, nearest: 0 north, 1 east, 2
/// south, 3 west. With `current`, the quarter last taken, that one until the camera looks
/// `HYSTERESIS` past its edge: a camera swinging about an edge would turn the whole compass back
/// and forth with it (2026-09-27). North for an angle that is not a number, or `current`.
pub fn quarter(angle: f64, current: Option<i64>) -> Result<i64> {
    if !angle.is_finite() {
        return Ok(current.unwrap_or(0));
    }
    let bearing = angle.sin().atan2(-angle.cos()).to_degrees();
    if let Some(current) = current {
        let off = py::modulo_f(bearing - (current * 90) as f64 + 180.0, 360.0) - 180.0;
        if off.abs() <= 45.0 + HYSTERESIS {
            return Ok(current);
        }
    }
    Ok(py::modulo(py::round(bearing / 90.0)?, QUARTERS))
}
