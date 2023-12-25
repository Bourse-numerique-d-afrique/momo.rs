


use serde::{Serialize, Deserialize};
use crate::{errors::error::ErrorReason, structs::party::Party};


#[derive(Debug, Serialize, Deserialize)]
pub struct RequestToPayResult {
    pub amount: String,
    pub currency: String,
    #[serde(rename = "financialTransactionId")]
    pub financial_transaction_id: String,
    #[serde(rename = "externalId")]
    pub external_id: String,
    pub payer: Party,
    #[serde(rename = "payerMessage")]
    pub payer_message: String,
    #[serde(rename = "payeeNote")]
    pub payee_note: String,
    pub status: String,
    pub reason: ErrorReason,
}