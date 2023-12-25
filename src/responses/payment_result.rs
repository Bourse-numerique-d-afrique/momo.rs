


use serde::{Serialize, Deserialize};

use crate::errors::error::ErrorReason;


#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentResult {
    #[serde(rename = "referenceId")]
    pub reference_id: String,
    pub status: String,
    #[serde(rename = "financialTransactionId")]
    pub financial_transaction_id: String,
    pub reason: ErrorReason,
}