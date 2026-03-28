//! Chain drive sound synthesis.
//!
//! Models the periodic rattle/clank of chain links engaging
//! sprocket teeth, common in motorcycles, bicycles, and industrial machinery.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_duration, validate_sample_rate};
#[cfg(feature = "naad-backend")]
use crate::error::GhurniError;
use crate::error::Result;
use crate::smooth::SmoothedParam;
use crate::traits::Synthesizer;

/// Chain drive synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainDrive {
    /// Number of links in the chain.
    links: u32,
    /// Number of teeth on the drive sprocket.
    sprocket_teeth: u32,
    sample_rate: f32,
    rpm: f32,
    sample_position: usize,
    smooth_rpm: SmoothedParam,
    dc_blocker: DcBlocker,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    impact_filter: naad::filter::BiquadFilter,
    #[cfg(not(feature = "naad-backend"))]
    rng: crate::rng::Rng,
}

impl ChainDrive {
    /// Creates a new chain drive synthesizer.
    ///
    /// - `links`: Number of chain links (typically 100-120 for motorcycles).
    /// - `sprocket_teeth`: Teeth on the drive sprocket (typically 14-18).
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn new(links: u32, sprocket_teeth: u32, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let links = links.clamp(10, 500);
        let sprocket_teeth = sprocket_teeth.clamp(4, 64);

        Ok(Self {
            links,
            sprocket_teeth,
            sample_rate,
            rpm: 0.0,
            sample_position: 0,
            smooth_rpm: SmoothedParam::new(0.0, 0.03, sample_rate),
            dc_blocker: DcBlocker::new(sample_rate),
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(
                naad::noise::NoiseType::White,
                links * 17 + sprocket_teeth,
            ),
            #[cfg(feature = "naad-backend")]
            impact_filter: naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                3000.0_f32.min(sample_rate * 0.49),
                3.0,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(links as u64 * 17 + sprocket_teeth as u64),
        })
    }

    /// Returns the link engagement frequency at the given sprocket RPM.
    #[must_use]
    #[inline]
    pub fn engagement_frequency(&self, rpm: f32) -> f32 {
        (rpm / 60.0) * self.sprocket_teeth as f32
    }

    /// Synthesizes chain drive sound (one-shot).
    pub fn synthesize(&mut self, rpm: f32, duration: f32) -> Result<Vec<f32>> {
        validate_duration(duration)?;
        self.set_rpm(rpm);
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills `output` with chain drive sound.
    pub fn process_block(&mut self, output: &mut [f32]) {
        self.smooth_rpm.set_target(self.rpm);

        for (i, sample) in output.iter_mut().enumerate() {
            let smooth = self.smooth_rpm.next_value();
            let engage_freq = self.engagement_frequency(smooth);

            if engage_freq < 1.0 {
                *sample = 0.0;
                continue;
            }

            let engage_period = self.sample_rate / engage_freq;
            let abs_pos = (self.sample_position + i) as f32;
            let phase = (abs_pos % engage_period) / engage_period;

            #[cfg(feature = "naad-backend")]
            {
                // Sharp impact at each link engagement
                let impact = if phase < 0.05 {
                    let t = phase / 0.05;
                    let impulse = (1.0 - t) * 0.3;
                    let ring = self.noise_gen.next_sample() * (1.0 - t);
                    impulse + self.impact_filter.process_sample(ring) * 0.2
                } else {
                    0.0
                };

                let rattle = self.noise_gen.next_sample() * 0.03;
                *sample = impact + rattle;
            }

            #[cfg(not(feature = "naad-backend"))]
            {
                let impact = if phase < 0.05 {
                    let t = phase / 0.05;
                    (1.0 - t) * 0.3 + self.rng.next_f32() * (1.0 - t) * 0.1
                } else {
                    0.0
                };

                let rattle = self.rng.next_f32() * 0.03;
                *sample = impact + rattle;
            }
        }

        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }
}

impl Synthesizer for ChainDrive {
    fn process_block(&mut self, output: &mut [f32]) {
        ChainDrive::process_block(self, output);
    }

    fn set_rpm(&mut self, rpm: f32) {
        self.rpm = rpm.clamp(0.0, 30000.0);
    }

    fn rpm(&self) -> f32 {
        self.rpm
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}
