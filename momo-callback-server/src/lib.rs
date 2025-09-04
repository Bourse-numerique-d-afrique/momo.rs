//! # MTN MoMo Callback Server Library
//!
//! This library provides the core functionality for running a production-ready
//! callback server for MTN MoMo payment callbacks as a library component.
//!
//! The server listens for HTTP callbacks from the MTN MoMo API and provides
//! a stream-based interface for processing these callbacks in your application.
//!
//! ## Features
//!
//! - **Complete Callback Coverage**: Handles all MTN MoMo callback types
//! - **Stream-based Processing**: Process callbacks as an async stream
//! - **Graceful Shutdown**: Supports Ctrl+C shutdown signals
//! - **Built-in Middleware**: CORS, Compression, Tracing, Request ID
//! - **Health Check Endpoint**: Built-in `/health` endpoint
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use momo_callback_server::{CallbackServerConfig, start_callback_server};
//! use futures_util::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = CallbackServerConfig::default();
//!     let mut callback_stream = start_callback_server(config).await?;
//!
//!     println!("Server started, processing callbacks...");
//!
//!     while let Some(callback) = callback_stream.next().await {
//!         println!("Received callback: {:?}", callback.response);
//!         // Process the callback according to your business logic
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Library Usage
//!
//! The library exposes three main components:
//!
//! 1. [`CallbackServerConfig`] - Configuration struct for the server
//! 2. [`start_callback_server`] - Function to start the server and get a callback stream
//! 3. [`create_callback_routes`] - Function to create routes for custom integration
//!
//! ## Configuration
//!
//! The server can be configured using [`CallbackServerConfig`]:
//!
//! ```rust
//! use momo_callback_server::CallbackServerConfig;
//!
//! let config = CallbackServerConfig {
//!     http_port: 8500,
//!     host: "127.0.0.1".to_string(),
//! };
//! ```
//!
//! ## Stream Processing
//!
//! Callbacks are delivered as a stream of [`mtnmomo::MomoUpdates`] structs:
//!
//! ```rust
//! use mtnmomo::CallbackResponse;
//!
//! match &update.response {
//!     CallbackResponse::RequestToPaySuccess { 
//!         external_id, 
//!         amount, 
//!         currency, 
//!         .. 
//!     } => {
//!         // Handle successful payment
//!     }
//!     CallbackResponse::RequestToPayFailed { 
//!         external_id, 
//!         reason, 
//!         .. 
//!     } => {
//!         // Handle failed payment
//!     }
//!     _ => {
//!         // Handle other callback types
//!     }
//! }
//! ```

// Re-export only the essential types and functions
mod main_impl {
    include!("main.rs");
}

pub use main_impl::{
    CallbackServerConfig,
    start_callback_server,
    create_callback_routes,
};