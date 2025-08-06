#!/bin/bash

# RustFS Launcher Build Script
# Downloads required binary files before building

set -e

BINARIES_DIR="src-tauri/binaries"
TEMP_DIR="temp_downloads"

# Create directories
mkdir -p "$BINARIES_DIR"
mkdir -p "$TEMP_DIR"

echo "Downloading RustFS binaries..."

# Download URLs
MACOS_AARCH64_URL="https://dl.rustfs.com/artifacts/rustfs/release/rustfs-macos-aarch64-latest.zip"
MACOS_X86_64_URL="https://dl.rustfs.com/artifacts/rustfs/release/rustfs-macos-x86_64-latest.zip"
WINDOWS_X86_64_URL="https://dl.rustfs.com/artifacts/rustfs/release/rustfs-windows-x86_64-latest.zip"

# Function to download and extract binary
download_binary() {
    local url=$1
    local filename=$2
    local target_name=$3
    
    # Check if binary already exists
    if [ -f "$BINARIES_DIR/$target_name" ]; then
        echo "✓ $target_name already exists, skipping download"
        return 0
    fi
    
    echo "Downloading $filename..."
    
    if curl -L -o "$TEMP_DIR/$filename.zip" "$url"; then
        echo "Extracting $filename..."
        unzip -o -q "$TEMP_DIR/$filename.zip" -d "$TEMP_DIR/$filename"
        
        # The extracted binary is named 'rustfs' or 'rustfs.exe'
        local extracted_binary=""
        if [ -f "$TEMP_DIR/$filename/rustfs.exe" ]; then
            extracted_binary="rustfs.exe"
        elif [ -f "$TEMP_DIR/$filename/rustfs" ]; then
            extracted_binary="rustfs"
        else
            echo "✗ Error: Binary not found in extracted files"
            ls -la "$TEMP_DIR/$filename/"
            exit 1
        fi
        
        cp "$TEMP_DIR/$filename/$extracted_binary" "$BINARIES_DIR/$target_name"
        chmod +x "$BINARIES_DIR/$target_name"
        echo "✓ $target_name downloaded and extracted successfully"
    else
        echo "✗ Error: Failed to download $filename"
        exit 1
    fi
}

# Download all binaries
download_binary "$MACOS_AARCH64_URL" "rustfs-macos-aarch64" "rustfs-macos-aarch64"
download_binary "$MACOS_X86_64_URL" "rustfs-macos-x86_64" "rustfs-macos-x86_64"
download_binary "$WINDOWS_X86_64_URL" "rustfs-windows-x86_64" "rustfs-windows-x86_64.exe"

# Clean up temporary files
echo "Cleaning up temporary files..."
rm -rf "$TEMP_DIR"

echo "All binaries downloaded successfully!"
echo "You can now run: cargo tauri build"