//! Turbocharger and supercharger sound synthesis.
//!
//! Models the whine of forced induction systems: turbo spool with
//! lag/inertia, supercharger direct-drive whine, and blow-off valve bursts.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_duration, validate_sample_rate};
#[cfg(feature = "naad-backend")]
use crate::error::GhurniError;
use crate::error::Result;
use crate::smooth::SmoothedParam;
use crate::traits::Synthesizer;

/// Forced induction type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InductionType {
    /// Turbocharger — exhaust-driven, has spool lag.
    Turbo,
    /// Supercharger — belt-driven, direct RPM coupling.
    Supercharger,
}

/// Forced induction synthesizer (turbocharger / supercharger).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForcedInduction {
    induction_type: InductionType,
    /// Drive ratio (supercharger) or max boost RPM multiplier (turbo).
    drive_ratio: f32,
    /// Spool inertia — how quickly turbo responds to RPM changes (seconds).
    spool_inertia: f32,
    sample_rate: f32,
    rpm: f32,
    load: f32,
    /// Whether blow-off valve is active.
    bov_active: bool,
    bov_remaining: usize,
    sample_position: usize,
    spool_rpm: SmoothedParam,
    dc_blocker: DcBlocker,
    #[cfg(feature = "naad-backend")]
    whine_osc: naad::oscillator::Oscillator,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    bov_filter: naad::filter::BiquadFilter,
    #[cfg(not(feature = "naad-backend"))]
    rng: crate::rng::Rng,
}

impl ForcedInduction {
    /// Creates a new forced induction synthesizer.
    ///
    /// - `induction_type`: Turbo or Supercharger.
    /// - `drive_ratio`: RPM multiplier for the compressor.
    /// - `spool_inertia`: Spool-up time constant in seconds (turbo only, ignored for supercharger).
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn new(
        induction_type: InductionType,
        drive_ratio: f32,
        spool_inertia: f32,
        sample_rate: f32,
    ) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let drive_ratio = drive_ratio.clamp(0.5, 10.0);
        let spool_inertia = match induction_type {
            InductionType::Turbo => spool_inertia.clamp(0.1, 5.0),
            InductionType::Supercharger => 0.01, // Near-instant response
        };

        Ok(Self {
            induction_type,
            drive_ratio,
            spool_inertia,
            sample_rate,
            rpm: 0.0,
            load: 0.0,
            bov_active: false,
            bov_remaining: 0,
            sample_position: 0,
            spool_rpm: SmoothedParam::new(0.0, spool_inertia, sample_rate),
            dc_blocker: DcBlocker::new(sample_rate),
            #[cfg(feature = "naad-backend")]
            whine_osc: naad::oscillator::Oscillator::new(
                naad::oscillator::Waveform::Saw,
                100.0,
                sample_rate,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(
                naad::noise::NoiseType::White,
                induction_type as u32 * 777 + drive_ratio.to_bits(),
            ),
            #[cfg(feature = "naad-backend")]
            bov_filter: naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                2000.0,
                2.0,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(induction_type as u64 * 777),
        })
    }

    /// Sets the engine RPM that drives this induction system.
    pub fn set_rpm(&mut self, rpm: f32) {
        self.rpm = rpm.clamp(0.0, 15000.0);
        let target_spool = self.rpm * self.drive_ratio * self.load;
        self.spool_rpm.set_target(target_spool);
    }

    /// Sets the load/boost demand (0.0-1.0).
    pub fn set_load(&mut self, load: f32) {
        let prev_load = self.load;
        self.load = load.clamp(0.0, 1.0);
        // Trigger BOV when load drops sharply while spool is high
        if prev_load > 0.5 && self.load < 0.2 && self.spool_rpm.current() > 5000.0 {
            self.trigger_bov();
        }
        let target_spool = self.rpm * self.drive_ratio * self.load;
        self.spool_rpm.set_target(target_spool);
    }

    /// Triggers a blow-off valve burst.
    pub fn trigger_bov(&mut self) {
        self.bov_active = true;
        self.bov_remaining = (self.sample_rate * 0.15) as usize; // 150ms burst
    }

    /// Synthesizes forced induction sound (one-shot).
    pub fn synthesize(&mut self, rpm: f32, load: f32, duration: f32) -> Result<Vec<f32>> {
        validate_duration(duration)?;
        self.set_rpm(rpm);
        self.set_load(load);
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills `output` with forced induction sound.
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
        let nyquist = self.sample_rate * 0.49;

        for sample in output.iter_mut() {
            let spool = self.spool_rpm.next_value();
            let whine_freq = (spool / 60.0).clamp(20.0, nyquist);
            let _ = self.whine_osc.set_frequency(whine_freq);

            // Whine amplitude scales with spool speed
            let whine_amp = (spool / 20000.0).clamp(0.0, 0.4);
            let whine = self.whine_osc.next_sample() * whine_amp;

            // Compressor noise — broadband hiss proportional to boost
            let hiss = self.noise_gen.next_sample() * whine_amp * 0.15;

            // BOV burst
            let bov = if self.bov_remaining > 0 {
                self.bov_remaining -= 1;
                let env = self.bov_remaining as f32 / (self.sample_rate * 0.15);
                let raw = self.noise_gen.next_sample() * env * 0.6;
                self.bov_filter.process_sample(raw)
            } else {
                self.bov_active = false;
                0.0
            };

            *sample = whine + hiss + bov;
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn process_block_fallback(&mut self, output: &mut [f32]) {
        let nyquist = self.sample_rate * 0.49;

        for (i, sample) in output.iter_mut().enumerate() {
            let spool = self.spool_rpm.next_value();
            let whine_freq = (spool / 60.0).clamp(20.0, nyquist);
            let whine_omega = core::f32::consts::TAU * whine_freq / self.sample_rate;
            let abs_pos = (self.sample_position + i) as f32;

            let whine_amp = (spool / 20000.0).clamp(0.0, 0.4);
            let whine = crate::math::f32::sin(whine_omega * abs_pos) * whine_amp;

            let hiss = self.rng.next_f32() * whine_amp * 0.15;

            let bov = if self.bov_remaining > 0 {
                self.bov_remaining -= 1;
                let env = self.bov_remaining as f32 / (self.sample_rate * 0.15);
                self.rng.next_f32() * env * 0.3
            } else {
                self.bov_active = false;
                0.0
            };

            *sample = whine + hiss + bov;
        }
    }
}

impl Synthesizer for ForcedInduction {
    fn process_block(&mut self, output: &mut [f32]) {
        ForcedInduction::process_block(self, output);
    }

    fn set_rpm(&mut self, rpm: f32) {
        ForcedInduction::set_rpm(self, rpm);
    }

    fn rpm(&self) -> f32 {
        self.rpm
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}
