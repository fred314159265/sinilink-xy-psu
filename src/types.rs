//! This module contains types relevant to the PSU Modbus data types.

use strum_macros::EnumIter;

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

impl From<ControlMode> for u16 {
    fn from(value: ControlMode) -> Self {
        match value {
            ControlMode::Cv => 0x00,
            ControlMode::Cc => 0x01,
        }
    }
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

/// "Protection status register".
#[derive(Debug, EnumIter, PartialEq, Clone, Copy)]
#[repr(u16)]
pub enum ProtectionStatus {
    /// 0: Alarm code
    AlarmCode = 0x00,
    /// 1: OVP overvoltage protection.
    OverVoltage = 0x01,
    /// 2: OCP overcurrent protection.
    OverCurrent = 0x02,
    /// 3: OPP, over-power protection.
    OverPower = 0x03,
    /// 4: LVP input undervoltage protection.
    InputUndervoltageProtection = 0x04,
    /// 5: OAH maximum output capacity.
    MaximumOutputCapacity = 0x05,
    /// 6: OHP maximum output time.
    MaximumOutputTime = 0x06,
    /// 7: OTP over-temperature protection.
    OverTemperature = 0x07,
    /// 8: OEP, with no output protection.
    NoOutput = 0x08,
    /// 9: OWH maximum energy output.
    MaximumEnergyOutput = 0x09,
    /// 10: ICP maximum input current protection.
    MaximumInputCurrent = 0x0A,
    /// 11: ETP, external temperature protection.
    ExternalTemperature = 0x0B,
}

impl ProtectionStatus {
    const MAX_VALUE: u16 = Self::ExternalTemperature as u16;
}

impl From<u16> for ProtectionStatus {
    fn from(value: u16) -> Self {
        use ProtectionStatus as PS;
        if value <= Self::MAX_VALUE {
            match value {
                0x00 => PS::AlarmCode,
                0x01 => PS::OverVoltage,
                0x02 => PS::OverCurrent,
                0x03 => PS::OverPower,
                0x04 => PS::InputUndervoltageProtection,
                0x05 => PS::MaximumOutputCapacity,
                0x06 => PS::MaximumOutputTime,
                0x07 => PS::OverTemperature,
                0x08 => PS::NoOutput,
                0x09 => PS::MaximumEnergyOutput,
                0x0A => PS::MaximumInputCurrent,
                0x0B => PS::ExternalTemperature,
                _ => panic!(),
            }
        } else {
            // Default to no alarms active if outside of expected values.
            Self::AlarmCode
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn protection_status_conversions() {
        // We are checking converting between u16 and ProtectionStatus is the same in both directions.
        for status in ProtectionStatus::iter() {
            let converted = ProtectionStatus::from(status as u16);
            // Converted value back as u16 should be the same as we started with.
            assert_eq!(converted, status);
        }
    }

    #[test]
    fn protection_status_max_value() {
        // We are checking ProtectionStatus::MAX_VALUE is correct.
        let mut max_value = 0;
        for status in ProtectionStatus::iter() {
            if status as u16 > max_value {
                max_value = status as u16;
            }
        }
        assert_eq!(max_value, ProtectionStatus::MAX_VALUE);
    }
}
