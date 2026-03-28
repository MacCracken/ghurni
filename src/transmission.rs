//! Transmission / gearbox sound synthesis.
//!
//! Models gear mesh whine at the current ratio, synchronizer noise
//! during shifts, and clutch engagement transients.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_duration, validate_sample_rate};
#[cfg(feature = "naad-backend")]
use crate::error::GhurniError;
use crate::error::Result;
use crate::smooth::SmoothedParam;
use crate::traits::Synthesizer;

/// Transmission synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transmission {
    /// Gear ratios (index 0 = 1st gear).
    ratios: Vec<f32>,
    /// Number of teeth on the output gear (affects mesh frequency).
    output_teeth: u32,
    current_gear: u32,
    sample_rate: f32,
    rpm: f32,
    /// Shift transient remaining samples.
    shift_remaining: usize,
    shift_duration_samples: usize,
    sample_position: usize,
    mesh_rpm: SmoothedParam,
    dc_blocker: DcBlocker,
    #[cfg(feature = "naad-backend")]
    mesh_osc: naad::oscillator::Oscillator,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    synchro_filter: naad::filter::BiquadFilter,
    #[cfg(not(feature = "naad-backend"))]
    rng: crate::rng::Rng,
}

impl Transmission {
    /// Creates a new transmission synthesizer.
    ///
    /// - `ratios`: Gear ratios (e.g., `[3.5, 2.1, 1.4, 1.0, 0.8]` for a 5-speed).
    /// - `output_teeth`: Teeth on the output shaft gear (affects mesh tone).
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn new(ratios: Vec<f32>, output_teeth: u32, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let output_teeth = output_teeth.clamp(8, 64);
        let shift_duration_samples = (sample_rate * 0.2) as usize; // 200ms shift

        Ok(Self {
            ratios,
            output_teeth,
            current_gear: 0,
            sample_rate,
            rpm: 0.0,
            shift_remaining: 0,
            shift_duration_samples,
            sample_position: 0,
            mesh_rpm: SmoothedParam::new(0.0, 0.05, sample_rate),
            dc_blocker: DcBlocker::new(sample_rate),
            #[cfg(feature = "naad-backend")]
            mesh_osc: naad::oscillator::Oscillator::new(
                naad::oscillator::Waveform::Sine,
                100.0,
                sample_rate,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(
                naad::noise::NoiseType::White,
                output_teeth * 31,
            ),
            #[cfg(feature = "naad-backend")]
            synchro_filter: naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                4000.0,
                3.0,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(output_teeth as u64 * 31),
        })
    }

    /// Shifts to the specified gear (0-indexed). Triggers shift transient.
    pub fn shift_to(&mut self, gear: u32) {
        if (gear as usize) < self.ratios.len() {
            self.current_gear = gear;
            self.shift_remaining = self.shift_duration_samples;
        }
    }

    /// Returns the current gear ratio.
    #[must_use]
    pub fn current_ratio(&self) -> f32 {
        self.ratios
            .get(self.current_gear as usize)
            .copied()
            .unwrap_or(1.0)
    }

    /// Returns the output shaft RPM.
    #[must_use]
    #[inline]
    pub fn output_rpm(&self) -> f32 {
        self.rpm / self.current_ratio()
    }

    /// Synthesizes transmission sound (one-shot).
    pub fn synthesize(&mut self, rpm: f32, duration: f32) -> Result<Vec<f32>> {
        validate_duration(duration)?;
        self.set_rpm(rpm);
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills `output` with transmission sound.
    pub fn process_block(&mut self, output: &mut [f32]) {
        let out_rpm = self.output_rpm();
        self.mesh_rpm.set_target(out_rpm);
        let nyquist = self.sample_rate * 0.49;

        for sample in output.iter_mut() {
            let smooth_rpm = self.mesh_rpm.next_value();
            let mesh_freq = (smooth_rpm / 60.0) * self.output_teeth as f32;

            #[cfg(feature = "naad-backend")]
            {
                let _ = self.mesh_osc.set_frequency(mesh_freq.clamp(1.0, nyquist));
                let mesh = self.mesh_osc.next_sample() * 0.15;

                // Synchro whine during shift
                let synchro = if self.shift_remaining > 0 {
                    self.shift_remaining -= 1;
                    let env = self.shift_remaining as f32 / self.shift_duration_samples as f32;
                    let raw = self.noise_gen.next_sample() * env * 0.3;
                    self.synchro_filter.process_sample(raw)
                } else {
                    0.0
                };

                let noise = self.noise_gen.next_sample() * 0.02;
                *sample = mesh + synchro + noise;
            }

            #[cfg(not(feature = "naad-backend"))]
            {
                let mesh_omega = core::f32::consts::TAU * mesh_freq.clamp(1.0, nyquist)
                    / self.sample_rate;
                let abs_pos = self.sample_position as f32;
                let mesh = crate::math::f32::sin(mesh_omega * abs_pos) * 0.15;

                let synchro = if self.shift_remaining > 0 {
                    self.shift_remaining -= 1;
                    let env = self.shift_remaining as f32 / self.shift_duration_samples as f32;
                    self.rng.next_f32() * env * 0.15
                } else {
                    0.0
                };

                let noise = self.rng.next_f32() * 0.02;
                *sample = mesh + synchro + noise;
                self.sample_position += 1;
            }

            #[cfg(feature = "naad-backend")]
            {
                self.sample_position += 1;
            }
        }

        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
    }
}

impl Synthesizer for Transmission {
    fn process_block(&mut self, output: &mut [f32]) {
        Transmission::process_block(self, output);
    }

    fn set_rpm(&mut self, rpm: f32) {
        self.rpm = rpm.clamp(0.0, 15000.0);
    }

    fn rpm(&self) -> f32 {
        self.rpm
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}
