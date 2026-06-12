# StickyBoard

StickyBoard is a lightweight sticky notes system designed specifically for Omarchy. It transforms a Hyprland workspace (6 by default) into a persistent corkboard of floating sticky notes.

StickyBoard is built using GTK4, SQLite, Tokio, and native Hyprland IPC.

---

## Features

- **Instant Note Capture**: Press `SUPER+SHIFT+K` to open a centered input popup, type your note, and hit `Enter` to spawn it instantly.
- **Dedicated Corkboard**: Automatically routes and manages all note windows on workspace 6.
- **Geometry Recovery**: Changes to note positions and sizes are autosaved and survive system reboots or Hyprland/daemon restarts. Notes are read-only and static once created to keep workspace 6 clean.
- **Omarchy System Theme Integration**: Note colors are loaded dynamically from your active Omarchy system theme (`~/.config/omarchy/current/theme/colors.toml`) and hot-reload instantly when the theme is changed.
- **Custom App Font**: Bundles and automatically installs the handwriting-style `Excalifont` (`Excalifont-Regular.ttf`) for premium, state-of-the-art aesthetics.
- **Minimalist Design**: Zero title bars or menu bars, transparent backgrounds, rounded corners, and smooth interactions.

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
Compile the binaries and copy them manually:
```bash
cargo build --release
sudo cp target/release/stickyboard* /usr/bin/
```
Alternatively, customize the provided `PKGBUILD.example` to build and install StickyBoard as an Arch package.

#### Option B: Download Prebuilt Binaries
If you prefer not to compile from source, download the latest release archive (`stickyboard-linux-x86_64.tar.gz`) from GitHub Releases.

Decompress the archive and copy the binaries to `/usr/bin/`:
```bash
tar -xzvf stickyboard-linux-x86_64.tar.gz
sudo cp stickyboard stickyboard-daemon stickyboard-capture stickyboard-note /usr/bin/

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
1. Append the window rules from [hyprland.conf.example](file:///home/parsa/Work/osticky/hyprland.conf.example) to your `~/.config/hypr/hyprland.conf` (directs note windows to workspace 6, makes them floating, and configures the capture popup).
2. Add the global hotkey binding to your Omarchy keybindings configuration file at `~/.config/hypr/bindings.conf`:
   ```hyprland
   bindd = SUPER SHIFT, K, Launch StickyBoard Capture, exec, stickyboard capture
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

## Credits

- **Excalifont**: The default handwriting style font of the application. Designed and provided by the [Excalidraw team](https://plus.excalidraw.com/excalifont).
