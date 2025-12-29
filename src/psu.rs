use crate::{
    error::{Error, Result},
    preset::{PresetGroup, ProtectionConfig, XyPresetBuilder},
    register::{
        BacklightBrightness, BaudRate, ControlMode, ProductModel, ProtectionStatus, State,
        Temperature, TemperatureUnit, XyRegister,
    },
    scaling::ScalingFactors,
};
use embedded_io::Error as _;
use fugit::Duration;

/// You can create a XyPsu using any interface which implements [embedded_io::Read] & [embedded_io::Write].
///
/// For it's methods, we generally use the nomenclature that "set" meant to write a configuration and "get" means to read
/// back a configuration value. Where as "read" means to get a measured value.
pub struct XyPsu<S: embedded_io::Read + embedded_io::Write, const L: usize = 128> {
    interface: S,
    /// Default for PSU is 0x01.
    unit_id: u8,
    /// Scaling factors for this PSU model. Lazily loaded on first use of scaled functions.
    scaling: Option<ScalingFactors>,
}

impl<S: embedded_io::Read + embedded_io::Write, const L: usize> XyPsu<S, L> {
    /// Create a new XyPsu instance with the given interface and unit ID
    ///
    /// Scaling factors are lazily loaded on first use of scaled measurement functions.
    /// You can manually specify scaling factors using [`Self::set_scaling_factors`].
    pub fn new(interface: S, unit_id: u8) -> Self {
        Self {
            interface,
            unit_id,
            scaling: None,
        }
    }

    /// Manually set the scaling factors for this PSU.
    ///
    /// This allows you to override the automatic scaling factor detection for models
    /// with unknown or incorrect scaling factors. Once set, these scaling factors will
    /// be used for all scaled measurement and configuration functions.
    pub fn set_scaling_factors(&mut self, scaling: ScalingFactors) {
        self.scaling = Some(scaling);
    }

    /// Ensure scaling factors are loaded for this PSU model.
    ///
    /// This is called automatically by scaled measurement functions.
    /// If the model's scaling factors are unknown, returns `ScalingNotAvailable` error.
    ///
    /// Returns a copy of the scaling factors so that self can be borrowed mutably afterwards.
    pub(crate) fn ensure_scaling(&mut self) -> Result<ScalingFactors, S::Error> {
        // If already cached, return a copy
        if let Some(scaling) = self.scaling {
            return Ok(scaling);
        }

        // Otherwise, fetch model and lookup scaling factors
        let model = self.get_product_model()?;
        let scaling = model.scaling_factors().ok_or(Error::ScalingNotAvailable)?;

        // Cache for future use
        self.scaling = Some(scaling);
        Ok(scaling)
    }

    /// Return the measured output voltage in millivolts.
    ///
    /// Requires known scaling factors for the PSU model. Returns `ScalingNotAvailable`
    /// error if the model's scaling factors are unknown.
    ///
    /// For unknown models, use [`set_scaling_factors`](Self::set_scaling_factors) to manually
    /// specify scaling factors.
    pub fn read_output_voltage_mv(&mut self) -> Result<u32, S::Error> {
        let scaling = self.ensure_scaling()?;
        let raw = self.read_modbus_single(XyRegister::VOut)?;
        Ok(scaling.raw_to_voltage_mv(raw))
    }

    /// Return the measured supply input voltage in millivolts.
    ///
    /// Requires known scaling factors for the PSU model. Returns `ScalingNotAvailable`
    /// error if the model's scaling factors are unknown.
    ///
    /// For unknown models, use [`set_scaling_factors`](Self::set_scaling_factors) to manually
    /// specify scaling factors.
    pub fn read_input_voltage_mv(&mut self) -> Result<u32, S::Error> {
        let scaling = self.ensure_scaling()?;
        let raw = self.read_modbus_single(XyRegister::UIn)?;
        Ok(scaling.raw_to_voltage_mv(raw))
    }

    /// Return the measured output current in milliamps.
    ///
    /// Requires known scaling factors for the PSU model. Returns `ScalingNotAvailable`
    /// error if the model's scaling factors are unknown.
    ///
    /// For unknown models, use [`set_scaling_factors`](Self::set_scaling_factors) to manually
    /// specify scaling factors.
    pub fn read_current_ma(&mut self) -> Result<u32, S::Error> {
        let scaling = self.ensure_scaling()?;
        let raw = self.read_modbus_single(XyRegister::IOut)?;
        Ok(scaling.raw_to_current_ma(raw))
    }

    /// Return the measured output power in milliwatts.
    ///
    /// Requires known scaling factors for the PSU model. Returns `ScalingNotAvailable`
    /// error if the model's scaling factors are unknown.
    ///
    /// For unknown models, use [`set_scaling_factors`](Self::set_scaling_factors) to manually
    /// specify scaling factors.
    pub fn read_power_mw(&mut self) -> Result<u32, S::Error> {
        let scaling = self.ensure_scaling()?;
        let raw = self.read_modbus_single(XyRegister::Power)?;
        Ok(scaling.raw_to_power_mw(raw))
    }

