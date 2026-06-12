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
