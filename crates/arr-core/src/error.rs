//! Typed arr-core errors. Every fallible function returns `Result<T, ArrError>`.

use std::fmt;

/// Domain error taxonomy (mirrors the fleet kernel: InvalidInput, Offline,
/// Timeout, Auth, Protocol, Unsupported).
#[derive(Debug)]
pub enum ArrError {
    InvalidInput(String),
    Offline(String),
    Timeout(String),
    Auth(String),
    Protocol(String),
    Unsupported(String),
}

impl fmt::Display for ArrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(m)
            | Self::Offline(m)
            | Self::Timeout(m)
            | Self::Auth(m)
            | Self::Protocol(m)
            | Self::Unsupported(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for ArrError {}
