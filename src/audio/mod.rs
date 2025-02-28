/// Audio processing module for analyzing WAV files and extracting harmonic information.
/// 
/// This module provides functionality to:
/// - Read and parse WAV files
/// - Analyze audio data to extract harmonic content
/// - Generate soundfonts from audio analysis
mod analysis;
mod types;
mod wav;

pub use analysis::analyze_harmonics;
pub use types::{AnalysisConfig, AudioError, WavData};
pub use wav::read_wav_file;
