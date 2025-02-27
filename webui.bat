@echo off
setlocal EnableDelayedExpansion

if not exist "target\release\desmos_midi_web.exe" (
    echo ERROR: Binary not found!
    echo Please run './build.bat' first to build the program.
    exit /b 1
)

target\release\desmos_midi_web.exe
if !ERRORLEVEL! neq 0 (
    exit /b !ERRORLEVEL!
)