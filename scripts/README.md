# Packaging Scripts for Desmos MIDI

This directory contains scripts to create distributable packages of the Desmos MIDI application.

## Prerequisites

### Windows
- PowerShell 5.1 or higher (included in Windows 10 and newer)
- .NET Framework 4.5 or higher (for zip functionality)
- Rust toolchain

### Unix/Linux
- `zip` package installed
- Bash shell
- Rust toolchain

## Windows

To create a Windows package:

```powershell
# Run from the project root directory
.\scripts\package_windows.ps1
```

This will:
1. Build the release binaries
2. Create a package directory with the binaries, configuration, and soundfonts
3. Package everything into a zip file in the `package` directory

## Unix/Linux

To create a Unix/Linux package:

```bash
# Run from the project root directory
# Make script executable if needed: chmod +x scripts/package_unix.sh
./scripts/package_unix.sh
```

This will:
1. Check if the `zip` command is available
2. Build the release binaries
3. Create a package directory with the binaries, configuration, and soundfonts
4. Package everything into a zip file in the `package` directory

## Output

The resulting package will be placed in the `package` directory and will include:
- The release binaries (`desmos_midi` and `desmos_midi_web`)
- The configuration file (`config.toml`)
- The soundfonts directory and its contents
- LICENSE files

The zip file name includes the version from Cargo.toml and the platform name.
