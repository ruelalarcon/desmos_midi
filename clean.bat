@echo off
setlocal EnableDelayedExpansion

echo Cleaning build artifacts...

if exist "target" (
    echo Removing target directory...
    rmdir /s /q target
    if !ERRORLEVEL! neq 0 (
        echo ERROR: Failed to remove target directory!
        echo This might be due to file permissions or locked files.
        exit /b 1
    )
)

echo Clean completed successfully!