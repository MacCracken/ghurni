//! Clock and precision mechanism sound synthesis.
//!
//! Models the tick-tock of escapements, spring resonance, and the
//! delicate mechanical sounds of horology.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_duration, validate_sample_rate};
#[cfg(feature = "naad-backend")]
use crate::error::GhurniError;
use crate::error::Result;
use crate::traits::Synthesizer;

/// Clock mechanism type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClockType {
    /// Wristwatch — tiny, high-frequency tick, metallic.
    Wristwatch,
    /// Wall clock — medium tick, wood resonance body.
    WallClock,
    /// Grandfather clock — deep tick, large pendulum, wood case resonance.
    GrandfatherClock,
    /// Pocket watch — small, bright, metallic case.
    PocketWatch,
}

/// Clock mechanism synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clock {
    clock_type: ClockType,
    tick_rate: f32,
    sample_rate: f32,
    sample_position: usize,
    dc_blocker: DcBlocker,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    body_filter: naad::filter::BiquadFilter,
    #[cfg(not(feature = "naad-backend"))]
    rng: crate::rng::Rng,
}

impl Clock {
    /// Creates a new clock synthesizer.
    ///
    /// - `clock_type`: Type of clock mechanism.
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn new(clock_type: ClockType, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let tick_rate = match clock_type {
            ClockType::Wristwatch => 8.0,
            ClockType::WallClock => 2.0,
            ClockType::GrandfatherClock => 1.0,
            ClockType::PocketWatch => 5.0,
        };

        #[allow(unused_variables)]
        let (resonance, decay, amp): (f32, f32, f32) = match clock_type {
            ClockType::Wristwatch => (6000.0, 0.003, 0.15),
            ClockType::WallClock => (2000.0, 0.01, 0.4),
            ClockType::GrandfatherClock => (800.0, 0.03, 0.6),
            ClockType::PocketWatch => (4500.0, 0.005, 0.25),
        };

        Ok(Self {
            clock_type,
            tick_rate,
            sample_rate,
            sample_position: 0,
            dc_blocker: DcBlocker::new(sample_rate),
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(
                naad::noise::NoiseType::White,
                clock_type as u32 * 997,
            ),
            #[cfg(feature = "naad-backend")]
            body_filter: naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                resonance.min(sample_rate * 0.49),
                8.0,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(clock_type as u64 * 997),
        })
    }

    /// Synthesizes clock ticking sound (one-shot).
    ///
    /// - `duration`: Duration in seconds.
    pub fn synthesize(&mut self, duration: f32) -> Result<Vec<f32>> {
        validate_duration(duration)?;
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills `output` with clock ticking sound.
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
        let tick_period = self.sample_rate / self.tick_rate;

        let (_resonance, decay, amp) = match self.clock_type {
            ClockType::Wristwatch => (6000.0, 0.003, 0.15),
            ClockType::WallClock => (2000.0, 0.01, 0.4),
            ClockType::GrandfatherClock => (800.0, 0.03, 0.6),
            ClockType::PocketWatch => (4500.0, 0.005, 0.25),
        };

        for (i, sample) in output.iter_mut().enumerate() {
            let abs_pos = (self.sample_position + i) as f32;
            let phase = (abs_pos % tick_period) / tick_period;

            let tick = if phase < 0.02 {
                let t = phase / 0.02;
                let impulse = (1.0 - t) * amp;
                let ring_excitation = self.noise_gen.next_sample() * amp;
                let ring = self.body_filter.process_sample(ring_excitation)
                    * naad::dsp_util::db_to_amplitude(
                        -phase / decay * 20.0 / core::f32::consts::LOG10_E,
                    );
                impulse * 0.5 + ring
            } else if phase < 0.15 {
                let ring_excitation = self.noise_gen.next_sample() * amp * 0.1;
                self.body_filter.process_sample(ring_excitation)
                    * naad::dsp_util::db_to_amplitude(
                        -phase / decay * 20.0 / core::f32::consts::LOG10_E,
                    )
                    * 0.3
            } else {
                0.0
            };

            let mech = self.noise_gen.next_sample() * amp * 0.01;

            *sample = tick + mech;
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn process_block_fallback(&mut self, output: &mut [f32]) {
        let tick_period = self.sample_rate / self.tick_rate;

        let (resonance, decay, amp) = match self.clock_type {
            ClockType::Wristwatch => (6000.0, 0.003, 0.15),
            ClockType::WallClock => (2000.0, 0.01, 0.4),
            ClockType::GrandfatherClock => (800.0, 0.03, 0.6),
            ClockType::PocketWatch => (4500.0, 0.005, 0.25),
        };

        let res_omega = core::f32::consts::TAU * resonance / self.sample_rate;

        for (i, sample) in output.iter_mut().enumerate() {
            let abs_pos = (self.sample_position + i) as f32;
            let phase = (abs_pos % tick_period) / tick_period;

            let tick = if phase < 0.02 {
                let t = phase / 0.02;
                let impulse = (1.0 - t) * amp;
                let ring = crate::math::f32::sin(res_omega * abs_pos)
                    * crate::math::f32::exp(-phase / decay)
                    * amp;
                impulse * 0.5 + ring
            } else if phase < 0.15 {
                crate::math::f32::sin(res_omega * abs_pos)
                    * crate::math::f32::exp(-phase / decay)
                    * amp
                    * 0.3
            } else {
                0.0
            };

            let mech = self.rng.next_f32() * amp * 0.01;

            *sample = tick + mech;
        }
    }
}

impl Synthesizer for Clock {
    fn process_block(&mut self, output: &mut [f32]) {
        Clock::process_block(self, output);
    }

    fn set_rpm(&mut self, _rpm: f32) {
        // Clock tick rate is fixed by type, not RPM-driven.
    }

    fn rpm(&self) -> f32 {
        // Return tick rate as equivalent "RPM" for interface compatibility.
        self.tick_rate * 60.0
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}
