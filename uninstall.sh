#!/bin/bash
set -e

PURGE_DATA=0
for arg in "$@"; do
    case "$arg" in
        --purge|-p)
            PURGE_DATA=1
            ;;
        --help|-h)
            echo "Usage: ./uninstall.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -p, --purge    Also delete the notes database and app data directory (~/.local/share/stickyboard)"
            echo "  -h, --help     Show this help message"
            exit 0
            ;;
    esac
done

echo "Uninstalling StickyBoard..."

# 1. Stop and disable systemd user service
echo "Stopping and disabling systemd service..."
systemctl --user stop stickyboard.service 2>/dev/null || true
systemctl --user disable stickyboard.service 2>/dev/null || true
rm -f ~/.config/systemd/user/stickyboard.service
systemctl --user daemon-reload 2>/dev/null || true
systemctl --user reset-failed 2>/dev/null || true

# 2. Kill any dangling processes
pkill -f "stickyboard-daemon" 2>/dev/null || true
pkill -f "stickyboard-note" 2>/dev/null || true

# 3. Remove installed binaries
echo "Removing binaries from ~/.local/bin/..."
rm -f ~/.local/bin/stickyboard \
      ~/.local/bin/stickyboard-daemon \
      ~/.local/bin/stickyboard-capture \
      ~/.local/bin/stickyboard-note

# 4. Remove installed font
if [ -f ~/.local/share/fonts/Excalifont-Regular.ttf ]; then
    echo "Removing Excalifont..."
    rm -f ~/.local/share/fonts/Excalifont-Regular.ttf
    fc-cache -f ~/.local/share/fonts/ 2>/dev/null || true
fi

# 5. Clean up Hyprland window rules and keybindings
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/install-hyprland-config.sh" ]; then
    # shellcheck disable=SC1091
    source "$SCRIPT_DIR/install-hyprland-config.sh"
    uninstall_stickyboard_hyprland_config
else
    echo "Cleaning up Hyprland configuration files..."
    rm -f ~/.config/hypr/stickyboard.lua
    python3 - << 'PYEOF' 2>/dev/null || true
import os, re
home = os.path.expanduser("~")

def clean(path, patterns):
    if not os.path.exists(path):
        return
    with open(path, "r") as f:
        content = f.read()
    for pattern in patterns:
        content = re.sub(pattern, "", content, flags=re.I | re.M)
    content = re.sub(r"\n{3,}", "\n\n", content).strip() + "\n"
    with open(path, "w") as f:
        f.write(content)

pats = [
    r'^[ \t]*require\(\s*["\'](?:hypr\.)?stickyboard["\']\s*\)[^\n]*\n?',
    r"^[ \t]*o\.bind\s*\([^\n]*stickyboard[^\n]*\n?",
    r"^[ \t]*hl\.bind\s*\([^\n]*stickyboard[^\n]*\n?",
    r"^[ \t]*bind[a-z]*\s*=\s*[^\n]*stickyboard[^\n]*\n?",
]
for rel in ("hypr/hyprland.lua", "hypr/autostart.lua", "hypr/bindings.lua",
            "hypr/autostart.conf", "hypr/bindings.conf"):
    clean(os.path.join(home, ".config", rel), pats)
PYEOF
    if command -v hyprctl >/dev/null 2>&1; then
        hyprctl reload 2>/dev/null || true
    fi
fi

# 6. Remove Omarchy/Quickshell plugin
if [ -f "$SCRIPT_DIR/install-omarchy-plugin.sh" ]; then
    # shellcheck disable=SC1091
    source "$SCRIPT_DIR/install-omarchy-plugin.sh"
    uninstall_stickyboard_omarchy_plugin
else
    rm -rf ~/.config/omarchy/plugins/stickyboard.notes
    if command -v omarchy-shell >/dev/null 2>&1; then
        omarchy-shell shell setPluginEnabled stickyboard.notes false >/dev/null 2>&1 || true
        omarchy-shell shell rescanPlugins >/dev/null 2>&1 || true
    fi
fi

# 7. Remove runtime socket
rm -f "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/stickyboard.sock"

# 8. Handle app data and notes database
if [ "$PURGE_DATA" -eq 1 ]; then
    echo "Purging StickyBoard data directory (~/.local/share/stickyboard)..."
    rm -rf ~/.local/share/stickyboard
else
    if [ -d ~/.local/share/stickyboard ]; then
        echo "Note: Notes database preserved at ~/.local/share/stickyboard (pass --purge to remove)."
    fi
fi

echo "StickyBoard has been uninstalled successfully!"
