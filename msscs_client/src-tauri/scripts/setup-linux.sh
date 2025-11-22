#!/bin/bash
# Linux build environment setup script

echo "Setting up MSSCS Linux build environment..."

# Check for required dependencies
echo "Checking dependencies..."

# Install Rust target
if ! rustup target list --installed | grep -q "x86_64-unknown-linux-gnu"; then
    echo "Installing Linux target..."
    rustup target add x86_64-unknown-linux-gnu
fi

# Check for system dependencies
echo "Checking system dependencies..."

# GTK dependencies for Linux GUI
if command -v apt-get > /dev/null; then
    echo "Detected Debian/Ubuntu. Installing GTK dependencies..."
    sudo apt-get update
    sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf
elif command -v dnf > /dev/null; then
    echo "Detected Fedora. Installing GTK dependencies..."
    sudo dnf install -y gtk3-devel webkit2gtk3-devel libappindicator-gtk3 librsvg2-devel patchelf
elif command -v pacman > /dev/null; then
    echo "Detected Arch Linux. Installing GTK dependencies..."
    sudo pacman -S --needed gtk3 webkit2gtk libappindicator-gtk3 librsvg patchelf
else
    echo "WARNING: Could not detect package manager. Please install GTK dependencies manually."
fi

echo "Linux build environment setup complete!"
echo "Run: pnpm build:linux"