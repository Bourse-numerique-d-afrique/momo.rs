

use serde::{Serialize, Deserialize};

use crate::{errors::error::ErrorReason, structs::party::Party};


#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceResult {
    #[serde(rename = "referenceId")]
    pub reference_id: String,
    #[serde(rename = "externalId")]
    pub external_id: String,
    pub amount: String,
    pub currency: String,
    pub status: String,
    #[serde(rename = "paymentReference")]
    pub payment_reference: String,
    #[serde(rename = "invoiceId")]
    pub invoice_id: String,
    #[serde(rename = "experyDateTime")]
    pub expiry_date_time: String,
    #[serde(rename = "payerFirstName")]
    pub payee_first_name: String,
    #[serde(rename = "payerLastName")]
    pub payee_last_name: String,
    #[serde(rename = "errorReason")]
    pub error_reason: ErrorReason,
    #[serde(rename = "intendedPayer")]
    pub intended_payer: Party,
    pub description: String,
}