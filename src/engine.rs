//! Internal combustion engine sound synthesis.
//!
//! Models engines as periodic combustion impulses at RPM-dependent rates,
//! shaped by exhaust resonance and intake noise. The firing order and
//! cylinder count determine the harmonic signature.
//!
//! ## Features
//!
//! - Multi-cylinder firing order (V8 burble vs inline-4 drone)
//! - Exhaust resonance with material-specific filtering
//! - Intake manifold Helmholtz resonance
//! - Deceleration crackle/pop
//! - Load-dependent timbre (roughness, harmonic content)
//! - Misfire and knock event triggers

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dsp::{DcBlocker, validate_duration, validate_sample_rate};
use crate::event::MechanicalEvent;
#[cfg(feature = "naad-backend")]
use crate::error::GhurniError;
use crate::error::Result;
use crate::smooth::SmoothedParam;
use crate::traits::Synthesizer;

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
    intake_resonance: f32,
    sample_rate: f32,
    rpm: f32,
    load: f32,
    prev_load: f32,
    sample_position: usize,
    /// Firing order — crank angle offsets (degrees) for each cylinder.
    firing_offsets: Vec<f32>,
    /// Per-cylinder misfire flags (cleared after one firing event).
    misfire_flags: Vec<bool>,
    /// Per-cylinder knock remaining samples.
    knock_remaining: Vec<usize>,
    /// Decel pop remaining samples.
    decel_pop_remaining: usize,
    /// Backfire remaining samples.
    backfire_remaining: usize,
    smooth_rpm: SmoothedParam,
    smooth_load: SmoothedParam,
    dc_blocker: DcBlocker,
    #[cfg(feature = "naad-backend")]
    noise_gen: naad::noise::NoiseGenerator,
    #[cfg(feature = "naad-backend")]
    exhaust_filter: naad::filter::BiquadFilter,
    #[cfg(feature = "naad-backend")]
    intake_filter: naad::filter::BiquadFilter,
    #[cfg(not(feature = "naad-backend"))]
    rng: crate::rng::Rng,
}

impl Engine {
    /// Creates a new engine synthesizer.
    ///
    /// - `engine_type`: Type of engine.
    /// - `cylinders`: Number of cylinders (1-16).
    /// - `sample_rate`: Audio sample rate in Hz.
    pub fn new(engine_type: EngineType, cylinders: u32, sample_rate: f32) -> Result<Self> {
        validate_sample_rate(sample_rate)?;
        let cylinders = cylinders.clamp(1, 16);
        let exhaust_resonance = match engine_type {
            EngineType::Gasoline => 150.0 + cylinders as f32 * 20.0,
            EngineType::Diesel => 80.0 + cylinders as f32 * 15.0,
            EngineType::TwoStroke => 200.0 + cylinders as f32 * 30.0,
            EngineType::Hybrid => 300.0,
        };

        // Intake resonance — higher than exhaust, Helmholtz-like
        let intake_resonance = match engine_type {
            EngineType::Gasoline => 400.0 + cylinders as f32 * 30.0,
            EngineType::Diesel => 250.0 + cylinders as f32 * 20.0,
            EngineType::TwoStroke => 500.0 + cylinders as f32 * 40.0,
            EngineType::Hybrid => 600.0,
        };

        // Default even firing order
        let firing_offsets = Self::compute_even_firing(engine_type, cylinders);

        Ok(Self {
            engine_type,
            cylinders,
            exhaust_resonance,
            intake_resonance,
            sample_rate,
            rpm: 800.0,
            load: 0.0,
            prev_load: 0.0,
            sample_position: 0,
            firing_offsets,
            misfire_flags: alloc::vec![false; cylinders as usize],
            knock_remaining: alloc::vec![0; cylinders as usize],
            decel_pop_remaining: 0,
            backfire_remaining: 0,
            smooth_rpm: SmoothedParam::new(800.0, 0.05, sample_rate),
            smooth_load: SmoothedParam::new(0.0, 0.02, sample_rate),
            dc_blocker: DcBlocker::new(sample_rate),
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
            #[cfg(feature = "naad-backend")]
            intake_filter: naad::filter::BiquadFilter::new(
                naad::filter::FilterType::BandPass,
                sample_rate,
                intake_resonance.min(sample_rate * 0.49),
                2.0,
            )
            .map_err(|e| GhurniError::SynthesisFailed(alloc::format!("{e}")))?,
            #[cfg(not(feature = "naad-backend"))]
            rng: crate::rng::Rng::new(engine_type as u64 * 1000 + cylinders as u64),
        })
    }

