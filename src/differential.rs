//! Differential (final drive) sound synthesis.
//!
//! Models the characteristic whine of hypoid gear mesh in the
//! differential, distinct from transmission gears.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_duration, validate_sample_rate};
#[cfg(feature = "naad-backend")]
use crate::error::GhurniError;
use crate::error::Result;
use crate::smooth::SmoothedParam;
use crate::traits::Synthesizer;

/// Differential synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Differential {
    /// Ring gear teeth.
    ring_teeth: u32,
    /// Pinion gear teeth.
    pinion_teeth: u32,
    sample_rate: f32,
    rpm: f32,
    sample_position: usize,
    smooth_rpm: SmoothedParam,
    dc_blocker: DcBlocker,
    #[cfg(feature = "naad-backend")]
    whine_osc: naad::oscillator::Oscillator,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    resonance_filter: naad::filter::BiquadFilter,
    #[cfg(not(feature = "naad-backend"))]
    rng: crate::rng::Rng,
}

impl Differential {
    /// Creates a new differential synthesizer.
    ///
    /// - `ring_teeth`: Teeth on the ring gear (typically 35-45).
    /// - `pinion_teeth`: Teeth on the pinion gear (typically 8-14).
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn new(ring_teeth: u32, pinion_teeth: u32, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let ring_teeth = ring_teeth.clamp(20, 80);
        let pinion_teeth = pinion_teeth.clamp(4, 30);
        let nyquist = sample_rate * 0.49;

        // Resonant frequency of the differential housing
        #[allow(unused_variables)]
        let housing_resonance = 2500.0_f32.min(nyquist);

        Ok(Self {
            ring_teeth,
            pinion_teeth,
            sample_rate,
            rpm: 0.0,
            sample_position: 0,
            smooth_rpm: SmoothedParam::new(0.0, 0.05, sample_rate),
            dc_blocker: DcBlocker::new(sample_rate),
            #[cfg(feature = "naad-backend")]
            whine_osc: naad::oscillator::Oscillator::new(
                naad::oscillator::Waveform::Sine,
                100.0,
                sample_rate,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(
                naad::noise::NoiseType::White,
                ring_teeth * 41 + pinion_teeth,
            ),
            #[cfg(feature = "naad-backend")]
            resonance_filter: naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                housing_resonance,
                6.0,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(ring_teeth as u64 * 41 + pinion_teeth as u64),
        })
    }

    /// Returns the gear ratio (ring/pinion).
    #[must_use]
    #[inline]
    pub fn ratio(&self) -> f32 {
        self.ring_teeth as f32 / self.pinion_teeth as f32
    }

    /// Returns the mesh frequency at the given driveshaft RPM.
    #[must_use]
    #[inline]
    pub fn mesh_frequency(&self, driveshaft_rpm: f32) -> f32 {
        (driveshaft_rpm / 60.0) * self.pinion_teeth as f32
    }

    /// Synthesizes differential whine (one-shot).
    pub fn synthesize(&mut self, rpm: f32, duration: f32) -> Result<Vec<f32>> {
        validate_duration(duration)?;
        self.set_rpm(rpm);
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills `output` with differential whine.
    pub fn process_block(&mut self, output: &mut [f32]) {
        self.smooth_rpm.set_target(self.rpm);
        let nyquist = self.sample_rate * 0.49;

        for sample in output.iter_mut() {
            let smooth = self.smooth_rpm.next_value();
            let mesh_freq = self.mesh_frequency(smooth);

            #[cfg(feature = "naad-backend")]
            {
                let _ = self.whine_osc.set_frequency(mesh_freq.clamp(1.0, nyquist));
                let whine = self.whine_osc.next_sample() * 0.2;

                // High-Q housing resonance excited by mesh
                let excitation = self.noise_gen.next_sample() * 0.1;
                let ring = self.resonance_filter.process_sample(excitation) * 0.15;

                let noise = self.noise_gen.next_sample() * 0.02;
                *sample = whine + ring + noise;
            }

            #[cfg(not(feature = "naad-backend"))]
            {
                let abs_pos = self.sample_position as f32;
                let mesh_omega =
                    core::f32::consts::TAU * mesh_freq.clamp(1.0, nyquist) / self.sample_rate;
                let whine = crate::math::f32::sin(mesh_omega * abs_pos) * 0.2;
                let noise = self.rng.next_f32() * 0.02;
                *sample = whine + noise;
            }
        }

        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }
}

impl Synthesizer for Differential {
    fn process_block(&mut self, output: &mut [f32]) {
        Differential::process_block(self, output);
    }

    fn set_rpm(&mut self, rpm: f32) {
        self.rpm = rpm.clamp(0.0, 50000.0);
    }

    fn rpm(&self) -> f32 {
        self.rpm
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}