    /// Return the measured output energy in milliwatt-hours.
    pub fn read_energy_mwh(&mut self) -> Result<u32, S::Error> {
        let energy_mwh_lower = self.read_modbus_single(XyRegister::WhLow)? as u32;
        let energy_mwh_upper = self.read_modbus_single(XyRegister::WhHigh)? as u32;
        // @TODO confirm raw value in milli-wattshours.
        Ok(energy_mwh_lower + (energy_mwh_upper << 16))
    }

    /// Return the measured output capacity in milliamp-hours.
    pub fn read_capacity_mah(&mut self) -> Result<u32, S::Error> {
        let energy_mah_lower = self.read_modbus_single(XyRegister::AhLow)? as u32;
        let energy_mah_upper = self.read_modbus_single(XyRegister::AhHigh)? as u32;
        // @TODO confirm raw value in milli-amphours.
        Ok(energy_mah_lower + (energy_mah_upper << 16))
    }

    /// Return the duration that the output has been enabled.
    ///
    /// @TODO create std version of this method.
    pub fn read_output_time(&mut self) -> Result<Duration<u32, 1, 1>, S::Error> {
        let time_h = self.read_modbus_single(XyRegister::OutH)? as u32;
        let time_m = self.read_modbus_single(XyRegister::OutM)? as u32;
        let time_s = self.read_modbus_single(XyRegister::OutS)? as u32;
        let duration = Duration::<u32, 1, 1>::hours(time_h)
            + Duration::<u32, 1, 1>::minutes(time_m)
            + Duration::<u32, 1, 1>::secs(time_s);
        Ok(duration)
    }

    /// Return the measured internal temperature.
    ///
    /// Unit of measurement depends on setting.
    pub fn read_temperature_internal(&mut self) -> Result<Temperature, S::Error> {
        let unit = self.get_temperature_unit()?;
        let temp_internal_raw = self.read_modbus_single(XyRegister::TIn)?;
        Ok(Temperature::from_centi(temp_internal_raw, unit))
    }

    /// Return the measured external temperature sensor.
    ///
    /// Unit of measurement depends on setting. See [Self::set_temperature_unit].
    ///
    /// @TODO test with external temp sensor.
    pub fn read_temperature_external(&mut self) -> Result<Temperature, S::Error> {
        let unit = self.get_temperature_unit()?;
        let temp_external_raw = self.read_modbus_single(XyRegister::TEx)?;
        Ok(Temperature::from_centi(temp_external_raw, unit))
    }

