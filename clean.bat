@echo off
setlocal EnableDelayedExpansion

echo Cleaning build artifacts...
echo.

if exist "bin" (
    echo Removing bin directory...
    rmdir /s /q bin
    if !ERRORLEVEL! neq 0 (
        echo.
        echo ERROR: Failed to remove bin directory!
        echo This might be due to file permissions or locked files.
        exit /b 1
    )
)

if exist "target" (
    echo Removing target directory...
    rmdir /s /q target
    if !ERRORLEVEL! neq 0 (
        echo.
        echo ERROR: Failed to remove target directory!
        echo This might be due to file permissions or locked files.
        exit /b 1
    )
)

echo.
echo Clean completed successfully!