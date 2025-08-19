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
    #[error("Invalid response received")]
    InvalidResponse,
}

impl<I: embedded_io::Error> From<rmodbus::ErrorKind> for Error<I> {
    fn from(err: rmodbus::ErrorKind) -> Self {
        Error::ModbusError(err)
    }
}

