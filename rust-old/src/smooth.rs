//! Parameter smoothing for click-free transitions.

use serde::{Deserialize, Serialize};

/// One-pole exponential smoother for real-time parameter changes.
///
/// Prevents clicks/artifacts when RPM, load, or other parameters
/// change mid-block by exponentially approaching the target value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmoothedParam {
    current: f32,
    target: f32,
    coeff: f32,
}

impl SmoothedParam {
    /// Creates a new smoother.
    ///
    /// - `initial`: Starting value.
    /// - `smooth_time_s`: Time constant in seconds (63% of the way to target).
    /// - `sample_rate`: Audio sample rate.
    #[inline]
    pub fn new(initial: f32, smooth_time_s: f32, sample_rate: f32) -> Self {
        let coeff = if smooth_time_s > 0.0 && sample_rate > 0.0 {
            (-1.0 / (smooth_time_s * sample_rate)).exp()
        } else {
            0.0
        };
        Self {
            current: initial,
            target: initial,
            coeff,
        }
    }

    /// Sets the target value (approached smoothly).
    #[inline]
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Advances one sample and returns the smoothed value.
    #[inline]
    pub fn next_value(&mut self) -> f32 {
        self.current = self.target + self.coeff * (self.current - self.target);
        self.current
    }

    /// Snaps immediately to the target (no smoothing).
    #[inline]
    pub fn snap(&mut self) {
        self.current = self.target;
    }

    /// Returns the current smoothed value without advancing.
    #[must_use]
    #[inline]
    pub fn current(&self) -> f32 {
        self.current
    }

    /// Returns the target value.
    #[must_use]
    #[inline]
    pub fn target(&self) -> f32 {
        self.target
    }

    /// Returns true if the value has settled (within epsilon of target).
    #[must_use]
    #[inline]
    pub fn is_settled(&self) -> bool {
        (self.current - self.target).abs() < 1e-6
    }
}

#[cfg(feature = "std")]
impl SmoothedParam {
    fn _exp(x: f32) -> f32 {
        x.exp()
    }
}
