use reqwest::Body;
use serde::{Serialize, Deserialize};

use crate::structs::party::Party;



#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl InvoiceRequest {
    pub fn new(amount: String, currency: String, validity_duration: String, intended_payer: Party, payee: Party, description: String) -> Self {
        let external_id = uuid::Uuid::new_v4().to_string();
        InvoiceRequest {
            external_id,
            amount,
            currency,
            validity_duration,
            intended_payer,
            payee,
            description
        }
    }
    
}



impl From<InvoiceRequest> for Body {
    fn from(invoice_request: InvoiceRequest) -> Self {
        Body::from(serde_json::to_string(&invoice_request).unwrap())
    }
}

impl From<&InvoiceRequest> for Body {
    fn from(invoice_request: &InvoiceRequest) -> Self {
        Body::from(serde_json::to_string(invoice_request).unwrap())
    }
}