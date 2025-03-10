use super::types::MidiError;
use std::fs;
use std::path::Path;

/// Default directory for soundfont files
const DEFAULT_SOUNDFONT_DIR: &str = "soundfonts";

/// Parses a soundfont file from the specified directory.
/// A soundfont file contains comma-separated floating point values representing harmonic weights.
///
/// # Arguments
/// * `filename` - Name of the file in the soundfonts directory
/// * `soundfont_dir` - Optional directory path; defaults to "soundfonts" if None
///
/// # Returns
/// * `Option<Vec<f32>>` - Vector of harmonic weights or None if the filename is "-"
///
/// # Errors
/// * If the file cannot be read
/// * If the file contains invalid floating point numbers
pub fn parse_soundfont_file(
    filename: &str,
    soundfont_dir: Option<&Path>,
) -> Result<Option<Vec<f32>>, MidiError> {
    if filename == "-" {
        return Ok(None);
    }

    let dir = soundfont_dir.unwrap_or_else(|| Path::new(DEFAULT_SOUNDFONT_DIR));
    let path = dir.join(filename);

    fs::read_to_string(&path)
        .map_err(|e| MidiError::Io(e))
        .and_then(|content| {
            content
                .trim()
                .split(',')
                .map(|s| s.trim().parse().map_err(MidiError::Parse))
                .collect::<Result<Vec<f32>, _>>()
                .map(Some)
        })
}

/// Checks if a soundfont file exists in the soundfont directory.
///
/// # Arguments
/// * `filename` - Name of the file to check
/// * `soundfont_dir` - Optional directory path; defaults to "soundfonts" if None
///
/// # Returns
/// * `bool` - True if the file exists, false otherwise
pub fn soundfont_exists(filename: &str, soundfont_dir: Option<&Path>) -> bool {
    if filename == "-" {
        return true;
    }

    let dir = soundfont_dir.unwrap_or_else(|| Path::new(DEFAULT_SOUNDFONT_DIR));
    dir.join(filename).exists()
}

/// Returns the General MIDI instrument name for a given program number.
///
/// # Arguments
/// * `program` - MIDI program number (0-127)
/// * `is_drum` - Whether this is a drum channel (channel 10)
///
/// # Returns
/// * `&str` - Name of the instrument
pub fn get_instrument_name(program: u8, is_drum: bool) -> &'static str {
    if is_drum {
        "Drum Kit"
    } else {
        match program {
            0 => "Acoustic Grand Piano",
            1 => "Bright Acoustic Piano",
            2 => "Electric Grand Piano",
            3 => "Honky-tonk Piano",
            4 => "Electric Piano 1",
            5 => "Electric Piano 2",
            6 => "Harpsichord",
            7 => "Clavinet",
            8 => "Celesta",
            9 => "Glockenspiel",
            10 => "Music Box",
            11 => "Vibraphone",
            12 => "Marimba",
            13 => "Xylophone",
            14 => "Tubular Bells",
            15 => "Dulcimer",
            16 => "Drawbar Organ",
            17 => "Percussive Organ",
            18 => "Rock Organ",
            19 => "Church Organ",
            20 => "Reed Organ",
            21 => "Accordion",
            22 => "Harmonica",
            23 => "Tango Accordion",
            24 => "Acoustic Guitar (nylon)",
            25 => "Acoustic Guitar (steel)",
            26 => "Electric Guitar (jazz)",
            27 => "Electric Guitar (clean)",
            28 => "Electric Guitar (muted)",
            29 => "Overdriven Guitar",
            30 => "Distortion Guitar",
            31 => "Guitar Harmonics",
            32 => "Acoustic Bass",
            33 => "Electric Bass (finger)",
            34 => "Electric Bass (pick)",
            35 => "Fretless Bass",
            36 => "Slap Bass 1",
            37 => "Slap Bass 2",
            38 => "Synth Bass 1",
            39 => "Synth Bass 2",
            40 => "Violin",
            41 => "Viola",
            42 => "Cello",
            43 => "Contrabass",
            44 => "Tremolo Strings",
            45 => "Pizzicato Strings",
            46 => "Orchestral Harp",
            47 => "Timpani",
            48 => "String Ensemble 1",
            49 => "String Ensemble 2",
            50 => "Synth Strings 1",
            51 => "Synth Strings 2",
            52 => "Choir Aahs",
            53 => "Voice Oohs",
            54 => "Synth Voice",
            55 => "Orchestra Hit",
            56 => "Trumpet",
            57 => "Trombone",
            58 => "Tuba",
            59 => "Muted Trumpet",
            60 => "French Horn",
            61 => "Brass Section",
            62 => "Synth Brass 1",
            63 => "Synth Brass 2",
            64 => "Soprano Sax",
            65 => "Alto Sax",
            66 => "Tenor Sax",
            67 => "Baritone Sax",
            68 => "Oboe",
            69 => "English Horn",
            70 => "Bassoon",
            71 => "Clarinet",
            72 => "Piccolo",
            73 => "Flute",
            74 => "Recorder",
            75 => "Pan Flute",
            76 => "Blown Bottle",
            77 => "Shakuhachi",
            78 => "Whistle",
            79 => "Ocarina",
            80 => "Lead 1 (square)",
            81 => "Lead 2 (sawtooth)",
            82 => "Lead 3 (calliope)",
            83 => "Lead 4 (chiff)",
            84 => "Lead 5 (charang)",
            85 => "Lead 6 (voice)",
            86 => "Lead 7 (fifths)",
            87 => "Lead 8 (bass + lead)",
            88 => "Pad 1 (new age)",
            89 => "Pad 2 (warm)",
            90 => "Pad 3 (polysynth)",
            91 => "Pad 4 (choir)",
            92 => "Pad 5 (bowed)",
            93 => "Pad 6 (metallic)",
            94 => "Pad 7 (halo)",
            95 => "Pad 8 (sweep)",
            96 => "FX 1 (rain)",
            97 => "FX 2 (soundtrack)",
            98 => "FX 3 (crystal)",
            99 => "FX 4 (atmosphere)",
            100 => "FX 5 (brightness)",
            101 => "FX 6 (goblins)",
            102 => "FX 7 (echoes)",
            103 => "FX 8 (sci-fi)",
            104 => "Sitar",
            105 => "Banjo",
            106 => "Shamisen",
            107 => "Koto",
            108 => "Kalimba",
            109 => "Bagpipe",
            110 => "Fiddle",
            111 => "Shanai",
            112 => "Tinkle Bell",
            113 => "Agogo",
            114 => "Steel Drums",
            115 => "Woodblock",
            116 => "Taiko Drum",
            117 => "Melodic Tom",
            118 => "Synth Drum",
            119 => "Reverse Cymbal",
            120 => "Guitar Fret Noise",
            121 => "Breath Noise",
            122 => "Seashore",
            123 => "Bird Tweet",
            124 => "Telephone Ring",
            125 => "Helicopter",
            126 => "Applause",
            127 => "Gunshot",
            _ => "Unknown Instrument",
        }
    }
}
