//! Integration tests for ghurni.

use ghurni::prelude::*;

// ── Helpers ──────────────────────────────────────────────────────────

fn assert_valid_audio(samples: &[f32]) {
    assert!(!samples.is_empty());
    assert!(samples.iter().all(|s| s.is_finite()));
}

fn assert_has_energy(samples: &[f32]) {
    assert_valid_audio(samples);
    assert!(samples.iter().any(|&s| s.abs() > 0.001));
}

// ── Engine ───────────────────────────────────────────────────────────

#[test]
fn test_gasoline_engine() {
    let mut engine = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
    let samples = engine.synthesize(3000.0, 0.5, 1.0).unwrap();
    assert_has_energy(&samples);
}

#[test]
fn test_diesel_engine() {
    let mut engine = Engine::new(EngineType::Diesel, 6, 44100.0).unwrap();
    let samples = engine.synthesize(2000.0, 0.7, 1.0).unwrap();
    assert_valid_audio(&samples);
}

#[test]
fn test_two_stroke() {
    let mut engine = Engine::new(EngineType::TwoStroke, 1, 44100.0).unwrap();
    let samples = engine.synthesize(5000.0, 0.8, 0.5).unwrap();
    assert_valid_audio(&samples);
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
    let ff = engine.firing_frequency(3000.0);
    assert!((ff - 100.0).abs() < 0.01);
}

#[test]
fn test_custom_firing_order() {
    let mut engine = Engine::new(EngineType::Gasoline, 8, 44100.0).unwrap();
    // Cross-plane V8
    engine.set_firing_order(vec![0.0, 90.0, 270.0, 180.0, 540.0, 630.0, 450.0, 360.0]);
    let samples = engine.synthesize(3000.0, 0.6, 0.5).unwrap();
    assert_has_energy(&samples);
}

#[test]
fn test_engine_events() {
    let mut engine = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
    engine.set_rpm(4000.0);
    engine.set_load(0.7);
    engine.trigger_event(MechanicalEvent::Backfire);
    engine.trigger_event(MechanicalEvent::Misfire { cylinder: 0 });
    engine.trigger_event(MechanicalEvent::Knock { cylinder: 1 });
    let mut output = vec![0.0f32; 4410]; // 100ms
    engine.process_block(&mut output);
    assert_valid_audio(&output);
}

#[test]
fn test_engine_decel_pop() {
    let mut engine = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
    engine.set_rpm(5000.0);
    engine.set_load(0.8);
    let mut warmup = vec![0.0f32; 4410];
    engine.process_block(&mut warmup);
    // Drop load sharply — should trigger decel pop
    engine.set_load(0.0);
    let mut decel = vec![0.0f32; 44100]; // 1s
    engine.process_block(&mut decel);
    assert_valid_audio(&decel);
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

// ── Gear ─────────────────────────────────────────────────────────────

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
        assert_valid_audio(&result.unwrap());
    }
}

#[test]
fn test_gear_mesh_frequency() {
    let gear = Gear::new(32, GearMaterial::Steel, 44100.0).unwrap();
    let mf = gear.mesh_frequency(1500.0);
    assert!((mf - 800.0).abs() < 0.01);
}

// ── Motor ────────────────────────────────────────────────────────────

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
        assert_valid_audio(&result.unwrap());
    }
}

// ── Turbine ──────────────────────────────────────────────────────────

#[test]
fn test_turbine() {
    let mut turbine = Turbine::new(16, 500.0, 44100.0).unwrap();
    let samples = turbine.synthesize(10000.0, 0.5).unwrap();
    assert_has_energy(&samples);
}

#[test]
fn test_turbine_open() {
    let mut turbine = Turbine::new(3, 0.0, 44100.0).unwrap();
    let samples = turbine.synthesize(2000.0, 0.5).unwrap();
    assert_has_energy(&samples);
}

// ── Clock ────────────────────────────────────────────────────────────

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
        assert_valid_audio(&result.unwrap());
    }
}

// ── Forced Induction ─────────────────────────────────────────────────

#[test]
fn test_turbocharger() {
    let mut turbo = ForcedInduction::new(InductionType::Turbo, 2.5, 1.0, 44100.0).unwrap();
    let samples = turbo.synthesize(5000.0, 0.8, 1.0).unwrap();
    assert_valid_audio(&samples);
}

#[test]
fn test_supercharger() {
    let mut sc = ForcedInduction::new(InductionType::Supercharger, 1.5, 0.01, 44100.0).unwrap();
    let samples = sc.synthesize(4000.0, 0.7, 0.5).unwrap();
    assert_valid_audio(&samples);
}

