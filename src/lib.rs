//! This crate provides an interface for communicating and controlling the Sinilink XY series of programmable power supplies.
//!
//! It supports `no_std` environments by use of the `no_std` feature flag.
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

#![cfg_attr(feature = "no_std", no_std)]

pub mod error;
pub mod preset;
pub mod psu;
pub mod register;

#[cfg(test)]
mod mock_serial;

// General @TODO:
// * Determine units of all values and protections, based on setting and reading over modbus.
//      * Update protection defaults to reflect this.
// * Do we need a lookup table to establish bounds checking on values set?
// * Add provisions for setting protections values using presets.
//     * Because importing profile will set all protection, this will need to either:
//         1. read active profile, modify one protection (and check all iset, vset match existing) and then apply.
//         2. User has to supply all protection values, and ones not provided are set to maximums, and then preset applied. (After making sure Vset, etc matches)
//     * Will loading presets enable/disable the output?
// Unify use of get/read/set/write

// How to exit once protection is activated?

// I suggest use of presets behind some kind of "set protections" method and a struct for configuring all protections.
// * General support for presets.
// * Make use of https://github.com/alttch/rmodbus?tab=readme-ov-file#custom-type-representations-in-u16-sized-registers ?
// * Expose all functions/registers
// Add conditional methods using float behind f32 feature flag

// Structure:
// - Transport (serial)
// - Modbus
// - Reading/setting registers
// - Abstract PSU control
