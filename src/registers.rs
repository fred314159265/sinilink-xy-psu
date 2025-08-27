//! This module is used to define the registers on the XY PSUs.
//!
//! @TODO - How can we handle presets and the need to use preset when setting protections?
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
    /// Manual suggests this should be between 0.75 * 0.85?
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
