//! Electric motor sound synthesis.
//!
//! Models electromagnetic hum, commutator noise, and bearing whine
//! at RPM-dependent frequencies.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::rng::Rng;

/// Electric motor type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MotorType {
    /// DC brushed motor — commutator noise, medium hum.
    DcBrushed,
    /// AC induction — smooth 50/60Hz hum base, higher harmonics.
    AcInduction,
    /// Brushless DC (BLDC) — very smooth, high-frequency whine.
    Brushless,
    /// Servo — precise, whiny, variable speed.
    Servo,
}

/// Electric motor synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Motor {
    /// Motor type.
    motor_type: MotorType,
    /// Number of poles (affects harmonic content).
    poles: u32,
    /// PRNG.
    rng: Rng,
}

impl Motor {
    /// Creates a new motor synthesizer.
    #[must_use]
    pub fn new(motor_type: MotorType, poles: u32) -> Self {
        Self {
            motor_type,
            poles: poles.clamp(2, 24),
            rng: Rng::new(motor_type as u64 * 100 + poles as u64),
        }
    }

    /// Synthesizes motor sound at the given RPM.
    ///
    /// - `rpm`: Motor speed.
    /// - `load`: Load factor (0.0-1.0), affects strain noise.
    /// - `sample_rate`: Audio sample rate.
    /// - `duration`: Duration in seconds.
    #[inline]
    pub fn synthesize(
        &mut self,
        rpm: f32,
        load: f32,
        sample_rate: f32,
        duration: f32,
    ) -> Result<Vec<f32>> {
        let rpm = rpm.clamp(0.0, 100000.0);
        let load = load.clamp(0.0, 1.0);
        let num_samples = (sample_rate * duration) as usize;

        // Fundamental electromagnetic frequency
        let em_freq = (rpm / 60.0) * self.poles as f32;
        let em_omega = core::f32::consts::TAU * em_freq / sample_rate;

        let amp = 0.15 + load * 0.2;
        let noise_level = match self.motor_type {
            MotorType::DcBrushed => 0.15,
            MotorType::AcInduction => 0.05,
            MotorType::Brushless => 0.02,
            MotorType::Servo => 0.08,
        };

        let mut output = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            // Electromagnetic hum: fundamental + harmonics
            let fundamental = crate::math::f32::sin(em_omega * i as f32);
            let harmonic2 = crate::math::f32::sin(em_omega * 2.0 * i as f32) * 0.3;
            let harmonic3 = crate::math::f32::sin(em_omega * 3.0 * i as f32) * 0.1;
            let hum = (fundamental + harmonic2 + harmonic3) * amp;

            // Commutator/bearing noise
            let noise = self.rng.next_f32() * noise_level * amp;

            output.push(hum + noise);
        }

        Ok(output)
    }
}
