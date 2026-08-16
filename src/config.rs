use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;

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

/// Ensures that the custom font Excalifont-Regular.ttf is copied to the user's local fonts directory
/// and cached with fontconfig so that GTK can load it.
pub fn ensure_font_installed() {
    let font_name = "Excalifont-Regular.ttf";
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

/// Helper to convert hex colors like #ffffff or #fff to rgba format
pub fn hex_to_rgba(hex: &str, alpha: f32) -> String {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return format!("rgba({}, {}, {}, {})", r, g, b, alpha);
        }
    } else if clean.len() == 3 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..1], 16),
            u8::from_str_radix(&clean[1..2], 16),
            u8::from_str_radix(&clean[2..3], 16),
        ) {
            return format!("rgba({}, {}, {}, {})", r * 17, g * 17, b * 17, alpha);
        }
    }
    format!("rgba(0, 0, 0, {})", alpha)
}

fn parse_hex_color(hex: &str) -> Option<(f32, f32, f32)> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()? as f32 / 255.0;
        Some((r, g, b))
    } else if clean.len() == 3 {
        let r = u8::from_str_radix(&clean[0..1], 16).ok()? as f32 / 15.0;
        let g = u8::from_str_radix(&clean[1..2], 16).ok()? as f32 / 15.0;
        let b = u8::from_str_radix(&clean[2..3], 16).ok()? as f32 / 15.0;
        Some((r, g, b))
    } else {
        None
    }
}

fn is_light_color(hex: &str) -> bool {
    if let Some((r, g, b)) = parse_hex_color(hex) {
        let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        l > 0.5
    } else {
        true
    }
}

pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub yellow_bg: String,
    pub yellow_fg: String,
    pub blue_bg: String,
    pub blue_fg: String,
    pub green_bg: String,
    pub green_fg: String,
    pub pink_bg: String,
    pub pink_fg: String,
    pub orange_bg: String,
    pub orange_fg: String,
}

impl ThemeColors {
    pub fn default_colors() -> Self {
        Self {
            background: "#1e1e2e".to_string(),
            foreground: "#cdd6f4".to_string(),
            accent: "#cba6f7".to_string(),
            yellow_bg: "#fef3c7".to_string(),
            yellow_fg: "#78350f".to_string(),
            blue_bg: "#dbeafe".to_string(),
            blue_fg: "#1e3a8a".to_string(),
            green_bg: "#d1fae5".to_string(),
            green_fg: "#065f46".to_string(),
            pink_bg: "#fce7f3".to_string(),
            pink_fg: "#831843".to_string(),
            orange_bg: "#ffedd5".to_string(),
            orange_fg: "#7c2d12".to_string(),
        }
    }

    /// Resolves the theme file path given a home directory path.
    /// Checks `<home>/.local/state/omarchy/current/theme/colors.toml` first (new Omarchy location),
    /// and falls back to `<home>/.config/omarchy/current/theme/colors.toml` (legacy location).
    pub fn resolve_theme_path(home: &std::path::Path) -> PathBuf {
        let new_path = home
            .join(".local")
            .join("state")
            .join("omarchy")
            .join("current")
            .join("theme")
            .join("colors.toml");

        if new_path.exists() {
            return new_path;
        }

        let old_path = home
            .join(".config")
            .join("omarchy")
            .join("current")
            .join("theme")
            .join("colors.toml");

        if old_path.exists() {
            return old_path;
        }

        new_path
    }

