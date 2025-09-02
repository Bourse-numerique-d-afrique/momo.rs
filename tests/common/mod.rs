use futures_core::Stream;
use momo_callback_server::{CallbackServerConfig, start_callback_server};
use mtnmomo::MomoUpdates;
use std::pin::Pin;

/// Helper for managing callback listeners during integration tests
pub struct CallbackTestHelper;

impl CallbackTestHelper {
    /// Create a new callback test helper that uses the production callback server
    /// This uses the existing momo-callback-server that listens on both ports 80 and 443
    pub async fn new() -> Result<Pin<Box<dyn Stream<Item = MomoUpdates> + Send>>, Box<dyn std::error::Error>> {
        // Create server configuration with default TLS settings
        let config = CallbackServerConfig::default();
        
        // Start the callback server (handles both HTTP and HTTPS)
        let stream = start_callback_server(config).await?;
        let stream = Box::pin(stream);
        Ok(stream)
    }
}