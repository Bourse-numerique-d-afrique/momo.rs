use futures_core::Stream;
#[cfg(not(feature = "skip-integration-tests"))]
use futures_util::StreamExt;
use momo_callback_server::{create_callback_routes, CallbackServerConfig};
use mtnmomo::MomoUpdates;
#[cfg(not(feature = "skip-integration-tests"))]
use poem::listener::TcpListener;
#[cfg(not(feature = "skip-integration-tests"))]
use poem::middleware::AddData;
#[cfg(not(feature = "skip-integration-tests"))]
use poem::web::Data;
#[cfg(not(feature = "skip-integration-tests"))]
use poem::{get, handler, post, Body, EndpointExt, Request, Response, Route, Server};
#[cfg(not(feature = "skip-integration-tests"))]
use std::pin::Pin;
#[cfg(not(feature = "skip-integration-tests"))]
use tracing::{error, info};

#[cfg(not(feature = "skip-integration-tests"))]
use std::process::Command;
#[cfg(not(feature = "skip-integration-tests"))]
use std::thread::sleep;
#[cfg(not(feature = "skip-integration-tests"))]
use std::time::Duration;

use std::sync::Once;

static INIT: Once = Once::new();

#[cfg(not(feature = "skip-integration-tests"))]
pub fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    });
}

/// Helper for managing callback listeners during integration tests
#[cfg(not(feature = "skip-integration-tests"))]
pub struct CallbackTestHelper {
    pub sender: tokio::sync::mpsc::Sender<MomoUpdates>,
    pub receiver: tokio::sync::mpsc::Receiver<MomoUpdates>,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    shutdown_rx: Option<tokio::sync::oneshot::Receiver<()>>,
}

#[cfg(not(feature = "skip-integration-tests"))]
impl CallbackTestHelper {
    /// Create a new callback test helper that uses the production callback server
    /// This uses the existing momo-callback-server that listens on both ports 80 and 443
    pub async fn new() -> Result<CallbackTestHelper, Box<dyn std::error::Error>> {
        init_tracing();
        let (tx, rx) = tokio::sync::mpsc::channel::<MomoUpdates>(100);
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        Ok(CallbackTestHelper {
            sender: tx,
            receiver: rx,
            shutdown: Some(shutdown_tx),
            shutdown_rx: Some(shutdown_rx),
        })
    }

    pub async fn listen<'a>(
        &'a mut self,
    ) -> Result<impl Stream<Item = MomoUpdates> + 'a, Box<dyn std::error::Error>> {
        let config = CallbackServerConfig::default();
        let app = create_callback_routes()
            .with(poem::middleware::Tracing)
            .with(poem::middleware::Cors::new())
            .with(poem::middleware::Compression::default())
            .with(poem::middleware::RequestId::default())
            .with(AddData::new(self.sender.clone()));

        let bind_address = format!("{}:{}", config.host, config.http_port);
        info!("Binding server to address: {}", bind_address);

        println!(
            "Starting MTN MoMo Callback Server on http://{}",
            bind_address
        );

        // Start HTTP server with shutdown channel
        let shutdown_rx = self.shutdown_rx.take();

        println!("Starting server...");
        tokio::spawn(async move {
            let listener = TcpListener::bind(&bind_address);
            match Server::new(listener)
                .run_with_graceful_shutdown(
                    app,
                    async {
                        if let Some(rx) = shutdown_rx {
                            let _ = rx.await;
                            info!("Received shutdown signal, stopping server...");
                            println!("Received shutdown signal, stopping server...");
                        }
                    },
                    None,
                )
                .await
            {
                Ok(_) => info!("Server stopped successfully"),
                Err(e) => error!("Server error: {}", e),
            }
        });

        info!("MTN MoMo Callback Server started successfully");

        Ok(async_stream::stream! {
            while let Some(msg) = self.receiver.recv().await {
                yield msg;
            }
        })
    }

    pub async fn stop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.sender.clone().closed().await;
        self.receiver.close();
    }
}
