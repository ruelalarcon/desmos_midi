@echo off
setlocal EnableDelayedExpansion

if not defined CARGO_HOME (
    set CARGO_HOME=%USERPROFILE%\.cargo
)
set PATH=%CARGO_HOME%\bin;%PATH%

echo Building in release mode...
cargo build --release
if !ERRORLEVEL! neq 0 (
    echo ERROR: Build failed! Please check the error messages above.
    exit /b 1
)

echo Build completed successfully! You can now use './convert.bat' or './webui.bat' to run the program.