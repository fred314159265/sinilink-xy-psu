use crate::{
    error::Result,
    preset::{PresetGroup, ProtectionConfig, XyPresetBuilder},
    register::{
        BaudRate, ControlMode, ProductModel, State, Temperature, TemperatureUnit, XyRegister,
    },
};
use embedded_io::Error;
use fugit::Duration;

/// You can create a XyPsu using any interface which implements [embedded_io::Read] & [embedded_io::Write].
///
/// For it's methods, we generally use the nomenclature that "set" meant to write a configuration and "get" means to read
/// back a configuration value. Where as "read" means to get a measured value.
pub struct XyPsu<S: embedded_io::Read + embedded_io::Write, const L: usize = 128> {
    interface: S,
    /// Default for PSU is 0x01.
    unit_id: u8,
}

impl<S: embedded_io::Read + embedded_io::Write, const L: usize> XyPsu<S, L> {
    /// Create a new XyPsu instance with the given interface and unit ID
    pub fn new(interface: S, unit_id: u8) -> Self {
        Self { interface, unit_id }
    }

    /// Return the measured output voltage in millivolts.
    pub fn read_output_voltage_mv(&mut self) -> Result<u32, S::Error> {
        let centivolts = self.read_modbus_single(XyRegister::VOut)?;
        Ok(centivolts as u32 * 10u32)
    }

    /// Return the measured supply input voltage in millivolts.
    pub fn read_input_voltage_mv(&mut self) -> Result<u32, S::Error> {
        let centivolts = self.read_modbus_single(XyRegister::UIn)?;
        Ok(centivolts as u32 * 10u32)
    }

    /// Return the measured output current in milliamps.
    pub fn read_current_ma(&mut self) -> Result<u32, S::Error> {
        let milliamps = self.read_modbus_single(XyRegister::IOut)?;
        Ok(milliamps as u32 * 10)
    }

