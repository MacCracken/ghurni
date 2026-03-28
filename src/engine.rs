//! Internal combustion engine sound synthesis.
//!
//! Models engines as periodic combustion impulses at RPM-dependent rates,
//! shaped by exhaust resonance and intake noise. The firing order and
//! cylinder count determine the harmonic signature.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::{GhurniError, Result};

/// Engine type — determines combustion character and exhaust signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EngineType {
    /// 4-stroke gasoline — higher RPM, smoother, mid-frequency exhaust.
    Gasoline,
    /// 4-stroke diesel — lower RPM, rougher combustion knock, deeper exhaust.
    Diesel,
    /// 2-stroke — fires every revolution, higher frequency, buzzy.
    TwoStroke,
    /// Electric motor with gear whine (hybrid context).
    Hybrid,
}

/// Internal combustion engine synthesizer.
///
/// The fundamental frequency is derived from RPM and cylinder count:
/// `firing_freq = (RPM / 60) * (cylinders / 2)` for 4-stroke engines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engine {
    engine_type: EngineType,
    cylinders: u32,
    exhaust_resonance: f32,
    sample_rate: f32,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    exhaust_filter: naad::filter::BiquadFilter,
    #[cfg(not(feature = "naad-backend"))]
    rng: crate::rng::Rng,
}

impl Engine {
    /// Creates a new engine synthesizer.
    ///
    /// - `cylinders`: Number of cylinders (1-16).
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn new(engine_type: EngineType, cylinders: u32, sample_rate: f32) -> Result<Self> {
        if sample_rate <= 0.0 {
            return Err(GhurniError::InvalidParameter(
                alloc::format!("sample_rate must be positive, got {sample_rate}"),
            ));
        }
        let cylinders = cylinders.clamp(1, 16);
        let exhaust_resonance = match engine_type {
            EngineType::Gasoline => 150.0 + cylinders as f32 * 20.0,
            EngineType::Diesel => 80.0 + cylinders as f32 * 15.0,
            EngineType::TwoStroke => 200.0 + cylinders as f32 * 30.0,
            EngineType::Hybrid => 300.0,
        };

        Ok(Self {
            engine_type,
            cylinders,
            exhaust_resonance,
            sample_rate,
            #[cfg(feature = "naad-backend")]
            noise_gen: naad::noise::NoiseGenerator::new(
                naad::noise::NoiseType::White,
                engine_type as u32 * 1000 + cylinders,
            ),
            #[cfg(feature = "naad-backend")]
            exhaust_filter: naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                exhaust_resonance,
                1.5,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(engine_type as u64 * 1000 + cylinders as u64),
        })
    }

    /// Returns the firing frequency at the given RPM.
    #[must_use]
    #[inline]
    pub fn firing_frequency(&self, rpm: f32) -> f32 {
        let revs_per_sec = rpm / 60.0;
        match self.engine_type {
            EngineType::Gasoline | EngineType::Diesel | EngineType::Hybrid => {
                revs_per_sec * self.cylinders as f32 / 2.0
            }
            EngineType::TwoStroke => revs_per_sec * self.cylinders as f32,
        }
    }

    /// Synthesizes engine sound at the given RPM and load.
    ///
    /// - `rpm`: Engine speed (100-15000).
    /// - `load`: Throttle/load factor (0.0 = idle, 1.0 = full load).
    /// - `duration`: Duration in seconds.
    #[inline]
    pub fn synthesize(
        &mut self,
        rpm: f32,
        load: f32,
        duration: f32,
    ) -> Result<Vec<f32>> {
        let rpm = rpm.clamp(100.0, 15000.0);
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
        let firing_freq = self.firing_frequency(rpm);
        let firing_period = self.sample_rate / firing_freq;
        let base_amp = 0.2 + load * 0.5;

        let roughness = match self.engine_type {
            EngineType::Diesel => 0.4,
            EngineType::TwoStroke => 0.3,
            EngineType::Gasoline => 0.15,
            EngineType::Hybrid => 0.05,
        };

        // Update exhaust filter frequency to match current resonance
        let _ = self.exhaust_filter.set_params(self.exhaust_resonance, 1.5, 0.0);

        for (i, sample) in output.iter_mut().enumerate() {
            let phase = (i as f32 % firing_period) / firing_period;

            // Combustion pulse: sharp impulse at each firing event
            let combustion = if phase < 0.1 {
                let t = phase / 0.1;
                let pulse = naad::dsp_util::db_to_amplitude(-8.0 * t * 20.0 / core::f32::consts::LOG10_E);
                pulse * base_amp * (1.0 + roughness * self.noise_gen.next_sample())
            } else {
                0.0
            };

            // Exhaust resonance: filter noise at exhaust frequency
            let exhaust_noise = self.noise_gen.next_sample() * base_amp * 0.3;
            let exhaust_decay = naad::dsp_util::db_to_amplitude(-3.0 * phase * 20.0 / core::f32::consts::LOG10_E);
            let exhaust = self.exhaust_filter.process_sample(exhaust_noise) * exhaust_decay;

            // Mechanical noise
            let mech_noise = self.noise_gen.next_sample() * roughness * base_amp * 0.1;

            *sample = combustion + exhaust + mech_noise;
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn synthesize_fallback(&mut self, output: &mut [f32], rpm: f32, load: f32) {
        let firing_freq = self.firing_frequency(rpm);
        let firing_period = self.sample_rate / firing_freq;
        let base_amp = 0.2 + load * 0.5;

        let roughness = match self.engine_type {
            EngineType::Diesel => 0.4,
            EngineType::TwoStroke => 0.3,
            EngineType::Gasoline => 0.15,
            EngineType::Hybrid => 0.05,
        };

        let exhaust_omega = core::f32::consts::TAU * self.exhaust_resonance / self.sample_rate;

        for (i, sample) in output.iter_mut().enumerate() {
            let phase = (i as f32 % firing_period) / firing_period;

            let combustion = if phase < 0.1 {
                let t = phase / 0.1;
                let pulse = crate::math::f32::exp(-8.0 * t);
                pulse * base_amp * (1.0 + roughness * self.rng.next_f32())
            } else {
                0.0
            };

            let exhaust = crate::math::f32::sin(exhaust_omega * i as f32)
                * base_amp
                * 0.3
                * crate::math::f32::exp(-3.0 * phase);

            let mech_noise = self.rng.next_f32() * roughness * base_amp * 0.1;

            *sample = combustion + exhaust + mech_noise;
        }
    }
}
