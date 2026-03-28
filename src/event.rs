//! Mechanical event triggers for discrete sound events.

use serde::{Deserialize, Serialize};

/// Discrete mechanical events that can be triggered on synthesizers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MechanicalEvent {
    /// Engine backfire — explosive pop in exhaust.
    Backfire,
    /// Cylinder misfire — skipped combustion event.
    Misfire {
        /// Which cylinder misfired (0-indexed).
        cylinder: u32,
    },
    /// Engine knock — pre-detonation metallic ping.
    Knock {
        /// Which cylinder is knocking (0-indexed).
        cylinder: u32,
    },
    /// Engine stall — RPM drops to zero.
    Stall,
    /// Rev limiter hit — fuel cut at max RPM.
    RevLimiterHit,
    /// Gear shift event.
    GearShift {
        /// Gear shifting from (0 = neutral).
        from: u32,
        /// Gear shifting to.
        to: u32,
    },
    /// Engine startup sequence.
    Startup,
    /// Engine shutdown sequence.
    Shutdown,
}
