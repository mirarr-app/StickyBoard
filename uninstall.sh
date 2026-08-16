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
echo "Cleaning up Hyprland configuration files..."
python3 - << 'PYEOF' 2>/dev/null || true
import re, os

home = os.path.expanduser("~")

def clean_file(path, patterns):
    if not os.path.exists(path):
        return
    with open(path, "r") as f:
        content = f.read()
    for pattern, repl in patterns:
        content = re.sub(pattern, repl, content, flags=re.IGNORECASE)
    content = re.sub(r"\n{3,}", "\n\n", content).strip() + "\n"
    with open(path, "w") as f:
        f.write(content)

# Autostart Lua
autostart_lua = os.path.join(home, ".config/hypr/autostart.lua")
clean_file(autostart_lua, [
    (r"--\s*StickyBoard[^\n]*\n?", ""),
    (r"--\s*Note Windows Rules[^\n]*\n?", ""),
    (r"--\s*Capture Popup Window Rules[^\n]*\n?", ""),
    (r"--\s*Direct all notes[^\n]*\n?", ""),
    (r"--\s*Float the capture window[^\n]*\n?", ""),
    (r"--\s*Append this to[^\n]*\n?", ""),
    (r"o\.window\s*\(\s*[\"'][^\"']*stickyboard[^\"']*[\"']\s*,\s*\{[\s\S]*?\}\s*\)\s*", ""),
])

# Autostart Conf
autostart_conf = os.path.join(home, ".config/hypr/autostart.conf")
clean_file(autostart_conf, [
    (r"#\s*StickyBoard[^\n]*\n?", ""),
    (r"#\s*Note Windows Rules[^\n]*\n?", ""),
    (r"#\s*Capture Popup Window Rules[^\n]*\n?", ""),
    (r"#\s*Direct all notes[^\n]*\n?", ""),
    (r"#\s*Float the capture window[^\n]*\n?", ""),
    (r"#\s*Append this to[^\n]*\n?", ""),
    (r"windowrule\s*\{[\s\S]*?stickyboard[\s\S]*?\}\s*", ""),
])

# Bindings Lua & Conf
bindings_lua = os.path.join(home, ".config/hypr/bindings.lua")
clean_file(bindings_lua, [
    (r"o\.bind\s*\(\s*[\"'][^\"']*[\"']\s*,\s*[\"'][^\"']*stickyboard[^\"']*[\"'][^\n]*\n?", ""),
    (r"o\.bind\s*\(\s*[\"'][^\"']*[\"']\s*,\s*[\"'][^\"']*[\"']\s*,\s*[\"'][^\"']*stickyboard[^\"']*[\"'][^\n]*\n?", ""),
])

bindings_conf = os.path.join(home, ".config/hypr/bindings.conf")
clean_file(bindings_conf, [
    (r"bind[a-z]*\s*=\s*[^\n]*stickyboard[^\n]*\n?", ""),
])
PYEOF

# Reload Hyprland rules if hyprctl is available
if command -v hyprctl >/dev/null 2>&1; then
    hyprctl reload 2>/dev/null || true
fi

# 6. Remove runtime socket
rm -f "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/stickyboard.sock"

# 7. Handle app data and notes database
if [ "$PURGE_DATA" -eq 1 ]; then
    echo "Purging StickyBoard data directory (~/.local/share/stickyboard)..."
    rm -rf ~/.local/share/stickyboard
else
    if [ -d ~/.local/share/stickyboard ]; then
        echo "Note: Notes database preserved at ~/.local/share/stickyboard (pass --purge to remove)."
    fi
fi

echo "StickyBoard has been uninstalled successfully!"
