@echo off
setlocal EnableDelayedExpansion

if not exist "bin\desmos_midi.exe" (
    echo.
    echo ERROR: Binary not found!
    echo.
    echo Please run './build.bat' first to build the program.
    exit /b 1
)

bin\desmos_midi.exe %*
if !ERRORLEVEL! neq 0 (
    exit /b !ERRORLEVEL!
)