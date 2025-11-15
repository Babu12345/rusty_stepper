//! Errors for this crate

/// Error type for the crate
#[derive(Debug)]
pub enum Error {
    /// Error when the phase configuration has an issue
    PhaseConfigurationError,
    /// Error when the duty confirguration has an issue
    DutyConfigurationError,
}

/// Result type
pub type Result<T> = core::result::Result<T, Error>;
