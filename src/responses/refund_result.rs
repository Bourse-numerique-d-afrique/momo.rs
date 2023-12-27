

use serde::{Serialize, Deserialize};

use crate::{structs::party::Party, errors::error::ErrorReason};


#[derive(Debug, Serialize, Deserialize)]
pub struct  RefundResult {
    pub amount : String,
    pub currency : String,
    #[serde(rename = "financialTransactionId")]
    pub financial_transaction_id : String,
    #[serde(rename = "externalId")]
    pub external_id : String,
    pub payee : Party,
    #[serde(rename = "payerMessage")]
    pub payer_message : String,
    #[serde(rename = "payeeNote")]
    pub payee_note : String,
    pub status : String,
    pub reason : ErrorReason,
}