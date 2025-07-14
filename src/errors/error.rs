

use serde::{Serialize, Deserialize};

#[derive(thiserror::Error, Debug)]
pub enum MtnMomoError {
    #[error("HTTP error: {0}")]
    HttpError(String),
    
    #[error("Token error: {0}")]
    TokenError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorReason {
    pub code: String,
    pub message: String,
}