    /// Computes even firing offsets in degrees for the given cylinder count.
    fn compute_even_firing(engine_type: EngineType, cylinders: u32) -> Vec<f32> {
        let cycle_degrees = match engine_type {
            EngineType::TwoStroke => 360.0,
            _ => 720.0, // 4-stroke
        };
        let spacing = cycle_degrees / cylinders as f32;
        (0..cylinders).map(|i| i as f32 * spacing).collect()
    }

    /// Sets a custom firing order (crank angle offsets in degrees).
    ///
    /// For a cross-plane V8: `[0, 90, 270, 180, 540, 630, 450, 360]`
    pub fn set_firing_order(&mut self, offsets: Vec<f32>) {
        if offsets.len() == self.cylinders as usize {
            self.firing_offsets = offsets;
        }
    }

    /// Sets the engine RPM (clamped to 100-15000).
    pub fn set_rpm(&mut self, rpm: f32) {
        self.rpm = rpm.clamp(100.0, 15000.0);
        self.smooth_rpm.set_target(self.rpm);
    }

    /// Sets the engine load (clamped to 0.0-1.0).
    pub fn set_load(&mut self, load: f32) {
        self.prev_load = self.load;
        self.load = load.clamp(0.0, 1.0);
        self.smooth_load.set_target(self.load);

        // Decel pop detection: load drops sharply while RPM is high
        if self.prev_load > 0.5 && self.load < 0.15 && self.rpm > 3000.0 {
            self.decel_pop_remaining = (self.sample_rate * 0.3) as usize;
        }
    }

