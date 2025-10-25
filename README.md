# 🔧 Riden Flash Tool - riden-flashtool
**Rust-based firmware tool for Riden RD60xx Power Supplies**

Cross-platform tool for flashing firmware on Riden power supply units.  
> 🦀 Rust implementation derived from [tjko/riden-flashtool](https://github.com/tjko/riden-flashtool/)

## 📋 Supported Models
**RD6006** • **RD6006P** • **RD6006W** • **RD6012** • **RD6012P** • **RD6018** • **RD6024** • **RD6030**

## ✨ Features
- 🔄 Firmware flashing via serial port
- 🔍 Device detection and diagnostics  
- 🚀 Automatic bootloader mode switching
- 📡 Serial port discovery (`-l` flag)
- 🎯 Zero external dependencies
- 🛡️ Safe model compatibility checks

## 📦 Download Pre-built Binaries

**Ready-to-use binaries** for Windows, Linux, and macOS are available in the [**Releases**](https://github.com/rssdev10/riden-flashtool/releases) section.

### 💻 Supported Platforms:
- 🪟 **Windows** (x64): `riden-flashtool-windows-amd64.exe.zip`
- 🐧 **Linux** (x64): `riden-flashtool-linux-amd64.zip`  
- 🐧 **Linux** (ARM64): `riden-flashtool-linux-arm64.zip`
- 🍎 **macOS** (Intel): `riden-flashtool-darwin-amd64.zip`
- 🍎 **macOS** (Apple Silicon): `riden-flashtool-darwin-arm64.zip`

Each release includes SHA-256 checksums for verification. Simply download, extract, and run!

## ⚡ Quick Start
```bash
# 1. List available serial ports
./riden-flashtool --list

# 2. Check your device (replace with your port)
./riden-flashtool /dev/ttyUSB0

# 3. Flash firmware (if needed)  
./riden-flashtool /dev/ttyUSB0 firmware.bin
```

## 🛠️ Build from Source
1. Install [Rust toolchain](https://www.rust-lang.org/tools/install)
2. Build from source:
```bash
cargo build --release
# Binary appears at: ./target/release/riden-flashtool
```

## 🚀 Usage
### 📝 Basic Commands
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

### ⚠️ Force Mode (DANGEROUS!)

**🛑 WARNING: Use only if you know what you're doing!**

The `--force` flag bypasses model compatibility checks and allows flashing unsupported devices:

```bash
# Force flash unsupported device (DANGEROUS):
./riden-flashtool /dev/ttyUSB0 firmware.bin --force
```

**⚠️ CRITICAL WARNINGS:**
- **May permanently brick your device** if firmware is incompatible
- **No warranty or support** provided for forced operations
- **Use at your own risk** - you may render your device unusable
- **Only for advanced users** who understand the consequences

**Safer alternatives:**
- Wait for official support for your device model
- Contact the developer to request support for new models
- Use only firmware specifically designed for your exact device model

### 📄 Example Output
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

## 🆘 Recovery Mode
If device fails to boot after flashing:
1. **Manual bootloader entry**: Press/hold `ENTER` while powering on unit
2. Re-run flashtool with firmware file

## 🔧 Troubleshooting
- `No response from device`:
  - Verify "Interface" setting is **USB** in PSU menu
  - Connect directly to computer (avoid USB hubs)
  - Ensure serial port permissions (Linux: `sudo usermod -aG dialout $USER`)
- Failed flashing: Retry in bootloader mode (manual method above)
