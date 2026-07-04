//! Shipped presets for common mechanical configurations.
//!
//! Each preset creates a fully configured synthesizer with realistic
//! parameters. All presets require a `sample_rate` argument.

use crate::engine::{Engine, EngineType};
use crate::error::Result;
use crate::forced_induction::{ForcedInduction, InductionType};
use crate::gear::{Gear, GearMaterial};
use crate::motor::{Motor, MotorType};
use crate::transmission::Transmission;
use crate::turbine::Turbine;

use alloc::vec;

/// V8 muscle car engine — cross-plane firing order, deep exhaust.
pub fn v8_muscle_car(sample_rate: f32) -> Result<Engine> {
    let mut engine = Engine::new(EngineType::Gasoline, 8, sample_rate)?;
    // Cross-plane V8 firing order (uneven intervals create the burble)
    engine.set_firing_order(vec![0.0, 90.0, 270.0, 180.0, 540.0, 630.0, 450.0, 360.0]);
    engine.set_rpm(800.0);
    engine.set_load(0.1);
    Ok(engine)
}

/// Inline-4 economy car — even firing, smooth tone.
pub fn inline4_economy(sample_rate: f32) -> Result<Engine> {
    let engine = Engine::new(EngineType::Gasoline, 4, sample_rate)?;
    Ok(engine)
}

/// Diesel truck engine — 6 cylinder, rough, low RPM.
pub fn diesel_truck(sample_rate: f32) -> Result<Engine> {
    let mut engine = Engine::new(EngineType::Diesel, 6, sample_rate)?;
    engine.set_rpm(700.0);
    engine.set_load(0.2);
    Ok(engine)
}

/// Single-cylinder motorcycle (2-stroke).
pub fn motorcycle_single(sample_rate: f32) -> Result<Engine> {
    let engine = Engine::new(EngineType::TwoStroke, 1, sample_rate)?;
    Ok(engine)
}

/// Electric vehicle motor — brushless, smooth, high-RPM capable.
pub fn electric_vehicle(sample_rate: f32) -> Result<Motor> {
    let motor = Motor::new(MotorType::Brushless, 8, sample_rate)?;
    Ok(motor)
}

/// Turbocharger with moderate spool lag.
pub fn turbocharger(sample_rate: f32) -> Result<ForcedInduction> {
    ForcedInduction::new(InductionType::Turbo, 2.5, 1.0, sample_rate)
}

/// Supercharger — belt-driven, instant response.
pub fn supercharger(sample_rate: f32) -> Result<ForcedInduction> {
    ForcedInduction::new(InductionType::Supercharger, 1.5, 0.01, sample_rate)
}

/// 5-speed manual transmission.
pub fn manual_5speed(sample_rate: f32) -> Result<Transmission> {
    Transmission::new(vec![3.5, 2.1, 1.4, 1.0, 0.8], 24, sample_rate)
}

/// 6-speed manual transmission (close-ratio).
pub fn manual_6speed(sample_rate: f32) -> Result<Transmission> {
    Transmission::new(vec![3.8, 2.3, 1.6, 1.2, 0.95, 0.78], 28, sample_rate)
}

/// Steel spur gear — bright metallic mesh.
pub fn steel_spur_gear(teeth: u32, sample_rate: f32) -> Result<Gear> {
    Gear::new(teeth, GearMaterial::Steel, sample_rate)
}

/// Industrial turbine — 24-blade ducted.
pub fn industrial_turbine(sample_rate: f32) -> Result<Turbine> {
    Turbine::new(24, 600.0, sample_rate)
}

/// Propeller — 3-blade open (no duct).
pub fn propeller(sample_rate: f32) -> Result<Turbine> {
    Turbine::new(3, 0.0, sample_rate)
}
