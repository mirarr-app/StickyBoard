use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read};
use serde::Deserialize;

use stickyboard::config::{DEFAULT_COLOR, DEFAULT_HEIGHT, DEFAULT_WIDTH};
use stickyboard::ipc::{send_ipc_request, IpcRequest, IpcResponse};

#[derive(Parser)]
#[command(
    name = "stickyboard",
    about = "StickyBoard - Corkboard sticky notes system for Hyprland",
    version = env!("CARGO_PKG_VERSION")
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the background daemon
    Daemon,

    /// Open the note capture popup
    Capture,

    /// Show all sticky notes (spawns note windows)
    Show,

    /// Hide all sticky notes (terminates note windows)
    Hide,

    /// Reload all sticky notes from database
    Reload,

    /// Create a new sticky note
    New {
        /// Text content of the note (if omitted, reads from stdin)
        #[arg(short, long)]
        text: Option<String>,

        /// Color of the note (yellow, blue, green, pink, orange)
        #[arg(short, long, default_value = "yellow")]
        color: String,
    },

    /// List all sticky notes
    List,

    /// Export sticky notes to a JSON file
    Export {
        /// Target JSON file path
        path: String,
    },

    /// Import sticky notes from a JSON file
    Import {
        /// Source JSON file path
        path: String,
    },
}

#[derive(Deserialize)]
struct ImportNote {
    text: String,
    color: Option<String>,
    pos_x: Option<i32>,
    pos_y: Option<i32>,
    width: Option<i32>,
    height: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon => {
            stickyboard::daemon::run().await?;
        }
        Commands::Capture => {
            stickyboard::capture::run();
        }
        Commands::Show => {
            match send_ipc_request(&IpcRequest::ShowAll) {
                Ok(IpcResponse::Ok) => println!("Success: All notes spawned/restored."),
                Ok(IpcResponse::Error { message }) => eprintln!("Error from daemon: {}", message),
                Err(e) => eprintln!("Error connecting to daemon: {}", e),
                _ => eprintln!("Unexpected response from daemon."),
            }
        }
        Commands::Hide => {
            match send_ipc_request(&IpcRequest::HideAll) {
                Ok(IpcResponse::Ok) => println!("Success: All notes hidden."),
                Ok(IpcResponse::Error { message }) => eprintln!("Error from daemon: {}", message),
                Err(e) => eprintln!("Error connecting to daemon: {}", e),
                _ => eprintln!("Unexpected response from daemon."),
            }
        }
        Commands::Reload => {
            match send_ipc_request(&IpcRequest::Reload) {
                Ok(IpcResponse::Ok) => println!("Success: Notes reloaded."),
                Ok(IpcResponse::Error { message }) => eprintln!("Error from daemon: {}", message),
                Err(e) => eprintln!("Error connecting to daemon: {}", e),
                _ => eprintln!("Unexpected response from daemon."),
            }
        }
        Commands::New { text, color } => {
            let note_text = match text {
                Some(t) => t,
                None => {
                    let mut buffer = String::new();
                    io::stdin().read_to_string(&mut buffer)?;
                    let trimmed = buffer.trim().to_string();
                    if trimmed.is_empty() {
                        eprintln!("Error: No text provided via --text argument or stdin.");
                        std::process::exit(1);
                    }
                    trimmed
                }
            };

            let req = IpcRequest::NewNote {
                text: note_text,
                color,
                pos_x: 100, // Daemon will cascade offset
                pos_y: 100,
                width: DEFAULT_WIDTH,
                height: DEFAULT_HEIGHT,
            };

            match send_ipc_request(&req) {
                Ok(IpcResponse::NewNoteCreated { id }) => println!("Success: Created note with ID {}.", id),
                Ok(IpcResponse::Error { message }) => eprintln!("Error from daemon: {}", message),
                Err(e) => eprintln!("Error connecting to daemon: {}", e),
                _ => eprintln!("Unexpected response from daemon."),
            }
        }
        Commands::List => {
            match send_ipc_request(&IpcRequest::ListNotes) {
                Ok(IpcResponse::NotesList { notes }) => {
                    if notes.is_empty() {
                        println!("No notes found.");
                    } else {
                        println!("{:<6} {:<10} {:<15} {}", "ID", "COLOR", "GEOMETRY", "TEXT");
                        println!("{}", "-".repeat(60));
                        for note in notes {
                            let geom = format!("({},{}) {}x{}", note.pos_x, note.pos_y, note.width, note.height);
                            // Truncate text preview to fit single line
                            let preview = note.text.replace('\n', " ");
                            let truncated = if preview.chars().count() > 30 {
                                format!("{}...", preview.chars().take(27).collect::<String>())
                            } else {
                                preview
                            };
                            println!("{:<6} {:<10} {:<15} {}", note.id, note.color, geom, truncated);
                        }
                    }
                }
                Ok(IpcResponse::Error { message }) => eprintln!("Error from daemon: {}", message),
                Err(e) => eprintln!("Error connecting to daemon: {}", e),
                _ => eprintln!("Unexpected response from daemon."),
            }
        }
        Commands::Export { path } => {
            match send_ipc_request(&IpcRequest::ListNotes) {
                Ok(IpcResponse::NotesList { notes }) => {
                    // Map to a clean public-facing representation
                    #[derive(serde::Serialize)]
                    struct ExportItem {
                        id: i64,
                        text: String,
                        color: String,
                        pos_x: i32,
                        pos_y: i32,
                        width: i32,
                        height: i32,
                    }
                    let export_list: Vec<ExportItem> = notes
                        .into_iter()
                        .map(|n| ExportItem {
                            id: n.id,
                            text: n.text,
                            color: n.color,
                            pos_x: n.pos_x,
                            pos_y: n.pos_y,
                            width: n.width,
                            height: n.height,
                        })
                        .collect();

                    let json = serde_json::to_string_pretty(&export_list)?;
                    fs::write(&path, json)?;
                    println!("Successfully exported notes to {}", path);
                }
                Ok(IpcResponse::Error { message }) => eprintln!("Error from daemon: {}", message),
                Err(e) => eprintln!("Error connecting to daemon: {}", e),
                _ => eprintln!("Unexpected response from daemon."),
            }
        }
        Commands::Import { path } => {
            let content = fs::read_to_string(&path)?;
            let import_items: Vec<ImportNote> = serde_json::from_str(&content)?;

            let mut success_count = 0;
            for item in import_items {
                let req = IpcRequest::NewNote {
                    text: item.text,
                    color: item.color.unwrap_or_else(|| DEFAULT_COLOR.to_string()),
                    pos_x: item.pos_x.unwrap_or(100),
                    pos_y: item.pos_y.unwrap_or(100),
                    width: item.width.unwrap_or(DEFAULT_WIDTH),
                    height: item.height.unwrap_or(DEFAULT_HEIGHT),
                };

                match send_ipc_request(&req) {
                    Ok(IpcResponse::NewNoteCreated { .. }) => success_count += 1,
                    Ok(IpcResponse::Error { message }) => eprintln!("Failed to import note: {}", message),
                    Err(e) => {
                        eprintln!("Error connecting to daemon: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            println!("Successfully imported {} notes.", success_count);
        }
    }

    Ok(())
}
