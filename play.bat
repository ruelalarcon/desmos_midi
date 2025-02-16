@echo off
if "%~1"=="" (
    echo Usage: play.bat ^<song_folder^>
    echo Plays a song from the specified folder
    exit /b 1
)
cargo run --release -- play "%~1"