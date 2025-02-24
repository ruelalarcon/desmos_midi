@echo off
setlocal EnableDelayedExpansion

if not defined CARGO_HOME (
    set CARGO_HOME=%USERPROFILE%\.cargo
)
set PATH=%CARGO_HOME%\bin;%PATH%

echo Building in release mode...
cargo build --release
if !ERRORLEVEL! neq 0 (
    echo.
    echo ERROR: Build failed! Please check the error messages above.
    exit /b 1
)

if not exist "bin" (
    echo Creating bin directory...
    mkdir bin
    if !ERRORLEVEL! neq 0 (
        echo.
        echo ERROR: Failed to create bin directory!
        exit /b 1
    )
)

echo Copying binary to bin directory...
copy "target\release\desmos_midi.exe" "bin\desmos_midi.exe" >nul
if !ERRORLEVEL! neq 0 (
    echo.
    echo ERROR: Failed to copy binary to bin directory!
    exit /b 1
)

echo.
echo Build completed successfully! You can now use './run.bat' to run the program.