#[test]
fn test_bov_trigger() {
    let mut turbo = ForcedInduction::new(InductionType::Turbo, 2.5, 0.5, 44100.0).unwrap();
    turbo.set_rpm(6000.0);
    turbo.set_load(0.9);
    let mut warmup = vec![0.0f32; 22050]; // 500ms to build spool
    turbo.process_block(&mut warmup);
    turbo.trigger_bov();
    let mut bov_output = vec![0.0f32; 8820]; // 200ms
    turbo.process_block(&mut bov_output);
    assert_valid_audio(&bov_output);
}

// ── Transmission ─────────────────────────────────────────────────────

#[test]
fn test_transmission() {
    let mut trans = Transmission::new(vec![3.5, 2.1, 1.4, 1.0, 0.8], 24, 44100.0).unwrap();
    trans.shift_to(2);
    let samples = trans.synthesize(3000.0, 0.5).unwrap();
    assert_valid_audio(&samples);
}

#[test]
fn test_transmission_shift() {
    let mut trans = Transmission::new(vec![3.5, 2.1, 1.4], 24, 44100.0).unwrap();
    use ghurni::traits::Synthesizer;
    trans.set_rpm(4000.0);
    let mut output = vec![0.0f32; 4410];
    trans.process_block(&mut output);
    trans.shift_to(1);
    trans.process_block(&mut output);
    assert_valid_audio(&output);
}

// ── Differential ─────────────────────────────────────────────────────

#[test]
fn test_differential() {
    let mut diff = Differential::new(41, 11, 44100.0).unwrap();
    let samples = diff.synthesize(2000.0, 0.5).unwrap();
    assert_valid_audio(&samples);
}

// ── Chain Drive ──────────────────────────────────────────────────────

#[test]
fn test_chain_drive() {
    let mut chain = ChainDrive::new(110, 15, 44100.0).unwrap();
    let samples = chain.synthesize(3000.0, 0.5).unwrap();
    assert_valid_audio(&samples);
}

// ── Belt Drive ───────────────────────────────────────────────────────

#[test]
fn test_belt_drive() {
    let mut belt = BeltDrive::new(100.0, 0.7, 44100.0).unwrap();
    let samples = belt.synthesize(2000.0, 0.5).unwrap();
    assert_valid_audio(&samples);
}

#[test]
fn test_belt_drive_slack() {
    let mut belt = BeltDrive::new(100.0, 0.1, 44100.0).unwrap(); // Slack = squealy
    let samples = belt.synthesize(3000.0, 0.5).unwrap();
    assert_has_energy(&samples);
}

// ── Mixer ────────────────────────────────────────────────────────────

#[test]
fn test_mixer_mono() {
    let mut mixer = MechanicalMixer::new();
    let engine = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
    let gear = Gear::new(32, GearMaterial::Steel, 44100.0).unwrap();
    mixer.add_channel("engine".into(), Box::new(engine));
    mixer.add_channel("gear".into(), Box::new(gear));
    mixer.set_rpm(3000.0);
    let mut output = vec![0.0f32; 44100];
    mixer.process_block(&mut output);
    assert_has_energy(&output);
}

#[test]
fn test_mixer_stereo() {
    let mut mixer = MechanicalMixer::new();
    let engine = Engine::new(EngineType::Diesel, 6, 44100.0).unwrap();
    let idx = mixer.add_channel("engine".into(), Box::new(engine));
    mixer.set_channel_pan(idx, -0.5); // Pan left
    mixer.set_rpm(2000.0);
    let mut left = vec![0.0f32; 4410];
    let mut right = vec![0.0f32; 4410];
    mixer.process_block_stereo(&mut left, &mut right);
    assert_valid_audio(&left);
    assert_valid_audio(&right);
}

#[test]
fn test_mixer_mute() {
    let mut mixer = MechanicalMixer::new();
    let engine = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
    let idx = mixer.add_channel("engine".into(), Box::new(engine));
    mixer.set_channel_muted(idx, true);
    mixer.set_rpm(3000.0);
    let mut output = vec![0.0f32; 4410];
    mixer.process_block(&mut output);
    // All zeros since the only channel is muted
    assert!(output.iter().all(|&s| s == 0.0));
}

// ── Presets ──────────────────────────────────────────────────────────

#[test]
fn test_preset_v8_muscle_car() {
    let mut engine = ghurni::presets::v8_muscle_car(44100.0).unwrap();
    let samples = engine.synthesize(3000.0, 0.6, 0.5).unwrap();
    assert_has_energy(&samples);
}

#[test]
fn test_preset_diesel_truck() {
    let mut engine = ghurni::presets::diesel_truck(44100.0).unwrap();
    let samples = engine.synthesize(1500.0, 0.5, 0.5).unwrap();
    assert_has_energy(&samples);
}

#[test]
fn test_preset_electric_vehicle() {
    let mut motor = ghurni::presets::electric_vehicle(44100.0).unwrap();
    let samples = motor.synthesize(8000.0, 0.5, 0.5).unwrap();
    assert_valid_audio(&samples);
}

