//! Error types for the ghurni crate.

use alloc::string::String;
use serde::{Deserialize, Serialize};

/// Errors that can occur during mechanical sound synthesis.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[non_exhaustive]
pub enum GhurniError {
    /// A parameter is out of valid range.
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    /// A synthesis operation failed.
    #[error("synthesis failed: {0}")]
    SynthesisFailed(String),

    /// A computation produced an invalid result.
    #[error("computation error: {0}")]
    ComputationError(String),
}

/// Convenience type alias for ghurni results.
pub type Result<T> = core::result::Result<T, GhurniError>;
