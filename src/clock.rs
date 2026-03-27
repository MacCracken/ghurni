//! Clock and precision mechanism sound synthesis.
//!
//! Models the tick-tock of escapements, spring resonance, and the
//! delicate mechanical sounds of horology.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::rng::Rng;

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
    /// Clock type.
    clock_type: ClockType,
    /// Ticks per second (escapement rate).
    tick_rate: f32,
    /// PRNG.
    rng: Rng,
}

impl Clock {
    /// Creates a new clock synthesizer.
    #[must_use]
    pub fn new(clock_type: ClockType) -> Self {
        let tick_rate = match clock_type {
            ClockType::Wristwatch => 8.0,       // 4 Hz escapement, 8 beats/s
            ClockType::WallClock => 2.0,        // 1 Hz pendulum, 2 beats/s
            ClockType::GrandfatherClock => 1.0, // 0.5 Hz pendulum
            ClockType::PocketWatch => 5.0,      // 2.5 Hz escapement
        };
        Self {
            clock_type,
            tick_rate,
            rng: Rng::new(clock_type as u64 * 997),
        }
    }

    /// Synthesizes clock ticking sound.
    #[inline]
    pub fn synthesize(&mut self, sample_rate: f32, duration: f32) -> Result<Vec<f32>> {
        let num_samples = (sample_rate * duration) as usize;
        let tick_period = sample_rate / self.tick_rate;

        let (resonance, decay, amp) = match self.clock_type {
            ClockType::Wristwatch => (6000.0, 0.003, 0.15),
            ClockType::WallClock => (2000.0, 0.01, 0.4),
            ClockType::GrandfatherClock => (800.0, 0.03, 0.6),
            ClockType::PocketWatch => (4500.0, 0.005, 0.25),
        };

        let res_omega = core::f32::consts::TAU * resonance / sample_rate;
        let mut output = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let phase = (i as f32 % tick_period) / tick_period;

            // Tick impulse: sharp transient + resonant decay
            let tick = if phase < 0.02 {
                // Sharp transient
                let t = phase / 0.02;
                let impulse = (1.0 - t) * amp;
                let ring = crate::math::f32::sin(res_omega * i as f32)
                    * crate::math::f32::exp(-phase / decay)
                    * amp;
                impulse * 0.5 + ring
            } else if phase < 0.15 {
                // Resonant tail
                crate::math::f32::sin(res_omega * i as f32)
                    * crate::math::f32::exp(-phase / decay)
                    * amp
                    * 0.3
            } else {
                0.0
            };

            // Subtle mechanical noise between ticks
            let mech = self.rng.next_f32() * amp * 0.01;

            output.push(tick + mech);
        }

        Ok(output)
    }
}
