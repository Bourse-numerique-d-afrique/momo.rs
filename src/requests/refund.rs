

use reqwest::Body;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Refund {
    pub amount: String,
    pub currency: String,
    #[serde(rename = "externalId")]
    pub external_id: String,
    #[serde(rename = "payerMessage")]
    pub payer_message: String,
    #[serde(rename = "payeeNote")]
    pub payee_note: String,
    #[serde(rename = "referenceIdToRefund")]
    pub reference_id_to_refund: String,
}

impl From<Refund> for Body {
    fn from(refund: Refund) -> Self {
        Body::from(serde_json::to_string(&refund).unwrap())
    }
}