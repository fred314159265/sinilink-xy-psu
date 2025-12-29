//! Scaling factors for different PSU models
//!
//! Different Sinilink XY-PSU models use different scaling factors for voltage, current,
//! and power measurements. This module defines the scaling factors for each known model.

use crate::register::ProductModel;

/// Scaling factors for converting raw register values to standard units
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScalingFactors {
    /// Multiplier for voltage values (e.g., 10 means raw value is in centivolts, multiply by 10 to get mV)
    pub voltage_divisor: u32,
    /// Multiplier for current values (e.g., 10 means raw value is in units of 10mA, multiply by 10 to get mA)
    pub current_divisor: u32,
    /// Multiplier for power values (e.g., 100 means raw value is in deciwatts, multiply by 100 to get mW)
    pub power_divisor: u32,
    /// Multiplier for capacity values (e.g., 10 means raw value is in units of 10mAh, multiply by 10 to get mAh)
    /// Defaults to current_divisor if not specified
    pub capacity_divisor: u32,
    /// Multiplier for energy values (e.g., 100 means raw value is in units of 100mWh, multiply by 100 to get mWh)
    /// Defaults to power_divisor/10 if not specified
    pub energy_divisor: u32,
}

impl Default for ScalingFactors {
    /// Default to no scaling.
    fn default() -> Self {
        Self {
            voltage_divisor: 1,
            current_divisor: 1,
            power_divisor: 1,
            capacity_divisor: 1,
            energy_divisor: 1,
        }
    }
}

impl ScalingFactors {
    /// Create a new `ScalingFactors` instance with the specified divisor values.
    ///
    /// # Arguments
    ///
    /// * `voltage_divisor` - Multiplier for voltage values (raw to mV).
    /// * `current_divisor` - Multiplier for current values (raw to mA).
    /// * `power_divisor` - Multiplier for power values (raw to mW).
    /// * `capacity_divisor` - Multiplier for capacity values (raw to mAh).
    /// * `energy_divisor` - Multiplier for energy values (raw to mWh).
    pub const fn new(
        voltage_divisor: u32,
        current_divisor: u32,
        power_divisor: u32,
        capacity_divisor: u32,
        energy_divisor: u32,
    ) -> Self {
        Self {
            voltage_divisor,
            current_divisor,
            power_divisor,
            capacity_divisor,
            energy_divisor,
        }
    }

    /// Convert raw voltage register value to millivolts
    ///
    /// If divisor is 10, raw is in centivolts (10mV units), so we multiply by 10.
    #[inline]
    pub const fn raw_to_voltage_mv(&self, raw: u16) -> u32 {
        (raw as u32) * self.voltage_divisor
    }

    /// Convert millivolts to raw voltage register value
    #[inline]
    pub const fn voltage_mv_to_raw(&self, voltage_mv: u32) -> u16 {
        (voltage_mv / self.voltage_divisor) as u16
    }

    /// Convert raw current register value to milliamps
    ///
    /// If divisor is 10, raw is in units of 10mA, so we multiply by 10.
    #[inline]
    pub const fn raw_to_current_ma(&self, raw: u16) -> u32 {
        (raw as u32) * self.current_divisor
    }

    /// Convert milliamps to raw current register value
    #[inline]
    pub const fn current_ma_to_raw(&self, current_ma: u32) -> u16 {
        (current_ma / self.current_divisor) as u16
    }

    /// Convert raw power register value to milliwatts
    ///
    /// If divisor is 100, raw is in units of 100mW (deciwatts), so we multiply by 100.
    #[inline]
    pub const fn raw_to_power_mw(&self, raw: u16) -> u32 {
        (raw as u32) * self.power_divisor
    }

    /// Convert milliwatts to raw power register value
    #[inline]
    pub const fn power_mw_to_raw(&self, power_mw: u32) -> u16 {
        (power_mw / self.power_divisor) as u16
    }
}

