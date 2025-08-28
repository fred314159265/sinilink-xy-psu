use fugit::Duration;
use strum::EnumCount;
use strum_macros::EnumCount as EnumCountMacro;
use thiserror::Error;

use crate::{
    error::Error,
    psu::XyPsu,
    register::{State, Temperature, TemperatureUnit},
};

/// Use [`XyPresetBuilder`] to create a preset.
pub struct XyPreset {
    /// Index number of this preset group (0 - 9).
    group: PresetGroup,
    /// Output voltage value.
    voltage_setting_mv: u32,
    /// Output current limit value.
    current_setting_ma: u16,
    /// Protection configuration levels.
    protection: ProtectionConfig,
    output_enable: State,
}

impl XyPreset {
    /// Write this preset to the device.
    pub fn write<S: embedded_io::Read + embedded_io::Write, const L: usize>(
        &self,
        interface: &mut XyPsu<S, L>,
    ) -> Result<(), Error<S::Error>> {
        // To be able to write the temperature limits, we first need to know the unit as configured.
        let unit = interface.read_temperature_unit()?;
        let (start_address, write_buffer) = self.generate_write_data_and_offset(unit);

        interface.write_modbus_bulk(start_address, write_buffer)
    }

    pub fn generate_write_data_and_offset(
        &self,
        temperature_unit: impl Into<TemperatureUnit>,
    ) -> (u16, [u16; XyPresetOffsets::COUNT]) {
        use XyPresetOffsets as XPO;

        let temperature_unit = temperature_unit.into();
        let mut write_buffer: [u16; _] = [0x00; XPO::COUNT];

        write_buffer[XPO::VSet as usize] = u16::try_from(self.voltage_setting_mv / 10).unwrap();
        write_buffer[XPO::ISet as usize] = self.current_setting_ma;
        write_buffer[XPO::SLvp as usize] = (self.protection.under_voltage_mv / 10) as u16;
        write_buffer[XPO::SOvp as usize] = (self.protection.over_voltage_mv / 10) as u16;
        write_buffer[XPO::SOcp as usize] = self.protection.over_current_ma;
        write_buffer[XPO::SOpp as usize] = (self.protection.over_power_mw / 10) as u16;
        write_buffer[XPO::SOhpH as usize] =
            u16::try_from(self.protection.over_time.to_hours()).unwrap();
        write_buffer[XPO::SoHpM as usize] =
            u16::try_from(self.protection.over_time.to_minutes() % 60).unwrap();
        write_buffer[XPO::SOahL as usize] = self.protection.over_capacity_mah as u16;
        write_buffer[XPO::SOahH as usize] = (self.protection.over_capacity_mah >> 16) as u16;
        write_buffer[XPO::SOwhL as usize] = self.protection.over_power_mw as u16;
        write_buffer[XPO::SOwhH as usize] = (self.protection.over_energy_mwh >> 16) as u16;
        write_buffer[XPO::SOtp as usize] =
            self.protection.over_temperature.as_unit(temperature_unit);
        write_buffer[XPO::SIni as usize] = self.output_enable as u16;
        write_buffer[XPO::SEtp as usize] =
            self.protection.over_temperature.as_unit(temperature_unit);

        let start_address = XPO::VSet.address_in_group(self.group);

        (start_address, write_buffer)
    }
}

/// Use this type to create a preset.
pub struct XyPresetBuilder {
    /// Index number of this preset group (0 - 9).
    group: Option<PresetGroup>,
    /// Output voltage value.
    voltage_setting_mv: u32,
    /// Output current limit value.
    current_setting_ma: u16,
    /// Protection configuration levels.
    protection: ProtectionConfig,
    /// What state the output should be in when the preset is loaded.
    output_enable: State,
}

#[allow(clippy::derivable_impls)]
impl Default for XyPresetBuilder {
    fn default() -> Self {
        XyPresetBuilder {
            group: None,
            voltage_setting_mv: 0,
            current_setting_ma: 0,
            protection: ProtectionConfig::default(),
            output_enable: State::default(),
        }
    }
}

impl XyPresetBuilder {
    pub fn new(
        group: impl Into<PresetGroup>,
        voltage_mv: u32,
        current_lim_ma: u16,
    ) -> XyPresetBuilder {
        let group_idx = Some(group.into());

        XyPresetBuilder {
            group: group_idx,
            voltage_setting_mv: voltage_mv,
            current_setting_ma: current_lim_ma,
            ..Default::default()
        }
    }

