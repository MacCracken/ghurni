//! Turbine and fan sound synthesis.
//!
//! Models blade pass frequency, whoosh, and tonal whine for
//! turbines, fans, propellers, and jet engines.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::rng::Rng;

/// Turbine/fan synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turbine {
    /// Number of blades.
    blades: u32,
    /// Duct resonant frequency (Hz). 0 = open/unducted.
    duct_resonance: f32,
    /// PRNG.
    rng: Rng,
}

impl Turbine {
    /// Creates a new turbine synthesizer.
    ///
    /// - `blades`: Number of blades/vanes (2-64).
    /// - `duct_resonance`: Duct resonant frequency in Hz. Use 0.0 for open propellers.
    #[must_use]
    pub fn new(blades: u32, duct_resonance: f32) -> Self {
        Self {
            blades: blades.clamp(2, 64),
            duct_resonance: duct_resonance.max(0.0),
            rng: Rng::new(blades as u64 * 13 + duct_resonance.to_bits() as u64),
        }
    }

    /// Returns blade pass frequency at the given RPM.
    #[must_use]
    #[inline]
    pub fn blade_pass_frequency(&self, rpm: f32) -> f32 {
        (rpm / 60.0) * self.blades as f32
    }

    /// Synthesizes turbine sound.
    ///
    /// - `rpm`: Shaft speed.
    /// - `sample_rate`: Audio sample rate.
    /// - `duration`: Duration in seconds.
    #[inline]
    pub fn synthesize(&mut self, rpm: f32, sample_rate: f32, duration: f32) -> Result<Vec<f32>> {
        let rpm = rpm.clamp(1.0, 200000.0);
        let num_samples = (sample_rate * duration) as usize;
        let bpf = self.blade_pass_frequency(rpm);
        let bpf_omega = core::f32::consts::TAU * bpf / sample_rate;

        let amp = 0.3;
        let mut output = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            // Blade pass tone + harmonics
            let tone = crate::math::f32::sin(bpf_omega * i as f32);
            let h2 = crate::math::f32::sin(bpf_omega * 2.0 * i as f32) * 0.4;

            // Whoosh: broadband noise modulated by blade pass
            let whoosh_mod = 0.5 + 0.5 * crate::math::f32::sin(bpf_omega * i as f32);
            let whoosh = self.rng.next_f32() * whoosh_mod * 0.2;

            // Duct resonance (if present)
            let duct = if self.duct_resonance > 0.0 {
                let duct_omega = core::f32::consts::TAU * self.duct_resonance / sample_rate;
                crate::math::f32::sin(duct_omega * i as f32) * 0.15
            } else {
                0.0
            };

            output.push((tone + h2) * amp + whoosh + duct);
        }

        Ok(output)
    }
}
