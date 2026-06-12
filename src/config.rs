use std::path::PathBuf;
use std::fs;

pub const APP_NAME: &str = "stickyboard";
pub const DEFAULT_COLOR: &str = "yellow";
pub const DEFAULT_WIDTH: i32 = 300;
pub const DEFAULT_HEIGHT: i32 = 250;

/// Returns the path to the app's local data directory (typically ~/.local/share/stickyboard)
pub fn get_app_dir() -> PathBuf {
    let mut dir = dirs::data_dir().unwrap_or_else(|| {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
            .join(".local")
            .join("share")
    });
    dir.push(APP_NAME);
    // Ensure the directory exists
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir
}

/// Returns the path to the SQLite database
pub fn get_db_path() -> PathBuf {
    get_app_dir().join("notes.db")
}

/// Returns the path to the IPC Unix socket
pub fn get_socket_path() -> PathBuf {
    if let Some(runtime_dir) = dirs::runtime_dir() {
        let sock_path = runtime_dir.join(format!("{}.sock", APP_NAME));
        return sock_path;
    }
    // Fallback to app directory
    get_app_dir().join(format!("{}.sock", APP_NAME))
}

/// Finds the path to a sibling binary or falls back to system PATH.
pub fn find_bin_path(bin_name: &str) -> PathBuf {
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let local_bin = parent.join(bin_name);
            if local_bin.exists() {
                return local_bin;
            }
        }
    }
    PathBuf::from(bin_name)
}

/// Ensures that the custom font excalifont.woff2 is copied to the user's local fonts directory
/// and cached with fontconfig so that GTK can load it.
pub fn ensure_font_installed() {
    let font_name = "excalifont.woff2";
    let user_fonts_dir = dirs::data_dir()
        .map(|d| d.join("fonts"))
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
                .join(".local")
                .join("share")
                .join("fonts")
        });

    let target_font_path = user_fonts_dir.join(font_name);
    if target_font_path.exists() {
        return;
    }

    // Try to find the font file to copy it
    let mut font_source: Option<PathBuf> = None;

    // 1. Check in current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join(font_name);
        if p.exists() {
            font_source = Some(p);
        }
    }

    // 2. Check relative to executable path
    if font_source.is_none() {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let p = parent.join(font_name);
                if p.exists() {
                    font_source = Some(p);
                } else if let Some(grandparent) = parent.parent() {
                    let p = grandparent.join(font_name);
                    if p.exists() {
                        font_source = Some(p);
                    } else if let Some(great_grandparent) = grandparent.parent() {
                        let p = great_grandparent.join(font_name);
                        if p.exists() {
                            font_source = Some(p);
                        }
                    }
                }
            }
        }
    }

    if let Some(src) = font_source {
        if let Err(e) = fs::create_dir_all(&user_fonts_dir) {
            eprintln!("Failed to create user fonts directory: {:?}", e);
            return;
        }
        if let Err(e) = fs::copy(&src, &target_font_path) {
            eprintln!("Failed to copy font to user fonts directory: {:?}", e);
            return;
        }
        // Run fc-cache -f on the user fonts directory
        let _ = std::process::Command::new("fc-cache")
            .arg("-f")
            .arg(user_fonts_dir.to_string_lossy().as_ref())
            .status();
    }
}

