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

# Install Hyprland 0.55+ Lua window rules and capture hotkey
HELPER="$TMP_DIR/install-hyprland-config.sh"
EXAMPLE="$TMP_DIR/hyprland.lua.example"
if [ ! -f "$HELPER" ]; then
    curl -sL "https://raw.githubusercontent.com/mirarr-app/StickyBoard/main/install-hyprland-config.sh" -o "$HELPER"
    # Older release archives shipped hyprlang snippets; refresh the Lua example too.
    curl -sL "https://raw.githubusercontent.com/mirarr-app/StickyBoard/main/hyprland.lua.example" -o "$EXAMPLE"
fi
# shellcheck disable=SC1090
source "$HELPER"
install_stickyboard_hyprland_config "$EXAMPLE"

# Install Omarchy/Quickshell add-note plugin
PLUGIN_SRC="$TMP_DIR/omarchy-plugin"
PLUGIN_HELPER="$TMP_DIR/install-omarchy-plugin.sh"
if [ ! -f "$PLUGIN_HELPER" ]; then
    curl -sL "https://raw.githubusercontent.com/mirarr-app/StickyBoard/main/install-omarchy-plugin.sh" -o "$PLUGIN_HELPER"
fi
if [ ! -f "$PLUGIN_SRC/manifest.json" ]; then
    mkdir -p "$PLUGIN_SRC"
    for plugin_file in manifest.json BarWidget.qml Panel.qml add-note.sh; do
        curl -sL "https://raw.githubusercontent.com/mirarr-app/StickyBoard/main/omarchy-plugin/${plugin_file}" -o "$PLUGIN_SRC/${plugin_file}"
    done
fi
# shellcheck disable=SC1090
source "$PLUGIN_HELPER"
install_stickyboard_omarchy_plugin "$PLUGIN_SRC"

# Setup Systemd user service
echo "Configuring Systemd user service..."
mkdir -p ~/.config/systemd/user/
cp "$TMP_DIR"/stickyboard.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now stickyboard.service

# Clean up
rm -rf "$TMP_DIR"
echo "StickyBoard installed and started successfully!"
