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

# Install Hyprland 0.55+ Lua window rules and capture hotkey
# shellcheck disable=SC1091
source "$SCRIPT_DIR/install-hyprland-config.sh"
install_stickyboard_hyprland_config "$SCRIPT_DIR/hyprland.lua.example"

# shellcheck disable=SC1091
source "$SCRIPT_DIR/install-omarchy-plugin.sh"
install_stickyboard_omarchy_plugin "$SCRIPT_DIR/omarchy-plugin"

# Setup Systemd user service
echo "Configuring Systemd user service..."
mkdir -p ~/.config/systemd/user/
cp stickyboard.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now stickyboard.service

echo "StickyBoard built from source, installed, and started successfully!"
