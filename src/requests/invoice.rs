use reqwest::Body;
use serde::{Serialize, Deserialize};

use crate::structs::party::Party;



#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceRequest {
    #[serde(rename = "externalId")]
    pub external_id: String,
    pub amount: String,
    pub currency: String,
    #[serde(rename = "validityDuration")]
    pub validity_duration: String,
    #[serde(rename = "intendedPayer")]
    pub intended_payer: Party,
    pub payee: Party,
    pub description: String,
}



impl From<InvoiceRequest> for Body {
    fn from(invoice_request: InvoiceRequest) -> Self {
        Body::from(serde_json::to_string(&invoice_request).unwrap())
    }
}