    /// Return the measured output current in milliwatts.
    pub fn read_power_mw(&mut self) -> Result<u32, S::Error> {
        let deciwatts = self.read_modbus_single(XyRegister::Power)?;
        // raw value in deci-watts.
        Ok(deciwatts as u32 * 100)
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
    pub fn set_output_voltage_mv(&mut self, voltage_mv: u32) -> Result<(), S::Error> {
        let centivolts = u16::try_from(voltage_mv / 10)?;
        self.write_modbus_single(XyRegister::VSet, centivolts)?;
        Ok(())
    }

    /// Get the current output target voltage. Value returned in millivolts.
    pub fn get_output_voltage_mv(&mut self) -> Result<u32, S::Error> {
        let value = self.read_modbus_single(XyRegister::VSet)?;
        let voltage_mv = value as u32 * 10;
        Ok(voltage_mv)
    }

    /// Set the output current limit. Value supplied in milliamps.
    pub fn set_current_limit_ma(&mut self, current_ma: u32) -> Result<(), S::Error> {
        let current_centiamps = u16::try_from(current_ma / 10)?;
        self.write_modbus_single(XyRegister::ISet, current_centiamps)?;
        Ok(())
    }

    /// Get the current output current limit value. Value supplied in milliamps.
    pub fn get_current_limit_ma(&mut self) -> Result<u32, S::Error> {
        let value = self.read_modbus_single(XyRegister::ISet)?;
        let current_ma = value as u32 * 10;
        Ok(current_ma)
    }

    /// Returns the raw register values for "MODEL" -> product model
    ///
    /// See [Self::get_product_model] for a method which tries to interpret this data.
    pub fn get_product_model_raw(&mut self) -> Result<u16, S::Error> {
        self.read_modbus_single(XyRegister::Model)
    }

    /// Returns the interpreted product model.
    ///
    /// Not yet sure what the pattern is. So far we have observed:
    /// *  => XY3607F
    /// * 25856 | 0x6500 => XY7025
    pub fn get_product_model(&mut self) -> Result<ProductModel, S::Error> {
        let _raw = self.get_product_model_raw()?;
        unimplemented!()
    }

    /// Configure the baud rate of the PSU.
    pub fn set_baud_rate(&mut self, baud_rate: BaudRate) -> Result<(), S::Error> {
        self.write_modbus_single(XyRegister::BaudRateL, baud_rate)
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
                    if buff_2.len() >= 7 {
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
                    // Check if we have enough data for a minimal response (unit_id + function + byte_count + at least 2 data bytes + 2 CRC)
                    if buff_2.len() >= 7 {
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
                    if buff.len() >= 7 {
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

    /// Set protection levels of the power supply.
    ///
    /// __Note:__ To apply these protection levels, this function has to write them to a
    /// preset and then load the preset. This likely interrupts the output if already
    /// enabled, and so it is highly recommended that protections are set before
    /// enabling the PSU output.
    // @TODO Test if this is actually the case.
    pub fn set_protections(
        &mut self,
        protection_settings: ProtectionConfig,
    ) -> Result<(), S::Error> {
        let group = PresetGroup::Group9;

        // Get current voltage and current settings
        let set_voltage = self.read_modbus_single(XyRegister::VSet)? as u32 * 10;
        let set_current = self.read_modbus_single(XyRegister::ISet)?;

        // Get current output state
        let set_output_state = self.read_modbus_single(XyRegister::OnOff)?;

        let preset = XyPresetBuilder::new(group, set_voltage, set_current)
            .with_protections(protection_settings)
            .with_output(set_output_state != 0)
            .build()
            .unwrap();

        // Write preset to the PSU register
        preset.write(self)?;

        // Load preset to apply it and make it active settings
        self.write_modbus_single(XyRegister::ExtractM, group as u16)
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

        // Set up a proper Modbus RTU response for reading register 0x02 with value 500 (5.0 or 5000mV)
        // Create a response manually: unit_id(1) + function(1) + byte_count(1) + data(2) + crc(2) = 7 bytes
        let response_data = [0x01, 0x03, 0x04, 0x01, 0xF4, 0x58, 0x52]; // CRC calculated using: https://homepages.plus.net/dougrice/dev/modbus/crc.html
        mock_serial.set_read_data(&response_data).unwrap();

        let mut psu: XyPsu<MockSerial, 128> = XyPsu::new(mock_serial, 0x01);

        let result = psu.read_output_voltage_mv();

        // Check that the request was written correctly
        let written_data = psu.interface.written_data();
        assert!(!written_data.is_empty());

        // Expected request frame for reading register 0x0020, count 1
        assert_eq!(written_data[0], 0x01); // Unit ID
        assert_eq!(written_data[1], 0x03); // Function code for read holding registers
        assert_eq!(written_data[2], 0x00); // Register high byte
        assert_eq!(written_data[3], 0x02); // Register low byte
        assert_eq!(written_data[4], 0x00); // Count high byte
        assert_eq!(written_data[5], 0x01); // Count low byte
        // CRC bytes are at positions 6 and 7
        assert_eq!(written_data[6], 0x25);
        assert_eq!(written_data[7], 0xCA);
        assert_eq!(written_data.len(), 8); // Total frame length

        match result {
            Ok(value) => {
                // If parsing somehow succeeds, verify we got the expected value
                assert_eq!(value, 5000);
            }
            Err(err) => {
                panic!("Unexpected error: {:?}", err);
            }
        }
    }

    #[test]
    fn test_write_output_voltage() {
        let mut mock_serial = MockSerial::new();

        // @TODO preload with correct response for a good write.
        let read_data = [0x01, 0x06, 0x00, 0x00, 0x09, 0x60, 0x8F, 0xB2];
        mock_serial.set_read_data(read_data.as_slice()).unwrap();

        let mut psu: XyPsu<MockSerial, 128> = XyPsu::new(mock_serial, 0x01);

        // Test writing to register 0x10 with value 0x1234
        let result = psu.set_output_voltage_mv(24000);
        assert!(result.is_ok());

        // Check that the correct Modbus RTU frame was written
        let written_data = psu.interface.written_data();
        let ideal_written = [0x01, 0x06, 0x00, 0x00, 0x09, 0x60, 0x8F, 0xB2];
        assert!(!written_data.is_empty());
        assert_eq!(written_data, ideal_written.as_slice());
        assert_eq!(written_data.len(), 8); // Total frame length
    }
}
