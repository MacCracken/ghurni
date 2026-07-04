//! Gear mesh sound synthesis.
//!
//! Models the sound of meshing gear teeth: tooth mesh frequency (teeth x RPM),
//! metallic resonance, and load-dependent noise.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_duration, validate_sample_rate};
#[cfg(feature = "naad-backend")]
use crate::error::GhurniError;
use crate::error::Result;
use crate::traits::Synthesizer;

/// Gear body material — affects resonance and decay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GearMaterial {
    /// Hardened steel — bright, long ring.
    Steel,
    /// Cast iron — duller, shorter decay.
    CastIron,
    /// Brass — warm, medium ring.
    Brass,
    /// Nylon/plastic — very dull, short decay.
    Nylon,
}

impl GearMaterial {
    /// Returns (resonance_hz, decay_s, brightness).
    #[must_use]
    fn properties(self) -> (f32, f32, f32) {
        match self {
            Self::Steel => (3500.0, 0.08, 0.9),
            Self::CastIron => (2000.0, 0.04, 0.5),
            Self::Brass => (2800.0, 0.06, 0.7),
            Self::Nylon => (1000.0, 0.01, 0.2),
        }
    }
}

/// Gear mesh synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gear {
    teeth: u32,
    material: GearMaterial,
    resonance: f32,
    decay: f32,
    brightness: f32,
    sample_rate: f32,
    rpm: f32,
    sample_position: usize,
    dc_blocker: DcBlocker,
    #[cfg(feature = "naad-backend")]
    mesh_osc: naad::oscillator::Oscillator,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    resonance_filter: naad::filter::BiquadFilter,
    #[cfg(not(feature = "naad-backend"))]
    rng: crate::rng::Rng,
}

impl Gear {
    /// Creates a new gear synthesizer.
    ///
    /// - `teeth`: Number of teeth (4+).
    /// - `material`: Gear body material.
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn new(teeth: u32, material: GearMaterial, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let teeth = teeth.max(4);
        let (resonance, decay, brightness) = material.properties();
        let nyquist = sample_rate * 0.49;

        #[allow(unused_variables)]
        let initial_mesh_freq = 100.0_f32.min(nyquist);

        Ok(Self {
            teeth,
            material,
            resonance,
            decay,
            brightness,
            sample_rate,
            rpm: 1000.0,
            sample_position: 0,
            dc_blocker: DcBlocker::new(sample_rate),
            #[cfg(feature = "naad-backend")]
            mesh_osc: naad::oscillator::Oscillator::new(
                naad::oscillator::Waveform::Sine,
                initial_mesh_freq,
                sample_rate,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(
                naad::noise::NoiseType::White,
                teeth * 7 + material as u32,
            ),
            #[cfg(feature = "naad-backend")]
            resonance_filter: naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                resonance.min(nyquist),
                4.0,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(teeth as u64 * 7 + material as u64),
        })
    }

    /// Sets the shaft RPM (clamped to 1-50000).
    pub fn set_rpm(&mut self, rpm: f32) {
        self.rpm = rpm.clamp(1.0, 50000.0);
    }

    /// Returns the tooth mesh frequency at the given RPM.
    #[must_use]
    #[inline]
    pub fn mesh_frequency(&self, rpm: f32) -> f32 {
        (rpm / 60.0) * self.teeth as f32
    }

    /// Synthesizes gear mesh sound (one-shot).
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

    /// Fills `output` with gear mesh sound using current RPM.
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
        let mesh_freq = self.mesh_frequency(self.rpm);
        let nyquist = self.sample_rate * 0.49;
        let _ = self.mesh_osc.set_frequency(mesh_freq.min(nyquist));
        let amp = 0.3;

        for (i, sample) in output.iter_mut().enumerate() {
            let mesh = self.mesh_osc.next_sample() * amp * 0.5;

            let abs_pos = (self.sample_position + i) as f32;
            let mesh_phase = (abs_pos * mesh_freq / self.sample_rate) % 1.0;
            let ring_env = if mesh_phase < 0.05 {
                1.0
            } else {
                naad::dsp_util::db_to_amplitude(
                    -mesh_phase / self.decay * 20.0 / core::f32::consts::LOG10_E,
                )
            };
            let ring_excitation = self.noise_gen.next_sample() * ring_env;
            let ring =
                self.resonance_filter.process_sample(ring_excitation) * amp * self.brightness;

            let noise = self.noise_gen.next_sample() * amp * 0.05;

            *sample = mesh + ring + noise;
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn process_block_fallback(&mut self, output: &mut [f32]) {
        let mesh_freq = self.mesh_frequency(self.rpm);
        let mesh_omega = core::f32::consts::TAU * mesh_freq / self.sample_rate;
        let res_omega = core::f32::consts::TAU * self.resonance / self.sample_rate;
        let amp = 0.3;

        for (i, sample) in output.iter_mut().enumerate() {
            let abs_pos = (self.sample_position + i) as f32;
            let mesh = crate::math::f32::sin(mesh_omega * abs_pos) * amp * 0.5;

            let mesh_phase = (abs_pos * mesh_freq / self.sample_rate) % 1.0;
            let ring_env = if mesh_phase < 0.05 {
                1.0
            } else {
                crate::math::f32::exp(-mesh_phase / self.decay)
            };
            let ring =
                crate::math::f32::sin(res_omega * abs_pos) * ring_env * amp * self.brightness;

            let noise = self.rng.next_f32() * amp * 0.05;

            *sample = mesh + ring + noise;
        }
    }
}

impl Synthesizer for Gear {
    fn process_block(&mut self, output: &mut [f32]) {
        Gear::process_block(self, output);
    }

    fn set_rpm(&mut self, rpm: f32) {
        Gear::set_rpm(self, rpm);
    }

    fn rpm(&self) -> f32 {
        self.rpm
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}
