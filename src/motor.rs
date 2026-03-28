//! Electric motor sound synthesis.
//!
//! Models electromagnetic hum, commutator noise, and bearing whine
//! at RPM-dependent frequencies.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::{GhurniError, Result};

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
        if sample_rate <= 0.0 {
            return Err(GhurniError::InvalidParameter(
                alloc::format!("sample_rate must be positive, got {sample_rate}"),
            ));
        }
        let poles = poles.clamp(2, 24);

        let _noise_cutoff: f32 = match motor_type {
            MotorType::DcBrushed => 4000.0,
            MotorType::AcInduction => 2000.0,
            MotorType::Brushless => 6000.0,
            MotorType::Servo => 3000.0,
        };

        Ok(Self {
            motor_type,
            poles,
            sample_rate,
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(
                naad::noise::NoiseType::White,
                motor_type as u32 * 100 + poles,
            ),
            #[cfg(feature = "naad-backend")]
            noise_filter: naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                _noise_cutoff.min(sample_rate * 0.49),
                1.0,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(motor_type as u64 * 100 + poles as u64),
        })
    }

    /// Synthesizes motor sound at the given RPM.
    ///
    /// - `rpm`: Motor speed.
    /// - `load`: Load factor (0.0-1.0), affects strain noise.
    /// - `duration`: Duration in seconds.
    #[inline]
    pub fn synthesize(
        &mut self,
        rpm: f32,
        load: f32,
        duration: f32,
    ) -> Result<Vec<f32>> {
        let rpm = rpm.clamp(0.0, 100000.0);
        let load = load.clamp(0.0, 1.0);
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];

        #[cfg(feature = "naad-backend")]
        self.synthesize_naad(&mut output, rpm, load);

        #[cfg(not(feature = "naad-backend"))]
        self.synthesize_fallback(&mut output, rpm, load);

        Ok(output)
    }

    #[cfg(feature = "naad-backend")]
    fn synthesize_naad(&mut self, output: &mut [f32], rpm: f32, load: f32) {
        let em_freq = (rpm / 60.0) * self.poles as f32;
        let em_omega = core::f32::consts::TAU * em_freq / self.sample_rate;

        let amp = 0.15 + load * 0.2;
        let noise_level = match self.motor_type {
            MotorType::DcBrushed => 0.15,
            MotorType::AcInduction => 0.05,
            MotorType::Brushless => 0.02,
            MotorType::Servo => 0.08,
        };

        for (i, sample) in output.iter_mut().enumerate() {
            let t = i as f32;
            // Electromagnetic hum: fundamental + harmonics via direct sin
            // (AdditiveSynth would need frequency updates per RPM change;
            //  direct computation is simpler for 3 harmonics)
            let fundamental = naad::dsp_util::soft_clip_tanh(
                libm::sinf(em_omega * t), 1.0,
            );
            let harmonic2 = libm::sinf(em_omega * 2.0 * t) * 0.3;
            let harmonic3 = libm::sinf(em_omega * 3.0 * t) * 0.1;
            let hum = (fundamental + harmonic2 + harmonic3) * amp;

            // Commutator/bearing noise — filtered
            let raw_noise = self.noise_gen.next_sample() * noise_level * amp;
            let noise = self.noise_filter.process_sample(raw_noise);

            *sample = hum + noise;
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn synthesize_fallback(&mut self, output: &mut [f32], rpm: f32, load: f32) {
        let em_freq = (rpm / 60.0) * self.poles as f32;
        let em_omega = core::f32::consts::TAU * em_freq / self.sample_rate;

        let amp = 0.15 + load * 0.2;
        let noise_level = match self.motor_type {
            MotorType::DcBrushed => 0.15,
            MotorType::AcInduction => 0.05,
            MotorType::Brushless => 0.02,
            MotorType::Servo => 0.08,
        };

        for (i, sample) in output.iter_mut().enumerate() {
            let fundamental = crate::math::f32::sin(em_omega * i as f32);
            let harmonic2 = crate::math::f32::sin(em_omega * 2.0 * i as f32) * 0.3;
            let harmonic3 = crate::math::f32::sin(em_omega * 3.0 * i as f32) * 0.1;
            let hum = (fundamental + harmonic2 + harmonic3) * amp;

            let noise = self.rng.next_f32() * noise_level * amp;

            *sample = hum + noise;
        }
    }
}
