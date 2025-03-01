@echo off
setlocal EnableDelayedExpansion

if not defined CARGO_HOME (
    set CARGO_HOME=%USERPROFILE%\.cargo
)
set PATH=%CARGO_HOME%\bin;%PATH%

:: Default to building everything
set BUILD_CLI=1
set BUILD_WEB=1

:: Parse command line arguments
:parse_args
if "%~1"=="" goto :done_args
if "%~1"=="--help" goto :show_help
if "%~1"=="-h" goto :show_help
if "%~1"=="--cli-only" (
    set BUILD_WEB=0
    shift
    goto :parse_args
)
if "%~1"=="--web-only" (
    set BUILD_CLI=0
    shift
    goto :parse_args
)
echo ERROR: Unknown argument: %~1
goto :show_help
:done_args

echo Building in release mode...

:: Build based on selected options
if !BUILD_CLI! equ 1 (
    if !BUILD_WEB! equ 1 (
        echo Building CLI and Web UI...
        cargo build --release --features "cli webui" --bins
    ) else (
        echo Building CLI...
        cargo build --release --features cli --bin desmos_midi
    )
) else (
    if !BUILD_WEB! equ 1 (
        echo Building Web UI...
        cargo build --release --features webui --bin desmos_midi_web
    )
)

if !ERRORLEVEL! neq 0 (
    echo ERROR: Build failed! Please check the error messages above.
    exit /b 1
)

echo Build completed successfully!
if !BUILD_CLI! equ 1 (
    echo You can use './cli.bat' to run the CLI.
)
if !BUILD_WEB! equ 1 (
    echo You can use './webui.bat' to run the Web UI.
)
exit /b 0

:show_help
echo Usage: build.bat [OPTIONS]
echo.
echo Options:
echo   --cli-only    Build only the CLI version
echo   --web-only    Build only the Web UI version
echo   -h, --help    Show this help message
echo.
echo If no options are provided, both CLI and Web UI will be built.
exit /b 1