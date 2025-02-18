@echo off
echo Cleaning build artifacts...

if exist "bin" (
    echo Removing bin directory...
    rmdir /s /q bin
)

if exist "target" (
    echo Removing target directory...
    rmdir /s /q target
)

echo Clean complete!