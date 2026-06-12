use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::config::get_socket_path;
use crate::models::Note;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum IpcRequest {
    NewNote {
        text: String,
        color: String,
        pos_x: i32,
        pos_y: i32,
        width: i32,
        height: i32,
    },
    DeleteNote {
        id: i64,
        from_window: bool,
    },
    UpdateNote {
        id: i64,
        text: String,
        color: String,
        pos_x: i32,
        pos_y: i32,
        width: i32,
        height: i32,
    },
    GetNote {
        id: i64,
    },
    ShowAll,
    HideAll,
    Reload,
    ListNotes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum IpcResponse {
    Ok,
    NewNoteCreated { id: i64 },
    NoteDetails { note: Note },
    NotesList { notes: Vec<Note> },
    Error { message: String },
}

/// Sends an IPC request to the daemon socket synchronously (blocking).
pub fn send_ipc_request(req: &IpcRequest) -> std::io::Result<IpcResponse> {
    let socket_path = get_socket_path();
    let mut stream = UnixStream::connect(&socket_path).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!(
                "Daemon is not running or socket is unavailable ({}). Error: {}",
                socket_path.display(),
                e
            ),
        )
    })?;

    // Set a timeout to prevent hanging forever
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    // Serialize request to JSON and append newline
    let req_json = serde_json::to_string(req)?;
    stream.write_all(req_json.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    // Read response line
    let mut reader = BufReader::new(stream);
    let mut resp_json = String::new();
    reader.read_line(&mut resp_json)?;

    if resp_json.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Received empty response from daemon.",
        ));
    }

    // Deserialize response
    let response: IpcResponse = serde_json::from_str(&resp_json)?;
    Ok(response)
}

/// Sends an IPC request to the daemon socket asynchronously.
pub async fn send_ipc_request_async(req: &IpcRequest) -> std::io::Result<IpcResponse> {
    use tokio::net::UnixStream as AsyncUnixStream;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};

    let socket_path = get_socket_path();
    let mut stream = AsyncUnixStream::connect(&socket_path).await.map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            format!(
                "Daemon is not running or socket is unavailable ({}). Error: {}",
                socket_path.display(),
                e
            ),
        )
    })?;

    let req_json = serde_json::to_string(req)?;
    stream.write_all(req_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let mut reader = AsyncBufReader::new(stream);
    let mut resp_json = String::new();
    reader.read_line(&mut resp_json).await?;

    if resp_json.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Received empty response from daemon.",
        ));
    }

    let response: IpcResponse = serde_json::from_str(&resp_json)?;
    Ok(response)
}
