//! # MTN MoMo Callback Server
//!
//! A production-ready HTTP callback server for handling MTN MoMo payment callbacks.
//! This module provides functionality to run an HTTP server that processes
//! all types of MTN MoMo callbacks including payments, invoices, disbursements, and remittances.

use std::error::Error;

use futures_core::Stream;
use poem::listener::TcpListener;
use poem::middleware::AddData;
use poem::web::Data;
use poem::{get, handler, post, Body, EndpointExt, Request, Response, Route, Server};
use tokio::sync::mpsc::{self, Sender};
use tracing::{error, info, warn};

use crate::{CallbackResponse, MomoUpdates};

/// Configuration structure for the MTN MoMo callback server.
#[derive(Debug, Clone)]
pub struct CallbackServerConfig {
    /// HTTP port number for the server to bind to.
    pub http_port: u16,
    /// Host address to bind the server to.
    pub host: String,
}

impl Default for CallbackServerConfig {
    fn default() -> Self {
        Self {
            http_port: 8500,
            host: "127.0.0.1".to_string(),
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
) -> Result<Response, poem::Error> {
    let remote_address = req.remote_addr().to_string();
    let body_string = body.into_string().await?;

    info!("Received callback from {}", remote_address);
    info!("Raw callback body: {}", body_string);

    let response_result: Result<CallbackResponse, serde_json::Error> =
        serde_json::from_str(&body_string);

    match response_result {
        Ok(callback_response) => {
            let momo_updates = MomoUpdates {
                remote_address,
                response: callback_response,
            };

            if let Err(e) = sender.send(momo_updates).await {
                error!("Failed to send callback update: {}", e);
            } else {
                info!("Successfully processed callback");
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
            "/collection_request_to_pay",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/collection_request_to_withdraw_v1",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/collection_request_to_withdraw_v2",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/collection_invoice",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/collection_payment",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/collection_preapproval",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        // Disbursement callbacks
        .at(
            "/disbursement_deposit_v1",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/disbursement_deposit_v2",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/disbursement_refund_v1",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/disbursement_refund_v2",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/disbursement_transfer",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        // Remittance callbacks
        .at(
            "/remittance_cash_transfer",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        .at(
            "/remittance_transfer",
            post(mtn_callback_handler).put(mtn_callback_handler),
        )
        // Health check endpoint
        .at("/health", get(health_check))
}

/// Starts the MTN MoMo callback server with the specified configuration.
pub async fn start_callback_server(
    config: CallbackServerConfig,
) -> Result<impl Stream<Item = MomoUpdates>, Box<dyn Error>> {
    info!("Starting MTN MoMo Callback Server");
    info!("Host: {}, Port: {}", config.host, config.http_port);

    let (tx, mut rx) = mpsc::channel::<MomoUpdates>(100);

    // Create the application with routes and middleware
    let app = create_callback_routes()
        .with(poem::middleware::Tracing)
        .with(poem::middleware::Cors::new())
        .with(poem::middleware::Compression::default())
        .with(poem::middleware::RequestId::default())
        .with(AddData::new(tx.clone()));

    // Start the server on the HTTP port only
    let bind_address = format!("{}:{}", config.host, config.http_port);
    info!("Binding server to address: {}", bind_address);

    // Start HTTP server
    tokio::spawn(async move {
        let listener = TcpListener::bind(&bind_address);

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