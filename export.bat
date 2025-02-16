@echo off
if "%~1"=="" (
    echo Usage: export.bat ^<midi_file^>
    echo Exports a MIDI file to timing.bin and formulas.txt
    exit /b 1
)
cargo run --release -- export "%~1"