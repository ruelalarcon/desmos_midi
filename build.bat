@echo off
echo Building in release mode...
cargo build --release

if not exist "bin" mkdir bin

echo Copying binary to bin directory...
copy "target\release\desmos_music.exe" "bin\desmos_music.exe"

echo Build complete! You can now use './run.bat' to run the program.