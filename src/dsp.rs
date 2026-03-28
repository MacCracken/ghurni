//! DSP utilities: DC blocker, validation helpers.

use serde::{Deserialize, Serialize};

use crate::error::{GhurniError, Result};

/// One-pole high-pass DC blocker.
///
/// Transfer function: `y[n] = x[n] - x[n-1] + R * y[n-1]`
/// Removes DC offset with -3dB cutoff at ~10 Hz.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DcBlocker {
    x_prev: f32,
    y_prev: f32,
    r: f32,
}

impl DcBlocker {
    /// Creates a DC blocker for the given sample rate.
    #[inline]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            x_prev: 0.0,
            y_prev: 0.0,
            r: (1.0 - (core::f32::consts::TAU * 10.0 / sample_rate)).clamp(0.9, 0.9999),
        }
    }

    /// Process a single sample, removing DC offset.
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = x - self.x_prev + self.r * self.y_prev;
        self.x_prev = x;
        self.y_prev = y;
        y
    }
}

/// Validates that sample_rate is positive.
#[inline]
pub(crate) fn validate_sample_rate(sample_rate: f32) -> Result<()> {
    if sample_rate <= 0.0 || sample_rate.is_nan() || sample_rate.is_infinite() {
        #[cfg(feature = "logging")]
        tracing::warn!(%sample_rate, "invalid sample rate");
        return Err(GhurniError::InvalidParameter(
            alloc::format!("sample_rate must be positive and finite, got {sample_rate}"),
        ));
    }
    Ok(())
}

/// Validates that duration is positive.
#[inline]
pub(crate) fn validate_duration(duration: f32) -> Result<()> {
    if duration <= 0.0 || duration.is_nan() || duration.is_infinite() {
        #[cfg(feature = "logging")]
        tracing::warn!(%duration, "invalid duration");
        return Err(GhurniError::InvalidParameter(
            alloc::format!("duration must be positive and finite, got {duration}"),
        ));
    }
    Ok(())
}

/// Converts a naad error into a GhurniError::SynthesisFailed with logging.
#[cfg(feature = "naad-backend")]
#[allow(unused_variables, dead_code)]
pub(crate) fn naad_init_error(
    synth_name: &str,
    component: &str,
    err: impl core::fmt::Display,
) -> GhurniError {
    #[cfg(feature = "logging")]
    tracing::error!(synth = synth_name, component, %err, "naad backend error");
    GhurniError::SynthesisFailed(alloc::format!("{err}"))
}
