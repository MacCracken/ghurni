//! Internal combustion engine sound synthesis.
//!
//! Models engines as periodic combustion impulses at RPM-dependent rates,
//! shaped by exhaust resonance and intake noise. The firing order and
//! cylinder count determine the harmonic signature.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::rng::Rng;

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
    /// Engine type.
    engine_type: EngineType,
    /// Number of cylinders.
    cylinders: u32,
    /// Exhaust resonant frequency (Hz) — determined by pipe length.
    exhaust_resonance: f32,
    /// PRNG for combustion noise.
    rng: Rng,
}

impl Engine {
    /// Creates a new engine synthesizer.
    ///
    /// `cylinders` is the number of cylinders (1-16).
    #[must_use]
    pub fn new(engine_type: EngineType, cylinders: u32) -> Self {
        let cylinders = cylinders.clamp(1, 16);
        let exhaust_resonance = match engine_type {
            EngineType::Gasoline => 150.0 + cylinders as f32 * 20.0,
            EngineType::Diesel => 80.0 + cylinders as f32 * 15.0,
            EngineType::TwoStroke => 200.0 + cylinders as f32 * 30.0,
            EngineType::Hybrid => 300.0,
        };
        Self {
            engine_type,
            cylinders,
            exhaust_resonance,
            rng: Rng::new(engine_type as u64 * 1000 + cylinders as u64),
        }
    }

    /// Returns the firing frequency at the given RPM.
    #[must_use]
    #[inline]
    pub fn firing_frequency(&self, rpm: f32) -> f32 {
        let revs_per_sec = rpm / 60.0;
        match self.engine_type {
            // 4-stroke: fires every 2 revolutions per cylinder
            EngineType::Gasoline | EngineType::Diesel | EngineType::Hybrid => {
                revs_per_sec * self.cylinders as f32 / 2.0
            }
            // 2-stroke: fires every revolution per cylinder
            EngineType::TwoStroke => revs_per_sec * self.cylinders as f32,
        }
    }

    /// Synthesizes engine sound at the given RPM and load.
    ///
    /// - `rpm`: Engine speed (100-15000).
    /// - `load`: Throttle/load factor (0.0 = idle, 1.0 = full load).
    /// - `sample_rate`: Audio sample rate in Hz.
    /// - `duration`: Duration in seconds.
    #[inline]
    pub fn synthesize(
        &mut self,
        rpm: f32,
        load: f32,
        sample_rate: f32,
        duration: f32,
    ) -> Result<Vec<f32>> {
        let rpm = rpm.clamp(100.0, 15000.0);
        let load = load.clamp(0.0, 1.0);
        let num_samples = (sample_rate * duration) as usize;
        let firing_freq = self.firing_frequency(rpm);
        let firing_period = sample_rate / firing_freq;

        let mut output = Vec::with_capacity(num_samples);

        // Amplitude scales with RPM and load
        let base_amp = 0.2 + load * 0.5;

        // Roughness: diesel is rougher, higher RPM is smoother
        let roughness = match self.engine_type {
            EngineType::Diesel => 0.4,
            EngineType::TwoStroke => 0.3,
            EngineType::Gasoline => 0.15,
            EngineType::Hybrid => 0.05,
        };

        let exhaust_omega = core::f32::consts::TAU * self.exhaust_resonance / sample_rate;

        for i in 0..num_samples {
            let phase = (i as f32 % firing_period) / firing_period;

            // Combustion pulse: sharp impulse at each firing event
            let combustion = if phase < 0.1 {
                let t = phase / 0.1;
                let pulse = crate::math::f32::exp(-8.0 * t);
                pulse * base_amp * (1.0 + roughness * self.rng.next_f32())
            } else {
                0.0
            };

            // Exhaust resonance: decaying tone at exhaust frequency
            let exhaust = crate::math::f32::sin(exhaust_omega * i as f32)
                * base_amp
                * 0.3
                * crate::math::f32::exp(-3.0 * phase);

            // Mechanical noise: broadband, scales with RPM
            let mech_noise = self.rng.next_f32() * roughness * base_amp * 0.1;

            output.push(combustion + exhaust + mech_noise);
        }

        Ok(output)
    }
}
