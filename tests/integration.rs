//! Integration tests for ghurni.

use ghurni::prelude::*;

#[test]
fn test_gasoline_engine() {
    let mut engine = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
    let samples = engine.synthesize(3000.0, 0.5, 1.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
    assert!(samples.iter().any(|&s| s.abs() > 0.01));
}

#[test]
fn test_diesel_engine() {
    let mut engine = Engine::new(EngineType::Diesel, 6, 44100.0).unwrap();
    let samples = engine.synthesize(2000.0, 0.7, 1.0).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_two_stroke() {
    let mut engine = Engine::new(EngineType::TwoStroke, 1, 44100.0).unwrap();
    let samples = engine.synthesize(5000.0, 0.8, 0.5).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_all_engine_types() {
    let types = [
        EngineType::Gasoline,
        EngineType::Diesel,
        EngineType::TwoStroke,
        EngineType::Hybrid,
    ];
    for t in &types {
        let mut engine = Engine::new(*t, 4, 44100.0).unwrap();
        let result = engine.synthesize(2000.0, 0.5, 0.3);
        assert!(result.is_ok(), "failed for {:?}", t);
    }
}

#[test]
fn test_firing_frequency() {
    let engine = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
    // 4-cylinder at 3000 RPM: 3000/60 * 4/2 = 100 Hz
    let ff = engine.firing_frequency(3000.0);
    assert!((ff - 100.0).abs() < 0.01);
}

#[test]
fn test_gear_all_materials() {
    let materials = [
        GearMaterial::Steel,
        GearMaterial::CastIron,
        GearMaterial::Brass,
        GearMaterial::Nylon,
    ];
    for m in &materials {
        let mut gear = Gear::new(32, *m, 44100.0).unwrap();
        let result = gear.synthesize(1500.0, 0.3);
        assert!(result.is_ok(), "failed for {:?}", m);
        assert!(result.unwrap().iter().all(|s| s.is_finite()));
    }
}

#[test]
fn test_gear_mesh_frequency() {
    let gear = Gear::new(32, GearMaterial::Steel, 44100.0).unwrap();
    // 32 teeth at 1500 RPM: 1500/60 * 32 = 800 Hz
    let mf = gear.mesh_frequency(1500.0);
    assert!((mf - 800.0).abs() < 0.01);
}

#[test]
fn test_motor_all_types() {
    let types = [
        MotorType::DcBrushed,
        MotorType::AcInduction,
        MotorType::Brushless,
        MotorType::Servo,
    ];
    for t in &types {
        let mut motor = Motor::new(*t, 4, 44100.0).unwrap();
        let result = motor.synthesize(3000.0, 0.5, 0.3);
        assert!(result.is_ok(), "failed for {:?}", t);
        assert!(result.unwrap().iter().all(|s| s.is_finite()));
    }
}

#[test]
fn test_turbine() {
    let mut turbine = Turbine::new(16, 500.0, 44100.0).unwrap();
    let samples = turbine.synthesize(10000.0, 0.5).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_turbine_open() {
    let mut turbine = Turbine::new(3, 0.0, 44100.0).unwrap(); // Open propeller, no duct
    let samples = turbine.synthesize(2000.0, 0.5).unwrap();
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

#[test]
fn test_clock_all_types() {
    let types = [
        ClockType::Wristwatch,
        ClockType::WallClock,
        ClockType::GrandfatherClock,
        ClockType::PocketWatch,
    ];
    for t in &types {
        let mut clock = Clock::new(*t, 44100.0).unwrap();
        let result = clock.synthesize(1.0);
        assert!(result.is_ok(), "failed for {:?}", t);
        assert!(result.unwrap().iter().all(|s| s.is_finite()));
    }
}

#[test]
fn test_higher_load_more_energy() {
    let mut lo = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
    let mut hi = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
    let lo_samples = lo.synthesize(3000.0, 0.2, 1.0).unwrap();
    let hi_samples = hi.synthesize(3000.0, 0.9, 1.0).unwrap();
    let lo_energy: f32 = lo_samples.iter().map(|s| s * s).sum();
    let hi_energy: f32 = hi_samples.iter().map(|s| s * s).sum();
    assert!(
        hi_energy > lo_energy,
        "higher load should produce more energy: hi={hi_energy}, lo={lo_energy}"
    );
}

#[test]
fn test_serde_roundtrip_engine_type() {
    let json = serde_json::to_string(&EngineType::Diesel).unwrap();
    let e2: EngineType = serde_json::from_str(&json).unwrap();
    assert_eq!(e2, EngineType::Diesel);
}

#[test]
fn test_serde_roundtrip_gear_material() {
    let json = serde_json::to_string(&GearMaterial::Brass).unwrap();
    let m2: GearMaterial = serde_json::from_str(&json).unwrap();
    assert_eq!(m2, GearMaterial::Brass);
}

#[test]
fn test_serde_roundtrip_motor_type() {
    let json = serde_json::to_string(&MotorType::Brushless).unwrap();
    let m2: MotorType = serde_json::from_str(&json).unwrap();
    assert_eq!(m2, MotorType::Brushless);
}

#[test]
fn test_serde_roundtrip_clock_type() {
    let json = serde_json::to_string(&ClockType::GrandfatherClock).unwrap();
    let c2: ClockType = serde_json::from_str(&json).unwrap();
    assert_eq!(c2, ClockType::GrandfatherClock);
}

#[test]
fn test_serde_roundtrip_error() {
    let err = GhurniError::SynthesisFailed("test".into());
    let json = serde_json::to_string(&err).unwrap();
    let e2: GhurniError = serde_json::from_str(&json).unwrap();
    assert_eq!(err.to_string(), e2.to_string());
}
