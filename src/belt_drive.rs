//! Belt drive sound synthesis.
//!
//! Models serpentine/timing belt squeal and flap,
//! driven by pulley RPM and belt tension.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_duration, validate_sample_rate};
#[cfg(feature = "naad-backend")]
use crate::error::GhurniError;
use crate::error::Result;
use crate::smooth::SmoothedParam;
use crate::traits::Synthesizer;

/// Belt drive synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeltDrive {
    /// Pulley diameter in mm (affects belt speed).
    pulley_diameter: f32,
    /// Belt tension (0.0 = slack/squealy, 1.0 = tight/quiet).
    tension: f32,
    sample_rate: f32,
    rpm: f32,
    sample_position: usize,
    smooth_rpm: SmoothedParam,
    dc_blocker: DcBlocker,
    #[cfg(feature = "naad-backend")]
    squeal_osc: naad::oscillator::Oscillator,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    flap_filter: naad::filter::BiquadFilter,
    #[cfg(not(feature = "naad-backend"))]
    rng: crate::rng::Rng,
}

impl BeltDrive {
    /// Creates a new belt drive synthesizer.
    ///
    /// - `pulley_diameter`: Pulley diameter in mm (typically 50-200).
    /// - `tension`: Belt tension 0.0 (slack) to 1.0 (tight).
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn new(pulley_diameter: f32, tension: f32, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let pulley_diameter = pulley_diameter.clamp(20.0, 500.0);
        let tension = tension.clamp(0.0, 1.0);
        let nyquist = sample_rate * 0.49;

        // Squeal frequency depends on belt speed and tension
        #[allow(unused_variables)]
        let squeal_freq = (2000.0 + (1.0 - tension) * 3000.0).min(nyquist);

        Ok(Self {
            pulley_diameter,
            tension,
            sample_rate,
            rpm: 0.0,
            sample_position: 0,
            smooth_rpm: SmoothedParam::new(0.0, 0.03, sample_rate),
            dc_blocker: DcBlocker::new(sample_rate),
            #[cfg(feature = "naad-backend")]
            squeal_osc: naad::oscillator::Oscillator::new(
                naad::oscillator::Waveform::Sine,
                squeal_freq,
                sample_rate,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(
                naad::noise::NoiseType::Pink,
                pulley_diameter.to_bits() + tension.to_bits(),
            ),
            #[cfg(feature = "naad-backend")]
            flap_filter: naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                500.0,
                2.0,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(pulley_diameter.to_bits() as u64),
        })
    }

    /// Sets belt tension (0.0 slack / squealy, 1.0 tight / quiet).
    pub fn set_tension(&mut self, tension: f32) {
        self.tension = tension.clamp(0.0, 1.0);
    }

    /// Returns the belt linear speed in m/s.
    #[must_use]
    #[inline]
    pub fn belt_speed(&self, rpm: f32) -> f32 {
        // v = π * d * RPM / 60000 (d in mm -> m)
        core::f32::consts::PI * self.pulley_diameter * rpm / 60000.0
    }

    /// Synthesizes belt drive sound (one-shot).
    pub fn synthesize(&mut self, rpm: f32, duration: f32) -> Result<Vec<f32>> {
        validate_duration(duration)?;
        self.set_rpm(rpm);
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills `output` with belt drive sound.
    pub fn process_block(&mut self, output: &mut [f32]) {
        self.smooth_rpm.set_target(self.rpm);
        #[allow(unused_variables)]
        let nyquist = self.sample_rate * 0.49;

        // Squeal amplitude inversely proportional to tension
        let squeal_amp = (1.0 - self.tension) * 0.3;
        // Flap depends on belt speed
        let speed = self.belt_speed(self.rpm);
        let flap_amp = (speed / 20.0).clamp(0.0, 0.2);

        // Belt rotation frequency (one full loop)
        let belt_circumference = core::f32::consts::PI * self.pulley_diameter / 1000.0; // meters
        let belt_rotation_freq = if belt_circumference > 0.0 {
            speed / belt_circumference.max(0.1)
        } else {
            0.0
        };

        for (i, sample) in output.iter_mut().enumerate() {
            let _smooth = self.smooth_rpm.next_value();

            #[cfg(feature = "naad-backend")]
            {
                // Squeal: high-frequency friction oscillation
                let squeal_freq = (2000.0 + (1.0 - self.tension) * 3000.0).min(nyquist);
                let _ = self.squeal_osc.set_frequency(squeal_freq);
                let squeal = self.squeal_osc.next_sample() * squeal_amp;

                // Flap: periodic low-frequency thump from belt joints/splices
                let raw_flap = self.noise_gen.next_sample() * flap_amp;
                let flap = self.flap_filter.process_sample(raw_flap);

                // Periodic modulation at belt rotation rate
                let abs_pos = (self.sample_position + i) as f32;
                let belt_mod = if belt_rotation_freq > 0.0 {
                    let belt_omega = core::f32::consts::TAU * belt_rotation_freq / self.sample_rate;
                    0.8 + 0.2 * libm::sinf(belt_omega * abs_pos)
                } else {
                    1.0
                };

                *sample = (squeal + flap) * belt_mod;
            }

            #[cfg(not(feature = "naad-backend"))]
            {
                let abs_pos = (self.sample_position + i) as f32;
                let squeal_freq = 2000.0 + (1.0 - self.tension) * 3000.0;
                let squeal_omega = core::f32::consts::TAU * squeal_freq / self.sample_rate;
                let squeal = crate::math::f32::sin(squeal_omega * abs_pos) * squeal_amp;

                let flap = self.rng.next_f32() * flap_amp;

                let belt_mod = if belt_rotation_freq > 0.0 {
                    let belt_omega = core::f32::consts::TAU * belt_rotation_freq / self.sample_rate;
                    0.8 + 0.2 * crate::math::f32::sin(belt_omega * abs_pos)
                } else {
                    1.0
                };

                *sample = (squeal + flap) * belt_mod;
            }
        }

        for sample in output.iter_mut() {
            *sample = self.dc_blocker.process(*sample);
        }
        self.sample_position += output.len();
    }
}

impl Synthesizer for BeltDrive {
    fn process_block(&mut self, output: &mut [f32]) {
        BeltDrive::process_block(self, output);
    }

    fn set_rpm(&mut self, rpm: f32) {
        self.rpm = rpm.clamp(0.0, 20000.0);
    }

    fn rpm(&self) -> f32 {
        self.rpm
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}
