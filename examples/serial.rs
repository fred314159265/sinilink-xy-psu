use std::env;

use inquire::Select;
use serialport::SerialPort;
use sinilink_xy_psu::psu::XyPsu;

// Configuration constants - adjust these for your setup
const BAUD_RATE: u32 = 115200;
// The PSU can take a while to respond, a reasonably large time out is required.
const SERIAL_TIMEOUT_MS: u64 = 300;
const MODBUS_UNIT_ID: u8 = 0x01;
const OUTPUT_VOLTAGE_MV: u32 = 5500; // 5V
const CURRENT_LIMIT_MA: u32 = 100; // 0.1A
const STABILIZATION_DELAY_MS: u64 = 1000;

pub struct PortWrapper(Box<dyn SerialPort>);

#[derive(Debug)]
pub struct IoError(std::io::Error);

impl core::fmt::Display for IoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for IoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl embedded_io::Error for IoError {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self.0.kind() {
            std::io::ErrorKind::NotFound => embedded_io::ErrorKind::NotFound,
            std::io::ErrorKind::PermissionDenied => embedded_io::ErrorKind::PermissionDenied,
            std::io::ErrorKind::ConnectionRefused => embedded_io::ErrorKind::ConnectionRefused,
            std::io::ErrorKind::ConnectionReset => embedded_io::ErrorKind::ConnectionReset,
            std::io::ErrorKind::ConnectionAborted => embedded_io::ErrorKind::ConnectionAborted,
            std::io::ErrorKind::NotConnected => embedded_io::ErrorKind::NotConnected,
            std::io::ErrorKind::AddrInUse => embedded_io::ErrorKind::AddrInUse,
            std::io::ErrorKind::AddrNotAvailable => embedded_io::ErrorKind::AddrNotAvailable,
            std::io::ErrorKind::BrokenPipe => embedded_io::ErrorKind::BrokenPipe,
            std::io::ErrorKind::AlreadyExists => embedded_io::ErrorKind::AlreadyExists,
            std::io::ErrorKind::InvalidInput => embedded_io::ErrorKind::InvalidInput,
            std::io::ErrorKind::InvalidData => embedded_io::ErrorKind::InvalidData,
            std::io::ErrorKind::TimedOut => embedded_io::ErrorKind::TimedOut,
            std::io::ErrorKind::Interrupted => embedded_io::ErrorKind::Interrupted,
            std::io::ErrorKind::Unsupported => embedded_io::ErrorKind::Unsupported,
            std::io::ErrorKind::OutOfMemory => embedded_io::ErrorKind::OutOfMemory,
            _ => embedded_io::ErrorKind::Other,
        }
    }
}

impl embedded_io::ErrorType for PortWrapper {
    type Error = IoError;
}

impl embedded_io::Read for PortWrapper {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        std::io::Read::read(&mut self.0, buf).map_err(IoError)
    }
}

impl embedded_io::Write for PortWrapper {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        std::io::Write::write(&mut self.0, buf).map_err(IoError)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        std::io::Write::flush(&mut self.0).map_err(IoError)
    }
}

fn main() {
    // Get serial port from command line arg or interactive selection
    let port_name = env::args().nth(1).unwrap_or_else(|| {
        // List available serial ports
        let ports = serialport::available_ports().expect("Failed to enumerate serial ports");

        if ports.is_empty() {
            eprintln!("No serial ports found!");
            std::process::exit(1);
        }

        let port_names: Vec<String> = ports.iter().map(|p| p.port_name.clone()).collect();

        // Interactive selection
        Select::new("Select a serial port:", port_names)
            .prompt()
            .expect("Failed to select port")
    });

    println!("Using port: {}", port_name);

    // Open serial port
    let port = serialport::new(&port_name, BAUD_RATE)
        .timeout(std::time::Duration::from_millis(SERIAL_TIMEOUT_MS))
        .open()
        .expect("Failed to open serial port");

    let port = PortWrapper(port);

    // Create a PSU object
    let mut psu: XyPsu<PortWrapper, 128> = XyPsu::new(port, MODBUS_UNIT_ID);

    // Get and display the product model
    let model_number = psu.get_product_model_raw().unwrap();
    println!("Product model: 0x{:04X} ({})", model_number, model_number);

    // Get and display the product model
    let model_number = psu.get_product_model();
    println!("Product model: {:#?}", model_number);

    // Set output voltage
    psu.set_output_voltage_mv(OUTPUT_VOLTAGE_MV).unwrap();
    println!(
        "Set output voltage to {}V",
        OUTPUT_VOLTAGE_MV as f32 / 1000.0
    );

    // Set current limit
    psu.set_current_limit_ma(CURRENT_LIMIT_MA).unwrap();
    println!("Set current limit to {}A", CURRENT_LIMIT_MA as f32 / 1000.0);

    // Enable the output
    psu.set_output_state(true).unwrap();
    println!("Output enabled");

    // Wait for output to stabilize
    std::thread::sleep(std::time::Duration::from_millis(STABILIZATION_DELAY_MS));

    // Measure and display the output voltage
    let measured_voltage = psu.read_output_voltage_mv().unwrap();
    println!(
        "Measured output voltage: {:.3}V",
        measured_voltage as f32 / 1000.0
    );

    // Read and display the protection configuration
    println!("\n--- Current Protection Configuration ---");
    let protections = psu.get_protections().unwrap();
    println!("{:#?}", protections);

    // Modify protection settings
    println!("\n--- Updating Protection Settings ---");
    use fugit::Duration;
    use sinilink_xy_psu::preset::ProtectionConfig;

    let new_protections = ProtectionConfig {
        under_voltage_mv: 11000,                         // 11.0V minimum
        over_voltage_mv: 15000,                         // 15.0V maximum
        over_current_ma: 123,                           // 0.123A maximum
        over_power_mw: 9876,                            // 9.876W maximum
        over_time: Duration::<u32, 1, 1>::hours(1),     // 1 hour maximum runtime
        over_capacity_mah: 1000,                        // 1000 mAh maximum
        over_energy_mwh: 5000,                          // 5 Wh maximum
        over_temperature: protections.over_temperature, // Keep existing temperature setting
    };

    println!("New protection configuration:");
    println!("{:#?}", new_protections);

    psu.set_protections(new_protections).unwrap();
    println!("\nProtection settings updated successfully!");

    // Verify the settings were applied
    println!("\n--- Verifying Updated Protection Configuration ---");
    let verified_protections = psu.get_protections().unwrap();
    println!("{:#?}", verified_protections);
}
