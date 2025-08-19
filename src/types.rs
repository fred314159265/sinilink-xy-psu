//! This module contains types relevant to the PSU Modbus data types.

/// This enum represents all possible product model versions.
#[derive(Debug)]
#[repr(u16)]
pub enum ProductModel {
    /// This model's "MODEL" register value has not been confirmed.
    XYSK60S,
    /// This model's "MODEL" register value has not been confirmed.
    XYSK120S,
    /// This model's "MODEL" register value has not been confirmed.
    XYSK150S,
    /// This model's "MODEL" register value has not been confirmed.
    XY3606B,
    /// This model's "MODEL" register value has not been confirmed.
    XY3607F = 3607,
    /// This model's "MODEL" register value has not been confirmed.
    XY6506,
    /// This model's "MODEL" register value has not been confirmed.
    XY6506S,
    /// This model's "MODEL" register value has not been confirmed.
    XY6509,
    /// This model's "MODEL" register value has not been confirmed.
    XY6509X,
    /// This model's "MODEL" register value has not been confirmed.
    XY7025 = 7025,
    /// This model's "MODEL" register value has not been confirmed.
    XY12522 = 12522,
}

/// Represents the two possible power supply control modes.
#[derive(Debug)]
pub enum ControlMode {
    /// Constant voltage regulation mode.
    Cv,
    /// Constant current regulation mode.
    Cc,
}

/// All possible baud rates supported by the XY PSUs.
#[derive(Debug)]
pub enum BaudRate {
    _9600 = 0,
    _14400 = 1,
    _19200 = 2,
    _38400 = 3,
    _5600 = 4,
    _576000 = 5,
    /// This is the default PSU baud rate.
    _115200 = 6,
    /// __Note:__ This baud rate is only supported by some of the PSU models.
    _2400 = 7,
    /// __Note:__ This baud rate is only supported by some of the PSU models.
    _4800 = 8,
}

/// Used to be less ambiguous and whether something is on or off.
#[derive(Debug)]
pub enum State {
    /// Disabled.
    Off,
    /// Enabled.
    On,
}

impl From<State> for bool {
    fn from(value: State) -> Self {
        match value {
            State::Off => false,
            State::On => true,
        }
    }
}

impl From<bool> for State {
    fn from(value: bool) -> Self {
        match value {
            true => State::On,
            false => State::Off,
        }
    }
}
