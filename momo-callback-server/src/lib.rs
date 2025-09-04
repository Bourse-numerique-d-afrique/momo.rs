//! # MTN MoMo Callback Server Library
//!
//! This library provides the core functionality for running a production-ready,
//! TLS-enabled callback server for MTN MoMo payment callbacks as a library component.

// Re-export only the essential types and functions
mod main_impl {
    include!("main.rs");
}

pub use main_impl::{
    CallbackServerConfig,
    start_callback_server,
    create_callback_routes,
};