#[tokio::main]
async fn main() {
    if let Err(e) = stickyboard::daemon::run().await {
        eprintln!("Daemon error: {}", e);
        std::process::exit(1);
    }
}