    /// Triggers a mechanical event on this engine.
    pub fn trigger_event(&mut self, event: MechanicalEvent) {
        match event {
            MechanicalEvent::Backfire => {
                self.backfire_remaining = (self.sample_rate * 0.1) as usize;
            }
            MechanicalEvent::Misfire { cylinder } => {
                if let Some(flag) = self.misfire_flags.get_mut(cylinder as usize) {
                    *flag = true;
                }
            }
            MechanicalEvent::Knock { cylinder } => {
                if let Some(remaining) = self.knock_remaining.get_mut(cylinder as usize) {
                    *remaining = (self.sample_rate * 0.02) as usize;
                }
            }
            _ => {}
        }
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

    /// Synthesizes engine sound at the given RPM and load (one-shot).
    pub fn synthesize(
        &mut self,
        rpm: f32,
        load: f32,
        duration: f32,
    ) -> Result<Vec<f32>> {
        validate_duration(duration)?;
        self.set_rpm(rpm);
        self.set_load(load);
        let num_samples = (self.sample_rate * duration) as usize;
        let mut output = alloc::vec![0.0f32; num_samples];
        self.process_block(&mut output);
        Ok(output)
    }

    /// Fills `output` with engine sound using current RPM and load.
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
        let _ = self.exhaust_filter.set_params(self.exhaust_resonance, 1.5, 0.0);

        let cycle_degrees: f32 = match self.engine_type {
            EngineType::TwoStroke => 360.0,
            _ => 720.0,
        };

        for (i, sample) in output.iter_mut().enumerate() {
            let rpm = self.smooth_rpm.next_value();
            let load = self.smooth_load.next_value();
            let abs_pos = (self.sample_position + i) as f32;

            // Load-dependent timbre
            let roughness = self.roughness_for_load(load);
            let base_amp = 0.2 + load * 0.5;

            // Crank angle in degrees at this sample
            let revs_per_sample = rpm / (60.0 * self.sample_rate);
            let crank_deg = (abs_pos * revs_per_sample * 360.0) % cycle_degrees;

            // Sum combustion impulses from each cylinder
            let mut combustion_sum = 0.0f32;
            for (cyl, &offset) in self.firing_offsets.iter().enumerate() {
                let cyl_phase = ((crank_deg - offset) % cycle_degrees + cycle_degrees) % cycle_degrees;
                let cyl_norm = cyl_phase / cycle_degrees;

                // Check for misfire
                if self.misfire_flags.get(cyl).copied().unwrap_or(false) && cyl_norm < 0.01 {
                    if let Some(flag) = self.misfire_flags.get_mut(cyl) {
                        *flag = false;
                    }
                    continue; // Skip this firing
                }

                // Combustion pulse window
                if cyl_norm < 0.08 {
                    let t = cyl_norm / 0.08;
                    let pulse = naad::dsp_util::db_to_amplitude(
                        -8.0 * t * 20.0 / core::f32::consts::LOG10_E,
                    );
                    combustion_sum +=
                        pulse * base_amp * (1.0 + roughness * self.noise_gen.next_sample());
                }
            }

            // Knock: high-frequency metallic ping
            let mut knock_sum = 0.0f32;
            for remaining in self.knock_remaining.iter_mut() {
                if *remaining > 0 {
                    *remaining -= 1;
                    let env = *remaining as f32 / (self.sample_rate * 0.02);
                    knock_sum += self.noise_gen.next_sample() * env * 0.2;
                }
            }

            // Exhaust resonance
            let exhaust_noise = self.noise_gen.next_sample() * base_amp * 0.3;
            let exhaust = self.exhaust_filter.process_sample(exhaust_noise) * (0.5 + load * 0.5);

            // Intake resonance (louder at higher load/RPM)
            let intake_noise = self.noise_gen.next_sample() * base_amp * 0.2 * load;
            let intake = self.intake_filter.process_sample(intake_noise);

            // Decel crackle/pop
            let decel = if self.decel_pop_remaining > 0 {
                self.decel_pop_remaining -= 1;
                let env = self.decel_pop_remaining as f32 / (self.sample_rate * 0.3);
                // Random pops — sparse impulses
                if self.noise_gen.next_sample() > 0.95 {
                    self.noise_gen.next_sample() * env * 0.4
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Backfire
            let backfire = if self.backfire_remaining > 0 {
                self.backfire_remaining -= 1;
                let env = self.backfire_remaining as f32 / (self.sample_rate * 0.1);
                self.noise_gen.next_sample() * env * 0.8
            } else {
                0.0
            };

            // Mechanical noise
            let mech_noise = self.noise_gen.next_sample() * roughness * base_amp * 0.1;

            *sample = combustion_sum + exhaust + intake + mech_noise + knock_sum + decel + backfire;
        }
    }

    #[cfg(not(feature = "naad-backend"))]
    fn process_block_fallback(&mut self, output: &mut [f32]) {
        let cycle_degrees: f32 = match self.engine_type {
            EngineType::TwoStroke => 360.0,
            _ => 720.0,
        };

        let exhaust_omega = core::f32::consts::TAU * self.exhaust_resonance / self.sample_rate;

        for (i, sample) in output.iter_mut().enumerate() {
            let rpm = self.smooth_rpm.next_value();
            let load = self.smooth_load.next_value();
            let abs_pos = (self.sample_position + i) as f32;

            let roughness = self.roughness_for_load(load);
            let base_amp = 0.2 + load * 0.5;

            let revs_per_sample = rpm / (60.0 * self.sample_rate);
            let crank_deg = (abs_pos * revs_per_sample * 360.0) % cycle_degrees;

            let mut combustion_sum = 0.0f32;
            for (cyl, &offset) in self.firing_offsets.iter().enumerate() {
                let cyl_phase = ((crank_deg - offset) % cycle_degrees + cycle_degrees) % cycle_degrees;
                let cyl_norm = cyl_phase / cycle_degrees;

                if self.misfire_flags.get(cyl).copied().unwrap_or(false) && cyl_norm < 0.01 {
                    if let Some(flag) = self.misfire_flags.get_mut(cyl) {
                        *flag = false;
                    }
                    continue;
                }

                if cyl_norm < 0.08 {
                    let t = cyl_norm / 0.08;
                    let pulse = crate::math::f32::exp(-8.0 * t);
                    combustion_sum +=
                        pulse * base_amp * (1.0 + roughness * self.rng.next_f32());
                }
            }

            let exhaust = crate::math::f32::sin(exhaust_omega * abs_pos)
                * base_amp
                * 0.3
                * (0.5 + load * 0.5);

            let decel = if self.decel_pop_remaining > 0 {
                self.decel_pop_remaining -= 1;
                let env = self.decel_pop_remaining as f32 / (self.sample_rate * 0.3);
                if self.rng.next_f32() > 0.9 {
                    self.rng.next_f32() * env * 0.4
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let backfire = if self.backfire_remaining > 0 {
                self.backfire_remaining -= 1;
                let env = self.backfire_remaining as f32 / (self.sample_rate * 0.1);
                self.rng.next_f32() * env * 0.8
            } else {
                0.0
            };

            let mech_noise = self.rng.next_f32() * roughness * base_amp * 0.1;

            *sample = combustion_sum + exhaust + mech_noise + decel + backfire;
        }
    }

    /// Returns roughness based on engine type and current load.
    #[inline]
    fn roughness_for_load(&self, load: f32) -> f32 {
        let base = match self.engine_type {
            EngineType::Diesel => 0.4,
            EngineType::TwoStroke => 0.3,
            EngineType::Gasoline => 0.15,
            EngineType::Hybrid => 0.05,
        };
        // Higher load = more roughness
        base * (0.7 + 0.3 * load)
    }
}

impl Synthesizer for Engine {
    fn process_block(&mut self, output: &mut [f32]) {
        Engine::process_block(self, output);
    }

    fn set_rpm(&mut self, rpm: f32) {
        Engine::set_rpm(self, rpm);
    }

    fn rpm(&self) -> f32 {
        self.rpm
    }

    fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}
