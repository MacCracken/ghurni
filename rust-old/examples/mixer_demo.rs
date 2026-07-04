//! Mixer demo — combining multiple mechanical sound sources.
//!
//! Shows how to use MechanicalMixer for stereo output with
//! per-component gain and panning.
//!
//! Run: `cargo run --example mixer_demo`

use ghurni::prelude::*;

fn main() -> ghurni::error::Result<()> {
    let mut mixer = MechanicalMixer::new();

    // Add engine (center)
    let engine = Engine::new(EngineType::Diesel, 6, 44100.0)?;
    let eng_idx = mixer.add_channel("engine".into(), Box::new(engine));
    mixer.set_channel_gain(eng_idx, 0.8);
    mixer.set_channel_pan(eng_idx, 0.0);

    // Add turbo (slightly right)
    let turbo = ForcedInduction::new(InductionType::Turbo, 2.5, 1.0, 44100.0)?;
    let turbo_idx = mixer.add_channel("turbo".into(), Box::new(turbo));
    mixer.set_channel_gain(turbo_idx, 0.4);
    mixer.set_channel_pan(turbo_idx, 0.3);

    // Add gear whine (slightly left)
    let gear = Gear::new(32, GearMaterial::Steel, 44100.0)?;
    let gear_idx = mixer.add_channel("gear".into(), Box::new(gear));
    mixer.set_channel_gain(gear_idx, 0.2);
    mixer.set_channel_pan(gear_idx, -0.3);

    // Set all components to 3000 RPM
    mixer.set_rpm(3000.0);

    // Process stereo output
    let mut left = vec![0.0f32; 44100];
    let mut right = vec![0.0f32; 44100];
    mixer.process_block_stereo(&mut left, &mut right);

    let peak_l = left.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let peak_r = right.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

    println!("Stereo mix ({} channels):", mixer.channel_count());
    println!("  left  peak: {peak_l:.4}");
    println!("  right peak: {peak_r:.4}");

    Ok(())
}