    /// Let's build it!
    pub fn build(self) -> Result<XyPreset, XyPresetBuilderError> {
        if let Some(group_idx) = self.group {
            Ok(XyPreset {
                group: group_idx,
                voltage_setting_mv: self.voltage_setting_mv,
                current_setting_ma: self.current_setting_ma,
                protection: self.protection,
                output_enable: self.output_enable,
            })
        } else {
            Err(XyPresetBuilderError::InvalidGroupIndex)
        }
    }

    /// Set output state.
    pub fn for_group(mut self, group: impl Into<PresetGroup>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Set all protection options at once.
    pub fn with_protections(mut self, protections: ProtectionConfig) -> Self {
        self.protection = protections;
        self
    }

    /// Set output voltage level.
    pub fn with_set_v(mut self, voltage_mv: u32) -> Self {
        self.voltage_setting_mv = voltage_mv;
        self
    }

    /// Set output current limit.
    pub fn with_set_i_lim(mut self, current_ma: u16) -> Self {
        self.current_setting_ma = current_ma;
        self
    }

    /// Set output state.
    pub fn with_output(mut self, output_enable: impl Into<State>) -> Self {
        self.output_enable = output_enable.into();
        self
    }

    /// Set under-voltage protection level in preset. (@TODO is UVP based on input voltage?)
    pub fn with_uvp(mut self, voltage_mv: u32) -> Self {
        self.protection.under_voltage_mv = voltage_mv;
        self
    }

    /// Set over-voltage protection level in preset.
    pub fn with_ovp(mut self, voltage_mv: u32) -> Self {
        self.protection.over_voltage_mv = voltage_mv;
        self
    }

    /// Set over-current protection level in preset.
    pub fn with_ocp(mut self, current_ma: u16) -> Self {
        self.protection.over_current_ma = current_ma;
        self
    }

    /// Set over-power protection level in preset.
    pub fn with_opp(mut self, power_mw: u32) -> Self {
        self.protection.over_power_mw = power_mw;
        self
    }

    /// Set over time protection level in preset.
    pub fn with_ohp(mut self, duration: Duration<u32, 1, 1>) -> Self {
        self.protection.over_time = duration;
        self
    }

    /// Set over capacity protection level in preset. Units: mAh.
    pub fn with_oahp(mut self, capacity_mah: u32) -> Self {
        self.protection.over_capacity_mah = capacity_mah;
        self
    }

    /// Set over energy protection level in preset. Units: mWh.
    pub fn with_owhp(mut self, energy_mwh: u32) -> Self {
        self.protection.over_energy_mwh = energy_mwh;
        self
    }

    /// Set over temperature protection level in preset.
    pub fn with_otp(mut self, temperature: impl Into<Temperature>) -> Self {
        self.protection.over_temperature = temperature.into();
        self
    }
}

#[derive(Error, Debug, Clone, Copy)]
pub enum XyPresetBuilderError {
    #[error("Preset group no not set")]
    InvalidGroupIndex,
}

/// This struct is used to define the configuration of the protection features. E.g. over-voltage protection.
#[derive(Debug)]
pub struct ProtectionConfig {
    /// Under-voltage protection level in milli-volts.
    pub under_voltage_mv: u32,
    /// Over-voltage protection level in milli-volts.
    pub over_voltage_mv: u32,
    /// Over-current protection level in milli-amps.
    pub over_current_ma: u16,
    /// Over-power protection level in milli-watts.
    pub over_power_mw: u32,
    /// Over-time protection duration.
    pub over_time: Duration<u32, 1, 1>,
    /// Over capacity protection level in milli-amp hours.
    pub over_capacity_mah: u32,
    /// Over energy protection level in milli-watt hours.
    pub over_energy_mwh: u32,
    /// Over-temperature protection level in unit as configured.
    pub over_temperature: Temperature,
}

/// Default protections are essentially disabled.
impl Default for ProtectionConfig {
    fn default() -> Self {
        // @TODO confirm units to raw conversion is as expected.
        ProtectionConfig {
            under_voltage_mv: 0,
            over_voltage_mv: 99999,
            over_current_ma: 9999,
            over_power_mw: 99999,
            over_time: Duration::<u32, _, _>::hours(999),
            over_capacity_mah: 99999,
            over_energy_mwh: 99999,
            over_temperature: Temperature::Celsius(100),
        }
    }
}

// impl ProtectionConfig {
//     pub fn write
// }

/// The base address of the first preset registers.
///
/// Base address of preset = PRESET_OFFSET + {group number} * 0x10.
///
/// There are 10 groups: M0 - M9.
pub static PRESET_OFFSET: u16 = 0x50;

/// These are the offsets from the base address of each preset group.
///
/// See [`PRESET_OFFSET`] for calculating the base address of any group.
#[derive(Debug, Copy, Clone, EnumCountMacro, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum XyPresetOffsets {
    /// __R/W__ - Voltage setting.
    ///
    /// Value is u16 in deci-volts. E.g. 5.0V => `500`.
    VSet = 0x00,
    /// __R/W__ - Current setting.
    ///
    /// Value is u16 in milli-volts. E.g. 1.5A => `1500`.
    ISet = 0x01,
    /// __R/W__ - Low voltage protection.
    SLvp = 0x02,
    /// __R/W__ - Over voltage protection.
    SOvp = 0x03,
    /// __R/W__ - Over current protection.
    SOcp = 0x04,
    /// __R/W__ - Over power protection.
    SOpp = 0x05,
    /// __R/W__ - Over time protection - hours.
    SOhpH = 0x06,
    /// __R/W__ - Over time protection - minutes.
    SoHpM = 0x07,
    /// __R/W__ - Over capacity protection lower 16 bits.
    SOahL = 0x08,
    /// __R/W__ - Over capacity protection upper 16 bits.
    SOahH = 0x09,
    /// __R/W__ - Over energy protection lower 16 bits.
    SOwhL = 0x0A,
    /// __R/W__ - Over energy protection upper 16 bits.
    SOwhH = 0x0B,
    /// __R/W__ - Over temperature protection.
    SOtp = 0x0C,
    /// __R/W__ - Power output enable switch.
    SIni = 0x0D,
    /// __R/W__ - External temperature protection level.
    // @TODO confirm this is correct description.
    SEtp = 0x0E,
}

