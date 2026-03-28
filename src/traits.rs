//! Common traits for mechanical sound synthesizers.

/// Common interface for all mechanical sound synthesizers.
///
/// Enables generic composition (mixers, wrappers, effects chains)
/// over any rotational/mechanical sound source.
pub trait Synthesizer: Send + Sync + core::fmt::Debug {
    /// Fills `output` with synthesized audio using current parameters.
    ///
    /// State is preserved across calls for seamless streaming.
    fn process_block(&mut self, output: &mut [f32]);

    /// Sets the rotational speed in RPM.
    fn set_rpm(&mut self, rpm: f32);

    /// Returns the current RPM.
    fn rpm(&self) -> f32;

    /// Returns the sample rate this synthesizer was created with.
    fn sample_rate(&self) -> f32;
}
