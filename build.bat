@echo off
REM RustFS Launcher Build Script for Windows
REM Downloads required binary files before building

setlocal enabledelayedexpansion

set BINARIES_DIR=src-tauri\binaries
set TEMP_DIR=temp_downloads

REM Create directories
if not exist "%BINARIES_DIR%" mkdir "%BINARIES_DIR%"
if not exist "%TEMP_DIR%" mkdir "%TEMP_DIR%"

echo Downloading RustFS binaries...

REM Download URLs
set MACOS_AARCH64_URL=https://dl.rustfs.com/artifacts/rustfs/release/rustfs-macos-aarch64-latest.zip
set MACOS_X86_64_URL=https://dl.rustfs.com/artifacts/rustfs/release/rustfs-macos-x86_64-latest.zip
set WINDOWS_X86_64_URL=https://dl.rustfs.com/artifacts/rustfs/release/rustfs-windows-x86_64-latest.zip

REM Function to download and extract binary
call :download_binary "%MACOS_AARCH64_URL%" "rustfs-macos-aarch64" "rustfs-macos-aarch64"
call :download_binary "%MACOS_X86_64_URL%" "rustfs-macos-x86_64" "rustfs-macos-x86_64"
call :download_binary "%WINDOWS_X86_64_URL%" "rustfs-windows-x86_64" "rustfs-windows-x86_64.exe"

REM Clean up temporary files
echo Cleaning up temporary files...
if exist "%TEMP_DIR%" rmdir /s /q "%TEMP_DIR%"

echo All binaries downloaded successfully!
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