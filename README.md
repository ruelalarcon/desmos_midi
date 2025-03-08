# Desmos MIDI Player

> A Rust-based CLI tool and Web UI for converting MIDI files into formulas for Desmos. Complete with support for note velocity, dynamic tempo, custom soundfonts for different channels/instruments, wav to soundfont conversion utilizing FFT audio analysis, and a webpage for creating and visualizing soundfonts from scratch.

## Quick Installation

Pre-built releases are available for both Windows and Linux:

1. Download the appropriate file for your operating system:
   - [Windows (64-bit)](https://github.com/ruelalarcon/desmos_midi/releases/download/v2.0.1/desmos_midi_2.0.1_windows_x64.zip)
   - [Linux (64-bit)](https://github.com/ruelalarcon/desmos_midi/releases/download/v2.0.1/desmos_midi_2.0.1_linux_x86_64.zip)

2. Extract the ZIP file to a location of your choice

3. Run the web interface:
   - On Windows: Double-click on `desmos_midi_web.exe`
   - On Linux: Open a terminal in the extracted folder and run `./desmos_midi_web`

This will start a local web server and automatically open your browser to the application.

## Web Interface Usage

The web interface provides a user-friendly experience for converting MIDI files to Desmos formulas:

1. Upload MIDI files by dragging and dropping or clicking to browse
2. View channel information for your MIDI file
3. Configure soundfonts for each channel
4. Convert to Desmos formula with a single click
5. Copy the formula to clipboard

To launch the web interface from the command line with options:

```bash
# Specify a custom port
./desmos_midi_web --port 9000

# Disable automatic browser opening
./desmos_midi_web --no-open-browser

# Show help
./desmos_midi_web --help
```

### WAV to Soundfont Converter

The web interface includes a WAV to soundfont converter that allows you to create custom soundfonts from audio files. To use it:

1. Upload a WAV file by dragging and dropping or clicking to browse
2. Configure the analysis parameters:

   - **Samples** (1024-32768): Number of samples to analyze. Higher values give better accuracy but slower analysis. The value is 2^n (e.g., 2^13 = 8192 samples).

   - **Start Time** (0-10s): Position in the audio file to begin analysis. Useful for skipping silence or finding the best-sounding part of the audio.

   - **Base Frequency** (0-2000Hz): Fundamental frequency to analyze. For best results, this should match the pitch of your audio. For example:
     - A4 = 440Hz
     - C5 = 523Hz
     - G4 = 392Hz

   - **Number of Harmonics** (1-64): Number of harmonics to extract from the audio. More harmonics create a richer sound, but too many can introduce artifacts.

   - **Boost** (0.5-2.0×): Amplification factor for the harmonics. Higher values make the sound brighter but may cause clipping.

3. Preview the generated soundfont using the built-in audio player
4. Save the soundfont when you're satisfied with the result

The converter uses FFT analysis to extract the harmonic content of your audio, which can then be used as a soundfont in the MIDI converter.

### Soundfont Studio

The web interface also includes a way to create soundfonts from scratch by adjusting sliders which correspond to values for harmonic weights. Like the WAV to Soundfont Converter, the studio comes with the ability to preview the sound in real-time as you make changes.

Comes with presets that you can start from and begin editing, including:

  - Sine Wave
  - Square Wave
  - Triangle Wave
  - Sawtooth Wave
  - Organ (Equivalent to the `default.txt` preset)

Once you're satisfied with the result, you can save the soundfont.

## Using the Graph

After converting your MIDI file, you'll need to use the formula in Desmos:

1. Navigate to [this Desmos graph](https://www.desmos.com/calculator/1rzq4xa5v0)
2. Paste your formula into an empty formula input
3. Enable audio in Desmos through the button in the top left
4. Play the song by clicking the arrow button to the left of the `t -> 0` formula
5. End the song by clicking the arrow button to the left of the `t -> ∞` formula

You can also:
- Set the value of `t` manually to seek to a specific point in the song
- Open the "Settings" folder to adjust:
  - `velocity`: Animation phase speed
  - `scale`: Visual waveform amplitude
  - `hertz`: Base frequency (default 440Hz)
  - `detune`: The amount of detune applied to the secondary tone
  - `volume`: Global volume control
  - `transpose`: Global pitch shift in semitones

## Configuration

The application uses a `config.toml` file in the application directory for configuration. If this file doesn't exist, default values will be used.

### Example Configuration

```toml
[common]
# Directory where soundfonts are stored
soundfonts_dir = "soundfonts"

[server]
# Time in minutes before uploaded files are deleted
file_expiration_minutes = 10

# Time in minutes before file expiration when refresh should occur
file_refresh_threshold_minutes = 5

# Maximum file size in megabytes
max_file_size_mb = 80

# WAV analysis parameter limits
[server.limits]
min_samples = 64        # Minimum number of samples (2^6)
max_samples = 65536     # Maximum number of samples (2^16)
min_start_time = 0.0    # Minimum start time in seconds
max_start_time = 300.0  # Maximum start time in seconds (5 minutes)
min_base_freq = 1.0     # Minimum base frequency in Hz
max_base_freq = 20000.0 # Maximum base frequency in Hz (human hearing limit)
min_harmonics = 1       # Minimum number of harmonics
max_harmonics = 256     # Maximum number of harmonics
min_boost = 0.5         # Minimum boost factor
max_boost = 2.0         # Maximum boost factor
```

## Command Line Interface

For advanced users, a command line interface is also available. The CLI supports two main commands: `midi` for MIDI file processing and `audio` for WAV file analysis.

### MIDI Processing

**Basic Usage:**
```bash
./desmos_midi midi <midi_file>
```

**Advanced Usage:**
```bash
# Specify soundfonts for each channel
./desmos_midi midi <midi_file> -s <soundfont1> <soundfont2> ...

# Show channel information
./desmos_midi midi <midi_file> -i
```

#### MIDI Arguments
- `<midi_file>`: Path to the input MIDI file to convert
- `-s, --soundfonts <FILES>`: Soundfont files to use for each channel (optional)
- `-i, --info`: Show MIDI channel information and exit
- `-c, --copy`: Copy output to clipboard instead of console

#### Soundfonts
By default:
- Regular channels use `default` soundfont
- Drum channels (channel 10) are automatically ignored
- To include drum sounds or use different soundfonts, use the `-s` option and specify a soundfont for each channel
- Use `-` as a soundfont name to ignore that channel
- The `.txt` extension is optional for soundfont files - it will be added automatically if not specified
- Soundfonts are loaded from the directory specified in `config.toml` (default: "soundfonts")

### Audio Analysis

The audio analysis command allows you to create soundfonts from WAV files:

**Basic Usage:**
```bash
# Outputs soundfont content to console (harmonic weights)
./desmos_midi audio <wav_file>
```

**Advanced Usage:**
```bash
# Customize analysis parameters
./desmos_midi audio <wav_file> --samples 16384 --base-freq 523 --harmonics 32

# Copy to clipboard
./desmos_midi audio <wav_file> -c

# Save soundfont to file
./desmos_midi audio <wav_file> > soundfonts/example.txt
```

#### Audio Arguments
- `<wav_file>`: Path to the input WAV file to analyze
- `--samples <NUM>`: Number of samples to analyze (default: 8192)
- `--start-time <SECONDS>`: Position in audio to begin analysis (default: 0.0)
- `--base-freq <HZ>`: Fundamental frequency to analyze (default: 440.0)
- `--harmonics <NUM>`: Number of harmonics to extract (default: 16)
- `--boost <FACTOR>`: Amplification factor for harmonics (default: 1.0)
- `-c, --copy`: Copy output to clipboard instead of console

## Building from Source

If you prefer to build the application from source, you'll need [Rust](https://www.rust-lang.org/tools/install) installed on your system.

### Prerequisites

**Windows:**
- No additional prerequisites

**Linux/WSL (Ubuntu/Debian):**
```bash
# Install X11 development libraries
sudo apt update
sudo apt install libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

**Other Linux distributions:**
- Equivalent X11 development packages for your package manager

### Building

Clone this repository and navigate to the directory:

```bash
git clone https://github.com/ruelalarcon/desmos_midi.git
cd desmos_midi
```

Then build the project using cargo:

```bash
# Build both CLI and Web UI
cargo build

# Build only the CLI
cargo build --bin desmos_midi

# Build only the Web UI
cargo build --bin desmos_midi_web
```

> For production/release builds, add the `--release` flag.

The CLI version has minimal dependencies and is quick to build. The Web UI version includes additional dependencies for the web server and interface.

To clean build artifacts at any time, you can use:
```bash
cargo clean
```

## Credits

- [Desmos](https://www.desmos.com/) for the formula visualization
- [Berrynote](https://www.youtube.com/@berrynote/videos) for the initial graph for playing notes on Desmos. [Berrynote's recent video](https://www.youtube.com/watch?v=g2Lp-gIa3es) was the inspiration and base for this project.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.