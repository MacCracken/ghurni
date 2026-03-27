//! Gear mesh sound synthesis.
//!
//! Models the sound of meshing gear teeth: tooth mesh frequency (teeth x RPM),
//! metallic resonance, and load-dependent noise.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::rng::Rng;

/// Gear body material — affects resonance and decay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GearMaterial {
    /// Hardened steel — bright, long ring.
    Steel,
    /// Cast iron — duller, shorter decay.
    CastIron,
    /// Brass — warm, medium ring.
    Brass,
    /// Nylon/plastic — very dull, short decay.
    Nylon,
}

impl GearMaterial {
    /// Returns (resonance_hz, decay_s, brightness).
    #[must_use]
    fn properties(self) -> (f32, f32, f32) {
        match self {
            Self::Steel => (3500.0, 0.08, 0.9),
            Self::CastIron => (2000.0, 0.04, 0.5),
            Self::Brass => (2800.0, 0.06, 0.7),
            Self::Nylon => (1000.0, 0.01, 0.2),
        }
    }
}

/// Gear mesh synthesizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gear {
    /// Number of teeth on the gear.
    teeth: u32,
    /// Material.
    material: GearMaterial,
    /// Material properties (cached).
    resonance: f32,
    decay: f32,
    brightness: f32,
    /// PRNG.
    rng: Rng,
}

impl Gear {
    /// Creates a new gear synthesizer.
    #[must_use]
    pub fn new(teeth: u32, material: GearMaterial) -> Self {
        let (resonance, decay, brightness) = material.properties();
        Self {
            teeth: teeth.max(4),
            material,
            resonance,
            decay,
            brightness,
            rng: Rng::new(teeth as u64 * 7 + material as u64),
        }
    }

    /// Returns the tooth mesh frequency at the given RPM.
    #[must_use]
    #[inline]
    pub fn mesh_frequency(&self, rpm: f32) -> f32 {
        (rpm / 60.0) * self.teeth as f32
    }

    /// Synthesizes gear mesh sound.
    ///
    /// - `rpm`: Shaft speed.
    /// - `sample_rate`: Audio sample rate.
    /// - `duration`: Duration in seconds.
    #[inline]
    pub fn synthesize(&mut self, rpm: f32, sample_rate: f32, duration: f32) -> Result<Vec<f32>> {
        let rpm = rpm.clamp(1.0, 50000.0);
        let num_samples = (sample_rate * duration) as usize;
        let mesh_freq = self.mesh_frequency(rpm);
        let mesh_omega = core::f32::consts::TAU * mesh_freq / sample_rate;
        let res_omega = core::f32::consts::TAU * self.resonance / sample_rate;

        let mut output = Vec::with_capacity(num_samples);
        let amp = 0.3;

        for i in 0..num_samples {
            // Tooth mesh tone
            let mesh = crate::math::f32::sin(mesh_omega * i as f32) * amp * 0.5;

            // Resonant ringing excited by mesh impacts
            let mesh_phase = (i as f32 * mesh_freq / sample_rate) % 1.0;
            let ring_env = if mesh_phase < 0.05 {
                1.0
            } else {
                crate::math::f32::exp(-mesh_phase / self.decay)
            };
            let ring =
                crate::math::f32::sin(res_omega * i as f32) * ring_env * amp * self.brightness;

            // Mechanical noise
            let noise = self.rng.next_f32() * amp * 0.05;

            output.push(mesh + ring + noise);
        }

        Ok(output)
    }
}
