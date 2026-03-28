//! Electric motor sound synthesis.
//!
//! Models electromagnetic hum, commutator noise, and bearing whine
//! at RPM-dependent frequencies.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_duration, validate_sample_rate};
#[cfg(feature = "naad-backend")]
use crate::error::GhurniError;
use crate::error::Result;
use crate::traits::Synthesizer;

/// Electric motor type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MotorType {
    /// DC brushed motor — commutator noise, medium hum.
    DcBrushed,
    /// AC induction — smooth 50/60Hz hum base, higher harmonics.
    AcInduction,
    /// Brushless DC (BLDC) — very smooth, high-frequency whine.
    Brushless,
    /// Servo — precise, whiny, variable speed.
    Servo,
}

/// Electric motor synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Motor {
    motor_type: MotorType,
    poles: u32,
    sample_rate: f32,
    rpm: f32,
    load: f32,
    sample_position: usize,
    dc_blocker: DcBlocker,
    #[cfg(feature = "naad-backend")]
    hum_synth: naad::synth::additive::AdditiveSynth,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    noise_filter: naad::filter::BiquadFilter,
    #[cfg(not(feature = "naad-backend"))]
    rng: crate::rng::Rng,
}

impl Motor {
    /// Creates a new motor synthesizer.
    ///
    /// - `motor_type`: Type of electric motor.
    /// - `poles`: Number of magnetic poles (2-24).
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn new(motor_type: MotorType, poles: u32, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let poles = poles.clamp(2, 24);

        #[allow(unused_variables)]
        let noise_cutoff: f32 = match motor_type {
            MotorType::DcBrushed => 4000.0,
            MotorType::AcInduction => 2000.0,
            MotorType::Brushless => 6000.0,
            MotorType::Servo => 3000.0,
        };

        #[cfg(feature = "naad-backend")]
        let hum_synth = {
            let mut synth = naad::synth::additive::AdditiveSynth::new(100.0, 3, sample_rate)
                .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?;
            synth.set_partial(0, 1.0, 1.0);
            synth.set_partial(1, 2.0, 0.3);
            synth.set_partial(2, 3.0, 0.1);
            synth
        };

        Ok(Self {
            motor_type,
            poles,
            sample_rate,
            rpm: 0.0,
            load: 0.0,
            sample_position: 0,
            dc_blocker: DcBlocker::new(sample_rate),
            #[cfg(feature = "naad-backend")]
            hum_synth,
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(
                naad::noise::NoiseType::White,
                motor_type as u32 * 100 + poles,
            ),
            #[cfg(feature = "naad-backend")]
            noise_filter: naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                noise_cutoff.min(sample_rate * 0.49),
                1.0,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(motor_type as u64 * 100 + poles as u64),
        })
    }

    /// Sets the motor RPM (clamped to 0-100000).
    pub fn set_rpm(&mut self, rpm: f32) {
        self.rpm = rpm.clamp(0.0, 100000.0);
    }

    /// Sets the motor load (clamped to 0.0-1.0).
    pub fn set_load(&mut self, load: f32) {
        self.load = load.clamp(0.0, 1.0);
    }

    /// Synthesizes motor sound (one-shot).
    ///
    /// - `rpm`: Motor speed.
    /// - `load`: Load factor (0.0-1.0).
    /// - `duration`: Duration in seconds.
    pub fn synthesize(&mut self, rpm: f32, load: f32, duration: f32) -> Result<Vec<f32>> {
        validate_duration(duration)?;
        self.set_rpm(rpm);
        self.set_load(load);
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills `output` with motor sound using current RPM and load.
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
        let em_freq = (self.rpm / 60.0) * self.poles as f32;
        let nyquist = self.sample_rate * 0.49;
        let _ = self.hum_synth.set_fundamental(em_freq.min(nyquist / 3.0));

        let amp = 0.15 + self.load * 0.2;
        let noise_level = match self.motor_type {
            MotorType::DcBrushed => 0.15,
            MotorType::AcInduction => 0.05,
            MotorType::Brushless => 0.02,
            MotorType::Servo => 0.08,
        };

        for sample in output.iter_mut() {
            let hum = self.hum_synth.next_sample() * amp;
            let raw_noise = self.noise_gen.next_sample() * noise_level * amp;
            let noise = self.noise_filter.process_sample(raw_noise);
            *sample = hum + noise;
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn process_block_fallback(&mut self, output: &mut [f32]) {
        let em_freq = (self.rpm / 60.0) * self.poles as f32;
        let em_omega = core::f32::consts::TAU * em_freq / self.sample_rate;

        let amp = 0.15 + self.load * 0.2;
        let noise_level = match self.motor_type {
            MotorType::DcBrushed => 0.15,
            MotorType::AcInduction => 0.05,
            MotorType::Brushless => 0.02,
            MotorType::Servo => 0.08,
        };

        for (i, sample) in output.iter_mut().enumerate() {
            let abs_pos = (self.sample_position + i) as f32;
            let fundamental = crate::math::f32::sin(em_omega * abs_pos);
            let harmonic2 = crate::math::f32::sin(em_omega * 2.0 * abs_pos) * 0.3;
            let harmonic3 = crate::math::f32::sin(em_omega * 3.0 * abs_pos) * 0.1;
            let hum = (fundamental + harmonic2 + harmonic3) * amp;
            let noise = self.rng.next_f32() * noise_level * amp;
            *sample = hum + noise;
        }
    }
}

impl Synthesizer for Motor {
    fn process_block(&mut self, output: &mut [f32]) {
        Motor::process_block(self, output);
    }

    fn set_rpm(&mut self, rpm: f32) {
        Motor::set_rpm(self, rpm);
    }

    fn rpm(&self) -> f32 {
        self.rpm
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}
