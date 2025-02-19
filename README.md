# Desmos Music Player

> A Rust-based CLI tool for converting MIDI files into formulas for Desmos.

## Installation

To use the MIDI Player, you must have [Rust](https://www.rust-lang.org/tools/install) installed on your system.

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
git clone https://github.com/ruelalarcon/desmos_music.git
cd desmos_music
```

Then build the project using the provided script:

**Windows:**
```bash
./build.bat
```

**Linux/Mac:**
```bash
./build.sh
```
> Note: You may need to run `chmod +x *.sh` first to make the scripts executable if they aren't by default.

To clean build artifacts at any time, you can use:
```bash
./clean.bat  # Windows
./clean.sh   # Linux/Mac
```

## Usage

Navigate to [this Desmos graph](https://www.desmos.com/calculator/dtm0eynshz).

Once you have a MIDI file you wish to use, run the following command:

**Windows:**
```bash
./run.bat <midi_file> [output_file]
```

**Linux/Mac:**
```bash
./run.sh <midi_file> [output_file]
```

- `<midi_file>`: Path to the input MIDI file to convert
- `<output_file>`: Path to a file to export the formula to, if not provided, the formula will be copied to the clipboard. This is necessary if you are using WSL on Windows.

### Usage Examples

Copy the formula to the clipboard for MIDI file `song.mid`:
```bash
./run.sh song.mid  # Linux/Mac
./run.bat song.mid  # Windows
```

Now enable audio in Desmos through the button in the top left:
![Enable audio](./assets/enable_audio.png)

### Controls

- To play the song, hit the arrow button to the left of the `t -> 0` formula.
- To end the song, hit the arrow button to the left of the `t -> ∞` formula.
- You may also set the value of `t` manually to seek to a specific point in the song.

## Technical Details

### MIDI Processing

1. **MIDI Parsing**:
   - Uses the `midly` crate to parse MIDI files
   - Extracts note events (Note On/Off) and timing information
   - Handles tempo changes to ensure accurate timing

2. **Timing Conversion**:
   - Converts MIDI ticks to milliseconds using the formula:
     ```
     ms = (ticks * tempo) / (ticks_per_quarter * 1000)
     ```
   - Where:
     - `ticks`: MIDI event time in ticks
     - `tempo`: Microseconds per quarter note (default: 500000 = 120 BPM)
     - `ticks_per_quarter`: MIDI file's time division (ticks per quarter note)

3. **Note Processing**:
   - Tracks active notes at each timestamp
   - Converts MIDI note numbers to relative positions from Bb (MIDI note 58)
   - Generates a Desmos piecewise function in the format:
     ```
     A=\left\{t<1:\left[0,4,7\right],t<2:\left[2,5,9\right],...\right\}
     ```
   - Where each number represents semitones relative to Bb (0)

## Dependencies

- `midly`: MIDI file parsing
- `clipboard`: System clipboard integration

## Credits

- [Desmos](https://www.desmos.com/) for the formula visualization
- [Berrynote](https://www.youtube.com/@berrynote/videos) for the initial graph for playing notes on Desmos. [Berrynote's recent video](https://www.youtube.com/watch?v=g2Lp-gIa3es) was the inspiration and base for this project.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.