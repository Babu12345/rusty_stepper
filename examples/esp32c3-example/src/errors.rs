//! Errors for the esp32c3-examples

use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum Error {
    CustomError(&'static str),
    ChannelConfigurationError,
    InternalStepperError(#[from] stepper::errors::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
