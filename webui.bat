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
if "%~1"=="--help" goto :show_help
if "%~1"=="-h" goto :show_help
if "%~1"=="-p" (
    if "%~2"=="" goto :invalid_port
    set "PORT_NUM=%~2"
    call :validate_port !PORT_NUM! || goto :invalid_port
    set "PORT=!PORT_NUM!"
    shift
    shift
    goto :parse_args
)
if "%~1"=="--port" (
    if "%~2"=="" goto :invalid_port
    set "PORT_NUM=%~2"
    call :validate_port !PORT_NUM! || goto :invalid_port
    set "PORT=!PORT_NUM!"
    shift
    shift
    goto :parse_args
)
echo ERROR: Unknown argument: %~1
goto :show_help
:done_args

:: Start the server and open browser
echo Starting server on port !PORT!...
start "" http://localhost:!PORT!
target\release\desmos_midi_web.exe --port !PORT!

if !ERRORLEVEL! neq 0 (
    exit /b !ERRORLEVEL!
)
exit /b 0

:validate_port
set "PORT_VAL=%~1"
echo !PORT_VAL!| findstr /r "^[1-9][0-9]*$" >nul || exit /b 1
if !PORT_VAL! GTR 65535 exit /b 1
exit /b 0

:show_help
echo Usage: webui.bat [OPTIONS]
echo.
echo Options:
echo   -p, --port PORT    Port to run the server on (default: 8573)
echo   -h, --help         Show this help message
echo.
echo The port number must be between 1 and 65535.
exit /b 1

:invalid_port
echo ERROR: Invalid port number. Must be a number between 1 and 65535.
exit /b 1