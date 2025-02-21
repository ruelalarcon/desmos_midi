@echo off
if not exist "bin\desmos_midi.exe" (
    echo Binary not found. Please run './build.bat' first.
    exit /b 1
)

bin\desmos_midi.exe %*