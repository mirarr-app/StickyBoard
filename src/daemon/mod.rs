use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tokio::sync::oneshot::Sender as OneshotSender;

use crate::config::{find_bin_path, get_socket_path};
use crate::db::Db;
use crate::ipc::{IpcRequest, IpcResponse};
use crate::{log_error, log_info};

pub struct DaemonState {
    db: Arc<Mutex<Db>>,
    running_notes: Arc<Mutex<HashMap<i64, OneshotSender<()>>>>,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    log_info!("Starting StickyBoard daemon...");
    crate::config::ensure_font_installed();

    // Initialize Database
    let db = match Db::open() {
        Ok(database) => Arc::new(Mutex::new(database)),
        Err(e) => {
            log_error!("Failed to open SQLite database: {}", e);
            return Err(e.into());
        }
    };

    let running_notes = Arc::new(Mutex::new(HashMap::new()));
    let state = Arc::new(DaemonState {
        db,
        running_notes,
    });

    // Clean up socket file from previous run if it exists
    let socket_path = get_socket_path();
    if socket_path.exists() {
        log_info!("Cleaning up stale socket file: {}", socket_path.display());
        let _ = fs::remove_file(&socket_path);
    }

    // Bind Unix Socket
    let listener = UnixListener::bind(&socket_path)?;
    log_info!("Listening on Unix socket: {}", socket_path.display());

    // Spawn existing notes on startup
    if let Err(e) = spawn_all_notes(state.clone()).await {
        log_error!("Failed to spawn notes on startup: {}", e);
    }

    // Accept connections loop
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, state_clone).await {
                        log_error!("Error handling connection: {}", e);
                    }
                });
            }
            Err(e) => {
                log_error!("Failed to accept Unix connection: {}", e);
            }
        }
    }
}

