


use serde::{Serialize, Deserialize};



#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorReason {
    pub code: String,
    pub message: String
}