//! Turbine and fan sound synthesis.
//!
//! Models blade pass frequency, whoosh, and tonal whine for
//! turbines, fans, propellers, and jet engines.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::{GhurniError, Result};

/// Turbine/fan synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turbine {
    blades: u32,
    duct_resonance: f32,
    sample_rate: f32,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    whoosh_lfo: naad::modulation::Lfo,
    #[cfg(not(feature = "naad-backend"))]
    rng: crate::rng::Rng,
}

impl Turbine {
    /// Creates a new turbine synthesizer.
    ///
    /// - `blades`: Number of blades/vanes (2-64).
    /// - `duct_resonance`: Duct resonant frequency in Hz. Use 0.0 for open propellers.
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn new(blades: u32, duct_resonance: f32, sample_rate: f32) -> Result<Self> {
        if sample_rate <= 0.0 {
            return Err(GhurniError::InvalidParameter(
                alloc::format!("sample_rate must be positive, got {sample_rate}"),
            ));
        }
        let blades = blades.clamp(2, 64);
        let duct_resonance = duct_resonance.max(0.0);

        // LFO rate placeholder — updated per synthesize() based on RPM
        let _initial_lfo_rate = 10.0_f32.min(sample_rate * 0.49);

        Ok(Self {
            blades,
            duct_resonance,
            sample_rate,
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(
                naad::noise::NoiseType::Pink,
                blades * 13 + duct_resonance.to_bits(),
            ),
            #[cfg(feature = "naad-backend")]
            whoosh_lfo: naad::modulation::Lfo::new(
                naad::modulation::LfoShape::Sine,
                _initial_lfo_rate,
                sample_rate,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(blades as u64 * 13 + duct_resonance.to_bits() as u64),
        })
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
    /// - `duration`: Duration in seconds.
    #[inline]
    pub fn synthesize(&mut self, rpm: f32, duration: f32) -> Result<Vec<f32>> {
        let rpm = rpm.clamp(1.0, 200000.0);
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];

        #[cfg(feature = "naad-backend")]
        self.synthesize_naad(&mut output, rpm);

        #[cfg(not(feature = "naad-backend"))]
        self.synthesize_fallback(&mut output, rpm);

        Ok(output)
    }

    #[cfg(feature = "naad-backend")]
    fn synthesize_naad(&mut self, output: &mut [f32], rpm: f32) {
        let bpf = self.blade_pass_frequency(rpm);
        let bpf_omega = core::f32::consts::TAU * bpf / self.sample_rate;
        let amp = 0.3;

        // Update LFO to blade pass rate (clamped to valid range)
        let lfo_freq = bpf.clamp(0.01, self.sample_rate * 0.49);
        let _ = self.whoosh_lfo.set_frequency(lfo_freq);

        for (i, sample) in output.iter_mut().enumerate() {
            let t = i as f32;

            // Blade pass tone + harmonics
            let tone = libm::sinf(bpf_omega * t);
            let h2 = libm::sinf(bpf_omega * 2.0 * t) * 0.4;

            // Whoosh: pink noise modulated by LFO at blade pass rate
            let whoosh_mod = 0.5 + 0.5 * self.whoosh_lfo.next_value();
            let whoosh = self.noise_gen.next_sample() * whoosh_mod * 0.2;

            // Duct resonance (if present)
            let duct = if self.duct_resonance > 0.0 {
                let duct_omega = core::f32::consts::TAU * self.duct_resonance / self.sample_rate;
                libm::sinf(duct_omega * t) * 0.15
            } else {
                0.0
            };

            *sample = (tone + h2) * amp + whoosh + duct;
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn synthesize_fallback(&mut self, output: &mut [f32], rpm: f32) {
        let bpf = self.blade_pass_frequency(rpm);
        let bpf_omega = core::f32::consts::TAU * bpf / self.sample_rate;
        let amp = 0.3;

        for (i, sample) in output.iter_mut().enumerate() {
            let tone = crate::math::f32::sin(bpf_omega * i as f32);
            let h2 = crate::math::f32::sin(bpf_omega * 2.0 * i as f32) * 0.4;

            let whoosh_mod = 0.5 + 0.5 * crate::math::f32::sin(bpf_omega * i as f32);
            let whoosh = self.rng.next_f32() * whoosh_mod * 0.2;

            let duct = if self.duct_resonance > 0.0 {
                let duct_omega = core::f32::consts::TAU * self.duct_resonance / self.sample_rate;
                crate::math::f32::sin(duct_omega * i as f32) * 0.15
            } else {
                0.0
            };

            *sample = (tone + h2) * amp + whoosh + duct;
        }
    }
}
