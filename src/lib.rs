//! This crate provides an interface for communicating and controlling the Sinilink XY series of programmable power supplies.
//!
//! It supports `no-std` environments by use of the `no-std` feature flag.
//!
//! @TODO add table including electrical spec.
//!
//! Example PSU model numbers which this should work with:
//! * XY6506
//! * XY6509
//! * XY-6506S
//! * XY7025
//! * XY6509X
//! * XY12522
//! * XY3607F
//!
//! PSU models which it may work with:
//! * XY3606B
//! * XY-SK60S
//! * XY-SK120S
//! * XY-SK150S
//!
//! It uses Modbus RTU under the hood, and is suitable for interfacing with the XY PSUs over serial/UART or RS485, but not Wi-Fi.
//!
//! The serial port used for PSU comms should be configured like so:
//! * Default baud rate: 115200
//! * Data bits: 8
//! * Stop bits: 1
//! * Parity: None

#![cfg_attr(feature = "no-std", no_std)]
// #![cfg_attr(not(feature = "std"), no_std)]

pub mod error;
pub mod psu;
mod registers;
mod types;

#[cfg(test)]
mod mock_serial;
