//! Turbine and fan sound synthesis.
//!
//! Models blade pass frequency, whoosh, and tonal whine for
//! turbines, fans, propellers, and jet engines.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_duration, validate_sample_rate};
#[cfg(feature = "naad-backend")]
use crate::error::GhurniError;
use crate::error::Result;
use crate::traits::Synthesizer;

/// Turbine/fan synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turbine {
    blades: u32,
    duct_resonance: f32,
    sample_rate: f32,
    rpm: f32,
    sample_position: usize,
    dc_blocker: DcBlocker,
    #[cfg(feature = "naad-backend")]
    blade_synth: naad::synth::additive::AdditiveSynth,
    #[cfg(feature = "naad-backend")]
    duct_osc: Option<naad::oscillator::Oscillator>,
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
        validate_sample_rate(sample_rate)?;
        let blades = blades.clamp(2, 64);
        let duct_resonance = duct_resonance.max(0.0);
        let nyquist = sample_rate * 0.49;

        #[cfg(feature = "naad-backend")]
        let blade_synth = {
            let mut synth = naad::synth::additive::AdditiveSynth::new(100.0, 2, sample_rate)
                .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?;
            synth.set_partial(0, 1.0, 1.0);
            synth.set_partial(1, 2.0, 0.4);
            synth
        };

        #[cfg(feature = "naad-backend")]
        let duct_osc = if duct_resonance > 0.0 {
            Some(
                naad::oscillator::Oscillator::new(
                    naad::oscillator::Waveform::Sine,
                    duct_resonance.min(nyquist),
                    sample_rate,
                )
                .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            )
        } else {
            None
        };

        #[allow(unused_variables)]
        let initial_lfo_rate = 10.0_f32.min(nyquist);

        Ok(Self {
            blades,
            duct_resonance,
            sample_rate,
            rpm: 1000.0,
            sample_position: 0,
            dc_blocker: DcBlocker::new(sample_rate),
            #[cfg(feature = "naad-backend")]
            blade_synth,
            #[cfg(feature = "naad-backend")]
            duct_osc,
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(
                naad::noise::NoiseType::Pink,
                blades * 13 + duct_resonance.to_bits(),
            ),
            #[cfg(feature = "naad-backend")]
            whoosh_lfo: naad::modulation::Lfo::new(
                naad::modulation::LfoShape::Sine,
                initial_lfo_rate,
                sample_rate,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(blades as u64 * 13 + duct_resonance.to_bits() as u64),
        })
    }

    /// Sets the shaft RPM (clamped to 1-200000).
    pub fn set_rpm(&mut self, rpm: f32) {
        self.rpm = rpm.clamp(1.0, 200000.0);
    }

    /// Returns blade pass frequency at the given RPM.
    #[must_use]
    #[inline]
    pub fn blade_pass_frequency(&self, rpm: f32) -> f32 {
        (rpm / 60.0) * self.blades as f32
    }

    /// Synthesizes turbine sound (one-shot).
    ///
    /// - `rpm`: Shaft speed.
    /// - `duration`: Duration in seconds.
    pub fn synthesize(&mut self, rpm: f32, duration: f32) -> Result<Vec<f32>> {
        validate_duration(duration)?;
        self.set_rpm(rpm);
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills `output` with turbine sound using current RPM.
    #[inline]
    pub fn process_block(&mut self, output: &mut [f32]) {
        #[cfg(feature = "naad-backend")]
        self.process_block_naad(output);

        #[cfg(not(feature = "naad-backend"))]
        self.process_block_fallback(output);

        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }

    #[cfg(feature = "naad-backend")]
    fn process_block_naad(&mut self, output: &mut [f32]) {
        let bpf = self.blade_pass_frequency(self.rpm);
        let nyquist = self.sample_rate * 0.49;
        let amp = 0.3;

        let _ = self.blade_synth.set_fundamental(bpf.min(nyquist / 2.0));
        let lfo_freq = bpf.clamp(0.01, nyquist);
        let _ = self.whoosh_lfo.set_frequency(lfo_freq);

        for sample in output.iter_mut() {
            let tone = self.blade_synth.next_sample();
            let whoosh_mod = 0.5 + 0.5 * self.whoosh_lfo.next_value();
            let whoosh = self.noise_gen.next_sample() * whoosh_mod * 0.2;
            let duct = match &mut self.duct_osc {
                Some(osc) => osc.next_sample() * 0.15,
                None => 0.0,
            };
            *sample = tone * amp + whoosh + duct;
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn process_block_fallback(&mut self, output: &mut [f32]) {
        let bpf = self.blade_pass_frequency(self.rpm);
        let bpf_omega = core::f32::consts::TAU * bpf / self.sample_rate;
        let amp = 0.3;

        for (i, sample) in output.iter_mut().enumerate() {
            let abs_pos = (self.sample_position + i) as f32;
            let tone = crate::math::f32::sin(bpf_omega * abs_pos);
            let h2 = crate::math::f32::sin(bpf_omega * 2.0 * abs_pos) * 0.4;

            let whoosh_mod = 0.5 + 0.5 * crate::math::f32::sin(bpf_omega * abs_pos);
            let whoosh = self.rng.next_f32() * whoosh_mod * 0.2;

            let duct = if self.duct_resonance > 0.0 {
                let duct_omega = core::f32::consts::TAU * self.duct_resonance / self.sample_rate;
                crate::math::f32::sin(duct_omega * abs_pos) * 0.15
            } else {
                0.0
            };

            *sample = (tone + h2) * amp + whoosh + duct;
        }
    }
}

impl Synthesizer for Turbine {
    fn process_block(&mut self, output: &mut [f32]) {
        Turbine::process_block(self, output);
    }

    fn set_rpm(&mut self, rpm: f32) {
        Turbine::set_rpm(self, rpm);
    }

    fn rpm(&self) -> f32 {
        self.rpm
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}
