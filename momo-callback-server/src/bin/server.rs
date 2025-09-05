// MTN MoMo Callback Server - Main executable
use std::error::Error;

use futures_util::StreamExt;
use tracing::info;
use tracing_subscriber;

use momo_callback_server::{start_callback_server, CallbackServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("MTN MoMo Callback Server starting...");

    // Load configuration
    let config = CallbackServerConfig::default();

    // Start the callback server
    let callback_stream = start_callback_server(config).await?;

    info!("Server is running. Press Ctrl+C to stop.");

    // Process incoming callbacks
    use futures_util::pin_mut;
    pin_mut!(callback_stream);
    while let Some(update) = callback_stream.next().await {
        info!("From: {}", update.remote_address);

        // Here you can add custom business logic to handle different callback types
        info!("Callback: {:?}", update.response);
    }

    Ok(())
}
