//! # ghurni — Mechanical Sound Synthesis
//!
//! **ghurni** (Sanskrit: rotation / spinning) provides procedural synthesis of
//! mechanical sounds: engines, gears, motors, turbines, and other RPM-driven
//! machinery. All sounds are generated from physical models — rotational
//! harmonics, combustion impulses, resonant bodies.
//!
//! ## Architecture
//!
//! ```text
//! Machine (type, RPM, load)
//!       |
//!       v
//! Rotational Core ──────────────── Output
//!   Engine:   combustion cycle, exhaust resonance
//!   Gear:     tooth mesh frequency, metallic ring
//!   Motor:    electromagnetic hum, commutator noise
//!   Turbine:  blade pass frequency, whoosh
//!   Clock:    escapement tick, spring resonance
//! ```
//!
//! ## Key Concepts
//!
//! - **RPM**: Rotational speed — the fundamental parameter driving all mechanical sounds
//! - **Machine**: A mechanical device with specific acoustic character
//! - **Load**: Operating load (0.0-1.0) affects strain, noise, and harmonic content
//! - **Material**: Body material affects resonance (steel, aluminum, cast iron)
//!
//! ## Quick Start
//!
//! ```rust
//! use ghurni::prelude::*;
//!
//! // Synthesize a diesel engine at 2000 RPM
//! let mut engine = Engine::new(EngineType::Diesel, 6, 44100.0).unwrap();
//! let samples = engine.synthesize(2000.0, 0.7, 1.0).unwrap();
//!
//! // Generate gear mesh sound
//! let mut gear = Gear::new(32, GearMaterial::Steel, 44100.0).unwrap();
//! let samples = gear.synthesize(1500.0, 0.5).unwrap();
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `std` | Yes | Standard library support. Disable for `no_std` + `alloc` |
//! | `naad-backend` | Yes | Use naad for DSP primitives (oscillators, filters, noise) |
//! | `logging` | No | Structured logging via tracing-subscriber |

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod clock;
pub mod engine;
pub mod error;
pub mod gear;
#[cfg(not(feature = "naad-backend"))]
#[allow(dead_code)]
mod math;
pub mod motor;
#[cfg(not(feature = "naad-backend"))]
#[allow(dead_code)]
pub(crate) mod rng;
pub mod turbine;

/// Convenience re-exports for common usage.
pub mod prelude {
    pub use crate::clock::{Clock, ClockType};
    pub use crate::engine::{Engine, EngineType};
    pub use crate::error::{GhurniError, Result};
    pub use crate::gear::{Gear, GearMaterial};
    pub use crate::motor::{Motor, MotorType};
    pub use crate::turbine::Turbine;
}

// Compile-time trait assertions: all public types must be Send + Sync.
#[cfg(test)]
mod assert_traits {
    fn _assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn public_types_are_send_sync() {
        _assert_send_sync::<crate::error::GhurniError>();
        _assert_send_sync::<crate::engine::Engine>();
        _assert_send_sync::<crate::engine::EngineType>();
        _assert_send_sync::<crate::gear::Gear>();
        _assert_send_sync::<crate::gear::GearMaterial>();
        _assert_send_sync::<crate::motor::Motor>();
        _assert_send_sync::<crate::motor::MotorType>();
        _assert_send_sync::<crate::turbine::Turbine>();
        _assert_send_sync::<crate::clock::Clock>();
        _assert_send_sync::<crate::clock::ClockType>();
    }
}
