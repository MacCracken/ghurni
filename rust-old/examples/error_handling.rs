//! Error handling patterns for ghurni.
//!
//! Demonstrates how to handle errors from constructors and synthesis.
//!
//! Run: `cargo run --example error_handling`

use ghurni::prelude::*;

fn main() {
    // Pattern 1: Propagate with ?
    if let Err(e) = run_synthesis() {
        println!("Synthesis error: {e}");
    }

    // Pattern 2: Match on variant
    match Engine::new(EngineType::Gasoline, 4, f32::NAN) {
        Err(GhurniError::InvalidParameter(msg)) => {
            println!("Invalid parameter (expected): {msg}");
        }
        Err(e) => println!("Other error: {e}"),
        Ok(_) => println!("Unexpected success with NaN sample rate"),
    }

    // Pattern 3: Negative duration
    let mut engine = Engine::new(EngineType::Diesel, 6, 44100.0).unwrap();
    match engine.synthesize(2000.0, 0.5, -1.0) {
        Err(GhurniError::InvalidParameter(msg)) => {
            println!("Invalid duration (expected): {msg}");
        }
        Err(e) => println!("Other error: {e}"),
        Ok(_) => println!("Unexpected success with negative duration"),
    }
}

fn run_synthesis() -> ghurni::error::Result<()> {
    let mut engine = Engine::new(EngineType::Gasoline, 4, 44100.0)?;
    let _samples = engine.synthesize(3000.0, 0.5, 1.0)?;
    Ok(())
}
