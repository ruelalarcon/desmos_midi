@echo off
setlocal EnableDelayedExpansion

if not exist "target\release\desmos_midi_web.exe" (
    echo ERROR: Binary not found!
    echo Please run './build.bat' first to build the program.
    exit /b 1
)

:: Default port
set PORT=8573

:: Parse command line arguments
:parse_args
if "%~1"=="" goto :done_args
if "%~1"=="-p" (
    set PORT=%~2
    shift
    shift
    goto :parse_args
)
if "%~1"=="--port" (
    set PORT=%~2
    shift
    shift
    goto :parse_args
)
shift
goto :parse_args
:done_args

:: Start the server and open browser
echo Starting server on port !PORT!...
start "" http://localhost:!PORT!
target\release\desmos_midi_web.exe --port !PORT!

if !ERRORLEVEL! neq 0 (
    exit /b !ERRORLEVEL!
)