impl ProductModel {
    /// Get scaling factors for this product model
    ///
    /// Returns `Some(ScalingFactors)` for models with known scaling factors,
    /// or `None` for models where the scaling factors have not yet been confirmed.
    ///
    /// # Known Models
    /// The following models have confirmed scaling factors:
    /// - XY3607F
    /// - XY7025
    /// - XY12522
    /// - XY6020L
    ///
    /// # For Unknown Models
    ///
    /// If this method returns `None` for your model, you can use [`XyPsu::set_scaling_factors`] 
    /// to manually specify scaling factors, and then use the normal scaled methods as normal.
    pub const fn scaling_factors(&self) -> Option<ScalingFactors> {
        match self {
            // These scaling factors have been checked.
            ProductModel::XY3607F => Some(ScalingFactors {
                voltage_divisor: 10,
                current_divisor: 1,
                power_divisor: 100,
                capacity_divisor: 1, // Same as current_divisor
                energy_divisor: 10,  // power_divisor / 10
            }),
            // These scaling factors have been checked.
            ProductModel::XY7025 => Some(ScalingFactors {
                voltage_divisor: 10,
                current_divisor: 10,
                power_divisor: 1000,
                capacity_divisor: 10, // Same as current_divisor
                energy_divisor: 100,  // power_divisor / 10
            }),
            // @TODO: Verify these are correct
            ProductModel::XY12522 => Some(ScalingFactors {
                voltage_divisor: 10,
                current_divisor: 10,
                power_divisor: 1000,
                capacity_divisor: 10, // Same as current_divisor
                energy_divisor: 100,  // power_divisor / 10
            }),
            // @TODO: Verify these are correct
            ProductModel::XY6020L => Some(ScalingFactors {
                voltage_divisor: 10,
                current_divisor: 10,
                power_divisor: 1000,
                capacity_divisor: 10, // Same as current_divisor
                energy_divisor: 100,  // power_divisor / 10
            }),
            // Unconfirmed models - return None to force users to use raw functions
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voltage_scaling() {
        let scaling = ScalingFactors {
            voltage_divisor: 10,
            current_divisor: 10,
            power_divisor: 100,
            capacity_divisor: 10,
            energy_divisor: 10,
        };

        // Raw value 1234 centvolts = 12340 mV
        assert_eq!(scaling.raw_to_voltage_mv(1234), 12340);
        // 12340 mV should convert back to 1234 raw
        assert_eq!(scaling.voltage_mv_to_raw(12340), 1234);
    }

    #[test]
    fn test_current_scaling() {
        let scaling = ScalingFactors {
            voltage_divisor: 10,
            current_divisor: 10,
            power_divisor: 100,
            capacity_divisor: 10,
            energy_divisor: 10,
        };

        // Raw value 500 (units of 10mA) = 5000 mA
        assert_eq!(scaling.raw_to_current_ma(500), 5000);
        // 5000 mA should convert back to 500 raw
        assert_eq!(scaling.current_ma_to_raw(5000), 500);
    }

    #[test]
    fn test_power_scaling() {
        let scaling = ScalingFactors {
            voltage_divisor: 10,
            current_divisor: 10,
            power_divisor: 100,
            capacity_divisor: 10,
            energy_divisor: 10,
        };

        // Raw value 123 (units of 100mW = deciwatts) = 12300 mW
        assert_eq!(scaling.raw_to_power_mw(123), 12300);
    }

    #[test]
    fn test_known_models_have_scaling() {
        assert!(ProductModel::XY3607F.scaling_factors().is_some());
        assert!(ProductModel::XY7025.scaling_factors().is_some());
        assert!(ProductModel::XY12522.scaling_factors().is_some());
        assert!(ProductModel::XY6020L.scaling_factors().is_some());
    }

    #[test]
    fn test_unknown_models_no_scaling() {
        assert!(ProductModel::XY6506.scaling_factors().is_none());
        assert!(ProductModel::XY6509.scaling_factors().is_none());
    }
}