    /// Returns the path to the Omarchy theme colors file.
    /// Checks `~/.local/state/omarchy/current/theme/colors.toml` first (new Omarchy location),
    /// and falls back to `~/.config/omarchy/current/theme/colors.toml` (legacy location).
    pub fn theme_path() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        if home.is_empty() {
            return None;
        }
        Some(Self::resolve_theme_path(std::path::Path::new(&home)))
    }

    pub fn parse(content: &str) -> Self {
        let mut colors = Self::default_colors();
        let mut map = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }
            if let Some(pos) = trimmed.find('=') {
                let key = trimmed[..pos].trim();
                let val = trimmed[pos + 1..].trim().trim_matches('"').trim_matches('\'').trim();
                map.insert(key.to_string(), val.to_string());
            }
        }

        if let Some(bg) = map.get("background") {
            colors.background = bg.clone();
        }
        if let Some(fg) = map.get("foreground") {
            colors.foreground = fg.clone();
        }
        if let Some(accent) = map.get("accent") {
            colors.accent = accent.clone();
        }

        // Map theme colors to note colors (supporting both named colors and ANSI indices)
        let yellow_bg = map
            .get("yellow")
            .or_else(|| map.get("bright_yellow"))
            .or_else(|| map.get("color3"))
            .or_else(|| map.get("color11"))
            .cloned()
            .unwrap_or_else(|| "#fef3c7".to_string());
        let yellow_fg = if is_light_color(&yellow_bg) { "#1a1a1a".to_string() } else { "#f8f9fa".to_string() };
        colors.yellow_bg = yellow_bg;
        colors.yellow_fg = yellow_fg;

        let blue_bg = map
            .get("blue")
            .or_else(|| map.get("bright_blue"))
            .or_else(|| map.get("color4"))
            .or_else(|| map.get("color12"))
            .cloned()
            .unwrap_or_else(|| "#dbeafe".to_string());
        let blue_fg = if is_light_color(&blue_bg) { "#1a1a1a".to_string() } else { "#f8f9fa".to_string() };
        colors.blue_bg = blue_bg;
        colors.blue_fg = blue_fg;

        let green_bg = map
            .get("green")
            .or_else(|| map.get("bright_green"))
            .or_else(|| map.get("color2"))
            .or_else(|| map.get("color10"))
            .cloned()
            .unwrap_or_else(|| "#d1fae5".to_string());
        let green_fg = if is_light_color(&green_bg) { "#1a1a1a".to_string() } else { "#f8f9fa".to_string() };
        colors.green_bg = green_bg;
        colors.green_fg = green_fg;

        let pink_bg = map
            .get("pink")
            .or_else(|| map.get("magenta"))
            .or_else(|| map.get("bright_magenta"))
            .or_else(|| map.get("color5"))
            .or_else(|| map.get("color13"))
            .cloned()
            .unwrap_or_else(|| "#fce7f3".to_string());
        let pink_fg = if is_light_color(&pink_bg) { "#1a1a1a".to_string() } else { "#f8f9fa".to_string() };
        colors.pink_bg = pink_bg;
        colors.pink_fg = pink_fg;

        let orange_bg = map
            .get("orange")
            .or_else(|| map.get("red"))
            .or_else(|| map.get("bright_orange"))
            .or_else(|| map.get("bright_red"))
            .or_else(|| map.get("color1"))
            .or_else(|| map.get("color9"))
            .cloned()
            .unwrap_or_else(|| "#ffedd5".to_string());
        let orange_fg = if is_light_color(&orange_bg) { "#1a1a1a".to_string() } else { "#f8f9fa".to_string() };
        colors.orange_bg = orange_bg;
        colors.orange_fg = orange_fg;

        colors
    }

    pub fn load() -> Self {
        let toml_path = match Self::theme_path() {
            Some(p) => p,
            None => return Self::default_colors(),
        };

        if !toml_path.exists() {
            return Self::default_colors();
        }

        if let Ok(content) = fs::read_to_string(&toml_path) {
            Self::parse(&content)
        } else {
            Self::default_colors()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors_parse_named() {
        let sample = r##"
            background = "#000000"
            foreground = "#ffffff"
            accent = "#8d8d8d"
            yellow = "#cecece"
            blue = "#8d8d8d"
            green = "#b6b6b6"
            magenta = "#9b9b9b"
            orange = "#b9b9b9"
        "##;
        let colors = ThemeColors::parse(sample);
        assert_eq!(colors.background, "#000000");
        assert_eq!(colors.foreground, "#ffffff");
        assert_eq!(colors.accent, "#8d8d8d");
        assert_eq!(colors.yellow_bg, "#cecece");
        assert_eq!(colors.blue_bg, "#8d8d8d");
        assert_eq!(colors.green_bg, "#b6b6b6");
        assert_eq!(colors.pink_bg, "#9b9b9b");
        assert_eq!(colors.orange_bg, "#b9b9b9");
    }

    #[test]
    fn test_theme_colors_parse_ansi() {
        let sample = r##"
            background = "#000000"
            foreground = "#ffffff"
            accent = "#ff00ff"
            color3 = "#ffff00"
            color4 = "#0000ff"
        "##;
        let colors = ThemeColors::parse(sample);
        assert_eq!(colors.background, "#000000");
        assert_eq!(colors.foreground, "#ffffff");
        assert_eq!(colors.accent, "#ff00ff");
        assert_eq!(colors.yellow_bg, "#ffff00");
        assert_eq!(colors.blue_bg, "#0000ff");
    }

    #[test]
    fn test_theme_path_resolution() {
        if let Some(path) = ThemeColors::theme_path() {
            assert!(path.ends_with("colors.toml"));
        }
    }

    #[test]
    fn test_theme_path_fallback_logic() {
        let temp_dir = std::env::temp_dir().join(format!("osticky_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);

        let new_theme_dir = temp_dir.join(".local").join("state").join("omarchy").join("current").join("theme");
        let old_theme_dir = temp_dir.join(".config").join("omarchy").join("current").join("theme");

        let new_file = new_theme_dir.join("colors.toml");
        let old_file = old_theme_dir.join("colors.toml");

        // 1. Neither exists -> returns new path by default
        assert_eq!(ThemeColors::resolve_theme_path(&temp_dir), new_file);

        // 2. Only legacy path exists -> fallback to old path
        fs::create_dir_all(&old_theme_dir).unwrap();
        fs::write(&old_file, "background = '#111111'").unwrap();
        assert_eq!(ThemeColors::resolve_theme_path(&temp_dir), old_file);

        // 3. Both exist -> prefers new path
        fs::create_dir_all(&new_theme_dir).unwrap();
        fs::write(&new_file, "background = '#222222'").unwrap();
        assert_eq!(ThemeColors::resolve_theme_path(&temp_dir), new_file);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
