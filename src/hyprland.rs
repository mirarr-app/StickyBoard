use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct HyprClient {
    pub address: String,
    pub title: String,
    pub class: String,
    pub at: [i32; 2],
    pub size: [i32; 2],
    pub workspace: HyprWorkspaceRef,
}

#[derive(Deserialize, Debug)]
pub struct HyprWorkspaceRef {
    pub id: i32,
    pub name: String,
}

/// Sends a command to the Hyprland command IPC socket (blocking).
pub fn send_hyprland_cmd(cmd: &str) -> std::io::Result<String> {
    let xdg_runtime = env::var("XDG_RUNTIME_DIR")
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "XDG_RUNTIME_DIR not set"))?;
    let hypr_sig = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "HYPRLAND_INSTANCE_SIGNATURE not set"))?;
    let socket_path = format!("{}/hypr/{}/.socket.sock", xdg_runtime, hypr_sig);
    let mut stream = UnixStream::connect(socket_path)?;
    
    // Command must be formatted properly for Hyprland IPC
    // Typically, commands are prefix-less raw strings, but some require specific trailing formats.
    // We write the command to the stream.
    stream.write_all(cmd.as_bytes())?;
    
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

/// Sends a command to the Hyprland command IPC socket (async).
pub async fn send_hyprland_cmd_async(cmd: &str) -> std::io::Result<String> {
    use tokio::net::UnixStream as AsyncUnixStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let xdg_runtime = env::var("XDG_RUNTIME_DIR")
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "XDG_RUNTIME_DIR not set"))?;
    let hypr_sig = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "HYPRLAND_INSTANCE_SIGNATURE not set"))?;
    let socket_path = format!("{}/hypr/{}/.socket.sock", xdg_runtime, hypr_sig);
    
    let mut stream = AsyncUnixStream::connect(socket_path).await?;
    stream.write_all(cmd.as_bytes()).await?;
    
    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    Ok(response)
}

/// Lists all active clients from Hyprland.
pub fn list_clients() -> std::io::Result<Vec<HyprClient>> {
    // "j/clients" fetches clients in JSON format
    let clients_json = send_hyprland_cmd("j/clients")?;
    let clients: Vec<HyprClient> = serde_json::from_str(&clients_json)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse Hyprland clients JSON: {}", e)))?;
    Ok(clients)
}

/// Finds a client window by its exact title.
pub fn find_client_by_title(title: &str) -> std::io::Result<Option<HyprClient>> {
    let clients = list_clients()?;
    for client in clients {
        if client.title == title {
            return Ok(Some(client));
        }
    }
    Ok(None)
}

/// Positions a note window by its ID using Hyprland IPC.
/// Returns Ok(true) if the window was found and positioned, Ok(false) if not found.
pub fn position_note_window(id: i64, x: i32, y: i32, w: i32, h: i32) -> std::io::Result<bool> {
    let title = format!("stickyboard-note-{}", id);
    if let Some(client) = find_client_by_title(&title)? {
        let address = client.address;
        // Move floating window to exact X Y
        let move_cmd = format!("dispatch movewindowpixel exact {} {},address:{}", x, y, address);
        // Resize floating window to exact W H
        let resize_cmd = format!("dispatch resizewindowpixel exact {} {},address:{}", w, h, address);
        
        send_hyprland_cmd(&move_cmd)?;
        send_hyprland_cmd(&resize_cmd)?;
        
        // Also ensure it is pinned (visible on all workspaces, or pinned in place)
        // Note: the pin dispatcher toggles pin state or sets it.
        // We rely on the window rule windowrulev2 = pin,class:^(stickyboard-note)$
        // but can double enforce it if needed.
        
        Ok(true)
    } else {
        Ok(false)
    }
}
