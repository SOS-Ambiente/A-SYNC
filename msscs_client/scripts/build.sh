#!/bin/bash
# Cross-platform build automation

set -e

# Detect platform
PLATFORM=$(uname -s)
case $PLATFORM in
    Linux*)     MACHINE=Linux;;
    Darwin*)    MACHINE=Mac;;
    CYGWIN*)    MACHINE=Cygwin;;
    MINGW*)     MACHINE=MinGw;;
    MSYS*)      MACHINE=MSys;;
    *)          MACHINE="UNKNOWN:${PLATFORM}"
esac

echo "Building MSSCS on ${MACHINE}..."

# Install dependencies
echo "Installing dependencies..."
pnpm install

# Build frontend
echo "Building frontend..."
pnpm run build

# Setup build environment based on platform
case $MACHINE in
    Linux)
        echo "Setting up Linux build environment..."
        chmod +x src-tauri/scripts/setup-linux.sh
        ./src-tauri/scripts/setup-linux.sh

        echo "Building for Linux..."
        pnpm run build:linux
        ;;
    MinGw|MSys|Cygwin)
        echo "Setting up Windows build environment..."
        src-tauri/scripts/setup-windows.bat

        echo "Building for Windows..."
        pnpm run build:windows
        ;;
    Mac)
        echo "Building for macOS..."
        pnpm run build
        ;;
    *)
        echo "Unsupported platform: ${MACHINE}"
        exit 1
        ;;
esac

echo "Build completed successfully!"
echo "Check src-tauri/target/release/bundle/ for output files."