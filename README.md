# StickyBoard

StickyBoard is a lightweight sticky notes system designed specifically for Omarchy. It transforms a Hyprland workspace (6 by default) into a persistent corkboard of floating sticky notes.

StickyBoard is built using GTK4, SQLite, Tokio, and native Hyprland IPC.

### Omarchy Quick Install

Install all binaries, default fonts, systemd user services, and automatically configure keybindings and window rules in a single command:
```bash
curl -sL https://raw.githubusercontent.com/mirarr-app/StickyBoard/main/install.sh | bash
```

---

## Features

- **Instant Note Capture**: Press `SUPER+SHIFT+K` to open a centered input popup, type your note, and hit `Enter` to spawn it instantly.
- **Dedicated Corkboard**: Automatically routes and manages all note windows on workspace 6 by default.
- **Geometry Recovery**: Changes to note positions and sizes are autosaved and survive system reboots or Hyprland/daemon restarts. Notes are read-only and static once created to keep workspace clean.
- **Omarchy System Theme Integration**: Note colors are loaded dynamically from your active Omarchy system theme (`~/.local/state/omarchy/current/theme/colors.toml`, falling back to `~/.config/omarchy/current/theme/colors.toml`) and hot-reload instantly when the theme is changed.



https://github.com/user-attachments/assets/319e65e9-3a7f-4889-8004-56b2b5a5456d



---

## System Architecture

StickyBoard runs as four coordinated binaries:
1. `stickyboard`: The unified CLI wrapper.
2. `stickyboard-daemon`: Background supervisor process that manages database records and spawns note windows.
3. `stickyboard-capture`: Quick-entry text popup window.
4. `stickyboard-note`: Individual floating note window process (one process per note for process isolation).

---

## System Requirements

Ensure the following packages are installed on your Arch system:
```bash
sudo pacman -S rust gtk4 sqlite
```

---

## Build Instructions

To build all StickyBoard binaries:
```bash
cargo build --release
```
This produces the following binaries in `target/release/`:
- `stickyboard`
- `stickyboard-daemon`
- `stickyboard-capture`
- `stickyboard-note`

---

## Installation & Setup

### 1. Install Binaries
#### Option A: Build from Source
Run the build-from-source installation script to compile, install binaries, fonts, window rules, keybindings, and start the service:
```bash
./build-from-source.sh
```
Or compile manually:
```bash
cargo build --release
mkdir -p ~/.local/bin
cp target/release/stickyboard target/release/stickyboard-daemon target/release/stickyboard-capture target/release/stickyboard-note ~/.local/bin/
```
Alternatively, customize the provided `PKGBUILD.example` to build and install StickyBoard as an Arch package.

#### Option B: Download Prebuilt Binaries (One-Liner Install)
You can download the latest prebuilt release, install all binaries, configurations, systemd services, fonts, and registers hotkeys automatically:
```bash
curl -sL https://raw.githubusercontent.com/mirarr-app/StickyBoard/main/install.sh | bash
```

Alternatively, to install manually:
1. Download the release archive `stickyboard-linux-x86_64.tar.gz` from the [GitHub Releases Page](https://github.com/mirarr-app/StickyBoard/releases).
2. Extract the archive and copy components:
   ```bash
   tar -xzvf stickyboard-linux-x86_64.tar.gz
   mkdir -p ~/.local/bin
   cp stickyboard stickyboard-daemon stickyboard-capture stickyboard-note ~/.local/bin/

   # Install the bundled default font
   mkdir -p ~/.local/share/fonts
   cp Excalifont-Regular.ttf ~/.local/share/fonts/
   fc-cache -f ~/.local/share/fonts/
   ```

### 2. Configure Systemd User Service
Install and enable the Systemd user service so the daemon starts automatically with your session:
```bash
mkdir -p ~/.config/systemd/user/
cp stickyboard.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now stickyboard.service
```

### 3. Add Hyprland Configurations
1. Append the window rules from [hyprland.lua.example](file:///home/parsa/Work/osticky/hyprland.lua.example) to your `~/.config/hypr/autostart.lua` (directs note windows to workspace 6, makes them floating, and configures the capture popup).
2. Add the global hotkey binding to your Omarchy keybindings configuration file at `~/.config/hypr/bindings.lua`:
   ```lua
   o.bind("SUPER + SHIFT + K", "Launch StickyBoard Capture", "stickyboard-capture")
   ```

Run `hyprctl reload` to apply rules.

---

## Command Line Interface (CLI)

The `stickyboard` binary controls all operations:

- **Launch background daemon**: `stickyboard daemon`
- **Launch capture window**: `stickyboard capture`
- **Show/Restore all note windows**: `stickyboard show`
- **Hide all note windows**: `stickyboard hide`
- **Reload notes**: `stickyboard reload`
- **Add a new note**:
  - Arguments: `stickyboard new --text "My text"`
  - Stdin stream: `echo "Clean my room" | stickyboard new`
- **List notes**: `stickyboard list`
- **Export notes**: `stickyboard export notes.json`
- **Import notes**: `stickyboard import notes.json`

---

## File & Socket Paths

- **Database**: `~/.local/share/stickyboard/notes.db`
- **IPC Unix Domain Socket**: `/run/user/<uid>/stickyboard.sock` (session-bound)
- **Local Logs**: Systemd journal logs (view using `journalctl --user -u stickyboard.service`)

---

## Uninstallation

To uninstall StickyBoard:
```bash
./uninstall.sh
```
To also purge the notes database and app data directory:
```bash
./uninstall.sh --purge
```

---

## Credits

- **Excalifont**: The default handwriting style font of the application. Designed and provided by the [Excalidraw team](https://plus.excalidraw.com/excalifont).
