//! This module is used to define the registers on the XY PSUs.

use strum_macros::EnumIter;

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum XyRegister {
    /// __R/W__ - Voltage setting.
    ///
    /// Value is u16 in deci-volts. E.g. 5.0V => `500`.
    VSet = 0x00,
    /// __R/W__ - Current setting.
    ///
    /// Value is u16 in milli-volts. E.g. 1.5A => `1500`.
    ISet = 0x01,
    /// __R__ - Output voltage display value.
    VOut = 0x02,
    /// __R__ - Output current display value.
    IOut = 0x03,
    /// __R__ - Output power display value.
    Power = 0x40,
    /// __R__ - Input voltage display value.
    UIn = 0x05,
    /// __R__ - Output Ah is low by 16 bits.
    AhLow = 0x06,
    /// __R__ - Output Ah is high by 16 bits.
    AhHigh = 0x07,
    /// __R__ - Output Wh is low by 16 bits.
    WhLow = 0x08,
    /// __R__ - Output Wh is high by 16 bits.
    WhHigh = 0x09,
    /// __R__ - Open time-length-hours.
    OutH = 0x0A,
    /// __R__ - Start length-correction.
    OutM = 0x0B,
    /// __R__ - Open time-seconds.
    OutS = 0x0C,
    /// __R__ - Internal temperature value.
    TIn = 0x0D,
    /// __R__ - External temperature value.
    TEx = 0x0E,
    /// __R/W__ - Key lock.
    /// * `0` - Unlocked.
    /// * `1` - Locked.
    Lock = 0x0F,
    /// __R/W__ - Protect status.
    ///
    /// See [`ProtectionStatus`](crate::types::ProtectionStatus) for possible protection statuses.
    Protect = 0x10,
    /// __R__ - Constant voltage constant current state.
    /// * `0` - CV.
    /// * `1` - CC.
    ///
    /// See [`ControlMode`](crate::types::ControlMode).
    CvCc = 0x11,
    /// __R/W__ - Switched output.
    /// * `0` - "Closed state".
    /// * `1` - "Open state".
    OnOff = 0x12,
    /// __R/W__ - The temperature symbol.
    FC = 0x13,
    /// __R/W__ - Backlight brightness level.
    ///
    /// Range = 0-5.
    ///
    /// 0 is darkest, and 5 is the brightest.
    BLed = 0x14,
    /// __R/W__ - Rest screen time.
    Sleep = 0x15,
    /// __R__ - Product model.
    Model = 0x16,
    /// __R__ - Firmware version number.
    Version = 0x17,
    /// __R/W__ - Slave address of the machine.
    SlaveAdd = 0x18,
    /// __R/W__ - Baud rate.
    ///
    /// See [`BaudRate`](crate::types::BaudRate) for possible options.
    BaudRateL = 0x19,
    /// __R/W__ - Internal temperature correction.
    TInOffset = 0x1A,
    /// __R/W__ - External temperature correction.
    TExOffset = 0x1B,
    /// __R/W__ - The buzzer switch.
    Buzzer = 0x1C,
    /// __R/W__ - Quickly call up the data group.
    ///
    /// The write value of the quick call-up data group function is 0-9,
    /// and the corresponding data group data will be automatically called up
    /// after writing.
    ExtractM = 0x1D,
    /// __R/W__ - Device status.
    Device = 0x1E,
    /// __R/W__ - MPPT switch.
    MpptSw = 0x1F,
    /// __R/W__ - MPPT maximum point coefficient.
    ///
    /// Manual suggests this should be between 0.75 - 0.85?
    MpptK = 0x20,
    /// __R/W__ - Full current current. (When in MPPT?)
    BatFul = 0x21,
    /// __R/W__ - Constant power switch. (When in MPPT?)
    CwSw = 0x22,
    /// __R/W__ - Constant power value. (When in MPPT?)
    Cw = 0x23,
}

impl From<XyRegister> for u16 {
    fn from(value: XyRegister) -> Self {
        value as u16
    }
}

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

/// Used for setting and reading unit used for temperature readings.
// @TODO read value from device to find out what value is what.
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum TemperatureUnit {
    Celsius = 0x00,
    Fahrenheit = 0x01,
}

impl TryFrom<u16> for TemperatureUnit {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            t if t == TemperatureUnit::Celsius as u16 => Ok(TemperatureUnit::Celsius),
            t if t == TemperatureUnit::Fahrenheit as u16 => Ok(TemperatureUnit::Fahrenheit),
            _ => Err(()),
        }
    }
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
#[repr(u16)]
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

impl From<BaudRate> for u16 {
    fn from(value: BaudRate) -> Self {
        value as u16
    }
}

/// Used to be less ambiguous and whether something is on or off.
#[repr(u16)]
#[derive(Debug, Clone, Copy, Default)]
pub enum State {
    /// Disabled.
    // @TODO Check value of on and off in registers.
    #[default]
    Off = 0x00,
    /// Enabled.
    On = 0x01,
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

/// Simple type to represent temperature depending on the unit used.
#[derive(Debug, Clone, Copy)]
pub enum Temperature {
    Fahrenheit(u16),
    Celsius(u16),
}

impl Temperature {
    pub fn new(value: u16, unit: TemperatureUnit) -> Self {
        match unit {
            TemperatureUnit::Celsius => Self::Celsius(value),
            TemperatureUnit::Fahrenheit => Self::Fahrenheit(value),
        }
    }

    /// Convert this temperature into celsius.
    pub fn as_celsius(&self) -> u16 {
        match *self {
            Self::Celsius(inner) => inner,
            Self::Fahrenheit(inner) => Self::f_to_c(inner),
        }
    }

    /// Convert this temperature into fahrenheit.
    pub fn as_fahrenheit(&self) -> u16 {
        match *self {
            Self::Celsius(inner) => Self::c_to_f(inner),
            Self::Fahrenheit(inner) => inner,
        }
    }

    /// Convert this temperature into a target temperature unit.
    pub fn as_unit(&self, unit: TemperatureUnit) -> u16 {
        match unit {
            TemperatureUnit::Celsius => self.as_celsius(),
            TemperatureUnit::Fahrenheit => self.as_fahrenheit(),
        }
    }

    /// Convert fahrenheit to celsius.
    fn f_to_c(temp_f: u16) -> u16 {
        let multiplied = ((temp_f * 10 - 320) * 5) / 9;

        let decimal = multiplied % 10;
        if decimal >= 5 {
            (multiplied / 10) + 1
        } else {
            multiplied / 10
        }
    }

    /// Convert celsius to fahrenheit.
    fn c_to_f(temp_c: u16) -> u16 {
        // We calculate with one fixed decimal place and manually calculate rounding.
        let multiplied = ((temp_c * 90) / 5) + 320;

        let decimal = multiplied % 10;
        if decimal >= 5 {
            (multiplied / 10) + 1
        } else {
            multiplied / 10
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn temperature_conversions() {
        let temp = Temperature::Celsius(10);
        assert_eq!(temp.as_celsius(), 10);
        assert_eq!(temp.as_fahrenheit(), 50);

        let temp = Temperature::Celsius(21);
        assert_eq!(temp.as_fahrenheit(), 70);

        let temp = Temperature::Fahrenheit(70);
        assert_eq!(temp.as_fahrenheit(), 70);
        assert_eq!(temp.as_celsius(), 21);
    }

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