impl XyPresetOffsets {
    /// Return the address of this register provided the group number (0 - 9).
    pub fn address_in_group(&self, group: PresetGroup) -> u16 {
        PRESET_OFFSET + (group as u16 * 0x10) + *self as u16
    }
}

/// This enum represents all possible preset groups.
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum PresetGroup {
    Group0 = 0x00,
    Group1 = 0x01,
    Group2 = 0x02,
    Group3 = 0x03,
    Group4 = 0x04,
    Group5 = 0x05,
    Group6 = 0x06,
    Group7 = 0x07,
    Group8 = 0x08,
    Group9 = 0x09,
}

impl TryFrom<u32> for PresetGroup {
    // @TODO should probably have own error type, but I am lazy.
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use PresetGroup as PG;
        match value {
            0 => Ok(PG::Group0),
            1 => Ok(PG::Group1),
            2 => Ok(PG::Group2),
            3 => Ok(PG::Group3),
            4 => Ok(PG::Group4),
            5 => Ok(PG::Group5),
            6 => Ok(PG::Group6),
            7 => Ok(PG::Group7),
            8 => Ok(PG::Group8),
            9 => Ok(PG::Group9),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn preset_register_adress() {
        let register = XyPresetOffsets::VSet;

        let address = register.address_in_group(PresetGroup::Group0);
        assert_eq!(address, 0x50);

        let address = register.address_in_group(PresetGroup::Group3);
        assert_eq!(address, 0x80);

        let register = XyPresetOffsets::SOwhL;
        let address = register.address_in_group(PresetGroup::Group3);
        assert_eq!(address, 0x80 + 0x0A);
    }

    #[test]
    fn preset_write_data_generation() {
        // Create a preset such that all registers should be non-zero.
        let preset = XyPresetBuilder::new(PresetGroup::Group3, 5000, 1000)
            .with_output(true)
            .with_uvp(1000)
            .with_ohp(Duration::<u32, _, _>::hours(10u32) + Duration::<u32, _, _>::minutes(10u32))
            .build()
            .unwrap();

        // Generate payload.
        let (start_address, write_buffer) = preset.generate_write_data_and_offset(TemperatureUnit::Celsius);

        // Check start address is as expected.
        assert_eq!(start_address, 0x80);

        // Check all values have been given a value.
        for double in write_buffer {
            assert_ne!(double, 0);
        }
    }
}