    /// Enable/disable the key lock.
    pub fn set_lock_state(&mut self, state: impl Into<State>) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::Lock, state.into() as u16)?;
        Ok(())
    }

    /// Get the current state of the key lock.
    pub fn get_lock_state(&mut self) -> Result<State, S::Error> {
        let value = self.read_modbus_single(XyRegister::Lock)?;
        let state = State::from(value != 0);
        Ok(state)
    }

    /// Get the currently active control mode. (CV or CC.)
    pub fn get_current_control_mode(&mut self) -> Result<ControlMode, S::Error> {
        let value = self.read_modbus_single(XyRegister::CvCc)?;
        let state = ControlMode::from(value);
        Ok(state)
    }

    /// Enable/disable the output.
    pub fn set_output_state(&mut self, state: impl Into<State>) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::OnOff, state.into() as u16)?;
        Ok(())
    }

    /// Read whether the output is enabled or disabled.
    pub fn get_output_state(&mut self) -> Result<State, S::Error> {
        let value = self.read_modbus_single(XyRegister::OnOff)?;
        let state = State::from(value != 0);
        Ok(state)
    }

    /// Read the current firmware version.
    ///
    /// Decimal value of `136` -> `v1.3.6`.
    pub fn get_firmware_version(&mut self) -> Result<u16, S::Error> {
        let value = self.read_modbus_single(XyRegister::Version)?;
        Ok(value)
    }

    /// Set the Modbus unit ID of this PSU.
    ///
    /// Appears to only be applied after a power cycle.
    pub fn set_slave_address(&mut self, address: u8) -> Result<(), S::Error> {
        // Only 1-247 range is suitable ID for single Modbus device.
        assert!(address <= 247);
        self.write_modbus_single(XyRegister::SlaveAdd, address as u16)?;
        Ok(())
    }

    /// Get the current Modbus unit ID of this PSU.
    pub fn get_slave_address(&mut self) -> Result<u8, S::Error> {
        let value = self.read_modbus_single(XyRegister::SlaveAdd)?;
        let address = u8::try_from(value)?;
        Ok(address)
    }

    /// Sets the configured baud rate on the PSU.
    ///
    /// Appears to only be applied after a power cycle.
    pub fn set_baudrate(&mut self, baud_rate: BaudRate) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::BaudRateL, baud_rate as u16)?;
        Ok(())
    }

    /// Reads the configured baud rate on the PSU.
    pub fn get_baudrate(&mut self) -> Result<BaudRate, S::Error> {
        let value = self.read_modbus_single(XyRegister::BaudRateL)?;
        let baudrate = BaudRate::try_from(value)?;
        Ok(baudrate)
    }

    /// Set the temperature unit to use.
    pub fn set_temperature_unit(&mut self, unit: TemperatureUnit) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::FC, unit as u16)?;
        Ok(())
    }

    /// Return the temperature unit in use.
    pub fn get_temperature_unit(&mut self) -> Result<TemperatureUnit, S::Error> {
        let value = self.read_modbus_single(XyRegister::FC)?;
        let unit = TemperatureUnit::try_from(value)?;
        Ok(unit)
    }

    /// Set the output target voltage. Value supplied in millivolts.
    ///
    /// Requires known scaling factors for the PSU model. Returns `ScalingNotAvailable`
    /// error if the model's scaling factors are unknown.
    ///
    /// For unknown models, use [`set_scaling_factors`](Self::set_scaling_factors) to manually
    /// specify scaling factors.
    pub fn set_output_voltage_mv(&mut self, voltage_mv: u32) -> Result<(), S::Error> {
        let scaling = self.ensure_scaling()?;
        let raw = scaling.voltage_mv_to_raw(voltage_mv);
        self.write_modbus_single(XyRegister::VSet, raw)?;
        Ok(())
    }

    /// Get the current output target voltage. Value returned in millivolts.
    ///
    /// Requires known scaling factors for the PSU model. Returns `ScalingNotAvailable`
    /// error if the model's scaling factors are unknown.
    ///
    /// For unknown models, use [`set_scaling_factors`](Self::set_scaling_factors) to manually
    /// specify scaling factors.
    pub fn get_output_voltage_mv(&mut self) -> Result<u32, S::Error> {
        let scaling = self.ensure_scaling()?;
        let raw = self.read_modbus_single(XyRegister::VSet)?;
        Ok(scaling.raw_to_voltage_mv(raw))
    }

    /// Set the output current limit. Value supplied in milliamps.
    ///
    /// Requires known scaling factors for the PSU model. Returns `ScalingNotAvailable`
    /// error if the model's scaling factors are unknown.
    ///
    /// For unknown models, use [`set_scaling_factors`](Self::set_scaling_factors) to manually
    /// specify scaling factors.
    pub fn set_current_limit_ma(&mut self, current_ma: u32) -> Result<(), S::Error> {
        let scaling = self.ensure_scaling()?;
        let raw = scaling.current_ma_to_raw(current_ma);
        self.write_modbus_single(XyRegister::ISet, raw)?;
        Ok(())
    }

    /// Get the current output current limit value. Value returned in milliamps.
    ///
    /// Requires known scaling factors for the PSU model. Returns `ScalingNotAvailable`
    /// error if the model's scaling factors are unknown.
    ///
    /// For unknown models, use [`set_scaling_factors`](Self::set_scaling_factors) to manually
    /// specify scaling factors.
    pub fn get_current_limit_ma(&mut self) -> Result<u32, S::Error> {
        let scaling = self.ensure_scaling()?;
        let raw = self.read_modbus_single(XyRegister::ISet)?;
        Ok(scaling.raw_to_current_ma(raw))
    }

    /// Returns the raw register values for "MODEL" -> product model
    ///
    /// See [Self::get_product_model] for a method which tries to interpret this data.
    pub fn get_product_model_raw(&mut self) -> Result<u16, S::Error> {
        self.read_modbus_single(XyRegister::Model)
    }

    /// Returns the interpreted product model.
    ///
    /// Only models where the ID has been observed are supported.
    ///
    /// If you have a model which is not supported, please submit a Github
    /// ticket with information so we can add it!
    pub fn get_product_model(&mut self) -> Result<ProductModel, S::Error> {
        use ProductModel as PM;

        let raw = self.get_product_model_raw()?;

        match raw {
            x if x == PM::XY6020L as u16 => Ok(PM::XY6020L),
            x if x == PM::XY12522 as u16 => Ok(PM::XY12522),
            x if x == PM::XY7025 as u16 => Ok(PM::XY7025),
            x if x == PM::XY3607F as u16 => Ok(PM::XY3607F),
            // x if x == PM::XYSK60S as u16 => Ok(PM::XYSK60S),
            // x if x == PM::XYSK120S as u16 => Ok(PM::XYSK120S),
            // x if x == PM::XYSK150S as u16 => Ok(PM::XYSK150S),
            // x if x == PM::XY3606B as u16 => Ok(PM::XY3606B),
            // x if x == PM::XY6506 as u16 => Ok(PM::XY6506),
            // x if x == PM::XY6506S as u16 => Ok(PM::XY6506S),
            // x if x == PM::XY6509 as u16 => Ok(PM::XY6509),
            // x if x == PM::XY6509X as u16 => Ok(PM::XY6509X),
            _ => unimplemented!(
                "The raw ID  0x{:04X} | {} is not currently recognised by this library.",
                raw,
                raw
            ),
        }
    }

    /// Configure the baud rate of the PSU.
    pub fn set_baud_rate(&mut self, baud_rate: BaudRate) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::BaudRateL, baud_rate)
    }

    /// Return which protections have been triggered, if any.
    pub fn get_protection_status(&mut self) -> Result<ProtectionStatus, S::Error> {
        let raw = self.read_modbus_single(XyRegister::Protect)?;
        let bytes = raw.to_le_bytes();
        let status = ProtectionStatus::from_bytes(bytes);
        Ok(status)
    }

    /// Clear any active protection flags.
    pub fn clear_protections(&mut self) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::Protect, 0x00_u16)?;
        Ok(())
    }

    /// Set the backlight brightness level.
    pub fn set_backlight(&mut self, level: BacklightBrightness) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::BLed, level as u16)?;
        Ok(())
    }

    /// Get the current backlight brightness level.
    pub fn get_backlight(&mut self) -> Result<BacklightBrightness, S::Error> {
        let value = self.read_modbus_single(XyRegister::BLed)?;
        let level = BacklightBrightness::try_from(value)?;
        Ok(level)
    }

    /// Enable/disable the buzzer..
    pub fn set_buzzer_enabled(&mut self, state: impl Into<State>) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::Buzzer, state.into() as u16)?;
        Ok(())
    }

    /// Get the current buzzer enable state.
    pub fn get_buzzer_enabled(&mut self) -> Result<State, S::Error> {
        let value = self.read_modbus_single(XyRegister::Buzzer)?;
        let state = State::from(value != 0);
        Ok(state)
    }

    /// Activate preset by index.
    pub fn set_active_preset(&mut self, group: impl Into<PresetGroup>) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::ExtractM, group.into() as u16)?;
        Ok(())
    }

    /// Get the currently active preset group.
    ///
    /// Returns the preset group (0-9) that is currently active on the PSU.
    pub fn get_active_preset(&mut self) -> Result<PresetGroup, S::Error> {
        let value = self.read_modbus_single(XyRegister::ExtractM)?;
        let group = PresetGroup::try_from(value)?;
        Ok(group)
    }

    /// Enter or exit sleep mode. (Screen off, ON/OFF button fading in and out red.)
    pub fn set_sleep_state(&mut self, activate_sleep: impl Into<State>) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::Device, !activate_sleep.into() as u16)?;
        Ok(())
    }

    /// Get whether the device is currently in sleep mode.
    pub fn get_sleep_state(&mut self) -> Result<State, S::Error> {
        let value = self.read_modbus_single(XyRegister::Device)?;
        let state = State::from(value != 0);
        Ok(state)
    }

    // /// Set the offset used for the internal temperature sensor.
    // pub fn set_temperature_offset_input(&mut self, offset: impl Into<Temperature>) -> Result<(), S::Error> {
    //     let unit = self.get_temperature_unit()?;
    //     let temperature_in_unit = offset.into().as_unit(unit);
    //     println!("Offset: {}", temperature_in_unit);
    //     self.write_modbus_single(XyRegister::TInOffset, temperature_in_unit * 10)?;
    //     Ok(())
    // }

    // /// Get the current offset used for the internal temperature sensor.
    // pub fn get_temperature_offset_input(&mut self) -> Result<Temperature, S::Error> {
    //     let unit = self.get_temperature_unit()?;
    //     let value = self.read_modbus_single(XyRegister::TInOffset)?;
    //     let temp_offset = Temperature::from_centi(value, unit);
    //     Ok(temp_offset)
    // }

    /// Enable or disable MPPT functionality.
    pub fn set_mppt_enabled(&mut self, activate_sleep: impl Into<State>) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::MpptSw, activate_sleep.into() as u16)?;
        Ok(())
    }

    /// Get whether MPPT is currently in enabled or disabled.
    pub fn get_mppt_enabled(&mut self) -> Result<State, S::Error> {
        let value = self.read_modbus_single(XyRegister::MpptSw)?;
        let state = State::from(value != 0);
        Ok(state)
    }

    /// Set the MPPT coefficient. Recommended [`75` - `85`]
    ///
    /// Note: Value passed in is 10x bigger than shown on screen.
    pub fn set_mppt_k_value(&mut self, mppt_k: u16) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::MpptK, mppt_k)?;
        Ok(())
    }

    /// Get the current MPPT coefficient. Default value of `80`.
    ///
    /// Value returned is 10x what is shown on the display.
    ///
    /// E.g. `0.75` on display => `75` as retuned by this function.
    pub fn get_mppt_k_value(&mut self) -> Result<u16, S::Error> {
        let value = self.read_modbus_single(XyRegister::MpptK)?;
        Ok(value)
    }

    // MPPT max charging current doesn't appear to work. Normal current limit value does seem to work.
    // /// Set the MPPT maximum charging current in units of milli-amps.
    // pub fn set_mppt_max_current_ma(&mut self, current_ma: u32) -> Result<(), S::Error> {
    //     let current_raw: u16 = (current_ma / 10).try_into()?;
    //     self.write_modbus_single(XyRegister::BatFul, current_raw)?;
    //     Ok(())
    // }

    // /// Get the MPPT maximum charging current in units of milli-amps.
    // pub fn get_mppt_max_current_ma(&mut self) -> Result<u32, S::Error> {
    //     let value = self.read_modbus_single(XyRegister::BatFul)?;
    //     Ok(value as u32 * 10)
    // }

    /// Enable or disable constant power mode.
    pub fn set_constant_power_enabled(
        &mut self,
        activate_sleep: impl Into<State>,
    ) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::CwSw, activate_sleep.into() as u16)?;
        Ok(())
    }

    /// Get whether constant power mode is currently enabled or disabled.
    pub fn get_constant_power_enabled(&mut self) -> Result<State, S::Error> {
        let value = self.read_modbus_single(XyRegister::CwSw)?;
        let state = State::from(value != 0);
        Ok(state)
    }

    /// Set the constant power power level. Units of watts.
    ///
    /// This can be set without enabling constant power mode.
    pub fn set_constant_power_level(&mut self, mppt_k: u16) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::Cw, mppt_k)?;
        Ok(())
    }

    /// Get the current constant power power level. Units of watts.
    ///
    /// This can be read without enabling constant power mode.
    pub fn get_constant_power_level(&mut self) -> Result<u16, S::Error> {
        let value = self.read_modbus_single(XyRegister::Cw)?;
        Ok(value)
    }

    /// Write to a single register of the PSU.
    pub fn write_modbus_single(
        &mut self,
        register: impl Into<u16>,
        data: impl Into<u16>,
    ) -> Result<(), S::Error> {
        // @TODO we could directly compare the incoming bytes to our buffer in sequence without storing all the RX'd bytes a second buffer.
        let mut buff_1: heapless::Vec<u8, L> = heapless::Vec::new();
        let mut buff_2: heapless::Vec<u8, L> = heapless::Vec::new();

        let mut req = rmodbus::client::ModbusRequest::new(self.unit_id, rmodbus::ModbusProto::Rtu);
        req.generate_set_holding(register.into(), data.into(), &mut buff_1)?;

        self.interface
            .write_all(&buff_1)
            .map_err(crate::error::Error::SerialError)?;

        // Read the response - keep reading until we get WouldBlock or have enough data
        let mut temp_buf = [0u8; 8]; // Temporary buffer for single reads
        loop {
            match self.interface.read(&mut temp_buf) {
                Ok(bytes_read) => {
                    // Add the read bytes to our buffer
                    if buff_2.extend_from_slice(&temp_buf[0..bytes_read]).is_err() {
                        return Err(crate::error::Error::BufferError);
                    }
                    // Check if we have enough data for a minimal response (unit_id + function + byte_count + at least 2 data bytes + 2 CRC)
                    if buff_2.len() >= 8 {
                        break;
                    }
                }
                Err(e) => {
                    // If WouldBlock and we have some data, break and try to parse
                    if matches!(
                        e.kind(),
                        embedded_io::ErrorKind::Other | embedded_io::ErrorKind::TimedOut
                    ) && !buff_2.is_empty()
                    {
                        break;
                    }
                    // Other errors should be propagated
                    return Err(crate::error::Error::SerialError(e));
                }
            }
        }
        if buff_1.as_slice() != buff_2.as_slice() {
            Err(crate::error::Error::InvalidResponse)
        } else {
            Ok(())
        }
    }

    /// Write to multiple, sequential PSU registers.
    pub fn write_modbus_bulk(
        &mut self,
        start_register: impl Into<u16>,
        data: impl AsRef<[u16]>,
    ) -> Result<(), S::Error> {
        let start_register = start_register.into();
        let data = data.as_ref();

        // @TODO we could directly compare the incoming bytes to our buffer in sequence without storing all the RX'd bytes a second buffer.
        let mut buff_1: heapless::Vec<u8, L> = heapless::Vec::new();
        let mut buff_2: heapless::Vec<u8, L> = heapless::Vec::new();

        let mut req = rmodbus::client::ModbusRequest::new(self.unit_id, rmodbus::ModbusProto::Rtu);
        req.generate_set_holdings_bulk(start_register, data, &mut buff_1)?;

        self.interface
            .write_all(&buff_1)
            .map_err(crate::error::Error::SerialError)?;

        // Read the response - keep reading until we get WouldBlock or have enough data
        let mut temp_buf = [0u8; 8]; // Temporary buffer for single reads
        loop {
            match self.interface.read(&mut temp_buf) {
                Ok(bytes_read) => {
                    // Add the read bytes to our buffer
                    if buff_2.extend_from_slice(&temp_buf[0..bytes_read]).is_err() {
                        return Err(crate::error::Error::BufferError);
                    }
                    // Check if we have enough data for a minimal response
                    if buff_2.len() >= 8 {
                        break;
                    }
                }
                Err(e) => {
                    // If WouldBlock and we have some data, break and try to parse
                    if matches!(
                        e.kind(),
                        embedded_io::ErrorKind::Other | embedded_io::ErrorKind::TimedOut
                    ) && !buff_2.is_empty()
                    {
                        break;
                    }
                    // Other errors should be propagated
                    return Err(crate::error::Error::SerialError(e));
                }
            }
        }
        // @TODO Check CRC?
        if buff_1.as_slice()[0..=5] != buff_2.as_slice()[0..=5] {
            // First 6 bytes of message sent should match.
            Err(crate::error::Error::InvalidResponse)
        } else {
            Ok(())
        }
    }

    /// Read a single register from the PSU.
    pub fn read_modbus_single(&mut self, register: impl Into<u16>) -> Result<u16, S::Error> {
        let mut buff: heapless::Vec<u8, L> = heapless::Vec::new();
        let mut req = rmodbus::client::ModbusRequest::new(self.unit_id, rmodbus::ModbusProto::Rtu);

        // @TODO check that 1 is one register, not one byte?
        req.generate_get_holdings(register.into(), 1, &mut buff)?;

        self.interface
            .write_all(&buff)
            .map_err(crate::error::Error::SerialError)?;

        // Reuse same buffer when reading back
        buff.clear();

        // Read the response - keep reading until we get WouldBlock or have enough data
        let mut temp_buf = [0u8; 8]; // Temporary buffer for single reads
        loop {
            match self.interface.read(&mut temp_buf) {
                Ok(bytes_read) => {
                    // Add the read bytes to our buffer
                    if buff.extend_from_slice(&temp_buf[0..bytes_read]).is_err() {
                        return Err(crate::error::Error::BufferError);
                    }
                    // Check if we have enough data for a minimal response (unit_id + function + byte_count + at least 2 data bytes + 2 CRC)
                    if buff.len() >= 8 {
                        break;
                    }
                }
                Err(e) => {
                    // If WouldBlock and we have some data, break and try to parse
                    if matches!(
                        e.kind(),
                        embedded_io::ErrorKind::Other | embedded_io::ErrorKind::TimedOut
                    ) && !buff.is_empty()
                    {
                        break;
                    }
                    // Other errors should be propagated
                    return Err(crate::error::Error::SerialError(e));
                }
            }
        }

        // Parse the response using rmodbus
        let mut parsed_data: heapless::Vec<u16, 64> = heapless::Vec::new();
        req.parse_u16(&buff, &mut parsed_data)
            .map_err(|_| crate::error::Error::InvalidResponse)?;

        // Return the first register value
        parsed_data
            .first()
            .copied()
            .ok_or(crate::error::Error::InvalidResponse)
    }

    /// Read multiple registers from the PSU.
    ///
    /// Returns a vector of u16 values representing the register contents.
    fn read_modbus_bulk(
        &mut self,
        start_register: u16,
        count: u16,
    ) -> Result<heapless::Vec<u16, 64>, S::Error> {
        let mut buff: heapless::Vec<u8, L> = heapless::Vec::new();
        let mut req = rmodbus::client::ModbusRequest::new(self.unit_id, rmodbus::ModbusProto::Rtu);

        req.generate_get_holdings(start_register, count, &mut buff)?;

        self.interface
            .write_all(&buff)
            .map_err(crate::error::Error::SerialError)?;

        // Reuse same buffer when reading back
        buff.clear();

        // Read the response - keep reading until we get WouldBlock or have enough data
        let expected_response_size = 5 + (count as usize * 2) + 2; // unit_id + func + byte_count + data + CRC
        let mut temp_buf = [0u8; 64];
        loop {
            match self.interface.read(&mut temp_buf) {
                Ok(bytes_read) => {
                    if buff.extend_from_slice(&temp_buf[0..bytes_read]).is_err() {
                        return Err(crate::error::Error::BufferError);
                    }
                    if buff.len() >= expected_response_size {
                        break;
                    }
                }
                Err(e) => {
                    if matches!(
                        e.kind(),
                        embedded_io::ErrorKind::Other | embedded_io::ErrorKind::TimedOut
                    ) && !buff.is_empty()
                    {
                        break;
                    }
                    return Err(crate::error::Error::SerialError(e));
                }
            }
        }

        // Parse the response using rmodbus
        let mut parsed_data: heapless::Vec<u16, 64> = heapless::Vec::new();
        req.parse_u16(&buff, &mut parsed_data)
            .map_err(|_| crate::error::Error::InvalidResponse)?;

        Ok(parsed_data)
    }

    /// Get the current protection configuration from the active preset.
    ///
    /// This reads the protection settings from the currently active preset group
    /// and returns them as a `ProtectionConfig` struct.
    ///
    /// Requires known scaling factors for the PSU model. Returns `ScalingNotAvailable`
    /// error if the model's scaling factors are unknown.
    ///
    /// # For Unknown Models
    ///
    /// If your PSU model has unknown scaling factors, use [`set_scaling_factors`](Self::set_scaling_factors)
    /// to manually specify them before calling this method:
    ///
    /// ```ignore
    /// // Set custom scaling factors for unknown model
    /// let scaling = ScalingFactors::new(10, 10, 100, 10, 10);
    /// psu.set_scaling_factors(scaling);
    ///
    /// // Now get_protections will use your custom scaling
    /// let protections = psu.get_protections()?;
    /// ```
    pub fn get_protections(&mut self) -> Result<ProtectionConfig, S::Error> {
        // Ensure scaling factors are loaded
        let scaling = self.ensure_scaling()?;
        use crate::preset::XyPresetOffsets as XPO;

        /// Helper function to calculate array index relative to the starting register (SLvp)
        #[inline]
        fn idx(register: XPO) -> usize {
            (register as usize) - (XPO::SLvp as usize)
        }

        // Get currently active preset group
        let group = self.get_active_preset()?;

        // Calculate the starting address for protection registers
        let start_address = XPO::SLvp.address_in_group(group);

        // Read all protection-related registers (SLvp through SEtp)
        // That's registers 0x02 through 0x0E in the preset group (13 registers)
        let registers = self.read_modbus_bulk(start_address, 13)?;

        let temp_unit = self.get_temperature_unit()?;

        // Parse the registers into a ProtectionConfig
        // Register layout (from XyPresetOffsets):
        // We read starting from SLvp, so indices are relative to SLvp (not VSet)
        // 0: SLvp (under voltage) = XPO::SLvp - XPO::SLvp = 0
        // 1: SOvp (over voltage) = XPO::SOvp - XPO::SLvp = 1
        // 2: SOcp (over current) = XPO::SOcp - XPO::SLvp = 2
        // 3: SOpp (over power) = XPO::SOpp - XPO::SLvp = 3
        // 4: SOhpH (over time hours) = XPO::SOhpH - XPO::SLvp = 4
        // 5: SoHpM (over time minutes) = XPO::SoHpM - XPO::SLvp = 5
        // 6: SOahL (over capacity low) = XPO::SOahL - XPO::SLvp = 6
        // 7: SOahH (over capacity high) = XPO::SOahH - XPO::SLvp = 7
        // 8: SOwhL (over energy low) = XPO::SOwhL - XPO::SLvp = 8
        // 9: SOwhH (over energy high) = XPO::SOwhH - XPO::SLvp = 9
        // 10: SOtp (over temperature) = XPO::SOtp - XPO::SLvp = 10
        // 11: SIni (output enable - skip) = XPO::SIni - XPO::SLvp = 11
        // 12: SEtp (external temperature) = XPO::SEtp - XPO::SLvp = 12

        let under_voltage_mv = scaling.raw_to_voltage_mv(registers[idx(XPO::SLvp)]);
        let over_voltage_mv = scaling.raw_to_voltage_mv(registers[idx(XPO::SOvp)]);
        let over_current_ma = scaling.raw_to_current_ma(registers[idx(XPO::SOcp)]);
        let over_power_mw = scaling.raw_to_power_mw(registers[idx(XPO::SOpp)]);
        let over_time = Duration::<u32, 1, 1>::hours(registers[idx(XPO::SOhpH)] as u32)
            + Duration::<u32, 1, 1>::minutes(registers[idx(XPO::SoHpM)] as u32);
        let over_capacity_mah = ((registers[idx(XPO::SOahL)] as u32)
            | ((registers[idx(XPO::SOahH)] as u32) << 16))
            * scaling.capacity_divisor;
        let over_energy_mwh = ((registers[idx(XPO::SOwhL)] as u32)
            | ((registers[idx(XPO::SOwhH)] as u32) << 16))
            * scaling.energy_divisor;
        let over_temperature = Temperature::new(registers[idx(XPO::SOtp)], temp_unit);

        Ok(ProtectionConfig {
            under_voltage_mv,
            over_voltage_mv,
            over_current_ma,
            over_power_mw,
            over_time,
            over_capacity_mah,
            over_energy_mwh,
            over_temperature,
        })
    }

    /// Set protection levels of the power supply.
    ///
    /// Requires known scaling factors for the PSU model. Returns `ScalingNotAvailable`
    /// error if the model's scaling factors are unknown.
    ///
    /// # For Unknown Models
    ///
    /// If your PSU model has unknown scaling factors, use [`set_scaling_factors`](Self::set_scaling_factors)
    /// to manually specify them before calling this method:
    ///
    /// ```ignore
    /// // Set custom scaling factors for unknown model
    /// let scaling = ScalingFactors::new(10, 10, 100, 10, 10);
    /// psu.set_scaling_factors(scaling);
    ///
    /// // Now set_protections will use your custom scaling
    /// psu.set_protections(protection_config)?;
    /// ```
    ///
    /// __Note:__ This works by modifying the active preset group. This
    /// could cause unintended modifications to preset groups if not careful.
    pub fn set_protections(
        &mut self,
        protection_settings: ProtectionConfig,
    ) -> Result<(), S::Error> {
        // Ensure scaling factors are loaded
        let scaling = self.ensure_scaling()?;
        // Get currently active preset group so we can write values to the active group.
        let group = self.get_active_preset()?;

        // Get current voltage and current settings (read raw and convert using scaling)
        let set_voltage_raw = self.read_modbus_single(XyRegister::VSet)?;
        let set_current_raw = self.read_modbus_single(XyRegister::ISet)?;

        let set_voltage = scaling.raw_to_voltage_mv(set_voltage_raw);
        let set_current = scaling.raw_to_current_ma(set_current_raw);

        // Get current output state
        let set_output_state = self.read_modbus_single(XyRegister::OnOff)?;

        let preset = XyPresetBuilder::new(group, set_voltage, set_current)
            .with_protections(protection_settings)
            .with_output(set_output_state != 0)
            .build()
            .unwrap();

        // Get temperature unit for writing
        let temp_unit = self.get_temperature_unit()?;
        let (start_address, write_buffer) =
            preset.generate_write_data_and_offset(temp_unit, scaling);

        self.write_modbus_bulk(start_address, write_buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_serial::MockSerial;

    #[test]
    fn test_write_modbus_single() {
        let mut mock_serial = MockSerial::new();
        let ideal_written = [0x01, 0x06, 0x00, 0x10, 0x12, 0x34, 0x85, 0x78];
        mock_serial.set_read_data(ideal_written.as_slice()).unwrap();

        let mut psu: XyPsu<MockSerial, 128> = XyPsu::new(mock_serial, 0x01);

        // Test writing to register 0x10 with value 0x1234
        let result = psu.write_modbus_single(0x10 as u16, 0x1234u16);
        assert!(result.is_ok());

        // Check that the correct Modbus RTU frame was written
        let written_data = psu.interface.written_data();
        assert!(!written_data.is_empty());
        assert_eq!(written_data, ideal_written.as_slice());
        assert_eq!(written_data.len(), 8); // Total frame length
    }

    #[test]
    fn test_read_modbus_single_bad_crc() {
        let mut mock_serial = MockSerial::new();

        // Set up a proper Modbus RTU response for reading register 0x20 with value 0x5678
        // Create a response manually: unit_id(1) + function(1) + byte_count(1) + data(2) + crc(2) = 7 bytes
        let response_data = [0x01, 0x03, 0x02, 0x56, 0x78, 0x00, 0x00]; // CRC will be wrong but that's ok for this test
        mock_serial.set_read_data(&response_data).unwrap();

        let mut psu: XyPsu<MockSerial, 128> = XyPsu::new(mock_serial, 0x01);

        let result = psu.read_modbus_single(0x20 as u16);

        // Check that the request was written correctly
        let written_data = psu.interface.written_data();
        assert!(!written_data.is_empty());

        // Expected request frame for reading register 0x0020, count 1
        assert_eq!(written_data[0], 0x01); // Unit ID
        assert_eq!(written_data[1], 0x03); // Function code for read holding registers
        assert_eq!(written_data[2], 0x00); // Register high byte
        assert_eq!(written_data[3], 0x20); // Register low byte
        assert_eq!(written_data[4], 0x00); // Count high byte
        assert_eq!(written_data[5], 0x01); // Count low byte
        // CRC bytes are at positions 6 and 7
        assert_eq!(written_data.len(), 8); // Total frame length

        // The result might be an error due to invalid CRC, but at least it shouldn't panic
        // If we get an error, it should be InvalidResponse due to CRC mismatch
        match result {
            Ok(value) => {
                // If parsing somehow succeeds, verify we got the expected value
                assert_eq!(value, 0x5678);
            }
            Err(crate::error::Error::InvalidResponse) => {
                // This is expected due to invalid CRC in our test data
            }
            Err(other) => {
                panic!("Unexpected error: {:?}", other);
            }
        }
    }

    #[test]
    fn test_read_modbus_single() {
        let mut mock_serial = MockSerial::new();

        // Set up a proper Modbus RTU response for reading register 0x20 with value 0x5678
        // Create a response manually: unit_id(1) + function(1) + byte_count(1) + data(2) + crc(2) = 7 bytes
        let response_data = [0x01, 0x03, 0x02, 0x56, 0x78, 0x87, 0xC6]; // CRC calculated using: https://homepages.plus.net/dougrice/dev/modbus/crc.html
        mock_serial.set_read_data(&response_data).unwrap();

        let mut psu: XyPsu<MockSerial, 128> = XyPsu::new(mock_serial, 0x01);

        let result = psu.read_modbus_single(0x20 as u16);

        // Check that the request was written correctly
        let written_data = psu.interface.written_data();
        assert!(!written_data.is_empty());

        // Expected request frame for reading register 0x0020, count 1
        assert_eq!(written_data[0], 0x01); // Unit ID
        assert_eq!(written_data[1], 0x03); // Function code for read holding registers
        assert_eq!(written_data[2], 0x00); // Register high byte
        assert_eq!(written_data[3], 0x20); // Register low byte
        assert_eq!(written_data[4], 0x00); // Count high byte
        assert_eq!(written_data[5], 0x01); // Count low byte
        // CRC bytes are at positions 6 and 7
        assert_eq!(written_data[6], 0x85);
        assert_eq!(written_data[7], 0xC0);
        assert_eq!(written_data.len(), 8); // Total frame length

        match result {
            Ok(value) => {
                // If parsing somehow succeeds, verify we got the expected value
                assert_eq!(value, 0x5678);
            }
            Err(err) => {
                panic!("Unexpected error: {:?}", err);
            }
        }
    }

    #[test]
    fn test_read_output_voltage() {
        let mut mock_serial = MockSerial::new();

        // Set up a proper Modbus RTU response for reading register 0x02 (VOut) with value 500 (0x01F4)
        // Response format: unit_id(1) + function(1) + byte_count(1) + data(2) + crc(2) = 7 bytes
        // Value 500 = 0x01F4 in big-endian
        let response_data = [0x01, 0x03, 0x02, 0x01, 0xF4, 0xB8, 0x53]; // CRC calculated using: https://homepages.plus.net/dougrice/dev/modbus/crc.html
        mock_serial.set_read_data(&response_data).unwrap();

        let mut psu: XyPsu<MockSerial, 128> = XyPsu::new(mock_serial, 0x01);

        // Read the raw VOut register to test modbus communication without scaling
        let result = psu.read_modbus_single(XyRegister::VOut);

        match result {
            Ok(value) => {
                // Raw value should be 500 (0x01F4)
                assert_eq!(value, 500);
            }
            Err(err) => {
                panic!("Unexpected error: {:?}", err);
            }
        }
    }

    #[test]
    fn test_write_output_voltage() {
        let mut mock_serial = MockSerial::new();

        // Mock the write response for setting voltage
        let write_response = [0x01, 0x06, 0x00, 0x00, 0x09, 0x60, 0x8F, 0xB2];
        mock_serial.set_read_data(&write_response).unwrap();

        let mut psu: XyPsu<MockSerial, 128> = XyPsu::new(mock_serial, 0x01);

        // Test writing raw value 2400 (0x0960) which represents 24.0V in centivolts
        // Using direct modbus write to test communication without scaling
        let result = psu.write_modbus_single(XyRegister::VSet, 2400u16);
        assert!(result.is_ok(), "Setting voltage should succeed");
    }
}
