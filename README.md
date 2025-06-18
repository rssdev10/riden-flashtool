# riden-flashtool
**Rust-based firmware tool for Riden RD60xx PSUs**

Unofficial cross-platform tool for flashing Riden RD6006(P)/RD6012/RD6018/RD6024 power supplies.  
> Derived from [tjko/riden-flashtool](https://github.com/tjko/riden-flashtool/) with Rust implementation

## Features
- Firmware flashing via serial port
- Device detection and diagnostics
- Bootloader mode switching
- Serial port listing (`-l` flag)
- No external dependencies

## Installation
1. Install [Rust toolchain](https://www.rust-lang.org/tools/install)
2. Build from source:
```bash
cargo build --release
# Binary appears at: ./target/release/riden-flashtool
```

## Usage
### Basic commands
```bash
# List available serial ports:
./riden-flashtool -l

# Get device info (normal mode):
./riden-flashtool /dev/ttyUSB0

# Flash firmware (auto-detects baudrate):
./riden-flashtool /dev/ttyUSB0 firmware.bin

# Specify custom baudrate:
./riden-flashtool /dev/ttyUSB0 firmware.bin --speed 115200
```

### Example Output
```bash
Serial port: /dev/ttyUSB0 (115200bps)
Firmware size: 109888 bytes
Check if device in bootloader... No
Found device: RD6006 (60062) v1.40
Rebooting to bootloader...
Device info (bootloader):
    Model: RD6006 (60062)
    Firmware: v1.40
    S/N: 000xxxxx
Updating firmware...b'OK'
Firmware update complete.
```

## Recovery Mode
If device fails to boot after flashing:
1. **Manual bootloader entry**: Press/hold `ENTER` while powering on unit
2. Re-run flashtool with firmware file

## Troubleshooting
- `No response from device`:
  - Verify "Interface" setting is **USB** in PSU menu
  - Connect directly to computer (avoid USB hubs)
  - Ensure serial port permissions (Linux: `sudo usermod -aG dialout $USER`)
- Failed flashing: Retry in bootloader mode (manual method above)
