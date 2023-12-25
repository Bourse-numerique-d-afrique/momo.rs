use reqwest::Body;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceDelete {
    #[serde(rename = "externalId")]
    pub external_id: String,
}



impl From<InvoiceDelete> for Body {
    fn from(invoice_delete: InvoiceDelete) -> Self {
        Body::from(serde_json::to_string(&invoice_delete).unwrap())
    }
}