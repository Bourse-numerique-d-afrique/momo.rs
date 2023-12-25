


use serde::{Serialize, Deserialize};

use crate::{errors::error::ErrorReason, structs::party::Party};


#[derive(Debug, Serialize, Deserialize)]
pub struct PreApprovalResult {
    pub payer: Party,
    #[serde(rename = "payerCurrency")]
    pub payer_currency: String,
    #[serde(rename = "payerMessage")]
    pub payer_message: String,
    pub status: String,
    #[serde(rename = "expirationDateTime")]
    pub expiration_date_time: String,
    pub reason: ErrorReason,
}