//! Vehicle scene — layers engine, turbo, transmission, and differential.
//!
//! Demonstrates building a complete vehicle sound from components
//! using the Synthesizer trait for uniform handling.
//!
//! Run: `cargo run --example vehicle_scene`

use ghurni::prelude::*;

fn main() -> ghurni::error::Result<()> {
    let sr = 44100.0;

    // Build vehicle components
    let mut engine = ghurni::presets::v8_muscle_car(sr)?;
    let mut turbo = ghurni::presets::turbocharger(sr)?;
    let mut trans = ghurni::presets::manual_5speed(sr)?;
    let mut diff = Differential::new(41, 11, sr)?;

    // Simulate acceleration: 2000 -> 5000 RPM over 2 seconds
    let block_size = 512;
    let total_samples = (sr * 2.0) as usize;
    let mut mixed = vec![0.0f32; total_samples];
    let mut block = vec![0.0f32; block_size];

    let mut offset = 0;
    while offset < total_samples {
        let progress = offset as f32 / total_samples as f32;
        let rpm = 2000.0 + progress * 3000.0;
        let load = 0.4 + progress * 0.4;

        engine.set_rpm(rpm);
        engine.set_load(load);
        turbo.set_rpm(rpm);
        turbo.set_load(load);

        // Transmission and diff get output RPM
        use ghurni::traits::Synthesizer;
        trans.set_rpm(rpm);
        diff.set_rpm(trans.output_rpm());

        let end = (offset + block_size).min(total_samples);
        let len = end - offset;

        // Mix all components
        block.iter_mut().for_each(|s| *s = 0.0);
        let mut temp = vec![0.0f32; len];

        engine.process_block(&mut temp);
        for (i, &s) in temp.iter().enumerate() {
            block[i] += s * 0.5;
        }

        turbo.process_block(&mut temp);
        for (i, &s) in temp.iter().enumerate() {
            block[i] += s * 0.3;
        }

        trans.process_block(&mut temp);
        for (i, &s) in temp.iter().enumerate() {
            block[i] += s * 0.1;
        }

        diff.process_block(&mut temp);
        for (i, &s) in temp.iter().enumerate() {
            block[i] += s * 0.1;
        }

        mixed[offset..end].copy_from_slice(&block[..len]);
        offset = end;
    }

    let peak = mixed.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let rms = (mixed.iter().map(|s| s * s).sum::<f32>() / mixed.len() as f32).sqrt();

    println!("Vehicle acceleration scene (2s):");
    println!("  components: engine + turbo + transmission + differential");
    println!("  RPM sweep:  2000 -> 5000");
    println!("  samples:    {}", mixed.len());
    println!("  peak:       {peak:.4}");
    println!("  rms:        {rms:.4}");

    Ok(())
}