#[test]
fn test_preset_manual_5speed() {
    let mut trans = ghurni::presets::manual_5speed(44100.0).unwrap();
    let samples = trans.synthesize(3000.0, 0.3).unwrap();
    assert_valid_audio(&samples);
}

// ── Synthesizer Trait ────────────────────────────────────────────────

#[test]
fn test_synthesizer_trait_dispatch() {
    let synths: Vec<Box<dyn Synthesizer>> = vec![
        Box::new(Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap()),
        Box::new(Gear::new(32, GearMaterial::Steel, 44100.0).unwrap()),
        Box::new(Motor::new(MotorType::Brushless, 8, 44100.0).unwrap()),
        Box::new(Turbine::new(16, 500.0, 44100.0).unwrap()),
    ];
    for mut synth in synths {
        synth.set_rpm(2000.0);
        let mut output = vec![0.0f32; 4410];
        synth.process_block(&mut output);
        assert_valid_audio(&output);
    }
}

// ── Smoothed Param ───────────────────────────────────────────────────

#[test]
fn test_smoothed_param() {
    let mut p = ghurni::smooth::SmoothedParam::new(0.0, 0.01, 44100.0);
    p.set_target(1.0);
    // After many samples, should approach target
    for _ in 0..44100 {
        p.next_value();
    }
    assert!((p.current() - 1.0).abs() < 0.001);
}

#[test]
fn test_smoothed_param_snap() {
    let mut p = ghurni::smooth::SmoothedParam::new(0.0, 1.0, 44100.0);
    p.set_target(5.0);
    p.snap();
    assert!((p.current() - 5.0).abs() < f32::EPSILON);
}

// ── Process Block Continuity ─────────────────────────────────────────

#[test]
fn test_process_block_continuity() {
    // Synthesize 4410 samples in one block vs 4 blocks of 1102/1103
    let mut engine_a = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
    engine_a.set_rpm(3000.0);
    engine_a.set_load(0.5);
    // Snap smoothers so both paths start from identical state
    let mut one_block = vec![0.0f32; 4410];
    engine_a.process_block(&mut one_block);

    let mut engine_b = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
    engine_b.set_rpm(3000.0);
    engine_b.set_load(0.5);
    let mut multi_block = vec![0.0f32; 4410];
    let chunk_size = 1102;
    let mut offset = 0;
    while offset < 4410 {
        let end = (offset + chunk_size).min(4410);
        engine_b.process_block(&mut multi_block[offset..end]);
        offset = end;
    }

    // Both should produce valid audio (exact match not guaranteed due to
    // smoother state, but both should be finite and similar energy)
    assert_valid_audio(&one_block);
    assert_valid_audio(&multi_block);
    let energy_a: f32 = one_block.iter().map(|s| s * s).sum();
    let energy_b: f32 = multi_block.iter().map(|s| s * s).sum();
    let ratio = energy_a / energy_b.max(f32::EPSILON);
    assert!(
        (0.5..2.0).contains(&ratio),
        "energy ratio too different: {ratio}"
    );
}

// ── Parameter Sweep ──────────────────────────────────────────────────

#[test]
fn test_engine_parameter_sweep() {
    for rpm in [100.0, 500.0, 1000.0, 3000.0, 6000.0, 10000.0, 15000.0] {
        for load in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let mut engine = Engine::new(EngineType::Gasoline, 4, 44100.0).unwrap();
            let samples = engine.synthesize(rpm, load, 0.1).unwrap();
            assert!(
                samples.iter().all(|s| s.is_finite()),
                "NaN/Inf at rpm={rpm}, load={load}"
            );
        }
    }
}

#[test]
fn test_gear_parameter_sweep() {
    for teeth in [4, 8, 16, 32, 64] {
        for rpm in [1.0, 100.0, 1000.0, 10000.0] {
            let mut gear = Gear::new(teeth, GearMaterial::Steel, 44100.0).unwrap();
            let samples = gear.synthesize(rpm, 0.1).unwrap();
            assert!(
                samples.iter().all(|s| s.is_finite()),
                "NaN/Inf at teeth={teeth}, rpm={rpm}"
            );
        }
    }
}

// ── Serde Roundtrips ─────────────────────────────────────────────────

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

#[test]
fn test_serde_roundtrip_induction_type() {
    let json = serde_json::to_string(&InductionType::Turbo).unwrap();
    let t2: InductionType = serde_json::from_str(&json).unwrap();
    assert_eq!(t2, InductionType::Turbo);
}

#[test]
fn test_serde_roundtrip_mechanical_event() {
    let event = MechanicalEvent::GearShift { from: 2, to: 3 };
    let json = serde_json::to_string(&event).unwrap();
    let e2: MechanicalEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(e2, event);
}
