use futures_core::Stream;
use momo_callback_server::{create_callback_routes, start_callback_server, CallbackServerConfig};
use mtnmomo::MomoUpdates;
use futures_util::StreamExt;
use poem::listener::TcpListener;
use poem::middleware::AddData;
use poem::web::Data;
use poem::{handler, post, get, Body, Request, Response, Route, Server, EndpointExt};
use tracing::{error, info};
use std::pin::Pin;

use std::process::Command;
use std::thread::sleep;
use std::time::Duration;


use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_tracing() {
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    });
}

/// Helper for managing callback listeners during integration tests
pub struct CallbackTestHelper {
    pub sender: tokio::sync::mpsc::Sender<MomoUpdates>,
    pub receiver: tokio::sync::mpsc::Receiver<MomoUpdates>,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    shutdown_rx: Option<tokio::sync::oneshot::Receiver<()>>,
}

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





    pub async fn listen<'a>(&'a mut self) -> Result<impl Stream<Item = MomoUpdates> + 'a, Box<dyn std::error::Error>> {
        let config = CallbackServerConfig::default();
            let app = create_callback_routes()
        .with(poem::middleware::Tracing::default())
        .with(poem::middleware::Cors::new())
        .with(poem::middleware::Compression::default())
        .with(poem::middleware::RequestId::default())
        .with(AddData::new(self.sender.clone()));

        let bind_address = format!("{}:{}", config.host, config.http_port);
        info!("Binding server to address: {}", bind_address);

        println!("Starting MTN MoMo Callback Server on http://{}", bind_address);

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
                ).await {
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