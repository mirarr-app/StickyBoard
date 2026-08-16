#!/bin/bash
set -e

# Create a temporary directory
TMP_DIR=$(mktemp -d)

# Download the latest release tarball
echo "Downloading the latest StickyBoard release..."
curl -sL https://github.com/mirarr-app/StickyBoard/releases/latest/download/stickyboard-linux-x86_64.tar.gz | tar -xz -C "$TMP_DIR"

# Install binaries
echo "Installing binaries to ~/.local/bin/..."
mkdir -p ~/.local/bin
cp "$TMP_DIR"/stickyboard "$TMP_DIR"/stickyboard-daemon "$TMP_DIR"/stickyboard-capture "$TMP_DIR"/stickyboard-note ~/.local/bin/

# Install font
echo "Installing Excalifont..."
mkdir -p ~/.local/share/fonts
cp "$TMP_DIR"/Excalifont-Regular.ttf ~/.local/share/fonts/
fc-cache -f ~/.local/share/fonts/

# Append Hyprland configurations
echo "Adding Hyprland window rules to autostart.lua..."
mkdir -p ~/.config/hypr
touch ~/.config/hypr/autostart.lua
if [ -f "$TMP_DIR/hyprland.lua.example" ]; then
    cat "$TMP_DIR/hyprland.lua.example" >> ~/.config/hypr/autostart.lua
elif [ -f "$TMP_DIR/hyprland.conf.example" ]; then
    cat "$TMP_DIR/hyprland.conf.example" >> ~/.config/hypr/autostart.lua
fi

echo "Adding keyboard shortcut to bindings.lua..."
touch ~/.config/hypr/bindings.lua
echo -e '\no.bind("SUPER + SHIFT + K", "Launch StickyBoard Capture", "stickyboard-capture")' >> ~/.config/hypr/bindings.lua

# Setup Systemd user service
echo "Configuring Systemd user service..."
mkdir -p ~/.config/systemd/user/
cp "$TMP_DIR"/stickyboard.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now stickyboard.service

# Clean up
rm -rf "$TMP_DIR"
echo "StickyBoard installed and started successfully!"
