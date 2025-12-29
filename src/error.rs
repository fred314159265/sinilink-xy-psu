//! Our error types for the XY PSUs.

use thiserror::Error;

pub type Result<T, I> = core::result::Result<T, Error<I>>;

/// Custom error type for Sinilink XY PSU communications.
#[derive(Error, Debug)]
pub enum Error<I: embedded_io::Error> {
    #[error("Serial communication error")]
    SerialError(I),
    #[error("Modbus protocol error: {0}")]
    ModbusError(rmodbus::ErrorKind),
    #[error("Communication timeout")]
    Timeout,
    #[error("Invalid range")]
    InvalidRange,
    #[error("Invalid modbus response received")]
    InvalidResponse,
    #[error("heapless::Vec full?")]
    BufferError,
    #[error("Passed value was too large to convert to u16.")]
    IntTooBig,
    #[error(
        "Scaling factors not available for this PSU model. You can use the *_raw() methods instead an apply scaling manually."
    )]
    ScalingNotAvailable,
    #[error("Other, non-descriptive error...")]
    Other,
}

impl<I: embedded_io::Error> From<rmodbus::ErrorKind> for Error<I> {
    fn from(err: rmodbus::ErrorKind) -> Self {
        Error::ModbusError(err)
    }
}

impl<I: embedded_io::Error> From<core::num::TryFromIntError> for Error<I> {
    fn from(_value: core::num::TryFromIntError) -> Self {
        Error::IntTooBig
    }
}

impl<I: embedded_io::Error> From<()> for Error<I> {
    fn from(_: ()) -> Self {
        Error::Other
    }
}
