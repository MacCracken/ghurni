//! Simple engine synthesis example.
//!
//! Demonstrates creating a V8 engine with custom firing order,
//! synthesizing audio, and inspecting the output.
//!
//! Run: `cargo run --example simple_engine`

use ghurni::prelude::*;

fn main() -> ghurni::error::Result<()> {
    // Create a V8 gasoline engine with cross-plane firing order
    let mut engine = Engine::new(EngineType::Gasoline, 8, 44100.0)?;
    engine.set_firing_order(vec![0.0, 90.0, 270.0, 180.0, 540.0, 630.0, 450.0, 360.0]);

    // Synthesize 1 second at 3000 RPM, 60% load
    let samples = engine.synthesize(3000.0, 0.6, 1.0)?;

    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();

    println!("V8 engine at 3000 RPM:");
    println!("  samples: {}", samples.len());
    println!("  peak:    {peak:.4}");
    println!("  rms:     {rms:.4}");
    println!(
        "  firing freq: {:.1} Hz",
        engine.firing_frequency(3000.0)
    );

    Ok(())
}
