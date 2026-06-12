#!/bin/bash
set -e

# Create a temporary directory
TMP_DIR=$(mktemp -d)

# Download the latest release tarball
echo "Downloading the latest StickyBoard release..."
curl -sL https://github.com/mirarr-app/StickyBoard/releases/latest/download/stickyboard-linux-x86_64.tar.gz | tar -xz -C "$TMP_DIR"

# Install binaries
echo "Installing binaries to /usr/bin/..."
sudo cp "$TMP_DIR"/stickyboard "$TMP_DIR"/stickyboard-daemon "$TMP_DIR"/stickyboard-capture "$TMP_DIR"/stickyboard-note /usr/bin/

# Install font
echo "Installing Excalifont..."
mkdir -p ~/.local/share/fonts
cp "$TMP_DIR"/Excalifont-Regular.ttf ~/.local/share/fonts/
fc-cache -f ~/.local/share/fonts/

# Append Hyprland configurations
echo "Adding Hyprland window rules to autostart.conf..."
mkdir -p ~/.config/hypr
touch ~/.config/hypr/autostart.conf
cat "$TMP_DIR"/hyprland.conf.example >> ~/.config/hypr/autostart.conf

echo "Adding keyboard shortcut to bindings.conf..."
touch ~/.config/hypr/bindings.conf
echo -e "\nbindd = SUPER SHIFT, K, Launch StickyBoard Capture, exec, stickyboard-capture" >> ~/.config/hypr/bindings.conf

# Setup Systemd user service
echo "Configuring Systemd user service..."
mkdir -p ~/.config/systemd/user/
cp "$TMP_DIR"/stickyboard.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now stickyboard.service

# Clean up
rm -rf "$TMP_DIR"
echo "StickyBoard installed and started successfully!"
