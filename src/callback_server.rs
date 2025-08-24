//! # MTN MoMo Callback Server
//!
//! A production-ready, TLS-enabled callback server for handling MTN MoMo payment callbacks.
//! This module provides functionality to run a secure HTTPS server that processes
//! all types of MTN MoMo callbacks including payments, invoices, disbursements, and remittances.

use std::env;
use std::error::Error;

use futures_core::Stream;
use poem::listener::{Listener, RustlsConfig, TcpListener};
use poem::middleware::AddData;
use poem::web::{Data, Path};
use poem::{get, handler, post, Body, EndpointExt, Request, Response, Route, Server};
use tokio::sync::mpsc::{self, Sender};
use tracing::{error, info, warn};

use crate::{CallbackResponse, CallbackType, MomoUpdates};

/// Configuration structure for the MTN MoMo callback server.
#[derive(Debug, Clone)]
pub struct CallbackServerConfig {
    /// Path to the TLS certificate file in PEM format.
    pub cert_path: String,
    /// Path to the TLS private key file in PEM format.
    pub key_path: String,
    /// Port number for the server to bind to.
    pub port: u16,
    /// Host address to bind the server to.
    pub host: String,
}

impl Default for CallbackServerConfig {
    fn default() -> Self {
        Self {
            cert_path: env::var("TLS_CERT_PATH").unwrap_or_else(|_| "cert.pem".to_string()),
            key_path: env::var("TLS_KEY_PATH").unwrap_or_else(|_| "key.pem".to_string()),
            port: 443,
            host: "0.0.0.0".to_string(),
        }
    }
}

/// Health check endpoint handler.
#[handler]
async fn health_check() -> &'static str {
    "OK"
}

/// Primary callback handler for all MTN MoMo callback requests.
#[handler]
async fn mtn_callback_handler(
    req: &Request,
    mut body: Body,
    sender: Data<&Sender<MomoUpdates>>,
    Path(callback_type): Path<String>,
) -> Result<Response, poem::Error> {
    let remote_address = req.remote_addr().to_string();
    let body_string = body.into_string().await?;

    info!(
        "Received callback from {}: {}",
        remote_address, callback_type
    );

    let response_result: Result<CallbackResponse, serde_json::Error> =
        serde_json::from_str(&body_string);

    match response_result {
        Ok(callback_response) => {
            let momo_updates = MomoUpdates {
                remote_address,
                response: callback_response,
                update_type: CallbackType::from_string(&callback_type),
            };

            if let Err(e) = sender.send(momo_updates).await {
                error!("Failed to send callback update: {}", e);
            } else {
                info!("Successfully processed {} callback", callback_type);
            }
        }
        Err(e) => {
            warn!(
                "Failed to parse callback body: {} - Body: {}",
                e, body_string
            );
        }
    }

    Ok(Response::builder()
        .status(poem::http::StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(r#"{"status": "success", "message": "Callback received successfully"}"#))
}

/// Creates and configures all callback routes for MTN MoMo services.
fn create_callback_routes() -> Route {
    Route::new()
        // Collection callbacks
        .at(
            "/collection_request_to_pay/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/collection_request_to_withdraw_v1/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/collection_request_to_withdraw_v2/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/collection_invoice/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/collection_payment/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/collection_preapproval/:callback_type",
            post(mtn_callback_handler),
        )
        // Disbursement callbacks
        .at(
            "/disbursement_deposit_v1/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/disbursement_deposit_v2/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/disbursement_refund_v1/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/disbursement_refund_v2/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/disbursement_transfer/:callback_type",
            post(mtn_callback_handler),
        )
        // Remittance callbacks
        .at(
            "/remittance_cash_transfer/:callback_type",
            post(mtn_callback_handler),
        )
        .at(
            "/remittance_transfer/:callback_type",
            post(mtn_callback_handler),
        )
        // Health check endpoint
        .at("/health", get(health_check))
}

/// Loads and validates TLS configuration from certificate and key files.
async fn load_tls_config(config: &CallbackServerConfig) -> Result<RustlsConfig, Box<dyn Error>> {
    info!("Loading TLS certificate from: {}", config.cert_path);
    info!("Loading TLS private key from: {}", config.key_path);

    let cert_data = std::fs::read(&config.cert_path)?;
    let key_data = std::fs::read(&config.key_path)?;

    let tls_config = RustlsConfig::new()
        .cert(cert_data)
        .key(key_data);

    info!("TLS configuration loaded successfully");
    Ok(tls_config)
}

/// Starts the MTN MoMo callback server with the specified configuration.
pub async fn start_callback_server(
    config: CallbackServerConfig,
) -> Result<impl Stream<Item = MomoUpdates>, Box<dyn Error>> {
    info!("Starting MTN MoMo Callback Server");
    info!("Host: {}, Port: {}", config.host, config.port);

    let (tx, mut rx) = mpsc::channel::<MomoUpdates>(100);

    // Load TLS configuration
    let tls_config = load_tls_config(&config).await?;

    // Create the application with routes and middleware
    let app = create_callback_routes()
        .with(poem::middleware::Tracing)
        .with(poem::middleware::Cors::new())
        .with(poem::middleware::Compression::default())
        .with(poem::middleware::RequestId::default())
        .with(AddData::new(tx));

    // Start the server
    let bind_address = format!("{}:{}", config.host, config.port);
    info!("Binding to address: {}", bind_address);

    tokio::spawn(async move {
        let listener = TcpListener::bind(&bind_address).rustls(tls_config);

        match Server::new(listener)
            .run_with_graceful_shutdown(
                app,
                async {
                    tokio::signal::ctrl_c().await.ok();
                    info!("Received shutdown signal, stopping server...");
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
        while let Some(msg) = rx.recv().await {
            yield msg;
        }
    })
}
