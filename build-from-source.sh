#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Build all binaries in release mode
echo "Building StickyBoard from source (release mode)..."
cargo build --release

# Install binaries
echo "Installing binaries to ~/.local/bin/..."
mkdir -p ~/.local/bin
cp target/release/stickyboard target/release/stickyboard-daemon target/release/stickyboard-capture target/release/stickyboard-note ~/.local/bin/

# Install font
echo "Installing Excalifont..."
mkdir -p ~/.local/share/fonts
cp Excalifont-Regular.ttf ~/.local/share/fonts/
fc-cache -f ~/.local/share/fonts/

# Append Hyprland configurations
echo "Adding Hyprland window rules to autostart.lua..."
mkdir -p ~/.config/hypr
touch ~/.config/hypr/autostart.lua
if [ -f "hyprland.lua.example" ]; then
    cat hyprland.lua.example >> ~/.config/hypr/autostart.lua
elif [ -f "hyprland.conf.example" ]; then
    cat hyprland.conf.example >> ~/.config/hypr/autostart.lua
fi

echo "Adding keyboard shortcut to bindings.lua..."
touch ~/.config/hypr/bindings.lua
echo -e '\no.bind("SUPER + SHIFT + K", "Launch StickyBoard Capture", "stickyboard-capture")' >> ~/.config/hypr/bindings.lua

# Setup Systemd user service
echo "Configuring Systemd user service..."
mkdir -p ~/.config/systemd/user/
cp stickyboard.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now stickyboard.service

echo "StickyBoard built from source, installed, and started successfully!"
