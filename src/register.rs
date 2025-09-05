//! This module is used to define the registers on the XY PSUs.

use modular_bitfield::prelude::*;

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum XyRegister {
    /// __R/W__ - Voltage setting.
    ///
    /// Value is u16 in centi-volts. E.g. 5.0V => `500`.
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
    Power = 0x04,
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
    /// See [`ProtectionStatus`] for possible protection statuses.
    ///
    /// Writing a `0x00` to this register will clear any active protections. This will stop the
    /// beeping on the device.
    Protect = 0x10,
    /// __R__ - Constant voltage constant current state.
    /// * `0` - CV.
    /// * `1` - CC.
    ///
    /// See [`ControlMode`].
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
    /// See [`BaudRate`] for possible options.
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
    /// __R/W__ - "Device status."
    ///
    /// Used to enter and exit sleep (screen off, ON/OFF button fading in and out red.)
    ///
    /// I don't know why it is called "device".
    ///
    /// 0x00 = sleep/off
    /// 0x01 = awake/on
    ///
    /// This means it is compatible with [`State`].
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

impl From<u16> for ControlMode {
    fn from(value: u16) -> Self {
        if value != 0 {
            ControlMode::Cc
        } else {
            ControlMode::Cv
        }
    }
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

impl TryFrom<u16> for BaudRate {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        use BaudRate as BR;
        match value {
            x if x == BR::_9600 as u16 => Ok(BR::_9600),
            x if x == BR::_14400 as u16 => Ok(BR::_14400),
            x if x == BR::_19200 as u16 => Ok(BR::_19200),
            x if x == BR::_38400 as u16 => Ok(BR::_38400),
            x if x == BR::_5600 as u16 => Ok(BR::_5600),
            x if x == BR::_576000 as u16 => Ok(BR::_576000),
            x if x == BR::_115200 as u16 => Ok(BR::_115200),
            x if x == BR::_2400 as u16 => Ok(BR::_2400),
            x if x == BR::_4800 as u16 => Ok(BR::_4800),
            _ => Err(()),
        }
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

impl std::ops::Not for State {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            State::Off => State::On,
            State::On => State::Off,
        }
    }
}

impl From<State> for u16 {
    fn from(value: State) -> Self {
        value as u16
    }
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
#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct ProtectionStatus {
    /// OVP overvoltage protection.
    #[allow(dead_code)]
    over_voltage: bool,
    /// OCP overcurrent protection.
    #[allow(dead_code)]
    over_current: bool,
    /// OPP, over-power protection.
    #[allow(dead_code)]
    over_power: bool,
    /// LVP input under voltage protection.
    #[allow(dead_code)]
    under_voltage_input: bool,
    /// OAH maximum output capacity.
    #[allow(dead_code)]
    over_capacity: bool,
    /// OHP maximum output time.
    #[allow(dead_code)]
    over_time: bool,
    /// OTP over-temperature protection.
    #[allow(dead_code)]
    over_temperature_internal: bool,
    /// OEP, with no output protection. - I don't understand what this means.
    #[allow(dead_code)]
    oep: bool,
    /// OWH maximum energy output.
    #[allow(dead_code)]
    over_energy: bool,
    /// ICP maximum input current protection.
    #[allow(dead_code)]
    over_current_input: bool,
    /// ETP, external temperature protection.
    #[allow(dead_code)]
    over_temperature_external: bool,
    // Last 5 MSB are not used.
    #[skip]
    __: B5,
}

/// All possible supported brightness levels of the display.
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum BacklightBrightness {
    Level0 = 0x00,
    Level1 = 0x01,
    Level2 = 0x02,
    Level3 = 0x03,
    Level4 = 0x04,
    Level5 = 0x05,
}

impl TryFrom<u16> for BacklightBrightness {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        use BacklightBrightness as BB;
        match value {
            x if x == BB::Level0 as u16 => Ok(BB::Level0),
            x if x == BB::Level1 as u16 => Ok(BB::Level1),
            x if x == BB::Level2 as u16 => Ok(BB::Level2),
            x if x == BB::Level3 as u16 => Ok(BB::Level3),
            x if x == BB::Level4 as u16 => Ok(BB::Level4),
            x if x == BB::Level5 as u16 => Ok(BB::Level5),
            _ => Err(()),
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
    /// Create a [`Temperature`] from a temperature value pass in using the units of centi-degree C/F.
    ///
    /// E.g. 294 => 29.4° but get rounded to 29°
    pub fn from_centi(value: u16, unit: TemperatureUnit) -> Self {
        let rounded = Self::div_10_and_round(value);
        Self::new(rounded, unit)
    }

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
        Self::div_10_and_round(multiplied)
    }

    /// Convert celsius to fahrenheit.
    fn c_to_f(temp_c: u16) -> u16 {
        // We calculate with one fixed centimal place and manually calculate rounding.
        let multiplied = ((temp_c * 90) / 5) + 320;
        Self::div_10_and_round(multiplied)
    }

    fn div_10_and_round(value: u16) -> u16 {
        let centimal = value % 10;
        if centimal >= 5 {
            (value / 10) + 1
        } else {
            value / 10
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
