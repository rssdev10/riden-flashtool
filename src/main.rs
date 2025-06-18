//  Riden RD60xx Firmware Flash Tool
//
//  based on https://github.com/tjko/riden-flashtool/blob/main/flash-rd.py
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program. If not, see <http://www.gnu.org/licenses/>.

use clap::Parser;
use serialport::{SerialPort, SerialPortType};
use std::{
    fs::File,
    io::{self, Read, Write},
    thread,
    time::Duration,
};
use tabled::settings::Style;
use tabled::{Table, Tabled};

const SUPPORTED_MODELS: [u16; 7] = [
    60062, 60065, 60066, 60121, 60125, 60181, 60241
];
const DEFAULT_BAUD_RATE: u32 = 115_200;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Parser, Debug)]
#[clap(
    name = "Riden RD60xx Firmware Flash Tool",
    about = "Firmware updater for Riden power supplies.",
    version,
    long_about = None,
    after_help = "Repository: https://github.com/rssdev10/riden-flashtool"
)]
struct Args {
    /// List available serial ports and exit
    #[clap(short, long, action)]
    list: bool,

    /// Serial port device (e.g. /dev/ttyUSB0 or COM3)
    #[clap(required_unless_present = "list")]
    port: Option<String>,

    /// Firmware file to flash
    #[clap(required_unless_present = "list")]
    firmware: Option<String>,

    /// Enable verbose output
    #[clap(short, long)]
    verbose: bool,

    /// Serial port baud rate
    #[clap(short, long, default_value_t = DEFAULT_BAUD_RATE)]
    speed: u32,
}

struct RidenFirmwareUpdater {
    port: Box<dyn SerialPort>,
    verbose: bool,
}

fn list_serial_ports() -> io::Result<()> {
    println!("Available serial ports:");
    match serialport::available_ports() {
        Ok(ports) => {
            // Apply platform-specific filtering
            let filtered_ports = ports
                .into_iter()
                .filter(|port| {
                    #[cfg(not(windows))]
                    {
                        port.port_name.contains("/tty.")
                            || port.port_name.contains("/ttyUSB")
                            || port.port_name.contains("/ttyACM")
                    }
                    #[cfg(windows)]
                    {
                        true
                    }
                })
                .collect::<Vec<_>>();

            if filtered_ports.is_empty() {
                println!("  No ports found");
                return Ok(());
            }

            #[derive(Tabled)]
            struct Row {
                #[tabled(rename = "Port")]
                port: String,
                #[tabled(rename = "Type")]
                type_str: String,
                #[tabled(rename = "Info")]
                info: String,
            }

            let mut rows = Vec::new();
            for port in &filtered_ports {
                match &port.port_type {
                    SerialPortType::UsbPort(info) => {
                        let product = info
                            .product
                            .clone()
                            .unwrap_or_else(|| "Unknown".to_string());
                        rows.push(Row {
                            port: port.port_name.clone(),
                            type_str: "USB".to_string(),
                            info: format!("{} ({:04x}:{:04x})", product, info.vid, info.pid),
                        });
                    }
                    SerialPortType::BluetoothPort => {
                        rows.push(Row {
                            port: port.port_name.clone(),
                            type_str: "Bluetooth".to_string(),
                            info: "".to_string(),
                        });
                    }
                    SerialPortType::PciPort => {
                        rows.push(Row {
                            port: port.port_name.clone(),
                            type_str: "PCI".to_string(),
                            info: "".to_string(),
                        });
                    }
                    SerialPortType::Unknown => {
                        rows.push(Row {
                            port: port.port_name.clone(),
                            type_str: "Unknown".to_string(),
                            info: "".to_string(),
                        });
                    }
                }
            }

            let mut table = Table::new(rows);
            table.with(Style::psql());

            println!("{}", table);
            Ok(())
        }
        Err(e) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Error listing ports: {}", e),
        )),
    }
}

impl RidenFirmwareUpdater {
    fn new(port: Box<dyn SerialPort>, verbose: bool) -> Self {
        Self { port, verbose }
    }

    fn read_data(&mut self, count: usize) -> io::Result<Vec<u8>> {
        if self.verbose {
            println!("Waiting data...");
        }
        let mut buf = vec![0; count];
        let bytes_read = self.port.read(&mut buf)?;
        buf.truncate(bytes_read);
        if self.verbose {
            println!("Read: {}: {:?}", buf.len(), buf);
        }
        Ok(buf)
    }

    fn write_data(&mut self, data: &[u8]) -> io::Result<()> {
        if self.verbose {
            println!("Write: {}: {:?}", data.len(), data);
        }
        self.port.write_all(data)
    }

