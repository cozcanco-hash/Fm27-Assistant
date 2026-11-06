//! Development host for protocol checks, without a WebView or a game installation.
use fm27_core::{assistant::Assistant, bridge::BridgeServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("Usage: bridge_host <isolated-data-directory>")?;
    let directory = std::path::PathBuf::from(path);
    let host = Assistant::open(&directory)?;
    let bridge = BridgeServer::start(host.clone(), &directory).await?;
    println!("FM27 Assistant bridge ready (protocol 1). Press Enter to stop.");
    tokio::task::spawn_blocking(|| {
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
    })
    .await?;
    bridge.stop().await;
    host.shutdown().await;
    Ok(())
}
