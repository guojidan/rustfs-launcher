# RustFS Launcher

A Tauri + Leptos application for launching RustFS.

## Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)
- [Trunk](https://trunkrs.dev/) - Install with `cargo install trunk`

## Building

Before building the application, you need to download the required RustFS binaries:

### On macOS/Linux:
```bash
# Download required binaries
./build.sh

# Build for development
cargo tauri dev

# Build for production
cargo tauri build
```

### On Windows:
```cmd
# Download required binaries
build.bat

# Build for development
cargo tauri dev

# Build for production
cargo tauri build
```

The build script will download the following binaries:
- `rustfs-macos-aarch64` - macOS ARM64 binary
- `rustfs-macos-x86_64` - macOS x86_64 binary  
- `rustfs-windows-x86_64.exe` - Windows x86_64 binary

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
