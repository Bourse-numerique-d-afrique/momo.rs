


use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentResult {
    #[serde(rename = "referenceId")]
    pub reference_id: String,
    pub status: String,
}