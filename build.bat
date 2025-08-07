@echo off
REM RustFS Launcher Build Script for Windows
REM Downloads required binary files for Windows platform before building

setlocal enabledelayedexpansion

set BINARIES_DIR=src-tauri\binaries
set TEMP_DIR=temp_downloads

REM Create directories
if not exist "%BINARIES_DIR%" mkdir "%BINARIES_DIR%"
if not exist "%TEMP_DIR%" mkdir "%TEMP_DIR%"

REM Detect architecture
set ARCH=%PROCESSOR_ARCHITECTURE%
if "%ARCH%"=="AMD64" set ARCH=x86_64
if "%ARCH%"=="x86" set ARCH=x86_64

echo Detected platform: Windows %ARCH%
echo Downloading RustFS binary for Windows platform...

REM Download Windows binary only
set WINDOWS_X86_64_URL=https://dl.rustfs.com/artifacts/rustfs/release/rustfs-windows-x86_64-latest.zip

if "%ARCH%"=="x86_64" (
    echo Downloading for Windows x86_64...
    call :download_binary "%WINDOWS_X86_64_URL%" "rustfs-windows-x86_64" "rustfs-windows-x86_64.exe"
) else (
    echo ✗ Error: Unsupported Windows architecture: %ARCH%
    echo Only x86_64 is supported
    exit /b 1
)

REM Clean up temporary files
echo Cleaning up temporary files...
if exist "%TEMP_DIR%" rmdir /s /q "%TEMP_DIR%"

echo Binary downloaded successfully for Windows %ARCH%!
echo You can now run: cargo tauri build
goto :eof

:download_binary
set url=%~1
set filename=%~2
set target_name=%~3

REM Check if binary already exists
if exist "%BINARIES_DIR%\%target_name%" (
    echo ✓ %target_name% already exists, skipping download
    goto :eof
)

echo Downloading %filename%...

REM Download using curl (available in Windows 10+)
curl -L -o "%TEMP_DIR%\%filename%.zip" "%url%"
if errorlevel 1 (
    echo ✗ Error: Failed to download %filename%
    exit /b 1
)

echo Extracting %filename%...
REM Extract using PowerShell
powershell -command "Expand-Archive -Path '%TEMP_DIR%\%filename%.zip' -DestinationPath '%TEMP_DIR%\%filename%' -Force"

REM Find and copy the binary
if exist "%TEMP_DIR%\%filename%\rustfs.exe" (
    copy "%TEMP_DIR%\%filename%\rustfs.exe" "%BINARIES_DIR%\%target_name%"
) else if exist "%TEMP_DIR%\%filename%\rustfs" (
    copy "%TEMP_DIR%\%filename%\rustfs" "%BINARIES_DIR%\%target_name%"
) else (
    echo ✗ Error: Binary not found in extracted files
    dir "%TEMP_DIR%\%filename%\"
    exit /b 1
)

echo ✓ %target_name% downloaded and extracted successfully
goto :eof