async fn handle_connection(
    stream: UnixStream,
    state: Arc<DaemonState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = tokio::io::split(stream);
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    // Read the IPC request (single line)
    buf_reader.read_line(&mut line).await?;
    if line.is_empty() {
        return Ok(());
    }

    // Deserialize request
    let request: IpcRequest = match serde_json::from_str(&line) {
        Ok(req) => req,
        Err(e) => {
            let resp = IpcResponse::Error {
                message: format!("Invalid JSON request: {}", e),
            };
            let resp_json = serde_json::to_string(&resp)?;
            writer.write_all(resp_json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            return Ok(());
        }
    };

    // Process request
    let response = process_request(request, state).await;

    // Serialize and write response
    let resp_json = serde_json::to_string(&response)?;
    writer.write_all(resp_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    Ok(())
}

async fn process_request(request: IpcRequest, state: Arc<DaemonState>) -> IpcResponse {
    match request {
        IpcRequest::NewNote {
            text,
            color: _,
            pos_x,
            pos_y,
            width,
            height,
        } => {
            let mut db = state.db.lock().await;

            // Generate a random color from the list
            let colors = ["yellow", "blue", "green", "pink", "orange"];
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_micros())
                .unwrap_or(0);
            let final_color = colors[(seed % colors.len() as u128) as usize];

            // If coordinates are default (e.g. 100, 100), we offset based on count
            let (mut final_x, mut final_y) = (pos_x, pos_y);
            if pos_x == 100 && pos_y == 100 {
                if let Ok(notes) = db.list_notes() {
                    let count = notes.len();
                    final_x = 100 + ((count as i32) % 5) * 40;
                    final_y = 100 + ((count as i32) % 5) * 40;
                }
            }

            match db.create_note(&text, final_color, final_x, final_y, width, height) {
                Ok(id) => {
                    log_info!("Created note id={}", id);
                    // Spawn the note window process
                    if let Err(e) = spawn_note_process(&state, id) {
                        log_error!("Failed to spawn process for new note id={}: {}", id, e);
                        IpcResponse::Error {
                            message: format!("Note created in DB, but failed to spawn window: {}", e),
                        }
                    } else {
                        IpcResponse::NewNoteCreated { id }
                    }
                }
                Err(e) => {
                    log_error!("Failed to save note to DB: {}", e);
                    IpcResponse::Error {
                        message: format!("DB Error: {}", e),
                    }
                }
            }
        }
        IpcRequest::DeleteNote { id, from_window } => {
            let mut db = state.db.lock().await;
            match db.delete_note(id) {
                Ok(_) => {
                    log_info!("Deleted note id={} (from_window={})", id, from_window);
                    // Kill the child window process only if it is not the window itself closing
                    let mut running = state.running_notes.lock().await;
                    if let Some(kill_tx) = running.remove(&id) {
                        if !from_window {
                            let _ = kill_tx.send(());
                        }
                    }
                    IpcResponse::Ok
                }
                Err(e) => {
                    log_error!("Failed to delete note id={} from DB: {}", id, e);
                    IpcResponse::Error {
                        message: format!("DB Error: {}", e),
                    }
                }
            }
        }
        IpcRequest::UpdateNote {
            id,
            text,
            color,
            pos_x,
            pos_y,
            width,
            height,
        } => {
            let mut db = state.db.lock().await;
            match db.update_note(id, &text, &color, pos_x, pos_y, width, height) {
                Ok(_) => {
                    log_info!("Updated note id={} in DB", id);
                    IpcResponse::Ok
                }
                Err(e) => {
                    log_error!("Failed to update note id={} in DB: {}", id, e);
                    IpcResponse::Error {
                        message: format!("DB Error: {}", e),
                    }
                }
            }
        }
        IpcRequest::GetNote { id } => {
            let db = state.db.lock().await;
            match db.get_note(id) {
                Ok(note) => IpcResponse::NoteDetails { note },
                Err(e) => {
                    log_error!("Failed to fetch note id={} from DB: {}", id, e);
                    IpcResponse::Error {
                        message: format!("DB Error: {}", e),
                    }
                }
            }
        }
        IpcRequest::ShowAll => {
            match spawn_all_notes(state).await {
                Ok(_) => IpcResponse::Ok,
                Err(e) => IpcResponse::Error {
                    message: format!("Failed to show all notes: {}", e),
                },
            }
        }
        IpcRequest::HideAll => {
            let mut running = state.running_notes.lock().await;
            log_info!("Killing all {} running note processes", running.len());
            for (_, kill_tx) in running.drain() {
                let _ = kill_tx.send(());
            }
            IpcResponse::Ok
        }
        IpcRequest::Reload => {
            // Kill all running notes first
            {
                let mut running = state.running_notes.lock().await;
                log_info!("Reloading: killing {} running note processes", running.len());
                for (_, kill_tx) in running.drain() {
                    let _ = kill_tx.send(());
                }
            }
            // Spawn them fresh
            match spawn_all_notes(state).await {
                Ok(_) => IpcResponse::Ok,
                Err(e) => IpcResponse::Error {
                    message: format!("Failed to spawn notes on reload: {}", e),
                },
            }
        }
        IpcRequest::ListNotes => {
            let db = state.db.lock().await;
            match db.list_notes() {
                Ok(notes) => IpcResponse::NotesList { notes },
                Err(e) => IpcResponse::Error {
                    message: format!("DB Error: {}", e),
                },
            }
        }
    }
}

async fn spawn_all_notes(state: Arc<DaemonState>) -> Result<(), Box<dyn std::error::Error>> {
    let db = state.db.lock().await;
    let notes = db.list_notes()?;
    log_info!("Found {} notes in database to spawn", notes.len());

    let mut running = state.running_notes.lock().await;
    for note in notes {
        if !running.contains_key(&note.id) {
            match spawn_note_process_internal(note.id) {
                Ok(mut child) => {
                    let note_id = note.id;
                    let (kill_tx, kill_rx) = tokio::sync::oneshot::channel::<()>();
                    running.insert(note_id, kill_tx);
                    
                    let running_notes_clone = state.running_notes.clone();
                    
                    // Spawn supervisor task
                    tokio::spawn(async move {
                        tokio::select! {
                            _ = child.wait() => {
                                log_info!("Note window process id={} exited", note_id);
                            }
                            _ = kill_rx => {
                                log_info!("Note window process id={} kill requested", note_id);
                                let _ = child.kill().await;
                            }
                        }
                        let mut map = running_notes_clone.lock().await;
                        map.remove(&note_id);
                    });
                }
                Err(e) => {
                    log_error!("Failed to spawn note process for id={}: {}", note.id, e);
                }
            }
        }
    }
    Ok(())
}

fn spawn_note_process(state: &Arc<DaemonState>, id: i64) -> std::io::Result<()> {
    let mut child = spawn_note_process_internal(id)?;
    let (kill_tx, kill_rx) = tokio::sync::oneshot::channel::<()>();
    let running_notes_clone = state.running_notes.clone();
    
    // Insert into map inside a task to avoid blocking sync function
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut map = state_clone.running_notes.lock().await;
        map.insert(id, kill_tx);
        
        tokio::spawn(async move {
            tokio::select! {
                _ = child.wait() => {
                    log_info!("Note window process id={} exited", id);
                }
                _ = kill_rx => {
                    log_info!("Note window process id={} kill requested", id);
                    let _ = child.kill().await;
                }
            }
            let mut map = running_notes_clone.lock().await;
            map.remove(&id);
        });
    });

    Ok(())
}

fn spawn_note_process_internal(id: i64) -> std::io::Result<tokio::process::Child> {
    let bin_path = find_bin_path("stickyboard-note");
    log_info!("Spawning note process: {} --id {}", bin_path.display(), id);
    
    tokio::process::Command::new(bin_path)
        .arg("--id")
        .arg(id.to_string())
        .spawn()
}
