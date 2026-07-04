//! Logging example — demonstrates tracing integration.
//!
//! ghurni emits structured traces via the `tracing` crate when the
//! `logging` feature is enabled. The consuming application provides
//! the subscriber (e.g., tracing-subscriber).
//!
//! Run: `cargo run --example logging --features logging`

fn main() {
    // In production, you'd initialize a tracing subscriber here:
    // tracing_subscriber::fmt::init();

    println!("ghurni logging example");
    println!("Enable the 'logging' feature to see tracing output.");
    println!();

    // These will emit tracing::warn! when logging is enabled
    match ghurni::engine::Engine::new(
        ghurni::engine::EngineType::Gasoline,
        4,
        -1.0, // Invalid!
    ) {
        Err(e) => println!("Expected error: {e}"),
        Ok(_) => println!("Unexpected success"),
    }

    match ghurni::engine::Engine::new(
        ghurni::engine::EngineType::Gasoline,
        4,
        f32::NAN, // Invalid!
    ) {
        Err(e) => println!("Expected error: {e}"),
        Ok(_) => println!("Unexpected success"),
    }

    println!();
    println!("In your application, add tracing-subscriber to see these warnings.");
}