    fn update_firmware(&mut self, firmware: &[u8]) -> io::Result<()> {
        self.port.set_timeout(Duration::from_secs(5))?;
        self.write_data(b"upfirm\r\n")?;

        let res = self.read_data(6)?;
        if res != b"upredy" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to initiate flashing: {:?}", res),
            ));
        }

        print!("Updating firmware...");
        io::stdout().flush()?;

        for chunk in firmware.chunks(64) {
            self.write_data(chunk)?;
            let res = self.read_data(2)?;
            if res != b"OK" {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Flash failed: {:?}", res),
                ));
            }
            print!(".");
            io::stdout().flush()?;
        }
        println!();
        Ok(())
    }

    fn bootloader_mode(&mut self) -> io::Result<()> {
        println!("Check if device is in bootloader mode...");
        io::stdout().flush()?;

        self.write_data(b"queryd\r\n")?;
        let res = self.read_data(4)?;
        if res == b"boot" {
            println!("Yes");
            return Ok(());
        }
        println!("No");

        // Switch to Modbus communication
        self.port.set_timeout(Duration::from_secs(5))?;

        // Send Modbus "Read Holding Registers" command: device address 0x01, function 0x03,
        // starting at register 0x0000, read 4 registers, CRC 0x4409
        self.write_data(&[0x01, 0x03, 0x00, 0x00, 0x00, 0x04, 0x44, 0x09])?;

        let res = self.read_data(13)?;
        if res.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "No response from device",
            ));
        }

        // Validate Modbus response header
        match res.as_slice() {
            [0x01, 0x03, 0x08, ..] if res.len() == 13 => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid response: {:?}", res),
                ));
            }
        }

        let model = u16::from_be_bytes([res[3], res[4]]);
        let fwver = res[10] as f32 / 100.0;
        println!(
            "Found device via Modbus: RD{} ({}) v{:.2}",
            model / 10,
            model,
            fwver
        );

        println!("Rebooting into bootloader mode...");
        // Send Modbus "Write Single Register" command to reboot into bootloader:
        // device address 0x01, function 0x06, register 0x0100, value 0x1601, CRC 0x4796
        self.write_data(&[0x01, 0x06, 0x01, 0x00, 0x16, 0x01, 0x47, 0x96])?;

        let res = self.read_data(1)?;
        if res != [0xFC] {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to reboot device",
            ));
        }

        thread::sleep(Duration::from_secs(3));
        Ok(())
    }

    fn device_info(&mut self) -> io::Result<(u16, f32, u32)> {
        self.write_data(b"getinf\r\n")?;
        let res = self.read_data(13)?;

        if res.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "No response from bootloader",
            ));
        }

        if res.len() != 13 || &res[0..3] != b"inf" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid bootloader response: {:?}", res),
            ));
        }

        let snum = u32::from_be_bytes([res[6], res[5], res[4], res[3]]);
        let model = u16::from_be_bytes([res[8], res[7]]);
        let fwver = res[11] as f32 / 100.0;

        Ok((model, fwver, snum))
    }

    fn is_supported_model(model: u16) -> bool {
        SUPPORTED_MODELS.contains(&model)
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    if args.list {
        return list_serial_ports();
    }

    let port_name = args.port.expect("Port is required when not using --list");

    println!("Serial port: {} ({} bps)", port_name, args.speed);

    let port = serialport::new(&port_name, args.speed)
        .timeout(DEFAULT_TIMEOUT)
        .open()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let firmware = if let Some(path) = args.firmware {
        let mut file = File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        println!("Firmware image size: {} bytes", data.len());
        Some(data)
    } else {
        None
    };

    let mut updater = RidenFirmwareUpdater::new(port, args.verbose);
    if let Err(e) = updater.bootloader_mode() {
        eprintln!("Error entering bootloader mode: {}", e);
        return Err(e);
    }

    match updater.device_info() {
        Ok((model, fwver, snum)) => {
            println!("Device information from bootloader:");
            println!("    Model: RD{} ({})", model / 10, model);
            println!(" Firmware: v{:.2}", fwver);
            println!("      S/N: {:08}", snum);

            if !RidenFirmwareUpdater::is_supported_model(model) {
                let msg = format!("Unsupported device model: {}", model);
                eprintln!("{}", msg);
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    msg,
                ));
            }
        }
        Err(e) => {
            eprintln!("Error getting device info: {}", e);
            return Err(e);
        }
    }

    if let Some(firmware) = firmware {
        if let Err(e) = updater.update_firmware(&firmware) {
            eprintln!("Firmware update failed: {}", e);
            return Err(e);
        }
        println!("Firmware update complete.");
    }

    Ok(())
}
